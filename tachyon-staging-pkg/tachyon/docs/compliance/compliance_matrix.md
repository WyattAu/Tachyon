# Tachyon Compliance Matrix

## Overview

This document tracks security compliance across OWASP Top 10, security headers, authentication, authorization, and related controls. All statuses reflect the current codebase as of the latest audit.

---

## OWASP Top 10 (2021) Coverage

| # | Category | Status | Implementation | Notes |
|---|----------|--------|----------------|-------|
| A01 | Broken Access Control | **Covered** | RBAC enforcer with permission inheritance (`tachyon/rbac`), per-document ownership, team/space scoping, authorization middleware | Owner-only document access enforced; admin role has elevated privileges via permission checker |
| A02 | Cryptographic Failures | **Covered** | TLS 1.2/1.3 via nginx (`nginx/ssl.conf`), Argon2id password hashing, HMAC-SHA256 JWT signing, SHA-256 API key hashing | SSL cipher suite restricts to AEAD ciphers only; HSTS enabled with 2-year max-age |
| A03 | Injection | **Covered** | SQLx compile-time checked queries, input validation layer (`validation/`), parameterized queries throughout | All database access uses parameterized queries via sqlx macros; input validation for users, documents, nodes |
| A04 | Insecure Design | **Covered** | Defense-in-depth with layered middleware: auth -> RBAC -> rate limiting -> security headers -> audit logging | Threat model informed by Semgrep `p/security-audit` + `p/owasp-top-ten` rules in CI |
| A05 | Security Misconfiguration | **Covered** | Security headers middleware (`middleware/security_headers.rs`), CSP with nonce, CORS with origin allowlist, production config validation | Config validation rejects wildcard CORS in production; default-deny security posture |
| A06 | Vulnerable Components | **Covered** | `cargo audit` in CI (`security-new.yml`), Trivy container scanning, dependency-review-action on PRs, SBOM generation | Ignored vulnerabilities documented with justification in workflow |
| A07 | Auth Failures | **Covered** | JWT + API key auth (`middleware/auth.rs`), MFA support, account lockout, password complexity, rate limiting on auth endpoints | Multi-secret JWT rotation supported; login failure auditing |
| A08 | Software/Data Integrity | **Covered** | Signed JWTs with key rotation, CSP nonce per request, no CDN script loading (except Tailwind CDN in dev) | `upgrade-insecure-requests` CSP directive enforced |
| A09 | Logging/Monitoring Failures | **Covered** | Comprehensive audit logging (`audit.rs`) covering 78/78 endpoints with structured events, severity levels, and context | All CRUD, auth, admin, and security events logged with IP, user-agent, request-id |
| A10 | SSRF | **Covered** | No user-controlled URL fetching; webhook URLs validated; import/export restricted to file uploads | No server-side request construction from user input |

---

## Security Headers Implementation

| Header | Status | Value | Source |
|--------|--------|-------|--------|
| `Content-Security-Policy` | **Enabled** | Strict CSP with per-request nonce, `object-src 'none'`, `frame-ancestors 'none'` | `security_headers.rs` |
| `X-Frame-Options` | **Enabled** | `DENY` (production) / `SAMEORIGIN` (dev) | `security_headers.rs` |
| `X-Content-Type-Options` | **Enabled** | `nosniff` | `security_headers.rs` |
| `Strict-Transport-Security` | **Enabled** | `max-age=31536000; includeSubDomains; preload` (prod) | `security_headers.rs`, `nginx/ssl.conf` |
| `X-XSS-Protection` | **Enabled** | `0` (modern best practice, CSP supersedes) | `security_headers.rs` |
| `Referrer-Policy` | **Enabled** | `strict-origin-when-cross-origin` | `security_headers.rs` |
| `Permissions-Policy` | **Enabled** | Restrictive: camera=(), microphone=(), geolocation=() | `security_headers.rs` |
| `Cross-Origin-Embedder-Policy` | **Enabled** | `require-corp` | `security_headers.rs` |
| `Cross-Origin-Opener-Policy` | **Enabled** | `same-origin` | `security_headers.rs` |
| `Cross-Origin-Resource-Policy` | **Enabled** | `same-origin` | `security_headers.rs` |

---

## Authentication Controls

| Control | Status | Implementation |
|---------|--------|----------------|
| JWT authentication | **Active** | HS256 signed tokens with issuer/audience validation, configurable expiration (`middleware/auth.rs`) |
| JWT key rotation | **Active** | Multi-secret support; first secret signs, all secrets validate (`config.rs` JwtConfig) |
| API key authentication | **Active** | SHA-256 hashed keys stored in DB, prefix-based lookup, expiration support (`middleware/auth.rs`) |
| Password hashing | **Active** | Argon2id via `tachyon-core` |
| Password complexity | **Active** | Minimum length, uppercase, lowercase, digit, special char requirements (`validation/user.rs`) |
| MFA (TOTP) | **Active** | Time-based one-time password support (`routes/mfa.rs`) |
| Account lockout | **Active** | Configurable max attempts with lockout period |
| OAuth2 (Google/GitHub) | **Active** | CSRF state nonce validation, 10-minute TTL (`routes/oauth2.rs`) |
| Session management | **Active** | JWT-based with refresh token support, session revocation (`routes/session.rs`) |
| Guest access | **Active** | Configurable guest JWT with Reader role, audited guest login attempts |

---

## Authorization Controls

| Control | Status | Implementation |
|---------|--------|----------------|
| RBAC engine | **Active** | `tachyon-rbac` crate with policy-based enforcement, permission inheritance, and caching |
| Role hierarchy | **Active** | Admin > Editor > Writer > Reader with configurable custom roles |
| Permission system | **Active** | Fine-grained permissions with resource/action matching, wildcard support |
| Document ownership | **Active** | Creator-based ownership with explicit sharing; private docs require authorization |
| Team scoping | **Active** | Team membership determines access to team-scoped resources |
| Space scoping | **Active** | Space membership controls access to space-scoped documents |
| Organization scoping | **Active** | Organization-level membership and role management |
| Admin override | **Active** | Admin role bypasses permission checks (audited) |
| Impersonation | **Active** | Admin impersonation with full audit trail (`audit.rs` ImpersonationStarted/Ended) |

---

## Input Validation Coverage

| Area | Status | Implementation |
|------|--------|----------------|
| User registration | **Validated** | Username, email, password, display_name validation (`validation/user.rs`) |
| Document CRUD | **Validated** | Title, content, tags, visibility validation |
| Search queries | **Validated** | Query sanitization, pagination bounds |
| File uploads | **Validated** | File type, size limits via `request_size_limit` middleware |
| API parameters | **Validated** | Path parameters, query parameters validated at handler level |
| Import/export | **Validated** | Format validation, rate limiting on import (`routes/import.rs`) |
| Webhook URLs | **Validated** | URL format validation for webhook endpoints |
| Node/edge operations | **Validated** | Knowledge graph node/edge structure validation |

---

## Audit Logging Coverage

| Metric | Value |
|--------|-------|
| Total event types | 78 (see `audit.rs` AuditEventType enum) |
| Endpoint coverage | 78/78 endpoints (100%) |
| Severity levels | Low, Medium, High, Critical |
| Context captured | IP address, user-agent, request-id, session-id, device-id, geo-location |
| Storage | Database-backed via `AuditLogger` with structured logging |

### Key Audit Event Categories

- **Authentication**: Success, failure, lockout, expired, logout, token refresh/revocation
- **Authorization**: Permission grant/revoke, RBAC policy changes, suspicious activity
- **Data access**: Document CRUD, sharing, access events
- **Admin actions**: User management, role changes, configuration changes, impersonation
- **Security events**: Rate limit exceeded, XSS attempt, SQL injection attempt, CSRF failure, CORS violation
- **System events**: Backup created/restored, export/import, plugin lifecycle

---

## Rate Limiting Coverage

| Layer | Status | Implementation |
|-------|--------|----------------|
| Global rate limit | **Active** | Configurable requests-per-minute per IP (`middleware/rate_limit.rs`) |
| Per-user rate limit | **Active** | Higher limits for authenticated users vs anonymous |
| Per-endpoint overrides | **Active** | Configurable tighter/relaxed limits per path (`config.rs` EndpointRateLimit) |
| Redis backend | **Active** | Optional Redis-backed distributed rate limiting for multi-instance |
| In-memory fallback | **Active** | Automatic fallback when Redis unavailable |
| Rate limit headers | **Active** | `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` on every response |
| 429 response | **Active** | Returns `Retry-After` header when rate limited |
| Auth endpoints | **Active** | Tighter rate limits on login/register/password-reset |
| Health endpoints | **Active** | Relaxed rate limits for monitoring probes |
| Nginx layer | **Active** | Additional `limit_req_zone` at 10r/s in `nginx.conf` |

---

## Content Security Policy (CSP) Coverage

| Directive | Production Value | Dev Value |
|-----------|-----------------|-----------|
| `default-src` | `'self'` | `'self' 'unsafe-inline'` |
| `script-src` | `'self' 'wasm-unsafe-eval' 'nonce-{random}'` | `'self' 'unsafe-inline' 'unsafe-eval'` |
| `style-src` | `'self' 'nonce-{random}'` | `'self' 'unsafe-inline'` |
| `img-src` | `'self' data: https:` | `'self' data: https: http:` |
| `connect-src` | `'self' wss:` | `'self' ws: wss:` |
| `object-src` | `'none'` | `'none'` |
| `frame-ancestors` | `'none'` | `'self'` |
| `base-uri` | `'self'` | `'self'` |
| `form-action` | `'self'` | `'self'` |
| `upgrade-insecure-requests` | Enabled | Disabled |
| `block-all-mixed-content` | Enabled | Disabled |

---

## TLS Configuration

| Setting | Value | Source |
|---------|-------|--------|
| Protocols | TLSv1.2, TLSv1.3 | `nginx/ssl.conf` |
| Cipher suites | AEAD only (ECDHE-ECDSA/RSA-AES128/256-GCM, CHACHA20-POLY1305) | `nginx/ssl.conf` |
| Server preference | Off (client chooses) | `nginx/ssl.conf` |
| Session timeout | 1 day | `nginx/ssl.conf` |
| Session cache | 50MB shared | `nginx/ssl.conf` |
| Session tickets | Disabled (forward secrecy) | `nginx/ssl.conf` |
| OCSP stapling | Enabled with verify | `nginx/ssl.conf` |
| HSTS | max-age=63072000 (2 years) | `nginx/ssl.conf` |
| Application TLS | Optional built-in TLS via rustls config | `config.rs` |

---

## Secret Management

| Aspect | Implementation |
|--------|----------------|
| JWT secrets | `TACHYON_JWT_SECRETS` (comma-separated for rotation), validated non-empty |
| Database credentials | `DATABASE_URL` environment variable, not committed |
| Redis URL | `REDIS_URL` environment variable |
| API keys | SHA-256 hashed in database, never stored plaintext |
| OAuth2 secrets | Environment variables (`TACHYON_GOOGLE_*`, `TACHYON_GITHUB_*`) |
| Email credentials | `TACHYON_SMTP_*` environment variables |
| CI/CD secrets | GitHub Secrets for deployment workflows |
| Config validation | Rejects empty JWT secrets, warns on default/weak values |
| .gitignore | `.env*` files excluded from version control |
