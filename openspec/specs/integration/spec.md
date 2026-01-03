# integration Specification

## Purpose
TBD - created by archiving change fix-integration-config. Update Purpose after archive.
## 需求
### 需求：配置文件路径检测符合官方规范
IntegrationManager 必须检测符合 Claude Code 官方规范的配置文件路径（`~/.claude/settings.json`），而不是当前错误的路径（`config.json` 和 `%APPDATA%\Claude\`）。

#### 场景：VS Code 插件用户配置文件检测
- **当** 用户使用 VS Code 插件方式安装 Claude Code，且配置文件位于 `C:\Users\xxx\.claude\settings.json`
- **且** 用户执行 `ccn setup`
- **那么** CCN 必须成功侦测到配置文件路径
- **且** 显示"找到 Claude Code 配置文件：C:\Users\xxx\.claude\settings.json"

#### 场景：CLI 安装用户配置文件检测
- **当** 用户使用 CLI 方式安装 Claude Code，且配置文件位于 `~/.claude/settings.json`
- **且** 用户执行 `ccn setup`
- **那么** CCN 必须成功侦测到配置文件路径
- **且** 配置文件路径与 VS Code 插件用户一致

#### 场景：配置文件不存在时
- **当** 用户未安装 Claude Code，且 `~/.claude/settings.json` 不存在
- **且** 用户执行 `ccn setup`
- **那么** 必须显示错误提示："无法侦测 Claude Code 配置文件"
- **且** 必须提供故障排查指导：
  - 确认 Claude Code 已安装
  - 确认配置文件路径：~/.claude/settings.json
  - 或设置 CLAUDE_CONFIG_DIR 环境变量自定义路径

#### 场景：使用 CLAUDE_CONFIG_DIR 环境变量
- **当** 用户设置了环境变量 `CLAUDE_CONFIG_DIR=D:\custom\claude`
- **且** 该目录下存在 `settings.json`
- **且** 用户执行 `ccn setup`
- **那么** CCN 必须侦测到 `D:\custom\claude\settings.json`
- **且** 显示"使用自定义配置目录：D:\custom\claude"

### 需求：hooks 注入使用正确的配置文件名
IntegrationManager 必须读写 `settings.json` 而不是 `config.json`。

#### 场景：注入 hooks 到 settings.json
- **当** Claude Code 配置文件位于 `~/.claude/settings.json`
- **且** 文件内容为：`{"permissions": {"allow": ["Bash(git *)"]}}`
- **且** 用户执行 `ccn setup`
- **那么** 配置文件必须被正确修改为包含 hooks 字段
- **且** hooks 必须包含 PostCommand 和 CommandError

#### 场景：读取现有 settings.json
- **当** `~/.claude/settings.json` 存在且包含现有配置
- **且** IntegrationManager 读取配置文件
- **那么** 必须成功解析 JSON 格式
- **且** 必须保留现有配置不变
- **且** 必须仅添加 hooks 字段

### 需求：Windows PATH 环境变量自动管理
setup 命令必须自动将 ccn.exe 所在目录添加到 Windows PATH 环境变量，uninstall 命令必须移除相关条目。

#### 场景：首次安装时添加到 PATH
- **当** Windows 10/11 系统
- **且** ccn.exe 位于 `C:\Users\xxx\AppData\Local\Programs\ccn\ccn.exe`
- **且** 用户 PATH 中不包含该目录
- **且** 用户执行 `ccn setup`
- **那么** `C:\Users\xxx\AppData\Local\Programs\ccn` 必须被添加到用户 PATH
- **且** 显示"已将 CCN 添加到系统 PATH"
- **且** 显示提示"请重启终端或 VS Code 以使 PATH 生效"

#### 场景：PATH 中已存在时跳过
- **当** 用户 PATH 中已包含 ccn.exe 所在目录
- **且** 用户执行 `ccn setup`
- **那么** 必须跳过 PATH 添加操作
- **且** 显示"PATH 已包含 CCN 目录，跳过"
- **且** 继续执行后续集成步骤

#### 场景：卸载时从 PATH 移除
- **当** 用户 PATH 中包含 ccn.exe 所在目录
- **且** settings.json 中包含 CCN hooks
- **且** 用户执行 `ccn uninstall`
- **那么** 必须从 settings.json 中移除 hooks
- **且** 必须从用户 PATH 中移除 ccn.exe 目录
- **且** 显示"CCN 已从系统 PATH 移除"

#### 场景：PATH 修改失败时
- **当** 系统权限受限或注册表损坏
- **且** 用户执行 `ccn setup`
- **那么** 必须显示错误："无法修改 PATH 环境变量"
- **且** 必须提供手动修改指导：
  - 打开系统设置 > 环境变量
  - 编辑用户变量 Path
  - 添加 ccn.exe 所在目录
- **且** 必须提供完整的目录路径
- **且** 询问是否继续注入 hooks

#### 场景：验证 PATH 配置成功
- **当** setup 命令已成功执行
- **且** 用户已重启终端
- **且** 在新终端中执行 `ccn notify --status=success`
- **那么** 命令必须成功执行
- **且** 显示 Windows 通知
- **且** 无"命令不存在"错误

### 需求：集成流程正确执行顺序
setup 和 uninstall 命令必须按照正确的顺序执行操作，确保集成和卸载的完整性。

#### 场景：setup 命令完整流程
- **当** Claude Code 已安装
- **且** ccn.exe 已放置在目标目录
- **且** 用户执行 `ccn setup`
- **那么** 必须按以下顺序执行：
  1. 侦测 Claude Code 配置文件路径
  2. 验证配置文件存在且可读写
  3. 将 ccn.exe 添加到 PATH
  4. 备份现有配置文件
  5. 注入 hooks 到 settings.json
  6. 发送测试通知
  7. 显示成功信息和后续步骤
- **且** 用户必须看到清晰的成功提示
- **且** 必须知道需要重启终端/VS Code

#### 场景：uninstall 命令完整流程
- **当** CCN 已成功集成
- **且** 用户执行 `ccn uninstall`
- **那么** 必须按以下顺序执行：
  1. 侦测 Claude Code 配置文件路径
  2. 移除 settings.json 中的 hooks
  3. 从 PATH 中移除 ccn.exe 目录
  4. 显示成功信息
- **且** 所有 CCN 相关配置必须被清理
- **且** settings.json 必须恢复到集成前状态
- **且** PATH 必须恢复到集成前状态

### 需求：友好的错误提示和故障排查
系统必须提供友好的错误提示和故障排查指导。

#### 场景：配置文件路径无法侦测
- **当** `ccn setup` 无法找到配置文件
- **那么** 必须显示错误提示和排查建议：
  - 确认 Claude Code 已安装（CLI 或 VS Code 插件）
  - 确认配置文件存在于：~/.claude/settings.json
  - 或设置 CLAUDE_CONFIG_DIR 环境变量自定义路径

#### 场景：PATH 修改成功但需要重启
- **当** `ccn setup` 成功修改 PATH
- **那么** 必须显示成功信息和重要提示：
  - 请重启终端或 VS Code 以使 PATH 环境变量生效
  - 重启后 hooks 将自动生效

#### 场景：集成验证失败
- **当** 测试通知发送失败
- **那么** 必须显示可能原因和排查建议：
  - Windows 通知服务未启用
  - ccn.exe 不在 PATH 中（请重启终端）
  - 防火墙或杀毒软件阻止
- **且** 必须提供手动测试命令

