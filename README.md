# Claude Code Notify (CCN)

> 为 Claude Code 打造的优雅任务通知工具

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

CCN 是一个轻量级的命令行工具，通过 Windows 原生通知系统为你的 Claude Code 任务提供实时反馈。当你执行长时间运行的任务（如构建、测试、部署）时，CCN 会在任务完成时自动通知你，无需频繁切换窗口查看进度。

## 特性

- **原生通知体验** - 使用 Windows 11 Toast 通知，状态感知图标（✅❌⏳）
- **智能通知策略** - 可配置的阈值过滤，避免短时间任务打扰
- **通知聚合** - 自动合并短时间内的多条通知
- **零配置自动集成** - 一条命令完成所有设置
- **交互式配置向导** - 简单的问答式配置界面
- **轻量级** - 无常驻进程，按需启动，秒级响应

## 安装

### 前置要求

- **Windows 11** 或 macOS
- [Claude Code](https://claude.ai/code)

### 方式 1: 下载预编译版本（推荐）

从 [Releases 页面](https://github.com/yangshare/claude-code-notify/releases) 下载适合你平台的二进制文件：

- **Windows x64**: `ccn-windows-x86_64.exe`
- **Windows ARM64**: `ccn-windows-aarch64.exe`
- **macOS Intel**: `ccn-macos-x86_64`
- **macOS Apple Silicon**: `ccn-macos-aarch64`

下载后：
1. 重命名为 `ccn.exe` (Windows) 或 `ccn` (macOS)
2. 添加到系统 PATH

### 方式 2: 使用 Scoop (Windows)

```powershell
scoop bucket add ccn https://github.com/yangshare/claude-code-notify
scoop install ccn
```

### 方式 3: 从源码构建

需要 [Rust 工具链](https://rustup.rs/)：

```bash
# 克隆仓库
git clone https://github.com/yangshare/claude-code-notify.git
cd claude-code-notify

# 构建
cargo build --release

# 将可执行文件添加到 PATH
# Windows:
copy target\release\ccn.exe C:\Windows\System32\
# 或添加到用户 PATH
```

## 快速开始

### 1. 自动集成（推荐）

一条命令完成所有配置：

```bash
ccn setup
```

这个命令会：
- 自动侦测 Claude Code 配置文件位置
- 将 ccn.exe 添加到系统 PATH（Windows）
- 创建配置文件备份
- 注入必要的 hooks 配置
- 发送测试通知验证安装

> **⚠️ Windows 用户注意**：运行 `ccn setup` 后，**必须重启您的终端或 VS Code**，才能使 PATH 环境变量生效。重启后，hooks 将自动生效。

### 2. 配置通知规则

运行交互式配置向导：

```bash
ccn init
```

配置向导会引导你设置：
- 是否启用通知声音
- 专注助手模式（尊重/始终/从不）
- 最小通知阈值（默认 10 秒）
- 白名单命令（始终通知的命令）
- 通知聚合设置
- 日志级别

### 3. 测试通知

```bash
ccn test
```

### 4. 验证集成

```bash
ccn verify
```

验证 CCN 是否正确集成，包括：
- 检查 ccn 命令是否在 PATH 中
- 测试通知功能是否正常工作

### 5. 查看当前配置

```bash
ccn config
```

## 使用方法

### 手动发送通知

```bash
# 成功通知
ccn notify --status success --duration 15 --cmd "npm run build"

# 失败通知
ccn notify --status error --duration 5 --cmd "npm test"

# 等待状态
ccn notify --status pending --duration 0 --cmd "deploying..."
```

### 卸载集成

```bash
ccn uninstall
```

这会：
- 从 Claude Code 配置中移除 CCN 的 hooks
- 从系统 PATH 中移除 ccn.exe（Windows）
- 保留配置文件备份

> **⚠️ Windows 用户注意**：运行 `ccn uninstall` 后，**建议重启终端**以使 PATH 更新生效。

## 配置文件

配置文件位于：

- **Windows**: `%APPDATA%\claude-code-notify\config.yaml`
- **macOS**: `~/Library/Application Support/claude-code-notify/config.yaml`

### 配置示例

```yaml
# 全局设置
version: "1.0"
sound_enabled: true
focus_assistant_mode: respect  # respect, always, never

# 通知阈值
threshold:
  min_duration: 10  # 秒，低于此值不通知（错误除外）
  whitelist:
    - deploy
    - release

# 场景化模板
templates:
  default:
    icon: auto
    sound: default
    duration: 5000  # 毫秒

  build:
    icon: icons/build.png
    sound: sounds/build_success.wav
    duration: 8000

# 通知聚合
aggregation:
  enabled: true
  window: 5000  # 毫秒，聚合时间窗口
  max_toasts: 3  # 最多聚合多少条

# 日志设置
logging:
  level: info  # debug, info, warn, error
  file: ""  # 空表示仅输出到 stderr
```

## 故障排查

### 问题：运行 `ccn setup` 后找不到 Claude Code 配置文件

**症状**：
```
❌ 未找到 Claude Code 配置文件
```

**解决方案**：

1. **确认 Claude Code 已安装**
   - 如果使用 VS Code 插件，确保已安装并运行过 Claude Code
   - 如果使用 CLI 版本，确保已正确安装

2. **检查默认配置路径**
   - 默认路径为：`~/.claude/settings.json`
   - Windows：`C:\Users\<用户名>\.claude\settings.json`
   - macOS/Linux：`~/.claude/settings.json`

3. **使用自定义配置路径**
   - 如果 Claude Code 配置在其他位置，设置环境变量：
     ```bash
     # Windows PowerShell
     $env:CLAUDE_CONFIG_DIR="D:\custom\path"
     ccn setup

     # Windows CMD
     set CLAUDE_CONFIG_DIR=D:\custom\path
     ccn setup

     # Linux/macOS
     export CLAUDE_CONFIG_DIR=/custom/path
     ccn setup
     ```

### 问题：hooks 不工作，收不到通知

**症状**：
- 任务完成后没有收到通知
- 在 Claude Code 中执行命令无响应

**解决方案**：

1. **验证 PATH 配置（Windows 用户）**
   ```bash
   # 运行验证命令
   ccn verify
   ```
   - 如果提示 "ccn 命令不在 PATH 中"，说明 PATH 未生效
   - **重启您的终端或 VS Code** 使 PATH 生效
   - 重启后再次运行 `ccn verify` 验证

2. **检查 hooks 是否正确注入**
   - 查看 `~/.claude/settings.json`
   - 确认 `hooks` 字段存在且包含 `PostCommand` 和 `CommandError`

3. **手动测试通知**
   ```bash
   ccn test
   ```
   - 如果测试通知正常显示，说明通知系统工作正常
   - 问题可能出在 hooks 配置或 Claude Code 集成

4. **检查日志**
   - 设置环境变量启用详细日志：
     ```bash
     # Windows
     set RUST_LOG=debug
     ccn notify --status=success --duration=1 --cmd=test

     # Linux/macOS
     RUST_LOG=debug ccn notify --status=success --duration=1 --cmd=test
     ```

### 问题：Windows 上无法修改 PATH

**症状**：
```
⚠ 无法修改 PATH: ...
```

**解决方案**：

1. **手动添加到 PATH**
   - 找到 ccn.exe 所在目录
   - 添加到系统 PATH：
     - 按 `Win + X`，选择"系统"
     - 点击"高级系统设置"
     - 点击"环境变量"
     - 在"用户变量"中找到 `Path`
     - 点击"编辑"，添加 ccn.exe 所在目录
   - 重启终端

2. **检查权限**
   - 确保当前用户有权限修改用户环境变量
   - 不需要管理员权限（修改的是用户级别 PATH）

### 问题：通知不显示

**症状**：
- 执行命令后看不到通知
- `ccn test` 没有反应

**解决方案**：

1. **检查通知权限（Windows 11）**
   - 设置 > 系统 > 通知和操作
   - 确保通知已启用
   - 确允许应用发送通知

2. **检查 AUMID 和快捷方式**
   - CCN 会自动创建 AUMID: `ClaudeCodeNotify.CCN`
   - 如果通知仍然不显示，尝试：
     ```bash
     ccn test
     ```
   - 如果首次运行，系统可能会提示是否允许通知

3. **检查专注助手设置**
   - 配置文件中的 `focus_assistant_mode` 设置
   - 如果设置为 `respect`，在专注模式下可能不显示通知
   - 修改为 `always` 强制显示所有通知

3. **查看 Windows 通知中心**
   - 点击任务栏右下角的通知图标
   - 检查通知是否在历史记录中

### 问题：配置文件格式错误

**症状**：
```
配置文件 JSON 格式错误
```

**解决方案**：

1. **恢复备份**
   - `ccn setup` 会自动创建备份
   - 备份文件位于：`settings.json.bak.<时间戳>`
   - 删除当前配置文件，重命名备份文件

2. **重新运行 setup**
   ```bash
   ccn uninstall
   ccn setup
   ```

## 功能详解

### 智能阈值过滤

默认情况下，执行时间低于 10 秒的任务不会发送通知，避免频繁打扰。

**例外情况**：
- 错误状态的任务**始终**发送通知，无论耗时多久
- 在白名单中的命令（如 `deploy`）始终发送通知

### 通知聚合

在短时间内（默认 5 秒）连续触发的多条通知会自动合并为一条聚合通知，例如：

```
3 个任务完成 (2 成功, 1 失败)

最近的任务:
  ✅ npm test (15秒)
  ✅ cargo build (20秒)
  ❌ npm run deploy (5秒)
```

### 状态感知图标

- ✅ **成功** - 任务成功完成
- ❌ **失败** - 任务执行失败
- ⏳ **等待** - 任务正在进行中

## 开发

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_aggregation_stats
```

### 项目结构

```
src/
├── main.rs          # 程序入口
├── cli.rs           # CLI 命令处理
├── config.rs        # 配置管理
├── notification.rs  # 通知平台抽象
├── policy.rs        # 智能策略引擎
├── integration.rs   # 自动集成管理
├── wizard.rs        # 配置向导
└── aggregator.rs    # 通知聚合器
```

## 技术栈

- **Rust** - 高性能、内存安全
- **clap** - CLI 参数解析
- **serde_yaml** - YAML 配置解析
- **windows** - 微软官方 Windows API 绑定（WinRT Toast 通知）

## 路线图

### 已实现 ✅

- [x] CLI 核心功能
- [x] Windows 原生通知
- [x] 智能阈值过滤
- [x] 通知聚合
- [x] 自动集成
- [x] 配置向导
- [x] 白名单命令
- [x] 场景化模板

### 计划中 🚧

- [ ] macOS 原生通知支持
- [ ] 通知交互按钮（查看日志、重试等）
- [ ] 配置热重载
- [ ] 单元测试覆盖率提升

## 贡献

欢迎贡献！请随时提交 Issue 或 Pull Request。

### 开发流程

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [Claude Code](https://claude.ai/code) - AI 驱动的编码助手
- [clap](https://github.com/clap-rs/clap) - Rust CLI 框架
- [win32_notif](https://github.com/ahq-softwares/win32_notif) - Windows Toast 通知库

---

**注意**: CCN 是独立的开源项目，不由 Anthropic 或 Claude Code 团队官方维护。
