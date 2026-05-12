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

### 2.2 Secret Rotation Support [Low]

JWT secret loaded once at startup. No rotation without downtime.

**Scope:**
- Support comma-separated `TACHYON_JWT_SECRET` (old, new) for graceful rotation
- Validate tokens against all secrets, sign with first (newest)
- Add `/api/v1/auth/rotate-secret` admin endpoint
- Document rotation procedure in deployment runbook

**Files:** `crates/server/src/middleware/auth.rs`, `crates/server/src/config.rs`

### 2.3 Prometheus `.expect()` Removal [Medium]

`crates/server/src/lib.rs:765` uses `.expect("failed to install Prometheus metrics recorder")` which panics if the global recorder is already installed.

**Scope:**
- Replace with `match` or `if let` to gracefully handle already-installed recorder
- Log a warning instead of panicking

**Files:** `crates/server/src/lib.rs`

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

### 3.3 Middleware Chain Integration Tests [Medium]

Individual middleware modules have unit tests but the combined pipeline is untested.

**Scope:**
- Test auth + rate_limit interaction (authenticated requests bypass login rate limit)
- Test request_id propagation through entire chain
- Test audit logging captures correct data from authenticated requests
- Test cache_control + compression interaction
- Test security_headers + CORS ordering

**Effort:** 2-3 days. 10-15 test cases.

### 3.4 CRDT WebSocket Integration Tests [Medium]

`crates/server/src/crdt.rs` has unit tests but no multi-client WebSocket integration tests.

**Scope:**
- Test 2 clients connect, one edits, other receives update
- Test 3 clients concurrent edit, verify convergence
- Test client disconnect, reconnect, state recovery
- Test presence detection (join/leave/ttl expiry)

**Effort:** 3-5 days. Requires WebSocket test infrastructure.

### 3.5 Coverage Threshold Enforcement [Medium]

`.coverage.toml` sets `fail_under_threshold = false`.

**Scope:**
- Set `fail_under_threshold = true` with minimum 60% branch coverage
- Add `fail_under_critical = 95` for crates: core, rbac, server
- Wire into CI pipeline
- Add coverage trend tracking (compare against baseline)

**Effort:** 1 day.

### 3.6 E2E Tests: Collaboration, Billing, SSG [Medium]

Playwright E2E tests only cover auth, documents, navigation.

**Scope:**
- Collaboration: 2 browser tabs, type in one, verify in other
- SSG: trigger build, download zip, verify contents
- Onboarding: complete 4-step wizard
- File upload: drag-and-drop a file, verify it appears

**Effort:** 5-7 days.

---

## Phase 4: Performance (v11.0.0) -- 2 weeks

### 4.1 Wire Response Cache [Medium]

`ApiCache` exists with 60s TTL but is not wired into any route handlers.

**Scope:**
- Apply cache layer to: `GET /api/v1/documents` (list), `GET /api/v1/documents/search`, `GET /api/v1/catalog/stats`
- Cache invalidation on document create/update/delete
- Add `X-Cache-Status` response header (HIT/MISS/STALE)
- Benchmark: measure P95 latency with and without cache

**Files:** `crates/server/src/middleware/api_cache.rs`, `crates/server/src/routes/document.rs`

### 4.2 Tantivy Integration [Medium]

`tachyon-search` has a full Tantivy-based index but the server uses PostgreSQL `tsvector`.

**Scope:**
- Create Tantivy index manager as a server state component
- Index documents on create/update/delete via repository events
- Wire search route to use Tantivy for queries, PostgreSQL as fallback
- Implement index lifecycle: create on startup, reindex command, index health check
- Benchmark: Tantivy vs tsvector for 10K, 100K, 1M documents

**Effort:** 5-7 days.

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

### 5.3 Readiness Probe Expansion [Medium]

`/health` only checks database connectivity.

**Scope:**
- Check PostgreSQL connectivity (existing)
- Check Redis connectivity (if configured)
- Check Tantivy index health (if configured)
- Check SMTP connectivity (if configured)
- Return JSON with component-level status

**Files:** `crates/server/src/routes/health.rs`

### 5.4 Structured JSON Logging [Low]

`tracing_subscriber` uses default fmt output.

**Scope:**
- Add `TACHYON_LOG_FORMAT=json` config option
- Use `tracing_subscriber::fmt::json()` for structured output
- Add request ID to all log entries (correlation)
- Add module-level log filtering via `TACHYON_LOG_FILTER`

**Files:** `crates/server/src/main.rs`

### 5.5 Migration Rollback Support [Medium]

No `migrations::rollback()` exists. Bad migrations require manual SQL.

**Scope:**
- Generate down-migration for each existing migration
- Add `tachyon-server migrate rollback [steps]` CLI command
- Add `tachyon-server migrate status` to show current version
- CI gate: no migration without corresponding down-migration

**Files:** `crates/database/src/migrations.rs`

### 5.6 CD Pipeline [Medium]

CI exists but no automated release/deployment pipeline.

**Scope:**
- Add `release.yml` workflow: tag trigger, build multi-arch Docker images, push to registry
- Add `deploy-staging.yml`: deploy to staging on merge to `develop`
- Add `deploy-production.yml`: deploy to production on tag (manual approval)
- Add rollback step in each deployment
- SBOM generation and attestation per release

**Files:** `.github/workflows/release.yml`, `.github/workflows/deploy-staging.yml`

### 5.7 Monitoring Dashboard [Medium]

Prometheus endpoint exists but no Grafana dashboards.

**Scope:**
- Create Grafana dashboards: API latency (P50/P95/P99), request rate, error rate, active WebSocket connections, database pool utilization, cache hit rate
- Define SLOs: 99.9% availability, P99 < 100ms for read endpoints, P99 < 500ms for write endpoints
- Alert rules: error rate > 1%, P99 > 200ms, DB pool exhaustion, disk > 80%
- Document alerting runbook

**Files:** New `monitoring/grafana/`, `monitoring/alerts/`

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
