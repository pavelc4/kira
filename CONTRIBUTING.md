# Contributing to Kira

Thank you for your interest in contributing to Kira! This guide will help you set up your development environment for the best experience.

## Development Requirements

- **Bun** (package manager) - [Install Bun](https://bun.sh)
- **Rust** (latest stable) - [Install Rust](https://rustup.rs)
- **Node.js** 18+ (for tooling)
- **Android SDK** (for ADB features)

### Platform-Specific Requirements

#### Linux
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Arch Linux
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module gtk3 libappindicator-gtk3 librsvg
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# No additional dependencies needed
```

#### Windows
```bash
# Install Microsoft C++ Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Install WebView2 (usually pre-installed on Windows 11)
# Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

## Getting Started

### Clone and Install

```bash
git clone https://github.com/pavelc4/kira
cd kira
bun install
```

### Running Development Server

#### Linux (Ubuntu/Wayland)

For optimal performance on Linux with Wayland, we provide optimization scripts:

```bash
# Setup performance optimizations (installs mold linker)
./optimize-linux-wayland.sh

# Run with X11 (recommended for best performance)
bun run tauri:x11 dev

# Or run with Wayland
bun run tauri:wayland dev
```

#### macOS & Windows

```bash
# Standard development command
bun run tauri dev
```

## Performance Optimization Guide

### Linux (Ubuntu/Wayland)

Tauri projects on Wayland often experience performance issues due to WebKit rendering and compositor conflicts. We've implemented several optimizations specifically for Linux development.

### Implemented Solutions

### 1. Cargo Build Optimization (`.cargo/config.toml`)
- Parallel compilation using all CPU cores
- Mold linker for 3-5x faster linking
- Level 3 dependency optimization
- Incremental compilation enabled

### 2. Platform-Specific NPM Scripts (`package.json`)

**Linux:**
```bash
# Use X11 (more stable and faster)
bun run tauri:x11 dev

# Use Wayland (if needed)
bun run tauri:wayland dev
```

**macOS & Windows:**
```bash
# Standard command
bun run tauri dev
```

### 3. Vite Optimization (`vite.config.ts`)
- Fast HMR configuration
- Modern build target
- Optimized dependency pre-bundling

### Installing Optimization Dependencies (Linux Only)

The mold linker significantly speeds up Rust compilation:

```bash
# Automated installation (Linux only)
./optimize-linux-wayland.sh

# Or manual installation
sudo apt update
sudo apt install mold clang

# Verify installation
mold --version
clang --version
```

### First Build

The first build will be slow (2-5 minutes) regardless of platform. This is normal for Rust projects.

```bash
# Clean build
cargo clean
bun run tauri dev  # or tauri:x11 dev on Linux
```

Subsequent builds will be much faster (10-30 seconds) thanks to incremental compilation.

## Development Tips (All Platforms)

### 1. Use `cargo watch` for Auto-rebuild
```bash
cargo install cargo-watch
cargo watch -x check
```

### 2. Build for Testing
```bash
# Debug build (faster compilation)
bun run tauri build --debug

# Release build (optimized)
bun run tauri build
```

**Binary locations:**
- Linux: `src-tauri/target/debug/kira` or `src-tauri/target/release/kira`
- macOS: `src-tauri/target/debug/kira` or `src-tauri/target/release/bundle/macos/`
- Windows: `src-tauri/target/debug/kira.exe` or `src-tauri/target/release/kira.exe`

### 3. Hot Reload Configuration
Edit `tauri.conf.json` and comment out `beforeDevCommand` if you want to develop without Vite hot reload.

## Troubleshooting

### Build Errors (All Platforms)

**Port Already in Use:**
```bash
# Kill existing Vite process
lsof -ti:5173 | xargs kill -9

# Or use different port in vite.config.ts
```

**Clean and Rebuild:**
```bash
# Clean and rebuild
cargo clean
rm -rf node_modules dist .svelte-kit
bun install
bun run tauri dev
```

### Linux-Specific Issues

**Slow Performance:**
1. Ensure mold is installed: `mold --version`
2. Try X11: `bun run tauri:x11 dev`
3. Temporarily disable compositor (KDE/GNOME settings)
4. Check swap usage: `free -h`

**WebKit Crashes:**
```bash
# Use software rendering
LIBGL_ALWAYS_SOFTWARE=1 bun run tauri:x11 dev
```

### macOS-Specific Issues

**Code Signing Errors:**
```bash
# Disable code signing for development
export TAURI_SKIP_DEVTOOLS_CHECK=true
```

**Permission Errors:**
```bash
# Grant terminal full disk access in System Preferences > Security & Privacy
```

### Windows-Specific Issues

**WebView2 Not Found:**
- Download and install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

**Build Tools Missing:**
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Select "Desktop development with C++" workload

## Performance Comparison (Linux)

| Method | First Build | Rebuild | Runtime |
|--------|-------------|---------|---------|
| Default Wayland | ~3-5 min | ~30-60s | Laggy |
| With Optimization + Wayland | ~2-3 min | ~10-20s | Better |
| With Optimization + X11 | ~2-3 min | ~10-20s | Smooth |

**Note:** macOS and Windows typically have smooth performance out of the box.

## Code Style

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **TypeScript/Svelte**: Use Prettier (run `bun run format`)
- **Commits**: Use conventional commits format

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting:
   ```bash
   cargo test
   bun run check
   bun run lint
   ```
5. Commit your changes (`git commit -m 'feat: add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Project Structure

```
kira/
├── src/              # Svelte frontend
├── src-tauri/        # Tauri Rust backend
├── kira-core/        # Core Rust library
├── android/          # Android native components
└── static/           # Static assets
```

## Need Help?

- Open an [Issue](https://github.com/pavelc4/kira/issues)
- Check existing [Discussions](https://github.com/pavelc4/kira/discussions)

## Additional Optimizations

If you find additional optimizations, please update this document and submit a PR!
