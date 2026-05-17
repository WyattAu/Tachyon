# SSG Capabilities Gap Analysis

## Current State

`tachyon-ssg` (~4100 LOC) is a minimal static site generator built with Rust. It produces responsive HTML pages from Markdown files with YAML frontmatter, supporting i18n (20 languages, RTL), RSS, sitemap, dark mode, and ZIP download. It delegates markdown rendering to `tachyon-renderer` (pulldown-cmark + ammonia + tree-sitter + katex).

### What Works Today

- HTML output with responsive layout, SEO meta tags, OG/Twitter cards
- RSS 2.0 feed with Atom self-link, per-language feeds
- XML sitemap with `xhtml:link hreflang` for i18n
- ZIP archive build (in-memory, downloadable via server API)
- i18n: per-language subdirectories, 20 language display names, RTL detection, language switcher
- Dark mode: auto/light/dark via CSS class + `prefers-color-scheme` media query
- Custom theming: 6 CSS custom properties, custom font families
- YAML frontmatter: title, description, author, tags, order, language
- Wikilinks: `[[target]]` and `[[target|display]]` with code block awareness
- GFM markdown: tables, task lists, strikethrough, footnotes, smart punctuation, heading attributes, autolinks
- XSS sanitization via ammonia
- Tag-based category pages
- CLI: `build` and `init` subcommands
- Server integration: 3 HTTP endpoints (config, build, download)
- GitHub Pages: `.nojekyll` file, default base URL

---

## Capabilities Gap Analysis

### Priority 1: Critical (Table Stakes)

| # | Capability | Docusaurus | VitePress | MkDocs Material | Hugo | Astro | Tachyon |
|---|-----------|-----------|-----------|-----------------|------|-------|---------|
| 1 | Client-side search | Pagefind | Minisearch | Search | FlexSearch | Pagefind | **Missing** |
| 2 | Sidebar / table of contents | Yes | Yes | Yes | Yes | Yes | **Missing** |
| 3 | Versioned docs (v1/v2) | Yes | No | No | No | No | **Missing** |
| 4 | Breadcrumbs | Yes | Yes | Yes | Yes | No | **Missing** |
| 5 | Prev/next page navigation | Yes | Yes | Yes | Yes | No | **Missing** |
| 6 | Syntax highlighting in pages | Prism | Shiki | Pygments | Chroma | Shiki | **Not integrated** (tree-sitter exists in renderer) |
| 7 | LaTeX/math rendering | KaTeX | KaTeX | MathJax | KaTeX | No | **Not integrated** (KaTeX exists in renderer) |
| 8 | Admonitions/callouts | Yes | Yes | Yes | Yes | Yes | **Missing** |

### Priority 2: Navigation & UX

| # | Capability | Status |
|---|-----------|--------|
| 9 | Nested sidebar navigation (hierarchical dir tree) | Missing |
| 10 | Collapsible sidebar sections | Missing |
| 11 | Mobile hamburger menu with slide-out drawer | Missing (nav just hidden) |
| 12 | Scroll-to-top button | Missing |
| 13 | Reading progress bar | Missing |
| 14 | "Edit this page" link to source `.md` | Missing |
| 15 | "Last updated by" with git blame info | Missing (date only) |
| 16 | Sticky header with scroll-aware hide/show | Partial (always sticky) |

### Priority 3: Content Features

| # | Capability | Status |
|---|-----------|--------|
| 17 | Tabs component | Missing |
| 18 | Details/accordion component | Missing |
| 19 | Multi-language code blocks (code group) | Missing |
| 20 | Mermaid diagram rendering | Missing (importer preserves, SSG doesn't render) |
| 21 | Image zoom/lightbox | Missing |
| 22 | Copy button on code blocks | Missing |
| 23 | Line highlighting in code blocks (`highlight-lines`) | Missing |
| 24 | Code block filename captions | Missing |
| 25 | Per-page table of contents | Missing |
| 26 | Custom shortcodes/components (`{{% %}}`, `{{< >}}`) | Missing |
| 27 | MDX support (JSX in markdown) | Missing |
| 28 | Interactive components (React/Vue/Svelte islands) | Missing |
| 29 | Content collections (typed frontmatter schemas) | Missing |
| 30 | Draft documents (`draft: true` frontmatter) | Missing |
| 31 | Slug customization via frontmatter `slug:` override | Missing (derived from path only) |
| 32 | Aliases/redirects for moved pages | Missing |
| 33 | Pagination for long document lists | Missing |
| 34 | Footnote styling/back-references | Partial (parsed but not styled) |

### Priority 4: SEO & Metadata

| # | Capability | Status |
|---|-----------|--------|
| 35 | JSON-LD structured data (Article, BreadcrumbList, FAQPage) | Missing |
| 36 | `robots.txt` generation | Missing |
| 37 | `hreflang` alternate links in `<head>` per page | Partial (sitemap only) |
| 38 | Open Graph image (`og:image`) | Missing |
| 39 | Reading time estimation | Missing |

### Priority 5: Theming & Customization

| # | Capability | Status |
|---|-----------|--------|
| 40 | Multiple built-in themes | Missing (one theme) |
| 41 | Theme gallery / community themes | Missing |
| 42 | Plugin/extension system | Missing |
| 43 | Custom page layouts per section | Missing (one template) |
| 44 | Reusable component library (buttons, cards, icons) | Missing |
| 45 | Icon library integration (FontAwesome, Lucide, Material) | Missing |
| 46 | Print stylesheet (`@media print`) | Missing |
| 47 | Custom color palettes (light/dark independent) | Partial (same palette both modes) |
| 48 | Font pairing (heading vs body) | Partial (both default to system-ui) |

### Priority 6: Build System & DX

| # | Capability | Status |
|---|-----------|--------|
| 49 | Incremental builds (file hashing, cache invalidation) | Missing (full rebuild) |
| 50 | Hot reload / live preview (`--watch` with browser sync) | Missing (one-shot CLI only) |
| 51 | Build-time CSS generation (no CDN dependency) | Missing (Tailwind via CDN) |
| 52 | Asset hashing for cache busting | Missing (`filehash = false`) |
| 53 | Image optimization (WebP, `srcset`, lazy loading) | Missing |
| 54 | Font preloading (`<link rel="preload">`) | Missing |
| 55 | Critical CSS inlining | Missing |
| 56 | Proper TOML/YAML parsing (serde-based) | Missing (hand-rolled parsers) |
| 57 | Content validation (frontmatter schema enforcement) | Missing |
| 58 | Content plugins (auto-generate docs from code) | Missing |
| 59 | Build statistics / warnings | Partial (basic stats only) |
| 60 | 404 page in library API (not just CLI) | Missing |

### Priority 7: i18n Improvements

| # | Capability | Status |
|---|-----------|--------|
| 61 | Translation completeness indicators | Missing |
| 62 | Locale-aware date/time formatting | Missing (RFC 3339 only) |
| 63 | Locale-aware number formatting | Missing |
| 64 | Translation memory / shared glossary | Missing |
| 65 | RTL-specific CSS adjustments | Missing (detection exists, no CSS fixes) |
| 66 | Fallback language chain (e.g., zh-CN -> zh -> en) | Missing |

### Priority 8: Deployment & Integration

| # | Capability | Status |
|---|-----------|--------|
| 67 | Git-based deployment (Vercel, Netlify, Cloudflare Pages) | Missing |
| 68 | Docker-based builds (working Dockerfile) | Broken (desktop crate) |
| 69 | PR preview deployments | Missing |
| 70 | Analytics dashboard integration | Partial (tracking_id field only) |

### Priority 9: API Documentation

| # | Capability | Status |
|---|-----------|--------|
| 71 | OpenAPI/Swagger spec rendering as docs | Missing (spec generated, not rendered) |
| 72 | Auto-generated API reference pages from OpenAPI | Missing |
| 73 | Interactive API explorer (Try-it buttons) | Missing |
| 84 | API version tabs (v1/v2 side-by-side) | Missing |

### Priority 10: Data & Dynamic Content

| # | Capability | Status |
|---|-----------|--------|
| 75 | Build-time data loading (JSON/YAML/CSV) | Missing |
| 76 | WASM-based interactivity in generated pages | Missing (despite being a Rust/WASM project) |
| 77 | Server-side rendering mode | Missing (pure static HTML) |
| 78 | Static params / dynamic routes | Missing |

---

## Architecture Issues

1. **Hand-rolled parsers** -- YAML frontmatter and TOML config use custom string parsing instead of `serde_yaml`/`toml` crates. Fragile, no error recovery, limited validation.
2. **No template engine for pages** -- All HTML is `format!()` string interpolation. No inheritance, no partials, no includes.
3. **Renderer integration gap** -- `tachyon-renderer` has tree-sitter (9 languages, 3 themes) and KaTeX support, but the SSG never invokes them during page rendering.
4. **No asset pipeline** -- No processing of images, fonts, CSS. No hashing, no optimization, no bundling.
5. **No extension points** -- No hooks, no plugins, no custom transformers, no middleware.

---

## Recommended Implementation Order

### Phase 1: Foundation (Unblocks everything else)
1. Replace hand-rolled parsers with `serde`-based parsing
2. Integrate syntax highlighting from `tachyon-renderer` into SSG HTML output
3. Integrate KaTeX rendering from `tachyon-renderer` into SSG HTML output
4. Add per-page table of contents generation
5. Add admonition/callout rendering (`> [!NOTE]`, `> [!WARNING]`, etc.)

### Phase 2: Navigation
6. Implement sidebar with auto-generated TOC from headings
7. Add breadcrumbs
8. Add prev/next page navigation
9. Add mobile hamburger menu
10. Add scroll-to-top button

### Phase 3: Search & SEO
11. Integrate Pagefind for client-side search
12. Add JSON-LD structured data
13. Generate `robots.txt`
14. Add `hreflang` links in page `<head>`
15. Add Open Graph images

### Phase 4: Content Features
16. Add admonitions/callouts
17. Add tabs component
18. Add code groups (multi-language)
19. Render mermaid diagrams
20. Add copy button on code blocks
21. Add draft document support

### Phase 5: Build System
22. Add incremental builds
23. Switch to build-time Tailwind CSS generation (remove CDN dependency)
24. Add asset hashing
25. Add image optimization
26. Add hot reload / live preview mode

### Phase 6: Advanced
27. Plugin system
28. Custom page layouts
29. Versioned docs
30. Content collections with typed schemas
31. API docs from OpenAPI spec
32. WASM interactivity

---

## References

- Docusaurus 3: https://docusaurus.io/
- VitePress: https://vitepress.dev/
- MkDocs Material: https://squidfunk.github.io/mkdocs-material/
- Hugo: https://gohugo.io/
- Astro: https://astro.build/
- Trunk: https://trunkrs.dev/
