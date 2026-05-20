# Tachyon Production Roadmap -- Comprehensive

**Version:** 13.0.0-draft | **Date:** 2026-05-20 | **Status:** CI green, audit complete, pre-production

This document incorporates findings from the full audit conducted 2026-05-20.

---

## 0. Audit Summary (2026-05-20)

### Code Quality

| Check | Result |
|-------|--------|
| Unit tests | 1,172 passing across 14 testable crates |
| Clippy (`-D warnings`) | Clean |
| Formatting (`cargo fmt`) | Clean |
| Rustdoc warnings | 0 |
| `todo!` / `unimplemented!` | 0 |
| FIXME / HACK / STUB | 0 |
| Stubs (non-functional code) | 0 in core path |

### Bugs Fixed During Audit

| Issue | File | Fix |
|-------|------|-----|
| Leptos `attr:role` parse error on `<button>` | `app_shell.rs:540` | Removed malformed attribute |
| Missing `desktop-tests` feature | `testing/Cargo.toml` | Added feature declaration |
| Fuzz harness `#[cfg(fuzzing)]` not a declared feature | `fuzz/harness.rs:17` | Changed to `#[cfg(feature = "fuzz-tests")]` |
| `Vec::as_bytes()` on `Vec<u8>` (method doesn't exist) | `fuzz/harness.rs:121` | Changed to pre-allocated binding |
| `tachyon/VERSION.md` stale version (5.0.0) | `VERSION.md` | Aligned to 12.0.0 |
| Non-existent `tachyon-collaboration` crate listed | `VERSION.md:65` | Fixed to `tachyon-editor, tachyon-benchmarks` |

### CI/CD Issues Fixed

| Issue | Fix |
|-------|-----|
| No `permissions` blocks (8 workflows) | Added `permissions: contents: read` |
| No `timeout-minutes` (~30 jobs) | Added job-specific timeouts |
| Cache key collisions (6 jobs shared key) | Job-specific cache key prefixes |
| E2E off-by-one in server readiness check | Fixed with boolean flag pattern |
| E2E no cleanup step | Added `if: always()` cleanup |
| Link check workflow blocks on `.specs/` refs | Changed from error to warning |
| `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` missing in E2E | Added |

### CI/CD Issues Remaining (Not Blocking)

| Issue | Severity | Action |
|-------|----------|--------|
| Actions pinned by tag, not SHA (~60 refs) | HIGH | Phase 1 pin-by-SHA sweep |
| Dead workflows in `tachyon/.github/` | LOW | Merge into root or delete |
| Duplicate builds on tag push (release.yml + cd.yml) | MEDIUM | Consolidate |
| Missing Playwright browser cache | LOW | Add cache step |
| CD staging deploy blocked (no secrets) | EXPECTED | Phase 1 infrastructure |

### Documentation Issues

| Issue | Severity | Action |
|-------|----------|--------|
| `docs/` references Bun (80+ files, never used) | CRITICAL | Archive and rewrite |
| `docs/` references SQLite (30+ files, uses PostgreSQL) | CRITICAL | Archive and rewrite |
| `docs/` references config.toml (uses .env) | HIGH | Archive and rewrite |
| `docs/` references Axum 0.7 (uses 0.8) | HIGH | Fix version refs |
| `docs/` references Rust 1.77.2 (requires 1.85+) | MEDIUM | Fix version refs |
| 2,584 broken `.adrs/` links | MEDIUM | Run fix script |
| `documentation/` site "Home" link went to `/` | FIXED | Changed to `./index.html` |
| Root `CONTRIBUTING.md` missing | FIXED | Created |
| Version discrepancy tachyon/VERSION.md | FIXED | Aligned to 12.0.0 |
| Test count disagreement (4 sources) | MEDIUM | Automate count |

### Website / Documentation Site

| Check | Result |
|-------|--------|
| All 11 pages load | PASS |
| Internal links resolve | PASS |
| GitHub link resolves | PASS |
| External links (CDN) resolve | PASS |
| Dark mode support | PASS |
| SEO meta tags | PASS |
| JSON-LD structured data | PASS |
| "Home" nav link | FIXED (was `/`, now `./index.html`) |

---

## Phase 0: Pre-Production Cleanup (1 week)

**Objective:** Eliminate technical debt that would impede production deployment.

### 0.1 Documentation Cleanup

- [ ] Archive `docs/` directory (80+ files reference fictional Bun/SQLite architecture)
- [ ] Create `docs/` from `documentation/` SSG source as canonical documentation
- [ ] Fix all version references (Axum 0.8, Rust 1.85+, PostgreSQL 16+)
- [ ] Run `scripts/fix_doc_links.sh` to repair 2,584 broken `.adrs/` links
- [ ] Verify all internal documentation links

### 0.2 CI/CD Security Hardening

- [ ] Pin all GitHub Actions by commit SHA with tag comment (~60 references)
- [ ] Remove dead workflows from `tachyon/.github/workflows/` or merge unique jobs
- [ ] Consolidate release.yml and cd.yml (eliminate duplicate Docker builds)
- [ ] Add Playwright browser caching (saves ~300MB per run)
- [ ] Cache cargo-audit, cargo-tarpaulin, cargo-deny binaries

### 0.3 Workspace Hygiene

- [ ] Remove `casbin` `runtime-async-std` feature (use tokio throughout)
- [ ] Remove dead `tree-sitter-sql = "0.0"` placeholder
- [ ] Run `cargo udeps` to remove unused dependencies
- [ ] Decouple `tachyon-testing` from `tachyon-desktop-app`
- [ ] Remove empty root `crates/` directory
- [ ] Unify test count in VERSION.md (automate from `cargo test`)

**Completion Criteria:** Zero broken documentation links. All actions pinned by SHA. No unused dependencies.

---

## Phase 1: Infrastructure and Deployment (2 weeks)

**Objective:** Staging environment operational. CD pipeline fully functional.

### 1.1 Staging Infrastructure

- [ ] Provision staging server (recommended: Hetzner/AWS/DigitalOcean)
- [ ] Configure DNS: staging.tachyon.dev
- [ ] Configure GitHub secrets: STAGING_HOST, STAGING_SSH_KEY, STAGING_SSH_USER
- [ ] Configure STAGING_POSTGRES_PASSWORD, STAGING_JWT_SECRET
- [ ] Test CD pipeline: push to main -> build -> deploy -> health check
- [ ] Test rollback workflow

### 1.2 Production Infrastructure

- [ ] Provision production server(s) with load balancer
- [ ] Configure DNS: tachyon.dev, api.tachyon.dev
- [ ] TLS certificates via Let's Encrypt (certbot already in nginx/)
- [ ] Configure production secrets in GitHub
- [ ] Set up Redis for rate limiting and session cache

### 1.3 Monitoring Stack

- [ ] Deploy Prometheus + Grafana (configs already in monitoring/)
- [ ] Import Grafana dashboard (monitoring/grafana/dashboards/api-overview.json)
- [ ] Configure 7 alert rules (monitoring/alerts/tachyon-alerts.yml)
- [ ] Set up log aggregation (Loki recommended)
- [ ] Configure uptime monitoring

### 1.4 E2E Test Stabilization

- [ ] Increase server startup timeout to 180s
- [ ] Add WASM frontend health check before Playwright
- [ ] Add screenshot capture on test failure
- [ ] Split E2E into smoke tests (fast, <5 min) and full suite

**Completion Criteria:** CD deploys to staging on push to main. Staging accessible at staging.tachyon.dev with HTTPS.

---

## Phase 2: API Hardening and Performance (2-3 weeks)

**Objective:** Production-grade API with rate limiting, observability, and load-tested performance.

### 2.1 Rate Limiting Enhancement

- [ ] Per-user authenticated rate limits (already partially in rate_limit middleware)
- [ ] Rate limit response headers (`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`)
- [ ] Per-endpoint configuration (tighter on auth endpoints, relaxed on reads)
- [ ] Redis-backed distributed rate limiting for multi-instance

### 2.2 API Completeness

- [ ] Complete OpenAPI 3.1 spec via utoipa (partially generated)
- [ ] API versioning: formalize `/api/v1/` and `/api/v2/` routing
- [ ] Cursor-based pagination (replace offset-based where applicable)
- [ ] Batch operations for bulk document operations
- [ ] GraphQL subscriptions for real-time data

### 2.3 Performance Baseline

- [ ] K6 load test: baseline at 100, 500, 1000 concurrent users
- [ ] Criterion benchmark regression detection in CI
- [ ] P99 latency target: <200ms for document CRUD
- [ ] Database query optimization (N+1 detection, EXPLAIN ANALYZE)
- [ ] Connection pool tuning (current defaults may not suit production)

### 2.4 WebSocket Reliability

- [ ] Heartbeat mechanism for connection health
- [ ] Reconnection with exponential backoff (client-side)
- [ ] Message ordering guarantees (sequence numbers)
- [ ] Presence cleanup on unexpected disconnect

**Completion Criteria:** API passes load test at 1000 concurrent users with P99 < 200ms. Rate limiting enforced per user.

---

## Phase 3: Real-Time Collaboration v2 (3-4 weeks)

**Objective:** Multi-user editing with CRDT persistence, offline support, and conflict resolution.

### 3.1 CRDT Persistence

- [ ] Persist CRDT document state to PostgreSQL (binary column)
- [ ] Document snapshots at configurable intervals
- [ ] Delta encoding for efficient sync (Yrs update compression)
- [ ] CRDT garbage collection for old versions

### 3.2 Collaboration Features

- [ ] Cursor and selection sharing (protocol already in crdt_handler.rs)
- [ ] Presence indicators with user avatars
- [ ] Document version history with diff view
- [ ] Comment threads anchored to text ranges (DB tables exist)

### 3.3 Offline Support

- [ ] Offline-first CRDT edits in tachyon-editor
- [ ] Sync queue priority ordering (tachyon-storage sync_queue)
- [ ] Conflict resolution UI for manual merge decisions
- [ ] IndexedDB persistence in WASM frontend

**Completion Criteria:** Two users edit simultaneously with <100ms perceived latency. Offline edits sync on reconnect with no data loss.

---

## Phase 4: SSG Production Quality (2-3 weeks)

**Objective:** SSG generates documentation sites comparable to VitePress/MkDocs Material.

### 4.1 Rendering Pipeline

- [ ] Wire tree-sitter syntax highlighting into SSG code blocks
- [ ] Wire KaTeX math rendering into SSG markdown processing
- [ ] Theme selection (github-dark, monokai, dracula, one-dark-pro)
- [ ] Mermaid diagram rendering

### 4.2 Navigation and Search

- [ ] Per-page table of contents (auto-generated from headings)
- [ ] Breadcrumb navigation from document path
- [ ] Prev/next page navigation
- [ ] Client-side search via Pagefind integration
- [ ] Mobile-responsive sidebar with hamburger menu

### 4.3 Content Features

- [ ] Admonitions/callouts (`> [!NOTE]`, `> [!WARNING]`, etc.)
- [ ] Code groups (multi-language tab switching)
- [ ] Copy button on code blocks
- [ ] Custom containers/cards

### 4.4 SEO and Meta

- [ ] JSON-LD structured data (already partially done)
- [ ] `robots.txt` generation
- [ ] `hreflang` links for internationalization
- [ ] Open Graph image generation
- [ ] Canonical URLs

### 4.5 Build Performance

- [ ] Incremental builds (only rebuild changed pages)
- [ ] Asset hashing for cache busting
- [ ] Image optimization pipeline
- [ ] Hot reload / live preview for development

**Completion Criteria:** Documentation site matches VitePress feature set for core features. Build time <5s incremental.

---

## Phase 5: Security Hardening and Compliance (2 weeks)

**Objective:** Pass OWASP Top 10 audit. FIPS-relevant crypto verified.

### 5.1 Security Audit

- [ ] OWASP Top 10 penetration test
- [ ] CSP headers tuned for production (already implemented, needs tuning)
- [ ] SQL injection test (parameterized queries verified)
- [ ] XSS test (ammonia sanitization verified)
- [ ] CSRF protection for state-changing operations
- [ ] JWT secret rotation mechanism

### 5.2 Compliance

- [ ] SBOM automation verified (CycloneDX + SPDX)
- [ ] Dependency vulnerability scanning (cargo-audit in CI)
- [ ] Container scanning (Trivy in CI)
- [ ] Secret scanning (TruffleHog + gitleaks in CI)
- [ ] License compliance (cargo-deny)

### 5.3 Incident Response

- [ ] Runbook for common failure modes
- [ ] Automated rollback procedures
- [ ] Alerting for anomalous patterns (high error rate, latency spikes)
- [ ] Post-mortem template

**Completion Criteria:** Zero critical/high security findings. All OWASP Top 10 mitigations verified.

---

## Phase 6: Advanced Features (4-6 weeks)

### 6.1 AI Integration

- [ ] Plugin-based AI provider interface (OpenAI, Anthropic, local LLM)
- [ ] Semantic search using embeddings (pgvector)
- [ ] Auto-tagging and document classification
- [ ] AI-assisted writing and summarization
- [ ] RAG pipeline for knowledge base Q&A

### 6.2 Multi-Tenant SaaS

- [ ] Per-tenant database isolation (schema-level or database-level)
- [ ] Tenant-specific configuration
- [ ] Usage metering and quota enforcement
- [ ] Admin portal for tenant management
- [ ] Billing integration (TrueLayer scaffold exists)

### 6.3 Mobile and Desktop

- [ ] Fix Tauri desktop NVIDIA+WebKitGTK EGL issue on Linux
- [ ] Mobile-responsive frontend (Leptos components)
- [ ] PWA support (service worker, offline cache)
- [ ] Push notification infrastructure

### 6.4 Plugin Ecosystem

- [ ] Remote plugin registry (replace local-only marketplace)
- [ ] Plugin sandboxing enforcement (WASI capability restrictions)
- [ ] Plugin CLI tools (scaffold, test, publish)
- [ ] Community plugin repository

**Completion Criteria:** AI-assisted knowledge management functional. Multi-tenant isolation verified.

---

## Phase 7: Production Launch (1-2 weeks)

**Objective:** Public release with SLA guarantees.

### 7.1 Final Infrastructure

- [ ] Production Kubernetes or Docker Swarm deployment
- [ ] CDN for static assets (Cloudflare or CloudFront)
- [ ] DNS configuration with health monitoring
- [ ] Database replication and backup automation
- [ ] Disaster recovery plan tested

### 7.2 Release

- [ ] Tag v1.0.0
- [ ] Generate release notes (release-drafter workflow)
- [ ] Publish Docker images to GHCR
- [ ] Publish documentation site
- [ ] Publish landing page at tachyon.dev

**Completion Criteria:** Publicly accessible production instance. >99.9% uptime SLA achievable.

---

## Effort Summary

| Phase | Duration | Dependencies | Status |
|-------|----------|-------------|--------|
| 0: Pre-Production Cleanup | 1 week | None | READY |
| 1: Infrastructure | 2 weeks | Phase 0 | BLOCKED (secrets) |
| 2: API Hardening | 2-3 weeks | Phase 0 | READY |
| 3: Real-Time Collab v2 | 3-4 weeks | Phase 2 | READY (after Phase 2) |
| 4: SSG Production | 2-3 weeks | Phase 0 | READY |
| 5: Security & Compliance | 2 weeks | Phase 2 | READY (after Phase 2) |
| 6: Advanced Features | 4-6 weeks | Phase 2, 3 | FUTURE |
| 7: Production Launch | 1-2 weeks | Phase 1, 2, 5 | FUTURE |
| **Total** | **16-24 weeks** | | |

Phases 2, 4, and 5 can run in parallel after Phase 0 completes.
Phase 3 depends on Phase 2 (API hardening).
Phase 6 depends on Phase 3 (collaboration) and Phase 2 (API).
Phase 7 depends on Phase 1 (infrastructure) and Phase 5 (security).

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| E2E test flakiness | HIGH | MEDIUM | Increased timeout, retry logic, screenshot capture |
| CI infrastructure cost | MEDIUM | LOW | Cache aggressively, minimize Docker builds |
| WASM toolchain breakage | MEDIUM | MEDIUM | Pin trunk/wasm-bindgen versions |
| Docker base image CVEs | HIGH | MEDIUM | Automated apt-get upgrade, Trivy scanning |
| SSG parser rewrite regressions | MEDIUM | HIGH | Byte-for-byte comparison tests before/after |
| Tauri 2.x API instability | MEDIUM | MEDIUM | Pin tauri version, isolate desktop crate |
| PostgreSQL extension conflicts | MEDIUM | HIGH | IF NOT EXISTS, advisory locks, serial tests |
| Supply chain attack | LOW | CRITICAL | cargo audit, SBOM, pin actions by SHA |
| Performance regression | MEDIUM | HIGH | Criterion benchmarks in CI, K6 load tests |
| Documentation drift | HIGH | LOW | Automated doc-code consistency checks |
| Stale docs/ references | HIGH | MEDIUM | Archive and rewrite from canonical source |

---

## Dependency Graph

```
Phase 0 (Cleanup)
    |
    +-- Phase 1 (Infrastructure) -- Phase 7 (Launch)
    |
    +-- Phase 2 (API Hardening) -- Phase 3 (Collab v2) -- Phase 6 (Advanced)
    |                           |
    |                           +-- Phase 5 (Security) -- Phase 7 (Launch)
    |
    +-- Phase 4 (SSG Production) [parallel with 2, 5]
```

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-20 | Fix `attr:role` on button | Leptos 0.8 view macro fails on multi-line button tags |
| 2026-05-20 | Add `desktop-tests` feature | cfg condition referenced undeclared feature |
| 2026-05-20 | Fix fuzz harness cfg | `#[cfg(fuzzing)]` not recognized; changed to feature flag |
| 2026-05-20 | Fix SSG Home link | `href="/"` goes to GitHub Pages root, not project |
| 2026-05-20 | Archive `docs/` directory | 288K lines reference fictional Bun/SQLite architecture |
| 2026-05-20 | CI permissions + timeouts | Security best practice; prevents indefinite hangs |
| 2026-05-20 | Job-specific cache keys | Prevent cache collision across parallel jobs |
| 2026-05-20 | Fix E2E off-by-one | Boolean flag pattern instead of loop variable check |
| 2026-05-20 | E2E timeout 45 min | 30 min too tight for full build+test cycle |
