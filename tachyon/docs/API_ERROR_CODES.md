# API Error Codes Reference

## Overview

All Tachyon API errors are returned as JSON with a consistent shape. Errors are produced by two server-side types:

- **`ServerError`** — the legacy unified error enum, converted to JSON via `IntoResponse`.
- **`ApiError`** — the preferred route-handler error type with builder methods (`ApiError::bad_request()`, `ApiError::not_found()`, etc.).

Both produce identical JSON output. Sub-crate error types (`DatabaseError`, `SearchError`, `RbacError`, `AuthError`) are transparently converted into these server types through `From` implementations.

## Error Response Format

### Standard API Error

Every API error (from `ServerError` or `ApiError`) returns:

```json
{
  "code": "NOT_FOUND",
  "message": "User not found"
}
```

| Field     | Type   | Description                        |
|-----------|--------|------------------------------------|
| `code`    | string | Machine-readable error code        |
| `message` | string | Human-readable error description   |

### Rate Limit Error

Rate limiting uses a separate response shape (see [Rate Limit Headers](#rate-limit-headers)):

```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Try again in 42 seconds.",
  "retry_after": 42
}
```

| Field        | Type   | Description                            |
|--------------|--------|----------------------------------------|
| `error`      | string | Always `"RATE_LIMIT_EXCEEDED"`         |
| `message`    | string | Human-readable message with retry time |
| `retry_after` | number | Seconds until the rate limit resets    |

### Auth Middleware Error

The auth middleware returns a different shape when rejecting requests:

```json
{
  "error": "Missing authorization header"
}
```

| Field   | Type   | Description                     |
|---------|--------|---------------------------------|
| `error` | string | Error message (not a code)      |

---

## Error Code Reference Table

### Authentication Errors

| Code              | HTTP Status | Description                                      | Routes                                                        |
|-------------------|-------------|--------------------------------------------------|---------------------------------------------------------------|
| `UNAUTHORIZED`    | 401         | Missing or invalid authentication                 | Any protected route, `/auth/password-reset/status`            |
| `AUTH_ERROR`      | 401         | Authentication failure (legacy `ServerError`)    | Any route using `ServerError::Auth`                           |
| `INVALID_TOKEN`   | 401         | JWT token is invalid, expired, or malformed      | `/auth/password-reset/status`, `/auth/email-verification/status` |
| `INVALID_CODE`    | 400         | TOTP/MFA code is invalid or wrong format         | `/auth/mfa/verify`, `/auth/mfa/disable`, `/auth/mfa/authenticate` |
| `MFA_NOT_SETUP`   | 400         | TOTP is not enabled or secret not found          | `/auth/mfa/verify`, `/auth/mfa/disable`, `/auth/mfa/authenticate` |
| `INVALID_USER`    | 400         | User ID in request is invalid                    | `/auth/mfa/setup`, `/auth/mfa/verify`, `/auth/mfa/disable`   |
| `USER_NOT_FOUND`  | 400         | User does not exist                              | `/auth/mfa/setup`, `/auth/login`                              |

**Example — `UNAUTHORIZED`:**

```json
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{
  "code": "UNAUTHORIZED",
  "message": "Authentication required"
}
```

**Example — `INVALID_TOKEN`:**

```json
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "code": "INVALID_TOKEN",
  "message": "Token is invalid, expired, or already used."
}
```

---

### Authorization (RBAC) Errors

| Code                       | HTTP Status | Description                                  | Routes                                           |
|----------------------------|-------------|----------------------------------------------|--------------------------------------------------|
| `FORBIDDEN`                | 403         | Insufficient permissions to access resource  | Any RBAC-protected route, `/roles/{id}/assign`  |
| `PERMISSION_DENIED`        | 403         | RBAC policy denied the action                | RBAC evaluation (internal, surfaced as `FORBIDDEN`) |
| `INVALID_SUBJECT`          | 400         | Invalid RBAC subject (validation)            | RBAC policy operations                           |
| `INVALID_RESOURCE`         | 400         | Invalid RBAC resource (validation)           | RBAC policy operations                           |
| `INVALID_POLICY`           | 400         | Invalid RBAC policy definition               | RBAC policy operations                           |
| `POLICY_EVALUATION_FAILED` | 422         | RBAC policy evaluation encountered an error  | RBAC enforcement                                 |
| `SESSION_ERROR`            | 422         | RBAC session-related error                   | RBAC operations                                  |
| `CACHE_ERROR`              | 422         | RBAC cache error (recoverable)               | RBAC enforcement                                 |

**Example — `FORBIDDEN`:**

```json
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "code": "FORBIDDEN",
  "message": "Insufficient permissions"
}
```

---

### Validation Errors

| Code                | HTTP Status | Description                                    | Routes                                              |
|---------------------|-------------|------------------------------------------------|-----------------------------------------------------|
| `VALIDATION_ERROR`  | 400         | General input validation failure                | `/auth/register`, `/auth/login`, `/documents`, `/teams`, `/spaces`, `/plugins`, `/roles` |
| `INVALID_ID`        | 400         | Malformed UUID or entity identifier             | `/users/{id}`, `/documents/{id}`, `/webhooks/{id}`, `/conflicts`, `/document/search` |
| `INVALID_SESSION_ID`| 400         | Session ID is not a valid UUID                 | `/sessions/create`, `/sessions/validate`, `/sessions/revoke` |
| `INVALID_USER_ID`   | 400         | User ID is not a valid UUID                    | `/sessions/create`                                  |
| `INVALID_PLAN`      | 400         | Invalid billing plan selected                   | `/billing/subscriptions`                            |
| `INVALID_STATUS`    | 400         | Invalid status transition for a review         | `/reviews/{id}`                                     |
| `INVALID_PAYLOAD`   | 400         | Malformed webhook payload                      | `/billing/webhook`                                  |
| `INVALID_SIGNATURE` | 400         | Webhook signature verification failed          | `/billing/webhook`                                  |
| `WEAK_PASSWORD`     | 400         | Password does not meet complexity requirements  | `/auth/password-reset/confirm`                      |

**Example — `VALIDATION_ERROR`:**

```json
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "code": "VALIDATION_ERROR",
  "message": "email: invalid format"
}
```

**Example — `WEAK_PASSWORD`:**

```json
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "code": "WEAK_PASSWORD",
  "message": "Password must be at least 8 characters with uppercase, lowercase, and digit"
}
```

---

### Resource Not Found Errors

| Code          | HTTP Status | Description                             | Routes                                              |
|---------------|-------------|-----------------------------------------|-----------------------------------------------------|
| `NOT_FOUND`   | 404         | Requested resource does not exist       | `/users/{id}`, `/documents/{id}`, `/teams/{id}`, `/roles/{id}`, `/webhooks/{id}`, `/repositories/{id}`, `/plugins/{id}`, `/billing/subscriptions/{org_id}`, `/billing/invoices/{org_id}`, `/sessions/{id}`, `/search/indexes/{name}`, `/organizations/{id}`, `/spaces/{id}` |
| `NO_INDEX`    | 404         | Search index does not exist             | `/search/query`, `/search/indexes/{name}/documents` |
| `NO_DOCUMENTS`| 400         | No documents available for SSG build    | `/ssg/generate`                                     |

**Example — `NOT_FOUND`:**

```json
HTTP/1.1 404 Not Found
Content-Type: application/json

{
  "code": "NOT_FOUND",
  "message": "User not found"
}
```

---

### Conflict Errors

| Code        | HTTP Status | Description                              | Routes                                              |
|-------------|-------------|------------------------------------------|-----------------------------------------------------|
| `CONFLICT`  | 409         | Resource already exists (duplicate)      | `/auth/register`, `/auth/login`, `/users`           |

**Example — `CONFLICT`:**

```json
HTTP/1.1 409 Conflict
Content-Type: application/json

{
  "code": "CONFLICT",
  "message": "email already exists"
}
```

---

### Database Errors

| Code                  | HTTP Status | Description                                  | Routes                                           |
|-----------------------|-------------|----------------------------------------------|--------------------------------------------------|
| `DATABASE_ERROR`      | 500         | General database operation failure           | Any route (transparent from `DatabaseError`)    |
| `CONSTRAINT_VIOLATION`| 400         | Database constraint violation                | Any route with foreign key / unique constraints |
| `SESSION_EXPIRED`     | 410         | Database session has expired                 | Session operations (surfaces as `GONE`)         |
| `SESSION_NOT_FOUND`   | 404         | Database session not found                   | Session operations (surfaces as `NOT_FOUND`)    |
| `RBAC_POLICY_ERROR`   | 500         | RBAC policy database error                   | RBAC operations (surfaces as `FORBIDDEN`)       |

**Example — `DATABASE_ERROR`:**

```json
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "code": "DATABASE_ERROR",
  "message": "Database error: connection refused"
}
```

---

### Search Errors

All search errors are surfaced as `500 Internal Server Error` with code `SEARCH_ERROR` at the API layer.

| Internal Code            | Search Category | Description                              |
|--------------------------|-----------------|------------------------------------------|
| `DOCUMENT_NOT_FOUND`     | Index           | Document not found in search index       |
| `INDEX_NOT_FOUND`        | Index           | Search index does not exist              |
| `INVALID_QUERY`          | Query           | Query syntax or semantics are invalid    |
| `PARSE_ERROR`            | Query           | Query string could not be parsed         |
| `FIELD_VALIDATION_ERROR` | Query           | Query field failed validation            |
| `CONFIG_ERROR`           | Index           | Search configuration is invalid          |
| `IO_ERROR`               | IO              | File I/O error during search operation   |
| `SERIALIZATION_ERROR`    | Index           | Serialization/deserialization failure    |
| `TANTIVY_ERROR`          | Index           | Underlying Tantivy search engine error   |
| `INTERNAL_ERROR`         | Internal        | Unexpected internal search state         |

**Example — `SEARCH_ERROR`:**

```json
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "code": "SEARCH_ERROR",
  "message": "Search error: [INDEX:INDEX_NOT_FOUND] Index my-index not found"
}
```

---

### Session Errors

| Code               | HTTP Status | Description                               | Routes                                |
|--------------------|-------------|-------------------------------------------|---------------------------------------|
| `GONE`             | 410         | Session has expired or is inactive        | `/sessions/validate`                  |
| `INVALID_SESSION_ID` | 400       | Session ID is not a valid UUID            | `/sessions/validate`, `/sessions/revoke` |

**Example — `GONE`:**

```json
HTTP/1.1 410 Gone
Content-Type: application/json

{
  "code": "GONE",
  "message": "Session has expired or is inactive"
}
```

---

### Server / Internal Errors

| Code             | HTTP Status | Description                                 | Routes                              |
|------------------|-------------|---------------------------------------------|-------------------------------------|
| `INTERNAL_ERROR` | 500         | Unexpected server error                     | Any route                           |
| `CONFIGURATION_ERROR` | 500      | Server or RBAC configuration error         | Startup, RBAC initialization        |

**Example — `INTERNAL_ERROR`:**

```json
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "code": "INTERNAL_ERROR",
  "message": "something broke"
}
```

---

### Rate Limit Error

| Code                 | HTTP Status | Description                            | All rate-limited routes              |
|----------------------|-------------|----------------------------------------|--------------------------------------|
| `RATE_LIMIT_EXCEEDED`| 429         | Too many requests in the time window   | `/api/v1/auth/login`, `/api/v1/auth/guest`, `/api/v1/documents`, and all other routes |

**Example:**

```json
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1714852800

{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Try again in 42 seconds.",
  "retry_after": 42
}
```

---

## Error Categories

### 4xx Client Errors — Show User Message

These indicate a problem with the request. Display the `message` field to the user and do **not** retry automatically.

| Category              | Codes                                                  |
|-----------------------|--------------------------------------------------------|
| Authentication        | `UNAUTHORIZED`, `AUTH_ERROR`, `INVALID_TOKEN`         |
| Authorization         | `FORBIDDEN`                                            |
| Validation            | `VALIDATION_ERROR`, `INVALID_ID`, `INVALID_SESSION_ID`, `INVALID_USER_ID`, `INVALID_PLAN`, `INVALID_STATUS`, `INVALID_PAYLOAD`, `INVALID_SIGNATURE`, `WEAK_PASSWORD`, `INVALID_CODE`, `INVALID_USER`, `USER_NOT_FOUND`, `MFA_NOT_SETUP` |
| Not Found             | `NOT_FOUND`, `NO_INDEX`, `NO_DOCUMENTS`                |
| Conflict              | `CONFLICT`                                             |
| Constraint Violation  | `CONSTRAINT_VIOLATION`                                  |
| Session Gone          | `GONE`                                                 |

### 429 Rate Limiting — Exponential Backoff

| Code                 | Action                                                                                     |
|----------------------|--------------------------------------------------------------------------------------------|
| `RATE_LIMIT_EXCEEDED` | Read the `retry_after` field (seconds) and wait at least that long before retrying. Use exponential backoff starting from `retry_after` if repeated 429s occur. |

### 5xx Server Errors — Generic Message + Retry

These indicate a server-side problem. Show a generic message like "Something went wrong. Please try again later." to the user, log the full error for debugging, and retry with exponential backoff.

| Category   | Codes                                                         |
|------------|---------------------------------------------------------------|
| Database   | `DATABASE_ERROR`                                              |
| Search     | `SEARCH_ERROR`                                                |
| Internal   | `INTERNAL_ERROR`, `CONFIGURATION_ERROR`                       |

### Network Errors — Offline Handling

When the client cannot reach the server at all (DNS failure, connection refused, timeout), the error originates on the client side, not from the API:

1. **Detect offline state** — Show a "You appear to be offline" message.
2. **Queue mutations** — Store failed write requests locally and replay when connectivity is restored.
3. **Serve cached data** — Return the last known good response for read operations.
4. **Retry automatically** — Use exponential backoff (e.g., 1s, 2s, 4s, 8s, 16s, 30s cap) with jitter to avoid thundering herd.

---

## Client Error Handling Guide

### Recommended Retry Strategy

```
4xx errors (except 429): Do NOT retry. Fix the request.
429:                        Retry after `retry_after` seconds, with exponential backoff.
5xx:                        Retry with exponential backoff (1s, 2s, 4s, 8s, 16s, 30s cap, with jitter).
Network errors:             Retry with exponential backoff. Show offline UI.
```

### Pseudocode

```javascript
async function apiRequest(url, options) {
  const response = await fetch(url, options);
  const body = await response.json();

  if (response.status === 429) {
    // Rate limited — wait and retry
    const retryAfter = body.retry_after || 1;
    await sleep(retryAfter * 1000);
    return apiRequest(url, options);
  }

  if (response.status >= 500) {
    // Server error — show generic message, optionally retry
    showError("Something went wrong. Please try again later.");
    logError(body.code, body.message);
    return null;
  }

  if (response.status >= 400) {
    // Client error — show the server message
    showError(body.message);
    return null;
  }

  return body;
}
```

---

## Rate Limit Headers

Every response (not just 429s) includes rate limit headers when rate limiting is enabled:

| Header                | Type   | Description                                      |
|-----------------------|--------|--------------------------------------------------|
| `X-RateLimit-Limit`   | number | Maximum requests allowed in the current window   |
| `X-RateLimit-Remaining`| number | Requests remaining in the current window         |
| `X-RateLimit-Reset`   | number | Unix timestamp when the rate limit window resets |

### Default Limits

| Endpoint                  | Requests | Window (seconds) |
|---------------------------|----------|-------------------|
| `/api/v1/auth/login`      | 5        | 60                |
| `/api/v1/auth/guest`      | 3        | 60                |
| `/api/v1/documents`       | 100      | 60                |
| All other endpoints       | 1000     | 60                |

### Configuration

Rate limiting is configured via `RateLimitConfig`:

- `TACHYON_RATE_LIMIT_ENABLED` — set to `"0"` or `"false"` to disable
- Redis backend available via `rate_limit.redis_url` for distributed deployments
- Falls back to in-memory store if Redis is unavailable

### 429 Response

When the limit is exceeded, the response includes all headers plus the JSON body with `retry_after`:

```
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1714852800

{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Try again in 42 seconds.",
  "retry_after": 42
}
```

---

## Error Code Mapping: Sub-Crate to API

### `DatabaseError` → API mapping (via `From<DatabaseError> for ApiError`)

| DatabaseError Variant   | API Code              | HTTP Status |
|--------------------------|-----------------------|-------------|
| `NotFound`               | `NOT_FOUND`           | 404         |
| `ValidationError`        | `VALIDATION_ERROR`    | 400         |
| `Duplicate`              | `CONFLICT`            | 409         |
| `ConstraintViolation`    | `CONSTRAINT_VIOLATION`| 400         |
| All other variants       | `INTERNAL_ERROR`      | 500         |

### `ServerError` → API mapping (via `From<ServerError> for ApiError`)

| ServerError Variant | API Code            | HTTP Status |
|---------------------|---------------------|-------------|
| `NotFound`          | `NOT_FOUND`         | 404         |
| `Validation`        | `VALIDATION_ERROR`  | 400         |
| `Auth`              | `AUTH_ERROR`        | 401         |
| `Rbac`              | `FORBIDDEN`         | 403         |
| `Database`          | `DATABASE_ERROR`    | 500         |
| `Search`            | `SEARCH_ERROR`      | 500         |
| `Internal`          | `INTERNAL_ERROR`    | 500         |

### `AuthError` → API mapping (via `From<AuthError> for ServerError`)

| AuthError Variant       | ServerError Variant | API Code       | HTTP Status |
|-------------------------|---------------------|----------------|-------------|
| `InsufficientPermissions`| `Rbac`             | `FORBIDDEN`    | 403         |
| All other variants      | `Auth`              | `AUTH_ERROR`   | 401         |

### `RbacError` → API mapping (via `From<RbacError> for ServerError`)

All `RbacError` variants are converted to `ServerError::Rbac`, which maps to HTTP 403 `FORBIDDEN` at the API layer.

### `SearchError` → API mapping (via `From<SearchError> for ServerError`)

All `SearchError` variants are converted to `ServerError::Search`, which maps to HTTP 500 `SEARCH_ERROR` at the API layer. The original search error code and category are embedded in the `message` string.
