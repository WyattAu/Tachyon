---
title: API Reference
description: REST API endpoints for the Tachyon server
order: 2
tags: [api, reference]
---

# API Reference

All API endpoints are prefixed with `/api/v1`. Authentication uses Bearer JWT tokens.

## Authentication

### Register
```
POST /api/v1/auth/register
Content-Type: application/json

{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

### Login
```
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "string",
  "password": "string"
}

Response: { "token": "jwt...", "user": { ... } }
```

## Documents

### List Documents
```
GET /api/v1/documents?page=1&page_size=20&status=published
Authorization: Bearer <token>
```

### Create Document
```
POST /api/v1/documents
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "string",
  "content": "markdown content",
  "project_id": "uuid (optional)"
}
```

### Get Document
```
GET /api/v1/documents/{id}
Authorization: Bearer <token>
```

### Update Document
```
PUT /api/v1/documents/{id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "string",
  "content": "markdown content",
  "status": "draft|published|archived"
}
```

### Delete Document
```
DELETE /api/v1/documents/{id}
Authorization: Bearer <token>
```

## Search

### Search Documents
```
GET /api/v1/search?q=query&status=published&tags=tag1,tag2&page=1&page_size=20
Authorization: Bearer <token>
```

### Global Search
```
GET /api/v1/search/global?q=query
Authorization: Bearer <token>
```

### Autocomplete Suggestions
```
GET /api/v1/search/suggest?q=que&limit=5
Authorization: Bearer <token>
```

## SSG (Static Site Generator)

### Get SSG Config
```
GET /api/v1/ssg/config
Authorization: Bearer <token>
```

### Build Static Site
```
POST /api/v1/ssg/build
Authorization: Bearer <token>

Response: { "build_result": { "pages": 5, "total_files": 8, ... } }
```

### Download Static Site (ZIP)
```
GET /api/v1/ssg/download
Authorization: Bearer <token>

Response: application/zip
```

## Teams

### List Teams
```
GET /api/v1/teams
Authorization: Bearer <token>
```

### Create Team
```
POST /api/v1/teams
Authorization: Bearer <token>
Content-Type: application/json

{ "name": "string", "description": "string" }
```

## Plugins

### List Plugins
```
GET /api/v1/plugins
Authorization: Bearer <token>
```

### Install Plugin
```
POST /api/v1/plugins/install
Authorization: Bearer <token>
Content-Type: application/json

{ "repository_url": "string" }
```
