#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/publish-release.sh <version>"
  echo "Example: ./scripts/publish-release.sh 1.0.0"
  exit 1
fi

TAG="v$VERSION"

echo "🚀 Publishing release $TAG..."

# 1. 检查 draft release 是否存在
RELEASE_EXISTS=$(gh release view "$TAG" --json isDraft -q .isDraft 2>/dev/null || echo "false")

if [ "$RELEASE_EXISTS" != "true" ]; then
  echo "❌ Draft release $TAG not found!"
  exit 1
fi

# 2. 列出所有附件
echo ""
echo "📦 Release assets:"
gh release view "$TAG" --json assets -q '.assets[] | "  - \(.name) (\(.size / 1024 / 1024 | floor)MB)"'

# 3. 确认发布
echo ""
read -p "❓ Publish this release? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "❌ Release not published"
  exit 1
fi

# 4. 发布
echo ""
echo "🚀 Publishing release..."
if ! gh release edit "$TAG" --draft=false; then
  echo "❌ Failed to publish release!"
  exit 1
fi

echo ""
echo "✅ Release $TAG published successfully!"
echo "🔗 $(gh release view "$TAG" --json url -q .url)"
