# Tachyon Roadmap

**Version:** 20.0.0 | **Date:** 2026-05-31 | **Status:** Production-Ready, Awaiting Infrastructure

This is the single authoritative roadmap. All prior roadmap variants have been consolidated into this document. Stale versions are archived in `docs.archived/`.

---

## Current State Summary

### Test Suite (2,054 tests, all passing)

| Crate | Tests | Status |
|-------|-------|--------|
| tachyon-server | 1,211 | PASS |
| tachyon-core | 145 | PASS |
| tachyon-database | 139 | PASS |
| tachyon-renderer | 122 | PASS |
| tachyon-editor | 116 | PASS |
| tachyon-import-export | 50 | PASS |
| tachyon-frontend | 89 | PASS |
| tachyon-plugin-runtime | 74 | PASS |
| tachyon-ssg | 66 | PASS |
| tachyon-search | 45 | PASS |
| tachyon-rbac | 37 | PASS |
| tachyon-storage | 31 | PASS |
| tachyon-desktop | 44 | PASS |
| tachyon-cli | 35 | PASS |
| **Total** | **2,054** | **ALL PASS** |

### Code Quality

| Check | Result |
|-------|--------|
| cargo fmt | Clean |
| clippy -D warnings | Clean (all crates including tachyon-cli) |
| cargo deny check | PASS |
| cargo audit | PASS (3 documented RUSTSEC overrides) |
| todo!/unimplemented!/STUB/FIXME/HACK | 0 |
| Pre-commit hook | 6 gates (fmt, clippy, test, rustdoc, secrets, artifacts) |
| Mutation testing | cargo-mutants@27.0.0 on core/database/search (CI, main only) |

### CI/CD Pipeline (11 workflows)

| Workflow | Status | Notes |
|----------|--------|-------|
| CI (check, lint, test, coverage, build, benchmarks) | GREEN | Note: tachyon-cli excluded from CI (depends on tachyon-desktop/GTK); verified locally via pre-commit |
| Security (audit, SAST, secrets, container) | GREEN | Semgrep, Trivy, TruffleHog |
| SBOM Generation | GREEN | CycloneDX via cargo-cyclonedx |
| Release Drafter | GREEN | Auto-draft on PR to main |
| Deploy Documentation | GREEN | GitHub Pages at wyattau.github.io/Tachyon |
| Backup | GREEN | SSH-based pg_dump every 6h |
| CD (Docker build + push to GHCR) | GREEN | Multi-arch (amd64+arm64) |
| Deploy Staging | GREEN | Skips deploy/health-check when STAGING_HOST not configured |
| E2E Tests | GREEN | Playwright smoke + full suite |
| OWASP ZAP Scan | GREEN | API baseline scan, HIGH/CRITICAL gate |
| Release | MANUAL | Multi-arch Docker + GitHub Release + SBOM |

### Performance Benchmarks (criterion, unit-level)

| Group | Benchmark | Mean Latency |
|-------|-----------|-------------|
| health_check | health_response | 6.4 us |
| documents | create_document | 19.0 us |
| documents | get_document | 8.2 us |
| documents | list_documents_50 | 74.9 us |
| markdown_rendering | small_100_words | 413.3 us |
| markdown_rendering | medium_500_words | 1.10 ms |
| markdown_rendering | large_2000_words | 3.50 ms |
| search_query | tantivy_search/single_term | 307.4 us |
| search_query | tantivy_search/multi_term | 399.9 us |
| search_query | tantivy_search/phrase_match | 1.05 ms |

Unit-level (no network/DB). API-level latency will be higher due to serialization, middleware, and I/O.

### Documentation Site (17 pages, all HTTP 200)

Landing page at `wyattau.github.io/Tachyon` with custom Rust SSG, Pagefind search, dark/light theme, KaTeX math rendering, Mermaid diagrams.

---

## Path to Production

### Phase 1: Infrastructure Provisioning (Week 1-2) [BLOCKER]

**Blocker:** No staging or production servers provisioned. No GitHub secrets configured.

| # | Action | Dependency |
|---|--------|-----------|
| 1 | Provision staging VPS (2 vCPU / 4 GB RAM, Docker host) | Cloud provider account |
| 2 | Provision production VPS (4 vCPU / 8 GB RAM) | Cloud provider account |
| 3 | Configure DNS (staging.tachyon.dev, tachyon.dev, api.tachyon.dev) | VPS IPs |
| 4 | Set 15 GitHub secrets (STAGING_*, PRODUCTION_*) | VPS access |
| 5 | Configure TLS via Let's Encrypt (certbot configs in `nginx/`) | DNS propagation |
| 6 | Verify CD pipeline end-to-end | Secrets set |

**Completion:** CD deploys to staging on push to main. Health check returns 200. Rollback workflow tested.

### Phase 2: API and Performance Validation (Week 2-3) [AUDITED]

Code-level audit complete. Infrastructure-dependent items remain.

| Status | Item |
|--------|------|
| DONE | OpenAPI 3.1 spec (Swagger UI at `/swagger-ui`) |
| DONE | k6 load tests (smoke: 50 VUs; stress: 1000 VUs) |
| DONE | Criterion benchmarks (14 benchmarks, 5 groups) |
| DONE | Cursor-based pagination (10 endpoints: 3 original + 7 new) |
| DONE | N+1 analysis (zero HIGH findings) |
| DONE | WebSocket heartbeat and connection limits |
| DONE | pgvector migration and dimension fix |
| DONE | Wire pgvector to document CRUD, add semantic search endpoint |
| REMAINING | Execute k6 against running server (scripts prepared) |
| REMAINING | Redis distributed rate limiting validation (script prepared) |
| REMAINING | WebSocket reconnection stress test at scale (script prepared) |

**Completion:** P99 < 200ms at 1,000 concurrent users. Rate limiting enforced. WebSocket survives 100 reconnect cycles.

### Phase 3: Security Hardening (Week 3-4) [AUDITED]

| Status | Item |
|--------|------|
| DONE | CSP (nonce-based, dev/prod split) |
| DONE | JWT secret rotation (multi-key, KID, usage tracker) |
| DONE | Audit logging framework |
| DONE | cargo-deny + cargo-audit in CI |
| DONE | SBOM (CycloneDX + SPDX) |
| DONE | Trivy container scanning (CRITICAL/HIGH gate, all Dockerfile variants) |
| DONE | Supply chain policy (`unknown-git = "deny"`) |
| DONE | GraphQL/Swagger audit middleware coverage |
| DONE | 19 hot-path regex compilations converted to LazyLock |
| DONE | Audit log persistence (database fire-and-forget write) |
| DONE | External penetration test (OWASP ZAP baseline in CI) |

**Completion:** Zero CRITICAL/HIGH security findings. OWASP Top 10 verified. SBOM in release artifacts.

### Phase 4: Production Launch (Week 4-5) [PENDING]

Depends on: Phase 1, Phase 3.

| # | Action |
|---|--------|
| 1 | Deploy to production with nginx reverse proxy |
| 2 | CDN for static assets (Cloudflare) |
| 3 | Automated database backups + replication |
| 4 | Prometheus + Grafana monitoring (configs in `monitoring/`) |
| 5 | Alerting (7 rules in `tachyon-alerts.yml`) |
| 6 | Tag v1.0.0: Docker images to GHCR, SBOM, changelog |

**Completion:** Public v1.0.0 release. Production at `tachyon.dev` with >99.9% uptime. Monitoring active.

---

## Post-Launch Phases

### Phase 5: AI Integration (Week 6-10) [CODE COMPLETE]

Code complete. Needs production validation.

- Plugin-based AI provider interface (OpenAI, Anthropic, Ollama)
- Semantic search via pgvector embeddings
- Auto-tagging, summarization, RAG Q&A
- Knowledge graph visualization
- **Remaining:** Validate rate limits, test at >100k documents, configure API keys, benchmark embedding latency

### Phase 6: Multi-Tenant SaaS (Week 10-14) [CODE COMPLETE]

Code complete. Needs production validation.

- Per-tenant database isolation (schema-level)
- Usage metering and quota enforcement
- Stripe billing integration
- **Remaining:** Tenant onboarding testing, Stripe webhook config, billing edge cases

### Phase 7: Desktop and Mobile (Week 12-18) [CODE COMPLETE]

Code complete. Known issue: Tauri NVIDIA+WebKitGTK on Linux.

- Mobile-responsive Leptos components
- PWA support (service worker, offline cache)
- Desktop client (Tauri 2.x)
- **Remaining:** Fix NVIDIA+WebKitGTK issue, real device testing, push notification registration

### Phase 8: Plugin Ecosystem (Week 16-22) [CODE COMPLETE]

Code complete. Needs infrastructure.

- Plugin marketplace with registry
- WASM sandboxing (Wasmtime)
- Plugin CLI (scaffold, test, publish)
- Plugin signing and permissions
- **Remaining:** Remote registry hosting, community review process, compatibility matrix

### Phase 9: Enterprise Features (Week 20-28) [CODE COMPLETE]

Code complete. Needs integration testing.

- SAML/SSO, advanced audit logging
- Custom roles and permissions
- White-label branding, org management
- **Remaining:** SAML IdP testing (Okta, Azure AD), LDAP sync, DLP, eDiscovery, SOC 2 prep

---

## Long-Term Vision (6-18 Months Post-Launch)

### Scalability
- Horizontal scaling (stateless backend + Redis session store)
- PostgreSQL read replicas for search/analytics
- CDN edge caching for WASM bundles
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

### Current (v22.0.0)

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
| 5. AI Integration | 2 weeks | Phase 4 | CODE COMPLETE |
| 6. Multi-Tenant SaaS | 2 weeks | Phase 4 | CODE COMPLETE |
| 7. Desktop/Mobile | 2 weeks | Phase 4 | CODE COMPLETE |
| 8. Plugin Ecosystem | 2 weeks | Phase 4 | CODE COMPLETE |
| 9. Enterprise | 4 weeks | Phase 6 | CODE COMPLETE |
| **Total to Production** | **~5 weeks** | | |
| **Total with all phases** | **~19 weeks** | | |

---

## Known Technical Debt

| Item | Severity | Status |
|------|----------|--------|
| 3 `.unwrap()` in production (heartbeat serde, all with fallback) | Low | Fixed (v22) |
| 6 offset-only endpoints (backward compat preserved, cursor endpoints added) | Low | Mitigated (v22) |
| pgvector `update_embedding()` / `search_semantic()` dead code | Medium | Fixed (v22) |
| Audit logging in-memory only (lost on restart) | High | Fixed (v22) |
| 18 highlight regexes compiled per Highlighter::new() | Low | Fixed (v22) |
| Trivy scans only root Dockerfile | Medium | Fixed (v22) |
| SBOM not attached to releases | Medium | Fixed (v22) |
| tachyon/CHANGELOG.md stale (missing v12-v20) | Low | Fixed (v22) |
| 6 stale roadmap files in root (consolidated here) | Low | Fixed (v22) |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | Wire pgvector embeddings into document CRUD | Connect dead code (update_embedding, search_semantic) to AI provider; async spawned to avoid blocking response |
| 2026-05-28 | Add semantic search endpoint | Embed query via AI, search pgvector HNSW index; requires AI configured |
| 2026-05-28 | Wrap GraphQL/Swagger with audit middleware | Routes merged after layer application in Axum bypass middleware; wrap individually |
| 2026-05-28 | Convert 19 regex to LazyLock | Hot-path regex compilation on every call; LazyLock compiles once |
| 2026-05-28 | Add cursor endpoints alongside offset endpoints | Backward compat; new /cursor routes use CursorParams/CursorPage format |
| 2026-05-28 | Trivy matrix scan (3 Dockerfiles) | Root Dockerfile alone insufficient; frontend and server variants have different dependencies |
| 2026-05-28 | Include tachyon-cli in pre-commit clippy/test | Catch lint errors locally; GTK available via Nix |
| 2026-05-28 | Keep tachyon-cli excluded from CI | tachyon-cli depends on tachyon-desktop (GTK), not available in CI runners |
| 2026-05-28 | Fix rustdoc bare URL in ssg/manifest.rs | Eliminate rustdoc warning |
| 2026-05-28 | Fix pre-commit rustdoc gate | Filter out Cargo.toml feature warnings from rustdoc check |
| 2026-05-28 | Consolidate 6 roadmap files into single ROADMAP.md | Eliminate version/count conflicts across documents |
| 2026-05-28 | Update test counts to 2,054 across VERSION.md and ROADMAP.md | Match actual `cargo test` output |
| 2026-05-28 | Fix DEVELOPER_GUIDE.md (Axum 0.8, Rust 1.85+, port 8080, 16 crates) | Correct stale references |
| 2026-05-28 | Replace unsafe from_utf8_unchecked in SSG | Eliminate UB risk |
| 2026-05-28 | Harden CI workflows (permissions, timeouts, pin actions) | Least privilege |
| 2026-05-28 | Persist audit events to database | Fire-and-forget async DB write alongside in-memory store; new audit_events table + permission_audit_log migration |
| 2026-05-28 | Fix theme switching IIFE | Missing }})(); prevented event listener binding; Pagefind output path was double-nested |
| 2026-05-28 | Fix Pagefind output directory | --output-subdir ./site created site/site/pagefind/ (404); fixed to pagefind |
| 2026-05-28 | Fix ZAP scan JWT secret length | 30 chars < 32 minimum; also pgvector/pgvector:pg16 image for vector extension |
| 2026-05-28 | Fix cargo-mutants crate name | crates.io name is cargo-mutants (with s), not cargo-mutant |
| 2026-05-28 | Install cargo-deny in Security CI | Binary not installed; added conditional install before check |
| 2026-05-28 | Skip staging deploy when secrets absent | Prevents false failures on repos without provisioned infrastructure |
| 2026-05-28 | Compile 18 highlight regexes once via LazyLock | Highlighter::new() allocated 18 regex structs per call; now zero-allocation |
| 2026-05-28 | Fix unused utoipa::IntoParams import | user/types.rs imported IntoParams but used full path in derive; CI clippy rejected |
| 2026-05-28 | Fix cargo-mutants --files flag | CLI uses --file (singular), not --files (plural); caused 0s runtime failure |
| 2026-05-28 | Add WebSocket stress test and rate limit validation k6 scripts | Complete load test coverage: smoke, load, stress, spike, soak, ws, rate-limit, security |
| 2026-05-28 | Remove stale k6-load.js, update load test README | Consolidated load test docs, added env var table and threshold table |
| 2026-05-26 | Switch CI to pgvector/pgvector:pg16 | Migration requires CREATE EXTENSION vector |
