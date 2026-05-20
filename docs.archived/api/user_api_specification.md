# TACHYON: USER API SPECIFICATION

**Document ID:** TACHYON-API-016-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** Technical Specification Document
**Dependencies:** [TACHYON-STD-V1.0](../.adrs/ [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md), [TACHYON-ADR-003-V1.0](../.adrs/adr-003-lru-cache-target.md), [TACHYON-ADR-007-V1.0](../.adrs/adr-007-thread-safety-strategy.md), [TACHYON-TMA-V1.0](../.adrs/

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [User API Design Principles](#2-user-api-design-principles)
3. [User CRUD API](#3-user-crud-api)
   - [List Users](#31-list-users)
   - [Get User](#32-get-user)
   - [Create User](#33-create-user)
   - [Update User](#34-update-user)
   - [Delete User](#35-delete-user)
4. [User Activity API](#4-user-activity-api)
5. [User Security](#5-user-security)
   - [Authentication](#51-authentication)
   - [Authorization](#52-authorization)
6. [User Performance](#6-user-performance)
   - [Latency Requirements](#61-latency-requirements)
   - [Caching Strategies](#62-caching-strategies)
7. [References](#7-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document specifies the User API for the Tachyon toolchain, providing comprehensive interface definitions for user management operations. The User API enables creation, retrieval, update, and deletion of user accounts, as well as user activity tracking and security management. This specification serves as the authoritative source for all User API implementations across desktop, server, and web components.

### 1.2. Scope

The User API encompasses the following functional domains:

1. **User CRUD Operations:** Create, read, update, and delete user accounts
2. **User Activity Tracking:** Monitor and retrieve user activity logs
3. **Authentication Management:** Handle user authentication sessions and credentials
4. **Authorization Enforcement:** Implement Role-Based Access Control (RBAC) for user operations
5. **User Profile Management:** Manage user preferences, settings, and metadata

The User API operates within the Tachyon server component, implemented using Rust with the Axum web framework and Tokio async runtime. All endpoints enforce authentication and authorization according to the security requirements specified in [TACHYON-TMA-V1.0](../.adrs/

### 1.3. Target Audience

This specification is intended for:

- **Backend Developers:** Implementing User API endpoints in Rust
- **Frontend Developers:** Consuming User API from web and desktop applications
- **Security Engineers:** Ensuring User API security controls are properly implemented
- **QA Engineers:** Testing User API functionality and security
- **System Architects:** Understanding User API integration within the broader Tachyon architecture

### 1.4. Compliance and Standards

This specification adheres to the following standards and requirements:

| Standard/Requirement | Reference | Relevance |
|---------------------|-------------|-------------|
| ISO/IEC 26514:2021 | [TACHYON-STD-V1.0](../.adrs/ | Documentation quality and structure |
| IEEE 1063-2001 | [TACHYON-STD-V1.0](../.adrs/ | User documentation standards |
| Rust Edition 2024 | [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md) | Type safety and memory safety |
| Axum v0.7 | [TACHYON-ADR-003-V1.0](../.adrs/adr-003-lru-cache-target.md) | HTTP/2 server framework |
| Tokio v1 | [TACHYON-ADR-007-V1.0](../.adrs/adr-007-thread-safety-strategy.md) | Async runtime |
| STRIDE Threat Model | [TACHYON-TMA-V1.0](../.adrs/ | Security threat analysis |

---

## 2. USER API DESIGN PRINCIPLES

### 2.1. RESTful Design

The User API adheres to REST (Representational State Transfer) architectural principles, ensuring resource-oriented design with standard HTTP methods and status codes.

**Resource Identification:**

Users are identified by Universally Unique Identifiers (UUID v4) to prevent Insecure Direct Object Reference (IDOR) vulnerabilities. Sequential integer IDs are avoided as they enable enumeration attacks.

**HTTP Method Semantics:**

| HTTP Method | Operation | Idempotent | Safe | Description |
|--------------|-----------|--------------|-------|-------------|
| GET | Retrieve user data | Yes | Yes | Retrieve user or list of users |
| POST | Create user | No | No | Create new user account |
| PUT | Replace user | Yes | No | Replace entire user record |
| PATCH | Update user | No | No | Partially update user fields |
| DELETE | Delete user | Yes | No | Delete user account |

**Status Code Usage:**

| Status Code | Usage | Example Scenarios |
|-------------|---------|-------------------|
| 200 OK | Successful GET/PUT/PATCH | User retrieved or updated successfully |
| 201 Created | Successful POST | User account created successfully |
| 204 No Content | Successful DELETE | User account deleted successfully |
| 400 Bad Request | Invalid request | Missing required fields, invalid data format |
| 401 Unauthorized | Missing authentication | No authentication token provided |
| 403 Forbidden | Insufficient permissions | User lacks permission to access resource |
| 404 Not Found | Resource not found | User ID does not exist |
| 409 Conflict | Resource conflict | Email or username already exists |
| 422 Unprocessable Entity | Validation error | Business rule violation |
| 429 Too Many Requests | Rate limit exceeded | Too many requests in time window |
| 500 Internal Server Error | Server error | Unexpected server failure |

### 2.2. Type Safety and Validation

The User API leverages Rust's type system to enforce compile-time guarantees and runtime validation.

**Compile-Time Type Safety:**

All request and response structures are defined using Rust's type system with derive macros for serialization and deserialization:

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 255))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
}
```

**Runtime Validation:**

The `validator` crate provides declarative validation rules enforced at runtime:

| Validation Rule | Purpose | Example |
|----------------|---------|---------|
| `length(min, max)` | String length constraints | Username: 1-255 characters |
| `email` | Email format validation | Email must be valid RFC 5322 |
| `regex(pattern)` | Pattern matching | Password complexity requirements |
| `custom(fn)` | Custom validation logic | Business rule validation |

**Boundary Value Analysis:**

All user inputs are validated against defined domain constraints:

| Field | Type | Min | Max | Default | Validation |
|-------|------|------|------|-------------|
| `username` | String | 1 | 255 | N/A | Alphanumeric, underscore, hyphen |
| `email` | String | 5 | 254 | N/A | Valid email format |
| `password` | String | 8 | 128 | N/A | Complexity requirements |
| `display_name` | String | 1 | 255 | N/A | Unicode characters |
| `bio` | String | 0 | 1000 | N/A | Markdown supported |
| `offset` | usize | 0 | 2^64-1 | 0 | Non-negative integer |
| `limit` | usize | 1 | 100 | 20 | Bounded pagination |

### 2.3. Security by Design

The User API implements security controls at multiple layers to protect against threats identified in the threat model.

**Authentication:**

All User API endpoints require valid authentication tokens. The API uses JSON Web Tokens (JWT) with the following characteristics:

- **Algorithm:** RS256 (RSA Signature with SHA-256)
- **Issuer:** Tachyon server domain
- **Audience:** Tachyon API endpoints
- **Expiration:** 1 hour (configurable)
- **Token Type:** Bearer token in Authorization header

**Authorization:**

Role-Based Access Control (RBAC) enforces the principle of least privilege:

| Role | Permissions | Description |
|-------|--------------|-------------|
| `admin` | Full access | Can perform all operations on all users |
| `moderator` | Limited admin | Can view and update users, cannot delete |
| `user` | Self-only | Can only view and update own profile |
| `guest` | Read-only | Can only view public user profiles |

**Rate Limiting:**

Rate limiting prevents brute force attacks and abuse:

| Endpoint | Rate Limit | Window | Algorithm |
|----------|-------------|---------|------------|
| POST /api/v1/users | 5 requests | 1 minute | Token bucket |
| POST /api/v1/auth/login | 10 requests | 5 minutes | Fixed window |
| GET /api/v1/users | 100 requests | 1 minute | Sliding window |

**Input Sanitization:**

All user inputs are sanitized to prevent injection attacks:

- **SQL Injection:** Parameterized queries with prepared statements
- **XSS:** HTML sanitization using ammonia crate
- **Command Injection:** Strict allow-list for file operations
- **Path Traversal:** Path canonicalization and validation

### 2.4. Performance Optimization

The User API is designed for high performance with sub-15 millisecond response times for read operations.

**Caching Strategy:**

User data is cached at multiple levels:

| Cache Type | Scope | TTL | Invalidation |
|-------------|--------|-----|--------------|
| In-memory | User profile | 5 minutes | On update |
| HTTP Cache | GET responses | 60 seconds | On mutation |
| CDN | Static assets | 1 hour | Version-based |

**Database Optimization:**

Queries are optimized for performance:

- Indexed queries on email and username fields
- Pagination to limit result set size
- Projection to retrieve only required fields
- Connection pooling for efficient resource usage

**Async Architecture:**

Tokio's work-stealing scheduler enables efficient concurrent request handling:

- Multi-threaded runtime with configurable thread pool
- Non-blocking I/O for database operations
- Efficient task scheduling and context switching

### 2.5. Error Handling

The User API implements comprehensive error handling with clear, actionable error messages.

**Error Response Format:**

```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional error details
    pub details: Option<serde_json::Value>,
    
    /// Request ID for tracing
    pub request_id: String,
}
```

**Error Codes:**

| Error Code | HTTP Status | Description |
|------------|--------------|-------------|
| `AUTH_MISSING` | 401 | Authentication token not provided |
| `AUTH_INVALID` | 401 | Invalid or expired authentication token |
| `AUTH_FORBIDDEN` | 403 | Insufficient permissions |
| `USER_NOT_FOUND` | 404 | User does not exist |
| `USER_EXISTS` | 409 | Email or username already exists |
| `VALIDATION_ERROR` | 422 | Input validation failed |
| `RATE_LIMIT_EXCEEDED` | 429 | Rate limit exceeded |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

**Logging and Tracing:**

All errors are logged with structured logging using the `tracing` crate:

- Request ID for distributed tracing
- Error context and stack traces
- User ID for security auditing
    - Timestamp and severity level

---

## 3. USER CRUD API

### 3.1. List Users

#### API-USR-001: List Users

**Element ID:** DES-USR-001
**Name:** GET /api/v1/users
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

Retrieves a paginated list of user accounts accessible to the authenticated user. This endpoint supports filtering, sorting, and searching to enable efficient user discovery and management. The response includes user metadata with sensitive fields excluded based on the requester's permissions.

**Request:**

```rust
use axum::extract::{Query, State};
use serde::Deserialize;
use validator::Validate;

/// Query parameters for listing users
#[derive(Debug, Deserialize, Validate)]
pub struct ListUsersQuery {
    /// Pagination offset (default: 0)
    #[validate(range(min = 0))]
    pub offset: Option<usize>,
    
    /// Page size (default: 20, max: 100)
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,
    
    /// Sort field (username, email, created_at, updated_at)
    pub sort: Option<String>,
    
    /// Sort order (asc, desc)
    pub order: Option<String>,
    
    /// Filter by role (admin, moderator, user, guest)
    pub role: Option<String>,
    
    /// Filter by status (active, inactive, suspended, deleted)
    pub status: Option<String>,
    
    /// Search query (searches username, email, display_name)
    #[validate(length(max = 100))]
    pub q: Option<String>,
}

/// Handler function for listing users
pub async fn list_users(
    Query(params): Query<ListUsersQuery>,
    State(auth_user): State<AuthenticatedUser>,
) -> Result<Json<UserListResponse>, ApiError>;
```

**Response:**

```rust
use serde::Serialize;
use chrono::{DateTime, Utc};

/// Response containing paginated list of users
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    /// List of user records
    pub users: Vec<UserSummary>,
    
    /// Total count of matching users
    pub total: usize,
    
    /// Current pagination offset
    pub offset: usize,
    
    /// Current page size
    pub limit: usize,
    
    /// Has more pages indicator
    pub has_more: bool,
}

/// Summary of user information (excludes sensitive data)
#[derive(Debug, Serialize)]
pub struct UserSummary {
    /// Unique user identifier
    pub id: UserId,
    
    /// Username for login
    pub username: String,
    
    /// Display name (may be null)
    pub display_name: Option<String>,
    
    /// Email address (hidden for non-admin users)
    pub email: Option<String>,
    
    /// User role
    pub role: UserRole,
    
    /// User account status
    pub status: UserStatus,
    
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Profile avatar URL (may be null)
    pub avatar_url: Option<String>,
}

/// User role enumeration
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Full system administrator
    Admin,
    /// Content moderator
    Moderator,
    /// Regular user
    User,
    /// Guest user (limited access)
    Guest,
}

/// User account status enumeration
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// Active and fully functional
    Active,
    /// Account created but not activated
    Inactive,
    /// Temporarily suspended
    Suspended,
    /// Marked for deletion
    Deleted,
}
```

**Constraints:**

| Parameter | Type | Min | Max | Default | Validation |
|-----------|------|-----|------|---------|-------------|
| `offset` | usize | 0 | 2^64-1 | 0 | Non-negative integer |
| `limit` | usize | 1 | 100 | 20 | Bounded pagination |
| `sort` | String | N/A | N/A | created_at | Must be valid field |
| `order` | String | N/A | N/A | asc | Must be asc or desc |
| `role` | String | N/A | N/A | N/A | Must be valid role |
| `status` | String | N/A | N/A | N/A | Must be valid status |
| `q` | String | 0 | 100 | N/A | Search query string |

**Dependencies:**

- REQ-SRV-001: User Management
- REQ-SRV-081: RBAC Enforcement
- [TACHYON-ADR-003-V1.0](../.adrs/adr-003-lru-cache-target.md): Axum HTTP/2 framework

**Rationale:**

Paginated user listing enables efficient browsing of user directories without overwhelming the client or server. Filtering and sorting capabilities allow users to quickly locate specific accounts. The exclusion of sensitive data (passwords, email for non-admins) protects user privacy while enabling necessary administrative functions.

**Security Considerations:**

1. **Authentication Required:** All requests must include a valid JWT token in the Authorization header.

2. **Authorization Enforcement:**
   - Regular users can only list users with `role: guest` or `role: user`
   - Moderators can list users with `role: user`, `role: guest`, or `role: moderator`
   - Admins can list all users regardless of role or status

3. **Data Filtering:**
   - Email addresses are only returned to admin users
   - Suspended and deleted users are only visible to admins
   - Inactive users are visible to all authenticated users

4. **Rate Limiting:**
   - 100 requests per minute per authenticated user
   - Token bucket algorithm for smooth rate limiting
   - Rate limit headers included in response

5. **Input Validation:**
   - All query parameters are validated against defined constraints
   - Invalid parameters return 422 Unprocessable Entity
   - SQL injection prevented through parameterized queries

**Example Request:**

```http
GET /api/v1/users?offset=0&limit=20&sort=created_at&order=desc&role=user&status=active HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

**Example Response (200 OK):**

```json
{
  "users": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "johndoe",
      "display_name": "John Doe",
      "email": null,
      "role": "user",
      "status": "active",
      "created_at": "2026-01-15T10:30:00Z",
      "updated_at": "2026-02-01T14:22:33Z",
      "avatar_url": "https://cdn.tachyon.example.com/avatars/550e8400.png"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "username": "janedoe",
      "display_name": "Jane Doe",
      "email": null,
      "role": "user",
      "status": "active",
      "created_at": "2026-01-20T09:15:00Z",
      "updated_at": "2026-02-03T11:45:12Z",
      "avatar_url": "https://cdn.tachyon.example.com/avatars/660e8400.png"
    }
  ],
  "total": 42,
  "offset": 0,
  "limit": 20,
  "has_more": true
}
```

**Error Responses:**

**401 Unauthorized:**

```json
{
  "code": "AUTH_MISSING",
  "message": "Authentication token is required",
  "details": null,
  "request_id": "req_abc123def456"
}
```

**403 Forbidden:**

```json
{
  "code": "AUTH_FORBIDDEN",
  "message": "Insufficient permissions to list users with role: admin",
  "details": {
      "required_role": "admin",
      "user_role": "user"
  },
  "request_id": "req_abc123def456"
}
```

**422 Unprocessable Entity:**

```json
{
  "code": "VALIDATION_ERROR",
  "message": "Invalid query parameters",
  "details": {
      "errors": [
          {
              "field": "limit",
              "message": "must be between 1 and 100"
          }
      ]
  },
  "request_id": "req_abc123def456"
}
```

**429 Too Many Requests:**

```json
{
  "code": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded",
  "details": {
      "limit": 100,
      "window": "1 minute",
      "retry_after": 30
  },
  "request_id": "req_abc123def456"
}
```

**Performance Characteristics:**

| Metric | Target | P50 | P95 | P99 |
|--------|---------|-----|-----|-----|
| Response Time | < 50ms | 15ms | 35ms | 48ms |
| Database Queries | ≤ 2 | 1 | 2 | 2 |
| Cache Hit Rate | > 80% | 85% | 82% | 78% |

**Caching Strategy:**

- **In-Memory Cache:** User list data cached for 60 seconds
- **Cache Key:** `users:{offset}:{limit}:{sort}:{order}:{role}:{status}:{q}`
- **Invalidation:** Cache invalidated on user creation, update, or deletion
- **HTTP Cache:** Cache-Control header with max-age=60, must-revalidate

---

### 3.2. Get User

#### API-USR-002: Get User

**Element ID:** DES-USR-002
**Name:** GET /api/v1/users/:id
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

Retrieves detailed information for a specific user by their unique identifier. This endpoint returns comprehensive user profile data including preferences, activity statistics, and role information. Sensitive fields are conditionally included based on the requester's permissions and relationship to the target user.

**Request:**

```rust
use axum::extract::{Path, State};
use uuid::Uuid;

/// Path parameter containing user ID
pub type UserId = Uuid;

/// Handler function for retrieving a specific user
pub async fn get_user(
    Path(id): Path<UserId>,
    State(auth_user): State<AuthenticatedUser>,
) -> Result<Json<UserResponse>, ApiError>;
```

**Response:**

```rust
use serde::Serialize;
use chrono::{DateTime, Utc};

/// Detailed user information response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// User identifier
    pub id: UserId,
    
    /// Username for login
    pub username: String,
    
    /// Display name (may be null)
    pub display_name: Option<String>,
    
    /// Email address (included for admins and self)
    pub email: Option<String>,
    
    /// User role
    pub role: UserRole,
    
    /// User account status
    pub status: UserStatus,
    
    /// Profile avatar URL (may be null)
    pub avatar_url: Option<String>,
    
    /// User biography (Markdown format, may be null)
    pub bio: Option<String>,
    
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Last login timestamp (may be null)
    pub last_login_at: Option<DateTime<Utc>>,
    
    /// User preferences
    pub preferences: UserPreferences,
    
    /// Activity statistics (included for admins and self)
    pub activity: Option<UserActivityStats>,
    
    /// Cache hit indicator
    pub cached: bool,
}

/// User preferences configuration
#[derive(Debug, Serialize)]
pub struct UserPreferences {
    /// Theme preference (light, dark, system)
    pub theme: String,
    
    /// Language preference (ISO 639-1 code)
    pub language: String,
    
    /// Timezone preference (IANA timezone database)
    pub timezone: String,
    
    /// Email notifications enabled
    pub email_notifications: bool,
    
    /// Desktop notifications enabled
    pub desktop_notifications: bool,
    
    /// Custom preferences (JSON object)
    pub custom: serde_json::Value,
}

/// User activity statistics
#[derive(Debug, Serialize)]
pub struct UserActivityStats {
    /// Total documents created
    pub documents_created: usize,
    
    /// Total documents edited
    pub documents_edited: usize,
    
    /// Total comments made
    pub comments_made: usize,
    
    /// Total login count
    pub login_count: usize,
    
    /// Account age in days
    pub account_age_days: usize,
    
    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,
}
```

**Constraints:**

| Parameter | Type | Validation | Description |
|-----------|------|-------------|-------------|
| `id` | UUID v4 | Must be valid UUID | User unique identifier |

**Dependencies:**

- REQ-SRV-002: User Retrieval
- REQ-SRV-081: RBAC Enforcement
- [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md): Rust type safety

**Rationale:**

Individual user retrieval enables viewing detailed profile information for user management, collaboration, and social features. Conditional inclusion of sensitive data (email, activity stats) balances privacy with administrative needs. The cached indicator enables clients to optimize refresh strategies.

**Security Considerations:**

1. **Authentication Required:** All requests must include a valid JWT token.

2. **Authorization Enforcement:**
   - Users can retrieve their own full profile
   - Users can retrieve public profiles of other users (limited data)
   - Admins can retrieve full profiles of any user
   - Moderators can retrieve profiles of regular users and guests

3. **Data Filtering Based on Requester:**

| Requester Role | Target User | Email | Activity Stats | Bio |
|----------------|-------------|-------|----------------|-----|
| Admin | Any | Yes | Yes | Yes |
| Moderator | User/Guest | No | Yes | Yes |
| User | Self | Yes | Yes | Yes |
| User | Other | No | No | Yes |
| Guest | Any | No | No | Yes |

4. **IDOR Prevention:**
   - UUID v4 identifiers prevent enumeration attacks
   - Authorization checks performed before database query
   - Invalid IDs return 404 without revealing existence

5. **Rate Limiting:**
   - 200 requests per minute per authenticated user
   - Higher limit for list users due to lower data volume

6. **Cache Privacy:**
   - Public profile data cached for 5 minutes
   - Private profile data (email, activity) never cached
   - Cache key includes requester role for proper filtering

**Example Request:**

```http
GET /api/v1/users/550e8400-e29b-41d4-a716-446655440000 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

**Example Response (200 OK) - Admin Requesting Other User:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "johndoe",
  "display_name": "John Doe",
  "email": "john.doe@example.com",
  "role": "user",
  "status": "active",
  "avatar_url": "https://cdn.tachyon.example.com/avatars/550e8400.png",
  "bio": "Software developer and documentation enthusiast.",
  "created_at": "2026-01-15T10:30:00Z",
  "updated_at": "2026-02-01T14:22:33Z",
  "last_login_at": "2026-02-05T09:15:22Z",
  "preferences": {
      "theme": "dark",
      "language": "en",
      "timezone": "America/New_York",
      "email_notifications": true,
      "desktop_notifications": false,
      "custom": {}
  },
  "activity": {
      "documents_created": 42,
      "documents_edited": 156,
      "comments_made": 89,
      "login_count": 234,
      "account_age_days": 21,
      "last_activity_at": "2026-02-05T09:15:22Z"
  },
  "cached": false
}
```

**Example Response (200 OK) - User Requesting Other User:**

```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "username": "janedoe",
  "display_name": "Jane Doe",
  "email": null,
  "role": "user",
  "status": "active",
  "avatar_url": "https://cdn.tachyon.example.com/avatars/660e8400.png",
  "bio": "Technical writer and editor.",
  "created_at": "2026-01-20T09:15:00Z",
  "updated_at": "2026-02-03T11:45:12Z",
  "last_login_at": null,
  "preferences": {
      "theme": "light",
      "language": "en",
      "timezone": "Europe/London",
      "email_notifications": true,
      "desktop_notifications": true,
      "custom": {}
  },
  "activity": null,
  "cached": true
}
```

**Error Responses:**

**401 Unauthorized:**

```json
{
  "code": "AUTH_MISSING",
  "message": "Authentication token is required",
  "details": null,
  "request_id": "req_xyz789abc123"
}
```

**403 Forbidden:**

```json
{
  "code": "AUTH_FORBIDDEN",
  "message": "Insufficient permissions to view this user profile",
  "details": {
      "required_role": "admin",
      "user_role": "user"
  },
  "request_id": "req_xyz789abc123"
}
```

**404 Not Found:**

```json
{
  "code": "USER_NOT_FOUND",
  "message": "User not found",
  "details": {
      "user_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "request_id": "req_xyz789abc123"
}
```

**Performance Characteristics:**

| Metric | Target | P50 | P95 | P99 |
|--------|---------|-----|-----|-----|
| Response Time | < 20ms | 8ms | 15ms | 18ms |
| Database Queries | ≤ 1 | 1 | 1 | 1 |
| Cache Hit Rate | > 90% | 92% | 89% | 85% |

**Caching Strategy:**

- **In-Memory Cache:** Public user data cached for 5 minutes
- **Cache Key:** `user:{id}:public` for public data
- **Cache Key:** `user:{id}:private:{requester_id}` for private data
- **Invalidation:** Cache invalidated on user profile update
- **HTTP Cache:** Cache-Control header with max-age=300 for public profiles
- **Private Data:** No caching for email and activity statistics

---

### 3.3. Create User

#### API-USR-003: Create User

**Element ID:** DES-USR-003
**Name:** POST /api/v1/users
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

Creates a new user account with the provided credentials and profile information. This endpoint validates all input data, checks for duplicate usernames and emails, and creates the user with appropriate default settings. Upon successful creation, the user account is initialized with default preferences and role assignment based on system configuration.

**Request:**

```rust
use axum::{Json, State};
use serde::Deserialize;
use validator::Validate;

/// Request body for creating a new user
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    /// Username for login (alphanumeric, underscore, hyphen)
    #[validate(length(min = 1, max = 255))]
    #[validate(regex = "^[a-zA-Z0-9_-]+$")]
    pub username: String,
    
    /// Email address (must be unique)
    #[validate(email)]
    #[validate(length(max = 254))]
    pub email: String,
    
    /// Password (will be hashed)
    #[validate(length(min = 8, max = 128))]
    #[validate(custom = "validate_password_complexity")]
    pub password: String,
    
    /// Display name (optional, defaults to username)
    #[validate(length(max = 255))]
    pub display_name: Option<String>,
    
    /// User biography (Markdown format, optional)
    #[validate(length(max = 1000))]
    pub bio: Option<String>,
    
    /// Initial user role (optional, requires admin permission)
    pub role: Option<UserRole>,
    
    /// User preferences (optional)
    pub preferences: Option<UserPreferences>,
}

/// Handler function for creating a new user
pub async fn create_user(
    Json(req): Json<CreateUserRequest>,
    State(auth_user): State<AuthenticatedUser>,
) -> Result<Json<UserResponse>, ApiError>;

/// Custom password complexity validation
fn validate_password_complexity(password: &str) -> Result<(), validator::ValidationError> {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    if has_upper && has_lower && has_digit && has_special {
        Ok(())
    } else {
        Err(validator::ValidationError::new("complexity"))
    }
}
```

**Response:**

```rust
// Returns UserResponse (see API-USR-002)
```

**Constraints:**

| Field | Type | Min | Max | Validation | Description |
|-------|------|-----|------|-------------|
| `username` | String | 1 | 255 | Alphanumeric, underscore, hyphen |
| `email` | String | 5 | 254 | Valid email format, unique |
| `password` | String | 8 | 128 | Complexity requirements |
| `display_name` | String | 0 | 255 | Optional, defaults to username |
| `bio` | String | 0 | 1000 | Optional, Markdown format |
| `role` | UserRole | N/A | N/A | Optional, requires admin |

**Password Complexity Requirements:**

- Minimum length: 8 characters
- Maximum length: 128 characters
- Must contain at least one uppercase letter
- Must contain at least one lowercase letter
- Must contain at least one digit
- Must contain at least one special character

**Dependencies:**

- REQ-SRV-003: User Creation
- REQ-SRV-081: RBAC Enforcement
- REQ-SEC-001: Password Security
- [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md): Rust type safety
- [TACHYON-TMA-V1.0](../.adrs/ Security requirements

**Rationale:**

User creation enables onboarding of new users to the Tachyon system. Comprehensive validation ensures data integrity and security. Password complexity requirements protect against brute force and dictionary attacks. Optional role assignment allows administrators to pre-configure user permissions during account creation.

**Security Considerations:**

1. **Authentication Required:**
   - Admin role required to specify custom role
   - Self-registration may be enabled with default user role

2. **Authorization Enforcement:**
   - Only admins can create users with custom roles
   - Self-registration creates users with default role (guest or user)
   - Role assignment logged for audit trail

3. **Password Security:**
   - Passwords are hashed using Argon2id (memory-hard, GPU-resistant)
   - Salt is generated per-user using cryptographically secure RNG
   - Password complexity enforced before hashing
   - Passwords never logged or stored in plaintext

4. **Data Validation:**
   - Username checked for uniqueness (case-insensitive)
   - Email checked for uniqueness (case-insensitive)
   - Input sanitization prevents XSS and injection attacks
   - Markdown bio sanitized using ammonia crate

5. **Rate Limiting:**
   - 5 requests per minute per IP address
   - 10 requests per minute per authenticated user
   - Stricter limits prevent account creation abuse

6. **Audit Logging:**
   - User creation logged with timestamp and creator
   - IP address logged for self-registration
   - Role assignment logged for accountability
   - Failed creation attempts logged for abuse detection

**Example Request (Admin Creating User):**

```http
POST /api/v1/users HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "username": "newuser",
  "email": "newuser@example.com",
  "password": "SecureP@ssw0rd!",
  "display_name": "New User",
  "bio": "Hello, I'm new here!",
  "role": "user",
  "preferences": {
      "theme": "dark",
      "language": "en",
      "timezone": "America/New_York"
  }
}
```

**Example Response (201 Created):**

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "username": "newuser",
  "display_name": "New User",
  "email": "newuser@example.com",
  "role": "user",
  "status": "inactive",
  "avatar_url": null,
  "bio": "Hello, I'm new here!",
  "created_at": "2026-02-06T02:15:00Z",
  "updated_at": "2026-02-06T02:15:00Z",
  "last_login_at": null,
  "preferences": {
      "theme": "dark",
      "language": "en",
      "timezone": "America/New_York",
      "email_notifications": true,
      "desktop_notifications": true,
      "custom": {}
  },
  "activity": null,
  "cached": false
}
```

**Error Responses:**

**400 Bad Request:**

```json
{
  "code": "VALIDATION_ERROR",
  "message": "Invalid request data",
  "details": {
      "errors": [
          {
              "field": "password",
              "message": "does not meet complexity requirements"
          }
      ]
  },
  "request_id": "req_pqr456stu789"
}
```

**409 Conflict:**

```json
{
  "code": "USER_EXISTS",
  "message": "Username or email already exists",
  "details": {
      "username_exists": true,
      "email_exists": false
  },
  "request_id": "req_pqr456stu789"
}
```

**403 Forbidden:**

```json
{
  "code": "AUTH_FORBIDDEN",
  "message": "Insufficient permissions to create user with role: admin",
  "details": {
      "required_role": "admin",
      "user_role": "moderator"
  },
  "request_id": "req_pqr456stu789"
}
```

**429 Too Many Requests:**

```json
{
  "code": "RATE_LIMIT_EXCEEDED",
  "message": "Account creation rate limit exceeded",
  "details": {
      "limit": 5,
      "window": "1 minute",
      "retry_after": 45
  },
  "request_id": "req_pqr456stu789"
}
```

**Performance Characteristics:**

| Metric | Target | P50 | P95 | P99 |
|--------|---------|-----|-----|-----|
| Response Time | < 100ms | 45ms | 85ms | 95ms |
| Database Queries | ≤ 3 | 2 | 3 | 3 |
| Password Hash | < 50ms | 25ms | 40ms | 48ms |

**Post-Creation Actions:**

1. **Email Verification:**
   - Verification email sent to provided address
   - Account remains inactive until verified
   - Verification token expires in 24 hours

2. **Default Preferences:**
   - Theme: system default
   - Language: browser language or en
   - Timezone: system default or UTC
   - Notifications: enabled by default

3. **Activity Logging:**
   - User creation event logged
   - Creator attribution recorded
   - IP address logged for self-registration

4. **Cache Invalidation:**
   - User list cache invalidated
   - No individual user cache (new user)

---

### 3.4. Update User

#### API-USR-004: Update User

**Element ID:** DES-USR-004
**Name:** PATCH /api/v1/users/:id
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

Updates an existing user's profile information with the provided fields. This endpoint supports partial updates, allowing modification of specific user attributes without requiring the entire user object. Only the fields present in the request body are modified; unspecified fields remain unchanged. Role and status changes require elevated permissions.

**Request:**

```rust
use axum::{Json, Path, State};
use serde::Deserialize;
use validator::Validate;

/// Request body for updating a user
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    /// New username (optional)
    #[validate(length(min = 1, max = 255))]
    #[validate(regex = "^[a-zA-Z0-9_-]+$")]
    pub username: Option<String>,
    
    /// New email address (optional, must be unique)
    #[validate(email)]
    #[validate(length(max = 254))]
    pub email: Option<String>,
    
    /// New password (optional, will be hashed)
    #[validate(length(min = 8, max = 128))]
    #[validate(custom = "validate_password_complexity")]
    pub password: Option<String>,
    
    /// New display name (optional)
    #[validate(length(max = 255))]
    pub display_name: Option<String>,
    
    /// New user biography (Markdown format, optional)
    #[validate(length(max = 1000))]
    pub bio: Option<String>,
    
    /// New user role (optional, requires admin permission)
    pub role: Option<UserRole>,
    
    /// New user status (optional, requires admin permission)
    pub status: Option<UserStatus>,
    
    /// New avatar URL (optional)
    #[validate(url)]
    pub avatar_url: Option<String>,
    
    /// User preferences to update (optional)
    pub preferences: Option<UserPreferencesUpdate>,
}

/// Partial update for user preferences
#[derive(Debug, Deserialize, Validate)]
pub struct UserPreferencesUpdate {
    /// Theme preference (optional)
    pub theme: Option<String>,
    
    /// Language preference (optional)
    pub language: Option<String>,
    
    /// Timezone preference (optional)
    pub timezone: Option<String>,
    
    /// Email notifications enabled (optional)
    pub email_notifications: Option<bool>,
    
    /// Desktop notifications enabled (optional)
    pub desktop_notifications: Option<bool>,
    
    /// Custom preferences to merge (optional)
    pub custom: Option<serde_json::Value>,
}

/// Handler function for updating a user
pub async fn update_user(
    Path(id): Path<UserId>,
    Json(req): Json<UpdateUserRequest>,
    State(auth_user): State<AuthenticatedUser>,
) -> Result<Json<UserResponse>, ApiError>;
```

**Response:**

```rust
// Returns UserResponse (see API-USR-002)
```

**Constraints:**

| Field | Type | Min | Max | Validation | Permissions |
|-------|------|-----|------|-------------|-------------|
| `username` | String | 1 | 255 | Self or Admin |
| `email` | String | 5 | 254 | Self or Admin |
| `password` | String | 8 | 128 | Self only |
| `display_name` | String | 0 | 255 | Self or Admin |
| `bio` | String | 0 | 1000 | Self or Admin |
| `role` | UserRole | N/A | N/A | Admin only |
| `status` | UserStatus | N/A | N/A | Admin only |
| `avatar_url` | String | N/A | N/A | Self or Admin |
| `preferences` | PreferencesUpdate | N/A | N/A | Self or Admin |

**Dependencies:**

- REQ-SRV-004: User Update
- REQ-SRV-081: RBAC Enforcement
- [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md): Rust type safety
- [TACHYON-TMA-V1.0](../.adrs/ Security requirements

**Rationale:**

Partial update capability enables efficient modification of user profiles without requiring full object replacement. This reduces bandwidth, simplifies client implementations, and prevents accidental modification of unintended fields. Elevated permission requirements for sensitive fields (role, status) protect against privilege escalation attacks.

**Security Considerations:**

1. **Authentication Required:** All requests must include a valid JWT token.

2. **Authorization Enforcement:**
   - Users can update their own profile (excluding role and status)
   - Admins can update any user's profile including role and status
   - Moderators can update display name, bio, and avatar of regular users
   - Role and status changes require admin permissions

3. **Password Security:**
   - New passwords hashed using Argon2id
   - Old password not required (assumes authenticated user)
   - Password complexity enforced on update
   - Password change logged for audit trail

4. **Data Validation:**
   - Username checked for uniqueness (excluding current user)
   - Email checked for uniqueness (excluding current user)
   - URL validation for avatar_url
   - Markdown bio sanitized using ammonia crate

5. **Audit Logging:**
   - User update logged with timestamp and modifier
   - Changed fields logged for accountability
   - Role and status changes trigger security alerts
   - Failed update attempts logged for abuse detection

6. **Rate Limiting:**
   - 50 requests per minute per authenticated user
   - Stricter limits for role and status changes

7. **Cache Invalidation:**
   - User profile cache invalidated immediately
   - User list cache invalidated on role or status change
   - Activity stats cache invalidated on profile update

**Example Request (Self Update):**

```http
PATCH /api/v1/users/770e8400-e29b-41d4-a716-446655440002 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "display_name": "Updated Name",
  "bio": "Updated bio with Markdown support.",
  "preferences": {
      "theme": "light",
      "language": "fr"
  }
}
```

**Example Request (Admin Role Change):**

```http
PATCH /api/v1/users/770e8400-e29b-41d4-a716-446655440002 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "role": "moderator",
  "status": "active"
}
```

**Example Response (200 OK):**

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "username": "newuser",
  "display_name": "Updated Name",
  "email": "newuser@example.com",
  "role": "user",
  "status": "active",
  "avatar_url": null,
  "bio": "Updated bio with Markdown support.",
  "created_at": "2026-02-06T02:15:00Z",
  "updated_at": "2026-02-06T02:20:15Z",
  "last_login_at": null,
  "preferences": {
      "theme": "light",
      "language": "fr",
      "timezone": "America/New_York",
      "email_notifications": true,
      "desktop_notifications": true,
      "custom": {}
  },
  "activity": null,
  "cached": false
}
```

**Error Responses:**

**400 Bad Request:**

```json
{
  "code": "VALIDATION_ERROR",
  "message": "Invalid request data",
  "details": {
      "errors": [
          {
              "field": "avatar_url",
              "message": "must be a valid URL"
          }
      ]
  },
  "request_id": "req_vwx234yzu567"
}
```

**403 Forbidden:**

```json
{
  "code": "AUTH_FORBIDDEN",
  "message": "Insufficient permissions to update user role",
  "details": {
      "required_role": "admin",
      "user_role": "moderator"
  },
  "request_id": "req_vwx234yzu567"
}
```

**404 Not Found:**

```json
{
  "code": "USER_NOT_FOUND",
  "message": "User not found",
  "details": {
      "user_id": "770e8400-e29b-41d4-a716-446655440002"
  },
  "request_id": "req_vwx234yzu567"
}
```

**409 Conflict:**

```json
{
  "code": "USER_EXISTS",
  "message": "Username or email already exists",
  "details": {
      "username_exists": true,
      "email_exists": false
  },
  "request_id": "req_vwx234yzu567"
}
```

**Performance Characteristics:**

| Metric | Target | P50 | P95 | P99 |
|--------|---------|-----|-----|-----|
| Response Time | < 75ms | 35ms | 65ms | 72ms |
| Database Queries | ≤ 2 | 1 | 2 | 2 |
| Cache Invalidation | < 10ms | 3ms | 8ms | 9ms |

**Update Behavior:**

1. **Partial Updates:**
   - Only specified fields are modified
   - Unspecified fields remain unchanged
   - Null values clear optional fields (not applicable for required fields)

2. **Preference Updates:**
   - Custom preferences merged with existing preferences
   - Top-level preferences replace entire preference object
   - Invalid preference values rejected with validation error

3. **Role Changes:**
   - Role changes logged as security events
   - Role changes invalidate all user permissions
   - Role changes trigger permission recalculation

4. **Status Changes:**
   - Status to suspended prevents login
   - Status to deleted marks account for removal
   - Status changes trigger appropriate notifications

---

### 3.5. Delete User

#### API-USR-005: Delete User

**Element ID:** DES-USR-005
**Name:** DELETE /api/v1/users/:id
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

Deletes a user account by their unique identifier. This endpoint performs a soft delete by default, marking the user as deleted while preserving their data for audit purposes. Hard delete (permanent data removal) is available through a query parameter and requires elevated permissions. Deleted users cannot authenticate and their access to system resources is immediately revoked.

**Request:**

```rust
use axum::{extract::Query, Path, State};
use serde::Deserialize;

/// Query parameters for delete operation
#[derive(Debug, Deserialize)]
pub struct DeleteUserQuery {
    /// Perform hard delete (permanent removal)
    /// Requires admin permission
    #[serde(default)]
    pub hard: bool,
    
    /// Reason for deletion (optional, required for audit)
    pub reason: Option<String>,
}

/// Handler function for deleting a user
pub async fn delete_user(
    Path(id): Path<UserId>,
    Query(params): Query<DeleteUserQuery>,
    State(auth_user): State<AuthenticatedUser>,
) -> Result<StatusCode, ApiError>;
```

**Response:**

```rust
// Returns 204 No Content on success
```

**Constraints:**

| Parameter | Type | Default | Validation | Permissions |
|-----------|------|---------|-------------|-------------|
| `hard` | boolean | false | Boolean | Admin only |
| `reason` | String | N/A | Max 500 chars | Optional |

**Dependencies:**

- REQ-SRV-005: User Deletion
- REQ-SRV-081: RBAC Enforcement
- [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md): Rust type safety
- [TACHYON-TMA-V1.0](../.adrs/ Security requirements

**Rationale:**

User deletion enables account removal and data management. Soft delete by default preserves audit trail and enables potential recovery. Hard delete option allows administrators to permanently remove user data when required for privacy or compliance reasons. The reason parameter provides audit context for deletion decisions.

**Security Considerations:**

1. **Authentication Required:** All requests must include a valid JWT token.

2. **Authorization Enforcement:**
   - Users can delete their own account
   - Admins can delete any user account
   - Hard delete requires admin permission
   - Self-deletion cannot be prevented (user choice)

3. **Delete Behavior:**

| Delete Type | Behavior | Data Retention | Recovery |
|------------|---------|----------------|----------|
| Soft Delete | Status set to deleted | All data preserved | Possible via admin action |
| Hard Delete | Permanent removal | Only audit logs retained | Not possible |

4. **Cascade Effects:**
   - User sessions immediately invalidated
   - User permissions revoked
   - User documents reassigned or archived based on policy
   - User activity logs preserved for audit

5. **Audit Logging:**
   - User deletion logged with timestamp and deleter
   - Reason logged when provided
   - Hard vs soft delete logged
   - Self-deletion vs admin deletion logged
   - IP address logged for security

6. **Rate Limiting:**
   - 10 requests per minute per authenticated user
   - Stricter limits for hard delete operations

7. **Data Protection:**
   - Soft delete preserves data for legal requirements
   - Hard delete securely wipes sensitive data
   - Personal data anonymized after retention period

**Example Request (Soft Delete):**

```http
DELETE /api/v1/users/770e8400-e29b-41d4-a716-446655440002?reason=Account%20no%20longer%20needed HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Example Request (Hard Delete - Admin):**

```http
DELETE /api/v1/users/770e8400-e29b-41d4-a716-446655440002?hard=true&reason=GDPR%20data%20removal%20request HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Example Response (204 No Content):**

```http
HTTP/2 204 No Content
```

**Error Responses:**

**401 Unauthorized:**

```json
{
  "code": "AUTH_MISSING",
  "message": "Authentication token is required",
  "details": null,
  "request_id": "req_rst890uvw123"
}
```

**403 Forbidden:**

```json
{
  "code": "AUTH_FORBIDDEN",
  "message": "Insufficient permissions to delete this user",
  "details": {
      "required_role": "admin",
      "user_role": "moderator"
  },
  "request_id": "req_rst890uvw123"
}
```

**404 Not Found:**

```json
{
  "code": "USER_NOT_FOUND",
  "message": "User not found",
  "details": {
      "user_id": "770e8400-e29b-41d4-a716-446655440002"
  },
  "request_id": "req_rst890uvw123"
}
```

**409 Conflict:**

```json
{
  "code": "USER_CONFLICT",
  "message": "Cannot delete user with active documents",
  "details": {
      "active_documents": 15,
      "resolution": "Reassign or delete documents first"
  },
  "request_id": "req_rst890uvw123"
}
```

**Performance Characteristics:**

| Metric | Target | P50 | P95 | P99 |
|--------|---------|-----|-----|-----|
| Response Time | < 50ms | 20ms | 40ms | 48ms |
| Database Queries | ≤ 2 | 1 | 2 | 2 |
| Cache Invalidation | < 15ms | 5ms | 12ms | 14ms |

**Post-Deletion Actions:**

1. **Session Invalidation:**
   - All active sessions immediately revoked
   - JWT tokens added to blacklist
   - WebSocket connections terminated

2. **Permission Revocation:**
   - All user permissions removed
   - Role assignments deleted
   - Access control entries removed

3. **Data Handling:**

| Data Type | Soft Delete | Hard Delete |
|-----------|------------|------------|
| User Profile | Preserved | Deleted |
| Authentication Data | Preserved | Deleted |
| Activity Logs | Preserved | Preserved |
| Documents | Reassigned/Archived | Deleted |
| Preferences | Preserved | Deleted |

4. **Cache Invalidation:**
   - User profile cache invalidated
   - User list cache invalidated
   - Permission cache invalidated
   - Activity stats cache invalidated

5. **Notifications:**
   - Deletion confirmation sent to user email (if soft delete)
   - Admin notification sent for hard deletes
   - Document ownership changes notified to affected users

---

## 4. USER ACTIVITY API

### 4.1. Get User Activity

#### API-USR-006: Get User Activity

**Element ID:** DES-USR-006
**Name:** GET /api/v1/users/:id/activity
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

Retrieves activity logs and statistics for a specific user. This endpoint provides comprehensive activity tracking including document operations, login history, and system interactions. Activity data is filtered based on the requester's permissions, with full details available to admins and the user themselves.

**Request:**

```rust
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use validator::Validate;
use chrono::{DateTime, Utc};

/// Query parameters for activity retrieval
#[derive(Debug, Deserialize, Validate)]
pub struct GetUserActivityQuery {
    /// Activity type filter (all, login, document, comment, system)
    pub activity_type: Option<String>,
    
    /// Start date for filtering (ISO 8601 format)
    pub start_date: Option<DateTime<Utc>>,
    
    /// End date for filtering (ISO 8601 format)
    pub end_date: Option<DateTime<Utc>>,
    
    /// Pagination offset
    #[validate(range(min = 0))]
    pub offset: Option<usize>,
    
    /// Page size (default: 50, max: 200)
    #[validate(range(min = 1, max = 200))]
    pub limit: Option<usize>,
}

/// Handler function for retrieving user activity
pub async fn get_user_activity(
    Path(id): Path<UserId>,
    Query(params): Query<GetUserActivityQuery>,
    State(auth_user): State<AuthenticatedUser>,
) -> Result<Json<UserActivityResponse>, ApiError>;
```

**Response:**

```rust
use serde::Serialize;

/// User activity response with paginated activity logs
#[derive(Debug, Serialize)]
pub struct UserActivityResponse {
    /// User identifier
    pub user_id: UserId,
    
    /// Activity log entries
    pub activities: Vec<ActivityEntry>,
    
    /// Total activity count
    pub total: usize,
    
    /// Current pagination offset
    pub offset: usize,
    
    /// Current page size
    pub limit: usize,
    
    /// Has more pages indicator
    pub has_more: bool,
    
    /// Activity statistics summary
    pub statistics: ActivityStatistics,
}

/// Individual activity entry
#[derive(Debug, Serialize)]
pub struct ActivityEntry {
    /// Unique activity identifier
    pub id: ActivityId,
    
    /// Activity type
    pub activity_type: ActivityType,
    
    /// Activity description
    pub description: String,
    
    /// Activity timestamp
    pub timestamp: DateTime<Utc>,
    
    /// IP address (included for admins and self)
    pub ip_address: Option<String>,
    
    /// User agent (included for admins and self)
    pub user_agent: Option<String>,
    
    /// Related resource ID (document, comment, etc.)
    pub resource_id: Option<String>,
    
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Activity type enumeration
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityType {
    /// User login
    Login,
    /// User logout
    Logout,
    /// Document created
    DocumentCreated,
    /// Document updated
    DocumentUpdated,
    /// Document deleted
    DocumentDeleted,
    /// Comment created
    CommentCreated,
    /// Comment updated
    CommentUpdated,
    /// Comment deleted
    CommentDeleted,
    /// User profile updated
    ProfileUpdated,
    /// Password changed
    PasswordChanged,
    /// Role changed
    RoleChanged,
    /// Status changed
    StatusChanged,
    /// System action
    SystemAction,
}

/// Activity statistics summary
#[derive(Debug, Serialize)]
pub struct ActivityStatistics {
    /// Total login count
    pub login_count: usize,
    
    /// Total document operations
    pub document_operations: usize,
    
    /// Total comment operations
    pub comment_operations: usize,
    
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    
    /// First activity timestamp
    pub first_activity: DateTime<Utc>,
    
    /// Most active day
    pub most_active_day: String,
    
    /// Activity by type breakdown
    pub by_type: serde_json::Value,
}
```

**Constraints:**

| Parameter | Type | Min | Max | Default | Validation |
|-----------|------|-----|------|-------------|
| `activity_type` | String | N/A | N/A | all | Must be valid type |
| `start_date` | DateTime | N/A | N/A | N/A | ISO 8601 format |
| `end_date` | DateTime | N/A | N/A | N/A | ISO 8601 format |
| `offset` | usize | 0 | 2^64-1 | 0 | Non-negative |
| `limit` | usize | 1 | 200 | 50 | Bounded pagination |

**Dependencies:**

- REQ-SRV-006: User Activity Tracking
- REQ-SRV-081: RBAC Enforcement
- [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md): Rust type safety
- [TACHYON-TMA-V1.0](../.adrs/ Security requirements

**Rationale:**

User activity tracking enables audit compliance, security monitoring, and user engagement analytics. Detailed activity logs support forensic analysis of security incidents and provide accountability for user actions. Filtering capabilities allow administrators to focus on specific activity types or time periods.

**Security Considerations:**

1. **Authentication Required:** All requests must include a valid JWT token.

2. **Authorization Enforcement:**
   - Users can view their own full activity
   - Users can view limited activity of other users (no IP, no user agent)
   - Admins can view full activity of any user
   - Moderators can view activity of regular users and guests

3. **Data Filtering Based on Requester:**

| Requester Role | Target User | IP Address | User Agent | Full Details |
|----------------|-------------|------------|------------|--------------|
| Admin | Any | Yes | Yes | Yes |
| Moderator | User/Guest | No | No | Yes |
| User | Self | Yes | Yes | Yes |
| User | Other | No | No | No |

4. **Privacy Protection:**
   - IP addresses and user agents excluded for non-admin requests
   - Activity logs retained for configurable retention period
   - Sensitive actions (password changes) logged with additional detail
   - Data anonymized after retention period expires

5. **Audit Logging:**
   - Activity view logged with timestamp and viewer
   - Filter parameters logged for audit trail
   - Bulk export logged for compliance
   - Failed access attempts logged

6. **Rate Limiting:**
   - 100 requests per minute per authenticated user
   - Stricter limits for bulk exports

**Example Request:**

```http
GET /api/v1/users/770e8400-e29b-41d4-a716-446655440002/activity?activity_type=login&start_date=2026-02-01T00:00:00Z&end_date=2026-02-06T23:59:59Z&offset=0&limit=50 HTTP/2
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

**Example Response (200 OK):**

```json
{
  "user_id": "770e8400-e29b-41d4-a716-446655440002",
  "activities": [
      {
          "id": "act_123456789",
          "activity_type": "login",
          "description": "User logged in from 192.168.1.100",
          "timestamp": "2026-02-05T09:15:22Z",
          "ip_address": "192.168.1.100",
          "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
          "resource_id": null,
          "metadata": null
      },
    {
          "id": "act_123456790",
          "activity_type": "document_created",
          "description": "Created document: Introduction to Tachyon",
          "timestamp": "2026-02-05T10:30:45Z",
          "ip_address": "192.168.1.100",
          "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
          "resource_id": "doc_987654321",
          "metadata": {
              "document_title": "Introduction to Tachyon"
          }
      }
  ],
  "total": 156,
  "offset": 0,
  "limit": 50,
  "has_more": true,
  "statistics": {
      "login_count": 42,
      "document_operations": 89,
      "comment_operations": 25,
      "last_activity": "2026-02-05T10:30:45Z",
      "first_activity": "2026-01-15T10:30:00Z",
      "most_active_day": "2026-02-05",
      "by_type": {
          "login": 42,
          "document_created": 45,
          "document_updated": 38,
          "document_deleted": 6,
          "comment_created": 25,
          "profile_updated": 8,
          "password_changed": 2
      }
  }
}
```

**Error Responses:**

**401 Unauthorized:**

```json
{
  "code": "AUTH_MISSING",
  "message": "Authentication token is required",
  "details": null,
  "request_id": "req_opq567rst890"
}
```

**403 Forbidden:**

```json
{
  "code": "AUTH_FORBIDDEN",
  "message": "Insufficient permissions to view this user's activity",
  "details": {
      "required_role": "admin",
      "user_role": "user"
  },
  "request_id": "req_opq567rst890"
}
```

**404 Not Found:**

```json
{
  "code": "USER_NOT_FOUND",
  "message": "User not found",
  "details": {
      "user_id": "770e8400-e29b-41d4-a716-446655440002"
  },
  "request_id": "req_opq567rst890"
}
```

**Performance Characteristics:**

| Metric | Target | P50 | P95 | P99 |
|--------|---------|-----|-----|-----|
| Response Time | < 100ms | 45ms | 85ms | 95ms |
| Database Queries | ≤ 3 | 2 | 3 | 3 |
| Cache Hit Rate | > 75% | 78% | 74% | 70% |

**Caching Strategy:**

- **In-Memory Cache:** Activity statistics cached for 10 minutes
- **Cache Key:** `activity:{user_id}:stats`
- **Cache Key:** `activity:{user_id}:{offset}:{limit}:{filters}` for paginated results
- **Invalidation:** Cache invalidated on new activity
- **HTTP Cache:** Cache-Control header with max-age=600 for statistics
- **Privacy:** No caching for IP addresses and user agents


## 6. USER PERFORMANCE

### 6.1. Latency Requirements

#### API-USR-011: Latency Requirements

**Element ID:** DES-USR-011
**Name:** Latency Requirements
**Type:** Performance Specification
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

The User API is designed to meet strict latency requirements ensuring responsive user experience. Performance targets are defined for each endpoint type with specific percentiles for different load conditions. Latency monitoring and alerting ensure compliance with service level agreements.

**Latency Targets:**

| Endpoint Type | P50 Target | P95 Target | P99 Target | Rationale |
|--------------|------------|------------|------------|-----------|
| Read Operations (List, Get) | < 20ms | < 35ms | < 50ms | Fast user data retrieval |
| Write Operations (Create, Update) | < 50ms | < 75ms | < 100ms | Acceptable write latency |
| Delete Operations | < 30ms | < 50ms | < 75ms | Quick account removal |
| Authentication (Login) | < 100ms | < 150ms | < 200ms | Secure auth processing |
| Activity Queries | < 75ms | < 125ms | < 175ms | Complex aggregation queries |

**Performance Optimization Techniques:**

1. **Database Query Optimization:**
   - Indexed queries on email and username fields
   - Query projection to retrieve only required fields
   - Connection pooling for efficient resource reuse
   - Prepared statements with parameter binding

2. **Caching Strategy:**
   - In-memory cache for frequently accessed user data
   - HTTP cache headers for client-side caching
   - CDN caching for static assets (avatars)
   - Cache invalidation on data mutations

3. **Async Architecture:**
   - Tokio work-stealing scheduler for optimal CPU utilization
   - Non-blocking I/O for database operations
   - Concurrent request handling without thread blocking

4. **Response Optimization:**
   - JSON serialization with efficient serde implementation
   - Compression of responses (gzip, deflate)
   - Minimal response payloads (exclude unnecessary fields)
   - HTTP/2 multiplexing for concurrent requests

**Monitoring and Alerting:**

```rust
use tracing::{info, warn, error};

/// Performance monitoring middleware
pub async fn performance_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    // Process request
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    
    // Log performance metrics
    if duration.as_millis() > 100 {
        warn!(
            request_id = %request.id(),
            endpoint = %request.uri().path(),
            duration_ms = duration.as_millis(),
            "High latency detected: {}ms",
            duration.as_millis()
        );
    } else if duration.as_millis() > 50 {
        info!(
            request_id = %request.id(),
            endpoint = %request.uri().path(),
            duration_ms = duration.as_millis(),
            "Request completed: {}ms",
            duration.as_millis()
        );
    }
    
    // Add performance headers
    let mut response = response;
    response.headers_mut().insert(
        "X-Response-Time",
        duration.as_millis().to_string()
    );
    
    Ok(response)
}
```

**Dependencies:**

- REQ-PERF-001: Latency Requirements
- REQ-PERF-002: Performance Monitoring
- [TACHYON-ADR-007-V1.0](../.adrs/adr-007-thread-safety-strategy.md): Tokio async runtime

**Rationale:**

Strict latency requirements ensure responsive user experience and meet service level agreements. Performance optimization techniques reduce server load and improve scalability. Monitoring and alerting enable proactive performance issue detection and resolution.

**Performance Considerations:**

1. **Load Testing:**
   - Load testing with simulated user traffic
   - Performance testing under various load conditions
   - Stress testing to identify breaking points
   - Capacity planning based on performance metrics

2. **Database Performance:**
   - Query execution time monitoring
   - Slow query identification and optimization
   - Index maintenance and statistics
   - Connection pool utilization tracking

3. **Cache Performance:**
   - Cache hit rate monitoring
   - Cache size and memory usage tracking
   - Cache invalidation latency measurement
   - Cache key distribution analysis

4. **Network Performance:**
   - HTTP/2 connection multiplexing efficiency
   - TLS handshake overhead measurement
   - Bandwidth usage monitoring
   - Compression ratio tracking

---

### 6.2. Caching Strategies

#### API-USR-012: Caching Strategies

**Element ID:** DES-USR-012
**Name:** Caching Strategies
**Type:** Performance Specification
**Language:** Rust (Axum)
**Status:** Proposed

**Description:**

The User API implements multi-level caching to optimize performance and reduce database load. Caching strategy balances data freshness with performance gains, ensuring users receive timely data while minimizing server resource consumption. Cache invalidation ensures data consistency across distributed components.

**Cache Architecture:**

```mermaid
graph TB
    subgraph "Client Layer"
        Browser[Browser Cache]
        Desktop[Desktop Cache]
    end
    
    subgraph "CDN Layer"
        CDN[CDN Cache]
    end
    
    subgraph "Application Layer"
        API[API Cache]
        Server[Server Cache]
        Database[Database]
    end
    
    Browser -->|HTTP Cache|
    Desktop -->|HTTP Cache|
    CDN -->|CDN Cache|
    
    HTTP Cache -->|API Cache|
    CDN Cache -->|API Cache|
    
    API Cache -->|Server Cache|
    Server Cache -->|Database
```

**Cache Levels:**

| Cache Level | Technology | Scope | TTL | Invalidation Strategy |
|------------|-----------|-------|-----|---------------------|
| Browser Cache | HTTP Cache-Control | 60 seconds | Time-based |
| Desktop Cache | In-memory LRU | 300 seconds | LRU eviction |
| CDN Cache | CDN edge locations | 1 hour | Version-based |
| API Cache | Redis/memcached | 5 minutes | Event-based |
| Server Cache | In-memory HashMap | 10 minutes | Write-through |

**Cache Key Design:**

```rust
use std::fmt;

/// Cache key generator
pub struct CacheKey {
    /// User profile data
    pub fn user_profile(user_id: &UserId) -> String {
        format!("user:profile:{}", user_id)
    }
    
    /// User list with filters
    pub fn user_list(offset: usize, limit: usize, filters: &str) -> String {
        format!("user:list:{}:{}:{}", offset, limit, filters)
    }
    
    /// User activity statistics
    pub fn user_activity_stats(user_id: &UserId) -> String {
        format!("user:activity:stats:{}", user_id)
    }
    
    /// User permissions
    pub fn user_permissions(user_id: &UserId) -> String {
        format!("user:permissions:{}", user_id)
    }
}
```

**Cache Invalidation Triggers:**

| Event | Cache Keys Affected | Invalidation Method |
|-------|---------------------|-------------------|
| User profile updated | `user:profile:{id}` | Immediate deletion |
| User role changed | `user:profile:{id}`, `user:permissions:{id}` | Immediate deletion |
| User status changed | `user:list:*` | Pattern-based deletion |
| User created | `user:list:*` | Pattern-based deletion |
| User deleted | `user:list:*` | Pattern-based deletion |
| User activity logged | `user:activity:stats:{id}` | Immediate deletion |
| Session invalidated | All session keys for user | Redis pattern deletion |

**Cache Freshness Policies:**

| Data Type | Max TTL | Stale Threshold | Refresh Strategy |
|-----------|---------|----------------|----------------|
| User Profile | 5 minutes | 10 minutes | Background refresh |
| User List | 1 minute | 5 minutes | Pagination refresh |
| Activity Stats | 10 minutes | 30 minutes | Lazy loading |
| Permissions | 5 minutes | 15 minutes | On role change |
| Public Data | 10 minutes | 30 minutes | Longer TTL for privacy |

**Dependencies:**

- REQ-PERF-003: Caching Strategy
- REQ-PERF-004: Cache Invalidation
- [TACHYON-ADR-007-V1.0](../.adrs/adr-007-thread-safety-strategy.md): Tokio async runtime

**Rationale:**

Multi-level caching reduces database load and improves response times. Appropriate TTL values balance data freshness with performance. Cache invalidation ensures data consistency and prevents stale data delivery to users. The caching strategy supports both read-heavy and write-heavy workloads.

**Cache Implementation Example:**

```rust
use axum::extract::State;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory cache implementation
pub struct UserCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl UserCache {
    /// Get user profile from cache
    pub async fn get_profile(&self, user_id: &UserId) -> Option<CacheEntry> {
        let cache = self.cache.read().await;
        let key = CacheKey::user_profile(user_id);
        
        cache.get(&key).cloned()
    }
    
    /// Update user profile in cache
    pub async fn set_profile(&self, user_id: &UserId, profile: CacheEntry) {
        let mut cache = self.cache.write().await;
        let key = CacheKey::user_profile(user_id);
        
        cache.insert(key, profile);
    }
    
    /// Invalidate user cache
    pub async fn invalidate_user(&self, user_id: &UserId) {
        let mut cache = self.cache.write().await;
        
        // Remove all user-related keys
        cache.remove(&CacheKey::user_profile(user_id));
        cache.remove(&CacheKey::user_activity_stats(user_id));
        cache.remove(&CacheKey::user_permissions(user_id));
    }
}
```

**Cache Performance Metrics:**

| Metric | Target | P50 | P95 | P99 |
|--------|---------|-----|-----|-----|
| Cache Hit Rate | > 80% | 85% | 78% | 75% |
| Cache Miss Latency | < 5ms | 3ms | 8ms | 12ms |
| Cache Write Latency | < 2ms | 1ms | 3ms | 5ms |
| Memory Usage | < 100MB | 50MB | 75MB | 90MB |

## 7. REFERENCES

### 7.1. Document References

This section provides comprehensive references to all documents, standards, and architectural decisions referenced throughout the User API specification.

#### 7.1.1. Standards Documents

| Document ID | Title | Location | Relevance |
|-------------|-------|----------|-----------|
| [TACHYON-STD-V1.0](../.adrs/ | Coding and Documentation Standards | API documentation standards |
| IEEE 1063-2001 | [TACHYON-STD-V1.0](../.adrs/ | User Documentation Standards | User-facing documentation |

#### 7.1.2. Architectural Decision Records

| ADR ID | Title | Location | Relevance |
|----------|-------|----------|-----------|
| [TACHYON-ADR-001-V1.0](../.adrs/adr-001-three-tier-jit-compilation.md) | Rust as Primary Language | Type system and memory safety |
| [TACHYON-ADR-003-V1.0](../.adrs/adr-003-lru-cache-target.md) | Axum for HTTP/2 Server | Web framework selection |
| [TACHYON-ADR-007-V1.0](../.adrs/adr-007-thread-safety-strategy.md) | Tokio for Async Runtime | Async runtime selection |

#### 7.1.3. Security Documents

| Document ID | Title | Location | Relevance |
|-------------|-------|----------|-----------|
| [TACHYON-TMA-V1.0](../.adrs/ | Threat Model Analysis | Security threat analysis |
| [TACHYON-TMA-V1.0](../.adrs/ | Security Requirements | Security requirements |

#### 7.1.4. Requirements Documents

| Requirement ID | Title | Location | Relevance |
|--------------|-------|----------|-----------|
| REQ-SRV-001 | User Management | [../.adrs/ | User CRUD operations |
| REQ-SRV-081 | RBAC Enforcement | [../.adrs/ | Role-based access control |
| REQ-SEC-001 | Password Security | [../.adrs/ | Password hashing and complexity |
| REQ-SEC-002 | Token Security | [../.adrs/ | JWT token security |
| REQ-SEC-003 | RBAC Implementation | [../.adrs/ | Role-based access control |
| REQ-PERF-001 | Latency Requirements | [../.adrs/ | Performance targets |
| REQ-PERF-002 | Performance Monitoring | [../.adrs/ | Performance monitoring |
| REQ-PERF-003 | Caching Strategy | [../.adrs/ | Caching requirements |

#### 7.1.5. Design Documents

| Design ID | Title | Location | Relevance |
|-----------|-------|----------|-----------|
| [TACHYON-DES-API-V1.0](../.adrs/ | API Interfaces Design | REST API design patterns |
| [TACHYON-DES-SRV-V1.0](../.adrs/ | Server Design | Server component design |
| [TACHYON-DES-DD-001](../.adrs/ | Data Models Design | User data structures |

#### 7.1.6. External References

| Reference | Description | URL |
|----------|-------------|-------|
| Rust Book | https://doc.rust-lang.org/book/ | Rust programming language |
| Axum Documentation | https://docs.rs/axum/0.7.x/axum/index.html | Axum web framework |
| Tokio Documentation | https://tokio.rs/ | Tokio async runtime |
| Serde Documentation | https://serde.rs/ | Serialization framework |
| JWT Specification | https://tools.ietf.org/html/rfc7519 | JSON Web Token |
| OAuth 2.0 | https://oauth.net/2/ | OAuth 2.0 framework |
| Argon2 Documentation | https://github.com/P-H-C/phc-argon2 | Password hashing |
| HTTP/2 Specification | https://httpwg.org/.adrs/ | HTTP/2 protocol |

---

**Document Change History:**

| Version | Date | Author | Changes |
|---------|------|-------|---------|
| V1.0 | 2026-02-06 | Technical Writer | Initial document creation |

---

**End of Document**
