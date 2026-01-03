## 上下文

CCN (Claude Code Notify) 是一个为 Claude Code 提供系统通知的工具。当前 Windows 平台使用 PowerShell + System.Windows.Forms.NotifyIcon 的临时方案，用户反馈这不是期望的实现方式。

需要实现符合 Windows 11 设计规范的原生 Toast 通知，同时保持与现有代码架构的兼容性。

**约束条件**：
- 必须保持 `NotificationManager` trait 抽象，不影响其他平台（macOS、Linux）
- 需要处理 Windows 通知的特殊要求（AUMID、快捷方式等）
- 必须有错误降级机制

## 目标 / 非目标

**目标**：
- 使用原生 Windows Runtime API 实现 Toast 通知
- 提供与现代 Windows 应用一致的视觉体验
- 支持成功/错误/进行中三种状态的通知展示
- 保持现有 API 接口不变，对上层调用者透明

**非目标**：
- 实现复杂的交互式通知（按钮、输入框等）
- 支持通知分组、历史记录等高级功能

## 决策

### 决策 1：选择 windows crate (WinRT API)

**选择**：使用微软官方 `windows` crate (v0.61) 的 WinRT API 作为 Windows Toast 通知的实现方案。

**理由**：
- 微软官方维护，API 最完整、最及时
- 直接使用 WinRT APIs，无需第三方封装
- 类型安全，完善的错误处理
- 活跃开发，文档齐全

**考虑的替代方案**：

| 方案 | 优点 | 缺点 | 决策 |
|------|------|------|------|
| `windows` crate (WinRT API) | 微软官方，API 完整，类型安全 | API 冗长，需要手动 XML 操作 | ✅ 最终选择 |
| `win32_notif` | 高层抽象，API 简单 | 通知不显示（测试验证） | ❌ 编译成功但不工作 |
| `winrt` crate | 早期 WinRT 绑定 | 模块结构已废弃，版本 0.8 无 `windows` 模块 | ❌ |
| `windows` crate (Win32 API) | 微软官方 | ToastNotificationManager API 签名不匹配 | ❌ |
| `win-toast-notify` | 另一个选择 | 文档较少，社区较小 | ❌ |
| `winrt-toast-reborn` | 功能完整 | 项目较老，维护不活跃 | ❌ |
| 继续 PowerShell | 无需依赖 | 非原生，性能差，体验不佳 | ❌ |

**实现过程**：
经过 4 次尝试：
1. `win32_notif` (0.6) - 编译成功，代码运行，但**通知不显示**
2. `windows` crate (0.61) Win32 API - API 签名不匹配
3. `winrt` crate (0.8) - 模块 `windows::data::xml::dom` 不存在
4. **`windows` crate (0.61) WinRT API** - ✅ 成功显示通知！

### 决策 2：AUMID 管理

**选择**：使用 `CreateToastNotifierWithId()` 直接创建带 AUMID 的 ToastNotifier。

**理由**：
- Windows Toast 通知要求应用有有效的 AUMID
- 使用 `ClaudeCodeNotify.CCN` 作为 AUMID
- `CreateToastNotifierWithId()` API 直接处理 AUMID 注册
- 无需手动创建开始菜单快捷方式

**实现方式**：
```rust
ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))
```

### 决策 3：错误降级机制

**选择**：当原生通知失败时，降级到控制台输出。

**理由**：
- 保持系统的可用性，即使通知功能失败也不应阻塞工作流
- 临时降级比完全失败体验更好
- 符合现有的错误处理模式（参考 PowerShell 实现中的降级逻辑）

**降级条件**：
- 初始化 WinRT 运行时失败
- 获取 XML 模板失败
- 创建通知对象失败
- 显示通知失败（但日志记录详细错误）

### 决策 4：XML 模板操作

**选择**：使用 WinRT XML DOM API 操作 Toast 模板。

**理由**：
- WinRT Toast 通知使用 XML 模板定义内容
- 必须使用 `GetTemplateContent()` 获取模板
- 使用 XML DOM API 操作节点
- 需要类型转换（`XmlText` → `IXmlNode`）

**关键 API**：
- `ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02)`
- `GetElementsByTagName(&HSTRING::from("text"))`
- `CreateTextNode(&HSTRING::from(text))`
- `.cast::<IXmlNode>()` - 类型转换
- `AppendChild(&node)` - 添加内容

## 架构

### 代码结构

```
src/notification.rs
├── NotificationManager trait (不变)
├── NotificationStatus enum (不变)
├── get_notification_manager() (不变)
└── platform 模块
    ├── [cfg(windows)] WindowsNotificationManager (重写)
    │   ├── 使用 windows::Data::Xml::Dom (XML 操作)
    │   ├── 使用 windows::UI::Notifications (Toast API)
    │   ├── 使用 windows::core::HSTRING (字符串类型)
    │   ├── 使用 windows::core::Interface (类型转换)
    │   ├── AUMID: ClaudeCodeNotify.CCN
    │   └── OnceLock 单例模式管理 ToastNotifier
    ├── [cfg(target_os = "macos")] MacOSNotificationManager (不变)
    └── [cfg(fallback)] FallbackNotificationManager (不变)
```

### 数据流

```
用户调用
  → get_notification_manager()
    → WindowsNotificationManager::send_notification()
      → get_notifier() (OnceLock 单例)
        → ToastNotificationManager::CreateToastNotifierWithId(AUMID)
      → ToastNotificationManager::GetTemplateContent(ToastText02)
        → GetElementsByTagName("text")
          → CreateTextNode(标题) → cast::<IXmlNode>() → AppendChild()
          → CreateTextNode(消息) → cast::<IXmlNode>() → AppendChild()
      → ToastNotification::CreateToastNotification(&xml)
      → notifier.Show(&toast)
        → 成功: 返回 Ok(()), 等待 2 秒
        → 失败: 降级到控制台 + 日志
```

### 关键实现细节

1. **类型转换**：`XmlText` 必须通过 `.cast::<IXmlNode>()` 转换为 `IXmlNode`
2. **Interface trait**：必须导入 `windows::core::Interface` 才能使用 `.cast()` 方法
3. **HSTRING**：WinRT 字符串类型，使用 `HSTRING::from(&str)` 转换
4. **单例模式**：`OnceLock<&'static ToastNotifier>` 确保 notifier 只创建一次
5. **等待时间**：`sleep(2秒)` 确保 Windows 有足够时间显示通知

## 风险 / 权衡

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| windows crate 版本更新 | API 变化可能导致编译失败 | 锁定明确版本 (0.61)，充分测试 |
| 通知被系统禁用 | 用户看不到通知 | 降级到控制台输出 |
| AUMID 冲突 | 通知显示异常 | 使用唯一 AUMID: ClaudeCodeNotify.CCN |
| CI/CD 环境没有 GUI | 测试失败 | 条件编译或 mock 实现 |
| XML 操作复杂 | 代码冗长，容易出错 | 封装为辅助函数，详细注释 |

## 迁移计划

### 实施步骤

1. **添加依赖**：在 Cargo.toml 中添加 `windows` crate (含 WinRT features)
2. **实现新管理器**：创建基于 WinRT API 的 WindowsNotificationManager
3. **添加 XML 操作**：实现 XML DOM 操作和类型转换
4. **替换实现**：用新实现替换 PowerShell 代码
5. **测试**：验证各种状态的通知显示
6. **文档更新**：更新 README 说明技术栈

### 实施结果

- ✅ 依赖已添加：`windows = "0.61"` with WinRT features
- ✅ WindowsNotificationManager 已重写
- ✅ XML 操作已实现（模板获取、节点操作、类型转换）
- ✅ 测试成功：用户确认通知显示在屏幕上
- ✅ 文档已更新：README.md, CHANGELOG.md, tasks.md, design.md

### 回滚计划

如果新实现出现严重问题：
- 保留 PowerShell 代码作为注释（实际未保留，因新实现已验证工作）
- 可通过 feature flag 快速切换回旧实现
- 发布新版本修复问题

### 兼容性

- **支持版本**：Windows 11
- **测试平台**：Windows 11 (已验证通知显示)

## 待决问题

1. ~~**快捷方式位置**：用户是否介意在开始菜单看到 CCN 快捷方式？~~
   - **已解决**：使用 `CreateToastNotifierWithId()` API 无需手动创建快捷方式

2. **通知持续时间**：是否需要可配置的持续时间？
   - 决定：先使用系统默认（5秒），后续可添加配置

3. **声音**：是否需要播放通知声音？
   - 决定：使用系统默认声音，可后续添加配置选项

4. **macOS 实现**：是否需要同步改进 macOS 通知？
   - 决定：不在本次变更范围内，作为未来改进项
