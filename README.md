# Tachyon

[![CI](https://github.com/WyattAu/Tachyon/actions/workflows/ci.yml/badge.svg)](https://github.com/WyattAu/Tachyon/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue.svg)](https://wyattau.github.io/Tachyon)

A deterministic, high-performance knowledge management system built in Rust. Comprises 16 workspace crates spanning a web server, WASM frontend, desktop client, static site generator, plugin sandbox, and CRDT-based real-time collaboration.

## Features

- **Markdown rendering** -- CommonMark + GFM via pulldown-cmark (SIMD-accelerated), tree-sitter syntax highlighting (12 languages), KaTeX math rendering, HTML sanitization
- **Real-time collaboration** -- WebSocket transport with Yrs/lib0 CRDT conflict resolution (Y.js-compatible), live cursors, presence detection
- **Full-text search** -- Tantivy (BM25 ranking, TF-IDF) for standalone deployments; PostgreSQL `tsvector` with trigram (`pg_trgm`) fuzzy matching as fallback; tag filtering, field-level queries
- **RBAC** -- Role-based access control with fine-grained permissions and audit logging
- **SEO** -- Server-side rendering, JSON-LD structured data, Open Graph metadata, `robots.txt`, `sitemap.xml`
- **HTTP caching** -- Path-aware `Cache-Control` with `stale-while-revalidate`, ETag support
- **API** -- RESTful JSON API with Swagger UI; WebSocket endpoint for real-time updates
- **Auth** -- JWT authentication (HS256) with optional guest access

## Architecture

```
┌──────────────────────────────────────────┐
│              Browser (WASM)               │
│          Leptos 0.8 + Tailwind            │
└──────────────────┬───────────────────────┘
                   │ HTTP / WebSocket
┌──────────────────┴───────────────────────┐
│            Axum 0.8 Server               │
│  ┌──────────┐ ┌────────┐ ┌──────────┐    │
│  │  API v1   │ │  SEO   │ │    WS    │    │
│  └──────────┘ └────────┘ └──────────┘    │
│  ┌──────────┐ ┌─────────────────────┐    │
│  │  Cache   │ │   RBAC Enforcer     │    │
│  └──────────┘ └─────────────────────┘    │
└──────────────────┬───────────────────────┘
                   │ sqlx (async)
┌──────────────────┴───────────────────────┐
│           PostgreSQL 16+                  │
│  documents, users, teams, roles,          │
│  search_index, audit_log                  │
└──────────────────────────────────────────┘
```

## Quick Start

### Nix (Recommended)

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
nix develop    # enter dev shell via flake
just build     # compile all crates
just test      # run backend test suite
just dev       # lint + test + build
```

### Docker

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
docker compose up -d
```

### Manual

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
# Requires PostgreSQL 16+ with database `tachyon` and user `tachyon`
cargo run --bin tachyon-server
```

## Development

```bash
just build     # Build all crates
just test      # Run backend test suite
just lint      # rustfmt + clippy
just dev       # lint + test + build
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/documents` | List documents |
| `POST` | `/api/v1/documents` | Create document |
| `GET` | `/api/v1/documents/{id}` | Get document |
| `PUT` | `/api/v1/documents/{id}` | Update document |
| `DELETE` | `/api/v1/documents/{id}` | Delete document |
| `GET` | `/api/v1/documents/search` | Full-text search |
| `POST` | `/api/v1/documents/{id}/versions` | Create version |
| `GET` | `/api/v1/documents/{id}/attachments` | List attachments |
| `POST` | `/api/v1/documents/{id}/attachments` | Upload attachment |
| `GET` | `/api/v1/auth/status` | Auth status check |
| `POST` | `/api/v1/auth/login` | Login |
| `POST` | `/api/v1/auth/guest` | Guest login |
| `GET` | `/api/v1/users` | List users |
| `GET` | `/api/v1/teams` | List teams |
| `POST` | `/api/v1/roles` | Create role |
| `GET` | `/health` | Health check |
| `GET` | `/robots.txt` | SEO robots.txt |
| `GET` | `/sitemap.xml` | Dynamic sitemap |
| `GET` | `/docs/{id}` | SSR document page |
| `GET` | `/swagger-ui/` | API documentation |
| `WS` | `/ws` | Real-time collaboration |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TACHYON_HOST` | `0.0.0.0` | Server bind address |
| `TACHYON_PORT` | `8080` | Server port |
| `DATABASE_URL` | `postgres://...` | PostgreSQL connection string |
| `TACHYON_JWT_SECRET` | (required) | JWT signing secret (min 32 chars) |
| `TACHYON_CORS_ORIGINS` | `http://localhost:8080` | Comma-separated allowed origins; never use `*` in production |
| `TACHYON_GUEST_LOGIN_ENABLED` | `false` | Enable guest auto-login |
| `TACHYON_PUBLIC_NOTES_ENABLED` | `false` | Allow public document access |
| `TACHYON_SITE_TITLE` | `Tachyon` | Site title for SEO |
| `TACHYON_BASE_URL` | `http://localhost:8080` | Canonical base URL |
| `TACHYON_RATE_LIMIT_ENABLED` | `true` | Enable rate limiting |

## Crate Layout

| Crate | Description |
|-------|-------------|
| `tachyon-core` | Core types, domain models, shared utilities |
| `tachyon-server` | Axum 0.8 HTTP/2 server, middleware, API routes |
| `tachyon-database` | PostgreSQL layer via sqlx, migrations, repositories |
| `tachyon-renderer` | Markdown-to-HTML with extensions, syntax highlighting, TOC generation |
| `tachyon-search` | Full-text search indexing and querying (Tantivy, BM25) |
| `tachyon-rbac` | Role-based access control engine |
| `tachyon-frontend` | WASM web frontend (Leptos 0.8 + Trunk) |
| `tachyon-desktop` | Native desktop client (Tauri 2) |
| `tachyon-cli` | Command-line interface |
| `tachyon-storage` | Pluggable storage backends (SQLite, in-memory) |
| `tachyon-editor` | Native text editor engine with CRDT support (Yrs) |
| `tachyon-import-export` | Import/export for Docusaurus, Obsidian, Markdown, JSON, HTML |
| `tachyon-ssg` | Static site generator |
| `tachyon-plugin-runtime` | WASM plugin sandbox (Wasmtime) |
| `tachyon-testing` | Shared test utilities, fuzzing harnesses, benchmarks |
| `tachyon-benchmarks` | Criterion benchmark suite |

## Technology Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2024 Edition |
| Async Runtime | Tokio |
| HTTP Server | Axum 0.8 |
| Frontend | Leptos 0.8 (compiled to WASM) |
| Desktop | Tauri 2 |
| Database | PostgreSQL 16+ via sqlx (async) |
| Search | Tantivy (BM25 + TF-IDF); PostgreSQL `tsvector` fallback |
| Markdown | pulldown-cmark (SIMD-accelerated) |
| Math | KaTeX |
| Syntax Highlighting | tree-sitter (12 languages) + syntect |
| CRDT | Yrs (Y.js-compatible) |
| Plugin Runtime | Wasmtime (WASM sandbox) |
| Auth | JWT (HS256, jsonwebtoken) |
| CSS | Tailwind CSS (CDN) |
| Fonts | Inter, JetBrains Mono |

## License

Apache License, Version 2.0. See [LICENSE](LICENSE).

## Documentation

Hosted at [wyattau.github.io/Tachyon](https://wyattau.github.io/Tachyon).

Release history: [CHANGELOG.md](CHANGELOG.md).
