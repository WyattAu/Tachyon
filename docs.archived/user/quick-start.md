# Quick Start Tutorial

Get up and running with Tachyon in 5 minutes.

## Prerequisites

- Tachyon installed (see [Installation Guide](installation.md))
- A Git repository with markdown files (or create a new one)

## Step 1: Launch Tachyon

### Desktop

Open Tachyon from your applications menu or run:
```bash
tachyon
```

### Server

```bash
tachyon serve --port 8080
```

Then open http://localhost:8080 in your browser.

## Step 2: Open a Repository

### Desktop

1. Click **File > Open Repository**
2. Navigate to your documentation folder
3. Click **Open**

Or use the command line:
```bash
tachyon /path/to/your/docs
```

### Server

Point Tachyon to your repository:
```bash
tachyon serve --port 8080 --path /path/to/your/docs
```

## Step 3: Create Your First Document

1. Click the **+** button in the sidebar or press `Ctrl+N` / `Cmd+N`
2. Enter a title for your document
3. Start writing in Markdown

```markdown
# My First Document

Welcome to **Tachyon**!

## Features

- Real-time preview
- Git integration
- Full-text search

## Code Example

```rust
fn main() {
    println!("Hello, Tachyon!");
}
```

## Math Support

Inline math: $E = mc^2$

Block math:
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

## Step 4: Real-Time Preview

As you type, the preview updates automatically. Notice:
- Headings render instantly
- Code blocks get syntax highlighting
- Math equations render with KaTeX

## Step 5: Use External Editor

Tachyon supports BYOE (Bring Your Own Editor):

1. Open the same file in VS Code, Neovim, or your preferred editor
2. Edit and save in your editor
3. Watch Tachyon update instantly

No file locks - both editors work simultaneously.

## Step 6: Search Your Documents

1. Press `Ctrl+K` / `Cmd+K` to open search
2. Type your query
3. Results appear as you type with sub-100ms response

Search supports:
- Full-text search
- Tag filtering (`tag:documentation`)
- Date ranges (`created:>2024-01-01`)

## Step 7: Commit Changes

### Desktop

1. Click the **Git** icon in the sidebar
2. Review your changes
3. Enter a commit message
4. Click **Commit**

Or use your external Git workflow - Tachyon detects all changes.

### Server

Changes are automatically committed with timestamps. Access history via the document menu.

## Step 8: Collaborate (Server Mode)

If running in server mode:

1. Share the URL with team members
2. See real-time cursors and presence
3. Collaborate on documents simultaneously

## Common Actions

| Action | Desktop Shortcut | Web |
|--------|-----------------|-----|
| New document | `Ctrl+N` | Click **+** |
| Save | `Ctrl+S` | Auto-saved |
| Search | `Ctrl+K` | Click search icon |
| Toggle preview | `Ctrl+P` | N/A |
| Toggle sidebar | `Ctrl+B` | Click hamburger |

## Configuration Basics

Create `tachyon.toml` in your repository root:

```toml
[rendering]
syntax_theme = "dark"      # light | dark | high-contrast
enable_diagrams = true     # Mermaid.js support
math_engine = "katex"      # katex | mathjax

[editor]
font_family = "JetBrains Mono"
font_size = 14
line_numbers = true
```

## What's Next?

- [Features Overview](features.md) - Learn about all capabilities
- [Document Management](documents.md) - Advanced document features
- [Search Guide](search.md) - Master the search functionality
- [Collaboration](collaboration.md) - Team collaboration features

## Troubleshooting

### File not updating?

1. Check if file watcher is running (green indicator in status bar)
2. Verify file permissions
3. Try manual refresh: `View > Refresh`

### Search not finding documents?

1. Wait for indexing to complete
2. Check file extensions (`.md`, `.markdown`)
3. Rebuild index: `Tools > Rebuild Index`

### Preview not rendering?

1. Check for Markdown syntax errors
2. View console for errors: `Help > Toggle Developer Tools`
3. Try disabling custom CSS
