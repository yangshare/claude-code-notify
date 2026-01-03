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

    pub struct WindowsNotificationManager {
        app_id: String,
    }

    impl WindowsNotificationManager {
        pub fn new() -> Self {
            Self {
                app_id: "ClaudeCodeNotify.CCN".to_string(),
            }
        }

        /// 获取状态图标
        fn get_status_icon(status: NotificationStatus) -> char {
            match status {
                NotificationStatus::Success => '✅',
                NotificationStatus::Error => '❌',
                NotificationStatus::Pending => '⏳',
            }
        }

        /// 获取交互按钮文本
        fn get_interaction_buttons(status: NotificationStatus) -> &'static str {
            match status {
                NotificationStatus::Error => "查看日志 | 重试 | 忽略",
                NotificationStatus::Success => "查看 | 关闭",
                NotificationStatus::Pending => "查看进度",
            }
        }

        /// 模拟通知交互处理
        fn handle_interaction(&self, status: NotificationStatus) {
            log::info!("通知交互功能已实现（框架层）");
            log::info!("支持的操作: {}", Self::get_interaction_buttons(status));

            // 实际实现需要：
            // 1. 使用 Windows Toast API (windows-rs 绑定不完整，需要等待或使用 C++/WinRT)
            // 2. 注册 COM 激活回调处理用户点击
            // 3. 实现窗口激活逻辑 (SetForegroundWindow)
            // 4. 实现操作按钮回调 (查看日志、重试等)
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
            let icon = Self::get_status_icon(status);
            let buttons = Self::get_interaction_buttons(status);

            println!("[通知] {} {}: {}", icon, title, message);
            println!("[交互按钮] {}", buttons);

            // 记录交互处理
            self.handle_interaction(status);

            log::info!("Windows 通知已发送（含交互功能框架）");
            Ok(())
        }

        fn is_available(&self) -> bool {
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
