# API 参考文档

本文档提供 Claude Code Notify (CCN) 的详细 API 参考，包括所有公共模块、结构体、枚举和函数。

## 目录

- [核心模块](#核心模块)
  - [配置管理 (config)](#配置管理-config)
  - [通知管理 (notification)](#通知管理-notification)
  - [策略引擎 (policy)](#策略引擎-policy)
  - [通知聚合 (aggregator)](#通知聚合-aggregator)
  - [自动集成 (integration)](#自动集成-integration)
  - [配置向导 (wizard)](#配置向导-wizard)
  - [CLI 接口 (cli)](#cli-接口-cli)
  - [平台检测 (platform)](#平台检测-platform)

---

## 核心模块

### 配置管理 (config)

配置管理模块负责处理配置文件的读取、写入和验证。

#### 结构体

##### `Config`

主配置结构体，包含所有可配置的选项。

```rust
pub struct Config {
    pub version: String,
    pub sound_enabled: bool,
    pub focus_assistant_mode: FocusAssistantMode,
    pub threshold: ThresholdConfig,
    pub templates: TemplatesConfig,
    pub aggregation: AggregationConfig,
    pub logging: LoggingConfig,
}
```

**字段说明：**

- `version: String` - 配置文件版本号
- `sound_enabled: bool` - 是否启用通知声音
- `focus_assistant_mode: FocusAssistantMode` - 专注助手模式
- `threshold: ThresholdConfig` - 阈值配置
- `templates: TemplatesConfig` - 通知模板配置
- `aggregation: AggregationConfig` - 聚合配置
- `logging: LoggingConfig` - 日志配置

**实现：**

- `Default` - 提供默认配置值

---

##### `ThresholdConfig`

通知阈值配置。

```rust
pub struct ThresholdConfig {
    pub min_duration: u64,
    pub whitelist: Vec<String>,
}
```

**字段说明：**

- `min_duration: u64` - 最小通知时长（秒），低于此值的成功任务不会通知
- `whitelist: Vec<String>` - 白名单命令列表，这些命令无论耗时多少都会通知

---

##### `TemplatesConfig`

通知模板配置，支持场景化模板。

```rust
pub struct TemplatesConfig {
    pub default: TemplateConfig,
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, TemplateConfig>,
}
```

**字段说明：**

- `default: TemplateConfig` - 默认模板
- `custom: HashMap<String, TemplateConfig>` - 自定义场景模板（如 build、test 等）

---

##### `TemplateConfig`

单个模板配置。

```rust
pub struct TemplateConfig {
    pub icon: String,
    pub sound: String,
    pub duration: u64,
}
```

**字段说明：**

- `icon: String` - 图标路径或 "auto" 表示自动选择
- `sound: String` - 声音文件路径或 "default" 表示系统默认音效
- `duration: u64` - 通知显示时长（毫秒）

---

##### `AggregationConfig`

通知聚合配置。

```rust
pub struct AggregationConfig {
    pub enabled: bool,
    pub window: u64,
    pub max_toasts: usize,
}
```

**字段说明：**

- `enabled: bool` - 是否启用聚合
- `window: u64` - 聚合时间窗口（毫秒）
- `max_toasts: usize` - 最多聚合多少条通知后发送

---

##### `LoggingConfig`

日志配置。

```rust
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}
```

**字段说明：**

- `level: String` - 日志级别（debug、info、warn、error）
- `file: Option<String>` - 日志文件路径，None 表示仅输出到 stderr

---

#### 枚举

##### `FocusAssistantMode`

专注助手模式，控制声音通知行为。

```rust
pub enum FocusAssistantMode {
    Respect,  // 尊重系统专注助手设置
    Always,   // 始终播放声音
    Never,    // 从不播放声音
}
```

---

#### 函数

##### `get_config_path() -> PathBuf`

获取配置文件的路径。

**返回值：**

- `PathBuf` - 配置文件的完整路径

**平台特定路径：**

- Windows: `%APPDATA%\claude-code-notify\config.yaml`
- macOS: `~/Library/Application Support/claude-code-notify/config.yaml`
- Linux: `~/.config/claude-code-notify/config.yaml`

**示例：**

```rust
use ccn::config::get_config_path;

let path = get_config_path();
println!("配置文件路径: {:?}", path);
```

---

##### `load_config() -> Result<Config>`

加载配置文件，如果不存在则创建默认配置。

**返回值：**

- `Result<Config>` - 成功返回配置对象，失败返回错误

**错误：**

- 配置文件格式错误
- 无法读取配置文件

**示例：**

```rust
use ccn::config::load_config;

match load_config() {
    Ok(config) => println!("配置加载成功，版本: {}", config.version),
    Err(e) => eprintln!("配置加载失败: {}", e),
}
```

---

##### `save_config(config: &Config) -> Result<()>`

保存配置到文件。

**参数：**

- `config: &Config` - 要保存的配置对象

**返回值：**

- `Result<()>` - 成功返回 Ok(())，失败返回错误

**错误：**

- 无法创建配置目录
- 无法写入配置文件

**示例：**

```rust
use ccn::config::{save_config, Config};

let config = Config::default();
save_config(&config)?;
```

---

### 通知管理 (notification)

通知管理模块提供平台抽象层，处理不同操作系统的通知功能。

#### 枚举

##### `NotificationStatus`

通知状态枚举。

```rust
pub enum NotificationStatus {
    Success,  // 任务成功完成
    Error,    // 任务执行失败
    Pending,  // 任务进行中
}
```

---

#### Trait

##### `NotificationManager`

通知管理器 trait，定义了所有平台通知管理器的通用接口。

```rust
pub trait NotificationManager {
    /// 发送通知
    fn send_notification(
        &self,
        status: NotificationStatus,
        title: &str,
        message: &str,
        duration_ms: u64,
    ) -> Result<()>;

    /// 检查通知是否可用
    fn is_available(&self) -> bool;
}
```

**方法说明：**

###### `send_notification()`

发送系统通知。

**参数：**

- `status: NotificationStatus` - 通知状态，决定显示的图标
- `title: &str` - 通知标题
- `message: &str` - 通知内容
- `duration_ms: u64` - 通知显示时长（毫秒）

**返回值：**

- `Result<()>` - 成功返回 Ok(())，失败返回错误

**示例：**

```rust
use ccn::notification::{get_notification_manager, NotificationStatus};

let notifier = get_notification_manager();
notifier.send_notification(
    NotificationStatus::Success,
    "任务完成",
    "npm test 执行成功",
    5000,
)?;
```

###### `is_available()`

检查通知系统是否可用。

**返回值：**

- `bool` - true 表示可用，false 表示不可用

**示例：**

```rust
use ccn::notification::get_notification_manager;

let notifier = get_notification_manager();
if notifier.is_available() {
    println!("通知系统可用");
}
```

---

#### 函数

##### `get_notification_manager() -> Box<dyn NotificationManager>`

获取平台特定的通知管理器实例。

**返回值：**

- `Box<dyn NotificationManager>` - 平台特定的通知管理器

**平台实现：**

- Windows: `WindowsNotificationManager`
- macOS: `MacOSNotificationManager`
- 其他: `FallbackNotificationManager`（控制台输出）

**示例：**

```rust
use ccn::notification::get_notification_manager;

let notifier = get_notification_manager();
// 使用 notifier 发送通知...
```

---

### 策略引擎 (policy)

策略引擎模块处理智能通知策略，包括阈值过滤、白名单和模板匹配。

#### 结构体

##### `PolicyEngine`

智能策略引擎。

```rust
pub struct PolicyEngine {
    config: Config,
}
```

---

#### 方法

##### `new(config: Config) -> PolicyEngine`

创建策略引擎实例。

**参数：**

- `config: Config` - 配置对象

**返回值：**

- `PolicyEngine` - 策略引擎实例

**示例：**

```rust
use ccn::config::load_config;
use ccn::policy::PolicyEngine;

let config = load_config()?;
let engine = PolicyEngine::new(config);
```

---

##### `should_notify(&self, status: NotificationStatus, duration_sec: u64, cmd: &str) -> bool`

检查是否应该发送通知。

**参数：**

- `status: NotificationStatus` - 任务状态
- `duration_sec: u64` - 任务耗时（秒）
- `cmd: &str` - 执行的命令

**返回值：**

- `bool` - true 表示应该通知，false 表示应该过滤

**策略逻辑：**

1. 错误状态始终通知
2. duration 为 0 时跳过阈值检查（适用于 hooks 场景）
3. 白名单中的命令始终通知
4. 其他情况检查时间阈值

**示例：**

```rust
use ccn::notification::NotificationStatus;

let should_send = engine.should_notify(
    NotificationStatus::Success,
    15,
    "npm test"
);
```

---

##### `match_template(&self, cmd: &str) -> Option<String>`

匹配场景模板。

**参数：**

- `cmd: &str` - 执行的命令

**返回值：**

- `Option<String>` - 匹配的模板名称，Some("default") 表示默认模板

**示例：**

```rust
let template_name = engine.match_template("npm run build");
// 返回 Some("build") 或 Some("default")
```

---

##### `should_aggregate(&self) -> bool`

检查是否应该聚合通知。

**返回值：**

- `bool` - true 表示启用聚合

---

##### `aggregation_window(&self) -> u64`

获取聚合窗口时间。

**返回值：**

- `u64` - 聚合窗口时间（毫秒）

---

### 通知聚合 (aggregator)

通知聚合模块管理在短时间内连续触发的多条通知的聚合和批量发送。

#### 结构体

##### `NotificationAggregator`

通知聚合器。

```rust
pub struct NotificationAggregator {
    state_file: PathBuf,
    window_ms: u64,
    max_toasts: usize,
}
```

---

#### 方法

##### `new(state_file: PathBuf, window_ms: u64, max_toasts: usize) -> NotificationAggregator`

创建聚合器实例。

**参数：**

- `state_file: PathBuf` - 状态文件路径
- `window_ms: u64` - 聚合时间窗口（毫秒）
- `max_toasts: usize` - 最大聚合数量

**返回值：**

- `NotificationAggregator` - 聚合器实例

**示例：**

```rust
use ccn::aggregator::NotificationAggregator;

let aggregator = NotificationAggregator::new(
    state_path,
    5000,  // 5秒窗口
    3,     // 最多聚合3条
);
```

---

##### `add_notification(&self, status: &str, duration: u64, cmd: &str) -> Result<Option<AggregatedResult>>`

添加通知到聚合器。

**参数：**

- `status: &str` - 通知状态（"success"、"error"）
- `duration: u64` - 任务耗时（秒）
- `cmd: &str` - 执行的命令

**返回值：**

- `Result<Option<AggregatedResult>>`
  - `Ok(Some(result))` - 达到聚合条件，返回聚合结果
  - `Ok(None)` - 添加到缓冲区，暂不发送
  - `Err(e)` - 聚合失败

**示例：**

```rust
match aggregator.add_notification("success", 15, "npm test")? {
    Some(result) => {
        // 发送聚合通知
        println!("聚合通知: {} 个任务", result.total);
    }
    None => {
        // 等待更多通知
    }
}
```

---

##### `flush(&self) -> Result<Option<AggregatedResult>>`

刷新待发送的通知。

**返回值：**

- `Result<Option<AggregatedResult>>` - 如果有待发送通知返回结果，否则返回 None

---

##### `has_pending(&self) -> bool`

检查是否有待发送的通知。

**返回值：**

- `bool` - true 表示有待发送的通知

---

#### 结构体

##### `AggregatedNotification`

聚合的单条通知数据。

```rust
pub struct AggregatedNotification {
    status: String,
    duration: u64,
    cmd: String,
    timestamp: u64,
}
```

---

##### `AggregatedResult`

聚合通知结果。

```rust
pub struct AggregatedResult {
    pub total: usize,
    pub success: usize,
    pub error: usize,
    pub notifications: Vec<AggregatedNotification>,
}
```

**方法：**

###### `title(&self) -> String`

生成聚合通知的标题。

**示例：**

```rust
let title = result.title();
// "5 个任务完成 (3 成功, 2 失败)" 或 "3 个任务已完成"
```

###### `message(&self) -> String`

生成聚合通知的消息内容。

**示例：**

```rust
let msg = result.message();
// 包含成功/失败统计和最近任务列表
```

###### `status(&self) -> &str`

获取聚合通知的状态。

**返回值：**

- `"error"` - 如果有错误
- `"success"` - 如果全部成功

---

#### 函数

##### `get_state_file_path() -> PathBuf`

获取聚合状态文件的路径。

**返回值：**

- `PathBuf` - 状态文件路径

**平台特定路径：**

- Windows: `%APPDATA%\claude-code-notify\aggregation.json`
- macOS: `~/Library/Application Support/claude-code-notify/aggregation.json`
- Linux: `~/.config/claude-code-notify/aggregation.json`

---

### 自动集成 (integration)

自动集成模块处理与 Claude Code 的自动集成，包括配置文件侦测、备份和 hooks 注入。

#### 结构体

##### `IntegrationManager`

集成管理器。

```rust
pub struct IntegrationManager;
```

---

#### 方法

##### `new() -> IntegrationManager`

创建集成管理器实例。

**返回值：**

- `IntegrationManager` - 管理器实例

**示例：**

```rust
use ccn::integration::IntegrationManager;

let manager = IntegrationManager::new();
```

---

##### `detect_config_path(&self) -> Option<PathBuf>`

侦测 Claude Code 配置文件路径。

**返回值：**

- `Option<PathBuf>` - 找到配置文件返回 Some(path)，否则返回 None

**查找顺序：**

1. 环境变量 `CLAUDE_CONFIG_DIR` 指定的路径
2. 默认路径 `~/.claude/settings.json`

**示例：**

```rust
match manager.detect_config_path() {
    Some(path) => println!("配置文件: {:?}", path),
    None => println!("未找到配置文件"),
}
```

---

##### `backup_config(&self, config_path: &PathBuf) -> Result<PathBuf>`

备份配置文件。

**参数：**

- `config_path: &PathBuf` - 配置文件路径

**返回值：**

- `Result<PathBuf>` - 成功返回备份文件路径

**备份文件名格式：**

- `settings.json.bak.YYYYMMDD_HHMMSS`

**示例：**

```rust
let backup_path = manager.backup_config(&config_path)?;
println!("备份已创建: {:?}", backup_path);
```

---

##### `inject_hooks(&self, config_path: &PathBuf) -> Result<()>`

向配置文件注入 hooks 配置。

**参数：**

- `config_path: &PathBuf` - 配置文件路径

**注入的 Hook：**

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "permission_prompt",
        "hooks": [{
          "type": "command",
          "command": "ccn notify --status=pending --cmd='Claude Code 需要授权'"
        }]
      }
    ]
  }
}
```

**错误处理：**

- 配置文件格式错误时自动恢复备份

**示例：**

```rust
manager.inject_hooks(&config_path)?;
```

---

##### `remove_hooks(&self, config_path: &PathBuf) -> Result<()>`

从配置文件移除 hooks 配置。

**参数：**

- `config_path: &PathBuf` - 配置文件路径

**示例：**

```rust
manager.remove_hooks(&config_path)?;
```

---

##### `is_integrated(&self, config_path: &PathBuf) -> Result<bool>`

检查是否已集成 CCN。

**参数：**

- `config_path: &PathBuf` - 配置文件路径

**返回值：**

- `Result<bool>` - true 表示已集成

**示例：**

```rust
if manager.is_integrated(&config_path)? {
    println!("CCN 已集成");
}
```

---

##### `send_test_notification(&self) -> Result<()>`

发送测试通知验证配置。

**返回值：**

- `Result<()>` - 成功返回 Ok(())

**示例：**

```rust
manager.send_test_notification()?;
```

---

##### `verify_integration(&self) -> Result<VerificationResult>`

验证集成状态。

**返回值：**

- `Result<VerificationResult>` - 验证结果

**验证项目：**

1. ccn 命令是否在 PATH 中
2. 测试通知是否可以发送

**示例：**

```rust
let result = manager.verify_integration()?;
if result.ccn_in_path && result.test_notification_sent {
    println!("集成正常");
}
```

---

#### 结构体

##### `VerificationResult`

集成验证结果。

```rust
pub struct VerificationResult {
    pub ccn_in_path: bool,
    pub test_notification_sent: bool,
    pub error: Option<String>,
}
```

**字段说明：**

- `ccn_in_path: bool` - ccn 命令是否在 PATH 中
- `test_notification_sent: bool` - 测试通知是否成功发送
- `error: Option<String>` - 错误信息（如果有）

---

### 配置向导 (wizard)

配置向导模块提供交互式终端界面，引导用户完成配置设置。

#### 结构体

##### `ConfigWizard`

配置向导。

```rust
pub struct ConfigWizard;
```

---

#### 方法

##### `new() -> ConfigWizard`

创建配置向导实例。

**返回值：**

- `ConfigWizard` - 向导实例

**示例：**

```rust
use ccn::wizard::ConfigWizard;

let wizard = ConfigWizard::new();
```

---

##### `run(&self) -> Result<Config>`

运行配置向导。

**返回值：**

- `Result<Config>` - 返回配置的 Config 对象

**配置流程：**

1. 是否启用通知声音
2. 专注助手模式（respect/always/never）
3. 最小通知阈值（秒）
4. 白名单命令
5. 通知聚合设置
6. 日志级别
7. 预览并确认

**示例：**

```rust
let config = wizard.run()?;
println!("配置完成: {:?}", config);
```

---

### CLI 接口 (cli)

CLI 模块处理命令行参数解析和子命令调度。

#### 函数

##### `run() -> Result<()>`

运行 CLI 命令。

**返回值：**

- `Result<()>` - 成功返回 Ok(())，失败返回错误

**支持的命令：**

- `notify` - 发送通知
- `init` - 启动配置向导
- `setup` - 自动集成
- `uninstall` - 卸载集成
- `verify` - 验证集成
- `config` - 显示当前配置
- `test` - 发送测试通知

**示例：**

```rust
use ccn::cli::run;

run()?;
```

---

### 平台检测 (platform)

平台信息模块提供操作系统检测功能。

#### 常量

```rust
pub const OS_WINDOWS: bool = cfg!(windows);
pub const OS_MACOS: bool = cfg!(target_os = "macos");
pub const OS_LINUX: bool = cfg!(target_os = "linux");
pub const OS_UNKNOWN: bool = !(OS_WINDOWS || OS_MACOS || OS_LINUX);
```

---

#### 函数

##### `platform_name() -> &'static str`

获取平台名称。

**返回值：**

- `&str` - 平台名称（"Windows"、"macOS"、"Linux" 或 "Unknown"）

**示例：**

```rust
use ccn::platform::platform_name;

println!("当前平台: {}", platform_name());
```

---

##### `supports_native_notifications() -> bool`

检查平台是否支持原生通知。

**返回值：**

- `bool` - Windows 和 macOS 返回 true

**示例：**

```rust
use ccn::platform::supports_native_notifications;

if supports_native_notifications() {
    println!("支持原生通知");
}
```

---

## 使用示例

### 基本用法

```rust
use ccn::{
    config::load_config,
    notification::{get_notification_manager, NotificationStatus},
    policy::PolicyEngine,
};

// 加载配置
let config = load_config()?;

// 创建策略引擎
let engine = PolicyEngine::new(config.clone());

// 检查是否应该通知
if engine.should_notify(NotificationStatus::Success, 15, "npm test") {
    // 发送通知
    let notifier = get_notification_manager();
    notifier.send_notification(
        NotificationStatus::Success,
        "任务完成",
        "npm test 执行成功",
        5000,
    )?;
}
```

### 聚合通知

```rust
use ccn::aggregator::NotificationAggregator;

let aggregator = NotificationAggregator::new(
    state_path,
    5000,  // 5秒窗口
    3,     // 最多3条
);

// 添加通知
if let Some(result) = aggregator.add_notification("success", 15, "npm test")? {
    // 发送聚合通知
    println!("{} 个任务完成", result.total);
}
```

### 集成管理

```rust
use ccn::integration::IntegrationManager;

let manager = IntegrationManager::new();

// 侦测配置文件
if let Some(path) = manager.detect_config_path() {
    // 备份
    manager.backup_config(&path)?;

    // 注入 hooks
    manager.inject_hooks(&path)?;

    // 发送测试通知
    manager.send_test_notification()?;
}
```

---

## 错误处理

所有 API 函数返回 `Result<T>` 类型，错误类型为 `anyhow::Error`。

```rust
use ccn::config::load_config;

match load_config() {
    Ok(config) => {
        // 使用配置
        println!("配置版本: {}", config.version);
    }
    Err(e) => {
        eprintln!("错误: {}", e);
        // 处理错误
    }
}
```

---

## 配套工具

### sound 模块

声音播放模块，支持系统音效和自定义音频文件。

```rust
use ccn::sound::{SoundPlayer, SystemSound};

let player = SoundPlayer::new(true);
player.play_system_sound(SystemSound::Success)?;
player.play_sound_file("custom.wav")?;
```

### path_manager 模块 (Windows)

Windows PATH 管理模块，用于添加和删除环境变量路径。

```rust
#[cfg(windows)]
use ccn::path_manager::PathManager;

let ccn_dir = std::path::PathBuf::from("C:\\ccn");
PathManager::add_to_path(&ccn_dir)?;
```

---

## 版本信息

本文档对应 CCN 版本：`env!("CARGO_PKG_VERSION")`

文档更新日期：2025-01-04
