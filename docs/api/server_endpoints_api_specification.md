# TACHYON: SERVER ENDPOINTS API SPECIFICATION

**Document ID:** TACHYON-API-009-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** Technical Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 830-1998, RFC 7540 (HTTP/2), RFC 7231 (HTTP/1.1)
**Dependencies:** [TACHYON-STD-V1.0](../../.adrs/ [TACHYON-REQ-SRV-V1.0](../../.adrs/ [TACHYON-DSN-SRV-V1.0](../../.adrs/ [TACHYON-ADR-003-V1.0](../../.adrs/adr-003-lru-cache-target.md), [TACHYON-ADR-007-V1.0](../../.adrs/adr-007-thread-safety-strategy.md), [TACHYON-TMA-V1.0](../../.adrs/

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Endpoint Design Principles](#2-endpoint-design-principments)
3. [Document Endpoints](#3-document-endpoints)
4. [Repository Endpoints](#4-repository-endpoints)
5. [Search Endpoints](#5-search-endpoints)
6. [User Endpoints](#6-user-endpoints)
7. [Authentication Endpoints](#7-authentication-endpoints)
8. [Endpoint Security](#8-endpoint-security)
9. [Endpoint Performance](#9-endpoint-performance)
10. [References](#10-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides a comprehensive specification of all HTTP/2 RESTful API endpoints exposed by the Tachyon Server Application. Each endpoint is formally defined with HTTP method, path, request parameters, request body schema, response codes, response body schema, error conditions, and security requirements. This specification serves as the authoritative reference for server implementation, client integration, and API testing.

### 1.2. Scope

This specification encompasses all server-side HTTP/2 endpoints:

**In Scope:**
- Document management endpoints (CRUD operations)
- Repository management endpoints (Git operations)
- Search and query endpoints
- User management endpoints
- Authentication and authorization endpoints
- WebSocket upgrade endpoints
- Health check and monitoring endpoints

**Out of Scope:**
- WebSocket message protocol (covered in WebSocket API specification)
- Desktop IPC commands (covered in Desktop Commands API specification)
- Web component internal APIs (covered in Web API specification)
- Internal server architecture (covered in Server Design documentation)

### 1.3. Conventions

#### 1.3.1. Notation Conventions

This specification uses the following notation conventions:

| Notation | Meaning |
|----------|---------|
| `{variable}` | Path parameter that must be substituted |
| `[variable]` | Query parameter that is optional |
| `variable` | Query parameter that is required |
| `|` | Alternative choices for a parameter value |
| `...` | Repeated elements in an array |

#### 1.3.2. HTTP Status Codes

The following HTTP status codes are used throughout this specification:

| Status Code | Category | Meaning |
|-------------|----------|---------|
| 200 OK | Success | Request succeeded |
| 201 Created | Success | Resource created successfully |
| 204 No Content | Success | Request succeeded, no response body |
| 400 Bad Request | Client Error | Invalid request parameters or body |
| 401 Unauthorized | Client Error | Authentication required or failed |
| 403 Forbidden | Client Error | Insufficient permissions |
| 404 Not Found | Client Error | Resource not found |
| 409 Conflict | Client Error | Resource conflict (e.g., duplicate) |
| 422 Unprocessable Entity | Client Error | Semantic validation error |
| 429 Too Many Requests | Client Error | Rate limit exceeded |
| 500 Internal Server Error | Server Error | Unexpected server error |
| 503 Service Unavailable | Server Error | Service temporarily unavailable |

#### 1.3.3. Response Format Conventions

All JSON responses follow a consistent structure:

```json
{
  "data": { /* Response data */ },
  "meta": { /* Metadata (pagination, timestamps, etc.) */ },
  "errors": [ /* Error messages, if any */ ]
}
```

### 1.4. System Context

The Tachyon Server Endpoints operate within the following architectural context:

```mermaid
graph TB
    subgraph "Client Layer"
        Desktop[Tauri Desktop]
        Web[Leptos Web]
        API_Client[REST API Client]
    end
    
    subgraph "Server Layer - Axum HTTP/2"
        Router[Router Layer]
        Auth[Auth Middleware]
        RBAC[RBAC Middleware]
        Handlers[Endpoint Handlers]
    end
    
    subgraph "Service Layer"
        DocSvc[Document Service]
        RepoSvc[Repository Service]
        SearchSvc[Search Service]
        UserSvc[User Service]
        AuthSvc[Auth Service]
    end
    
    subgraph "Data Layer"
        Git[Git Repository]
        SQLite[(SQLite Database)]
        Tantivy[(Tantivy Index)]
        Cache[(LRU Cache)]
    end
    
    Desktop -->|HTTP/2| Router
    Web -->|HTTP/2| Router
    API_Client -->|HTTP/2| Router
    Router --> Auth
    Auth --> RBAC
    RBAC --> Handlers
    Handlers --> DocSvc
    Handlers --> RepoSvc
    Handlers --> SearchSvc
    Handlers --> UserSvc
    Handlers --> AuthSvc
    DocSvc --> Git
    DocSvc --> Cache
    RepoSvc --> Git
    SearchSvc --> Tantivy
    UserSvc --> SQLite
    AuthSvc --> SQLite
    
    style Router fill:#e1f5ff
    style Git fill:#ffe1e1
    style SQLite fill:#e1ffe1
    style Tantivy fill:#fff5e1
```

---

## 2. ENDPOINT DESIGN PRINCIPLES

### 2.1. RESTful Design

All endpoints adhere to RESTful architectural principles:

| Principle | Implementation |
|-----------|----------------|
| **Resource Identification** | Each resource is identified by a unique URI path |
| **Uniform Interface** | Standard HTTP methods (GET, POST, PUT, DELETE, PATCH) |
| **Stateless** | Each request contains all necessary information |
| **Cacheable** | Responses include appropriate cache-control headers |
| **Layered System** | Client cannot distinguish between proxy and server |
| **Code on Demand** | Optional: JavaScript can be returned for progressive enhancement |

### 2.2. HTTP Method Semantics

| Method | Idempotent | Safe | Semantics |
|--------|------------|------|-----------|
| GET | Yes | Yes | Retrieve resource representation |
| HEAD | Yes | Yes | Retrieve resource metadata only |
| POST | No | No | Create new resource or trigger action |
| PUT | Yes | No | Replace entire resource |
| PATCH | No | No | Partial resource update |
| DELETE | Yes | No | Delete resource |

### 2.3. URI Design Conventions

All URIs follow these conventions:

1. **Noun-based paths:** Use nouns, not verbs (e.g., `/documents` not `/getDocuments`)
2. **Plural nouns:** Use plural form for collection resources (e.g., `/documents`, `/users`)
3. **Hierarchical structure:** Reflect resource relationships (e.g., `/documents/{id}/revisions`)
4. **Lowercase with hyphens:** Use lowercase with hyphens for readability (e.g., `/search-results`)
5. **Query parameters:** Use for filtering, sorting, and pagination
6. **Versioning:** API version in URL path (e.g., `/api/v1/documents`)

### 2.4. Request/Response Design

#### 2.4.1. Request Headers

Standard request headers used across endpoints:

| Header | Purpose | Example |
|--------|---------|---------|
| `Authorization` | Bearer token authentication | `Bearer eyJhbGciOiJIUzI1NiIs...` |
| `Content-Type` | Request body media type | `application/json` |
| `Accept` | Desired response media type | `application/json` |
| `Accept-Encoding` | Preferred content encoding | `gzip, deflate, br` |
| `If-Match` | Conditional request (ETag) | `"33a64df551425fcc55e4d42a148795d9f25f89d4"` |
| `If-None-Match` | Conditional cache validation | `"33a64df551425fcc55e4d42a148795d9f25f89d4"` |

#### 2.4.2. Response Headers

Standard response headers used across endpoints:

| Header | Purpose | Example |
|--------|---------|---------|
| `Content-Type` | Response body media type | `application/json; charset=utf-8` |
| `Content-Encoding` | Content encoding applied | `gzip` |
| `ETag` | Entity tag for caching | `"33a64df551425fcc55e4d42a148795d9f25f89d4"` |
| `Cache-Control` | Cache directives | `public, max-age=3600` |
| `Last-Modified` | Last modification timestamp | `Wed, 21 Oct 2015 07:28:00 GMT` |
| `X-Request-ID` | Request tracing identifier | `550e8400-e29b-41d4-a716-446655440000` |
| `X-RateLimit-Limit` | Rate limit quota | `100` |
| `X-RateLimit-Remaining` | Remaining requests | `95` |
| `X-RateLimit-Reset` | Rate limit reset timestamp | `1643723400` |

### 2.5. Error Response Format

All error responses follow a consistent structure:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": [
      {
        "field": "title",
        "message": "Title must be between 1 and 200 characters"
      }
    ],
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-05T16:33:32.070Z"
  }
}
```

#### 2.5.1. Error Codes

Standard error codes used across endpoints:

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `AUTHENTICATION_FAILED` | 401 | Authentication credentials invalid |
| `AUTHORIZATION_FAILED` | 403 | Insufficient permissions |
| `RESOURCE_NOT_FOUND` | 404 | Requested resource does not exist |
| `RESOURCE_CONFLICT` | 409 | Resource already exists or state conflict |
| `RATE_LIMIT_EXCEEDED` | 429 | Request rate limit exceeded |
| `INTERNAL_SERVER_ERROR` | 500 | Unexpected server error |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |

### 2.6. Pagination Design

List endpoints support pagination using cursor-based pagination:

#### 2.6.1. Pagination Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | No | Maximum number of items to return (default: 50, max: 100) |
| `cursor` | string | No | Pagination cursor for next page (opaque string) |

#### 2.6.2. Pagination Response

```json
{
  "data": [ /* Array of items */ ],
  "meta": {
    "pagination": {
      "next_cursor": "eyJpZCI6IjEyMzQ1Njc4OTAiLCJjcmVhdGVkX2F0IjoiMjAyNi0wMi0wNVQxNjozMzozMi4wNzBaIn0=",
      "has_next": true,
      "limit": 50,
      "count": 50
    }
  }
}
```

### 2.7. Filtering and Sorting Design

List endpoints support filtering and sorting:

#### 2.7.1. Filtering Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `filter[{field}]` | string | Filter by field value (supports operators: `eq`, `ne`, `gt`, `lt`, `gte`, `lte`, `contains`) |
| `filter[{field}][operator]` | string | Filter with specific operator |

Examples:
- `?filter[status]=published` (equals)
- `?filter[created_at][gte]=2026-01-01` (greater than or equal)
- `?filter[title][contains]=tachyon` (contains substring)

#### 2.7.2. Sorting Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `sort` | string | Sort field with direction (e.g., `created_at:asc`, `title:desc`) |

Multiple sort fields can be specified: `?sort=created_at:desc,title:asc`

### 2.8. Versioning Strategy

The API uses URI-based versioning:

- Current version: `/api/v1/`
- Version format: `/api/v{major}/`
- Major version changes indicate breaking changes
- Minor version changes are backwards compatible
- Deprecation policy: Minimum 6 months notice before major version removal

---

## 3. DOCUMENT ENDPOINTS

### 3.1. List Documents

Retrieves a paginated list of documents accessible to the authenticated user.

#### 3.1.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/documents` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 3.1.2. Request Parameters

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 50 | Maximum items to return (1-100) |
| `cursor` | string | No | - | Pagination cursor |
| `filter[status]` | string | No | - | Filter by document status (`draft`, `published`, `archived`) |
| `filter[author_id]` | string | No | - | Filter by author user ID |
| `filter[created_at][gte]` | datetime | No | - | Filter by creation date (greater than or equal) |
| `filter[created_at][lte]` | datetime | No | - | Filter by creation date (less than or equal) |
| `sort` | string | No | `created_at:desc` | Sort field and direction |

#### 3.1.3. Request Example

```http
GET /api/v1/documents?limit=20&filter[status]=published&sort=created_at:desc HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 3.1.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "id": "doc_550e8400-e29b-41d4-a716-446655440000",
      "title": "Introduction to Tachyon",
      "slug": "introduction-to-tachyon",
      "status": "published",
      "author_id": "user_123e4567-e89b-12d3-a456-426614174000",
      "author_name": "John Doe",
      "created_at": "2026-02-01T10:30:00.000Z",
      "updated_at": "2026-02-05T14:20:00.000Z",
      "published_at": "2026-02-03T09:15:00.000Z",
      "word_count": 1250,
      "read_time_minutes": 5,
      "tags": ["documentation", "getting-started"],
      "excerpt": "Tachyon is a modern documentation toolchain..."
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": "eyJpZCI6ImRvY18xMjM0NTY3ODkwIiwiY3JlYXRlZF9hdCI6IjIwMjYtMDItMDFUMTA6MzA6MDAuMDAwWiJ9",
      "has_next": true,
      "limit": 20,
      "count": 20
    }
  }
}
```

#### 3.1.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 3.1.6. Related Requirements

- [REQ-SRV-021](../../.adrs/ - Document List Endpoint

---

### 3.2. Retrieve Document

Retrieves a specific document by ID with full content and metadata.

#### 3.2.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/documents/{id}` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 3.2.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document UUID identifier |

#### 3.2.3. Request Example

```http
GET /api/v1/documents/doc_550e8400-e29b-41d4-a716-446655440000 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 3.2.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "id": "doc_550e8400-e29b-41d4-a716-446655440000",
    "title": "Introduction to Tachyon",
    "slug": "introduction-to-tachyon",
    "status": "published",
    "author_id": "user_123e4567-e89b-12d3-a456-426614174000",
    "author_name": "John Doe",
    "created_at": "2026-02-01T10:30:00.000Z",
    "updated_at": "2026-02-05T14:20:00.000Z",
    "published_at": "2026-02-03T09:15:00.000Z",
    "content": "# Introduction to Tachyon\n\nTachyon is a modern documentation toolchain...",
    "content_type": "text/markdown",
    "frontmatter": {
      "title": "Introduction to Tachyon",
      "author": "John Doe",
      "date": "2026-02-01",
      "tags": ["documentation", "getting-started"],
      "access": "public",
      "roles": []
    },
    "rendered_html": "<h1>Introduction to Tachyon</h1>\n<p>Tachyon is a modern documentation toolchain...</p>",
    "word_count": 1250,
    "read_time_minutes": 5,
    "tags": ["documentation", "getting-started"],
    "revisions_count": 5,
    "current_revision_id": "rev_98765432-10ab-cdef-1234-567890abcdef",
    "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890"
  },
  "meta": {
    "etag": "\"33a64df551425fcc55e4d42a148795d9f25f89d4\""
  }
}
```

#### 3.2.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid document ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to access document |
| 404 | `RESOURCE_NOT_FOUND` | Document not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 3.2.6. Related Requirements

- [REQ-SRV-022](../../.adrs/ - Document Retrieval Endpoint
- [REQ-SRV-083](../../.adrs/ - Block Redaction

---

### 3.3. Create Document

Creates a new document in the repository.

#### 3.3.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/documents` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 3.3.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Document title (1-200 characters) |
| `content` | string | Yes | Document content in Markdown format |
| `slug` | string | No | URL-friendly slug (auto-generated if not provided) |
| `status` | string | No | Document status (`draft`, `published`, `archived`; default: `draft`) |
| `tags` | array of string | No | Document tags |
| `frontmatter` | object | No | YAML frontmatter metadata |

#### 3.3.3. Request Example

```http
POST /api/v1/documents HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "title": "Getting Started with Tachyon",
  "content": "# Getting Started\n\nThis guide will help you get started...",
  "status": "draft",
  "tags": ["tutorial", "getting-started"]
}
```

#### 3.3.4. Response Definition

**Success Response (201 Created):**

```json
{
  "data": {
    "id": "doc_550e8400-e29b-41d4-a716-446655440001",
    "title": "Getting Started with Tachyon",
    "slug": "getting-started-with-tachyon",
    "status": "draft",
    "author_id": "user_123e4567-e89b-12d3-a456-426614174000",
    "author_name": "John Doe",
    "created_at": "2026-02-05T16:33:32.070Z",
    "updated_at": "2026-02-05T16:33:32.070Z",
    "published_at": null,
    "content": "# Getting Started\n\nThis guide will help you get started...",
    "content_type": "text/markdown",
    "frontmatter": {
      "title": "Getting Started with Tachyon",
      "author": "John Doe",
      "date": "2026-02-05",
      "tags": ["tutorial", "getting-started"],
      "access": "private",
      "roles": []
    },
    "rendered_html": "<h1>Getting Started</h1>\n<p>This guide will help you get started...</p>",
    "word_count": 42,
    "read_time_minutes": 1,
    "tags": ["tutorial", "getting-started"],
    "revisions_count": 1,
    "current_revision_id": "rev_98765432-10ab-cdef-1234-567890abcde1",
    "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890"
  }
}
```

#### 3.3.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to create documents |
| 409 | `RESOURCE_CONFLICT` | Document with same slug already exists |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 3.3.6. Related Requirements

- [REQ-SRV-023](../../.adrs/ - Document Creation Endpoint
- [REQ-SRV-047](../../.adrs/ - Commit Management

---

### 3.4. Update Document

Updates an existing document. Supports partial updates (PATCH) or full replacement (PUT).

#### 3.4.1. Endpoint Definition (PATCH)

| Property | Value |
|----------|-------|
| **Method** | `PATCH` |
| **Path** | `/api/v1/documents/{id}` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 3.4.2. Endpoint Definition (PUT)

| Property | Value |
|----------|-------|
| **Method** | `PUT` |
| **Path** | `/api/v1/documents/{id}` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Required |

#### 3.4.3. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document UUID identifier |

#### 3.4.4. Request Body (PATCH)

**Content-Type:** `application/json`

All fields are optional. Only provided fields are updated.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | No | Document title (1-200 characters) |
| `content` | string | No | Document content in Markdown format |
| `slug` | string | No | URL-friendly slug |
| `status` | string | No | Document status (`draft`, `published`, `archived`) |
| `tags` | array of string | No | Document tags |
| `frontmatter` | object | No | YAML frontmatter metadata |

#### 3.4.5. Request Example (PATCH)

```http
PATCH /api/v1/documents/doc_550e8400-e29b-41d4-a716-446655440000 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
If-Match: "33a64df551425fcc55e4d42a148795d9f25f89d4"

{
  "title": "Introduction to Tachyon (Updated)",
  "status": "published"
}
```

#### 3.4.6. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "id": "doc_550e8400-e29b-41d4-a716-446655440000",
    "title": "Introduction to Tachyon (Updated)",
    "slug": "introduction-to-tachyon",
    "status": "published",
    "author_id": "user_123e4567-e89b-12d3-a456-426614174000",
    "author_name": "John Doe",
    "created_at": "2026-02-01T10:30:00.000Z",
    "updated_at": "2026-02-05T16:35:00.000Z",
    "published_at": "2026-02-05T16:35:00.000Z",
    "content": "# Introduction to Tachyon\n\nTachyon is a modern documentation toolchain...",
    "content_type": "text/markdown",
    "frontmatter": {
      "title": "Introduction to Tachyon (Updated)",
      "author": "John Doe",
      "date": "2026-02-01",
      "tags": ["documentation", "getting-started"],
      "access": "public",
      "roles": []
    },
    "rendered_html": "<h1>Introduction to Tachyon (Updated)</h1>\n<p>Tachyon is a modern documentation toolchain...</p>",
    "word_count": 1250,
    "read_time_minutes": 5,
    "tags": ["documentation", "getting-started"],
    "revisions_count": 6,
    "current_revision_id": "rev_98765432-10ab-cdef-1234-567890abcde2",
    "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890"
  },
  "meta": {
    "etag": "\"44b75ef662536gdd60f5e53b259a6e0g36g90e5\""
  }
}
```

#### 3.4.7. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body or document ID |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to update document |
| 404 | `RESOURCE_NOT_FOUND` | Document not found |
| 409 | `RESOURCE_CONFLICT` | Document with same slug already exists |
| 412 | `PRECONDITION_FAILED` | ETag mismatch (concurrent modification) |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 3.4.8. Related Requirements

- [REQ-SRV-024](../../.adrs/ - Document Update Endpoint
- [REQ-SRV-047](../../.adrs/ - Commit Management

---

### 3.5. Delete Document

Deletes an existing document.

#### 3.5.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `DELETE` |
| **Path** | `/api/v1/documents/{id}` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Required |

#### 3.5.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document UUID identifier |

#### 3.5.3. Request Example

```http
DELETE /api/v1/documents/doc_550e8400-e29b-41d4-a716-446655440000 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### 3.5.4. Response Definition

**Success Response (204 No Content):**

Empty response body with status code 204.

#### 3.5.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid document ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to delete document |
| 404 | `RESOURCE_NOT_FOUND` | Document not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 3.5.6. Related Requirements

- [REQ-SRV-025](../../.adrs/ - Document Deletion Endpoint
- [REQ-SRV-047](../../.adrs/ - Commit Management

---

### 3.6. Document Revisions

Retrieves revision history for a specific document.

#### 3.6.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/documents/{id}/revisions` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 3.6.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document UUID identifier |

#### 3.6.3. Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 50 | Maximum items to return (1-100) |
| `cursor` | string | No | - | Pagination cursor |

#### 3.6.4. Request Example

```http
GET /api/v1/documents/doc_550e8400-e29b-41d4-a716-446655440000/revisions?limit=10 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 3.6.5. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "id": "rev_98765432-10ab-cdef-1234-567890abcde2",
      "document_id": "doc_550e8400-e29b-41d4-a716-446655440000",
      "commit_hash": "a1b2c3d4e5f6789012345678901234567890abcd",
      "author_id": "user_123e4567-e89b-12d3-a456-426614174000",
      "author_name": "John Doe",
      "created_at": "2026-02-05T16:35:00.000Z",
      "message": "Update title and publish document",
      "changes": {
        "title": {
          "from": "Introduction to Tachyon",
          "to": "Introduction to Tachyon (Updated)"
        },
        "status": {
          "from": "draft",
          "to": "published"
        }
      }
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": null,
      "has_next": false,
      "limit": 10,
      "count": 6
    }
  }
}
```

#### 3.6.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid document ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to view revisions |
| 404 | `RESOURCE_NOT_FOUND` | Document not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 3.6.7. Related Requirements

- [REQ-SRV-032](../../.adrs/ - Commit History Endpoint

---

## 4. REPOSITORY ENDPOINTS

### 4.1. List Repositories

Retrieves a paginated list of repositories accessible to the authenticated user.

#### 4.1.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/repositories` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 4.1.2. Request Parameters

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 50 | Maximum items to return (1-100) |
| `cursor` | string | No | - | Pagination cursor |
| `filter[visibility]` | string | No | - | Filter by visibility (`public`, `private`) |
| `filter[owner_id]` | string | No | - | Filter by owner user ID |
| `sort` | string | No | `created_at:desc` | Sort field and direction |

#### 4.1.3. Request Example

```http
GET /api/v1/repositories?limit=20&filter[visibility]=public&sort=created_at:desc HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 4.1.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "id": "repo_abcdef12-3456-7890-abcd-ef1234567890",
      "name": "tachyon-docs",
      "description": "Tachyon documentation repository",
      "visibility": "public",
      "owner_id": "user_123e4567-e89b-12d3-a456-426614174000",
      "owner_name": "John Doe",
      "default_branch": "main",
      "current_branch": "main",
      "created_at": "2026-01-15T10:00:00.000Z",
      "updated_at": "2026-02-05T14:20:00.000Z",
      "document_count": 42,
      "branch_count": 3,
      "remote_url": "https://github.com/example/tachyon-docs.git",
      "sync_status": "synced",
      "last_sync_at": "2026-02-05T14:20:00.000Z"
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": null,
      "has_next": false,
      "limit": 20,
      "count": 1
    }
  }
}
```

#### 4.1.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.1.6. Related Requirements

- [REQ-SRV-046](../../.adrs/ - Repository Access

---

### 4.2. Retrieve Repository

Retrieves a specific repository by ID with full metadata.

#### 4.2.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/repositories/{id}` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 4.2.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Repository UUID identifier |

#### 4.2.3. Request Example

```http
GET /api/v1/repositories/repo_abcdef12-3456-7890-abcd-ef1234567890 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 4.2.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "id": "repo_abcdef12-3456-7890-abcd-ef1234567890",
    "name": "tachyon-docs",
    "description": "Tachyon documentation repository",
    "visibility": "public",
    "owner_id": "user_123e4567-e89b-12d3-a456-426614174000",
    "owner_name": "John Doe",
    "default_branch": "main",
    "current_branch": "main",
    "created_at": "2026-01-15T10:00:00.000Z",
    "updated_at": "2026-02-05T14:20:00.000Z",
    "document_count": 42,
    "branch_count": 3,
    "remote_url": "https://github.com/example/tachyon-docs.git",
    "sync_status": "synced",
    "last_sync_at": "2026-02-05T14:20:00.000Z",
    "git_status": {
      "branch": "main",
      "ahead": 0,
      "behind": 0,
      "staged": 0,
      "unstaged": 0,
      "untracked": 0
    }
  }
}
```

#### 4.2.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid repository ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to access repository |
| 404 | `RESOURCE_NOT_FOUND` | Repository not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.2.6. Related Requirements

- [REQ-SRV-031](../../.adrs/ - Repository Status Endpoint

---

### 4.3. Add Repository

Adds a new Git repository to the Tachyon server.

#### 4.3.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/repositories` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 4.3.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Repository name (1-100 characters, alphanumeric and hyphens) |
| `description` | string | No | Repository description |
| `visibility` | string | No | Repository visibility (`public`, `private`; default: `private`) |
| `remote_url` | string | No | Git remote URL for syncing |
| `default_branch` | string | No | Default branch name (default: `main`) |

#### 4.3.3. Request Example

```http
POST /api/v1/repositories HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "name": "my-docs",
  "description": "My documentation repository",
  "visibility": "private",
  "remote_url": "https://github.com/user/my-docs.git",
  "default_branch": "main"
}
```

#### 4.3.4. Response Definition

**Success Response (201 Created):**

```json
{
  "data": {
    "id": "repo_bcdef123-4567-8901-bcde-f12345678901",
    "name": "my-docs",
    "description": "My documentation repository",
    "visibility": "private",
    "owner_id": "user_123e4567-e89b-12d3-a456-426614174000",
    "owner_name": "John Doe",
    "default_branch": "main",
    "current_branch": "main",
    "created_at": "2026-02-05T16:40:00.000Z",
    "updated_at": "2026-02-05T16:40:00.000Z",
    "document_count": 0,
    "branch_count": 1,
    "remote_url": "https://github.com/user/my-docs.git",
    "sync_status": "pending",
    "last_sync_at": null,
    "git_status": {
      "branch": "main",
      "ahead": 0,
      "behind": 0,
      "staged": 0,
      "unstaged": 0,
      "untracked": 0
    }
  }
}
```

#### 4.3.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to create repositories |
| 409 | `RESOURCE_CONFLICT` | Repository with same name already exists |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.3.6. Related Requirements

- [REQ-SRV-046](../../.adrs/ - Repository Access
- [REQ-SRV-050](../../.adrs/ - Repository Sync

---

### 4.4. Remove Repository

Removes a repository from the Tachyon server.

#### 4.4.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `DELETE` |
| **Path** | `/api/v1/repositories/{id}` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Required |

#### 4.4.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Repository UUID identifier |

#### 4.4.3. Request Example

```http
DELETE /api/v1/repositories/repo_abcdef12-3456-7890-abcd-ef1234567890 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### 4.4.4. Response Definition

**Success Response (204 No Content):**

Empty response body with status code 204.

#### 4.4.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid repository ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to delete repository |
| 404 | `RESOURCE_NOT_FOUND` | Repository not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.4.6. Related Requirements

- [REQ-SRV-046](../../.adrs/ - Repository Access

---

### 4.5. Sync Repository

Synchronizes a repository with its remote Git repository.

#### 4.5.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/repositories/{id}/sync` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 4.5.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Repository UUID identifier |

#### 4.5.3. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `direction` | string | No | Sync direction (`fetch`, `push`, `both`; default: `both`) |
| `branch` | string | No | Specific branch to sync (default: current branch) |

#### 4.5.4. Request Example

```http
POST /api/v1/repositories/repo_abcdef12-3456-7890-abcd-ef1234567890/sync HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "direction": "both",
  "branch": "main"
}
```

#### 4.5.5. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890",
    "sync_direction": "both",
    "sync_branch": "main",
    "status": "completed",
    "started_at": "2026-02-05T16:45:00.000Z",
    "completed_at": "2026-02-05T16:45:05.000Z",
    "duration_seconds": 5,
    "fetch_result": {
      "success": true,
      "commits_fetched": 3,
      "branches_updated": 1
    },
    "push_result": {
      "success": true,
      "commits_pushed": 2,
      "branches_updated": 1
    },
    "conflicts": [],
    "errors": []
  }
}
```

#### 4.5.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body or repository ID |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to sync repository |
| 404 | `RESOURCE_NOT_FOUND` | Repository not found |
| 409 | `RESOURCE_CONFLICT` | Merge conflicts detected |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.5.7. Related Requirements

- [REQ-SRV-050](../../.adrs/ - Repository Sync

---

### 4.6. List Branches

Retrieves a list of branches for a specific repository.

#### 4.6.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/repositories/{id}/branches` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 4.6.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Repository UUID identifier |

#### 4.6.3. Request Example

```http
GET /api/v1/repositories/repo_abcdef12-3456-7890-abcd-ef1234567890/branches HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 4.6.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "name": "main",
      "is_default": true,
      "is_current": true,
      "commit_hash": "a1b2c3d4e5f6789012345678901234567890abcd",
      "commit_short_hash": "a1b2c3d",
      "commit_message": "Update documentation",
      "commit_author": "John Doe",
      "commit_date": "2026-02-05T14:20:00.000Z",
      "ahead": 0,
      "behind": 0
    },
    {
      "name": "feature/new-api",
      "is_default": false,
      "is_current": false,
      "commit_hash": "b2c3d4e5f6789012345678901234567890abcde",
      "commit_short_hash": "b2c3d4e",
      "commit_message": "Add new API endpoints",
      "commit_author": "Jane Smith",
      "commit_date": "2026-02-04T10:15:00.000Z",
      "ahead": 2,
      "behind": 1
    }
  ]
}
```

#### 4.6.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid repository ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to view branches |
| 404 | `RESOURCE_NOT_FOUND` | Repository not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.6.6. Related Requirements

- [REQ-SRV-033](../../.adrs/ - Branch List Endpoint

---

### 4.7. Switch Branch

Switches the current branch for a repository.

#### 4.7.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/repositories/{id}/branches/switch` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 4.7.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Repository UUID identifier |

#### 4.7.3. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `branch` | string | Yes | Branch name to switch to |
| `create_if_not_exists` | boolean | No | Create branch if it doesn't exist (default: `false`) |

#### 4.7.4. Request Example

```http
POST /api/v1/repositories/repo_abcdef12-3456-7890-abcd-ef1234567890/branches/switch HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "branch": "feature/new-api"
}
```

#### 4.7.5. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890",
    "previous_branch": "main",
    "current_branch": "feature/new-api",
    "commit_hash": "b2c3d4e5f6789012345678901234567890abcde",
    "commit_short_hash": "b2c3d4e",
    "switched_at": "2026-02-05T16:50:00.000Z"
  }
}
```

#### 4.7.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body or repository ID |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to switch branches |
| 404 | `RESOURCE_NOT_FOUND` | Repository or branch not found |
| 409 | `RESOURCE_CONFLICT` | Uncommitted changes prevent branch switch |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.7.7. Related Requirements

- [REQ-SRV-034](../../.adrs/ - Branch Switch Endpoint
- [REQ-SRV-048](../../.adrs/ - Branch Operations

---

### 4.8. Get Commit History

Retrieves commit history for a repository.

#### 4.8.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/repositories/{id}/commits` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 4.8.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Repository UUID identifier |

#### 4.8.3. Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 50 | Maximum items to return (1-100) |
| `cursor` | string | No | - | Pagination cursor |
| `branch` | string | No | current branch | Branch to get commits from |
| `since` | datetime | No | - | Start date for commit history |
| `until` | datetime | No | - | End date for commit history |

#### 4.8.4. Request Example

```http
GET /api/v1/repositories/repo_abcdef12-3456-7890-abcd-ef1234567890/commits?limit=20&branch=main HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 4.8.5. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "hash": "a1b2c3d4e5f6789012345678901234567890abcd",
      "short_hash": "a1b2c3d",
      "message": "Update documentation",
      "author_name": "John Doe",
      "author_email": "john@example.com",
      "authored_at": "2026-02-05T14:20:00.000Z",
      "committer_name": "John Doe",
      "committer_email": "john@example.com",
      "committed_at": "2026-02-05T14:20:00.000Z",
      "branch": "main",
      "files_changed": 5,
      "insertions": 42,
      "deletions": 12
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": "eyJhYXQiOiIyMDI2LTAyLTA1VDE0OjIwOjAwLjAwMFoiLCJoYXNoIjoiYTFiMmMzZCJ9",
      "has_next": true,
      "limit": 20,
      "count": 20
    }
  }
}
```

#### 4.8.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters or repository ID |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to view commits |
| 404 | `RESOURCE_NOT_FOUND` | Repository not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.8.7. Related Requirements

- [REQ-SRV-032](../../.adrs/ - Commit History Endpoint

---

### 4.9. Get Diff

Retrieves the diff between two commits or commits and working directory.

#### 4.9.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/repositories/{id}/diff` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 4.9.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Repository UUID identifier |

#### 4.9.3. Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `from` | string | No | HEAD | Starting commit hash or branch |
| `to` | string | No | working directory | Ending commit hash, branch, or `working` |
| `path` | string | No | - | Specific path to diff (optional) |

#### 4.9.4. Request Example

```http
GET /api/v1/repositories/repo_abcdef12-3456-7890-abcd-ef1234567890/diff?from=main&to=working HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 4.9.5. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "from": "main",
    "to": "working",
    "files": [
      {
        "path": "docs/introduction.md",
        "status": "modified",
        "additions": 15,
        "deletions": 5,
        "diff": "--- a/docs/introduction.md\n+++ b/docs/introduction.md\n@@ -1,5 +1,10 @@\n # Introduction\n\n-Tachyon is a tool.\n+Tachyon is a modern documentation toolchain.\n+It provides powerful features for...\n"
      },
      {
        "path": "docs/new-feature.md",
        "status": "added",
        "additions": 42,
        "deletions": 0,
        "diff": "+++ b/docs/new-feature.md\n@@ -0,0 +1,42 @@\n+# New Feature\n\nThis document describes the new feature...\n"
      }
    ],
    "summary": {
      "files_changed": 2,
      "additions": 57,
      "deletions": 5
    }
  }
}
```

#### 4.9.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters or repository ID |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to view diff |
| 404 | `RESOURCE_NOT_FOUND` | Repository or commit not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 4.9.7. Related Requirements

- [REQ-SRV-035](../../.adrs/ - Diff View Endpoint

---

## 5. SEARCH ENDPOINTS

### 5.1. Full-Text Search

Performs full-text search across all accessible documents using Tantivy search index.

#### 5.1.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/search` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 5.1.2. Request Parameters

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `q` | string | Yes | - | Search query string |
| `limit` | integer | No | 20 | Maximum results to return (1-100) |
| `cursor` | string | No | - | Pagination cursor |
| `repository_id` | string | No | - | Filter by repository ID |
| `tag` | string | No | - | Filter by tag |
| `author_id` | string | No | - | Filter by author ID |
| `created_after` | datetime | No | - | Filter by creation date (after) |
| `created_before` | datetime | No | - | Filter by creation date (before) |
| `highlight` | boolean | No | `true` | Enable search term highlighting |
| `snippet_length` | integer | No | 150 | Length of result snippet (50-500) |

#### 5.1.3. Request Example

```http
GET /api/v1/search?q=tachyon%20documentation&limit=10&highlight=true&snippet_length=200 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 5.1.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "document_id": "doc_550e8400-e29b-41d4-a716-446655440000",
      "title": "Introduction to Tachyon",
      "slug": "introduction-to-tachyon",
      "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890",
      "repository_name": "tachyon-docs",
      "author_id": "user_123e4567-e89b-12d3-a456-426614174000",
      "author_name": "John Doe",
      "score": 0.9542,
      "snippet": "Tachyon is a modern <mark>documentation</mark> toolchain that provides powerful features for creating, managing, and publishing technical <mark>documentation</mark>.",
      "highlights": [
        {
          "field": "title",
          "matches": ["Tachyon"]
        },
        {
          "field": "content",
          "matches": ["documentation", "toolchain"]
        }
      ],
      "tags": ["documentation", "getting-started"],
      "created_at": "2026-02-01T10:30:00.000Z",
      "updated_at": "2026-02-05T14:20:00.000Z"
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": "eyJzY29yZSI6MC45NTQyLCJkb2NfaWQiOiJkb2NfNTUwZTg0MDAifQ==",
      "has_next": true,
      "limit": 10,
      "count": 10
    },
    "search": {
      "query": "tachyon documentation",
      "total_results": 42,
      "search_time_ms": 15,
      "index_version": "v1.2.3"
    }
  }
}
```

#### 5.1.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 422 | `UNPROCESSABLE_ENTITY` | Invalid search query syntax |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 5.1.6. Related Requirements

- [REQ-SRV-026](../../.adrs/ - Full-Text Search Endpoint
- [REQ-SRV-056](../../.adrs/ - Tantivy Integration

---

### 5.2. Search Autocomplete

Provides autocomplete suggestions for search queries based on document content and metadata.

#### 5.2.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/search/autocomplete` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 5.2.2. Request Parameters

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `q` | string | Yes | - | Partial search query |
| `limit` | integer | No | 10 | Maximum suggestions to return (1-50) |
| `type` | string | No | `all` | Suggestion type (`all`, `title`, `tag`, `author`) |

#### 5.2.3. Request Example

```http
GET /api/v1/search/autocomplete?q=tach&limit=10&type=all HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 5.2.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "type": "title",
      "text": "Tachyon Documentation Guide",
      "completion": "Tachyon Documentation Guide",
      "document_id": "doc_550e8400-e29b-41d4-a716-446655440000"
    },
    {
      "type": "title",
      "text": "Tachyon API Reference",
      "completion": "Tachyon API Reference",
      "document_id": "doc_550e8400-e29b-41d4-a716-446655440001"
    },
    {
      "type": "tag",
      "text": "tachyon",
      "completion": "tag:tachyon",
      "count": 42
    },
    {
      "type": "author",
      "text": "Tachyon Team",
      "completion": "author:Tachyon Team",
      "user_id": "user_123e4567-e89b-12d3-a456-426614174000"
    }
  ]
}
```

#### 5.2.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 5.2.6. Related Requirements

- [REQ-SRV-028](../../.adrs/ - Search Autocomplete Endpoint

---

### 5.3. Faceted Search

Performs faceted search with advanced filtering capabilities.

#### 5.3.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/search/faceted` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 5.3.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `query` | string | Yes | Search query string |
| `filters` | object | No | Faceted filters |
| `filters.tags` | array of string | No | Filter by tags |
| `filters.authors` | array of string | No | Filter by author IDs |
| `filters.repositories` | array of string | No | Filter by repository IDs |
| `filters.date_range` | object | No | Date range filter |
| `filters.date_range.start` | datetime | No | Start date |
| `filters.date_range.end` | datetime | No | End date |
| `filters.status` | array of string | No | Filter by document status |
| `limit` | integer | No | Maximum results (default: 20, max: 100) |
| `cursor` | string | No | Pagination cursor |

#### 5.3.3. Request Example

```http
POST /api/v1/search/faceted HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "query": "api documentation",
  "filters": {
    "tags": ["api", "reference"],
    "status": ["published"],
    "date_range": {
      "start": "2026-01-01T00:00:00.000Z",
      "end": "2026-02-05T23:59:59.999Z"
    }
  },
  "limit": 20
}
```

#### 5.3.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "document_id": "doc_550e8400-e29b-41d4-a716-446655440000",
      "title": "Tachyon API Reference",
      "slug": "tachyon-api-reference",
      "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890",
      "repository_name": "tachyon-docs",
      "author_id": "user_123e4567-e89b-12d3-a456-426614174000",
      "author_name": "John Doe",
      "score": 0.9876,
      "snippet": "The Tachyon API provides a comprehensive set of endpoints for <mark>api</mark> <mark>documentation</mark> management...",
      "tags": ["api", "reference", "documentation"],
      "status": "published",
      "created_at": "2026-02-01T10:30:00.000Z",
      "updated_at": "2026-02-05T14:20:00.000Z"
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": null,
      "has_next": false,
      "limit": 20,
      "count": 5
    },
    "search": {
      "query": "api documentation",
      "total_results": 5,
      "search_time_ms": 22,
      "index_version": "v1.2.3"
    },
    "facets": {
      "tags": {
        "api": 5,
        "reference": 3,
        "documentation": 5
      },
      "authors": {
        "user_123e4567-e89b-12d3-a456-426614174000": 5
      },
      "repositories": {
        "repo_abcdef12-3456-7890-abcd-ef1234567890": 5
      },
      "status": {
        "published": 5
      }
    }
  }
}
```

#### 5.3.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 422 | `UNPROCESSABLE_ENTITY` | Invalid search query or filter syntax |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 5.3.6. Related Requirements

- [REQ-SRV-027](../../.adrs/ - Faceted Search Endpoint
- [REQ-SRV-029](../../.adrs/ - Search Pagination

---

### 5.4. Search Statistics

Retrieves statistics about the search index and search usage.

#### 5.4.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/search/statistics` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 5.4.2. Request Example

```http
GET /api/v1/search/statistics HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 5.4.3. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "index": {
      "document_count": 1250,
      "total_terms": 45000,
      "index_size_bytes": 52428800,
      "index_version": "v1.2.3",
      "last_updated": "2026-02-05T16:00:00.000Z",
      "last_optimized": "2026-02-04T02:00:00.000Z"
    },
    "repositories": [
      {
        "repository_id": "repo_abcdef12-3456-7890-abcd-ef1234567890",
        "repository_name": "tachyon-docs",
        "document_count": 42
      }
    ],
    "usage": {
      "total_searches_today": 1520,
      "average_response_time_ms": 18,
      "popular_queries": [
        {
          "query": "tachyon documentation",
          "count": 45
        },
        {
          "query": "api reference",
          "count": 32
        }
      ]
    }
  }
}
```

#### 5.4.4. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 5.4.5. Related Requirements

- [REQ-SRV-060](../../.adrs/ - Index Statistics Endpoint

---

## 6. USER ENDPOINTS

### 6.1. List Users

Retrieves a paginated list of users. Requires administrative privileges.

#### 6.1.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/users` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |
| **Authorization** | Admin required |

#### 6.1.2. Request Parameters

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 50 | Maximum items to return (1-100) |
| `cursor` | string | No | - | Pagination cursor |
| `filter[role]` | string | No | - | Filter by role (`admin`, `editor`, `viewer`) |
| `filter[status]` | string | No | - | Filter by status (`active`, `inactive`, `suspended`) |
| `sort` | string | No | `created_at:desc` | Sort field and direction |

#### 6.1.3. Request Example

```http
GET /api/v1/users?limit=20&filter[status]=active&sort=created_at:desc HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 6.1.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "id": "user_123e4567-e89b-12d3-a456-426614174000",
      "username": "john.doe",
      "email": "john.doe@example.com",
      "display_name": "John Doe",
      "avatar_url": "https://gravatar.com/avatar/...",
      "role": "editor",
      "status": "active",
      "created_at": "2026-01-15T10:00:00.000Z",
      "updated_at": "2026-02-05T14:20:00.000Z",
      "last_login_at": "2026-02-05T16:30:00.000Z",
      "document_count": 42,
      "repository_count": 3
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": null,
      "has_next": false,
      "limit": 20,
      "count": 1
    }
  }
}
```

#### 6.1.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions (admin required) |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 6.1.6. Related Requirements

- [REQ-SRV-081](../../.adrs/ - RBAC Enforcement

---

### 6.2. Retrieve User

Retrieves a specific user by ID. Users can retrieve their own profile; administrators can retrieve any user.

#### 6.2.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/users/{id}` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 6.2.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | User UUID identifier |

#### 6.2.3. Request Example

```http
GET /api/v1/users/user_123e4567-e89b-12d3-a456-426614174000 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 6.2.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "id": "user_123e4567-e89b-12d3-a456-426614174000",
    "username": "john.doe",
    "email": "john.doe@example.com",
    "display_name": "John Doe",
    "avatar_url": "https://gravatar.com/avatar/...",
    "role": "editor",
    "status": "active",
    "created_at": "2026-01-15T10:00:00.000Z",
    "updated_at": "2026-02-05T14:20:00.000Z",
    "last_login_at": "2026-02-05T16:30:00.000Z",
    "mfa_enabled": true,
    "mfa_type": "totp",
    "document_count": 42,
    "repository_count": 3,
    "permissions": {
      "can_create_documents": true,
      "can_edit_documents": true,
      "can_delete_documents": true,
      "can_create_repositories": false,
      "can_manage_users": false
    }
  }
}
```

#### 6.2.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid user ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to view user |
| 404 | `RESOURCE_NOT_FOUND` | User not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 6.2.6. Related Requirements

- [REQ-SRV-081](../../.adrs/ - RBAC Enforcement
- [REQ-SRV-084](../../.adrs/ - Principle of Least Privilege

---

### 6.3. Update User

Updates a user's profile information. Users can update their own profile; administrators can update any user.

#### 6.3.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `PATCH` |
| **Path** | `/api/v1/users/{id}` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 6.3.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | User UUID identifier |

#### 6.3.3. Request Body

**Content-Type:** `application/json`

All fields are optional. Only provided fields are updated.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display_name` | string | No | Display name (1-100 characters) |
| `email` | string | No | Email address (must be unique) |
| `avatar_url` | string | No | Avatar URL |
| `role` | string | No | User role (`admin`, `editor`, `viewer`; admin only) |
| `status` | string | No | User status (`active`, `inactive`, `suspended`; admin only) |

#### 6.3.4. Request Example

```http
PATCH /api/v1/users/user_123e4567-e89b-12d3-a456-426614174000 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "display_name": "John Doe Jr.",
  "email": "john.doe.jr@example.com"
}
```

#### 6.3.5. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "id": "user_123e4567-e89b-12d3-a456-426614174000",
    "username": "john.doe",
    "email": "john.doe.jr@example.com",
    "display_name": "John Doe Jr.",
    "avatar_url": "https://gravatar.com/avatar/...",
    "role": "editor",
    "status": "active",
    "created_at": "2026-01-15T10:00:00.000Z",
    "updated_at": "2026-02-05T16:40:00.000Z",
    "last_login_at": "2026-02-05T16:30:00.000Z",
    "mfa_enabled": true,
    "mfa_type": "totp",
    "document_count": 42,
    "repository_count": 3
  }
}
```

#### 6.3.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body or user ID |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to update user |
| 404 | `RESOURCE_NOT_FOUND` | User not found |
| 409 | `RESOURCE_CONFLICT` | Email already in use |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 6.3.7. Related Requirements

- [REQ-SRV-081](../../.adrs/ - RBAC Enforcement

---

### 6.4. Delete User

Deletes a user account. Requires administrative privileges.

#### 6.4.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `DELETE` |
| **Path** | `/api/v1/users/{id}` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Required |
| **Authorization** | Admin required |

#### 6.4.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | User UUID identifier |

#### 6.4.3. Request Example

```http
DELETE /api/v1/users/user_123e4567-e89b-12d3-a456-426614174000 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### 6.4.4. Response Definition

**Success Response (204 No Content):**

Empty response body with status code 204.

#### 6.4.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid user ID format |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions (admin required) |
| 404 | `RESOURCE_NOT_FOUND` | User not found |
| 409 | `RESOURCE_CONFLICT` | Cannot delete last admin user |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 6.4.6. Related Requirements

- [REQ-SRV-081](../../.adrs/ - RBAC Enforcement

---

### 6.5. Get User Activity

Retrieves activity history for a specific user.

#### 6.5.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/users/{id}/activity` |
| **Idempotent** | Yes |
| **Cachable** | Yes |
| **Authentication** | Required |

#### 6.5.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | User UUID identifier |

#### 6.5.3. Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 50 | Maximum items to return (1-100) |
| `cursor` | string | No | - | Pagination cursor |
| `type` | string | No | - | Filter by activity type (`document`, `repository`, `auth`, `all`) |

#### 6.5.4. Request Example

```http
GET /api/v1/users/user_123e4567-e89b-12d3-a456-426614174000/activity?limit=20&type=document HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

#### 6.5.5. Response Definition

**Success Response (200 OK):**

```json
{
  "data": [
    {
      "id": "act_550e8400-e29b-41d4-a716-446655440001",
      "type": "document",
      "action": "created",
      "description": "Created document 'Introduction to Tachyon'",
      "document_id": "doc_550e8400-e29b-41d4-a716-446655440000",
      "document_title": "Introduction to Tachyon",
      "timestamp": "2026-02-05T16:35:00.000Z",
      "ip_address": "192.168.1.100"
    }
  ],
  "meta": {
    "pagination": {
      "next_cursor": null,
      "has_next": false,
      "limit": 20,
      "count": 1
    }
  }
}
```

#### 6.5.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid query parameters or user ID |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 403 | `AUTHORIZATION_FAILED` | Insufficient permissions to view activity |
| 404 | `RESOURCE_NOT_FOUND` | User not found |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 6.5.7. Related Requirements

- [REQ-SRV-085](../../.adrs/ - Access Logging

---

## 7. AUTHENTICATION ENDPOINTS

### 7.1. Login

Authenticates a user and returns an access token.

#### 7.1.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/auth/login` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Not required |

#### 7.1.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `username` | string | Yes | Username or email address |
| `password` | string | Yes | User password |
| `mfa_code` | string | No | MFA code (if MFA is enabled) |
| `remember_me` | boolean | No | Extend session duration (default: `false`) |

#### 7.1.3. Request Example

```http
POST /api/v1/auth/login HTTP/2
Host: api.tachyon.example.com
Content-Type: application/json

{
  "username": "john.doe",
  "password": "securepassword123",
  "remember_me": false
}
```

#### 7.1.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    "token_type": "Bearer",
    "expires_in": 3600,
    "refresh_expires_in": 2592000,
    "user": {
      "id": "user_123e4567-e89b-12d3-a456-426614174000",
      "username": "john.doe",
      "email": "john.doe@example.com",
      "display_name": "John Doe",
      "role": "editor",
      "mfa_enabled": true,
      "mfa_required": false
    }
  }
}
```

**MFA Required Response (200 OK):**

```json
{
  "data": {
    "mfa_required": true,
    "mfa_type": "totp",
    "mfa_methods": ["totp", "sms", "email"],
    "session_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

#### 7.1.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Invalid credentials |
| 403 | `AUTHORIZATION_FAILED` | Account suspended or locked |
| 422 | `UNPROCESSABLE_ENTITY` | Invalid MFA code |
| 429 | `RATE_LIMIT_EXCEEDED` | Too many login attempts |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.1.6. Related Requirements

- [REQ-SRV-036](../../.adrs/ - Login Endpoint
- [REQ-SRV-076](../../.adrs/ - Session Management
- [REQ-SRV-077](../../.adrs/ - MFA Support

---

### 7.2. Logout

Invalidates the current session and refresh token.

#### 7.2.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/auth/logout` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Required |

#### 7.2.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `refresh_token` | string | Yes | Refresh token to invalidate |
| `all_sessions` | boolean | No | Invalidate all user sessions (default: `false`) |

#### 7.2.3. Request Example

```http
POST /api/v1/auth/logout HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "all_sessions": false
}
```

#### 7.2.4. Response Definition

**Success Response (204 No Content):**

Empty response body with status code 204.

#### 7.2.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.2.6. Related Requirements

- [REQ-SRV-037](../../.adrs/ - Logout Endpoint
- [REQ-SRV-089](../../.adrs/ - Session Revocation

---

### 7.3. Refresh Token

Refreshes an access token using a refresh token.

#### 7.3.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/auth/refresh` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Not required |

#### 7.3.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `refresh_token` | string | Yes | Valid refresh token |

#### 7.3.3. Request Example

```http
POST /api/v1/auth/refresh HTTP/2
Host: api.tachyon.example.com
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### 7.3.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    "token_type": "Bearer",
    "expires_in": 3600,
    "refresh_expires_in": 2592000
  }
}
```

#### 7.3.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Invalid or expired refresh token |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.3.6. Related Requirements

- [REQ-SRV-038](../../.adrs/ - Token Refresh Endpoint
- [REQ-SRV-087](../../.adrs/ - Session Refresh

---

### 7.4. MFA Setup

Initiates multi-factor authentication setup for a user.

#### 7.4.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/auth/mfa/setup` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 7.4.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `mfa_type` | string | Yes | MFA type (`totp`, `sms`, `email`) |

#### 7.4.3. Request Example

```http
POST /api/v1/auth/mfa/setup HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "mfa_type": "totp"
}
```

#### 7.4.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "mfa_type": "totp",
    "secret": "JBSWY3DPEHPK3PXP",
    "qr_code_url": "otpauth://totp/Tachyon:john.doe@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Tachyon",
    "backup_codes": [
      "1234 5678 9012",
      "3456 7890 1234",
      "5678 9012 3456",
      "7890 1234 5678",
      "9012 3456 7890"
    ],
    "setup_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

#### 7.4.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 409 | `RESOURCE_CONFLICT` | MFA already enabled |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.4.6. Related Requirements

- [REQ-SRV-039](../../.adrs/ - MFA Setup Endpoint

---

### 7.5. MFA Verify

Verifies and completes MFA setup.

#### 7.5.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/auth/mfa/verify` |
| **Idempotent** | No |
| **Cachable** | No |
| **Authentication** | Required |

#### 7.5.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `setup_token` | string | Yes | MFA setup token from setup endpoint |
| `code` | string | Yes | MFA verification code |

#### 7.5.3. Request Example

```http
POST /api/v1/auth/mfa/verify HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "setup_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "code": "123456"
}
```

#### 7.5.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "mfa_enabled": true,
    "mfa_type": "totp",
    "verified_at": "2026-02-05T16:50:00.000Z"
  }
}
```

#### 7.5.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Authentication required |
| 422 | `UNPROCESSABLE_ENTITY` | Invalid MFA code |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.5.6. Related Requirements

- [REQ-SRV-040](../../.adrs/ - MFA Verification Endpoint

---

### 7.6. MFA Disable

Disables multi-factor authentication for a user.

#### 7.6.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/v1/auth/mfa/disable` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Required |

#### 7.6.2. Request Body

**Content-Type:** `application/json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `password` | string | Yes | User password for verification |

#### 7.6.3. Request Example

```http
POST /api/v1/auth/mfa/disable HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "password": "securepassword123"
}
```

#### 7.6.4. Response Definition

**Success Response (200 OK):**

```json
{
  "data": {
    "mfa_enabled": false,
    "disabled_at": "2026-02-05T16:55:00.000Z"
  }
}
```

#### 7.6.5. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body |
| 401 | `AUTHENTICATION_FAILED` | Authentication required or invalid password |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.6.6. Related Requirements

- [REQ-SRV-077](../../.adrs/ - MFA Support

---

### 7.7. OAuth 2.0 Authorization

Initiates OAuth 2.0 authorization flow with external providers.

#### 7.7.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/auth/oauth/{provider}` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Not required |

#### 7.7.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider` | string | Yes | OAuth provider (`google`, `github`, `microsoft`, `saml`) |

#### 7.7.3. Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `redirect_uri` | string | Yes | - | Redirect URI after authorization |
| `state` | string | No | - | CSRF protection token |
| `scope` | string | No | - | OAuth scope |

#### 7.7.4. Request Example

```http
GET /api/v1/auth/oauth/google?redirect_uri=https://example.com/callback&state=abc123 HTTP/2
Host: api.tachyon.example.com
```

#### 7.7.5. Response Definition

**Success Response (302 Found):**

Redirects to the OAuth provider's authorization endpoint.

#### 7.7.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid provider or parameters |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.7.7. Related Requirements

- [REQ-SRV-078](../../.adrs/ - OAuth 2.0 Integration
- [REQ-SRV-079](../../.adrs/ - SAML Integration
- [REQ-SRV-080](../../.adrs/ - OpenID Connect

---

### 7.8. OAuth 2.0 Callback

Handles OAuth 2.0 callback from external providers.

#### 7.8.1. Endpoint Definition

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/v1/auth/oauth/{provider}/callback` |
| **Idempotent** | Yes |
| **Cachable** | No |
| **Authentication** | Not required |

#### 7.8.2. Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider` | string | Yes | OAuth provider (`google`, `github`, `microsoft`, `saml`) |

#### 7.8.3. Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `code` | string | Yes | - | OAuth authorization code |
| `state` | string | No | - | CSRF protection token |

#### 7.8.4. Request Example

```http
GET /api/v1/auth/oauth/google/callback?code=4/0AX4XfWh...&state=abc123 HTTP/2
Host: api.tachyon.example.com
```

#### 7.8.5. Response Definition

**Success Response (302 Found):**

Redirects to the application with access token in URL fragment or query parameter.

#### 7.8.6. Error Responses

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid provider or parameters |
| 401 | `AUTHENTICATION_FAILED` | OAuth authorization failed |
| 500 | `INTERNAL_SERVER_ERROR` | Unexpected server error |

#### 7.8.7. Related Requirements

- [REQ-SRV-078](../../.adrs/ - OAuth 2.0 Integration

---

## 8. ENDPOINT SECURITY

### 8.1. Authentication Requirements

#### 8.1.1. Authentication Mechanisms

The Tachyon Server API supports multiple authentication mechanisms:

| Mechanism | Description | Use Case |
|------------|-------------|-----------|
| **Bearer Token (JWT)** | JSON Web Token in Authorization header | Standard API authentication |
| **Session Cookie** | HttpOnly, Secure, SameSite cookie | Web application sessions |
| **OAuth 2.0** | External provider authentication | SSO integration |
| **SAML 2.0** | Enterprise SSO integration | Enterprise deployments |
| **API Key** | Service-to-service authentication | Automated integrations |

#### 8.1.2. JWT Token Structure

Access tokens follow the JWT (JSON Web Token) standard with the following claims:

| Claim | Type | Description |
|-------|------|-------------|
| `sub` | string | Subject (user ID) |
| `name` | string | User display name |
| `email` | string | User email address |
| `role` | string | User role (`admin`, `editor`, `viewer`) |
| `iat` | integer | Issued at (Unix timestamp) |
| `exp` | integer | Expiration time (Unix timestamp) |
| `jti` | string | JWT ID (unique token identifier) |

**Example JWT Header:**

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c
```

#### 8.1.3. Token Validation Rules

All authenticated endpoints enforce the following validation rules:

| Rule | Description |
|-------|-------------|
| **Signature Verification** | JWT signature must be verified using server secret |
| **Expiration Check** | Tokens must not be expired |
| **Revocation Check** | Tokens must not be revoked |
| **Algorithm Restriction** | Only RS256 or ES256 algorithms allowed |
| **Issuer Validation** | Token issuer must match configured issuer |

#### 8.1.4. Session Management

Session cookies adhere to the following security requirements:

| Attribute | Value | Rationale |
|-----------|-------|------------|
| `HttpOnly` | `true` | Prevents JavaScript access (XSS protection) |
| `Secure` | `true` | Only transmitted over HTTPS |
| `SameSite` | `Strict` | Prevents CSRF attacks |
| `Path` | `/` | Available to entire application |
| `Max-Age` | Configurable | Session timeout (default: 1 hour) |

#### 8.1.5. Related Requirements

- [REQ-SRV-076](../../.adrs/ - Session Management
- [REQ-SRV-090](../../.adrs/ - Secure Cookies

---

### 8.2. Authorization Requirements

#### 8.2.1. Role-Based Access Control (RBAC)

The API enforces RBAC with the following role hierarchy:

```
admin > editor > viewer
```

| Role | Permissions |
|------|-------------|
| **admin** | Full access to all resources, user management, system configuration |
| **editor** | Create, read, update documents and repositories |
| **viewer** | Read-only access to authorized documents and repositories |

#### 8.2.2. Resource-Level Authorization

Authorization is enforced at the resource level with the following checks:

| Resource Type | Authorization Check |
|---------------|---------------------|
| **Document** | User must have read access based on document frontmatter (`access` field, `roles` list) |
| **Repository** | User must be repository owner or have explicit access |
| **User Profile** | Users can access their own profile; admins can access any profile |
| **System Configuration** | Admin role required |

#### 8.2.3. Frontmatter Access Control

Document access is controlled via YAML frontmatter:

```yaml
---
title: "Internal Documentation"
access: "private"  # public, private, restricted
roles: ["admin", "editor"]  # Required roles for access
---
```

| Access Level | Description |
|--------------|-------------|
| `public` | Accessible to all authenticated users |
| `private` | Accessible only to document author |
| `restricted` | Accessible only to users with specified roles |

#### 8.2.4. Internal Block Redaction

Internal blocks are redacted from unauthorized users:

```markdown
::: internal
This content is only visible to authorized users.
:::
```

Redaction rules:
- Users without `admin` or specified roles see `[REDACTED]` placeholder
- Content is removed from search index for unauthorized users
- Audit log records access attempts to internal content

#### 8.2.5. Related Requirements

- [REQ-SRV-081](../../.adrs/ - RBAC Enforcement
- [REQ-SRV-082](../../.adrs/ - Frontmatter Access Control
- [REQ-SRV-083](../../.adrs/ - Block Redaction
- [REQ-SRV-084](../../.adrs/ - Principle of Least Privilege

---

### 8.3. Rate Limiting

#### 8.3.1. Rate Limiting Strategy

The API implements token bucket rate limiting with the following tiers:

| Tier | Requests | Window | Applicability |
|-------|-----------|---------|----------------|
| **Anonymous** | 10 requests | 1 minute | Unauthenticated endpoints |
| **Authenticated** | 100 requests | 1 minute | Standard authenticated users |
| **Premium** | 1000 requests | 1 minute | Premium tier users |
| **Admin** | Unlimited | - | Administrative users |

#### 8.3.2. Rate Limit Headers

Rate limit information is communicated via response headers:

| Header | Description | Example |
|---------|-------------|---------|
| `X-RateLimit-Limit` | Request limit for current window | `100` |
| `X-RateLimit-Remaining` | Remaining requests in window | `95` |
| `X-RateLimit-Reset` | Unix timestamp when window resets | `1643723400` |
| `Retry-After` | Seconds until retry allowed (when limited) | `30` |

#### 8.3.3. Rate Limit Response

When rate limit is exceeded:

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Please retry later.",
    "details": {
      "limit": 100,
      "window": "1 minute",
      "retry_after": 30
    },
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-05T16:33:32.070Z"
  }
}
```

#### 8.3.4. Related Requirements

- Threat Model - Rate Limiting Considerations

---

### 8.4. DDoS Protection

#### 8.4.1. Protection Mechanisms

The server implements multiple layers of DDoS protection:

| Mechanism | Description | Threshold |
|-----------|-------------|------------|
| **Connection Limiting** | Maximum concurrent connections per IP | 100 connections |
| **Request Rate Limiting** | Requests per second per IP | 10 requests/second |
| **Challenge-Response** | CAPTCHA for suspicious traffic | Dynamic |
| **IP Blacklisting** | Temporary blocking of abusive IPs | Automatic |
| **Geoblocking** | Optional blocking by geographic region | Configurable |

#### 8.4.2. Challenge-Response Flow

When suspicious activity is detected:

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant ChallengeService
    
    Client->>Server: Request
    Server->>ChallengeService: Check Suspicion Score
    ChallengeService-->>Server: High Risk Detected
    Server-->>Client: 429 + CAPTCHA Challenge
    Client->>Server: Request + CAPTCHA Token
    Server->>ChallengeService: Verify CAPTCHA
    ChallengeService-->>Server: Valid
    Server-->>Client: 200 OK
```

#### 8.4.3. IP Blacklisting

Blacklisting rules:

| Condition | Blacklist Duration |
|-----------|-------------------|
| Repeated rate limit violations | 1 hour |
| Malicious payload attempts | 24 hours |
| Known botnet IPs | Permanent (manual review) |

#### 8.4.4. Related Requirements

- Threat Model - DDoS Protection Considerations

---

### 8.5. Input Validation

#### 8.5.1. Validation Strategy

All endpoint inputs undergo multi-layer validation:

| Layer | Validation Type | Description |
|-------|-----------------|-------------|
| **Type Validation** | Schema validation | Verify data types and structure |
| **Length Validation** | Size constraints | Enforce maximum lengths |
| **Format Validation** | Pattern matching | Email, URL, UUID formats |
| **Semantic Validation** | Business rules | Check logical constraints |
| **Sanitization** | Content cleaning | Remove malicious content |

#### 8.5.2. Common Validation Rules

| Field Type | Validation Rule | Example |
|-------------|-----------------|---------|
| **UUID** | RFC 4122 format | `550e8400-e29b-41d4-a716-446655440000` |
| **Email** | RFC 5322 format | `user@example.com` |
| **URL** | RFC 3986 format | `https://example.com` |
| **Slug** | Alphanumeric + hyphens | `my-document-title` |
| **Markdown** | Sanitized HTML output | Prevent XSS |

#### 8.5.3. SQL Injection Prevention

All database queries use parameterized statements:

```rust
// Correct: Parameterized query
let user = sqlx::query_as!(
    pool,
    "SELECT * FROM users WHERE id = $1",
    user_id
)
.fetch_one()
.await?;

// Incorrect: String concatenation (FORBIDDEN)
let query = format!("SELECT * FROM users WHERE id = '{}'", user_id);
```

#### 8.5.4. XSS Prevention

All user-generated content is sanitized:

| Content Type | Sanitization Method |
|---------------|---------------------|
| **Markdown** | HTML sanitization with allowlist |
| **HTML** | DOMPurify or equivalent |
| **JSON** | Escaped in HTML context |
| **URLs** | URL validation and encoding |

#### 8.5.5. Related Requirements

- [REQ-SRV-044](../../.adrs/ - Content Sanitization
- Threat Model - Input Validation Considerations

---

### 8.6. Transport Security

#### 8.6.1. TLS Configuration

All endpoints require TLS 1.3 encryption:

| Setting | Value | Rationale |
|---------|-------|------------|
| **Minimum TLS Version** | 1.3 | Latest security standards |
| **Cipher Suites** | Secure only | TLS_AES_256_GCM_SHA384, etc. |
| **Certificate** | Let's Encrypt or manual | Automated renewal preferred |
| **HSTS** | Enabled with max-age=31536000 | Enforce HTTPS |
| **OCSP Stapling** | Enabled | Improved certificate validation |

#### 8.6.2. HSTS Header

```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
```

#### 8.6.3. Related Requirements

- [REQ-SRV-016](../../.adrs/ - TLS 1.3 Support
- [REQ-SRV-017](../../.adrs/ - Certificate Management
- [REQ-SRV-018](../../.adrs/ - HSTS Headers

---

## 9. ENDPOINT PERFORMANCE

### 9.1. Latency Requirements

#### 9.1.1. Response Time Targets

All endpoints must meet the following latency requirements:

| Endpoint Category | P50 Latency | P95 Latency | P99 Latency |
|-----------------|-------------|-------------|-------------|
| **Health Check** | < 5 ms | < 10 ms | < 20 ms |
| **Authentication** | < 50 ms | < 100 ms | < 200 ms |
| **Document List** | < 100 ms | < 200 ms | < 500 ms |
| **Document Retrieve** | < 50 ms | < 100 ms | < 200 ms |
| **Document Create** | < 200 ms | < 500 ms | < 1000 ms |
| **Document Update** | < 200 ms | < 500 ms | < 1000 ms |
| **Search** | < 100 ms | < 200 ms | < 500 ms |
| **Repository Sync** | < 1000 ms | < 2000 ms | < 5000 ms |

#### 9.1.2. Performance Monitoring

All endpoints include performance metrics in response headers:

| Header | Description | Example |
|---------|-------------|---------|
| `X-Response-Time` | Request processing time in milliseconds | `45` |
| `X-DB-Query-Time` | Database query time in milliseconds | `12` |
| `X-Cache-Hit` | Cache hit indicator | `true` |

#### 9.1.3. Related Requirements

- [REQ-SRV-041](../../.adrs/ - JIT Rendering
- [REQ-SRV-042](../../.adrs/ - Cache Management

---

### 9.2. Throughput Requirements

#### 9.2.1. Concurrency Targets

The server must support the following concurrent connection levels:

| Metric | Target | Notes |
|---------|---------|--------|
| **Concurrent Connections** | 10,000 | HTTP/2 multiplexing |
| **Requests per Second** | 5,000 | Average request mix |
| **WebSocket Connections** | 1,000 | Real-time features |
| **Concurrent Auth Sessions** | 2,000 | Active user sessions |

#### 9.2.2. Resource Limits

Per-request resource limits:

| Resource | Limit | Rationale |
|-----------|-------|------------|
| **Request Body** | 10 MB | Prevent abuse |
| **Response Body** | 50 MB | Prevent memory exhaustion |
| **Query String** | 2 KB | URL length limits |
| **Headers** | 8 KB | Header size limits |

#### 9.2.3. Related Requirements

- [REQ-SRV-011](../../.adrs/ - HTTP/2 Protocol
- [REQ-SRV-013](../../.adrs/ - Multiplexing

---

### 9.3. Caching Strategy

#### 9.3.1. Cache Hierarchy

The server implements a multi-level caching strategy:

```mermaid
graph TB
    Client[Client Request]
    L1[L1: In-Memory LRU Cache]
    L2[L2: Redis Cache]
    L3[L3: CDN / Browser Cache]
    Data[Data Source]
    
    Client -->|Check| L3
    L3 -->|Miss| L2
    L2 -->|Miss| L1
    L1 -->|Miss| Data
    Data -->|Response| L1
    L1 -->|Populate| L2
    L2 -->|Populate| L3
    L3 -->|Response| Client
    
    style L1 fill:#e1ffe1
    style L2 fill:#ffe1e1
    style L3 fill:#e1e1ff
```

#### 9.3.2. Cache Configuration

| Cache Level | TTL | Size | Content |
|-------------|-----|------|---------|
| **L1: In-Memory** | 5 minutes | 1 GB | Frequently accessed documents |
| **L2: Redis** | 1 hour | 10 GB | Search results, user sessions |
| **L3: CDN/Browser** | 24 hours | - | Static assets, rendered HTML |

#### 9.3.3. Cache Invalidation

Cache invalidation triggers:

| Trigger | Scope | Method |
|---------|-------|--------|
| **Document Update** | Specific document | Tag-based invalidation |
| **Repository Sync** | Entire repository | Namespace invalidation |
| **User Role Change** | User-specific cache | Key-based invalidation |
| **Search Index Update** | Search results | Pattern-based invalidation |

#### 9.3.4. Cache Headers

Cache control headers for client caching:

| Endpoint | Cache-Control | ETag |
|-----------|---------------|-------|
| **Document Retrieve** | `public, max-age=300` | Yes |
| **Static Assets** | `public, max-age=86400, immutable` | Yes |
| **Search Results** | `private, max-age=60` | No |
| **User Profile** | `private, max-age=0` | No |

#### 9.3.5. Related Requirements

- [REQ-SRV-042](../../.adrs/ - Cache Management
- [REQ-SRV-066](../../.adrs/ - Asset Serving
- [REQ-SRV-067](../../.adrs/ - Asset Versioning

---

### 9.4. Optimization Strategies

#### 9.4.1. Database Optimization

| Strategy | Description | Impact |
|-----------|-------------|---------|
| **Connection Pooling** | Reuse database connections | Reduces connection overhead |
| **Query Optimization** | Indexed queries, prepared statements | Reduces query time |
| **Read Replicas** | Separate read/write databases | Improves read throughput |
| **Batch Operations** | Combine multiple operations | Reduces round trips |

#### 9.4.2. Search Optimization

| Strategy | Description | Impact |
|-----------|-------------|---------|
| **Incremental Indexing** | Update index on document changes | Reduces re-indexing time |
| **Index Sharding** | Partition index by repository | Improves query parallelism |
| **Query Caching** | Cache common search queries | Reduces search latency |
| **Index Optimization** | Periodic index compaction | Improves query performance |

#### 9.4.3. Content Optimization

| Strategy | Description | Impact |
|-----------|-------------|---------|
| **JIT Rendering** | Render on-demand with caching | Reduces storage, enables dynamic content |
| **Asset Compression** | Gzip/Brotli compression | Reduces transfer size |
| **Image Optimization** | Resize, compress, format conversion | Reduces bandwidth |
| **CDN Distribution** | Edge caching for static assets | Reduces latency |

#### 9.4.4. Related Requirements

- [REQ-SRV-041](../../.adrs/ - JIT Rendering
- [REQ-SRV-045](../../.adrs/ - Image Processing
- [REQ-SRV-057](../../.adrs/ - Incremental Indexing
- [REQ-SRV-059](../../.adrs/ - Index Optimization
- [REQ-SRV-068](../../.adrs/ - Asset Compression

---

## 10. REFERENCES

### 10.1. Standards and Specifications

| Standard | Reference | Description |
|----------|-----------|-------------|
| **ISO/IEC 26514:2021** | Systems and Software Engineering - Requirements for designers and developers of user documentation |
| **IEEE 830-1998** | IEEE Recommended Practice for Software Requirements Specifications |
| **RFC 7540** | Hypertext Transfer Protocol Version 2 (HTTP/2) |
| **RFC 7231** | Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content |
| **RFC 7234** | Hypertext Transfer Protocol (HTTP/1.1): Conditional Requests |
| **RFC 7235** | Hypertext Transfer Protocol (HTTP/1.1): Range Requests |
| **RFC 7519** | JSON Web Token (JWT) |
| **RFC 6749** | OAuth 2.0 Authorization Framework |
| **RFC 7518** | The OAuth 2.0 Authorization Framework: Bearer Token Usage |
| **RFC 7616** | JSON Web Algorithms (JWA) |
| **RFC 7617** | JSON Web Key (JWK) |
| **RFC 7618** | JSON Web Signature (JWS) |

### 10.2. Architectural Decision Records

| ADR | Title | Reference |
|-----|-------|-----------|
| **ADR-003** | Axum for HTTP/2 Server | [`.adrs/adr-003-lru-cache-target.md](../../.adrs/adr-003-lru-cache-target.md) |
| **ADR-007** | Tokio for Async Runtime | [`.adrs/adr-007-thread-safety-strategy.md](../../.adrs/adr-007-thread-safety-strategy.md) |

### 10.3. Requirements Specifications

| Requirement ID | Title | Reference |
|----------------|-------|-----------|
| **REQ-SRV-001** | Server Startup | [`.adrs/ |
| **REQ-SRV-002** | Graceful Shutdown | [`.adrs/ |
| **REQ-SRV-003** | Configuration Loading | [`.adrs/ |
| **REQ-SRV-005** | Health Check | [`.adrs/ |
| **REQ-SRV-011** | HTTP/2 Protocol | [`.adrs/ |
| **REQ-SRV-016** | TLS 1.3 Support | [`.adrs/ |
| **REQ-SRV-021** | Document List | [`.adrs/ |
| **REQ-SRV-022** | Document Retrieval | [`.adrs/ |
| **REQ-SRV-023** | Document Creation | [`.adrs/ |
| **REQ-SRV-024** | Document Update | [`.adrs/ |
| **REQ-SRV-025** | Document Deletion | [`.adrs/ |
| **REQ-SRV-026** | Full-Text Search | [`.adrs/ |
| **REQ-SRV-027** | Faceted Search | [`.adrs/ |
| **REQ-SRV-028** | Search Autocomplete | [`.adrs/ |
| **REQ-SRV-029** | Search Pagination | [`.adrs/ |
| **REQ-SRV-030** | Search Highlighting | [`.adrs/ |
| **REQ-SRV-031** | Repository Status | [`.adrs/ |
| **REQ-SRV-032** | Commit History | [`.adrs/ |
| **REQ-SRV-033** | Branch List | [`.adrs/ |
| **REQ-SRV-034** | Branch Switch | [`.adrs/ |
| **REQ-SRV-035** | Diff View | [`.adrs/ |
| **REQ-SRV-036** | Login | [`.adrs/ |
| **REQ-SRV-037** | Logout | [`.adrs/ |
| **REQ-SRV-038** | Token Refresh | [`.adrs/ |
| **REQ-SRV-039** | MFA Setup | [`.adrs/ |
| **REQ-SRV-040** | MFA Verification | [`.adrs/ |
| **REQ-SRV-041** | JIT Rendering | [`.adrs/ |
| **REQ-SRV-042** | Cache Management | [`.adrs/ |
| **REQ-SRV-043** | Frontmatter Processing | [`.adrs/ |
| **REQ-SRV-044** | Content Sanitization | [`.adrs/ |
| **REQ-SRV-045** | Image Processing | [`.adrs/ |
| **REQ-SRV-046** | Repository Access | [`.adrs/ |
| **REQ-SRV-047** | Commit Management | [`.adrs/ |
| **REQ-SRV-048** | Branch Operations | [`.adrs/ |
| **REQ-SRV-049** | Merge Operations | [`.adrs/ |
| **REQ-SRV-050** | Repository Sync | [`.adrs/ |
| **REQ-SRV-051** | SQLite Integration | [`.adrs/ |
| **REQ-SRV-052** | Connection Pooling | [`.adrs/ |
| **REQ-SRV-053** | Transaction Management | [`.adrs/ |
| **REQ-SRV-054** | Database Migration | [`.adrs/ |
| **REQ-SRV-055** | Backup Support | [`.adrs/ |
| **REQ-SRV-056** | Tantivy Integration | [`.adrs/ |
| **REQ-SRV-057** | Incremental Indexing | [`.adrs/ |
| **REQ-SRV-058** | Index Synchronization | [`.adrs/ |
| **REQ-SRV-059** | Index Optimization | [`.adrs/ |
| **REQ-SRV-060** | Index Statistics | [`.adrs/ |
| **REQ-SRV-061** | Loopback Binding | [`.adrs/ |
| **REQ-SRV-062** | Random Port Allocation | [`.adrs/ |
| **REQ-SRV-063** | Desktop Authentication | [`.adrs/ |
| **REQ-SRV-064** | Desktop RBAC Bypass | [`.adrs/ |
| **REQ-SRV-065** | Desktop WebSocket | [`.adrs/ |
| **REQ-SRV-066** | Asset Serving | [`.adrs/ |
| **REQ-SRV-067** | Asset Versioning | [`.adrs/ |
| **REQ-SRV-068** | Asset Compression | [`.adrs/ |
| **REQ-SRV-069** | Asset Integrity | [`.adrs/ |
| **REQ-SRV-070** | Asset Optimization | [`.adrs/ |
| **REQ-SRV-071** | Bundle Serving | [`.adrs/ |
| **REQ-SRV-072** | Bundle Caching | [`.adrs/ |
| **REQ-SRV-073** | Bundle Updates | [`.adrs/ |
| **REQ-SRV-074** | Bundle Integrity | [`.adrs/ |
| **REQ-SRV-075** | Bundle Compression | [`.adrs/ |
| **REQ-SRV-076** | Session Management | [`.adrs/ |
| **REQ-SRV-077** | MFA Support | [`.adrs/ |
| **REQ-SRV-078** | OAuth 2.0 Integration | [`.adrs/ |
| **REQ-SRV-079** | SAML Integration | [`.adrs/ |
| **REQ-SRV-080** | OpenID Connect | [`.adrs/ |
| **REQ-SRV-081** | RBAC Enforcement | [`.adrs/ |
| **REQ-SRV-082** | Frontmatter Access Control | [`.adrs/ |
| **REQ-SRV-083** | Block Redaction | [`.adrs/ |
| **REQ-SRV-084** | Principle of Least Privilege | [`.adrs/ |
| **REQ-SRV-085** | Access Logging | [`.adrs/ |
| **REQ-SRV-086** | Session Timeout | [`.adrs/ |
| **REQ-SRV-087** | Session Refresh | [`.adrs/ |
| **REQ-SRV-088** | Concurrent Session Limit | [`.adrs/ |
| **REQ-SRV-089** | Session Revocation | [`.adrs/ |
| **REQ-SRV-090** | Secure Cookies | [`.adrs/ |
| **REQ-SRV-091** | WebSocket Endpoint | [`.adrs/ |
| **REQ-SRV-092** | Connection Authentication | [`.adrs/ |
| **REQ-SRV-093** | Connection Limiting | [`.adrs/ |
| **REQ-SRV-094** | Heartbeat Mechanism | [`.adrs/ |
| **REQ-SRV-095** | Graceful Disconnection | [`.adrs/ |
| **REQ-SRV-096** | Content Updates | [`.adrs/ |
| **REQ-SRV-097** | User Presence | [`.adrs/ |
| **REQ-SRV-098** | Conflict Notifications | [`.adrs/ |
| **REQ-SRV-099** | Typing Indicators | [`.adrs/ |
| **REQ-SRV-100** | Cursor Position | [`.adrs/ |
| **REQ-SRV-101** | Last-Write-Wins | [`.adrs/ |
| **REQ-SRV-102** | Edit Queue | [`.adrs/ |
| **REQ-SRV-103** | Conflict Resolution UI | [`.adrs/ |
| **REQ-SRV-104** | Edit History | [`.adrs/ |
| **REQ-SRV-105** | Locking Mechanism | [`.adrs/ |

### 10.4. Design Specifications

| Design Element | Title | Reference |
|---------------|-------|-----------|
| **DSN-SRV-001** | Server Architecture | [`.adrs/ |

### 10.5. Coding Standards

| Standard | Reference | Description |
|---------|-----------|-------------|
| **TACHYON-STD-V1.0** | Coding and Documentation Standards | [`.adrs/ |

### 10.6. Threat Model

| Document | Reference | Description |
|---------|-----------|-------------|
| **TACHYON-TMA-V1.0** | Threat Model Analysis | [`.adrs/ |

### 10.7. External References

| Reference | URL | Description |
|-----------|-----|-------------|
| **Axum Documentation** | https://docs.rs/axum/ | HTTP/2 web framework for Rust |
| **Tokio Documentation** | https://tokio.rs/ | Asynchronous runtime for Rust |
| **JWT Specification** | https://tools.ietf.org/html/rfc7519 | JSON Web Token specification |
| **OAuth 2.0 Specification** | https://tools.ietf.org/html/rfc6749 | OAuth 2.0 authorization framework |
| **HTTP/2 Specification** | https://httpwg.org/.adrs/ | HTTP/2 protocol specification |
| **Tantivy Documentation** | https://docs.rs/tantivy/ | Full-text search engine for Rust |
| **git2-rs Documentation** | https://docs.rs/git2/ | Git bindings for Rust |

---

## DOCUMENT CONTROL

### Change History

| Version | Date | Author | Description |
|---------|-------|---------|-------------|
| 1.0 | 2026-02-05 | Kilo Code | Initial document creation |

### Review Status

| Reviewer | Date | Status | Comments |
|----------|-------|--------|----------|
| - | - | Pending | - |

### Approval Status

| Approver | Date | Status |
|----------|-------|--------|
| - | - | Pending |

---

**END OF DOCUMENT**
