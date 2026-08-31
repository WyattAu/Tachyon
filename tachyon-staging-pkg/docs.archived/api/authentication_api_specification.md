# TACHYON: AUTHENTICATION API SPECIFICATION

**Document ID:** TACHYON-API-017-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** Technical API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 830-1998, RFC 6749 (OAuth 2.0), RFC 7519 (JWT)

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Authentication Design Principles](#2-authentication-design-principles)
3. [JWT Authentication](#3-jwt-authentication)
4. [Session Management](#4-session-management)
5. [Multi-Factor Authentication (MFA)](#5-multi-factor-authentication-mfa)
6. [Token Refresh](#6-token-refresh)
7. [Logout](#7-logout)
8. [OAuth 2.0 Integration](#8-oauth-20-integration)
9. [Authentication Security](#9-authentication-security)
10. [References](#10-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document specifies the Authentication API for the Tachyon toolchain, providing comprehensive documentation for user authentication, session management, token lifecycle, and multi-factor authentication mechanisms. The API enables secure authentication across desktop, web, and server components using industry-standard protocols and cryptographic primitives.

### 1.2. Scope

The Authentication API encompasses:

- **User Authentication:** Username/password authentication, OAuth 2.0 integration, and certificate-based authentication
- **Token Management:** JWT access tokens, refresh tokens, and token rotation
- **Session Management:** Session creation, validation, timeout, and revocation
- **Multi-Factor Authentication (MFA):** MFA setup, verification, and recovery
- **Security Controls:** Rate limiting, DDoS protection, and audit logging

Out of scope:
- Authorization and permission management (covered in Authorization API Specification)
- User registration and profile management (covered in User Management API)
- Password reset and recovery workflows (covered in User Management API)

### 1.3. Architecture Context

The Authentication API operates within the Tachyon three-tier architecture:

```
┌─────────────┐     HTTP/2      ┌─────────────┐
│   Desktop   │ ◄──────────────- │   Server    │
│  (Tauri)    │                  │   (Axum)    │
└─────────────┘                  └─────────────┘
                                          │
                                          -
┌─────────────┐     HTTP/2      ┌─────────────┐
│     Web     │ ◄──────────────- │   Server    │
│  (Leptos)   │                  │   (Axum)    │
└─────────────┘                  └─────────────┘
```

The server component implements the Authentication API using:
- **Runtime:** Tokio asynchronous runtime ([ADR-007](../../.adrs/adr-007-thread-safety-strategy.md))
- **Framework:** Axum web framework ([ADR-003](../../.adrs/adr-003-lru-cache-target.md))
- **Language:** Rust ([ADR-001](../../.adrs/adr-001-three-tier-jit-compilation.md))

### 1.4. Protocol Stack

The Authentication API uses the following protocol stack:

| Layer | Protocol | Standard |
|-------|-----------|----------|
| Transport | TCP | RFC 793 |
| Application | HTTP/2 | RFC 7540 |
| Security | TLS 1.3 | RFC 8446 |
| Authentication | JWT | RFC 7519 |
| Authorization | OAuth 2.0 | RFC 6749 |
| Data Format | JSON | RFC 8259 |

### 1.5. Related Documents

This specification references and depends on:

- [TACHYON-STD-V1.0](../../.adrs/ - Coding and Documentation Standards
- [TACHYON-REQ-SEC-V1.0](../../.adrs/ - Security Requirements
- [TACHYON-DES-SEC-V1.0](../../.adrs/ - Security Design
- [TACHYON-DES-API-V1.0](../../.adrs/ - API Interfaces Design
- [ADR-010](../../.adrs/adr-010-synchronization-primitives.md) - Security Architecture

### 1.6. Conventions

#### 1.6.1. Notation

- **Endpoints:** Represented as `METHOD /api/v1/path`
- **Request Bodies:** JSON objects with explicit type definitions
- **Response Bodies:** JSON objects with explicit type definitions
- **Error Codes:** String identifiers in format `AUTH_ERROR_XXX`
- **Status Codes:** HTTP status codes per RFC 7231

#### 1.6.2. Type Definitions

Type definitions use Rust syntax for precision, with JSON Schema representations for interoperability:

```rust
/// Example type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleType {
    /// Field description
    pub field_name: FieldType,
}
```

#### 1.6.3. Security Markings

All security-sensitive information is marked with appropriate classifications:
- `[SECRET]` - Cryptographic keys and secrets
- `[CONFIDENTIAL]` - User credentials and personal data
- `[INTERNAL]` - Implementation details not exposed to clients

---

## 2. AUTHENTICATION DESIGN PRINCIPLES

### 2.1. Core Security Principles

The Authentication API implements the following security principles derived from [REQ-SEC-001](../../.adrs/ through [REQ-SEC-010](../../.adrs/

#### 2.1.1. Defense in Depth

Multiple layers of security controls protect authentication operations:

1. **Transport Layer:** TLS 1.3 with perfect forward secrecy
2. **Application Layer:** Request validation and sanitization
3. **Authentication Layer:** Cryptographic token validation
4. **Authorization Layer:** Permission verification
5. **Audit Layer:** Comprehensive logging of all authentication events

#### 2.1.2. Least Privilege

Each authentication operation requires only the minimum privileges necessary:

- Authentication endpoints require no prior authentication
- Token refresh requires valid refresh token only
- Session management requires valid session token
- MFA operations require authenticated session

#### 2.1.3. Fail-Safe Defaults

All security configurations use safe defaults:

- Session timeout: 15 minutes (configurable)
- Maximum concurrent sessions: 3 per user
- Token expiration: 15 minutes (access), 30 days (refresh)
- Rate limiting: 10 requests per minute per IP
- MFA required: For all privileged operations

#### 2.1.4. Zero Trust

All authentication requests are verified regardless of source:

- Validate all input parameters against schemas
- Verify cryptographic signatures on all tokens
- Enforce rate limiting on all endpoints
- Log all authentication events for audit

### 2.2. Authentication Mechanisms

The Authentication API supports multiple authentication methods as specified in [DES-SEC-001](../../.adrs/

#### 2.2.1. Password-Based Authentication

Traditional username/password authentication with the following characteristics:

- **Password Storage:** Argon2id hashing with memory-hard parameters
- **Password Requirements:** Minimum 12 characters, complexity enforced per [REQ-SEC-012](../../.adrs/
- **Password Verification:** Constant-time comparison to prevent timing attacks
- **Failed Login Tracking:** Exponential backoff after failed attempts

#### 2.2.2. OAuth 2.0 Authentication

Federated authentication using OAuth 2.0 as specified in [REQ-SEC-013](../../.adrs/

- **Authorization Code Flow:** For web and desktop applications
- **PKCE (Proof Key for Code Exchange):** Required for public clients
- **State Parameter:** CSRF protection for authorization requests
- **Token Validation:** Verify issuer, audience, and signature

#### 2.2.3. Certificate-Based Authentication

X.509 certificate authentication for enterprise environments:

- **Certificate Validation:** Chain verification with CRL/OCSP checking
- **Certificate Pinning:** Enforce specific certificate authorities
- **Client Certificates:** Mutual TLS for inter-service authentication

### 2.3. Token Architecture

The Authentication API uses JSON Web Tokens (JWT) as specified in [REQ-SEC-016](../../.adrs/

#### 2.3.1. Access Token

Short-lived JWT used for API authentication:

```rust
/// JWT access token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    /// Token issuer
    pub iss: String,
    
    /// Token subject (user ID)
    pub sub: Uuid,
    
    /// Token audience
    pub aud: String,
    
    /// Token expiration timestamp
    pub exp: i64,
    
    /// Token issued at timestamp
    pub iat: i64,
    
    /// Token not valid before timestamp
    pub nbf: i64,
    
    /// Unique token identifier (JTI)
    pub jti: Uuid,
    
    /// User roles and permissions
    pub roles: Vec<String>,
    
    /// Token type identifier
    pub typ: String,  // "access"
}
```

**Constraints:**
- Algorithm: RS256 (RSA Signature with SHA-256)
- Expiration: 15 minutes (900 seconds) from issuance
- Key ID: Included in header for key rotation support
- Audience: `tachyon-api`

#### 2.3.2. Refresh Token

Long-lived token used to obtain new access tokens:

```rust
/// JWT refresh token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// Token issuer
    pub iss: String,
    
    /// Token subject (user ID)
    pub sub: Uuid,
    
    /// Token audience
    pub aud: String,
    
    /// Token expiration timestamp
    pub exp: i64,
    
    /// Token issued at timestamp
    pub iat: i64,
    
    /// Unique token identifier (JTI)
    pub jti: Uuid,
    
    /// Original access token JTI (for rotation)
    pub access_jti: Uuid,
    
    /// Token type identifier
    pub typ: String,  // "refresh"
}
```

**Constraints:**
- Algorithm: RS256 (RSA Signature with SHA-256)
- Expiration: 30 days (2,592,000 seconds) from issuance
- Rotation: Required on each refresh (old token invalidated)
- Audience: `tachyon-refresh`

### 2.4. Session Management

Session management follows the principles specified in [REQ-SEC-016](../../.adrs/ through [REQ-SEC-020](../../.adrs/

#### 2.4.1. Session Lifecycle

```
┌─────────────┐
│   Login     │
└──────┬──────┘
       │
       -
┌─────────────┐
│  Create     │
│  Session    │
└──────┬──────┘
       │
       -
┌─────────────┐     ┌─────────────┐
│   Active    │────-│   Refresh   │
│   Session   │     │   Tokens    │
└──────┬──────┘     └─────────────┘
       │
       -
┌─────────────┐
│   Timeout   │
└──────┬──────┘
       │
       -
┌─────────────┐
│   Expired   │
│   Session   │
└─────────────┘
```

#### 2.4.2. Session Timeout

Sessions expire after a period of inactivity:

- **Default Timeout:** 15 minutes
- **Configurable:** Per-user or per-organization settings
- **Warning:** Client receives warning 2 minutes before expiration
- **Grace Period:** 30 seconds to allow token refresh

#### 2.4.3. Concurrent Session Limits

Per [REQ-SEC-019](../../.adrs/ the system limits concurrent sessions:

- **Default Limit:** 3 concurrent sessions per user
- **Enforcement:** Oldest session invalidated when limit exceeded
- **Notification:** User notified of session termination
- **Exception:** Administrative users may have higher limits

### 2.5. Multi-Factor Authentication

MFA implementation follows [REQ-SEC-011](../../.adrs/

#### 2.5.1. MFA Methods

Supported MFA methods:

1. **TOTP (Time-based One-Time Password):** RFC 6238 compliant
2. **SMS-based OTP:** Backup method for recovery
3. **Hardware Tokens:** U2F/FIDO2 support (future)

#### 2.5.2. MFA Enforcement

MFA is required for:

- Initial authentication after account creation
- Authentication from new device or IP address
- Privileged operations (admin functions)
- Authentication after suspicious activity detected

### 2.6. Error Handling

Authentication errors follow the error handling conventions specified in [TACHYON-DES-API-V1.0](../../.adrs/

#### 2.6.1. Error Response Structure

```rust
/// Authentication error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthErrorResponse {
    /// Error code identifier
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional error details
    pub details: Option<serde_json::Value>,
    
    /// Request identifier for tracing
    pub request_id: String,
}
```

#### 2.6.2. Error Codes

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `AUTH_ERROR_001` | 400 | Invalid request format |
| `AUTH_ERROR_002` | 401 | Invalid credentials |
| `AUTH_ERROR_003` | 401 | Expired token |
| `AUTH_ERROR_004` | 401 | Invalid token signature |
| `AUTH_ERROR_005` | 403 | Account locked |
| `AUTH_ERROR_006` | 403 | MFA required |
| `AUTH_ERROR_007` | 403 | Session expired |
| `AUTH_ERROR_008` | 429 | Rate limit exceeded |
| `AUTH_ERROR_009` | 500 | Internal server error |
| `AUTH_ERROR_010` | 503 | Service unavailable |

---

**End of Chunk 1**

---

## 3. JWT AUTHENTICATION

### 3.1. Authentication Endpoint

#### 3.1.1. Login

**Endpoint:** `POST /api/v1/auth/login`

**Element ID:** API-AUTH-001
**Related Requirements:** [REQ-SEC-011](../../.adrs/ [REQ-SEC-012](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Authenticates a user with username and password credentials, returning JWT access and refresh tokens.

**Request Headers:**
```http
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:**
```rust
/// Login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Username or email address
    /// Constraints: 3-128 characters, valid email or username format
    #[serde(alias = "email")]
    pub username: String,
    
    /// User password
    /// Constraints: 12-256 characters
    /// Security: Transmitted over TLS 1.3
    #[serde(rename = "password")]
    pub password: String,
    
    /// Client device identifier for session tracking
    /// Constraints: Valid UUID v4
    #[serde(rename = "device_id")]
    pub device_id: Uuid,
    
    /// User agent string for device fingerprinting
    /// Constraints: 1-512 characters
    #[serde(rename = "user_agent")]
    pub user_agent: String,
    
    /// Client IP address (optional, server validates)
    /// Constraints: Valid IPv4 or IPv6 address
    #[serde(rename = "client_ip")]
    pub client_ip: Option<String>,
    
    /// Remember me flag for extended session
    /// Constraints: boolean
    #[serde(rename = "remember_me", default)]
    pub remember_me: bool,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["username", "password", "device_id", "user_agent"],
  "properties": {
    "username": {
      "type": "string",
      "minLength": 3,
      "maxLength": 128,
      "description": "Username or email address"
    },
    "password": {
      "type": "string",
      "minLength": 12,
      "maxLength": 256,
      "description": "User password"
    },
    "device_id": {
      "type": "string",
      "format": "uuid",
      "description": "Client device identifier"
    },
    "user_agent": {
      "type": "string",
      "minLength": 1,
      "maxLength": 512,
      "description": "User agent string"
    },
    "client_ip": {
      "type": "string",
      "format": "ipv4-or-ipv6",
      "description": "Client IP address"
    },
    "remember_me": {
      "type": "boolean",
      "default": false,
      "description": "Remember me flag"
    }
  }
}
```

**Response Headers (Success):**
```http
Content-Type: application/json
X-Request-ID: <uuid>
Cache-Control: no-store, no-cache, must-revalidate
Pragma: no-cache
```

**Response Body (Success - 200 OK):**
```rust
/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    /// JWT access token
    /// Constraints: RS256 signed, 15 minute expiration
    #[serde(rename = "access_token")]
    pub access_token: String,
    
    /// JWT refresh token
    /// Constraints: RS256 signed, 30 day expiration
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    
    /// Token type
    #[serde(rename = "token_type")]
    pub token_type: String,  // "Bearer"
    
    /// Access token expiration in seconds
    #[serde(rename = "expires_in")]
    pub expires_in: u64,  // 900
    
    /// Refresh token expiration in seconds
    #[serde(rename = "refresh_expires_in")]
    pub refresh_expires_in: u64,  // 2592000
    
    /// User information
    #[serde(rename = "user")]
    pub user: User,
    
    /// Session identifier
    #[serde(rename = "session_id")]
    pub session_id: Uuid,
    
    /// MFA required flag
    #[serde(rename = "mfa_required")]
    pub mfa_required: bool,
    
    /// MFA setup required flag
    #[serde(rename = "mfa_setup_required")]
    pub mfa_setup_required: bool,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    #[serde(rename = "id")]
    pub id: Uuid,
    
    /// Username
    #[serde(rename = "username")]
    pub username: String,
    
    /// Email address
    #[serde(rename = "email")]
    pub email: String,
    
    /// Display name
    #[serde(rename = "display_name")]
    pub display_name: Option<String>,
    
    /// User roles
    #[serde(rename = "roles")]
    pub roles: Vec<String>,
    
    /// Account status
    #[serde(rename = "status")]
    pub status: AccountStatus,
}

/// Account status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    /// Active account
    Active,
    /// Account locked
    Locked,
    /// Account pending verification
    Pending,
    /// Account disabled
    Disabled,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["access_token", "refresh_token", "token_type", "expires_in", "refresh_expires_in", "user", "session_id", "mfa_required", "mfa_setup_required"],
  "properties": {
    "access_token": {
      "type": "string",
      "description": "JWT access token"
    },
    "refresh_token": {
      "type": "string",
      "description": "JWT refresh token"
    },
    "token_type": {
      "type": "string",
      "enum": ["Bearer"],
      "description": "Token type"
    },
    "expires_in": {
      "type": "integer",
      "description": "Access token expiration in seconds"
    },
    "refresh_expires_in": {
      "type": "integer",
      "description": "Refresh token expiration in seconds"
    },
    "user": {
      "$ref": "#/definitions/User"
    },
    "session_id": {
      "type": "string",
      "format": "uuid",
      "description": "Session identifier"
    },
    "mfa_required": {
      "type": "boolean",
      "description": "MFA required flag"
    },
    "mfa_setup_required": {
      "type": "boolean",
      "description": "MFA setup required flag"
    }
  },
  "definitions": {
    "User": {
      "type": "object",
      "required": ["id", "username", "email", "roles", "status"],
      "properties": {
        "id": {
          "type": "string",
          "format": "uuid"
        },
        "username": {
          "type": "string"
        },
        "email": {
          "type": "string",
          "format": "email"
        },
        "display_name": {
          "type": "string"
        },
        "roles": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "status": {
          "type": "string",
          "enum": ["active", "locked", "pending", "disabled"]
        }
      }
    }
  }
}
```

**Response Body (Error - 400 Bad Request):**
```json
{
  "code": "AUTH_ERROR_001",
  "message": "Invalid request format",
  "details": {
    "field": "username",
    "error": "Username must be between 3 and 128 characters"
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response Body (Error - 401 Unauthorized):**
```json
{
  "code": "AUTH_ERROR_002",
  "message": "Invalid credentials",
  "details": {
    "attempts_remaining": 4,
    "lockout_time": null
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response Body (Error - 403 Forbidden):**
```json
{
  "code": "AUTH_ERROR_005",
  "message": "Account locked",
  "details": {
    "reason": "Too many failed login attempts",
    "lockout_until": "2026-02-06T03:00:00Z"
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response Body (Error - 429 Too Many Requests):**
```json
{
  "code": "AUTH_ERROR_008",
  "message": "Rate limit exceeded",
  "details": {
    "retry_after": 60,
    "limit": "10 requests per minute"
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Constraints:**
- `username`: 3-128 characters, valid email or username format
- `password`: 12-256 characters
- `device_id`: Valid UUID v4
- `user_agent`: 1-512 characters
- Rate limiting: 10 requests per minute per IP address
- Failed login tracking: Exponential backoff after 5 failed attempts

**Security Considerations:**
- Passwords are compared using constant-time comparison
- Failed login attempts are logged with IP address and user agent
- Account lockout after 5 consecutive failed attempts (30 minutes)
- MFA enforced if user has MFA enabled
- Session created and tracked for concurrent session limits

**Dependencies:**
- [REQ-SEC-011](../../.adrs/ Multi-Factor Authentication
- [REQ-SEC-012](../../.adrs/ Password Requirements
- [REQ-SEC-019](../../.adrs/ Concurrent Session Limits
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging

**Rationale:** Traditional username/password authentication provides a familiar authentication method while maintaining security through strong password requirements, rate limiting, and MFA support.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-4466554401" \
  -d '{
    "username": "user@example.com",
    "password": "SecurePassword123!",
    "device_id": "550e8400-e29b-41d4-a716-4466554402",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    "remember_me": true
  }'
```

---

**End of Chunk 2**

---

## 4. SESSION MANAGEMENT

### 4.1. Session Validation

#### 4.1.1. Validate Session

**Endpoint:** `GET /api/v1/auth/session/validate`

**Element ID:** API-AUTH-002
**Related Requirements:** [REQ-SEC-016](../../.adrs/ [REQ-SEC-017](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Validates the current session and returns session information including remaining time and MFA status.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:** None

**Response Headers (Success):**
```http
Content-Type: application/json
X-Request-ID: <uuid>
```

**Response Body (Success - 200 OK):**
```rust
/// Session validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionValidationResponse {
    /// Session identifier
    #[serde(rename = "session_id")]
    pub session_id: Uuid,
    
    /// User identifier
    #[serde(rename = "user_id")]
    pub user_id: Uuid,
    
    /// Session status
    #[serde(rename = "status")]
    pub status: SessionStatus,
    
    /// Session creation timestamp
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Session last activity timestamp
    #[serde(rename = "last_activity_at")]
    pub last_activity_at: DateTime<Utc>,
    
    /// Session expiration timestamp
    #[serde(rename = "expires_at")]
    pub expires_at: DateTime<Utc>,
    
    /// Remaining time in seconds
    #[serde(rename = "remaining_seconds")]
    pub remaining_seconds: u64,
    
    /// MFA verified flag
    #[serde(rename = "mfa_verified")]
    pub mfa_verified: bool,
    
    /// Device information
    #[serde(rename = "device")]
    pub device: DeviceInfo,
    
    /// Concurrent sessions count
    #[serde(rename = "concurrent_sessions")]
    pub concurrent_sessions: usize,
    
    /// Maximum concurrent sessions allowed
    #[serde(rename = "max_concurrent_sessions")]
    pub max_concurrent_sessions: usize,
}

/// Session status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Active session
    Active,
    /// Session expiring soon (within 2 minutes)
    ExpiringSoon,
    /// Session expired
    Expired,
    /// Session revoked
    Revoked,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device identifier
    #[serde(rename = "device_id")]
    pub device_id: Uuid,
    
    /// Device name (user-provided)
    #[serde(rename = "device_name")]
    pub device_name: Option<String>,
    
    /// Device type
    #[serde(rename = "device_type")]
    pub device_type: DeviceType,
    
    /// User agent string
    #[serde(rename = "user_agent")]
    pub user_agent: String,
    
    /// IP address
    #[serde(rename = "ip_address")]
    pub ip_address: String,
    
    /// Location (country, city)
    #[serde(rename = "location")]
    pub location: Option<LocationInfo>,
}

/// Device type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    /// Desktop application
    Desktop,
    /// Web browser
    Web,
    /// Mobile application
    Mobile,
    /// Unknown device type
    Unknown,
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    /// Country code (ISO 3166-1 alpha-2)
    #[serde(rename = "country_code")]
    pub country_code: String,
    
    /// Country name
    #[serde(rename = "country_name")]
    pub country_name: String,
    
    /// City name
    #[serde(rename = "city")]
    pub city: Option<String>,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["session_id", "user_id", "status", "created_at", "last_activity_at", "expires_at", "remaining_seconds", "mfa_verified", "device", "concurrent_sessions", "max_concurrent_sessions"],
  "properties": {
    "session_id": {
      "type": "string",
      "format": "uuid"
    },
    "user_id": {
      "type": "string",
      "format": "uuid"
    },
    "status": {
      "type": "string",
      "enum": ["active", "expiring_soon", "expired", "revoked"]
    },
    "created_at": {
      "type": "string",
      "format": "date-time"
    },
    "last_activity_at": {
      "type": "string",
      "format": "date-time"
    },
    "expires_at": {
      "type": "string",
      "format": "date-time"
    },
    "remaining_seconds": {
      "type": "integer"
    },
    "mfa_verified": {
      "type": "boolean"
    },
    "device": {
      "$ref": "#/definitions/DeviceInfo"
    },
    "concurrent_sessions": {
      "type": "integer"
    },
    "max_concurrent_sessions": {
      "type": "integer"
    }
  },
  "definitions": {
    "DeviceInfo": {
      "type": "object",
      "required": ["device_id", "device_type", "user_agent", "ip_address"],
      "properties": {
        "device_id": {
          "type": "string",
          "format": "uuid"
        },
        "device_name": {
          "type": "string"
        },
        "device_type": {
          "type": "string",
          "enum": ["desktop", "web", "mobile", "unknown"]
        },
        "user_agent": {
          "type": "string"
        },
        "ip_address": {
          "type": "string"
        },
        "location": {
          "$ref": "#/definitions/LocationInfo"
        }
      }
    },
    "LocationInfo": {
      "type": "object",
      "required": ["country_code", "country_name"],
      "properties": {
        "country_code": {
          "type": "string"
        },
        "country_name": {
          "type": "string"
        },
        "city": {
          "type": "string"
        }
      }
    }
  }
}
```

**Response Body (Error - 401 Unauthorized):**
```json
{
  "code": "AUTH_ERROR_003",
  "message": "Expired token",
  "details": {
    "token_type": "access",
    "expired_at": "2026-02-06T02:10:00Z"
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554403"
}
```

**Response Body (Error - 403 Forbidden):**
```json
{
  "code": "AUTH_ERROR_007",
  "message": "Session expired",
  "details": {
    "session_id": "550e8400-e29b-41d4-a716-4466554404",
    "expired_at": "2026-02-06T02:10:00Z"
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554405"
}
```

**Constraints:**
- Requires valid JWT access token
- Session timeout: 15 minutes of inactivity
- Maximum concurrent sessions: 3 per user
- Rate limiting: 30 requests per minute per session

**Security Considerations:**
- Validates token signature and expiration
- Updates session last activity timestamp
- Checks for session revocation
- Enforces concurrent session limits
- Logs session validation events

**Dependencies:**
- [REQ-SEC-016](../../.adrs/ Secure Session Tokens
- [REQ-SEC-017](../../.adrs/ Session Timeout
- [REQ-SEC-019](../../.adrs/ Concurrent Session Limits
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging

**Rationale:** Session validation enables clients to check session status and proactively handle expiration, improving user experience and security.

**Usage Example:**
```bash
curl -X GET https://api.tachyon.dev/api/v1/auth/session/validate \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-4466554406"
```

---

### 4.2. Session List

#### 4.2.1. List Sessions

**Endpoint:** `GET /api/v1/auth/sessions`

**Element ID:** API-AUTH-003
**Related Requirements:** [REQ-SEC-019](../../.adrs/ [REQ-SEC-020](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Retrieves all active sessions for the authenticated user.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Query Parameters:**
```rust
/// Session list query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsQuery {
    /// Include expired sessions
    #[serde(rename = "include_expired", default)]
    pub include_expired: bool,
    
    /// Include revoked sessions
    #[serde(rename = "include_revoked", default)]
    pub include_revoked: bool,
}
```

**Response Body (Success - 200 OK):**
```rust
/// Session list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListResponse {
    /// List of sessions
    #[serde(rename = "sessions")]
    pub sessions: Vec<SessionInfo>,
    
    /// Total count
    #[serde(rename = "total")]
    pub total: usize,
    
    /// Active count
    #[serde(rename = "active_count")]
    pub active_count: usize,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session identifier
    #[serde(rename = "session_id")]
    pub session_id: Uuid,
    
    /// Session status
    #[serde(rename = "status")]
    pub status: SessionStatus,
    
    /// Device information
    #[serde(rename = "device")]
    pub device: DeviceInfo,
    
    /// Session creation timestamp
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Session last activity timestamp
    #[serde(rename = "last_activity_at")]
    pub last_activity_at: DateTime<Utc>,
    
    /// Session expiration timestamp
    #[serde(rename = "expires_at")]
    pub expires_at: DateTime<Utc>,
    
    /// Current session flag
    #[serde(rename = "is_current")]
    pub is_current: bool,
    
    /// MFA verified flag
    #[serde(rename = "mfa_verified")]
    pub mfa_verified: bool,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["sessions", "total", "active_count"],
  "properties": {
    "sessions": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/SessionInfo"
      }
    },
    "total": {
      "type": "integer"
    },
    "active_count": {
      "type": "integer"
    }
  },
  "definitions": {
    "SessionInfo": {
      "type": "object",
      "required": ["session_id", "status", "device", "created_at", "last_activity_at", "expires_at", "is_current", "mfa_verified"],
      "properties": {
        "session_id": {
          "type": "string",
          "format": "uuid"
        },
        "status": {
          "type": "string",
          "enum": ["active", "expiring_soon", "expired", "revoked"]
        },
        "device": {
          "$ref": "#/definitions/DeviceInfo"
        },
        "created_at": {
          "type": "string",
          "format": "date-time"
        },
        "last_activity_at": {
          "type": "string",
          "format": "date-time"
        },
        "expires_at": {
          "type": "string",
          "format": "date-time"
        },
        "is_current": {
          "type": "boolean"
        },
        "mfa_verified": {
          "type": "boolean"
        }
      }
    }
  }
}
```

**Constraints:**
- Requires valid JWT access token
- Rate limiting: 10 requests per minute per session
- Maximum returned sessions: 100

**Security Considerations:**
- Only returns sessions for the authenticated user
- Current session is clearly marked
- Device information is included for identification
- MFA status is shown for each session

**Dependencies:**
- [REQ-SEC-019](../../.adrs/ Concurrent Session Limits
- [REQ-SEC-020](../../.adrs/ Session Revocation

**Rationale:** Session listing enables users to monitor their active sessions and identify suspicious activity.

**Usage Example:**
```bash
curl -X GET "https://api.tachyon.dev/api/v1/auth/sessions?include_expired=true" \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-4466554407"
```

---

**End of Chunk 3**

---

## 5. MULTI-FACTOR AUTHENTICATION (MFA)

### 5.1. MFA Setup

#### 5.1.1. Initiate MFA Setup

**Endpoint:** `POST /api/v1/auth/mfa/setup/initiate`

**Element ID:** API-AUTH-004
**Related Requirements:** [REQ-SEC-011](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Initiates MFA setup for TOTP (Time-based One-Time Password) authentication. Returns a secret key and QR code for authenticator app registration.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:**
```rust
/// MFA setup initiation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupInitiateRequest {
    /// MFA method
    #[serde(rename = "method")]
    pub method: MfaMethod,
    
    /// Device name for this MFA factor
    /// Constraints: 1-64 characters
    #[serde(rename = "device_name")]
    pub device_name: String,
}

/// MFA method enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MfaMethod {
    /// Time-based One-Time Password (RFC 6238)
    Totp,
    /// SMS-based OTP (backup method)
    Sms,
    /// Hardware token (future)
    Hardware,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["method", "device_name"],
  "properties": {
    "method": {
      "type": "string",
      "enum": ["totp", "sms", "hardware"],
      "description": "MFA method"
    },
    "device_name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 64,
      "description": "Device name for this MFA factor"
    }
  }
}
```

**Response Body (Success - 200 OK):**
```rust
/// MFA setup initiation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupInitiateResponse {
    /// Temporary setup token (valid for 5 minutes)
    #[serde(rename = "setup_token")]
    pub setup_token: String,
    
    /// TOTP secret key (base32 encoded)
    /// Security: [SECRET] - Must be protected
    #[serde(rename = "secret")]
    pub secret: String,
    
    /// QR code data URI for authenticator app
    /// Format: otpauth://totp/...
    #[serde(rename = "qr_code_uri")]
    pub qr_code_uri: String,
    
    /// Backup codes for account recovery
    /// Constraints: 10 codes, each 8 characters
    #[serde(rename = "backup_codes")]
    pub backup_codes: Vec<String>,
    
    /// Setup expiration timestamp
    #[serde(rename = "expires_at")]
    pub expires_at: DateTime<Utc>,
    
    /// MFA method
    #[serde(rename = "method")]
    pub method: MfaMethod,
    
    /// Issuer name
    #[serde(rename = "issuer")]
    pub issuer: String,
    
    /// Account name
    #[serde(rename = "account_name")]
    pub account_name: String,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["setup_token", "secret", "qr_code_uri", "backup_codes", "expires_at", "method", "issuer", "account_name"],
  "properties": {
    "setup_token": {
      "type": "string",
      "description": "Temporary setup token"
    },
    "secret": {
      "type": "string",
      "description": "TOTP secret key (base32 encoded)"
    },
    "qr_code_uri": {
      "type": "string",
      "description": "QR code data URI for authenticator app"
    },
    "backup_codes": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "minItems": 10,
      "maxItems": 10,
      "description": "Backup codes for account recovery"
    },
    "expires_at": {
      "type": "string",
      "format": "date-time"
    },
    "method": {
      "type": "string",
      "enum": ["totp", "sms", "hardware"]
    },
    "issuer": {
      "type": "string"
    },
    "account_name": {
      "type": "string"
    }
  }
}
```

**Response Body (Error - 400 Bad Request):**
```json
{
  "code": "AUTH_ERROR_001",
  "message": "Invalid request format",
  "details": {
    "field": "method",
    "error": "Unsupported MFA method"
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554408"
}
```

**Response Body (Error - 409 Conflict):**
```json
{
  "code": "AUTH_ERROR_006",
  "message": "MFA already enabled",
  "details": {
    "mfa_methods": ["totp"],
    "setup_required": false
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554409"
}
```

**Constraints:**
- Requires valid JWT access token
- Setup token expires: 5 minutes
- TOTP secret: 160-bit random value (base32 encoded)
- Backup codes: 10 codes, each 8 characters
- Rate limiting: 3 setup attempts per hour

**Security Considerations:**
- Secret key is generated server-side using cryptographically secure RNG
- Setup token is single-use and expires quickly
- Backup codes are generated using cryptographically secure RNG
- QR code URI follows otpauth:// URI scheme (RFC 6238)
- All MFA setup events are logged

**Dependencies:**
- [REQ-SEC-011](../../.adrs/ Multi-Factor Authentication
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging
- RFC 6238: TOTP specification

**Rationale:** MFA setup initiation provides the necessary information for users to register their authenticator app while maintaining security through short-lived setup tokens.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/api/v1/auth/mfa/setup/initiate \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-4466554410" \
  -d '{
    "method": "totp",
    "device_name": "iPhone 15 Pro"
  }'
```

---

#### 5.1.2. Complete MFA Setup

**Endpoint:** `POST /api/v1/auth/mfa/setup/complete`

**Element ID:** API-AUTH-005
**Related Requirements:** [REQ-SEC-011](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Completes MFA setup by verifying the TOTP code from the authenticator app.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:**
```rust
/// MFA setup completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupCompleteRequest {
    /// Setup token from initiation
    #[serde(rename = "setup_token")]
    pub setup_token: String,
    
    /// TOTP code from authenticator app
    /// Constraints: 6 digits, RFC 6238 compliant
    #[serde(rename = "code")]
    pub code: String,
    
    /// Device name for this MFA factor
    #[serde(rename = "device_name")]
    pub device_name: String,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["setup_token", "code", "device_name"],
  "properties": {
    "setup_token": {
      "type": "string",
      "description": "Setup token from initiation"
    },
    "code": {
      "type": "string",
      "pattern": "^[0-9]{6}$",
      "description": "TOTP code from authenticator app"
    },
    "device_name": {
      "type": "string",
      "description": "Device name for this MFA factor"
    }
  }
}
```

**Response Body (Success - 200 OK):**
```rust
/// MFA setup completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupCompleteResponse {
    /// MFA factor identifier
    #[serde(rename = "factor_id")]
    pub factor_id: Uuid,
    
    /// MFA method
    #[serde(rename = "method")]
    pub method: MfaMethod,
    
    /// Device name
    #[serde(rename = "device_name")]
    pub device_name: String,
    
    /// MFA enabled flag
    #[serde(rename = "mfa_enabled")]
    pub mfa_enabled: bool,
    
    /// Remaining backup codes
    #[serde(rename = "remaining_backup_codes")]
    pub remaining_backup_codes: usize,
    
    /// Setup completion timestamp
    #[serde(rename = "completed_at")]
    pub completed_at: DateTime<Utc>,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["factor_id", "method", "device_name", "mfa_enabled", "remaining_backup_codes", "completed_at"],
  "properties": {
    "factor_id": {
      "type": "string",
      "format": "uuid"
    },
    "method": {
      "type": "string",
      "enum": ["totp", "sms", "hardware"]
    },
    "device_name": {
      "type": "string"
    },
    "mfa_enabled": {
      "type": "boolean"
    },
    "remaining_backup_codes": {
      "type": "integer"
    },
    "completed_at": {
      "type": "string",
      "format": "date-time"
    }
  }
}
```

**Response Body (Error - 400 Bad Request):**
```json
{
  "code": "AUTH_ERROR_001",
  "message": "Invalid TOTP code",
  "details": {
    "attempts_remaining": 2,
    "setup_token_valid": true
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554411"
}
```

**Response Body (Error - 401 Unauthorized):**
```json
{
  "code": "AUTH_ERROR_002",
  "message": "Invalid or expired setup token",
  "details": {
    "expired": true,
    "expires_at": "2026-02-06T02:30:00Z"
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554412"
}
```

**Constraints:**
- Requires valid JWT access token
- Setup token: Must be valid and unexpired
- TOTP code: 6 digits, valid within 1 time step window
- Maximum verification attempts: 3 per setup token
- Rate limiting: 10 verification attempts per hour

**Security Considerations:**
- TOTP code is validated with time-step tolerance (±1 step)
- Setup token is invalidated after successful verification
- Failed verification attempts are logged
- MFA factor is stored with encrypted secret
- Backup codes are stored using Argon2id hashing

**Dependencies:**
- [REQ-SEC-011](../../.adrs/ Multi-Factor Authentication
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging
- RFC 6238: TOTP specification

**Rationale:** MFA setup completion verifies that the user has successfully registered their authenticator app before enabling MFA.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/api/v1/auth/mfa/setup/complete \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-4466554413" \
  -d '{
    "setup_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "code": "123456",
    "device_name": "iPhone 15 Pro"
  }'
```

---

**End of Chunk 4**

---

### 5.2. MFA Verification

#### 5.2.1. Verify MFA Code

**Endpoint:** `POST /api/v1/auth/mfa/verify`

**Element ID:** API-AUTH-006
**Related Requirements:** [REQ-SEC-011](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Verifies MFA code during login or for privileged operations.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:**
```rust
/// MFA verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerifyRequest {
    /// MFA code (TOTP or backup code)
    /// Constraints: 6 digits (TOTP) or 8 characters (backup code)
    #[serde(rename = "code")]
    pub code: String,
    
    /// Verification context
    #[serde(rename = "context")]
    pub context: MfaVerificationContext,
    
    /// Remember this device (skip MFA for future logins)
    #[serde(rename = "remember_device", default)]
    pub remember_device: bool,
    
    /// Device identifier for remembered devices
    #[serde(rename = "device_id")]
    pub device_id: Option<Uuid>,
}

/// MFA verification context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MfaVerificationContext {
    /// Login verification
    #[serde(rename = "login")]
    Login {
        /// Session identifier from initial login
        session_id: Uuid,
    },
    
    /// Privileged operation verification
    #[serde(rename = "privileged")]
    Privileged {
        /// Operation being performed
        operation: String,
        /// Resource identifier (if applicable)
        resource_id: Option<Uuid>,
    },
    
    /// New device verification
    #[serde(rename = "new_device")]
    NewDevice {
        /// Device identifier
        device_id: Uuid,
    },
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["code", "context"],
  "properties": {
    "code": {
      "type": "string",
      "description": "MFA code (TOTP or backup code)"
    },
    "remember_device": {
      "type": "boolean",
      "default": false
    },
    "device_id": {
      "type": "string",
      "format": "uuid"
    },
    "context": {
      "oneOf": [
        {
          "type": "object",
          "required": ["type", "session_id"],
          "properties": {
            "type": {
              "const": "login"
            },
            "session_id": {
              "type": "string",
              "format": "uuid"
            }
          }
        },
        {
          "type": "object",
          "required": ["type", "operation"],
          "properties": {
            "type": {
              "const": "privileged"
            },
            "operation": {
              "type": "string"
            },
            "resource_id": {
              "type": "string",
              "format": "uuid"
            }
          }
        },
        {
          "type": "object",
          "required": ["type", "device_id"],
          "properties": {
            "type": {
              "const": "new_device"
            },
            "device_id": {
              "type": "string",
              "format": "uuid"
            }
          }
        }
      ]
    }
  }
}
```

**Response Body (Success - 200 OK):**
```rust
/// MFA verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerifyResponse {
    /// Verification successful flag
    #[serde(rename = "success")]
    pub success: bool,
    
    /// MFA verified flag for session
    #[serde(rename = "mfa_verified")]
    pub mfa_verified: bool,
    
    /// New access token (if login context)
    #[serde(rename = "access_token")]
    pub access_token: Option<String>,
    
    /// New refresh token (if login context)
    #[serde(rename = "refresh_token")]
    pub refresh_token: Option<String>,
    
    /// Token expiration (if login context)
    #[serde(rename = "expires_in")]
    pub expires_in: Option<u64>,
    
    /// Device remembered flag
    #[serde(rename = "device_remembered")]
    pub device_remembered: bool,
    
    /// Remaining backup codes
    #[serde(rename = "remaining_backup_codes")]
    pub remaining_backup_codes: usize,
    
    /// Verification timestamp
    #[serde(rename = "verified_at")]
    pub verified_at: DateTime<Utc>,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["success", "mfa_verified", "device_remembered", "remaining_backup_codes", "verified_at"],
  "properties": {
    "success": {
      "type": "boolean"
    },
    "mfa_verified": {
      "type": "boolean"
    },
    "access_token": {
      "type": "string"
    },
    "refresh_token": {
      "type": "string"
    },
    "expires_in": {
      "type": "integer"
    },
    "device_remembered": {
      "type": "boolean"
    },
    "remaining_backup_codes": {
      "type": "integer"
    },
    "verified_at": {
      "type": "string",
      "format": "date-time"
    }
  }
}
```

**Response Body (Error - 400 Bad Request):**
```json
{
  "code": "AUTH_ERROR_001",
  "message": "Invalid MFA code",
  "details": {
    "attempts_remaining": 4,
    "code_type": "totp",
    "time_window": "±30 seconds"
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554414"
}
```

**Response Body (Error - 403 Forbidden):**
```json
{
  "code": "AUTH_ERROR_006",
  "message": "MFA not required",
  "details": {
    "mfa_enabled": false,
    "mfa_setup_required": false
  },
  "request_id": "550e8400-e29b-41d4-a716-4466554415"
}
```

**Constraints:**
- Requires valid JWT access token
- TOTP code: 6 digits, valid within ±1 time step (30 seconds)
- Backup code: 8 characters, single-use
- Maximum verification attempts: 5 per session
- Rate limiting: 10 verification attempts per minute
- Device remember: 30 days maximum

**Security Considerations:**
- TOTP code validated with time-step tolerance
- Backup codes are single-use and removed after use
- Failed verification attempts are logged with IP address
- Device remember tokens are cryptographically signed
- MFA verification events are logged for audit

**Dependencies:**
- [REQ-SEC-011](../../.adrs/ Multi-Factor Authentication
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging
- RFC 6238: TOTP specification

**Rationale:** MFA verification provides the second factor of authentication, protecting against credential theft and unauthorized access.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/api/v1/auth/mfa/verify \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-4466554416" \
  -d '{
    "code": "123456",
    "context": {
      "type": "login",
      "session_id": "550e8400-e29b-41d4-a716-4466554417"
    },
    "remember_device": true,
    "device_id": "550e8400-e29b-41d4-a716-4466554418"
  }'
```

---

#### 5.2.2. List MFA Factors

**Endpoint:** `GET /api/v1/auth/mfa/factors`

**Element ID:** API-AUTH-007
**Related Requirements:** [REQ-SEC-011](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Retrieves all MFA factors configured for the authenticated user.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:** None

**Response Body (Success - 200 OK):**
```rust
/// MFA factors list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactorsListResponse {
    /// List of MFA factors
    #[serde(rename = "factors")]
    pub factors: Vec<MfaFactor>,
    
    /// MFA enabled flag
    #[serde(rename = "mfa_enabled")]
    pub mfa_enabled: bool,
    
    /// Remaining backup codes
    #[serde(rename = "remaining_backup_codes")]
    pub remaining_backup_codes: usize,
}

/// MFA factor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactor {
    /// Factor identifier
    #[serde(rename = "factor_id")]
    pub factor_id: Uuid,
    
    /// MFA method
    #[serde(rename = "method")]
    pub method: MfaMethod,
    
    /// Device name
    #[serde(rename = "device_name")]
    pub device_name: String,
    
    /// Factor enabled flag
    #[serde(rename = "enabled")]
    pub enabled: bool,
    
    /// Primary factor flag
    #[serde(rename = "primary")]
    pub primary: bool,
    
    /// Last used timestamp
    #[serde(rename = "last_used_at")]
    pub last_used_at: Option<DateTime<Utc>>,
    
    /// Created timestamp
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["factors", "mfa_enabled", "remaining_backup_codes"],
  "properties": {
    "factors": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/MfaFactor"
      }
    },
    "mfa_enabled": {
      "type": "boolean"
    },
    "remaining_backup_codes": {
      "type": "integer"
    }
  },
  "definitions": {
    "MfaFactor": {
      "type": "object",
      "required": ["factor_id", "method", "device_name", "enabled", "primary", "created_at"],
      "properties": {
        "factor_id": {
          "type": "string",
          "format": "uuid"
        },
        "method": {
          "type": "string",
          "enum": ["totp", "sms", "hardware"]
        },
        "device_name": {
          "type": "string"
        },
        "enabled": {
          "type": "boolean"
        },
        "primary": {
          "type": "boolean"
        },
        "last_used_at": {
          "type": "string",
          "format": "date-time"
        },
        "created_at": {
          "type": "string",
          "format": "date-time"
        }
      }
    }
  }
}
```

**Constraints:**
- Requires valid JWT access token
- Rate limiting: 10 requests per minute per session

**Security Considerations:**
- Only returns factors for authenticated user
- Primary factor is clearly marked
- Last used timestamp shows recent activity
- Backup code count is shown for monitoring

**Dependencies:**
- [REQ-SEC-011](../../.adrs/ Multi-Factor Authentication

**Rationale:** MFA factor listing enables users to manage their MFA devices and identify any unauthorized factors.

**Usage Example:**
```bash
curl -X GET https://api.tachyon.dev/api/v1/auth/mfa/factors \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-4466554419"
```

---

**End of Chunk 5**

---

## 6. TOKEN REFRESH

### 6.1. Refresh Access Token

#### 6.1.1. Refresh Token

**Endpoint:** `POST /api/v1/auth/refresh`

**Element ID:** API-AUTH-008
**Related Requirements:** [REQ-SEC-016](../../.adrs/ [REQ-SEC-018](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Refreshes access token using a valid refresh token. Implements token rotation for enhanced security.

**Request Headers:**
```http
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:**
```rust
/// Token refresh request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshRequest {
    /// Refresh token
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    
    /// Device identifier for session tracking
    #[serde(rename = "device_id")]
    pub device_id: Uuid,
    
    /// Client IP address (optional, server validates)
    #[serde(rename = "client_ip")]
    pub client_ip: Option<String>,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["refresh_token", "device_id"],
  "properties": {
    "refresh_token": {
      "type": "string",
      "description": "Refresh token"
    },
    "device_id": {
      "type": "string",
      "format": "uuid",
      "description": "Device identifier for session tracking"
    },
    "client_ip": {
      "type": "string",
      "format": "ipv4-or-ipv6",
      "description": "Client IP address"
    }
  }
}
```

**Response Headers (Success):**
```http
Content-Type: application/json
X-Request-ID: <uuid>
Cache-Control: no-store, no-cache, must-revalidate
Pragma: no-cache
```

**Response Body (Success - 200 OK):**
```rust
/// Token refresh response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResponse {
    /// New JWT access token
    #[serde(rename = "access_token")]
    pub access_token: String,
    
    /// New JWT refresh token (rotated)
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    
    /// Token type
    #[serde(rename = "token_type")]
    pub token_type: String,  // "Bearer"
    
    /// Access token expiration in seconds
    #[serde(rename = "expires_in")]
    pub expires_in: u64,  // 900
    
    /// Refresh token expiration in seconds
    #[serde(rename = "refresh_expires_in")]
    pub refresh_expires_in: u64,  // 2592000
    
    /// Session identifier
    #[serde(rename = "session_id")]
    pub session_id: Uuid,
    
    /// MFA verified flag
    #[serde(rename = "mfa_verified")]
    pub mfa_verified: bool,
    
    /// User information
    #[serde(rename = "user")]
    pub user: User,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["access_token", "refresh_token", "token_type", "expires_in", "refresh_expires_in", "session_id", "mfa_verified", "user"],
  "properties": {
    "access_token": {
      "type": "string",
      "description": "New JWT access token"
    },
    "refresh_token": {
      "type": "string",
      "description": "New JWT refresh token (rotated)"
    },
    "token_type": {
      "type": "string",
      "enum": ["Bearer"],
      "description": "Token type"
    },
    "expires_in": {
      "type": "integer",
      "description": "Access token expiration in seconds"
    },
    "refresh_expires_in": {
      "type": "integer",
      "description": "Refresh token expiration in seconds"
    },
    "session_id": {
      "type": "string",
      "format": "uuid",
      "description": "Session identifier"
    },
    "mfa_verified": {
      "type": "boolean",
      "description": "MFA verified flag"
    },
    "user": {
      "$ref": "#/definitions/User"
    }
  },
  "definitions": {
    "User": {
      "type": "object",
      "required": ["id", "username", "email", "roles", "status"],
      "properties": {
        "id": {
          "type": "string",
          "format": "uuid"
        },
        "username": {
          "type": "string"
        },
        "email": {
          "type": "string",
          "format": "email"
        },
        "display_name": {
          "type": "string"
        },
        "roles": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "status": {
          "type": "string",
          "enum": ["active", "locked", "pending", "disabled"]
        }
      }
    }
  }
}
```

**Response Body (Error - 401 Unauthorized):**
```json
{
  "code": "AUTH_ERROR_003",
  "message": "Expired refresh token",
  "details": {
    "expired_at": "2026-02-06T03:00:00Z",
    "session_id": "550e8400-e29b-41d4-a716-44665544420"
  },
  "request_id": "550e8400-e29b-41d4-a716-44665544421"
}
```

**Response Body (Error - 403 Forbidden):**
```json
{
  "code": "AUTH_ERROR_007",
  "message": "Session expired or revoked",
  "details": {
    "session_id": "550e8400-e29b-41d4-a716-44665544422",
    "revoked": true
  },
  "request_id": "550e8400-e29b-41d4-a716-44665544423"
}
```

**Response Body (Error - 429 Too Many Requests):**
```json
{
  "code": "AUTH_ERROR_008",
  "message": "Rate limit exceeded",
  "details": {
    "retry_after": 60,
    "limit": "30 refreshes per hour"
  },
  "request_id": "550e8400-e29b-41d4-a716-44665544424"
}
```

**Constraints:**
- Refresh token expiration: 30 days from issuance
- Access token expiration: 15 minutes from issuance
- Token rotation: Required on each refresh (old token invalidated)
- Rate limiting: 30 refresh requests per hour per session
- Maximum refresh attempts: 100 per refresh token lifetime

**Security Considerations:**
- Token rotation prevents replay attacks
- Old refresh token is invalidated immediately after successful refresh
- Refresh token is validated against session and device
- IP address changes trigger additional security checks
- Failed refresh attempts are logged

**Dependencies:**
- [REQ-SEC-016](../../.adrs/ Secure Session Tokens
- [REQ-SEC-018](../../.adrs/ Session Refresh
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging
- RFC 7519: JWT specification

**Rationale:** Token refresh with rotation provides a balance between security and usability, allowing long-lived sessions while limiting token exposure.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-44665544425" \
  -d '{
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "device_id": "550e8400-e29b-41d4-a716-44665544426"
  }'
```

---

### 6.2. Revoke Refresh Token

#### 6.2.1. Revoke Token

**Endpoint:** `POST /api/v1/auth/revoke`

**Element ID:** API-AUTH-009
**Related Requirements:** [REQ-SEC-020](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Revokes a refresh token and associated session. Used for explicit logout and security incidents.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:**
```rust
/// Token revocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRevokeRequest {
    /// Refresh token to revoke
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    
    /// Revocation reason
    #[serde(rename = "reason")]
    pub reason: RevokeReason,
    
    /// Additional context for security audit
    #[serde(rename = "context")]
    pub context: Option<String>,
}

/// Revocation reason enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevokeReason {
    /// User logout
    Logout,
    /// Security incident
    Security,
    /// Suspicious activity
    SuspiciousActivity,
    /// Password change
    PasswordChange,
    /// Account deletion
    AccountDeletion,
    /// Other reason
    Other,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["refresh_token", "reason"],
  "properties": {
    "refresh_token": {
      "type": "string",
      "description": "Refresh token to revoke"
    },
    "reason": {
      "type": "string",
      "enum": ["logout", "security", "suspicious_activity", "password_change", "account_deletion", "other"],
      "description": "Revocation reason"
    },
    "context": {
      "type": "string",
      "description": "Additional context for security audit"
    }
  }
}
```

**Response Body (Success - 200 OK):**
```rust
/// Token revocation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRevokeResponse {
    /// Revocation successful flag
    #[serde(rename = "success")]
    pub success: bool,
    
    /// Revoked session identifier
    #[serde(rename = "session_id")]
    pub session_id: Uuid,
    
    /// Revocation timestamp
    #[serde(rename = "revoked_at")]
    pub revoked_at: DateTime<Utc>,
    
    /// Revocation reason
    #[serde(rename = "reason")]
    pub reason: RevokeReason,
    
    /// All tokens revoked flag
    #[serde(rename = "all_tokens_revoked")]
    pub all_tokens_revoked: bool,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["success", "session_id", "revoked_at", "reason", "all_tokens_revoked"],
  "properties": {
    "success": {
      "type": "boolean"
    },
    "session_id": {
      "type": "string",
      "format": "uuid"
    },
    "revoked_at": {
      "type": "string",
      "format": "date-time"
    },
    "reason": {
      "type": "string",
      "enum": ["logout", "security", "suspicious_activity", "password_change", "account_deletion", "other"]
    },
    "all_tokens_revoked": {
      "type": "boolean"
    }
  }
}
```

**Response Body (Error - 401 Unauthorized):**
```json
{
  "code": "AUTH_ERROR_002",
  "message": "Invalid refresh token",
  "details": {
    "token_type": "refresh",
    "session_id": "550e8400-e29b-41d4-a716-44665544427"
  },
  "request_id": "550e8400-e29b-41d4-a716-44665544428"
}
```

**Constraints:**
- Requires valid JWT access token
- Refresh token must belong to authenticated user
- Rate limiting: 20 revocation requests per minute per session

**Security Considerations:**
- Refresh token is immediately invalidated
- Associated access tokens are also invalidated
- Session is marked as revoked
- Revocation reason is logged for audit
- Security-related revocations trigger alerts

**Dependencies:**
- [REQ-SEC-020](../../.adrs/ Session Revocation
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging

**Rationale:** Token revocation provides explicit control over session lifecycle, enabling users to log out and administrators to respond to security incidents.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/api/v1/auth/revoke \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-44665544429" \
  -d '{
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "reason": "logout"
  }'
```

---

**End of Chunk 6**

---

## 7. LOGOUT

### 7.1. Logout

#### 7.1.1. Logout

**Endpoint:** `POST /api/v1/auth/logout`

**Element ID:** API-AUTH-010
**Related Requirements:** [REQ-SEC-020](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Logs out the current session, invalidating access and refresh tokens.

**Request Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:**
```rust
/// Logout request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutRequest {
    /// Logout all devices flag
    #[serde(rename = "logout_all", default)]
    pub logout_all: bool,
    
    /// Logout reason
    #[serde(rename = "reason")]
    pub reason: LogoutReason,
}

/// Logout reason enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogoutReason {
    /// User-initiated logout
    UserInitiated,
    /// Session timeout
    Timeout,
    /// Security logout
    Security,
    /// Password change
    PasswordChange,
    /// Account deletion
    AccountDeletion,
    /// Other reason
    Other { description: String },
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["reason"],
  "properties": {
    "logout_all": {
      "type": "boolean",
      "default": false,
      "description": "Logout all devices flag"
    },
    "reason": {
      "oneOf": [
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "user_initiated"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "timeout"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "security"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "password_change"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "account_deletion"
            }
          }
        },
        {
          "type": "object",
          "required": ["type", "description"],
          "properties": {
            "type": {
              "const": "other"
            },
            "description": {
              "type": "string"
            }
          }
        }
      ]
    }
  }
}
```

**Response Body (Success - 200 OK):**
```rust
/// Logout response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    /// Logout successful flag
    #[serde(rename = "success")]
    pub success: bool,
    
    /// Logged out session identifier
    #[serde(rename = "session_id")]
    pub session_id: Uuid,
    
    /// Logout timestamp
    #[serde(rename = "logged_out_at")]
    pub logged_out_at: DateTime<Utc>,
    
    /// All sessions logged out flag
    #[serde(rename = "all_sessions_logged_out")]
    pub all_sessions_logged_out: bool,
    
    /// Number of sessions logged out
    #[serde(rename = "sessions_logged_out")]
    pub sessions_logged_out: usize,
    
    /// Logout reason
    #[serde(rename = "reason")]
    pub reason: LogoutReason,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["success", "session_id", "logged_out_at", "all_sessions_logged_out", "sessions_logged_out", "reason"],
  "properties": {
    "success": {
      "type": "boolean"
    },
    "session_id": {
      "type": "string",
      "format": "uuid"
    },
    "logged_out_at": {
      "type": "string",
      "format": "date-time"
    },
    "all_sessions_logged_out": {
      "type": "boolean"
    },
    "sessions_logged_out": {
      "type": "integer"
    },
    "reason": {
      "oneOf": [
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "user_initiated"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "timeout"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "security"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "password_change"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "account_deletion"
            }
          }
        },
        {
          "type": "object",
          "required": ["type", "description"],
          "properties": {
            "type": {
              "const": "other"
            },
            "description": {
              "type": "string"
            }
          }
        }
      ]
    }
  }
}
```

**Response Body (Error - 401 Unauthorized):**
```json
{
  "code": "AUTH_ERROR_003",
  "message": "Invalid or expired token",
  "details": {
    "token_type": "access",
    "session_id": "550e8400-e29b-41d4-a716-44665544430"
  },
  "request_id": "550e8400-e29b-41d4-a716-44665544431"
}
```

**Constraints:**
- Requires valid JWT access token
- Rate limiting: 20 logout requests per minute per session
- Logout all: Maximum 10 sessions per user

**Security Considerations:**
- Access token is immediately invalidated
- Refresh token is immediately invalidated
- Session is marked as logged out
- Logout all invalidates all user sessions
- Logout events are logged for audit

**Dependencies:**
- [REQ-SEC-020](../../.adrs/ Session Revocation
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging

**Rationale:** Logout provides explicit session termination, enabling users to control their session lifecycle and administrators to respond to security incidents.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/api/v1/auth/logout \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-44665544432" \
  -d '{
    "logout_all": false,
    "reason": {
      "type": "user_initiated"
    }
  }'
```

---

**End of Chunk 7**

---

## 8. OAUTH 2.0 INTEGRATION

### 8.1. OAuth 2.0 Authorization

#### 8.1.1. OAuth 2.0 Authorization Endpoint

**Endpoint:** `GET /oauth2/authorize`

**Element ID:** API-AUTH-011
**Related Requirements:** [REQ-SEC-013](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** OAuth 2.0 authorization endpoint for initiating federated authentication flow.

**Request Query Parameters:**
```rust
/// OAuth 2.0 authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2AuthorizeRequest {
    /// OAuth 2.0 response type
    /// Constraints: Must be "code"
    #[serde(rename = "response_type")]
    pub response_type: String,
    
    /// OAuth 2.0 client identifier
    /// Constraints: Valid client ID
    #[serde(rename = "client_id")]
    pub client_id: String,
    
    /// OAuth 2.0 redirect URI
    /// Constraints: Must match registered redirect URI
    #[serde(rename = "redirect_uri")]
    pub redirect_uri: String,
    
    /// OAuth 2.0 scope(s)
    /// Constraints: Space-delimited, valid scopes
    #[serde(rename = "scope")]
    pub scope: String,
    
    /// OAuth 2.0 state parameter (CSRF protection)
    /// Constraints: 1-256 characters
    #[serde(rename = "state")]
    pub state: String,
    
    /// OAuth 2.0 code challenge (PKCE)
    /// Constraints: 43-128 characters, URL-safe base64
    #[serde(rename = "code_challenge")]
    pub code_challenge: String,
    
    /// OAuth 2.0 code challenge method (PKCE)
    /// Constraints: "S256" or "plain"
    #[serde(rename = "code_challenge_method")]
    pub code_challenge_method: String,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["response_type", "client_id", "redirect_uri", "scope", "state", "code_challenge", "code_challenge_method"],
  "properties": {
    "response_type": {
      "type": "string",
      "enum": ["code"],
      "description": "OAuth 2.0 response type"
    },
    "client_id": {
      "type": "string",
      "description": "OAuth 2.0 client identifier"
    },
    "redirect_uri": {
      "type": "string",
      "format": "uri",
      "description": "OAuth 2.0 redirect URI"
    },
    "scope": {
      "type": "string",
      "description": "OAuth 2.0 scope(s)"
    },
    "state": {
      "type": "string",
      "minLength": 1,
      "maxLength": 256,
      "description": "OAuth 2.0 state parameter (CSRF protection)"
    },
    "code_challenge": {
      "type": "string",
      "minLength": 43,
      "maxLength": 128,
      "description": "OAuth 2.0 code challenge (PKCE)"
    },
    "code_challenge_method": {
      "type": "string",
      "enum": ["S256", "plain"],
      "description": "OAuth 2.0 code challenge method (PKCE)"
    }
  }
}
```

**Response (Redirect):**
```
HTTP/1.1 302 Found
Location: https://client.example.com/callback?code=<authorization_code>&state=<state>
```

**Response (Error - 400 Bad Request):**
```
HTTP/1.1 302 Found
Location: https://client.example.com/callback?error=invalid_request&error_description=Invalid+request&state=<state>
```

**Constraints:**
- Response type: Must be "code"
- Client ID: Must be registered
- Redirect URI: Must match registered URI
- Scope: Valid scope(s), space-delimited
- State: 1-256 characters
- Code challenge: 43-128 characters (PKCE required for public clients)
- Code challenge method: "S256" or "plain"

**Security Considerations:**
- PKCE (Proof Key for Code Exchange) required for public clients
- State parameter validated for CSRF protection
- Redirect URI validated against registered URIs
- Authorization code is single-use and expires in 10 minutes
- OAuth 2.0 events are logged for audit

**Dependencies:**
- [REQ-SEC-013](../../.adrs/ OAuth 2.0 Support
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging
- RFC 6749: OAuth 2.0 Authorization Framework
- RFC 7636: PKCE for Public Clients

**Rationale:** OAuth 2.0 authorization enables federated authentication with external identity providers while maintaining security through PKCE and state validation.

**Usage Example:**
```bash
curl -L "https://api.tachyon.dev/oauth2/authorize?response_type=code&client_id=abc123&redirect_uri=https%3A%2Fclient.example.com%2Fcallback&scope=read%20write&state=xyz789&code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWtuGv2B&code_challenge_method=S256"
```

---

#### 8.1.2. OAuth 2.0 Token Endpoint

**Endpoint:** `POST /oauth2/token`

**Element ID:** API-AUTH-012
**Related Requirements:** [REQ-SEC-013](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** OAuth 2.0 token endpoint for exchanging authorization codes for access tokens.

**Request Headers:**
```http
Content-Type: application/x-www-form-urlencoded
```

**Request Body:**
```rust
/// OAuth 2.0 token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenRequest {
    /// OAuth 2.0 grant type
    /// Constraints: "authorization_code" or "refresh_token"
    #[serde(rename = "grant_type")]
    pub grant_type: String,
    
    /// OAuth 2.0 authorization code (for authorization_code grant)
    #[serde(rename = "code")]
    pub code: Option<String>,
    
    /// OAuth 2.0 redirect URI (for authorization_code grant)
    #[serde(rename = "redirect_uri")]
    pub redirect_uri: Option<String>,
    
    /// OAuth 2.0 client identifier
    #[serde(rename = "client_id")]
    pub client_id: String,
    
    /// OAuth 2.0 client secret (for confidential clients)
    #[serde(rename = "client_secret")]
    pub client_secret: Option<String>,
    
    /// OAuth 2.0 refresh token (for refresh_token grant)
    #[serde(rename = "refresh_token")]
    pub refresh_token: Option<String>,
    
    /// OAuth 2.0 code verifier (PKCE)
    #[serde(rename = "code_verifier")]
    pub code_verifier: Option<String>,
}
```

**Request Body (Form-Encoded):**
```
grant_type=authorization_code
&code=SplxlOBeZQQYbYS6WxSbKx
&redirect_uri=https://client.example.com/callback
&client_id=abc123
&code_verifier=dBjFTJe2R8SMYmY6QjW5jG
```

**Response Body (Success - 200 OK):**
```rust
/// OAuth 2.0 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    /// OAuth 2.0 access token
    #[serde(rename = "access_token")]
    pub access_token: String,
    
    /// OAuth 2.0 token type
    #[serde(rename = "token_type")]
    pub token_type: String,  // "Bearer"
    
    /// OAuth 2.0 expires in (seconds)
    #[serde(rename = "expires_in")]
    pub expires_in: u64,  // 900
    
    /// OAuth 2.0 refresh token
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    
    /// OAuth 2.0 scope
    #[serde(rename = "scope")]
    pub scope: String,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["access_token", "token_type", "expires_in", "refresh_token", "scope"],
  "properties": {
    "access_token": {
      "type": "string",
      "description": "OAuth 2.0 access token"
    },
    "token_type": {
      "type": "string",
      "enum": ["Bearer"],
      "description": "OAuth 2.0 token type"
    },
    "expires_in": {
      "type": "integer",
      "description": "OAuth 2.0 expires in (seconds)"
    },
    "refresh_token": {
      "type": "string",
      "description": "OAuth 2.0 refresh token"
    },
    "scope": {
      "type": "string",
      "description": "OAuth 2.0 scope"
    }
  }
}
```

**Response Body (Error - 400 Bad Request):**
```json
{
  "error": "invalid_request",
  "error_description": "Missing required parameter: code"
}
```

**Response Body (Error - 401 Unauthorized):**
```json
{
  "error": "invalid_client",
  "error_description": "Invalid client credentials"
}
```

**Constraints:**
- Grant type: "authorization_code" or "refresh_token"
- Authorization code: Single-use, expires in 10 minutes
- Refresh token: 30 days from issuance
- Code verifier: Required for PKCE (43-128 characters)
- Rate limiting: 20 token requests per minute per client

**Security Considerations:**
- PKCE code verifier validated against code challenge
- Client secret validated for confidential clients
- Authorization code is single-use and invalidated after use
- Token rotation enforced for refresh token grants
- OAuth 2.0 events are logged for audit

**Dependencies:**
- [REQ-SEC-013](../../.adrs/ OAuth 2.0 Support
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging
- RFC 6749: OAuth 2.0 Authorization Framework
- RFC 7636: PKCE for Public Clients

**Rationale:** OAuth 2.0 token endpoint provides secure token exchange while maintaining security through PKCE and client validation.

**Usage Example:**
```bash
curl -X POST https://api.tachyon.dev/oauth2/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d 'grant_type=authorization_code&code=SplxlOBeZQQYbYS6WxSbKx&redirect_uri=https%3A%2Fclient.example.com%2Fcallback&client_id=abc123&code_verifier=dBjFTJe2R8SMYmY6QjW5jG'
```

---

### 8.2. OAuth 2.0 Provider Configuration

#### 8.2.1. List OAuth 2.0 Providers

**Endpoint:** `GET /api/v1/auth/oauth2/providers`

**Element ID:** API-AUTH-013
**Related Requirements:** [REQ-SEC-013](../../.adrs/
**Related Design Elements:** [DES-SEC-001](../../.adrs/

**Description:** Retrieves list of configured OAuth 2.0 identity providers.

**Request Headers:**
```http
Content-Type: application/json
X-Request-ID: <uuid>
```

**Request Body:** None

**Response Body (Success - 200 OK):**
```rust
/// OAuth 2.0 providers list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2ProvidersListResponse {
    /// List of OAuth 2.0 providers
    #[serde(rename = "providers")]
    pub providers: Vec<OAuth2Provider>,
}

/// OAuth 2.0 provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Provider {
    /// Provider identifier
    #[serde(rename = "provider_id")]
    pub provider_id: String,
    
    /// Provider name
    #[serde(rename = "name")]
    pub name: String,
    
    /// Provider display name
    #[serde(rename = "display_name")]
    pub display_name: String,
    
    /// Provider logo URL
    #[serde(rename = "logo_url")]
    pub logo_url: Option<String>,
    
    /// OAuth 2.0 authorization endpoint
    #[serde(rename = "authorization_endpoint")]
    pub authorization_endpoint: String,
    
    /// OAuth 2.0 token endpoint
    #[serde(rename = "token_endpoint")]
    pub token_endpoint: String,
    
    /// OAuth 2.0 scopes
    #[serde(rename = "scopes")]
    pub scopes: Vec<String>,
    
    /// Provider enabled flag
    #[serde(rename = "enabled")]
    pub enabled: bool,
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["providers"],
  "properties": {
    "providers": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/OAuth2Provider"
      }
    }
  },
  "definitions": {
    "OAuth2Provider": {
      "type": "object",
      "required": ["provider_id", "name", "display_name", "authorization_endpoint", "token_endpoint", "scopes", "enabled"],
      "properties": {
        "provider_id": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "display_name": {
          "type": "string"
        },
        "logo_url": {
          "type": "string",
          "format": "uri"
        },
        "authorization_endpoint": {
          "type": "string",
          "format": "uri"
        },
        "token_endpoint": {
          "type": "string",
          "format": "uri"
        },
        "scopes": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "enabled": {
          "type": "boolean"
        }
      }
    }
  }
}
```

**Constraints:**
- Rate limiting: 30 requests per minute per IP

**Security Considerations:**
- Only enabled providers are returned
- Provider endpoints are validated
- Sensitive information (client secrets) is not exposed

**Dependencies:**
- [REQ-SEC-013](../../.adrs/ OAuth 2.0 Support

**Rationale:** OAuth 2.0 provider listing enables clients to discover available identity providers for federated authentication.

**Usage Example:**
```bash
curl -X GET https://api.tachyon.dev/api/v1/auth/oauth2/providers \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: 550e8400-e29b-41d4-a716-44665544433"
```

---

**End of Chunk 8**

---

## 9. AUTHENTICATION SECURITY

### 9.1. Rate Limiting

#### 9.1.1. Rate Limiting Strategy

The Authentication API implements multi-layered rate limiting to prevent brute-force attacks and ensure service availability:

**Rate Limiting Tiers:**

| Tier | Scope | Limit | Window | Enforcement |
|------|-------|-------|-------------|
| 1 | Per IP address | 10 requests/minute | Sliding window |
| 2 | Per user account | 30 requests/minute | Sliding window |
| 3 | Per session | 100 requests/minute | Sliding window |
| 4 | Per client ID | 50 requests/minute | Sliding window |

**Rate Limiting Headers:**
```http
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
X-RateLimit-Reset: 16441234567
X-RateLimit-Used: 3
```

**Rate Limiting Response (429 Too Many Requests):**
```rust
/// Rate limit exceeded response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitExceededResponse {
    /// Error code
    #[serde(rename = "code")]
    pub code: String,  // "AUTH_ERROR_008"
    
    /// Error message
    #[serde(rename = "message")]
    pub message: String,  // "Rate limit exceeded"
    
    /// Error details
    #[serde(rename = "details")]
    pub details: RateLimitDetails,
    
    /// Request identifier
    #[serde(rename = "request_id")]
    pub request_id: String,
}

/// Rate limit details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitDetails {
    /// Retry after (seconds)
    #[serde(rename = "retry_after")]
    pub retry_after: u64,
    
    /// Rate limit
    #[serde(rename = "limit")]
    pub limit: u64,
    
    /// Rate limit window
    #[serde(rename = "window")]
    pub window: u64,
    
    /// Requests used
    #[serde(rename = "used")]
    pub used: u64,
    
    /// Rate limit scope
    #[serde(rename = "scope")]
    pub scope: String,  // "ip_address", "user", "session", "client"
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["code", "message", "details", "request_id"],
  "properties": {
    "code": {
      "type": "string",
      "const": "AUTH_ERROR_008"
    },
    "message": {
      "type": "string",
      "const": "Rate limit exceeded"
    },
    "details": {
      "$ref": "#/definitions/RateLimitDetails"
    },
    "request_id": {
      "type": "string",
      "format": "uuid"
    }
  },
  "definitions": {
    "RateLimitDetails": {
      "type": "object",
      "required": ["retry_after", "limit", "window", "used", "scope"],
      "properties": {
        "retry_after": {
          "type": "integer",
          "description": "Retry after (seconds)"
        },
        "limit": {
          "type": "integer",
          "description": "Rate limit"
        },
        "window": {
          "type": "integer",
          "description": "Rate limit window (seconds)"
        },
        "used": {
          "type": "integer",
          "description": "Requests used"
        },
        "scope": {
          "type": "string",
          "enum": ["ip_address", "user", "session", "client"],
          "description": "Rate limit scope"
        }
      }
    }
  }
}
```

**Rate Limiting Configuration:**

```rust
/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// IP address rate limit
    #[serde(rename = "ip_address_limit")]
    pub ip_address_limit: u64,  // 10
    
    /// IP address rate limit window (seconds)
    #[serde(rename = "ip_address_window")]
    pub ip_address_window: u64,  // 60
    
    /// User rate limit
    #[serde(rename = "user_limit")]
    pub user_limit: u64,  // 30
    
    /// User rate limit window (seconds)
    #[serde(rename = "user_window")]
    pub user_window: u64,  // 60
    
    /// Session rate limit
    #[serde(rename = "session_limit")]
    pub session_limit: u64,  // 100
    
    /// Session rate limit window (seconds)
    #[serde(rename = "session_window")]
    pub session_window: u64,  // 60
    
    /// Client rate limit
    #[serde(rename = "client_limit")]
    pub client_limit: u64,  // 50
    
    /// Client rate limit window (seconds)
    #[serde(rename = "client_window")]
    pub client_window: u64,  // 60
    
    /// Rate limit burst size
    #[serde(rename = "burst_size")]
    pub burst_size: u64,  // 20
    
    /// Rate limit algorithm
    #[serde(rename = "algorithm")]
    pub algorithm: RateLimitAlgorithm,
}

/// Rate limit algorithm enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitAlgorithm {
    /// Token bucket algorithm
    TokenBucket,
    /// Leaky bucket algorithm
    LeakyBucket,
    /// Sliding window algorithm
    SlidingWindow,
    /// Fixed window algorithm
    FixedWindow,
}
```

**Security Considerations:**
- Rate limiting enforced at multiple layers (IP, user, session, client)
- Sliding window algorithm prevents burst attacks
- Rate limit headers enable client-side throttling
- Rate limit violations are logged for security monitoring
- Trusted IPs (e.g., internal services) may have higher limits

**Dependencies:**
- [REQ-SEC-068](../../.adrs/ Alerting
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging

**Rationale:** Multi-layered rate limiting protects against brute-force attacks while ensuring legitimate users are not overly constrained.

---

### 9.2. DDoS Protection

#### 9.2.1. DDoS Mitigation Strategy

The Authentication API implements comprehensive DDoS protection to ensure service availability:

**DDoS Protection Layers:**

| Layer | Mechanism | Threshold | Action |
|-------|-----------|-----------|--------|
| 1 | IP reputation | Low score | Challenge/Block |
| 2 | Geographic blocking | High-risk regions | Block |
| 3 | Request rate | >1000 req/sec | Throttle |
| 4 | Connection rate | >500 conn/sec | Throttle |
| 5 | Resource usage | >80% CPU/Mem | Throttle |

**DDoS Challenge Response:**
```rust
/// DDoS challenge response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosChallengeResponse {
    /// Challenge type
    #[serde(rename = "challenge_type")]
    pub challenge_type: DdosChallengeType,
    
    /// Challenge token
    #[serde(rename = "challenge_token")]
    pub challenge_token: String,
    
    /// Challenge expiration
    #[serde(rename = "expires_at")]
    pub expires_at: DateTime<Utc>,
    
    /// Challenge data
    #[serde(rename = "challenge_data")]
    pub challenge_data: serde_json::Value,
}

/// DDoS challenge type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DdosChallengeType {
    /// JavaScript challenge
    Javascript,
    /// CAPTCHA challenge
    Captcha,
    /// Cookie challenge
    Cookie,
    /// Wait challenge
    Wait { seconds: u64 },
}
```

**JSON Schema:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["challenge_type", "challenge_token", "expires_at", "challenge_data"],
  "properties": {
    "challenge_type": {
      "oneOf": [
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "javascript"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "captcha"
            }
          }
        },
        {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "const": "cookie"
            }
          }
        },
        {
          "type": "object",
          "required": ["type", "seconds"],
          "properties": {
            "type": {
              "const": "wait"
            },
            "seconds": {
              "type": "integer"
            }
          }
        }
      ]
    },
    "challenge_token": {
      "type": "string"
    },
    "expires_at": {
      "type": "string",
      "format": "date-time"
    },
    "challenge_data": {
      "type": "object"
    }
  }
}
```

**DDoS Blocked Response:**
```http
HTTP/1.1 403 Forbidden
X-Request-ID: <uuid>
Content-Type: application/json

{
  "code": "AUTH_ERROR_009",
  "message": "Service temporarily unavailable",
  "details": {
    "reason": "ddos_protection",
    "retry_after": 300
  },
  "request_id": "550e8400-e29b-41d4-a716-44665544434"
}
```

**DDoS Protection Configuration:**

```rust
/// DDoS protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosProtectionConfig {
    /// IP reputation enabled
    #[serde(rename = "ip_reputation_enabled")]
    pub ip_reputation_enabled: bool,
    
    /// Geographic blocking enabled
    #[serde(rename = "geo_blocking_enabled")]
    pub geo_blocking_enabled: bool,
    
    /// Blocked regions (ISO 3166-1 alpha-2)
    #[serde(rename = "blocked_regions")]
    pub blocked_regions: Vec<String>,
    
    /// Request rate threshold (requests per second)
    #[serde(rename = "request_rate_threshold")]
    pub request_rate_threshold: u64,  // 1000
    
    /// Connection rate threshold (connections per second)
    #[serde(rename = "connection_rate_threshold")]
    pub connection_rate_threshold: u64,  // 500
    
    /// Resource usage threshold (percentage)
    #[serde(rename = "resource_usage_threshold")]
    pub resource_usage_threshold: u64,  // 80
    
    /// Challenge enabled
    #[serde(rename = "challenge_enabled")]
    pub challenge_enabled: bool,
    
    /// Challenge types
    #[serde(rename = "challenge_types")]
    pub challenge_types: Vec<DdosChallengeType>,
    
    /// Auto-block threshold (violations per minute)
    #[serde(rename = "auto_block_threshold")]
    pub auto_block_threshold: u64,  // 100
    
    /// Auto-block duration (seconds)
    #[serde(rename = "auto_block_duration")]
    pub auto_block_duration: u64,  // 3600
}
```

**Security Considerations:**
- DDoS protection operates at network and application layers
- IP reputation scoring uses threat intelligence feeds
- Geographic blocking for high-risk regions
- JavaScript challenge verifies browser execution
- CAPTCHA challenge for suspicious traffic
- Trusted IPs (e.g., CDN, internal services) bypass challenges
- DDoS events trigger security alerts

**Dependencies:**
- [REQ-SEC-068](../../.adrs/ Alerting
- [REQ-SEC-061](../../.adrs/ Authentication Events Logging

**Rationale:** Comprehensive DDoS protection ensures service availability during attack scenarios while minimizing impact on legitimate users.

---

### 9.3. Security Headers

#### 9.3.1. Security Response Headers

The Authentication API includes security headers to enhance client-side security:

**Security Headers:**
```http
# Content Security Policy
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https:; frame-ancestors 'none'; frame-src 'none'; base-uri 'self'; form-action 'self';

# HTTP Strict Transport Security
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload

# X-Content-Type-Options
X-Content-Type-Options: nosniff

# X-Frame-Options
X-Frame-Options: DENY

# X-XSS-Protection
X-XSS-Protection: 1; mode=block

# Referrer-Policy
Referrer-Policy: strict-origin-when-cross-origin

# Permissions-Policy
Permissions-Policy: geolocation=(), microphone=(), camera=()

# Cross-Origin-Opener-Policy
Cross-Origin-Opener-Policy: same-origin

# Cross-Origin-Resource-Policy
Cross-Origin-Resource-Policy: same-origin
```

**Security Considerations:**
- CSP headers prevent XSS attacks
- HSTS enforces HTTPS connections
- X-Frame-Options prevents clickjacking
- X-XSS-Protection provides XSS filtering
- Referrer-Policy controls referrer information leakage
- Permissions-Policy restricts browser APIs
- Cross-Origin headers prevent tab-nabbing attacks

**Dependencies:**
- [REQ-SEC-050](../../.adrs/ Content Security Policy
- [REQ-SEC-033](../../.adrs/ HSTS Headers
- [REQ-SEC-075](../../.adrs/ Secure Headers

**Rationale:** Security headers provide defense-in-depth by protecting against client-side attacks and enforcing secure browser behavior.

---

### 9.4. Audit Logging

#### 9.4.1. Authentication Event Logging

All authentication events are logged for security monitoring and compliance:

**Event Types:**
| Event Type | Description | Severity |
|------------|-------------|----------|
| `auth_login_success` | Successful login | Info |
| `auth_login_failure` | Failed login | Warning |
| `auth_logout` | User logout | Info |
| `auth_token_refresh` | Token refresh | Info |
| `auth_mfa_setup` | MFA setup | Info |
| `auth_mfa_verify` | MFA verification | Info |
| `auth_mfa_failure` | MFA failure | Warning |
| `auth_session_created` | Session created | Info |
| `auth_session_revoked` | Session revoked | Warning |
| `auth_rate_limit_exceeded` | Rate limit exceeded | Warning |
| `auth_ddos_challenge` | DDoS challenge | Warning |
| `auth_oauth_callback` | OAuth callback | Info |

**Event Schema:**
```rust
/// Authentication event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    /// Event identifier
    #[serde(rename = "event_id")]
    pub event_id: Uuid,
    
    /// Event type
    #[serde(rename = "event_type")]
    pub event_type: String,
    
    /// Event timestamp
    #[serde(rename = "timestamp")]
    pub timestamp: DateTime<Utc>,
    
    /// User identifier (if applicable)
    #[serde(rename = "user_id")]
    pub user_id: Option<Uuid>,
    
    /// Session identifier (if applicable)
    #[serde(rename = "session_id")]
    pub session_id: Option<Uuid>,
    
    /// Client IP address
    #[serde(rename = "client_ip")]
    pub client_ip: String,
    
    /// User agent
    #[serde(rename = "user_agent")]
    pub user_agent: String,
    
    /// Request identifier
    #[serde(rename = "request_id")]
    pub request_id: String,
    
    /// Event severity
    #[serde(rename = "severity")]
    pub severity: EventSeverity,
    
    /// Event data
    #[serde(rename = "data")]
    pub data: serde_json::Value,
}

/// Event severity enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    /// Debug severity
    Debug,
    /// Info severity
    Info,
    /// Warning severity
    Warning,
    /// Error severity
    Error,
    /// Critical severity
    Critical,
}
```

**Security Considerations:**
- All authentication events are logged with full context
- Events are stored in write-once, read-many storage
- Events are cryptographically signed to prevent tampering
- Events are retained for minimum 90 days
- Event access is restricted to authorized personnel
- High-severity events trigger security alerts

**Dependencies:**
- [REQ-SEC-056](../../.adrs/ Comprehensive Logging
- [REQ-SEC-057](../../.adrs/ Immutable Logs
- [REQ-SEC-058](../../.adrs/ Log Tamper Protection
- [REQ-SEC-059](../../.adrs/ Log Retention
- [REQ-SEC-060](../../.adrs/ Log Access

**Rationale:** Comprehensive audit logging enables incident response, compliance, and security monitoring.

---

**End of Chunk 9**

---

## 10. REFERENCES

### 10.1. Standards and Specifications

| Reference | Title | Organization | Year | URL |
|-----------|-------|--------------|------|-----|
| RFC 6749 | The OAuth 2.0 Authorization Framework | IETF | 2012 | https://tools.ietf.org/html/rfc6749 |
| RFC 6238 | TOTP: Time-Based One-Time Password Algorithm | IETF | 2011 | https://tools.ietf.org/html/rfc6238 |
| RFC 7519 | JSON Web Token (JWT) | IETF | 2015 | https://tools.ietf.org/html/rfc7519 |
| RFC 7636 | Proof Key for Code Exchange by OAuth Public Clients | IETF | 2015 | https://tools.ietf.org/html/rfc7636 |
| RFC 8259 | The JavaScript Object Notation (JSON) Data Interchange Format | IETF | 2017 | https://tools.ietf.org/html/rfc8259 |
| RFC 7540 | HTTP/2 | IETF | 2015 | https://tools.ietf.org/html/rfc7540 |
| RFC 8446 | The Transport Layer Security (TLS) Protocol Version 1.3 | IETF | 2018 | https://tools.ietf.org/html/rfc8446 |
| ISO/IEC 27001:2013 | Information Technology - Security Techniques - Information Security Management Systems | ISO/IEC | 2013 | https://www.iso.org/standard/27001 |
| ISO/IEC 26514:2021 | Systems and Software Engineering - Requirements for Designers and Developers of User Documentation | ISO/IEC | 2021 | https://www.iso.org/standard/26514 |
| IEEE 830-1998 | IEEE Recommended Practice for Software Requirements Specifications | IEEE | 1998 | https://standards.ieee.org/standard/830 |
| NIST SP 800-63B | Digital Identity Guidelines | NIST | 2017 | https://csrc.nist.gov/publications/detail/sp/800-63b |
| OWASP Top 10 | OWASP Top 10 Web Application Security Risks | OWASP | 2021 | https://owasp.org/Top10 |

### 10.2. Project Documentation

| Reference | Title | Version | Date | Path |
|-----------|-------|--------|------|------|
| TACHYON-STD-V1.0 | TACHYON: CODING AND DOCUMENTATION STANDARDS | 1.0 | February 2026 | [`.adrs/ |
| TACHYON-REQ-SEC-V1.0 | TACHYON: SECURITY REQUIREMENTS | 1.0 | February 2026 | [`.adrs/ |
| TACHYON-DES-SEC-V1.0 | TACHYON: SECURITY DESIGN | 1.0 | February 2026 | [`.adrs/ |
| TACHYON-DES-API-V1.0 | TACHYON: API INTERFACES DESIGN | 1.0 | February 2026 | [`.adrs/ |
| TACHYON-TSK-V1.0 | TACHYON: EXECUTION TASKS AND WORK BREAKDOWN STRUCTURE | 1.0 | February 2026 | [`.adrs/ |

### 10.3. Architectural Decision Records

| Reference | Title | Version | Date | Path |
|-----------|-------|--------|------|------|
| ADR-001 | ADR-001: Rust as Primary Language | 1.0 | February 2026 | [`.adrs/adr-001-three-tier-jit-compilation.md](../../.adrs/adr-001-three-tier-jit-compilation.md) |
| ADR-003 | ADR-003: Axum for HTTP/2 Server | 1.0 | February 2026 | [`.adrs/adr-003-lru-cache-target.md](../../.adrs/adr-003-lru-cache-target.md) |
| ADR-007 | ADR-007: Tokio for Async Runtime | 1.0 | February 2026 | [`.adrs/adr-007-thread-safety-strategy.md](../../.adrs/adr-007-thread-safety-strategy.md) |
| ADR-010 | ADR-010: Security Architecture | 1.0 | February 2026 | [`.adrs/adr-010-synchronization-primitives.md](../../.adrs/adr-010-synchronization-primitives.md) |

### 10.4. Requirements Traceability

| Requirement ID | Title | Status | Section |
|---------------|-------|--------|---------|
| REQ-SEC-011 | Multi-Factor Authentication | Proposed | [5.2.1. Authentication](#52-authentication) |
| REQ-SEC-012 | Password Requirements | Proposed | [2.2.1. Password-Based Authentication](#522-password-based-authentication) |
| REQ-SEC-013 | OAuth 2.0 Support | Proposed | [2.2.2. OAuth 2.0 Authentication](#523-oauth-20-authentication) |
| REQ-SEC-016 | Secure Session Tokens | Proposed | [2.3.1. Access Token](#231-access-token) |
| REQ-SEC-017 | Session Timeout | Proposed | [2.4.2. Session Timeout](#5242-session-timeout) |
| REQ-SEC-018 | Session Refresh | Proposed | [6.1.1. Refresh Token](#611-refresh-token) |
| REQ-SEC-019 | Concurrent Session Limits | Proposed | [2.4.3. Concurrent Session Limits](#5243-concurrent-session-limits) |
| REQ-SEC-020 | Session Revocation | Proposed | [6.2.1. Revoke Token](#621-revoke-token) |
| REQ-SEC-056 | Comprehensive Logging | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-057 | Immutable Logs | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-058 | Log Tamper Protection | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-059 | Log Retention | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-060 | Log Access | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-061 | Authentication Events | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-062 | Authorization Events | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-063 | Data Access Events | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-064 | Configuration Events | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-065 | Security Events | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-066 | Real-Time Monitoring | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-067 | Anomaly Detection | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-068 | Alerting | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-069 | Dashboard | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-070 | Metrics Export | Proposed | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |
| REQ-SEC-071 | HTTP/2 Only | Proposed | [1.5. Protocol Stack](#15-protocol-stack) |
| REQ-SEC-075 | Secure Headers | Proposed | [9.3.1. Security Response Headers](#931-security-response-headers) |

### 10.5. Design Elements Traceability

| Design Element ID | Title | Section |
|------------------|-------|---------|
| DES-SEC-001 | AuthenticationProvider | [2.2.1. Password-Based Authentication](#522-password-based-authentication) |
| DES-SEC-002 | Credentials | [2.2.1. Password-Based Authentication](#522-password-based-authentication) |
| DES-SEC-003 | AuthResult | [2.2.1. Password-Based Authentication](#522-password-based-authentication) |
| DES-SEC-004 | TokenValidation | [2.2.1. Password-Based Authentication](#522-password-based-authentication) |
| DES-SEC-005 | AccessTokenClaims | [2.3.1. Access Token](#231-access-token) |
| DES-SEC-006 | RefreshTokenClaims | [2.3.2. Refresh Token](#232-refresh-token) |
| DES-SEC-007 | SessionStatus | [4.1.1. Validate Session](#511-validate-session) |
| DES-SEC-008 | DeviceInfo | [4.1.1. Validate Session](#511-validate-session) |
| DES-SEC-009 | DeviceType | [4.1.1. Validate Session](#511-validate-session) |
| DES-SEC-010 | LocationInfo | [4.1.1. Validate Session](#511-validate-session) |
| DES-SEC-011 | MfaMethod | [5.1.1. Initiate MFA Setup](#551-initiate-mfa-setup) |
| DES-SEC-012 | MfaVerificationContext | [5.2.1. Verify MFA Code](#552-verify-mfa-code) |
| DES-SEC-013 | RevokeReason | [6.2.1. Revoke Token](#621-revoke-token) |
| DES-SEC-014 | LogoutReason | [7.1.1. Logout](#751-logout) |
| DES-SEC-015 | RateLimitAlgorithm | [9.1.1. Rate Limiting Strategy](#911-rate-limiting-strategy) |
| DES-SEC-016 | DdosChallengeType | [9.2.1. DDoS Mitigation Strategy](#921-ddos-mitigation-strategy) |
| DES-SEC-017 | EventSeverity | [9.4.1. Authentication Event Logging](#941-authentication-event-logging) |

### 10.6. API Endpoints Summary

| Endpoint ID | HTTP Method | Path | Section |
|-------------|-------------|------|---------|
| API-AUTH-001 | POST | /api/v1/auth/login | [3.1.1. Login](#511-login) |
| API-AUTH-002 | GET | /api/v1/auth/session/validate | [4.1.1. Validate Session](#511-validate-session) |
| API-AUTH-003 | GET | /api/v1/auth/sessions | [4.2.1. List Sessions](#522-list-sessions) |
| API-AUTH-004 | POST | /api/v1/auth/mfa/setup/initiate | [5.1.1. Initiate MFA Setup](#551-initiate-mfa-setup) |
| API-AUTH-005 | POST | /api/v1/auth/mfa/setup/complete | [5.1.2. Complete MFA Setup](#552-complete-mfa-setup) |
| API-AUTH-006 | POST | /api/v1/auth/mfa/verify | [5.2.1. Verify MFA Code](#552-verify-mfa-code) |
| API-AUTH-007 | GET | /api/v1/auth/mfa/factors | [5.2.2. List MFA Factors](#552-list-mfa-factors) |
| API-AUTH-008 | POST | /api/v1/auth/refresh | [6.1.1. Refresh Token](#611-refresh-token) |
| API-AUTH-009 | POST | /api/v1/auth/revoke | [6.2.1. Revoke Token](#621-revoke-token) |
| API-AUTH-010 | POST | /api/v1/auth/logout | [7.1.1. Logout](#751-logout) |
| API-AUTH-011 | GET | /oauth2/authorize | [8.1.1. OAuth 2.0 Authorization Endpoint](#861-oauth-20-authorization-endpoint) |
| API-AUTH-012 | POST | /oauth2/token | [8.1.2. OAuth 2.0 Token Endpoint](#862-oauth-20-token-endpoint) |
| API-AUTH-013 | GET | /api/v1/auth/oauth2/providers | [8.2.1. List OAuth 2.0 Providers](#822-list-oauth-20-providers) |

### 10.7. Error Codes Summary

| Error Code | HTTP Status | Description | Section |
|------------|-------------|-------------|---------|
| AUTH_ERROR_001 | 400 | Invalid request format | [2.6.2. Error Codes](#562-error-codes) |
| AUTH_ERROR_002 | 401 | Invalid credentials | [3.1.1. Login](#511-login) |
| AUTH_ERROR_003 | 401 | Expired token | [4.1.1. Validate Session](#511-validate-session) |
| AUTH_ERROR_004 | 401 | Invalid token signature | [2.6.2. Error Codes](#562-error-codes) |
| AUTH_ERROR_005 | 403 | Account locked | [3.1.1. Login](#511-login) |
| AUTH_ERROR_006 | 403 | MFA required | [2.6.2. Error Codes](#562-error-codes) |
| AUTH_ERROR_007 | 403 | Session expired | [4.1.1. Validate Session](#511-validate-session) |
| AUTH_ERROR_008 | 429 | Rate limit exceeded | [9.1.1. Rate Limiting Strategy](#911-rate-limiting-strategy) |
| AUTH_ERROR_009 | 500 | Internal server error | [2.6.2. Error Codes](#562-error-codes) |
| AUTH_ERROR_010 | 503 | Service unavailable | [2.6.2. Error Codes](#562-error-codes) |

### 10.8. Glossary

| Term | Definition |
|-------|------------|
| JWT | JSON Web Token - A compact, URL-safe means of representing claims to be transferred between two parties |
| TOTP | Time-based One-Time Password - An algorithm that generates a one-time password using current time |
| PKCE | Proof Key for Code Exchange - An extension to OAuth 2.0 to prevent authorization code interception |
| MFA | Multi-Factor Authentication - A security system that requires more than one method of authentication |
| OAuth 2.0 | OAuth 2.0 - An authorization framework that enables third-party applications to obtain limited access to user accounts |
| CSRF | Cross-Site Request Forgery - A type of malicious exploit of a website where unauthorized commands are transmitted from a user that the website trusts |
| XSS | Cross-Site Scripting - A type of security vulnerability typically found in web applications |
| CSP | Content Security Policy - An added layer of security that helps to detect and mitigate certain types of attacks |
| HSTS | HTTP Strict Transport Security - A web security policy mechanism that helps protect websites against protocol downgrade attacks |
| DDoS | Distributed Denial of Service - A malicious attempt to disrupt normal traffic of a targeted server, service or network |
| Rate Limiting | A technique to limit the rate of traffic sent or received by a network interface controller or process |

### 10.9. Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | February 2026 | Technical Writer | Initial document creation |

---

**End of Document**

