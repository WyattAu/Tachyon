# TACHYON: REST API SPECIFICATION

**Document ID:** TACHYON-API-004-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** Technical Specification Document
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [REST API Design Principles](#2-rest-api-design-principles)
3. [API Versioning Strategy](#3-api-versioning-strategy)
4. [REST Endpoints Overview](#4-rest-endpoints-overview)
5. [Document Endpoints](#5-document-endpoints)
6. [Repository Endpoints](#6-repository-endpoints)
7. [Search Endpoints](#7-search-endpoints)
8. [User Endpoints](#8-user-endpoints)
9. [Authentication Endpoints](#9-authentication-endpoints)
10. [API Security](#10-api-security)
11. [API Performance](#11-api-performance)
12. [API Documentation](#12-api-documentation)
13. [References](#13-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document defines the comprehensive REST API specification for the Tachyon server component. The API provides programmatic access to document management, repository operations, search functionality, user management, and authentication services. This specification serves as the authoritative reference for API consumers, including the desktop application, web frontend, and external integrations.

### 1.2. Scope

This specification covers all REST API endpoints exposed by the Tachyon server, including:

- Document CRUD operations (create, read, update, delete)
- Repository management (Git operations, branch management)
- Search functionality (full-text search, autocomplete)
- User management (profile, permissions)
- Authentication and authorization (login, logout, token refresh)

The specification does not cover WebSocket protocols, which are documented separately in [TACHYON-DES-API-V1.0](../.specs/04_future_state/design/api_interfaces.md).

### 1.3. Document Dependencies

This document depends on the following documents:

- [TACHYON-STD-V1.0](../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-DES-API-V1.0](../.specs/04_future_state/design/api_interfaces.md) - API Interfaces Design
- [TACHYON-ADR-003-V1.0](../.specs/02_adrs/003_axum_for_http2_server.md) - Axum for HTTP/2 Server
- [TACHYON-ADR-007-V1.0](../.specs/02_adrs/007_tokio_for_async_runtime.md) - Tokio for Async Runtime
- [TACHYON-THR-V1.0](../.specs/04_threat_model/threat_model.md) - Threat Model

### 1.4. Target Audience

This specification is intended for:

- **Frontend Developers:** Developers implementing the Tachyon web and desktop interfaces
- **Integration Developers:** Developers integrating Tachyon with external systems
- **API Testers:** QA engineers validating API behavior and performance
- **System Architects:** Architects designing systems that interact with Tachyon

### 1.5. Conventions Used in This Document

#### 1.5.1. Endpoint Specification Format

Each endpoint is specified using the following format:

```
**Endpoint:** HTTP_METHOD /api/v1/resource

**Description:** Brief description of endpoint purpose

**Authentication:** Required/Optional/None

**Request Parameters:**
- `param1` (type): Description
- `param2` (type): Description

**Request Body:** (if applicable)
```json
{
  "field1": "value1",
  "field2": "value2"
}
```

**Response Body:**
```json
{
  "data": { ... },
  "success": true,
  "meta": { ... }
}
```

**Status Codes:**
- 200 OK - Success
- 400 Bad Request - Invalid input
- 401 Unauthorized - Authentication required
- 403 Forbidden - Insufficient permissions
- 404 Not Found - Resource not found
- 500 Internal Server Error - Server error
```

#### 1.5.2. Type Notation

The following type notation is used throughout this specification:

| Type | Description | Example |
|-------|-------------|---------|
| `string` | Text string | `"example"` |
| `integer` | Integer number | `42` |
| `number` | Floating-point number | `3.14` |
| `boolean` | Boolean value | `true` |
| `array<T>` | Array of type T | `[1, 2, 3]` |
| `object` | JSON object | `{"key": "value"}` |
| `uuid` | UUID v4 identifier | `"550e8400-e29b-41d4-a716-446655440000"` |
| `datetime` | ISO 8601 datetime | `"2026-02-05T14:00:00Z"` |

#### 1.5.3. Parameter Location Notation

Parameter locations are indicated as follows:

- **Path Parameter:** `/api/v1/documents/:id` - `id` is extracted from URL path
- **Query Parameter:** `/api/v1/documents?limit=20` - `limit` is extracted from query string
- **Header Parameter:** Included in HTTP headers
- **Body Parameter:** Included in request body

---

## 2. REST API DESIGN PRINCIPLES

### 2.1. RESTful Conformance

The Tachyon API adheres to REST architectural style principles as defined in Fielding's dissertation [1]:

1. **Resource-Oriented Design:** All API operations are performed on resources identified by URIs
2. **Uniform Interface:** Consistent use of HTTP methods (GET, POST, PUT, DELETE) for resource manipulation
3. **Stateless Communication:** Each request contains all information necessary to understand and process the request
4. **Cacheability:** Responses are explicitly marked as cacheable or non-cacheable
5. **Layered System:** Client cannot distinguish between proxy, gateway, or server components

### 2.2. HTTP Method Semantics

The API uses HTTP methods according to their semantic meaning:

| Method | Purpose | Idempotent | Safe |
|--------|---------|-------------|-------|
| `GET` | Retrieve resource representation | Yes | Yes |
| `POST` | Create new resource or trigger action | No | No |
| `PUT` | Replace resource representation | Yes | No |
| `PATCH` | Partially update resource | No | No |
| `DELETE` | Delete resource | Yes | No |

**Idempotency:** Multiple identical requests have the same effect as a single request.

**Safety:** Request does not modify server state.

### 2.3. URI Design Principles

API URIs follow these design principles:

1. **Noun-Based Resources:** URIs use nouns to represent resources (e.g., `/documents`, `/users`)
2. **Hierarchical Structure:** Related resources are nested (e.g., `/documents/:id/versions`)
3. **Plural Nouns:** Collection resources use plural form (e.g., `/documents`, not `/document`)
4. **Lowercase with Hyphens:** URI paths use lowercase with hyphens for readability
5. **Trailing Slash Omitted:** URIs do not include trailing slashes

**Examples:**
- [PASS] `/api/v1/documents`
- [PASS] `/api/v1/documents/:id/versions`
- [FAIL] `/api/v1/getDocuments`
- [FAIL] `/api/v1/document/`
- [FAIL] `/API/v1/Documents`

### 2.4. Status Code Usage

The API uses HTTP status codes according to RFC 7231 [2]:

#### 2.4.1. Success Codes (2xx)

| Code | Meaning | Usage |
|------|---------|--------|
| 200 OK | Request succeeded | Standard success response |
| 201 Created | Resource created | Response to POST creating resource |
| 204 No Content | Success, no content | Response to DELETE or PUT with no return body |

#### 2.4.2. Redirection Codes (3xx)

| Code | Meaning | Usage |
|------|---------|--------|
| 301 Moved Permanently | Resource permanently moved | Deprecated endpoint |
| 308 Permanent Redirect | Use different method | HTTP method change required |

#### 2.4.3. Client Error Codes (4xx)

| Code | Meaning | Usage |
|------|---------|--------|
| 400 Bad Request | Invalid request | Malformed request body or parameters |
| 401 Unauthorized | Authentication required | Missing or invalid authentication token |
| 403 Forbidden | Insufficient permissions | Valid authentication but insufficient permissions |
| 404 Not Found | Resource not found | Requested resource does not exist |
| 409 Conflict | Resource conflict | Concurrent modification conflict |
| 422 Unprocessable Entity | Semantic errors | Valid syntax but semantic errors |
| 429 Too Many Requests | Rate limit exceeded | Request rate limit exceeded |

#### 2.4.4. Server Error Codes (5xx)

| Code | Meaning | Usage |
|------|---------|--------|
| 500 Internal Server Error | Server error | Unhandled server error |
| 503 Service Unavailable | Service unavailable | Server temporarily unavailable |

### 2.5. Request/Response Format Standards

#### 2.5.1. Request Format

**Content-Type Header:** All requests with body must include `Content-Type: application/json`.

**Character Encoding:** All requests use UTF-8 character encoding.

**Accept Header:** Clients may specify preferred response format using `Accept` header.

#### 2.5.2. Response Format

**Standard Response Structure:**

All successful responses follow this structure:

```json
{
  "data": { ... },
  "success": true,
  "meta": {
    "request_id": "uuid",
    "timestamp": "2026-02-05T14:00:00Z",
    "version": "1.0"
  }
}
```

**Error Response Structure:**

All error responses follow this structure:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": { ... },
    "request_id": "uuid",
    "timestamp": "2026-02-05T14:00:00Z"
  }
}
```

### 2.6. Pagination

List endpoints support pagination to efficiently handle large result sets.

**Pagination Parameters:**

| Parameter | Type | Default | Max | Description |
|-----------|-------|---------|-----|-------------|
| `offset` | integer | 0 | - | Number of items to skip |
| `limit` | integer | 20 | 100 | Number of items to return |

**Pagination Response:**

```json
{
  "data": [ ... ],
  "meta": {
    "total": 1000,
    "offset": 0,
    "limit": 20,
    "has_more": true
  }
}
```

---

## 3. API VERSIONING STRATEGY

### 3.1. Versioning Approach

The API uses URL path versioning to maintain backward compatibility while enabling evolution.

**Version Format:** `/api/v{major_version}/resource`

**Current Version:** v1

**Example:** `/api/v1/documents`

### 3.2. Version Lifecycle

#### 3.2.1. Version Support Policy

- **Stable Versions:** Maintained for minimum 12 months after deprecation
- **Deprecation Notice:** Announced 3 months in advance of removal
- **Breaking Changes:** Require new major version
- **Non-Breaking Changes:** Added to existing version

#### 3.2.2. Version Deprecation Process

1. **Announcement:** Deprecation announced via API changelog and developer notifications
2. **Grace Period:** 3-month grace period for migration
3. **Warning Headers:** Deprecated endpoints return `Deprecation` header
4. **Removal:** Endpoint removed after grace period

### 3.3. Version Detection

Clients should specify API version using:

1. **URL Path:** `/api/v1/documents` (recommended)
2. **Accept Header:** `Accept: application/vnd.tachyon.v1+json`

**Priority:** URL path takes precedence over Accept header.

### 3.4. Backward Compatibility

The following changes are considered non-breaking:

- Adding new optional request parameters
- Adding new fields to response objects
- Adding new endpoints
- Changing error messages (while maintaining error codes)

The following changes are considered breaking:

- Removing or renaming request parameters
- Removing or renaming response fields
- Changing parameter types
- Changing HTTP method for endpoint
- Removing endpoints

---

## 4. REST ENDPOINTS OVERVIEW

### 4.1. Endpoint Categorization

The Tachyon REST API is organized into the following endpoint categories:

| Category | Base Path | Purpose |
|----------|------------|---------|
| **Document Endpoints** | `/api/v1/documents` | Document CRUD operations |
| **Repository Endpoints** | `/api/v1/git` | Git repository operations |
| **Search Endpoints** | `/api/v1/search` | Full-text search and autocomplete |
| **User Endpoints** | `/api/v1/users` | User profile and management |
| **Authentication Endpoints** | `/api/v1/auth` | Authentication and session management |

### 4.2. URI Structure

All API endpoints follow the URI structure pattern:

```
/api/v{version}/{category}/{resource}[/{resource_id}][/{sub-resource}]
```

**Components:**

- `api`: Fixed API prefix
- `v{version}`: API version (e.g., `v1`)
- `{category}`: Endpoint category (e.g., `documents`, `git`)
- `{resource}`: Resource type (e.g., `documents`, `commits`)
- `{resource_id}`: Resource identifier (e.g., UUID for documents)
- `{sub-resource}`: Nested resource (e.g., `versions`, `branches`)

**Examples:**

| URI | Description |
|-----|-------------|
| `/api/v1/documents` | List all documents |
| `/api/v1/documents/:id` | Get specific document |
| `/api/v1/documents/:id/versions` | Get document versions |
| `/api/v1/git/status` | Get repository status |
| `/api/v1/git/commits` | Get commit history |

### 4.3. HTTP Methods

The API supports the following HTTP methods:

| Method | Usage | Body Required | Idempotent |
|--------|---------|---------------|-------------|
| `GET` | Retrieve resources | No | Yes |
| `POST` | Create resources | Yes | No |
| `PUT` | Replace resources | Yes | Yes |
| `PATCH` | Update resources | Yes | No |
| `DELETE` | Delete resources | No | Yes |

### 4.4. Request/Response Formats

#### 4.4.1. Standard Request Headers

All API requests should include the following headers:

| Header | Required | Description | Example |
|--------|-----------|-------------|---------|
| `Content-Type` | Yes (for body) | Media type of request body | `application/json` |
| `Accept` | No | Preferred response format | `application/json` |
| `Authorization` | Yes (authenticated) | Bearer token | `Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...` |
| `User-Agent` | Recommended | Client identification | `Tachyon-Desktop/1.0.0` |
| `X-Request-ID` | No | Request tracking UUID | `550e8400-e29b-41d4-a716-446655440000` |

#### 4.4.2. Standard Response Headers

All API responses include the following headers:

| Header | Description | Example |
|--------|-------------|---------|
| `Content-Type` | Media type of response body | `application/json` |
| `X-Request-ID` | Request tracking UUID | `550e8400-e29b-41d4-a716-446655440000` |
| `X-Response-Time` | Response processing time (ms) | `15` |
| `X-API-Version` | API version | `1.0` |
| `Cache-Control` | Cache directives | `no-cache, max-age=3600` |
| `RateLimit-Remaining` | Remaining requests in window | `95` |
| `RateLimit-Reset` | Unix timestamp when limit resets | `1738732800` |

### 4.5. Endpoint Summary

The following table provides a complete summary of all REST API endpoints:

| Method | Endpoint | Description | Auth Required | Section |
|--------|-----------|-------------|----------------|---------|
| `GET` | `/api/v1/documents` | List documents | Yes | [Document Endpoints](#5-document-endpoints) |
| `GET` | `/api/v1/documents/:id` | Get document | Yes | [Document Endpoints](#5-document-endpoints) |
| `POST` | `/api/v1/documents` | Create document | Yes | [Document Endpoints](#5-document-endpoints) |
| `PUT` | `/api/v1/documents/:id` | Update document | Yes | [Document Endpoints](#5-document-endpoints) |
| `DELETE` | `/api/v1/documents/:id` | Delete document | Yes | [Document Endpoints](#5-document-endpoints) |
| `GET` | `/api/v1/git/status` | Get repository status | Yes | [Repository Endpoints](#6-repository-endpoints) |
| `GET` | `/api/v1/git/commits` | Get commit history | Yes | [Repository Endpoints](#6-repository-endpoints) |
| `POST` | `/api/v1/git/branches/switch` | Switch branch | Yes | [Repository Endpoints](#6-repository-endpoints) |
| `GET` | `/api/v1/search` | Search documents | Yes | [Search Endpoints](#7-search-endpoints) |
| `GET` | `/api/v1/search/autocomplete` | Autocomplete suggestions | Yes | [Search Endpoints](#7-search-endpoints) |
| `GET` | `/api/v1/users` | List users | Yes | [User Endpoints](#8-user-endpoints) |
| `GET` | `/api/v1/users/:id` | Get user profile | Yes | [User Endpoints](#8-user-endpoints) |
| `PUT` | `/api/v1/users/:id` | Update user profile | Yes | [User Endpoints](#8-user-endpoints) |
| `DELETE` | `/api/v1/users/:id` | Delete user | Yes | [User Endpoints](#8-user-endpoints) |
| `POST` | `/api/v1/auth/login` | Login | No | [Authentication Endpoints](#9-authentication-endpoints) |
| `POST` | `/api/v1/auth/logout` | Logout | Yes | [Authentication Endpoints](#9-authentication-endpoints) |
| `POST` | `/api/v1/auth/refresh` | Refresh token | Yes | [Authentication Endpoints](#9-authentication-endpoints) |

---

## 5. DOCUMENT ENDPOINTS

### 5.1. List Documents

**Endpoint:** `GET /api/v1/documents`

**Description:** Retrieves a paginated list of accessible documents with optional filtering and sorting.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Default | Max | Description |
|-----------|-------|-----------|-----------|-----|-------------|
| `offset` | integer | Query | No | 0 | - | Number of documents to skip |
| `limit` | integer | Query | No | 20 | 100 | Number of documents to return |
| `sort` | string | Query | No | `date` | - | Sort field (`title`, `date`, `size`) |
| `order` | string | Query | No | `desc` | - | Sort order (`asc`, `desc`) |
| `tag` | string | Query | No | - | - | Filter by tag |
| `q` | string | Query | No | - | - | Search query string |

**Constraints:**

- `offset`: Must be non-negative integer
- `limit`: Must be between 1 and 100 inclusive
- `sort`: Must be one of `title`, `date`, `size`
- `order`: Must be `asc` or `desc`
- `tag`: Maximum 64 characters
- `q`: Maximum 1000 characters

**Request Example:**

```http
GET /api/v1/documents?limit=20&offset=0&sort=date&order=desc&tag=documentation
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "System Architecture Overview",
      "path": "docs/architecture/system_architecture.md",
      "created_at": "2026-02-01T10:00:00Z",
      "updated_at": "2026-02-05T14:00:00Z",
      "author": {
        "id": "user-001",
        "username": "johndoe"
      },
      "tags": ["architecture", "overview"],
      "size": 15360,
      "access": {
        "read": ["*"],
        "write": ["admin", "editors"]
      }
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "title": "API Reference",
      "path": "docs/api/api_reference.md",
      "created_at": "2026-02-02T09:30:00Z",
      "updated_at": "2026-02-04T16:45:00Z",
      "author": {
        "id": "user-002",
        "username": "janesmith"
      },
      "tags": ["api", "reference"],
      "size": 28450,
      "access": {
        "read": ["*"],
        "write": ["admin", "editors"]
      }
    }
  ],
  "success": true,
  "meta": {
    "request_id": "770e8400-e29b-41d4-a716-4466554440000",
    "timestamp": "2026-02-05T14:30:00Z",
    "version": "1.0",
    "total": 42,
    "offset": 0,
    "limit": 20,
    "has_more": true
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Documents retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-021: Document List
- REQ-SRV-081: RBAC Enforcement

**Related Design Elements:**

- DES-API-001: List Documents

### 5.2. Get Document

**Endpoint:** `GET /api/v1/documents/:id`

**Description:** Retrieves a specific document by ID with rendered content.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|-------|-----------|-----------|-------------|
| `id` | uuid | Path | Yes | Document UUID v4 identifier |

**Request Example:**

```http
GET /api/v1/documents/550e8400-e29b-41d4-a716-446655440000
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": {
    "metadata": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "System Architecture Overview",
      "path": "docs/architecture/system_architecture.md",
      "created_at": "2026-02-01T10:00:00Z",
      "updated_at": "2026-02-05T14:00:00Z",
      "author": {
        "id": "user-001",
        "username": "johndoe"
      },
      "tags": ["architecture", "overview"],
      "size": 15360,
      "access": {
        "read": ["*"],
        "write": ["admin", "editors"]
      }
    },
    "raw": "# System Architecture Overview\n\nThe Tachyon system...",
    "html": "<h1>System Architecture Overview</h1>\n<p>The Tachyon system...</p>",
    "toc": [
      {
        "level": 1,
        "title": "System Architecture Overview",
        "anchor": "system-architecture-overview"
      },
      {
        "level": 2,
        "title": "Introduction",
        "anchor": "introduction"
      },
      {
        "level": 2,
        "title": "Architecture Principles",
        "anchor": "architecture-principles"
      }
    ],
    "cached": true
  },
  "success": true,
  "meta": {
    "request_id": "770e8400-e29b-41d4-a716-4466554440001",
    "timestamp": "2026-02-05T14:31:00Z",
    "version": "1.0"
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Document retrieved successfully |
| 400 Bad Request | Invalid document ID format |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions to access document |
| 404 Not Found | Document not found |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-022: Document Retrieval
- REQ-SRV-041: JIT Rendering

**Related Design Elements:**

- DES-API-002: Get Document

### 5.3. Create Document

**Endpoint:** `POST /api/v1/documents`

**Description:** Creates a new document with provided content.

**Authentication:** Required

**Request Body:**

```json
{
  "title": "New Document Title",
  "path": "docs/new_document.md",
  "content": "# New Document\n\nThis is a new document...",
  "tags": ["tag1", "tag2"],
  "access": {
    "read": ["*"],
    "write": ["admin", "editors"]
  }
}
```

**Request Body Fields:**

| Field | Type | Required | Constraints | Description |
|-------|-------|-----------|-------------|-------------|
| `title` | string | Yes | 1-255 characters | Document title |
| `path` | string | Yes | 1-1024 characters | Document path (relative to repository root) |
| `content` | string | Yes | Max 100MB | Markdown content |
| `tags` | array<string> | No | Max 50 tags, 64 characters each | Document tags |
| `access` | object | No | - | Access control configuration |

**Access Control Object:**

| Field | Type | Required | Description |
|-------|-------|-----------|-------------|
| `read` | array<string> | Yes | List of roles/groups with read access |
| `write` | array<string> | Yes | List of roles/groups with write access |

**Request Example:**

```http
POST /api/v1/documents
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json

{
  "title": "New Document Title",
  "path": "docs/new_document.md",
  "content": "# New Document\n\nThis is a new document...",
  "tags": ["tag1", "tag2"],
  "access": {
    "read": ["*"],
    "write": ["admin", "editors"]
  }
}
```

**Response Body:**

```json
{
  "data": {
    "metadata": {
      "id": "770e8400-e29b-41d4-a716-4466554440002",
      "title": "New Document Title",
      "path": "docs/new_document.md",
      "created_at": "2026-02-05T14:32:00Z",
      "updated_at": "2026-02-05T14:32:00Z",
      "author": {
        "id": "user-001",
        "username": "johndoe"
      },
      "tags": ["tag1", "tag2"],
      "size": 256,
      "access": {
        "read": ["*"],
        "write": ["admin", "editors"]
      }
    },
    "raw": "# New Document\n\nThis is a new document...",
    "html": "<h1>New Document Title</h1>\n<p>This is a new document...</p>",
    "toc": [
      {
        "level": 1,
        "title": "New Document Title",
        "anchor": "new-document-title"
      }
    ],
    "cached": false
  },
  "success": true,
  "meta": {
    "request_id": "880e8400-e29b-41d4-a716-4466554440000",
    "timestamp": "2026-02-05T14:32:00Z",
    "version": "1.0"
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 201 Created | Document created successfully |
| 400 Bad Request | Invalid request body or parameters |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions to create document |
| 409 Conflict | Document with same path already exists |
| 422 Unprocessable Entity | Path validation failed |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-023: Document Creation
- REQ-SRV-047: Commit Management

**Related Design Elements:**

- DES-API-003: Create Document

### 5.4. Update Document

**Endpoint:** `PUT /api/v1/documents/:id`

**Description:** Updates an existing document with provided content. Only provided fields are modified (partial update).

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|-------|-----------|-----------|-------------|
| `id` | uuid | Path | Yes | Document UUID v4 identifier |

**Request Body:**

```json
{
  "title": "Updated Document Title",
  "content": "# Updated Document\n\nThis is updated content...",
  "tags": ["tag1", "tag3"],
  "access": {
    "read": ["*"],
    "write": ["admin", "editors", "reviewers"]
  }
}
```

**Request Body Fields:**

| Field | Type | Required | Constraints | Description |
|-------|-------|-----------|-------------|-------------|
| `title` | string | No | 1-255 characters | New document title |
| `content` | string | No | Max 100MB | New Markdown content |
| `tags` | array<string> | No | Max 50 tags, 64 characters each | New document tags |
| `access` | object | No | - | New access control configuration |

**Request Example:**

```http
PUT /api/v1/documents/550e8400-e29b-41d4-a716-446655440000
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json

{
  "title": "Updated Document Title",
  "content": "# Updated Document\n\nThis is updated content...",
  "tags": ["tag1", "tag3"],
  "access": {
    "read": ["*"],
    "write": ["admin", "editors", "reviewers"]
  }
}
```

**Response Body:**

```json
{
  "data": {
    "metadata": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "Updated Document Title",
      "path": "docs/architecture/system_architecture.md",
      "created_at": "2026-02-01T10:00:00Z",
      "updated_at": "2026-02-05T14:33:00Z",
      "author": {
        "id": "user-001",
        "username": "johndoe"
      },
      "tags": ["tag1", "tag3"],
      "size": 15360,
      "access": {
        "read": ["*"],
        "write": ["admin", "editors", "reviewers"]
      }
    },
    "raw": "# Updated Document\n\nThis is updated content...",
    "html": "<h1>Updated Document Title</h1>\n<p>This is updated content...</p>",
    "toc": [
      {
        "level": 1,
        "title": "Updated Document Title",
        "anchor": "updated-document-title"
      }
    ],
    "cached": false
  },
  "success": true,
  "meta": {
    "request_id": "990e8400-e29b-41d4-a716-4466554440000",
    "timestamp": "2026-02-05T14:33:00Z",
    "version": "1.0"
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Document updated successfully |
| 400 Bad Request | Invalid request body or parameters |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions to update document |
| 404 Not Found | Document not found |
| 409 Conflict | Concurrent modification conflict |
| 422 Unprocessable Entity | Validation failed |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-024: Document Update
- REQ-SRV-047: Commit Management

**Related Design Elements:**

- DES-API-004: Update Document

### 5.5. Delete Document

**Endpoint:** `DELETE /api/v1/documents/:id`

**Description:** Deletes a document by ID.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|-------|-----------|-----------|-------------|
| `id` | uuid | Path | Yes | Document UUID v4 identifier |

**Request Example:**

```http
DELETE /api/v1/documents/550e8400-e29b-41d4-a716-446655440000
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

Returns `204 No Content` on success with no response body.

**Status Codes:**

| Code | Meaning |
|------|---------|
| 204 No Content | Document deleted successfully |
| 400 Bad Request | Invalid document ID format |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions to delete document |
| 404 Not Found | Document not found |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-025: Document Deletion
- REQ-SRV-047: Commit Management

**Related Design Elements:**

- DES-API-005: Delete Document

---

## 6. REPOSITORY ENDPOINTS

### 6.1. Get Git Status

**Endpoint:** `GET /api/v1/git/status`

**Description:** Retrieves current Git repository status including branch, commit, modified files, and synchronization state.

**Authentication:** Required

**Request Parameters:** None

**Request Example:**

```http
GET /api/v1/git/status
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": {
    "branch": "main",
    "commit": {
      "hash": "a1b2c3d4e5f67890",
      "short_hash": "a1b2c3d",
      "author": "johndoe",
      "email": "john.doe@example.com",
      "message": "Update system architecture documentation",
      "timestamp": "2026-02-05T14:00:00Z"
    },
    "modified": [
      {
        "path": "docs/architecture/system_architecture.md",
        "status": "modified"
      },
      {
        "path": "docs/api/rest_api_specification.md",
        "status": "added"
      }
    ],
    "staged": [
      {
        "path": "docs/architecture/data_flow.md",
        "status": "modified"
      }
    ],
    "untracked": [
      "docs/new_document.md"
    ],
    "ahead": 3,
    "behind": 0
  },
  "success": true,
  "meta": {
    "request_id": "100e8400-e29b-41d4-a716-4466554400003",
    "timestamp": "2026-02-05T14:34:00Z",
    "version": "1.0"
  }
}
```

**Response Fields:**

| Field | Type | Description |
|-------|-------|-------------|
| `branch` | string | Current branch name |
| `commit` | object | Current commit information |
| `commit.hash` | string | Full commit hash |
| `commit.short_hash` | string | Short commit hash (7 characters) |
| `commit.author` | string | Commit author name |
| `commit.email` | string | Commit author email |
| `commit.message` | string | Commit message |
| `commit.timestamp` | datetime | Commit timestamp |
| `modified` | array<object> | Modified files |
| `staged` | array<object> | Staged files |
| `untracked` | array<string> | Untracked files |
| `ahead` | integer | Number of commits ahead of remote |
| `behind` | integer | Number of commits behind remote |

**File Status Object:**

| Field | Type | Description |
|-------|-------|-------------|
| `path` | string | File path relative to repository root |
| `status` | string | File status (`added`, `modified`, `deleted`, `renamed`) |

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Repository status retrieved successfully |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions to access repository |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-031: Repository Status
- REQ-SRV-046: Repository Access

**Related Design Elements:**

- DES-API-008: Get Git Status

### 6.2. Get Commit History

**Endpoint:** `GET /api/v1/git/commits`

**Description:** Retrieves Git commit history with pagination support.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Default | Max | Description |
|-----------|-------|-----------|-----------|-----|-------------|
| `offset` | integer | Query | No | 0 | - | Number of commits to skip |
| `limit` | integer | Query | No | 20 | 100 | Number of commits to return |
| `path` | string | Query | No | - | - | Filter by file path |

**Constraints:**

- `offset`: Must be non-negative integer
- `limit`: Must be between 1 and 100 inclusive
- `path`: Maximum 1024 characters

**Request Example:**

```http
GET /api/v1/git/commits?limit=20&offset=0&path=docs/architecture/
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": [
    {
      "hash": "a1b2c3d4e5f67890",
      "short_hash": "a1b2c3d",
      "author": "johndoe",
      "email": "john.doe@example.com",
      "message": "Update system architecture documentation",
      "timestamp": "2026-02-05T14:00:00Z",
      "files": [
        "docs/architecture/system_architecture.md",
        "docs/architecture/component_architecture.md"
      ]
    },
    {
      "hash": "b2c3d4e5f67890a1",
      "short_hash": "b2c3d4e",
      "author": "janesmith",
      "email": "jane.smith@example.com",
      "message": "Add API documentation",
      "timestamp": "2026-02-04T16:45:00Z",
      "files": [
        "docs/api/rest_api_specification.md"
      ]
    }
  ],
  "success": true,
  "meta": {
    "request_id": "110e8400-e29b-41d4-a716-4466554400004",
    "timestamp": "2026-02-05T14:35:00Z",
    "version": "1.0",
    "total": 42,
    "offset": 0,
    "limit": 20,
    "has_more": true
  }
}
```

**Commit Object Fields:**

| Field | Type | Description |
|-------|-------|-------------|
| `hash` | string | Full commit hash |
| `short_hash` | string | Short commit hash (7 characters) |
| `author` | string | Commit author name |
| `email` | string | Commit author email |
| `message` | string | Commit message |
| `timestamp` | datetime | Commit timestamp |
| `files` | array<string> | Files modified in commit |

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Commit history retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions to access repository |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-032: Commit History
- REQ-SRV-049: History Viewing

**Related Design Elements:**

- DES-API-009: Get Commit History

### 6.3. Switch Branch

**Endpoint:** `POST /api/v1/git/branches/switch`

**Description:** Switches to a different Git branch.

**Authentication:** Required

**Request Body:**

```json
{
  "branch": "feature/new-architecture"
}
```

**Request Body Fields:**

| Field | Type | Required | Constraints | Description |
|-------|-------|-----------|-------------|-------------|
| `branch` | string | Yes | 1-255 characters | Target branch name |

**Request Example:**

```http
POST /api/v1/git/branches/switch
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json

{
  "branch": "feature/new-architecture"
}
```

**Response Body:**

```json
{
  "data": {
    "branch": "feature/new-architecture",
    "commit": "c3d4e5f67890a1b2c"
  },
  "success": true,
  "meta": {
    "request_id": "120e8400-e29b-41d4-a716-4466554400005",
    "timestamp": "2026-02-05T14:36:00Z",
    "version": "1.0"
  }
}
```

**Response Fields:**

| Field | Type | Description |
|-------|-------|-------------|
| `branch` | string | Current branch name after switch |
| `commit` | string | Current commit hash |

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Branch switched successfully |
| 400 Bad Request | Invalid branch name |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions to switch branch |
| 404 Not Found | Branch does not exist |
| 409 Conflict | Uncommitted changes exist |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-034: Branch Switch
- REQ-SRV-048: Branch Operations

**Related Design Elements:**

- DES-API-010: Switch Branch

---

## 7. SEARCH ENDPOINTS

### 7.1. Search Documents

**Endpoint:** `GET /api/v1/search`

**Description:** Performs full-text search with faceted filtering and relevance ranking.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Default | Max | Description |
|-----------|-------|-----------|-----------|-----|-------------|
| `q` | string | Query | Yes | - | 1000 characters | Search query string |
| `filters` | string | Query | No | - | - | Facet filters (JSON-encoded) |
| `sort` | string | Query | No | `relevance` | - | Sort order (`relevance`, `date`, `title`) |
| `offset` | integer | Query | No | 0 | - | Number of results to skip |
| `limit` | integer | Query | No | 20 | 100 | Number of results to return |
| `fuzzy` | boolean | Query | No | `true` | - | Enable fuzzy search |

**Constraints:**

- `q`: Non-empty, maximum 1000 characters
- `limit`: Must be between 1 and 100 inclusive
- `offset`: Must be non-negative integer
- `sort`: Must be one of `relevance`, `date`, `title`
- `fuzzy`: Boolean value
- `filters`: Valid JSON string

**Request Example:**

```http
GET /api/v1/search?q=architecture&sort=relevance&limit=20&offset=0&fuzzy=true
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": [
    {
      "document_id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "System Architecture Overview",
      "path": "docs/architecture/system_architecture.md",
      "snippet": "The Tachyon system architecture follows a three-tier design...",
      "score": 0.95,
      "highlights": [
        "The <mark>Tachyon</mark> system architecture follows a three-tier design..."
      ]
    },
    {
      "document_id": "660e8400-e29b-41d4-a716-446655440001",
      "title": "Component Architecture Documentation",
      "path": "docs/architecture/component_architecture.md",
      "snippet": "Component architecture defines the boundaries between...",
      "score": 0.87,
      "highlights": [
        "Component <mark>architecture</mark> defines the boundaries between..."
      ]
    }
  ],
  "success": true,
  "meta": {
    "request_id": "130e8400-e29b-41d4-a716-4466554400006",
    "timestamp": "2026-02-05T14:37:00Z",
    "version": "1.0",
    "total": 15,
    "offset": 0,
    "limit": 20,
    "has_more": false,
    "query_time_ms": 12
  }
}
```

**Search Result Object Fields:**

| Field | Type | Description |
|-------|-------|-------------|
| `document_id` | uuid | Document UUID |
| `title` | string | Document title |
| `path` | string | Document path |
| `snippet` | string | Search result snippet |
| `score` | number | Relevance score (0.0-1.0) |
| `highlights` | array<string> | Highlighted search terms |

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Search completed successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-026: Full-Text Search
- REQ-SRV-056: Tantivy Integration

**Related Design Elements:**

- DES-API-006: Search Documents

### 7.2. Search Autocomplete

**Endpoint:** `GET /api/v1/search/autocomplete`

**Description:** Provides search suggestions as user types for improved search UX.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Default | Max | Description |
|-----------|-------|-----------|-----------|-----|-------------|
| `q` | string | Query | Yes | - | 100 characters | Partial query string |
| `limit` | integer | Query | No | 10 | 20 | Maximum suggestions to return |

**Constraints:**

- `q`: Non-empty, maximum 100 characters
- `limit`: Must be between 1 and 20 inclusive

**Request Example:**

```http
GET /api/v1/search/autocomplete?q=arch&limit=10
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": {
    "suggestions": [
      {
        "text": "architecture",
        "type": "tag",
        "document_id": null
      },
      {
        "text": "System Architecture Overview",
        "type": "title",
        "document_id": "550e8400-e29b-41d4-a716-446655440000"
      },
      {
        "text": "docs/architecture/system_architecture.md",
        "type": "path",
        "document_id": "550e8400-e29b-41d4-a716-446655440000"
      }
    ]
  },
  "success": true,
  "meta": {
    "request_id": "140e8400-e29b-41d4-a716-4466554400007",
    "timestamp": "2026-02-05T14:38:00Z",
    "version": "1.0"
  }
}
```

**Suggestion Object Fields:**

| Field | Type | Description |
|-------|-------|-------------|
| `text` | string | Suggestion text |
| `type` | string | Suggestion type (`title`, `tag`, `path`) |
| `document_id` | uuid | Document UUID (null for tags) |

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Autocomplete suggestions retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-028: Search Autocomplete

**Related Design Elements:**

- DES-API-007: Search Autocomplete

---

## 8. USER ENDPOINTS

### 8.1. List Users

**Endpoint:** `GET /api/v1/users`

**Description:** Retrieves a paginated list of users.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Default | Max | Description |
|-----------|-------|-----------|-----------|-----|-------------|
| `offset` | integer | Query | No | 0 | - | Number of users to skip |
| `limit` | integer | Query | No | 20 | 100 | Number of users to return |
| `sort` | string | Query | No | `username` | - | Sort field (`username`, `created_at`) |
| `order` | string | Query | No | `asc` | - | Sort order (`asc`, `desc`) |

**Constraints:**

- `offset`: Must be non-negative integer
- `limit`: Must be between 1 and 100 inclusive
- `sort`: Must be one of `username`, `created_at`
- `order`: Must be `asc` or `desc`

**Request Example:**

```http
GET /api/v1/users?limit=20&offset=0&sort=username&order=asc
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": [
    {
      "id": "user-001",
      "username": "johndoe",
      "email": "john.doe@example.com",
      "display_name": "John Doe",
      "role": "admin",
      "created_at": "2026-01-15T10:00:00Z",
      "last_login": "2026-02-05T14:00:00Z",
      "active": true
    },
    {
      "id": "user-002",
      "username": "janesmith",
      "email": "jane.smith@example.com",
      "display_name": "Jane Smith",
      "role": "editor",
      "created_at": "2026-01-20T09:30:00Z",
      "last_login": "2026-02-04T16:45:00Z",
      "active": true
    }
  ],
  "success": true,
  "meta": {
    "request_id": "150e8400-e29b-41d4-a716-4466554400008",
    "timestamp": "2026-02-05T14:39:00Z",
    "version": "1.0",
    "total": 25,
    "offset": 0,
    "limit": 20,
    "has_more": true
  }
}
```

**User Object Fields:**

| Field | Type | Description |
|-------|-------|-------------|
| `id` | string | User ID |
| `username` | string | Username |
| `email` | string | Email address |
| `display_name` | string | Display name |
| `role` | string | User role (`admin`, `editor`, `viewer`) |
| `created_at` | datetime | Account creation timestamp |
| `last_login` | datetime | Last login timestamp |
| `active` | boolean | Account active status |

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Users retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions |
| 500 Internal Server Error | Server error |

### 8.2. Get User

**Endpoint:** `GET /api/v1/users/:id`

**Description:** Retrieves a specific user profile by ID.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|-------|-----------|-----------|-------------|
| `id` | string | Path | Yes | User ID |

**Request Example:**

```http
GET /api/v1/users/user-001
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": {
    "id": "user-001",
    "username": "johndoe",
    "email": "john.doe@example.com",
    "display_name": "John Doe",
    "role": "admin",
    "permissions": [
      "documents:read",
      "documents:write",
      "documents:delete",
      "users:read",
      "users:write",
      "users:delete"
    ],
    "created_at": "2026-01-15T10:00:00Z",
    "last_login": "2026-02-05T14:00:00Z",
    "active": true
  },
  "success": true,
  "meta": {
    "request_id": "160e8400-e29b-41d4-a716-4466554400009",
    "timestamp": "2026-02-05T14:40:00Z",
    "version": "1.0"
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | User retrieved successfully |
| 400 Bad Request | Invalid user ID format |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | User not found |
| 500 Internal Server Error | Server error |

### 8.3. Update User

**Endpoint:** `PUT /api/v1/users/:id`

**Description:** Updates a user profile.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|-------|-----------|-----------|-------------|
| `id` | string | Path | Yes | User ID |

**Request Body:**

```json
{
  "display_name": "John Doe",
  "email": "john.doe@example.com",
  "role": "admin",
  "permissions": [
    "documents:read",
    "documents:write",
    "documents:delete",
    "users:read",
    "users:write",
    "users:delete"
  ]
}
```

**Request Body Fields:**

| Field | Type | Required | Constraints | Description |
|-------|-------|-----------|-------------|-------------|
| `display_name` | string | No | 1-255 characters | Display name |
| `email` | string | No | Valid email format | Email address |
| `role` | string | No | Valid role | User role |
| `permissions` | array<string> | No | - | Permission list |

**Request Example:**

```http
PUT /api/v1/users/user-001
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json

{
  "display_name": "John Doe",
  "email": "john.doe@example.com",
  "role": "admin",
  "permissions": [
    "documents:read",
    "documents:write",
    "documents:delete",
    "users:read",
    "users:write",
    "users:delete"
  ]
}
```

**Response Body:**

```json
{
  "data": {
    "id": "user-001",
    "username": "johndoe",
    "email": "john.doe@example.com",
    "display_name": "John Doe",
    "role": "admin",
    "permissions": [
      "documents:read",
      "documents:write",
      "documents:delete",
      "users:read",
      "users:write",
      "users:delete"
    ],
    "created_at": "2026-01-15T10:00:00Z",
    "last_login": "2026-02-05T14:00:00Z",
    "active": true
  },
  "success": true,
  "meta": {
    "request_id": "170e8400-e29b-41d4-a716-4466554400010",
    "timestamp": "2026-02-05T14:41:00Z",
    "version": "1.0"
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | User updated successfully |
| 400 Bad Request | Invalid request body or parameters |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | User not found |
| 422 Unprocessable Entity | Validation failed |
| 500 Internal Server Error | Server error |

### 8.4. Delete User

**Endpoint:** `DELETE /api/v1/users/:id`

**Description:** Deletes a user account.

**Authentication:** Required

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|-------|-----------|-----------|-------------|
| `id` | string | Path | Yes | User ID |

**Request Example:**

```http
DELETE /api/v1/users/user-001
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

Returns `204 No Content` on success with no response body.

**Status Codes:**

| Code | Meaning |
|------|---------|
| 204 No Content | User deleted successfully |
| 400 Bad Request | Invalid user ID format |
| 401 Unauthorized | Authentication required |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | User not found |
| 409 Conflict | Cannot delete own account |
| 500 Internal Server Error | Server error |

---

## 9. AUTHENTICATION ENDPOINTS

### 9.1. Login

**Endpoint:** `POST /api/v1/auth/login`

**Description:** Authenticates user and returns session token.

**Authentication:** Not required

**Request Body:**

```json
{
  "identifier": "johndoe",
  "password": "securepassword123",
  "mfa_code": "123456"
}
```

**Request Body Fields:**

| Field | Type | Required | Constraints | Description |
|-------|-------|-----------|-------------|-------------|
| `identifier` | string | Yes | 1-255 characters | Username or email |
| `password` | string | Yes | 8-128 characters | Password |
| `mfa_code` | string | No | 6 digits | MFA code (if required) |

**Request Example:**

```http
POST /api/v1/auth/login
Host: api.tachyon.example.com
Content-Type: application/json
Accept: application/json

{
  "identifier": "johndoe",
  "password": "securepassword123",
  "mfa_code": "123456"
}
```

**Response Body:**

```json
{
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_at": "2026-02-06T14:00:00Z",
    "user": {
      "id": "user-001",
      "username": "johndoe",
      "email": "john.doe@example.com",
      "display_name": "John Doe",
      "role": "admin",
      "permissions": [
        "documents:read",
        "documents:write",
        "documents:delete",
        "users:read",
        "users:write",
        "users:delete"
      ]
    },
    "mfa_required": false
  },
  "success": true,
  "meta": {
    "request_id": "180e8400-e29b-41d4-a716-4466554400011",
    "timestamp": "2026-02-05T14:42:00Z",
    "version": "1.0"
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Login successful |
| 400 Bad Request | Invalid request body |
| 401 Unauthorized | Invalid credentials |
| 403 Forbidden | Account disabled or locked |
| 429 Too Many Requests | Rate limit exceeded |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-036: Login
- REQ-SRV-076: Session Management

### 9.2. Logout

**Endpoint:** `POST /api/v1/auth/logout`

**Description:** Invalidates user session.

**Authentication:** Required

**Request Body:** None

**Request Example:**

```http
POST /api/v1/auth/logout
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

Returns `204 No Content` on success with no response body.

**Status Codes:**

| Code | Meaning |
|------|---------|
| 204 No Content | Logout successful |
| 401 Unauthorized | Invalid or expired token |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-037: Logout
- REQ-SRV-089: Session Revocation

### 9.3. Refresh Token

**Endpoint:** `POST /api/v1/auth/refresh`

**Description:** Refreshes session token with rotation.

**Authentication:** Required

**Request Body:** None

**Request Example:**

```http
POST /api/v1/auth/refresh
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Accept: application/json
```

**Response Body:**

```json
{
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_at": "2026-02-06T15:00:00Z"
  },
  "success": true,
  "meta": {
    "request_id": "190e8400-e29b-41d4-a716-4466554400012",
    "timestamp": "2026-02-05T14:43:00Z",
    "version": "1.0"
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 OK | Token refreshed successfully |
| 401 Unauthorized | Invalid or expired token |
| 403 Forbidden | Session expired |
| 500 Internal Server Error | Server error |

**Related Requirements:**

- REQ-SRV-038: Token Refresh
- REQ-SRV-087: Session Refresh

---

## 10. API SECURITY

### 10.1. Authentication Requirements

#### 10.1.1. Bearer Token Authentication

All authenticated endpoints require Bearer token authentication via the `Authorization` header.

**Header Format:**

```
Authorization: Bearer <token>
```

**Token Format:**

- JWT (JSON Web Token) signed with RS256 algorithm
- Includes user ID, role, permissions, and expiration
- Tokens expire after 24 hours by default
- Tokens are rotated on refresh

**Token Structure:**

```json
{
  "sub": "user-001",
  "iat": 1738732800,
  "exp": 1738819200,
  "role": "admin",
  "permissions": [
    "documents:read",
    "documents:write",
    "documents:delete",
    "users:read",
    "users:write",
    "users:delete"
  ]
}
```

#### 10.1.2. Token Validation

Tokens are validated on each authenticated request using the following process:

1. **Header Extraction:** Extract `Authorization` header from request
2. **Token Parsing:** Parse Bearer token and extract JWT
3. **Signature Verification:** Verify JWT signature using server secret
4. **Expiration Check:** Verify token has not expired
5. **User Lookup:** Retrieve user from token subject
6. **Permission Check:** Verify user has required permissions
7. **Request Processing:** Proceed with authenticated request

**Error Responses:**

| Error Code | HTTP Status | Description |
|------------|--------------|-------------|
| `AUTH_REQUIRED` | 401 | Missing or invalid Authorization header |
| `AUTH_INVALID` | 401 | Invalid token signature or format |
| `AUTH_EXPIRED` | 403 | Token has expired |
| `PERMISSION_DENIED` | 403 | Insufficient permissions |

### 10.2. Authorization Requirements

#### 10.2.1. Role-Based Access Control (RBAC)

The API implements Role-Based Access Control (RBAC) to enforce fine-grained permissions.

**Roles:**

| Role | Description | Permissions |
|-------|-------------|-------------|
| `admin` | Full system access | All permissions |
| `editor` | Content editing access | `documents:read`, `documents:write`, `users:read` |
| `viewer` | Read-only access | `documents:read`, `users:read` |

**Permissions:**

| Permission | Description | Admin | Editor | Viewer |
|------------|-------------|-------|--------|--------|
| `documents:read` | Read documents | Yes | Yes | Yes |
| `documents:write` | Create/update documents | Yes | Yes | No |
| `documents:delete` | Delete documents | Yes | No | No |
| `users:read` | Read user profiles | Yes | Yes | Yes |
| `users:write` | Update user profiles | Yes | No | No |
| `users:delete` | Delete user accounts | Yes | No | No |
| `git:read` | Read repository status | Yes | Yes | Yes |
| `git:write` | Modify repository | Yes | No | No |

#### 10.2.2. Document-Level Access Control

Documents support fine-grained access control at the document level.

**Access Control Object:**

```json
{
  "read": ["*"],
  "write": ["admin", "editors"]
}
```

**Access Rules:**

- `read`: List of roles/groups with read access
- `write`: List of roles/groups with write access
- `"*"`: Wildcard for all authenticated users
- Empty list: No access (except owner)

**Example Configurations:**

| Configuration | Read Access | Write Access |
|--------------|-------------|--------------|
| Public document | All users | Admin only |
| Internal document | Admin, editors | Admin only |
| Private document | Owner only | Owner only |

### 10.3. Rate Limiting

#### 10.3.1. Rate Limit Configuration

The API implements rate limiting to prevent abuse and ensure fair resource allocation.

**Rate Limiting Algorithm:** Token bucket with sliding window

**Default Limits:**

| Endpoint Category | Max Requests | Window | Burst Allowance |
|-----------------|--------------|--------|----------------|
| Authentication | 5 | 1 minute | 10 |
| Document operations | 100 | 1 minute | 120 |
| Search | 50 | 1 minute | 60 |
| User operations | 20 | 1 minute | 30 |
| Git operations | 30 | 1 minute | 40 |

**Rate Limit Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| `RateLimit-Limit` | Maximum requests per window | `100` |
| `RateLimit-Remaining` | Remaining requests in window | `95` |
| `RateLimit-Reset` | Unix timestamp when window resets | `1738732860` |
| `RateLimit-Retry-After` | Seconds until retry allowed | `30` |

#### 10.3.2. Rate Limit Exceeded Response

When rate limit is exceeded, the API returns:

**HTTP Status:** `429 Too Many Requests`

**Response Body:**

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded. Please retry after 30 seconds.",
    "details": {
      "limit": 100,
      "window": 60,
      "retry_after": 30
    },
    "request_id": "200e8400-e29b-41d4-a716-4466554400013",
    "timestamp": "2026-02-05T14:44:00Z"
  }
}
```

### 10.4. CORS Configuration

#### 10.4.1. CORS Policy

The API implements Cross-Origin Resource Sharing (CORS) to allow controlled cross-origin requests.

**Allowed Origins:** Configurable per deployment

**Default Configuration:**

| Setting | Value | Description |
|----------|-------|-------------|
| `allowed_origins` | `["https://tachyon.example.com"]` | Allowed origin domains |
| `allowed_methods` | `["GET", "POST", "PUT", "DELETE", "PATCH"]` | Allowed HTTP methods |
| `allowed_headers` | `["Authorization", "Content-Type"]` | Allowed request headers |
| `exposed_headers` | `["X-Request-ID", "X-Response-Time"]` | Exposed response headers |
| `max_age` | `3600` | Cache duration in seconds |
| `allow_credentials` | `false` | Allow credentials in requests |

#### 10.4.2. CORS Headers

**Response Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| `Access-Control-Allow-Origin` | Allowed origin | `https://tachyon.example.com` |
| `Access-Control-Allow-Methods` | Allowed methods | `GET, POST, PUT, DELETE, PATCH, OPTIONS` |
| `Access-Control-Allow-Headers` | Allowed headers | `Authorization, Content-Type` |
| `Access-Control-Expose-Headers` | Exposed headers | `X-Request-ID, X-Response-Time` |
| `Access-Control-Max-Age` | Cache duration | `3600` |
| `Access-Control-Allow-Credentials` | Allow credentials | `false` |

### 10.5. Input Validation

#### 10.5.1. Validation Strategy

The API implements comprehensive input validation to prevent injection attacks and ensure data integrity.

**Validation Layers:**

1. **Type Validation:** Verify data types match expected schema
2. **Range Validation:** Enforce numeric and string length constraints
3. **Format Validation:** Verify formats (UUID, email, datetime)
4. **Business Logic Validation:** Enforce business rules
5. **Sanitization:** Remove or escape dangerous content

#### 10.5.2. Sanitization

**Markdown Content:** Sanitized using pulldown-cmark with safe rendering

- Removes HTML tags (except allowed subset)
- Escapes special characters
- Validates links and images
- Prevents XSS attacks

**JSON Content:** Parsed using serde with strict validation

- Type-safe deserialization
- Rejects unknown fields
- Validates structure before processing

**Path Parameters:** Validated against allowed character set

- Prevents directory traversal attacks
- Validates UUID format
- Sanitizes file paths

### 10.6. Security Headers

#### 10.6.1. Security-Related Response Headers

**Response Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| `Strict-Transport-Security` | Enforce HTTPS | `max-age=31536000; includeSubDomains` |
| `X-Content-Type-Options` | Prevent MIME sniffing | `nosniff` |
| `X-Frame-Options` | Prevent clickjacking | `DENY` |
| `Content-Security-Policy` | XSS protection | `default-src 'self'` |
| `X-XSS-Protection` | XSS filter | `1; mode=block` |

---

## 11. API PERFORMANCE

### 11.1. Latency Requirements

#### 11.1.1. Performance Targets

The API is designed to meet the following latency requirements:

| Endpoint | P50 Latency | P95 Latency | P99 Latency |
|----------|-------------|--------------|--------------|
| Document list | 10 ms | 25 ms | 50 ms |
| Document get | 15 ms | 30 ms | 60 ms |
| Document create | 20 ms | 40 ms | 80 ms |
| Document update | 20 ms | 40 ms | 80 ms |
| Document delete | 15 ms | 30 ms | 60 ms |
| Git status | 10 ms | 25 ms | 50 ms |
| Commit history | 15 ms | 30 ms | 60 ms |
| Search | 20 ms | 50 ms | 100 ms |
| Autocomplete | 10 ms | 20 ms | 40 ms |
| Login | 25 ms | 50 ms | 100 ms |
| Logout | 10 ms | 20 ms | 40 ms |
| Token refresh | 15 ms | 30 ms | 60 ms |

#### 11.1.2. Latency Monitoring

The API measures and reports latency for all requests:

**Response Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| `X-Response-Time` | Response processing time (ms) | `15` |

**Latency Metrics:**

- Request processing time
- Database query time
- Git operation time
- Rendering time
- Total response time

### 11.2. Throughput Requirements

#### 11.2.1. Performance Targets

The API is designed to support the following throughput:

| Metric | Target | Measurement |
|---------|--------|-------------|
| Requests per second | 1,000 RPS | Sustained |
| Concurrent connections | 10,000 | Simultaneous |
| WebSocket connections | 5,000 | Per server |

#### 11.2.2. Throughput Optimization

**Optimization Techniques:**

1. **HTTP/2 Multiplexing:** Multiple requests over single connection
2. **Connection Pooling:** Reuse HTTP connections
3. **Asynchronous Processing:** Non-blocking I/O with Tokio
4. **Caching:** LRU cache for frequently accessed documents
5. **Database Indexing:** Optimized queries with proper indexes

### 11.3. Caching Strategies

#### 11.3.1. Cache Configuration

The API implements multi-layer caching for optimal performance.

**Cache Layers:**

| Layer | Technology | TTL | Purpose |
|--------|-------------|-----|---------|
| In-memory | LRU cache | 5 minutes | Frequently accessed documents |
| HTTP Cache | Browser cache | 1 hour | Static assets |
| CDN | Edge cache | 24 hours | Global distribution |

**Cache Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| `Cache-Control` | Cache directives | `public, max-age=3600` |
| `ETag` | Entity tag | `"a1b2c3d4e5f67890"` |
| `Last-Modified` | Modification time | `Tue, 05 Feb 2026 14:00:00 GMT` |

#### 11.3.2. Cache Invalidation

**Invalidation Triggers:**

- Document update or deletion
- Git repository changes
- User permission changes
- Configuration updates

**Invalidation Strategy:**

- Immediate invalidation on write operations
- Time-based expiration for read operations
- Tag-based validation for conditional requests

### 11.4. Optimization Techniques

#### 11.4.1. Database Optimization

**Optimization Strategies:**

1. **Connection Pooling:** Reuse database connections
2. **Prepared Statements:** Use prepared SQL statements
3. **Query Optimization:** Optimize queries with proper indexes
4. **Batch Operations:** Batch multiple operations where possible
5. **Read Replicas:** Distribute read operations across replicas

#### 11.4.2. Git Optimization

**Optimization Strategies:**

1. **Shallow Clones:** Use shallow clones for status checks
2. **Commit Caching:** Cache commit history
3. **Parallel Operations:** Parallelize independent Git operations
4. **Incremental Indexing:** Update search index incrementally

#### 11.4.3. Rendering Optimization

**Optimization Strategies:**

1. **JIT Rendering:** Render Markdown on demand
2. **Rendered Cache:** Cache rendered HTML
3. **Lazy Loading:** Load content as needed
4. **Incremental Rendering:** Render large documents in chunks

---

## 12. API DOCUMENTATION

### 12.1. OpenAPI Specification

The Tachyon API is documented using OpenAPI 3.0 specification for machine-readable API documentation.

**Specification Location:** `/api/v1/openapi.json`

**OpenAPI Structure:**

```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "Tachyon REST API",
    "version": "1.0.0",
    "description": "REST API for Tachyon documentation management system",
    "contact": {
      "name": "Tachyon Team",
      "email": "api@tachyon.example.com"
    },
    "license": {
      "name": "MIT",
      "url": "https://opensource.org/licenses/MIT"
    }
  },
  "servers": [
    {
      "url": "https://api.tachyon.example.com/api/v1",
      "description": "Production server"
    }
  ],
  "paths": {
    "/documents": {
      "get": {
        "summary": "List documents",
        "operationId": "listDocuments",
        "tags": ["Documents"],
        "security": [{"bearerAuth": []}],
        "responses": {
          "200": {
            "description": "Successful response",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/DocumentListResponse"
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "securitySchemes": {
      "bearerAuth": {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT"
      }
    },
    "schemas": {
      "DocumentListResponse": {
        "type": "object",
        "properties": {
          "data": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/DocumentMetadata"
            }
          },
          "success": {
            "type": "boolean"
          },
          "meta": {
            "$ref": "#/components/schemas/ResponseMeta"
          }
        }
      }
    }
  }
}
```

### 12.2. Code Examples

#### 12.2.1. JavaScript/TypeScript Example

**Fetch API Example:**

```typescript
// API Client Configuration
const API_BASE_URL = 'https://api.tachyon.example.com/api/v1';
const API_TOKEN = 'your-jwt-token-here';

// Request Headers
const headers = {
  'Authorization': `Bearer ${API_TOKEN}`,
  'Content-Type': 'application/json',
  'Accept': 'application/json'
};

// List Documents
async function listDocuments(offset = 0, limit = 20) {
  const response = await fetch(
    `${API_BASE_URL}/documents?offset=${offset}&limit=${limit}`,
    { headers }
  );
  
  if (!response.ok) {
    throw new Error(`API Error: ${response.status}`);
  }
  
  const data = await response.json();
  return data;
}

// Get Document
async function getDocument(documentId: string) {
  const response = await fetch(
    `${API_BASE_URL}/documents/${documentId}`,
    { headers }
  );
  
  if (!response.ok) {
    throw new Error(`API Error: ${response.status}`);
  }
  
  const data = await response.json();
  return data;
}

// Create Document
async function createDocument(document: CreateDocumentRequest) {
  const response = await fetch(
    `${API_BASE_URL}/documents`,
    {
      method: 'POST',
      headers,
      body: JSON.stringify(document)
    }
  );
  
  if (!response.ok) {
    throw new Error(`API Error: ${response.status}`);
  }
  
  const data = await response.json();
  return data;
}

// Usage Example
async function main() {
  try {
    const documents = await listDocuments(0, 20);
    console.log('Documents:', documents.data);
    
    const document = await getDocument(documents.data[0].id);
    console.log('Document:', document.data);
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

#### 12.2.2. Python Example

**Requests API Example:**

```python
import requests
from typing import Dict, Any, Optional

# API Client Configuration
API_BASE_URL = 'https://api.tachyon.example.com/api/v1'
API_TOKEN = 'your-jwt-token-here'

# Request Headers
headers = {
    'Authorization': f'Bearer {API_TOKEN}',
    'Content-Type': 'application/json',
    'Accept': 'application/json'
}

class TachyonAPI:
    def __init__(self, base_url: str = API_BASE_URL, token: str = API_TOKEN):
        self.base_url = base_url
        self.token = token
        self.headers = headers
    
    def list_documents(self, offset: int = 0, limit: int = 20) -> Dict[str, Any]:
        """List documents with pagination."""
        response = requests.get(
            f'{self.base_url}/documents',
            params={'offset': offset, 'limit': limit},
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()
    
    def get_document(self, document_id: str) -> Dict[str, Any]:
        """Get specific document by ID."""
        response = requests.get(
            f'{self.base_url}/documents/{document_id}',
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()
    
    def create_document(self, document: Dict[str, Any]) -> Dict[str, Any]:
        """Create new document."""
        response = requests.post(
            f'{self.base_url}/documents',
            json=document,
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()
    
    def update_document(self, document_id: str, document: Dict[str, Any]) -> Dict[str, Any]:
        """Update existing document."""
        response = requests.put(
            f'{self.base_url}/documents/{document_id}',
            json=document,
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()
    
    def delete_document(self, document_id: str) -> None:
        """Delete document by ID."""
        response = requests.delete(
            f'{self.base_url}/documents/{document_id}',
            headers=self.headers
        )
        response.raise_for_status()

# Usage Example
def main():
    try:
        api = TachyonAPI()
        
        # List documents
        documents = api.list_documents(offset=0, limit=20)
        print(f'Found {documents["meta"]["total"]} documents')
        
        # Get first document
        document = api.get_document(documents['data'][0]['id'])
        print(f'Document: {document["data"]["metadata"]["title"]}')
        
    except requests.exceptions.HTTPError as e:
        print(f'API Error: {e}')

if __name__ == '__main__':
    main()
```

### 12.3. Usage Examples

#### 12.3.1. Document Management Workflow

**Typical Document Management Workflow:**

```mermaid
sequenceDiagram
    participant Client
    participant API as "Tachyon API"
    participant Git as "Git Repository"
    participant Cache as "LRU Cache"
    
    Client->>API: GET /api/v1/documents
    API->>Cache: Check cache
    alt Cache Miss
        API->>Git: Read documents
        Git-->>API: Return documents
        API->>Cache: Store in cache
    end
    alt Cache Hit
        Cache-->>API: Return cached documents
    end
    API-->>Client: Return document list
    Client->>API: POST /api/v1/documents
    API->>Git: Create document
    API->>Cache: Invalidate cache
    Git-->>API: Document created
    API-->>Client: Return new document
```

#### 12.3.2. Authentication Workflow

**Typical Authentication Workflow:**

```mermaid
sequenceDiagram
    participant Client
    participant API as "Tachyon API"
    participant Auth as "Auth Service"
    
    Client->>API: POST /api/v1/auth/login
    API->>Auth: Validate credentials
    Auth-->>API: Return JWT token
    API-->>Client: Return token
    Client->>API: GET /api/v1/documents (with token)
    API->>Auth: Validate token
    Auth-->>API: Token valid
    API-->>Client: Return documents
    Client->>API: POST /api/v1/auth/refresh
    API->>Auth: Validate token
    Auth-->>API: Return new token
    API-->>Client: Return refreshed token
```

### 12.4. Best Practices

#### 12.4.1. API Usage Best Practices

**Recommended Practices:**

1. **Use HTTPS:** Always use HTTPS for API communication
2. **Validate Responses:** Validate response structure and status codes
3. **Handle Errors:** Implement proper error handling for all error codes
4. **Retry Logic:** Implement exponential backoff for retryable errors
5. **Rate Limiting:** Respect rate limit headers and implement backoff
6. **Token Management:** Store tokens securely and refresh before expiration
7. **Pagination:** Use pagination for large result sets
8. **Caching:** Respect cache headers and implement client-side caching
9. **Request IDs:** Include request IDs for tracing and debugging
10. **User Agent:** Include descriptive user agent for monitoring

#### 12.4.2. Security Best Practices

**Security Recommendations:**

1. **Token Security:** Never expose tokens in client-side code or logs
2. **HTTPS Only:** Reject unencrypted connections
3. **Input Validation:** Validate all input before sending to API
4. **Sanitization:** Sanitize all user-generated content
5. **Permission Principle:** Request minimum required permissions
6. **Audit Logging:** Log all API access for security monitoring
7. **Error Messages:** Never expose sensitive information in error messages
8. **CORS:** Configure CORS properly for your use case
9. **Rate Limiting:** Implement client-side rate limiting
10. **Session Management:** Implement proper session timeout and cleanup

---

## 13. REFERENCES

[1] R. T. Fielding, "Architectural Styles and the Design of Network-based Software Architectures," PhD dissertation, University of California, Irvine, 2000.

[2] RFC 7231, "Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content," IETF, 2014.

[3] TACHYON-STD-V1.0, "TACHYON: CODING AND DOCUMENTATION STANDARDS," February 2026.

[4] TACHYON-DES-API-V1.0, "TACHYON: API INTERFACES DESIGN," February 2026.

[5] TACHYON-ADR-003-V1.0, "ADR-003: Axum for HTTP/2 Server," February 2026.

[6] TACHYON-ADR-007-V1.0, "ADR-007: Tokio for Async Runtime," February 2026.

[7] RFC 7540, "Hypertext Transfer Protocol Version 2 (HTTP/2)," IETF, 2015.

[8] RFC 8252, "The JSON Data Interchange Format," IETF, 2017.
