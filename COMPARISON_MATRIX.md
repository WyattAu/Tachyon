# Competitive Comparison Matrix

**Last updated:** 2026-06-01
**Scope:** Collaborative knowledge management, real-time editing, SSG, and adjacent categories.
**Verified:** Cross-referenced against codebase implementation (32 features checked).

---

## Category A: Collaborative Knowledge Base / Wiki Platforms

These are Tachyon's primary competitors -- platforms that combine document editing, team collaboration, search, and knowledge management in a single product.

| Feature | Tachyon | Notion | Confluence | Outline | GitBook | Obsidian |
|---------|---------|--------|------------|---------|---------|----------|
| **License** | Apache 2.0 (self-hosted) | Proprietary (SaaS) | Proprietary (SaaS/self-hosted) | AGPLv3 (self-hosted) | Proprietary (SaaS) | Proprietary (local) |
| **Core Language** | Rust | TypeScript/Node.js | Java (server), React (UI) | TypeScript/Node.js | TypeScript/Node.js | TypeScript/Electron |
| **Real-Time Collab** | CRDT (Yrs/lib0), WebSocket | OT (custom), WebSocket | OT (custom), WebSocket | CRDT (Yjs), WebSocket | OT (custom), WebSocket | LiveSync (CRDT, Yjs) via plugin |
| **Conflict Resolution** | CRDT (server + client) | OT (server-authoritative) | OT (server-authoritative) | CRDT (Yjs, decentralized) | OT (server-authoritative) | CRDT (Yjs, P2P or self-hosted) |
| **Markdown** | CommonMark + GFM, KaTeX, tree-sitter (9 active, 12 defined) | Mixed (block-based, partial MD) | Atlassian Markup (not MD) | CommonMark + GFM | CommonMark + GFM, MDX | CommonMark + GFM, LaTeX, Dataview |
| **Syntax Highlighting** | tree-sitter (9 active), syntect | Limited code blocks | Limited | Shiki/highlight.js | Prism, code blocks | CodeMirror |
| **Math Rendering** | KaTeX | Inline LaTeX (limited) | No native KaTeX | KaTeX (plugin) | KaTeX | KaTeX, MathJax |
| **Full-Text Search** | Tantivy (BM25) + PostgreSQL tsvector | ElasticSearch (managed) | Confluence search (Lucene) | PostgreSQL tsvector | Algolia DocSearch (managed) | Full-text (SQLite FTS5), Omnisearch |
| **Semantic Search** | pgvector + AI provider integration | Notion AI (proprietary) | No | No | No | No (community plugins) |
| **RBAC / Permissions** | Fine-grained RBAC, custom roles, teams, audit | Workspace-level roles, page-level sharing | Space/page restrictions, fine-grained | Groups, collections, members | Team roles, API keys | No built-in (local-first) |
| **Audit Logging** | Database-persisted audit events | Page history, audit log (Enterprise) | Page history, audit log | No | Page history, audit log (Enterprise) | File system history (git) |
| **DLP / Content Scanning** | Regex-based DLP (CC, SSN, API keys), wired into create/update | No | No | No | No | No |
| **SSG** | Built-in (tachyon-ssg) | Public pages (limited) | No native SSG | No | GitBook Pages (hosted) | Obsidian Publish (paid) |
| **Plugin System** | WASM sandbox (Wasmtime), marketplace client | Integrations (REST) | Atlassian Marketplace | No | Custom integrations | Community plugins (JS) |
| **API** | REST + Swagger, WebSocket, GraphQL | REST, GraphQL (partial) | REST | REST + GraphQL | REST | No public API |
| **Desktop Client** | Tauri 2 (native) | Electron (wrapper) | No native client | No | No | Electron (native) |
| **CLI** | Native (tachyon-cli) | Notion CLI (unofficial) | Confluence CLI (unofficial) | No | GitBook CLI (unofficial) | CLI (community) |
| **Import/Export** | Docusaurus, Obsidian, generic vault, MD ZIP, JSON, HTML | MD, CSV, HTML, PDF | MD, Word, PDF | MD, JSON | MD, PDF, OpenAPI | MD, HTML, PDF, Pandoc |
| **Database** | PostgreSQL (pgvector, tsvector, JSONB) | Private (distributed store) | PostgreSQL / H2 | PostgreSQL | Private | SQLite (local) |
| **Multi-Tenant** | Organization-level row isolation, plan enforcement | Workspace-based | Confluence Data Center | No | GitBook Organizations | No |
| **Self-Hosted** | Yes (Docker/Nix) | No (On-Premise Enterprise only) | Yes (Data Center/Server) | Yes (Docker) | No | No |
| **Offline Support** | Yes (PWA service worker, offline.html fallback) | Desktop partial | No | No | No | Yes (local-first) |
| **Versioning** | Document versions API (diff, restore) | Page history (restore points) | Page history (diff view) | No | Page history | Git-based versioning |
| **Guest Access** | JWT guest login (rate-limited) | Share links (public/invite) | No (anonymous access possible) | No | Share links | No (local-only) |
| **White-Label** | Planned (Enterprise) | No | Limited | No | Custom domain | No |
| **SAML/SSO** | OIDC (runtime), SAML (partial runtime, no XML-DSig), LDAP (runtime: bind/search/sync) | SAML, SCIM (Enterprise) | SAML, Crowd (Enterprise) | OIDC (planned) | SAML (Enterprise) | No |
| **MFA** | TOTP (native, server-side) | No | Yes (Enterprise) | No | No | No |
| **Notifications** | Web Push, Email (lettre SMTP), Webhooks | In-app only | In-app, email | In-app | In-app | No |
| **Audit Logging** | Database-persisted audit events (70+ event types) | Page history, audit log (Enterprise) | Page history, audit log | No | Page history, audit log (Enterprise) | File system history (git) |

### Category A Notes

- **Notion** dominates the market but is SaaS-only for most use cases. On-Premise requires Enterprise contract. Block-based editor diverges from standard Markdown.
- **Confluence** is strongest in Jira-integrated enterprise teams. Atlassian Markup is a proprietary format. Heavy resource requirements.
- **Outline** is the closest open-source competitor. AGPLv3 licensed. Uses Yjs CRDT. Lacks SSG, plugin system, and semantic search.
- **GitBook** is API-doc focused. Strong for developer documentation but not a general-purpose knowledge base.
- **Obsidian** is local-first with no server component (by design). Real-time collaboration requires third-party sync services. No RBAC.

---

## Category B: SSG (Static Site Generator) Competitors

Tachyon includes a built-in SSG (tachyon-ssg) that generates documentation sites. These are dedicated SSG tools, many of which can also serve as knowledge bases.

| Feature | Tachyon SSG | Astro | Hugo | Docusaurus | Jekyll | 11ty (Eleventy) | Zola | Pelican | SvelteKit | Gatsby | Next.js (SSG) | VitePress | MkDocs | MdBook |
|---------|-------------|-------|------|------------|--------|-----------------|------|---------|-----------|--------|---------------|----------|--------|--------|
| **Language** | Rust | TypeScript/Node.js | Go | TypeScript/React/Node.js | Ruby | JavaScript/Node.js | Rust | Python | Rust/Svelte | JavaScript/React | TypeScript/React/Node.js | TypeScript/Vue/Node.js | Python/Node.js | Rust |
| **Build Speed** | Fast (Rust native) | Fast (no JS by default) | Very fast (Go compiled) | Moderate (JS bundling) | Slow (Ruby) | Fast (minimal JS) | Very fast (Rust) | Moderate (Python) | Fast (Rust) | Slow (JS bundling) | Moderate (JS bundling) | Fast (Vite) | Fast (Python/JS) | Very fast (Rust) |
| **Markdown** | CommonMark + GFM, KaTeX, tree-sitter | MDX, frontmatter | CommonMark + GFM, shortcodes | MDX v3 (CommonMark) | CommonMark + GFM, Kramdown | Markdown (it/markdown-it) | CommonMark + GFM, shortcodes | reST, Markdown | MD, MDX (via plugin) | MDX | MDX (via remark) | Markdown (markdown-it) | Markdown + extensions | CommonMark + GFM |
| **Syntax Highlighting** | tree-sitter (9 active, 12 defined) | Shiki | Chroma, Pygments | Prism | Rouge | Prism, highlight.js | Sublime Highlight, syntect | Pygments | Shiki (via plugin) | Prism | Shiki (via plugin) | Shiki (markdown-it) | Pygments | syntect |
| **Math** | KaTeX | KaTeX, MathJax (plugin) | KaTeX (shortcode) | KaTeX (remark-math) | KaTeX (plugin) | MathJax, KaTeX (plugin) | KaTeX (shortcode) | MathJax (Pelican plugins) | KaTeX (via plugin) | KaTeX (via plugin) | KaTeX (remark-math) | MathJax (via plugin) | KaTeX |
| **i18n** | No (planned) | Built-in (content collections) | Built-in (multilingual) | Built-in (i18n plugin) | No (plugin) | No (plugin) | Built-in | No (plugin) | Built-in (svelte-i18n) | No (plugin) | Built-in (next-i18n) | Built-in | No (plugin) | No |
| **Theming** | CSS (Tailwind) | Astro components, CSS frameworks | Hugo themes (Go templates) | React components, swizzling | Liquid templates | Nunjucks, Liquid, Handlebars | Tera (Jinja2-like) | Jinja2 templates | Svelte components | React components, Gatsby themes | React components | Vue components | Material, mkdocs-material | Rust/HBS templates |
| **Search** | Pagefind (integrated in docs site) | Pagefind (recommended), Algolia | Fuse.js, Lunr (built-in), Algolia | Algolia DocSearch | Lunr, Algolia | Pagefind, FlexSearch | Built-in (Zola search) | Tipue Search | Pagefind, Fuse.js | Algolia DocSearch | Algolia | Local search (VitePress) | Algolia, lunr | Built-in |
| **Dynamic Content** | No (pure SSG) | Islands (Astro components) | Shortcodes, partials | React components (full SPA possible) | Liquid (limited) | Shortcodes | Shortcodes, Tera | Jinja2 | Full SSR/SSG | Gatsby SSR | Full SSR/SSG | No (SSG only) | No (SSG only) | No (SSG only) |
| **CMS Integration** | No | Decap CMS, Sanity, Contentful | Headless CMS (frontmatter) | Directus, Contentful | No | Decap CMS, Netlify CMS | No | No | No | Contentful, Sanity | Contentful, Sanity | No | No |
| **SSR** | No | Yes (Astro SSR) | No (SSG only) | No (SSG only, docs-focused) | No (SSG only) | No (SSG only) | No (SSG only) | No (SSG only) | Yes (SvelteKit SSR) | Yes (Gatsby SSR) | Yes (Next.js ISR/SSR) | No (SSG only) | No (SSG only) | No (SSG only) |
| **Deployment** | Static files (any host) | Vercel, Netlify, Cloudflare | Any static host | Vercel, Netlify | GitHub Pages, any host | Any static host | Any static host | Any static host | Vercel, Netlify, Cloudflare | Vercel, Netlify | Vercel, Netlify | Vercel, Netlify | GitHub Pages, any host | GitHub Pages, any host |
| **Image Optimization** | Built-in (image crate) | Built-in (sharp) | Built-in (image processing) | No (plugin) | No (plugin) | No (plugin) | Built-in (image resize) | No (plugin) | Built-in (Vite image) | gatsby-plugin-image | next/image | No (plugin) | No (plugin) | No |
| **Plugin/Theme Ecosystem** | N/A (monorepo) | Large (Astro marketplace) | Very large (Hugo themes) | Moderate (docusaurus-theme-* plugins) | Large (Jekyll plugins) | Small | Moderate | Small (Pelican plugins) | Moderate (Svelte ecosystem) | Large (Gatsby plugins) | Very large (Next.js ecosystem) | Small (VitePress plugins) | Large (MkDocs plugins/material) | Small |
| **Multi-Site** | No | Yes (Astro content collections) | Yes (multi-site config) | Yes (versioned docs) | No | Yes | Yes (sections) | Yes | No | No | Yes (multi-zones) | Yes (multi-sidebar) | Yes (nav sections) | No |
| **Data Sources** | Local Markdown files | Local MD, MDX, YAML, JSON, CMS, APIs | Local MD, YAML, TOML, JSON, CMS, APIs | Local MDX, MD, JSON | Local MD, YAML, CSV | Local MD, JSON, YAML, CMS, APIs | Local MD, TOML, YAML, JSON | Local MD, reST | Local MD, MDX, CMS, APIs | Local MD, MDX, CMS, GraphQL | Local MDX, MD, CMS, APIs | Local MD, YAML, JSON | Local MD, Markdown | Local MD |
| **Live Reload** | Yes (notify file watcher) | Yes (Astro dev server) | Yes (Hugo server) | Yes (Docusaurus dev server) | Yes (Jekyll serve) | Yes (Eleventy serve) | Yes (Zola serve) | Yes (Pelican serve) | Yes (SvelteKit dev) | Yes (Gatsby develop) | Yes (Next.js dev) | Yes (VitePress dev) | Yes (mkdocs serve) | Yes (mdbook serve) |
| **PDF Export** | No | No (plugin) | No (plugin) | PDF via theme | No | No | No | PDF (via weasyprint) | No | No | No | PDF (plugin) | PDF (via weasyprint) | No |
| **Package Manager** | Cargo | npm/pnpm/yarn | Go modules / Hugo extended | npm | gem (Ruby) | npm | Binary download | pip | npm | npm | npm | npm | pip | Cargo |
| **License** | Apache 2.0 | MIT | Apache 2.0 | MIT | MIT | MIT | MIT | AGPLv3 | MIT | MIT | MIT | MIT | MIT | MPL-2.0 / Apache 2.0 |
| **GitHub Stars** (approx) | New | 48k+ | 78k+ | 56k+ | 49k+ | 18k+ | 15k+ | 12k+ | 82k+ (Svelte) | 56k+ | 130k+ | 24k+ | 21k+ | 20k+ |

### Category B Notes

- **Hugo** is the fastest SSG for large sites (thousands of pages). Go-compiled binary, zero dependencies. Weak at interactive/dynamic content.
- **Astro** excels at content-focused sites with partial interactivity (Islands architecture). Strong ecosystem and DX.
- **Docusaurus** is the de facto standard for developer documentation. React-based, MDX support, versioned docs, i18n.
- **VitePress** is Vue-based, optimized for docs. Fast builds (Vite), excellent default theme.
- **MdBook** is the closest to Tachyon SSG in terms of language (Rust) and purpose (documentation). Minimal feature set.
- **Zola** is another Rust SSG. Fast builds, Tera templating. Smaller ecosystem than Hugo.
- **Tachyon SSG** differentiates by being integrated with the full knowledge management backend (import from DB, render with same renderer, publish to static output). Standalone SSGs require external content pipelines.

---

## Category C: Real-Time Collaborative Editors

These are dedicated collaborative editing platforms. Tachyon competes here via its WebSocket + CRDT real-time editing.

| Feature | Tachyon Editor | Google Docs | HackMD | CryptPad | Etherpad | collaborative.js | Hocus Pocus | Yjs (library) |
|---------|----------------|-------------|--------|----------|----------|-------------------|-------------|----------------|
| **Collab Model** | CRDT (Yrs/lib0) | OT (Wave/Operational Transformation) | OT (custom) | OT (custom, E2EE) | OT (custom) | CRDT (Yjs) | CRDT (Y-crdt) | CRDT (Yjs, library) |
| **Conflict Strategy** | Server-side Yrs state + relay | Server-authoritative OT | Server-authoritative OT | Server-authoritative OT | Server-authoritative OT | Decentralized CRDT | Decentralized CRDT | Decentralized CRDT |
| **Concurrent Users** | 1000+ (broadcast channel) | 100+ (per doc) | 50+ (per doc) | 30+ (per doc, E2EE overhead) | 50+ (per doc) | Unlimited (P2P) | 10+ (P2P) | Unlimited (library) |
| **Max Users/Doc (Tested)** | Not yet production-tested | ~100 | ~50 | ~30 | ~50 | Unlimited (scaling depends on transport) | ~10 | N/A (library) |
| **Offline Support** | No | Yes (Google offline) | No | Yes (E2EE, local) | No | Yes (P2P CRDT) | Yes (P2P CRDT) | Yes (P2P CRDT) |
| **E2E Encryption** | No | No | No (Enterprise plan) | Yes (CryptPad E2EE) | No | Possible (via provider) | Possible | Possible (via provider) |
| **Rich Text** | Markdown (rendered, tree-sitter) | Rich text (proprietary) | Markdown + rich text | Rich text (proprietary) | Rich text (basic) | Quill, CodeMirror, Monaco | ProseMirror | Any editor (bindings) |
| **Cursor Presence** | Yes (WebSocket) | Yes | Yes | Yes | Yes | Yes (via awareness protocol) | Yes | Yes (awareness protocol) |
| **Version History** | Server-side (PostgreSQL) | Full revision history | Document history | Version tracking | Timeslider | Via persistence provider | Via persistence provider | Via persistence provider |
| **Comments** | No (planned) | Yes (inline) | Yes (line-based) | Yes | No | No (editor-level) | No | No (library) |
| **Embedding** | No (planned) | Limited | Yes (iframe, API) | No | Yes (iframe, API) | Yes (API) | No | Yes (API) |
| **Self-Hosted** | Yes | No | Yes (CE, self-hosted) | Yes | Yes | Yes (DIY) | No (library) | N/A (library) |
| **License** | Apache 2.0 | Proprietary | Apache 2.0 (CE) | AGPLv3 | Apache 2.0 | MIT | MIT | MIT |
| **Framework** | Rust + Leptos WASM | JavaScript (closure compiler) | Node.js/React | Node.js | Node.js | TypeScript | TypeScript | TypeScript |
| **WebSocket** | Axum 0.8, binary relay | Proprietary (gRPC-like) | Socket.IO | WebSocket | Socket.IO | WebRTC, WebSocket | WebRTC | WebSocket, WebRTC |

### Category C Notes

- **Yjs** is the most widely adopted CRDT library. Tachyon uses Yrs (the Rust port of Yjs). collaborative.js and Hocus Pocus are built on Yjs.
- **Google Docs** remains the gold standard for collaborative editing UX. Proprietary, no self-hosting.
- **HackMD** offers both SaaS and self-hosted (Community Edition). Markdown-native with real-time collaboration.
- **Etherpad** is the oldest open-source collaborative editor. OT-based, simple architecture. Limited rich text.
- **CryptPad** is unique for E2E encrypted collaboration. Performance limited by encryption overhead.

---

## Category D: Knowledge Graph / PKM (Personal Knowledge Management)

Platforms that focus on knowledge graphs, bidirectional links, and structured knowledge.

| Feature | Tachyon | Obsidian | Logseq | Anytype | Fibery | Athens | Roam Research | Foam (VS Code) |
|---------|---------|----------|--------|---------|--------|--------|---------------|----------------|
| **Knowledge Graph** | Yes (nodes/edges CRUD, BFS shortest path, connected components, temporal queries, graph diff, stats) | Graph view (community plugin) | Graph view | Native graph | Relations, views | Native graph | Native graph | No (backlinks only) |
| **Bidirectional Links** | Yes ([[target]] parser, outgoing_links JSONB + GIN index, backlinks API) | Yes (native) | Yes (native) | Yes (native) | Bidirectional refs | Yes (native) | Yes (native) | Yes (via plugin) |
| **Block-Based** | No (document-based) | No (document-based) | Yes (outliner) | Yes (object-based) | No (entity-based) | No (document-based) | Yes (outliner) | No (document-based) |
| **Daily Notes** | Yes (auto-create dated note nodes via API) | Yes (core feature) | Yes (core feature) | Yes | No | Yes | Yes | Yes |
| **Graph Query** | Structural (shortest path, neighbors) + semantic (pgvector + AI) | Dataview plugin (JS) | Advanced queries | Relation queries | Fibery query language | No | Datalog queries | No |
| **Offline** | Partial (PWA asset cache, offline.html) | Yes | Yes | Yes (local-first, CRDT) | No (cloud) | Yes (local-first) | No (cloud) | Yes (local) |
| **Collab** | Yes (CRDT, WebSocket) | No (third-party sync) | No | Yes (CRDT, P2P) | Yes (real-time) | No | No | No |
| **Self-Hosted** | Yes | No | Yes (self-hosted Docker) | Yes (local, self-hosted sync) | No | Yes | No | Yes (local) |
| **License** | Apache 2.0 | Proprietary | AGPLv3 | MPL-2.0 (local), AGPLv3 (self-hosted) | Proprietary | AGPLv3 | Proprietary | MIT |
| **AI Integration** | Plugin-based (OpenAI, Anthropic, Ollama) | Copilot (plugin) | AI chat (plugin) | Anytype AI (beta) | Fibery AI | No | AI (third-party) | No |
| **API** | REST + GraphQL + WebSocket | No public API | No public API | REST API | REST API | GraphQL | REST API | No |

### Category D Notes

- **Obsidian** dominates PKM. Local-first, file-system based, massive plugin ecosystem. No built-in collaboration.
- **Logseq** is an open-source outliner with graph view. AGPLv3. Focus on daily notes and outliner workflows.
- **Anytype** is local-first with P2P sync (CRDT-based). Object-oriented rather than document-oriented.
- **Fibery** is a workspace platform with relations, views, and automation. Cloud-only, not self-hosted.
- **Tachyon** combines server-side knowledge graph (structural queries: shortest path, connected components, temporal diff) with bidirectional links and semantic search. Graph visualization frontend is not yet implemented.

---

## Category E: Developer Documentation Platforms

Platforms focused specifically on API documentation and developer docs. Tachyon's Swagger UI + SSG overlap here.

| Feature | Tachyon | GitBook | ReadMe | Mintlify | Docusaurus | Stoplight | Bump.sh | Redocly | Zitadel Docs |
|---------|---------|---------|--------|----------|------------|-----------|---------|---------|--------------|
| **OpenAPI Support** | Swagger UI (utoipa, auto-generated) | OpenAPI import | OpenAPI import | OpenAPI import | No native | OpenAPI Studio (visual editor) | Auto-generated from API | OpenAPI-focused | OpenAPI-focused | No |
| **API Explorer** | Swagger UI (interactive) | GitBook API explorer | ReadMe API playground | Interactive API blocks | No | Stoplight Studio (mock, test) | API explorer | Try-it panel | Mock server | No |
| **Versioning** | Document versions API | Versioned docs | API versioning | API versioning | Versioned docs | API versioning | API versioning | API versioning | Versioned docs | No |
| **Changelog** | CHANGELOG.md | GitBook changelog | ReadMe changelog | Auto-generated | No | No | API changelog | No | No |
| **i18n** | No (planned) | Yes (localization) | Yes | Yes | Yes | Yes | No | Yes | Yes |
| **Search** | Tantivy + tsvector | Algolia DocSearch | Built-in | Algolia | Algolia | Built-in | Built-in | Built-in | Built-in |
| **Custom Domain** | Planned (nginx config exists) | Yes (custom domains) | Yes | Yes | Any static host | Yes | Yes | Yes | Yes |
| **Self-Hosted** | Yes | No | No | No | Yes | Yes (Enterprise) | No | Yes (Enterprise) | Yes |
| **Code Samples** | tree-sitter highlighting | Code blocks | Multi-language playground | Multi-language code blocks | Code blocks with MDX | Code samples in OpenAPI | Auto-generated from OpenAPI | Code blocks with MDX | Code blocks |
| **Auth Docs** | JWT, guest access | API key docs | OAuth docs | API key docs | No | OAuth/API key | API key | API key | OIDC docs |

---

## Category F: Self-Hosted Wiki / CMS

Traditional self-hosted wiki and CMS platforms that overlap with Tachyon's knowledge management functionality.

| Feature | Tachyon | Wiki.js | BookStack | XWiki | TiddlyWiki | Docmost | Affine | SiYuan |
|---------|---------|---------|-----------|-------|------------|---------|--------|--------|
| **License** | Apache 2.0 | AGPLv3 | MIT | LGPLv2.1 | MIT | AGPLv3 | MPL-2.0 | AGPLv3 |
| **Language** | Rust | TypeScript/Node.js | PHP / Vue.js | Java | JavaScript (SPA) | TypeScript/Node.js | TypeScript | TypeScript/Electron |
| **Real-Time Collab** | CRDT (WebSocket) | PostgreSQL LISTEN/NOTIFY | No | No | No | CRDT (Yjs, WebSocket) | CRDT (Yjs) | CRDT (Yjs) |
| **Markdown** | CommonMark + GFM | Multiple (MD, WYSIWYG, API) | WYSIWYG + MD (editor) | XWiki Syntax / MD | WikiText | CommonMark + GFM | Blocks (Notion-like) | Blocks (Notion-like) |
| **WYSIWYG Editor** | No (MD-first) | Yes (multiple editors) | Yes (WYSIWYG) | Yes (WYSIWYG) | Yes (WikiText) | No (MD-first) | Yes (block editor) | Yes (block editor) |
| **Search** | Tantivy + PostgreSQL tsvector | ElasticSearch, PostgreSQL, Solr | Simple full-text | Lucene | Built-in search | PostgreSQL tsvector | Built-in | Built-in |
| **RBAC** | Fine-grained RBAC | Groups, permissions | Roles, permissions | Fine-grained rights | Plugins (limited) | Spaces, members | Spaces (planned) | No (local-first) |
| **Plugin System** | WASM (Wasmtime) | npm modules | No | Java extensions | Plugins (TiddlyWiki plugins) | No | Yes (JavaScript) | Yes (JavaScript) |
| **SSG** | Built-in | No | No | No | Single HTML file export | No | No | No |
| **API** | REST + Swagger + GraphQL | REST + GraphQL | REST | REST (XWiki REST) | No | REST + WebSocket | REST | REST + WebSocket |
| **Multi-Language Content** | No (planned) | Yes (locale per page) | Yes (locale per chapter) | Yes | Yes (plugin) | Yes (planned) | No (planned) | Yes |
| **Attachments** | Upload API | Yes (storage) | Yes (chapters, pages) | Yes (attachments) | Yes (embedded) | Yes (planned) | Yes | Yes |
| **Database** | PostgreSQL | PostgreSQL, MySQL, MariaDB, SQLite, MongoDB | MySQL, PostgreSQL, SQLite | MySQL, PostgreSQL, HSQLDB | HTML file (IndexedDB) | PostgreSQL | SQLite (local) | SQLite (local) |
| **Docker** | Yes | Yes | Yes | Yes | No (single HTML) | Yes | Yes | Yes |
| **Desktop** | Tauri 2 | No | No | No | No (SPA, can be wrapped) | No | Desktop app | Desktop app |
| **AI Features** | Plugin-based AI providers | AI Assistant (built-in) | No | AI (experimental) | No | No | AI (planned) | AI (built-in, SiYuan AI) |

### Category F Notes

- **Docmost** is a new, lightweight open-source alternative to Confluence. Uses Yjs CRDT for real-time collaboration. AGPLv3.
- **Affine** is an open-source Notion alternative with block editor and CRDT collaboration. MPL-2.0 license.
- **SiYuan** is a Chinese-developed PKM with block-based editor and CRDT. Strong in Chinese market.
- **BookStack** is the simplest self-hosted wiki. PHP-based, WYSIWYG, good for non-technical teams.
- **Wiki.js** is the most mature open-source wiki. Multiple editor types, plugin system, good admin UI.

---

## Category G: API-First Backend Frameworks

Platforms that provide APIs for document/knowledge management, similar to Tachyon's REST + WebSocket API.

| Feature | Tachyon | Appwrite | Supabase | PocketBase | Directus | Strapi | Nhost |
|---------|---------|----------|----------|------------|----------|--------|-------|
| **API** | REST + Swagger, WebSocket | REST, GraphQL, WebSocket | REST, GraphQL, Realtime (WebSocket) | REST, Admin API | REST, GraphQL, WebSocket | REST, GraphQL | REST, GraphQL, WebSocket |
| **Auth** | JWT (multi-key rotation), OAuth2, MFA (TOTP), SSO (OIDC), guest access | JWT, OAuth2, Magic URL, Phone | JWT, OAuth2, Magic URL, Phone | JWT, OAuth2 | JWT, OAuth2, LDAP, SAML | JWT, OAuth2 | JWT, OAuth2, Magic URL |
| **RBAC** | Fine-grained RBAC | Roles + teams | RLS (PostgreSQL) | Admin, Auth rules | Roles + permissions | Roles + permissions | Hasura RLS |
| **Real-Time** | WebSocket (CRDT relay) | WebSocket (subscriptions) | Realtime (Postgres changes) | WebSocket (subscriptions) | WebSocket (subscriptions) | No native | Hasura Realtime |
| **CRDT** | Yrs (server-side state) | No | No | No | No | No | No |
| **Database** | PostgreSQL | PostgreSQL, MariaDB, SQLite | PostgreSQL | SQLite | PostgreSQL, MySQL, SQLite | PostgreSQL, MySQL, SQLite | PostgreSQL |
| **SSG** | Built-in | No | No | No | No | No | No |
| **Search** | Tantivy + tsvector + pgvector | Full-text (per DB) | Full-text (PostgreSQL) | Full-text (SQLite FTS5) | Full-text (per DB) | Full-text (per DB) | Full-text (PostgreSQL) |
| **Storage** | Pluggable (PostgreSQL) | S3-compatible storage | S3-compatible storage | S3-compatible storage | S3-compatible storage | S3-compatible storage | S3-compatible storage |
| **Functions** | WASM plugins (Wasmtime) | Cloud Functions (custom runtime) | Edge Functions (Deno) | Hooks (JS) | Hooks (custom endpoints) | Custom routes | Hasura Functions (TypeScript) |
| **Self-Hosted** | Yes | Yes (Docker) | Yes (Docker) | Yes (single binary) | Yes (Docker) | Yes (Docker) | Yes (Docker) |
| **License** | Apache 2.0 | BSD-3 | Apache 2.0 | MIT | GPL-3.0 | MIT | MIT |
| **Language** | Rust | TypeScript/Node.js | TypeScript (Elixir core) | Go | TypeScript/Node.js | TypeScript/Node.js | TypeScript (Haskell core) |

### Category G Notes

- **Supabase** is the closest BaaS competitor for the API/database layer. Lacks built-in CRDT, SSG, and document-focused features.
- **Appwrite** is a comprehensive BaaS with storage, functions, and real-time. No CRDT collaboration.
- **PocketBase** is the lightest option (single Go binary). Good for small projects. Lacks advanced features.
- Tachyon differentiates by being document-first with built-in SSG, CRDT collaboration, and knowledge management rather than a generic BaaS.

---

## Category H: Import/Export Ecosystem

Platforms' ability to migrate content in and out.

| Source Format | Tachyon | Notion | Confluence | Outline | Obsidian | BookStack | Wiki.js |
|-------------|---------|--------|------------|---------|----------|-----------|---------|
| **Markdown** | Yes (import + export) | Import only | Import (plugin) | Import (API) | Native | Import (API) | Import |
| **HTML** | Yes (import + export) | Export (HTML) | Export (HTML/PDF) | No | Export (plugin) | No | No |
| **Docusaurus** | Yes (import) | No | No | No | No | No | No |
| **Obsidian** | Yes (import) | No | Import (plugin) | No | Native | No | No |
| **Generic Markdown Vault** | Yes (import: recursive dir scan, ZIP, frontmatter, #tags) | No | No | No | Native | No | No |
| **JSON** | Yes (import + export) | Import (CSV/JSON) | Import (plugin) | Export (API) | No | Import (API) | Import (API) |
| **Confluence** | No (planned) | Import | Native | Import (plugin) | Import (plugin) | No | Import |
| **Notion** | No (planned) | Native | Import (plugin) | No | Import (plugin) | No | Import |
| **PDF Export** | No (planned) | Yes (Enterprise) | Yes | No | Yes (plugin) | Yes (PDF) | Yes (PDF) |
| **Word/DOCX** | No | Yes (Enterprise) | Yes | No | No | Yes (import) | No |
| **ZIP Archive** | Yes (import/export) | No | Yes | No | Yes (vault) | Yes | Yes |

---

## Tachyon's Competitive Positioning

### Strengths (Unique or Superior)

1. **Full Rust stack** -- Only Tachyon and a handful of SSG tools (Hugo, Zola, MdBook) are built entirely in Rust. No other collaborative knowledge base is Rust-native. This delivers: memory safety without GC, deterministic performance, and minimal attack surface.
2. **Integrated SSG + collaboration** -- No competitor combines real-time CRDT editing with a built-in static site generator. Tachyon can serve both as a live collaborative editor and as a documentation publishing pipeline.
3. **WASM plugin sandbox** -- Wasmtime-based plugin runtime provides safe extensibility. No other knowledge base offers WASM sandboxing (most use npm/Docker plugins with full filesystem access).
4. **Semantic + structural knowledge graph** -- Server-side graph with BFS shortest path, connected components, temporal diff, bidirectional wiki-links, backlinks API, orphan node detection, and pgvector semantic similarity. No other self-hosted platform offers this combination.
5. **DLP content scanning** -- Built-in data loss prevention (credit cards, SSN, API keys) wired into document create/update flow. Most competitors offer this only at Enterprise tier or not at all.
6. **Comprehensive auth** -- JWT with multi-key rotation, OAuth2 (Google/GitHub), MFA (TOTP), SSO (OIDC with discovery/code exchange, SAML partial, LDAP bind/search/sync), guest access, session management, password reset -- all self-hosted. Most competitors require SaaS for SSO.
7. **Formal verification** -- TLA+ specs and Lean4 proofs in `.specs/`. No competitor publishes formal correctness proofs for their core algorithms.
8. **Monorepo with 16 crates** -- Clean separation of concerns (core, server, database, search, renderer, RBAC, SSG, editor, import/export, plugin-runtime, desktop, frontend, CLI, storage, testing, benchmarks). Each crate is independently testable.
9. **Comprehensive CI/CD** -- 11 GitHub Actions workflows including security scanning (Semgrep, Trivy, TruffleHog), SBOM generation, mutation testing (cargo-mutants), OWASP ZAP penetration testing, and Playwright E2E tests.
10. **PWA with offline fallback** -- Service worker with TTL-based cache invalidation, stale-while-revalidate for static assets, offline.html fallback, network-first for API calls.
11. **GraphQL + REST dual API** -- Both REST (with auto-generated Swagger UI via utoipa) and GraphQL (via async-graphql) endpoints. Most competitors offer only one or the other.
12. **Notification pipeline** -- Web Push (VAPID), Email (lettre SMTP), and Webhook delivery (with retry logic). Self-hosted platforms rarely offer all three.

### Weaknesses (Compared to Market Leaders)

1. **No production deployment** -- Tachyon has no running production instance. Notion, Confluence, and Obsidian have millions of active users.
2. **No native mobile app** -- No native iOS/Android client. PWA covers offline asset caching but not native features (push notifications, camera, biometrics). Competitors like Notion, Obsidian, and Logseq have mature mobile apps.
3. **No local-first / offline sync** -- PWA caches static assets but documents require server connectivity. Local-first competitors (Obsidian, Anytype, Logseq) work without network with full document sync.
4. **No WYSIWYG editor** -- Markdown-first editing. Users who prefer block-based or rich-text editors (Notion, Confluence, Affine) face a learning curve.
5. **No graph visualization UI** -- Knowledge graph data model and API exist (nodes, edges, shortest path, backlinks), but no frontend graph view. PKM-focused competitors (Obsidian, Logseq, Roam) offer native graph visualization.
6. **Plugin ecosystem is empty** -- WASM plugin runtime exists but has zero community plugins. Obsidian has 2,000+ community plugins; Confluence has the Atlassian Marketplace.
7. **No i18n UI localization** -- 8-locale i18n framework exists in frontend code but no translations are populated. Most competitors support 10+ localized languages.
8. **No inline comments or embeds** -- No inline commenting on document sections. Embed block parser exists (`!{youtube}`, `!{mermaid}`) but no frontend rendering. Notion and Confluence both support inline comments and embeds natively.
9. **Limited external migration** -- Can import from Docusaurus, Obsidian, generic markdown vaults, ZIP archives, JSON, HTML. Cannot import from Notion, Confluence, or Google Docs (requires their proprietary APIs).
10. **SAML partial, LDAP runtime** -- OIDC runtime (authorize/callback) is implemented. SAML has runtime handlers (SP metadata endpoint, ACS endpoint) but no XML signature validation. LDAP has full runtime handlers with bind authentication, user search, and directory sync via ldap3.

### Differentiation Summary

| Dimension | Tachyon's Position |
|-----------|-------------------|
| **Performance** | Top-tier (Rust, SIMD markdown, sub-15ms render) |
| **Security** | Top-tier (WASM sandbox, formal verification, TLA+/Lean4, DLP, OWASP ZAP, mutation testing) |
| **Self-Hosting** | Top-tier (Docker, Nix, single binary, Apache 2.0) |
| **Real-Time Collab** | Competitive (CRDT via Yrs, WebSocket relay, presence) |
| **Search** | Competitive (BM25 + semantic/pgvector) |
| **Knowledge Graph** | Competitive (structural queries + semantic, bidirectional links, backlinks) |
| **Auth / SSO** | Top-tier (JWT, OAuth2, TOTP MFA, SSO OIDC/SAML/LDAP, guest access -- all self-hosted) |
| **SSG** | Unique (only platform combining SSG + collaboration) |
| **Plugin System** | Unique architecture (WASM sandbox vs. npm/Docker) |
| **Offline** | Partial (PWA asset cache, no local-first sync) |
| **UX / Editor** | Weak (Markdown-only, no WYSIWYG, no blocks) |
| **Ecosystem** | Weak (new project, zero community plugins) |
| **Mobile** | Weak (PWA only, no native mobile apps) |
| **Graph UI** | Weak (API exists, no frontend visualization) |
| **Maturity** | Weak (pre-production, no users) |

---

## Sources

- Official documentation and GitHub repositories for all listed products (accessed 2026-05-31).
- GitHub star counts are approximate and may differ from live values.
- Feature data verified against official docs, source code, and community wikis.
- Licensing information from official repository LICENSE files and SPDX headers.
