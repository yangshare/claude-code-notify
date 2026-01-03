//! 自动集成模块
//!
//! 处理与 Claude Code 的自动集成

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

/// 集成管理器
pub struct IntegrationManager;

impl IntegrationManager {
    pub fn new() -> Self {
        Self
    }

    /// 侦测 Claude Code 配置文件
    pub fn detect_config_path(&self) -> Option<PathBuf> {
        #[cfg(windows)]
        let paths = vec![
            std::env::var("APPDATA")
                .ok()
                .map(|p| PathBuf::from(p).join("Claude").join("config.json")),
            std::env::var("USERPROFILE")
                .ok()
                .map(|p| PathBuf::from(p).join(".claude").join("config.json")),
        ];

        #[cfg(target_os = "macos")]
        let paths = vec![
            std::env::var("HOME")
                .ok()
                .map(|p| PathBuf::from(p).join(".claude").join("config.json")),
        ];

        #[cfg(not(any(windows, target_os = "macos")))]
        let paths = vec![
            std::env::var("HOME")
                .ok()
                .map(|p| PathBuf::from(p).join(".claude").join("config.json")),
        ];

        for path_option in paths {
            if let Some(path) = path_option {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    /// 备份配置文件
    pub fn backup_config(&self, config_path: &PathBuf) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = config_path.with_extension(&format!("bak.{}", timestamp));

        fs::copy(config_path, &backup_path)
            .with_context(|| format!("无法备份配置文件: {:?}", config_path))?;

        log::info!("配置文件已备份到: {:?}", backup_path);
        Ok(backup_path)
    }

    /// 注入 hooks 配置
    pub fn inject_hooks(&self, config_path: &PathBuf) -> Result<()> {
        // 读取现有配置
        let content = fs::read_to_string(config_path)
            .context("无法读取配置文件")?;

        let mut config: Value = serde_json::from_str(&content)
            .context("配置文件 JSON 格式错误")?;

        // 确保 hooks 对象存在
        if !config.is_object() {
            config = json!({});
        }

        let hooks = config
            .as_object_mut()
            .unwrap()
            .entry("hooks")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .unwrap();

        // 注入 PostCommand hook
        hooks.insert(
            "PostCommand".to_string(),
            json!("ccn notify --status=success --duration=$DURATION --cmd='$COMMAND' || true")
        );

        // 注入 CommandError hook
        hooks.insert(
            "CommandError".to_string(),
            json!("ccn notify --status=error --duration=$DURATION --cmd='$COMMAND' || true")
        );

        // 写回配置文件
        let updated_content = serde_json::to_string_pretty(&config)
            .context("序列化配置失败")?;

        fs::write(config_path, updated_content)
            .context("无法写入配置文件")?;

        log::info!("Hooks 已注入到配置文件");
        Ok(())
    }

    /// 发送测试通知
    pub fn send_test_notification(&self) -> Result<()> {
        use crate::notification::{get_notification_manager, NotificationStatus};

        let notifier = get_notification_manager();
        notifier.send_notification(
            NotificationStatus::Success,
            "CCN 集成成功！",
            "CCN 已成功集成到 Claude Code",
            5000,
        )?;

        Ok(())
    }

    /// 移除 hooks 配置
    pub fn remove_hooks(&self, config_path: &PathBuf) -> Result<()> {
        // 读取现有配置
        let content = fs::read_to_string(config_path)
            .context("无法读取配置文件")?;

        let mut config: Value = serde_json::from_str(&content)
            .context("配置文件 JSON 格式错误")?;

        // 移除 hooks
        if let Some(obj) = config.as_object_mut() {
            obj.remove("hooks");
        }

        // 写回配置文件
        let updated_content = serde_json::to_string_pretty(&config)
            .context("序列化配置失败")?;

        fs::write(config_path, updated_content)
            .context("无法写入配置文件")?;

        log::info!("Hooks 已从配置文件移除");
        Ok(())
    }

    /// 检查是否已集成
    pub fn is_integrated(&self, config_path: &PathBuf) -> Result<bool> {
        let content = fs::read_to_string(config_path)
            .context("无法读取配置文件")?;

        let config: Value = serde_json::from_str(&content)
            .context("配置文件 JSON 格式错误")?;

        if let Some(obj) = config.as_object() {
            if let Some(hooks) = obj.get("hooks") {
                if let Some(hooks_obj) = hooks.as_object() {
                    return Ok(hooks_obj.contains_key("PostCommand") ||
                               hooks_obj.contains_key("CommandError"));
                }
            }
        }

        Ok(false)
    }
}
