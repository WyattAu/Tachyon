# Changelog

All notable changes to Tachyon are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [10.0.0] - 2026-05-07

### Added

**Real-Time Collaboration**
- Yjs CRDT sync with y-prosemirror binding, collaborative cursors, and awareness protocol (v0.26)
- WebSocket-based CRDT relay with room-scoped broadcast and binary message routing (v0.26, v0.28.1)
- CRDT collaboration toggle in editor settings with auto-connect and presence indicators (v0.28)
- Selection sharing via WebSocket message broadcasting with cursor position sync (v0.40)
- WebSocket reconnect with exponential backoff (1s base, 30s cap, 5 attempts) (v0.40)
- Stale WebSocket connection cleanup with periodic tasks every 60s (v0.49)

**Offline-First & Desktop**
- SQLite storage backend (SqliteStore) with WAL mode, FTS5, full CRUD, search, and pagination (v0.25)
- Sync queue for offline mutations with pending/in_flight/synced/failed state machine (v0.25)
- Tauri desktop app wired with Leptos WASM frontend, system tray, and file watcher sync (v0.20, v0.21)
- Embedded Axum server for single-process desktop mode binding to 127.0.0.1:0 (v0.23)
- Online/offline detection with connection status tracking (v0.25)

**Knowledge Graph**
- Graph engine with PostgreSQL persistence, typed nodes/edges, and auto-extraction pipeline (v0.3)
- Recursive CTE neighbor traversal (depth-limited to 5), BFS shortest path, connected components (v0.3)
- Temporal graph with `deactivated_at` timestamps, point-in-time queries, and graph diff (v0.8)
- SVG force-directed graph visualization with drag, hover highlighting, and click-to-inspect (v0.14)
- Tags browser with tag cloud, search/filter, and document cards per tag (v0.24)
- 23 property-based graph invariant tests with proptest (v0.6)

**Editor & Documents**
- ProseMirror rich text editor with formatting toolbar, auto-save (3s debounce), and live preview (v0.23)
- Wikilink `[[target]]` and `[[target|display]]` preprocessing with code block preservation (v0.15)
- Backlinks endpoint with GIN-indexed JSONB containment query and sidebar tab (v0.15)
- Wikilink autocomplete plugin for ProseMirror with debounced search and keyboard navigation (v0.24, v0.42)
- Document review workflow with state machine, auto-versioning, and server-side LCS diff (v0.10)
- Conflict resolution with OT (char-level UTF-8 safe), three-way merge via diff3, and side-by-side UI (v0.11)
- 15 OT edge case tests including delete-delete transform/compose and TP1 convergence property (v0.51)
- Image preview modal with zoom in/out/reset, scroll-wheel zoom, and keyboard close (v0.42)
- Command palette (Cmd+K) with search and keyboard navigation (v0.16)
- Table of contents with heading extraction, slug generation, and indented nav (v0.16)
- Drag-and-drop file upload with DropZone and UploadProgress components (v0.42)

**Spaces & Organization**
- Spaces with hierarchical tree navigation (parent/child), icon/color selectors, and member management (v0.30)
- 13 REST endpoints for space CRUD, member management, and document moves (v0.30)
- Auto-create personal space trigger on user registration (v0.30)

**Plugin System**
- Plugin system with DB registry, manifest (JSONB), runtime types, and enable/disable toggle (v0.29)
- WASM plugin execution endpoint (`POST /plugins/invoke`) wired to real PluginRuntime (v0.29)
- Plugin sandbox with path validation, WASM loading, and 29 runtime tests (v0.29, v0.49)

**Authentication & Security**
- MFA/2FA with TOTP (Google Authenticator compatible), backup codes (10 single-use), and OTPAuth URI (v0.42)
- OAuth2 CSRF protection with 32-byte cryptographic random state nonce, single-use with 10-min TTL (v0.53)
- Brute-force login protection with progressive backoff tiers (5→1min, 10→5min, 20→15min, 50→1hr, 100→24hr) (v0.53)
- Password strength validation (8+ chars, upper, lower, digit) (v0.35)
- JWT token refresh with refresh_tokens table, validation, revocation, and cleanup (v0.40)
- Google and GitHub OAuth2 authorization code flow (v0.27)

**Search & Indexing**
- Tantivy full-text search integration with BM25 ranking, auto-sync on document changes, and PG fallback (v0.5)
- Real prefix-based search suggestions using Tantivy PhrasePrefixQuery (v0.40)
- Shared Tantivy IndexWriter via `Arc<Mutex<Writer>>` eliminating 50MB per-document allocation (v0.50)
- Concurrent search facets using `tokio::join!` (4 queries in parallel) (v0.50)

**SSG & Content**
- Static site generator with incremental builds via SHA-256 content hash manifest (v0.4)
- Watch mode with file watching (notify crate), 500ms debounce, and concurrent processing (v0.4)
- SSG template customization with minijinja, file-based loading, and three embedded defaults (v0.11)
- SSG CLI binary (`tachyon-ssg-cli`) for markdown to static HTML conversion (v0.31)
- JSON export with metadata and pretty-print options (v0.40)

**Import/Export**
- Obsidian vault import with YAML frontmatter, wikilinks, inline tags, and callout blocks (v0.19)
- Markdown ZIP import/export with frontmatter parsing (v0.19)
- HTML export with responsive CSS, dark mode, index page, and ZIP archive (v0.19)

**Notifications & Activity**
- Notification system with create/list/unread_count/mark_read/mark_all_read (v0.13)
- Activity feed with event types (created, updated), filtering, and relative timestamps (v0.12, v0.13)
- Email notification system with template rendering (`{{variable}}` substitution) and fire-and-forget delivery (v0.40)
- Review events emit notifications (create/approve/reject/comment) (v0.13)

**Billing**
- Plan upgrade/downgrade with proration calculation and plan transition validation (free→pro→team→enterprise) (v0.40)
- Real usage metrics (document count, storage, spaces, team members) (v0.40)

**User Experience**
- Profile page with avatar, display name, bio, timezone, MFA status, and danger zone (v0.42)
- Theme toggle (light/dark/system) with localStorage persistence and system preference listener (v0.42)
- Mobile responsiveness: ResponsiveContainer, MobileNav drawer, 44px touch targets, responsive tables (v0.42)
- User avatar component with image URL or deterministic initials fallback (8-color palette) (v0.40)
- Breadcrumbs component with linked/unlinked items (v0.16)
- Image embedding fix — ammonia sanitizer now allows `<img>` tags (v0.16)
- Print styles with `@media print` (v0.16)

**Settings & Configuration**
- Settings page with tabbed layout (Profile, Preferences, Account, Danger Zone) (v0.33)
- Collaboration settings section (cursor sharing, presence, auto-connect toggles) (v0.28)
- Connected accounts section (Google + GitHub OAuth2 provider cards) (v0.28)
- Settings persistence with user_preferences table and GET/PUT `/auth/me` (v0.14)
- Onboarding wizard (4-step: Welcome → Display Name → Workspace Name → Done) (v0.28)
- `.env.example` documenting all 52 environment variables with categories (v0.43)
- Config validation with DB URL format, JWT secret length, CORS origin, and log level checks (v0.51)

**Internationalization**
- 8 locales (EN, ZH, JA, DE, FR, ES, KO, PT) with reactive language switching (v0.31)
- RTL support for Arabic/Hebrew/Farsi documents with `dir="rtl"` attribute (from v5.0.0 SSG)
- Per-language sitemaps with `xhtml:link` alternates and RSS feeds (from v5.0.0 SSG)

**Accessibility (WCAG 2.1 AA)**
- SkipLinks component (skip to main content, skip to navigation) (v0.41)
- FocusTrap, KeyboardShortcut, VisuallyHidden, LiveRegion components (v0.41)
- ARIA tab pattern with arrow key navigation (settings, document sidebar, editor panels) (v0.49, v0.51)
- ARIA dialog pattern with FocusTrap on command palette, search panel, settings, modals (v0.49, v0.51)
- `aria-hidden="true"` on 23+ decorative SVGs across 7 components (v0.48, v0.50)
- `aria-haspopup`, `aria-expanded`, `aria-controls` on dropdown menus (v0.48)
- `role=switch`, `aria-checked`, `aria-label` on ToggleSwitch (v0.48)
- Form label associations (`id`/`for` pairs) and color contrast fixes (v0.48)
- Focus-visible indicators (2px blue outline) and `prefers-reduced-motion` support (v0.41)
- ARIA labels on editor toolbar (`role=toolbar`), search (`role=search`), document tree (`role=tree`) (v0.41)

**Cloud Storage**
- StorageBackend trait with `put`, `get`, `delete`, `exists`, `presigned_url` (v0.42)
- LocalStorage implementation with path traversal prevention and directory creation (v0.42)
- S3Storage stub ready for aws-sdk-s3 integration (v0.42)

### Changed

- Server now serves frontend static assets via `tower-http ServeDir` with SPA routing fallback (v0.52)
- Rate limiter rewritten from `RwLock<HashMap>` to lock-free `DashMap` with per-key sharding (v0.32)
- Unified `ServerError` type with `IntoResponse` for Axum and `From` impls for all error types (v0.32)
- `AppState` extracted from 23-element tuple into named struct (v0.37)
- CRDT protocol switched from custom JSON wire format to y-websocket binary relay (v0.28.1)
- CLI `serve` now delegates to real `tachyon_server::build_server()` instead of 2-route stub (v0.22)
- CLI `gui` launches actual Tauri process instead of stub (v0.22)
- Tauri loads full Leptos WASM frontend instead of old htmx shell (v0.21)
- Graph BFS shortest path: N+1 sequential queries replaced with single recursive CTE (v0.50)
- `HashMap` replaced with `BTreeMap` in 17+ serialized structs for deterministic JSON output (v0.35)
- 20+ staging-gated features promoted to stable (editor settings, presence indicators, previews) (v0.40)
- CI workflows pruned from 15 to 6 (removed 9 broken template workflows) (v0.31)
- `config.rs` and `audit.rs` switched to `BTreeMap` for deterministic serialization (v0.32)
- `#[doc(hidden)]` added to 4 glob re-exports and RBAC re-exports (v0.32)
- `pub(crate)` added to 20+ internal items across 6 crates (v0.35)
- 82 `#[allow(dead_code)]` annotations cleaned up (reduced from 92 to ~20) (v0.43)
- 6 stale test files deleted (duplicated coverage already in integration suite) (v0.45)
- `ErrorResponse.details` changed from `HashMap` to `BTreeMap` (v0.36)
- Integration tests made fully parallel-safe (removed all 53 `TRUNCATE CASCADE` teardown calls) (v0.47)
- CI no longer requires `--test-threads=1` (tests natively parallel-safe) (v0.47)
- Document routes split from 1,751-line file into document_crud, document_versions, document_attachments modules (v0.37)
- API client split from 1,401-line file into auth, documents, spaces, search, files, collaboration, teams modules (v0.37)
- SSG lib split from 1,327-line file into build, render, rss, sitemap, i18n, manifest modules (v0.37)

### Fixed

- Guaranteed deadlock in `leave_document` (tokio::RwLock double-acquire) — standardized lock ordering (v0.35)
- CORS wildcard silently allowing all origins — now errors in production (v0.53)
- SQL injection in SSG via `format!` interpolation — replaced with bind params (v0.35)
- 5 webhook delivery sites silently dropping futures — converted to `tokio::spawn` (v0.36)
- 3 dead notification futures — converted to `tokio::spawn` (v0.36)
- Blocking `Path::canonicalize()` in 6 async file handlers — deferred to blocking thread (v0.48)
- Blocking `std::fs` in async context — replaced with `tokio::fs` (v0.35)
- WASM mutex poisoning in `ApiClient` — added `unwrap_or_else` on poisoned lock (v0.32)
- `HeaderValue` unwrap in rate limit headers — added safe fallback (v0.32)
- HMAC `expect` with safe fallback pattern (v0.32)
- Prometheus handler double-install panic on 2nd request (v0.32)
- `CREATE EXTENSION` race condition — added `DO $$ BEGIN ... EXCEPTION WHEN duplicate_object THEN NULL END $$` (v0.44)
- `UserRepository` RETURNING clause missing `totp_*` columns (v0.44)
- `UserRepository` delete failing on FK constraint — added CASCADE (v0.44)
- `TeamRepository` UUID type mismatch between `TeamRecord` and `TeamMemberRecord` (v0.44)
- `SessionRepository` UUID binding — added `parse_uuid` helper (v0.44)
- CRDT WebSocket protocol conversion (`http://` vs `ws://`) (v0.28.1)
- CRDT `onChange` callback lost during collaboration setup (v0.28.1)
- String injection vulnerability in JS eval (escaped single quotes) (v0.28.1)
- 19 production `unwrap`/`expect` calls eliminated across 5 crates (search, database, storage, core, renderer) (v0.55)
- 18 `.unwrap()` calls replaced with safe error handling in frontend components (v0.18)
- Rate limiter `InMemoryStore` unbounded memory growth — added cleanup (v0.48)
- File walk unbounded — capped to 10,000 files (v0.48)
- `unwrap()` on `Response::builder()` in files handler (v0.51)
- CSP `HeaderValue::from_str().unwrap()` — replaced with `if let Ok` pattern (v0.36)
- IndexedDB `request.result().unwrap()` — replaced with graceful degradation (v0.36)
- 3 `window.local_storage().unwrap()` — replaced with graceful degradation (v0.36)
- CORS disabled mode being MORE permissive than enabled mode (v0.35)
- Auth bypass from blanket prefix matching — narrowed to explicit route list (v0.35)
- Auth error responses leaking information — unified to generic 401 (v0.35)
- JWT secret validation was a warning — now fails fast at startup (v0.35)
- 2 unsafe blocks in WASM (`Uint8Array::view`) — replaced with safe copies (v0.35)
- `thiserror` version mismatch between renderer (v2) and workspace (v1) (v0.36)
- `.html` incorrectly included in `ALLOWED_EXTENSIONS` (v0.36)
- `orphaned code block` compilation error in document_editor.rs (v0.28)
- 100+ clippy auto-fixes (needless borrows, redundant closures, unsigned_abs, etc.) (v0.35)
- 20+ clippy fixes (clone_on_copy, dead_code, unused imports) (v0.40)
- All-features compilation failures with staging feature-gated code (v0.54)
- Doc test referencing nonexistent `run_server()` — updated to use actual public API (v0.54)
- Flaky `test_table` test (replaced `contains()` with `find()`) (v0.43)
- Unclosed brace in rbac `role.rs` from Default derive refactor (v0.43)
- N+1 queries in organization and space list endpoints (v0.35, v0.36)
- Dead Redis rate limiter stub silently disabling rate limits (v0.32)

### Security

- OAuth2 CSRF protection: cryptographic random state nonce (32 bytes hex), DashMap storage, single-use validation, 10-min TTL (v0.53)
- XSS sanitization proof: 6 tests demonstrating ammonia protection against script, iframe, event handlers, javascript: URIs, SVG onload (v0.53)
- Brute-force login protection: per-IP `LoginAttemptTracker` with progressive backoff (1min → 24hr), success resets (v0.53)
- CORS wildcard now errors in production (was warning), still warns in development (v0.53)
- Production CSP: strict `script-src`, `frame-ancestors none`, `upgrade-insecure` (v0.41)
- Development CSP: relaxed for localhost debugging (v0.41)
- HSTS: `max-age=31536000`, `includeSubDomains`, `preload` (v0.41)
- `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` (v0.41)
- `Referrer-Policy: strict-origin-when-cross-origin` (v0.41)
- `Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=()` (v0.41)
- Cross-Origin headers: COEP credentialless, COOP same-origin, CORP same-origin (v0.41)
- Password strength validation enforced (8+ chars, upper, lower, digit) (v0.35)
- Auth error responses unified (no information leakage on failed login) (v0.35)
- JWT secret validation enforced at startup (fail-fast, not warning) (v0.35)
- Auth bypass narrowed from blanket prefix to explicit public route list (v0.35)
- TrueLayer webhook HMAC-SHA256 signature verification (v0.36)
- Content-type validation on file upload (16 allowed extensions) (v0.40)
- Path traversal prevention with UUID-safe filenames (v0.40)
- Per-route body limits (1MB general, 50MB uploads, 1GB WebSocket) (from v5.0.0)
- Audit logging middleware for all authenticated requests (from v5.0.0)
- Request ID tracking with UUID per request and X-Request-Id header (from v5.0.0)
- Explicit rate limits on auth endpoints (register: 3/min, refresh: 10/min, password-reset: 3/hr) (v0.41)
- Vulnerability reduction from 24 → 1 transitive (only `rsa` from `sqlx-mysql` remains) (v0.46, v0.47)
- 2 WASM `unsafe` blocks eliminated (v0.35)
- Only 1 `unsafe` block in entire codebase (test env `set_var`) (v0.45)

### Infrastructure

- Docker multi-stage Dockerfile with cargo-chef dependency caching (v0.41)
- `.dockerignore` (exclude .git, target, node_modules, flake) (v0.41)
- `docker-compose.yml` with app + PostgreSQL + Redis healthchecks (v0.41)
- CI Docker build with BuildKit caching and deploy workflow (GHCR push + semantic tags) (v0.41)
- GitHub Actions E2E workflow with Playwright (v0.52)
- `e2e.sh` runner script (prerequisites check, build, start, test, cleanup) (v0.52)
- GitHub Pages auto-deploy workflow for documentation (v0.31)
- CI clippy scope expanded to `--all-targets` (was excluding frontend/testing) (v0.45)
- CI `--test-threads=1` removed (tests now natively parallel-safe) (v0.47)
- GitHub Actions workflows pruned from 15 to 6 (removed broken template workflows) (v0.31)
- Graceful shutdown with SIGINT/SIGTERM → 30s drain (v0.41)
- Compression middleware (gzip + brotli + deflate) (v0.41)
- Static asset serving via `tower-http ServeDir` with SPA routing fallback (v0.52)
- Cache headers middleware (WASM/JS/CSS: 1yr immutable, HTML: no-cache, images: 1yr) (v0.41)
- In-memory API cache with BTreeMap, configurable TTL, and prefix invalidation (v0.41)
- WASM preload + DNS prefetch in index.html (v0.41)
- Per-route rate limiting rules (auth: 3-5/min, API: 100/min) (v0.41)
- Request size limit (10MB default) (v0.41)
- Structured JSON logging (`LOG_FORMAT=json`, `RUST_LOG` env var) (v0.27, v0.41)
- Unique request ID per request with x-request-id header (v0.41)
- Slow request detection (>1s warning) and error request logging (v0.41)
- Health endpoint checking database + Redis connectivity with version and uptime (v0.41)
- Readiness probe endpoint (503 if DB down) (v0.41)
- Prometheus metrics endpoint (`/metrics`) with request counts, duration, uptime (v0.27, v0.41)
- Playwright E2E test infrastructure with AppPage helper (register, login, create doc, search, logout) (v0.39)
- Criterion benchmarks crate for search, renderer, and RBAC (v0.39)
- Lean4 formal verification specs (OT convergence, review state machine, graph invariants, diff3 merge) (v0.17)

### Dependencies

- wasmtime 18 → 44 (sandbox.rs rewrite for WasiP1Ctx API) (v0.46)
- wasmtime 44.0.0 → 44.0.1 (RUSTSEC-2026-0114 table allocation panic) (v0.47)
- git2 0.18 → 0.20 across workspace (v0.46)
- rustls-webpki 0.103.9 → 0.103.13 (CRL/wildcard vulnerability fixes) (v0.46)
- thin-vec 0.2.14 → 0.2.16 (RUSTSEC-2026-0103 use-after-free) (v0.47)
- lz4_flex 0.11.5 → 0.11.6 (yanked version fix via tantivy) (v0.46)
- zip v2, serde_yaml v0.9, walkdir, regex, uuid (import-export crate) (v0.19)
- deadpool-redis (Redis rate limiting) (v0.38)
- proptest (property-based graph invariant tests) (v0.6)
- y-websocket, y-prosemirror (CRDT collaboration) (v0.26)
- minijinja (SSG template customization) (v0.11)
- tauri-plugin-notification (desktop notifications) (v0.20)
- metrics + metrics-exporter-prometheus (Prometheus integration) (v0.27)
- tower-http ServeDir (static file serving) (v0.52)
- ammonia (XSS sanitization in markdown renderer) (v0.53)
- web-sys (Window, Storage, File, KeyboardEvent, MouseEvent for editor) (v0.9)

### Documentation

- 211 doc comments across database and server crates (template, notification, saved_search, webhook, onboarding, plugin, organization, document_version, user, and all 18 route handler files) (v0.51)
- 54 doc comments on frontend API client (auth, documents, search, spaces, teams, billing, files methods) (v0.49)
- 39 doc comments on database repositories (graph, space, team, permissions, comment, document_review, billing) (v0.48)
- 30 doc comments on server route handlers (document CRUD, user auth, sessions, spaces, search, graph nodes) (v0.50)
- Crate-level doc comments on 6 crates (v0.35)
- `.env.example` with 120 lines documenting all 52 environment variables organized by category (v0.43)
- `REMAINING_ITEMS.md` with 48 items across 10 categories (infrastructure, features, testing, docs, a11y, perf, security, tech debt, ops) (v0.52)
- 7 documentation pages (getting-started, editor-guide, api-reference, etc.) (v0.31)
- Competitive analysis of 14 codebases with honest assessment (v0.31)
- README rewrite with accurate features, quick start, config table, crate listing, formal verification section (v0.17)

### Performance

- Graph BFS shortest path: N+1 sequential queries → single recursive CTE in PostgreSQL (v0.50)
- Tantivy IndexWriter: 50MB allocation per single-doc index → shared `Arc<Mutex<Writer>>` (v0.50)
- Search facets: 4 sequential DB queries → `tokio::join!` concurrent (4 connections) (v0.50)
- Blocking `Path::canonicalize()` in 6 async file handlers → deferred to blocking thread pool (v0.48)
- Rate limiter `InMemoryStore` unbounded memory growth → periodic cleanup (v0.48)
- File walk bounded to 10,000 files (v0.48)
- N+1 queries eliminated in 4 endpoints (organization members, space list variants, document search, graph queries) (v0.35, v0.36)
- Shared `reqwest::Client` across DocumentState, ReviewState, TrueLayerClient (eliminated per-request `Client::new()`) (v0.32, v0.36)
- Incremental SSG builds via SHA-256 content hash manifest (O(changed) not O(total)) (v0.4)
- Compression middleware (gzip + brotli + deflate) (v0.41)
- Cache headers middleware (WASM/JS/CSS: 1yr immutable, HTML: no-cache) (v0.41)
- In-memory API cache with BTreeMap and configurable TTL (v0.41)
- WASM preload + DNS prefetch in index.html (v0.41)
- DB queries bounded with LIMIT clauses (11 unbounded queries fixed: webhooks 100, billing 100/50, presence 200/100, documents 100, connected_accounts 50) (v0.32)
- `HashMap` → `BTreeMap` for deterministic JSON serialization (no ordering overhead on small maps) (v0.35)
- `DashMap` rate limiter eliminates global write lock on every request (v0.32)

### Tests

- **1,589 total tests** (1,377 lib/bin + 212 integration + 3 doc) — up from 276 at v5.0.0
- **189 integration tests** against real PostgreSQL (user, space, document, auth, RBAC, search, template, team, tag, activity, billing, org, files, plugin, onboarding, notification, webhook, CRDT) (v0.38, v0.44)
- **86 database integration tests** (user CRUD 9, space CRUD 8, document CRUD 9, auth flow 8, RBAC 9, search 7, template 7, team 8, tag 4, activity 7) (v0.38)
- **100 database unit tests** (organization, search, session, billing, plugin, presence, comment, template, attachment, saved_search, password_reset, document_version, onboarding) (v0.37)
- **97 Leptos component tests** (types 14, error 11, markdown 30, TOC 10, version_history 15, review_panel 8, role_badge 16, storage 7, API 10) (v0.39)
- **29 plugin runtime tests** (error 7, sandbox 7, lib 15) — was zero (v0.49)
- **23 auth middleware integration tests** (JWT validation, public path bypass, RBAC permission checks, edge cases) (v0.50)
- **23 property-based graph invariant tests** (no self-loops, weight bounds, deterministic reversal, connected components, degree sum, serialization round-trips) (v0.6)
- **16 CRDT document manager tests** (get_or_create, apply_update, concurrent access, invalid input) (v0.48)
- **36 security tests** (6 OAuth2 CSRF, 7 brute-force, 6 XSS proof, 10 HMAC webhook, 17 OAuth2 unit) (v0.50, v0.53)
- **15 OT edge case tests** (delete-delete transform/compose, TP1 convergence property, concurrent deletes) (v0.51)
- **Testing crate** with unit tests (core_types 14, core_utils 20, database 25, session 17, search 22, repository 18, rbac 27) and integration helpers (server 10, search 8) plus common fixtures and factories (v0.43)
- **Criterion benchmarks** for search (indexing 100/1K/10K docs, query latency), renderer (small/medium/large), RBAC (1/10/100 roles) (v0.39)
- **Playwright E2E** infrastructure (auth flow, documents, navigation, 404) (v0.39, v0.52)
- **Full parallel test safety** — all 189 integration tests pass with any thread count (v0.47)
- **Lean4 formal verification** — TLA+ specs (OT convergence, review state machine, graph invariants, diff3 merge) and 10 proved theorems (v0.17, v0.18)
- **Zero clippy warnings** across entire workspace with `-D warnings` (v0.45, maintained through v0.55)
- **Zero compilation errors** across all workspace crates and all feature combinations (v0.45, v0.54)

[10.0.0]: https://github.com/WyattAu/Tachyon/releases/tag/v10.0.0
