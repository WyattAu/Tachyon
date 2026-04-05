# System Architecture

Comprehensive overview of Tachyon's system architecture.

## Overview

Tachyon is a hybrid documentation platform supporting both local-first desktop usage and centralized server deployment. The architecture prioritizes:

- **Performance**: Sub-15ms render latency
- **Reliability**: Local-first with optional sync
- **Security**: Defense-in-depth approach
- **Modularity**: Clear crate boundaries

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Desktop   │  │   Server    │  │    Web Frontend     │  │
│  │   (Tauri)   │  │   (Axum)    │  │     (Leptos)        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Processing Layer                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Renderer  │  │   Search    │  │     Git/Storage     │  │
│  │ (Markdown)  │  │ (Tantivy)   │  │     (git2-rs)       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Reactive Layer                            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              File Watcher (notify)                       ││
│  │         Cache Invalidation / Event Bus                   ││
│  └─────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│                    Runtime Layer                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Tokio Async Runtime                         ││
│  │    (IOCP / Kqueue / Epoll / io_uring)                   ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Core Crates

### tachyon-core

Core domain types and traits:

```rust
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub content: String,
    pub metadata: Metadata,
    pub status: DocumentStatus,
    pub visibility: Visibility,
}

pub trait Repository: Send + Sync {
    async fn get(&self, id: &DocumentId) -> Result<Option<Document>>;
    async fn save(&self, doc: &Document) -> Result<()>;
    async fn delete(&self, id: &DocumentId) -> Result<()>;
    async fn list(&self, filter: Filter) -> Result<Vec<Document>>;
}
```

### tachyon-server

HTTP/2 server with WebSocket support:

```rust
pub struct Server {
    router: Router,
    state: AppState,
}

impl Server {
    pub async fn start(self, addr: SocketAddr) -> Result<()> {
        axum::Server::bind(&addr)
            .serve(self.router.into_make_service())
            .await?;
        Ok(())
    }
}
```

### tachyon-desktop

Tauri-based desktop application:

```rust
#[tauri::command]
async fn open_document(id: String, state: State<'_, AppState>) -> Result<Document, String> {
    state.repository.get(&id.parse()?)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Document not found".to_string())
}
```

### tachyon-frontend

Leptos-based web components:

```rust
#[component]
pub fn DocumentEditor(cx: Scope, document: Document) -> impl IntoView {
    let content = create_rw_signal(cx, document.content);
    
    view! { cx,
        <div class="editor">
            <textarea 
                on:input=move |ev| content.set(event_target_value(&ev))
                prop:value=content
            />
            <DocumentPreview content=content />
        </div>
    }
}
```

### tachyon-renderer

Markdown to HTML rendering:

```rust
pub struct Renderer {
    cache: LruCache<DocumentHash, String>,
    options: RenderOptions,
}

impl Renderer {
    pub fn render(&self, markdown: &str) -> Result<RenderOutput> {
        let parser = Parser::new_ext(markdown, self.options.markdown_extensions);
        let mut html = String::new();
        html::push_html(&mut html, parser);
        Ok(RenderOutput { html })
    }
}
```

### tachyon-search

Full-text search with Tantivy:

```rust
pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    writer: Option<IndexWriter>,
}

impl SearchEngine {
    pub fn search(&self, query: &Query) -> Result<Vec<SearchHit>> {
        let searcher = self.reader.searcher();
        let query = self.parse_query(query)?;
        let hits = searcher.search(&query, &TopDocs::with_limit(20))?;
        Ok(hits.into_iter().map(|(score, doc)| {
            SearchHit { score, document: doc }
        }).collect())
    }
}
```

### tachyon-database

SQLite database operations:

```rust
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub async fn get_document(&self, id: &DocumentId) -> Result<Option<Document>> {
        let conn = self.pool.get()?;
        let stmt = conn.prepare_cached(
            "SELECT * FROM documents WHERE id = ?"
        )?;
        // ...
    }
}
```

### tachyon-rbac

Role-based access control:

```rust
pub struct Rbac {
    roles: HashMap<RoleName, Role>,
    user_roles: HashMap<UserId, Vec<RoleName>>,
}

impl Rbac {
    pub fn check_permission(
        &self,
        user: &UserId,
        resource: &Resource,
        action: &Action,
    ) -> bool {
        self.user_roles.get(user)
            .map(|roles| {
                roles.iter().any(|r| {
                    self.roles.get(r)
                        .map(|role| role.allows(resource, action))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }
}
```

## Data Flow

### Document Read Flow

```
User Request → Router → DocumentState → Repository → Cache/Filesystem
                    ↓
              Renderer (if needed)
                    ↓
              HTML Response
```

### Document Write Flow

```
User Request → Validation → DocumentState → Repository
                                        ↓
                                   Filesystem
                                        ↓
                                   File Watcher
                                        ↓
                                   Index Update
                                        ↓
                               WebSocket Broadcast
```

### Search Flow

```
User Query → SearchEngine → Tantivy Index
                                ↓
                         Ranked Results
                                ↓
                         Repository (fetch docs)
                                ↓
                         Search Response
```

## Concurrency Model

### Tokio Runtime

```
┌────────────────────────────────────────┐
│         Tokio Runtime (Multi-thread)    │
│  ┌──────────┐  ┌──────────┐            │
│  │ Worker 1 │  │ Worker 2 │  ...       │
│  │ (Steal)  │  │ (Steal)  │            │
│  └──────────┘  └──────────┘            │
│  ┌─────────────────────────────────┐   │
│  │      Work-Stealing Scheduler    │   │
│  └─────────────────────────────────┘   │
└────────────────────────────────────────┘
```

### Async Patterns

```rust
// Concurrent document loading
let docs: Vec<_> = futures::future::join_all(
    ids.iter().map(|id| repository.get(id))
).await;

// Streaming large results
let stream = repository.stream_all();
pin_mut!(stream);
while let Some(doc) = stream.next().await {
    process(doc?)?;
}
```

## Caching Strategy

### L1: In-Memory Cache

```rust
pub struct Cache<T> {
    data: DashMap<CacheKey, CacheEntry<T>>,
    max_size: usize,
    ttl: Duration,
}
```

### L2: Rendered HTML Cache

Documents cached after rendering:

```rust
pub struct RenderCache {
    cache: LruCache<ContentHash, RenderedDocument>,
    max_entries: usize,
}
```

### Cache Invalidation

File watcher triggers invalidation:

```rust
fn handle_file_event(event: Event) {
    match event.kind {
        EventKind::Modify(_) => {
            cache.invalidate(&event.paths);
            index.reindex(&event.paths);
        }
        EventKind::Create(_) => {
            index.add(&event.paths);
        }
        EventKind::Remove(_) => {
            cache.invalidate(&event.paths);
            index.remove(&event.paths);
        }
        _ => {}
    }
}
```

## Security Architecture

### Defense in Depth

1. **Memory Safety**: Rust ownership system
2. **Input Validation**: All inputs validated
3. **Access Control**: RBAC at multiple levels
4. **Encryption**: TLS for transport
5. **Audit Logging**: All access logged

### Authentication Flow

```
Client → Auth Provider → Token → Session → User Context
                                    ↓
                              RBAC Check
                                    ↓
                              Resource Access
```

### Permission Check

```rust
fn check_access(
    user: &User,
    document: &Document,
    action: Action,
) -> Result<()> {
    if !rbac.check(&user.id, &document.resource(), action) {
        return Err(Error::Forbidden);
    }
    if document.access == AccessLevel::Restricted {
        let groups = user.groups.intersection(&document.groups);
        if groups.is_empty() {
            return Err(Error::Forbidden);
        }
    }
    Ok(())
}
```

## Deployment Modes

### Desktop Mode

```
┌─────────────────────────────────┐
│         Desktop App             │
│  ┌───────────┐  ┌─────────────┐│
│  │  WebView  │  │ Rust Core   ││
│  │  (UI)     │←→│ (Tauri IPC) ││
│  └───────────┘  └─────────────┘│
│                       ↓         │
│              Local SQLite       │
│              Local Git Repo     │
└─────────────────────────────────┘
```

### Server Mode

```
┌─────────────────────────────────────┐
│            Load Balancer            │
└───────────────┬─────────────────────┘
                ↓
┌─────────────────────────────────────┐
│         Tachyon Server              │
│  ┌─────────────────────────────┐   │
│  │    Axum HTTP/2 Server       │   │
│  │    WebSocket Handler        │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │    Shared State             │   │
│  │    (Arc<Mutex<AppState>>)   │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
        ↓               ↓
   SQLite DB      Git Repository
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Render latency | < 15ms |
| Search query | < 100ms |
| File watch response | < 50ms |
| WebSocket update | < 10ms |
| Memory usage | < 100MB base |

## Further Reading

- [Database Schema](../architecture/database.md)
- [API Design](../architecture/api.md)
- [WebSocket Protocol](../architecture/websocket.md)
- [Security Architecture](../architecture/security.md)
