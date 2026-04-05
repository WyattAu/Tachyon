# Tachyon API Reference

**Document ID:** TACHYON-APIREF-V1.0
**Date:** 2026-02-11
**Version:** 0.2.0-beta
**Status:** Released
**Accessibility:** WCAG 2.1 AA Compliant

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [HTTP API Endpoints](#2-http-api-endpoints)
3. [WebSocket Events](#3-websocket-events)
4. [Internal Module Interfaces](#4-internal-module-interfaces)
5. [Data Models](#5-data-models)
6. [Error Handling](#6-error-handling)
7. [Authentication and Authorization](#7-authentication-and-authorization)
8. [Rate Limiting](#8-rate-limiting)
9. [Webhooks (Planned)](#9-webhooks-planned)

---

## 1. Introduction

### 1.1. API Overview

Tachyon provides a RESTful HTTP API for Server Mode operations and WebSocket API for real-time updates. The API follows RESTful conventions and is designed for high performance and low latency.

### 1.2. Base URL

All API endpoints are prefixed with the base URL configured in tachyon.toml:

```
http://localhost:8080/api/v1
```

### 1.3. Authentication

All API requests require authentication (except public endpoints):

```
Authorization: Bearer <token>
```

### 1.4. Response Format

All API responses use JSON format:

```json
{
  "data": {},
  "error": null
}
```

### 1.5. HTTP Status Codes

| Code | Description |
|-------|-------------|
| 200 OK | Request succeeded |
| 201 Created | Resource created successfully |
| 400 Bad Request | Malformed request syntax |
| 401 Unauthorized | Missing or invalid authentication |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Resource not found |
| 409 Conflict | Resource conflict |
| 422 Unprocessable Entity | Validation error |
| 429 Too Many Requests | Rate limit exceeded |
| 500 Internal Server Error | Server error |

---

## 2. HTTP API Endpoints

### 2.1. Documents

#### GET /api/v1/documents

List all documents in the repository.

**Request:**
None

**Response (200 OK):**

```json
{
  "data": {
    "documents": [
      {
        "path": "docs/user_guide.md",
        "title": "User Guide",
        "description": "Complete user guide for Tachyon",
        "modified_at": "2026-02-11T21:56:00Z",
        "size_bytes": 15360,
        "access": "public"
      }
    ],
    "total": 1
  },
  "error": null
}
```

#### GET /api/v1/documents/{path}

Retrieve a specific document by path.

**Parameters:**
- `path`: Document path (URL-encoded)

**Example:**

```
GET /api/v1/documents/docs/user_guide.md
```

**Response (200 OK):**

```json
{
  "data": {
    "path": "docs/user_guide.md",
    "title": "User Guide",
    "content": "# Tachyon User Guide...",
    "frontmatter": {
      "title": "User Guide",
      "tags": ["guide", "user"]
    },
    "commit_hash": "abc123def456",
    "modified_at": "2026-02-11T21:56:00Z"
  },
  "error": null
}
```

#### POST /api/v1/documents/{path}

Create or update a document.

**Request Body:**

```json
{
  "content": "# New Document\n\nContent here...",
  "message": "Commit message",
  "frontmatter": {
    "title": "New Document",
    "tags": ["tag1", "tag2"]
  }
}
```

**Response (201 Created):**

```json
{
  "data": {
    "path": "docs/new_document.md",
    "commit_hash": "def456abc123789",
    "created_at": "2026-02-11T21:56:00Z"
  },
  "error": null
}
```

#### DELETE /api/v1/documents/{path}

Delete a document.

**Response (200 OK):**

```json
{
  "data": {
    "path": "docs/deleted.md",
    "deleted_at": "2026-02-11T21:56:00Z"
  },
  "error": null
}
```

### 2.2. Search

#### GET /api/v1/search

Full-text search across all documents.

**Query Parameters:**
- `q`: Search query (required, max 256 characters)
- `limit`: Maximum results (optional, default 100, max 1000)
- `offset`: Pagination offset (optional)
- `tags`: Filter by tags (optional, array)
- `author`: Filter by author (optional)
- `after`: Date filter (optional, ISO 8601 format)

**Example:**

```
GET /api/v1/search?q=tachyon&limit=10
```

**Response (200 OK):**

```json
{
  "data": {
    "results": [
      {
        "path": "docs/white_paper.md",
        "title": "White Paper",
        "snippet": "Tachyon is a high-performance knowledge management...",
        "score": 2.45,
        "highlighted_terms": ["tachyon", "performance"]
      }
    ],
    "total": 1,
    "query_time_ms": 45
  },
  "error": null
}
```

### 2.3. Git Operations

#### GET /api/v1/git/commits

Get commit history for a document.

**Query Parameters:**
- `path`: Document path (required)
- `limit`: Maximum commits (optional, default 50)

**Response (200 OK):**

```json
{
  "data": {
    "commits": [
      {
        "hash": "abc123def456",
        "message": "Initial commit",
        "author": "John Doe",
        "timestamp": "2026-02-11T21:00:00Z"
      }
    ],
    "total": 1
  },
  "error": null
}
```

#### GET /api/v1/git/diff

Get diff between two commits.

**Query Parameters:**
- `path`: Document path (required)
- `from`: Starting commit hash (required)
- `to`: Ending commit hash (required)

**Response (200 OK):**

```json
{
  "data": {
    "diff": "--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,4 @@\n"
  },
  "error": null
}
```

#### POST /api/v1/git/commit

Commit changes to Git repository.

**Request Body:**

```json
{
  "message": "Commit message",
  "files": ["file1.md", "file2.md"]
}
```

**Response (201 Created):**

```json
{
  "data": {
    "commit_hash": "def456abc123789",
    "created_at": "2026-02-11T21:56:00Z"
  },
  "error": null
}
```

### 2.4. Health

#### GET /api/v1/health

Health check endpoint.

**Response (200 OK):**

```json
{
  "data": {
    "status": "healthy",
    "version": "0.2.0-beta",
    "uptime_seconds": 86400
  },
  "error": null
}
```

---

## 3. WebSocket Events

### 3.1. Connection

WebSocket endpoint: `ws://localhost:8080/ws`

**Connection Request:**

```json
{
  "action": "connect",
  "token": "auth_token_here"
}
```

### 3.2. File Change Events

**Server to Client:**

```json
{
  "event": "file_changed",
  "data": {
    "path": "docs/user_guide.md",
    "commit_hash": "abc123def456"
  }
}
```

### 3.3. Search Results

**Server to Client:**

```json
{
  "event": "search_result",
  "data": {
    "query_id": "uuid-here",
    "results": [...]
  }
}
```

### 3.4. Document Saved

**Client to Server:**

```json
{
  "action": "document_saved",
  "data": {
    "path": "docs/new_document.md",
    "content": "Updated content..."
  }
}
```

---

## 4. Internal Module Interfaces

### 4.1. Content Management

#### parse_markdown()

Parse Markdown document with frontmatter extraction.

**Parameters:**
- `content`: Raw Markdown text (UTF-8)
- `path`: File system path for context

**Returns:**
- `Result<ParsedDocument, ParseError>`
  - ParsedDocument contains AST, frontmatter, metadata
  - ParseError for invalid syntax

**Throws:**
- `ParseError` on malformed Markdown
- `IoError` on file read failure

**Traceability:** CM-RQ-001, CM-RQ-002

#### commit_document()

Commit document changes to Git repository.

**Parameters:**
- `repo_path`: Path to Git repository
- `file_path`: Relative path to document
- `message`: Commit message

**Returns:**
- `Result<String, GitError>`
  - String is 40-character SHA-1 commit hash
  - GitError on commit failure

**Invariants:**
- Pre-condition: Repository is initialized
- Post-condition: Commit is recorded in Git history

**Traceability:** CM-RQ-003, CM-RQ-005, CM-RQ-006

#### index_document()

Index document for full-text search.

**Parameters:**
- `document_id`: Unique document identifier
- `content`: Document content (plain text)
- `metadata`: Document metadata (title, tags)

**Returns:**
- `Result<(), IndexError>`
  - Empty on success
  - IndexError on indexing failure

**Performance Constraints:**
- Indexing time: <500ms per document

**Traceability:** SD-RQ-002

### 4.2. Rendering Engine

#### render_document()

Render document to HTML with JIT compilation.

**Parameters:**
- `document_path`: Path to Markdown file
- `commit_hash`: Git commit hash for versioning
- `user_role`: User role for RBAC redaction

**Returns:**
- `Result<String, RenderError>`
  - String is rendered HTML
  - RenderError on rendering failure

**Performance Constraints:**
- Cache hit: <1ms
- Cache miss: <15ms

**Traceability:** RE-RQ-001, RE-RQ-002, RE-RQ-005

#### invalidate_cache()

Invalidate cached HTML for a document.

**Parameters:**
- `cache_key`: SHA256(file_path || commit_hash || user_role)

**Returns:**
- `Result<(), CacheError>`
  - Empty on success
  - CacheError on invalidation failure

**Traceability:** RE-RQ-006

### 4.3. Search Engine

#### search()

Execute full-text search query.

**Parameters:**
- `query`: Search query string (1-256 characters)
- `limit`: Maximum results (default 100)

**Returns:**
- `Result<Vec<SearchResult>, SearchError>`
  - Vec<SearchResult> is ranked by BM25 score
  - SearchError on query execution failure

**Performance Constraints:**
- Query time: <100ms

**Traceability:** SD-RQ-001, SD-RQ-003

#### index_document()

Index document for search.

**Parameters:**
- `document_id`: Unique document identifier
- `content`: Document content
- `metadata`: Document metadata

**Returns:**
- `Result<(), IndexError>`
  - Empty on success
  - IndexError on indexing failure

**Performance Constraints:**
- Indexing time: <500ms per document

**Traceability:** SD-RQ-002

---

## 5. Data Models

### 5.1. ParsedDocument

```typescript
interface ParsedDocument {
  ast: AST;
  frontmatter: Frontmatter;
  metadata: DocumentMetadata;
}

interface Frontmatter {
  title: string;
  description?: string;
  tags?: string[];
  access: "public" | "restricted" | "internal";
  groups?: string[];
  date?: string;
  author?: string;
}

interface DocumentMetadata {
  title: string;
  tags?: string[];
  access?: string;
  groups?: string[];
  date?: string;
  author?: string;
}
```

### 5.2. SearchResult

```typescript
interface SearchResult {
  path: string;
  title: string;
  snippet: string;
  score: number;
  highlighted_terms: string[];
  commit_hash: string;
  modified_at: string;
}
```

### 5.3. Commit

```typescript
interface Commit {
  hash: string;
  message: string;
  author: string;
  timestamp: string;
}
```

### 5.4. Diff

```typescript
interface Diff {
  path: string;
  from_hash: string;
  to_hash: string;
  diff: string;
}
```

---

## 6. Error Handling

### 6.1. Error Response Format

All errors follow consistent format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

### 6.2. Error Codes

| Code | HTTP Status | Message |
|-------|-------------|---------|
| PARSE_ERROR | 422 | Failed to parse Markdown |
| GIT_ERROR | 500 | Git operation failed |
| RENDER_ERROR | 500 | Rendering failed |
| SEARCH_ERROR | 500 | Search query failed |
| CACHE_ERROR | 500 | Cache operation failed |
| NOT_FOUND | 404 | Resource not found |
| UNAUTHORIZED | 401 | Authentication required |
| FORBIDDEN | 403 | Insufficient permissions |
| VALIDATION_ERROR | 422 | Request validation failed |
| RATE_LIMIT | 429 | Too many requests |

### 6.3. Error Handling Best Practices

**Client-Side:**
1. Always check `error` field in responses
2. Implement exponential backoff for 429 errors
3. Cache authentication tokens securely
4. Display user-friendly error messages

**Server-Side:**
1. Return appropriate HTTP status codes
2. Include detailed error information
3. Log errors for debugging
4. Never expose sensitive data in error messages

---

## 7. Authentication and Authorization

### 7.1. Authentication Methods

Tachyon supports multiple authentication methods:

| Method | Description | Status |
|---------|-------------|--------|
| Bearer Token | JWT-based authentication | Supported |
| API Key | Header-based authentication | Supported |
| OAuth 2.0 | Third-party authentication | Planned |

### 7.2. Token Endpoints

#### POST /api/v1/auth/login

Authenticate with username/password.

**Request Body:**

```json
{
  "username": "user@example.com",
  "password": "secure_password"
}
```

**Response (200 OK):**

```json
{
  "data": {
    "token": "jwt_token_here",
    "expires_in": 3600,
    "user": {
      "id": "user_123",
      "username": "user@example.com",
      "roles": ["admin", "editor"]
    }
  },
  "error": null
}
```

#### POST /api/v1/auth/refresh

Refresh expired authentication token.

**Request Headers:**

```
Authorization: Bearer <expired_token>
```

**Response (200 OK):**

```json
{
  "data": {
    "token": "new_jwt_token_here",
    "expires_in": 3600
  },
  "error": null
}
```

#### POST /api/v1/auth/logout

Invalidate authentication token.

**Request Headers:**

```
Authorization: Bearer <token>
```

**Response (200 OK):**

```json
{
  "data": {
    "message": "Logged out successfully"
  },
  "error": null
}
```

### 7.3. Role-Based Access Control

Tachyon implements RBAC at the parsing level. Access control is enforced based on:

1. **Frontmatter access field:** `public`, `restricted`, `internal`
2. **Group membership:** User must be member of specified groups
3. **Internal blocks:** Content marked with `::: internal` is redacted

**Access Levels:**

| Level | Description | Behavior |
|-------|-------------|----------|
| `public` | No access restrictions | Accessible to all authenticated users |
| `restricted` | Group-based access | Accessible only to users in specified groups |
| `internal` | Administrator only | Accessible only to users with admin role |

---

## 8. Rate Limiting

### 8.1. Rate Limits

| Endpoint | Limit | Window | Burst Allowance |
|-----------|-------|--------------|---------------|
| Search API | 100 requests/minute | 10 requests | 120 requests/minute |
| Document API | 60 requests/minute | 10 requests | 60 requests/minute |
| Git API | 30 requests/minute | 10 requests | 30 requests/minute |
| WebSocket | 10 connections/minute | 5 connections | 20 connections/minute |

### 8.2. Rate Limit Headers

When rate limits are enforced, response headers include:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

### 8.3. Rate Limit Error Response

```json
{
  "error": {
    "code": "RATE_LIMIT",
    "message": "Rate limit exceeded",
    "details": {
      "limit": 100,
      "remaining": 95,
      "reset_at": 1640995200
    }
  }
}
```

---

## 9. Webhooks (Planned)

### 9.1. Webhook Overview

Tachyon will support webhook notifications for:

- Document changes
- Commit events
- Search index updates
- User activity

### 9.2. Webhook Registration

**Planned Endpoint:**

```
POST /api/v1/webhooks
```

**Request Body (Planned):**

```json
{
  "url": "https://example.com/webhook",
  "events": ["document.created", "document.updated"],
  "secret": "webhook_secret_here"
}
```

---

## Appendix A: Quick Reference

### A.1. HTTP Status Codes Quick Reference

| Status | Use Case |
|---------|----------|
| 200 | Success |
| 201 | Resource created |
| 400 | Validation error |
| 401 | Authentication required |
| 403 | Forbidden |
| 404 | Not found |
| 409 | Conflict |
| 422 | Unprocessable |
| 429 | Rate limit |
| 500 | Server error |

### A.2. Common Headers

| Header | Description | Example |
|---------|-------------|----------|
| Authorization | Bearer token | `Authorization: Bearer eyJhb...` |
| Content-Type | Request format | `Content-Type: application/json` |
| Accept | Response format | `Accept: application/json` |

---

**Document Control**

| Version | Date | Author | Changes |
|---------|-------|--------|----------|
| 1.0 | 2026-02-11 | Brand Strategist | Initial API reference from verified implementation |

---

**Accessibility Statement:** This document is WCAG 2.1 AA compliant with proper heading structure, sufficient color contrast, and keyboard navigation support.
