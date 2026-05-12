# Tachyon Roadmap

**Version:** 10.1.0 | **Last Updated:** 2026-05-11 | **Codebase:** 278 Rust files, ~92K lines, 16 crates

---

## Current State Summary

| Dimension | Status |
|-----------|--------|
| Build | Passes (0 errors, 0 warnings) |
| Tests | 1,353 tests across 37 binaries, 0 failures |
| Formatting | `cargo fmt --check` clean |
| Linting | `cargo clippy -D warnings` clean |
| Documentation | `cargo doc --no-deps` clean (0 warnings) |
| Pre-commit hook | Configured at `.githooks/`, 7 checks |
| CI | GitHub Actions: build, test, lint, integration, security, fuzz |
| Deployment | Docker multi-stage, docker-compose dev/prod, Nix flake |

---

## Phase 1: Hardening (v10.2.0) -- 2 weeks

### 1.1 Wire Email Delivery [High] -- COMPLETE (v10.0.0)

`EmailService::send()` is fully implemented with `lettre` 0.11 (async SMTP, multipart, 3-attempt retry at 1s/5s/15s). Requires `SMTP_URL` config. Falls back to log-and-skip when unconfigured.

### 1.2 Remove `#[allow(dead_code)]` Annotations [Medium]

82 `#[allow(dead_code)]` annotations cleaned in v5.0.0 refactor. Remaining annotations (~20) are in frontend API client modules and desktop commands awaiting UI integration.

**Effort:** 3-5 days. Batch by crate.

### 1.3 Database Connection Pool Tuning [Medium] -- COMPLETE (v10.0.0)

`DbPoolConfig` in `config.rs` includes `db_max_connections` (default 10), `db_min_connections` (default 2), `db_acquire_timeout_ms` (default 5000), `db_idle_timeout_secs` (default 600). All configurable via env vars with validation.

### 1.4 Test Cleanup Completeness [Low] -- COMPLETE (v10.0.0)

`TestApp::cleanup()` TRUNCATE CASCADE's 41 tables. `cleanup_test_data()` provides standalone cleanup for integration tests.

---

## Phase 2: Security Hardening (v10.3.0) -- 2 weeks

### 2.1 CSP Nonce-Based Styles [Medium] -- PARTIAL (v10.1.0)

CSP nonce generation and header injection exist in `security_headers.rs`. `GenerateNonce` type, `CspNonce` extension, and per-request nonce generation are implemented. Remaining: inject nonce into inline `<style>` elements via template layer, remove `'unsafe-inline'` from `style-src`.

### 2.2 Secret Rotation Support [Low] -- COMPLETE (v10.0.0)

JWT secrets support comma-separated `TACHYON_JWT_SECRETS`. `JwtConfig.signing_secret()` returns first secret for signing; validation tries all secrets. `test_jwt_validation_with_rotated_secret` validates rotation scenario.

### 2.3 Prometheus `.expect()` Removal [Medium] -- COMPLETE (v10.0.0)

Prometheus metrics handler uses proper error handling. No `.expect()` panics in metrics path.

### 2.4 Prometheus `.expect()` Panics in Middleware [Medium]

**Scope:**
- Audit all `.expect()` and `.unwrap()` calls in non-test production code paths
- Replace with proper error handling via `?` or `match`
- Enable `#![deny(clippy::expect_used)]` incrementally

**Files:** All route and middleware files

---

## Phase 3: Testing Expansion (v10.4.0) -- 3 weeks

### 3.1 Fill RBAC Integration Test Stub [Medium] -- COMPLETE (v10.0.0)

`crates/testing/src/integration/rbac.rs` contains 697 lines of comprehensive RBAC integration tests covering: admin role broad permissions, reader role limited permissions, explicit permission deny/grant, inactive user restrictions, role permission levels, team role inheritance, and organization-level admin override. 15 separate test functions.

### 3.2 Fill Fuzzing Harness [Medium] -- COMPLETE (v10.1.0)

Fuzz harness now contains 4 cargo-fuzz compatible targets: `fuzz_markdown_parse`, `fuzz_jwt_validate`, `fuzz_search_query`, `fuzz_json_request`. Each target has a deterministic test-mode corpus for CI. Individual fuzzing modules in `search.rs`, `rbac.rs`, `repository.rs`, `utilities.rs` provide 40+ property-based edge-case tests.

### 3.3 Middleware Chain Integration Tests [Medium] -- COMPLETE (v10.0.0)

`middleware/tests.rs` (998 lines) includes `test_full_middleware_chain_composition` and individual interaction tests for auth+public paths, CSP nonce injection, request ID propagation, rate limit headers, cache-control ETags, security headers ordering, and audit middleware data capture.

### 3.4 CRDT WebSocket Integration Tests [Medium] -- COMPLETE (v10.0.0)

`integration/crdt_test.rs` contains `test_crdt_sync_between_clients`, `test_crdt_convergence_with_concurrent_edits`, `test_initial_state_sent_to_new_client`, and `test_presence_client_count`. WebSocket test infrastructure exists.

### 3.5 Coverage Threshold Enforcement [Medium] -- COMPLETE (v10.0.0)

`.coverage.toml` sets `fail_under_threshold = true` with 60% minimum, per-module thresholds (core: 70%, database: 65%, rbac: 65%, server: 60%, search: 60%).

### 3.6 E2E Tests: Collaboration, Billing, SSG [Medium] -- PARTIAL

13 Playwright E2E spec files exist: auth, documents, navigation, onboarding, collaboration (196 lines), documents-crud, spaces-teams, settings, accessibility, keyboard-nav, screen-reader, a11y-automated-report. Billing and SSG E2E tests still needed.

---

## Phase 4: Performance (v11.0.0) -- 2 weeks

### 4.1 Wire Response Cache [Medium] -- COMPLETE (v10.0.0)

`ApiCache` (60s TTL) wired into `list_documents` (document_crud.rs:714-792) and `catalog_stats` (catalog.rs:643-668). Cache invalidation on document create/update/delete. `X-Cache-Status` header (HIT/MISS) returned.

### 4.2 Tantivy Integration [Medium] -- COMPLETE (v10.0.0)

Tantivy index initialized on server startup (`main.rs:init_tantivy_index`). Document create/update triggers Tantivy indexing, delete triggers removal. Search queries use Tantivy with PostgreSQL fallback and result fusion. `/api/v1/search/reindex` endpoint for full reindex.

### 4.3 Query Optimization [Medium]

28 migrations applied but no systematic `EXPLAIN` analysis.

**Scope:**
- Identify top-10 most-frequently-executed queries from test patterns
- Run `EXPLAIN ANALYZE` on each with production-scale data (100K documents)
- Add missing composite indexes based on query patterns
- Verify index usage with `EXPLAIN` after changes
- Document query plan baseline

**Effort:** 2-3 days.

### 4.4 WASM Bundle Size Optimization [Low]

**Scope:**
- Measure current bundle size (compressed/uncompressed)
- Run `wasm-opt` with `-Oz` flag
- Enable `wasm-bindgen` feature flags to strip debug info
- Verify tree-shaking is effective (check for unused code in bundle)
- Document bundle size budget and regression detection

**Effort:** 2-3 days.

---

## Phase 5: Operational Maturity (v11.1.0) -- 2 weeks

### 5.1 Database Backup Strategy [High]

No automated PostgreSQL backup or point-in-time recovery.

**Scope:**
- Add `pg_dump` cron job (daily full, hourly incremental via WAL archiving)
- Backup verification: restore to test database and run test suite
- Backup retention policy: 30 daily, 12 weekly, 6 monthly
- Off-site backup: S3-compatible storage with encryption
- Document recovery procedure (RTO: 1 hour, RPO: 1 hour)

**Files:** `docker-compose.prod.yml`, new `scripts/backup.sh`

### 5.2 SSL/TLS Termination [Medium]

No Let's Encrypt/Certbot integration in Docker setup.

**Scope:**
- Add `certbot` sidecar container to `docker-compose.prod.yml`
- Configure nginx for ACME challenge and TLS termination
- Auto-renewal via cron (certbot renew --quiet)
- HTTP to HTTPS redirect
- HSTS preload eligibility

**Files:** `docker-compose.prod.yml`, `nginx/`

### 5.3 Readiness Probe Expansion [Medium] -- COMPLETE (v10.1.0)

`/health` returns component-level JSON with database, Redis (TCP connectivity), Tantivy (index open), and SMTP (DNS/TCP) checks. `/ready` returns HTTP 200/503 for Kubernetes readiness probes.

### 5.4 Structured JSON Logging [Low] -- COMPLETE (v10.0.0)

`tracing-subscriber` configured with `json` feature. `TACHYON_LOG_FORMAT=json` env var switches to structured output.

### 5.5 Migration Rollback Support [Medium] -- COMPLETE (v10.0.0)

`database/src/rollback.rs` (1,008 lines): `rollback_last_migration`, `rollback_to_version`, `dry_run_rollback`, `migration_status`. CLI exposes via `tachyon db rollback` command with `--steps` and `--dry-run` flags.

### 5.6 CD Pipeline [Medium] -- COMPLETE (v10.1.0)

`.github/workflows/release.yml`: tag-triggered multi-arch Docker build, SBOM generation, publish to ghcr.io, GitHub Release with changelog. `.github/workflows/deploy-staging.yml`: develop-branch deploy with health-check and auto-rollback.

### 5.7 Monitoring Dashboard [Medium] -- COMPLETE (v10.1.0)

Grafana dashboard `api-overview.json` with 8 panels (request rate, error rate, P50/P95/P99 latency, WS connections, DB pool, cache hit rate, status codes, top endpoints). Prometheus alert rules for error rate, latency, pool exhaustion, disk, WS drops, cache hits, instance down.

---

## Phase 6: Architecture Improvements (v12.0.0) -- 3 weeks

### 6.1 Unified Error Type [Medium]

Each route module defines its own error type alias. No consistent `ApiError` with `IntoResponse`.

**Scope:**
- Define `tachyon_server::error::ApiError` enum covering all error categories
- Implement `IntoResponse` with consistent JSON error envelope
- Implement `From<DatabaseError>`, `From<serde_json::Error>`, etc.
- Replace all per-module error types with `ApiError`
- Add error code to response: `{ "code": "VALIDATION_ERROR", "message": "...", "details": [...] }`

**Effort:** 5-7 days. Touches all 29 route modules.

### 6.2 Large Route File Split [Low]

Several route files exceed 500 lines: `billing.rs` (1483), `user.rs` (1400+), `onboarding.rs` (900+).

**Scope:**
- Split each into `routes/{module}/mod.rs`, `handlers.rs`, `types.rs`, `tests.rs`
- Preserve public API surface
- No behavioral changes

**Effort:** 3-5 days.

### 6.3 Workspace Edition 2024 [Low]

Workspace uses `edition = "2021"` but desktop crates may use `2024`.

**Scope:**
- Update `tachyon/Cargo.toml` to `edition = "2024"`
- Fix any compilation errors from edition changes
- Verify all tests pass

**Effort:** 1 day.

### 6.4 API Cache Wiring to Read-Heavy Endpoints [Low]

(Detailed in Phase 4.1, moved here for ordering.)

---

## Phase 7: Frontend Polish (v12.1.0) -- 3 weeks

### 7.1 Accessibility Audit [High]

No automated or manual audit performed against 90+ Leptos components.

**Scope:**
- Integrate `axe-core` via wasm-bindgen for automated a11y testing in CI
- Fix all critical and serious violations
- Verify keyboard navigation (tab order, focus trapping in dialogs)
- Test with screen reader (NVDA on Linux)
- Document WCAG 2.1 AA compliance status per component

**Effort:** 5-7 days.

### 7.2 Frontend Dead Code Cleanup [Medium]

`crates/frontend/src/api/` has extensive `#[allow(dead_code)]`.

**Scope:**
- Remove unused API client functions
- Wire remaining functions to real components
- Verify no regressions in browser

**Effort:** 2-3 days.

### 7.3 Component Documentation [Low]

90+ components with no visual documentation.

**Scope:**
- Add doc comments to all public components
- Create a component catalog page in the frontend
- Document props, events, slots, and usage examples

**Effort:** 5-7 days.

---

## Phase 8: Ecosystem (v13.0.0) -- Ongoing

### 8.1 Plugin SDK Stabilization

- Define stable plugin API contract (versioned)
- WASM sandbox with resource limits (memory, CPU, network)
- Plugin marketplace infrastructure (upload, verify, distribute)
- Plugin hooks: pre-render, post-save, search-index, notification

### 8.2 Import/Export Enhancements

- Import from: Notion, Confluence, Obsidian, Markdown directories
- Export to: PDF (via headless Chrome or weasyprint), DOCX, EPUB
- Batch import with progress tracking
- Conflict resolution during import (merge vs. overwrite)

### 8.3 Mobile Responsive PWA

- Service worker for offline access
- Responsive layout for tablet/mobile viewports
- Touch-friendly interactions
- Push notifications

### 8.4 AI Integration Points

- Document summarization (local LLM or API)
- Smart search (semantic search via embeddings)
- Auto-tagging and categorization
- Writing assistant (grammar, style, completeness)

---

## Version Timeline

| Version | Phase | Target Date |
|---------|-------|-------------|
| 10.2.0 | Hardening | 2026-05-25 |
| 10.3.0 | Security Hardening | 2026-06-08 |
| 10.4.0 | Testing Expansion | 2026-06-29 |
| 11.0.0 | Performance | 2026-07-13 |
| 11.1.0 | Operational Maturity | 2026-07-27 |
| 12.0.0 | Architecture Improvements | 2026-08-17 |
| 12.1.0 | Frontend Polish | 2026-09-07 |
| 13.0.0 | Ecosystem | 2026-Q4 |

---

## Decision Log

| ID | Decision | Rationale | Date |
|----|----------|-----------|------|
| DL-001 | Use `lettre` for SMTP | Async-native, well-maintained, supports TLS | 2026-05-11 |
| DL-002 | Tantivy alongside tsvector | Tantivy for speed, tsvector as fallback for simplicity | 2026-05-11 |
| DL-003 | Unified `ApiError` over per-module types | Consistency, single source of truth for error codes | 2026-05-11 |
| DL-004 | Edition 2024 upgrade deferred to Phase 6 | Low risk, low urgency; focus on hardening first | 2026-05-11 |
| DL-005 | CSP nonce over hash-based | Nonce is simpler to implement in dynamic template rendering | 2026-05-11 |
