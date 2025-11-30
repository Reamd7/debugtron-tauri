#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/build-and-upload.sh <version>"
  echo "Example: ./scripts/build-and-upload.sh 1.0.0"
  exit 1
fi

TAG="v$VERSION"

echo "🍎 Building macOS version $VERSION for multiple architectures..."

# 1. 检查 Draft Release 是否存在
echo "📋 Checking if draft release exists..."
RELEASE_EXISTS=$(gh release view "$TAG" --json isDraft -q .isDraft 2>/dev/null || echo "false")

if [ "$RELEASE_EXISTS" = "false" ]; then
  echo "📝 Creating draft release $TAG..."
  if ! gh release create "$TAG" \
    --draft \
    --title "Release $VERSION" \
    --notes "Release notes for version $VERSION"; then
    echo "❌ Failed to create release!"
    exit 1
  fi
  echo "✅ Draft release created successfully"
else
  echo "✅ Draft release $TAG already exists"
fi

# 2. 安装依赖
echo "📦 Installing dependencies..."
npm install

# 定义架构列表
ARCHITECTURES=("aarch64-apple-darwin" "x86_64-apple-darwin")
ARCH_NAMES=("arm64" "x86_64")

# 用于收集所有产物路径
UPLOAD_ARGS=""

# 3. 遍历每个架构进行构建
for i in "${!ARCHITECTURES[@]}"; do
  ARCH="${ARCHITECTURES[$i]}"
  ARCH_NAME="${ARCH_NAMES[$i]}"

  echo ""
  echo "🔨 Building for $ARCH_NAME ($ARCH)..."

  # 构建指定架构
  npm run tauri build -- --target "$ARCH"

  # 查找构建产物
  DMG_PATH=$(find "src-tauri/target/$ARCH/release/bundle/dmg" -name "*.dmg" | head -n 1)
  APP_PATH="src-tauri/target/$ARCH/release/bundle/macos/Debugtron.app"

  if [ ! -f "$DMG_PATH" ]; then
    echo "❌ DMG file not found for $ARCH_NAME!"
    exit 1
  fi

  if [ ! -d "$APP_PATH" ]; then
    echo "❌ .app bundle not found for $ARCH_NAME!"
    exit 1
  fi

  # 打包 .app
  APP_ARCHIVE="Debugtron_${VERSION}_macOS_${ARCH_NAME}.app.tar.gz"
  echo "📦 Packaging $ARCH_NAME .app bundle..."
  tar -czf "$APP_ARCHIVE" -C "$(dirname "$APP_PATH")" "$(basename "$APP_PATH")"

  # 重命名 DMG 文件（直接复制为新文件名）
  DMG_RENAMED="Debugtron_${VERSION}_macOS_${ARCH_NAME}.dmg"
  echo "📝 Renaming DMG file..."
  cp "$DMG_PATH" "$DMG_RENAMED"

  # 添加到上传参数
  UPLOAD_ARGS="$UPLOAD_ARGS \"$DMG_RENAMED\" \"$APP_ARCHIVE\""

  echo "✅ $ARCH_NAME build complete"
  echo "   DMG: $DMG_RENAMED"
  echo "   APP: $APP_ARCHIVE"
done

# 4. 上传所有产物到 Release
echo ""
echo "⬆️  Uploading all macOS artifacts to release..."

# 使用 eval 执行动态构建的命令
if ! eval gh release upload "$TAG" "$UPLOAD_ARGS" --clobber; then
  echo "❌ Failed to upload artifacts!"
  exit 1
fi

# 5. 清理临时文件
echo ""
echo "🧹 Cleaning up temporary files..."
for ARCH_NAME in "${ARCH_NAMES[@]}"; do
  rm -f "Debugtron_${VERSION}_macOS_${ARCH_NAME}.dmg"
  rm -f "Debugtron_${VERSION}_macOS_${ARCH_NAME}.app.tar.gz"
done

echo ""
echo "✅ All macOS builds complete and uploaded!"
echo "   Architectures: ${ARCH_NAMES[*]}"
echo "   Release: $TAG"
