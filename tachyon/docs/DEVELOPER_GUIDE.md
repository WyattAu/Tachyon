# Tachyon Developer Guide

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Prerequisites](#prerequisites)
4. [Quick Start](#quick-start)
5. [Environment Variables](#environment-variables)
6. [Development Workflow](#development-workflow)
7. [Leptos 0.8 Patterns](#leptos-08-patterns)
8. [API Client Usage](#api-client-usage)
9. [Adding New Components](#adding-new-components)
10. [Testing](#testing)
11. [Docker Deployment](#docker-deployment)

---

## Project Overview

Tachyon is a knowledge management system built with Rust. It provides a collaborative document editing platform with real-time presence, version history, a service catalog, and full-text search.

**Tech stack:**

- **Frontend:** Leptos 0.8 (WASM, CSR mode), Tailwind CSS, gloo-net (HTTP/WebSocket)
- **Backend:** Axum 0.7, SQLx (PostgreSQL), Redis, Tantivy (search)
- **Build:** Trunk (WASM), Cargo (workspace)
- **Desktop (optional):** Tauri 2.0 beta
- **Testing:** cargo test, Playwright (E2E), testcontainers

---

## Architecture

Tachyon is a Cargo workspace with the following crates:

| Crate | Path | Description |
|-------|------|-------------|
| `tachyon-core` | `crates/core` | Shared domain types, error handling, business logic |
| `tachyon-database` | `crates/database` | PostgreSQL schema, migrations, repository layer |
| `tachyon-server` | `crates/server` | Axum HTTP server, REST API, WebSocket handler |
| `tachyon-frontend` | `crates/frontend` | Leptos 0.8 WASM frontend (CSR) |
| `tachyon-rbac` | `crates/rbac` | Role-based access control engine |
| `tachyon-search` | `crates/search` | Tantivy full-text search with faceted filtering |
| `tachyon-renderer` | `crates/renderer` | Markdown/HTML rendering pipeline |
| `tachyon-cli` | `crates/cli` | CLI tool for administrative tasks |
| `tachyon-desktop` | `crates/desktop` | Tauri desktop application wrapper |
| `tachyon-testing` | `crates/testing` | Shared test utilities, fixtures, mocks |

**Frontend module layout** (`crates/frontend/src/`):

```
lib.rs          -- App entry point, Router, theme management
api.rs          -- ApiClient (gloo-net HTTP), all REST endpoints
websocket.rs    -- WebSocketClient for real-time collaboration
types.rs        -- Shared types mirroring backend API responses
components/     -- Reusable UI components
pages/          -- Route-level page components
styles.rs       -- Global CSS/Tailwind style injection
```

**Communication flow:**

```
Browser (WASM) --HTTP/JSON--> Axum Server --SQLx--> PostgreSQL
Browser (WASM) --WebSocket--> Axum Server
                     |
                  Redis (caching/sessions)
                  Tantivy (search index)
```

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | nightly (1.75+) | `rustup default nightly` |
| Trunk | latest | `cargo install trunk` |
| wasm-pack | latest | `cargo install wasm-pack` |
| PostgreSQL | 16+ | Local or via Docker |
| Redis | 7+ | Local or via Docker |
| Node.js | 18+ | Required for Playwright E2E tests |
| Playwright | latest | `npx playwright install` (after adding E2E) |

---

## Quick Start

### 1. Clone and build

```bash
git clone <repo-url> tachyon
cd tachyon/tachyon

# Build all crates (ensures compilation)
cargo build
```

### 2. Start infrastructure (Docker)

```bash
# Start PostgreSQL and Redis only
docker compose up -d postgres redis
```

### 3. Run the backend

```bash
# From tachyon/tachyon/
export DATABASE_URL="postgresql://tachyon:tachyon@localhost:5432/tachyon"
export TACHYON_JWT_SECRET="dev-secret-change-me"
export TACHYON_HOST="0.0.0.0"
export TACHYON_PORT="8080"
cargo run -p tachyon-server
```

The backend API will be available at `http://localhost:8080/api/v1`.

### 4. Run the frontend dev server

```bash
# From tachyon/tachyon/
trunk serve --open --port 8081
```

The frontend dev server will be available at `http://localhost:8081`. It proxies API calls to `http://localhost:8080`.

### 5. Run E2E tests

```bash
# Ensure backend + frontend are running, then:
npx playwright test
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | -- | PostgreSQL connection string (required) |
| `TACHYON_REDIS_URL` | `redis://localhost:6379` | Redis connection string |
| `TACHYON_HOST` | `0.0.0.0` | Server bind address |
| `TACHYON_PORT` | `8080` | Server bind port |
| `TACHYON_BIND_ADDRESS` | -- | Alternative: `host:port` combined |
| `TACHYON_JWT_SECRET` | -- | Secret for JWT token signing (required) |
| `TACHYON_CORS_ORIGINS` | `http://localhost:8080` | Comma-separated allowed CORS origins |
| `RUST_LOG` | `info` | Log level filter (e.g., `debug`, `tachyon=trace`) |

---

## Development Workflow

### Checking compilation

```bash
# Check all workspace crates
cargo check

# Check frontend only
cargo check -p tachyon-frontend --target wasm32-unknown-unknown
```

### Building the frontend WASM

```bash
# Production build
trunk build --release

# Output goes to dist/
```

### Running tests

```bash
# All unit tests across workspace
cargo test

# Specific crate
cargo test -p tachyon-frontend

# With output
cargo test -- --nocapture
```

### Full Docker development stack

```bash
# Start everything (postgres, redis, backend, frontend)
docker compose up --build

# Stop
docker compose down

# Stop and remove volumes
docker compose down -v
```

---

## Leptos 0.8 Patterns

This project targets **Leptos 0.8** in **CSR (Client-Side Rendering)** mode. Key patterns used throughout:

### Signals

Signals are the reactive state primitive. In Leptos 0.8, `signal()` returns a tuple of `(ReadSignal, WriteSignal)`:

```rust
let (count, set_count) = signal(0);

view! {
    <p>{count}</p>
    <button on:click=move |_| set_count.update(|n| *n += 1)>"+1"</button>
}
```

For read-write signals, use `RwSignal`:

```rust
let content = RwSignal::new(String::new());
content.set("hello".to_string());
let current = content.get();
```

### Resources (Async Data)

Use `LocalResource` for async data fetching in CSR mode (no server function integration):

```rust
let data = LocalResource::new(move || {
    let client = api_client.clone();
    async move {
        client.list_documents(None, None).await.unwrap_or_default()
    }
});

view! {
    <Suspense fallback=view! { <p>"Loading..."</p> }>
        {move || data.get().map(|docs| /* render docs */)}
    </Suspense>
}
```

### Callbacks

Leptos 0.8 `Callback` wraps closures for inter-component communication:

```rust
#[component]
fn MyComponent(on_action: Callback<String>) -> impl IntoView {
    view! {
        <button on:click=move |_| on_action.run("clicked".to_string())>
            "Do something"
        </button>
    }
}
```

### Suspense

Wrap async resources with `<Suspense>` to show a loading fallback:

```rust
view! {
    <Suspense fallback=view! { <LoadingSpinner/> }>
        {move || resource.get().map(|data| view! { /* content */ })}
    </Suspense>
}
```

### For (Keyed Lists)

Use `<For>` for efficient keyed iteration:

```rust
view! {
    <For
        each=move || items.get()
        key=|item| item.id.clone()
        let:item
    >
        <div>{item.name}</div>
    </For>
}
```

### view! Macro Rules

- HTML elements use lowercase: `<div>`, `<p>`, `<button>`
- Components use PascalCase: `<MyComponent prop=value />`
- Dynamic attributes: `class=format!("...")`, `class=move || { ... }`
- Event handlers: `on:click=move |_| { ... }`
- `Children` props are accessed via `{children()}` inside the view
- Use `into_any()` to return different view types from conditional branches

### Effects

`Effect::new()` runs side effects reactively:

```rust
Effect::new(move || {
    let value = some_signal.get();
    web_sys::console::log_1(&format!("Changed: {}", value).into());
});
```

---

## API Client Usage

The `ApiClient` (in `crates/frontend/src/api.rs`) wraps all HTTP communication with the backend using `gloo-net`.

### Creating a client

```rust
use tachyon_frontend::api::ApiClient;

// Default: connects to http://localhost:8080/api/v1
let client = ApiClient::default();

// Custom base URL
let client = ApiClient::new("https://api.example.com/api/v1");
```

### Authentication flow

```rust
let client = ApiClient::default();

// Login
let response = client.login("username", "password").await?;
if response.success {
    if let Some(token) = response.access_token {
        client.set_auth_token(token);
    }
}

// Guest auto-authentication
let authenticated = client.auto_authenticate_guest().await?;

// Check status
let status = client.auth_status().await?;

// Logout
client.logout().await?;
client.clear_auth_token();
```

### Error handling

All API methods return `Result<T, ApiError>`:

```rust
match client.get_project("some-id").await {
    Ok(project) => { /* use project */ }
    Err(ApiError::Network(msg)) => { /* network failure */ }
    Err(ApiError::Api(msg)) => { /* server returned error */ }
    Err(ApiError::NotFound(msg)) => { /* 404 */ }
    Err(ApiError::Serialization(msg)) => { /* parse error */ }
}
```

### Available API methods

| Category | Methods |
|----------|---------|
| **Auth** | `login`, `guest_login`, `guest_status`, `auto_authenticate_guest`, `auth_status`, `logout` |
| **Catalog** | `get_catalog_stats`, `list_projects`, `get_project`, `get_project_by_slug`, `create_project`, `update_project`, `delete_project`, `list_project_components`, `list_project_members` |
| **Documents** | `list_documents` |
| **Versions** | `list_versions`, `get_version`, `create_version` |
| **Attachments** | `list_attachments`, `upload_attachment`, `delete_attachment` |
| **Templates** | `list_templates`, `get_template`, `create_template`, `list_template_categories` |
| **Search** | `search`, `global_search` |
| **Saved Search** | `create_saved_search`, `list_saved_searches`, `get_saved_search`, `update_saved_search`, `delete_saved_search` |

### WebSocket client

```rust
let ws = client.websocket();
ws.connect();
ws.join_document("doc-id", "user-id", "User Name")?;
ws.send_edit("doc-id", "user-id", operation_value)?;
ws.leave_document("doc-id", "user-id");
```

---

## Adding New Components

### Step-by-step example

#### 1. Create the component file

Create `crates/frontend/src/components/my_feature.rs`:

```rust
use leptos::prelude::*;

#[component]
pub fn MyFeature(
    /// The ID of the resource to display
    resource_id: String,
    /// Optional callback when user interacts
    #[prop(optional)]
    on_select: Option<Callback<String>>,
) -> impl IntoView {
    let (selected, set_selected) = signal(None::<String>);

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                    "My Feature"
                </h3>
            </div>
            <div class="p-4">
                <p class="text-gray-600 dark:text-gray-300">"Resource: "{resource_id}</p>
            </div>
        </div>
    }
}
```

#### 2. Register in the components module

Edit `crates/frontend/src/components/mod.rs`:

```rust
pub mod my_feature;
pub use my_feature::MyFeature;
```

#### 3. Use in a page

```rust
use crate::components::MyFeature;

#[component]
fn SomePage() -> impl IntoView {
    view! {
        <MyFeature resource_id="abc123".into() />
    }
}
```

#### 4. Check compilation

```bash
cargo check -p tachyon-frontend --target wasm32-unknown-unknown
```

---

## Testing

### Unit tests

```bash
# Run all workspace tests
cargo test

# Run specific crate tests
cargo test -p tachyon-frontend

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_permission_level -- --nocapture
```

Frontend unit tests use `wasm-bindgen-test` for WASM-compatible tests and standard `#[cfg(test)]` modules for non-WASM logic (like the role/permission tests in `role_badge.rs`).

### E2E tests (Playwright)

E2E tests live outside the Rust workspace and require a running server:

```bash
# Start backend + frontend
docker compose up -d

# Run E2E tests
npx playwright test

# Run with UI
npx playwright test --ui

# Run specific test file
npx playwright test docs.spec.ts
```

### WASM tests

```bash
# Requires wasm-pack
wasm-pack test --chrome -p tachyon-frontend
```

---

## Docker Deployment

### Development (`docker-compose.yml`)

Starts PostgreSQL, Redis, backend, and frontend:

```bash
docker compose up --build
```

- Frontend: `http://localhost:8080`
- Backend API: `http://localhost:3000/api/v1`
- PostgreSQL: `localhost:5432` (user/pass/db: `tachyon`)
- Redis: `localhost:6379`

### Production (`docker-compose.prod.yml`)

Extends the base compose with production overrides:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

Production overrides add:
- Resource limits (CPU/memory)
- Health checks with proper intervals
- Log rotation (`json-file`, size/file limits)
- Redis authentication and AOF persistence
- Environment variables from `.env` file
- 2 backend replicas with restart policies
- Custom network subnet

Required `.env` variables for production:

```env
POSTGRES_USER=tachyon
POSTGRES_PASSWORD=<strong-password>
POSTGRES_DB=tachyon
REDIS_PASSWORD=<strong-password>
JWT_SECRET=<strong-jwt-secret>
CORS_ORIGINS=https://your-domain.com
REGISTRY=ghcr.io
IMAGE_NAME=WyattAu/Tachyon
VERSION=latest
```
