use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::path::PathBuf;

use crate::config::ConfigManager;
use crate::server::SessionRegistry;

#[derive(Debug, Deserialize, Default)]
pub struct HookInput {
    /// 会话 ID
    #[serde(default)]
    pub session_id: String,

    /// 用户输入的 prompt（仅 UserPromptSubmit）
    #[serde(default)]
    pub prompt: Option<String>,

    /// 当前工作目录
    #[serde(default)]
    pub cwd: Option<PathBuf>,

    /// transcript 文件路径（仅 Stop）
    #[serde(default)]
    pub transcript_path: Option<PathBuf>,

    /// 是否在 Stop Hook 循环中（仅 Stop）
    #[serde(default)]
    pub stop_hook_active: bool,

    /// 用户自定义的提示文本（可用于自定义 hook 提示信息）
    #[serde(default)]
    pub user_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "decision", rename_all = "lowercase")]
pub enum HookDecision {
    Approve,
    Block { reason: String },
}

impl HookDecision {
    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("序列化 HookDecision 失败")
    }

    /// 输出到 stdout（供 Hook 脚本使用）
    pub fn output(&self) -> Result<()> {
        println!("{}", self.to_json()?);
        Ok(())
    }
}

pub struct HookHandler;

impl HookHandler {
    /// 从 stdin 读取输入并处理
    pub fn handle_from_stdin(hook_type: &str) -> Result<HookDecision> {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("读取 stdin 失败")?;

        let input: HookInput =
            serde_json::from_str(&buffer).context("解析 Hook 输入失败")?;

        Self::handle(hook_type, input)
    }

    /// 处理 Hook 事件
    fn handle(hook_type: &str, input: HookInput) -> Result<HookDecision> {
        match hook_type {
            "user-prompt-submit" => handle_user_prompt_submit(input),
            "stop" => handle_stop(input),
            "notification" => handle_notification(input),
            _ => {
                anyhow::bail!("未知的 Hook 类型: {}", hook_type);
            }
        }
    }
}

fn handle_user_prompt_submit(input: HookInput) -> Result<HookDecision> {
    let config_mgr = ConfigManager::new()?;

    // 检查是否是远程控制命令
    match input.prompt.as_deref() {
        Some(">remote-on") => handle_remote_on(&config_mgr),
        Some(">remote-off") => handle_remote_off(&config_mgr, &input.session_id),
        Some(">remote-status") => handle_remote_status(&config_mgr),
        _ => {
            // 其他 prompt 正常通过
            Ok(HookDecision::Approve)
        }
    }
}

fn handle_remote_on(config_mgr: &ConfigManager) -> Result<HookDecision> {
    config_mgr.enable_remote()?;

    let config = config_mgr.load()?;

    let reason = format!(
        "✅ 远程模式已启用\n\n\
         配置信息：\n\
         - 目标微信：{}\n\
         - 监听地址：{}\n\
         - 标记文件：~/.gewe-cc/remote.lock\n\n\
         任务完成后将自动等待微信指令。",
        config.notification.wxid, config.notification.listen
    );

    Ok(HookDecision::Block { reason })
}

fn handle_remote_off(config_mgr: &ConfigManager, session_id: &str) -> Result<HookDecision> {
    // 会话级别关闭远程模式（不禁用全局模式）
    let mut extra = String::new();
    if session_id.trim().is_empty() {
        extra.push_str("\n\n⚠️ 未获取到会话 ID，无法记录会话禁用状态。");
    } else {
        config_mgr.disable_session(session_id)?;
    }

    let reason = format!(
        "✅ 当前会话的远程模式已关闭\n\n\
         会话 ID: {}\n\n\
         全局远程模式仍处于启用状态，后续新任务将继续进入远程控制。{}",
        session_id, extra
    );

    Ok(HookDecision::Block { reason })
}

fn handle_remote_status(config_mgr: &ConfigManager) -> Result<HookDecision> {
    let config = config_mgr.load()?;
    let enabled = config_mgr.is_remote_enabled();

    let status = if enabled { "✅ 已启用" } else { "❌ 未启用" };

    let reason = if enabled {
        format!(
            "📊 远程模式状态\n\n\
             状态：{}\n\
             配置：\n\
             - 目标微信：{}\n\
             - 监听地址：{}\n\
             - 标记文件：~/.gewe-cc/remote.lock",
            status, config.notification.wxid, config.notification.listen
        )
    } else {
        format!(
            "📊 远程模式状态\n\n\
             状态：{}\n\n\
             使用 >remote-on 启用远程模式。",
            status
        )
    };

    Ok(HookDecision::Block { reason })
}

fn handle_stop(input: HookInput) -> Result<HookDecision> {
    let config_mgr = ConfigManager::new()?;

    // 1. 防止无限循环：如果已经在 Stop Hook 中，直接批准
    if input.stop_hook_active {
        return Ok(HookDecision::Approve);
    }

    // 2. 检查全局远程模式是否启用
    if !config_mgr.is_remote_enabled() {
        return Ok(HookDecision::Approve);
    }

    // 2.1 如果该会话已被禁用，直接允许停止
    if config_mgr.is_session_disabled(&input.session_id) {
        return Ok(HookDecision::Approve);
    }

    // 3. 注册 session（如果提供了 transcript_path）
    if let Some(transcript_path) = &input.transcript_path
        && let Ok(registry) = SessionRegistry::new()
    {
        // 忽略注册失败（不应阻塞主流程）
        let _ = registry.register(input.session_id.clone(), transcript_path.clone());
    }

    // 4. 远程模式已启用，阻止停止并激活 Skill
    let project = input
        .cwd
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // 如果用户提供了自定义提示文本，使用自定义文本
    let reason = if let Some(custom_prompt) = input.user_prompt {
        format!(
            "🔄 远程模式已启用\n\n\
             {}\n\n\
             上下文信息：\n\
             - 项目: {}\n\
             - 目录: {}\n\
             - 会话: {}",
            custom_prompt,
            project,
            input
                .cwd
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            input.session_id
        )
    } else {
        // 使用默认提示 - 明确指示激活 skill
        format!(
            "🔄 远程模式已启用 - 如果未激活，请立即激活 remote-control skill\n\n\
             执行：/gewe-cc:remote-control\n\n\
             上下文信息：\n\
             - 项目: {}\n\
             - 目录: {}\n\
             - 会话: {}",
            project,
            input
                .cwd
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            input.session_id
        )
    };

    Ok(HookDecision::Block { reason })
}

fn handle_notification(input: HookInput) -> Result<HookDecision> {
    let config_mgr = ConfigManager::new()?;

    // 只在远程模式启用时发送通知
    if !config_mgr.is_remote_enabled() {
        return Ok(HookDecision::Approve);
    }

    let config = config_mgr.load()?;

    // 获取项目名
    let project = input
        .cwd
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // 构建通知消息
    let message = format!(
        "【Claude Code】\n\
         ⚠️ 会话可能挂起\n\
         📁 项目: {}\n\
         🕐 检测到 60 秒以上无响应\n\n\
         请检查终端是否在等待输入。\n\
         会话 ID: {}",
        project,
        input.session_id
    );

    // 使用 gewe-cli 直接发送文本消息（不等待回复）
    let output = std::process::Command::new(&config.gewe_cli.command)
        .args([
            "message",
            "send-text",
            "--to",
            &config.notification.wxid,
            "--content",
            &message,
        ])
        .output();

    // 忽略发送失败（兜底功能，不应阻塞流程）
    if let Err(e) = output {
        eprintln!("⚠️ 发送空闲通知失败: {}", e);
    }

    // 始终允许通知继续
    Ok(HookDecision::Approve)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_decision_serialization() {
        let decision = HookDecision::Approve;
        let json = decision.to_json().unwrap();
        assert_eq!(json, r#"{"decision":"approve"}"#);

        let decision = HookDecision::Block {
            reason: "测试原因".to_string(),
        };
        let json = decision.to_json().unwrap();
        assert!(json.contains("block"));
        assert!(json.contains("测试原因"));
    }

    #[test]
    fn test_hook_input_deserialization() {
        let json = r#"{"session_id":"test-123","prompt":">remote-on"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "test-123");
        assert_eq!(input.prompt, Some(">remote-on".to_string()));
    }
}
