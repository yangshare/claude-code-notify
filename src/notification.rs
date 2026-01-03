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
    use std::process::Command;
    use std::os::windows::process::CommandExt;

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

        /// 使用 PowerShell 发送系统托盘通知
        fn send_powershell_notification(&self, title: &str, message: &str) -> Result<()> {
            // 转义特殊字符
            let escaped_title = title
                .replace('\\', "\\\\")
                .replace('\"', "\\\"")
                .replace('`', "``");
            let escaped_message = message
                .replace('\\', "\\\\")
                .replace('\"', "\\\"")
                .replace('`', "``");

            // PowerShell 脚本：创建系统托盘气泡通知
            let script = format!(
                "Add-Type -AssemblyName System.Windows.Forms; \
                 $n = New-Object System.Windows.Forms.NotifyIcon; \
                 $n.Icon = [System.Drawing.SystemIcons]::Information; \
                 $n.BalloonTipTitle = '{}'; \
                 $n.BalloonTipText = '{}'; \
                 $n.Visible = $true; \
                 $n.ShowBalloonTip(10000); \
                 Start-Sleep -Seconds 10; \
                 $n.Dispose()",
                escaped_title, escaped_message
            );

            let output = Command::new("powershell")
                .args(["-ExecutionPolicy", "Bypass", "-Command", &script])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output();

            match output {
                Ok(o) if o.status.success() => Ok(()),
                Ok(e) => Err(anyhow::anyhow!("PowerShell 通知失败: {}", String::from_utf8_lossy(&e.stderr))),
                Err(e) => Err(anyhow::anyhow!("无法执行 PowerShell: {}", e)),
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

            // 发送 PowerShell 通知
            match self.send_powershell_notification(&formatted_title, message) {
                Ok(_) => {
                    log::info!("通知已发送: {}", formatted_title);
                    Ok(())
                }
                Err(e) => {
                    log::warn!("PowerShell 通知失败: {}", e);
                    // 降级到控制台输出
                    println!("[通知] {} {}: {}", icon, title, message);
                    Ok(())
                }
            }
        }

        fn is_available(&self) -> bool {
            // PowerShell 在 Windows 上总是可用
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
