# 变更：修复 Claude Code Hooks 配置

## 为什么

当前 Claude Code hooks 配置存在严重错误，导致 hooks 功能完全无法工作。具体问题包括：
1. 使用了不存在的 `PostToolUseFailure` 事件类型（官方文档中没有此事件）
2. 使用了不存在的环境变量 `$DURATION` 和 `$COMMAND`（hooks 通过 stdin 传递 JSON 输入）
3. `ccn notify` 命令的 `--duration` 参数是必需的，无法处理空值情况

这些错误导致用户在配置 hooks 后无法收到任何通知，违背了项目的核心功能目标。

## 变更内容

- **修改 `ccn notify` 命令**：将 `--duration` 参数改为可选，默认值为 0
- **修改策略引擎**：当 duration 为 0 时跳过时间阈值检查（适用于 hooks 场景）
- **修改聚合逻辑**：当 duration 为 0 或状态为 error/pending 时立即发送通知（绕过聚合）
- **修改 hooks 事件类型**：从错误的 `PostToolUse`/`PostToolUseFailure` 改为正确的 `PermissionRequest`
- **简化 hooks 命令**：移除环境变量依赖，使用固定文本，不需要额外脚本
- **扩大匹配范围**：从 `Bash` 扩展到 `Bash|Read|Write|Edit`

## 影响

- **受影响规范**：integration（修改 hooks 配置需求和策略相关需求）
- **受影响代码**：
  - `src/cli.rs` - 修改 `--duration` 参数为可选
  - `src/policy.rs` - duration 为 0 时跳过阈值检查
  - `src/integration.rs` - 更新 hooks 注入逻辑
- **受影响文档**：README、故障排查文档
