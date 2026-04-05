# TACHYON: WEB API SPECIFICATION

**Document ID:** TACHYON-API-003-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Technical Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [API Design Principles](#2-api-design-principles)
3. [Versioning Strategy](#3-versioning-strategy)
4. [Web Client API](#4-web-client-api)
5. [Web Components API](#5-web-components-api)
6. [API Security](#6-api-security)
7. [API Performance](#7-api-performance)
8. [API Documentation](#8-api-documentation)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides a comprehensive specification of the Tachyon Web API, defining the interfaces and protocols for communication between the Leptos-based web frontend and the Axum-based server component. The Web API specification encompasses both HTTP/2 RESTful endpoints and WebSocket real-time communication channels, ensuring type-safe, performant, and secure interactions across the application architecture.

### 1.2. Scope

The Web API specification covers:
- HTTP/2 RESTful API endpoints for server communication
- WebSocket API for real-time bidirectional communication
- Client-side API functions and abstractions
- Component communication interfaces
- State synchronization mechanisms
- Error handling and recovery protocols
- Security and authentication mechanisms
- Performance optimization strategies

Out of scope:
- Desktop application IPC protocols (covered in desktop API specification)
- Server internal APIs (covered in server API specification)
- Core rendering engine APIs (covered in rendering engine specification)

### 1.3. Document Dependencies

This document depends on the following documents:
- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-REQ-WEB-V1.0](../../.specs/04_future_state/reqs/web_requirements.md) - Web Frontend Requirements
- [TACHYON-DES-WD-V1.0](../../.specs/04_future_state/design/web_design.md) - Web Frontend Design
- [TACHYON-ADR-004-V1.0](../../.specs/02_adrs/004_leptos_for_web_frontend.md) - ADR-004: Leptos for Web Frontend
- [TACHYON-ADR-005-V1.0](../../.specs/02_adrs/005_bun_for_javascript_runtime.md) - ADR-005: Bun for JavaScript Runtime
- [TACHYON-TMA-V1.0](../../.specs/03_threat_model/analysis.md) - Threat Model Analysis

### 1.4. Terminology

| Term | Definition |
|------|------------|
| **API Client** | Type-safe HTTP client abstraction for server communication |
| **WebSocket Client** | Real-time bidirectional communication client for live updates |
| **Signal** | Leptos reactive primitive for state management |
| **Hydration** | Process of attaching event listeners to server-rendered HTML |
| **SSR** | Server-Side Rendering - initial HTML rendering on server |
| **CSR** | Client-Side Rendering - interactive rendering in browser |
| **WASM** | WebAssembly - binary instruction format for web |

---

## 2. API DESIGN PRINCIPLES

### 2.1. Architectural Principles

The Tachyon Web API adheres to the following architectural principles:

#### 2.1.1. Type Safety First

All API interfaces are defined with strict type safety using TypeScript and Rust type definitions. The type system ensures compile-time correctness and prevents entire classes of runtime errors.

**Implementation:**
- TypeScript interfaces for all request/response types
- Rust structs for WASM module exports
- Generated type definitions from Rust backend
- Strict null checking enabled in TypeScript configuration

**Rationale:** Type safety reduces bugs, improves developer experience, and enables confident refactoring [REQ-WEB-037].

#### 2.1.2. Reactive State Management

The API leverages Leptos's fine-grained reactivity model for efficient state updates. State changes propagate automatically to dependent components without manual re-rendering.

**Implementation:**
- Leptos signals for reactive state
- Automatic dependency tracking
- Minimal DOM updates
- No virtual DOM overhead

**Rationale:** Fine-grained reactivity provides superior performance compared to virtual DOM diffing [ADR-004].

#### 2.1.3. Isomorphic Architecture

The API supports both server-side rendering (SSR) and client-side rendering (CSR), enabling code reuse between server and client environments.

**Implementation:**
- Shared component definitions
- Leptos SSR with leptos_axum
- Client-side hydration
- Consistent API interfaces

**Rationale:** Isomorphic architecture provides fast initial loads and progressive enhancement [ADR-004].

#### 2.1.4. Performance Optimization

The API is designed for optimal performance with minimal latency and efficient resource utilization.

**Implementation:**
- HTTP/2 for multiplexed requests
- WebSocket for real-time updates
- Request deduplication
- Intelligent caching strategies
- Lazy loading of resources

**Rationale:** Performance optimization ensures responsive user experience and efficient resource usage [REQ-WEB-066, REQ-WEB-067].

#### 2.1.5. Security by Design

Security considerations are integrated into all API design decisions, not added as an afterthought.

**Implementation:**
- TLS 1.3 for all communications
- Input validation and sanitization
- Authentication and authorization checks
- Rate limiting and throttling
- Secure session management

**Rationale:** Security by design prevents vulnerabilities and protects sensitive data [TACHYON-TMA-V1.0].

### 2.2. API Design Patterns

#### 2.2.1. RESTful Conventions

HTTP/2 RESTful endpoints follow REST architectural principles for consistent and predictable API design.

**Conventions:**
- Resource-based URLs (e.g., `/documents/{id}`)
- HTTP methods for semantic operations (GET, POST, PUT, DELETE)
- Status codes for response semantics (200, 201, 400, 401, 403, 404, 500)
- Content negotiation for response formats
- HATEOAS for discoverability

**Example:**
```
GET    /api/documents          - List all documents
GET    /api/documents/{id}     - Get specific document
POST   /api/documents          - Create new document
PUT    /api/documents/{id}     - Update document
DELETE /api/documents/{id}     - Delete document
```

#### 2.2.2. WebSocket Protocol

WebSocket communication follows a structured message protocol for type-safe real-time updates.

**Message Format:**
```typescript
interface WebSocketMessage {
  type: MessageType;
  payload: unknown;
  timestamp: number;
  messageId: string;
}

type MessageType =
  | "document_update"
  | "user_presence"
  | "sync_status"
  | "error"
  | "ping"
  | "pong";
```

**Rationale:** Structured message format enables type-safe parsing and handling of real-time events [REQ-WEB-041].

#### 2.2.3. Error Handling Convention

All API errors follow a consistent error response format for predictable client-side error handling.

**Error Response Format:**
```typescript
interface ApiError {
  code: ErrorCode;
  message: string;
  details?: Record<string, unknown>;
  timestamp: number;
  requestId: string;
}

type ErrorCode =
  | "VALIDATION_ERROR"
  | "AUTHENTICATION_ERROR"
  | "AUTHORIZATION_ERROR"
  | "NOT_FOUND"
  | "CONFLICT"
  | "RATE_LIMIT_EXCEEDED"
  | "INTERNAL_ERROR";
```

**Rationale:** Consistent error format enables predictable error handling and user-friendly error messages [REQ-WEB-038].

---

## 3. VERSIONING STRATEGY

### 3.1. Semantic Versioning

The Web API follows Semantic Versioning 2.0.0 (SemVer) for clear communication of changes.

**Version Format:** `MAJOR.MINOR.PATCH`

- **MAJOR:** Incompatible API changes
- **MINOR:** Backwards-compatible functionality additions
- **PATCH:** Backwards-compatible bug fixes

**Example:**
- `1.0.0` → `1.1.0` - Added new endpoint
- `1.1.0` → `1.1.1` - Fixed bug in existing endpoint
- `1.1.1` → `2.0.0` - Breaking change to request format

### 3.2. API Versioning in URLs

API endpoints include version prefix for clear version identification.

**Format:** `/api/v{MAJOR}/{resource}`

**Examples:**
- `/api/v1/documents` - Version 1 documents API
- `/api/v2/documents` - Version 2 documents API (breaking changes)

### 3.3. Deprecation Policy

Deprecated API endpoints follow a clear deprecation policy to enable smooth transitions.

**Deprecation Process:**
1. Announce deprecation in release notes
2. Add `Deprecation` header to deprecated endpoints
3. Maintain deprecated endpoints for minimum 6 months
4. Remove deprecated endpoints in next major version

**Deprecation Header:**
```http
Deprecation: true; Sunset=2026-08-01; Link=/api/v2/documents
```

### 3.4. Backwards Compatibility

The API maintains backwards compatibility within major versions to prevent breaking existing clients.

**Compatibility Rules:**
- New fields may be added to responses
- Request fields may be made optional
- Enum values may be added (never removed)
- Default values must be provided for new required fields
- Removal of fields requires major version increment

**Rationale:** Backwards compatibility enables client upgrades without service disruption.

---

## 4. WEB CLIENT API

### 4.1. API Client Initialization

The Web Client API provides a type-safe HTTP client abstraction for communication with the Axum server component.

#### 4.1.1. ApiClient Interface

**TypeScript Interface:**
```typescript
/**
 * Type-safe HTTP client for server communication.
 * 
 * @remarks
 * The ApiClient provides type-safe methods for HTTP/2 communication with
 * the Axum server. All methods are generic and support automatic
 * request/response serialization.
 * 
 * @example
 * ```typescript
 * const client = new ApiClient({
 *   baseUrl: 'https://api.example.com',
 *   authToken: createSignal<string | null>(null)
 * });
 * 
 * const documents = await client.get<Document[]>('/api/v1/documents');
 * ```
 */
export class ApiClient {
  /**
   * Creates a new ApiClient instance.
   * 
   * @param config - Client configuration options
   * @throws {ConfigurationError} When configuration is invalid
   */
  constructor(config: ApiClientConfig);

  /**
   * Performs an HTTP GET request.
   * 
   * @template T - Expected response type
   * @param endpoint - API endpoint path (e.g., '/api/v1/documents')
   * @param options - Optional request options
   * @returns Promise resolving to typed response
   * @throws {ApiError} When request fails
   */
  async get<T>(endpoint: string, options?: RequestOptions): Promise<T>;

  /**
   * Performs an HTTP POST request.
   * 
   * @template T - Expected response type
   * @template B - Request body type
   * @param endpoint - API endpoint path
   * @param body - Request body data
   * @param options - Optional request options
   * @returns Promise resolving to typed response
   * @throws {ApiError} When request fails
   */
  async post<T, B>(endpoint: string, body: B, options?: RequestOptions): Promise<T>;

  /**
   * Performs an HTTP PUT request.
   * 
   * @template T - Expected response type
   * @template B - Request body type
   * @param endpoint - API endpoint path
   * @param body - Request body data
   * @param options - Optional request options
   * @returns Promise resolving to typed response
   * @throws {ApiError} When request fails
   */
  async put<T, B>(endpoint: string, body: B, options?: RequestOptions): Promise<T>;

  /**
   * Performs an HTTP DELETE request.
   * 
   * @template T - Expected response type
   * @param endpoint - API endpoint path
   * @param options - Optional request options
   * @returns Promise resolving to typed response
   * @throws {ApiError} When request fails
   */
  async delete<T>(endpoint: string, options?: RequestOptions): Promise<T>;

  /**
   * Cancels all pending requests.
   * 
   * @remarks
   * This method is called during navigation changes to prevent
   * requests from previous pages from completing.
   */
  cancelAllRequests(): void;
}

/**
 * Configuration options for ApiClient initialization.
 */
export interface ApiClientConfig {
  /**
   * Base URL for API requests.
   */
  baseUrl: string;

  /**
   * Reactive signal containing authentication token.
   * Token is automatically included in Authorization header.
   */
  authToken: Signal<string | null>;

  /**
   * Default timeout for requests in milliseconds.
   * @default 30000
   */
  timeout?: number;

  /**
   * Maximum number of retry attempts for failed requests.
   * @default 3
   */
  maxRetries?: number;

  /**
   * Custom fetch implementation (useful for testing).
   */
  fetch?: typeof fetch;
}

/**
 * Options for individual API requests.
 */
export interface RequestOptions {
  /**
   * Query parameters to append to URL.
   */
  params?: Record<string, string | number | boolean>;

  /**
   * Custom headers to include in request.
   */
  headers?: Record<string, string>;

  /**
   * Request timeout in milliseconds (overrides default).
   */
  timeout?: number;

  /**
   * Abort signal for request cancellation.
   */
  signal?: AbortSignal;
}
```

**Rationale:** The ApiClient provides type-safe HTTP communication with automatic token management, request deduplication, and error handling [REQ-WEB-037, REQ-WEB-039, REQ-WEB-040].

### 4.2. HTTP Client Methods

#### 4.2.1. Document API Methods

**TypeScript Interfaces:**
```typescript
/**
 * Document metadata structure.
 */
export interface DocumentMetadata {
  /**
   * Unique document identifier.
   */
  id: string;

  /**
   * Document title.
   */
  title: string;

  /**
   * Document summary (first 200 characters).
   */
  summary: string;

  /**
   * Document author user ID.
   */
  authorId: string;

  /**
   * Document creation timestamp (ISO 8601).
   */
  createdAt: string;

  /**
   * Document last modification timestamp (ISO 8601).
   */
  updatedAt: string;

  /**
   * Document tags.
   */
  tags: string[];

  /**
   * Document file path in repository.
   */
  path: string;
}

/**
 * Complete document content including metadata and body.
 */
export interface DocumentContent extends DocumentMetadata {
  /**
   * Document body content (Markdown).
   */
  content: string;

  /**
   * Document word count.
   */
  wordCount: number;

  /**
   * Document reading time in minutes.
   */
  readTimeMinutes: number;
}

/**
 * Request options for listing documents.
 */
export interface ListDocumentsOptions {
  /**
   * Number of documents per page.
   */
  limit?: number;

  /**
   * Pagination cursor for next page.
   */
  cursor?: string;

  /**
   * Filter by tags.
   */
  tags?: string[];

  /**
   * Search query string.
   */
  search?: string;

  /**
   * Sort field and direction.
   */
  sort?: SortField;
}

/**
 * Sort field options.
 */
export type SortField =
  | 'createdAt_asc'
  | 'createdAt_desc'
  | 'updatedAt_asc'
  | 'updatedAt_desc'
  | 'title_asc'
  | 'title_desc';

/**
 * Response for paginated document list.
 */
export interface PaginatedDocumentsResponse {
  /**
   * Array of documents.
   */
  documents: DocumentMetadata[];

  /**
   * Cursor for next page (null if no more pages).
   */
  nextCursor: string | null;

  /**
   * Total document count.
   */
  totalCount: number;
}

/**
 * Document API client methods.
 */
export class DocumentApiClient {
  /**
   * Lists documents with pagination and filtering.
   * 
   * @param options - List options (pagination, filters, sorting)
   * @returns Promise resolving to paginated document list
   * @throws {ApiError} When request fails
   */
  async listDocuments(options?: ListDocumentsOptions): Promise<PaginatedDocumentsResponse>;

  /**
   * Retrieves a specific document by ID.
   * 
   * @param id - Document identifier
   * @returns Promise resolving to document content
   * @throws {NotFoundError} When document does not exist
   */
  async getDocument(id: string): Promise<DocumentContent>;

  /**
   * Creates a new document.
   * 
   * @param document - Document data to create
   * @returns Promise resolving to created document
   * @throws {ValidationError} When document data is invalid
   */
  async createDocument(document: CreateDocumentRequest): Promise<DocumentContent>;

  /**
   * Updates an existing document.
   * 
   * @param id - Document identifier
   * @param document - Document data to update
   * @returns Promise resolving to updated document
   * @throws {NotFoundError} When document does not exist
   * @throws {ValidationError} When document data is invalid
   * @throws {ConflictError} When document has concurrent modifications
   */
  async updateDocument(id: string, document: UpdateDocumentRequest): Promise<DocumentContent>;

  /**
   * Deletes a document.
   * 
   * @param id - Document identifier
   * @returns Promise resolving when deletion is complete
   * @throws {NotFoundError} When document does not exist
   */
  async deleteDocument(id: string): Promise<void>;

  /**
   * Searches documents by query.
   * 
   * @param query - Search query string
   * @param options - Search options (filters, pagination)
   * @returns Promise resolving to search results
   * @throws {ApiError} When search fails
   */
  async searchDocuments(query: string, options?: SearchOptions): Promise<SearchResults>;
}

/**
 * Request for creating a new document.
 */
export interface CreateDocumentRequest {
  /**
   * Document title.
   */
  title: string;

  /**
   * Document body content (Markdown).
   */
  content: string;

  /**
   * Document tags.
   */
  tags?: string[];

  /**
   * Document file path in repository.
   */
  path?: string;
}

/**
 * Request for updating an existing document.
 */
export interface UpdateDocumentRequest {
  /**
   * Updated document title.
   */
  title?: string;

  /**
   * Updated document body content (Markdown).
   */
  content?: string;

  /**
   * Updated document tags.
   */
  tags?: string[];

  /**
   * Expected document version for optimistic concurrency control.
   */
  expectedVersion?: number;
}

/**
 * Search options.
 */
export interface SearchOptions {
  /**
   * Filter by tags.
   */
  tags?: string[];

  /**
   * Filter by date range.
   */
  dateRange?: {
    start: string;
    end: string;
  };

  /**
   * Filter by author.
   */
  authorId?: string;

  /**
   * Pagination options.
   */
  pagination?: {
    limit?: number;
    cursor?: string;
  };
}

/**
 * Search results response.
 */
export interface SearchResults {
  /**
   * Array of search result items.
   */
  results: SearchResultItem[];

  /**
   * Total result count.
   */
  totalCount: number;

  /**
   * Search execution time in milliseconds.
   */
  executionTimeMs: number;
}

/**
 * Individual search result item.
 */
export interface SearchResultItem {
  /**
   * Document metadata.
   */
  document: DocumentMetadata;

  /**
   * Relevance score (0-1).
   */
  score: number;

  /**
   * Highlighted snippet showing match context.
   */
  snippet: string;
}
```

**Rationale:** Type-safe document API methods ensure compile-time correctness and provide excellent developer experience [REQ-WEB-016, REQ-WEB-021].

#### 4.2.2. Repository API Methods

**TypeScript Interfaces:**
```typescript
/**
 * Repository metadata structure.
 */
export interface Repository {
  /**
   * Unique repository identifier.
   */
  id: string;

  /**
   * Repository name.
   */
  name: string;

  /**
   * Repository description.
   */
  description: string;

  /**
   * Repository file system path.
   */
  path: string;

  /**
   * Git branch name.
   */
  branch: string;

  /**
   * Repository sync status.
   */
  syncStatus: SyncStatus;

  /**
   * Repository creation timestamp.
   */
  createdAt: string;
}

/**
 * Repository sync status.
 */
export type SyncStatus =
  | 'synced'
  | 'syncing'
  | 'conflict'
  | 'error'
  | 'offline';

/**
 * Git status information.
 */
export interface GitStatus {
  /**
   * Current branch name.
   */
  branch: string;

  /**
   * Current commit hash.
   */
  commit: string;

  /**
   * Number of uncommitted changes.
   */
  uncommittedChanges: number;

  /**
   * Number of commits ahead of remote.
   */
  ahead: number;

  /**
   * Number of commits behind remote.
   */
  behind: number;

  /**
   * Array of changed files.
   */
  changedFiles: GitFileStatus[];
}

/**
 * Individual file status.
 */
export interface GitFileStatus {
  /**
   * File path.
   */
  path: string;

  /**
   * File status (modified, added, deleted, etc.).
   */
  status: GitFileStatusType;

  /**
   * File change summary.
   */
  summary?: string;
}

/**
 * Git file status type.
 */
export type GitFileStatusType =
  | 'modified'
  | 'added'
  | 'deleted'
  | 'renamed'
  | 'copied'
  | 'untracked';

/**
 * Repository API client methods.
 */
export class RepositoryApiClient {
  /**
   * Lists all repositories.
   * 
   * @returns Promise resolving to array of repositories
   * @throws {ApiError} When request fails
   */
  async listRepositories(): Promise<Repository[]>;

  /**
   * Retrieves a specific repository by ID.
   * 
   * @param id - Repository identifier
   * @returns Promise resolving to repository details
   * @throws {NotFoundError} When repository does not exist
   */
  async getRepository(id: string): Promise<Repository>;

  /**
   * Creates a new repository.
   * 
   * @param repository - Repository data to create
   * @returns Promise resolving to created repository
   * @throws {ValidationError} When repository data is invalid
   */
  async createRepository(repository: CreateRepositoryRequest): Promise<Repository>;

  /**
   * Updates repository configuration.
   * 
   * @param id - Repository identifier
   * @param repository - Repository data to update
   * @returns Promise resolving to updated repository
   * @throws {NotFoundError} When repository does not exist
   */
  async updateRepository(id: string, repository: UpdateRepositoryRequest): Promise<Repository>;

  /**
   * Deletes a repository.
   * 
   * @param id - Repository identifier
   * @returns Promise resolving when deletion is complete
   * @throws {NotFoundError} When repository does not exist
   */
  async deleteRepository(id: string): Promise<void>;

  /**
   * Synchronizes repository with remote.
   * 
   * @param id - Repository identifier
   * @returns Promise resolving to sync status
   * @throws {ApiError} When sync fails
   */
  async syncRepository(id: string): Promise<SyncStatus>;

  /**
   * Retrieves Git status for repository.
   * 
   * @param id - Repository identifier
   * @returns Promise resolving to Git status
   * @throws {ApiError} When status retrieval fails
   */
  async getGitStatus(id: string): Promise<GitStatus>;

  /**
   * Creates a new branch.
   * 
   * @param id - Repository identifier
   * @param branchName - New branch name
   * @param baseBranch - Base branch to branch from
   * @returns Promise resolving to updated repository
   * @throws {ApiError} When branch creation fails
   */
  async createBranch(id: string, branchName: string, baseBranch?: string): Promise<Repository>;

  /**
   * Merges branches.
   * 
   * @param id - Repository identifier
   * @param sourceBranch - Source branch to merge
   * @param targetBranch - Target branch to merge into
   * @returns Promise resolving to merge result
   * @throws {ConflictError} When merge has conflicts
   * @throws {ApiError} When merge fails
   */
  async mergeBranches(id: string, sourceBranch: string, targetBranch: string): Promise<MergeResult>;
}

/**
 * Request for creating a new repository.
 */
export interface CreateRepositoryRequest {
  /**
   * Repository name.
   */
  name: string;

  /**
   * Repository description.
   */
  description?: string;

  /**
   * Repository file system path.
   */
  path: string;

  /**
   * Initial branch name.
   */
  branch?: string;
}

/**
 * Request for updating repository configuration.
 */
export interface UpdateRepositoryRequest {
  /**
   * Updated repository name.
   */
  name?: string;

  /**
   * Updated repository description.
   */
  description?: string;
}

/**
 * Merge operation result.
 */
export interface MergeResult {
  /**
   * Merge success status.
   */
  success: boolean;

  /**
   * Merge commit hash.
   */
  commit?: string;

  /**
   * Conflict files (if any).
   */
  conflicts?: string[];

  /**
   * Merge error message (if failed).
   */
  error?: string;
}
```

**Rationale:** Type-safe repository API methods enable Git operations with proper error handling and conflict resolution [REQ-WEB-030].

### 4.3. WebSocket Client Methods

#### 4.3.1. WebSocketClient Interface

**TypeScript Interface:**
```typescript
/**
 * WebSocket client for real-time bidirectional communication.
 * 
 * @remarks
 * The WebSocketClient manages WebSocket connections with automatic
 * reconnection, message queuing, and type-safe message handling.
 * 
 * @example
 * ```typescript
 * const client = new WebSocketClient({
 *   url: 'wss://api.example.com/ws',
 *   authToken: createSignal<string | null>(null)
 * });
 * 
 * client.on('document_update', (message) => {
 *   console.log('Document updated:', message.payload);
 * });
 * 
 * await client.connect();
 * ```
 */
export class WebSocketClient {
  /**
   * Creates a new WebSocketClient instance.
   * 
   * @param config - Client configuration options
   * @throws {ConfigurationError} When configuration is invalid
   */
  constructor(config: WebSocketClientConfig);

  /**
   * Establishes WebSocket connection.
   * 
   * @returns Promise resolving when connection is established
   * @throws {ConnectionError} When connection fails
   */
  async connect(): Promise<void>;

  /**
   * Disconnects WebSocket connection.
   * 
   * @remarks
   * Gracefully closes connection and clears message queue.
   */
  disconnect(): void;

  /**
   * Registers a message handler for specific message type.
   * 
   * @template T - Expected payload type
   * @param messageType - Message type to handle
   * @param handler - Handler function for message
   * @returns Unsubscribe function
   */
  on<T>(messageType: string, handler: (message: WebSocketMessage<T>) => void): () => void;

  /**
   * Sends a message to server.
   * 
   * @template T - Message payload type
   * @param messageType - Message type
   * @param payload - Message payload
   * @throws {ConnectionError} When client is not connected
   */
  send<T>(messageType: string, payload: T): void;

  /**
   * Gets current connection state.
   * 
   * @returns Current connection state
   */
  getConnectionState(): ConnectionState;

  /**
   * Gets reactive signal for connection state.
   * 
   * @returns Signal containing connection state
   */
  getConnectionStateSignal(): Signal<ConnectionState>;
}

/**
 * Configuration options for WebSocketClient initialization.
 */
export interface WebSocketClientConfig {
  /**
   * WebSocket server URL.
   */
  url: string;

  /**
   * Reactive signal containing authentication token.
   * Token is automatically included in connection handshake.
   */
  authToken: Signal<string | null>;

  /**
   * Enable automatic reconnection with exponential backoff.
   * @default true
   */
  autoReconnect?: boolean;

  /**
   * Maximum reconnection attempts before giving up.
   * @default 10
   */
  maxReconnectAttempts?: number;

  /**
   * Initial reconnection delay in milliseconds.
   * @default 1000
   */
  initialReconnectDelay?: number;

  /**
   * Maximum reconnection delay in milliseconds.
   * @default 30000
   */
  maxReconnectDelay?: number;

  /**
   * Enable message queuing during disconnection.
   * @default true
   */
  queueMessages?: boolean;

  /**
   * Maximum queue size (messages dropped when exceeded).
   * @default 100
   */
  maxQueueSize?: number;

  /**
   * Heartbeat interval in milliseconds (0 to disable).
   * @default 30000
   */
  heartbeatInterval?: number;
}

/**
 * WebSocket connection state.
 */
export type ConnectionState =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'error';

/**
 * Generic WebSocket message structure.
 */
export interface WebSocketMessage<T = unknown> {
  /**
   * Message type identifier.
   */
  type: string;

  /**
   * Message payload data.
   */
  payload: T;

  /**
   * Message timestamp (ISO 8601).
   */
  timestamp: string;

  /**
   * Unique message identifier for request/response correlation.
   */
  messageId?: string;
}
```

**Rationale:** The WebSocketClient provides robust real-time communication with automatic reconnection and message queuing [REQ-WEB-041, REQ-WEB-042, REQ-WEB-045].

#### 4.3.2. Real-Time Message Types

**TypeScript Interfaces:**
```typescript
/**
 * Document update message payload.
 */
export interface DocumentUpdatePayload {
  /**
   * Document identifier.
   */
  documentId: string;

  /**
   * Update type (created, updated, deleted).
   */
  updateType: 'created' | 'updated' | 'deleted';

  /**
   * Updated document metadata.
   */
  metadata?: DocumentMetadata;

  /**
   * Document version number.
   */
  version?: number;
}

/**
 * User presence message payload.
 */
export interface UserPresencePayload {
  /**
   * User identifier.
   */
  userId: string;

  /**
   * User display name.
   */
  userName: string;

  /**
   * User presence status.
   */
  status: 'online' | 'away' | 'offline';

  /**
   * Currently viewed document ID (if any).
   */
  currentDocumentId?: string;

  /**
   * Cursor position in document (if editing).
   */
  cursorPosition?: {
    line: number;
    column: number;
  };
}

/**
 * Sync status message payload.
 */
export interface SyncStatusPayload {
  /**
   * Repository identifier.
   */
  repositoryId: string;

  /**
   * Sync status.
   */
  status: SyncStatus;

  /**
   * Sync progress (0-1).
   */
  progress?: number;

  /**
   * Sync error message (if failed).
   */
  error?: string;
}

/**
 * Error message payload.
 */
export interface ErrorPayload {
  /**
   * Error code.
   */
  code: string;

  /**
   * Error message.
   */
  message: string;

  /**
   * Error details.
   */
  details?: Record<string, unknown>;
}

/**
 * Ping message payload (for heartbeat).
 */
export interface PingPayload {
  /**
   * Ping timestamp.
   */
  timestamp: number;
}

/**
 * Pong message payload (heartbeat response).
 */
export interface PongPayload {
  /**
   * Original ping timestamp.
   */
  pingTimestamp: number;

  /**
   * Pong timestamp.
   */
  pongTimestamp: number;

  /**
   * Server latency in milliseconds.
   */
  latencyMs: number;
}
```

**Rationale:** Type-safe message payloads ensure correct parsing and handling of real-time events [REQ-WEB-043, REQ-WEB-046, REQ-WEB-047].

### 4.4. Error Handling

#### 4.4.1. Error Types

**TypeScript Interfaces:**
```typescript
/**
 * Base API error class.
 */
export class ApiError extends Error {
  /**
   * Error code.
   */
  readonly code: ErrorCode;

  /**
   * HTTP status code.
   */
  readonly statusCode: number;

  /**
   * Request identifier for tracing.
   */
  readonly requestId: string;

  /**
   * Additional error details.
   */
  readonly details?: Record<string, unknown>;

  constructor(
    code: ErrorCode,
    message: string,
    statusCode: number,
    requestId: string,
    details?: Record<string, unknown>
  );
}

/**
 * Validation error (400 Bad Request).
 */
export class ValidationError extends ApiError {
  /**
   * Validation errors by field.
   */
  readonly fieldErrors: Record<string, string[]>;

  constructor(
    message: string,
    requestId: string,
    fieldErrors: Record<string, string[]>
  );
}

/**
 * Authentication error (401 Unauthorized).
 */
export class AuthenticationError extends ApiError {
  constructor(message: string, requestId: string);
}

/**
 * Authorization error (403 Forbidden).
 */
export class AuthorizationError extends ApiError {
  constructor(message: string, requestId: string);
}

/**
 * Not found error (404 Not Found).
 */
export class NotFoundError extends ApiError {
  /**
   * Resource type that was not found.
   */
  readonly resourceType: string;

  /**
   * Resource identifier.
   */
  readonly resourceId: string;

  constructor(resourceType: string, resourceId: string, requestId: string);
}

/**
 * Conflict error (409 Conflict).
 */
export class ConflictError extends ApiError {
  /**
   * Conflict type (version, concurrent, etc.).
   */
  readonly conflictType: string;

  constructor(message: string, requestId: string, conflictType: string);
}

/**
 * Rate limit exceeded error (429 Too Many Requests).
 */
export class RateLimitExceededError extends ApiError {
  /**
   * Rate limit window in seconds.
   */
  readonly retryAfter: number;

  constructor(message: string, requestId: string, retryAfter: number);
}

/**
 * Internal server error (500 Internal Server Error).
 */
export class InternalError extends ApiError {
  constructor(message: string, requestId: string);
}

/**
 * Error code enumeration.
 */
export type ErrorCode =
  | 'VALIDATION_ERROR'
  | 'AUTHENTICATION_ERROR'
  | 'AUTHORIZATION_ERROR'
  | 'NOT_FOUND'
  | 'CONFLICT'
  | 'RATE_LIMIT_EXCEEDED'
  | 'INTERNAL_ERROR';
```

**Rationale:** Structured error types enable predictable error handling and user-friendly error messages [REQ-WEB-038].

#### 4.4.2. Error Handling Strategy

**TypeScript Implementation:**
```typescript
/**
 * Error handler utility for API errors.
 */
export class ApiErrorHandler {
  /**
   * Handles API error and displays appropriate user message.
   * 
   * @param error - Error to handle
   * @param options - Handling options
   */
  static handleError(error: unknown, options?: ErrorHandlerOptions): void;

  /**
   * Converts unknown error to ApiError.
   * 
   * @param error - Unknown error
   * @returns ApiError instance
   */
  static toApiError(error: unknown): ApiError;

  /**
   * Determines if error is retryable.
   * 
   * @param error - Error to check
   * @returns True if error is retryable
   */
  static isRetryable(error: ApiError): boolean;

  /**
   * Extracts user-friendly message from error.
   * 
   * @param error - Error to extract message from
   * @returns User-friendly error message
   */
  static getUserMessage(error: ApiError): string;
}

/**
 * Error handler options.
 */
export interface ErrorHandlerOptions {
  /**
   * Show error notification to user.
   * @default true
   */
  showNotification?: boolean;

  /**
   * Log error to console.
   * @default true
   */
  logError?: boolean;

  /**
   * Custom error message override.
   */
  customMessage?: string;

  /**
   * Error recovery callback.
   */
  onRecovery?: () => void;
}
```

**Rationale:** Centralized error handling ensures consistent error presentation and recovery across the application.

---

## 5. WEB COMPONENTS API

### 5.1. Component Communication

The Web Components API defines interfaces for communication between Leptos components, enabling type-safe data flow and event handling.

#### 5.1.1. Component Props Interface

**TypeScript Interface:**
```typescript
/**
 * Base props interface for all components.
 * 
 * @remarks
 * This interface provides common props that all components may accept,
 * including children, CSS classes, and accessibility attributes.
 */
export interface ComponentProps {
  /**
   * Child components or elements.
   */
  children?: Children;

  /**
   * CSS class names for styling.
   */
  className?: string;

  /**
   * Unique identifier for component instance.
   */
  id?: string;

  /**
   * ARIA role for accessibility.
   */
  role?: string;

  /**
   * Additional ARIA attributes.
   */
  aria?: Record<string, string | boolean>;
}

/**
 * Props for document-related components.
 */
export interface DocumentComponentProps extends ComponentProps {
  /**
   * Document metadata.
   */
  document: DocumentMetadata;

  /**
   * Document content (if loaded).
   */
  content?: string;

  /**
   * Loading state.
   */
  loading?: boolean;

  /**
   * Error state.
   */
  error?: ApiError | null;

  /**
   * Edit mode flag.
   */
  editMode?: boolean;

  /**
   * On edit callback.
   */
  onEdit?: (document: DocumentMetadata) => void;

  /**
   * On delete callback.
   */
  onDelete?: (documentId: string) => void;
}

/**
 * Props for document editor component.
 */
export interface DocumentEditorProps extends ComponentProps {
  /**
   * Document content.
   */
  content: Signal<string>;

  /**
   * Document metadata.
   */
  metadata: Signal<DocumentMetadata>;

  /**
   * Auto-save interval in milliseconds.
   */
  autoSaveInterval?: number;

  /**
   * On save callback.
   */
  onSave?: (content: string, metadata: DocumentMetadata) => Promise<void>;

  /**
   * On change callback.
   */
  onChange?: (content: string) => void;
}

/**
 * Props for search component.
 */
export interface SearchComponentProps extends ComponentProps {
  /**
   * Search query signal.
   */
  query: Signal<string>;

  /**
   * Search results signal.
   */
  results: Signal<SearchResults>;

  /**
   * Loading state.
   */
  loading?: boolean;

  /**
   * On search callback.
   */
  onSearch?: (query: string) => void;

  /**
   * On result select callback.
   */
  onSelectResult?: (result: SearchResultItem) => void;
}

/**
 * Props for navigation component.
 */
export interface NavigationComponentProps extends ComponentProps {
  /**
   * Repository structure.
   */
  repositories: Signal<Repository[]>;

  /**
   * Active repository signal.
   */
  activeRepository: Signal<Repository | null>;

  /**
   * Document structure for active repository.
   */
  documents: Signal<DocumentMetadata[]>;

  /**
   * Active document signal.
   */
  activeDocument: Signal<DocumentMetadata | null>;

  /**
   * On repository select callback.
   */
  onSelectRepository?: (repository: Repository) => void;

  /**
   * On document select callback.
   */
  onSelectDocument?: (document: DocumentMetadata) => void;

  /**
   * On create document callback.
   */
  onCreateDocument?: () => void;
}

/**
 * Props for notification component.
 */
export interface NotificationComponentProps extends ComponentProps {
  /**
   * Notifications signal.
   */
  notifications: Signal<Notification[]>;

  /**
   * On dismiss callback.
   */
  onDismiss?: (notification: Notification) => void;

  /**
   * On action callback.
   */
  onAction?: (notification: Notification, action: string) => void;
}

/**
 * Notification structure.
 */
export interface Notification {
  /**
   * Unique notification identifier.
   */
  id: string;

  /**
   * Notification type (info, warning, error, success).
   */
  type: 'info' | 'warning' | 'error' | 'success';

  /**
   * Notification title.
   */
  title: string;

  /**
   * Notification message.
   */
  message: string;

  /**
   * Notification timestamp.
   */
  timestamp: string;

  /**
   * Available actions.
   */
  actions?: NotificationAction[];

  /**
   * Auto-dismiss delay in milliseconds (0 for manual).
   */
  autoDismiss?: number;
}

/**
 * Notification action structure.
 */
export interface NotificationAction {
  /**
   * Action identifier.
   */
  id: string;

  /**
   * Action label.
   */
  label: string;

  /**
   * Action callback.
   */
  callback: () => void;

  /**
   * Primary action flag.
   */
  primary?: boolean;
}
```

**Rationale:** Type-safe component props enable compile-time correctness and prevent runtime errors [REQ-WEB-026, REQ-WEB-027].

### 5.2. State Synchronization

The Web Components API provides mechanisms for synchronizing state between components and with the server.

#### 5.2.1. State Store Interface

**TypeScript Interface:**
```typescript
/**
 * Global application state store.
 * 
 * @remarks
 * The AppState manages global application state using Leptos signals,
 * providing reactive state updates across all components.
 * 
 * @example
 * ```typescript
 * const appState = new AppState({
 *   session: createSignal<Session | null>(null),
 *   theme: createSignal<Theme>('light'),
 *   notifications: createSignal<Notification[]>([])
 * });
 * 
 * // In component
 * const session = useAppState(state => state.session);
 * ```
 */
export class AppState {
  /**
   * Creates a new AppState instance.
   * 
   * @param initialState - Initial state values
   */
  constructor(initialState: AppStateInit);

  /**
   * Creates a selector for accessing state slice.
   * 
   * @template T - State slice type
   * @param selector - Selector function
   * @returns Reactive signal containing selected state
   */
  createSelector<T>(
    selector: (state: AppState) => T
  ): Signal<T>;

  /**
   * Creates a derived state signal.
   * 
   * @template T - Derived state type
   * @param selector - Selector function
   * @param dependencies - Dependency signals
   * @returns Reactive signal containing derived state
   */
  createDerivedState<T>(
    selector: (state: AppState) => T,
    dependencies: Signal<unknown>[]
  ): Signal<T>;

  /**
   * Dispatches an action to update state.
   * 
   * @param action - Action to dispatch
   */
  dispatch(action: AppStateAction): void;

  /**
   * Gets current state value (non-reactive).
   * 
   * @returns Current state snapshot
   */
  getState(): AppStateSnapshot;
}

/**
 * Initial state for AppState.
 */
export interface AppStateInit {
  /**
   * User session data.
   */
  session?: Session | null;

  /**
   * Application theme.
   */
  theme?: Theme;

  /**
   * Layout mode.
   */
  layout?: LayoutMode;

  /**
   * Notifications.
   */
  notifications?: Notification[];

  /**
   * Loading overlay state.
   */
  loadingOverlay?: boolean;

  /**
   * Loading message.
   */
  loadingMessage?: string | null;
}

/**
 * User session data.
 */
export interface Session {
  /**
   * User identifier.
   */
  userId: string;

  /**
   * User display name.
   */
  userName: string;

  /**
   * User email.
   */
  email: string;

  /**
   * User roles.
   */
  roles: string[];

  /**
   * Authentication token.
   */
  token: string;

  /**
   * Session expiration timestamp.
   */
  expiresAt: string;
}

/**
 * Application theme.
 */
export type Theme =
  | 'light'
  | 'dark'
  | 'high-contrast';

/**
 * Layout mode.
 */
export type LayoutMode =
  | 'sidebar-left'
  | 'sidebar-right'
  | 'no-sidebar'
  | 'full-screen';

/**
 * State action for updating AppState.
 */
export type AppStateAction =
  | { type: 'SET_SESSION'; payload: Session | null }
  | { type: 'SET_THEME'; payload: Theme }
  | { type: 'SET_LAYOUT'; payload: LayoutMode }
  | { type: 'ADD_NOTIFICATION'; payload: Notification }
  | { type: 'REMOVE_NOTIFICATION'; payload: string }
  | { type: 'SET_LOADING_OVERLAY'; payload: boolean }
  | { type: 'SET_LOADING_MESSAGE'; payload: string | null };

/**
 * State snapshot (immutable).
 */
export interface AppStateSnapshot {
  /**
   * User session data.
   */
  session: Session | null;

  /**
   * Application theme.
   */
  theme: Theme;

  /**
   * Layout mode.
   */
  layout: LayoutMode;

  /**
   * Notifications.
   */
  notifications: Notification[];

  /**
   * Loading overlay state.
   */
  loadingOverlay: boolean;

  /**
   * Loading message.
   */
  loadingMessage: string | null;
}
```

**Rationale:** Centralized state store provides single source of truth and enables reactive updates across components [REQ-WEB-026, REQ-WEB-027, REQ-WEB-030].

#### 5.2.2. Document State Management

**TypeScript Interface:**
```typescript
/**
 * Document-specific state store.
 * 
 * @remarks
 * The DocumentState manages document-related state including metadata,
 * content, cache, and search results.
 */
export class DocumentState {
  /**
   * Creates a new DocumentState instance.
   * 
   * @param initialState - Initial state values
   */
  constructor(initialState: DocumentStateInit);

  /**
   * Gets documents signal.
   * 
   * @returns Reactive signal containing documents
   */
  getDocuments(): Signal<DocumentMetadata[]>;

  /**
   * Gets active document signal.
   * 
   * @returns Reactive signal containing active document
   */
  getActiveDocument(): Signal<DocumentContent | null>;

  /**
   * Gets document cache signal.
   * 
   * @returns Reactive signal containing document cache
   */
  getCache(): Signal<Map<string, DocumentContent>>;

  /**
   * Gets search results signal.
   * 
   * @returns Reactive signal containing search results
   */
  getSearchResults(): Signal<SearchResults>;

  /**
   * Gets search query signal.
   * 
   * @returns Reactive signal containing search query
   */
  getSearchQuery(): Signal<string>;

  /**
   * Sets active document.
   * 
   * @param document - Document to set as active
   */
  setActiveDocument(document: DocumentContent | null): void;

  /**
   * Updates document in cache.
   * 
   * @param documentId - Document identifier
   * @param document - Document content to cache
   */
  updateCache(documentId: string, document: DocumentContent): void;

  /**
   * Invalidates document cache.
   * 
   * @param documentId - Document identifier to invalidate
   */
  invalidateCache(documentId: string): void;

  /**
   * Sets search results.
   * 
   * @param results - Search results to set
   */
  setSearchResults(results: SearchResults): void;

  /**
   * Sets search query.
   * 
   * @param query - Search query to set
   */
  setSearchQuery(query: string): void;

  /**
   * Clears search results.
   */
  clearSearch(): void;
}

/**
 * Initial state for DocumentState.
 */
export interface DocumentStateInit {
  /**
   * Documents metadata array.
   */
  documents?: DocumentMetadata[];

  /**
   * Active document content.
   */
  activeDocument?: DocumentContent | null;

  /**
   * Document cache map.
   */
  cache?: Map<string, DocumentContent>;

  /**
   * Search results.
   */
  searchResults?: SearchResults;

  /**
   * Search query.
   */
  searchQuery?: string;

  /**
   * Loading state.
   */
  loading?: boolean;
}
```

**Rationale:** Document-specific state store provides efficient caching and reactive updates for document operations [REQ-WEB-026, REQ-WEB-034].

#### 5.2.3. Repository State Management

**TypeScript Interface:**
```typescript
/**
 * Repository-specific state store.
 * 
 * @remarks
 * The RepositoryState manages repository-related state including metadata,
 * sync status, and Git status.
 */
export class RepositoryState {
  /**
   * Creates a new RepositoryState instance.
   * 
   * @param initialState - Initial state values
   */
  constructor(initialState: RepositoryStateInit);

  /**
   * Gets repositories signal.
   * 
   * @returns Reactive signal containing repositories
   */
  getRepositories(): Signal<Repository[]>;

  /**
   * Gets active repository signal.
   * 
   * @returns Reactive signal containing active repository
   */
  getActiveRepository(): Signal<Repository | null>;

  /**
   * Gets sync status signal.
   * 
   * @returns Reactive signal containing sync status map
   */
  getSyncStatus(): Signal<Map<string, SyncStatus>>;

  /**
   * Gets Git status signal.
   * 
   * @returns Reactive signal containing Git status map
   */
  getGitStatus(): Signal<Map<string, GitStatus>>;

  /**
   * Sets active repository.
   * 
   * @param repository - Repository to set as active
   */
  setActiveRepository(repository: Repository | null): void;

  /**
   * Updates repository in list.
   * 
   * @param repository - Repository to update
   */
  updateRepository(repository: Repository): void;

  /**
   * Updates sync status.
   * 
   * @param repositoryId - Repository identifier
   * @param status - Sync status to set
   */
  updateSyncStatus(repositoryId: string, status: SyncStatus): void;

  /**
   * Updates Git status.
   * 
   * @param repositoryId - Repository identifier
   * @param status - Git status to set
   */
  updateGitStatus(repositoryId: string, status: GitStatus): void;

  /**
   * Clears sync status for repository.
   * 
   * @param repositoryId - Repository identifier
   */
  clearSyncStatus(repositoryId: string): void;
}

/**
 * Initial state for RepositoryState.
 */
export interface RepositoryStateInit {
  /**
   * Repositories array.
   */
  repositories?: Repository[];

  /**
   * Active repository.
   */
  activeRepository?: Repository | null;

  /**
   * Sync status map.
   */
  syncStatus?: Map<string, SyncStatus>;

  /**
   * Git status map.
   */
  gitStatus?: Map<string, GitStatus>;

  /**
   * Loading state.
   */
  loading?: boolean;
}
```

**Rationale:** Repository-specific state store provides efficient status tracking and reactive updates for repository operations [REQ-WEB-026, REQ-WEB-030].

### 5.3. Event Handling

The Web Components API defines event handling mechanisms for component communication and user interaction.

#### 5.3.1. Event Bus Interface

**TypeScript Interface:**
```typescript
/**
 * Event bus for component communication.
 * 
 * @remarks
 * The EventBus provides a publish-subscribe pattern for decoupled
 * component communication, enabling components to communicate without
 * direct references.
 * 
 * @example
 * ```typescript
 * const eventBus = new EventBus();
 * 
 * // Subscribe to event
 * const unsubscribe = eventBus.subscribe('document_updated', (data) => {
 *   console.log('Document updated:', data);
 * });
 * 
 * // Publish event
 * eventBus.publish('document_updated', { documentId: '123' });
 * 
 * // Unsubscribe when done
 * unsubscribe();
 * ```
 */
export class EventBus {
  /**
   * Creates a new EventBus instance.
   */
  constructor();

  /**
   * Subscribes to an event type.
   * 
   * @template T - Event payload type
   * @param eventType - Event type to subscribe to
   * @param handler - Handler function for event
   * @returns Unsubscribe function
   */
  subscribe<T>(
    eventType: string,
    handler: (payload: T) => void
  ): () => void;

  /**
   * Publishes an event.
   * 
   * @template T - Event payload type
   * @param eventType - Event type to publish
   * @param payload - Event payload data
   */
  publish<T>(eventType: string, payload: T): void;

  /**
   * Subscribes to an event type once.
   * 
   * @template T - Event payload type
   * @param eventType - Event type to subscribe to
   * @param handler - Handler function for event
   * @returns Unsubscribe function
   */
  subscribeOnce<T>(
    eventType: string,
    handler: (payload: T) => void
  ): () => void;

  /**
   * Clears all subscribers for an event type.
   * 
   * @param eventType - Event type to clear
   */
  clear(eventType: string): void;

  /**
   * Clears all subscribers.
   */
  clearAll(): void;
}

/**
 * Event type definitions.
 */
export type AppEventType =
  | 'document_created'
  | 'document_updated'
  | 'document_deleted'
  | 'repository_synced'
  | 'repository_conflict'
  | 'user_logged_in'
  | 'user_logged_out'
  | 'theme_changed'
  | 'notification_added'
  | 'notification_dismissed';

/**
 * Event payload types.
 */
export interface DocumentCreatedEvent {
  documentId: string;
  document: DocumentContent;
}

export interface DocumentUpdatedEvent {
  documentId: string;
  document: DocumentContent;
  changes: string[];
}

export interface DocumentDeletedEvent {
  documentId: string;
}

export interface RepositorySyncedEvent {
  repositoryId: string;
  status: SyncStatus;
}

export interface RepositoryConflictEvent {
  repositoryId: string;
  conflicts: string[];
}

export interface UserLoggedInEvent {
  userId: string;
  userName: string;
}

export interface UserLoggedOutEvent {
  userId: string;
}

export interface ThemeChangedEvent {
  theme: Theme;
}

export interface NotificationAddedEvent {
  notification: Notification;
}

export interface NotificationDismissedEvent {
  notificationId: string;
}
```

**Rationale:** Event bus provides decoupled component communication, reducing tight coupling between components [REQ-WEB-026].

#### 5.3.2. Component Lifecycle Hooks

**TypeScript Interface:**
```typescript
/**
 * Custom hook for component lifecycle management.
 * 
 * @remarks
 * The useComponentLifecycle hook provides lifecycle callbacks for
 * component mounting, updating, and unmounting, enabling
 * cleanup of resources and subscriptions.
 * 
 * @example
 * ```typescript
 * const { onMount, onUnmount } = useComponentLifecycle();
 * 
 * onMount(() => {
 *   console.log('Component mounted');
 *   // Set up subscriptions, timers, etc.
 * });
 * 
 * onUnmount(() => {
 *   console.log('Component unmounted');
 *   // Clean up subscriptions, timers, etc.
 * });
 * ```
 */
export function useComponentLifecycle(): ComponentLifecycleResult {
  /**
   * Callback invoked when component mounts.
   */
  onMount(callback: () => void): void;

  /**
   * Callback invoked when component updates.
   */
  onUpdate(callback: () => void): void;

  /**
   * Callback invoked when component unmounts.
   */
  onUnmount(callback: () => void): void;
}

/**
 * Result from useComponentLifecycle hook.
 */
export interface ComponentLifecycleResult {
  /**
   * Register mount callback.
   */
  onMount: (callback: () => void) => void;

  /**
   * Register update callback.
   */
  onUpdate: (callback: () => void) => void;

  /**
   * Register unmount callback.
   */
  onUnmount: (callback: () => void) => void;
}

/**
 * Custom hook for async operations.
 * 
 * @remarks
 * The useAsyncOperation hook provides loading state, error handling,
 * and retry logic for async operations.
 * 
 * @example
 * ```typescript
 * const { execute, loading, error } = useAsyncOperation();
 * 
 * const result = await execute(
 *   () => fetchDocument(documentId),
 *   { retries: 3, retryDelay: 1000 }
 * );
 * ```
 */
export function useAsyncOperation(): AsyncOperationResult {
  /**
   * Executes an async operation with loading state and error handling.
   * 
   * @template T - Result type
   * @param operation - Async operation to execute
   * @param options - Operation options
   * @returns Promise resolving to operation result
   */
  execute<T>(
    operation: () => Promise<T>,
    options?: AsyncOperationOptions
  ): Promise<T>;

  /**
   * Current loading state.
   */
  loading: Signal<boolean>;

  /**
   * Current error state.
   */
  error: Signal<ApiError | null>;

  /**
   * Resets error state.
   */
  resetError: () => void;
}

/**
 * Result from useAsyncOperation hook.
 */
export interface AsyncOperationResult {
  /**
   * Execute async operation.
   */
  execute: <T>(
    operation: () => Promise<T>,
    options?: AsyncOperationOptions
  ) => Promise<T>;

  /**
   * Loading state signal.
   */
  loading: Signal<boolean>;

  /**
   * Error state signal.
   */
  error: Signal<ApiError | null>;

  /**
   * Reset error state.
   */
  resetError: () => void;
}

/**
 * Options for async operations.
 */
export interface AsyncOperationOptions {
  /**
   * Number of retry attempts.
   * @default 0
   */
  retries?: number;

  /**
   * Retry delay in milliseconds.
   * @default 1000
   */
  retryDelay?: number;

  /**
   * Exponential backoff for retries.
   * @default false
   */
  exponentialBackoff?: boolean;

  /**
   * Error handler callback.
   */
  onError?: (error: ApiError) => void;

  /**
   * Success callback.
   */
  onSuccess?: (result: unknown) => void;
}
```

**Rationale:** Custom hooks provide reusable lifecycle and async operation logic across components [REQ-WEB-026, REQ-WEB-038].

---
## 6. API SECURITY

### 6.1. Authentication

The Web API implements comprehensive authentication mechanisms to ensure secure access to system resources.

#### 6.1.1. Authentication Methods

**Authentication Methods:**
1. **Token-Based Authentication:** JWT (JSON Web Tokens) for stateless authentication
2. **Multi-Factor Authentication (MFA):** Optional second factor for enhanced security
3. **Session-Based Authentication:** Server-side session management for WebSocket connections

**Token-Based Authentication:**
- JWT tokens issued upon successful login
- Tokens included in Authorization header: `Bearer <token>`
- Token expiration enforced server-side
- Automatic token refresh before expiration
- Secure token storage in memory (not localStorage for sensitive tokens)

**Multi-Factor Authentication:**
- TOTP (Time-based One-Time Password) support
- SMS-based verification option
- Backup codes for recovery
- MFA enforcement per user role configuration

**Session-Based Authentication:**
- Session tokens for WebSocket connections
- Session expiration and renewal
- Session invalidation on logout
- Secure session cookie attributes (HttpOnly, Secure, SameSite)

#### 6.1.2. Authentication API

**TypeScript Interface:**
```typescript
/**
 * Authentication API client.
 * 
 * @remarks
 * The AuthClient provides methods for user authentication,
 * token management, and session handling.
 * 
 * @example
 * ```typescript
 * const authClient = new AuthClient({
 *   apiClient: apiClient,
 *   mfaEnabled: true
 * });
 * 
 * // Login with credentials
 * const session = await authClient.login({
 *   email: 'user@example.com',
 *   password: 'password123'
 * });
 * 
 * // Login with MFA
 * const session = await authClient.loginWithMFA({
 *   email: 'user@example.com',
 *   password: 'password123',
 *   totpCode: '123456'
 * });
 * 
 * // Refresh token
 * const newToken = await authClient.refreshToken(session.token);
 * 
 * // Logout
 * await authClient.logout(session.token);
 * ```
 */
export class AuthClient {
  /**
   * Creates a new AuthClient instance.
   * 
   * @param config - Client configuration options
   * @throws {ConfigurationError} When configuration is invalid
   */
  constructor(config: AuthClientConfig);

  /**
   * Authenticates user with credentials.
   * 
   * @param credentials - User credentials
   * @returns Promise resolving to session data
   * @throws {AuthenticationError} When authentication fails
   * @throws {ValidationError} When credentials are invalid
   */
  async login(credentials: LoginCredentials): Promise<Session>;

  /**
   * Authenticates user with credentials and MFA.
   * 
   * @param credentials - User credentials
   * @param mfaCode - MFA verification code
   * @returns Promise resolving to session data
   * @throws {AuthenticationError} When authentication fails
   * @throws {ValidationError} When credentials or MFA are invalid
   */
  async loginWithMFA(
    credentials: LoginCredentials,
    mfaCode: string
  ): Promise<Session>;

  /**
   * Refreshes authentication token.
   * 
   * @param currentToken - Current authentication token
   * @returns Promise resolving to new session data
   * @throws {AuthenticationError} When refresh fails
   */
  async refreshToken(currentToken: string): Promise<Session>;

  /**
   * Logs out user and invalidates session.
   * 
   * @param token - Authentication token to invalidate
   * @returns Promise resolving when logout is complete
   * @throws {ApiError} When logout fails
   */
  async logout(token: string): Promise<void>;

  /**
   * Verifies current authentication token.
   * 
   * @param token - Authentication token to verify
   * @returns Promise resolving to verification result
   * @throws {ApiError} When verification fails
   */
  async verifyToken(token: string): Promise<TokenVerificationResult>;
}

/**
 * Authentication client configuration.
 */
export interface AuthClientConfig {
  /**
   * API client for HTTP requests.
   */
  apiClient: ApiClient;

  /**
   * Enable multi-factor authentication.
   * @default false
   */
  mfaEnabled?: boolean;

  /**
   * Token refresh threshold (seconds before expiration).
   * @default 300
   */
  tokenRefreshThreshold?: number;

  /**
   * Maximum token refresh attempts.
   * @default 3
   */
  maxRefreshAttempts?: number;
}

/**
 * Login credentials.
 */
export interface LoginCredentials {
  /**
   * User email address.
   */
  email: string;

  /**
   * User password.
   */
  password: string;

  /**
   * Remember me flag.
   */
  rememberMe?: boolean;
}

/**
 * Session data structure.
 */
export interface Session {
  /**
   * User identifier.
   */
  userId: string;

  /**
   * User display name.
   */
  userName: string;

  /**
   * User email address.
   */
  email: string;

  /**
   * User roles.
   */
  roles: string[];

  /**
   * Authentication token.
   */
  token: string;

  /**
   * Token expiration timestamp (ISO 8601).
   */
  expiresAt: string;

  /**
   * Refresh token (if available).
   */
  refreshToken?: string;
}

/**
 * Token verification result.
 */
export interface TokenVerificationResult {
  /**
   * Valid flag.
   */
  valid: boolean;

  /**
   * User identifier (if valid).
   */
  userId?: string;

  /**
   * User roles (if valid).
   */
  roles?: string[];

  /**
   * Expiration timestamp (if valid).
   */
  expiresAt?: string;
}
```

**Rationale:** Type-safe authentication API ensures secure credential handling and token management [REQ-WEB-018, REQ-WEB-030].

### 6.2. Authorization

The Web API implements Role-Based Access Control (RBAC) to enforce principle of least privilege.

#### 6.2.1. RBAC Model

**RBAC Principles:**
1. **Principle of Least Privilege:** Users granted minimum permissions required for tasks
2. **Role-Based Access:** Permissions grouped by role for easier management
3. **Permission Inheritance:** Roles inherit permissions from parent roles
4. **Resource-Based Access:** Permissions apply to specific resources (documents, repositories)
5. **Attribute-Based Access:** Fine-grained permissions based on resource attributes (ownership, tags)

**Role Definitions:**
| Role | Permissions | Description |
|------|-------------|-------------|
| **Administrator** | All permissions (full system access) |
| **Editor** | Create, read, update, delete documents |
| **Viewer** | Read documents only |
| **Reviewer** | Read and comment on documents |
| **Contributor** | Create and edit documents |

**Permission Definitions:**
| Permission | Resource | Action | Description |
|-----------|----------|--------|-------------|
| `document:create` | Documents | Create | Create new documents |
| `document:read` | Documents | Read | Read document content |
| `document:update` | Documents | Update | Update existing documents |
| `document:delete` | Documents | Delete | Delete documents |
| `document:comment` | Documents | Comment | Add comments to documents |
| `repository:create` | Repositories | Create | Create new repositories |
| `repository:read` | Repositories | Read | Read repository metadata |
| `repository:update` | Repositories | Update | Update repository configuration |
| `repository:delete` | Repositories | Delete | Delete repositories |
| `repository:sync` | Repositories | Sync | Synchronize repositories |
| `repository:branch` | Repositories | Branch | Create and manage branches |
| `repository:merge` | Repositories | Merge | Merge branches |
| `user:manage` | Users | Manage | Manage user accounts and roles |

#### 6.2.2. Authorization API

**TypeScript Interface:**
```typescript
/**
 * Authorization API client.
 * 
 * @remarks
 * The AuthzClient provides methods for checking user permissions
 * and enforcing access control.
 * 
 * @example
 * ```typescript
 * const authzClient = new AuthzClient({
 *   apiClient: apiClient
 * });
 * 
 * // Check if user can create documents
 * const canCreate = await authzClient.hasPermission(
 *   session.userId,
 *   'document:create'
 * );
 * 
 * // Check if user can update specific document
 * const canUpdate = await authzClient.canUpdateDocument(
 *   session.userId,
 *   documentId,
 *   session.userId // Owner check
 * );
 * ```
 */
export class AuthzClient {
  /**
   * Creates a new AuthzClient instance.
   * 
   * @param config - Client configuration options
   * @throws {ConfigurationError} When configuration is invalid
   */
  constructor(config: AuthzClientConfig);

  /**
   * Checks if user has specific permission.
   * 
   * @param userId - User identifier
   * @param permission - Permission to check
   * @returns Promise resolving to permission check result
   * @throws {ApiError} When permission check fails
   */
  async hasPermission(userId: string, permission: string): Promise<PermissionCheckResult>;

  /**
   * Checks if user can perform action on resource.
   * 
   * @param userId - User identifier
   * @param resourceType - Resource type (document, repository)
   * @param resourceId - Resource identifier
   * @param action - Action to check
   * @returns Promise resolving to permission check result
   * @throws {ApiError} When permission check fails
   */
  async canPerformAction(
    userId: string,
    resourceType: ResourceType,
    resourceId: string,
    action: string
  ): Promise<PermissionCheckResult>;

  /**
   * Gets user's permissions.
   * 
   * @param userId - User identifier
   * @returns Promise resolving to user permissions
   * @throws {ApiError} When permission retrieval fails
   */
  async getUserPermissions(userId: string): Promise<string[]>;

  /**
   * Gets user's roles.
   * 
   * @param userId - User identifier
   * @returns Promise resolving to user roles
   * @throws {ApiError} When role retrieval fails
   */
  async getUserRoles(userId: string): Promise<string[]>;
}

/**
 * Authorization client configuration.
 */
export interface AuthzClientConfig {
  /**
   * API client for HTTP requests.
   */
  apiClient: ApiClient;

  /**
   * Permission cache TTL in milliseconds.
   * @default 300000 (5 minutes)
   */
  permissionCacheTtl?: number;
}

/**
 * Permission check result.
 */
export interface PermissionCheckResult {
  /**
   * Permission granted flag.
   */
  allowed: boolean;

  /**
   * Reason for denial (if denied).
   */
  reason?: string;
}

/**
 * Resource type.
 */
export type ResourceType =
  | 'document'
  | 'repository'
  | 'user';

/**
 * User permissions structure.
 */
export interface UserPermissions {
  /**
   * User roles.
   */
  roles: string[];

  /**
   * Granted permissions.
   */
  permissions: string[];

  /**
   * Resource-specific permissions.
   */
  resourcePermissions: Record<string, string[]>;
}
```

**Rationale:** Type-safe authorization API ensures proper access control and principle of least privilege [REQ-WEB-032, REQ-WEB-033].

### 6.3. Input Validation

The Web API implements comprehensive input validation to prevent injection attacks and data corruption.

#### 6.3.1. Validation Rules

**Validation Principles:**
1. **Validate Early:** Validate inputs as early as possible (client-side)
2. **Whitelist Approach:** Allow only known safe values (reject unknown)
3. **Sanitization:** Remove or escape dangerous characters
4. **Length Constraints:** Enforce minimum and maximum lengths
5. **Format Validation:** Validate data formats (email, URLs, dates)
6. **Type Validation:** Ensure correct data types

**Client-Side Validation:**
- Email format validation (RFC 5322)
- Password strength validation (minimum 8 characters, complexity requirements)
- URL validation (HTTP/HTTPS only)
- Markdown sanitization (prevent XSS)
- File path validation (prevent directory traversal)
- Tag validation (alphanumeric, hyphens, underscores)

**Server-Side Validation:**
- Schema validation for all request bodies
- SQL injection prevention (parameterized queries)
- XSS prevention (content sanitization)
- CSRF protection (token validation)
- File size limits (prevent DoS)

#### 6.3.2. Validation API

**TypeScript Interface:**
```typescript
/**
 * Input validation utility.
 * 
 * @remarks
 * The InputValidator provides methods for validating user input
 * and sanitizing data to prevent security vulnerabilities.
 * 
 * @example
 * ```typescript
 * const validator = new InputValidator();
 * 
 * // Validate email
 * const emailValid = validator.isValidEmail('user@example.com');
 * 
 * // Validate password
 * const passwordValid = validator.isValidPassword('Password123!');
 * 
 * // Validate URL
 * const urlValid = validator.isValidUrl('https://example.com');
 * 
 * // Sanitize Markdown
 * const sanitized = validator.sanitizeMarkdown('<script>alert("xss")</script>');
 * ```
 */
export class InputValidator {
  /**
   * Creates a new InputValidator instance.
   */
  constructor();

  /**
   * Validates email address.
   * 
   * @param email - Email address to validate
   * @returns Validation result
   */
  isValidEmail(email: string): ValidationResult;

  /**
   * Validates password strength.
   * 
   * @param password - Password to validate
   * @returns Validation result
   */
  isValidPassword(password: string): ValidationResult;

  /**
   * Validates URL format.
   * 
   * @param url - URL to validate
   * @param allowHttp - Allow HTTP URLs (default: HTTPS only)
   * @returns Validation result
   */
  isValidUrl(url: string, allowHttp?: boolean): ValidationResult;

  /**
   * Validates file path for directory traversal.
   * 
   * @param path - File path to validate
   * @returns Validation result
   */
  isValidFilePath(path: string): ValidationResult;

  /**
   * Sanitizes Markdown content to prevent XSS.
   * 
   * @param markdown - Markdown content to sanitize
   * @returns Sanitized Markdown content
   */
  sanitizeMarkdown(markdown: string): string;

  /**
   * Validates tag format.
   * 
   * @param tag - Tag to validate
   * @returns Validation result
   */
  isValidTag(tag: string): ValidationResult;

  /**
   * Validates document title.
   * 
   * @param title - Title to validate
   * @returns Validation result
   */
  isValidDocumentTitle(title: string): ValidationResult;
}

/**
 * Validation result structure.
 */
export interface ValidationResult {
  /**
   * Valid flag.
   */
  valid: boolean;

  /**
   * Error message (if invalid).
   */
  error?: string;

  /**
   * Field-specific errors (for form validation).
   */
  fieldErrors?: Record<string, string[]>;
}

/**
 * Validation constraints.
 */
export interface ValidationConstraints {
  /**
   * Email validation constraints.
   */
  email: {
    /**
     * Maximum email length.
     */
    maxLength: 255;

    /**
     * Email regex pattern.
     */
    pattern: string;
  };

  /**
   * Password validation constraints.
   */
  password: {
    /**
     * Minimum password length.
     */
    minLength: 8;

    /**
     * Maximum password length.
     */
    maxLength: 128;

    /**
     * Require uppercase letter.
     */
    requireUppercase?: boolean;

    /**
     * Require lowercase letter.
     */
    requireLowercase?: boolean;

    /**
     * Require number.
     */
    requireNumber?: boolean;

    /**
     * Require special character.
     */
    requireSpecialChar?: boolean;
  };

  /**
   * Document title constraints.
   */
  documentTitle: {
    /**
     * Minimum title length.
     */
    minLength: 1;

    /**
     * Maximum title length.
     */
    maxLength: 200;
  };

  /**
   * Tag validation constraints.
   */
  tag: {
    /**
     * Maximum tag length.
     */
    maxLength: 50;

    /**
     * Allowed characters pattern.
     */
    pattern: string;
  };
}
```

**Rationale:** Comprehensive input validation prevents injection attacks and data corruption [TACHYON-TMA-V1.0].

### 6.4. Rate Limiting

The Web API implements rate limiting to prevent abuse and ensure fair resource allocation.

#### 6.4.1. Rate Limiting Strategy

**Rate Limiting Principles:**
1. **Per-User Limits:** Limit requests per user per time window
2. **Per-Endpoint Limits:** Limit requests per endpoint
3. **Tiered Limits:** Different limits for different user tiers
4. **Burst Protection:** Allow short bursts within limits
5. **Gradual Backoff:** Exponential backoff for rate limit violations
6. **Whitelist Exemption:** Whitelisted IPs exempt from rate limits

**Rate Limit Categories:**
| Category | Limit | Time Window | Description |
|----------|-------|-------------|-------------|
| **Authentication** | 5 requests/minute | Login and token refresh |
| **Read Operations** | 100 requests/minute | Document reads |
| **Write Operations** | 20 requests/minute | Document creates/updates |
| **Search** | 10 requests/minute | Search queries |
| **WebSocket** | 30 messages/minute | WebSocket messages |
| **File Upload** | 5 requests/minute | File uploads |

#### 6.4.2. Rate Limiting API

**TypeScript Interface:**
```typescript
/**
 * Rate limiting utility.
 * 
 * @remarks
 * The RateLimiter tracks request rates and enforces limits
 * to prevent abuse and ensure fair resource allocation.
 * 
 * @example
 * ```typescript
 * const rateLimiter = new RateLimiter({
 *   limits: RATE_LIMITS
 * });
 * 
 * // Check if request is allowed
 * const allowed = await rateLimiter.checkLimit('document_create', userId);
 * 
 * if (!allowed) {
 *   console.log('Rate limit exceeded, retry after:', allowed.retryAfter);
 * }
 * ```
 */
export class RateLimiter {
  /**
   * Creates a new RateLimiter instance.
   * 
   * @param limits - Rate limit configuration
   */
  constructor(limits: RateLimitConfig);

  /**
   * Checks if request is within rate limits.
   * 
   * @param category - Request category
   * @param userId - User identifier
   * @returns Promise resolving to rate limit check result
   */
  async checkLimit(category: string, userId?: string): Promise<RateLimitResult>;

  /**
   * Records request for rate limiting.
   * 
   * @param category - Request category
   * @param userId - User identifier
   */
  recordRequest(category: string, userId?: string): void;

  /**
   * Gets remaining request count for user.
   * 
   * @param category - Request category
   * @param userId - User identifier
   * @returns Remaining request count
   */
  getRemainingRequests(category: string, userId?: string): number;

  /**
   * Resets rate limit counters for user.
   * 
   * @param userId - User identifier
   */
  resetUser(userId: string): void;

  /**
   * Checks if IP is whitelisted.
   * 
   * @param ipAddress - IP address to check
   * @returns True if IP is whitelisted
   */
  isWhitelisted(ipAddress: string): boolean;
}

/**
 * Rate limit configuration.
 */
export interface RateLimitConfig {
  /**
   * Per-user rate limits.
   */
  perUser: Record<string, PerUserLimit>;

  /**
   * Per-endpoint rate limits.
   */
  perEndpoint: Record<string, PerEndpointLimit>;

  /**
   * Whitelisted IP addresses.
   */
  whitelistedIps?: string[];

  /**
   * Rate limit window in milliseconds.
   * @default 60000 (1 minute)
   */
  windowMs?: number;
}

/**
 * Per-user rate limit.
 */
export interface PerUserLimit {
  /**
   * Maximum requests per window.
   */
  maxRequests: number;

  /**
   * Window duration in milliseconds.
   */
  windowMs: number;
}

/**
 * Per-endpoint rate limit.
 */
export interface PerEndpointLimit {
  /**
   * Maximum requests per window.
   */
  maxRequests: number;

  /**
   * Window duration in milliseconds.
   */
  windowMs: number;
}

/**
 * Rate limit check result.
 */
export interface RateLimitResult {
  /**
   * Request allowed flag.
   */
  allowed: boolean;

  /**
   * Remaining requests in window.
   */
  remaining: number;

  /**
   * Time until limit resets (milliseconds).
   */
  resetAfter: number;

  /**
   * Suggested retry delay (milliseconds).
   */
  retryAfter?: number;
}

/**
 * Standard rate limit definitions.
 */
export const RATE_LIMITS: RateLimitConfig = {
  perUser: {
    authentication: { maxRequests: 5, windowMs: 60000 },
    read: { maxRequests: 100, windowMs: 60000 },
    write: { maxRequests: 20, windowMs: 60000 },
    search: { maxRequests: 10, windowMs: 60000 },
    websocket: { maxRequests: 30, windowMs: 60000 },
    fileUpload: { maxRequests: 5, windowMs: 60000 }
  },
  perEndpoint: {
    '/api/v1/documents': { maxRequests: 50, windowMs: 60000 },
    '/api/v1/repositories': { maxRequests: 20, windowMs: 60000 },
    '/api/v1/search': { maxRequests: 30, windowMs: 60000 }
  },
  whitelistedIps: []
};
```

**Rationale:** Rate limiting prevents abuse and ensures fair resource allocation [TACHYON-TMA-V1.0].

### 6.5. Security Headers

The Web API implements security headers for secure communication.

#### 6.5.1. Header Definitions

**Security Headers:**
| Header | Value | Description |
|--------|-------|-------------|
| **Authorization** | `Bearer <token>` | JWT token for authentication |
| **Content-Type** | `application/json` | JSON request/response format |
| **X-Request-ID** | `<uuid>` | Unique request identifier for tracing |
| **X-Client-Version** | `1.0.0` | Client version for compatibility |
| **X-Request-Timestamp** | `<iso8601>` | Request timestamp for replay protection |
| **X-Forwarded-For** | `<ip>` | Original client IP for proxy detection |
| **Strict-Transport-Security** | `max-age=0` | Prevent caching of sensitive data |

#### 6.5.2. Header Management API

**TypeScript Interface:**
```typescript
/**
 * Security header utility.
 * 
 * @remarks
 * The SecurityHeaders utility provides methods for generating
 * and managing security headers for API requests.
 * 
 * @example
 * ```typescript
 * const headers = SecurityHeaders.create({
 *   token: session.token,
 *   requestId: generateRequestId()
 * });
 * 
 * // Use with fetch
 * fetch('/api/v1/documents', {
 *   headers: headers.toHeaders()
 * });
 * ```
 */
export class SecurityHeaders {
  /**
   * Creates security headers for request.
   * 
   * @param options - Header generation options
   * @returns SecurityHeaders instance
   */
  static create(options: SecurityHeadersOptions): SecurityHeaders;

  /**
   * Generates authorization header.
   * 
   * @param token - Authentication token
   * @returns Authorization header value
   */
  static createAuthorizationHeader(token: string): string;

  /**
   * Generates request ID header.
   * 
   * @returns Request ID header value
   */
  static createRequestIdHeader(): string;

  /**
   * Generates timestamp header.
   * 
   * @returns Timestamp header value
   */
  static createTimestampHeader(): string;

  /**
   * Generates client version header.
   * 
   * @returns Client version header value
   */
  static createClientVersionHeader(): string;

  /**
   * Converts headers to fetch HeadersInit.
   * 
   * @returns Fetch HeadersInit object
   */
  toHeaders(): HeadersInit;
}

/**
 * Security headers options.
 */
export interface SecurityHeadersOptions {
  /**
   * Authentication token.
   */
  token?: string;

  /**
   * Custom request ID.
   */
  requestId?: string;

  /**
   * Client version.
   */
  clientVersion?: string;

  /**
   * Additional headers.
   */
  additionalHeaders?: Record<string, string>;
}

/**
 * Security headers instance.
 */
export class SecurityHeaders {
  /**
   * Authorization header value.
   */
  readonly authorization: string;

  /**
   * Request ID header value.
   */
  readonly requestId: string;

  /**
   * Timestamp header value.
   */
  readonly timestamp: string;

  /**
   * Client version header value.
   */
  readonly clientVersion: string;

  /**
   * Additional headers.
   */
  readonly additional: Record<string, string>;

  /**
   * Converts headers to fetch HeadersInit.
   */
  toHeaders(): HeadersInit;
}

/**
 * Fetch HeadersInit interface.
 */
export interface HeadersInit {
  /**
   * Authorization header.
   */
  Authorization?: string;

  /**
   * Content-Type header.
   */
  'Content-Type'?: string;

  /**
   * X-Request-ID header.
   */
  'X-Request-ID'?: string;

  /**
   * X-Client-Version header.
   */
  'X-Client-Version'?: string;

  /**
   * X-Request-Timestamp header.
   */
  'X-Request-Timestamp'?: string;

  /**
   * X-Forwarded-For header.
   */
  'X-Forwarded-For'?: string;

  /**
   * Strict-Transport-Security header.
   */
  'Strict-Transport-Security'?: string;
}

/**
 * Generates unique request ID.
 */
export function generateRequestId(): string {
  return crypto.randomUUID();
}
```

**Rationale:** Security headers ensure secure communication and enable request tracing [TACHYON-TMA-V1.0].

---

## 7. API PERFORMANCE

The Web API implements performance optimizations to ensure responsive user experience and efficient resource utilization.

#### 7.1. Latency Requirements

**Performance Targets:**
| Metric | Target | Description | Requirement |
|--------|--------|-------------|
| **First Contentful Paint** | < 1 second | Initial page render [REQ-WEB-066] |
| **Time to Interactive** | < 2 seconds | Interactive state [REQ-WEB-067] |
| **API Response Time** | < 100ms | P95th percentile [REQ-WEB-067] |
| **WebSocket Message Latency** | < 50ms | P95th percentile [REQ-WEB-067] |
| **Scroll Performance** | 60fps | Documents up to 100,000 words [REQ-WEB-068] |

#### 7.2. Caching Strategies

**Caching Levels:**
1. **Browser Cache:** Client-side caching of API responses
2. **Server Cache:** Server-side caching of frequently accessed data
3. **CDN Cache:** Content Delivery Network (CDN) caching for static assets
4. **Memory Cache:** In-memory caching of frequently accessed data
5. **Offline Cache:** Service Worker caching for offline access

**Cache Invalidation:**
- Time-based expiration (TTL)
- Version-based invalidation
- Manual invalidation trigger
- WebSocket push invalidation

**Caching Configuration:**
| Resource | Cache Type | TTL | Description |
|----------|----------|----------|-------------|
| **Document Metadata** | Browser Cache | 5 minutes | Document metadata |
| **Document Content** | Browser Cache | 10 minutes | Document content |
| **Search Results** | Memory Cache | 5 minutes | Search results |
| **User Permissions** | Memory Cache | 5 minutes | User permissions |
| **Repository Status** | Memory Cache | 2 minutes | Repository sync status |

#### 7.3. Optimization Techniques

**Optimization Techniques:**
1. **Request Deduplication:** Prevent duplicate concurrent requests for same resource
2. **Response Compression:** Gzip/Brotli compression for large responses
3. **Lazy Loading:** Code splitting for route-based lazy loading
4. **Tree Shaking:** Eliminate unused code from bundles
5. **Image Optimization:** WebP/WebP format with responsive images
6. **Debouncing:** Debounce rapid user input to prevent unnecessary re-renders

**Request Deduplication Strategy:**
- In-flight request tracking
- Request deduplication window (100ms)
- Automatic request cancellation on navigation changes
- Request hash-based deduplication

**Response Compression:**
- Compress responses > 10KB
- Use gzip for text, brotli for binary

**Lazy Loading:**
- Split routes into separate chunks
- Load routes on-demand
- Preload critical routes

**Tree Shaking:**
- Analyze bundle for unused exports
- Remove dead code paths
- Optimize import statements

**Image Optimization:**
- Responsive images with srcset
- Lazy load off-screen images
- Use WebP format for compression

#### 7.4. Performance API

**TypeScript Interface:**
```typescript
/**
 * Performance optimization utility.
 * 
 * @remarks
 * The PerformanceOptimizer provides methods for optimizing
 * API performance through caching, request deduplication,
 * and lazy loading.
 * 
 * @example
 * ```typescript
 * const optimizer = new PerformanceOptimizer({
 *   cacheConfig: CACHE_CONFIG
 * });
 * 
 * // Cached API call
 * const documents = await optimizer.cachedFetch(
 *   () => apiClient.get<Document[]>('/api/v1/documents')
 * );
 * 
 * // Deduplicated request
 * const result = await optimizer.deduplicatedFetch(
 *   () => apiClient.get<Document[]>('/api/v1/documents')
 * );
 * ```
 */
export class PerformanceOptimizer {
  /**
   * Creates a new PerformanceOptimizer instance.
   * 
   @param config - Optimization configuration
   */
  constructor(config: PerformanceOptimizerConfig);

  /**
   * Performs cached API call.
   * 
   * @template T - Response type
   * @param requestFn - Request function to execute
   * @param options - Request options
   * @returns Promise resolving to response
   */
  async cachedFetch<T>(
    requestFn: () => Promise<T>,
    options?: CachedRequestOptions
  ): Promise<T>;

  /**
   * Performs deduplicated API call.
   * 
   * @template T - Response type
   * @param requestFn - Request function to execute
   * @param options - Request options
   * @returns Promise resolving to response
   */
  async deduplicatedFetch<T>(
    requestFn: () => Promise<T>,
    options?: DeduplicatedRequestOptions
  ): Promise<T>;

  /**
   * Clears cache for specific resource.
   * 
   * @param cacheKey - Cache key to clear
   */
  clearCache(cacheKey: string): void;

  /**
   * Clears all caches.
   */
  clearAllCaches(): void;

  /**
   * Gets cache statistics.
   * 
   @returns Cache statistics
   */
  getCacheStats(): CacheStats;
}

/**
 * Cached request options.
 */
export interface CachedRequestOptions {
  /**
   * Cache key for deduplication.
   */
  cacheKey?: string;

  /**
   * Cache TTL in milliseconds.
   */
  ttl?: number;

  /**
   * Force cache refresh.
   */
  forceRefresh?: boolean;
}

/**
 * Deduplicated request options.
 */
export interface DeduplicatedRequestOptions {
  /**
   * Deduplication window in milliseconds.
   */
  deduplicationWindowMs?: number;

  /**
   * Request hash function.
   */
  hashFn?: (request: Request) => string;
}

/**
 * Cache statistics.
 */
export interface CacheStats {
  /**
   * Cache hit count.
   */
  hits: number;

  /**
   Cache miss count.
   */
  misses: number;

  /**
   Cache hit rate.
   */
  hitRate: number;
}

/**
 * Performance optimization configuration.
 */
export interface PerformanceOptimizerConfig {
  /**
   * Cache configuration.
   */
  cacheConfig: Record<string, CacheConfig>;

  /**
   * Deduplication configuration.
   */
  deduplicationConfig?: DeduplicationConfig;

  /**
   * Lazy loading configuration.
   */
  lazyLoading?: boolean;
}

/**
 * Cache configuration.
 */
export interface CacheConfig {
  /**
   * Cache TTL in milliseconds.
   */
  ttl: number;

  /**
   Maximum cache size.
   */
  maxSize?: number;
}

/**
 * Standard cache configurations.
 */
export const CACHE_CONFIG: Record<string, CacheConfig> = {
  'document_metadata': { ttl: 300000, maxSize: 1024 * 1024KB },
  'document_content': { ttl: 600000, maxSize: 1048576 }, // 10MB
  'search_results': { ttl: 300000, maxSize: 5120 }, // 5KB
  'user_permissions': { ttl: 300000, maxSize: 1024 }, // 1KB
  'repository_status': { ttl: 120000, maxSize: 512 } // 512B
};
```

**Rationale:** Performance optimizations ensure responsive user experience and efficient resource utilization [REQ-WEB-066, REQ-WEB-067, REQ-WEB-068].

---

## 8. API DOCUMENTATION

### 8.1. Usage Examples

This section provides practical examples of Web API usage patterns and best practices.

#### 8.1. Basic API Usage

**Example 1: Fetching Documents**

```typescript
// Initialize API client
const apiClient = new ApiClient({
  baseUrl: 'https://api.example.com',
  authToken: createSignal<string | null>(null)
});

// Fetch all documents
const documents = await apiClient.get<DocumentMetadata[]>('/api/v1/documents');

// Fetch single document
const document = await apiClient.get<DocumentContent>('/api/v1/documents/123');

// Create new document
const newDoc = await apiClient.post<DocumentContent>('/api/v1/documents', {
  title: 'New Document',
  content: '# Hello World'
});

// Update existing document
const updated = await apiClient.put<DocumentContent>('/api/v1/documents/123', {
  title: 'Updated Document',
  content: '# Updated Content'
});

// Delete document
await apiClient.delete<void>('/api/v1/documents/123');
```

**Example 2: Search with Pagination**

```typescript
// Initialize search
const query = 'search term';

// First page
const firstPage = await apiClient.get<PaginatedDocumentsResponse>('/api/v1/documents', {
  params: {
    limit: 20,
    search: query
  }
});

// Navigate to next page
if (firstPage.nextCursor) {
  const nextPage = await apiClient.get<PaginatedDocumentsResponse>('/api/v1/documents', {
    params: {
      limit: 20,
      cursor: firstPage.nextCursor
    }
  });
}
```

**Example 3: WebSocket Connection**

```typescript
// Initialize WebSocket client
const wsClient = new WebSocketClient({
  url: 'wss://api.example.com/ws',
  authToken: createSignal<string | null>(null)
});

// Subscribe to document updates
wsClient.on<DocumentUpdatePayload>('document_update', (message) => {
  console.log('Document updated:', message.payload);
  // Update local state
  documentState.updateCache(message.payload.documentId, message.payload.metadata);
});

// Subscribe to user presence
wsClient.on<UserPresencePayload>('user_presence', (message) => {
  console.log('User online:', message.payload.userName);
  // Update presence indicators
  userPresenceState.set(message.payload.userId, message.payload.status);
});

// Handle connection state
wsClient.getConnectionStateSignal().subscribe((state) => {
  console.log('Connection state:', state);
});
```

**Example 4: Error Handling**

```typescript
// Initialize error handler
const errorHandler = new ApiErrorHandler();

// Try API call with error handling
try {
  const document = await errorHandler.execute(
    () => apiClient.get<DocumentContent>('/api/v1/documents/123'),
    {
      showNotification: true,
      onRecovery: () => router.navigate('/documents')
    }
  );
} catch (error) {
  errorHandler.handleError(error);
}
```

#### 8.2. Best Practices

**Best Practice 1: Use Type Safety**

Always leverage TypeScript's type system to catch errors at compile time rather than runtime.

```typescript
// ❌ Bad: Using any type
function processData(data: any): void {
  console.log(data.title); // No type safety
}

// ✅ Good: Using generic type
function processData<T extends { title: string }>(data: T): void {
  console.log(data.title); // Compile-time type checking
}
```

**Best Practice 2: Leverage Reactive State**

Use Leptos signals for reactive state management instead of manual DOM manipulation.

```typescript
// ❌ Bad: Manual DOM updates
document.getElementById('title').textContent = newTitle;

// ✅ Good: Reactive updates
const [title, setTitle] = createSignal('Untitled');
setTitle(newTitle);
```

**Best Practice 3: Handle Loading States**

Always show loading indicators and disable interactions during async operations.

```typescript
const [loading, setLoading] = createSignal(false);

async function loadDocument(id: string): Promise<void> {
  setLoading(true);
  try {
    const doc = await apiClient.get<DocumentContent>(`/api/v1/documents/${id}`);
    documentState.setActiveDocument(doc);
  } finally {
    setLoading(false);
  }
}
```

**Best Practice 4: Optimize Re-Renders**

Use Leptos's fine-grained reactivity to minimize unnecessary re-renders.

```typescript
// ❌ Bad: Re-render entire list
documents.map(doc => <DocumentCard document={doc} />);

// ✅ Good: Only render changed items
<For each (doc, index) in documents>
  <DocumentCard key={doc.id} document={doc} />
```

**Best Practice 5: Secure Token Storage**

Never store authentication tokens in localStorage or cookies for sensitive applications.

```typescript
// ❌ Bad: localStorage
localStorage.setItem('auth_token', token);

// ✅ Good: Memory-only storage
const [authToken, setAuthToken] = createSignal<string | null>(null);
// Token automatically included in Authorization header
```

### 8.3. Code Examples

#### 8.3.1. Document State Management

```typescript
import { createSignal, createEffect, createMemo } from 'leptos';

// Document state store
const documentState = new DocumentState({
  documents: [],
  activeDocument: null,
  cache: new Map<string, DocumentContent>(),
  loading: false
});

// Load document with caching
async function loadDocument(id: string): Promise<void> {
  // Check cache first
  const cached = documentState.getCache().get(id);
  if (cached) {
    documentState.setActiveDocument(cached);
    return;
  }

  // Fetch from API
  documentState.setLoading(true);
  try {
    const doc = await apiClient.get<DocumentContent>(`/api/v1/documents/${id}`);
    documentState.updateCache(id, doc);
    documentState.setActiveDocument(doc);
  } finally {
    documentState.setLoading(false);
  }
}

// Update document with optimistic UI
async function updateDocument(id: string, updates: Partial<DocumentContent>): Promise<void> {
  // Update local state immediately
  documentState.updateCache(id, {
    ...updates,
    version: documentState.getActiveDocument()?.version + 1
  });
  
  // Then sync with server
  const updated = await apiClient.put<DocumentContent>(`/api/v1/documents/${id}`, updates);
  documentState.updateCache(id, updated);
}
```

#### 8.3.2. WebSocket Integration

```typescript
import { createSignal, createEffect } from 'leptos';

// WebSocket state
const [connectionState, setConnectionState] = createSignal<ConnectionState>('disconnected');

// Connect to WebSocket
wsClient.connect().then(() => {
  setConnectionState('connected');
}).catch(() => {
  setConnectionState('error');
});

// Subscribe to document updates
wsClient.on<DocumentUpdatePayload>('document_update', (message) => {
  const doc = message.payload;
  documentState.updateCache(doc.documentId, doc);
});

// Handle disconnection
createEffect(() => {
  const state = connectionState();
  if (state === 'disconnected') {
    // Show offline indicator
    showNotification('Connection lost');
  }
});

// Reconnect on visibility change
createEffect(() => {
  if (documentVisibilityState.get()) {
    wsClient.connect();
  }
});
```

#### 8.3.3. Event-Driven Updates

```typescript
import { createSignal } from 'leptos';

// Subscribe to document events
const [documents, setDocuments] = createSignal<DocumentMetadata[]>([]);

// Listen for document events
eventBus.subscribe<DocumentCreatedEvent>('document_created', (event) => {
  const docs = [...documents(), event.document];
  setDocuments(docs);
});

// Update local state on event
eventBus.subscribe<DocumentUpdatedEvent>('document_updated', (event) => {
  documentState.updateCache(event.documentId, event.document);
});
```

### 8.4. Testing Examples

#### 8.4.1. Unit Testing

```typescript
import { describe, it, expect, beforeEach, afterEach } from 'bun:test';

describe('DocumentApiClient', () => {
  let apiClient: DocumentApiClient;
  let mockFetch: jest.Mock;

  beforeEach(() => {
    apiClient = new DocumentApiClient({
      apiClient: {
        fetch: mockFetch
      }
    });
    mockFetch.mockResolvedValue({
      data: [{ id: '1', title: 'Test Document' }]
    });
  });

  it('should fetch documents', async () => {
    const docs = await apiClient.listDocuments();
    expect(docs).toHaveLength(1);
    expect(docs[0]).toEqual('Test Document');
  });

  it('should create document', async () => {
    const newDoc = await apiClient.createDocument({
      title: 'New Document',
      content: '# Test Content'
    });
    expect(newDoc).toBeDefined();
    expect(newDoc.id).toBeTruthy();
  });
});
```

#### 8.4.2. Integration Testing

```typescript
import { describe, it, expect, beforeAll, afterAll } from 'bun:test';

describe('Web API Integration', () => {
  let apiClient: ApiClient;
  let mockWsClient: jest.Mock<WebSocketClient>;

  beforeAll(() => {
    mockWsClient = jest.fn();
    mockWsClient.connect.mockResolvedValue(true);
  });

  it('should handle WebSocket messages', async () => {
    wsClient.on('document_update', (message) => {
      expect(message.payload).toBeDefined();
    });
  });
});
```

#### 8.4.3. Performance Testing

```typescript
import { describe, it, expect } from 'bun:test';

describe('Performance Optimization', () => {
  const optimizer: PerformanceOptimizer;

  it('should cache API responses', async () => {
    const uncached = await optimizer.cachedFetch(
      () => apiClient.get<Document[]>('/api/v1/documents')
    );
    const cached = await optimizer.cachedFetch(
      () => apiClient.get<Document[]>('/api/v1/documents')
    );
    
    // First call should be slower (cache miss)
    expect(uncached.executionTimeMs).toBeGreaterThan(cached.executionTimeMs);
    
    // Second call should be faster (cache hit)
    expect(cached.executionTimeMs).toBeLessThan(cached.executionTimeMs);
  });

  it('should deduplicate requests', async () => {
    const result1 = await optimizer.deduplicatedFetch(
      () => apiClient.get<Document[]>('/api/v1/documents')
    );
    const result2 = await optimizer.deduplicatedFetch(
      () => apiClient.get<Document[]>('/api/v1/documents')
    );
    
    // Deduplicated call should skip API call
    expect(result1.executionTimeMs).toBeLessThan(result2.executionTimeMs);
  });
});
```

---

## 9. REFERENCES

### 9.1. Document References

| Document ID | Title | Reference |
|-----------|--------|-------------|
| [TACHYON-REQ-WEB-V1.0](../../.specs/04_future_state/reqs/web_requirements.md) - Web Frontend Requirements |
| [TACHYON-DES-WD-V1.0](../../.specs/04_future_state/design/web_design.md) - Web Frontend Design |
| [TACHYON-ADR-004-V1.0](../../.specs/02_adrs/004_leptos_for_web_frontend.md) - ADR-004: Leptos for Web Frontend |
| [TACHYON-ADR-005-V1.0](../../.specs/02_adrs/005_bun_for_javascript_runtime.md) - ADR-005: Bun for JavaScript Runtime |
| [TACHYON-TMA-V1.0](../../.specs/03_threat_model/analysis.md) - Threat Model Analysis |

### 9.2. Standards References

| Document ID | Title | Reference |
|-----------|--------|-------------|
| [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards |
| [ISO/IEC 26514:2021](https://www.iso.org/standard/26514) - Systems and Software Engineering - Documentation |
| [IEEE 1063:2001](https://standards.ieee.org/standard/1063.html) - Standard for Software User Documentation |
| [ISO/IEC 829:2001](https://www.iso.org/standard/829.html) - Systems and Software Engineering - Lifecycle Processes |
| [ISO/IEC 12207:2017](https://www.iso.org/standard/12207.html) - Systems and Software Engineering - Lifecycle Processes |
| [ISO/IEC 25010:2011](https://www.iso.org/standard/25010.html) - Systems and Software Quality Requirements |
| [RFC 7540](https://datatracker.ietf.org/doc/html/rfc7540) - Hypertext Transfer Protocol (HTTP/1.1) |
| [RFC 6266](https://datatracker.ietf.org/doc/html/rfc6266) - Hypertext Transfer Protocol (HTTP/2) |
| [RFC 8441](https://datatracker.ietf.org/doc/html/rfc8441) - WebSocket Protocol |

### 9.3. External References

| Document ID | Title | Reference |
|-----------|--------|-------------|
| [Leptos Framework](https://leptos.dev/) - Leptos Framework Documentation |
| [Bun Runtime](https://bun.sh/) - Bun Runtime Documentation |
| [Axum Framework](https://docs.rs/axum/) - Axum Framework Documentation |
| [TypeScript](https://www.typescriptlang.org/) - TypeScript Language Specification |
| [WebAssembly](https://webassembly.github.io/) - WebAssembly Specification |
| [WebSocket API](https://websockets.spec.whatwg.org/) - WebSocket Protocol Specification |

### 9.4. Glossary Terms

| Term | Definition |
|------|----------|-------------|
| **API** | Application Programming Interface |
| **REST** | Representational State Transfer |
| **WebSocket** | Full-Duplex Communication Protocol |
| **JWT** | JSON Web Token |
| **RBAC** | Role-Based Access Control |
| **SSR** | Server-Side Rendering |
| **CSR** | Client-Side Rendering |
| **Hydration** | Progressive Enhancement |
| **WASM** | WebAssembly |
| **Signal** | Reactive Primitive |
| **Signal** | Reactive State Primitive |
| **Token** | Authentication Token |
| **Cache** | Temporary Storage |
| **TTL** | Time-To-Live |
| **TTL** | Time-To-Live |
| **MFA** | Multi-Factor Authentication |

---

**Document History:**

| Version | Date | Changes | Author | Description |
|--------|--------|------|----------|
| 1.0.0 | 2026-02-05 | Initial document creation | Initial version |
| | | | | Complete document with header, introduction, API design principles, versioning strategy |

---

**Document Approval Status:** Approved for Implementation

**Change Log:**

| Date | Version | Change Description | Author |
|------|----------|------|----------|

---

**Document Signature:**

This document has been reviewed and approved for implementation according to TACHYON-STD-V1.0 coding and documentation standards.

---
