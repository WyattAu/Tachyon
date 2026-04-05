# Authentication API Reference

Complete API reference for authentication endpoints.

## Base URL

```
https://api.example.com/api/v1
```

## Endpoints

### Login

Authenticate a user and receive a JWT token.

**Request**

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your-password"
}
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "00000000-0000-0000-0000-000000000001",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "user",
    "created_at": "2026-01-01T00:00:00Z"
  },
  "expires_at": "2026-03-10T12:00:00Z"
}
```

**Error Response**

```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid email or password"
  }
}
```

---

### Logout

Invalidate the current session.

**Request**

```http
DELETE /auth/logout
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 204 No Content
```

---

### Refresh Token

Refresh an existing JWT token.

**Request**

```http
POST /auth/refresh
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2026-03-10T12:00:00Z"
}
```

---

### Guest Login

Authenticate as a guest user (if enabled).

**Request**

```http
POST /auth/guest
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "00000000-0000-0000-0000-000000000099",
    "email": "guest@tachyon.local",
    "name": "Guest User",
    "role": "guest"
  },
  "expires_at": "2026-03-09T13:00:00Z"
}
```

**Error Response**

```http
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "error": {
    "code": "GUEST_LOGIN_DISABLED",
    "message": "Guest login is not enabled"
  }
}
```

---

### Register

Register a new user account.

**Request**

```http
POST /auth/register
Content-Type: application/json

{
  "email": "newuser@example.com",
  "password": "secure-password",
  "name": "Jane Doe"
}
```

**Response**

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "user": {
    "id": "00000000-0000-0000-0000-000000000002",
    "email": "newuser@example.com",
    "name": "Jane Doe",
    "role": "user",
    "created_at": "2026-03-09T12:00:00Z"
  },
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Response**

```http
HTTP/1.1 409 Conflict
Content-Type: application/json

{
  "error": {
    "code": "EMAIL_EXISTS",
    "message": "An account with this email already exists"
  }
}
```

---

### Get Current User

Get the currently authenticated user's profile.

**Request**

```http
GET /users/me
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000001",
  "email": "user@example.com",
  "name": "John Doe",
  "role": "user",
  "created_at": "2026-01-01T00:00:00Z",
  "last_login_at": "2026-03-09T11:00:00Z"
}
```

---

### Update Profile

Update the current user's profile.

**Request**

```http
PATCH /users/me
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "name": "John Smith",
  "email": "john.smith@example.com"
}
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000001",
  "email": "john.smith@example.com",
  "name": "John Smith",
  "role": "user",
  "updated_at": "2026-03-09T12:00:00Z"
}
```

---

### Change Password

Change the current user's password.

**Request**

```http
POST /users/me/password
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "current_password": "old-password",
  "new_password": "new-secure-password"
}
```

**Response**

```http
HTTP/1.1 204 No Content
```

**Error Response**

```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "error": {
    "code": "INVALID_PASSWORD",
    "message": "Current password is incorrect"
  }
}
```

---

## API Key Endpoints

### List API Keys

List all API keys for the current user.

**Request**

```http
GET /users/me/api-keys
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "api_keys": [
    {
      "id": "00000000-0000-0000-0000-000000000010",
      "name": "CI/CD Pipeline",
      "description": "Automated deployment key",
      "key_prefix": "tchk_1a2b3c",
      "scopes": ["documents:read", "documents:write"],
      "created_at": "2026-03-01T00:00:00Z",
      "expires_at": "2026-12-31T23:59:59Z",
      "last_used_at": "2026-03-09T10:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Create API Key

Create a new API key.

**Request**

```http
POST /users/me/api-keys
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "name": "CI/CD Pipeline",
  "description": "Automated deployment key",
  "scopes": ["documents:read", "documents:write"],
  "expires_at": "2026-12-31T23:59:59Z"
}
```

**Response**

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "id": "00000000-0000-0000-0000-000000000011",
  "name": "CI/CD Pipeline",
  "description": "Automated deployment key",
  "key": "tchk_1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyz",
  "scopes": ["documents:read", "documents:write"],
  "created_at": "2026-03-09T12:00:00Z",
  "expires_at": "2026-12-31T23:59:59Z"
}
```

**Note**: The full API key is only returned once. Store it securely.

---

### Revoke API Key

Revoke an API key.

**Request**

```http
DELETE /users/me/api-keys/{key_id}
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 204 No Content
```

---

## Session Endpoints

### List Sessions

List all active sessions for the current user.

**Request**

```http
GET /sessions
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "sessions": [
    {
      "id": "00000000-0000-0000-0000-000000000020",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "created_at": "2026-03-09T10:00:00Z",
      "expires_at": "2026-03-10T10:00:00Z",
      "is_current": true
    }
  ],
  "total": 1
}
```

---

### Revoke Session

Revoke a specific session.

**Request**

```http
DELETE /sessions/{session_id}
Authorization: Bearer YOUR_TOKEN
```

**Response**

```http
HTTP/1.1 204 No Content
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_CREDENTIALS` | 401 | Invalid email or password |
| `TOKEN_EXPIRED` | 401 | JWT token has expired |
| `TOKEN_INVALID` | 401 | JWT token is invalid |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `EMAIL_EXISTS` | 409 | Email already registered |
| `INVALID_PASSWORD` | 400 | Password validation failed |
| `GUEST_LOGIN_DISABLED` | 403 | Guest login not enabled |

---

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/auth/login` | 5 requests | 60 seconds |
| `/auth/guest` | 3 requests | 60 seconds |
| `/auth/register` | 3 requests | 3600 seconds |

---

## Next Steps

- [User Guide: Authentication](../user-guide/authentication.md)
- [User Guide: API Keys](../user-guide/api-keys.md)
- [Developer: API Guide](../developer/api.md)
