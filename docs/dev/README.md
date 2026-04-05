# Developer Documentation

Welcome to the Tachyon developer documentation.

## Quick Links

| Guide | Description |
|-------|-------------|
| [Architecture](architecture.md) | System architecture overview |
| [Setup](setup.md) | Development environment setup |
| [Contributing](contributing.md) | Contribution guidelines |
| [Testing](testing.md) | Testing guide |
| [Deployment](deployment.md) | Deployment guide |

## Project Structure

```
tachyon/
├── crates/
│   ├── core/           # Core domain logic
│   ├── server/         # HTTP/2 server (Axum)
│   ├── desktop/        # Desktop application (Tauri)
│   ├── frontend/       # Web frontend (Leptos)
│   ├── database/       # Database layer (SQLite)
│   ├── renderer/       # Markdown rendering
│   ├── search/         # Full-text search (Tantivy)
│   ├── rbac/           # Role-based access control
│   ├── cli/            # Command-line interface
│   └── testing/        # Testing utilities
├── docs/               # Documentation
├── scripts/            # Build and utility scripts
└── tachyon.toml        # Configuration
```

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
| Markdown | pulldown-cmark |
| Git | git2-rs |

## Key Features

### Just-In-Time Rendering

No build step - content renders on demand:

```rust
use tachyon_renderer::{Renderer, OutputFormat};

let renderer = Renderer::new();
let html = renderer.render_to_html(&markdown)?;
```

### Real-Time File Watching

Kernel-level file monitoring:

```rust
use notify::{Watcher, RecursiveMode, watcher};

let (tx, rx) = channel();
let mut watcher = watcher(tx, Duration::from_millis(100))?;
watcher.watch("./docs", RecursiveMode::Recursive)?;
```

### Full-Text Search

Sub-100ms search with Tantivy:

```rust
use tachyon_search::{SearchEngine, Query};

let engine = SearchEngine::open("./index")?;
let results = engine.search(Query::parse("api authentication")?)?;
```

## Architecture Overview

### Layers

1. **Runtime Layer**: Tokio async runtime
2. **Reactive Layer**: File watching and cache invalidation
3. **Processing Layer**: Git, rendering, search
4. **Presentation Layer**: Desktop (Tauri), Server (Axum), Web (Leptos)

### Crates

| Crate | Responsibility |
|-------|----------------|
| `tachyon-core` | Domain types, traits, error handling |
| `tachyon-server` | HTTP/2 server, WebSocket, REST API |
| `tachyon-desktop` | Tauri application, IPC |
| `tachyon-frontend` | Web UI components |
| `tachyon-database` | SQLite operations, migrations |
| `tachyon-renderer` | Markdown parsing and rendering |
| `tachyon-search` | Full-text search indexing |
| `tachyon-rbac` | Permissions and access control |

## Getting Started

1. [Set up your development environment](setup.md)
2. [Understand the architecture](architecture.md)
3. [Read the contributing guidelines](contributing.md)
4. [Run the tests](testing.md)

## Documentation

- [API Documentation](../api/) - REST and WebSocket APIs
- [Architecture](../architecture/) - System design documents
- [Component Guides](components/) - Individual component docs

## Community

- GitHub Issues: Bug reports and feature requests
- Pull Requests: Code contributions
- Discussions: Questions and ideas

## License

Apache License 2.0 - See [LICENSE](../../LICENSE) for details.
