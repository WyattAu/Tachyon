# Tachyon Production Roadmap

**Version:** 12.0.0 | **Date:** 2026-05-19 | **Status:** Post-audit

---

## Current State Summary

| Metric | Value |
|--------|-------|
| Crates | 16 (+ 1 benchmarks) |
| Lib unit tests | 1,070 passing |
| Zero stubs/TODOs | Confirmed |
| Clippy | Clean (`-D warnings`) |
| Rustdoc | Zero warnings |
| Pre-commit | 6-gate hook active |
| CI | All green (8/8 jobs) |
| Documentation site | Live at wyattau.github.io/Tachyon |
| Deployment modes | Server, CLI, Desktop (Tauri), SSG, WASM frontend |

---

## Known Deficiencies (from audit)

### Critical

1. **2,338 broken `.adrs/ links** across 134 files. The `.adrs/ directory was migrated to `.adrs/` but cross-references were never updated. `scripts/fix_doc_links.sh` exists but was never run at scale.

### High

2. **CD pipeline Docker multi-arch timeout** -- `cd.yml` `Build & Push Docker Images` times out at 30 min for arm64 cross-compilation. Fix: increase timeout-minutes to 60, or build arm64 in a separate job, or use cross-rs.
3. **`backup.yml` will fail** -- missing `.env.template`, no AWS credentials, `kubectl` not installed, missing `gunzip` before `pg_restore`. Entire workflow needs redesign for current infrastructure.
4. **`deploy-staging.yml` image mismatch** -- builds monolithic `Dockerfile` but compose expects separate `tachyon/server` and `tachyon/frontend` images.

### Medium

5. **Action pinning** -- 8+ GitHub Actions referenced by tag, not SHA (supply chain risk).
6. **SSG gap analysis** -- 78 items identified in `SSG_CAPABILITIES_GAP_ANALYSIS.md` remain unaddressed (client-side search, versioned docs, sidebar/TOC, breadcrumbs, syntax highlighting, LaTeX, admonitions).
7. **Performance claims without citations** in `docs/api/api_documentation.md` and `docs/api/authorization_api_specification.md`.
8. **Test count disagreement** -- VERSION.md (1,395), ROADMAP.md (1,325), CHANGELOG (1,589). Actual measured: 1,070 lib tests.

---

## Phase 1: Infrastructure Hardening (1-2 weeks)

### 1.1 Fix broken `.adrs/ references

- Run `scripts/fix_doc_links.sh` across all 134 affected files
- Validate with `rg '\.adrs/ docs/ .adrs/ .reports/ .patterns/ --count`
- Target: zero broken references

### 1.2 Fix CD pipeline

- `cd.yml`: Increase Docker build timeout to 60 min
- `cd.yml`: Use conditional RUSTFLAGS (only for amd64)
- `deploy-staging.yml`: Align Dockerfile references with compose expectations
- `backup.yml`: Redesign for current infra (remove k8s steps, add env template, fix gunzip)
- Pin all GitHub Actions to SHA

### 1.3 CI optimization

- Add `--exclude tachyon-benchmarks` to `e2e.yml` cargo commands
- Cache trunk/wasm-opt binaries in E2E workflow
- Add `cargo-deny` config file if not present

### Deliverables

- All CI/CD workflows passing on main branch
- Zero broken internal links in documentation
- All actions SHA-pinned

---

## Phase 2: SSG Foundation (3-4 weeks)

Based on the 78-item gap analysis in `SSG_CAPABILITIES_GAP_ANALYSIS.md`.

### 2.1 Client-side search

- Port Tantivy search index to WASM or use pre-built index
- Implement search UI component with debounced input
- Fuse with existing search.md documentation
- Test vectors: index 100+ documents, sub-100ms query

### 2.2 Navigation (sidebar, TOC, breadcrumbs, prev/next)

- Implement collapsible sidebar from `site.toml` menu structure
- Extract heading hierarchy for in-page TOC
- Breadcrumb generation from slug path
- Prev/next navigation from document ordering

### 2.3 Syntax highlighting

- Replace hand-rolled approach with tree-sitter highlighting (already in tachyon-renderer)
- Port highlight styles to SSG output
- Support all 11 languages from tree-sitter

### 2.4 LaTeX rendering

- Integrate KaTeX CSS+JS into SSG templates
- Render LaTeX blocks inline in generated HTML
- Test vectors from `tachyon-renderer` latex tests

### 2.5 Admonitions

- Implement `> [!NOTE]`, `> [!WARNING]`, `> [!TIP]`, `> [!DANGER]` syntax
- Style with appropriate colors and icons
- Support in both renderer and SSG

### 2.6 Versioned documentation

- Implement version branching in SSG build
- Version selector UI component
- Redirect latest to unversioned path

### Deliverables

- SSG parity with MkDocs Material for core features
- `tachyon-ssg` builds documentation site with all features above
- Documentation site deployed via GitHub Pages with new features

---

## Phase 3: Build System and Developer Experience (2-3 weeks)

### 3.1 Nix flake improvements

- Add `tachyon-benchmarks` to dev shell
- Add `playwright` for E2E testing
- Add `cargo-deny` to dev tools
- Validate flake evaluates: `nix flake check`

### 3.2 Makefile consolidation

- Merge root `Makefile` and `tachyon/justfile`
- Ensure all targets work with both `make` and `just`
- Add `make docs-preview` for local SSG preview

### 3.3 Workspace hygiene

- Remove orphan `crates/database/src/repository.rs` at repo root
- Remove legacy `tachyon/web/` directory if unused
- Clean up `tachyon/data/tachyon.db` (should not be committed)

### 3.4 Devcontainer updates

- Update `.devcontainer/` to match current flake
- Add VS Code extensions for Rust, Leptos, TOML

### Deliverables

- `nix flake check` passes
- Single source of truth for build commands
- Clean workspace with no orphan files

---

## Phase 4: API and Server Hardening (2-3 weeks)

### 4.1 OpenAPI completeness

- Verify all 33 route modules have utoipa annotations
- Generate OpenAPI spec and validate with `spectral`
- Add response examples for all endpoints

### 4.2 API versioning

- Implement `/api/v2/` prefix with version negotiation
- Deprecation headers for v1 endpoints
- Version-aware middleware

### 4.3 Rate limiting improvements

- Per-user rate limits (not just per-IP)
- Rate limit headers in all responses
- Admin override capability

### 4.4 GraphQL maturity

- Expand beyond placeholder queries in `tachyon-server/graphql/`
- Add subscription support for real-time updates
- Schema stitching with REST API

### 4.5 Performance validation

- Run criterion benchmarks and update VERSION.md with actual numbers
- Validate all performance claims in docs have reproducible benchmarks
- Add P99 latency SLOs to health check endpoint

### Deliverables

- OpenAPI spec covers 100% of endpoints
- API v2 scaffold with version negotiation
- Benchmarks reproducible via `cargo bench`

---

## Phase 5: Real-Time Collaboration v2 (3-4 weeks)

### 5.1 CRDT optimization

- Benchmark Yrs with 100K+ character documents
- Implement lazy document loading (only sync visible viewport)
- Add undo/redo stack per client with server-side validation

### 5.2 Presence and awareness

- Real-time cursor positions with user identification
- Typing indicators
- Active users list per document
- Conflict notification UI

### 5.3 Offline-first improvements

- Offline queue with conflict resolution on reconnect
- Background sync indicator
- Document diff viewer for conflict resolution
- Local-first storage with IndexedDB (WASM frontend)

### 5.4 Operational transform deprecation

- Migrate fully from OT to CRDT (Yrs)
- Remove `websocket/operational_transform.rs` if no longer needed
- Update all tests to CRDT-only paths

### Deliverables

- CRDT collaboration stable at 100K+ char documents
- Offline edit with automatic merge on reconnect
- Zero data loss in concurrent edit scenarios

---

## Phase 6: Advanced Features (4-6 weeks)

### 6.1 AI integration

- Plugin-based AI assistant architecture (via WASM plugin-runtime)
- Embedding generation for semantic search
- RAG (Retrieval-Augmented Generation) pipeline
- Local LLM support via plugin interface

### 6.2 Mobile architecture

- Responsive Leptos frontend for tablet/mobile
- Touch-optimized editor controls
- PWA support (service worker, offline shell)
- Push notification integration

### 6.3 SaaS features

- Multi-tenant isolation at database level
- Per-tenant rate limits and resource quotas
- Billing integration (Stripe webhook processing)
- Usage analytics dashboard

### 6.4 Advanced search

- Semantic search with embeddings
- Faceted search (date ranges, authors, tags, content type)
- Search result highlighting with context snippets
- Saved searches and search alerts

### 6.5 Knowledge graph UI

- Interactive graph visualization (force-directed layout)
- Node clustering and community detection
- Graph-based navigation (click node to open document)
- Relationship editing UI

### Deliverables

- AI plugin interface functional with local model
- PWA installable from documentation site
- Multi-tenant billing operational

---

## Phase 7: Production Launch (1-2 weeks)

### 7.1 Security hardening pass

- Run `cargo audit` with zero critical advisories
- Penetration test against staging environment
- CSP nonce validation in production
- Dependency pinning (Cargo.lock verified)

### 7.2 Observability

- Structured JSON logging throughout server
- Distributed tracing with OpenTelemetry
- Custom Grafana dashboards for all SLOs
- Alert rules for all critical metrics

### 7.3 Documentation finalization

- Fix all 2,338 broken `.adrs/ references
- Validate all performance claims with benchmarks
- Add architecture decision records for all major choices
- User guide with screenshots

### 7.4 Release

- Tag v13.0.0
- Multi-arch Docker images on GHCR
- GitHub Release with SBOM
- Documentation site deployed
- Release notes generated

### 7.5 Post-launch monitoring

- Error rate < 0.1%
- P99 latency < 100ms for all API endpoints
- Uptime > 99.9%
- Zero critical security vulnerabilities

### Deliverables

- Production deployment on target infrastructure
- Monitoring and alerting operational
- User-facing documentation complete

---

## Effort Estimate

| Phase | Duration | Dependencies |
|-------|----------|-------------|
| Phase 1: Infrastructure | 1-2 weeks | None |
| Phase 2: SSG Foundation | 3-4 weeks | Phase 1 |
| Phase 3: Build System | 2-3 weeks | Phase 1 |
| Phase 4: API Hardening | 2-3 weeks | Phase 3 |
| Phase 5: Collaboration v2 | 3-4 weeks | Phase 4 |
| Phase 6: Advanced Features | 4-6 weeks | Phase 4, Phase 5 |
| Phase 7: Production Launch | 1-2 weeks | All above |
| **Total** | **16-24 weeks** | |

Phases 2 and 3 can run in parallel. Phase 4 depends on Phase 3. Phase 5 depends on Phase 4. Phase 6 can start once Phase 4 is complete. Phase 7 is the final gate.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| arm64 Docker build timeout | High | Medium | Increase timeout, separate arm64 job |
| SSG feature creep | High | Medium | Strict scope per gap analysis item |
| CRDT performance at scale | Medium | High | Benchmark early, lazy loading |
| Breaking API changes | Medium | High | Version negotiation from Phase 4 |
| Dependency vulnerability | Low | Critical | Continuous `cargo audit`, pinning |
| Broken doc links regress | Medium | Low | CI check for `.adrs/ references |
