# TACHYON: IPC API DOCUMENTATION

**Document ID:** TACHYON-API-004-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Technical Documentation - API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [IPC API Framework](#2-ipc-api-framework)
3. [JSON-RPC Specification](#3-json-rpc-specification)
4. [Document Commands](#4-document-commands)
5. [Git Commands](#5-git-commands)
6. [Window Commands](#6-window-commands)
7. [Dialog Commands](#7-dialog-commands)
8. [Configuration Commands](#8-configuration-commands)
9. [Plugin Commands](#9-plugin-commands)
10. [Event Commands](#10-event-commands)
11. [Error Handling](#11-error-handling)
12. [Security Considerations](#12-security-considerations)
13. [References](#13-references)

---

## 1. INTRODUCTION

### 1.1. Purpose and Scope

This document provides comprehensive technical documentation for the Inter-Process Communication (IPC) Application Programming Interface (API) of the Tachyon toolchain. The IPC API enables secure, type-safe communication between the Tauri desktop application frontend (WebView) and the Rust backend services, as well as facilitating communication between web frontend components and the HTTP/2 server.

The Tachyon IPC API is designed to meet the following objectives:

1. **Type Safety:** Compile-time type checking for all IPC messages to prevent runtime errors
2. **Security:** Capability-based authorization with session-based authentication
3. **Performance:** Sub-millisecond message latency with efficient serialization
4. **Reliability:** Comprehensive error handling with proper error propagation
5. **Bidirectional Communication:** Support for both request-response and event-driven patterns
6. **Extensibility:** Versioned message formats for backward compatibility

### 1.2. Document Dependencies

This document depends on the following specifications and architectural decisions:

- [TACHYON-STD-V1.0](../.adrs/ - Coding and Documentation Standards
- [TACHYON-ADR-009-V1.0](../.adrs/adr-009-race-condition-mitigation.md) - IPC Communication Architecture
- [TACHYON-ADR-010-V1.0](../.adrs/adr-010-synchronization-primitives.md) - Security Architecture
- [TACHYON-REQ-IPC-V1.0](../.adrs/ - IPC Communication Requirements
- [TACHYON-DES-IPC-V1.0](../.adrs/ - IPC Protocol Design

### 1.3. Target Audience

This document is intended for:

- **Desktop Application Developers:** Developers working on the Tauri-based desktop application
- **Backend Engineers:** Engineers implementing Rust backend services
- **Frontend Developers:** Developers working on the Leptos-based web frontend
- **System Integrators:** Engineers integrating Tachyon with external systems
- **Security Auditors:** Personnel conducting security reviews of the IPC layer
- **QA Engineers:** Testers verifying IPC functionality and security

### 1.4. Conventions and Notation

Throughout this document, the following conventions are used:

- **Rust Code:** Code blocks with `rust` syntax highlighting denote Rust implementations
- **JavaScript Code:** Code blocks with `javascript` syntax highlighting denote frontend implementations
- **JSON:** Code blocks with `json` syntax highlighting denote serialized message formats
- **Requirement References:** `[REQ-IPC-XXX]` denotes references to IPC requirements
- **ADR References:** `[ADR-009]` denotes references to Architectural Decision Records
- **Design References:** `[DES-IPC-XXX]` denotes references to IPC design elements

---

## 2. IPC API FRAMEWORK

### 2.1. Architecture Overview

The Tachyon IPC API framework is built upon Tauri's IPC mechanisms, providing a robust foundation for inter-process communication. The architecture consists of three primary components:

1. **Command System:** Request-response communication from frontend to backend
2. **Event System:** Publish-subscribe communication from backend to frontend
3. **Authentication Layer:** JWT-based session management and capability enforcement

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri Desktop Application                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐         ┌─────────────┐         ┌─────────────┐ │
│  │  WebView    │────────-│  IPC Bridge │────────-│  Rust       │ │
│  │  Frontend   │  Commands│  Layer      │  Events  │  Backend    │ │
│  │  (Leptos)   │◀────────│             │◀────────│  Services   │ │
│  └─────────────┘  Events  └─────────────┘  Commands└─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2. Communication Channels

The IPC API provides three distinct communication channels:

#### 2.2.1. Command Channel

The command channel enables request-response communication from the frontend to the backend. Each command is a structured message with a defined request and response schema.

**Characteristics:**
- **Direction:** Frontend → Backend
- **Pattern:** Request-Response
- **Guarantees:** Exactly-once delivery, ordered processing
- **Timeout:** Configurable per-command (default: 30 seconds)
- **Error Handling:** Structured error responses with error codes

**Example Command Flow:**

```rust
// Frontend (JavaScript/TypeScript)
import { invoke } from '@tauri-apps/api/core';

const response = await invoke<GetDocumentResponse>('get_document', {
  id: 'doc-123'
});

// Backend (Rust)
#[tauri::command]
pub async fn get_document(
    id: String,
    state: State<'_, AppState>,
) -> Result<GetDocumentResponse, IpcError> {
    let document = state.core.get_document(&id).await
        .map_err(IpcError::from)?;
    
    Ok(GetDocumentResponse { document })
}
```

#### 2.2.2. Event Channel

The event channel enables publish-subscribe communication from the backend to the frontend. Events are broadcast to all subscribed listeners.

**Characteristics:**
- **Direction:** Backend → Frontend
- **Pattern:** Publish-Subscribe
- **Guarantees:** At-least-once delivery, ordered within event type
- **Filtering:** Support for event type and custom criteria filtering
- **Deduplication:** Automatic deduplication within configurable time window

**Example Event Flow:**

```rust
// Backend (Rust)
use tauri::Manager;

app.emit_all("document_changed", DocumentChangedEvent {
    document_id: "doc-123".to_string(),
    timestamp: Utc::now(),
    change_type: ChangeType::Modified,
})?;

// Frontend (JavaScript/TypeScript)
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<DocumentChangedEvent>(
    'document_changed',
    (event) => {
        console.log('Document changed:', event.payload);
    }
);
```

#### 2.2.3. Stream Channel

The stream channel enables continuous data transfer for real-time updates. Streams are established for long-running operations that produce incremental results.

**Characteristics:**
- **Direction:** Bidirectional
- **Pattern:** Streaming
- **Guarantees:** Ordered delivery, backpressure support
- **Flow Control:** Backpressure via Tokio channels
- **Termination:** Graceful shutdown with cleanup

**Example Stream Flow:**

```rust
// Backend (Rust)
#[tauri::command]
pub async fn stream_search_results(
    query: SearchQuery,
    app: AppHandle,
) -> Result<(), IpcError> {
    let mut stream = state.core.search_stream(&query).await?;
    
    while let Some(result) = stream.next().await {
        app.emit_all("search_result", result)?;
    }
    
    Ok(())
}
```

### 2.3. Message Envelope

All IPC messages are encapsulated in a standardized message envelope that provides metadata for correlation, authentication, and debugging.

**Message Envelope Structure:**

```rust
/// Base IPC message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessageEnvelope {
    /// Unique message identifier for correlation
    pub message_id: Uuid,
    
    /// Message type discriminator
    pub message_type: MessageType,
    
    /// Message payload
    pub payload: JsonValue,
    
    /// Timestamp when message was created
    pub timestamp: DateTime<Utc>,
    
    /// Optional authentication token
    pub auth_token: Option<String>,
    
    /// Optional correlation ID for request-response matching
    pub correlation_id: Option<Uuid>,
    
    /// Message version for backward compatibility
    pub version: String,
}

/// Message type discriminator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum MessageType {
    /// Command from frontend to backend
    #[serde(rename = "command")]
    Command,
    
    /// Response from backend to frontend
    #[serde(rename = "response")]
    Response,
    
    /// Event from backend to frontend
    #[serde(rename = "event")]
    Event,
    
    /// Error response
    #[serde(rename = "error")]
    Error,
}
```

**Message Lifecycle:**

1. **Creation:** Message envelope created with unique message ID and timestamp
2. **Serialization:** Message payload serialized to JSON via serde
3. **Transmission:** Message transmitted across IPC boundary
4. **Deserialization:** Message deserialized and validated against schema
5. **Processing:** Message processed by appropriate handler
6. **Response:** Response envelope created for command messages
7. **Cleanup:** Message resources released after processing

### 2.4. Type System

The IPC API leverages Rust's type system to ensure compile-time type safety. All command and event definitions are strongly typed, with automatic serialization and deserialization via serde.

**Type Safety Guarantees:**

1. **Compile-Time Checking:** Type mismatches caught at compile time
2. **Automatic Serialization:** serde automatically handles type conversion
3. **Version Compatibility:** Type versioning enables backward compatibility
4. **Documentation:** Types serve as self-documenting contracts

**Type Definition Example:**

```rust
/// Document command request type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentRequest {
    /// Unique document identifier
    #[serde(validate = custom = "validate_document_id")]
    pub id: DocumentId,
    
    /// Optional version to retrieve
    pub version: Option<String>,
}

/// Document command response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentResponse {
    /// Document content
    pub document: Document,

    /// Metadata about the document
    pub metadata: DocumentMetadata,
}
```

---

## 3. JSON-RPC SPECIFICATION

### 3.1. Protocol Overview

The Tachyon IPC API implements a JSON-RPC 2.0 compatible protocol for command communication. This specification defines the message format, request-response semantics, and error handling conventions for IPC commands.

**Protocol Characteristics:**

- **Specification:** JSON-RPC 2.0 compliant
- **Transport:** Tauri IPC bridge
- **Encoding:** UTF-8 JSON
- **Serialization:** serde
- **Versioning:** Protocol version in message envelope
- **Compatibility:** Backward compatible with minor version increments

**Rationale:** JSON-RPC provides a standardized, lightweight protocol for remote procedure calls that aligns with Tachyon's requirements for type-safe, efficient IPC communication. The protocol's simplicity and widespread adoption facilitate integration and debugging.

### 3.2. Request Format

IPC command requests follow the JSON-RPC 2.0 specification with Tachyon-specific extensions for authentication and correlation.

**Request Structure:**

```json
{
  "jsonrpc": "2.0",
  "method": "command_name",
  "params": {
    "param1": "value1",
    "param2": "value2"
  },
  "id": "request-id",
  "tachyon": {
    "version": "1.0",
    "auth_token": "optional-jwt-token",
    "correlation_id": "optional-correlation-id"
  }
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | String | Yes | JSON-RPC version, must be "2.0" |
| `method` | String | Yes | Name of the command to execute |
| `params` | Object/Array | No | Parameters for the command |
| `id` | String/Number/Null | Yes | Request identifier for response correlation |
| `tachyon` | Object | No | Tachyon-specific metadata |

**Tachyon Metadata Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | String | Yes | IPC protocol version |
| `auth_token` | String | Conditional | JWT authentication token |
| `correlation_id` | String | No | Correlation ID for distributed tracing |

**Request Example:**

```json
{
  "jsonrpc": "2.0",
  "method": "get_document",
  "params": {
    "id": "doc-123",
    "version": null
  },
  "id": "req-001",
  "tachyon": {
    "version": "1.0",
    "auth_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "correlation_id": "corr-001"
  }
}
```

### 3.3. Response Format

IPC command responses follow the JSON-RPC 2.0 specification with Tachyon-specific extensions for performance metrics and tracing.

**Success Response Structure:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "field1": "value1",
    "field2": "value2"
  },
  "id": "request-id",
  "tachyon": {
    "version": "1.0",
    "duration_ms": 5.2,
    "trace_id": "trace-id"
  }
}
```

**Error Response Structure:**

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "field": "id",
      "reason": "Invalid document ID format"
    }
  },
  "id": "request-id",
  "tachyon": {
    "version": "1.0",
    "duration_ms": 0.8,
    "trace_id": "trace-id"
  }
}
```

**Response Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | String | Yes | JSON-RPC version, must be "2.0" |
| `result` | Any | Conditional | Result of successful command execution |
| `error` | Object | Conditional | Error object for failed commands |
| `id` | String/Number/Null | Yes | Request identifier from original request |
| `tachyon` | Object | No | Tachyon-specific metadata |

**Error Object Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `code` | Integer | Yes | Error code (see Section 11) |
| `message` | String | Yes | Human-readable error message |
| `data` | Any | No | Additional error context |

**Success Response Example:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "document": {
      "id": "doc-123",
      "title": "Example Document",
      "content": "Document content here..."
    },
    "metadata": {
      "created_at": "2026-02-07T16:00:00Z",
      "modified_at": "2026-02-07T16:30:00Z",
      "author": "user@example.com"
    }
  },
  "id": "req-001",
  "tachyon": {
    "version": "1.0",
    "duration_ms": 5.2,
    "trace_id": "trace-abc123"
  }
}
```

### 3.4. Notification Format

Notifications are special requests that do not expect a response. They are used for one-way communication where the sender does not need confirmation.

**Notification Structure:**

```json
{
  "jsonrpc": "2.0",
  "method": "notification_name",
  "params": {
    "param1": "value1"
  },
  "tachyon": {
    "version": "1.0"
  }
}
```

**Notification Characteristics:**

- **ID Field:** Omitted (null) to indicate no response expected
- **Use Cases:** Logging, telemetry, fire-and-forget operations
- **Guarantees:** Best-effort delivery, no acknowledgment

**Notification Example:**

```json
{
  "jsonrpc": "2.0",
  "method": "log_event",
  "params": {
    "level": "info",
    "message": "User opened document",
    "document_id": "doc-123"
  },
  "tachyon": {
    "version": "1.0"
  }
}
```

### 3.5. Batch Requests

The IPC API supports batch requests for executing multiple commands in a single IPC transaction. This reduces overhead for operations that can be executed independently.

**Batch Request Structure:**

```json
[
  {
    "jsonrpc": "2.0",
    "method": "get_document",
    "params": { "id": "doc-001" },
    "id": "req-001"
  },
  {
    "jsonrpc": "2.0",
    "method": "get_document",
    "params": { "id": "doc-002" },
    "id": "req-002"
  },
  {
    "jsonrpc": "2.0",
    "method": "list_documents",
    "params": {},
    "id": "req-003"
  }
]
```

**Batch Response Structure:**

```json
[
  {
    "jsonrpc": "2.0",
    "result": { "document": { "id": "doc-001" } },
    "id": "req-001"
  },
  {
    "jsonrpc": "2.0",
    "result": { "document": { "id": "doc-002" } },
    "id": "req-002"
  },
  {
    "jsonrpc": "2.0",
    "result": { "documents": [...] },
    "id": "req-003"
  }
]
```

**Batch Request Guarantees:**

- **Atomicity:** All requests in batch are executed atomically
- **Ordering:** Responses maintain the same order as requests
- **Error Isolation:** Failure of one request does not affect others
- **Performance:** Reduced IPC overhead for multiple operations

### 3.6. Protocol Compliance

The Tachyon IPC API implements the following JSON-RPC 2.0 features:

**Implemented Features:**

| Feature | Status | Notes |
|---------|--------|-------|
| Request-Response | [PASS] Implemented | Full support with correlation |
| Notifications | [PASS] Implemented | Fire-and-forget operations |
| Batch Requests | [PASS] Implemented | Atomic batch execution |
| Named Parameters | [PASS] Implemented | Object-based parameter passing |
| Positional Parameters | [WARN] Deprecated | Use named parameters instead |
| Error Codes | [PASS] Implemented | Custom Tachyon error codes (see Section 11) |

**Deviations from JSON-RPC 2.0:**

1. **Tachyon Metadata Extension:** Additional `tachyon` object for versioning, authentication, and tracing
2. **Type Safety:** Strong typing via Rust type system exceeds JSON-RPC requirements
3. **Authentication:** Built-in JWT authentication not specified in JSON-RPC 2.0
4. **Performance Metrics:** Duration and trace ID in responses for observability

---

## 4. DOCUMENT COMMANDS

### 4.1. Command Overview

Document commands provide comprehensive CRUD (Create, Read, Update, Delete) operations for managing documents within the Tachyon system. These commands enable the frontend to interact with the document storage and retrieval subsystems.

**Related Requirements:**
- [REQ-IPC-016](../.adrs/ - Document Commands
- [REQ-FR-003](../.adrs/ - Document Management

**Related Design Elements:**
- [DES-IPC-001](../.adrs/ - DocumentCommand
- [DES-DM-001](../.adrs/ - DocumentId
- [DES-DM-003](../.adrs/ - DocumentMetadata
- [DES-DM-004](../.adrs/ - DocumentContent

### 4.2. get_document

Retrieves a document by its unique identifier.

**Method:** `get_document`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `id` | String | Yes | Unique document identifier |
| `version` | String/Null | No | Optional version to retrieve (null for latest) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `document` | Object | Document object with content and metadata |
| `metadata` | Object | Document metadata (created_at, modified_at, author) |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Document ID format is invalid |
| -32603 | Internal error | Document retrieval failed |
| 404 | Document not found | Document with specified ID does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "get_document",
  "params": {
    "id": "doc-123",
    "version": null
  },
  "id": "req-001"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "document": {
      "id": "doc-123",
      "title": "Example Document",
      "content": "Document content here..."
    },
    "metadata": {
      "created_at": "2026-02-07T16:00:00Z",
      "modified_at": "2026-02-07T16:30:00Z",
      "author": "user@example.com"
    }
  },
  "id": "req-001"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn get_document(
    id: String,
    version: Option<String>,
    state: State<'_, AppState>,
) -> Result<GetDocumentResponse, IpcError> {
    // Validate document ID format
    let document_id = DocumentId::parse(id)
        .map_err(|_| IpcError::invalid_params("id", "Invalid document ID format"))?;

    // Retrieve document from storage
    let document = state.core.get_document(&document_id, version.as_deref())
        .await
        .map_err(IpcError::internal_error)?;

    Ok(GetDocumentResponse {
        document,
        metadata: document.metadata().clone(),
    })
}
```

### 4.3. list_documents

Lists all documents with optional filtering and sorting.

**Method:** `list_documents`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `filters` | Object/Null | No | Document filters (content_type, author, tags, dates) |
| `sort` | Object/Null | No | Sort options (field, direction) |
| `pagination` | Object/Null | No | Pagination (page, page_size) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `documents` | Array | Array of document objects |
| `total_count` | Integer | Total number of documents matching filters |
| `page` | Integer | Current page number |
| `page_size` | Integer | Number of documents per page |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Filter or sort parameters are invalid |
| -32603 | Internal error | Document listing failed |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "list_documents",
  "params": {
    "filters": {
      "content_type": "markdown",
      "author": "user@example.com"
    },
    "sort": {
      "field": "modified_at",
      "direction": "desc"
    },
    "pagination": {
      "page": 1,
      "page_size": 20
    }
  },
  "id": "req-002"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "documents": [
      {
        "id": "doc-001",
        "title": "Document 1",
        "metadata": { "created_at": "2026-02-07T10:00:00Z" }
      },
      {
        "id": "doc-002",
        "title": "Document 2",
        "metadata": { "created_at": "2026-02-07T11:00:00Z" }
      }
    ],
    "total_count": 42,
    "page": 1,
    "page_size": 20
  },
  "id": "req-002"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn list_documents(
    filters: Option<DocumentFilters>,
    sort: Option<SortOptions>,
    pagination: Option<PaginationOptions>,
    state: State<'_, AppState>,
) -> Result<ListDocumentsResponse, IpcError> {
    // Validate and parse filters
    let filters = filters.unwrap_or_default();

    // Validate and parse sort options
    let sort = sort.unwrap_or_else(|| SortOptions::default());

    // Validate and parse pagination
    let pagination = pagination.unwrap_or_else(|| PaginationOptions::default());

    // Query documents with filters, sort, and pagination
    let result = state.core.list_documents(filters, sort, pagination)
        .await
        .map_err(IpcError::internal_error)?;

    Ok(ListDocumentsResponse {
        documents: result.documents,
        total_count: result.total_count,
        page: pagination.page,
        page_size: pagination.page_size,
    })
}
```

### 4.4. create_document

Creates a new document with specified metadata.

**Method:** `create_document`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `metadata` | Object | Yes | Document metadata (title, content_type, tags) |
| `content` | String | No | Initial document content (empty string if omitted) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `document` | Object | Created document object |
| `document_id` | String | Unique identifier of created document |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Metadata is invalid or missing required fields |
| -32603 | Internal error | Document creation failed |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "create_document",
  "params": {
    "metadata": {
      "title": "New Document",
      "content_type": "markdown",
      "tags": ["important", "draft"]
    },
    "content": "# New Document\n\nInitial content here..."
  },
  "id": "req-003"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "document": {
      "id": "doc-456",
      "title": "New Document",
      "content": "# New Document\n\nInitial content here..."
    },
    "document_id": "doc-456"
  },
  "id": "req-003"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn create_document(
    metadata: DocumentMetadata,
    content: Option<String>,
    state: State<'_, AppState>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Validate metadata
    metadata.validate()
        .map_err(|e| IpcError::invalid_params("metadata", &e.to_string()))?;

    // Create document with provided content or empty string
    let content = content.unwrap_or_default();
    let document = state.core.create_document(metadata, content)
        .await
        .map_err(IpcError::internal_error)?;

    Ok(CreateDocumentResponse {
        document: document.clone(),
        document_id: document.id().to_string(),
    })
}
```

### 4.5. update_document

Updates an existing document's content and/or metadata.

**Method:** `update_document`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `id` | String | Yes | Unique document identifier |
| `content` | String/Null | No | New document content (null to keep existing) |
| `metadata` | Object/Null | No | New metadata (null to keep existing) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `document` | Object | Updated document object |
| `version` | String | New version identifier |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Document ID or update parameters are invalid |
| -32603 | Internal error | Document update failed |
| 404 | Document not found | Document with specified ID does not exist |
| 409 | Conflict | Document has been modified by another user |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "update_document",
  "params": {
    "id": "doc-123",
    "content": "# Updated Document\n\nNew content here...",
    "metadata": {
      "title": "Updated Title",
      "tags": ["important", "published"]
    }
  },
  "id": "req-004"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "document": {
      "id": "doc-123",
      "title": "Updated Title",
      "content": "# Updated Document\n\nNew content here..."
    },
    "version": "v2"
  },
  "id": "req-004"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn update_document(
    id: String,
    content: Option<String>,
    metadata: Option<DocumentMetadata>,
    state: State<'_, AppState>,
) -> Result<UpdateDocumentResponse, IpcError> {
    // Validate document ID
    let document_id = DocumentId::parse(id)
        .map_err(|_| IpcError::invalid_params("id", "Invalid document ID format"))?;

    // Validate metadata if provided
    if let Some(ref metadata) = metadata {
        metadata.validate()
            .map_err(|e| IpcError::invalid_params("metadata", &e.to_string()))?;
    }

    // Update document
    let result = state.core.update_document(document_id, content, metadata)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Document not found"),
            ErrorKind::Conflict => IpcError::conflict("Document has been modified"),
            _ => IpcError::internal_error(),
        })?;

    Ok(UpdateDocumentResponse {
        document: result.document,
        version: result.version,
    })
}
```

### 4.6. delete_document

Deletes a document by its unique identifier.

**Method:** `delete_document`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `id` | String | Yes | Unique document identifier |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `deleted` | Boolean | True if document was successfully deleted |
| `document_id` | String | Identifier of deleted document |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Document ID format is invalid |
| -32603 | Internal error | Document deletion failed |
| 404 | Document not found | Document with specified ID does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "delete_document",
  "params": {
    "id": "doc-123"
  },
  "id": "req-005"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "deleted": true,
    "document_id": "doc-123"
  },
  "id": "req-005"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn delete_document(
    id: String,
    state: State<'_, AppState>,
) -> Result<DeleteDocumentResponse, IpcError> {
    // Validate document ID
    let document_id = DocumentId::parse(id)
        .map_err(|_| IpcError::invalid_params("id", "Invalid document ID format"))?;

    // Delete document
    state.core.delete_document(&document_id)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Document not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(DeleteDocumentResponse {
        deleted: true,
        document_id: document_id.to_string(),
    })
}
```

### 4.7. search_documents

Performs a full-text search across all documents with optional filtering.

**Method:** `search_documents`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `query` | String | Yes | Search query string |
| `filters` | Object/Null | No | Additional search filters |
| `limit` | Integer/Null | No | Maximum number of results (default: 100) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `results` | Array | Array of search result objects |
| `total_count` | Integer | Total number of matching documents |
| `query_time_ms` | Float | Time taken to execute query in milliseconds |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Query string is empty or invalid |
| -32603 | Internal error | Search execution failed |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "search_documents",
  "params": {
    "query": "important document",
    "filters": {
      "content_type": "markdown",
      "date_from": "2026-01-01T00:00:00Z"
    },
    "limit": 50
  },
  "id": "req-006"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "results": [
      {
        "document_id": "doc-123",
        "title": "Important Document",
        "snippet": "...important document...",
        "score": 0.95,
        "metadata": { "created_at": "2026-02-07T10:00:00Z" }
      }
    ],
    "total_count": 15,
    "query_time_ms": 12.5
  },
  "id": "req-006"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn search_documents(
    query: String,
    filters: Option<SearchFilters>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<SearchDocumentsResponse, IpcError> {
    // Validate query
    if query.trim().is_empty() {
        return Err(IpcError::invalid_params("query", "Query cannot be empty"));
    }

    // Set default limit
    let limit = limit.unwrap_or(100);

    // Execute search
    let start = Instant::now();
    let results = state.core.search_documents(&query, filters, limit)
        .await
        .map_err(IpcError::internal_error)?;
    let query_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(SearchDocumentsResponse {
        results,
        total_count: results.len(),
        query_time_ms,
    })
}
```

---

## 5. GIT COMMANDS

### 5.1. Command Overview

Git commands provide comprehensive Git repository management operations for version control integration. These commands enable the frontend to interact with Git repositories, including status queries, commits, branch management, and remote operations.

**Related Requirements:**
- [REQ-IPC-017](../.adrs/ - Git Commands
- [REQ-FR-005](../.adrs/ - Repository Management

**Related Design Elements:**
- [DES-IPC-002](../.adrs/ - RepositoryCommand
- [DES-DM-002](../.adrs/ - RepositoryPath
- [DES-DM-005](../.adrs/ - Repository
- [DES-DM-008](../.adrs/ - GitStatus

### 5.2. git_status

Retrieves the current Git status of the repository.

**Method:** `git_status`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `repository_path` | String | Yes | Path to the Git repository |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `branch` | String | Current branch name |
| `head_commit` | String | HEAD commit hash |
| `modified_files` | Array | List of modified files |
| `untracked_files` | Array | List of untracked files |
| `staged_files` | Array | List of staged files |
| `ahead` | Integer | Number of commits ahead of remote |
| `behind` | Integer | Number of commits behind remote |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Repository path is invalid |
| -32603 | Internal error | Git status query failed |
| 404 | Not a Git repository | Path is not a Git repository |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "git_status",
  "params": {
    "repository_path": "/path/to/repository"
  },
  "id": "req-007"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "branch": "main",
    "head_commit": "abc123def456",
    "modified_files": ["document.md"],
    "untracked_files": [],
    "staged_files": [],
    "ahead": 2,
    "behind": 0
  },
  "id": "req-007"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn git_status(
    repository_path: String,
    state: State<'_, AppState>,
) -> Result<GitStatusResponse, IpcError> {
    // Validate repository path
    let repo_path = PathBuf::from(&repository_path);
    if !repo_path.exists() {
        return Err(IpcError::invalid_params("repository_path", "Path does not exist"));
    }

    // Query Git status
    let status = state.core.git_status(&repo_path)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Not a Git repository"),
            _ => IpcError::internal_error(),
        })?;

    Ok(GitStatusResponse {
        branch: status.branch,
        head_commit: status.head_commit,
        modified_files: status.modified_files,
        untracked_files: status.untracked_files,
        staged_files: status.staged_files,
        ahead: status.ahead,
        behind: status.behind,
    })
}
```

### 5.3. git_commit

Creates a new Git commit with staged changes.

**Method:** `git_commit`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `repository_path` | String | Yes | Path to the Git repository |
| `message` | String | Yes | Commit message |
| `author_name` | String/Null | No | Author name (null for default) |
| `author_email` | String/Null | No | Author email (null for default) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `commit_hash` | String | Hash of the created commit |
| `branch` | String | Branch where commit was made |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Repository path or commit message is invalid |
| -32603 | Internal error | Git commit failed |
| 404 | Not a Git repository | Path is not a Git repository |
| 409 | No changes to commit | No staged changes available |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "git_commit",
  "params": {
    "repository_path": "/path/to/repository",
    "message": "Update document content",
    "author_name": null,
    "author_email": null
  },
  "id": "req-008"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "commit_hash": "def456ghi789",
    "branch": "main"
  },
  "id": "req-008"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn git_commit(
    repository_path: String,
    message: String,
    author_name: Option<String>,
    author_email: Option<String>,
    state: State<'_, AppState>,
) -> Result<GitCommitResponse, IpcError> {
    // Validate repository path
    let repo_path = PathBuf::from(&repository_path);
    if !repo_path.exists() {
        return Err(IpcError::invalid_params("repository_path", "Path does not exist"));
    }

    // Validate commit message
    if message.trim().is_empty() {
        return Err(IpcError::invalid_params("message", "Commit message cannot be empty"));
    }

    // Create commit
    let result = state.core.git_commit(&repo_path, &message, author_name, author_email)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Not a Git repository"),
            ErrorKind::InvalidData => IpcError::conflict("No changes to commit"),
            _ => IpcError::internal_error(),
        })?;

    Ok(GitCommitResponse {
        commit_hash: result.commit_hash,
        branch: result.branch,
    })
}
```

### 5.4. git_branch_list

Lists all branches in the repository.

**Method:** `git_branch_list`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `repository_path` | String | Yes | Path to the Git repository |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `branches` | Array | List of branch objects |
| `current_branch` | String | Currently checked out branch |

**Branch Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Branch name |
| `is_head` | Boolean | True if this is the current branch |
| `is_remote` | Boolean | True if this is a remote branch |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Repository path is invalid |
| -32603 | Internal error | Git branch listing failed |
| 404 | Not a Git repository | Path is not a Git repository |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "git_branch_list",
  "params": {
    "repository_path": "/path/to/repository"
  },
  "id": "req-009"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "branches": [
      {
        "name": "main",
        "is_head": true,
        "is_remote": false
      },
      {
        "name": "develop",
        "is_head": false,
        "is_remote": false
      },
      {
        "name": "origin/main",
        "is_head": false,
        "is_remote": true
      }
    ],
    "current_branch": "main"
  },
  "id": "req-009"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn git_branch_list(
    repository_path: String,
    state: State<'_, AppState>,
) -> Result<GitBranchListResponse, IpcError> {
    // Validate repository path
    let repo_path = PathBuf::from(&repository_path);
    if !repo_path.exists() {
        return Err(IpcError::invalid_params("repository_path", "Path does not exist"));
    }

    // List branches
    let result = state.core.git_branch_list(&repo_path)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Not a Git repository"),
            _ => IpcError::internal_error(),
        })?;

    Ok(GitBranchListResponse {
        branches: result.branches,
        current_branch: result.current_branch,
    })
}
```

### 5.5. git_branch_create

Creates a new branch in the repository.

**Method:** `git_branch_create`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `repository_path` | String | Yes | Path to the Git repository |
| `branch_name` | String | Yes | Name for the new branch |
| `start_point` | String/Null | No | Starting point (commit hash or branch name) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `branch_name` | String | Name of the created branch |
| `start_point` | String | Starting point used |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Repository path or branch name is invalid |
| -32603 | Internal error | Git branch creation failed |
| 404 | Not a Git repository | Path is not a Git repository |
| 409 | Branch already exists | Branch with specified name already exists |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "git_branch_create",
  "params": {
    "repository_path": "/path/to/repository",
    "branch_name": "feature/new-feature",
    "start_point": null
  },
  "id": "req-010"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "branch_name": "feature/new-feature",
    "start_point": "HEAD"
  },
  "id": "req-010"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn git_branch_create(
    repository_path: String,
    branch_name: String,
    start_point: Option<String>,
    state: State<'_, AppState>,
) -> Result<GitBranchCreateResponse, IpcError> {
    // Validate repository path
    let repo_path = PathBuf::from(&repository_path);
    if !repo_path.exists() {
        return Err(IpcError::invalid_params("repository_path", "Path does not exist"));
    }

    // Validate branch name
    if branch_name.trim().is_empty() {
        return Err(IpcError::invalid_params("branch_name", "Branch name cannot be empty"));
    }

    // Create branch
    let start_point = start_point.unwrap_or_else(|| "HEAD".to_string());
    let result = state.core.git_branch_create(&repo_path, &branch_name, &start_point)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Not a Git repository"),
            ErrorKind::InvalidData => IpcError::conflict("Branch already exists"),
            _ => IpcError::internal_error(),
        })?;

    Ok(GitBranchCreateResponse {
        branch_name: result.branch_name,
        start_point: result.start_point,
    })
}
```

### 5.6. git_push

Pushes local commits to a remote repository.

**Method:** `git_push`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `repository_path` | String | Yes | Path to the Git repository |
| `remote` | String | Yes | Remote name (e.g., "origin") |
| `branch` | String/Null | No | Branch to push (null for current branch) |
| `force` | Boolean | No | Force push (default: false) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `remote` | String | Remote name |
| `branch` | String | Branch that was pushed |
| `commits_pushed` | Integer | Number of commits pushed |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Repository path or remote name is invalid |
| -32603 | Internal error | Git push failed |
| 404 | Not a Git repository | Path is not a Git repository |
| 404 | Remote not found | Specified remote does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "git_push",
  "params": {
    "repository_path": "/path/to/repository",
    "remote": "origin",
    "branch": null,
    "force": false
  },
  "id": "req-011"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "remote": "origin",
    "branch": "main",
    "commits_pushed": 2
  },
  "id": "req-011"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn git_push(
    repository_path: String,
    remote: String,
    branch: Option<String>,
    force: bool,
    state: State<'_, AppState>,
) -> Result<GitPushResponse, IpcError> {
    // Validate repository path
    let repo_path = PathBuf::from(&repository_path);
    if !repo_path.exists() {
        return Err(IpcError::invalid_params("repository_path", "Path does not exist"));
    }

    // Validate remote name
    if remote.trim().is_empty() {
        return Err(IpcError::invalid_params("remote", "Remote name cannot be empty"));
    }

    // Push to remote
    let result = state.core.git_push(&repo_path, &remote, branch, force)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Not a Git repository or remote not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(GitPushResponse {
        remote: result.remote,
        branch: result.branch,
        commits_pushed: result.commits_pushed,
    })
}
```

### 5.7. git_pull

Pulls changes from a remote repository.

**Method:** `git_pull`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `repository_path` | String | Yes | Path to the Git repository |
| `remote` | String | Yes | Remote name (e.g., "origin") |
| `branch` | String/Null | No | Branch to pull (null for current branch) |
| `rebase` | Boolean | No | Rebase instead of merge (default: false) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `remote` | String | Remote name |
| `branch` | String | Branch that was pulled |
| `commits_pulled` | Integer | Number of commits pulled |
| `files_changed` | Integer | Number of files changed |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Repository path or remote name is invalid |
| -32603 | Internal error | Git pull failed |
| 404 | Not a Git repository | Path is not a Git repository |
| 409 | Merge conflict | Pull resulted in merge conflicts |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "git_pull",
  "params": {
    "repository_path": "/path/to/repository",
    "remote": "origin",
    "branch": null,
    "rebase": false
  },
  "id": "req-012"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "remote": "origin",
    "branch": "main",
    "commits_pulled": 3,
    "files_changed": 5
  },
  "id": "req-012"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn git_pull(
    repository_path: String,
    remote: String,
    branch: Option<String>,
    rebase: bool,
    state: State<'_, AppState>,
) -> Result<GitPullResponse, IpcError> {
    // Validate repository path
    let repo_path = PathBuf::from(&repository_path);
    if !repo_path.exists() {
        return Err(IpcError::invalid_params("repository_path", "Path does not exist"));
    }

    // Validate remote name
    if remote.trim().is_empty() {
        return Err(IpcError::invalid_params("remote", "Remote name cannot be empty"));
    }

    // Pull from remote
    let result = state.core.git_pull(&repo_path, &remote, branch, rebase)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Not a Git repository or remote not found"),
            ErrorKind::InvalidData => IpcError::conflict("Merge conflict occurred"),
            _ => IpcError::internal_error(),
        })?;

    Ok(GitPullResponse {
        remote: result.remote,
        branch: result.branch,
        commits_pulled: result.commits_pulled,
        files_changed: result.files_changed,
    })
}
```

---

## 6. WINDOW COMMANDS

### 6.1. Command Overview

Window commands provide comprehensive window management operations for the Tauri desktop application. These commands enable the frontend to control window behavior, including creation, positioning, sizing, and state management.

**Related Requirements:**
- [REQ-IPC-019](../.adrs/ - File Dialog Commands
- [REQ-DESK-066](../.adrs/ - Window Management

**Related Design Elements:**
- [DES-IPC-004](../.adrs/ - SystemCommand
- [DES-DD-006](../.adrs/ - SystemCommands

### 6.2. window_create

Creates a new window with specified configuration.

**Method:** `window_create`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `label` | String | Yes | Unique window identifier |
| `title` | String | Yes | Window title |
| `url` | String | Yes | URL to load in the window |
| `width` | Integer/Null | No | Window width (null for default) |
| `height` | Integer/Null | No | Window height (null for default) |
| `resizable` | Boolean/Null | No | Whether window is resizable (null for default) |
| `decorations` | Boolean/Null | No | Whether window has decorations (null for default) |
| `always_on_top` | Boolean/Null | No | Whether window is always on top (null for default) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `window_id` | String | Unique identifier of the created window |
| `label` | String | Window label |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Window configuration is invalid |
| -32603 | Internal error | Window creation failed |
| 409 | Window already exists | Window with specified label already exists |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "window_create",
  "params": {
    "label": "settings",
    "title": "Settings",
    "url": "/settings",
    "width": 800,
    "height": 600,
    "resizable": true,
    "decorations": true,
    "always_on_top": false
  },
  "id": "req-013"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "window_id": "win-001",
    "label": "settings"
  },
  "id": "req-013"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn window_create(
    label: String,
    title: String,
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    resizable: Option<bool>,
    decorations: Option<bool>,
    always_on_top: Option<bool>,
    app: AppHandle,
) -> Result<WindowCreateResponse, IpcError> {
    // Validate window label
    if label.trim().is_empty() {
        return Err(IpcError::invalid_params("label", "Window label cannot be empty"));
    }

    // Validate window title
    if title.trim().is_empty() {
        return Err(IpcError::invalid_params("title", "Window title cannot be empty"));
    }

    // Create window
    let window = tauri::WindowBuilder::new(&app, &label, url)
        .title(&title)
        .inner_size(width.unwrap_or(800), height.unwrap_or(600))
        .resizable(resizable.unwrap_or(true))
        .decorations(decorations.unwrap_or(true))
        .always_on_top(always_on_top.unwrap_or(false))
        .build()
        .map_err(IpcError::internal_error)?;

    Ok(WindowCreateResponse {
        window_id: window.label().to_string(),
        label,
    })
}
```

### 6.3. window_close

Closes an existing window.

**Method:** `window_close`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `label` | String | Yes | Window label to close |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `closed` | Boolean | True if window was successfully closed |
| `label` | String | Label of the closed window |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Window label is invalid |
| -32603 | Internal error | Window close failed |
| 404 | Window not found | Window with specified label does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "window_close",
  "params": {
    "label": "settings"
  },
  "id": "req-014"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "closed": true,
    "label": "settings"
  },
  "id": "req-014"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn window_close(
    label: String,
    app: AppHandle,
) -> Result<WindowCloseResponse, IpcError> {
    // Validate window label
    if label.trim().is_empty() {
        return Err(IpcError::invalid_params("label", "Window label cannot be empty"));
    }

    // Get window
    let window = app.get_window(&label)
        .ok_or_else(|| IpcError::not_found("Window not found"))?;

    // Close window
    window.close()
        .map_err(IpcError::internal_error)?;

    Ok(WindowCloseResponse {
        closed: true,
        label,
    })
}
```

### 6.4. window_show

Shows a hidden window.

**Method:** `window_show`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `label` | String | Yes | Window label to show |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `visible` | Boolean | True if window is now visible |
| `label` | String | Label of the window |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Window label is invalid |
| -32603 | Internal error | Window show failed |
| 404 | Window not found | Window with specified label does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "window_show",
  "params": {
    "label": "settings"
  },
  "id": "req-015"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "visible": true,
    "label": "settings"
  },
  "id": "req-015"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn window_show(
    label: String,
    app: AppHandle,
) -> Result<WindowShowResponse, IpcError> {
    // Validate window label
    if label.trim().is_empty() {
        return Err(IpcError::invalid_params("label", "Window label cannot be empty"));
    }

    // Get window
    let window = app.get_window(&label)
        .ok_or_else(|| IpcError::not_found("Window not found"))?;

    // Show window
    window.show()
        .map_err(IpcError::internal_error)?;

    Ok(WindowShowResponse {
        visible: true,
        label,
    })
}
```

### 6.5. window_hide

Hides a visible window.

**Method:** `window_hide`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `label` | String | Yes | Window label to hide |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `visible` | Boolean | True if window is now hidden |
| `label` | String | Label of the window |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Window label is invalid |
| -32603 | Internal error | Window hide failed |
| 404 | Window not found | Window with specified label does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "window_hide",
  "params": {
    "label": "settings"
  },
  "id": "req-016"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "visible": false,
    "label": "settings"
  },
  "id": "req-016"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn window_hide(
    label: String,
    app: AppHandle,
) -> Result<WindowHideResponse, IpcError> {
    // Validate window label
    if label.trim().is_empty() {
        return Err(IpcError::invalid_params("label", "Window label cannot be empty"));
    }

    // Get window
    let window = app.get_window(&label)
        .ok_or_else(|| IpcError::not_found("Window not found"))?;

    // Hide window
    window.hide()
        .map_err(IpcError::internal_error)?;

    Ok(WindowHideResponse {
        visible: false,
        label,
    })
}
```

### 6.6. window_set_position

Sets the position of a window.

**Method:** `window_set_position`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `label` | String | Yes | Window label |
| `x` | Integer | Yes | X coordinate |
| `y` | Integer | Yes | Y coordinate |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `x` | Integer | New X coordinate |
| `y` | Integer | New Y coordinate |
| `label` | String | Label of the window |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Window label or coordinates are invalid |
| -32603 | Internal error | Window position setting failed |
| 404 | Window not found | Window with specified label does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "window_set_position",
  "params": {
    "label": "main",
    "x": 100,
    "y": 100
  },
  "id": "req-017"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "x": 100,
    "y": 100,
    "label": "main"
  },
  "id": "req-017"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn window_set_position(
    label: String,
    x: i32,
    y: i32,
    app: AppHandle,
) -> Result<WindowSetPositionResponse, IpcError> {
    // Validate window label
    if label.trim().is_empty() {
        return Err(IpcError::invalid_params("label", "Window label cannot be empty"));
    }

    // Validate coordinates
    if x < 0 || y < 0 {
        return Err(IpcError::invalid_params("coordinates", "Coordinates must be non-negative"));
    }

    // Get window
    let window = app.get_window(&label)
        .ok_or_else(|| IpcError::not_found("Window not found"))?;

    // Set window position
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(IpcError::internal_error)?;

    Ok(WindowSetPositionResponse {
        x,
        y,
        label,
    })
}
```

### 6.7. window_set_size

Sets the size of a window.

**Method:** `window_set_size`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `label` | String | Yes | Window label |
| `width` | Integer | Yes | Window width |
| `height` | Integer | Yes | Window height |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `width` | Integer | New window width |
| `height` | Integer | New window height |
| `label` | String | Label of the window |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Window label or dimensions are invalid |
| -32603 | Internal error | Window size setting failed |
| 404 | Window not found | Window with specified label does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "window_set_size",
  "params": {
    "label": "main",
    "width": 1024,
    "height": 768
  },
  "id": "req-018"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "width": 1024,
    "height": 768,
    "label": "main"
  },
  "id": "req-018"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn window_set_size(
    label: String,
    width: u32,
    height: u32,
    app: AppHandle,
) -> Result<WindowSetSizeResponse, IpcError> {
    // Validate window label
    if label.trim().is_empty() {
        return Err(IpcError::invalid_params("label", "Window label cannot be empty"));
    }

    // Validate dimensions
    if width < 100 || height < 100 {
        return Err(IpcError::invalid_params("dimensions", "Dimensions must be at least 100x100"));
    }

    // Get window
    let window = app.get_window(&label)
        .ok_or_else(|| IpcError::not_found("Window not found"))?;

    // Set window size
    window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
        .map_err(IpcError::internal_error)?;

    Ok(WindowSetSizeResponse {
        width,
        height,
        label,
    })
}
```

---

## 7. DIALOG COMMANDS

### 7.1. Command Overview

Dialog commands provide native OS dialog operations for file selection, directory browsing, and user prompts. These commands enable the frontend to interact with the host operating system's native dialogs for a consistent user experience.

**Related Requirements:**
- [REQ-IPC-019](../.adrs/ - File Dialog Commands
- [REQ-DESK-066](../.adrs/ - Native Dialogs

**Related Design Elements:**
- [DES-IPC-004](../.adrs/ - SystemCommand
- [DES-DD-006](../.adrs/ - SystemCommands

### 7.2. dialog_open_file

Opens a native file open dialog for selecting one or more files.

**Method:** `dialog_open_file`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `title` | String/Null | No | Dialog title (null for default) |
| `default_path` | String/Null | No | Initial directory (null for default) |
| `filters` | Array/Null | No | File type filters |
| `multiple` | Boolean/Null | No | Allow multiple file selection (null for false) |

**Filter Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Display name for the filter |
| `extensions` | Array | List of file extensions (e.g., ["md", "txt"]) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `selected` | Array | Array of selected file paths |
| `cancelled` | Boolean | True if user cancelled the dialog |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Dialog parameters are invalid |
| -32603 | Internal error | Dialog failed to open |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "dialog_open_file",
  "params": {
    "title": "Select Document",
    "default_path": "/home/user/documents",
    "filters": [
      {
        "name": "Markdown Files",
        "extensions": ["md", "markdown"]
      },
      {
        "name": "Text Files",
        "extensions": ["txt"]
      }
    ],
    "multiple": false
  },
  "id": "req-019"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "selected": ["/home/user/document.md"],
    "cancelled": false
  },
  "id": "req-019"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn dialog_open_file(
    title: Option<String>,
    default_path: Option<String>,
    filters: Option<Vec<FileFilter>>,
    multiple: Option<bool>,
    app: AppHandle,
) -> Result<DialogOpenFileResponse, IpcError> {
    use tauri::dialog::FileDialogBuilder;

    // Build dialog
    let mut dialog = FileDialogBuilder::new();

    if let Some(t) = title {
        dialog = dialog.title(&t);
    }

    if let Some(path) = default_path {
        dialog = dialog.set_directory(&path);
    }

    if let Some(filters) = filters {
        for filter in filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &extensions);
        }
    }

    // Open dialog
    let result = if multiple.unwrap_or(false) {
        dialog.pick_files(&app).await
    } else {
        dialog.pick_file(&app).await
    };

    match result {
        Ok(Some(paths)) => Ok(DialogOpenFileResponse {
            selected: paths,
            cancelled: false,
        }),
        Ok(None) => Ok(DialogOpenFileResponse {
            selected: vec![],
            cancelled: true,
        }),
        Err(e) => Err(IpcError::internal_error()),
    }
}
```

### 7.3. dialog_save_file

Opens a native file save dialog for selecting a file save location.

**Method:** `dialog_save_file`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `title` | String/Null | No | Dialog title (null for default) |
| `default_path` | String/Null | No | Initial directory (null for default) |
| `default_name` | String/Null | No | Default file name (null for none) |
| `filters` | Array/Null | No | File type filters |

**Filter Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Display name for the filter |
| `extensions` | Array | List of file extensions (e.g., ["md", "txt"]) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `selected` | String/Null | Selected file path (null if cancelled) |
| `cancelled` | Boolean | True if user cancelled the dialog |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Dialog parameters are invalid |
| -32603 | Internal error | Dialog failed to open |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "dialog_save_file",
  "params": {
    "title": "Save Document",
    "default_path": "/home/user/documents",
    "default_name": "document.md",
    "filters": [
      {
        "name": "Markdown Files",
        "extensions": ["md", "markdown"]
      }
    ]
  },
  "id": "req-020"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "selected": "/home/user/document.md",
    "cancelled": false
  },
  "id": "req-020"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn dialog_save_file(
    title: Option<String>,
    default_path: Option<String>,
    default_name: Option<String>,
    filters: Option<Vec<FileFilter>>,
    app: AppHandle,
) -> Result<DialogSaveFileResponse, IpcError> {
    use tauri::dialog::FileDialogBuilder;

    // Build dialog
    let mut dialog = FileDialogBuilder::new();

    if let Some(t) = title {
        dialog = dialog.title(&t);
    }

    if let Some(path) = default_path {
        dialog = dialog.set_directory(&path);
    }

    if let Some(name) = default_name {
        dialog = dialog.set_file_name(&name);
    }

    if let Some(filters) = filters {
        for filter in filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &extensions);
        }
    }

    // Open dialog
    let result = dialog.save_file(&app).await;

    match result {
        Ok(Some(path)) => Ok(DialogSaveFileResponse {
            selected: Some(path),
            cancelled: false,
        }),
        Ok(None) => Ok(DialogSaveFileResponse {
            selected: None,
            cancelled: true,
        }),
        Err(e) => Err(IpcError::internal_error()),
    }
}
```

### 7.4. dialog_open_folder

Opens a native folder selection dialog.

**Method:** `dialog_open_folder`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `title` | String/Null | No | Dialog title (null for default) |
| `default_path` | String/Null | No | Initial directory (null for default) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `selected` | String/Null | Selected folder path (null if cancelled) |
| `cancelled` | Boolean | True if user cancelled the dialog |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Dialog parameters are invalid |
| -32603 | Internal error | Dialog failed to open |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "dialog_open_folder",
  "params": {
    "title": "Select Repository Folder",
    "default_path": "/home/user"
  },
  "id": "req-021"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "selected": "/home/user/repository",
    "cancelled": false
  },
  "id": "req-021"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn dialog_open_folder(
    title: Option<String>,
    default_path: Option<String>,
    app: AppHandle,
) -> Result<DialogOpenFolderResponse, IpcError> {
    use tauri::dialog::FileDialogBuilder;

    // Build dialog
    let mut dialog = FileDialogBuilder::new();

    if let Some(t) = title {
        dialog = dialog.title(&t);
    }

    if let Some(path) = default_path {
        dialog = dialog.set_directory(&path);
    }

    // Open dialog
    let result = dialog.pick_folder(&app).await;

    match result {
        Ok(Some(path)) => Ok(DialogOpenFolderResponse {
            selected: Some(path),
            cancelled: false,
        }),
        Ok(None) => Ok(DialogOpenFolderResponse {
            selected: None,
            cancelled: true,
        }),
        Err(e) => Err(IpcError::internal_error()),
    }
}
```

### 7.5. dialog_message

Opens a native message dialog with customizable buttons.

**Method:** `dialog_message`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `title` | String | Yes | Dialog title |
| `message` | String | Yes | Message to display |
| `kind` | String | No | Dialog kind: "info", "warning", "error" (default: "info") |
| `buttons` | Array/Null | No | Custom buttons (null for default OK button) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `button` | String/Null | Button that was clicked (null if dialog was closed) |
| `cancelled` | Boolean | True if user cancelled the dialog |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Dialog parameters are invalid |
| -32603 | Internal error | Dialog failed to open |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "dialog_message",
  "params": {
    "title": "Confirm Delete",
    "message": "Are you sure you want to delete this document?",
    "kind": "warning",
    "buttons": ["Yes", "No", "Cancel"]
  },
  "id": "req-022"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "button": "Yes",
    "cancelled": false
  },
  "id": "req-022"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn dialog_message(
    title: String,
    message: String,
    kind: Option<String>,
    buttons: Option<Vec<String>>,
    app: AppHandle,
) -> Result<DialogMessageResponse, IpcError> {
    use tauri::dialog::MessageDialogBuilder;

    // Parse dialog kind
    let kind = kind.unwrap_or_else(|| "info".to_string());
    let msg_kind = match kind.as_str() {
        "info" => tauri::dialog::MessageDialogKind::Info,
        "warning" => tauri::dialog::MessageDialogKind::Warning,
        "error" => tauri::dialog::MessageDialogKind::Error,
        _ => tauri::dialog::MessageDialogKind::Info,
    };

    // Build dialog
    let mut dialog = MessageDialogBuilder::new(msg_kind, &title, &message);

    if let Some(buttons) = buttons {
        dialog = dialog.buttons(buttons);
    }

    // Open dialog
    let result = dialog.show(&app).await;

    match result {
        Ok(Some(button)) => Ok(DialogMessageResponse {
            button: Some(button),
            cancelled: false,
        }),
        Ok(None) => Ok(DialogMessageResponse {
            button: None,
            cancelled: true,
        }),
        Err(e) => Err(IpcError::internal_error()),
     }
}
```
```

---

## 8. CONFIGURATION COMMANDS

### 8.1. Command Overview

Configuration commands provide application settings management operations. These commands enable the frontend to get, set, and reset application configuration settings.

**Related Requirements:**
- [REQ-IPC-020](../.adrs/ - Settings Commands
- [REQ-FR-001](../.adrs/ - Application Management

**Related Design Elements:**
- [DES-IPC-004](../.adrs/ - SystemCommand
- [DES-DD-006](../.adrs/ - SystemCommands

### 8.2. config_get

Retrieves the value of a configuration setting.

**Method:** `config_get`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `key` | String | Yes | Configuration key |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `value` | Any | Configuration value (can be string, number, boolean, object, or array) |
| `key` | String | Configuration key |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Configuration key is invalid |
| -32603 | Internal error | Configuration retrieval failed |
| 404 | Key not found | Configuration key does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "config_get",
  "params": {
    "key": "theme"
  },
  "id": "req-023"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "value": "dark",
    "key": "theme"
  },
  "id": "req-023"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn config_get(
    key: String,
    state: State<'_, AppState>,
) -> Result<ConfigGetResponse, IpcError> {
    // Validate configuration key
    if key.trim().is_empty() {
        return Err(IpcError::invalid_params("key", "Configuration key cannot be empty"));
    }

    // Get configuration value
    let value = state.config.get(&key)
        .await
        .map_err(IpcError::internal_error)?
        .ok_or_else(|| IpcError::not_found("Configuration key not found"))?;

    Ok(ConfigGetResponse {
        value,
        key,
    })
}
```

### 8.3. config_set

Sets the value of a configuration setting.

**Method:** `config_set`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `key` | String | Yes | Configuration key |
| `value` | Any | Yes | Configuration value (string, number, boolean, object, or array) |
| `persist` | Boolean/Null | No | Whether to persist the setting (null for default) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `key` | String | Configuration key |
| `value` | Any | New configuration value |
| `persisted` | Boolean | True if setting was persisted |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Configuration key or value is invalid |
| -32603 | Internal error | Configuration setting failed |
| 409 | Read-only setting | Configuration key is read-only |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "config_set",
  "params": {
    "key": "theme",
    "value": "light",
    "persist": true
  },
  "id": "req-024"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "key": "theme",
    "value": "light",
    "persisted": true
  },
  "id": "req-024"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn config_set(
    key: String,
    value: JsonValue,
    persist: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ConfigSetResponse, IpcError> {
    // Validate configuration key
    if key.trim().is_empty() {
        return Err(IpcError::invalid_params("key", "Configuration key cannot be empty"));
    }

    // Check if setting is read-only
    if state.config.is_read_only(&key).await {
        return Err(IpcError::conflict("Configuration key is read-only"));
    }

    // Set configuration value
    let persisted = persist.unwrap_or(true);
    state.config.set(&key, value, persisted)
        .await
        .map_err(IpcError::internal_error)?;

    Ok(ConfigSetResponse {
        key,
        value,
        persisted,
    })
}
```

### 8.4. config_reset

Resets a configuration setting to its default value.

**Method:** `config_reset`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `key` | String | Yes | Configuration key |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `key` | String | Configuration key |
| `value` | Any | Reset configuration value |
| `reset` | Boolean | True if setting was reset |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Configuration key is invalid |
| -32603 | Internal error | Configuration reset failed |
| 404 | Key not found | Configuration key does not exist |
| 409 | Read-only setting | Configuration key is read-only |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "config_reset",
  "params": {
    "key": "theme"
  },
  "id": "req-025"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "key": "theme",
    "value": "dark",
    "reset": true
  },
  "id": "req-025"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn config_reset(
    key: String,
    state: State<'_, AppState>,
) -> Result<ConfigResetResponse, IpcError> {
    // Validate configuration key
    if key.trim().is_empty() {
        return Err(IpcError::invalid_params("key", "Configuration key cannot be empty"));
    }

    // Check if setting is read-only
    if state.config.is_read_only(&key).await {
        return Err(IpcError::conflict("Configuration key is read-only"));
    }

    // Reset configuration value
    let value = state.config.reset(&key)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Configuration key not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(ConfigResetResponse {
        key,
        value,
        reset: true,
    })
}
```

### 8.5. config_list

Lists all configuration settings.

**Method:** `config_list`

**Request Parameters:**

None

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `settings` | Object | Object containing all configuration settings |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32603 | Internal error | Configuration listing failed |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "config_list",
  "params": {},
  "id": "req-026"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "settings": {
      "theme": "dark",
      "language": "en",
      "auto_save": true,
      "auto_save_interval": 2000
    }
  },
  "id": "req-026"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn config_list(
    state: State<'_, AppState>,
) -> Result<ConfigListResponse, IpcError> {
    // List all configuration settings
    let settings = state.config.list()
        .await
        .map_err(IpcError::internal_error)?;

    Ok(ConfigListResponse {
        settings,
    })
}
```

---

## 9. PLUGIN COMMANDS

### 9.1. Command Overview

Plugin commands provide plugin management operations for extending Tachyon functionality. These commands enable the frontend to list, install, uninstall, and manage application plugins.

**Related Requirements:**
- [REQ-IPC-018](../.adrs/ - Search Commands
- [REQ-FR-001](../.adrs/ - Application Management

**Related Design Elements:**
- [DES-IPC-004](../.adrs/ - SystemCommand
- [DES-DD-006](../.adrs/ - SystemCommands

### 9.2. plugin_list

Lists all installed plugins.

**Method:** `plugin_list`

**Request Parameters:**

None

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `plugins` | Array | Array of plugin objects |

**Plugin Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique plugin identifier |
| `name` | String | Plugin name |
| `version` | String | Plugin version |
| `enabled` | Boolean | True if plugin is enabled |
| `description` | String/Null | Plugin description |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32603 | Internal error | Plugin listing failed |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "plugin_list",
  "params": {},
  "id": "req-027"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "plugins": [
      {
        "id": "plugin-spellcheck",
        "name": "Spell Check",
        "version": "1.0.0",
        "enabled": true,
        "description": "Spell checking plugin"
      },
      {
        "id": "plugin-wordcount",
        "name": "Word Count",
        "version": "2.1.0",
        "enabled": false,
        "description": "Word counting plugin"
      }
    ]
  },
  "id": "req-027"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn plugin_list(
    state: State<'_, AppState>,
) -> Result<PluginListResponse, IpcError> {
    // List all plugins
    let plugins = state.plugin_manager.list()
        .await
        .map_err(IpcError::internal_error)?;

    Ok(PluginListResponse {
        plugins,
    })
}
```

### 9.3. plugin_install

Installs a plugin from a specified source.

**Method:** `plugin_install`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `source` | String | Yes | Plugin source (URL or local path) |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `plugin_id` | String | Installed plugin identifier |
| `name` | String | Plugin name |
| `version` | String | Plugin version |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Plugin source is invalid |
| -32603 | Internal error | Plugin installation failed |
| 409 | Plugin already installed | Plugin is already installed |
| 404 | Plugin not found | Plugin source not found |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "plugin_install",
  "params": {
    "source": "https://plugins.tachyon.dev/spellcheck.tauri-plugin"
  },
  "id": "req-028"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "plugin_id": "plugin-spellcheck",
    "name": "Spell Check",
    "version": "1.0.0"
  },
  "id": "req-028"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn plugin_install(
    source: String,
    state: State<'_, AppState>,
) -> Result<PluginInstallResponse, IpcError> {
    // Validate plugin source
    if source.trim().is_empty() {
        return Err(IpcError::invalid_params("source", "Plugin source cannot be empty"));
    }

    // Install plugin
    let result = state.plugin_manager.install(&source)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::AlreadyExists => IpcError::conflict("Plugin already installed"),
            ErrorKind::NotFound => IpcError::not_found("Plugin source not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(PluginInstallResponse {
        plugin_id: result.plugin_id,
        name: result.name,
        version: result.version,
    })
}
```

### 9.4. plugin_uninstall

Uninstalls a plugin.

**Method:** `plugin_uninstall`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `plugin_id` | String | Yes | Plugin identifier |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `plugin_id` | String | Uninstalled plugin identifier |
| `uninstalled` | Boolean | True if plugin was uninstalled |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Plugin ID is invalid |
| -32603 | Internal error | Plugin uninstallation failed |
| 404 | Plugin not found | Plugin with specified ID does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "plugin_uninstall",
  "params": {
    "plugin_id": "plugin-spellcheck"
  },
  "id": "req-029"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "plugin_id": "plugin-spellcheck",
    "uninstalled": true
  },
  "id": "req-029"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn plugin_uninstall(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<PluginUninstallResponse, IpcError> {
    // Validate plugin ID
    if plugin_id.trim().is_empty() {
        return Err(IpcError::invalid_params("plugin_id", "Plugin ID cannot be empty"));
    }

    // Uninstall plugin
    state.plugin_manager.uninstall(&plugin_id)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Plugin not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(PluginUninstallResponse {
        plugin_id,
        uninstalled: true,
    })
}
```

### 9.5. plugin_enable

Enables a previously installed plugin.

**Method:** `plugin_enable`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `plugin_id` | String | Yes | Plugin identifier |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `plugin_id` | String | Enabled plugin identifier |
| `enabled` | Boolean | True if plugin is now enabled |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Plugin ID is invalid |
| -32603 | Internal error | Plugin enablement failed |
| 404 | Plugin not found | Plugin with specified ID does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "plugin_enable",
  "params": {
    "plugin_id": "plugin-wordcount"
  },
  "id": "req-030"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "plugin_id": "plugin-wordcount",
    "enabled": true
  },
  "id": "req-030"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn plugin_enable(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<PluginEnableResponse, IpcError> {
    // Validate plugin ID
    if plugin_id.trim().is_empty() {
        return Err(IpcError::invalid_params("plugin_id", "Plugin ID cannot be empty"));
    }

    // Enable plugin
    state.plugin_manager.enable(&plugin_id)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Plugin not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(PluginEnableResponse {
        plugin_id,
        enabled: true,
    })
}
```

### 9.6. plugin_disable

Disables a previously enabled plugin.

**Method:** `plugin_disable`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `plugin_id` | String | Yes | Plugin identifier |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `plugin_id` | String | Disabled plugin identifier |
| `disabled` | Boolean | True if plugin is now disabled |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Plugin ID is invalid |
| -32603 | Internal error | Plugin disablement failed |
| 404 | Plugin not found | Plugin with specified ID does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "plugin_disable",
  "params": {
    "plugin_id": "plugin-wordcount"
  },
  "id": "req-031"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "plugin_id": "plugin-wordcount",
    "disabled": true
  },
  "id": "req-031"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn plugin_disable(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<PluginDisableResponse, IpcError> {
    // Validate plugin ID
    if plugin_id.trim().is_empty() {
        return Err(IpcError::invalid_params("plugin_id", "Plugin ID cannot be empty"));
    }

    // Disable plugin
    state.plugin_manager.disable(&plugin_id)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Plugin not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(PluginDisableResponse {
        plugin_id,
        disabled: true,
    })
}
```

---

## 10. EVENT COMMANDS

### 10.1. Command Overview

Event commands provide event subscription and management operations for the IPC event system. These commands enable the frontend to subscribe to, unsubscribe from, and manage event listeners.

**Related Requirements:**
- [REQ-IPC-021](../.adrs/ - File Change Events
- [REQ-IPC-022](../.adrs/ - Cache Invalidation Events
- [REQ-IPC-023](../.adrs/ - Sync Status Events
- [REQ-IPC-024](../.adrs/ - Error Events
- [REQ-IPC-025](../.adrs/ - Progress Events

**Related Design Elements:**
- [DES-IPC-005](../.adrs/ - DocumentEvent
- [DES-IPC-006](../.adrs/ - RepositoryEvent
- [DES-IPC-007](../.adrs/ - SystemEvent
- [DES-DD-007](../.adrs/ - DocumentEvents

### 10.2. event_subscribe

Subscribes to an event type.

**Method:** `event_subscribe`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `event_type` | String | Yes | Event type to subscribe to |
| `filter` | Object/Null | No | Event filter criteria |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `subscription_id` | String | Unique subscription identifier |
| `event_type` | String | Subscribed event type |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Event type or filter is invalid |
| -32603 | Internal error | Event subscription failed |
| 409 | Already subscribed | Already subscribed to this event type |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "event_subscribe",
  "params": {
    "event_type": "document_changed",
    "filter": {
      "document_id": "doc-123"
    }
  },
  "id": "req-032"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "subscription_id": "sub-001",
    "event_type": "document_changed"
  },
  "id": "req-032"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn event_subscribe(
    event_type: String,
    filter: Option<JsonMap>,
    app: AppHandle,
) -> Result<EventSubscribeResponse, IpcError> {
    // Validate event type
    if event_type.trim().is_empty() {
        return Err(IpcError::invalid_params("event_type", "Event type cannot be empty"));
    }

    // Subscribe to event
    let subscription_id = state.event_manager.subscribe(&event_type, filter)
        .await
        .map_err(IpcError::internal_error)?;

    Ok(EventSubscribeResponse {
        subscription_id,
        event_type,
    })
}
```

### 10.3. event_unsubscribe

Unsubscribes from an event type.

**Method:** `event_unsubscribe`

**Request Parameters:**

| Parameter | Type | Required | Description |
|------------|------|----------|-------------|
| `subscription_id` | String | Yes | Subscription identifier |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `subscription_id` | String | Unsubscribed subscription identifier |
| `unsubscribed` | Boolean | True if subscription was removed |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32602 | Invalid params | Subscription ID is invalid |
| -32603 | Internal error | Event unsubscription failed |
| 404 | Subscription not found | Subscription with specified ID does not exist |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "event_unsubscribe",
  "params": {
    "subscription_id": "sub-001"
  },
  "id": "req-033"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "subscription_id": "sub-001",
    "unsubscribed": true
  },
  "id": "req-033"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn event_unsubscribe(
    subscription_id: String,
    state: State<'_, AppState>,
) -> Result<EventUnsubscribeResponse, IpcError> {
    // Validate subscription ID
    if subscription_id.trim().is_empty() {
        return Err(IpcError::invalid_params("subscription_id", "Subscription ID cannot be empty"));
    }

    // Unsubscribe from event
    state.event_manager.unsubscribe(&subscription_id)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Subscription not found"),
            _ => IpcError::internal_error(),
        })?;

    Ok(EventUnsubscribeResponse {
        subscription_id,
        unsubscribed: true,
    })
}
```

### 10.4. event_list

Lists all active event subscriptions.

**Method:** `event_list`

**Request Parameters:**

None

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| `subscriptions` | Array | Array of subscription objects |

**Subscription Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `subscription_id` | String | Unique subscription identifier |
| `event_type` | String | Subscribed event type |
| `filter` | Object/Null | Event filter criteria |

**Errors:**

| Code | Message | Condition |
|------|---------|-----------|
| -32603 | Internal error | Event listing failed |

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "event_list",
  "params": {},
  "id": "req-034"
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "subscriptions": [
      {
        "subscription_id": "sub-001",
        "event_type": "document_changed",
        "filter": {
          "document_id": "doc-123"
        }
      },
      {
        "subscription_id": "sub-002",
        "event_type": "cache_invalidated",
        "filter": null
      }
    ]
  },
  "id": "req-034"
}
```

**Rust Implementation:**

```rust
#[tauri::command]
pub async fn event_list(
    state: State<'_, AppState>,
) -> Result<EventListResponse, IpcError> {
    // List all subscriptions
    let subscriptions = state.event_manager.list()
        .await
        .map_err(IpcError::internal_error)?;

    Ok(EventListResponse {
        subscriptions,
    })
}
```

---

## 11. ERROR HANDLING

### 11.1. Error Overview

The IPC API uses structured error handling with comprehensive error codes and messages. All errors follow the JSON-RPC 2.0 error specification with Tachyon-specific extensions for detailed error context and recovery guidance.

**Related Requirements:**
- [REQ-IPC-066](../.adrs/ - Error Types
- [REQ-IPC-067](../.adrs/ - Error Context
- [REQ-IPC-068](../.adrs/ - Error Categorization
- [REQ-IPC-069](../.adrs/ - Error Propagation
- [REQ-SEC-081](../.adrs/ - Capability Enforcement

**Related Design Elements:**
- [DES-IPC-008](../.adrs/ - IpcError
- [DES-SEC-001](../.adrs/ - Error Handling

### 11.2. Error Response Format

All error responses follow the JSON-RPC 2.0 error specification format with Tachyon-specific extensions.

**Error Response Structure:**

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "field": "id",
      "reason": "Invalid document ID format"
    }
  },
  "id": "req-id"
}
```

**Error Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `code` | Integer | Error code (see Section 11.3) |
| `message` | String | Human-readable error message |
| `data` | Any/Null | Additional error context |

### 11.3. Error Codes

The IPC API defines the following error codes:

#### JSON-RPC Standard Errors

| Code | Message | Description |
|------|---------|-----------|
| -32700 | Parse error | Invalid JSON was received by the server |
| -32600 | Invalid Request | The JSON sent is not a valid Request object |
| -32601 | Method not found | The method does not exist / is not available |
| -32602 | Invalid params | Invalid method parameter(s) |
| -32603 | Internal error | Internal JSON-RPC error |

#### Tachyon-Specific Errors

| Code | Message | Description |
|------|---------|-----------|
| 400 | Not Found | Requested resource does not exist |
| 401 | Unauthorized | Authentication or authorization failed |
| 403 | Forbidden | Operation is not permitted |
| 404 | Conflict | Resource conflict (e.g., concurrent modification) |
| 409 | Already Exists | Resource already exists |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Unhandled server error |
| 503 | Service Unavailable | Required service is unavailable |

#### Custom Error Codes

| Code | Message | Description |
|------|---------|-----------|
| 1001 | Document Not Found | Document with specified ID does not exist |
| 1002 | Document Creation Failed | Failed to create new document |
| 1003 | Document Update Failed | Failed to update document |
| 1004 | Document Deletion Failed | Failed to delete document |
| 2001 | Repository Not Found | Git repository does not exist |
| 2002 | Git Operation Failed | Git operation failed |
| 2003 | Branch Not Found | Git branch does not exist |
| 2004 | Merge Conflict | Git merge resulted in conflicts |
| 3001 | Window Not Found | Window with specified label does not exist |
| 3002 | Window Operation Failed | Window operation failed |
| 4001 | Configuration Not Found | Configuration key does not exist |
| 4002 | Configuration Read-Only | Configuration key is read-only |
| 4003 | Configuration Update Failed | Failed to update configuration |
| 5001 | Plugin Not Found | Plugin with specified ID does not exist |
| 5002 | Plugin Installation Failed | Failed to install plugin |
| 5003 | Plugin Uninstallation Failed | Failed to uninstall plugin |
| 5004 | Plugin Not Enabled | Plugin is not enabled |
| 6001 | Event Subscription Failed | Failed to subscribe to event |
| 6002 | Event Unsubscription Failed | Failed to unsubscribe from event |

### 11.4. Error Categories

Errors are categorized for better handling and recovery:

| Category | Description | Example Codes |
|----------|-------------|---------------|
| **Validation Errors** | Input validation failures | -32602, 1001, 3001, 4001 |
| **Authorization Errors** | Authentication/authorization failures | 401, 403 |
| **Resource Errors** | Resource-related failures | 400, 404, 409 |
| **Conflict Errors** | Concurrency conflicts | 404, 2004 |
| **Internal Errors** | Server-side failures | -32603, 500, 503 |
| **Rate Limit Errors** | Throttling violations | 429 |

### 11.5. Error Handling Best Practices

**Frontend Error Handling:**

```javascript
// Example frontend error handling
import { invoke } from '@tauri-apps/api/core';

try {
  const response = await invoke('get_document', { id: 'doc-123' });
  // Process response
} catch (error) {
  // Handle error based on code
  switch (error.code) {
    case 404:
      // Conflict - prompt user to refresh
      showConflictDialog();
      break;
    case 401:
      // Unauthorized - redirect to login
      redirectToLogin();
      break;
    default:
      // Other errors - show error message
      showErrorDialog(error.message);
  }
}
```

**Backend Error Handling:**

```rust
// Example backend error handling
use tauri::State;

#[tauri::command]
pub async fn get_document(
    id: String,
    state: State<'_, AppState>,
) -> Result<GetDocumentResponse, IpcError> {
    // Validate input
    let document_id = DocumentId::parse(id)
        .map_err(|e| IpcError::invalid_params("id", e.to_string()))?;

    // Retrieve document
    let document = state.core.get_document(&document_id)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => IpcError::not_found("Document not found"),
            ErrorKind::Conflict => IpcError::conflict("Document has been modified"),
            _ => IpcError::internal_error(),
        })?;

    Ok(GetDocumentResponse {
        document,
        metadata: document.metadata().clone(),
    })
}
```

### 11.6. Error Recovery Strategies

**Retry Logic:**

- Transient errors should be retried with exponential backoff
- Maximum retry count: 3
- Initial backoff: 100ms
- Maximum backoff: 1000ms

**Fallback Mechanisms:**

- Cache stale data for offline access
- Provide degraded functionality when services are unavailable
- Use default values for configuration errors

**User Notification:**

- All errors should be presented to the user with actionable messages
- Error messages should include recovery suggestions where applicable
- Critical errors should trigger user notifications
```

---

## 12. SECURITY CONSIDERATIONS

### 12.1. Security Overview

The IPC API implements comprehensive security controls to protect against unauthorized access, injection attacks, and information disclosure. Security is implemented through multiple layers following the defense-in-depth principle.

**Related Requirements:**
- [REQ-IPC-056](../.adrs/ - Capability Enforcement
- [REQ-IPC-057](../.adrs/ - Input Validation
- [REQ-IPC-058](../.adrs/ - Output Sanitization
- [REQ-IPC-059](../.adrs/ - Path Traversal Prevention
- [REQ-IPC-060](../.adrs/ - Command Authorization
- [REQ-SEC-081](../.adrs/ - Capability Enforcement
- [REQ-SEC-082](../.adrs/ - Message Validation
- [REQ-SEC-083](../.adrs/ - IPC Rate Limiting
- [REQ-SEC-084](../.adrs/ - IPC Logging
- [REQ-SEC-085](../.adrs/ - Desktop Isolation

**Related Design Elements:**
- [DES-IPC-009](../.adrs/ - IpcAuth
- [DES-SEC-001](../.adrs/ - AuthenticationProvider
- [DES-SEC-002](../.adrs/ - AuthorizationMiddleware
- [DES-SEC-003](../.adrs/ - InputValidator
- [DES-SEC-004](../.adrs/ - OutputSanitizer
- [DES-SEC-005](../.adrs/ - SecurityLogger
- [DES-SEC-006](../.adrs/ - RateLimiter

### 12.2. Authentication

The IPC API uses JWT-based session authentication for secure access control.

**Authentication Flow:**

1. **Login:** User provides credentials to obtain JWT token
2. **Token Storage:** Token stored securely in memory (never persisted to disk)
3. **Token Validation:** Token validated on each IPC command
4. **Token Expiration:** Tokens expire after configurable duration (default: 1 hour)
5. **Token Refresh:** Automatic token refresh before expiration

**JWT Token Structure:**

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "tachyon-ipc",
    "user_id": "user-123",
    "exp": 3600,
    "iat": 1700000000
  },
  "signature": "signature-here"
}
```

**Authentication Example:**

```rust
// Backend authentication
use jsonwebtoken::{encode, Algorithm, Header, Validation, EncodingKey};

#[tauri::command]
pub async fn authenticate(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<AuthenticateResponse, IpcError> {
    // Validate credentials
    if username.trim().is_empty() || password.trim().is_empty() {
        return Err(IpcError::invalid_params("credentials", "Username and password cannot be empty"));
    }

    // Authenticate user
    let user_id = state.auth.authenticate(&username, &password)
        .await
        .map_err(IpcError::internal_error)?;

    // Generate JWT token
    let token = state.auth.generate_token(user_id)
        .map_err(IpcError::internal_error)?;

    Ok(AuthenticateResponse {
        token,
        user_id,
        expires_in: 3600,
    })
}
```

### 12.3. Authorization

The IPC API implements capability-based authorization following the principle of least privilege.

**Capability System:**

```json
{
  "identifier": "default",
  "description": "Default capability set",
  "windows": ["main"],
  "permissions": [
    "core:document:read",
    "core:document:write",
    "core:document:delete",
    "core:git:read",
    "core:git:write",
    "dialog:file:open",
    "dialog:file:save",
    "config:read",
    "config:write",
    "event:subscribe",
    "event:unsubscribe"
  ]
}
```

**Authorization Enforcement:**

```rust
// Backend authorization
use tauri::State;

#[tauri::command]
pub async fn get_document(
    id: String,
    auth_token: Option<String>,
    state: State<'_, AppState>,
) -> Result<GetDocumentResponse, IpcError> {
    // Validate authentication token
    let user_id = if let Some(token) = auth_token {
        state.auth.validate_token(&token)
            .map_err(IpcError::unauthorized())?
    } else {
        return Err(IpcError::unauthorized());
    };

    // Check capability permission
    if !state.auth.has_capability(user_id, "core:document:read") {
        return Err(IpcError::forbidden("Insufficient permissions"));
    }

    // Retrieve document
    let document = state.core.get_document(&id)
        .await
        .map_err(IpcError::internal_error)?;

    Ok(GetDocumentResponse {
        document,
        metadata: document.metadata().clone(),
    })
}
```

### 12.4. Input Validation

All IPC inputs are validated against schemas to prevent injection attacks and ensure data integrity.

**Validation Rules:**

1. **Type Validation:** All inputs must match expected types
2. **Range Validation:** Numeric values must be within allowed ranges
3. **Format Validation:** String values must match required formats
4. **Length Validation:** String and array lengths must be within limits
5. **Pattern Validation:** String values must match allowed patterns
6. **Path Validation:** File paths must be canonicalized and validated

**Input Validator Example:**

```rust
// Input validation
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentId(String);

impl DocumentId {
    fn validate(&self) -> Result<(), String> {
        // Validate format: doc-XXX where XXX is alphanumeric
        let re = Regex::new(r"^doc-[a-zA-Z0-9]{1,50}$").unwrap();
        if !re.is_match(&self.0) {
            return Err("Invalid document ID format".to_string());
        }

        // Validate length
        if self.0.len() > 50 {
            return Err("Document ID too long (max 50 characters)".to_string());
        }

        Ok(())
    }
}

impl<'de> serde::de::Deserialize<'de> for DocumentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::validate(&s).map_err(D::Error::custom)?;
        Ok(s)
    }
}
```

### 12.5. Output Sanitization

All IPC outputs are sanitized to prevent information disclosure and injection attacks.

**Sanitization Rules:**

1. **HTML Escaping:** HTML entities are escaped in string outputs
2. **JSON Sanitization:** JSON structure is validated before serialization
3. **Path Redaction:** File paths are redacted when appropriate
4. **Error Message Sanitization:** Error messages are genericized for security

**Output Sanitizer Example:**

```rust
// Output sanitization
use ammonia::clean;

pub fn sanitize_output(output: &str) -> String {
    // Escape HTML entities
    clean(output).to_string()
}
```

### 12.6. Path Traversal Prevention

File system operations protect against path traversal attacks through canonicalization and allow-lists.

**Path Validation:**

1. **Canonicalization:** Paths are resolved to absolute paths
2. **Allow-List Validation:** Paths are validated against allowed directories
3. **Symbolic Link Prevention:** Symbolic links are rejected

**Path Validator Example:**

```rust
// Path traversal prevention
use std::path::{Path, PathBuf};

pub fn validate_path(path: &str, allowed_dirs: &[PathBuf]) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);

    // Canonicalize path
    let path = path.canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    // Check against allow-list
    for allowed_dir in allowed_dirs {
        if path.starts_with(allowed_dir) {
            return Ok(path);
        }
    }

    Err("Path not in allowed directories".to_string())
}
```

### 12.7. Rate Limiting

IPC commands are rate-limited to prevent denial-of-service attacks.

**Rate Limiting Configuration:**

| Command Type | Rate Limit | Window |
|-------------|-----------|---------|
| Document Commands | 100/minute | 1 minute |
| Git Commands | 10/minute | 6 seconds |
| Window Commands | 50/minute | 30 seconds |
| Dialog Commands | 20/minute | 3 seconds |
| Configuration Commands | 200/minute | 30 seconds |
| Plugin Commands | 10/minute | 6 seconds |
| Event Commands | 100/minute | 1 minute |

**Rate Limiter Example:**

```rust
// Rate limiting
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct RateLimiter {
    limits: HashMap<String, (u32, Duration)>,
    requests: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    fn check_rate_limit(&self, user_id: &str, command: &str) -> Result<(), String> {
        let mut requests = self.requests.lock().await;
        let user_requests = requests.entry(user_id.to_string()).or_insert_with(Vec::new());

        let now = Instant::now();
        user_requests.retain(|&req| {
            now.duration_since(req) < self.limits.get(command).unwrap_or(&(60, Duration::from_secs(60)))
        });

        if user_requests.len() >= self.limits.get(command).unwrap().0 {
            return Err("Rate limit exceeded".to_string());
        }

        user_requests.push(now);
        Ok(())
    }
}
```

### 12.8. IPC Logging

All IPC operations are logged for security auditing and debugging.

**Logging Requirements:**

1. **User Identity:** All logs include user ID
2. **Operation:** All logs include IPC command/method
3. **Parameters:** All logs include input parameters (sanitized)
4. **Result:** All logs include operation result
5. **Timestamp:** All logs include UTC timestamp
6. **Error:** All logs include error information
7. **Duration:** All logs include operation duration

**Log Entry Structure:**

```json
{
  "timestamp": "2026-02-07T16:00:00.000Z",
  "level": "info",
  "user_id": "user-123",
  "operation": "get_document",
  "parameters": {
    "id": "doc-123"
  },
  "result": {
    "success": true
  },
  "duration_ms": 5.2,
  "trace_id": "trace-abc123"
}
```

**Logger Example:**

```rust
// IPC logging
use tracing::{info, warn, error, instrument};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct IpcLogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    user_id: String,
    operation: String,
    parameters: Option<JsonMap>,
    result: Option<JsonMap>,
    duration_ms: f64,
    trace_id: Option<String>,
}

pub struct IpcLogger {
    fn log(&self, entry: IpcLogEntry) {
        match entry.level.as_str() {
            "info" => info!(
                user_id = %s, operation = %s, duration_ms = %{}ms, trace_id = {:?}",
                entry.user_id, entry.operation, entry.duration_ms, entry.trace_id
            ),
            "warn" => warn!(
                user_id = %s, operation = %s, duration_ms = %{}ms, trace_id = {:?}",
                entry.user_id, entry.operation, entry.duration_ms, entry.trace_id
            ),
            "error" => error!(
                user_id = %s, operation = %s, error = {:?}, duration_ms = %{}ms, trace_id = {:?}",
                entry.user_id, entry.operation, entry.result, entry.duration_ms, entry.trace_id
            ),
            _ => (),
        }
    }
}
```

---

## 13. REFERENCES

### 13.1. Standards and Specifications

| Document | Version | Description |
|----------|---------|-------------|
| [JSON-RPC 2.0](https://www.jsonrpc.org/specification) | 2.0 | JSON-RPC Specification |
| [ISO/IEC 19510:2016](https://www.iso.org/standard/62884.html) | 2016 | Information technology — Open Distributed Processing — Reference Model |
| [ISO/IEC 27001:2013](https://www.iso.org/standard/54534.html) | 2013 | Information technology — Security techniques — Information security management systems |
| [ISO/IEC 27002:2013](https://www.iso.org/standard/54533.html) | 2013 | Information technology — Security techniques — Code of practice for information security controls |
| [RFC 7519](https://tools.ietf.org/html/rfc7519) | - | JSON Web Token (JWT) |
| [RFC 7523](https://tools.ietf.org/html/rfc7523) | - | JWT Secured Authorization Grants (JAG) |

### 13.2. Architecture Decision Records (ADRs)

| ADR | Title | Description |
|-----|-------|-------------|
| [ADR-001](../.adrs/adr-001-three-tier-jit-compilation.md) | Rust Language Selection | Rationale for selecting Rust as the primary implementation language |
| [ADR-010](../.adrs/adr-010-synchronization-primitives.md) | Security Architecture | Security architecture and threat modeling approach |

### 13.3. Requirements

| Requirement ID | Description |
|----------------|-------------|
| [REQ-IPC-001](../.adrs/ | IPC Protocol Requirements |
| [REQ-IPC-002](../.adrs/ | IPC Type Safety |
| [REQ-IPC-003](../.adrs/ | IPC Performance |
| [REQ-IPC-004](../.adrs/ | IPC Security |
| [REQ-IPC-005](../.adrs/ | IPC Reliability |
| [REQ-IPC-006](../.adrs/ | IPC Scalability |
| [REQ-IPC-007](../.adrs/ | IPC Testability |
| [REQ-IPC-008](../.adrs/ | IPC Observability |
| [REQ-IPC-009](../.adrs/ | IPC Maintainability |
| [REQ-IPC-010](../.adrs/ | IPC Documentation |
| [REQ-IPC-011](../.adrs/ | IPC Error Handling |
| [REQ-IPC-012](../.adrs/ | IPC Logging |
| [REQ-IPC-013](../.adrs/ | IPC Metrics |
| [REQ-IPC-014](../.adrs/ | IPC Tracing |
| [REQ-IPC-015](../.adrs/ | IPC Rate Limiting |
| [REQ-IPC-016](../.adrs/ | IPC Caching |
| [REQ-IPC-017](../.adrs/ | IPC Compression |
| [REQ-IPC-018](../.adrs/ | IPC Encryption |
| [REQ-IPC-019](../.adrs/ | IPC Authentication |
| [REQ-IPC-020](../.adrs/ | IPC Authorization |
| [REQ-IPC-021](../.adrs/ | IPC Input Validation |
| [REQ-IPC-022](../.adrs/ | IPC Output Sanitization |
| [REQ-IPC-023](../.adrs/ | IPC Path Traversal Prevention |
| [REQ-IPC-024](../.adrs/ | IPC Command Authorization |
| [REQ-IPC-025](../.adrs/ | IPC Event Subscription |
| [REQ-IPC-026](../.adrs/ | IPC Event Unsubscription |
| [REQ-IPC-027](../.adrs/ | IPC Event Filtering |
| [REQ-IPC-028](../.adrs/ | IPC Event Transformation |
| [REQ-IPC-029](../.adrs/ | IPC Event Aggregation |
| [REQ-IPC-030](../.adrs/ | IPC Event Correlation |
| [REQ-IPC-031](../.adrs/ | IPC Event Deduplication |
| [REQ-IPC-032](../.adrs/ | IPC Event Ordering |
| [REQ-IPC-033](../.adrs/ | IPC Event Buffering |
| [REQ-IPC-034](../.adrs/ | IPC Event Persistence |
| [REQ-IPC-035](../.adrs/ | IPC Event Replay |
| [REQ-IPC-036](../.adrs/ | IPC Event Archiving |
| [REQ-IPC-037](../.adrs/ | IPC Event Compression |
| [REQ-IPC-038](../.adrs/ | IPC Event Encryption |
| [REQ-IPC-039](../.adrs/ | IPC Event Authentication |
| [REQ-IPC-040](../.adrs/ | IPC Event Authorization |
| [REQ-IPC-041](../.adrs/ | IPC Event Input Validation |
| [REQ-IPC-042](../.adrs/ | IPC Event Output Sanitization |
| [REQ-IPC-043](../.adrs/ | IPC Event Path Traversal Prevention |
| [REQ-IPC-044](../.adrs/ | IPC Event Command Authorization |
| [REQ-IPC-045](../.adrs/ | IPC Stream Backpressure |
| [REQ-IPC-046](../.adrs/ | IPC Stream Flow Control |
| [REQ-IPC-047](../.adrs/ | IPC Stream Buffering |
| [REQ-IPC-048](../.adrs/ | IPC Stream Compression |
| [REQ-IPC-049](../.adrs/ | IPC Stream Encryption |
| [REQ-IPC-050](../.adrs/ | IPC Stream Authentication |
| [REQ-IPC-051](../.adrs/ | IPC Stream Authorization |
| [REQ-IPC-052](../.adrs/ | IPC Stream Input Validation |
| [REQ-IPC-053](../.adrs/ | IPC Stream Output Sanitization |
| [REQ-IPC-054](../.adrs/ | IPC Stream Path Traversal Prevention |
| [REQ-IPC-055](../.adrs/ | IPC Stream Command Authorization |
| [REQ-IPC-056](../.adrs/ | Capability Enforcement |
| [REQ-IPC-057](../.adrs/ | Input Validation |
| [REQ-IPC-058](../.adrs/ | Output Sanitization |
| [REQ-IPC-059](../.adrs/ | Path Traversal Prevention |
| [REQ-IPC-060](../.adrs/ | Command Authorization |
| [REQ-IPC-061](../.adrs/ | Document Commands |
| [REQ-IPC-062](../.adrs/ | Git Commands |
| [REQ-IPC-063](../.adrs/ | Window Commands |
| [REQ-IPC-064](../.adrs/ | Dialog Commands |
| [REQ-IPC-065](../.adrs/ | Configuration Commands |
| [REQ-IPC-066](../.adrs/ | Error Types |
| [REQ-IPC-067](../.adrs/ | Error Context |
| [REQ-IPC-068](../.adrs/ | Error Categorization |
| [REQ-IPC-069](../.adrs/ | Error Propagation |
| [REQ-IPC-070](../.adrs/ | Error Recovery |
| [REQ-SEC-081](../.adrs/ | Capability Enforcement |
| [REQ-SEC-082](../.adrs/ | Message Validation |
| [REQ-SEC-083](../.adrs/ | IPC Rate Limiting |
| [REQ-SEC-084](../.adrs/ | IPC Logging |
| [REQ-SEC-085](../.adrs/ | Desktop Isolation |

### 13.4. Design Elements

| Design Element | Description |
|----------------|-------------|
| [DES-IPC-001](../.adrs/ | IPC Protocol |
| [DES-IPC-002](../.adrs/ | IPC Command |
| [DES-IPC-003](../.adrs/ | IPC Event |
| [DES-IPC-004](../.adrs/ | IPC Stream |
| [DES-IPC-005](../.adrs/ | IPC Message |
| [DES-IPC-006](../.adrs/ | IPC Envelope |
| [DES-IPC-007](../.adrs/ | IPC Type |
| [DES-IPC-008](../.adrs/ | IpcError |
| [DES-IPC-009](../.adrs/ | IpcAuth |
| [DES-SEC-001](../.adrs/ | AuthenticationProvider |
| [DES-SEC-002](../.adrs/ | AuthorizationMiddleware |
| [DES-SEC-003](../.adrs/ | InputValidator |
| [DES-SEC-004](../.adrs/ | OutputSanitizer |
| [DES-SEC-005](../.adrs/ | SecurityLogger |
| [DES-SEC-006](../.adrs/ | RateLimiter |

### 13.5. Test Plan

| Test Plan Section | Description |
|-------------------|-------------|
| [Test Plan](../.adrs/ | Comprehensive test plan for IPC API |

### 13.6. External References

| Reference | URL | Description |
|-----------|-----|-------------|
| Tauri Documentation | https://tauri.app/v1/guides/ | Tauri framework documentation |
| Tauri IPC | https://tauri.app/v1/guides/features/inter-process-communication | Tauri IPC guide |
| Rust Serde | https://serde.rs/ | Rust serialization framework |
| Rust Tokio | https://tokio.rs/ | Rust async runtime |
| Rust Tracing | https://docs.rs/tracing/ | Rust instrumentation framework |
| JSON Web Tokens | https://jwt.io/ | JWT specification and libraries |
| OWASP Top 10 | https://owasp.org/www-project-top-ten/ | OWASP security risks |

---

**Document Control**

| Field | Value |
|-------|-------|
| Document ID | TACHYON-API-004 |
| Version | 1.0 |
| Status | Final |
| Last Updated | 2026-02-07 |
| Author | Technical Writer |
| Reviewer | Architecture Team |
| Approver | Project Lead |

---

**Change History**

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-02-07 | Technical Writer | Initial release of IPC API documentation |

---

**End of Document**
```
