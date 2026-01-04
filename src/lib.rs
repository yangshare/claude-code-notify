//! Claude Code Notify - 优雅的任务通知工具
//!
//! 这是一个为 Claude Code 设计的通知系统，可以在任务完成时发送系统级通知。

pub mod cli;
pub mod config;
pub mod notification;
pub mod policy;
pub mod aggregator;
pub mod integration;
pub mod wizard;
pub mod sound;

#[cfg(windows)]
pub mod path_manager;

/// 平台信息模块
pub mod platform {
    /// 检测当前操作系统平台
    pub const OS_WINDOWS: bool = cfg!(windows);
    pub const OS_MACOS: bool = cfg!(target_os = "macos");
    pub const OS_LINUX: bool = cfg!(target_os = "linux");
    pub const OS_UNKNOWN: bool = !(OS_WINDOWS || OS_MACOS || OS_LINUX);

    /// 获取平台名称
    pub fn platform_name() -> &'static str {
        if OS_WINDOWS {
            "Windows"
        } else if OS_MACOS {
            "macOS"
        } else if OS_LINUX {
            "Linux"
        } else {
            "Unknown"
        }
    }

    /// 检查是否支持原生通知
    pub fn supports_native_notifications() -> bool {
        OS_WINDOWS || OS_MACOS
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_platform_detection() {
            // 确保只有一个平台被检测为 true
            let platforms = vec![OS_WINDOWS, OS_MACOS, OS_LINUX];
            let detected_count = platforms.iter().filter(|&&p| p).count();

            // 应该至少有一个平台被检测到（可能是 unknown）
            assert!(detected_count <= 1 || (detected_count == 0 && OS_UNKNOWN));
        }

        #[test]
        fn test_platform_name() {
            let name = platform_name();
            assert!(!name.is_empty());
            assert!(name == "Windows" || name == "macOS" || name == "Linux" || name == "Unknown");
        }

        #[test]
        fn test_supports_native_notifications() {
            let supported = supports_native_notifications();
            // Windows 和 macOS 应该支持原生通知
            if OS_WINDOWS || OS_MACOS {
                assert!(supported);
            }
        }
    }
}
