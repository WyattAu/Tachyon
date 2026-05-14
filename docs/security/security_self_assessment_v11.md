# Security Self-Assessment: Tachyon v11.0.0

**Date:** 2026-05-14 | **Assessor:** Automated | **Classification:** Security Documentation

## Scope

All 16 crates in the Tachyon workspace. Server, renderer, database, RBAC, storage, and middleware components.

## Assessment Results

### Input Validation

- HTML sanitization: ammonia::clean() applied to all renderer output (sanitize.rs)
- SQL injection: All queries use parameterized statements via sqlx
- XSS: Script tags, event handlers, javascript: URIs stripped by ammonia
- Rate limiting: Per-IP rate limiting with configurable thresholds (rate_limit.rs)

### Authentication and Authorization

- JWT-based authentication with configurable expiry
- RBAC with hierarchical roles (owner, admin, member, viewer)
- 13-layer middleware stack including auth guard
- HMAC webhook signature verification (billing/handlers.rs)

### Cryptographic Practices

- CSPRNG nonce generation with fallback to static nonce (security_headers.rs)
- HMAC-SHA256 for webhook verification
- Password hashing via argon2
- TLS configuration fields in ServerConfig

### Error Handling

- Zero unwrap() on fallible operations in server request paths
- System clock operations use unwrap_or_default()
- HMAC key construction returns Result
- CSPRNG unavailable: logs error, returns static nonce (degraded mode)

### Dependency Security

- cargo audit in CI pipeline
- No known critical vulnerabilities
- SBOM generation configured

### Secrets Management

- Pre-commit hook scans for hardcoded secrets (API keys, tokens, passwords)
- Environment variable based configuration
- .env.example provided, .env in .gitignore

### Transport Security

- HTTPS configuration supported via ServerConfig
- CSP headers with nonce injection
- HSTS, X-Content-Type-Options, X-Frame-Options headers set

### Logging and Monitoring

- Structured JSON logging for production (TACHYON_LOG_FORMAT=json)
- Per-module log level configuration (TACHYON_LOG_FILTER)
- Grafana dashboards with 9 alerting rules
- Prometheus metrics endpoint

### Supply Chain

- All dependencies vendored or locked via Cargo.lock
- No runtime downloads
- WASM plugins sandboxed

## Outstanding Items

- CSP 'unsafe-inline' remains in style-src (requires nonce injection into SSG templates - deferred)
- SSL/TLS termination requires deployment-level configuration (Certbot/nginx sidecar)
- E2E security testing limited to auth, documents, navigation flows

## Risk Rating

Overall: LOW. No critical or high-severity findings. Two medium items (CSP unsafe-inline, TLS deployment config) deferred to post-v11.0.

## Recommendations

1. Complete CSP nonce injection into SSG templates (Phase A.2 follow-up)
2. Add Certbot/Let's Encrypt to deployment configuration
3. Expand E2E security test coverage to billing, admin, collaboration flows
4. Schedule quarterly cargo audit and dependency review
