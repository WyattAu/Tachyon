# Tachyon GUI Refactor Roadmap

## Current Stack

| Layer | Technology | Status |
|-------|-----------|--------|
| Desktop shell | Tauri v2 | Working |
| Frontend framework | Leptos 0.8 (CSR) | Working |
| Build | Trunk | Working |
| Styling | Tailwind CSS (CDN) | Working (no HMR) |
| Icons | Emoji/unicode literals | Ugly |
| Components | All hand-built | Tedious |
| Keybindings | Hardcoded match blocks | Just refactored to shared fn |
| Animations | None | Missing |
| Dark mode | Manual class toggle | Fragile |
| i18n | None | Missing |
| Notifications | None | Missing |

## Ecosystem Analysis

### Tier 1: Adopt Now (High Impact, Low Risk)

#### 1. `lepticons` — Lucide icons for Leptos
**What:** Lucide icon toolkit with searchable picker, stroke draw-in animations, tree-shaking.
**Why:** Our toolbar uses emoji characters (link, image, compass) which render inconsistently across platforms and look unprofessional. Lucide provides 1500+ consistent SVG icons.
**Impact:** Toolbar, sidebar, buttons, status bar — everywhere we use text/emoji for icons.
**Risk:** Low — drop-in component replacement, no API changes.
**Effort:** 1-2 days. Replace emoji literals with `<Icon icon=LucideBold />` components.

#### 2. `leptos-use` — Reactive primitives
**What:** `useEventListener`, `useResizeObserver`, `useLocalStorage`, `useMediaQuery`, `useDarkMode`, etc.
**Why:** We manually implement:
- `ResizeObserver` (our `on_mounted` handler) → `useResizeObserver`
- Global `window.addEventListener("keydown", ...)` with Closure+forget → `useEventListener`
- Manual localStorage reads in `ApiClient::default()` → `useLocalStorage`
- Dark mode detection → `useMediaQuery("(prefers-color-scheme: dark)")`
**Impact:** Eliminates ~200 lines of manual WASM interop code, prevents memory leaks (closures that `forget()` are never freed).
**Risk:** Low — well-maintained, pure additive.
**Effort:** 2-3 days. Migrate one primitive at a time.

#### 3. `leptos-darkmode` — Dark mode management
**What:** Adds `dark` class to `<html>` based on localStorage or system preference.
**Why:** Our dark mode is fragile — we manually check `dark:` Tailwind classes. This package handles the toggle, persistence, and system preference detection.
**Impact:** All pages with dark/light mode styling.
**Risk:** Low — standard Tailwind integration.
**Effort:** Half day.

### Tier 2: Adopt Soon (High Impact, Medium Risk)

#### 4. `leptos-hotkeys` — Declarative keybindings
**What:** Declaratively bind key combinations to callbacks.
**Why:** We just refactored to a shared `handle_editor_key()` function. This package goes further — lets you define bindings as data, not code. Supports `when` conditions (e.g., "only when editor focused").
**Impact:** Editor keybindings, global app shortcuts.
**Risk:** Medium — need to verify it handles our WebKitGTK quirks (global listener, `prevent_default`).
**Effort:** 2-3 days. Replace the shared function with declarative bindings.

#### 5. `leptos_toaster` — Toast notifications
**What:** Sonner-inspired toast component for Leptos.
**Why:** We have no notification system. Users need feedback for save, copy, errors, etc.
**Impact:** Every user-facing action.
**Risk:** Low — self-contained component.
**Effort:** 1 day. Add `<Toaster />` + `toast.success("Saved")` calls.

#### 6. `thaw` — Component library
**What:** Full Leptos component library (Button, Input, Modal, Tabs, Tooltip, etc.).
**Why:** We hand-built: toolbar buttons, find panel inputs, sidebar tabs, modals. Thaw provides production-tested components with accessibility built in.
**Impact:** Find/Replace panel, settings dialogs, any future UI.
**Risk:** Medium — need to verify Tailwind integration, may conflict with our custom CSS.
**Effort:** 3-5 days for full adoption. Start with find panel components.

#### 7. `leptos-animate` — Animations
**What:** FLIP transitions, CSS in/out transitions, easing utilities.
**Why:** We have no animations. Cursor blink is CSS-only, no scroll animation, no panel transitions.
**Impact:** Smooth scroll, find panel open/close, toast slide-in, panel transitions.
**Risk:** Low — purely additive.
**Effort:** 1-2 days.

### Tier 3: Evaluate (Medium Impact, Higher Risk)

#### 8. `tauri-plugin-screenshots` — Native screenshot API
**What:** Tauri plugin for window/monitor screenshots.
**Why:** We built a custom ImageMagick `import` solution. This plugin provides native screenshots via the OS API.
**Impact:** Traversal test screenshots, potential user-facing screenshot feature.
**Risk:** Medium — need to verify WebKitGTK rendering is captured (not just the window shell).
**Effort:** 1 day to test and potentially replace our `import` hack.

#### 9. `tauri-plugin-clipboard` — Native clipboard
**What:** Full clipboard support (text, image, HTML, RTF, file monitoring).
**Why:** We use `navigator.clipboard` via JS eval. The native plugin is more reliable and works with the system clipboard manager.
**Impact:** Copy/Cut/Paste, potential future features (copy image, copy formatted text).
**Risk:** Low-medium — plugin is well-maintained.
**Effort:** 1 day. Replace `js_sys::eval("navigator.clipboard...")` with Tauri command.

#### 10. `leptos-fetch` — Async data fetching
**What:** Cache management for server data, SWR-like patterns.
**Why:** Our `ApiClient` does manual fetch+error handling. This provides caching, deduplication, stale-while-revalidate.
**Impact:** Document list, dashboard stats, all API calls.
**Risk:** Medium — may conflict with our CRDT sync model.
**Effort:** 2-3 days. Evaluate for read-heavy pages (documents list, dashboard).

#### 11. `leptos-tracked` — Signal composition
**What:** Utility traits for composing signals with fewer nested closures.
**Why:** Our render closures are deeply nested with `move || { ... let x = signal.get(); ... }`.
**Impact:** Code readability across all pages.
**Risk:** Low — utility library, no runtime changes.
**Effort:** 1 day to evaluate, gradual adoption.

### Tier 4: Skip for Now (Low Impact or Not Applicable)

| Package | Reason to skip |
|---------|---------------|
| `leptos_i18n` | No immediate internationalization needs |
| `leptos-leaflet` / `leptos_maplibre` | No map features planned |
| `leptos_pdf` | No PDF viewing needed |
| `leptos-content-collection` | No static site generation |
| `leptos-tea` | We use signals, not TEA architecture |
| `leptos-server-signal` / `leptos_ws` | We have CRDT sync, not server-pushed signals |
| `leptos-captcha` | Not a web application |
| `leptos-chartistry` | No charting needs currently |
| `leptos_drag_reorder` | Not needed yet (could be for sidebar reordering) |
| `leptos-image` | Server-side only, we're CSR |
| `turf` (SCSS) | We use Tailwind, not SCSS |
| `Stylance` / `Stylers` | We use Tailwind, not CSS modules |
| `leptos-signals` | Already in Leptos core |

### Tauri Plugins — Adopt

| Plugin | Why |
|--------|-----|
| `tauri-plugin-notification` (official) | Replace custom notification handling |
| `tauri-plugin-clipboard` | Replace JS eval clipboard |
| `tauri-plugin-screenshots` | Replace ImageMagick `import` hack |
| `sentry-tauri` | Crash reporting (production) |
| `tauri-plugin-aptabase` | Privacy-first analytics (production) |

### Tauri Plugins — Skip

| Plugin | Reason |
|--------|--------|
| All mobile plugins | Desktop-only app |
| All crypto/wallet plugins | Not a financial app |
| `tauri-plugin-blec` | No BLE needs |
| `tauri-plugin-graphql` | We use REST API |
| `tauri-plugin-python` | All Rust backend |

## Migration Roadmap

### Phase 1: Icons & Dark Mode (Week 1)
**Goal:** Replace emoji with Lucide icons, fix dark mode.

| Task | Package | Files changed | Effort |
|------|---------|--------------|--------|
| Replace toolbar emoji with Lucide icons | `lepticons` | `editor_toolbar.rs` | 0.5d |
| Replace sidebar icons with Lucide | `lepticons` | `sidebar.rs` or equivalent | 0.5d |
| Replace button icons (new doc, search, etc.) | `lepticons` | Multiple pages | 0.5d |
| Add `leptos-darkmode` | `leptos-darkmode` | `lib.rs`, `index.html` | 0.5d |
| Remove manual dark mode toggle code | — | `settings.rs` | 0.5d |
| Add `leptos_toaster` | `leptos_toaster` | `lib.rs` | 0.5d |
| Wire toast for save/copy/error actions | — | `editor_toolbar.rs`, `document_edit_page.rs` | 0.5d |

**Verification:** Traversal test shows correct icons in all screenshots.

### Phase 2: Reactive Primitives & Animations (Week 2)
**Goal:** Eliminate manual WASM interop, add polish.

| Task | Package | Files changed | Effort |
|------|---------|--------------|--------|
| Replace manual ResizeObserver with `useResizeObserver` | `leptos-use` | `native_editor.rs` | 0.5d |
| Replace manual keydown listener with `useEventListener` | `leptos-use` | `native_editor.rs` | 1d |
| Replace localStorage reads with `useLocalStorage` | `leptos-use` | `api/mod.rs` | 0.5d |
| Add smooth scroll animation | `leptos-animate` | `native_editor.rs` | 0.5d |
| Add panel transition animations | `leptos-animate` | Find panel, modals | 0.5d |

**Verification:** No `Closure::forget()` calls in editor code. Memory usage stable during long sessions.

### Phase 3: Component Library (Week 3)
**Goal:** Adopt `thaw` for production-quality components.

| Task | Package | Files changed | Effort |
|------|---------|--------------|--------|
| Add `thaw` dependency, verify build | `thaw` | `Cargo.toml` | 0.5d |
| Replace find panel inputs with `thaw::Input` | `thaw` | `native_editor.rs` | 0.5d |
| Replace sidebar tabs with `thaw::Tabs` | `thaw` | `documents.rs` | 0.5d |
| Replace toolbar buttons with `thaw::Button` | `thaw` | `editor_toolbar.rs` | 1d |
| Add `thaw::Modal` for settings | `thaw` | Settings page | 0.5d |
| Add `thaw::Tooltip` for toolbar hover hints | `thaw` | `editor_toolbar.rs` | 0.5d |

**Verification:** Traversal test passes. Tab navigation works. Tooltips appear on hover.

### Phase 4: Keybinding Refactor (Week 4)
**Goal:** Declarative keybinding system.

| Task | Package | Files changed | Effort |
|------|---------|--------------|--------|
| Evaluate `leptos-hotkeys` compatibility | `leptos-hotkeys` | — | 0.5d |
| Migrate editor keybindings to declarative format | `leptos-hotkeys` | `native_editor.rs` | 2d |
| Migrate global shortcuts (Ctrl+N, Ctrl+S, etc.) | `leptos-hotkeys` | `lib.rs` | 1d |

**Verification:** All existing keybindings work. New shortcuts can be added by editing a config, not code.

### Phase 5: Tauri Plugin Migration (Week 5)
**Goal:** Replace JS eval hacks with native plugins.

| Task | Package | Files changed | Effort |
|------|---------|--------------|--------|
| Add `tauri-plugin-clipboard`, replace eval | `tauri-plugin-clipboard` | `native_editor.rs`, `Cargo.toml` | 1d |
| Test `tauri-plugin-screenshots` for traversal | `tauri-plugin-screenshots` | `commands.rs` | 1d |
| Add `sentry-tauri` for crash reporting | `sentry-tauri` | `lib.rs` | 0.5d |

**Verification:** Copy/Cut/Paste work without JS eval. Screenshots captured natively. Crashes reported to Sentry.

### Phase 6: Async Data & Polish (Week 6)
**Goal:** Production-quality data layer.

| Task | Package | Files changed | Effort |
|------|---------|--------------|--------|
| Evaluate `leptos-fetch` for document list | `leptos-fetch` | `documents.rs` | 0.5d |
| Evaluate `leptos-tracked` for closure cleanup | `leptos-tracked` | Multiple pages | 0.5d |
| Add `leptos_hotkeys` for find-next/prev keyboard shortcuts | `leptos-hotkeys` | `native_editor.rs` | 0.5d |
| Final cleanup: remove all manual WASM interop | — | All files | 1d |

**Verification:** Full traversal test. Memory leak check. Code review.

## Decision: Migrate vs. Keep

| Decision | Verdict | Rationale |
|----------|---------|-----------|
| Keep Leptos | Keep | Best Rust WASM framework. CSR mode works well. No reason to switch. |
| Keep Tauri | Keep | Best Rust desktop shell. Plugin ecosystem growing fast. |
| Keep Trunk | Keep | Works, no HMR issues to solve (Trunk is the build tool, not the HMR provider) |
| Adopt Tailwind fully | Adopt | Already using it via CDN. Switch to proper PostCSS build for HMR. |
| Adopt `thaw` | Adopt | Saves weeks of component building. Accessibility built in. |
| Adopt `lepticons` | Adopt | Immediate visual improvement. Zero risk. |
| Adopt `leptos-use` | Adopt | Eliminates memory leak vectors. Modern patterns. |
| Adopt `leptos-hotkeys` | Evaluate | Need to verify WebKitGTK compatibility before full adoption. |
| Replace ImageMagick | Evaluate | Plugin may not capture WebKitGTK rendering. Test first. |
| SSR migration | Not now | CSR works fine for desktop. SSR adds complexity without benefit. |

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| `thaw` CSS conflicts with Tailwind | Medium | Use `thaw` in isolated components first, test on all pages |
| `leptos-hotkeys` doesn't work with WebKitGTK global listener | High | Fallback to our refactored shared function |
| `tauri-plugin-screenshots` doesn't capture WebKitGTK content | Medium | Keep ImageMagick fallback |
| `leptos-use` memory leak with closures | Low | Test with long-running sessions |
| `leptos-animate` conflicts with CSS `scroll-behavior: smooth` | Low | Use one or the other, not both |
| Plugin compatibility with Tauri v2 | Low | Only use v2-compatible plugins |

## Success Metrics

| Metric | Before | Target |
|--------|--------|--------|
| Emoji icons | 15+ | 0 |
| `Closure::forget()` calls | 3 | 0 |
| Lines of manual WASM interop | ~200 | <20 |
| Component library coverage | 0% | 60%+ |
| Toast notifications | 0 | All user actions |
| Keybinding definitions | 200-line match block | Declarative table |
| Memory leaks (30-min session) | Unknown | 0 |
