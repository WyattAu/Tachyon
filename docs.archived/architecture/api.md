# API Design

Documentation of Tachyon's REST API design principles and patterns.

## Overview

Tachyon provides a RESTful API with:
- JSON request/response bodies
- Standard HTTP methods
- Meaningful status codes
- Consistent error handling

## Base URL

```
https://tachyon.example.com/api/v1
```

## Authentication

### Bearer Token

```http
Authorization: Bearer <token>
```

### Session Cookie

For web clients:
```http
Cookie: session=<session_id>
```

## Common Patterns

### Response Format

**Success:**
```json
{
  "data": { ... },
  "meta": {
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

**Error:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input",
    "details": [
      {
        "field": "title",
        "message": "Title is required"
      }
    ]
  }
}
```

### Pagination

```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "pages": 5
  }
}
```

Query parameters:
- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 20, max: 100)

### Sorting

Query parameters:
- `sort`: Field to sort by
- `order`: `asc` or `desc` (default: `desc`)

```
GET /api/v1/documents?sort=updated_at&order=desc
```

### Filtering

Query parameters for filters:

```
GET /api/v1/documents?status=published&author=john
```

## HTTP Methods

| Method | Usage | Idempotent |
|--------|-------|------------|
| GET | Retrieve resources | Yes |
| POST | Create resources | No |
| PUT | Replace resources | Yes |
| PATCH | Partial update | Yes |
| DELETE | Remove resources | Yes |

## Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (success) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 409 | Conflict |
| 422 | Validation Error |
| 429 | Rate Limited |
| 500 | Internal Error |
| 503 | Service Unavailable |

## Resources

### Documents

```
GET    /api/v1/documents           # List documents
POST   /api/v1/documents           # Create document
GET    /api/v1/documents/:id       # Get document
PUT    /api/v1/documents/:id       # Update document
DELETE /api/v1/documents/:id       # Delete document
GET    /api/v1/documents/:id/history # Get history
POST   /api/v1/documents/:id/publish # Publish document
```

### Search

```
GET    /api/v1/search              # Search documents
POST   /api/v1/search/advanced     # Advanced search
```

### Users

```
GET    /api/v1/users               # List users (admin)
POST   /api/v1/users               # Create user (admin)
GET    /api/v1/users/:id           # Get user
PUT    /api/v1/users/:id           # Update user
DELETE /api/v1/users/:id           # Delete user (admin)
```

### Groups

```
GET    /api/v1/groups              # List groups
POST   /api/v1/groups              # Create group (admin)
GET    /api/v1/groups/:id          # Get group
PUT    /api/v1/groups/:id          # Update group (admin)
DELETE /api/v1/groups/:id          # Delete group (admin)
POST   /api/v1/groups/:id/members  # Add member
DELETE /api/v1/groups/:id/members/:userId # Remove member
```

### Comments

```
GET    /api/v1/documents/:id/comments # List comments
POST   /api/v1/documents/:id/comments # Create comment
PUT    /api/v1/comments/:id       # Update comment
DELETE /api/v1/comments/:id       # Delete comment
POST   /api/v1/comments/:id/resolve # Resolve thread
```

### Render

```
POST   /api/v1/render             # Render markdown
```

## Rate Limiting

Headers:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1705315800
```

Limit exceeded:
```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded",
    "retry_after": 60
  }
}
```

## Versioning

API version in URL path:
```
/api/v1/documents
/api/v2/documents (future)
```

## Examples

### Create Document

```http
POST /api/v1/documents
Content-Type: application/json
Authorization: Bearer <token>

{
  "title": "API Documentation",
  "content": "# Introduction\n\n...",
  "tags": ["api", "reference"],
  "visibility": "public"
}
```

Response:
```json
{
  "data": {
    "id": "doc_abc123",
    "title": "API Documentation",
    "content": "# Introduction\n\n...",
    "tags": ["api", "reference"],
    "visibility": "public",
    "status": "draft",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

### Search Documents

```http
GET /api/v1/search?q=api&status=published&page=1&per_page=10
Authorization: Bearer <token>
```

Response:
```json
{
  "data": [
    {
      "id": "doc_abc123",
      "title": "API Documentation",
      "excerpt": "...matching text...",
      "score": 0.95,
      "highlights": ["<mark>API</mark> Documentation"]
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 10,
    "total": 25,
    "pages": 3
  }
}
```

### Error Response

```http
POST /api/v1/documents
Content-Type: application/json

{
  "title": "",
  "content": "test"
}
```

Response (422):
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "details": [
      {
        "field": "title",
        "message": "Title cannot be empty"
      }
    ]
  }
}
```

## OpenAPI Specification

Full specification available at:
```
GET /api/v1/openapi.json
```

Interactive docs at:
```
GET /api/docs
```
