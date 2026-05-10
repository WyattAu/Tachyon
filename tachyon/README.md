# Tachyon

A deterministic, high-performance knowledge management platform for teams and individuals.

## Features

- **Sub-15ms Rendering** — Just-in-time Markdown rendering with SIMD-accelerated pulldown-cmark
- **Full-Text Search** — Sub-100ms search powered by Tantivy
- **Real-Time Collaboration** — WebSocket-based live editing with conflict detection
- **Role-Based Access Control** — Fine-grained RBAC with teams, projects, and permissions
- **REST API** — Comprehensive REST API with OpenAPI/Swagger docs
- **Desktop App** — Native desktop client via Tauri
- **WASM Frontend** — WebAssembly-based web UI
- **CLI** — Command-line interface for scripting and automation
- **PostgreSQL Backend** — Persistent storage with migrations
- **Formal Verification** — TLA+ specs and Lean4 proofs in `specs/`

## Quick Start

### Nix (recommended)

```bash
nix develop
cargo run --release -p tachyon-server
```

### Docker

```bash
docker compose up
```

### Cargo

```bash
cd tachyon
cargo run --release -p tachyon-server
```

The server starts at `http://localhost:8080`. API docs are available at `/api/docs`.

## Development

### Prerequisites

- **Rust** 1.75+ (edition 2021)
- **PostgreSQL** 16+ (for database-backed features)
- **Node.js** 20+ (for frontend builds, if modifying the WASM UI)
- **Trunk** (install via `cargo install trunk` for frontend dev)

### Running Tests

```bash
cargo test --workspace --lib
```

### Running Frontend

```bash
cd crates/frontend
trunk serve
```

### Linting

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
```

## Configuration

Configure via environment variables or a `.env` file:

| Variable | Description | Default |
|---|---|---|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://tachyon:tachyon_dev_password@127.0.0.1:5433/tachyon` |
| `TACHYON_HOST` | Server bind host | `0.0.0.0` |
| `TACHYON_PORT` | Server bind port | `8080` |
| `TACHYON_JWT_SECRET` | JWT signing secret (min 32 chars) | `change-this-secret-key-in-production` |
| `TACHYON_JWT_EXPIRATION` | Token expiration in seconds | `86400` |
| `TACHYON_CORS_ORIGINS` | Allowed CORS origins | `*` |
| `TACHYON_GUEST_LOGIN_ENABLED` | Enable guest auto-login | `false` |
| `TACHYON_BASE_URL` | Canonical base URL | `http://localhost:8080` |

See `.env.example` for a production template.

## Project Structure

| Crate | Description |
|---|---|
| `tachyon-core` | Core types, domain models, and shared utilities |
| `tachyon-server` | HTTP/2 server with Axum, middleware, and API routes |
| `tachyon-database` | PostgreSQL layer with migrations and repositories |
| `tachyon-renderer` | Markdown rendering with extensions and TOC |
| `tachyon-search` | Full-text search indexing and querying with Tantivy |
| `tachyon-rbac` | Role-based access control engine |
| `tachyon-frontend` | WASM-based web frontend (Leptos 0.8 + Trunk) |
| `tachyon-desktop` | Native desktop client (Tauri) |
| `tachyon-cli` | Command-line interface |
| `tachyon-testing` | Shared test utilities, fuzzing harnesses, and benchmarks |

## Formal Verification

The `specs/` directory contains formal specifications:

- **TLA+** (`specs/tla/`) — Temporal logic models for concurrency, state machines, and distributed consensus
- **Lean4** (`specs/lean/`) — Proof-carrying code for correctness properties

## License

Apache License 2.0
