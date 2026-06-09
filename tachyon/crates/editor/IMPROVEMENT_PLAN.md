# Tachyon Editor Feature Gap Analysis & Improvement Plan

## Current State Summary

Tachyon editor: ~7,200 lines across 15 files. Rope-based buffer, line+col cursor, multi-cursor support, syntax highlighting (regex + tree-sitter), CRDT sync, undo/redo (1000-deep stack), find/replace (backend), line operations, auto-indent. Renders as absolutely-positioned DOM lines in a Leptos WASM component. Full visual verification via ImageMagick X11 window capture.

## Feature Gap Matrix

| Feature | Tachyon | Monaco | Neovim | JetBrains | Priority | Status |
|---------|---------|--------|--------|-----------|----------|--------|
| **Buffer: Rope** | ✅ | Piece table | B-tree | Chars seq | Done | ✅ |
| **Multi-cursor** | ✅ (struct exists, not wired in UI) | ✅ | ❌ (via macros) | ✅ | P1 | Backend done |
| **Undo/Redo** | ✅ (flat stack) | ✅ (grouped) | ✅ (tree) | ✅ (command groups) | P1 | Done |
| **Selection rendering** | ✅ (multi-line) | ✅ | ✅ | ✅ | P0 | ✅ Done |
| **Cursor: desired_col** | ✅ | ✅ | ✅ (curswant) | ✅ | P0 | ✅ Done |
| **Cursor: grapheme-aware** | ❌ (char-based) | ✅ (Intl.Segmenter) | ✌ (byte-based) | ✅ | P2 | — |
| **Line numbers** | ✅ | ✅ | ✅ | ✅ | Done | ✅ |
| **Word wrap** | ✅ (CSS word-wrap) | ✅ (view-aware) | ✅ | ✅ | P2 | ✅ Done |
| **Find/Replace panel** | ✅ (UI + backend) | ✅ (full panel) | ✅ (`/`+incsearch) | ✅ (multi-line regex) | P1 | ✅ Done |
| **Minimap** | ❌ | ✅ | ❌ | ✅ (error stripes) | P3 | — |
| **Code folding** | ❌ | ✅ | ✅ (`zc`/`zo`) | ✅ | P2 | — |
| **Bracket matching** | ✅ (highlight overlay) | ✅ (pair colorization) | ✅ (`%`) | ✅ (matched highlight) | P1 | ✅ Done |
| **Auto-close brackets** | ✅ (wired) | ✅ | ❌ | ✅ | P0 | ✅ Done |
| **Indent guides** | ✅ (subtle vertical lines) | ✅ | ❌ | ✅ (rainbow) | P1 | ✅ Done |
| **Sticky scroll** | ❌ | ✅ | ❌ | ✅ | P2 | — |
| **Clipboard (copy/cut/paste)** | ✅ (Ctrl+C/X/V) | ✅ | ✅ (registers) | ✅ | P0 | ✅ Done |
| **Scroll beyond last line** | ✅ (+200px padding) | ✅ | ❌ | ✅ | P1 | ✅ Done |
| **Smooth scrolling** | ❌ | ✅ | ❌ | ✅ | P2 | — |
| **Visible lines (dynamic)** | ✅ (ResizeObserver) | ✅ (computed) | ✅ | ✅ | P0 | ✅ Done |
| **Line operations** | ✅ (delete, duplicate, move, join) | ✅ | ✅ (dd, yy, p) | ✅ | Done | ✅ |
| **Comment toggle** | ✅ | ✅ | ✅ (gc) | ✅ | Done | ✅ |
| **Word navigation** | ✅ (Ctrl+Left/Right) | ✅ | ✅ (w/b/e) | ✅ (CamelHumps) | P1 | ✅ Done |
| **Word selection** | ✅ (Ctrl+Shift+Left/Right) | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Line selection** | ✅ (Ctrl+L) | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Document selection** | ✅ (Ctrl+Shift+Home/End) | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Mouse selection drag** | ✅ (mousedown/move/up) | ✅ | ❌ (visual mode) | ✅ | P1 | ✅ Done |
| **Double-click word select** | ✅ | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Triple-click line select** | ✅ | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Cursor blink** | ✅ (CSS animation) | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Status bar** | ✅ (Ln/Col, selection, language) | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Bracket matching highlight** | ✅ | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Join lines** | ✅ (Ctrl+J) | ✅ (Ctrl+J) | ✅ (J) | ✅ (Ctrl+Shift+J) | P1 | ✅ Done |
| **Wikilinks** | ✅ | ❌ | ❌ | ❌ | Done | ✅ |
| **CRDT sync** | ✅ (yrs) | ❌ | ❌ | ❌ | Done | ✅ |
| **Theme system** | ✅ (3 built-in) | ✅ (300+) | ✅ (highlights) | ✅ | Done | ✅ |
| **Ctrl+Z/Z undo** | ✅ | ✅ | ✅ | ✅ | Done | ✅ |
| **Ctrl+A select all** | ✅ | ✅ | ✅ | ✅ | Done | ✅ |
| **Find/Replace options** | ✅ (case, whole word, regex) | ✅ | ✅ | ✅ | P1 | ✅ Done |
| **Join lines** | ✅ | ✅ | ✅ (J) | ✅ (Ctrl+Shift+J) | P1 | ✅ Done |

## Prioritized Implementation Plan

### Phase 1: Essential Fixes & Core UX (P0) — COMPLETE ✅

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
