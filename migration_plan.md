# Debugtron Electron → Tauri 迁移分析文档

> **项目**: Debugtron - Debug in-production Electron based App
> **技术栈**: Electron → Tauri + Rust
> **分析日期**: 2025-11-26
> **分析工具**: Claude Code (Sonnet 4.5)

---

## 📋 目录

1. [项目概述](#项目概述)
2. [技术栈分析](#技术栈分析)
3. [项目结构分析](#项目结构分析)
4. [核心功能模块分析](#核心功能模块分析)
5. [Tauri 迁移重构计划](#tauri-迁移重构计划)
6. [实施步骤](#实施步骤)

---

## 项目概述

**Debugtron** 是一个用于调试生产环境 Electron 应用程序的强大桌面工具。它提供了以下核心功能：

### 核心特性
- 🔍 **自动应用发现**: 跨平台检测已安装的 Electron 应用程序
- 🚀 **一键调试会话**: 启动任何 Electron 应用并启用调试标志
- 🛠️ **DevTools 集成**: 访问 Node.js 主进程和渲染进程的 Chrome DevTools
- 📊 **实时监控**: 通过专业终端界面实时显示 stdout/stderr 日志

### 使用场景
- **开发与测试**: 调试生产构建、性能分析、功能验证
- **生产支持**: 调查已部署应用的问题，复现客户问题
- **质量保证**: 跨平台测试不带内置调试功能的应用

---

## 技术栈分析

### 前端技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| React | 19.2.0 | UI 框架 |
| TypeScript | 5.9.3 | 类型系统 |
| Redux Toolkit | 2.10.1 | 状态管理 |
| electron-redux | 2.0.0 | 主进程/渲染进程状态同步 |
| Radix UI | Latest | 无障碍 UI 组件库 |
| TailwindCSS | 4.1.16 | 样式框架 |
| xterm.js | 5.5.0 | 终端模拟器 |
| Vite | 7.1.12 | 构建工具 |

### 后端技术栈 (Electron 主进程)

| 依赖 | 版本 | 用途 |
|------|------|------|
| Electron | 39.1.0 | 桌面应用框架 |
| registry-js | 1.16.1 | Windows 注册表读取 |
| simple-plist | 1.3.1 | macOS plist 文件解析 |
| get-port | 7.1.0 | 动态端口分配 |
| node-machine-id | 1.1.12 | 机器标识 |
| uuid | 13.0.0 | UUID 生成 |
| universal-analytics | 0.5.3 | Google Analytics |

### 构建工具链

| 工具 | 版本 | 用途 |
|------|------|------|
| Electron Forge | 7.10.2 | 打包和分发 |
| Vite Plugin | 7.10.2 | Electron + Vite 集成 |
| Fuses Plugin | 7.10.2 | 安全配置 |

**打包格式支持**:
- macOS: ZIP
- Windows: Squirrel
- Linux: DEB, RPM

---

## 项目结构分析

### 目录树

```
debugtron/
├── src/
│   ├── main.ts                      # Electron 主进程入口
│   ├── preload.ts                   # IPC 桥接层 (contextBridge)
│   │
│   ├── main/                        # 主进程业务逻辑
│   │   ├── actions.ts              # Redux thunk actions
│   │   ├── store.ts                # 主进程 Redux store
│   │   ├── utils.ts                # 工具函数 (analytics, updater)
│   │   └── targets/                # 目标设备适配器模式
│   │       ├── registry.ts         # 设备注册中心
│   │       ├── types.ts            # TypeScript 接口定义
│   │       ├── local/              # 本地平台适配器
│   │       │   ├── adapter.ts      # LocalTargetAdapter 实现
│   │       │   └── platforms/      # 平台特定代码
│   │       │       ├── macos.ts    # macOS 应用发现
│   │       │       ├── win.ts      # Windows 应用发现
│   │       │       ├── linux.ts    # Linux 应用发现
│   │       │       └── utils.ts
│   │       └── remote/             # 远程设备适配器
│   │           └── adb.ts          # Android ADB 适配器
│   │
│   ├── renderer/                    # React 渲染进程
│   │   ├── index.tsx               # 应用入口
│   │   ├── app.tsx                 # 主应用组件
│   │   ├── session.tsx             # 调试会话界面
│   │   ├── device-panel.tsx        # 设备应用列表面板
│   │   ├── device-sidebar.tsx      # 设备侧边栏
│   │   ├── device-details-dialog.tsx # 设备详情对话框
│   │   ├── xterm.tsx               # 终端组件封装
│   │   ├── store.ts                # 渲染进程 Redux store
│   │   └── lib/
│   │       └── utils.ts            # 工具函数 (cn, clsx)
│   │
│   ├── reducers/                    # Redux state slices
│   │   ├── app.ts                  # AppInfo 状态管理
│   │   ├── session.ts              # SessionInfo 状态管理
│   │   └── target.ts               # TargetInfo 状态管理
│   │
│   └── types/                       # 全局类型定义
│       └── electron.d.ts
│
├── assets/                          # 静态资源
│   └── icon.png                    # 应用图标
│
├── forge.config.ts                  # Electron Forge 配置
├── vite.main.config.ts             # Vite 主进程配置
├── vite.preload.config.ts          # Vite preload 配置
├── vite.renderer.config.ts         # Vite 渲染进程配置
├── tsconfig.json                    # TypeScript 配置
├── eslint.config.js                # ESLint 配置
├── postcss.config.js               # PostCSS 配置
└── package.json                    # NPM 依赖配置
```

### 关键配置文件

#### package.json 核心脚本
```json
{
  "scripts": {
    "start": "electron-forge start",     // 开发模式
    "make": "electron-forge make",       // 打包
    "package": "electron-forge package", // 仅打包不制作安装包
    "publish": "electron-forge publish"  // 发布到 GitHub Releases
  }
}
```

#### forge.config.ts 结构
```typescript
{
  packagerConfig: {
    asar: true,  // 启用 ASAR 打包
  },
  makers: [
    MakerSquirrel,  // Windows
    MakerZIP,       // macOS
    MakerRpm,       // Linux RPM
    MakerDeb,       // Linux DEB
  ],
  plugins: [
    VitePlugin,     // Vite 集成
    FusesPlugin,    // 安全配置
  ]
}
```

---

## 核心功能模块分析

### 1. IPC 通信机制

#### Preload 暴露的 API (`src/preload.ts`)

```typescript
// 通过 contextBridge 暴露安全的 API
window.debugtronAPI = {
  // 调试操作
  debug: (appInfo: AppInfo) => void,
  debugPath: (path: string) => void,

  // 远程设备管理
  addRemoteDevice: (options: RemoteDeviceOptions) => void,
  removeDevice: (targetId: string) => void,
  refreshDeviceApps: (targetId: string) => void,

  // DevTools 操作
  openDevTools: (url: string) => void,

  // 事件监听
  onSessionUpdate: (callback: Function) => UnsubscribeFn,
}
```

#### 主进程监听的 IPC 事件 (`src/main.ts`)

```typescript
ipcMain.on('debug', (e, appInfo: AppInfo) => { ... })
ipcMain.on('debug-path', (_, path: string) => { ... })
ipcMain.on('add-remote-device', (_, options: RemoteDeviceOptions) => { ... })
ipcMain.on('remove-device', (_, targetId: string) => { ... })
ipcMain.on('refresh-device-apps', (_, targetId: string) => { ... })
ipcMain.on('open-devtools', (_, url: string) => { ... })
```

---

### 2. 主进程核心逻辑 (`src/main/actions.ts`)

#### init() - 初始化流程
```typescript
export const init: ThunkActionCreator = () => async (dispatch, getState) => {
  // 1. 初始化本地目标设备 (macOS/Windows/Linux)
  await targetRegistry.initializeLocalTargets();

  // 2. 注册所有目标到 Redux store
  const targets = targetRegistry.getAllInfo();
  targets.forEach(target => {
    dispatch(targetSlice.actions.registered(target));
  });

  // 3. 发现所有平台的 Electron 应用
  const allApps: AppInfo[] = [];
  for (const target of targetRegistry.getAll()) {
    const appsResult = await target.discoverApps();
    if (appsResult.ok) {
      allApps.push(...appsResult.val);
    }
  }
  dispatch(appSlice.actions.found(allApps));

  // 4. 启动轮询定时器 (每 3 秒)
  setInterval(() => {
    // 轮询所有调试会话的 /json 端点
    // 更新 DevTools 页面列表
  }, 3000);
};
```

#### debug() - 启动调试会话
```typescript
export const debug: ThunkActionCreator<AppInfo> = (app) => async (dispatch) => {
  // 1. 获取目标适配器
  const target = targetRegistry.getById(app.targetId);

  // 2. 启动应用 (分配调试端口)
  const connectionResult = await target.launch(app, {});
  const connection = connectionResult.val;

  // 3. 添加会话到 Redux store
  dispatch(sessionSlice.actions.added({
    sessionId: connection.connectionId,
    appId: app.id,
    targetId: app.targetId,
    connection: {
      type: 'local-process',
      nodePort: connection.debugPorts.node,
      windowPort: connection.debugPorts.renderer,
    },
  }));

  // 4. 监听进程事件
  const sp = connection.processHandle;
  sp.on('error', (err) => { dialog.showErrorBox(...) });
  sp.on('close', () => { dispatch(sessionSlice.actions.removed(...)) });

  // 5. 捕获日志输出
  sp.stdout?.on('data', (chunk) => {
    dispatch(sessionSlice.actions.logAppended({
      sessionId,
      content: chunk.toString(),
    }));
  });
};
```

---

### 3. 目标设备适配器模式

#### 接口定义 (`src/main/targets/types.ts`)

```typescript
interface TargetAdapter {
  // 元数据
  type: 'local' | 'remote';
  id: string;        // 例: 'local-macos', 'remote-adb-192.168.1.100'
  name: string;      // 例: 'Local (macOS)', 'Android TV (192.168.1.100)'

  // 核心方法
  discoverApps(): Promise<Result<AppInfo[], Error>>;
  launch(app: AppInfo, options: LaunchOptions): Promise<Result<DebugConnection, Error>>;
  disconnect(connectionId: string): Promise<void>;
  isAvailable(): Promise<boolean>;
}

interface DebugConnection {
  connectionId: string;
  debugPorts: {
    node?: number;       // Node.js inspector port
    renderer?: number;   // Chromium remote debugging port
    websocket?: string;  // WebSocket URL (远程设备)
  };
  processHandle?: ChildProcess;  // 仅本地进程
  cleanup: () => Promise<void>;
}
```

#### LocalTargetAdapter 实现 (`src/main/targets/local/adapter.ts`)

```typescript
class LocalTargetAdapter implements TargetAdapter {
  async discoverApps(): Promise<Result<AppInfo[], Error>> {
    // 1. 根据平台导入对应的适配器
    const { adapter } = await importByPlatform();

    // 2. 读取所有 Electron 应用
    const apps = await adapter.readAll();

    // 3. 添加目标元数据
    return apps.map(app => ({
      ...app,
      id: `${this.id}:${app.id}`,
      targetId: this.id,
      targetType: 'local',
      metadata: { platform: process.platform },
    }));
  }

  async launch(app: AppInfo, options: LaunchOptions): Promise<Result<DebugConnection, Error>> {
    // 1. 动态分配调试端口
    const nodePort = await getPort();
    const windowPort = await getPort();

    // 2. 启动子进程
    const sp = spawn(app.exePath, [
      `--inspect=${nodePort}`,
      `--remote-debugging-port=${windowPort}`,
      '--remote-allow-origins=devtools://devtools',
    ]);

    // 3. 返回连接对象
    return {
      connectionId: uuid(),
      debugPorts: { node: nodePort, renderer: windowPort },
      processHandle: sp,
      cleanup: () => { sp.kill(); },
    };
  }
}
```

#### 平台特定实现

##### macOS (`src/main/targets/local/platforms/macos.ts`)
```typescript
async function readAll(): Promise<AppInfo[]> {
  const appDirs = [
    '/Applications',
    path.join(os.homedir(), 'Applications'),
  ];

  const apps: AppInfo[] = [];
  for (const dir of appDirs) {
    const entries = await fs.readdir(dir);
    for (const entry of entries.filter(e => e.endsWith('.app'))) {
      const plistPath = path.join(dir, entry, 'Contents/Info.plist');
      const plist = await readPlist(plistPath);

      // 检查是否为 Electron 应用
      if (isElectronApp(plist)) {
        apps.push({
          id: plist.CFBundleIdentifier,
          name: plist.CFBundleDisplayName || entry.replace('.app', ''),
          icon: await extractIcon(path.join(dir, entry)),
          exePath: path.join(dir, entry, 'Contents/MacOS', plist.CFBundleExecutable),
        });
      }
    }
  }
  return apps;
}
```

##### Windows (`src/main/targets/local/platforms/win.ts`)
```typescript
async function readAll(): Promise<AppInfo[]> {
  const apps: AppInfo[] = [];

  // 1. 从注册表读取 Uninstall 信息
  const uninstallKey = 'SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall';
  const entries = await registryjs.enumerateValues(HKLM, uninstallKey);

  for (const entry of entries) {
    const displayName = entry.DisplayName;
    const installLocation = entry.InstallLocation;

    if (installLocation && await isElectronApp(installLocation)) {
      apps.push({
        id: entry.Publisher + '.' + displayName,
        name: displayName,
        icon: await extractIcon(entry.DisplayIcon),
        exePath: path.join(installLocation, findExe(installLocation)),
      });
    }
  }

  // 2. 扫描常见安装目录
  const commonDirs = [
    'C:\\Program Files',
    'C:\\Program Files (x86)',
    path.join(os.homedir(), 'AppData/Local/Programs'),
  ];
  // ... 文件系统扫描逻辑

  return apps;
}
```

##### Linux (`src/main/targets/local/platforms/linux.ts`)
```typescript
async function readAll(): Promise<AppInfo[]> {
  const desktopDirs = [
    '/usr/share/applications',
    path.join(os.homedir(), '.local/share/applications'),
  ];

  const apps: AppInfo[] = [];
  for (const dir of desktopDirs) {
    const entries = await fs.readdir(dir);
    for (const entry of entries.filter(e => e.endsWith('.desktop'))) {
      const desktopFile = await parseDesktopFile(path.join(dir, entry));

      if (await isElectronApp(desktopFile.Exec)) {
        apps.push({
          id: desktopFile.StartupWMClass || entry,
          name: desktopFile.Name,
          icon: desktopFile.Icon,
          exePath: extractExePath(desktopFile.Exec),
        });
      }
    }
  }
  return apps;
}
```

---

### 4. 调试会话管理

#### 端口分配策略
```typescript
// 使用 get-port 动态分配可用端口
const nodePort = await getPort();      // Node.js inspector (例: 9229)
const windowPort = await getPort();    // Chromium DevTools (例: 9222)

// 启动参数
const debugFlags = [
  `--inspect=${nodePort}`,                          // 主进程调试
  `--remote-debugging-port=${windowPort}`,          // 渲染进程调试
  '--remote-allow-origins=devtools://devtools',     // 允许 DevTools 连接
];
```

#### 页面发现轮询 (每 3 秒)
```typescript
setInterval(async () => {
  const { session } = getState();
  const sessions = Object.values(session);

  // 收集所有调试端口
  const ports = [];
  sessions.forEach(s => {
    if (s.connection.type === 'local-process') {
      ports.push(s.connection.nodePort, s.connection.windowPort);
    }
  });

  // 并发请求所有 /json 端点
  const responses = await Promise.allSettled(
    ports.map(port =>
      fetch(`http://127.0.0.1:${port}/json`).then(res => res.json())
    )
  );

  // 更新页面信息
  dispatch(sessionSlice.actions.pageUpdated(responses));
}, 3000);
```

#### DevTools URL 转换
```typescript
// 原始 URL (from /json endpoint)
// "/devtools/inspector.html?ws=127.0.0.1:9229/..."

// 转换后 (Electron BrowserWindow 可加载)
// "devtools://devtools/bundled/inspector.html?ws=127.0.0.1:9229/..."

function transformDevToolsUrl(url: string): string {
  return url
    .replace(/^\/devtools/, 'devtools://devtools/bundled')
    .replace(/^chrome-devtools:\/\//, 'devtools://');
}
```

---

### 5. Redux 状态管理

#### State 结构

```typescript
// 全局 State 类型
interface RootState {
  app: Record<string, AppInfo>;        // 已发现的应用
  session: Record<string, SessionInfo>; // 活跃的调试会话
  target: Record<string, TargetInfo>;   // 已连接的设备
}

// AppInfo (应用信息)
interface AppInfo {
  id: string;           // 'local-macos:com.example.app'
  name: string;         // 'My Electron App'
  icon: string;         // Base64 图标数据
  exePath?: string;     // '/Applications/MyApp.app/Contents/MacOS/MyApp'
  targetId: string;     // 'local-macos'
  targetType: 'local' | 'remote';
  metadata?: {
    platform?: 'darwin' | 'win32' | 'linux';
    deviceInfo?: string;
    packageName?: string;
  };
}

// SessionInfo (调试会话)
interface SessionInfo {
  appId: string;
  targetId: string;
  page: Record<string, PageInfo>;  // DevTools 页面列表
  log: string;                     // 累积的日志输出
  connection: {
    type: 'local-process' | 'remote-adb' | 'remote-websocket';
    nodePort?: number;
    windowPort?: number;
    websocketUrl?: string;
  };
}

// TargetInfo (目标设备)
interface TargetInfo {
  id: string;
  type: 'local' | 'remote';
  name: string;
  status: 'connected' | 'disconnected' | 'connecting';
  lastDiscovery?: number;
}
```

#### electron-redux 同步机制

```typescript
// 主进程 (src/main/store.ts)
import { forwardToRenderer, replayActionMain, getInitialStateRenderer } from 'electron-redux';

const store = configureStore({
  reducer: rootReducer,
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware().concat(forwardToRenderer),
});

replayActionMain(store);

// 渲染进程 (src/renderer/store.ts)
import { forwardToMain, replayActionRenderer, getInitialStateRenderer } from 'electron-redux/renderer';

const store = configureStore({
  reducer: rootReducer,
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware().concat(forwardToMain),
  preloadedState: getInitialStateRenderer(),
});

replayActionRenderer(store);
```

**工作原理**:
1. 主进程 dispatch action → `forwardToRenderer` 拦截 → 通过 IPC 发送到所有渲染进程
2. 渲染进程 dispatch action → `forwardToMain` 拦截 → 通过 IPC 发送到主进程
3. 主进程接收后 replay action，状态更新后再次广播

---

### 6. 前端组件架构

#### 组件层次结构

```
App.tsx (主容器)
├── DeviceSidebar                 # 左侧设备列表
│   ├── LocalDevice               # 本地设备项
│   └── RemoteDevice[]            # 远程设备项
│
├── DevicePanel                   # 中间应用列表
│   └── AppCard[]                 # 应用卡片
│
├── Session (仅在有会话时显示)    # 右侧调试会话
│   ├── Tabs.Root                 # 多会话标签页
│   │   ├── Tabs.List             # 标签头
│   │   └── Tabs.Content[]        # 标签内容
│   │       ├── ConnectionInfo    # 连接信息
│   │       ├── ProcessTable      # 进程/页面列表
│   │       └── Xterm             # 日志终端
│   └── ...
│
├── DeviceDetailsDialog           # 设备详情弹窗
└── AddRemoteDeviceDialog         # 添加远程设备弹窗
```

#### 核心交互流程

```typescript
// 1. 用户点击应用卡片
<AppCard onClick={() => {
  window.debugtronAPI.debug(appInfo);
}} />

// 2. 主进程处理 debug 请求
ipcMain.on('debug', (e, appInfo) => {
  store.dispatch(debug(appInfo));
});

// 3. Redux action 启动应用并创建会话
async function debug(appInfo) {
  const connection = await target.launch(appInfo);
  dispatch(sessionSlice.actions.added({
    sessionId: connection.connectionId,
    // ...
  }));
}

// 4. electron-redux 同步状态到渲染进程

// 5. Session 组件自动显示
{Object.keys(sessionStore).length > 0 && <Session />}

// 6. 轮询更新 DevTools 页面列表
setInterval(() => {
  fetch(`http://127.0.0.1:${port}/json`)
    .then(res => res.json())
    .then(pages => dispatch(sessionSlice.actions.pageUpdated(pages)));
}, 3000);

// 7. 用户点击 Inspect 按钮
<button onClick={() => {
  window.debugtronAPI.openDevTools(page.devtoolsFrontendUrl);
}}>
  Inspect
</button>
```

---

## Tauri 迁移重构计划

### 架构映射表

| Electron 组件 | Tauri 对应方案 | 迁移策略 |
|--------------|---------------|----------|
| **主进程** (main.ts, main/*) | **Rust 后端** (src-tauri/src/) | 完全重写为 Rust |
| **Preload 脚本** (preload.ts) | **Tauri Commands** | 用 `#[tauri::command]` 替代 |
| **IPC (ipcMain/ipcRenderer)** | **Tauri Commands + Events** | 1:1 映射到 Tauri API |
| **BrowserWindow** | **Tauri Window** | 配置文件 `tauri.conf.json` |
| **Redux Store 同步 (electron-redux)** | **Tauri State + Events** | Rust Mutex + Event Emitter |
| **child_process (Node.js)** | **std::process::Command (Rust)** | 原生替代 |
| **Node.js 依赖** | **Rust Crates** | 逐个查找替代品 |

---

### Rust Crates 依赖映射

| 功能 | Electron 依赖 | Tauri Rust Crate | 备注 |
|------|--------------|------------------|------|
| Windows 注册表 | `registry-js` | `winreg` | 官方推荐 |
| macOS Plist | `simple-plist` | `plist` | 成熟稳定 |
| 端口分配 | `get-port` (npm) | `portpicker` | 简单易用 |
| 进程管理 | `child_process` | `tokio::process` | 异步支持 |
| UUID 生成 | `uuid` (npm) | `uuid` | 同名 crate |
| 文件系统遍历 | `fs` + `readdir` | `walkdir` | 递归遍历 |
| 错误处理 | `ts-results` | `anyhow` / `thiserror` | 惯用模式 |
| JSON 序列化 | `JSON.stringify/parse` | `serde_json` | 高性能 |
| 状态管理 | `electron-redux` | `tauri::State` + `tokio::sync::Mutex` | 内置支持 |
| HTTP 请求 | `fetch` (Node.js) | `reqwest` | 异步 HTTP 客户端 |
| 定时器 | `setInterval` | `tokio::time::interval` | 异步定时器 |

---

### 核心模块 Rust 重构设计

#### 1. 目标设备适配器 Trait

```rust
// src-tauri/src/targets/mod.rs

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub exe_path: Option<String>,
    pub target_id: String,
    pub target_type: TargetType,
    pub metadata: Option<AppMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConnection {
    pub connection_id: String,
    pub debug_ports: DebugPorts,
    pub process_handle: Option<u32>,  // PID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugPorts {
    pub node: Option<u16>,
    pub renderer: Option<u16>,
    pub websocket: Option<String>,
}

#[async_trait]
pub trait TargetAdapter: Send + Sync {
    fn get_type(&self) -> TargetType;
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;

    async fn discover_apps(&self) -> Result<Vec<AppInfo>>;
    async fn launch(&self, app: &AppInfo, options: LaunchOptions) -> Result<DebugConnection>;
    async fn disconnect(&self, connection_id: &str) -> Result<()>;
    async fn is_available(&self) -> bool;
}
```

#### 2. 本地平台适配器实现

```rust
// src-tauri/src/targets/local/adapter.rs

use super::TargetAdapter;
use std::process::Stdio;
use tokio::process::Command;

pub struct LocalTargetAdapter {
    id: String,
    name: String,
    platform: String,
}

impl LocalTargetAdapter {
    pub fn new() -> Self {
        let platform = std::env::consts::OS;
        Self {
            id: format!("local-{}", platform),
            name: format!("Local ({})", Self::platform_name(platform)),
            platform: platform.to_string(),
        }
    }

    fn platform_name(os: &str) -> &str {
        match os {
            "macos" => "macOS",
            "windows" => "Windows",
            "linux" => "Linux",
            _ => os,
        }
    }
}

#[async_trait]
impl TargetAdapter for LocalTargetAdapter {
    fn get_id(&self) -> String { self.id.clone() }
    fn get_name(&self) -> String { self.name.clone() }

    async fn discover_apps(&self) -> Result<Vec<AppInfo>> {
        match self.platform.as_str() {
            "macos" => platforms::macos::discover_apps(&self.id).await,
            "windows" => platforms::windows::discover_apps(&self.id).await,
            "linux" => platforms::linux::discover_apps(&self.id).await,
            _ => Err(anyhow::anyhow!("Unsupported platform")),
        }
    }

    async fn launch(&self, app: &AppInfo, _options: LaunchOptions) -> Result<DebugConnection> {
        let exe_path = app.exe_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No executable path"))?;

        // 动态分配端口
        let node_port = portpicker::pick_unused_port()
            .ok_or_else(|| anyhow::anyhow!("No available port"))?;
        let renderer_port = portpicker::pick_unused_port()
            .ok_or_else(|| anyhow::anyhow!("No available port"))?;

        // 启动子进程
        let mut child = Command::new(exe_path)
            .arg(format!("--inspect={}", node_port))
            .arg(format!("--remote-debugging-port={}", renderer_port))
            .arg("--remote-allow-origins=devtools://devtools")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let connection_id = uuid::Uuid::new_v4().to_string();

        // 启动日志读取任务
        let pid = child.id().unwrap();
        tokio::spawn(async move {
            // 读取 stdout/stderr 并发送到前端
            // ...
        });

        Ok(DebugConnection {
            connection_id,
            debug_ports: DebugPorts {
                node: Some(node_port),
                renderer: Some(renderer_port),
                websocket: None,
            },
            process_handle: Some(pid),
        })
    }
}
```

#### 3. macOS 应用发现

```rust
// src-tauri/src/targets/local/platforms/macos.rs

use plist::Value;
use std::path::PathBuf;
use walkdir::WalkDir;

pub async fn discover_apps(target_id: &str) -> Result<Vec<AppInfo>> {
    let app_dirs = vec![
        PathBuf::from("/Applications"),
        dirs::home_dir().unwrap().join("Applications"),
    ];

    let mut apps = Vec::new();

    for dir in app_dirs {
        if !dir.exists() { continue; }

        for entry in WalkDir::new(&dir).max_depth(1) {
            let entry = entry?;
            let path = entry.path();

            if !path.extension().map_or(false, |e| e == "app") {
                continue;
            }

            let info_plist_path = path.join("Contents/Info.plist");
            if !info_plist_path.exists() { continue; }

            // 解析 plist
            let plist = Value::from_file(&info_plist_path)?;
            let dict = plist.as_dictionary().unwrap();

            // 检查是否为 Electron 应用
            if is_electron_app(dict) {
                let bundle_id = dict.get("CFBundleIdentifier")
                    .and_then(|v| v.as_string())
                    .unwrap_or("unknown");

                let display_name = dict.get("CFBundleDisplayName")
                    .or(dict.get("CFBundleName"))
                    .and_then(|v| v.as_string())
                    .unwrap_or_else(|| path.file_stem().unwrap().to_str().unwrap());

                let executable = dict.get("CFBundleExecutable")
                    .and_then(|v| v.as_string())
                    .unwrap();

                apps.push(AppInfo {
                    id: format!("{}:{}", target_id, bundle_id),
                    name: display_name.to_string(),
                    icon: extract_icon(path).await?,
                    exe_path: Some(path.join("Contents/MacOS").join(executable).to_string_lossy().to_string()),
                    target_id: target_id.to_string(),
                    target_type: TargetType::Local,
                    metadata: Some(AppMetadata {
                        platform: Some("macos".to_string()),
                        ..Default::default()
                    }),
                });
            }
        }
    }

    Ok(apps)
}

fn is_electron_app(dict: &plist::Dictionary) -> bool {
    // 检查 LSApplicationCategoryType 或其他特征
    // 简化实现: 检查是否包含 "Electron" 相关字段
    dict.get("ElectronVersion").is_some() ||
    dict.get("CFBundleExecutable").map_or(false, |v| {
        v.as_string().unwrap_or("").contains("Electron")
    })
}
```

#### 4. Tauri Commands

```rust
// src-tauri/src/commands.rs

use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub async fn debug(
    app_info: AppInfo,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut targets = state.targets.lock().await;
    let target = targets.get(&app_info.target_id)
        .ok_or("Target not found")?;

    let connection = target.launch(&app_info, LaunchOptions::default())
        .await
        .map_err(|e| e.to_string())?;

    // 保存到会话列表
    let mut sessions = state.sessions.lock().await;
    sessions.insert(connection.connection_id.clone(), Session {
        app_id: app_info.id.clone(),
        target_id: app_info.target_id.clone(),
        connection: connection.clone(),
        pages: HashMap::new(),
        log: String::new(),
    });

    // 发送事件到前端
    state.app_handle.emit_all("session-added", &connection)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn add_remote_device(
    options: RemoteDeviceOptions,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 实现 ADB 设备连接
    // ...
}

#[tauri::command]
pub async fn refresh_device_apps(
    target_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let targets = state.targets.lock().await;
    let target = targets.get(&target_id)
        .ok_or("Target not found")?;

    let apps = target.discover_apps()
        .await
        .map_err(|e| e.to_string())?;

    // 发送到前端
    state.app_handle.emit_all("apps-updated", &apps)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn open_devtools(url: String) -> Result<(), String> {
    use tauri::WindowBuilder;

    WindowBuilder::new(
        &app_handle,
        "devtools",
        tauri::WindowUrl::External(url.parse().unwrap())
    )
    .title("DevTools")
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

#### 5. 全局状态管理

```rust
// src-tauri/src/state.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::AppHandle;

pub struct AppState {
    pub targets: Arc<Mutex<HashMap<String, Box<dyn TargetAdapter>>>>,
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
    pub apps: Arc<Mutex<HashMap<String, AppInfo>>>,
    pub app_handle: AppHandle,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            targets: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            apps: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // 注册本地适配器
        let local_adapter = LocalTargetAdapter::new();
        let target_id = local_adapter.get_id();

        let mut targets = self.targets.lock().await;
        targets.insert(target_id.clone(), Box::new(local_adapter));
        drop(targets);

        // 发现应用
        self.refresh_apps(&target_id).await?;

        Ok(())
    }

    pub async fn refresh_apps(&self, target_id: &str) -> Result<()> {
        let targets = self.targets.lock().await;
        let target = targets.get(target_id)
            .ok_or_else(|| anyhow::anyhow!("Target not found"))?;

        let apps = target.discover_apps().await?;

        // 更新全局状态
        let mut apps_state = self.apps.lock().await;
        for app in &apps {
            apps_state.insert(app.id.clone(), app.clone());
        }
        drop(apps_state);

        // 发送到前端
        self.app_handle.emit_all("apps-updated", &apps)?;

        Ok(())
    }
}
```

#### 6. 轮询任务 (Tokio)

```rust
// src-tauri/src/polling.rs

use tokio::time::{interval, Duration};

pub async fn start_polling(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(3));

    loop {
        ticker.tick().await;

        let sessions = state.sessions.lock().await;
        for (session_id, session) in sessions.iter() {
            if let Some(node_port) = session.connection.debug_ports.node {
                // 请求 /json 端点
                if let Ok(response) = reqwest::get(format!("http://127.0.0.1:{}/json", node_port)).await {
                    if let Ok(pages) = response.json::<Vec<PageInfo>>().await {
                        // 发送到前端
                        let _ = state.app_handle.emit_all("pages-updated", &pages);
                    }
                }
            }
        }
    }
}
```

---

### 前端代码调整

#### 1. 移除 electron-redux

**原实现**:
```typescript
// electron-redux 自动同步主进程/渲染进程状态
import { forwardToMain, replayActionRenderer } from 'electron-redux/renderer';
```

**新实现**:
```typescript
// 使用 Tauri Events 手动同步
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useDispatch } from 'react-redux';

function useAppSync() {
  const dispatch = useDispatch();

  useEffect(() => {
    // 监听应用列表更新
    const unlistenApps = listen('apps-updated', (event) => {
      dispatch(appSlice.actions.found(event.payload));
    });

    // 监听会话日志追加
    const unlistenLogs = listen('session-log-appended', (event) => {
      dispatch(sessionSlice.actions.logAppended(event.payload));
    });

    // 监听页面更新
    const unlistenPages = listen('pages-updated', (event) => {
      dispatch(sessionSlice.actions.pageUpdated(event.payload));
    });

    return () => {
      unlistenApps.then(f => f());
      unlistenLogs.then(f => f());
      unlistenPages.then(f => f());
    };
  }, [dispatch]);
}
```

#### 2. 替换 IPC 调用

**原实现**:
```typescript
// Electron preload API
window.debugtronAPI.debug(appInfo);
window.debugtronAPI.openDevTools(url);
```

**新实现**:
```typescript
// Tauri invoke
import { invoke } from '@tauri-apps/api/tauri';

await invoke('debug', { appInfo });
await invoke('open_devtools', { url });
```

**完整替换映射**:
```typescript
// src/api/tauri.ts
import { invoke } from '@tauri-apps/api/tauri';

export const debugtronAPI = {
  debug: (appInfo: AppInfo) => invoke('debug', { appInfo }),
  debugPath: (path: string) => invoke('debug_path', { path }),
  addRemoteDevice: (options: RemoteDeviceOptions) => invoke('add_remote_device', { options }),
  removeDevice: (targetId: string) => invoke('remove_device', { targetId }),
  refreshDeviceApps: (targetId: string) => invoke('refresh_device_apps', { targetId }),
  openDevTools: (url: string) => invoke('open_devtools', { url }),
};
```

#### 3. Window API 调整

**原实现** (Electron):
```typescript
// src/main.ts
const mainWindow = new BrowserWindow({
  width: 1024,
  height: 768,
  titleBarStyle: "hidden",
  trafficLightPosition: { x: 14, y: 14 },
});
```

**新实现** (Tauri):
```json
// tauri.conf.json
{
  "tauri": {
    "windows": [{
      "title": "Debugtron",
      "width": 1024,
      "height": 768,
      "decorations": false,
      "titleBarStyle": "Overlay"
    }]
  }
}
```

```typescript
// src/components/TitleBar.tsx (自定义标题栏)
import { appWindow } from '@tauri-apps/api/window';

function TitleBar() {
  return (
    <div
      data-tauri-drag-region
      className="h-10 flex items-center justify-center bg-background"
    >
      <span>Debugtron</span>
      <div className="ml-auto flex">
        <button onClick={() => appWindow.minimize()}>−</button>
        <button onClick={() => appWindow.toggleMaximize()}>□</button>
        <button onClick={() => appWindow.close()}>×</button>
      </div>
    </div>
  );
}
```

---

### Tauri 项目目录结构

```
debugtron-tauri/
├── src/                             # 前端代码 (React + TypeScript)
│   ├── components/                 # React 组件
│   │   ├── App.tsx
│   │   ├── Session.tsx
│   │   ├── DevicePanel.tsx
│   │   ├── DeviceSidebar.tsx
│   │   ├── DeviceDetailsDialog.tsx
│   │   └── Xterm.tsx
│   ├── store/                      # Redux store (简化版)
│   │   ├── index.ts
│   │   ├── slices/
│   │   │   ├── appSlice.ts
│   │   │   ├── sessionSlice.ts
│   │   │   └── targetSlice.ts
│   ├── api/                        # Tauri API 封装
│   │   └── tauri.ts
│   ├── hooks/                      # React Hooks
│   │   └── useAppSync.ts
│   ├── types/                      # TypeScript 类型
│   │   └── index.ts
│   ├── main.tsx                    # 应用入口
│   └── styles.css                  # 全局样式
│
├── src-tauri/                       # Rust 后端
│   ├── Cargo.toml                  # Rust 依赖配置
│   ├── tauri.conf.json             # Tauri 配置
│   ├── build.rs                    # 构建脚本
│   ├── icons/                      # 应用图标
│   │   ├── icon.icns               # macOS
│   │   ├── icon.ico                # Windows
│   │   └── icon.png                # Linux
│   └── src/
│       ├── main.rs                 # Rust 入口 + Tauri setup
│       ├── commands.rs             # Tauri commands 实现
│       ├── state.rs                # 全局状态管理
│       ├── polling.rs              # 轮询任务
│       ├── targets/                # 目标设备适配器
│       │   ├── mod.rs
│       │   ├── types.rs            # Trait 和类型定义
│       │   ├── registry.rs         # 设备注册中心
│       │   ├── local/
│       │   │   ├── mod.rs
│       │   │   ├── adapter.rs      # LocalTargetAdapter
│       │   │   └── platforms/
│       │   │       ├── mod.rs
│       │   │       ├── macos.rs
│       │   │       ├── windows.rs
│       │   │       └── linux.rs
│       │   └── remote/
│       │       ├── mod.rs
│       │       └── adb.rs          # ADB 适配器
│       └── utils/                  # 工具函数
│           ├── icon.rs             # 图标提取
│           └── process.rs          # 进程管理
│
├── public/                          # 静态资源
│   └── assets/
│
├── package.json                    # 前端依赖 (npm/yarn/pnpm)
├── vite.config.ts                  # Vite 配置
├── tsconfig.json                   # TypeScript 配置
├── tailwind.config.js              # TailwindCSS 配置
└── README.md
```

---

## 实施步骤

### 阶段 1: 初始化 Tauri 项目 (1 天)

#### 1.1 安装 Tauri CLI
```bash
# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Tauri CLI
cargo install tauri-cli
```

#### 1.2 创建项目骨架
```bash
cd debugtron-tauri
npm create tauri-app@latest . -- --template react-ts
```

#### 1.3 配置 Vite + React + TypeScript
```bash
npm install
npm install @reduxjs/toolkit react-redux
npm install @radix-ui/react-dialog @radix-ui/react-radio-group @radix-ui/react-tabs @radix-ui/react-tooltip
npm install @xterm/xterm @xterm/addon-canvas @xterm/addon-fit
npm install tailwindcss postcss autoprefixer
npm install clsx tailwind-merge
npm install @tauri-apps/api @tauri-apps/cli
```

#### 1.4 复制静态资源
```bash
cp -r ../debugtron/assets ./public/
cp ../debugtron/src/renderer/app.css ./src/
```

---

### 阶段 2: Rust 后端核心模块 (3-5 天)

#### 2.1 实现 TargetAdapter Trait (0.5 天)
- [ ] 定义 `TargetAdapter` trait (`src-tauri/src/targets/types.rs`)
- [ ] 定义 `AppInfo`, `DebugConnection`, `LaunchOptions` 等类型
- [ ] 实现 `TargetRegistry` 注册中心

#### 2.2 实现本地平台适配器 (2 天)

**macOS** (0.5 天):
- [ ] 使用 `plist` crate 解析 Info.plist
- [ ] 遍历 `/Applications` 和 `~/Applications`
- [ ] 实现 `is_electron_app()` 检测逻辑
- [ ] 实现图标提取 (ICNS → PNG → Base64)

**Windows** (1 天):
- [ ] 使用 `winreg` crate 读取注册表
- [ ] 扫描 `Program Files` 等目录
- [ ] 实现 Electron 特征检测
- [ ] 实现图标提取 (ICO → PNG → Base64)

**Linux** (0.5 天):
- [ ] 解析 `.desktop` 文件
- [ ] 扫描 `/usr/share/applications` 等目录
- [ ] 实现 Electron 检测 (通过 `ldd` 或文件特征)

#### 2.3 实现进程管理和端口分配 (0.5 天)
- [ ] 使用 `portpicker` crate 分配调试端口
- [ ] 使用 `tokio::process::Command` 启动子进程
- [ ] 实现 stdout/stderr 日志捕获
- [ ] 实现进程生命周期管理

#### 2.4 实现状态管理 (0.5 天)
- [ ] 定义 `AppState` 结构体
- [ ] 使用 `Arc<Mutex<HashMap>>` 管理 targets/sessions/apps
- [ ] 实现状态初始化逻辑
- [ ] 实现状态更新和事件发射

#### 2.5 实现轮询任务 (0.5 天)
- [ ] 使用 `tokio::time::interval` 实现定时器
- [ ] 使用 `reqwest` 请求 `/json` 端点
- [ ] 解析 DevTools Protocol 响应
- [ ] 通过 Tauri Events 发送到前端

---

### 阶段 3: Tauri Commands (1 天)

#### 3.1 实现核心命令
- [ ] `debug(app_info)` - 启动调试会话
- [ ] `debug_path(path)` - 从路径启动
- [ ] `add_remote_device(options)` - 添加远程设备
- [ ] `remove_device(target_id)` - 移除设备
- [ ] `refresh_device_apps(target_id)` - 刷新应用列表
- [ ] `open_devtools(url)` - 打开 DevTools 窗口

#### 3.2 实现事件发射
- [ ] `apps-updated` - 应用列表更新
- [ ] `session-added` - 新会话创建
- [ ] `session-removed` - 会话关闭
- [ ] `session-log-appended` - 日志追加
- [ ] `pages-updated` - DevTools 页面更新

#### 3.3 错误处理
- [ ] 统一错误类型 (`anyhow::Error`)
- [ ] 实现错误转换为字符串返回给前端
- [ ] 添加日志记录 (`tracing` crate)

---

### 阶段 4: 前端迁移 (2-3 天)

#### 4.1 复制并调整组件 (1 天)
- [ ] 复制 `src/renderer/*` → `src/components/`
- [ ] 移除 `electron-redux` 导入
- [ ] 替换 `window.debugtronAPI` 为 `invoke()`
- [ ] 调整样式类名 (保持 TailwindCSS)

#### 4.2 实现 Tauri 同步机制 (1 天)
- [ ] 创建 `useAppSync` hook
- [ ] 监听 Tauri Events (`apps-updated`, `session-log-appended` 等)
- [ ] 手动 dispatch Redux actions
- [ ] 测试状态同步正确性

#### 4.3 调整窗口和系统交互 (0.5 天)
- [ ] 实现自定义标题栏组件
- [ ] 使用 `data-tauri-drag-region` 支持窗口拖拽
- [ ] 调整菜单栏 (使用 Tauri 菜单 API)

#### 4.4 调整 Vite 配置 (0.5 天)
- [ ] 配置 `@tauri-apps/cli` Vite 插件
- [ ] 调整构建输出路径
- [ ] 配置开发服务器端口

---

### 阶段 5: 远程设备支持 (2-3 天) [可选]

#### 5.1 实现 ADB 适配器 (2 天)
- [ ] 使用 `std::process::Command` 调用 `adb` 命令
- [ ] 实现设备连接检测 (`adb connect`)
- [ ] 实现应用列表获取 (`adb shell pm list packages`)
- [ ] 实现应用启动和端口转发 (`adb forward`)
- [ ] 实现 WebSocket 调试连接

#### 5.2 实现远程设备 UI (1 天)
- [ ] 添加设备对话框已实现，确保后端集成
- [ ] 实现设备状态显示
- [ ] 实现设备断开逻辑

---

### 阶段 6: 测试和优化 (2-3 天)

#### 6.1 功能测试 (1 天)
- [ ] macOS 平台应用发现测试
- [ ] Windows 平台应用发现测试
- [ ] Linux 平台应用发现测试
- [ ] 调试会话启动测试
- [ ] DevTools 打开测试
- [ ] 日志输出测试
- [ ] 多会话并发测试

#### 6.2 性能优化 (0.5 天)
- [ ] 优化应用发现速度 (并发扫描)
- [ ] 优化状态同步延迟
- [ ] 优化日志输出性能 (批量发送)
- [ ] 优化轮询频率

#### 6.3 错误处理完善 (0.5 天)
- [ ] 添加错误提示对话框
- [ ] 处理进程崩溃情况
- [ ] 处理端口占用情况
- [ ] 处理权限不足情况

#### 6.4 打包和分发 (1 天)
- [ ] 配置 `tauri.conf.json` 打包选项
- [ ] 生成 macOS DMG
- [ ] 生成 Windows MSI
- [ ] 生成 Linux AppImage/DEB/RPM
- [ ] 配置 GitHub Actions 自动构建
- [ ] 测试安装包

---

## 技术挑战和解决方案

### 挑战 1: Electron 检测逻辑

**问题**: 如何准确识别一个应用是否为 Electron 应用？

**解决方案**:
- **macOS**: 检查 `Info.plist` 中是否包含 `ElectronVersion` 字段，或检查可执行文件依赖
- **Windows**: 检查应用目录中是否存在 `resources/electron.asar` 或 `electron.exe`
- **Linux**: 使用 `ldd` 检查可执行文件是否链接 `libnode.so`

### 挑战 2: 图标提取

**问题**: 不同平台图标格式不同 (ICNS, ICO, PNG)。

**解决方案**:
- **macOS**: 使用 `icns` crate 解析 ICNS 文件，提取最大尺寸图标
- **Windows**: 使用 `ico` crate 解析 ICO 文件
- **Linux**: 直接读取 PNG/SVG 文件
- 统一转换为 Base64 PNG 返回给前端

### 挑战 3: Redux 状态同步

**问题**: Electron 的 `electron-redux` 提供自动同步，Tauri 需要手动实现。

**解决方案**:
- 使用 Tauri Events 替代 IPC
- 在 Rust 端维护单一状态源
- 前端通过 Events 被动接收更新
- 避免双向同步冲突

### 挑战 4: 进程日志捕获

**问题**: 如何高效捕获子进程 stdout/stderr 并实时推送到前端？

**解决方案**:
```rust
let mut stdout = child.stdout.take().unwrap();
let mut stderr = child.stderr.take().unwrap();

tokio::spawn(async move {
    let mut buf = [0u8; 4096];
    loop {
        match stdout.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                let content = String::from_utf8_lossy(&buf[..n]);
                app_handle.emit_all("session-log-appended", &LogPayload {
                    session_id: session_id.clone(),
                    content: content.to_string(),
                }).unwrap();
            },
            Err(_) => break,
        }
    }
});
```

### 挑战 5: DevTools URL 处理

**问题**: Tauri Window 是否支持加载 `devtools://` 协议？

**解决方案**:
- 如果不支持，使用 `chrome://inspect` 或自定义 WebSocket 客户端
- 或者使用系统默认浏览器打开 DevTools URL

---

## 依赖清单

### Rust Crates (Cargo.toml)

```toml
[dependencies]
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4"] }
reqwest = { version = "0.11", features = ["json"] }
async-trait = "0.1"
portpicker = "0.1"

# Platform-specific
[target.'cfg(target_os = "macos")'.dependencies]
plist = "1.5"
icns = "0.3"

[target.'cfg(target_os = "windows")'.dependencies]
winreg = "0.51"
ico = "0.3"

[target.'cfg(target_os = "linux")'.dependencies]
walkdir = "2"
freedesktop_entry_parser = "1.3"
```

### NPM Packages (package.json)

```json
{
  "dependencies": {
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "@reduxjs/toolkit": "^2.10.1",
    "react-redux": "^9.2.0",
    "@radix-ui/react-dialog": "^1.1.15",
    "@radix-ui/react-radio-group": "^1.3.8",
    "@radix-ui/react-tabs": "^1.1.13",
    "@radix-ui/react-tooltip": "^1.2.8",
    "@xterm/xterm": "^5.5.0",
    "@xterm/addon-canvas": "^0.7.0",
    "@xterm/addon-fit": "^0.10.0",
    "clsx": "^2.1.1",
    "tailwind-merge": "^3.3.1",
    "@tauri-apps/api": "^1.5.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^1.5.0",
    "@vitejs/plugin-react": "^4.0.0",
    "typescript": "^5.9.3",
    "vite": "^7.1.12",
    "tailwindcss": "^4.1.16",
    "autoprefixer": "^10.4.21",
    "postcss": "^8.5.6"
  }
}
```

---

## 预期改进

### 性能提升
- **启动速度**: Rust 编译后的二进制文件启动更快
- **内存占用**: 无 Node.js 运行时，内存占用显著降低
- **包体积**: Tauri 应用比 Electron 小 50-80%

### 安全性
- **沙盒隔离**: Tauri 默认更严格的权限控制
- **代码混淆**: Rust 编译后的代码更难逆向
- **攻击面减少**: 无 Node.js 集成，减少潜在漏洞

### 跨平台一致性
- Rust 生态的跨平台库通常比 Node.js 更稳定
- 统一的系统 API 调用方式

---

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| **学习曲线** | 中 | 逐步迁移，保留原项目作为参考 |
| **平台兼容性** | 高 | 在所有平台上进行充分测试 |
| **依赖缺失** | 中 | 提前调研 Rust crate 替代方案 |
| **DevTools 集成** | 高 | 测试 Tauri Window 对 `devtools://` 支持 |
| **状态同步复杂度** | 中 | 使用 Tauri Events 简化设计 |
| **时间成本** | 中 | 预计 2-3 周完成基础迁移 |

---

## 总结

本文档详细分析了 Debugtron 项目的技术架构，并提出了完整的 Tauri 迁移方案。迁移的核心目标是：

1. **保留前端代码**: 复用 React + Redux + TailwindCSS 界面
2. **重写后端逻辑**: 用 Rust 替代 Node.js 主进程
3. **简化架构**: 移除 `electron-redux`，改用 Tauri Events
4. **提升性能**: 利用 Rust 的编译优势和更小的运行时

通过分阶段实施，预计 2-3 周可完成基础迁移，后续可逐步完善远程设备调试等高级功能。

---

**下一步行动**:
- [ ] 确认迁移方案细节
- [ ] 初始化 Tauri 项目
- [ ] 开始 Rust 后端开发

**文档维护**:
- 本文档将随迁移进度持续更新
- 所有设计决策和技术选型都会记录在此
