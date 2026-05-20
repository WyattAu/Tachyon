# Developer Documentation

Welcome to the Tachyon developer documentation.

## Quick Links

| Guide | Description |
|-------|-------------|
| [Architecture](architecture.md) | System architecture overview |
| [Database](database.md) | Database schema and migrations |
| [API Guide](api.md) | API usage guide |
| [WebSockets](websockets.md) | WebSocket protocol |
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
| Database | PostgreSQL (sqlx) |
| Search | Tantivy |
| Markdown | pulldown-cmark |
| Git | git2-rs |

## Getting Started

### Prerequisites

- Rust 1.75+ (2024 edition)
- PostgreSQL 12+
- Node.js 18+ (for frontend)
- pnpm 8+ (for frontend)

### Setup

```bash
# Clone the repository
git clone https://github.com/WyattAu/Tachyon.git
cd tachyon

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
cargo build

# Setup database
cargo run --bin tachyon-db-setup

# Run development server
cargo run --bin tachyon-server
```

### Development Commands

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run specific tests
cargo test --package tachyon-server

# Check code
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Generate documentation
cargo doc --open
```

## Architecture

### System Overview

```mermaid
graph TD
    A[Client] --> B[Server]
    B --> C[Database]
    B --> D[Search Index]
    B --> E[File System]
    
    B --> F[WebSocket]
    F --> G[Real-Time Updates]
    
    B --> H[Renderer]
    H --> I[Markdown to HTML]
    
    A --> J[Desktop App]
    A --> K[Web Interface]
```

### Crate Responsibilities

| Crate | Responsibility |
|-------|----------------|
| `tachyon-core` | Domain types, traits, error handling |
| `tachyon-server` | HTTP/2 server, WebSocket, REST API |
| `tachyon-desktop` | Tauri application, IPC |
| `tachyon-frontend` | Web UI components |
| `tachyon-database` | PostgreSQL operations, migrations |
| `tachyon-renderer` | Markdown parsing and rendering |
| `tachyon-search` | Full-text search indexing |
| `tachyon-rbac` | Permissions and access control |

## Key Features

### Just-In-Time Rendering

No build step - content renders on demand:

```rust
use tachyon_renderer::{Renderer, OutputFormat};

let renderer = Renderer::new();
let html = renderer.render_to_html(&markdown)?;
```

### Real-Time Collaboration

WebSocket-based collaboration:

```rust
use tachyon_server::websocket::ConnectionManager;

let manager = ConnectionManager::new();
manager.broadcast(document_id, event).await?;
```

### Full-Text Search

Sub-100ms search with Tantivy:

```rust
use tachyon_search::{SearchEngine, Query};

let engine = SearchEngine::open("./index")?;
let results = engine.search(Query::parse("api authentication")?)?;
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

Follow the [Contributing Guide](contributing.md) for code style and conventions.

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run specific tests
cargo test --package tachyon-server --test api_tests

# Run with coverage
cargo tarpaulin --out Html
```

### 4. Check Code Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### 5. Submit Pull Request

See [Contributing Guide](contributing.md) for PR guidelines.

## Configuration

### Environment Variables

```bash
# Server configuration
export TACHYON_HOST=0.0.0.0
export TACHYON_PORT=8080
export DATABASE_URL=postgres://tachyon:password@localhost:5432/tachyon

# JWT configuration
export TACHYON_JWT_SECRET=your-secret-key
export TACHYON_JWT_EXPIRATION=86400

# Logging
export RUST_LOG=tachyon_server=debug,tower_http=trace
```

### Configuration File

Create `tachyon.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080
database_url = "postgres://tachyon:password@localhost:5432/tachyon"

[jwt]
secret = "your-secret-key"
expiration_secs = 86400

[websocket]
enabled = true
max_connections = 1000
```

## Debugging

### Enable Debug Logging

```bash
export RUST_LOG=tachyon=debug,tower_http=trace
cargo run
```

### Database Debugging

```bash
# Connect to PostgreSQL
psql postgres://tachyon:password@localhost:5432/tachyon

# Check tables
\dt

# Query documents
SELECT * FROM documents LIMIT 10;
```

### Performance Profiling

```bash
# Build with debug symbols
cargo build --profile release-with-debug

# Run with perf
perf record -g ./target/release-with-debug/tachyon-server
perf report
```

## Common Tasks

### Add a New API Endpoint

1. Define route in `crates/server/src/routes/`
2. Add handler function
3. Create request/response types
4. Add tests
5. Update API documentation

### Add Database Migration

```bash
# Create migration
cargo run --bin tachyon-db-migrate -- create add_new_table

# Edit migration file in migrations/
# Run migrations
cargo run --bin tachyon-db-migrate -- up
```

### Update Frontend

1. Make changes in `crates/frontend/src/`
2. Build WASM: `cargo build --target wasm32-unknown-unknown`
3. Test in browser
4. Update component tests

## Resources

- [Architecture Guide](architecture.md) - Detailed system architecture
- [Database Guide](database.md) - Database schema and operations
- [API Guide](api.md) - REST API documentation
- [WebSocket Guide](websockets.md) - WebSocket protocol
- [Contributing](contributing.md) - Contribution guidelines
- [Testing Guide](testing.md) - Testing strategies
- [Deployment](deployment.md) - Production deployment

## Community

- **GitHub Issues**: Bug reports and feature requests
- **Pull Requests**: Code contributions
- **Discussions**: Questions and ideas
- **Security**: security@tachyon.example.com

## License

Apache License 2.0 - See [LICENSE](../../LICENSE) for details.
