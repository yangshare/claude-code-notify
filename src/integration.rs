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
        // 获取配置目录（支持 CLAUDE_CONFIG_DIR 环境变量）
        let config_dir = Self::get_config_dir()?;
        let settings_file = config_dir.join("settings.json");

        if settings_file.exists() {
            Some(settings_file)
        } else {
            None
        }
    }

    /// 获取 Claude Code 配置目录
    fn get_config_dir() -> Option<PathBuf> {
        // 优先使用环境变量
        if let Ok(custom_dir) = std::env::var("CLAUDE_CONFIG_DIR") {
            return Some(PathBuf::from(custom_dir));
        }

        // 使用默认路径 ~/.claude
        Self::home_dir()?.join(".claude").into()
    }

    /// 获取主目录
    fn home_dir() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var("USERPROFILE").ok().map(PathBuf::from)
        }

        #[cfg(not(windows))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
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

        // 清理旧的 hooks
        hooks.remove("PostCommand");
        hooks.remove("CommandError");
        hooks.remove("PostToolUse");
        hooks.remove("PostToolUseFailure");

        // 定义 PermissionRequest hook（在需要权限时通知）
        let permission_hook = json!({
            "matcher": "Bash|Read|Write|Edit",
            "hooks": [
                {
                    "type": "command",
                    "command": "ccn notify --status=pending --cmd='Claude Code 需要授权' || true"
                }
            ]
        });

        // 注入 PermissionRequest hook
        let permission_request = hooks.entry("PermissionRequest").or_insert_with(|| json!([]));
        if let Some(arr) = permission_request.as_array_mut() {
            // 简单去重检查
            let has_hook = arr.iter().any(|h| {
                h["hooks"].as_array().map_or(false, |cmds| {
                    cmds.iter().any(|cmd| {
                        cmd["command"].as_str().map_or(false, |s| s.contains("ccn notify --status=pending"))
                    })
                })
            });

            if !has_hook {
                arr.push(permission_hook);
            }
        }

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
            if let Some(hooks) = obj.get_mut("hooks").and_then(|h| h.as_object_mut()) {
                // 移除旧的 legacy hooks
                hooks.remove("PostCommand");
                hooks.remove("CommandError");

                // 移除 PostToolUse 中的 ccn hooks
                if let Some(arr) = hooks.get_mut("PostToolUse").and_then(|v| v.as_array_mut()) {
                    arr.retain(|h| {
                        !h["hooks"].as_array().map_or(false, |cmds| {
                            cmds.iter().any(|cmd| {
                                cmd["command"].as_str().map_or(false, |s| s.contains("ccn notify"))
                            })
                        })
                    });
                    if arr.is_empty() {
                        hooks.remove("PostToolUse");
                    }
                }

                // 移除 PostToolUseFailure 中的 ccn hooks
                if let Some(arr) = hooks.get_mut("PostToolUseFailure").and_then(|v| v.as_array_mut()) {
                    arr.retain(|h| {
                        !h["hooks"].as_array().map_or(false, |cmds| {
                            cmds.iter().any(|cmd| {
                                cmd["command"].as_str().map_or(false, |s| s.contains("ccn notify"))
                            })
                        })
                    });
                    if arr.is_empty() {
                        hooks.remove("PostToolUseFailure");
                    }
                }
            }
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
                    let has_success = hooks_obj.get("PostToolUse")
                        .and_then(|v| v.as_array())
                        .map_or(false, |arr| {
                            arr.iter().any(|h| {
                                h["hooks"].as_array().map_or(false, |cmds| {
                                    cmds.iter().any(|cmd| {
                                        cmd["command"].as_str().map_or(false, |s| s.contains("ccn notify"))
                                    })
                                })
                            })
                        });
                    
                    // 同时也检查旧的配置，以便向后兼容检测
                    let has_legacy = hooks_obj.contains_key("PostCommand") ||
                                   hooks_obj.contains_key("CommandError");

                    return Ok(has_success || has_legacy);
                }
            }
        }

        Ok(false)
    }

    /// 验证集成：测试 hooks 命令是否可执行
    pub fn verify_integration(&self) -> Result<VerificationResult> {
        use std::process::Command;

        // 检查 ccn 命令是否在 PATH 中
        let ccn_available = match Command::new("ccn")
            .arg("--version")
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if !ccn_available {
            return Ok(VerificationResult {
                ccn_in_path: false,
                test_notification_sent: false,
                error: Some("ccn 命令不在 PATH 中，请重新运行 `ccn setup` 并重启终端".to_string()),
            });
        }

        // 尝试发送测试通知
        let test_result = Command::new("ccn")
            .args(&["notify", "--status=success", "--duration=1", "--cmd=test"])
            .output();

        let test_success = match test_result {
            Ok(output) => output.status.success(),
            Err(e) => {
                return Ok(VerificationResult {
                    ccn_in_path: true,
                    test_notification_sent: false,
                    error: Some(format!("测试通知失败: {}", e)),
                });
            }
        };

        Ok(VerificationResult {
            ccn_in_path: true,
            test_notification_sent: test_success,
            error: if test_success { None } else { Some("测试通知执行失败".to_string()) },
        })
    }
}

/// 集成验证结果
#[derive(Debug)]
pub struct VerificationResult {
    /// ccn 命令是否在 PATH 中
    pub ccn_in_path: bool,
    /// 测试通知是否成功发送
    pub test_notification_sent: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}
