---
title: Welcome to Tachyon
description: A deterministic, high-performance knowledge management system built with Rust
order: -1
---

# Welcome to Tachyon

Tachyon is a deterministic, high-performance knowledge management system built entirely in Rust. It features real-time collaboration via CRDTs, a native editor, and offline-first architecture.

## Features

- **Markdown-first editing** with rich preview and KaTeX math
- **Real-time collaboration** with CRDT sync (Yrs/lib0)
- **Offline-first** -- works without network, syncs when connected
- **Knowledge graph** with bidirectional links
- **Full-text search** with Tantivy + PostgreSQL
- **Static site generation** for documentation
- **Plugin system** (WASM via Wasmtime 44)
- **Role-based access control** with teams, spaces, and organizations
- **MFA and OAuth2** authentication
- **Desktop app** (Tauri 2.x) and CLI

## Quick Start

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
cargo run --release -p tachyon-server
```

The server starts at `http://localhost:8080`.

## Architecture

Tachyon is built as a Rust workspace with 17 crates:

| Crate | Purpose |
|-------|---------|
| `tachyon-core` | Shared types, error handling, utilities |
| `tachyon-server` | Axum HTTP server with JWT auth, WebSocket, GraphQL |
| `tachyon-frontend` | Leptos 0.8 WASM frontend (CSR) |
| `tachyon-editor` | Native Rust editor with CRDT support |
| `tachyon-database` | PostgreSQL via sqlx with migration support |
| `tachyon-search` | Tantivy + PostgreSQL hybrid search |
| `tachyon-renderer` | Markdown to HTML with syntax highlighting, KaTeX, sanitization |
| `tachyon-rbac` | Custom RBAC engine |
| `tachyon-storage` | SQLite and in-memory storage backends |
| `tachyon-ssg` | Static site generator with i18n, RSS, sitemap |
| `tachyon-import-export` | Obsidian, Docusaurus, ZIP, JSON, HTML import/export |
| `tachyon-plugin-runtime` | WASM sandbox via Wasmtime 44 |
| `tachyon-cli` | Command-line interface |
| `tachyon-desktop` | Desktop client library |
| `tachyon-desktop-app` | Tauri 2.x desktop application |
| `tachyon-testing` | Shared test utilities, fixtures, fuzzing, benchmarks |
