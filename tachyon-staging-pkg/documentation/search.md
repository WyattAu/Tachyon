---
title: Search
description: Full-text search with Tantivy and PostgreSQL
order: 7
tags: [search, reference]
---

# Search

Tachyon provides full-text search powered by Tantivy (BM25 ranking) with PostgreSQL as the persistence layer.

## Query Syntax

### Basic Search

```
GET /api/v1/search?q=rust programming&page=1&page_size=20
```

### Field-Specific Search

```
GET /api/v1/search?q=title:architecture&status=published
```

### Tag Filtering

```
GET /api/v1/search?q=rust&tags=tutorial,guide
```

### Faceted Search

```
GET /api/v1/search/facets?q=rust
```

Returns document counts grouped by tag, author, and status.

### Autocomplete

```
GET /api/v1/search/suggest?q=rus&limit=5
```

Returns term suggestions for query completion.

## Search Architecture

```
Document Created/Updated
        |
        v
  Content Pipeline
  /            \
 v              v
Tantivy      PostgreSQL
Index         tsvector
  \            /
   v          v
  Query Engine (merge + rank)
        |
        v
    Results (BM25 scored)
```

## Index Configuration

The Tantivy index is stored at `.tachyon/search_index/` by default. Fields indexed:

| Field | Type | Indexed | Stored |
|-------|------|---------|--------|
| `title` | TEXT | tokenized | yes |
| `content` | TEXT | tokenized | yes |
| `tags` | KEYWORD | tokenized | yes |
| `author` | TEXT | tokenized | yes |
| `status` | KEYWORD | exact | yes |
| `created_at` | DATE | range | yes |
| `updated_at` | DATE | range | yes |

## Reindexing

Trigger a full reindex via the API:

```
POST /api/v1/search/reindex
Authorization: Bearer <token>
```

Or via CLI:

```bash
cargo run --release -p tachyon-server -- reindex
```

## Performance

Tantivy provides sub-100ms query latency for indices up to 100,000 documents. Benchmarks are available in `tachyon/crates/benchmarks/`.

## Further Reading

- [API Reference](api-reference.html) - Search endpoints
- [Architecture](architecture.html) - System design
