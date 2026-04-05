# TACHYON: API DESIGN GUIDE

**Document ID:** TACHYON-DEV-002-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Technical Documentation & Developer Guide
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [API Design Framework](#2-api-design-framework)
3. [REST API Design](#3-rest-api-design)
4. [WebSocket API Design](#4-websocket-api-design)
5. [IPC API Design](#5-ipc-api-design)
6. [Error Handling Design](#6-error-handling-design)
7. [Security Design](#7-security-design)
8. [Versioning Design](#8-versioning-design)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Purpose and Scope

This document establishes the comprehensive API design framework governing all application programming interfaces (APIs) within the Tachyon toolchain. The guide provides rigorous, PhD thesis-level specifications for designing REST APIs, WebSocket APIs, and Inter-Process Communication (IPC) APIs, ensuring consistency, maintainability, and compliance with established standards.

The Tachyon toolchain encompasses multiple API surfaces:
- **REST APIs:** HTTP/2-based server APIs exposed via Axum framework
- **WebSocket APIs:** Real-time bidirectional communication for live updates
- **IPC APIs:** Tauri-based communication between desktop frontend and Rust backend

### 1.2. Applicability

This guide applies to:
1. All REST API endpoints exposed by the HTTP/2 server component
2. All WebSocket channels and message protocols
3. All IPC commands and events in the Tauri desktop application
4. All API documentation and specifications
5. All API client libraries and SDKs

### 1.3. Design Principles

The Tachyon API design framework is founded on the following core principles:

#### 1.3.1. Type Safety

All APIs must enforce type safety at compile time where possible, and at runtime for dynamic languages. Rust APIs leverage the type system for compile-time guarantees, while TypeScript APIs provide static type checking for client code.

**Formal Property:** API contracts must be expressible in formal type systems and enforceable by tooling.

#### 1.3.2. Idempotency

State-modifying operations should be designed for idempotency where appropriate. Idempotent operations can be safely retried without unintended side effects.

**Formal Property:** For idempotent operation `f`, `f(x) = f(f(x))` for all valid inputs `x`.

#### 1.3.3. Orthogonality

API endpoints should be orthogonal—each endpoint should perform a single, well-defined operation. This principle reduces coupling and increases composability.

**Formal Property:** API operations should have minimal overlapping functionality and clear separation of concerns.

#### 1.3.4. Consistency

API design patterns must be consistent across all API surfaces. Naming conventions, error handling, response formats, and authentication mechanisms should follow established patterns.

**Formal Property:** API contracts should exhibit structural and behavioral consistency across all endpoints.

#### 1.3.5. Performance

APIs must meet performance requirements defined in the system architecture. REST APIs should respond within specified latency bounds, and WebSocket APIs should maintain low message latency.

**Formal Property:** API response time `t_response` must satisfy `t_response ≤ t_max` for all defined operations.

---

## 2. API DESIGN FRAMEWORK

### 2.1. API Surface Taxonomy

The Tachyon system exposes three distinct API surfaces, each serving different communication patterns:

#### 2.1.1. REST API Surface

**Purpose:** Synchronous request-response communication for CRUD operations and administrative functions.

**Protocol:** HTTP/2 over TLS 1.3

**Framework:** Axum web framework (Rust)

**Characteristics:**
- Stateless request-response model
- Resource-oriented URI design
- Standard HTTP methods (GET, POST, PUT, DELETE, PATCH)
- JSON request/response payloads
- OpenAPI 3.1 specification

#### 2.1.2. WebSocket API Surface

**Purpose:** Real-time bidirectional communication for live updates and collaborative features.

**Protocol:** WebSocket over TLS 1.3 (wss://)

**Framework:** tokio-tungstenite (Rust)

**Characteristics:**
- Persistent bidirectional connection
- Event-driven message protocol
- JSON message format
- Subscription-based event channels
- Automatic reconnection handling

#### 2.1.3. IPC API Surface

**Purpose:** Secure communication between Tauri frontend (WebView) and Rust backend in desktop application.

**Protocol:** Tauri IPC mechanism

**Framework:** Tauri command and event systems

**Characteristics:**
- Type-safe command definitions
- Request-response and event-based patterns
- Capability-based access control
- serde serialization for type safety
- Session-based authentication

### 2.2. API Contract Specification

All API contracts must be formally specified using appropriate specification languages:

#### 2.2.1. REST API Specification

REST APIs must be documented using OpenAPI 3.1 specification. The specification must include:
- Endpoint paths and HTTP methods
- Request and response schemas
- Authentication and authorization requirements
- Error response formats
- Rate limiting constraints

**Example Specification Structure:**

```yaml
openapi: 3.1.0
info:
  title: Tachyon REST API
  version: 1.0.0
paths:
  /api/v1/documents:
    get:
      summary: List documents
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: '#/components/schemas/Document'
```

#### 2.2.2. WebSocket API Specification

WebSocket APIs must be documented using AsyncAPI 2.6 specification. The specification must include:
- Channel definitions and message types
- Message schemas for all directions
- Authentication requirements
- Reconnection strategies
- Error handling protocols

#### 2.2.3. IPC API Specification

IPC APIs must be documented using inline Rust documentation with type definitions. The specification must include:
- Command function signatures
- Request and response types
- Capability requirements
- Error types and conditions

### 2.3. API Design Lifecycle

API design follows a structured lifecycle ensuring quality and consistency:

#### 2.3.1. Requirements Analysis

**Input:** Functional requirements, use cases, performance requirements

**Activities:**
- Identify API consumers and their requirements
- Define resource models and operations
- Specify performance and security constraints
- Document integration scenarios

**Deliverable:** API Requirements Document

#### 2.3.2. API Design

**Input:** API Requirements Document

**Activities:**
- Design resource hierarchy and URI structure
- Define request/response schemas
- Specify authentication and authorization
- Design error handling strategy
- Create API specification (OpenAPI/AsyncAPI)

**Deliverable:** API Specification Document

#### 2.3.3. API Review

**Input:** API Specification Document

**Activities:**
- Technical review for correctness and completeness
- Security review for vulnerabilities
- Usability review for developer experience
- Compliance review against standards

**Deliverable:** Approved API Specification

#### 2.3.4. API Implementation

**Input:** Approved API Specification

**Activities:**
- Implement API endpoints/handlers
- Implement authentication and authorization
- Implement error handling
- Write unit and integration tests
- Generate API documentation

**Deliverable:** Implemented API with tests

#### 2.3.5. API Testing

**Input:** Implemented API

**Activities:**
- Unit testing of individual endpoints
- Integration testing of API workflows
- Performance testing for latency and throughput
- Security testing for vulnerabilities
- Contract testing against specification

**Deliverable:** Test results and coverage report

#### 2.3.6. API Documentation

**Input:** Implemented API and API Specification

**Activities:**
- Generate interactive API documentation
- Write usage examples and tutorials
- Document authentication and authorization
- Document error conditions and recovery

**Deliverable:** Published API Documentation

### 2.4. API Quality Metrics

API quality is measured using the following metrics:

#### 2.4.1. Functional Correctness

- **Metric:** Test coverage percentage
- **Target:** ≥ 90% line coverage for critical paths
- **Measurement:** Code coverage analysis

#### 2.4.2. Performance

- **Metric:** API response latency (p50, p95, p99)
- **Target:** p95 latency ≤ 100ms for most operations
- **Measurement:** Performance monitoring and profiling

#### 2.4.3. Reliability

- **Metric:** API uptime and error rate
- **Target:** ≥ 99.9% uptime, ≤ 0.1% error rate
- **Measurement:** Monitoring and alerting

#### 2.4.4. Security

- **Metric:** Security vulnerabilities per endpoint
- **Target:** Zero critical/high severity vulnerabilities
- **Measurement:** Security scanning and penetration testing

#### 2.4.5. Usability

- **Metric:** Developer satisfaction and time-to-first-call
- **Target:** ≥ 80% satisfaction, ≤ 15 minutes to first successful call
- **Measurement:** Developer surveys and onboarding metrics

---

## 3. REST API DESIGN

### 3.1. REST Principles

The Tachyon REST API adheres to the Representational State Transfer (REST) architectural style, as defined by Fielding's dissertation and subsequent RFC standards. The API design follows these core principles:

#### 3.1.1. Resource-Oriented Architecture

Resources are the fundamental abstraction in the Tachyon REST API. Each resource represents a distinct entity in the system (documents, users, projects, etc.) and is identified by a unique URI.

**Formal Properties:**
- **Resource Identification:** Each resource is uniquely identified by a URI
- **Resource Representation:** Resources are represented in JSON format
- **Resource State:** Resources have state that can be retrieved and modified
- **Resource Links:** Resources may contain links to related resources

**URI Structure:**

```
/api/v1/{resource}/{identifier}
```

**Examples:**
- `/api/v1/documents` - Collection of documents
- `/api/v1/documents/{document_id}` - Specific document
- `/api/v1/users/{user_id}/documents` - Documents belonging to a user

#### 3.1.2. Uniform Interface

The API uses a uniform interface with standard HTTP methods to perform operations on resources.

| HTTP Method | Operation | Idempotent | Safe | Description |
|-------------|-----------|-------------|-------|-------------|
| GET | Read | Yes | Yes | Retrieve a resource representation |
| POST | Create | No | No | Create a new resource |
| PUT | Replace | Yes | No | Replace a resource entirely |
| PATCH | Update | No | No | Partially update a resource |
| DELETE | Delete | Yes | No | Delete a resource |

**Formal Definitions:**
- **Idempotent:** Multiple identical requests have the same effect as a single request
- **Safe:** Request does not modify server state

#### 3.1.3. Stateless Communication

Each HTTP request contains all information necessary to understand and process the request. The server does not maintain client context between requests.

**Formal Property:** For any request `r`, the response `response(r)` depends only on `r` and server state `S`, not on previous requests.

**Implementation:**
- Authentication credentials included in each request (via Authorization header)
- Session state maintained client-side (via tokens)
- No server-side session storage required

#### 3.1.4. Cacheability

Responses must explicitly indicate their caching behavior using HTTP cache control headers.

**Cache Control Headers:**
- `Cache-Control: public, max-age=3600` - Publicly cacheable for 1 hour
- `Cache-Control: private, no-cache` - Not cacheable
- `ETag: "abc123"` - Entity tag for conditional requests

### 3.2. URI Design

#### 3.2.1. URI Structure Rules

URIs must follow these structural rules:

**Rule 1: Use Nouns, Not Verbs**
```
✓ /api/v1/documents
✗ /api/v1/getDocuments
```

**Rule 2: Use Plural Nouns for Collections**
```
✓ /api/v1/documents
✗ /api/v1/document
```

**Rule 3: Use Hierarchical Structure for Relationships**
```
✓ /api/v1/users/{user_id}/documents
✗ /api/v1/documents?user_id={user_id}
```

**Rule 4: Use Query Parameters for Filtering, Sorting, and Pagination**
```
✓ /api/v1/documents?status=published&sort=-created_at&page=2&limit=50
✗ /api/v1/documents/published/sort/created_at/desc/page/2/limit/50
```

**Rule 5: Use kebab-case for Path Segments**
```
✓ /api/v1/workspaces/{workspace_id}
✗ /api/v1/workSpaces/{workspace_id}
```

#### 3.2.2. URI Versioning

API versioning is handled via URI path prefixing.

**Version Format:**
```
/api/v{major_version}
```

**Versioning Strategy:**
- **Major Version:** Breaking changes require new major version
- **Minor Version:** Non-breaking additions do not require version change
- **Patch Version:** Bug fixes do not require version change

**Examples:**
- `/api/v1/documents` - Version 1 API
- `/api/v2/documents` - Version 2 API (breaking changes from v1)

### 3.3. HTTP Method Usage

#### 3.3.1. GET Method

**Purpose:** Retrieve a resource representation without modifying server state.

**Request:**
```http
GET /api/v1/documents/{document_id} HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
Accept: application/json
```

**Success Response (200 OK):**
```http
HTTP/2 200 OK
Content-Type: application/json
ETag: "abc123"
Cache-Control: public, max-age=3600

{
  "data": {
    "id": "doc_123",
    "title": "Example Document",
    "content": "Document content...",
    "created_at": "2026-02-06T16:00:00Z",
    "updated_at": "2026-02-06T16:30:00Z"
  }
}
```

**Error Responses:**
- `404 Not Found` - Resource does not exist
- `403 Forbidden` - Client lacks permission to access resource
- `401 Unauthorized` - Authentication required or failed

#### 3.3.2. POST Method

**Purpose:** Create a new resource.

**Request:**
```http
POST /api/v1/documents HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "title": "New Document",
  "content": "Document content..."
}
```

**Success Response (201 Created):**
```http
HTTP/2 201 Created
Content-Type: application/json
Location: /api/v1/documents/doc_456

{
  "data": {
    "id": "doc_456",
    "title": "New Document",
    "content": "Document content...",
    "created_at": "2026-02-06T16:45:00Z",
    "updated_at": "2026-02-06T16:45:00Z"
  }
}
```

**Error Responses:**
- `400 Bad Request` - Invalid request body
- `409 Conflict` - Resource already exists
- `422 Unprocessable Entity` - Validation errors

#### 3.3.3. PUT Method

**Purpose:** Replace an entire resource with a new representation.

**Request:**
```http
PUT /api/v1/documents/{document_id} HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "id": "doc_123",
  "title": "Updated Document",
  "content": "Updated content..."
}
```

**Success Response (200 OK):**
```http
HTTP/2 200 OK
Content-Type: application/json

{
  "data": {
    "id": "doc_123",
    "title": "Updated Document",
    "content": "Updated content...",
    "created_at": "2026-02-06T16:00:00Z",
    "updated_at": "2026-02-06T17:00:00Z"
  }
}
```

**Error Responses:**
- `400 Bad Request` - Invalid request body
- `404 Not Found` - Resource does not exist
- `412 Precondition Failed` - Conditional request failed

#### 3.3.4. PATCH Method

**Purpose:** Partially update a resource.

**Request:**
```http
PATCH /api/v1/documents/{document_id} HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "title": "Partially Updated Document"
}
```

**Success Response (200 OK):**
```http
HTTP/2 200 OK
Content-Type: application/json

{
  "data": {
    "id": "doc_123",
    "title": "Partially Updated Document",
    "content": "Original content...",
    "created_at": "2026-02-06T16:00:00Z",
    "updated_at": "2026-02-06T17:05:00Z"
  }
}
```

**Error Responses:**
- `400 Bad Request` - Invalid request body
- `404 Not Found` - Resource does not exist
- `422 Unprocessable Entity` - Validation errors

#### 3.3.5. DELETE Method

**Purpose:** Delete a resource.

**Request:**
```http
DELETE /api/v1/documents/{document_id} HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
```

**Success Response (204 No Content):**
```http
HTTP/2 204 No Content
```

**Error Responses:**
- `404 Not Found` - Resource does not exist
- `403 Forbidden` - Client lacks permission to delete resource

### 3.4. Request and Response Formats

#### 3.4.1. Request Format

**Content-Type:** `application/json`

**Request Body Structure:**
```json
{
  "data": {
    "field1": "value1",
    "field2": "value2"
  }
}
```

**Naming Conventions:**
- **Property Names:** snake_case (e.g., `created_at`, `user_id`)
- **Enum Values:** SCREAMING_SNAKE_CASE (e.g., `DOCUMENT_STATUS_PUBLISHED`)
- **Boolean Values:** lowercase `true`/`false`
- **Null Values:** JSON `null`

#### 3.4.2. Response Format

**Content-Type:** `application/json`

**Success Response Structure:**
```json
{
  "data": {
    "id": "resource_id",
    "field1": "value1",
    "field2": "value2"
  },
  "meta": {
    "timestamp": "2026-02-06T17:00:00Z",
    "request_id": "req_abc123"
  }
}
```

**Collection Response Structure:**
```json
{
  "data": [
    {
      "id": "resource_1",
      "field1": "value1"
    },
    {
      "id": "resource_2",
      "field1": "value2"
    }
  ],
  "meta": {
    "total": 100,
    "page": 1,
    "limit": 50,
    "total_pages": 2
  }
}
```

**Error Response Structure:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed for field 'title'",
    "details": [
      {
        "field": "title",
        "message": "Title is required"
      }
    ]
  },
  "meta": {
    "timestamp": "2026-02-06T17:00:00Z",
    "request_id": "req_abc123"
  }
}
```

### 3.5. Filtering, Sorting, and Pagination

#### 3.5.1. Filtering

Filtering is performed using query parameters.

**Syntax:**
```
?{field}={value}
```

**Examples:**
- `/api/v1/documents?status=published` - Filter by status
- `/api/v1/documents?created_at.gte=2026-01-01` - Filter by date range
- `/api/v1/documents?author_id=user_123&status=published` - Multiple filters

**Supported Operators:**
- `{field}` - Exact match
- `{field}.ne` - Not equal
- `{field}.gt` - Greater than
- `{field}.gte` - Greater than or equal
- `{field}.lt` - Less than
- `{field}.lte` - Less than or equal
- `{field}.like` - Pattern match (SQL LIKE)

#### 3.5.2. Sorting

Sorting is performed using the `sort` query parameter.

**Syntax:**
```
?sort={field}          # Ascending
?sort=-{field}         # Descending
?sort={field1},{field2}  # Multiple fields
```

**Examples:**
- `/api/v1/documents?sort=created_at` - Sort by created_at ascending
- `/api/v1/documents?sort=-updated_at` - Sort by updated_at descending
- `/api/v1/documents?sort=status,-created_at` - Sort by status ascending, then created_at descending

#### 3.5.3. Pagination

Pagination is performed using `page` and `limit` query parameters.

**Syntax:**
```
?page={page_number}&limit={page_size}
```

**Parameters:**
- `page` - Page number (1-indexed, default: 1)
- `limit` - Number of items per page (default: 50, maximum: 200)

**Examples:**
- `/api/v1/documents?page=1&limit=50` - First page, 50 items
- `/api/v1/documents?page=2&limit=100` - Second page, 100 items

**Response Metadata:**
```json
{
  "meta": {
    "total": 250,
    "page": 2,
    "limit": 50,
    "total_pages": 5
  }
}
```

### 3.6. Conditional Requests

Conditional requests allow clients to avoid transferring data that has not changed.

#### 3.6.1. ETag-Based Conditional Requests

**GET with If-None-Match:**
```http
GET /api/v1/documents/{document_id} HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
If-None-Match: "abc123"
```

**Response (304 Not Modified):**
```http
HTTP/2 304 Not Modified
ETag: "abc123"
```

**Response (200 OK):**
```http
HTTP/2 200 OK
Content-Type: application/json
ETag: "def456"

{
  "data": {
    "id": "doc_123",
    ...
  }
}
```

#### 3.6.2. Last-Modified-Based Conditional Requests

**GET with If-Modified-Since:**
```http
GET /api/v1/documents/{document_id} HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
If-Modified-Since: Wed, 06 Feb 2026 16:00:00 GMT
```

**Response (304 Not Modified):**
```http
HTTP/2 304 Not Modified
Last-Modified: Wed, 06 Feb 2026 16:00:00 GMT
```

### 3.7. REST API Patterns

#### 3.7.1. CRUD Operations Pattern

**Create:**
```http
POST /api/v1/{resource}
```

**Read (Single):**
```http
GET /api/v1/{resource}/{id}
```

**Read (Collection):**
```http
GET /api/v1/{resource}
```

**Update (Full):**
```http
PUT /api/v1/{resource}/{id}
```

**Update (Partial):**
```http
PATCH /api/v1/{resource}/{id}
```

**Delete:**
```http
DELETE /api/v1/{resource}/{id}
```

#### 3.7.2. Nested Resources Pattern

**URI Structure:**
```
/api/v1/{parent_resource}/{parent_id}/{child_resource}
```

**Examples:**
- `/api/v1/users/{user_id}/documents` - Documents belonging to a user
- `/api/v1/workspaces/{workspace_id}/members` - Members of a workspace

**Operations:**
- **List Children:** `GET /api/v1/users/{user_id}/documents`
- **Create Child:** `POST /api/v1/users/{user_id}/documents`
- **Get Specific Child:** `GET /api/v1/users/{user_id}/documents/{document_id}`

#### 3.7.3. Action Pattern

For actions that don't fit CRUD semantics, use a dedicated resource with POST.

**URI Structure:**
```
POST /api/v1/{resource}/{id}/actions/{action}
```

**Examples:**
- `POST /api/v1/documents/{id}/actions/publish` - Publish a document
- `POST /api/v1/documents/{id}/actions/archive` - Archive a document

**Request:**
```http
POST /api/v1/documents/{id}/actions/publish HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "publish_at": "2026-02-07T09:00:00Z"
}
```

**Response (200 OK):**
```http
HTTP/2 200 OK
Content-Type: application/json

{
  "data": {
    "id": "doc_123",
    "status": "PUBLISHED",
    "published_at": "2026-02-07T09:00:00Z"
  }
}
```

---

## 4. WEBSOCKET API DESIGN

### 4.1. WebSocket Architecture Overview

The Tachyon WebSocket API provides real-time bidirectional communication between clients and the server, enabling live updates, collaborative features, and event-driven interactions. The WebSocket API complements the REST API by providing low-latency push notifications and real-time data synchronization.

**Protocol:** WebSocket over TLS 1.3 (wss://)

**Framework:** tokio-tungstenite (Rust)

**Characteristics:**
- Persistent bidirectional connection
- Full-duplex communication
- Low message latency (< 10ms)
- Automatic reconnection handling
- Subscription-based event channels

### 4.2. WebSocket Connection Lifecycle

#### 4.2.1. Connection Establishment

**WebSocket Handshake Request:**
```http
GET /ws/v1/connect HTTP/1.1
Host: ws.tachyon.example.com
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
Authorization: Bearer {access_token}
```

**WebSocket Handshake Response:**
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

**Connection Parameters:**
- **Endpoint:** `/ws/v1/connect`
- **Protocol:** WebSocket (RFC 6455)
- **Transport:** TLS 1.3 (wss://)
- **Authentication:** Bearer token in Authorization header
- **Origin:** Validated against allowed origins

#### 4.2.2. Connection Authentication

Authentication is performed during the WebSocket handshake using the Authorization header.

**Authentication Flow:**
1. Client includes Bearer token in handshake request
2. Server validates token and extracts user identity
3. Server establishes WebSocket connection with authenticated session
4. Connection is associated with user context for authorization

**Authentication Failure:**
- HTTP 401 Unauthorized - Invalid or expired token
- HTTP 403 Forbidden - Token valid but insufficient permissions

#### 4.2.3. Connection Termination

**Normal Closure (Client-Initiated):**
```json
{
  "type": "close",
  "code": 1000,
  "reason": "Normal closure"
}
```

**Normal Closure (Server-Initiated):**
```json
{
  "type": "close",
  "code": 1000,
  "reason": "Server shutting down"
}
```

**Error Closure Codes:**
- `1000` - Normal closure
- `1001` - Going away (server shutting down)
- `1002` - Protocol error
- `1003` - Unsupported data type
- `1008` - Policy violation
- `1011` - Internal error

### 4.3. WebSocket Message Protocol

#### 4.3.1. Message Format

All WebSocket messages use JSON format with a consistent envelope structure.

**Message Envelope:**
```json
{
  "id": "msg_abc123",
  "type": "message_type",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "field1": "value1",
    "field2": "value2"
  }
}
```

**Message Fields:**
- `id` - Unique message identifier (UUID v4)
- `type` - Message type identifier
- `timestamp` - ISO 8601 UTC timestamp
- `data` - Message payload (type-specific)

#### 4.3.2. Message Types

WebSocket messages are categorized into three types: client-to-server requests, server-to-client responses, and server-to-client events.

**Client-to-Server Requests:**
- `subscribe` - Subscribe to an event channel
- `unsubscribe` - Unsubscribe from an event channel
- `ping` - Heartbeat/ping message
- `action` - Perform an action via WebSocket

**Server-to-Client Responses:**
- `response` - Response to client request
- `pong` - Heartbeat/pong response
- `error` - Error response

**Server-to-Client Events:**
- `document_updated` - Document update notification
- `document_created` - Document creation notification
- `document_deleted` - Document deletion notification
- `user_joined` - User joined workspace notification
- `user_left` - User left workspace notification

### 4.4. Subscription Model

#### 4.4.1. Channel Subscription

Clients subscribe to event channels to receive real-time updates.

**Subscribe Request:**
```json
{
  "id": "msg_sub_001",
  "type": "subscribe",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "channel": "documents:{document_id}",
    "filter": {
      "events": ["updated", "deleted"]
    }
  }
}
```

**Subscribe Response:**
```json
{
  "id": "msg_sub_001",
  "type": "response",
  "timestamp": "2026-02-06T17:00:01Z",
  "data": {
    "status": "subscribed",
    "channel": "documents:{document_id}"
  }
}
```

**Channel Patterns:**
- `documents:{document_id}` - Updates to a specific document
- `documents` - Updates to all documents (user-scoped)
- `workspaces:{workspace_id}` - Updates to a workspace
- `users:{user_id}` - Updates to a user

#### 4.4.2. Channel Unsubscription

Clients unsubscribe from event channels to stop receiving updates.

**Unsubscribe Request:**
```json
{
  "id": "msg_unsub_001",
  "type": "unsubscribe",
  "timestamp": "2026-02-06T17:05:00Z",
  "data": {
    "channel": "documents:{document_id}"
  }
}
```

**Unsubscribe Response:**
```json
{
  "id": "msg_unsub_001",
  "type": "response",
  "timestamp": "2026-02-06T17:05:01Z",
  "data": {
    "status": "unsubscribed",
    "channel": "documents:{document_id}"
  }
}
```

### 4.5. Heartbeat Mechanism

#### 4.5.1. Ping/Pong Protocol

A heartbeat mechanism ensures connection liveness and detects stale connections.

**Ping Message (Client-to-Server):**
```json
{
  "id": "msg_ping_001",
  "type": "ping",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "sequence": 1
  }
}
```

**Pong Response (Server-to-Client):**
```json
{
  "id": "msg_ping_001",
  "type": "pong",
  "timestamp": "2026-02-06T17:00:00.500Z",
  "data": {
    "sequence": 1
  }
}
```

**Heartbeat Configuration:**
- **Interval:** 30 seconds
- **Timeout:** 60 seconds (2 missed heartbeats)
- **Action on Timeout:** Close connection with code 1001

#### 4.5.2. Server-Initiated Heartbeat

The server may also initiate heartbeats to detect unresponsive clients.

**Server Ping:**
```json
{
  "id": "msg_server_ping_001",
  "type": "ping",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "sequence": 1,
    "server_time": "2026-02-06T17:00:00Z"
  }
}
```

**Client Pong:**
```json
{
  "id": "msg_server_ping_001",
  "type": "pong",
  "timestamp": "2026-02-06T17:00:00.100Z",
  "data": {
    "sequence": 1,
    "client_time": "2026-02-06T17:00:00.100Z"
  }
}
```

### 4.6. Reconnection Strategy

#### 4.6.1. Automatic Reconnection

Clients must implement automatic reconnection with exponential backoff.

**Reconnection Algorithm:**
1. Detect connection closure
2. Wait for backoff interval: `min(2^n, 30)` seconds
3. Attempt reconnection
4. If successful, resubscribe to previous channels
5. If failed, increment backoff and retry

**Backoff Sequence:**
- Attempt 1: 0 seconds (immediate)
- Attempt 2: 2 seconds
- Attempt 3: 4 seconds
- Attempt 4: 8 seconds
- Attempt 5: 16 seconds
- Attempt 6+: 30 seconds (max)

#### 4.6.2. State Synchronization

After reconnection, clients must synchronize state to account for missed messages.

**State Sync Request:**
```json
{
  "id": "msg_sync_001",
  "type": "action",
  "timestamp": "2026-02-06T17:10:00Z",
  "data": {
    "action": "sync_state",
    "since": "2026-02-06T16:00:00Z"
  }
}
```

**State Sync Response:**
```json
{
  "id": "msg_sync_001",
  "type": "response",
  "timestamp": "2026-02-06T17:10:01Z",
  "data": {
    "events": [
      {
        "type": "document_updated",
        "document_id": "doc_123",
        "timestamp": "2026-02-06T16:30:00Z"
      },
      {
        "type": "document_created",
        "document_id": "doc_456",
        "timestamp": "2026-02-06T16:45:00Z"
      }
    ]
  }
}
```

### 4.7. WebSocket API Patterns

#### 4.7.1. Real-Time Document Collaboration Pattern

**Use Case:** Multiple users editing a document simultaneously.

**Channel:** `documents:{document_id}`

**Event Flow:**
1. User A subscribes to document channel
2. User B subscribes to document channel
3. User A makes changes via REST API
4. Server broadcasts `document_updated` event to all subscribers
5. User B receives update and refreshes view

**Document Updated Event:**
```json
{
  "id": "msg_event_001",
  "type": "document_updated",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "document_id": "doc_123",
    "updated_by": "user_456",
    "fields": ["title", "content"],
    "version": 5
  }
}
```

#### 4.7.2. Workspace Presence Pattern

**Use Case:** Track which users are active in a workspace.

**Channel:** `workspaces:{workspace_id}`

**User Joined Event:**
```json
{
  "id": "msg_event_002",
  "type": "user_joined",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "workspace_id": "ws_123",
    "user_id": "user_456",
    "user_name": "John Doe",
    "joined_at": "2026-02-06T17:00:00Z"
  }
}
```

**User Left Event:**
```json
{
  "id": "msg_event_003",
  "type": "user_left",
  "timestamp": "2026-02-06T17:30:00Z",
  "data": {
    "workspace_id": "ws_123",
    "user_id": "user_456",
    "left_at": "2026-02-06T17:30:00Z"
  }
}
```

#### 4.7.3. Progress Notification Pattern

**Use Case:** Notify clients of long-running operation progress.

**Channel:** `operations:{operation_id}`

**Progress Update Event:**
```json
{
  "id": "msg_event_004",
  "type": "progress_update",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "operation_id": "op_789",
    "operation_type": "document_export",
    "progress": 50,
    "status": "in_progress",
    "message": "Processing page 5 of 10"
  }
}
```

**Completion Event:**
```json
{
  "id": "msg_event_005",
  "type": "operation_complete",
  "timestamp": "2026-02-06T17:10:00Z",
  "data": {
    "operation_id": "op_789",
    "operation_type": "document_export",
    "progress": 100,
    "status": "completed",
    "result": {
      "download_url": "https://example.com/export/doc_123.pdf"
    }
  }
}
```

---

## 5. IPC API DESIGN

### 5.1. IPC Architecture Overview

The Tachyon IPC (Inter-Process Communication) API provides secure, type-safe communication between the Tauri frontend (WebView) and the Rust backend in the desktop application. The IPC API enables the frontend to invoke backend operations and receive asynchronous events.

**Protocol:** Tauri IPC mechanism

**Framework:** Tauri command and event systems

**Characteristics:**
- Type-safe command definitions
- Request-response and event-based patterns
- Capability-based access control
- serde serialization for type safety
- Session-based authentication

### 5.2. IPC Command System

#### 5.2.1. Command Definition

IPC commands are defined in Rust using Tauri's command system with type-safe serialization.

**Command Definition Example:**
```rust
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentRequest {
    pub document_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn get_document(
    request: GetDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<GetDocumentResponse, String> {
    // Command implementation
    Ok(GetDocumentResponse {
        id: request.document_id,
        title: "Example Document".to_string(),
        content: "Document content...".to_string(),
        created_at: "2026-02-06T16:00:00Z".to_string(),
        updated_at: "2026-02-06T16:30:00Z".to_string(),
    })
}
```

**Command Registration:**
```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_document,
            create_document,
            update_document,
            delete_document,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### 5.2.2. Command Invocation

Frontend invokes IPC commands using Tauri's invoke API.

**TypeScript Invocation:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

interface GetDocumentRequest {
  documentId: string;
}

interface GetDocumentResponse {
  id: string;
  title: string;
  content: string;
  createdAt: string;
  updatedAt: string;
}

async function getDocument(
  request: GetDocumentRequest
): Promise<GetDocumentResponse> {
  return await invoke<GetDocumentResponse>('get_document', request);
}

// Usage
const document = await getDocument({ documentId: 'doc_123' });
console.log(document.title);
```

**Error Handling:**
```typescript
try {
  const document = await getDocument({ documentId: 'doc_123' });
} catch (error) {
  console.error('Failed to get document:', error);
}
```

### 5.3. IPC Event System

#### 5.3.1. Event Emission

Backend emits events to frontend using Tauri's event system.

**Event Emission Example:**
```rust
use tauri::Manager;

#[tauri::command]
pub async fn update_document(
    request: UpdateDocumentRequest,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<UpdateDocumentResponse, String> {
    // Update document logic
    
    // Emit event to frontend
    app.emit_all("document_updated", DocumentUpdatedEvent {
        document_id: request.document_id.clone(),
        updated_by: request.user_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })?;
    
    Ok(UpdateDocumentResponse {
        id: request.document_id,
        status: "updated".to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentUpdatedEvent {
    pub document_id: String,
    pub updated_by: String,
    pub timestamp: String,
}
```

#### 5.3.2. Event Subscription

Frontend subscribes to IPC events using Tauri's listen API.

**Event Subscription Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

interface DocumentUpdatedEvent {
  documentId: string;
  updatedBy: string;
  timestamp: string;
}

function subscribeToDocumentUpdates(
  callback: (event: DocumentUpdatedEvent) => void
): () => void {
  const unlisten = listen<DocumentUpdatedEvent>(
    'document_updated',
    (event) => {
      callback(event.payload);
    }
  );
  
  return unlisten;
}

// Usage
const unsubscribe = subscribeToDocumentUpdates((event) => {
  console.log(`Document ${event.documentId} updated by ${event.updatedBy}`);
});

// Unsubscribe when done
unsubscribe();
```

### 5.4. IPC Capability-Based Access Control

#### 5.4.1. Capability Definition

Tauri capabilities define fine-grained permissions for IPC commands.

**Capability Definition (tauri.conf.json):**
```json
{
  "capabilities": [
    {
      "identifier": "document-read",
      "description": "Allows reading documents",
      "windows": ["main"],
      "permissions": [
        "allow:read-document",
        "allow:list-documents"
      ]
    },
    {
      "identifier": "document-write",
      "description": "Allows creating and updating documents",
      "windows": ["main"],
      "permissions": [
        "allow:create-document",
        "allow:update-document",
        "allow:delete-document"
      ]
    }
  ]
}
```

#### 5.4.2. Command Permission Assignment

Commands are assigned to capabilities in the Tauri configuration.

**Command Permission Assignment:**
```json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "document-read": {
        "all": true
      },
      "document-write": {
        "all": true
      }
    }
  }
}
```

### 5.5. IPC Error Handling

#### 5.5.1. Error Types

IPC commands return errors using Rust's Result type with descriptive error messages.

**Error Type Definition:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum IpcError {
    ValidationError(String),
    NotFound(String),
    PermissionDenied(String),
    InternalError(String),
}

impl From<IpcError> for String {
    fn from(error: IpcError) -> String {
        match error {
            IpcError::ValidationError(msg) => format!("Validation error: {}", msg),
            IpcError::NotFound(msg) => format!("Not found: {}", msg),
            IpcError::PermissionDenied(msg) => format!("Permission denied: {}", msg),
            IpcError::InternalError(msg) => format!("Internal error: {}", msg),
        }
    }
}
```

**Error Handling in Command:**
```rust
#[tauri::command]
pub async fn get_document(
    request: GetDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<GetDocumentResponse, String> {
    // Validate request
    if request.document_id.is_empty() {
        return Err(IpcError::ValidationError("Document ID is required".to_string()).into());
    }
    
    // Retrieve document
    let document = app_state
        .document_store
        .get(&request.document_id)
        .ok_or_else(|| IpcError::NotFound("Document not found".to_string()))?;
    
    // Check permissions
    if !app_state.has_permission(&request.user_id, "read", &document) {
        return Err(IpcError::PermissionDenied("Permission denied".to_string()).into());
    }
    
    Ok(GetDocumentResponse {
        id: document.id,
        title: document.title,
        content: document.content,
        created_at: document.created_at,
        updated_at: document.updated_at,
    })
}
```

#### 5.5.2. Frontend Error Handling

Frontend handles IPC errors using try-catch blocks.

**Error Handling Example:**
```typescript
interface IpcError {
  type: 'ValidationError' | 'NotFound' | 'PermissionDenied' | 'InternalError';
  message: string;
}

async function getDocument(
  request: GetDocumentRequest
): Promise<GetDocumentResponse> {
  try {
    return await invoke<GetDocumentResponse>('get_document', request);
  } catch (error) {
    const ipcError = error as IpcError;
    
    switch (ipcError.type) {
      case 'ValidationError':
        throw new Error(`Validation error: ${ipcError.message}`);
      case 'NotFound':
        throw new Error(`Not found: ${ipcError.message}`);
      case 'PermissionDenied':
        throw new Error(`Permission denied: ${ipcError.message}`);
      case 'InternalError':
        throw new Error(`Internal error: ${ipcError.message}`);
      default:
        throw new Error(`Unknown error: ${ipcError.message}`);
    }
  }
}
```

### 5.6. IPC API Patterns

#### 5.6.1. CRUD Operations Pattern

**Create Document Command:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_document(
    request: CreateDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<CreateDocumentResponse, String> {
    // Implementation
}
```

**Read Document Command:**
```rust
#[tauri::command]
pub async fn get_document(
    request: GetDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<GetDocumentResponse, String> {
    // Implementation
}
```

**Update Document Command:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub document_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[tauri::command]
pub async fn update_document(
    request: UpdateDocumentRequest,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<UpdateDocumentResponse, String> {
    // Implementation
}
```

**Delete Document Command:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentRequest {
    pub document_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentResponse {
    pub id: String,
    pub status: String,
}

#[tauri::command]
pub async fn delete_document(
    request: DeleteDocumentRequest,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<DeleteDocumentResponse, String> {
    // Implementation
}
```

#### 5.6.2. Batch Operations Pattern

**Batch Get Documents Command:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchGetDocumentsRequest {
    pub document_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchGetDocumentsResponse {
    pub documents: Vec<GetDocumentResponse>,
    pub not_found: Vec<String>,
}

#[tauri::command]
pub async fn batch_get_documents(
    request: BatchGetDocumentsRequest,
    app_state: State<'_, AppState>,
) -> Result<BatchGetDocumentsResponse, String> {
    let mut documents = Vec::new();
    let mut not_found = Vec::new();
    
    for document_id in &request.document_ids {
        match app_state.document_store.get(document_id) {
            Some(document) => documents.push(GetDocumentResponse {
                id: document.id.clone(),
                title: document.title.clone(),
                content: document.content.clone(),
                created_at: document.created_at.clone(),
                updated_at: document.updated_at.clone(),
            }),
            None => not_found.push(document_id.clone()),
        }
    }
    
    Ok(BatchGetDocumentsResponse {
        documents,
        not_found,
    })
}
```

#### 5.6.3. Streaming Operations Pattern

**Stream Document Changes Command:**
```rust
use tauri::Emitter;

#[tauri::command]
pub async fn stream_document_changes(
    request: StreamDocumentChangesRequest,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let document_id = request.document_id.clone();
    
    // Subscribe to document changes
    let mut rx = app_state
        .document_store
        .subscribe_changes(&document_id)?;
    
    // Stream changes to frontend
    while let Some(change) = rx.recv().await {
        app.emit("document_change", DocumentChangeEvent {
            document_id: document_id.clone(),
            change_type: change.change_type,
            timestamp: change.timestamp,
        })?;
    }
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamDocumentChangesRequest {
    pub document_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentChangeEvent {
    pub document_id: String,
    pub change_type: String,
    pub timestamp: String,
}
```

---

## 6. ERROR HANDLING DESIGN

### 6.1. Error Handling Principles

The Tachyon API error handling framework is designed to provide consistent, informative, and actionable error responses across all API surfaces (REST, WebSocket, and IPC). Error handling follows these core principles:

#### 6.1.1. Fail-Safe Behavior

All APIs must fail safely, ensuring that error conditions do not compromise system integrity or expose sensitive information.

**Formal Property:** For any error condition `E`, the system must transition to a well-defined safe state `S_safe` without violating system invariants.

#### 6.1.2. Error Visibility

Errors must be visible and actionable for API consumers. Error responses must include sufficient information for clients to understand and recover from error conditions.

**Formal Property:** For any error response `R`, `R` must contain: error code, human-readable message, and recovery guidance.

#### 6.1.3. Error Consistency

Error responses must be consistent across all API surfaces, using standardized error codes, formats, and HTTP status codes.

**Formal Property:** For equivalent error conditions across API surfaces, error representations must be structurally identical.

### 6.2. Error Classification

#### 6.2.1. Error Categories

Errors are classified into four categories based on their origin and recoverability:

**Category 1: Client Errors (4xx)**
- **Description:** Errors caused by invalid client requests
- **Recoverability:** Client can fix and retry
- **Examples:** Validation errors, authentication failures, permission denials

**Category 2: Server Errors (5xx)**
- **Description:** Errors caused by server-side failures
- **Recoverability:** Client cannot fix, may retry after delay
- **Examples:** Internal errors, service unavailability, database failures

**Category 3: Network Errors**
- **Description:** Errors caused by network connectivity issues
- **Recoverability:** Client can retry with backoff
- **Examples:** Timeouts, connection failures, DNS resolution failures

**Category 4: Protocol Errors**
- **Description:** Errors caused by protocol violations
- **Recoverability:** Client must fix protocol usage
- **Examples:** Invalid message format, unsupported protocol version

#### 6.2.2. Error Codes

Standardized error codes provide machine-readable error identifiers.

**Error Code Format:**
```
{CATEGORY}_{SPECIFIC_ERROR}
```

**Error Code Examples:**
- `VALIDATION_ERROR` - Input validation failed
- `AUTHENTICATION_ERROR` - Authentication failed
- `AUTHORIZATION_ERROR` - Authorization failed
- `NOT_FOUND_ERROR` - Resource not found
- `CONFLICT_ERROR` - Resource conflict
- `INTERNAL_ERROR` - Internal server error
- `SERVICE_UNAVAILABLE_ERROR` - Service temporarily unavailable

### 6.3. REST API Error Handling

#### 6.3.1. HTTP Status Codes

REST APIs use appropriate HTTP status codes to indicate error categories.

| Status Code | Category | Description |
|-------------|----------|-------------|
| 400 Bad Request | Client Error | Invalid request syntax |
| 401 Unauthorized | Client Error | Authentication required or failed |
| 403 Forbidden | Client Error | Authorization failed |
| 404 Not Found | Client Error | Resource not found |
| 409 Conflict | Client Error | Resource conflict |
| 422 Unprocessable Entity | Client Error | Semantic error in request |
| 429 Too Many Requests | Client Error | Rate limit exceeded |
| 500 Internal Server Error | Server Error | Internal server error |
| 502 Bad Gateway | Server Error | Invalid response from upstream |
| 503 Service Unavailable | Server Error | Service temporarily unavailable |
| 504 Gateway Timeout | Server Error | Upstream timeout |

#### 6.3.2. Error Response Format

REST API error responses follow a consistent JSON format.

**Error Response Structure:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed for field 'title'",
    "details": [
      {
        "field": "title",
        "message": "Title is required and must be at least 1 character"
      }
    ]
  },
  "meta": {
    "timestamp": "2026-02-06T17:00:00Z",
    "request_id": "req_abc123"
  }
}
```

**Error Response Fields:**
- `error.code` - Machine-readable error code (SCREAMING_SNAKE_CASE)
- `error.message` - Human-readable error message
- `error.details` - Optional detailed error information
- `meta.timestamp` - ISO 8601 UTC timestamp
- `meta.request_id` - Unique request identifier for tracing

#### 6.3.3. Validation Error Example

**Request:**
```http
POST /api/v1/documents HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "title": "",
  "content": "Document content..."
}
```

**Response (422 Unprocessable Entity):**
```http
HTTP/2 422 Unprocessable Entity
Content-Type: application/json

{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed for one or more fields",
    "details": [
      {
        "field": "title",
        "message": "Title is required and must be at least 1 character"
      }
    ]
  },
  "meta": {
    "timestamp": "2026-02-06T17:00:00Z",
    "request_id": "req_abc123"
  }
}
```

#### 6.3.4. Authentication Error Example

**Request:**
```http
GET /api/v1/documents HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer invalid_token
```

**Response (401 Unauthorized):**
```http
HTTP/2 401 Unauthorized
Content-Type: application/json

{
  "error": {
    "code": "AUTHENTICATION_ERROR",
    "message": "Authentication failed: Invalid or expired token",
    "details": [
      {
        "field": "authorization",
        "message": "Token is expired or invalid"
      }
    ]
  },
  "meta": {
    "timestamp": "2026-02-06T17:00:00Z",
    "request_id": "req_def456"
  }
}
```

### 6.4. WebSocket API Error Handling

#### 6.4.1. Error Message Format

WebSocket error messages use the same envelope structure as other messages.

**Error Message Structure:**
```json
{
  "id": "msg_error_001",
  "type": "error",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed for field 'channel'",
    "details": [
      {
        "field": "channel",
        "message": "Channel must match pattern 'documents:{document_id}'"
      }
    ]
  }
}
```

#### 6.4.2. Error Message Examples

**Subscription Validation Error:**
```json
{
  "id": "msg_error_002",
  "type": "error",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid channel subscription",
    "details": [
      {
        "field": "channel",
        "message": "Channel 'invalid_channel' does not match required pattern"
      }
    ]
  }
}
```

**Authentication Error:**
```json
{
  "id": "msg_error_003",
  "type": "error",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "code": "AUTHENTICATION_ERROR",
    "message": "Authentication failed: Session expired",
    "details": []
  }
}
```

### 6.5. IPC API Error Handling

#### 6.5.1. Error Type Definition

IPC errors are defined as Rust enums with serialization support.

**Error Type Definition:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum IpcError {
    ValidationError {
        field: String,
        message: String,
    },
    AuthenticationError {
        message: String,
    },
    AuthorizationError {
        resource: String,
        action: String,
    },
    NotFoundError {
        resource_type: String,
        resource_id: String,
    },
    ConflictError {
        resource_type: String,
        conflict_details: String,
    },
    InternalError {
        message: String,
    },
}

impl From<IpcError> for String {
    fn from(error: IpcError) -> String {
        serde_json::to_string(&error).unwrap_or_else(|_| {
            serde_json::json!({
                "code": "INTERNAL_ERROR",
                "message": "Failed to serialize error"
            }).to_string()
        })
    }
}
```

#### 6.5.2. Error Return Pattern

IPC commands return errors using Rust's Result type.

**Error Return Example:**
```rust
#[tauri::command]
pub async fn get_document(
    request: GetDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<GetDocumentResponse, String> {
    // Validate request
    if request.document_id.is_empty() {
        return Err(IpcError::ValidationError {
            field: "document_id".to_string(),
            message: "Document ID is required".to_string(),
        }.into());
    }
    
    // Retrieve document
    let document = app_state
        .document_store
        .get(&request.document_id)
        .ok_or_else(|| IpcError::NotFoundError {
            resource_type: "document".to_string(),
            resource_id: request.document_id.clone(),
        })?;
    
    // Check permissions
    if !app_state.has_permission(&request.user_id, "read", &document) {
        return Err(IpcError::AuthorizationError {
            resource: format!("document:{}", request.document_id),
            action: "read".to_string(),
        }.into());
    }
    
    Ok(GetDocumentResponse {
        id: document.id,
        title: document.title,
        content: document.content,
        created_at: document.created_at,
        updated_at: document.updated_at,
    })
}
```

### 6.6. Error Handling Patterns

#### 6.6.1. Validation Error Pattern

Validate all inputs before processing and return detailed validation errors.

**Rust Validation:**
```rust
fn validate_document_request(request: &CreateDocumentRequest) -> Result<(), IpcError> {
    if request.title.is_empty() {
        return Err(IpcError::ValidationError {
            field: "title".to_string(),
            message: "Title is required".to_string(),
        });
    }
    
    if request.title.len() > 200 {
        return Err(IpcError::ValidationError {
            field: "title".to_string(),
            message: "Title must be at most 200 characters".to_string(),
        });
    }
    
    if request.content.len() > 100_000 {
        return Err(IpcError::ValidationError {
            field: "content".to_string(),
            message: "Content must be at most 100,000 characters".to_string(),
        });
    }
    
    Ok(())
}
```

#### 6.6.2. Resource Not Found Pattern

Return not found errors when requested resources do not exist.

**Rust Not Found Handling:**
```rust
pub async fn get_document(
    request: GetDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<GetDocumentResponse, String> {
    let document = app_state
        .document_store
        .get(&request.document_id)
        .ok_or_else(|| IpcError::NotFoundError {
            resource_type: "document".to_string(),
            resource_id: request.document_id.clone(),
        })?;
    
    Ok(GetDocumentResponse {
        id: document.id,
        title: document.title,
        content: document.content,
        created_at: document.created_at,
        updated_at: document.updated_at,
    })
}
```

#### 6.6.3. Authorization Error Pattern

Check permissions before performing operations and return authorization errors.

**Rust Authorization Handling:**
```rust
pub async fn delete_document(
    request: DeleteDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<DeleteDocumentResponse, String> {
    let document = app_state
        .document_store
        .get(&request.document_id)
        .ok_or_else(|| IpcError::NotFoundError {
            resource_type: "document".to_string(),
            resource_id: request.document_id.clone(),
        })?;
    
    if !app_state.has_permission(&request.user_id, "delete", &document) {
        return Err(IpcError::AuthorizationError {
            resource: format!("document:{}", request.document_id),
            action: "delete".to_string(),
        }.into());
    }
    
    app_state.document_store.delete(&request.document_id)?;
    
    Ok(DeleteDocumentResponse {
        id: request.document_id,
        status: "deleted".to_string(),
    })
}
```

---

## 7. SECURITY DESIGN

### 7.1. Security Principles

The Tachyon API security framework implements a defense-in-depth strategy with multiple layers of security controls aligned with [ADR-010](.specs/02_adrs/010_security_architecture.md).

#### 7.1.1. Zero Trust Architecture

No implicit trust is granted within or across security boundaries. All API requests must be authenticated and authorized regardless of origin.

**Formal Property:** For any API request `R`, authentication `A(R)` and authorization `Z(R)` must be verified before processing.

#### 7.1.2. Principle of Least Privilege

API operations are granted the minimum permissions required to complete their function.

**Formal Property:** For any operation `O` requiring permissions `P`, `P` must be the minimal set such that `O` can complete successfully.

#### 7.1.3. Secure by Default

All APIs have secure default configurations. Security controls are enabled by default and require explicit configuration to disable.

**Formal Property:** For any security control `C`, default state `C_default = enabled`.

#### 7.1.4. Fail-Safe Security

Security failures result in safe, secure states. Error conditions do not bypass security controls.

**Formal Property:** For any security failure `F`, system transitions to secure state `S_secure`.

### 7.2. REST API Security

#### 7.2.1. Authentication

REST APIs use JWT (JSON Web Token) authentication with Bearer token scheme.

**Authentication Flow:**
1. Client authenticates and receives JWT access token
2. Client includes token in Authorization header
3. Server validates token signature and claims
4. Server extracts user identity from token
5. Request proceeds with authenticated user context

**Authentication Header:**
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**JWT Claims Structure:**
```json
{
  "sub": "user_123",
  "iat": 1738846400,
  "exp": 1738850000,
  "scope": ["read:documents", "write:documents"]
}
```

**Token Validation:**
```rust
use jsonwebtoken::{decode, Validation, Algorithm, DecodingKey, TokenData};

fn validate_token(token: &str) -> Result<TokenData, String> {
    let decoding_key = DecodingKey::from_secret(b"your-256-bit-secret");
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Invalid token: {}", e))
}
```

#### 7.2.2. Authorization

REST APIs implement capability-based authorization with resource-level and action-level permissions.

**Permission Model:**
- **Resource:** Entity being accessed (e.g., document, workspace)
- **Action:** Operation being performed (e.g., read, write, delete)
- **Scope:** Token scopes granted to user

**Authorization Check:**
```rust
pub struct AuthorizationContext {
    pub user_id: String,
    pub scopes: Vec<String>,
}

impl AuthorizationContext {
    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        let required_scope = format!("{}:{}", action, resource);
        self.scopes.contains(&required_scope)
    }
}

// Usage in API handler
pub async fn get_document(
    request: GetDocumentRequest,
    auth: AuthContext,
) -> Result<GetDocumentResponse, String> {
    if !auth.has_permission("documents", "read") {
        return Err("Permission denied".to_string());
    }
    
    // Proceed with document retrieval
}
```

#### 7.2.3. Rate Limiting

REST APIs implement rate limiting to prevent abuse and ensure fair resource allocation.

**Rate Limiting Strategy:**
- **Token Bucket Algorithm:** Allows bursts while maintaining average rate
- **Per-User Limits:** Separate limits per authenticated user
- **Endpoint-Specific Limits:** Different limits for different endpoints

**Rate Limit Configuration:**
```rust
use governor::{Quota, RateLimiter};

const RATE_LIMIT: Quota = Quota::per_second(10); // 10 requests per second

pub struct RateLimiter {
    limiter: RateLimiter<...>,
}

impl RateLimiter {
    pub fn check_rate_limit(&self, user_id: &str) -> Result<(), String> {
        if self.limiter.check().is_err() {
            return Err("Rate limit exceeded".to_string());
        }
        Ok(())
    }
}
```

**Rate Limit Response:**
```http
HTTP/2 429 Too Many Requests
Content-Type: application/json

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Please retry after 60 seconds.",
    "details": [
      {
        "field": "retry_after",
        "message": "60"
      }
    ]
  },
  "meta": {
    "timestamp": "2026-02-06T17:00:00Z",
    "request_id": "req_abc123"
  }
}
```

#### 7.2.4. Input Validation

All REST API inputs are validated to prevent injection attacks and ensure data integrity.

**Validation Rules:**
- **Type Validation:** Ensure data types match expected schemas
- **Length Validation:** Enforce minimum and maximum lengths
- **Format Validation:** Validate formats (email, URL, UUID)
- **Content Validation:** Sanitize and validate content

**Validation Example:**
```rust
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    #[validate(length(max = 100_000))]
    pub content: String,
    
    #[validate(url)]
    pub external_url: Option<String>,
}

pub async fn create_document(
    request: CreateDocumentRequest,
) -> Result<CreateDocumentResponse, String> {
    request.validate()
        .map_err(|e| format!("Validation error: {}", e))?;
    
    // Proceed with document creation
}
```

### 7.3. WebSocket API Security

#### 7.3.1. WebSocket Authentication

WebSocket connections are authenticated during the handshake using the Authorization header.

**Handshake Authentication:**
```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    auth: AuthContext,
) -> impl IntoResponse {
    ws.protocols(["tachyon-v1"])
        .on_upgrade(move |socket| async move {
            // WebSocket connection established
            // Auth context already validated
        })
}
```

**Authentication Failure:**
```http
HTTP/1.1 401 Unauthorized
```

#### 7.3.2. Channel Authorization

WebSocket channel subscriptions are authorized to prevent unauthorized access to event streams.

**Channel Authorization:**
```rust
pub struct ChannelAuthorization {
    pub user_id: String,
    pub scopes: Vec<String>,
}

impl ChannelAuthorization {
    pub fn can_subscribe(&self, channel: &str) -> bool {
        match channel {
            channel if channel.starts_with("documents:") => {
                self.scopes.contains(&"read:documents".to_string())
            }
            channel if channel.starts_with("workspaces:") => {
                self.scopes.contains(&"read:workspaces".to_string())
            }
            _ => false,
        }
    }
}
```

**Authorization Failure Message:**
```json
{
  "id": "msg_error_001",
  "type": "error",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "code": "AUTHORIZATION_ERROR",
    "message": "Not authorized to subscribe to channel",
    "details": [
      {
        "field": "channel",
        "message": "Permission denied for channel 'documents:doc_123'"
      }
    ]
  }
}
```

### 7.4. IPC API Security

#### 7.4.1. Capability-Based Access Control

IPC APIs use Tauri's capability system for fine-grained access control.

**Capability Definition (tauri.conf.json):**
```json
{
  "capabilities": [
    {
      "identifier": "document-read",
      "description": "Allows reading documents",
      "windows": ["main"],
      "permissions": [
        "allow:read-document",
        "allow:list-documents"
      ]
    },
    {
      "identifier": "document-write",
      "description": "Allows creating and updating documents",
      "windows": ["main"],
      "permissions": [
        "allow:create-document",
        "allow:update-document",
        "allow:delete-document"
      ]
    }
  ]
}
```

#### 7.4.2. Command Authorization

IPC commands check capabilities before executing operations.

**Authorization Check:**
```rust
#[tauri::command]
pub async fn delete_document(
    request: DeleteDocumentRequest,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<DeleteDocumentResponse, String> {
    // Check capability
    if !app_state.has_capability(&request.user_id, "delete-document") {
        return Err(IpcError::AuthorizationError {
            resource: format!("document:{}", request.document_id),
            action: "delete".to_string(),
        }.into());
    }
    
    // Proceed with document deletion
}
```

### 7.5. Security Patterns

#### 7.5.1. Input Sanitization Pattern

All inputs are sanitized to prevent injection attacks.

**Sanitization Example:**
```rust
use ammonia::clean;

pub fn sanitize_html(input: &str) -> String {
    clean(input).to_string()
}

pub async fn create_document(
    request: CreateDocumentRequest,
) -> Result<CreateDocumentResponse, String> {
    let sanitized_content = sanitize_html(&request.content);
    
    // Store sanitized content
}
```

#### 7.5.2. Secure Logging Pattern

Security-sensitive information is not logged. Logs are sanitized before output.

**Secure Logging:**
```rust
use log::{info, warn};

pub async fn login(
    request: LoginRequest,
) -> Result<LoginResponse, String> {
    // Validate credentials
    if validate_credentials(&request.username, &request.password)? {
        info!("User logged in: {}", request.username);
        
        return Ok(LoginResponse {
            token: generate_token(&request.username)?,
        });
    }
    
    warn!("Failed login attempt for user: {}", request.username);
    Err("Invalid credentials".to_string())
}
```

#### 7.5.3. Secure Error Messages Pattern

Error messages do not expose sensitive information. Generic messages are used for security-related errors.

**Secure Error Messages:**
```rust
pub async fn get_document(
    request: GetDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<GetDocumentResponse, String> {
    let document = app_state
        .document_store
        .get(&request.document_id)
        .ok_or_else(|| IpcError::NotFoundError {
            resource_type: "document".to_string(),
            resource_id: request.document_id.clone(),
        })?;
    
    // Check permissions
    if !app_state.has_permission(&request.user_id, "read", &document) {
        // Generic error message, don't expose document details
        return Err(IpcError::AuthorizationError {
            resource: "document".to_string(),
            action: "read".to_string(),
        }.into());
    }
    
    Ok(GetDocumentResponse {
        id: document.id,
        title: document.title,
        content: document.content,
        created_at: document.created_at,
        updated_at: document.updated_at,
    })
}
```

---

## 8. VERSIONING DESIGN

### 8.1. Versioning Principles

The Tachyon API versioning strategy ensures backward compatibility, clear deprecation paths, and smooth transitions between API versions.

#### 8.1.1. Semantic Versioning

API versioning follows semantic versioning principles with major, minor, and patch versions.

**Version Format:**
```
v{MAJOR}.{MINOR}.{PATCH}
```

**Version Semantics:**
- **MAJOR:** Breaking changes (incompatible API changes)
- **MINOR:** Backward-compatible additions (new features, endpoints)
- **PATCH:** Backward-compatible bug fixes

**Formal Property:** For version `V1 = v{M1}.{N1}.{P1}` and `V2 = v{M2}.{N2}.{P2}`:
- If `M1 ≠ M2`, APIs are incompatible
- If `N1 ≠ N2`, APIs are compatible but `V2` has additional features
- If `P1 ≠ P2`, APIs are identical except for bug fixes

#### 8.1.2. URI Versioning

REST APIs use URI path versioning for clear version identification.

**Version Format:**
```
/api/v{MAJOR}/{resource}
```

**Examples:**
- `/api/v1/documents` - Version 1 API
- `/api/v2/documents` - Version 2 API

**Versioning Rules:**
- Only major versions appear in URI
- Minor and patch versions do not require URI changes
- New major versions maintain backward compatibility with previous versions

#### 8.1.3. Deprecation Policy

API endpoints and features are deprecated before removal to allow client migration.

**Deprecation Timeline:**
1. **Announcement:** Deprecation announced 6 months before removal
2. **Warning Period:** Deprecation warnings in API responses for 3 months
3. **Sunset Period:** Deprecated features continue to work for 3 months
4. **Removal:** Deprecated features removed after 6 months

**Deprecation Response Header:**
```http
X-API-Deprecation: true
X-API-Deprecation-Date: 2026-08-06
X-API-Sunset-Date: 2027-02-06
X-API-Replacement: /api/v2/documents
```

### 8.2. REST API Versioning

#### 8.2.1. Version Strategy

REST APIs use URI-based versioning with major version numbers.

**Version URI Examples:**
- `/api/v1/documents` - Version 1
- `/api/v2/documents` - Version 2 (breaking changes)

**Version-Specific Implementation:**
```rust
use axum::Router;

pub fn api_router() -> Router {
    Router::new()
        .nest("/api/v1", v1_routes())
        .nest("/api/v2", v2_routes())
}

pub fn v1_routes() -> Router {
    Router::new()
        .route("/documents", get(get_documents_v1))
        .route("/documents/:id", get(get_document_v1))
}

pub fn v2_routes() -> Router {
    Router::new()
        .route("/documents", get(get_documents_v2))
        .route("/documents/:id", get(get_document_v2))
}
```

#### 8.2.2. Breaking Changes

Breaking changes require a new major version.

**Breaking Change Examples:**
- Removing or renaming an endpoint
- Changing request/response field names
- Changing field data types
- Removing required request fields
- Changing authentication requirements
- Changing error response format

**Non-Breaking Change Examples:**
- Adding new optional request fields
- Adding new response fields
- Adding new endpoints
- Adding new query parameters
- Changing error messages (not codes)

#### 8.2.3. Version Migration

Clients are notified of new versions and provided with migration guidance.

**Migration Guide Structure:**
```markdown
# API v1 to v2 Migration Guide

## Breaking Changes

### 1. Document Resource Changes

**v1:**
```json
{
  "id": "doc_123",
  "doc_title": "Example Document",
  "doc_content": "Content..."
}
```

**v2:**
```json
{
  "id": "doc_123",
  "title": "Example Document",
  "content": "Content..."
}
```

**Migration:** Rename `doc_title` to `title` and `doc_content` to `content`.

### 2. Authentication Changes

**v1:** Bearer token in `Authorization` header
**v2:** Bearer token in `Authorization` header with `Bearer` prefix required

**Migration:** Ensure all requests include `Bearer` prefix in Authorization header.
```

### 8.3. WebSocket API Versioning

#### 8.3.1. Protocol Versioning

WebSocket APIs use protocol negotiation during handshake for versioning.

**Protocol Negotiation:**
```http
GET /ws/v1/connect HTTP/1.1
Host: ws.tachyon.example.com
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Protocol: tachyon-v1
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
Authorization: Bearer {access_token}
```

**Protocol Response:**
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Protocol: tachyon-v1
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

#### 8.3.2. Message Format Versioning

WebSocket message formats include version information for compatibility.

**Message Envelope with Version:**
```json
{
  "id": "msg_abc123",
  "type": "subscribe",
  "version": "1.0",
  "timestamp": "2026-02-06T17:00:00Z",
  "data": {
    "channel": "documents:{document_id}"
  }
}
```

**Version Compatibility Check:**
```rust
pub fn validate_message_version(message: &Message) -> Result<(), String> {
    match message.version.as_str() {
        "1.0" => Ok(()),
        "2.0" => Ok(()),
        _ => Err(format!("Unsupported message version: {}", message.version)),
    }
}
```

### 8.4. IPC API Versioning

#### 8.4.1. Command Versioning

IPC commands are versioned using Rust module organization.

**Versioned Command Modules:**
```rust
// v1/commands.rs
pub mod commands {
    pub mod v1 {
        use serde::{Deserialize, Serialize};
        
        #[tauri::command]
        pub async fn get_document(
            request: GetDocumentRequestV1,
            app_state: State<'_, AppState>,
        ) -> Result<GetDocumentResponseV1, String> {
            // v1 implementation
        }
    }
}

// v2/commands.rs
pub mod commands {
    pub mod v2 {
        use serde::{Deserialize, Serialize};
        
        #[tauri::command]
        pub async fn get_document(
            request: GetDocumentRequestV2,
            app_state: State<'_, AppState>,
        ) -> Result<GetDocumentResponseV2, String> {
            // v2 implementation
        }
    }
}
```

#### 8.4.2. Version Selection

Clients select the command version during invocation.

**Version Selection:**
```typescript
// Invoke v1 command
const documentV1 = await invoke<GetDocumentResponseV1>('get_document_v1', {
  documentId: 'doc_123'
});

// Invoke v2 command
const documentV2 = await invoke<GetDocumentResponseV2>('get_document_v2', {
  documentId: 'doc_123'
});
```

### 8.5. Versioning Patterns

#### 8.5.1. Backward Compatibility Pattern

New API versions maintain backward compatibility with previous versions.

**Backward Compatible Change:**
```rust
// v1 response
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentResponseV1 {
    pub id: String,
    pub title: String,
    pub content: String,
}

// v2 response (backward compatible)
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentResponseV2 {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,  // New field
    pub updated_at: String,  // New field
}
```

#### 8.5.2. Adapter Pattern

Adapter pattern allows v2 clients to work with v1 APIs during migration.

**Adapter Implementation:**
```rust
pub struct V1ToV2Adapter;

impl V1ToV2Adapter {
    pub fn adapt_document_v1_to_v2(v1: DocumentResponseV1) -> DocumentResponseV2 {
        DocumentResponseV2 {
            id: v1.id,
            title: v1.title,
            content: v1.content,
            created_at: "1970-01-01T00:00:00Z".to_string(),  // Default value
            updated_at: "1970-01-01T00:00:00Z".to_string(),  // Default value
        }
    }
}
```

#### 8.5.3. Feature Flag Pattern

Feature flags control availability of new features across versions.

**Feature Flag Implementation:**
```rust
pub struct FeatureFlags {
    pub enable_new_document_fields: bool,
    pub enable_batch_operations: bool,
}

pub async fn get_document(
    request: GetDocumentRequest,
    app_state: State<'_, AppState>,
) -> Result<GetDocumentResponse, String> {
    let document = app_state.document_store.get(&request.document_id)?;
    
    let response = if app_state.feature_flags.enable_new_document_fields {
        GetDocumentResponse::V2(GetDocumentResponseV2 {
            id: document.id,
            title: document.title,
            content: document.content,
            created_at: document.created_at,
            updated_at: document.updated_at,
        })
    } else {
        GetDocumentResponse::V1(GetDocumentResponseV1 {
            id: document.id,
            title: document.title,
            content: document.content,
        })
    };
    
    Ok(response)
}
```

---

## 9. REFERENCES

### 9.1. Standards and Specifications

| Reference | Description |
|-----------|-------------|
| [ISO/IEC 26514:2021](https://www.iso.org/standard/71398.html) | Systems and software engineering — Requirements for designers and developers of user documentation |
| [ISO/IEC 12207:2017](https://www.iso.org/standard/63712.html) | Systems and software engineering — Software life cycle processes |
| [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) | Systems and software engineering — Systems and software Quality Requirements and Evaluation (SQuaRE) |
| [IEEE 1063:2001](https://standards.ieee.org/standard/1063-2001.html) | IEEE Standard for Software User Documentation |
| [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) | The WebSocket Protocol |
| [RFC 7231](https://datatracker.ietf.org/doc/html/rfc7231) | Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content |
| [RFC 7540](https://datatracker.ietf.org/doc/html/rfc7540) | Hypertext Transfer Protocol Version 2 (HTTP/2) |
| [RFC 7519](https://datatracker.ietf.org/doc/html/rfc7519) | HTTP/2 |
| [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.0) | OpenAPI Specification |
| [AsyncAPI 2.6](https://www.asyncapi.com/docs/specifications/v2.6.0) | AsyncAPI Specification |

### 9.2. Architecture Decision Records

| ADR | Title | Relevance |
|-----|-------|----------|
| [ADR-001](.specs/02_adrs/001_rust_as_primary_language.md) | Rust as Primary Language | Defines Rust as the primary language for API implementation |
| [ADR-002](.specs/02_adrs/002_tauri_for_desktop_application.md) | Tauri for Desktop Application | Defines Tauri for desktop IPC API |
| [ADR-003](.specs/02_adrs/003_axum_for_http2_server.md) | Axum for HTTP/2 Server | Defines Axum for REST API implementation |
| [ADR-009](.specs/02_adrs/009_ipc_communication_architecture.md) | IPC Communication Architecture | Defines IPC communication patterns |
| [ADR-010](.specs/02_adrs/010_security_architecture.md) | Security Architecture | Defines security controls for APIs |

### 9.3. Framework and Library Documentation

| Framework/Library | Documentation URL |
|-----------------|-------------------|
| Rust | https://doc.rust-lang.org/ |
| Axum | https://docs.rs/axum/latest/axum/ |
| Tauri | https://tauri.app/v1/guides/ |
| tokio | https://docs.rs/tokio/latest/tokio/ |
| tokio-tungstenite | https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/ |
| serde | https://serde.rs/ |
| jsonwebtoken | https://docs.rs/jsonwebtoken/latest/jsonwebtoken/ |
| validator | https://docs.rs/validator/latest/validator/ |
| governor | https://docs.rs/governor/latest/governor/ |
| ammonia | https://docs.rs/ammonia/latest/ammonia/ |

### 9.4. Related Documentation

| Document | Path |
|----------|------|
| Coding and Documentation Standards | [`.specs/01_standards/coding_standards.md`](.specs/01_standards/coding_standards.md) |
| Tasks and Work Breakdown Structure | [`.specs/tasks.md`](.specs/tasks.md) |
| Threat Model | [`.specs/03_threat_model/threat_model.md`](.specs/03_threat_model/threat_model.md) |
| Future State Design | [`.specs/04_future_state/future_state_design.md`](.specs/04_future_state/future_state_design.md) |
| Test Plan | [`.specs/08_test_plan/test_plan.md`](.specs/08_test_plan/test_plan.md) |
| Developer Guide | [`.docs/developer/developer_guide.md`](.docs/developer/developer_guide.md) |

### 9.5. Additional Resources

| Resource | URL |
|----------|-----|
| REST API Design Best Practices | https://restfulapi.net/ |
| WebSocket Protocol Specification | https://datatracker.ietf.org/doc/html/rfc6455 |
| JSON Web Token (JWT) | https://jwt.io/ |
| OAuth 2.0 | https://oauth.net/2/ |
| API Security Best Practices | https://owasp.org/www-project-api-security/ |
| Semantic Versioning | https://semver.org/ |

---

**Document Control**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-06 | Technical Writer | Initial release |

---

**End of Document**
