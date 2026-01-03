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

    pub struct WindowsNotificationManager;

    impl WindowsNotificationManager {
        pub fn new() -> Self {
            Self
        }

        /// 获取状态图标
        fn get_status_icon(status: NotificationStatus) -> char {
            match status {
                NotificationStatus::Success => '✅',
                NotificationStatus::Error => '❌',
                NotificationStatus::Pending => '⏳',
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
            // Windows Toast 通知需要 COM 初始化，在 MVP 版本中先使用简化版本
            self.send_fallback_notification(status, title, message);

            log::info!("Windows 通知已发送（使用简化版本）");
            Ok(())
        }

        fn is_available(&self) -> bool {
            true
        }
    }

    impl WindowsNotificationManager {
        fn send_fallback_notification(&self, status: NotificationStatus, title: &str, message: &str) {
            let icon = Self::get_status_icon(status);
            println!("[通知] {} {}: {}", icon, title, message);
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
