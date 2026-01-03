# notification Specification

## Purpose
TBD - created by archiving change implement-windows-toast-notifications. Update Purpose after archive.
## 需求
### 需求：跨平台通知抽象

系统必须提供跨平台的通知抽象层，通过 `NotificationManager` trait 定义统一接口，支持不同操作系统的具体实现。

#### 场景：获取平台特定的通知管理器
- **当** 代码调用 `get_notification_manager()`
- **且** 当前运行在 Windows 平台
- **那么** 必须返回 `WindowsNotificationManager` 实例
- **且** 该实例必须实现 `NotificationManager` trait

#### 场景：macOS 平台获取通知管理器
- **当** 代码调用 `get_notification_manager()`
- **且** 当前运行在 macOS 平台
- **那么** 必须返回 `MacOSNotificationManager` 实例
- **且** 该实例必须实现 `NotificationManager` trait

#### 场景：其他平台使用后备管理器
- **当** 代码调用 `get_notification_manager()`
- **且** 当前运行在 Linux 或其他平台
- **那么** 必须返回 `FallbackNotificationManager` 实例
- **且** 该实例必须实现 `NotificationManager` trait
- **且** `is_available()` 必须返回 `false`

### 需求：通知状态

系统必须支持三种通知状态，用于表示不同的通知类型和视觉样式。

#### 场景：成功状态通知
- **当** 通知状态为 `NotificationStatus::Success`
- **那么** 通知必须显示成功图标（✅）
- **且** 通知标题应包含"成功"文本

#### 场景：错误状态通知
- **当** 通知状态为 `NotificationStatus::Error`
- **那么** 通知必须显示错误图标（❌）
- **且** 通知标题应包含"错误"文本

#### 场景：进行中状态通知
- **当** 通知状态为 `NotificationStatus::Pending`
- **那么** 通知必须显示进行中图标（⏳）
- **且** 通知标题应包含"进行中"文本

### 需求：Windows 原生 Toast 通知

Windows 平台必须使用原生 Windows Runtime (WinRT) API 实现 Toast 通知，提供符合 Windows 11 设计规范的现代通知体验。

#### 场景：发送成功通知
- **当** 用户在 Windows 11 系统上执行 `ccn notify --status=success`
- **且** 标题为"构建完成"，消息为"项目已成功编译"
- **那么** 必须显示原生 Windows Toast 通知
- **且** 通知必须包含成功图标（✅）
- **且** 通知标题必须为"✅成功 - 构建完成"
- **且** 通知消息必须为"项目已成功编译"
- **且** 通知必须出现在 Windows 通知中心
- **且** 通知必须符合 Windows 11 Fluent Design 风格

#### 场景：发送错误通知
- **当** 用户在 Windows 11 系统上执行 `ccn notify --status=error`
- **且** 标题为"构建失败"，消息为"编译错误：语法错误"
- **那么** 必须显示原生 Windows Toast 通知
- **且** 通知必须包含错误图标（❌）
- **且** 通知标题必须为"❌错误 - 构建失败"
- **且** 通知消息必须为"编译错误：语法错误"

#### 场景：发送进行中通知
- **当** 用户在 Windows 11 系统上执行 `ccn notify --status=pending`
- **且** 标题为"构建中"，消息为"正在编译项目..."
- **那么** 必须显示原生 Windows Toast 通知
- **且** 通知必须包含进行中图标（⏳）
- **且** 通知标题必须为"⏳进行中 - 构建中"
- **且** 通知消息必须为"正在编译项目..."

#### 场景：首次初始化创建 AUMID
- **当** Windows 用户首次运行 CCN 通知功能
- **且** 系统中不存在 CCN 的 AUMID (Application User Model ID)
- **那么** 必须自动创建 AUMID: `ClaudeCodeNotify.CCN`
- **且** 必须在用户 AppData 目录创建应用快捷方式
- **且** 后续通知必须使用此 AUMID

#### 场景：重复使用已存在的 AUMID
- **当** Windows用户已运行过 CCN 通知功能
- **且** 系统中已存在 CCN 的 AUMID
- **那么** 必须复用现有 AUMID
- **且** 不应重复创建快捷方式

#### 场景：通知失败时降级到控制台
- **当** Windows Toast 通知初始化失败
- **或** 显示通知失败
- **那么** 必须降级到控制台输出
- **且** 必须输出格式：`[通知] ⚠️ <标题>: <消息>`
- **且** 必须记录详细错误日志
- **且** 不应抛出异常中断程序

### 需求：Windows 通知可用性检测

系统必须能够检测 Windows 通知功能是否可用。

#### 场景：Windows 11 系统通知可用
- **当** Windows 版本为 Windows 11
- **且** 通知服务已启用
- **那么** `is_available()` 必须返回 `true`

### 需求：通知持续时间

通知系统必须支持持续时间参数（当前未使用，保留用于未来扩展）。

#### 场景：传递持续时间参数
- **当** 调用 `send_notification()` 时传递 `duration_ms: 5000`
- **那么** Windows 实现应使用系统默认持续时间
- **且** 参数必须被接受（即使当前不使用）
- **且** 不应抛出参数相关异常

### 需求：错误处理

通知系统必须正确处理各种错误情况，确保不会影响主程序流程。

#### 场景：通知管理器初始化失败
- **当** Windows Toast 初始化失败（如 WinRT 运行时不可用）
- **那么** 必须返回错误而非 panic
- **且** 上层调用者可以处理错误
- **且** 必须记录详细错误信息

#### 场景：特殊字符转义
- **当** 通知标题或消息包含特殊字符（如 `<`、`>`、`&`、`"`）
- **那么** 必须正确转义这些字符
- **且** 通知必须正确显示，不应出现格式问题
- **且** 不应导致 XML 解析错误（如使用 XML）

