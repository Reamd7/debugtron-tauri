param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"
$TAG = "v$Version"

Write-Host "🪟 Building Windows version $Version..." -ForegroundColor Cyan

# 1. 检查 Draft Release 是否存在
Write-Host "📋 Checking if draft release exists..." -ForegroundColor Yellow
$releaseExists = $false
try {
    $releaseInfo = gh release view $TAG --json isDraft 2>&1
    if ($LASTEXITCODE -eq 0) {
        $releaseData = $releaseInfo | ConvertFrom-Json
        $releaseExists = $true
        Write-Host "✅ Draft release $TAG already exists" -ForegroundColor Green
    }
} catch {
    # Release doesn't exist, will create it
}

if (-not $releaseExists) {
    Write-Host "📝 Creating draft release $TAG..." -ForegroundColor Yellow
    $createOutput = gh release create $TAG --draft --title "Release $Version" --notes "Release notes for version $Version" 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed to create release!" -ForegroundColor Red
        Write-Host $createOutput -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Draft release created successfully" -ForegroundColor Green
}

# 2. 构建 Windows 应用
Write-Host "🔨 Building Windows application..." -ForegroundColor Yellow
npm install
npm run tauri build

# 3. 查找构建产物
$EXE_PATH = Get-ChildItem -Path "src-tauri\target\release\bundle\nsis" -Filter "*-setup.exe" -Recurse | Select-Object -First 1 -ExpandProperty FullName
$MSI_PATH = Get-ChildItem -Path "src-tauri\target\release\bundle\msi" -Filter "*.msi" -Recurse | Select-Object -First 1 -ExpandProperty FullName

if (-not $EXE_PATH) {
    Write-Host "❌ EXE installer not found!" -ForegroundColor Red
    exit 1
}

# 4. 重命名文件（带版本号）
$EXE_NAME = "Debugtron_${Version}_Windows_x64-setup.exe"
$MSI_NAME = "Debugtron_${Version}_Windows_x64.msi"

Copy-Item $EXE_PATH -Destination $EXE_NAME
if ($MSI_PATH) {
    Copy-Item $MSI_PATH -Destination $MSI_NAME
}

# 5. 上传到 Release
Write-Host "⬆️  Uploading Windows artifacts to release..." -ForegroundColor Yellow
if ($MSI_PATH) {
    $uploadOutput = gh release upload $TAG $EXE_NAME $MSI_NAME --clobber 2>&1
} else {
    $uploadOutput = gh release upload $TAG $EXE_NAME --clobber 2>&1
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to upload artifacts!" -ForegroundColor Red
    Write-Host $uploadOutput -ForegroundColor Red
    exit 1
}
Write-Host "✅ Upload successful" -ForegroundColor Green

# 6. 清理临时文件
Remove-Item $EXE_NAME
if ($MSI_PATH) {
    Remove-Item $MSI_NAME
}

Write-Host ""
Write-Host "✅ Windows build complete and uploaded!" -ForegroundColor Green
Write-Host "   EXE: $EXE_NAME" -ForegroundColor Cyan
if ($MSI_PATH) {
    Write-Host "   MSI: $MSI_NAME" -ForegroundColor Cyan
}
