# Tachyon

[![CI](https://github.com/WyattAu/Tachyon/actions/workflows/ci.yml/badge.svg)](https://github.com/WyattAu/Tachyon/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue.svg)](https://wyattau.github.io/Tachyon)

A deterministic, high-performance knowledge management system built with Rust.

## Features

- **Markdown rendering** -- CommonMark + GFM, syntax highlighting for 11 languages (tree-sitter), KaTeX math, HTML sanitization
- **Real-time collaboration** -- WebSocket with operational transform, live cursors, presence detection
- **Full-text search** -- PostgreSQL tsvector with trigram fuzzy matching, tag filtering, field-level queries
- **RBAC** -- Role-based access control with fine-grained permissions, audit logging
- **SEO** -- Client-side rendering (CSR) via Leptos WASM, JSON-LD, Open Graph, robots.txt, sitemap.xml
- **Caching** -- Path-aware Cache-Control with stale-while-revalidate, ETags
- **API** -- RESTful API with Swagger UI, WebSocket for real-time updates
- **Auth** -- JWT authentication with guest access support

## Architecture

```
┌─────────────────────────────────────────┐
│              Browser (WASM)              │
│         Leptos 0.8 + Tailwind           │
└──────────────────┬──────────────────────┘
                   │ HTTP / WebSocket
┌──────────────────-──────────────────────┐
│           Axum 0.8 Server              │
│  ┌──────────┐ ┌────────┐ ┌──────────┐   │
│  │  API v1   │ │  SEO  │ │   WS     │   │
│  └──────────┘ └────────┘ └──────────┘   │
│  ┌──────────┐ ┌─────────────────────┐   │
│  │  Cache   │ │  RBAC Enforcer    │   │
│  └──────────┘ └─────────────────────┘   │
└──────────────────┬──────────────────────┘
                   │ sqlx
┌──────────────────-──────────────────────┐
│          PostgreSQL 16+                │
│    documents, users, teams, roles,       │
│    search_index, audit_log             │
└────────────────────────────────────────┘
```

## Quick Start

### With Nix (recommended)

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
nix develop    # or: use flake
just build     # build all crates
just test      # run backend test suite
just dev       # lint + test + build
```

### With Docker

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
docker compose up -d
```

### Manual

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon

# Start PostgreSQL (requires createdb tachyon and tachyon user)
cargo run --bin tachyon-server
```

## Development

```bash
just build          # Build all crates
just test           # Run backend test suite
just lint           # Check formatting + clippy
just dev            # Lint + test + build
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/documents` | List documents |
| POST | `/api/v1/documents` | Create document |
| GET | `/api/v1/documents/{id}` | Get document |
| PUT | `/api/v1/documents/{id}` | Update document |
| DELETE | `/api/v1/documents/{id}` | Delete document |
| GET | `/api/v1/documents/search` | Full-text search |
| POST | `/api/v1/documents/{id}/versions` | Create version |
| GET | `/api/v1/documents/{id}/attachments` | List attachments |
| POST | `/api/v1/documents/{id}/attachments` | Upload attachment |
| GET | `/api/v1/auth/status` | Check auth status |
| POST | `/api/v1/auth/login` | Login |
| POST | `/api/v1/auth/guest` | Guest login |
| GET | `/api/v1/users` | List users |
| GET | `/api/v1/teams` | List teams |
| POST | `/api/v1/roles` | Create role |
| GET | `/health` | Health check |
| GET | `/robots.txt` | SEO robots.txt |
| GET | `/sitemap.xml` | Dynamic sitemap |
| GET | `/docs/{id}` | SSR document page |
| GET | `/swagger-ui/` | API documentation |
| WS | `/ws` | Real-time collaboration |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TACHYON_HOST` | `0.0.0.0` | Server bind address |
| `TACHYON_PORT` | `8080` | Server port |
| `DATABASE_URL` | `postgres://...` | PostgreSQL connection string |
| `TACHYON_JWT_SECRET` | (required) | JWT signing secret (min 32 chars) |
| `TACHYON_GUEST_LOGIN_ENABLED` | `false` | Enable guest auto-login |
| `TACHYON_PUBLIC_NOTES_ENABLED` | `false` | Allow public document access |
| `TACHYON_SITE_TITLE` | `Tachyon` | Site title for SEO |
| `TACHYON_BASE_URL` | `http://localhost:8080` | Canonical base URL |
| `TACHYON_RATE_LIMIT_ENABLED` | `true` | Enable rate limiting |

## Crate Layout

| Crate | Description |
|-------|-------------|
| `tachyon-core` | Core types, domain models, and shared utilities |
| `tachyon-server` | HTTP/2 server with Axum, middleware, and API routes |
| `tachyon-database` | PostgreSQL layer with migrations and repositories |
| `tachyon-renderer` | Markdown rendering with extensions, syntax highlighting, and TOC |
| `tachyon-search` | Full-text search indexing and querying with Tantivy |
| `tachyon-rbac` | Role-based access control engine |
| `tachyon-frontend` | WASM-based web frontend (Leptos 0.8 + Trunk) |
| `tachyon-desktop` | Native desktop client (Tauri 2) |
| `tachyon-cli` | Command-line interface |
| `tachyon-storage` | Pluggable storage backends (SQLite, in-memory) |
| `tachyon-editor` | Native Rust text editor engine with CRDT support |
| `tachyon-import-export` | Import/export (Docusaurus, Obsidian, Markdown, JSON, HTML) |
| `tachyon-ssg` | Static site generator |
| `tachyon-plugin-runtime` | WASM plugin runtime using Wasmtime |
| `tachyon-testing` | Shared test utilities, fuzzing harnesses, and benchmarks |
| `tachyon-benchmarks` | Criterion benchmark suite |

## Technology Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2024 Edition |
| Async Runtime | Tokio |
| HTTP | Axum 0.8 |
| Frontend | Leptos 0.8 (WASM) |
| Database | PostgreSQL 16+ via sqlx |
| Search | Tantivy (BM25) + PostgreSQL tsvector fallback |
| Markdown | pulldown-cmark (SIMD) |
| Math | KaTeX |
| Syntax | tree-sitter (12 languages) + syntect |
| Auth | JWT (jsonwebtoken) |
| CSS | Tailwind CSS (CDN) |
| Fonts | Inter, JetBrains Mono |

## License

Apache License, Version 2.0 -- see [LICENSE](LICENSE) for details.

## Documentation

Full documentation is available at [wyattau.github.io/Tachyon](https://wyattau.github.io/Tachyon).

See [CHANGELOG.md](CHANGELOG.md) for release history.
