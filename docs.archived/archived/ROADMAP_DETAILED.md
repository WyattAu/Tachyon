# Tachyon Production Roadmap

**Version:** 12.0.0 | **Date:** 2026-05-20 | **Status:** CI green, not production-deployed

This document supersedes `ROADMAP.md` and the previous `ROADMAP_DETAILED.md`.

---

## 1. Current State

### 1.1 Codebase Summary

| Metric | Value |
|--------|-------|
| Crates | 16 + 1 benchmarks |
| Total Rust LOC | ~142,000 |
| Test modules (`#[cfg(test)]`) | 201 |
| Test functions (`#[test]`) | 1,657 |
| `todo!` / `unimplemented!` | 0 |
| FIXME / HACK / STUB markers | 0 |
| Clippy (`-D warnings`) | Clean |
| Rustdoc warnings | 0 |
| Pre-commit gates | 6 (fmt, clippy, test, doc, secrets, artifacts) |

### 1.2 Deployment Targets

| Target | Technology | Status |
|--------|-----------|--------|
| Server | Axum 0.8, PostgreSQL 16+ | Functional, not deployed |
| CLI | `tachyon-cli` | Functional (init, serve, build, gui) |
| Desktop | Tauri 2 (Linux/macOS/Windows) | Compiles; NVIDIA+WebKitGTK EGL issue on Linux |
| SSG | `tachyon-ssg` | Functional; multi-language, color themes, RSS/sitemap |
| WASM Frontend | Leptos 0.8 + Trunk | Functional; pages for documents, search, teams, spaces, onboarding |

### 1.3 Server API Surface

24 route modules registered under `/api/v1/`:

activity, billing, catalog, collaboration, conflict, document, ecosystem, files, health, metrics, mfa, node, notification, oauth2, onboarding, organization, password_reset, plugin, repository, review, role, search, seo, session, space, ssg, tags, team, user, v2, webhook

GraphQL endpoint at `/graphql` with async-graphql (QueryRoot + MutationRoot). No subscriptions yet.

### 1.4 Database Layer

35 source files in `tachyon-database/src/`. Repositories: document, user, space, team, role, comment, billing (subscriptions, invoices), presence, webhook, plugin, catalog, activity, notification, graph, search, session, organization, onboarding, password_reset, connected_account, user_preferences, template, schema, saved_search, document_version, document_review, query_logger, rollback, attachment, refresh_token.

All backed by PostgreSQL via sqlx. In-memory storage crate (`tachyon-storage`) provides SQLite and HashMap backends for testing.

### 1.5 Pipeline Health

| Pipeline | Status | Notes |
|----------|--------|-------|
| CI | GREEN | 8/8 jobs passing |
| CD | BLOCKED | Needs STAGING_HOST secret, no staging server |
| Security | GREEN | Dependency audit, SAST, secrets scan |
| E2E | FLAKY | Browser timeout in CI (infrastructure, not code) |
| Docs | GREEN | GitHub Pages deployed |
| SBOM | GREEN | CycloneDX generation |
| Release Drafter | GREEN | Auto-draft on PR |
| Backup | BLOCKED | Needs DATABASE_HOST, no production DB |

### 1.6 What Is Actually Implemented vs. Stubbed

**Fully implemented (PostgreSQL-backed, tested):**
- Document CRUD, search, versioning
- User registration, authentication (JWT), session management
- RBAC engine (roles, permissions, teams)
- Spaces, tags, activity feed, notifications
- Full-text search (Tantivy + PostgreSQL tsvector)
- Markdown rendering (CommonMark + GFM, tree-sitter highlighting, KaTeX math)
- CRDT editor engine (Yrs-based)
- WebSocket collaboration (presence, cursors, broadcasts)
- Plugin runtime (Wasmtime sandbox, marketplace registry, AI capability types)
- SSG (multi-language, color themes, RSS, sitemap, SEO meta)
- Import/export (Docusaurus, Obsidian, Markdown, JSON, HTML)
- Billing data model (subscriptions, invoices, notification preferences)
- OAuth2 route scaffold, MFA route scaffold, password reset

**Stubbed / incomplete:**
- GraphQL: types defined, resolvers wired to DB for documents/spaces/users/search, but `me()` returns `None`, `update_profile` always errors, no subscriptions
- AI plugin: data types only (`ai.rs`); no actual model invocation, no embedding pipeline, no RAG
- Marketplace: local registry only (`marketplace.rs`); no remote registry client, no actual WASM download/install flow
- Sandbox: `sandbox.rs` exists but no filesystem/network isolation enforcement
- Billing: data model exists, no payment processor integration
- OAuth2/MFA: route handlers exist, not wired to external providers
- SSG: no client-side search (Pagefind), no versioned docs, no admonitions, no Mermaid, no template engine (uses `format!` for HTML)

---

## 2. Implementation Status by Crate

| Crate | LOC | Status | Gaps |
|-------|-----|--------|------|
| `tachyon-core` | ~3,000 | Complete | -- |
| `tachyon-server` | ~18,000 | Nearly complete | GraphQL subscriptions, rate limit headers per-user, OpenAPI completeness |
| `tachyon-database` | ~12,000 | Complete | -- |
| `tachyon-renderer` | ~5,000 | Complete | -- |
| `tachyon-search` | ~3,500 | Complete | Semantic search (embeddings) |
| `tachyon-rbac` | ~2,500 | Complete | -- |
| `tachyon-frontend` | ~8,000 | Functional | Mobile responsive, PWA, offline sync |
| `tachyon-desktop` | ~3,000 | Compiles | NVIDIA EGL workaround, no auto-update |
| `tachyon-cli` | ~5,000 | Complete | -- |
| `tachyon-storage` | ~2,000 | Complete | -- |
| `tachyon-editor` | ~3,500 | Complete | -- |
| `tachyon-import-export` | ~2,000 | Complete | -- |
| `tachyon-ssg` | ~2,500 | Partial | No Pagefind, no template engine, no admonitions |
| `tachyon-plugin-runtime` | ~2,000 | Partial | Types only for AI; no real model calls |
| `tachyon-testing` | ~1,500 | Complete | -- |
| `tachyon-benchmarks` | ~300 | Minimal | Only rbac, renderer, search, API benchmarks |

---

## 3. Known Technical Debt

1. **Broken `.adrs/` links**: ~2,338 references across 134 files. `scripts/fix_doc_links.sh` exists but not run.
2. **Verbose API docs**: `docs/api/` contains ~60K lines across 15 files, mostly scaffolded. Needs consolidation.
3. **E2E test flakiness**: Browser timeout in CI. WASM or server may not start within 60s window.
4. **Node.js 20 deprecation**: GitHub Actions on Node.js 20, deprecated Sept 2026.
5. **CD pipeline**: Requires STAGING_HOST/PRODUCTION_HOST secrets and actual servers.
6. **Backup pipeline**: Requires DATABASE_HOST and S3 credentials.
7. **Action pinning**: 8+ GitHub Actions referenced by tag, not SHA.
8. **CD Docker multi-arch timeout**: arm64 cross-compilation exceeds 30-minute limit.
9. **Test count disagreement**: VERSION.md, ROADMAP.md, and CHANGELOG disagree on total count (actual measured: ~1,657 test functions, 1,116 tests reported in VERSION.md from `cargo test` output).
10. **Workspace hygiene**: `casbin` `runtime-async-std` feature unused, dead `tree-sitter-sql = "0.0"` placeholder, `tachyon-testing` depends on `tachyon-desktop-app`.

---

## 4. Roadmap Phases

### Phase 1: Infrastructure Hardening (Weeks 1-2)

**Objective:** All CI/CD pipelines pass. Zero broken documentation links. Staging deployable.

#### 1.1 E2E Test Stabilization

- Increase server startup timeout from 60s to 120s
- Add WASM frontend health check before Playwright tests
- Split E2E into smoke (fast) and full (slow) suites
- Add retry logic with screenshot-on-failure
- Pin trunk/wasm-bindgen versions

#### 1.2 CD Pipeline Activation

- Provision staging (GitHub Codespaces ephemeral or lightweight VPS)
- Configure STAGING_HOST, STAGING_SSH_KEY secrets
- Test end-to-end: push main -> Docker build -> deploy -> health check
- Increase Docker arm64 build timeout to 60 min (or separate arm64 job)
- Fix `deploy-staging.yml` image mismatch (monolithic vs. separate server/frontend)
- Redesign `backup.yml` for current infrastructure

#### 1.3 Documentation Link Cleanup

- Run `scripts/fix_doc_links.sh` across all 134 affected files
- Validate with link checker in CI
- Prune/consolidate `docs/api/` files (~60K lines)

#### 1.4 CI Hardening

- Pin all GitHub Actions to SHA (supply chain)
- Pin Docker base image digest
- Add multi-arch Docker build (linux/arm64)
- Set `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`
- Add `cargo-deny` config

**Deliverables:** All 10 workflow pipelines pass on main. Zero broken internal doc links. Staging deployment verified.

---

### Phase 2: SSG Foundation (Weeks 2-5, overlaps Phase 1)

**Objective:** SSG parity with MkDocs Material for documentation use case.

Based on 78-item gap analysis in `SSG_CAPABILITIES_GAP_ANALYSIS.md`.

#### 2.1 Template Engine (Week 2)

- Replace `format!()` HTML interpolation with minijinja or askama
- Template inheritance (base layout, page partials)
- Custom filters (date formatting, slugify, markdown rendering)

#### 2.2 Client-Side Search (Week 3)

- Integrate Pagefind for client-side search
- Generate search index during SSG build
- Search UI component in generated pages

#### 2.3 Navigation (Week 3)

- Collapsible sidebar from `site.toml` menu structure
- In-page table of contents from heading hierarchy
- Breadcrumb generation from slug path
- Prev/next navigation
- Mobile hamburger menu

#### 2.4 Content Features (Week 4)

- Admonitions/callouts (`> [!NOTE]`, `> [!WARNING]`, `> [!TIP]`, `> [!DANGER]`)
- Code groups (multi-language tabs)
- Copy button on code blocks
- Mermaid diagram rendering

#### 2.5 SEO and Meta (Week 4-5)

- JSON-LD structured data
- `robots.txt` generation
- Open Graph images
- Canonical URLs

#### 2.6 Build Performance (Week 5)

- Incremental builds (only rebuild changed pages)
- Asset hashing for cache busting
- Image optimization pipeline

**Deliverables:** Documentation site matches MkDocs Material in core features. Priority 1 and 2 gap items implemented.

---

### Phase 3: Build System and Developer Experience (Weeks 2-4, overlaps Phase 1)

**Objective:** Fast deterministic builds. New contributor operational in <10 minutes.

#### 3.1 Workspace Hygiene

- Remove `casbin` `runtime-async-std` feature
- Remove dead `tree-sitter-sql = "0.0"` placeholder
- Run `cargo udeps` to remove unused dependencies
- Decouple `tachyon-testing` from `tachyon-desktop-app`
- Remove orphan files (root `crates/` if present, `tachyon/data/tachyon.db`)

#### 3.2 Nix Flake

- Add `tachyon-benchmarks`, `playwright`, `cargo-deny` to dev shell
- Validate with `nix flake check`

#### 3.3 Devcontainer

- Update `.devcontainer/` to match current flake
- Add VS Code extensions for Rust, Leptos, TOML

#### 3.4 Build Commands

- Merge root `Makefile` and `tachyon/justfile`
- Add `make docs-preview` for local SSG preview
- Document all commands in CONTRIBUTING.md

**Deliverables:** `nix flake check` passes. Single source of truth for build commands. Clean workspace.

---

### Phase 4: Security Hardening (Weeks 4-6)

**Objective:** Production-grade security posture.

#### 4.1 Authentication Hardening

- Wire OAuth2 handlers to actual providers (Google, GitHub minimum)
- Wire MFA handlers to TOTP implementation
- Implement refresh token rotation
- Add account lockout after failed attempts
- Rate limit login endpoints separately

#### 4.2 API Security

- Complete OpenAPI 3.1 spec (verify all route modules have utoipa annotations)
- Validate with `spectral`
- API versioning (`/api/v2/`) with deprecation headers
- Cursor-based pagination (replace offset)
- Input size limits on all endpoints
- CSP nonce validation

#### 4.3 Observability

- Structured JSON logging throughout server
- Request tracing with request ID propagation
- Prometheus metrics endpoint (expand beyond placeholder)
- Health check with dependency status (PostgreSQL, Redis, Tantivy)

#### 4.4 Penetration Testing

- Re-run OWASP Top 10 suite against staging
- Address large payload warning
- Validate all previous pen-test results still pass

**Deliverables:** OpenAPI spec covers 100% of endpoints. OAuth2 functional. Pen-test clean on staging.

---

### Phase 5: Server Hardening and Performance (Weeks 5-7)

**Objective:** Server stable under production load.

#### 5.1 Rate Limiting

- Per-user rate limits (authenticated)
- Per-endpoint configuration
- Rate limit headers (`X-RateLimit-Remaining`, `Retry-After`)

#### 5.2 WebSocket Reliability

- Automatic reconnection with exponential backoff (client-side)
- Message ordering guarantees
- Room-based broadcast optimization
- Presence TTL cleanup

#### 5.3 Performance Validation

- Expand benchmark suite (criterion) to cover all API endpoints
- Load test with K6: 1000 concurrent users target
- P99 latency SLOs: <100ms for read, <200ms for write
- Memory leak detection under sustained load

#### 5.4 GraphQL Maturity

- Wire `me()` resolver to JWT context
- Implement `update_profile` with auth
- Add subscription support (real-time updates via WebSocket)
- Expand mutation coverage (teams, spaces, tags)

**Deliverables:** K6 load test stable at 1000 concurrent users. P99 latency within SLOs.

---

### Phase 6: Real-Time Collaboration v2 (Weeks 7-10)

**Objective:** Multi-user editing with offline support.

#### 6.1 CRDT Persistence

- Persist CRDT state to PostgreSQL
- Document snapshots with delta encoding
- Lazy document loading (only sync visible viewport)
- Benchmark Yrs at 100K+ character documents

#### 6.2 Collaboration Features

- Cursor and selection sharing between users
- Typing indicators
- Active users list per document
- Comment threads anchored to text ranges

#### 6.3 Offline Support

- Offline-first CRDT edits in WASM frontend (IndexedDB)
- Sync queue with priority ordering
- Conflict resolution UI on reconnect
- Background sync indicator

#### 6.4 OT Deprecation

- Migrate fully from OT to CRDT (Yrs)
- Remove `operational_transform.rs` if unused
- Update all tests to CRDT-only paths

**Deliverables:** Two users editing same document with sub-100ms perceived latency. Offline edits sync on reconnect with zero data loss.

---

### Phase 7: Advanced Features (Weeks 8-14)

### 7.1 AI Integration (Weeks 8-10)

- Plugin-based AI provider interface (OpenAI, Anthropic, local LLM)
- Embedding generation for semantic search (pgvector or Qdrant)
- RAG pipeline: embed documents -> vector store -> retrieve -> generate
- Auto-tagging and document classification
- AI-assisted writing and summarization in editor

### 7.2 SSG Advanced (Weeks 9-10)

- Versioned documentation (v1/v2 side-by-side)
- Plugin system for build pipeline
- Content collections with typed schemas

### 7.3 Multi-Tenant SaaS (Weeks 10-12)

- Per-tenant database isolation
- Tenant-specific configuration
- Usage metering and quota enforcement
- Billing integration (Stripe webhook processing)
- Admin portal for tenant management

### 7.4 Advanced Search (Weeks 11-12)

- Semantic search with embeddings
- Faceted search (date ranges, authors, tags, content type)
- Saved searches and search alerts

### 7.5 Knowledge Graph UI (Weeks 12-14)

- Interactive force-directed graph visualization
- Node clustering and community detection
- Graph-based navigation (click node to open document)
- Relationship editing UI

**Deliverables:** AI plugin functional with local model. Multi-tenant billing operational. Semantic search operational.

---

### Phase 8: Production Launch (Weeks 14-16)

#### 8.1 Infrastructure

- Production deployment (Docker Swarm or Kubernetes)
- TLS certificates (Let's Encrypt)
- CDN for static assets (CloudFront or Cloudflare)
- DNS and health monitoring (UptimeRobot)

#### 8.2 Security Final Pass

- `cargo audit` with zero critical advisories
- Penetration test against staging
- Secret rotation policy documented
- Incident response runbooks

#### 8.3 Monitoring

- Grafana dashboards (API latency, error rates, resource usage)
- Prometheus alerting rules
- Log aggregation (Loki or ELK)

#### 8.4 Documentation Finalization

- Fix all broken `.adrs/` references (from Phase 1, verify no regressions)
- Validate all performance claims with benchmarks
- User guide with screenshots
- Architecture decision records for major choices

#### 8.5 Release

- Tag v13.0.0 (production release)
- Multi-arch Docker images on GHCR
- GitHub Release with SBOM
- Documentation site deployed

**Deliverables:** Publicly accessible production instance. Monitoring and alerting operational. >99.9% uptime SLA.

---

## 5. Post-Production Vision (Weeks 17+)

### 5.1 Mobile Applications

- Responsive Leptos frontend for tablet/mobile (touch-optimized controls)
- PWA support (service worker, offline shell, push notifications)
- Native mobile client (React Native or Flutter) if PWA insufficient
- Biometric authentication
- Offline-first sync with CRDT

### 5.2 Plugin Ecosystem

- Remote plugin registry (HTTP API)
- Plugin sandbox with filesystem/network isolation enforcement
- WASM plugin SDK documentation
- Community plugin repository
- Plugin marketplace UI in server admin

### 5.3 Enterprise Features

- SSO/SAML integration
- Audit log export (SIEM integration)
- Data retention policies
- Compliance certifications (SOC 2, GDPR DPIA)
- Dedicated support tier

### 5.4 Performance at Scale

- Horizontal scaling (stateless server behind load balancer)
- Read replicas for search queries
- Connection pooling optimization
- WASM frontend bundle size optimization

---

## 6. Effort Summary

| Phase | Duration | Key Deliverable | Dependencies |
|-------|----------|-----------------|--------------|
| 1: Infrastructure Hardening | 2 weeks | All pipelines green, staging deployed | Infrastructure provisioning |
| 2: SSG Foundation | 4 weeks | Feature-parity docs site | Phase 1 |
| 3: Build System & DX | 3 weeks | Fast builds, easy onboarding | Phase 1 |
| 4: Security Hardening | 3 weeks | OAuth2, OpenAPI, pen-test clean | Phase 1 |
| 5: Server Hardening | 3 weeks | Load-tested, production-ready API | Phase 4 |
| 6: Collaboration v2 | 4 weeks | Multi-user editing with offline | Phase 5 |
| 7: Advanced Features | 7 weeks | AI, SaaS, semantic search | Phase 5, Phase 6 |
| 8: Production Launch | 3 weeks | Public release, monitoring | All above |
| **Total** | **~29 weeks** | **Full production platform** | |

Parallelization: Phases 2, 3, and 4 can overlap (weeks 2-6). Phase 5 starts after Phase 4. Phase 6 starts after Phase 5. Phase 7 can begin once Phase 5 is complete. Phase 8 is the final gate.

Realistic timeline with parallel execution: **~20 weeks** (5 months) to production.

---

## 7. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| E2E test flakiness | High | Medium | Split smoke/full, increase timeouts, retry logic |
| arm64 Docker build timeout | High | Medium | Separate arm64 job, increase timeout, or use cross-rs |
| SSG feature creep | High | Medium | Strict scope per gap analysis item; template engine first |
| CRDT performance at 100K+ chars | Medium | High | Benchmark early, implement lazy viewport loading |
| Breaking API changes | Medium | High | API versioning from Phase 4; deprecation headers |
| Dependency vulnerability (supply chain) | Low | Critical | `cargo audit`, SBOM, SHA-pinned actions, `cargo-deny` |
| OAuth2 provider API changes | Low | Medium | Isolate provider adapters behind trait |
| Documentation link regressions | Medium | Low | CI link checker as part of pre-commit |
| Tauri 2.x API instability | Medium | Medium | Pin tauri version, isolate desktop crate |
| PostgreSQL extension conflicts in CI | Medium | High | `IF NOT EXISTS`, advisory locks, serial integration tests |
| Performance regression | Medium | High | Criterion benchmarks in CI, K6 load tests, P99 SLOs |
| AI provider API rate limits/costs | Medium | Medium | Local model fallback, token budgeting, caching |

---

## 8. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-20 | Consolidate ROADMAP.md + ROADMAP_DETAILED.md into single file | Eliminate drift between two documents |
| 2026-05-20 | Reclassify project state from "PRODUCTION READY" to "CI green, not production-deployed" | No staging/production infrastructure exists; CD/backup pipelines blocked |
| 2026-05-19 | Delete DEPLOYMENT_SUMMARY.md | Severely outdated, incorrect database references |
| 2026-05-19 | Fix VERSION.md to 12.0.0 | Align with ROADMAP.md version |
| 2026-05-19 | Fix README.md commands | `just db-reset` and `just dev` did not exist in justfile |
| 2026-05-19 | Fix architecture overview | SQLite -> PostgreSQL in storage layer diagram |
| 2026-05-19 | Fix docs.yml cache path | `target` -> `tachyon/target` for workspace-correct caching |
| 2026-05-19 | Add 4 documentation pages | search, authentication, cli-reference, collaboration |
| 2026-05-18 | Disable wasm-opt in trunk build | Auto-downloaded binaryen incompatible with bulk-memory WASM |
| 2026-05-18 | Exclude desktop crates via sed in Docker | Cannot build GTK headless in CI |
| 2026-05-18 | Ignore RUSTSEC-2023-0071 | rsa via sqlx-mysql, no fix available, MySQL not used |
| 2026-05-18 | Serial integration tests | PostgreSQL `CREATE EXTENSION` race condition |
| 2026-05-13 | Use `ammonia` for HTML sanitization | OWASP-recommended |
| 2026-04-18 | Go Deep: replace all in-memory stubs with PostgreSQL | Comments, billing, presence, plugin invoke, collaboration broadcast |
| 2026-02-11 | Nix flake for dev environment | Reproducible builds |
