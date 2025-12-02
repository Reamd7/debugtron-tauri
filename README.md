# Debugtron Tauri

> Debug in-production Electron based App - Tauri Edition

A powerful desktop debugging tool for in-production Electron applications, rebuilt with Tauri + Rust for better performance, smaller bundle size, and cross-platform support.

## 🎉 Project Status

**Phase 1-8 Complete!** Windows and macOS platform support fully implemented.

### ✅ Current Features

- **Cross-platform Electron App Discovery**
  - **macOS**: Scans `/Applications` and `~/Applications`, detects Electron Framework, extracts ICNS icons
  - **Windows**: Scans Program Files directories, detects `resources/app.asar`, extracts PE resource icons
  - **Linux**: *Planned* (`.desktop` file scanning)

- **One-Click Debug Sessions**
  - Automatic port allocation using `portpicker`
  - Support for both `--inspect` (normal) and `--inspect-brk` (start paused) modes
  - Right-click context menu for advanced debugging options

- **Dual-Mode DevTools Integration**
  - **Tauri Window Mode**: Open DevTools in integrated Tauri window (1200x800, resizable)
  - **Browser Mode**: Open DevTools in system default browser for full extension support
  - Intelligent DevTools frontend selection (`inspector.html` for renderer, `js_app.html` for Node.js)

- **Real-Time Monitoring & Logging**
  - Live stdout/stderr streaming via Tauri Events
  - Xterm.js terminal with incremental log appending (optimized performance)
  - Process exit detection and automatic session cleanup

- **Advanced Features**
  - Progressive polling strategy (100ms → 3s) for instant debug target discovery
  - Multi-session debugging with proper tab synchronization
  - Local DevTools HTTP server using `axum` + `tower-http`
  - Custom path debugging via drag-and-drop (macOS)

### 🚀 User Workflow

1. **Launch** Debugtron Tauri application
2. **Discover** View automatically detected Electron apps in your system
3. **Start Debugging**:
   - *Left-click* → Normal debug mode (app starts normally)
   - *Right-click* → Advanced menu:
     - "Debug" - Normal mode
     - "Debug with --inspect-brk" - Start paused at first line
4. **Monitor** View real-time logs in the terminal panel
5. **Inspect** Open DevTools:
   - Blue "Inspect" button → Tauri window mode
   - Green "Open in Browser" button → System browser mode
6. **Debug** Directly debug the application using Chrome DevTools

## Development

### Prerequisites

- Node.js 22+
- Rust 1.70+
- Platform-specific requirements:
  - macOS: Xcode Command Line Tools
  - Windows: Microsoft C++ Build Tools
  - Linux: webkit2gtk, libssl-dev, build-essential

### Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

### Project Structure

```
debugtron-tauri/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/        # React components
│   ├── store/            # Redux store
│   ├── api/              # Tauri API wrappers
│   └── main.tsx          # Entry point
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── main.rs       # Tauri setup
│   │   ├── commands.rs   # Tauri commands
│   │   ├── state.rs      # App state management
│   │   └── targets/      # Platform adapters
│   └── Cargo.toml        # Rust dependencies
├── migration_plan.md      # Migration documentation
└── claude.md             # Project management
```

## 📚 Documentation

This project maintains comprehensive documentation split into focused files:

### Core Documentation
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture, technical stack, and core modules
- **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - Development setup, commands, Git workflow
- **[DECISION_LOG.md](DECISION_LOG.md)** - Technical decision records and implementation details
- **[RELEASE_PROCESS.md](RELEASE_PROCESS.md)** - Multi-platform build and release process
- **[TODO.md](TODO.md)** - Pending tasks, feature roadmap, and issue tracking

### Project History
- **[migration_plan.md](migration_plan.md)** - Original migration strategy from Electron to Tauri
- **[CLAUDE.md](CLAUDE.md)** - Project management and documentation index

### Feature Documentation
- **[LOG_FILTER_IMPLEMENTATION.md](LOG_FILTER_IMPLEMENTATION.md)** - Log filtering feature implementation
- **[LOG_FILTER_TESTING.md](LOG_FILTER_TESTING.md)** - Log filtering testing procedures

## Acknowledgments

This project is a complete rewrite from Electron to Tauri, with the backend reimplemented in Rust. Parts of the frontend code are based on the original [Debugtron](https://github.com/pd4d10/debugtron) project by [Rongjian Zhang](https://github.com/pd4d10).

**Key Differences:**
- **Backend**: Complete rewrite in Rust (originally Node.js/Electron)
- **Architecture**: Tauri instead of Electron
- **Bundle Size**: Significantly smaller (~30MB vs ~100MB)
- **Performance**: Better resource usage and startup time
- **Frontend**: Based on original React components with Tauri API adaptations

Special thanks to Rongjian Zhang for creating the original Debugtron project.

## License

MIT
