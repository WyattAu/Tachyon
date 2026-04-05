# TACHYON: SERVER API SPECIFICATION

**Document ID:** TACHYON-API-002-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** Technical Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 830-1998, RFC 7540 (HTTP/2)
**Dependencies:** [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md), [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md), [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md), [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md), [TACHYON-TMA-V1.0](../../.specs/03_threat_model/analysis.md)

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [API Design Principles](#2-api-design-principles)
3. [Versioning Strategy](#3-versioning-strategy)
4. [HTTP/2 Endpoints](#4-http2-endpoints)
5. [WebSocket Endpoints](#5-websocket-endpoints)
6. [API Security](#6-api-security)
7. [API Performance](#7-api-performance)
8. [API Documentation](#8-api-documentation)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides a comprehensive specification of the Tachyon Server Application Programming Interface (API). The API defines the contract between the Axum-based HTTP/2 server component and client applications, including the Tauri desktop application and Leptos web frontend. This specification serves as the authoritative reference for API implementation, client integration, and system interoperability.

### 1.2. Scope

This specification encompasses:

**In Scope:**
- HTTP/2 RESTful API endpoints for document management
- WebSocket endpoints for real-time communication
- Authentication and authorization mechanisms
- Search and query interfaces
- Git integration endpoints
- File upload/download operations
- Notification and event streaming
- Error handling and status codes
- Rate limiting and DDoS protection
- Caching strategies and performance optimization

**Out of Scope:**
- Desktop application internal APIs (covered in desktop API specification)
- Web frontend component APIs (covered in web client API specification)
- Internal inter-component communication protocols (covered in IPC protocol specification)
- Build system and deployment APIs (covered in operations documentation)

### 1.3. Intended Audience

This specification is intended for:

1. **Backend Developers:** Implementing the server API using Axum framework
2. **Frontend Developers:** Integrating with the server API from Tauri or Leptos
3. **API Consumers:** Building third-party integrations with Tachyon
4. **Security Engineers:** Implementing authentication and authorization
5. **Quality Assurance:** Testing API compliance and behavior
6. **Technical Writers:** Creating user-facing API documentation

### 1.4. System Context

The Tachyon Server API operates within a three-tier architecture:

```mermaid
graph TB
    subgraph "Client Layer"
        Desktop[Tauri Desktop Application]
        Web[Leptos Web Frontend]
        ThirdParty[Third-Party Integrations]
    end
    
    subgraph "Server Layer"
        API[Server API - Axum HTTP/2]
        WS[WebSocket - Real-time Communication]
    end
    
    subgraph "Data Layer"
        Git[Git Repository Storage]
        DB[SQLite Database]
        Search[Tantivy Search Index]
        Cache[In-Memory Cache]
    end
    
    Desktop -->|HTTP/2 + WebSocket| API
    Web -->|HTTP/2 + WebSocket| API
    ThirdParty -->|HTTP/2| API
    API --> WS
    API --> Git
    API --> DB
    API --> Search
    API --> Cache
    WS --> Desktop
    WS --> Web
    
    style Desktop fill:#e1f5ff
    style Web fill:#e1f5ff
    style ThirdParty fill:#ffe1e1
    style API fill:#ccffcc
    style WS fill:#ccffcc
    style Git fill:#ccccff
    style DB fill:#ccccff
    style Search fill:#ccccff
    style Cache fill:#ccccff
```

### 1.5. Technology Stack

The Server API is implemented using the following technology stack:

| Component | Technology | Version | Purpose |
|-----------|-------------|----------|---------|
| **Programming Language** | Rust | 2024 Edition | Primary implementation language (ADR-001) |
| **HTTP Framework** | Axum | v0.7 | HTTP/2 server framework (ADR-003) |
| **Async Runtime** | Tokio | v1.0 | Async I/O and task scheduling (ADR-007) |
| **HTTP Protocol** | HTTP/2 | RFC 7540 | Multiplexing and header compression |
| **TLS Protocol** | TLS 1.3 | RFC 8446 | Secure communication |
| **WebSocket** | tokio-tungstenite | Latest | Real-time bidirectional communication |
| **Serialization** | serde, serde_json | Latest | JSON serialization/deserialization |
| **Database** | rusqlite | Latest | SQLite database bindings |
| **Search** | tantivy | Latest | Full-text search engine |
| **Git Integration** | git2 | Latest | Git repository operations |

---

## 2. API DESIGN PRINCIPLES

### 2.1. RESTful Design Principles

The Server API adheres to REST (Representational State Transfer) architectural principles for HTTP/2 endpoints:

#### 2.1.1. Resource-Oriented Design

**Principle:** Resources are the fundamental abstraction in the API, identified by URIs and manipulated through standard HTTP methods.

**Implementation:**
- Each resource type (documents, users, repositories) has a unique URI pattern
- Resources are hierarchical and reflect natural relationships
- Resource representations are self-describing with appropriate media types

**Resource URI Patterns:**

| Resource Type | URI Pattern | Example |
|---------------|--------------|----------|
| Document Collection | `/api/documents` | List all documents |
| Document Instance | `/api/documents/{id}` | Retrieve specific document |
| Repository Collection | `/api/repositories` | List all repositories |
| Repository Instance | `/api/repositories/{id}` | Retrieve specific repository |
| User Collection | `/api/users` | List all users |
| User Instance | `/api/users/{id}` | Retrieve specific user |

#### 2.1.2. Uniform Interface

**Principle:** The API provides a uniform interface for resource manipulation using standard HTTP methods.

**HTTP Method Semantics:**

| Method | Safe | Idempotent | Purpose | Example |
|--------|-------|-------------|---------|---------|
| GET | Yes | Yes | Retrieve resource representation | `GET /api/documents/{id}` |
| POST | No | No | Create new resource | `POST /api/documents` |
| PUT | No | Yes | Replace resource | `PUT /api/documents/{id}` |
| PATCH | No | No | Partial resource update | `PATCH /api/documents/{id}` |
| DELETE | No | Yes | Delete resource | `DELETE /api/documents/{id}` |
| HEAD | Yes | Yes | Retrieve resource metadata | `HEAD /api/documents/{id}` |
| OPTIONS | Yes | Yes | Retrieve communication options | `OPTIONS /api/documents` |

#### 2.1.3. Stateless Communication

**Principle:** Each request from client to server must contain all information necessary to understand and process the request.

**Implementation:**
- No server-side session state stored between requests
- All authentication and authorization data included in request headers
- No conversational state maintained across requests
- Session tokens are stateless (JWT) and self-contained

**Benefits:**
- Improved scalability through horizontal scaling
- Simplified server architecture
- Enhanced reliability through stateless request handling
- Easier caching and load balancing

#### 2.1.4. Cacheability

**Principle:** Responses must explicitly define their cacheability to improve performance and reduce server load.

**Cache-Control Directives:**

| Directive | Purpose | Example Use Case |
|-----------|---------|------------------|
| `public` | Response may be cached by any cache | Static assets, public documents |
| `private` | Response may only be cached by client | User-specific data |
| `no-cache` | Response must not be cached | Real-time data, authentication |
| `max-age={seconds}` | Maximum time to cache response | Static content with TTL |
| `must-revalidate` | Cache must validate before use | Content with conditional updates |
| `no-store` | Response must not be stored in cache | Sensitive data |

#### 2.1.5. Layered System

**Principle:** The client cannot determine whether it is connected to an end server or an intermediary.

**Implementation:**
- Load balancers and reverse proxies are transparent to clients
- Caching layers (CDN, reverse proxy cache) are transparent
- API gateway provides unified interface to multiple services
- Layered architecture enables independent scaling and deployment

### 2.2. HTTP/2 Specific Design

#### 2.2.1. Multiplexing

**Principle:** HTTP/2 enables multiple concurrent requests over a single TCP connection, reducing latency and connection overhead.

**Implementation:**
- Server supports HTTP/2 multiplexing for all clients
- Single TCP connection handles multiple concurrent requests
- Stream prioritization enables efficient resource delivery
- Connection reuse reduces handshake overhead

**Performance Impact:**

| Metric | HTTP/1.1 | HTTP/2 | Improvement |
|---------|-----------|---------|-------------|
| Connections per Page Load | 6 | 1 | 83% reduction |
| Latency (P50) | 120ms | 40ms | 67% improvement |
| Bandwidth Usage | 1.2 MB | 0.8 MB | 33% reduction |
| Connection Overhead | High | Low | Significant reduction |

#### 2.2.2. Header Compression

**Principle:** HTTP/2 uses HPACK compression to reduce header overhead.

**Implementation:**
- Server uses HPACK compression for all HTTP/2 connections
- Static table compression for common headers
- Dynamic table compression for request-specific headers
- Huffman encoding for efficient header representation

**Header Compression Benefits:**

| Header Type | HTTP/1.1 Size | HTTP/2 Size | Compression Ratio |
|--------------|-----------------|---------------|-------------------|
| Request Headers | 820 bytes | 280 bytes | 66% reduction |
| Response Headers | 650 bytes | 220 bytes | 66% reduction |
| Total per Request | 1,470 bytes | 500 bytes | 66% reduction |

#### 2.2.3. Server Push

**Principle:** HTTP/2 enables proactive resource pushing to reduce latency.

**Implementation:**
- Server pushes critical CSS and JavaScript resources
- Pushed resources are associated with the requesting page
- Client can decline pushed resources if already cached
- Server push is used selectively for performance-critical resources

**Server Push Use Cases:**

| Resource Type | Push Condition | Benefit |
|---------------|-----------------|---------|
| Critical CSS | First page load | Eliminates request latency |
| Critical JavaScript | First page load | Enables earlier execution |
| Inline Images | First page load | Reduces layout shift |
| Font Files | First page load | Improves text rendering |

### 2.3. Error Handling Principles

#### 2.3.1. HTTP Status Code Usage

**Principle:** Appropriate HTTP status codes are used to indicate the outcome of API requests.

**Status Code Categories:**

| Category | Status Codes | Purpose |
|----------|---------------|---------|
| **Informational** | 100-199 | Request received, continuing processing |
| **Success** | 200-299 | Request successfully received, understood, and accepted |
| **Redirection** | 300-399 | Further action needed to complete request |
| **Client Error** | 400-499 | Request contains bad syntax or cannot be fulfilled |
| **Server Error** | 500-599 | Server failed to fulfill valid request |

**Common Status Codes:**

| Status Code | Name | Usage Condition |
|-------------|------|----------------|
| 200 | OK | Successful GET, PUT, PATCH, DELETE |
| 201 | Created | Successful POST creating new resource |
| 204 | No Content | Successful DELETE or PUT with no response body |
| 400 | Bad Request | Malformed request syntax |
| 401 | Unauthorized | Authentication required or failed |
| 403 | Forbidden | Authentication successful but insufficient permissions |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Request conflicts with current resource state |
| 422 | Unprocessable Entity | Well-formed request but semantic errors |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Unexpected server error |
| 503 | Service Unavailable | Server temporarily unavailable |

#### 2.3.2. Error Response Format

**Principle:** Error responses follow a consistent format providing sufficient information for client handling.

**Error Response Schema:**

```json
{
  "error": {
    "code": "string",
    "message": "string",
    "details": {
      "field": "string",
      "reason": "string"
    },
    "request_id": "string",
    "timestamp": "ISO 8601 UTC timestamp"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `error` | Object | Yes | Root error object |
| `error.code` | String | Yes | Machine-readable error code |
| `error.message` | String | Yes | Human-readable error message |
| `error.details` | Object | No | Additional error details |
| `error.details.field` | String | No | Field causing error (for validation errors) |
| `error.details.reason` | String | No | Reason for error (for validation errors) |
| `error.request_id` | String | Yes | Unique request identifier for debugging |
| `error.timestamp` | String | Yes | ISO 8601 UTC timestamp of error |

**Error Response Example:**

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": {
      "field": "title",
      "reason": "Title must be between 1 and 200 characters"
    },
    "request_id": "req_1234567890",
    "timestamp": "2026-02-05T13:00:00.000Z"
  }
}
```

### 2.4. Security Principles

#### 2.4.1. Defense in Depth

**Principle:** Multiple layers of security controls protect the API from various attack vectors.

**Security Layers:**

| Layer | Security Controls | Threats Mitigated |
|-------|------------------|-------------------|
| **Network** | TLS 1.3, DDoS protection, rate limiting | Eavesdropping, DoS, brute force |
| **Transport** | HTTP/2, CORS, CSP | Protocol attacks, XSS |
| **Application** | Input validation, output encoding, RBAC | Injection, privilege escalation |
| **Data** | Encryption at rest, integrity verification | Data theft, tampering |
| **Audit** | Comprehensive logging, anomaly detection | Repudiation, unauthorized access |

#### 2.4.2. Principle of Least Privilege

**Principle:** Users and components are granted only the minimum permissions necessary to perform their functions.

**Implementation:**
- Role-Based Access Control (RBAC) with fine-grained permissions
- Default deny-all authorization policy
- Scoped authentication tokens with limited permissions
- Separate read and write permissions for resources

**Permission Model:**

| Permission | Scope | Description |
|------------|-------|-------------|
| `documents:read` | Document resources | Read access to documents |
| `documents:write` | Document resources | Write access to documents |
| `documents:delete` | Document resources | Delete access to documents |
| `repositories:read` | Repository resources | Read access to repositories |
| `repositories:write` | Repository resources | Write access to repositories |
| `users:manage` | User resources | Manage user accounts |
| `system:admin` | System resources | Administrative system access |

#### 2.4.3. Secure by Default

**Principle:** The API is configured with secure defaults, requiring explicit configuration for less secure options.

**Secure Defaults:**

| Setting | Default Value | Rationale |
|---------|----------------|-----------|
| TLS | Enabled | Encryption at rest and in transit |
| Authentication | Required | Prevent unauthorized access |
| Rate Limiting | Enabled | Prevent abuse and DoS |
| CORS | Restricted | Prevent unauthorized cross-origin access |
| Content Security Policy | Strict | Prevent XSS attacks |
| Session Timeout | 1 hour | Limit exposure of compromised sessions |
| Password Policy | Strong | Prevent weak credentials |

---

## 3. VERSIONING STRATEGY

### 3.1. API Versioning Approach

The Server API employs **URI-based versioning** for clear and explicit version identification.

**Rationale for URI-Based Versioning:**

| Approach | Advantages | Disadvantages |
|----------|-------------|----------------|
| **URI-Based** | Clear version identification, easy caching, explicit version selection | Requires client updates for new versions |
| **Header-Based** | Clean URIs, flexible version selection | Caching complexity, less explicit |
| **Content Negotiation** | Standard HTTP mechanism, flexible | Complex implementation, less explicit |
| **No Versioning** | Simplicity, no version management | Breaking changes require coordinated updates |

**URI Version Pattern:**

```
/api/v{version}/{resource}
```

**Example URIs:**

| Version | Example URI | Description |
|---------|--------------|-------------|
| v1 | `/api/v1/documents` | Current stable API version |
| v2 | `/api/v2/documents` | Next API version (when available) |

### 3.2. Version Lifecycle

#### 3.2.1. Version States

| State | Description | URI Pattern | Support Duration |
|-------|-------------|--------------|-----------------|
| **Development** | Work in progress, may change | `/api/v{version}-dev/{resource}` | Until stable |
| **Stable** | Production-ready, stable interface | `/api/v{version}/{resource}` | Minimum 12 months |
| **Deprecated** | Still supported, will be retired | `/api/v{version}/{resource}` | 6-12 months |
| **Retired** | No longer supported | N/A | N/A |

#### 3.2.2. Version Deprecation Process

**Deprecation Timeline:**

1. **Announcement (Month 0):** Public announcement of deprecation
2. **Warning Period (Months 1-3):** Warning headers in API responses
3. **Transition Period (Months 4-6):** Client migration support
4. **Sunset (Month 6):** Version retired, endpoints return 410 Gone

**Deprecation Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `X-API-Deprecated` | `true` | Indicates deprecated version |
| `X-API-Sunset-Date` | ISO 8601 date | Date of version retirement |
| `X-API-Recommended-Version` | `v{version}` | Recommended version to migrate to |
| `Link` | `<https://api.example.com/docs/v{version}>; rel="alternate"` | Link to new version documentation |

### 3.3. Backward Compatibility

#### 3.3.1. Compatibility Guarantees

**Stable Version Guarantees:**

| Guarantee | Description | Duration |
|-----------|-------------|----------|
| **URI Stability** | Existing URIs remain functional | Entire stable lifecycle |
| **Response Format** | Response schema remains compatible | Entire stable lifecycle |
| **HTTP Methods** | Existing HTTP methods remain supported | Entire stable lifecycle |
| **Error Codes** | Existing error codes remain consistent | Entire stable lifecycle |
| **Authentication** | Existing authentication methods remain supported | Entire stable lifecycle |

#### 3.3.2. Breaking Changes

**Definition:** Changes that require client modifications to maintain functionality.

**Breaking Change Categories:**

| Category | Examples | Mitigation |
|----------|-----------|-------------|
| **URI Changes** | Renamed resources, changed URI structure | New version, redirect from old version |
| **HTTP Method Changes** | Changed allowed methods for resource | New version, 405 Method Not Allowed |
| **Request Schema Changes** | Removed or renamed fields | New version, validation error |
| **Response Schema Changes** | Removed or renamed fields | New version, version-specific response |
| **Authentication Changes** | Changed authentication mechanism | New version, 401 Unauthorized |

**Non-Breaking Changes:**

| Change Type | Examples | Compatibility Impact |
|-------------|-----------|---------------------|
| **Additive Changes** | New optional fields, new resources | Fully backward compatible |
| **Bug Fixes** | Corrected behavior without schema change | Fully backward compatible |
| **Documentation Updates** | Clarified descriptions, added examples | No API impact |
| **Performance Improvements** | Optimized response times, caching | No API impact |

### 3.4. Version Negotiation

#### 3.4.1. Explicit Version Selection

**Principle:** Clients explicitly specify the API version in the request URI.

**Implementation:**

```
GET /api/v1/documents HTTP/2
Host: api.tachyon.example.com
```

**Benefits:**
- Clear version identification
- Simplified caching
- Explicit client control
- Easy version-specific routing

#### 3.4.2. Default Version

**Principle:** When no version is specified, the server defaults to the current stable version.

**Implementation:**

```
GET /api/documents HTTP/2
Host: api.tachyon.example.com
→ Redirects to /api/v1/documents
```

**Response Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Location` | `/api/v1/documents` | Redirect to default version |
| `X-API-Version` | `v1` | Indicates current version |
| `Cache-Control` | `public, max-age=3600` | Cache redirect |

#### 3.4.3. Version Discovery

**Principle:** Clients can discover available API versions through a dedicated endpoint.

**Endpoint:**

```
GET /api/versions HTTP/2
```

**Response:**

```json
{
  "versions": [
    {
      "version": "v1",
      "status": "stable",
      "release_date": "2026-02-01T00:00:00.000Z",
      "deprecation_date": null,
      "sunset_date": null,
      "documentation_url": "https://api.tachyon.example.com/docs/v1"
    },
    {
      "version": "v2",
      "status": "development",
      "release_date": null,
      "deprecation_date": null,
      "sunset_date": null,
      "documentation_url": "https://api.tachyon.example.com/docs/v2-dev"
    }
  ],
  "default_version": "v1"
}
```

---

## 4. HTTP/2 ENDPOINTS

### 4.1. Document Endpoints

#### 4.1.1. List Documents

**Endpoint:** `GET /api/v1/documents`

**Description:** Retrieves a paginated list of documents accessible to the authenticated user.

**Authentication:** Required

**Authorization:** `documents:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `page` | Integer | Query | No | Page number (1-indexed) | Minimum: 1, Default: 1 |
| `page_size` | Integer | Query | No | Number of documents per page | Minimum: 1, Maximum: 100, Default: 20 |
| `sort_by` | String | Query | No | Field to sort by | Values: `created_at`, `updated_at`, `title`, Default: `created_at` |
| `sort_order` | String | Query | No | Sort order | Values: `asc`, `desc`, Default: `desc` |
| `filter` | String | Query | No | Filter query (searches title and content) | Maximum length: 256 characters |
| `tag` | String | Query | No | Filter by tag | Maximum length: 64 characters |

**Request Example:**

```http
GET /api/v1/documents?page=2&page_size=50&sort_by=updated_at&sort_order=desc HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": [
    {
      "id": "string",
      "title": "string",
      "path": "string",
      "created_at": "ISO 8601 UTC timestamp",
      "updated_at": "ISO 8601 UTC timestamp",
      "author": {
        "id": "string",
        "username": "string"
      },
      "tags": ["string"],
      "metadata": {
        "word_count": "integer",
        "reading_time_minutes": "integer"
      }
    }
  ],
  "pagination": {
    "page": "integer",
    "page_size": "integer",
    "total_items": "integer",
    "total_pages": "integer",
    "has_next": "boolean",
    "has_previous": "boolean"
  }
}
```

**Response Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Content-Type` | `application/json` | Response media type |
| `Cache-Control` | `private, max-age=60` | Caching directive |
| `X-Request-ID` | UUID | Request identifier for debugging |
| `X-Response-Time` | Milliseconds | Server processing time |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Documents retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 422 Unprocessable Entity | Query parameter validation failed |

**Related Requirements:**
- REQ-SRV-021: Document List Endpoint

#### 4.1.2. Retrieve Document

**Endpoint:** `GET /api/v1/documents/{id}`

**Description:** Retrieves a specific document by its unique identifier.

**Authentication:** Required

**Authorization:** `documents:read` permission required

**Path Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `id` | String | Yes | Unique document identifier | UUID v4 format |

**Query Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `format` | String | No | Response format | Values: `json`, `html`, `markdown`, Default: `json` |
| `include_metadata` | Boolean | No | Include document metadata | Default: `true` |
| `include_content` | Boolean | No | Include document content | Default: `true` |

**Request Example:**

```http
GET /api/v1/documents/550e8400-e29b-41d4-a716-446655440100?format=html&include_metadata=true&include_content=true HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: text/html
```

**Response Schema (JSON format):**

```json
{
  "data": {
    "id": "string",
    "title": "string",
    "path": "string",
    "content": "string",
    "html": "string",
    "created_at": "ISO 8601 UTC timestamp",
    "updated_at": "ISO 8601 UTC timestamp",
    "author": {
      "id": "string",
      "username": "string",
      "email": "string"
    },
    "tags": ["string"],
    "frontmatter": {
      "title": "string",
      "description": "string",
      "tags": ["string"],
      "access": "string",
      "custom": "object"
    },
    "metadata": {
      "word_count": "integer",
      "reading_time_minutes": "integer",
      "last_commit_hash": "string",
      "branch": "string"
    }
  }
}
```

**Response Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Content-Type` | `application/json` or `text/html` or `text/markdown` | Response media type |
| `ETag` | Content hash | Entity tag for conditional requests |
| `Cache-Control` | `private, max-age=300` | Caching directive |
| `X-Request-ID` | UUID | Request identifier for debugging |
| `X-Response-Time` | Milliseconds | Server processing time |

**Conditional Requests:**

The endpoint supports conditional requests using `If-None-Match` and `If-Match` headers:

| Header | Value | Purpose |
|--------|-------|---------|
| `If-None-Match` | ETag value | Return 304 Not Modified if content unchanged |
| `If-Match` | ETag value | Return 412 Precondition Failed if content changed |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Document retrieved successfully |
| 304 Not Modified | Document not modified (conditional request) |
| 400 Bad Request | Invalid path or query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Document not found |
| 412 Precondition Failed | Conditional request precondition failed |

**Related Requirements:**
- REQ-SRV-022: Document Retrieval Endpoint

#### 4.1.3. Create Document

**Endpoint:** `POST /api/v1/documents`

**Description:** Creates a new document in the repository.

**Authentication:** Required

**Authorization:** `documents:write` permission required

**Request Headers:**

| Header | Required | Description |
|--------|-----------|-------------|
| `Content-Type` | Yes | Request media type (must be `application/json`) |
| `Authorization` | Yes | Authentication token |

**Request Body Schema:**

```json
{
  "title": "string",
  "content": "string",
  "path": "string",
  "tags": ["string"],
  "frontmatter": {
    "description": "string",
    "access": "string",
    "custom": "object"
  },
  "commit_message": "string"
}
```

**Field Descriptions:**

| Field | Type | Required | Description | Constraints |
|-------|------|-----------|-------------|-------------|
| `title` | String | Yes | Document title | Minimum: 1, Maximum: 200 characters |
| `content` | String | Yes | Document content (Markdown) | Maximum: 10 MB |
| `path` | String | Yes | Document file path | Maximum: 512 characters, must end with `.md` |
| `tags` | Array | No | Document tags | Maximum: 10 tags, 64 characters each |
| `frontmatter` | Object | No | YAML frontmatter metadata | Maximum: 5 KB |
| `frontmatter.description` | String | No | Document description | Maximum: 500 characters |
| `frontmatter.access` | String | No | Access control level | Values: `public`, `internal`, `restricted`, Default: `internal` |
| `frontmatter.custom` | Object | No | Custom metadata fields | Maximum: 10 fields |
| `commit_message` | String | No | Git commit message | Maximum: 200 characters, Default: "Create {title}" |

**Request Example:**

```http
POST /api/v1/documents HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Content-Type: application/json
Content-Length: 245

{
  "title": "Introduction to Tachyon",
  "content": "# Introduction\n\nTachyon is a modern documentation platform...",
  "path": "docs/introduction.md",
  "tags": ["getting-started", "introduction"],
  "frontmatter": {
    "description": "An introduction to Tachyon platform",
    "access": "public"
  },
  "commit_message": "Add introduction document"
}
```

**Response Schema:**

```json
{
  "data": {
    "id": "string",
    "title": "string",
    "path": "string",
    "content": "string",
    "created_at": "ISO 8601 UTC timestamp",
    "updated_at": "ISO 8601 UTC timestamp",
    "author": {
      "id": "string",
      "username": "string"
    },
    "tags": ["string"],
    "commit": {
      "hash": "string",
      "message": "string",
      "author": {
        "username": "string",
        "email": "string"
      },
      "timestamp": "ISO 8601 UTC timestamp"
    }
  }
}
```

**Response Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Content-Type` | `application/json` | Response media type |
| `Location` | `/api/v1/documents/{id}` | URI of created document |
| `X-Request-ID` | UUID | Request identifier for debugging |
| `X-Response-Time` | Milliseconds | Server processing time |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 201 Created | Document created successfully |
| 400 Bad Request | Invalid request body or parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 409 Conflict | Document path already exists |
| 413 Payload Too Large | Request body exceeds size limit |
| 422 Unprocessable Entity | Request body validation failed |

**Related Requirements:**
- REQ-SRV-023: Document Creation Endpoint

#### 4.1.4. Update Document

**Endpoint:** `PUT /api/v1/documents/{id}`

**Description:** Updates an existing document by replacing its content entirely.

**Authentication:** Required

**Authorization:** `documents:write` permission required

**Path Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `id` | String | Yes | Unique document identifier | UUID v4 format |

**Query Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `create_commit` | Boolean | No | Create Git commit for update | Default: `true` |

**Request Body Schema:**

```json
{
  "title": "string",
  "content": "string",
  "path": "string",
  "tags": ["string"],
  "frontmatter": {
    "description": "string",
    "access": "string",
    "custom": "object"
  },
  "commit_message": "string"
}
```

**Request Example:**

```http
PUT /api/v1/documents/550e8400-e29b-41d4-a716-446655440100?create_commit=true HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Content-Type: application/json
Content-Length: 245

{
  "title": "Introduction to Tachyon (Updated)",
  "content": "# Introduction\n\nTachyon is a modern documentation platform with enhanced features...",
  "path": "docs/introduction.md",
  "tags": ["getting-started", "introduction", "updated"],
  "frontmatter": {
    "description": "An updated introduction to Tachyon platform",
    "access": "public"
  },
  "commit_message": "Update introduction document"
}
```

**Response Schema:**

```json
{
  "data": {
    "id": "string",
    "title": "string",
    "path": "string",
    "content": "string",
    "updated_at": "ISO 8601 UTC timestamp",
    "author": {
      "id": "string",
      "username": "string"
    },
    "tags": ["string"],
    "commit": {
      "hash": "string",
      "message": "string",
      "author": {
        "username": "string",
        "email": "string"
      },
      "timestamp": "ISO 8601 UTC timestamp"
    }
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Document updated successfully |
| 400 Bad Request | Invalid request body or parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Document not found |
| 409 Conflict | Concurrent modification conflict |
| 413 Payload Too Large | Request body exceeds size limit |
| 422 Unprocessable Entity | Request body validation failed |

**Related Requirements:**
- REQ-SRV-024: Document Update Endpoint

#### 4.1.5. Delete Document

**Endpoint:** `DELETE /api/v1/documents/{id}`

**Description:** Deletes an existing document from the repository.

**Authentication:** Required

**Authorization:** `documents:delete` permission required

**Path Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `id` | String | Yes | Unique document identifier | UUID v4 format |

**Query Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `create_commit` | Boolean | No | Create Git commit for deletion | Default: `true` |
| `commit_message` | String | No | Custom commit message | Maximum: 200 characters |

**Request Example:**

```http
DELETE /api/v1/documents/550e8400-e29b-41d4-a716-446655440100?create_commit=true&commit_message=Remove%20introduction%20document HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
```

**Response Schema:**

```json
{
  "data": {
    "id": "string",
    "title": "string",
    "path": "string",
    "deleted_at": "ISO 8601 UTC timestamp",
    "commit": {
      "hash": "string",
      "message": "string",
      "author": {
        "username": "string",
        "email": "string"
      },
      "timestamp": "ISO 8601 UTC timestamp"
    }
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Document deleted successfully |
| 204 No Content | Document deleted successfully (no response body) |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Document not found |

**Related Requirements:**
- REQ-SRV-025: Document Deletion Endpoint

### 4.2. Repository Endpoints

#### 4.2.1. List Repositories

**Endpoint:** `GET /api/v1/repositories`

**Description:** Retrieves a list of repositories accessible to the authenticated user.

**Authentication:** Required

**Authorization:** `repositories:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `page` | Integer | Query | No | Page number (1-indexed) | Minimum: 1, Default: 1 |
| `page_size` | Integer | Query | No | Number of repositories per page | Minimum: 1, Maximum: 100, Default: 20 |

**Request Example:**

```http
GET /api/v1/repositories?page=1&page_size=50 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": [
    {
      "id": "string",
      "name": "string",
      "description": "string",
      "path": "string",
      "url": "string",
      "default_branch": "string",
      "created_at": "ISO 8601 UTC timestamp",
      "updated_at": "ISO 8601 UTC timestamp",
      "owner": {
        "id": "string",
        "username": "string"
      },
      "access_level": "string",
      "is_private": "boolean"
    }
  ],
  "pagination": {
    "page": "integer",
    "page_size": "integer",
    "total_items": "integer",
    "total_pages": "integer",
    "has_next": "boolean",
    "has_previous": "boolean"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Repositories retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |

#### 4.2.2. Retrieve Repository

**Endpoint:** `GET /api/v1/repositories/{id}`

**Description:** Retrieves details of a specific repository.

**Authentication:** Required

**Authorization:** `repositories:read` permission required

**Path Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `id` | String | Yes | Unique repository identifier | UUID v4 format |

**Response Schema:**

```json
{
  "data": {
    "id": "string",
    "name": "string",
    "description": "string",
    "path": "string",
    "url": "string",
    "default_branch": "string",
    "current_branch": "string",
    "created_at": "ISO 8601 UTC timestamp",
    "updated_at": "ISO 8601 UTC timestamp",
    "owner": {
      "id": "string",
      "username": "string",
      "email": "string"
    },
    "access_level": "string",
    "is_private": "boolean",
    "statistics": {
      "document_count": "integer",
      "commit_count": "integer",
      "branch_count": "integer",
      "contributor_count": "integer"
    }
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Repository retrieved successfully |
| 400 Bad Request | Invalid path parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Repository not found |

### 4.3. Search Endpoints

#### 4.3.1. Full-Text Search

**Endpoint:** `GET /api/v1/search`

**Description:** Performs full-text search across all accessible documents.

**Authentication:** Required

**Authorization:** `documents:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `q` | String | Query | Yes | Search query string | Minimum: 1, Maximum: 256 characters |
| `page` | Integer | Query | No | Page number (1-indexed) | Minimum: 1, Default: 1 |
| `page_size` | Integer | Query | No | Number of results per page | Minimum: 1, Maximum: 100, Default: 20 |
| `repository_id` | String | Query | No | Filter by repository | UUID v4 format |
| `tag` | String | Query | No | Filter by tag | Maximum: 64 characters |
| `author_id` | String | Query | No | Filter by author | UUID v4 format |
| `date_from` | String | Query | No | Filter by date (from) | ISO 8601 UTC timestamp |
| `date_to` | String | Query | No | Filter by date (to) | ISO 8601 UTC timestamp |
| `highlight` | Boolean | Query | No | Highlight search terms | Default: `true` |

**Request Example:**

```http
GET /api/v1/search?q=authentication&page=1&page_size=20&highlight=true HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": [
    {
      "id": "string",
      "title": "string",
      "path": "string",
      "repository_id": "string",
      "repository_name": "string",
      "snippet": "string",
      "score": "float",
      "highlighted_content": "string",
      "created_at": "ISO 8601 UTC timestamp",
      "updated_at": "ISO 8601 UTC timestamp",
      "author": {
        "id": "string",
        "username": "string"
      },
      "tags": ["string"]
    }
  ],
  "pagination": {
    "page": "integer",
    "page_size": "integer",
    "total_items": "integer",
    "total_pages": "integer",
    "has_next": "boolean",
    "has_previous": "boolean"
  },
  "search_metadata": {
    "query": "string",
    "execution_time_ms": "float",
    "total_documents_searched": "integer"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Search completed successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 422 Unprocessable Entity | Query validation failed |

**Related Requirements:**
- REQ-SRV-026: Full-Text Search Endpoint
- REQ-SRV-027: Faceted Search Endpoint
- REQ-SRV-028: Search Autocomplete Endpoint
- REQ-SRV-029: Search Pagination
- REQ-SRV-030: Search Highlighting

#### 4.3.2. Search Autocomplete

**Endpoint:** `GET /api/v1/search/autocomplete`

**Description:** Provides search suggestions based on partial query input.

**Authentication:** Required

**Authorization:** `documents:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `q` | String | Query | Yes | Partial search query | Minimum: 1, Maximum: 100 characters |
| `limit` | Integer | Query | No | Maximum number of suggestions | Minimum: 1, Maximum: 10, Default: 5 |

**Request Example:**

```http
GET /api/v1/search/autocomplete?q=auth&limit=5 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": [
    {
      "suggestion": "string",
      "type": "string",
      "count": "integer"
    }
  ]
}
```

**Suggestion Types:**

| Type | Description |
|------|-------------|
| `document` | Suggestion is a document title |
| `tag` | Suggestion is a tag |
| `author` | Suggestion is an author name |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Suggestions retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |

### 4.4. Git Integration Endpoints

#### 4.4.1. Repository Status

**Endpoint:** `GET /api/v1/git/status`

**Description:** Retrieves Git repository status for the current repository.

**Authentication:** Required

**Authorization:** `repositories:read` permission required

**Query Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `repository_id` | String | Yes | Repository identifier | UUID v4 format |

**Request Example:**

```http
GET /api/v1/git/status?repository_id=550e8400-e29b-41d4-a716-446655440100 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": {
    "repository_id": "string",
    "branch": "string",
    "head_commit": {
      "hash": "string",
      "message": "string",
      "author": {
        "username": "string",
        "email": "string"
      },
      "timestamp": "ISO 8601 UTC timestamp"
    },
    "status": {
      "branch_ahead": "integer",
      "branch_behind": "integer",
      "has_uncommitted_changes": "boolean",
      "untracked_files": ["string"],
      "modified_files": ["string"],
      "staged_files": ["string"]
    }
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Status retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Repository not found |

**Related Requirements:**
- REQ-SRV-031: Repository Status Endpoint

#### 4.4.2. Commit History

**Endpoint:** `GET /api/v1/git/commits`

**Description:** Retrieves commit history for a repository.

**Authentication:** Required

**Authorization:** `repositories:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `repository_id` | String | Query | Yes | Repository identifier | UUID v4 format |
| `branch` | String | Query | No | Branch to retrieve history for | Maximum: 256 characters |
| `page` | Integer | Query | No | Page number (1-indexed) | Minimum: 1, Default: 1 |
| `page_size` | Integer | Query | No | Number of commits per page | Minimum: 1, Maximum: 100, Default: 20 |

**Request Example:**

```http
GET /api/v1/git/commits?repository_id=550e8400-e29b-41d4-a716-446655440100&page=1&page_size=20 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": [
    {
      "hash": "string",
      "short_hash": "string",
      "message": "string",
      "author": {
        "username": "string",
        "email": "string"
      },
      "committer": {
        "username": "string",
        "email": "string"
      },
      "timestamp": "ISO 8601 UTC timestamp",
      "branch": "string",
      "files_changed": ["string"]
    }
  ],
  "pagination": {
    "page": "integer",
    "page_size": "integer",
    "total_items": "integer",
    "total_pages": "integer",
    "has_next": "boolean",
    "has_previous": "boolean"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Commits retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Repository not found |

**Related Requirements:**
- REQ-SRV-032: Commit History Endpoint

#### 4.4.3. Branch List

**Endpoint:** `GET /api/v1/git/branches`

**Description:** Retrieves a list of branches in the repository.

**Authentication:** Required

**Authorization:** `repositories:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `repository_id` | String | Query | Yes | Repository identifier | UUID v4 format |

**Request Example:**

```http
GET /api/v1/git/branches?repository_id=550e8400-e29b-41d4-a716-446655440100 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": [
    {
      "name": "string",
      "is_default": "boolean",
      "is_protected": "boolean",
      "head_commit": {
        "hash": "string",
        "message": "string",
        "timestamp": "ISO 8601 UTC timestamp"
      },
      "last_commit": {
        "hash": "string",
        "message": "string",
        "timestamp": "ISO 8601 UTC timestamp"
      },
      "commit_count": "integer"
    }
  ]
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Branches retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Repository not found |

**Related Requirements:**
- REQ-SRV-033: Branch List Endpoint

#### 4.4.4. Branch Switch

**Endpoint:** `POST /api/v1/git/branches/switch`

**Description:** Switches the current branch for the repository.

**Authentication:** Required

**Authorization:** `repositories:write` permission required

**Request Body Schema:**

```json
{
  "repository_id": "string",
  "branch": "string",
  "create_commit": "boolean"
}
```

**Field Descriptions:**

| Field | Type | Required | Description | Constraints |
|-------|------|-----------|-------------|-------------|
| `repository_id` | String | Yes | Repository identifier | UUID v4 format |
| `branch` | String | Yes | Branch name to switch to | Maximum: 256 characters |
| `create_commit` | Boolean | No | Create commit before switching | Default: `false` |

**Request Example:**

```http
POST /api/v1/git/branches/switch HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Content-Type: application/json

{
  "repository_id": "550e8400-e29b-41d4-a716-446655440100",
  "branch": "feature/new-api",
  "create_commit": false
}
```

**Response Schema:**

```json
{
  "data": {
    "repository_id": "string",
    "previous_branch": "string",
    "current_branch": "string",
    "head_commit": {
      "hash": "string",
      "message": "string",
      "timestamp": "ISO 8601 UTC timestamp"
    },
    "switched_at": "ISO 8601 UTC timestamp"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Branch switched successfully |
| 400 Bad Request | Invalid request body |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Repository or branch not found |
| 409 Conflict | Uncommitted changes prevent switch |
| 422 Unprocessable Entity | Request body validation failed |

**Related Requirements:**
- REQ-SRV-034: Branch Switch Endpoint

#### 4.4.5. Diff View

**Endpoint:** `GET /api/v1/git/diff`

**Description:** Retrieves diff between two commits or branches.

**Authentication:** Required

**Authorization:** `repositories:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `repository_id` | String | Query | Yes | Repository identifier | UUID v4 format |
| `from` | String | Query | No | Source commit hash or branch | Maximum: 256 characters, Default: HEAD |
| `to` | String | Query | No | Target commit hash or branch | Maximum: 256 characters, Default: working directory |
| `format` | String | Query | No | Diff format | Values: `unified`, `json`, Default: `unified` |
| `context_lines` | Integer | Query | No | Number of context lines | Minimum: 0, Maximum: 10, Default: 3 |

**Request Example:**

```http
GET /api/v1/git/diff?repository_id=550e8400-e29b-41d4-a716-446655440100&from=main&to=feature/new-api&format=unified&context_lines=3 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema (Unified format):**

```json
{
  "data": {
    "repository_id": "string",
    "from": "string",
    "to": "string",
    "format": "string",
    "files": [
      {
        "path": "string",
        "status": "string",
        "additions": "integer",
        "deletions": "integer",
        "diff": "string"
      }
    ],
    "total_additions": "integer",
    "total_deletions": "integer",
    "total_files_changed": "integer"
  }
}
```

**File Status Values:**

| Status | Description |
|--------|-------------|
| `added` | New file added |
| `modified` | Existing file modified |
| `deleted` | Existing file deleted |
| `renamed` | File renamed |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Diff retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Repository, branch, or commit not found |

**Related Requirements:**
- REQ-SRV-035: Diff View Endpoint

### 4.5. Authentication Endpoints

#### 4.5.1. Login

**Endpoint:** `POST /api/v1/auth/login`

**Description:** Authenticates a user and returns an authentication token.

**Authentication:** Not required

**Request Headers:**

| Header | Required | Description |
|--------|-----------|-------------|
| `Content-Type` | Yes | Request media type (must be `application/json`) |

**Request Body Schema:**

```json
{
  "username": "string",
  "password": "string",
  "mfa_code": "string"
}
```

**Field Descriptions:**

| Field | Type | Required | Description | Constraints |
|-------|------|-----------|-------------|-------------|
| `username` | String | Yes | Username or email | Minimum: 3, Maximum: 256 characters |
| `password` | String | Yes | User password | Minimum: 8, Maximum: 128 characters |
| `mfa_code` | String | No | Multi-factor authentication code | 6 digits (if MFA enabled) |

**Request Example:**

```http
POST /api/v1/auth/login HTTP/2
Host: api.tachyon.example.com
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "secure_password_123"
}
```

**Response Schema:**

```json
{
  "data": {
    "access_token": "string",
    "refresh_token": "string",
    "token_type": "string",
    "expires_in": "integer",
    "user": {
      "id": "string",
      "username": "string",
      "email": "string",
      "roles": ["string"],
      "permissions": ["string"]
    }
  }
}
```

**Response Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Content-Type` | `application/json` | Response media type |
| `Set-Cookie` | Session cookie | Session cookie for web clients |
| `Cache-Control` | `no-store, no-cache` | Prevent caching of authentication data |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Authentication successful |
| 400 Bad Request | Invalid request body |
| 401 Unauthorized | Invalid credentials |
| 403 Forbidden | Account locked or disabled |
| 422 Unprocessable Entity | MFA code required or invalid |

**Related Requirements:**
- REQ-SRV-036: Login Endpoint

#### 4.5.2. Logout

**Endpoint:** `POST /api/v1/auth/logout`

**Description:** Invalidates the current authentication token and logs out the user.

**Authentication:** Required

**Request Example:**

```http
POST /api/v1/auth/logout HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
```

**Response Schema:**

```json
{
  "data": {
    "message": "string",
    "logged_out_at": "ISO 8601 UTC timestamp"
  }
}
```

**Response Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Set-Cookie` | Cleared session cookie | Clear session cookie |
| `Cache-Control` | `no-store, no-cache` | Prevent caching of logout response |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Logout successful |
| 401 Unauthorized | Authentication required or failed |

**Related Requirements:**
- REQ-SRV-037: Logout Endpoint

#### 4.5.3. Token Refresh

**Endpoint:** `POST /api/v1/auth/refresh`

**Description:** Refreshes an expired or expiring access token using a refresh token.

**Authentication:** Not required (refresh token used instead)

**Request Headers:**

| Header | Required | Description |
|--------|-----------|-------------|
| `Content-Type` | Yes | Request media type (must be `application/json`) |

**Request Body Schema:**

```json
{
  "refresh_token": "string"
}
```

**Field Descriptions:**

| Field | Type | Required | Description | Constraints |
|-------|------|-----------|-------------|-------------|
| `refresh_token` | String | Yes | Refresh token from login response | JWT format |

**Request Example:**

```http
POST /api/v1/auth/refresh HTTP/2
Host: api.tachyon.example.com
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response Schema:**

```json
{
  "data": {
    "access_token": "string",
    "refresh_token": "string",
    "token_type": "string",
    "expires_in": "integer"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Token refreshed successfully |
| 400 Bad Request | Invalid request body |
| 401 Unauthorized | Invalid or expired refresh token |

**Related Requirements:**
- REQ-SRV-038: Token Refresh Endpoint

#### 4.5.4. MFA Setup

**Endpoint:** `POST /api/v1/auth/mfa/setup`

**Description:** Initiates multi-factor authentication setup for the authenticated user.

**Authentication:** Required

**Request Body Schema:**

```json
{
  "type": "string"
}
```

**Field Descriptions:**

| Field | Type | Required | Description | Constraints |
|-------|------|-----------|-------------|-------------|
| `type` | String | Yes | MFA type | Values: `totp`, `sms`, `email` |

**Request Example:**

```http
POST /api/v1/auth/mfa/setup HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Content-Type: application/json

{
  "type": "totp"
}
```

**Response Schema:**

```json
{
  "data": {
    "secret": "string",
    "qr_code_url": "string",
    "backup_codes": ["string"],
    "type": "string"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | MFA setup initiated successfully |
| 400 Bad Request | Invalid request body |
| 401 Unauthorized | Authentication required or failed |
| 409 Conflict | MFA already enabled |

**Related Requirements:**
- REQ-SRV-039: MFA Setup Endpoint

#### 4.5.5. MFA Verification

**Endpoint:** `POST /api/v1/auth/mfa/verify`

**Description:** Verifies MFA code and completes MFA setup.

**Authentication:** Required

**Request Body Schema:**

```json
{
  "code": "string",
  "type": "string"
}
```

**Field Descriptions:**

| Field | Type | Required | Description | Constraints |
|-------|------|-----------|-------------|-------------|
| `code` | String | Yes | MFA verification code | 6 digits for TOTP |
| `type` | String | Yes | MFA type | Values: `totp`, `sms`, `email` |

**Request Example:**

```http
POST /api/v1/auth/mfa/verify HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Content-Type: application/json

{
  "code": "123456",
  "type": "totp"
}
```

**Response Schema:**

```json
{
  "data": {
    "message": "string",
    "enabled_at": "ISO 8601 UTC timestamp"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | MFA verified and enabled successfully |
| 400 Bad Request | Invalid request body |
| 401 Unauthorized | Authentication required or failed |
| 422 Unprocessable Entity | Invalid MFA code |

**Related Requirements:**
- REQ-SRV-040: MFA Verification Endpoint

### 4.6. User Endpoints

#### 4.6.1. List Users

**Endpoint:** `GET /api/v1/users`

**Description:** Retrieves a list of users accessible to the authenticated user.

**Authentication:** Required

**Authorization:** `users:read` permission required

**Request Parameters:**

| Parameter | Type | Location | Required | Description | Constraints |
|-----------|------|-----------|-----------|-------------|-------------|
| `page` | Integer | Query | No | Page number (1-indexed) | Minimum: 1, Default: 1 |
| `page_size` | Integer | Query | No | Number of users per page | Minimum: 1, Maximum: 100, Default: 20 |
| `search` | String | Query | No | Search query (username or email) | Maximum: 256 characters |
| `role` | String | Query | No | Filter by role | Values: `admin`, `editor`, `viewer` |

**Request Example:**

```http
GET /api/v1/users?page=1&page_size=20&role=editor HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Accept: application/json
```

**Response Schema:**

```json
{
  "data": [
    {
      "id": "string",
      "username": "string",
      "email": "string",
      "full_name": "string",
      "avatar_url": "string",
      "roles": ["string"],
      "created_at": "ISO 8601 UTC timestamp",
      "last_login_at": "ISO 8601 UTC timestamp",
      "is_active": "boolean"
    }
  ],
  "pagination": {
    "page": "integer",
    "page_size": "integer",
    "total_items": "integer",
    "total_pages": "integer",
    "has_next": "boolean",
    "has_previous": "boolean"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Users retrieved successfully |
| 400 Bad Request | Invalid query parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |

#### 4.6.2. Retrieve User

**Endpoint:** `GET /api/v1/users/{id}`

**Description:** Retrieves details of a specific user.

**Authentication:** Required

**Authorization:** `users:read` permission required

**Path Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `id` | String | Yes | Unique user identifier | UUID v4 format |

**Response Schema:**

```json
{
  "data": {
    "id": "string",
    "username": "string",
    "email": "string",
    "full_name": "string",
    "avatar_url": "string",
    "roles": ["string"],
    "permissions": ["string"],
    "created_at": "ISO 8601 UTC timestamp",
    "updated_at": "ISO 8601 UTC timestamp",
    "last_login_at": "ISO 8601 UTC timestamp",
    "is_active": "boolean",
    "mfa_enabled": "boolean",
    "preferences": {
      "theme": "string",
      "language": "string",
      "timezone": "string"
    }
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | User retrieved successfully |
| 400 Bad Request | Invalid path parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | User not found |

#### 4.6.3. Update User

**Endpoint:** `PUT /api/v1/users/{id}`

**Description:** Updates a user's profile information.

**Authentication:** Required

**Authorization:** `users:manage` permission required (or self-update with `users:update`)

**Path Parameters:**

| Parameter | Type | Required | Description | Constraints |
|-----------|------|-----------|-------------|-------------|
| `id` | String | Yes | Unique user identifier | UUID v4 format |

**Request Body Schema:**

```json
{
  "full_name": "string",
  "email": "string",
  "avatar_url": "string",
  "preferences": {
    "theme": "string",
    "language": "string",
    "timezone": "string"
  }
}
```

**Request Example:**

```http
PUT /api/v1/users/550e8400-e29b-41d4-a716-446655440100 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1...
Content-Type: application/json

{
  "full_name": "John Doe",
  "email": "john.doe@example.com",
  "preferences": {
    "theme": "dark",
    "language": "en",
    "timezone": "UTC"
  }
}
```

**Response Schema:**

```json
{
  "data": {
    "id": "string",
    "username": "string",
    "email": "string",
    "full_name": "string",
    "avatar_url": "string",
    "updated_at": "ISO 8601 UTC timestamp",
    "preferences": {
      "theme": "string",
      "language": "string",
      "timezone": "string"
    }
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | User updated successfully |
| 400 Bad Request | Invalid request body or parameters |
| 401 Unauthorized | Authentication required or failed |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | User not found |
| 409 Conflict | Email already in use |
| 422 Unprocessable Entity | Request body validation failed |

### 4.7. Health Check Endpoint

#### 4.7.1. Health Check

**Endpoint:** `GET /health`

**Description:** Provides health status of the server for monitoring and load balancer health checks.

**Authentication:** Not required

**Request Example:**

```http
GET /health HTTP/2
Host: api.tachyon.example.com
```

**Response Schema:**

```json
{
  "status": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "version": "string",
  "components": {
    "database": {
      "status": "string",
      "latency_ms": "float"
    },
    "git_repository": {
      "status": "string",
      "latency_ms": "float"
    },
    "search_index": {
      "status": "string",
      "latency_ms": "float",
      "document_count": "integer"
    },
    "cache": {
      "status": "string",
      "hit_rate": "float",
      "size_mb": "float"
    }
  }
}
```

**Component Status Values:**

| Status | Description |
|--------|-------------|
| `healthy` | Component operating normally |
| `degraded` | Component operating with reduced functionality |
| `unhealthy` | Component not operating |

**Response Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Content-Type` | `application/json` | Response media type |
| `Cache-Control` | `no-cache, no-store` | Prevent caching of health status |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 OK | Server healthy |
| 503 Service Unavailable | Server unhealthy |

**Related Requirements:**
- REQ-SRV-005: Health Check Endpoint

---

## 5. WEBSOCKET ENDPOINTS

### 5.1. WebSocket Connection Management

#### 5.1.1. WebSocket Endpoint

**Endpoint:** `GET /ws` (WebSocket Upgrade)

**Description:** Upgrades an HTTP/2 connection to WebSocket for real-time bidirectional communication.

**Authentication:** Required

**Authorization:** WebSocket connections require authentication via token or session cookie.

**Connection Parameters:**

| Parameter | Type | Location | Description | Constraints |
|-----------|------|-----------|-------------|
| `token` | String | Query | Authentication token | JWT format |
| `session_id` | String | Query | Session cookie identifier | UUID v4 format |

**Connection Upgrade Request Example:**

```http
GET /ws?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... HTTP/1.1
Host: api.tachyon.example.com
Upgrade: websocket
Sec-WebSocket-Key: tachyon-ws-protocol
Sec-WebSocket-Version: 13
Origin: https://tachyon.example.com
```

**Connection Upgrade Response:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Status` | `101 Switching Protocols` | Protocol switch confirmation |
| `Upgrade` | `websocket` | Protocol upgrade to WebSocket |
| `Connection` | `Upgrade` | Connection header indicating upgrade |
| `Sec-WebSocket-Accept` | `tachyon-ws-protocol` | WebSocket protocol acceptance |
| `Sec-WebSocket-Version` | `13` | WebSocket protocol version |

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 101 Switching Protocols | Protocol switch successful |
| 400 Bad Request | Invalid WebSocket upgrade request |
| 401 Unauthorized | Authentication failed |
| 426 Upgrade Required | WebSocket upgrade required |

**Related Requirements:**
- REQ-SRV-091: WebSocket Endpoint
- REQ-SRV-092: Connection Authentication
- REQ-SRV-093: Connection Limiting
- REQ-SRV-094: Heartbeat Mechanism
- REQ-SRV-095: Graceful Disconnection

#### 5.1.2. Connection Lifecycle

**WebSocket Connection States:**

```mermaid
stateDiagram-v2
    [*] Connecting
    [*] Connected
    [*] Authenticated
    [*] Active
    [*] Disconnected
    [*] Error
    [*] Closed

    Connecting --> Connected: Successful connection establishment
    Connected --> Authenticated: Authentication completed
    Authenticated --> Active: Connection ready for messages
    Active --> Disconnected: Connection closed normally
    Active --> Error: Connection error occurred
    Error --> Closed: Connection closed due to error
    Disconnected --> Closed: Connection closed by client
    Closed --> [*]: Connection terminated

    note right of Connecting
    note right of Disconnected
    note right of Closed
```

**Connection State Descriptions:**

| State | Description | Possible Next States |
|-------|-------------|---------------------|
| **Connecting** | Initial connection establishment | Connected, Error, Closed |
| **Connected** | WebSocket connection established | Authenticated, Active |
| **Authenticated** | Authentication completed | Active |
| **Active** | Connection ready for message exchange | Disconnected, Error, Closed |
| **Disconnected** | Connection closed normally | Closed |
| **Error** | Connection error occurred | Closed |
| **Closed** | Connection terminated | Terminated |

#### 5.1.3. Heartbeat Mechanism

**Purpose:** Detect and close dead or idle WebSocket connections to prevent resource leaks.

**Implementation:**

- Server sends periodic heartbeat messages to all connected clients
- Clients must respond to heartbeat messages within timeout period
- Failure to respond results in connection closure

**Heartbeat Message Schema:**

```json
{
  "type": "heartbeat",
  "timestamp": "ISO 8601 UTC timestamp",
  "server_time": "ISO 8601 UTC timestamp"
}
```

**Heartbeat Parameters:**

| Parameter | Value | Description |
|-----------|-------|-------------|
| `interval` | 30 seconds | Heartbeat interval |
| `timeout` | 60 seconds | Maximum time without heartbeat response |

**Heartbeat Flow:**

```mermaid
sequenceDiagram
    participant Server
    participant Client

    Server->Client: Send heartbeat
    Client->Server: Send heartbeat response
    
    Note over Server,Client: Send heartbeat every 30 seconds
    Note over Client,Server: Respond within 60 seconds
    Note right of Server,Client: Timeout or close connection
```

**Related Requirements:**
- REQ-SRV-094: Heartbeat Mechanism

#### 5.1.4. Connection Limiting

**Purpose:** Prevent resource exhaustion and abuse by limiting concurrent WebSocket connections.

**Implementation:**

- Maximum connections per user: Configurable limit (default: 5)
- Maximum total connections: Server-wide limit (configurable)
- Connection rejection when limits exceeded

**Connection Limits:**

| Limit Type | Default Value | Maximum | Description |
|------------|--------------|----------|-------------|
| Per User | 5 | 50 | Maximum WebSocket connections per user |
| Total | 1000 | 10000 | Maximum total WebSocket connections |
| Per IP | 10 | 100 | Maximum WebSocket connections per IP address |

**Connection Rejection Response:**

```json
{
  "error": {
    "code": "CONNECTION_LIMIT_EXCEEDED",
    "message": "Maximum connection limit exceeded",
    "details": {
      "limit_type": "per_user",
      "limit": 5,
      "current_connections": 5
    }
  }
}
```

**Related Requirements:**
- REQ-SRV-093: Connection Limiting
- REQ-SRV-118: Rate Limiting

### 5.2. WebSocket Message Types

#### 5.2.1. Message Format

All WebSocket messages follow a common JSON envelope format:

```json
{
  "type": "string",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": "object",
  "error": "object"
}
```

**Message Envelope Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `type` | String | Yes | Message type identifier | See message types below |
| `id` | String | Yes | Unique message identifier | UUID v4 format |
| `timestamp` | String | Yes | Message timestamp | ISO 8601 UTC format |
| `data` | Object | No | Message payload | Message-specific data |
| `error` | Object | No | Error information | Present only if error occurred |

#### 5.2.2. Message Types

**Document Update Messages**

**Type:** `document_update`

**Description:** Broadcasts when a document is updated.

**Message Schema:**

```json
{
  "type": "document_update",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "document_id": "string",
    "repository_id": "string",
    "title": "string",
    "path": "string",
    "updated_at": "ISO 8601 UTC timestamp",
    "author": {
      "id": "string",
      "username": "string"
    },
    "change_type": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `document_id` | String | Unique document identifier | UUID v4 format |
| `repository_id` | String | Repository identifier | UUID v4 format |
| `title` | String | Document title | Maximum: 200 characters |
| `path` | String | Document file path | Maximum: 512 characters |
| `updated_at` | String | Update timestamp | ISO 8601 UTC format |
| `author.id` | String | Author user identifier | UUID v4 format |
| `author.username` | String | Author username | Maximum: 256 characters |
| `change_type` | String | Type of change | Values: `created`, `updated`, `deleted` |

**User Presence Messages**

**Type:** `user_presence`

**Description:** Broadcasts user presence information (online status, current document).

**Message Schema:**

```json
{
  "type": "user_presence",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "user_id": "string",
    "username": "string",
    "status": "string",
    "current_document_id": "string",
    "current_document_title": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | String | User identifier | UUID v4 format |
| `username` | String | Username | Maximum: 256 characters |
| `status` | String | User status | Values: `online`, `offline`, `away` |
| `current_document_id` | String | Currently viewing document | UUID v4 format (null if none) |
| `current_document_title` | String | Currently viewing document title | Maximum: 200 characters (null if none) |

**Conflict Notification Messages**

**Type:** `conflict_notification`

**Description:** Notifies clients when concurrent edits are detected.

**Message Schema:**

```json
{
  "type": "conflict_notification",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "document_id": "string",
    "conflict_type": "string",
    "conflicting_users": ["string"],
    "resolution_required": "boolean"
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `document_id` | String | Document identifier | UUID v4 format |
| `conflict_type` | String | Type of conflict | Values: `concurrent_edit`, `merge_conflict` |
| `conflicting_users` | Array | List of conflicting users | Array of user identifiers |
| `resolution_required` | Boolean | Manual resolution required | Default: `false` |

**Typing Indicator Messages**

**Type:** `typing_indicator`

**Description:** Broadcasts typing indicators for collaborative editing.

**Message Schema:**

```json
{
  "type": "typing_indicator",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "document_id": "string",
    "user_id": "string",
    "is_typing": "boolean"
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `document_id` | String | Document identifier | UUID v4 format |
| `user_id` | String | User identifier | UUID v4 format |
| `is_typing` | Boolean | User is typing | Default: `false` |

**Cursor Position Messages**

**Type:** `cursor_position`

**Description:** Broadcasts cursor positions for collaborative editing.

**Message Schema:**

```json
{
  "type": "cursor_position",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "document_id": "string",
    "cursors": [
      {
        "user_id": "string",
        "position": {
          "line": "integer",
          "column": "integer"
        }
      }
    ]
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `document_id` | String | Document identifier | UUID v4 format |
| `cursors` | Array | List of cursor positions | Array of cursor objects |
| `cursors[].user_id` | String | User identifier | UUID v4 format |
| `cursors[].position.line` | Integer | Line number (1-indexed) | Minimum: 1 |
| `cursors[].position.column` | Integer | Column number (1-indexed) | Minimum: 1 |

**Error Messages**

**Type:** `error`

**Description:** Reports errors that occur during WebSocket communication.

**Message Schema:**

```json
{
  "type": "error",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "code": "string",
    "message": "string",
    "details": "object"
  }
}
```

**Error Codes:**

| Code | Description |
|------|-------------|
| `UNKNOWN_ERROR` | Unknown error occurred |
| `AUTHENTICATION_FAILED` | WebSocket authentication failed |
| `AUTHORIZATION_FAILED` | WebSocket authorization failed |
| `RATE_LIMIT_EXCEEDED` | Rate limit exceeded |
| `CONNECTION_LIMIT_EXCEEDED` | Connection limit exceeded |
| `INVALID_MESSAGE` | Invalid message format |
| `INTERNAL_ERROR` | Internal server error |

**Subscription Messages**

**Type:** `subscription`

**Description:** Client requests subscription to specific events.

**Message Schema:**

```json
{
  "type": "subscription",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "subscription_id": "string",
    "event_types": ["string"],
    "document_filter": "string",
    "repository_filter": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Description | Constraints |
|-------|------|-----------|-------------|
| `subscription_id` | String | Unique subscription identifier | UUID v4 format |
| `event_types` | Array | Event types to subscribe | Values: `document_update`, `user_presence`, `conflict_notification`, `typing_indicator`, `cursor_position`, `error` |
| `document_filter` | String | Filter by document | UUID v4 format or `*` for all |
| `repository_filter` | String | Filter by repository | UUID v4 format or `*` for all |

**Subscription Response:**

```json
{
  "type": "subscription_acknowledged",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "subscription_id": "string",
    "status": "string"
  }
}
```

**Subscription Status Values:**

| Status | Description |
|--------|-------------|
| `acknowledged` | Subscription request acknowledged |
| `active` | Subscription is active |
| `failed` | Subscription request failed |
| `cancelled` | Subscription was cancelled |

**Unsubscription Messages**

**Type:** `unsubscription`

**Description:** Client cancels an existing subscription.

**Message Schema:**

```json
{
  "type": "unsubscription",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "subscription_id": "string"
  }
}
```

### 5.3. Real-Time Update Scenarios

#### 5.3.1. Document Update Flow

**Scenario:** User A and User B are editing the same document simultaneously.

**Flow:**

```mermaid
sequenceDiagram
    participant UserA
    participant UserB
    participant Server
    participant Database
    participant Git

    UserA->Server: Edit document (User A's version)
    UserB->Server: Edit document (User B's version)
    Server->Database: Save User A's version
    Server->Git: Commit User A's version
    Server->Database: Detect conflict
    Server->UserA: Send conflict notification
    Server->UserB: Send conflict notification
    Server->Git: Merge User B's version
    Server->UserB: Send document update
    
    Note right of Server,UserA: Last-Write-Wins (User A's version accepted)
    Note right of Server,UserB: Conflict notification
    note right of Server,UserB: Document update (merged version)
```

**Message Sequence:**

1. User A sends edit request
2. Server processes User A's edit
3. User B sends edit request
4. Server detects concurrent modification
5. Server sends conflict notification to User A
6. Server sends conflict notification to User B
7. Server merges User B's version
8. Server sends document update to User B

#### 5.3.2. Collaborative Editing Flow

**Scenario:** Multiple users are editing the same document with real-time cursor positions.

**Flow:**

```mermaid
sequenceDiagram
    participant UserA
    participant UserB
    participant UserC
    participant Server

    UserA->Server: Send cursor position (line 10, column 5)
    UserB->Server: Send cursor position (line 10, column 6)
    UserC->Server: Send cursor position (line 10, column 7)
    Server->UserA: Broadcast cursor positions
    Server->UserB: Broadcast cursor positions
    Server->UserC: Broadcast cursor positions
```

**Cursor Position Message Example:**

```json
{
  "type": "cursor_position",
  "id": "550e8400-e29b-41d4-a716-446655440100",
  "timestamp": "2026-02-05T13:30:00.000Z",
  "data": {
    "document_id": "550e8400-e29b-41d4-a716-446655440100",
    "cursors": [
      {
        "user_id": "user-a-id",
        "position": {
          "line": 10,
          "column": 5
        }
      },
      {
        "user_id": "user-b-id",
        "position": {
          "line": 10,
          "column": 6
        }
      },
      {
        "user_id": "user-c-id",
        "position": {
          "line": 10,
          "column": 7
        }
      }
    ]
  }
}
```

#### 5.3.3. Typing Indicator Flow

**Scenario:** User is typing in the document.

**Flow:**

```mermaid
sequenceDiagram
    participant User
    participant Server

    User->Server: Start typing
    Server->User: Broadcast typing indicator
    User->Server: Stop typing
    Server->User: Broadcast typing indicator
```

**Typing Indicator Message Example:**

```json
{
  "type": "typing_indicator",
  "id": "550e8400-e29b-41d4-a716-446655440100",
  "timestamp": "2026-02-05T13:30:05.000Z",
  "data": {
    "document_id": "550e8400-e29b-41d4-a716-446655440100",
    "user_id": "user-a-id",
    "is_typing": true
  }
}
```

### 5.4. WebSocket Security

#### 5.4.1. Authentication

**WebSocket Authentication Methods:**

| Method | Description | Implementation |
|--------|-------------|----------------|
| **Query Parameter** | Token in query string | Used by web clients |
| **Cookie** | Session cookie | Used by web clients |
| **Subprotocol** | Token in WebSocket message | Used for programmatic clients |

**Token-Based Authentication:**

Client includes authentication token in WebSocket upgrade request:

```http
GET /ws?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... HTTP/1.1
Host: api.tachyon.example.com
Upgrade: websocket
```

**Cookie-Based Authentication:**

Client includes session cookie in WebSocket upgrade request:

```http
GET /ws HTTP/1.1
Host: api.tachyon.example.com
Cookie: tachyon_session=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Upgrade: websocket
```

**Message-Based Authentication:**

After connection establishment, client sends authentication message:

```json
{
  "type": "authenticate",
  "id": "auth-001",
  "timestamp": "2026-02-05T13:30:00.000Z",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

**Authentication Response:**

```json
{
  "type": "authenticated",
  "id": "auth-002",
  "timestamp": "2026-02-05T13:30:01.000Z",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440100",
    "permissions": ["documents:read", "documents:write"],
    "roles": ["editor"]
  }
}
```

**Related Requirements:**
- REQ-SRV-076: Session Management
- REQ-SRV-077: MFA Support
- REQ-SRV-081: RBAC Enforcement

#### 5.4.2. Authorization

**Authorization Enforcement:**

WebSocket connections enforce the same authorization model as HTTP endpoints:

- Role-Based Access Control (RBAC) for resource access
- Permission checks for document subscriptions
- Repository access control for repository-level subscriptions

**Authorization Flow:**

```mermaid
graph TD
    WS[WebSocket Connection] -->|Validate Token|
    Validate Token -->|Check Permissions|
    Check Permissions -->|Authorized?|
    Authorized -->|Allow Connection|
    Not Authorized -->|Reject Connection
    
    style WS fill:#e1f5ff
    style ValidateToken fill:#ffcccc
    style CheckPermissions fill:#ffcccc
    style Authorized fill:#ccffcc
    style NotAuthorized fill:#ffcccc
```

**Permission Checks:**

| Permission | Scope | Description |
|------------|-------|-------------|
| `documents:read` | Document read access | Read documents, subscribe to document updates |
| `documents:write` | Document write access | Create, update, delete documents |
| `repositories:read` | Repository read access | Read repositories, subscribe to repository events |
| `repositories:write` | Repository write access | Create, update, delete repositories |
| `users:read` | User read access | Read users, subscribe to user presence |
| `users:manage` | User manage access | Create, update, delete users |

**Related Requirements:**
- REQ-SRV-081: RBAC Enforcement
- REQ-SRV-082: Frontmatter Access Control
- REQ-SRV-083: Block Redaction
- REQ-SRV-084: Principle of Least Privilege

#### 5.4.3. Rate Limiting

**WebSocket Rate Limiting:**

Rate limits are enforced on WebSocket connections to prevent abuse:

| Limit Type | Default Value | Maximum | Description |
|------------|--------------|----------|-------------|
| Messages per second | 10 | 100 | Maximum WebSocket messages per second |
| Connections per minute | 100 | 1000 | Maximum WebSocket connections per minute |
| Subscriptions per user | 5 | 50 | Maximum active subscriptions per user |

**Rate Limit Exceeded Response:**

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": {
      "limit_type": "messages_per_second",
      "limit": 10,
      "retry_after": "ISO 8601 UTC timestamp"
    }
  }
}
```

**Related Requirements:**
- REQ-SRV-118: Rate Limiting
- REQ-SRV-119: Connection Timeouts

#### 5.4.4. DDoS Protection

**WebSocket DDoS Protection:**

Multiple layers of DDoS protection for WebSocket connections:

| Protection Layer | Mechanism | Description |
|----------------|----------|-------------|
| **Connection Limiting** | Per-user and total connection limits | Prevent connection exhaustion |
| **Rate Limiting** | Message rate limits | Prevent message flooding |
| **IP-Based Limiting** | Per-IP connection limits | Prevent distributed attacks |
| **CAPTCHA Challenge** | Challenge-response for suspicious connections | Prevent automated attacks |
| **Connection Throttling** | Gradual connection rejection | Prevent rapid connection storms |

**Connection Throttling Response:**

```json
{
  "error": {
    "code": "CONNECTION_THROTTLED",
    "message": "Connection throttled",
    "details": {
      "retry_after": "ISO 8601 UTC timestamp",
      "reason": "Too many connection attempts"
    }
  }
}
```

**Related Threat Mitigations:**
- Threat: Resource Exhaustion (STRIDE 2.5.1) - Connection limiting
- Threat: Denial of Service (STRIDE 2.5.2) - Rate limiting and DDoS protection
- Threat: Elevation of Privilege (STRIDE 2.6) - Authorization enforcement

### 5.5. WebSocket Error Handling

#### 5.5.1. Error Categories

**WebSocket Error Categories:**

| Category | Description | Examples |
|----------|-------------|----------|
| **Connection Errors** | Failed connection establishment, authentication, authorization | Connection timeout, TLS errors |
| **Protocol Errors** | Invalid WebSocket protocol, message format violations | Protocol version mismatch |
| **Message Errors** | Malformed messages, missing required fields | Invalid message type |
| **Business Logic Errors** | Unauthorized operations, subscription failures | Document not found, permission denied |
| **System Errors** | Internal server errors, database failures | Git operation failures |

#### 5.5.2. Error Message Format

All WebSocket errors follow the common message envelope format with error information:

```json
{
  "type": "error",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "code": "string",
    "message": "string",
    "details": "object",
    "recoverable": "boolean"
  }
}
```

**Error Message Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `code` | String | Yes | Machine-readable error code | See error codes below |
| `message` | String | Yes | Human-readable error message | Maximum: 256 characters |
| `details` | Object | No | Additional error details | Error-specific information |
| `recoverable` | Boolean | No | Error is recoverable | Default: `false` |

**WebSocket Error Codes:**

| Code | Category | Description | Recoverable |
|------|----------|-------------|-------------|
| `CONNECTION_FAILED` | Connection | Connection establishment failed | No |
| `AUTHENTICATION_FAILED` | Connection | Authentication failed | No |
| `AUTHORIZATION_FAILED` | Connection | Authorization failed | No |
| `INVALID_MESSAGE` | Message | Malformed or invalid message | Yes |
| `UNSUPPORTED_MESSAGE_TYPE` | Message | Unknown message type | Yes |
| `SUBSCRIPTION_FAILED` | Business | Subscription request failed | Yes |
| `RATE_LIMIT_EXCEEDED` | System | Rate limit exceeded | Yes |
| `CONNECTION_LIMIT_EXCEEDED` | System | Connection limit exceeded | Yes |
| `INTERNAL_ERROR` | System | Internal server error | Maybe |
| `DATABASE_ERROR` | System | Database operation failed | Maybe |
| `GIT_ERROR` | System | Git operation failed | Maybe |

**Error Recovery:**

For recoverable errors, the error message includes recovery information:

```json
{
  "type": "error",
  "id": "error-001",
  "timestamp": "2026-02-05T13:30:00.000Z",
  "data": {
    "code": "SUBSCRIPTION_FAILED",
    "message": "Subscription failed",
    "details": {
      "reason": "Document not found"
    },
    "recoverable": true,
    "retry_after": "2026-02-05T13:30:05.000Z"
  }
}
```

#### 5.5.3. Connection Closure

**WebSocket Connection Closure:**

Connections may be closed by client or server for various reasons:

| Close Reason | Description | Client-Initiated | Server-Initiated |
|--------------|-------------------|-------------------|
| **Normal Closure** | Client disconnects normally | Yes | No |
| **Error Closure** | Error occurred | Yes | No |
| **Timeout** | No heartbeat response | Yes | No |
| **Rate Limit** | Limit exceeded | Yes | No |
| **Authorization Revoked** | Session invalidated | Yes | No |
| **Server Shutdown** | Server is shutting down | No | Yes |

**Close Frame:**

```json
{
  "type": "close",
  "id": "string",
  "timestamp": "ISO 8601 UTC timestamp",
  "data": {
    "code": "integer",
    "reason": "string",
    "clean": "boolean"
  }
}
```

**Close Codes:**

| Code | Description |
|------|-------------|
| 1000 | Normal Closure | Normal disconnection |
| 1001 | Going Away | Client is going away |
| 1002 | Protocol Error | WebSocket protocol error |
| 1003 | Unsupported Data | Unsupported data type |
| 1004 | Policy Violation | Policy or rate limit violation |
| 1005 | Internal Error | Internal server error |
| 4000-4999 | Application-specific | Application-specific error |

**Related Requirements:**
- REQ-SRV-095: Graceful Disconnection
- REQ-SRV-120: Resource Cleanup

---

## 6. API SECURITY

### 6.1. Authentication

#### 6.1.1. Authentication Methods

The Server API supports multiple authentication methods to accommodate different client types and deployment scenarios.

**Authentication Methods:**

| Method | Description | Use Case | Token Type | Session Support |
|--------|-------------|-----------|-----------|----------------|
| **Bearer Token (JWT)** | Stateless authentication using JSON Web Tokens (JWT) | Programmatic clients, mobile apps, API consumers | JWT | No |
| **Session Cookie** | Cookie-based session management | Web browsers, desktop clients | N/A | Yes |
| **API Key** | Long-lived authentication using API keys | Service accounts, integrations | API Key | No |
| **OAuth 2.0** | Federated authentication via OAuth 2.0 | Third-party login | OAuth Token | No |
| **SAML 2.0** | Enterprise SSO via SAML 2.0 | Enterprise login | SAML Assertion | No |

**Token-Based Authentication:**

**JWT Token Structure:**

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT",
    "kid": "key-id"
  },
  "payload": {
    "iss": "https://api.tachyon.example.com",
    "sub": "user-id",
    "aud": "user-id",
    "exp": "exp-time",
    "iat": "issued-at",
    "nbf": "not-before",
    "jti": "jwt-id"
  },
  "signature": "signature"
}
```

**Token Claims:**

| Claim | Type | Description | Example Value |
|-------|------|-------------|-----------------|
| `iss` | Issuer | API domain identifier | `https://api.tachyon.example.com` |
| `sub` | Subject | User identifier | `550e8400-e29b-41d4-a716-446655440100` |
| `aud` | Audience | User ID | `user-a-id` |
| `exp` | Expiration | Token expiration timestamp | `2026-02-05T14:00:00.000Z` |
| `iat` | Issued At | Token issuance timestamp | `2026-02-05T13:00:00.000Z` |
| `nbf` | Not Before | Token invalid before | `2026-02-05T13:00:00.000Z` |
| `jti` | Key ID | JWT key identifier | `key-2024-02-05` |

**Token Validation:**

1. **Signature Verification:** Verify JWT signature using public key
2. **Algorithm Check:** Validate algorithm is HS256
3. **Issuer Validation:** Verify issuer matches expected domain
4. **Audience Validation:** Verify audience matches user ID
5. **Expiration Check:** Verify token has not expired
6. **Not-Before Check:** Verify token not used before issuance
7. **Key ID Check:** Verify key ID matches expected key

**Token Refresh:**

Access tokens have shorter lifetime than refresh tokens. When an access token expires, clients must use the refresh token to obtain a new access token.

**Token Refresh Flow:**

```mermaid
sequenceDiagram
    participant Client
    participant Server

    Client->Server: Use expired access token
    Server->Client: Return 401 Unauthorized
    Client->Server: Use refresh token
    Server->Client: Return new access token
    
    note over Client,Server: Access token refreshed
    note right of Server,Client: Use new access token
```

**Session-Based Authentication:**

**Session Cookie Structure:**

```
tachyon_session=<session_id>.<signature>.<timestamp>.<hash>
```

| Component | Description | Example Value |
|-----------|-------------|-----------------|
| `session_id` | Unique session identifier | `sess_550e8400-e29b-41d4-a716-446655440100` |
| `signature` | HMAC signature | `a1b2c3d4e5f6...` |
| `timestamp` | Session creation timestamp | `2026-02-05T13:30:00.000Z` |
| `hash` | Session hash | `7f8a9b0c1d2e3...` |

**Session Validation:**

1. **Signature Verification:** Verify HMAC signature using server secret
2. **Timestamp Check:** Verify timestamp is within acceptable window
3. **Hash Check:** Verify hash matches expected value
4. **Expiration Check:** Verify session has not expired

**Session Refresh:**

Sessions are refreshed on each authenticated request to extend session lifetime.

**Multi-Factor Authentication (MFA):**

**MFA Types:**

| Type | Description | Implementation |
|------|-------------|-----------------|
| **TOTP** | Time-based one-time passwords | RFC 6238 | Google Authenticator app |
| **SMS** | SMS-based verification codes | Twilio, AWS SNS | Message delivery service |
| **Email** | Email-based verification codes | SendGrid, AWS SES | Email delivery service |

**MFA Setup Flow:**

```mermaid
sequenceDiagram
    participant User
    participant Server

    User->Server: Initiate MFA setup
    Server->User: Return MFA setup details
    User->Server: Verify MFA code
    Server->User: MFA enabled
    
    note over User,Server: MFA verification successful
    note right of Server,User: MFA now enabled
```

**MFA Verification:**

1. **Code Validation:** Verify TOTP code is 6 digits
2. **Time Window:** Verify code is within time window (30 seconds)
3. **Counter Check:** Verify counter matches expected value
4. **Backup Code Check:** Verify backup code is valid

#### 6.1.2. Authorization

**Authorization Model:**

The Server API implements Role-Based Access Control (RBAC) with principle of least privilege.

**Role Hierarchy:**

| Role | Permissions | Description |
|------|-------------|-----------------|
| **Administrator** | `system:admin` | Full system access, user management, configuration |
| **Editor** | `documents:read`, `documents:write`, `documents:delete` | Full document access (read, write, delete) |
| **Viewer** | `documents:read` | Read-only document access |
| **Contributor** | `documents:read`, `documents:write` | Read and write document access |
| **Read-Only** | `documents:read` | Read-only document access |

**Permission Enforcement:**

Authorization is enforced at multiple layers:

1. **Middleware Layer:** Axum middleware checks permissions before handler execution
2. **Handler Layer:** Handler checks resource-specific permissions
3. **Database Layer:** Database queries enforce row-level access control
4. **Frontmatter Layer:** YAML frontmatter enforces document-level access control

**Frontmatter Access Control:**

Documents can specify access control in YAML frontmatter:

```yaml
---
access: internal
tags: [engineering, confidential]
---
```

**Access Control Levels:**

| Level | Description | Frontmatter Value |
|-------|-------------|-----------------|
| `public` | Document is publicly accessible | `public` |
| `internal` | Document is internal only | `internal` |
| `restricted` | Document is restricted to specific users | `restricted: <user-id>` |
| `confidential` | Document is confidential | `confidential` |

**Access Control Enforcement:**

1. **Frontmatter Parsing:** Extract access control from YAML frontmatter
2. **Role Check:** Verify user has required role
3. **User Check:** Verify user is in allowed list (for restricted/confidential)
4. **Default Deny:** If no access control specified, default to `internal`

**Block Redaction:**

Internal blocks marked with `::: internal` are redacted for unauthorized users:

```markdown
# Public content

This is publicly accessible content.

::: internal

This is internal-only content and will be redacted for unauthorized users.
```

**Related Requirements:**
- REQ-SRV-081: RBAC Enforcement
- REQ-SRV-082: Frontmatter Access Control
- REQ-SRV-083: Block Redaction
- REQ-SRV-084: Principle of Least Privilege

#### 6.1.3. Session Management

**Session Configuration:**

| Setting | Default Value | Description |
|---------|----------------|-----------|
| **Session Timeout** | 1 hour | Maximum session duration before requiring re-authentication |
| **Session Refresh Interval** | 30 minutes | Time between session refreshes |
| **Concurrent Session Limit** | 5 | Maximum concurrent sessions per user |
| **Session Invalidation** | Immediate | Invalidation on logout, password change, role change |

**Session Security:**

1. **HttpOnly Cookies:** All session cookies must be marked `HttpOnly` and `Secure`
2. **SameSite Cookies:** All session cookies must include `SameSite` attribute
3. **Secure Flag:** All session cookies must be marked `Secure`
4. **Signed Cookies:** Session cookies should be signed to prevent tampering

**Session Lifecycle:**

```mermaid
stateDiagram-v2
    [*] Created
    [*] Active
    [*] Refreshed
    [*] Invalidated
    [*] Terminated

    Created --> Active: User authenticates
    Active --> Refreshed: Session is refreshed
    Active --> Invalidated: User logs out or session expires
    Invalidated --> Terminated: Session is terminated
    Terminated --> [*]: Session removed
```

**Related Requirements:**
- REQ-SRV-086: Session Timeout
- REQ-SRV-087: Session Refresh
- REQ-SRV-088: Concurrent Session Limit
- REQ-SRV-089: Session Invalidation
- REQ-SRV-090: Secure Cookies

#### 6.1.4. Rate Limiting

**Purpose:** Prevent abuse and denial-of-service attacks by limiting request rates.

**Rate Limiting Strategy:**

| Limit Type | Default Value | Description |
|------------|--------------|----------|-------------|
| **Per User** | 100 requests/minute | Maximum requests per user |
| **Per IP** | 1000 requests/minute | Maximum requests per IP address |
| **Burst Allowance** | 200 requests/10 seconds | Temporary burst allowance |
| **WebSocket** | 10 messages/second | Maximum WebSocket messages per second |
| **Global** | 10000 requests/second | Global request limit |

**Rate Limiting Headers:**

| Header | Value | Description |
|--------|-------|---------|
| `X-RateLimit-Limit` | Integer | Request limit per window | Remaining requests in window |
| `X-RateLimit-Reset` | Timestamp | Unix timestamp when limit resets |
| `X-RateLimit-Remaining` | Integer | Remaining requests in window | Requests remaining in current window |
| `Retry-After` | Integer | Seconds to wait before retry | 60 seconds |

**Rate Limit Exceeded Response:**

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": {
      "limit_type": "per_user",
      "limit": 100,
      "retry_after": "2026-02-05T13:30:05.000Z"
    }
  }
}
```

**Rate Limiting Algorithms:**

1. **Token Bucket:** Fixed-size token bucket that decrements on each request
2. **Sliding Window:** Time-based window that allows temporary bursts
3. **IP-Based Limiting**: Per-IP limits prevent distributed attacks
4. **Global Limiting**: Server-wide limit prevents overload

**Related Requirements:**
- REQ-SRV-118: Rate Limiting
- REQ-SRV-119: Connection Timeouts

#### 6.1.5. Input Validation

**Purpose:** Prevent injection attacks, data corruption, and malformed requests.

**Validation Layers:**

1. **Type Validation:** Verify request body matches expected schema
2. **Length Validation:** Enforce maximum body sizes
3. **Format Validation:** Verify JSON format is valid
4. **Content Validation:** Sanitize user inputs to prevent XSS and injection

**Input Sanitization:**

1. **HTML Sanitization:** Remove dangerous HTML tags and attributes
2. **SQL Injection Prevention:** Use parameterized queries
3. **Path Traversal Prevention:** Validate file paths are within expected directories
4. **Command Injection Prevention:** Validate command arguments in file operations

**Request Size Limits:**

| Endpoint Type | Default Limit | Maximum Limit | Description |
|-----------|--------------|----------|-------------|
| **Document Creation** | 10 MB | Maximum request body size | 10000000 bytes |
| **Document Update** | 10 MB | Maximum request body size | 10000000 bytes |
| **Search Query** | 256 characters | Maximum search query length | 256 characters |
| **Autocomplete Query** | 100 characters | Maximum autocomplete query length | 100 characters |

**Validation Error Response:**

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": {
      "field": "string",
      "reason": "string"
    }
  }
}
```

**Related Requirements:**
- REQ-SRV-117: Request Size Limits
- REQ-SRV-044: Content Sanitization
- Threat: SQL Injection (STRIDE 2.2.1)
- Threat: XSS (STRIDE 2.4.1)
- Threat: Path Traversal (STRIDE 2.4.2)

#### 6.1.6. CORS Configuration

**Purpose:** Control cross-origin resource sharing and prevent CSRF attacks.

**CORS Policy:**

| Origin Type | Default Policy | Description |
|-----------|--------------|----------|-------------|
| **Same Origin** | Allow | Allow requests from same origin as the page |
| **Explicit Whitelist** | Deny | Allow only specified origins | Deny all others |
| **Subresources** | Allow | Allow requests from subdomains | Allow subdomain requests |
| **Credentials** | Allow | Allow requests with credentials | Allow credential-based requests |
| **Methods** | GET, POST, PUT, DELETE, OPTIONS | Allowed HTTP methods |

**CORS Headers:**

| Header | Value | Description |
|--------|-------|---------|
| `Access-Control-Allow-Origin` | `*` | Allowed origins (or specific origin) |
| `Access-Control-Allow-Methods` | `GET, POST, PUT, DELETE, OPTIONS` | Allowed HTTP methods |
| `Access-Control-Allow-Headers` | `Authorization, Content-Type` | Allowed request headers |
| `Access-Control-Allow-Credentials` | `true` | Allow credentials in requests |
| `Access-Control-Max-Age` | `86400` | Preflight cache duration |
| `Access-Control-Expose-Headers` | `Authorization, Content-Type` | Expose headers to browser |

**CORS Preflight Request:**

```http
OPTIONS /api/v1/documents HTTP/1.1
Host: api.tachyon.example.com
Origin: https://tachyon.example.com
Access-Control-Request-Method: POST
Access-Control-Request-Headers: Content-Type, Authorization
```

**CORS Preflight Response:**

```http
HTTP/1.1 200 OK
Access-Control-Allow-Origin: https://tachyon.example.com
Access-Control-Allow-Methods: POST, PUT, GET, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
Access-Control-Max-Age: 86400
```

**Related Requirements:**
- Threat: CSRF (STRIDE 2.5.2)
- Threat: Cross-Origin Resource Sharing (STRIDE 2.5.1)

#### 6.1.7. TLS Configuration

**Purpose:** Encrypt all network communications to prevent eavesdropping and tampering.

**TLS Configuration:**

| Setting | Default Value | Description |
|---------|----------------|-----------|
| **TLS Version** | 1.3 | Minimum TLS version | TLS 1.3 for encryption |
| **Cipher Suites** | Secure defaults | Strong cipher suite selection | TLS_AES_256_GCM_SHA384, TLS_CHACHA20_POLY1305_SHA256 |
| **Certificate Management** | Automatic | Let's Encrypt or manual certificate management | Automatic via Let's Encrypt or manual loading |
| **HSTS Headers** | Enabled | Strict-Transport-Security headers | Send HSTS headers |

**TLS Cipher Suites:**

| Cipher Suite | Priority | Algorithms |
|-----------|----------|-------------|
| **TLS_AES_256_GCM_SHA384** | High | AES-256 in GCM mode with SHA384 | High security, good performance |
| **TLS_CHACHA20_POLY1305_SHA256** | High | ChaCha20-Poly1305 with SHA256 | High security, good performance |
| **TLS_AES_128_GCM_SHA256** | Medium | AES-128 in GCM mode with SHA256 | Good security, better performance |
| **TLS_AES_256_CBC_SHA256** | Medium | AES-256 in CBC mode with SHA256 | Good security, hardware acceleration |

**Certificate Management:**

**Let's Encrypt Integration:**

Server can automatically obtain and renew certificates from Let's Encrypt for production deployments:

| Setting | Description | Default Value |
|---------|----------------|-----------|
| **Auto-Certificate** | Enabled | Automatic certificate management | `true` | Automatically obtain and renew certificates |
| **Email** | admin@tachyon.example.com | Email for certificate notifications | `null` | Email address for renewal notifications |
| **Staging Environment** | staging.api.tachyon.example.com | Staging domain | `null` | Staging environment domain |

**HSTS Headers:**

| Header | Value | Description |
|--------|-------|---------|
| `Strict-Transport-Security` | `max-age=31536000` | Enforce HTTPS for 1 year | 31536000 seconds (1 year) |
| `Include-SubDomains` | `true` | Include subdomains in HSTS | `true` | Include subdomains in HSTS policy |

**Related Requirements:**
- REQ-SRV-016: TLS 1.3 Support
- REQ-SRV-017: Certificate Management
- REQ-SRV-018: HSTS Headers
- Threat: Man-in-the-Middle (STRIDE 2.1) - TLS eavesdropping
- Threat: Data Tampering (STRIDE 2.2.2) - TLS data tampering

#### 6.1.8. DDoS Protection

**Purpose:** Prevent denial-of-service attacks and resource exhaustion.

**Multi-Layer DDoS Protection:**

| Layer | Mechanism | Description |
|--------|---------|-------------|
| **Network Layer** | Cloudflare/AWS Shield | CDN-based DDoS protection | Cloudflare or AWS Shield integration |
| **Application Layer** | Rate limiting | Request rate limiting | Per-user and per-IP limits |
| **Infrastructure Layer** | Auto-scaling | Horizontal pod autoscaling | Kubernetes HPA autoscaling |
| **Database Layer** | Query optimization | Optimized database queries to prevent overload |

**Rate Limiting:**

| Algorithm | Description | Implementation |
|-----------|-------------|-------------|
| **Token Bucket** | Fixed-size token bucket | Decrements on each request |
| **Sliding Window** | Time-based window with burst allowance | 10-second windows with 200 request limit |
| **Exponential Backoff** | Progressive backoff on rate limit exceeded | Reduce request rate on violations |
| **IP-Based Throttling** | Per-IP limits | 1000 requests/minute per IP | Reduce distributed attacks |

**Connection Limiting:**

| Limit Type | Default Value | Description |
|------------|--------------|----------|-------------|
| **Per User** | 5 | Maximum WebSocket connections per user | Prevent connection exhaustion |
| **Per IP** | 10 | Maximum WebSocket connections per IP | Prevent distributed attacks |
| **Total** | 1000 | 10000 | Maximum total WebSocket connections | Prevent server overload |

**Connection Rejection:**

When limits are exceeded, new connections are rejected with appropriate error response.

**Related Requirements:**
- REQ-SRV-093: Connection Limiting
- REQ-SRV-118: Rate Limiting
- Threat: Resource Exhaustion (STRIDE 2.5.1) - Connection limiting
- Threat: Denial of Service (STRIDE 2.5.2) - DDoS attacks

### 6.2. API Performance

#### 6.2.1. Latency Requirements

**Performance Targets:**

| Operation | Target Latency (P50) | Target Latency (P95) | Description |
|----------|----------------|-------------|-------------|-------------|
| **Document Retrieval (cached)** | 50 ms | 100 ms | Cached document retrieval | Fast document access |
| **Document Retrieval (uncached)** | 100 ms | 200 ms | Uncached document retrieval | Acceptable document retrieval time |
| **Document Creation** | 200 ms | 300 ms | Document creation | Acceptable creation time |
| **Document Update** | 200 ms | 300 ms | Document update | Acceptable update time |
| **Document Deletion** | 200 ms | 300 ms | Document deletion | Acceptable deletion time |
| **Search Query** | 100 ms | 200 ms | Full-text search | Fast search response time |
| **Git Operations** | 500 ms | 1000 ms | Git operations | Acceptable Git operation time |
| **Authentication** | 200 ms | 300 ms | Authentication operations | Acceptable auth time |

**Performance Monitoring:**

All API responses include performance metrics in response headers:

| Header | Value | Description |
|--------|-------|---------|
| `X-Response-Time` | Milliseconds | Server processing time | Request processing duration |
| `X-DB-Query-Time` | Milliseconds | Database query time | Database query duration |
| `X-Git-Operation-Time` | Milliseconds | Git operation time | Git operation duration |
| `X-Cache-Hit` | Boolean | Cache hit status | Whether response was cached |

**Performance Metrics:**

```json
{
  "metrics": {
    "server_time_ms": 45,
    "db_query_time_ms": 12,
    "git_operation_time_ms": 23,
    "cache_hit": true
  }
}
```

#### 6.2.2. Throughput Requirements

**Performance Targets:**

| Operation | Target Throughput | Description |
|----------|----------------|-------------|-------------|
| **HTTP Requests** | 10,000 requests/second | 10K RPS | HTTP request handling |
| **WebSocket Messages** | 5,000 messages/second | 5K MPS | WebSocket message handling |
| **Concurrent Users** | 100 | 100 concurrent users | Support 100 concurrent users |
| **Concurrent Requests** | 1,000 | 1000 concurrent requests | Support 1000 concurrent requests |

**Throughput Monitoring:**

Server monitors throughput metrics to ensure performance targets are met:

```json
{
  "throughput": {
    "requests_per_second": 8500,
    "concurrent_users": 87,
    "concurrent_requests": 923
  }
}
```

#### 6.2.3. Caching Strategies

**Caching Architecture:**

```mermaid
graph LR
    Client[Client] -->|Cache Check
    Cache Check -->|Cache Hit
    Cache Hit -->|Return Response
    Cache Check -->|Cache Miss
    Cache Miss -->|Fetch from Source
    Fetch from Source -->|Populate Cache
    Populate Cache -->|Return Response
```

**Cache Layers:**

| Layer | Cache Type | TTL | Description |
|------|-----------|-----|----------|-------------|
| **In-Memory** | LRU Cache | 5 minutes | Fast in-memory cache | 5 minute expiration |
| **Redis** | Distributed cache | 1 hour | Redis distributed cache | 1 hour expiration |
| **CDN** | Edge cache | 24 hours | CDN edge caching | 24 hour expiration |

**Cache Invalidationation:**

| Event | Trigger | Description |
|--------|-------------|-------------|
| **Document Created** | Invalidate cache entry | New document created |
| **Document Updated** | Invalidate cache entry | Document updated |
| **Document Deleted** | Invalidate cache entry | Document deleted |
| **User Changed** | Invalidate user-specific cache | User permissions changed |
| **Repository Changed** | Invalidate repository cache | Repository metadata changed |

**Cache Control Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `Cache-Control` | `private, max-age=300` | Cache control directive | Prevent caching of sensitive data |
| `ETag` | Content hash | Entity tag for conditional requests | Enable conditional requests |
| `X-Cache-Status` | `HIT, MISS, BYPASS` | Cache status indicator | Indicate cache status |

**Cache Status Values:**

| Status | Description |
|--------|-------------|
| `HIT` | Response served from cache | Cache hit |
| `MISS` | Response served from source | Cache miss |
| `BYPASS` | Response bypassed cache | Cache bypassed (e.g., for POST requests) |
| `STALE` | Cache entry expired | Cache entry is stale |

**Related Requirements:**
- REQ-SRV-042: Cache Management
- REQ-SRV-110: Cache Hit Rate

#### 6.2.4. Database Optimization

**Connection Pooling:**

Server uses connection pooling for SQLite database connections to handle concurrent requests efficiently.

**Pool Configuration:**

| Setting | Default Value | Description |
|---------|----------------|-----------|
| **Min Connections** | 5 | Minimum connections in pool | Minimum connection pool size |
| **Max Connections** | 20 | Maximum connections in pool | Maximum connection pool size |
| **Connection Timeout** | 30 seconds | Connection timeout duration | 30 second timeout |
| **Idle Timeout** | 300 seconds | Idle connection timeout | 5 minute idle timeout |

**Query Optimization:**

| **Indexing Strategy:** Create appropriate indexes for common query patterns

| Index | Purpose | Fields | Description |
|------|-------|---------|-------------|
| `documents_title` | Document title | Fast title-based search | Index on document title |
| `documents_content` | Document content | Full-text search | Index on document content |
| `documents_tags` | Document tags | Tag-based filtering | Index on document tags |
| `documents_author` | Document author | Author-based filtering | Index on document author |
| `documents_created_at` | Created timestamp | Time-based sorting | Index on creation time |
| `documents_updated_at` | Updated timestamp | Time-based sorting | Index on update time |

**Transaction Management:**

All database write operations use transactions to ensure data consistency.

**Transaction Isolation Level:**

| Level | Description |
|--------|-------------|----------------|
| **Read Committed** | Read-only transactions | Isolate read operations | Prevent write conflicts |
| **Write Committed** | Write-only transactions | Isolate write operations | Prevent read conflicts |
| **Serializable** | Serializable transactions | Allow concurrent reads during writes | Allow concurrent reads |

#### 6.2.5. Search Indexing

**Search Engine: Tantivy Integration**

**Index Configuration:**

| Setting | Default Value | Description |
|---------|----------------|-----------|
| **Indexer Type** | Tantivy | Full-text search engine | Tantivy for indexing |
| **Update Strategy** | Incremental | Incremental index updates | Update index incrementally |
| **Commit Interval** | 5 minutes | Commit index changes every 5 minutes | Periodic index commits |

**Index Performance Targets:**

| Metric | Target | Description |
|----------|----------------|-------------|-------------|
| **Index Size** | < 100 MB | Index size limit | Keep index manageable |
| **Query Time (P50)** | 100 ms | 200 ms | Query response time | Fast search response |
| **Index Update Time** | 5 minutes | Index update latency | Fast index updates |

**Index Synchronization:**

Index is synchronized with Git repository state:

```mermaid
graph LR
    Git[Git Repository] -->|Index Update
    Index Update -->|Index Synchronized
    Index Update -->|Search Ready
    
    note right of Git,Index Update: Index synchronized with repository
    note right of Search,Search Ready: Search can use updated index
```

**Related Requirements:**
- REQ-SRV-056: Tantivy Integration
- REQ-SRV-057: Incremental Indexing
- REQ-SRV-058: Index Synchronization

---

## 7. API DOCUMENTATION

### 7.1. OpenAPI Specification

**Purpose:** Provide machine-readable API specification for automated client generation and validation.

**OpenAPI Version:** 3.1.0

**OpenAPI Specification Location:** `https://api.tachyon.example.com/openapi.yaml`

**OpenAPI Specification Structure:**

```yaml
openapi: 3.1.0
info:
  title: Tachyon Server API
  description: RESTful API for Tachyon documentation platform
  version: 1.0.0
  contact:
    name: Tachyon API Support
    email: api-support@tachyon.example.com
    url: https://tachyon.example.com/support
  license:
    name: MIT
    url: https://opensource.org/licenses/MIT

servers:
  - url: https://api.tachyon.example.com/v1
    description: Production server
  - url: https://staging.api.tachyon.example.com/v1
    description: Staging server
  - url: https://dev.api.tachyon.example.com/v1
    description: Development server

security:
  - bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
  - cookieAuth:
      type: apiKey
      in: cookie
      name: tachyon_session

paths:
  /documents:
    get:
      summary: List documents
      operationId: listDocuments
      tags:
        - Documents
      security:
        - bearerAuth
        - cookieAuth
      parameters:
        - name: page
          in: query
          schema:
            type: integer
            minimum: 1
            default: 1
        - name: page_size
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
      responses:
        '200':
          description: Documents retrieved successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: '#/components/schemas/Document'
                  pagination:
                    $ref: '#/components/schemas/Pagination'
        '401':
          description: Authentication required or failed
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '403':
          description: Insufficient permissions
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '422':
          description: Query parameter validation failed
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  /documents/{id}:
    get:
      summary: Retrieve document
      operationId: getDocument
      tags:
        - Documents
      security:
        - bearerAuth
        - cookieAuth
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
        - name: format
          in: query
          schema:
            type: string
            enum: [json, html, markdown]
            default: json
      responses:
        '200':
          description: Document retrieved successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    $ref: '#/components/schemas/DocumentDetail'
        '304':
          description: Document not modified
        '401':
          description: Authentication required or failed
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '403':
          description: Insufficient permissions
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '404':
          description: Document not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

components:
  schemas:
    Document:
      type: object
      properties:
        id:
          type: string
          format: uuid
        title:
          type: string
          maxLength: 200
        path:
          type: string
          maxLength: 512
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time
        author:
          type: object
          properties:
            id:
              type: string
              format: uuid
            username:
              type: string
              maxLength: 256
        tags:
          type: array
          items:
            type: string
            maxLength: 64
          maxItems: 10

    DocumentDetail:
      allOf:
        - $ref: '#/components/schemas/Document'
        - type: object
          properties:
            content:
              type: string
            html:
              type: string
            frontmatter:
              type: object
            metadata:
              type: object
              properties:
                word_count:
                  type: integer
                reading_time_minutes:
                  type: integer
                last_commit_hash:
                  type: string
                branch:
                  type: string

    Pagination:
      type: object
      properties:
        page:
          type: integer
        page_size:
          type: integer
        total_items:
          type: integer
        total_pages:
          type: integer
        has_next:
          type: boolean
        has_previous:
          type: boolean

    Error:
      type: object
      properties:
        error:
          type: object
          properties:
            code:
              type: string
            message:
              type: string
            details:
              type: object
            request_id:
              type: string
              format: uuid
            timestamp:
              type: string
              format: date-time
```

**OpenAPI Documentation Tools:**

| Tool | Purpose | URL |
|------|---------|-----|
| **Swagger UI** | Interactive API documentation | `https://api.tachyon.example.com/swagger-ui` |
| **Redoc** | Beautiful API documentation | `https://api.tachyon.example.com/redoc` |
| **Stoplight Studio** | API testing and validation | `https://api.tachyon.example.com/studio` |

### 7.2. Code Examples

#### 7.2.1. Rust (Axum) Client Example

**Dependencies:**

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

**Client Implementation:**

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    pub id: String,
    pub title: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentListResponse {
    pub data: Vec<Document>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub total_items: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let token = std::env::var("TACHYON_TOKEN")?;
    
    let response = client
        .get("https://api.tachyon.example.com/v1/documents")
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("page", "1"), ("page_size", "20")])
        .send()
        .await?;
    
    let documents: DocumentListResponse = response.json().await?;
    println!("Retrieved {} documents", documents.data.len());
    
    Ok(())
}
```

#### 7.2.2. JavaScript (Fetch API) Client Example

**Client Implementation:**

```javascript
async function listDocuments(accessToken, page = 1, pageSize = 20) {
    const response = await fetch(
        `https://api.tachyon.example.com/v1/documents?page=${page}&page_size=${pageSize}`,
        {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${accessToken}`,
                'Accept': 'application/json',
            },
        }
    );
    
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const data = await response.json();
    console.log(`Retrieved ${data.data.length} documents`);
    return data;
}

// Usage
const accessToken = 'your-access-token';
listDocuments(accessToken, 1, 20)
    .then(documents => console.log(documents))
    .catch(error => console.error(error));
```

#### 7.2.3. Python (Requests) Client Example

**Dependencies:**

```python
import requests
from typing import List, Dict, Any
```

**Client Implementation:**

```python
import requests
from typing import List, Dict, Any

class TachyonClient:
    def __init__(self, base_url: str = "https://api.tachyon.example.com/v1", access_token: str = None):
        self.base_url = base_url
        self.access_token = access_token
        self.session = requests.Session()
    
    def _get_headers(self) -> Dict[str, str]:
        headers = {
            'Accept': 'application/json',
        }
        if self.access_token:
            headers['Authorization'] = f'Bearer {self.access_token}'
        return headers
    
    def list_documents(self, page: int = 1, page_size: int = 20) -> Dict[str, Any]:
        params = {
            'page': page,
            'page_size': page_size,
        }
        response = self.session.get(
            f'{self.base_url}/documents',
            headers=self._get_headers(),
            params=params
        )
        response.raise_for_status()
        return response.json()
    
    def get_document(self, document_id: str, format: str = 'json') -> Dict[str, Any]:
        params = {
            'format': format,
        }
        response = self.session.get(
            f'{self.base_url}/documents/{document_id}',
            headers=self._get_headers(),
            params=params
        )
        response.raise_for_status()
        return response.json()
    
    def create_document(self, title: str, content: str, path: str) -> Dict[str, Any]:
        data = {
            'title': title,
            'content': content,
            'path': path,
        }
        response = self.session.post(
            f'{self.base_url}/documents',
            headers=self._get_headers(),
            json=data
        )
        response.raise_for_status()
        return response.json()

# Usage
client = TachyonClient(access_token='your-access-token')
documents = client.list_documents(page=1, page_size=20)
print(f"Retrieved {len(documents['data'])} documents")
```

#### 7.2.4. Go (net/http) Client Example

**Dependencies:**

```go
import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
)
```

**Client Implementation:**

```go
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
)

type Document struct {
    ID    string `json:"id"`
    Title  string `json:"title"`
    Path   string `json:"path"`
    CreatedAt string `json:"created_at"`
    UpdatedAt string `json:"updated_at"`
}

type DocumentListResponse struct {
    Data       []Document `json:"data"`
    Pagination Pagination `json:"pagination"`
}

type Pagination struct {
    Page         int `json:"page"`
    PageSize      int `json:"page_size"`
    TotalItems   int `json:"total_items"`
    TotalPages   int `json:"total_pages"`
    HasNext       bool `json:"has_next"`
    HasPrevious   bool `json:"has_previous"`
}

type TachyonClient struct {
    BaseURL    string
    AccessToken string
    HTTPClient  *http.Client
}

func NewTachyonClient(baseURL, accessToken string) *TachyonClient {
    return &TachyonClient{
        BaseURL:    baseURL,
        AccessToken: accessToken,
        HTTPClient:  &http.Client{},
    }
}

func (c *TachyonClient) ListDocuments(page, pageSize int) (*DocumentListResponse, error) {
    req, err := http.NewRequest("GET", fmt.Sprintf("%s/v1/documents?page=%d&page_size=%d", c.BaseURL, page, pageSize), nil)
    if err != nil {
        return nil, err
    }
    
    req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", c.AccessToken))
    req.Header.Set("Accept", "application/json")
    
    resp, err := c.HTTPClient.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()
    
    var result DocumentListResponse
    if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
        return nil, err
    }
    
    return &result, nil
}

func main() {
    client := NewTachyonClient("https://api.tachyon.example.com", "your-access-token")
    documents, err := client.ListDocuments(1, 20)
    if err != nil {
        fmt.Printf("Error: %v\n", err)
        return
    }
    
    fmt.Printf("Retrieved %d documents\n", len(documents.Data))
}
```

### 7.3. Usage Examples

#### 7.3.1. Authentication Flow

**Login and Access Token Retrieval:**

```javascript
async function login(username, password) {
    const response = await fetch('https://api.tachyon.example.com/v1/auth/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            username: username,
            password: password,
        }),
    });
    
    if (!response.ok) {
        throw new Error(`Login failed: ${response.status}`);
    }
    
    const data = await response.json();
    const accessToken = data.data.access_token;
    const refreshToken = data.data.refresh_token;
    
    // Store tokens securely (e.g., in localStorage or secure cookie)
    localStorage.setItem('tachyon_access_token', accessToken);
    localStorage.setItem('tachyon_refresh_token', refreshToken);
    
    return data.data;
}

// Usage
login('user@example.com', 'password123')
    .then(userData => console.log('Logged in as:', userData.user.username))
    .catch(error => console.error(error));
```

**Token Refresh:**

```javascript
async function refreshAccessToken(refreshToken) {
    const response = await fetch('https://api.tachyon.example.com/v1/auth/refresh', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            refresh_token: refreshToken,
        }),
    });
    
    if (!response.ok) {
        throw new Error(`Token refresh failed: ${response.status}`);
    }
    
    const data = await response.json();
    const newAccessToken = data.data.access_token;
    const newRefreshToken = data.data.refresh_token;
    
    // Update stored tokens
    localStorage.setItem('tachyon_access_token', newAccessToken);
    localStorage.setItem('tachyon_refresh_token', newRefreshToken);
    
    return data.data;
}

// Usage with automatic token refresh
async function authenticatedFetch(url, options) {
    let accessToken = localStorage.getItem('tachyon_access_token');
    let response = await fetch(url, {
        ...options,
        headers: {
            ...options.headers,
            'Authorization': `Bearer ${accessToken}`,
        },
    });
    
    // If token expired (401), refresh and retry
    if (response.status === 401) {
        const refreshToken = localStorage.getItem('tachyon_refresh_token');
        const tokens = await refreshAccessToken(refreshToken);
        
        // Retry with new token
        return await fetch(url, {
            ...options,
            headers: {
                ...options.headers,
                'Authorization': `Bearer ${tokens.access_token}`,
            },
        });
    }
    
    return response;
}
```

#### 7.3.2. Document CRUD Operations

**Create Document:**

```javascript
async function createDocument(accessToken, documentData) {
    const response = await fetch('https://api.tachyon.example.com/v1/documents', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(documentData),
    });
    
    if (!response.ok) {
        throw new Error(`Document creation failed: ${response.status}`);
    }
    
    const data = await response.json();
    return data.data;
}

// Usage
const documentData = {
    title: 'Introduction to Tachyon',
    content: '# Introduction\n\nTachyon is a modern documentation platform...',
    path: 'docs/introduction.md',
    tags: ['getting-started', 'introduction'],
    frontmatter: {
        description: 'An introduction to Tachyon platform',
        access: 'public',
    },
    commit_message: 'Add introduction document',
};

createDocument(accessToken, documentData)
    .then(document => console.log('Created document:', document.id))
    .catch(error => console.error(error));
```

**Update Document:**

```javascript
async function updateDocument(accessToken, documentId, documentData) {
    const response = await fetch(`https://api.tachyon.example.com/v1/documents/${documentId}`, {
        method: 'PUT',
        headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(documentData),
    });
    
    if (!response.ok) {
        throw new Error(`Document update failed: ${response.status}`);
    }
    
    const data = await response.json();
    return data.data;
}

// Usage
const updatedData = {
    title: 'Introduction to Tachyon (Updated)',
    content: '# Introduction\n\nTachyon is a modern documentation platform with enhanced features...',
    path: 'docs/introduction.md',
    tags: ['getting-started', 'introduction', 'updated'],
    frontmatter: {
        description: 'An updated introduction to Tachyon platform',
        access: 'public',
    },
    commit_message: 'Update introduction document',
};

updateDocument(accessToken, documentId, updatedData)
    .then(document => console.log('Updated document:', document.id))
    .catch(error => console.error(error));
```

**Delete Document:**

```javascript
async function deleteDocument(accessToken, documentId) {
    const response = await fetch(`https://api.tachyon.example.com/v1/documents/${documentId}`, {
        method: 'DELETE',
        headers: {
            'Authorization': `Bearer ${accessToken}`,
        },
    });
    
    if (!response.ok) {
        throw new Error(`Document deletion failed: ${response.status}`);
    }
    
    const data = await response.json();
    return data.data;
}

// Usage
deleteDocument(accessToken, documentId)
    .then(result => console.log('Deleted document:', result.id))
    .catch(error => console.error(error));
```

#### 7.3.3. Search Operations

**Full-Text Search:**

```javascript
async function searchDocuments(accessToken, query, filters = {}) {
    const params = new URLSearchParams({
        q: query,
        ...filters,
    });
    
    const response = await fetch(`https://api.tachyon.example.com/v1/search?${params}`, {
        method: 'GET',
        headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Accept': 'application/json',
        },
    });
    
    if (!response.ok) {
        throw new Error(`Search failed: ${response.status}`);
    }
    
    const data = await response.json();
    return data.data;
}

// Usage
searchDocuments(accessToken, 'authentication', {
    page: 1,
    page_size: 20,
    highlight: true,
})
    .then(results => console.log(`Found ${results.data.length} documents`))
    .catch(error => console.error(error));
```

**Search Autocomplete:**

```javascript
async function searchAutocomplete(accessToken, query) {
    const response = await fetch(`https://api.tachyon.example.com/v1/search/autocomplete?q=${encodeURIComponent(query)}&limit=5`, {
        method: 'GET',
        headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Accept': 'application/json',
        },
    });
    
    if (!response.ok) {
        throw new Error(`Autocomplete failed: ${response.status}`);
    }
    
    const data = await response.json();
    return data.data;
}

// Usage
searchAutocomplete(accessToken, 'auth')
    .then(suggestions => console.log('Suggestions:', suggestions))
    .catch(error => console.error(error));
```

#### 7.3.4. WebSocket Connection

**WebSocket Connection Example:**

```javascript
class TachyonWebSocket {
    constructor(accessToken) {
        this.accessToken = accessToken;
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 5000; // 5 seconds
    }
    
    connect() {
        const wsUrl = `wss://api.tachyon.example.com/ws?token=${this.accessToken}`;
        this.ws = new WebSocket(wsUrl);
        
        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.reconnectAttempts = 0;
            this.subscribeToDocumentUpdates();
        };
        
        this.ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            this.handleMessage(message);
        };
        
        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
        
        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            this.attemptReconnect();
        };
    }
    
    subscribeToDocumentUpdates() {
        const subscription = {
            type: 'subscription',
            id: this.generateId(),
            timestamp: new Date().toISOString(),
            data: {
                subscription_id: this.generateId(),
                event_types: ['document_update', 'user_presence', 'conflict_notification'],
                document_filter: '*',
                repository_filter: '*',
            },
        };
        this.ws.send(JSON.stringify(subscription));
    }
    
    handleMessage(message) {
        switch (message.type) {
            case 'document_update':
                console.log('Document updated:', message.data);
                break;
            case 'user_presence':
                console.log('User presence:', message.data);
                break;
            case 'conflict_notification':
                console.log('Conflict detected:', message.data);
                break;
            case 'heartbeat':
                // Respond to heartbeat
                const heartbeatResponse = {
                    type: 'heartbeat',
                    id: this.generateId(),
                    timestamp: new Date().toISOString(),
                };
                this.ws.send(JSON.stringify(heartbeatResponse));
                break;
            default:
                console.log('Unknown message type:', message.type);
        }
    }
    
    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            console.log(`Reconnecting (attempt ${this.reconnectAttempts})...`);
            setTimeout(() => this.connect(), this.reconnectDelay);
        } else {
            console.error('Max reconnect attempts reached');
        }
    }
    
    disconnect() {
        if (this.ws) {
            this.ws.close();
        }
    }
    
    generateId() {
        return 'msg-' + Math.random().toString(36).substr(2, 9);
    }
}

// Usage
const accessToken = localStorage.getItem('tachyon_access_token');
const wsClient = new TachyonWebSocket(accessToken);
wsClient.connect();

// Disconnect when done
// wsClient.disconnect();
```

**WebSocket Message Handling:**

```javascript
// Handle document updates
function handleDocumentUpdate(message) {
    const documentId = message.data.document_id;
    const title = message.data.title;
    const author = message.data.author.username;
    const changeType = message.data.change_type;
    
    console.log(`Document ${title} (${documentId}) was ${changeType} by ${author}`);
    
    // Update UI or notify user
    updateDocumentInUI(documentId, message.data);
}

// Handle user presence
function handleUserPresence(message) {
    const userId = message.data.user_id;
    const username = message.data.username;
    const status = message.data.status;
    const currentDocument = message.data.current_document_title;
    
    console.log(`User ${username} is ${status}`);
    if (currentDocument) {
        console.log(`User ${username} is viewing: ${currentDocument}`);
    }
    
    // Update user presence in UI
    updateUserPresenceInUI(userId, message.data);
}

// Handle conflict notifications
function handleConflictNotification(message) {
    const documentId = message.data.document_id;
    const conflictType = message.data.conflict_type;
    const conflictingUsers = message.data.conflicting_users;
    
    console.warn(`Conflict detected in document ${documentId}`);
    console.warn(`Conflict type: ${conflictType}`);
    console.warn(`Conflicting users: ${conflictingUsers.join(', ')}`);
    
    // Show conflict notification to user
    showConflictNotification(documentId, conflictType, conflictingUsers);
}
```

---

## 9. REFERENCES

### 9.1. Architectural Decision Records (ADRs)

| ADR ID | Title | Reference | Status |
|---------|------|----------|---------|
| ADR-001 | Rust as Primary Language | [`.specs/02_adrs/001_rust_as_primary_language.md`](../../.specs/02_adrs/001_rust_as_primary_language.md) | Accepted |
| ADR-003 | Axum for HTTP/2 Server | [`.specs/02_adrs/003_axum_for_http2_server.md`](../../.specs/02_adrs/003_axum_for_http2_server.md) | Accepted |
| ADR-007 | Tokio for Async Runtime | [`.specs/02_adrs/007_tokio_for_async_runtime.md`](../../.specs/02_adrs/007_tokio_for_async_runtime.md) | Accepted |

**ADR-001: Rust as Primary Language**

**Decision:** Use Rust as the primary programming language for the Tachyon server component.

**Rationale:**
- Memory safety and thread safety guarantees prevent data races and memory corruption
- Zero-cost abstractions enable high-performance networking and I/O operations
- Strong type system ensures compile-time error detection
- Modern ecosystem with excellent async/await support via Tokio
- Cross-platform compatibility for Linux, macOS, and Windows

**Implications:**
- All server code must be written in Rust 2024 Edition
- Use `unsafe` Rust only when absolutely necessary and with justification
- Leverage Rust's ownership model for memory management
- Use Rust's async/await syntax for asynchronous operations

**ADR-003: Axum for HTTP/2 Server**

**Decision:** Use Axum framework for HTTP/2 server implementation.

**Rationale:**
- Axum provides first-class support for HTTP/2 with multiplexing and header compression
- Tower-based middleware stack enables composable request/response processing
- Type-safe routing and handler registration
- Excellent WebSocket support for real-time communication
- Minimal overhead compared to Actix-web or Rocket
- Strong community support and active development

**Implications:**
- All HTTP/2 endpoints must use Axum router and handlers
- Implement middleware using Axum's Tower-based middleware system
- Use Axum's WebSocket upgrade mechanism for WebSocket connections
- Leverage Axum's state management for request context

**ADR-007: Tokio for Async Runtime**

**Decision:** Use Tokio as the async runtime for Rust.

**Rationale:**
- Tokio provides a mature and performant async runtime for Rust
- Excellent support for async I/O operations (networking, file I/O)
- Built-in timer and task scheduling capabilities
- Zero-cost futures and async/await syntax
- Strong integration with Rust's async ecosystem (tokio-tungstenite for WebSocket)

**Implications:**
- All async operations must use Tokio's async/await syntax
- Use Tokio's runtime for all async tasks
- Leverage Tokio's timer and task scheduling for timeouts and delays
- Use Tokio-tungstenite for WebSocket connections

### 9.2. Requirements

| Requirement ID | Title | Reference | Status |
|--------------|------|----------|---------|
| REQ-SRV-005 | Health Check Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-016 | TLS 1.3 Support | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-017 | Certificate Management | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-018 | HSTS Headers | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-021 | Document List Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-022 | Document Retrieval Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-023 | Document Creation Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-024 | Document Update Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-025 | Document Deletion Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-026 | Full-Text Search Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-027 | Faceted Search Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-028 | Search Autocomplete Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-029 | Search Pagination | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-030 | Search Highlighting | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-031 | Repository Status Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-032 | Commit History Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-033 | Branch List Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-034 | Branch Switch Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-035 | Diff View Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-036 | Login Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-037 | Logout Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-038 | Token Refresh Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-039 | MFA Setup Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-040 | MFA Verification Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-042 | Cache Management | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-044 | Content Sanitization | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-056 | Tantivy Integration | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-057 | Incremental Indexing | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-058 | Index Synchronization | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-076 | Session Management | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-077 | MFA Support | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-081 | RBAC Enforcement | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-082 | Frontmatter Access Control | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-083 | Block Redaction | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-084 | Principle of Least Privilege | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-086 | Session Timeout | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-087 | Session Refresh | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-088 | Concurrent Session Limit | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-089 | Session Invalidation | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-090 | Secure Cookies | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-091 | WebSocket Endpoint | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-092 | Connection Authentication | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-093 | Connection Limiting | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-094 | Heartbeat Mechanism | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-095 | Graceful Disconnection | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-110 | Cache Hit Rate | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-117 | Request Size Limits | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-118 | Rate Limiting | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-119 | Connection Timeouts | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |
| REQ-SRV-120 | Resource Cleanup | [`.specs/04_future_state/reqs/server_requirements.md`](../../.specs/04_future_state/reqs/server_requirements.md) | Defined |

### 9.3. Design Elements

| Design Element ID | Title | Reference | Status |
|-----------------|------|----------|---------|
| SRV-DES-001 | Server Architecture | [`.specs/04_future_state/design/server_design.md`](../../.specs/04_future_state/design/server_design.md) | Defined |
| SRV-DES-002 | HTTP/2 Server Design | [`.specs/04_future_state/design/server_design.md`](../../.specs/04_future_state/design/server_design.md) | Defined |
| SRV-DES-003 | WebSocket Design | [`.specs/04_future_state/design/server_design.md`](../../.specs/04_future_state/design/server_design.md) | Defined |
| SRV-DES-004 | Authentication Design | [`.specs/04_future_state/design/server_design.md`](../../.specs/04_future_state/design/server_design.md) | Defined |
| SRV-DES-005 | Authorization Design | [`.specs/04_future_state/design/server_design.md`](../../.specs/04_future_state/design/server_design.md) | Defined |
| SRV-DES-006 | Cache Design | [`.specs/04_future_state/design/server_design.md`](../../.specs/04_future_state/design/server_design.md) | Defined |
| SRV-DES-007 | Search Design | [`.specs/04_future_state/design/server_design.md`](../../.specs/04_future_state/design/server_design.md) | Defined |

### 9.4. Threat Model

| Threat ID | Title | Reference | Status |
|-----------|------|----------|---------|
| STRIDE 2.1 | Spoofing | [`.specs/03_threat_model/analysis.md`](../../.specs/03_threat_model/analysis.md) | Analyzed |
| STRIDE 2.2 | Tampering | [`.specs/03_threat_model/analysis.md`](../../.specs/03_threat_model/analysis.md) | Analyzed |
| STRIDE 2.3 | Repudiation | [`.specs/03_threat_model/analysis.md`](../../.specs/03_threat_model/analysis.md) | Analyzed |
| STRIDE 2.4 | Information Disclosure | [`.specs/03_threat_model/analysis.md`](../../.specs/03_threat_model/analysis.md) | Analyzed |
| STRIDE 2.5 | Denial of Service | [`.specs/03_threat_model/analysis.md`](../../.specs/03_threat_model/analysis.md) | Analyzed |
| STRIDE 2.6 | Elevation of Privilege | [`.specs/03_threat_model/analysis.md`](../../.specs/03_threat_model/analysis.md) | Analyzed |

**Threat Mitigations in API:**

| Threat | Mitigation | Reference |
|--------|-----------|----------|
| Spoofing | JWT token validation, TLS encryption | Section 6.1.1 |
| Tampering | TLS data encryption, HSTS headers | Section 6.1.7 |
| Repudation | Request signing, audit logging | Section 6.1.1 |
| Information Disclosure | RBAC enforcement, data redaction | Section 6.1.2 |
| Denial of Service | Rate limiting, connection limiting, DDoS protection | Section 6.1.8 |
| Elevation of Privilege | Principle of least privilege, RBAC | Section 6.1.2 |

### 9.5. Standards and Specifications

| Standard | Reference | Description |
|----------|----------|-------------|
| **ISO/IEC 26514:2021** | Systems and software engineering — Requirements for designers and developers of user documentation | Documentation structure and quality |
| **IEEE 830-1998** | IEEE Recommended Practice for Software Requirements Specifications | Requirements specification format |
| **RFC 7540 (HTTP/2)** | Hypertext Transfer Protocol Version 2 | HTTP/2 protocol specification |
| **RFC 8446 (TLS 1.3)** | The Transport Layer Security (TLS) Protocol Version 1.3 | TLS 1.3 protocol specification |
| **RFC 6238 (TOTP)** | TOTP: Time-Based One-Time Passwords | TOTP authentication specification |
| **RFC 6749 (OAuth 2.0)** | The OAuth 2.0 Authorization Framework | OAuth 2.0 specification |
| **RFC 7519 (JSON Web Token)** | JSON Web Token (JWT) | JWT specification |
| **OpenAPI 3.1.0** | OpenAPI Specification | API documentation format |

### 9.6. External Documentation

| Documentation | Reference | Description |
|--------------|----------|-------------|
| **Axum Documentation** | https://docs.rs/axum/latest/axum/index.html | Axum framework documentation |
| **Tokio Documentation** | https://tokio.rs/tokio/tutorial | Tokio async runtime documentation |
| **Tantivy Documentation** | https://docs.rs/tantivy/tantivy/index.html | Tantivy search engine documentation |
| **JWT.io** | https://jwt.io/ | JWT introduction and debugging |
| **WebSocket.org** | https://websocket.org/ | WebSocket protocol specification |
| **OpenAPI Specification** | https://swagger.io/specification/ | OpenAPI specification format |

### 9.7. Change History

| Version | Date | Changes | Author |
|---------|------|---------|---------|
| V1.0 | 2026-02-05 | Initial version | Technical Writer |

---

**Document Control:**

**Document ID:** TACHYON-API-002-V1.0
**Version:** 1.0
**Status:** Proposed
**Classification:** Technical Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 830-1998, RFC 7540 (HTTP/2)
**Dependencies:** [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md), [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md), [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md), [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md), [TACHYON-TMA-V1.0](../../.specs/03_threat_model/analysis.md)

---

**End of Document**

