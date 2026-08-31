# Configuration Guide

This guide covers all configuration options for Tachyon.

## Configuration Methods

Tachyon can be configured using:

1. **Configuration File**: `tachyon.toml`
2. **Environment Variables**: Upper-case with `TACHYON_` prefix
3. **Command-Line Arguments**: Passed to the executable

Priority: Command-Line > Environment Variables > Configuration File > Defaults

## Configuration File

Create `tachyon.toml` in your working directory:

```toml
# Server Configuration
[server]
host = "0.0.0.0"
port = 8080
database_url = "postgres://tachyon:password@localhost:5432/tachyon"
cache_size_mb = 256

# TLS Configuration
enable_tls = false
tls_cert_path = "/path/to/cert.pem"
tls_key_path = "/path/to/key.pem"

# JWT Configuration
[jwt]
secret = "your-secret-key-minimum-32-characters-long"
expiration_secs = 86400  # 24 hours
issuer = "tachyon-server"
audience = "tachyon-client"

# API Key Configuration
[api_keys]
enabled = true
header_name = "X-API-Key"
key_prefix = "tchk_"

# CORS Configuration
[cors]
enabled = true
allowed_origins = ["*"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"]
allowed_headers = ["Content-Type", "Authorization", "X-API-Key"]
exposed_headers = []
allow_credentials = false
max_age_secs = 3600

# WebSocket Configuration
[websocket]
enabled = true
path = "/ws"
max_message_size = 10485760  # 10MB
connection_timeout_secs = 300
heartbeat_interval_secs = 30
max_connections = 1000

# Guest Access Configuration
[guest]
guest_login_enabled = false
public_notes_enabled = false
guest_user_id = "00000000-0000-0000-0000-000000000099"

# Rate Limiting Configuration
[rate_limit]
enabled = true
redis_url = "redis://localhost:6379"
default_requests_per_minute = 1000
cleanup_interval_secs = 60

[rate_limit.endpoint_limits]
"/api/v1/auth/login" = { max_requests = 5, window_secs = 60 }
"/api/v1/auth/guest" = { max_requests = 3, window_secs = 60 }
"/api/v1/documents" = { max_requests = 100, window_secs = 60 }

# Security Configuration
[security]
enable_security_headers = true
environment = "production"
enable_hsts = true
hsts_max_age = 31536000
hsts_include_subdomains = true
hsts_preload = false
csp_report_only = false

[security.csp_directives]
"default-src" = "'self'"
"script-src" = "'self' 'unsafe-inline'"
"style-src" = "'self' 'unsafe-inline'"

# Rendering Configuration
[rendering]
math_engine = "katex"
syntax_theme = "axiom-dark"
enable_diagrams = true
cache_rendered_html = true

# Search Configuration
[search]
index_path = "./search-index"
auto_index = true
index_interval_secs = 300

# Logging Configuration
[logging]
level = "info"
format = "json"
output = "/var/log/tachyon/server.log"
```

## Server Configuration

### Basic Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `host` | string | `0.0.0.0` | Server bind address |
| `port` | integer | `8080` | Server port |
| `database_url` | string | - | PostgreSQL connection string |
| `cache_size_mb` | integer | `256` | Cache size in megabytes |

### TLS Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable_tls` | boolean | `false` | Enable HTTPS |
| `tls_cert_path` | string | - | Path to TLS certificate |
| `tls_key_path` | string | - | Path to TLS private key |

Example:
```toml
[server]
enable_tls = true
tls_cert_path = "/etc/letsencrypt/live/example.com/fullchain.pem"
tls_key_path = "/etc/letsencrypt/live/example.com/privkey.pem"
```

## Authentication Configuration

### JWT Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `secret` | string | Required | JWT signing secret (min 32 chars) |
| `expiration_secs` | integer | `86400` | Token expiration in seconds |
| `issuer` | string | `tachyon-server` | Token issuer |
| `audience` | string | `tachyon-client` | Token audience |

Example:
```toml
[jwt]
secret = "change-this-to-a-secure-random-string-at-least-32-characters"
expiration_secs = 604800  # 7 days
issuer = "tachyon.example.com"
audience = "tachyon-web"
```

### API Key Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable API key authentication |
| `header_name` | string | `X-API-Key` | Header for API key |
| `key_prefix` | string | `tchk_` | API key prefix |

## CORS Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable CORS |
| `allowed_origins` | array | `["*"]` | Allowed origins |
| `allowed_methods` | array | All methods | Allowed HTTP methods |
| `allowed_headers` | array | Common headers | Allowed headers |
| `exposed_headers` | array | `[]` | Exposed headers |
| `allow_credentials` | boolean | `false` | Allow credentials |
| `max_age_secs` | integer | `3600` | Preflight cache duration |

Example:
```toml
[cors]
enabled = true
allowed_origins = ["https://example.com", "https://app.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allow_credentials = true
```

## WebSocket Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable WebSocket |
| `path` | string | `/ws` | WebSocket endpoint path |
| `max_message_size` | integer | `10485760` | Max message size (bytes) |
| `connection_timeout_secs` | integer | `300` | Connection timeout |
| `heartbeat_interval_secs` | integer | `30` | Heartbeat interval |
| `max_connections` | integer | `1000` | Max concurrent connections |

## Guest Access Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `guest_login_enabled` | boolean | `false` | Enable guest login |
| `public_notes_enabled` | boolean | `false` | Enable public document access |
| `guest_user_id` | string | UUID | Guest user identifier |

Example:
```toml
[guest]
guest_login_enabled = true
public_notes_enabled = true
guest_user_id = "00000000-0000-0000-0000-000000000099"
```

## Rate Limiting Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable rate limiting |
| `redis_url` | string | - | Redis URL for distributed rate limiting |
| `default_requests_per_minute` | integer | `1000` | Default rate limit |
| `cleanup_interval_secs` | integer | `60` | Cleanup interval |
| `endpoint_limits` | map | - | Per-endpoint limits |

Example:
```toml
[rate_limit]
enabled = true
default_requests_per_minute = 500

[rate_limit.endpoint_limits]
"/api/v1/auth/login" = { max_requests = 5, window_secs = 60 }
"/api/v1/search" = { max_requests = 50, window_secs = 60 }
```

## Security Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable_security_headers` | boolean | `true` | Add security headers |
| `environment` | string | `development` | Environment mode |
| `enable_hsts` | boolean | `false` | Enable HSTS |
| `hsts_max_age` | integer | `31536000` | HSTS max age |
| `hsts_include_subdomains` | boolean | `true` | HSTS subdomains |
| `hsts_preload` | boolean | `false` | HSTS preload |
| `csp_report_only` | boolean | `false` | CSP report-only mode |

Example:
```toml
[security]
enable_security_headers = true
environment = "production"
enable_hsts = true

[security.csp_directives]
"default-src" = "'self'"
"img-src" = "'self' data: https:"
```

## Environment Variables

All configuration options can be set via environment variables:

```bash
# Server
export TACHYON_HOST=0.0.0.0
export TACHYON_PORT=8080
export DATABASE_URL=postgres://user:pass@host:5432/db

# JWT
export TACHYON_JWT_SECRET=your-secret-key
export TACHYON_JWT_EXPIRATION=86400

# TLS
export TACHYON_TLS_ENABLED=true
export TACHYON_TLS_CERT_PATH=/path/to/cert.pem
export TACHYON_TLS_KEY_PATH=/path/to/key.pem

# Guest Access
export TACHYON_GUEST_LOGIN_ENABLED=true
export TACHYON_PUBLIC_NOTES_ENABLED=true
```

## Validation

Tachyon validates configuration on startup. Common validation errors:

- **"Host cannot be empty"**: Set `host` or `TACHYON_HOST`
- **"Port must be greater than 0"**: Set valid `port` number
- **"JWT secret must be at least 32 characters"**: Use longer secret
- **"TLS certificate path required"**: Set cert path when TLS enabled

## Configuration Examples

### Development

```toml
[server]
host = "127.0.0.1"
port = 8080
database_url = "postgres://tachyon:dev@localhost:5432/tachyon_dev"

[jwt]
secret = "development-secret-key-do-not-use-in-production"
expiration_secs = 604800

[guest]
guest_login_enabled = true
public_notes_enabled = true

[security]
environment = "development"
enable_hsts = false
```

### Production

```toml
[server]
host = "0.0.0.0"
port = 443
database_url = "postgres://tachyon:${DB_PASSWORD}@db.internal:5432/tachyon"
enable_tls = true
tls_cert_path = "/etc/ssl/certs/tachyon.crt"
tls_key_path = "/etc/ssl/private/tachyon.key"

[jwt]
secret = "${JWT_SECRET}"
expiration_secs = 3600

[cors]
enabled = true
allowed_origins = ["https://docs.example.com"]
allow_credentials = true

[rate_limit]
enabled = true
redis_url = "redis://redis.internal:6379"

[security]
environment = "production"
enable_hsts = true
hsts_max_age = 31536000
```

## Next Steps

- [Authentication Setup](authentication.md) - Configure authentication
- [Deployment Guide](../developer/deployment.md) - Deploy to production
- [API Reference](../api/authentication.md) - API documentation
