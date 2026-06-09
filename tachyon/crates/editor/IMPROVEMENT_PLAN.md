# Tachyon Editor Feature Gap Analysis & Improvement Plan

## Current State Summary

Tachyon editor: ~7,200 lines across 15 files. Rope-based buffer, line+col cursor, multi-cursor support, syntax highlighting (regex + tree-sitter), CRDT sync, undo/redo (1000-deep stack), find/replace (backend), line operations, auto-indent. Renders as absolutely-positioned DOM lines in a Leptos WASM component. Full visual verification via ImageMagick X11 window capture.

## Feature Gap Matrix

| Feature | Tachyon | Monaco | Neovim | JetBrains | Priority | Status |
|---------|---------|--------|--------|-----------|----------|--------|
| **Buffer: Rope** | [x] | Piece table | B-tree | Chars seq | Done | [x] |
| **Multi-cursor** | [x] (struct exists, not wired in UI) | [x] | [ ] (via macros) | [x] | P1 | Backend done |
| **Undo/Redo** | [x] (flat stack) | [x] (grouped) | [x] (tree) | [x] (command groups) | P1 | Done |
| **Selection rendering** | [x] (multi-line) | [x] | [x] | [x] | P0 | [x] Done |
| **Cursor: desired_col** | [x] | [x] | [x] (curswant) | [x] | P0 | [x] Done |
| **Cursor: grapheme-aware** | [ ] (char-based) | [x] (Intl.Segmenter) | [~] (byte-based) | [x] | P2 | — |
| **Line numbers** | [x] | [x] | [x] | [x] | Done | [x] |
| **Word wrap** | [x] (CSS word-wrap) | [x] (view-aware) | [x] | [x] | P2 | [x] Done |
| **Find/Replace panel** | [x] (UI + backend) | [x] (full panel) | [x] (`/`+incsearch) | [x] (multi-line regex) | P1 | [x] Done |
| **Minimap** | [ ] | [x] | [ ] | [x] (error stripes) | P3 | — |
| **Code folding** | [ ] | [x] | [x] (`zc`/`zo`) | [x] | P2 | — |
| **Bracket matching** | [x] (highlight overlay) | [x] (pair colorization) | [x] (`%`) | [x] (matched highlight) | P1 | [x] Done |
| **Auto-close brackets** | [x] (wired) | [x] | [ ] | [x] | P0 | [x] Done |
| **Indent guides** | [x] (subtle vertical lines) | [x] | [ ] | [x] (rainbow) | P1 | [x] Done |
| **Sticky scroll** | [ ] | [x] | [ ] | [x] | P2 | — |
| **Clipboard (copy/cut/paste)** | [x] (Ctrl+C/X/V) | [x] | [x] (registers) | [x] | P0 | [x] Done |
| **Scroll beyond last line** | [x] (+200px padding) | [x] | [ ] | [x] | P1 | [x] Done |
| **Smooth scrolling** | [ ] | [x] | [ ] | [x] | P2 | — |
| **Visible lines (dynamic)** | [x] (ResizeObserver) | [x] (computed) | [x] | [x] | P0 | [x] Done |
| **Line operations** | [x] (delete, duplicate, move, join) | [x] | [x] (dd, yy, p) | [x] | Done | [x] |
| **Comment toggle** | [x] | [x] | [x] (gc) | [x] | Done | [x] |
| **Word navigation** | [x] (Ctrl+Left/Right) | [x] | [x] (w/b/e) | [x] (CamelHumps) | P1 | [x] Done |
| **Word selection** | [x] (Ctrl+Shift+Left/Right) | [x] | [x] | [x] | P1 | [x] Done |
| **Line selection** | [x] (Ctrl+L) | [x] | [x] | [x] | P1 | [x] Done |
| **Document selection** | [x] (Ctrl+Shift+Home/End) | [x] | [x] | [x] | P1 | [x] Done |
| **Mouse selection drag** | [x] (mousedown/move/up) | [x] | [ ] (visual mode) | [x] | P1 | [x] Done |
| **Double-click word select** | [x] | [x] | [x] | [x] | P1 | [x] Done |
| **Triple-click line select** | [x] | [x] | [x] | [x] | P1 | [x] Done |
| **Cursor blink** | [x] (CSS animation) | [x] | [x] | [x] | P1 | [x] Done |
| **Status bar** | [x] (Ln/Col, selection, language) | [x] | [x] | [x] | P1 | [x] Done |
| **Bracket matching highlight** | [x] | [x] | [x] | [x] | P1 | [x] Done |
| **Join lines** | [x] (Ctrl+J) | [x] (Ctrl+J) | [x] (J) | [x] (Ctrl+Shift+J) | P1 | [x] Done |
| **Wikilinks** | [x] | [ ] | [ ] | [ ] | Done | [x] |
| **CRDT sync** | [x] (yrs) | [ ] | [ ] | [ ] | Done | [x] |
| **Theme system** | [x] (3 built-in) | [x] (300+) | [x] (highlights) | [x] | Done | [x] |
| **Ctrl+Z/Z undo** | [x] | [x] | [x] | [x] | Done | [x] |
| **Ctrl+A select all** | [x] | [x] | [x] | [x] | Done | [x] |
| **Find/Replace options** | [x] (case, whole word, regex) | [x] | [x] | [x] | P1 | [x] Done |
| **Join lines** | [x] | [x] | [x] (J) | [x] (Ctrl+Shift+J) | P1 | [x] Done |

## Prioritized Implementation Plan

### Phase 1: Essential Fixes & Core UX (P0) — COMPLETE [x]

- [x] 1.1 Fix multi-line selection rendering
- [x] 1.2 Add `desired_col` to cursor
- [x] 1.3 Dynamic visible_lines (ResizeObserver)
- [x] 1.4 Wire auto-close brackets
- [x] 1.5 Implement clipboard (Ctrl+C/X/V)
- [x] 1.6 Scroll-beyond-last-line (+200px)

### Phase 2: Professional Feel (P1) — MOSTLY COMPLETE

- [x] 2.1 Cursor blink animation (CSS)
- [x] 2.2 Selection drag (mouse)
- [x] 2.3 Status bar (Ln/Col, language, word count)
- [x] 2.4 Find/Replace panel UI (Ctrl+F, search, replace, case/whole-word toggles)
- [x] 2.5 Bracket matching highlight
- [x] 2.6 Indent guides
- [x] 2.7 Expose search options (case sensitivity, whole word, regex setters)
- [ ] 2.8 Keybinding abstraction layer (replace hardcoded match)
- [x] 2.9 Word navigation (Ctrl+Left/Right, Ctrl+Shift+Left/Right)
- [x] 2.10 Document selection (Ctrl+Shift+Home/End)
- [x] 2.11 Join lines (Ctrl+J)

### Phase 3: Advanced Features (P2) — NOT STARTED

- [ ] 3.1 Code folding
- [ ] 3.2 Sticky scroll (pin function signatures at top)
- [ ] 3.3 Word wrap with indent continuation (view-aware)
- [ ] 3.4 Smooth scrolling
- [ ] 3.5 Grapheme-aware cursor movement
- [ ] 3.6 Undo grouping (temporal)
- [ ] 3.7 Auto-scroll to cursor on navigation

### Phase 4: Nice-to-Have (P3) — NOT STARTED

- [ ] 4.1 Minimap
- [ ] 4.2 Breadcrumbs
- [ ] 4.3 Inlay hints
- [ ] 4.4 Multi-cursor UI wiring (Ctrl+D, Alt+Click)
- [ ] 4.5 Soft wrap with visual line navigation
- [ ] 4.6 Drag-and-drop text reordering
- [ ] 4.7 Block selection (Ctrl+Shift+B / column selection)

## Verified Via

- 19/19 route traversal with 0 panics
- ImageMagick X11 window capture (actual pixel verification)
- DOM dump layout analysis (structural verification)
- Editor content: "Hello from traverse! Second line." visible in X11 screenshot
- All toolbar buttons, status bar, sidebar tabs rendered correctly
