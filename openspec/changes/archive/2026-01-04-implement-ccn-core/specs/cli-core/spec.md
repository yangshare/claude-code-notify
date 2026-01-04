## 新增需求

### 需求：CLI 命令框架
系统必须提供命令行工具 `ccn.exe`，支持以下子命令：
- `notify`：发送通知
- `init`：启动交互式配置向导
- `setup`：自动集成到 Claude Code
- `version`：显示版本信息
- `help`：显示帮助信息

#### 场景：用户执行 notify 命令
- **当** 用户执行 `ccn notify --status=success --duration=15 --cmd="npm test"`
- **那么** 系统应解析参数并发送成功通知，显示耗时 15 秒和命令信息

#### 场景：用户执行 init 命令
- **当** 用户执行 `ccn init`
- **那么** 系统应启动交互式配置向导（TUI）

#### 场景：用户执行 setup 命令
- **当** 用户执行 `ccn setup`
- **那么** 系统应自动侦测并配置 Claude Code hooks

### 需求：进程生命周期管理
CLI 工具必须是无界面、无常驻进程的设计，启动后执行任务立即退出。

#### 场景：命令执行完成后退出
- **当** `ccn notify` 命令完成通知发送
- **那么** 进程必须在 100ms 内退出

#### 场景：工具崩溃不影响 Claude Code
- **当** ccn.exe 发生内部错误
- **那么** 必须捕获异常并优雅退出，不得影响调用方进程

### 需求：命令行参数解析
系统必须支持以下参数格式：
- 短选项：`-s`, `-d`, `-c`
- 长选项：`--status`, `--duration`, `--cmd`
- 等号赋值：`--status=success`
- 空格赋值：`--status success`

#### 场景：解析短选项参数
- **当** 用户执行 `ccn notify -s error -d 5 -c "make build"`
- **那么** 系统应正确解析 status=error, duration=5, cmd="make build"

#### 场景：解析长选项参数
- **当** 用户执行 `ccn notify --status=success --duration=10 --cmd="npm install"`
- **那么** 系统应正确解析所有参数

### 需求：错误处理和日志
系统必须提供完善的错误处理和可配置的日志记录。

#### 场景：参数缺失时显示错误
- **当** 用户执行 `ccn notify`（缺少必需参数）
- **那么** 系统应显示友好的错误信息和使用示例

#### 场景：记录调试日志
- **当** 配置文件设置 `logging.level=debug`
- **那么** 系统应输出详细的调试信息到 stderr 或日志文件

### 需求：版本信息
系统必须提供版本信息命令。

#### 场景：查询版本
- **当** 用户执行 `ccn version` 或 `ccn --version`
- **那么** 系统应输出版本号、构建日期和 Git commit hash
