# Tachyon Roadmap

**Version:** 21.0.0 | **Date:** 2026-05-27 | **Status:** Production-Ready, Awaiting Infrastructure

---

## Current State Summary

### Test Suite (1,327 tests, all passing)

| Crate | Tests | Status |
|-------|-------|--------|
| tachyon-server | 486 | PASS |
| tachyon-core | 145 | PASS |
| tachyon-database | 139 | PASS |
| tachyon-editor | 116 | PASS |
| tachyon-renderer | 74 | PASS |
| tachyon-ssg | 66 | PASS |
| tachyon-plugin-runtime | 45 | PASS |
| tachyon-search | 45 | PASS |
| tachyon-import-export | 43 | PASS |
| tachyon-rbac | 37 | PASS |
| tachyon-storage | 31 | PASS |
| **Total** | **1,327** | **ALL PASS** |

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

| Item | Severity | Effort |
|------|----------|--------|
| ~40 `.unwrap()` in production code (mostly Regex::new on literals) | Low | 2h |
| `.unwrap()` on queue operations in `sync_queue.rs` | Medium | 1h |
| `.expect()` in `graphql/schema.rs:33` | Medium | 30m |
| GUI does not match amoebic/brutalist/spatial-materialism aesthetic | Low | 40h |
| Performance claims ("sub-100ms", "sub-15ms") lack reproducible benchmarks | Low | 4h |
| CHANGELOG version ordering and missing release links | Low | 1h |
| Tree-sitter language count inconsistency (README vs CHANGELOG) | Low | Fixed |

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

**No infrastructure dependency. Can start in parallel with Phase 1.**

**Actions:**

1. OpenAPI 3.1 spec validation: verify utoipa annotations, publish Swagger UI
2. Cursor-based pagination: verify all high-volume list endpoints use cursors
3. Redis rate limiting: test distributed rate limiting with real Redis
4. Query optimization: run EXPLAIN ANALYZE on hot paths, verify N+1 elimination
5. WebSocket reliability: presence heartbeat cleanup, reconnection stress test
6. Load testing with k6: target 1,000 concurrent users, P99 < 200ms
7. pgvector validation: verify semantic search embedding pipeline at scale

**Completion Criteria:**
- API load test passes: P99 < 200ms at 1,000 concurrent users
- Rate limiting enforced: HTTP 429 after threshold
- All endpoints documented in OpenAPI spec
- WebSocket survives 100 reconnect cycles without data loss

---

## Phase 3: Security Hardening (Week 3-4)

**Depends on:** Phase 2 (load testing may reveal new vulnerabilities)

**Actions:**

1. External penetration test: OWASP ZAP baseline + manual review
2. CSP tuning: production Content-Security-Policy for WASM + CDN
3. JWT secret rotation: implement rotation without downtime
4. Audit logging: verify all CRUD operations produce audit trail entries
5. Container scanning: enforce Trivy zero CRITICAL/HIGH gate
6. SBOM validation: verify CycloneDX completeness for all 17 crates
7. Supply chain: review cargo-deny bans/advisories, update RUSTSEC overrides

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

**Depends on:** Phase 4 (post-launch)

The current GUI uses Tailwind defaults with rounded corners and soft shadows. The target aesthetic is:

**Amoebic UI:** Organic, non-rectilinear forms. Fluid blob-like containers using `clip-path: path()` and CSS `border-radius` with asymmetric values (e.g., `60% 40% 30% 70% / 60% 30% 70% 40%`). Animated SVG backgrounds with noise textures.

**Brutalism:** Raw structure exposed. Monospace typography throughout (not just code blocks). Sharp corners (`rounded-none`) for primary containers. Stark contrast. No gradients. System fonts or monospace. Visible grid/layout structure.

**Spatial Materialism:** Physical depth metaphor. Multi-layer z-stacking with parallax. Material textures (noise, grain via CSS filters). Shadow depth indicates hierarchy (not just decoration). Transitions feel physical (spring physics, not linear easing).

**Actions:**

1. Design system: define CSS custom properties for the three aesthetic pillars
2. Typography: switch to monospace-first (JetBrains Mono or IBM Plex Mono)
3. Containers: replace `rounded-lg` with asymmetric `border-radius` or `rounded-none`
4. Shadows: replace soft shadows with hard, offset shadows (spatial depth)
5. Backgrounds: add CSS noise/grain texture overlays
6. Animations: implement spring-physics transitions (CSS or JS)
7. Landing page: redesign with amoebic hero section
8. Components: audit all 28 `border-radius` occurrences, apply new system

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
| 2. API Validation | 2 weeks | None (code-only) | READY |
| 3. Security | 2 weeks | Phase 2 | READY |
| 4. Launch | 1 week | Phase 1, 3 | PENDING |
| 5. GUI Redesign | 3 weeks | Phase 4 | PLANNED |
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
