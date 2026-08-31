# Server Component

The `tachyon-server` crate provides the HTTP/2 server implementation.

## Overview

Server implements:
- REST API endpoints
- WebSocket connections
- Authentication middleware
- Request routing

## Architecture

```
┌─────────────────────────────────────────────┐
│                 Axum Router                  │
│  ┌────────────────────────────────────────┐ │
│  │              Middleware Stack          │ │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ │ │
│  │  │   Auth   │ │  Logging │ │  CORS  │ │ │
│  │  └──────────┘ └──────────┘ └────────┘ │ │
│  └────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────┐ │
│  │              Route Handlers            │ │
│  │  /api/v1/documents                     │ │
│  │  /api/v1/search                        │ │
│  │  /api/v1/users                         │ │
│  │  /ws                                   │ │
│  └────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────┐ │
│  │              State Layer               │ │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ │ │
│  │  │   Repo   │ │ Renderer │ │ Search │ │ │
│  │  └──────────┘ └──────────┘ └────────┘ │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

## Key Types

### Server

```rust
pub struct Server {
    router: Router,
    state: Arc<AppState>,
    addr: SocketAddr,
}

pub struct AppState {
    pub repository: Arc<dyn Repository>,
    pub renderer: Arc<Renderer>,
    pub search: Arc<SearchEngine>,
    pub rbac: Arc<Rbac>,
    pub config: Config,
}
```

### Configuration

```rust
pub struct Config {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub tls: Option<TlsConfig>,
    pub auth: AuthConfig,
}
```

## REST API

### Document Endpoints

```rust
// Create document
pub async fn create_document(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, ApiError> {
    let doc = Document::new(payload.title, payload.content);
    state.repository.save(&doc).await?;
    Ok(Json(doc))
}

// Get document
pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<DocumentId>,
) -> Result<Json<Document>, ApiError> {
    state.repository.get(&id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}
```

### Route Registration

```rust
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/documents", post(create_document))
        .route("/api/v1/documents/:id", get(get_document))
        .route("/api/v1/documents/:id", put(update_document))
        .route("/api/v1/documents/:id", delete(delete_document))
        .route("/api/v1/search", get(search_documents))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}
```

## WebSocket

### Connection Handler

```rust
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut tx, mut rx) = socket.split();
    
    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let event: Event = serde_json::from_str(&text)?;
                handle_event(event, &mut tx, &state).await?;
            }
            Ok(Message::Close(_)) => break,
            Err(e) => error!("WebSocket error: {}", e),
        }
    }
}
```

### Event Types

```rust
pub enum Event {
    DocumentUpdate { id: DocumentId, content: String },
    CursorMove { position: Position },
    UserJoin { user: User },
    UserLeave { user_id: UserId },
}
```

## Middleware

### Authentication

```rust
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;
    
    let user = state.auth.verify_token(token).await?;
    req.extensions_mut().insert(user);
    
    Ok(next.run(req).await)
}
```

### Logging

```rust
pub async fn logging_middleware(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    info!("{} {} - {}ms", method, path, duration.as_millis());
    
    response
}
```

## Error Handling

```rust
#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Unauthorized,
    Forbidden,
    Validation(String),
    Internal(Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, &msg),
            ApiError::Internal(e) => {
                error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            }
        };
        
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

## Usage

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let state = AppState::new(config).await?;
    
    Server::bind(&addr)
        .serve(create_router(state).into_make_service())
        .await?;
    
    Ok(())
}
```
