mod cli;
mod config;
mod notification;
mod policy;
mod integration;
mod wizard;
mod aggregator;
mod sound;

use anyhow::Result;

fn main() -> Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // 运行 CLI
    cli::run()
}
