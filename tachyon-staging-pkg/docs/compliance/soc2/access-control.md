# Access Control Procedures

**Policy ID:** ACC-001
**Version:** 1.0
**Effective Date:** 2026-06-09
**Review Cycle:** Quarterly
**Classification:** Internal

## 1. Purpose

This document defines the access control mechanisms implemented in Tachyon to ensure that only authorized users, processes, and systems can access customer data and system resources.

## 2. Authentication Mechanisms

### 2.1 JWT Token-Based Authentication

Tachyon uses JSON Web Tokens (JWT) for stateless authentication:

```
Implementation: tachyon/crates/server/src/config.rs:124-157
```

**Configuration:**
- Signing secrets: Minimum 32 characters, 64+ recommended
- Key rotation: Enabled by default with multi-secret support
- Token expiration: Configurable (default 24 hours)
- Issuer/Audience validation enforced

**Token Lifecycle:**
1. User authenticates via login endpoint
2. Server issues signed JWT with claims (user ID, roles, expiration)
3. Client includes JWT in `Authorization: Bearer <token>` header
4. Server validates signature, expiration, issuer, and audience
5. Token refresh endpoint issues new token before expiration

### 2.2 OAuth2 Delegated Authentication

```
Implementation: tachyon/crates/server/src/config.rs:491-511
Environment: TACHYON_OAUTH2_ENABLED, TACHYON_GOOGLE_CLIENT_ID, etc.
```

Supported providers:
- **Google OAuth2** — Consumer and enterprise accounts
- **GitHub OAuth2** — Developer-focused authentication

**Flow:**
1. Client redirects to provider authorization endpoint
2. Provider authenticates user and returns authorization code
3. Server exchanges code for access token
4. Server fetches user profile and creates/links account
5. Server issues Tachyon JWT

### 2.3 Enterprise SSO

#### OIDC (OpenID Connect)
```
Configuration: config.sso_oidc (HashMap<String, OidcConfig>)
```
- Multiple provider support via named configurations
- Standard OIDC discovery endpoint support
- Configurable claim mapping

#### SAML 2.0
```
Configuration: config.sso_saml (Option<SamlConfig>)
```
- Identity provider metadata URL
- SP certificate and key configuration
- Signed assertion validation

#### LDAP
```
Configuration: config.sso_ldap (Option<LdapConfig>)
```
- Directory server connection
- Bind DN and credential configuration
- User search base and filter

### 2.4 Passwordless Authentication

#### Magic Link
```
Implementation: tachyon/crates/server/src/config.rs:404-424
Environment: TACHYON_MAGIC_LINK_ENABLED, TACHYON_MAGIC_LINK_TTL_SECS
```
- Time-limited tokens (default: 15 minutes)
- One-time use enforcement
- Rate-limited to 3 requests per minute per endpoint

#### SMS OTP
```
Implementation: tachyon/crates/server/src/config.rs:426-460
Environment: TACHYON_SMS_OTP_ENABLED, TACHYON_TWILIO_ACCOUNT_SID, etc.
```
- Configurable TTL (default: 5 minutes)
- Provider abstraction (Twilio, custom API)
- Rate-limited to 3 requests per minute

### 2.5 API Key Authentication

```
Implementation: tachyon/crates/server/src/config.rs:159-170
Header: X-API-Key (configurable)
Prefix: tchk_ (identifiable key format)
```

Used for:
- Service-to-service communication
- Automated integrations
- CI/CD pipeline access

## 3. Authorization Controls

### 3.1 Role-Based Access Control (RBAC)

Access is enforced at the API handler level:

| Role | Capabilities |
|------|-------------|
| Guest | Read-only access to public notes (when enabled) |
| User | Full CRUD on own documents, read shared |
| Admin | User management, system configuration |

### 3.2 Guest Access Controls

```
Implementation: tachyon/crates/server/src/config.rs:211-219
Environment: TACHYON_GUEST_LOGIN_ENABLED, TACHYON_PUBLIC_NOTES_ENABLED
```

- Guest login disabled by default
- Public notes disabled by default
- Guest user ID is configurable for audit trail association

## 4. Rate Limiting

```
Implementation: tachyon/crates/server/src/config.rs:222-245
```

### 4.1 Default Limits

| Endpoint | Max Requests | Window |
|----------|-------------|--------|
| `/api/v1/auth/login` | 5 | 60s |
| `/api/v1/auth/register` | 3 | 60s |
| `/api/v1/auth/refresh` | 10 | 60s |
| `/api/v1/auth/guest` | 3 | 60s |
| `/api/v1/auth/password-reset` | 3 | 60s |
| `/api/v1/auth/magic-link/request` | 3 | 60s |
| `/api/v1/auth/sms-otp/request` | 3 | 60s |
| `/api/v1/documents` | 100 | 60s |
| `/health`, `/ready` | 1000 | 60s |

### 4.2 Distributed Rate Limiting

- Redis-backed rate limiting for horizontal scaling
- Configurable via `TACHYON_RATE_LIMIT_REDIS_URL`
- Per-endpoint overrides via `TACHYON_RATE_LIMIT_ENDPOINTS` JSON

## 5. Session Management

```
Implementation: tachyon/crates/server/src/config.rs:303-309
```

- **Session Expiry:** 24 hours default (configurable 1-720 hours)
- **Concurrent Sessions:** Maximum 100 per user (configurable)
- **Token Refresh:** Separate endpoint with its own rate limit

## 6. Network Access Controls

### 6.1 CORS Policy

```
Implementation: tachyon/crates/server/src/config.rs:172-189
```

- Production: Explicit origin whitelist required
- Wildcard (`*`) origins blocked in non-development mode
- Credentials allowed only with explicit origins

### 6.2 Security Headers

```
Implementation: tachyon/crates/server/src/config.rs:253-309
```

| Header | Default | Purpose |
|--------|---------|---------|
| Content-Security-Policy | Enabled | XSS prevention |
| Strict-Transport-Security | Enabled | HTTPS enforcement |
| Permissions-Policy | Enabled | Feature restriction |
| Cross-Origin-Embedder-Policy | Enabled | Spectre mitigation |
| X-Frame-Options | `'none'` | Clickjacking prevention |

## 7. Configuration Validation

```
Implementation: tachyon/crates/server/src/config.rs:820-963
```

Startup validation enforces:
- Minimum JWT secret length (32 characters)
- Non-default JWT secrets in production
- Valid CORS origins in production
- Reasonable session expiry limits
- Database pool consistency checks

## 8. Compliance Evidence

| Control | Evidence |
|---------|----------|
| JWT Configuration | `config.rs:124-157` — Secrets, rotation, expiration |
| OAuth2 | `config.rs:491-511` — Provider configurations |
| Rate Limiting | `config.rs:222-245` — Per-endpoint limits |
| Session Management | `config.rs:303-309` — Expiry, concurrent limits |
| CORS | `config.rs:172-189` — Origin restrictions |
| Security Headers | `config.rs:253-309` — CSP, HSTS, etc. |

## 9. Related Documents

- [Security Policy](security-policy.md)
- [Change Management Procedures](change-management.md)
- [Incident Response Plan](incident-response.md)
