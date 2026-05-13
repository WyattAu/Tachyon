# Tachyon Roadmap to Production and Beyond

**Version:** 10.1.1 | **Date:** 2026-05-13 | **Status:** 1,358 tests, 0 failures

---

## Executive Summary

Tachyon is a Rust-based knowledge management system with 16 crates, 16 deployment targets
(CLI, server, desktop, SSG, frontend), and 1,358 passing tests. This document describes the
phased path from the current state to production readiness, followed by a long-term
evolution plan.

The codebase is structurally sound: zero `todo!()`, zero `unimplemented!()`, zero `FIXME`,
zero `HACK` in production code. Clippy passes with `-D warnings`. All formatting is clean.
The pre-commit hook enforces fmt, clippy, tests, rustdoc, secret detection, and artifact
checks on every commit.

Remaining work falls into three categories: (1) security hardening, (2) operational
maturity, and (3) feature completion.

---

## Current Architecture

```
tachyon/
  crates/
    core/           -- Domain types, error handling, utilities
    server/         -- Axum REST API, WebSocket, middleware stack (13 layers)
    database/       -- PostgreSQL persistence (35+ tables, 28 migrations)
    search/         -- Tantivy full-text index (BM25, faceted search)
    renderer/       -- pulldown-cmark Markdown-to-HTML with GFM extensions
    rbac/           -- Role-based access control with hierarchical permissions
    storage/        -- SQLite (offline) + PostgreSQL (server) dual-backend
    ssg/            -- Static site generator with i18n, RTL, sitemap, RSS
    plugin-runtime/ -- WASM plugin execution with marketplace registry
    editor/         -- Syntax highlighting, markdown parsing utilities
    cli/            -- CLI binary (init, serve, build, gui)
    frontend/       -- Leptos WASM SPA (90+ components, 30+ pages)
    desktop/        -- Tauri desktop wrapper (single-process, embedded server)
    import-export/  -- Document import/export (Markdown, HTML, JSON)
    testing/        -- Test infrastructure (TestApp, fixtures, helpers)
    benchmarks/     -- Criterion benchmarks (search, renderer, RBAC)
```

---

## Phase A: Security Hardening (v10.2.0) -- 2 weeks

### A.1 HTML Sanitization in Renderer Output [Critical]

The markdown renderer passes raw HTML blocks through unescaped. While `ammonia` is
available in the server validation layer, the renderer crate itself does not sanitize.

**Action:** Add `ammonia::clean()` to the renderer's HTML output path. Create a
`SanitizedHtml` type that wraps the cleaned output. Add 5+ property-based tests
verifying that `<script>`, `onerror=`, `javascript:`, and SVG-based XSS vectors
are stripped.

**Files:** `tachyon/crates/renderer/src/lib.rs`, new `tachyon/crates/renderer/src/sanitize.rs`
**Effort:** 2 days

### A.2 CSP Nonce Injection into Templates [High]

CSP nonce generation exists in `security_headers.rs` but `'unsafe-inline'` remains in
`style-src`. The nonce must be injected into all inline `<style>` and `<script>` blocks.

**Action:** Add nonce to the SSG template context. Remove `'unsafe-inline'` from
`style-src` and `script-src` in production mode. Add integration test verifying
CSP header matches page content.

**Files:** `tachyon/crates/server/src/middleware/security_headers.rs`,
`tachyon/crates/ssg/src/templates.rs`
**Effort:** 2 days

### A.3 `unwrap()` Audit -- Server Request Paths [High]

47 `unwrap()` calls exist in production code. Most are in hardcoded regex compilation
(which is infallible by construction). The actionable ones are:

| Location | Risk | Fix |
|----------|------|-----|
| `rate_limit.rs:226,251,264,364` | System clock before UNIX epoch | Replace with `UNIX_EPOCH.elapsed().unwrap_or_default()` |
| `billing/handlers.rs:28` | HMAC key construction | Replace with `HmacSha256::new_from_slice()` returning `Result` |
| `security_headers.rs:18` | CSPRNG unavailable | Log error and return a static nonce (degraded mode) |
| Frontend DOM `.unwrap()` (20+ locations) | Browser context failure | Replace with `.ok()` + fallback or error boundary |

**Effort:** 3 days

### A.4 Frontend Wire-Up: Logout, i18n, Role Editing [Medium]

Three features have backend support but are not connected in the UI:

1. `logout()` in `auth_guard.rs` -- wire to user menu dropdown
2. `use_locale()` in `i18n/mod.rs` -- wire to settings page language selector
3. `UpdateRoleRequest` in `admin/roles.rs` -- wire to role editing form

**Effort:** 3 days

### A.5 Dead Code Cleanup [Medium]

100+ `#[allow(dead_code)]` annotations remain, concentrated in frontend API client
modules. Each annotation represents an API client function that is defined but not
called from any component.

**Action:** Audit each annotation. Remove genuinely unused code. Wire endpoints that
have working backend routes. Leave only feature-gated items annotated.

**Effort:** 3 days

---

## Phase B: Operational Maturity (v10.3.0) -- 3 weeks

### B.1 Email Delivery Integration [High]

`EmailService::send()` with `lettre` 0.11 exists but falls back to log-and-skip when
`SMTP_URL` is unset. Password reset, email verification, and notification emails are
silently dropped.

**Action:** Verify end-to-end email delivery with a real SMTP provider (Resend, SES,
or self-hosted Postfix). Add integration test with a mock SMTP server. Document SMTP
configuration.

**Effort:** 2 days

### B.2 Database Backup Automation [High]

No automated PostgreSQL backup or point-in-time recovery is configured.

**Action:** Add `pg_dump` cron job to the Docker deployment. Configure WAL archiving.
Add backup verification (restore test). Document backup/restore procedures in a
runbook.

**Files:** `docker-compose.prod.yml`, new `scripts/backup.sh`, new `docs/runbook.md`
**Effort:** 2 days

### B.3 SSL/TLS Termination [Medium]

TLS config fields exist in `ServerConfig` but are not wired into the Docker/nginx
deployment.

**Action:** Add Certbot/Let's Encrypt sidecar to `docker-compose.prod.yml`. Configure
nginx to terminate TLS and proxy to the backend over HTTP. Add certificate renewal
automation.

**Files:** `nginx/nginx.conf`, `docker-compose.prod.yml`
**Effort:** 2 days

### B.4 Readiness Health Checks [Medium]

`/health` returns "OK". `/ready` checks database connectivity only.

**Action:** Extend `/ready` to check Redis (if configured), SMTP (if configured),
and storage backend availability. Return 503 with diagnostic details when any
dependency is unreachable.

**Files:** `tachyon/crates/server/src/routes/health.rs`
**Effort:** 1 day

### B.5 Structured JSON Logging [Medium]

`tracing_subscriber` uses default (human-readable) output. Production deployments need
JSON logs for aggregation.

**Action:** Add `TACHYON_LOG_FORMAT=json` env var. When set, use `tracing_subscriber::fmt::json()`. Add per-module log level configuration via `TACHYON_LOG_FILTER`.

**Files:** `tachyon/crates/server/src/main.rs`
**Effort:** 1 day

### B.6 Migration Rollback Support [Medium]

`migrations::run_migrations()` applies all pending migrations forward. No rollback.

**Action:** Add `migrations::rollback(n: usize)` function. Add CLI subcommand
`tachyon migrate rollback --steps N`. Document manual rollback procedure.

**Files:** `tachyon/crates/database/src/migrations.rs`
**Effort:** 2 days

### B.7 CD Pipeline for Automated Releases [Medium]

CI exists (build, test, lint, security scan). No CD for automated deployment.

**Action:** Add GitHub Actions workflow triggered by semver tags. Build multi-arch
Docker image, push to registry, deploy to staging. Add manual approval gate for
production deployment.

**Files:** `.github/workflows/cd.yml`
**Effort:** 2 days

---

## Phase C: Performance and Scale (v10.4.0) -- 2 weeks

### C.1 Response Caching for Read-Heavy Endpoints [High]

`ApiCache` middleware exists with 60s TTL but is not wired into any route handlers.

**Action:** Apply cache layer to `GET /api/v1/documents`, `GET /api/v1/search`,
`GET /api/v1/spaces`. Add cache invalidation on write operations (document update,
space update). Measure cache hit rate under load.

**Files:** `tachyon/crates/server/src/middleware/api_cache.rs`
**Effort:** 3 days

### C.2 Tantivy Search Integration [High]

`tachyon-search` crate has a full Tantivy-based index but the server uses PostgreSQL
`tsvector` for search. Tantivy provides better ranking (BM25), faceting, and
suggestion support.

**Action:** Wire `tachyon-search` into the server's `/api/v1/search` endpoint.
Implement index lifecycle management (create on startup, update on document
write, reindex API). Add benchmark comparing Tantivy vs tsvector at 10K, 100K,
1M documents.

**Files:** `tachyon/crates/search/`, `tachyon/crates/server/src/routes/search.rs`
**Effort:** 5 days

### C.3 Database Query Optimization [Medium]

28 migrations exist but no systematic query performance analysis.

**Action:** Run `EXPLAIN ANALYZE` on the 10 most frequently called queries.
Add composite indexes where needed (the `20260428000000_composite_indexes.sql`
migration is a start). Document query performance baseline.

**Effort:** 2 days

### C.4 Load Testing and Connection Pool Tuning [Medium]

Initial benchmarks (200 concurrent requests) showed sub-ms latency. No sustained
load tests exist.

**Action:** Deploy k6 load test suite. Test at 1K, 5K, 10K concurrent connections.
Measure p50/p95/p99 latency, error rate, memory usage. Tune `DbPoolConfig`
based on results.

**Files:** new `load-tests/k6/`
**Effort:** 3 days

### C.5 WASM Bundle Size Optimization [Low]

Release profile uses `opt-level = "z"` and LTO but no WASM-specific tuning.

**Action:** Run `wasm-opt` with `-Oz`. Measure bundle size before/after. Add
tree-shaking verification. Document bundle size budget (< 500KB compressed).

**Effort:** 2 days

---

## Phase D: Testing Completion (v10.5.0) -- 2 weeks

### D.1 Integration Test Stubs [High]

Three integration test modules are stubs:

1. `crates/testing/src/integration/rbac.rs` -- placeholder RBAC enforcement test
2. `crates/testing/src/integration/ipc.rs` -- placeholder desktop IPC test
3. `crates/testing/src/fuzz/harness.rs` -- placeholder fuzz harness

**Action:** Implement real RBAC integration tests covering document, space, team,
and organization routes with different role levels. Implement at least 3 fuzz
targets for the API (JSON deserialization, markdown rendering, search query
parsing). Leave IPC tests gated on Tauri runtime availability.

**Effort:** 5 days

### D.2 E2E Test Expansion [Medium]

Playwright E2E tests cover auth, documents, and navigation only.

**Action:** Add E2E tests for: collaboration (WebSocket sync), billing flow,
SSG build/download, file upload, onboarding wizard, admin operations.

**Files:** `tachyon/e2e/tests/`
**Effort:** 5 days

### D.3 Coverage Threshold Enforcement [Medium]

`.coverage.toml` has `fail_under_threshold = false`.

**Action:** Set `fail_under_threshold = true`. Add `cargo-tarpaulin` step to CI
that fails the build if coverage drops below 80% overall, 95% on critical paths.

**Files:** `.coverage.toml`, `.github/workflows/ci.yml`
**Effort:** 1 day

### D.4 Middleware Chain Integration Tests [Medium]

Individual middleware modules have unit tests but the combined stack is not tested
as a pipeline.

**Action:** Add integration test that creates a full middleware stack and verifies
request propagation, header injection order, and error handling across the
complete chain.

**Effort:** 2 days

---

## Phase E: Documentation and Accessibility (v10.6.0) -- 2 weeks

### E.1 API Reference Publication [Medium]

OpenAPI spec generation exists in `api_docs.rs` but is not published as user-facing
documentation.

**Action:** Auto-generate API reference from OpenAPI spec. Publish to `docs/api/`.
Add example requests/responses for every endpoint.

**Effort:** 3 days

### E.2 Error Code Reference [Medium]

Error codes are scattered across route modules.

**Action:** Create centralized error code registry in `docs/error-codes.md`. Map
each code to HTTP status, description, and remediation steps.

**Effort:** 1 day

### E.3 Operational Runbooks [Medium]

No incident response documentation exists.

**Action:** Write runbooks for: database outage, WebSocket storm, failed migration,
certificate expiry, security incident, performance degradation.

**Files:** new `docs/runbooks/`
**Effort:** 3 days

### E.4 Accessibility Audit [High]

Accessibility primitives exist but no audit has been performed.

**Action:** Run automated axe-core audit against all 30+ pages. Fix violations.
Verify keyboard navigation and screen reader compatibility (NVDA).

**Effort:** 3 days

### E.5 Monitoring Dashboards [Medium]

Prometheus metrics endpoint exists but no Grafana dashboards.

**Action:** Create Grafana dashboards for: request latency, error rate, active
connections, cache hit rate, database pool utilization. Define alerting rules
for SLO violations.

**Files:** `monitoring/grafana/`
**Effort:** 3 days

---

## Phase F: Production Release (v11.0.0) -- 1 week

### F.1 Production Readiness Review

Gate criteria:

| Criterion | Target | Current |
|-----------|--------|---------|
| Tests passing | 100% | 100% (1,358/1,358) |
| Clippy warnings | 0 | 0 |
| Critical security issues | 0 | 0 (post Phase A) |
| E2E test coverage | Core flows | Auth, documents, nav |
| Load test (p99) | < 100ms | < 1ms (200 concurrent) |
| XSS sanitization | Complete | Pending (A.1) |
| CSP | No unsafe-inline | Pending (A.2) |
| SSL/TLS | Configured | Pending (B.3) |
| Database backup | Automated | Pending (B.2) |
| CD pipeline | Tag-triggered | Pending (B.7) |
| Monitoring | Dashboards + alerts | Pending (E.5) |
| Accessibility | WCAG 2.1 AA | Pending (E.4) |
| Error runbooks | All critical paths | Pending (E.3) |

### F.2 Release Checklist

1. All Phase A-E items completed and verified
2. Docker image built for `linux/amd64` and `linux/arm64`
3. Helm chart or docker-compose documentation published
4. Upgrade guide from v10.x to v11.0 published
5. CHANGELOG updated with all changes since v10.0.0
6. `VERSION.md` updated with final test counts and deployment verification
7. Security self-assessment updated

---

## Phase G: Post-Production Evolution (v11.x, v12.x) -- Ongoing

### G.1 Plugin Ecosystem (v11.1)

- Publish plugin SDK documentation and API
- Create plugin template repository
- Add plugin marketplace with CI-based verification
- Implement plugin permissions sandboxing (filesystem, network, API scopes)

### G.2 Real-Time Collaboration v2 (v11.2)

- CRDT persistence to PostgreSQL (currently in-memory)
- Offline-first with conflict resolution on reconnect
- Cursor and selection sharing with presence indicators
- Document history with diff view

### G.3 AI Integration (v11.3)

- Plugin-based AI provider interface (OpenAI, Anthropic, local LLM)
- Semantic search using embeddings (pgvector or Qdrant)
- Auto-tagging and document classification
- AI-assisted writing and summarization

### G.4 Mobile Applications (v12.0)

- React Native or Flutter mobile client
- Offline-first sync with CRDT
- Push notifications
- Biometric authentication

### G.5 Multi-Tenant SaaS (v12.1)

- Per-tenant database isolation (schema or physical)
- Tenant-specific configuration (features, branding, limits)
- Usage metering and quota enforcement
- Admin portal for tenant management

### G.6 Performance at Scale (v12.2)

- Horizontal scaling with stateless server nodes
- Redis-backed session and cache cluster
- Read replicas for database
- CDN for static assets and SSG output
- Query optimization for 10M+ document datasets

### G.7 Advanced Search (v12.3)

- Hybrid search (keyword + semantic)
- Natural language query understanding
- Cross-document relationship traversal
- Search result ranking personalization
- Search analytics and query suggestions

---

## Effort Summary

| Phase | Version | Duration | Items | Key Deliverable |
|-------|---------|----------|-------|-----------------|
| A | 10.2.0 | 2 weeks | 5 | Security hardening |
| B | 10.3.0 | 3 weeks | 7 | Operational maturity |
| C | 10.4.0 | 2 weeks | 5 | Performance and scale |
| D | 10.5.0 | 2 weeks | 4 | Testing completion |
| E | 10.6.0 | 2 weeks | 5 | Documentation and accessibility |
| F | 11.0.0 | 1 week | 2 | Production release |
| **Total to v11.0** | | **12 weeks** | **28** | **Production-ready** |
| G | 11.x-12.x | Ongoing | 7 | Post-production evolution |

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| XSS bypass in renderer | Medium | Critical | Phase A.1 + continuous fuzzing |
| Performance regression | Medium | High | Phase C.4 load tests in CI |
| Database migration failure | Low | Critical | Phase B.6 rollback support |
| Supply chain vulnerability | Medium | High | `cargo audit` in CI, SBOM generation |
| TLS certificate expiry | Low | High | Phase B.3 + monitoring alert |
| WebSocket connection storm | Medium | Medium | Rate limiting + connection limits |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-13 | Use `ammonia` for HTML sanitization | Already a dependency; battle-tested; OWASP-recommended |
| 2026-05-13 | Keep `tsvector` as fallback search | Tantivy adds deployment complexity; tsvector is sufficient for < 100K docs |
| 2026-05-13 | Single-tenant architecture for v11.0 | Multi-tenant isolation is a v12 feature; premature optimization |
| 2026-05-13 | Nix flake for development environment | Reproducible builds; pins Rust version and system dependencies |
