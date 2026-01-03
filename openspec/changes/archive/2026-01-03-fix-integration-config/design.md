# 架构设计文档

## 上下文

**问题描述**：

当前 CCN 的 Claude Code 集成功能存在两个关键问题：

1. **配置文件路径不符合官方规范**：
   - 当前检测 `config.json`，官方文件名为 `settings.json`
   - Windows 平台优先检测 `%APPDATA%\Claude\`，但实际应为 `~/.claude/`
   - 导致使用 VS Code 插件方式安装 Claude Code 的用户无法被侦测到

2. **Windows 环境变量未配置**：
   - `ccn setup` 注入 hooks 时使用命令 `ccn notify ...`
   - 但 `ccn.exe` 不在系统 PATH 中，导致 hooks 执行失败
   - 用户手动添加 PATH 后才能正常使用

**影响范围**：
- 所有 Windows 用户（尤其是 VS Code 插件用户）
- 自动集成功能完全失效

## 目标 / 非目标

### 目标
- 修正配置文件路径检测，符合 Claude Code 官方规范
- 自动将 ccn.exe 添加到 Windows PATH，确保 hooks 可执行
- 支持用户通过环境变量自定义配置目录
- 提供友好的错误提示和故障排查指导

### 非目标
- 不修改 hooks 的命令格式（保持 `ccn notify ...`）
- 不实现复杂的 PATH 管理（仅添加/删除条目）
- 不修改配置文件结构（保持现有 settings.json 格式）

## 决策

### 决策 1：统一配置文件路径

**选择**：所有平台统一使用 `~/.claude/settings.json`

**理由**：
- 符合 Claude Code 官方文档规范
- VS Code 插件和 CLI 安装方式使用相同路径
- 简化代码逻辑，减少平台差异处理

**考虑的替代方案**：
- **多路径检测**：同时检测多个可能的位置
  - 问题：增加复杂度，可能检测到错误的配置文件
- **用户手动指定**：让用户提供配置文件路径
  - 问题：降低用户体验，违背"零配置"目标

### 决策 2：环境变量支持

**选择**：支持 `CLAUDE_CONFIG_DIR` 环境变量

**理由**：
- Claude Code 官方支持此环境变量
- 为高级用户提供自定义能力
- 向后兼容，不影响默认行为

**实现方式**：
```rust
fn get_claude_config_dir() -> PathBuf {
    std::env::var("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|p| p.join(".claude"))
                .expect("无法确定主目录")
        })
}
```

### 决策 3：PATH 修改策略

**选择**：修改用户级别的 PATH 环境变量（非系统级别）

**理由**：
- 不需要管理员权限
- 不影响其他用户
- 更安全，降低风险

**实现方式**：
- Windows：修改注册表 `HKEY_CURRENT_USER\Environment\Path`
- 添加前验证路径是否已存在
- 提供回滚功能

**考虑的替代方案**：
- **修改系统级别 PATH**：需要管理员权限，不推荐
- **使用完整路径注入 hooks**：可行但不够优雅，用户无法直接在终端使用 `ccn`

### 决策 4：hooks 命令格式

**选择**：保持现有格式 `ccn notify ...`，不使用完整路径

**理由**：
- 简洁易读
- 用户可以直接在终端使用相同命令
- 通过修改 PATH 确保可执行性

**考虑的替代方案**：
- **使用完整路径**：如 `"C:\\Users\\...\\ccn.exe" notify ...`
  - 问题：路径因用户而异，不够优雅；用户无法直接复用命令

## 系统架构

### 当前架构问题

```
┌─────────────────────────────────────────────────────────────┐
│                   Claude Code (VS Code 插件)                │
│                  配置路径: C:\Users\xxx\.claude\            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              IntegrationManager::detect_config_path         │
│  ❌ 检测 config.json (应为 settings.json)                  │
│  ❌ Windows 优先检测 %APPDATA%\Claude\ (应为 ~/.claude/)   │
│  ❌ 导致 VS Code 插件用户无法被侦测                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    ccn setup 注入 hooks                     │
│  ✅ 成功注入到配置文件                                      │
│  ❌ 但 ccn.exe 不在 PATH 中                                │
│  ❌ 导致 hooks 执行失败                                     │
└─────────────────────────────────────────────────────────────┘
```

### 修复后架构

```
┌─────────────────────────────────────────────────────────────┐
│              Claude Code (CLI 或 VS Code 插件)              │
│           统一配置路径: ~/.claude/settings.json             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              IntegrationManager::detect_config_path         │
│  ✅ 检测 settings.json                                     │
│  ✅ 统一使用 ~/.claude/ 路径                               │
│  ✅ 支持 CLAUDE_CONFIG_DIR 环境变量覆盖                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              ccn setup 完整集成流程                          │
│  1. 侦测配置文件路径 ✅                                     │
│  2. 将 ccn.exe 添加到 PATH ✅                              │
│  3. 注入 hooks 到 settings.json ✅                         │
│  4. 发送测试通知验证 ✅                                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Claude Code hooks 执行                    │
│  PostCommand: ccn notify --status=success ...               │
│  ✅ ccn 在 PATH 中，正常执行 ✅                             │
└─────────────────────────────────────────────────────────────┘
```

## 模块变更

### 1. IntegrationManager

**变更内容**：

```rust
// 修改前
impl IntegrationManager {
    pub fn detect_config_path(&self) -> Option<PathBuf> {
        #[cfg(windows)]
        let paths = vec![
            std::env::var("APPDATA")
                .ok()
                .map(|p| PathBuf::from(p).join("Claude").join("config.json")),  // ❌ 错误
            std::env::var("USERPROFILE")
                .ok()
                .map(|p| PathBuf::from(p).join(".claude").join("config.json")), // ❌ 文件名错误
        ];
        // ...
    }
}

// 修改后
impl IntegrationManager {
    pub fn detect_config_path(&self) -> Option<PathBuf> {
        let config_dir = Self::get_config_dir();  // ✅ 统一处理
        let settings_file = config_dir.join("settings.json");  // ✅ 正确文件名

        if settings_file.exists() {
            Some(settings_file)
        } else {
            None
        }
    }

    fn get_config_dir() -> PathBuf {
        // ✅ 支持 CLAUDE_CONFIG_DIR 环境变量
        std::env::var("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|p| p.join(".claude"))
                    .expect("无法确定主目录")
            })
    }
}
```

### 2. 新增 PathManager 模块

**职责**：管理 Windows PATH 环境变量

```rust
pub struct PathManager;

impl PathManager {
    /// 将指定目录添加到用户 PATH
    pub fn add_to_path(directory: &PathBuf) -> Result<()> {
        #[cfg(windows)]
        {
            // 1. 获取当前用户 PATH
            let current_path = Self::get_user_path()?;

            // 2. 检查是否已存在
            if Self::contains_path(&current_path, directory) {
                log::info!("PATH 已包含: {:?}", directory);
                return Ok(());
            }

            // 3. 添加到 PATH
            let new_path = Self::append_to_path(&current_path, directory);

            // 4. 更新注册表
            Self::set_user_path(&new_path)?;

            log::info!("已添加到 PATH: {:?}", directory);
            Ok(())
        }

        #[cfg(not(windows))]
        {
            // Unix 系统通常通过包管理器安装，已在 PATH 中
            Ok(())
        }
    }

    /// 从用户 PATH 中移除指定目录
    pub fn remove_from_path(directory: &PathBuf) -> Result<()> {
        #[cfg(windows)]
        {
            let current_path = Self::get_user_path()?;
            let new_path = Self::remove_from_path_str(&current_path, directory);
            Self::set_user_path(&new_path)?;

            log::info!("已从 PATH 移除: {:?}", directory);
            Ok(())
        }

        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    #[cfg(windows)]
    fn get_user_path() -> Result<String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        let environment = hklm.open_subkey_with_flags("Environment", KEY_READ)?;
        let path: String = environment.get_value("Path")?;
        Ok(path)
    }

    #[cfg(windows)]
    fn set_user_path(path: &str) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        let environment = hklm.open_subkey_with_flags("Environment", KEY_WRITE)?;
        environment.set_value("Path", &path)?;

        // 通知系统环境变量已更改
        unsafe {
            winapi::um::winuser::SendMessageTimeoutW(
                winapi::um::winuser::HWND_BROADCAST,
                winapi::um::winuser::WM_SETTINGCHANGE,
                0 as usize,
                "Environment".as_ptr() as isize,
                0,
                5000,
                std::ptr::null_mut(),
            );
        }

        Ok(())
    }
}
```

### 3. Setup/Uninstall 命令集成

```rust
// ccn setup 命令
pub fn execute_setup() -> Result<()> {
    let integration = IntegrationManager::new();

    // 1. 侦测配置文件
    let config_path = integration.detect_config_path()
        .ok_or_else(|| anyhow!("无法侦测 Claude Code 配置文件"))?;

    // 2. 添加到 PATH
    let ccn_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow!("无法获取可执行文件目录"))?
        .to_path_buf();

    PathManager::add_to_path(&ccn_dir)?;

    // 3. 备份配置
    integration.backup_config(&config_path)?;

    // 4. 注入 hooks
    integration.inject_hooks(&config_path)?;

    // 5. 发送测试通知
    integration.send_test_notification()?;

    println!("✅ CCN 已成功集成到 Claude Code");
    println!("⚠️  请重启终端或 VS Code 以使 PATH 生效");

    Ok(())
}

// ccn uninstall 命令
pub fn execute_uninstall() -> Result<()> {
    let integration = IntegrationManager::new();

    // 1. 侦测配置文件
    let config_path = integration.detect_config_path()
        .ok_or_else(|| anyhow!("无法侦测 Claude Code 配置文件"))?;

    // 2. 移除 hooks
    integration.remove_hooks(&config_path)?;

    // 3. 从 PATH 移除
    let ccn_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow!("无法获取可执行文件目录"))?
        .to_path_buf();

    PathManager::remove_from_path(&ccn_dir)?;

    println!("✅ CCN 已从 Claude Code 卸载");
    println!("⚠️  请重启终端以使 PATH 更新生效");

    Ok(())
}
```

## 数据流

### 修复后的集成流程

```
用户执行: ccn setup
    ↓
1. IntegrationManager::detect_config_path()
   ├─ 检查 CLAUDE_CONFIG_DIR 环境变量
   ├─ 使用默认 ~/.claude/ 路径
   └─ 验证 settings.json 是否存在
    ↓
2. PathManager::add_to_path(ccn_dir)
   ├─ 获取当前用户 PATH
   ├─ 检查 ccn_dir 是否已存在
   ├─ 添加到 PATH
   └─ 更新注册表并通知系统
    ↓
3. IntegrationManager::backup_config()
   └─ 备份 settings.json
    ↓
4. IntegrationManager::inject_hooks()
   ├─ 读取 settings.json
   ├─ 添加 PostCommand 和 CommandError hooks
   └─ 写回 settings.json
    ↓
5. IntegrationManager::send_test_notification()
   └─ 验证集成是否成功
    ↓
6. 用户重启终端/VS Code
    ↓
7. Claude Code 执行 hooks
   └─ ccn notify ... (在 PATH 中，正常执行)
```

## 风险 / 权衡

### 风险 1：修改 PATH 环境变量
**风险**：可能影响系统稳定性或与其他软件冲突
**缓解措施**：
- 仅修改用户级别 PATH，不涉及系统级别
- 添加前检查是否已存在，避免重复
- 提供完整的卸载和回滚功能
- 详细的错误提示和日志记录

### 风险 2：注册表操作失败
**风险**：权限不足或注册表损坏
**缓解措施**：
- 使用用户级别注册表，不需要管理员权限
- 添加详细的错误处理和恢复机制
- 提供手动修改 PATH 的指导文档

### 风险 3：配置文件格式变化
**风险**：Claude Code 未来可能修改 settings.json 格式
**缓解措施**：
- 使用 serde_json 进行解析，支持容错
- 仅添加 hooks 字段，不修改现有内容
- 注入前验证配置文件完整性

### 权衡 1：PATH 修改 vs 完整路径 hooks
**决策**：修改 PATH + 保持 hooks 简洁
**理由**：
- 更符合用户期望
- 用户可以在终端直接使用 `ccn` 命令
- hooks 命令更简洁易读

### 权衡 2：自动修改 vs 用户确认
**决策**：自动修改，但在卸载时提供回滚
**理由**：
- 符合"零配置"目标
- PATH 修改是低风险操作
- 用户可通过 uninstall 撤销

## 测试计划

### 单元测试
1. **配置路径检测**
   - 测试默认路径（~/.claude/settings.json）
   - 测试 CLAUDE_CONFIG_DIR 环境变量覆盖
   - 测试配置文件不存在的情况

2. **PATH 管理**
   - 测试添加到 PATH
   - 测试重复添加检测
   - 测试从 PATH 移除

### 集成测试
1. **setup 命令**
   - 完整流程测试
   - 配置文件验证
   - PATH 验证

2. **uninstall 命令**
   - hooks 移除验证
   - PATH 清理验证

3. **hooks 执行**
   - 在 Claude Code 中触发 hooks
   - 验证通知是否正常显示

### 手动测试场景
1. **VS Code 插件用户**
   - 首次安装
   - 配置路径：C:\Users\xxx\.claude\settings.json
   - 验证集成是否成功

2. **CLI 安装用户**
   - 配置路径相同
   - 验证集成是否成功

3. **自定义 CLAUDE_CONFIG_DIR**
   - 设置环境变量
   - 验证是否能正确侦测

## 迁移计划

### 向后兼容性
- **现有用户**：无需特殊处理，重新运行 `ccn setup` 即可
- **受影响功能**：仅集成功能，不影响通知显示

### 部署步骤
1. 发布新版本 CCN
2. 更新 README 和文档
3. 通知现有用户重新运行 `ccn setup`

### 回滚策略
- `ccn uninstall` 完整移除所有更改
- 配置文件自动备份
- PATH 恢复到修改前状态

## 待决问题

1. **依赖库选择**：PATH 管理是否使用专门的库？
   - 建议：直接使用 winreg，避免引入额外依赖

2. **PATH 修改通知**：是否需要立即通知所有应用程序？
   - 建议：使用 WM_SETTINGCHANGE 消息，但建议用户重启终端

3. **错误恢复**：如果 setup 中途失败，如何处理？
   - 建议：事务式处理，失败时回滚所有更改

4. **多版本共存**：如果用户安装了多个版本的 ccn？
   - 建议：PATH 中仅保留一个版本，setup 时覆盖旧版本路径
