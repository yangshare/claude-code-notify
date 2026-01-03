# 变更日志

本项目的所有重要变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增
- `ccn verify` 命令：验证 CCN 是否正确集成到 Claude Code
- 集成验证功能：自动测试 hooks 命令是否可执行
- README 新增故障排查章节，提供常见问题的解决方案

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
- 更新 README：添加 Windows PATH 配置说明
- 添加故障排查章节：涵盖配置文件路径、hooks 不工作、PATH 修改等常见问题
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
