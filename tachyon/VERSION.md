# Tachyon Version Information

## Current Status
- **Version:** 22.0.0
- **Phase:** Production-Ready — Full monorepo with 16 crates, 1,555+ unit tests, CI/CD pipeline, staging server live
- **Status:** Active development — all core infrastructure complete, 0 compilation errors, 0 clippy warnings
- **Last Updated:** 2026-05-28
- **Codebase:** 279+ Rust files, ~97K lines, 25 DB migrations, 29 route modules, 32 DB modules
- **Staging:** Live at http://192.168.1.3:8080 (TrueNAS Docker, PostgreSQL 16)

## What Changed (v22.0.0 — Security, Semantic Search, Cursor Pagination)

### Security
- **GraphQL audit middleware**: Wrapped `/graphql` and `/graphql/playground` with audit + request-id middleware (previously bypassed)
- **Swagger audit middleware**: Wrapped `/swagger-ui` and `/api/v1/openapi.json` with audit + request-id middleware
- **Heartbeat serde safety**: Replaced 2 `serde_json::to_string().unwrap()` in WebSocket heartbeat with `unwrap_or_else` fallback
- **Swagger serialization safety**: Replaced `expect("failed to serialize")` with error logging + JSON error response
- **Trivy multi-Dockerfile**: Container scan now scans Dockerfile, Dockerfile.server, and Dockerfile.frontend via matrix strategy

### Semantic Search (pgvector)
- **Document embedding on create**: Async spawned task embeds title+content via AI provider and persists to pgvector
- **Document embedding on update**: Re-embeds when content changes, skips for metadata-only updates
- **Semantic search endpoint**: `GET /api/v1/documents/semantic-search?q=...&limit=20&threshold=0.5` — embeds query, searches HNSW index via cosine distance
- **AiManager in DocumentState**: New `with_ai_manager()` builder method, wired in `init_app_state()`

### Cursor Pagination (7 new endpoints)
- **Spaces**: `GET /spaces/cursor` with CursorParams (after/before/limit)
- **Plugins**: `GET /plugins/cursor`
- **Organizations**: `GET /organizations/cursor`
- **Nodes**: `GET /nodes/cursor`
- **Users**: `GET /users/cursor`
- **Catalog/Projects**: `GET /projects/cursor`
- **Search**: `GET /search/cursor`
- All return CursorPage format: data, has_next, has_prev, next_cursor, prev_cursor, total_count
- Existing offset-based endpoints preserved for backward compatibility

### Performance
- **19 regex compilations converted to LazyLock**: Eliminated per-call regex compilation in hot paths
  - server/validation/common.rs: whitespace normalization, slug validation
  - core/util.rs: slugify, sanitize_string, validate_tag_name
  - server/sync.rs: slug collapse
  - ssg/render.rs: 15 statics for TOC, code blocks, tabs, mermaid, admonitions, images

### SBOM
- **SBOM attached to releases**: Confirmed release.yml uploads tachyon-sbom.spdx.json to GitHub Releases (was already working)

## What Changed (v20.0.0 — CI Hardening, Developer Guide)

### CI/CD
- **tachyon-cli excluded from CI Check job**: GTK transitive dependency (tachyon-desktop) not available in CI runners
- **Pre-commit hook**: tachyon-cli included locally (Nix provides GTK), excluded from CI
- **Rustdoc gate**: Filter out Cargo.toml feature warnings from rustdoc check
- **Secret detection**: Broadened exclusions for doc files with placeholder values

### Documentation
- **DEVELOPER_GUIDE.md**: Updated Axum 0.7->0.8, Rust nightly->stable 1.85+, port 3000->8080, added 7 missing crates, fixed api.rs->api/ path, marked Redis optional
- **ROADMAP.md**: Consolidated 6 stale roadmap files into single v22.0.0 document

## What Changed (v19.0.0 — Offline Sync Queue)

## What Changed (v18.0.0 — Delta Sync, Sync Priority, Staging)

### CRDT Delta Sync
- **`encode_diff()`**: Returns `None` if client is up-to-date (StateVector comparison), otherwise minimal diff via `encode_diff_v1`
- **`encode_delta_sync_response()`**: Formats diff as y-websocket sync step 2 message
- **`msg_type 3`**: Delta sync request handler — client sends state vector, server responds with diff only
- **5 CRDT delta tests**: up-to-date None, stale returns data, convergence after applying diff, empty SV, valid SV roundtrip

### Sync Queue Priority
- **`SyncPriority` enum**: Low(0), Normal(1), High(2), Critical(3) with auto-derivation from operation type
- **Priority ordering**: `pending_entries()` returns highest priority first, then oldest (SQL ORDER BY)
- **`enqueue_with_priority()`**: Explicit priority control for custom sync strategies
- **4 new tests**: priority ordering, operation derivation, same-priority FIFO, multi-operation

### Staging Deployment
- **Live staging server**: TrueNAS Docker (192.168.1.3:8080), PostgreSQL 16, all APIs functional
- **Deploy script**: `tachyon/deploy/staging/deploy.sh` — build, patchelf, scp, restart
- **Verified**: Auth, document CRUD, security headers, rate limiting, health checks all working

### CRDT Persistence (Phase 3.1)
- **Database tables**: `crdt_documents` (state BYTEA, version counter), `crdt_updates` (update log with seq ordering)
- **Save-on-evict**: LRU eviction now persists to PostgreSQL before dropping (fire-and-forget)
- **Load-on-connect**: `get_or_load()` restores persisted state when clients connect
- **Flush API**: `flush_document()` / `flush_all()` for explicit persistence
- **Garbage collection**: `gc_document_updates()` prunes old update log entries

### SSG
- **Image optimization pipeline**: `assets` module with `image` crate (PNG/JPEG re-encoding)
- **Static asset passthrough**: Images, fonts, CSS, JS copied from input to output
- **Configurable code theme**: `code_theme` field (default: github-dark)

### Performance
- **N+1 query fixes**: Search index update (UNNEST batch), tag search (single OR-clause), batch operations (concurrent via join_all)
- **Connection pool tuning**: `TACHYON_DB_MAX/MIN_CONNECTIONS`, `TACHYON_DB_CONNECTION_TIMEOUT` env vars
- **WebSocket jitter**: ±25% random jitter on reconnect backoff

### CI/CD
- **Criterion benchmark regression** in CI (main-only)
- **E2E smoke tests**: Fast critical-path tests (<5min)
- **Duplicate Docker builds eliminated**: release.yml manual-only
- **RUSTSEC-2026-0149**: wasmtime 44.0.1 → 44.0.2 (high severity fix)
- **Binary caching**: cargo-audit and cargo-tarpaulin cached between runs

### SSG Features
- **Syntax highlighting**: Highlight.js CDN with configurable theme (`code_theme` in SiteConfig, default: github-dark)
- **Dev server**: `tachyon-ssg-cli serve` with live rebuild via file watcher (notify crate, 500ms debounce)
- **Mermaid exclusion**: Highlight.js skips `language-mermaid` blocks
- **60 SSG tests** (was 56)

### WebSocket
- **Reconnection jitter**: ±25% random jitter on exponential backoff to prevent thundering herd
- **Sequence numbers**: Already implemented (atomic counter per broadcast)

### CI/CD
- **Duplicate Docker builds eliminated**: release.yml now manual-only (workflow_dispatch); cd.yml handles tag-based builds
- **RUSTSEC-2026-0097**: Audit override for rand 0.7.3 transitive dev-dep (wiremock)
- **All release.yml jobs**: Added timeout-minutes, permissions, FORCE_JAVASCRIPT_ACTIONS_TO_NODE24

### Documentation
- **Fixed broken .adrs/ reference** in collaboration.md

## What Changed (v5.0.0 — Three Horizons)

### Horizon 1: Alpha — Personal Knowledge Base (Complete)
- [x] **Route wiring** — Billing, Organization, SSG routes connected into production router
- [x] **Database exports** — Organization module registered in database lib.rs (7 types re-exported)
- [x] **RBAC policy seeding** — Default role-based policies (admin/editor/writer/reader) auto-loaded
- [x] **Password reset** — Token-based reset with SHA-256 hashing, expiry, webhook email delivery
- [x] **Email verification** — Token-based email change verification
- [x] **Local file browsing API** — 6 endpoints: list, read, search, tree, stats, recent
- [x] **Path traversal prevention** — All file ops validated against configurable root directory
- [x] **Frontmatter parsing** — YAML frontmatter extraction from markdown files
- [x] **Onboarding flow** — 4-step wizard, 5 sample documents, personalized suggestions

### Horizon 2: Beta — Team Collaboration (Complete)
- [x] **Organization management** — Full CRUD, member management (add/remove/update roles)
- [x] **Multi-tenancy tables** — Organizations, organization_members with auto-creation triggers
- [x] **SSG routes wired** — POST /ssg/build, GET /ssg/download (ZIP generation)
- [x] **Docker deployment** — Production docker-compose, nginx reverse proxy, SSL config
- [x] **Self-hosted guide** — Complete deployment README with backup/restore/scaling

### Horizon 3: GA — Production SaaS (Complete)
- [x] **TrueLayer billing** — OAuth client credentials, payment mandates, direct debit
- [x] **Payment tracking** — payments table, mandate status on subscriptions
- [x] **TrueLayer webhook** — Signature verification, status update handling
- [x] **Security hardening** — CSP with WASM support, Permissions-Policy, COEP/COOP
- [x] **Request ID tracking** — UUID per request, X-Request-Id header, tracing span
- [x] **Audit logging middleware** — Structured logging of all authenticated requests
- [x] **Per-route body limits** — 1MB general, 50MB uploads, 1GB WebSocket

### Infrastructure (Complete)
- [x] **CI pipeline** — GitHub Actions: check, lint, test (with PG), build release, WASM check
- [x] **Integration tests** — 27 tests across auth, documents, billing, orgs, files
- [x] **Unit tests** — 41 new tests for password reset, files, onboarding
- [x] **Testing crate** — TestApp, MockDataGenerator, db helpers, assertion utilities
- [x] **SSG dependency** — tachyon-ssg added to server Cargo.toml
- [x] **Billing bug fixes** — Borrow-after-move, missing Json() wrapper

## What Changed (v4.1.0 — Go Deep)

### All stubs replaced with real database-backed implementations
- [x] Comments persisted to PostgreSQL (document_comments table)
- [x] Billing persisted (subscriptions, invoices, notification_preferences)
- [x] Presence persisted (UPSERT with TTL, 5-minute expiry)
- [x] SSG color themes wired into templates (CSS custom properties)
- [x] SSG multi-language generation (per-language subdirs, RTL support)
- [x] Plugin WASM execution endpoint (POST /plugins/invoke)
- [x] Collaboration WebSocket broadcast (presence/comments → WS)
- [x] Collaboration & ecosystem routes registered in production router

## Architecture Summary

### Crates (16)
tachyon-core, tachyon-server, tachyon-desktop, tachyon-renderer, tachyon-rbac,
tachyon-search, tachyon-database, tachyon-storage, tachyon-testing, tachyon-cli,
tachyon-frontend, tachyon-import-export, tachyon-ssg, tachyon-plugin-runtime,
tachyon-desktop-app (Tauri), tachyon-editor, tachyon-benchmarks

### Route Modules (29)
activity, billing, catalog, collaboration, conflict, document, ecosystem, files,
node, notification, oauth2, onboarding, organization, password_reset, plugin,
repository, review, role, search, seo, session, space, ssg, tags, team, user, webhook

### Database Modules (32)
activity, attachment, billing, catalog, comment, connected_account, document_review,
document_version, error, graph, notification, onboarding, organization, password_reset,
permissions, plugin, presence, rbac, repository, saved_search, schema, search,
session, space, team, template, types, user_preferences, user, webhook

### Middleware (7)
auth (JWT + API key + RBAC), rate_limit, security_headers, cache_control, cors,
request_id, audit

### External Integrations
- TrueLayer (open banking payments)
- Google OAuth2, GitHub OAuth2
- Prometheus metrics
- Swagger/OpenAPI UI

### Database Tables (35+)
users, documents, repositories, document_versions, document_attachments, document_templates,
saved_searches, projects, components, project_members, roles, user_roles, audit_log,
search_index, teams, team_members, sessions, api_keys, knowledge_graph_nodes,
knowledge_graph_edges, document_reviews, review_comments, activity_events, notifications,
user_preferences, webhooks, plugins, spaces, space_members, connected_accounts,
organizations, organization_members, document_comments, subscriptions, invoices,
notification_preferences, document_presence, password_reset_tokens,
email_verification_tokens, payments

## Services
- **Backend:** http://localhost:8080
- **Frontend:** http://localhost:8080 (WASM served by backend in dev)
- **API:** http://localhost:8080/api/v1/
- **Swagger:** http://localhost:8080/swagger-ui/
- **WebSocket:** ws://localhost:8080/ws
- **CRDT WebSocket:** ws://localhost:8080/ws/crdt/{roomId}
- **Database:** PostgreSQL @ localhost:5432

## Configuration
```bash
# Server
TACHYON_HOST=0.0.0.0
TACHYON_PORT=8080
DATABASE_URL=postgres://tachyon:tachyon@localhost:5432/tachyon
TACHYON_JWT_SECRET=<32+ character secret>

# Auth
TACHYON_GUEST_LOGIN_ENABLED=false
TACHYON_PUBLIC_NOTES_ENABLED=false
TACHYON_ADMIN_USERNAME=admin
TACHYON_ADMIN_PASSWORD=<generated-if-not-set>
TACHYON_ADMIN_EMAIL=admin@tachyon.local

# OAuth2
TACHYON_OAUTH2_GOOGLE_CLIENT_ID=
TACHYON_OAUTH2_GOOGLE_CLIENT_SECRET=
TACHYON_OAUTH2_GITHUB_CLIENT_ID=
TACHYON_OAUTH2_GITHUB_CLIENT_SECRET=

# Files
TACHYON_FILES_ROOT=./content

# TrueLayer Billing
TRUELAYER_ENABLED=false
TRUELAYER_CLIENT_ID=
TRUELAYER_CLIENT_SECRET=
TRUELAYER_ENV=sandbox
TRUELAYER_MERCHANT_ACCOUNT_ID=
TRUELAYER_WEBHOOK_SECRET=

# Security
TACHYON_SECURITY_CSP_ENABLED=true
TACHYON_SECURITY_COEP_ENABLED=false

# SEO
TACHYON_SITE_TITLE=Tachyon
TACHYON_SITE_DESCRIPTION=A deterministic knowledge management system
TACHYON_BASE_URL=https://tachyon.dev
```
