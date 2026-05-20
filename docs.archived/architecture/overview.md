# Architecture Overview

High-level overview of Tachyon's system architecture.

## Design Principles

### Local-First

Tachyon operates on local files without requiring network connectivity. All data resides in the local Git repository, enabling:
- Offline operation
- Fast local search
- Version control integration
- Data sovereignty

### Just-In-Time Rendering

Unlike static site generators, Tachyon renders content on-demand:
- No build step
- Sub-15ms render latency
- Always up-to-date content
- External editor compatibility

### Hybrid Architecture

Single codebase supports multiple deployment modes:
- Desktop: Native application with local server
- Server: Headless HTTP/2 server
- Static: Export to static HTML

## System Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                           User Interface                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │
│  │  Desktop (Tauri)│  │  Web Browser    │  │  API Client         │  │
│  │  WebView        │  │  HTTP/WS        │  │  REST API           │  │
│  └────────┬────────┘  └────────┬────────┘  └──────────┬──────────┘  │
└───────────┼────────────────────┼──────────────────────┼─────────────┘
            │                    │                      │
            │ IPC                │ HTTP/2 + WebSocket   │ HTTP/2
            -                    -                      -
┌─────────────────────────────────────────────────────────────────────┐
│                        Presentation Layer                            │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                      Tachyon Core Server                         ││
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────────┐ ││
│  │  │  Router   │ │  Auth     │ │  Session  │ │  WebSocket      │ ││
│  │  │  (Axum)   │ │  Middleware│ │ Manager   │ │  Handler        │ ││
│  │  └───────────┘ └───────────┘ └───────────┘ └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
            │
            -
┌─────────────────────────────────────────────────────────────────────┐
│                        Processing Layer                              │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────────┐ ││
│  │  │ Renderer  │ │  Search   │ │  RBAC     │ │  Git            │ ││
│  │  │ (Markdown)│ │ (Tantivy) │ │  Engine   │ │  Operations     │ ││
│  │  └───────────┘ └───────────┘ └───────────┘ └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
            │
            -
┌─────────────────────────────────────────────────────────────────────┐
│                        Storage Layer                                 │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  ┌───────────┐ ┌───────────────────────────────────────────┐   ││
│  │  │  Database │ │            File System                     │   ││
│  │  │(PostgreSQL)│ │  ┌────────────────┐  ┌─────────────────┐  │   ││
│  │  │           │ │  │  Git Repository│  │  Search Index   │  │   ││
│  │  └───────────┘ │  └────────────────┘  └─────────────────┘  │   ││
│  │                └───────────────────────────────────────────┘   ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

## Component Interaction

### Document Read Flow

```
Request → Router → Auth → DocumentService → Repository → Cache
                                                    ↓
                                               Renderer
                                                    ↓
                                               Response
```

### Document Write Flow

```
Request → Router → Auth → Validation → DocumentService
                                           ↓
                                      Repository
                                           ↓
                                      File System
                                           ↓
                                      File Watcher
                                           ↓
                                      Search Index
                                           ↓
                                      WebSocket Broadcast
```

## Data Flow Patterns

### Request/Response

Synchronous HTTP requests for document operations:
- CRUD operations
- Search queries
- Configuration

### Event Streaming

WebSocket for real-time updates:
- Document changes
- User presence
- Collaboration events

### File Watching

Reactive updates on file changes:
- External editor edits
- Git operations
- Index updates

## Scalability

### Vertical

Single server handles:
- 1000+ concurrent users
- 100,000+ documents
- Sub-100ms search

### Horizontal

Stateless design enables:
- Load balancing
- Session affinity for WebSocket
- Shared storage (NFS, S3)

## Security Architecture

### Layers

1. **Transport**: TLS encryption
2. **Authentication**: Token-based (JWT, OAuth)
3. **Authorization**: RBAC
4. **Content**: Block-level redaction
5. **Audit**: Complete logging

### Data Protection

- No telemetry
- Local-first data storage
- Encryption at rest (optional)
- Secure secret management

## Performance Targets

| Metric | Target |
|--------|--------|
| Render latency | < 15ms |
| Search query | < 100ms |
| File watch response | < 50ms |
| WebSocket latency | < 10ms |
| Memory usage | < 100MB base |
| Startup time | < 2s |

> Targets derived from Criterion benchmarks in tachyon/crates/benchmarks/ and K6 load tests
> in tachyon/load-tests/. Run `cargo bench -p tachyon-benchmarks` to reproduce locally.

## Further Reading

- [Database Schema](database.md)
- [API Design](api.md)
- [WebSocket Protocol](websocket.md)
- [Security Architecture](security.md)
