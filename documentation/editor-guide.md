---
title: Editor Guide
description: Keyboard shortcuts, features, and configuration for the native editor
order: 1
tags: [guide, editor]
---

# Editor Guide

Tachyon uses a native Rust editor (not CodeMirror or Monaco) built on `ropey` for text management and `yrs` for CRDT sync.

## Keyboard Shortcuts

### Navigation

| Shortcut | Action |
|----------|--------|
| `Ctrl+G` / `Cmd+G` | Go to line |
| `Ctrl+F` / `Cmd+F` | Find in document |
| `Ctrl+H` / `Cmd+H` | Find and replace |
| `Ctrl+Home` | Jump to start of document |
| `Ctrl+End` | Jump to end of document |

### Editing

| Shortcut | Action |
|----------|--------|
| `Tab` | Insert indent |
| `Shift+Tab` | Outdent |
| `Ctrl+Z` / `Cmd+Z` | Undo |
| `Ctrl+Shift+Z` / `Cmd+Shift+Z` | Redo |
| `Ctrl+A` / `Cmd+A` | Select all |
| `Ctrl+D` / `Cmd+D` | Delete line |

### Markdown Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` / `Cmd+B` | Bold (`**text**`) |
| `Ctrl+I` / `Cmd+I` | Italic (`*text*`) |
| `Ctrl+K` / `Cmd+K` | Insert link |
| `Ctrl+Shift+K` | Insert code block |

## Features

### Syntax Highlighting

The editor provides syntax highlighting for:

- Markdown headings, bold, italic, links, code
- Code blocks with language detection
- Lists (ordered and unordered)
- Blockquotes and tables

### Real-time Collaboration

When multiple users edit the same document, changes are synced in real-time via WebSocket using Yrs (Yjs Rust port):

- Character-level conflict resolution
- Cursor presence (see other users' cursors)
- No merge conflicts — CRDTs handle concurrent edits

### Search

Press `Ctrl+F` to open the search panel:

- **Case sensitive** toggle (`Aa`)
- **Whole word** match toggle (`W`)
- **Regex** mode toggle (`.*`)
- Navigate matches with arrow buttons or `Enter`/`Shift+Enter`
- Replace single or all matches

## Markdown Support

Tachyon supports CommonMark markdown with extensions:

- Tables
- Task lists (`- [x] done`)
- Footnotes
- Strikethrough (`~~text~~`)
- Highlight (`==text==`)
- Math (basic inline)
- Mermaid diagrams (via plugin)
