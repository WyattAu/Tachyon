# OWASP Top 10 (2021) Security Audit — Tachyon Server

Date: 2026-05-21
Scope: `tachyon-server` crate — HTTP middleware, authentication, validation, and configuration.

---

## A01:2021 – Broken Access Control

### Risk
Users acting outside intended permissions — accessing other users' data, escalating privileges, or modifying resources without authorization.

### What Tachyon Already Does
- **JWT-based authentication** (`middleware/auth.rs`): All non-public routes require a valid JWT or API key.
- **RBAC enforcement** (`middleware/auth.rs`): `AuthState.check_rbac_permission()` integrates with `tachyon-rbac` to evaluate `AccessRequest(subject, resource, action, context)`.
- **Role hierarchy**: `UserRole::Admin > Editor > Writer > Reader` with `can_perform()` checks.
- **Permission guards**: `PermissionGuard` and `require_permission_middleware` for fine-grained access.
- **Public path allowlist**: Only explicitly listed paths (`/api/v1/auth/login`, `/health`, etc.) skip authentication.
- **Team-scoped access**: `AuthContext.team_id` and team-based queries isolate tenant data.

### Gaps
- **No object-level authorization checks in middleware**: The middleware verifies identity but does not validate that the authenticated user owns or has access to the specific resource ID in the URL. This is currently left to route handlers.
- **No centralized resource ownership enforcement**: Each handler must independently verify ownership. Inconsistent implementation across handlers could lead to IDOR.
- **CORS wildcard in default config**: `CorsConfig::default()` uses `allowed_origins: ["*"]` — safe only in development.

### Recommendations
1. Add a middleware layer or extractor that validates resource ownership (e.g., `require_owner("document_id")`) to prevent IDOR systematically.
2. Ensure all route handlers use the RBAC system consistently — audit every handler for ownership checks.
3. Production config validation already blocks CORS `*` — keep this enforced. Document that `TACHYON_CORS_ALLOWED_ORIGINS` must be set in production.

---

## A02:2021 – Cryptographic Failures

### Risk
Exposure of sensitive data through weak cryptography, improper key management, or传输明文.

### What Tachyon Already Does
- **JWT with HMAC-SHA256** (`middleware/auth.rs`): Uses `jsonwebtoken` crate with `Algorithm::HS256`.
- **Multi-secret rotation** (`config.rs`): `JwtConfig.secrets` supports multiple keys; validation tries all, signing uses the first.
- **API keys stored as SHA-256 hashes** (`middleware/auth.rs`): `validate_api_key()` hashes with `Sha256` and queries by `key_hash`, never storing plaintext.
- **Key prefix stored separately**: `key_prefix` (first 12 chars) stored for lookup efficiency without exposing the full key.
- **Minimum secret length enforcement** (`config.rs:validate()`): Rejects secrets < 32 characters, warns if < 64.
- **TLS support** (`config.rs`): `enable_tls`, `tls_cert_path`, `tls_key_path` configuration. Production validation requires cert+key when TLS enabled.
- **HSTS** (`middleware/security_headers.rs`): `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`.
- **TrueLayer webhook signature verification**: `webhook_secret` configured for HMAC verification of payment webhooks.

### Gaps
- **No `kid` (Key ID) in JWT header**: Tokens don't identify which secret signed them, making rotation detection harder.
- **Default JWT secret is a placeholder**: `"change-this-secret-key-in-production"` — `validate()` catches this but it's still the default.
- **No encryption at rest for database**: Sensitive fields (emails, etc.) are stored plaintext in PostgreSQL.
- **No secret rotation logging**: When a token is validated with a non-primary secret, there's no indication rotation is in progress.

### Recommendations
1. Add `kid` to JWT header — implemented in Phase 5 (Task 3).
2. Add rotation detection logging — implemented in Phase 5 (Task 3).
3. Consider encrypting PII fields (email, display name) at rest using application-level encryption (AES-256-GCM) with a key management system.
4. Consider adding refresh token rotation to detect token reuse/theft.

---

## A03:2021 – Injection

### Risk
SQL injection, XSS, command injection, or other injection attacks through unsanitized user input.

### What Tachyon Already Does
- **Parameterized SQL queries** (`middleware/auth.rs`): All database queries use `sqlx::query()` with bind parameters (`$1`, `$2`), not string interpolation.
- **Comprehensive input validation** (`validation/`):
  - `common.rs`: `sanitize_string()`, `strip_html_tags()`, `validate_no_html()`, `validate_no_scripts()`, `validate_no_javascript_protocol()`, `validate_no_data_uri()`, `remove_control_chars()`.
  - `search.rs`: SQL injection pattern detection — blocks `--`, `/*`, `;--`, `UNION`, `OR 1=1`, `DROP`, `SELECT(`, etc.
  - `document.rs`: Validates titles, content, tags, slugs with length limits and sanitization.
  - `user.rs`: Username format validation (alphanumeric + `_`/`-`), email regex, password complexity, reserved username blocking.
- **Content Security Policy** (`middleware/security_headers.rs`): CSP with `default-src 'self'`, `object-src 'none'`, `script-src 'self'` prevents XSS.
- **XSS response header**: `X-Content-Type-Options: nosniff` prevents MIME sniffing.
- **Request size limit**: `max_request_size_bytes: 10MB` prevents oversized payload attacks.
- **HTML escaping**: `escape_html()` and `html_escape` crate for output encoding.

### Gaps
- **No CSP nonce in development mode**: Dev CSP allows `'unsafe-inline'` and `'unsafe-eval'`, which weakens XSS protection during development.
- **Search SQL injection patterns are blocklist-based**: Blocklists can be bypassed with encoding tricks. Parameterized queries are the real defense; the blocklist is defense-in-depth.
- **No server-side template injection (SSTI) protection documented**: If templates are used for SSR/SEO, ensure they sandbox user input.

### Recommendations
1. Ensure all database access uses parameterized queries (audit for any raw SQL string building).
2. Add CSP nonce support to development mode for consistency — or accept the dev relaxation as intentional.
3. Document that the search validation blocklist is supplementary to parameterized queries, not a replacement.
4. Audit any template rendering paths for SSTI.

---

## A04:2021 – Insecure Design

### Risk
Flawed architecture or design patterns that create security vulnerabilities.

### What Tachyon Already Does
- **Layered security architecture**: Authentication → RBAC → Rate limiting → Security headers → Input validation — multiple independent layers.
- **Separation of concerns**: Auth middleware, validation, and business logic are separated into distinct modules.
- **Fail-safe defaults**: Security headers enabled by default, CORS wildcard blocked in production, JWT validation fails closed.
- **Brute-force protection** (`middleware/rate_limit.rs`): `LoginAttemptTracker` with progressive lockout (5→60s, 10→5m, 20→15m, 50→1h, 100→24h).
- **Per-endpoint rate limits**: Login (5/min), register (3/min), password-reset (3/min).
- **Request tracing**: `x-request-id` on every response for forensic correlation.
- **Audit middleware**: Captures request data for security event analysis.
- **Guest access isolation**: `GuestConfig` with dedicated guest user ID and configurable enablement.

### Gaps
- **No threat model documented**: No formal threat model identifying attack surfaces and trust boundaries.
- **No session management strategy documented**: JWT is stateless but there's no explicit session revocation mechanism (no token blacklist).
- **Guest login risk**: If `guest_login_enabled: true`, any user can access the system as a guest without authentication.

### Recommendations
1. Create a threat model for the Tachyon server — identify trust boundaries between users, API consumers, and the database.
2. Implement JWT revocation (token blacklist in Redis/DB) for compromised tokens.
3. If guest login is enabled, ensure guest users are strictly limited to read-only access on public resources only.

---

## A05:2021 – Security Misconfiguration

### Risk
Insecure default configurations, incomplete setup, open cloud storage, misconfigured HTTP headers.

### What Tachyon Already Does
- **Comprehensive security headers** (`middleware/security_headers.rs`):
  - `Content-Security-Policy`: Strict CSP with nonce support.
  - `X-Frame-Options: DENY`: Prevents clickjacking.
  - `X-Content-Type-Options: nosniff`: Prevents MIME sniffing.
  - `Referrer-Policy: strict-origin-when-cross-origin`: Limits referrer leakage.
  - `Strict-Transport-Security`: 1-year max-age with includeSubDomains + preload.
  - `Permissions-Policy`: Restricts camera, microphone, geolocation, and 20+ other APIs.
  - `Cross-Origin-Embedder-Policy: require-corp` / `credentialless`.
  - `Cross-Origin-Opener-Policy: same-origin`.
  - `Cross-Origin-Resource-Policy: same-origin`.
- **Environment-aware configuration** (`config.rs`): Development mode relaxes CSP, disables HSTS; production mode enforces strict headers.
- **Configuration validation** (`config.rs:validate()`): Checks for default JWT secrets, TLS cert requirements, CORS wildcard in production, max request size limits, session expiry bounds.
- **Configurable via environment variables**: All security settings overridable without code changes.

### Gaps
- **Default config is development-mode**: `SecurityConfig::default()` has `development: true`, which disables HSTS and relaxes CSP. Deployers must explicitly set `TACHYON_SECURITY_DEVELOPMENT=false`.
- **No automated configuration hardening check**: No startup banner or health endpoint that reports security posture.
- **CSP report-only mode available but not configured by default**: Could be used for gradual rollout of stricter policies.

### Recommendations
1. Add a startup security posture summary that logs the current state of all security features (HSTS, CSP mode, CORS origins, etc.).
2. Consider a `/security/posture` admin endpoint for operational monitoring.
3. Default to production configuration when `TACHYON_ENV=production` is set, rather than requiring individual flags.

---

## A06:2021 – Vulnerable and Outdated Components

### Risk
Using libraries with known vulnerabilities.

### What Tachyon Already Does
- **Rust ecosystem**: Memory-safe language eliminates entire classes of vulnerabilities (buffer overflows, use-after-free).
- **Pinned dependencies via Cargo.lock**: Ensures reproducible builds.
- **Modern crate versions**: `axum 0.8`, `sqlx 0.8`, `jsonwebtoken 9`, `tokio 1` — all current.
- **TLS via `rustls`**: No OpenSSL dependency, reducing attack surface (`reqwest` uses `rustls-tls`).
- **No native C dependencies** in the security-critical path.

### Gaps
- **No automated dependency scanning**: No CI integration with `cargo audit` or Dependabot/Renovate for Rust crates.
- **No SBOM (Software Bill of Materials)**: No generated SBOM for compliance.

### Recommendations
1. Add `cargo audit` to CI pipeline — run on every push/PR.
2. Enable Dependabot or Renovate for `Cargo.toml` updates.
3. Generate an SBOM using `cargo-cyclonedx` or similar for compliance.

---

## A07:2021 – Identification and Authentication Failures

### Risk
Weak authentication mechanisms, credential stuffing, session management flaws.

### What Tachyon Already Does
- **JWT with issuer + audience validation**: Tokens are validated against configured `issuer` and `audience`, preventing cross-service token reuse.
- **Multi-method authentication**: JWT Bearer tokens and API keys (SHA-256 hashed).
- **API key security**:
  - Minimum length check (12 chars for prefix lookup).
  - SHA-256 hashed storage — plaintext never stored.
  - Expiry support (`expires_at` column).
  - Active flag (`is_active`) for revocation.
  - Last-used tracking (`last_used_at`).
- **Password complexity** (`validation/user.rs`): Minimum 8 chars, requires 3 of 4 character classes, blocks common passwords.
- **Brute-force protection** (`rate_limit.rs`):
  - `LoginAttemptTracker` with progressive lockout tiers.
  - Per-endpoint rate limits on auth endpoints.
  - IP-based tracking with isolation.
- **CORS credential control**: `allow_credentials` is `false` by default.
- **OAuth2 support**: Google and GitHub OAuth2 providers.
- **MFA support**: TOTP-based MFA with `totp` module.

### Gaps
- **No account lockout mechanism**: `LoginAttemptTracker` locks by IP, not by account. An attacker can try different IPs for the same account.
- **No credential stuffing detection**: No detection of the same password tried across many accounts.
- **No JWT refresh token rotation**: Refresh tokens are not rotated on use, preventing theft detection.
- **API key has no scope/permission field**: API keys inherit the user's full permissions with no scope restriction.

### Recommendations
1. Add per-account lockout (not just per-IP) to prevent distributed brute-force against a single account.
2. Implement refresh token rotation — issue a new refresh token on each use and invalidate the old one.
3. Add API key scopes to limit what an API key can do (e.g., `read:documents`, `write:documents`).
4. Add password breach check using HaveIBeenPwned API (k-anonymity model) during registration/password change.

---

## A08:2021 – Software and Data Integrity Failures

### Risk
Code or data that is modified without detection — insecure CI/CD, unsigned updates, untrusted CDNs.

### What Tachyon Already Does
- **Rust compile-time guarantees**: Type system prevents many integrity violations.
- **TrueLayer webhook verification**: `webhook_secret` used for HMAC signature verification of incoming payment webhooks.
- **CSP with `integrity` support**: CSP can enforce SRI (Subresource Integrity) for external resources.
- **No auto-updates**: Server doesn't auto-update, reducing supply chain risk during runtime.

### Gaps
- **No CI/CD pipeline integrity verification**: No documented process for verifying build provenance.
- **No code signing**: Release artifacts are not signed.
- **Tailwind CDN in CSP**: `style-src` allows `https://cdn.tailwindcss.com` — this is a third-party CDN without SRI. If compromised, it could inject malicious CSS.
- **No database migration integrity check**: SQL migrations are applied but not checksummed post-application.

### Recommendations
1. Remove `https://cdn.tailwindcss.com` from CSP in production — bundle Tailwind locally or use SRI.
2. Add SRI hashes for any external resources in the CSP.
3. Sign release artifacts and publish checksums.
4. Add migration checksum verification to detect tampering.

---

## A09:2021 – Security Logging and Monitoring Failures

### Risk
Insufficient logging and monitoring to detect and respond to security incidents.

### What Tachyon Already Does
- **Structured logging via `tracing`**: All middleware uses `tracing` with structured fields.
- **Authentication event logging**: `debug!("Authentication successful")` and `warn!("Authentication failed")` with user ID and method.
- **Rate limit logging**: `warn!("Rate limit exceeded")` with path, key, and retry_after.
- **Request tracing**: `request_id_middleware` generates unique IDs, `request_logging_middleware` captures request/response metadata.
- **Audit middleware**: `audit_middleware` captures request data for compliance.
- **Configurable log levels**: `TACHYON_LOG_LEVEL` and `TACHYON_LOG_FILTER` environment variables.
- **JSON log format**: `LogConfig.format: "json"` for production log aggregation.

### Gaps
- **No authentication success logging in production log level**: Success is `debug!` level, which may be filtered out in production (`info` level).
- **No login attempt tracking persistence**: `LoginAttemptTracker` is in-memory — lost on restart, no historical analysis.
- **No security event metrics**: No Prometheus counters for failed logins, rate limit hits, or auth errors.
- **No alerting integration**: No webhook or notification for suspicious activity (e.g., mass login failures).
- **No request body logging**: Audit middleware captures metadata but not request bodies (by design for PII, but limits forensic capability).

### Recommendations
1. Promote authentication success/failure to `info!` level to ensure visibility in production.
2. Persist login attempt data to the database for trend analysis and incident response.
3. Add Prometheus counters: `auth_failures_total`, `rate_limit_hits_total`, `login_attempts_total{status="success|failure"}`.
4. Add configurable alerting for security events (e.g., >10 failed logins from one IP in 5 minutes).
5. Consider logging a SHA-256 hash of the request body for forensic correlation without storing PII.

---

## A10:2021 – Server-Side Request Forgery (SSRF)

### Risk
Server makes requests to user-specified URLs, potentially accessing internal services.

### What Tachyon Already Does
- **Limited URL input surface**: The server accepts URLs in limited contexts (webhook URLs, file URLs, OAuth2 redirects).
- **URL validation** (`validation/common.rs`): `validate_url()` requires `http://` or `https://` scheme.
- **No direct file:// support**: URL validation rejects non-HTTP schemes.
- **OAuth2 redirect URL validation**: `redirect_base_url` is configured, not user-supplied.
- **TrueLayer environment isolation**: Sandbox vs production configuration.

### Gaps
- **No webhook URL allowlist**: If webhook URLs are user-supplied, they could target internal services (e.g., `http://169.254.169.254/` for cloud metadata).
- **No DNS rebinding protection**: An attacker could register a domain that resolves to an internal IP after validation.
- **reqwest client has no restrictions**: The `http_client: reqwest::Client` used for webhooks and external calls has no built-in SSRF protection.

### Recommendations
1. Add webhook URL validation that blocks private IP ranges (RFC 1918), link-local addresses, and cloud metadata endpoints.
2. Implement a URL allowlist or domain allowlist for webhook targets.
3. Configure `reqwest::Client` with a custom DNS resolver that rejects internal IPs.
4. Validate resolved IP addresses before making outbound requests.

---

## Summary

| OWASP Category | Risk Level | Status |
|---|---|---|
| A01 – Broken Access Control | Medium | Partial — RBAC exists, needs object-level checks |
| A02 – Cryptographic Failures | Low | Good — HMAC-SHA256, hashed API keys, HSTS |
| A03 – Injection | Low | Good — parameterized queries, input validation |
| A04 – Insecure Design | Medium | Good — layered defense, needs threat model |
| A05 – Security Misconfiguration | Low | Good — comprehensive headers, env-aware config |
| A06 – Vulnerable Components | Low | Good — Rust memory safety, modern deps |
| A07 – Auth Failures | Medium | Partial — strong auth, needs account lockout |
| A08 – Integrity Failures | Low | Partial — webhook verification, needs SRI |
| A09 – Logging Failures | Medium | Partial — tracing exists, needs security metrics |
| A10 – SSRF | Medium | Weak — no outbound URL restrictions |
