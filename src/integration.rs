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

        // 清理旧的 hooks（包括不存在的 PermissionRequest 和 Stop）
        hooks.remove("PostCommand");
        hooks.remove("CommandError");
        hooks.remove("PostToolUse");
        hooks.remove("PostToolUseFailure");
        hooks.remove("PermissionRequest");
        hooks.remove("Stop");

        // 定义 Notification hook（当 Claude Code 发送权限请求通知时触发）
        let notification_hook = json!({
            "matcher": "permission_prompt",
            "hooks": [{
                "type": "command",
                "command": "ccn notify --status=pending --cmd='Claude Code 需要授权' || true"
            }]
        });

        // 注入 Notification hook
        let notification = hooks.entry("Notification").or_insert_with(|| json!([]));
        if let Some(arr) = notification.as_array_mut() {
            // 简单去重检查
            let has_hook = arr.iter().any(|h| {
                h["matcher"].as_str().map_or(false, |m| m == "permission_prompt")
                    && h["hooks"].as_array().map_or(false, |cmds| {
                        cmds.iter().any(|cmd| {
                            cmd["command"].as_str().map_or(false, |c| c.contains("ccn notify"))
                        })
                    })
            });

            if !has_hook {
                arr.push(notification_hook);
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
                hooks.remove("PermissionRequest");
                hooks.remove("Stop");

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

                // 移除 Notification 中的 ccn hooks
                if let Some(arr) = hooks.get_mut("Notification").and_then(|v| v.as_array_mut()) {
                    arr.retain(|h| {
                        !h["matcher"].as_str().map_or(false, |m| m == "permission_prompt")
                            || !h["hooks"].as_array().map_or(false, |cmds| {
                                cmds.iter().any(|cmd| {
                                    cmd["command"].as_str().map_or(false, |c| c.contains("ccn notify"))
                                })
                            })
                    });
                    if arr.is_empty() {
                        hooks.remove("Notification");
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
                    // 检查 Notification hook
                    let has_notification = hooks_obj.get("Notification")
                        .and_then(|v| v.as_array())
                        .map_or(false, |arr| {
                            arr.iter().any(|h| {
                                h["matcher"].as_str().map_or(false, |m| m == "permission_prompt")
                                    && h["hooks"].as_array().map_or(false, |cmds| {
                                        cmds.iter().any(|cmd| {
                                            cmd["command"].as_str().map_or(false, |c| c.contains("ccn notify"))
                                        })
                                    })
                            })
                        });

                    // 检查旧的 Stop hook
                    let has_stop = hooks_obj.get("Stop")
                        .and_then(|v| v.as_array())
                        .map_or(false, |arr| {
                            arr.iter().any(|h| {
                                h["type"].as_str().map_or(false, |t| t == "command")
                                    && h["command"].as_str().map_or(false, |c| c.contains("ccn notify"))
                            })
                        });

                    // 检查旧的 PostToolUse hook
                    let has_post_tool = hooks_obj.get("PostToolUse")
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

                    return Ok(has_notification || has_stop || has_post_tool || has_legacy);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_manager_new() {
        let manager = IntegrationManager::new();
        // 测试创建成功（无 panic）
        let _ = &manager;
    }

    #[test]
    fn test_backup_config_filename_format() {
        let manager = IntegrationManager::new();

        // 这个测试需要实际文件存在，所以我们在临时目录中测试
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_settings.json");

        // 创建测试文件
        if let Ok(_) = fs::write(&test_file, "{}") {
            let backup_result = manager.backup_config(&test_file);

            // 验证备份文件名格式
            assert!(backup_result.is_ok());
            let backup_path = backup_result.unwrap();
            let backup_filename = backup_path.file_name().unwrap().to_string_lossy();

            // 备份文件应该包含 .bak. 前缀和时间戳
            assert!(backup_filename.contains(".bak."));

            // 清理
            let _ = fs::remove_file(&test_file);
            let _ = fs::remove_file(&backup_path);
        }
    }

    #[test]
    fn test_home_dir_detection() {
        // 测试主目录检测逻辑
        let home = IntegrationManager::home_dir();

        // 在大多数环境中应该能找到主目录
        if cfg!(windows) {
            assert!(home.is_some());
            let path = home.unwrap();
            // Windows 上通常是 USERPROFILE
            assert!(path.to_string_lossy().contains(":\\") || path.starts_with("\\\\"));
        } else {
            assert!(home.is_some());
            let path = home.unwrap();
            // Unix 上通常是 /home 或 /Users
            assert!(path.starts_with("/"));
        }
    }

    #[test]
    fn test_verification_result_structure() {
        let result = VerificationResult {
            ccn_in_path: true,
            test_notification_sent: false,
            error: Some("测试错误".to_string()),
        };

        assert_eq!(result.ccn_in_path, true);
        assert_eq!(result.test_notification_sent, false);
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "测试错误");
    }

    #[test]
    fn test_verification_result_no_error() {
        let result = VerificationResult {
            ccn_in_path: true,
            test_notification_sent: true,
            error: None,
        };

        assert!(result.ccn_in_path);
        assert!(result.test_notification_sent);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_get_config_dir_default() {
        // 测试默认配置目录检测
        let config_dir = IntegrationManager::get_config_dir();

        assert!(config_dir.is_some());
        let path = config_dir.unwrap();

        // 应该包含 .claude
        assert!(path.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn test_hooks_json_structure() {
        // 测试 hooks JSON 结构（Notification 事件）
        let hooks_json = json!({
            "Notification": [
                {
                    "matcher": "permission_prompt",
                    "hooks": [{
                        "type": "command",
                        "command": "ccn notify --status=pending --cmd='Claude Code 需要授权'"
                    }]
                }
            ]
        });

        assert!(hooks_json.is_object());
        assert!(hooks_json.get("Notification").is_some());

        if let Some(arr) = hooks_json["Notification"].as_array() {
            assert!(!arr.is_empty());
            assert!(arr[0]["matcher"].is_string());
            assert!(arr[0]["hooks"].is_array());
            if let Some(hooks) = arr[0]["hooks"].as_array() {
                assert!(!hooks.is_empty());
                assert!(hooks[0]["type"].is_string());
                assert!(hooks[0]["command"].is_string());
            }
        }
    }
}
