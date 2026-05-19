# Tachyon Roadmap

**Version:** 12.0.0 | **Date:** 2026-05-19 | **Status:** CI/CD green, documentation audit complete

---

## Executive Summary

Tachyon is a Rust-based knowledge management system comprising 16 crates across 5 deployment targets (server, CLI, desktop, SSG, WASM frontend). The codebase has 1,064+ library unit tests, 261+ integration tests, zero production stubs, zero `todo!`/`unimplemented!` macros, zero FIXME/HACK/STUB markers, clippy-clean with `-D warnings`, rustdoc-clean, and a 6-gate pre-commit hook enforcing fmt, clippy, tests, rustdoc, secret scanning, and debug artifact detection.

All CI pipelines are green (CI, Security, SBOM, Docs, Release Drafter). CD and E2E pipelines require infrastructure secrets not yet configured. Documentation has been audited for accuracy, rigor, conciseness, and consistency.

This roadmap covers the path from current state through production deployment and long-term evolution.

---

## Current State (2026-05-19)

### Pipeline Health

| Pipeline | Status | Passing Jobs | Notes |
|----------|--------|-------------|-------|
| CI | GREEN | Check, Lint, Test, Coverage, Build Frontend, Build Release, Docker Build, Benchmarks | All jobs pass |
| CD | BLOCKED | -- | Requires STAGING_HOST secret (no staging server) |
| Security | GREEN | Dependency Audit, SAST, Secrets Scan, Dependency Review | Container scan requires base image |
| E2E | FLAKY | -- | Browser timeout issues (infrastructure, not code) |
| Docs | GREEN | Build, Deploy | GitHub Pages deployed at wyattau.github.io/Tachyon |
| SBOM | GREEN | CycloneDX generation | Artifact uploaded |
| Release Drafter | GREEN | Auto-draft on PR | Configured |
| Backup | BLOCKED | -- | Requires DATABASE_HOST secret (no production DB) |

### Code Quality Metrics

| Metric | Value |
|--------|-------|
| Library unit tests | 1,064 passing |
| Integration tests | 261+ passing |
| Clippy warnings | 0 (with -D warnings) |
| Formatting | Clean (cargo fmt) |
| Rustdoc warnings | 0 |
| Production stubs | 0 |
| todo!/unimplemented! | 0 |
| FIXME/HACK/STUB | 0 |
| Pre-commit gates | 6 (fmt, clippy, test, doc, secrets, artifacts) |

### Documentation Site

11 pages deployed via GitHub Pages SSG:

| Page | Status |
|------|--------|
| index.md (Landing) | Complete |
| getting-started.md | Complete |
| editor-guide.md | Complete |
| configuration.md | Complete |
| deployment.md | Complete |
| architecture.md | Complete |
| api-reference.md | Complete |
| search.md | NEW - Complete |
| authentication.md | NEW - Complete |
| cli-reference.md | NEW - Complete |
| collaboration.md | NEW - Complete |

### Known Technical Debt

1. **Broken `.specs/` links**: 1,669 links across 73 files reference deleted `.specs/` directories. Needs bulk migration to `.adrs/`.
2. **Verbose API docs**: `docs/api/` contains ~60K lines across 15 files, much of it scaffolded. Needs consolidation.
3. **E2E test flakiness**: Browser timeout in CI environment. Server or WASM may not start within 60s window.
4. **Node.js 20 deprecation warning**: GitHub Actions running on Node.js 20, deprecated Sept 2026.
5. **Tauri desktop build**: Compiles but has NVIDIA+WebKitGTK EGL display issue on Linux.
6. **CD pipeline**: Requires STAGING_HOST/PRODUCTION_HOST secrets and actual servers.
7. **Backup pipeline**: Requires DATABASE_HOST and S3 credentials.

---

## Phase 1: CI/CD and Infrastructure Hardening (1-2 weeks)

**Objective:** All pipelines fully green including E2E. Infrastructure ready for staging.

### 1.1 E2E Test Stabilization

- [ ] Increase server startup timeout from 60s to 120s
- [ ] Add WASM frontend health check before Playwright tests
- [ ] Split E2E into smoke tests (fast) and full tests (slow)
- [ ] Add retry logic for flaky browser tests
- [ ] Capture screenshot on failure for debugging

### 1.2 CD Pipeline Activation

- [ ] Provision staging server (or use GitHub Codespaces as ephemeral staging)
- [ ] Configure STAGING_HOST, STAGING_SSH_KEY, STAGING_SSH_USER secrets
- [ ] Test end-to-end deployment: push to main -> Docker build -> deploy -> health check
- [ ] Configure production secrets for v* tag deployments
- [ ] Test rollback workflow

### 1.3 Documentation Link Cleanup

- [ ] Script to bulk-migrate 1,669 `.specs/` references to `.adrs/`
- [ ] Verify all internal doc links resolve correctly
- [ ] Prune or consolidate verbose `docs/api/` files

### 1.4 Minor CI Improvements

- [ ] Pin all tool versions in CI (trunk 0.21.14, tarpaulin, cargo-audit)
- [ ] Pin Docker base image digest for reproducibility
- [ ] Add multi-arch Docker build (linux/arm64)
- [ ] Add Node.js 24 compatibility (or set FORCE_JAVASCRIPT_ACTIONS_TO_NODE24)

**Completion Criteria:** All 10 workflow pipelines pass on main. Staging deployment verified end-to-end.

---

## Phase 2: SSG Foundation (3-4 weeks)

**Objective:** Bring `tachyon-ssg` to feature parity with Docusaurus/VitePress for documentation use case.

Based on the 78-item gap analysis in `SSG_CAPABILITIES_GAP_ANALYSIS.md`.

### 2.1 Replace Hand-Rolled Parsers (Week 1)

- [ ] Add `serde` derive to frontmatter structs
- [ ] Replace YAML parser with `serde_yaml`
- [ ] Replace TOML parser with `toml` crate
- [ ] Add schema validation for config files

### 2.2 Integrate Renderer Capabilities (Week 1)

- [ ] Wire tree-sitter syntax highlighting into SSG code block rendering
- [ ] Wire KaTeX math rendering into SSG markdown processing
- [ ] Add theme selection to SSG config (github, monokai, dracula)

### 2.3 Page Structure (Week 2)

- [ ] Per-page table of contents
- [ ] Breadcrumbs from document path
- [ ] Prev/next page navigation
- [ ] Sidebar with auto-generated heading navigation
- [ ] Mobile hamburger menu

### 2.4 Search (Week 2)

- [ ] Integrate Pagefind for client-side search
- [ ] Generate search index during SSG build
- [ ] Search UI component in generated pages

### 2.5 Content Features (Week 3)

- [ ] Admonitions/callouts (`> [!NOTE]`, `> [!WARNING]`)
- [ ] Tabs component
- [ ] Code groups (multi-language)
- [ ] Copy button on code blocks
- [ ] Mermaid diagram rendering

### 2.6 SEO and Meta (Week 3-4)

- [ ] JSON-LD structured data
- [ ] Generate `robots.txt`
- [ ] `hreflang` links for i18n
- [ ] Open Graph images
- [ ] Canonical URLs

**Completion Criteria:** Documentation site matches MkDocs Material in core features. Priority 1 and 2 gap items implemented.

---

## Phase 3: Build System and Developer Experience (2-3 weeks)

**Objective:** Fast, deterministic builds. Pleasant developer onboarding.

### 3.1 SSG Build Performance

- [ ] Incremental builds (only rebuild changed pages)
- [ ] Build-time Tailwind CSS generation
- [ ] Asset hashing for cache busting
- [ ] Image optimization pipeline
- [ ] Hot reload / live preview mode

### 3.2 Workspace Hygiene

- [ ] Remove `casbin` `runtime-async-std` feature (use tokio throughout)
- [ ] Unify database backends (PostgreSQL as canonical)
- [ ] Run `cargo udeps` to remove unused dependencies
- [ ] Remove dead `tree-sitter-sql = "0.0"` placeholder
- [ ] Decouple `tachyon-testing` from `tachyon-desktop-app`

### 3.3 Template Engine

- [ ] Replace `format!()` HTML interpolation with minijinja or askama
- [ ] Template inheritance (base layout, page partials)
- [ ] Custom filters (date formatting, slugify, markdown)

### 3.4 Developer Tooling

- [ ] VS Code devcontainer configuration
- [ ] One-command setup: clone, nix-shell, cargo test
- [ ] Pre-commit hook documentation in CONTRIBUTING.md

**Completion Criteria:** `tachyon ssg build` incremental <5s. New contributor builds and tests in <10 minutes.

---

## Phase 4: API and Server Hardening (2-3 weeks)

**Objective:** Production-ready server with rate limiting, observability, and graceful degradation.

### 4.1 Rate Limiting

- [ ] Token bucket rate limiter middleware
- [ ] Per-endpoint configuration
- [ ] Per-user rate limits (authenticated)
- [ ] Rate limit headers (`X-RateLimit-Remaining`)

### 4.2 Observability

- [ ] Structured JSON logging
- [ ] Request tracing (request ID propagation)
- [ ] Metrics endpoint (Prometheus format)
- [ ] Health check with dependency status (PostgreSQL, Redis, Tantivy)

### 4.3 API Improvements

- [ ] OpenAPI 3.1 spec generation (utoipa, already partially integrated)
- [ ] API versioning (`/api/v1/`, `/api/v2/`)
- [ ] Cursor-based pagination (replace offset)
- [ ] Field selection (`?fields=id,title`)
- [ ] Batch operations

### 4.4 WebSocket Reliability

- [ ] Automatic reconnection with exponential backoff
- [ ] Message ordering guarantees
- [ ] Presence heartbeat with TTL cleanup
- [ ] Room-based broadcasting optimization

**Completion Criteria:** Server passes production readiness review. Load test with K6 shows stable performance under 1000 concurrent users.

---

## Phase 5: Real-Time Collaboration v2 (3-4 weeks)

**Objective:** Multi-user editing with conflict resolution and offline support.

### 5.1 CRDT Persistence

- [ ] Persist CRDT state to PostgreSQL
- [ ] CRDT document snapshots
- [ ] Efficient CRDT delta encoding

### 5.2 Collaboration Features

- [ ] Cursor and selection sharing
- [ ] Presence indicators with avatars
- [ ] Document history with diff view
- [ ] Comment threads anchored to text ranges

### 5.3 Offline Support

- [ ] Offline-first CRDT edits
- [ ] Conflict resolution on reconnect
- [ ] Sync queue with priority ordering

**Completion Criteria:** Two users can simultaneously edit the same document with sub-100ms perceived latency. Offline edits sync correctly on reconnect.

---

## Phase 6: Advanced Features (4-6 weeks)

### 6.1 AI Integration

- [ ] Plugin-based AI provider interface (OpenAI, Anthropic, local LLM)
- [ ] Semantic search using embeddings (pgvector or Qdrant)
- [ ] Auto-tagging and document classification
- [ ] AI-assisted writing and summarization

### 6.2 SSG Advanced

- [ ] Plugin system for build pipeline
- [ ] Custom page layouts
- [ ] Versioned documentation (v1/v2 side-by-side)
- [ ] Content collections with typed schemas

### 6.3 Multi-Tenant SaaS

- [ ] Per-tenant database isolation
- [ ] Tenant-specific configuration
- [ ] Usage metering and quota enforcement
- [ ] Admin portal for tenant management

### 6.4 Mobile Applications

- [ ] React Native or Flutter mobile client
- [ ] Offline-first sync with CRDT
- [ ] Push notifications
- [ ] Biometric authentication

**Completion Criteria:** Platform supports AI-assisted knowledge management for single-tenant and multi-tenant deployments.

---

## Phase 7: Production Launch (1-2 weeks)

**Objective:** Public release with production infrastructure.

### 7.1 Infrastructure

- [ ] Production Kubernetes or Docker Swarm deployment
- [ ] TLS certificates (Let's Encrypt or managed)
- [ ] CDN for static assets (CloudFront or Cloudflare)
- [ ] DNS configuration and health monitoring

### 7.2 Security Hardening

- [ ] Penetration testing (OWASP Top 10)
- [ ] CSP headers tuned for production
- [ ] Secret rotation policy
- [ ] Incident response runbooks

### 7.3 Monitoring

- [ ] Grafana dashboards (API latency, error rates, resource usage)
- [ ] Prometheus alerting rules
- [ ] Log aggregation (Loki or ELK)
- [ ] Uptime monitoring (UptimeRobot or equivalent)

### 7.4 Release

- [ ] Tag v1.0.0
- [ ] Generate release notes automatically
- [ ] Publish Docker images to GHCR
- [ ] Publish documentation site
- [ ] Announce on relevant channels

**Completion Criteria:** Publicly accessible production instance with >99.9% uptime SLA.

---

## Effort Summary

| Phase | Duration | Key Deliverable | Dependencies |
|-------|----------|-----------------|--------------|
| 1: CI/CD Hardening | 1-2 weeks | All pipelines green, staging deployed | Infrastructure provisioning |
| 2: SSG Foundation | 3-4 weeks | Feature-parity docs site | Phase 1 |
| 3: Build System & DX | 2-3 weeks | Fast builds, easy onboarding | Phase 1 |
| 4: API & Server | 2-3 weeks | Production-ready API | Phase 1 |
| 5: Real-Time Collab v2 | 3-4 weeks | Multi-user editing | Phase 4 |
| 6: Advanced Features | 4-6 weeks | AI, mobile, SaaS | Phase 4, Phase 5 |
| 7: Production Launch | 1-2 weeks | Public v1.0.0 | Phase 4, Phase 6 |
| **Total** | **16-24 weeks** | **Full production platform** | |

Phases 2 and 3 can run in parallel. Phase 4 can start after Phase 1 completes.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| E2E test flakiness | High | Medium | Split smoke/full, increase timeouts, retry logic |
| CI infrastructure cost | Medium | Low | Cache aggressively, minimize Docker builds |
| WASM toolchain breakage | Medium | Medium | Pin trunk/wasm-bindgen versions |
| Docker base image CVEs | High | Medium | Automated `apt-get upgrade`, Trivy scanning |
| SSG parser rewrite regressions | Medium | High | Byte-for-byte comparison tests before/after |
| Tauri 2.x API instability | Medium | Medium | Pin tauri version, isolate desktop crate |
| PostgreSQL extension conflicts | Medium | High | `IF NOT EXISTS`, advisory locks, serial tests |
| Supply chain attack | Low | Critical | `cargo audit`, SBOM, pinned digests |
| Performance regression | Medium | High | Criterion benchmarks in CI, K6 load tests |
| Documentation drift | High | Low | Automated doc-code consistency checks |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-19 | Delete DEPLOYMENT_SUMMARY.md | Self-documented as severely outdated, SQLite refs wrong |
| 2026-05-19 | Fix VERSION.md to 12.0.0 | Align with ROADMAP.md version |
| 2026-05-19 | Fix README.md commands | `just db-reset` and `just dev` did not exist in justfile |
| 2026-05-19 | Fix architecture overview | SQLite -> PostgreSQL in storage layer diagram |
| 2026-05-19 | Fix docs.yml cache path | `target` -> `tachyon/target` for workspace-correct caching |
| 2026-05-19 | Add 4 documentation pages | search, authentication, cli-reference, collaboration |
| 2026-05-19 | Expand site.toml navigation | 6 -> 10 menu items for complete doc site |
| 2026-05-18 | Disable wasm-opt in trunk build | Auto-downloaded binaryen incompatible with bulk-memory WASM |
| 2026-05-18 | Exclude desktop crates via sed in Docker | Can't build GTK headless in CI |
| 2026-05-18 | Ignore RUSTSEC-2023-0071 | rsa via sqlx-mysql, no fix available, MySQL not used |
| 2026-05-18 | Serial integration tests | PostgreSQL `CREATE EXTENSION` race condition |
| 2026-05-13 | Use `ammonia` for HTML sanitization | OWASP-recommended |
| 2026-02-11 | Nix flake for dev environment | Reproducible builds |

---

## Superseded Documents

This document supersedes:
- `ROADMAP.md` (v11.0.0) -- Phases A-F marked complete
- `ROADMAP_FORWARD.md` -- Items folded into Phases 1-7 above
- `AUDIT_AND_ROADMAP.md` -- Audit findings addressed; remaining items in Phase 1
- `SSG_CAPABILITIES_GAP_ANALYSIS.md` -- Detailed gap list; implementation in Phase 2
- `DEPLOYMENT_SUMMARY.md` -- Deleted (severely outdated, incorrect database references)
