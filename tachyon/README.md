# Tachyon

A deterministic, high-performance knowledge management platform for teams and individuals.

## Overview

Tachyon is a Rust-based documentation engine that operates directly on your file system or Git repository. It provides instant, just-in-time rendering without build steps.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tachyon-server = "0.2"
```

## Quick Start

### Server Mode

```rust,no_run
use tachyon_server::config::ServerConfig;
use tachyon_server::run_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::from_env();
    run_server(config).await
}
```

### Configuration

```rust
use tachyon_server::config::ServerConfig;

let config = ServerConfig {
    host: "0.0.0.0".to_string(),
    port: 8080,
    database_url: "postgres://localhost/tachyon".to_string(),
    ..Default::default()
};
```

## Features

- **Sub-15ms Rendering**: Just-in-time rendering without build steps
- **Full-Text Search**: Sub-100ms search with Tantivy
- **Real-Time Collaboration**: WebSocket-based live editing
- **Role-Based Access Control**: Fine-grained permissions
- **REST API**: Comprehensive REST API with OpenAPI docs

## Crates

| Crate | Description |
|-------|-------------|
| `tachyon-server` | HTTP/2 server with Axum |
| `tachyon-database` | PostgreSQL database layer |
| `tachyon-renderer` | Markdown rendering |
| `tachyon-search` | Full-text search with Tantivy |
| `tachyon-rbac` | Role-based access control |

## Documentation

- [User Guide](../docs/user-guide/README.md)
- [Developer Guide](../docs/developer/README.md)
- [API Reference](../docs/api/authentication.md)

## License

Apache License 2.0
