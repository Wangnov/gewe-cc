use anyhow::{Context, Result};
use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
    body::Body,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path as StdPath, PathBuf};
use std::sync::{Arc, RwLock};
use tokio_util::io::ReaderStream;

use crate::config::ConfigManager;
use crate::transcript;

/// Session 映射管理器
#[derive(Debug, Clone)]
pub struct SessionRegistry {
    sessions: Arc<RwLock<HashMap<String, PathBuf>>>,
    sessions_file: PathBuf,
}

impl SessionRegistry {
    pub fn new() -> Result<Self> {
        let config_mgr = ConfigManager::new()?;
        let config_dir = config_mgr.config_file().parent().unwrap().to_path_buf();
        let sessions_file = config_dir.join("sessions.json");

        // 读取现有的 session 映射
        let sessions = if sessions_file.exists() {
            let content = fs::read_to_string(&sessions_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            sessions: Arc::new(RwLock::new(sessions)),
            sessions_file,
        })
    }

    /// 注册一个 session（同步版本）
    pub fn register(&self, session_id: String, transcript_path: PathBuf) -> Result<()> {
        let mut sessions = self.sessions.write().map_err(|e| {
            anyhow::anyhow!("获取写锁失败: {}", e)
        })?;

        sessions.insert(session_id, transcript_path);

        // 保存到文件
        self.save_sessions(&sessions)?;

        Ok(())
    }

    /// 获取 session 的 transcript 路径（同步版本）
    pub fn get(&self, session_id: &str) -> Option<PathBuf> {
        if let Ok(sessions) = self.sessions.read()
            && let Some(path) = sessions.get(session_id)
        {
            return Some(path.clone());
        }

        // 可能有新的 sessions.json 写入，尝试重新加载
        if let Ok(content) = fs::read_to_string(&self.sessions_file)
            && let Ok(updated) = serde_json::from_str::<HashMap<String, PathBuf>>(&content)
            && let Ok(mut sessions) = self.sessions.write()
        {
            *sessions = updated;
        }

        let sessions = self.sessions.read().ok()?;
        sessions.get(session_id).cloned()
    }

    /// 保存 sessions 到文件
    fn save_sessions(&self, sessions: &HashMap<String, PathBuf>) -> Result<()> {
        let content = serde_json::to_string_pretty(sessions)?;
        fs::write(&self.sessions_file, content)?;
        Ok(())
    }
}

/// 启动 HTTP 服务器
pub async fn start_server(port: u16) -> Result<()> {
    let registry = SessionRegistry::new()?;

    let app = Router::new()
        .route("/{session_id}", get(transcript_handler))
        .route("/assets/{*path}", get(static_handler))
        .route("/health", get(health_handler))
        .with_state(registry);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("绑定地址失败: {}", addr))?;

    println!("🚀 HTTP 服务器已启动: http://{}", addr);
    println!("   本地访问: http://localhost:{}", port);
    println!("   配置 frpc 转发后可通过域名访问");

    axum::serve(listener, app)
        .await
        .context("HTTP 服务器运行失败")?;

    Ok(())
}

/// Transcript 路由处理
async fn transcript_handler(
    Path(session_id): Path<String>,
    axum::extract::State(registry): axum::extract::State<SessionRegistry>,
) -> impl IntoResponse {
    // 尝试从注册表获取路径
    let transcript_path = if let Some(path) = registry.get(&session_id) {
        path
    } else {
        // 如果注册表中没有，尝试从 Claude Code 的默认路径推导
        match infer_transcript_path(&session_id) {
            Some(path) if path.exists() => path,
            _ => {
                return (
                    StatusCode::NOT_FOUND,
                    Html(format!(
                        r#"<!DOCTYPE html>
<html>
<head><title>Session Not Found</title></head>
<body>
    <h1>❌ Session 不存在</h1>
    <p>Session ID: <code>{}</code></p>
    <p>请检查 Session ID 是否正确</p>
</body>
</html>"#,
                        session_id
                    )),
                )
                    .into_response();
            }
        }
    };

    // 检查文件是否存在
    if !transcript_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Html(format!(
                r#"<!DOCTYPE html>
<html>
<head><title>Transcript Not Found</title></head>
<body>
    <h1>❌ Transcript 文件不存在</h1>
    <p>Session ID: <code>{}</code></p>
    <p>路径: <code>{}</code></p>
</body>
</html>"#,
                session_id,
                transcript_path.display()
            )),
        )
            .into_response();
    }

    // 解析 transcript
    let messages = match transcript::parse_transcript(&transcript_path) {
        Ok(msgs) => msgs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>Parse Error</title></head>
<body>
    <h1>❌ 解析 Transcript 失败</h1>
    <p>Session ID: <code>{}</code></p>
    <p>错误: <code>{}</code></p>
</body>
</html>"#,
                    session_id, e
                )),
            )
                .into_response();
        }
    };

    // 渲染 HTML
    let html = transcript::render_to_html(&messages, &session_id);

    (StatusCode::OK, Html(html)).into_response()
}

/// 健康检查
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// 静态文件服务
async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    let assets_dir = get_assets_dir();

    // 确保资源目录存在
    if let Err(e) = fs::create_dir_all(&assets_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("创建资源目录失败: {}", e)
        ).into_response();
    }

    let request_path = StdPath::new(&path);
    if !is_safe_relative_path(request_path) {
        return (StatusCode::FORBIDDEN, "禁止访问").into_response();
    }

    let file_path = assets_dir.join(request_path);

    // 检查文件是否存在
    if !file_path.exists() || !file_path.is_file() {
        return (StatusCode::NOT_FOUND, "文件不存在").into_response();
    }

    // 进一步防止符号链接逃逸
    let assets_canon = match fs::canonicalize(&assets_dir) {
        Ok(path) => path,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "资源目录不可用").into_response(),
    };
    let file_canon = match fs::canonicalize(&file_path) {
        Ok(path) => path,
        Err(_) => return (StatusCode::NOT_FOUND, "文件不存在").into_response(),
    };
    if !file_canon.starts_with(&assets_canon) {
        return (StatusCode::FORBIDDEN, "禁止访问").into_response();
    }

    // 读取文件
    let file = match tokio::fs::File::open(&file_path).await {
        Ok(file) => file,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "无法读取文件").into_response(),
    };

    // 根据文件扩展名设置 Content-Type
    let content_type = match file_path.extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };

    // 转换为流
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .unwrap()
        .into_response()
}

/// 获取资源目录路径
fn get_assets_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(".gewe-cc")
        .join("assets")
}

fn is_safe_relative_path(path: &StdPath) -> bool {
    path.components().all(|component| matches!(component, Component::Normal(_)))
}

/// 从 session_id 推导 transcript 路径
///
/// Claude Code 的 transcript 路径通常在：
/// ~/.claude/projects/{project_hash}/{session_id}.jsonl
fn infer_transcript_path(session_id: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let projects_dir = home.join(".claude/projects");

    if !projects_dir.exists() {
        return None;
    }

    // 遍历所有项目目录，查找匹配的 session_id.jsonl
    for entry in fs::read_dir(&projects_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.is_dir() {
            let transcript_file = path.join(format!("{}.jsonl", session_id));
            if transcript_file.exists() {
                return Some(transcript_file);
            }
        }
    }

    None
}
