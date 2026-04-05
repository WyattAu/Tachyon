# Search API Reference

Complete API reference for search endpoints.

## Endpoints

### Global Search

Search across all accessible documents.

**Request**

```http
GET /search?q=authentication&tags=api&per_page=20
Authorization: Bearer YOUR_TOKEN
```

**Query Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Search query |
| `project_id` | UUID | Filter by project |
| `tags` | string | Comma-separated tags |
| `author` | string | Filter by author email |
| `is_public` | boolean | Filter by visibility |
| `created_after` | date | Filter by creation date |
| `created_before` | date | Filter by creation date |
| `sort` | string | Sort field (`score`, `created_at`, `updated_at`) |
| `order` | string | Sort order (`asc`, `desc`) |
| `page` | integer | Page number |
| `per_page` | integer | Results per page |
| `highlight` | boolean | Enable highlighting |

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "results": [
    {
      "id": "00000000-0000-0000-0000-000000000001",
      "project_id": "00000000-0000-0000-0000-000000000010",
      "title": "Authentication Guide",
      "excerpt": "This guide covers authentication methods including JWT and API keys...",
      "highlighted": "This guide covers <mark>authentication</mark> methods including JWT and API keys...",
      "score": 0.95,
      "tags": ["api", "auth", "security"],
      "created_at": "2026-03-01T00:00:00Z",
      "updated_at": "2026-03-09T12:00:00Z"
    }
  ],
  "total": 42,
  "page": 1,
  "per_page": 20,
  "query_time_ms": 23
}
```

---

### Search Suggestions

Get auto-complete suggestions.

**Request**

```http
GET /search/suggest?q=auth
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "suggestions": [
    "authentication",
    "authorization",
    "author",
    "auth token"
  ]
}
```

---

### Related Searches

Get related search queries.

**Request**

```http
GET /search/related?q=authentication
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "related": [
    "API authentication",
    "JWT tokens",
    "OAuth",
    "API keys"
  ]
}
```

---

### Search Index Status

Get search index status.

**Request**

```http
GET /search/status
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "ready",
  "document_count": 1250,
  "index_size_mb": 45,
  "last_indexed": "2026-03-09T12:00:00Z",
  "indexing_in_progress": false
}
```

---

### Rebuild Search Index

Force rebuild of search index (admin only).

**Request**

```http
POST /search/reindex
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 202 Accepted
Content-Type: application/json

{
  "status": "started",
  "estimated_documents": 1250
}
```

---

## Search Syntax

### Basic Search

```
authentication
```

### Phrase Search

```
"API authentication"
```

### Boolean Operators

```
api AND authentication
api OR rest
api -deprecated
```

### Field Search

```
title:authentication
content:api
tags:guide
author:john
```

### Wildcards

```
auth*
api?guide
```

### Fuzzy Search

```
authenticaton~
authenticaton~2
```

### Proximity

```
"api authentication"~5
```

### Range

```
created_at:[2026-01-01 TO 2026-12-31]
version:[1 TO 5]
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_QUERY` | 400 | Invalid search query |
| `INDEX_ERROR` | 500 | Search index error |

---

## Next Steps

- [User Guide: Search](../user-guide/search.md)
- [Documents API](documents.md)
- [Projects API](projects.md)
