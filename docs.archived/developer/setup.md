# Development Setup

Guide to setting up your development environment for Tachyon.

## Prerequisites

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Primary language |
| Node.js | 18+ | Frontend tooling |
| Bun | 1.2+ | Package manager |
| Git | 2.30+ | Version control |

### Platform-Specific

**Windows:**
- Visual Studio Build Tools (MSVC)
- WebView2 (included in Windows 10/11)

**macOS:**
- Xcode Command Line Tools: `xcode-select --install`

**Linux:**
- GTK3 development files
- WebKit2GTK
- OpenSSL development files

### Optional

| Tool | Purpose |
|------|---------|
| Docker | Containerized testing |
| Nix | Reproducible builds |
| just | Task runner |

## Quick Setup

```bash
# Clone the repository
git clone https://github.com/WyattAu/Tachyon.git
cd tachyon

# Run quickstart script
./scripts/quickstart.sh setup

# Start development server
./scripts/quickstart.sh start
```

## Manual Setup

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Add required targets:
```bash
rustup target add wasm32-unknown-unknown
```

### 2. Install Node.js and Bun

**Node.js (via nvm):**
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

**Bun:**
```bash
curl -fsSL https://bun.sh/install | bash
```

### 3. Install Platform Dependencies

**Ubuntu/Debian:**
```bash
sudo apt install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl-dev \
    pkg-config
```

**Fedora:**
```bash
sudo dnf install -y \
    gtk3-devel \
    webkit2gtk3-devel \
    libappindicator-gtk3-devel \
    openssl-devel
```

**Arch Linux:**
```bash
sudo pacman -S --needed \
    gtk3 \
    webkit2gtk \
    libappindicator-gtk3 \
    openssl
```

**macOS:**
```bash
xcode-select --install
brew install openssl
```

### 4. Clone and Build

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd tachyon/tachyon

# Build all crates
cargo build

# Build frontend
cd crates/frontend
bun install
bun run build
```

### 5. Verify Setup

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Start server
cargo run --bin tachyon-server
```

## Project Structure

```
tachyon/
├── crates/
│   ├── core/           # Core domain logic
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── document.rs
│   │   │   ├── error.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   ├── server/         # HTTP server
│   ├── desktop/        # Desktop app
│   ├── frontend/       # Web frontend
│   ├── database/       # Database layer
│   ├── renderer/       # Markdown rendering
│   ├── search/         # Search engine
│   ├── rbac/           # Access control
│   ├── cli/            # CLI tools
│   └── testing/        # Test utilities
├── docs/               # Documentation
├── scripts/            # Utility scripts
├── Cargo.toml          # Workspace config
└── flake.nix           # Nix flake (optional)
```

## Development Workflow

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p tachyon-server

# Build in release mode
cargo build --release

# Build frontend
cd crates/frontend && bun run build
```

### Running

```bash
# Run server
cargo run --bin tachyon-server -- --port 8080

# Run desktop app
cargo run --bin tachyon-desktop

# Run CLI
cargo run --bin tachyon-cli -- --help
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p tachyon-core

# Run specific test
cargo test test_document_creation

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Run clippy with strict warnings
cargo clippy -- -D warnings
```

### Documentation

```bash
# Generate docs
cargo doc --open

# Generate docs with private items
cargo doc --document-private-items
```

## IDE Setup

### VS Code

Install extensions:
- rust-analyzer
- CodeLLDB
- Better TOML
- markdownlint

Settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer",
        "editor.formatOnSave": true
    }
}
```

Launch configuration (`.vscode/launch.json`):
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Server",
            "cargo": {
                "args": ["build", "--bin=tachyon-server"],
                "filter": {
                    "name": "tachyon-server",
                    "kind": "bin"
                }
            },
            "args": ["--port", "8080"]
        }
    ]
}
```

### IntelliJ IDEA / CLion

1. Install Rust plugin
2. Open project as Cargo project
3. Configure Rust toolchain in Settings

### Neovim

Using nvim-lspconfig with rust-analyzer:

```lua
local lspconfig = require('lspconfig')
lspconfig.rust_analyzer.setup({
    settings = {
        ['rust-analyzer'] = {
            checkOnSave = {
                command = 'clippy'
            }
        }
    }
})
```

## Environment Variables

Create `.env.development`:

```bash
# Server
TACHYON_HOST=127.0.0.1
TACHYON_PORT=8080
TACHYON_MODE=development

# Database
DATABASE_URL=sqlite:./tachyon.db

# Logging
RUST_LOG=tachyon=debug,info

# Security (development only)
TACHYON_DISABLE_AUTH=true
```

## Nix Development (Optional)

If using Nix:

```bash
# Enter development shell
nix develop

# Or use direnv
direnv allow
```

The `flake.nix` provides:
- Rust toolchain
- Node.js and Bun
- Platform dependencies
- Development tools

## Docker Development

```bash
# Build development image
docker build -t tachyon-dev -f Dockerfile.dev .

# Run with volume mount
docker run -it --rm \
    -v $(pwd):/app \
    -p 8080:8080 \
    tachyon-dev
```

## Database Setup

```bash
# Run migrations
cargo run --bin tachyon-cli -- database migrate

# Seed test data
cargo run --bin tachyon-cli -- database seed

# Reset database
cargo run --bin tachyon-cli -- database reset
```

## Troubleshooting

### Build Errors

**Linker errors:**
```bash
# Install platform dependencies (see above)
# Or set custom linker
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=gcc
```

**OpenSSL errors:**
```bash
# Linux
export OPENSSL_DIR=/usr/local/ssl
# macOS
export OPENSSL_DIR=$(brew --prefix openssl)
```

### Runtime Errors

**Port in use:**
```bash
# Find process using port
lsof -i :8080
# Kill process
kill -9 <PID>
```

**Database locked:**
```bash
# Close all connections
cargo run --bin tachyon-cli -- database close
```

### Desktop App Issues

**Tauri build fails:**
```bash
# Clear Tauri cache
rm -rf ~/.cache/tauri
# Rebuild
cargo tauri dev
```

## Next Steps

1. Read [Architecture Overview](architecture.md)
2. Review [Contributing Guidelines](contributing.md)
3. Run the [Test Suite](testing.md)
4. Explore the [API Documentation](../api/)
