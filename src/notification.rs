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

    pub struct MacOSNotificationManager;

    impl MacOSNotificationManager {
        pub fn new() -> Self {
            Self
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
            // macOS 通知实现
            let icon = match status {
                NotificationStatus::Success => "✅",
                NotificationStatus::Error => "❌",
                NotificationStatus::Pending => "⏳",
            };

            log::info!("macOS 通知: {} {} - {}", icon, title, message);
            println!("[macOS 通知] {} {}: {}", icon, title, message);

            // TODO: 使用 cocoa 和 objc 实现真正的 macOS 通知
            Ok(())
        }

        fn is_available(&self) -> bool {
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
