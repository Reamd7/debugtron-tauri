#!/bin/bash
# 清理所有 releases 和 tags

set +e  # 允许命令失败继续执行

echo "🧹 Cleaning up all releases and tags..."

# 1. 列出并删除所有 releases
echo ""
echo "📋 Listing all releases..."
RELEASES=$(gh release list --json tagName --limit 100 --jq '.[].tagName' 2>/dev/null)

if [ -z "$RELEASES" ]; then
    echo "✅ No releases found"
else
    echo "Found releases, deleting..."
    while IFS= read -r tag; do
        echo "🗑️  Deleting release: $tag"
        if gh release delete "$tag" --yes 2>/dev/null; then
            echo "   ✅ Deleted release: $tag"
        else
            echo "   ⚠️  Failed to delete release: $tag"
        fi
    done <<< "$RELEASES"
fi

# 2. 列出并删除所有远程 tags
echo ""
echo "📋 Listing all remote tags..."
REMOTE_TAGS=$(git ls-remote --tags origin | awk '{print $2}' | sed 's|refs/tags/||' | sed 's|\^{}||' | sort -u)

if [ -z "$REMOTE_TAGS" ]; then
    echo "✅ No remote tags found"
else
    echo "Found remote tags, deleting..."
    while IFS= read -r tag; do
        if [ -n "$tag" ]; then
            echo "🗑️  Deleting remote tag: $tag"
            if git push origin ":refs/tags/$tag" 2>/dev/null; then
                echo "   ✅ Deleted remote tag: $tag"
            else
                echo "   ⚠️  Failed to delete remote tag: $tag"
            fi
        fi
    done <<< "$REMOTE_TAGS"
fi

# 3. 列出并删除所有本地 tags
echo ""
echo "📋 Listing all local tags..."
LOCAL_TAGS=$(git tag -l)

if [ -z "$LOCAL_TAGS" ]; then
    echo "✅ No local tags found"
else
    echo "Found local tags, deleting..."
    while IFS= read -r tag; do
        if [ -n "$tag" ]; then
            echo "🗑️  Deleting local tag: $tag"
            if git tag -d "$tag" 2>/dev/null; then
                echo "   ✅ Deleted local tag: $tag"
            else
                echo "   ⚠️  Failed to delete local tag: $tag"
            fi
        fi
    done <<< "$LOCAL_TAGS"
fi

# 验证
echo ""
echo "✅ Cleanup complete!"
echo "📋 Verification:"

REMAINING_RELEASES=$(gh release list --json tagName --limit 100 --jq '. | length' 2>/dev/null || echo "0")
echo "   Releases: $REMAINING_RELEASES"

REMAINING_REMOTE=$(git ls-remote --tags origin | wc -l)
echo "   Remote tags: $REMAINING_REMOTE"

REMAINING_LOCAL=$(git tag -l | wc -l)
echo "   Local tags: $REMAINING_LOCAL"
