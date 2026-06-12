# Gap Closure Roadmap

**Date:** 2026-06-12 | **Version:** 1.0 | **Author:** Tachyon Architecture Team

---

## Strategy

Close gaps in order of **impact per week of effort**. Focus on:
1. Migration tooling (user acquisition)
2. Table-stakes features (PDF, admonitions)
3. Developer experience (Vim, MDX)
4. Visual features (Canvas, graph interactivity)
5. Platform expansion (Mobile, i18n)

---

## Phase A: v1.0 Launch (6 weeks)

**Goal:** Close critical migration and documentation gaps for launch.

| # | Gap | Effort | Impact | Files | Status |
|---|-----|--------|--------|-------|--------|
| 1 | PDF export wiring | 1 week | High | `import-export/src/pdf_export.rs`, `server/src/routes/document/` | SCAFFOLDING -> DONE |
| 2 | Notion API import | 2 weeks | Critical | `import-export/src/notion.rs`, `server/src/routes/import.rs` | SCAFFOLDING -> DONE |
| 3 | Confluence API import | 2 weeks | Critical | `import-export/src/confluence.rs`, `server/src/routes/import.rs` | SCAFFOLDING -> DONE |
| 4 | Admonitions in editor | 1 week | Medium | `renderer/src/markdown.rs`, `frontend/src/components/editor_preview.rs` | SSG only -> EDITOR |

### Deliverables
- [ ] PDF export works for any document
- [ ] Notion OAuth import flow complete
- [ ] Confluence REST API import complete
- [ ] Admonitions render in editor preview
- [ ] All 1,504 tests pass
- [ ] E2E tests for new features

---

## Phase B: v1.1 Developer Experience (8 weeks)

**Goal:** Make Tachyon the best Markdown editor for developers.

| # | Gap | Effort | Impact | Files | Status |
|---|-----|--------|--------|-------|--------|
| 5 | Vim keybindings | 2 weeks | High | `editor/src/keybindings/` (new) | MISSING -> DONE |
| 6 | MDX support | 4 weeks | Medium | `renderer/src/mdx.rs` (new), `ssg/src/render.rs` | MISSING -> DONE |
| 7 | Admonition editor support | 1 week | Medium | `editor/src/editor.rs`, `renderer/src/markdown.rs` | PARTIAL -> DONE |
| 8 | Slash commands full wiring | 1 week | Medium | `frontend/src/components/slash_commands.rs` | PARTIAL -> DONE |

### Deliverables
- [ ] Vim mode with common motions (hjkl, w/b/e, dd, yy, p)
- [ ] Emacs mode with common bindings (C-n/p/f/b, C-a/e)
- [ ] MDX components render in SSG output
- [ ] Admonition syntax highlighting in editor
- [ ] All slash commands insert correct markdown

---

## Phase C: v1.2 Collaboration & Integration (10 weeks)

**Goal:** Best-in-class real-time collaboration and integrations.

| # | Gap | Effort | Impact | Files | Status |
|---|-----|--------|--------|-------|--------|
| 9 | Graph view interactivity | 2 weeks | Medium | `frontend/src/pages/graph.rs` | BASIC -> INTERACTIVE |
| 10 | Embed blocks | 1 week | Medium | `renderer/src/embeds.rs` (new), `frontend/src/components/embed_preview.rs` | MISSING -> DONE |
| 11 | TOC scroll sync | 1 week | Low | `frontend/src/components/table_of_contents.rs` | BASIC -> SYNCED |
| 12 | Webhook management UI | 1 week | Medium | `frontend/src/pages/webhooks.rs` (new) | MISSING -> DONE |
| 13 | i18n for frontend | 2 weeks | Medium | `frontend/src/i18n/` (extend) | SSG only -> FULL |
| 14 | File drag-and-drop | 1 week | Medium | `frontend/src/components/drop_zone.rs` | BASIC -> FULL |
| 15 | Custom CSS themes | 1 week | Low | `frontend/src/components/theme_toggle.rs` | COLORS -> CSS |
| 16 | Analytics dashboard | 2 weeks | Low | `frontend/src/pages/analytics.rs` (new), `server/src/routes/analytics.rs` | MISSING -> DONE |

### Deliverables
- [ ] Force-directed graph with zoom, pan, filter
- [ ] YouTube, Figma, Mermaid embeds render inline
- [ ] TOC highlights current heading on scroll
- [ ] Webhook create/edit/delete UI
- [ ] Frontend translated to 8 languages
- [ ] Image paste/upload inserts markdown
- [ ] Custom CSS theme editor
- [ ] Usage analytics dashboard

---

## Phase D: v2.0 Platform Expansion (14 weeks)

**Goal:** Mobile-first, visual knowledge management.

| # | Gap | Effort | Impact | Files | Status |
|---|-----|--------|--------|-------|--------|
| 17 | Canvas/whiteboard | 4 weeks | High | `frontend/src/components/canvas.rs` (new) | MISSING -> DONE |
| 18 | PWA offline support | 2 weeks | High | `frontend/public/sw.js`, `frontend/src/offline.rs` | BASIC -> FULL |
| 19 | Push notifications | 1 week | Medium | `server/src/routes/push.rs` (new), `frontend/src/push.rs` | MISSING -> DONE |
| 20 | Outliner mode | 4 weeks | Medium | `editor/src/outliner.rs` (new), `frontend/src/components/outliner.rs` | MISSING -> DONE |
| 21 | Flashcards/spaced repetition | 2 weeks | Low | `server/src/routes/flashcards.rs` (new) | MISSING -> DONE |
| 22 | PDF annotation | 1 week | Low | `frontend/src/components/pdf_viewer.rs` | MISSING -> BASIC |

### Deliverables
- [ ] Canvas with nodes, edges, sticky notes, images
- [ ] Offline document access via service worker
- [ ] Push notifications for mentions and comments
- [ ] Outliner mode with bullet hierarchy
- [ ] Basic flashcard system
- [ ] PDF viewer with highlight/annotation

---

## Phase E: v3.0 Enterprise & Scale (12 weeks)

**Goal:** Enterprise-grade features and massive scale.

| # | Gap | Effort | Impact | Files | Status |
|---|-----|--------|--------|-------|--------|
| 23 | SCIM provisioning | 2 weeks | Medium | `server/src/routes/scim.rs` | TYPES -> RUNTIME |
| 24 | SOC 2 automation | 4 weeks | Medium | `server/src/compliance/` | SCAFFOLDING -> DONE |
| 25 | E2E encryption | 3 weeks | Low | `server/src/e2e.rs`, `frontend/src/crypto.rs` | SCAFFOLDING -> DONE |
| 26 | Versioned documentation | 2 weeks | Low | `ssg/src/versioning.rs` | BASIC -> FULL |
| 27 | Blog system | 1 week | Low | `ssg/src/blog.rs` (new) | MISSING -> DONE |

### Deliverables
- [ ] SCIM 2.0 user provisioning from Okta/Azure AD
- [ ] Automated SOC 2 evidence collection
- [ ] Client-side E2E encryption for sensitive documents
- [ ] Version branches for documentation
- [ ] Blog with RSS, tags, categories

---

## Implementation Priority Matrix

```
                    HIGH IMPACT
                        |
    Phase A (v1.0)      |      Phase B (v1.1)
    PDF, Notion,        |      Vim, MDX,
    Confluence,         |      Admonitions,
    Admonitions         |      Slash cmds
                        |
  LOW EFFORT -----------+----------- HIGH EFFORT
                        |
    Phase C (v1.2)      |      Phase D (v2.0)
    Graph, Embeds,      |      Canvas, Offline,
    TOC, Webhooks,      |      Outliner,
    i18n, Themes        |      Flashcards
                        |
                    LOW IMPACT
```

---

## Resource Requirements

| Phase | Duration | Developer | Skills |
|-------|----------|-----------|--------|
| A (v1.0) | 6 weeks | 1 senior | Rust, API design, OAuth |
| B (v1.1) | 8 weeks | 1 senior + 1 junior | Editor internals, MDX parsing |
| C (v1.2) | 10 weeks | 1 senior + 1 frontend | Leptos, Canvas API, WebSocket |
| D (v2.0) | 14 weeks | 2 senior | Canvas, Service Workers, CRDT |
| E (v3.0) | 12 weeks | 1 senior + 1 security | SCIM, Encryption, Compliance |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Canvas implementation complexity | High | High | Start with simple node/edge, iterate |
| Vim keybinding scope creep | Medium | Medium | Implement only 20 most-used motions |
| MDX parser correctness | Medium | High | Use existing MDX test suite |
| PWA offline sync conflicts | High | High | CRDT handles conflicts, test thoroughly |
| SCIM protocol compliance | Low | High | Use existing SCIM library, test with Okta |

---

## Success Metrics

| Metric | v1.0 | v1.1 | v1.2 | v2.0 | v3.0 |
|--------|------|------|------|------|------|
| Features implemented | 107 | 111 | 119 | 125 | 130 |
| Test coverage | >90% | >92% | >94% | >95% | >95% |
| E2E tests | 15 | 20 | 30 | 40 | 50 |
| Competitor parity | 75% | 85% | 92% | 95% | 98% |
| Documentation pages | 20 | 30 | 40 | 50 | 60 |
