use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod hook;
mod init;
mod notify;
mod remote;
mod sanitize;
mod server;
mod transcript;

use hook::HookHandler;

#[derive(Parser)]
#[command(name = "gewe-cc")]
#[command(version, about = "Claude Code 远程协作模式命令行工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化环境（检查依赖、生成配置）
    Init,

    /// 启用全局远程模式
    On,

    /// 禁用全局远程模式
    Off {
        /// 可选：会话 ID（仅关闭该会话，不禁用全局模式）
        #[arg(long)]
        session_id: Option<String>,
    },

    /// 查看远程模式状态
    Status,

    /// 修改配置
    Config {
        /// 微信 ID
        #[arg(long)]
        wxid: Option<String>,

        /// 监听地址
        #[arg(long)]
        listen: Option<String>,

        /// 超时时间（秒，0 表示无限等待）
        #[arg(long)]
        timeout: Option<u64>,

        /// Transcript 展示域名
        #[arg(long)]
        transcript_domain: Option<String>,
    },

    /// 启动 HTTP 服务器（用于展示 transcript）
    Serve {
        /// 监听端口
        #[arg(short, long, default_value = "4400")]
        port: u16,
    },

    /// 发送链接卡片并等待回复
    SendLink {
        /// Session ID
        #[arg(long)]
        session_id: String,

        /// 任务摘要
        #[arg(long)]
        summary: String,
    },

    /// 发送消息并等待回复
    WaitReply {
        /// 消息内容
        #[arg(short = 'M', long)]
        message: String,

        /// 可选：临时覆盖配置中的目标微信 ID
        #[arg(long)]
        to_wxid: Option<String>,

        /// 可选：临时覆盖配置中的监听地址
        #[arg(long)]
        listen: Option<String>,

        /// 可选：超时时间（秒）
        #[arg(long, short = 't')]
        timeout: Option<u64>,
    },

    /// 发送通知（不等待回复）
    Notify {
        /// 消息内容
        #[arg(short = 'M', long)]
        message: String,

        /// 可选：临时覆盖配置中的目标微信 ID
        #[arg(long)]
        to_wxid: Option<String>,
    },

    /// 处理 Hook 事件（由 plugin 调用，非用户命令）
    Hook {
        /// Hook 类型：user-prompt-submit 或 stop
        hook_type: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            init::run()?;
        }
        Commands::On => {
            remote::handle_on()?;
        }
        Commands::Off { session_id } => {
            remote::handle_off(session_id)?;
        }
        Commands::Status => {
            remote::handle_status()?;
        }
        Commands::Config { wxid, listen, timeout, transcript_domain } => {
            remote::handle_config(wxid, listen, timeout, transcript_domain)?;
        }
        Commands::Serve { port } => {
            // 使用 tokio 运行时启动 HTTP 服务器
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async {
                server::start_server(port).await
            })?;
        }
        Commands::SendLink { session_id, summary } => {
            let reply = notify::send_link_and_wait(session_id, summary)?;
            println!("{}", reply);
        }
        Commands::WaitReply {
            message,
            to_wxid,
            listen,
            timeout,
        } => {
            let reply = notify::wait_reply(message, to_wxid, listen, timeout)?;
            println!("{}", reply);
        }
        Commands::Notify { message, to_wxid } => {
            notify::send_notification(message, to_wxid)?;
            println!("✅ 消息已发送");
        }
        Commands::Hook { hook_type } => {
            let decision = HookHandler::handle_from_stdin(&hook_type)?;
            decision.output()?;
        }
    }

    Ok(())
}
