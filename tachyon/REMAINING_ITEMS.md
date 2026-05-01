# Tachyon Remaining Items

**Version:** 5.0.0 | **Last Updated:** 2026-05-01 | **Codebase:** 278 Rust files, ~92K lines

---

## 1. What Was Accomplished (v0.45 -> v5.0.0)

### Core Infrastructure
- Full PostgreSQL-backed persistence layer (35+ tables, 28 migrations, 32 DB modules)
- Axum-based REST API with 29 route modules, 13 middleware layers
- JWT + API key + RBAC authentication with 7 middleware (auth, rate_limit, security_headers, cache_control, cors, request_id, audit)
- WebSocket support for real-time collaboration and CRDT (yrs-based)
- TOTP MFA with backup codes (`crates/server/src/routes/mfa.rs`)
- OAuth2 flows for Google and GitHub (`crates/server/src/routes/oauth2.rs`)
- Password reset with token hashing and expiry (`crates/server/src/routes/password_reset.rs`)

### Feature Implementation
- Document CRUD, search (Tantivy/BM25), versioning, reviews
- Knowledge graph with temporal edges and auto-extraction (`crates/server/src/graph_extractor.rs`)
- Organization/team/space multi-tenancy with member management
- Static site generation with multi-language, RTL, sitemap, RSS (`crates/ssg/`)
- Plugin WASM runtime with hook invocation (`crates/plugin-runtime/`)
- Billing integration via TrueLayer (OAuth mandates, direct debit, webhooks)
- Onboarding flow (4-step wizard, 5 sample documents)
- File browsing API with path traversal prevention
- Conflict resolution via 3-way merge (LCS-based) (`crates/server/src/conflict.rs`)
- Ecosystem routes (API metadata, feature flags, email notifications)

### Frontend (Leptos/WASM)
- 90+ components and pages including dashboard, documents, teams, spaces, billing, SSG, onboarding
- Accessibility primitives: `skip_links.rs`, `accessible.rs` (VisuallyHidden, LiveRegion, AccessibleDialog)
- i18n module with translation loading
- IndexedDB storage layer, WebSocket client, command palette

### Testing & CI
- 276+ unit/integration tests, E2E with Playwright (auth, documents, navigation)
- Criterion benchmarks for search, renderer, RBAC
- CI pipeline: check, lint, unit tests, integration tests (with PG service), build release, WASM check
- Docker multi-stage builds, docker-compose for dev and prod

---

## 2. Infrastructure / External Dependencies

### 2.1 Email Delivery Is a No-Op
- **Priority:** High
- **Effort:** Medium (1-3 days)
- **Description:** `EmailService::send()` in `crates/server/src/email.rs:43` logs and returns `Ok(())` when `smtp_url` is `None`. No actual SMTP/SES/Resend integration exists. Password reset, email verification, and notification emails are silently dropped in production.
- **Dependencies:** None
- **Files:** `crates/server/src/email.rs`

### 2.2 TrueLayer Requires Real Credentials
- **Priority:** Medium
- **Effort:** Small (< 1 day)
- **Description:** TrueLayer billing is gated behind `TRUELAYER_ENABLED=false` and requires sandbox/production credentials. All billing routes degrade gracefully but no end-to-end payment flow has been verified with real TrueLayer API.
- **Dependencies:** TrueLayer sandbox account
- **Files:** `crates/server/src/truelayer.rs`, `crates/server/src/routes/billing.rs`

### 2.3 OAuth2 CSRF State Not Validated
- **Priority:** Critical
- **Effort:** Small (< 1 day)
- **Description:** The `state` parameter in `CallbackQuery` (`crates/server/src/routes/oauth2.rs:59`) is parsed but never validated against the value sent in the authorize redirect. This makes OAuth2 flows vulnerable to CSRF attacks.
- **Dependencies:** Session storage for state nonce
- **Files:** `crates/server/src/routes/oauth2.rs`

### 2.4 Rate Limiting Falls Back to In-Memory Without Redis
- **Priority:** Low
- **Effort:** Medium (1-3 days)
- **Description:** When `redis_url` is `None`, rate limiting uses an in-memory `DashMap` (`crates/server/src/middleware/rate_limit.rs`). This is ineffective in multi-instance deployments. No Redis integration has been verified end-to-end.
- **Dependencies:** Redis deployment
- **Files:** `crates/server/src/middleware/rate_limit.rs`

### 2.5 CORS Defaults to Permissive
- **Priority:** High
- **Effort:** Small (< 1 day)
- **Description:** Default `CorsConfig` uses `"*"` for `allowed_origins`. Production deployments must explicitly set allowed origins, but there is no startup validation to enforce this when `TACHYON_CORS_ALLOWED_ORIGINS` is unset.
- **Dependencies:** None
- **Files:** `crates/server/src/config.rs`, `crates/server/src/lib.rs:651`

### 2.6 Tauri Desktop EGL Display Issue
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** NVIDIA + WebKitGTK EGL display initialization fails on certain Linux environments. The Tauri binary compiles but cannot display a window.
- **Dependencies:** NVIDIA/WebKitGTK environment fix
- **Files:** `crates/desktop/src-tauri/`

---

## 3. Feature Gaps

### 3.1 XSS Sanitization in Markdown Renderer Output
- **Priority:** Critical
- **Effort:** Medium (1-3 days)
- **Description:** The penetration testing report (VERSION.md:419) flagged XSS in content and title as Medium severity. While `sanitize_string()` exists in `crates/server/src/validation/common.rs:69`, the markdown renderer (`crates/renderer/`) does not sanitize HTML output. Raw HTML blocks in markdown pass through unescaped.
- **Dependencies:** ammonia or similar HTML sanitizer crate
- **Files:** `crates/renderer/src/`, `crates/server/src/validation/common.rs`

### 3.2 RBAC Integration Tests Are Stubs
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** `crates/testing/src/integration/rbac.rs:7` is a placeholder `test_rbac_integration_stub()`. Real RBAC enforcement across routes (documents, spaces, teams, organizations) is not tested at the integration level.
- **Dependencies:** Test database setup
- **Files:** `crates/testing/src/integration/rbac.rs`

### 3.3 IPC Integration Tests Are Stubs
- **Priority:** Low
- **Effort:** Medium (1-3 days)
- **Description:** `crates/testing/src/integration/ipc.rs:6` is a placeholder `test_ipc_stub()`. Desktop-to-server IPC is not integration-tested.
- **Dependencies:** Tauri runtime environment
- **Files:** `crates/testing/src/integration/ipc.rs`

### 3.4 Fuzzing Harness Is a Stub
- **Priority:** Medium
- **Effort:** Large (3-7 days)
- **Description:** `crates/testing/src/fuzz/harness.rs:8` is a placeholder that prints "not yet implemented". The `cargo-fuzz` dependency is in `Cargo.toml` but no actual fuzz targets exist for the API, renderer, or parser.
- **Dependencies:** None
- **Files:** `crates/testing/src/fuzz/harness.rs`, `crates/testing/src/fuzz/mod.rs`

### 3.5 Performance Benchmarks Are Stubs (testing crate)
- **Priority:** Medium
- **Effort:** Large (3-7 days)
- **Description:** Four benchmark modules in `crates/testing/src/benchmarks/` are placeholder comments: `search_bench.rs:3`, `repository_bench.rs:3`, `rbac_bench.rs:3`, `database_bench.rs:3`. Note: actual criterion benchmarks exist in `crates/benchmarks/benches/` (search, renderer, rbac), so the testing crate duplicates are lower priority.
- **Dependencies:** None
- **Files:** `crates/testing/src/benchmarks/{search,repository,rbac,database}_bench.rs`

### 3.6 Document Persistence Not End-to-End Verified
- **Priority:** High
- **Effort:** Small (< 1 day)
- **Description:** VERSION.md:486 notes "API creates documents in memory but doesn't persist to database (repository layer implementation needed)." The Go Deep pass (v4.1.0) addressed comments/billing/presence, but the core document CRUD routes may still have gaps between in-memory handling and actual PostgreSQL persistence for all fields.
- **Dependencies:** Integration test suite against real DB
- **Files:** `crates/server/src/routes/document/`

### 3.7 Large Payload Error Handling
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** Penetration testing flagged missing proper error responses for oversized requests (VERSION.md:423). Body limits exist (1MB general, 50MB uploads) but error messages to the client are generic.
- **Dependencies:** None
- **Files:** `crates/server/src/middleware/request_limit.rs`

### 3.8 No Actual SMTP Integration
- **Priority:** High
- **Effort:** Medium (1-3 days)
- **Description:** The `smtp_url` config field exists but no SMTP client code is implemented. `EmailService` has `#[allow(dead_code)]` on `client` and `from_address` fields (`email.rs:23,27`).
- **Dependencies:** SMTP provider (SES, Resend, SendGrid, or self-hosted)
- **Files:** `crates/server/src/email.rs`

---

## 4. Testing Gaps

### 4.1 Coverage Threshold Not Enforced in CI
- **Priority:** Medium
- **Effort:** Small (< 1 day)
- **Description:** `.coverage.toml:48` sets `fail_under_threshold = false`. CI will not fail if coverage drops below the 60% minimum.
- **Dependencies:** None
- **Files:** `.coverage.toml`, `.github/workflows/ci.yml`

### 4.2 No E2E Tests for Collaboration, Billing, SSG
- **Priority:** Medium
- **Effort:** Large (3-7 days)
- **Description:** Playwright E2E tests only cover auth, documents, and navigation (`e2e/tests/`). No E2E coverage for: real-time collaboration (WebSocket), billing flows, SSG build/download, file upload, onboarding, or admin operations.
- **Dependencies:** None
- **Files:** `e2e/tests/`

### 4.3 No Load/Stress Testing
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** Initial benchmarks (200 concurrent requests) showed sub-ms latency. No sustained load tests, connection pool exhaustion tests, or memory leak detection under load have been performed.
- **Dependencies:** Load testing tool (k6, hey, wrk)
- **Files:** N/A (new test infrastructure needed)

### 4.4 Test Cleanup Is Incomplete
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** `TestApp::cleanup()` and `db_helpers::cleanup_test_data()` only clean 5 tables (documents, projects, sessions, teams, templates). 30+ other tables with test data (organizations, spaces, comments, notifications, etc.) are not cleaned between test runs.
- **Dependencies:** None
- **Files:** `crates/testing/src/lib.rs:70-81`, `crates/testing/src/lib.rs:155-163`

### 4.5 No Tests for Middleware Chain
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** Individual middleware modules have unit tests, but the combined middleware stack (auth -> request_id -> audit -> cache_control -> compression -> rate_limit -> security_headers -> CORS) is not tested as a pipeline. Interaction effects (e.g., rate limit + auth) are untested.
- **Dependencies:** None
- **Files:** `crates/server/src/middleware/`

### 4.6 No Tests for CRDT Document Manager
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** `crates/server/src/crdt.rs` has unit tests for the in-memory `CrdtDocumentManager`, but there are no integration tests for the CRDT WebSocket handler (`websocket/crdt_handler.rs`) that verifies state sync across multiple clients.
- **Dependencies:** WebSocket test infrastructure
- **Files:** `crates/server/src/crdt.rs`, `crates/server/src/websocket/crdt_handler.rs`

---

## 5. Documentation Gaps

### 5.1 Only 2 Developer Docs
- **Priority:** Medium
- **Effort:** Large (3-7 days)
- **Description:** `tachyon/docs/` contains only `DEVELOPER_GUIDE.md` and `COMPONENTS.md`. Missing: API reference (auto-generated from OpenAPI is available but not published), architecture decision records, deployment runbook, on-call playbook, data model documentation, migration guide.
- **Dependencies:** None
- **Files:** `tachyon/docs/`

### 5.2 Frontend Component Storybook / Examples
- **Priority:** Low
- **Effort:** X-Large (> 7 days)
- **Description:** 90+ Leptos components exist but there is no visual documentation, component catalog, or interactive playground.
- **Dependencies:** None
- **Files:** `crates/frontend/src/components/`

### 5.3 CONTRIBUTING.md Missing
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** No contributor guide exists. New developers must infer build steps, test commands, and code conventions from `justfile` and CI config.
- **Dependencies:** None
- **Files:** N/A (new file)

### 5.4 API Error Code Reference
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** Error codes like `VALIDATION_ERROR`, `INVALID_ID`, `NOT_FOUND` are used across routes but there is no centralized reference documenting all error codes, their HTTP status mappings, and when they are returned.
- **Dependencies:** None
- **Files:** `crates/server/src/error.rs`

---

## 6. Accessibility Gaps

### 6.1 Frontend Accessibility Not Audited
- **Priority:** High
- **Effort:** Medium (1-3 days)
- **Description:** While accessibility primitives exist (`skip_links.rs`, `accessible.rs` with VisuallyHidden, LiveRegion, AccessibleDialog), no automated or manual accessibility audit has been performed against the 90+ Leptos components. Compliance claims (WCAG 2.1 AA, Section 508) are aspirational without evidence.
- **Dependencies:** axe-core or Pa11y integration
- **Files:** `crates/frontend/src/components/`

### 6.2 Keyboard Navigation Not Verified
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** `keyboard.rs` defines shortcuts but full keyboard navigability (tab order, focus management, dialog trapping) across all pages has not been verified.
- **Dependencies:** None
- **Files:** `crates/frontend/src/components/keyboard.rs`

### 6.3 Screen Reader Compatibility Untested
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** No testing with NVDA, VoiceOver, or JAWS. ARIA labels, roles, and live regions exist in isolated components but have not been verified in page-level flows.
- **Dependencies:** Screen reader testing environment
- **Files:** `crates/frontend/src/components/accessible.rs`

---

## 7. Performance Opportunities

### 7.1 No Database Connection Pool Tuning
- **Priority:** Medium
- **Effort:** Small (< 1 day)
- **Description:** `DatabasePool` uses sqlx default pool settings. No explicit configuration for `max_connections`, `min_connections`, `acquire_timeout`, or `idle_timeout`. Under load these defaults may be suboptimal.
- **Dependencies:** Load testing data
- **Files:** `crates/database/src/schema.rs`

### 7.2 No Response Caching for Read-Heavy Endpoints
- **Priority:** Low
- **Effort:** Medium (1-3 days)
- **Description:** `ApiCache` exists (`crates/server/src/middleware/api_cache.rs`) with a 60-second TTL but is not wired into any route handlers. Read-heavy endpoints like document list, search results, and catalog could benefit significantly.
- **Dependencies:** Cache invalidation strategy
- **Files:** `crates/server/src/middleware/api_cache.rs`

### 7.3 No Query Optimization / EXPLAIN Analysis
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** 28 migrations have been applied but no systematic query performance analysis has been done. The `20260428000000_composite_indexes.sql` migration suggests some optimization has started, but coverage is unknown.
- **Dependencies:** Production-scale data set
- **Files:** `crates/database/migrations/`

### 7.4 Tantivy Index Not Integrated with API Search
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** `tachyon-search` crate has a full Tantivy-based index, but the server search route (`crates/server/src/routes/search.rs`) uses PostgreSQL full-text search (`tsvector`) via `SearchRepository`. The Tantivy index is not wired into the production server.
- **Dependencies:** Index lifecycle management (create, update, reindex)
- **Files:** `crates/search/`, `crates/server/src/routes/search.rs`

### 7.5 WASM Bundle Size Not Optimized
- **Priority:** Low
- **Effort:** Medium (1-3 days)
- **Description:** Release profile uses `opt-level = "z"` and LTO, but no WASM-specific optimizations (wasm-opt, wasm-bindgen wasm-pack profile tuning, tree-shaking) have been verified. Initial bundle size measurements are not documented.
- **Dependencies:** wasm-opt toolchain
- **Files:** `Cargo.toml` (release profile)

---

## 8. Security Considerations

### 8.1 OAuth2 CSRF State Validation Missing
- **Priority:** Critical
- **Effort:** Small (< 1 day)
- **Description:** (Duplicate of 2.3) OAuth2 callback accepts any `state` parameter without validation. An attacker can craft a callback URL to hijack OAuth tokens.
- **Dependencies:** Session/cookie storage for CSRF nonce
- **Files:** `crates/server/src/routes/oauth2.rs:59`

### 8.2 XSS in Rendered Markdown
- **Priority:** Critical
- **Effort:** Medium (1-3 days)
- **Description:** (Duplicate of 3.1) Raw HTML in markdown passes through unescaped in the renderer output.
- **Dependencies:** HTML sanitizer crate
- **Files:** `crates/renderer/src/`

### 8.3 CSP Allows `'unsafe-inline'` Styles
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** Default security headers (`crates/server/src/middleware/security_headers.rs:72,155`) include `'unsafe-inline'` for `style_src`. This weakens XSS protection. Moving to nonce-based or hash-based CSP would require template changes.
- **Dependencies:** Frontend template refactoring for nonce injection
- **Files:** `crates/server/src/middleware/security_headers.rs`

### 8.4 No Secret Rotation Mechanism
- **Priority:** Low
- **Effort:** Medium (1-3 days)
- **Description:** JWT secret is loaded once at startup. No mechanism exists for secret rotation without downtime. Refresh tokens use hashed storage but no rotation policy is enforced.
- **Dependencies:** None
- **Files:** `crates/server/src/config.rs`, `crates/server/src/routes/user.rs`

### 8.5 No Brute-Force Protection on Login
- **Priority:** High
- **Effort:** Small (< 1 day)
- **Description:** Rate limiting exists (5 requests/60s on `/api/v1/auth/login`) but there is no account lockout mechanism, progressive backoff, or CAPTCHA after failed attempts.
- **Dependencies:** None
- **Files:** `crates/server/src/routes/user.rs`, `crates/server/src/middleware/rate_limit.rs`

### 8.6 `unwrap()` in Non-Test Production Code
- **Priority:** Medium
- **Effort:** Small (< 1 day)
- **Description:** `crates/server/src/lib.rs:765` uses `.expect("failed to install Prometheus metrics recorder")` which panics at runtime if the global recorder is already installed. Production middleware should handle this gracefully.
- **Dependencies:** None
- **Files:** `crates/server/src/lib.rs:757-769`

---

## 9. Technical Debt

### 9.1 Excessive `#[allow(dead_code)]` Annotations (100+)
- **Priority:** Medium
- **Effort:** Large (3-7 days)
- **Description:** Over 100 `#[allow(dead_code)]` annotations across the codebase, concentrated in `crates/frontend/src/api/` (billing, documents, teams, spaces, files, auth, search, projects, templates, settings, plugins, graph), `crates/server/src/websocket/`, `crates/server/src/email.rs`, `crates/search/src/api.rs`, `crates/desktop/src-tauri/src/commands.rs`, and `crates/testing/src/unit/`. These indicate unused structs, fields, and functions that should be either wired in or removed.
- **Dependencies:** None
- **Files:** See grep results (100+ files)

### 9.2 ProrationResult Is Dead Code
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** `ProrationResult` in `crates/server/src/routes/billing.rs:31` has 3 `#[allow(dead_code)]` fields and is never constructed or returned.
- **Dependencies:** None
- **Files:** `crates/server/src/routes/billing.rs:31-39`

### 9.3 No Centralized Error Type
- **Priority:** Medium
- **Effort:** Large (3-7 days)
- **Description:** Each route module defines its own error type alias (e.g., `type Error = (StatusCode, Json<UserErrorResponse>)` in mfa.rs, `type Error = (StatusCode, Json<ErrorResponse>)` in conflict.rs). No unified `ApiError` type with `IntoResponse` is used consistently.
- **Dependencies:** None
- **Files:** All route modules in `crates/server/src/routes/`

### 9.4 Large Route Files
- **Priority:** Low
- **Effort:** Medium (1-3 days)
- **Description:** Several route files exceed 500 lines: `billing.rs` (1483 lines), `user.rs` (1400+ lines), `onboarding.rs` (900+ lines), `review.rs` (511 lines), `collaboration.rs` (540 lines). These should be split into handler, types, and test submodules.
- **Dependencies:** None
- **Files:** `crates/server/src/routes/{billing,user,onboarding,review,collaboration}.rs`

### 9.5 Edition Mismatch
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** Workspace `Cargo.toml:7` uses `edition = "2021"` but the CHANGELOG notes desktop crates were upgraded to `2024`. The workspace default should be updated to 2024.
- **Dependencies:** None
- **Files:** `tachyon/Cargo.toml:7`

### 9.6 TestApp Connects to Default DB Port
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** `TestApp::new()` in `crates/testing/src/lib.rs:51` defaults to port 5432 but CI uses port 5432 for the service. This could conflict. `create_router()` in `routes/mod.rs:67` defaults to port 5433. These should be consistent.
- **Dependencies:** None
- **Files:** `crates/testing/src/lib.rs:51`, `crates/server/src/routes/mod.rs:67`

---

## 10. Operational

### 10.1 No Database Backup Strategy
- **Priority:** High
- **Effort:** Medium (1-3 days)
- **Description:** No automated PostgreSQL backup, point-in-time recovery, or backup verification is configured. `docker-compose.prod.yml` has volume persistence but no `pg_dump` cron or WAL archiving.
- **Dependencies:** Backup storage (S3, GCS, or local)
- **Files:** `docker-compose.prod.yml`, `deploy/`

### 10.2 No Health Check Probes for Readiness
- **Priority:** Medium
- **Effort:** Small (< 1 day)
- **Description:** `/health` returns "OK" and `/ready` exists but the readiness check in `routes/health.rs` only verifies database connectivity. It does not check Redis, TrueLayer, or SMTP connectivity.
- **Dependencies:** None
- **Files:** `crates/server/src/routes/health.rs`

### 10.3 No Structured Logging Configuration
- **Priority:** Low
- **Effort:** Small (< 1 day)
- **Description:** `tracing_subscriber` is initialized with default settings. No JSON logging mode, log level configuration per module, or structured field extraction for log aggregation (ELK, Loki, CloudWatch).
- **Dependencies:** Log aggregation platform
- **Files:** `crates/server/src/main.rs`

### 10.4 No Graceful Migration Rollback
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** `migrations::run_migrations()` applies all pending migrations. No `migrations::rollback()` or versioned migration management exists. Rolling back a bad migration requires manual SQL.
- **Dependencies:** None
- **Files:** `crates/database/src/migrations.rs`

### 10.5 No SSL/TLS Termination in Docker Setup
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** `PHASE8_SUMMARY.md:230` lists "Add SSL/TLS" as a next step. The nginx config and docker-compose do not include Let's Encrypt/Certbot integration. TLS config fields exist in `ServerConfig` but are not used in the Docker deployment.
- **Dependencies:** Domain name, DNS
- **Files:** `docker-compose.prod.yml`, `nginx/`, `crates/server/src/config.rs`

### 10.6 No Incident Response Playbook
- **Priority:** Low
- **Effort:** Medium (1-3 days)
- **Description:** No runbooks exist for: database outage, WebSocket disconnection storms, failed migrations, certificate expiry, or security incidents.
- **Dependencies:** None
- **Files:** N/A (new documentation)

### 10.7 No Monitoring Dashboard
- **Priority:** Medium
- **Effort:** Large (3-7 days)
- **Description:** Prometheus metrics endpoint exists (`/metrics/prometheus`) but no Grafana dashboards, alerting rules, or SLO definitions have been created.
- **Dependencies:** Grafana deployment
- **Files:** N/A (new configuration)

### 10.8 No CD Pipeline for Automated Releases
- **Priority:** Medium
- **Effort:** Medium (1-3 days)
- **Description:** CI pipeline exists but there is no CD pipeline for automated releases (tag creation, Docker image publishing to registry, deployment to staging/production). The `ci.yml` only has check, test, lint, and build jobs.
- **Dependencies:** Container registry, deployment target
- **Files:** `.github/workflows/ci.yml`

---

## Summary by Priority

| Priority | Count | Key Items |
|----------|-------|-----------|
| **Critical** | 2 | OAuth2 CSRF, XSS in markdown |
| **High** | 6 | Email no-op, CORS defaults, document persistence, brute-force, DB backups, accessibility audit |
| **Medium** | 28 | Stubs, testing gaps, CSP, RBAC tests, load testing, docs, monitoring, SSL |
| **Low** | 12 | Dead code, edition mismatch, port consistency, large files, CONTRIBUTING.md |

**Recommended Sprint Order:**
1. Fix Critical security issues (OAuth2 CSRF, XSS sanitization)
2. Wire email delivery
3. Harden CORS defaults + brute-force protection
4. Fill test stubs (RBAC, fuzzing, middleware chain)
5. Enforce coverage in CI + expand E2E tests
6. Performance: response caching, DB pool tuning, Tantivy integration
7. Operational: backups, SSL, monitoring, CD pipeline
8. Documentation: API reference, contributor guide, runbooks
9. Technical debt: dead code cleanup, error type unification
