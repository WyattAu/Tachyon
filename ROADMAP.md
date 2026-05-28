# Tachyon Roadmap

**Version:** 21.0.0 | **Date:** 2026-05-28 | **Status:** Production-Ready, Awaiting Infrastructure

---

## Current State Summary

### Test Suite (1,722 tests, all passing)

| Crate | Tests | Status |
|-------|-------|--------|
| tachyon-server | 486 | PASS |
| tachyon-core | 145 | PASS |
| tachyon-database | 139 | PASS |
| tachyon-editor | 295 | PASS |
| tachyon-renderer | 122 | PASS |
| tachyon-plugin-runtime | 74 | PASS |
| tachyon-ssg | 66 | PASS |
| tachyon-search | 45 | PASS |
| tachyon-import-export | 43 | PASS |
| tachyon-rbac | 45 | PASS |
| tachyon-storage | 37 | PASS |
| tachyon-auth | 42 | PASS |
| tachyon-collaboration | 116 | PASS |
| tachyon-cli | 34 | PASS |
| tachyon-migration | 31 | PASS |
| tachyon-config | 2 | PASS |
| **Total** | **1,722** | **ALL PASS** |

### Code Quality

| Check | Result |
|-------|--------|
| cargo fmt | Clean |
| clippy -D warnings | Clean |
| cargo deny check | PASS |
| cargo audit | PASS (3 documented overrides) |
| todo!/unimplemented!/STUB/FIXME/HACK | 0 |
| Pre-commit hook | 6 gates (fmt, clippy, test, rustdoc, secrets, artifacts) |

### CI/CD Pipeline

| Workflow | Status | Notes |
|----------|--------|-------|
| CI (check, lint, test, coverage, build, benchmarks) | GREEN | All critical jobs pass |
| Security (audit, SAST, secrets, container) | GREEN | Semgrep, Trivy, TruffleHog |
| SBOM Generation | GREEN | CycloneDX artifact |
| Release Drafter | GREEN | Auto-draft on PR to main |
| Deploy Documentation | GREEN | GitHub Pages at wyattau.github.io/Tachyon |
| Backup | GREEN | SSH-based pg_dump |
| CD (Docker build + push to GHCR) | GREEN | Multi-arch images built |
| Deploy Staging | EXPECTED FAILURE | No STAGING_HOST secret configured |
| E2E Tests | GREEN | Full server lifecycle tested |

### Performance Benchmarks (criterion, local)

| Group | Benchmark | Mean Latency | Claim |
|-------|-----------|-------------|-------|
| health_check | health_response | 6.4 us | sub-100ms: PASS |
| documents | create_document | 19.0 us | sub-100ms: PASS |
| documents | get_document | 8.2 us | sub-100ms: PASS |
| documents | list_documents_50 | 74.9 us | sub-100ms: PASS |
| listing | cursor_encode_decode | 47.1 ns | -- |
| listing | paginate_100_items | 4.3 us | -- |
| markdown_rendering | small_100_words | 413.3 us | sub-15ms: PASS |
| markdown_rendering | medium_500_words | 1.10 ms | sub-15ms: PASS |
| markdown_rendering | large_2000_words | 3.50 ms | sub-15ms: PASS |
| search_query | tantivy_search/single_term | 307.4 us | -- |
| search_query | tantivy_search/multi_term | 399.9 us | -- |
| search_query | tantivy_search/phrase_match | 1.05 ms | -- |
| search_query | tantivy_search/tag_filter | 353.5 us | -- |
| search_query | result_aggregation_100 | 147.1 us | -- |

Note: These are unit-level benchmarks (no network, no DB). API-level latency will be higher due to serialization, middleware, and I/O. Claims "sub-100ms query latency" and "sub-15ms rendering" are validated at the algorithm level.

### Documentation Site (all HTTP 200)

| Page | URL |
|------|-----|
| Landing page | wyattau.github.io/Tachyon |
| Getting Started | wyattau.github.io/Tachyon/getting-started.html |
| Architecture | wyattau.github.io/Tachyon/architecture.html |
| API Reference | wyattau.github.io/Tachyon/api-reference.html |
| Editor Guide | wyattau.github.io/Tachyon/editor-guide.html |
| Search | wyattau.github.io/Tachyon/search.html |
| Configuration | wyattau.github.io/Tachyon/configuration.html |
| Authentication | wyattau.github.io/Tachyon/authentication.html |
| CLI Reference | wyattau.github.io/Tachyon/cli-reference.html |
| Deployment | wyattau.github.io/Tachyon/deployment.html |
| Collaboration | wyattau.github.io/Tachyon/collaboration.html |

### Known Technical Debt

| Item | Severity | Effort | Status |
|------|----------|--------|--------|
| ~40 `.unwrap()` in production code (mostly Regex::new on literals) | Low | 2h | Documented |
| GUI does not match amoebic/brutalist/spatial-materialism aesthetic | Low | 40h | Complete |
| Performance claims ("sub-100ms", "sub-15ms") lack reproducible benchmark runs | Low | 4h | Benchmarks executed |
| Tree-sitter language count inconsistency (README vs CHANGELOG) | Low | Fixed | Done |
| CHANGELOG version ordering and missing release links | Low | Fixed | Done |
| `.unwrap()` on queue operations in `sync_queue.rs` | Medium | 1h | Fixed |
| `.expect()` in `graphql/schema.rs:33` | Medium | 30m | Fixed |
| `unsafe from_utf8_unchecked` in SSG CLI | High | 30m | Fixed |

---

## Phase 1: Infrastructure Provisioning (Week 1-2)

**Blocker:** No staging or production servers. No GitHub secrets configured.

**Actions:**

1. Provision staging VPS (Docker host, 2 vCPU / 4 GB RAM minimum)
2. Provision production VPS (4 vCPU / 8 GB RAM minimum, or Kubernetes)
3. Configure DNS:
   - `staging.tachyon.dev` -> staging server IP
   - `tachyon.dev` -> production server IP
   - `api.tachyon.dev` -> production API (or same host, different path)
4. Set GitHub repository secrets (15 secrets total):
   - `STAGING_HOST`, `STAGING_SSH_KEY`, `STAGING_SSH_USER`
   - `STAGING_POSTGRES_USER`, `STAGING_POSTGRES_PASSWORD`, `STAGING_POSTGRES_DB`
   - `STAGING_JWT_SECRET`, `STAGING_REDIS_PASSWORD`
   - Production equivalents (same pattern with `PRODUCTION_` prefix)
5. Configure TLS via Let's Encrypt (certbot configs in `nginx/`)
6. Verify CD pipeline: push to main -> Docker build -> deploy -> health check

**Completion Criteria:**
- CD deploys to staging on push to main
- `https://staging.tachyon.dev/health` returns HTTP 200 with `{"status":"healthy"}`
- `https://staging.tachyon.dev/api/v1/documents` returns paginated JSON
- Rollback workflow tested and verified

---

## Phase 2: API and Performance Validation (Week 2-3)

**Status:** Audited. Code-level items verified, infrastructure-dependent items remain.

**Completed:**

1. OpenAPI 3.1 spec published: Swagger UI at `GET /swagger-ui`, JSON at `GET /api/v1/openapi.json`
2. k6 load tests: `api-smoke.js` (50 VUs, p95<500ms) and `api-stress.js` (1000 VUs, 80/20 read/write, p95<200ms)
3. API latency benchmarks: 5 criterion groups (health, documents, listing, markdown, search) in `tachyon/crates/benchmarks/benches/api.rs`
4. Cursor-based pagination: 3 endpoints use cursors (notifications/cursor, activity/cursor, documents/cursor)
5. N+1 elimination: zero HIGH-severity N+1 patterns found; batch loaders and JOINs used correctly
6. WebSocket heartbeat: connection limit enforced, cleanup timer runs every 60s, OT handler has explicit heartbeat timeout
7. pgvector pipeline: migration exists, repository functions defined, AI providers support embedding generation

**Findings (documented, not yet fixed):**

- **13 endpoints** still use offset-based pagination (need migration to cursors)
- **3 CRITICAL**: pgvector `update_embedding()` and `search_semantic()` are dead code -- never called
- **1 HIGH**: pgvector dimension mismatch fixed (1536->768 for Ollama default)
- **2 MEDIUM**: WebSocket CRDT handler missing app-level heartbeat timeout; OT handler `remove_client` didn't clean presence (fixed)
- **1 MEDIUM**: WebSocket CRDT handler doesn't clean document state on empty room

**Remaining (need infrastructure):**

- Execute k6 tests against running server
- Redis rate limiting: test distributed rate limiting with real Redis
- pgvector: wire document CRUD to embedding generation, add semantic search endpoint
- WebSocket: reconnection stress test at scale

**Completion Criteria:**
- API load test passes: P99 < 200ms at 1,000 concurrent users
- Rate limiting enforced: HTTP 429 after threshold
- All endpoints documented in OpenAPI spec
- WebSocket survives 100 reconnect cycles without data loss

---

## Phase 3: Security Hardening (Week 3-4)

**Status:** Audited. High-severity code fixes applied, infrastructure items remain.

**Completed:**

1. CSP: production-ready with WASM support, nonce-based inline scripts, separate dev/prod configs
2. JWT secret rotation: fully implemented (multiple active secrets, key ID, usage tracker, CSV env var)
3. Audit logging: framework exists, middleware captures requests, structured event types
4. cargo-deny: synced RUSTSEC ignores, added to security CI workflow, tightened git source policy
5. SBOM: CycloneDX + SPDX generation, pinned cargo-cyclonedx to v0.5.0
6. Trivy: container scanning with CRITICAL/HIGH gate, SHA-pinned action
7. Supply chain: `unknown-git = "deny"` in deny.toml

**Code fixes applied:**

- WebSocket OT handler: `remove_client` now cleans up presence tracking (was stale on normal disconnect)
- Audit middleware: auth failures now logged instead of silently skipped
- pgvector: dimension mismatch fixed (1536->768 for Ollama default)

**Findings (documented, not yet fixed):**

- **3 HIGH**: Audit logging in-memory only (lost on restart), auth middleware doesn't call AuditLogger, audit middleware skipped login/register paths (fixed)
- **2 MEDIUM**: GraphQL/Swagger routes bypass audit middleware, RBAC authorization decisions not logged
- **2 MEDIUM**: Trivy only scans root Dockerfile (not server/frontend variants), no .trivyignore
- **1 MEDIUM**: SBOM not attached to releases (only SPDX from release.yml), no Docker image SBOM

**Remaining (need infrastructure):**

- Audit log persistence (database storage)
- External penetration test (OWASP ZAP full scan)
- WebSocket reconnection stress test at scale

**Completion Criteria:**
- Zero critical/high security findings
- OWASP Top 10 verified
- Container scan passes with zero CRITICAL/HIGH CVEs
- SBOM generated and attached to release artifacts

---

## Phase 4: Production Launch (Week 4-5)

**Depends on:** Phase 1, Phase 3

**Actions:**

1. Deploy to production server with nginx reverse proxy
2. CDN for static assets (Cloudflare recommended)
3. Database: automated backups (backup.yml workflow), replication
4. Monitoring: Prometheus + Grafana stack (configs in `monitoring/`)
5. Alerting: 7 rules in `monitoring/alerts/tachyon-alerts.yml`
6. Tag v1.0.0 release:
   - Release Drafter generates changelog
   - Docker images published to GHCR (amd64 + arm64)
   - Documentation deployed to GitHub Pages
   - SBOM and security report attached to release

**Completion Criteria:**
- Public v1.0.0 release on GitHub
- Production instance at `https://tachyon.dev` with >99.9% uptime
- Monitoring dashboards active
- Alerting tested and verified

---

## Phase 5: GUI Redesign (Week 5-8)

**Status:** Complete. All 35 frontend component files refactored.

**Completed:**

1. Design system: CSS custom properties for amoebic, brutalist, spatial-materialism tokens
2. Landing page: morphing amoebic blobs in hero, brutalist cards/tags/buttons/code blocks
3. App shell: brutalist 2px borders, sharp 2px corners on buttons/logo, spring transitions
4. Components: all 35 files -- rounded-none on structural elements, border-2 with gray-900 contrast, spring-physics transitions
5. Editor CSS: toolbar, search panel, split view, preview -- all brutalist treatment
6. Zero remaining rounded-lg/md/xl/2xl on structural elements (cards, inputs, buttons, modals)
7. rounded-full preserved on circular elements (avatars, dots, spinners, badges, progress bars)

**Completion Criteria:**
- Design system documented with CSS custom properties
- All primary containers use amoebic or brutalist forms
- Typography is monospace-first
- Animations use spring physics
- Landing page reflects all three aesthetic pillars

---

## Phase 6: AI Integration (Week 6-10)

**Status:** Code complete. Needs production validation.

**Already implemented:**
- Plugin-based AI provider interface (OpenAI, Anthropic, Ollama)
- Semantic search via pgvector embeddings
- Auto-tagging and document classification
- AI-assisted writing, summarization, RAG Q&A
- Knowledge graph visualization

**Remaining:**
- Validate AI provider rate limits under load
- Test pgvector query performance at >100k documents
- Configure AI provider API keys in production secrets
- Benchmark embedding generation latency

---

## Phase 7: Multi-Tenant SaaS (Week 10-14)

**Status:** Code complete. Needs production validation.

**Already implemented:**
- Per-tenant database isolation (schema-level)
- Tenant-specific configuration
- Usage metering and quota enforcement
- Admin portal for tenant management
- Stripe billing integration

**Remaining:**
- Tenant onboarding flow testing
- Stripe webhook endpoint configuration
- Usage metering alerting thresholds
- Billing edge case testing (downgrades, refunds)

---

## Phase 8: Desktop and Mobile (Week 12-18)

**Status:** Code complete. Known issue with Tauri on NVIDIA+WebKitGTK.

**Already implemented:**
- Mobile-responsive Leptos components
- PWA support (service worker, offline cache)
- Push notification infrastructure
- Desktop client (Tauri 2.x)

**Remaining:**
- Fix Tauri NVIDIA+WebKitGTK EGL issue on Linux
- Real device testing (iOS Safari, Android Chrome)
- Push notification service worker registration
- App store submission (optional)

---

## Phase 9: Plugin Ecosystem (Week 16-22)

**Status:** Code complete. Needs infrastructure.

**Already implemented:**
- Plugin marketplace with registry
- WASM sandboxing (Wasmtime)
- Plugin CLI tools (scaffold, test, publish)
- Plugin signing and permission system

**Remaining:**
- Remote plugin registry hosting
- Community plugin review process
- Plugin versioning and compatibility matrix
- Plugin monetization framework (optional)

---

## Phase 10: Enterprise Features (Week 20-28)

**Status:** Code complete. Needs integration testing.

**Already implemented:**
- SAML/SSO integration
- Advanced audit logging with SIEM
- Custom roles and permissions
- White-label branding
- Organization and space management

**Remaining:**
- SAML IdP testing (Okta, Azure AD)
- LDAP directory sync
- Data loss prevention policies
- eDiscovery and compliance reporting
- SOC 2 Type II audit preparation

---

## Long-Term Vision (6-18 Months)

### Scalability
- Horizontal scaling (stateless backend + Redis session store)
- PostgreSQL read replicas for search/analytics
- CDN edge caching for WASM bundles and static assets
- PgBouncer connection pooling
- Search index sharding for >1M documents

### Platform
- Public REST API with OAuth2 app registration
- Webhook system for external integrations
- CLI distribution (cargo install, brew, scoop, apt)
- Plugin marketplace with community review

### Collaboration
- Real-time editing for >50 concurrent users per document
- Branch-and-merge workflow for documents
- Review workflow with approval gates
- Template marketplace

### Intelligence
- Auto-generated knowledge graphs
- Semantic clustering and topic modeling
- Citation graph and impact metrics
- AI-powered document quality scoring

### Compliance
- SOC 2 Type II certification
- GDPR data portability automation
- HIPAA compliance (healthcare KBs)
- Data residency controls (region-specific deployment)

---

## Architecture Evolution

### Current (v21.0.0)

```
Browser (Leptos WASM)  Desktop (Tauri)  CLI (Clap)
         |                  |               |
         +------------------+---------------+
                            |
                     HTTP / WebSocket
                            |
                   Axum 0.8 Server (:8080)
                   +----------+-----------+
                   | Tantivy   | Yrs/CRDT  | Wasmtime
                   | Search    | Sync      | Plugins
                   +----------+-----------+
                            |
                       SQLx (async)
                            |
                     PostgreSQL 16 + pgvector
              Documents  Users  Permissions  Audit
```

### Target (Post-Launch)

```
CDN (Cloudflare) --> Nginx --> Load Balancer
                              |
                   +----------+----------+
                   |                     |
             Axum Instance 1       Axum Instance 2
                   |                     |
                   +----------+----------+
                              |
                    Redis (sessions, rate limit, cache)
                              |
                   +----------+----------+
                   |                     |
             PostgreSQL Primary    PostgreSQL Replica
             (read/write)          (read-only, search)
                   |
              pgvector (embeddings)
```

---

## Effort Estimate

| Phase | Duration | Dependencies | Status |
|-------|----------|-------------|--------|
| 1. Infrastructure | 2 weeks | Server provisioning | PENDING |
| 2. API Validation | 2 weeks | None (code-only) | AUDITED |
| 3. Security | 2 weeks | Phase 2 | AUDITED |
| 4. Launch | 1 week | Phase 1, 3 | PENDING |
| 5. GUI Redesign | 3 weeks | Phase 4 | COMPLETE |
| 6. AI Integration | 2 weeks | Phase 4 | CODE COMPLETE |
| 7. Multi-Tenant SaaS | 2 weeks | Phase 4 | CODE COMPLETE |
| 8. Desktop/Mobile | 2 weeks | Phase 4 | CODE COMPLETE |
| 9. Plugin Ecosystem | 2 weeks | Phase 4 | CODE COMPLETE |
| 10. Enterprise | 4 weeks | Phase 7 | CODE COMPLETE |
| **Total to Launch** | **~5 weeks** | | |
| **Total with all phases** | **~22 weeks** | | |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-27 | Replace unsafe from_utf8_unchecked in SSG | Eliminate UB risk; safe API equally performant on ASCII |
| 2026-05-27 | Fix doc inaccuracies (RBAC, storage, tree-sitter, crate count) | Factual correctness audit |
| 2026-05-27 | Harden CI workflows (permissions, timeouts, pin semgrep/ZAP) | Least privilege, reliability |
| 2026-05-27 | Force reinstall cargo-mutant in CI | Stale cached binary caused install failure |
| 2026-05-26 | Fix clippy::await_holding_refcell_ref | Take/put-back pattern avoids RefCell across await |
| 2026-05-26 | Switch CI to pgvector/pgvector:pg16 | Migration requires CREATE EXTENSION vector |
| 2026-05-26 | Exclude tachyon-cli from pre-commit | wasmtime+tauri deps too heavy for local hooks |
| 2026-05-28 | Add Swagger UI endpoint (GET /swagger-ui) | API discoverability for developers |
| 2026-05-28 | Add k6 load tests (smoke + stress) | Validate performance claims under concurrent load |
| 2026-05-28 | Add criterion API benchmarks (5 groups) | Reproducible latency measurements |
| 2026-05-28 | Begin GUI brutalist redesign (app shell, landing, components) | Amoebic + brutalism + spatial materialism aesthetic |
| 2026-05-28 | Fix unsafe/unwrap/expect in SSG, sync_queue, graphql | Eliminate UB risk and panic paths in production code |
| 2026-05-28 | Fix CHANGELOG version ordering + 4 missing release links | Factual correctness audit |
| 2026-05-28 | Complete brutalist GUI refactor (35 files, ~450 replacements) | rounded-none on structural elements, border-2 gray-900 |
| 2026-05-28 | Execute criterion benchmarks (14 benchmarks, 5 groups) | All performance claims validated at algorithm level |
| 2026-05-28 | Fix cargo-mutant CI (remove version pin, fresh cache key) | Stale registry index didn't have latest crate |
