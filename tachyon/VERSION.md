# Tachyon Version Information

## Current Status
- **Version:** 5.0.0
- **Phase:** Alpha Complete — All three horizons (Alpha/Beta/GA) implemented at API level
- **Status:** Active development — core infrastructure complete, all routes wired, 0 compilation errors
- **Last Updated:** 2026-04-19
- **Codebase:** 278 Rust files, ~92K lines, 25 DB migrations, 29 route modules, 32 DB modules

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
tachyon-desktop-app (Tauri), tachyon-collaboration

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
