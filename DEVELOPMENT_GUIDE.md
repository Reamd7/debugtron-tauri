# Debugtron Tauri Development Guide

> Development environment setup, commands, workflows, and best practices

## Development Environment Setup

### Prerequisites

- **Node.js** 22+ (with npm)
- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **Tauri CLI**: `cargo install tauri-cli`

#### Platform-Specific Requirements

**macOS**:
```bash
xcode-select --install  # Xcode Command Line Tools
```

**Windows**:
- Microsoft C++ Build Tools (Visual Studio Build Tools)
- WebView2 (usually pre-installed on Windows 10/11)

**Linux**:
```bash
# Debian/Ubuntu
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Arch Linux
sudo pacman -S webkit2gtk-4.1 base-devel curl wget openssl gtk3 libayatana-appindicator librsvg
```

### Initial Setup

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd debugtron-tauri
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Verify Rust toolchain**:
   ```bash
   rustc --version
   cargo --version
   ```

4. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

## Common Commands

### Development
```bash
# Start development server with hot reload
npm run tauri dev

# Build for production (current platform)
npm run tauri build

# Build for specific target (cross-compilation)
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target aarch64-apple-darwin
npm run tauri build -- --target x86_64-apple-darwin
```

### Testing
```bash
# Run Rust tests
cargo test

# Run frontend tests (if configured)
npm test

# Run linting
cargo clippy           # Rust linting
npm run lint           # TypeScript/JavaScript linting
```

### Code Quality
```bash
# Format code
cargo fmt              # Format Rust code
npm run format         # Format TypeScript/React code

# Check for unused dependencies
cargo udeps            # Requires cargo-udeps installation
```

### Release Management
```bash
# Create new version and tag
npm run release <version>      # e.g., npm run release 1.0.0

# Build and upload for current platform
./scripts/build-and-upload.sh <version>      # macOS
.\scripts\build-and-upload.ps1 <version>     # Windows

# Publish release after all platforms uploaded
./scripts/publish-release.sh <version>
```

## Project Structure

```
debugtron-tauri/
├── src/                    # Frontend code (React + TypeScript)
│   ├── components/        # React components
│   │   ├── App.tsx        # Main application container
│   │   ├── DeviceSidebar.tsx
│   │   ├── DevicePanel.tsx
│   │   ├── Session.tsx
│   │   └── Xterm.tsx      # Terminal component
│   ├── store/            # Redux store and slices
│   │   ├── index.ts      # Store configuration
│   │   ├── device.ts     # Device state slice
│   │   └── session.ts    # Session state slice
│   ├── api/              # Tauri API type definitions
│   │   └── tauri.ts      # Command and event types
│   ├── hooks/            # Custom React hooks
│   │   └── useTauriEvents.ts # Tauri event listener
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
│   ├── sync-version.cjs  # Version synchronization
│   ├── release.cjs       # Create release and tag
│   ├── build-and-upload.sh      # macOS build/upload
│   ├── build-and-upload.ps1     # Windows build/upload
│   └── publish-release.cjs      # Publish release
├── public/               # Static assets
└── docs/                # Documentation
    ├── ARCHITECTURE.md
    ├── DEVELOPMENT_GUIDE.md (this file)
    ├── DECISION_LOG.md
    ├── RELEASE_PROCESS.md
    ├── TODO.md
    ├── PROGRESS.md
    └── CLAUDE.md
```

## Git Workflow

### Branch Strategy

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - New features (e.g., `feature/log-filter`)
- `fix/*` - Bug fixes (e.g., `fix/session-sync`)
- `release/*` - Release preparation

### Typical Workflow

1. **Create feature branch**:
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. **Make changes and commit**:
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

3. **Push branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create pull request** from feature branch to `develop`

5. **After review and merge**, delete the feature branch

### Release Process

1. **Create release branch** from `develop`:
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b release/v1.0.0
   ```

2. **Update version numbers** (automated via `npm run release`):
   ```bash
   npm run release 1.0.0
   ```

3. **Build and test** on all platforms

4. **Merge to main** and tag:
   ```bash
   git checkout main
   git merge release/v1.0.0
   git tag v1.0.0
   git push origin main --tags
   ```

5. **Merge back to develop**:
   ```bash
   git checkout develop
   git merge release/v1.0.0
   git push origin develop
   ```

## Code Review Checklist

Before submitting code for review, ensure:

### Rust Code
- [ ] Code passes `cargo clippy` without warnings
- [ ] Code formatted with `cargo fmt`
- [ ] All public APIs have doc comments (`///`)
- [ ] Error handling uses `anyhow::Result` appropriately
- [ ] Async code uses `tokio` runtime correctly
- [ ] No `unwrap()` calls without explicit justification
- [ ] Platform-specific code guarded with `#[cfg(target_os = "...")]`

### TypeScript/React Code
- [ ] Code passes ESLint (`npm run lint`)
- [ ] TypeScript compilation succeeds (`npm run type-check`)
- [ ] No `any` types without justification
- [ ] React components use appropriate hooks (useMemo, useCallback)
- [ ] Redux state updates are immutable
- [ ] Tauri commands/events have proper type definitions in `src/api/tauri.ts`

### Cross-Platform Considerations
- [ ] Tested on at least one target platform
- [ ] Platform-specific behavior documented
- [ ] No hardcoded platform assumptions in shared code

## Commit Message Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style/formatting (no logic changes)
- `refactor`: Code restructuring (no behavior change)
- `perf`: Performance improvements
- `test`: Test additions/modifications
- `chore`: Build/tooling changes
- `build`: Build system changes
- `ci`: CI configuration changes
- `revert`: Revert previous commit

### Examples
```
feat(targets): add Windows PE resource icon extraction
fix(session): correct sessionId mismatch in multi-tab debugging
docs(architecture): update data flow diagrams
chore(deps): update Tauri to v1.5.1
```

## Testing Guidelines

### Rust Tests
- Unit tests in same file as code (`#[cfg(test)] mod tests { ... }`)
- Integration tests in `tests/` directory
- Mock external dependencies (filesystem, network) where possible

### Frontend Tests
- Component tests with React Testing Library
- Redux slice tests for state logic
- Mock Tauri API calls using jest

### End-to-End Testing
- Consider using [WebDriver](https://webdriver.io/) for UI automation
- Test critical user workflows:
  1. Application discovery
  2. Debug session startup
  3. Log display
  4. DevTools opening

## Debugging Tips

### Common Issues

**"Failed to load resource" in DevTools window**
- Check local DevTools server is running (`devtools_server.rs`)
- Verify port allocation and firewall settings

**Session logs not appearing**
- Check `session-log` events are being emitted
- Verify Xterm component `lastContentLengthRef` logic
- Ensure sessionId matches between events and Redux state

**Windows icon extraction fails**
- Verify Windows API dependencies in `Cargo.toml`
- Check icon file permissions
- Test with different Electron applications

**Cross-compilation errors**
- Ensure target toolchain installed: `rustup target add <target>`
- Check Tauri configuration in `tauri.conf.json`

### Logging

Add debug logs in Rust:
```rust
println!("[DEBUG] Starting session for app: {}", app_id);
```

In TypeScript:
```typescript
console.log('[DEBUG] Session added:', sessionId);
```

## Related Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture and design
- [DECISION_LOG.md](DECISION_LOG.md) - Technical decision records
- [RELEASE_PROCESS.md](RELEASE_PROCESS.md) - Detailed release procedures
- [TODO.md](TODO.md) - Pending tasks and roadmap

---

*Last Updated: 2025-12-02*