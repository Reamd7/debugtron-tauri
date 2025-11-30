# Chrome DevTools Protocol 能力审计报告

> 探索 Debugtron 现有能力边界和 CDP 未利用功能

---

## 执行摘要

- **现有能力**：7 个核心功能已实现，覆盖基础调试需求 70%
- **CDP 利用率**：当前仅使用 `/json` 端点，未建立 WebSocket 连接
- **扩展潜力**：可集成 6 大 CDP domains，覆盖性能、网络、调试等高级功能
- **技术储备**：架构完全就绪，tokio 异步运行时支持 WebSocket

---

## 1. 现有能力清单

### ✅ 已实现的核心功能

#### 后端架构 (Rust/Tauri)

**1. 基础调试目标发现** (`src-tauri/src/state.rs:191-302`)
- 轮询 `/json` 端点获取调试目标列表
- 支持主进程 (node port) 和渲染进程 (renderer port)
- 渐进式轮询：100ms 基础间隔，自适应频率

**2. 应用启动和会话管理** (`src-tauri/src/targets/local/mod.rs:69-200`)
- 自动端口分配 (portpicker)
- 动态调试标志注入：`--inspect` 和 `--inspect-brk`
- 完整的 stdout/stderr 流捕获和转发
- 进程生命周期监控

**3. 本地 HTTP DevTools 服务器** (`src-tauri/src/devtools_server.rs`)
- 自动解压 Chrome DevTools 前端资源包
- 智能选择前端：`inspector.html`（渲染进程）vs `js_app.html`（主进程）

**4. 会话管理和事件系统**
- SessionState 结构体追踪：connection, active 状态, poll_count
- 7 个核心事件：session-added, session-removed, session-log, pages-updated

**当前 CDP 使用范围**：
- ✅ 仅使用 `/json` 端点
- ❌ 不建立 WebSocket 直接连接
- ❌ 完全依赖 Chrome DevTools 前端处理调试

---

## 2. CDP 未利用功能列表

### A. 实时调试 (Debugger Domain) ⭐⭐⭐⭐⭐

| 功能 | CDP 方法 | 优势 | 实现难度 |
|------|---------|------|---------|
| 断点管理 | `Debugger.setBreakpoint` | 自动化脚本调试 | ⭐⭐⭐ 高 |
| 执行控制 | `Debugger.stepOver/stepInto` | 无头调试工作流 | ⭐⭐⭐ 高 |
| 调用栈 | `Debugger.getStackTrace` | 结构化错误诊断 | ⭐⭐⭐ 高 |
| 表达式求值 | `Debugger.evaluateOnCallFrame` | 交互式变量检查 | ⭐⭐⭐ 高 |

**缺失痛点**：
- 当前完全依赖 Chrome UI，无法自动化调试
- 无法编程式获取调用栈（自动错误分析）
- 无法远程执行动态代码

---

### B. 性能分析 (Profiler + Performance) ⭐⭐⭐⭐⭐

| 功能 | CDP 方法 | 应用场景 | 实现难度 |
|------|---------|---------|---------|
| CPU 采样 | `Profiler.startPreciseCoverage` | 自动性能监控 | ⭐⭐⭐ 高 |
| 内存快照 | `HeapProfiler.takeHeapSnapshot` | 内存泄漏检测 | ⭐⭐⭐ 高 |
| 性能指标 | `Performance.getMetrics` | 实时指标仪表板 | ⭐⭐ 中 |
| 时间线 | `Tracing` domain | 可视化性能分析 | ⭐⭐⭐⭐ 很高 |

**当前状态**：
- 0% 性能数据采集
- 无法检测性能回归
- 无法识别内存泄漏

**集成价值**：
1. 自动生成性能报告
2. 与监控系统集成（Prometheus/ELK）
3. CI/CD 性能门限检查
4. 长运行应用的性能趋势追踪

---

### C. 网络监控 (Network + Fetch Domains) ⭐⭐⭐⭐⭐

| 功能 | CDP 方法 | 需求度 | 实现难度 |
|------|---------|--------|---------|
| 启用追踪 | `Network.enable` | 高 | ⭐ 低 |
| 请求拦截 | `Network.setRequestInterception` | 高 | ⭐⭐⭐ 高 |
| 响应修改 | `Fetch.fulfillRequest` | 高 | ⭐⭐⭐ 高 |
| 响应体获取 | `Network.getResponseBody` | 中 | ⭐⭐ 中 |

**未利用机会**：
- 无 API 调试面板
- 无网络瀑布图
- 无响应注入能力（mock/stub）
- 无带宽限流和延迟测试

---

### D. 控制台和代码执行 (Runtime Domain) ⭐⭐⭐⭐⭐

| 功能 | CDP 方法 | 优势 | 实现难度 |
|------|---------|------|---------|
| 代码执行 | `Runtime.evaluate` | 最高频需求 | ⭐⭐ 中 |
| 函数调用 | `Runtime.callFunctionOn` | 自动化测试 | ⭐⭐ 中 |
| 日志事件 | `Runtime.consoleAPICalled` | 日志分类 | ⭐⭐ 中 |

**当前限制**：
- 无法远程执行代码
- 日志仅有 stdout/stderr（无 console.log/warn/error 区分）
- 无对象属性检查能力

---

### E. 代码覆盖率 (Coverage Domain) ⭐⭐⭐

| 功能 | 用途 | 实现难度 |
|------|------|---------|
| JS 覆盖率 | `Coverage.startJSCoverage` | ⭐⭐⭐ 高 |
| CSS 覆盖率 | `Coverage.startCSSCoverage` | ⭐⭐⭐ 高 |
| 导出报告 | 生成 LCOV 格式 | ⭐⭐ 中 |

**缺失意义**：
- 无自动化测试覆盖率统计
- 无 Dead code 识别
- 无 CI/CD 门限集成

---

## 3. 技术可行性评估

### 简单实现 (1-2 天) ⭐

**1. WebSocket 连接管理**
```rust
依赖：tokio-tungstenite
工作量：4-6 小时
文件修改：
  - Cargo.toml
  - src-tauri/src/cdp/client.rs (新文件)
```

**2. Runtime.evaluate 支持**
```rust
前置条件：WebSocket 已实现
工作量：2-3 小时
文件修改：
  - src-tauri/src/commands.rs
  - src/api/tauri.ts
```

**3. 日志聚合（console.log）**
```rust
CDP 方法：Runtime.consoleAPICalled
工作量：3-4 小时
增强：区分 log/warn/error/info
```

---

### 中等实现 (3-5 天) ⭐⭐

**4. 性能采样和数据收集**
```
CDP 方法：Profiler.*, Performance.getMetrics
工作量：12-16 小时
UI 工作：可视化展示
```

**5. 网络请求拦截**
```
CDP 方法：Network.setRequestInterception, Fetch.fulfillRequest
工作量：16-20 小时
UI 工作：规则编辑器，请求查看器
```

**6. 性能时间线**
```
CDP 方法：Tracing domain
工作量：14-18 小时
UI 工作：甘特图可视化
```

---

### 复杂实现 (2-3 周) ⭐⭐⭐

**7. 完整调试器**
```
CDP 方法：Debugger.* (全集)
工作量：40-60 小时
参考：VS Code Debugger
包含：断点、调用栈、变量检查、Watch 表达式
```

**8. 性能自动化分析**
```
工作量：30-40 小时
包含：多次运行聚合、异常检测、告警、外部系统集成
```

---

## 4. 关键代码文件清单

### 需要修改的文件

| 文件 | 修改范围 | 优先级 |
|------|---------|--------|
| `src-tauri/Cargo.toml` | 添加 WebSocket 依赖 | P0 |
| `src-tauri/src/targets/types.rs` | 扩展 LaunchOptions, DebugConnection | P0 |
| `src-tauri/src/state.rs` | CDP 会话管理 | P0 |
| `src-tauri/src/commands.rs` | CDP 命令 (evaluate, setBreakpoint) | P0 |
| `src-tauri/src/targets/local/mod.rs` | 增强日志捕获 | P1 |
| `src/api/tauri.ts` | 扩展 CDP API 定义 | P0 |
| `src/session.tsx` | 添加高级调试 UI | P1 |

### 新增文件结构

```
src-tauri/src/cdp/
├── mod.rs                 # CDP 模块导出
├── client.rs             # WebSocket 客户端
├── types.rs              # CDP 类型定义
├── runtime.rs            # Runtime domain 实现
├── debugger.rs           # Debugger domain 实现
├── network.rs            # Network domain 实现
├── profiler.rs           # Profiler domain 实现
└── coverage.rs           # Coverage domain 实现

src/components/cdp/
├── Console.tsx           # 控制台 UI
├── Debugger.tsx          # 调试器 UI
├── Network.tsx           # 网络监控 UI
├── Performance.tsx       # 性能分析 UI
└── Coverage.tsx          # 覆盖率 UI
```

---

## 5. 实施优先级建议

### Phase 1: 基础设施 (Week 1-4)
- [ ] WebSocket 连接管理 + 消息编码/解码
- [ ] Runtime.evaluate 支持
- [ ] 基础 CDP 错误处理

### Phase 2: 功能扩展 (Week 5-12)
- [ ] 日志聚合（console API）
- [ ] 性能指标收集（Memory, CPU）
- [ ] 网络请求观察（无拦截）

### Phase 3: 高级功能 (Week 13-20)
- [ ] 网络请求拦截和修改
- [ ] 代码覆盖率统计
- [ ] 性能时间线可视化

### Phase 4: 完整化 (Week 21+)
- [ ] 完整调试器（可选，大量工程）
- [ ] 性能自动化分析
- [ ] AI 辅助诊断（可选）

---

## 6. 核心建议

**立即可做的** (1-2 周)
- WebSocket 连接 + Runtime.evaluate
- 日志聚合 (console.log 分类)
- 简单性能指标 (内存使用)

**短期高价值** (1-2 个月)
- 网络请求观察
- CPU 采样和性能时间线
- 代码覆盖率统计

**长期差异化** (2-3 个月+)
- 完整调试器（vs Chrome）
- 多应用性能对比
- 自动性能回归检测

**架构完全就绪**
- 事件驱动模式完美支持
- Tokio 异步运行时就位
- SessionState 易于扩展
