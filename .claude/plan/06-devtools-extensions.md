# DevTools 扩展集成支持 - 实施计划

**创建时间**: 2025-12-01
**功能优先级**: ⭐⭐⭐ P1 (高优先级)
**预计工作量**: 9 天 / 1.5 周

---

## 📋 目录

1. [功能概述](#功能概述)
2. [用户需求分析](#用户需求分析)
3. [技术架构](#技术架构)
4. [实现计划](#实现计划)
5. [关键技术挑战](#关键技术挑战)
6. [文件清单](#文件清单)
7. [测试计划](#测试计划)
8. [用户文档](#用户文档)
9. [成功指标](#成功指标)

---

## 功能概述

### 目标

允许用户在 Debugtron 中加载 Chrome DevTools 扩展（如 React DevTools、Redux DevTools、Vue.js DevTools），以便在调试 Electron 应用和网页时获得完整的调试体验。

### 核心价值

- ✅ 支持 React/Vue/Redux/Apollo 等框架专用的 DevTools 扩展
- ✅ 提供统一的扩展管理界面
- ✅ 兼容 Chrome 137+ 的安全限制
- ✅ 提升开发者调试体验

### 应用场景

1. **React 应用调试**: 使用 React DevTools 检查组件树、Props、State
2. **Redux 状态管理**: 使用 Redux DevTools 查看 Action、State 变化
3. **Vue 应用调试**: 使用 Vue.js DevTools 检查组件和 Vuex
4. **GraphQL 调试**: 使用 Apollo Client DevTools 查看查询和缓存

---

## 用户需求分析

### 用户痛点

**问题**: 开发者在调试 React/Vue 应用时，Chrome DevTools 默认不包含框架专用的调试面板。

**现状**: 开发者需要：
1. 手动在 Chrome 中安装扩展
2. 使用复杂的命令行参数启动 Chrome
3. 无法在 Debugtron 中直接使用这些扩展

**期望**: 在 Debugtron 中一键配置扩展，启动调试时自动加载。

### 用户决策

根据用户需求调研，确定以下技术路线：

| 决策点 | 用户选择 | 理由 |
|--------|---------|------|
| **扩展来源** | 用户手动配置本地路径 | 简单、可控、无需下载 |
| **加载时机** | 调试 Electron 或网页前 | 确保扩展在 DevTools 打开时可用 |
| **Chrome 兼容性** | 添加兼容标志 | 兼容 Chrome 137+ 安全限制 |

---

## 技术架构

### 整体流程

```
用户配置扩展路径 (Settings)
  ↓
ExtensionManager 扫描并验证 manifest.json
  ↓
存储到 ~/.debugtron/extensions.json
  ↓
启动浏览器/Electron 时添加启动参数
  ↓
Chrome 加载扩展 (--load-extension)
  ↓
DevTools 中可用 React/Redux 面板
```

### 核心组件设计

#### 1. ExtensionManager (Rust)

**文件**: `src-tauri/src/extensions/mod.rs`

```rust
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 扩展配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    pub id: String,           // UUID
    pub name: String,         // 显示名称（如 "React DevTools"）
    pub path: PathBuf,        // 本地路径
    pub enabled: bool,        // 是否启用
    pub manifest_version: u8, // Manifest V2/V3
}

/// 扩展管理器
#[derive(Clone)]
pub struct ExtensionManager {
    config_path: PathBuf,
    extensions: Arc<Mutex<Vec<ExtensionConfig>>>,
}

impl ExtensionManager {
    /// 创建新的扩展管理器实例
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        Self {
            config_path,
            extensions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 获取配置文件路径
    fn get_config_path() -> PathBuf {
        let config_dir = dirs::home_dir()
            .expect("Cannot find home directory")
            .join(".debugtron");

        // 确保配置目录存在
        let _ = fs::create_dir_all(&config_dir);

        config_dir.join("extensions.json")
    }

    /// 加载配置文件
    pub async fn load_config(&self) -> Result<()> {
        if !self.config_path.exists() {
            // 配置文件不存在，创建空配置
            self.save_config().await?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: ExtensionsConfigFile = serde_json::from_str(&content)?;

        let mut extensions = self.extensions.lock().await;
        *extensions = config.extensions;

        println!("[ExtensionManager] Loaded {} extensions", extensions.len());
        Ok(())
    }

    /// 保存配置文件
    pub async fn save_config(&self) -> Result<()> {
        let extensions = self.extensions.lock().await;
        let config = ExtensionsConfigFile {
            extensions: extensions.clone(),
        };

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, content)?;

        println!("[ExtensionManager] Config saved to {:?}", self.config_path);
        Ok(())
    }

    /// 验证扩展路径是否有效
    pub fn validate_extension(&self, path: &Path) -> Result<ExtensionConfig> {
        if !path.exists() {
            return Err(anyhow!("Extension directory does not exist: {:?}", path));
        }

        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(anyhow!("manifest.json not found in {:?}", path));
        }

        let manifest_content = fs::read_to_string(manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

        // 提取扩展信息
        let name = manifest["name"]
            .as_str()
            .unwrap_or("Unknown Extension")
            .to_string();

        let manifest_version = manifest["manifest_version"]
            .as_u64()
            .unwrap_or(2) as u8;

        Ok(ExtensionConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            path: path.to_path_buf(),
            enabled: true,
            manifest_version,
        })
    }

    /// 添加扩展
    pub async fn add(&self, extension: ExtensionConfig) -> Result<()> {
        let mut extensions = self.extensions.lock().await;

        // 检查是否已存在相同路径的扩展
        if extensions.iter().any(|e| e.path == extension.path) {
            return Err(anyhow!("Extension already exists at this path"));
        }

        println!("[ExtensionManager] Adding extension: {}", extension.name);
        extensions.push(extension);

        Ok(())
    }

    /// 删除扩展
    pub async fn remove(&self, id: &str) -> Result<()> {
        let mut extensions = self.extensions.lock().await;

        let initial_len = extensions.len();
        extensions.retain(|e| e.id != id);

        if extensions.len() == initial_len {
            return Err(anyhow!("Extension not found: {}", id));
        }

        println!("[ExtensionManager] Removed extension: {}", id);
        Ok(())
    }

    /// 切换扩展启用状态
    pub async fn toggle(&self, id: &str) -> Result<()> {
        let mut extensions = self.extensions.lock().await;

        if let Some(ext) = extensions.iter_mut().find(|e| e.id == id) {
            ext.enabled = !ext.enabled;
            println!(
                "[ExtensionManager] Toggled extension {}: enabled={}",
                ext.name, ext.enabled
            );
            Ok(())
        } else {
            Err(anyhow!("Extension not found: {}", id))
        }
    }

    /// 获取所有扩展
    pub async fn get_all(&self) -> Result<Vec<ExtensionConfig>> {
        let extensions = self.extensions.lock().await;
        Ok(extensions.clone())
    }

    /// 生成 Chrome 启动参数
    pub async fn get_launch_args(&self) -> Vec<String> {
        let extensions = self.extensions.lock().await;

        let enabled_paths: Vec<String> = extensions
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.path.to_string_lossy().to_string())
            .collect();

        if enabled_paths.is_empty() {
            return vec![];
        }

        #[cfg(target_os = "windows")]
        let separator = ";";
        #[cfg(not(target_os = "windows"))]
        let separator = ":";

        let joined = enabled_paths.join(separator);

        vec![
            // Chrome 137+ 兼容标志
            "--disable-features=DisableLoadExtensionCommandLineSwitch".to_string(),

            // 加载扩展
            format!("--load-extension={}", joined),

            // 仅保留这些扩展（禁用其他）
            format!("--disable-extensions-except={}", joined),
        ]
    }
}

/// 扩展管理器配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtensionsConfigFile {
    extensions: Vec<ExtensionConfig>,
}
```

#### 2. 扩展配置文件

**文件路径**: `~/.debugtron/extensions.json`

```json
{
  "extensions": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "React Developer Tools",
      "path": "/Users/user/Library/Application Support/Google/Chrome/Default/Extensions/fmkadmapgofadopljbjfkapdkoienihi/4.28.0_0",
      "enabled": true,
      "manifest_version": 3
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "Redux DevTools",
      "path": "/Users/user/Library/Application Support/Google/Chrome/Default/Extensions/lmhkpmbekcpmknklioeibfkpmmfibljd/3.1.0_0",
      "enabled": true,
      "manifest_version": 2
    }
  ]
}
```

#### 3. Tauri Commands

**文件**: `src-tauri/src/commands.rs` (扩展部分)

```rust
use crate::extensions::ExtensionManager;
use tauri::State;

#[tauri::command]
pub async fn get_extensions(
    extension_manager: State<'_, ExtensionManager>,
) -> Result<Vec<crate::extensions::ExtensionConfig>, String> {
    println!("[COMMAND] get_extensions called");
    extension_manager
        .get_all()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_extension(
    path: String,
    extension_manager: State<'_, ExtensionManager>,
) -> Result<(), String> {
    println!("[COMMAND] add_extension called with path: {}", path);

    use std::path::Path;

    // 验证扩展
    let ext = extension_manager
        .validate_extension(Path::new(&path))
        .map_err(|e| format!("扩展验证失败: {}", e))?;

    // 添加扩展
    extension_manager
        .add(ext)
        .await
        .map_err(|e| e.to_string())?;

    // 保存配置
    extension_manager
        .save_config()
        .await
        .map_err(|e| e.to_string())?;

    println!("[COMMAND] Extension added successfully");
    Ok(())
}

#[tauri::command]
pub async fn toggle_extension(
    id: String,
    extension_manager: State<'_, ExtensionManager>,
) -> Result<(), String> {
    println!("[COMMAND] toggle_extension called with id: {}", id);

    // 切换扩展
    extension_manager
        .toggle(&id)
        .await
        .map_err(|e| e.to_string())?;

    // 保存配置
    extension_manager
        .save_config()
        .await
        .map_err(|e| e.to_string())?;

    println!("[COMMAND] Extension toggled successfully");
    Ok(())
}

#[tauri::command]
pub async fn remove_extension(
    id: String,
    extension_manager: State<'_, ExtensionManager>,
) -> Result<(), String> {
    println!("[COMMAND] remove_extension called with id: {}", id);

    // 删除扩展
    extension_manager
        .remove(&id)
        .await
        .map_err(|e| e.to_string())?;

    // 保存配置
    extension_manager
        .save_config()
        .await
        .map_err(|e| e.to_string())?;

    println!("[COMMAND] Extension removed successfully");
    Ok(())
}
```

#### 4. 集成到浏览器启动流程

**修改文件**: `src-tauri/src/targets/browser/mod.rs` 或 `src-tauri/src/targets/local/mod.rs`

```rust
// 在 launch() 方法中添加

pub async fn launch(&self, app_info: &AppInfo, options: LaunchOptions) -> Result<DebugConnection> {
    // ... 现有代码 ...

    // ⭐ 新增：加载扩展管理器并获取启动参数
    let extension_manager = ExtensionManager::new();
    extension_manager.load_config().await?;
    let extension_args = extension_manager.get_launch_args().await;

    let mut command = Command::new(&exe_path);

    // 基础参数
    command.args(&[
        &format!("--remote-debugging-port={}", chrome_port),
        &format!("--inspect={}", node_port),
        // ... 其他参数
    ]);

    // ⭐ 添加扩展参数
    for arg in extension_args {
        command.arg(arg);
    }

    // 启动应用
    let child = command.spawn()?;

    // ... 后续逻辑
}
```

#### 5. 前端扩展管理 UI

**新建文件**: `src/components/ExtensionManager.tsx`

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import { open } from '@tauri-apps/api/dialog';

interface ExtensionConfig {
  id: string;
  name: string;
  path: string;
  enabled: boolean;
  manifest_version: number;
}

export function ExtensionManager() {
  const [extensions, setExtensions] = useState<ExtensionConfig[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 加载扩展列表
  useEffect(() => {
    loadExtensions();
  }, []);

  const loadExtensions = async () => {
    try {
      setLoading(true);
      setError(null);
      const exts = await invoke<ExtensionConfig[]>('get_extensions');
      setExtensions(exts);
    } catch (err) {
      setError(`Failed to load extensions: ${err}`);
      console.error('Failed to load extensions:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAddExtension = async () => {
    try {
      setError(null);
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Chrome Extension Directory',
      });

      if (!selected || typeof selected !== 'string') {
        return;
      }

      await invoke('add_extension', { path: selected });
      await loadExtensions();
    } catch (err) {
      setError(`Failed to add extension: ${err}`);
      console.error('Failed to add extension:', err);
    }
  };

  const handleToggleExtension = async (id: string) => {
    try {
      setError(null);
      await invoke('toggle_extension', { id });
      await loadExtensions();
    } catch (err) {
      setError(`Failed to toggle extension: ${err}`);
      console.error('Failed to toggle extension:', err);
    }
  };

  const handleRemoveExtension = async (id: string) => {
    try {
      setError(null);
      await invoke('remove_extension', { id });
      await loadExtensions();
    } catch (err) {
      setError(`Failed to remove extension: ${err}`);
      console.error('Failed to remove extension:', err);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold mb-2">DevTools Extensions</h1>
        <p className="text-gray-600 dark:text-gray-400">
          Manage Chrome extensions that will be loaded into DevTools when debugging applications.
        </p>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-red-800 dark:text-red-200 text-sm">{error}</p>
        </div>
      )}

      <div className="mb-6">
        <button
          onClick={handleAddExtension}
          disabled={loading}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          + Add Extension
        </button>
      </div>

      {loading && extensions.length === 0 ? (
        <div className="text-center py-8 text-gray-500">Loading extensions...</div>
      ) : extensions.length === 0 ? (
        <div className="text-center py-8">
          <p className="text-gray-500 mb-4">No extensions added yet.</p>
          <p className="text-sm text-gray-400">
            Click "Add Extension" to load a Chrome extension directory.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {extensions.map((ext) => (
            <div
              key={ext.id}
              className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:shadow-md transition-shadow"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3 flex-1">
                  <input
                    type="checkbox"
                    checked={ext.enabled}
                    onChange={() => handleToggleExtension(ext.id)}
                    className="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <div className="flex-1">
                    <h3 className="font-semibold text-gray-900 dark:text-gray-100">
                      {ext.name}
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                      {ext.path}
                    </p>
                    <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                      Manifest v{ext.manifest_version}
                    </p>
                  </div>
                </div>
                <button
                  onClick={() => handleRemoveExtension(ext.id)}
                  className="ml-4 px-3 py-1 text-sm text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                >
                  Remove
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="mt-8 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
        <h3 className="font-semibold mb-2 text-gray-900 dark:text-gray-100">
          How to find Chrome extension paths:
        </h3>
        <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
          <li>
            <strong>Development Extensions:</strong> Point to the unpacked extension directory
            containing <code className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">manifest.json</code>
          </li>
          <li>
            <strong>Installed Extensions (Chrome):</strong>
            <ul className="ml-4 mt-1 space-y-1">
              <li>• macOS: <code className="text-xs bg-gray-200 dark:bg-gray-700 px-1 py-0.5 rounded">~/Library/Application Support/Google/Chrome/Default/Extensions/</code></li>
              <li>• Windows: <code className="text-xs bg-gray-200 dark:bg-gray-700 px-1 py-0.5 rounded">%LOCALAPPDATA%\Google\Chrome\User Data\Default\Extensions\</code></li>
              <li>• Linux: <code className="text-xs bg-gray-200 dark:bg-gray-700 px-1 py-0.5 rounded">~/.config/google-chrome/Default/Extensions/</code></li>
            </ul>
          </li>
          <li>
            <strong>Popular DevTools Extensions:</strong>
            <ul className="ml-4 mt-1 space-y-1">
              <li>• React Developer Tools</li>
              <li>• Redux DevTools</li>
              <li>• Vue.js DevTools</li>
              <li>• Apollo Client DevTools</li>
            </ul>
          </li>
        </ul>
      </div>

      <div className="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
        <p className="text-sm text-blue-800 dark:text-blue-200">
          <strong>Note:</strong> Extensions will be loaded when you start debugging an application.
          Changes take effect on the next debug session.
        </p>
      </div>
    </div>
  );
}
```

---

## 实现计划

### Week 1-2: 完整实现（9 天）

#### Phase 1: ExtensionManager 核心模块 (Rust) - 2 天

**任务**:
- [ ] 定义 `ExtensionConfig` 和 `ExtensionManager` 结构体
- [ ] 实现 `load_config()` 和 `save_config()`
- [ ] 实现 `validate_extension()` - 验证 `manifest.json`
- [ ] 实现 `add()`, `remove()`, `toggle()` 方法
- [ ] 实现 `get_launch_args()` - 生成 Chrome 参数
- [ ] 编写单元测试

**交付物**: `src-tauri/src/extensions/mod.rs` 完整实现

#### Phase 2: 扩展配置 JSON 读写逻辑 - 1 天

**任务**:
- [ ] 定义 `ExtensionsConfigFile` 结构体
- [ ] 实现配置文件序列化/反序列化
- [ ] 处理配置文件不存在的情况
- [ ] 添加错误处理和日志

**交付物**: JSON 配置文件读写功能完成

#### Phase 3: 集成扩展到浏览器启动流程 - 1 天

**任务**:
- [ ] 修改 `BrowserTargetAdapter::launch()` (如有)
- [ ] 修改 `LocalTargetAdapter::launch()` (Electron 调试)
- [ ] 在启动命令中添加扩展参数
- [ ] 测试扩展参数生成逻辑

**交付物**: 浏览器/Electron 启动时正确加载扩展

#### Phase 4: 实现前端扩展管理 UI 组件 - 2 天

**任务**:
- [ ] 创建 `ExtensionManager.tsx` 组件
- [ ] 实现扩展列表展示
- [ ] 实现添加扩展功能（文件夹选择对话框）
- [ ] 实现启用/禁用切换
- [ ] 实现删除扩展功能
- [ ] 添加帮助说明和错误提示
- [ ] 样式优化

**交付物**: 完整的扩展管理 UI

#### Phase 5: 实现 Tauri Commands (get/add/toggle/remove) - 1 天

**任务**:
- [ ] 在 `src-tauri/src/commands.rs` 中添加扩展命令
- [ ] 实现 `get_extensions` 命令
- [ ] 实现 `add_extension` 命令
- [ ] 实现 `toggle_extension` 命令
- [ ] 实现 `remove_extension` 命令
- [ ] 添加错误处理和日志

**交付物**: 4 个 Tauri Commands 完成

#### Phase 6: 注册 Commands 到 main.rs 和前端路由 - 1 天

**任务**:
- [ ] 在 `main.rs` 中注册 `ExtensionManager` 状态
- [ ] 注册扩展管理命令到 `invoke_handler`
- [ ] 在启动时加载扩展配置
- [ ] 在 `device-sidebar.tsx` 添加"扩展管理"导航入口
- [ ] 在 `App.tsx` 添加路由

**交付物**: 扩展管理功能集成到主应用

#### Phase 7: 测试扩展加载和 Chrome 137+ 兼容性 - 1 天

**任务**:
- [ ] 测试添加/删除/启用/禁用扩展
- [ ] 测试 React DevTools 加载
- [ ] 测试 Redux DevTools 加载
- [ ] 测试 Chrome 137+ 兼容性
- [ ] 跨平台测试（macOS/Windows/Linux）
- [ ] 修复发现的问题

**交付物**: 完整测试通过

---

## 关键技术挑战

### 挑战 1: Chrome 137+ 安全限制

**问题**: Chrome 137+ 默认禁用 `--load-extension` 命令行标志

**解决方案**: 使用兼容标志
```bash
--disable-features=DisableLoadExtensionCommandLineSwitch
```

**备选方案**: 如果未来 Chrome 完全移除此功能，可考虑：
1. 使用 Chrome Extension API (如果可用)
2. 引导用户手动安装扩展
3. 使用自定义 Chromium 构建

### 挑战 2: 跨平台路径分隔符

**问题**: Windows 使用 `;`，Unix 系统使用 `:` 作为路径分隔符

**解决方案**:
```rust
#[cfg(target_os = "windows")]
let separator = ";";
#[cfg(not(target_os = "windows"))]
let separator = ":";

let joined = enabled_paths.join(separator);
```

### 挑战 3: Manifest V3 兼容性

**问题**: 部分扩展已迁移到 Manifest V3，使用 Service Worker 而非 Background Page

**解决方案**:
- 在验证扩展时读取 `manifest_version` 字段
- 记录并提示用户
- Chromium/Electron 对 V3 的支持正在改进中

### 挑战 4: 扩展路径查找困难

**问题**: 用户可能不知道 Chrome 扩展安装在哪里

**解决方案**:
1. 提供详细的帮助文档（UI 中显示默认路径）
2. 使用文件夹选择对话框（`@tauri-apps/api/dialog`）
3. 未来可考虑"从 Chrome 导入"功能（自动扫描默认路径）

---

## 文件清单

### 新建文件

1. **`src-tauri/src/extensions/mod.rs`**
   - ExtensionManager 核心实现
   - ExtensionConfig 数据结构
   - 配置文件读写逻辑
   - Chrome 启动参数生成

2. **`src/components/ExtensionManager.tsx`**
   - 前端扩展管理 UI
   - 扩展列表展示
   - 添加/删除/启用/禁用功能
   - 帮助说明

3. **`~/.debugtron/extensions.json`**
   - 用户扩展配置文件（运行时生成）

### 修改文件

1. **`src-tauri/src/main.rs`**
   ```rust
   mod extensions;
   use extensions::ExtensionManager;

   fn main() {
       let extension_manager = ExtensionManager::new();

       tauri::Builder::default()
           .manage(extension_manager.clone())
           .setup(|app| {
               // 加载扩展配置
               let ext_mgr = extension_manager.clone();
               tauri::async_runtime::spawn(async move {
                   match ext_mgr.load_config().await {
                       Ok(_) => println!("[MAIN] Extension config loaded"),
                       Err(e) => eprintln!("[MAIN] Failed to load extensions: {}", e),
                   }
               });
               Ok(())
           })
           .invoke_handler(tauri::generate_handler![
               // ... 现有命令
               commands::get_extensions,
               commands::add_extension,
               commands::toggle_extension,
               commands::remove_extension,
           ])
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

2. **`src-tauri/src/commands.rs`**
   - 添加扩展管理命令（见上文）

3. **`src-tauri/src/targets/browser/mod.rs`** 或 **`src-tauri/src/targets/local/mod.rs`**
   - 在 `launch()` 方法中集成扩展启动参数

4. **`src-tauri/Cargo.toml`**
   ```toml
   [dependencies]
   uuid = { version = "1.0", features = ["v4"] }
   serde_json = "1.0"
   dirs = "5.0"
   ```

5. **`src/device-sidebar.tsx`**
   - 添加"扩展管理"导航入口

6. **`src/App.tsx`**
   - 添加 ExtensionManager 路由

---

## 测试计划

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path() {
        let manager = ExtensionManager::new();
        assert!(manager.config_path.ends_with(".debugtron/extensions.json"));
    }

    #[tokio::test]
    async fn test_add_and_remove() {
        let manager = ExtensionManager::new();

        let ext = ExtensionConfig {
            id: "test-ext".to_string(),
            name: "Test Extension".to_string(),
            path: PathBuf::from("/tmp/test-ext"),
            enabled: true,
            manifest_version: 3,
        };

        manager.add(ext.clone()).await.unwrap();

        let all = manager.get_all().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Test Extension");

        manager.remove("test-ext").await.unwrap();

        let all = manager.get_all().await.unwrap();
        assert_eq!(all.len(), 0);
    }

    #[tokio::test]
    async fn test_toggle() {
        let manager = ExtensionManager::new();

        let ext = ExtensionConfig {
            id: "test-ext".to_string(),
            name: "Test Extension".to_string(),
            path: PathBuf::from("/tmp/test-ext"),
            enabled: true,
            manifest_version: 3,
        };

        manager.add(ext).await.unwrap();

        manager.toggle("test-ext").await.unwrap();

        let all = manager.get_all().await.unwrap();
        assert_eq!(all[0].enabled, false);

        manager.toggle("test-ext").await.unwrap();

        let all = manager.get_all().await.unwrap();
        assert_eq!(all[0].enabled, true);
    }
}
```

### 集成测试

#### 测试场景 1: 添加 React DevTools 扩展

**步骤**:
1. 找到 React DevTools 路径：
   ```bash
   # macOS
   ~/Library/Application Support/Google/Chrome/Default/Extensions/fmkadmapgofadopljbjfkapdkoienihi/4.28.0_0
   ```
2. 在 Debugtron UI 中点击"添加扩展"
3. 选择扩展目录
4. 验证扩展出现在列表中
5. 启动调试会话
6. 打开 DevTools
7. 验证 "React" 面板可见

**预期结果**: ✅ React 面板正常显示

#### 测试场景 2: 禁用扩展

**步骤**:
1. 取消勾选 React DevTools
2. 启动新的调试会话
3. 打开 DevTools
4. 验证 "React" 面板不可见

**预期结果**: ✅ React 面板不显示

#### 测试场景 3: Chrome 137+ 兼容性

**步骤**:
1. 使用最新版 Chrome (137+)
2. 添加扩展
3. 启动调试
4. 检查 Chrome 启动参数包含 `--disable-features=DisableLoadExtensionCommandLineSwitch`

**预期结果**: ✅ 扩展正常加载

#### 测试场景 4: 跨平台测试

**步骤**:
- macOS: 测试完整流程
- Windows: 测试完整流程（注意路径分隔符）
- Linux: 测试完整流程

**预期结果**: ✅ 所有平台正常工作

---

## 用户文档

### 快速入门指南

#### 如何在 Debugtron 中使用 React DevTools

**步骤 1: 查找 React DevTools 路径**

**macOS**:
```bash
~/Library/Application Support/Google/Chrome/Default/Extensions/fmkadmapgofadopljbjfkapdkoienihi/4.28.0_0
```

**Windows**:
```bash
%LOCALAPPDATA%\Google\Chrome\User Data\Default\Extensions\fmkadmapgofadopljbjfkapdkoienihi\4.28.0_0
```

**Linux**:
```bash
~/.config/google-chrome/Default/Extensions/fmkadmapgofadopljbjfkapdkoienihi/4.28.0_0
```

**提示**: 在 Chrome 地址栏输入 `chrome://version` 可查看"配置文件路径"。扩展目录在该路径下的 `Extensions/` 文件夹中。

**步骤 2: 在 Debugtron 中添加扩展**

1. 打开 Debugtron
2. 点击侧边栏的"🧩 扩展管理"
3. 点击"+ Add Extension"按钮
4. 选择扩展目录（包含 `manifest.json` 的文件夹）
5. 点击"打开"

**步骤 3: 启动调试**

1. 返回应用列表
2. 选择一个 React 应用进行调试
3. 打开 DevTools
4. 在 DevTools 顶部导航栏中找到"React"面板

### 常见问题 (FAQ)

#### Q: 为什么我添加的扩展不生效？

**A**: 请检查以下项：
1. 路径是否包含版本号文件夹（如 `/path/to/extension/4.28.0_0`）
2. 目录下是否有 `manifest.json` 文件
3. 扩展是否已勾选"启用"
4. 尝试重启 Debugtron

#### Q: 支持哪些扩展？

**A**: 所有 Chrome/Chromium 扩展理论上都支持。常见的 DevTools 扩展：
- React Developer Tools
- Redux DevTools
- Vue.js devtools
- Apollo Client Devtools
- MobX Developer Tools

#### Q: 如何找到已安装的 Chrome 扩展？

**A**:
1. 打开 Chrome，访问 `chrome://extensions`
2. 开启"开发者模式"
3. 查看扩展 ID（如 `fmkadmapgofadopljbjfkapdkoienihi`）
4. 根据 ID 在上述默认路径中查找

#### Q: Manifest V3 扩展是否支持？

**A**: 是的，Debugtron 支持 Manifest V2 和 V3 扩展。扩展的 Manifest 版本会在 UI 中显示。

#### Q: 扩展配置文件在哪里？

**A**: `~/.debugtron/extensions.json`

---

## 成功指标

### 功能完成度

- [ ] 用户可以添加扩展（通过文件夹选择器）
- [ ] 用户可以删除扩展
- [ ] 用户可以启用/禁用扩展
- [ ] 扩展在浏览器/Electron 启动时正确加载
- [ ] React DevTools 等常用扩展可正常工作
- [ ] Chrome 137+ 兼容性验证通过
- [ ] 跨平台测试通过（macOS + Windows + Linux）

### 代码质量

- [ ] 单元测试覆盖率 > 80%
- [ ] 所有 Rust 代码通过 `cargo clippy`
- [ ] 所有 TypeScript 代码通过 ESLint

### 用户体验

- [ ] UI 响应流畅（< 100ms）
- [ ] 错误提示清晰
- [ ] 帮助文档完整易懂

---

## 依赖项

### Rust Crates

```toml
[dependencies]
uuid = { version = "1.0", features = ["v4"] }
serde_json = "1.0"
dirs = "5.0"
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
```

### TypeScript Packages

```json
{
  "dependencies": {
    "@tauri-apps/api": "^1.5.0"
  }
}
```

---

## 风险与缓解

| 风险 | 严重性 | 概率 | 缓解措施 |
|------|--------|------|---------|
| Chrome 未来完全禁用扩展加载 | 高 | 低 | 监控 Chromium 公告，准备备选方案 |
| Manifest V3 兼容性问题 | 中 | 中 | 测试主流扩展，记录兼容性 |
| 用户找不到扩展路径 | 中 | 高 | 提供详细文档和帮助说明 |
| 跨平台路径问题 | 低 | 低 | 充分测试，使用平台特定代码 |

---

## 后续优化

**Phase 8** (可选，延后实现):
- [ ] 自动扫描 Chrome 扩展目录
- [ ] "从 Chrome 导入"一键功能
- [ ] 扩展版本管理
- [ ] 扩展更新检测

---

**文档版本**: v1.0
**最后更新**: 2025-12-01
**负责人**: Claude Code
