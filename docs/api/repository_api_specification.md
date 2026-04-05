# TACHYON: REPOSITORY API SPECIFICATION

**Document ID:** TACHYON-API-014-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001
**Dependencies:** [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md), [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md), [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md), [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md)

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Repository API Design Principles](#2-repository-api-design-principles)
3. [Repository CRUD API](#3-repository-crud-api)
4. [Repository Sync API](#4-repository-sync-api)
5. [Repository Status API](#5-repository-status-api)
6. [Repository Branch API](#6-repository-branch-api)
7. [Repository Security](#7-repository-security)
8. [References](#8-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document specifies comprehensive Repository API for Tachyon toolchain, defining all endpoints, request/response schemas, error handling, and security considerations for repository management operations. The Repository API enables clients (desktop and web components) to perform CRUD operations on Git repositories, synchronize remote repositories, query repository status, and manage branches.

### 1.2. Scope

The Repository API encompasses the following functional areas:

- **Repository CRUD Operations:** Create, read, update, and delete repository configurations
- **Repository Synchronization:** Fetch, pull, and push operations for remote repositories
- **Repository Status:** Query current Git status, branch information, and commit history
- **Branch Management:** List, create, switch, and delete Git branches
- **Security:** Authentication and authorization for all repository operations

### 1.3. Target Audience

This specification is intended for:

- **Backend Developers:** Implementing repository API endpoints in Rust using Axum
- **Frontend Developers:** Consuming repository API from TypeScript/JavaScript clients
- **System Architects:** Understanding of repository API's role in the overall system architecture
- **QA Engineers:** Designing test cases for repository API endpoints

### 1.4. Document Conventions

This document follows conventions established in [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md):

- **Formal Tone:** Third-person perspective, objective language, precise terminology
- **PhD Thesis Rigor:** All statements are precise, unambiguous, and verifiable
- **ISO/IEEE Compliance:** Adheres to ISO/IEC 26514:2021 and IEEE 1063-2001 standards
- **Cross-References:** All references use relative paths and descriptive link text
- **Code Examples:** Rust code examples use proper documentation comments

### 1.5. System Context

The Repository API operates within the Tachyon server component, which provides centralized repository management for enterprise deployments. The API integrates with:

- **Git Integration Layer:** Uses the `git2` crate for Git repository operations [1]
- **Async Runtime:** Leverages Tokio v1 for asynchronous I/O operations [2]
- **HTTP/2 Server:** Implemented using Axum v0.7 framework [3]
- **Authentication:** Integrates with the system's authentication and RBAC mechanisms

The Repository API serves both desktop and web clients, enabling:

- **Desktop Mode:** Local-first repository operations with optional remote synchronization
- **Server Mode:** Centralized repository management with real-time collaboration features

---

## 2. REPOSITORY API DESIGN PRINCIPLES

### 2.1. RESTful Design

The Repository API adheres to REST architectural constraints:

- **Resource-Oriented:** Endpoints represent repository resources with clear nouns
- **HTTP Methods:** Appropriate use of GET (read), POST (create), PUT (update), DELETE (delete)
- **Stateless:** Each request contains all information necessary for processing
- **Uniform Interface:** Consistent endpoint structure and response formats
- **Cacheable:** Responses include cache control headers where appropriate

### 2.2. HTTP/2 Native

The API leverages HTTP/2 features for improved performance:

- **Multiplexing:** Multiple concurrent requests over single TCP connection
- **Header Compression:** HPACK compression reduces header overhead
- **Server Push:** Proactive resource pushing for related resources
- **Stream Prioritization:** Priority-based request processing

### 2.3. Type Safety

The API enforces type safety through Rust's type system:

- **Compile-Time Validation:** Request and response types are validated at compile time
- **Serde Serialization:** Automatic JSON serialization/deserialization with type checking
- **Path Parameter Extraction:** Type-safe URL path parameter extraction
- **Query Parameter Validation:** Type-safe query string parsing and validation

### 2.4. Asynchronous Processing

All repository operations use asynchronous I/O:

- **Non-Blocking:** Git operations do not block request threads
- **Concurrent Processing:** Multiple repository operations processed concurrently
- **Tokio Runtime:** Uses Tokio's work-stealing scheduler for optimal CPU utilization
- **Timeout Support:** Built-in timeout support for async operations

### 2.5. Error Handling

The API provides comprehensive error handling:

- **Structured Error Types:** Custom error types implementing `IntoResponse` trait
- **HTTP Status Codes:** Appropriate HTTP status codes for error conditions
- **Error Messages:** Clear, actionable error messages for clients
- **Error Logging:** Structured logging for debugging and monitoring

### 2.6. Security First

Security considerations are integrated throughout API design:

- **Authentication Required:** All endpoints require valid authentication tokens
- **Authorization Checks:** RBAC enforcement for repository access permissions
- **Path Validation:** Repository paths validated to prevent directory traversal attacks
- **Input Sanitization:** All user inputs are validated and sanitized
- **Audit Logging:** All repository operations logged for audit purposes

### 2.7. Performance Considerations

The API is designed for high performance:

- **Sub-15ms Response Times:** Fast response times for simple queries
- **Pagination Support:** Efficient handling of large result sets
- **Caching:** In-memory caching for frequently accessed repository metadata
- **Connection Pooling:** Reuse of HTTP/2 connections reduces overhead

### 2.8. Backward Compatibility

The API supports backward compatibility:

- **API Versioning:** Versioned endpoints (`/api/v1/`) for future evolution
- **Deprecation Policy:** Clear deprecation timeline for obsolete endpoints
- **Migration Guides:** Documentation for migrating between API versions

---

## 3. REPOSITORY CRUD API

### 3.1. List Repositories

#### API-014-001: GET /api/v1/repositories

**Element ID:** DES-API-014-001
**Name:** List Repositories
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** GET
**Endpoint:** `/api/v1/repositories`

**Description:** Retrieves a paginated list of repositories accessible to the authenticated user. This endpoint enables browsing and discovery of repositories with filtering and sorting capabilities.

**Request:**

```rust
use axum::extract::{Query, State};
use serde::Deserialize;

/// Query parameters for listing repositories
#[derive(Debug, Deserialize)]
pub struct ListRepositoriesQuery {
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,

    /// Page size (default: 20, max: 100)
    #[serde(default)]
    pub limit: Option<usize>,

    /// Sort field (name, path, created_at, updated_at)
    #[serde(default)]
    pub sort: Option<String>,

    /// Sort order (asc, desc)
    #[serde(default)]
    pub order: Option<String>,

    /// Filter by repository name (case-insensitive partial match)
    pub name: Option<String>,

    /// Filter by repository path (case-insensitive partial match)
    pub path: Option<String>,

    /// Filter by sync status (synced, unsynced, syncing, error)
    pub status: Option<String>,
}

/// Handler for listing repositories
///
/// # Parameters
///
/// * `params` - Query parameters for pagination, sorting, and filtering
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response containing paginated list of repositories
///
/// # Errors
///
/// * `400 Bad Request` - Invalid query parameters
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `500 Internal Server Error` - Server error during repository listing
pub async fn list_repositories(
    Query(params): Query<ListRepositoriesQuery>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<RepositoryListResponse>, ApiError>;
```

**Response:**

```rust
use serde::Serialize;
use chrono::{DateTime, Utc};

/// Response containing paginated list of repositories
#[derive(Debug, Serialize)]
pub struct RepositoryListResponse {
    /// List of repositories
    pub repositories: Vec<RepositorySummary>,

    /// Total count of repositories matching query
    pub total: usize,

    /// Current pagination offset
    pub offset: usize,

    /// Page size
    pub limit: usize,

    /// Has more results indicator
    pub has_more: bool,
}

/// Summary information for a repository
#[derive(Debug, Serialize)]
pub struct RepositorySummary {
    /// Unique repository identifier
    pub id: RepositoryId,

    /// Repository name (from directory name or configuration)
    pub name: String,

    /// Repository path (relative to configured root)
    pub path: String,

    /// Remote URL (if configured)
    pub remote_url: Option<String>,

    /// Current branch
    pub current_branch: String,

    /// Sync status
    pub sync_status: SyncStatus,

    /// Last sync timestamp
    pub last_sync_at: Option<DateTime<Utc>>,

    /// Total commits count
    pub commits_count: usize,

    /// Uncommitted changes indicator
    pub has_uncommitted_changes: bool,

    /// Repository creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}

/// Repository synchronization status
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    /// Repository is synchronized with remote
    Synced,

    /// Repository has local changes not pushed
    Unsynced,

    /// Synchronization in progress
    Syncing,

    /// Last sync failed
    Error(String),

    /// No remote configured
    NoRemote,
}

/// Unique repository identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryId(pub String);
```

**Constraints:**

- `limit`: Must be between 1 and 100 inclusive, defaults to 20
- `offset`: Must be non-negative, defaults to 0
- `sort`: Must be one of `name`, `path`, `created_at`, `updated_at`, defaults to `name`
- `order`: Must be `asc` or `desc`, defaults to `asc`
- `name`: Case-insensitive partial match, max 255 characters
- `path`: Case-insensitive partial match, max 1024 characters
- `status`: Must be one of `synced`, `unsynced`, `syncing`, `error`

**Dependencies:**

- REQ-SRV-025: Repository List
- REQ-SRV-081: RBAC Enforcement
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository

**Rationale:** Paginated list enables efficient browsing of large repository collections. Filtering and sorting capabilities support various use cases including finding specific repositories and organizing by different criteria.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository access permissions
- Only returns repositories accessible to the authenticated user
- Validates repository paths to prevent directory traversal attacks
- Restricts repository creation to configured root directories
- Validates remote URLs to prevent SSRF attacks
- Sanitizes repository names and paths in response
- Filters out repositories marked as internal or restricted
- Logs repository listing operations for audit purposes

**Performance Considerations:**

- Uses in-memory caching for repository metadata
- Pagination prevents returning large result sets
- Database queries optimized with proper indexing
- Response time target: < 50ms for typical queries

**Example Request:**

```http
GET /api/v1/repositories?limit=20&offset=0&sort=updated_at&order=desc&status=synced
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Host: api.tachyon.example.com
```

**Example Response:**

```json
{
  "repositories": [
    {
      "id": "repo-123e4567-e89b-12d3-a456-426614174000",
      "name": "documentation",
      "path": "/home/user/docs/documentation",
      "remote_url": "https://github.com/user/documentation.git",
      "current_branch": "main",
      "sync_status": "synced",
      "last_sync_at": "2026-02-05T23:30:00Z",
      "commits_count": 1523,
      "has_uncommitted_changes": false,
      "created_at": "2025-01-15T10:00:00Z",
      "updated_at": "2026-02-05T23:30:00Z"
    },
    {
      "id": "repo-223e4567-e89b-12d3-a456-426614174001",
      "name": "project-notes",
      "path": "/home/user/docs/project-notes",
      "remote_url": null,
      "current_branch": "develop",
      "sync_status": "no_remote",
      "last_sync_at": null,
      "commits_count": 89,
      "has_uncommitted_changes": true,
      "created_at": "2025-03-20T14:30:00Z",
      "updated_at": "2026-02-05T22:15:00Z"
    }
  ],
  "total": 2,
  "offset": 0,
  "limit": 20,
  "has_more": false
}
```

**Error Responses:**

**400 Bad Request - Invalid Query Parameters:**

```json
{
  "error": {
    "code": "invalid_query_parameters",
    "message": "Invalid query parameters",
    "details": {
      "limit": "must be between 1 and 100",
      "sort": "must be one of: name, path, created_at, updated_at"
    }
  }
}
```

**401 Unauthorized - Missing Authentication:**

```json
{
  "error": {
    "code": "unauthorized",
    "message": "Authentication required"
  }
}
```

---

### 3.2. Get Repository

#### API-014-002: GET /api/v1/repositories/:id

**Element ID:** DES-API-014-002
**Name:** Get Repository
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** GET
**Endpoint:** `/api/v1/repositories/:id`

**Description:** Retrieves detailed information for a specific repository by its unique identifier. This endpoint provides comprehensive repository metadata including configuration, status, and recent activity.

**Request:**

```rust
use axum::extract::{Path, State};

/// Handler for retrieving a specific repository
///
/// # Parameters
///
/// * `id` - Repository unique identifier (UUID v4)
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response containing detailed repository information
///
/// # Errors
///
/// * `400 Bad Request` - Invalid repository ID format
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User lacks permission to access repository
/// * `404 Not Found` - Repository does not exist
/// * `500 Internal Server Error` - Server error during repository retrieval
pub async fn get_repository(
    Path(id): Path<RepositoryId>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<RepositoryResponse>, ApiError>;
```

**Response:**

```rust
use serde::Serialize;
use chrono::{DateTime, Utc};

/// Detailed repository information
#[derive(Debug, Serialize)]
pub struct RepositoryResponse {
    /// Repository identifier
    pub id: RepositoryId,

    /// Repository name
    pub name: String,

    /// Repository path (relative to configured root)
    pub path: String,

    /// Repository description (optional)
    pub description: Option<String>,

    /// Remote URL (if configured)
    pub remote_url: Option<String>,

    /// Remote name (e.g., "origin")
    pub remote_name: Option<String>,

    /// Current branch
    pub current_branch: String,

    /// List of all branches
    pub branches: Vec<BranchSummary>,

    /// Sync status
    pub sync_status: SyncStatus,

    /// Last sync timestamp
    pub last_sync_at: Option<DateTime<Utc>>,

    /// Last sync error (if applicable)
    pub last_sync_error: Option<String>,

    /// Git status
    pub git_status: GitStatus,

    /// Repository statistics
    pub statistics: RepositoryStatistics,

    /// Repository configuration
    pub configuration: RepositoryConfiguration,

    /// Repository creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,

    /// User access level
    pub access_level: AccessLevel,
}

/// Summary information for a branch
#[derive(Debug, Serialize)]
pub struct BranchSummary {
    /// Branch name
    pub name: String,

    /// Branch is current indicator
    pub is_current: bool,

    /// Branch is remote indicator
    pub is_remote: bool,

    /// Last commit SHA
    pub last_commit_sha: String,

    /// Last commit message
    pub last_commit_message: String,

    /// Last commit author
    pub last_commit_author: String,

    /// Last commit timestamp
    pub last_commit_at: DateTime<Utc>,

    /// Commits ahead of remote
    pub commits_ahead: usize,

    /// Commits behind remote
    pub commits_behind: usize,
}

/// Git repository status
#[derive(Debug, Serialize)]
pub struct GitStatus {
    /// Uncommitted changes indicator
    pub has_changes: bool,

    /// Modified files
    pub modified: Vec<String>,

    /// Added files
    pub added: Vec<String>,

    /// Deleted files
    pub deleted: Vec<String>,

    /// Renamed files (old_path -> new_path)
    pub renamed: Vec<RenamedFile>,

    /// Untracked files
    pub untracked: Vec<String>,

    /// Staged changes indicator
    pub has_staged: bool,
}

/// Renamed file mapping
#[derive(Debug, Serialize)]
pub struct RenamedFile {
    /// Old path
    pub old_path: String,

    /// New path
    pub new_path: String,
}

/// Repository statistics
#[derive(Debug, Serialize)]
pub struct RepositoryStatistics {
    /// Total commits count
    pub total_commits: usize,

    /// Total branches count
    pub total_branches: usize,

    /// Total tags count
    pub total_tags: usize,

    /// Total contributors count
    pub total_contributors: usize,

    /// Total file count
    pub total_files: usize,

    /// Repository size in bytes
    pub size_bytes: u64,

    /// Average commits per day (last 30 days)
    pub commits_per_day: f64,
}

/// Repository configuration
#[derive(Debug, Serialize)]
pub struct RepositoryConfiguration {
    /// Auto-sync enabled indicator
    pub auto_sync_enabled: bool,

    /// Auto-sync interval in seconds
    pub auto_sync_interval: Option<u64>,

    /// Default branch name
    pub default_branch: String,

    /// Repository is private indicator
    pub is_private: bool,

    /// Repository is archived indicator
    pub is_archived: bool,

    /// Repository is template indicator
    pub is_template: bool,
}

/// User access level for repository
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    /// Read-only access
    Read,

    /// Read and write access
    Write,

    /// Administrative access
    Admin,

    /// Owner access
    Owner,
}
```

**Constraints:**

- `id`: Must be valid UUID v4 format
- Repository must exist and be accessible to the authenticated user
- Response includes all repository metadata, status, and configuration

**Dependencies:**

- REQ-SRV-026: Repository Retrieval
- REQ-SRV-081: RBAC Enforcement
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository
- DES-DM-008: GitStatus

**Rationale:** Detailed repository information enables comprehensive repository management, including viewing status, configuration, and recent activity. This endpoint is the primary interface for repository details display.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository access permissions
- Validates repository ID format to prevent injection attacks
- Only returns repositories accessible to the authenticated user
- Filters sensitive configuration data based on access level
- Sanitizes file paths in Git status to prevent information disclosure
- Filters sensitive author information (emails) based on access level

**Performance Considerations:**

- Uses in-memory caching for repository metadata
- Git status queries are optimized for performance
- Response time target: < 100ms for typical repositories

**Example Request:**

```http
GET /api/v1/repositories/repo-123e4567-e89b-12d3-a456-426614174000
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Host: api.tachyon.example.com
```

**Example Response:**

```json
{
  "id": "repo-123e4567-e89b-12d3-a456-426614174000",
  "name": "documentation",
  "path": "/home/user/docs/documentation",
  "description": "Main documentation repository",
  "remote_url": "https://github.com/user/documentation.git",
  "remote_name": "origin",
  "current_branch": "main",
  "branches": [
    {
      "name": "main",
      "is_current": true,
      "is_remote": true,
      "last_commit_sha": "a1b2c3d4e5f6789012345678901234567890123",
      "last_commit_message": "Update API documentation",
      "last_commit_author": "John Doe",
      "last_commit_at": "2026-02-05T23:30:00Z",
      "commits_ahead": 0,
      "commits_behind": 0
    },
    {
      "name": "develop",
      "is_current": false,
      "is_remote": true,
      "last_commit_sha": "b2c3d4e5f67890123456789012345678901234",
      "last_commit_message": "Add feature branch",
      "last_commit_author": "Jane Smith",
      "last_commit_at": "2026-02-04T15:20:00Z",
      "commits_ahead": 2,
      "commits_behind": 0
    }
  ],
  "sync_status": "synced",
  "last_sync_at": "2026-02-05T23:30:00Z",
  "last_sync_error": null,
  "git_status": {
    "has_changes": false,
    "modified": [],
    "added": [],
    "deleted": [],
    "renamed": [],
    "untracked": [],
    "has_staged": false
  },
  "statistics": {
    "total_commits": 1523,
    "total_branches": 3,
    "total_tags": 45,
    "total_contributors": 12,
    "total_files": 847,
    "size_bytes": 52428800,
    "commits_per_day": 3.2
  },
  "configuration": {
    "auto_sync_enabled": true,
    "auto_sync_interval": 300,
    "default_branch": "main",
    "is_private": false,
    "is_archived": false,
    "is_template": false
  },
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2026-02-05T23:30:00Z",
  "access_level": "admin"
}
```

**Error Responses:**

**400 Bad Request - Invalid Repository ID:**

```json
{
  "error": {
    "code": "invalid_repository_id",
    "message": "Invalid repository ID format",
    "details": {
      "id": "must be a valid UUID v4"
    }
  }
}
```

**403 Forbidden - Access Denied:**

```json
{
  "error": {
    "code": "access_denied",
    "message": "User does not have permission to access this repository"
  }
}
```

**404 Not Found - Repository Not Found:**

```json
{
  "error": {
    "code": "repository_not_found",
    "message": "Repository not found"
  }
}
```

---

### 3.3. Create Repository

#### API-014-003: POST /api/v1/repositories

**Element ID:** DES-API-014-003
**Name:** Create Repository
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** POST
**Endpoint:** `/api/v1/repositories`

**Description:** Creates a new repository configuration and initializes a Git repository at the specified path. This endpoint supports both creating new repositories and registering existing Git repositories.

**Request:**

```rust
use axum::{Json, State};
use serde::Deserialize;

/// Request for creating a new repository
#[derive(Debug, Deserialize)]
pub struct CreateRepositoryRequest {
    /// Repository name (required, max 255 characters)
    pub name: String,

    /// Repository path (required, max 1024 characters)
    pub path: String,

    /// Repository description (optional, max 1000 characters)
    pub description: Option<String>,

    /// Remote URL for cloning (optional)
    pub remote_url: Option<String>,

    /// Remote name (default: "origin")
    #[serde(default = "default_remote_name")]
    pub remote_name: String,

    /// Initialize as bare repository (default: false)
    #[serde(default)]
    pub bare: bool,

    /// Initialize with initial commit (default: true)
    #[serde(default = "default_initial_commit")]
    pub initial_commit: bool,

    /// Initial commit message (if initial_commit is true)
    pub initial_commit_message: Option<String>,

    /// Default branch name (default: "main")
    #[serde(default = "default_branch_name")]
    pub default_branch: String,

    /// Auto-sync enabled (default: false)
    #[serde(default)]
    pub auto_sync_enabled: bool,

    /// Auto-sync interval in seconds (default: 300)
    #[serde(default = "default_auto_sync_interval")]
    pub auto_sync_interval: u64,

    /// Repository is private (default: false)
    #[serde(default)]
    pub is_private: bool,

    /// Repository is template (default: false)
    #[serde(default)]
    pub is_template: bool,
}

/// Handler for creating a new repository
///
/// # Parameters
///
/// * `req` - Repository creation request
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response containing created repository information
///
/// # Errors
///
/// * `400 Bad Request` - Invalid request parameters
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User lacks permission to create repository
/// * `409 Conflict` - Repository already exists at specified path
/// * `500 Internal Server Error` - Server error during repository creation
pub async fn create_repository(
    Json(req): Json<CreateRepositoryRequest>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<RepositoryResponse>, ApiError>;

fn default_remote_name() -> String {
    "origin".to_string()
}

fn default_initial_commit() -> bool {
    true
}

fn default_branch_name() -> String {
    "main".to_string()
}

fn default_auto_sync_interval() -> u64 {
    300
}
```

**Response:**

```rust
// Returns RepositoryResponse (see API-014-002)
```

**Constraints:**

- `name`: Non-empty, max 255 characters, must be unique within user's repositories
- `path`: Valid file system path, max 1024 characters, must be within configured root
- `description`: Max 1000 characters
- `remote_url`: Valid URL if provided, supports HTTPS, SSH, and Git protocols
- `remote_name`: Max 255 characters, alphanumeric and hyphens only
- `default_branch`: Max 255 characters, valid Git branch name
- `auto_sync_interval`: Must be between 60 and 86400 seconds (1 minute to 24 hours)
- Repository path must not already exist unless registering existing repository
- At least one field must be provided in the request

**Dependencies:**

- REQ-SRV-027: Repository Creation
- REQ-SRV-047: Commit Management
- REQ-DESK-037: Repository Cloning
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository

**Rationale:** Repository creation enables users to set up new documentation repositories or register existing Git repositories with the Tachyon system. The endpoint supports both initialization and registration workflows.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository creation permissions
- Validates repository path to prevent directory traversal attacks
- Restricts repository creation to configured root directories
- Validates remote URLs to prevent SSRF attacks
- Sanitizes repository names and descriptions
- Creates repository with user as owner
- Logs repository creation for audit purposes
- Prevents modification of immutable repository properties (e.g., creation timestamp)

**Performance Considerations:**

- Git initialization is performed asynchronously
- Repository cloning from remote URLs is performed in background
- Response time target: < 500ms for local initialization, < 2s for remote clone initiation

**Example Request:**

```http
POST /api/v1/repositories
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Host: api.tachyon.example.com

{
  "name": "new-documentation",
  "path": "/home/user/docs/new-documentation",
  "description": "New documentation repository",
  "remote_url": "https://github.com/user/new-documentation.git",
  "remote_name": "origin",
  "bare": false,
  "initial_commit": true,
  "initial_commit_message": "Initial commit",
  "default_branch": "main",
  "auto_sync_enabled": true,
  "auto_sync_interval": 300,
  "is_private": false,
  "is_template": false
}
```

**Example Response:**

```json
{
  "id": "repo-323e4567-e89b-12d3-a456-426614174002",
  "name": "new-documentation",
  "path": "/home/user/docs/new-documentation",
  "description": "New documentation repository",
  "remote_url": "https://github.com/user/new-documentation.git",
  "remote_name": "origin",
  "current_branch": "main",
  "branches": [
    {
      "name": "main",
      "is_current": true,
      "is_remote": true,
      "last_commit_sha": "c3d4e5f6789012345678901234567890123",
      "last_commit_message": "Initial commit",
      "last_commit_author": "John Doe",
      "last_commit_at": "2026-02-06T00:00:00Z",
      "commits_ahead": 0,
      "commits_behind": 0
    }
  ],
  "sync_status": "synced",
  "last_sync_at": "2026-02-06T00:00:00Z",
  "last_sync_error": null,
  "git_status": {
    "has_changes": false,
    "modified": [],
    "added": [],
    "deleted": [],
    "renamed": [],
    "untracked": [],
    "has_staged": false
  },
  "statistics": {
    "total_commits": 1,
    "total_branches": 1,
    "total_tags": 0,
    "total_contributors": 1,
    "total_files": 0,
    "size_bytes": 0,
    "commits_per_day": 0.0
  },
  "configuration": {
    "auto_sync_enabled": true,
    "auto_sync_interval": 300,
    "default_branch": "main",
    "is_private": false,
    "is_archived": false,
    "is_template": false
  },
  "created_at": "2026-02-06T00:00:00Z",
  "updated_at": "2026-02-06T00:00:00Z",
  "access_level": "owner"
}
```

**Error Responses:**

**400 Bad Request - Invalid Request Parameters:**

```json
{
  "error": {
    "code": "invalid_request_parameters",
    "message": "Invalid request parameters",
    "details": {
      "name": "must be between 1 and 255 characters",
      "path": "must be within configured root directory"
    }
  }
}
```

**409 Conflict - Repository Already Exists:**

```json
{
  "error": {
    "code": "repository_already_exists",
    "message": "Repository already exists at specified path",
    "details": {
      "path": "/home/user/docs/new-documentation"
    }
  }
}
```

---

### 3.4. Update Repository

#### API-014-004: PUT /api/v1/repositories/:id

**Element ID:** DES-API-014-004
**Name:** Update Repository
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** PUT
**Endpoint:** `/api/v1/repositories/:id`

**Description:** Updates an existing repository's configuration. This endpoint supports partial updates, allowing modification of individual repository properties without requiring a full update.

**Request:**

```rust
use axum::{Json, Path, State};
use serde::Deserialize;

/// Request for updating a repository
#[derive(Debug, Deserialize)]
pub struct UpdateRepositoryRequest {
    /// New repository name (optional, max 255 characters)
    pub name: Option<String>,

    /// New repository description (optional, max 1000 characters)
    pub description: Option<String>,

    /// New remote URL (optional)
    pub remote_url: Option<String>,

    /// New remote name (optional, max 255 characters)
    pub remote_name: Option<String>,

    /// New default branch name (optional, max 255 characters)
    pub default_branch: Option<String>,

    /// Auto-sync enabled (optional)
    pub auto_sync_enabled: Option<bool>,

    /// Auto-sync interval in seconds (optional, 60-86400)
    pub auto_sync_interval: Option<u64>,

    /// Repository is private (optional)
    pub is_private: Option<bool>,

    /// Repository is archived (optional)
    pub is_archived: Option<bool>,

    /// Repository is template (optional)
    pub is_template: Option<bool>,
}

/// Handler for updating a repository
///
/// # Parameters
///
/// * `id` - Repository unique identifier (UUID v4)
/// * `req` - Repository update request
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response containing updated repository information
///
/// # Errors
///
/// * `400 Bad Request` - Invalid request parameters
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User lacks permission to update repository
/// * `404 Not Found` - Repository does not exist
/// * `500 Internal Server Error` - Server error during repository update
pub async fn update_repository(
    Path(id): Path<RepositoryId>,
    Json(req): Json<UpdateRepositoryRequest>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<RepositoryResponse>, ApiError>;
```

**Response:**

```rust
// Returns RepositoryResponse (see API-014-002)
```

**Constraints:**

- `id`: Must be valid UUID v4 format
- `name`: If provided, must be non-empty, max 255 characters, must be unique within user's repositories
- `description`: If provided, max 1000 characters
- `remote_url`: If provided, must be valid URL (HTTPS, SSH, or Git protocol)
- `remote_name`: If provided, max 255 characters, alphanumeric and hyphens only
- `default_branch`: If provided, max 255 characters, must be valid Git branch name
- `auto_sync_interval`: If provided, must be between 60 and 86400 seconds
- Repository must exist and be accessible to the authenticated user
- At least one field must be provided in the request
- Partial updates are supported (only provided fields are modified)

**Dependencies:**

- REQ-SRV-028: Repository Update
- REQ-SRV-047: Commit Management
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository

**Rationale:** Repository update enables modification of repository configuration without requiring full replacement. Partial updates allow efficient modification of individual properties.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository update permissions
- Validates repository ID format to prevent injection attacks
- Only allows updates to repositories where user has write or admin access
- Validates remote URLs to prevent SSRF attacks
- Sanitizes repository names and descriptions
- Logs repository updates for audit purposes
- Prevents modification of immutable repository properties (e.g., creation timestamp)

**Performance Considerations:**

- Updates are performed asynchronously for Git operations
- Configuration updates are performed synchronously
- Response time target: < 200ms for configuration-only updates

**Example Request:**

```http
PUT /api/v1/repositories/repo-123e4567-e89b-12d3-a456-426614174000
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Host: api.tachyon.example.com

{
  "name": "updated-documentation",
  "description": "Updated documentation repository",
  "auto_sync_enabled": true,
  "auto_sync_interval": 600
}
```

**Example Response:**

```json
{
  "id": "repo-123e4567-e89b-12d3-a456-426614174000",
  "name": "updated-documentation",
  "path": "/home/user/docs/documentation",
  "description": "Updated documentation repository",
  "remote_url": "https://github.com/user/documentation.git",
  "remote_name": "origin",
  "current_branch": "main",
  "branches": [
    {
      "name": "main",
      "is_current": true,
      "is_remote": true,
      "last_commit_sha": "a1b2c3d4e5f6789012345678901234567890123",
      "last_commit_message": "Update API documentation",
      "last_commit_author": "John Doe",
      "last_commit_at": "2026-02-05T23:55:00Z",
      "commits_ahead": 0,
      "commits_behind": 0
    }
  ],
  "sync_status": "synced",
  "last_sync_at": "2026-02-05T23:55:00Z",
  "last_sync_error": null,
  "git_status": {
    "has_changes": false,
    "modified": [],
    "added": [],
    "deleted": [],
    "renamed": [],
    "untracked": [],
    "has_staged": false
  },
  "statistics": {
    "total_commits": 1524,
    "total_branches": 3,
    "total_tags": 45,
    "total_contributors": 12,
    "total_files": 847,
    "size_bytes": 52428800,
    "commits_per_day": 3.2
  },
  "configuration": {
    "auto_sync_enabled": true,
    "auto_sync_interval": 600,
    "default_branch": "main",
    "is_private": false,
    "is_archived": false,
    "is_template": false
  },
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2026-02-05T23:55:00Z",
  "access_level": "admin"
}
```

**Error Responses:**

**400 Bad Request - No Fields Provided:**

```json
{
  "error": {
    "code": "no_update_fields",
    "message": "At least one field must be provided for update"
  }
}
```

**403 Forbidden - Access Denied:**

```json
{
  "error": {
    "code": "access_denied",
    "message": "User does not have permission to update this repository"
  }
}
```

---

### 3.5. Delete Repository

#### API-014-005: DELETE /api/v1/repositories/:id

**Element ID:** DES-API-014-005
**Name:** Delete Repository
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** DELETE
**Endpoint:** `/api/v1/repositories/:id`

**Description:** Deletes a repository from the system. This operation removes the repository configuration and optionally deletes the underlying Git repository and files.

**Request:**

```rust
use axum::{Json, Path, Query, State};
use serde::Deserialize;

/// Query parameters for repository deletion
#[derive(Debug, Deserialize)]
pub struct DeleteRepositoryQuery {
    /// Delete underlying Git repository and files (default: false)
    #[serde(default)]
    pub delete_files: bool,

    /// Force deletion without confirmation (default: false)
    #[serde(default)]
    pub force: bool,
}

/// Handler for deleting a repository
///
/// # Parameters
///
/// * `id` - Repository unique identifier (UUID v4)
/// * `params` - Query parameters for deletion options
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response confirming deletion
///
/// # Errors
///
/// * `400 Bad Request` - Invalid request parameters
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User lacks permission to delete repository
/// * `404 Not Found` - Repository does not exist
/// * `409 Conflict` - Repository has uncommitted changes or active operations
/// * `500 Internal Server Error` - Server error during repository deletion
pub async fn delete_repository(
    Path(id): Path<RepositoryId>,
    Query(params): Query<DeleteRepositoryQuery>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<DeleteRepositoryResponse>, ApiError>;
```

**Response:**

```rust
use serde::Serialize;

/// Response confirming repository deletion
#[derive(Debug, Serialize)]
pub struct DeleteRepositoryResponse {
    /// Deleted repository ID
    pub id: RepositoryId,

    /// Deletion status
    pub status: DeletionStatus,

    /// Files deleted indicator
    pub files_deleted: bool,

    /// Number of files deleted
    pub files_count: Option<usize>,

    /// Deletion timestamp
    pub deleted_at: DateTime<Utc>,
}

/// Repository deletion status
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeletionStatus {
    /// Repository configuration deleted only
    ConfigurationOnly,

    /// Repository and files deleted
    Complete,

    /// Repository marked for deletion (async deletion in progress)
    Pending,
}
```

**Constraints:**

- `id`: Must be valid UUID v4 format
- Repository must exist and be accessible to the authenticated user
- `delete_files`: If true, deletes underlying Git repository and files
- `force`: If true, bypasses confirmation and safety checks
- Repository must not have active operations (syncing, cloning) unless forced
- Repository with uncommitted changes requires confirmation unless forced

**Dependencies:**

- REQ-SRV-029: Repository Deletion
- REQ-SRV-081: RBAC Enforcement
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository

**Rationale:** Repository deletion enables removal of repositories that are no longer needed. The endpoint supports both configuration-only deletion and complete deletion including files.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository deletion permissions
- Validates repository ID format to prevent injection attacks
- Only allows deletion of repositories where user has owner or admin access
- Requires confirmation for repositories with uncommitted changes unless forced
- Logs repository deletions for audit purposes
- Prevents deletion of system repositories or protected repositories
- Validates file deletion paths to prevent directory traversal attacks

**Performance Considerations:**

- Configuration-only deletion is performed synchronously
- File deletion is performed asynchronously for large repositories
- Response time target: < 100ms for configuration-only deletion

**Example Request:**

```http
DELETE /api/v1/repositories/repo-123e4567-e89b-12d3-a456-426614174000?delete_files=true&force=false
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Host: api.tachyon.example.com
```

**Example Response:**

```json
{
  "id": "repo-123e4567-e89b-12d3-a456-426614174000",
  "status": "complete",
  "files_deleted": true,
  "files_count": 847,
  "deleted_at": "2026-02-06T00:00:00Z"
}
```

**Error Responses:**

**409 Conflict - Repository Has Active Operations:**

```json
{
  "error": {
    "code": "repository_has_active_operations",
    "message": "Repository has active operations and cannot be deleted",
    "details": {
      "active_operations": ["syncing"]
    }
  }
}
```

**409 Conflict - Repository Has Uncommitted Changes:**

```json
{
  "error": {
    "code": "repository_has_uncommitted_changes",
    "message": "Repository has uncommitted changes",
    "details": {
      "suggestion": "Commit changes or use force=true parameter"
    }
  }
}
```

---

## 4. REPOSITORY SYNC API

### 4.1. Sync Repository

#### API-014-006: POST /api/v1/repositories/:id/sync

**Element ID:** DES-API-014-006
**Name:** Sync Repository
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** POST
**Endpoint:** `/api/v1/repositories/:id/sync`

**Description:** Synchronizes a repository with its remote. This endpoint supports fetch, pull, and push operations, enabling bidirectional synchronization between local and remote repositories.

**Request:**

```rust
use axum::{Json, Path, State};
use serde::Deserialize;

/// Request for synchronizing a repository
#[derive(Debug, Deserialize)]
pub struct SyncRepositoryRequest {
    /// Sync operation type (fetch, pull, push, full)
    pub operation: SyncOperation,

    /// Remote name (default: "origin")
    #[serde(default = "default_remote_name")]
    pub remote: String,

    /// Branch name (default: current branch)
    pub branch: Option<String>,

    /// Force push (default: false)
    #[serde(default)]
    pub force: bool,

    /// Merge strategy (merge, rebase, fast-forward)
    pub merge_strategy: Option<MergeStrategy>,
}

/// Sync operation type
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncOperation {
    /// Fetch from remote without merging
    Fetch,

    /// Pull from remote and merge
    Pull,

    /// Push local changes to remote
    Push,

    /// Full sync (pull then push)
    Full,
}

/// Merge strategy
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    /// Standard merge
    Merge,

    /// Rebase local changes
    Rebase,

    /// Fast-forward only
    FastForward,
}

/// Handler for synchronizing a repository
///
/// # Parameters
///
/// * `id` - Repository unique identifier (UUID v4)
/// * `req` - Sync request
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response containing sync operation result
///
/// # Errors
///
/// * `400 Bad Request` - Invalid request parameters
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User lacks permission to sync repository
/// * `404 Not Found` - Repository does not exist
/// * `409 Conflict` - Merge conflicts or other sync issues
/// * `500 Internal Server Error` - Server error during sync
pub async fn sync_repository(
    Path(id): Path<RepositoryId>,
    Json(req): Json<SyncRepositoryRequest>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<SyncRepositoryResponse>, ApiError>;

fn default_remote_name() -> String {
    "origin".to_string()
}
```

**Response:**

```rust
use serde::Serialize;

/// Response containing sync operation result
#[derive(Debug, Serialize)]
pub struct SyncRepositoryResponse {
    /// Repository ID
    pub id: RepositoryId,

    /// Sync operation performed
    pub operation: SyncOperation,

    /// Sync status
    pub status: SyncStatus,

    /// Commits fetched
    pub commits_fetched: usize,

    /// Commits pushed
    pub commits_pushed: usize,

    /// Merge conflicts (if any)
    pub conflicts: Vec<MergeConflict>,

    /// Sync started timestamp
    pub started_at: DateTime<Utc>,

    /// Sync completed timestamp
    pub completed_at: Option<DateTime<Utc>>,

    /// Sync duration in seconds
    pub duration_seconds: Option<f64>,
}

/// Merge conflict information
#[derive(Debug, Serialize)]
pub struct MergeConflict {
    /// File with conflict
    pub file_path: String,

    /// Conflict type
    pub conflict_type: ConflictType,

    /// Conflict description
    pub description: String,
}

/// Conflict type
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictType {
    /// Both sides modified
    BothModified,

    /// Both sides added
    BothAdded,

    /// Both sides deleted
    BothDeleted,

    /// Local modified, remote deleted
    LocalModifiedRemoteDeleted,

    /// Local deleted, remote modified
    LocalDeletedRemoteModified,
}
```

**Constraints:**

- `id`: Must be valid UUID v4 format
- `operation`: Must be one of `fetch`, `pull`, `push`, `full`
- `remote`: Max 255 characters, must be valid remote name
- `branch`: If provided, must be valid Git branch name
- `merge_strategy`: Must be one of `merge`, `rebase`, `fast_forward`
- Repository must exist and be accessible to the authenticated user
- Repository must have a configured remote for sync operations

**Dependencies:**

- REQ-SRV-030: Repository Synchronization
- REQ-SRV-047: Commit Management
- REQ-DESK-037: Repository Cloning
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository

**Rationale:** Repository synchronization enables keeping local and remote repositories in sync. The endpoint supports various sync operations to accommodate different workflows.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository sync permissions
- Validates repository ID format to prevent injection attacks
- Only allows sync operations on repositories where user has write access
- Validates remote URLs to prevent SSRF attacks
- Logs sync operations for audit purposes
- Rate limits sync operations to prevent abuse
- Validates branch names to prevent injection attacks

**Performance Considerations:**

- Sync operations are performed asynchronously
- Long-running sync operations return immediately with status
- Response time target: < 200ms for sync initiation
- Background processing for large sync operations

**Example Request:**

```http
POST /api/v1/repositories/repo-123e4567-e89b-12d3-a456-426614174000/sync
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
Host: api.tachyon.example.com

{
  "operation": "pull",
  "remote": "origin",
  "branch": "main",
  "force": false,
  "merge_strategy": "merge"
}
```

**Example Response:**

```json
{
  "id": "repo-123e4567-e89b-12d3-a456-426614174000",
  "operation": "pull",
  "status": "synced",
  "commits_fetched": 5,
  "commits_pushed": 0,
  "conflicts": [],
  "started_at": "2026-02-06T00:00:00Z",
  "completed_at": "2026-02-06T00:00:02Z",
  "duration_seconds": 2.3
}
```

**Error Responses:**

**409 Conflict - Merge Conflicts:**

```json
{
  "error": {
    "code": "merge_conflicts",
    "message": "Sync operation resulted in merge conflicts",
    "details": {
      "conflicts": [
        {
          "file_path": "docs/api.md",
          "conflict_type": "both_modified",
          "description": "Both local and remote modified the same section"
        }
      ]
    }
  }
}
```

**409 Conflict - Remote Not Found:**

```json
{
  "error": {
    "code": "remote_not_found",
    "message": "Specified remote not found",
    "details": {
      "remote": "origin"
    }
  }
}
```

---

## 5. REPOSITORY STATUS API

### 5.1. Get Repository Status

#### API-014-007: GET /api/v1/repositories/:id/status

**Element ID:** DES-API-014-007
**Name:** Get Repository Status
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** GET
**Endpoint:** `/api/v1/repositories/:id/status`

**Description:** Retrieves current Git status of a repository, including uncommitted changes, branch information, and commit history. This endpoint provides real-time status information for repository monitoring and synchronization.

**Request:**

```rust
use axum::{Json, Path, State};

/// Handler for retrieving repository status
///
/// # Parameters
///
/// * `id` - Repository unique identifier (UUID v4)
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response containing repository status
///
/// # Errors
///
/// * `400 Bad Request` - Invalid repository ID format
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User lacks permission to access repository
/// * `404 Not Found` - Repository does not exist
/// * `500 Internal Server Error` - Server error during status retrieval
pub async fn get_repository_status(
    Path(id): Path<RepositoryId>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<RepositoryStatusResponse>, ApiError>;
```

**Response:**

```rust
use serde::Serialize;

/// Response containing repository status
#[derive(Debug, Serialize)]
pub struct RepositoryStatusResponse {
    /// Repository ID
    pub id: RepositoryId,

    /// Repository name
    pub name: String,

    /// Current branch
    pub current_branch: String,

    /// Git status
    pub git_status: GitStatus,

    /// Recent commits
    pub recent_commits: Vec<CommitInfo>,

    /// Branch status
    pub branch_status: BranchStatus,

    /// Status retrieved timestamp
    pub retrieved_at: DateTime<Utc>,
}

/// Commit information
#[derive(Debug, Serialize)]
pub struct CommitInfo {
    /// Commit SHA
    pub sha: String,

    /// Commit message
    pub message: String,

    /// Commit author
    pub author: String,

    /// Commit author email (sanitized)
    pub author_email: Option<String>,

    /// Commit timestamp
    pub timestamp: DateTime<Utc>,

    /// Commit is merge indicator
    pub is_merge: bool,

    /// Changed files count
    pub files_changed: usize,

    /// Insertions count
    pub insertions: usize,

    /// Deletions count
    pub deletions: usize,
}

/// Branch status
#[derive(Debug, Serialize)]
pub struct BranchStatus {
    /// Current branch name
    pub current: String,

    /// Local branches count
    pub local_branches: usize,

    /// Remote branches count
    pub remote_branches: usize,

    /// Commits ahead of remote
    pub commits_ahead: usize,

    /// Commits behind remote
    pub commits_behind: usize,

    /// Default branch name
    pub default_branch: String,

    /// On default branch indicator
    pub on_default_branch: bool,
}
```

**Constraints:**

- `id`: Must be valid UUID v4 format
- Repository must exist and be accessible to authenticated user
- `recent_commits`: Limited to most recent 20 commits

**Dependencies:**

- REQ-SRV-031: Repository Status
- REQ-SRV-047: Commit Management
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository
- DES-DM-008: GitStatus

**Rationale:** Repository status enables monitoring of repository state, including uncommitted changes, branch information, and recent commit history. This endpoint is essential for synchronization workflows and repository health monitoring.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository access permissions
- Validates repository ID format to prevent injection attacks
- Only returns status for repositories accessible to authenticated user
- Sanitizes author email addresses to prevent information disclosure
- Filters sensitive file paths in Git status
- Logs status queries for audit purposes

**Performance Considerations:**

- Uses in-memory caching for repository status
- Git status queries are optimized for performance
- Response time target: < 100ms for typical repositories

**Example Request:**

```http
GET /api/v1/repositories/repo-123e4567-e89b-12d3-a456-426614174000/status
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Host: api.tachyon.example.com
```

**Example Response:**

```json
{
  "id": "repo-123e4567-e89b-12d3-a456-426614174000",
  "name": "documentation",
  "current_branch": "main",
  "git_status": {
    "has_changes": true,
    "modified": ["docs/api.md"],
    "added": [],
    "deleted": [],
    "renamed": [],
    "untracked": ["temp.txt"],
    "has_staged": false
  },
  "recent_commits": [
    {
      "sha": "a1b2c3d4e5f6789012345678901234567890123",
      "message": "Update API documentation",
      "author": "John Doe",
      "author_email": "j***@example.com",
      "timestamp": "2026-02-05T23:30:00Z",
      "is_merge": false,
      "files_changed": 3,
      "insertions": 45,
      "deletions": 12
    }
  ],
  "branch_status": {
    "current": "main",
    "local_branches": 5,
    "remote_branches": 8,
    "commits_ahead": 0,
    "commits_behind": 0,
    "default_branch": "main",
    "on_default_branch": true
  },
  "retrieved_at": "2026-02-06T00:00:00Z"
}
```

**Error Responses:**

**404 Not Found - Repository Not Found:**

```json
{
  "error": {
    "code": "repository_not_found",
    "message": "Repository not found"
  }
}
```

**500 Internal Server Error - Git Operation Failed:**

```json
{
  "error": {
    "code": "git_operation_failed",
    "message": "Failed to retrieve repository status",
    "details": {
      "reason": "Repository is not a valid Git repository"
    }
  }
}
```

---

## 6. REPOSITORY BRANCH API

### 6.1. List Branches

#### API-014-008: GET /api/v1/repositories/:id/branches

**Element ID:** DES-API-014-008
**Name:** List Branches
**Type:** REST Endpoint
**Language:** Rust (Axum)
**HTTP Method:** GET
**Endpoint:** `/api/v1/repositories/:id/branches`

**Description:** Retrieves a list of all branches for a repository, including local and remote branches. This endpoint enables branch management and navigation.

**Request:**

```rust
use axum::{Json, Path, Query, State};
use serde::Deserialize;

/// Query parameters for listing branches
#[derive(Debug, Deserialize)]
pub struct ListBranchesQuery {
    /// Filter by branch type (local, remote, all)
    #[serde(default = "default_branch_type")]
    pub branch_type: String,

    /// Include merged branches (default: false)
    #[serde(default)]
    pub include_merged: bool,
}

/// Handler for listing repository branches
///
/// # Parameters
///
/// * `id` - Repository unique identifier (UUID v4)
/// * `params` - Query parameters for filtering
/// * `user` - Authenticated user context
///
/// # Returns
///
/// JSON response containing list of branches
///
/// # Errors
///
/// * `400 Bad Request` - Invalid query parameters
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User lacks permission to access repository
/// * `404 Not Found` - Repository does not exist
/// * `500 Internal Server Error` - Server error during branch listing
pub async fn list_branches(
    Path(id): Path<RepositoryId>,
    Query(params): Query<ListBranchesQuery>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<BranchListResponse>, ApiError>;

fn default_branch_type() -> String {
    "all".to_string()
}
```

**Response:**

```rust
use serde::Serialize;

/// Response containing list of branches
#[derive(Debug, Serialize)]
pub struct BranchListResponse {
    /// List of branches
    pub branches: Vec<BranchInfo>,

    /// Total branches count
    pub total: usize,

    /// Current branch
    pub current_branch: String,

    /// Default branch
    pub default_branch: String,
}

/// Branch information
#[derive(Debug, Serialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,

    /// Branch is current indicator
    pub is_current: bool,

    /// Branch is remote indicator
    pub is_remote: bool,

    /// Branch is merged indicator
    pub is_merged: bool,

    /// Last commit SHA
    pub last_commit_sha: String,

    /// Last commit message
    pub last_commit_message: String,

    /// Last commit author
    pub last_commit_author: String,

    /// Last commit timestamp
    pub last_commit_at: DateTime<Utc>,

    /// Commits ahead of remote
    pub commits_ahead: usize,

    /// Commits behind remote
    pub commits_behind: usize,

    /// Branch creation timestamp
    pub created_at: Option<DateTime<Utc>>,
}
```

**Constraints:**

- `id`: Must be valid UUID v4 format
- `branch_type`: Must be one of `local`, `remote`, `all`
- Repository must exist and be accessible to authenticated user

**Dependencies:**

- REQ-SRV-032: Branch Management
- REQ-SRV-047: Commit Management
- DES-DM-002: RepositoryPath
- DES-DM-005: Repository

**Rationale:** Branch listing enables users to view and navigate repository branches, supporting branch management workflows and collaboration features.

**Security Considerations:**

- Requires authentication with valid JWT token
- Enforces RBAC based on user's repository access permissions
- Validates repository ID format to prevent injection attacks
- Only returns branches for repositories accessible to authenticated user
- Sanitizes branch names and commit messages
- Logs branch listing operations for audit purposes

**Performance Considerations:**

- Uses in-memory caching for branch information
- Git branch queries are optimized for performance
- Response time target: < 100ms for typical repositories

**Example Request:**

```http
GET /api/v1/repositories/repo-123e4567-e89b-12d3-a456-426614174000/branches?branch_type=all&include_merged=false
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Host: api.tachyon.example.com
```

**Example Response:**

```json
{
  "branches": [
    {
      "name": "main",
      "is_current": true,
      "is_remote": true,
      "is_merged": false,
      "last_commit_sha": "a1b2c3d4e5f6789012345678901234567890123",
      "last_commit_message": "Update API documentation",
      "last_commit_author": "John Doe",
      "last_commit_at": "2026-02-05T23:30:00Z",
      "commits_ahead": 0,
      "commits_behind": 0,
      "created_at": "2025-01-15T10:00:00Z"
    },
    {
      "name": "develop",
      "is_current": false,
      "is_remote": true,
      "is_merged": false,
      "last_commit_sha": "b2c3d4e5f67890123456789012345678901234",
      "last_commit_message": "Add feature branch",
      "last_commit_author": "Jane Smith",
      "last_commit_at": "2026-02-04T15:20:00Z",
      "commits_ahead": 2,
      "commits_behind": 0,
      "created_at": "2025-03-20T14:30:00Z"
    }
  ],
  "total": 2,
  "current_branch": "main",
  "default_branch": "main"
}
```

**Error Responses:**

**400 Bad Request - Invalid Branch Type:**

```json
{
  "error": {
    "code": "invalid_branch_type",
    "message": "Invalid branch type parameter",
    "details": {
      "branch_type": "must be one of: local, remote, all"
    }
  }
}
```

**404 Not Found - Repository Not Found:**

```json
{
  "error": {
    "code": "repository_not_found",
    "message": "Repository not found"
  }
}
```

---

## 7. REPOSITORY SECURITY

### 7.1. Authentication

All Repository API endpoints require authentication using JSON Web Tokens (JWT). Authentication tokens are issued by the Tachyon authentication service and must be included in the `Authorization` header.

**Authentication Header Format:**

```http
Authorization: Bearer <jwt_token>
```

**Token Validation:**

- Tokens are validated for signature, expiration, and issuer
- Invalid or expired tokens result in `401 Unauthorized` responses
- Token claims include user ID, role, and permissions

**Token Claims:**

```rust
use serde::{Deserialize, Serialize};

/// JWT token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User unique identifier
    pub sub: String,

    /// Token issuer
    pub iss: String,

    /// Token audience
    pub aud: String,

    /// Token expiration timestamp
    pub exp: i64,

    /// Token issued at timestamp
    pub iat: i64,

    /// User role
    pub role: String,

    /// User permissions
    pub permissions: Vec<String>,
}
```

**Authentication Flow:**

1. Client obtains JWT token from authentication service
2. Client includes token in `Authorization` header for each request
3. Server validates token signature and claims
4. Server extracts user context from token claims
5. Request is processed with authenticated user context

**Security Considerations:**

- Tokens are signed using RS256 algorithm for enhanced security
- Token expiration is enforced to limit exposure window
- Token refresh mechanism supports long-lived sessions
- Token revocation list supports immediate token invalidation
- Failed authentication attempts are logged for security monitoring

### 7.2. Authorization

Repository API enforces Role-Based Access Control (RBAC) to ensure users have appropriate permissions for repository operations.

**Access Levels:**

- **Read:** View repository information and status
- **Write:** Modify repository content and configuration
- **Admin:** Full administrative access including deletion
- **Owner:** Full ownership rights including permission management

**Authorization Enforcement:**

Authorization is enforced at multiple levels:

1. **Endpoint Level:** Each endpoint defines required access level
2. **Resource Level:** User's access level is checked against repository permissions
3. **Operation Level:** Specific operations may require elevated permissions

**Permission Matrix:**

| Operation | Read | Write | Admin | Owner |
|-----------|-------|-------|-------|-------|
| List Repositories | ✓ | ✓ | ✓ | ✓ |
| Get Repository | ✓ | ✓ | ✓ | ✓ |
| Create Repository | ✗ | ✓ | ✓ | ✓ |
| Update Repository | ✗ | ✓ | ✓ | ✓ |
| Delete Repository | ✗ | ✗ | ✓ | ✓ |
| Sync Repository | ✗ | ✓ | ✓ | ✓ |
| Get Status | ✓ | ✓ | ✓ | ✓ |
| List Branches | ✓ | ✓ | ✓ | ✓ |

**Authorization Middleware:**

```rust
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// Middleware for enforcing repository access permissions
pub async fn require_repository_access(
    State(user): State<AuthenticatedUser>,
    required_level: AccessLevel,
) -> Result<(), ApiError> {
    let user_level = user.access_level_for_repository(&repository_id);
    
    if user_level >= required_level {
        Ok(())
    } else {
        Err(ApiError::Forbidden {
            message: "Insufficient permissions for this operation".to_string(),
        })
    }
}
```

**Security Considerations:**

- Authorization checks are performed on every request
- Permission changes are reflected immediately (no caching of permissions)
- Audit logs record all authorization decisions
- Failed authorization attempts are logged for security monitoring
- Permission escalation requires explicit approval and audit trail

---

## 8. REFERENCES

### 8.1. Architecture Decision Records

- [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md): Rust as Primary Language
- [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md): Axum for HTTP/2 Server
- [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md): Tokio for Async Runtime

### 8.2. Design Documents

- [DES-DM-002](../../.specs/04_future_state/design/data_models.md): RepositoryPath Type
- [DES-DM-005](../../.specs/04_future_state/design/data_models.md): Repository Type
- [DES-DM-008](../../.specs/04_future_state/design/data_models.md): GitStatus Type
- [DES-API-001](../../.specs/04_future_state/design/api_interfaces.md): API Interface Patterns

### 8.3. Requirements

- [REQ-SRV-025](../../.specs/04_future_state/reqs/server_requirements.md): Repository List
- [REQ-SRV-026](../../.specs/04_future_state/reqs/server_requirements.md): Repository Retrieval
- [REQ-SRV-027](../../.specs/04_future_state/reqs/server_requirements.md): Repository Creation
- [REQ-SRV-028](../../.specs/04_future_state/reqs/server_requirements.md): Repository Update
- [REQ-SRV-029](../../.specs/04_future_state/reqs/server_requirements.md): Repository Deletion
- [REQ-SRV-030](../../.specs/04_future_state/reqs/server_requirements.md): Repository Synchronization
- [REQ-SRV-031](../../.specs/04_future_state/reqs/server_requirements.md): Repository Status
- [REQ-SRV-032](../../.specs/04_future_state/reqs/server_requirements.md): Branch Management
- [REQ-SRV-047](../../.specs/04_future_state/reqs/server_requirements.md): Commit Management
- [REQ-SRV-081](../../.specs/04_future_state/reqs/server_requirements.md): RBAC Enforcement
- [REQ-DESK-037](../../.specs/04_future_state/reqs/desktop_requirements.md): Repository Cloning

### 8.4. Standards

- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md): Coding Standards
- ISO/IEC 26514:2021: Systems and software engineering — Requirements for designers and developers of user documentation
- IEEE 1063-2001: IEEE Standard for Software User Documentation

### 8.5. External References

- [Rust Programming Language](https://www.rust-lang.org/): Official Rust documentation
- [Axum Framework](https://github.com/tokio-rs/axum): Axum web framework documentation
- [Tokio Runtime](https://tokio.rs/): Tokio async runtime documentation
- [git2 Crate](https://github.com/rust-lang/git2-rs): libgit2 bindings for Rust
- [Serde](https://serde.rs/): Serialization framework for Rust
- [JSON Web Tokens (JWT)](https://jwt.io/): JWT specification and best practices
- [HTTP/2 Specification](https://httpwg.org/specs/rfc7540.html): HTTP/2 protocol specification

---

**Document Control:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| V1.0 | February 2026 | Initial version |

---

**End of Document**