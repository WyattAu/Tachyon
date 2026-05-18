# Tachyon Roadmap

**Version:** 12.0.0 | **Date:** 2026-05-18 | **Status:** CI/CD hardening in progress

---

## Executive Summary

Tachyon is a Rust-based knowledge management system comprising 16 crates across 5 deployment targets (server, CLI, desktop, SSG, WASM frontend). The codebase has 1,395 tests, zero production stubs, clippy-clean with `-D warnings`, and a pre-commit hook enforcing fmt, clippy, tests, rustdoc, and secret scanning.

This roadmap consolidates the previous `ROADMAP.md` (v11), `ROADMAP_FORWARD.md`, `AUDIT_AND_ROADMAP.md`, and `SSG_CAPABILITIES_GAP_ANALYSIS.md` into a single source of truth. Phases A-E from the original roadmap are complete. This document covers the path to production deployment and long-term evolution.

---

## Current CI/CD Status

### Pipeline Health (as of 2026-05-18)

| Pipeline | Status | Passing Jobs | Failing Jobs |
|----------|--------|-------------|--------------|
| CI | Fix pending | Check, Lint, Build Frontend | Test (integration race), Coverage |
| CD | Fix pending | -- | Docker build, trunk build |
| Security | Fix pending | Dependency Audit, SAST, Secrets | Container Scan (OS CVEs) |
| E2E | Fix pending | -- | WASM build |
| Docs | Green | All | None |
| SBOM | Green | All | None |

### Known Issues Being Resolved

1. **Integration test race condition**: `CREATE EXTENSION` in PostgreSQL migration conflicts when tests run concurrently. Fix: `--test-threads=1` for integration test step.
2. **WASM wasm-opt incompatibility**: trunk auto-downloads binaryen with version incompatible with bulk-memory WASM. Fix: `data-wasm-opt="0"` in `index.html`.
3. **Docker desktop crate references**: `Cargo.toml` workspace includes desktop/cli/testing/benchmarks that can't build headless. Fix: `sed` to remove members in Dockerfile.
4. **OS-level CVEs in Docker base image**: glibc, gnutls, zlib vulnerabilities in `debian:bookworm-slim`. Fix: `apt-get upgrade` in runtime stage.
5. **Dependency vulnerabilities**: `lettre` TLS hostname verification (updated to 0.11.22), `rsa` Marvin Attack via sqlx-mysql (ignored: no fix, MySQL unused).

---

## Architecture Overview

```
tachyon/crates/
  core/            -- Domain types, error handling, ID generation, utilities
  server/          -- Axum REST API, WebSocket, middleware (13 layers), 35+ routes
  database/        -- PostgreSQL persistence, 30 migrations, 10+ repositories
  search/          -- Tantivy full-text index (BM25, faceted search)
  renderer/        -- pulldown-cmark Markdown, tree-sitter (9 langs), KaTeX
  rbac/            -- Role-based access control, casbin policy engine
  storage/         -- Dual-backend: SQLite (offline) + PostgreSQL (server)
  ssg/             -- Static site generator, i18n (20 langs), RTL, RSS, sitemap
  plugin-runtime/  -- WASM plugin execution, marketplace registry
  editor/          -- Syntax highlighting, markdown parsing
  cli/             -- CLI binary (init, serve, build, gui)
  frontend/        -- Leptos WASM SPA (90+ components, 30+ pages)
  desktop/         -- Tauri desktop wrapper (embedded server)
  import-export/   -- Document import/export (Markdown, HTML, JSON)
  testing/         -- Test infrastructure (TestApp, fixtures, helpers)
  benchmarks/      -- Criterion benchmarks
```

### Technology Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum 0.8, Tower 0.6, SQLx 0.8, Tantivy 0.25 |
| Frontend | Rust, Leptos, Trunk (WASM), Tailwind CSS 4 |
| Desktop | Tauri 2.x, WebKitGTK |
| Database | PostgreSQL 16 |
| Search | Tantivy (embedded), tsvector fallback |
| Auth | JWT (HS256), bcrypt, optional TOTP MFA |
| CI/CD | GitHub Actions (7 workflows) |
| Container | Docker multi-stage build |
| Documentation | GitHub Pages, custom SSG |

---

## Phase 1: CI/CD Hardening (1 week)

**Objective:** All CI pipelines green. Zero flaky tests. Deterministic builds.

### 1.1 Integration Test Stability

- [x] Fix migration bug (`comments` -> `document_comments` table name)
- [x] Add `--test-threads=1` for integration tests (PostgreSQL extension race)
- [ ] Add migration lock to prevent concurrent `CREATE EXTENSION` (long-term fix)
- [ ] Add test isolation: per-test database schema or transaction rollback

### 1.2 Docker Build

- [x] Exclude desktop/testing/cli/benchmarks from workspace in Dockerfile
- [x] Add `apt-get upgrade` to runtime stage for OS CVEs
- [ ] Pin base image digest for reproducibility
- [ ] Add multi-arch support (arm64)
- [ ] Add HEALTHCHECK validation in CI

### 1.3 Security Pipeline

- [x] Update `lettre` to 0.11.22 (RUSTSEC-2026-0141)
- [x] Ignore RUSTSEC-2023-0071 (rsa via sqlx-mysql, no fix)
- [x] Fix SAST false positives (nginx proxy_pass)
- [ ] Establish baseline for Trivy container scan (upgrade base, accept residual)
- [ ] Pin all tool versions (trunk, trivy, trufflehog, semgrep)

### 1.4 E2E Pipeline

- [x] Fix WASM build (disable wasm-opt)
- [ ] Verify Playwright E2E tests pass with new WASM build
- [ ] Add E2E test results as artifact

**Completion Criteria:** All 7 workflow pipelines pass on `main`. Zero flaky test failures across 5 consecutive runs.

---

## Phase 2: SSG Foundation (3-4 weeks)

**Objective:** Bring `tachyon-ssg` to feature parity with Docusaurus/VitePress for documentation use case.

Based on the 78-item gap analysis in `SSG_CAPABILITIES_GAP_ANALYSIS.md`.

### 2.1 Replace Hand-Rolled Parsers (Week 1)

**Problem:** YAML frontmatter and TOML config use custom string parsing instead of `serde_yaml`/`toml` crates. Fragile, no error recovery.

- [ ] Add `serde` derive to frontmatter structs
- [ ] Replace YAML parser with `serde_yaml`
- [ ] Replace TOML parser with `toml` crate
- [ ] Add schema validation for config files
- [ ] Write migration guide for existing content

**Effort:** 3-4 days

### 2.2 Integrate Renderer Capabilities (Week 1)

**Problem:** `tachyon-renderer` has tree-sitter (9 languages, 3 themes) and KaTeX support, but the SSG never invokes them during page rendering.

- [ ] Wire tree-sitter syntax highlighting into SSG code block rendering
- [ ] Wire KaTeX math rendering into SSG markdown processing
- [ ] Add theme selection to SSG config (github, monokai, dracula)
- [ ] Verify output matches renderer standalone output

**Effort:** 3-4 days

### 2.3 Page Structure (Week 2)

- [ ] Per-page table of contents (extract headings during render)
- [ ] Breadcrumbs (derive from document path)
- [ ] Prev/next page navigation (derive from sort order)
- [ ] Sidebar with auto-generated heading navigation
- [ ] Mobile hamburger menu with slide-out drawer
- [ ] Scroll-to-top button

**Effort:** 5-7 days

### 2.4 Search (Week 2)

- [ ] Integrate Pagefind for client-side search
- [ ] Generate search index during SSG build
- [ ] Add search UI component to generated pages
- [ ] Support multi-language search

**Effort:** 3-4 days

### 2.5 Content Features (Week 3)

- [ ] Admonitions/callouts (`> [!NOTE]`, `> [!WARNING]`, etc.)
- [ ] Tabs component
- [ ] Code groups (multi-language)
- [ ] Copy button on code blocks
- [ ] Mermaid diagram rendering
- [ ] Draft document support

**Effort:** 5-7 days

### 2.6 SEO and Meta (Week 3-4)

- [ ] JSON-LD structured data
- [ ] Generate `robots.txt`
- [ ] `hreflang` links in page `<head>`
- [ ] Open Graph images
- [ ] Canonical URLs

**Effort:** 2-3 days

**Completion Criteria:** Documentation site matches or exceeds MkDocs Material in core features. All Priority 1 and Priority 2 items from gap analysis implemented.

---

## Phase 3: Build System and Developer Experience (2-3 weeks)

**Objective:** Fast, deterministic builds. Pleasant developer onboarding.

### 3.1 SSG Build Performance

- [ ] Incremental builds (only rebuild changed pages)
- [ ] Build-time Tailwind CSS generation (remove CDN dependency)
- [ ] Asset hashing for cache busting
- [ ] Image optimization pipeline
- [ ] Hot reload / live preview mode

**Effort:** 5-7 days

### 3.2 Workspace Hygiene

- [ ] Remove `casbin` `runtime-async-std` feature (use tokio throughout)
- [ ] Unify database backends (PostgreSQL as canonical)
- [ ] Run `cargo udeps` to remove unused dependencies
- [ ] Remove dead `tree-sitter-sql = "0.0"` placeholder
- [ ] Decouple `tachyon-testing` from `tachyon-desktop-app`

**Effort:** 3-5 days

### 3.3 Template Engine

- [ ] Replace `format!()` HTML interpolation with minijinja or askama
- [ ] Template inheritance (base layout, page partials)
- [ ] Custom filters (date formatting, slugify, markdown)

**Effort:** 3-5 days

### 3.4 Developer Tooling

- [ ] `just` task runner for common commands
- [ ] Pre-built Nix dev shell with all dependencies
- [ ] VS Code devcontainer configuration
- [ ] One-command setup: clone, nix-shell, cargo test

**Effort:** 2-3 days

**Completion Criteria:** `tachyon ssg build` runs in <5s for incremental. New contributor can build and test in <10 minutes.

---

## Phase 4: API and Server Hardening (2-3 weeks)

**Objective:** Production-ready server. Rate limiting, observability, graceful degradation.

### 4.1 Rate Limiting

- [ ] Token bucket rate limiter middleware
- [ ] Per-endpoint configuration
- [ ] Per-user rate limits (authenticated)
- [ ] Rate limit headers (`X-RateLimit-Remaining`)

### 4.2 Observability

- [ ] Structured JSON logging
- [ ] Request tracing (request ID propagation)
- [ ] Metrics endpoint (Prometheus format)
- [ ] Health check with dependency status

### 4.3 API Improvements

- [ ] OpenAPI 3.1 spec generation (utoipa)
- [ ] API versioning (`/api/v1/`, `/api/v2/`)
- [ ] Cursor-based pagination (replace offset)
- [ ] Field selection (`?fields=id,title`)
- [ ] Batch operations

### 4.4 WebSocket Reliability

- [ ] Automatic reconnection with exponential backoff
- [ ] Message ordering guarantees
- [ ] Presence heartbeat with TTL cleanup
- [ ] Room-based broadcasting optimization

**Effort:** 10-15 days total

---

## Phase 5: Real-Time Collaboration v2 (3-4 weeks)

**Objective:** Multi-user editing with conflict resolution.

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

**Effort:** 15-20 days total

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
- [ ] API docs from OpenAPI spec

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

**Effort:** 20-30 days total

---

## Effort Summary

| Phase | Duration | Key Deliverable |
|-------|----------|-----------------|
| 1: CI/CD Hardening | 1 week | All pipelines green |
| 2: SSG Foundation | 3-4 weeks | Feature-parity docs site |
| 3: Build System & DX | 2-3 weeks | Fast builds, easy onboarding |
| 4: API & Server | 2-3 weeks | Production-ready API |
| 5: Real-Time Collab v2 | 3-4 weeks | Multi-user editing |
| 6: Advanced Features | 4-6 weeks | AI, mobile, SaaS |
| **Total** | **15-21 weeks** | **Full production platform** |

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CI flaky tests | Medium | High | Serial integration tests, migration locking |
| WASM toolchain breakage | Medium | Medium | Pin trunk/wasm-bindgen versions in CI |
| Docker base image CVEs | High | Medium | Automated `apt-get upgrade`, Trivy scanning |
| SSG parser rewrite regressions | Medium | High | Byte-for-byte comparison tests before/after |
| Tauri 2.x API instability | Medium | Medium | Pin tauri version, isolate desktop crate |
| PostgreSQL extension conflicts | Medium | High | `IF NOT EXISTS`, advisory locks, serial tests |
| Supply chain attack | Low | Critical | `cargo audit`, SBOM, pinned digests |
| Performance regression | Medium | High | Criterion benchmarks in CI, alerting |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-18 | Disable wasm-opt in trunk build | Auto-downloaded binaryen incompatible with bulk-memory WASM |
| 2026-05-18 | Use `data-wasm-opt="0"` not Trunk.toml | Trunk.toml `wasm_opt` only controls version; HTML attribute controls behavior |
| 2026-05-18 | Exclude desktop crates via sed in Docker | Can't remove from workspace (Tauri deps); Docker can't build GTK headless |
| 2026-05-18 | Ignore RUSTSEC-2023-0071 | rsa via sqlx-mysql, no fix available, MySQL not used |
| 2026-05-18 | Serial integration tests | PostgreSQL `CREATE EXTENSION` race condition in concurrent setup |
| 2026-05-13 | Use `ammonia` for HTML sanitization | OWASP-recommended, battle-tested |
| 2026-05-13 | Single-tenant architecture for v11 | Multi-tenant is v12 feature |
| 2026-02-11 | Nix flake for dev environment | Reproducible builds |

---

## Migration Guide for Existing Roadmap Files

This document supersedes:
- `ROADMAP.md` (v11.0.0) -- Phases A-F marked complete
- `ROADMAP_FORWARD.md` -- Items folded into Phases 1-6 above
- `AUDIT_AND_ROADMAP.md` -- Audit findings addressed; remaining items in Phase 1
- `SSG_CAPABILITIES_GAP_ANALYSIS.md` -- Detailed gap list; implementation in Phase 2

Those files should be archived or marked as superseded once this roadmap is ratified.
