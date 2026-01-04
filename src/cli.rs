//! CLI 命令行模块
//!
//! 处理命令行参数解析和子命令调度

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};

use crate::config::{load_config, Config};
use crate::notification::{get_notification_manager, NotificationStatus};
use crate::policy::PolicyEngine;
use crate::integration::IntegrationManager;
use crate::wizard::ConfigWizard;
use crate::aggregator::{NotificationAggregator, get_state_file_path};
use crate::sound::{SoundPlayer, SystemSound};
#[cfg(windows)]
use crate::path_manager::PathManager;

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

        /// 任务耗时（秒），默认为0
        #[arg(short, long, default_value = "0", value_name = "SECS")]
        duration: Option<u64>,

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

    /// 验证集成
    Verify,

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
            handle_notify(status, duration.unwrap_or(0), cmd)
        }

        Commands::Init => {
            handle_init()
        }

        Commands::Setup => {
            handle_setup()
        }

        Commands::Uninstall => {
            handle_uninstall()
        }

        Commands::Verify => {
            handle_verify()
        }

        Commands::Config => {
            handle_config()
        }

        Commands::Test => {
            handle_test()
        }
    }
}

/// 处理 notify 命令
fn handle_notify(status: String, duration: u64, cmd: String) -> Result<()> {
    log::info!("收到通知请求: status={}, duration={}, cmd={}", status, duration, cmd);

    // 解析状态
    let notification_status = match status.to_lowercase().as_str() {
        "success" => NotificationStatus::Success,
        "error" | "failed" | "failure" => NotificationStatus::Error,
        "pending" | "running" => NotificationStatus::Pending,
        _ => {
            eprintln!("警告: 未知状态 '{}', 使用默认状态 'success'", status);
            NotificationStatus::Success
        }
    };

    // 加载配置
    let config = load_config()
        .context("无法加载配置文件")?;

    // 创建策略引擎
    let policy_engine = PolicyEngine::new(config.clone());

    // 检查是否应该发送通知
    if !policy_engine.should_notify(notification_status, duration, &cmd) {
        log::info!("通知被策略过滤（时间阈值低于 {} 秒）", config.threshold.min_duration);
        return Ok(());
    }

    // duration 为 0 或 error/pending 状态时直接发送（绕过聚合）
    let should_bypass_aggregation = duration == 0
        || matches!(notification_status, NotificationStatus::Error | NotificationStatus::Pending);

    // 如果启用聚合且不需要绕过，使用聚合器
    if config.aggregation.enabled && !should_bypass_aggregation {
        return handle_aggregated_notification(&config, &status, duration, &cmd, notification_status);
    }

    // 直接发送通知
    send_single_notification(notification_status, &cmd, duration, &config)
}

/// 处理聚合通知
fn handle_aggregated_notification(
    config: &Config,
    status_str: &str,
    duration: u64,
    cmd: &str,
    notification_status: NotificationStatus,
) -> Result<()> {
    let aggregator = NotificationAggregator::new(
        get_state_file_path(),
        config.aggregation.window,
        config.aggregation.max_toasts,
    );

    // 尝试添加到聚合器
    match aggregator.add_notification(status_str, duration, cmd) {
        Ok(Some(result)) => {
            // 达到聚合条件，发送聚合通知
            log::info!("发送聚合通知: {} 个任务", result.total);

            let status = match result.status() {
                "error" => NotificationStatus::Error,
                _ => NotificationStatus::Success,
            };

            let notifier = get_notification_manager();
            notifier.send_notification(
                status,
                &result.title(),
                &result.message(),
                config.templates.default.duration,
            )?;
        }
        Ok(None) => {
            // 添加到聚合缓冲区，暂不发送
            log::info!("通知已添加到聚合缓冲区");
        }
        Err(e) => {
            log::warn!("聚合失败，发送单个通知: {}", e);
            send_single_notification(notification_status, cmd, duration, config)?;
        }
    }

    Ok(())
}

/// 发送单个通知
fn send_single_notification(
    status: NotificationStatus,
    cmd: &str,
    duration: u64,
    config: &Config,
) -> Result<()> {
    // 播放音效
    play_notification_sound(status, config);

    let notifier = get_notification_manager();

    // 构建通知内容
    let title = build_title(status, cmd);
    let message = build_message(duration, cmd);

    // 发送通知
    let template_name = PolicyEngine::new(config.clone()).match_template(cmd);
    let duration_ms = if let Some(name) = &template_name {
        if name == "default" {
            config.templates.default.duration
        } else {
            config.templates.custom.get(name)
                .map(|t| t.duration)
                .unwrap_or(config.templates.default.duration)
        }
    } else {
        config.templates.default.duration
    };

    notifier.send_notification(status, &title, &message, duration_ms)
        .context("发送通知失败")?;

    log::info!("通知已发送");
    Ok(())
}

/// 播放通知音效
fn play_notification_sound(status: NotificationStatus, config: &Config) {
    if !config.sound_enabled {
        return;
    }

    let sound_player = SoundPlayer::new(true);

    // 尝试播放自定义音效
    let template_name = PolicyEngine::new(config.clone()).match_template("");
    let sound_file = if let Some(name) = &template_name {
        if name == "default" {
            &config.templates.default.sound
        } else {
            config.templates.custom.get(name)
                .map(|t| &t.sound)
                .unwrap_or(&config.templates.default.sound)
        }
    } else {
        &config.templates.default.sound
    };

    // 如果配置了自定义音效文件
    if sound_file != "default" && !sound_file.is_empty() {
        if let Err(e) = sound_player.play_sound_file(sound_file) {
            log::warn!("播放自定义音效失败: {}", e);
        }
        return;
    }

    // 否则播放系统提示音
    let system_sound = match status {
        NotificationStatus::Success => SystemSound::Success,
        NotificationStatus::Error => SystemSound::Error,
        NotificationStatus::Pending => SystemSound::Notification,
    };

    if let Err(e) = sound_player.play_system_sound(system_sound) {
        log::warn!("播放系统音效失败: {}", e);
    }
}

/// 处理 test 命令
fn handle_test() -> Result<()> {
    println!("发送测试通知...");

    let notifier = get_notification_manager();

    if !notifier.is_available() {
        println!("警告: 通知系统不可用");
        return Ok(());
    }

    // 发送成功测试通知
    notifier.send_notification(
        NotificationStatus::Success,
        "CCN 测试通知",
        "CCN 已成功集成！",
        5000,
    )?;

    println!("测试通知已发送");
    Ok(())
}

/// 处理 config 命令
fn handle_config() -> Result<()> {
    let config = load_config()
        .context("无法加载配置文件")?;

    println!("当前配置：");
    println!("版本: {}", config.version);
    println!("声音: {}", if config.sound_enabled { "启用" } else { "禁用" });
    println!("专注助手模式: {:?}", config.focus_assistant_mode);
    println!("最小阈值: {} 秒", config.threshold.min_duration);
    println!("白名单: {:?}", config.threshold.whitelist);
    println!("聚合: {}", if config.aggregation.enabled { "启用" } else { "禁用" });
    println!("聚合窗口: {} 毫秒", config.aggregation.window);
    println!("日志级别: {}", config.logging.level);

    Ok(())
}

/// 处理 init 命令
fn handle_init() -> Result<()> {
    println!("启动配置向导...\n");

    let wizard = ConfigWizard::new();
    match wizard.run() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("配置向导出错: {}", e);
            Err(e)
        }
    }
}

/// 处理 setup 命令
fn handle_setup() -> Result<()> {
    println!("正在设置 CCN 集成...");

    let manager = IntegrationManager::new();

    // 侦测配置文件
    let config_path = manager.detect_config_path();
    let config_path = match config_path {
        Some(path) => {
            println!("✓ 找到 Claude Code 配置文件: {:?}", path);
            path
        }
        None => {
            println!("❌ 未找到 Claude Code 配置文件\n");
            println!("请确认：");
            println!("  1. Claude Code 已安装（CLI 或 VS Code 插件）");
            println!("  2. 配置文件存在于: ~/.claude/settings.json");
            println!("\n或者设置自定义路径：");
            println!("  Windows: set CLAUDE_CONFIG_DIR=D:\\custom\\path");
            println!("  Linux/macOS: export CLAUDE_CONFIG_DIR=/custom/path");
            return Ok(());
        }
    };

    // 检查是否已集成
    if manager.is_integrated(&config_path).unwrap_or(false) {
        println!("⚠ CCN 已经集成到 Claude Code");
        println!("如需重新集成，请先运行 `ccn uninstall`");
        return Ok(());
    }

    // 添加到 PATH（仅 Windows）
    #[cfg(windows)]
    {
        println!("正在配置 PATH 环境变量...");
        let ccn_dir = std::env::current_exe()
            .context("无法获取可执行文件路径")?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?
            .to_path_buf();

        match PathManager::add_to_path(&ccn_dir) {
            Ok(true) => {
                println!("✓ 已将 CCN 添加到系统 PATH");
            }
            Ok(false) => {
                println!("ℹ PATH 已包含 CCN 目录，跳过");
            }
            Err(e) => {
                println!("⚠ 无法修改 PATH: {}", e);
                println!("请手动添加到 PATH: {:?}", ccn_dir);
            }
        }
    }

    // 备份配置文件
    println!("正在备份配置文件...");
    let backup_path = manager.backup_config(&config_path)?;
    println!("✓ 备份已创建: {:?}", backup_path);

    // 注入 hooks
    println!("正在注入 hooks...");
    manager.inject_hooks(&config_path)?;
    println!("✓ Hooks 已成功注入");

    // 发送测试通知
    println!("正在发送测试通知...");
    manager.send_test_notification()?;

    println!("\n✅ CCN 集成成功！");

    #[cfg(windows)]
    {
        println!("\n⚠ 重要提示：");
        println!("请重启您的终端或 VS Code，以使 PATH 环境变量生效。");
        println!("重启后，hooks 将自动生效，您会收到任务完成通知。");
    }

    Ok(())
}

/// 处理 uninstall 命令
fn handle_uninstall() -> Result<()> {
    println!("正在卸载 CCN 集成...");

    let manager = IntegrationManager::new();

    // 侦测配置文件
    let config_path = manager.detect_config_path();
    let config_path = match config_path {
        Some(path) => {
            println!("✓ 找到 Claude Code 配置文件: {:?}", path);
            path
        }
        None => {
            println!("❌ 未找到 Claude Code 配置文件");
            println!("CCN 可能未安装");
            return Ok(());
        }
    };

    // 检查是否已集成
    if !manager.is_integrated(&config_path).unwrap_or(false) {
        println!("⚠ CCN 未集成到 Claude Code");
        return Ok(());
    }

    // 移除 hooks
    println!("正在移除 hooks...");
    manager.remove_hooks(&config_path)?;
    println!("✓ Hooks 已移除");

    // 从 PATH 移除（仅 Windows）
    #[cfg(windows)]
    {
        println!("正在清理 PATH 环境变量...");
        let ccn_dir = std::env::current_exe()
            .context("无法获取可执行文件路径")?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?
            .to_path_buf();

        match PathManager::remove_from_path(&ccn_dir) {
            Ok(true) => {
                println!("✓ 已从系统 PATH 移除 CCN");
            }
            Ok(false) => {
                println!("ℹ PATH 中不包含 CCN 目录");
            }
            Err(e) => {
                println!("⚠ 无法清理 PATH: {}", e);
            }
        }
    }

    println!("\n✅ CCN 集成已移除");
    println!("配置文件的备份仍保留在原位置");

    #[cfg(windows)]
    {
        println!("\n⚠ 请重启终端以使 PATH 更新生效");
    }

    Ok(())
}

/// 处理 verify 命令
fn handle_verify() -> Result<()> {
    println!("正在验证 CCN 集成...\n");

    let manager = IntegrationManager::new();

    // 验证集成
    let result = manager.verify_integration()?;

    // 显示结果
    println!("集成验证结果：");
    println!("  ccn 命令在 PATH 中: {}", if result.ccn_in_path { "✓" } else { "✗" });
    println!("  测试通知发送: {}", if result.test_notification_sent { "✓" } else { "✗" });

    if let Some(error) = result.error {
        println!("\n错误: {}", error);
        return Err(anyhow::anyhow!("集成验证失败"));
    }

    println!("\n✅ 集成验证通过！CCN 已正确配置。");
    Ok(())
}

/// 构建通知标题
fn build_title(status: NotificationStatus, _cmd: &str) -> String {
    let status_text = match status {
        NotificationStatus::Success => "完成",
        NotificationStatus::Error => "失败",
        NotificationStatus::Pending => "进行中",
    };

    format!("任务{}", status_text)
}

/// 构建通知消息
fn build_message(duration: u64, cmd: &str) -> String {
    format!("{} (耗时: {}秒)", cmd, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::NotificationStatus;

    #[test]
    fn test_build_title_success() {
        let title = build_title(NotificationStatus::Success, "npm test");
        assert_eq!(title, "任务完成");
    }

    #[test]
    fn test_build_title_error() {
        let title = build_title(NotificationStatus::Error, "cargo build");
        assert_eq!(title, "任务失败");
    }

    #[test]
    fn test_build_title_pending() {
        let title = build_title(NotificationStatus::Pending, "deploy");
        assert_eq!(title, "任务进行中");
    }

    #[test]
    fn test_build_message() {
        let message = build_message(15, "npm test");
        assert_eq!(message, "npm test (耗时: 15秒)");
    }

    #[test]
    fn test_build_message_zero_duration() {
        let message = build_message(0, "unknown cmd");
        assert_eq!(message, "unknown cmd (耗时: 0秒)");
    }

    #[test]
    fn test_build_message_long_duration() {
        let message = build_message(3600, "long running task");
        assert_eq!(message, "long running task (耗时: 3600秒)");
    }
}
