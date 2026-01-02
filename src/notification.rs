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
        Box::new(super::notification::platform::WindowsNotificationManager::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(super::notification::platform::MacOSNotificationManager::new())
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        Box::new(super::notification::platform::FallbackNotificationManager)
    }
}

// 平台特定实现将在下面提供

#[cfg(windows)]
mod platform {
    use super::{NotificationManager, NotificationStatus, Result};
    use windows::{core::*, Win32::UI::Notifications::*};

    pub struct WindowsNotificationManager;

    impl WindowsNotificationManager {
        pub fn new() -> Self {
            Self
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
            // TODO: 实现完整的 Windows 通知逻辑
            log::info!("Windows 通知: {:?} - {} - {}", status, title, message);
            println!("[Windows 通知] {}: {}", title, message);
            Ok(())
        }

        fn is_available(&self) -> bool {
            true
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{NotificationManager, NotificationStatus, Result};

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
            // TODO: 实现 macOS 通知逻辑
            log::info!("macOS 通知: {:?} - {} - {}", status, title, message);
            println!("[macOS 通知] {}: {}", title, message);
            Ok(())
        }

        fn is_available(&self) -> bool {
            true
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    use super::{NotificationManager, NotificationStatus, Result};

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
            log::info!("后备通知: {:?} - {} - {}", status, title, message);
            println!("[通知] {}: {}", title, message);
            Ok(())
        }

        fn is_available(&self) -> bool {
            false
        }
    }
}
