# Debugtron Tauri

> Debug in-production Electron based App - Tauri Edition

A powerful desktop debugging tool for in-production Electron applications, rebuilt with Tauri for better performance and smaller bundle size.

## Features

- 🔍 **Automatic App Discovery**: Cross-platform detection of installed Electron applications
- 🚀 **One-Click Debug Sessions**: Launch any Electron app with debugging flags enabled
- 🛠️ **DevTools Integration**: Access Chrome DevTools for both Node.js main process and renderer processes
- 📊 **Real-Time Monitoring**: Live stdout/stderr logging with professional terminal interface

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

## Documentation

- [Migration Plan](migration_plan.md) - Detailed migration strategy from Electron to Tauri
- [Project Tasks](claude.md) - Development progress and task management

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
