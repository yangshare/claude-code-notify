#!/bin/bash
# 发布辅助脚本 - 用于创建和推送 git tag

set -e

# 检查是否提供了版本号
if [ -z "$1" ]; then
    echo "用法: ./release.sh <版本号>"
    echo "示例: ./release.sh v1.0.0"
    exit 1
fi

VERSION=$1

# 检查版本号格式
if [[ ! $VERSION =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "错误: 版本号格式不正确"
    echo "格式应为: v主版本号.次版本号.修订号"
    echo "示例: v1.0.0, v0.2.1, etc."
    exit 1
fi

echo "准备发布版本: $VERSION"

# 检查是否有未提交的更改
if [ -n "$(git status --porcelain)" ]; then
    echo "警告: 存在未提交的更改"
    git status
    read -p "是否继续? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 更新 Cargo.toml 中的版本
echo "更新 Cargo.toml 版本..."
sed -i.bak "s/^version = \".*\"/version = \"${VERSION#v}\"/" Cargo.toml
rm Cargo.toml.bak

# 提交版本更新
git add Cargo.toml
git commit -m "chore: bump version to ${VERSION#v}" || echo "没有版本变更需要提交"

# 创建 tag
echo "创建 git tag: $VERSION"
git tag -a "$VERSION" -m "Release $VERSION"

# 推送 tag
echo "推送 tag 到远程..."
git push origin "$VERSION"

echo ""
echo "✅ 发布完成！"
echo "GitHub Actions 将自动构建并发布 $VERSION"
echo "查看进度: https://github.com/yangshare/claude-code-notify/actions"
