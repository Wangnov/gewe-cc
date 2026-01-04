use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Input};
use std::fs;
use std::process::Command;

use crate::config::{Config, ConfigManager};

pub fn run() -> Result<()> {
    print_banner();

    println!("正在检查环境...\n");

    let deps = check_dependencies();

    if !deps.all_satisfied() {
        print_installation_guide(&deps);
        return Ok(());
    }

    println!("{}\n", "✅ 所有依赖已满足".green());

    create_config()?;

    print_success_message();

    Ok(())
}

fn print_banner() {
    println!(
        "{}",
        "╔═══════════════════════════════════════╗".bright_cyan()
    );
    println!(
        "{}",
        "║       gewe-cc 初始化向导              ║".bright_cyan()
    );
    println!(
        "{}",
        "║   Claude Code 远程协作模式工具        ║".bright_cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════╝".bright_cyan()
    );
    println!();
}

struct DependencyStatus {
    gewe_cli: Option<String>,
    claude_code: Option<String>,
    plugin: bool,
}

impl DependencyStatus {
    fn all_satisfied(&self) -> bool {
        self.gewe_cli.is_some() && self.claude_code.is_some() && self.plugin
    }
}

fn check_dependencies() -> DependencyStatus {
    let mut status = DependencyStatus {
        gewe_cli: None,
        claude_code: None,
        plugin: false,
    };

    // 检查 gewe-cli
    print!("  检查 gewe-cli... ");
    if let Ok(output) = Command::new("gewe-cli").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{} ({})", "✅".green(), version);
            status.gewe_cli = Some(version);
        } else {
            println!("{}", "❌ 未安装".red());
        }
    } else {
        println!("{}", "❌ 未安装".red());
    }

    // 检查 Claude Code
    print!("  检查 Claude Code... ");
    if let Ok(output) = Command::new("claude").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{} ({})", "✅".green(), version);
            status.claude_code = Some(version);
        } else {
            println!("{}", "❌ 未安装".red());
        }
    } else {
        println!("{}", "❌ 未安装".red());
    }

    // 检查 plugin
    print!("  检查 gewe-cc-plugin... ");
    let settings_file = dirs::home_dir()
        .unwrap()
        .join(".claude/settings.json");

    if settings_file.exists() {
        if let Ok(content) = fs::read_to_string(&settings_file) {
            if content.contains("gewe-cc") {
                println!("{}", "✅ 已安装".green());
                status.plugin = true;
            } else {
                println!("{}", "❌ 未安装".red());
            }
        } else {
            println!("{}", "❌ 未安装".red());
        }
    } else {
        println!("{}", "❌ 未安装".red());
    }

    println!();
    status
}

fn print_installation_guide(deps: &DependencyStatus) {
    println!(
        "{}",
        "═══════════════════════════════════════".yellow()
    );
    println!(
        "{}",
        "  缺少必需依赖，请先完成以下安装：".yellow()
    );
    println!(
        "{}",
        "═══════════════════════════════════════".yellow()
    );
    println!();

    // gewe-cli 安装指引
    if deps.gewe_cli.is_none() {
        println!("{}", "📦 gewe-cli (微信消息收发工具)".bright_white().bold());
        println!();
        println!("  {} 从 GitHub 安装 (推荐):", "方式 1:".bright_cyan());
        println!(
            "    {}",
            "curl -fsSL https://raw.githubusercontent.com/wangnov/gewe-cli/main/install.sh | sh"
                .bright_black()
        );
        println!();
        println!("  {} 使用 Cargo 安装:", "方式 2:".bright_cyan());
        println!("    {}", "cargo install gewe-cli".bright_black());
        println!();
        println!(
            "  {} 使用 Homebrew 安装 (macOS):",
            "方式 3:".bright_cyan()
        );
        println!("    {}", "brew install gewe-cli".bright_black());
        println!();
        println!("  {} GitHub Releases:", "方式 4:".bright_cyan());
        println!(
            "    {}",
            "https://github.com/wangnov/gewe-cli/releases".bright_black()
        );
        println!();
        println!("  安装后验证: {}", "gewe-cli --version".dimmed());
        println!();
        println!("{}", "─────────────────────────────────────".dimmed());
        println!();
    }

    // Claude Code 安装指引
    if deps.claude_code.is_none() {
        println!(
            "{}",
            "📦 Claude Code (Anthropic 官方 CLI)".bright_white().bold()
        );
        println!();
        println!("  {} 官方安装脚本:", "方式 1:".bright_cyan());
        println!(
            "    {}",
            "curl -fsSL https://install.claudecode.com | sh".bright_black()
        );
        println!();
        println!("  {} 使用 npm 安装:", "方式 2:".bright_cyan());
        println!(
            "    {}",
            "npm install -g @anthropic-ai/claude-code".bright_black()
        );
        println!();
        println!("  {} 官方文档:", "详细信息:".bright_cyan());
        println!(
            "    {}",
            "https://docs.claudecode.com/installation".bright_black()
        );
        println!();
        println!("  安装后验证: {}", "claude --version".dimmed());
        println!();
        println!("{}", "─────────────────────────────────────".dimmed());
        println!();
    }

    // plugin 安装指引
    if !deps.plugin {
        println!(
            "{}",
            "📦 gewe-cc-plugin (Claude Code 插件)".bright_white().bold()
        );
        println!();
        println!(
            "  {} 从 GitHub 安装 (推荐):",
            "方式 1:".bright_cyan()
        );
        println!("    {}", "# 添加 marketplace".dimmed());
        println!(
            "    {}",
            "claude plugin marketplace add wangnov/gewe-cc".bright_black()
        );
        println!("    {}", "# 安装 plugin".dimmed());
        println!(
            "    {}",
            "claude plugin install gewe-cc".bright_green().bold()
        );
        println!();
        println!(
            "  {} 本地安装 (开发模式):",
            "方式 2:".bright_cyan()
        );
        println!("    {}", "git clone https://github.com/wangnov/gewe-cc.git".bright_black());
        println!("    {}", "cd gewe-cc".bright_black());
        println!("    {}", "claude plugin marketplace add ./plugin".bright_black());
        println!("    {}", "claude plugin install gewe-cc".bright_black());
        println!();
        println!("  安装后验证: {}", "claude plugin list".dimmed());
        println!();
        println!("{}", "─────────────────────────────────────".dimmed());
        println!();
    }

    println!("{}", "完成以上安装后，重新运行:".yellow());
    println!("  {}", "gewe-cc init".bright_green().bold());
    println!();
}

fn create_config() -> Result<()> {
    println!("{}", "⚙️  生成配置文件".bright_white().bold());
    println!();

    let config_mgr = ConfigManager::new()?;

    if config_mgr.config_file().exists() {
        println!("  配置文件已存在: {}", config_mgr.config_file().display());
        if !Confirm::new()
            .with_prompt("  是否重新配置?")
            .default(false)
            .interact()?
        {
            println!("  {} 保留现有配置", "✅".green());
            return Ok(());
        }
    }

    // 询问用户配置
    println!("  请输入微信配置:");
    println!();

    let wxid: String = loop {
        let input: String = Input::new()
            .with_prompt("    目标微信 ID")
            .interact_text()?;

        if input.trim().is_empty() {
            println!("    {} 微信 ID 不能为空，请重新输入", "❌".red());
            continue;
        }

        break input.trim().to_string();
    };

    let listen: String = Input::new()
        .with_prompt("    监听地址")
        .default("0.0.0.0:4399".to_string())
        .interact_text()?;

    println!();

    // 生成配置
    let config = Config {
        notification: crate::config::NotificationConfig {
            wxid,
            listen,
            ..Default::default()
        },
        ..Default::default()
    };

    config_mgr.save(&config)?;

    println!(
        "  {} 配置已保存到: {}",
        "✅".green(),
        config_mgr.config_file().display()
    );
    println!();

    Ok(())
}

fn print_success_message() {
    println!("{}", "═══════════════════════════════════════".green());
    println!("{}", "  ✅ 初始化完成！".green().bold());
    println!("{}", "═══════════════════════════════════════".green());
    println!();
    println!("{}", "下一步操作:".bright_white().bold());
    println!();
    println!("  {} 启用全局远程模式:", "1.".bright_cyan());
    println!("     {}", "gewe-cc on".bright_green());
    println!();
    println!("  {} 启动 Claude Code:", "2.".bright_cyan());
    println!("     {}", "claude".bright_green());
    println!();
    println!("  {} 在 Claude Code 中工作:", "3.".bright_cyan());
    println!("     {}", "创建一个文件 test.txt".dimmed());
    println!();
    println!("  {} 任务完成后会自动:", "4.".bright_cyan());
    println!("     {} 发送微信通知", "•".dimmed());
    println!("     {} 等待你的回复", "•".dimmed());
    println!("     {} 根据回复继续工作或停止", "•".dimmed());
    println!();
    println!("{}", "─────────────────────────────────────".dimmed());
    println!();
    println!(
        "  {} 查看状态: {}",
        "💡".bright_yellow(),
        "gewe-cc status".bright_black()
    );
    println!(
        "  {} 禁用远程模式: {}",
        "💡".bright_yellow(),
        "gewe-cc off".bright_black()
    );
    println!(
        "  {} 查看文档: {}",
        "💡".bright_yellow(),
        "https://github.com/wangnov/gewe-cc".bright_black()
    );
    println!();
}
