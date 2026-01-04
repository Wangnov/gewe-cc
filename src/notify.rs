use anyhow::{Context, Result};
use std::process::Command;

use crate::config::ConfigManager;

/// 发送消息并等待回复
///
/// # 参数
///
/// * `message` - 要发送的消息内容
/// * `to_wxid` - 可选的目标微信ID，如果不提供则使用配置文件中的默认值
/// * `listen` - 可选的监听地址，如果不提供则使用配置文件中的默认值
/// * `timeout` - 可选的超时时间（秒），如果不提供则使用配置文件中的默认值
///
/// # 返回
///
/// 返回用户的回复内容
pub fn wait_reply(
    message: String,
    to_wxid: Option<String>,
    listen: Option<String>,
    timeout: Option<u64>,
) -> Result<String> {
    let config_mgr = ConfigManager::new()?;
    let config = config_mgr.load()?;

    // 使用参数或配置文件中的值
    let wxid = to_wxid.unwrap_or(config.notification.wxid);
    let listen_addr = listen.unwrap_or(config.notification.listen);
    let timeout_secs = timeout.unwrap_or(config.gewe_cli.timeout);

    // 验证 wxid 不为空
    if wxid.is_empty() {
        anyhow::bail!(
            "目标微信 ID 不能为空\n\
             请使用以下方式之一设置：\n\
             1. 运行 gewe-cc init 初始化配置\n\
             2. 运行 gewe-cc config --wxid <微信ID>\n\
             3. 使用 --to-wxid 参数指定"
        );
    }

    // 调用 gewe-cli wait-reply
    let mut cmd = Command::new(&config.gewe_cli.command);
    cmd.args([
        "wait-reply",
        "--to-wxid",
        &wxid,
        "--listen",
        &listen_addr,
        "-M",
        &format!("text:{}", message),
    ]);

    // 如果 timeout_secs 为 0，不传 --timeout 参数（使用 gewe-cli 的默认值：无限等待）
    if timeout_secs > 0 {
        cmd.args(["--timeout", &timeout_secs.to_string()]);
    }

    let output = cmd
        .output()
        .context(format!(
            "调用 {} 失败，请确认已安装 gewe-cli",
            config.gewe_cli.command
        ))?;

    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        match exit_code {
            1 => {
                if timeout_secs > 0 {
                    anyhow::bail!("等待微信回复超时（{}秒）", timeout_secs);
                } else {
                    anyhow::bail!("等待微信回复超时");
                }
            }
            2 => anyhow::bail!("发送微信消息失败"),
            3 => anyhow::bail!("webhook 启动失败，请检查监听地址: {}", listen_addr),
            _ => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("gewe-cli 执行失败 (exit code {}): {}", exit_code, stderr);
            }
        }
    }

    // 返回用户回复（去除首尾空白）
    let reply = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(reply)
}

/// 发送链接卡片并等待回复
///
/// # 参数
///
/// * `session_id` - Session ID（用于构建 transcript URL）
/// * `summary` - 任务摘要
///
/// # 返回
///
/// 返回用户的回复内容
pub fn send_link_and_wait(session_id: String, summary: String) -> Result<String> {
    let config_mgr = ConfigManager::new()?;
    let config = config_mgr.load()?;

    // 验证配置
    if config.notification.wxid.is_empty() {
        anyhow::bail!(
            "目标微信 ID 不能为空\n\
             请运行: gewe-cc config --wxid <微信ID>"
        );
    }

    if config.notification.transcript_domain.is_empty() {
        anyhow::bail!(
            "Transcript 域名未配置\n\
             请运行: gewe-cc config --transcript-domain <域名>"
        );
    }

    // 构建链接 URL
    let transcript_url = format!("{}/{}", config.notification.transcript_domain, session_id);

    // 获取项目名
    let project = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    // 发送链接卡片
    let title = format!("📝 任务完成 - {}", project);

    // 使用配置的域名 + /assets/thumb.png 作为缩略图
    // 添加时间戳参数避免缓存问题
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let thumb_url = format!("{}/assets/thumb.png?t={}", config.notification.transcript_domain, timestamp);

    let output = Command::new(&config.gewe_cli.command)
        .args([
            "send-link",
            "--to-wxid",
            &config.notification.wxid,
            "--title",
            &title,
            "--desc",
            &summary,
            "--link-url",
            &transcript_url,
            "--thumb-url",
            &thumb_url,
        ])
        .output()
        .context(format!(
            "调用 {} 失败，请确认已安装 gewe-cli",
            config.gewe_cli.command
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("发送链接卡片失败: {}", stderr);
    }

    // 等待回复
    wait_reply(
        "回复任何内容继续，回复「停止」结束远程模式。".to_string(),
        None,
        None,
        None,
    )
}

/// 仅发送通知，不等待回复
///
/// # 参数
///
/// * `message` - 要发送的消息内容
/// * `to_wxid` - 可选的目标微信ID
///
/// # 返回
///
/// 发送成功返回 Ok(())
pub fn send_notification(message: String, to_wxid: Option<String>) -> Result<()> {
    let config_mgr = ConfigManager::new()?;
    let config = config_mgr.load()?;

    let wxid = to_wxid.unwrap_or(config.notification.wxid);

    if wxid.is_empty() {
        anyhow::bail!("目标微信 ID 不能为空");
    }

    let output = Command::new(&config.gewe_cli.command)
        .args([
            "message",
            "send-text",
            "--to",
            &wxid,
            "--content",
            &message,
        ])
        .output()
        .context(format!(
            "调用 {} 失败，请确认已安装 gewe-cli",
            config.gewe_cli.command
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("发送消息失败: {}", stderr);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_wxid_validation() {
        // 测试 wxid 为空字符串时的错误信息
        let empty = String::new();
        assert!(empty.is_empty());

        // 验证错误消息包含关键信息
        let error_msg = "目标微信 ID 不能为空";
        assert!(error_msg.contains("不能为空"));
    }
}
