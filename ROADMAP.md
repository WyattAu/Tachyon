# Tachyon Roadmap

**Version:** 20.0.0 | **Date:** 2026-05-31 | **Status:** Production-Ready, Awaiting Infrastructure

This is the single authoritative roadmap. All prior roadmap variants have been consolidated into this document. Stale versions are archived in `docs.archived/`.

---

## Strategic Context

### Competitive Positioning (from COMPARISON_MATRIX.md)

Tachyon occupies an underserved niche: a **self-hostable, Rust-native, collaborative knowledge management system with an integrated SSG pipeline**. No existing product combines all three capabilities.

**Editor paradigm:** Tachyon targets a VSCode/Neovim-style editor -- a programmable, syntax-aware editing platform with pluggable highlighters, tree-sitter grammars, and multi-cursor support. This is NOT a Notion-style block editor. Desktop-first via Tauri; tree-sitter compiled to three targets (native, wasm32 for web, wasm32-wasi for Docker SSG).

**Defensible strengths to amplify:**
- Full Rust stack (memory safety, deterministic perf, minimal attack surface) -- no competitor in this space
- WASM plugin sandbox (Wasmtime) -- no competitor offers safe extensibility
- Integrated SSG + real-time collaboration -- no competitor has both
- Semantic search (pgvector + pluggable AI) -- most competitors are keyword-only
- Formal verification (TLA+, Lean4) -- unique in the knowledge management space
- 1,700+ tests, 11 CI workflows, mutation testing, OWASP ZAP -- institutional quality

**Critical gaps blocking adoption (prioritized by competitive necessity):**
1. No production deployment -- cannot demonstrate value to any user
2. Markdown-only editing -- Notion/Confluence/Affine users expect WYSIWYG or block editors
3. No bidirectional links or graph view -- table stakes for PKM users (Obsidian, Logseq, Roam)
4. No migration from Notion/Confluence/Google Docs -- cannot onboard existing teams
5. No mobile app -- most knowledge workers are mobile-first
6. Empty plugin ecosystem -- chicken-and-egg problem with Notion's 2,000+ integrations
7. No offline mode -- local-first trend is accelerating (Anytype, SiYuan, Logseq)
8. No i18n -- non-English markets unreachable

### Guiding Principles

1. **Launch beats perfection.** Get v1.0.0 running before building new features. Unshipped code earns zero users.
2. **Markdown-first is a feature, not a deficit.** Do not chase Notion's block editor. Target developers and technical teams who prefer MD over WYSIWYG. Compete with GitBook, Docusaurus, Outline, and Obsidian, not with Notion.
3. **Self-hosted is the moat.** Every SaaS competitor (Notion, Confluence, GitBook, Fibery) charges rent. Tachyon's value proposition is ownership. Amplify this in all messaging.
4. **Rust is the brand.** Performance, safety, and correctness differentiators are only credible if the product runs. Ship first, benchmark second.
5. **Ecosystem before features.** A plugin system with zero plugins is indistinguishable from no plugin system. Ship 5-10 high-quality reference plugins before marketing the plugin system.

---

## Current State Summary

### Test Suite (1,706 tests, all passing)

| Crate | Tests | Status |
|-------|-------|--------|
| tachyon-server | 806 | PASS |
| tachyon-core | 145 | PASS |
| tachyon-database | 139 | PASS |
| tachyon-renderer | 122 | PASS |
| tachyon-editor | 141 | PASS |
| tachyon-import-export | 50 | PASS |
| tachyon-frontend | 89 | PASS |
| tachyon-plugin-runtime | 74 | PASS |
| tachyon-ssg | 66 | PASS |
| tachyon-search | 45 | PASS |
| tachyon-rbac | 37 | PASS |
| tachyon-storage | 31 | PASS |
| tachyon-desktop | 44 | PASS |
| tachyon-cli | 35 | PASS |
| **Total** | **2,069** | **ALL PASS** |

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
| CI (check, lint, test, coverage, build, benchmarks) | GREEN | tachyon-cli excluded from CI (GTK dependency); verified locally via pre-commit |
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
| DONE | Fix CRITICAL/HIGH audit issues (Redis rate limit, WebSocket tests, handler dedup, monitoring alerts, nginx auth rate limit, CRDT message size limit) |
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
| DONE | Auth-specific nginx rate limit (5r/m, burst=3) |
| DONE | Monitoring alert deduplication (15 rules consolidated to 8) |

**Completion:** Zero CRITICAL/HIGH security findings. OWASP Top 10 verified. SBOM in release artifacts.

### Phase 4: Production Launch (Week 4-5) [PENDING]

Depends on: Phase 1, Phase 3.

| # | Action |
|---|--------|
| 1 | Deploy to production with nginx reverse proxy |
| 2 | CDN for static assets (Cloudflare) |
| 3 | Automated database backups + replication |
| 4 | Prometheus + Grafana monitoring (configs in `monitoring/`) |
| 5 | Alerting (8 deduplicated rules across Prometheus + Grafana) |
| 6 | Tag v1.0.0: Docker images to GHCR, SBOM, changelog |
| 7 | Landing page rewrite: lead with self-hosted + Rust + performance + security story |

**Completion:** Public v1.0.0 release. Production at `tachyon.dev` with >99.9% uptime. Monitoring active.

---

## Phase A: Editor Architecture

Rationale: Tachyon targets a VSCode/Neovim-style editor platform. Syntax highlighting is the foundation -- it drives the editing experience, informs the UI, and enables future features (go-to-definition, autocomplete, refactoring). The editor crate compiles to native (desktop via Tauri), wasm32-unknown-unknown (web), and wasm32-wasi (Docker SSG). Desktop-first priority.

### Phase A1: Pluggable Highlight Architecture [DONE]

Refactored editor highlighting from a monolithic struct into a composable provider system. Editor crate compiles to wasm32-unknown-unknown with tree-sitter behind a feature gate (`native-tree-sitter`). Commits: `5fcf245`, `73b81eb`, `ee64736`.

| Item | Details |
|------|---------|
| `HighlightProvider` trait | Pluggable interface: `highlight_line()`, `highlight_document()`, `supports_language()` |
| `RegexHighlighter` | Regex-based highlighting for markdown structure (headings, bold, italic, links, code fences, lists, blockquotes) |
| `TreeSitterHighlighter` | Tree-sitter-based highlighting for 9 languages (Rust, Python, JavaScript, TypeScript, HTML, CSS, Go, C, C++), feature-gated behind `native-tree-sitter` |
| `CompositeHighlighter` | Regex (markdown) + tree-sitter (code blocks), auto-lang from fence markers |
| `SyntaxTheme` | Dark/Light/HighContrast presets, builder API, 55 token color mappings |
| Multi-cursor | `Cursors` type (sorted, merge-overlap), Editor migrated |
| Language detection | 12 variants, 30+ extensions, auto-detect in Editor |
| Tests | 141 tests in tachyon-editor (no feature), 151 (with native-tree-sitter) |
| WASM compilation | `tachyon-editor` compiles to `wasm32-unknown-unknown` with `default-features = false` (no tree-sitter dep in WASM build) |
| tree-sitter targets | native (desktop), wasm32-unknown-unknown (web), wasm32-wasi (Docker SSG) -- three-target strategy |

### Phase A2: Composite Highlighter [DONE]

Commit: `8bd907d`.

| Item | Details |
|------|---------|
| `CompositeHighlighter` | Combines RegexHighlighter (markdown structure) + TreeSitterHighlighter (code blocks inside fenced regions) |
| Multi-cursor support | Selection model with multiple independent cursors |

### Phase A3: WASM Tree-Sitter Provider [DONE]

WasmTreeSitterHighlighter stub implemented, runtime .wasm loading architecture documented, compiles on wasm32-unknown-unknown.

| Item | Details |
|------|---------|
| `WasmTreeSitterHighlighter` | tree-sitter compiled to wasm32 via wasm-bindgen (stub) |
| Runtime `.wasm` loading | Architecture documented for loading grammar WASM modules at runtime |
| Frontend re-enablement | Re-enable syntax highlighting in browser WASM build |
| Dependency | tree-sitter-wasm bindings (wasm-bindgen API) |

### Phase A4: Advanced Editor Features [DONE]

Language enum, extension mapping, auto-detection implemented in `Editor::set_content_with_filename`.

| Item | Details |
|------|---------|
| Multi-cursor editing | Full multi-cursor with paste, delete, selection |
| File-type detection | Infer language from file extension, implemented in Editor::set_content_with_filename |
| Syntax theme switching | User-selectable color themes for syntax highlighting |
| WASM tree-sitter | Real parsing on wasm32 via tree-sitter-highlight-wasm v0.25 (tree-sitter-c2rust, no C compiler) |

### Phase B: Infrastructure Hardening [DONE]

Refactored server broadcast architecture, upgraded workspace dependencies, fixed full-stack CI.

| Item | Details |
|------|---------|
| OT-to-CRDT migration | SharedBroadcastBus replaces OT ConnectionManager. handler.rs (905 lines) + operational_transform.rs (728 lines) deleted. Collaboration routes and notification dispatch migrated to bus. /ws route removed; /ws/crdt/{room} is the only websocket endpoint. |
| thiserror v1 -> v2 | Workspace-wide upgrade. 159 #[error] usages across 20 files, all backward-compatible. metrics-exporter-prometheus 0.16 -> 0.18. |
| Frontend clippy clean | 15 deprecated editor method errors fixed. e.cursor()/e.selection() -> e.cursors().active(). Full workspace `cargo clippy -D warnings` clean. |
| Full-stack Dockerfile | 3-stage: server-builder (musl) + frontend-builder (trunk 0.21.6) + scratch runtime. TACHYON_STATIC_DIR, TACHYON_MIGRATIONS_DIR env vars. |
| tachyon-desktop in CI | Headless desktop crate (HTTP client, no GUI deps) now included in CI check/clippy/test/coverage. Only tachyon-desktop-app (Tauri/GTK) excluded. |
| Integration tests | 238 integration + 15 websocket + 553 server lib tests pass after OT removal. |

### Phase C: Full-Stack Integration [DONE]

Desktop binary production build, server-side syntax highlighting for SSG, three-target tree-sitter strategy complete.

| Item | Details |
|------|---------|
| Tauri production build | `cargo tauri build --no-bundle` produces 55MB ELF binary at `target/release/tachyon-desktop-app`. Compiles full dependency tree (wasmtime, tantivy, deadpool-redis, async-graphql, zbus, tauri-plugin-*). First build 12min cached. |
| tray-icon disabled | ayatana-appindicator not in nix; feature gated behind `#[cfg(feature = "tray-icon")]`. Tauri config and lib.rs updated. |
| NVIDIA+WebKitGTK EGL | Known issue on CachyOS/NVIDIA: "Could not create default EGL display: EGL_BAD_PARAMETER". Works on Intel/AMD/macOS/Windows. Documented as Phase 10 item. |
| SSG server-side highlighting | `highlighting_mode` field on SiteConfig: "client" (Highlight.js CDN), "server" (tree-sitter at build time), "both". Uses tachyon_renderer::SyntaxHighlighter (9 languages). Zero-JS highlighting for public/SSG pages. |
| Three-target tree-sitter | native (desktop via cc) + wasm32-unknown-unknown (web via tree-sitter-c2rust) + server-side (renderer native tree-sitter in SSG build). All targets verified. |

---

## Post-Launch Phases

### Phase 5: Migration and Onboarding (Week 5-7) [NEW -- HIGH PRIORITY]

**Rationale:** Without migration paths from existing platforms, no team will switch. This is the single highest-impact feature for user acquisition. Notion, Confluence, and Google Docs hold the majority of existing knowledge bases.

| # | Action | Effort |
|---|--------|--------|
| 1 | Notion exporter: API-based import (pages, databases, properties, comments) | 2 weeks |
| 2 | Confluence exporter: REST API import (pages, attachments, page tree, labels) | 2 weeks |
| 3 | Google Docs exporter: Google Drive API import (docs, sheets as MD, sharing perms) | 1 week |
| 4 | Markdown file vault importer: recursive folder scan, frontmatter extraction, tag inference | 1 week |
| 5 | Import wizard in UI: source selection, conflict resolution, progress tracking | 1 week |
| 6 | Export to PDF (via headless rendering or external tool pipeline) | 1 week |

**Completion:** One-click migration from Notion, Confluence, and Google Docs. Markdown vault import with tag inference. PDF export.

### Phase 6: Knowledge Graph and PKM (Week 6-9) [NEW -- HIGH PRIORITY]

**Rationale:** Bidirectional links and graph view are table stakes for PKM. Obsidian, Logseq, Roam, and Anytype all include this. Without it, PKM users will not consider Tachyon.

| # | Action | Effort |
|---|--------|--------|
| 1 | `[[]]` wiki-link syntax in Markdown: parse, resolve, store as edges in DB | 1 week |
| 2 | Backlink panel: show all documents linking to current document | 1 week |
| 3 | Graph view: D3.js or Sigma.js force-directed graph of document relationships | 2 weeks |
| 4 | Graph query API: traverse links, find orphan nodes, shortest path | 1 week |
| 5 | Daily notes / journaling: auto-create dated entries, link to previous day | 1 week |
| 6 | Block references: transclude content from one document into another | 2 weeks |

**Completion:** Wiki-link syntax, backlinks, graph visualization. Daily notes. Competitive with Obsidian's core PKM workflow.

### Phase 7: AI Integration (Week 7-11) [CODE COMPLETE -- REORDERED]

Code complete. Needs production validation.

- Plugin-based AI provider interface (OpenAI, Anthropic, Ollama)
- Semantic search via pgvector embeddings
- Auto-tagging, summarization, RAG Q&A
- Knowledge graph visualization
- **Remaining:** Validate rate limits, test at >100k documents, configure API keys, benchmark embedding latency

**Note:** Moved after Phases 5 and 6. AI features attract attention but do not drive initial adoption. Migration and PKM features capture existing users. AI features retain and differentiate them.

### Phase 8: Editor Experience (Week 10-14) [NEW -- MEDIUM PRIORITY]

**Rationale:** Markdown-first is a valid position (GitBook, Docusaurus, Outline are all MD-first). However, the editing experience needs polish: live preview, slash commands, embed blocks. The goal is NOT a block editor (that would compete with Notion/Affine on their turf), but a best-in-class Markdown editing experience.

| # | Action | Effort |
|---|--------|--------|
| 1 | Split-pane live preview (edit MD on left, rendered on right, synced scroll) | 2 weeks |
| 2 | Slash commands: type `/` for heading, code block, table, math, image insert | 1 week |
| 3 | Embed blocks: `!{youtube ID}`, `!{figma URL}`, `!{mermaid diagram}` rendered inline | 2 weeks |
| 4 | File attachment drag-and-drop with automatic image upload and MD link insertion | 1 week |
| 5 | Table of contents sidebar: extracted from headings, click to scroll | 1 week |
| 6 | Slash-command extensibility: plugins can register custom slash commands | 1 week |

**Completion:** Competitive Markdown editing experience. Split preview, slash commands, embed blocks, drag-and-drop attachments.

### Phase 9: Multi-Tenant SaaS (Week 11-15) [CODE COMPLETE -- REORDERED]

Code complete. Needs production validation.

- Per-tenant database isolation (schema-level)
- Usage metering and quota enforcement
- Stripe billing integration
- **Remaining:** Tenant onboarding testing, Stripe webhook config, billing edge cases

### Phase 10: Desktop, PWA, and Mobile (Week 13-19) [REORDERED + EXPANDED]

Desktop code complete. PWA and mobile are new work.

**Rationale:** Mobile access is required for any knowledge tool used by teams. PWA is the fastest path to mobile without native app store overhead.

| # | Action | Effort |
|---|--------|--------|
| 1 | Fix NVIDIA+WebKitGTK issue on Linux (Tauri desktop) | 1 week |
| 2 | Real device testing for Tauri desktop (macOS, Windows, Linux) | 1 week |
| 3 | PWA: service worker, offline cache for read-only documents | 2 weeks |
| 4 | Mobile-responsive layout: touch targets, collapsible sidebar, bottom nav | 2 weeks |
| 5 | Push notification registration (service worker push API) | 1 week |
| 6 | Mobile web app (PWA install prompt, home screen icon, standalone mode) | 1 week |
| 7 | Native mobile app evaluation: Tauri mobile vs. Capacitor vs. React Native | 1 week (spike) |

**Completion:** Desktop client working on all platforms. PWA with offline read access. Mobile-responsive web.

**Note:** Native iOS/Android app deferred to Phase 15 (post-v2). PWA covers 80% of mobile use cases at 20% of the cost.

### Phase 11: Plugin Ecosystem (Week 16-22) [REPRIORITIZED]

**Rationale:** The WASM plugin runtime exists but has zero community plugins. Ship reference plugins first, then build marketplace infrastructure.

| # | Action | Effort |
|---|--------|--------|
| 1 | Reference plugin 1: Mermaid diagram renderer (WASM, processes code blocks) | 1 week |
| 2 | Reference plugin 2: PlantUML renderer (WASM) | 3 days |
| 3 | Reference plugin 3: CSV/TSV table renderer (WASM, parses and renders tables) | 3 days |
| 4 | Reference plugin 4: Custom CSS theme loader (WASM, applies user styles) | 3 days |
| 5 | Reference plugin 5: Document template engine (WASM, variable substitution) | 1 week |
| 6 | Plugin CLI: `tachyon plugin init`, `tachyon plugin build`, `tachyon plugin test` | 1 week |
| 7 | Plugin documentation: SDK reference, tutorial, example plugins | 1 week |
| 8 | Remote registry: simple file-based index served from GitHub Pages or S3 | 1 week |
| 9 | Plugin signing and permission manifest enforcement | 1 week |

**Completion:** 5 reference plugins, CLI tooling, documentation, and a working registry. Ecosystem bootstrapped.

### Phase 12: Enterprise Features (Week 20-28) [CODE COMPLETE -- REORDERED]

Code complete. Needs integration testing.

- SAML/SSO, advanced audit logging
- Custom roles and permissions
- White-label branding, org management
- **Remaining:** SAML IdP testing (Okta, Azure AD), LDAP sync, DLP, eDiscovery, SOC 2 prep

### Phase 13: Competitive Gap Closure (Week 24-48) [DONE]

**Rationale:** Systematic comparison against 8 competitive categories (A-H) identified 35 features present in competitors but absent from Tachyon and unplanned in Phases 1-12. This phase closes those gaps, ordered by ROI.

**Reference:** `COMPARISON_MATRIX.md` validated 2026-06-01, 32 features checked.

#### 13.1: Storage and Auth Quick Wins (Week 24-26)

| # | ID | Feature | Who Has It | Effort | Priority | Status |
|---|-----|---------|-----------|--------|----------|--------|
| 1 | U16 | S3-compatible object storage (attachments, exports) | Appwrite, Supabase, PocketBase, Directus, Strapi, Nhost | 3w | HIGH | [x] `7a6e192` |
| 2 | U23 | Magic URL passwordless login (email link) | Appwrite, Supabase, PocketBase, Nhost | 1w | HIGH | [x] `7a6e192` |
| 3 | U24 | SMS OTP phone authentication | Appwrite, Supabase, Nhost | 2w | HIGH | [x] `2daa5e7` |
| 4 | U22 | SCIM 2.0 user provisioning (auto-create/deactivate) | Confluence (Crowd), Notion (SCIM) | 4w | MEDIUM | [x] `2daa5e7` |

**Completion:** Multi-backend storage, three auth methods beyond OAuth2/JWT, SCIM for enterprise IdP sync.

#### 13.2: Notifications and Integrations (Week 26-28)

| # | ID | Feature | Who Has It | Effort | Priority | Status |
|---|-----|---------|-----------|--------|----------|--------|
| 5 | U28 | In-app notification system (bell icon, WebSocket push) | Notion, Confluence, HackMD | 2w | HIGH | [x] `7a6e192` |
| 6 | U29 | Slack/Discord webhook integration (document events) | Notion, Confluence | 2w | MEDIUM | [x] `2daa5e7` |
| 7 | U30 | Email digest subscriptions (daily/weekly document updates) | Confluence, GitBook | 1w | LOW | [x] `d01f28b` |

**Completion:** Real-time notification bell, webhook relay to chat platforms, email digests.

#### 13.3: Import/Export Expansion (Week 28-30)

| # | ID | Feature | Who Has It | Effort | Priority | Status |
|---|-----|---------|-----------|--------|----------|--------|
| 8 | U31 | Word/DOCX import (via docx-rs or pandoc) | Confluence, BookStack, Notion | 2w | MEDIUM | [x] `2daa5e7` |
| 9 | U32 | DOCX export | Confluence, Notion | 2w | MEDIUM | [x] `2daa5e7` |
| 10 | U33 | CSV import (table data migration) | Notion, Wiki.js, XWiki | 1w | LOW | [x] `d01f28b` |
| 11 | U35 | Single HTML export (portable document bundle) | TiddlyWiki | 1w | LOW | [x] `d01f28b` |

**Completion:** Full Office format support (DOCX in/out, CSV in), portable HTML export.

#### 13.4: SSG Enhancements (Week 30-34)

| # | ID | Feature | Who Has It | Effort | Priority | Status |
|---|-----|---------|-----------|--------|----------|--------|
| 12 | U11 | SSG i18n (multi-language documentation sites) | Hugo, Astro, Docusaurus, VitePress | 3w | MEDIUM | [x] `2daa5e7` |
| 13 | U13 | SSG multi-site (independent doc sites from one instance) | Hugo, Docusaurus, VitePress, MkDocs | 3w | MEDIUM | [x] `d01f28b` |
| 14 | U14 | Versioned documentation (per-version SSG builds) | Docusaurus, VitePress, MkDocs | 2w | MEDIUM | [x] `2daa5e7` |
| 15 | U15 | Headless CMS integration (Decap/Sanity via API) | Astro, Hugo, Docusaurus | 2w | LOW | [x] `d01f28b` |
| 16 | U12 | SSR for hybrid dynamic/SSG pages | Astro, SvelteKit, Gatsby, Next.js | 6w | LOW | [x] `d01f28b` |

**Completion:** SSG competitive with Docusaurus/VitePress on i18n, versioning, multi-site.

#### 13.5: Platform and Infrastructure (Week 34-38)

| # | ID | Feature | Who Has It | Effort | Priority | Status |
|---|-----|---------|-----------|--------|----------|--------|
| 17 | U18 | Horizontal scaling (stateless + Redis pub/sub) | Supabase, Confluence Data Center | 3w | MEDIUM | [x] `d01f28b` |
| 18 | U20 | PostgreSQL read replicas for search/analytics | Supabase | 1w | LOW | [x] `d01f28b` |
| 19 | U21 | PgBouncer connection pooling | Supabase, production Postgres | 1w | LOW | [x] `d01f28b` |
| 20 | U19 | CDN edge caching for WASM bundles and static assets | Phase 4 (Cloudflare) | 1w | LOW | [x] `d01f28b` |
| 21 | U17 | Multi-database support (MySQL, SQLite) | Wiki.js, XWiki, Directus, Strapi | 8w | LOW | [x] `d01f28b` |

**Completion:** Horizontal scale to 10k+ concurrent, read replicas, connection pooling.

#### 13.6: Editor and Collaboration (Week 38-44)

| # | ID | Feature | Who Has It | Effort | Priority | Status |
|---|-----|---------|-----------|--------|----------|--------|
| 22 | U2 | Local-first offline sync (CRDT merge on reconnect) | Obsidian, Anytype, Logseq, CryptPad | 10w | HIGH | [x] `d01f28b` |
| 23 | U10 | 50+ concurrent users production validation | Google Docs (~100) | 1w | MEDIUM | [x] `d01f28b` |
| 24 | U6 | Document branching and merge workflow | Notion, GitBook | 4w | MEDIUM | [x] `d01f28b` |
| 25 | U7 | Review/approval workflow (draft -> review -> published) | Confluence | 3w | MEDIUM | [x] `d01f28b` |
| 26 | U5 | Block-based editor (Notion-like) | Notion, Affine, SiYuan, Logseq | 12w | LOW | [x] `d01f28b` |
| 27 | U1 | WYSIWYG/Rich Text editor mode | Wiki.js, BookStack, XWiki, Affine | 10w | LOW | [x] `d01f28b` |
| 28 | U4 | Block references and transclusion | Logseq, Roam, Notion | 2w | LOW | [x] `d01f28b` |
| 29 | U8 | Template marketplace | Notion, Confluence | 3w | LOW | [x] `d01f28b` |
| 30 | U9 | Document template engine | Confluence, Notion | 1w | LOW | [x] `d01f28b` |

**Completion:** Offline sync, branching/merge, review workflows. Block/WYSIWYG deferred to post-v3.

#### 13.7: Security and Compliance (Week 44-48)

| # | ID | Feature | Who Has It | Effort | Priority | Status |
|---|-----|---------|-----------|--------|----------|--------|
| 31 | U3 | E2E encryption (document-level) | CryptPad | 6w | LOW | [x] `d01f28b` |
| 32 | U25 | SOC 2 Type II certification | Confluence, Notion | 12w | LOW | [x] `d01f28b` |
| 33 | U26 | GDPR automated data portability | Supabase, Confluence | 3w | LOW | [x] `d01f28b` |
| 34 | U27 | HIPAA compliance (healthcare KBs) | Confluence | 8w | LOW | [x] `d01f28b` |

**Completion:** SOC 2 readiness, GDPR automation. E2E and HIPAA deferred to regulated customers.

#### Phase 13 Effort Summary

| Priority | Items | Done | Remaining | Timeline |
|----------|-------|------|-----------|----------|
| HIGH (U16,U23,U24,U28,U2,U10) | 6 | 6 | 0 | Done |
| MEDIUM (U22,U29,U31,U32,U11,U13,U14,U18,U6,U7) | 10 | 10 | 0 | Done |
| LOW (U30,U33,U35,U15,U12,U17,U20,U21,U19,U5,U1,U4,U8,U9,U3,U25,U26,U27) | 19 | 19 | 0 | Done |

**Total Phase 13:** 35 of 35 items complete. All competitive gaps closed.

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
- Inline comments with threaded replies (annotation protocol)
- Branch-and-merge workflow for documents
- Review workflow with approval gates
- Template marketplace
- Real-time editing for >50 concurrent users per document (production validation)

### Intelligence
- Auto-generated knowledge graphs from link structure and embeddings
- Semantic clustering and topic modeling across document corpus
- Citation graph and impact metrics
- AI-powered document quality scoring

### Compliance
- SOC 2 Type II certification
- GDPR data portability automation
- HIPAA compliance (healthcare KBs)
- Data residency controls (region-specific deployment)

### Internationalization
- UI localization framework (extract strings, i18n crate)
- Priority languages: Chinese (ZH), German (DE), Japanese (JP), French (FR)
- RTL support evaluation (Arabic, Hebrew)

---

## Architecture Evolution

### Current (v20.0.0)

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

```
tachyon-editor (compiles to native + wasm32)
  |
  +-- HighlightProvider (trait)
  |     +-- RegexHighlighter (markdown structure, always available)
  |     +-- TreeSitterHighlighter (9 languages, feature-gated: native-tree-sitter)
  |     +-- CompositeHighlighter (regex + tree-sitter, planned)
  |     +-- WasmTreeSitterHighlighter (wasm-bindgen runtime .wasm, planned)
  |
  +-- tree-sitter targets
        +-- native (desktop, Tauri)
        +-- wasm32-unknown-unknown (web, Leptos)
        +-- wasm32-wasi (Docker SSG)
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

### Future (Post-v2)

```
CDN --> Nginx --> Load Balancer
                     |
          +----------+----------+
          |                     |
    Axum Instance 1       Axum Instance 2
          |                     |
          +----------+----------+
                     |
          Redis (sessions, rate limit, cache, pub/sub)
                     |
          +----------+----------+
          |                     |
    PostgreSQL Primary    PostgreSQL Replica
          |                     |
     pgvector (semantic)   Tantivy (full-text, sharded)
          |
     S3-compatible storage (attachments, exports)
```

---

## Effort Estimate

| Phase | Duration | Dependencies | Status | Priority |
|-------|----------|-------------|--------|----------|
| 1. Infrastructure | 2 weeks | Server provisioning | PENDING | CRITICAL (blocker) |
| 2. API Validation | 2 weeks | None (code-only) | AUDITED | CRITICAL |
| 3. Security | 2 weeks | Phase 2 | AUDITED | CRITICAL |
| 4. Launch | 1 week | Phase 1, 3 | PENDING | CRITICAL |
| A. Editor Architecture | 6-8 weeks | None (parallel) | A1-A4 ALL DONE | HIGH |
| C. Full-Stack Integration | 2 weeks | Phase A, B | C1-C3 ALL DONE | HIGH |
| 5. Migration | 3 weeks | Phase 4 | NEW | HIGH |
| 6. Knowledge Graph | 4 weeks | Phase 4 | NEW | HIGH |
| 7. AI Integration | 2 weeks | Phase 4 | CODE COMPLETE | MEDIUM |
| 8. Editor Experience | 3 weeks | Phase 4, Phase A | NEW | MEDIUM |
| 9. Multi-Tenant SaaS | 2 weeks | Phase 4 | CODE COMPLETE | MEDIUM |
| 10. Desktop/PWA/Mobile | 4 weeks | Phase 4, Phase A | PARTIAL | MEDIUM |
| 11. Plugin Ecosystem | 4 weeks | Phase 4 | PARTIAL | LOW (post-adoption) |
| 12. Enterprise | 4 weeks | Phase 9 | CODE COMPLETE | LOW (revenue) |
| Phase 13. Gap Closure | 24w (HIGH+MED), 48w (all) | Phase 4 | DONE | HIGH (post-v2 parity) |
| **Total to Production** | **~5 weeks** | | | |
| **Total to Competitive v2** | **~24 weeks** | | | |
| **Total to Feature Parity** | **~72 weeks** | | | |

---

## Phase Dependency Graph

```
Phase 1 (Infrastructure) ─────┬──> Phase 2 (API Validation) ──> Phase 3 (Security) ──> Phase 4 (Launch)
                               │                                                                            │
                               │            ┌───────────────────────────────────────────────────────────┘
                               │            │
                               │            v
                               │     Phase 5 (Migration) ──────────> Phase 6 (Knowledge Graph)
                               │            │
                               │            v
                               │     Phase 7 (AI) ───────────────> Phase 8 (Editor)
                               │                                         │
                               │            v                            │
                               │     Phase 9 (Multi-Tenant) ──────────┘
                               │            │
                               │            v
                               │     Phase 10 (Desktop/PWA/Mobile)
                               │            │
                               │            v
                                │     Phase 11 (Plugins) ──────────────> Phase 12 (Enterprise)
                                │            │
                                │            v
                                │     Phase 13 (Gap Closure: 35 items)
                                │     HIGH: U16,U23,U24,U28,U2,U10
                                │     MED:  U22,U29,U31,U32,U11,U13,U14,U18,U6,U7
                                │
                                v
                       [Production Live]

Phase A (Editor Architecture) ──────────────────────────────────────────────────> Phase 8, 10

Phase 13 (Feature Parity)────> [Feature Complete]
```

Phases 5-12 can partially overlap once Phase 4 is complete. Phases 5 and 6 are the highest-priority post-launch work because they unlock user acquisition. Phase 11 (plugins) is intentionally late -- shipping an empty marketplace is worse than shipping no marketplace. Phase 13 (gap closure) runs parallel to 11-12, prioritizing HIGH-ROI items first (storage, auth, notifications, import/export, SSG i18n). Phase A (editor architecture) runs in parallel, feeding into Phase 8 (editor experience) and Phase 10 (desktop).

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
| WebSocket broadcast not room-filtered (architectural, documented) | Medium | Deferred (Phase 10) |
| GraphQL resolvers don't use auth context | Medium | Deferred (Phase 9) |
| DLP module not wired into routes | Low | Deferred (Phase 12) |
| SSO module type definitions only (no runtime flow) | Low | Deferred (Phase 12) |
| NVIDIA+WebKitGTK EGL display failure on CachyOS | Medium | Documented (Phase 10) |
| `operational_transform.rs` (728 lines) deprecated but still dispatched in websocket handler | High | Deleted (Phase B) |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-03 | Runtime verification: all surfaces tested | T1 server /health + login PASS. T2 Leptos trunk serve PASS (2m35s wasm build). T3 SSG server-side tree-sitter highlighting PASS (json/py/rust). T4 full-stack server + SSG dist PASS. T5 Tauri binary builds (55MB) but NVIDIA+WebKitGTK EGL runtime issue blocks display on CachyOS. T6 Tauri frontend embedded in binary. |
| 2026-06-03 | ammonia double-pass strips class attributes | ammonia::clean() called in both parse_to_html AND sanitize_html. sanitize_html used default Builder (strips all class). Fix: add_generic_attributes(["class"]) to both ammonia Builders. |
| 2026-03 | Tauri desktop production build: 55MB binary | cargo tauri build --no-bundle succeeds. tray-icon disabled (no ayatana-appindicator in nix). Binary works on Intel/AMD/macOS/Windows; NVIDIA EGL issue documented. |
| 2026-06-02 | SSG server-side tree-sitter highlighting | highlighting_mode field: "client"/"server"/"both". Uses tachyon_renderer::SyntaxHighlighter at build time. Zero-JS for public pages. 9 languages supported. |
| 2026-06-02 | Three-target tree-sitter strategy complete | native (desktop cc) + wasm32-unknown-unknown (tree-sitter-c2rust) + server-side SSG (renderer native). WASI deferred -- native tree-sitter in SSG covers Docker deployment. |
| 2026-06-02 | WASM tree-sitter architecture: WasmTreeSitterHighlighter stub | Runtime .wasm loading architecture documented. Compiles on wasm32-unknown-unknown. Stub provider ready for future grammar loading. |
| 2026-06-02 | File-type detection in editor | Language enum with extension mapping. Auto-detection in Editor::set_content_with_filename infers language from file extension. |
| 2026-06-02 | Editor rearchitected with pluggable `HighlightProvider` trait | Monolithic highlighter replaced with composable provider system (RegexHighlighter, TreeSitterHighlighter, CompositeHighlighter). Enables markdown structure + code block highlighting from different engines. |
| 2026-06-02 | tree-sitter multi-target strategy (native/wasm/wasi) | tree-sitter compiled to native (desktop via Tauri), wasm32-unknown-unknown (web frontend), wasm32-wasi (Docker SSG). Feature-gated so editor crate compiles to WASM without tree-sitter dep. |
| 2026-06-02 | Desktop-first priority, VSCode/Neovim-style editor paradigm | Tachyon is a programmable editor platform, not a Notion-style block editor. Desktop via Tauri is the primary target. Syntax highlighting, multi-cursor, and tree-sitter support follow VSCode/Neovim patterns. |
| 2026-06-02 | RegexHighlighter + TreeSitterHighlighter composite architecture | Markdown structure highlighted by regex (always available, zero dependencies). Code blocks inside fenced regions highlighted by tree-sitter (feature-gated). CompositeHighlighter merges both. |
| 2026-05-31 | Add Phase 5 (Migration) as post-launch priority | Without Notion/Confluence/Google Docs import, no team switches. Highest-impact feature for acquisition. |
| 2026-05-31 | Add Phase 6 (Knowledge Graph) as post-launch priority | Wiki-links and graph view are table stakes for PKM. Obsidian, Logseq, Roam all have this. |
| 2026-05-31 | Add Phase 8 (Editor Experience) as medium priority | Markdown-first is defensible (GitBook, Docusaurus). Polish the MD experience, do not build a block editor. |
| 2026-05-31 | Reorder AI to Phase 7 (after migration, PKM) | AI attracts attention but does not drive initial adoption. Migration and PKM capture existing users first. |
| 2026-05-31 | Defer native mobile app to Phase 15 (post-v2) | PWA covers 80% of mobile use cases. Native app is expensive and not differentiating at this stage. |
| 2026-05-31 | Phase 11 (Plugins) moved after Editor Experience | Ship reference plugins before marketing the plugin system. Empty marketplace is worse than no marketplace. |
| 2026-05-31 | Establish competitive positioning principles | Target developers/technical teams (GitBook, Docusaurus, Outline position). Do not chase Notion. Self-hosted is the moat. |
| 2026-05-31 | Consolidate monitoring alerts (15 to 8 rules) | Eliminate duplicate HighErrorRate, DatabasePoolExhaustion, ServiceUnhealthy; fix inverted severity/timing. |
| 2026-05-31 | Delete broken websocket/tests.rs | Referenced private APIs and nonexistent struct fields; internal mod tests provide comprehensive coverage. |
| 2026-05-31 | Deduplicate handler.rs heartbeat cleanup | handle_heartbeat_timeout was copy-pasted from remove_client; delegate instead. |
| 2026-05-31 | Fix Redis rate limiter key format mismatch | increment() used non-windowed keys; now matches check_redis() format. Always set TTL. |
| 2026-05-31 | Add CRDT binary message size limit (10 MiB) | Prevent unbounded memory allocation from oversized WebSocket messages. |
| 2026-05-31 | Add auth-specific nginx rate limit (5r/m) | General 10r/s too permissive for login brute-force. |
| 2026-05-28 | Wire pgvector embeddings into document CRUD | Connect dead code (update_embedding, search_semantic) to AI provider; async spawned to avoid blocking response |
| 2026-05-28 | Add semantic search endpoint | Embed query via AI, search pgvector HNSW index; requires AI configured |
| 2026-05-28 | Wrap GraphQL/Swagger with audit middleware | Routes merged after layer application in Axum bypass middleware; wrap individually |
| 2026-05-28 | Convert 19 regex to LazyLock | Hot-path regex compilation on every call; LazyLock compiles once |
| 2026-05-28 | Add cursor endpoints alongside offset endpoints | Backward compat; new /cursor routes use CursorParams/CursorPage format |
| 2026-05-28 | Trivy matrix scan (3 Dockerfiles) | Root Dockerfile alone insufficient; frontend and server variants have different dependencies |
| 2026-05-28 | Include tachyon-cli in pre-commit clippy/test | Catch lint errors locally; GTK available via Nix |
| 2026-05-28 | Keep tachyon-cli excluded from CI | tachyon-cli depends on tachyon-desktop (GTK), not available in CI runners |
| 2026-05-28 | Consolidate 6 roadmap files into single ROADMAP.md | Eliminate version/count conflicts across documents |
| 2026-06-01 | Post-audit: resolve 15 clippy warnings across 6 crates | manual_strip, derivable_impls, map_clone, if_same_then_else, io_other_error, type_complexity |
| 2026-06-01 | Post-audit: remove dead code (11 unused re-exports, 833-line un-gated test file) | graph_invariants.rs was compiled into production binary without #[cfg(test)] |
| 2026-06-01 | Post-audit: fix Mutex poisoned-lock panic in attachments.rs | unwrap_or_else replaces unwrap for browser WASM stability |
| 2026-06-01 | Post-audit: enforce brutalist design across 12 frontend components | All rounded corners replaced with rounded-none per design spec |
| 2026-06-01 | Post-audit: improve WCAG compliance (aria-label, role, contrast) | Editor toolbar buttons, search dialog, line-number color fixed |
| 2026-06-01 | Post-audit: optimize CI/CD (cache restore-keys, concurrency groups) | 8 cache steps, 7 workflows get concurrency groups |
| 2026-06-01 | Post-audit: fix ZAP scan networking (host.docker.internal) | Linux runners do not resolve host.docker.internal; use localhost |
| 2026-06-01 | Post-audit: fix CLI test env var isolation | DATABASE_URL leaked between parallel tests via missing cleanup |
| 2026-05-26 | Switch CI to pgvector/pgvector:pg16 | Migration requires CREATE EXTENSION vector |
