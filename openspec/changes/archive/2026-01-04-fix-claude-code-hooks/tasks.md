# 任务列表：修复 Claude Code Hooks 配置

## 阶段 1：修改 ccn notify 命令

- [x] **任务 1.1**：将 `--duration` 参数改为可选
  - 修改 `src/cli.rs`，将 `duration` 参数类型改为 `Option<u64>`
  - 添加 `default_value = "0"` 属性
  - 在调用处使用 `unwrap_or(0)` 解包
  - 验证：命令可以不带 `--duration` 参数执行

- [x] **任务 1.2**：修改策略引擎跳过 duration=0 的阈值检查
  - 修改 `src/policy.rs` 的 `should_notify` 方法
  - 添加 `duration == 0` 时直接返回 true 的逻辑
  - 验证：duration=0 的通知不会被阈值过滤

- [x] **任务 1.3**：修改聚合逻辑绕过 duration=0 或紧急通知
  - 修改 `src/cli.rs` 的聚合判断逻辑
  - 当 duration=0 或状态为 error/pending 时跳过聚合
  - 验证：紧急通知立即显示

## 阶段 2：更新 IntegrationManager

- [x] **任务 2.1**：修改 hooks 事件类型
  - 将 `PostToolUse`/`PostToolUseFailure` 改为 `PermissionRequest`
  - 更新 matcher 为 `Bash|Read|Write|Edit`
  - 移除旧的事件类型清理逻辑
  - 验证：生成的 hooks 配置符合预期

- [x] **任务 2.2**：简化 hooks 命令
  - 移除 `--duration=$DURATION` 参数（使用默认值 0）
  - 移除 `$COMMAND` 环境变量，使用固定文本
  - 修改状态为 `pending`，消息为 "Claude Code 需要授权"
  - 验证：hooks 命令不依赖环境变量

## 阶段 3：测试和验证

- [x] **任务 3.1**：测试 ccn notify 命令
  - 测试不带 `--duration` 参数的命令
  - 测试 `--duration=0` 的命令
  - 测试 error 和 pending 状态立即显示
  - 验证：所有测试通过

- [x] **任务 3.2**：端到端集成测试
  - 运行 `ccn uninstall` 清理旧配置
  - 运行 `ccn setup` 重新集成
  - 验证 settings.json 中的 hooks 配置正确
  - 测试通知发送功能
  - 验证：hooks 正常工作

## 阶段 4：更新规范和文档

- [x] **任务 4.1**：更新 integration 规范
  - 更新 hooks 配置相关的需求
  - 添加 duration 参数可选的需求
  - 添加策略引擎对 duration=0 的处理需求
  - 验证：`openspec-cn validate` 通过

- [x] **任务 4.2**：更新文档
  - 更新 README 中的 hooks 说明
  - 更新故障排查文档
  - 添加新的配置示例
  - 更新 CHANGELOG.md
  - 验证：文档准确反映新实现

## 完成状态

✅ **已完成**：所有阶段 (1、2、3、4)

## 关键决策

1. **不使用 Python/PowerShell 脚本**：简化部署，开箱即用
2. **使用 PermissionRequest 事件**：官方支持，触发准确
3. **duration 默认值为 0**：表示"未知/不适用"，跳过阈值检查
4. **紧急通知立即显示**：error/pending 状态绕过聚合
