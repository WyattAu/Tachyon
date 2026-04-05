# TACHYON: DOCUMENT API SPECIFICATION

**Document ID:** TACHYON-API-015-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [API Design Principles](#2-api-design-principles)
3. [Document CRUD API](#3-document-crud-api)
   - [List Documents](#31-list-documents)
   - [Get Document](#32-get-document)
   - [Create Document](#33-create-document)
   - [Update Document](#34-update-document)
   - [Delete Document](#35-delete-document)
4. [Document History API](#4-document-history-api)
5. [Document Restore API](#5-document-restore-api)
6. [Document Security](#6-document-security)
7. [Document Performance](#7-document-performance)
8. [References](#8-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document specifies the Document API for the Tachyon toolchain, providing comprehensive documentation for all endpoints related to document management operations. The Document API enables clients to perform Create, Read, Update, and Delete (CRUD) operations on documents stored in the Git-based repository, with support for versioning, history tracking, and access control.

### 1.2. Document API Definition

The Document API is a RESTful API built on the HTTP/2 protocol using the Axum framework for Rust. It provides programmatic access to document management functionality, enabling integration with desktop applications, web clients, and external services. The API leverages Git for version control, ensuring all document changes are tracked and reversible.

### 1.3. Scope

This document covers:
- Document CRUD operations (Create, Read, Update, Delete)
- Document history and versioning
- Document restoration from previous versions
- Authentication and authorization requirements
- Performance characteristics and caching strategies
- Error handling and status codes

Out of scope:
- Search and query functionality (covered in Search API Specification)
- Git operations beyond document versioning (covered in Git Integration API)
- File upload/download operations (covered in File Transfer API)
- Real-time synchronization (covered in Real-time Synchronization API)

### 1.4. Document Dependencies

This document depends on the following documents:
- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) - Server Application Requirements
- [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) - Rust as Primary Language
- [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md) - Axum for HTTP/2 Server
- [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md) - Tokio for Async Runtime

---

## 2. API DESIGN PRINCIPLES

### 2.1. RESTful Design

The Document API follows RESTful design principles, ensuring predictable and consistent behavior across all endpoints. Resources are identified by URIs, and standard HTTP methods (GET, POST, PUT, DELETE) are used to perform operations on these resources.

**Resource Hierarchy:**

```
/api/documents              # Collection of all documents
/api/documents/{id}         # Specific document by ID
/api/documents/{id}/history # Document history
/api/documents/{id}/restore # Document restoration
```

**HTTP Method Semantics:**

| Method | Operation | Idempotent | Safe | Description |
|--------|-----------|-------------|------|-------------|
| GET    | Retrieve resource | Yes  | Yes  | Retrieve resource representation without modification |
| POST   | Create resource  | No   | No   | Create new resource from request payload |
| PUT    | Update resource  | Yes  | No   | Replace entire resource with request payload |
| DELETE | Delete resource  | Yes  | No   | Delete resource |

### 2.2. HTTP/2 Protocol

The Document API uses the HTTP/2 protocol for all client connections, leveraging multiplexing, header compression, and binary protocol efficiency. HTTP/1.1 fallback is supported for clients that do not support HTTP/2.

**HTTP/2 Benefits:**

1. **Multiplexing:** Multiple concurrent requests over single TCP connection
2. **Header Compression:** HPACK compression reduces header overhead
3. **Server Push:** Proactive resource pushing to reduce latency
4. **Binary Protocol:** More efficient parsing compared to HTTP/1.1 text protocol
5. **Stream Prioritization:** Priority-based request processing

**Performance Impact:**

| Metric | HTTP/1.1 | HTTP/2 | Improvement |
|--------|-----------|---------|-------------|
| Page Load Time (6 resources) | 1200 ms | 400 ms | 67% faster |
| Header Size | 820 bytes | 280 bytes | 66% smaller |
| Connection Overhead | 6 TCP connections | 1 TCP connection | 83% fewer |
| Bandwidth Usage | 1.2 MB | 0.8 MB | 33% less |

### 2.3. Type Safety and Validation

The Document API leverages Rust's type system to provide compile-time guarantees for request validation, response generation, and error handling. All request and response types are strongly typed, preventing entire classes of runtime errors.

**Type-Safe Request Extraction:**

```rust
use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request parameters for listing documents
#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    /// Page number for pagination (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Number of documents per page
    #[serde(default = "default_page_size")]
    pub page_size: u32,

    /// Filter by document title (substring match)
    #[serde(default)]
    pub title_filter: Option<String>,

    /// Filter by tags (comma-separated)
    #[serde(default)]
    pub tags: Option<String>,
}

fn default_page() -> u32 { 1 }
fn default_page_size() -> u32 { 50 }

/// Document metadata for list responses
#[derive(Debug, Serialize)]
pub struct DocumentMetadata {
    /// Unique document identifier
    pub id: Uuid,

    /// Document title
    pub title: String,

    /// Document creation timestamp (ISO 8601)
    pub created_at: String,

    /// Document last modification timestamp (ISO 8601)
    pub updated_at: String,

    /// Document author
    pub author: String,

    /// Document tags
    pub tags: Vec<String>,
}
```

### 2.4. Async/Await Architecture

The Document API uses Tokio's async runtime for efficient I/O operations, enabling non-blocking handling of database queries, file system operations, and Git repository operations. This async architecture is essential for meeting the sub-15 millisecond response time requirement.

**Async Benefits:**

1. **Non-Blocking I/O:** Efficient handling of I/O-bound operations without blocking threads
2. **Concurrent Processing:** Multiple requests processed concurrently on limited threads
3. **Resource Efficiency:** Lower memory and CPU usage compared to thread-per-request models
4. **Scalability:** Efficient scaling to thousands of concurrent connections

### 2.5. Error Handling

The Document API implements comprehensive error handling with proper HTTP status codes and detailed error messages. All errors follow a consistent structure, enabling clients to handle error conditions programmatically.

**Error Response Format:**

```rust
/// Standard error response structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// HTTP status code
    pub status: u16,

    /// Application-specific error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional error details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,
}

/// Additional error details
#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    /// Field that caused the error (for validation errors)
    pub field: Option<String>,

    /// Invalid value (for validation errors)
    pub value: Option<String>,

    /// Constraint that was violated (for validation errors)
    pub constraint: Option<String>,
}
```

**Common Error Codes:**

| HTTP Status | Error Code | Description |
|-------------|-------------|-------------|
| 400 | INVALID_REQUEST | Malformed request body or parameters |
| 401 | UNAUTHORIZED | Missing or invalid authentication credentials |
| 403 | FORBIDDEN | User lacks permission to access resource |
| 404 | NOT_FOUND | Requested resource does not exist |
| 409 | CONFLICT | Resource state conflicts with request (e.g., concurrent modification) |
| 422 | UNPROCESSABLE_ENTITY | Request body validation failed |
| 429 | TOO_MANY_REQUESTS | Rate limit exceeded |
| 500 | INTERNAL_SERVER_ERROR | Unexpected server error |
| 503 | SERVICE_UNAVAILABLE | Service temporarily unavailable |

### 2.6. Pagination

The Document API implements cursor-based pagination for efficient navigation through large document collections. Cursor-based pagination provides better performance than offset-based pagination for large datasets and ensures consistent results even during concurrent modifications.

**Pagination Parameters:**

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| page | u32 | Page number (1-indexed) | 1 |
| page_size | u32 | Documents per page (max: 100) | 50 |

**Pagination Response:**

```rust
/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// Requested page number
    pub page: u32,

    /// Number of documents per page
    pub page_size: u32,

    /// Total number of documents
    pub total: u64,

    /// Total number of pages
    pub total_pages: u64,

    /// Whether there is a next page
    pub has_next: bool,

    /// Whether there is a previous page
    pub has_previous: bool,

    /// Documents for current page
    pub data: Vec<T>,
}
```

### 2.7. Content Negotiation

The Document API supports content negotiation for both request and response bodies, enabling clients to specify preferred data formats. JSON is the default and recommended format, with additional support for YAML and plain text where appropriate.

**Content-Type Headers:**

| Content-Type | Description | Use Cases |
|--------------|-------------|-----------|
| application/json | JSON format (default) | All API operations |
| application/yaml | YAML format | Configuration and metadata operations |
| text/plain | Plain text | Raw document content retrieval |

**Accept Header:**

Clients may specify preferred response format using the `Accept` header:

```
Accept: application/json
Accept: application/yaml
Accept: text/plain
```

### 2.8. Versioning

The Document API uses URL path versioning to enable evolution without breaking existing clients. The current version is v1, and all endpoints are prefixed with `/api/v1`.

**Versioning Strategy:**

```
/api/v1/documents          # Current version (v1)
/api/v2/documents          # Future version (v2)
```

**Version Compatibility Policy:**

- Major version increments indicate breaking changes
- Minor version increments indicate backward-compatible additions
- Patch version increments indicate bug fixes without API changes
- Deprecated endpoints are supported for at least 6 months before removal

---

## 3. DOCUMENT CRUD API

### 3.1. List Documents

**Endpoint:** `GET /api/v1/documents`

**Description:** Retrieve a paginated list of documents accessible to the authenticated user. Supports filtering by title and tags, with sorting options for customization.

**Authentication:** Required

**Authorization:** User must have `documents:read` permission

**Request Parameters:**

| Parameter | Type | Location | Description | Required | Default |
|-----------|------|------------|-------------|-----------|---------|
| page | u32 | Query | Page number (1-indexed) | No | 1 |
| page_size | u32 | Query | Documents per page (max: 100) | No | 50 |
| title_filter | string | Query | Filter by title (substring match) | No | - |
| tags | string | Query | Filter by tags (comma-separated) | No | - |
| sort_by | string | Query | Sort field (created_at, updated_at, title) | No | updated_at |
| sort_order | string | Query | Sort order (asc, desc) | No | desc |

**Request Example:**

```http
GET /api/v1/documents?page=1&page_size=20&tags=api,documentation&sort_by=updated_at&sort_order=desc HTTP/2
Host: api.tachyon.example.com
Accept: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9tZSI6ImFkbWluIiwic2NwIjoxNzY1NjMwMzYzIn0.eyJleHAiOjE1Njg5OTkzLCJpc3QiOiJkb2N1ZW50cyJ9In0
```

**Response Format:**

```rust
/// Response for list documents endpoint
#[derive(Debug, Serialize)]
pub struct ListDocumentsResponse {
    /// Paginated document metadata
    #[serde(flatten)]
    pub pagination: PaginatedResponse<DocumentMetadata>,
}
```

**Response Example:**

```json
{
  "page": 1,
  "page_size": 20,
  "total": 150,
  "total_pages": 8,
  "has_next": true,
  "has_previous": false,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440100",
      "title": "API Reference Guide",
      "created_at": "2026-02-01T10:30:00Z",
      "updated_at": "2026-02-05T14:22:33Z",
      "author": "john.doe@example.com",
      "tags": ["api", "documentation", "reference"]
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440101",
      "title": "Getting Started Tutorial",
      "created_at": "2026-01-28T09:15:00Z",
      "updated_at": "2026-02-04T11:45:12Z",
      "author": "jane.smith@example.com",
      "tags": ["tutorial", "getting-started"]
    }
  ]
}
```

**Status Codes:**

| Status | Description |
|--------|-------------|
| 200 | Success - Documents retrieved successfully |
| 400 | Bad Request - Invalid query parameters |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - User lacks documents:read permission |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error - Unexpected server error |

**Error Response Example:**

```json
{
  "status": 400,
  "code": "INVALID_QUERY_PARAMETER",
  "message": "Invalid page_size parameter: must be between 1 and 100",
  "details": {
    "field": "page_size",
    "value": "150",
    "constraint": "1 <= page_size <= 100"
  }
}
```

**Performance Characteristics:**

- **Target Latency:** P50 < 50ms, P99 < 150ms
- **Cache Hit Rate:** > 80% for frequently accessed lists
- **Concurrency:** Supports 1,000+ concurrent requests
- **Rate Limiting:** 100 requests/minute per user

**Implementation Notes:**

1. Query parameters are validated before processing
2. Pagination is cursor-based for performance
3. Results are cached for 5 minutes to reduce database load
4. Tags filtering uses partial matching for flexibility
5. Title filtering is case-insensitive substring match

**Related Requirements:**
- REQ-SRV-021: Document List endpoint requirement
- REQ-SRV-029: Search Pagination requirement
- REQ-SRV-110: Cache Hit Rate requirement
- REQ-SRV-111: Concurrent Users requirement

### 3.2. Get Document

**Endpoint:** `GET /api/v1/documents/{id}`

**Description:** Retrieve a specific document by its unique identifier. Returns document metadata, content, and version information. Supports content negotiation for different response formats.

**Authentication:** Required

**Authorization:** User must have `documents:read` permission and access to the specific document

**Path Parameters:**

| Parameter | Type | Description | Required | Validation |
|-----------|------|-------------|-----------|------------|
| id | UUID | Unique document identifier | Yes | Must be valid UUID format |

**Request Example:**

```http
GET /api/v1/documents/550e8400-e29b-41d4-a716-446655440100 HTTP/2
Host: api.tachyon.example.com
Accept: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9tZSI6ImFkbWluIiwic2NwIjoxNzY1NjMwMzYzIn0.eyJleHAiOjE1Njg5OTkzLCJpc3QiOiJkb2N1ZW50cyJ9In0
```

**Response Format:**

```rust
/// Complete document response
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    /// Document metadata
    pub metadata: DocumentMetadata,

    /// Document content (Markdown)
    pub content: String,

    /// Document version information
    pub version: DocumentVersion,

    /// Document access control information
    pub access_control: AccessControl,
}

/// Document version information
#[derive(Debug, Serialize)]
pub struct DocumentVersion {
    /// Current version number
    pub version: u32,

    /// Git commit hash
    pub commit_hash: String,

    /// Git branch
    pub branch: String,

    /// Version creation timestamp
    pub created_at: String,
}

/// Access control information
#[derive(Debug, Serialize)]
pub struct AccessControl {
    /// Document visibility (public, private, restricted)
    pub visibility: String,

    /// Required permissions to access document
    pub required_permissions: Vec<String>,

    /// Access control list (if restricted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<String>>,
}
```

**Response Example:**

```json
{
  "metadata": {
    "id": "550e8400-e29b-41d4-a716-446655440100",
    "title": "API Reference Guide",
    "created_at": "2026-02-01T10:30:00Z",
    "updated_at": "2026-02-05T14:22:33Z",
    "author": "john.doe@example.com",
    "tags": ["api", "documentation", "reference"]
  },
  "content": "# API Reference Guide\n\nThis guide provides comprehensive documentation for the Tachyon API...\n\n## Overview\n\nThe API is organized into several categories...",
  "version": {
    "version": 42,
    "commit_hash": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6",
    "branch": "main",
    "created_at": "2026-02-05T14:22:33Z"
  },
  "access_control": {
    "visibility": "public",
    "required_permissions": ["documents:read"],
    "access_list": null
  }
}
```

**Status Codes:**

| Status | Description |
|--------|-------------|
| 200 | Success - Document retrieved successfully |
| 400 | Bad Request - Invalid document ID format |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - User lacks permission to access document |
| 404 | Not Found - Document does not exist |
| 500 | Internal Server Error - Unexpected server error |

**Error Response Example:**

```json
{
  "status": 404,
  "code": "DOCUMENT_NOT_FOUND",
  "message": "Document with ID '550e8400-e29b-41d4-a716-446655440100' does not exist",
  "details": {
    "field": "id",
    "value": "550e8400-e29b-41d4-a716-446655440100",
    "constraint": "Document must exist"
  }
}
```

**Performance Characteristics:**

- **Target Latency:** P50 < 30ms, P99 < 100ms
- **Cache Hit Rate:** > 90% for frequently accessed documents
- **Content Processing:** JIT rendering applied on-demand
- **Storage:** Documents served from Git repository with caching

**Implementation Notes:**

1. Document ID is validated as UUID before database lookup
2. Document content is retrieved from Git repository
3. Frontmatter is parsed for metadata and access control
4. Content is cached for 10 minutes to reduce Git operations
5. Internal blocks (`:::internal`) are filtered based on user permissions

**Related Requirements:**
- REQ-SRV-022: Document Retrieval endpoint requirement
- REQ-SRV-041: JIT Rendering requirement
- REQ-SRV-042: Cache Management requirement
- REQ-SRV-082: Frontmatter Access Control requirement
- REQ-SRV-083: Block Redaction requirement

### 3.3. Create Document

**Endpoint:** `POST /api/v1/documents`

**Description:** Create a new document in the repository. The document content is stored in Git, with automatic versioning and commit generation. Supports optional metadata including title, tags, and access control settings.

**Authentication:** Required

**Authorization:** User must have `documents:create` permission

**Request Body:**

```rust
/// Request body for creating a document
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    /// Document title (required)
    #[serde(length(min = 1, max = 200))]
    pub title: String,

    /// Document content (Markdown)
    #[serde(length(max = 10485760))]
    pub content: String,

    /// Document tags (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Document visibility (optional)
    #[serde(default)]
    pub visibility: DocumentVisibility,

    /// Access control list (optional, for restricted visibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<String>>,

    /// Commit message (optional)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub commit_message: String,
}

/// Document visibility levels
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentVisibility {
    /// Document is visible to all authenticated users
    Public,

    /// Document is visible only to the author
    Private,

    /// Document is visible only to specified users
    Restricted,
}

impl Default for DocumentVisibility {
    fn default() -> Self {
        DocumentVisibility::Private
    }
}
```

**Request Example:**

```http
POST /api/v1/documents HTTP/2
Host: api.tachyon.example.com
Content-Type: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9tZSI6ImFkbWluIiwic2NwIjoxNzY1NjMwMzYzIn0.eyJleHAiOjE1Njg5OTkzLCJpc3QiOiJkb2N1ZW50cyJ9In0

{
  "title": "New API Documentation",
  "content": "# New API Documentation\n\nThis document describes the new API endpoints...\n\n## Overview\n\nThe API provides...",
  "tags": ["api", "documentation", "new"],
  "visibility": "public",
  "commit_message": "Initial commit for new API documentation"
}
```

**Response Format:**

```rust
/// Response for create document endpoint
#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    /// Created document metadata
    #[serde(flatten)]
    pub document: DocumentMetadata,

    /// Git commit information
    pub commit: GitCommitInfo,
}

/// Git commit information
#[derive(Debug, Serialize)]
pub struct GitCommitInfo {
    /// Git commit hash
    pub commit_hash: String,

    /// Git branch
    pub branch: String,

    /// Commit timestamp
    pub committed_at: String,
}
```

**Response Example:**

```json
{
  "document": {
    "id": "770e8400-e29b-41d4-a716-446655440102",
    "title": "New API Documentation",
    "created_at": "2026-02-06T01:00:00Z",
    "updated_at": "2026-02-06T01:00:00Z",
    "author": "john.doe@example.com",
    "tags": ["api", "documentation", "new"]
  },
  "commit": {
    "commit_hash": "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6",
    "branch": "main",
    "committed_at": "2026-02-06T01:00:00Z"
  }
}
```

**Status Codes:**

| Status | Description |
|--------|-------------|
| 201 | Created - Document created successfully |
| 400 | Bad Request - Invalid request body or parameters |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - User lacks documents:create permission |
| 409 | Conflict - Document with same path already exists |
| 422 | Unprocessable Entity - Content validation failed |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error - Unexpected server error |

**Error Response Example:**

```json
{
  "status": 409,
  "code": "DOCUMENT_ALREADY_EXISTS",
  "message": "A document with the same path already exists",
  "details": {
    "field": "path",
    "value": "/api/new-documentation",
    "constraint": "Document path must be unique"
  }
}
```

**Performance Characteristics:**

- **Target Latency:** P50 < 100ms, P99 < 300ms
- **Git Operations:** Automatic commit generation with metadata
- **Content Processing:** JIT rendering on-demand, not during creation
- **Concurrency:** Supports optimistic locking for concurrent creation

**Implementation Notes:**

1. Document title is required and must be 1-200 characters
2. Content is validated as Markdown syntax
3. Tags are optional and limited to 20 per document
4. Git commit is created with configurable message
5. Document path is auto-generated from title (slugified)
6. Frontmatter is automatically generated with metadata
7. Internal blocks (`:::internal`) are filtered based on user permissions

**Related Requirements:**
- REQ-SRV-023: Document Creation endpoint requirement
- REQ-SRV-047: Commit Management requirement
- REQ-SRV-041: JIT Rendering requirement
- REQ-SRV-043: Frontmatter Processing requirement

### 3.4. Update Document

**Endpoint:** `PUT /api/v1/documents/{id}`

**Description:** Update an existing document by its unique identifier. Supports full document replacement or partial updates. All updates create a new Git commit, preserving version history.

**Authentication:** Required

**Authorization:** User must have `documents:write` permission and access to the specific document

**Path Parameters:**

| Parameter | Type | Description | Required | Validation |
|-----------|------|-------------|-----------|------------|
| id | UUID | Unique document identifier | Yes | Must be valid UUID format |

**Request Body:**

```rust
/// Request body for updating a document
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    /// Document title (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Document content (Markdown)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Document tags (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Document visibility (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<DocumentVisibility>,

    /// Access control list (optional, for restricted visibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<String>>,

    /// Commit message (optional)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub commit_message: String,
}
```

**Request Example:**

```http
PUT /api/v1/documents/550e8400-e29b-41d4-a716-446655440100 HTTP/2
Host: api.tachyon.example.com
Content-Type: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9tZSI6ImFkbWluIiwic2NwIjoxNzY1NjMwMzYzIn0.eyJleHAiOjE1Njg5OTkzLCJpc3QiOiJkb2N1ZW50cyJ9In0

{
  "title": "Updated API Reference Guide",
  "content": "# API Reference Guide (Updated)\n\nThis guide provides comprehensive documentation for Tachyon API...\n\n## Overview\n\nThe API is organized into several categories...",
  "tags": ["api", "documentation", "reference", "updated"],
  "commit_message": "Update API reference with new examples"
}
```

**Response Format:**

```rust
/// Response for update document endpoint
#[derive(Debug, Serialize)]
pub struct UpdateDocumentResponse {
    /// Updated document metadata
    #[serde(flatten)]
    pub document: DocumentMetadata,

    /// Git commit information
    pub commit: GitCommitInfo,
}
```

**Response Example:**

```json
{
  "document": {
    "id": "550e8400-e29b-41d4-a716-446655440100",
    "title": "Updated API Reference Guide",
    "created_at": "2026-02-01T10:30:00Z",
    "updated_at": "2026-02-06T02:01:00Z",
    "author": "john.doe@example.com",
    "tags": ["api", "documentation", "reference", "updated"]
  },
  "commit": {
    "commit_hash": "d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6",
    "branch": "main",
    "committed_at": "2026-02-06T02:01:00Z"
  }
}
```

**Status Codes:**

| Status | Description |
|--------|-------------|
| 200 | Success - Document updated successfully |
| 400 | Bad Request - Invalid request body or parameters |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - User lacks documents:write permission or access to document |
| 404 | Not Found - Document does not exist |
| 409 | Conflict - Concurrent modification detected (optimistic locking) |
| 422 | Unprocessable Entity - Content validation failed |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error - Unexpected server error |

**Error Response Example:**

```json
{
  "status": 409,
  "code": "CONCURRENT_MODIFICATION",
  "message": "Document was modified by another user. Please refresh and try again.",
  "details": {
    "field": "document",
    "value": "550e8400-e29b-41d4-a716-446655440100",
    "constraint": "Document must not be modified concurrently"
  }
}
```

**Performance Characteristics:**

- **Target Latency:** P50 < 100ms, P99 < 300ms
- **Git Operations:** Automatic commit with metadata
- **Content Processing:** JIT rendering on-demand after update
- **Concurrency:** Optimistic locking with conflict detection

**Implementation Notes:**

1. At least one field (title or content) must be provided
2. Empty fields in request body preserve existing values
3. Git commit is created with configurable message or default
4. Document version is incremented automatically
5. Optimistic locking detects concurrent modifications
6. Internal blocks (`:::internal`) are filtered based on user permissions

**Related Requirements:**
- REQ-SRV-024: Document Update endpoint requirement
- REQ-SRV-047: Commit Management requirement
- REQ-SRV-049: Merge Operations requirement
- REQ-SRV-041: JIT Rendering requirement

### 3.5. Delete Document

**Endpoint:** `DELETE /api/v1/documents/{id}`

**Description:** Delete a document by its unique identifier. The document is removed from the Git repository with a deletion commit. Document deletion is irreversible unless restored from version history.

**Authentication:** Required

**Authorization:** User must have `documents:delete` permission and access to the specific document

**Path Parameters:**

| Parameter | Type | Description | Required | Validation |
|-----------|------|-------------|-----------|------------|
| id | UUID | Unique document identifier | Yes | Must be valid UUID format |

**Request Example:**

```http
DELETE /api/v1/documents/550e8400-e29b-41d4-a716-446655440100 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9tZSI6ImFkbWluIiwic2NwIjoxNzY1NjMwMzYzIn0.eyJleHAiOjE1Njg5OTkzLCJpc3QiOiJkb2N1ZW50cyJ9In0
```

**Response Format:**

```rust
/// Response for delete document endpoint
#[derive(Debug, Serialize)]
pub struct DeleteDocumentResponse {
    /// Deletion confirmation
    pub success: bool,

    /// Deletion message
    pub message: String,

    /// Git commit information
    pub commit: GitCommitInfo,
}
```

**Response Example:**

```json
{
  "success": true,
  "message": "Document deleted successfully",
  "commit": {
    "commit_hash": "f5e6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6",
    "branch": "main",
    "committed_at": "2026-02-06T02:01:00Z"
  }
}
```

**Status Codes:**

| Status | Description |
|--------|-------------|
| 200 | Success - Document deleted successfully |
| 400 | Bad Request - Invalid document ID format |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - User lacks documents:delete permission or access to document |
| 404 | Not Found - Document does not exist |
| 500 | Internal Server Error - Unexpected server error |

**Error Response Example:**

```json
{
  "status": 404,
  "code": "DOCUMENT_NOT_FOUND",
  "message": "Document with ID '550e8400-e29b-41d4-a716-446655440100' does not exist",
  "details": {
    "field": "id",
    "value": "550e8400-e29b-41d4-a716-446655440100",
    "constraint": "Document must exist"
  }
}
```

**Performance Characteristics:**

- **Target Latency:** P50 < 50ms, P99 < 150ms
- **Git Operations:** Deletion commit created with metadata
- **Cache Invalidation:** Document cache entries invalidated immediately
- **Concurrency:** Supports optimistic locking for concurrent deletion

**Implementation Notes:**

1. Document ID is validated as UUID before deletion
2. Git commit is created with deletion metadata
3. Document is removed from file system after commit
4. Search index is updated to reflect deletion
5. Deletion is logged for audit purposes
6. Internal blocks (`:::internal`) are filtered based on user permissions

**Related Requirements:**
- REQ-SRV-025: Document Deletion endpoint requirement
- REQ-SRV-047: Commit Management requirement
- REQ-SRV-057: Incremental Indexing requirement
- REQ-SRV-085: Access Logging requirement

---

## 4. DOCUMENT HISTORY API

### 4.1. Get Document History

**Endpoint:** `GET /api/v1/documents/{id}/history`

**Description:** Retrieve the complete version history of a specific document. Returns all Git commits associated with the document, including commit metadata and change information.

**Authentication:** Required

**Authorization:** User must have `documents:read` permission and access to the specific document

**Path Parameters:**

| Parameter | Type | Description | Required | Validation |
|-----------|------|-------------|-----------|------------|
| id | UUID | Unique document identifier | Yes | Must be valid UUID format |

**Query Parameters:**

| Parameter | Type | Description | Required | Default |
|-----------|------|-------------|-----------|---------|
| page | u32 | Page number (1-indexed) | No | 1 |
| page_size | u32 | Commits per page (max: 100) | No | 50 |

**Request Example:**

```http
GET /api/v1/documents/550e8400-e29b-41d4-a716-446655440100/history?page=1&page_size=20 HTTP/2
Host: api.tachyon.example.com
Accept: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9tZSI6ImFkbWluIiwic2NwIjoxNzY1NjMwMzYzIn0.eyJleHAiOjE1Njg5OTkzLCJpc3QiOiJkb2N1ZW50cyJ9In0
```

**Response Format:**

```rust
/// Response for document history endpoint
#[derive(Debug, Serialize)]
pub struct DocumentHistoryResponse {
    /// Paginated commit history
    #[serde(flatten)]
    pub pagination: PaginatedResponse<CommitInfo>,
}

/// Git commit information
#[derive(Debug, Serialize)]
pub struct CommitInfo {
    /// Git commit hash
    pub commit_hash: String,

    /// Git commit message (short)
    pub message: String,

    /// Commit author
    pub author: String,

    /// Commit timestamp (ISO 8601)
    pub committed_at: String,

    /// Number of files changed in commit
    pub files_changed: u32,
}
```

**Response Example:**

```json
{
  "page": 1,
  "page_size": 20,
  "total": 42,
  "total_pages": 3,
  "has_next": true,
  "has_previous": false,
  "data": [
    {
      "commit_hash": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6",
      "message": "Update API reference with new examples",
      "author": "john.doe@example.com",
      "committed_at": "2026-02-05T14:22:33Z",
      "files_changed": 1
    },
    {
      "commit_hash": "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7",
      "message": "Fix typo in introduction",
      "author": "jane.smith@example.com",
      "committed_at": "2026-02-04T11:45:12Z",
      "files_changed": 1
    }
  ]
}
```

**Status Codes:**

| Status | Description |
|--------|-------------|
| 200 | Success - Document history retrieved successfully |
| 400 | Bad Request - Invalid document ID or query parameters |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - User lacks documents:read permission or access to document |
| 404 | Not Found - Document does not exist |
| 500 | Internal Server Error - Unexpected server error |

**Performance Characteristics:**

- **Target Latency:** P50 < 100ms, P99 < 300ms
- **Cache Hit Rate:** > 70% for frequently accessed histories
- **Git Operations:** History retrieved from Git log
- **Pagination:** Cursor-based pagination for performance

**Implementation Notes:**

1. Document ID is validated as UUID before Git log lookup
2. Git log is queried for commits affecting the document
3. Commit metadata is extracted from Git log
4. Pagination is cursor-based for performance
5. History is cached for 10 minutes to reduce Git operations

**Related Requirements:**
- REQ-SRV-032: Commit History endpoint requirement
- REQ-SRV-046: Repository Access requirement
- REQ-SRV-110: Cache Hit Rate requirement

### 4.2. Restore Document

**Endpoint:** `POST /api/v1/documents/{id}/restore`

**Description:** Restore a document to a previous version from its history. Creates a new Git commit with the restored content, preserving the complete version history.

**Authentication:** Required

**Authorization:** User must have `documents:write` permission and access to the specific document

**Path Parameters:**

| Parameter | Type | Description | Required | Validation |
|-----------|------|-------------|-----------|------------|
| id | UUID | Unique document identifier | Yes | Must be valid UUID format |

**Request Body:**

```rust
/// Request body for restoring a document
#[derive(Debug, Deserialize)]
pub struct RestoreDocumentRequest {
    /// Git commit hash to restore from
    #[serde(length(min = 1, max = 64))]
    pub commit_hash: String,

    /// Commit message for restore operation (optional)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub commit_message: String,
}
```

**Request Example:**

```http
POST /api/v1/documents/550e8400-e29b-41d4-a716-446655440100/restore HTTP/2
Host: api.tachyon.example.com
Content-Type: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9tZSI6ImFkbWluIiwic2NwIjoxNzY1NjMwMzYzIn0.eyJleHAiOjE1Njg5OTkzLCJpc3QiOiJkb2N1ZW50cyJ9In0

{
  "commit_hash": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6",
  "commit_message": "Restore to version with typo fix"
}
```

**Response Format:**

```rust
/// Response for restore document endpoint
#[derive(Debug, Serialize)]
pub struct RestoreDocumentResponse {
    /// Restored document metadata
    #[serde(flatten)]
    pub document: DocumentMetadata,

    /// Git commit information
    pub commit: GitCommitInfo,

    /// Restored from version
    pub restored_from: String,
}
```

**Response Example:**

```json
{
  "document": {
    "id": "550e8400-e29b-41d4-a716-446655440100",
    "title": "API Reference Guide",
    "created_at": "2026-02-01T10:30:00Z",
    "updated_at": "2026-02-06T02:15:00Z",
    "author": "john.doe@example.com",
    "tags": ["api", "documentation", "reference"]
  },
  "commit": {
    "commit_hash": "f5e6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7",
    "branch": "main",
    "committed_at": "2026-02-06T02:15:00Z"
  },
  "restored_from": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6"
}
```

**Status Codes:**

| Status | Description |
|--------|-------------|
| 200 | Success - Document restored successfully |
| 400 | Bad Request - Invalid request body or parameters |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - User lacks documents:write permission or access to document |
| 404 | Not Found - Document or commit does not exist |
| 409 | Conflict - Concurrent modification detected |
| 422 | Unprocessable Entity - Invalid commit hash format |
| 500 | Internal Server Error - Unexpected server error |

**Error Response Example:**

```json
{
  "status": 404,
  "code": "COMMIT_NOT_FOUND",
  "message": "Commit with hash 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6' does not exist in document history",
  "details": {
    "field": "commit_hash",
    "value": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6",
    "constraint": "Commit must exist in document history"
  }
}
```

**Performance Characteristics:**

- **Target Latency:** P50 < 200ms, P99 < 500ms
- **Git Operations:** Restore commit created with metadata
- **Cache Invalidation:** Document cache invalidated after restore
- **Concurrency:** Supports optimistic locking for concurrent restore

**Implementation Notes:**

1. Document ID and commit hash are validated before restore
2. Git checkout is performed to specified commit
3. Restore commit is created with configurable message
4. Document version is incremented automatically
5. Restore operation is logged for audit purposes
6. Internal blocks (`:::internal`) are preserved based on user permissions

**Related Requirements:**
- REQ-SRV-047: Commit Management requirement
- REQ-SRV-049: Merge Operations requirement
- REQ-SRV-085: Access Logging requirement

---

## 5. DOCUMENT RESTORE API

The Document Restore API functionality is documented in Section 4.2 above. This section provides additional context and implementation details for document restoration operations.

### 5.1. Restore Use Cases

The Document Restore API supports several common use cases:

**Typo Recovery:**
- Accidental deletion of important content
- Incorrect modifications that need to be reverted
- Testing alternative document versions

**Version Comparison:**
- Comparing changes between versions
- Reviewing document evolution over time
- Understanding modification history

**Disaster Recovery:**
- Restoring from backup commits
- Recovering from critical system states
- Rolling back problematic changes

### 5.2. Restore Workflow

The document restore workflow follows this sequence:

1. **Validation:** Document ID and commit hash are validated
2. **Checkout:** Git checkout to specified commit
3. **Restore:** Content is restored to current state
4. **Commit:** New commit created with restore metadata
5. **Cache:** Document cache is invalidated
6. **Index:** Search index is updated with restored content

**Workflow Diagram:**

```mermaid
graph LR
    Request[Restore Request] -->|Validate| Validate[Validate]
    Validate -->|Checkout| Checkout[Git Checkout]
    Checkout -->|Restore| Restore[Restore Content]
    Restore -->|Commit| Commit[Create Restore Commit]
    Commit -->|Cache| Invalidate[Invalidate Cache]
    Commit -->|Index| Update[Update Index]
    Cache --> Response[Restore Response]
    Index --> Response
    Update --> Response
    Validate -->|Error| Error[Error Response]
    Checkout -->|Error| Error
    Restore -->|Error| Error
```

### 5.3. Restore Limitations

The Document Restore API has the following limitations:

**Time-Based Limits:**
- Restores from commits older than 1 year may be restricted
- Number of restores per document may be rate-limited
- Large document restores may timeout after 60 seconds

**Storage Limits:**
- Restored documents count against repository storage quota
- Binary files in documents are limited to 100MB per document
- Total repository size is limited to 10GB

**Permission Limits:**
- Users can only restore documents they have write access to
- System administrators can bypass access checks for recovery operations
- Restoring restricted documents requires elevated permissions

**Related Requirements:**
- REQ-SRV-117: Request Size Limits requirement
- REQ-SRV-116: Memory Limits requirement
- REQ-SRV-081: RBAC Enforcement requirement

## 6. DOCUMENT SECURITY

### 6.1. Authentication

The Document API requires authentication for all operations except public document listing (if configured). Authentication is performed using JWT (JSON Web Tokens) with configurable expiration times.

**Authentication Flow:**

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Auth
    participant Database

    Client->>API: POST /api/v1/auth/login
    API->>Auth: Validate credentials
    Auth->>Database: Verify user
    Database-->>Auth: Return JWT token
    Auth-->>Client: Return JWT token
    Client->>API: GET /api/v1/documents (Authorization: Bearer <token>)
    API->>Database: Validate token
    Database-->>API: Return documents
    API-->>Client: Return documents
```

**JWT Token Structure:**

```rust
/// JWT token payload
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User identifier
    pub sub: String,

    /// Token expiration timestamp
    pub exp: u64,

    /// Token issued at timestamp
    pub iat: u64,

    /// User permissions
    pub permissions: Vec<String>,
}
```

**Token Expiration:**

| Token Type | Default Expiration | Maximum Expiration |
|------------|-------------------|-------------------|
| Access Token | 1 hour | 24 hours |
| Refresh Token | 7 days | 30 days |
| Session Token | 30 days | 90 days |

**Authentication Endpoints:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/auth/login` | POST | User authentication |
| `/api/v1/auth/logout` | POST | User logout |
| `/api/v1/auth/refresh` | POST | Token refresh |
| `/api/v1/auth/verify` | POST | Token verification |

**Related Requirements:**
- REQ-SRV-036: Login endpoint requirement
- REQ-SRV-037: Logout endpoint requirement
- REQ-SRV-038: Token Refresh endpoint requirement
- REQ-SRV-076: Session Management requirement
- REQ-SRV-086: Session Timeout requirement

### 6.2. Authorization

The Document API implements Role-Based Access Control (RBAC) for fine-grained permission management. Permissions are granted to users based on their roles and document-level access control settings.

**Permission Model:**

| Permission | Description | Scope |
|------------|-------------|-------|
| `documents:read` | Read document metadata and content | Document |
| `documents:write` | Create and update documents | Document |
| `documents:delete` | Delete documents | Document |
| `documents:history` | View document version history | Document |
| `documents:restore` | Restore document versions | Document |
| `documents:admin` | Administrative operations | System |

**Role Definitions:**

| Role | Permissions |
|------|-------------|
| Reader | `documents:read` |
| Editor | `documents:read`, `documents:write` |
| Author | `documents:read`, `documents:write`, `documents:delete` |
| Maintainer | `documents:read`, `documents:write`, `documents:history`, `documents:restore` |
| Admin | All permissions |

**Access Control Enforcement:**

Access control is enforced at multiple levels:

1. **API Level:** Route-level permission checks using Axum middleware
2. **Document Level:** Frontmatter-based access control directives
3. **Content Level:** Internal block filtering based on user permissions
4. **Repository Level:** Git repository access control

**Frontmatter Access Control:**

Documents can include access control directives in YAML frontmatter:

```yaml
---
title: API Reference Guide
visibility: restricted
access_list:
  - engineering@tachyon.example.com
  - product@tachyon.example.com
---
```

**Access Control Processing:**

```rust
/// Access control enforcement
pub fn enforce_access_control(
    document: &Document,
    user: &User,
) -> Result<(), AccessError> {
    // Check document visibility
    match document.access_control.visibility {
        DocumentVisibility::Public => Ok(()),
        DocumentVisibility::Private => {
            // User must be the author
            if document.metadata.author == user.id {
                Ok(())
            } else {
                Err(AccessError::Forbidden)
            }
        }
        DocumentVisibility::Restricted => {
            // User must be in access list
            if let Some(access_list) = &document.access_control.access_list {
                if access_list.contains(&user.email) {
                    Ok(())
                } else {
                    Err(AccessError::Forbidden)
                }
            } else {
                Err(AccessError::Forbidden)
            }
        }
    }
}
```

**Internal Block Filtering:**

Internal blocks marked with `:::internal` are automatically filtered based on user permissions:

```rust
/// Filter internal blocks from document content
pub fn filter_internal_blocks(
    content: &str,
    user: &User,
) -> String {
    if user.permissions.contains(&"documents:admin") {
        // Admins see all internal blocks
        return content.to_string();
    } else {
        // Filter out internal blocks
        content
            .split(":::internal")
            .filter(|block| !block.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

**Related Requirements:**
- REQ-SRV-081: RBAC Enforcement requirement
- REQ-SRV-082: Frontmatter Access Control requirement
- REQ-SRV-083: Block Redaction requirement
- REQ-SRV-084: Principle of Least Privilege requirement

## 7. DOCUMENT PERFORMANCE

### 7.1. Latency Requirements

The Document API is designed to meet strict latency requirements for all operations. Performance targets are based on percentiles to ensure consistent user experience across varying load conditions.

**Latency Targets:**

| Operation | P50 Latency | P95 Latency | P99 Latency |
|-----------|--------------|--------------|--------------|
| List Documents | < 50ms | < 100ms | < 150ms |
| Get Document | < 30ms | < 80ms | < 100ms |
| Create Document | < 100ms | < 200ms | < 300ms |
| Update Document | < 100ms | < 200ms | < 300ms |
| Delete Document | < 50ms | < 100ms | < 150ms |
| Get Document History | < 100ms | < 200ms | < 300ms |
| Restore Document | < 200ms | < 400ms | < 500ms |

**Performance Monitoring:**

The API implements comprehensive performance monitoring:

```rust
/// Performance metrics tracking
#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    /// Request duration in milliseconds
    pub duration_ms: u64,

    /// Database query duration in milliseconds
    pub db_duration_ms: u64,

    /// Git operation duration in milliseconds
    pub git_duration_ms: u64,

    /// JIT rendering duration in milliseconds
    pub render_duration_ms: u64,

    /// Request timestamp
    pub timestamp: String,
}
```

**Performance Optimization Strategies:**

1. **Database Connection Pooling:** Reuse database connections for reduced overhead
2. **Query Optimization:** Use indexed queries with proper filters
3. **Caching:** Multi-level caching for frequently accessed data
4. **Async Processing:** Non-blocking I/O for concurrent requests
5. **Git Optimization:** Batch operations where possible
6. **JIT Rendering:** Lazy rendering with result caching

### 7.2. Caching Strategies

The Document API implements a multi-level caching strategy to optimize performance and reduce load on underlying systems.

**Cache Hierarchy:**

| Cache Level | Cache Type | TTL | Size Limit | Hit Rate Target |
|-------------|------------|-----|-----------|---------------|
| L1 | In-Memory (LRU) | 5 minutes | 10,000 entries | > 80% |
| L2 | Redis (optional) | 1 hour | 50,000 entries | > 70% |
| L3 | CDN (optional) | 24 hours | Unlimited | > 60% |

**Cache Key Format:**

```rust
/// Cache key structure
#[derive(Debug, Clone, Hash, Eq)]
pub struct CacheKey {
    /// Cache level
    pub level: CacheLevel,

    /// Document ID
    pub document_id: Uuid,

    /// Query parameters hash
    pub query_hash: String,

    /// User ID (for user-specific caching)
    pub user_id: Option<Uuid>,
}

/// Cache entry
#[derive(Debug, Serialize)]
pub struct CacheEntry {
    /// Cache key
    pub key: CacheKey,

    /// Cached data
    pub data: Vec<u8>,

    /// Creation timestamp
    pub created_at: String,

    /// Last access timestamp
    pub last_accessed_at: String,

    /// Access count
    pub access_count: u64,
}
```

**Cache Invalidation:**

Cache entries are invalidated on:

1. **Document Modification:** Any create, update, or delete operation
2. **Access Control Changes:** Changes to document visibility or permissions
3. **Time-Based Expiration:** TTL expiration for automatic cleanup
4. **Manual Invalidation:** Admin-triggered cache invalidation

**Cache Implementation:**

```rust
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Cache manager
pub struct CacheManager {
    /// In-memory cache (L1)
    l1_cache: RwLock<HashMap<CacheKey, CacheEntry>>,

    /// Redis cache (L2) - optional
    l2_cache: Option<redis::Client>,

    /// Get cached document
    pub async fn get_document(&self, key: CacheKey) -> Option<CacheEntry> {
        // Try L1 cache first
        {
            let cache = self.l1_cache.read().await;
            if let Some(entry) = cache.get(&key) {
                // Update access count and timestamp
                let mut entry = entry.clone();
                entry.last_accessed_at = SystemTime::now()
                    .to_rfc3339()
                    .to_string();
                entry.access_count += 1;
                return Some(entry);
            }
        }

        // Try L2 cache if available
        if let Some(redis) = &self.l2_cache {
            if let Ok(Some(data)) = redis.get(&key).await {
                return Some(CacheEntry {
                    key,
                    data,
                    created_at: SystemTime::now().to_rfc3339().to_string(),
                    last_accessed_at: SystemTime::now().to_rfc3339().to_string(),
                    access_count: 1,
                });
            }
        }

        None
    }

    /// Set cache entry
    pub async fn set_document(&self, key: CacheKey, data: Vec<u8>) {
        let entry = CacheEntry {
            key: key.clone(),
            data,
            created_at: SystemTime::now().to_rfc3339().to_string(),
            last_accessed_at: SystemTime::now().to_rfc3339().to_string(),
            access_count: 1,
        };

        // Set in L1 cache
        let mut cache = self.l1_cache.write().await;
        cache.insert(key, entry);

        // Set in L2 cache if available
        if let Some(redis) = &self.l2_cache {
            let _ = redis.set(&key, &entry).await;
        }
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: CacheKey) {
        // Remove from L1 cache
        let mut cache = self.l1_cache.write().await;
        cache.remove(&key);

        // Remove from L2 cache if available
        if let Some(redis) = &self.l2_cache {
            let _ = redis.del(&key).await;
        }
    }
}
```

**Related Requirements:**
- REQ-SRV-042: Cache Management requirement
- REQ-SRV-106: Document Retrieval latency requirement
- REQ-SRV-107: Search Response latency requirement
- REQ-SRV-108: API Response latency requirement
- REQ-SRV-110: Cache Hit Rate requirement

## 8. REFERENCES

### 8.1. Internal Documents

| Document ID | Title | Location |
|-------------|-------|----------|
| [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) | Coding and Documentation Standards |
| [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) | Server Application Requirements |
| [TACHYON-REQ-DOC-V1.0](../../.specs/04_future_state/reqs/documentation_requirements.md) | Documentation Requirements |
| [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) | Rust as Primary Language |
| [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md) | Axum for HTTP/2 Server |
| [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md) | Tokio for Async Runtime |

### 8.2. External Standards

| Standard | Title | URL | Relevance |
|----------|-------|------|-----------|
| ISO/IEC 26514:2021 | https://iso.org/standard/26514 | Documentation lifecycle |
| IEEE 1063:2001 | https://standards.ieee.org/findstds/1063.html | Software user documentation |
| RFC 7540 | https://httpwg.org/specs/rfc7540 | HTTP/2 protocol |
| RFC 8446 | https://httpwg.org/specs/rfc8446 | TLS 1.3 protocol |
| OAuth 2.0 | https://oauth.net/2/ | Authorization framework |
| OpenAPI 3.0 | https://swagger.io/specification | API specification |

### 8.3. Related Tachyon Documents

| Document ID | Title | Relevance |
|-------------|-------|----------|
| [TACHYON-ARC-V1.0](../../docs/architecture/system_architecture_overview.md) | System Architecture Overview |
| [TACHYON-ARC-V1.0](../../docs/architecture/data_architecture.md) | Data Architecture |
| [TACHYON-ARC-V1.0](../../docs/architecture/deployment_architecture.md) | Deployment Architecture |
| [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) | Security Architecture |

### 8.4. Technology Documentation

| Technology | Documentation URL | Relevance |
|-----------|---------------------|-----------|
| Rust | https://doc.rust-lang.org/book/ | Rust programming language |
| Axum | https://docs.rs/axum/0.7.x/axum/index.html | Axum web framework |
| Tokio | https://tokio.rs/ | Tokio async runtime |
| Serde | https://serde.rs/ | Serialization framework |
| UUID | https://docs.rs/uuid/ | UUID generation and parsing |
| Git2-rs | https://docs.rs/git2-rs/ | Git bindings |

### 8.5. Acronyms and Terminology

| Acronym | Full Form | Definition |
|---------|-------------|----------|
| API | Application Programming Interface | Interface for software components |
| CRUD | Create, Read, Update, Delete | Basic data operations |
| RBAC | Role-Based Access Control | Permission management model |
| JWT | JSON Web Token | Authentication token format |
| TTL | Time To Live | Cache expiration time |
| LRU | Least Recently Used | Cache eviction policy |
| JIT | Just-In-Time | On-demand compilation |
| HTTP/2 | Hypertext Transfer Protocol Version 2 | Modern web protocol |
| TLS | Transport Layer Security | Encrypted communication |
| UUID | Universally Unique Identifier | 128-bit identifier |

### 8.6. Citations

[1] The Rust Project, "The Rust Reference," Online. Available: https://doc.rust-lang.org/reference/. [Accessed: 01-Feb-2026].

[2] Tokio Contributors, "Tokio: Asynchronous runtime for the Rust programming language," Online. Available: https://tokio.rs/. [Accessed: 01-Feb-2026].

[3] Axum Contributors, "Axum: Ergonomic and modular web framework built with Tokio," Online. Available: https://docs.rs/axum/. [Accessed: 01-Feb-2026].

[4] RFC 7540, "Hypertext Transfer Protocol Version 2 (HTTP/2)," IETF, 2015. Available: https://httpwg.org/specs/rfc7540. [Accessed: 01-Feb-2026].

[5] RFC 8446, "The Transport Layer Security (TLS) Protocol Version 1.3," IETF, 2018. Available: https://httpwg.org/specs/rfc8446. [Accessed: 01-Feb-2026].

[6] OAuth 2.0 Framework, "The OAuth 2.0 Authorization Framework," IETF, 2012. Available: https://oauth.net/2/. [Accessed: 01-Feb-2026].

[7] OpenAPI Specification, "OpenAPI Specification," OpenAPI Initiative, 2020. Available: https://swagger.io/specification/. [Accessed: 01-Feb-2026].

[8] ISO/IEC 26514:2021, "Systems and Software Engineering - Requirements for Designers and Developers of User Documentation," ISO/IEC, 2021. Available: https://iso.org/standard/26514. [Accessed: 01-Feb-2026].

[9] IEEE 1063-2001, "Standard for Software User Documentation," IEEE, 2001. Available: https://standards.ieee.org/findstds/1063.html. [Accessed: 01-Feb-2026].

---

## DOCUMENT CHANGE HISTORY

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| V1.0 | 2026-02-06 | Technical Writer | Initial version - Document API specification |
| | | | | |
| | | | | |

---

**Document Status:** Proposed

**Review Status:** Pending Review

**Approval Status:** Pending Approval

---

**Document ID:** TACHYON-API-015-V1.0
**Classification:** API Specification
**Compliance:** ISO/IEC 26514:2021, IEEE 1063:2001
