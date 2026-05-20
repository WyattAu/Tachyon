# API Keys Guide

This guide covers API key usage and management in Tachyon.

## Overview

API keys provide secure programmatic access to Tachyon for automation, integrations, and service accounts.

```mermaid
graph LR
    A[API Key] --> B[Authentication]
    B --> C[API Access]
    C --> D[Documents]
    C --> E[Search]
    C --> F[Teams]
    
    A --> G[Scopes]
    G --> H[Read]
    G --> I[Write]
    G --> J[Admin]
```

## API Key Format

API keys follow the format: `tchk_<random-string>`

Example: `tchk_1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyz`

## Creating API Keys

### Via Web Interface

1. Navigate to User Settings
2. Click "API Keys"
3. Click "Create API Key"
4. Enter name and select scopes
5. Set expiration date
6. Click "Create"
7. **Copy the key immediately** (shown only once)

### Via API

```bash
POST /api/v1/users/me/api-keys
Authorization: Bearer YOUR_JWT_TOKEN
Content-Type: application/json

{
  "name": "CI/CD Pipeline",
  "description": "Automated documentation deployment",
  "scopes": ["documents:read", "documents:write"],
  "expires_at": "2026-12-31T23:59:59Z"
}
```

**Response:**
```json
{
  "id": "apikey-uuid",
  "name": "CI/CD Pipeline",
  "description": "Automated documentation deployment",
  "key": "tchk_1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyz",
  "scopes": ["documents:read", "documents:write"],
  "created_at": "2026-03-09T12:00:00Z",
  "expires_at": "2026-12-31T23:59:59Z",
  "last_used_at": null
}
```

## Using API Keys

### Authentication Header

Include the API key in the `X-API-Key` header:

```bash
curl -H "X-API-Key: tchk_1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyz" \
  https://api.example.com/api/v1/documents
```

### Authorization Header (Alternative)

Can also use Bearer token format:

```bash
curl -H "Authorization: Bearer tchk_1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyz" \
  https://api.example.com/api/v1/documents
```

### Programmatic Usage

#### Python

```python
import requests

api_key = "tchk_1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyz"
headers = {"X-API-Key": api_key}

# Get documents
response = requests.get(
    "https://api.example.com/api/v1/documents",
    headers=headers
)
documents = response.json()

# Create document
new_doc = {
    "title": "API Guide",
    "content": "# API Guide\n\nContent here...",
    "project_id": "project-uuid"
}
response = requests.post(
    "https://api.example.com/api/v1/documents",
    headers=headers,
    json=new_doc
)
```

#### JavaScript/Node.js

```javascript
const API_KEY = 'tchk_1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyz';
const BASE_URL = 'https://api.example.com/api/v1';

// Get documents
const response = await fetch(`${BASE_URL}/documents`, {
  headers: {
    'X-API-Key': API_KEY
  }
});
const documents = await response.json();

// Create document
const newDoc = {
  title: 'API Guide',
  content: '# API Guide\n\nContent here...',
  project_id: 'project-uuid'
};
const createResponse = await fetch(`${BASE_URL}/documents`, {
  method: 'POST',
  headers: {
    'X-API-Key': API_KEY,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(newDoc)
});
```

#### cURL

```bash
# List documents
curl -H "X-API-Key: tchk_your_key_here" \
  https://api.example.com/api/v1/documents

# Create document
curl -X POST \
  -H "X-API-Key: tchk_your_key_here" \
  -H "Content-Type: application/json" \
  -d '{"title":"New Doc","content":"Content","project_id":"uuid"}' \
  https://api.example.com/api/v1/documents

# Search
curl -H "X-API-Key: tchk_your_key_here" \
  "https://api.example.com/api/v1/search?q=authentication"
```

## API Key Scopes

### Available Scopes

| Scope | Description |
|-------|-------------|
| `documents:read` | Read documents |
| `documents:write` | Create and update documents |
| `documents:delete` | Delete documents |
| `search:read` | Search documents |
| `projects:read` | Read projects |
| `projects:write` | Create and update projects |
| `teams:read` | Read team information |
| `teams:write` | Manage teams |
| `users:read` | Read user profiles |
| `admin` | Full administrative access |

### Scope Examples

#### Read-Only Access

```json
{
  "scopes": ["documents:read", "search:read", "projects:read"]
}
```

#### Full Document Access

```json
{
  "scopes": ["documents:read", "documents:write", "documents:delete"]
}
```

#### Admin Access

```json
{
  "scopes": ["admin"]
}
```

## Managing API Keys

### List Your API Keys

```bash
GET /api/v1/users/me/api-keys
Authorization: Bearer YOUR_JWT_TOKEN
```

**Response:**
```json
{
  "api_keys": [
    {
      "id": "apikey-uuid",
      "name": "CI/CD Pipeline",
      "description": "Automated deployment",
      "scopes": ["documents:read", "documents:write"],
      "created_at": "2026-03-09T12:00:00Z",
      "expires_at": "2026-12-31T23:59:59Z",
      "last_used_at": "2026-03-09T14:30:00Z",
      "prefix": "tchk_1a2b3c"
    }
  ],
  "total": 3
}
```

Note: The full key is not returned, only the prefix for identification.

### Update API Key

```bash
PATCH /api/v1/users/me/api-keys/{key_id}
Authorization: Bearer YOUR_JWT_TOKEN
Content-Type: application/json

{
  "name": "Updated Name",
  "scopes": ["documents:read", "documents:write", "search:read"]
}
```

### Revoke API Key

```bash
DELETE /api/v1/users/me/api-keys/{key_id}
Authorization: Bearer YOUR_JWT_TOKEN
```

## Security Best Practices

### 1. Use Minimal Scopes

Only request the permissions you need:

```json
// Bad: Too permissive
{
  "scopes": ["admin"]
}

// Good: Minimal required scopes
{
  "scopes": ["documents:read", "documents:write"]
}
```

### 2. Set Expiration Dates

Always set an expiration date:

```json
{
  "expires_at": "2026-12-31T23:59:59Z"
}
```

### 3. Store Securely

Never commit API keys to version control:

```bash
# Use environment variables
export TACHYON_API_KEY="tchk_your_key_here"

# In code
const apiKey = process.env.TACHYON_API_KEY;
```

### 4. Rotate Regularly

Create new keys and rotate periodically:

1. Create new API key
2. Update applications to use new key
3. Verify everything works
4. Revoke old key

### 5. Monitor Usage

Check key usage regularly:

```bash
GET /api/v1/users/me/api-keys/{key_id}/usage
Authorization: Bearer YOUR_JWT_TOKEN
```

**Response:**
```json
{
  "requests_total": 1234,
  "requests_by_endpoint": {
    "/api/v1/documents": 856,
    "/api/v1/search": 378
  },
  "last_used_at": "2026-03-09T14:30:00Z",
  "errors_total": 12
}
```

### 6. Use Different Keys for Different Purposes

- **CI/CD**: Key for automated deployments
- **Integration**: Key for third-party integrations
- **Development**: Key for local development
- **Production**: Key for production systems

## Use Cases

### CI/CD Integration

```yaml
# GitHub Actions
name: Deploy Docs
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Sync documentation
        env:
          TACHYON_API_KEY: ${{ secrets.TACHYON_API_KEY }}
        run: |
          curl -X POST \
            -H "X-API-Key: $TACHYON_API_KEY" \
            -H "Content-Type: application/json" \
            -d @docs.json \
            https://api.example.com/api/v1/documents/bulk
```

### Webhook Integration

```javascript
// Express.js webhook handler
app.post('/webhook/docs', async (req, res) => {
  const apiKey = process.env.TACHYON_API_KEY;
  
  const response = await fetch('https://api.example.com/api/v1/documents', {
    method: 'POST',
    headers: {
      'X-API-Key': apiKey,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(req.body)
  });
  
  res.json(await response.json());
});
```

### Automated Backup

```python
# Python backup script
import requests
import json
from datetime import datetime

API_KEY = os.environ['TACHYON_API_KEY']
BASE_URL = 'https://api.example.com/api/v1'

def backup_documents():
    headers = {'X-API-Key': API_KEY}
    response = requests.get(f'{BASE_URL}/documents', headers=headers)
    documents = response.json()
    
    filename = f'backup_{datetime.now().strftime("%Y%m%d_%H%M%S")}.json'
    with open(filename, 'w') as f:
        json.dump(documents, f, indent=2)
    
    print(f'Backed up {len(documents)} documents to {filename}')

if __name__ == '__main__':
    backup_documents()
```

## Rate Limiting

API keys are subject to rate limiting based on server configuration:

```bash
# Check rate limit headers
curl -I -H "X-API-Key: your_key" \
  https://api.example.com/api/v1/documents

# Response headers:
# X-RateLimit-Limit: 1000
# X-RateLimit-Remaining: 995
# X-RateLimit-Reset: 1709992800
```

## Troubleshooting

### Invalid API Key

```
Error: Invalid API key
```

**Solutions:**
- Verify key format (starts with `tchk_`)
- Check key hasn't been revoked
- Ensure key hasn't expired

### Insufficient Permissions

```
Error: Insufficient permissions
```

**Solutions:**
- Check key has required scopes
- Verify resource permissions
- Contact admin for access

### Rate Limit Exceeded

```
Error: Rate limit exceeded
```

**Solutions:**
- Wait for limit reset
- Reduce request frequency
- Contact admin for higher limit

## API Key API Endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /api/v1/users/me/api-keys` | Create API key |
| `GET /api/v1/users/me/api-keys` | List API keys |
| `GET /api/v1/users/me/api-keys/{id}` | Get API key |
| `PATCH /api/v1/users/me/api-keys/{id}` | Update API key |
| `DELETE /api/v1/users/me/api-keys/{id}` | Revoke API key |
| `GET /api/v1/users/me/api-keys/{id}/usage` | Key usage stats |

## Next Steps

- [Authentication](authentication.md) - Authentication methods
- [API Reference](../api/authentication.md) - API documentation
- [Configuration](configuration.md#api-keys) - API key configuration
