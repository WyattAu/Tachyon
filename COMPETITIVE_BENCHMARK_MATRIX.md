# Competitive Benchmark Matrix

**Date:** 2026-06-12 | **Version:** 1.0 | **Author:** Tachyon Architecture Team

---

## Executive Summary

Tachyon is compared against 10 competitors across 12 feature categories. Tachyon's unique position: **self-hosted, Rust-native, Markdown-first, real-time collaboration with SSG**. No other tool combines all four.

| Dimension | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|-----------|---------|----------|--------|------------|-----------|---------|---------|--------|------------|---------|-----------|
| **Self-Hosted** | Yes | Local | No | Static | Static | No | Yes | Local | Yes (DC) | Yes | Yes |
| **Open Source** | Yes | No | No | Yes | Yes | No | BSL | Yes | No | Yes | MIT |
| **Real-Time Collab** | Yes | No | Yes | No | No | Yes | Yes | No | Yes | Limited | No |
| **Markdown-First** | Yes | Yes | Partial | Yes | Yes | Yes | Yes | Yes | Partial | Yes | Yes |
| **SSG Built-In** | Yes | Publish ($) | No | Yes | Yes | No | No | No | No | No | No |
| **Plugin System** | WASM | 1000+ | Limited | Good | Good | Limited | Limited | Good | 1000+ | Limited | None |
| **AI Features** | Yes | Plugins | Yes | No | No | Yes | Yes | No | Yes (Rovo) | No | No |
| **Mobile App** | PWA | Yes | Yes | N/A | N/A | No | No | Yes | Yes | Yes | No |
| **Pricing** | Free | Free | $0-16/mo | Free | Free | $0-249/site | $0-249/mo | Free | $0-12/user | $0-12/mo | Free |

---

## Feature Comparison Matrix

### Core Editing

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| Markdown editor | [x] | [x] | Partial | [x] | [x] | [x] | [x] | [x] | Partial | [x] | [x] |
| Live preview | [x] | [x] | [x] | N/A | N/A | [x] | [x] | [x] | [x] | [x] | [x] |
| Code highlighting | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [ ] | [x] |
| Math/LaTeX | [x] | [x] | [x] | MDX | MDX | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Mermaid diagrams | [x] | Plugin | [x] | MDX | [x] | [x] | [ ] | [ ] | [x] | [ ] | [x] |
| Tables | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] |
| Task lists | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] |
| Footnotes | [x] | [x] | [ ] | [x] | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Split view | [x] | [x] | [x] | N/A | N/A | [x] | [x] | [x] | [x] | [x] | [ ] |
| Multi-cursor | [x] | [x] | [ ] | N/A | N/A | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Find/replace regex | [x] | [x] | [x] | N/A | N/A | [x] | [x] | [x] | [x] | [ ] | [x] |
| Auto-save | [x] | [x] | [x] | N/A | N/A | [x] | [x] | [x] | [x] | [x] | [x] |
| Version history | [x] | [x] | [x] | Git | Git | [x] | [x] | Git | [x] | [x] | [x] |
| Vim keybindings | [ ] | Plugin | [ ] | N/A | N/A | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Offline sync | [x] | [x] | [ ] | N/A | N/A | [ ] | [ ] | [x] | [ ] | [x] | [ ] |

### Knowledge Management

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| Wiki-links [[]] | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [x] | [ ] |
| Backlinks | [x] | [x] | [x] | [ ] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] |
| Graph view | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] |
| Tags | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] |
| Spaces/workspaces | [x] | [x] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [x] | [x] |
| Templates | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] |
| Daily notes | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] | [ ] |
| Canvas/whiteboard | [ ] | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [x] | [x] | [x] | [ ] |
| Block references | [x] | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [x] | [ ] |
| Trash/soft delete | [x] | [x] | [x] | Git | Git | [x] | [x] | [x] | [x] | [x] | [x] |

### Search & Discovery

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| Full-text search | [x] | [x] | [x] | Algolia | Algolia | [x] | [x] | [x] | [x] | [x] | [x] |
| Fuzzy search | [x] | [x] | [x] | Algolia | Algolia | [x] | [x] | [x] | [x] | [ ] | [x] |
| Semantic search | [x] | [ ] | [x] AI | [ ] | [ ] | [x] AI | [x] AI | [ ] | [x] Rovo | [ ] | [ ] |
| Search filters | [x] | [x] | [x] | [ ] | [ ] | [x] | [x] | [x] | [x] | [ ] | [x] |
| Saved searches | [x] | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |
| Autocomplete | [x] | [x] | [x] | [ ] | [ ] | [x] | [x] | [x] | [x] | [ ] | [ ] |

### Collaboration

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| Real-time co-editing | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | Limited | [ ] |
| Comments | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] | [ ] |
| Review/approval | [x] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] |
| Branch and merge | [x] | Git | [ ] | Git | Git | Git | [ ] | Git | [ ] | [ ] | Git |
| RBAC | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [x] | [x] |
| Teams/orgs | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [x] | [x] |
| Presence | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] | [ ] |
| Notifications | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] | [ ] |
| Audit log | [x] | [ ] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |

### Import & Export

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| Markdown import | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] |
| Obsidian import | [x] | N/A | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Notion import | [x] | [ ] | N/A | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Confluence import | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | N/A | [ ] | [ ] |
| DOCX import/export | [x] | Plugin | [x] | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [x] |
| PDF export | [x] | Plugin | [x] | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [x] |
| HTML export | [x] | Plugin | [x] | [x] | [x] | [x] | [x] | [ ] | [x] | [x] | [x] |
| SSG | [x] | Publish ($) | [ ] | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| CSV import | [x] | [ ] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] |

### AI & Intelligence

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| AI writing | [x] | Plugin | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] | [ ] |
| Auto-tagging | [x] | Plugin | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |
| RAG Q&A | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] | [ ] |
| Summarization | [x] | Plugin | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] | [ ] |
| Translation | [ ] | Plugin | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |

### Platform & Infrastructure

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| Web app | [x] WASM | [ ] | [x] | SSG | SSG | [x] | [x] | [ ] | [x] | [x] | [x] |
| Desktop app | [x] Tauri | [x] Electron | [ ] | [ ] | [ ] | [ ] | [ ] | [x] Electron | [ ] | [x] Electron | [ ] |
| Mobile responsive | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] | [x] |
| PWA | [x] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| REST API | [x] | Plugin | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [x] | [x] |
| GraphQL | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| WebSocket | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [ ] | [ ] |
| CLI | [x] | [ ] | [ ] | [x] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

### Security & Compliance

| Feature | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|---------|:-------:|:--------:|:------:|:----------:|:---------:|:-------:|:-------:|:------:|:----------:|:-------:|:---------:|
| JWT auth | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [x] | [x] |
| RBAC | [x] | [ ] | [x] | [ ] | [ ] | [x] | [x] | [ ] | [x] | [x] | [x] |
| SAML/SSO | [x] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] |
| LDAP | [x] | [ ] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |
| OIDC | [x] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] |
| MFA/TOTP | [x] | [ ] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |
| SCIM | [ ] | [ ] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |
| DLP scanning | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Audit logging | [x] | [ ] | [x] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] | [ ] |
| E2E encryption | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] | [x] | [ ] |
| GDPR | [x] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] |
| SOC 2 | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] | [x] | [ ] | [ ] |

### Pricing Comparison

| Tier | Tachyon | Obsidian | Notion | Docusaurus | Starlight | GitBook | Outline | Logseq | Confluence | Anytype | BookStack |
|------|---------|----------|--------|------------|-----------|---------|---------|--------|------------|---------|-----------|
| **Free** | Self-host | Personal | 1 user | Open source | Open source | 1 user | Self-host | Personal | 10 users | 1GB | Open source |
| **Paid** | N/A | Sync $4-5/mo | Plus $8.50/mo | N/A | N/A | $65/site | $10/mo | Sync $5/mo | $6/user/mo | $12/mo | N/A |
| **Enterprise** | N/A | $50/user/yr | Custom | N/A | N/A | $249/site | $249/mo | N/A | $12/user/mo | Custom | N/A |

---

## Tachyon Competitive Position

### Where Tachyon Wins
1. **Performance**: Rust-native, WASM frontend, sub-millisecond API latency
2. **Self-hosting**: Full control, no vendor lock-in, no per-user pricing
3. **Security**: Formal verification, RBAC, SAML/SSO, LDAP, OIDC, DLP, audit logging
4. **SSG + Collab**: Only tool that combines real-time collaboration with static site generation
5. **Import breadth**: Imports from Obsidian, Notion, Confluence, DOCX, CSV, Docusaurus
6. **Plugin system**: WASM sandbox (Wasmtime) vs JavaScript-only alternatives
7. **AI integration**: 3 providers (OpenAI, Anthropic, Ollama) with semantic search
8. **Dual license**: AGPL-3.0 (open source) + commercial license available

### Where Tachyon Loses
1. **Ecosystem maturity**: Obsidian has 1000+ plugins, Notion has massive integrations
2. **Mobile**: No native mobile apps (PWA only vs Obsidian/Notion/Logseq native apps)
3. **Offline-first**: Obsidian/Logseq/Anytype work fully offline; Tachyon needs server
4. **UX polish**: Notion/GitBook have more refined visual design
5. **Migration tooling**: No automated migration from Notion/Confluence (API-based)
6. **Graph visualization**: Obsidian/Logseq have more interactive graph views
7. **Canvas/whiteboard**: Missing (Notion, Obsidian, Logseq all have this)
8. **Outliner mode**: Logseq's signature feature, not in Tachyon

### Unique Value Proposition
> **Tachyon is the only self-hosted knowledge management system that combines real-time CRDT collaboration, Markdown-first editing, built-in static site generation, and enterprise-grade security (SAML/SSO, RBAC, DLP) in a single Rust-native platform.**

No other tool offers:
- Real-time collaboration + SSG in one platform
- WASM plugin sandbox (vs JavaScript-only)
- Formal verification of algorithms
- Sub-millisecond API latency
- Self-hosted with enterprise SSO/RBAC/DLP
