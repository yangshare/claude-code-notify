//! PATH 环境变量管理模块
//!
//! 处理 Windows 系统 PATH 环境变量的添加和移除

#![allow(dead_code)]

use anyhow::Result;
use std::path::PathBuf;

/// PATH 管理器
pub struct PathManager;

impl PathManager {
    /// 将指定目录添加到用户 PATH
    #[cfg(windows)]
    pub fn add_to_path(directory: &PathBuf) -> Result<bool> {
        // 1. 获取当前用户 PATH
        let current_path = Self::get_user_path()?;

        // 2. 检查是否已存在
        if Self::contains_path(&current_path, directory) {
            log::info!("PATH 已包含: {:?}", directory);
            return Ok(false);
        }

        // 3. 添加到 PATH
        let new_path = Self::append_to_path(&current_path, directory);

        // 4. 更新注册表
        Self::set_user_path(&new_path)?;

        // 5. 通知系统环境变量已更改
        Self::notify_environment_change();

        log::info!("已添加到 PATH: {:?}", directory);
        Ok(true)
    }

    /// 将指定目录添加到用户 PATH (非 Windows 平台)
    #[cfg(not(windows))]
    pub fn add_to_path(_directory: &PathBuf) -> Result<bool> {
        // Unix 系统通常通过包管理器安装，已在 PATH 中
        log::info!("非 Windows 平台，跳过 PATH 修改");
        Ok(false)
    }

    /// 从用户 PATH 中移除指定目录
    #[cfg(windows)]
    pub fn remove_from_path(directory: &PathBuf) -> Result<bool> {
        // 1. 获取当前用户 PATH
        let current_path = Self::get_user_path()?;

        // 2. 检查是否存在
        if !Self::contains_path(&current_path, directory) {
            log::info!("PATH 中不存在: {:?}", directory);
            return Ok(false);
        }

        // 3. 从 PATH 移除
        let new_path = Self::remove_from_path_str(&current_path, directory);

        // 4. 更新注册表
        Self::set_user_path(&new_path)?;

        // 5. 通知系统环境变量已更改
        Self::notify_environment_change();

        log::info!("已从 PATH 移除: {:?}", directory);
        Ok(true)
    }

    /// 从用户 PATH 中移除指定目录 (非 Windows 平台)
    #[cfg(not(windows))]
    pub fn remove_from_path(_directory: &PathBuf) -> Result<bool> {
        log::info!("非 Windows 平台，跳过 PATH 修改");
        Ok(false)
    }

    /// 获取用户 PATH
    #[cfg(windows)]
    fn get_user_path() -> Result<String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        let environment = hklm.open_subkey_with_flags("Environment", KEY_READ)?;
        let path: String = environment.get_value("Path").unwrap_or_default();
        Ok(path)
    }

    /// 设置用户 PATH
    #[cfg(windows)]
    fn set_user_path(path: &str) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        let environment = hklm.open_subkey_with_flags("Environment", KEY_WRITE)?;
        environment.set_value("Path", &path)?;

        Ok(())
    }

    /// 检查 PATH 是否包含指定目录
    fn contains_path(path: &str, directory: &PathBuf) -> bool {
        let dir_str = directory.to_string_lossy().to_lowercase();
        path.to_lowercase().contains(&dir_str)
    }

    /// 将目录追加到 PATH
    fn append_to_path(path: &str, directory: &PathBuf) -> String {
        let dir_str = directory.to_string_lossy().to_string();
        if path.is_empty() {
            dir_str
        } else {
            format!("{};{}", path, dir_str)
        }
    }

    /// 从 PATH 字符串中移除指定目录
    fn remove_from_path_str(path: &str, directory: &PathBuf) -> String {
        let dir_str = directory.to_string_lossy().to_string();

        path.split(';')
            .filter(|p| !p.eq(&dir_str.as_str()))
            .collect::<Vec<_>>()
            .join(";")
    }

    /// 通知系统环境变量已更改
    #[cfg(windows)]
    fn notify_environment_change() {
        use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG};
        use windows::Win32::Foundation::{WPARAM, LPARAM};

        unsafe {
            let msg = "Environment\0".encode_utf16().collect::<Vec<u16>>();
            let _ = SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(msg.as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                5000,
                None,
            );
        }
    }
}
