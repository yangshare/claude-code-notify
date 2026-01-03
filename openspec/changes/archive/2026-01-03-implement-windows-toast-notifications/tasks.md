# 实施任务清单

## 1. 依赖和配置

- [x] 1.1 在 `Cargo.toml` 中添加 `windows` crate 依赖（含 WinRT features）
- [x] 1.2 设置目标平台为 `cfg(windows)` 条件编译
- [x] 1.3 验证依赖在 Windows 环境下可正常编译

## 2. AUMID 和快捷方式管理

- [x] 2.1 实现 AUMID 常量定义：`ClaudeCodeNotify.CCN`
- [x] 2.2 使用 `ToastNotificationManager::CreateToastNotifierWithId()` 创建 notifier
- [x] 2.3 使用 `OnceLock` 实现 notifier 单例模式

## 3. WindowsNotificationManager 实现

- [x] 3.1 保留 `NotificationManager` trait 和 `NotificationStatus` enum（不变）
- [x] 3.2 重写 `WindowsNotificationManager` 结构体
- [x] 3.3 实现基于 `windows` crate WinRT API 的 `send_notification()` 方法
- [x] 3.4 实现通知标题格式化（包含状态图标和文本）
- [x] 3.5 实现 `is_available()` 方法（检测 Windows 版本）
- [x] 3.6 移除旧的 PowerShell 实现代码

## 4. XML 操作和类型转换

- [x] 4.1 使用 `ToastNotificationManager::GetTemplateContent()` 获取模板
- [x] 4.2 使用 `GetElementsByTagName()` 查找文本节点
- [x] 4.3 使用 `CreateTextNode()` 创建文本内容
- [x] 4.4 使用 `.cast::<IXmlNode>()` 进行类型转换
- [x] 4.5 使用 `AppendChild()` 添加文本到模板

## 5. 错误处理和降级

- [x] 5.1 实现 Toast 初始化错误处理
- [x] 5.2 实现通知显示失败的错误处理
- [x] 5.3 实现降级到控制台输出的逻辑
- [x] 5.4 添加详细的错误日志记录

## 6. 测试和验证

- [x] 6.1 在 Windows 11 上测试成功通知
- [x] 6.2 在 Windows 11 上测试错误通知
- [x] 6.3 在 Windows 11 上测试进行中通知
- [x] 6.4 验证通知实际显示在屏幕上（用户确认）
- [x] 6.5 测试通知失败时的降级逻辑
- [x] 6.6 测试集成：`ccn test` 命令测试
- [x] 6.7 调整 sleep 时间确保通知有足够时间显示

## 7. 文档更新

- [x] 7.1 更新 README.md 技术栈说明（windows crate 替换 win32_notif）
- [x] 7.2 更新 CHANGELOG.md
- [x] 7.3 更新 tasks.md 实施说明

## 实施说明

### 实际实现方案

经过多次尝试，最终采用官方 `windows` crate (v0.61) 的 WinRT API 实现：

**尝试过的方案：**
1. `win32_notif` (0.6) - 编译成功，但通知不显示
2. `windows` crate (0.61) 错误 API - API 签名不匹配
3. `winrt` crate (0.8) - 模块结构不存在
4. **✅ `windows` crate (0.61) WinRT API** - 成功！

### 核心实现

主要代码更改：
- `Cargo.toml`：添加 `windows` crate 依赖，包含 WinRT features：
  - `Data_Xml_Dom`
  - `UI_Notifications`
  - `Win32_Foundation`
  - `Win32_UI_Notifications`
  - `Win32_UI_WindowsAndMessaging`
- `src/notification.rs`：
  - 导入 `windows::core::{HSTRING, Interface}` trait
  - 使用 `ToastNotificationManager::GetTemplateContent()` 获取 XML 模板
  - 使用 `CreateTextNode()` 和 `.cast::<IXmlNode>()` 操作 XML
  - 使用 `CreateToastNotifierWithId()` 和 `Show()` 显示通知

### 关键技术点

1. **HSTRING**：Windows 字符串类型，使用 `HSTRING::from()` 转换
2. **类型转换**：使用 `.cast::<IXmlNode>()` 将 `XmlText` 转换为 `IXmlNode`
3. **Interface trait**：必须导入才能使用 `.cast()` 方法
4. **单例模式**：使用 `OnceLock` 确保 notifier 只创建一次

### 测试结果

运行 `./target/release/ccn.exe test` 成功发送 Windows Toast 通知，用户确认在屏幕上看到了通知弹窗！

日志输出：
```
[DEBUG] 准备发送 Windows Toast 通知: ✅成功 - CCN 测试通知 - CCN 已成功集成！
[INFO] Toast 通知已发送: ✅成功 - CCN 测试通知
```

### 已完成工作

- [x] 更新 README.md 技术栈说明（windows crate）
- [x] 更新 CHANGELOG.md（依赖、功能说明）
- [x] 更新 design.md（实现方案、技术细节）
- [x] 所有测试通过，用户确认通知显示正常
