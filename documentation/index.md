---
title: Welcome to Tachyon
description: A fast, offline-first knowledge management platform built with Rust
order: -1
---

# Welcome to Tachyon

Tachyon is a high-performance knowledge management platform built entirely in Rust. It features real-time collaboration via CRDTs, a native editor, and offline-first architecture.

## Features

- **Markdown-first editing** with rich preview
- **Real-time collaboration** with CRDT sync (Yrs/lib0)
- **Offline-first** — works without network, syncs when connected
- **Knowledge graph** with bidirectional links
- **Full-text search** with Tantivy + PostgreSQL
- **Static site generation** for documentation
- **Plugin system** for extensibility
- **Role-based access control** with teams and spaces

## Quick Start

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
cargo run --release -p tachyon-server
```

The server starts at `http://localhost:8080`.

## Architecture

Tachyon is built as a Rust workspace with the following crates:

| Crate | Purpose |
|-------|---------|
| `tachyon-server` | Axum HTTP server with JWT auth |
| `tachyon-frontend` | Leptos 0.8 WASM frontend (CSR) |
| `tachyon-editor` | Native Rust editor with CRDT support |
| `tachyon-core` | Shared types and utilities |
| `tachyon-database` | PostgreSQL via sqlx |
| `tachyon-search` | Tantivy + PostgreSQL hybrid search |
| `tachyon-ssg` | Static site generator |
| `tachyon-renderer` | Markdown to HTML renderer |
