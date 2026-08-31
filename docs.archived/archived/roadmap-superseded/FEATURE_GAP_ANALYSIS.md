# Feature Gap Analysis

**Date:** 2026-06-12 | **Version:** 1.0 | **Author:** Tachyon Architecture Team

---

## Current State

| Metric | Value |
|--------|-------|
| Features Implemented | 103 |
| Features Scaffolding | 13 |
| Features Missing | 2 |
| Total Features Audited | 118 |
| Implementation Rate | 87% |
| Test Coverage | 1,504 tests passing |
| E2E Tests | 12/12 passing |

---

## Gap Priority Scoring

Each gap is scored on 4 dimensions (1-5):

| Dimension | Description |
|-----------|-------------|
| **User Impact** | How many users need this? (1=rare, 5=universal) |
| **Competitive Pressure** | Do key competitors have it? (1=nobody, 5=all) |
| **Implementation Effort** | How hard to build? (1=trivial, 5=months) |
| **Strategic Value** | Does it differentiate us? (1=commodity, 5=unique) |

**Gap Score** = (User Impact + Competitive Pressure + Strategic Value) / Implementation Effort

---

## Critical Gaps (Score >= 8.0)

### 1. Notion API Import (Score: 10.0)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 5 | #1 migration path for teams switching from Notion |
| Competitive Pressure | 5 | Every competitor offers Notion import |
| Implementation Effort | 2 | Notion API is well-documented, OAuth flow standard |
| Strategic Value | 5 | Enables user acquisition from largest competitor |

**Current State:** SCAFFOLDING (ZIP export parser only, no API-based import)
**Gap:** Need OAuth flow + API pagination + database property mapping
**Effort:** 2 weeks
**Files:** `crates/import-export/src/notion.rs`, `crates/server/src/routes/import.rs`

### 2. Confluence REST API Import (Score: 9.3)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 5 | Enterprise teams migrating from Confluence |
| Competitive Pressure | 5 | Enterprise standard |
| Implementation Effort | 2 | REST API + XML parsing already exists |
| Strategic Value | 4 | Enterprise adoption path |

**Current State:** SCAFFOLDING (XML export parser only)
**Gap:** Need REST API client + pagination + space/page tree import
**Effort:** 2 weeks
**Files:** `crates/import-export/src/confluence.rs`

### 3. PDF Export (Score: 9.0)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 5 | Table stakes for documentation |
| Competitive Pressure | 5 | Every competitor has PDF export |
| Implementation Effort | 2 | PDF crate exists, just needs wiring |
| Strategic Value | 4 | Required for compliance/legal use cases |

**Current State:** SCAFFOLDING (feature-gated, untested)
**Gap:** Need to wire pdf_export to document routes, test with real documents
**Effort:** 1 week
**Files:** `crates/import-export/src/pdf_export.rs`

### 4. Canvas/Whiteboard (Score: 8.0)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 4 | Visual thinkers need spatial organization |
| Competitive Pressure | 5 | Notion, Obsidian, Logseq, Confluence all have it |
| Implementation Effort | 3 | Complex: rendering, persistence, collaboration |
| Strategic Value | 4 | Visual knowledge organization is trending |

**Current State:** MISSING
**Gap:** Need canvas component, node/edge system, real-time sync
**Effort:** 4 weeks
**Files:** New `crates/frontend/src/components/canvas.rs`

---

## High Priority Gaps (Score 5.0-7.9)

### 5. Mobile Native Apps (Score: 7.5)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 5 | Mobile access is required for any knowledge tool |
| Competitive Pressure | 5 | Obsidian, Notion, Logseq, Confluence all have mobile |
| Implementation Effort | 3 | React Native or Flutter, or enhance PWA |
| Strategic Value | 3 | Table stakes, not differentiating |

**Current State:** PWA (responsive web)
**Gap:** Native apps with offline support, push notifications
**Effort:** 8 weeks
**Recommendation:** Enhance PWA first (offline cache, push notifications), native apps later

### 6. Vim/Emacs Keybindings (Score: 7.0)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 4 | Developer audience expects vim/emacs |
| Competitive Pressure | 4 | Obsidian has vim mode, most editors support it |
| Implementation Effort | 2 | Key mapping layer, not core editor rewrite |
| Strategic Value | 3 | Developer productivity |

**Current State:** MISSING
**Gap:** Need vim emulation layer for editor
**Effort:** 2 weeks
**Files:** `crates/editor/src/keybindings/` (new module)

### 7. Admonitions/Callouts (Score: 6.5)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 4 | Common in technical documentation |
| Competitive Pressure | 4 | Docusaurus, Starlight, GitBook all have admonitions |
| Implementation Effort | 2 | Markdown extension, CSS styling |
| Strategic Value | 3 | Documentation quality |

**Current State:** SSG has admonition rendering, editor does not
**Gap:** Need editor support for admonition syntax, preview rendering
**Effort:** 1 week
**Files:** `crates/renderer/src/markdown.rs`, `crates/frontend/src/components/editor_preview.rs`

### 8. MDX Support (Score: 6.0)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 3 | Technical documentation teams |
| Competitive Pressure | 4 | Docusaurus, Starlight are MDX-native |
| Implementation Effort | 3 | Complex: JSX parsing, component embedding |
| Strategic Value | 3 | Documentation platform positioning |

**Current State:** MISSING
**Gap:** Need MDX parser, JSX component rendering, interactive docs
**Effort:** 4 weeks
**Files:** `crates/renderer/src/mdx.rs` (new module)

### 9. Webhook System Enhancement (Score: 5.5)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 3 | Integration with external tools |
| Competitive Pressure | 4 | Notion, GitBook, Confluence have webhooks |
| Implementation Effort | 2 | Webhook delivery exists, needs UI and more events |
| Strategic Value | 3 | Integration ecosystem |

**Current State:** IMPLEMENTED (webhook delivery with HMAC signatures)
**Gap:** Need webhook management UI, more event types, retry logic
**Effort:** 1 week
**Files:** `crates/server/src/routes/webhook.rs`, `crates/frontend/src/pages/webhooks.rs`

### 10. i18n for UI (Score: 5.0)

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| User Impact | 4 | International teams |
| Competitive Pressure | 4 | Docusaurus, Starlight, Confluence have i18n |
| Implementation Effort | 2 | Translation strings, locale detection |
| Strategic Value | 3 | Global market reach |

**Current State:** SSG has i18n (8 languages), frontend does not
**Gap:** Need frontend i18n framework, translation files, locale switching
**Effort:** 2 weeks
**Files:** `crates/frontend/src/i18n/` (exists for SSG, extend to frontend)

---

## Medium Priority Gaps (Score 3.0-4.9)

| # | Gap | Score | Effort | Rationale |
|---|-----|-------|--------|-----------|
| 11 | Graph view interactivity | 4.5 | 2 weeks | Obsidian/Logseq have force-directed, zoom, filter |
| 12 | Embed blocks (YouTube, Figma) | 4.5 | 1 week | Rich content embedding |
| 13 | Slash commands full wiring | 4.0 | 1 week | Commands exist but need editor integration |
| 14 | TOC sidebar scroll sync | 4.0 | 1 week | Highlight current heading |
| 15 | File drag-and-drop upload | 4.0 | 1 week | Image upload to editor |
| 16 | Outliner mode | 3.5 | 4 weeks | Logseq signature feature |
| 17 | Flashcards/spaced repetition | 3.5 | 2 weeks | Logseq feature, learning use case |
| 18 | PDF annotation | 3.5 | 2 weeks | Logseq feature, research use case |
| 19 | Custom CSS themes | 3.5 | 1 week | Branding beyond colors |
| 20 | Analytics dashboard | 3.5 | 2 weeks | Usage metrics for admins |

---

## Low Priority Gaps (Score < 3.0)

| # | Gap | Score | Effort | Rationale |
|---|-----|-------|--------|-----------|
| 21 | Kanban boards | 2.5 | 3 weeks | Notion feature, project management |
| 22 | Calendar view | 2.5 | 2 weeks | Notion feature, scheduling |
| 23 | Form builder | 2.0 | 3 weeks | Notion feature, data collection |
| 24 | API playground | 2.0 | 1 week | GitBook feature, developer docs |
| 25 | Custom domains | 2.0 | 1 week | SaaS feature, reverse proxy config |
| 26 | Versioned documentation | 2.0 | 2 weeks | Docusaurus feature, SSG extension |
| 27 | Blog system | 1.5 | 2 weeks | Docusaurus feature, SSG extension |
| 28 | Changelog automation | 1.5 | 1 week | Docusaurus feature, git-based |
| 29 | Tabbed content | 1.5 | 1 week | Docusaurus feature, markdown extension |
| 30 | Syntax highlighting themes | 1.5 | 1 week | Customization, CSS variables |

---

## Summary

| Priority | Count | Total Effort | Key Gaps |
|----------|-------|-------------|----------|
| Critical (>= 8.0) | 4 | 9 weeks | Notion import, Confluence import, PDF export, Canvas |
| High (5.0-7.9) | 6 | 18 weeks | Mobile, Vim, Admonitions, MDX, Webhooks, i18n |
| Medium (3.0-4.9) | 10 | 17 weeks | Graph interactivity, Embeds, Slash commands, TOC |
| Low (< 3.0) | 10 | 14 weeks | Kanban, Calendar, Forms, Blog |
| **Total** | **30** | **58 weeks** | |

### Recommended v1.0 Scope

For v1.0 launch, close only Critical gaps that don't require new UI:
1. **PDF export** (1 week) - wire existing code
2. **Notion API import** (2 weeks) - OAuth + API client
3. **Confluence API import** (2 weeks) - REST API client
4. **Admonitions** (1 week) - markdown extension

**Total v1.0 gap closure: 6 weeks**

### Post-Launch Roadmap

| Version | Gaps Closed | Effort | Timeline |
|---------|-------------|--------|----------|
| v1.0 | #1-4 (Critical, no new UI) | 6 weeks | Now |
| v1.1 | #5-10 (High priority) | 18 weeks | +4 months |
| v1.2 | #11-20 (Medium priority) | 17 weeks | +8 months |
| v2.0 | #21-30 (Low priority) + Canvas | 21 weeks | +13 months |
