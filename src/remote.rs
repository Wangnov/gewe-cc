use anyhow::Result;
use colored::*;
use crate::config::ConfigManager;

pub fn handle_on() -> Result<()> {
    let config_mgr = ConfigManager::new()?;
    config_mgr.enable_remote()?;

    let config = config_mgr.load()?;

    println!("{}", "═══════════════════════════════════════".green());
    println!("{}", "  ✅ 远程模式已启用".green().bold());
    println!("{}", "═══════════════════════════════════════".green());
    println!();
    println!("{}", "配置信息:".bright_white().bold());
    println!("  {} {}", "目标微信:".dimmed(), config.notification.wxid);
    println!("  {} {}", "监听地址:".dimmed(), config.notification.listen);
    println!("  {} ~/.gewe-cc/remote.lock", "标记文件:".dimmed());
    println!();
    println!("{}", "任务完成后将自动等待微信指令。".dimmed());
    println!();

    Ok(())
}

pub fn handle_off(session_id: Option<String>) -> Result<()> {
    // 如果提供了 session_id，只关闭当前会话
    if let Some(sid) = session_id {
        let config_mgr = crate::config::ConfigManager::new()?;
        config_mgr.disable_session(&sid)?;

        println!("{}", "═══════════════════════════════════════".cyan());
        println!("{}", "  🛑 会话已关闭".cyan().bold());
        println!("{}", "═══════════════════════════════════════".cyan());
        println!();
        println!("  会话 ID: {}", sid.dimmed());
        println!();
        println!("{}", "本会话已结束，但全局远程模式仍处于启用状态。".dimmed());
        println!("{}", "后续新任务仍将进入远程控制流程。".dimmed());
        println!();
        return Ok(());
    }

    // 否则关闭全局远程模式
    let config_mgr = crate::config::ConfigManager::new()?;
    config_mgr.disable_remote()?;

    println!("{}", "═══════════════════════════════════════".yellow());
    println!("{}", "  ❌ 远程模式已禁用".yellow().bold());
    println!("{}", "═══════════════════════════════════════".yellow());
    println!();
    println!("{}", "任务完成后将正常停止，不再等待微信指令。".dimmed());
    println!();

    Ok(())
}

pub fn handle_status() -> Result<()> {
    let config_mgr = ConfigManager::new()?;
    let enabled = config_mgr.is_remote_enabled();

    println!("{}", "═══════════════════════════════════════".cyan());
    println!("{}", "  📊 远程模式状态".cyan().bold());
    println!("{}", "═══════════════════════════════════════".cyan());
    println!();

    if enabled {
        let config = config_mgr.load()?;
        println!("  {}: {}", "状态".bright_white().bold(), "✅ 已启用".green());
        println!();
        println!("  {}:", "配置".bright_white().bold());
        println!("    {} {}", "目标微信:".dimmed(), config.notification.wxid);
        println!("    {} {}", "监听地址:".dimmed(), config.notification.listen);
        println!("    {} ~/.gewe-cc/remote.lock", "标记文件:".dimmed());
        println!();
        println!("  {} gewe-cc off", "禁用:".dimmed());
    } else {
        println!("  {}: {}", "状态".bright_white().bold(), "❌ 未启用".red());
        println!();
        println!("  {} gewe-cc on", "启用:".dimmed());
    }

    println!();

    Ok(())
}

pub fn handle_config(wxid: Option<String>, listen: Option<String>, timeout: Option<u64>, transcript_domain: Option<String>) -> Result<()> {
    let config_mgr = ConfigManager::new()?;

    // 检查是否提供了至少一个参数
    if wxid.is_none() && listen.is_none() && timeout.is_none() && transcript_domain.is_none() {
        println!("{}", "═══════════════════════════════════════".yellow());
        println!("{}", "  ⚙️  配置管理".yellow().bold());
        println!("{}", "═══════════════════════════════════════".yellow());
        println!();

        let config = config_mgr.load()?;

        println!("{}", "当前配置:".bright_white().bold());
        println!("  {} {}", "目标微信:".dimmed(), config.notification.wxid);
        println!("  {} {}", "监听地址:".dimmed(), config.notification.listen);
        println!("  {} {}", "超时时间:".dimmed(), if config.gewe_cli.timeout == 0 {
            "无限等待".to_string()
        } else {
            format!("{} 秒", config.gewe_cli.timeout)
        });
        println!("  {} {}", "Transcript域名:".dimmed(),
            if config.notification.transcript_domain.is_empty() {
                "未配置".to_string()
            } else {
                config.notification.transcript_domain.clone()
            }
        );
        println!("  {} {}", "配置文件:".dimmed(), config_mgr.config_file().display());
        println!();
        println!("{}", "修改配置:".bright_white().bold());
        println!("  gewe-cc config --wxid <新的微信ID>");
        println!("  gewe-cc config --listen <新的监听地址>");
        println!("  gewe-cc config --timeout <超时秒数>  # 0 表示无限等待");
        println!("  gewe-cc config --transcript-domain <域名>");
        println!("  gewe-cc config --wxid <微信ID> --listen <监听地址> --timeout <秒数>");
        println!();

        return Ok(());
    }

    // 更新配置
    config_mgr.update_notification(wxid.clone(), listen.clone(), transcript_domain.clone())?;

    // 更新 timeout
    if let Some(timeout_val) = timeout {
        let mut config = config_mgr.load()?;
        config.gewe_cli.timeout = timeout_val;
        config_mgr.save(&config)?;
    }

    println!("{}", "═══════════════════════════════════════".green());
    println!("{}", "  ✅ 配置已更新".green().bold());
    println!("{}", "═══════════════════════════════════════".green());
    println!();

    if let Some(wxid) = wxid {
        println!("  {} {}", "目标微信:".dimmed(), wxid);
    }

    if let Some(listen) = listen {
        println!("  {} {}", "监听地址:".dimmed(), listen);
    }

    if let Some(timeout_val) = timeout {
        println!("  {} {}", "超时时间:".dimmed(), if timeout_val == 0 {
            "无限等待".to_string()
        } else {
            format!("{} 秒", timeout_val)
        });
    }

    if let Some(domain) = transcript_domain {
        println!("  {} {}", "Transcript域名:".dimmed(), domain);
    }

    println!();

    Ok(())
}
