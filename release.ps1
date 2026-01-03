# 发布辅助脚本 - PowerShell 版本

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

# 检查版本号格式
if ($Version -notmatch '^v\d+\.\d+\.\d+$') {
    Write-Host "错误: 版本号格式不正确" -ForegroundColor Red
    Write-Host "格式应为: v主版本号.次版本号.修订号"
    Write-Host "示例: v1.0.0, v0.2.1"
    exit 1
}

Write-Host "准备发布版本: $Version" -ForegroundColor Green

# 检查是否有未提交的更改
$status = git status --porcelain
if ($status) {
    Write-Host "警告: 存在未提交的更改" -ForegroundColor Yellow
    git status
    $confirm = Read-Host "是否继续? (y/n)"
    if ($confirm -ne 'y') {
        exit 1
    }
}

# 更新 Cargo.toml 中的版本
Write-Host "更新 Cargo.toml 版本..." -ForegroundColor Cyan
$cargoContent = Get-Content Cargo.toml -Raw
$cargoContent = $cargoContent -replace '^version = ".*"', "version = `"$($Version.Substring(1))`""
Set-Content Cargo.toml -Value $cargoContent -NoNewline

# 提交版本更新
git add Cargo.toml
try {
    git commit -m "chore: bump version to $($Version.Substring(1))"
} catch {
    Write-Host "没有版本变更需要提交" -ForegroundColor Yellow
}

# 创建 tag
Write-Host "创建 git tag: $Version" -ForegroundColor Cyan
git tag -a $Version -m "Release $Version"

# 推送 tag
Write-Host "推送 tag 到远程..." -ForegroundColor Cyan
git push origin $Version

Write-Host ""
Write-Host "✅ 发布完成！" -ForegroundColor Green
Write-Host "GitHub Actions 将自动构建并发布 $Version"
Write-Host "查看进度: https://github.com/yangshare/claude-code-notify/actions"
