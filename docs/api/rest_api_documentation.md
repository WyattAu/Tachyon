# REST API Documentation

**Document ID:** TACHYON-API-002-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** API Specification
**Compliance Level:** OpenAPI 3.0.3, RFC 7231, ISO/IEC 26514:2021

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [REST API Framework](#2-rest-api-framework)
3. [Authentication Endpoints](#3-authentication-endpoints)
4. [Document Endpoints](#4-document-endpoints)
5. [Workspace Endpoints](#5-workspace-endpoints)
6. [Git Endpoints](#6-git-endpoints)
7. [Plugin Endpoints](#7-plugin-endpoints)
8. [Configuration Endpoints](#8-configuration-endpoints)
9. [Event Endpoints](#9-event-endpoints)
10. [Monitoring Endpoints](#10-monitoring-endpoints)
11. [Error Responses](#11-error-responses)
12. [Rate Limiting](#12-rate-limiting)
13. [Pagination](#13-pagination)
14. [Filtering and Sorting](#14-filtering-and-sorting)
15. [References](#15-references)

---

## 1. INTRODUCTION

### 1.1. Purpose and Scope

This document provides comprehensive API documentation for the Tachyon REST API, which enables programmatic access to the Tachyon toolchain's server component. The API follows RESTful principles and OpenAPI 3.0.3 specification standards, providing a standardized interface for client applications including the Tauri desktop application and Leptos web frontend.

The Tachyon REST API serves as the primary communication layer for:
- Document management operations (CRUD, search, indexing)
- Workspace and project management
- Git repository operations
- Plugin system integration
- Configuration management
- Real-time event streaming
- System monitoring and health checks

### 1.2. Document Dependencies

This document depends on the following specifications:
- [TACHYON-STD-V1.0](../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-ADR-001-V1.0](../.specs/02_adrs/001_rust_as_primary_language.md) - Rust as Primary Language
- [TACHYON-ADR-003-V1.0](../.specs/02_adrs/003_axum_for_http2_server.md) - Axum for HTTP/2 Server
- [TACHYON-ADR-010-V1.0](../.specs/02_adrs/010_security_architecture.md) - Security Architecture

### 1.3. API Versioning

The Tachyon REST API follows semantic versioning for endpoint paths:

**Current Version:** v1

**Versioning Strategy:**
- Major version changes indicate breaking changes in API contract
- Minor version changes indicate backward-compatible additions
- Patch version changes indicate backward-compatible bug fixes

**Version Format:** `/api/v{major}/{endpoint}`

**Example:** `GET /api/v1/documents/{id}`

### 1.4. Base URL

The base URL for the Tachyon REST API depends on the deployment environment:

| Environment | Base URL |
|--------------|-----------|
| **Development** | `http://localhost:8080/api/v1` |
| **Staging** | `https://staging.tachyon.dev/api/v1` |
| **Production** | `https://api.tachyon.dev/api/v1` |

### 1.5. Supported HTTP Methods

The Tachyon REST API supports the following HTTP methods:

| Method | Purpose | Idempotent | Safe |
|--------|---------|-------------|-------|
| **GET** | Retrieve resources | Yes | Yes |
| **POST** | Create resources | No | No |
| **PUT** | Update/Replace resources | Yes | No |
| **PATCH** | Partially update resources | No | No |
| **DELETE** | Delete resources | Yes | No |
| **HEAD** | Retrieve headers only | Yes | Yes |
| **OPTIONS** | Retrieve allowed methods | Yes | Yes |

---

## 2. REST API FRAMEWORK

### 2.1. Request Format

#### 2.1.1. Content Types

The Tachyon REST API supports the following content types for requests:

| Content Type | Purpose | Endpoints |
|--------------|---------|-----------|
| `application/json` | JSON payloads | All endpoints |
| `application/x-www-form-urlencoded` | Form data | Authentication endpoints |
| `multipart/form-data` | File uploads | Document endpoints |
| `text/markdown` | Markdown content | Document endpoints |

**Default Content Type:** `application/json`

#### 2.1.2. Character Encoding

All requests must use UTF-8 character encoding:

```
Content-Type: application/json; charset=utf-8
```

#### 2.1.3. Request Headers

Standard headers required for all requests:

| Header | Required | Description | Example |
|--------|-----------|-------------|---------|
| `Content-Type` | Yes (for body) | Media type of request body | `application/json` |
| `Accept` | Yes | Media types accepted in response | `application/json` |
| `Authorization` | Conditional | Bearer token for authenticated requests | `Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...` |
| `User-Agent` | Recommended | Client identifier | `Tachyon-Desktop/1.0.0` |
| `X-Request-ID` | Recommended | Unique request identifier for tracing | `550e8400-e29b-41d4-a716-446655440000` |
| `X-Client-Version` | Recommended | Client version for compatibility | `1.0.0` |

### 2.2. Response Format

#### 2.2.1. Content Types

The Tachyon REST API supports the following content types for responses:

| Content Type | Purpose | Endpoints |
|--------------|---------|-----------|
| `application/json` | JSON responses | All endpoints |
| `text/markdown` | Markdown content | Document retrieval endpoints |
| `text/plain` | Plain text responses | Health check endpoints |

#### 2.2.2. Response Headers

Standard headers included in all responses:

| Header | Description | Example |
|--------|-------------|---------|
| `Content-Type` | Media type of response body | `application/json` |
| `Content-Length` | Length of response body in bytes | `1234` |
| `Date` | Response timestamp (RFC 7231) | `Wed, 07 Feb 2026 14:37:18 GMT` |
| `X-Request-ID` | Echo of request identifier | `550e8400-e29b-41d4-a716-446655440000` |
| `X-RateLimit-Limit` | Rate limit quota | `1000` |
| `X-RateLimit-Remaining` | Remaining requests in quota | `999` |
| `X-RateLimit-Reset` | Unix timestamp of quota reset | `1738943838` |

#### 2.2.3. Response Structure

All JSON responses follow a consistent structure:

**Success Response:**
```json
{
  "data": {
    // Response data specific to endpoint
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**Error Response:**
```json
{
  "error": {
    "code": "DOCUMENT_NOT_FOUND",
    "message": "Document with ID '550e8400-e29b-41d4-a716-446655440000' not found",
    "details": {
      "document_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

### 2.3. HTTP Status Codes

The Tachyon REST API uses standard HTTP status codes:

| Code | Category | Description |
|------|----------|-------------|
| **2xx** | Success | Request was successfully received, understood, and accepted |
| **4xx** | Client Error | Request contains bad syntax or cannot be fulfilled |
| **5xx** | Server Error | Server failed to fulfill a valid request |

**Common Status Codes:**

| Code | Name | Usage |
|------|------|-------|
| 200 | OK | Successful GET, PUT, PATCH, DELETE |
| 201 | Created | Successful POST creating a resource |
| 204 | No Content | Successful DELETE or PUT with no response body |
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Valid authentication but insufficient permissions |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource conflict (duplicate, version mismatch) |
| 422 | Unprocessable Entity | Semantically invalid request |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Unexpected server error |
| 503 | Service Unavailable | Server temporarily unavailable |

### 2.4. Data Types

#### 2.4.1. Primitive Types

| Type | Format | Example |
|------|---------|---------|
| `string` | UTF-8 encoded string | `"Hello, World!"` |
| `number` | IEEE 754 double-precision floating point | `42.0` |
| `integer` | 64-bit signed integer | `-9223372036854775808` to `9223372036854775807` |
| `boolean` | `true` or `false` | `true` |
| `null` | Null value | `null` |

#### 2.4.2. Complex Types

**UUID (Universally Unique Identifier):**
- Format: `xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`
- Example: `"550e8400-e29b-41d4-a716-446655440000"`
- Used for: Document IDs, User IDs, Workspace IDs

**ISO 8601 Timestamp:**
- Format: `YYYY-MM-DDTHH:mm:ss.sssZ`
- Example: `"2026-02-07T14:37:18.896Z"`
- Used for: Created timestamps, Updated timestamps

**URI (Uniform Resource Identifier):**
- Format: RFC 3986 compliant URI
- Example: `"https://api.tachyon.dev/api/v1/documents/550e8400-e29b-41d4-a716-446655440000"`
- Used for: Resource references

### 2.5. Authentication

#### 2.5.1. Authentication Mechanisms

The Tachyon REST API supports the following authentication mechanisms:

| Mechanism | Type | Usage |
|-----------|------|-------|
| **Bearer Token** | JWT | Primary authentication for API requests |
| **API Key** | Static key | Service account authentication |
| **Session Cookie** | Cookie | Web frontend authentication |

#### 2.5.2. Bearer Token Authentication

Bearer tokens are JSON Web Tokens (JWT) signed with RS256 algorithm.

**Request Header:**
```
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Token Structure:**
```json
{
  "sub": "user-id",
  "name": "User Name",
  "email": "user@example.com",
  "roles": ["user"],
  "exp": 1738943838,
  "iat": 1738940238
}
```

**Token Claims:**
| Claim | Description | Type |
|-------|-------------|------|
| `sub` | Subject (user ID) | string (UUID) |
| `name` | User name | string |
| `email` | User email | string |
| `roles` | User roles | array of strings |
| `exp` | Expiration timestamp | integer (Unix timestamp) |
| `iat` | Issued at timestamp | integer (Unix timestamp) |

### 2.6. Security

#### 2.6.1. Transport Security

All API communication must use TLS 1.3:

- **Protocol:** TLS 1.3
- **Cipher Suites:** TLS_AES_256_GCM_SHA384, TLS_CHACHA20_POLY1305_SHA256
- **Certificate Validation:** Strict validation required
- **HSTS:** HTTP Strict Transport Security enabled

#### 2.6.2. Input Validation

All input is validated according to the following principles:

1. **Type Validation:** All inputs are validated against expected types
2. **Length Validation:** String inputs are validated for minimum and maximum lengths
3. **Format Validation:** Structured inputs (email, UUID, URI) are validated for format
4. **Range Validation:** Numeric inputs are validated for minimum and maximum values
5. **Content Validation:** Content is validated for allowed characters and encoding

#### 2.6.3. Output Encoding

All output is encoded to prevent injection attacks:

1. **JSON Encoding:** JSON responses are properly escaped
2. **HTML Encoding:** HTML content is entity-encoded
3. **XSS Prevention:** Content-Type headers are set correctly
4. **CSP Headers:** Content-Security-Policy headers are included

### 2.7. CORS (Cross-Origin Resource Sharing)

The Tachyon REST API implements CORS for cross-origin requests:

**CORS Headers:**

| Header | Value | Description |
|--------|--------|-------------|
| `Access-Control-Allow-Origin` | `*` or specific origin | Allowed origins for requests |
| `Access-Control-Allow-Methods` | `GET, POST, PUT, PATCH, DELETE, OPTIONS` | Allowed HTTP methods |
| `Access-Control-Allow-Headers` | `Content-Type, Authorization, X-Request-ID` | Allowed request headers |
| `Access-Control-Max-Age` | `86400` | Preflight cache duration (seconds) |
| `Access-Control-Allow-Credentials` | `true` | Allow credentials in requests |

**Preflight Request:**

```
OPTIONS /api/v1/documents HTTP/1.1
Host: api.tachyon.dev
Origin: https://tachyon.dev
Access-Control-Request-Method: POST
Access-Control-Request-Headers: Content-Type, Authorization
```

**Preflight Response:**

```
HTTP/1.1 204 No Content
Access-Control-Allow-Origin: https://tachyon.dev
Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Request-ID
Access-Control-Max-Age: 86400
Access-Control-Allow-Credentials: true
```

---

## 3. AUTHENTICATION ENDPOINTS

Authentication endpoints provide mechanisms for users to authenticate with the Tachyon system, obtain access tokens, and manage their authentication sessions.

### 3.1. POST /auth/login

**Description:** Authenticates a user with email and password credentials, returning a JWT bearer token for subsequent authenticated requests.

**Authentication:** Not required

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
```

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "secure_password_123"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `email` | string | Yes | Valid email format, max 255 characters | User email address |
| `password` | string | Yes | Min 8 characters, max 128 characters | User password |

**Response:**

**200 OK:** Authentication successful

```json
{
  "data": {
    "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAwIiwibmFtZSI6IkpvaG4gRG9lIiwiZW1haWwiOiJqb2huLmRvZUBleGFtcGxlLmNvbSIsInJvbGVzIjpbInVzZXIiXSwiZXhwIjoxNzM4OTQzODM4LCJpYXQiOjE3Mzg5NDAyMzh9.invalid_signature",
    "token_type": "Bearer",
    "expires_in": 3600,
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAwIiwidHlwZSI6InJlZnJlc2giLCJleHAiOjE3MzkyMjQyMzh9.invalid_signature",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "John Doe",
      "email": "john.doe@example.com",
      "roles": ["user"]
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

```json
{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid email or password",
    "details": {
      "field": "email"
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Authentication failed

```json
{
  "error": {
    "code": "AUTHENTICATION_FAILED",
    "message": "Authentication failed",
    "details": {}
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**429 Too Many Requests:** Rate limit exceeded

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many login attempts. Please try again later.",
    "details": {
      "retry_after": 60
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**Rate Limiting:** Maximum 5 login attempts per 15 minutes per IP address.

### 3.2. POST /auth/logout

**Description:** Invalidates the current authentication session, logging the user out of the system.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:** None

**Response:**

**204 No Content:** Logout successful

**401 Unauthorized:** Invalid or expired token

```json
{
  "error": {
    "code": "INVALID_TOKEN",
    "message": "Invalid or expired authentication token",
    "details": {}
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

### 3.3. POST /auth/refresh

**Description:** Refreshes an expired or expiring access token using a valid refresh token.

**Authentication:** Not required (refresh token in request body)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
```

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAwIiwidHlwZSI6InJlZnJlc2giLCJleHAiOjE3MzkyMjQyMzh9.invalid_signature"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `refresh_token` | string | Yes | Valid JWT refresh token | Refresh token obtained from login |

**Response:**

**200 OK:** Token refreshed successfully

```json
{
  "data": {
    "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAwIiwibmFtZSI6IkpvaG4gRG9lIiwiZW1haWwiOiJqb2huLmRvZUBleGFtcGxlLmNvbSIsInJvbGVzIjpbInVzZXIiXSwiZXhwIjoxNzM4OTQzODM4LCJpYXQiOjE3Mzg5NDAyMzh9.invalid_signature",
    "token_type": "Bearer",
    "expires_in": 3600,
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAwIiwidHlwZSI6InJlZnJlc2giLCJleHAiOjE3MzkyMjQyMzh9.invalid_signature"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid refresh token

```json
{
  "error": {
    "code": "INVALID_REFRESH_TOKEN",
    "message": "Invalid or expired refresh token",
    "details": {}
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

### 3.4. GET /auth/me

**Description:** Retrieves information about the currently authenticated user.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Parameters:** None

**Response:**

**200 OK:** User information retrieved

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "John Doe",
    "email": "john.doe@example.com",
    "roles": ["user"],
    "created_at": "2026-01-01T00:00:00.000Z",
    "updated_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

```json
{
  "error": {
    "code": "INVALID_TOKEN",
    "message": "Invalid or expired authentication token",
    "details": {}
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

### 3.5. POST /auth/register

**Description:** Registers a new user account with email and password credentials.

**Authentication:** Not required

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
```

**Request Body:**
```json
{
  "name": "John Doe",
  "email": "john.doe@example.com",
  "password": "secure_password_123"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `name` | string | Yes | Min 1 character, max 100 characters | User display name |
| `email` | string | Yes | Valid email format, max 255 characters | User email address |
| `password` | string | Yes | Min 8 characters, max 128 characters | User password |

**Response:**

**201 Created:** User registered successfully

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "John Doe",
    "email": "john.doe@example.com",
    "roles": ["user"],
    "created_at": "2026-02-07T14:37:18.896Z",
    "updated_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

```json
{
  "error": {
    "code": "INVALID_REGISTRATION",
    "message": "Invalid registration data",
    "details": {
      "errors": [
        {
          "field": "email",
          "message": "Invalid email format"
        },
        {
          "field": "password",
          "message": "Password must be at least 8 characters"
        }
      ]
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**409 Conflict:** Email already registered

```json
{
  "error": {
    "code": "EMAIL_ALREADY_EXISTS",
    "message": "Email address is already registered",
    "details": {
      "email": "john.doe@example.com"
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**Rate Limiting:** Maximum 3 registration attempts per hour per IP address.

---

## 4. DOCUMENT ENDPOINTS

Document endpoints provide CRUD operations for managing documents within the Tachyon system, including creation, retrieval, update, deletion, and search functionality.

### 4.1. GET /documents

**Description:** Retrieves a paginated list of documents with optional filtering and sorting.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|-----------|---------|-------------|-------------|
| `page` | integer | No | 1, min 1 | Page number for pagination |
| `per_page` | integer | No | 20, min 1, max 100 | Number of documents per page |
| `workspace_id` | string (UUID) | No | Valid UUID | Filter by workspace ID |
| `search` | string | No | Max 256 characters | Search query for full-text search |
| `sort_by` | string | No | `created_at` | `created_at`, `updated_at`, `title` |
| `sort_order` | string | No | `desc` | `asc`, `desc` |
| `tag` | string | No | Max 100 characters | Filter by tag |

**Response:**

**200 OK:** Documents retrieved successfully

```json
{
  "data": {
    "documents": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
        "title": "Document Title",
        "content": "Document content...",
        "tags": ["tag1", "tag2"],
        "created_at": "2026-02-07T14:37:18.896Z",
        "updated_at": "2026-02-07T14:37:18.896Z"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_pages": 5,
      "total_items": 100
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**400 Bad Request:** Invalid query parameters

### 4.2. POST /documents

**Description:** Creates a new document with the provided content and metadata.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
  "title": "Document Title",
  "content": "Document content...",
  "tags": ["tag1", "tag2"]
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `workspace_id` | string (UUID) | Yes | Valid UUID | Workspace ID for the document |
| `title` | string | Yes | Min 1 character, max 200 characters | Document title |
| `content` | string | Yes | Max 10MB | Document content (Markdown) |
| `tags` | array of strings | No | Max 50 tags, max 100 characters each | Document tags |

**Response:**

**201 Created:** Document created successfully

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
    "title": "Document Title",
    "content": "Document content...",
    "tags": ["tag1", "tag2"],
    "created_at": "2026-02-07T14:37:18.896Z",
    "updated_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Workspace not found

### 4.3. GET /documents/{id}

**Description:** Retrieves a specific document by its unique identifier.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Document ID |

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|-----------|---------|-------------|
| `include_content` | boolean | No | `false` | Include full document content |

**Response:**

**200 OK:** Document retrieved successfully

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
    "title": "Document Title",
    "content": "Document content...",
    "tags": ["tag1", "tag2"],
    "created_at": "2026-02-07T14:37:18.896Z",
    "updated_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to access document

**404 Not Found:** Document not found

### 4.4. PUT /documents/{id}

**Description:** Updates an existing document, replacing all fields with the provided values.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Document ID |

**Request Body:**
```json
{
  "title": "Updated Document Title",
  "content": "Updated document content...",
  "tags": ["tag1", "tag2", "tag3"]
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `title` | string | No | Min 1 character, max 200 characters | Document title |
| `content` | string | No | Max 10MB | Document content (Markdown) |
| `tags` | array of strings | No | Max 50 tags, max 100 characters each | Document tags |

**Response:**

**200 OK:** Document updated successfully

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
    "title": "Updated Document Title",
    "content": "Updated document content...",
    "tags": ["tag1", "tag2", "tag3"],
    "created_at": "2026-02-07T14:37:18.896Z",
    "updated_at": "2026-02-07T14:40:00.000Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:40:00.000Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to update document

**404 Not Found:** Document not found

**409 Conflict:** Document version conflict (concurrent modification)

### 4.5. PATCH /documents/{id}

**Description:** Partially updates an existing document, updating only the provided fields.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Document ID |

**Request Body:**
```json
{
  "title": "Partially Updated Title"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `title` | string | No | Min 1 character, max 200 characters | Document title |
| `content` | string | No | Max 10MB | Document content (Markdown) |
| `tags` | array of strings | No | Max 50 tags, max 100 characters each | Document tags |

**Response:**

**200 OK:** Document updated successfully

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to update document

**404 Not Found:** Document not found

**409 Conflict:** Document version conflict (concurrent modification)

### 4.6. DELETE /documents/{id}

**Description:** Deletes a document by its unique identifier.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Document ID |

**Response:**

**204 No Content:** Document deleted successfully

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to delete document

**404 Not Found:** Document not found

### 4.7. GET /documents/{id}/content

**Description:** Retrieves the raw Markdown content of a specific document.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: text/markdown
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Document ID |

**Response:**

**200 OK:** Document content retrieved successfully

```
Content-Type: text/markdown; charset=utf-8

# Document Title

Document content...
```

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to access document

**404 Not Found:** Document not found

### 4.8. POST /documents/search

**Description:** Performs a full-text search across all documents accessible to the user.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "query": "search terms",
  "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
  "page": 1,
  "per_page": 20
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `query` | string | Yes | Min 1 character, max 256 characters | Search query |
| `workspace_id` | string (UUID) | No | Valid UUID | Filter by workspace ID |
| `page` | integer | No | 1, min 1 | Page number for pagination |
| `per_page` | integer | No | 20, min 1, max 100 | Number of results per page |

**Response:**

**200 OK:** Search completed successfully

```json
{
  "data": {
    "results": [
      {
        "document_id": "550e8400-e29b-41d4-a716-446655440000",
        "title": "Document Title",
        "snippet": "...search terms highlighted...",
        "score": 0.95
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_pages": 3,
      "total_items": 50
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid search query

**401 Unauthorized:** Invalid or expired token

---

## 5. WORKSPACE ENDPOINTS

Workspace endpoints provide CRUD operations for managing workspaces within the Tachyon system, including creation, retrieval, update, and deletion of workspaces.

### 5.1. GET /workspaces

**Description:** Retrieves a list of workspaces accessible to the authenticated user.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|-----------|---------|-------------|-------------|
| `page` | integer | No | 1 | Min 1 | Page number for pagination |
| `per_page` | integer | No | 20 | Min 1, max 100 | Number of workspaces per page |

**Response:**

**200 OK:** Workspaces retrieved successfully

```json
{
  "data": {
    "workspaces": [
      {
        "id": "660e9511-f3ac-52e5-b827-557766551111",
        "name": "My Workspace",
        "description": "A workspace for my documents",
        "created_at": "2026-02-07T14:37:18.896Z",
        "updated_at": "2026-02-07T14:37:18.896Z"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_pages": 1,
      "total_items": 1
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

### 5.2. POST /workspaces

**Description:** Creates a new workspace with the provided name and description.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "name": "My Workspace",
  "description": "A workspace for my documents"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `name` | string | Yes | Min 1 character, max 100 characters | Workspace name |
| `description` | string | No | Max 500 characters | Workspace description |

**Response:**

**201 Created:** Workspace created successfully

```json
{
  "data": {
    "id": "660e9511-f3ac-52e5-b827-557766551111",
    "name": "My Workspace",
    "description": "A workspace for my documents",
    "created_at": "2026-02-07T14:37:18.896Z",
    "updated_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

### 5.3. GET /workspaces/{id}

**Description:** Retrieves a specific workspace by its unique identifier.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Workspace ID |

**Response:**

**200 OK:** Workspace retrieved successfully

```json
{
  "data": {
    "id": "660e9511-f3ac-52e5-b827-557766551111",
    "name": "My Workspace",
    "description": "A workspace for my documents",
    "created_at": "2026-02-07T14:37:18.896Z",
    "updated_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to access workspace

**404 Not Found:** Workspace not found

### 5.4. PUT /workspaces/{id}

**Description:** Updates an existing workspace, replacing all fields with the provided values.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Workspace ID |

**Request Body:**
```json
{
  "name": "Updated Workspace Name",
  "description": "Updated workspace description"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `name` | string | No | Min 1 character, max 100 characters | Workspace name |
| `description` | string | No | Max 500 characters | Workspace description |

**Response:**

**200 OK:** Workspace updated successfully

```json
{
  "data": {
    "id": "660e9511-f3ac-52e5-b827-557766551111",
    "name": "Updated Workspace Name",
    "description": "Updated workspace description",
    "created_at": "2026-02-07T14:37:18.896Z",
    "updated_at": "2026-02-07T14:40:00.000Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:40:00.000Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to update workspace

**404 Not Found:** Workspace not found

### 5.5. DELETE /workspaces/{id}

**Description:** Deletes a workspace by its unique identifier.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Workspace ID |

**Response:**

**204 No Content:** Workspace deleted successfully

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to delete workspace

**404 Not Found:** Workspace not found

### 5.6. GET /workspaces/{id}/documents

**Description:** Retrieves a paginated list of documents within a specific workspace.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `id` | string (UUID) | Yes | Valid UUID | Workspace ID |

**Query Parameters:**

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|-----------|---------|-------------|-------------|
| `page` | integer | No | 1 | Min 1 | Page number for pagination |
| `per_page` | integer | No | 20 | Min 1, max 100 | Number of documents per page |
| `search` | string | No | | Max 256 characters | Search query for full-text search |
| `sort_by` | string | No | `created_at` | `created_at`, `updated_at`, `title` | Sort field |
| `sort_order` | string | No | `desc` | `asc`, `desc` | Sort order |

**Response:**

**200 OK:** Documents retrieved successfully

```json
{
  "data": {
    "documents": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
        "title": "Document Title",
        "tags": ["tag1", "tag2"],
        "created_at": "2026-02-07T14:37:18.896Z",
        "updated_at": "2026-02-07T14:37:18.896Z"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_pages": 5,
      "total_items": 100
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions to access workspace

**404 Not Found:** Workspace not found

---

## 6. GIT ENDPOINTS

Git endpoints provide operations for managing Git repositories within Tachyon system, including repository status, commit operations, and branch management.

### 6.1. GET /git/status

**Description:** Retrieves Git status of a workspace, including current branch, modified files, and untracked files.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `workspace_id` | string (UUID) | Yes | Valid UUID | Workspace ID |

**Response:**

**200 OK:** Git status retrieved successfully

```json
{
  "data": {
    "branch": "main",
    "commit": "abc123def456789",
    "modified": [
      {
        "path": "documents/example.md",
        "status": "modified"
      }
    ],
    "untracked": [
      {
        "path": "documents/new.md",
        "status": "untracked"
      }
    ]
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Workspace not found

### 6.2. GET /git/commits

**Description:** Retrieves a paginated list of Git commits for a workspace.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|-----------|---------|-------------|-------------|
| `workspace_id` | string (UUID) | Yes | | Valid UUID | Workspace ID |
| `page` | integer | No | 1 | Min 1 | Page number for pagination |
| `per_page` | integer | No | 20 | Min 1, max 100 | Number of commits per page |

**Response:**

**200 OK:** Commits retrieved successfully

```json
{
  "data": {
    "commits": [
      {
        "hash": "abc123def456789",
        "author": "John Doe <john.doe@example.com>",
        "message": "Update document",
        "timestamp": "2026-02-07T14:37:18.896Z"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_pages": 5,
      "total_items": 100
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Workspace not found

### 6.3. POST /git/commit

**Description:** Creates a new Git commit with provided message and staged changes.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
  "message": "Commit message",
  "files": [
    {
      "path": "documents/example.md",
      "content": "Updated content"
    }
  ]
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `workspace_id` | string (UUID) | Yes | Valid UUID | Workspace ID |
| `message` | string | Yes | Min 1 character, max 200 characters | Commit message |
| `files` | array of objects | No | Max 100 files | Files to commit |

**Response:**

**201 Created:** Commit created successfully

```json
{
  "data": {
    "hash": "abc123def456789",
    "message": "Commit message",
    "author": "John Doe <john.doe@example.com>",
    "timestamp": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Workspace not found

### 6.4. GET /git/branches

**Description:** Retrieves a list of Git branches for a workspace.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `workspace_id` | string (UUID) | Yes | Valid UUID | Workspace ID |

**Response:**

**200 OK:** Branches retrieved successfully

```json
{
  "data": {
    "branches": [
      {
        "name": "main",
        "is_current": true
      },
      {
        "name": "feature/new-feature",
        "is_current": false
      }
    ]
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Workspace not found

### 6.5. POST /git/branches

**Description:** Creates a new Git branch in a workspace.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
  "branch_name": "feature/new-feature",
  "start_point": "main"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `workspace_id` | string (UUID) | Yes | Valid UUID | Workspace ID |
| `branch_name` | string | Yes | Min 1 character, max 100 characters | Branch name |
| `start_point` | string | No | Valid branch name or commit hash | Starting point for new branch |

**Response:**

**201 Created:** Branch created successfully

```json
{
  "data": {
    "name": "feature/new-feature",
    "created_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Workspace not found

**409 Conflict:** Branch already exists

### 6.6. POST /git/merge

**Description:** Merges a Git branch into another branch.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "workspace_id": "660e9511-f3ac-52e5-b827-557766551111",
  "source_branch": "feature/new-feature",
  "target_branch": "main",
  "merge_strategy": "merge"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `workspace_id` | string (UUID) | Yes | Valid UUID | Workspace ID |
| `source_branch` | string | Yes | Valid branch name | Source branch to merge |
| `target_branch` | string | Yes | Valid branch name | Target branch to merge into |
| `merge_strategy` | string | No | `merge`, `squash`, `rebase` | Merge strategy |

**Response:**

**200 OK:** Merge completed successfully

```json
{
  "data": {
    "merge_commit": "abc123def456789",
    "merged_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Workspace or branch not found

**409 Conflict:** Merge conflict detected

---

## 7. PLUGIN ENDPOINTS

Plugin endpoints provide operations for managing plugins within Tachyon system, including plugin discovery, installation, and configuration.

### 7.1. GET /plugins

**Description:** Retrieves a list of available plugins.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|-----------|---------|-------------|-------------|
| `page` | integer | No | 1 | Min 1 | Page number for pagination |
| `per_page` | integer | No | 20 | Min 1, max 100 | Number of plugins per page |

**Response:**

**200 OK:** Plugins retrieved successfully

```json
{
  "data": {
    "plugins": [
      {
        "id": "plugin-id",
        "name": "Plugin Name",
        "version": "1.0.0",
        "description": "Plugin description",
        "author": "Plugin Author"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_pages": 1,
      "total_items": 10
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

### 7.2. POST /plugins/install

**Description:** Installs a plugin by its identifier.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "plugin_id": "plugin-id",
  "version": "1.0.0"
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `plugin_id` | string | Yes | Valid plugin ID | Plugin identifier |
| `version` | string | No | Valid semantic version | Plugin version |

**Response:**

**201 Created:** Plugin installed successfully

```json
{
  "data": {
    "plugin_id": "plugin-id",
    "version": "1.0.0",
    "installed_at": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Plugin not found

**409 Conflict:** Plugin already installed

### 7.3. DELETE /plugins/{plugin_id}

**Description:** Uninstalls a plugin by its identifier.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `plugin_id` | string | Yes | Valid plugin ID | Plugin identifier |

**Response:**

**204 No Content:** Plugin uninstalled successfully

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Plugin not found

### 7.4. GET /plugins/{plugin_id}/config

**Description:** Retrieves configuration for a specific plugin.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `plugin_id` | string | Yes | Valid plugin ID | Plugin identifier |

**Response:**

**200 OK:** Plugin configuration retrieved successfully

```json
{
  "data": {
    "plugin_id": "plugin-id",
    "config": {
      "setting1": "value1",
      "setting2": "value2"
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Plugin not found

### 7.5. PUT /plugins/{plugin_id}/config

**Description:** Updates configuration for a specific plugin.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Path Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `plugin_id` | string | Yes | Valid plugin ID | Plugin identifier |

**Request Body:**
```json
{
  "config": {
    "setting1": "new_value1",
    "setting2": "new_value2"
  }
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `config` | object | Yes | Plugin-specific | Plugin configuration |

**Response:**

**200 OK:** Plugin configuration updated successfully

```json
{
  "data": {
    "plugin_id": "plugin-id",
    "config": {
      "setting1": "new_value1",
      "setting2": "new_value2"
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

**404 Not Found:** Plugin not found

---

## 8. CONFIGURATION ENDPOINTS

Configuration endpoints provide operations for managing system configuration, including user preferences and application settings.

### 8.1. GET /config

**Description:** Retrieves configuration for the authenticated user.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response:**

**200 OK:** Configuration retrieved successfully

```json
{
  "data": {
    "user": {
      "theme": "dark",
      "language": "en",
      "timezone": "UTC"
    },
    "application": {
      "editor_font_size": 14,
      "editor_line_numbers": true
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

### 8.2. PUT /config

**Description:** Updates configuration for the authenticated user.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Content-Type: application/json
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**
```json
{
  "user": {
    "theme": "light",
    "language": "en",
    "timezone": "America/New_York"
  },
  "application": {
    "editor_font_size": 16,
    "editor_line_numbers": false
  }
}
```

**Request Parameters:**

| Parameter | Type | Required | Constraints | Description |
|-----------|------|-----------|-------------|-------------|
| `user` | object | No | User preferences | User configuration |
| `application` | object | No | Application settings | Application configuration |

**Response:**

**200 OK:** Configuration updated successfully

```json
{
  "data": {
    "user": {
      "theme": "light",
      "language": "en",
      "timezone": "America/New_York"
    },
    "application": {
      "editor_font_size": 16,
      "editor_line_numbers": false
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**400 Bad Request:** Invalid request parameters

**401 Unauthorized:** Invalid or expired token

---

## 9. EVENT ENDPOINTS

Event endpoints provide operations for real-time event streaming and event history.

### 9.1. GET /events

**Description:** Retrieves a paginated list of events for the authenticated user.

**Authentication:** Required (Bearer token)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Query Parameters:**

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|-----------|---------|-------------|-------------|
| `page` | integer | No | 1 | Min 1 | Page number for pagination |
| `per_page` | integer | No | 20 | Min 1, max 100 | Number of events per page |
| `event_type` | string | No | | Event type filter |
| `since` | string (ISO 8601) | No | | Events since timestamp |
| `until` | string (ISO 8601) | No | | Events until timestamp |

**Response:**

**200 OK:** Events retrieved successfully

```json
{
  "data": {
    "events": [
      {
        "id": "event-id",
        "type": "document_created",
        "data": {
          "document_id": "550e8400-e29b-41d4-a716-446655440000"
        },
        "timestamp": "2026-02-07T14:37:18.896Z"
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_pages": 5,
      "total_items": 100
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

---

## 10. MONITORING ENDPOINTS

Monitoring endpoints provide health checks and metrics for system observability.

### 10.1. GET /health

**Description:** Performs a health check on the Tachyon API server.

**Authentication:** Not required

**Request Headers:**
```
Accept: application/json
```

**Response:**

**200 OK:** System is healthy

```json
{
  "data": {
    "status": "healthy",
    "version": "1.0.0",
    "timestamp": "2026-02-07T14:37:18.896Z"
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**503 Service Unavailable:** System is unhealthy

### 10.2. GET /metrics

**Description:** Retrieves system metrics for monitoring and observability.

**Authentication:** Required (Bearer token, requires admin role)

**Request Headers:**
```
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response:**

**200 OK:** Metrics retrieved successfully

```json
{
  "data": {
    "requests": {
      "total": 10000,
      "per_second": 50
    },
    "documents": {
      "total": 1000,
      "created_today": 50
    },
    "users": {
      "total": 100,
      "active": 80
    },
    "system": {
      "uptime": 99.9,
      "memory_usage": "512MB",
      "cpu_usage": "25%"
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

**401 Unauthorized:** Invalid or expired token

**403 Forbidden:** Insufficient permissions (admin role required)

---

## 11. ERROR RESPONSES

The Tachyon REST API uses a consistent error response format for all error conditions.

### 11.1. Error Response Structure

All error responses follow this structure:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      // Additional error-specific details
    }
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

### 11.2. Error Codes

| Error Code | HTTP Status | Description |
|-----------|-------------|-------------|
| `INVALID_CREDENTIALS` | 400 | Invalid email or password |
| `AUTHENTICATION_FAILED` | 401 | Authentication failed |
| `INVALID_TOKEN` | 401 | Invalid or expired authentication token |
| `PERMISSION_DENIED` | 403 | Valid authentication but insufficient permissions |
| `DOCUMENT_NOT_FOUND` | 404 | Document not found |
| `WORKSPACE_NOT_FOUND` | 404 | Workspace not found |
| `USER_NOT_FOUND` | 404 | User not found |
| `PLUGIN_NOT_FOUND` | 404 | Plugin not found |
| `INVALID_REQUEST` | 400 | Invalid request parameters |
| `VALIDATION_ERROR` | 422 | Request validation failed |
| `EMAIL_ALREADY_EXISTS` | 409 | Email address is already registered |
| `BRANCH_ALREADY_EXISTS` | 409 | Git branch already exists |
| `MERGE_CONFLICT` | 409 | Git merge conflict detected |
| `RATE_LIMIT_EXCEEDED` | 429 | Rate limit exceeded |
| `INTERNAL_ERROR` | 500 | Internal server error |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |

---

## 12. RATE LIMITING

The Tachyon REST API implements rate limiting to prevent abuse and ensure fair usage.

### 12.1. Rate Limiting Strategy

Rate limiting is implemented per endpoint and per authenticated user:

| Endpoint | Rate Limit | Window |
|----------|-----------|---------|
| `POST /auth/login` | 5 requests per 15 minutes | IP address |
| `POST /auth/register` | 3 requests per hour | IP address |
| `GET /documents` | 100 requests per minute | Authenticated user |
| `POST /documents` | 50 requests per minute | Authenticated user |
| `PUT /documents/{id}` | 50 requests per minute | Authenticated user |
| `DELETE /documents/{id}` | 50 requests per minute | Authenticated user |
| `GET /git/commits` | 100 requests per minute | Authenticated user |
| `POST /git/commit` | 50 requests per minute | Authenticated user |
| `GET /health` | 1000 requests per minute | IP address |

### 12.2. Rate Limiting Headers

Rate limiting information is included in response headers:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Total requests allowed in rate limit window |
| `X-RateLimit-Remaining` | Remaining requests in current rate limit window |
| `X-RateLimit-Reset` | Unix timestamp when rate limit window resets |

### 12.3. Rate Limiting Behavior

When rate limit is exceeded:

1. **HTTP Status:** 429 Too Many Requests
2. **Retry-After Header:** Included in response indicating seconds to wait
3. **Rate Limiting Headers:** Included in response with current quota information

---

## 13. PAGINATION

List endpoints support pagination to handle large result sets efficiently.

### 13.1. Pagination Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|-----------|---------|-------------|-------------|
| `page` | integer | No | 1 | Min 1 | Page number (1-indexed) |
| `per_page` | integer | No | 20 | Min 1, max 100 | Number of items per page |

### 13.2. Pagination Response

Paginated responses include pagination metadata:

```json
{
  "data": {
    "items": [
      // Array of items
    ]
  },
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total_pages": 5,
    "total_items": 100
  },
  "meta": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-07T14:37:18.896Z",
    "version": "1.0.0"
  }
}
```

### 13.3. Pagination Behavior

- **Page Number:** 1-indexed (first page is 1)
- **Per Page:** Maximum 100 items per page (configurable)
- **Empty Results:** Returns empty array with pagination metadata
- **Out of Range:** Returns 400 Bad Request with error details

---

## 14. FILTERING AND SORTING

List endpoints support filtering and sorting to customize result sets.

### 14.1. Filtering Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `workspace_id` | string (UUID) | Filter by workspace ID |
| `search` | string | Full-text search query |
| `tag` | string | Filter by tag |
| `event_type` | string | Filter by event type |
| `since` | string (ISO 8601) | Filter events since timestamp |
| `until` | string (ISO 8601) | Filter events until timestamp |

### 14.2. Sorting Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `sort_by` | string | `created_at` | Field to sort by |
| `sort_order` | string | `desc` | Sort order (`asc` or `desc`) |

### 14.3. Supported Sort Fields

| Field | Description |
|-------|-------------|
| `created_at` | Sort by creation timestamp |
| `updated_at` | Sort by update timestamp |
| `title` | Sort by document title |

---

## 15. REFERENCES

### 15.1. Standards and Specifications

| Document | URL | Description |
|----------|-----|-------------|
| OpenAPI 3.0.3 | https://spec.openapis.org/oas/v3.0.3 | OpenAPI Specification |
| RFC 7231 | https://datatracker.ietf.org/doc/html/rfc7231 | Hypertext Transfer Protocol (HTTP/1.1) |
| ISO 8601 | https://www.iso.org/standard/iso8601 | Data elements and interchange formats |
| JWT (JSON Web Token) | https://jwt.io/ | JWT specification |

### 15.2. Related Documentation

| Document | Path | Description |
|----------|------|-------------|
| [TACHYON-STD-V1.0](../.specs/01_standards/coding_standards.md) | Coding and Documentation Standards |
| [TACHYON-ADR-001-V1.0](../.specs/02_adrs/001_rust_as_primary_language.md) | Rust as Primary Language |
| [TACHYON-ADR-003-V1.0](../.specs/02_adrs/003_axum_for_http2_server.md) | Axum for HTTP/2 Server |
| [TACHYON-ADR-010-V1.0](../.specs/02_adrs/010_security_architecture.md) | Security Architecture |

### 15.3. Related Requirements

| Requirement ID | Description |
|---------------|-------------|
| REQ-DOC-031 | OpenAPI Specification for REST endpoints |
| REQ-DOC-032 | Example code for API operations |
| REQ-127 | API Development Requirements |
| REQ-128 | API Documentation Requirements |

---

**Document Control Information:**

- **Document ID:** TACHYON-API-002-V1.0
- **Classification:** API Specification
- **Status:** Approved for Implementation
- **Version:** 1.0.0
- **Last Updated:** 2026-02-07T14:37:18.896Z
- **Next Review:** 2026-08-07T14:37:18.896Z
