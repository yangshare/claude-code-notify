//! 通知管理模块
//!
//! 平台抽象层，处理不同操作系统的通知功能

use anyhow::Result;

/// 通知状态
#[derive(Debug, Clone, Copy)]
pub enum NotificationStatus {
    Success,
    Error,
    Pending,
}

/// 通知管理器 trait
pub trait NotificationManager {
    /// 发送通知
    fn send_notification(
        &self,
        status: NotificationStatus,
        title: &str,
        message: &str,
        duration_ms: u64,
    ) -> Result<()>;

    /// 检查通知是否可用
    fn is_available(&self) -> bool;
}

/// 获取平台特定的通知管理器
pub fn get_notification_manager() -> Box<dyn NotificationManager> {
    #[cfg(windows)]
    {
        Box::new(platform::WindowsNotificationManager::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(platform::MacOSNotificationManager::new())
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        Box::new(platform::FallbackNotificationManager)
    }
}

// 平台特定实现

#[cfg(windows)]
mod platform {
    use super::{NotificationManager, NotificationStatus};
    use anyhow::Result;
    use std::sync::OnceLock;
    use windows::core::{HSTRING, Interface};
    use windows::Data::Xml::Dom::*;
    use windows::UI::Notifications::*;

    /// AUMID (Application User Model ID)
    const AUMID: &str = "ClaudeCodeNotify.CCN";

    pub struct WindowsNotificationManager;

    impl WindowsNotificationManager {
        pub fn new() -> Self {
            Self
        }

        /// 获取状态对应的图标
        fn get_status_icon(status: NotificationStatus) -> &'static str {
            match status {
                NotificationStatus::Success => "✅",
                NotificationStatus::Error => "❌",
                NotificationStatus::Pending => "⏳",
            }
        }

        /// 获取状态文本
        fn get_status_text(status: NotificationStatus) -> &'static str {
            match status {
                NotificationStatus::Success => "成功",
                NotificationStatus::Error => "错误",
                NotificationStatus::Pending => "进行中",
            }
        }

        /// 获取或创建 ToastNotifier
        fn get_notifier() -> Result<&'static ToastNotifier> {
            static NOTIFIER: OnceLock<Result<ToastNotifier>> = OnceLock::new();

            let result = NOTIFIER.get_or_init(|| {
                ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))
                    .map_err(|e| anyhow::anyhow!("创建 ToastNotifier 失败: {:?}", e))
            });

            match result {
                Ok(notifier) => Ok(notifier),
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            }
        }
    }

    impl NotificationManager for WindowsNotificationManager {
        fn send_notification(
            &self,
            status: NotificationStatus,
            title: &str,
            message: &str,
            _duration_ms: u64,
        ) -> Result<()> {
            // 格式化标题
            let icon = Self::get_status_icon(status);
            let status_text = Self::get_status_text(status);
            let formatted_title = format!("{}{} - {}", icon, status_text, title);

            log::debug!("准备发送 Windows Toast 通知: {} - {}", formatted_title, message);

            // 尝试发送 Toast 通知
            match (|| -> Result<()> {
                // 获取 Toast XML 模板
                let toast_xml = ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02)
                    .map_err(|e| anyhow::anyhow!("获取模板失败: {:?}", e))?;

                // 填充文本元素
                let text_elements = toast_xml
                    .GetElementsByTagName(&HSTRING::from("text"))
                    .map_err(|e| anyhow::anyhow!("获取 text 元素失败: {:?}", e))?;

                // 设置第一个文本（标题）
                if let Some(title_node) = text_elements.Item(0).ok() {
                    let title_text = toast_xml
                        .CreateTextNode(&HSTRING::from(&formatted_title))
                        .map_err(|e| anyhow::anyhow!("创建标题节点失败: {:?}", e))?;
                    let title_node_ref: IXmlNode = title_text
                        .cast()
                        .map_err(|e| anyhow::anyhow!("转换 IXmlNode 失败: {:?}", e))?;
                    title_node.AppendChild(&title_node_ref)
                        .map_err(|e| anyhow::anyhow!("追加标题失败: {:?}", e))?;
                }

                // 设置第二个文本（消息）
                if let Some(message_node) = text_elements.Item(1).ok() {
                    let message_text = toast_xml
                        .CreateTextNode(&HSTRING::from(message))
                        .map_err(|e| anyhow::anyhow!("创建消息节点失败: {:?}", e))?;
                    let message_node_ref: IXmlNode = message_text
                        .cast()
                        .map_err(|e| anyhow::anyhow!("转换 IXmlNode 失败: {:?}", e))?;
                    message_node.AppendChild(&message_node_ref)
                        .map_err(|e| anyhow::anyhow!("追加消息失败: {:?}", e))?;
                }

                // 创建 Toast 通知对象
                let toast = ToastNotification::CreateToastNotification(&toast_xml)
                    .map_err(|e| anyhow::anyhow!("创建 Toast 通知失败: {:?}", e))?;

                // 获取 notifier 并显示通知
                let notifier = Self::get_notifier()?;
                notifier.Show(&toast)
                    .map_err(|e| anyhow::anyhow!("显示通知失败: {:?}", e))?;

                Ok(())
            })() {
                Ok(_) => {
                    log::info!("Toast 通知已发送: {}", formatted_title);
                    // 等待通知显示，确保 Windows 有足够时间处理
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    Ok(())
                }
                Err(e) => {
                    log::warn!("Toast 通知失败: {}, 降级到控制台输出", e);
                    // 降级到控制台输出
                    let icon = Self::get_status_icon(status);
                    println!("[通知] {} {}: {}", icon, title, message);
                    Ok(())
                }
            }
        }

        fn is_available(&self) -> bool {
            // Windows 11 上总是可用
            true
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{NotificationManager, NotificationStatus};
    use anyhow::Result;
    use std::process::Command;

    pub struct MacOSNotificationManager;

    impl MacOSNotificationManager {
        pub fn new() -> Self {
            Self
        }

        /// 检查是否安装了 terminal-notifier
        fn has_terminal_notifier() -> bool {
            Command::new("which")
                .arg("terminal-notifier")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }

        /// 使用 terminal-notifier 发送通知
        fn send_with_terminal_notifier(
            title: &str,
            message: &str,
            sound: &str,
        ) -> Result<()> {
            let output = Command::new("terminal-notifier")
                .arg("-title")
                .arg("Claude Code Notify")
                .arg("-message")
                .arg(message)
                .arg("-subtitle")
                .arg(title)
                .arg("-sound")
                .arg(sound)
                .output()?;

            if output.status.success() {
                log::info!("使用 terminal-notifier 发送通知成功");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("terminal-notifier 失败: {}", stderr))
            }
        }

        /// 使用 osascript 发送通知（内置方案）
        fn send_with_osascript(title: &str, message: &str) -> Result<()> {
            let script = format!(
                "display notification \"{}\" with title \"{}\" subtitle \"Claude Code Notify\"",
                message.replace('"', "'"), // 转义引号
                title.replace('"', "'")
            );

            let output = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()?;

            if output.status.success() {
                log::info!("使用 osascript 发送通知成功");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("osascript 失败: {}", stderr))
            }
        }
    }

    impl NotificationManager for MacOSNotificationManager {
        fn send_notification(
            &self,
            status: NotificationStatus,
            title: &str,
            message: &str,
            _duration_ms: u64,
        ) -> Result<()> {
            let icon = match status {
                NotificationStatus::Success => "✅",
                NotificationStatus::Error => "❌",
                NotificationStatus::Pending => "⏳",
            };

            // 构建带图标的消息
            let formatted_title = format!("{} {}", icon, title);
            let formatted_message = message.to_string();

            // 获取对应的声音
            let sound = match status {
                NotificationStatus::Success => "Glass",
                NotificationStatus::Error => "Basso",
                NotificationStatus::Pending => "Ping",
            };

            // 优先使用 terminal-notifier（提供更好的用户体验）
            if Self::has_terminal_notifier() {
                match Self::send_with_terminal_notifier(&formatted_title, &formatted_message, sound) {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        log::warn!("terminal-notifier 失败，尝试 osascript: {}", e);
                    }
                }
            }

            // 后备方案：使用 osascript（macOS 内置）
            match Self::send_with_osascript(&formatted_title, &formatted_message) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // 最后的后备方案：控制台输出
                    log::warn!("macOS 通知失败: {}, 降级到控制台输出", e);
                    println!("[macOS 通知] {} {}: {}", icon, formatted_title, formatted_message);
                    Ok(())
                }
            }
        }

        fn is_available(&self) -> bool {
            // macOS 总是有某种通知方式可用（至少 osascript）
            true
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    use super::{NotificationManager, NotificationStatus};
    use anyhow::Result;

    pub struct FallbackNotificationManager;

    impl NotificationManager for FallbackNotificationManager {
        fn send_notification(
            &self,
            status: NotificationStatus,
            title: &str,
            message: &str,
            _duration_ms: u64,
        ) -> Result<()> {
            // 后备方案：输出到终端
            let icon = match status {
                NotificationStatus::Success => "✅",
                NotificationStatus::Error => "❌",
                NotificationStatus::Pending => "⏳",
            };

            log::info!("后备通知: {} {} - {}", icon, title, message);
            println!("[通知] {} {}: {}", icon, title, message);
            Ok(())
        }

        fn is_available(&self) -> bool {
            false
        }
    }
}
