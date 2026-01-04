# 发布流程指南

本文档说明如何使用 GitHub Actions CI/CD 流水线自动构建和发布 CCN。

## 自动发布流程

CCN 使用 GitHub Actions 实现自动化发布：

1. 创建并推送 git tag
2. GitHub Actions 自动触发
3. 构建多平台二进制文件
4. 生成 SHA256 校验和
5. 创建 GitHub Release
6. 上传所有构建产物

## 如何发布新版本

### 方式 1: 使用发布脚本（推荐）

#### Linux/macOS:

```bash
./release.sh v1.0.0
```

#### Windows (PowerShell):

```powershell
.\release.ps1 -Version v1.0.0
```

脚本会自动：
- 更新 `Cargo.toml` 中的版本号
- 创建 git tag
- 推送 tag 到远程仓库
- 触发 GitHub Actions 自动构建

### 方式 2: 手动发布

```bash
# 1. 更新版本号
vim Cargo.toml  # 修改 version 字段

# 2. 提交变更
git add Cargo.toml
git commit -m "chore: bump version to 1.0.0"

# 3. 创建 tag
git tag -a v1.0.0 -m "Release v1.0.0"

# 4. 推送
git push origin main
git push origin v1.0.0
```

## 构建的平台

CI/CD 流水线会自动构建以下平台的二进制文件：

| 平台 | 目标三元组 | 文件名 |
|------|-----------|--------|
| Windows x64 | x86_64-pc-windows-msvc | ccn-windows-x86_64.exe |
| Windows ARM64 | aarch64-pc-windows-msvc | ccn-windows-aarch64.exe |
| macOS Intel | x86_64-apple-darwin | ccn-macos-x86_64 |
| macOS Apple Silicon | aarch64-apple-darwin | ccn-macos-aarch64 |

## 自动生成的内容

每次发布会自动生成：

- ✅ 多平台二进制文件
- ✅ SHA256 校验和文件
- ✅ GitHub Release 页面
- ✅ Release notes（从 git commits 生成）
- ✅ Scoop manifest 更新

## 持续集成 (CI)

每次 push 或 pull request 都会触发 CI 检查：

- ✅ 运行单元测试
- ✅ 代码格式检查 (rustfmt)
- ✅ 代码质量检查 (clippy)
- ✅ 安全审计 (cargo-audit)
- ✅ 文档生成检查

查看 CI 状态：https://github.com/yangshare/claude-code-notify/actions

## 版本号规范

遵循 [语义化版本](https://semver.org/lang/zh-CN/)：

- **主版本号** (Major): 不兼容的 API 变更
- **次版本号** (Minor): 向下兼容的功能性新增
- **修订号** (Patch): 向下兼容的问题修正

示例：
- `v1.0.0` - 首个稳定版本
- `v1.1.0` - 新增功能
- `v1.1.1` - Bug 修复
- `v2.0.0` - 重大变更

## 发布检查清单

发布前确认：

- [ ] 所有测试通过
- [ ] CI/CD 检查无错误
- [ ] 更新 CHANGELOG.md
- [ ] 版本号符合语义化版本规范
- [ ] Release notes 准备就绪

## Scoop 更新

发布流程会自动更新 Scoop bucket manifest。用户可以运行：

```powershell
scoop update ccn
```

来获取最新版本。

## 回滚计划

如果发布出现问题：

1. 删除有问题的 Release
2. 从 GitHub 删除对应的 tag
3. 本地删除 tag: `git tag -d v1.0.0`
4. 修复问题后重新发布

## 常见问题

### Q: 构建失败怎么办？

A: 查看 Actions 页面的构建日志，修复错误后重新推送 tag。

### Q: 如何测试预发布版本？

A: 使用 pre-release 标签（如 `v1.0.0-beta.1`），GitHub Actions 会自动标记为预发布版本。

### Q: 如何修改构建配置？

A: 编辑 `.github/workflows/release.yml` 文件。

## 相关链接

- GitHub Actions: https://github.com/yangshare/claude-code-notify/actions
- Releases: https://github.com/yangshare/claude-code-notify/releases
- Release 配置: `.github/workflows/release.yml`
