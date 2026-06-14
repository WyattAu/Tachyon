# Tachyon Competitive Gap Analysis & Closure Roadmap

**Document ID:** TACHYON-GAP-V1.0
**Date:** 2026-06-12
**Baseline:** v20.0.0 (Code Complete)
**Target:** v1.0.0 → v3.0.0

---

## 1. Feature Parity Matrix

### Legend

| Symbol | Meaning |
|--------|---------|
| ✓ | Implemented and working |
| ◐ | Partially implemented (scaffolding, types-only, or incomplete) |
| ✗ | Not implemented |
| N/A | Not applicable to this product's design |

### 1.1 Core Editing & Content

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| Markdown editor | ✓ | ✓ | ✗ (block) | ✓ | ✓ | ✓ | ✓ | ✗ (Atlassian) |
| Live preview (split-pane) | ✗ | ✓ | N/A | N/A | N/A | N/A | ✗ | N/A |
| WYSIWYG / block editor | ✗ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ |
| MDX support | ✗ | ✗ | N/A | ✓ | ✓ | ✗ | ✗ | N/A |
| Syntax highlighting | ✓ (tree-sitter) | ✓ (CodeMirror) | ✗ | ✓ (Prism) | ✓ (Prism) | ✓ (Shiki) | ✗ | ✗ |
| Math rendering (KaTeX) | ✓ | ✓ | ◐ | ✓ | ✓ | ◐ | ✓ | ✗ |
| Table editing | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ |
| Code blocks (fenced) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Embed blocks (YouTube, Figma) | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ |
| Admonitions / callouts | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Tabbed content | ✗ | ✗ | N/A | ✓ | ✗ | ✗ | ✗ | ✗ |
| Templates | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ |
| Slash commands | ✗ | ✓ | ✓ | N/A | N/A | N/A | ✓ | N/A |
| Drag-and-drop attachments | ✗ | ✓ | ✓ | N/A | N/A | N/A | ✗ | ✓ |

### 1.2 Collaboration & Real-Time

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| Real-time collaboration | ✓ (CRDT) | ◐ (plugin) | ✓ (OT) | N/A | ✓ (OT) | ✓ (CRDT) | ✗ | ✓ (OT) |
| Live cursors / presence | ✓ | ✗ | ✓ | N/A | ✓ | ✓ | ✗ | ✓ |
| Inline comments | ✗ | ✗ | ✓ | N/A | ✓ | ✓ | ✗ | ✓ |
| Threaded discussions | ✗ | ✗ | ✓ | N/A | ✓ | ✗ | ✗ | ✓ |
| Review / approval workflow | ✓ | ✗ | ✗ | N/A | ✓ | ✗ | ✗ | ✓ |
| Branch-and-merge | ✓ | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✗ |
| Version history (diff/restore) | ✓ | ✓ (git) | ✓ | ✗ | ✓ | ✗ | ✓ (git) | ✓ |
| Offline sync | ◐ (PWA cache) | ✓ | ◐ | N/A | ✗ | ✗ | ✓ | ✗ |
| Conflict resolution | ✓ (CRDT) | ✓ (git) | ✓ (OT) | N/A | ✓ (OT) | ✓ (CRDT) | ✓ (CRDT) | ✓ (OT) |

### 1.3 Knowledge Management

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| Backlinks | ✓ (API) | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Wiki-links `[[target]]` | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Graph view (visual) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| Graph query (structural) | ✓ | ◐ (Dataview) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Semantic search (AI) | ✓ (pgvector) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Tags / categories | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Folders / spaces | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Daily notes / journal | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| Block references (transclude) | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ |
| Properties / metadata | ✓ (frontmatter) | ✓ | ✓ | ✓ (frontmatter) | ✓ | ✓ | ✓ | ✓ |
| Canvas / whiteboards | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ |
| Flashcards / spaced repetition | ✗ | ✓ (plugin) | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| PDF annotation | ✗ | ✓ (plugin) | ✗ | N/A | ✗ | ✗ | ✓ | ✗ |

### 1.4 Search & Discovery

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| Full-text search | ✓ (Tantivy) | ✓ (SQLite FTS5) | ✓ (ElasticSearch) | ✓ (Algolia) | ✓ (Algolia) | ✓ (tsvector) | ✓ (SQLite FTS5) | ✓ (Lucene) |
| Fuzzy search | ✓ (pg_trgm) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Faceted search | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Search suggestions | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Search ranking (BM25) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Semantic search | ✓ (pgvector) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |

### 1.5 Platform & Deployment

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| Self-hosted | ✓ | ✓ (local) | ✗ | ✓ | ✗ | ✓ | ✓ | ✓ (DC) |
| Docker deployment | ✓ | N/A | N/A | ✓ | N/A | ✓ | ✓ | ✓ |
| Desktop app (native) | ✓ (Tauri) | ✓ (Electron) | ✗ | N/A | N/A | N/A | ✓ (Electron) | ✗ |
| PWA (mobile web) | ✓ | ✗ | ✓ | N/A | ✓ | ✗ | ✗ | ✓ |
| Native mobile app | ✗ | ✓ | ✓ | N/A | ✗ | ✗ | ✓ | ✓ |
| CLI | ✓ | ◐ (community) | ✗ | ✓ | ✗ | ✗ | ✗ | ◐ (community) |
| Single binary distribution | ✓ | ✓ | N/A | ✓ | N/A | N/A | ✓ | N/A |
| Static site generation | ✓ | ✓ (Publish, paid) | ✗ | ✓ | ✓ (Pages) | ✗ | ✗ | ✗ |
| SSG versioned docs | ✓ | ✗ | N/A | ✓ | ✓ | N/A | N/A | N/A |
| SSG i18n | ◐ (framework, no translations) | ✗ | N/A | ✓ | ✓ | N/A | N/A | N/A |

### 1.6 Security & Enterprise

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| RBAC (fine-grained) | ✓ | ✗ | ◐ | N/A | ◐ | ◐ | ✗ | ✓ |
| Custom roles | ✓ | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✓ |
| JWT auth | ✓ | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| OAuth2 (Google/GitHub) | ✓ | N/A | ✓ | N/A | ✓ | ◐ | N/A | ✓ |
| MFA (TOTP) | ✓ | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✓ |
| SSO (OIDC) | ✓ | ✗ | ✓ | N/A | ✓ | ◐ | N/A | ✓ |
| SSO (SAML) | ◐ (no XML-DSig) | ✗ | ✓ | N/A | ✓ | ✗ | N/A | ✓ |
| LDAP sync | ✓ | ✗ | ✗ | N/A | ✗ | ✗ | N/A | ✓ |
| SCIM provisioning | ✓ | ✗ | ✓ | N/A | ✗ | ✗ | N/A | ✗ |
| Guest access | ✓ | ✗ | ✓ | N/A | ✓ | ✗ | ✗ | ✗ |
| Audit logging | ✓ | ✓ (git) | ✓ | N/A | ✓ | ✗ | ✓ (git) | ✓ |
| DLP (content scanning) | ✓ | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✗ |
| E2E encryption | ◐ (scaffolding) | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✗ |
| SOC 2 / GDPR / HIPAA | ◐ (scaffolding) | ✗ | ✓ | N/A | ✓ | ✗ | N/A | ✓ |

### 1.7 API & Integration

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| REST API | ✓ | ✗ | ✓ | N/A | ✓ | ✓ | ✗ | ✓ |
| GraphQL API | ✓ | ✗ | ◐ | N/A | ✗ | ✓ | ✗ | ✗ |
| WebSocket API | ✓ | ✗ | ✗ | N/A | ✗ | ✓ | ✗ | ✗ |
| OpenAPI / Swagger UI | ✓ | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✗ |
| Webhooks | ◐ (planned) | ✗ | ✓ | N/A | ✓ | ✗ | ✗ | ✓ |
| Plugin system | ✓ (WASM sandbox) | ✓ (JS) | ✓ (REST) | ✓ (React) | ✓ (custom) | ✗ | ✓ (JS) | ✓ (Marketplace) |
| Plugin marketplace | ✗ | ✓ (2000+) | ✗ | ✗ | ✗ | N/A | ✗ | ✓ |
| Notification (Web Push) | ✓ | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✓ |
| Notification (Email) | ✓ | ✗ | ✓ | N/A | ✗ | ✗ | ✗ | ✓ |
| Notification (Slack/Discord) | ✓ | ✗ | ✓ | N/A | ✓ | ✗ | ✗ | ✓ |
| Git sync | ✓ | ✓ (native) | ✗ | ✓ | ✓ | ✗ | ✓ (native) | ✗ |
| Jira integration | ✗ | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✓ |
| Custom domains | ◐ (nginx) | N/A | N/A | N/A | ✓ | N/A | N/A | N/A |

### 1.8 Import & Export

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| Markdown import | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Markdown export | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| DOCX import | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ |
| DOCX export | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ |
| PDF export | ✗ | ✓ (plugin) | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ |
| HTML export | ✓ | ✓ | ✓ | N/A | ✗ | ✗ | ✗ | ✓ |
| CSV import | ✓ | ✓ | ✓ | N/A | ✗ | ✗ | ✗ | ✗ |
| JSON import/export | ✓ | ✗ | ✓ | N/A | ✗ | ✓ | ✗ | ✗ |
| Obsidian vault import | ✓ | N/A | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Notion import | ✗ | ✗ | N/A | ✗ | ✗ | ✗ | ✗ | ✗ |
| Confluence import | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | N/A |
| Google Docs import | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |

### 1.9 Analytics & Insights

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| Page views / analytics | ✗ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ |
| Search analytics | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| User activity logs | ✓ (audit) | ✗ | ✓ | N/A | ✓ | ✗ | ✗ | ✓ |
| Content quality metrics | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |

### 1.10 AI & Intelligence

| Feature | Tachyon | Obsidian | Notion | Docusaurus | GitBook | Outline | Logseq | Confluence |
|---------|---------|----------|--------|------------|---------|---------|--------|------------|
| AI writing assistant | ✓ (plugin) | ✓ (plugin) | ✓ | ✗ | ✗ | ✗ | ✓ (plugin) | ✗ |
| AI search / RAG | ✓ (pgvector) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Auto-tagging | ✓ (AI) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| AI-powered knowledge graph | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| AI document summarization | ✓ (plugin) | ✓ (plugin) | ✓ | ✗ | ✗ | ✗ | ✓ (plugin) | ✗ |

---

## 2. Gap Priority Scoring

### Scoring Dimensions

| Dimension | Scale | Description |
|-----------|-------|-------------|
| **User Impact** (U) | 1-5 | How many users want this? 5 = universal need, 1 = niche |
| **Competitive Pressure** (C) | 1-5 | Do competitors have it? 5 = all major competitors, 1 = none |
| **Implementation Effort** (E) | 1-5 | How hard to build? 5 = very hard, 1 = trivial |
| **Strategic Value** (S) | 1-5 | Does it differentiate? 5 = core differentiator, 1 = commodity |

### Priority Score = (U × 2 + C × 2 + S × 2) / E

Higher score = higher priority.

### 2.1 Critical Gaps (Score ≥ 6.0)

| # | Gap | U | C | E | S | Score | Notes |
|---|-----|---|---|---|---|-------|-------|
| G01 | Notion import | 5 | 5 | 3 | 5 | **10.0** | Migration is #1 blocker for adoption |
| G02 | Confluence import | 5 | 4 | 3 | 5 | **9.3** | Enterprise migration path |
| G03 | PDF export | 5 | 5 | 2 | 4 | **9.0** | Table stakes for documentation |
| G04 | Graph view (visual) | 5 | 4 | 4 | 5 | **8.5** | PKM table stakes, API exists |
| G05 | Inline comments | 5 | 5 | 3 | 4 | **8.7** | Collaboration table stakes |
| G06 | Templates | 4 | 5 | 2 | 3 | **8.0** | All competitors have this |
| G07 | Admonitions / callouts | 4 | 5 | 1 | 3 | **9.0** | Markdown ecosystem standard |
| G08 | Live preview (split-pane) | 5 | 3 | 4 | 4 | **7.3** | MD editors expect this |
| G09 | Google Docs import | 4 | 4 | 4 | 4 | **7.0** | Enterprise migration |
| G10 | Table editing | 5 | 5 | 3 | 3 | **7.7** | Universal need |

### 2.2 High Gaps (Score 4.0 - 5.9)

| # | Gap | U | C | E | S | Score | Notes |
|---|-----|---|---|---|---|-------|-------|
| G11 | Embed blocks (YouTube, Figma) | 4 | 5 | 3 | 3 | **5.7** | Content richness |
| G12 | Block references (transclude) | 4 | 4 | 4 | 5 | **5.5** | PKM power feature |
| G13 | Slash commands | 4 | 4 | 2 | 3 | **5.5** | Editor UX standard |
| G14 | Native mobile app | 5 | 5 | 5 | 3 | **5.0** | High effort, high impact |
| G15 | Plugin marketplace | 4 | 4 | 5 | 4 | **5.0** | Ecosystem growth |
| G16 | SSO (SAML complete) | 3 | 4 | 4 | 5 | **5.0** | Enterprise requirement |
| G17 | i18n UI localization | 4 | 4 | 3 | 3 | **5.0** | International adoption |
| G18 | Webhooks | 3 | 4 | 2 | 4 | **5.0** | Integration backbone |
| G19 | DOCX export | 3 | 4 | 2 | 3 | **5.0** | Enterprise documents |
| G20 | Custom domains (SSG) | 2 | 3 | 1 | 4 | **5.0** | SSG deployment |
| G21 | Canvas / whiteboards | 3 | 4 | 5 | 4 | **4.5** | Visual thinking |
| G22 | Page analytics | 3 | 3 | 3 | 4 | **4.3** | Content optimization |
| G23 | Drag-and-drop attachments | 4 | 4 | 2 | 2 | **5.0** | UX quality-of-life |
| G24 | Blog support (SSG) | 2 | 3 | 2 | 3 | **4.0** | Developer marketing |

### 2.3 Medium Gaps (Score 2.5 - 3.9)

| # | Gap | U | C | E | S | Score | Notes |
|---|-----|---|---|---|---|-------|-------|
| G25 | MDX support | 3 | 3 | 4 | 3 | **3.5** | Docusaurus overlap |
| G26 | Tabbed content | 2 | 2 | 2 | 2 | **3.0** | Docusaurus only |
| G27 | Outliner mode | 3 | 2 | 4 | 3 | **3.0** | Logseq only |
| G28 | Flashcards / SRS | 2 | 2 | 3 | 2 | **2.7** | Niche PKM |
| G29 | PDF annotation | 2 | 2 | 4 | 2 | **2.5** | Obsidian/Logseq |
| G30 | Jira integration | 2 | 2 | 3 | 3 | **3.0** | Confluence only |
| G31 | E2E encryption (complete) | 2 | 1 | 5 | 4 | **2.4** | CryptPad only |
| G32 | Analytics dashboard | 2 | 3 | 3 | 3 | **3.3** | Enterprise |
| G33 | AI knowledge graph | 2 | 1 | 5 | 5 | **2.4** | No competitor has this |

### 2.4 Low Gaps (Score < 2.5)

| # | Gap | U | C | E | S | Score | Notes |
|---|-----|---|---|---|---|-------|-------|
| G34 | WYSIWYG / block editor | 3 | 3 | 5 | 1 | **2.4** | Against Tachyon's MD-first philosophy |
| G35 | SOC 2 Type II | 1 | 3 | 5 | 3 | **2.0** | Long-term enterprise |
| G36 | LDAP directory sync | 1 | 2 | 3 | 3 | **2.0** | Enterprise niche |

---

## 3. Gap Closure Roadmap

### Phase A: v1.0 Launch (Current Priority)

**Rationale:** Must ship. No deployment = no users. These are prerequisites for any launch.

| # | Deliverable | Effort | Dependencies | Status |
|---|-------------|--------|-------------|--------|
| A1 | Provision staging + production VPS | 1 week | Cloud provider, domain | BLOCKING |
| A2 | Configure DNS + TLS | 3 days | A1 | BLOCKING |
| A3 | Set GitHub secrets for CD | 1 day | A1 | BLOCKING |
| A4 | Verify CD pipeline end-to-end | 2 days | A1-A3 | BLOCKING |
| A5 | k6 load tests against staging | 1 day | A4 | PENDING |
| A6 | OWASP ZAP scan against live | 1 day | A4 | PENDING |
| A7 | Fix WebSocket room filtering (T1) | 2 days | None | PENDING |
| A8 | Fix GraphQL auth bypass (T2) | 3 days | None | PENDING |
| A9 | Tag v1.0.0 | 1 day | A1-A8 | PENDING |

**Effort:** ~2 weeks
**Completion criterion:** `https://tachyon.dev/health` returns 200; v1.0.0 tagged.

---

### Phase B: v1.1 — Migration & Content Gaps (Post-Launch, Highest Priority)

**Rationale:** Without migration paths from Notion/Confluence, no team switches. Without PDF export and admonitions, documentation teams won't adopt.

| # | Gap | Deliverable | Effort | Score |
|---|-----|-------------|--------|-------|
| B1 | G01 | Notion import (API-based: pages, databases, properties, comments) | 2 weeks | 10.0 |
| B2 | G02 | Confluence import (REST API: pages, attachments, page tree, labels) | 2 weeks | 9.3 |
| B3 | G03 | PDF export (headless rendering, TOC, page numbers) | 1 week | 9.0 |
| B4 | G07 | Admonitions / callouts (`!!!note`, `!!!warning`, etc.) | 3 days | 9.0 |
| B5 | G05 | Inline comments (threaded, @mentions) | 1 week | 8.7 |
| B6 | G06 | Templates (create from template, template gallery) | 1 week | 8.0 |
| B7 | G10 | Table editing (Markdown table editor with toolbar) | 1 week | 7.7 |
| B8 | G08 | Split-pane live preview (edit MD / rendered HTML, synced scroll) | 2 weeks | 7.3 |
| B9 | G09 | Google Docs import (Drive API) | 1 week | 7.0 |

**Effort:** ~11 weeks (serial, 1 person)
**Completion criterion:** One-click migration from Notion, Confluence, Google Docs; PDF export; admonitions render; live preview works.

---

### Phase C: v1.2 — Knowledge Graph & Collaboration Polish

**Rationale:** Graph view and comments are table stakes for PKM. Embed blocks and slash commands complete the editor experience.

| # | Gap | Deliverable | Effort | Score |
|---|-----|-------------|--------|-------|
| C1 | G04 | Graph view (force-directed, zoomable, click-to-navigate) | 3 weeks | 8.5 |
| C2 | G11 | Embed blocks (YouTube, Figma, Mermaid rendered inline) | 2 weeks | 5.7 |
| C3 | G12 | Block references (transclude content, auto-update) | 2 weeks | 5.5 |
| C4 | G13 | Slash commands (`/` for heading, code, table, math, image) | 1 week | 5.5 |
| C5 | G17 | i18n UI localization (framework + 4 languages: EN, ZH, DE, JP) | 3 weeks | 5.0 |
| C6 | G18 | Webhooks (outbound HTTP callbacks on document events) | 1 week | 5.0 |
| C7 | G19 | DOCX export | 1 week | 5.0 |
| C8 | G20 | Custom domains (SSG config + nginx template) | 3 days | 5.0 |
| C9 | G23 | Drag-and-drop attachments (image upload → `![](url)`) | 1 week | 5.0 |

**Effort:** ~14 weeks (serial, 1 person)
**Completion criterion:** Graph view renders; slash commands work; embeds render; i18n framework operational.

---

### Phase D: v2.0 — Platform & Ecosystem

**Rationale:** Mobile access, plugin ecosystem, and enterprise features unlock scale. These are post-validation investments.

| # | Gap | Deliverable | Effort | Score |
|---|-----|-------------|--------|-------|
| D1 | G14 | Native mobile app (React Native or Flutter wrapper) | 8 weeks | 5.0 |
| D2 | G15 | Plugin marketplace (registry, signing, permissions, CLI) | 6 weeks | 5.0 |
| D3 | G16 | SAML complete (XML-DSig validation, SP metadata) | 2 weeks | 5.0 |
| D4 | G22 | Page analytics (views, search terms, popular docs) | 2 weeks | 4.3 |
| D5 | G21 | Canvas / whiteboards | 6 weeks | 4.5 |
| D6 | G24 | Blog support (SSG: post listings, RSS, date-based URLs) | 1 week | 4.0 |
| D7 | G32 | Analytics dashboard (admin: usage, growth, content health) | 2 weeks | 3.3 |

**Effort:** ~27 weeks (serial, 1 person)
**Completion criterion:** Mobile app in app stores; plugin registry live; SAML validated.

---

### Phase E: v3.0 — Differentiation & Advanced Features

**Rationale:** These features don't exist in competitors. Building them creates unique value.

| # | Gap | Deliverable | Effort | Score |
|---|-----|-------------|--------|-------|
| E1 | G33 | AI knowledge graph (auto-generate links from embeddings) | 4 weeks | 2.4 |
| E2 | G25 | MDX support (custom components in Markdown) | 3 weeks | 3.5 |
| E3 | G27 | Outliner mode (bullet-based editing, Logseq-style) | 4 weeks | 3.0 |
| E4 | G28 | Flashcards / spaced repetition (from document blocks) | 3 weeks | 2.7 |
| E5 | G29 | PDF annotation (highlight, comment, draw on PDFs) | 4 weeks | 2.5 |
| E6 | G31 | E2E encryption (complete implementation) | 6 weeks | 2.4 |
| E7 | G35 | SOC 2 Type II documentation | 4 weeks | 2.0 |

**Effort:** ~28 weeks (serial, 1 person)
**Completion criterion:** AI graph generates links; MDX renders; outliner mode works.

---

## 4. Competitive Positioning

### Where Tachyon Wins

| Dimension | Tachyon Advantage | Competitor Weakness |
|-----------|-------------------|---------------------|
| **Performance** | Rust-native, SIMD markdown, sub-15ms render, 6.4μs health check | Notion (Node.js), Confluence (Java), Obsidian (Electron) all have GC overhead |
| **Self-hosting** | Docker, Nix, single binary, Apache 2.0 | Notion (SaaS-only), GitBook (SaaS-only), Confluence ($$) |
| **Security** | WASM sandbox, TLA+/Lean4 proofs, DLP, OWASP ZAP, mutation testing | No competitor publishes formal correctness proofs |
| **SSG + Collaboration** | Only platform combining CRDT editing with built-in SSG | Docusaurus (SSG only), Notion (collab only), GitBook (hosted only) |
| **Auth / SSO** | JWT + OAuth2 + TOTP MFA + OIDC + SAML + LDAP — all self-hosted | Most require SaaS for SSO |
| **Semantic search** | pgvector + BM25 + pg_trgm (three-tier search) | Obsidian (local only), Logseq (no search), Outline (tsvector only) |
| **Knowledge graph** | Structural queries (BFS, shortest path, connected components) + semantic | Only API-level; no frontend visualization yet |
| **DLP** | Built-in content scanning (CC, SSN, API keys) at document create | Enterprise-only or absent in competitors |
| **CRDT collaboration** | Yrs (Y.js-compatible), server-side state, offline queue | Obsidian (plugin-dependent), Logseq (no collab), Confluence (OT) |

### Where Tachyon Loses

| Dimension | Tachyon Gap | Competitor Advantage |
|-----------|-------------|---------------------|
| **Ecosystem maturity** | Zero community plugins, no users, pre-production | Obsidian (2000+ plugins), Notion (millions of users) |
| **Mobile** | PWA only, no native app | Notion, Obsidian, Logseq have native iOS/Android |
| **Offline / local-first** | PWA caches assets; documents need server | Obsidian (file-based), Logseq (local-first), Anytype (P2P CRDT) |
| **UX polish** | Markdown-only, no WYSIWYG, no live preview | Notion (block editor), Confluence (WYSIWYG), GitBook (rich text) |
| **Migration** | No Notion/Confluence/Google Docs import | Outline, Wiki.js, BookStack all support these |
| **Graph visualization** | API exists; no frontend | Obsidian, Logseq have native graph views |
| **Comments / discussions** | Not implemented | All collaborative tools have inline comments |
| **Templates** | Not implemented | Universal feature across all competitors |
| **Analytics** | Audit log only; no page views or content metrics | Notion, GitBook, Confluence have analytics dashboards |
| **Market awareness** | New project, zero GitHub stars trajectory | Established brands with communities |

### Unique Value Proposition

**Tachyon is the only self-hosted, Rust-native knowledge management platform that combines real-time CRDT collaboration with a built-in static site generator, WASM plugin sandbox, and semantic search — all with formal correctness proofs.**

Positioning statement:

> *"Tachyon is for teams that want Notion's collaboration, GitBook's documentation publishing, and Obsidian's knowledge graph — without the SaaS lock-in, JavaScript overhead, or vendor dependency. Built in Rust. Self-hosted. Your data, your infrastructure, your rules."*

### Target Segments

| Segment | Why Tachyon | Migration Path |
|---------|-------------|----------------|
| **Developer teams** | Rust performance, Git integration, SSG, API-first | From GitBook, Docusaurus |
| **Security-conscious orgs** | Self-hosted, WASM sandbox, DLP, formal verification | From Confluence, Notion |
| **PKM enthusiasts** | Knowledge graph, bidirectional links, daily notes | From Obsidian, Logseq |
| **Open-source projects** | Apache 2.0, free, self-hosted, community-driven | From GitBook, Docusaurus |
| **Enterprise (regulated)** | RBAC, SSO, audit logging, SOC 2 prep, HIPAA scaffolding | From Confluence |

---

## 5. Effort Summary

| Phase | Version | Gaps Closed | Duration | Dependencies |
|-------|---------|-------------|----------|-------------|
| A | v1.0 | Infrastructure + security fixes | 2 weeks | Cloud provider |
| B | v1.1 | 9 gaps (migration, PDF, editor UX) | 11 weeks | Phase A |
| C | v1.2 | 9 gaps (graph, i18n, embeds) | 14 weeks | Phase B |
| D | v2.0 | 7 gaps (mobile, plugins, enterprise) | 27 weeks | Phase C |
| E | v3.0 | 7 gaps (AI graph, MDX, outliner) | 28 weeks | Phase D |
| **Total** | | **32 gaps** | **~82 weeks** | |

### Critical Path

```
Phase A (2 weeks) → Phase B (11 weeks) → Phase C (14 weeks) → Phase D (27 weeks) → Phase E (28 weeks)
```

Phase B (Migration) is the single highest-impact investment. Without Notion/Confluence import, no team switches. Without PDF export, documentation teams won't adopt. These two features alone (G01, G03) justify the entire Phase B effort.

---

## 6. Recommendations

### Immediate (Next 2 Weeks)
1. Complete infrastructure provisioning (Phase A)
2. Tag v1.0.0
3. Begin Notion import (B1) — highest-impact gap

### Short-Term (Months 1-3)
4. Ship PDF export (B3)
5. Ship admonitions (B4)
6. Ship inline comments (B5)
7. Ship live preview (B8)

### Medium-Term (Months 3-6)
8. Ship graph view (C1)
9. Ship i18n (C5)
10. Ship slash commands (C4)

### Long-Term (Months 6-12)
11. Native mobile app (D1)
12. Plugin marketplace (D2)
13. AI knowledge graph (E1) — unique differentiator

---

## Appendix: Gap-to-Phase Mapping

| Gap ID | Description | Phase | Priority Score |
|--------|-------------|-------|----------------|
| G01 | Notion import | B | 10.0 |
| G02 | Confluence import | B | 9.3 |
| G03 | PDF export | B | 9.0 |
| G07 | Admonitions / callouts | B | 9.0 |
| G05 | Inline comments | B | 8.7 |
| G04 | Graph view (visual) | C | 8.5 |
| G06 | Templates | B | 8.0 |
| G10 | Table editing | B | 7.7 |
| G08 | Live preview (split-pane) | B | 7.3 |
| G09 | Google Docs import | B | 7.0 |
| G11 | Embed blocks | C | 5.7 |
| G12 | Block references | C | 5.5 |
| G13 | Slash commands | C | 5.5 |
| G14 | Native mobile app | D | 5.0 |
| G15 | Plugin marketplace | D | 5.0 |
| G16 | SSO (SAML complete) | D | 5.0 |
| G17 | i18n UI localization | C | 5.0 |
| G18 | Webhooks | C | 5.0 |
| G19 | DOCX export | C | 5.0 |
| G20 | Custom domains (SSG) | C | 5.0 |
| G23 | Drag-and-drop attachments | C | 5.0 |
| G21 | Canvas / whiteboards | D | 4.5 |
| G22 | Page analytics | D | 4.3 |
| G24 | Blog support (SSG) | D | 4.0 |
| G25 | MDX support | E | 3.5 |
| G32 | Analytics dashboard | D | 3.3 |
| G26 | Tabbed content | E | 3.0 |
| G27 | Outliner mode | E | 3.0 |
| G30 | Jira integration | E | 3.0 |
| G28 | Flashcards / SRS | E | 2.7 |
| G29 | PDF annotation | E | 2.5 |
| G34 | WYSIWYG / block editor | E | 2.4 |
| G31 | E2E encryption (complete) | E | 2.4 |
| G33 | AI knowledge graph | E | 2.4 |
| G35 | SOC 2 Type II | E | 2.0 |
| G36 | LDAP directory sync | E | 2.0 |

---

**Document Status:** Complete
**Next Review:** After v1.0.0 tag
**Owner:** Tachyon Core Team
