# TACHYON: DESKTOP API SPECIFICATION

**Document ID:** TACHYON-API-001-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification Document
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [API Design Principles](#2-api-design-principles)
3. [Versioning Strategy](#3-versioning-strategy)
4. [Desktop Commands API](#4-desktop-commands-api)
5. [Desktop Events API](#5-desktop-events-api)
6. [IPC Communication Protocol](#6-ipc-communication-protocol)
7. [API Security](#7-api-security)
8. [API Performance](#8-api-performance)
9. [API Documentation](#9-api-documentation)
10. [References](#10-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document defines comprehensive API specifications for the Tachyon Desktop Application, implemented using the Tauri framework. The specifications provide detailed definitions of all inter-process communication (IPC) commands, events, and protocols between the WebView frontend and Rust backend, enabling type-safe, secure, and efficient communication.

### 1.2. Scope

This document covers the following API categories:

- **Desktop Commands API:** Request/response commands for document operations, repository management, search functionality, and system operations
- **Desktop Events API:** Event-based notifications for document changes, repository updates, and system events
- **IPC Communication Protocol:** Tauri IPC mechanisms, serialization, error handling, and security controls
- **API Security:** Authentication, authorization, input validation, and audit logging
- **API Performance:** Latency requirements, throughput targets, caching strategies, and optimization techniques

Out of scope:
- Core rendering engine APIs (covered in core API specification)
- Server component APIs (covered in server API specification)
- Web frontend APIs (covered in web API specification)

### 1.3. Document Dependencies

This document depends on the following documents:

- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-REQ-DESK-V1.0](../../.specs/04_future_state/reqs/desktop_requirements.md) - Desktop Application Requirements
- [TACHYON-DES-DESK-V1.0](../../.specs/04_future_state/design/desktop_design.md) - Desktop Application Design
- [TACHYON-ADR-002-V1.0](../../.specs/02_adrs/002_tauri_for_desktop_application.md) - Tauri for Desktop Application
- [TACHYON-ADR-009-V1.0](../../.specs/02_adrs/009_ipc_communication_architecture.md) - IPC Communication Architecture
- [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) - Security Architecture

### 1.4. Target Audience

This document is intended for:

- **Frontend Developers:** Developers working on the Leptos WebView frontend
- **Backend Developers:** Developers working on the Rust backend and IPC command handlers
- **System Architects:** Architects designing system integration and communication patterns
- **Quality Assurance Engineers:** Engineers testing IPC communication and API contracts
- **Security Engineers:** Engineers auditing API security and access controls

### 1.5. Conventions and Notation

#### 1.5.1. Type Definitions

All type definitions use Rust syntax for backend types and TypeScript syntax for frontend types.

**Rust Type Notation:**
```rust
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
}
```

**TypeScript Type Notation:**
```typescript
interface Document {
    id: string;
    title: string;
    content: string;
}
```

#### 1.5.2. Command Signature Format

IPC commands are documented using the following format:

```
Command: command_name

Description:
    Brief description of command purpose.

Parameters:
    - param1: Type - Description
    - param2: Type - Description

Returns:
    Type - Description of return value.

Errors:
    - ErrorType1 - Description of error condition
    - ErrorType2 - Description of error condition

Example:
    Frontend invocation example.
```

#### 1.5.3. Event Signature Format

IPC events are documented using the following format:

```
Event: event_name

Description:
    Brief description of event purpose.

Payload:
    Type - Description of event payload.

Emission Conditions:
    Conditions under which event is emitted.

Example:
    Frontend subscription example.
```

---

## 2. API DESIGN PRINCIPLES

### 2.1. Type Safety

**Principle:** All IPC communication must be type-safe at compile time, preventing entire classes of runtime errors.

**Implementation:**

- Tauri's command system with automatic type generation
- serde for type-safe serialization and deserialization
- Rust's type system for compile-time checking
- TypeScript type definitions generated from Rust types

**Benefits:**

1. **Compile-Time Error Detection:** Type errors caught at compile time
2. **Automatic Serialization:** serde automatically handles serialization
3. **Refactoring Safety:** Refactoring automatically updates IPC types
4. **Documentation:** Types serve as living documentation

**Requirement Traceability:**
- REQ-IPC-025: Type-Safe Serialization
- REQ-IPC-026: Command Registration

**ADR Reference:**
- [ADR-009: IPC Communication Architecture](../../.specs/02_adrs/009_ipc_communication_architecture.md) - Type Safety section

### 2.2. Principle of Least Privilege

**Principle:** All IPC commands must operate with minimum required permissions, implementing capability-based access control.

**Implementation:**

- Tauri's capability system for fine-grained permissions
- File system access scoped to specific directories
- Shell command execution restricted to allow-listed commands
- Window operations restricted to allow-listed windows

**Benefits:**

1. **Reduced Attack Surface:** Minimal privileges reduce attack surface
2. **Compartmentalization:** Failed attacks are contained
3. **Auditable Access:** All access is controlled and auditable
4. **Explicit Authorization:** Explicit authorization for all operations

**Requirement Traceability:**
- REQ-DESK-052: WebView Security
- REQ-IPC-028: Capability-Based Authorization
- REQ-SEC-015: Principle of Least Privilege

**ADR Reference:**
- [ADR-010: Security Architecture](../../.specs/02_adrs/010_security_architecture.md) - Capability-Based Access Control section

### 2.3. Fail-Safe Error Handling

**Principle:** All IPC errors must be handled securely without exposing sensitive information or creating security vulnerabilities.

**Implementation:**

- Result<T, E> type for error propagation
- Custom error types with user-friendly messages
- No sensitive information in error messages
- Comprehensive error logging for debugging

**Benefits:**

1. **No Information Leakage:** Sensitive information not exposed in errors
2. **Secure Defaults:** Default error handling is secure
3. **Fail-Safe:** System fails safely on errors
4. **User-Friendly Messages:** Error messages are user-friendly but secure

**Requirement Traceability:**
- REQ-IPC-029: Error Propagation
- REQ-SEC-018: Fail-Safe Error Handling

**ADR Reference:**
- [ADR-010: Security Architecture](../../.specs/02_adrs/010_security_architecture.md) - Fail-Safe Error Handling section

### 2.4. Idempotency

**Principle:** All state-modifying commands must be idempotent, enabling safe retry without side effects.

**Implementation:**

- Document save operations use content hash for idempotency
- Repository sync operations use Git's idempotent operations
- Cache clear operations are idempotent by design
- State update operations use version numbers for idempotency

**Benefits:**

1. **Safe Retry:** Failed operations can be safely retried
2. **Network Resilience:** Network issues don't cause data corruption
3. **Concurrent Safety:** Concurrent operations don't cause conflicts
4. **Deterministic Behavior:** Operations have predictable outcomes

**Requirement Traceability:**
- REQ-IPC-030: Idempotent Operations
- REQ-DESK-038: Auto-Commit

### 2.5. Asynchronous Non-Blocking

**Principle:** All IPC commands must be asynchronous and non-blocking, maintaining UI responsiveness.

**Implementation:**

- Async/await for all command handlers
- Tokio runtime for asynchronous execution
- Cancellation tokens for long-running operations
- Progress events for long-running operations

**Benefits:**

1. **UI Responsiveness:** UI remains responsive during operations
2. **Cancellation Support:** Long-running operations can be cancelled
3. **Progress Feedback:** Progress events provide user feedback
4. **Resource Efficiency:** Efficient resource usage

**Requirement Traceability:**
- REQ-IPC-031: Asynchronous Commands
- REQ-DESK-090: Responsive UI

---

## 3. VERSIONING STRATEGY

### 3.1. Semantic Versioning

**Principle:** API versioning follows Semantic Versioning 2.0.0 (semver) specification.

**Version Format:** `MAJOR.MINOR.PATCH`

- **MAJOR:** Incompatible API changes
- **MINOR:** Backwards-compatible functionality additions
- **PATCH:** Backwards-compatible bug fixes

**Examples:**

| Version | Change Type | Description |
|---------|-------------|-------------|
| 1.0.0 → 1.1.0 | MINOR | Added new command `search_by_tag` |
| 1.1.0 → 1.1.1 | PATCH | Fixed bug in `get_document` command |
| 1.1.1 → 2.0.0 | MAJOR | Removed deprecated `old_command` |

### 3.2. API Deprecation Policy

**Principle:** Deprecated APIs must be supported for at least two minor versions before removal.

**Deprecation Timeline:**

1. **Announcement:** Deprecated API announced in release notes
2. **Warning:** Deprecation warnings emitted in logs
3. **Support:** Deprecated API supported for two minor versions
4. **Removal:** Deprecated API removed in next major version

**Example:**

| Version | Status | Action |
|---------|--------|--------|
| 1.0.0 | Stable | `old_command` available |
| 1.1.0 | Deprecated | `old_command` deprecated, warning emitted |
| 1.2.0 | Deprecated | `old_command` still available, warning emitted |
| 2.0.0 | Removed | `old_command` removed |

### 3.3. Backwards Compatibility

**Principle:** MINOR and PATCH versions must maintain backwards compatibility.

**Backwards Compatibility Requirements:**

1. **New Commands:** New commands can be added without breaking changes
2. **New Parameters:** New optional parameters can be added to existing commands
3. **New Event Payloads:** New optional fields can be added to event payloads
4. **New Enum Variants:** New enum variants can be added to existing enums

**Breaking Changes Require MAJOR Version:**

1. **Command Removal:** Removing a command
2. **Parameter Removal:** Removing a required parameter
3. **Parameter Type Change:** Changing parameter type in incompatible way
4. **Event Payload Change:** Changing event payload structure
5. **Enum Variant Removal:** Removing enum variant

### 3.4. Type Generation

**Principle:** TypeScript types are automatically generated from Rust type definitions.

**Type Generation Process:**

1. **Rust Types:** Define Rust types with serde serialization
2. **Tauri CLI:** Run `tauri info` to generate TypeScript types
3. **Type Export:** Export generated types to frontend codebase
4. **Type Validation:** Validate generated types in CI/CD pipeline

**Benefits:**

1. **Single Source of Truth:** Rust types are the source of truth
2. **Automatic Updates:** TypeScript types automatically updated
3. **Type Safety:** Compile-time type checking across IPC boundary
4. **Reduced Maintenance:** No manual type synchronization required

**Requirement Traceability:**
- REQ-IPC-032: Type Generation

---

## 4. DESKTOP COMMANDS API

This section defines all IPC commands for desktop-specific operations, including document management, repository operations, search functionality, and system operations.

### 4.1. Document Commands

#### 4.1.1. Create Document

**Command:** `create_document`

**Description:**
Creates a new document in the current repository with specified title and content.

**Rust Signature:**
```rust
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub document_id: String,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Validate title length
    if request.title.is_empty() || request.title.len() > 100 {
        return Err(IpcError::InvalidInput("Title must be 1-100 characters".to_string()));
    }

    // Validate content size
    if request.content.len() > 10 * 1024 * 1024 {
        return Err(IpcError::InvalidInput("Content exceeds 10MB limit".to_string()));
    }

    // Create document
    let document_id = state.write().await.core.create_document(
        &request.title,
        &request.content,
        request.path.as_deref(),
    ).await?;

    Ok(CreateDocumentResponse {
        document_id,
        path: request.path.unwrap_or_else(|| "./".to_string()),
        created_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface CreateDocumentRequest {
    title: string;
    content: string;
    path?: string;
}

interface CreateDocumentResponse {
    document_id: string;
    path: string;
    created_at: string;
}

async function createDocument(request: CreateDocumentRequest): Promise<CreateDocumentResponse>;
```

**Parameters:**
- `title`: String - Document title (1-100 characters)
- `content`: String - Document content (max 10MB)
- `path`: Optional<String> - Document path relative to repository root

**Returns:**
- `CreateDocumentResponse` - Document creation result with ID, path, and timestamp

**Errors:**
- `InvalidInput` - Title or content validation failed
- `RepositoryNotInitialized` - No repository is currently open
- `FileSystemError` - File system operation failed
- `PermissionDenied` - Insufficient permissions to create document

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('create_document', {
    title: 'My First Document',
    content: '# Hello World\n\nThis is my first document.',
    path: 'docs/my-first-document.md'
});

console.log('Document created:', result.document_id);
```

**Requirement Traceability:**
- REQ-DESK-035: File Operations
- REQ-IPC-027: Command Execution

#### 4.1.2. Read Document

**Command:** `get_document`

**Description:**
Retrieves a document by ID or path from the current repository.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentRequest {
    pub id: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentResponse {
    pub document: Document,
    pub rendered_html: String,
    pub metadata: DocumentMetadata,
}

#[tauri::command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentResponse, IpcError> {
    // Either id or path must be provided
    if request.id.is_none() && request.path.is_none() {
        return Err(IpcError::InvalidInput("Either id or path must be provided".to_string()));
    }

    // Retrieve document
    let document = state.read().await.core.get_document(
        request.id.as_deref(),
        request.path.as_deref(),
    ).await?;

    // Render HTML
    let rendered_html = state.read().await.core.render_document(&document).await?;

    // Get metadata
    let metadata = state.read().await.core.get_document_metadata(&document.id).await?;

    Ok(GetDocumentResponse {
        document,
        rendered_html,
        metadata,
    })
}
```

**TypeScript Signature:**
```typescript
interface GetDocumentRequest {
    id?: string;
    path?: string;
}

interface GetDocumentResponse {
    document: Document;
    rendered_html: string;
    metadata: DocumentMetadata;
}

async function getDocument(request: GetDocumentRequest): Promise<GetDocumentResponse>;
```

**Parameters:**
- `id`: Optional<String> - Document ID
- `path`: Optional<String> - Document path

**Returns:**
- `GetDocumentResponse` - Document with rendered HTML and metadata

**Errors:**
- `InvalidInput` - Neither id nor path provided
- `DocumentNotFound` - Document not found
- `RepositoryNotInitialized` - No repository is currently open
- `FileSystemError` - File system operation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('get_document', {
    id: 'doc-123'
});

console.log('Document title:', result.document.title);
console.log('Rendered HTML:', result.rendered_html);
```

**Requirement Traceability:**
- REQ-DESK-031: File Watching
- REQ-DESK-087: Initial Load Time

#### 4.1.3. Update Document

**Command:** `update_document`

**Description:**
Updates an existing document with new title and/or content.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentResponse {
    pub document_id: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

#[tauri::command]
pub async fn update_document(
    request: UpdateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<UpdateDocumentResponse, IpcError> {
    // Validate at least one field is being updated
    if request.title.is_none() && request.content.is_none() {
        return Err(IpcError::InvalidInput("Either title or content must be provided".to_string()));
    }

    // Validate title if provided
    if let Some(ref title) = request.title {
        if title.is_empty() || title.len() > 100 {
            return Err(IpcError::InvalidInput("Title must be 1-100 characters".to_string()));
        }
    }

    // Validate content size if provided
    if let Some(ref content) = request.content {
        if content.len() > 10 * 1024 * 1024 {
            return Err(IpcError::InvalidInput("Content exceeds 10MB limit".to_string()));
        }
    }

    // Update document
    let (document_id, version) = state.write().await.core.update_document(
        &request.id,
        request.title.as_deref(),
        request.content.as_deref(),
    ).await?;

    Ok(UpdateDocumentResponse {
        document_id,
        updated_at: chrono::Utc::now(),
        version,
    })
}
```

**TypeScript Signature:**
```typescript
interface UpdateDocumentRequest {
    id: string;
    title?: string;
    content?: string;
}

interface UpdateDocumentResponse {
    document_id: string;
    updated_at: string;
    version: number;
}

async function updateDocument(request: UpdateDocumentRequest): Promise<UpdateDocumentResponse>;
```

**Parameters:**
- `id`: String - Document ID
- `title`: Optional<String> - New document title (1-100 characters)
- `content`: Optional<String> - New document content (max 10MB)

**Returns:**
- `UpdateDocumentResponse` - Update result with ID, timestamp, and version

**Errors:**
- `InvalidInput` - Title or content validation failed
- `DocumentNotFound` - Document not found
- `VersionConflict` - Document version conflict (optimistic locking)
- `RepositoryNotInitialized` - No repository is currently open
- `FileSystemError` - File system operation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('update_document', {
    id: 'doc-123',
    title: 'Updated Title'
});

console.log('Document updated:', result.document_id);
console.log('New version:', result.version);
```

**Requirement Traceability:**
- REQ-DESK-034: File Locking
- REQ-IPC-030: Idempotent Operations

#### 4.1.4. Delete Document

**Command:** `delete_document`

**Description:**
Deletes a document from the current repository.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentRequest {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentResponse {
    pub document_id: String,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn delete_document(
    request: DeleteDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<DeleteDocumentResponse, IpcError> {
    // Delete document
    state.write().await.core.delete_document(&request.id).await?;

    Ok(DeleteDocumentResponse {
        document_id: request.id,
        deleted_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface DeleteDocumentRequest {
    id: string;
}

interface DeleteDocumentResponse {
    document_id: string;
    deleted_at: string;
}

async function deleteDocument(request: DeleteDocumentRequest): Promise<DeleteDocumentResponse>;
```

**Parameters:**
- `id`: String - Document ID

**Returns:**
- `DeleteDocumentResponse` - Deletion result with ID and timestamp

**Errors:**
- `DocumentNotFound` - Document not found
- `RepositoryNotInitialized` - No repository is currently open
- `FileSystemError` - File system operation failed
- `PermissionDenied` - Insufficient permissions to delete document

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('delete_document', {
    id: 'doc-123'
});

console.log('Document deleted:', result.document_id);
```

**Requirement Traceability:**
- REQ-DESK-035: File Operations

#### 4.1.5. List Documents

**Command:** `list_documents`

**Description:**
Lists all documents in the current repository with optional filtering and pagination.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsRequest {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentSummary>,
    pub total_count: u64,
    pub offset: u64,
    pub limit: u64,
}

#[tauri::command]
pub async fn list_documents(
    request: ListDocumentsRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<ListDocumentsResponse, IpcError> {
    // Set defaults
    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(50).min(100); // Max 100
    let sort_by = request.sort_by.unwrap_or_else(|| "created_at".to_string());
    let sort_order = request.sort_order.unwrap_or_else(|| "desc".to_string());

    // List documents
    let (documents, total_count) = state.read().await.core.list_documents(
        offset,
        limit,
        &sort_by,
        &sort_order,
    ).await?;

    Ok(ListDocumentsResponse {
        documents,
        total_count,
        offset,
        limit,
    })
}
```

**TypeScript Signature:**
```typescript
interface ListDocumentsRequest {
    offset?: number;
    limit?: number;
    sort_by?: string;
    sort_order?: string;
}

interface ListDocumentsResponse {
    documents: DocumentSummary[];
    total_count: number;
    offset: number;
    limit: number;
}

async function listDocuments(request: ListDocumentsRequest): Promise<ListDocumentsResponse>;
```

**Parameters:**
- `offset`: Optional<u64> - Pagination offset (default: 0)
- `limit`: Optional<u64> - Pagination limit (default: 50, max: 100)
- `sort_by`: Optional<String> - Sort field (default: "created_at")
- `sort_order`: Optional<String> - Sort order: "asc" or "desc" (default: "desc")

**Returns:**
- `ListDocumentsResponse` - List of documents with pagination metadata

**Errors:**
- `InvalidInput` - Invalid sort field or order
- `RepositoryNotInitialized` - No repository is currently open
- `FileSystemError` - File system operation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('list_documents', {
    offset: 0,
    limit: 20,
    sort_by: 'updated_at',
    sort_order: 'desc'
});

console.log('Total documents:', result.total_count);
console.log('Documents:', result.documents);
```

**Requirement Traceability:**
- REQ-DESK-020: Quick Open
- REQ-DESK-088: Large File Handling

### 4.2. Repository Commands

#### 4.2.1. Initialize Repository

**Command:** `init_repository`

**Description:**
Initializes a new Git repository at the specified path.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct InitRepositoryRequest {
    pub path: String,
    pub initial_commit: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitRepositoryResponse {
    pub repository_id: String,
    pub path: String,
    pub initialized_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn init_repository(
    request: InitRepositoryRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<InitRepositoryResponse, IpcError> {
    // Validate path
    let path = std::path::PathBuf::from(&request.path);
    if !path.exists() {
        return Err(IpcError::InvalidInput("Path does not exist".to_string()));
    }

    // Initialize repository
    let repository_id = state.write().await.core.init_repository(
        &path,
        request.initial_commit.unwrap_or(true),
    ).await?;

    // Update state
    state.write().await.repository_path = Some(path.clone());
    state.write().await.current_branch = Some("main".to_string());

    Ok(InitRepositoryResponse {
        repository_id,
        path: request.path,
        initialized_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface InitRepositoryRequest {
    path: string;
    initial_commit?: boolean;
}

interface InitRepositoryResponse {
    repository_id: string;
    path: string;
    initialized_at: string;
}

async function initRepository(request: InitRepositoryRequest): Promise<InitRepositoryResponse>;
```

**Parameters:**
- `path`: String - Repository path
- `initial_commit`: Optional<bool> - Create initial commit (default: true)

**Returns:**
- `InitRepositoryResponse` - Repository initialization result

**Errors:**
- `InvalidInput` - Path validation failed
- `RepositoryAlreadyExists` - Repository already exists at path
- `FileSystemError` - File system operation failed
- `PermissionDenied` - Insufficient permissions to initialize repository

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('init_repository', {
    path: '/home/user/documents/my-repo',
    initial_commit: true
});

console.log('Repository initialized:', result.repository_id);
```

**Requirement Traceability:**
- REQ-DESK-036: Repository Initialization
- REQ-DESK-068: Folder Selection

#### 4.2.2. Clone Repository

**Command:** `clone_repository`

**Description:**
Clones an existing Git repository from a remote URL.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CloneRepositoryRequest {
    pub url: String,
    pub path: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloneRepositoryResponse {
    pub repository_id: String,
    pub path: String,
    pub cloned_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn clone_repository(
    request: CloneRepositoryRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CloneRepositoryResponse, IpcError> {
    // Validate URL
    if !request.url.starts_with("http://") && !request.url.starts_with("https://") && !request.url.starts_with("git@") {
        return Err(IpcError::InvalidInput("Invalid repository URL".to_string()));
    }

    // Clone repository
    let repository_id = state.write().await.core.clone_repository(
        &request.url,
        &request.path,
        request.branch.as_deref(),
    ).await?;

    // Update state
    state.write().await.repository_path = Some(std::path::PathBuf::from(&request.path));
    state.write().await.current_branch = request.branch.clone();

    Ok(CloneRepositoryResponse {
        repository_id,
        path: request.path,
        cloned_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface CloneRepositoryRequest {
    url: string;
    path: string;
    branch?: string;
}

interface CloneRepositoryResponse {
    repository_id: string;
    path: string;
    cloned_at: string;
}

async function cloneRepository(request: CloneRepositoryRequest): Promise<CloneRepositoryResponse>;
```

**Parameters:**
- `url`: String - Repository URL (HTTPS, SSH, or local path)
- `path`: String - Destination path
- `branch`: Optional<String> - Branch to clone (default: default branch)

**Returns:**
- `CloneRepositoryResponse` - Repository clone result

**Errors:**
- `InvalidInput` - URL validation failed
- `RepositoryAlreadyExists` - Repository already exists at path
- `CloneFailed` - Git clone operation failed
- `FileSystemError` - File system operation failed
- `PermissionDenied` - Insufficient permissions to clone repository

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('clone_repository', {
    url: 'https://github.com/user/repo.git',
    path: '/home/user/documents/my-repo',
    branch: 'main'
});

console.log('Repository cloned:', result.repository_id);
```

**Requirement Traceability:**
- REQ-DESK-037: Repository Cloning
- REQ-DESK-068: Folder Selection

#### 4.2.3. Sync Repository

**Command:** `sync_repository`

**Description:**
Synchronizes the local repository with remote (pull and push).

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRepositoryRequest {
    pub pull: Option<bool>,
    pub push: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRepositoryResponse {
    pub repository_id: String,
    pub pulled: bool,
    pub pushed: bool,
    pub synced_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn sync_repository(
    request: SyncRepositoryRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SyncRepositoryResponse, IpcError> {
    // Set defaults
    let pull = request.pull.unwrap_or(true);
    let push = request.push.unwrap_or(true);

    // Sync repository
    let (pulled, pushed) = state.write().await.core.sync_repository(
        pull,
        push,
    ).await?;

    Ok(SyncRepositoryResponse {
        repository_id: state.read().await.repository_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        pulled,
        pushed,
        synced_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface SyncRepositoryRequest {
    pull?: boolean;
    push?: boolean;
}

interface SyncRepositoryResponse {
    repository_id: string;
    pulled: boolean;
    pushed: boolean;
    synced_at: string;
}

async function syncRepository(request: SyncRepositoryRequest): Promise<SyncRepositoryResponse>;
```

**Parameters:**
- `pull`: Optional<bool> - Pull from remote (default: true)
- `push`: Optional<bool> - Push to remote (default: true)

**Returns:**
- `SyncRepositoryResponse` - Sync result with pull/push status

**Errors:**
- `RepositoryNotInitialized` - No repository is currently open
- `NoRemoteConfigured` - No remote repository configured
- `SyncFailed` - Git sync operation failed
- `MergeConflict` - Merge conflict detected
- `PermissionDenied` - Insufficient permissions to sync repository

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('sync_repository', {
    pull: true,
    push: true
});

console.log('Pulled:', result.pulled);
console.log('Pushed:', result.pushed);
```

**Requirement Traceability:**
- REQ-DESK-038: Auto-Commit
- REQ-DESK-081: Sync Notifications

#### 4.2.4. Get Repository Status

**Command:** `get_repository_status`

**Description:**
Retrieves the current status of the repository including branch, modified files, and sync status.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GetRepositoryStatusResponse {
    pub repository_id: String,
    pub current_branch: String,
    pub modified_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ahead: u64,
    pub behind: u64,
    pub sync_status: SyncStatus,
}

#[tauri::command]
pub async fn get_repository_status(
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetRepositoryStatusResponse, IpcError> {
    // Get repository status
    let status = state.read().await.core.get_repository_status().await?;

    Ok(GetRepositoryStatusResponse {
        repository_id: status.repository_id,
        current_branch: status.current_branch,
        modified_files: status.modified_files,
        staged_files: status.staged_files,
        untracked_files: status.untracked_files,
        ahead: status.ahead,
        behind: status.behind,
        sync_status: status.sync_status,
    })
}
```

**TypeScript Signature:**
```typescript
interface GetRepositoryStatusResponse {
    repository_id: string;
    current_branch: string;
    modified_files: string[];
    staged_files: string[];
    untracked_files: string[];
    ahead: number;
    behind: number;
    sync_status: SyncStatus;
}

async function getRepositoryStatus(): Promise<GetRepositoryStatusResponse>;
```

**Parameters:**
None

**Returns:**
- `GetRepositoryStatusResponse` - Repository status information

**Errors:**
- `RepositoryNotInitialized` - No repository is currently open
- `GitError` - Git operation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const status = await invoke('get_repository_status');

console.log('Current branch:', status.current_branch);
console.log('Modified files:', status.modified_files);
console.log('Sync status:', status.sync_status);
```

**Requirement Traceability:**
- REQ-DESK-040: History Viewing
- REQ-DESK-030: Status Bar

### 4.3. Search Commands

#### 4.3.1. Search Documents

**Command:** `search_documents`

**Description:**
Searches documents in the current repository using full-text search with optional filters.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchDocumentsRequest {
    pub query: String,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub filters: Option<SearchFilters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFilters {
    pub path_prefix: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchDocumentsResponse {
    pub results: Vec<SearchResult>,
    pub total_count: u64,
    pub query: String,
    pub search_time_ms: u64,
}

#[tauri::command]
pub async fn search_documents(
    request: SearchDocumentsRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SearchDocumentsResponse, IpcError> {
    // Validate query
    if request.query.is_empty() {
        return Err(IpcError::InvalidInput("Query cannot be empty".to_string()));
    }

    // Set defaults
    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(20).min(100); // Max 100

    // Search documents
    let start = std::time::Instant::now();
    let (results, total_count) = state.read().await.core.search_documents(
        &request.query,
        offset,
        limit,
        request.filters.as_ref(),
    ).await?;
    let search_time_ms = start.elapsed().as_millis() as u64;

    Ok(SearchDocumentsResponse {
        results,
        total_count,
        query: request.query,
        search_time_ms,
    })
}
```

**TypeScript Signature:**
```typescript
interface SearchDocumentsRequest {
    query: string;
    offset?: number;
    limit?: number;
    filters?: SearchFilters;
}

interface SearchFilters {
    path_prefix?: string;
    file_types?: string[];
    modified_after?: string;
    modified_before?: string;
}

interface SearchDocumentsResponse {
    results: SearchResult[];
    total_count: number;
    query: string;
    search_time_ms: number;
}

async function searchDocuments(request: SearchDocumentsRequest): Promise<SearchDocumentsResponse>;
```

**Parameters:**
- `query`: String - Search query
- `offset`: Optional<u64> - Pagination offset (default: 0)
- `limit`: Optional<u64> - Pagination limit (default: 20, max: 100)
- `filters`: Optional<SearchFilters> - Search filters

**Returns:**
- `SearchDocumentsResponse` - Search results with metadata

**Errors:**
- `InvalidInput` - Query validation failed
- `RepositoryNotInitialized` - No repository is currently open
- `SearchError` - Search operation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('search_documents', {
    query: 'Tachyon architecture',
    offset: 0,
    limit: 20,
    filters: {
        path_prefix: 'docs/',
        file_types: ['.md']
    }
});

console.log('Found', result.total_count, 'results in', result.search_time_ms, 'ms');
```

**Requirement Traceability:**
- REQ-DESK-020: Quick Open
- REQ-DESK-088: Large File Handling

### 4.4. System Commands

#### 4.4.1. Get Application Status

**Command:** `get_app_status`

**Description:**
Retrieves the current application status including version, repository state, and system information.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GetAppStatusResponse {
    pub app_version: String,
    pub tauri_version: String,
    pub repository_open: bool,
    pub repository_path: Option<String>,
    pub current_branch: Option<String>,
    pub sync_status: SyncStatus,
    pub cache_stats: CacheStatistics,
}

#[tauri::command]
pub async fn get_app_status(
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetAppStatusResponse, IpcError> {
    let state_read = state.read().await;

    Ok(GetAppStatusResponse {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        tauri_version: tauri::get_version().await?,
        repository_open: state_read.repository_path.is_some(),
        repository_path: state_read.repository_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        current_branch: state_read.current_branch.clone(),
        sync_status: state_read.sync_status,
        cache_stats: state_read.cache_stats.clone(),
    })
}
```

**TypeScript Signature:**
```typescript
interface GetAppStatusResponse {
    app_version: string;
    tauri_version: string;
    repository_open: boolean;
    repository_path?: string;
    current_branch?: string;
    sync_status: SyncStatus;
    cache_stats: CacheStatistics;
}

async function getAppStatus(): Promise<GetAppStatusResponse>;
```

**Parameters:**
None

**Returns:**
- `GetAppStatusResponse` - Application status information

**Errors:**
None

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const status = await invoke('get_app_status');

console.log('App version:', status.app_version);
console.log('Repository open:', status.repository_open);
```

**Requirement Traceability:**
- REQ-DESK-030: Status Bar
- REQ-DESK-047: Server Health Monitoring

#### 4.4.2. Get Configuration

**Command:** `get_config`

**Description:**
Retrieves the application configuration.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GetConfigResponse {
    pub config: serde_json::Value,
}

#[tauri::command]
pub async fn get_config(
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetConfigResponse, IpcError> {
    let config = state.read().await.config.get_all().await?;

    Ok(GetConfigResponse { config })
}
```

**TypeScript Signature:**
```typescript
interface GetConfigResponse {
    config: Record<string, any>;
}

async function getConfig(): Promise<GetConfigResponse>;
```

**Parameters:**
None

**Returns:**
- `GetConfigResponse` - Application configuration

**Errors:**
- `ConfigError` - Configuration retrieval failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('get_config');

console.log('Editor font size:', result.config.editor_font_size);
```

**Requirement Traceability:**
- REQ-DESK-026: Main Menu Bar
- REQ-DESK-029: Command Palette

#### 4.4.3. Set Configuration

**Command:** `set_config`

**Description:**
Updates application configuration values.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SetConfigRequest {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetConfigResponse {
    pub key: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn set_config(
    request: SetConfigRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SetConfigResponse, IpcError> {
    // Validate key
    if request.key.is_empty() {
        return Err(IpcError::InvalidInput("Key cannot be empty".to_string()));
    }

    // Set configuration
    state.write().await.config.set(&request.key, &request.value).await?;

    Ok(SetConfigResponse {
        key: request.key,
        updated_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface SetConfigRequest {
    key: string;
    value: any;
}

interface SetConfigResponse {
    key: string;
    updated_at: string;
}

async function setConfig(request: SetConfigRequest): Promise<SetConfigResponse>;
```

**Parameters:**
- `key`: String - Configuration key
- `value`: serde_json::Value - Configuration value

**Returns:**
- `SetConfigResponse` - Configuration update result

**Errors:**
- `InvalidInput` - Key validation failed
- `ConfigError` - Configuration update failed
- `PermissionDenied` - Insufficient permissions to update configuration

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('set_config', {
    key: 'editor_font_size',
    value: 14
});

console.log('Configuration updated:', result.key);
```

**Requirement Traceability:**
- REQ-DESK-026: Main Menu Bar
- REQ-DESK-062: Settings Sync

#### 4.4.4. Clear Cache

**Command:** `clear_cache`

**Description:**
Clears the application cache and returns cache statistics.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ClearCacheResponse {
    pub cache_stats: CacheStatistics,
    pub cleared_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn clear_cache(
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<ClearCacheResponse, IpcError> {
    // Clear cache
    let cache_stats = state.write().await.cache.clear().await?;

    Ok(ClearCacheResponse {
        cache_stats,
        cleared_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface ClearCacheResponse {
    cache_stats: CacheStatistics;
    cleared_at: string;
}

async function clearCache(): Promise<ClearCacheResponse>;
```

**Parameters:**
None

**Returns:**
- `ClearCacheResponse` - Cache clear result with statistics

**Errors:**
- `CacheError` - Cache clear operation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('clear_cache');

console.log('Cache cleared:', result.cleared_at);
console.log('Cache size:', result.cache_stats.size_bytes);
```

**Requirement Traceability:**
- REQ-DESK-041: LRU Cache
- REQ-DESK-045: Manual Cache Clear

#### 4.4.5. Quit Application

**Command:** `quit_application`

**Description:**
Initiates graceful shutdown of the application.

**Rust Signature:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct QuitApplicationResponse {
    pub quitting: bool,
    pub quit_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn quit_application(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<QuitApplicationResponse, IpcError> {
    // Perform graceful shutdown
    state.write().await.shutdown().await?;

    // Quit application
    app_handle.exit(0);

    Ok(QuitApplicationResponse {
        quitting: true,
        quit_at: chrono::Utc::now(),
    })
}
```

**TypeScript Signature:**
```typescript
interface QuitApplicationResponse {
    quitting: boolean;
    quit_at: string;
}

async function quitApplication(): Promise<QuitApplicationResponse>;
```

**Parameters:**
None

**Returns:**
- `QuitApplicationResponse` - Quit confirmation

**Errors:**
- `ShutdownError` - Shutdown operation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('quit_application');

console.log('Application quitting:', result.quitting);
```

**Requirement Traceability:**
- REQ-DESK-006: Graceful Shutdown
- REQ-DESK-007: Auto-Save on Close

### 4.5. Type Definitions

#### 4.5.1. Common Types

**Document Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}
```

```typescript
interface Document {
    id: string;
    title: string;
    content: string;
    path: string;
    created_at: string;
    updated_at: string;
    version: number;
}
```

**DocumentSummary Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: String,
    pub title: String,
    pub path: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub word_count: u64,
}
```

```typescript
interface DocumentSummary {
    id: string;
    title: string;
    path: string;
    updated_at: string;
    word_count: number;
}
```

**DocumentMetadata Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub word_count: u64,
    pub character_count: u64,
    pub reading_time_minutes: u64,
    pub tags: Vec<String>,
}
```

```typescript
interface DocumentMetadata {
    word_count: number;
    character_count: number;
    reading_time_minutes: number;
    tags: string[];
}
```

**SearchResult Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub title: String,
    pub path: String,
    pub snippet: String,
    pub score: f64,
}
```

```typescript
interface SearchResult {
    document_id: string;
    title: string;
    path: string;
    snippet: string;
    score: number;
}
```

**SyncStatus Type:**
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error(String),
}
```

```typescript
type SyncStatus =
    | { type: 'Idle' }
    | { type: 'Syncing' }
    | { type: 'Error'; message: string };
```

**CacheStatistics Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub entry_count: u64,
    pub size_bytes: u64,
    pub hit_rate: f64,
    pub last_cleared: Option<chrono::DateTime<chrono::Utc>>,
}
```

```typescript
interface CacheStatistics {
    entry_count: number;
    size_bytes: number;
    hit_rate: number;
    last_cleared?: string;
}
```

#### 4.5.2. Error Types

**IpcError Type:**
```rust
#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Repository not initialized")]
    RepositoryNotInitialized,

    #[error("Repository already exists")]
    RepositoryAlreadyExists,

    #[error("No remote configured")]
    NoRemoteConfigured,

    #[error("Sync failed: {0}")]
    SyncFailed(String),

    #[error("Merge conflict detected")]
    MergeConflict,

    #[error("Version conflict: document={0}, expected={1}")]
    VersionConflict(String, u64),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
```

```typescript
type IpcError =
    | { type: 'InvalidInput'; message: string }
    | { type: 'DocumentNotFound'; id: string }
    | { type: 'RepositoryNotInitialized' }
    | { type: 'RepositoryAlreadyExists' }
    | { type: 'NoRemoteConfigured' }
    | { type: 'SyncFailed'; message: string }
    | { type: 'MergeConflict' }
    | { type: 'VersionConflict'; document_id: string; expected_version: number }
    | { type: 'FileSystemError'; message: string }
    | { type: 'PermissionDenied' }
    | { type: 'ConfigError'; message: string }
    | { type: 'CacheError'; message: string }
    | { type: 'InternalError'; message: string };
```

---

## 5. DESKTOP EVENTS API

This section defines all IPC events for desktop-specific notifications, including document changes, repository updates, and system events.

### 5.1. Document Events

#### 5.1.1. Document Created

**Event:** `document_created`

**Description:**
Emitted when a new document is created in the current repository.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreatedEvent {
    pub document_id: String,
    pub title: String,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

```typescript
interface DocumentCreatedEvent {
    document_id: string;
    title: string;
    path: string;
    created_at: string;
}
```

**Emission Conditions:**
- Document successfully created via `create_document` command
- Document successfully imported from external file

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<DocumentCreatedEvent>('document_created', (event) => {
    console.log('Document created:', event.payload.document_id);
    console.log('Title:', event.payload.title);
    console.log('Path:', event.payload.path);
    console.log('Created at:', event.payload.created_at);
});
```

**Requirement Traceability:**
- REQ-DESK-031: File Watching
- REQ-IPC-031: Event Emission

#### 5.1.2. Document Updated

**Event:** `document_updated`

**Description:**
Emitted when an existing document is updated.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpdatedEvent {
    pub document_id: String,
    pub title: String,
    pub path: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}
```

```typescript
interface DocumentUpdatedEvent {
    document_id: string;
    title: string;
    path: string;
    updated_at: string;
    version: number;
}
```

**Emission Conditions:**
- Document successfully updated via `update_document` command
- Document modified by external editor (file watcher detected)

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<DocumentUpdatedEvent>('document_updated', (event) => {
    console.log('Document updated:', event.payload.document_id);
    console.log('New version:', event.payload.version);
    console.log('Updated at:', event.payload.updated_at);
});
```

**Requirement Traceability:**
- REQ-DESK-032: External Editor Sync
- REQ-IPC-031: Event Emission

#### 5.1.3. Document Deleted

**Event:** `document_deleted`

**Description:**
Emitted when a document is deleted from the current repository.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDeletedEvent {
    pub document_id: String,
    pub title: String,
    pub path: String,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}
```

```typescript
interface DocumentDeletedEvent {
    document_id: string;
    title: string;
    path: string;
    deleted_at: string;
}
```

**Emission Conditions:**
- Document successfully deleted via `delete_document` command
- Document deleted by external file system operation

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<DocumentDeletedEvent>('document_deleted', (event) => {
    console.log('Document deleted:', event.payload.document_id);
    console.log('Title:', event.payload.title);
    console.log('Deleted at:', event.payload.deleted_at);
});
```

**Requirement Traceability:**
- REQ-DESK-031: File Watching
- REQ-IPC-031: Event Emission

#### 5.1.4. Document Synced

**Event:** `document_synced`

**Description:**
Emitted when a document is synchronized with remote repository.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSyncedEvent {
    pub document_id: String,
    pub sync_type: SyncType,
    pub synced_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncType {
    Pushed,
    Pulled,
    Merged,
}
```

```typescript
interface DocumentSyncedEvent {
    document_id: string;
    sync_type: 'Pushed' | 'Pulled' | 'Merged';
    synced_at: string;
}
```

**Emission Conditions:**
- Document successfully pushed to remote repository
- Document successfully pulled from remote repository
- Document successfully merged with remote changes

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<DocumentSyncedEvent>('document_synced', (event) => {
    console.log('Document synced:', event.payload.document_id);
    console.log('Sync type:', event.payload.sync_type);
    console.log('Synced at:', event.payload.synced_at);
});
```

**Requirement Traceability:**
- REQ-DESK-038: Auto-Commit
- REQ-DESK-081: Sync Notifications

### 5.2. Repository Events

#### 5.2.1. Repository Added

**Event:** `repository_added`

**Description:**
Emitted when a new repository is added or initialized.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAddedEvent {
    pub repository_id: String,
    pub path: String,
    pub is_new: bool,
    pub added_at: chrono::DateTime<chrono::Utc>,
}
```

```typescript
interface RepositoryAddedEvent {
    repository_id: string;
    path: string;
    is_new: boolean;
    added_at: string;
}
```

**Emission Conditions:**
- Repository successfully initialized via `init_repository` command
- Repository successfully cloned via `clone_repository` command

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<RepositoryAddedEvent>('repository_added', (event) => {
    console.log('Repository added:', event.payload.repository_id);
    console.log('Path:', event.payload.path);
    console.log('Is new:', event.payload.is_new);
});
```

**Requirement Traceability:**
- REQ-DESK-036: Repository Initialization
- REQ-DESK-037: Repository Cloning

#### 5.2.2. Repository Removed

**Event:** `repository_removed`

**Description:**
Emitted when a repository is removed or closed.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRemovedEvent {
    pub repository_id: String,
    pub path: String,
    pub removed_at: chrono::DateTime<chrono::Utc>,
}
```

```typescript
interface RepositoryRemovedEvent {
    repository_id: string;
    path: string;
    removed_at: string;
}
```

**Emission Conditions:**
- Repository successfully closed
- Repository successfully removed from application

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<RepositoryRemovedEvent>('repository_removed', (event) => {
    console.log('Repository removed:', event.payload.repository_id);
    console.log('Path:', event.payload.path);
    console.log('Removed at:', event.payload.removed_at);
});
```

**Requirement Traceability:**
- REQ-DESK-006: Graceful Shutdown

#### 5.2.3. Sync Started

**Event:** `sync_started`

**Description:**
Emitted when repository synchronization operation is started.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStartedEvent {
    pub repository_id: String,
    pub operation: SyncOperation,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncOperation {
    Pull,
    Push,
    Both,
}
```

```typescript
interface SyncStartedEvent {
    repository_id: string;
    operation: 'Pull' | 'Push' | 'Both';
    started_at: string;
}
```

**Emission Conditions:**
- Pull operation started via `sync_repository` command
- Push operation started via `sync_repository` command
- Both pull and push operations started

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<SyncStartedEvent>('sync_started', (event) => {
    console.log('Sync started:', event.payload.repository_id);
    console.log('Operation:', event.payload.operation);
    console.log('Started at:', event.payload.started_at);
});
```

**Requirement Traceability:**
- REQ-DESK-081: Sync Notifications

#### 5.2.4. Sync Completed

**Event:** `sync_completed`

**Description:**
Emitted when repository synchronization operation is completed.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCompletedEvent {
    pub repository_id: String,
    pub operation: SyncOperation,
    pub success: bool,
    pub error_message: Option<String>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
}
```

```typescript
interface SyncCompletedEvent {
    repository_id: string;
    operation: 'Pull' | 'Push' | 'Both';
    success: boolean;
    error_message?: string;
    completed_at: string;
    duration_ms: number;
}
```

**Emission Conditions:**
- Pull operation completed successfully or with error
- Push operation completed successfully or with error
- Both pull and push operations completed

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<SyncCompletedEvent>('sync_completed', (event) => {
    console.log('Sync completed:', event.payload.repository_id);
    console.log('Success:', event.payload.success);
    console.log('Duration:', event.payload.duration_ms, 'ms');
    if (!event.payload.success) {
        console.error('Error:', event.payload.error_message);
    }
});
```

**Requirement Traceability:**
- REQ-DESK-081: Sync Notifications
- REQ-DESK-082: Error Notifications

#### 5.2.5. Repository Status Changed

**Event:** `repository_status_changed`

**Description:**
Emitted when repository status changes (branch, modified files, sync status).

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatusChangedEvent {
    pub repository_id: String,
    pub current_branch: String,
    pub modified_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ahead: u64,
    pub behind: u64,
    pub sync_status: SyncStatus,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}
```

```typescript
interface RepositoryStatusChangedEvent {
    repository_id: string;
    current_branch: string;
    modified_files: string[];
    staged_files: string[];
    untracked_files: string[];
    ahead: number;
    behind: number;
    sync_status: SyncStatus;
    changed_at: string;
}
```

**Emission Conditions:**
- Git status changes detected by file watcher
- Branch switch operation completed
- Sync operation completed

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<RepositoryStatusChangedEvent>('repository_status_changed', (event) => {
    console.log('Branch:', event.payload.current_branch);
    console.log('Modified files:', event.payload.modified_files);
    console.log('Ahead:', event.payload.ahead, 'Behind:', event.payload.behind);
    console.log('Sync status:', event.payload.sync_status);
});
```

**Requirement Traceability:**
- REQ-DESK-040: History Viewing
- REQ-DESK-030: Status Bar

### 5.3. System Events

#### 5.3.1. Application Initialized

**Event:** `application_initialized`

**Description:**
Emitted when application initialization is complete.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInitializedEvent {
    pub app_version: String,
    pub tauri_version: String,
    pub repository_open: bool,
    pub initialized_at: chrono::DateTime<chrono::Utc>,
}
```

```typescript
interface ApplicationInitializedEvent {
    app_version: string;
    tauri_version: string;
    repository_open: boolean;
    initialized_at: string;
}
```

**Emission Conditions:**
- Application startup completed
- All components initialized successfully

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<ApplicationInitializedEvent>('application_initialized', (event) => {
    console.log('App initialized:', event.payload.app_version);
    console.log('Tauri version:', event.payload.tauri_version);
    console.log('Repository open:', event.payload.repository_open);
});
```

**Requirement Traceability:**
- REQ-DESK-001: Application Startup
- REQ-DESK-002: Single Instance

#### 5.3.2. Error Occurred

**Event:** `error_occurred`

**Description:**
Emitted when an error occurs in the application.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorOccurredEvent {
    pub error_type: ErrorType,
    pub error_message: String,
    pub context: Option<String>,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorType {
    FileSystem,
    Git,
    Network,
    Configuration,
    Cache,
    Internal,
}
```

```typescript
interface ErrorOccurredEvent {
    error_type: 'FileSystem' | 'Git' | 'Network' | 'Configuration' | 'Cache' | 'Internal';
    error_message: string;
    context?: string;
    occurred_at: string;
}
```

**Emission Conditions:**
- File system operation failed
- Git operation failed
- Network operation failed
- Configuration error occurred
- Cache error occurred
- Internal error occurred

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<ErrorOccurredEvent>('error_occurred', (event) => {
    console.error('Error type:', event.payload.error_type);
    console.error('Message:', event.payload.error_message);
    console.error('Context:', event.payload.context);
    console.error('Occurred at:', event.payload.occurred_at);
});
```

**Requirement Traceability:**
- REQ-DESK-082: Error Notifications
- REQ-IPC-029: Error Propagation

#### 5.3.3. Warning Issued

**Event:** `warning_issued`

**Description:**
Emitted when a warning is issued in the application.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningIssuedEvent {
    pub warning_type: WarningType,
    pub warning_message: String,
    pub context: Option<String>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WarningType {
    DeprecatedFeature,
    Performance,
    Configuration,
    ResourceUsage,
}
```

```typescript
interface WarningIssuedEvent {
    warning_type: 'DeprecatedFeature' | 'Performance' | 'Configuration' | 'ResourceUsage';
    warning_message: string;
    context?: string;
    issued_at: string;
}
```

**Emission Conditions:**
- Deprecated feature is used
- Performance degradation detected
- Configuration issue detected
- Resource usage exceeds threshold

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<WarningIssuedEvent>('warning_issued', (event) => {
    console.warn('Warning type:', event.payload.warning_type);
    console.warn('Message:', event.payload.warning_message);
    console.warn('Context:', event.payload.context);
    console.warn('Issued at:', event.payload.issued_at);
});
```

**Requirement Traceability:**
- REQ-DESK-082: Error Notifications
- REQ-DESK-091: Memory Usage

#### 5.3.4. Cache Invalidated

**Event:** `cache_invalidated`

**Description:**
Emitted when cache entries are invalidated.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInvalidatedEvent {
    pub document_id: Option<String>,
    pub reason: String,
    pub invalidated_at: chrono::DateTime<chrono::Utc>,
}
```

```typescript
interface CacheInvalidatedEvent {
    document_id?: string;
    reason: string;
    invalidated_at: string;
}
```

**Emission Conditions:**
- Document is modified
- Git commit is made
- Cache is manually cleared

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<CacheInvalidatedEvent>('cache_invalidated', (event) => {
    console.log('Cache invalidated:', event.payload.reason);
    if (event.payload.document_id) {
        console.log('Document ID:', event.payload.document_id);
    }
    console.log('Invalidated at:', event.payload.invalidated_at);
});
```

**Requirement Traceability:**
- REQ-DESK-042: Cache Invalidation
- REQ-IPC-031: Event Emission

#### 5.3.5. Theme Changed

**Event:** `theme_changed`

**Description:**
Emitted when application theme changes.

**Payload:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeChangedEvent {
    pub theme: Theme,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}
```

```typescript
interface ThemeChangedEvent {
    theme: 'Light' | 'Dark' | 'System';
    changed_at: string;
}
```

**Emission Conditions:**
- User changes theme via settings
- System theme changes (when theme is set to "System")

**Example:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = listen<ThemeChangedEvent>('theme_changed', (event) => {
    console.log('Theme changed to:', event.payload.theme);
    console.log('Changed at:', event.payload.changed_at);
});
```

**Requirement Traceability:**
- REQ-DESK-025: Editor Theming
- REQ-DESK-063: Theme Sync

### 5.4. Event Subscription Patterns

#### 5.4.1. Event Listener Registration

**Pattern:** Register event listener with automatic cleanup.

```typescript
import { listen } from '@tauri-apps/api/event';

function registerEventListener<T>(
    eventName: string,
    callback: (event: TauriEvent<T>) => void
): () => void {
    const unlisten = listen<T>(eventName, callback);
    return unlisten;
}

// Usage
const unlistenDocumentCreated = registerEventListener<DocumentCreatedEvent>(
    'document_created',
    (event) => {
        console.log('Document created:', event.payload);
    }
);

// Cleanup when component unmounts
onCleanup(() => {
    unlistenDocumentCreated();
});
```

#### 5.4.2. Event Filtering

**Pattern:** Filter events based on payload criteria.

```typescript
import { listen } from '@tauri-apps/api/event';

function filterEventListener<T>(
    eventName: string,
    filter: (payload: T) => boolean,
    callback: (event: TauriEvent<T>) => void
): () => void {
    const unlisten = listen<T>(eventName, (event) => {
        if (filter(event.payload)) {
            callback(event);
        }
    });
    return unlisten;
}

// Usage: Filter for specific document
const unlistenMyDocumentUpdated = filterEventListener<DocumentUpdatedEvent>(
    'document_updated',
    (payload) => payload.document_id === 'doc-123',
    (event) => {
        console.log('My document updated:', event.payload);
    }
);
```

#### 5.4.3. Event Debouncing

**Pattern:** Debounce rapid-fire events.

```typescript
import { listen } from '@tauri-apps/api/event';

function debounceEventListener<T>(
    eventName: string,
    delayMs: number,
    callback: (event: TauriEvent<T>) => void
): () => void {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    let lastEvent: TauriEvent<T> | null = null;

    const unlisten = listen<T>(eventName, (event) => {
        lastEvent = event;
        if (timeoutId !== null) {
            clearTimeout(timeoutId);
        }
        timeoutId = setTimeout(() => {
            if (lastEvent !== null) {
                callback(lastEvent);
            }
            timeoutId = null;
            lastEvent = null;
        }, delayMs);
    });

    return () => {
        if (timeoutId !== null) {
            clearTimeout(timeoutId);
        }
        unlisten();
    };
}

// Usage: Debounce document updates (500ms)
const unlistenDocumentUpdated = debounceEventListener<DocumentUpdatedEvent>(
    'document_updated',
    500,
    (event) => {
        console.log('Debounced document update:', event.payload);
    }
);
```

#### 5.4.4. Event Throttling

**Pattern:** Throttle rapid-fire events.

```typescript
import { listen } from '@tauri-apps/api/event';

function throttleEventListener<T>(
    eventName: string,
    intervalMs: number,
    callback: (event: TauriEvent<T>) => void
): () => void {
    let lastCallTime = 0;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    let lastEvent: TauriEvent<T> | null = null;

    const unlisten = listen<T>(eventName, (event) => {
        lastEvent = event;
        const now = Date.now();
        const timeSinceLastCall = now - lastCallTime;

        if (timeSinceLastCall >= intervalMs) {
            lastCallTime = now;
            callback(event);
        } else {
            if (timeoutId !== null) {
                timeoutId = setTimeout(() => {
                    lastCallTime = Date.now();
                    if (lastEvent !== null) {
                        callback(lastEvent);
                    }
                    timeoutId = null;
                }, intervalMs - timeSinceLastCall);
            }
        }
    });

    return () => {
        if (timeoutId !== null) {
            clearTimeout(timeoutId);
        }
        unlisten();
    };
}

// Usage: Throttle cache invalidation events (1000ms)
const unlistenCacheInvalidated = throttleEventListener<CacheInvalidatedEvent>(
    'cache_invalidated',
    1000,
    (event) => {
        console.log('Throttled cache invalidation:', event.payload);
    }
);
```

---

## 6. IPC COMMUNICATION PROTOCOL

This section defines the IPC communication protocol between WebView frontend and Rust backend, including command registration, invocation patterns, event emission, type-safe serialization, error propagation, and security controls.

### 6.1. Tauri IPC Command Registration

#### 6.1.1. Command Registration Pattern

**Principle:** All IPC commands must be registered with Tauri's command system for type-safe invocation.

**Rust Command Registration:**
```rust
use tauri::command;

/// Creates a new document in the current repository.
///
/// # Arguments
///
/// * `request` - Document creation request with title, content, and optional path
///
/// # Returns
///
/// Document creation result with ID, path, and timestamp
///
/// # Errors
///
/// * `InvalidInput` - Title or content validation failed
/// * `RepositoryNotInitialized` - No repository is currently open
/// * `FileSystemError` - File system operation failed
/// * `PermissionDenied` - Insufficient permissions to create document
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Implementation
}
```

**TypeScript Command Invocation:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function createDocument(request: CreateDocumentRequest): Promise<CreateDocumentResponse> {
    return await invoke<CreateDocumentResponse>('create_document', request);
}

// Usage
const result = await createDocument({
    title: 'My Document',
    content: '# Hello World',
    path: 'docs/my-document.md'
});
```

**Requirement Traceability:**
- REQ-IPC-026: Command Registration
- REQ-IPC-027: Command Execution

**ADR Reference:**
- [ADR-009: IPC Communication Architecture](../../specs/02_adrs/009_ipc_communication_architecture.md) - Type Safety section

#### 6.1.2. Command Handler Best Practices

**Best Practice 1: Input Validation**

Validate all inputs before processing to prevent injection attacks and data corruption.

```rust
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Validate title length
    if request.title.is_empty() || request.title.len() > 100 {
        return Err(IpcError::InvalidInput(
            "Title must be 1-100 characters".to_string()
        ));
    }

    // Validate content size
    if request.content.len() > 10 * 1024 * 1024 {
        return Err(IpcError::InvalidInput(
            "Content exceeds 10MB limit".to_string()
        ));
    }

    // Process request
    // ...
}
```

**Best Practice 2: Async Error Handling**

Use async/await for all I/O operations and propagate errors properly.

```rust
#[command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentResponse, IpcError> {
    // Use async I/O operations
    let document = state.read().await.core.get_document(
        request.id.as_deref(),
        request.path.as_deref(),
    ).await.map_err(IpcError::from)?;

    Ok(GetDocumentResponse {
        document,
        rendered_html: state.read().await.core.render_document(&document).await?,
        metadata: state.read().await.core.get_document_metadata(&document.id).await?,
    })
}
```

**Best Practice 3: State Access Pattern**

Use read/write locks appropriately for state access to prevent race conditions.

```rust
#[command]
pub async fn update_document(
    request: UpdateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<UpdateDocumentResponse, IpcError> {
    // Read state for validation
    {
        let state_read = state.read().await;
        if !state_read.repository_path.is_some() {
            return Err(IpcError::RepositoryNotInitialized);
        }
    }

    // Write state for updates
    let (document_id, version) = state.write().await.core.update_document(
        &request.id,
        request.title.as_deref(),
        request.content.as_deref(),
    ).await?;

    Ok(UpdateDocumentResponse {
        document_id,
        updated_at: chrono::Utc::now(),
        version,
    })
}
```

### 6.2. Tauri IPC Event Emission

#### 6.2.1. Event Emission Pattern

**Principle:** All IPC events must be emitted through Tauri's event system with type-safe payloads.

**Rust Event Emission:**
```rust
use tauri::Window;

/// Emits document created event.
///
/// # Arguments
///
/// * `window` - Tauri window handle for event emission
/// * `event` - Document created event payload
fn emit_document_created(
    window: &Window,
    event: DocumentCreatedEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    window.emit("document_created", event)?;
    Ok(())
}

// Usage in command handler
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    window: tauri::Window,
) -> Result<CreateDocumentResponse, IpcError> {
    // Create document
    let document_id = state.write().await.core.create_document(
        &request.title,
        &request.content,
        request.path.as_deref(),
    ).await?;

    // Emit event
    emit_document_created(
        &window,
        DocumentCreatedEvent {
            document_id: document_id.clone(),
            title: request.title.clone(),
            path: request.path.unwrap_or_else(|| "./".to_string()),
            created_at: chrono::Utc::now(),
        },
    )?;

    Ok(CreateDocumentResponse {
        document_id,
        path: request.path.unwrap_or_else(|| "./".to_string()),
        created_at: chrono::Utc::now(),
    })
}
```

**TypeScript Event Subscription:**
```typescript
import { listen } from '@tauri-apps/api/event';

interface DocumentCreatedEvent {
    document_id: string;
    title: string;
    path: string;
    created_at: string;
}

function subscribeToDocumentCreated(
    callback: (event: DocumentCreatedEvent) => void
): () => void {
    const unlisten = listen<DocumentCreatedEvent>('document_created', (event) => {
        callback(event.payload);
    });
    return unlisten;
}

// Usage
const unlisten = subscribeToDocumentCreated((event) => {
    console.log('Document created:', event.document_id);
    console.log('Title:', event.title);
});

// Cleanup when component unmounts
onCleanup(() => {
    unlisten();
});
```

**Requirement Traceability:**
- REQ-IPC-031: Event Emission
- REQ-IPC-032: Event Subscription

**ADR Reference:**
- [ADR-009: IPC Communication Architecture](../../specs/02_adrs/009_ipc_communication_architecture.md) - Bidirectional Communication section

#### 6.2.2. Event Emission Best Practices

**Best Practice 1: Rate Limiting**

Limit event emission rate to prevent overwhelming the frontend.

```rust
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct EventEmitter {
    last_emission: Arc<Mutex<Option<Instant>>>,
    min_interval: Duration,
}

impl EventEmitter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_emission: Arc::new(Mutex::new(None)),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    pub async fn emit<F: serde::Serialize>(
        &self,
        window: &Window,
        event_name: &str,
        payload: &F,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_emission = self.last_emission.lock().await;
        let now = Instant::now();

        if let Some(last) = *last_emission {
            if now.duration_since(last) < self.min_interval {
                return Ok(()); // Rate limited
            }
        }

        *last_emission = Some(now);
        window.emit(event_name, payload)?;
        Ok(())
    }
}
```

**Best Practice 2: Event Payload Sanitization**

Sanitize event payloads before emission to prevent information leakage.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorOccurredEvent {
    pub error_type: ErrorType,
    pub error_message: String,
    pub context: Option<String>, // Redacted sensitive context
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[command]
pub async fn handle_error(
    error: CoreError,
    window: tauri::Window,
) -> Result<(), IpcError> {
    // Redact sensitive information
    let context = match &error {
        CoreError::FileSystemError(path) => {
            Some(format!("Error accessing file: {}", sanitize_path(path)))
        }
        CoreError::GitError(msg) => {
            Some(msg.clone()) // Already sanitized
        }
        _ => None,
    };

    let event = ErrorOccurredEvent {
        error_type: error.error_type(),
        error_message: error.user_friendly_message(),
        context,
        occurred_at: chrono::Utc::now(),
    };

    window.emit("error_occurred", event)?;
    Ok(())
}

fn sanitize_path(path: &str) -> String {
    // Redact sensitive paths
    if path.contains("/home/") {
        path.replace(path, "/home/", "/home/[REDACTED]/")
    } else {
        path.to_string()
    }
}
```

### 6.3. Type-Safe Serialization

#### 6.3.1. Serde Serialization

**Principle:** All IPC payloads must be serializable with serde for type-safe communication.

**Rust Serialization:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub document_id: String,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

**TypeScript Type Generation:**
```typescript
// Generated by Tauri CLI
export interface CreateDocumentRequest {
    title: string;
    content: string;
    path?: string;
}

export interface CreateDocumentResponse {
    document_id: string;
    path: string;
    created_at: string;
}
```

**Type Generation Process:**
```bash
# Generate TypeScript types from Rust types
tauri info --json > tauri-types.json

# Extract types from JSON
jq '.types' tauri-types.json > src/types/tauri.d.ts
```

**Requirement Traceability:**
- REQ-IPC-025: Type-Safe Serialization
- REQ-IPC-032: Type Generation

**ADR Reference:**
- [ADR-009: IPC Communication Architecture](../../specs/02_adrs/009_ipc_communication_architecture.md) - Efficient Serialization section

#### 6.3.2. Serialization Best Practices

**Best Practice 1: Use Strong Typing**

Use specific types instead of generic types for better type safety.

```rust
// Good: Specific types
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentId(String);

// Bad: Generic type
pub type DocumentId = String;
```

**Best Practice 2: Validate Serialization**

Validate that types can be serialized and deserialized correctly.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_serialization() {
        let document = Document {
            id: "doc-123".to_string(),
            title: "Test Document".to_string(),
            content: "# Test".to_string(),
            path: "test.md".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };

        // Serialize
        let serialized = serde_json::to_string(&document).unwrap();

        // Deserialize
        let deserialized: Document = serde_json::from_str(&serialized).unwrap();

        assert_eq!(document.id, deserialized.id);
        assert_eq!(document.title, deserialized.title);
    }
}
```

### 6.4. Error Propagation

#### 6.4.1. Error Propagation Pattern

**Principle:** All errors must be propagated across IPC boundary with appropriate context.

**Rust Error Propagation:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Repository not initialized")]
    RepositoryNotInitialized,

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Internal error: {0}")]
    InternalError(String),
}

// Implement From trait for automatic error conversion
impl From<CoreError> for IpcError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::DocumentNotFound(id) => IpcError::DocumentNotFound(id),
            CoreError::FileSystemError(msg) => IpcError::FileSystemError(msg),
            _ => IpcError::InternalError(error.to_string()),
        }
    }
}

// Command handler with error propagation
#[command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentResponse, IpcError> {
    // Core error automatically converted to IPC error
    let document = state.read().await.core.get_document(&request.id).await?;

    Ok(GetDocumentResponse {
        document,
        rendered_html: state.read().await.core.render_document(&document).await?,
        metadata: state.read().await.core.get_document_metadata(&document.id).await?,
    })
}
```

**TypeScript Error Handling:**
```typescript
interface IpcError {
    type: 'InvalidInput' | 'DocumentNotFound' | 'RepositoryNotInitialized' | 'FileSystemError' | 'PermissionDenied' | 'InternalError';
    message: string;
}

async function getDocument(request: GetDocumentRequest): Promise<GetDocumentResponse> {
    try {
        return await invoke<GetDocumentResponse>('get_document', request);
    } catch (error) {
        const ipcError = error as IpcError;
        console.error('IPC Error:', ipcError.type, ipcError.message);
        throw error;
    }
}
```

**Requirement Traceability:**
- REQ-IPC-029: Error Propagation
- REQ-SEC-018: Fail-Safe Error Handling

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Fail-Safe Error Handling section

#### 6.4.2. Error Handling Best Practices

**Best Practice 1: Provide User-Friendly Messages**

Map technical errors to user-friendly messages.

```rust
impl IpcError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            IpcError::InvalidInput(msg) => {
                format!("Invalid input: {}", msg)
            }
            IpcError::DocumentNotFound(id) => {
                format!("Document '{}' not found", id)
            }
            IpcError::RepositoryNotInitialized => {
                "No repository is currently open. Please open or initialize a repository.".to_string()
            }
            IpcError::FileSystemError(msg) => {
                format!("File system error: {}", msg)
            }
            IpcError::PermissionDenied => {
                "Permission denied. Please check your file permissions.".to_string()
            }
            IpcError::InternalError(msg) => {
                format!("Internal error occurred: {}", msg)
            }
        }
    }
}
```

**Best Practice 2: Log Technical Details**

Log detailed error information for debugging while providing user-friendly messages.

```rust
use tracing::{error, warn};

#[command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentResponse, IpcError> {
    match state.read().await.core.get_document(&request.id).await {
        Ok(document) => Ok(GetDocumentResponse {
            document,
            rendered_html: state.read().await.core.render_document(&document).await?,
            metadata: state.read().await.core.get_document_metadata(&document.id).await?,
        }),
        Err(CoreError::DocumentNotFound(id)) => {
            // Log technical details
            error!(
                document_id = %id,
                error = "Document not found in repository"
            );
            // Return user-friendly error
            Err(IpcError::DocumentNotFound(id))
        }
        Err(error) => {
            // Log technical details
            error!(
                document_id = %request.id,
                error = %error,
                "Failed to retrieve document"
            );
            // Return generic error
            Err(IpcError::InternalError(error.to_string()))
        }
    }
}
```

### 6.5. Security Controls

#### 6.5.1. Capability-Based Authorization

**Principle:** All IPC commands must be protected by Tauri's capability system.

**Capability Configuration:**
```json
{
  "identifier": "default",
  "description": "Default capability set for Tachyon desktop application",
  "windows": ["main"],
  "permissions": [
    {
      "identifier": "fs:read",
      "allow": [
        { "path": "$HOME/Documents" }
      ]
    },
    {
      "identifier": "fs:write",
      "allow": [
        { "path": "$HOME/Documents" }
      ]
    },
    {
      "identifier": "dialog:allow-open",
      "allow": [
        { "title": "Open File" },
        { "title": "Open Folder" }
      ]
    },
    {
      "identifier": "notification:allow-send",
      "allow": []
    }
  ]
}
```

**Capability Enforcement:**
```rust
// Tauri automatically enforces capabilities based on configuration
// Commands requiring fs:read will fail if path is not in $HOME/Documents

#[command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentResponse, IpcError> {
    // If document path is outside $HOME/Documents, Tauri will deny access
    let document = state.read().await.core.get_document(&request.id).await?;
    // ...
}
```

**Requirement Traceability:**
- REQ-IPC-028: Capability-Based Authorization
- REQ-SEC-015: Principle of Least Privilege

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Capability-Based Access Control section

#### 6.5.2. Input Validation and Sanitization

**Principle:** All IPC inputs must be validated and sanitized before processing.

**Input Validation:**
```rust
use validator::ValidateLength;

#[derive(Debug, ValidateLength)]
pub struct DocumentTitle {
    #[validate(length(min = 1, max = 100))]
    pub title: String,
}

#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Validate title
    let title = DocumentTitle {
        title: request.title.clone(),
    };
    if let Err(errors) = title.validate() {
        return Err(IpcError::InvalidInput(
            format!("Invalid title: {}", errors.join(", "))
        ));
    }

    // Validate content size
    if request.content.len() > 10 * 1024 * 1024 {
        return Err(IpcError::InvalidInput(
            "Content exceeds 10MB limit".to_string()
        ));
    }

    // Sanitize path
    let path = request.path
        .map(|p| sanitize_path(&p))
        .unwrap_or_else(|| "./".to_string());

    // Process request
    // ...
}

fn sanitize_path(path: &str) -> String {
    // Prevent path traversal attacks
    let sanitized = path
        .replace("..", "")
        .replace("~", "");
    
    // Ensure path is within repository
    if sanitized.starts_with("/") {
        sanitized = sanitized.trim_start_matches('/');
    }
    
    sanitized
}
```

**Requirement Traceability:**
- REQ-SEC-016: Input Validation
- REQ-SEC-017: Path Sanitization

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Input Validation Layer section

#### 6.5.3. Rate Limiting

**Principle:** All IPC commands must be rate-limited to prevent abuse and DoS attacks.

**Rate Limiting Implementation:**
```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_ms: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_millis(window_ms),
        }
    }

    pub async fn check(&self, command: &str) -> Result<(), RateLimitError> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let key = command.to_string();

        // Clean up old requests
        if let Some(request_times) = requests.get_mut(&key) {
            request_times.retain(|&t| now.duration_since(t) < self.window);
        }

        // Check rate limit
        let request_count = requests.get(&key).map(|v| v.len()).unwrap_or(0);
        if request_count >= self.max_requests {
            return Err(RateLimitError::TooManyRequests);
        }

        // Record request
        requests.entry(key).or_insert_with(Vec::new).push(now);
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Too many requests")]
    TooManyRequests,
}
```

**Rate Limiting Middleware:**
```rust
pub struct RateLimiterMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimiterMiddleware {
    pub fn new(limiter: Arc<RateLimiter>) -> Self {
        Self { limiter }
    }

    pub async fn wrap<F, R>(
        &self,
        command: &str,
        f: F,
    ) -> Result<R, IpcError>
    where
        F: std::future::Future<Output = Result<R, IpcError>>,
    {
        // Check rate limit
        self.limiter.check(command).await
            .map_err(|e| IpcError::RateLimited(e.to_string()))?;

        // Execute command
        f.await
    }
}

// Usage in command handler
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    rate_limiter: State<'_, Arc<RateLimiter>>,
) -> Result<CreateDocumentResponse, IpcError> {
    rate_limiter.wrap("create_document", async {
        // Command implementation
        // ...
    }).await
}
```

**Requirement Traceability:**
- REQ-SEC-019: Rate Limiting
- REQ-SEC-020: DoS Prevention

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Security Controls section

---

## 7. API SECURITY

This section defines security controls for the Desktop API, including authentication, authorization, input validation, rate limiting, and audit logging.

### 7.1. Authentication

#### 7.1.1. Session-Based Authentication

**Principle:** All IPC commands must be authenticated using session-based authentication tokens.

**Authentication Flow:**
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub permissions: Vec<String>,
}

pub struct AuthManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    session_duration: chrono::Duration,
}

impl AuthManager {
    pub fn new(session_duration_hours: i64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_duration: chrono::Duration::hours(session_duration_hours),
        }
    }

    pub async fn create_session(
        &self,
        user_id: Option<String>,
        permissions: Vec<String>,
    ) -> Result<String, AuthError> {
        let session_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let expires_at = now + self.session_duration;

        let session = Session {
            session_id: session_id.clone(),
            user_id,
            created_at: now,
            expires_at,
            permissions,
        };

        self.sessions.write().await.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub async fn validate_session(
        &self,
        session_id: &str,
        required_permission: &str,
    ) -> Result<bool, AuthError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or(AuthError::InvalidSession)?;

        // Check if session is expired
        if session.expires_at < chrono::Utc::now() {
            return Err(AuthError::SessionExpired);
        }

        // Check if session has required permission
        if !session.permissions.contains(&required_permission.to_string()) {
            return Err(AuthError::PermissionDenied);
        }

        Ok(true)
    }
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid session")]
    InvalidSession,

    #[error("Session expired")]
    SessionExpired,

    #[error("Permission denied")]
    PermissionDenied,
}
```

**Session Middleware:**
```rust
pub struct AuthMiddleware {
    auth_manager: Arc<AuthManager>,
}

impl AuthMiddleware {
    pub fn new(auth_manager: Arc<AuthManager>) -> Self {
        Self { auth_manager }
    }

    pub async fn wrap<F, R>(
        &self,
        session_id: Option<String>,
        required_permission: &str,
        f: F,
    ) -> Result<R, IpcError>
    where
        F: std::future::Future<Output = Result<R, IpcError>>,
    {
        // Validate session if provided
        if let Some(sid) = session_id {
            self.auth_manager.validate_session(&sid, required_permission).await
                .map_err(|e| IpcError::AuthenticationFailed(e.to_string()))?;
        }

        // Execute command
        f.await
    }
}

// Usage in command handler
#[command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    auth_manager: State<'_, Arc<AuthManager>>,
    session: Option<String>,
) -> Result<GetDocumentResponse, IpcError> {
    auth_manager.wrap(session, "document:read", async {
        // Command implementation
        let document = state.read().await.core.get_document(&request.id).await?;
        Ok(GetDocumentResponse {
            document,
            rendered_html: state.read().await.core.render_document(&document).await?,
            metadata: state.read().await.core.get_document_metadata(&document.id).await?,
        })
    }).await
}
```

**Requirement Traceability:**
- REQ-SEC-001: Session-Based Authentication
- REQ-SEC-002: Session Management

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Defense-in-Depth Strategy section

#### 7.1.2. Session Management

**Principle:** Sessions must be managed with automatic cleanup and expiration.

**Session Cleanup:**
```rust
use tokio::time::{interval, Duration};

impl AuthManager {
    pub fn start_session_cleanup(&self) {
        let auth_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes

            loop {
                interval.tick().await;

                // Clean up expired sessions
                auth_manager.cleanup_expired_sessions().await;
            }
        });
    }

    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now();

        sessions.retain(|_, session| session.expires_at > now);
    }
}
```

**Requirement Traceability:**
- REQ-SEC-003: Session Cleanup
- REQ-SEC-004: Session Expiration

### 7.2. Authorization

#### 7.2.1. Permission-Based Authorization

**Principle:** All IPC commands must be authorized based on user permissions.

**Permission System:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // Document permissions
    DocumentRead,
    DocumentWrite,
    DocumentDelete,

    // Repository permissions
    RepositoryRead,
    RepositoryWrite,
    RepositorySync,

    // System permissions
    SystemConfig,
    SystemAdmin,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::DocumentRead => "document:read",
            Permission::DocumentWrite => "document:write",
            Permission::DocumentDelete => "document:delete",
            Permission::RepositoryRead => "repository:read",
            Permission::RepositoryWrite => "repository:write",
            Permission::RepositorySync => "repository:sync",
            Permission::SystemConfig => "system:config",
            Permission::SystemAdmin => "system:admin",
        }
    }
}

pub struct PermissionManager {
    user_permissions: Arc<RwLock<HashMap<String, Vec<Permission>>>>,
}

impl PermissionManager {
    pub async fn check_permission(
        &self,
        user_id: &str,
        permission: Permission,
    ) -> Result<bool, AuthError> {
        let permissions = self.user_permissions.read().await;
        let user_permissions = permissions.get(user_id)
            .ok_or(AuthError::UserNotFound)?;

        Ok(user_permissions.contains(&permission))
    }

    pub async fn grant_permission(
        &self,
        user_id: String,
        permission: Permission,
    ) -> Result<(), AuthError> {
        let mut permissions = self.user_permissions.write().await;
        let user_permissions = permissions.entry(user_id).or_insert_with(Vec::new());

        if !user_permissions.contains(&permission) {
            user_permissions.push(permission);
        }

        Ok(())
    }

    pub async fn revoke_permission(
        &self,
        user_id: String,
        permission: Permission,
    ) -> Result<(), AuthError> {
        let mut permissions = self.user_permissions.write().await;
        let user_permissions = permissions.get_mut(&user_id)
            .ok_or(AuthError::UserNotFound)?;

        user_permissions.retain(|p| p != &permission);
        Ok(())
    }
}
```

**Permission Middleware:**
```rust
pub struct PermissionMiddleware {
    permission_manager: Arc<PermissionManager>,
}

impl PermissionMiddleware {
    pub fn new(permission_manager: Arc<PermissionManager>) -> Self {
        Self { permission_manager }
    }

    pub async fn wrap<F, R>(
        &self,
        user_id: Option<String>,
        required_permission: Permission,
        f: F,
    ) -> Result<R, IpcError>
    where
        F: std::future::Future<Output = Result<R, IpcError>>,
    {
        // Check permission if user_id provided
        if let Some(uid) = user_id {
            self.permission_manager.check_permission(&uid, required_permission).await
                .map_err(|e| IpcError::AuthorizationFailed(e.to_string()))?;
        }

        // Execute command
        f.await
    }
}

// Usage in command handler
#[command]
pub async fn delete_document(
    request: DeleteDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    permission_manager: State<'_, Arc<PermissionManager>>,
    user_id: Option<String>,
) -> Result<DeleteDocumentResponse, IpcError> {
    permission_manager.wrap(user_id, Permission::DocumentDelete, async {
        // Command implementation
        state.write().await.core.delete_document(&request.id).await?;
        Ok(DeleteDocumentResponse {
            document_id: request.id,
            deleted_at: chrono::Utc::now(),
        })
    }).await
}
```

**Requirement Traceability:**
- REQ-SEC-005: Permission-Based Authorization
- REQ-SEC-006: Permission Management

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Capability-Based Access Control section

### 7.3. Input Validation

#### 7.3.1. Input Sanitization

**Principle:** All IPC inputs must be sanitized to prevent injection attacks.

**Path Sanitization:**
```rust
pub fn sanitize_path(path: &str) -> Result<String, SanitizationError> {
    // Prevent path traversal attacks
    let sanitized = path
        .replace("..", "")
        .replace("~", "")
        .trim();

    // Validate path doesn't contain invalid characters
    if sanitized.contains(|c| c.is_control() && c != '/') {
        return Err(SanitizationError::InvalidCharacters);
    }

    // Ensure path is absolute or relative
    if sanitized.starts_with("/") || sanitized.starts_with("./") {
        Ok(sanitized)
    } else {
        Ok(format!("./{}", sanitized))
    }
}

#[derive(Error, Debug)]
pub enum SanitizationError {
    #[error("Invalid characters in path")]
    InvalidCharacters,
}
```

**String Sanitization:**
```rust
pub fn sanitize_string(input: &str, max_length: usize) -> Result<String, SanitizationError> {
    // Remove null bytes
    let sanitized: String = input.chars()
        .filter(|c| !c.is_control())
        .collect();

    // Trim whitespace
    let sanitized = sanitized.trim();

    // Validate length
    if sanitized.len() > max_length {
        return Err(SanitizationError::TooLong);
    }

    Ok(sanitized)
}
```

**Input Validation Middleware:**
```rust
pub struct ValidationMiddleware;

impl ValidationMiddleware {
    pub async fn wrap<F, R>(
        f: F,
    ) -> Result<R, IpcError>
    where
        F: std::future::Future<Output = Result<R, IpcError>>,
    {
        // Execute command with validation
        f.await
    }
}

// Usage in command handler
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Validate title
    let title = sanitize_string(&request.title, 100)
        .map_err(|e| IpcError::InvalidInput(e.to_string()))?;

    // Validate content size
    if request.content.len() > 10 * 1024 * 1024 {
        return Err(IpcError::InvalidInput(
            "Content exceeds 10MB limit".to_string()
        ));
    }

    // Sanitize path if provided
    let path = if let Some(p) = request.path.as_ref() {
        Some(sanitize_path(p).map_err(|e| IpcError::InvalidInput(e.to_string()))?)
    } else {
        None
    };

    // Process request
    let document_id = state.write().await.core.create_document(
        &title,
        &request.content,
        path.as_deref(),
    ).await?;

    Ok(CreateDocumentResponse {
        document_id,
        path: path.unwrap_or_else(|| "./".to_string()),
        created_at: chrono::Utc::now(),
    })
}
```

**Requirement Traceability:**
- REQ-SEC-007: Input Sanitization
- REQ-SEC-008: Path Validation

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Input Validation Layer section

#### 7.3.2. Type Validation

**Principle:** All IPC inputs must be validated against type constraints.

**Type Validation:**
```rust
use validator::ValidateLength;

#[derive(Debug, ValidateLength)]
pub struct DocumentTitle {
    #[validate(length(min = 1, max = 100))]
    pub title: String,
}

#[derive(Debug, ValidateLength)]
pub struct DocumentContent {
    #[validate(length(max = 10 * 1024 * 1024))] // 10MB
    pub content: String,
}

#[derive(Debug, ValidateLength)]
pub struct DocumentPath {
    #[validate(length(max = 4096))] // Max path length
    pub path: String,
}

// Usage in command handler
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Validate title
    let title = DocumentTitle {
        title: request.title.clone(),
    };
    if let Err(errors) = title.validate() {
        return Err(IpcError::InvalidInput(
            format!("Invalid title: {}", errors.join(", "))
        ));
    }

    // Validate content
    let content = DocumentContent {
        content: request.content.clone(),
    };
    if let Err(errors) = content.validate() {
        return Err(IpcError::InvalidInput(
            format!("Invalid content: {}", errors.join(", "))
        ));
    }

    // Validate path if provided
    if let Some(path) = request.path.as_ref() {
        let document_path = DocumentPath {
            path: path.clone(),
        };
        if let Err(errors) = document_path.validate() {
            return Err(IpcError::InvalidInput(
                format!("Invalid path: {}", errors.join(", "))
            ));
        }
    }

    // Process request
    // ...
}
```

**Requirement Traceability:**
- REQ-SEC-009: Type Validation
- REQ-SEC-010: Length Constraints

### 7.4. Rate Limiting

#### 7.4.1. Command Rate Limiting

**Principle:** All IPC commands must be rate-limited to prevent abuse and DoS attacks.

**Rate Limiting Implementation:**
```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_ms: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_millis(window_ms),
        }
    }

    pub async fn check(&self, command: &str) -> Result<(), RateLimitError> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let key = command.to_string();

        // Clean up old requests
        if let Some(request_times) = requests.get_mut(&key) {
            request_times.retain(|&t| now.duration_since(t) < self.window);
        }

        // Check rate limit
        let request_count = requests.get(&key).map(|v| v.len()).unwrap_or(0);
        if request_count >= self.max_requests {
            return Err(RateLimitError::TooManyRequests);
        }

        // Record request
        requests.entry(key).or_insert_with(Vec::new).push(now);
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Too many requests")]
    TooManyRequests,
}
```

**Rate Limiting Middleware:**
```rust
pub struct RateLimiterMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimiterMiddleware {
    pub fn new(limiter: Arc<RateLimiter>) -> Self {
        Self { limiter }
    }

    pub async fn wrap<F, R>(
        &self,
        command: &str,
        f: F,
    ) -> Result<R, IpcError>
    where
        F: std::future::Future<Output = Result<R, IpcError>>,
    {
        // Check rate limit
        self.limiter.check(command).await
            .map_err(|e| IpcError::RateLimited(e.to_string()))?;

        // Execute command
        f.await
    }
}

// Usage in command handler
#[command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    rate_limiter: State<'_, Arc<RateLimiter>>,
) -> Result<CreateDocumentResponse, IpcError> {
    rate_limiter.wrap("create_document", async {
        // Command implementation
        let document_id = state.write().await.core.create_document(
            &request.title,
            &request.content,
            request.path.as_deref(),
        ).await?;

        Ok(CreateDocumentResponse {
            document_id,
            path: request.path.unwrap_or_else(|| "./".to_string()),
            created_at: chrono::Utc::now(),
        })
    }).await
}
```

**Requirement Traceability:**
- REQ-SEC-011: Rate Limiting
- REQ-SEC-012: DoS Prevention

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Security Controls section

#### 7.4.2. Event Rate Limiting

**Principle:** All IPC events must be rate-limited to prevent overwhelming the frontend.

**Event Rate Limiting Implementation:**
```rust
pub struct EventEmitter {
    last_emission: Arc<Mutex<HashMap<String, Instant>>>,
    min_interval: Duration,
}

impl EventEmitter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_emission: Arc::new(Mutex::new(HashMap::new())),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    pub async fn emit<F: serde::Serialize>(
        &self,
        window: &Window,
        event_name: &str,
        payload: &F,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_emission = self.last_emission.lock().await;
        let now = Instant::now();
        let key = event_name.to_string();

        // Check rate limit
        if let Some(last) = last_emission.get(&key) {
            if now.duration_since(last) < self.min_interval {
                return Ok(()); // Rate limited
            }
        }

        *last_emission = Some(now);
        window.emit(event_name, payload)?;
        Ok(())
    }
}
```

**Requirement Traceability:**
- REQ-SEC-013: Event Rate Limiting
- REQ-SEC-014: Frontend Protection

### 7.5. Audit Logging

#### 7.5.1. Security Event Logging

**Principle:** All security-relevant events must be logged for audit trail.

**Audit Logging:**
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn get_document(
    id: String,
    user: User,
) -> Result<Document, ApiError> {
    info!(user_id = %user.id, document_id = %id);

    if !user.can_access_document(&id) {
        warn!(user_id = %user.id, document_id = %id, action = "access_denied");
        return Err(ApiError::PermissionDenied);
    }

    let document = fetch_document(&id).await?;
    info!(user_id = %user.id, document_id = %id, action = "document_retrieved");

    Ok(document)
}
```

**Audit Log Format:**
```json
{
  "timestamp": "2026-02-05T12:00:00Z",
  "level": "info",
  "user_id": "user-123",
  "document_id": "doc-456",
  "action": "document_retrieved",
  "ip_address": "127.0.0.1",
  "session_id": "session-789"
}
```

**Requirement Traceability:**
- REQ-SEC-015: Audit Logging
- REQ-SEC-016: Security Event Tracking

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Audit Logging Layer section

#### 7.5.2. Error Logging

**Principle:** All errors must be logged with appropriate context for debugging.

**Error Logging:**
```rust
use tracing::{error, instrument};

#[instrument(skip(self))]
pub async fn get_document(
    id: String,
    user: User,
) -> Result<Document, ApiError> {
    match fetch_document(&id).await {
        Ok(document) => Ok(document),
        Err(CoreError::DocumentNotFound(id)) => {
            error!(
                user_id = %user.id,
                document_id = %id,
                error = "Document not found in repository"
            );
            Err(ApiError::DocumentNotFound(id))
        }
        Err(error) => {
            error!(
                user_id = %user.id,
                document_id = %id,
                error = %error,
                "Failed to retrieve document"
            );
            Err(ApiError::InternalError(error.to_string()))
        }
    }
}
```

**Requirement Traceability:**
- REQ-SEC-017: Error Logging
- REQ-SEC-018: Error Context

### 7.6. Data Protection

#### 7.6.1. Sensitive Data Handling

**Principle:** Sensitive data must be protected from unauthorized access and leakage.

**Sensitive Data Protection:**
```rust
use secrecy::Secret;

pub struct SecureConfig {
    #[serde(skip_serializing)]
    api_key: Secret<String>,
    #[serde(skip_serializing)]
    database_password: Secret<String>,
}

impl SecureConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            api_key: Secret::new(std::env::var("API_KEY").unwrap_or_default()),
            database_password: Secret::new(std::env::var("DB_PASSWORD").unwrap_or_default()),
        })
    }
}

// Usage
#[command]
pub async fn sync_repository(
    request: SyncRepositoryRequest,
    config: State<'_, SecureConfig>,
) -> Result<SyncRepositoryResponse, IpcError> {
    // Access sensitive data securely
    let api_key = config.api_key.expose();
    let db_password = config.database_password.expose();

    // Use credentials
    // ...

    Ok(SyncRepositoryResponse {
        repository_id: request.repository_id,
        pulled: true,
        pushed: true,
        synced_at: chrono::Utc::now(),
    })
}
```

**Requirement Traceability:**
- REQ-SEC-019: Sensitive Data Protection
- REQ-SEC-020: Credential Management

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Encryption Layer section

#### 7.6.2. Data Encryption

**Principle:** Sensitive data at rest must be encrypted.

**Data Encryption:**
```rust
use aes_gcm::{
    aead::{Aead256, AeadCipher, KeyInit},
    aead::{NewAead, NoncePadding},
    rand::{rngs::OsRng, RngCore},
};
use std::fs::File;

pub struct EncryptionManager {
    key: [u8; 32],
}

impl EncryptionManager {
    pub fn new() -> Result<Self, EncryptionError> {
        let key = OsRng.gen::<[u8; 32]>();
        Ok(Self { key })
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aead256::new(&self.key, NewAead)?;
        let nonce = Aead256::generate_nonce(&mut OsRng)?;

        let ciphertext = cipher.encrypt(nonce, data, b"", &mut OsRng)?;
        Ok(ciphertext)
    }

    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aead256::new(&self.key, NewAead)?;

        let plaintext = cipher.decrypt(nonce, b"", ciphertext)?;
        Ok(plaintext)
    }
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),
}
```

**Requirement Traceability:**
- REQ-SEC-021: Data Encryption
- REQ-SEC-022: Encryption at Rest

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Encryption Layer section
---

## 8. API PERFORMANCE

This section defines performance requirements for the Desktop API, including latency requirements, throughput targets, caching strategies, and optimization techniques.

### 8.1. Latency Requirements

#### 8.1.1. Command Latency

**Principle:** All IPC commands must complete within specified latency targets.

**Latency Targets:**

| Command Type | Target Latency | Measurement Point |
|--------------|----------------|------------------|
| **Document Read** | < 50 ms | From command invocation to response |
| **Document Write** | < 100 ms | From command invocation to response |
| **Document List** | < 200 ms | From command invocation to response |
| **Repository Sync** | < 5 s | From command invocation to response |
| **Search** | < 500 ms | From command invocation to response |
| **System Commands** | < 10 ms | From command invocation to response |

**Latency Measurement:**
```rust
use std::time::Instant;

#[command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentResponse, IpcError> {
    let start = Instant::now();

    // Execute command
    let document = state.read().await.core.get_document(&request.id).await?;

    // Render HTML
    let rendered_html = state.read().await.core.render_document(&document).await?;

    // Get metadata
    let metadata = state.read().await.core.get_document_metadata(&document.id).await?;

    let latency = start.elapsed();

    // Log latency
    tracing::info!(
        command = "get_document",
        document_id = %request.id,
        latency_ms = latency.as_millis(),
        "Command completed"
    );

    Ok(GetDocumentResponse {
        document,
        rendered_html,
        metadata,
    })
}
```

**Requirement Traceability:**
- REQ-DESK-086: Hot-Reload Latency
- REQ-DESK-087: Initial Load Time

**ADR Reference:**
- [ADR-009: IPC Communication Architecture](../../specs/02_adrs/009_ipc_communication_architecture.md) - Performance Characteristics section

#### 8.1.2. Event Latency

**Principle:** All IPC events must be emitted within specified latency targets.

**Event Latency Targets:**

| Event Type | Target Latency | Measurement Point |
|------------|----------------|------------------|
| **Document Events** | < 10 ms | From state change to event emission |
| **Repository Events** | < 20 ms | From state change to event emission |
| **System Events** | < 5 ms | From state change to event emission |

**Event Latency Measurement:**
```rust
use std::time::Instant;

pub struct EventEmitter {
    last_emission: Arc<Mutex<HashMap<String, Instant>>>,
}

impl EventEmitter {
    pub async fn emit<F: serde::Serialize>(
        &self,
        window: &Window,
        event_name: &str,
        payload: &F,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();

        // Emit event
        window.emit(event_name, payload)?;

        let latency = start.elapsed();

        // Log latency
        tracing::debug!(
            event = %event_name,
            latency_ms = latency.as_millis(),
            "Event emitted"
        );

        Ok(())
    }
}
```

**Requirement Traceability:**
- REQ-DESK-086: Hot-Reload Latency
- REQ-IPC-031: Event Emission

### 8.2. Throughput Requirements

#### 8.2.1. Command Throughput

**Principle:** The API must support specified throughput targets.

**Throughput Targets:**

| Command Type | Target Throughput | Measurement Point |
|--------------|-------------------|------------------|
| **Document Read** | > 100 req/s | Commands per second |
| **Document Write** | > 50 req/s | Commands per second |
| **Document List** | > 20 req/s | Commands per second |
| **Repository Sync** | > 10 req/s | Commands per second |
| **Search** | > 5 req/s | Commands per second |
| **System Commands** | > 200 req/s | Commands per second |

**Throughput Measurement:**
```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct ThroughputMonitor {
    command_counts: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
}

impl ThroughputMonitor {
    pub fn record_command(&self, command: &str) {
        let mut counts = self.command_counts.lock().unwrap();
        let now = Instant::now();
        let (count, _) = counts.entry(command.to_string()).or_insert((0, now));
        *count += 1;
    }

    pub fn get_throughput(&self, command: &str, window_secs: u64) -> f64 {
        let counts = self.command_counts.lock().unwrap();
        if let Some((count, start)) = counts.get(command) {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0 {
                return count as f64 / elapsed as f64;
            }
        }
        0.0
    }
}
```

**Requirement Traceability:**
- REQ-IPC-033: Throughput Requirements
- REQ-DESK-090: Responsive UI

#### 8.2.2. Event Throughput

**Principle:** The API must support specified event throughput targets.

**Event Throughput Targets:**

| Event Type | Target Throughput | Measurement Point |
|------------|----------------|------------------|
| **Document Events** | > 1000 events/s | Events per second |
| **Repository Events** | > 500 events/s | Events per second |
| **System Events** | > 2000 events/s | Events per second |

**Requirement Traceability:**
- REQ-IPC-034: Event Throughput
- REQ-IPC-031: Event Emission

### 8.3. Caching Strategies

#### 8.3.1. LRU Cache

**Principle:** The API must implement LRU caching for frequently accessed data.

**LRU Cache Implementation:**
```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use lru::LruCache;

pub struct CacheManager {
    document_cache: Arc<Mutex<LruCache<String, Document>>>,
    rendered_html_cache: Arc<Mutex<LruCache<String, String>>>,
    metadata_cache: Arc<Mutex<LruCache<String, DocumentMetadata>>>,
}

impl CacheManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            document_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            rendered_html_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            metadata_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }

    pub async fn get_document(&self, id: &str) -> Option<Document> {
        let cache = self.document_cache.lock().await;
        cache.get(id).cloned()
    }

    pub async fn cache_document(&self, document: Document) {
        let mut cache = self.document_cache.lock().await;
        cache.put(document.id.clone(), document);
    }

    pub async fn get_rendered_html(&self, id: &str) -> Option<String> {
        let cache = self.rendered_html_cache.lock().await;
        cache.get(id).cloned()
    }

    pub async fn cache_rendered_html(&self, id: String, html: String) {
        let mut cache = self.rendered_html_cache.lock().await;
        cache.put(id, html);
    }

    pub async fn get_metadata(&self, id: &str) -> Option<DocumentMetadata> {
        let cache = self.metadata_cache.lock().await;
        cache.get(id).cloned()
    }

    pub async fn cache_metadata(&self, id: String, metadata: DocumentMetadata) {
        let mut cache = self.metadata_cache.lock().await;
        cache.put(id, metadata);
    }

    pub async fn invalidate_document(&self, id: &str) {
        let mut doc_cache = self.document_cache.lock().await;
        let mut html_cache = self.rendered_html_cache.lock().await;
        let mut meta_cache = self.metadata_cache.lock().await;

        doc_cache.pop(id);
        html_cache.pop(id);
        meta_cache.pop(id);
    }
}
```

**Cache Configuration:**
```rust
pub struct CacheConfig {
    pub document_capacity: usize,
    pub rendered_html_capacity: usize,
    pub metadata_capacity: usize,
    pub max_size_bytes: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            document_capacity: 1000, // Cache up to 1000 documents
            rendered_html_capacity: 500, // Cache up to 500 rendered HTML
            metadata_capacity: 2000, // Cache up to 2000 metadata entries
            max_size_bytes: 500 * 1024 * 1024, // 500MB max cache size
        }
    }
}
```

**Requirement Traceability:**
- REQ-DESK-041: LRU Cache
- REQ-DESK-042: Cache Invalidation

**ADR Reference:**
- [ADR-009: IPC Communication Architecture](../../specs/02_adrs/009_ipc_communication_architecture.md) - Performance Characteristics section

#### 8.3.2. Cache Invalidation

**Principle:** Cache entries must be invalidated when source data changes.

**Cache Invalidation Triggers:**
```rust
#[command]
pub async fn update_document(
    request: UpdateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    cache_manager: State<'_, Arc<CacheManager>>,
) -> Result<UpdateDocumentResponse, IpcError> {
    // Update document
    let (document_id, version) = state.write().await.core.update_document(
        &request.id,
        request.title.as_deref(),
        request.content.as_deref(),
    ).await?;

    // Invalidate cache entries
    cache_manager.invalidate_document(&document_id).await;

    Ok(UpdateDocumentResponse {
        document_id,
        updated_at: chrono::Utc::now(),
        version,
    })
}
```

**Requirement Traceability:**
- REQ-DESK-042: Cache Invalidation
- REQ-DESK-086: Hot-Reload Latency

### 8.4. Optimization Techniques

#### 8.4.1. Async Processing

**Principle:** All I/O operations must be asynchronous to prevent blocking.

**Async Processing Implementation:**
```rust
use tokio::fs;

#[command]
pub async fn list_documents(
    request: ListDocumentsRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<ListDocumentsResponse, IpcError> {
    // Use async file system operations
    let (documents, total_count) = state.read().await.core.list_documents(
        request.offset.unwrap_or(0),
        request.limit.unwrap_or(50).min(100),
        &request.sort_by.unwrap_or_else(|| "created_at".to_string()),
        &request.sort_order.unwrap_or_else(|| "desc".to_string()),
    ).await?;

    Ok(ListDocumentsResponse {
        documents,
        total_count,
        offset: request.offset.unwrap_or(0),
        limit: request.limit.unwrap_or(50).min(100),
    })
}
```

**Requirement Traceability:**
- REQ-IPC-031: Asynchronous Commands
- REQ-DESK-090: Responsive UI

#### 8.4.2. Batch Processing

**Principle:** Multiple operations should be batched when possible to reduce overhead.

**Batch Processing Implementation:**
```rust
#[command]
pub async fn get_documents_batch(
    request: GetDocumentsBatchRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentsBatchResponse, IpcError> {
    // Batch document retrieval
    let documents = state.read().await.core.get_documents_batch(&request.ids).await?;

    Ok(GetDocumentsBatchResponse {
        documents,
        retrieved_at: chrono::Utc::now(),
    })
}
```

**Batch Request Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentsBatchRequest {
    pub ids: Vec<String>,
}
```

**Batch Response Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentsBatchResponse {
    pub documents: Vec<Document>,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}
```

**Requirement Traceability:**
- REQ-IPC-035: Batch Processing
- REQ-DESK-088: Large File Handling

#### 8.4.3. Connection Pooling

**Principle:** Reuse connections to reduce connection overhead.

**Connection Pooling Implementation:**
```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ConnectionPool<T> {
    connections: Arc<Mutex<Vec<T>>>,
    max_connections: usize,
}

impl<T> ConnectionPool<T> {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            max_connections,
        }
    }

    pub async fn acquire(&self) -> Result<T, PoolError> {
        let mut connections = self.connections.lock().await;

        if connections.len() < self.max_connections {
            // Create new connection
            let connection = T::new().await?;
            connections.push(connection);
            Ok(connection)
        } else {
            // Wait for available connection
            drop(connections);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            self.acquire().await
        }
    }

    pub async fn release(&self, connection: T) {
        let mut connections = self.connections.lock().await;
        connections.retain(|c| Arc::strong_ptr_eq(c, connection) == false);
    }
}

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Connection pool exhausted")]
    PoolExhausted,
}
```

**Requirement Traceability:**
- REQ-IPC-036: Connection Pooling
- REQ-DESK-047: Server Health Monitoring

#### 8.4.4. Memory Pooling

**Principle:** Reuse memory allocations to reduce allocation overhead.

**Memory Pooling Implementation:**
```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MemoryPool<T> {
    pool: Arc<Mutex<Vec<T>>>,
    max_size: usize,
}

impl<T: Default + Clone> MemoryPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }

    pub fn acquire(&self) -> Option<T> {
        let mut pool = self.pool.lock().unwrap();
        pool.pop()
    }

    pub fn release(&self, item: T) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push(item);
        }
    }
}
```

**Requirement Traceability:**
- REQ-IPC-037: Memory Pooling
- REQ-DESK-091: Memory Usage

### 8.5. Performance Monitoring

#### 8.5.1. Metrics Collection

**Principle:** Performance metrics must be collected and reported.

**Metrics Collection:**
```rust
use std::sync::Arc;
use std::time::Instant;

pub struct PerformanceMetrics {
    command_latencies: Arc<Mutex<HashMap<String, Vec<u64>>>>,
    event_latencies: Arc<Mutex<HashMap<String, Vec<u64>>>>,
    command_counts: Arc<Mutex<HashMap<String, u64>>>>,
    event_counts: Arc<Mutex<HashMap<String, u64>>>>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            command_latencies: Arc::new(Mutex::new(HashMap::new())),
            event_latencies: Arc::new(Mutex::new(HashMap::new())),
            command_counts: Arc::new(Mutex::new(HashMap::new())),
            event_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record_command_latency(&self, command: &str, latency_ms: u64) {
        let mut latencies = self.command_latencies.lock().unwrap();
        latencies.entry(command.to_string()).or_insert_with(Vec::new).push(latency_ms);
    }

    pub fn record_event_latency(&self, event: &str, latency_ms: u64) {
        let mut latencies = self.event_latencies.lock().unwrap();
        latencies.entry(event.to_string()).or_insert_with(Vec::new).push(latency_ms);
    }

    pub fn record_command(&self, command: &str) {
        let mut counts = self.command_counts.lock().unwrap();
        *counts.entry(command.to_string()).or_insert(0) += 1;
    }

    pub fn record_event(&self, event: &str) {
        let mut counts = self.event_counts.lock().unwrap();
        *counts.entry(event.to_string()).or_insert(0) += 1;
    }

    pub fn get_command_stats(&self, command: &str) -> CommandStats {
        let latencies = self.command_latencies.lock().unwrap();
        let counts = self.command_counts.lock().unwrap();

        let latency_values = latencies.get(command).map(|v| v.as_slice()).unwrap_or(&[]);
        let count = *counts.get(command).unwrap_or(&0);

        CommandStats {
            count,
            avg_latency_ms: if latency_values.is_empty() {
                0
            } else {
                latency_values.iter().sum::<u64>() / latency_values.len() as u64
            },
            min_latency_ms: *latency_values.iter().min().unwrap_or(&0),
            max_latency_ms: *latency_values.iter().max().unwrap_or(&0),
            p50_latency_ms: percentile(latency_values, 50),
            p95_latency_ms: percentile(latency_values, 95),
            p99_latency_ms: percentile(latency_values, 99),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandStats {
    pub count: u64,
    pub avg_latency_ms: u64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
}

fn percentile(values: &[u64], p: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    let index = (sorted.len() as f64 * p as usize).min(sorted.len() - 1) as usize;
    sorted[index]
}
```

**Requirement Traceability:**
- REQ-SEC-023: Performance Monitoring
- REQ-SEC-024: Metrics Reporting

#### 8.5.2. Performance Alerts

**Principle:** Performance alerts must be triggered when thresholds are exceeded.

**Performance Alert Implementation:**
```rust
pub struct PerformanceAlertManager {
    metrics: Arc<PerformanceMetrics>,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub command_latency_p99_ms: u64,
    pub event_latency_p99_ms: u64,
    pub error_rate_threshold: f64,
}

impl PerformanceAlertManager {
    pub fn new(metrics: Arc<PerformanceMetrics>) -> Self {
        Self {
            metrics,
            alert_thresholds: AlertThresholds::default(),
        }
    }

    pub async fn check_performance(&self) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();

        // Check command latencies
        for (command, _) in self.metrics.command_latencies.lock().unwrap().iter() {
            let stats = self.metrics.get_command_stats(command);
            if stats.p99_latency_ms > self.alert_thresholds.command_latency_p99_ms {
                alerts.push(PerformanceAlert {
                    alert_type: AlertType::HighCommandLatency,
                    command: command.clone(),
                    metric: stats.p99_latency_ms,
                    threshold: self.alert_thresholds.command_latency_p99_ms,
                });
            }
        }

        // Check error rates
        let total_commands: u64 = self.metrics.command_counts.lock().unwrap().values().map(|_, c| c).sum();
        let total_errors: u64 = self.metrics.command_counts.lock().unwrap().values().map(|_, c| c).sum();

        if total_commands > 0 {
            let error_rate = total_errors as f64 / total_commands as f64;
            if error_rate > self.alert_thresholds.error_rate_threshold {
                alerts.push(PerformanceAlert {
                    alert_type: AlertType::HighErrorRate,
                    metric: error_rate,
                    threshold: self.alert_thresholds.error_rate_threshold,
                });
            }
        }

        alerts
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub command: Option<String>,
    pub metric: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertType {
    HighCommandLatency,
    HighEventLatency,
    HighErrorRate,
    LowThroughput,
}
```

**Requirement Traceability:**
- REQ-SEC-025: Performance Alerts
- REQ-SEC-026: Threshold Monitoring

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Security Controls section
---

## 9. API DOCUMENTATION

This section provides comprehensive documentation for the Desktop API, including OpenAPI/Swagger specifications, code examples, usage examples, and best practices.

### 9.1. OpenAPI/Swagger Specification

**Principle:** The Desktop API must be documented using OpenAPI 3.1.0 specification format.

**OpenAPI Specification:**
```yaml
openapi: 3.1.0
info:
  title: Tachyon Desktop API
  description: |
    The Tachyon Desktop API provides inter-process communication (IPC) between
    the WebView frontend and Rust backend for the Tachyon document management
    system. This API enables document operations, repository management,
    search functionality, and system configuration.

    **Key Features:**
    - Type-safe IPC commands using Tauri
    - Event-driven architecture for real-time updates
    - Comprehensive error handling with detailed error codes
    - Performance monitoring and metrics collection
    - Security controls including authentication and authorization

    **Versioning:**
    The API follows Semantic Versioning 2.0.0 (semver) format:
    - MAJOR version: Incompatible API changes
    - MINOR version: Backwards-compatible functionality additions
    - PATCH version: Backwards-compatible bug fixes

    **Documentation:**
    For detailed API documentation, see:
    - [Desktop API Specification](./desktop_api_specification.md)
    - [IPC Communication Architecture](../../specs/02_adrs/009_ipc_communication_architecture.md)
    - [Security Architecture](../../specs/02_adrs/010_security_architecture.md)
  version: 1.0.0
  contact:
    name: Tachyon Project
    url: https://github.com/tachyon-toolchain/tachyon
    email: support@tachyon.dev
  license:
    name: MIT License
    url: https://opensource.org/licenses/MIT

servers:
  - url: ipc://tachyon-desktop
    description: Local IPC endpoint for desktop application

tags:
  - name: Documents
    description: Document management operations
  - name: Repositories
    description: Repository management operations
  - name: Search
    description: Search and query operations
  - name: System
    description: System configuration and operations

paths:
  /documents:
    get:
      summary: List documents
      description: Retrieve a paginated list of documents with optional filtering and sorting
      operationId: list_documents
      tags:
        - Documents
      parameters:
        - name: offset
          in: query
          description: Number of documents to skip (default: 0)
          schema:
            type: integer
            minimum: 0
            default: 0
        - name: limit
          in: query
          description: Maximum number of documents to return (default: 50, max: 100)
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 50
        - name: sort_by
          in: query
          description: Field to sort by (default: created_at)
          schema:
            type: string
            enum: [created_at, updated_at, title]
            default: created_at
        - name: sort_order
          in: query
          description: Sort order (default: desc)
          schema:
            type: string
            enum: [asc, desc]
            default: desc
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ListDocumentsResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '401':
          description: Unauthorized
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

  /documents/{id}:
    get:
      summary: Get document
      description: Retrieve a document by ID with rendered HTML and metadata
      operationId: get_document
      tags:
        - Documents
      parameters:
        - name: id
          in: path
          description: Document ID
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/GetDocumentResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '404':
          description: Document not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

    put:
      summary: Update document
      description: Update a document's title and/or content
      operationId: update_document
      tags:
        - Documents
      parameters:
        - name: id
          in: path
          description: Document ID
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateDocumentRequest'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UpdateDocumentResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '404':
          description: Document not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

    delete:
      summary: Delete document
      description: Delete a document by ID
      operationId: delete_document
      tags:
        - Documents
      parameters:
        - name: id
          in: path
          description: Document ID
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DeleteDocumentResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '404':
          description: Document not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

  /documents:
    post:
      summary: Create document
      description: Create a new document
      operationId: create_document
      tags:
        - Documents
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateDocumentRequest'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CreateDocumentResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

  /repositories:
    get:
      summary: List repositories
      description: Retrieve a list of all repositories
      operationId: list_repositories
      tags:
        - Repositories
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ListRepositoriesResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

    post:
      summary: Add repository
      description: Add a new repository to the application
      operationId: add_repository
      tags:
        - Repositories
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AddRepositoryRequest'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AddRepositoryResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

  /repositories/{id}:
    delete:
      summary: Remove repository
      description: Remove a repository from the application
      operationId: remove_repository
      tags:
        - Repositories
      parameters:
        - name: id
          in: path
          description: Repository ID
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/RemoveRepositoryResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '404':
          description: Repository not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

    post:
      summary: Sync repository
      description: Sync a repository with remote
      operationId: sync_repository
      tags:
        - Repositories
      parameters:
        - name: id
          in: path
          description: Repository ID
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SyncRepositoryResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '404':
          description: Repository not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

  /search:
    post:
      summary: Search documents
      description: Search documents with query, filters, and sorting
      operationId: search_documents
      tags:
        - Search
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/SearchRequest'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SearchResponse'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

  /system/status:
    get:
      summary: Get system status
      description: Retrieve the current system status
      operationId: get_system_status
      tags:
        - System
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SystemStatus'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

  /system/config:
    get:
      summary: Get system configuration
      description: Retrieve the current system configuration
      operationId: get_system_config
      tags:
        - System
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SystemConfig'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

    put:
      summary: Update system configuration
      description: Update the system configuration
      operationId: update_system_config
      tags:
        - System
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/SystemConfig'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SystemConfig'
        '400':
          description: Bad request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IpcError'

components:
  schemas:
    Document:
      type: object
      required:
        - id
        - title
        - content
        - created_at
        - updated_at
        - version
      properties:
        id:
          type: string
          format: uuid
          description: Unique document identifier
        title:
          type: string
          description: Document title
        content:
          type: string
          description: Document content in Markdown format
        created_at:
          type: string
          format: date-time
          description: Document creation timestamp
        updated_at:
          type: string
          format: date-time
          description: Document last update timestamp
        version:
          type: integer
          description: Document version number

    DocumentMetadata:
      type: object
      required:
        - id
        - document_id
        - word_count
        - reading_time
        - tags
      properties:
        id:
          type: string
          format: uuid
          description: Unique metadata identifier
        document_id:
          type: string
          format: uuid
          description: Associated document ID
        word_count:
          type: integer
          description: Document word count
        reading_time:
          type: integer
          description: Estimated reading time in minutes
        tags:
          type: array
          items:
            type: string
          description: Document tags

    ListDocumentsResponse:
      type: object
      required:
        - documents
        - total_count
        - offset
        - limit
      properties:
        documents:
          type: array
          items:
            $ref: '#/components/schemas/Document'
          description: List of documents
        total_count:
          type: integer
          description: Total number of documents
        offset:
          type: integer
          description: Pagination offset
        limit:
          type: integer
          description: Pagination limit

    GetDocumentResponse:
      type: object
      required:
        - document
        - rendered_html
        - metadata
      properties:
        document:
          $ref: '#/components/schemas/Document'
        rendered_html:
          type: string
          description: Rendered HTML content
        metadata:
          $ref: '#/components/schemas/DocumentMetadata'

    UpdateDocumentRequest:
      type: object
      properties:
        title:
          type: string
          description: New document title (optional)
        content:
          type: string
          description: New document content (optional)

    UpdateDocumentResponse:
      type: object
      required:
        - document_id
        - updated_at
        - version
      properties:
        document_id:
          type: string
          format: uuid
          description: Updated document ID
        updated_at:
          type: string
          format: date-time
          description: Update timestamp
        version:
          type: integer
          description: New document version

    DeleteDocumentResponse:
      type: object
      required:
        - document_id
        - deleted_at
      properties:
        document_id:
          type: string
          format: uuid
          description: Deleted document ID
        deleted_at:
          type: string
          format: date-time
          description: Deletion timestamp

    CreateDocumentRequest:
      type: object
      required:
        - title
      properties:
        title:
          type: string
          description: Document title
        content:
          type: string
          description: Document content (optional)

    CreateDocumentResponse:
      type: object
      required:
        - document
        - created_at
        - version
      properties:
        document:
          $ref: '#/components/schemas/Document'
        created_at:
          type: string
          format: date-time
          description: Creation timestamp
        version:
          type: integer
          description: Document version

    Repository:
      type: object
      required:
        - id
        - name
        - path
        - remote_url
        - branch
        - sync_status
        - last_synced_at
      properties:
        id:
          type: string
          format: uuid
          description: Unique repository identifier
        name:
          type: string
          description: Repository name
        path:
          type: string
          description: Repository local path
        remote_url:
          type: string
          description: Repository remote URL
        branch:
          type: string
          description: Current branch name
        sync_status:
          type: string
          enum: [synced, out_of_sync, syncing, error]
          description: Repository sync status
        last_synced_at:
          type: string
          format: date-time
          description: Last sync timestamp

    ListRepositoriesResponse:
      type: object
      required:
        - repositories
      properties:
        repositories:
          type: array
          items:
            $ref: '#/components/schemas/Repository'
          description: List of repositories

    AddRepositoryRequest:
      type: object
      required:
        - name
        - path
      properties:
        name:
          type: string
          description: Repository name
        path:
          type: string
          description: Repository local path
        remote_url:
          type: string
          description: Repository remote URL (optional)

    AddRepositoryResponse:
      type: object
      required:
        - repository
        - added_at
      properties:
        repository:
          $ref: '#/components/schemas/Repository'
        added_at:
          type: string
          format: date-time
          description: Addition timestamp

    RemoveRepositoryResponse:
      type: object
      required:
        - repository_id
        - removed_at
      properties:
        repository_id:
          type: string
          format: uuid
          description: Removed repository ID
        removed_at:
          type: string
          format: date-time
          description: Removal timestamp

    SyncRepositoryResponse:
      type: object
      required:
        - repository_id
        - sync_status
        - synced_at
      properties:
        repository_id:
          type: string
          format: uuid
          description: Synced repository ID
        sync_status:
          type: string
          enum: [synced, out_of_sync, syncing, error]
          description: Repository sync status
        synced_at:
          type: string
          format: date-time
          description: Sync timestamp

    SearchRequest:
      type: object
      required:
        - query
      properties:
        query:
          type: string
          description: Search query
        filters:
          type: object
          description: Search filters
        sort_by:
          type: string
          description: Sort field
        sort_order:
          type: string
          enum: [asc, desc]
          description: Sort order

    SearchResponse:
      type: object
      required:
        - results
        - total_count
      properties:
        results:
          type: array
          items:
            $ref: '#/components/schemas/Document'
          description: Search results
        total_count:
          type: integer
          description: Total number of results

    SystemStatus:
      type: object
      required:
        - status
        - version
        - uptime
      properties:
        status:
          type: string
          enum: [running, stopped, error]
          description: System status
        version:
          type: string
          description: Application version
        uptime:
          type: integer
          description: Uptime in seconds

    SystemConfig:
      type: object
      properties:
        theme:
          type: string
          enum: [light, dark, system]
          description: UI theme
        language:
          type: string
          description: Application language
        auto_sync:
          type: boolean
          description: Auto-sync repositories
        sync_interval:
          type: integer
          description: Sync interval in seconds

    IpcError:
      type: object
      required:
        - code
        - message
      properties:
        code:
          type: string
          description: Error code
        message:
          type: string
          description: Error message
        details:
          type: object
          description: Additional error details
```

**Requirement Traceability:**
- REQ-IPC-021: Type Safety
- REQ-IPC-022: Error Handling

### 9.2. Code Examples

#### 9.2.1. Rust Backend Examples

**Example 1: Registering IPC Commands**

```rust
// src-tauri/src/lib.rs

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize application state
            let state = Arc::new(RwLock::new(ApplicationState::new()?));
            app.manage(state.clone());
            app.manage(Arc::new(CacheManager::new(1000)));
            app.manage(Arc::new(PerformanceMetrics::new()));

            // Register IPC commands
            tauri::generate_handler![
                // Document commands
                create_document,
                get_document,
                update_document,
                delete_document,
                list_documents,
                get_documents_batch,

                // Repository commands
                add_repository,
                remove_repository,
                sync_repository,
                list_repositories,
                get_repository_status,

                // Search commands
                search_documents,
                search_with_filters,

                // System commands
                get_system_status,
                get_system_config,
                update_system_config,
                quit_application,
            ];

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Example 2: Implementing Document Commands**

```rust
// src-tauri/src/commands/document_commands.rs

use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn create_document(
    request: CreateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<CreateDocumentResponse, IpcError> {
    // Validate input
    if request.title.trim().is_empty() {
        return Err(IpcError::ValidationError {
            field: "title".to_string(),
            message: "Title cannot be empty".to_string(),
        });
    }

    // Create document
    let document = state.write().await.core.create_document(
        &request.title,
        request.content.as_deref(),
    ).await?;

    // Emit event
    let window = state.read().await.window();
    window.emit("document:created", &document)?;

    Ok(CreateDocumentResponse {
        document,
        created_at: chrono::Utc::now(),
        version: 1,
    })
}

#[tauri::command]
pub async fn get_document(
    request: GetDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    cache_manager: State<'_, Arc<CacheManager>>,
) -> Result<GetDocumentResponse, IpcError> {
    // Check cache first
    if let Some(cached_doc) = cache_manager.get_document(&request.id).await {
        return Ok(GetDocumentResponse {
            document: cached_doc,
            rendered_html: cache_manager.get_rendered_html(&request.id)
                .await
                .unwrap_or_default(),
            metadata: cache_manager.get_metadata(&request.id)
                .await
                .unwrap(),
        });
    }

    // Get document from core
    let document = state.read().await.core.get_document(&request.id).await?;

    // Render HTML
    let rendered_html = state.read().await.core.render_document(&document).await?;

    // Get metadata
    let metadata = state.read().await.core.get_document_metadata(&document.id).await?;

    // Cache results
    cache_manager.cache_document(document.clone()).await;
    cache_manager.cache_rendered_html(document.id.clone(), rendered_html.clone()).await;
    cache_manager.cache_metadata(document.id.clone(), metadata.clone()).await;

    Ok(GetDocumentResponse {
        document,
        rendered_html,
        metadata,
    })
}

#[tauri::command]
pub async fn update_document(
    request: UpdateDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    cache_manager: State<'_, Arc<CacheManager>>,
) -> Result<UpdateDocumentResponse, IpcError> {
    // Validate input
    if let Some(ref title) = request.title {
        if title.trim().is_empty() {
            return Err(IpcError::ValidationError {
                field: "title".to_string(),
                message: "Title cannot be empty".to_string(),
            });
        }
    }

    // Update document
    let (document_id, version) = state.write().await.core.update_document(
        &request.id,
        request.title.as_deref(),
        request.content.as_deref(),
    ).await?;

    // Invalidate cache
    cache_manager.invalidate_document(&document_id).await;

    // Emit event
    let window = state.read().await.window();
    window.emit("document:updated", &document_id)?;

    Ok(UpdateDocumentResponse {
        document_id,
        updated_at: chrono::Utc::now(),
        version,
    })
}

#[tauri::command]
pub async fn delete_document(
    request: DeleteDocumentRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
    cache_manager: State<'_, Arc<CacheManager>>,
) -> Result<DeleteDocumentResponse, IpcError> {
    // Delete document
    state.write().await.core.delete_document(&request.id).await?;

    // Invalidate cache
    cache_manager.invalidate_document(&request.id).await;

    // Emit event
    let window = state.read().await.window();
    window.emit("document:deleted", &request.id)?;

    Ok(DeleteDocumentResponse {
        document_id: request.id,
        deleted_at: chrono::Utc::now(),
    })
}

#[tauri::command]
pub async fn list_documents(
    request: ListDocumentsRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<ListDocumentsResponse, IpcError> {
    // List documents
    let (documents, total_count) = state.read().await.core.list_documents(
        request.offset.unwrap_or(0),
        request.limit.unwrap_or(50).min(100),
        &request.sort_by.unwrap_or_else(|| "created_at".to_string()),
        &request.sort_order.unwrap_or_else(|| "desc".to_string()),
    ).await?;

    Ok(ListDocumentsResponse {
        documents,
        total_count,
        offset: request.offset.unwrap_or(0),
        limit: request.limit.unwrap_or(50).min(100),
    })
}

#[tauri::command]
pub async fn get_documents_batch(
    request: GetDocumentsBatchRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<GetDocumentsBatchResponse, IpcError> {
    // Validate input
    if request.ids.is_empty() {
        return Err(IpcError::ValidationError {
            field: "ids".to_string(),
            message: "IDs list cannot be empty".to_string(),
        });
    }

    if request.ids.len() > 100 {
        return Err(IpcError::ValidationError {
            field: "ids".to_string(),
            message: "Cannot batch more than 100 documents".to_string(),
        });
    }

    // Batch get documents
    let documents = state.read().await.core.get_documents_batch(&request.ids).await?;

    Ok(GetDocumentsBatchResponse {
        documents,
        retrieved_at: chrono::Utc::now(),
    })
}
```

**Example 3: Implementing Repository Commands**

```rust
// src-tauri/src/commands/repository_commands.rs

use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn add_repository(
    request: AddRepositoryRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<AddRepositoryResponse, IpcError> {
    // Validate input
    if request.name.trim().is_empty() {
        return Err(IpcError::ValidationError {
            field: "name".to_string(),
            message: "Repository name cannot be empty".to_string(),
        });
    }

    if request.path.trim().is_empty() {
        return Err(IpcError::ValidationError {
            field: "path".to_string(),
            message: "Repository path cannot be empty".to_string(),
        });
    }

    // Add repository
    let repository = state.write().await.core.add_repository(
        &request.name,
        &request.path,
        request.remote_url.as_deref(),
    ).await?;

    // Emit event
    let window = state.read().await.window();
    window.emit("repository:added", &repository)?;

    Ok(AddRepositoryResponse {
        repository,
        added_at: chrono::Utc::now(),
    })
}

#[tauri::command]
pub async fn remove_repository(
    request: RemoveRepositoryRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<RemoveRepositoryResponse, IpcError> {
    // Remove repository
    state.write().await.core.remove_repository(&request.id).await?;

    // Emit event
    let window = state.read().await.window();
    window.emit("repository:removed", &request.id)?;

    Ok(RemoveRepositoryResponse {
        repository_id: request.id,
        removed_at: chrono::Utc::now(),
    })
}

#[tauri::command]
pub async fn sync_repository(
    request: SyncRepositoryRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SyncRepositoryResponse, IpcError> {
    // Emit sync started event
    let window = state.read().await.window();
    window.emit("repository:sync_started", &request.id)?;

    // Sync repository
    let (repository_id, sync_status) = state.write().await.core.sync_repository(&request.id).await?;

    // Emit sync completed event
    window.emit("repository:sync_completed", &(repository_id.clone(), sync_status.clone()))?;

    Ok(SyncRepositoryResponse {
        repository_id,
        sync_status,
        synced_at: chrono::Utc::now(),
    })
}

#[tauri::command]
pub async fn list_repositories(
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<ListRepositoriesResponse, IpcError> {
    // List repositories
    let repositories = state.read().await.core.list_repositories().await?;

    Ok(ListRepositoriesResponse {
        repositories,
    })
}

#[tauri::command]
pub async fn get_repository_status(
    request: GetRepositoryStatusRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<RepositoryStatus, IpcError> {
    // Get repository status
    let status = state.read().await.core.get_repository_status(&request.id).await?;

    Ok(status)
}
```

**Example 4: Implementing Search Commands**

```rust
// src-tauri/src/commands/search_commands.rs

use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn search_documents(
    request: SearchRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SearchResponse, IpcError> {
    // Validate input
    if request.query.trim().is_empty() {
        return Err(IpcError::ValidationError {
            field: "query".to_string(),
            message: "Search query cannot be empty".to_string(),
        });
    }

    // Search documents
    let (results, total_count) = state.read().await.core.search_documents(
        &request.query,
        request.filters.as_ref(),
        request.sort_by.as_deref(),
        request.sort_order.as_deref(),
    ).await?;

    Ok(SearchResponse {
        results,
        total_count,
    })
}

#[tauri::command]
pub async fn search_with_filters(
    request: SearchWithFiltersRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SearchResponse, IpcError> {
    // Validate input
    if request.query.trim().is_empty() {
        return Err(IpcError::ValidationError {
            field: "query".to_string(),
            message: "Search query cannot be empty".to_string(),
        });
    }

    // Search documents with filters
    let (results, total_count) = state.read().await.core.search_with_filters(
        &request.query,
        &request.filters,
        request.sort_by.as_deref(),
        request.sort_order.as_deref(),
    ).await?;

    Ok(SearchResponse {
        results,
        total_count,
    })
}
```

**Example 5: Implementing System Commands**

```rust
// src-tauri/src/commands/system_commands.rs

use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn get_system_status(
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SystemStatus, IpcError> {
    // Get system status
    let status = state.read().await.core.get_system_status().await?;

    Ok(status)
}

#[tauri::command]
pub async fn get_system_config(
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SystemConfig, IpcError> {
    // Get system config
    let config = state.read().await.core.get_system_config().await?;

    Ok(config)
}

#[tauri::command]
pub async fn update_system_config(
    request: UpdateSystemConfigRequest,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<SystemConfig, IpcError> {
    // Validate input
    if let Some(ref sync_interval) = request.sync_interval {
        if *sync_interval < 60 {
            return Err(IpcError::ValidationError {
                field: "sync_interval".to_string(),
                message: "Sync interval must be at least 60 seconds".to_string(),
            });
        }
    }

    // Update system config
    let config = state.write().await.core.update_system_config(request).await?;

    Ok(config)
}

#[tauri::command]
pub async fn quit_application(
    app: tauri::AppHandle,
    state: State<'_, Arc<RwLock<ApplicationState>>>,
) -> Result<(), IpcError> {
    // Save state
    state.write().await.core.save_state().await?;

    // Emit quit event
    app.emit("system:quit", ())?;

    // Quit application
    app.exit(0);

    Ok(())
}
```

#### 9.2.2. TypeScript Frontend Examples

**Example 1: Invoking IPC Commands**

```typescript
// src/api/desktop.ts

import { invoke } from '@tauri-apps/api/core';
import type {
  CreateDocumentRequest,
  CreateDocumentResponse,
  GetDocumentRequest,
  GetDocumentResponse,
  UpdateDocumentRequest,
  UpdateDocumentResponse,
  DeleteDocumentRequest,
  DeleteDocumentResponse,
  ListDocumentsRequest,
  ListDocumentsResponse,
  GetDocumentsBatchRequest,
  GetDocumentsBatchResponse,
  AddRepositoryRequest,
  AddRepositoryResponse,
  RemoveRepositoryRequest,
  RemoveRepositoryResponse,
  SyncRepositoryRequest,
  SyncRepositoryResponse,
  ListRepositoriesResponse,
  GetRepositoryStatusRequest,
  RepositoryStatus,
  SearchRequest,
  SearchResponse,
  SearchWithFiltersRequest,
  SystemStatus,
  SystemConfig,
  UpdateSystemConfigRequest,
  IpcError,
} from './types';

/**
 * Desktop API client for Tauri IPC commands
 */
export class DesktopApi {
  /**
   * Create a new document
   * @param request Document creation request
   * @returns Created document response
   * @throws IpcError if the operation fails
   */
  static async createDocument(request: CreateDocumentRequest): Promise<CreateDocumentResponse> {
    try {
      return await invoke<CreateDocumentResponse>('create_document', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Get a document by ID
   * @param request Document retrieval request
   * @returns Document response with rendered HTML and metadata
   * @throws IpcError if the operation fails
   */
  static async getDocument(request: GetDocumentRequest): Promise<GetDocumentResponse> {
    try {
      return await invoke<GetDocumentResponse>('get_document', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Update a document
   * @param request Document update request
   * @returns Updated document response
   * @throws IpcError if the operation fails
   */
  static async updateDocument(request: UpdateDocumentRequest): Promise<UpdateDocumentResponse> {
    try {
      return await invoke<UpdateDocumentResponse>('update_document', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Delete a document
   * @param request Document deletion request
   * @returns Deletion confirmation
   * @throws IpcError if the operation fails
   */
  static async deleteDocument(request: DeleteDocumentRequest): Promise<DeleteDocumentResponse> {
    try {
      return await invoke<DeleteDocumentResponse>('delete_document', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * List documents with pagination
   * @param request List documents request
   * @returns List of documents
   * @throws IpcError if the operation fails
   */
  static async listDocuments(request?: ListDocumentsRequest): Promise<ListDocumentsResponse> {
    try {
      return await invoke<ListDocumentsResponse>('list_documents', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Get multiple documents in batch
   * @param request Batch document retrieval request
   * @returns Batch document response
   * @throws IpcError if the operation fails
   */
  static async getDocumentsBatch(request: GetDocumentsBatchRequest): Promise<GetDocumentsBatchResponse> {
    try {
      return await invoke<GetDocumentsBatchResponse>('get_documents_batch', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Add a repository
   * @param request Repository addition request
   * @returns Added repository response
   * @throws IpcError if the operation fails
   */
  static async addRepository(request: AddRepositoryRequest): Promise<AddRepositoryResponse> {
    try {
      return await invoke<AddRepositoryResponse>('add_repository', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Remove a repository
   * @param request Repository removal request
   * @returns Removal confirmation
   * @throws IpcError if the operation fails
   */
  static async removeRepository(request: RemoveRepositoryRequest): Promise<RemoveRepositoryResponse> {
    try {
      return await invoke<RemoveRepositoryResponse>('remove_repository', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Sync a repository
   * @param request Repository sync request
   * @returns Sync status
   * @throws IpcError if the operation fails
   */
  static async syncRepository(request: SyncRepositoryRequest): Promise<SyncRepositoryResponse> {
    try {
      return await invoke<SyncRepositoryResponse>('sync_repository', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * List all repositories
   * @returns List of repositories
   * @throws IpcError if the operation fails
   */
  static async listRepositories(): Promise<ListRepositoriesResponse> {
    try {
      return await invoke<ListRepositoriesResponse>('list_repositories');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Get repository status
   * @param request Repository status request
   * @returns Repository status
   * @throws IpcError if the operation fails
   */
  static async getRepositoryStatus(request: GetRepositoryStatusRequest): Promise<RepositoryStatus> {
    try {
      return await invoke<RepositoryStatus>('get_repository_status', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Search documents
   * @param request Search request
   * @returns Search results
   * @throws IpcError if the operation fails
   */
  static async searchDocuments(request: SearchRequest): Promise<SearchResponse> {
    try {
      return await invoke<SearchResponse>('search_documents', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Search documents with filters
   * @param request Search with filters request
   * @returns Search results
   * @throws IpcError if the operation fails
   */
  static async searchWithFilters(request: SearchWithFiltersRequest): Promise<SearchResponse> {
    try {
      return await invoke<SearchResponse>('search_with_filters', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Get system status
   * @returns System status
   * @throws IpcError if the operation fails
   */
  static async getSystemStatus(): Promise<SystemStatus> {
    try {
      return await invoke<SystemStatus>('get_system_status');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Get system configuration
   * @returns System configuration
   * @throws IpcError if the operation fails
   */
  static async getSystemConfig(): Promise<SystemConfig> {
    try {
      return await invoke<SystemConfig>('get_system_config');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Update system configuration
   * @param request System config update request
   * @returns Updated system configuration
   * @throws IpcError if the operation fails
   */
  static async updateSystemConfig(request: UpdateSystemConfigRequest): Promise<SystemConfig> {
    try {
      return await invoke<SystemConfig>('update_system_config', { request });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Quit the application
   * @throws IpcError if the operation fails
   */
  static async quitApplication(): Promise<void> {
    try {
      await invoke('quit_application');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Handle IPC errors
   * @param error Error from IPC invocation
   * @returns Typed IpcError
   */
  private static handleError(error: unknown): IpcError {
    if (error && typeof error === 'object' && 'code' in error) {
      return error as IpcError;
    }
    return {
      code: 'UNKNOWN_ERROR',
      message: String(error),
    };
  }
}
```

**Example 2: Listening to IPC Events**

```typescript
// src/api/events.ts

import { listen } from '@tauri-apps/api/event';
import type {
  DocumentCreatedEvent,
  DocumentUpdatedEvent,
  DocumentDeletedEvent,
  DocumentSyncedEvent,
  RepositoryAddedEvent,
  RepositoryRemovedEvent,
  RepositorySyncStartedEvent,
  RepositorySyncCompletedEvent,
  SystemInitializedEvent,
  SystemErrorEvent,
  SystemWarningEvent,
} from './types';

/**
 * Desktop API event listener
 */
export class DesktopEvents {
  private static listeners: Map<string, () => void> = new Map();

  /**
   * Listen to document created events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onDocumentCreated(callback: (event: DocumentCreatedEvent) => void): () => void {
    const unlisten = listen<DocumentCreatedEvent>('document:created', (event) => {
      callback(event.payload);
    });

    const key = 'document:created';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to document updated events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onDocumentUpdated(callback: (event: DocumentUpdatedEvent) => void): () => void {
    const unlisten = listen<DocumentUpdatedEvent>('document:updated', (event) => {
      callback(event.payload);
    });

    const key = 'document:updated';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to document deleted events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onDocumentDeleted(callback: (event: DocumentDeletedEvent) => void): () => void {
    const unlisten = listen<DocumentDeletedEvent>('document:deleted', (event) => {
      callback(event.payload);
    });

    const key = 'document:deleted';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to document synced events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onDocumentSynced(callback: (event: DocumentSyncedEvent) => void): () => void {
    const unlisten = listen<DocumentSyncedEvent>('document:synced', (event) => {
      callback(event.payload);
    });

    const key = 'document:synced';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to repository added events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onRepositoryAdded(callback: (event: RepositoryAddedEvent) => void): () => void {
    const unlisten = listen<RepositoryAddedEvent>('repository:added', (event) => {
      callback(event.payload);
    });

    const key = 'repository:added';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to repository removed events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onRepositoryRemoved(callback: (event: RepositoryRemovedEvent) => void): () => void {
    const unlisten = listen<RepositoryRemovedEvent>('repository:removed', (event) => {
      callback(event.payload);
    });

    const key = 'repository:removed';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to repository sync started events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onRepositorySyncStarted(callback: (event: RepositorySyncStartedEvent) => void): () => void {
    const unlisten = listen<RepositorySyncStartedEvent>('repository:sync_started', (event) => {
      callback(event.payload);
    });

    const key = 'repository:sync_started';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to repository sync completed events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onRepositorySyncCompleted(callback: (event: RepositorySyncCompletedEvent) => void): () => void {
    const unlisten = listen<RepositorySyncCompletedEvent>('repository:sync_completed', (event) => {
      callback(event.payload);
    });

    const key = 'repository:sync_completed';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to system initialized events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onSystemInitialized(callback: (event: SystemInitializedEvent) => void): () => void {
    const unlisten = listen<SystemInitializedEvent>('system:initialized', (event) => {
      callback(event.payload);
    });

    const key = 'system:initialized';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to system error events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onSystemError(callback: (event: SystemErrorEvent) => void): () => void {
    const unlisten = listen<SystemErrorEvent>('system:error', (event) => {
      callback(event.payload);
    });

    const key = 'system:error';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Listen to system warning events
   * @param callback Event callback
   * @returns Unsubscribe function
   */
  static onSystemWarning(callback: (event: SystemWarningEvent) => void): () => void {
    const unlisten = listen<SystemWarningEvent>('system:warning', (event) => {
      callback(event.payload);
    });

    const key = 'system:warning';
    this.listeners.set(key, unlisten);

    return () => {
      this.listeners.delete(key);
      unlisten.then(fn => fn());
    };
  }

  /**
   * Unsubscribe from all events
   */
  static unsubscribeAll(): void {
    this.listeners.forEach((unlisten) => {
      unlisten.then(fn => fn());
    });
    this.listeners.clear();
  }
}
```

**Requirement Traceability:**
- REQ-IPC-021: Type Safety
- REQ-IPC-022: Error Handling
- REQ-IPC-031: Event Emission

### 9.3. Usage Examples

#### 9.3.1. Document Management

**Example: Creating a New Document**

```typescript
// src/components/DocumentCreator.tsx

import React, { useState } from 'react';
import { DesktopApi } from '../api/desktop';

export function DocumentCreator() {
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await DesktopApi.createDocument({
        title,
        content: content || undefined,
      });

      console.log('Document created:', response.document);
      // Navigate to the new document
      // navigate(`/documents/${response.document.id}`);
    } catch (err) {
      setError(err.message || 'Failed to create document');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h2>Create New Document</h2>
      <input
        type="text"
        placeholder="Document title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
      />
      <textarea
        placeholder="Document content (optional)"
        value={content}
        onChange={(e) => setContent(e.target.value)}
      />
      <button onClick={handleCreate} disabled={loading}>
        {loading ? 'Creating...' : 'Create Document'}
      </button>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

**Example: Viewing a Document**

```typescript
// src/components/DocumentViewer.tsx

import React, { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { DesktopApi } from '../api/desktop';
import { DesktopEvents } from '../api/events';
import type { Document, DocumentMetadata } from '../api/types';

export function DocumentViewer() {
  const { id } = useParams<{ id: string }>();
  const [document, setDocument] = useState<Document | null>(null);
  const [renderedHtml, setRenderedHtml] = useState<string>('');
  const [metadata, setMetadata] = useState<DocumentMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;

    const loadDocument = async () => {
      setLoading(true);
      setError(null);

      try {
        const response = await DesktopApi.getDocument({ id });
        setDocument(response.document);
        setRenderedHtml(response.rendered_html);
        setMetadata(response.metadata);
      } catch (err) {
        setError(err.message || 'Failed to load document');
      } finally {
        setLoading(false);
      }
    };

    loadDocument();

    // Listen to document updates
    const unsubscribe = DesktopEvents.onDocumentUpdated((event) => {
      if (event.document_id === id) {
        loadDocument();
      }
    });

    return () => unsubscribe();
  }, [id]);

  if (loading) return <div>Loading...</div>;
  if (error) return <div className="error">{error}</div>;
  if (!document) return <div>Document not found</div>;

  return (
    <div>
      <h1>{document.title}</h1>
      <div className="metadata">
        <span>Created: {new Date(document.created_at).toLocaleString()}</span>
        <span>Updated: {new Date(document.updated_at).toLocaleString()}</span>
        <span>Version: {document.version}</span>
        {metadata && (
          <>
            <span>Words: {metadata.word_count}</span>
            <span>Reading time: {metadata.reading_time} min</span>
          </>
        )}
      </div>
      <div
        className="content"
        dangerouslySetInnerHTML={{ __html: renderedHtml }}
      />
    </div>
  );
}
```

**Example: Updating a Document**

```typescript
// src/components/DocumentEditor.tsx

import React, { useState } from 'react';
import { DesktopApi } from '../api/desktop';

export function DocumentEditor({ document }: { document: Document }) {
  const [title, setTitle] = useState(document.title);
  const [content, setContent] = useState(document.content);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSuccess(false);

    try {
      const response = await DesktopApi.updateDocument({
        id: document.id,
        title: title !== document.title ? title : undefined,
        content: content !== document.content ? content : undefined,
      });

      console.log('Document updated:', response);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err.message || 'Failed to update document');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div>
      <input
        type="text"
        placeholder="Document title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
      />
      <textarea
        placeholder="Document content"
        value={content}
        onChange={(e) => setContent(e.target.value)}
      />
      <button onClick={handleSave} disabled={saving}>
        {saving ? 'Saving...' : 'Save Document'}
      </button>
      {success && <div className="success">Document saved successfully!</div>}
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

**Example: Listing Documents**

```typescript
// src/components/DocumentList.tsx

import React, { useEffect, useState } from 'react';
import { DesktopApi } from '../api/desktop';
import type { Document } from '../api/types';

export function DocumentList() {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [offset, setOffset] = useState(0);
  const [limit] = useState(20);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadDocuments = async () => {
      setLoading(true);
      setError(null);

      try {
        const response = await DesktopApi.listDocuments({ offset, limit });
        setDocuments(response.documents);
        setTotalCount(response.total_count);
      } catch (err) {
        setError(err.message || 'Failed to load documents');
      } finally {
        setLoading(false);
      }
    };

    loadDocuments();
  }, [offset, limit]);

  const handlePrevious = () => {
    if (offset > 0) {
      setOffset(offset - limit);
    }
  };

  const handleNext = () => {
    if (offset + limit < totalCount) {
      setOffset(offset + limit);
    }
  };

  if (loading) return <div>Loading...</div>;
  if (error) return <div className="error">{error}</div>;

  return (
    <div>
      <h2>Documents ({totalCount})</h2>
      <ul>
        {documents.map((doc) => (
          <li key={doc.id}>
            <a href={`/documents/${doc.id}`}>{doc.title}</a>
            <span>Created: {new Date(doc.created_at).toLocaleDateString()}</span>
          </li>
        ))}
      </ul>
      <div className="pagination">
        <button onClick={handlePrevious} disabled={offset === 0}>
          Previous
        </button>
        <span>
          {offset + 1} - {Math.min(offset + limit, totalCount)} of {totalCount}
        </span>
        <button onClick={handleNext} disabled={offset + limit >= totalCount}>
          Next
        </button>
      </div>
    </div>
  );
}
```

#### 9.3.2. Repository Management

**Example: Adding a Repository**

```typescript
// src/components/RepositoryAdder.tsx

import React, { useState } from 'react';
import { DesktopApi } from '../api/desktop';

export function RepositoryAdder() {
  const [name, setName] = useState('');
  const [path, setPath] = useState('');
  const [remoteUrl, setRemoteUrl] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAdd = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await DesktopApi.addRepository({
        name,
        path,
        remote_url: remoteUrl || undefined,
      });

      console.log('Repository added:', response.repository);
      // Refresh repository list
      // onRepositoryAdded();
    } catch (err) {
      setError(err.message || 'Failed to add repository');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h2>Add Repository</h2>
      <input
        type="text"
        placeholder="Repository name"
        value={name}
        onChange={(e) => setName(e.target.value)}
      />
      <input
        type="text"
        placeholder="Repository path"
        value={path}
        onChange={(e) => setPath(e.target.value)}
      />
      <input
        type="text"
        placeholder="Remote URL (optional)"
        value={remoteUrl}
        onChange={(e) => setRemoteUrl(e.target.value)}
      />
      <button onClick={handleAdd} disabled={loading}>
        {loading ? 'Adding...' : 'Add Repository'}
      </button>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

**Example: Syncing a Repository**

```typescript
// src/components/RepositorySyncer.tsx

import React, { useState, useEffect } from 'react';
import { DesktopApi } from '../api/desktop';
import { DesktopEvents } from '../api/events';
import type { Repository } from '../api/types';

export function RepositorySyncer({ repository }: { repository: Repository }) {
  const [syncing, setSyncing] = useState(false);
  const [syncStatus, setSyncStatus] = useState(repository.sync_status);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Listen to sync events
    const unsubscribeStarted = DesktopEvents.onRepositorySyncStarted((event) => {
      if (event.repository_id === repository.id) {
        setSyncing(true);
      }
    });

    const unsubscribeCompleted = DesktopEvents.onRepositorySyncCompleted((event) => {
      if (event.repository_id === repository.id) {
        setSyncing(false);
        setSyncStatus(event.sync_status);
      }
    });

    return () => {
      unsubscribeStarted();
      unsubscribeCompleted();
    };
  }, [repository.id]);

  const handleSync = async () => {
    setSyncing(true);
    setError(null);

    try {
      const response = await DesktopApi.syncRepository({ id: repository.id });
      setSyncStatus(response.sync_status);
    } catch (err) {
      setError(err.message || 'Failed to sync repository');
      setSyncing(false);
    }
  };

  return (
    <div>
      <h3>{repository.name}</h3>
      <p>Status: {syncStatus}</p>
      <button onClick={handleSync} disabled={syncing}>
        {syncing ? 'Syncing...' : 'Sync Repository'}
      </button>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

#### 9.3.3. Search

**Example: Searching Documents**

```typescript
// src/components/DocumentSearch.tsx

import React, { useState } from 'react';
import { DesktopApi } from '../api/desktop';
import type { Document } from '../api/types';

export function DocumentSearch() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Document[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = async () => {
    if (!query.trim()) {
      setResults([]);
      setTotalCount(0);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await DesktopApi.searchDocuments({
        query,
        sort_by: 'updated_at',
        sort_order: 'desc',
      });

      setResults(response.results);
      setTotalCount(response.total_count);
    } catch (err) {
      setError(err.message || 'Failed to search documents');
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  return (
    <div>
      <h2>Search Documents</h2>
      <input
        type="text"
        placeholder="Search query"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyPress={handleKeyPress}
      />
      <button onClick={handleSearch} disabled={loading}>
        {loading ? 'Searching...' : 'Search'}
      </button>
      {error && <div className="error">{error}</div>}
      {totalCount > 0 && <p>Found {totalCount} result(s)</p>}
      <ul>
        {results.map((doc) => (
          <li key={doc.id}>
            <a href={`/documents/${doc.id}`}>{doc.title}</a>
          </li>
        ))}
      </ul>
    </div>
  );
}
```

**Requirement Traceability:**
- REQ-IPC-021: Type Safety
- REQ-IPC-022: Error Handling
- REQ-IPC-031: Event Emission

### 9.4. Best Practices

#### 9.4.1. Error Handling

**Principle:** All IPC commands must handle errors gracefully and provide meaningful error messages.

**Best Practices:**
1. **Always use try-catch blocks** when invoking IPC commands
2. **Validate input** before invoking commands
3. **Display user-friendly error messages** for common errors
4. **Log detailed errors** for debugging purposes
5. **Retry failed operations** when appropriate

**Example:**
```typescript
async function safeDocumentOperation(id: string) {
  try {
    const response = await DesktopApi.getDocument({ id });
    return response;
  } catch (error) {
    // Log detailed error
    console.error('Failed to get document:', error);

    // Display user-friendly error
    if (error.code === 'DOCUMENT_NOT_FOUND') {
      throw new Error('Document not found. It may have been deleted.');
    } else if (error.code === 'PERMISSION_DENIED') {
      throw new Error('You do not have permission to access this document.');
    } else {
      throw new Error('Failed to load document. Please try again.');
    }
  }
}
```

#### 9.4.2. Performance Optimization

**Principle:** Optimize API usage to minimize latency and maximize throughput.

**Best Practices:**
1. **Use caching** for frequently accessed data
2. **Batch operations** when possible
3. **Debounce rapid requests** (e.g., search input)
4. **Lazy load large lists** with pagination
5. **Cancel in-flight requests** when navigating away

**Example:**
```typescript
import { debounce } from 'lodash';

const debouncedSearch = debounce(async (query: string) => {
  try {
    const response = await DesktopApi.searchDocuments({ query });
    return response.results;
  } catch (error) {
    console.error('Search failed:', error);
    return [];
  }
}, 300);

// Usage
debouncedSearch('search query');
```

#### 9.4.3. Event Handling

**Principle:** Subscribe to events reactively and clean up subscriptions properly.

**Best Practices:**
1. **Always unsubscribe** from events when components unmount
2. **Filter events** by relevant IDs when necessary
3. **Handle event errors** gracefully
4. **Use event listeners** for real-time updates instead of polling

**Example:**
```typescript
import { useEffect } from 'react';
import { DesktopEvents } from '../api/events';

function DocumentComponent({ documentId }: { documentId: string }) {
  useEffect(() => {
    const unsubscribe = DesktopEvents.onDocumentUpdated((event) => {
      // Only process events for this document
      if (event.document_id === documentId) {
        console.log('Document updated:', event);
        // Refresh document data
      }
    });

    return () => {
      unsubscribe();
    };
  }, [documentId]);

  return <div>...</div>;
}
```

#### 9.4.4. Type Safety

**Principle:** Leverage TypeScript's type system to ensure type safety across the IPC boundary.

**Best Practices:**
1. **Generate types** from Rust definitions using Tauri CLI
2. **Use strict mode** in TypeScript configuration
3. **Avoid `any` types** - use proper type definitions
4. **Validate runtime types** for external inputs

**Example:**
```typescript
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictBindCallApply": true,
    "strictPropertyInitialization": true,
    "noImplicitThis": true,
    "alwaysStrict": true
  }
}
```

#### 9.4.5. Security

**Principle:** Follow security best practices to protect against vulnerabilities.

**Best Practices:**
1. **Validate all inputs** before processing
2. **Sanitize HTML** before rendering
3. **Use HTTPS** for remote operations
4. **Implement rate limiting** for API calls
5. **Never expose sensitive data** in error messages

**Example:**
```typescript
import DOMPurify from 'dompurify';

function renderHtml(html: string): string {
  // Sanitize HTML before rendering
  return DOMPurify.sanitize(html);
}

// Usage
<div dangerouslySetInnerHTML={{ __html: renderHtml(renderedHtml) }} />
```

**Requirement Traceability:**
- REQ-SEC-001: Input Validation
- REQ-SEC-002: Output Sanitization
- REQ-SEC-003: Rate Limiting

**ADR Reference:**
- [ADR-010: Security Architecture](../../specs/02_adrs/010_security_architecture.md) - Security Controls section
---

## 10. REFERENCES

This section provides references to all relevant documents, standards, and external resources referenced throughout this specification.

### 10.1. Internal References

#### 10.1.1. Architecture Decision Records (ADRs)

| ADR ID | Title | Location | Relevance |
|---------|-------|-----------|------------|
| **ADR-002** | Tauri for Desktop Application | [`.specs/02_adrs/002_tauri_for_desktop_application.md`](../../specs/02_adrs/002_tauri_for_desktop_application.md) | Framework selection and architecture |
| **ADR-009** | IPC Communication Architecture | [`.specs/02_adrs/009_ipc_communication_architecture.md`](../../specs/02_adrs/009_ipc_communication_architecture.md) | IPC patterns and protocols |
| **ADR-010** | Security Architecture | [`.specs/02_adrs/010_security_architecture.md`](../../specs/02_adrs/010_security_architecture.md) | Security controls and threat mitigation |

**ADR-002: Tauri for Desktop Application**
- **Decision:** Selected Tauri v2.10.0 as the desktop application framework
- **Rationale:** Lightweight bundle size, memory safety, cross-platform support, modern WebView integration
- **Key Points:**
  - Rust backend provides memory safety and performance
  - WebView frontend enables web technologies
  - Cross-platform support (Windows, macOS, Linux)
  - Small bundle size compared to Electron
  - Active community and ecosystem
- **Impact on API:** Defines the IPC communication mechanism, type generation, and security model

**ADR-009: IPC Communication Architecture**
- **Decision:** Defined IPC communication architecture using Tauri's mechanisms
- **Rationale:** Type safety, efficient serialization, security controls, bidirectional communication
- **Key Points:**
  - Type-safe IPC commands using Rust and TypeScript
  - Efficient serialization using serde
  - Security controls including input validation and rate limiting
  - Bidirectional communication with commands and events
  - Comprehensive error handling
  - Performance optimization with caching and batching
- **Impact on API:** Defines all IPC command patterns, event emission patterns, and error handling mechanisms

**ADR-010: Security Architecture**
- **Decision:** Defined defense-in-depth security architecture
- **Rationale:** Multiple layers of security controls to protect against vulnerabilities
- **Key Points:**
  - Memory safety using Rust
  - Capability-based access control
  - Input validation and sanitization
  - Encryption at rest and in transit
  - Audit logging
  - Supply chain security
  - Fail-safe error handling
- **Impact on API:** Defines all security controls, authentication, authorization, and audit logging requirements

#### 10.1.2. Requirements

| Requirement ID | Title | Location | Relevance |
|----------------|-------|-----------|------------|
| **REQ-DESK-001** | Application Lifecycle | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Application initialization and shutdown |
| **REQ-DESK-002** | Window Management | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Window creation and management |
| **REQ-DESK-003** | Window State Persistence | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Window state persistence |
| **REQ-DESK-004** | System Tray Integration | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | System tray functionality |
| **REQ-DESK-005** | Native Notifications | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Native notification support |
| **REQ-DESK-006** | Global Hotkeys | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Global hotkey support |
| **REQ-DESK-007** | Dark Mode Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Dark mode UI theme |
| **REQ-DESK-008** | Responsive UI | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Responsive UI design |
| **REQ-DESK-009** | Accessibility Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Accessibility features |
| **REQ-DESK-010** | Keyboard Navigation | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Keyboard navigation support |
| **REQ-DESK-011** | Screen Reader Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Screen reader compatibility |
| **REQ-DESK-012** | High Contrast Mode | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | High contrast mode |
| **REQ-DESK-013** | Font Size Adjustment | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Font size adjustment |
| **REQ-DESK-014** | Focus Indicators | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Focus indicators |
| **REQ-DESK-015** | Color Blindness Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Color blindness support |
| **REQ-DESK-016** | Local Document Storage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Local document storage |
| **REQ-DESK-017** | Document Metadata | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document metadata |
| **REQ-DESK-018** | Document Versioning | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document versioning |
| **REQ-DESK-019** | Document Search | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document search functionality |
| **REQ-DESK-020** | Document Filtering | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document filtering |
| **REQ-DESK-021** | Document Sorting | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document sorting |
| **REQ-DESK-022** | Document Pagination | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document pagination |
| **REQ-DESK-023** | Document Export | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document export |
| **REQ-DESK-024** | Document Import | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document import |
| **REQ-DESK-025** | Document Preview | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document preview |
| **REQ-DESK-026** | Document Editing | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Document editing |
| **REQ-DESK-027** | Markdown Rendering | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Markdown rendering |
| **REQ-DESK-028** | Code Syntax Highlighting | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Code syntax highlighting |
| **REQ-DESK-029** | Image Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Image support |
| **REQ-DESK-030** | Table Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Table support |
| **REQ-DESK-031** | Math Formula Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Math formula support |
| **REQ-DESK-032** | Diagram Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Diagram support |
| **REQ-DESK-033** | Git Integration | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Git integration |
| **REQ-DESK-034** | Repository Management | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Repository management |
| **REQ-DESK-035** | Branch Management | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Branch management |
| **REQ-DESK-036** | Commit Management | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Commit management |
| **REQ-DESK-037** | Merge Management | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Merge management |
| **REQ-DESK-038** | Conflict Resolution | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Conflict resolution |
| **REQ-DESK-039** | Remote Sync | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Remote sync |
| **REQ-DESK-040** | Offline Mode | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Offline mode |
| **REQ-DESK-041** | LRU Cache | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | LRU cache |
| **REQ-DESK-042** | Cache Invalidation | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Cache invalidation |
| **REQ-DESK-043** | File System Access | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | File system access |
| **REQ-DESK-044** | Clipboard Integration | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Clipboard integration |
| **REQ-DESK-045** | Drag and Drop | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Drag and drop |
| **REQ-DESK-046** | File Dialogs | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | File dialogs |
| **REQ-DESK-047** | Server Health Monitoring | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Server health monitoring |
| **REQ-DESK-048** | Server Status Display | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Server status display |
| **REQ-DESK-049** | Server Restart | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Server restart |
| **REQ-DESK-050** | Server Configuration | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Server configuration |
| **REQ-DESK-051** | Server Logs | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Server logs |
| **REQ-DESK-052** | Server Metrics | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Server metrics |
| **REQ-DESK-053** | Server Alerts | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Server alerts |
| **REQ-DESK-054** | Auto-Update | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Auto-update |
| **REQ-DESK-055** | Update Notifications | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Update notifications |
| **REQ-DESK-056** | Update Download | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Update download |
| **REQ-DESK-057** | Update Installation | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Update installation |
| **REQ-DESK-058** | Error Reporting | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Error reporting |
| **REQ-DESK-059** | Crash Reporting | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Crash reporting |
| **REQ-DESK-060** | Telemetry | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Telemetry |
| **REQ-DESK-061** | Analytics | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Analytics |
| **REQ-DESK-062** | User Feedback | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | User feedback |
| **REQ-DESK-063** | Feature Requests | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Feature requests |
| **REQ-DESK-064** | Bug Reports | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Bug reports |
| **REQ-DESK-065** | Documentation | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Documentation |
| **REQ-DESK-066** | Tutorials | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Tutorials |
| **REQ-DESK-067** | Examples | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Examples |
| **REQ-DESK-068** | FAQ | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | FAQ |
| **REQ-DESK-069** | Support | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Support |
| **REQ-DESK-070** | Community | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Community |
| **REQ-DESK-071** | Contributing | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Contributing |
| **REQ-DESK-072** | Code of Conduct | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Code of conduct |
| **REQ-DESK-073** | License | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | License |
| **REQ-DESK-074** | Privacy Policy | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Privacy policy |
| **REQ-DESK-075** | Terms of Service | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Terms of service |
| **REQ-DESK-076** | Data Collection | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data collection |
| **REQ-DESK-077** | Data Storage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data storage |
| **REQ-DESK-078** | Data Sharing | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data sharing |
| **REQ-DESK-079** | Data Deletion | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data deletion |
| **REQ-DESK-080** | Data Export | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data export |
| **REQ-DESK-081** | Data Import | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data import |
| **REQ-DESK-082** | Data Backup | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data backup |
| **REQ-DESK-083** | Data Restore | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data restore |
| **REQ-DESK-084** | Data Encryption | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data encryption |
| **REQ-DESK-085** | Data Compression | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Data compression |
| **REQ-DESK-086** | Hot-Reload Latency | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Hot-reload latency |
| **REQ-DESK-087** | Initial Load Time | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Initial load time |
| **REQ-DESK-088** | Large File Handling | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Large file handling |
| **REQ-DESK-089** | Memory Usage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Memory usage |
| **REQ-DESK-090** | Responsive UI | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Responsive UI |
| **REQ-DESK-091** | Memory Usage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Memory usage |
| **REQ-DESK-092** | CPU Usage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | CPU usage |
| **REQ-DESK-093** | Disk Usage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Disk usage |
| **REQ-DESK-094** | Network Usage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Network usage |
| **REQ-DESK-095** | Battery Usage | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Battery usage |
| **REQ-DESK-096** | Thermal Management | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Thermal management |
| **REQ-DESK-097** | Power Management | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Power management |
| **REQ-DESK-098** | Background Tasks | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Background tasks |
| **REQ-DESK-099** | Scheduled Tasks | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Scheduled tasks |
| **REQ-DESK-100** | Task Queuing | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task queuing |
| **REQ-DESK-101** | Task Prioritization | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task prioritization |
| **REQ-DESK-102** | Task Cancellation | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task cancellation |
| **REQ-DESK-103** | Task Retry | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task retry |
| **REQ-DESK-104** | Task Timeout | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task timeout |
| **REQ-DESK-105** | Task Progress | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task progress |
| **REQ-DESK-106** | Task Notifications | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task notifications |
| **REQ-DESK-107** | Task History | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task history |
| **REQ-DESK-108** | Task Logging | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task logging |
| **REQ-DESK-109** | Task Metrics | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task metrics |
| **REQ-DESK-110** | Task Analytics | [`.specs/04_future_state/reqs/desktop_requirements.md`](../../specs/04_future_state/reqs/desktop_requirements.md) | Task analytics |

**Requirements Summary:**
- **Application Lifecycle:** REQ-DESK-001 through REQ-DESK-015
- **Local Data Management:** REQ-DESK-016 through REQ-DESK-032
- **Git Integration:** REQ-DESK-033 through REQ-DESK-040
- **Native OS Integration:** REQ-DESK-041 through REQ-DESK-046
- **Server Integration:** REQ-DESK-047 through REQ-DESK-053
- **Update Management:** REQ-DESK-054 through REQ-DESK-057
- **Error Reporting:** REQ-DESK-058 through REQ-DESK-064
- **Documentation:** REQ-DESK-065 through REQ-DESK-072
- **Legal:** REQ-DESK-073 through REQ-DESK-075
- **Data Management:** REQ-DESK-076 through REQ-DESK-085
- **Performance:** REQ-DESK-086 through REQ-DESK-090
- **Resource Management:** REQ-DESK-091 through REQ-DESK-097
- **Task Management:** REQ-DESK-098 through REQ-DESK-110

#### 10.1.3. Design Elements

| Design Element | Location | Relevance |
|---------------|-----------|------------|
| **Desktop Application Architecture** | [`.specs/04_future_state/design/desktop_design.md`](../../specs/04_future_state/design/desktop_design.md) | Overall architecture and component design |
| **Application State** | [`.specs/04_future_state/design/desktop_design.md`](../../specs/04_future_state/design/desktop_design.md) | Application state management |
| **IPC Command Handlers** | [`.specs/04_future_state/design/desktop_design.md`](../../specs/04_future_state/design/desktop_design.md) | IPC command handler implementation |
| **UI Components** | [`.specs/04_future_state/design/desktop_design.md`](../../specs/04_future_state/design/desktop_design.md) | UI component design |
| **Type Definitions** | [`.specs/04_future_state/design/desktop_design.md`](../../specs/04_future_state/design/desktop_design.md) | Rust and TypeScript type definitions |

#### 10.1.4. Standards

| Standard | Location | Relevance |
|----------|-----------|------------|
| **Coding Standards** | [`.specs/01_standards/coding_standards.md`](../../specs/01_standards/coding_standards.md) | Documentation structure and writing style |
| **Documentation Standards** | [`.specs/01_standards/coding_standards.md`](../../specs/01_standards/coding_standards.md) | PhD thesis level rigor requirements |

### 10.2. External References

#### 10.2.1. Tauri Documentation

| Resource | URL | Relevance |
|----------|------|------------|
| **Tauri Documentation** | https://tauri.app/v1/guides/ | Framework documentation |
| **Tauri IPC Guide** | https://tauri.app/v1/guides/features/command/ | IPC command implementation |
| **Tauri Events Guide** | https://tauri.app/v1/guides/features/events/ | Event emission and listening |
| **Tauri Security Guide** | https://tauri.app/v1/guides/security/ | Security best practices |
| **Tauri Type Generation** | https://tauri.app/v1/guides/features/tauri-cli/ | Type generation from Rust definitions |

#### 10.2.2. Rust Documentation

| Resource | URL | Relevance |
|----------|------|------------|
| **Rust Book** | https://doc.rust-lang.org/book/ | Rust language fundamentals |
| **Rust Reference** | https://doc.rust-lang.org/reference/ | Rust language reference |
| **Tokio Documentation** | https://tokio.rs/ | Async runtime |
| **Serde Documentation** | https://serde.rs/ | Serialization framework |
| **Tauri API Documentation** | https://docs.rs/tauri/ | Tauri API reference |

#### 10.2.3. TypeScript Documentation

| Resource | URL | Relevance |
|----------|------|------------|
| **TypeScript Handbook** | https://www.typescriptlang.org/docs/handbook/intro.html | TypeScript fundamentals |
| **TypeScript Deep Dive** | https://basarat.gitbook.io/typescript/ | Advanced TypeScript concepts |
| **Tauri TypeScript API** | https://tauri.app/v1/api/js/ | Tauri JavaScript/TypeScript API |

#### 10.2.4. OpenAPI Specification

| Resource | URL | Relevance |
|----------|------|------------|
| **OpenAPI 3.1.0 Specification** | https://spec.openapis.org/oas/v3.1.0 | API specification format |
| **OpenAPI Tools** | https://openapi-generator.tech/ | Code generation tools |
| **Swagger Editor** | https://editor.swagger.io/ | API specification editor |

#### 10.2.5. Security Standards

| Resource | URL | Relevance |
|----------|------|------------|
| **OWASP Top 10** | https://owasp.org/www-project-top-ten/ | Web application security risks |
| **CWE Top 25** | https://cwe.mitre.org/top25/ | Common software weaknesses |
| **ISO/IEC 27001** | https://www.iso.org/standard/27001 | Information security management |
| **NIST Cybersecurity Framework** | https://www.nist.gov/cyberframework | Cybersecurity framework |

#### 10.2.6. Performance Standards

| Resource | URL | Relevance |
|----------|------|------------|
| **Web Performance** | https://web.dev/performance/ | Web performance best practices |
| **Performance API** | https://developer.mozilla.org/en-US/docs/Web/API/Performance | Browser performance APIs |
| **Rust Performance** | https://doc.rust-lang.org/nomicon/ | Rust performance guide |

### 10.3. Version History

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| **1.0.0** | 2026-02-05 | Technical Writer | Initial release of Desktop API Specification |

### 10.4. Document Control

**Document ID:** TACHYON-API-001-V1.0  
**Document Title:** Desktop API Specification  
**Document Owner:** Technical Writer  
**Document Status:** Draft  
**Classification:** Internal  
**Review Cycle:** Quarterly  

**Change Control:**
- All changes must be reviewed and approved by the Technical Lead
- Minor changes (typographical corrections, clarifications) may be approved by the Technical Writer
- Major changes (API additions, deprecations) must be approved by the Architecture Team
- All changes must be documented in the Version History section

**Distribution:**
- Development Team
- QA Team
- Documentation Team
- Architecture Team

**Retention:**
- This document must be retained for the lifetime of the Tachyon project
- Historical versions must be retained for at least 2 years
- Document metadata must be preserved in the document management system

---

## APPENDIX A: GLOSSARY

| Term | Definition |
|-------|------------|
| **IPC** | Inter-Process Communication - mechanism for exchanging data between processes |
| **Tauri** | Cross-platform desktop application framework using Rust and WebView |
| **WebView** | Browser component for rendering web content in native applications |
| **Command** | IPC operation invoked from frontend to backend |
| **Event** | IPC message emitted from backend to frontend |
| **Type Safety** | Compile-time type checking to prevent type-related errors |
| **Serialization** | Converting data structures to a format suitable for transmission |
| **Deserialization** | Converting serialized data back to data structures |
| **LRU Cache** | Least Recently Used cache eviction strategy |
| **Rate Limiting** | Restricting the rate of API requests to prevent abuse |
| **Input Validation** | Verifying that input data meets specified constraints |
| **Output Sanitization** | Cleaning output data to prevent security vulnerabilities |
| **Audit Logging** | Recording security-relevant events for compliance and debugging |
| **Semantic Versioning** | Versioning scheme using MAJOR.MINOR.PATCH format |
| **OpenAPI** | Specification for defining RESTful APIs |
| **Swagger** | Toolset for implementing and documenting OpenAPI specifications |

---

## APPENDIX B: ACRONYMS

| Acronym | Full Form |
|---------|-----------|
| **ADR** | Architecture Decision Record |
| **API** | Application Programming Interface |
| **CORS** | Cross-Origin Resource Sharing |
| **CRUD** | Create, Read, Update, Delete |
| **DOM** | Document Object Model |
| **HTML** | HyperText Markup Language |
| **HTTP** | Hypertext Transfer Protocol |
| **HTTPS** | Hypertext Transfer Protocol Secure |
| **IPC** | Inter-Process Communication |
| **JSON** | JavaScript Object Notation |
| **LRU** | Least Recently Used |
| **MD** | Markdown |
| **OWASP** | Open Web Application Security Project |
| **PCI DSS** | Payment Card Industry Data Security Standard |
| **REST** | Representational State Transfer |
| **RSA** | Rivest-Shamir-Adleman |
| **TLS** | Transport Layer Security |
| **UI** | User Interface |
| **UUID** | Universally Unique Identifier |
| **W3C** | World Wide Web Consortium |
| **WCAG** | Web Content Accessibility Guidelines |
| **XML** | Extensible Markup Language |

---

**END OF DOCUMENT**
