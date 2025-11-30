#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh <version>"
  echo "Example: ./scripts/release.sh 1.0.0"
  exit 1
fi

# 验证版本号格式
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "❌ Invalid version format! Please use semantic versioning (e.g., 1.0.0)"
  exit 1
fi

# 读取当前版本号
CURRENT_VERSION=$(node -p "require('./package.json').version")

if [ "$CURRENT_VERSION" = "$VERSION" ]; then
  echo "❌ Version $VERSION is already the current version!"
  echo "Please specify a different version number."
  exit 1
fi

echo "📦 Preparing release v$VERSION..."
echo "   Current version: $CURRENT_VERSION"
echo "   New version: $VERSION"

# 1. 更新版本号
echo ""
echo "📝 Updating version numbers..."
if ! npm version $VERSION --no-git-tag-version; then
  echo "❌ Failed to update version!"
  exit 1
fi

# 2. 提交
echo ""
echo "💾 Committing version changes..."
git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
if ! git commit -m "chore: release v$VERSION"; then
  echo "❌ Failed to commit changes!"
  exit 1
fi

# 3. 创建 tag
echo ""
echo "🏷️  Creating tag v$VERSION..."
if ! git tag "v$VERSION"; then
  echo "❌ Failed to create tag!"
  exit 1
fi

# 4. 推送
echo ""
echo "⬆️  Pushing to GitHub..."
if ! git push origin master; then
  echo "❌ Failed to push commits!"
  exit 1
fi

if ! git push origin "v$VERSION"; then
  echo "❌ Failed to push tag!"
  exit 1
fi

echo ""
echo "✅ Tag v$VERSION created and pushed!"
echo ""
echo "📋 Next steps:"
echo "  1. On macOS machine: ./scripts/build-and-upload.sh $VERSION"
echo "  2. On Windows machine: .\\scripts\\build-and-upload.ps1 $VERSION"
echo "  3. After both platforms complete: ./scripts/publish-release.sh $VERSION"
