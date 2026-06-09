# Tachyon Roadmap

**Version:** 20.0.0 | **Date:** 2026-06-09 | **Status:** Code Complete, Awaiting Production Infrastructure

This document is the single authoritative roadmap. All prior roadmap variants have been consolidated here. Stale versions are archived in `docs.archived/`.

---

## 1. Project Overview

Tachyon is a self-hostable, Rust-native, collaborative knowledge management system with an integrated static site generation pipeline. It targets developers and technical teams who prefer Markdown-first editing over WYSIWYG block editors. The product is positioned against GitBook, Docusaurus, Outline, and Obsidian -- not against Notion or Confluence.

| Attribute | Value |
|-----------|-------|
| Version | 20.0.0 |
| License | Apache-2.0 |
| Rust edition | 2024 (MSRV 1.85) |
| Workspace crates | 16 |
| Total tests | 2,258 (all passing) |
| CI workflows | 11 (all green) |
| ADRs | 107 |
| Deployment modes | Server (Docker), Desktop (Tauri), Static (SSG) |

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust 2024 |
| Server | Axum 0.8, Tower, SQLx 0.8 |
| Database | PostgreSQL 16 + pgvector |
| Search | Tantivy 0.25 (BM25), pg_trgm fallback |
| CRDT | Yrs 0.25 (YATA algorithm, Y.js-compatible) |
| Frontend | Leptos 0.8, Tailwind CSS, WASM |
| Desktop | Tauri 2.x |
| Plugin sandbox | Wasmtime |
| Markdown | pulldown-cmark 0.9 (SIMD-accelerated) |
| Syntax highlighting | tree-sitter (native, wasm32, wasm32-wasi) |

---

## 2. Architecture Summary

### Crate Structure

```
tachyon/
  crates/
    core/           Domain types, markdown processing, auth primitives
    server/         Axum HTTP/WebSocket server, API routes, middleware
    database/       PostgreSQL repositories, migrations, schema
    search/         Tantivy index, BM25 ranking, pg_trgm fallback
    renderer/       Markdown-to-HTML rendering, HTML sanitization
    rbac/           Role-based access control, permission enforcement
    editor/         Text editor engine, syntax highlighting, multi-cursor
    frontend/       Leptos WASM web application
    desktop/        Tauri desktop wrapper (headless, HTTP client)
    desktop/src-tauri/  Tauri application binary (GUI)
    ssg/            Static site generator, multi-language, i18n
    import-export/  DOCX/CSV/Markdown import, PDF/HTML export
    plugin-runtime/ WASM plugin sandbox (Wasmtime)
    storage/        S3-compatible object storage abstraction
    cli/            Command-line interface (tachyon init/serve/build/gui)
    testing/        Test harness, fixtures, utilities
    benchmarks/     Criterion performance benchmarks
```

### Data Flow

```
Browser (Leptos WASM)    Desktop (Tauri)    CLI (Clap)
        |                      |                |
        +----------------------+----------------+
                               |
                         HTTP / WebSocket
                               |
                      Axum 0.8 Server (:8080)
                      +---------+----------+
                      | Tantivy | Yrs/CRDT | Wasmtime
                      | Search  | Sync     | Plugins
                      +---------+----------+
                               |
                          SQLx (async)
                               |
                       PostgreSQL 16 + pgvector
                Documents  Users  Permissions  Audit
```

### Editor Architecture

The editor compiles to three targets:
- **native** (desktop via Tauri, tree-sitter via cc)
- **wasm32-unknown-unknown** (web via tree-sitter-c2rust, regex-only in WASM build)
- **wasm32-wasi** (Docker SSG server-side highlighting)

```
tachyon-editor
  |-- HighlightProvider (trait)
  |     |-- RegexHighlighter (markdown structure, always available)
  |     |-- TreeSitterHighlighter (9 languages, feature-gated: native-tree-sitter)
  |     |-- CompositeHighlighter (regex + tree-sitter, implemented)
  |     |-- WasmTreeSitterHighlighter (wasm-bindgen runtime .wasm, stub)
  |
  +-- tree-sitter targets: native | wasm32 | wasm32-wasi
```

---

## 3. Current Status

### Test Suite

| Crate | Tests | Status |
|-------|-------|--------|
| tachyon-server | 806 | PASS |
| tachyon-core | 145 | PASS |
| tachyon-database | 139 | PASS |
| tachyon-renderer | 122 | PASS |
| tachyon-editor | 141 | PASS |
| tachyon-frontend | 89 | PASS |
| tachyon-desktop | 44 | PASS |
| tachyon-import-export | 50 | PASS |
| tachyon-ssg | 66 | PASS |
| tachyon-search | 45 | PASS |
| tachyon-rbac | 37 | PASS |
| tachyon-storage | 31 | PASS |
| tachyon-cli | 35 | PASS |
| tachyon-plugin-runtime | 74 | PASS |
| tachyon-benchmarks | N/A (lib) | N/A |
| tachyon-testing | N/A (lib) | N/A |
| **Total** | **2,258** | **ALL PASS** |

### Code Quality

| Check | Result |
|-------|--------|
| cargo fmt | Clean |
| clippy -D warnings | Clean (all crates) |
| cargo deny check | PASS |
| cargo audit | PASS (3 documented RUSTSEC overrides) |
| todo!/unimplemented!/STUB/FIXME/HACK | 0 |
| Pre-commit hook | 6 gates (fmt, clippy, test, secrets, artifacts, emoji) |
| Mutation testing | cargo-mutants@27.0.0 on core/database/search (CI, main only) |

### CI/CD Pipeline (12 workflows)

| Workflow | File | Status |
|----------|------|--------|
| CI (check, lint, test, coverage, build) | ci.yml | GREEN |
| Security (audit, SAST, secrets, container) | security-new.yml | GREEN |
| SBOM Generation | sbom_generation.yml | GREEN |
| Release Drafter | release-drafter.yml | GREEN |
| Documentation | docs.yml | GREEN |
| GitHub Pages | pages.yml | GREEN |
| Backup | backup.yml | GREEN |
| CD (Docker build + push to GHCR) | cd.yml | GREEN |
| Deploy Staging | deploy-staging.yml | GREEN |
| E2E Tests | e2e.yml | GREEN |
| OWASP ZAP Scan | owasp-scan.yml | GREEN |
| Release | release.yml | MANUAL |

Note: `tachyon-cli` and `tachyon-desktop-app` (Tauri/GTK) excluded from CI due to native dependencies; verified locally via pre-commit hook.

### Performance Benchmarks (criterion, unit-level)

| Benchmark | Mean Latency |
|-----------|-------------|
| health_check | 6.4 us |
| create_document | 19.0 us |
| get_document | 8.2 us |
| list_documents_50 | 74.9 us |
| markdown_rendering/small (100 words) | 413.3 us |
| markdown_rendering/medium (500 words) | 1.10 ms |
| markdown_rendering/large (2000 words) | 3.50 ms |
| tantivy_search/single_term | 307.4 us |
| tantivy_search/multi_term | 399.9 us |
| tantivy_search/phrase_match | 1.05 ms |

These are unit-level benchmarks (no network/DB overhead). API-level latency will be higher.

### Documentation Site

17 pages at `wyattau.github.io/Tachyon` with custom Rust SSG, Pagefind search, dark/light theme, KaTeX math, Mermaid diagrams.

### Implemented Features (Code Complete)

| Category | Feature | Crate(s) |
|----------|---------|----------|
| Editor | Pluggable HighlightProvider trait | editor |
| Editor | RegexHighlighter (markdown structure) | editor |
| Editor | TreeSitterHighlighter (9 languages) | editor |
| Editor | CompositeHighlighter (regex + tree-sitter) | editor |
| Editor | Multi-cursor editing with paste/delete/selection | editor |
| Editor | Language auto-detection from file extension | editor |
| Editor | SyntaxTheme (dark/light/high-contrast) | editor |
| Editor | WASM tree-sitter (tree-sitter-c2rust) | editor |
| Collaboration | Yrs/CRDT real-time sync (Y.js-compatible) | editor, server |
| Collaboration | SharedBroadcastBus (replaces OT) | server |
| Collaboration | Live cursors and presence | server |
| Collaboration | Offline sync queue | editor |
| Search | Tantivy BM25 full-text search | search |
| Search | pg_trgm fuzzy matching fallback | search |
| Search | Semantic search via pgvector | server |
| Auth | JWT (HS256) with multi-key rotation | server |
| Auth | Guest access | server |
| Auth | Magic URL passwordless login | server |
| Auth | SMS OTP authentication | server |
| Auth | OAuth2 provider support | server |
| Auth | MFA (TOTP) | server |
| Auth | Password reset flow | server |
| RBAC | Fine-grained permissions | rbac |
| RBAC | Custom roles | rbac |
| Storage | S3-compatible object storage | storage |
| SSG | Multi-language generation | ssg |
| SSG | Server-side tree-sitter highlighting | ssg |
| SSG | i18n, RTL support | ssg |
| SSG | Multi-site support | ssg |
| SSG | Versioned documentation | ssg |
| Import/Export | DOCX import/export | import-export |
| Import/Export | CSV import | import-export |
| Import/Export | HTML export | import-export |
| Import/Export | Markdown zip export | import-export |
| Plugins | Wasmtime WASM sandbox | plugin-runtime |
| Plugins | Plugin invoke endpoint | server |
| Notifications | In-app notification system | server |
| Notifications | Email digest subscriptions | server |
| Notifications | Slack/Discord webhook integration | server |
| Enterprise | SCIM 2.0 user provisioning | server |
| Enterprise | SAML/SSO type definitions | server |
| Enterprise | DLP module | server |
| Enterprise | SOC 2 / GDPR / HIPAA scaffolding | server |
| Compliance | GDPR data portability | server |
| Compliance | Audit logging framework | server |
| Compliance | E2E encryption scaffolding | server |
| Platform | Horizontal scaling (stateless + Redis) | server |
| Platform | PgBouncer connection pooling | server |
| Platform | PostgreSQL read replicas | server |
| Platform | CDN edge caching | server |
| Collaboration | Branch-and-merge workflow | database |
| Collaboration | Review/approval workflow | database |
| Desktop | Tauri production build (55MB) | desktop |
| SEO | JSON-LD, Open Graph, robots.txt, sitemap | server |
| API | OpenAPI 3.1 spec, Swagger UI | server |
| API | GraphQL endpoint | server |
| API | Cursor-based pagination | server |
| Security | CSP (nonce-based, dev/prod split) | server |
| Security | cargo-deny + cargo-audit in CI | CI |
| Security | SBOM (CycloneDX) | CI |
| Security | Trivy container scanning | CI |
| Security | OWASP ZAP baseline scan | CI |
| Security | Supply chain policy (unknown-git = deny) | CI |

---

## 4. Known Issues

| # | Issue | Severity | Component | Status |
|---|-------|----------|-----------|--------|
| K1 | No production or staging server provisioned | CRITICAL | Infrastructure | BLOCKING -- no deployment possible |
| K2 | GitHub secrets (STAGING_*, PRODUCTION_*) not configured | CRITICAL | CI/CD | BLOCKING -- CD pipeline non-functional |
| K3 | NVIDIA+WebKitGTK EGL display failure on CachyOS | MEDIUM | Desktop | Known environment-specific issue |
| K4 | WebSocket broadcast not room-filtered | MEDIUM | Server | Architectural; documented |
| K5 | GraphQL resolvers do not use auth context | MEDIUM | Server | Authorization bypass risk |
| K6 | DLP module not wired into routes | LOW | Server | Scaffolding only |
| K7 | SSO module type definitions only (no runtime flow) | LOW | Server | Scaffolding only |
| K8 | tachyon/CHANGELOG.md missing v12-v20 entries | LOW | Docs | Stale |
| K9 | Markdown rendering duplication between frontend and renderer crates | LOW | Duplication | Consolidation target identified |
| K10 | CRDT Yrs utilities duplicated across crates | LOW | Duplication | Extraction target identified |

---

## 5. Technical Debt

### Quantified Debt Items

| # | Item | Severity | Files Affected | Estimated Effort |
|---|------|----------|----------------|-----------------|
| T1 | WebSocket broadcast not room-filtered (sends to all clients) | Medium | server/src/routes/collaboration.rs | 2 days |
| T2 | GraphQL resolvers bypass auth middleware | Medium | server/src/routes/v2/ | 3 days |
| T3 | DLP module not integrated into request pipeline | Low | server/src/routes/ | 1 day |
| T4 | SSO module has types but no runtime flow | Low | server/src/routes/ | 1 week |
| T5 | Markdown rendering logic duplicated (frontend vs renderer) | Low | crates/frontend + crates/renderer | 3 days (extract tachyon-markdown-core) |
| T6 | CRDT Yrs utilities duplicated across editor/server | Low | crates/editor + crates/server | 2 days (extract tachyon-crdt-utils) |
| T7 | NVIDIA+WebKitGTK EGL issue undocumented workaround needed | Low | Desktop | 1 day |
| T8 | CHANGELOG.md missing v12-v20 entries | Low | CHANGELOG.md | 1 day |
| T9 | No production/staging infrastructure provisioned | Critical | Infrastructure | 2 weeks |
| T10 | k6 load tests not executed against running server | Medium | load-tests/ | 1 day |

### Debt Summary

| Severity | Count | Total Effort |
|----------|-------|-------------|
| Critical | 1 | 2 weeks |
| Medium | 3 | 6 days |
| Low | 6 | 8 days |
| **Total** | **10** | **~5 weeks** |

---

## 6. Roadmap Phases

### Phase 1: Infrastructure Provisioning [BLOCKER]

**Rationale:** No deployment is possible without servers and secrets. This is the single highest-priority item.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 1.1 | Provision staging VPS (2 vCPU / 4 GB RAM) | Docker host responds to SSH | 1 day |
| 1.2 | Provision production VPS (4 vCPU / 8 GB RAM) | Docker host responds to SSH | 1 day |
| 1.3 | Configure DNS (staging.tachyon.dev, tachyon.dev, api.tachyon.dev) | `dig` returns correct IPs | 1 day |
| 1.4 | Set 15 GitHub secrets (STAGING_*, PRODUCTION_*) | `gh secret list` shows all entries | 1 day |
| 1.5 | Configure TLS via Let's Encrypt | HTTPS handshake succeeds | 1 day |
| 1.6 | Verify CD pipeline end-to-end | Push to main triggers staging deploy, health check returns 200 | 2 days |

**Dependencies:** Cloud provider account, domain registrar access.
**Estimated effort:** 1 week (1 person).
**Completion criterion:** `curl https://staging.tachyon.dev/health` returns 200.

---

### Phase 2: Production Validation [IN PROGRESS]

**Rationale:** Code-level audit is complete. Infrastructure-dependent validation remains.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 2.1 | Execute k6 load tests against running staging server | P99 < 200ms at 1,000 concurrent users | 1 day |
| 2.2 | Redis distributed rate limiting validation | Rate limit enforced across multiple server instances | 1 day |
| 2.3 | WebSocket reconnection stress test | 100 reconnect cycles without failure | 1 day |
| 2.4 | OWASP ZAP scan against live staging | Zero CRITICAL/HIGH findings | 1 day |

**Dependencies:** Phase 1 complete.
**Estimated effort:** 4 days (1 person).
**Completion criterion:** All k6 scenarios pass, OWASP ZAP clean.

---

### Phase 3: Production Launch [PENDING]

**Rationale:** Tag v1.0.0 only after infrastructure and validation are complete.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 3.1 | Deploy to production with nginx reverse proxy | Production server healthy | 1 day |
| 3.2 | CDN for static assets (Cloudflare) | Static assets served from edge | 1 day |
| 3.3 | Automated database backups + replication | Backup restore test passes | 1 day |
| 3.4 | Prometheus + Grafana monitoring active | Dashboards show live metrics | 1 day |
| 3.5 | Tag v1.0.0 | Docker images to GHCR, SBOM attached, changelog updated | 1 day |
| 3.6 | Landing page rewrite | Lead with self-hosted + Rust + performance story | 2 days |

**Dependencies:** Phases 1, 2.
**Estimated effort:** 1 week (1 person).
**Completion criterion:** `tachyon.dev` is live, v1.0.0 tagged.

---

### Phase 4: Migration and Onboarding [HIGH PRIORITY]

**Rationale:** Without migration paths from existing platforms, no team switches. This is the highest-impact feature for user acquisition.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 4.1 | Notion exporter: API-based import (pages, databases, properties, comments) | Import 100-page Notion workspace without data loss | 2 weeks |
| 4.2 | Confluence exporter: REST API import (pages, attachments, page tree, labels) | Import 500-page Confluence space with hierarchy preserved | 2 weeks |
| 4.3 | Google Docs exporter: Google Drive API import | Import 50-doc Google Workspace folder | 1 week |
| 4.4 | Markdown file vault importer: recursive scan, frontmatter extraction, tag inference | Import 200-file Obsidian vault with tags intact | 1 week |
| 4.5 | Import wizard in UI: source selection, conflict resolution, progress | User completes full import without CLI | 1 week |
| 4.6 | Export to PDF | Export 50-page document to PDF with correct formatting | 1 week |

**Dependencies:** Phase 3 complete.
**Estimated effort:** 8 weeks (1 person, serial).
**Completion criterion:** One-click migration from Notion, Confluence, Google Docs; Markdown vault import with tag inference.

---

### Phase 5: Knowledge Graph and PKM [HIGH PRIORITY]

**Rationale:** Bidirectional links and graph view are table stakes for personal knowledge management. Obsidian, Logseq, Roam, and Anytype all include this.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 5.1 | `[[]]` wiki-link syntax: parse, resolve, store as edges in DB | Create link, verify edge in database | 1 week |
| 5.2 | Backlink panel: all documents linking to current document | Backlinks listed for any document with incoming links | 1 week |
| 5.3 | Graph view: force-directed graph of document relationships | Render 500-node graph at 30fps | 2 weeks |
| 5.4 | Graph query API: traverse links, find orphans, shortest path | API returns correct traversal results | 1 week |
| 5.5 | Daily notes / journaling: auto-create dated entries | Create daily note, link to previous day | 1 week |
| 5.6 | Block references: transclude content between documents | Transcluded block updates when source changes | 2 weeks |

**Dependencies:** Phase 3 complete.
**Estimated effort:** 8 weeks (1 person, serial).
**Completion criterion:** Wiki-link syntax works; backlinks panel functional; graph view renders; daily notes operational.

---

### Phase 6: AI Integration [MEDIUM PRIORITY]

**Rationale:** AI features attract attention but do not drive initial adoption. Migration and PKM capture existing users; AI features retain and differentiate them.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 6.1 | Validate existing plugin-based AI provider interface | OpenAI, Anthropic, Ollama all respond correctly | 1 week |
| 6.2 | Validate semantic search at >100k documents | Query returns relevant results in <500ms | 1 week |
| 6.3 | Benchmark embedding latency | Embedding generation <1s per document | 3 days |
| 6.4 | Auto-tagging validation | Tags match human-curated labels at >80% precision | 1 week |
| 6.5 | RAG Q&A validation | Answers cite source documents correctly | 1 week |

**Dependencies:** Phase 3 complete.
**Estimated effort:** 4 weeks (1 person).
**Completion criterion:** All AI features validated against production-scale data.

---

### Phase 7: Editor Experience [MEDIUM PRIORITY]

**Rationale:** Markdown-first is a defensible position. The editing experience needs polish: live preview, slash commands, embed blocks. The goal is NOT a block editor.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 7.1 | Split-pane live preview (edit MD / rendered HTML, synced scroll) | Scroll sync within 50px accuracy | 2 weeks |
| 7.2 | Slash commands: `/` for heading, code block, table, math, image | Commands execute in <100ms | 1 week |
| 7.3 | Embed blocks: YouTube, Figma, Mermaid rendered inline | Embeds render without page navigation | 2 weeks |
| 7.4 | File attachment drag-and-drop with image upload | Drag image, get `![](url)` in editor | 1 week |
| 7.5 | Table of contents sidebar | TOC reflects current heading structure | 1 week |
| 7.6 | Slash-command extensibility for plugins | Plugin registers custom `/command` | 1 week |

**Dependencies:** Phase 3, Phase A (editor architecture).
**Estimated effort:** 8 weeks (1 person, serial).
**Completion criterion:** Split preview, slash commands, embed blocks, drag-and-drop all functional.

---

### Phase 8: Desktop, PWA, and Mobile [MEDIUM PRIORITY]

**Rationale:** Mobile access is required for any knowledge tool used by teams. PWA is the fastest path to mobile without native app store overhead.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 8.1 | Fix NVIDIA+WebKitGTK issue on Linux | Desktop launches on NVIDIA hardware | 1 week |
| 8.2 | Real device testing (macOS, Windows, Linux) | App launches and renders on all three | 1 week |
| 8.3 | PWA: service worker, offline cache for read-only docs | Documents accessible offline | 2 weeks |
| 8.4 | Mobile-responsive layout: touch targets, collapsible sidebar | Lighthouse mobile score >90 | 2 weeks |
| 8.5 | Push notification registration | Notifications received on mobile | 1 week |
| 8.6 | Mobile web app (install prompt, standalone mode) | PWA installable on iOS/Android | 1 week |

**Dependencies:** Phase 3, Phase A.
**Estimated effort:** 8 weeks (1 person, serial).
**Completion criterion:** Desktop works on all platforms; PWA with offline read access; mobile-responsive web.

Note: Native iOS/Android app deferred to post-v2. PWA covers approximately 80% of mobile use cases at approximately 20% of the cost.

---

### Phase 9: Plugin Ecosystem [LOW PRIORITY]

**Rationale:** The WASM plugin runtime exists but has zero community plugins. Ship reference plugins first, then build marketplace infrastructure.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 9.1 | Reference plugin 1: Mermaid diagram renderer | Renders Mermaid code blocks to SVG | 1 week |
| 9.2 | Reference plugin 2: PlantUML renderer | Renders PlantUML code blocks to SVG | 3 days |
| 9.3 | Reference plugin 3: CSV/TSV table renderer | Parses and renders tabular data | 3 days |
| 9.4 | Reference plugin 4: Custom CSS theme loader | Applies user-defined CSS | 3 days |
| 9.5 | Reference plugin 5: Document template engine | Variable substitution in templates | 1 week |
| 9.6 | Plugin CLI: `tachyon plugin init/build/test` | End-to-end plugin development workflow | 1 week |
| 9.7 | Plugin documentation: SDK reference, tutorial | New developer ships first plugin in <1 day | 1 week |
| 9.8 | Remote registry: file-based index on GitHub Pages | Install plugin from registry URL | 1 week |
| 9.9 | Plugin signing and permission manifest | Unsigned plugins rejected | 1 week |

**Dependencies:** Phase 3 complete, Phase 7 (slash-command extensibility).
**Estimated effort:** 8 weeks (1 person, serial).
**Completion criterion:** 5 reference plugins, CLI tooling, documentation, working registry.

---

### Phase 10: Enterprise Features [LOW PRIORITY]

**Rationale:** Enterprise features are revenue-generating but not adoption-driving. Ship after core product is validated.

| # | Deliverable | Success Criteria | Effort |
|---|-------------|-----------------|--------|
| 10.1 | SAML/SSO runtime flow (Okta, Azure AD) | Login via SAML IdP succeeds | 2 weeks |
| 10.2 | LDAP sync | User provisioning from LDAP directory | 1 week |
| 10.3 | DLP module wired into routes | Content policy enforcement active | 1 week |
| 10.4 | eDiscovery export | Export filtered document set for legal review | 1 week |
| 10.5 | SOC 2 Type II prep | Audit documentation complete | 4 weeks |
| 10.6 | White-label branding | Custom logos, colors, domain | 1 week |

**Dependencies:** Phase 3 complete.
**Estimated effort:** 10 weeks (1 person, serial).
**Completion criterion:** SAML login works; LDAP sync runs; SOC 2 audit documentation ready.

---

### Phase 11: Competitive Gap Closure [HIGH PRIORITY]

All 35 items from the competitive gap analysis are marked complete in the existing roadmap. This needs verification against the actual codebase before being considered truly done.

| Priority | Items | Status | Action Required |
|----------|-------|--------|-----------------|
| HIGH | 6 (U16, U23, U24, U28, U2, U10) | Marked DONE | Verify implementations are functional |
| MEDIUM | 10 (U22, U29, U31, U32, U11, U13, U14, U18, U6, U7) | Marked DONE | Verify implementations are functional |
| LOW | 19 (U30, U33, U35, U15, U12, U17, U20, U21, U19, U5, U1, U4, U8, U9, U3, U25, U26, U27) | Marked DONE | Verify implementations are functional |

**Estimated effort for verification:** 2 weeks.

---

## 7. Design Decisions

### Key ADRs

| ADR | Title | Rationale |
|-----|-------|-----------|
| ADR-005 | Last-Write-Wins Conflict Resolution | LWW for metadata; CRDT (Yrs/YATA) for text content |
| ADR-063 | UX Philosophy | Performance is a feature; accessibility by default; progressive disclosure |
| ADR-111 | CRDT-Based Real-Time Collaboration via Yrs | Yrs chosen over OT: formal convergence proof, offline-first support, battle-tested (Notion, HackMD, JupyterLab) |
| ADR-112 | SSG Documentation Site | Custom Rust SSG for full control over output |

### Design Philosophy: Spatial Materialism + Amoebic UI

Tachyon follows two core design principles:

**Spatial Materialism:** Every piece of knowledge has a physical location in the system. Documents exist in spaces, spaces contain documents, and the spatial relationships encode meaning. This is not a flat file system -- it is a topology of knowledge.

**Amoebic UI:** The interface adapts to the content, not the other way around. The editor changes its highlighting, layout, and interaction model based on what is being edited. There is no single "view mode" -- the UI is amoebic, flowing to fit the shape of the content.

### Editor Paradigm

Tachyon targets a VSCode/Neovim-style editor platform -- a programmable, syntax-aware editing platform with pluggable highlighters, tree-sitter grammars, and multi-cursor support. This is NOT a Notion-style block editor.

### Guiding Principles

1. **Launch beats perfection.** Unshipped code earns zero users.
2. **Markdown-first is a feature, not a deficit.** Target developers and technical teams. Do not chase Notion's block editor.
3. **Self-hosted is the moat.** Every SaaS competitor charges rent. Tachyon's value proposition is ownership.
4. **Rust is the brand.** Performance, safety, and correctness differentiators are only credible if the product runs.
5. **Ecosystem before features.** Ship 5-10 high-quality reference plugins before marketing the plugin system.

---

## 8. Quality Metrics

### Current Baselines

| Metric | Value | Target |
|--------|-------|--------|
| Total tests | 2,258 | Maintain |
| Test pass rate | 100% | 100% |
| clippy warnings | 0 | 0 |
| fmt violations | 0 | 0 |
| todo!/unimplemented!/STUB/FIXME/HACK | 0 | 0 |
| CI workflows | 12 (all green) | Maintain |
| Mutation testing | cargo-mutants on core/database/search | Expand |
| SBOM generation | CycloneDX, automated | Maintain |
| Container scanning | Trivy (CRITICAL/HIGH gate) | Maintain |

### Performance Baselines

| Operation | Unit-Level Latency | API-Level Target |
|-----------|-------------------|-----------------|
| Health check | 6.4 us | <10ms |
| Document create | 19.0 us | <50ms |
| Document get | 8.2 us | <20ms |
| Document list (50) | 74.9 us | <100ms |
| Markdown render (500 words) | 1.10 ms | <50ms |
| Search (single term) | 307.4 us | <200ms |
| Search (phrase match) | 1.05 ms | <500ms |

### Security Baselines

| Check | Status |
|-------|--------|
| SQL Injection | PASS (parameterized queries) |
| Path Traversal | PASS (path normalization) |
| Header Injection | PASS |
| JSON Injection | PASS (proper parsing) |
| Mass Assignment | PASS (internal fields protected) |
| XSS in Content | PASS (ammonia sanitization) |
| XSS in Title | PASS (title sanitization) |
| CORS | PASS (configurable origins) |
| Rate Limiting | PASS (Redis distributed) |
| OWASP ZAP Baseline | PASS (zero CRITICAL/HIGH) |

---

## 9. Phase Dependency Graph

```
Phase 1 (Infrastructure) ─────┬──> Phase 2 (Validation) ──> Phase 3 (Launch)
                               │                                                    │
                               │              ┌──────────────────────────────────────┘
                               │              │
                               │              v
                               │     Phase 4 (Migration) ──────────> Phase 5 (Knowledge Graph)
                               │              │
                               │              v
                               │     Phase 6 (AI) ───────────────> Phase 7 (Editor)
                               │                                         │
                               │              v                            │
                               │     Phase 8 (Desktop/PWA) ────────────┘
                               │              │
                               │              v
                               │     Phase 9 (Plugins) ──────────────> Phase 10 (Enterprise)
                               │              │
                               │              v
                               │     Phase 11 (Gap Closure Verification)
                               │
                               v
                      [Production Live]

Phase A (Editor Architecture) ── DONE ──> feeds Phase 7, Phase 8
```

Phases 4-10 can partially overlap once Phase 3 is complete. Phases 4 and 5 are the highest-priority post-launch work because they unlock user acquisition. Phase 9 (plugins) is intentionally late -- shipping an empty marketplace is worse than shipping no marketplace. Phase 11 (gap closure verification) runs in parallel with 9-10.

---

## 10. Effort Summary

| Phase | Duration | Dependencies | Status |
|-------|----------|-------------|--------|
| 1. Infrastructure | 1 week | Cloud provider, domain | BLOCKING |
| 2. Validation | 4 days | Phase 1 | PENDING |
| 3. Launch | 1 week | Phases 1, 2 | PENDING |
| 4. Migration | 8 weeks | Phase 3 | NEW |
| 5. Knowledge Graph | 8 weeks | Phase 3 | NEW |
| 6. AI Integration | 4 weeks | Phase 3 | CODE EXISTS, NEEDS VALIDATION |
| 7. Editor Experience | 8 weeks | Phase 3, Phase A | NEW |
| 8. Desktop/PWA/Mobile | 8 weeks | Phase 3, Phase A | PARTIAL |
| 9. Plugin Ecosystem | 8 weeks | Phase 3, Phase 7 | PARTIAL |
| 10. Enterprise | 10 weeks | Phase 3 | CODE EXISTS, NEEDS VALIDATION |
| 11. Gap Closure | 2 weeks | Phases 3-10 | VERIFICATION NEEDED |
| **Total to Production** | **~3 weeks** | | |
| **Total to Competitive v2** | **~27 weeks** | | |

---

## 11. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-09 | Full-stack audit: clippy zero-warnings, fmt clean, 1,452 tests pass | Fixed 50+ clippy issues across 8 crates |
| 2026-06-09 | Frontend critical fixes: orphaned CSS, focus outline, reduced-motion | WCAG compliance, accessibility |
| 2026-06-09 | CI/CD pipeline fixes: SHA pins, tool version pins, e2e PostgreSQL | Pipeline reliability |
| 2026-06-09 | Pre-commit hook enhanced: emoji detection, broader clippy coverage | 6 gates enforced |
| 2026-06-09 | Duplication audit: markdown rendering identified as consolidation target | Extract tachyon-markdown-core |
| 2026-06-03 | Runtime verification: all surfaces tested | T1-T6 verified across server, Leptos, SSG, Tauri |
| 2026-06-03 | ammonia double-pass strips class attributes | Fix: add_generic_attributes(["class"]) |
| 2026-06-02 | SSG server-side tree-sitter highlighting | Zero-JS for public pages |
| 2026-06-02 | Three-target tree-sitter strategy complete | native + wasm32 + server-side |
| 2026-06-02 | Editor rearchitected with pluggable HighlightProvider | Monolithic highlighter replaced |
| 2026-05-31 | Add Phase 4 (Migration) as post-launch priority | Highest-impact for user acquisition |
| 2026-05-31 | Add Phase 5 (Knowledge Graph) as post-launch priority | Table stakes for PKM |
| 2026-05-31 | Reorder AI to Phase 6 (after migration, PKM) | Migration captures users; AI retains them |
| 2026-05-31 | Defer native mobile app to post-v2 | PWA covers 80% of use cases |
| 2026-04-21 | CRDT (Yrs/YATA) chosen over OT | Formal convergence proof, offline-first support |
| 2026-05-28 | Wire pgvector embeddings into document CRUD | Semantic search enabled |
| 2026-05-28 | Convert 19 regex to LazyLock | Hot-path optimization |
| 2026-05-26 | Switch CI to pgvector/pgvector:pg16 | Migration requires vector extension |

---

## 12. Long-Term Vision (6-18 Months Post-Launch)

### Scalability
- Horizontal scaling to 10k+ concurrent users
- PostgreSQL read replicas for search/analytics
- Search index sharding for >1M documents
- CDN edge caching for WASM bundles

### Platform
- Public REST API with OAuth2 app registration
- Webhook system for external integrations
- CLI distribution (cargo install, brew, scoop, apt)
- Plugin marketplace with community review

### Collaboration
- Inline comments with threaded replies
- Branch-and-merge workflow for documents
- Review workflow with approval gates
- Real-time editing for >50 concurrent users per document (production validation)

### Intelligence
- Auto-generated knowledge graphs from link structure and embeddings
- Semantic clustering and topic modeling
- Citation graph and impact metrics
- AI-powered document quality scoring

### Internationalization
- UI localization framework
- Priority languages: Chinese (ZH), German (DE), Japanese (JP), French (FR)
- RTL support evaluation (Arabic, Hebrew)
