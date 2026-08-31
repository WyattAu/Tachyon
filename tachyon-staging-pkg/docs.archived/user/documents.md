# Document Management

Complete guide to creating, editing, and managing documents in Tachyon.

## Creating Documents

### From the UI

1. Click the **+** button in the sidebar
2. Choose a location for the document
3. Enter a title
4. Start writing

### From File System

Create a new `.md` file in your repository:
```bash
echo "# My New Document" > docs/new-doc.md
```

Tachyon automatically detects and indexes the file.

### Using Templates

Create reusable templates in `.tachyon/templates/`:

```markdown
---
template: meeting-notes
---

# Meeting: {{title}}
Date: {{date}}
Attendees:

## Agenda
1.
2.
3.

## Discussion Notes

## Action Items
| Owner | Task | Due |
|-------|------|-----|
|       |      |     |
```

Use templates when creating new documents.

## Document Structure

### Frontmatter

YAML metadata at the beginning of documents:

```yaml
---
title: API Reference
description: Complete API documentation
author: Jane Doe
date: 2024-01-15
tags: [api, reference, v2]
status: published
visibility: public
access: restricted
groups: [developers, admins]
---
```

**Required fields:**
- `title`: Document title

**Optional fields:**
- `description`: Brief summary
- `author`: Document author
- `date`: Creation or update date
- `tags`: Array of tags
- `status`: draft | published | archived
- `visibility`: public | private | restricted
- `access`: public | restricted | internal
- `groups`: Array of group names with access

### Body Content

Standard Markdown after frontmatter:

```markdown
# Heading 1

Content here...

## Heading 2

More content...
```

## Editing Documents

### Built-in Editor

Tachyon includes a full-featured editor:

| Feature | Description |
|---------|-------------|
| Syntax highlighting | Markdown and code blocks |
| Live preview | Real-time rendering |
| Auto-save | Continuous saving |
| Word count | Character and word statistics |
| Spell check | Browser-based spell checking |

### External Editors

Use any external editor:

1. Open document in Tachyon
2. Click **Open in Editor** or use your IDE
3. Edit and save externally
4. Tachyon updates automatically

Supported editors:
- VS Code
- Neovim/Vim
- JetBrains IDEs
- Sublime Text
- Any editor that saves to disk

### Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| Save | `Ctrl+S` | `Cmd+S` |
| Bold | `Ctrl+B` | `Cmd+B` |
| Italic | `Ctrl+I` | `Cmd+I` |
| Link | `Ctrl+K` | `Cmd+K` |
| Code | `Ctrl+`` ` | `Cmd+`` ` |
| Heading | `Ctrl+1-6` | `Cmd+1-6` |
| List | `Ctrl+Shift+L` | `Cmd+Shift+L` |

## Document Organization

### Folders

Organize documents in folders:

```
docs/
├── getting-started/
│   ├── installation.md
│   └── quick-start.md
├── features/
│   ├── search.md
│   └── collaboration.md
└── api/
    ├── endpoints.md
    └── authentication.md
```

### Tags

Add tags for cross-cutting organization:

```yaml
---
tags: [api, authentication, security]
---
```

Search by tag: `tag:authentication`

### Collections

Group related documents:

```yaml
---
collection: api-guide
order: 3
---
```

Collections appear in navigation with ordering.

## Document Status

### Lifecycle States

| Status | Description |
|--------|-------------|
| `draft` | Work in progress |
| `published` | Visible to all users |
| `archived` | Historical, not in search |
| `deleted` | Soft delete, recoverable |

### Status Transitions

```yaml
---
status: draft
---

# Move to published when ready
```

Documents in `draft` status:
- Not visible to viewers
- Not included in search
- Can be previewed with direct link

## Version History

### Viewing History

1. Open document
2. Click **History** in toolbar
3. Browse previous versions

### Comparing Versions

1. Select two versions
2. Click **Compare**
3. View diff with additions/deletions

### Restoring Versions

1. Open a previous version
2. Click **Restore this version**
3. Confirm restoration

## Document Operations

### Move

Move documents between folders:

1. Right-click document
2. Select **Move**
3. Choose destination

Or use drag-and-drop in the sidebar.

### Duplicate

Create a copy:

1. Right-click document
2. Select **Duplicate**
3. Edit the copy

### Delete

Soft delete documents:

1. Right-click document
2. Select **Delete**
3. Confirm deletion

Deleted documents go to trash and can be restored within 30 days.

### Export

Export to various formats:

| Format | Command |
|--------|---------|
| HTML | `File > Export > HTML` |
| PDF | `File > Export > PDF` |
| Markdown | `File > Export > Markdown` |
| JSON | `File > Export > JSON` |

## Special Features

### Internal Links

Link to other documents:

```markdown
See [Installation Guide](./installation.md) for setup instructions.
```

Relative paths work from the current document's location.

### Embeds

Embed content from other documents:

```markdown
{{embed ./partials/header.md}}
```

### Table of Contents

Auto-generated from headings:

```markdown
{{toc}}
```

Or configure in frontmatter:

```yaml
---
toc: true
toc_depth: 3
---
```

### Custom Blocks

Use custom block types:

```markdown
::: tip
This is a helpful tip.
:::

::: warning
This is a warning.
:::

::: danger
This is important.
:::

::: internal
Only visible to internal team members.
:::
```

### Code Groups

Group code blocks with tabs:

````markdown
```rust
fn main() {
    println!("Hello");
}
```

```python
def main():
    print("Hello")
```
````

## Best Practices

### File Naming

- Use lowercase with hyphens: `my-document.md`
- Be descriptive: `api-authentication.md`
- Avoid special characters

### Structure

- One main heading (`#`) per document
- Use heading hierarchy (H1 → H2 → H3)
- Keep sections focused

### Metadata

- Always include `title` in frontmatter
- Add `description` for SEO
- Tag consistently

### Content

- Write clear, concise prose
- Use code blocks for examples
- Include diagrams for complex concepts
- Link to related documents
