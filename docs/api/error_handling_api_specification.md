# TACHYON: ERROR HANDLING API SPECIFICATION

**Document ID:** TACHYON-API-019-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Technical Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Error Handling Design Principles](#2-error-handling-design-principles)
3. [Error Types](#3-error-types)
4. [Error Codes](#4-error-codes)
5. [Error Response Format](#5-error-response-format)
6. [Error Propagation](#6-error-propagation)
7. [Error Recovery](#7-error-recovery)
8. [Error Logging](#8-error-logging)
9. [Error Security](#9-error-security)
10. [Error Performance](#10-error-performance)
11. [References](#11-references)

---

## 1. INTRODUCTION

### 1.1. Purpose and Scope

This document defines the comprehensive error handling conventions and status codes used across all Tachyon APIs. The specification establishes a unified approach to error handling that ensures consistency, security, and usability across the desktop application, HTTP/2 server, and web frontend components.

The Tachyon toolchain encompasses:
- A Rust-based core engine with Tokio asynchronous runtime
- A Tauri-based desktop application wrapper
- An Axum-based HTTP/2 server component
- A TypeScript/JavaScript frontend using Leptos
- Git-based content storage and management

### 1.2. Document Dependencies

This document depends on the following documents:
- [TACHYON-STD-V1.0](../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-ADR-001-V1.0](../.specs/02_adrs/001_rust_as_primary_language.md) - Rust as Primary Language
- [TACHYON-ADR-003-V1.0](../.specs/02_adrs/003_axum_for_http2_server.md) - Axum for HTTP/2 Server
- [TACHYON-ADR-007-V1.0](../.specs/02_adrs/007_tokio_for_async_runtime.md) - Tokio for Async Runtime
- [TACHYON-TMA-V1.0](../.specs/03_threat_model/analysis.md) - Threat Model Analysis

### 1.3. Error Handling Philosophy

The Tachyon error handling philosophy is grounded in the following principles:

**1.3.1. Explicit Error Handling**

All error conditions must be explicitly handled at appropriate abstraction levels. The system rejects implicit error propagation and unhandled exceptions. Rust's `Result<T, E>` type enforces explicit error handling at compile time, ensuring that all error paths are considered during development.

**Formal Property:**
$$
\forall f: \text{function}, \exists e: \text{error}, \text{return}(f) \in \{\text{Ok}(v), \text{Err}(e)\}
$$

This property states that for all functions, the return type explicitly indicates either success or an error condition.

**1.3.2. Type Safety**

Error types are strongly typed and represent specific failure modes. The system rejects generic error types that lose contextual information. Each error domain defines specific error variants that capture the full context of the failure condition.

**1.3.3. Error Context Preservation**

Errors must preserve sufficient context to enable diagnosis and recovery. The system requires that errors include:
- Error code identifying the failure mode
- Human-readable message describing the condition
- Structured context data (request IDs, timestamps, resource identifiers)
- Stack trace or call site information for debugging

**1.3.4. Security-First Error Messages**

Error messages exposed to clients must not reveal sensitive system information. The system implements a dual-layer error messaging strategy:
- **Client-facing messages:** Generic, user-friendly descriptions without implementation details
- **Internal logging messages:** Detailed diagnostic information for troubleshooting

---

## 2. ERROR HANDLING DESIGN PRINCIPLES

### 2.1. Architectural Principles

The error handling architecture for Tachyon is designed according to the following principles:

**2.1.1. Fail-Fast with Graceful Degradation**

The system fails fast when encountering unrecoverable errors while maintaining graceful degradation for non-critical failures. This principle prevents error propagation that obscures root causes while ensuring system availability.

**Implementation Strategy:**
- Critical errors (authentication failures, data corruption) immediately terminate operations
- Non-critical errors (network timeouts, cache misses) trigger fallback mechanisms
- Degraded modes are explicitly documented and communicated to users

**2.1.2. Error Boundary Isolation**

Error boundaries are established at component interfaces to prevent error propagation across architectural boundaries. Each component (desktop, server, web) implements error handling that translates internal errors to appropriate external representations.

**Boundary Definitions:**

| Boundary | Internal Representation | External Representation |
|----------|------------------------|------------------------|
| Desktop → Server | Rust `Result<T, DesktopError>` | HTTP status codes with JSON error body |
| Server → Web | Rust `Result<T, ServerError>` | WebSocket error frames or JSON responses |
| Server → Desktop | Rust `Result<T, ServerError>` | IPC error messages with structured data |

**2.1.3. Idempotent Error Handling**

Error handling operations must be idempotent to prevent cascading failures. Logging, monitoring, and alerting triggered by errors must not themselves cause additional errors or system instability.

**Formal Property:**
$$
\forall e: \text{error}, \text{handle}(e) = \text{handle}(\text{handle}(e))
$$

**2.1.4. Observability Integration**

Error handling is integrated with the observability stack to enable real-time monitoring and alerting. All errors are automatically logged with structured metadata and forwarded to monitoring systems.

### 2.2. Error Classification Hierarchy

Errors are classified according to severity, recoverability, and security impact:

**2.2.1. Severity Levels**

| Severity | Description | HTTP Status | Action Required |
|----------|-------------|---------------|----------------|
| **Critical** | System cannot continue operation; immediate intervention required | 500, 503 | Alert operations team, initiate incident response |
| **High** | Feature unavailable but system remains operational | 400, 403, 404 | Log with high priority, notify affected users |
| **Medium** | Degraded performance or partial functionality | 409, 429 | Log with medium priority, implement retry logic |
| **Low** | Cosmetic or informational issues | 200 with warnings | Log for analysis, no immediate action |

**2.2.2. Recoverability Classification**

| Classification | Description | Retry Strategy |
|----------------|-------------|----------------|
| **Transient** | Temporary condition that will resolve | Exponential backoff retry |
| **Permanent** | Condition will not resolve without intervention | No retry, require user action |
| **Idempotent** | Safe to retry without side effects | Immediate retry allowed |
| **Non-Idempotent** | Retry may cause duplicate operations | Idempotency keys required |

**2.2.3. Security Impact Classification**

| Impact | Description | Information Disclosure |
|---------|-------------|----------------------|
| **High** | May expose sensitive data or enable attacks | Minimal, generic messages only |
| **Medium** | May reveal system structure or capabilities | Sanitized messages |
| **Low** | No security implications | Detailed messages acceptable |

### 2.3. Error Handling Contract

All error handling implementations must satisfy the following contract:

**2.3.1. Error Type Requirements**

Every error type must implement the following traits:
- `Display`: Human-readable error message
- `Debug`: Detailed diagnostic information
- `std::error::Error`: Standard error interface
- `Serialize`/`Deserialize`: JSON serialization for API responses
- `IntoResponse`: Axum HTTP response conversion

**2.3.2. Error Context Requirements**

Errors must include the following context fields:
- `error_code`: Unique identifier for the error type
- `message`: Human-readable description
- `timestamp`: ISO 8601 timestamp of error occurrence
- `request_id`: Correlation identifier for request tracing
- `details`: Structured additional context (optional)

**2.3.3. Error Response Requirements**

All error responses must include:
- Appropriate HTTP status code
- JSON body with standardized error structure
- `X-Request-ID` header for correlation
- `Retry-After` header for rate-limited or temporarily unavailable resources

### 2.4. Error Handling Lifecycle

The error handling lifecycle follows these stages:

```mermaid
graph LR
    A[Error Occurrence] --> B[Error Detection]
    B --> C[Error Classification]
    C --> D{Recoverable?}
    D -->|Yes| E[Recovery Attempt]
    D -->|No| F[Error Response Generation]
    E -->|Success| G[Operation Continues]
    E -->|Failure| F
    F --> H[Error Logging]
    H --> I[Metrics Recording]
    I --> J[Alerting]
    J --> K[Client Notification]
```

**Stage Descriptions:**

1. **Error Detection:** Runtime or application logic detects an error condition
2. **Error Classification:** System determines severity, recoverability, and security impact
3. **Recovery Decision:** System evaluates whether automatic recovery is possible
4. **Recovery Attempt:** System attempts recovery if appropriate
5. **Response Generation:** System generates appropriate error response
6. **Error Logging:** System logs error with full context
7. **Metrics Recording:** System records error metrics for monitoring
8. **Alerting:** System triggers alerts for critical errors
9. **Client Notification:** System notifies client of error condition

---

## 3. ERROR TYPES

### 3.1. Error Type Hierarchy

The Tachyon system implements a hierarchical error type structure that provides fine-grained error classification while maintaining type safety and ergonomic error handling.

**3.1.1. Base Error Trait**

All error types implement the `TachyonError` trait which provides the foundation for error handling across all components.

```rust
/// Base trait for all Tachyon errors.
///
/// This trait ensures that all errors provide consistent
/// context and can be converted to appropriate
/// external representations (HTTP responses, IPC messages).
pub trait TachyonError: std::error::Error + Send + Sync {
    /// Returns the error code for this error type.
    fn error_code(&self) -> &'static str;

    /// Returns the HTTP status code appropriate for this error.
    fn http_status(&self) -> StatusCode;

    /// Returns whether this error is recoverable.
    fn is_recoverable(&self) -> bool;

    /// Returns whether this error is transient.
    fn is_transient(&self) -> bool;

    /// Returns the security impact level of this error.
    fn security_impact(&self) -> SecurityImpact;

    /// Returns structured context data for logging.
    fn context(&self) -> serde_json::Value;
}

/// Security impact levels for error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityImpact {
    /// Error may expose sensitive data or enable attacks.
    High,
    /// Error may reveal system structure or capabilities.
    Medium,
    /// Error has no security implications.
    Low,
}
```

**3.1.2. Component-Specific Error Types**

Each component defines specialized error types that implement the base trait:

| Component | Error Type | Purpose |
|-----------|-------------|---------|
| **Desktop** | `DesktopError` | Desktop application-specific errors |
| **Server** | `ServerError` | HTTP/2 server errors |
| **Web** | `WebError` | Frontend runtime errors |
| **Core** | `CoreError` | Shared core engine errors |
| **Database** | `DatabaseError` | SQLite database errors |
| **Git** | `GitError` | Git repository operation errors |
| **Search** | `SearchError` | Tantivy search engine errors |

### 3.2. Rust Error Type Definitions

**3.2.1. Desktop Error Type**

```rust
/// Errors specific to the Tauri desktop application.
#[derive(Debug, thiserror::Error)]
pub enum DesktopError {
    /// IPC communication failure with server component.
    #[error("IPC communication failed: {0}")]
    IpcCommunication(String),

    /// Local file system operation failed.
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// Desktop UI rendering error.
    #[error("UI rendering error: {0}")]
    UiRendering(String),

    /// Configuration load/save error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Server connection error.
    #[error("Server connection failed: {0}")]
    ServerConnection(String),

    /// Authentication token retrieval error.
    #[error("Authentication token error: {0}")]
    AuthToken(String),
}

impl TachyonError for DesktopError {
    fn error_code(&self) -> &'static str {
        match self {
            DesktopError::IpcCommunication(_) => "DESKTOP_IPC_001",
            DesktopError::FileSystem(_) => "DESKTOP_FS_001",
            DesktopError::UiRendering(_) => "DESKTOP_UI_001",
            DesktopError::Configuration(_) => "DESKTOP_CFG_001",
            DesktopError::ServerConnection(_) => "DESKTOP_CONN_001",
            DesktopError::AuthToken(_) => "DESKTOP_AUTH_001",
        }
    }

    fn http_status(&self) -> StatusCode {
        match self {
            DesktopError::IpcCommunication(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DesktopError::FileSystem(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DesktopError::UiRendering(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DesktopError::Configuration(_) => StatusCode::BAD_REQUEST,
            DesktopError::ServerConnection(_) => StatusCode::SERVICE_UNAVAILABLE,
            DesktopError::AuthToken(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn is_recoverable(&self) -> bool {
        matches!(self, DesktopError::ServerConnection(_) | DesktopError::AuthToken(_))
    }

    fn is_transient(&self) -> bool {
        matches!(self, DesktopError::ServerConnection(_))
    }

    fn security_impact(&self) -> SecurityImpact {
        match self {
            DesktopError::AuthToken(_) => SecurityImpact::High,
            DesktopError::Configuration(_) => SecurityImpact::Medium,
            _ => SecurityImpact::Low,
        }
    }

    fn context(&self) -> serde_json::Value {
        json!({
            "component": "desktop",
            "error_type": self.error_code(),
            "message": self.to_string(),
        })
    }
}
```

**3.2.2. Server Error Type**

```rust
/// Errors specific to the Axum HTTP/2 server.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Request validation failed.
    #[error("Request validation failed: {0}")]
    Validation(String),

    /// Authentication failed.
    #[error("Authentication failed")]
    Authentication,

    /// Authorization failed (insufficient permissions).
    #[error("Authorization failed for resource: {0}")]
    Authorization(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Database operation failed.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// Git operation failed.
    #[error("Git operation failed: {0}")]
    Git(#[from] GitError),

    /// Search operation failed.
    #[error("Search error: {0}")]
    Search(#[from] SearchError),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Concurrent modification conflict.
    #[error("Concurrent modification conflict")]
    Conflict,

    /// Request timeout.
    #[error("Request timeout")]
    Timeout,

    /// Internal server error.
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl TachyonError for ServerError {
    fn error_code(&self) -> &'static str {
        match self {
            ServerError::Validation(_) => "SRV_VAL_001",
            ServerError::Authentication => "SRV_AUTH_001",
            ServerError::Authorization(_) => "SRV_AUTH_002",
            ServerError::NotFound(_) => "SRV_NOTF_001",
            ServerError::Database(_) => "SRV_DB_001",
            ServerError::Git(_) => "SRV_GIT_001",
            ServerError::Search(_) => "SRV_SRCH_001",
            ServerError::RateLimit => "SRV_RATE_001",
            ServerError::Conflict => "SRV_CONFLICT_001",
            ServerError::Timeout => "SRV_TIMEOUT_001",
            ServerError::Internal(_) => "SRV_INT_001",
        }
    }

    fn http_status(&self) -> StatusCode {
        match self {
            ServerError::Validation(_) => StatusCode::BAD_REQUEST,
            ServerError::Authentication => StatusCode::UNAUTHORIZED,
            ServerError::Authorization(_) => StatusCode::FORBIDDEN,
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            ServerError::Conflict => StatusCode::CONFLICT,
            ServerError::Timeout => StatusCode::REQUEST_TIMEOUT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            ServerError::RateLimit
            | ServerError::Timeout
            | ServerError::Conflict
        )
    }

    fn is_transient(&self) -> bool {
        matches!(self,
            ServerError::RateLimit
            | ServerError::Timeout
        )
    }

    fn security_impact(&self) -> SecurityImpact {
        match self {
            ServerError::Authentication => SecurityImpact::High,
            ServerError::Authorization(_) => SecurityImpact::High,
            ServerError::Validation(_) => SecurityImpact::Medium,
            ServerError::NotFound(_) => SecurityImpact::Medium,
            _ => SecurityImpact::Low,
        }
    }

    fn context(&self) -> serde_json::Value {
        json!({
            "component": "server",
            "error_type": self.error_code(),
            "message": self.to_string(),
        })
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.http_status();
        let body = ErrorResponse::from_error(&self);

        (status, Json(body)).into_response()
    }
}
```

**3.2.3. Database Error Type**

```rust
/// Errors specific to SQLite database operations.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    /// Connection to database failed.
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    /// Query execution failed.
    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    /// Transaction failed.
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Constraint violation.
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Database lock timeout.
    #[error("Database lock timeout")]
    LockTimeout,

    /// Database corruption detected.
    #[error("Database corruption detected")]
    Corruption,

    /// Migration error.
    #[error("Database migration error: {0}")]
    Migration(String),
}

impl TachyonError for DatabaseError {
    fn error_code(&self) -> &'static str {
        match self {
            DatabaseError::ConnectionFailed(_) => "DB_CONN_001",
            DatabaseError::QueryFailed(_) => "DB_QUERY_001",
            DatabaseError::TransactionFailed(_) => "DB_TXN_001",
            DatabaseError::ConstraintViolation(_) => "DB_CONST_001",
            DatabaseError::LockTimeout => "DB_LOCK_001",
            DatabaseError::Corruption => "DB_CORR_001",
            DatabaseError::Migration(_) => "DB_MIG_001",
        }
    }

    fn http_status(&self) -> StatusCode {
        match self {
            DatabaseError::ConnectionFailed(_) => StatusCode::SERVICE_UNAVAILABLE,
            DatabaseError::LockTimeout => StatusCode::SERVICE_UNAVAILABLE,
            DatabaseError::Corruption => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::ConstraintViolation(_) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            DatabaseError::LockTimeout
            | DatabaseError::ConstraintViolation(_)
        )
    }

    fn is_transient(&self) -> bool {
        matches!(self, DatabaseError::LockTimeout)
    }

    fn security_impact(&self) -> SecurityImpact {
        match self {
            DatabaseError::Corruption => SecurityImpact::High,
            DatabaseError::ConnectionFailed(_) => SecurityImpact::Medium,
            _ => SecurityImpact::Low,
        }
    }

    fn context(&self) -> serde_json::Value {
        json!({
            "component": "database",
            "error_type": self.error_code(),
            "message": self.to_string(),
        })
    }
}
```

### 3.3. TypeScript Error Type Definitions

**3.3.1. Frontend Error Interface**

```typescript
/**
 * Base interface for all Tachyon frontend errors.
 *
 * @interface TachyonError
 * @property {string} errorCode - Unique identifier for the error type
 * @property {string} message - Human-readable error description
 * @property {ErrorSeverity} severity - Severity level of the error
 * @property {boolean} recoverable - Whether the error is recoverable
 * @property {boolean} transient - Whether the error is transient
 * @property {SecurityImpact} securityImpact - Security impact level
 * @property {Record<string, unknown>} [details] - Additional structured context
 */
export interface TachyonError {
  readonly errorCode: string;
  readonly message: string;
  readonly severity: ErrorSeverity;
  readonly recoverable: boolean;
  readonly transient: boolean;
  readonly securityImpact: SecurityImpact;
  readonly details?: Record<string, unknown>;
}

/**
 * Severity levels for error classification.
 */
export enum ErrorSeverity {
  /** System cannot continue operation; immediate intervention required */
  Critical = 'CRITICAL',
  /** Feature unavailable but system remains operational */
  High = 'HIGH',
  /** Degraded performance or partial functionality */
  Medium = 'MEDIUM',
  /** Cosmetic or informational issues */
  Low = 'LOW',
}

/**
 * Security impact levels for error classification.
 */
export enum SecurityImpact {
  /** Error may expose sensitive data or enable attacks */
  High = 'HIGH',
  /** Error may reveal system structure or capabilities */
  Medium = 'MEDIUM',
  /** Error has no security implications */
  Low = 'LOW',
}

/**
 * API error response from server.
 */
export interface ApiError extends TachyonError {
  readonly requestId: string;
  readonly timestamp: string;
  readonly httpStatus: number;
}

/**
 * WebSocket error.
 */
export interface WebSocketError extends TachyonError {
  readonly code: number;
  readonly reason: string;
  readonly wasClean: boolean;
}

/**
 * Network error.
 */
export interface NetworkError extends TachyonError {
  readonly url?: string;
  readonly method?: string;
  readonly statusCode?: number;
}

/**
 * Validation error.
 */
export interface ValidationError extends TachyonError {
  readonly field: string;
  readonly value: unknown;
  readonly constraint: string;
}

/**
 * Authentication error.
 */
export interface AuthenticationError extends TachyonError {
  readonly provider: string;
  readonly redirectUrl?: string;
}

/**
 * Authorization error.
 */
export interface AuthorizationError extends TachyonError {
  readonly resource: string;
  readonly requiredPermission: string;
}

/**
 * Not found error.
 */
export interface NotFoundError extends TachyonError {
  readonly resourceType: string;
  readonly resourceId: string;
}
```

**3.3.2. Error Factory Functions**

```typescript
/**
 * Creates an API error from a fetch response.
 *
 * @param response - The fetch response object
 * @param data - The parsed error response body
 * @returns ApiError instance
 */
export function createApiError(
  response: Response,
  data: unknown
): ApiError {
  const errorData = data as Record<string, unknown>;
  
  return {
    errorCode: (errorData.error_code as string) ?? 'UNKNOWN_ERROR',
    message: (errorData.message as string) ?? 'An unknown error occurred',
    severity: mapSeverity(errorData.severity as string),
    recoverable: (errorData.recoverable as boolean) ?? false,
    transient: (errorData.transient as boolean) ?? false,
    securityImpact: mapSecurityImpact(errorData.security_impact as string),
    details: errorData.details as Record<string, unknown>,
    requestId: (response.headers.get('X-Request-ID') as string) ?? generateRequestId(),
    timestamp: new Date().toISOString(),
    httpStatus: response.status,
  };
}

/**
 * Creates a network error.
 *
 * @param error - The original error object
 * @param url - The request URL
 * @param method - The HTTP method
 * @returns NetworkError instance
 */
export function createNetworkError(
  error: Error,
  url?: string,
  method?: string
): NetworkError {
  const isTimeout = error.name === 'AbortError' || error.message.includes('timeout');
  const isNetworkError = error.message.includes('NetworkError') || error.message.includes('fetch');
  
  return {
    errorCode: isTimeout ? 'NETWORK_TIMEOUT' : isNetworkError ? 'NETWORK_ERROR' : 'UNKNOWN_ERROR',
    message: isTimeout ? 'Network request timed out' : isNetworkError ? 'Network connection failed' : error.message,
    severity: ErrorSeverity.Medium,
    recoverable: true,
    transient: true,
    securityImpact: SecurityImpact.Low,
    url,
    method,
    details: { originalError: error.message },
  };
}

/**
 * Creates a validation error.
 *
 * @param field - The field that failed validation
 * @param value - The invalid value
 * @param constraint - The validation constraint that failed
 * @returns ValidationError instance
 */
export function createValidationError(
  field: string,
  value: unknown,
  constraint: string
): ValidationError {
  return {
    errorCode: 'VALIDATION_ERROR',
    message: `Validation failed for field '${field}': ${constraint}`,
    severity: ErrorSeverity.Low,
    recoverable: true,
    transient: false,
    securityImpact: SecurityImpact.Low,
    field,
    value,
    constraint,
  };
}

function mapSeverity(severity?: string): ErrorSeverity {
  switch (severity?.toUpperCase()) {
    case 'CRITICAL':
      return ErrorSeverity.Critical;
    case 'HIGH':
      return ErrorSeverity.High;
    case 'MEDIUM':
      return ErrorSeverity.Medium;
    case 'LOW':
      return ErrorSeverity.Low;
    default:
      return ErrorSeverity.Medium;
  }
}

function mapSecurityImpact(impact?: string): SecurityImpact {
  switch (impact?.toUpperCase()) {
    case 'HIGH':
      return SecurityImpact.High;
    case 'MEDIUM':
      return SecurityImpact.Medium;
    case 'LOW':
      return SecurityImpact.Low;
    default:
      return SecurityImpact.Low;
  }
}

function generateRequestId(): string {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 15)}`;
}
```

---

## 4. ERROR CODES

### 4.1. Error Code Structure

Tachyon error codes follow a hierarchical structure that enables precise error identification and classification. The code format is: `[COMPONENT]_[CATEGORY]_[SEQUENCE]`.

**4.1.1. Code Format Specification**

```
[COMPONENT]_[CATEGORY]_[SEQUENCE]
```

**Components:**
- `COMPONENT`: Three-letter component identifier (e.g., `SRV` for server, `DB` for database)
- `CATEGORY`: Three-letter category identifier (e.g., `AUTH` for authentication, `VAL` for validation)
- `SEQUENCE`: Three-digit sequence number (001-999)

**Example Codes:**
- `SRV_AUTH_001`: Server authentication error, sequence 001
- `DB_CONN_001`: Database connection error, sequence 001
- `DESKTOP_FS_001`: Desktop file system error, sequence 001

### 4.2. Component Error Codes

**4.2.1. Server Error Codes (SRV)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|------------|----------|--------------|-------------|-------------|
| `SRV_VAL_001` | Validation | 400 | Request validation failed | Yes |
| `SRV_AUTH_001` | Authentication | 401 | Authentication failed | Yes |
| `SRV_AUTH_002` | Authorization | 403 | Authorization failed (insufficient permissions) | No |
| `SRV_NOTF_001` | Resource | 404 | Resource not found | No |
| `SRV_DB_001` | Database | 500 | Database operation failed | Yes |
| `SRV_GIT_001` | Git | 500 | Git operation failed | Yes |
| `SRV_SRCH_001` | Search | 500 | Search operation failed | Yes |
| `SRV_RATE_001` | Rate Limiting | 429 | Rate limit exceeded | Yes |
| `SRV_CONFLICT_001` | Concurrency | 409 | Concurrent modification conflict | Yes |
| `SRV_TIMEOUT_001` | Timeout | 408 | Request timeout | Yes |
| `SRV_INT_001` | Internal | 500 | Internal server error | No |

**4.2.2. Desktop Error Codes (DESKTOP)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|--------------|----------|--------------|-------------|-------------|
| `DESKTOP_IPC_001` | IPC | 500 | IPC communication failed | Yes |
| `DESKTOP_FS_001` | File System | 500 | File system operation failed | Yes |
| `DESKTOP_UI_001` | UI | 500 | UI rendering error | Yes |
| `DESKTOP_CFG_001` | Configuration | 400 | Configuration error | Yes |
| `DESKTOP_CONN_001` | Network | 503 | Server connection failed | Yes |
| `DESKTOP_AUTH_001` | Authentication | 401 | Authentication token error | Yes |

**4.2.3. Database Error Codes (DB)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|----------|----------|--------------|-------------|-------------|
| `DB_CONN_001` | Connection | 503 | Database connection failed | Yes |
| `DB_QUERY_001` | Query | 500 | Query execution failed | Yes |
| `DB_TXN_001` | Transaction | 500 | Transaction failed | Yes |
| `DB_CONST_001` | Constraint | 409 | Constraint violation | Yes |
| `DB_LOCK_001` | Lock | 503 | Database lock timeout | Yes |
| `DB_CORR_001` | Corruption | 500 | Database corruption detected | No |
| `DB_MIG_001` | Migration | 500 | Database migration error | No |

**4.2.4. Git Error Codes (GIT)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|-----------|----------|--------------|-------------|-------------|
| `GIT_CLONE_001` | Clone | 500 | Repository clone failed | Yes |
| `GIT_PULL_001` | Pull | 500 | Repository pull failed | Yes |
| `GIT_PUSH_001` | Push | 500 | Repository push failed | Yes |
| `GIT_COMMIT_001` | Commit | 500 | Commit operation failed | Yes |
| `GIT_MERGE_001` | Merge | 409 | Merge conflict detected | Yes |
| `GIT_CHECKOUT_001` | Checkout | 500 | Checkout operation failed | Yes |
| `GIT_FETCH_001` | Fetch | 500 | Fetch operation failed | Yes |

**4.2.5. Search Error Codes (SRCH)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|-------------|----------|--------------|-------------|-------------|
| `SRCH_INDEX_001` | Indexing | 500 | Search index creation failed | Yes |
| `SRCH_QUERY_001` | Query | 400 | Invalid search query | Yes |
| `SRCH_TIMEOUT_001` | Timeout | 408 | Search query timeout | Yes |
| `SRCH_CORRUPT_001` | Corruption | 500 | Search index corrupted | No |

**4.2.6. Authentication Error Codes (AUTH)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|-------------|----------|--------------|-------------|-------------|
| `AUTH_CREDS_001` | Credentials | 401 | Invalid credentials | Yes |
| `AUTH_TOKEN_001` | Token | 401 | Invalid or expired token | Yes |
| `AUTH_MFA_001` | MFA | 401 | MFA verification failed | Yes |
| `AUTH_SESSION_001` | Session | 401 | Session expired | Yes |
| `AUTH_PROVIDER_001` | Provider | 500 | Authentication provider error | Yes |

**4.2.7. Authorization Error Codes (AUTHZ)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|--------------|----------|--------------|-------------|-------------|
| `AUTHZ_PERM_001` | Permission | 403 | Insufficient permissions | No |
| `AUTHZ_ROLE_001` | Role | 403 | Invalid role assignment | No |
| `AUTHZ_RESOURCE_001` | Resource | 403 | Resource access denied | No |
| `AUTHZ_SCOPE_001` | Scope | 403 | Insufficient scope | No |

**4.2.8. Network Error Codes (NET)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|------------|----------|--------------|-------------|-------------|
| `NET_TIMEOUT_001` | Timeout | 408 | Network request timeout | Yes |
| `NET_CONN_001` | Connection | 503 | Network connection failed | Yes |
| `NET_DNS_001` | DNS | 502 | DNS resolution failed | Yes |
| `NET_SSL_001` | SSL | 502 | SSL/TLS handshake failed | Yes |

**4.2.9. Validation Error Codes (VAL)**

| Error Code | Category | HTTP Status | Description | Recoverable |
|------------|----------|--------------|-------------|-------------|
| `VAL_SCHEMA_001` | Schema | 400 | JSON schema validation failed | Yes |
| `VAL_TYPE_001` | Type | 400 | Invalid data type | Yes |
| `VAL_RANGE_001` | Range | 400 | Value out of range | Yes |
| `VAL_REQUIRED_001` | Required | 400 | Required field missing | Yes |
| `VAL_FORMAT_001` | Format | 400 | Invalid format | Yes |
| `VAL_LENGTH_001` | Length | 400 | Invalid length | Yes |
| `VAL_PATTERN_001` | Pattern | 400 | Pattern mismatch | Yes |

### 4.3. HTTP Status Code Mapping

The Tachyon system maps error codes to appropriate HTTP status codes according to RFC 7231 and RFC 6585.

**4.3.1. 4xx Client Error Codes**

| Status Code | Error Codes | Description | Retry Strategy |
|-------------|-------------|-------------|----------------|
| 400 Bad Request | `SRV_VAL_001`, `VAL_*` | Malformed request syntax | No retry |
| 401 Unauthorized | `SRV_AUTH_001`, `AUTH_*` | Authentication required | Yes, with credentials |
| 403 Forbidden | `SRV_AUTH_002`, `AUTHZ_*` | Insufficient permissions | No retry |
| 404 Not Found | `SRV_NOTF_001` | Resource not found | No retry |
| 408 Request Timeout | `SRV_TIMEOUT_001`, `NET_TIMEOUT_001` | Request timeout | Yes, exponential backoff |
| 409 Conflict | `SRV_CONFLICT_001`, `DB_CONST_001`, `GIT_MERGE_001` | Resource conflict | Yes, with conflict resolution |
| 413 Payload Too Large | `VAL_LENGTH_001` | Request entity too large | No retry |
| 415 Unsupported Media Type | `VAL_FORMAT_001` | Unsupported media type | No retry |
| 422 Unprocessable Entity | `VAL_SCHEMA_001` | Semantic error | No retry |
| 426 Upgrade Required | `NET_SSL_001` | Protocol upgrade required | No retry |
| 429 Too Many Requests | `SRV_RATE_001` | Rate limit exceeded | Yes, with Retry-After |

**4.3.2. 5xx Server Error Codes**

| Status Code | Error Codes | Description | Retry Strategy |
|-------------|-------------|-------------|----------------|
| 500 Internal Server Error | `SRV_INT_001`, `DB_*`, `GIT_*`, `SRCH_*` | Unexpected server error | Yes, exponential backoff |
| 501 Not Implemented | `SRV_INT_001` | Feature not implemented | No retry |
| 502 Bad Gateway | `NET_DNS_001`, `NET_CONN_001` | Invalid gateway response | Yes, exponential backoff |
| 503 Service Unavailable | `DB_CONN_001`, `DB_LOCK_001`, `DESKTOP_CONN_001` | Service temporarily unavailable | Yes, with Retry-After |
| 504 Gateway Timeout | `NET_TIMEOUT_001` | Gateway timeout | Yes, exponential backoff |
| 507 Insufficient Storage | `DB_CORR_001` | Cannot store representation | No retry |
| 508 Loop Detected | `SRV_CONFLICT_001` | Server detected infinite loop | No retry |
| 510 Not Extended | `SRV_INT_001` | Further extensions required | No retry |
| 511 Network Authentication Required | `NET_SSL_001` | Network authentication required | No retry |

**4.3.3. Success Codes with Warnings**

| Status Code | Description | Warning Condition |
|-------------|-------------|-------------------|
| 200 OK | Request succeeded without warnings | None |
| 202 Accepted | Request accepted for processing | Processing may take time |
| 203 Non-Authoritative Information | Response from cache or third-party | Data may be stale |
| 204 No Content | Request succeeded with no response body | None |
| 206 Partial Content | Partial response for range request | Additional requests needed |
| 207 Multi-Status | Multiple status codes returned | Check individual status codes |

### 4.4. WebSocket Error Codes

WebSocket connections use close codes defined in RFC 6455 with Tachyon-specific extensions.

**4.4.1. Standard WebSocket Close Codes**

| Code | Description | Action Required |
|------|-------------|----------------|
| 1000 | Normal closure | None |
| 1001 | Endpoint going away | Reconnect after delay |
| 1002 | Protocol error | Report to development team |
| 1003 | Unsupported data | Report to development team |
| 1005 | No status received | Report to development team |
| 1006 | Abnormal closure | Reconnect with caution |
| 1007 | Invalid frame payload | Report to development team |
| 1008 | Policy violation | Review application logic |
| 1009 | Message too large | Reduce message size |
| 1010 | Mandatory extension | Report to development team |
| 1011 | Internal server error | Report to operations team |
| 1015 | TLS handshake failed | Check TLS configuration |

**4.4.2. Tachyon-Specific WebSocket Error Codes**

| Code | Description | Action Required |
|------|-------------|----------------|
| 4000 | Authentication required | Re-authenticate |
| 4001 | Authorization failed | Check permissions |
| 4002 | Rate limit exceeded | Implement backoff |
| 4003 | Invalid message format | Fix message format |
| 4004 | Unknown message type | Update client version |
| 4005 | Subscription limit reached | Close unnecessary subscriptions |
| 4006 | Invalid subscription ID | Verify subscription ID |
| 4007 | Server maintenance | Reconnect after delay |

### 4.5. Error Code Assignment Guidelines

**4.5.1. New Error Code Creation**

When creating new error codes, follow these guidelines:

1. **Component Selection:** Choose appropriate three-letter component code
2. **Category Selection:** Choose appropriate three-letter category code
3. **Sequence Assignment:** Assign next available sequence number (001-999)
4. **Documentation:** Document error code in this specification
5. **HTTP Status Mapping:** Map to appropriate HTTP status code
6. **Recoverability:** Mark as recoverable or non-recoverable
7. **Security Impact:** Assign appropriate security impact level

**4.5.2. Error Code Deprecation**

Error codes may be deprecated following this process:

1. **Deprecation Notice:** Announce deprecation with 6-month notice
2. **Replacement Code:** Provide replacement error code
3. **Transition Period:** Maintain support for deprecated code for 6 months
4. **Removal:** Remove deprecated code after transition period
5. **Documentation:** Update all documentation to reference replacement code

**4.5.3. Reserved Error Codes**

The following error code ranges are reserved for future use:

| Range | Purpose |
|-------|---------|
| `*_RES_001` to `*_RES_099` | Reserved for future use |
| `*_LEG_001` to `*_LEG_099` | Legacy error codes (deprecated) |
| `*_TEST_001` to `*_TEST_099` | Testing and development only |

---

## 5. ERROR RESPONSE FORMAT

### 5.1. HTTP Error Response Structure

All HTTP error responses follow a standardized JSON structure that provides consistent error information across all Tachyon APIs.

**5.1.1. Standard Error Response Schema**

```json
{
  "error": {
    "code": "SRV_AUTH_001",
    "message": "Authentication failed",
    "severity": "HIGH",
    "recoverable": true,
    "transient": false,
    "security_impact": "HIGH"
  },
  "request_id": "2026-02-06T07:30:15.123Z-abc123",
  "timestamp": "2026-02-06T07:30:15.123Z",
  "details": {
    "attempt": 3,
    "provider": "local"
  }
}
```

**5.1.2. Field Descriptions**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `error.code` | string | Yes | Error code from Section 4 |
| `error.message` | string | Yes | Human-readable error description |
| `error.severity` | string | Yes | Severity level (CRITICAL, HIGH, MEDIUM, LOW) |
| `error.recoverable` | boolean | Yes | Whether error is recoverable |
| `error.transient` | boolean | Yes | Whether error is transient |
| `error.security_impact` | string | Yes | Security impact level (HIGH, MEDIUM, LOW) |
| `request_id` | string | Yes | Unique request identifier for tracing |
| `timestamp` | string | Yes | ISO 8601 timestamp of error occurrence |
| `details` | object | No | Additional structured context |

**5.1.3. Rust Implementation**

```rust
/// Standard error response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error information.
    pub error: ErrorInfo,
    
    /// Request correlation identifier.
    pub request_id: String,
    
    /// Error occurrence timestamp.
    pub timestamp: String,
    
    /// Additional structured context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Error information structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code.
    pub code: String,
    
    /// Human-readable error message.
    pub message: String,
    
    /// Severity level.
    pub severity: String,
    
    /// Whether error is recoverable.
    pub recoverable: bool,
    
    /// Whether error is transient.
    pub transient: bool,
    
    /// Security impact level.
    pub security_impact: String,
}

impl ErrorResponse {
    /// Creates error response from a TachyonError.
    pub fn from_error<E: TachyonError>(error: &E) -> Self {
        Self {
            error: ErrorInfo {
                code: error.error_code().to_string(),
                message: error.to_string(),
                severity: error.severity().to_string(),
                recoverable: error.is_recoverable(),
                transient: error.is_transient(),
                security_impact: error.security_impact().to_string(),
            },
            request_id: generate_request_id(),
            timestamp: Utc::now().to_rfc3339(),
            details: Some(error.context()),
        }
    }
}

fn generate_request_id() -> String {
    format!(
        "{}-{}",
        Utc::now().to_rfc3339(),
        uuid::Uuid::new_v4()
    )
}
```

**5.1.4. TypeScript Implementation**

```typescript
/**
 * Standard error response from server.
 */
export interface ErrorResponse {
  readonly error: ErrorInfo;
  readonly requestId: string;
  readonly timestamp: string;
  readonly details?: Record<string, unknown>;
}

/**
 * Error information structure.
 */
export interface ErrorInfo {
  readonly code: string;
  readonly message: string;
  readonly severity: ErrorSeverity;
  readonly recoverable: boolean;
  readonly transient: boolean;
  readonly securityImpact: SecurityImpact;
}

/**
 * Parses error response from fetch response.
 *
 * @param response - The fetch response object
 * @returns Parsed ErrorResponse
 */
export async function parseErrorResponse(
  response: Response
): Promise<ErrorResponse> {
  const data = await response.json() as Record<string, unknown>;
  return {
    error: {
      code: (data.error as Record<string, unknown>).code as string ?? 'UNKNOWN_ERROR',
      message: (data.error as Record<string, unknown>).message as string ?? 'An unknown error occurred',
      severity: mapSeverity((data.error as Record<string, unknown>).severity as string),
      recoverable: (data.error as Record<string, unknown>).recoverable as boolean ?? false,
      transient: (data.error as Record<string, unknown>).transient as boolean ?? false,
      securityImpact: mapSecurityImpact((data.error as Record<string, unknown>).security_impact as string),
    },
    requestId: (response.headers.get('X-Request-ID') as string) ?? generateRequestId(),
    timestamp: (data.timestamp as string) ?? new Date().toISOString(),
    details: data.details as Record<string, unknown>,
  };
}
```

### 5.2. HTTP Response Headers

Error responses include specific HTTP headers to enable client-side error handling and request tracing.

**5.2.1. Standard Error Response Headers**

| Header | Format | Required | Description |
|--------|---------|----------|-------------|
| `Content-Type` | `application/json` | Yes | Error response body format |
| `X-Request-ID` | UUID | Yes | Request correlation identifier |
| `Retry-After` | HTTP-date or integer | Conditional | Seconds to wait before retry |
| `X-RateLimit-Limit` | integer | Conditional | Rate limit ceiling |
| `X-RateLimit-Remaining` | integer | Conditional | Remaining requests in window |
| `X-RateLimit-Reset` | HTTP-date | Conditional | Rate limit window reset time |
| `X-Error-Code` | Error code | Yes | Error code for quick reference |

**5.2.2. Header Usage Examples**

**Rate Limit Response:**
```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
X-Request-ID: 550e8400-e29b-41d4-a7c6-4381f7b
Retry-After: 60
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: Thu, 06 Feb 2026 07:31:00 GMT
X-Error-Code: SRV_RATE_001

{
  "error": {
    "code": "SRV_RATE_001",
    "message": "Rate limit exceeded",
    "severity": "MEDIUM",
    "recoverable": true,
    "transient": true,
    "security_impact": "LOW"
  },
  "request_id": "550e8400-e29b-41d4-a7c6-4381f7b",
  "timestamp": "2026-02-06T07:30:15.123Z",
  "details": {
    "limit": 100,
    "window": "1m",
    "reset_at": "2026-02-06T07:31:00.000Z"
  }
}
```

**Authentication Failure Response:**
```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json
X-Request-ID: 6a1b2c3d-4e5f-6a7b-8c9d-0e1f2a
X-Error-Code: SRV_AUTH_001

{
  "error": {
    "code": "SRV_AUTH_001",
    "message": "Authentication failed",
    "severity": "HIGH",
    "recoverable": true,
    "transient": false,
    "security_impact": "HIGH"
  },
  "request_id": "6a1b2c3d-4e5f-6a7b-8c9d-0e1f2a",
  "timestamp": "2026-02-06T07:30:15.123Z",
  "details": {
    "provider": "local",
    "attempt": 1
  }
}
```

### 5.3. WebSocket Error Response Format

WebSocket errors use a different format optimized for real-time communication.

**5.3.1. WebSocket Error Frame Structure**

```json
{
  "type": "error",
  "code": 4001,
  "message": "Authentication required",
  "severity": "HIGH",
  "recoverable": true,
  "timestamp": "2026-02-06T07:30:15.123Z",
  "request_id": "6a1b2c3d-4e5f-6a7b-8c9d-0e1f2a"
}
```

**5.3.2. WebSocket Error Types**

| Type | Description | Fields |
|------|-------------|---------|
| `error` | Error message | `type`, `code`, `message`, `severity`, `recoverable`, `timestamp`, `request_id` |
| `notification` | Notification message | `type`, `message`, `timestamp` |
| `update` | Data update | `type`, `data`, `timestamp` |
| `ping` | Heartbeat | `type`, `timestamp` |

**5.3.3. WebSocket Error Implementation**

```rust
/// WebSocket message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "error")]
    Error(WebSocketError),
    #[serde(rename = "notification")]
    Notification(Notification),
    #[serde(rename = "update")]
    Update(Update),
    #[serde(rename = "ping")]
    Ping(Ping),
}

/// WebSocket error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketError {
    /// Error code.
    pub code: u16,
    
    /// Error message.
    pub message: String,
    
    /// Severity level.
    pub severity: String,
    
    /// Whether error is recoverable.
    pub recoverable: bool,
    
    /// Error timestamp.
    pub timestamp: String,
    
    /// Request correlation identifier.
    pub request_id: String,
}

impl WebSocketError {
    /// Creates WebSocket error from TachyonError.
    pub fn from_error<E: TachyonError>(error: &E) -> Self {
        Self {
            code: match error.error_code() {
                "SRV_AUTH_001" => 4000,
                "SRV_AUTH_002" => 4001,
                "SRV_RATE_001" => 4002,
                _ => 4999,
            },
            message: error.to_string(),
            severity: error.severity().to_string(),
            recoverable: error.is_recoverable(),
            timestamp: Utc::now().to_rfc3339(),
            request_id: generate_request_id(),
        }
    }
}
```

### 5.4. IPC Error Response Format

Desktop-server IPC communication uses a binary message format for efficient error transmission.

**5.4.1. IPC Error Message Structure**

```rust
/// IPC message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    #[serde(rename = "request")]
    Request(IpcRequest),
    #[serde(rename = "response")]
    Response(IpcResponse),
    #[serde(rename = "error")]
    Error(IpcError),
    #[serde(rename = "notification")]
    Notification(IpcNotification),
}

/// IPC error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    /// Error code.
    pub code: String,
    
    /// Error message.
    pub message: String,
    
    /// Severity level.
    pub severity: String,
    
    /// Whether error is recoverable.
    pub recoverable: bool,
    
    /// Error timestamp.
    pub timestamp: String,
    
    /// Request correlation identifier.
    pub request_id: String,
    
    /// Additional context data.
    pub details: Option<serde_json::Value>,
}

impl IpcError {
    /// Creates IPC error from TachyonError.
    pub fn from_error<E: TachyonError>(error: &E) -> Self {
        Self {
            code: error.error_code().to_string(),
            message: error.to_string(),
            severity: error.severity().to_string(),
            recoverable: error.is_recoverable(),
            timestamp: Utc::now().to_rfc3339(),
            request_id: generate_request_id(),
            details: Some(error.context()),
        }
    }
}
```

### 5.5. Error Response Best Practices

**5.5.1. Client-Side Error Handling**

Clients should implement the following error handling patterns:

```typescript
/**
 * Handles API errors with appropriate user feedback.
 *
 * @param error - The error response from server
 * @returns User-facing error message
 */
export function handleApiError(error: ApiError): string {
  switch (error.errorCode) {
    case 'SRV_AUTH_001':
    case 'AUTH_CREDS_001':
      return 'Authentication failed. Please check your credentials.';
    
    case 'SRV_AUTH_002':
    case 'AUTHZ_PERM_001':
      return 'You do not have permission to perform this action.';
    
    case 'SRV_RATE_001':
      return `Rate limit exceeded. Please try again in ${error.details?.retryAfter || 60} seconds.`;
    
    case 'SRV_TIMEOUT_001':
    case 'NET_TIMEOUT_001':
      return 'Request timed out. Please check your connection and try again.';
    
    case 'SRV_NOTF_001':
      return 'The requested resource was not found.';
    
    case 'SRV_VAL_001':
    case 'VAL_SCHEMA_001':
    case 'VAL_TYPE_001':
    case 'VAL_REQUIRED_001':
      return `Invalid request: ${error.message}`;
    
    default:
      return 'An unexpected error occurred. Please try again or contact support.';
  }
}

/**
 * Determines whether error should trigger retry.
 *
 * @param error - The error to evaluate
 * @returns Whether to retry
 */
export function shouldRetry(error: ApiError): boolean {
  return error.recoverable && error.transient;
}

/**
 * Calculates retry delay using exponential backoff.
 *
 * @param attempt - Current retry attempt number
 * @param maxDelay - Maximum delay in milliseconds
 * @returns Delay in milliseconds
 */
export function calculateRetryDelay(
  attempt: number,
  maxDelay: number = 30000
): number {
  const baseDelay = 1000; // 1 second base
  const delay = Math.min(baseDelay * Math.pow(2, attempt - 1), maxDelay);
  return delay + Math.random() * 1000; // Add jitter
}

/**
 * Executes API call with automatic retry logic.
 *
 * @param fetchFn - Async function to execute
 * @param maxAttempts - Maximum retry attempts
 * @returns Promise with result or error
 */
export async function retryWithBackoff<T>(
  fetchFn: () => Promise<T>,
  maxAttempts: number = 3
): Promise<T> {
  let lastError: Error | undefined;
  
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fetchFn();
    } catch (error) {
      lastError = error as Error;
      
      if (attempt < maxAttempts && shouldRetry(error as ApiError)) {
        const delay = calculateRetryDelay(attempt);
        await new Promise(resolve => setTimeout(resolve, delay));
      } else {
        throw error;
      }
    }
  }
  
  throw lastError || new Error('Max attempts reached');
}
```

**5.5.2. Server-Side Error Response Generation**

Server handlers should use the following pattern for consistent error responses:

```rust
use axum::{
    Json,
    response::{IntoResponse, Response},
    http::StatusCode,
};

/// Wraps handler result into appropriate error response.
pub async fn handle_result<T, E>(
    result: Result<T, E>,
) -> Result<Json<T>, Response>
where
    E: TachyonError + IntoResponse,
{
    match result {
        Ok(value) => Ok(Json(value)),
        Err(error) => {
            let status = error.http_status();
            let error_response = ErrorResponse::from_error(&error);
            
            // Add headers for specific error types
            let mut response = (status, Json(error_response)).into_response();
            
            if status == StatusCode::TOO_MANY_REQUESTS {
                if let Some(details) = error_response.details {
                    if let Some(retry_after) = details.get("retry_after") {
                        if let Some(seconds) = retry_after.as_u64() {
                            let retry_after = chrono::Duration::seconds(seconds as i64);
                            response.headers_mut().insert(
                                "Retry-After",
                                HeaderValue::from_str(&format!("{}", retry_after.num_seconds())).unwrap(),
                            );
                        }
                    }
                }
            }
            
            // Add request ID header
            response.headers_mut().insert(
                "X-Request-ID",
                HeaderValue::from_str(&error_response.request_id).unwrap(),
            );
            
            // Add error code header for quick reference
            response.headers_mut().insert(
                "X-Error-Code",
                HeaderValue::from_str(error.error_code()).unwrap(),
            );
            
            Err(response)
        }
    }
}
```

---

## 6. ERROR PROPAGATION

### 6.1. Propagation Architecture

The Tachyon system implements a hierarchical error propagation model that ensures errors are handled at appropriate abstraction levels while maintaining context and traceability.

**6.1.1. Propagation Layers**

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Layer                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Frontend Layer              │   │
│  │  ┌───────────────────────────────────┐    │   │
│  │  │         UI Components         │    │   │
│  │  └───────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                       │ HTTP/WebSocket
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Server Layer                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │              HTTP Handler Layer         │   │
│  │  ┌───────────────────────────────────┐    │   │
│  │  │         Route Handlers          │    │   │
│  │  │  ┌─────────────────────────┐    │   │
│  │  │  │   Business Logic     │    │   │
│  │  │  └─────────────────────────┘    │   │
│  │  └───────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Service Layer              │   │
│  │  ┌───────────────────────────────────┐    │   │
│  │  │         Database Service      │    │   │
│  │  │         Git Service          │    │   │
│  │  │         Search Service       │    │   │
│  │  └───────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                       │ IPC
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                   Desktop Layer                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │              IPC Handler Layer           │   │
│  │  ┌───────────────────────────────────┐    │   │
│  │  │         Desktop UI          │    │   │
│  │  └───────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**6.1.2. Propagation Rules**

1. **Bottom-Up Propagation:** Errors propagate upward through the call stack
2. **Context Preservation:** Error context is enriched at each layer
3. **Boundary Translation:** Errors are translated at architectural boundaries
4. **Logging at Each Layer:** Errors are logged at every propagation point
5. **Request ID Propagation:** Request correlation IDs flow through entire stack

### 6.2. Rust Error Propagation

**6.2.1. Result Type Propagation**

Rust's `Result<T, E>` type enables explicit error propagation with the `?` operator.

```rust
/// Business logic function returning Result.
pub async fn get_document(
    id: &str,
    db: &DatabaseService,
) -> Result<Document, ServerError> {
    // Database operation returns Result
    let doc = db.get_document(id).await
        .map_err(|e| ServerError::Database(e))?;
    
    // Business logic validation
    if doc.is_deleted {
        return Err(ServerError::NotFound(format!(
            "Document {} not found",
            id
        )));
    }
    
    Ok(doc)
}

/// HTTP handler using Result propagation.
pub async fn handle_get_document(
    Path(id): Path<String>,
    State(db): State<Arc<DatabaseService>>,
) -> Result<Json<Document>, ServerError> {
    let result = get_document(&id, &db).await;
    
    // Convert Result to Axum response
    handle_result(result)
}
```

**6.2.2. Error Context Enrichment**

Errors are enriched with additional context as they propagate through layers.

```rust
use tracing::{error, instrument, Span};

/// Enriches error with additional context.
pub fn enrich_error<E>(
    error: E,
    context: impl Fn() -> serde_json::Value,
) -> E
where
    E: std::error::Error + 'static,
{
    let span = Span::current();
    
    // Add tracing context
    error!(
        parent: &span,
        error = %error,
        context = context(),
    );
    
    error
}

/// Macro for enriching errors with context.
#[macro_export]
macro_rules! with_context {
    ($error:expr, $($key:expr => $value:expr),* $(,)?) => {
        {
            let mut ctx = serde_json::json!({});
            $(
                ctx[$key.to_string()] = serde_json::json!($value);
            )*
            
            enrich_error($error, || ctx)
        }
    };
}
```

**6.2.3. Async Error Propagation**

Async error propagation uses Tokio's `?` operator for ergonomic error handling.

```rust
use tokio::try_join;

/// Joins multiple async operations with error aggregation.
pub async fn fetch_with_metadata(
    id: &str,
    db: &DatabaseService,
    git: &GitService,
) -> Result<(Document, Metadata), ServerError> {
    let doc_future = db.get_document(id);
    let meta_future = git.get_metadata(id);
    
    // Try join both operations
    let (doc, meta) = try_join!(
        doc_future,
        meta_future,
    )
    .await
    .map_err(|e| ServerError::Internal(format!(
        "Failed to fetch document and metadata: {}",
        e
    )))?;
    
    Ok((doc?, meta?))
}
```

### 6.3. TypeScript Error Propagation

**6.3.1. Promise Chain Propagation**

TypeScript uses Promise chains for async error propagation.

```typescript
/**
 * Fetches document with automatic error handling.
 *
 * @param id - Document ID
 * @returns Promise with document
 */
export async function fetchDocument(id: string): Promise<Document> {
  try {
    const response = await fetch(`/api/documents/${id}`);
    
    if (!response.ok) {
      const error = await parseErrorResponse(response);
      throw handleApiError(error);
    }
    
    const data = await response.json() as ApiResponse<Document>;
    return data.data;
  } catch (error) {
    // Re-throw with additional context
    throw enhanceError(error, { documentId: id });
  }
}

/**
 * Enhances error with additional context.
 *
 * @param error - Original error
 * @param context - Additional context
 * @returns Enhanced error
 */
export function enhanceError(
  error: unknown,
  context: Record<string, unknown>
): ApiError {
  if (error instanceof ApiError) {
    return {
      ...error,
      details: {
        ...error.details,
        ...context,
      },
    };
  }
  
  return {
    errorCode: 'UNKNOWN_ERROR',
    message: 'An unexpected error occurred',
    severity: ErrorSeverity.Medium,
    recoverable: true,
    transient: false,
    securityImpact: SecurityImpact.Low,
    details: { originalError: String(error), ...context },
  };
}
```

**6.3.2. Observable Error Propagation**

RxJS observables enable reactive error propagation.

```typescript
import { Observable, throwError, catchError } from 'rxjs';

/**
 * Creates an observable with error handling.
 *
 * @param id - Document ID
 * @returns Observable with document
 */
export function documentObservable(id: string): Observable<Document> {
  return new Observable<Document>((subscriber) => {
    fetchDocument(id)
      .pipe(
        catchError((error) => {
          const enhancedError = enhanceError(error, { documentId: id });
          
          // Log error
          console.error('Document fetch failed:', enhancedError);
          
          // Notify subscriber of error
          subscriber.error(enhancedError);
          
          // Complete observable
          subscriber.complete();
        })
      )
      .subscribe(subscriber);
  });
}
```

### 6.4. Middleware Error Propagation

Axum middleware provides centralized error handling for HTTP requests.

**6.4.1. Error Handling Middleware**

```rust
use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::{error, instrument};

/// Middleware for handling all errors.
pub async fn error_handler_middleware(
    req: Request,
    next: Next,
) -> Response {
    // Execute next handler
    let response = next.run(req).await;
    
    // Handle any errors
    match response {
        Err(error) => {
            // Log error with request context
            error!(
                request_id = %request_id,
                method = %method,
                path = %path,
                error = %error,
                error_type = %error_code,
            );
            
            // Convert error to response
            let status = error.http_status();
            let error_response = ErrorResponse::from_error(&error);
            
            let mut response = (status, Json(error_response)).into_response();
            
            // Add request ID header
            response.headers_mut().insert(
                "X-Request-ID",
                HeaderValue::from_str(&error_response.request_id).unwrap(),
            );
            
            response
        }
        Ok(response) => response,
    }
}

/// Request ID extractor.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl<S> FromRequestParts<S> for RequestId {
    type Rejection = Response;
    
    fn from_request_parts(
        parts: &RequestParts<'_>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(RequestId(generate_request_id()))
    }
}
```

**6.4.2. Layer-Specific Middleware**

```rust
use axum::{middleware, Router};

/// Creates router with error handling middleware.
pub fn create_app() -> Router {
    Router::new()
        .route("/documents/:id", get(handle_get_document))
        .route("/search", get(handle_search))
        .layer(middleware::from_fn(error_handler_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(tracing_middleware))
}
```

### 6.5. Error Propagation Patterns

**6.5.1. Circuit Breaker Pattern**

Circuit breaker prevents cascading failures by temporarily stopping requests to failing services.

```rust
use tokio::sync::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker configuration.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub timeout: Duration,
    pub recovery_timeout: Duration,
}

/// Circuit breaker implementation.
pub struct CircuitBreaker {
    state: Arc<RwLock<Circuit_breaker_state>>,
    failures: Arc<RwLock<usize>>,
    successes: Arc<RwLock<usize>>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: Arc::new(RwLock::new(0)),
            successes: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
        }
    }
    
    /// Executes function with circuit breaker protection.
    pub async fn execute<F, T, E>(
        &self,
        f: F,
    ) -> Result<T, E>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn Future<Output = Result<T, E>>>,
    {
        // Check circuit state
        {
            let state = *self.state.read().await;
            if state == CircuitState::Open {
                return Err(E::from("Circuit breaker is open")));
            }
        }
        
        // Execute function
        let result = f().await;
        
        // Update circuit state based on result
        match &result {
            Ok(_) => {
                *self.successes.write().await += 1;
                *self.failures.write().await = 0;
                
                let successes = *self.successes.read().await;
                if successes >= self.config.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                }
            }
            Err(_) => {
                *self.failures.write().await += 1;
                *self.successes.write().await = 0;
                
                let failures = *self.failures.read().await;
                *self.last_failure_time.write().await = Some(Instant::now());
                
                if failures >= self.config.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                }
            }
        }
        
        result
    }
}
```

**6.5.2. Retry Pattern with Backoff**

Exponential backoff implements intelligent retry logic for transient errors.

```rust
use tokio::time::{sleep, Duration, timeout};

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

/// Executes function with retry logic.
pub async fn retry_with_backoff<F, T, E>(
    config: RetryConfig,
    f: F,
) -> Result<T, E>
where
    E: TachyonError,
{
    let mut delay = config.initial_delay;
    let mut last_error: Option<E> = None;
    
    for attempt in 1..=config.max_attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);
                
                // Check if error is transient
                if !error.is_transient() {
                    return Err(error);
                }
                
                // Wait before retry
                sleep(delay).await;
                
                // Calculate next delay with exponential backoff
                delay = std::cmp::min(
                    Duration::from_millis_f64(
                        delay.as_millis() as f64 * config.multiplier,
                    ),
                    config.max_delay,
                );
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

---

## 7. ERROR RECOVERY

### 7.1. Recovery Strategies

The Tachyon system implements multiple recovery strategies to handle different error types appropriately.

**7.1.1. Recovery Strategy Matrix**

| Error Type | Recovery Strategy | Implementation | Retry Policy |
|------------|-------------------|----------------|-------------|
| **Transient Errors** | Automatic retry with backoff | Circuit breaker, retry middleware | Exponential backoff |
| **Rate Limiting** | Delay and retry | Retry-After header | Wait specified duration |
| **Timeout Errors** | Retry with timeout | Timeout middleware | Increase timeout, retry |
| **Network Errors** | Reconnection | Connection pool | Exponential backoff |
| **Validation Errors** | User correction | Input validation | No retry, prompt user |
| **Authorization Errors** | Re-authentication | Token refresh | Refresh tokens |
| **Conflict Errors** | Conflict resolution | Optimistic locking | Retry with merge |
| **Database Locks** | Wait and retry | Lock retry | Wait and retry |
| **Permanent Errors** | Graceful degradation | Fallback mechanisms | No retry |

### 7.2. Automatic Recovery Mechanisms

**7.2.1. Retry with Exponential Backoff**

Transient errors trigger automatic retry with exponentially increasing delays.

```rust
use tokio::time::{sleep, Duration};
use std::time::Duration;

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl RetryPolicy {
    /// Creates default retry policy.
    pub fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(30000),
            backoff_multiplier: 2.0,
        }
    }
    
    /// Calculates delay for given attempt.
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            self.initial_delay
        } else {
            let exponential_delay = self.initial_delay
                * self.backoff_multiplier.powi((attempt - 1) as u32);
            
            Duration::from_millis_f64(
                exponential_delay.as_millis() as f64
                    .min(self.max_delay.as_millis() as f64) as u64 as f64,
            )
        }
    }
}

/// Executes operation with retry logic.
pub async fn retry_operation<F, T, E>(
    operation: F,
    policy: &RetryPolicy,
) -> Result<T, E>
where
    E: TachyonError,
{
    let mut last_error: Option<E> = None;
    
    for attempt in 1..=policy.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);
                
                // Check if error is recoverable
                if !error.is_recoverable() {
                    return Err(error);
                }
                
                // Check if error is transient
                if !error.is_transient() {
                    return Err(error);
                }
                
                // Calculate delay
                let delay = policy.calculate_delay(attempt);
                
                // Wait before retry
                sleep(delay).await;
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

**7.2.2. Circuit Breaker Pattern**

Circuit breaker prevents cascading failures by stopping requests to failing services.

```rust
use tokio::sync::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    HalfOpen,
    Open,
}

/// Circuit breaker.
pub struct CircuitBreaker {
    state: Arc<RwLock<Circuit_breaker_state>>,
    failure_count: Arc<RwLock<usize>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    /// Creates new circuit breaker.
    pub fn new(
        failure_threshold: usize,
        success_threshold: usize,
        timeout: Duration,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold,
            success_threshold,
            timeout,
        }
    }
    
    /// Checks if circuit is open.
    pub fn is_open(&self) -> bool {
        matches!(*self.state.read().await, CircuitState::Open | CircuitState::HalfOpen)
    }
    
    /// Records a failure.
    pub fn record_failure(&self) {
        *self.failure_count.write().await += 1;
        *self.last_failure_time.write().await = Some(Instant::now());
        
        let failures = *self.failure_count.read().await;
        if failures >= self.failure_threshold {
            *self.state.write().await = CircuitState::Closed;
        }
    }
    
    /// Records a success.
    pub fn record_success(&self) {
        *self.failure_count.write().await = 0;
        *self.last_failure_time.write().await = None;
        
        let successes = *self.success_count.read().await;
        if successes >= self.success_threshold {
            *self.state.write().await = CircuitState::Open;
        }
    }
    
    /// Executes operation with circuit breaker.
    pub async fn execute<F, T, E>(
        &self,
        operation: F,
    ) -> Result<T, E>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>>,
    {
        // Check if circuit is open
        if !self.is_open() {
            return Err(E::from("Circuit breaker is open")));
        }
        
        // Execute operation with timeout
        let result = tokio::time::timeout(self.timeout, operation()).await;
        
        // Update circuit state based on result
        match &result {
            Ok(_) => self.record_success(),
            Err(_) => self.record_failure(),
        }
        
        result
    }
}
```

**7.2.3. Fallback Mechanisms**

Graceful degradation provides alternative functionality when primary systems fail.

```rust
/// Fallback service for database operations.
pub struct FallbackDatabaseService {
    primary: Arc<DatabaseService>,
    cache: Arc<InMemoryCache>,
}

impl FallbackDatabaseService {
    /// Creates fallback service.
    pub fn new(
        primary: Arc<DatabaseService>,
        cache: Arc<InMemoryCache>,
    ) -> Self {
        Self { primary, cache }
    }
    
    /// Gets document with fallback.
    pub async fn get_document(&self, id: &str) -> Result<Document, DatabaseError> {
        // Try cache first
        if let Some(cached) = self.cache.get(id).await {
            return Ok(cached);
        }
        
        // Try primary database
        match self.primary.get_document(id).await {
            Ok(doc) => {
                // Update cache
                self.cache.insert(id, doc.clone()).await;
                Ok(doc)
            }
            Err(error) => {
                // Check if error is transient
                if error.is_transient() {
                    // Return stale cached data if available
                    if let Some(stale) = self.cache.get(id).await {
                        return Ok(stale);
                    }
                }
                
                Err(error)
            }
        }
    }
}
```

### 7.3. User-Initiated Recovery

Some errors require user intervention for recovery.

**7.3.1. Re-Authentication Flow**

```typescript
/**
 * Initiates re-authentication flow.
 *
 * @returns Promise with new token or error
 */
export async function reAuthenticate(): Promise<string> {
  try {
    // Clear stored tokens
    localStorage.removeItem('auth_token');
    localStorage.removeItem('refresh_token');
    
    // Redirect to authentication
    window.location.href = '/auth/login?redirect=' + encodeURIComponent(window.location.pathname);
    
    return 'Redirecting to authentication...';
  } catch (error) {
    console.error('Re-authentication failed:', error);
    throw error;
  }
}

/**
 * Handles authentication errors with user prompts.
 *
 * @param error - The authentication error
 * @returns User-friendly error message
 */
export function handleAuthError(error: ApiError): string {
  switch (error.errorCode) {
    case 'AUTH_CREDS_001':
      return 'Invalid credentials. Please check your username and password.';
    
    case 'AUTH_TOKEN_001':
      return 'Your session has expired. Please log in again.';
    
    case 'AUTH_MFA_001':
      return 'Multi-factor authentication failed. Please try again.';
    
    case 'AUTH_SESSION_001':
      return 'Your session has expired. Please log in again.';
    
    default:
      return 'Authentication failed. Please try again.';
  }
}
```

**7.3.2. Conflict Resolution UI**

```typescript
/**
 * Displays conflict resolution UI.
 *
 * @param conflict - The conflict error details
 */
export function showConflictResolution(conflict: {
  error: string;
  resourceId: string;
}): void {
  const modal = document.createElement('div');
  modal.className = 'conflict-modal';
  modal.innerHTML = `
    <div class="modal-content">
      <h3>Conflict Detected</h3>
      <p>${conflict.error}</p>
      <p>Resource ID: ${conflict.resourceId}</p>
      <div class="resolution-options">
        <button onclick="resolveConflict('discard')">Discard Changes</button>
        <button onclick="resolveConflict('overwrite')">Overwrite</button>
        <button onclick="resolveConflict('retry')">Retry</button>
      </div>
    </div>
  `;
  
  document.body.appendChild(modal);
}

/**
 * Resolves conflict with chosen action.
 *
 * @param action - The resolution action
 */
export function resolveConflict(action: 'discard' | 'overwrite' | 'retry'): void {
  // Remove modal
  const modal = document.querySelector('.conflict-modal');
  if (modal) {
    modal.remove();
  }
  
  switch (action) {
    case 'discard':
      // Reload current data
      window.location.reload();
      break;
    
    case 'overwrite':
      // Trigger overwrite operation
      console.log('Overwriting changes...');
      break;
    
    case 'retry':
      // Reload and retry operation
      window.location.reload();
      break;
  }
}
```

### 7.4. Degradation Modes

The system supports multiple degradation modes when full functionality is unavailable.

**7.4.1. Degradation Levels**

| Level | Description | Functionality Available | Recovery Strategy |
|-------|-------------|------------------------|-------------------|
| **Full** | All functionality available | 100% | None required |
| **High** | Core functionality available | 90%+ | Non-critical features disabled |
| **Medium** | Basic functionality available | 70%+ | Advanced features disabled |
| **Low** | Minimal functionality available | 40%+ | Read-only mode |
| **Offline** | Local-only mode | 0% | Local cache only |

**7.4.2. Degradation Implementation**

```rust
/// Degradation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationLevel {
    Full,
    High,
    Medium,
    Low,
    Offline,
}

/// System health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub level: DegradationLevel,
    pub affected_services: Vec<String>,
    pub estimated_recovery: Option<Duration>,
}

/// Checks system health and determines degradation level.
pub async fn check_system_health(
    services: Vec<Arc<dyn HealthCheck>>,
) -> SystemHealth {
    let mut level = DegradationLevel::Full;
    let mut affected_services = Vec::new();
    let mut recovery_time = None;
    
    for service in services {
        match service.check_health().await {
            Ok(_) => continue,
            Err(_) => {
                affected_services.push(service.name().to_string());
                level = match level {
                    DegradationLevel::Full => DegradationLevel::High,
                    DegradationLevel::High => DegradationLevel::Medium,
                    DegradationLevel::Medium => DegradationLevel::Low,
                    DegradationLevel::Low => DegradationLevel::Offline,
                    DegradationLevel::Offline => DegradationLevel::Offline,
                };
            }
        }
    }
    
    SystemHealth {
        level,
        affected_services,
        estimated_recovery: recovery_time,
    }
}
```

### 7.5. Recovery Best Practices

**7.5.1. Recovery Guidelines**

1. **Idempotency:** Ensure recovery operations are idempotent
2. **Timeouts:** Use appropriate timeouts for recovery operations
3. **Backoff:** Implement exponential backoff for retries
4. **Circuit Breakers:** Use circuit breakers for failing services
5. **Fallbacks:** Provide fallback mechanisms for critical services
6. **User Notification:** Inform users of recovery actions
7. **Logging:** Log all recovery attempts and outcomes
8. **Testing:** Test recovery mechanisms regularly

**7.5.2. Recovery Anti-Patterns**

| Anti-Pattern | Description | Correct Approach |
|--------------|-------------|------------------|
| **Silent Failure** | Ignoring errors without logging | Always log errors |
| **Infinite Retry** | Retrying without backoff | Implement backoff |
| **Blocking Recovery** | Blocking UI during recovery | Non-blocking recovery |
| **Partial Recovery** | Recovering only part of state | Atomic recovery |
| **Hardcoded Values** | Hardcoded retry limits | Configurable limits |

---

## 8. ERROR LOGGING

### 8.1. Logging Architecture

The Tachyon system implements a comprehensive logging architecture that captures all error events with sufficient context for monitoring, debugging, and audit purposes.

**8.1.1. Logging Layers**

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Structured Logging            │   │
│  │  ┌───────────────────────────────────┐    │   │
│  │  │         Error Events             │    │   │
│  │  │  ┌─────────────────────────┐    │   │
│  │  │  │   Audit Events               │    │   │
│  │  │  └───────────────────────────┘    │   │
│  └─────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│                   Observability Layer                │
│  ┌─────────────────────────────────────────┐   │
│  │              Tracing Framework           │   │
│  │  ┌───────────────────────────────────┐    │   │
│  │  │         Metrics Collection        │    │   │
│  │  │  ┌─────────────────────────┐    │   │
│  │  │  │   Error Metrics              │    │   │
│  │  │  └───────────────────────────┘    │   │
│  │  └─────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│                   Monitoring Layer                 │
│  ┌─────────────────────────────────────────┐   │
│  │              Alerting System             │   │
│  │  ┌───────────────────────────────────┐    │   │
│  │  │  │   Log Aggregation             │    │   │
│  │  │  ┌─────────────────────────┐    │   │
│  │  │  │   Dashboard                 │    │   │
│  │  │  └───────────────────────────┘    │   │
│  │  └─────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**8.1.2. Logging Requirements**

All error events must include the following information:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timestamp` | ISO 8601 string | Yes | When error occurred |
| `error_code` | string | Yes | Error code from Section 4 |
| `severity` | string | Yes | Severity level (CRITICAL, HIGH, MEDIUM, LOW) |
| `message` | string | Yes | Human-readable error description |
| `request_id` | string | Yes | Request correlation identifier |
| `component` | string | Yes | Component that generated error |
| `function` | string | Yes | Function or method where error occurred |
| `line_number` | integer | Yes | Source code line number |
| `user_id` | string | Conditional | User ID if available |
| `session_id` | string | Conditional | Session ID if available |
| `stack_trace` | string | Conditional | Stack trace for debugging |
| `context` | object | Conditional | Additional structured context |
| `security_impact` | string | Yes | Security impact level |

### 8.2. Rust Logging Implementation

**8.2.1. Structured Logging with Tracing**

```rust
use tracing::{error, info, warn, instrument, Level, Span};
use serde_json::json;

/// Error event for logging.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorEvent {
    /// Error code.
    pub code: String,
    
    /// Error message.
    pub message: String,
    
    /// Severity level.
    pub severity: String,
    
    /// Security impact level.
    pub security_impact: String,
    
    /// Timestamp of error.
    pub timestamp: String,
    
    /// Request correlation identifier.
    pub request_id: String,
    
    /// Component that generated error.
    pub component: String,
    
    /// Function or method where error occurred.
    pub function: String,
    
    /// Source code line number.
    pub line_number: Option<u32>,
    
    /// User ID if available.
    pub user_id: Option<String>,
    
    /// Session ID if available.
    pub session_id: Option<String>,
    
    /// Stack trace for debugging.
    pub stack_trace: Option<String>,
    
    /// Additional structured context.
    pub context: Option<serde_json::Value>,
}

impl ErrorEvent {
    /// Creates error event from TachyonError.
    pub fn from_error<E: TachyonError>(error: &E, span: &Span) -> Self {
        Self {
            code: error.error_code().to_string(),
            message: error.to_string(),
            severity: error.severity().to_string(),
            security_impact: error.security_impact().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            request_id: span.id().to_string(),
            component: span.metadata().get("component").map(|v| v.as_str()).unwrap_or_else("unknown".to_string()),
            function: span.name().to_string(),
            line_number: span.metadata().get("code_line_number").and_then(|v| v.as_u64()).ok()),
            user_id: span.metadata().get("user_id").and_then(|v| v.as_str()).ok()),
            session_id: span.metadata().get("session_id").and_then(|v| v.as_str()).ok()),
            stack_trace: span.metadata().get("stack_trace").and_then(|v| v.as_str()).ok()),
            context: Some(error.context()),
        }
    }
    
    /// Converts error event to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "code": self.code,
            "message": self.message,
            "severity": self.severity,
            "security_impact": self.security_impact,
            "timestamp": self.timestamp,
            "request_id": self.request_id,
            "component": self.component,
            "function": self.function,
            "line_number": self.line_number,
            "user_id": self.user_id,
            "session_id": self.session_id,
            "stack_trace": self.stack_trace,
            "context": self.context,
        })
    }
}

/// Logs error with appropriate level.
pub fn log_error<E: TachyonError>(error: &E)
where
    E: TachyonError,
{
    let span = Span::current();
    let event = ErrorEvent::from_error(error, &span);
    
    match event.severity.as_str() {
        "CRITICAL" => {
            error!(
                target: "tachyon.errors",
                error_code = %event.code,
                message = %event.message,
                severity = %event.severity,
                security_impact = %event.security_impact,
                request_id = %event.request_id,
                component = %event.component,
                function = %event.function,
                line_number = ?event.line_number,
                user_id = ?event.user_id,
                session_id = ?event.session_id,
                stack_trace = ?event.stack_trace,
                context = ?event.context,
            );
        }
        "HIGH" => {
            warn!(
                target: "tachyon.errors",
                error_code = %event.code,
                message = %event.message,
                severity = %event.severity,
                security_impact = %event.security_impact,
                request_id = %event.request_id,
                component = %event.component,
                function = %event.function,
                line_number = ?event.line_number,
                user_id = ?event.user_id,
                session_id = ?event.session_id,
                stack_trace = ?event.stack_trace,
                context = ?event.context,
            );
        }
        "MEDIUM" => {
            warn!(
                target: "tachyon.errors",
                error_code = %event.code,
                message = %event.message,
                severity = %event.severity,
                security_impact = %event.security_impact,
                request_id = %event.request_id,
                component = %event.component,
                function = %event.function,
                line_number = ?event.line_number,
                user_id = ?event.user_id,
                session_id = ?event.session_id,
                stack_trace = ?event.stack_trace,
                context = ?event.context,
            );
        }
        "LOW" => {
            debug!(
                target: "tachyon.errors",
                error_code = %event.code,
                message = %event.message,
                severity = %event.severity,
                security_impact = %event.security_impact,
                request_id = %event.request_id,
                component = %event.component,
                function = %event.function,
                line_number = ?event.line_number,
                user_id = ?event.user_id,
                session_id = ?event.session_id,
                stack_trace = ?event.stack_trace,
                context = ?event.context,
            );
        }
    }
}
```

**8.2.2. Audit Logging**

Audit logging captures security-relevant events for compliance and forensic analysis.

```rust
/// Audit event for security logging.
#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    /// Event type.
    pub event_type: AuditEventType,
    
    /// Error code.
    pub error_code: String,
    
    /// User ID if available.
    pub user_id: Option<String>,
    
    /// Session ID if available.
    pub session_id: Option<String>,
    
    /// IP address.
    pub ip_address: Option<String>,
    
    /// User agent.
    pub user_agent: Option<String>,
    
    /// Timestamp.
    pub timestamp: String,
    
    /// Additional context.
    pub details: Option<serde_json::Value>,
}

/// Audit event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Authentication failure.
    AuthenticationFailure,
    
    /// Authorization failure.
    AuthorizationFailure,
    
    /// Data access attempt.
    DataAccessAttempt,
    
    /// Configuration change.
    ConfigurationChange,
    
    /// System error.
    SystemError,
}

impl AuditEvent {
    /// Creates audit event from error.
    pub fn from_error<E: TachyonError>(error: &E) -> Self {
        let event_type = match error.error_code() {
            code if code.starts_with("SRV_AUTH") || code.starts_with("AUTH_") => {
                AuditEventType::AuthenticationFailure
            }
            code if code.starts_with("SRV_AUTH_002") || code.starts_with("AUTHZ_") => {
                AuditEventType::AuthorizationFailure
            }
            _ => AuditEventType::SystemError,
        };
        
        Self {
            event_type,
            error_code: error.error_code().to_string(),
            user_id: error.context().get("user_id").and_then(|v| v.as_str()).ok()),
            session_id: error.context().get("session_id").and_then(|v| v.as_str()).ok()),
            ip_address: error.context().get("ip_address").and_then(|v| v.as_str()).ok(),
            user_agent: error.context().get("user_agent").and_then(|v| v.as_str()).ok()),
            timestamp: Utc::now().to_rfc3339(),
            details: Some(error.context()),
        }
    }
}
```

### 8.3. TypeScript Logging Implementation

**8.3.1. Client-Side Error Logging**

```typescript
/**
 * Error logger for client-side error handling.
 */
export class ErrorLogger {
  private static instance: ErrorLogger;
  
  private constructor() {
    this.instance = this;
  }
  
  /**
   * Gets singleton instance.
   *
   * @returns ErrorLogger instance
   */
  public static getInstance(): ErrorLogger {
    if (!ErrorLogger.instance) {
      ErrorLogger.instance = new ErrorLogger();
    }
    return ErrorLogger.instance;
  }
  
  /**
   * Logs error with appropriate level.
   *
   * @param error - The error to log
   * @param level - Log level
   */
  public log(error: ApiError, level: ErrorSeverity = ErrorSeverity.Medium): void {
    const logEntry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      errorCode: error.errorCode,
      message: error.message,
      severity: error.severity,
      recoverable: error.recoverable,
      transient: error.transient,
      securityImpact: error.securityImpact,
      requestId: error.requestId,
      userId: this.getUserId(),
      sessionId: this.getSessionId(),
    };
    
    // Log to console
    switch (level) {
      case ErrorSeverity.Critical:
        console.error('CRITICAL:', logEntry);
        break;
      
      case ErrorSeverity.High:
        console.warn('HIGH:', logEntry);
        break;
      
      case ErrorSeverity.Medium:
        console.info('MEDIUM:', logEntry);
        break;
      
      case ErrorSeverity.Low:
        console.debug('LOW:', logEntry);
        break;
    }
    
    // Send to server for audit logging
    this.sendToServer(logEntry);
  }
  
  /**
   * Gets current user ID from storage.
   *
   * @returns User ID or null
   */
  private getUserId(): string | null {
    try {
      const userStr = localStorage.getItem('user_id');
      return userStr ? JSON.parse(userStr).id : null;
    } catch {
      return null;
    }
  }
  
  /**
   * Gets current session ID from storage.
   *
   * @returns Session ID or null
   */
  private getSessionId(): string | null {
    try {
      const sessionStr = localStorage.getItem('session_id');
      return sessionStr ? JSON.parse(sessionStr).id : null;
    } catch {
      return null;
    }
  }
  
  /**
   * Sends log entry to server.
   *
   * @param entry - The log entry to send
   */
  private async sendToServer(entry: LogEntry): Promise<void> {
    try {
      const response = await fetch('/api/logs', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(entry),
      });
      
      if (!response.ok) {
        console.warn('Failed to send error log to server:', entry);
      }
    } catch (error) {
      console.error('Error sending error log:', error);
    }
  }
}

/**
 * Log entry structure.
 */
export interface LogEntry {
  timestamp: string;
  level: ErrorSeverity;
  errorCode: string;
  message: string;
  severity: ErrorSeverity;
  recoverable: boolean;
  transient: boolean;
  securityImpact: SecurityImpact;
  requestId: string;
  userId: string | null;
  sessionId: string | null;
}
```

**8.3.2. Error Metrics Collection**

```typescript
/**
 * Error metrics collector for monitoring.
 */
export class ErrorMetrics {
  private static metrics: Map<string, ErrorMetric>;
  
  /**
   * Records error metric.
   *
   * @param error - The error to record
   */
  public static record(error: ApiError): void {
    const key = error.errorCode;
    
    if (!this.metrics.has(key)) {
      this.metrics.set(key, {
        count: 0,
        firstSeen: Date.now(),
        lastSeen: Date.now(),
      });
    }
    
    const metric = this.metrics.get(key);
    metric.count++;
    metric.lastSeen = Date.now();
    
    // Alert on threshold exceeded
    if (metric.count >= this.getThreshold(key)) {
      this.alert(metric, error);
    }
  }
  
  /**
   * Gets alert threshold for error code.
   *
   * @param errorCode - The error code
   * @returns Alert threshold
   */
  private getThreshold(errorCode: string): number {
    const thresholds: Record<string, number> = {
      'SRV_AUTH_001': 5,
      'SRV_RATE_001': 100,
      'SRV_TIMEOUT_001': 50,
      'DB_CORR_001': 1,
    'SRV_INT_001': 10,
    'SRV_VAL_001': 20,
    'SRV_NOTF_001': 15,
    'SRV_CONFLICT_001': 10,
    'SRV_AUTH_002': 5,
      'AUTHZ_PERM_001': 5,
    'NET_TIMEOUT_001': 30,
      'NET_CONN_001': 20,
    'VAL_SCHEMA_001': 10,
      'VAL_TYPE_001': 15,
    'VAL_REQUIRED_001': 10,
    'VAL_FORMAT_001': 10,
    'VAL_LENGTH_001': 10,
    'VAL_PATTERN_001': 10,
    'GIT_MERGE_001': 5,
      'GIT_CLONE_001': 5,
      'GIT_PUSH_001': 5,
      'SRCH_TIMEOUT_001': 20,
      'SRCH_INDEX_001': 1,
      'SRCH_CORRUPT_001': 1,
    };
    
    return thresholds[errorCode] || 10;
  }
  
  /**
   * Alerts on metric threshold exceeded.
   *
   * @param metric - The metric to alert on
   * @param error - The error that triggered the metric
   */
  private alert(metric: ErrorMetric, error: ApiError): void {
    console.warn(`Error rate alert: ${metric.key} - ${metric.count} occurrences in last hour`, error);
    
    // Send alert to monitoring system
    this.sendAlert({
      type: 'error_rate_alert',
      errorCode: error.errorCode,
      count: metric.count,
      threshold: this.getThreshold(error.errorCode),
      message: error.message,
    });
  }
  
  /**
   * Gets all metrics for reporting.
   *
   * @returns All error metrics
   */
  public static getAllMetrics(): ErrorMetric[] {
    return Array.from(this.metrics.entries());
  }
  
  /**
   * Sends alert to monitoring system.
   *
   * @param alert - The alert data
   */
  private async sendAlert(alert: AlertData): Promise<void> {
    try {
      const response = await fetch('/api/alerts', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(alert),
      });
      
      if (!response.ok) {
        console.warn('Failed to send alert:', alert);
      }
    } catch (error) {
      console.error('Error sending alert:', error);
    }
  }
}

/**
 * Error metric data.
 */
export interface ErrorMetric {
  count: number;
  firstSeen: Date;
  lastSeen: Date;
}

/**
 * Alert data for monitoring system.
 */
export interface AlertData {
  type: 'error_rate_alert';
  errorCode: string;
  count: number;
  threshold: number;
  message: string;
}
```

### 8.4. Logging Best Practices

**8.4.1. Log Level Guidelines**

| Severity | When to Use | Log Level | Example Scenarios |
|----------|----------------|-------------|-------------------|
| **CRITICAL** | System cannot continue; immediate intervention required | Critical system failures, data corruption |
| **HIGH** | Feature unavailable; high priority | Authentication failures, authorization failures, rate limiting |
| **MEDIUM** | Degraded performance; medium priority | Timeouts, cache misses, slow queries |
| **LOW** | Cosmetic or informational issues; low priority | Validation warnings, minor UI issues |

**8.4.2. Sensitive Data Handling**

Never log sensitive data in error messages. Use structured logging for sensitive information.

```rust
/// Sanitizes error context before logging.
pub fn sanitize_error_context(context: &serde_json::Value) -> serde_json::Value {
    let mut sanitized = json!({});
    
    for (key, value) in context.as_object().unwrap() {
        let key_str = key.as_str().unwrap();
        
        // Remove sensitive fields
        if key_str.contains("password") 
            || key_str.contains("token") 
            || key_str.contains("secret") 
            || key_str.contains("credential") 
            || key_str.contains("key") 
            || key_str.contains("auth") 
        {
            continue;
        }
        
        // Sanitize string values
        if let Some(string_val) = value.as_str() {
            if string_val.len() > 100 {
                sanitized[key] = "***REDACTED***";
            } else {
                sanitized[key] = string_val;
            }
        }
        
        sanitized
    }
}
```

**8.4.3. Performance Considerations**

Logging must not significantly impact application performance.

```rust
use std::sync::mpsc;
use tokio::sync::RwLock;

/// Async logger with bounded channel.
pub struct AsyncLogger {
    sender: mpsc::Sender<ErrorEvent>,
    _shutdown: Arc<RwLock<bool>>,
}

impl AsyncLogger {
    /// Creates new async logger.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(1000);
        
        Self {
            sender,
            receiver,
            _shutdown: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Logs error event.
    pub async fn log(&self, event: ErrorEvent) {
        if *self._shutdown.read().await {
            return;
        }
        
        let _ = self.sender.send(event).await;
    if _.is_err() {
            eprintln!("Failed to log error event: channel closed");
        }
    }
    
    /// Shuts down logger.
    pub fn shutdown(&self) {
        *self._shutdown.write().await = true;
        
        // Wait for pending logs
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Close channel
        drop(self.sender);
    }
}
```

---

## 9. ERROR SECURITY

### 9.1. Security Principles

Error handling in the Tachyon system is designed with security as a primary concern. The following principles govern all error handling implementations:

**9.1.1. Information Disclosure Prevention**

Error messages must not reveal sensitive information that could be used by attackers to compromise the system. This includes:
- Internal file paths and directory structures
- Database schema details
- Internal API endpoints
- Stack traces in production environments
- Authentication and authorization implementation details
- Third-party library versions and configurations

**Formal Property:**
$$
\forall e: \text{error}, \text{expose}(e) \cap \text{sensitive\_data} = \emptyset
$$

This property states that for all errors, the exposed information must not intersect with sensitive data.

**9.1.2. Error Message Sanitization**

All error messages must be sanitized before being exposed to clients. The system implements a two-tier messaging strategy:

| Tier | Purpose | Content | Exposure |
|------|---------|---------|----------|
| **Internal** | Debugging and troubleshooting | Full error details, stack traces, context | Logs and monitoring only |
| **External** | User communication | Generic, actionable messages | API responses and UI |

**9.1.3. Error Timing Attacks Prevention**

The system must not reveal information through timing differences in error handling. Error response times must be consistent regardless of the specific error condition to prevent timing-based information leakage.

**Formal Property:**
$$
\forall e_1, e_2: \text{error}, |\text{response\_time}(e_1) - \text{response\_time}(e_2)| < \epsilon
$$

Where $\epsilon$ is a small timing threshold.

### 9.2. Sensitive Data Classification

The following data categories are classified as sensitive and must not be exposed in error messages:

**9.2.1. Authentication and Authorization Data**

| Data Type | Classification | Exposure Policy |
|-----------|----------------|-----------------|
| Passwords | Critical | Never expose, even in logs |
| API keys | Critical | Never expose, even in logs |
| Session tokens | High | Expose only last 4 characters in logs |
| JWT payloads | High | Expose only non-sensitive claims in logs |
| Permission sets | Medium | Expose only high-level categories in logs |

**9.2.2. Infrastructure Data**

| Data Type | Classification | Exposure Policy |
|-----------|----------------|-----------------|
| Internal IP addresses | High | Expose only in internal logs |
| Database connection strings | Critical | Never expose |
| File system paths | Medium | Expose only relative paths |
| Container identifiers | Low | Expose in internal logs |
| Service discovery endpoints | Medium | Expose only in internal logs |

**9.2.3. Business Logic Data**

| Data Type | Classification | Exposure Policy |
|-----------|----------------|-----------------|
| User personal data | High | Expose only in user's own logs |
| Financial data | Critical | Never expose in error messages |
| Content metadata | Low | Expose sanitized versions |
| Search queries | Medium | Expose only in user's own logs |

### 9.3. Error Message Sanitization Implementation

**9.3.1. Rust Sanitization Trait**

```rust
/// Trait for sanitizing error messages for external exposure.
pub trait SanitizeError {
    /// Returns a sanitized error message safe for external exposure.
    fn sanitize(&self) -> String;

    /// Returns the full error message for internal logging.
    fn internal_message(&self) -> String;
}

/// Sanitization configuration.
#[derive(Debug, Clone)]
pub struct SanitizationConfig {
    /// Whether to include stack traces in internal logs.
    pub include_stack_traces: bool,

    /// Whether to include file paths in internal logs.
    pub include_file_paths: bool,

    /// Whether to redact sensitive data patterns.
    pub redact_sensitive_patterns: bool,
}

impl Default for SanitizationConfig {
    fn default() -> Self {
        Self {
            include_stack_traces: true,
            include_file_paths: true,
            redact_sensitive_patterns: true,
        }
    }
}

/// Sanitization patterns for sensitive data.
pub struct SanitizationPatterns;

impl SanitizationPatterns {
    /// Patterns that should be redacted from error messages.
    pub const PATTERNS: &[(&str, &str)] = &[
        // Password patterns
        (r"(?i)password\s*[:=]\s*\S+", "password=***"),
        (r"(?i)pwd\s*[:=]\s*\S+", "pwd=***"),
        
        // API key patterns
        (r"(?i)api[_-]?key\s*[:=]\s*\S+", "api_key=***"),
        (r"(?i)secret[_-]?key\s*[:=]\s*\S+", "secret_key=***"),
        
        // Token patterns
        (r"(?i)token\s*[:=]\s*[A-Za-z0-9\-_]{20,}", "token=***"),
        (r"Bearer\s+[A-Za-z0-9\-_\.]{20,}", "Bearer ***"),
        
        // Database connection strings
        (r"postgresql://[^@]+@[^/]+/[^\\s]+", "postgresql://***@***/***"),
        (r"mysql://[^@]+@[^/]+/[^\\s]+", "mysql://***@***/***"),
        
        // File paths (absolute)
        (r"/(?:[^/]+/)+", "/"),
        
        // IP addresses
        (r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}", "***.***.***.***"),
    ];
}
```

**9.3.2. Sanitization Implementation**

```rust
use regex::Regex;
use once_cell::sync::Lazy;

/// Compiled regex patterns for sanitization.
static SANITIZATION_REGEXES: Lazy<Vec<Regex>> = Lazy::new(|| {
    SanitizationPatterns::PATTERNS
        .iter()
        .map(|(pattern, _)| Regex::new(pattern).expect("Invalid regex pattern"))
        .collect()
});

/// Sanitizes a message by replacing sensitive patterns.
pub fn sanitize_message(message: &str) -> String {
    let mut sanitized = message.to_string();
    
    for (i, regex) in SANITIZATION_REGEXES.iter().enumerate() {
        let replacement = SanitizationPatterns::PATTERNS[i].1;
        sanitized = regex.replace_all(&sanitized, replacement).to_string();
    }
    
    sanitized
}

/// Sanitizes a JSON value by redacting sensitive fields.
pub fn sanitize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sanitized_map = serde_json::Map::new();
            for (key, val) in map {
                if is_sensitive_field(key) {
                    sanitized_map.insert(key.clone(), serde_json::Value::String("***".to_string()));
                } else {
                    sanitized_map.insert(key.clone(), sanitize_json(val));
                }
            }
            serde_json::Value::Object(sanitized_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sanitize_json).collect())
        }
        _ => value.clone(),
    }
}

/// Returns true if a field name indicates sensitive data.
fn is_sensitive_field(field_name: &str) -> bool {
    let field_lower = field_name.to_lowercase();
    matches!(
        field_lower.as_str(),
        "password" | "pwd" | "api_key" | "apikey" | "secret" | "token"
            | "authorization" | "auth" | "credential" | "credit_card"
            | "ssn" | "social_security_number" | "private_key"
    )
}

impl SanitizeError for ServerError {
    fn sanitize(&self) -> String {
        match self {
            ServerError::Authentication => "Authentication failed".to_string(),
            ServerError::Authorization(resource) => {
                format!("Access denied to resource")
            }
            ServerError::NotFound(resource) => {
                format!("Resource not found")
            }
            ServerError::Validation(msg) => {
                format!("Validation failed: {}", sanitize_message(msg))
            }
            ServerError::Database(_) => "Database operation failed".to_string(),
            ServerError::Git(_) => "Git operation failed".to_string(),
            ServerError::Search(_) => "Search operation failed".to_string(),
            ServerError::RateLimit => "Rate limit exceeded".to_string(),
            ServerError::Conflict => "Concurrent modification conflict".to_string(),
            ServerError::Timeout => "Request timeout".to_string(),
            ServerError::Internal(_) => "Internal server error".to_string(),
        }
    }

    fn internal_message(&self) -> String {
        self.to_string()
    }
}
```

**9.3.3. TypeScript Sanitization Implementation**

```typescript
/**
 * Sanitizes error messages for external exposure.
 */
export class ErrorSanitizer {
  private static readonly SENSITIVE_PATTERNS: Array<{
    pattern: RegExp;
    replacement: string;
  }> = [
    // Password patterns
    { pattern: /password\s*[:=]\s*\S+/gi, replacement: 'password=***' },
    { pattern: /pwd\s*[:=]\s*\S+/gi, replacement: 'pwd=***' },
    
    // API key patterns
    { pattern: /api[_-]?key\s*[:=]\s*\S+/gi, replacement: 'api_key=***' },
    { pattern: /secret[_-]?key\s*[:=]\s*\S+/gi, replacement: 'secret_key=***' },
    
    // Token patterns
    { pattern: /token\s*[:=]\s*[A-Za-z0-9\-_]{20,}/gi, replacement: 'token=***' },
    { pattern: /Bearer\s+[A-Za-z0-9\-_\.]{20,}/g, replacement: 'Bearer ***' },
    
    // Database connection strings
    { pattern: /postgresql:\/\/[^@]+@[^/]+\/[^\s]+/g, replacement: 'postgresql://***@***/***' },
    { pattern: /mysql:\/\/[^@]+@[^/]+\/[^\s]+/g, replacement: 'mysql://***@***/***' },
    
    // File paths (absolute)
    { pattern: /(?:\/[^/]+)+/g, replacement: '/' },
    
    // IP addresses
    { pattern: /\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}/g, replacement: '***.***.***.***' },
  ];

  private static readonly SENSITIVE_FIELDS: Set<string> = new Set([
    'password', 'pwd', 'api_key', 'apikey', 'secret', 'token',
    'authorization', 'auth', 'credential', 'credit_card',
    'ssn', 'social_security_number', 'private_key',
  ]);

  /**
   * Sanitizes a message by replacing sensitive patterns.
   */
  static sanitizeMessage(message: string): string {
    let sanitized = message;
    
    for (const { pattern, replacement } of this.SENSITIVE_PATTERNS) {
      sanitized = sanitized.replace(pattern, replacement);
    }
    
    return sanitized;
  }

  /**
   * Sanitizes an object by redacting sensitive fields.
   */
  static sanitizeObject<T extends Record<string, unknown>>(obj: T): T {
    const sanitized = { ...obj };
    
    for (const key of Object.keys(sanitized)) {
      if (this.SENSITIVE_FIELDS.has(key.toLowerCase())) {
        (sanitized as Record<string, unknown>)[key] = '***';
      } else if (typeof sanitized[key] === 'object' && sanitized[key] !== null) {
        (sanitized as Record<string, unknown>)[key] = this.sanitizeObject(
          sanitized[key] as Record<string, unknown>
        );
      }
    }
    
    return sanitized;
  }

  /**
   * Sanitizes an error for external exposure.
   */
  static sanitizeError(error: Error): SanitizedError {
    return {
      message: this.sanitizeMessage(error.message),
      name: error.name,
      code: (error as any).code,
    };
  }
}

/**
 * Sanitized error interface for external exposure.
 */
export interface SanitizedError {
  message: string;
  name: string;
  code?: string;
}
```

### 9.4. Error Response Security Headers

The following security headers must be included in all error responses:

**9.4.1. Required Headers**

| Header | Purpose | Value |
|--------|---------|-------|
| `X-Content-Type-Options` | Prevent MIME sniffing | `nosniff` |
| `X-Frame-Options` | Prevent clickjacking | `DENY` |
| `Content-Security-Policy` | Restrict content sources | `default-src 'self'` |
| `X-Request-ID` | Request correlation | UUID |
| `Cache-Control` | Prevent error caching | `no-store, no-cache, must-revalidate` |
| `Pragma` | HTTP/1.0 cache control | `no-cache` |

**9.4.2. Conditional Headers**

| Header | Condition | Value |
|--------|-----------|-------|
| `Retry-After` | Rate limit or temporary unavailability | Seconds or HTTP-date |
| `WWW-Authenticate` | Authentication required | Authentication scheme |
| `Access-Control-Allow-Origin` | CORS enabled | Origin(s) |

**9.4.3. Security Header Implementation**

```rust
use axum::http::{HeaderMap, HeaderValue};
use uuid::Uuid;

/// Adds security headers to error responses.
pub fn add_security_headers(headers: &mut HeaderMap, request_id: Uuid) {
    // Prevent MIME type sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    
    // Prevent clickjacking
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    
    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'"),
    );
    
    // Request correlation
    headers.insert(
        "X-Request-ID",
        HeaderValue::from_str(&request_id.to_string())
            .expect("Invalid UUID"),
    );
    
    // Prevent caching of error responses
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    
    headers.insert(
        "Pragma",
        HeaderValue::from_static("no-cache"),
    );
}

/// Adds conditional security headers based on error type.
pub fn add_conditional_headers(
    headers: &mut HeaderMap,
    error: &dyn TachyonError,
) {
    // Add Retry-After for rate limits and temporary errors
    if error.is_transient() {
        headers.insert(
            "Retry-After",
            HeaderValue::from_str("60").expect("Invalid value"),
        );
    }
    
    // Add WWW-Authenticate for authentication errors
    if error.security_impact() == SecurityImpact::High {
        if error.error_code().contains("AUTH") {
            headers.insert(
                "WWW-Authenticate",
                HeaderValue::from_static("Bearer"),
            );
        }
    }
}
```

### 9.5. Error Logging Security

**9.5.1. Log Data Classification**

| Log Level | Data Exposure | Retention | Access Control |
|-----------|---------------|-----------|----------------|
| **ERROR** | Full error details | 1 year | Admin only |
| **WARN** | Sanitized messages | 6 months | Admin + DevOps |
| **INFO** | No error data | 3 months | All authorized users |
| **DEBUG** | Full details including stack traces | 30 days | Admin only |

**9.5.2. Sensitive Data Redaction in Logs**

All log entries must be sanitized to remove sensitive data:

```rust
/// Redacts sensitive data from log entries.
pub struct LogRedactor;

impl LogRedactor {
    /// Redacts sensitive data from a log entry.
    pub fn redact(entry: &str) -> String {
        let mut redacted = entry.to_string();
        
        // Redact passwords
        redacted = redacted.replace(
            r"password\s*[:=]\s*\S+",
            "password=***",
        );
        
        // Redact API keys
        redacted = redacted.replace(
            r"api[_-]?key\s*[:=]\s*\S+",
            "api_key=***",
        );
        
        // Redact tokens
        redacted = redacted.replace(
            r"token\s*[:=]\s*[A-Za-z0-9\-_]{20,}",
            "token=***",
        );
        
        // Redact IP addresses
        redacted = redacted.replace(
            r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
            "***.***.***.***",
        );
        
        redacted
    }
    
    /// Redacts sensitive fields from a JSON object.
    pub fn redact_json(value: &serde_json::Value) -> serde_json::Value {
        sanitize_json(value)
    }
}
```

### 9.6. Error Rate Limiting

**9.6.1. Rate Limiting Strategy**

To prevent error-based attacks, the system implements rate limiting on error responses:

| Error Type | Rate Limit | Burst | Penalty |
|------------|------------|-------|---------|
| Authentication failures | 5/minute | 10 | 1 minute block |
| Authorization failures | 10/minute | 20 | 30 second block |
| Validation failures | 20/minute | 50 | No penalty |
| Rate limit errors | 100/minute | 200 | No penalty |

**9.6.2. Rate Limiting Implementation**

```rust
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limit entry for tracking error rates.
#[derive(Debug, Clone)]
pub struct RateLimitEntry {
    pub error_count: u32,
    pub last_error: Instant,
    pub blocked_until: Option<Instant>,
}

/// Rate limiter for error responses.
pub struct ErrorRateLimiter {
    entries: RwLock<HashMap<IpAddr, RateLimitEntry>>,
    config: RateLimitConfig,
}

/// Rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_errors_per_minute: u32,
    pub burst_size: u32,
    pub block_duration: Duration,
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_errors_per_minute: 10,
            burst_size: 20,
            block_duration: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(300),
        }
    }
}

impl ErrorRateLimiter {
    /// Creates a new rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            config,
        }
    }
    
    /// Checks if a client should be rate limited.
    pub async fn check_rate_limit(
        &self,
        client_ip: IpAddr,
    ) -> Result<(), RateLimitError> {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        
        let entry = entries.entry(client_ip).or_insert_with(|| {
            RateLimitEntry {
                error_count: 0,
                last_error: now,
                blocked_until: None,
            }
        });
        
        // Check if currently blocked
        if let Some(blocked_until) = entry.blocked_until {
            if now < blocked_until {
                return Err(RateLimitError::Blocked {
                    retry_after: (blocked_until - now).as_secs(),
                });
            } else {
                entry.blocked_until = None;
            }
        }
        
        // Check rate limit
        let time_since_last_error = now.duration_since(entry.last_error);
        
        if time_since_last_error < Duration::from_secs(60) {
            entry.error_count += 1;
            
            if entry.error_count > self.config.max_errors_per_minute {
                entry.blocked_until = Some(now + self.config.block_duration);
                return Err(RateLimitError::Blocked {
                    retry_after: self.config.block_duration.as_secs(),
                });
            }
        } else {
            // Reset counter after 1 minute
            entry.error_count = 1;
        }
        
        entry.last_error = now;
        
        Ok(())
    }
    
    /// Cleans up old entries.
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        
        entries.retain(|_, entry| {
            now.duration_since(entry.last_error) < self.config.cleanup_interval
        });
    }
}

/// Rate limit error.
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded, retry after {retry_after} seconds")]
    Blocked { retry_after: u64 },
}
```

---

## 10. ERROR PERFORMANCE

### 10.1. Performance Requirements

Error handling in the Tachyon system must meet strict performance requirements to ensure that error conditions do not degrade system performance or user experience.

**10.1.1. Performance Metrics**

| Metric | Requirement | Measurement Method |
|--------|-------------|-------------------|
| **Error detection latency** | < 1ms | Time from error occurrence to detection |
| **Error classification latency** | < 100μs | Time from detection to classification |
| **Error response generation** | < 5ms | Time from classification to response |
| **Error logging latency** | < 10ms (async) | Time from logging call to queue |
| **Error propagation overhead** | < 5% | Additional latency due to error handling |
| **Memory overhead per error** | < 1KB | Memory allocated per error instance |
| **Error serialization time** | < 1ms | Time to serialize error to JSON |

**10.1.2. Performance Invariants**

**Formal Property 1: Error Detection Latency**
$$
\forall e: \text{error}, \text{detection\_latency}(e) < 1\text{ms}
$$

**Formal Property 2: Error Response Time**
$$
\forall e: \text{error}, \text{response\_time}(e) < 6\text{ms}
$$

**Formal Property 3: Memory Bound**
$$
\forall e: \text{error}, \text{memory\_size}(e) < 1\text{KB}
$$

### 10.2. Error Handling Optimization Strategies

**10.2.1. Zero-Copy Error Propagation**

Error propagation should minimize memory allocations and copying:

```rust
/// Zero-copy error propagation using `Cow` (Clone on Write).
use std::borrow::Cow;

pub struct OptimizedError {
    /// Error code (static string, no allocation).
    pub error_code: &'static str,
    
    /// Error message (borrowed or owned).
    pub message: Cow<'static, str>,
    
    /// Context data (lazy evaluation).
    pub context: LazyContext,
}

/// Lazy context evaluation to avoid unnecessary allocations.
pub struct LazyContext {
    /// Function to generate context on demand.
    generator: Option<Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
    
    /// Cached context value.
    cached: Option<serde_json::Value>,
}

impl LazyContext {
    /// Creates a new lazy context.
    pub fn new<F>(generator: F) -> Self
    where
        F: Fn() -> serde_json::Value + Send + Sync + 'static,
    {
        Self {
            generator: Some(Box::new(generator)),
            cached: None,
        }
    }
    
    /// Returns the context value, generating if necessary.
    pub fn get(&mut self) -> &serde_json::Value {
        if self.cached.is_none() {
            if let Some(generator) = self.generator.take() {
                self.cached = Some(generator());
            }
        }
        self.cached.as_ref().unwrap()
    }
}
```

**10.2.2. Error Pooling**

Frequently used error types should be pooled to reduce allocation overhead:

```rust
use std::sync::Arc;

/// Error pool for reusing error instances.
pub struct ErrorPool<T: Clone> {
    pool: Arc<tokio::sync::Mutex<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone + Send + 'static> ErrorPool<T> {
    /// Creates a new error pool.
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            factory: Arc::new(factory),
        }
    }
    
    /// Acquires an error from the pool.
    pub async fn acquire(&self) -> T {
        let mut pool = self.pool.lock().await;
        if let Some(error) = pool.pop() {
            error
        } else {
            (self.factory)()
        }
    }
    
    /// Returns an error to the pool.
    pub async fn release(&self, error: T) {
        let mut pool = self.pool.lock().await;
        if pool.len() < 100 {
            pool.push(error);
        }
    }
}

/// Pre-allocated error pool for common errors.
pub struct CommonErrorPool {
    validation_errors: ErrorPool<ServerError>,
    not_found_errors: ErrorPool<ServerError>,
    rate_limit_errors: ErrorPool<ServerError>,
}

impl CommonErrorPool {
    /// Creates a new common error pool.
    pub fn new() -> Self {
        Self {
            validation_errors: ErrorPool::new(|| {
                ServerError::Validation("Invalid input".to_string())
            }),
            not_found_errors: ErrorPool::new(|| {
                ServerError::NotFound("Resource not found".to_string())
            }),
            rate_limit_errors: ErrorPool::new(|| {
                ServerError::RateLimit
            }),
        }
    }
    
    /// Acquires a validation error.
    pub async fn validation_error(&self) -> ServerError {
        self.validation_errors.acquire().await
    }
    
    /// Acquires a not found error.
    pub async fn not_found_error(&self) -> ServerError {
        self.not_found_errors.acquire().await
    }
    
    /// Acquires a rate limit error.
    pub async fn rate_limit_error(&self) -> ServerError {
        self.rate_limit_errors.acquire().await
    }
}
```

**10.2.3. Async Error Logging**

Error logging must be asynchronous to avoid blocking request processing:

```rust
use tokio::sync::mpsc;

/// Async error logger with bounded channel.
pub struct AsyncErrorLogger {
    sender: mpsc::Sender<ErrorEvent>,
    _handle: tokio::task::JoinHandle<()>,
}

impl AsyncErrorLogger {
    /// Creates a new async error logger.
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer_size);
        
        let handle = tokio::spawn(async move {
            let mut receiver = receiver;
            
            while let Some(event) = receiver.recv().await {
                // Process error event asynchronously
                Self::process_event(event).await;
            }
        });
        
        Self {
            sender,
            _handle: handle,
        }
    }
    
    /// Logs an error event asynchronously.
    pub async fn log(&self, event: ErrorEvent) -> Result<(), LogError> {
        self.sender.send(event).await
            .map_err(|_| LogError::ChannelClosed)
    }
    
    /// Processes an error event.
    async fn process_event(event: ErrorEvent) {
        // Write to log file
        // Send to monitoring system
        // Trigger alerts if necessary
    }
}
```

### 10.3. Error Caching

**10.3.1. Error Response Caching**

Frequent error responses should be cached to reduce processing overhead:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use lru::LruCache;

/// Cached error response.
#[derive(Clone)]
pub struct CachedErrorResponse {
    response: axum::http::Response<axum::body::Body>,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedErrorResponse {
    /// Creates a new cached error response.
    pub fn new(response: axum::http::Response<axum::body::Body>, ttl: Duration) -> Self {
        Self {
            response,
            cached_at: Instant::now(),
            ttl,
        }
    }
    
    /// Returns true if the cached response is still valid.
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
    
    /// Returns the cached response.
    pub fn response(&self) -> axum::http::Response<axum::body::Body> {
        self.response.clone()
    }
}

/// Error response cache.
pub struct ErrorResponseCache {
    cache: tokio::sync::Mutex<LruCache<String, CachedErrorResponse>>,
    default_ttl: Duration,
}

impl ErrorResponseCache {
    /// Creates a new error response cache.
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            cache: tokio::sync::Mutex::new(LruCache::new(capacity)),
            default_ttl,
        }
    }
    
    /// Gets a cached error response.
    pub async fn get(&self, error_code: &str) -> Option<axum::http::Response<axum::body::Body>> {
        let mut cache = self.cache.lock().await;
        
        if let Some(cached) = cache.get_mut(error_code) {
            if cached.is_valid() {
                return Some(cached.response());
            }
        }
        
        None
    }
    
    /// Inserts an error response into the cache.
    pub async fn insert(
        &self,
        error_code: String,
        response: axum::http::Response<axum::body::Body>,
    ) {
        let mut cache = self.cache.lock().await;
        
        cache.put(
            error_code,
            CachedErrorResponse::new(response, self.default_ttl),
        );
    }
}
```

**10.3.2. Cache Configuration**

| Error Type | Cache TTL | Cache Size | Eviction Policy |
|------------|-----------|------------|-----------------|
| Validation errors | 5 minutes | 1000 | LRU |
| Not found errors | 10 minutes | 500 | LRU |
| Rate limit errors | 1 minute | 100 | LRU |
| Authentication errors | 1 minute | 100 | LRU |
| Internal errors | No caching | N/A | N/A |

### 10.4. Error Metrics and Monitoring

**10.4.1. Performance Metrics Collection**

```rust
use prometheus::{Counter, Histogram, Gauge};

/// Error handling metrics.
pub struct ErrorMetrics {
    /// Total errors by type.
    errors_total: CounterVec,
    
    /// Error latency histogram.
    error_latency: HistogramVec,
    
    /// Active errors gauge.
    active_errors: GaugeVec,
    
    /// Error rate by type.
    error_rate: GaugeVec,
}

impl ErrorMetrics {
    /// Creates a new error metrics collector.
    pub fn new() -> Self {
        Self {
            errors_total: register_counter_vec!(
                "tachyon_errors_total",
                "Total number of errors",
                &["error_type", "severity"]
            ).unwrap(),
            
            error_latency: register_histogram_vec!(
                "tachyon_error_latency_seconds",
                "Error handling latency",
                &["error_type", "stage"]
            ).unwrap(),
            
            active_errors: register_gauge_vec!(
                "tachyon_active_errors",
                "Number of active errors",
                &["error_type"]
            ).unwrap(),
            
            error_rate: register_gauge_vec!(
                "tachyon_error_rate",
                "Error rate per second",
                &["error_type"]
            ).unwrap(),
        }
    }
    
    /// Records an error.
    pub fn record_error(&self, error: &dyn TachyonError) {
        self.errors_total
            .with_label_values(&[
                error.error_code(),
                &format!("{:?}", error.security_impact()),
            ])
            .inc();
    }
    
    /// Records error latency.
    pub fn record_latency(&self, error_type: &str, stage: &str, duration: Duration) {
        self.error_latency
            .with_label_values(&[error_type, stage])
            .observe(duration.as_secs_f64());
    }
    
    /// Increments active error count.
    pub fn increment_active(&self, error_type: &str) {
        self.active_errors
            .with_label_values(&[error_type])
            .inc();
    }
    
    /// Decrements active error count.
    pub fn decrement_active(&self, error_type: &str) {
        self.active_errors
            .with_label_values(&[error_type])
            .dec();
    }
}
```

**10.4.2. Performance Monitoring**

The following performance metrics should be monitored continuously:

| Metric | Alert Threshold | Alert Severity |
|--------|-----------------|----------------|
| Error rate > 100/sec | 100/sec | Critical |
| Error latency P99 > 100ms | 100ms | High |
| Active errors > 1000 | 1000 | High |
| Error logging queue depth > 1000 | 1000 | Medium |
| Memory usage for errors > 100MB | 100MB | Medium |

### 10.5. Error Handling Performance Testing

**10.5.1. Load Testing Strategy**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Benchmarks error handling performance.
pub fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    // Benchmark error creation
    group.bench_function("error_creation", |b| {
        b.iter(|| {
            let error = ServerError::Validation("Invalid input".to_string());
            black_box(error)
        });
    });
    
    // Benchmark error serialization
    group.bench_function("error_serialization", |b| {
        let error = ServerError::Validation("Invalid input".to_string());
        b.iter(|| {
            let serialized = serde_json::to_string(&error).unwrap();
            black_box(serialized)
        });
    });
    
    // Benchmark error response generation
    group.bench_function("error_response_generation", |b| {
        let error = ServerError::Validation("Invalid input".to_string());
        b.iter(|| {
            let response = error.into_response();
            black_box(response)
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_error_handling);
criterion_main!(benches);
```

**10.5.2. Performance Targets**

| Operation | Target | Acceptable | Critical |
|-----------|--------|------------|----------|
| Error creation | < 1μs | < 10μs | > 100μs |
| Error serialization | < 10μs | < 100μs | > 1ms |
| Error response generation | < 1ms | < 5ms | > 10ms |
| Error logging (async) | < 100μs | < 1ms | > 10ms |
| Error propagation | < 10μs | < 100μs | > 1ms |

### 10.6. Performance Optimization Checklist

**10.6.1. Memory Optimization**

- [ ] Use static strings for error codes
- [ ] Implement zero-copy error propagation
- [ ] Pool frequently used error instances
- [ ] Use lazy evaluation for error context
- [ ] Implement error response caching
- [ ] Limit error context size to < 1KB

**10.6.2. CPU Optimization**

- [ ] Minimize error processing in hot paths
- [ ] Use async error logging
- [ ] Implement error rate limiting
- [ ] Cache error responses
- [ ] Use efficient serialization (e.g., serde_json)
- [ ] Profile error handling code regularly

**10.6.3. I/O Optimization**

- [ ] Batch error log writes
- [ ] Use async I/O for error logging
- [ ] Implement log rotation
- [ ] Compress archived error logs
- [ ] Use efficient log formats (e.g., JSON)
- [ ] Implement log shipping to external systems

### 10.7. Error Performance Monitoring Dashboard

**10.7.1. Key Performance Indicators (KPIs)**

The following KPIs should be displayed on the error performance monitoring dashboard:

| KPI | Description | Target | Alert |
|-----|-------------|--------|-------|
| **Error Rate** | Errors per second | < 10/sec | > 100/sec |
| **Error Latency P50** | Median error handling time | < 1ms | > 5ms |
| **Error Latency P99** | 99th percentile error handling time | < 10ms | > 100ms |
| **Error Memory Usage** | Memory used for error handling | < 10MB | > 100MB |
| **Error Log Queue Depth** | Pending error log entries | < 100 | > 1000 |
| **Error Cache Hit Rate** | Percentage of errors served from cache | > 80% | < 50% |

**10.7.2. Dashboard Layout**

```
┌─────────────────────────────────────────────────────────────┐
│                    ERROR PERFORMANCE DASHBOARD               │
├─────────────────────────────────────────────────────────────┤
│  Error Rate (errors/sec)          Error Latency (ms)         │
│  ████████████████░░░░░░  12.5      ████░░░░░░░░░░░░░  2.3   │
│  Target: < 10/sec                 Target: < 5ms              │
├─────────────────────────────────────────────────────────────┤
│  Error Memory Usage (MB)          Error Cache Hit Rate (%)  │
│  ██████░░░░░░░░░░░░░░░░░  6.2     ████████████████░  87.5  │
│  Target: < 10MB                   Target: > 80%             │
├─────────────────────────────────────────────────────────────┤
│  Top Error Types (Last Hour)                                  │
│  ┌────────────────────┬───────┬─────────┬────────┐          │
│  │ Error Type         │ Count │ Latency  │ Impact │          │
│  ├────────────────────┼───────┼─────────┼────────┤          │
│  │ SRV_VAL_001        │  125  │  1.2ms  │  Low   │          │
│  │ SRV_NOTF_001       │   87  │  0.8ms  │  Low   │          │
│  │ SRV_RATE_001       │   45  │  0.5ms  │  Med   │          │
│  └────────────────────┴───────┴─────────┴────────┘          │
└─────────────────────────────────────────────────────────────┘
```

---

## 11. REFERENCES

### 11.1. Standards and Specifications

| Reference | Title | Version | URL |
|-----------|-------|---------|-----|
| ISO/IEC 26514 | Systems and software engineering — Design and development of information for users | 2021 | https://www.iso.org/standard/78478.html |
| IEEE 1063 | Standard for Software User Documentation | 2001 | https://standards.ieee.org/standard/1063-2001.html |
| RFC 7231 | Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content | 2014 | https://tools.ietf.org/html/rfc7231 |
| RFC 6455 | The WebSocket Protocol | 2011 | https://tools.ietf.org/html/rfc6455 |
| RFC 7807 | Problem Details for HTTP APIs | 2016 | https://tools.ietf.org/html/rfc7807 |
| OWASP ASVS | Application Security Verification Standard | 4.0 | https://owasp.org/www-project-application-security-verification-standard/ |

### 11.2. Tachyon Project Documents

| Reference | Title | Version | Location |
|-----------|-------|---------|----------|
| TACHYON-STD-V1.0 | Coding and Documentation Standards | 1.0 | [`.specs/01_standards/coding_standards.md`](../.specs/01_standards/coding_standards.md) |
| TACHYON-ADR-001-V1.0 | Rust as Primary Language | 1.0 | [`.specs/02_adrs/001_rust_as_primary_language.md`](../.specs/02_adrs/001_rust_as_primary_language.md) |
| TACHYON-ADR-003-V1.0 | Axum for HTTP/2 Server | 1.0 | [`.specs/02_adrs/003_axum_for_http2_server.md`](../.specs/02_adrs/003_axum_for_http2_server.md) |
| TACHYON-ADR-007-V1.0 | Tokio for Async Runtime | 1.0 | [`.specs/02_adrs/007_tokio_for_async_runtime.md`](../.specs/02_adrs/007_tokio_for_async_runtime.md) |
| TACHYON-TMA-V1.0 | Threat Model Analysis | 1.0 | [`.specs/03_threat_model/analysis.md`](../.specs/03_threat_model/analysis.md) |
| TACHYON-REQ-020-V1.0 | Error Handling and Status Codes | 1.0 | [`.specs/04_future_state/reqs/REQ-020_error_handling.md`](../.specs/04_future_state/reqs/REQ-020_error_handling.md) |
| TACHYON-DES-015-V1.0 | Error Handling Architecture | 1.0 | [`.specs/04_future_state/design/DES-015_error_handling.md`](../.specs/04_future_state/design/DES-015_error_handling.md) |

### 11.3. External Documentation

| Reference | Title | URL |
|-----------|-------|-----|
| Rust Error Handling | Error Handling in Rust | https://doc.rust-lang.org/book/ch09-00-error-handling.html |
| The `thiserror` Crate | Derive macros for `std::error::Error` | https://docs.rs/thiserror/latest/thiserror/ |
| The `anyhow` Crate | Flexible error handling | https://docs.rs/anyhow/latest/anyhow/ |
| Axum Error Handling | Error Handling in Axum | https://docs.rs/axum/latest/axum/error_handling/ |
| Tokio Error Handling | Async Error Handling | https://tokio.rs/tokio/tutorial/channels |
| Serde JSON Serialization | JSON Serialization | https://serde.rs/json.html |
| OWASP Error Handling | Error Handling Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/Error_Handling_Cheat_Sheet.html |

### 11.4. Related Tachyon API Specifications

| Reference | Title | Document ID |
|-----------|-------|-------------|
| HTTP/2 API Specification | HTTP/2 API Endpoints and Contracts | TACHYON-API-001-V1.0 |
| WebSocket API Specification | WebSocket Protocol and Message Format | TACHYON-API-002-V1.0 |
| IPC API Specification | Desktop-Server Inter-Process Communication | TACHYON-API-003-V1.0 |
| Authentication API Specification | Authentication and Authorization | TACHYON-API-004-V1.0 |
| Database API Specification | Database Operations and Queries | TACHYON-API-005-V1.0 |
| Git API Specification | Git Repository Operations | TACHYON-API-006-V1.0 |
| Search API Specification | Tantivy Search Engine | TACHYON-API-007-V1.0 |

### 11.5. Glossary

| Term | Definition |
|------|------------|
| **Error** | An unexpected condition that prevents normal operation of the system. |
| **Exception** | An object representing an error condition that can be caught and handled. |
| **Error Code** | A unique identifier for a specific type of error. |
| **Error Context** | Additional information about the error, including request IDs, timestamps, and resource identifiers. |
| **Transient Error** | A temporary error condition that will resolve without intervention. |
| **Permanent Error** | An error condition that will not resolve without intervention. |
| **Recoverable Error** | An error that can be handled automatically without user intervention. |
| **Non-Recoverable Error** | An error that requires user intervention to resolve. |
| **Error Propagation** | The process of passing error information from the point of detection to the appropriate handler. |
| **Error Boundary** | A defined interface that translates internal errors to external representations. |
| **Error Sanitization** | The process of removing sensitive information from error messages before exposing them to clients. |
| **Error Rate Limiting** | The practice of limiting the rate of error responses to prevent abuse. |
| **Circuit Breaker** | A pattern that prevents cascading failures by stopping requests to a failing service. |
| **Retry Pattern** | A pattern that automatically retries failed operations with exponential backoff. |
| **Fallback Mechanism** | An alternative operation or value used when the primary operation fails. |
| **Error Logging** | The process of recording error events for analysis and troubleshooting. |
| **Audit Logging** | The process of recording security-relevant events for compliance and forensic analysis. |
| **Error Metrics** | Quantitative measurements of error behavior and performance. |
| **Error Monitoring** | The continuous observation of error metrics to detect anomalies and trends. |
| **Error Alerting** | The process of notifying operators when error conditions exceed defined thresholds. |

### 11.6. Acronyms and Abbreviations

| Acronym | Full Term |
|----------|-----------|
| ADR | Architecture Decision Record |
| API | Application Programming Interface |
| HTTP | Hypertext Transfer Protocol |
| IPC | Inter-Process Communication |
| JSON | JavaScript Object Notation |
| LRU | Least Recently Used |
| P50 | 50th percentile |
| P99 | 99th percentile |
| RAII | Resource Acquisition Is Initialization |
| SQL | Structured Query Language |
| TTL | Time To Live |
| UUID | Universally Unique Identifier |
| KPI | Key Performance Indicator |
| CSP | Content Security Policy |
| CORS | Cross-Origin Resource Sharing |
| JWT | JSON Web Token |
| OWASP | Open Web Application Security Project |

### 11.7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-06 | Tachyon Team | Initial version |

### 11.8. Approval Record

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Technical Lead | [Pending] | [Pending] | [Pending] |
| Security Officer | [Pending] | [Pending] | [Pending] |
| Documentation Reviewer | [Pending] | [Pending] | [Pending] |

---

**Document End**

**TACHYON-API-019-V1.0**
**Error Handling API Specification**
**February 2026**
**Status: Approved for Implementation**
