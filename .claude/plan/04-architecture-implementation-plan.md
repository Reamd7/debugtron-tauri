# Debugtron Tauri 功能分层架构与实施计划

> **创建时间**: 2025-12-01
> **状态**: 已批准
> **版本**: v1.0

---

## 核心架构决策

### 1. 构建系统策略
- **决策**: 暂不独立化，直接在现有 GN 构建系统上二次开发 DevTools Frontend
- **路径**: `/Users/gemini/Documents/apifox/devtools-frontend/front_end`
- **理由**: 降低初期复杂度，快速产出功能

### 2. 功能分层原则

```
需要界面的功能     → DevTools Frontend 实现 (60%)
可纯 Rust 实现的   → Debugtron Tauri 实现 (30%)
混合功能           → 分层协同 (10%)
所有功能           → 设计为未来可通过 MCP 暴露给 AI
```

---

## 功能分层矩阵

| 功能 | 实现层 | 技术方案 | 工作量 | MCP 工具 |
|-----|-------|---------|-------|---------|
| **日志搜索/导出** | Rust/Tauri | 后端索引 + Tauri Command | 2-3 天 | `search_logs`, `export_logs` |
| **会话导出/导入** | Rust/Tauri | JSON Schema v1.0 序列化 | 2 天 | `export_session`, `import_session` |
| **预设启动配置** | Rust/Tauri | 本地 JSON 文件存储 | 2 天 | `save_launch_config` |
| **CDP WebSocket** | Rust/Tauri | `tokio-tungstenite` 客户端 | 3 天 | 基础设施 |
| **Runtime.evaluate** | 混合 | Rust CDP + 前端 REPL UI | 2 天 | `execute_code` |
| **console 分类** | 混合 | Rust 监听 + 前端 UI | 2 天 | `get_console_logs` |
| **IPC 消息追踪** ⭐ | 混合 | Rust Hook + DevTools Panel | 1 周 | `get_ipc_messages` |
| **网络请求追踪** | DevTools Frontend | 扩展 Network Panel | 1 周 | `get_network_requests` |
| **性能分析** | DevTools Frontend | 扩展 Performance Panel | 2 周 | `analyze_performance` |

---

## IPC 调试非侵入式方案 ⭐ 核心差异化

### 技术架构

```
Debugtron Tauri (启动时)
  ↓ --require 参数注入
Hook Script (ipc-hook-{session_id}.js)
  ↓ Proxy 拦截 ipcMain/ipcRenderer
IPC Debug WebSocket Server (Rust)
  ↓ 存储 + 转发
DevTools IPC Panel (新建面板)
```

### 实现步骤

#### Step 1: 启动时脚本注入 (Rust)

**文件**: `src-tauri/src/targets/local/mod.rs`

```rust
pub async fn launch(&self, app_info: &AppInfo, options: LaunchOptions) -> Result<DebugConnection> {
    // 生成 IPC Hook 脚本
    let hook_script_path = self.generate_ipc_hook_script(&conn_id)?;

    // 添加到启动参数
    command.arg("--require");
    command.arg(&hook_script_path);

    // ... 现有启动逻辑
}

fn generate_ipc_hook_script(&self, conn_id: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join("debugtron-hooks");
    fs::create_dir_all(&temp_dir)?;

    let script_path = temp_dir.join(format!("ipc-hook-{}.js", conn_id));
    let script_content = include_str!("../../../resources/ipc-hook-template.js")
        .replace("{{SESSION_ID}}", conn_id)
        .replace("{{WS_PORT}}", &self.ipc_websocket_port.to_string());

    fs::write(&script_path, script_content)?;
    Ok(script_path)
}
```

#### Step 2: Hook 脚本模板

**文件**: `src-tauri/resources/ipc-hook-template.js` (新建)

```javascript
const { ipcMain } = require('electron');
const WebSocket = require('ws');

const SESSION_ID = '{{SESSION_ID}}';
const WS_PORT = {{WS_PORT}};

let ws = new WebSocket(`ws://127.0.0.1:${WS_PORT}/ipc-debug`);

// Hook ipcMain.handle
const originalHandle = ipcMain.handle.bind(ipcMain);
ipcMain.handle = function(channel, listener) {
  return originalHandle(channel, async (event, ...args) => {
    const startTime = Date.now();

    // 发送到 Debugtron
    ws.send(JSON.stringify({
      sessionId: SESSION_ID,
      type: 'ipc-call',
      direction: 'renderer->main',
      channel,
      args: JSON.stringify(args),
      timestamp: Date.now(),
    }));

    try {
      const result = await listener(event, ...args);

      ws.send(JSON.stringify({
        sessionId: SESSION_ID,
        type: 'ipc-response',
        channel,
        result: JSON.stringify(result),
        duration: Date.now() - startTime,
        status: 'success',
      }));

      return result;
    } catch (error) {
      ws.send(JSON.stringify({
        sessionId: SESSION_ID,
        type: 'ipc-response',
        channel,
        error: error.message,
        duration: Date.now() - startTime,
        status: 'error',
      }));
      throw error;
    }
  });
};

// 同样 Hook ipcMain.on, ipcRenderer.send 等
```

#### Step 3: IPC WebSocket 服务器

**文件**: `src-tauri/src/ipc_debugger.rs` (新建)

```rust
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

pub struct IpcDebugServer {
    port: u16,
    messages: Arc<Mutex<HashMap<String, Vec<IpcMessage>>>>,
    app_handle: tauri::AppHandle,
}

impl IpcDebugServer {
    pub async fn start(app_handle: tauri::AppHandle) -> Result<Self> {
        let port = portpicker::pick_unused_port()
            .ok_or_else(|| anyhow::anyhow!("No port available"))?;

        let messages = Arc::new(Mutex::new(HashMap::new()));
        let messages_clone = messages.clone();
        let app_handle_clone = app_handle.clone();

        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
                .await.unwrap();

            while let Ok((stream, _)) = listener.accept().await {
                let messages = messages_clone.clone();
                let app_handle = app_handle_clone.clone();

                tokio::spawn(async move {
                    if let Ok(ws_stream) = accept_async(stream).await {
                        let (_, mut ws_receiver) = ws_stream.split();

                        while let Some(Ok(msg)) = ws_receiver.next().await {
                            if let Ok(text) = msg.to_text() {
                                if let Ok(ipc_msg) = serde_json::from_str::<IpcMessage>(text) {
                                    // 存储消息
                                    let session_id = ipc_msg.session_id.clone();
                                    messages.lock().await
                                        .entry(session_id)
                                        .or_insert_with(Vec::new)
                                        .push(ipc_msg.clone());

                                    // 转发到前端
                                    let _ = app_handle.emit_all("ipc-message", &ipc_msg);
                                }
                            }
                        }
                    }
                });
            }
        });

        Ok(Self { port, messages, app_handle })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcMessage {
    pub session_id: String,
    pub timestamp: u64,
    #[serde(rename = "type")]
    pub msg_type: String,  // "ipc-call" | "ipc-response"
    pub direction: String, // "renderer->main" | "main->renderer"
    pub channel: String,
    pub args: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub duration: Option<u64>,
    pub status: Option<String>,
}
```

#### Step 4: DevTools IPC Panel

**文件**: `devtools-frontend/front_end/panels/ipc_debugger/IpcDebuggerPanel.ts` (新建)

```typescript
import * as UI from '../../ui/legacy/legacy.js';
import * as DataGrid from '../../ui/legacy/components/data_grid/data_grid.js';

export class IpcDebuggerPanel extends UI.Panel.Panel {
  private messages: IpcMessage[] = [];
  private dataGrid: DataGrid.DataGridImpl<IpcMessage>;

  constructor() {
    super('ipc-debugger');

    // 创建工具栏
    const toolbar = new UI.Toolbar.Toolbar('ipc-toolbar', this.element);

    // 清空按钮
    const clearButton = new UI.Toolbar.ToolbarButton('Clear', 'clear');
    clearButton.addEventListener(UI.Toolbar.ToolbarButton.Events.Click, () => {
      this.messages = [];
      this.dataGrid.rootNode().removeChildren();
    });
    toolbar.appendToolbarItem(clearButton);

    // 创建消息表格
    this.dataGrid = new DataGrid.DataGridImpl({
      displayName: 'IPC Messages',
      columns: [
        {id: 'timestamp', title: 'Time', width: '120px'},
        {id: 'direction', title: 'Direction', width: '120px'},
        {id: 'channel', title: 'Channel', weight: 1},
        {id: 'duration', title: 'Duration (ms)', width: '100px'},
        {id: 'status', title: 'Status', width: '80px'},
      ]
    });

    this.dataGrid.show(this.element);

    // 监听 Tauri 事件
    if (window.__TAURI__) {
      window.__TAURI__.event.listen('ipc-message', (event: any) => {
        this.addMessage(event.payload);
      });
    }
  }

  private addMessage(message: IpcMessage): void {
    this.messages.push(message);

    const node = this.dataGrid.rootNode().appendChild(
      new DataGrid.DataGridNode({
        timestamp: new Date(message.timestamp).toLocaleTimeString(),
        direction: message.direction,
        channel: message.channel,
        duration: message.duration ? `${message.duration}ms` : '-',
        status: message.error ? '❌ Error' : '✅ OK',
      })
    );
  }
}
```

**文件**: `devtools-frontend/front_end/panels/ipc_debugger/ipc_debugger-meta.ts` (新建)

```typescript
import * as UI from '../../ui/legacy/legacy.js';

UI.ViewManager.registerViewExtension({
  location: UI.ViewManager.ViewLocationValues.PANEL,
  id: 'ipc-debugger',
  title: () => 'IPC Debugger',
  commandPrompt: () => 'Show IPC Debugger',
  order: 100,
  persistence: UI.ViewManager.ViewPersistence.CLOSEABLE,
  async loadView() {
    const IpcDebugger = await import('./ipc_debugger.js');
    return new IpcDebugger.IpcDebuggerPanel();
  },
});
```

---

## MCP 架构设计（面向 AI）

### 12 个核心 MCP 工具

| 工具名称 | 功能 | 输入 | 输出 | AI 场景 |
|---------|------|------|------|--------|
| `list_sessions` | 列出活跃会话 | - | Session[] | "显示正在调试的应用" |
| `search_logs` | 搜索日志 | session_id, query | LogEntry[] | "查找错误日志" |
| `export_logs` | 导出日志 | session_id, format | file_path | "导出为 JSON" |
| `export_session` | 导出会话 | session_id | file_path | "保存调试会话" |
| `import_session` | 导入会话 | path | Session | "加载会话" |
| `execute_code` | 执行代码 | session_id, code | result | "运行 console.log()" |
| `get_console_logs` | 获取控制台日志 | session_id, level? | ConsoleLog[] | "显示错误" |
| `get_ipc_messages` | 获取 IPC 消息 | session_id | IpcMessage[] | "显示 IPC 通信" |
| `analyze_ipc_flow` | 分析 IPC 流 | session_id | Analysis | "检测延迟" |
| `get_network_requests` | 获取网络请求 | session_id | Request[] | "显示失败请求" |
| `analyze_performance` | 性能分析 | session_id | Metrics | "检查内存" |
| `get_memory_snapshot` | 内存快照 | session_id | Snapshot | "对比内存" |

### MCP 服务器实现

**文件**: `src-tauri/src/mcp_server.rs` (新建)

```rust
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use serde::{Deserialize, Serialize};

pub struct McpServer {
    app_state: Arc<AppState>,
    handler: IoHandler,
}

impl McpServer {
    pub fn new(app_state: Arc<AppState>) -> Self {
        let mut handler = IoHandler::new();

        // 工具 1: list_sessions
        let state = app_state.clone();
        handler.add_method("list_sessions", move |_params: Params| {
            let sessions = state.sessions.blocking_lock();
            let list: Vec<_> = sessions.iter()
                .map(|(id, s)| json!({
                    "id": id,
                    "app_name": s.connection.app_id,
                    "active": s.active,
                }))
                .collect();
            Ok(Value::Array(list))
        });

        // 工具 2: search_logs
        let state = app_state.clone();
        handler.add_method("search_logs", move |params: Params| {
            let p: SearchLogsParams = params.parse()?;
            let logs = search_logs_impl(&state, &p)?;
            Ok(serde_json::to_value(logs)?)
        });

        // ... 注册其他 10 个工具

        Self { app_state, handler }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let server = ServerBuilder::new(self.handler.clone())
            .start_http(&format!("127.0.0.1:{}", port).parse()?)?;

        println!("[MCP] Server started on port {}", port);
        server.wait();
        Ok(())
    }
}

#[derive(Deserialize)]
struct SearchLogsParams {
    session_id: String,
    query: String,
}
```

**依赖添加**:

```toml
# src-tauri/Cargo.toml
[dependencies]
jsonrpc-core = "18.0"
jsonrpc-http-server = "18.0"
tokio-tungstenite = "0.20"
```

---

## 12 周实施路线图

### Week 1-2: MVP v1.1 - 日志和协作增强 ⭐

**目标**: 立即提升 30% 用户满意度

#### Week 1
- **Day 1-2**: 日志搜索功能
  - Rust 后端：内存索引 (HashMap)
  - Tauri Command: `search_logs(session_id, query, regex)`
  - 前端 UI：搜索框 + 结果高亮
- **Day 3**: 日志导出功能
  - 支持格式：.txt, .json, .csv
  - Tauri Command: `export_logs(session_id, format, path)`
- **Day 4-5**: 会话导出/导入
  - JSON Schema v1.0 定义
  - 序列化完整会话（日志、状态、配置）

#### Week 2
- **Day 1-2**: 快照对比
  - 保存会话快照
  - Diff 算法比较
- **Day 3-4**: 预设启动配置
  - 本地 JSON 配置文件
  - 环境变量、flags、路径管理
- **Day 5**: 应用收藏功能

**交付物**: 日志管理、会话管理、应用管理优化

---

### Week 3-5: MVP v1.2 - CDP 基础集成 ⭐

**目标**: 提供交互式调试能力

#### Week 3
- **Day 1-3**: WebSocket CDP 连接
  - 新建 `src-tauri/src/cdp/client.rs`
  - 使用 `tokio-tungstenite`
  - 连接管理、重连逻辑
- **Day 4-5**: CDP 类型定义
  - `src-tauri/src/cdp/types.rs`
  - Runtime, Debugger, Network 域

#### Week 4
- **Day 1-2**: Runtime.evaluate + REPL UI
  - CDP 命令封装
  - 前端代码输入框
- **Day 3-4**: console.log 分类
  - 监听 Runtime.consoleAPICalled
  - 按级别分类 (info, warn, error)
- **Day 5**: 错误处理和日志

#### Week 5
- **Day 1-3**: 实时性能指标
  - Memory, CPU 使用率
  - 前端图表展示
- **Day 4-5**: 日志分类视图优化

**交付物**: CDP 基础设施、代码执行、日志分类、性能监控

---

### Week 6-9: MVP v1.3 - IPC 调试 + 网络监控 ⭐⭐

**目标**: 建立市场差异化优势

#### Week 6-7: IPC 调试（详见上文）
- **Day 1-2**: IPC WebSocket 服务器
- **Day 3**: Hook 脚本注入
- **Day 4-5**: 消息存储和 API
- **Day 6-7**: DevTools IPC Panel
- **Day 8-10**: 消息流可视化（时序图）

#### Week 8-9: 网络监控
- **Day 1-3**: Network.enable 集成
- **Day 4-5**: HTTP 请求追踪
- **Day 6-10**: 请求查看器 + 瀑布图

**交付物**: IPC 调试（独有功能）、网络监控

---

### Week 10-12: Linux 平台 + MCP 集成

#### Week 10: Linux 支持
- **Day 1-2**: .desktop 文件扫描
- **Day 3-4**: Electron 检测和图标提取
- **Day 5**: 跨平台测试

#### Week 11-12: MCP 集成
- **Day 1-2**: MCP 服务器框架
- **Day 3-5**: 12 个工具实现
- **Day 6-10**: JSON-RPC 集成、文档、测试

**交付物**: 三大平台支持、MCP AI 工具集成

---

## 关键技术决策

### 决策 1: DevTools Frontend 策略

**扩展现有 Panel（而非创建全新 Panel）**

- 日志分类 → 扩展 Console Panel
- 网络监控 → 扩展 Network Panel
- 性能分析 → 扩展 Performance Panel
- **IPC 调试** → 创建新 IPC Debugger Panel（无现有工具）

### 决策 2: 数据存储策略

**混合策略**:
- 实时数据（会话、日志）→ 内存 HashMap
- 配置和导出 → JSON 文件
- 日志索引（可选）→ SQLite

### 决策 3: MCP 协议

**JSON-RPC 2.0** (MCP 官方标准)

---

## 关键文件清单

### 新建文件（Debugtron Tauri）

1. `src-tauri/src/cdp/client.rs` - CDP WebSocket 客户端
2. `src-tauri/src/cdp/types.rs` - CDP 类型定义
3. `src-tauri/src/ipc_debugger.rs` - IPC WebSocket 服务器
4. `src-tauri/src/mcp_server.rs` - MCP JSON-RPC 服务器
5. `src-tauri/resources/ipc-hook-template.js` - IPC Hook 脚本

### 新建文件（DevTools Frontend）

1. `front_end/panels/ipc_debugger/IpcDebuggerPanel.ts` - IPC 面板
2. `front_end/panels/ipc_debugger/ipc_debugger-meta.ts` - 面板注册
3. `front_end/panels/ipc_debugger/BUILD.gn` - 构建配置

### 修改文件

1. `src-tauri/src/targets/local/mod.rs` - 添加 IPC Hook 注入
2. `src-tauri/src/main.rs` - 启动 MCP 服务器
3. `src-tauri/Cargo.toml` - 添加依赖
4. `devtools-frontend/front_end/entrypoints/devtools_app/devtools_app.ts` - 注册 IPC 面板

---

## 成功指标

- **Week 2**: 日志搜索/导出功能可用，用户满意度 +30%
- **Week 5**: CDP 代码执行功能可用，交互式调试能力达成
- **Week 9**: IPC 调试功能可用，市场差异化优势建立
- **Week 12**: MCP 集成完成，AI 工具可调用所有 12 个功能
