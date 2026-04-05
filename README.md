# Tachyon

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.2.0--beta-green.svg)](VERSION.md)

A deterministic, high-performance knowledge management platform for teams and individuals.

## Overview

Tachyon is a local-first documentation engine that operates directly on your file system or Git repository. Unlike traditional static site generators, Tachyon eliminates build steps by providing instant, just-in-time rendering.

### Key Features

- **Sub-15ms Rendering**: Content renders on-demand without build steps
- **Local-First**: Full offline functionality with Git-based version control
- **Real-Time Collaboration**: Live cursors, presence, and collaborative editing
- **Full-Text Search**: Sub-100ms search with fuzzy matching
- **Role-Based Access Control**: Fine-grained permissions and content redaction
- **Cross-Platform**: Native desktop apps + web interface + headless server

## Quick Start

```bash
# Clone and setup
git clone https://github.com/tachyon-org/tachyon.git
cd tachyon
./scripts/quickstart.sh setup

# Start development server
./scripts/quickstart.sh start

# Run tests
./scripts/quickstart.sh test
```

| Command | Description |
|---------|-------------|
| `setup` | Build project and install dependencies |
| `start` | Start development server |
| `stop` | Stop development server |
| `test` | Run test suite |
| `status` | Show project status |
| `clean` | Clean build artifacts |

## Installation

### Desktop Application

Download platform-specific installers:

| Platform | Download |
|----------|----------|
| Windows | `tachyon_setup_x64.exe` |
| macOS | `Tachyon.dmg` |
| Linux | `tachyon_amd64.deb` / `tachyon-x86_64.AppImage` |

### Server (Docker)

```bash
docker pull tachyon-org/tachyon-server:latest
docker run -d -p 8080:8080 -v /path/to/docs:/docs tachyon-org/tachyon-server:latest
```

### Server (Binary)

```bash
cargo build --release --no-default-features --features "server-mode"
```

See [Installation Guide](docs/user/installation.md) for detailed instructions.

## Operation Modes

### Desktop Mode

Local-first application for personal knowledge management:

```bash
tachyon /path/to/docs
```

- Operates on local Git repository
- No network required
- Full offline functionality

### Server Mode

Headless server for team documentation portals:

```bash
tachyon serve --port 8080 --config tachyon.toml
```

- Multi-user collaboration
- Authentication and RBAC
- Real-time editing

### Static Export

Generate static HTML for any hosting:

```bash
tachyon build --output ./dist
```

- Deploy to GitHub Pages, Netlify, etc.
- No server required

## Configuration

Create `tachyon.toml` in your repository root:

```toml
[system]
mode = "hybrid"          # desktop | server | static
watch_interval_ms = 100

[server]
host = "0.0.0.0"
port = 8080
auth_provider = "kanidm"
enable_sso = true

[rendering]
math_engine = "katex"
syntax_theme = "axiom-dark"
enable_diagrams = true

[security]
exclude = [".env", "*.secret.md", "private/"]
```

See [Configuration Guide](docs/user/configuration_guide.md) for all options.

## Features

### Markdown Support

- CommonMark and GitHub Flavored Markdown
- Syntax highlighting for 12+ languages
- KaTeX math rendering
- Mermaid.js diagrams
- YAML frontmatter

### Search

- Full-text search with Tantivy
- Fuzzy matching
- Field filters (`author:`, `tag:`, `status:`)
- Date ranges

### Collaboration

- Real-time editing with operational transform
- Live cursors and presence
- Comments and threads
- Version history

### Security

- Multiple auth providers (Kanidm, OAuth, LDAP)
- Role-based access control
- Document-level permissions
- Block-level content redaction

## Documentation

### User Documentation

| Guide | Description |
|-------|-------------|
| [Getting Started](docs/user/README.md) | Overview and quick links |
| [Installation](docs/user/installation.md) | Installation guide |
| [Quick Start](docs/user/quick-start.md) | 5-minute tutorial |
| [Features](docs/user/features.md) | Feature overview |
| [Documents](docs/user/documents.md) | Document management |
| [Search](docs/user/search.md) | Search functionality |
| [Collaboration](docs/user/collaboration.md) | Real-time collaboration |
| [Permissions](docs/user/permissions.md) | Roles and access control |

### Developer Documentation

| Guide | Description |
|-------|-------------|
| [Developer Overview](docs/dev/README.md) | Getting started for developers |
| [Architecture](docs/dev/architecture.md) | System architecture |
| [Setup](docs/dev/setup.md) | Development environment |
| [Contributing](docs/dev/contributing.md) | Contribution guidelines |
| [Testing](docs/dev/testing.md) | Testing guide |
| [Deployment](docs/dev/deployment.md) | Deployment guide |

### API Documentation

- [REST API](docs/api/rest_api_documentation.md)
- [WebSocket API](docs/api/websocket_api_documentation.md)
- [Search API](docs/api/search_api_specification.md)

### Architecture

| Document | Description |
|----------|-------------|
| [Overview](docs/architecture/overview.md) | High-level architecture |
| [Database](docs/architecture/database.md) | Database schema |
| [API Design](docs/architecture/api.md) | REST API design |
| [WebSocket](docs/architecture/websocket.md) | WebSocket protocol |
| [Security](docs/architecture/security.md) | Security architecture |

## Technology Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2024 Edition |
| Async Runtime | Tokio |
| Server | Axum (HTTP/2) |
| Desktop | Tauri 2.0 |
| Frontend | Leptos (WASM) |
| Database | SQLite (rusqlite) |
| Search | Tantivy |
| Markdown | pulldown-cmark (SIMD) |
| Git | git2-rs |

## Performance

| Metric | Target |
|--------|--------|
| Render latency | < 15ms |
| Search query | < 100ms |
| File watch response | < 50ms |
| WebSocket update | < 10ms |
| Memory usage | < 100MB base |

## Requirements

### Desktop

| Platform | Minimum |
|----------|---------|
| Windows | Windows 10 (Build 1903+) |
| macOS | macOS 11 (Big Sur) |
| Linux | Kernel ≥ 5.4, GTK3 |

### Server

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| RAM | 2GB | 4GB+ |
| CPU | 2 cores | 4+ cores |
| Disk | 1GB/1000 docs | SSD |

## Contributing

We welcome contributions! Please see:

- [Contributing Guide](docs/dev/contributing.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

### Development Setup

```bash
git clone https://github.com/tachyon-org/tachyon.git
cd tachyon
cargo build
cargo test
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

Copyright © 2026. Licensed under the [Apache License, Version 2.0](LICENSE).

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

## Support

- **Documentation**: [docs/](docs/)
- **Issues**: GitHub Issues
- **Security**: security@tachyon.example.com

## Acknowledgments

Built with:
- [Rust](https://rust-lang.org/)
- [Tokio](https://tokio.rs/)
- [Axum](https://github.com/tokio-rs/axum)
- [Tauri](https://tauri.app/)
- [Leptos](https://leptos.dev/)
- [Tantivy](https://github.com/quickwit-oss/tantivy)
