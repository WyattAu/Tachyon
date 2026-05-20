# Documents API Reference

Complete API reference for document management endpoints.

## Base URL

```
https://api.example.com/api/v1
```

## Authentication

All endpoints require authentication:

```http
Authorization: Bearer YOUR_TOKEN
```

Or using API key:

```http
X-API-Key: tchk_your_api_key
```

## Endpoints

### List Documents

Retrieve a paginated list of documents.

**Request**

```http
GET /documents?project_id={project_id}&page=1&per_page=20
Authorization: Bearer YOUR_TOKEN
```

**Query Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `project_id` | UUID | Filter by project |
| `parent_id` | UUID | Filter by parent document |
| `is_public` | boolean | Filter by visibility |
| `tags` | string | Comma-separated tags |
| `page` | integer | Page number (default: 1) |
| `per_page` | integer | Items per page (default: 20, max: 100) |

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "data": [
    {
      "id": "00000000-0000-0000-0000-000000000001",
      "project_id": "00000000-0000-0000-0000-000000000010",
      "parent_id": null,
      "title": "Getting Started",
      "content": "# Getting Started\n\nWelcome to Tachyon!",
      "content_type": "markdown",
      "tags": ["guide", "intro"],
      "metadata": {},
      "is_public": false,
      "version": 3,
      "created_by": "00000000-0000-0000-0000-000000000100",
      "updated_by": "00000000-0000-0000-0000-000000000100",
      "created_at": "2026-03-01T00:00:00Z",
      "updated_at": "2026-03-09T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 45,
    "total_pages": 3
  }
}
```

---

### Get Document

Retrieve a single document by ID.

**Request**

```http
GET /documents/{document_id}
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000001",
  "project_id": "00000000-0000-0000-0000-000000000010",
  "parent_id": null,
  "title": "Getting Started",
  "content": "# Getting Started\n\nWelcome to Tachyon!",
  "content_type": "markdown",
  "tags": ["guide", "intro"],
  "metadata": {
    "author": "John Doe",
    "category": "documentation"
  },
  "is_public": false,
  "version": 3,
  "created_by": "00000000-0000-0000-0000-000000000100",
  "updated_by": "00000000-0000-0000-0000-000000000100",
  "created_at": "2026-03-01T00:00:00Z",
  "updated_at": "2026-03-09T12:00:00Z"
}
```

**Error Response**

```http
HTTP/1.1 404 Not Found
Content-Type: application/json

{
  "error": {
    "code": "DOCUMENT_NOT_FOUND",
    "message": "Document not found"
  }
}
```

---

### Create Document

Create a new document.

**Request**

```http
POST /documents
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "project_id": "00000000-0000-0000-0000-000000000010",
  "parent_id": null,
  "title": "API Guide",
  "content": "# API Guide\n\nThis guide covers the REST API.",
  "content_type": "markdown",
  "tags": ["api", "guide"],
  "metadata": {
    "author": "Jane Doe"
  },
  "is_public": false
}
```

**Response**

```http
HTTP/1.1 201 Created
Content-Type: application/json
Location: /documents/00000000-0000-0000-0000-000000000002

{
  "id": "00000000-0000-0000-0000-000000000002",
  "project_id": "00000000-0000-0000-0000-000000000010",
  "title": "API Guide",
  "content": "# API Guide\n\nThis guide covers the REST API.",
  "content_type": "markdown",
  "tags": ["api", "guide"],
  "metadata": {
    "author": "Jane Doe"
  },
  "is_public": false,
  "version": 1,
  "created_at": "2026-03-09T12:00:00Z",
  "updated_at": "2026-03-09T12:00:00Z"
}
```

---

### Update Document

Update an existing document.

**Request**

```http
PUT /documents/{document_id}
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "title": "Updated API Guide",
  "content": "# Updated API Guide\n\nNew content here.",
  "tags": ["api", "guide", "updated"],
  "metadata": {
    "author": "Jane Doe",
    "reviewed": true
  }
}
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000002",
  "title": "Updated API Guide",
  "content": "# Updated API Guide\n\nNew content here.",
  "version": 2,
  "updated_at": "2026-03-09T13:00:00Z"
}
```

---

### Partial Update Document

Partially update a document.

**Request**

```http
PATCH /documents/{document_id}
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "tags": ["api", "guide", "v2"]
}
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000002",
  "tags": ["api", "guide", "v2"],
  "version": 3,
  "updated_at": "2026-03-09T14:00:00Z"
}
```

---

### Delete Document

Delete a document (soft delete).

**Request**

```http
DELETE /documents/{document_id}
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 204 No Content
```

### Permanent Delete

Permanently delete a document.

**Request**

```http
DELETE /documents/{document_id}?permanent=true
Authorization: Bearer YOUR_TOKEN
```

---

### Restore Document

Restore a soft-deleted document.

**Request**

```http
POST /documents/{document_id}/restore
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000002",
  "deleted_at": null,
  "restored_at": "2026-03-09T15:00:00Z"
}
```

---

### Get Document Versions

Get version history for a document.

**Request**

```http
GET /documents/{document_id}/versions
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "versions": [
    {
      "version": 3,
      "title": "Updated API Guide",
      "content": "# Updated API Guide\n\nNew content.",
      "changed_by": "00000000-0000-0000-0000-000000000100",
      "change_summary": "Added new section",
      "created_at": "2026-03-09T14:00:00Z"
    },
    {
      "version": 2,
      "title": "API Guide",
      "content": "# API Guide\n\nContent.",
      "changed_by": "00000000-0000-0000-0000-000000000100",
      "change_summary": "Updated content",
      "created_at": "2026-03-09T13:00:00Z"
    },
    {
      "version": 1,
      "title": "API Guide",
      "content": "# API Guide",
      "changed_by": "00000000-0000-0000-0000-000000000100",
      "change_summary": "Initial creation",
      "created_at": "2026-03-09T12:00:00Z"
    }
  ]
}
```

---

### Restore Specific Version

Restore a document to a specific version.

**Request**

```http
POST /documents/{document_id}/restore/{version}
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000002",
  "version": 4,
  "restored_from": 2,
  "updated_at": "2026-03-09T16:00:00Z"
}
```

---

### Duplicate Document

Create a copy of a document.

**Request**

```http
POST /documents/{document_id}/duplicate
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "title": "Copy of API Guide",
  "project_id": "00000000-0000-0000-0000-000000000011"
}
```

**Response**

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000003",
  "title": "Copy of API Guide",
  "content": "# API Guide\n\nContent...",
  "duplicated_from": "00000000-0000-0000-0000-000000000002"
}
```

---

### Export Document

Export a document in various formats.

**Request**

```http
GET /documents/{document_id}/export?format=pdf
Authorization: Bearer YOUR_TOKEN
```

**Query Parameters**

| Parameter | Type | Options |
|-----------|------|---------|
| `format` | string | `markdown`, `html`, `pdf` |

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/pdf
Content-Disposition: attachment; filename="API Guide.pdf"

[Binary PDF content]
```

---

### Bulk Create

Create multiple documents at once.

**Request**

```http
POST /documents/bulk
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "documents": [
    {
      "title": "Doc 1",
      "content": "Content 1",
      "project_id": "00000000-0000-0000-0000-000000000010"
    },
    {
      "title": "Doc 2",
      "content": "Content 2",
      "project_id": "00000000-0000-0000-0000-000000000010"
    }
  ]
}
```

**Response**

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "created": [
    {"id": "...", "title": "Doc 1"},
    {"id": "...", "title": "Doc 2"}
  ],
  "failed": [],
  "total_created": 2
}
```

---

### Bulk Delete

Delete multiple documents.

**Request**

```http
DELETE /documents/bulk
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "document_ids": [
    "00000000-0000-0000-0000-000000000001",
    "00000000-0000-0000-0000-000000000002"
  ]
}
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "deleted": 2,
  "failed": 0
}
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `DOCUMENT_NOT_FOUND` | 404 | Document not found |
| `PERMISSION_DENIED` | 403 | Insufficient permissions |
| `VALIDATION_ERROR` | 400 | Invalid request data |
| `PROJECT_NOT_FOUND` | 404 | Project not found |

---

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/documents` (GET) | 100 requests | 60 seconds |
| `/documents` (POST) | 50 requests | 60 seconds |
| `/documents/bulk` | 10 requests | 60 seconds |

---

## Next Steps

- [User Guide: Documents](../user-guide/documents.md)
- [Search API](search.md)
- [Projects API](projects.md)
