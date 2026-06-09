# Information Security Policy

**Policy ID:** SEC-001
**Version:** 1.0
**Effective Date:** 2026-06-09
**Review Cycle:** Annual
**Classification:** Internal

## 1. Purpose

This policy establishes the information security framework for Tachyon, a knowledge management system. It defines the controls and practices required to protect the confidentiality, integrity, and availability of customer data processed by the Tachyon platform.

## 2. Scope

This policy applies to:

- All Tachyon server infrastructure (`tachyon/crates/server/`)
- Customer data stored in PostgreSQL/SQLite databases
- Authentication and authorization systems
- Network communications and API endpoints
- Development, staging, and production environments

## 3. Security Principles

### 3.1 Authentication & Identity Management

Tachyon implements multi-layer authentication:

| Mechanism | Implementation | File Reference |
|-----------|---------------|----------------|
| JWT Token Authentication | RS256/HS256 signed tokens with key rotation | `tachyon/crates/server/src/config.rs:124-157` |
| OAuth2 (Google, GitHub) | Delegated identity verification | `tachyon/crates/server/src/config.rs:491-511` |
| OIDC SSO | Enterprise single sign-on integration | `config.sso_oidc` field |
| SAML 2.0 SSO | Enterprise federation | `config.sso_saml` field |
| LDAP SSO | Directory service integration | `config.sso_ldap` field |
| Magic Link | Passwordless email authentication | `tachyon/crates/server/src/config.rs:404-424` |
| SMS OTP | Two-factor phone verification | `tachyon/crates/server/src/config.rs:426-460` |
| API Key | Service-to-service authentication | `tachyon/crates/server/src/config.rs:159-170` |

**Controls:**
- JWT secrets are minimum 32 characters; 64+ recommended (`config.rs:856-875`)
- JWT key rotation is enabled by default with multi-secret support (`config.rs:136-157`)
- Token expiration defaults to 24 hours; configurable per deployment (`config.rs:661-669`)
- API keys use prefixed format (`tchk_`) for identification (`config.rs:573-581`)

### 3.2 Authorization & Access Control

Role-Based Access Control (RBAC) is enforced at the API layer:

- Guest users have limited access when `guest_login_enabled` is true (`config.rs:211-219`)
- Public notes can be exposed without authentication (`config.rs:215`)
- Rate limiting enforces per-endpoint quotas (`config.rs:222-245`)
- Concurrent session limits prevent credential sharing (`config.rs:308`)

### 3.3 Data Protection

- **Encryption in Transit:** TLS enforcement with configurable HSTS (`config.rs:253-309`)
- **Encryption at Rest:** Database-level encryption via PostgreSQL/SQLite
- **Request Size Limits:** Maximum 10MB default, configurable (`config.rs:301-302`)
- **Security Headers:** CSP, HSTS, Permissions-Policy, COEP enabled by default (`config.rs:253-309`)

### 3.4 Audit Logging

Tachyon provides comprehensive audit trails:

- Authentication events (login, logout, token refresh)
- Data access and modification events
- Administrative actions
- Rate limit violations
- Configuration changes

All logs are structured (JSON format available) and support correlation via request IDs.

### 3.5 Network Security

- CORS restrictions enforced in production (`config.rs:896-920`)
- Wildcard origins blocked in non-development mode
- WebSocket connections limited to configurable maximums (`config.rs:191-208`)
- Frame ancestors restricted to `'none'` by default (`config.rs:293`)

## 4. Vulnerability Management

- Dependencies audited via `cargo audit` in CI pipeline
- Security headers validated on every response
- Input validation enforced at API boundaries
- SQL injection prevented via parameterized queries (SQLx/SeaORM)

## 5. Compliance Evidence

| Control | Evidence Location |
|---------|-------------------|
| Authentication | `tachyon/crates/server/src/config.rs` — JWT, OAuth2, SSO configs |
| Rate Limiting | `config.rs:222-245` — Per-endpoint rate limits |
| Session Management | `config.rs:303-309` — Expiry and concurrent session limits |
| Security Headers | `config.rs:253-309` — CSP, HSTS, Permissions-Policy |
| TLS Enforcement | `config.rs:36-42` — Certificate and key path configuration |

## 6. Exceptions

Any exceptions to this policy must be documented in the Security Exception Register and approved by the Information Security Officer. Exceptions are reviewed quarterly.

## 7. Enforcement

Violations of this policy may result in disciplinary action, up to and including termination of employment or contract. Third-party violations may result in termination of service agreements.

## 8. Related Documents

- [Access Control Procedures](access-control.md)
- [Change Management Procedures](change-management.md)
- [Incident Response Plan](incident-response.md)
- [Data Retention Policy](data-retention.md)
- [Risk Assessment Framework](risk-assessment.md)
