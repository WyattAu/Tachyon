# API Error Code Reference

All Tachyon API endpoints return errors in a consistent format:

```json
{
  "code": "ERROR_CODE",
  "message": "Human-readable description of the error"
}
```

Some errors include an additional `details` field with structured validation information:

```json
{
  "code": "VALIDATION_ERROR",
  "message": "Invalid request body",
  "details": {
    "email": "must be a valid email address",
    "password": "must be at least 8 characters"
  }
}
```

---

## Error Codes

### `NOT_FOUND` — 404 Not Found

The requested resource does not exist or the caller is not authorized to see it.

| When | Example |
|---|---|
| Resource ID does not exist in the database | `GET /api/v1/documents/nonexistent-id` |
| Deleted resource | Document was soft-deleted |
| Private resource accessed by non-owner | Guest accessing a private document |

---

### `VALIDATION_ERROR` — 400 Bad Request

The request body, query parameters, or path parameters failed validation.

| When | Example |
|---|---|
| Missing required field | `POST /signup` without `email` |
| Invalid format | Email without `@`, username with special characters |
| Out-of-range value | Password < 8 characters, username > 50 characters |
| Malformed UUID | `"not-a-uuid"` passed as a document ID |
| Invalid JSON | Malformed request body |

---

### `AUTH_ERROR` — 401 Unauthorized

The request lacks valid authentication credentials.

| When | Example |
|---|---|
| Missing or invalid JWT token | No `Authorization: Bearer` header |
| Expired token | JWT `exp` claim has passed |
| Invalid API key | `X-API-Key` header with unknown or revoked key |
| Invalid magic link token | Expired or already-used magic link |
| Invalid password | Wrong password on login |

---

### `FORBIDDEN` — 403 Forbidden

The authenticated user lacks the required permissions for this action.

| When | Example |
|---|---|
| Insufficient role | Reader trying to delete a document |
| Non-owner attempting owner-only action | User modifying another user's profile |
| Admin-only endpoint accessed by non-admin | `GET /compliance/soc2/report` |
| Cross-tenant access | User accessing another organization's resources |

---

### `CONFLICT` — 409 Conflict

The request conflicts with the current state of the resource.

| When | Example |
|---|---|
| Duplicate email | `POST /signup` with existing email |
| Duplicate username | Creating user with existing username |
| Duplicate resource name | Creating a space with an existing name in the same scope |
| Version conflict | Concurrent edit conflict detected |

---

### `RATE_LIMITED` — 429 Too Many Requests

The client has exceeded the rate limit for this endpoint.

| When | Example |
|---|---|
| Exceeded per-IP limit | > 60 requests/minute on auth endpoints |
| Exceeded per-user limit | > 200 requests/minute on document endpoints |
| Exceeded endpoint-specific limit | > 3 requests/minute on magic link request |

Response includes `Retry-After` header with seconds until the limit resets.

---

### `DATABASE_ERROR` — 500 Internal Server Error

An unexpected database error occurred. This indicates a server-side bug or infrastructure issue.

| When | Example |
|---|---|
| Connection pool exhausted | All connections busy (should not happen with proper pool sizing) |
| Query execution failure | Schema mismatch, constraint violation |
| Transaction failure | Rollback due to conflict or timeout |

---

### `SEARCH_ERROR` — 500 Internal Server Error

A search subsystem error occurred.

| When | Example |
|---|---|
| Tantivy index unavailable | Search crate initialization failed |
| Full-text search query failure | Malformed tsquery |

---

### `INTERNAL_ERROR` — 500 Internal Server Error

An unexpected server error occurred. This is a catch-all for errors not covered by other codes.

| When | Example |
|---|---|
| JSON serialization failure | Response body could not be serialized |
| File system error | File upload/write failure |
| External service failure | Email delivery, webhook dispatch |
| Renderer error | Markdown rendering failure |

---

## Additional Error Codes (Route-Specific)

These codes appear in route handlers that define their own error types outside `ServerError`:

| Code | HTTP Status | Where | Description |
|---|---|---|---|
| `USER_NOT_FOUND` | 404 | `/auth/login`, `/auth/mfa/*` | User does not exist |
| `MFA_REQUIRED` | 200 (partial) | `/auth/login` | Login succeeded but MFA is required; response includes `mfa_required: true` |
| `PASSWORD_ERROR` | 500 | `/signup` | Password hashing failure |
| `AI_UNAVAILABLE` | 503 | `/ai/*` | AI service not configured or unreachable |
| `AI_ERROR` | 500 | `/ai/*` | AI processing failure |
| `INVALID_ID` | 400 | Various | ID parameter is not a valid UUID |

---

## Error Response Headers

| Header | When | Description |
|---|---|---|
| `Retry-After` | 429 responses | Seconds until the rate limit window resets |
| `X-Request-Id` | All responses | Unique request identifier for debugging |
| `WWW-Authenticate` | 401 responses | Authentication scheme hint (Bearer) |

---

## Notes for API Consumers

1. **Always check the `code` field**, not just the HTTP status, for programmatic error handling
2. **Rate limit responses** include `Retry-After` — implement exponential backoff
3. **Validation errors** may include a `details` object with per-field messages
4. **404 responses for private resources** use the same code as truly missing resources (deliberate information-hiding)
5. **500 errors** should be reported; they indicate server-side bugs
