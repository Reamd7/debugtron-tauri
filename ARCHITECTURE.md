# Debugtron Tauri Architecture

> System architecture, technical stack, and core module design

## Overview

Debugtron Tauri is a cross-platform desktop application for debugging in-production Electron applications. It follows a client-server architecture where:

- **Frontend**: React-based UI that communicates with Rust backend via Tauri Commands and Events
- **Backend**: Rust layer handling platform-specific operations, process management, and DevTools serving
- **Communication**: Tauri's IPC bridge with type-safe commands and real-time events

## Technology Stack

### Frontend (Reused from original project)
- **Framework**: React 19 + TypeScript
- **State Management**: Redux Toolkit
- **UI Components**: Radix UI (accessible component library)
- **Styling**: TailwindCSS
- **Terminal**: xterm.js
- **Build Tool**: Vite

### Backend (Reimplemented in Rust)
- **Framework**: Tauri 1.5
- **Language**: Rust
- **Async Runtime**: tokio
- **Serialization**: serde
- **Error Handling**: anyhow
- **Port Allocation**: portpicker
- **HTTP Server**: axum + tower-http (for DevTools)
- **Browser Opening**: opener crate

### Platform-Specific Dependencies
- **macOS**: `plist` (Info.plist parsing)
- **Windows**: `winapi` (Windows API), `winreg` (registry), `walkdir` (directory traversal), `image` (icon encoding)

## Core Modules

### 1. Target Device Adapters (`src-tauri/src/targets/`)
Responsible for application discovery and debugging connections across different platforms and device types:

- **`LocalTargetAdapter`**: Local platform (macOS/Windows/Linux)
  - Scans system directories for Electron applications
  - Extracts application metadata and icons
  - Platform-specific implementations in `platforms/` subdirectory
- **`AdbTargetAdapter`**: Android remote debugging (optional, planned)

### 2. Tauri Commands (`src-tauri/src/commands.rs`)
Frontend-invoked Rust commands exposed via Tauri's IPC:

- `debug(app_info)` - Start a debugging session for an application
- `debug_path(path, options)` - Debug an application at custom path
- `get_targets()` - Get registered target devices
- `get_apps()` - Get discovered applications
- `open_devtools(url)` - Open DevTools in system browser
- `open_devtools_window(url)` - Open DevTools in Tauri window
- `refresh_device_apps(target_id)` - Refresh application list for a device

### 3. State Management (`src-tauri/src/state.rs`)
Global application state maintained in Rust:

- `targets`: HashMap of registered target devices
- `sessions`: HashMap of active debugging sessions (keyed by connectionId UUID)
- `apps`: HashMap of discovered applications per target
- `poll_trigger`: Arc<Notify> for immediate polling trigger
- `SessionState`: Tracks session-specific state (active flag, poll_count)

### 4. Frontend Components (`src/components/`)
- `App.tsx` - Main application container
- `DeviceSidebar.tsx` - Device list sidebar
- `DevicePanel.tsx` - Application list with discovery and debugging controls
- `Session.tsx` - Debugging session interface with tabs
- `Xterm.tsx` - Terminal component for log display

### 5. Event System (`src/hooks/useTauriEvents.ts`)
Real-time communication between Rust backend and React frontend:

- `target-registered` - New target device registered
- `apps-updated` - Application list updated
- `session-added` - New debugging session started
- `session-removed` - Session terminated
- `session-log` - Real-time stdout/stderr logs
- `pages-updated` - Debug targets (pages) discovered

## Data Flow

### Application Discovery
```
1. Frontend mounts → calls `get_targets()` and `get_apps()` commands
2. Backend initializes LocalTargetAdapter for current platform
3. Adapter scans system directories (macOS: /Applications, Windows: Program Files)
4. Detects Electron applications via framework presence or app.asar
5. Extracts metadata (name, version, icon) and sends `apps-updated` event
6. Frontend displays discovered applications
```

### Debug Session Lifecycle
```
1. User clicks "Debug" → frontend calls `debug(app_info)` command
2. Backend allocates ports (Node.js + Chrome DevTools) using portpicker
3. Spawns Electron app with --inspect/--inspect-brk and --remote-debugging-port
4. Creates session entry in state.sessions HashMap
5. Starts async tasks for:
   - Log capture (stdout/stderr streaming via `session-log` events)
   - Progressive polling for debug targets (100ms → 3s strategy)
   - Process exit monitoring
6. When debug targets discovered → sends `pages-updated` event
7. Frontend displays targets in process table
8. User clicks "Inspect" → opens DevTools in Tauri window or browser
9. On process exit → sends `session-removed` event → frontend cleans up
```

### Real-Time Log Streaming
```
Electron app stdout/stderr
  → tokio task async line reading
  → emit_all("session-log", {sessionId, type, message})
  → Frontend useTauriEvents hook
  → dispatch(sessionLogAppended)
  → Redux store update
  → Xterm component incremental append (content.slice(lastLength))
```

## Key Design Decisions

### 1. State Synchronization (vs electron-redux)
**Problem**: Electron used `electron-redux` for automatic main/renderer process state sync.

**Solution**: Tauri Events + Active Pull hybrid model
- Rust maintains single source of truth
- State changes emit events via `app.emit_all()`
- Frontend listens and dispatches Redux actions
- Frontend can actively pull initial data via `get_targets()`/`get_apps()`

### 2. Cross-Platform Icon Extraction
**macOS**: Manual ICNS format parsing to extract embedded PNG data
**Windows**: Three-tier strategy: file search → recursive search → PE resource extraction via Windows API

### 3. Progressive Polling Strategy
**Problem**: Fixed 3-second polling caused 1-3 second delay in debug target discovery.

**Solution**: Session state tracking + immediate trigger + adaptive intervals
- `poll_trigger: Arc<Notify>` for immediate polling on session start
- `SessionState.active` flag: new sessions poll every 100ms, active sessions every 3s
- Response time improved 10-30x (3s → 100ms)

### 4. Multi-Session Tab Synchronization
**Problem**: Array index matching caused tab content mismatch in multi-session debugging.

**Solution**: HashMap-based exact matching + consistent sessionId usage
- Backend sends `HashMap<String, Vec<PageInfo>>` instead of `Vec<Vec<PageInfo>>`
- Frontend uses `sessionId` keys for exact matching
- Unified `connectionId` (UUID) as session identifier across all events

### 5. Dual-Mode DevTools Opening
**Tauri Window Mode**: Integrated experience (1200x800 window, deduplication)
**Browser Mode**: Full browser functionality and extensions
- Window label sanitization: `.` → `_` to comply with Tauri naming rules

## Directory Structure

```
debugtron-tauri/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/        # React components
│   ├── store/            # Redux store and slices
│   ├── api/              # Tauri API type definitions
│   ├── hooks/            # Custom React hooks (useTauriEvents)
│   └── main.tsx          # Application entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs       # Tauri application setup
│   │   ├── commands.rs   # Tauri command handlers
│   │   ├── state.rs      # Global state management
│   │   ├── targets/      # Target adapter implementations
│   │   │   ├── local/mod.rs      # LocalTargetAdapter
│   │   │   ├── local/platforms/  # Platform-specific code
│   │   │   │   ├── macos.rs      # macOS implementation
│   │   │   │   ├── windows.rs    # Windows implementation
│   │   │   │   └── linux.rs      # Linux (planned)
│   │   │   └── types.rs          # Shared type definitions
│   │   └── devtools_server.rs    # Local DevTools HTTP server
│   ├── Cargo.toml        # Rust dependencies
│   └── tauri.conf.json   # Tauri configuration
├── scripts/              # Build and release scripts
└── docs/                # Documentation (this file and others)
```

## Related Documentation

- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - Development setup and workflows
- [DECISION_LOG.md](DECISION_LOG.md) - Detailed technical decision records
- [RELEASE_PROCESS.md](RELEASE_PROCESS.md) - Build and release procedures
- [TODO.md](TODO.md) - Pending tasks and roadmap
- [CLAUDE.md](CLAUDE.md) - Documentation index and project overview
- [migration_plan.md](migration_plan.md) - Original Electron to Tauri migration analysis

---

*Last Updated: 2025-12-02*