# Tachyon API Reference

Base URL: `/api/v1`

Interactive Swagger UI: `/api/docs`

## Authentication

All protected endpoints require a Bearer token in the `Authorization` header or an `X-API-Key` header.

### Login
```
POST /api/v1/auth/login
Content-Type: application/json

Request:
{
  "username": "string",
  "password": "string"
}

Response: 200
{
  "success": true,
  "user_id": "string",
  "access_token": "string",
  "refresh_token": "string",
  "token_type": "Bearer",
  "expires_in": 3600,
  "error": null,
  "user": { ... },
  "mfa_required": false,
  "mfa_user_id": null
}
```

Rate limit: 5 requests per minute per IP. Returns `mfa_required: true` if MFA is enabled.

### Register
```
POST /api/v1/auth/register
Content-Type: application/json

Request:
{
  "username": "string (3-50 chars)",
  "display_name": "string (1-100 chars)",
  "password": "string (8+ chars)",
  "email": "string (optional)"
}

Response: 200
{ "AuthenticateResponse" (same as login) }
```

Rate limit: 3 requests per minute per IP. Creates user with Reader role.

### Token Refresh
```
POST /api/v1/auth/refresh
Content-Type: application/json

Request:
{ "refresh_token": "string" }

Response: 200
{ "AuthenticateResponse" (new access + refresh token pair) }
```

Rate limit: 10 requests per minute per IP. Revokes the old refresh token.

### Guest Login
```
POST /api/v1/auth/guest

Response: 200
{ "AuthenticateResponse" (no refresh_token) }
```

Returns 403 if guest login is disabled.

### Auth Status
```
GET /api/v1/auth/status

Response: 200
{ "authenticated": true, "user": { "id": "...", "role": "..." } }
```

### Logout
```
POST /api/v1/auth/logout
Content-Type: application/json

Request (optional):
{ "refresh_token": "string" }

Response: 200
{ "success": true, "message": "Logged out successfully" }
```

### Current User
```
GET /api/v1/auth/me
PUT /api/v1/auth/me
```

Requires `Authorization: Bearer <token>`.

### Guest Status (public)
```
GET /api/v1/auth/guest-status

Response: 200
{ "guest_login_enabled": bool, "public_notes_enabled": bool }
```

### Password Reset
```
POST /api/v1/auth/password-reset/request
POST /api/v1/auth/password-reset/confirm
```

### Email Verification
```
POST /api/v1/auth/email-verify/request
POST /api/v1/auth/email-verify/confirm
```

### MFA (Multi-Factor Authentication)
```
POST /api/v1/auth/mfa/enable
POST /api/v1/auth/mfa/verify
POST /api/v1/auth/mfa/disable
POST /api/v1/auth/mfa/authenticate
```

### OAuth2
```
GET /api/v1/auth/oauth2/google/authorize
GET /api/v1/auth/oauth2/google/callback
GET /api/v1/auth/oauth2/github/authorize
GET /api/v1/auth/oauth2/github/callback
```

---

## Users

Admin-only user management (can set arbitrary roles).

```
GET    /api/v1/users?page=1&page_size=20&role=reader
POST   /api/v1/users
GET    /api/v1/users/me
GET    /api/v1/users/{user_id}
PUT    /api/v1/users/{user_id}
DELETE /api/v1/users/{user_id}
```

### Create User (admin)
```
POST /api/v1/users
Content-Type: application/json

Request:
{
  "username": "string (3-50 chars)",
  "display_name": "string",
  "password": "string (8+ chars)",
  "email": "string (optional)",
  "role": "admin|editor|writer|reader (optional, default: reader)"
}
```

---

## Documents

```
GET    /api/v1/documents?page=1&page_size=20&search=&project_id=&author_id=
POST   /api/v1/documents
GET    /api/v1/documents/{document_id}
PUT    /api/v1/documents/{document_id}
DELETE /api/v1/documents/{document_id}
GET    /api/v1/documents/{document_id}/metadata
GET    /api/v1/documents/search?q=keyword
GET    /api/v1/documents/{document_id}/backlinks
```

### Create Document
```
POST /api/v1/documents
Content-Type: application/json

Request:
{
  "title": "string",
  "content": "string",
  "project_id": "string (optional)",
  "tags": ["string"],
  "visibility": "public|private|restricted (optional)"
}
```

### Document Versions
```
GET  /api/v1/documents/{document_id}/versions
POST /api/v1/documents/{document_id}/versions
GET  /api/v1/documents/{document_id}/versions/{version_number}
GET  /api/v1/documents/{document_id}/versions/{v1}/diff/{v2}
```

### Document Attachments
```
GET    /api/v1/documents/{document_id}/attachments
POST   /api/v1/documents/{document_id}/attachments (multipart, max 50 MB)
DELETE /api/v1/documents/{document_id}/attachments/{attachment_id}
```

### Document Templates
```
GET    /api/v1/templates?page=1&page_size=20
POST   /api/v1/templates
GET    /api/v1/templates/{template_id}
PUT    /api/v1/templates/{template_id}
DELETE /api/v1/templates/{template_id}
```

### Markdown Rendering
```
POST /api/v1/render/markdown
Content-Type: application/json

Request:
{ "content": "# Hello", "format": "markdown" }
```

---

## Search

Full-text search with faceted filtering, fusing PostgreSQL and Tantivy results.

```
GET  /api/v1/search?q=keyword&page=1&page_size=20&content_type=&status=&visibility=&project_id=&author_id=&tags=&date_from=&date_to=
GET  /api/v1/search/global?q=keyword
GET  /api/v1/search/suggest?q=&limit=10
POST /api/v1/search/reindex
```

### Saved Searches
```
POST   /api/v1/search/saved
GET    /api/v1/search/saved
GET    /api/v1/search/saved/{id}
PUT    /api/v1/search/saved/{id}
DELETE /api/v1/search/saved/{id}
```

---

## Spaces

```
GET    /api/v1/spaces?owner_id=
GET    /api/v1/spaces/root?owner_id=
GET    /api/v1/spaces/default?owner_id=
GET    /api/v1/spaces/{space_id}/children?owner_id=
POST   /api/v1/spaces
GET    /api/v1/spaces/{space_id}
PUT    /api/v1/spaces/{space_id}
DELETE /api/v1/spaces/{space_id}
PUT    /api/v1/spaces/move-document/{document_id}
```

### Space Members
```
GET    /api/v1/spaces/{space_id}/members
POST   /api/v1/spaces/{space_id}/members
PUT    /api/v1/spaces/{space_id}/members/{user_id}
DELETE /api/v1/spaces/{space_id}/members/{user_id}
```

### Create Space
```
POST /api/v1/spaces
Content-Type: application/json

Request:
{
  "name": "string",
  "description": "string (optional)",
  "icon": "string (optional)",
  "color": "string (optional)",
  "parent_id": "string (optional)",
  "visibility": "public|private (optional)",
  "owner_id": "string (optional)"
}
```

---

## Sessions

```
POST   /api/v1/sessions
GET    /api/v1/sessions/{session_id}
GET    /api/v1/sessions/{session_id}/validate
DELETE /api/v1/sessions/{session_id}
GET    /api/v1/users/{user_id}/sessions
DELETE /api/v1/users/{user_id}/sessions
```

---

## Teams

```
GET    /api/v1/teams
POST   /api/v1/teams
GET    /api/v1/teams/slug/{slug}
GET    /api/v1/teams/{team_id}
PUT    /api/v1/teams/{team_id}
DELETE /api/v1/teams/{team_id}
```

### Team Members
```
GET    /api/v1/teams/{team_id}/members
POST   /api/v1/teams/{team_id}/members
PUT    /api/v1/teams/{team_id}/members/{user_id}
DELETE /api/v1/teams/{team_id}/members/{user_id}
```

---

## Roles

```
GET    /api/v1/roles
POST   /api/v1/roles
POST   /api/v1/roles/seed
GET    /api/v1/roles/{role_id}
PUT    /api/v1/roles/{role_id}
DELETE /api/v1/roles/{role_id}
```

---

## Organizations

```
GET    /api/v1/organizations
POST   /api/v1/organizations
GET    /api/v1/organizations/{id}
PUT    /api/v1/organizations/{id}
DELETE /api/v1/organizations/{id}
```

### Organization Members
```
GET    /api/v1/organizations/{org_id}/members
POST   /api/v1/organizations/{org_id}/members
PUT    /api/v1/organizations/{org_id}/members/{user_id}
DELETE /api/v1/organizations/{org_id}/members/{user_id}
```

---

## Projects (Service Catalog)

```
POST   /api/v1/projects
GET    /api/v1/projects
GET    /api/v1/projects/{id}
PUT    /api/v1/projects/{id}
DELETE /api/v1/projects/{id}
GET    /api/v1/projects/slug/{slug}
```

### Project Components
```
POST   /api/v1/components
GET    /api/v1/components/{id}
DELETE /api/v1/components/{id}
GET    /api/v1/projects/{project_id}/components
```

### Project Members
```
POST   /api/v1/projects/{project_id}/members
GET    /api/v1/projects/{project_id}/members
DELETE /api/v1/projects/{project_id}/members/{user_id}
```

### Catalog Stats
```
GET /api/v1/catalog/stats
```

---

## Knowledge Graph

```
POST   /api/v1/nodes
GET    /api/v1/nodes?page=1&page_size=20
GET    /api/v1/nodes/{node_id}
PUT    /api/v1/nodes/{node_id}
DELETE /api/v1/nodes/{node_id}
POST   /api/v1/edges
GET    /api/v1/nodes/{node_id}/edges
DELETE /api/v1/edges/{edge_id}
POST   /api/v1/graph/query
GET    /api/v1/graph/stats
GET    /api/v1/graph/at?at=2026-01-01T00:00:00Z
GET    /api/v1/graph/diff?from=...&to=...
```

### Graph Query
```
POST /api/v1/graph/query
Content-Type: application/json

Request:
{
  "source_id": "string",
  "direction": "incoming|outgoing|both (optional, default: both)",
  "edge_type": "string (optional)",
  "depth": 3 (optional, max 5),
  "target_id": "string (optional, for shortest path)"
}
```

---

## Repositories

```
POST   /api/v1/repositories/init
POST   /api/v1/repositories/clone
POST   /api/v1/repositories/{repository_id}/commit
POST   /api/v1/repositories/{repository_id}/push
GET    /api/v1/repositories/{repository_id}/status
GET    /api/v1/repositories
GET    /api/v1/repositories/{repository_id}
DELETE /api/v1/repositories/{repository_id}
```

---

## Notifications

```
GET  /api/v1/notifications?limit=20&offset=0&include_read=true
GET  /api/v1/notifications/unread-count
POST /api/v1/notifications/read-all
POST /api/v1/notifications/{id}/read
```

---

## Collaboration

### Presence
```
PUT    /api/v1/collaboration/presence
GET    /api/v1/collaboration/presence/{document_id}
DELETE /api/v1/collaboration/presence/{document_id}/{user_id}
```

### Comments
```
GET    /api/v1/collaboration/documents/{document_id}/comments
POST   /api/v1/collaboration/comments
PUT    /api/v1/collaboration/comments/{comment_id}
DELETE /api/v1/collaboration/comments/{comment_id}
```

### Mentions
```
GET /api/v1/collaboration/mentions/{user_id}
```

---

## Webhooks

```
POST   /api/v1/webhooks
GET    /api/v1/webhooks
DELETE /api/v1/webhooks/{id}
```

---

## Plugins

```
GET    /api/v1/plugins
POST   /api/v1/plugins
POST   /api/v1/plugins/invoke
GET    /api/v1/plugins/{plugin_id}
PUT    /api/v1/plugins/{plugin_id}
DELETE /api/v1/plugins/{plugin_id}
```

---

## Onboarding

```
GET  /api/v1/onboarding
POST /api/v1/onboarding/complete
POST /api/v1/onboarding/sample-content
GET  /api/v1/onboarding/suggestions
```

---

## Reviews

```
POST /api/v1/reviews
PUT  /api/v1/reviews/{review_id}
GET  /api/v1/reviews
```

---

## Activity

```
GET  /api/v1/activity
POST /api/v1/activity
```

---

## Tags

```
GET /api/v1/tags
```

Returns top 100 tags with usage counts.

---

## Conflicts

```
GET  /api/v1/conflicts/{document_id}
POST /api/v1/conflicts/{document_id}/resolve
```

---

## Files

```
GET  /api/v1/files/list?path=/&all=false
GET  /api/v1/files/read?path=/README.md
GET  /api/v1/files/search?query=test&path=/
GET  /api/v1/files/tree?path=/&depth=2
GET  /api/v1/files/stats
GET  /api/v1/files/recent?limit=20
POST /api/v1/files/upload (multipart, max 50 MB)
```

---

## SSG (Static Site Generation)

```
GET  /api/v1/ssg/config
POST /api/v1/ssg/build
GET  /api/v1/ssg/download (returns ZIP)
```

---

## Billing (TrueLayer Payments)

```
GET  /api/v1/billing/plans
POST /api/v1/billing/subscriptions
GET  /api/v1/billing/subscriptions/{org_id}
POST /api/v1/billing/subscriptions/{org_id}/cancel
POST /api/v1/billing/subscription/change-plan
GET  /api/v1/billing/invoices/{org_id}
GET  /api/v1/billing/usage/{org_id}
POST /api/v1/billing/mandates
GET  /api/v1/billing/mandates/{mandate_id}
POST /api/v1/billing/payments
GET  /api/v1/billing/payments/{payment_id}
POST /api/v1/billing/webhook
```

---

## Health & Metrics

```
GET /health
GET /ready
GET /metrics/prometheus
```

### Health Check Response
```json
{
  "status": "healthy|degraded|unhealthy",
  "version": "1.0.0",
  "uptime_secs": 3600,
  "checks": {
    "database": { "status": "ok", "latency_ms": 5, "error": null },
    "redis": { "status": "ok|disabled", "latency_ms": null, "error": null },
    "tantivy": { "status": "ok|disabled", "latency_ms": null, "error": null },
    "smtp": { "status": "ok|disabled", "latency_ms": null, "error": null }
  }
}
```

### Prometheus Metrics
```
tachyon_requests_total
tachyon_requests_successful
tachyon_requests_failed
tachyon_request_duration_avg_ms
tachyon_uptime_seconds
tachyon_version_info
```

---

## Error Format

All errors follow this format:
```json
{
  "code": "ERROR_CODE",
  "message": "Human-readable description"
}
```

Some endpoints may include a `details` map for field-level errors:
```json
{
  "code": "VALIDATION_ERROR",
  "message": "Validation failed",
  "details": {
    "field_name": "specific error"
  }
}
```

### Common Error Codes
| Code | HTTP Status | Description |
|------|-------------|-------------|
| VALIDATION_ERROR | 400 | Request validation failed |
| NOT_FOUND | 404 | Resource not found |
| AUTH_ERROR | 401 | Authentication required or failed |
| FORBIDDEN | 403 | Insufficient permissions |
| CONFLICT | 409 | Resource conflict (duplicate) |
| RATE_LIMITED | 429 | Too many requests |
| DATABASE_ERROR | 500 | Database operation failed |
| SEARCH_ERROR | 500 | Search operation failed |
| INTERNAL_ERROR | 500 | Server error |

### Additional Error Codes
| Code | Context |
|------|---------|
| INVALID_ID | Malformed UUID or identifier |
| INVALID_TOKEN | Invalid or expired refresh token |
| USER_NOT_FOUND | User referenced by token does not exist |
| ACCOUNT_DISABLED | Account has been deactivated |
| PASSWORD_ERROR | Password hashing/processing failure |
| TOKEN_ERROR | JWT generation failure |
| GUEST_LOGIN_DISABLED | Guest access not configured |
| PAYMENTS_DISABLED | TrueLayer payments not configured |
| MISSING_SIGNATURE | Webhook missing signature header |
| INVALID_SIGNATURE | Webhook signature verification failed |
