# 变更：实现 Windows 11 原生 Toast 通知

## 为什么

当前 Windows 平台使用 PowerShell 调用 System.Windows.Forms.NotifyIcon 作为临时通知方案，该方案存在以下问题：
- 通知样式过时，不符合 Windows 11 的现代 UI 风格
- 需要启动 PowerShell 进程，性能开销大
- 可能在某些系统配置下不稳定或被安全软件拦截

用户期望获得与系统其他应用一致的原生 Windows Toast 通知体验。

## 变更内容

- 使用 `win32_notif` crate 实现原生 Windows Toast 通知
- 保留现有的 `NotificationManager` trait 抽象层
- 为 Windows 平台实现新的通知管理器
- 移除 PowerShell 临时实现代码
- 添加 AUMID（Application User Model ID）支持以正确识别应用
- 支持通知状态图标（成功/错误/进行中）的视觉展示
- 保留错误时的降级处理机制

## 影响

- **受影响规范**：新增 `notification` 规范
- **受影响代码**：
  - `src/notification.rs` (主要变更)
  - `Cargo.toml` (添加依赖)
- **用户体验**：通知样式更现代，性能更好，与系统一致
- **系统要求**：Windows 11
