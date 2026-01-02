//! CLI 命令行模块
//!
//! 处理命令行参数解析和子命令调度

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "ccn")]
#[command(about = "Claude Code Notify - 优雅的任务通知工具", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 发送通知
    Notify {
        /// 任务状态: success, error, pending
        #[arg(short, long)]
        status: String,

        /// 任务耗时（秒）
        #[arg(short, long)]
        duration: u64,

        /// 执行的命令
        #[arg(short, long)]
        cmd: String,
    },

    /// 启动交互式配置向导
    Init,

    /// 自动集成到 Claude Code
    Setup,

    /// 卸载集成
    Uninstall,

    /// 显示当前配置
    Config,

    /// 发送测试通知
    Test,
}

/// 运行 CLI 命令
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Notify { status, duration, cmd } => {
            log::info!("发送通知: status={}, duration={}, cmd={}", status, duration, cmd);
            // TODO: 实现通知发送逻辑
            println!("通知功能待实现");
            Ok(())
        }

        Commands::Init => {
            log::info!("启动配置向导");
            // TODO: 实现配置向导
            println!("配置向导待实现");
            Ok(())
        }

        Commands::Setup => {
            log::info!("开始自动集成");
            // TODO: 实现自动集成
            println!("自动集成待实现");
            Ok(())
        }

        Commands::Uninstall => {
            log::info!("卸载集成");
            // TODO: 实现卸载
            println!("卸载功能待实现");
            Ok(())
        }

        Commands::Config => {
            log::info!("显示当前配置");
            // TODO: 实现配置显示
            println!("配置显示待实现");
            Ok(())
        }

        Commands::Test => {
            log::info!("发送测试通知");
            // TODO: 实现测试通知
            println!("测试通知待实现");
            Ok(())
        }
    }
}
