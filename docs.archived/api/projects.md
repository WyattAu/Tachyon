# Projects API Reference

Complete API reference for project management endpoints.

## Endpoints

### List Projects

Retrieve a paginated list of projects.

**Request**

```http
GET /projects?page=1&per_page=20
Authorization: Bearer YOUR_TOKEN
```

**Query Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `team_id` | UUID | Filter by team |
| `visibility` | string | Filter by visibility (`team`, `public`) |
| `page` | integer | Page number |
| `per_page` | integer | Items per page |

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "data": [
    {
      "id": "00000000-0000-0000-0000-000000000010",
      "team_id": "00000000-0000-0000-0000-000000000020",
      "name": "API Documentation",
      "description": "REST API reference documentation",
      "visibility": "team",
      "document_count": 45,
      "created_by": "00000000-0000-0000-0000-000000000100",
      "created_at": "2026-02-01T00:00:00Z",
      "updated_at": "2026-03-09T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 5,
    "total_pages": 1
  }
}
```

---

### Get Project

Retrieve a single project.

**Request**

```http
GET /projects/{project_id}
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000010",
  "team_id": "00000000-0000-0000-0000-000000000020",
  "name": "API Documentation",
  "description": "REST API reference documentation",
  "visibility": "team",
  "settings": {
    "default_content_type": "markdown",
    "auto_index": true
  },
  "document_count": 45,
  "created_by": "00000000-0000-0000-0000-000000000100",
  "created_at": "2026-02-01T00:00:00Z",
  "updated_at": "2026-03-09T12:00:00Z"
}
```

---

### Create Project

Create a new project.

**Request**

```http
POST /projects
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "team_id": "00000000-0000-0000-0000-000000000020",
  "name": "User Guide",
  "description": "User documentation and guides",
  "visibility": "team",
  "settings": {
    "default_content_type": "markdown"
  }
}
```

**Response**

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000011",
  "name": "User Guide",
  "description": "User documentation and guides",
  "visibility": "team",
  "document_count": 0,
  "created_at": "2026-03-09T12:00:00Z"
}
```

---

### Update Project

Update a project.

**Request**

```http
PUT /projects/{project_id}
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "name": "Updated User Guide",
  "description": "Updated user documentation",
  "visibility": "public"
}
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000011",
  "name": "Updated User Guide",
  "updated_at": "2026-03-09T13:00:00Z"
}
```

---

### Delete Project

Delete a project and all its documents.

**Request**

```http
DELETE /projects/{project_id}
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 204 No Content
```

---

### Get Project Tree

Get the document tree for a project.

**Request**

```http
GET /projects/{project_id}/tree
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "tree": {
    "id": "00000000-0000-0000-0000-000000000100",
    "title": "Root",
    "children": [
      {
        "id": "00000000-0000-0000-0000-000000000101",
        "title": "Getting Started",
        "children": [
          {
            "id": "00000000-0000-0000-0000-000000000102",
            "title": "Installation",
            "children": []
          }
        ]
      }
    ]
  }
}
```

---

### Project Search

Search within a project.

**Request**

```http
GET /projects/{project_id}/search?q=authentication
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "results": [
    {
      "id": "00000000-0000-0000-0000-000000000101",
      "title": "Authentication Guide",
      "excerpt": "...guide covers authentication...",
      "score": 0.95
    }
  ],
  "total": 5
}
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `PROJECT_NOT_FOUND` | 404 | Project not found |
| `PERMISSION_DENIED` | 403 | Insufficient permissions |

---

## Next Steps

- [Documents API](documents.md)
- [Search API](search.md)
- [Teams Guide](../user-guide/teams.md)
