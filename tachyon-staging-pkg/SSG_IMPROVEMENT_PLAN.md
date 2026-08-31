# Tachyon SSG Improvement Plan

## Current State

The Tachyon SSG (`tachyon/crates/ssg/`) generates static documentation sites from Markdown with
features including KaTeX math, Mermaid diagrams, syntax highlighting, code groups, admonitions,
multi-language support, RSS feeds, sitemaps, and Pagefind search integration. However, it has
several bugs and significant feature gaps compared to established generators like Docusaurus and
VitePress.

---

## Part A: Bug Fixes (Critical)

### A1. Subdirectory link resolution (root_prefix not used)

**Problem:** `root_prefix` is computed in `render.rs:187-195` (e.g. `"../"` for
`operations/incident-response.html`) and stored in `PageContext`, but the template in
`templates.rs` never references it. Every navigation link from subdirectory pages resolves
incorrectly:

- Nav "Home": `href="index.html"` instead of `href="../index.html"`
- Sidebar items: `href="getting-started.html"` instead of `href="../getting-started.html"`
- Prev/Next: `href="{slug}.html"` instead of `href="../{slug}.html"`
- Pagefind assets: `href="/Tachyon/pagefind/..."` (absolute) instead of `href="../pagefind/..."`

**Impact:** Every link from a subdirectory page (operations, security, etc.) 404s. This is the
root cause of the user-reported "clicking on documentation shows 404" bug.

**Fix:** Pass `root_prefix` to the template and prefix all relative links:
- `templates.rs:600` — nav home link
- `templates.rs:822` — sidebar item hrefs
- `render.rs:215,223` — prev/next link hrefs
- `templates.rs:103-127` — pagefind CSS/JS paths (use relative instead of absolute)

**Files:** `tachyon/crates/ssg/src/templates.rs`, `tachyon/crates/ssg/src/render.rs`

### A2. Prev/Next ordering does not follow sidebar structure

**Problem:** Prev/next is determined by `(d.order, d.title)` sort order across all documents
(`build.rs:217`). The sidebar menu has its own ordering via `site.toml` weight. These two
orderings diverge, causing prev/next to jump between unrelated sections.

**Example:** On `architecture.html`, clicking "Next" goes to `search.html` instead of the
next sidebar item `api-reference.html`.

**Fix:** Derive prev/next from the sidebar order (which respects categories and sub-items)
rather than the flat document sort.

**Files:** `tachyon/crates/ssg/src/render.rs`, `tachyon/crates/ssg/src/build.rs`

### A3. Pagefind not run as part of the build pipeline

**Problem:** The template outputs a comment `<!-- Run: npx pagefind --site <output_dir> -->`
but never actually runs Pagefind. The docs workflow (`docs.yml`) does not run `npx pagefind`.
Search returns no results.

**Fix:** Add a `post_build` step to the SSG CLI that runs `npx pagefind --site <output_dir>`
(if pagefind is available). Update `docs.yml` to install and run Pagefind.

**Files:** `tachyon/crates/ssg/src/ssg_cli.rs`, `.github/workflows/docs.yml`

---

## Part B: Navigation and Layout

### B1. Left sidebar with collapsible sections

**Current:** A left sidebar exists (`render_sidebar`) but links are broken from subdirectory
pages (A1). Uses `<details>/<summary>` for collapsible sections.

**Missing vs Docusaurus/VitePress:**
- No scroll-spy highlighting of current heading in sidebar
- No collapsed-by-default state persisted in localStorage
- No active section indicator in sidebar
- Categories with links (clickable category headers)
- Sidebar width not configurable

**Plan:**
1. Fix root_prefix usage first (A1) — this restores basic sidebar functionality
2. Add scroll-spy: JS that highlights the sidebar item matching the current viewport heading
3. Add `collapsed` config option in `site.toml` per menu category
4. Persist sidebar collapsed state in localStorage
5. Add active section indicator (left border highlight)

### B2. Right-side table of contents (On This Page)

**Current:** Sticky right sidebar with h2/h3 heading links.

**Missing:**
- No scroll-spy — current heading not highlighted while scrolling
- No close/dismiss button on mobile (TOC takes full width on small screens)
- No configurable heading levels (hardcoded h2/h3)

**Plan:**
1. Add scroll-spy JS: IntersectionObserver on headings, highlight active TOC link
2. Add dismiss/close button that hides the TOC sidebar and stores preference in localStorage
3. Make heading levels configurable via `site.toml` (e.g. `toc_levels = [2, 3, 4]`)

### B3. Top navigation bar improvements

**Current:** Sticky top nav with logo, nav links, language switcher.

**Missing:**
- No hamburger menu on mobile (nav links hidden, only the floating sidebar FAB exists)
- No mobile dropdown for nav links

**Plan:**
1. Add hamburger menu button in the top nav for mobile
2. On mobile tap: show dropdown with nav links
3. Keep the existing floating FAB for the left sidebar (separate concern)

### B4. Multi-column footer

**Current:** Single-line footer text.

**Plan:** Add configurable footer with columns:
```toml
[footer]
copyright = "2026 Tachyon Contributors"
[[footer.columns]]
title = "Documentation"
links = [
    { name = "Getting Started", url = "/getting-started.html" },
    { name = "API Reference", url = "/api-reference.html" },
]
[[footer.columns]]
title = "Community"
links = [
    { name = "GitHub", url = "https://github.com/WyattAu/Tachyon" },
]
```

### B5. Breadcrumbs

**Current:** Auto-generated from slug path segments with Schema.org microdata. Only breadcrumbs
use `root_prefix` correctly.

**Plan:** Add option to hide breadcrumbs via frontmatter (`hide_breadcrumbs: true`).

---

## Part C: Theming

### C1. Dark/light mode toggle

**Problem:** The SSG supports `dark`/`light`/`auto` themes via CSS, but there is no UI toggle.
Theme is determined at build time only.

**Plan:**
1. Add a sun/moon toggle button in the top navbar
2. JS: toggle `dark` class on `<html>` and persist preference in localStorage
3. Respect `prefers-color-scheme` on first visit (auto mode)
4. Three states: light, dark, auto (follows system)

**Reference:** Docusaurus uses a similar three-state toggle.

### C2. Color scheme customization

**Current:** `ColorTheme` in `site.toml` supports primary/secondary/accent/code_bg/font colors.

**Missing:**
- No preview of color changes during development
- Limited to single accent color
- No dark-mode-specific color overrides

**Plan:**
1. Add `dark_*` variants for each color in `ColorTheme` (e.g. `dark_primary`, `dark_bg`)
2. Generate CSS variables for both light and dark modes
3. Add `tachyon-ssg serve --watch` with live reload for theme editing

### C3. Font control

**Current:** `font_family` and `heading_font_family` in `ColorTheme`.

**Plan:** Add web font loading with `font-display: swap` for custom Google Fonts.

---

## Part D: Search

### D1. Pagefind integration (complete the pipeline)

**Current:** Template outputs `data-pagefind-body` and `data-pagefind-ignore` attributes, plus a
`#search` div. But Pagefind is never actually run.

**Plan:**
1. In `ssg_cli.rs`, after `build_to_dir`, spawn `npx pagefind --site <output_dir>`
2. In `docs.yml`, install Node.js and run Pagefind as a post-build step
3. Add a visible search input in the top navbar (not just the `#search` div below the nav)
4. Add keyboard shortcut: `Ctrl+K` / `Cmd+K` opens search
5. Style the Pagefind search results to match the site theme

### D2. Search UI improvements

**Plan:**
1. Move search from below-nav to an overlay modal triggered by Ctrl+K
2. Add search icon in the navbar
3. Style the Pagefind dialog with site colors
4. Show keyboard shortcut hint in the search placeholder ("Search... Ctrl+K")

---

## Part E: Content Features

### E1. Code block improvements

**Current:** Highlight.js with copy button and auto-grouping of adjacent code blocks by language.

**Missing:**
- No line numbers
- No line highlighting (e.g. `{2,4-6}`)
- No filename/title label
- No diff highlighting (+/- lines)

**Plan:**
1. Add `line_numbers` option: `[//]: # (line-numbers)` or frontmatter
2. Add line highlighting: `[//]: # (highlight=2,4-6)` syntax
3. Add code block title: ````rust title=main.rs`
4. Add diff language support: ````diff with +/- coloring
5. All configurable via `site.toml` defaults

### E2. Content tabs (non-code)

**Current:** Only code-group tabs (adjacent code blocks auto-grouped).

**Missing:** Generic content tabs like Docusaurus `<Tabs>`.

**Plan:**
1. Add tab syntax using HTML comments or custom markdown extension:
   ```
   <!-- tabs -->
   **Tab 1**
   Content for tab 1.

   **Tab 2**
   Content for tab 2.
   <!-- /tabs -->
   ```
2. Render as a tabbed UI component

### E3. Image improvements

**Current:** Image re-encoding (PNG/JPEG) via the `image` crate.

**Missing:**
- No WebP encoding (falls back to PNG)
- No `loading="lazy"` attribute
- No responsive `srcset`
- No `<figure>` with `<figcaption>`

**Plan:**
1. Add WebP output option (`image_format = "webp"` in config)
2. Add `loading="lazy"` to all images by default
3. Generate `<picture>` with `<source>` for WebP + JPEG fallback
4. Wrap images in `<figure>` with caption support via alt text or `title` attribute

### E4. Admonition improvements

**Current:** GitHub-style `> [!NOTE/WARNING/TIP/DANGER/INFO/SUCCESS]` syntax.

**Missing:**
- No custom titles (always uses the type name)
- No collapsible admonitions

**Plan:**
1. Support custom titles: `> [!WARNING Custom Title Here]`
2. Add collapsible option: `> [!NOTE] {collapsed}` or config default

---

## Part F: SEO and Performance

### F1. Self-hosted assets (remove CDN dependency)

**Problem:** All JS/CSS (Tailwind, KaTeX, Highlight.js, Mermaid, Pagefind) loaded from CDN.
CDN failure = broken site. No offline support.

**Plan:**
1. Add `assets_dir` config option in `site.toml`
2. During build, download CDN assets to `assets_dir` (or use vendored copies)
3. Generate local `<link>` and `<script>` tags pointing to local copies
4. Keep CDN as fallback with `<link rel="preconnect">` for performance
5. Hash filenames for cache busting

### F2. Lazy loading

**Plan:**
1. Add `loading="lazy"` to all `<img>` tags
2. Defer non-critical CSS (KaTeX, Mermaid) until viewport intersection
3. Add `<link rel="preload">` for critical fonts/CSS

### F3. Critical CSS extraction

**Current:** All CSS is inline in `<style>` blocks per page.

**Plan:**
1. Extract critical (above-the-fold) CSS inline
2. Move non-critical CSS to external file loaded asynchronously
3. Reduce per-page HTML size

---

## Part G: Accessibility

### G1. Skip-to-content link

**Plan:** Add `<a href="#main" class="sr-only focus:not-sr-only">Skip to content</a>` at
the top of every page.

### G2. Focus management

**Plan:**
1. When mobile sidebar opens, trap focus inside the sidebar
2. When sidebar closes, return focus to the toggle button
3. Add `aria-expanded` to sidebar toggle button

### G3. Keyboard shortcuts

**Plan:**
1. `Ctrl+K` / `Cmd+K` — open search
2. `Escape` — close search / close sidebar
3. Document shortcuts in an accessibility statement

---

## Part H: Future Features (Post-v1)

### H1. Blog system

Docusaurus-style blog with dated posts, archive pages, tag pages, and RSS feed.
Requires a `blog/` content directory, post frontmatter (date, author, tags), and
chronological listing templates.

### H2. Doc versioning

Serve multiple API versions side-by-side. Requires versioned content directories,
version dropdown in navbar, and versioned sidebar configs.

### H3. Plugin system

Extensible build pipeline with lifecycle hooks (pre-build, post-build, render).
Allow custom markdown extensions, custom templates, and custom page types.

### H4. MDX / Component system

Interactive components embedded in markdown. This is a significant architectural
change (requires a JS runtime in the generated output). Deferred to post-v1.

### H5. Translatable UI strings

Move hardcoded UI strings (prev/next, "On this page", search placeholder, etc.)
to a per-language translation file in `site.toml`.

---

## Implementation Priority

| Priority | Item | Effort | Impact |
|----------|------|--------|--------|
| P0 | A1: Fix root_prefix in all template links | Small | Fixes all subdirectory 404s |
| P0 | A2: Fix prev/next to follow sidebar order | Small | Fixes navigation flow |
| P0 | A3: Run Pagefind in build pipeline | Small | Search actually works |
| P1 | C1: Dark/light toggle button | Medium | Core UX feature |
| P1 | B1: Scroll-spy in sidebar | Small | Navigation polish |
| P1 | B2: TOC close button + scroll-spy | Small | Mobile UX |
| P1 | D1: Navbar search with Ctrl+K | Medium | Discoverability |
| P1 | B3: Hamburger menu in top nav | Small | Mobile navigation |
| P2 | E1: Code block line numbers + highlighting | Medium | Documentation quality |
| P2 | F1: Self-hosted assets | Medium | Reliability, offline support |
| P2 | B4: Multi-column footer | Small | Professional appearance |
| P2 | E3: Image improvements (WebP, lazy, srcset) | Medium | Performance |
| P2 | G1-G3: Accessibility (skip link, focus, keyboard) | Small | Compliance |
| P2 | E4: Admonition custom titles + collapsible | Small | Content flexibility |
| P2 | C2: Dark-mode color overrides | Small | Theme polish |
| P3 | E2: Content tabs | Medium | Content flexibility |
| P3 | F2-F3: Lazy loading, critical CSS | Medium | Performance |
| P3 | C3: Web font loading | Small | Typography |
| P3 | B5: Configurable breadcrumbs | Trivial | Flexibility |
| P4 | H1: Blog system | Large | New content type |
| P4 | H2: Doc versioning | Large | Multi-version support |
| P4 | H3: Plugin system | Large | Extensibility |
| P4 | H4: MDX/Component system | Very Large | Interactive docs |
| P4 | H5: Translatable UI strings | Medium | i18n completeness |

---

## Recommended Approach

Phase 1 (P0 — immediate): Fix all navigation bugs. Three small targeted changes to
`templates.rs` and `render.rs`. This unblocks the documentation site.

Phase 2 (P1 — next sprint): Dark/light toggle, scroll-spy, search UX, mobile hamburger menu.
These are the most visible missing features compared to Docusaurus/VitePress.

Phase 3 (P2 — following sprint): Code block polish, self-hosted assets, footer, images,
accessibility. Production-quality documentation site.

Phase 4 (P3+): Content tabs, performance optimization, blog, versioning, plugin system.
Advanced features.
