# Tachyon Error Code Reference

## Overview

All API errors are returned as JSON with a `code` and `message` field. Some errors may include an optional `details` map for field-level validation information.

```json
{
  "code": "VALIDATION_ERROR",
  "message": "Username must be between 3 and 50 characters",
  "details": {
    "username": "too_short"
  }
}
```

## Error Variants

Defined in `tachyon/crates/server/src/error.rs` as the `ServerError` enum.

### VALIDATION_ERROR (400)
- **Variant**: `ServerError::Validation`
- **HTTP Status**: 400 Bad Request
- **Description**: The request body, query parameters, or path parameters failed validation.
- **Common causes**:
  - Missing required fields
  - Invalid field formats (e.g., non-UUID ID)
  - Out-of-range values (e.g., `page_size` > 100)
  - Empty required strings
- **Constructors**: `ServerError::bad_request(msg)`
- **Example**: `"Username must be between 3 and 50 characters"`

### NOT_FOUND (404)
- **Variant**: `ServerError::NotFound`
- **HTTP Status**: 404 Not Found
- **Description**: The requested resource does not exist.
- **Constructors**: `ServerError::not_found(resource, id)`
- **Example**: `"Document 'abc123' not found"`

### AUTH_ERROR (401)
- **Variant**: `ServerError::Auth`
- **HTTP Status**: 401 Unauthorized
- **Description**: Authentication is required or the provided credentials are invalid.
- **Constructors**: `ServerError::unauthorized(msg)`
- **Example**: `"Invalid or expired token"`, `"Missing or invalid Authorization header"`

### FORBIDDEN (403)
- **Variant**: `ServerError::Rbac`
- **HTTP Status**: 403 Forbidden
- **Description**: The authenticated user lacks permission for the requested action.
- **Constructors**: `ServerError::forbidden(msg)`
- **Example**: `"Insufficient permissions"`, `"Cannot delete role: protected system role"`

### CONFLICT (409)
- **Variant**: `ServerError::Conflict`
- **HTTP Status**: 409 Conflict
- **Description**: The request conflicts with the current state of the resource.
- **Constructors**: `ServerError::conflict(msg)`
- **Example**: `"Username or email already exists"`, `"Email already in use"`

### RATE_LIMITED (429)
- **Variant**: `ServerError::RateLimit`
- **HTTP Status**: 429 Too Many Requests
- **Description**: The client has exceeded the rate limit.
- **Constructors**: `ServerError::rate_limited(retry_after_seconds)`
- **Example**: `"Too many requests. Retry after 60 seconds."`

### DATABASE_ERROR (500)
- **Variant**: `ServerError::Database`
- **HTTP Status**: 500 Internal Server Error
- **Description**: A database operation failed.
- **Constructors**: `ServerError::database(msg)`
- **Source**: Converted from `tachyon_database::DatabaseError` and `sqlx::Error`
- **Example**: `"Failed to create space: connection refused"`

### SEARCH_ERROR (500)
- **Variant**: `ServerError::Search`
- **HTTP Status**: 500 Internal Server Error
- **Description**: A search operation (Tantivy or PostgreSQL full-text) failed.
- **Source**: Converted from `tachyon_search::SearchError`
- **Example**: `"Tantivy search failed: index corrupted"`

### INTERNAL_ERROR (500)
- **Variant**: `ServerError::Internal`
- **HTTP Status**: 500 Internal Server Error
- **Description**: An unexpected server-side error.
- **Constructors**: `ServerError::internal(msg)`
- **Source**: Converted from `std::io::Error`, `serde_json::Error`, `tachyon_renderer::RendererError`, `tachyon_ssg::SsgError`, `tachyon_import_export::ImportExportError`, `tachyon_plugin_runtime::PluginRuntimeError`
- **Example**: `"Failed to generate JWT"`

## Error With Details

`ServerErrorWithDetails` wraps a `ServerError` with an optional `BTreeMap<String, String>` for structured field-level errors.

```json
{
  "code": "VALIDATION_ERROR",
  "message": "Validation failed",
  "details": {
    "email": "already in use",
    "username": "too short"
  }
}
```

**Constructors**:
- `ServerError::with_details(details)` - attach a map of field-level errors
- `ServerError::with_detail_string(detail)` - attach a single string detail (backward compat)

## Domain-Specific Error Codes

These codes appear in endpoints that use their own error response types (not `ServerError`):

### User/Auth Errors
| Code | Status | Description |
|------|--------|-------------|
| VALIDATION_ERROR | 400 | Input validation failure |
| CONFLICT | 409 | Duplicate username or email |
| NOT_FOUND | 404 | User not found |
| UNAUTHORIZED | 401 | Missing/invalid token |
| INVALID_ID | 400 | Malformed user UUID |
| INTERNAL_ERROR | 500 | Server failure |
| PASSWORD_ERROR | 500 | Password hashing failure |
| TOKEN_ERROR | 500 | JWT generation failure |
| INVALID_TOKEN | 401 | Expired or invalid refresh token |
| USER_NOT_FOUND | 401 | User referenced by token missing |
| ACCOUNT_DISABLED | 401 | User account deactivated |
| GUEST_LOGIN_DISABLED | 403 | Guest access not configured |

### Billing Errors
| Code | Status | Description |
|------|--------|-------------|
| NOT_FOUND | 404 | No subscription found |
| DB_ERROR | 500 | Database operation failed |
| PAYMENTS_DISABLED | 503 | TrueLayer payments not configured |
| MISSING_SIGNATURE | 401 | Webhook missing signature |
| INVALID_SIGNATURE | 401 | Webhook signature mismatch |
| INVALID_PAYLOAD | 400 | Malformed webhook body |

## From Conversions

The following types are automatically converted to `ServerError` via `From` implementations:

| Source Type | Target Variant |
|------------|----------------|
| `tachyon_database::DatabaseError` | `Database` |
| `sqlx::Error` | `Database` |
| `tachyon_search::SearchError` | `Search` |
| `std::io::Error` | `Internal` |
| `serde_json::Error` | `Internal` |
| `tachyon_renderer::RendererError` | `Internal` |
| `tachyon_rbac::RbacError` | `Rbac` |
| `tachyon_ssg::SsgError` | `Internal` |
| `tachyon_import_export::ImportExportError` | `Internal` |
| `tachyon_plugin_runtime::PluginRuntimeError` | `Internal` |
| `crate::middleware::auth::AuthError` | `Auth` or `Rbac` |
