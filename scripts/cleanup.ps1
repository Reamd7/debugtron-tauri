#!/usr/bin/env pwsh
# 清理所有 releases 和 tags

Write-Host "🧹 Cleaning up all releases and tags..." -ForegroundColor Cyan

# 1. 列出所有 releases
Write-Host "`n📋 Listing all releases..." -ForegroundColor Yellow
$releases = gh release list --json tagName --limit 100 | ConvertFrom-Json

if ($releases.Count -eq 0) {
    Write-Host "✅ No releases found" -ForegroundColor Green
} else {
    Write-Host "Found $($releases.Count) release(s)" -ForegroundColor Yellow

    # 删除所有 releases
    foreach ($release in $releases) {
        $tag = $release.tagName
        Write-Host "🗑️  Deleting release: $tag" -ForegroundColor Red
        gh release delete $tag --yes 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   ✅ Deleted release: $tag" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️  Failed to delete release: $tag" -ForegroundColor Yellow
        }
    }
}

# 2. 列出所有远程 tags
Write-Host "`n📋 Listing all remote tags..." -ForegroundColor Yellow
$remoteTags = git ls-remote --tags origin | ForEach-Object {
    if ($_ -match 'refs/tags/(.+)$') {
        $matches[1] -replace '\^\{\}$', ''
    }
} | Select-Object -Unique

if ($remoteTags.Count -eq 0) {
    Write-Host "✅ No remote tags found" -ForegroundColor Green
} else {
    Write-Host "Found $($remoteTags.Count) remote tag(s)" -ForegroundColor Yellow

    # 删除所有远程 tags
    foreach ($tag in $remoteTags) {
        Write-Host "🗑️  Deleting remote tag: $tag" -ForegroundColor Red
        git push origin ":refs/tags/$tag" 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   ✅ Deleted remote tag: $tag" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️  Failed to delete remote tag: $tag" -ForegroundColor Yellow
        }
    }
}

# 3. 列出所有本地 tags
Write-Host "`n📋 Listing all local tags..." -ForegroundColor Yellow
$localTags = git tag -l

if ($localTags.Count -eq 0) {
    Write-Host "✅ No local tags found" -ForegroundColor Green
} else {
    Write-Host "Found $($localTags.Count) local tag(s)" -ForegroundColor Yellow

    # 删除所有本地 tags
    foreach ($tag in $localTags) {
        Write-Host "🗑️  Deleting local tag: $tag" -ForegroundColor Red
        git tag -d $tag 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   ✅ Deleted local tag: $tag" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️  Failed to delete local tag: $tag" -ForegroundColor Yellow
        }
    }
}

Write-Host "`n✅ Cleanup complete!" -ForegroundColor Green
Write-Host "📋 Verification:" -ForegroundColor Cyan
Write-Host "   Releases: " -NoNewline
gh release list 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    $remaining = (gh release list --json tagName --limit 100 | ConvertFrom-Json).Count
    Write-Host "$remaining" -ForegroundColor $(if ($remaining -eq 0) { "Green" } else { "Yellow" })
} else {
    Write-Host "Unable to check" -ForegroundColor Yellow
}

Write-Host "   Remote tags: " -NoNewline
$remainingRemote = (git ls-remote --tags origin | Measure-Object).Count
Write-Host "$remainingRemote" -ForegroundColor $(if ($remainingRemote -eq 0) { "Green" } else { "Yellow" })

Write-Host "   Local tags: " -NoNewline
$remainingLocal = (git tag -l | Measure-Object).Count
Write-Host "$remainingLocal" -ForegroundColor $(if ($remainingLocal -eq 0) { "Green" } else { "Yellow" })
