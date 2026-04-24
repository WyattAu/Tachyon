---
title: Architecture
description: System architecture and design decisions
order: 5
tags: [architecture, reference]
---

# Architecture

## System Overview

Tachyon is a full-stack Rust application:

```
┌─────────────────────────────────────────────────────┐
│                   Browser (WASM)                     │
│  ┌─────────────────────────────────────────────┐    │
│  │  Leptos 0.8 (CSR) + Tailwind CSS           │    │
│  │  ┌─────────────┐  ┌──────────────────────┐  │    │
│  │  │ Native Editor│  │ WebSocket (CRDT)     │  │    │
│  │  │ (ropey+yrs) │  │ (axum-tungstenite)  │  │    │
│  │  └─────────────┘  └──────────────────────┘  │    │
│  └─────────────────────────────────────────────┘    │
└──────────────────────┬──────────────────────────────┘
                       │ HTTP + WebSocket
┌──────────────────────┴──────────────────────────────┐
│                   Axum Server (:8080)                │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────┐   │
│  │ JWT Auth │ │ REST API │ │ WebSocket Handler │   │
│  └──────────┘ └────┬─────┘ └───────────────────┘   │
│                    │                                 │
│  ┌─────────────────┴────────────────────────────┐   │
│  │         Service Layer (routes/)              │   │
│  │  Documents, Search, Teams, SSG, Plugins...   │   │
│  └─────────────────┬────────────────────────────┘   │
│                    │                                 │
│  ┌────────┐ ┌─────┴──────┐ ┌──────────┐            │
│  │ Tantivy│ │ PostgreSQL │ │ Redis    │            │
│  │ Search │ │ (sqlx)     │ │ (cache)  │            │
│  └────────┘ └────────────┘ └──────────┘            │
└─────────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
tachyon-server
├── tachyon-database → sqlx → PostgreSQL
├── tachyon-core (shared types)
├── tachyon-search → tantivy
├── tachyon-ssg → tachyon-renderer
├── tachyon-rbac (role-based access control)
└── tachyon-auth → jsonwebtoken

tachyon-frontend (WASM)
├── leptos 0.8 + leptos_router
├── tachyon-editor → ropey + yrs
├── gloo-net (HTTP/WebSocket)
└── web-sys
```

## Key Design Decisions

### Why CSR (Client-Side Rendering)?

Tachyon uses Leptos in CSR mode because:
1. The editor needs direct DOM access for cursor management
2. Offline-first requires all logic in the browser
3. CRDT sync runs entirely client-side
4. No server-side rendering complexity for a desktop-first app

### Why Yrs (Yjs Rust Port)?

- Battle-tested CRDT implementation (used by Notion, Figma)
- Rust-native (no WASM bridge needed)
- Character-level conflict resolution
- Efficient binary encoding for network sync

### Why Axum?

- Tokio-native async runtime
- Tower middleware ecosystem
- WebSocket support via axum-tungstenite
- Extractor pattern for clean route handlers

### Why PostgreSQL?

- Full-text search with tsvector
- JSONB for flexible metadata
- Row-level security
- Mature tooling (sqlx, migrations)
