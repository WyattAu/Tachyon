# TACHYON: DESKTOP COMMANDS API SPECIFICATION

**Document ID:** TACHYON-API-007-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001
**Dependencies:** [TACHYON-STD-V1.0](../../.adrs/ [TACHYON-REQ-DESK-V1.0](../../.adrs/ [TACHYON-DES-DESK-V1.0](../../.adrs/ [TACHYON-ADR-002-V1.0](../../.adrs/adr-002-bm25-search-parameters.md), [TACHYON-ADR-009-V1.0](../../.adrs/adr-009-race-condition-mitigation.md)

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Command Design Principles](#2-command-design-principles)
3. [Document Commands](#3-document-commands)
4. [Repository Commands](#4-repository-commands)
5. [Search Commands](#5-search-commands)
6. [System Commands](#6-system-commands)
7. [Command Security](#7-command-security)
8. [Command Performance](#8-command-performance)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document defines the comprehensive Desktop Commands API specification for the Tachyon Desktop Application. The Desktop Commands API provides the interface between the Tauri-based WebView frontend and the Rust backend, enabling secure, type-safe inter-process communication (IPC) for all desktop-specific operations.

### 1.2. Scope

This specification covers:

- **Document Commands:** Create, read, update, delete, list, and batch operations for documents
- **Repository Commands:** Git repository management operations (add, remove, sync, status, list)
- **Search Commands:** Query, filter, sort, and autocomplete operations for document search
- **System Commands:** Application initialization, configuration, status, and termination operations
- **Command Security:** Authentication, authorization, and input validation requirements
- **Command Performance:** Latency, throughput, and optimization requirements

Out of scope:

- Server HTTP API endpoints (covered in [TACHYON-API-002-V1.0](server_http_api_specification.md))
- WebSocket protocol specification (covered in [TACHYON-PRO-003-V1.0](websocket_protocol_specification.md))
- Core rendering engine internal APIs (covered in [TACHYON-API-001-V1.0](core_api_specification.md))

### 1.3. Document Dependencies

This document depends on the following documents:

- [TACHYON-STD-V1.0](../../.adrs/ - Coding and Documentation Standards
- [TACHYON-REQ-DESK-V1.0](../../.adrs/ - Desktop Application Requirements
- [TACHYON-DES-DESK-V1.0](../../.adrs/ - Desktop Application Design
- [TACHYON-ADR-002-V1.0](../../.adrs/adr-002-bm25-search-parameters.md) - Tauri for Desktop Application
- [TACHYON-ADR-009-V1.0](../../.adrs/adr-009-race-condition-mitigation.md) - IPC Communication Architecture
- [TACHYON-TMA-V1.0](../../.adrs/ - Threat Model Analysis

### 1.4. Terminology

| Term | Definition |
|-------|-------------|
| **IPC Command** | A Tauri command that enables the WebView frontend to invoke Rust backend functions with type-safe serialization |
| **IPC Event** | A Tauri event that enables the Rust backend to push notifications to the WebView frontend |
| **Document** | A Markdown file stored in the Git repository containing knowledge content |
| **Repository** | A Git repository containing documents and metadata managed by the Tachyon system |
| **Command Handler** | A Rust function decorated with `#[tauri::command]` that processes IPC commands from the WebView |

---

## 2. COMMAND DESIGN PRINCIPLES

### 2.1. Architectural Principles

The Desktop Commands API adheres to the following architectural principles derived from [ADR-002](../../.adrs/adr-002-bm25-search-parameters.md) and [ADR-009](../../.adrs/adr-009-race-condition-mitigation.md):

#### 2.1.1. Type Safety

**Principle:** All IPC commands must provide compile-time type safety through Rust's type system and serde serialization.

**Implementation:**

- All command parameters must be strongly-typed Rust structs implementing `serde::Serialize` and `serde::Deserialize`
- Command return types must be `Result<T, E>` where `T` implements `serde::Serialize`
- Error types must implement `std::error::Error` and `std::fmt::Display`
- TypeScript types are automatically generated from Rust command definitions

**Rationale:** Type safety eliminates entire classes of runtime errors and enables confident refactoring. Compile-time type checking ensures that all IPC contracts are verified before deployment.

#### 2.1.2. Capability-Based Security

**Principle:** All IPC commands must enforce capability-based authorization following the principle of least privilege.

**Implementation:**

- Each command must declare required capabilities in the Tauri capability configuration
- Commands must validate user permissions before executing privileged operations
- File system operations must be scoped to authorized directories
- Git operations must be scoped to authorized repositories

**Rationale:** Capability-based security provides fine-grained control over system resource access, reducing the attack surface and preventing unauthorized operations.

#### 2.1.3. Input Validation

**Principle:** All IPC command inputs must be validated before processing to prevent injection attacks and ensure data integrity.

**Implementation:**

- All command parameters must be validated for type, range, and format constraints
- File paths must be sanitized to prevent directory traversal attacks
- User input must be validated against defined domain constraints
- Validation failures must return descriptive error messages without exposing internal details

**Rationale:** Input validation prevents injection attacks, ensures data integrity, and provides clear error messages for users.

#### 2.1.4. Error Handling

**Principle:** All IPC commands must implement comprehensive error handling with proper error propagation and user-friendly messages.

**Implementation:**

- Commands must return `Result<T, E>` where `E` is a descriptive error type
- Error types must implement conversion from underlying error types
- Error messages must be user-friendly while preserving debug information
- Critical errors must be logged for audit trail and debugging

**Rationale:** Comprehensive error handling ensures that errors are handled consistently across the IPC boundary and provides actionable information to users and developers.

#### 2.1.5. Performance

**Principle:** All IPC commands must meet defined performance requirements for latency and throughput.

**Implementation:**

- Commands must complete within specified latency requirements (typically < 100ms)
- Long-running operations must support cancellation and progress reporting
- Commands must use async/await to avoid blocking the UI thread
- Resource-intensive operations must implement batching and caching

**Rationale:** Performance requirements ensure that the desktop application remains responsive and provides a smooth user experience.

### 2.2. Command Naming Conventions

**Standard:** All IPC command names must use `snake_case` naming convention with descriptive, action-oriented names.

**Format:**

- **Create operations:** `create_<resource>` (e.g., `create_document`)
- **Read operations:** `get_<resource>` (e.g., `get_document`)
- **Update operations:** `update_<resource>` (e.g., `update_document`)
- **Delete operations:** `delete_<resource>` (e.g., `delete_document`)
- **List operations:** `list_<resources>` (e.g., `list_documents`)
- **Batch operations:** `batch_<operation>` (e.g., `batch_create_documents`)

**Rationale:** Consistent naming conventions improve discoverability, reduce cognitive load, and enable automated type generation.

### 2.3. Command Response Format

**Standard:** All IPC command responses must follow a consistent format for success and error cases.

**Success Response Format:**

```json
{
  "success": true,
  "data": { /* Response data specific to command */ }
}
```

**Error Response Format:**

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "User-friendly error message",
    "details": "Additional error details for debugging"
  }
}
```

**Rationale:** Consistent response formats enable uniform error handling across the frontend and simplify client-side code.

### 2.4. Command Versioning

**Standard:** All IPC commands must support versioning to enable backward compatibility and graceful deprecation.

**Implementation:**

- Commands must accept an optional `version` parameter with default value
- New versions of commands must maintain backward compatibility with previous versions
- Deprecated commands must emit deprecation warnings and document migration paths
- Major version changes must be communicated through release notes and documentation

**Rationale:** Versioning enables smooth evolution of the API without breaking existing clients and provides clear migration paths for developers.

### 2.5. Command Documentation

**Standard:** All IPC commands must be documented with formal documentation comments specifying parameters, returns, and error conditions.

**Rust Documentation Format:**

```rust
/// Summary of the command's purpose.
///
/// # Arguments
///
/// * `param1` - Description of parameter 1 and its constraints
/// * `param2` - Description of parameter 2 and its constraints
///
/// # Returns
///
/// Description of the return value and its structure.
///
/// # Errors
///
/// * `ErrorType1` - Description of when this error occurs
/// * `ErrorType2` - Description of when this error occurs
///
/// # Examples
///
/// ```rust
/// let result = command_name(param1, param2).await?;
/// ```
#[tauri::command]
pub async fn command_name(
    param1: Type1,
    param2: Type2,
) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

**TypeScript Documentation Format:**

```typescript
/**
 * Summary of the command's purpose.
 *
 * @param param1 - Description of parameter 1 and its constraints
 * @param param2 - Description of parameter 2 and its constraints
 * @returns Promise resolving to the return value
 * @throws {ErrorType1} Description of when this error occurs
 * @throws {ErrorType2} Description of when this error occurs
 *
 * @example
 * ```typescript
 * const result = await invoke('command_name', { param1, param2 });
 * ```
 */
export async function commandName(
  param1: Type1,
  param2: Type2
): Promise<ReturnType> {
  return invoke('command_name', { param1, param2 });
}
```

**Rationale:** Comprehensive documentation ensures that commands are self-documenting, facilitates code generation, and enables developers to use commands correctly without referring to external documentation.

---

## 3. DOCUMENT COMMANDS

Document commands provide CRUD (Create, Read, Update, Delete) operations for managing Markdown documents within the Tachyon system. These commands enable the WebView frontend to create, retrieve, modify, and delete documents stored in the Git repository.

### 3.1. Create Document

**Command ID:** CMD-DOC-001
**Command Name:** `create_document`
**Related Requirements:** REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit)

#### Rust Implementation

```rust
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use std::path::PathBuf;

/// Creates a new document in the repository with the specified content.
///
/// # Arguments
///
/// * `request` - Document creation request containing path and content
///
/// # Returns
///
/// `CreateDocumentResponse` containing the created document's metadata
///
/// # Errors
///
/// * `DocumentError::InvalidPath` - When the document path is invalid or contains illegal characters
/// * `DocumentError::DocumentExists` - When a document already exists at the specified path
/// * `DocumentError::WriteFailed` - When writing the document to disk fails
/// * `DocumentError::GitCommitFailed` - When auto-commit operation fails
///
/// # Examples
///
/// ```rust
/// let response = create_document(CreateDocumentRequest {
///     path: "docs/new-document.md".to_string(),
///     content: "# New Document\n\nContent here.".to_string(),
///     author: "user@example.com".to_string(),
/// }).await?;
/// ```
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, AppState>,
) -> Result<CreateDocumentResponse, DocumentError> {
    // Validate document path
    let path = validate_document_path(&request.path)?;
    
    // Check if document already exists
    let full_path = state.repository_path.join(&path);
    if full_path.exists() {
        return Err(DocumentError::DocumentExists(request.path));
    }
    
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| DocumentError::WriteFailed(e.to_string()))?;
    }
    
    // Write document content
    tokio::fs::write(&full_path, request.content.as_bytes()).await
        .map_err(|e| DocumentError::WriteFailed(e.to_string()))?;
    
    // Trigger auto-commit if enabled
    if state.preferences.auto_save_enabled {
        let commit_message = format!("Create document: {}", request.path);
        git_commit(&state.repository_path, &commit_message, &request.author).await?;
    }
    
    // Invalidate cache for this document
    state.cache.invalidate(&path).await;
    
    // Emit document created event
    state.window.emit("document-created", DocumentCreatedEvent {
        path: request.path.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    
    Ok(CreateDocumentResponse {
        path: request.path,
        created_at: chrono::Utc::now(),
        size: request.content.len(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    /// Relative path to the document within the repository
    pub path: String,
    
    /// Markdown content of the document
    pub content: String,
    
    /// Author identifier for Git commit
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    /// Path of the created document
    pub path: String,
    
    /// Timestamp when the document was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Size of the document in bytes
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentCreatedEvent {
    pub path: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Creates a new document in the repository.
 *
 * @param request - Document creation request
 * @returns Promise resolving to the created document's metadata
 * @throws {DocumentError} When document creation fails
 *
 * @example
 * ```typescript
 * const response = await createDocument({
 *   path: 'docs/new-document.md',
 *   content: '# New Document\n\nContent here.',
 *   author: 'user@example.com'
 * });
 * ```
 */
export async function createDocument(
  request: CreateDocumentRequest
): Promise<CreateDocumentResponse> {
  return invoke<CreateDocumentResponse>('create_document', request);
}

export interface CreateDocumentRequest {
  /** Relative path to the document within the repository */
  path: string;
  
  /** Markdown content of the document */
  content: string;
  
  /** Author identifier for Git commit */
  author: string;
}

export interface CreateDocumentResponse {
  /** Path of the created document */
  path: string;
  
  /** Timestamp when the document was created (ISO 8601) */
  created_at: string;
  
  /** Size of the document in bytes */
  size: number;
}
```

#### Request/Response Format

**Request:**
```json
{
  "path": "docs/new-document.md",
  "content": "# New Document\n\nContent here.",
  "author": "user@example.com"
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "path": "docs/new-document.md",
    "created_at": "2026-02-05T16:00:00.000Z",
    "size": 42
  }
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "code": "DOCUMENT_EXISTS",
    "message": "A document already exists at the specified path",
    "details": "Path: docs/new-document.md"
  }
}
```

#### Constraints

- `path`: Must be relative to repository root, maximum 255 characters, no `..` segments
- `content`: Maximum 10MB (10,485,760 bytes)
- `author`: Valid email address format, maximum 254 characters

#### Performance Requirements

- **Latency:** < 50ms for documents < 1MB
- **Throughput:** Support for 100 concurrent create operations

### 3.2. Get Document

**Command ID:** CMD-DOC-002
**Command Name:** `get_document`
**Related Requirements:** REQ-DESK-087 (Initial Load Time), REQ-DESK-041 (LRU Cache)

#### Rust Implementation

```rust
/// Retrieves a document from the repository by path.
///
/// # Arguments
///
/// * `path` - Relative path to the document within the repository
///
/// # Returns
///
/// `GetDocumentResponse` containing the document's content and metadata
///
/// # Errors
///
/// * `DocumentError::NotFound` - When the document does not exist
/// * `DocumentError::ReadFailed` - When reading the document fails
/// * `DocumentError::InvalidPath` - When the document path is invalid
///
/// # Examples
///
/// ```rust
/// let response = get_document("docs/example.md".to_string()).await?;
/// ```
#[command]
pub async fn get_document(
    path: String,
    state: State<'_, AppState>,
) -> Result<GetDocumentResponse, DocumentError> {
    // Validate document path
    let validated_path = validate_document_path(&path)?;
    
    // Check cache first
    if let Some(cached) = state.cache.get(&validated_path).await {
        return Ok(GetDocumentResponse {
            path: validated_path,
            content: cached.content,
            metadata: cached.metadata,
            cached: true,
        });
    }
    
    // Read document from disk
    let full_path = state.repository_path.join(&validated_path);
    let content = tokio::fs::read_to_string(&full_path).await
        .map_err(|e| DocumentError::ReadFailed(e.to_string()))?;
    
    // Get document metadata
    let metadata = tokio::fs::metadata(&full_path).await
        .map_err(|e| DocumentError::ReadFailed(e.to_string()))?;
    
    // Cache the document
    state.cache.insert(validated_path.clone(), content.clone()).await;
    
    Ok(GetDocumentResponse {
        path: validated_path,
        content,
        metadata: DocumentMetadata {
            size: metadata.len() as usize,
            modified: metadata.modified()
                .ok_or_else(|| DocumentError::ReadFailed(
                    "Failed to get modification time".to_string()
                ))?,
            created: metadata.created()
                .ok_or_else(|| DocumentError::ReadFailed(
                    "Failed to get creation time".to_string()
                ))?,
        },
        cached: false,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentResponse {
    /// Path of the document
    pub path: String,
    
    /// Markdown content of the document
    pub content: String,
    
    /// Document metadata
    pub metadata: DocumentMetadata,
    
    /// Whether the response was served from cache
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Size of the document in bytes
    pub size: usize,
    
    /// Last modification timestamp
    pub modified: chrono::DateTime<chrono::Utc>,
    
    /// Creation timestamp
    pub created: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Retrieves a document from the repository.
 *
 * @param path - Relative path to the document
 * @returns Promise resolving to the document's content and metadata
 * @throws {DocumentError} When document retrieval fails
 *
 * @example
 * ```typescript
 * const response = await getDocument('docs/example.md');
 * console.log(response.content);
 * ```
 */
export async function getDocument(
  path: string
): Promise<GetDocumentResponse> {
  return invoke<GetDocumentResponse>('get_document', { path });
}

export interface GetDocumentResponse {
  /** Path of the document */
  path: string;
  
  /** Markdown content of the document */
  content: string;
  
  /** Document metadata */
  metadata: DocumentMetadata;
  
  /** Whether the response was served from cache */
  cached: boolean;
}

export interface DocumentMetadata {
  /** Size of the document in bytes */
  size: number;
  
  /** Last modification timestamp (ISO 8601) */
  modified: string;
  
  /** Creation timestamp (ISO 8601) */
  created: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "path": "docs/example.md"
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "path": "docs/example.md",
    "content": "# Example Document\n\nContent here.",
    "metadata": {
      "size": 42,
      "modified": "2026-02-05T16:00:00.000Z",
      "created": "2026-02-05T15:00:00.000Z"
    },
    "cached": false
  }
}
```

#### Constraints

- `path`: Must be relative to repository root, maximum 255 characters

#### Performance Requirements

- **Latency:** < 20ms for cached documents, < 100ms for uncached documents < 1MB
- **Cache Hit Rate:** > 80% for frequently accessed documents

### 3.3. Update Document

**Command ID:** CMD-DOC-003
**Command Name:** `update_document`
**Related Requirements:** REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit), REQ-DESK-042 (Cache Invalidation)

#### Rust Implementation

```rust
/// Updates an existing document with new content.
///
/// # Arguments
///
/// * `request` - Document update request containing path and new content
///
/// # Returns
///
/// `UpdateDocumentResponse` containing the updated document's metadata
///
/// # Errors
///
/// * `DocumentError::NotFound` - When the document does not exist
/// * `DocumentError::InvalidPath` - When the document path is invalid
/// * `DocumentError::WriteFailed` - When writing the document fails
/// * `DocumentError::GitCommitFailed` - When auto-commit operation fails
///
/// # Examples
///
/// ```rust
/// let response = update_document(UpdateDocumentRequest {
///     path: "docs/example.md".to_string(),
///     content: "# Updated Document\n\nNew content.".to_string(),
///     author: "user@example.com".to_string(),
/// }).await?;
/// ```
#[command]
pub async fn update_document(
    request: UpdateDocumentRequest,
    state: State<'_, AppState>,
) -> Result<UpdateDocumentResponse, DocumentError> {
    // Validate document path
    let path = validate_document_path(&request.path)?;
    
    // Check if document exists
    let full_path = state.repository_path.join(&path);
    if !full_path.exists() {
        return Err(DocumentError::NotFound(request.path));
    }
    
    // Write updated content
    tokio::fs::write(&full_path, request.content.as_bytes()).await
        .map_err(|e| DocumentError::WriteFailed(e.to_string()))?;
    
    // Trigger auto-commit if enabled
    if state.preferences.auto_save_enabled {
        let commit_message = format!("Update document: {}", request.path);
        git_commit(&state.repository_path, &commit_message, &request.author).await?;
    }
    
    // Invalidate cache for this document
    state.cache.invalidate(&path).await;
    
    // Emit document updated event
    state.window.emit("document-updated", DocumentUpdatedEvent {
        path: request.path.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    
    Ok(UpdateDocumentResponse {
        path: request.path,
        updated_at: chrono::Utc::now(),
        size: request.content.len(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    /// Relative path to the document within the repository
    pub path: String,
    
    /// Updated Markdown content of the document
    pub content: String,
    
    /// Author identifier for Git commit
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentResponse {
    /// Path of the updated document
    pub path: String,
    
    /// Timestamp when the document was updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    
    /// Size of the document in bytes
    pub size: usize,
}
```

#### TypeScript Implementation

```typescript
/**
 * Updates an existing document with new content.
 *
 * @param request - Document update request
 * @returns Promise resolving to the updated document's metadata
 * @throws {DocumentError} When document update fails
 *
 * @example
 * ```typescript
 * const response = await updateDocument({
 *   path: 'docs/example.md',
 *   content: '# Updated Document\n\nNew content.',
 *   author: 'user@example.com'
 * });
 * ```
 */
export async function updateDocument(
  request: UpdateDocumentRequest
): Promise<UpdateDocumentResponse> {
  return invoke<UpdateDocumentResponse>('update_document', request);
}

export interface UpdateDocumentRequest {
  /** Relative path to the document within the repository */
  path: string;
  
  /** Updated Markdown content of the document */
  content: string;
  
  /** Author identifier for Git commit */
  author: string;
}

export interface UpdateDocumentResponse {
  /** Path of the updated document */
  path: string;
  
  /** Timestamp when the document was updated (ISO 8601) */
  updated_at: string;
  
  /** Size of the document in bytes */
  size: number;
}
```

#### Request/Response Format

**Request:**
```json
{
  "path": "docs/example.md",
  "content": "# Updated Document\n\nNew content.",
  "author": "user@example.com"
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "path": "docs/example.md",
    "updated_at": "2026-02-05T16:00:00.000Z",
    "size": 45
  }
}
```

#### Constraints

- `path`: Must be relative to repository root, maximum 255 characters
- `content`: Maximum 10MB (10,485,760 bytes)
- `author`: Valid email address format, maximum 254 characters

#### Performance Requirements

- **Latency:** < 50ms for documents < 1MB
- **Throughput:** Support for 100 concurrent update operations

### 3.4. Delete Document

**Command ID:** CMD-DOC-004
**Command Name:** `delete_document`
**Related Requirements:** REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit)

#### Rust Implementation

```rust
/// Deletes a document from the repository.
///
/// # Arguments
///
/// * `request` - Document deletion request containing path and author
///
/// # Returns
///
/// `DeleteDocumentResponse` confirming the deletion
///
/// # Errors
///
/// * `DocumentError::NotFound` - When the document does not exist
/// * `DocumentError::InvalidPath` - When the document path is invalid
/// * `DocumentError::DeleteFailed` - When deleting the document fails
/// * `DocumentError::GitCommitFailed` - When auto-commit operation fails
///
/// # Examples
///
/// ```rust
/// let response = delete_document(DeleteDocumentRequest {
///     path: "docs/example.md".to_string(),
///     author: "user@example.com".to_string(),
/// }).await?;
/// ```
#[command]
pub async fn delete_document(
    request: DeleteDocumentRequest,
    state: State<'_, AppState>,
) -> Result<DeleteDocumentResponse, DocumentError> {
    // Validate document path
    let path = validate_document_path(&request.path)?;
    
    // Check if document exists
    let full_path = state.repository_path.join(&path);
    if !full_path.exists() {
        return Err(DocumentError::NotFound(request.path));
    }
    
    // Delete the document
    tokio::fs::remove_file(&full_path).await
        .map_err(|e| DocumentError::DeleteFailed(e.to_string()))?;
    
    // Trigger auto-commit if enabled
    if state.preferences.auto_save_enabled {
        let commit_message = format!("Delete document: {}", request.path);
        git_commit(&state.repository_path, &commit_message, &request.author).await?;
    }
    
    // Invalidate cache for this document
    state.cache.invalidate(&path).await;
    
    // Emit document deleted event
    state.window.emit("document-deleted", DocumentDeletedEvent {
        path: request.path.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    
    Ok(DeleteDocumentResponse {
        path: request.path,
        deleted_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentRequest {
    /// Relative path to the document within the repository
    pub path: String,
    
    /// Author identifier for Git commit
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentResponse {
    /// Path of the deleted document
    pub path: String,
    
    /// Timestamp when the document was deleted
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Deletes a document from the repository.
 *
 * @param request - Document deletion request
 * @returns Promise resolving to confirmation of deletion
 * @throws {DocumentError} When document deletion fails
 *
 * @example
 * ```typescript
 * const response = await deleteDocument({
 *   path: 'docs/example.md',
 *   author: 'user@example.com'
 * });
 * ```
 */
export async function deleteDocument(
  request: DeleteDocumentRequest
): Promise<DeleteDocumentResponse> {
  return invoke<DeleteDocumentResponse>('delete_document', request);
}

export interface DeleteDocumentRequest {
  /** Relative path to the document within the repository */
  path: string;
  
  /** Author identifier for Git commit */
  author: string;
}

export interface DeleteDocumentResponse {
  /** Path of the deleted document */
  path: string;
  
  /** Timestamp when the document was deleted (ISO 8601) */
  deleted_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "path": "docs/example.md",
  "author": "user@example.com"
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "path": "docs/example.md",
    "deleted_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- `path`: Must be relative to repository root, maximum 255 characters
- `author`: Valid email address format, maximum 254 characters

#### Performance Requirements

- **Latency:** < 50ms
- **Throughput:** Support for 100 concurrent delete operations

### 3.5. List Documents

**Command ID:** CMD-DOC-005
**Command Name:** `list_documents`
**Related Requirements:** REQ-DESK-016 (Sidebar Navigation), REQ-DESK-041 (LRU Cache)

#### Rust Implementation

```rust
/// Lists all documents in the repository matching optional filters.
///
/// # Arguments
///
/// * `request` - Document list request with optional filters
///
/// # Returns
///
/// `ListDocumentsResponse` containing the list of documents
///
/// # Errors
///
/// * `DocumentError::ReadFailed` - When reading the repository fails
/// * `DocumentError::InvalidPath` - When the base path is invalid
///
/// # Examples
///
/// ```rust
/// let response = list_documents(ListDocumentsRequest {
///     path: Some("docs".to_string()),
///     extension: Some("md".to_string()),
///     recursive: Some(true),
/// }).await?;
/// ```
#[command]
pub async fn list_documents(
    request: ListDocumentsRequest,
    state: State<'_, AppState>,
) -> Result<ListDocumentsResponse, DocumentError> {
    let base_path = request.path.unwrap_or_else(|| ".".to_string());
    let validated_path = validate_document_path(&base_path)?;
    let full_path = state.repository_path.join(&validated_path);
    
    let mut documents = Vec::new();
    
    // Walk the directory tree
    let mut entries = tokio::fs::read_dir(&full_path).await
        .map_err(|e| DocumentError::ReadFailed(e.to_string()))?;
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| DocumentError::ReadFailed(e.to_string()))?
    {
        let path = entry.path();
        let metadata = entry.metadata().await
            .map_err(|e| DocumentError::ReadFailed(e.to_string()))?;
        
        if metadata.is_file() {
            // Check extension filter
            if let Some(ref ext) = request.extension {
                if path.extension().and_then(|s| s.to_str()) != Some(ext.as_str()) {
                    continue;
                }
            }
            
            // Get relative path
            let relative_path = path.strip_prefix(&state.repository_path)
                .map_err(|_| DocumentError::InvalidPath(
                    "Failed to get relative path".to_string()
                ))?
                .to_string_lossy()
                .to_string();
            
            documents.push(DocumentInfo {
                path: relative_path,
                size: metadata.len() as usize,
                modified: metadata.modified()
                    .ok_or_else(|| DocumentError::ReadFailed(
                        "Failed to get modification time".to_string()
                    ))?,
                created: metadata.created()
                    .ok_or_else(|| DocumentError::ReadFailed(
                        "Failed to get creation time".to_string()
                    ))?,
            });
        } else if metadata.is_dir() && request.recursive.unwrap_or(false) {
            // Recursively process directories
            let sub_request = ListDocumentsRequest {
                path: Some(path.strip_prefix(&state.repository_path)
                    .map_err(|_| DocumentError::InvalidPath(
                        "Failed to get relative path".to_string()
                    ))?
                    .to_string_lossy()
                    .to_string()),
                extension: request.extension.clone(),
                recursive: Some(true),
            };
            let sub_response = list_documents(sub_request, state).await?;
            documents.extend(sub_response.documents);
        }
    }
    
    // Sort documents by path
    documents.sort_by(|a, b| a.path.cmp(&b.path));
    
    Ok(ListDocumentsResponse { documents })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsRequest {
    /// Base path to list documents from (default: repository root)
    pub path: Option<String>,
    
    /// File extension filter (e.g., "md")
    pub extension: Option<String>,
    
    /// Whether to recursively list subdirectories (default: false)
    pub recursive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsResponse {
    /// List of documents matching the filters
    pub documents: Vec<DocumentInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentInfo {
    /// Relative path to the document
    pub path: String,
    
    /// Size of the document in bytes
    pub size: usize,
    
    /// Last modification timestamp
    pub modified: chrono::DateTime<chrono::Utc>,
    
    /// Creation timestamp
    pub created: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Lists all documents in the repository matching optional filters.
 *
 * @param request - Document list request with optional filters
 * @returns Promise resolving to the list of documents
 * @throws {DocumentError} When document listing fails
 *
 * @example
 * ```typescript
 * const response = await listDocuments({
 *   path: 'docs',
 *   extension: 'md',
 *   recursive: true
 * });
 * console.log(response.documents);
 * ```
 */
export async function listDocuments(
  request: ListDocumentsRequest
): Promise<ListDocumentsResponse> {
  return invoke<ListDocumentsResponse>('list_documents', request);
}

export interface ListDocumentsRequest {
  /** Base path to list documents from (default: repository root) */
  path?: string;
  
  /** File extension filter (e.g., "md") */
  extension?: string;
  
  /** Whether to recursively list subdirectories (default: false) */
  recursive?: boolean;
}

export interface ListDocumentsResponse {
  /** List of documents matching the filters */
  documents: DocumentInfo[];
}

export interface DocumentInfo {
  /** Relative path to the document */
  path: string;
  
  /** Size of the document in bytes */
  size: number;
  
  /** Last modification timestamp (ISO 8601) */
  modified: string;
  
  /** Creation timestamp (ISO 8601) */
  created: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "path": "docs",
  "extension": "md",
  "recursive": true
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "documents": [
      {
        "path": "docs/example.md",
        "size": 42,
        "modified": "2026-02-05T16:00:00.000Z",
        "created": "2026-02-05T15:00:00.000Z"
      }
    ]
  }
}
```

#### Constraints

- `path`: Must be relative to repository root, maximum 255 characters
- `extension`: Maximum 10 characters
- `recursive`: Boolean value

#### Performance Requirements

- **Latency:** < 200ms for repositories with up to 10,000 documents
- **Throughput:** Support for 50 concurrent list operations

### 3.6. Batch Create Documents

**Command ID:** CMD-DOC-006
**Command Name:** `batch_create_documents`
**Related Requirements:** REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit)

#### Rust Implementation

```rust
/// Creates multiple documents in a single batch operation.
///
/// # Arguments
///
/// * `request` - Batch document creation request
///
/// # Returns
///
/// `BatchCreateDocumentsResponse` containing results for each document
///
/// # Errors
///
/// * `DocumentError::BatchFailed` - When the batch operation fails
///
/// # Examples
///
/// ```rust
/// let response = batch_create_documents(BatchCreateDocumentsRequest {
///     documents: vec![
///         CreateDocumentRequest {
///             path: "docs/doc1.md".to_string(),
///             content: "# Doc 1".to_string(),
///             author: "user@example.com".to_string(),
///         },
///         CreateDocumentRequest {
///             path: "docs/doc2.md".to_string(),
///             content: "# Doc 2".to_string(),
///             author: "user@example.com".to_string(),
///         },
///     ],
/// }).await?;
/// ```
#[command]
pub async fn batch_create_documents(
    request: BatchCreateDocumentsRequest,
    state: State<'_, AppState>,
) -> Result<BatchCreateDocumentsResponse, DocumentError> {
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for doc_request in request.documents {
        match create_document(doc_request, state.clone()).await {
            Ok(response) => {
                results.push(BatchDocumentResult {
                    path: response.path,
                    success: true,
                    error: None,
                });
                success_count += 1;
            }
            Err(error) => {
                results.push(BatchDocumentResult {
                    path: error.path().unwrap_or("unknown".to_string()),
                    success: false,
                    error: Some(error.to_string()),
                });
                failure_count += 1;
            }
        }
    }
    
    // Commit all successful operations
    if success_count > 0 && state.preferences.auto_save_enabled {
        let commit_message = format!(
            "Batch create documents: {} created, {} failed",
            success_count, failure_count
        );
        git_commit(&state.repository_path, &commit_message, "batch@example.com").await?;
    }
    
    Ok(BatchCreateDocumentsResponse {
        results,
        success_count,
        failure_count,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCreateDocumentsRequest {
    /// List of document creation requests
    pub documents: Vec<CreateDocumentRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCreateDocumentsResponse {
    /// Results for each document creation attempt
    pub results: Vec<BatchDocumentResult>,
    
    /// Number of successful document creations
    pub success_count: usize,
    
    /// Number of failed document creations
    pub failure_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchDocumentResult {
    /// Path of the document
    pub path: String,
    
    /// Whether the operation succeeded
    pub success: bool,
    
    /// Error message if the operation failed
    pub error: Option<String>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Creates multiple documents in a single batch operation.
 *
 * @param request - Batch document creation request
 * @returns Promise resolving to results for each document
 * @throws {DocumentError} When the batch operation fails
 *
 * @example
 * ```typescript
 * const response = await batchCreateDocuments({
 *   documents: [
 *     { path: 'docs/doc1.md', content: '# Doc 1', author: 'user@example.com' },
 *     { path: 'docs/doc2.md', content: '# Doc 2', author: 'user@example.com' }
 *   ]
 * });
 * ```
 */
export async function batchCreateDocuments(
  request: BatchCreateDocumentsRequest
): Promise<BatchCreateDocumentsResponse> {
  return invoke<BatchCreateDocumentsResponse>('batch_create_documents', request);
}

export interface BatchCreateDocumentsRequest {
  /** List of document creation requests */
  documents: CreateDocumentRequest[];
}

export interface BatchCreateDocumentsResponse {
  /** Results for each document creation attempt */
  results: BatchDocumentResult[];
  
  /** Number of successful document creations */
  success_count: number;
  
  /** Number of failed document creations */
  failure_count: number;
}

export interface BatchDocumentResult {
  /** Path of the document */
  path: string;
  
  /** Whether the operation succeeded */
  success: boolean;
  
  /** Error message if the operation failed */
  error?: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "documents": [
    {
      "path": "docs/doc1.md",
      "content": "# Doc 1",
      "author": "user@example.com"
    },
    {
      "path": "docs/doc2.md",
      "content": "# Doc 2",
      "author": "user@example.com"
    }
  ]
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "results": [
      {
        "path": "docs/doc1.md",
        "success": true,
        "error": null
      },
      {
        "path": "docs/doc2.md",
        "success": true,
        "error": null
      }
    ],
    "success_count": 2,
    "failure_count": 0
  }
}
```

#### Constraints

- `documents`: Maximum 100 documents per batch
- Individual document constraints same as `create_document`

#### Performance Requirements

- **Latency:** < 500ms for batches of up to 100 documents
- **Throughput:** Support for 10 concurrent batch operations

---

## 4. REPOSITORY COMMANDS

Repository commands provide Git repository management operations including initialization, cloning, synchronization, and status queries. These commands enable the WebView frontend to interact with Git repositories for version control and remote synchronization.

### 4.1. Add to Repository

**Command ID:** CMD-REP-001
**Command Name:** `add_to_repository`
**Related Requirements:** REQ-DESK-036 (Repository Initialization), REQ-DESK-038 (Auto-Commit)

#### Rust Implementation

```rust
use git2::{Repository, IndexAddOption};

/// Stages files for commit in the Git repository.
///
/// # Arguments
///
/// * `request` - Repository add request containing file paths
///
/// # Returns
///
/// `AddToRepositoryResponse` containing the staged files
///
/// # Errors
///
/// * `RepositoryError::NotInitialized` - When the repository is not initialized
/// * `RepositoryError::FileNotFound` - When a specified file does not exist
/// * `RepositoryError::GitFailed` - When the Git operation fails
///
/// # Examples
///
/// ```rust
/// let response = add_to_repository(AddToRepositoryRequest {
///     paths: vec!["docs/example.md".to_string()],
/// }).await?;
/// ```
#[command]
pub async fn add_to_repository(
    request: AddToRepositoryRequest,
    state: State<'_, AppState>,
) -> Result<AddToRepositoryResponse, RepositoryError> {
    // Open the repository
    let repo = Repository::open(&state.repository_path)
        .map_err(|e| RepositoryError::NotInitialized(e.to_string()))?;
    
    let mut index = repo.index()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
    
    let mut staged_files = Vec::new();
    
    for path in request.paths {
        let full_path = state.repository_path.join(&path);
        
        // Check if file exists
        if !full_path.exists() {
            return Err(RepositoryError::FileNotFound(path));
        }
        
        // Add file to index
        index.add_path(&full_path, IndexAddOption::DEFAULT)
            .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
        
        staged_files.push(path);
    }
    
    // Write the index
    index.write()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
    
    // Emit repository status changed event
    state.window.emit("repository-status-changed", RepositoryStatusEvent {
        staged: staged_files.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    
    Ok(AddToRepositoryResponse {
        staged_files,
        staged_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToRepositoryRequest {
    /// List of file paths to stage
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToRepositoryResponse {
    /// List of successfully staged files
    pub staged_files: Vec<String>,
    
    /// Timestamp when files were staged
    pub staged_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Stages files for commit in the Git repository.
 *
 * @param request - Repository add request containing file paths
 * @returns Promise resolving to the staged files
 * @throws {RepositoryError} When the Git operation fails
 *
 * @example
 * ```typescript
 * const response = await addToRepository({
 *   paths: ['docs/example.md']
 * });
 * ```
 */
export async function addToRepository(
  request: AddToRepositoryRequest
): Promise<AddToRepositoryResponse> {
  return invoke<AddToRepositoryResponse>('add_to_repository', request);
}

export interface AddToRepositoryRequest {
  /** List of file paths to stage */
  paths: string[];
}

export interface AddToRepositoryResponse {
  /** List of successfully staged files */
  staged_files: string[];
  
  /** Timestamp when files were staged (ISO 8601) */
  staged_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "paths": ["docs/example.md", "docs/another.md"]
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "staged_files": ["docs/example.md", "docs/another.md"],
    "staged_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- `paths`: Maximum 100 files per request
- Individual paths: Maximum 255 characters, relative to repository root

#### Performance Requirements

- **Latency:** < 100ms for up to 100 files
- **Throughput:** Support for 50 concurrent add operations

### 4.2. Remove from Repository

**Command ID:** CMD-REP-002
**Command Name:** `remove_from_repository`
**Related Requirements:** REQ-DESK-036 (Repository Initialization)

#### Rust Implementation

```rust
/// Removes files from the Git repository index.
///
/// # Arguments
///
/// * `request` - Repository remove request containing file paths
///
/// # Returns
///
/// `RemoveFromRepositoryResponse` containing the unstaged files
///
/// # Errors
///
/// * `RepositoryError::NotInitialized` - When the repository is not initialized
/// * `RepositoryError::FileNotFound` - When a specified file is not staged
/// * `RepositoryError::GitFailed` - When the Git operation fails
///
/// # Examples
///
/// ```rust
/// let response = remove_from_repository(RemoveFromRepositoryRequest {
///     paths: vec!["docs/example.md".to_string()],
/// }).await?;
/// ```
#[command]
pub async fn remove_from_repository(
    request: RemoveFromRepositoryRequest,
    state: State<'_, AppState>,
) -> Result<RemoveFromRepositoryResponse, RepositoryError> {
    // Open the repository
    let repo = Repository::open(&state.repository_path)
        .map_err(|e| RepositoryError::NotInitialized(e.to_string()))?;
    
    let mut index = repo.index()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
    
    let mut unstaged_files = Vec::new();
    
    for path in request.paths {
        let full_path = state.repository_path.join(&path);
        
        // Remove file from index
        index.remove_path(&full_path, None)
            .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
        
        unstaged_files.push(path);
    }
    
    // Write the index
    index.write()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
    
    // Emit repository status changed event
    state.window.emit("repository-status-changed", RepositoryStatusEvent {
        unstaged: unstaged_files.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    
    Ok(RemoveFromRepositoryResponse {
        unstaged_files,
        unstaged_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFromRepositoryRequest {
    /// List of file paths to unstage
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFromRepositoryResponse {
    /// List of successfully unstaged files
    pub unstaged_files: Vec<String>,
    
    /// Timestamp when files were unstaged
    pub unstaged_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Removes files from the Git repository index.
 *
 * @param request - Repository remove request containing file paths
 * @returns Promise resolving to the unstaged files
 * @throws {RepositoryError} When the Git operation fails
 *
 * @example
 * ```typescript
 * const response = await removeFromRepository({
 *   paths: ['docs/example.md']
 * });
 * ```
 */
export async function removeFromRepository(
  request: RemoveFromRepositoryRequest
): Promise<RemoveFromRepositoryResponse> {
  return invoke<RemoveFromRepositoryResponse>('remove_from_repository', request);
}

export interface RemoveFromRepositoryRequest {
  /** List of file paths to unstage */
  paths: string[];
}

export interface RemoveFromRepositoryResponse {
  /** List of successfully unstaged files */
  unstaged_files: string[];
  
  /** Timestamp when files were unstaged (ISO 8601) */
  unstaged_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "paths": ["docs/example.md"]
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "unstaged_files": ["docs/example.md"],
    "unstaged_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- `paths`: Maximum 100 files per request
- Individual paths: Maximum 255 characters, relative to repository root

#### Performance Requirements

- **Latency:** < 100ms for up to 100 files
- **Throughput:** Support for 50 concurrent remove operations

### 4.3. Sync Repository

**Command ID:** CMD-REP-003
**Command Name:** `sync_repository`
**Related Requirements:** REQ-DESK-037 (Repository Cloning), REQ-DESK-038 (Auto-Commit)

#### Rust Implementation

```rust
use git2::{Direction, Remote, FetchOptions, PushOptions};

/// Synchronizes the local repository with the remote repository.
///
/// # Arguments
///
/// * `request` - Repository sync request
///
/// # Returns
///
/// `SyncRepositoryResponse` containing sync results
///
/// # Errors
///
/// * `RepositoryError::NotInitialized` - When the repository is not initialized
/// * `RepositoryError::NoRemote` - When no remote repository is configured
/// * `RepositoryError::GitFailed` - When the Git operation fails
/// * `RepositoryError::Conflict` - When merge conflicts occur
///
/// # Examples
///
/// ```rust
/// let response = sync_repository(SyncRepositoryRequest {
///     direction: SyncDirection::Pull,
///     remote: Some("origin".to_string()),
/// }).await?;
/// ```
#[command]
pub async fn sync_repository(
    request: SyncRepositoryRequest,
    state: State<'_, AppState>,
) -> Result<SyncRepositoryResponse, RepositoryError> {
    // Open the repository
    let repo = Repository::open(&state.repository_path)
        .map_err(|e| RepositoryError::NotInitialized(e.to_string()))?;
    
    let remote_name = request.remote.unwrap_or_else(|| "origin".to_string());
    
    match request.direction {
        SyncDirection::Pull => {
            // Fetch from remote
            let mut remote = repo.find_remote(&remote_name)
                .map_err(|_| RepositoryError::NoRemote)?;
            
            remote.fetch(&[Some("main")], None, None)
                .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
            
            // Merge changes
            let head = repo.head()
                .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
            
            // Check for conflicts
            if has_merge_conflicts(&repo).await? {
                return Err(RepositoryError::Conflict(
                    "Merge conflicts detected".to_string()
                ));
            }
            
            // Emit sync status changed event
            state.window.emit("sync-status-changed", SyncStatusEvent {
                operation: "pull".to_string(),
                status: "completed".to_string(),
                timestamp: chrono::Utc::now(),
            })?;
            
            Ok(SyncRepositoryResponse {
                direction: SyncDirection::Pull,
                remote: remote_name,
                pulled_at: chrono::Utc::now(),
                conflicts: false,
            })
        }
        SyncDirection::Push => {
            // Push to remote
            let mut remote = repo.find_remote(&remote_name)
                .map_err(|_| RepositoryError::NoRemote)?;
            
            remote.push(&[Some("refs/heads/main")], None)
                .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
            
            // Emit sync status changed event
            state.window.emit("sync-status-changed", SyncStatusEvent {
                operation: "push".to_string(),
                status: "completed".to_string(),
                timestamp: chrono::Utc::now(),
            })?;
            
            Ok(SyncRepositoryResponse {
                direction: SyncDirection::Push,
                remote: remote_name,
                pushed_at: chrono::Utc::now(),
                conflicts: false,
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRepositoryRequest {
    /// Sync direction (pull or push)
    pub direction: SyncDirection,
    
    /// Remote name (default: "origin")
    pub remote: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncDirection {
    Pull,
    Push,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRepositoryResponse {
    /// Sync direction performed
    pub direction: SyncDirection,
    
    /// Remote repository name
    pub remote: String,
    
    /// Timestamp when sync completed (direction-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulled_at: Option<chrono::DateTime<chrono::Utc>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pushed_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Whether conflicts occurred
    pub conflicts: bool,
}
```

#### TypeScript Implementation

```typescript
/**
 * Synchronizes the local repository with the remote repository.
 *
 * @param request - Repository sync request
 * @returns Promise resolving to sync results
 * @throws {RepositoryError} When the Git operation fails
 *
 * @example
 * ```typescript
 * const response = await syncRepository({
 *   direction: 'Pull',
 *   remote: 'origin'
 * });
 * ```
 */
export async function syncRepository(
  request: SyncRepositoryRequest
): Promise<SyncRepositoryResponse> {
  return invoke<SyncRepositoryResponse>('sync_repository', request);
}

export interface SyncRepositoryRequest {
  /** Sync direction (pull or push) */
  direction: 'Pull' | 'Push';
  
  /** Remote name (default: "origin") */
  remote?: string;
}

export interface SyncRepositoryResponse {
  /** Sync direction performed */
  direction: 'Pull' | 'Push';
  
  /** Remote repository name */
  remote: string;
  
  /** Timestamp when pull completed (ISO 8601) */
  pulled_at?: string;
  
  /** Timestamp when push completed (ISO 8601) */
  pushed_at?: string;
  
  /** Whether conflicts occurred */
  conflicts: boolean;
}
```

#### Request/Response Format

**Request:**
```json
{
  "direction": "Pull",
  "remote": "origin"
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "direction": "Pull",
    "remote": "origin",
    "pulled_at": "2026-02-05T16:00:00.000Z",
    "pushed_at": null,
    "conflicts": false
  }
}
```

#### Constraints

- `remote`: Maximum 255 characters
- `direction`: Must be either "Pull" or "Push"

#### Performance Requirements

- **Latency:** < 5 seconds for typical repositories (< 1GB)
- **Throughput:** Support for 10 concurrent sync operations

### 4.4. Get Repository Status

**Command ID:** CMD-REP-004
**Command Name:** `get_repository_status`
**Related Requirements:** REQ-DESK-036 (Repository Initialization), REQ-DESK-040 (History Viewing)

#### Rust Implementation

```rust
/// Retrieves the current status of the Git repository.
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// `GetRepositoryStatusResponse` containing repository status
///
/// # Errors
///
/// * `RepositoryError::NotInitialized` - When the repository is not initialized
/// * `RepositoryError::GitFailed` - When the Git operation fails
///
/// # Examples
///
/// ```rust
/// let response = get_repository_status().await?;
/// ```
#[command]
pub async fn get_repository_status(
    state: State<'_, AppState>,
) -> Result<GetRepositoryStatusResponse, RepositoryError> {
    // Open the repository
    let repo = Repository::open(&state.repository_path)
        .map_err(|e| RepositoryError::NotInitialized(e.to_string()))?;
    
    // Get current branch
    let head = repo.head()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
    let current_branch = head.shorthand()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?
        .to_string();
    
    // Get status
    let status = repo.statuses(None)
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
    
    let mut modified = Vec::new();
    let mut staged = Vec::new();
    let mut untracked = Vec::new();
    
    for entry in status.iter() {
        let path = entry.path()
            .map_err(|e| RepositoryError::GitFailed(e.to_string()))?
            .to_string_lossy()
            .to_string();
        
        match entry.status() {
            Some(s) if s.contains(git2::Status::WT_NEW) => {
                untracked.push(path);
            }
            Some(s) if s.contains(git2::Status::INDEX_MODIFIED) => {
                staged.push(path);
            }
            Some(s) if s.contains(git2::Status::WT_MODIFIED) => {
                modified.push(path);
            }
            _ => {}
        }
    }
    
    // Get remote information
    let remote_url = repo.find_remote("origin")
        .ok()
        .and_then(|r| r.url().ok())
        .map(|u| u.to_string());
    
    Ok(GetRepositoryStatusResponse {
        current_branch,
        modified,
        staged,
        untracked,
        remote_url,
        checked_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRepositoryStatusResponse {
    /// Current branch name
    pub current_branch: String,
    
    /// Modified files
    pub modified: Vec<String>,
    
    /// Staged files
    pub staged: Vec<String>,
    
    /// Untracked files
    pub untracked: Vec<String>,
    
    /// Remote repository URL
    pub remote_url: Option<String>,
    
    /// Timestamp when status was checked
    pub checked_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Retrieves the current status of the Git repository.
 *
 * @returns Promise resolving to repository status
 * @throws {RepositoryError} When the Git operation fails
 *
 * @example
 * ```typescript
 * const response = await getRepositoryStatus();
 * console.log(response.current_branch);
 * ```
 */
export async function getRepositoryStatus(): Promise<GetRepositoryStatusResponse> {
  return invoke<GetRepositoryStatusResponse>('get_repository_status');
}

export interface GetRepositoryStatusResponse {
  /** Current branch name */
  current_branch: string;
  
  /** Modified files */
  modified: string[];
  
  /** Staged files */
  staged: string[];
  
  /** Untracked files */
  untracked: string[];
  
  /** Remote repository URL */
  remote_url?: string;
  
  /** Timestamp when status was checked (ISO 8601) */
  checked_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "current_branch": "main",
    "modified": ["docs/example.md"],
    "staged": ["docs/new.md"],
    "untracked": ["docs/untracked.md"],
    "remote_url": "https://github.com/user/repo.git",
    "checked_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- None

#### Performance Requirements

- **Latency:** < 100ms for repositories with up to 10,000 files
- **Throughput:** Support for 100 concurrent status queries

### 4.5. List Branches

**Command ID:** CMD-REP-005
**Command Name:** `list_branches`
**Related Requirements:** REQ-DESK-039 (Branch Management)

#### Rust Implementation

```rust
/// Lists all branches in the Git repository.
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// `ListBranchesResponse` containing the list of branches
///
/// # Errors
///
/// * `RepositoryError::NotInitialized` - When the repository is not initialized
/// * `RepositoryError::GitFailed` - When the Git operation fails
///
/// # Examples
///
/// ```rust
/// let response = list_branches().await?;
/// ```
#[command]
pub async fn list_branches(
    state: State<'_, AppState>,
) -> Result<ListBranchesResponse, RepositoryError> {
    // Open the repository
    let repo = Repository::open(&state.repository_path)
        .map_err(|e| RepositoryError::NotInitialized(e.to_string()))?;
    
    // Get current branch
    let head = repo.head()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
    let current_branch = head.shorthand()
        .map_err(|e| RepositoryError::GitFailed(e.to_string()))?
        .to_string();
    
    // Get all branches
    let mut branches = Vec::new();
    
    for branch in repo.branches(Some(git2::BranchType::Local))? {
        let name = branch.name()
            .map_err(|e| RepositoryError::GitFailed(e.to_string()))?
            .to_string();
        
        let is_current = name == current_branch;
        
        // Get last commit
        let commit = branch.get().peel_to_commit()
            .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
        
        let commit_time = commit.time()
            .map_err(|e| RepositoryError::GitFailed(e.to_string()))?;
        
        branches.push(BranchInfo {
            name,
            is_current,
            last_commit: commit.id().to_string(),
            last_commit_time: chrono::DateTime::from_timestamp(commit_time.seconds())
                .ok_or_else(|| RepositoryError::GitFailed(
                    "Invalid commit timestamp".to_string()
                ))?,
        });
    }
    
    // Sort branches by name
    branches.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(ListBranchesResponse { branches })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListBranchesResponse {
    /// List of branches
    pub branches: Vec<BranchInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    
    /// Whether this is the current branch
    pub is_current: bool,
    
    /// Last commit hash
    pub last_commit: String,
    
    /// Last commit timestamp
    pub last_commit_time: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Lists all branches in the Git repository.
 *
 * @returns Promise resolving to the list of branches
 * @throws {RepositoryError} When the Git operation fails
 *
 * @example
 * ```typescript
 * const response = await listBranches();
 * console.log(response.branches);
 * ```
 */
export async function listBranches(): Promise<ListBranchesResponse> {
  return invoke<ListBranchesResponse>('list_branches');
}

export interface ListBranchesResponse {
  /** List of branches */
  branches: BranchInfo[];
}

export interface BranchInfo {
  /** Branch name */
  name: string;
  
  /** Whether this is the current branch */
  is_current: boolean;
  
  /** Last commit hash */
  last_commit: string;
  
  /** Last commit timestamp (ISO 8601) */
  last_commit_time: string;
}
```

#### Request/Response Format

**Request:**
```json
{}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "branches": [
      {
        "name": "main",
        "is_current": true,
        "last_commit": "abc123def456",
        "last_commit_time": "2026-02-05T16:00:00.000Z"
      },
      {
        "name": "feature-branch",
        "is_current": false,
        "last_commit": "def456ghi789",
        "last_commit_time": "2026-02-04T15:00:00.000Z"
      }
    ]
  }
}
```

#### Constraints

- None

#### Performance Requirements

- **Latency:** < 100ms for repositories with up to 100 branches
- **Throughput:** Support for 100 concurrent list operations

---

## 5. SEARCH COMMANDS

Search commands provide document search functionality including query, filter, sort, and autocomplete operations. These commands enable the WebView frontend to search and discover documents within the repository efficiently.

### 5.1. Search Documents

**Command ID:** CMD-SRC-001
**Command Name:** `search_documents`
**Related Requirements:** REQ-DESK-020 (Quick Open), REQ-DESK-041 (LRU Cache)

#### Rust Implementation

```rust
use tantivy::{collector::TopDocs, query::QueryParser, Index, IndexWriter, ReloadPolicy};

/// Searches documents in the repository using full-text search.
///
/// # Arguments
///
/// * `request` - Search request containing query and filters
///
/// # Returns
///
/// `SearchDocumentsResponse` containing search results
///
/// # Errors
///
/// * `SearchError::InvalidQuery` - When the search query is invalid
/// * `SearchError::IndexNotReady` - When the search index is not ready
/// * `SearchError::SearchFailed` - When the search operation fails
///
/// # Examples
///
/// ```rust
/// let response = search_documents(SearchDocumentsRequest {
///     query: "example document".to_string(),
///     limit: Some(10),
/// }).await?;
/// ```
#[command]
pub async fn search_documents(
    request: SearchDocumentsRequest,
    state: State<'_, AppState>,
) -> Result<SearchDocumentsResponse, SearchError> {
    // Validate query
    if request.query.is_empty() {
        return Err(SearchError::InvalidQuery(
            "Search query cannot be empty".to_string()
        ));
    }
    
    // Get search index
    let index = state.search_index.get_index()
        .ok_or_else(|| SearchError::IndexNotReady)?;
    
    // Parse query
    let query_parser = QueryParser::for_index(&index);
    let query = query_parser.parse_query(&request.query)
        .map_err(|e| SearchError::InvalidQuery(e.to_string()))?;
    
    // Execute search
    let searcher = index.reader();
    let top_docs = searcher.search(&query, &state.search_index.get_collector())
        .map_err(|e| SearchError::SearchFailed(e.to_string()))?;
    
    // Collect results
    let mut results = Vec::new();
    let limit = request.limit.unwrap_or(10).min(100);
    
    for (score, doc_address) in top_docs.iter().take(limit) {
        let retrieved_doc = index.doc(doc_address)
            .map_err(|e| SearchError::SearchFailed(e.to_string()))?;
        
        let path = retrieved_doc.get_first("path")
            .ok_or_else(|| SearchError::SearchFailed(
                "Document path not found".to_string()
            ))?
            .as_str()
            .map_err(|e| SearchError::SearchFailed(e.to_string()))?
            .to_string();
        
        let title = retrieved_doc.get_first("title")
            .and_then(|v| v.as_str().ok())
            .unwrap_or_else(|| {
                path.split('/').last().unwrap_or("Untitled").to_string()
            });
        
        let snippet = retrieved_doc.get_first("content")
            .and_then(|v| v.as_str().ok())
            .map(|s| {
                let max_len = 200;
                if s.len() > max_len {
                    format!("{}...", &s[..max_len])
                } else {
                    s.to_string()
                }
            });
        
        results.push(SearchResult {
            path,
            title,
            snippet,
            score: *score,
        });
    }
    
    Ok(SearchDocumentsResponse {
        query: request.query,
        results,
        total_count: top_docs.len(),
        searched_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchDocumentsRequest {
    /// Search query string
    pub query: String,
    
    /// Maximum number of results (default: 10, max: 100)
    pub limit: Option<usize>,
    
    /// Path filter to restrict search scope
    pub path_filter: Option<String>,
    
    /// File extension filter
    pub extension_filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchDocumentsResponse {
    /// Original search query
    pub query: String,
    
    /// Search results
    pub results: Vec<SearchResult>,
    
    /// Total number of matching documents
    pub total_count: usize,
    
    /// Timestamp when search was performed
    pub searched_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document path
    pub path: String,
    
    /// Document title
    pub title: String,
    
    /// Content snippet with highlighted matches
    pub snippet: Option<String>,
    
    /// Relevance score
    pub score: f32,
}
```

#### TypeScript Implementation

```typescript
/**
 * Searches documents in the repository using full-text search.
 *
 * @param request - Search request containing query and filters
 * @returns Promise resolving to search results
 * @throws {SearchError} When the search operation fails
 *
 * @example
 * ```typescript
 * const response = await searchDocuments({
 *   query: 'example document',
 *   limit: 10
 * });
 * console.log(response.results);
 * ```
 */
export async function searchDocuments(
  request: SearchDocumentsRequest
): Promise<SearchDocumentsResponse> {
  return invoke<SearchDocumentsResponse>('search_documents', request);
}

export interface SearchDocumentsRequest {
  /** Search query string */
  query: string;
  
  /** Maximum number of results (default: 10, max: 100) */
  limit?: number;
  
  /** Path filter to restrict search scope */
  path_filter?: string;
  
  /** File extension filter */
  extension_filter?: string;
}

export interface SearchDocumentsResponse {
  /** Original search query */
  query: string;
  
  /** Search results */
  results: SearchResult[];
  
  /** Total number of matching documents */
  total_count: number;
  
  /** Timestamp when search was performed (ISO 8601) */
  searched_at: string;
}

export interface SearchResult {
  /** Document path */
  path: string;
  
  /** Document title */
  title: string;
  
  /** Content snippet with highlighted matches */
  snippet?: string;
  
  /** Relevance score */
  score: number;
}
```

#### Request/Response Format

**Request:**
```json
{
  "query": "example document",
  "limit": 10,
  "path_filter": "docs",
  "extension_filter": "md"
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "query": "example document",
    "results": [
      {
        "path": "docs/example.md",
        "title": "Example Document",
        "snippet": "This is an example document...",
        "score": 0.95
      }
    ],
    "total_count": 1,
    "searched_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- `query`: Minimum 2 characters, maximum 500 characters
- `limit`: Minimum 1, maximum 100
- `path_filter`: Maximum 255 characters, relative to repository root
- `extension_filter`: Maximum 10 characters

#### Performance Requirements

- **Latency:** < 100ms for queries with up to 10,000 indexed documents
- **Throughput:** Support for 100 concurrent search operations

### 5.2. Filter Documents

**Command ID:** CMD-SRC-002
**Command Name:** `filter_documents`
**Related Requirements:** REQ-DESK-016 (Sidebar Navigation)

#### Rust Implementation

```rust
/// Filters documents based on specified criteria.
///
/// # Arguments
///
/// * `request` - Filter request containing filter criteria
///
/// # Returns
///
/// `FilterDocumentsResponse` containing filtered documents
///
/// # Errors
///
/// * `DocumentError::InvalidFilter` - When the filter criteria is invalid
/// * `DocumentError::ReadFailed` - When reading documents fails
///
/// # Examples
///
/// ```rust
/// let response = filter_documents(FilterDocumentsRequest {
///     path: Some("docs".to_string()),
///     extension: Some("md".to_string()),
///     modified_after: Some(chrono::Utc::now() - chrono::Duration::days(7)),
/// }).await?;
/// ```
#[command]
pub async fn filter_documents(
    request: FilterDocumentsRequest,
    state: State<'_, AppState>,
) -> Result<FilterDocumentsResponse, DocumentError> {
    let base_path = request.path.unwrap_or_else(|| ".".to_string());
    let validated_path = validate_document_path(&base_path)?;
    let full_path = state.repository_path.join(&validated_path);
    
    let mut documents = Vec::new();
    
    // Walk the directory tree
    let mut entries = tokio::fs::read_dir(&full_path).await
        .map_err(|e| DocumentError::ReadFailed(e.to_string()))?;
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| DocumentError::ReadFailed(e.to_string()))?
    {
        let path = entry.path();
        let metadata = entry.metadata().await
            .map_err(|e| DocumentError::ReadFailed(e.to_string()))?;
        
        if metadata.is_file() {
            // Apply filters
            if let Some(ref ext) = request.extension {
                if path.extension().and_then(|s| s.to_str()) != Some(ext.as_str()) {
                    continue;
                }
            }
            
            if let Some(ref after) = request.modified_after {
                let modified = metadata.modified()
                    .ok_or_else(|| DocumentError::ReadFailed(
                        "Failed to get modification time".to_string()
                    ))?;
                if modified < *after {
                    continue;
                }
            }
            
            if let Some(ref before) = request.modified_before {
                let modified = metadata.modified()
                    .ok_or_else(|| DocumentError::ReadFailed(
                        "Failed to get modification time".to_string()
                    ))?;
                if modified > *before {
                    continue;
                }
            }
            
            if let Some(ref min_size) = request.min_size {
                if metadata.len() < *min_size as u64 {
                    continue;
                }
            }
            
            if let Some(ref max_size) = request.max_size {
                if metadata.len() > *max_size as u64 {
                    continue;
                }
            }
            
            // Get relative path
            let relative_path = path.strip_prefix(&state.repository_path)
                .map_err(|_| DocumentError::InvalidPath(
                    "Failed to get relative path".to_string()
                ))?
                .to_string_lossy()
                .to_string();
            
            documents.push(DocumentInfo {
                path: relative_path,
                size: metadata.len() as usize,
                modified: metadata.modified()
                    .ok_or_else(|| DocumentError::ReadFailed(
                        "Failed to get modification time".to_string()
                    ))?,
                created: metadata.created()
                    .ok_or_else(|| DocumentError::ReadFailed(
                        "Failed to get creation time".to_string()
                    ))?,
            });
        } else if metadata.is_dir() && request.recursive.unwrap_or(false) {
            // Recursively process directories
            let sub_request = FilterDocumentsRequest {
                path: Some(path.strip_prefix(&state.repository_path)
                    .map_err(|_| DocumentError::InvalidPath(
                        "Failed to get relative path".to_string()
                    ))?
                    .to_string_lossy()
                    .to_string()),
                extension: request.extension.clone(),
                modified_after: request.modified_after,
                modified_before: request.modified_before,
                min_size: request.min_size,
                max_size: request.max_size,
                recursive: Some(true),
            };
            let sub_response = filter_documents(sub_request, state).await?;
            documents.extend(sub_response.documents);
        }
    }
    
    Ok(FilterDocumentsResponse { documents })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterDocumentsRequest {
    /// Base path to filter documents from (default: repository root)
    pub path: Option<String>,
    
    /// File extension filter (e.g., "md")
    pub extension: Option<String>,
    
    /// Filter documents modified after this timestamp
    pub modified_after: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Filter documents modified before this timestamp
    pub modified_before: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Minimum file size in bytes
    pub min_size: Option<usize>,
    
    /// Maximum file size in bytes
    pub max_size: Option<usize>,
    
    /// Whether to recursively filter subdirectories (default: false)
    pub recursive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterDocumentsResponse {
    /// List of documents matching the filters
    pub documents: Vec<DocumentInfo>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Filters documents based on specified criteria.
 *
 * @param request - Filter request containing filter criteria
 * @returns Promise resolving to filtered documents
 * @throws {DocumentError} When filtering fails
 *
 * @example
 * ```typescript
 * const response = await filterDocuments({
 *   path: 'docs',
 *   extension: 'md',
 *   modified_after: '2026-01-29T16:00:00.000Z',
 *   recursive: true
 * });
 * ```
 */
export async function filterDocuments(
  request: FilterDocumentsRequest
): Promise<FilterDocumentsResponse> {
  return invoke<FilterDocumentsResponse>('filter_documents', request);
}

export interface FilterDocumentsRequest {
  /** Base path to filter documents from (default: repository root) */
  path?: string;
  
  /** File extension filter (e.g., "md") */
  extension?: string;
  
  /** Filter documents modified after this timestamp (ISO 8601) */
  modified_after?: string;
  
  /** Filter documents modified before this timestamp (ISO 8601) */
  modified_before?: string;
  
  /** Minimum file size in bytes */
  min_size?: number;
  
  /** Maximum file size in bytes */
  max_size?: number;
  
  /** Whether to recursively filter subdirectories (default: false) */
  recursive?: boolean;
}

export interface FilterDocumentsResponse {
  /** List of documents matching the filters */
  documents: DocumentInfo[];
}
```

#### Request/Response Format

**Request:**
```json
{
  "path": "docs",
  "extension": "md",
  "modified_after": "2026-01-29T16:00:00.000Z",
  "min_size": 100,
  "recursive": true
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "documents": [
      {
        "path": "docs/example.md",
        "size": 1024,
        "modified": "2026-02-05T16:00:00.000Z",
        "created": "2026-02-01T16:00:00.000Z"
      }
    ]
  }
}
```

#### Constraints

- `path`: Maximum 255 characters
- `extension`: Maximum 10 characters
- `min_size`: Minimum 0 bytes
- `max_size`: Maximum 10MB (10,485,760 bytes)

#### Performance Requirements

- **Latency:** < 200ms for repositories with up to 10,000 documents
- **Throughput:** Support for 50 concurrent filter operations

### 5.3. Sort Documents

**Command ID:** CMD-SRC-003
**Command Name:** `sort_documents`
**Related Requirements:** REQ-DESK-016 (Sidebar Navigation)

#### Rust Implementation

```rust
/// Sorts documents based on specified criteria.
///
/// # Arguments
///
/// * `request` - Sort request containing sort criteria
///
/// # Returns
///
/// `SortDocumentsResponse` containing sorted documents
///
/// # Errors
///
/// * `DocumentError::InvalidSort` - When the sort criteria is invalid
///
/// # Examples
///
/// ```rust
/// let response = sort_documents(SortDocumentsRequest {
///     documents: vec![/* ... */],
///     sort_by: SortBy::Modified,
///     order: SortOrder::Descending,
/// }).await?;
/// ```
#[command]
pub async fn sort_documents(
    request: SortDocumentsRequest,
    _state: State<'_, AppState>,
) -> Result<SortDocumentsResponse, DocumentError> {
    let mut documents = request.documents;
    
    match request.sort_by {
        SortBy::Name => {
            documents.sort_by(|a, b| a.path.cmp(&b.path));
        }
        SortBy::Size => {
            documents.sort_by(|a, b| a.size.cmp(&b.size));
        }
        SortBy::Modified => {
            documents.sort_by(|a, b| a.modified.cmp(&b.modified));
        }
        SortBy::Created => {
            documents.sort_by(|a, b| a.created.cmp(&b.created));
        }
    }
    
    // Apply sort order
    if request.order == SortOrder::Descending {
        documents.reverse();
    }
    
    Ok(SortDocumentsResponse {
        documents,
        sorted_by: request.sort_by,
        order: request.order,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SortDocumentsRequest {
    /// List of documents to sort
    pub documents: Vec<DocumentInfo>,
    
    /// Sort criteria
    pub sort_by: SortBy,
    
    /// Sort order (default: Ascending)
    pub order: SortOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Created,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SortDocumentsResponse {
    /// Sorted list of documents
    pub documents: Vec<DocumentInfo>,
    
    /// Sort criteria used
    pub sorted_by: SortBy,
    
    /// Sort order used
    pub order: SortOrder,
}
```

#### TypeScript Implementation

```typescript
/**
 * Sorts documents based on specified criteria.
 *
 * @param request - Sort request containing sort criteria
 * @returns Promise resolving to sorted documents
 * @throws {DocumentError} When sorting fails
 *
 * @example
 * ```typescript
 * const response = await sortDocuments({
 *   documents: [/* ... */],
 *   sort_by: 'Modified',
 *   order: 'Descending'
 * });
 * ```
 */
export async function sortDocuments(
  request: SortDocumentsRequest
): Promise<SortDocumentsResponse> {
  return invoke<SortDocumentsResponse>('sort_documents', request);
}

export interface SortDocumentsRequest {
  /** List of documents to sort */
  documents: DocumentInfo[];
  
  /** Sort criteria */
  sort_by: 'Name' | 'Size' | 'Modified' | 'Created';
  
  /** Sort order (default: Ascending) */
  order: 'Ascending' | 'Descending';
}

export interface SortDocumentsResponse {
  /** Sorted list of documents */
  documents: DocumentInfo[];
  
  /** Sort criteria used */
  sorted_by: 'Name' | 'Size' | 'Modified' | 'Created';
  
  /** Sort order used */
  order: 'Ascending' | 'Descending';
}
```

#### Request/Response Format

**Request:**
```json
{
  "documents": [
    {
      "path": "docs/example.md",
      "size": 1024,
      "modified": "2026-02-05T16:00:00.000Z",
      "created": "2026-02-01T16:00:00.000Z"
    }
  ],
  "sort_by": "Modified",
  "order": "Descending"
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "documents": [
      {
        "path": "docs/example.md",
        "size": 1024,
        "modified": "2026-02-05T16:00:00.000Z",
        "created": "2026-02-01T16:00:00.000Z"
      }
    ],
    "sorted_by": "Modified",
    "order": "Descending"
  }
}
```

#### Constraints

- `documents`: Maximum 10,000 documents per request

#### Performance Requirements

- **Latency:** < 50ms for up to 10,000 documents
- **Throughput:** Support for 100 concurrent sort operations

### 5.4. Autocomplete Documents

**Command ID:** CMD-SRC-004
**Command Name:** `autocomplete_documents`
**Related Requirements:** REQ-DESK-020 (Quick Open), REQ-DESK-024 (Auto-Complete)

#### Rust Implementation

```rust
/// Provides autocomplete suggestions for document paths.
///
/// # Arguments
///
/// * `request` - Autocomplete request containing partial path
///
/// # Returns
///
/// `AutocompleteDocumentsResponse` containing suggestions
///
/// # Errors
///
/// * `SearchError::InvalidQuery` - When the query is invalid
///
/// # Examples
///
/// ```rust
/// let response = autocomplete_documents(AutocompleteDocumentsRequest {
///     query: "doc".to_string(),
///     limit: Some(10),
/// }).await?;
/// ```
#[command]
pub async fn autocomplete_documents(
    request: AutocompleteDocumentsRequest,
    state: State<'_, AppState>,
) -> Result<AutocompleteDocumentsResponse, SearchError> {
    // Validate query
    if request.query.is_empty() {
        return Err(SearchError::InvalidQuery(
            "Autocomplete query cannot be empty".to_string()
        ));
    }
    
    // Get search index
    let index = state.search_index.get_index()
        .ok_or_else(|| SearchError::IndexNotReady)?;
    
    // Create fuzzy query
    let query_parser = QueryParser::for_index(&index);
    let fuzzy_query = format!("{}*", request.query);
    let query = query_parser.parse_query(&fuzzy_query)
        .map_err(|e| SearchError::InvalidQuery(e.to_string()))?;
    
    // Execute search
    let searcher = index.reader();
    let top_docs = searcher.search(&query, &state.search_index.get_collector())
        .map_err(|e| SearchError::SearchFailed(e.to_string()))?;
    
    // Collect suggestions
    let mut suggestions = Vec::new();
    let limit = request.limit.unwrap_or(10).min(20);
    
    for (score, doc_address) in top_docs.iter().take(limit) {
        let retrieved_doc = index.doc(doc_address)
            .map_err(|e| SearchError::SearchFailed(e.to_string()))?;
        
        let path = retrieved_doc.get_first("path")
            .ok_or_else(|| SearchError::SearchFailed(
                "Document path not found".to_string()
            ))?
            .as_str()
            .map_err(|e| SearchError::SearchFailed(e.to_string()))?
            .to_string();
        
        suggestions.push(AutocompleteSuggestion {
            path,
            score: *score,
        });
    }
    
    Ok(AutocompleteDocumentsResponse {
        query: request.query,
        suggestions,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutocompleteDocumentsRequest {
    /// Partial query for autocomplete
    pub query: String,
    
    /// Maximum number of suggestions (default: 10, max: 20)
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutocompleteDocumentsResponse {
    /// Original query
    pub query: String,
    
    /// Autocomplete suggestions
    pub suggestions: Vec<AutocompleteSuggestion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutocompleteSuggestion {
    /// Document path
    pub path: String,
    
    /// Relevance score
    pub score: f32,
}
```

#### TypeScript Implementation

```typescript
/**
 * Provides autocomplete suggestions for document paths.
 *
 * @param request - Autocomplete request containing partial path
 * @returns Promise resolving to suggestions
 * @throws {SearchError} When autocomplete fails
 *
 * @example
 * ```typescript
 * const response = await autocompleteDocuments({
 *   query: 'doc',
 *   limit: 10
 * });
 * console.log(response.suggestions);
 * ```
 */
export async function autocompleteDocuments(
  request: AutocompleteDocumentsRequest
): Promise<AutocompleteDocumentsResponse> {
  return invoke<AutocompleteDocumentsResponse>('autocomplete_documents', request);
}

export interface AutocompleteDocumentsRequest {
  /** Partial query for autocomplete */
  query: string;
  
  /** Maximum number of suggestions (default: 10, max: 20) */
  limit?: number;
}

export interface AutocompleteDocumentsResponse {
  /** Original query */
  query: string;
  
  /** Autocomplete suggestions */
  suggestions: AutocompleteSuggestion[];
}

export interface AutocompleteSuggestion {
  /** Document path */
  path: string;
  
  /** Relevance score */
  score: number;
}
```

#### Request/Response Format

**Request:**
```json
{
  "query": "doc",
  "limit": 10
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "query": "doc",
    "suggestions": [
      {
        "path": "docs/document.md",
        "score": 0.95
      },
      {
        "path": "docs/documentation.md",
        "score": 0.90
      }
    ]
  }
}
```

#### Constraints

- `query`: Minimum 1 character, maximum 100 characters
- `limit`: Minimum 1, maximum 20

#### Performance Requirements

- **Latency:** < 50ms for queries with up to 10,000 indexed documents
- **Throughput:** Support for 200 concurrent autocomplete operations

---

## 6. SYSTEM COMMANDS

System commands provide application-level operations including initialization, configuration management, status queries, and termination. These commands enable the WebView frontend to interact with the desktop application's system-level functionality.

### 6.1. Initialize Application

**Command ID:** CMD-SYS-001
**Command Name:** `initialize_application`
**Related Requirements:** REQ-DESK-001 (Application Startup), REQ-DESK-046 (Server Spawn)

#### Rust Implementation

```rust
/// Initializes the desktop application and its components.
///
/// # Arguments
///
/// * `request` - Initialization request containing configuration
///
/// # Returns
///
/// `InitializeApplicationResponse` containing initialization status
///
/// # Errors
///
/// * `SystemError::AlreadyInitialized` - When the application is already initialized
/// * `SystemError::InitializationFailed` - When initialization fails
///
/// # Examples
///
/// ```rust
/// let response = initialize_application(InitializeApplicationRequest {
///     repository_path: Some("/path/to/repo".to_string()),
///     config: Some(config),
/// }).await?;
/// ```
#[command]
pub async fn initialize_application(
    request: InitializeApplicationRequest,
    mut state: State<'_, AppState>,
) -> Result<InitializeApplicationResponse, SystemError> {
    // Check if already initialized
    if state.initialized.load(std::sync::Ordering::Acquire) {
        return Err(SystemError::AlreadyInitialized(
            "Application is already initialized".to_string()
        ));
    }
    
    // Set repository path if provided
    if let Some(ref repo_path) = request.repository_path {
        state.repository_path = std::path::PathBuf::from(repo_path);
    }
    
    // Initialize local server
    let server_port = spawn_local_server(&mut state).await?;
    
    // Initialize file watcher
    let file_watcher = spawn_file_watcher(&state).await?;
    state.file_watcher = Some(file_watcher);
    
    // Initialize search index
    state.search_index.initialize(&state.repository_path).await?;
    
    // Mark as initialized
    state.initialized.store(true, std::sync::Ordering::Release);
    
    // Emit initialization complete event
    state.window.emit("application-initialized", InitializationEvent {
        server_port,
        repository_path: state.repository_path.to_string_lossy().to_string(),
        timestamp: chrono::Utc::now(),
    })?;
    
    Ok(InitializeApplicationResponse {
        initialized: true,
        server_port,
        repository_path: state.repository_path.to_string_lossy().to_string(),
        initialized_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeApplicationRequest {
    /// Repository path (optional, uses default if not provided)
    pub repository_path: Option<String>,
    
    /// Configuration override (optional)
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeApplicationResponse {
    /// Whether initialization was successful
    pub initialized: bool,
    
    /// Local server port
    pub server_port: u16,
    
    /// Repository path
    pub repository_path: String,
    
    /// Timestamp when initialization completed
    pub initialized_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Initializes the desktop application and its components.
 *
 * @param request - Initialization request containing configuration
 * @returns Promise resolving to initialization status
 * @throws {SystemError} When initialization fails
 *
 * @example
 * ```typescript
 * const response = await initializeApplication({
 *   repository_path: '/path/to/repo',
 *   config: { /* ... */ }
 * });
 * ```
 */
export async function initializeApplication(
  request: InitializeApplicationRequest
): Promise<InitializeApplicationResponse> {
  return invoke<InitializeApplicationResponse>('initialize_application', request);
}

export interface InitializeApplicationRequest {
  /** Repository path (optional, uses default if not provided) */
  repository_path?: string;
  
  /** Configuration override (optional) */
  config?: Record<string, unknown>;
}

export interface InitializeApplicationResponse {
  /** Whether initialization was successful */
  initialized: boolean;
  
  /** Local server port */
  server_port: number;
  
  /** Repository path */
  repository_path: string;
  
  /** Timestamp when initialization completed (ISO 8601) */
  initialized_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "repository_path": "/path/to/repo",
  "config": {
    "auto_save_enabled": true,
    "auto_save_interval_seconds": 2
  }
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "initialized": true,
    "server_port": 54321,
    "repository_path": "/path/to/repo",
    "initialized_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- `repository_path`: Maximum 4096 characters
- `config`: Maximum 10KB (10,240 bytes)

#### Performance Requirements

- **Latency:** < 3 seconds for complete initialization
- **Throughput:** N/A (one-time operation)

### 6.2. Get Configuration

**Command ID:** CMD-SYS-002
**Command Name:** `get_config`
**Related Requirements:** REQ-DESK-026 (State Management)

#### Rust Implementation

```rust
/// Retrieves the current application configuration.
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// `GetConfigResponse` containing current configuration
///
/// # Errors
///
/// * `SystemError::NotInitialized` - When the application is not initialized
///
/// # Examples
///
/// ```rust
/// let response = get_config().await?;
/// ```
#[command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<GetConfigResponse, SystemError> {
    // Check if initialized
    if !state.initialized.load(std::sync::Ordering::Acquire) {
        return Err(SystemError::NotInitialized(
            "Application is not initialized".to_string()
        ));
    }
    
    // Get configuration from config manager
    let config = state.config.get_all().await;
    
    Ok(GetConfigResponse {
        config,
        retrieved_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetConfigResponse {
    /// Current configuration
    pub config: serde_json::Value,
    
    /// Timestamp when configuration was retrieved
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Retrieves the current application configuration.
 *
 * @returns Promise resolving to current configuration
 * @throws {SystemError} When configuration retrieval fails
 *
 * @example
 * ```typescript
 * const response = await getConfig();
 * console.log(response.config);
 * ```
 */
export async function getConfig(): Promise<GetConfigResponse> {
  return invoke<GetConfigResponse>('get_config');
}

export interface GetConfigResponse {
  /** Current configuration */
  config: Record<string, unknown>;
  
  /** Timestamp when configuration was retrieved (ISO 8601) */
  retrieved_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "config": {
      "auto_save_enabled": true,
      "auto_save_interval_seconds": 2,
      "editor_font_size": 14,
      "editor_theme": "dark"
    },
    "retrieved_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- None

#### Performance Requirements

- **Latency:** < 50ms
- **Throughput:** Support for 100 concurrent config queries

### 6.3. Set Configuration

**Command ID:** CMD-SYS-003
**Command Name:** `set_config`
**Related Requirements:** REQ-DESK-026 (State Management)

#### Rust Implementation

```rust
/// Updates application configuration.
///
/// # Arguments
///
/// * `request` - Configuration update request
///
/// # Returns
///
/// `SetConfigResponse` confirming the update
///
/// # Errors
///
/// * `SystemError::NotInitialized` - When the application is not initialized
/// * `SystemError::InvalidConfig` - When the configuration is invalid
///
/// # Examples
///
/// ```rust
/// let response = set_config(SetConfigRequest {
///     key: "auto_save_enabled".to_string(),
///     value: serde_json::json!(true),
/// }).await?;
/// ```
#[command]
pub async fn set_config(
    request: SetConfigRequest,
    state: State<'_, AppState>,
) -> Result<SetConfigResponse, SystemError> {
    // Check if initialized
    if !state.initialized.load(std::sync::Ordering::Acquire) {
        return Err(SystemError::NotInitialized(
            "Application is not initialized".to_string()
        ));
    }
    
    // Validate configuration key
    validate_config_key(&request.key)?;
    
    // Set configuration value
    state.config.set(&request.key, request.value).await;
    
    // Emit configuration changed event
    state.window.emit("config-changed", ConfigChangedEvent {
        key: request.key.clone(),
        value: request.value.clone(),
        timestamp: chrono::Utc::now(),
    })?;
    
    Ok(SetConfigResponse {
        key: request.key,
        updated_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetConfigRequest {
    /// Configuration key
    pub key: String,
    
    /// Configuration value
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetConfigResponse {
    /// Configuration key that was updated
    pub key: String,
    
    /// Timestamp when configuration was updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Updates application configuration.
 *
 * @param request - Configuration update request
 * @returns Promise resolving to confirmation of update
 * @throws {SystemError} When configuration update fails
 *
 * @example
 * ```typescript
 * const response = await setConfig({
 *   key: 'auto_save_enabled',
 *   value: true
 * });
 * ```
 */
export async function setConfig(
  request: SetConfigRequest
): Promise<SetConfigResponse> {
  return invoke<SetConfigResponse>('set_config', request);
}

export interface SetConfigRequest {
  /** Configuration key */
  key: string;
  
  /** Configuration value */
  value: unknown;
}

export interface SetConfigResponse {
  /** Configuration key that was updated */
  key: string;
  
  /** Timestamp when configuration was updated (ISO 8601) */
  updated_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "key": "auto_save_enabled",
  "value": true
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "key": "auto_save_enabled",
    "updated_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- `key`: Maximum 100 characters
- `value`: Maximum 10KB (10,240 bytes)

#### Performance Requirements

- **Latency:** < 100ms
- **Throughput:** Support for 50 concurrent config updates

### 6.4. Get Application Status

**Command ID:** CMD-SYS-004
**Command Name:** `get_application_status`
**Related Requirements:** REQ-DESK-047 (Server Health Monitoring), REQ-DESK-030 (Status Bar)

#### Rust Implementation

```rust
/// Retrieves the current application status.
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// `GetApplicationStatusResponse` containing application status
///
/// # Errors
///
/// * `SystemError::NotInitialized` - When the application is not initialized
///
/// # Examples
///
/// ```rust
/// let response = get_application_status().await?;
/// ```
#[command]
pub async fn get_application_status(
    state: State<'_, AppState>,
) -> Result<GetApplicationStatusResponse, SystemError> {
    // Check if initialized
    let initialized = state.initialized.load(std::sync::Ordering::Acquire);
    if !initialized {
        return Err(SystemError::NotInitialized(
            "Application is not initialized".to_string()
        ));
    }
    
    // Get server status
    let server_status = state.local_server.as_ref()
        .map(|s| s.get_status())
        .unwrap_or(ServerStatus::Stopped);
    
    // Get repository status
    let repo_status = get_repository_status_internal(&state).await?;
    
    // Get cache statistics
    let cache_stats = state.cache.get_statistics().await;
    
    Ok(GetApplicationStatusResponse {
        initialized,
        server_status,
        repository_status: repo_status,
        cache_statistics: cache_stats,
        checked_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetApplicationStatusResponse {
    /// Whether the application is initialized
    pub initialized: bool,
    
    /// Local server status
    pub server_status: ServerStatus,
    
    /// Repository status
    pub repository_status: RepositoryStatus,
    
    /// Cache statistics
    pub cache_statistics: CacheStatistics,
    
    /// Timestamp when status was checked
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerStatus {
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Current branch
    pub current_branch: String,
    
    /// Remote URL
    pub remote_url: Option<String>,
    
    /// Number of modified files
    pub modified_count: usize,
    
    /// Number of staged files
    pub staged_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Cache size in bytes
    pub size_bytes: usize,
    
    /// Number of cached entries
    pub entry_count: usize,
    
    /// Cache hit rate
    pub hit_rate: f32,
}
```

#### TypeScript Implementation

```typescript
/**
 * Retrieves the current application status.
 *
 * @returns Promise resolving to application status
 * @throws {SystemError} When status retrieval fails
 *
 * @example
 * ```typescript
 * const response = await getApplicationStatus();
 * console.log(response.server_status);
 * ```
 */
export async function getApplicationStatus(): Promise<GetApplicationStatusResponse> {
  return invoke<GetApplicationStatusResponse>('get_application_status');
}

export interface GetApplicationStatusResponse {
  /** Whether the application is initialized */
  initialized: boolean;
  
  /** Local server status */
  server_status: ServerStatus;
  
  /** Repository status */
  repository_status: RepositoryStatus;
  
  /** Cache statistics */
  cache_statistics: CacheStatistics;
  
  /** Timestamp when status was checked (ISO 8601) */
  checked_at: string;
}

export type ServerStatus = 
  | { type: 'Running' }
  | { type: 'Stopped' }
  | { type: 'Error'; message: string };

export interface RepositoryStatus {
  /** Current branch */
  current_branch: string;
  
  /** Remote URL */
  remote_url?: string;
  
  /** Number of modified files */
  modified_count: number;
  
  /** Number of staged files */
  staged_count: number;
}

export interface CacheStatistics {
  /** Cache size in bytes */
  size_bytes: number;
  
  /** Number of cached entries */
  entry_count: number;
  
  /** Cache hit rate */
  hit_rate: number;
}
```

#### Request/Response Format

**Request:**
```json
{}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "initialized": true,
    "server_status": {
      "type": "Running"
    },
    "repository_status": {
      "current_branch": "main",
      "remote_url": "https://github.com/user/repo.git",
      "modified_count": 1,
      "staged_count": 0
    },
    "cache_statistics": {
      "size_bytes": 524288000,
      "entry_count": 100,
      "hit_rate": 0.85
    },
    "checked_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- None

#### Performance Requirements

- **Latency:** < 100ms
- **Throughput:** Support for 100 concurrent status queries

### 6.5. Quit Application

**Command ID:** CMD-SYS-005
**Command Name:** `quit_application`
**Related Requirements:** REQ-DESK-006 (Graceful Shutdown)

#### Rust Implementation

```rust
/// Gracefully shuts down the desktop application.
///
/// # Arguments
///
/// * `request` - Quit request containing shutdown options
///
/// # Returns
///
/// `QuitApplicationResponse` confirming the shutdown
///
/// # Errors
///
/// * `SystemError::ShutdownFailed` - When the shutdown fails
///
/// # Examples
///
/// ```rust
/// let response = quit_application(QuitApplicationRequest {
///     save_changes: true,
/// }).await?;
/// ```
#[command]
pub async fn quit_application(
    request: QuitApplicationRequest,
    mut state: State<'_, AppState>,
) -> Result<QuitApplicationResponse, SystemError> {
    // Save unsaved changes if requested
    if request.save_changes {
        save_all_unsaved_changes(&mut state).await?;
    }
    
    // Stop file watcher
    if let Some(file_watcher) = state.file_watcher.take() {
        file_watcher.stop().await;
    }
    
    // Stop local server
    if let Some(local_server) = state.local_server.take() {
        local_server.stop().await;
    }
    
    // Persist cache
    state.cache.persist().await;
    
    // Emit quit event
    state.window.emit("application-quitting", QuitEvent {
        timestamp: chrono::Utc::now(),
    })?;
    
    // Mark as uninitialized
    state.initialized.store(false, std::sync::Ordering::Release);
    
    // Schedule window close
    tauri::api::process::exit(0);
    
    Ok(QuitApplicationResponse {
        quit_at: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuitApplicationRequest {
    /// Whether to save unsaved changes before quitting
    pub save_changes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuitApplicationResponse {
    /// Timestamp when application quit
    pub quit_at: chrono::DateTime<chrono::Utc>,
}
```

#### TypeScript Implementation

```typescript
/**
 * Gracefully shuts down the desktop application.
 *
 * @param request - Quit request containing shutdown options
 * @returns Promise resolving to confirmation of shutdown
 * @throws {SystemError} When shutdown fails
 *
 * @example
 * ```typescript
 * const response = await quitApplication({
 *   save_changes: true
 * });
 * ```
 */
export async function quitApplication(
  request: QuitApplicationRequest
): Promise<QuitApplicationResponse> {
  return invoke<QuitApplicationResponse>('quit_application', request);
}

export interface QuitApplicationRequest {
  /** Whether to save unsaved changes before quitting */
  save_changes: boolean;
}

export interface QuitApplicationResponse {
  /** Timestamp when application quit (ISO 8601) */
  quit_at: string;
}
```

#### Request/Response Format

**Request:**
```json
{
  "save_changes": true
}
```

**Success Response:**
```json
{
  "success": true,
  "data": {
    "quit_at": "2026-02-05T16:00:00.000Z"
  }
}
```

#### Constraints

- None

#### Performance Requirements

- **Latency:** < 5 seconds for complete shutdown (including save operations)
- **Throughput:** N/A (one-time operation)

---

## 7. COMMAND SECURITY

Command security defines authentication, authorization, and input validation requirements for all Desktop Commands API operations. These requirements ensure that the Desktop Commands API operates securely and protects against common attack vectors.

### 7.1. Authentication Requirements

**Requirement ID:** REQ-SEC-001
**Related Requirements:** REQ-DESK-052 (WebView Security), REQ-DESK-080 (Link Validation)

#### 7.1.1. Session-Based Authentication

**Standard:** All IPC commands must require a valid session token for authentication.

**Implementation:**

- Session tokens are generated upon application initialization
- Session tokens are stored in secure memory with limited lifetime
- Session tokens are validated on each command invocation
- Session tokens are invalidated on application shutdown

**Rationale:** Session-based authentication provides fine-grained control over command execution and enables revocation of compromised sessions.

#### 7.1.2. Session Token Format

**Standard:** Session tokens must follow a cryptographically secure format.

**Format:**

```
<version>:<algorithm>:<base64-encoded-random-bytes>:<signature>
```

**Components:**

- `version`: Token format version (e.g., "v1")
- `algorithm`: Cryptographic algorithm used (e.g., "HS256")
- `base64-encoded-random-bytes`: 256 bits of cryptographically secure random data
- `signature`: HMAC signature of the token components

**Rationale:** Cryptographically secure tokens prevent token forgery and replay attacks.

#### 7.1.3. Session Lifetime

**Standard:** Session tokens must have a limited lifetime and automatic expiration.

**Constraints:**

- Maximum session lifetime: 24 hours
- Session inactivity timeout: 30 minutes
- Automatic session renewal: Sessions are renewed if activity continues within timeout window

**Rationale:** Limited session lifetime reduces the window of opportunity for token compromise and enforces regular re-authentication.

### 7.2. Authorization Requirements

**Requirement ID:** REQ-SEC-002
**Related Requirements:** REQ-DESK-052 (WebView Security), REQ-DESK-034 (File Locking)

#### 7.2.1. Capability-Based Authorization

**Standard:** All IPC commands must enforce capability-based authorization following the principle of least privilege.

**Implementation:**

- Each command declares required capabilities in Tauri capability configuration
- Commands validate that the session has required capabilities before execution
- Capabilities are granted based on user roles and permissions
- Capability revocation is supported for compromised sessions

**Capability Categories:**

| Category | Capabilities | Purpose |
|----------|--------------|---------|
| **File System** | `fs:read`, `fs:write`, `fs:scope` | Controlled file access |
| **Window** | `window:allow-create`, `window:allow-close` | Window management |
| **Shell** | `shell:allow-execute`, `shell:allow-open` | Command execution |
| **Dialog** | `dialog:allow-open`, `dialog:allow-save` | Native dialogs |
| **HTTP** | `http:allow-request`, `http:allow-fetch` | Network requests |
| **Notification** | `notification:allow-send` | System notifications |

**Rationale:** Capability-based authorization provides fine-grained control over system resource access and reduces attack surface.

#### 7.2.2. Role-Based Access Control

**Standard:** Users are assigned roles that determine their capabilities.

**Roles:**

| Role | Capabilities | Description |
|-------|--------------|-------------|
| **Administrator** | All capabilities | Full system access |
| **Editor** | File read/write, Git operations | Document editing |
| **Viewer** | File read only | Read-only access |
| **Guest** | Limited capabilities | Restricted access |

**Rationale:** Role-based access control enables separation of duties and limits the impact of compromised accounts.

#### 7.2.3. Resource Scoping

**Standard:** File system operations must be scoped to authorized directories.

**Implementation:**

- Each capability includes path scope configuration
- Path scopes are validated against repository root
- Path traversal attacks are prevented through path normalization
- Symbolic link attacks are prevented through path resolution

**Example Configuration:**

```json
{
  "identifier": "default",
  "description": "Default capability set",
  "windows": ["main"],
  "permissions": [
    {
      "identifier": "fs:read",
      "allow": [{ "path": "$HOME/Documents" }]
    },
    {
      "identifier": "fs:write",
      "allow": [{ "path": "$HOME/Documents" }]
    }
  ]
}
```

**Rationale:** Resource scoping prevents unauthorized access to sensitive system directories and mitigates path traversal attacks.

### 7.3. Input Validation Requirements

**Requirement ID:** REQ-SEC-003
**Related Requirements:** REQ-DESK-080 (Link Validation), REQ-DESK-034 (File Locking)

#### 7.3.1. Path Validation

**Standard:** All file paths must be validated before use.

**Validation Rules:**

- Paths must be relative to repository root
- Paths must not contain `..` segments (path traversal prevention)
- Paths must not exceed 255 characters
- Paths must not contain illegal characters (null, control characters)
- Paths must be normalized before use

**Implementation:**

```rust
fn validate_document_path(path: &str) -> Result<String, DocumentError> {
    // Check length
    if path.len() > 255 {
        return Err(DocumentError::InvalidPath(
            "Path exceeds maximum length".to_string()
        ));
    }
    
    // Check for path traversal
    if path.contains("..") {
        return Err(DocumentError::InvalidPath(
            "Path contains parent directory references".to_string()
        ));
    }
    
    // Check for illegal characters
    if path.chars().any(|c| c.is_control() || c == '\0') {
        return Err(DocumentError::InvalidPath(
            "Path contains illegal characters".to_string()
        ));
    }
    
    // Normalize path
    let normalized = std::path::PathBuf::from(path)
        .canonicalize()
        .map_err(|_| DocumentError::InvalidPath(
            "Failed to normalize path".to_string()
        ))?;
    
    Ok(normalized.to_string_lossy().to_string())
}
```

**Rationale:** Path validation prevents path traversal attacks and ensures that operations are limited to authorized directories.

#### 7.3.2. Content Validation

**Standard:** All content must be validated before processing.

**Validation Rules:**

- Content size must not exceed maximum limits
- Content must be valid UTF-8
- Content must not contain malicious patterns
- Content must be sanitized before storage

**Implementation:**

```rust
fn validate_document_content(content: &str) -> Result<(), DocumentError> {
    // Check size
    if content.len() > MAX_DOCUMENT_SIZE {
        return Err(DocumentError::InvalidContent(
            "Content exceeds maximum size".to_string()
        ));
    }
    
    // Check UTF-8 validity
    if !content.is_utf8() {
        return Err(DocumentError::InvalidContent(
            "Content is not valid UTF-8".to_string()
        ));
    }
    
    // Check for malicious patterns
    if contains_malicious_patterns(content) {
        return Err(DocumentError::InvalidContent(
            "Content contains potentially malicious patterns".to_string()
        ));
    }
    
    Ok(())
}
```

**Rationale:** Content validation prevents injection attacks and ensures data integrity.

#### 7.3.3. Query Validation

**Standard:** All search queries must be validated before execution.

**Validation Rules:**

- Query length must not exceed maximum limits
- Query must be sanitized to prevent injection attacks
- Query must not contain control characters

**Implementation:**

```rust
fn validate_search_query(query: &str) -> Result<(), SearchError> {
    // Check length
    if query.is_empty() || query.len() > MAX_QUERY_LENGTH {
        return Err(SearchError::InvalidQuery(
            "Query length is invalid".to_string()
        ));
    }
    
    // Sanitize query
    let sanitized = query.chars()
        .filter(|c| !c.is_control())
        .collect::<String>();
    
    if sanitized.len() != query.len() {
        return Err(SearchError::InvalidQuery(
            "Query contains invalid characters".to_string()
        ));
    }
    
    Ok(())
}
```

**Rationale:** Query validation prevents injection attacks and ensures search operation stability.

### 7.4. Error Handling Requirements

**Requirement ID:** REQ-SEC-004
**Related Requirements:** REQ-DESK-082 (Error Notifications)

#### 7.4.1. Error Message Sanitization

**Standard:** Error messages must be sanitized to prevent information disclosure.

**Implementation:**

- Error messages must not contain internal system paths
- Error messages must not contain stack traces
- Error messages must not reveal implementation details
- Debug information must be logged separately

**Rationale:** Error message sanitization prevents information disclosure attacks while maintaining usability.

#### 7.4.2. Error Logging

**Standard:** All errors must be logged for audit trail and debugging.

**Implementation:**

- Errors are logged with timestamp, severity, and context
- Sensitive information is redacted from logs
- Logs are rotated and retained for a defined period
- Log access is restricted to authorized users

**Rationale:** Error logging provides audit trail and enables incident response while protecting sensitive information.

### 7.5. Threat Mitigation

**Requirement ID:** REQ-SEC-005
**Related Requirements:** REQ-DESK-052 (WebView Security), REQ-DESK-080 (Link Validation)

#### 7.5.1. Path Traversal Prevention

**Threat:** Attackers attempt to access files outside the authorized directory using `..` sequences.

**Mitigation:**

- All paths are normalized before use
- Paths are validated to be within authorized scope
- Symbolic links are resolved and validated

#### 7.5.2. Injection Attack Prevention

**Threat:** Attackers attempt to inject malicious content or commands through input fields.

**Mitigation:**

- All input is validated and sanitized
- Parameterized queries are used for database operations
- Content Security Policy is enforced in WebView

#### 7.5.3. Denial of Service Prevention

**Threat:** Attackers attempt to exhaust system resources through excessive requests.

**Mitigation:**

- Rate limiting is enforced for all commands
- Resource limits are enforced per session
- Long-running operations support cancellation

#### 7.5.4. Cross-Site Scripting Prevention

**Threat:** Attackers attempt to inject malicious scripts through document content.

**Mitigation:**

- Content Security Policy is enforced in WebView
- HTML content is sanitized before rendering
- JavaScript execution is restricted to authorized contexts

#### 7.5.5. Data Exfiltration Prevention

**Threat:** Attackers attempt to exfiltrate sensitive data through compromised commands.

**Mitigation:**

- Capability-based authorization limits data access
- Sensitive information is redacted from error messages
- Network requests are restricted to authorized endpoints

---

## 8. COMMAND PERFORMANCE

Command performance defines latency, throughput, and optimization requirements for all Desktop Commands API operations. These requirements ensure that the Desktop Commands API meets the performance expectations of users and supports efficient operation at scale.

### 8.1. Latency Requirements

**Requirement ID:** REQ-PERF-001
**Related Requirements:** REQ-DESK-086 (Hot-Reload Latency), REQ-DESK-087 (Initial Load Time)

#### 8.1.1. Latency Targets

**Standard:** All commands must meet defined latency targets for typical operations.

**Latency Targets by Command Category:**

| Command Category | Target Latency | Measurement Method |
|-----------------|---------------|-------------------|
| **Document Read** | < 20ms (cached), < 100ms (uncached) | End-to-end time |
| **Document Write** | < 50ms | End-to-end time |
| **Document List** | < 200ms (10K docs) | End-to-end time |
| **Repository Status** | < 100ms | End-to-end time |
| **Search Query** | < 100ms (10K docs) | End-to-end time |
| **System Status** | < 50ms | End-to-end time |
| **Configuration** | < 100ms | End-to-end time |

**Rationale:** Defined latency targets ensure that the desktop application remains responsive and provides a smooth user experience.

#### 8.1.2. Latency Measurement

**Standard:** Latency must be measured from WebView to Rust backend and back.

**Implementation:**

- Latency is measured at command boundaries
- Latency includes serialization and deserialization overhead
- Latency is logged for performance monitoring
- Latency outliers trigger alerts

**Measurement Points:**

```rust
#[command]
pub async fn example_command(
    request: ExampleRequest,
    state: State<'_, AppState>,
) -> Result<ExampleResponse, ExampleError> {
    let start = std::time::Instant::now();
    
    // Execute command logic
    let result = execute_command_logic(request, state).await?;
    
    let duration = start.elapsed();
    
    // Log latency
    log_latency("example_command", duration).await;
    
    Ok(result)
}

async fn log_latency(command_name: &str, duration: Duration) {
    if duration > LATENCY_THRESHOLD {
        warn!("High latency detected: {} took {}ms", command_name, duration.as_millis());
    }
}
```

**Rationale:** Latency measurement enables performance monitoring and identifies regression opportunities.

### 8.2. Throughput Requirements

**Requirement ID:** REQ-PERF-002
**Related Requirements:** REQ-DESK-090 (Responsive UI)

#### 8.2.1. Throughput Targets

**Standard:** All commands must support defined throughput targets for concurrent operations.

**Throughput Targets by Command Category:**

| Command Category | Target Throughput | Concurrent Connections |
|-----------------|------------------|---------------------|
| **Document Read** | 100 req/s | 100 concurrent |
| **Document Write** | 100 req/s | 100 concurrent |
| **Document List** | 50 req/s | 50 concurrent |
| **Repository Status** | 100 req/s | 100 concurrent |
| **Search Query** | 100 req/s | 100 concurrent |
| **System Status** | 100 req/s | 100 concurrent |
| **Configuration** | 50 req/s | 50 concurrent |

**Rationale:** Defined throughput targets ensure that the desktop application can handle concurrent user operations efficiently.

#### 8.2.2. Connection Pooling

**Standard:** IPC connections must be pooled to reduce overhead.

**Implementation:**

- Connections are reused across multiple commands
- Connection pool size is configurable
- Idle connections are closed after timeout
- Connection health is monitored

**Implementation:**

```rust
pub struct ConnectionPool {
    connections: Vec<Connection>,
    max_connections: usize,
    idle_timeout: Duration,
}

impl ConnectionPool {
    pub async fn acquire(&mut self) -> Connection {
        // Reuse existing connection or create new one
        if let Some(conn) = self.connections.pop() {
            conn
        } else {
            create_new_connection().await
        }
    }
    
    pub fn release(&mut self, conn: Connection) {
        self.connections.push(conn);
    }
}
```

**Rationale:** Connection pooling reduces connection overhead and improves throughput.

### 8.3. Optimization Strategies

**Requirement ID:** REQ-PERF-003
**Related Requirements:** REQ-DESK-041 (LRU Cache), REQ-DESK-042 (Cache Invalidation)

#### 8.3.1. Caching Strategy

**Standard:** Frequently accessed data must be cached to improve performance.

**Implementation:**

- LRU cache is used for document content
- Cache size is configurable (default: 500MB)
- Cache entries are invalidated on modification
- Cache statistics are monitored

**Cache Configuration:**

```rust
pub struct LruCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    current_size: usize,
    access_order: VecDeque<String>,
}

impl LruCache {
    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(entry) = self.entries.get(key) {
            // Update access order
            self.access_order.retain(|k| k != key);
            self.access_order.push_back(key.to_string());
            Some(entry.content.clone())
        } else {
            None
        }
    }
    
    pub fn insert(&mut self, key: String, content: String) {
        // Evict if necessary
        while self.current_size + content.len() > self.max_size {
            if let Some(evicted_key) = self.access_order.pop_front() {
                if let Some(entry) = self.entries.remove(&evicted_key) {
                    self.current_size -= entry.size;
                }
            }
        }
        
        // Insert new entry
        self.entries.insert(key.clone(), CacheEntry { content, size: content.len() });
        self.current_size += content.len();
        self.access_order.push_back(key);
    }
}
```

**Rationale:** Caching reduces disk I/O and improves latency for frequently accessed data.

#### 8.3.2. Batch Processing

**Standard:** Multiple operations should be batched to reduce overhead.

**Implementation:**

- Batch commands support multiple operations in a single request
- Batch operations are executed atomically
- Batch results include per-item status

**Rationale:** Batch processing reduces IPC overhead and improves throughput.

#### 8.3.3. Async Processing

**Standard:** All I/O operations must be asynchronous to avoid blocking.

**Implementation:**

- File I/O uses Tokio async runtime
- Network I/O uses async/await
- Long-running operations support cancellation

**Rationale:** Async processing ensures that the UI thread remains responsive during I/O operations.

#### 8.3.4. Memory Pooling

**Standard:** Memory allocations must be pooled to reduce GC pressure.

**Implementation:**

- Buffers are reused across operations
- Memory pools are used for frequent allocations
- Pool size is configurable

**Implementation:**

```rust
pub struct BufferPool<T> {
    buffers: Vec<T>,
    max_buffers: usize,
}

impl<T: Default + Clone> BufferPool<T> {
    pub fn acquire(&mut self) -> T {
        self.buffers.pop().unwrap_or_else(|| T::default())
    }
    
    pub fn release(&mut self, buffer: T) {
        if self.buffers.len() < self.max_buffers {
            self.buffers.push(buffer);
        }
    }
}
```

**Rationale:** Memory pooling reduces GC pressure and improves performance.

### 8.4. Performance Monitoring

**Requirement ID:** REQ-PERF-004
**Related Requirements:** REQ-DESK-047 (Server Health Monitoring)

#### 8.4.1. Metrics Collection

**Standard:** Performance metrics must be collected for monitoring and optimization.

**Metrics Collected:**

- Command latency (p50, p95, p99)
- Command throughput (requests per second)
- Error rate (errors per second)
- Cache hit rate
- Memory usage
- CPU usage

**Implementation:**

```rust
pub struct PerformanceMetrics {
    command_latencies: HashMap<String, Vec<Duration>>,
    command_throughput: HashMap<String, f64>,
    error_rates: HashMap<String, f64>,
    cache_hit_rate: f32,
    memory_usage: usize,
    cpu_usage: f32,
}

impl PerformanceMetrics {
    pub fn record_latency(&mut self, command: &str, latency: Duration) {
        self.command_latencies
            .entry(command.to_string())
            .or_insert_with(Vec::new)
            .push(latency);
    }
    
    pub fn get_p50(&self, command: &str) -> Option<Duration> {
        let latencies = self.command_latencies.get(command)?;
        if latencies.is_empty() {
            return None;
        }
        let mut sorted = latencies.clone();
        sorted.sort();
        Some(sorted[sorted.len() / 2])
    }
}
```

**Rationale:** Performance metrics enable data-driven optimization and regression detection.

#### 8.4.2. Alerting

**Standard:** Performance alerts must be triggered when thresholds are exceeded.

**Alert Thresholds:**

- Latency > 2x target for 5 consecutive operations
- Error rate > 1% for 5 consecutive minutes
- Memory usage > 80% of limit
- CPU usage > 80% for 5 consecutive minutes

**Rationale:** Alerting enables proactive performance issue detection and resolution.

---

## 9. REFERENCES

This section provides references to related documents, standards, and external resources that inform the Desktop Commands API specification.

### 9.1. Internal Project References

| Document ID | Title | Path | Relevance |
|-------------|-------|------|-----------|
| [TACHYON-STD-V1.0](../../.adrs/ | Coding and Documentation Standards | Defines coding standards and documentation conventions |
| [TACHYON-REQ-DESK-V1.0](../../.adrs/ | Desktop Application Requirements | Defines functional requirements for desktop application |
| [TACHYON-DES-DESK-V1.0](../../.adrs/ | Desktop Application Design | Defines technical design for desktop application |
| [TACHYON-ADR-002-V1.0](../../.adrs/adr-002-bm25-search-parameters.md) | ADR-002: Tauri for Desktop Application | Justifies Tauri framework selection |
| [TACHYON-ADR-009-V1.0](../../.adrs/adr-009-race-condition-mitigation.md) | ADR-009: IPC Communication Architecture | Justifies IPC architecture decisions |
| [TACHYON-TMA-V1.0](../../.adrs/ | Threat Model Analysis | Defines security threats and mitigations |
| [TACHYON-TSK-V1.0](../../.adrs/ | Execution Tasks and Work Breakdown Structure | Defines task TSK-017 context |

### 9.2. Requirement Traceability

The following table maps Desktop Commands API elements to related requirements from [TACHYON-REQ-DESK-V1.0](../../.adrs/

| Command ID | Command Name | Related Requirements |
|------------|-------------|---------------------|
| CMD-DOC-001 | create_document | REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit) |
| CMD-DOC-002 | get_document | REQ-DESK-087 (Initial Load Time), REQ-DESK-041 (LRU Cache) |
| CMD-DOC-003 | update_document | REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit), REQ-DESK-042 (Cache Invalidation) |
| CMD-DOC-004 | delete_document | REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit) |
| CMD-DOC-005 | list_documents | REQ-DESK-016 (Sidebar Navigation), REQ-DESK-041 (LRU Cache) |
| CMD-DOC-006 | batch_create_documents | REQ-DESK-035 (File Operations), REQ-DESK-038 (Auto-Commit) |
| CMD-REP-001 | add_to_repository | REQ-DESK-036 (Repository Initialization), REQ-DESK-038 (Auto-Commit) |
| CMD-REP-002 | remove_from_repository | REQ-DESK-036 (Repository Initialization) |
| CMD-REP-003 | sync_repository | REQ-DESK-037 (Repository Cloning), REQ-DESK-038 (Auto-Commit) |
| CMD-REP-004 | get_repository_status | REQ-DESK-036 (Repository Initialization), REQ-DESK-040 (History Viewing) |
| CMD-REP-005 | list_branches | REQ-DESK-039 (Branch Management) |
| CMD-SRC-001 | search_documents | REQ-DESK-020 (Quick Open), REQ-DESK-041 (LRU Cache) |
| CMD-SRC-002 | filter_documents | REQ-DESK-016 (Sidebar Navigation) |
| CMD-SRC-003 | sort_documents | REQ-DESK-016 (Sidebar Navigation) |
| CMD-SRC-004 | autocomplete_documents | REQ-DESK-020 (Quick Open), REQ-DESK-024 (Auto-Complete) |
| CMD-SYS-001 | initialize_application | REQ-DESK-001 (Application Startup), REQ-DESK-046 (Server Spawn) |
| CMD-SYS-002 | get_config | REQ-DESK-026 (State Management) |
| CMD-SYS-003 | set_config | REQ-DESK-026 (State Management) |
| CMD-SYS-004 | get_application_status | REQ-DESK-047 (Server Health Monitoring), REQ-DESK-030 (Status Bar) |
| CMD-SYS-005 | quit_application | REQ-DESK-006 (Graceful Shutdown) |

### 9.3. Design Element Traceability

The following table maps Desktop Commands API elements to related design elements from [TACHYON-DES-DESK-V1.0](../../.adrs/

| Command ID | Command Name | Related Design Elements |
|------------|-------------|------------------------|
| CMD-DOC-001 through CMD-DOC-006 | Document Commands | DES-DESK-003 (IpcCommandHandlers) |
| CMD-REP-001 through CMD-REP-005 | Repository Commands | DES-DESK-003 (IpcCommandHandlers) |
| CMD-SRC-001 through CMD-SRC-004 | Search Commands | DES-DESK-003 (IpcCommandHandlers) |
| CMD-SYS-001 through CMD-SYS-005 | System Commands | DES-DESK-003 (IpcCommandHandlers) |

### 9.4. ADR Traceability

The following table maps Desktop Commands API elements to related Architectural Decision Records:

| Command Category | Related ADRs |
|----------------|-------------|
| All Commands | [ADR-002](../../.adrs/adr-002-bm25-search-parameters.md) (Tauri for Desktop Application) |
| IPC Communication | [ADR-009](../../.adrs/adr-009-race-condition-mitigation.md) (IPC Communication Architecture) |
| Security | [ADR-002](../../.adrs/adr-002-bm25-search-parameters.md) (Capability-Based Security) |

### 9.5. External Standards and References

| Standard/Reference | Description | URL |
|------------------|-----------|-----|
| ISO/IEC 26514:2021 | Systems and Software Engineering -- Requirements for documentation | https://www.iso.org/standard/iso-iec-26514 |
| ISO/IEC 12207:2017 | Systems and Software Engineering -- Software lifecycle processes | https://www.iso.org/standard/iso-iec-12207 |
| ISO/IEC 25010:2011 | System and Software Quality Requirements | https://www.iso.org/standard/iso-iec-25010 |
| IEEE 829-2008 | Software Test Documentation | https://standards.ieee.org/standard/829/2008.html |
| IEEE 1063-2001 | Standard for Software User Documentation | https://standards.ieee.org/standard/1063/2001.html |
| IEEE 1016-2009 | Standard for Information Technology | https://standards.ieee.org/standard/1016/2009.html |
| Tauri Documentation | Tauri Framework Documentation | https://tauri.app/v1/guides/ |
| Rust Documentation | The Rust Programming Language | https://doc.rust-lang.org/ |
| Serde Documentation | Serialization Framework for Rust | https://serde.rs/ |
| Tokio Documentation | Asynchronous Runtime for Rust | https://tokio.rs/ |
| Git2-rs Documentation | Git Bindings for Rust | https://docs.rs/git2-rs/ |

### 9.6. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| V1.0 | 2026-02-05 | Initial release | Initial specification document |

---

**END OF DOCUMENT**

This document concludes the Desktop Commands API specification for the Tachyon Desktop Application. For questions or clarifications regarding this specification, please refer to the project documentation standards in [TACHYON-STD-V1.0](../../.adrs/ or contact the architecture team.
