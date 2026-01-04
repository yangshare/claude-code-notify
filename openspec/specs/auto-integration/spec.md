# auto-integration Specification

## Purpose
TBD - created by archiving change implement-ccn-core. Update Purpose after archive.
## 需求
### 需求：Claude Code 配置文件自动侦测
系统必须自动侦测 Claude Code 配置文件的位置。

#### 场景：在 Windows 上侦测配置文件
- **当** 用户在 Windows 系统执行 `ccn setup`
- **那么** 系统必须按以下优先级查找配置文件：
  1. `%APPDATA%\Claude\config.json`
  2. `%USERPROFILE%\.claude\config.json`
  3. 当前目录的 `.claude\config.json`

#### 场景：配置文件不存在
- **当** 系统无法找到 Claude Code 配置文件
- **那么** 必须提示用户手动指定路径或退出

#### 场景：找到配置文件
- **当** 系统成功找到配置文件
- **那么** 必须显示文件路径并请求用户确认

### 需求：配置文件自动备份
系统必须在修改配置文件前自动创建备份。

#### 场景：创建备份文件
- **当** 系统准备修改配置文件
- **那么** 必须在同级目录创建备份文件 `config.json.bak-[timestamp]`

#### 场景：备份已存在
- **当** 同名备份文件已存在
- **那么** 必须使用新的时间戳覆盖旧备份或创建新备份

### 需求：Hooks 字段安全注入
系统必须安全地在配置文件中插入 hooks 配置。

#### 场景：配置文件已有 hooks 字段
- **当** 配置文件已存在 `hooks` 字段
- **那么** 必须在该对象中添加或更新 `PostCommand` 和 `CommandError` 键

#### 场景：配置文件没有 hooks 字段
- **当** 配置文件不存在 `hooks` 字段
- **那么** 必须在根级别添加 `hooks` 对象，包含必要的钩子

#### 场景：注入 PostCommand 钩子
- **当** 系统注入 `PostCommand` 钩子
- **那么** 必须设置为：`"ccn notify --status=success --duration=$DURATION --cmd='$COMMAND'"`

#### 场景：注入 CommandError 钩子
- **当** 系统注入 `CommandError` 钩子
- **那么** 必须设置为：`"ccn notify --status=error --duration=$DURATION --cmd='$COMMAND'"`

### 需求：JSON 格式验证
系统必须在修改后验证 JSON 格式的正确性。

#### 场景：验证 JSON 格式
- **当** 系统完成配置文件修改
- **那么** 必须解析 JSON 验证语法正确性

#### 场景：JSON 格式错误
- **当** JSON 解析失败
- **那么** 必须恢复备份文件并显示错误信息

### 需求：测试通知验证
系统必须在集成完成后发送测试通知验证配置。

#### 场景：发送成功测试通知
- **当** 集成完成
- **那么** 必须发送一条测试通知显示 "CCN 集成成功！"

#### 场景：测试通知失败
- **当** 测试通知发送失败
- **那么** 必须显示错误信息和排查建议

### 需求：集成回滚
系统必须提供回滚集成的方法。

#### 场景：用户运行 ccn uninstall
- **当** 用户执行 `ccn uninstall` 命令
- **那么** 必须从配置文件中移除 hooks 字段或删除相关的 hook 配置

#### 场景：使用备份恢复
- **当** 用户选择从备份恢复
- **那么** 必须用最新的备份文件覆盖当前配置文件

