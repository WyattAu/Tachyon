---
title: Authentication
description: JWT authentication, OAuth2, and guest access
order: 8
tags: [auth, reference]
---

# Authentication

Tachyon uses JWT (JSON Web Tokens) for stateless authentication with optional OAuth2 and guest access.

## JWT Authentication

### Registration

```
POST /api/v1/auth/register
Content-Type: application/json

{
  "username": "alice",
  "email": "alice@example.com",
  "password": "secure-password"
}
```

Response:

```json
{
  "token": "eyJ...",
  "user": {
    "id": "uuid",
    "username": "alice",
    "email": "alice@example.com",
    "role": "admin"
  }
}
```

The first registered user receives the `admin` role automatically.

### Login

```
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "alice",
  "password": "secure-password"
}
```

### Using the Token

Include the JWT in the `Authorization` header for all authenticated endpoints:

```
Authorization: Bearer eyJ...
```

Token expiration is configurable via `TACHYON_JWT_EXPIRY_HOURS` (default: 24 hours).

## Guest Access

When `TACHYON_GUEST_LOGIN_ENABLED=true`, unauthenticated users receive a limited-access token:

```
POST /api/v1/auth/guest
```

Guest users have read-only access to public documents.

## OAuth2

Tachyon supports OAuth2 for Google and GitHub login:

```
GET /api/v1/auth/oauth2/google
GET /api/v1/auth/oauth2/github
```

Configure via environment variables:

| Variable | Description |
|----------|-------------|
| `TACHYON_GOOGLE_CLIENT_ID` | Google OAuth2 client ID |
| `TACHYON_GOOGLE_CLIENT_SECRET` | Google OAuth2 client secret |
| `TACHYON_GITHUB_CLIENT_ID` | GitHub OAuth2 client ID |
| `TACHYON_GITHUB_CLIENT_SECRET` | GitHub OAuth2 client secret |

## Password Security

Passwords are hashed using Argon2id with the following parameters:

| Parameter | Value |
|-----------|-------|
| Memory cost | 64 MiB |
| Time cost | 3 iterations |
| Parallelism | 4 lanes |

## Security Headers

All responses include:

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Content-Security-Policy` (configurable)
- `Strict-Transport-Security` (in production)

## Further Reading

- [API Reference](api-reference.html) - Auth endpoints
- [Configuration](configuration.html) - Environment variables
- [Deployment](deployment.html) - Production security
