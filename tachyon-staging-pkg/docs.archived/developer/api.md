# API Usage Guide

This guide covers using the Tachyon REST API for integrations and automation.

## Overview

Tachyon provides a RESTful API with:
- JSON request/response format
- JWT and API key authentication
- OpenAPI/Swagger documentation
- Rate limiting
- Comprehensive error handling

## Base URL

```
Production: https://api.example.com/api/v1
Development: http://localhost:8080/api/v1
```

## Authentication

### JWT Token

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  https://api.example.com/api/v1/documents
```

### API Key

```bash
curl -H "X-API-Key: tchk_your_api_key" \
  https://api.example.com/api/v1/documents
```

See [Authentication Guide](../user-guide/authentication.md) for details.

## Common Headers

| Header | Description |
|--------|-------------|
| `Authorization` | Bearer token for JWT auth |
| `X-API-Key` | API key for service accounts |
| `Content-Type` | `application/json` for POST/PUT |
| `Accept` | `application/json` |
| `X-Request-ID` | Unique request ID for tracing |

## Response Format

### Success Response

```json
{
  "id": "uuid",
  "title": "Document Title",
  "content": "Document content...",
  "created_at": "2026-03-09T12:00:00Z"
}
```

### Error Response

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request",
    "details": [
      {
        "field": "title",
        "message": "Title is required"
      }
    ]
  }
}
```

## Pagination

List endpoints support pagination:

```bash
GET /api/v1/documents?page=2&per_page=50
```

Response includes pagination metadata:

```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "per_page": 50,
    "total": 245,
    "total_pages": 5
  }
}
```

## Rate Limiting

Rate limit headers in responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1709992800
```

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request data |
| `CONFLICT` | 409 | Resource conflict |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

## API Endpoints Overview

### Authentication

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/auth/login` | POST | Login with credentials |
| `/auth/logout` | DELETE | Logout current session |
| `/auth/refresh` | POST | Refresh JWT token |
| `/auth/guest` | POST | Guest login |

### Documents

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/documents` | GET | List documents |
| `/documents` | POST | Create document |
| `/documents/:id` | GET | Get document |
| `/documents/:id` | PUT | Update document |
| `/documents/:id` | DELETE | Delete document |
| `/documents/:id/versions` | GET | Get versions |

### Projects

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/projects` | GET | List projects |
| `/projects` | POST | Create project |
| `/projects/:id` | GET | Get project |
| `/projects/:id` | PUT | Update project |
| `/projects/:id` | DELETE | Delete project |

### Search

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/search` | GET | Search documents |
| `/search/suggest` | GET | Search suggestions |

### Teams

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/teams` | GET | List teams |
| `/teams` | POST | Create team |
| `/teams/:id` | GET | Get team |
| `/teams/:id/members` | GET | List members |
| `/teams/:id/members` | POST | Invite member |

### Users

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/users/me` | GET | Get current user |
| `/users/me` | PATCH | Update profile |
| `/users/me/api-keys` | GET | List API keys |
| `/users/me/api-keys` | POST | Create API key |

## Examples

### Create Document

```bash
curl -X POST https://api.example.com/api/v1/documents \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "project_id": "project-uuid",
    "title": "API Guide",
    "content": "# API Guide\n\nContent here...",
    "tags": ["api", "guide"]
  }'
```

### Search Documents

```bash
curl "https://api.example.com/api/v1/search?q=authentication&tags=api&per_page=20" \
  -H "Authorization: Bearer $TOKEN"
```

### Upload Document with File

```bash
curl -X POST https://api.example.com/api/v1/documents/import \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@document.md" \
  -F "project_id=project-uuid"
```

## SDK Examples

### Python

```python
import requests

class TachyonClient:
    def __init__(self, base_url, api_key):
        self.base_url = base_url
        self.headers = {"X-API-Key": api_key}
    
    def get_documents(self, project_id):
        response = requests.get(
            f"{self.base_url}/documents",
            headers=self.headers,
            params={"project_id": project_id}
        )
        return response.json()
    
    def create_document(self, data):
        response = requests.post(
            f"{self.base_url}/documents",
            headers={**self.headers, "Content-Type": "application/json"},
            json=data
        )
        return response.json()

# Usage
client = TachyonClient("https://api.example.com/api/v1", "tchk_your_key")
docs = client.get_documents("project-uuid")
```

### JavaScript

```javascript
class TachyonClient {
  constructor(baseUrl, apiKey) {
    this.baseUrl = baseUrl;
    this.headers = { 'X-API-Key': apiKey };
  }

  async getDocuments(projectId) {
    const response = await fetch(
      `${this.baseUrl}/documents?project_id=${projectId}`,
      { headers: this.headers }
    );
    return response.json();
  }

  async createDocument(data) {
    const response = await fetch(`${this.baseUrl}/documents`, {
      method: 'POST',
      headers: { ...this.headers, 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    return response.json();
  }
}

// Usage
const client = new TachyonClient('https://api.example.com/api/v1', 'tchk_your_key');
const docs = await client.getDocuments('project-uuid');
```

## OpenAPI Documentation

Access interactive API documentation:

- Swagger UI: `http://localhost:8080/swagger-ui/`
- OpenAPI Spec: `http://localhost:8080/api-docs/openapi.json`

## Best Practices

1. **Use API Keys** for automation and services
2. **Handle pagination** for large result sets
3. **Implement retry logic** with exponential backoff
4. **Cache responses** when appropriate
5. **Use proper error handling**
6. **Set appropriate timeouts**

## Next Steps

- [Authentication](../user-guide/authentication.md) - Authentication details
- [API Reference](../api/authentication.md) - Detailed API docs
- [WebSockets](websockets.md) - Real-time features
