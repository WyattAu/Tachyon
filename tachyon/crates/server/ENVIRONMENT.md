# Tachyon Server Environment Variables

## Required

| Variable | Description | Default |
|---|---|---|
| `DATABASE_URL` | PostgreSQL connection string (`postgres://...` or `postgresql://...`) | `postgres://tachyon:tachyon@127.0.0.1:5433/tachyon` |
| `TACHYON_JWT_SECRETS` | Comma-separated JWT signing secrets (first = signing, rest = validation). **Must be set in production.** | Falls back to `TACHYON_JWT_SECRET`, then insecure default |
| `TACHYON_JWT_SECRET` | Single JWT signing secret (legacy alias for `TACHYON_JWT_SECRETS`) | `change-this-secret-key-in-production` |

## Server

| Variable | Description | Default |
|---|---|---|
| `TACHYON_HOST` | Bind address | `0.0.0.0` |
| `TACHYON_PORT` | HTTP listen port | `8080` |
| `TACHYON_TLS_ENABLED` | Enable TLS (`1`/`true`) | `false` |
| `TACHYON_TLS_CERT_PATH` | TLS certificate file path | — |
| `TACHYON_TLS_KEY_PATH` | TLS private key file path | — |

## Database

| Variable | Description | Default |
|---|---|---|
| `TACHYON_DATABASE_BACKEND` | `postgresql`, `sqlite`, or `mysql` | `postgresql` |
| `TACHYON_DATABASE_PATH` | Legacy database file path (SQLite) | — |
| `TACHYON_DB_MAX_CONNECTIONS` | Connection pool max | `10` |
| `TACHYON_DB_MIN_CONNECTIONS` | Connection pool min | `2` |
| `TACHYON_DB_ACQUIRE_TIMEOUT_MS` | Acquire timeout (ms) | `5000` |
| `TACHYON_DB_IDLE_TIMEOUT_SECS` | Idle connection timeout (s) | `600` |
| `TACHYON_PGBOUNCER_ENABLED` | PgBouncer mode | `false` |
| `TACHYON_READ_REPLICA_URLS` | Comma-separated read replica URLs | — |

## JWT

| Variable | Description | Default |
|---|---|---|
| `TACHYON_JWT_EXPIRATION` | Token TTL in seconds | `86400` (24h) |
| `TACHYON_JWT_ROTATION_ENABLED` | Enable rotation logging | `true` |

## Logging

| Variable | Description | Default |
|---|---|---|
| `RUST_LOG` / `TACHYON_LOG_LEVEL` | Log level filter | `info` |
| `TACHYON_LOG_FORMAT` / `LOG_FORMAT` | `text` or `json` | `text` |

## Authentication

### OAuth2

| Variable | Description | Default |
|---|---|---|
| `TACHYON_OAUTH2_ENABLED` | Enable OAuth2 | `false` |
| `TACHYON_GOOGLE_CLIENT_ID` | Google OAuth2 client ID | — |
| `TACHYON_GOOGLE_CLIENT_SECRET` | Google OAuth2 client secret | — |
| `TACHYON_GITHUB_CLIENT_ID` | GitHub OAuth2 client ID | — |
| `TACHYON_GITHUB_CLIENT_SECRET` | GitHub OAuth2 client secret | — |
| `TACHYON_OAUTH2_REDIRECT_BASE_URL` | OAuth2 callback base URL | — |

### Guest / Magic Link / SMS OTP

| Variable | Description | Default |
|---|---|---|
| `TACHYON_GUEST_LOGIN_ENABLED` | Auto-authenticate guests | `false` |
| `TACHYON_PUBLIC_NOTES_ENABLED` | Public notes without auth | `false` |
| `TACHYON_GUEST_USER_ID` | Guest user UUID | `00000000-0000-0000-0000-000000000099` |
| `TACHYON_SMS_OTP_ENABLED` | Enable SMS OTP | `false` |
| `TACHYON_SMS_OTP_TTL_SECS` | OTP TTL (s) | `300` |
| `TACHYON_TWILIO_ACCOUNT_SID` | Twilio account SID | — |
| `TACHYON_TWILIO_AUTH_TOKEN` | Twilio auth token | — |
| `TACHYON_TWILIO_FROM_NUMBER` | Twilio sender number | — |

## Email / SMTP

| Variable | Description | Default |
|---|---|---|
| `TACHYON_SMTP_URL` | SMTP connection URL | — |
| `TACHYON_SMTP_FROM` | Sender email address | — |
| `TACHYON_SMTP_USERNAME` | SMTP username | — |
| `TACHYON_SMTP_PASSWORD` | SMTP password | — |
| `TACHYON_SMTP_PORT` | SMTP port override | — |
| `TACHYON_SMTP_TLS` | Use TLS for SMTP | `true` |
| `TACHYON_EMAIL_HTTP_API_URL` | HTTP email API (Mailgun/SendGrid) | — |
| `TACHYON_EMAIL_HTTP_API_KEY` | HTTP email API key | — |

## Security

| Variable | Description | Default |
|---|---|---|
| `TACHYON_SECURITY_DEVELOPMENT` | Dev mode (disables strict CSP/HSTS) | `true` |
| `TACHYON_SECURITY_CSP_ENABLED` | Content-Security-Policy header | `true` |
| `TACHYON_SECURITY_CSP_CUSTOM` | Override CSP value | — |
| `TACHYON_SECURITY_PERMISSIONS_POLICY` | Permissions-Policy header | `true` |
| `TACHYON_SECURITY_COEP_ENABLED` | Cross-Origin-Embedder-Policy | `false` |
| `TACHYON_SECURITY_HSTS_ENABLED` | HSTS header | `true` |
| `TACHYON_SECURITY_FRAME_ANCESTORS` | CSP frame-ancestors | `'none'` |
| `TACHYON_SECURITY_TRUSTED_ORIGINS` | Comma-separated trusted origins | — |
| `TACHYON_SECURITY_MAX_REQUEST_SIZE_BYTES` | Max request body (bytes) | `10485760` (10MB) |
| `TACHYON_SECURITY_SESSION_EXPIRY_HOURS` | Session TTL (hours) | `24` |
| `TACHYON_SECURITY_MAX_CONCURRENT_SESSIONS` | Max sessions per user | `100` |

## Rate Limiting

| Variable | Description | Default |
|---|---|---|
| `TACHYON_RATE_LIMIT_ENABLED` | Enable rate limiting | `true` |
| `TACHYON_RATE_LIMIT_REDIS_URL` | Redis for distributed limiting | — |
| `TACHYON_RATE_LIMIT_ENDPOINTS` | JSON map of `{path: {max, window}}` | See defaults |

## CORS

| Variable | Description | Default |
|---|---|---|
| `TACHYON_CORS_ORIGINS` | Comma-separated allowed origins | `http://localhost:8080` |

> **Production:** Set specific origins. Wildcard `*` is rejected when `TACHYON_SECURITY_DEVELOPMENT=false`.

## CDN / Branding

| Variable | Description | Default |
|---|---|---|
| `TACHYON_CDN_ENABLED` | Enable CDN edge caching | `false` |
| `TACHYON_CDN_BASE_URL` | CDN base URL | — |
| `TACHYON_CDN_STATIC_TTL_SECS` | Static asset TTL | `3600` |
| `TACHYON_CDN_API_TTL_SECS` | API response TTL | `60` |
| `TACHYON_BRAND_COMPANY_NAME` | Company name in UI | `Tachyon` |
| `TACHYON_BRAND_PRIMARY_COLOR` | Primary color (hex) | `#3B82F6` |
| `TACHYON_BRAND_SECONDARY_COLOR` | Secondary color (hex) | `#10B981` |
| `TACHYON_BRAND_LOGO_URL` | Logo URL | — |
| `TACHYON_BRAND_FAVICON_URL` | Favicon URL | — |
| `TACHYON_BRAND_CUSTOM_CSS` | Custom CSS injected into `<head>` | — |
| `TACHYON_BRAND_CUSTOM_DOMAIN` | Custom domain for white-label | — |

## Redis / WebSocket

| Variable | Description | Default |
|---|---|---|
| `TACHYON_REDIS_PUBSUB_URL` | Redis for WebSocket pub/sub (horizontal scaling) | — |
| `TACHYON_STATIC_DIR` | Static file directory | `dist` |

---

## Production Recommendations

### Security

- **JWT secrets:** Set `TACHYON_JWT_SECRETS` to a comma-separated list of at least 64-character random strings. The first secret signs new tokens; all are validated. Rotate by prepending a new secret and removing the old one after all tokens expire.
- **CORS:** Set `TACHYON_CORS_ORIGINS` to your exact frontend origin(s). Never use `*` in production.
- **PostgreSQL password:** Set `POSTGRES_PASSWORD` in your `.env` file (used by both `postgres` and `server` services).
- **TLS termination:** Place a reverse proxy (nginx, Caddy, Cloudflare) in front of the server. If terminating TLS at the server, set `TACHYON_TLS_ENABLED=1` and provide cert/key paths.
- **Development mode:** Set `TACHYON_SECURITY_DEVELOPMENT=false` in production to enable strict CSP, HSTS, and reject wildcard CORS.

### Performance

- **Connection pool:** Increase `TACHYON_DB_MAX_CONNECTIONS` for high-traffic deployments (default: 10).
- **Redis:** Set `TACHYON_RATE_LIMIT_REDIS_URL` and `TACHYON_REDIS_PUBSUB_URL` for distributed rate limiting and WebSocket scaling across multiple server instances.
- **Read replicas:** Set `TACHYON_READ_REPLICA_URLS` to offload search/analytics queries.

### Operations

- **Logging:** Set `TACHYON_LOG_FORMAT=json` for structured logging in production (works with ELK, Datadog, etc.).
- **Rate limiting:** Default limits are applied per-endpoint. Override via `TACHYON_RATE_LIMIT_ENDPOINTS` JSON.
- **Database migrations:** Migrations run automatically on startup from `/app/migrations`.
