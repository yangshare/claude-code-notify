## 新增需求

### 需求：hooks 使用 PermissionRequest 事件

系统必须使用 Claude Code 官方文档中定义的正确事件类型（PermissionRequest）进行 hooks 配置，而非不存在的 PostToolUseFailure 事件。

#### 场景：注入 PermissionRequest hooks 到 settings.json
- **当** Claude Code 配置文件位于 `~/.claude/settings.json`
- **且** 用户执行 `ccn setup`
- **那么** 配置文件必须包含 `PermissionRequest` hooks 配置
- **且** matcher 必须为 `Bash|Read|Write|Edit`
- **且** command 必须调用 `ccn notify --status=pending`
- **且** hooks 配置格式必须符合：
  ```json
  {
    "hooks": {
      "PermissionRequest": [
        {
          "matcher": "Bash|Read|Write|Edit",
          "hooks": [
            {
              "type": "command",
              "command": "ccn notify --status=pending --cmd='Claude Code 需要授权' || true"
            }
          ]
        }
      ]
    }
  }
  ```

#### 场景：权限请求时发送通知
- **当** Claude Code 显示权限请求对话框
- **且** PermissionRequest hook 被触发
- **那么** 必须调用 `ccn notify --status=pending`
- **且** 通知标题应包含"任务进行中"
- **且** 通知消息应提示需要授权
- **且** 通知应立即显示在 Windows 通知中心

## 新增需求

### 需求：notify 命令 duration 参数可选

`ccn notify` 命令的 `--duration` 参数必须为可选，默认值为 0，以支持 hooks 场景（无法获取执行时长）。

#### 场景：不提供 duration 参数执行通知
- **当** 用户执行 `ccn notify --status=success --cmd="test"`
- **且** 不提供 `--duration` 参数
- **那么** 命令必须成功执行
- **且** duration 必须使用默认值 0
- **且** 通知必须正常显示

#### 场景：duration 为 0 时显示通知
- **当** `ccn notify` 命令的 duration 参数为 0
- **且** 状态为 success 或 pending
- **那么** 必须跳过时间阈值检查（不应用 min_duration 过滤）
- **且** 通知必须正常显示

## 新增需求

### 需求：紧急通知立即发送

系统必须立即发送紧急通知（duration=0 或 error/pending 状态），绕过聚合功能，确保用户及时看到重要提醒。

#### 场景：duration 为 0 时绕过聚合
- **当** `ccn notify` 的 duration 参数为 0
- **且** 聚合功能已启用
- **那么** 必须跳过聚合，立即发送通知
- **且** 通知必须在 1 秒内显示

#### 场景：error 或 pending 状态绕过聚合
- **当** 通知状态为 `error` 或 `pending`
- **且** 聚合功能已启用
- **那么** 必须跳过聚合，立即发送通知
- **且** 通知必须在 1 秒内显示
