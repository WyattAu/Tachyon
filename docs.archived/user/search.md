# Search Functionality

Complete guide to Tachyon's full-text search capabilities.

## Overview

Tachyon provides sub-100ms full-text search powered by Tantivy. Search indexes your entire document repository with support for:

- Full-text queries
- Field-specific filters
- Boolean operators
- Fuzzy matching
- Phrase search

## Basic Search

### Quick Search

Press `Ctrl+K` (Windows/Linux) or `Cmd+K` (macOS) to open the search bar.

Type your query and results appear instantly.

### Search Bar

The search bar provides:
- Instant results as you type
- Highlighted matches
- Document previews
- Keyboard navigation

### Results Display

Each result shows:
- Document title
- Matching excerpt (highlighted)
- File path
- Last modified date

## Query Syntax

### Simple Terms

```
authentication
```

Finds documents containing "authentication".

### Multiple Terms

```
api authentication
```

Finds documents containing BOTH terms (implicit AND).

### Phrase Search

```
"exact phrase"
```

Finds documents with the exact phrase in quotes.

### Boolean Operators

| Operator | Example | Result |
|----------|---------|--------|
| AND | `api AND authentication` | Both terms required |
| OR | `api OR rest` | Either term |
| NOT | `api NOT graphql` | First term, exclude second |
| - | `api -graphql` | Exclude term |

### Wildcards

| Wildcard | Example | Matches |
|----------|---------|---------|
| `*` | `auth*` | authenticate, authentication, author |
| `?` | `te?t` | text, test |

### Fuzzy Search

```
authentikation~
```

Finds similar terms (typo-tolerant). Add a number for specificity:

```
authentikation~2
```

Higher numbers allow more differences.

### Range Queries

```
created:[2024-01-01 TO 2024-12-31]
```

Finds documents created in 2024.

## Field Filters

Search specific document fields:

### Available Fields

| Field | Example |
|-------|---------|
| `title` | `title:api` |
| `author` | `author:john` |
| `tag` | `tag:documentation` |
| `status` | `status:published` |
| `created` | `created:>2024-01-01` |
| `modified` | `modified:<2024-06-01` |
| `path` | `path:docs/api` |

### Examples

Documents by specific author:
```
author:"Jane Doe"
```

Published documents with "api" in title:
```
status:published AND title:api
```

Recent documents:
```
modified:>2024-01-15
```

Documents in specific folder:
```
path:docs/api
```

Multiple tags:
```
(tag:api OR tag:rest) AND status:published
```

## Advanced Queries

### Combining Filters

```
status:published AND (tag:api OR tag:reference) AND created:>2024-01-01
```

### Complex Boolean

```
(api OR rest OR graphql) NOT deprecated
```

### Exact Field Match

```
title:"API Reference"
```

### Date Ranges

```
created:[2024-01-01 TO 2024-03-31]
modified:{2024-01-01 TO 2024-12-31}
```

`[]` = inclusive, `{}` = exclusive

## Search Results

### Sorting

Sort results by:
- Relevance (default)
- Date created
- Date modified
- Title

### Pagination

Results are paginated (20 per page by default). Use arrow keys or click to navigate.

### Opening Results

- **Enter**: Open in current tab
- **Ctrl+Enter** / **Cmd+Enter**: Open in new tab
- **Click**: Open in current tab

## Search Settings

### Configuration

```toml
[search]
results_per_page = 20
highlight_matches = true
fuzzy_threshold = 2
index_delay_ms = 100
```

### Reindexing

Force a full reindex:

1. Open **Tools > Rebuild Index**
2. Wait for indexing to complete
3. Search is updated

Via CLI:
```bash
tachyon index rebuild
```

## Search API

Programmatic access via REST API:

### Search Endpoint

```
GET /api/v1/documents/search?q=api&status=published
```

### Parameters

| Parameter | Description |
|-----------|-------------|
| `q` | Search query |
| `page` | Page number (default: 1) |
| `limit` | Results per page (default: 20) |
| `sort` | Sort field |
| `order` | Sort order (asc/desc) |

### Response

```json
{
  "results": [
    {
      "id": "doc-123",
      "title": "API Reference",
      "excerpt": "...matching text...",
      "path": "docs/api/reference.md",
      "score": 0.95,
      "modified": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 42,
  "page": 1,
  "pages": 3
}
```

## Tips and Tricks

### Quick Filters

Use keyboard shortcuts for common filters:
- `Ctrl+Shift+P`: Published only
- `Ctrl+Shift+D`: Drafts only
- `Ctrl+Shift+R`: Recent (last 7 days)

### Search History

Access recent searches from the search dropdown.

### Saved Searches

Save frequent searches:
1. Enter your query
2. Click **Save Search**
3. Name the search

Access saved searches from the sidebar.

### Search Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open search |
| `Esc` | Close search |
| `↑` / `↓` | Navigate results |
| `Enter` | Open result |
| `Tab` | Focus filters |

## Troubleshooting

### No Results Found

1. Check spelling
2. Try simpler terms
3. Remove filters
4. Wait for indexing

### Outdated Results

1. Rebuild index: `Tools > Rebuild Index`
2. Check file watcher status
3. Verify files are in indexed directories

### Slow Search

1. Reduce result limit
2. Use more specific queries
3. Add filters to narrow scope
