# Scoop 安装指南

CCN 支持 Scoop 包管理器安装，这是 Windows 上最方便的安装方式。

## 安装步骤

### 1. 安装 Scoop（如果尚未安装）

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
irm get.scoop.sh | iex
```

### 2. 从 bucket 安装 CCN

```powershell
scoop bucket add ccn https://github.com/yangshare/claude-code-notify
scoop install ccn
```

## 更新 CCN

```powershell
scoop update ccn
```

## 卸载 CCN

```powershell
scoop uninstall ccn
```

## 手动下载

如果不想使用 Scoop，也可以从 [Releases 页面](https://github.com/yangshare/claude-code-notify/releases) 下载最新的二进制文件。
