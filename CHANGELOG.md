# 变更日志

本项目的所有重要变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增
- **Windows 11 原生 Toast 通知**：使用微软官方 `windows` crate (WinRT API) 实现真正的 Windows 11 Toast 通知
  - 替换之前的 PowerShell 临时方案
  - 使用 AUMID (`ClaudeCodeNotify.CCN`) 进行通知管理
  - 支持状态感知图标（✅成功、❌错误、⏳进行中）
  - 符合 Windows 11 Fluent Design 风格
  - 自动处理 XML 模板和类型转换
- 错误降级机制：Toast 通知失败时自动降级到控制台输出
- `ccn verify` 命令：验证 CCN 是否正确集成到 Claude Code
- 集成验证功能：自动测试 hooks 命令是否可执行
- README 新增故障排查章节，提供常见问题的解决方案

### 变更
- **系统要求更新**：最低系统要求从 Windows 10 更新为 Windows 11
- **依赖更新**：
  - 新增 `windows = "0.61"` 用于 Windows Toast 通知（WinRT API）
  - features: `Data_Xml_Dom`, `UI_Notifications`, `Win32_Foundation`, `Win32_UI_Notifications`, `Win32_UI_WindowsAndMessaging`, `Win32_System_Registry`
  - 保留 `winreg = "0.52"` 用于 PATH 管理

### 修复
- 修正配置文件路径检测逻辑
  - 统一使用 `~/.claude/settings.json`（官方规范路径）
  - 移除错误的 `config.json` 文件名检测
  - 修正 Windows 平台路径检测优先级
- 新增 Windows PATH 自动配置功能
  - `ccn setup` 命令自动将 ccn.exe 添加到用户 PATH
  - `ccn uninstall` 命令自动清理 PATH 中的条目
- 添加 `CLAUDE_CONFIG_DIR` 环境变量支持
- 改进用户提示：明确告知 Windows 用户需要重启终端使 PATH 生效

### 文档
- 更新 README：添加 Windows 11 系统要求说明
- 更新 README：添加 Windows Toast 通知功能说明
- 更新 README：添加 AUMID 相关故障排查信息
- 更新技术栈说明：`windows-rs` → `win32_notif`
- 更新路线图：标记 "真正的 Windows Toast 通知" 为已完成
- 添加故障排查章节：涵盖配置文件路径、hooks 不工作、PATH 修改、通知不显示等常见问题
- 添加 `ccn verify` 命令的使用说明

## [0.1.0] - 2024-XX-XX

### 新增
- 首次发布
- CLI 核心功能：notify、init、setup、uninstall、config、test 命令
- Windows 原生通知支持
- 智能通知策略：阈值过滤、白名单命令
- 通知聚合功能
- 交互式配置向导
- 自动集成到 Claude Code
- 场景化通知模板
- 自定义音效支持

[未发布]: https://github.com/yangshare/claude-code-notify/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yangshare/claude-code-notify/releases/tag/v0.1.0
