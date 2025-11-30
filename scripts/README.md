# 发版脚本使用指南

本目录包含用于跨平台发布 Debugtron Tauri 应用的脚本。

## 📐 架构说明

### macOS 多架构构建策略

从 v1.0.3 开始，macOS 构建采用**多架构分别构建**策略，而不是 Universal 包：

**优势**：
- ✅ **体积减少 50%**：单架构包体积约为 Universal 包的一半
- ✅ **用户友好**：用户可根据自己的 Mac 芯片下载对应版本
  - Apple Silicon (M1/M2/M3) → ARM64 版本
  - Intel Mac → x86_64 版本
- ✅ **灵活性**：可单独更新某个架构的构建

**输出产物**：
- `Debugtron_{VERSION}_macOS_arm64.dmg` - ARM64 DMG 安装包
- `Debugtron_{VERSION}_macOS_arm64.app.tar.gz` - ARM64 应用程序包
- `Debugtron_{VERSION}_macOS_x86_64.dmg` - x86_64 DMG 安装包
- `Debugtron_{VERSION}_macOS_x86_64.app.tar.gz` - x86_64 应用程序包

---

## 前置准备

### 1. 安装 GitHub CLI

**macOS**:
```bash
brew install gh
gh auth login
```

**Windows**:
```powershell
winget install --id GitHub.cli
gh auth login
```

### 2. 确保权限

**macOS/Linux**:
```bash
chmod +x scripts/*.sh
```

## 脚本说明

| 脚本 | 功能 | 平台 |
|------|------|------|
| `sync-version.cjs` | 自动同步版本号到 package.json、Cargo.toml、tauri.conf.json | 通用 |
| `release.cjs` | 更新版本号 → 提交 → 创建 tag → 推送 | 通用（推荐） |
| `release.sh` | 更新版本号 → 提交 → 创建 tag → 推送 | macOS/Linux |
| `build-and-upload.sh` | 构建 macOS 产物（ARM64 + x86_64）→ 创建/追加到 draft release → 上传 | macOS |
| `build-and-upload.ps1` | 构建 Windows 产物 → 创建/追加到 draft release → 上传 | Windows |
| `publish-release.cjs` | 检查产物 → 人工确认 → 发布正式版本 | 通用（推荐） |
| `publish-release.sh` | 检查产物 → 人工确认 → 发布正式版本 | macOS/Linux |

## 🚀 完整发版流程（从零开始）

### 📋 前置检查

在开始发版之前，确认以下事项：

```powershell
# 1. 确认当前状态干净
git status
# 应该没有未提交的更改

# 2. 确认当前版本号
node -p "require('./package.json').version"
# 例如: 1.0.1

# 3. 确认 GitHub CLI 已认证
gh auth status
```

### 🎯 版本号选择

使用语义化版本号（Semantic Versioning）：

- **补丁版本**（bug 修复）：`1.0.1` → `1.0.2`
- **次要版本**（新功能，向后兼容）：`1.0.1` → `1.1.0`
- **主要版本**（重大变更，不向后兼容）：`1.0.1` → `2.0.0`

### Step 1: 创建版本和 Tag（任意平台，只需执行一次）

**方式 1: 使用 npm script（推荐，跨平台）**

```powershell
# Windows PowerShell / macOS Terminal
npm run release -- 1.0.2
```

**方式 2: 直接调用脚本**

```powershell
# Windows PowerShell
node scripts/release.cjs 1.0.2

# macOS / Linux Terminal
node scripts/release.cjs 1.0.2
```

**方式 3: 使用 Shell 脚本（macOS/Linux）**

```bash
./scripts/release.sh 1.0.2
```

**这个脚本会：**
- ✅ 验证版本号格式
- ✅ 检查版本号是否与当前版本不同
- ✅ 更新 package.json、Cargo.toml、tauri.conf.json 的版本号
- ✅ 提交版本号更改（commit message: `chore: release v1.0.2`）
- ✅ 创建 Git Tag `v1.0.2`
- ✅ 推送到 GitHub

**预期输出：**
```
📦 Preparing release v1.0.2...
   Current version: 1.0.1
   New version: 1.0.2

📝 Updating version numbers...
✅ Synced version to 1.0.2

💾 Committing version changes...

🏷️  Creating tag v1.0.2...

⬆️  Pushing to GitHub...

✅ Tag v1.0.2 created and pushed!

📋 Next steps:
  1. On macOS machine: ./scripts/build-and-upload.sh 1.0.2
  2. On Windows machine: .\scripts\build-and-upload.ps1 1.0.2
  3. After both platforms complete: node scripts/publish-release.cjs 1.0.2
```

### Step 2: 在 Windows 机器上构建并上传

```powershell
# Windows PowerShell
.\scripts\build-and-upload.ps1 1.0.2
```

**这个脚本会：**
- ✅ 检查 draft release 是否存在（不存在则自动创建）
- ✅ 运行 `npm install`
- ✅ 运行 `npm run tauri build`
- ✅ 查找构建产物（.exe 和 .msi）
- ✅ 上传到 GitHub Draft Release

**预期输出：**
```
🪟 Building Windows version 1.0.2...

📋 Checking if draft release exists...
📝 Creating draft release v1.0.2...
✅ Draft release created successfully

🔨 Building Windows application...
（构建过程，约 5-10 分钟...）

⬆️  Uploading Windows artifacts to release...
✅ Upload successful

✅ Windows build complete and uploaded!
   EXE: Debugtron_1.0.2_Windows_x64-setup.exe
   MSI: Debugtron_1.0.2_Windows_x64.msi
```

**验证上传：**
```powershell
# 查看 draft release
gh release view v1.0.2

# 查看附件（应该看到 2 个文件）
gh release view v1.0.2 --json assets --jq '.assets[] | "\(.name) (\(.size / 1024 / 1024 | floor)MB)"'
```

### Step 3: 在 macOS 机器上构建并上传（可选）

```bash
# macOS Terminal
./scripts/build-and-upload.sh 1.0.2
```

**这个脚本会：**
- ✅ 检查 draft release（应该已存在）
- ✅ 分别构建 ARM64 和 x86_64 两个架构（减少单个包体积）
- ✅ 为每个架构生成 DMG 和 .app.tar.gz
- ✅ 上传所有产物到同一个 GitHub Draft Release

**预期输出：**
```
🍎 Building macOS version 1.0.2 for multiple architectures...

📋 Checking if draft release exists...
✅ Draft release v1.0.2 already exists

📦 Installing dependencies...

🔨 Building for arm64 (aarch64-apple-darwin)...
（构建过程...）
📦 Packaging arm64 .app bundle...
✅ arm64 build complete
   DMG: Debugtron.dmg → Debugtron_1.0.2_macOS_arm64.dmg
   APP: Debugtron_1.0.2_macOS_arm64.app.tar.gz

🔨 Building for x86_64 (x86_64-apple-darwin)...
（构建过程...）
📦 Packaging x86_64 .app bundle...
✅ x86_64 build complete
   DMG: Debugtron.dmg → Debugtron_1.0.2_macOS_x86_64.dmg
   APP: Debugtron_1.0.2_macOS_x86_64.app.tar.gz

⬆️  Uploading all macOS artifacts to release...

🧹 Cleaning up temporary files...

✅ All macOS builds complete and uploaded!
   Architectures: arm64 x86_64
   Release: v1.0.2
```

**体积优化说明：**
- ARM64 单架构包体积约为 Universal 包的 50%
- x86_64 单架构包体积约为 Universal 包的 50%
- 用户可根据自己的 Mac 芯片类型下载对应版本

**如果只在 Windows 上构建：**
- 可以跳过这一步
- 直接进入 Step 4 发布

### Step 4: 检查并发布 Release（任意平台）

**先检查所有产物：**

```powershell
# 查看 draft release 详情
gh release view v1.0.2

# 查看所有附件和大小
gh release view v1.0.2 --json assets --jq '.assets[] | "\(.name) (\(.size / 1024 / 1024 | floor)MB)"'
```

**预期看到：**

**仅 Windows**：
```
Debugtron_1.0.2_Windows_x64-setup.exe (30MB)
Debugtron_1.0.2_Windows_x64.msi (28MB)
```

**Windows + macOS（多架构）**：
```
Debugtron_1.0.2_Windows_x64-setup.exe (30MB)
Debugtron_1.0.2_Windows_x64.msi (28MB)
Debugtron_1.0.2_macOS_arm64.dmg (13MB)        # macOS ARM64 架构
Debugtron_1.0.2_macOS_arm64.app.tar.gz (10MB)
Debugtron_1.0.2_macOS_x86_64.dmg (13MB)       # macOS x86_64 架构
Debugtron_1.0.2_macOS_x86_64.app.tar.gz (10MB)
```

**发布 Release：**

**方式 1: 使用 npm script（推荐）**

```powershell
npm run publish-release -- 1.0.2
```

**方式 2: 直接调用脚本**

```powershell
node scripts/publish-release.cjs 1.0.2
```

**方式 3: 使用 Shell 脚本（macOS/Linux）**

```bash
./scripts/publish-release.sh 1.0.2
```

**交互式确认：**
```
🚀 Publishing release v1.0.2...

📋 Checking draft release...

📦 Release assets:
  - Debugtron_1.0.2_Windows_x64-setup.exe (30MB)
  - Debugtron_1.0.2_Windows_x64.msi (28MB)

❓ Publish this release? (y/N): y

🚀 Publishing release...

✅ Release v1.0.2 published successfully!
🔗 https://github.com/your-org/debugtron-tauri/releases/tag/v1.0.2
```

**在浏览器中查看：**
```powershell
# 自动打开浏览器
gh release view v1.0.2 --web
```

---

## 🎉 完成！

发布成功后，用户可以从 GitHub Release 页面下载安装包。

---

## ⚡ 快速参考（仅 Windows）

如果你只在 Windows 上构建，完整流程只需 3 步：

```powershell
# 1. 创建版本和 tag
npm run release -- 1.0.2

# 2. 构建并上传
.\scripts\build-and-upload.ps1 1.0.2

# 3. 检查并发布
gh release view v1.0.2                    # 检查产物
npm run publish-release -- 1.0.2          # 发布
```

---

## 📝 注意事项

1. **版本号格式**: 必须使用语义化版本号，如 `1.0.0`、`1.2.3`
2. **Draft Release 机制**: macOS 和 Windows 可以异步构建，互不阻塞
3. **幂等性**: 脚本可以重复运行，`--clobber` 会覆盖已有文件
4. **人工确认**: 发布前可以手动检查所有产物
5. **分支**: 默认推送到 `master` 分支，如需修改请编辑脚本
6. **错误处理**: 所有脚本都有完整的错误检查，失败时会明确提示

## 📋 多平台发版示例

**跨平台完整流程**:

```powershell
# Step 1: 在开发机上创建版本（任意平台，只需一次）
npm run release -- 1.0.2

# Step 2: 在 Windows 机器上
.\scripts\build-and-upload.ps1 1.0.2

# Step 3: 在 macOS 机器上（可选）
./scripts/build-and-upload.sh 1.0.2

# Step 4: 检查并发布（任意平台）
gh release view v1.0.2                    # 检查产物
npm run publish-release -- 1.0.2          # 发布
```

## 🔧 故障排查

### Step 1 失败

**版本号已存在**：
```
❌ Version 1.0.2 is already the current version!
```
解决方案：使用不同的版本号

**版本号格式错误**：
```
❌ Invalid version format!
```
解决方案：使用语义化版本号，如 `1.0.2`

**未提交的更改**：
```
❌ Failed to commit changes!
```
解决方案：先提交或暂存所有更改

### Step 2/3 失败

**GitHub CLI 未认证**：
```powershell
gh auth status
gh auth login
```

**找不到构建产物**：
```
❌ EXE installer not found!
```
解决方案：
- 检查构建是否成功
- 查看 `src-tauri/target/release/bundle/` 目录

**创建 Release 失败**：
```
❌ Failed to create release!
```
解决方案：
- 确认 tag 已推送：`git ls-remote --tags origin`
- 检查 GitHub 权限

**上传失败**：
```
❌ Failed to upload artifacts!
```
解决方案：
- 检查网络连接
- 重新运行脚本（会覆盖已有文件）

### Step 4 失败

**找不到 Draft Release**：
```
❌ Draft release v1.0.2 not found!
```
解决方案：
```powershell
# 检查是否存在
gh release list

# 手动创建（如果需要）
gh release create v1.0.2 --draft --title "Release 1.0.2" --notes "Release notes"
```

**没有附件**：
```powershell
# 检查附件
gh release view v1.0.2 --json assets

# 重新上传
.\scripts\build-and-upload.ps1 1.0.2
```

### 需要重新开始

如果发版过程出现问题，想要完全重新开始：

```powershell
# 运行清理脚本
.\scripts\cleanup.ps1

# 从 Step 1 重新开始
npm run release -- 1.0.2
```

### 脚本无执行权限（macOS/Linux）
```bash
chmod +x scripts/*.sh
```

### 构建时间过长
- Windows 构建约需 5-10 分钟
- macOS ARM64 构建约需 5-8 分钟
- macOS x86_64 构建约需 5-8 分钟
- macOS 完整构建（两个架构）约需 10-15 分钟
- 首次构建会更慢（需要下载依赖）

## 🧹 清理工具

如果需要删除所有 releases 和 tags 重新开始：

**Windows**:
```powershell
.\scripts\cleanup.ps1
```

**macOS/Linux**:
```bash
./scripts/cleanup.sh
```

这会删除：
- ✅ 所有 GitHub Releases
- ✅ 所有远程 Tags
- ✅ 所有本地 Tags

---

## 📚 相关资源

- [GitHub CLI 文档](https://cli.github.com/)
- [Tauri 构建文档](https://tauri.app/v1/guides/building/)
- [语义化版本规范](https://semver.org/)
- [Draft Release 说明](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)