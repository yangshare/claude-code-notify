# 变更：修复 Claude Code 集成配置问题

## 为什么

当前 CCN 的 Claude Code 集成功能存在两个关键问题：

1. **配置文件路径不符合官方规范**：当前代码检测 `config.json`，而 Claude Code 官方配置文件名为 `settings.json`。此外，Windows 平台的路径检测逻辑与其他平台不一致，优先检测 `%APPDATA%\Claude\` 而非统一的 `~/.claude/` 路径。

2. **Windows 环境变量未配置**：`ccn setup` 命令虽然会注入 hooks 到 settings.json，但不会将 `ccn.exe` 添加到系统 PATH 环境变量。这导致 hooks 命令（如 `ccn notify --status=success`）无法执行，因为 Windows 找不到 `ccn.exe`。

这些问题导致用户在使用 VS Code 插件方式安装 Claude Code 时，集成功能完全失效。

## 变更内容

- **修正配置文件检测逻辑**：
  - 统一所有平台使用 `~/.claude/settings.json` 作为主要配置路径
  - 支持 `CLAUDE_CONFIG_DIR` 环境变量自定义配置目录
  - 移除错误的 `config.json` 文件名检测

- **新增 Windows PATH 配置功能**：
  - `ccn setup` 命令在 Windows 上自动将 ccn.exe 所在目录添加到用户 PATH 环境变量
  - 提供回滚功能，`ccn uninstall` 时从 PATH 中移除相关条目

- **改进路径侦测逻辑**：
  - 支持跨平台统一路径处理
  - 添加配置文件存在性验证
  - 改进错误提示，指导用户手动配置

## 影响

- **受影响规范**：
  - 集成管理规范（integration）
  - 配置管理规范（config）

- **受影响代码**：
  - `src/integration.rs` - 配置文件路径检测逻辑
  - `src/main.rs` - setup/uninstall 命令处理
  - `src/config.rs` - 配置管理相关代码（如存在）

- **向后兼容性**：
  - 此变更是修复性变更，不涉及 API 变更
  - 用户需要重新运行 `ccn setup` 以应用修复
