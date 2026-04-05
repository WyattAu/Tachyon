# TACHYON: INTEGRATION DOCUMENTATION

**Document ID:** TACHYON-INT-001-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Technical Integration Documentation
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1058-2009

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Integration Framework](#2-integration-framework)
3. [Integration Architecture](#3-integration-architecture)
4. [REST Integration](#4-rest-integration)
5. [WebSocket Integration](#5-websocket-integration)
6. [IPC Integration](#6-ipc-integration)
7. [Webhook Integration](#7-webhook-integration)
8. [OAuth Integration](#8-oauth-integration)
9. [Plugin Integration](#9-plugin-integration)
10. [Testing](#10-testing)
11. [Error Handling](#11-error-handling)
12. [Security Considerations](#12-security-considerations)
13. [References](#13-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides comprehensive technical specifications for all integration interfaces within the Tachyon toolchain. It defines the protocols, data formats, security mechanisms, and operational procedures for integrating Tachyon components with external systems, services, and user interfaces. This document serves as the authoritative reference for implementing, testing, and maintaining integration points across the system.

### 1.2. Scope

This document covers the following integration categories:

1. **REST API Integration** - HTTP/2-based RESTful API endpoints for client-server communication
2. **WebSocket Integration** - Real-time bidirectional communication for collaborative features
3. **IPC Integration** - Inter-process communication between desktop application and backend services
4. **Webhook Integration** - Event-driven notifications to external services
5. **OAuth Integration** - Third-party authentication and authorization providers
6. **Plugin Integration** - Extensible plugin architecture for custom functionality

### 1.3. Document Audience

**Primary Audience:**
- Systems Engineers implementing integration interfaces
- Frontend developers consuming APIs
- Security engineers reviewing integration security
- Quality assurance engineers testing integrations
- DevOps engineers deploying integrated systems

**Secondary Audience:**
- Technical architects designing integration patterns
- Product managers understanding integration capabilities
- Third-party developers building integrations

### 1.4. Document Dependencies

This document depends on the following documents:
- [TACHYON-STD-V1.0](.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-ADR-001-V1.0](.specs/02_adrs/001_rust_as_primary_language.md) - Rust Language Selection
- [TACHYON-ADR-010-V1.0](.specs/02_adrs/010_security_architecture.md) - Security Architecture
- [TACHYON-REQ-INDEX-V1.0](.specs/04_future_state/reqs/000-index.md) - Requirements Index
- [TACHYON-DSN-INDEX-V1.0](.specs/04_future_state/design/000-index.md) - Design Index
- [TACHYON-TST-V1.0](.specs/04_future_state/test_plan.md) - Test Plan

### 1.5. Terminology and Definitions

| Term | Definition |
|-------|------------|
| **Integration Point** | A defined interface through which Tachyon components communicate with external systems |
| **Endpoint** | A specific URL or address that accepts requests for a particular function |
| **Payload** | The data transmitted in a request or response body |
| **Middleware** | Software that processes requests/responses before they reach handlers |
| **Webhook** | An HTTP callback triggered by specific events |
| **OAuth** | Open standard for authorization (RFC 6749) |
| **IPC** | Inter-Process Communication - mechanisms for processes to exchange data |
| **WebSocket** - A communication protocol providing full-duplex communication over a single TCP connection |
| **Plugin** | A software component that extends Tachyon functionality without modifying core code |

---

## 2. INTEGRATION FRAMEWORK

### 2.1. Framework Overview

The Tachyon integration framework provides a unified abstraction layer for all integration types, ensuring consistent behavior, security, and error handling across different communication protocols. The framework implements the following core principles:

1. **Protocol Abstraction** - Common interfaces for REST, WebSocket, IPC, and webhook integrations
2. **Type Safety** - Strongly typed message contracts enforced at compile time
3. **Security by Default** - Mandatory authentication, authorization, and encryption
4. **Observability** - Comprehensive logging, tracing, and metrics for all integrations
5. **Resilience** - Automatic retries, circuit breakers, and graceful degradation

### 2.2. Integration Categories

The Tachyon system implements six primary integration categories:

| Category | Protocol | Purpose | Latency Target | Security Model |
|-----------|-----------|---------|----------------|----------------|
| **REST API** | HTTP/2 | Client-server communication | <100ms | JWT + TLS 1.3 |
| **WebSocket** | WebSocket (RFC 6455) | Real-time collaboration | <50ms | JWT + TLS 1.3 |
| **IPC** | Tauri Commands | Desktop-backend communication | <10ms | Capability-based |
| **Webhook** | HTTPS | Event notifications | <500ms | HMAC signatures |
| **OAuth** | OAuth 2.0 (RFC 6749) | Third-party auth | <200ms | PKCE + TLS 1.3 |
| **Plugin** | WASM/FFI | Custom extensions | Variable | Sandboxed execution |

### 2.3. Integration Lifecycle

All integrations follow a standardized lifecycle:

```mermaid
stateDiagram-v2
    [*] --> Initialization: Integration Request
    Initialization --> Configuration: Load Configuration
    Configuration --> Validation: Validate Parameters
    Validation --> Active: Validation Success
    Validation --> Error: Validation Failure
    Error --> [*]: Log and Terminate
    Active --> Processing: Process Requests
    Processing --> Active: Continue Processing
    Processing --> Degraded: Performance Threshold
    Processing --> Error: Critical Error
    Degraded --> Active: Recovery
    Error --> [*]: Graceful Shutdown
    Active --> Shutdown: Termination Request
    Shutdown --> [*]: Cleanup Complete
```

### 2.4. Integration Configuration

All integrations are configured through a unified configuration structure:

```rust
/// Integration configuration trait
pub trait IntegrationConfig: Send + Sync {
    /// Unique identifier for this integration
    fn id(&self) -> &str;
    
    /// Integration type (REST, WebSocket, IPC, etc.)
    fn integration_type(&self) -> IntegrationType;
    
    /// Enable/disable this integration
    fn enabled(&self) -> bool;
    
    /// Maximum concurrent requests
    fn max_concurrency(&self) -> usize;
    
    /// Request timeout in milliseconds
    fn timeout_ms(&self) -> u64;
    
    /// Retry configuration
    fn retry_config(&self) -> RetryConfig;
    
    /// Security configuration
    fn security_config(&self) -> SecurityConfig;
}
```

### 2.5. Error Handling Strategy

The integration framework implements a comprehensive error handling strategy:

1. **Error Classification** - Errors are classified as transient, permanent, or rate-limited
2. **Automatic Retry** - Transient errors are automatically retried with exponential backoff
3. **Circuit Breaking** - Failed integrations are temporarily disabled to prevent cascading failures
4. **Error Logging** - All errors are logged with full context for debugging
5. **User Feedback** - Appropriate error messages are provided to end users

### 2.6. Observability

All integrations provide comprehensive observability:

| Metric | Type | Purpose | Collection Interval |
|---------|------|---------|-------------------|
| **Request Count** | Counter | Total requests processed | 1s |
| **Response Time** | Histogram | Request latency distribution | 1s |
| **Error Rate** | Gauge | Percentage of failed requests | 1s |
| **Active Connections** | Gauge | Current connection count | 1s |
| **Queue Depth** | Gauge | Pending request queue size | 1s |
| **Circuit State** | Gauge | Circuit breaker status | 1s |

### 2.7. Integration Testing

The framework provides built-in testing utilities:

```rust
/// Integration test trait
#[async_trait]
pub trait IntegrationTest {
    /// Setup test environment
    async fn setup(&mut self) -> Result<(), TestError>;
    
    /// Execute test scenario
    async fn execute(&self, scenario: TestScenario) -> Result<TestResult, TestError>;
    
    /// Verify expected outcomes
    async fn verify(&self, result: &TestResult) -> Result<(), TestError>;
    
    /// Teardown test environment
    async fn teardown(&mut self) -> Result<(), TestError>;
}
```

### 2.8. Documentation Standards

All integration interfaces must be documented with:

1. **Purpose Statement** - Clear description of the integration's purpose
2. **Protocol Specification** - Complete protocol documentation (HTTP, WebSocket, etc.)
3. **Request/Response Formats** - Detailed message schemas with examples
4. **Error Codes** - Comprehensive list of error codes and meanings
5. **Security Requirements** - Authentication and authorization requirements
6. **Rate Limits** - Request rate limits and throttling behavior
7. **Examples** - Working code examples in multiple languages
8. **Testing Procedures** - Step-by-step testing instructions

---

## 3. INTEGRATION ARCHITECTURE

### 3.1. Architecture Overview

The Tachyon integration architecture implements a layered approach with clear separation of concerns:

```mermaid
graph TB
    subgraph "Client Layer"
        Desktop[Desktop Application]
        Web[Web Frontend]
        Mobile[Mobile Client]
    end
    
    subgraph "Integration Layer"
        REST[REST API Gateway]
        WS[WebSocket Gateway]
        IPC[IPC Bridge]
        WH[Webhook Dispatcher]
        OA[OAuth Provider]
        PL[Plugin Manager]
    end
    
    subgraph "Service Layer"
        DocSvc[Document Service]
        SearchSvc[Search Service]
        RepoSvc[Repository Service]
        AuthSvc[Auth Service]
    end
    
    subgraph "Data Layer"
        DB[(Database)]
        Cache[(Cache)]
        Git[(Git Repositories)]
        Files[(File System)]
    end
    
    Desktop --> REST
    Web --> REST
    Web --> WS
    Mobile --> REST
    Desktop --> IPC
    REST --> DocSvc
    REST --> SearchSvc
    REST --> RepoSvc
    WS --> DocSvc
    WS --> SearchSvc
    IPC --> DocSvc
    IPC --> RepoSvc
    WH --> External[External Services]
    OA --> AuthSvc
    PL --> DocSvc
    PL --> SearchSvc
    DocSvc --> DB
    DocSvc --> Cache
    DocSvc --> Git
    RepoSvc --> Git
    RepoSvc --> Files
    SearchSvc --> Cache
    AuthSvc --> DB
```

### 3.2. Component Responsibilities

#### 3.2.1. REST API Gateway

**Responsibilities:**
- HTTP/2 request routing and handling
- Request validation and sanitization
- Authentication and authorization enforcement
- Rate limiting and throttling
- Response formatting and compression
- CORS and security headers

**Implementation:**
- Built with Axum web framework
- Hyper HTTP/2 server
- Tokio async runtime
- Middleware pipeline for cross-cutting concerns

#### 3.2.2. WebSocket Gateway

**Responsibilities:**
- WebSocket connection management
- Message routing and broadcasting
- Presence tracking for collaboration
- Connection state synchronization
- Heartbeat and keepalive management

**Implementation:**
- Tungstenite WebSocket library
- Tokio async runtime
- Message serialization with serde_json
- Connection pooling and load balancing

#### 3.2.3. IPC Bridge

**Responsibilities:**
- Command routing between desktop and backend
- Event emission to frontend
- State synchronization
- File system operation delegation
- Native OS integration

**Implementation:**
- Tauri command system
- Type-safe message passing
- Capability-based access control
- Async command execution

#### 3.2.4. Webhook Dispatcher

**Responsibilities:**
- Event queue management
- Webhook delivery with retries
- Signature generation and verification
- Delivery status tracking
- Dead letter queue for failed deliveries

**Implementation:**
- Tokio async task scheduler
- HTTP client with retry logic
- HMAC signature generation
- Persistent queue storage

#### 3.2.5. OAuth Provider

**Responsibilities:**
- OAuth 2.0 flow orchestration
- Token management and refresh
- PKCE implementation for public clients
- Provider-specific implementations
- Session management

**Implementation:**
- OAuth2 client library
- Secure token storage
- Provider adapters (Google, GitHub, etc.)
- CSRF protection

#### 3.2.6. Plugin Manager

**Responsibilities:**
- Plugin discovery and loading
- Plugin lifecycle management
- Sandboxed execution environment
- Plugin API exposure
- Resource isolation

**Implementation:**
- WASM runtime for plugins
- Capability-based permission system
- Plugin manifest validation
- Resource monitoring and limits

### 3.3. Data Flow Architecture

#### 3.3.1. Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Gateway
    participant Middleware
    participant Service
    participant Data
    
    Client->>Gateway: HTTP/2 Request
    Gateway->>Middleware: Request Processing
    Middleware->>Middleware: Authentication
    Middleware->>Middleware: Authorization
    Middleware->>Middleware: Rate Limiting
    Middleware->>Middleware: Validation
    Middleware->>Service: Validated Request
    Service->>Data: Query/Operation
    Data-->>Service: Result
    Service-->>Middleware: Response
    Middleware->>Middleware: Response Formatting
    Middleware->>Middleware: Compression
    Gateway-->>Client: HTTP/2 Response
```

#### 3.3.2. Event Flow (Webhook)

```mermaid
sequenceDiagram
    participant System
    participant EventQueue
    participant Dispatcher
    participant Webhook
    participant External
    
    System->>EventQueue: Emit Event
    EventQueue->>Dispatcher: Dequeue Event
    Dispatcher->>Dispatcher: Generate Signature
    Dispatcher->>Webhook: POST with Signature
    alt Delivery Success
        Webhook-->>Dispatcher: 200 OK
        Dispatcher->>EventQueue: Mark Delivered
    else Delivery Failure
        Webhook-->>Dispatcher: Error
        Dispatcher->>EventQueue: Schedule Retry
    end
```

### 3.4. Security Architecture

#### 3.4.1. Defense in Depth

The integration architecture implements defense-in-depth security:

1. **Network Layer** - TLS 1.3 encryption, certificate pinning
2. **Authentication Layer** - JWT tokens, OAuth 2.0, capability-based access
3. **Authorization Layer** - Role-based access control, permission checks
4. **Input Validation Layer** - Schema validation, sanitization, length limits
5. **Output Encoding Layer** - JSON encoding, HTML escaping, XSS prevention
6. **Audit Layer** - Comprehensive logging, security event tracking

#### 3.4.2. Authentication Mechanisms

| Integration Type | Authentication Method | Token Format | Lifetime |
|----------------|---------------------|---------------|------------|
| **REST API** | JWT Bearer Token | JWS signed with RS256 | 1 hour (refreshable) |
| **WebSocket** | JWT Bearer Token | JWS signed with RS256 | Connection duration |
| **IPC** | Capability Token | Capability assertion | Session duration |
| **Webhook** | HMAC Signature | SHA-256 HMAC | Per request |
| **OAuth** | OAuth 2.0 | Access Token + Refresh Token | 1 hour / 30 days |
| **Plugin** | Sandboxed | No direct auth | Plugin lifetime |

### 3.5. Performance Architecture

#### 3.5.1. Performance Targets

| Metric | Target | Measurement Method |
|---------|---------|------------------|
| **REST API Latency** | P50: 50ms, P95: 100ms, P99: 200ms | Request timing |
| **WebSocket Message Latency** | P50: 20ms, P95: 50ms, P99: 100ms | Message timing |
| **IPC Command Latency** | P50: 5ms, P95: 10ms, P99: 20ms | Command timing |
| **Webhook Delivery** | P50: 200ms, P95: 500ms, P99: 1000ms | Delivery timing |
| **OAuth Flow** | P50: 500ms, P95: 1000ms, P99: 2000ms | Flow timing |

#### 3.5.2. Caching Strategy

The integration architecture implements multi-level caching:

1. **Response Cache** - Cache GET requests with TTL
2. **Connection Pool** - Reuse HTTP/2 connections
3. **WebSocket Pool** - Maintain connection pools for servers
4. **Query Cache** - Cache database query results
5. **Compute Cache** - Cache expensive computations

### 3.6. Resilience Architecture

#### 3.6.1. Circuit Breaker Pattern

```mermaid
stateDiagram-v2
    [*] --> Closed: Normal Operation
    Closed --> Open: Failure Threshold Exceeded
    Open --> HalfOpen: Timeout Elapsed
    HalfOpen --> Closed: Success Threshold Met
    HalfOpen --> Open: Failure Threshold Exceeded
    HalfOpen --> Closed: Timeout Elapsed
```

**Circuit Breaker States:**
- **Closed** - Normal operation, requests pass through
- **Open** - Circuit tripped, requests fail fast
- **Half-Open** - Testing if service recovered

#### 3.6.2. Retry Strategy

```rust
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    
    /// Initial backoff delay in milliseconds
    pub initial_backoff_ms: u64,
    
    /// Maximum backoff delay in milliseconds
    pub max_backoff_ms: u64,
    
    /// Backoff multiplier (exponential)
    pub backoff_multiplier: f64,
    
    /// Jitter factor for randomized backoff
    pub jitter_factor: f64,
    
    /// Retryable error predicates
    pub retryable_errors: Vec<ErrorPredicate>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retryable_errors: vec![
                ErrorPredicate::Transient,
                ErrorPredicate::Timeout,
                ErrorPredicate::RateLimited,
            ],
        }
    }
}
```

### 3.7. Integration Governance

#### 3.7.1. API Versioning

All integrations implement semantic versioning:

- **Major Version** - Breaking changes, requires client update
- **Minor Version** - Backward-compatible additions
- **Patch Version** - Backward-compatible bug fixes

**Versioning Strategy:**
- URL path versioning: `/api/v1/documents`
- Header versioning: `Accept: application/vnd.tachyon.v1+json`
- Content negotiation: Client specifies preferred version

#### 3.7.2. Deprecation Policy

1. **Deprecation Notice** - 6 months before removal
2. **Warning Headers** - `Deprecation: true, Sunset: 2026-12-31`
3. **Migration Guide** - Documentation for migrating to new version
4. **Grace Period** - 3 months after deprecation before removal

---

---

## 4. REST INTEGRATION

### 4.1. REST API Overview

The Tachyon REST API provides a comprehensive HTTP/2-based interface for client-server communication. The API follows RESTful principles with resource-oriented endpoints, standard HTTP methods, and proper status codes.

**Design Principles:**

1. **Resource-Oriented** - URLs represent resources, not actions
2. **HTTP Method Semantics** - Proper use of GET, POST, PUT, PATCH, DELETE
3. **Stateless** - Each request contains all necessary information
4. **Cacheable** - Responses include cache control headers
5. **Uniform Interface** - Consistent response formats across endpoints
6. **Hypermedia-Driven** - Responses include links to related resources

### 4.2. API Specification

#### 4.2.1. Base URL

```
Production: https://api.tachyon.io/v1
Staging: https://api-staging.tachyon.io/v1
Development: http://localhost:8080/v1
```

#### 4.2.2. Authentication

All API endpoints require authentication via JWT Bearer token:

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Token Format:**
- Algorithm: RS256 (RSASSA-PKCS1-v1_5 using SHA-256)
- Issuer: Tachyon API
- Audience: Tachyon API
- Expiration: 1 hour from issuance

#### 4.2.3. Request Headers

| Header | Required | Description | Example |
|--------|-----------|-------------|---------|
| **Authorization** | Yes | JWT Bearer token | `Bearer eyJhbGci...` |
| **Content-Type** | Conditional | Request content type | `application/json` |
| **Accept** | No | Preferred response format | `application/json` |
| **Accept-Encoding** | No | Preferred compression | `gzip, deflate` |
| **User-Agent** | Recommended | Client identification | `TachyonDesktop/1.0.0` |
| **X-Request-ID** | Recommended | Request tracing | `550e8400-e29b-41d4-a716-446655440e00` |
| **X-Client-Version** | Recommended | Client version | `1.0.0` |

#### 4.2.4. Response Headers

| Header | Description | Example |
|--------|-------------|---------|
| **Content-Type** | Response content type | `application/json` |
| **Content-Encoding** | Compression algorithm | `gzip` |
| **Cache-Control** | Caching directives | `max-age=3600, private` |
| **ETag** | Entity tag for caching | `"33a64df551425fcc55e4d42a148795d9e25"` |
| **X-Request-ID** | Request tracing | `550e8400-e29b-41d4-a716-446655440e00` |
| **X-RateLimit-Remaining** | Remaining requests | `99` |
| **X-RateLimit-Reset** | Rate limit reset time | `1641234567` |
| **X-Response-Time** | Response time in ms | `45` |

### 4.3. Document Endpoints

#### 4.3.1. List Documents

```http
GET /api/v1/documents
```

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|-----------|---------|-------------|
| **page** | integer | No | 1 | Page number (1-indexed) |
| **per_page** | integer | No | 20 | Items per page (max: 100) |
| **sort** | string | No | created_at | Sort field |
| **order** | string | No | desc | Sort order (asc, desc) |
| **search** | string | No | - | Search query |
| **tag** | string | No | - | Filter by tag |

**Response (200 OK):**

```json
{
  "data": [
    {
      "id": "doc_abc123",
      "title": "Example Document",
      "content": "# Example\n\nThis is an example.",
      "metadata": {
        "created_at": "2026-02-07T22:00:00Z",
        "updated_at": "2026-02-07T22:30:00Z",
        "author": "user@example.com",
        "tags": ["example", "documentation"]
      },
      "links": {
        "self": "/api/v1/documents/doc_abc123",
        "update": "/api/v1/documents/doc_abc123",
        "delete": "/api/v1/documents/doc_abc123"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 45,
    "total_pages": 3,
    "links": {
      "first": "/api/v1/documents?page=1",
      "last": "/api/v1/documents?page=3",
      "next": "/api/v1/documents?page=2",
      "prev": null
    }
  }
}
```

#### 4.3.2. Get Document

```http
GET /api/v1/documents/{id}
```

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| **id** | string | Yes | Document ID |

**Response (200 OK):**

```json
{
  "data": {
    "id": "doc_abc123",
    "title": "Example Document",
    "content": "# Example\n\nThis is an example.",
    "metadata": {
      "created_at": "2026-02-07T22:00:00Z",
      "updated_at": "2026-02-07T22:30:00Z",
      "author": "user@example.com",
      "tags": ["example", "documentation"]
    },
    "links": {
      "self": "/api/v1/documents/doc_abc123",
      "update": "/api/v1/documents/doc_abc123",
      "delete": "/api/v1/documents/doc_abc123"
    }
  }
}
```

**Response (404 Not Found):**

```json
{
  "error": {
    "code": "DOCUMENT_NOT_FOUND",
    "message": "Document with id 'doc_abc123' not found",
    "details": {
      "document_id": "doc_abc123"
    },
    "request_id": "550e8400-e29b-41d4-a716-446655440e00"
  }
}
```

#### 4.3.3. Create Document

```http
POST /api/v1/documents
```

**Request Body:**

```json
{
  "title": "New Document",
  "content": "# New Document\n\nThis is a new document.",
  "tags": ["new", "example"],
  "metadata": {
    "custom_field": "custom_value"
  }
}
```

**Validation Rules:**

| Field | Validation | Error Code |
|-------|-------------|------------|
| **title** | 1-100 characters, required | `TITLE_INVALID` |
| **content** | Max 10MB, required | `CONTENT_TOO_LARGE` |
| **tags** | Max 10 tags, max 50 chars each | `TAGS_INVALID` |

**Response (201 Created):**

```json
{
  "data": {
    "id": "doc_xyz789",
    "title": "New Document",
    "content": "# New Document\n\nThis is a new document.",
    "metadata": {
      "created_at": "2026-02-07T22:35:00Z",
      "updated_at": "2026-02-07T22:35:00Z",
      "author": "user@example.com",
      "tags": ["new", "example"]
    },
    "links": {
      "self": "/api/v1/documents/doc_xyz789",
      "update": "/api/v1/documents/doc_xyz789",
      "delete": "/api/v1/documents/doc_xyz789"
    }
  }
}
```

**Response (400 Bad Request):**

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": {
      "fields": [
        {
          "field": "title",
          "message": "Title is required"
        }
      ]
    },
    "request_id": "550e8400-e29b-41d4-a716-446655440e00"
  }
}
```

#### 4.3.4. Update Document

```http
PATCH /api/v1/documents/{id}
```

**Request Body (partial update):**

```json
{
  "title": "Updated Title",
  "content": "# Updated Content\n\nThis is updated."
}
```

**Response (200 OK):**

```json
{
  "data": {
    "id": "doc_abc123",
    "title": "Updated Title",
    "content": "# Updated Content\n\nThis is updated.",
    "metadata": {
      "created_at": "2026-02-07T22:00:00Z",
      "updated_at": "2026-02-07T22:40:00Z",
      "author": "user@example.com",
      "tags": ["example", "documentation"]
    },
    "links": {
      "self": "/api/v1/documents/doc_abc123",
      "update": "/api/v1/documents/doc_abc123",
      "delete": "/api/v1/documents/doc_abc123"
    }
  }
}
```

#### 4.3.5. Delete Document

```http
DELETE /api/v1/documents/{id}
```

**Response (204 No Content):**

Empty response body with status code 204.

### 4.4. Search Endpoints

#### 4.4.1. Search Documents

```http
GET /api/v1/search
```

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|-----------|---------|-------------|
| **q** | string | Yes | - | Search query |
| **page** | integer | No | 1 | Page number |
| **per_page** | integer | No | 20 | Items per page |
| **fields** | array | No | all | Fields to search (title, content, tags) |
| **fuzzy** | boolean | No | true | Enable fuzzy search |

**Response (200 OK):**

```json
{
  "data": [
    {
      "id": "doc_abc123",
      "title": "Example Document",
      "snippet": "...This is an example...",
      "score": 0.95,
      "metadata": {
        "created_at": "2026-02-07T22:00:00Z",
        "author": "user@example.com",
        "tags": ["example", "documentation"]
      },
      "links": {
        "self": "/api/v1/documents/doc_abc123"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 12,
    "total_pages": 1,
    "links": {
      "first": "/api/v1/search?q=example&page=1",
      "last": "/api/v1/search?q=example&page=1"
    }
  }
}
```

### 4.5. Authentication Endpoints

#### 4.5.1. Login

```http
POST /api/v1/auth/login
```

**Request Body:**

```json
{
  "email": "user@example.com",
  "password": "secure_password_123",
  "mfa_code": null
}
```

**Response (200 OK):**

```json
{
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
      "id": "usr_123",
      "email": "user@example.com",
      "name": "User Name"
    }
  }
}
```

#### 4.5.2. Refresh Token

```http
POST /api/v1/auth/refresh
```

**Request Body:**

```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (200 OK):**

```json
{
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600
  }
}
```

### 4.6. Error Codes

| Code | HTTP Status | Description | Retryable |
|------|-------------|-------------|-----------|
| **VALIDATION_ERROR** | 400 | Request validation failed | No |
| **UNAUTHORIZED** | 401 | Authentication required | No |
| **FORBIDDEN** | 403 | Insufficient permissions | No |
| **NOT_FOUND** | 404 | Resource not found | No |
| **CONFLICT** | 409 | Resource conflict | Yes |
| **RATE_LIMITED** | 429 | Rate limit exceeded | Yes |
| **INTERNAL_ERROR** | 500 | Internal server error | Yes |
| **SERVICE_UNAVAILABLE** | 503 | Service temporarily unavailable | Yes |

### 4.7. Rate Limiting

**Rate Limits:**

| Tier | Requests | Window | Burst |
|------|----------|--------|-------|
| **Free** | 100 | 1 hour | 10 |
| **Pro** | 1000 | 1 hour | 50 |
| **Enterprise** | 10000 | 1 hour | 200 |

**Rate Limit Headers:**

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1641234567
Retry-After: 60
```

### 4.8. Client Implementation Example

#### 4.8.1. Rust Client

```rust
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct CreateDocumentRequest {
    title: String,
    content: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: T,
}

async fn create_document(
    client: &Client,
    token: &str,
    request: CreateDocumentRequest,
) -> Result<Document, Box<dyn std::error::Error>> {
    let response = client
        .post("https://api.tachyon.io/v1/documents")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<Document> = response.json().await?;
        Ok(api_response.data)
    } else {
        Err(format!("API request failed: {}", response.status()).into())
    }
}
```

#### 4.8.2. TypeScript Client

```typescript
interface Document {
  id: string;
  title: string;
  content: string;
  metadata: {
    created_at: string;
    updated_at: string;
    author: string;
    tags: string[];
  };
}

interface ApiResponse<T> {
  data: T;
}

class TachyonApiClient {
  private baseUrl: string;
  private token: string;

  constructor(baseUrl: string, token: string) {
    this.baseUrl = baseUrl;
    this.token = token;
  }

  async createDocument(
    title: string,
    content: string,
    tags: string[] = []
  ): Promise<Document> {
    const response = await fetch(`${this.baseUrl}/documents`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ title, content, tags }),
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status}`);
    }

    const apiResponse: ApiResponse<Document> = await response.json();
    return apiResponse.data;
  }

  async getDocument(id: string): Promise<Document> {
    const response = await fetch(`${this.baseUrl}/documents/${id}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${this.token}`,
      },
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status}`);
    }

    const apiResponse: ApiResponse<Document> = await response.json();
    return apiResponse.data;
  }
}
```

---

---

## 5. WEBSOCKET INTEGRATION

### 5.1. WebSocket Overview

The Tachyon WebSocket integration provides real-time bidirectional communication for collaborative features including live document editing, presence tracking, and instant notifications. The WebSocket protocol (RFC 6455) enables low-latency communication over a single TCP connection with full-duplex messaging.

**Design Principles:**

1. **Event-Driven** - All communication is event-based with typed messages
2. **State Synchronization** - Automatic state synchronization across clients
3. **Presence Tracking** - Real-time user presence and cursor tracking
4. **Conflict Resolution** - Operational transformation for concurrent edits
5. **Graceful Degradation** - Fallback to polling when WebSocket unavailable

### 5.2. Connection Protocol

#### 5.2.1. Connection URL

```
Production: wss://api.tachyon.io/v1/ws
Staging: wss://api-staging.tachyon.io/v1/ws
Development: ws://localhost:8080/v1/ws
```

#### 5.2.2. Connection Handshake

```http
GET /v1/ws HTTP/1.1
Host: api.tachyon.io
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZzFubE=
Sec-WebSocket-Version: 13
Sec-WebSocket-Protocol: tachyon-v1
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Server Response (101 Switching Protocols):**

```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9yXRyRA==
Sec-WebSocket-Protocol: tachyon-v1
```

### 5.3. Message Format

#### 5.3.1. Message Envelope

All WebSocket messages use a common envelope format:

```json
{
  "type": "document_update",
  "id": "msg_abc123",
  "timestamp": "2026-02-07T22:45:00.123Z",
  "data": {
    "document_id": "doc_xyz789",
    "changes": [...]
  }
}
```

**Envelope Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| **type** | string | Yes | Message type identifier |
| **id** | string | Yes | Unique message ID |
| **timestamp** | ISO 8601 | Yes | Message generation time |
| **data** | object | Yes | Message payload |

#### 5.3.2. Message Types

| Message Type | Direction | Description |
|--------------|-----------|-------------|
| **auth** | Client → Server | Authentication message |
| **document_update** | Bidirectional | Document content update |
| **cursor_move** | Bidirectional | Cursor position update |
| **presence** | Server → Client | User presence notification |
| **typing_indicator** | Bidirectional | Typing indicator |
| **error** | Server → Client | Error notification |
| **ping** | Bidirectional | Heartbeat ping |
| **pong** | Bidirectional | Heartbeat pong |

### 5.4. Authentication

#### 5.4.1. Auth Message

**Client → Server:**

```json
{
  "type": "auth",
  "id": "auth_001",
  "timestamp": "2026-02-07T22:45:00.123Z",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

**Server → Client (Success):**

```json
{
  "type": "auth_success",
  "id": "auth_001",
  "timestamp": "2026-02-07T22:45:00.456Z",
  "data": {
    "user_id": "usr_123",
    "session_id": "ses_456"
  }
}
```

**Server → Client (Failure):**

```json
{
  "type": "auth_error",
  "id": "auth_001",
  "timestamp": "2026-02-07T22:45:00.789Z",
  "data": {
    "code": "INVALID_TOKEN",
    "message": "Authentication token is invalid or expired"
  }
}
```

### 5.5. Document Collaboration

#### 5.5.1. Document Update Message

**Client → Server:**

```json
{
  "type": "document_update",
  "id": "msg_002",
  "timestamp": "2026-02-07T22:45:05.123Z",
  "data": {
    "document_id": "doc_xyz789",
    "version": 5,
    "operation": "insert",
    "position": {
      "line": 10,
      "column": 5
    },
    "content": "New text",
    "length": 8
  }
}
```

**Server → Client (Broadcast):**

```json
{
  "type": "document_update",
  "id": "msg_002",
  "timestamp": "2026-02-07T22:45:05.234Z",
  "data": {
    "document_id": "doc_xyz789",
    "version": 5,
    "operation": "insert",
    "user_id": "usr_456",
    "position": {
      "line": 10,
      "column": 5
    },
    "content": "New text",
    "length": 8
  }
}
```

#### 5.5.2. Cursor Move Message

**Client → Server:**

```json
{
  "type": "cursor_move",
  "id": "msg_003",
  "timestamp": "2026-02-07T22:45:10.123Z",
  "data": {
    "document_id": "doc_xyz789",
    "position": {
      "line": 15,
      "column": 20
    },
    "selection": {
      "start": {
        "line": 15,
        "column": 20
      },
      "end": {
        "line": 15,
        "column": 25
      }
    }
  }
}
```

**Server → Client (Broadcast):**

```json
{
  "type": "cursor_move",
  "id": "msg_003",
  "timestamp": "2026-02-07T22:45:10.456Z",
  "data": {
    "document_id": "doc_xyz789",
    "user_id": "usr_789",
    "position": {
      "line": 15,
      "column": 20
    },
    "selection": {
      "start": {
        "line": 15,
        "column": 20
      },
      "end": {
        "line": 15,
        "column": 25
      }
    }
  }
}
```

### 5.6. Presence Tracking

#### 5.6.1. Presence Message

**Server → Client:**

```json
{
  "type": "presence",
  "id": "msg_004",
  "timestamp": "2026-02-07T22:45:15.123Z",
  "data": {
    "document_id": "doc_xyz789",
    "users": [
      {
        "user_id": "usr_123",
        "name": "Alice",
        "status": "active",
        "cursor": {
          "line": 10,
          "column": 5
        },
        "last_seen": "2026-02-07T22:45:15.000Z"
      },
      {
        "user_id": "usr_456",
        "name": "Bob",
        "status": "idle",
        "cursor": null,
        "last_seen": "2026-02-07T22:44:30.000Z"
      }
    ]
  }
}
```

**User Status Values:**

| Status | Description |
|--------|-------------|
| **active** | User is actively editing |
| **idle** | User is viewing but not editing |
| **away** | User has been inactive for > 5 minutes |
| **offline** | User has disconnected |

### 5.7. Heartbeat Mechanism

#### 5.7.1. Ping Message

**Bidirectional:**

```json
{
  "type": "ping",
  "id": "msg_005",
  "timestamp": "2026-02-07T22:45:20.000Z",
  "data": {
    "sequence": 12345
  }
}
```

#### 5.7.2. Pong Message

**Bidirectional:**

```json
{
  "type": "pong",
  "id": "msg_005",
  "timestamp": "2026-02-07T22:45:20.050Z",
  "data": {
    "sequence": 12345,
    "server_time": "2026-02-07T22:45:20.025Z"
  }
}
```

**Heartbeat Configuration:**

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Interval** | 30 seconds | Ping/pong interval |
| **Timeout** | 60 seconds | Connection timeout |
| **Missed Pings** | 3 | Max missed pings before disconnect |

### 5.8. Error Handling

#### 5.8.1. Error Message

**Server → Client:**

```json
{
  "type": "error",
  "id": "msg_006",
  "timestamp": "2026-02-07T22:45:25.000Z",
  "data": {
    "code": "RATE_LIMITED",
    "message": "Message rate limit exceeded",
    "retry_after": "2026-02-07T22:46:00.000Z"
  }
}
```

**Error Codes:**

| Code | Description | Retryable |
|------|-------------|-----------|
| **INVALID_MESSAGE** | Malformed message | No |
| **UNAUTHORIZED** | Authentication required | No |
| **FORBIDDEN** | Insufficient permissions | No |
| **DOCUMENT_NOT_FOUND** | Document does not exist | No |
| **RATE_LIMITED** | Rate limit exceeded | Yes |
| **CONFLICT** | Version conflict | Yes |
| **INTERNAL_ERROR** | Server error | Yes |
| **TIMEOUT** | Operation timeout | Yes |

### 5.9. Client Implementation

#### 5.9.1. Rust Client (Tungstenite)

```rust
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(Debug, Serialize, Deserialize)]
struct MessageEnvelope<T> {
    #[serde(rename = "type")]
    message_type: String,
    id: String,
    timestamp: String,
    data: T,
}

#[derive(Debug, Serialize)]
struct AuthMessage {
    token: String,
}

#[derive(Debug, Deserialize)]
struct AuthSuccess {
    user_id: String,
    session_id: String,
}

pub async fn connect_websocket(
    token: &str,
) -> Result<WebSocketStream<tokio_tungstenite::MaybeTlsStream>, Box<dyn std::error::Error>> {
    let url = url::Url::parse("wss://api.tachyon.io/v1/ws")?;
    let request = tungstenite::handshake::client::Request::from_url(url)?;
    
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
    
    // Send authentication
    let auth_msg = MessageEnvelope {
        message_type: "auth".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        data: AuthMessage {
            token: token.to_string(),
        },
    };
    ws_stream.send(Message::Text(json!(auth_msg).to_string())).await?;
    
    // Handle incoming messages
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if let Ok(envelope) = serde_json::from_str::<MessageEnvelope<serde_json::Value>>(&text) {
                    match envelope.message_type.as_str() {
                        "document_update" => {
                            // Handle document update
                        }
                        "presence" => {
                            // Handle presence update
                        }
                        _ => {
                            eprintln!("Unknown message type: {}", envelope.message_type);
                        }
                    }
                }
            }
            Ok(Message::Close(close_frame)) => {
                    eprintln!("WebSocket closed: {:?}", close_frame);
                    break;
            }
            Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
            }
            _ => {}
        }
    }
    
    Ok(ws_stream)
}
```

#### 5.9.2. JavaScript Client

```typescript
interface MessageEnvelope<T> {
  type: string;
  id: string;
  timestamp: string;
  data: T;
}

interface AuthMessage {
  token: string;
}

interface AuthSuccess {
  user_id: string;
  session_id: string;
}

class TachyonWebSocketClient {
  private ws: WebSocket | null = null;
  private token: string;
  private messageHandlers: Map<string, (data: any) => void> = new Map();

  constructor(url: string, token: string) {
    this.token = token;
    this.connect(url);
  }

  private connect(url: string): void {
    this.ws = new WebSocket(url);
    
    this.ws.onopen = () => {
      this.sendAuth();
    };
    
    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const envelope: MessageEnvelope<any> = JSON.parse(event.data);
        this.handleMessage(envelope);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };
    
    this.ws.onerror = (error: Event) => {
      console.error('WebSocket error:', error);
    };
    
    this.ws.onclose = (event: CloseEvent) => {
      console.log('WebSocket closed:', event);
      // Attempt reconnection after delay
      setTimeout(() => this.connect(url), 5000);
    };
  }

  private sendAuth(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const authMessage: MessageEnvelope<AuthMessage> = {
        type: 'auth',
        id: this.generateId(),
        timestamp: new Date().toISOString(),
        data: { token: this.token },
      };
      this.ws.send(JSON.stringify(authMessage));
    }
  }

  private handleMessage(envelope: MessageEnvelope<any>): void {
    const handler = this.messageHandlers.get(envelope.type);
    if (handler) {
      handler(envelope.data);
    }
  }

  public on<T>(messageType: string, handler: (data: T) => void): void {
    this.messageHandlers.set(messageType, handler);
  }

  public send<T>(messageType: string, data: T): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const envelope: MessageEnvelope<T> = {
        type: messageType,
        id: this.generateId(),
        timestamp: new Date().toISOString(),
        data,
      };
      this.ws.send(JSON.stringify(envelope));
    }
  }

  private generateId(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  public disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
```

---

---

## 6. IPC INTEGRATION

### 6.1. IPC Overview

The Tachyon IPC (Inter-Process Communication) integration enables communication between the Tauri desktop frontend and the Rust backend services. The IPC layer provides type-safe, capability-based communication with built-in security and performance optimizations.

**Design Principles:**

1. **Type Safety** - Strongly typed messages with compile-time validation
2. **Capability-Based Security** - Fine-grained permissions for each operation
3. **Async/Await** - Non-blocking communication with Promise-based API
4. **Event-Driven** - Bidirectional event system for real-time updates
5. **Error Isolation** - Errors don't crash the entire application

### 6.2. IPC Architecture

```mermaid
graph TB
    subgraph "Frontend (Tauri)"
        WebView[WebView Frontend]
        IPC[Tauri IPC Bridge]
    end
    
    subgraph "Backend (Rust)"
        CommandHandlers[Command Handlers]
        EventEmitters[Event Emitters]
        Services[Business Services]
    end
    
    WebView --> IPC
    IPC --> CommandHandlers
    CommandHandlers --> Services
    Services --> EventEmitters
    EventEmitters --> IPC
    IPC --> WebView
    
    style WebView fill:#e1f5fe
    style IPC fill:#95e1d3
    style CommandHandlers fill:#f9ca24
    style EventEmitters fill:#f9ca24
    style Services fill:#95e1d3
```

### 6.3. Command System

#### 6.3.1. Command Registration

Commands are registered in the Tauri backend using the `#[tauri::command]` attribute:

```rust
use tauri::State;
use serde::{Deserialize, Serialize};

/// Document command trait
#[tauri::command]
pub async fn get_document(
    state: State<'_>,
    id: String,
) -> Result<Document, String> {
    let app_state = state.lock().await;
    
    match app_state.document_service.get_document(&id).await {
        Ok(doc) => Ok(doc),
        Err(e) => Err(format!("Failed to get document: {}", e)),
    }
}

/// Create document command
#[tauri::command]
pub async fn create_document(
    state: State<'_>,
    title: String,
    content: String,
) -> Result<Document, String> {
    let app_state = state.lock().await;
    
    match app_state.document_service.create_document(title, content).await {
        Ok(doc) => Ok(doc),
        Err(e) => Err(format!("Failed to create document: {}", e)),
    }
}

/// Update document command
#[tauri::command]
pub async fn update_document(
    state: State<'_>,
    id: String,
    title: Option<String>,
    content: Option<String>,
) -> Result<Document, String> {
    let app_state = state.lock().await;
    
    match app_state.document_service.update_document(&id, title, content).await {
        Ok(doc) => Ok(doc),
        Err(e) => Err(format!("Failed to update document: {}", e)),
    }
}
```

#### 6.3.2. Command Invocation (Frontend)

```typescript
import { invoke } from '@tauri-apps/api';

interface Document {
  id: string;
  title: string;
  content: string;
}

// Get document
export async function getDocument(id: string): Promise<Document> {
  return await invoke<Document>('get_document', { id });
}

// Create document
export async function createDocument(
  title: string,
  content: string
): Promise<Document> {
  return await invoke<Document>('create_document', { title, content });
}

// Update document
export async function updateDocument(
  id: string,
  title?: string,
  content?: string
): Promise<Document> {
  return await invoke<Document>('update_document', { id, title, content });
}
```

### 6.4. Event System

#### 6.4.1. Event Emission (Backend)

```rust
use tauri::Event;

/// Emit document update event
pub async fn emit_document_update(
    document_id: &str,
    changes: Vec<Change>,
) -> Result<(), String> {
    let payload = serde_json::to_string(&DocumentUpdateEvent {
        document_id: document_id.to_string(),
        changes,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }).map_err(|e| e.to_string())?;
    
    Event::emit("document-update", payload).map_err(|e| e.to_string())
}

/// Emit error event
pub async fn emit_error(
    code: &str,
    message: &str,
) -> Result<(), String> {
    let payload = serde_json::to_string(&ErrorEvent {
        code: code.to_string(),
        message: message.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }).map_err(|e| e.to_string())?;
    
    Event::emit("error", payload).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct DocumentUpdateEvent {
    document_id: String,
    changes: Vec<Change>,
    timestamp: String,
}

#[derive(Serialize)]
struct ErrorEvent {
    code: String,
    message: String,
    timestamp: String,
}
```

#### 6.4.2. Event Listening (Frontend)

```typescript
import { listen } from '@tauri-apps/api';

interface DocumentUpdateEvent {
  document_id: string;
  changes: Change[];
  timestamp: string;
}

interface ErrorEvent {
  code: string;
  message: string;
  timestamp: string;
}

// Listen to document updates
export function listenToDocumentUpdates(callback: (event: DocumentUpdateEvent) => void): () => {
  const unlisten = listen<DocumentUpdateEvent>('document-update', (event) => {
    callback(event);
  });
  
  return unlisten;
}

// Listen to errors
export function listenToErrors(callback: (event: ErrorEvent) => void): () => {
  const unlisten = listen<ErrorEvent>('error', (event) => {
    callback(event);
  });
  
  return unlisten;
}
```

### 6.5. Capability System

#### 6.5.1. Capability Definition

Capabilities are defined in the Tauri capabilities file (`src-tauri/capabilities/default.json`):

```json
{
  "identifier": "default",
  "description": "Default capabilities for Tachyon desktop application",
  "windows": ["main"],
  "permissions": [
    {
      "identifier": "fs:read",
      "allow": [
        { "path": "$HOME/Documents/**" },
        { "path": "$HOME/.tachyon/**" }
      ]
    },
    {
      "identifier": "fs:write",
      "allow": [
        { "path": "$HOME/Documents/**" },
        { "path": "$HOME/.tachyon/**" }
      ]
    },
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$HOME/Documents" },
        { "path": "$HOME/.tachyon" }
      ]
    },
    {
      "identifier": "http:allow-request",
      "allow": [
        { "url": "https://api.tachyon.io/**" }
      ]
    },
    {
      "identifier": "dialog:allow-open",
      "allow": true
    },
    {
      "identifier": "dialog:allow-save",
      "allow": true
    },
    {
      "identifier": "notification:allow-send",
      "allow": true
    }
  ]
}
```

#### 6.5.2. Capability Categories

| Category | Capabilities | Purpose |
|----------|-------------|---------|
| **File System** | fs:read, fs:write, fs:scope | File access |
| **Network** | http:allow-request | API requests |
| **Dialog** | dialog:allow-open, dialog:allow-save | Native dialogs |
| **Notification** | notification:allow-send | System notifications |
| **Shell** | shell:allow-execute | Command execution |

### 6.6. File System Integration

#### 6.6.1. Read File Command

```rust
use std::path::PathBuf;
use tokio::fs;

#[tauri::command]
pub async fn read_file(
    path: String,
) -> Result<String, String> {
    // Validate path is within allowed scope
    let path_buf = PathBuf::from(&path);
    
    // Check if path is allowed
    if !is_path_allowed(&path_buf) {
        return Err("Path is not allowed".to_string());
    }
    
    // Read file
    match tokio::fs::read_to_string(&path_buf).await {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

fn is_path_allowed(path: &PathBuf) -> bool {
    // Check against allowed paths
    let home = dirs::home_dir().unwrap_or_default();
    let documents_dir = home.join("Documents");
    let tachyon_dir = home.join(".tachyon");
    
    path.starts_with(&documents_dir) || path.starts_with(&tachyon_dir)
}
```

#### 6.6.2. Write File Command

```rust
#[tauri::command]
pub async fn write_file(
    path: String,
    content: String,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    
    if !is_path_allowed(&path_buf) {
        return Err("Path is not allowed".to_string());
    }
    
    // Ensure parent directory exists
    if let Some(parent) = path_buf.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    
    // Write file
    tokio::fs::write(&path_buf, content).await
        .map_err(|e| format!("Failed to write file: {}", e).to_string())
}
```

### 6.7. Git Integration

#### 6.7.1. Git Status Command

```rust
use git2::{Repository, StatusOptions};

#[tauri::command]
pub async fn git_status(
    path: String,
) -> Result<GitStatus, String> {
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    
    let statuses = repo.statuses(Some(&mut opts))
        .map_err(|e| format!("Failed to get git status: {}", e))?;
    
    Ok(GitStatus {
        path: path.clone(),
        statuses: statuses.iter().map(|s| GitFileStatus {
            path: s.path().to_string_lossy(),
            status: format!("{:?}", s.status()),
        }).collect(),
    })
}

#[derive(Serialize)]
struct GitStatus {
    path: String,
    statuses: Vec<GitFileStatus>,
}

#[derive(Serialize)]
struct GitFileStatus {
    path: String,
    status: String,
}
```

#### 6.7.2. Git Commit Command

```rust
#[tauri::command]
pub async fn git_commit(
    path: String,
    message: String,
) -> Result<String, String> {
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;
    
    // Stage all changes
    index.update_all(None, None)
        .map_err(|e| format!("Failed to stage changes: {}", e))?;
    
    // Create commit
    let oid = repo.commit(
        &message,
        None, // author
        None, // signature
        None, // commit time
        false, // allow empty commit
    ).map_err(|e| format!("Failed to commit: {}", e))?;
    
    Ok(oid.to_string())
}
```

### 6.8. Error Handling

#### 6.8.1. Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Command failed: {0}")]
    CommandFailed(String),
    
    #[error("Event emission failed: {0}")]
    EventEmissionFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl From<IpcError> for String {
    fn from(error: IpcError) -> Self {
        error.to_string()
    }
}
```

#### 6.8.2. Error Propagation

```typescript
// Frontend error handling
export async function safeInvoke<T>(
  command: string,
  args: Record<string, any> = {}
): Promise<T> {
  try {
    const result = await invoke<T>(command, args);
    return { success: true, data: result };
  } catch (error) {
    console.error(`IPC command failed: ${command}`, error);
    
    // Emit error event
    emitError('IPC_ERROR', {
      command,
      error: error instanceof Error ? error.message : String(error),
      timestamp: new Date().toISOString(),
    });
    
    return { 
      success: false, 
      error: error instanceof Error ? error.message : String(error) 
    };
  }
}

interface InvokeResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}
```

---

---

## 7. WEBHOOK INTEGRATION

### 7.1. Webhook Overview

The Tachyon webhook integration provides event-driven notifications to external services when specific events occur within the system. Webhooks enable integration with third-party services, automation tools, and external systems.

**Design Principles:**

1. **Event-Driven** - Webhooks are triggered by system events
2. **Reliable Delivery** - Automatic retries with exponential backoff
3. **Signature Verification** - HMAC signatures for webhook authenticity
4. **Idempotent** - Multiple deliveries of same event are safe
5. **Dead Letter Queue** - Failed deliveries are tracked and retryable

### 7.2. Webhook Architecture

```mermaid
sequenceDiagram
    participant System
    participant EventQueue
    participant Dispatcher
    participant Webhook
    participant External
    
    System->>EventQueue: Emit Event
    EventQueue->>Dispatcher: Dequeue Event
    Dispatcher->>Dispatcher: Generate Signature
    Dispatcher->>Webhook: POST with Signature
    alt Delivery Success
        Webhook-->>Dispatcher: 200 OK
        Dispatcher->>EventQueue: Mark Delivered
    else Delivery Failure
        Webhook-->>Dispatcher: Error
        Dispatcher->>Dispatcher: Schedule Retry
        Dispatcher->>EventQueue: Update Retry Count
    end
```

### 7.3. Event Types

| Event Type | Trigger | Description | Payload |
|-----------|---------|-------------|---------|
| **document_created** | Document created | Document metadata |
| **document_updated** | Document modified | Document changes |
| **document_deleted** | Document deleted | Document ID |
| **user_joined** | User registered | User profile |
| **user_left** | User removed | User ID |
| **repository_synced** | Git sync completed | Repository status |
| **error_occurred** | System error | Error details |
| **backup_completed** | Backup finished | Backup metadata |

### 7.4. Webhook Registration

#### 7.4.1. Register Webhook

```http
POST /api/v1/webhooks
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

**Request Body:**

```json
{
  "url": "https://external.example.com/tachyon/webhook",
  "events": ["document_created", "document_updated"],
  "secret": "webhook_secret_abc123",
  "active": true
}
```

**Request Validation:**

| Field | Validation | Error Code |
|-------|-------------|------------|
| **url** | Valid HTTPS URL | `INVALID_URL` |
| **events** | 1-10 valid event types | `INVALID_EVENTS` |
| **secret** | 16-64 characters, alphanumeric | `INVALID_SECRET` |
| **active** | Boolean | `INVALID_ACTIVE` |

**Response (201 Created):**

```json
{
  "data": {
    "id": "whk_xyz789",
    "url": "https://external.example.com/tachyon/webhook",
    "events": ["document_created", "document_updated"],
    "secret": "webhook_secret_abc123",
    "active": true,
    "created_at": "2026-02-07T22:45:00Z",
    "delivery_url": "https://api.tachyon.io/v1/webhooks/whk_xyz789/deliver"
  }
}
```

#### 7.4.2. List Webhooks

```http
GET /api/v1/webhooks
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response (200 OK):**

```json
{
  "data": [
    {
      "id": "whk_xyz789",
      "url": "https://external.example.com/tachyon/webhook",
      "events": ["document_created", "document_updated"],
      "active": true,
      "created_at": "2026-02-07T22:45:00Z",
      "delivery_stats": {
        "total_deliveries": 12345,
        "successful_deliveries": 12300,
        "failed_deliveries": 45,
        "success_rate": 0.9964
      }
    }
  ]
}
```

#### 7.4.3. Delete Webhook

```http
DELETE /api/v1/webhooks/{id}
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response (204 No Content):**

Empty response body with status code 204.

### 7.5. Webhook Delivery

#### 7.5.1. Delivery Endpoint

```http
POST /api/v1/webhooks/{id}/deliver
```

**Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| **X-Tachyon-Signature** | HMAC SHA-256 signature | `sha256=abc123...` |
| **X-Tachyon-Timestamp** | Delivery timestamp | `1641234567` |
| **X-Tachyon-Event-ID** | Unique event ID | `evt_abc123` |
| **Content-Type** | Event payload | `application/json` |

**Request Body:**

```json
{
  "event": "document_created",
  "id": "evt_abc123",
  "timestamp": "2026-02-07T22:45:00.123Z",
  "data": {
    "document_id": "doc_xyz789",
    "title": "Example Document",
    "content": "# Example\n\nContent here.",
    "metadata": {
      "created_at": "2026-02-07T22:45:00Z",
      "author": "user@example.com",
      "tags": ["example", "documentation"]
    }
  }
}
```

#### 7.5.2. Signature Verification

**Signature Generation (Server):**

```rust
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use hex;

pub fn generate_webhook_signature(
    secret: &str,
    payload: &str,
    timestamp: i64,
) -> String {
    let mut mac = Hmac::new(Sha256::new());
    mac.update(secret.as_bytes());
    mac.update(timestamp.to_be_bytes());
    mac.update(payload.as_bytes());
    
    let result = mac.finalize();
    let signature = hex::encode(result);
    
    signature
}
```

**Signature Verification (Webhook Receiver):**

```rust
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use hex;

pub fn verify_webhook_signature(
    secret: &str,
    payload: &str,
    timestamp: i64,
    received_signature: &str,
) -> bool {
    let mut mac = Hmac::new(Sha256::new());
    mac.update(secret.as_bytes());
    mac.update(timestamp.to_be_bytes());
    mac.update(payload.as_bytes());
    
    let result = mac.finalize();
    let expected_signature = hex::encode(result);
    
    // Constant-time comparison to prevent timing attacks
    if received_signature.len() != expected_signature.len() {
        return false;
    }
    
    let mut result = true;
    for (a, b) in received_signature.bytes().zip(expected_signature.bytes()) {
        result &= a == b;
    }
    
    result
}
```

#### 7.5.3. Delivery Response

**Expected Response (200 OK):**

```json
{
  "status": "delivered",
  "event_id": "evt_abc123",
  "delivered_at": "2026-02-07T22:45:00.456Z"
}
```

**Expected Response (Retry Later):**

```json
{
  "status": "retry_later",
  "event_id": "evt_abc123",
  "retry_after": "2026-02-07T22:46:00.000Z",
  "reason": "rate_limit_exceeded"
}
```

### 7.6. Retry Mechanism

#### 7.6.1. Retry Configuration

```rust
pub struct WebhookRetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    
    /// Initial backoff delay in milliseconds
    pub initial_backoff_ms: u64,
    
    /// Maximum backoff delay in milliseconds
    pub max_backoff_ms: u64,
    
    /// Backoff multiplier (exponential)
    pub backoff_multiplier: f64,
    
    /// Jitter factor for randomized backoff
    pub jitter_factor: f64,
}

impl Default for WebhookRetryConfig {
    fn default() -> Self {
        WebhookRetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 1000,  // 1 second
            max_backoff_ms: 300000,  // 5 minutes
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}
```

#### 7.6.2. Retry Logic

```rust
use tokio::time::{sleep, Duration};

pub async fn deliver_with_retry(
    webhook: &Webhook,
    event: &WebhookEvent,
    config: &WebhookRetryConfig,
) -> Result<(), WebhookError> {
    let mut attempt = 0;
    let mut backoff = config.initial_backoff_ms;
    
    loop {
        attempt += 1;
        
        match deliver_webhook(webhook, event).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < config.max_attempts => {
                eprintln!("Webhook delivery attempt {} failed: {}", attempt, e);
                
                // Calculate next backoff with jitter
                let jitter = (backoff as f64 * config.jitter_factor) as u64;
                let delay = backoff + jitter;
                
                sleep(Duration::from_millis(delay)).await;
                backoff = (backoff as f64 * config.backoff_multiplier) as u64;
                backoff = backoff.min(config.max_backoff_ms);
            }
            Err(e) => {
                return Err(WebhookError::MaxRetriesExceeded {
                    webhook_id: webhook.id.clone(),
                    event_id: event.id.clone(),
                    last_error: e.to_string(),
                });
            }
        }
    }
}
```

### 7.7. Dead Letter Queue

#### 7.7.1. Failed Delivery Tracking

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub webhook_id: String,
    pub event_id: String,
    pub payload: serde_json::Value,
    pub attempts: usize,
    pub last_error: String,
    pub failed_at: chrono::DateTime<chrono::Utc>,
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct DeadLetterQueue {
    entries: Vec<DeadLetterEntry>,
}

impl DeadLetterQueue {
    pub fn new() -> Self {
        DeadLetterQueue {
            entries: Vec::new(),
        }
    }
    
    pub fn add(&mut self, entry: DeadLetterEntry) {
        self.entries.push(entry);
    }
    
    pub fn get_retryable(&self) -> Vec<DeadLetterEntry> {
        let now = chrono::Utc::now();
        self.entries
            .iter()
            .filter(|e| {
                if let Some(retry_at) = e.next_retry_at {
                    retry_at <= now
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
}
```

#### 7.7.2. Retry Processing

```rust
pub async fn process_dead_letter_queue(
    queue: &mut DeadLetterQueue,
    webhook_manager: &WebhookManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let retryable = queue.get_retryable();
    
    for entry in retryable {
        eprintln!("Retrying webhook delivery: {}", entry.webhook_id);
        
        match deliver_webhook_by_id(&entry.webhook_id, &entry.event_id).await {
            Ok(_) => {
                // Remove from queue on success
                queue.remove(&entry.webhook_id, &entry.event_id);
            }
            Err(e) => {
                // Update retry time
                let retry_at = chrono::Utc::now() + chrono::Duration::seconds(300);
                queue.update_retry_time(&entry.webhook_id, &entry.event_id, retry_at);
            }
        }
    }
    
    Ok(())
}
```

### 7.8. Webhook Management API

#### 7.8.1. Get Delivery Status

```http
GET /api/v1/webhooks/{id}/deliveries
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|-----------|---------|-------------|
| **status** | string | No | all | Filter by status (delivered, failed, retrying) |
| **since** | ISO 8601 | No | - | Filter deliveries after timestamp |
| **limit** | integer | No | 50 | Maximum results |

**Response (200 OK):**

```json
{
  "data": {
    "webhook_id": "whk_xyz789",
    "deliveries": [
      {
        "event_id": "evt_abc123",
        "status": "delivered",
        "delivered_at": "2026-02-07T22:45:00.456Z",
        "attempts": 1
      },
      {
        "event_id": "evt_def456",
        "status": "failed",
        "attempts": 3,
        "last_error": "connection_timeout",
        "failed_at": "2026-02-07T22:46:30.789Z"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 50,
      "total": 12345,
      "total_pages": 247
    }
  }
}
```

### 7.9. Webhook Receiver Implementation

#### 7.9.1. Rust Receiver

```rust
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    event: String,
    id: String,
    timestamp: String,
    data: serde_json::Value,
}

pub async fn handle_webhook(
    req: HttpRequest,
    webhook_secret: &str,
) -> HttpResponse {
    // Extract headers
    let signature = req.headers().get("X-Tachyon-Signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    let timestamp_header = req.headers().get("X-Tachyon-Timestamp")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.parse::<i64>().ok())
        .unwrap_or(0);
    
    // Read payload
    let payload = match req.body().limit(1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Payload too large"
        })),
    };
    
    // Verify signature
    let payload_str = String::from_utf8_lossy(&payload);
    if !verify_webhook_signature(webhook_secret, &payload_str, timestamp_header, signature) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid signature"
        }));
    }
    
    // Parse payload
    let webhook_payload: WebhookPayload = match serde_json::from_str(&payload_str) {
        Ok(p) => p,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid JSON"
        })),
    };
    
    // Process event
    match process_webhook_event(&webhook_payload).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "delivered",
            "event_id": webhook_payload.id,
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn process_webhook_event(
    payload: &WebhookPayload,
) -> Result<(), Box<dyn std::error::Error>> {
    match payload.event.as_str() {
        "document_created" => {
            // Handle document created event
            handle_document_created(&payload.data).await?;
        }
        "document_updated" => {
            // Handle document updated event
            handle_document_updated(&payload.data).await?;
        }
        _ => {
            // Log unknown event type
            eprintln!("Unknown webhook event type: {}", payload.event);
        }
    }
    
    Ok(())
}
```

#### 7.9.2. TypeScript Receiver

```typescript
import crypto from 'crypto';

interface WebhookPayload {
  event: string;
  id: string;
  timestamp: string;
  data: any;
}

export async function handleWebhook(
  request: Request,
  webhookSecret: string
): Promise<Response> {
  const signature = request.headers.get('X-Tachyon-Signature') || '';
  const timestamp = parseInt(request.headers.get('X-Tachyon-Timestamp') || '0', 10);
  
  // Read payload
  const payload = await request.text();
  
  // Verify signature
  const expectedSignature = generateSignature(webhookSecret, timestamp, payload);
  if (signature !== expectedSignature) {
    return new Response(
      JSON.stringify({ error: 'Invalid signature' }),
      { status: 401 }
    );
  }
  
  // Parse payload
  let webhookPayload: WebhookPayload;
  try {
    webhookPayload = JSON.parse(payload);
  } catch (error) {
    return new Response(
      JSON.stringify({ error: 'Invalid JSON' }),
      { status: 400 }
    );
  }
  
  // Process event
  try {
    await processWebhookEvent(webhookPayload);
    return new Response(
      JSON.stringify({ 
        status: 'delivered',
        eventId: webhookPayload.id 
      }),
      { status: 200 }
    );
  } catch (error) {
    console.error('Failed to process webhook:', error);
    return new Response(
      JSON.stringify({ error: error.message }),
      { status: 500 }
    );
  }
}

function generateSignature(
  secret: string,
  timestamp: number,
  payload: string
): string {
  const hmac = crypto.createHmac('sha256', secret);
  hmac.update(timestamp.toString());
  hmac.update(payload);
  return hmac.digest('hex');
}

async function processWebhookEvent(payload: WebhookPayload): Promise<void> {
  switch (payload.event) {
    case 'document_created':
      await handleDocumentCreated(payload.data);
      break;
    case 'document_updated':
      await handleDocumentUpdated(payload.data);
      break;
    default:
      console.log(`Unknown webhook event type: ${payload.event}`);
  }
}
```

---

---

## 8. OAUTH INTEGRATION

### 8.1. OAuth Overview

The Tachyon OAuth integration enables third-party authentication and authorization using OAuth 2.0 (RFC 6749). This allows users to authenticate using their existing accounts from providers like Google, GitHub, and other OAuth-compliant services.

**Design Principles:**

1. **OAuth 2.0 Compliance** - Full implementation of RFC 6749 specification
2. **PKCE Support** - Proof Key for Code Exchange for public clients
3. **Multiple Providers** - Extensible provider architecture
4. **Token Security** - Secure token storage and refresh mechanism
5. **State Management** - Secure state parameter handling

### 8.2. Supported Providers

| Provider | OAuth Version | Scopes | PKCE Support |
|----------|--------------|--------|------------|
| **Google** | 2.0 | openid, email, profile | Yes |
| **GitHub** | 2.0 | user:email, repo, read:user | Yes |
| **Microsoft** | 2.0 | openid, email, profile | Yes |
| **GitLab** | 2.0 | read_user, read_repository | Yes |
| **Bitbucket** | 2.0 | account, repository:write, repository:read | Yes |

### 8.3. OAuth Flow

#### 8.3.1. Authorization Code Flow

```mermaid
sequenceDiagram
    participant Client
    participant Browser
    participant Tachyon
    participant Provider
    
    Client->>Browser: Initiate OAuth
    Browser->>Tachyon: GET /api/v1/auth/oauth/{provider}
    Tachyon->>Tachyon: Generate state and code verifier
    Tachyon-->>Browser: Redirect to Provider
    Browser->>Provider: GET /authorize?response_type=code...
    Provider->>Browser: User authenticates
    Provider-->>Browser: Redirect to Tachyon with code
    Browser->>Tachyon: GET /api/v1/auth/oauth/{provider}/callback?code=...
    Tachyon->>Provider: Exchange code for tokens
    Provider-->>Tachyon: Access token + refresh token
    Tachyon-->>Browser: Authentication complete
```

#### 8.3.2. Authorization Endpoint

```http
GET /api/v1/auth/oauth/{provider}
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| **provider** | string | Yes | Provider identifier (google, github, etc.) |
| **redirect_uri** | string | Yes | OAuth redirect URI |
| **state** | string | No | Anti-CSRF state parameter |
| **response_type** | string | Yes | Response type (code, token) |

**Response (302 Found):**

```http
HTTP/1.1 302 Found
Location: https://accounts.google.com/o/oauth2/v2/auth?client_id=...&redirect_uri=...&response_type=code&state=...
```

#### 8.3.3. Callback Endpoint

```http
GET /api/v1/auth/oauth/{provider}/callback
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| **code** | string | Yes | Authorization code from provider |
| **state** | string | Yes | State parameter for CSRF protection |
| **error** | string | No | Error code if authorization failed |

**Response (200 OK):**

```json
{
  "data": {
    "access_token": "ya29.a0AfH6SM...",
    "refresh_token": "1//0gX...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
      "id": "usr_123",
      "email": "user@example.com",
      "name": "User Name",
      "avatar": "https://example.com/avatar.jpg"
    },
    "provider": "google"
  }
}
```

**Response (Error):**

```json
{
  "error": {
    "code": "OAUTH_ERROR",
    "message": "Authorization failed",
    "details": {
      "provider": "google",
      "error_code": "access_denied",
      "error_description": "User denied authorization"
    }
  }
}
```

### 8.4. PKCE Implementation

#### 8.4.1. Code Verifier Generation

```rust
use base64::{Engine as Base64Engine, engine::general_purpose::STANDARD};
use sha2::{Sha256, Digest};
use rand::Rng;

/// Generate PKCE code verifier
pub fn generate_pkce_verifier() -> (String, String) {
    // Generate random code verifier
    let mut rng = rand::thread_rng();
    let code_verifier: [u8; 128] = rng.gen();
    
    // Encode to base64 URL-safe
    let code_verifier_b64 = Base64Engine::encode(&code_verifier);
    let code_verifier_url = code_verifier_b64
        .replace("+", "-")
        .replace("/", "_")
        .replace("=", "");
    
    // Generate code challenge
    let code_challenge = generate_code_challenge();
    
    (code_verifier_url, code_challenge)
}

/// Generate code challenge for PKCE
fn generate_code_challenge() -> String {
    // Generate random code challenge
    let mut rng = rand::thread_rng();
    let code_challenge: [u8; 128] = rng.gen();
    
    // Hash with SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&code_challenge);
    let hash = hasher.finalize();
    
    // Encode to base64 URL-safe
    let hash_b64 = Base64Engine::encode(&hash);
    hash_b64
        .replace("+", "-")
        .replace("/", "_")
        .replace("=", "");
    
    hash_b64
}
```

#### 8.4.2. PKCE Authorization URL

```http
GET /api/v1/auth/oauth/google
?client_id=...
&redirect_uri=...
&response_type=code
&state=...
&code_challenge=...
&code_verifier=...
```

### 8.5. Token Management

#### 8.5.1. Token Storage

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}
```

#### 8.5.2. Token Refresh

```rust
use tokio::time::{sleep, Duration};

/// Refresh OAuth access token
pub async fn refresh_oauth_token(
    client: &reqwest::Client,
    provider: &str,
    refresh_token: &str,
) -> Result<OAuthTokens, Box<dyn std::error::Error>> {
    let refresh_url = format!("https://oauth.tachyon.io/v1/{}/refresh", provider);
    
    let request = RefreshTokenRequest {
        refresh_token: refresh_token.to_string(),
    };
    
    let response = client
        .post(&refresh_url)
        .json(&request)
        .send()
        .await?;
    
    if response.status().is_success() {
        let tokens: OAuthTokens = response.json().await?;
        Ok(tokens)
    } else {
        Err(format!("Token refresh failed: {}", response.status()).into())
    }
}

/// Auto-refresh token before expiration
pub async fn ensure_valid_token(
    client: &reqwest::Client,
    tokens: &mut OAuthTokens,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();
    let expires_soon = tokens.expires_at - chrono::Duration::minutes(5);
    
    if now >= expires_soon {
        eprintln!("Token expiring soon, refreshing...");
        
        match refresh_oauth_token(client, &tokens.provider, &tokens.refresh_token).await {
            Ok(new_tokens) => {
                *tokens = new_tokens;
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to refresh token: {}", e);
                Err(e)
            }
        }
    } else {
        Ok(())
    }
}
```

### 8.6. Provider Implementations

#### 8.6.1. Google OAuth

```rust
use reqwest::Client;
use serde_json::json;

pub struct GoogleOAuthProvider;

impl GoogleOAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        GoogleOAuthProvider {
            client_id,
            client_secret,
        }
    }
    
    pub async fn get_authorization_url(
        &self,
        redirect_uri: &str,
        code_challenge: &str,
        code_verifier: &str,
    ) -> String {
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&code_challenge={}&code_verifier={}",
            self.client_id,
            urlencoding::encode(redirect_uri),
            code_challenge,
            code_verifier
        )
    }
    
    pub async fn exchange_code_for_tokens(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<OAuthTokens, Box<dyn std::error::Error>> {
        let client = Client::new();
        
        let mut params = vec![
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("code_verifier", code_verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "http://localhost:8080/auth/callback"),
        ];
        
        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let tokens: serde_json::from_str::<serde_json::Value>(response.text().await?)?;
            
            Ok(OAuthTokens {
                access_token: tokens["access_token"].as_str().unwrap().to_string(),
                refresh_token: tokens["refresh_token"].as_str().unwrap().to_string(),
                token_type: "Bearer".to_string(),
                expires_at: chrono::Utc::now() + chrono::Duration::seconds(3600),
                provider: "google".to_string(),
                user_id: String::new(), // Will be populated from token response
            })
        } else {
            Err(format!("Failed to exchange code: {}", response.status()).into())
        }
    }
}
```

#### 8.6.2. GitHub OAuth

```rust
pub struct GitHubOAuthProvider;

impl GitHubOAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        GitHubOAuthProvider {
            client_id,
            client_secret,
        }
    }
    
    pub async fn get_authorization_url(
        &self,
        redirect_uri: &str,
        state: &str,
    ) -> String {
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email%20read:user&state={}",
            self.client_id,
            urlencoding::encode(redirect_uri),
            state
        )
    }
    
    pub async fn exchange_code_for_tokens(
        &self,
        code: &str,
    ) -> Result<OAuthTokens, Box<dyn std::error::Error>> {
        let client = Client::new();
        
        let mut params = vec![
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "http://localhost:8080/auth/callback"),
        ];
        
        let response = client
            .post("https://github.com/login/oauth/access_token")
            .form(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let tokens: serde_json::from_str::<serde_json::Value>(response.text().await?)?;
            
            Ok(OAuthTokens {
                access_token: tokens["access_token"].as_str().unwrap().to_string(),
                refresh_token: tokens.get("refresh_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                token_type: "Bearer".to_string(),
                expires_at: chrono::Utc::now() + chrono::Duration::seconds(3600),
                provider: "github".to_string(),
                user_id: String::new(),
            })
        } else {
            Err(format!("Failed to exchange code: {}", response.status()).into())
        }
    }
}
```

### 8.7. Client Integration

#### 8.7.1. JavaScript Client

```typescript
interface OAuthConfig {
  provider: 'google' | 'github' | 'microsoft';
  clientId: string;
  redirectUri: string;
}

interface OAuthTokens {
  accessToken: string;
  refreshToken: string;
  tokenType: string;
  expiresAt: string;
  provider: string;
  user: {
    id: string;
    email: string;
    name: string;
    avatar?: string;
  };
}

class OAuthClient {
  private config: OAuthConfig;
  private tokens: OAuthTokens | null = null;

  constructor(config: OAuthConfig) {
    this.config = config;
    this.loadTokens();
  }

  async initiateOAuth(): Promise<void> {
    const state = this.generateState();
    const { codeChallenge, codeVerifier } = this.generatePKCE();
    
    const authUrl = this.buildAuthorizationUrl(state, codeChallenge, codeVerifier);
    
    // Store state for callback verification
    sessionStorage.setItem('oauth_state', state);
    sessionStorage.setItem('oauth_code_verifier', codeVerifier);
    
    // Redirect to provider
    window.location.href = authUrl;
  }

  async handleCallback(code: string, state: string): Promise<OAuthTokens> {
    // Verify state
    const storedState = sessionStorage.getItem('oauth_state');
    if (state !== storedState) {
      throw new Error('Invalid state parameter');
    }
    
    // Exchange code for tokens
    const tokens = await this.exchangeCodeForTokens(code);
    
    // Store tokens
    this.saveTokens(tokens);
    
    // Clean up
    sessionStorage.removeItem('oauth_state');
    sessionStorage.removeItem('oauth_code_verifier');
    
    return tokens;
  }

  async exchangeCodeForTokens(code: string): Promise<OAuthTokens> {
    const codeVerifier = sessionStorage.getItem('oauth_code_verifier') || '';
    
    const response = await fetch('/api/v1/auth/oauth/' + this.config.provider + '/callback?code=' + code + '&code_verifier=' + codeVerifier);
    
    if (!response.ok) {
      throw new Error('Failed to exchange code for tokens');
    }
    
    const data = await response.json();
    return data.data;
  }

  private saveTokens(tokens: OAuthTokens): void {
    this.tokens = tokens;
    localStorage.setItem('oauth_tokens', JSON.stringify(tokens));
  }

  private loadTokens(): void {
    const stored = localStorage.getItem('oauth_tokens');
    if (stored) {
      this.tokens = JSON.parse(stored);
    }
  }

  private generateState(): string {
    return 'state_' + Math.random().toString(36).substr(2, 9);
  }

  private generatePKCE(): { codeChallenge: string; codeVerifier: string } {
    const codeChallenge = this.base64UrlEncode(this.generateRandomString(128));
    const codeVerifier = this.base64UrlEncode(this.generateRandomString(128));
    
    return { codeChallenge, codeVerifier };
  }

  private generateRandomString(length: number): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }

  private base64UrlEncode(str: string): string {
    return btoa(str)
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }

  async refreshToken(): Promise<OAuthTokens> {
    if (!this.tokens) {
      throw new Error('No tokens available');
    }

    const response = await fetch('/api/v1/auth/oauth/' + this.tokens.provider + '/refresh', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        refresh_token: this.tokens.refreshToken,
      }),
    });

    if (!response.ok) {
      throw new Error('Failed to refresh token');
    }

    const data = await response.json();
    this.saveTokens(data.data);
    return data.data;
  }

  async logout(): Promise<void> {
    this.tokens = null;
    localStorage.removeItem('oauth_tokens');
    sessionStorage.clear();
  }
}
```

### 8.8. Security Considerations

#### 8.8.1. State Parameter Security

The state parameter is used to prevent CSRF attacks:

```rust
use rand::Rng;

/// Generate secure state parameter
pub fn generate_oauth_state() -> String {
    let mut rng = rand::thread_rng();
    let state: String = (0..32)
        .map(|_| rng.gen::<char>())
        .collect();
    
    // Add timestamp to prevent replay attacks
    let timestamp = chrono::Utc::now().timestamp();
    
    format!("{}|{}", state, timestamp)
}
```

#### 8.8.2. Token Storage Security

**Security Requirements:**

1. **HttpOnly Cookies** - Tokens stored in HttpOnly cookies
2. **Secure Flag** - Secure flag set for HTTPS
3. **SameSite** - SameSite=Strict or Lax
4. **Encrypted Storage** - Tokens encrypted at rest
5. **Short Lifetime** - Access tokens expire in 1 hour

#### 8.8.3. Token Refresh Security

**Refresh Flow:**

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant Provider
    
    Client->>Server: POST /refresh with refresh_token
    Server->>Provider: Exchange refresh_token for new access_token
    Provider-->>Server: New access_token + refresh_token
    Server-->>Client: Updated tokens
```

**Security Measures:**

1. **Refresh Token Rotation** - New refresh tokens on each refresh
2. **Token Revocation** - Revoke old tokens on logout
3. **Rate Limiting** - Limit refresh attempts
4. **Token Binding** - Bind tokens to device/session

---

---

## 9. PLUGIN INTEGRATION

### 9.1. Plugin Overview

The Tachyon plugin integration provides an extensible architecture for adding custom functionality without modifying core code. Plugins are executed in a sandboxed WebAssembly (WASM) environment with capability-based permissions.

**Design Principles:**

1. **Sandboxed Execution** - Plugins run in isolated WASM sandbox
2. **Capability-Based Security** - Fine-grained permissions for each operation
3. **Hot-Reloading** - Plugins can be reloaded without restart
4. **Version Compatibility** - Plugin API versioning for compatibility
5. **Resource Monitoring** - CPU, memory, and network usage limits
6. **Error Isolation** - Plugin errors don't crash the application

### 9.2. Plugin Architecture

```mermaid
graph TB
    subgraph "Tachyon Core"
        PluginManager[Plugin Manager]
        PluginAPI[Plugin API]
        CapabilitySystem[Capability System]
        ResourceMonitor[Resource Monitor]
    end
    
    subgraph "Plugin Sandbox"
        WASMRuntime[WASM Runtime]
        Plugin[Loaded Plugins]
    end
    
    subgraph "External Plugins"
        PluginManifest[Plugin Manifests]
        PluginCode[Plugin WASM Code]
        PluginAssets[Plugin Assets]
    end
    
    PluginManager --> PluginAPI
    PluginAPI --> PluginManager
    PluginManager --> CapabilitySystem
    PluginManager --> ResourceMonitor
    PluginManager --> WASMRuntime
    CapabilitySystem --> PluginManager
    ResourceMonitor --> PluginManager
    WASMRuntime --> Plugin
    
    PluginManifest --> PluginCode
    PluginCode --> PluginAssets
```

### 9.3. Plugin Manifest

#### 9.3.1. Manifest Format

```json
{
  "name": "example-plugin",
  "version": "1.0.0",
  "tachyon_version": "1.0.0",
  "description": "An example plugin for Tachyon",
  "author": "Plugin Author",
  "license": "MIT",
  "homepage": "https://github.com/example/tachyon-plugin",
  "repository": "https://github.com/example/tachyon-plugin.git",
  
  "capabilities": [
    {
      "id": "read_documents",
      "description": "Read document metadata",
      "permissions": ["documents:read"]
    },
    {
      "id": "write_documents",
      "description": "Write document content",
      "permissions": ["documents:write"]
    }
  ],
  
  "exports": {
    "init": "init_plugin",
    "destroy": "destroy_plugin",
    "on_document_open": "on_document_open",
    "on_document_save": "on_document_save"
  },
  
  "resources": {
    "max_memory_mb": 128,
    "max_cpu_percent": 50,
    "max_network_bandwidth_kbps": 1000
  }
}
```

#### 9.3.2. Manifest Validation

```rust
use serde::{Deserialize, Serialize};
use validator::ValidateLength;

#[derive(Debug, Deserialize, Serialize, ValidateLength)]
pub struct PluginManifest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(length(min = 1, max = 100))]
    pub version: String,
    
    #[validate(length(min = 1, max = 100))]
    pub tachyon_version: String,
    
    #[validate(length(min = 10, max = 1000))]
    pub description: String,
    
    #[validate(length(min = 1, max = 100))]
    pub author: String,
    
    #[validate(length(min = 1, max = 100))]
    pub license: String,
    
    #[validate(url)]
    pub homepage: String,
    
    #[validate(url)]
    pub repository: String,
    
    #[validate(length(min = 1))]
    pub capabilities: Vec<PluginCapability>,
    
    #[validate(length(min = 1))]
    pub exports: PluginExports,
    
    pub resources: PluginResources,
}

#[derive(Debug, Deserialize, Serialize, ValidateLength)]
pub struct PluginCapability {
    #[validate(length(min = 1, max = 100))]
    pub id: String,
    
    #[validate(length(min = 10, max = 500))]
    pub description: String,
    
    #[validate(length(min = 1))]
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginExports {
    pub init: String,
    pub destroy: String,
    pub on_document_open: Option<String>,
    pub on_document_save: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginResources {
    pub max_memory_mb: usize,
    pub max_cpu_percent: usize,
    pub max_network_bandwidth_kbps: usize,
}

pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    
    // Check Tachyon version compatibility
    let required_version = "1.0.0";
    if !is_version_compatible(&manifest.tachyon_version, required_version) {
        errors.push(format!(
            "Incompatible Tachyon version: {} (requires {})",
            manifest.tachyon_version, required_version
        ));
    }
    
    // Validate capabilities
    for capability in &manifest.capabilities {
        if capability.permissions.is_empty() {
            errors.push(format!(
                "Capability {} has no permissions",
                capability.id
            ));
        }
    }
    
    // Validate resources
    if manifest.resources.max_memory_mb > 256 {
        errors.push(format!(
            "Max memory {} exceeds limit of 256 MB",
            manifest.resources.max_memory_mb
        ));
    }
    
    if manifest.resources.max_cpu_percent > 100 {
        errors.push("Max CPU usage exceeds 100%");
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn is_version_compatible(version: &str, required: &str) -> bool {
    let v_parts: Vec<&str> = version.split('.').collect();
    let r_parts: Vec<&str> = required.split('.').collect();
    
    for (i, (v, r)) in v_parts.iter().zip(r_parts.iter()) {
        match v.cmp(r) {
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Greater => return false,
            _ => continue,
        }
    }
    
    true
}
```

### 9.4. Plugin API

#### 9.4.1. Plugin Lifecycle

```rust
use wasm_bindgen::prelude::*;

/// Plugin trait defining lifecycle methods
#[wasm_bindgen]
pub trait TachyonPlugin {
    /// Initialize the plugin
    #[wasm_bindgen(method, js_name = "init")]
    fn init(&self) -> Result<(), JsValue>;
    
    /// Destroy the plugin and cleanup resources
    #[wasm_bindgen(method, js_name = "destroy")]
    fn destroy(&self) -> Result<(), JsValue>;
    
    /// Get plugin metadata
    #[wasm_bindgen(method, getter)]
    fn metadata(&self) -> PluginMetadata;
}

/// Plugin metadata structure
#[wasm_bindgen]
pub struct PluginMetadata {
    #[wasm_bindgen(readonly)]
    pub name: String,
    
    #[wasm_bindgen(readonly)]
    pub version: String,
    
    #[wasm_bindgen(readonly)]
    pub description: String,
}

/// Example plugin implementation
#[wasm_bindgen]
pub struct ExamplePlugin;

impl TachyonPlugin for ExamplePlugin {
    fn init(&self) -> Result<(), JsValue> {
        // Initialize plugin state
        Ok(JsValue::UNDEFINED)
    }
    
    fn destroy(&self) -> Result<(), JsValue> {
        // Cleanup resources
        Ok(JsValue::UNDEFINED)
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "example-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "An example plugin for Tachyon".to_string(),
        }
    }
}
```

#### 9.4.2. Document Events

```rust
use wasm_bindgen::prelude::*;

/// Document event data structure
#[wasm_bindgen]
pub struct DocumentEvent {
    #[wasm_bindgen(readonly)]
    pub document_id: String,
    
    #[wasm_bindgen(readonly)]
    pub title: String,
    
    #[wasm_bindgen(readonly)]
    pub content: String,
}

/// Document event handler trait
#[wasm_bindgen]
pub trait DocumentEventHandler {
    #[wasm_bindgen(method, js_name = "onDocumentOpen")]
    fn on_document_open(&self, event: DocumentEvent) -> Result<(), JsValue>;
    
    #[wasm_bindgen(method, js_name = "onDocumentSave")]
    fn on_document_save(&self, event: DocumentEvent) -> Result<(), JsValue>;
}

/// Event emitter for document events
#[wasm_bindgen]
pub struct DocumentEventEmitter {
    listeners: Vec<Box<dyn DocumentEventHandler>>,
}

impl DocumentEventEmitter {
    pub fn new() -> Self {
        DocumentEventEmitter {
            listeners: Vec::new(),
        }
    }
    
    pub fn register(&mut self, handler: Box<dyn DocumentEventHandler>) {
        self.listeners.push(handler);
    }
    
    pub fn emit(&self, event: DocumentEvent) {
        for listener in &self.listeners {
            let _ = listener.handle_event(event.clone());
        }
    }
}
```

### 9.5. Capability System

#### 9.5.1. Capability Definition

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CapabilityRequest {
    pub capability_id: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct CapabilityResponse {
    pub granted: bool,
    pub reason: Option<String>,
}
```

#### 9.5.2. Capability Check

```rust
pub async fn check_capability(
    plugin_id: &str,
    request: CapabilityRequest,
) -> Result<CapabilityResponse, String> {
    // Load plugin manifest
    let manifest = load_plugin_manifest(plugin_id).await?;
    
    // Find capability in manifest
    let capability = manifest.capabilities
        .iter()
        .find(|c| c.id == request.capability_id)
        .ok_or_else(|| "Capability not found".to_string())?;
    
    // Check if plugin has required permissions
    let has_permissions = request.context
        .as_object()
        .and_then(|obj| obj.get("permissions"))
        .and_then(|perms| perms.as_array())
        .map(|perms| {
            perms.iter().all(|p| capability.permissions.contains(p))
        })
        .unwrap_or(false);
    
    Ok(CapabilityResponse {
        granted: has_permissions,
        reason: if !has_permissions {
            Some(format!("Missing required permissions"))
        } else {
            None
        },
    })
}
```

### 9.6. Resource Monitoring

#### 9.6.1. Resource Limits

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub network_kbps: f64,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct ResourceMonitor {
    max_memory_mb: f64,
    max_cpu_percent: f64,
    max_network_kbps: f64,
    current_usage: ResourceUsage,
}

impl ResourceMonitor {
    pub fn new(max_memory_mb: f64, max_cpu_percent: f64, max_network_kbps: f64) -> Self {
        ResourceMonitor {
            max_memory_mb,
            max_cpu_percent,
            max_network_kbps,
            current_usage: ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                network_kbps: 0.0,
                timestamp: Instant::now(),
            },
        }
    }
    
    pub fn check_limits(&self) -> bool {
        let memory_exceeded = self.current_usage.memory_mb > self.max_memory_mb;
        let cpu_exceeded = self.current_usage.cpu_percent > self.max_cpu_percent;
        let network_exceeded = self.current_usage.network_kbps > self.max_network_kbps;
        
        !memory_exceeded && !cpu_exceeded && !network_exceeded
    }
    
    pub fn update_usage(&mut self, memory_mb: f64, cpu_percent: f64, network_kbps: f64) {
        self.current_usage.memory_mb = memory_mb;
        self.current_usage.cpu_percent = cpu_percent;
        self.current_usage.network_kbps = network_kbps;
        self.current_usage.timestamp = Instant::now();
    }
}
```

#### 9.6.2. Resource Enforcement

```rust
pub async fn enforce_resource_limits(
    monitor: &mut ResourceMonitor,
    plugin_id: &str,
) -> Result<(), String> {
    if !monitor.check_limits() {
        // Suspend plugin
        suspend_plugin(plugin_id).await?;
        
        return Err(format!(
            "Plugin {} suspended due to resource limit violation",
            plugin_id
        ));
    }
    
    Ok(())
}

async fn suspend_plugin(plugin_id: &str) -> Result<(), String> {
    // Notify plugin to cleanup
    // Terminate plugin WASM instance
    // Release allocated resources
    
    Ok(())
}
```

### 9.7. Plugin Manager

#### 9.7.1. Plugin Loading

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    runtime: Option<wasmer::Runtime>,
    lock: RwLock<()>,
}

#[derive(Debug)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub instance: Option<wasmer::Instance>,
    pub resource_usage: ResourceUsage,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
            runtime: None,
            lock: RwLock::new(),
        }
    }
    
    pub async fn load_plugin(
        &mut self,
        plugin_path: &Path,
    ) -> Result<(), String> {
        let _guard = self.lock.write().await;
        
        // Read plugin manifest
        let manifest_content = tokio::fs::read_to_string(plugin_path).await
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Invalid manifest: {}", e))?;
        
        // Validate manifest
        validate_manifest(&manifest)?;
        
        // Load WASM module
        let wasm_bytes = tokio::fs::read(&plugin_path.with_extension("wasm")).await
            .map_err(|e| format!("Failed to read WASM: {}", e))?;
        
        // Initialize WASM runtime
        let mut config = wasmer::Config::default();
        config.max_memory_pages = Some(manifest.resources.max_memory_mb * 1024 / 65536);
        
        let runtime = wasmer::Runtime::new(&config)
            .map_err(|e| format!("Failed to create WASM runtime: {}", e))?;
        
        // Instantiate plugin
        let instance = runtime.instantiate(&wasm_bytes)
            .map_err(|e| format!("Failed to instantiate plugin: {}", e))?;
        
        // Store plugin
        let loaded_plugin = LoadedPlugin {
            manifest: manifest.clone(),
            instance: Some(instance),
            resource_usage: ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                network_kbps: 0.0,
                timestamp: chrono::Utc::now(),
            },
        };
        
        self.plugins.insert(manifest.name.clone(), loaded_plugin);
        
        Ok(())
    }
}
```

#### 9.7.2. Plugin Invocation

```rust
pub async fn invoke_plugin_method(
    manager: &PluginManager,
    plugin_name: &str,
    method_name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let _guard = manager.lock.read().await;
    
    let plugin = manager.plugins
        .get(plugin_name)
        .ok_or_else(|| format!("Plugin {} not loaded", plugin_name))?;
    
    let instance = plugin.instance
        .as_ref()
        .ok_or_else(|| format!("Plugin {} not instantiated", plugin_name))?;
    
    // Invoke method
    let result = instance
        .call(method_name, args)
        .map_err(|e| format!("Plugin method failed: {}", e))?;
    
    Ok(result)
}
```

### 9.8. Plugin Development

#### 9.8.1. Plugin Template

```rust
use wasm_bindgen::prelude::*;

/// Example plugin template
#[wasm_bindgen]
pub struct TemplatePlugin;

impl TemplatePlugin {
    // Plugin state
    state: String,
}

#[wasm_bindgen]
impl TemplatePlugin {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        TemplatePlugin {
            state: String::new(),
        }
    }
    
    #[wasm_bindgen(method, js_name = "init")]
    pub fn init(&mut self) -> Result<(), JsValue> {
        self.state = "initialized".to_string();
        Ok(JsValue::UNDEFINED)
    }
    
    #[wasm_bindgen(method, js_name = "process")]
    pub fn process(&mut self, input: String) -> Result<String, JsValue> {
        // Process input
        let result = format!("Processed: {}", input);
        self.state = result.clone();
        Ok(result)
    }
    
    #[wasm_bindgen(method, js_name = "getState")]
    pub fn get_state(&self) -> String {
        self.state.clone()
    }
    
    #[wasm_bindgen(method, js_name = "destroy")]
    pub fn destroy(&mut self) -> Result<(), JsValue> {
        self.state = String::new();
        Ok(JsValue::UNDEFINED)
    }
}
```

#### 9.8.2. Build Configuration

```toml
[package]
name = "tachyon-plugin-template"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
tachyon-plugin-api = { path = "../tachyon-plugin-api" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "z"
lto = true

[package.metadata.tachyon]
tachyon-version = "1.0.0"
plugin-type = "document-processor"
```

---

---

## 10. TESTING

### 10.1. Testing Overview

The Tachyon integration testing strategy follows the test plan defined in [`.specs/04_future_state/test_plan.md`](.specs/04_future_state/test_plan.md) with specific focus on integration points. Testing ensures reliability, security, and performance of all integrations.

**Testing Principles:**

1. **Test-Driven Development** - Tests written before implementation
2. **Pyramid Structure** - Unit, Integration, E2E test distribution
3. **Automated Execution** - CI/CD integration for all tests
4. **Continuous Monitoring** - Real-time test execution and reporting
5. **Coverage Requirements** - Minimum coverage thresholds enforced

### 10.2. Unit Testing

#### 10.2.1. REST API Unit Tests

```rust
use tokio::test;
use reqwest::Client;
use serde_json::json;

#[cfg(test)]
mod rest_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_document_success() {
        let client = Client::new();
        let response = client
            .get("https://api.tachyon.io/v1/documents/doc_123")
            .header("Authorization", "Bearer test_token")
            .send()
            .await;
        
        assert!(response.status().is_success());
        let doc: Document = response.json().await.unwrap();
        assert_eq!(doc.id, "doc_123");
    }
    
    #[tokio::test]
    async fn test_get_document_not_found() {
        let client = Client::new();
        let response = client
            .get("https://api.tachyon.io/v1/documents/nonexistent")
            .header("Authorization", "Bearer test_token")
            .send()
            .await;
        
        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
        let error: ApiError = response.json().await.unwrap();
        assert_eq!(error.code, "DOCUMENT_NOT_FOUND");
    }
    
    #[tokio::test]
    async fn test_create_document_validation() {
        let client = Client::new();
        let invalid_doc = serde_json::json!({
            "title": "", // Empty title
        });
        
        let response = client
            .post("https://api.tachyon.io/v1/documents")
            .header("Authorization", "Bearer test_token")
            .json(&invalid_doc)
            .send()
            .await;
        
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        let error: ApiError = response.json().await.unwrap();
        assert_eq!(error.code, "VALIDATION_ERROR");
    }
}
```

#### 10.2.2. WebSocket Unit Tests

```rust
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use futures_util::stream::StreamExt;

#[cfg(test)]
mod websocket_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_websocket_connection() {
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(
            tungstenite::handshake::client::Request::from_url(
                url::Url::parse("ws://localhost:8080/v1/ws").unwrap()
            )
        ).await.unwrap();
        
        // Send auth message
        let auth_msg = serde_json::json!({
            "type": "auth",
            "id": "test_auth",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": { "token": "test_token" }
        });
        ws_stream.send(Message::Text(auth_msg)).await.unwrap();
        
        // Wait for auth success
        let auth_success = tokio::time::timeout(Duration::from_secs(5), ws_stream.next()).await;
        match auth_success {
            Ok(Message::Text(text)) => {
                let response: serde_json::from_str::<serde_json::Value>(&text).unwrap();
                assert_eq!(response["type"], "auth_success");
            }
            _ => panic!("Expected auth_success message"),
        }
    }
    
    #[tokio::test]
    async fn test_document_update_broadcast() {
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(
            tungstenite::handshake::client::Request::from_url(
                url::Url::parse("ws://localhost:8080/v1/ws").unwrap()
            )
        ).await.unwrap();
        
        // Send auth
        let auth_msg = serde_json::json!({
            "type": "auth",
            "id": "test_auth",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": { "token": "test_token" }
        });
        ws_stream.send(Message::Text(auth_msg)).await.unwrap();
        
        // Wait for auth success
        let _ = tokio::time::timeout(Duration::from_secs(5), ws_stream.next()).await;
        
        // Send document update
        let update_msg = serde_json::json!({
            "type": "document_update",
            "id": "test_update",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": {
                "document_id": "doc_123",
                "version": 1,
                "operation": "insert",
                "content": "Test content"
            }
        });
        ws_stream.send(Message::Text(update_msg)).await.unwrap();
        
        // Receive broadcast
        let broadcast = tokio::time::timeout(Duration::from_millis(100), ws_stream.next()).await;
        match broadcast {
            Ok(Message::Text(text)) => {
                let response: serde_json::from_str::<serde_json::Value>(&text).unwrap();
                assert_eq!(response["type"], "document_update");
                assert_eq!(response["data"]["document_id"], "doc_123");
            }
            _ => panic!("Expected broadcast message"),
        }
    }
}
```

#### 10.2.3. IPC Unit Tests

```rust
use tauri::State;
use serde_json::json;

#[cfg(test)]
mod ipc_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ipc_command_success() {
        let state = State::mock();
        
        let result = get_document(&state, "doc_123".to_string()).await;
        assert!(result.is_ok());
        
        let doc = result.unwrap();
        assert_eq!(doc.id, "doc_123");
    }
    
    #[tokio::test]
    async fn test_ipc_command_error() {
        let state = State::mock();
        
        let result = get_document(&state, "".to_string()).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert_eq!(error, "Document ID is required");
    }
    
    #[tokio::test]
    async fn test_ipc_event_emission() {
        // This test verifies events are emitted correctly
        // Implementation depends on Tauri's event system
    }
}
```

### 10.3. Integration Testing

#### 10.3.1. REST Integration Tests

```rust
use reqwest::Client;
use serde_json::json;

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rest_integration_e2e() {
        // Test end-to-end REST API flow
        let client = Client::new();
        
        // Create document
        let create_doc = serde_json::json!({
            "title": "Integration Test",
            "content": "# Test"
        });
        let create_response = client
            .post("https://api.tachyon.io/v1/documents")
            .header("Authorization", "Bearer test_token")
            .json(&create_doc)
            .send()
            .await;
        
        assert!(create_response.status().is_success());
        let created: Document = create_response.json().await.unwrap();
        let doc_id = created.id.clone();
        
        // Get document
        let get_response = client
            .get(&format!("https://api.tachyon.io/v1/documents/{}", doc_id))
            .header("Authorization", "Bearer test_token")
            .send()
            .await;
        
        assert!(get_response.status().is_success());
        let retrieved: Document = get_response.json().await.unwrap();
        assert_eq!(retrieved.id, doc_id);
        assert_eq!(retrieved.title, "Integration Test");
        
        // Update document
        let update_doc = serde_json::json!({
            "title": "Updated Integration Test"
        });
        let update_response = client
            .patch(&format!("https://api.tachyon.io/v1/documents/{}", doc_id))
            .header("Authorization", "Bearer test_token")
            .json(&update_doc)
            .send()
            .await;
        
        assert!(update_response.status().is_success());
        let updated: Document = update_response.json().await.unwrap();
        assert_eq!(updated.title, "Updated Integration Test");
        
        // Delete document
        let delete_response = client
            .delete(&format!("https://api.tachyon.io/v1/documents/{}", doc_id))
            .header("Authorization", "Bearer test_token")
            .send()
            .await;
        
        assert!(delete_response.status().is_success());
        
        // Verify deletion
        let get_after_response = client
            .get(&format!("https://api.tachyon.io/v1/documents/{}", doc_id))
            .header("Authorization", "Bearer test_token")
            .send()
            .await;
        
        assert_eq!(get_after_response.status(), reqwest::StatusCode::NOT_FOUND);
    }
}
```

#### 10.3.2. WebSocket Integration Tests

```rust
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[cfg(test)]
mod websocket_integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_websocket_e2e_flow() {
        // Test end-to-end WebSocket flow
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(
            tungstenite::handshake::client::Request::from_url(
                url::Url::parse("ws://localhost:8080/v1/ws").unwrap()
            )
        ).await.unwrap();
        
        // Authenticate
        let auth_msg = serde_json::json!({
            "type": "auth",
            "id": "test_auth",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": { "token": "test_token" }
        });
        ws_stream.send(Message::Text(auth_msg)).await.unwrap();
        
        // Wait for auth success
        let _ = tokio::time::timeout(Duration::from_secs(5), ws_stream.next()).await;
        
        // Send document update
        let update_msg = serde_json::json!({
            "type": "document_update",
            "id": "test_update",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": {
                "document_id": "doc_123",
                "version": 1,
                "operation": "insert",
                "content": "Test content"
            }
        });
        ws_stream.send(Message::Text(update_msg)).await.unwrap();
        
        // Receive broadcast
        let broadcast = tokio::time::timeout(Duration::from_millis(100), ws_stream.next()).await;
        match broadcast {
            Ok(Message::Text(text)) => {
                let response: serde_json::from_str::<serde_json::Value>(&text).unwrap();
                assert_eq!(response["type"], "document_update");
                assert_eq!(response["data"]["document_id"], "doc_123");
            }
            _ => panic!("Expected broadcast message"),
        }
        
        // Close connection
        ws_stream.close(None).await;
    }
}
```

### 10.4. Performance Testing

#### 10.4.1. Load Testing

```rust
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::test]
async fn test_rest_api_performance() {
    let client = reqwest::Client::new();
        let iterations = 100;
        
        let start = Instant::now();
        
        for i in 0..iterations {
            let response = client
                .get("https://api.tachyon.io/v1/documents")
                .header("Authorization", "Bearer test_token")
                .send()
                .await;
            
            assert!(response.status().is_success());
        }
        
        let duration = start.elapsed();
        let avg_latency = duration.as_millis() as f64 / iterations as f64;
        
        assert!(avg_latency < 100.0, "Average latency should be under 100ms");
    }
```

#### 10.4.2. WebSocket Performance

```rust
#[tokio::test]
async fn test_websocket_latency() {
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(
            tungstenite::handshake::client::Request::from_url(
                url::Url::parse("ws://localhost:8080/v1/ws").unwrap()
            )
        ).await.unwrap();
    
    // Authenticate
    let auth_msg = serde_json::json!({
            "type": "auth",
            "id": "test_auth",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": { "token": "test_token" }
        });
    ws_stream.send(Message::Text(auth_msg)).await.unwrap();
    
    // Wait for auth success
    let _ = tokio::time::timeout(Duration::from_secs(5), ws_stream.next()).await;
    
    let iterations = 1000;
    let latencies: Vec<f64> = Vec::new();
    
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        
        // Send ping
        let ping_msg = serde_json::json!({
            "type": "ping",
            "id": format!("ping_{}", _),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": { "sequence": _ }
        });
        ws_stream.send(Message::Text(ping_msg)).await.unwrap();
        
        // Wait for pong
        let pong = tokio::time::timeout(Duration::from_millis(50), ws_stream.next()).await;
        match pong {
            Ok(Message::Text(text)) => {
                let response: serde_json::from_str::<serde_json::Value>(&text).unwrap();
                assert_eq!(response["type"], "pong");
            }
            _ => {}
        }
        
        latencies.push(start.elapsed().as_millis() as f64);
    }
    
    let avg_latency: latencies.iter().sum::<f64>() / latencies.len() as f64;
    
    assert!(avg_latency < 50.0, "Average WebSocket latency should be under 50ms");
}
```

### 10.5. Security Testing

#### 10.5.1. Authentication Tests

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unauthorized_access() {
        let client = reqwest::Client::new();
        
        let response = client
            .get("https://api.tachyon.io/v1/documents/doc_123")
            .send()
            .await;
        
        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
        let error: ApiError = response.json().await.unwrap();
        assert_eq!(error.code, "UNAUTHORIZED");
    }
    
    #[tokio::test]
    async fn test_invalid_token() {
        let client = reqwest::Client::new();
        
        let response = client
            .get("https://api.tachyon.io/v1/documents/doc_123")
            .header("Authorization", "Bearer invalid_token")
            .send()
            .await;
        
        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
        let error: ApiError = response.json().await.unwrap();
        assert_eq!(error.code, "UNAUTHORIZED");
    }
    
    #[tokio::test]
    async fn test_expired_token() {
        // This test requires mocking time
        // Implementation would verify expired tokens are rejected
    }
}
```

#### 10.5.2. Input Validation Tests

```rust
use validator::ValidateLength;

#[derive(Debug, ValidateLength)]
struct TestDocument {
    #[validate(length(min = 1, max = 100))]
    title: String,
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    #[tokio::test]
    fn test_title_too_short() {
        let doc = TestDocument {
            title: "".to_string(),
        };
        
        let result = doc.validate();
        assert!(result.is_err());
    }
    
    #[tokio::test]
    fn test_title_too_long() {
        let doc = TestDocument {
            title: "a".repeat(101),
        };
        
        let result = doc.validate();
        assert!(result.is_err());
    }
    
    #[tokio::test]
    fn test_content_too_large() {
        let content = "x".repeat(10 * 1024 * 1024); // 10 MB
        
        assert!(content.len() > 10 * 1024 * 1024, "Content exceeds 10MB limit");
    }
}
```

---

---

## 11. ERROR HANDLING

### 11.1. Error Handling Overview

The Tachyon integration framework implements comprehensive error handling across all integration types. Error handling ensures system reliability, user experience, and security by preventing information leakage and enabling graceful degradation.

**Error Handling Principles:**

1. **Fail-Safe Defaults** - System fails safely on errors
2. **No Information Leakage** - Errors don't expose sensitive data
3. **User-Friendly Messages** - Clear, actionable error messages
4. **Comprehensive Logging** - All errors logged with context
5. **Retry with Backoff** - Transient errors automatically retried
6. **Circuit Breaking** - Failed integrations temporarily disabled
7. **Error Classification** - Errors categorized for appropriate handling

### 11.2. Error Types

#### 11.2.1. Error Hierarchy

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    /// Network-related errors
    #[error("Network request failed: {0}")]
    NetworkError {
        source: String,
        #[source] reqwest::Error,
    },
    
    /// Authentication/authorization errors
    #[error("Authentication failed: {0}")]
    AuthenticationError {
        reason: String,
    },
    
    /// Validation errors
    #[error("Request validation failed: {0}")]
    ValidationError {
        field: String,
        message: String,
    },
    
    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimitError {
        retry_after: chrono::DateTime<chrono::Utc>,
    },
    
    /// Timeout errors
    #[error("Operation timed out: {0}")]
    TimeoutError {
        operation: String,
        timeout_ms: u64,
    },
    
    /// Internal server errors
    #[error("Internal server error: {0}")]
    InternalError {
        code: String,
        message: String,
    },
    
    /// Plugin errors
    #[error("Plugin operation failed: {0}")]
    PluginError {
        plugin_id: String,
        operation: String,
    },
}
```

#### 11.2.2. Error Classification

| Category | Error Types | Retryable | User Action |
|----------|-------------|-----------|-------------|
| **Transient** | NetworkError, TimeoutError | Yes | Retry automatically |
| **Permanent** | ValidationError, AuthenticationError | No | Show error message |
| **Rate Limited** | RateLimitError | Yes | Wait and retry |
| **Internal** | InternalError | Yes | Contact support |

### 11.3. REST Error Handling

#### 11.3.1. Error Response Format

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub request_id: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "None")]
    pub details: Option<serde_json::Value>,
}
```

#### 11.3.2. Error Codes

| Code | HTTP Status | Category | Description | Retryable |
|------|-------------|-----------|-------------|-----------|
| **VALIDATION_ERROR** | 400 | Validation | No | Fix request |
| **UNAUTHORIZED** | 401 | Authentication | No | Re-authenticate |
| **FORBIDDEN** | 403 | Authorization | No | Check permissions |
| **NOT_FOUND** | 404 | Resource | No | Verify resource exists |
| **CONFLICT** | 409 | Resource | Yes | Resolve conflict |
| **RATE_LIMITED** | 429 | Rate Limit | Yes | Wait and retry |
| **INTERNAL_ERROR** | 500 | Server | Yes | Retry later |
| **SERVICE_UNAVAILABLE** | 503 | Server | Yes | Retry later |

#### 11.3.3. Error Handling Middleware

```rust
use axum::{
    extract::Path,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, Json},
    Json,
};
use serde_json::json;

pub async fn error_handler(
    error: IntegrationError,
) -> impl IntoResponse for IntegrationError {
    let (request_id, request_body) = error.request_id_and_body();
    
    match error {
        IntegrationError::NetworkError { source } => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let response = Json(json!({
                "error": {
                    "code": "NETWORK_ERROR",
                    "message": "Network request failed",
                    "request_id": request_id,
                }
            }));
            (status, response).into_response()
        }
        
        IntegrationError::AuthenticationError { reason } => {
            let status = StatusCode::UNAUTHORIZED;
            let response = Json(json!({
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": reason,
                    "request_id": request_id,
                }
            }));
            (status, response).into_response()
        }
        
        IntegrationError::ValidationError { field, message } => {
            let status = StatusCode::BAD_REQUEST;
            let mut details = serde_json::json::({});
            details[field] = message.into();
            let response = Json(json!({
                "error": {
                    "code": "VALIDATION_ERROR",
                    "message": format!("Invalid {}", field),
                    "details": details,
                    "request_id": request_id,
                }
            }));
            (status, response).into_response()
        }
        
        IntegrationError::RateLimitError { retry_after } => {
            let status = StatusCode::TOO_MANY_REQUESTS;
            let response = Json(json!({
                "error": {
                    "code": "RATE_LIMITED",
                    "message": "Rate limit exceeded",
                    "retry_after": retry_after.to_rfc3339(),
                    "request_id": request_id,
                }
            }));
            (status, response).into_response()
        }
        
        IntegrationError::TimeoutError { operation, timeout_ms } => {
            let status = StatusCode::REQUEST_TIMEOUT;
            let response = Json(json!({
                "error": {
                    "code": "TIMEOUT",
                    "message": format!("{} timed out after {}ms", operation, timeout_ms),
                    "request_id": request_id,
                }
            }));
            (status, response).into_response()
        }
        
        IntegrationError::InternalError { code, message } => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let response = Json(json!({
                "error": {
                    "code": code,
                    "message": message,
                    "request_id": request_id,
                }
            }));
            (status, response).into_response()
        }
        
        _ => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let response = Json(json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": "An unexpected error occurred",
                    "request_id": request_id,
                }
            }));
            (status, response).into_response()
        }
    }
}
```

### 11.4. WebSocket Error Handling

#### 11.4.1. Error Message Format

```json
{
  "type": "error",
  "id": "msg_abc123",
  "timestamp": "2026-02-07T22:45:00.789Z",
  "data": {
    "code": "CONNECTION_LOST",
    "message": "WebSocket connection lost",
    "details": {
      "reason": "Network timeout",
      "reconnect": true
    }
  }
}
```

#### 11.4.2. Error Codes

| Code | Description | Action |
|------|-------------|--------|
| **CONNECTION_ERROR** | WebSocket connection failed | Attempt reconnection |
| **AUTHENTICATION_FAILED** | Invalid credentials | Re-authenticate |
| **INVALID_MESSAGE** | Malformed message | Log and ignore |
| **RATE_LIMITED** | Too many messages | Throttle sending |
| **TIMEOUT** | No heartbeat received | Send ping |
| **INTERNAL_ERROR** | Server error | Log and reconnect |

#### 11.4.3. Error Handling

```rust
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub async fn handle_websocket_errors(
    ws_stream: &mut WebSocketStream<tokio_tungstenite::MaybeTlsStream>,
) -> Result<(), IntegrationError> {
    loop {
        match ws_stream.next().await {
            Ok(Some(message)) => match message {
                Message::Text(text) => {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        match event["type"].as_str() {
                            "error" => {
                                // Handle error message
                                eprintln!("WebSocket error: {:?}", event);
                                
                                // Send acknowledgment
                                let ack = serde_json::json!({
                                    "type": "error_ack",
                                    "id": event["id"].as_str().unwrap_or(""),
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                });
                                ws_stream.send(Message::Text(ack)).await
                                    .map_err(|e| IntegrationError::InternalError {
                                        code: "INTERNAL_ERROR".to_string(),
                                        message: "Failed to send error acknowledgment".to_string(),
                                    })?;
                            }
                            _ => {}
                        }
                    } else {
                        // Log unknown message type
                        eprintln!("Unknown WebSocket message type: {:?}", message);
                    }
                }
                
                Ok(Message::Close(close_frame)) => {
                    eprintln!("WebSocket closed: {:?}", close_frame);
                    return Err(IntegrationError::NetworkError {
                        source: "WebSocket connection closed".to_string(),
                    });
                }
                
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    return Err(IntegrationError::InternalError {
                        code: "INTERNAL_ERROR".to_string(),
                        message: format!("WebSocket error: {}", e),
                    });
                }
            }
        }
    }
}
```

### 11.5. IPC Error Handling

#### 11.5.1. Error Response Format

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct IpcResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<IpcError>,
}

#[derive(Serialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
}
```

#### 11.5.2. Error Codes

| Code | Description | User Action |
|------|-------------|--------|-------------|
| **COMMAND_FAILED** | Command execution failed | Retry or report |
| **PERMISSION_DENIED** | Insufficient permissions | Request permission |
| **INVALID_PARAMETERS** | Invalid parameters | Fix request |
| **SERVICE_UNAVAILABLE** | Backend service down | Wait and retry |
| **TIMEOUT** | Command timeout | Retry with longer timeout |

#### 11.5.3. Error Handling

```rust
use tauri::State;

pub async fn handle_ipc_command<R>(
    state: State<'_>,
    command_name: &str,
    args: R,
    handler: impl FnOnce<(&State<'_>, R) -> Result<R, String>>,
) -> IpcResponse<R> {
    match handler.call_once(&state, args).await {
        Ok(result) => IpcResponse {
            success: true,
            data: Some(result),
            error: None,
        },
        Err(e) => IpcResponse {
            success: false,
            data: None,
            error: Some(IpcError {
                code: "COMMAND_FAILED".to_string(),
                message: e.to_string(),
            }),
        },
    }
}
```

### 11.6. Webhook Error Handling

#### 11.6.1. Delivery Status Tracking

```rust
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct DeliveryStatus {
    pub webhook_id: String,
    pub event_id: String,
    pub status: DeliveryStatus,
    pub attempts: usize,
    pub last_error: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryStatus {
    #[strum(display = "Pending")]
    Pending,
    #[strum(display = "Delivered")]
    Delivered,
    #[strum(display = "Failed")]
    Failed,
    #[strum(display = "RetryScheduled")]
    RetryScheduled,
}
```

#### 11.6.2. Retry Logic

```rust
use tokio::time::{sleep, Duration};

pub async fn deliver_webhook_with_retry(
    webhook: &Webhook,
    event: &WebhookEvent,
    config: &WebhookRetryConfig,
) -> Result<(), IntegrationError> {
    let mut attempt = 0;
    let mut backoff = config.initial_backoff_ms;
    
    loop {
        attempt += 1;
        
        match deliver_webhook(webhook, event).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < config.max_attempts => {
                eprintln!("Webhook delivery attempt {} failed: {}", attempt, e);
                
                // Calculate next backoff with jitter
                let jitter = (backoff as f64 * config.jitter_factor) as u64;
                let delay = backoff + jitter;
                
                sleep(Duration::from_millis(delay)).await;
                backoff = (backoff as f64 * config.backoff_multiplier) as u64;
                backoff = backoff.min(config.max_backoff_ms);
            }
            Err(e) => {
                return Err(IntegrationError::NetworkError {
                    source: format!("Webhook delivery failed after {} attempts", config.max_attempts),
                });
            }
        }
    }
}
```

### 11.7. OAuth Error Handling

#### 11.7.1. Error Response Format

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct OAuthError {
    pub code: String,
    pub message: String,
    pub provider: String,
    pub details: Option<serde_json::Value>,
}
```

#### 11.7.2. Error Codes

| Code | Description | User Action |
|------|-------------|--------|-------------|
| **AUTH_ERROR** | Authentication failed | Re-authenticate |
| **INVALID_STATE** | Invalid OAuth state | Restart flow |
| **TOKEN_EXPIRED** | Access token expired | Refresh token |
| **PROVIDER_ERROR** | Provider error | Try again later |
| **USER_DENIED** | User denied authorization | Check permissions |
| **RATE_LIMITED** | Too many requests | Wait and retry |

---

---

## 12. SECURITY CONSIDERATIONS

### 12.1. Security Overview

The Tachyon integration security architecture implements defense-in-depth security controls aligned with [ADR-010: Security Architecture](.specs/02_adrs/010_security_architecture.md). Security is enforced at all integration layers including REST API, WebSocket, IPC, webhooks, OAuth, and plugins.

**Security Principles:**

1. **Defense-in-Depth** - Multiple security layers for redundancy
2. **Zero Trust** - No trust assumptions within security boundaries
3. **Principle of Least Privilege** - Minimal access required for operations
4. **Secure by Default** - All integrations use secure defaults
5. **Fail-Safe** - System fails safely on security violations
6. **Audit Logging** - Comprehensive security event logging
7. **Supply Chain Security** - Verified dependencies and builds

### 12.2. REST API Security

#### 12.2.1. Authentication

**JWT Token Security:**

```rust
use jsonwebtoken::{encode, AlgorithmHeader, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: String,
    pub jti: String,
}

/// Generate JWT token
pub fn generate_jwt_token(
    user_id: &str,
    secret: &DecodingKey,
) -> Result<String, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();
    let exp = now.timestamp() + chrono::Duration::hours(1);
    
    let mut header = Header::new(Algorithm::HS256);
    header.typ = "JWT";
    header.kid = "some-secret-key";
    
    let claims = Claims {
        sub: user_id,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        nbf: String::new(),
        jti: uuid::Uuid::new_v4().to_string(),
    };
    
    encode(&header, &claims, secret)
        .map_err(|e| e.to_string())
}

/// Validate JWT token
pub fn validate_jwt_token(
    token: &str,
    secret: &DecodingKey,
) -> Result<Claims, Box<dyn std::error::Error>> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp(chrono::Utc::now() + chrono::Duration::hours(1))?;
    
    let token_data = decode::<Validation>(token, secret, &Validation::new(Algorithm::HS256))
        .map_err(|e| e.to_string())?;
    
    let claims: token_data.claims;
    validation.validate_exp(claims.exp)?;
    validation.validate_aud(claims.exp)?;
    
    Ok(claims)
}
```

#### 12.2.2. TLS Configuration

```rust
use rustls::{ClientConfig, ServerConfig};
use rustls_pemfile::{certs_and_keys, PrivateKey};
use std::sync::Arc;

/// Create TLS configuration
pub fn create_tls_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let certs = certs_and_keys("certs/server.pem")
        .map_err(|e| format!("Failed to load certificates: {}", e))?;
    
    let key = PrivateKey::new(&certs_and_keys("keys/server.key"))
        .map_err(|e| format!("Failed to load private key: {}", e))?;
    
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![Arc::new(cert)])
        .with_single_private_key(Arc::new(key));
    
    Ok(config)
}
```

### 12.3. WebSocket Security

#### 12.3.1. Connection Security

```rust
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use sha2::{Digest, Sha256};

/// Generate WebSocket signature
pub fn generate_websocket_signature(
    secret: &str,
    payload: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(payload.as_bytes());
    
    hex::encode(hasher.finalize())
}

/// Verify WebSocket signature
pub fn verify_websocket_signature(
    secret: &str,
    payload: &str,
    signature: &str,
) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(payload.as_bytes());
    
    let expected = hex::encode(hasher.finalize());
    
    // Constant-time comparison to prevent timing attacks
    if signature.len() != expected.len() {
        return false;
    }
    
    let mut result = true;
    for (a, b) in signature.bytes().zip(expected.bytes()) {
        result &= a == b;
    }
    
    result
}
```

#### 12.3.2. Message Validation

```rust
use serde_json::json;
use validator::ValidateLength;

#[derive(Debug, Deserialize, ValidateLength)]
pub struct DocumentUpdateMessage {
    #[validate(length(min = 1, max = 100))]
    pub document_id: String,
    
    #[validate(length(min = 1))]
    pub version: u32,
    
    #[validate(length(min = 1))]
    pub operation: String,
    
    #[validate(length(max = 10000000))] // 10MB
    pub content: String,
}

pub fn validate_websocket_message(
    message: &DocumentUpdateMessage,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    
    if message.document_id.is_empty() {
        errors.push("Document ID is required".to_string());
    }
    
    if message.content.len() > 10000000 {
        errors.push("Content exceeds 10MB limit".to_string());
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### 12.4. IPC Security

#### 12.4.1. Capability-Based Access Control

```rust
use tauri::State;

/// Check if command is allowed
pub async fn check_command_capability(
    state: State<'_>,
    command: &str,
    capability: &str,
) -> Result<(), String> {
    // Check if capability is granted
    let capabilities = state.capabilities();
    
    if !capabilities.contains(capability) {
        return Err(format!(
            "Capability {} is not granted for command {}",
            capability, command
        ));
    }
    
    Ok(())
}
```

#### 12.4.2. Input Sanitization

```rust
use ammonia::{clean, Url, UrlWrite};

/// Sanitize user input
pub fn sanitize_input(input: &str) -> String {
    clean(&input)
        .map_err(|e| e.to_string())
}

/// Sanitize file path
pub fn sanitize_path(path: &str) -> Result<String, String> {
    // Prevent path traversal
    if path.contains("..") || path.starts_with('/') {
        return Err("Path traversal not allowed".to_string());
    }
    
    Ok(path.to_string())
}
```

### 12.5. Webhook Security

#### 12.5.1. Signature Verification

```rust
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use hex;

/// Generate HMAC signature
pub fn generate_webhook_signature(
    secret: &str,
    payload: &str,
    timestamp: i64,
) -> String {
    let mut mac = Hmac::new(Sha256::new());
    mac.update(secret.as_bytes());
    mac.update(timestamp.to_be_bytes());
    mac.update(payload.as_bytes());
    
    let result = mac.finalize();
    hex::encode(result)
}

/// Verify HMAC signature
pub fn verify_webhook_signature(
    secret: &str,
    payload: &str,
    timestamp: i64,
    signature: &str,
) -> bool {
    let mut mac = Hmac::new(Sha256::new());
    mac.update(secret.as_bytes());
    mac.update(timestamp.to_be_bytes());
    mac.update(payload.as_bytes());
    
    let expected = hex::encode(mac.finalize());
    
    // Constant-time comparison
    if signature.len() != expected.len() {
        return false;
    }
    
    let mut result = true;
    for (a, b) in signature.bytes().zip(expected.bytes()) {
        result &= a == b;
    }
    
    result
}
```

#### 12.5.2. Replay Protection

```rust
use std::collections::HashSet;
use std::sync::Mutex;

/// Replay protection for webhooks
pub struct ReplayProtection {
    seen_events: Mutex<HashSet<String>>,
    ttl_seconds: u64,
}

impl ReplayProtection {
    pub fn new(ttl_seconds: u64) -> Self {
        ReplayProtection {
            seen_events: Mutex::new(HashSet::new()),
            ttl_seconds,
        }
    }
    
    pub fn check_and_add(&self, event_id: &str) -> bool {
        let mut seen = self.seen_events.lock().unwrap();
        
        // Check if event exists and is within TTL
        if let Some(&timestamp) = seen.get(event_id) {
            let elapsed = chrono::Utc::now().timestamp() - timestamp;
            if elapsed.num_seconds() < self.ttl_seconds {
                // Event is too old, reject
                return false;
            } else {
                // Event is not in cache or expired, accept
                seen.insert(event_id.to_string(), chrono::Utc::now().timestamp());
                return true;
            }
        }
    }
}
```

### 12.6. OAuth Security

#### 12.6.1. PKCE Implementation

```rust
use base64::{Engine as Base64Engine, engine::general_purpose::STANDARD};
use sha2::{Sha256, Digest};
use rand::Rng;

/// Generate PKCE code verifier
pub fn generate_pkce_verifier() -> (String, String) {
    let mut rng = rand::thread_rng();
    let code_verifier: [u8; 128] = rng.gen();
    
    // Encode to base64 URL-safe
    let code_verifier_b64 = Base64Engine::encode(&code_verifier);
    let code_verifier_url = code_verifier_b64
        .replace("+", "-")
        .replace("/", "_")
        .replace("=", "");
    
    let code_challenge = generate_code_challenge();
    
    (code_verifier_url, code_challenge)
}

/// Generate code challenge
fn generate_code_challenge() -> String {
    let mut rng = rand::thread_rng();
    let code_challenge: [u8; 128] = rng.gen();
    
    // Hash with SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&code_challenge);
    
    // Encode to base64 URL-safe
    let hash_b64 = Base64Engine::encode(&hasher.finalize());
    hash_b64
        .replace("+", "-")
        .replace("/", "_")
        .replace("=", "");
    
    hash_b64
}
```

#### 12.6.2. Token Storage Security

```rust
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

/// Store tokens securely
pub fn store_tokens(
    tokens: &StoredTokens,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create secure directory
    let config_dir = dirs::config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let token_dir = config_dir.join("tachyon/tokens");
    tokio::fs::create_dir_all(&token_dir).await
        .map_err(|e| format!("Failed to create token directory: {}", e))?;
    
    // Write tokens with restricted permissions
    let token_file = token_dir.join("tokens.json");
    let token_content = serde_json::to_string(tokens)?;
    tokio::fs::write(&token_file, token_content)
        .await
        .map_err(|e| format!("Failed to write tokens: {}", e))?;
    
    // Set restrictive permissions (read/write only by owner)
    #[cfg(unix)]
    {
        use tokio::fs::set_permissions;
        tokio::fs::set_permissions(&token_file, 0o600)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    
    Ok(())
}

/// Load tokens securely
pub async fn load_tokens() -> Result<StoredTokens, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let token_file = config_dir.join("tachyon/tokens.json");
    
    let content = tokio::fs::read_to_string(&token_file).await
        .map_err(|e| format!("Failed to read tokens: {}", e))?;
    
    let tokens: StoredTokens = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tokens: {}", e))?;
    
    Ok(tokens)
}
```

### 12.7. Plugin Security

#### 12.7.1. Resource Monitoring

```rust
use std::time::{Duration, Instant};

/// Plugin resource monitor
pub struct PluginResourceMonitor {
    max_memory_mb: f64,
    max_cpu_percent: f64,
    max_network_kbps: f64,
}

impl PluginResourceMonitor {
    pub fn new(max_memory_mb: f64, max_cpu_percent: f64, max_network_kbps: f64) -> Self {
        PluginResourceMonitor {
            max_memory_mb,
            max_cpu_percent,
            max_network_kbps,
        }
    }
    
    pub fn check_limits(&self, usage: &ResourceUsage) -> bool {
        let memory_exceeded = usage.memory_mb > self.max_memory_mb;
        let cpu_exceeded = usage.cpu_percent > self.max_cpu_percent;
        let network_exceeded = usage.network_kbps > self.max_network_kbps;
        
        !memory_exceeded && !cpu_exceeded && !network_exceeded
    }
}
```

#### 12.7.2. Sandboxing

```rust
use wasmer::Runtime;

/// Create sandboxed WASM runtime
pub fn create_sandboxed_runtime() -> Result<Runtime, String> {
    let mut config = wasmer::Config::default();
    
    // Limit memory
    config.max_memory_pages(Some(128)); // 128 pages (~8MB)
    
    // Limit CPU
    config.max_wasm_threads(Some(4)); // 4 threads
    
    // Disable WASI features that could escape sandbox
    config.wasi(false);
    
    let runtime = Runtime::new(&config)
        .map_err(|e| format!("Failed to create WASM runtime: {}", e))?;
    
    Ok(runtime)
}
```

---

**End of Chunk 11**
