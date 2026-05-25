# Tachyon Security Architecture

## Architecture Overview

Tachyon employs a defense-in-depth security model with multiple layers: network (nginx), transport (TLS), application (middleware chain), and data (parameterized queries, hashing). All security-relevant events are audit-logged.

```
Client
  │
  ▼
nginx (TLS termination, rate limiting, security headers)
  │
  ▼
Axum middleware chain:
  Request ID → Compression → CORS → Security Headers → CSP Nonce →
  Auth (JWT/API Key) → Rate Limiting → Audit → Request Size Limit →
  Router (RBAC enforcement at handler level)
  │
  ▼
PostgreSQL (parameterized queries) + Redis (rate limiting, sessions)
```

---

## Authentication Flow

### JWT Authentication

```
┌──────────┐     POST /auth/login      ┌──────────────┐
│  Client   │ ───────────────────────▶ │  Server       │
│          │     {username, password}  │               │
│          │                           │  1. Validate  │
│          │     200 {access_token}    │     creds     │
│          │ ◀─────────────────────── │  2. Generate  │
│          │                           │     JWT       │
│          │     GET /api/v1/...       │  3. Sign w/   │
│          │ ───────────────────────▶ │     HS256     │
│          │     Authorization:        │               │
│          │     Bearer <token>        │  4. Validate  │
│          │                           │     signature │
│          │     200 {data}            │  5. Check exp │
│          │ ◀─────────────────────── │  6. Extract   │
└──────────┘                           │     claims    │
                                       └──────────────┘
```

**JWT structure:**
- Algorithm: HS256 (HMAC-SHA256)
- Claims: `sub` (user ID), `iss` (issuer), `aud` (audience), `exp` (expiration), `iat` (issued-at), `role`, `permissions[]`, `team_id`
- Signing: First secret in `TACHYON_JWT_SECRETS` signs new tokens
- Validation: All secrets tried in order (supports rotation)
- Key rotation: Set `TACHYON_JWT_SECRETS=old_secret,new_secret` — new tokens sign with `new_secret`, existing tokens validate against both

**Source:** `tachyon/crates/server/src/middleware/auth.rs`, `tachyon/crates/server/src/routes/user/handlers.rs`

### API Key Authentication

```
┌──────────┐     GET /api/v1/...       ┌──────────────┐
│  Client   │ ───────────────────────▶ │  Server       │
│          │     X-API-Key: <key>      │               │
│          │                           │  1. SHA-256   │
│          │                           │     hash key  │
│          │                           │  2. Lookup by │
│          │     200 {data}            │     prefix +  │
│          │ ◀─────────────────────── │     hash      │
│          │                           │  3. Check exp │
└──────────┘                           │  4. Update    │
                                       │     last_used │
                                       └──────────────┘
```

- Keys stored as SHA-256 hash + 12-char prefix in `api_keys` table
- Supports expiration dates and deactivation
- Last-used timestamp tracked for rotation decisions
- Header name configurable via `TACHYON_API_KEY_HEADER` (default: `X-API-Key`)

**Source:** `tachyon/crates/server/src/middleware/auth.rs:248-303`

### Authentication Middleware Chain

```
Request → Extract Authorization header
         ├─ Bearer token? → validate_jwt() → AuthContext { user_id, role, permissions }
         ├─ X-API-Key?    → validate_api_key() → AuthContext { user_id, role }
         └─ Neither?      → Reject 401 (unless public endpoint)
```

Public endpoints (no auth required): `/health`, `/ready`, `/api/v1/auth/register`, `/api/v1/auth/login`, `/api/v1/auth/password-reset`

---

## Authorization Model (RBAC)

### Architecture

```
┌─────────────────────────────────────────────────┐
│                  RBAC Enforcer                   │
│  (tachyon/crates/rbac)                          │
│                                                  │
│  ┌─────────────┐  ┌──────────────┐              │
│  │ Policy Store │  │  Permission   │              │
│  │ (policies)   │  │  Checker      │              │
│  │              │  │  (w/ cache)   │              │
│  └──────┬──────┘  └──────┬───────┘              │
│         │                │                       │
│         ▼                ▼                       │
│  ┌─────────────────────────────┐                │
│  │     Enforcement Decision     │                │
│  │  policy_effect AND           │                │
│  │  permission_allowed          │                │
│  │  → ALLOW or DENY             │                │
│  └─────────────────────────────┘                │
└─────────────────────────────────────────────────┘
```

### Role Hierarchy

| Role | Level | Capabilities |
|------|-------|-------------|
| Admin | 0 | Full system access, user management, impersonation, config changes |
| Editor | 1 | Create, edit, delete content; manage team members |
| Writer | 2 | Create and edit own content; comment on shared content |
| Reader | 3 | View shared/public content; search; comment |

### Permission Resolution

1. **Policy evaluation**: Subject-role-resource-action tuple evaluated against policy store
2. **Permission check**: Fine-grained permission matching with inheritance (e.g., `document.*` implies `document.read`, `document.write`, `document.delete`)
3. **Cache layer**: LRU permission cache (1000 entries) with subject/resource invalidation
4. **Admin bypass**: Admin role short-circuits all permission checks (audited)

### Scoping Boundaries

- **Documents**: Owner-based with explicit sharing; private visibility restricts to owner
- **Teams**: Team membership required for team-scoped resources
- **Spaces**: Space membership required for space-scoped resources
- **Organizations**: Organization-level roles for cross-team management

**Source:** `tachyon/crates/rbac/`, `tachyon/crates/server/src/middleware/auth.rs`

---

## WebSocket Security

### Connection Establishment

```
Client                              Server
  │                                    │
  │  GET /ws (Upgrade: websocket)      │
  │  Cookie: session=...               │
  │ ─────────────────────────────────▶ │
  │                                    │  1. Validate auth
  │                                    │  2. Check max connections
  │  101 Switching Protocols           │
  │ ◀───────────────────────────────── │
  │                                    │
  │  Binary/Text frames                │
  │ ◀────────────────────────────────▶ │
  │                                    │  3. Rate limit messages
  │                                    │  4. Validate message schema
```

### Security Controls

| Control | Implementation |
|---------|---------------|
| Authentication | JWT extracted from query param or cookie during upgrade handshake |
| Connection limit | Configurable max concurrent connections (`ConnectionManager` cap) |
| Message validation | JSON schema validation on all incoming messages; malformed messages disconnect |
| Room isolation | Document-scoped rooms; users must have read access to join |
| Origin validation | WebSocket upgrade rejected if Origin header doesn't match allowed origins |
| CRDT collaboration | Binary relay via `crdt_handler.rs`; server maintains Yrs document state |

### Endpoints

- `/ws` — General WebSocket for document editing, presence, activity
- `/ws/crdt/{roomId}` — CRDT sync (y-websocket protocol) for real-time collaboration

**Source:** `tachyon/crates/server/src/websocket/`

---

## CORS Policy

### Configuration

```rust
// CorsConfig (config.rs)
{
    enabled: true,
    allowed_origins: Vec<String>,   // Explicit origin allowlist
    allowed_methods: Vec<String>,   // GET, POST, PUT, DELETE, PATCH, OPTIONS
    allowed_headers: Vec<String>,   // Content-Type, Authorization, X-API-Key, ...
    expose_headers: Vec<String>,    // X-Request-Id, X-RateLimit-*, ...
    allow_credentials: bool,        // false by default
    max_age_secs: Option<u64>,      // Preflight cache duration
}
```

### Security Rules

1. **Production**: Wildcard (`*`) origin rejected by config validation — must specify explicit origins
2. **Development**: Wildcard allowed with warning
3. **Credentials**: `allow_credentials` cannot be true when using wildcard origin (enforced by tower-http)
4. **Trusted origins**: `TACHYON_CORS_ALLOWED_ORIGINS` adds to the security policy enforcement origins
5. **Preflight**: All CORS preflight requests include `Access-Control-Allow-Methods` and `Access-Control-Allow-Headers`

**Source:** `tachyon/crates/server/src/lib.rs:715-774`, `tachyon/crates/server/src/middleware/cors.rs`

---

## Rate Limiting Architecture

### Multi-Layer Design

```
Layer 1: nginx
  │  limit_req_zone 10r/s (IP-based)
  ▼
Layer 2: Application rate limiter
  │  Per-IP + Per-User (authenticated get higher limits)
  │  Per-endpoint overrides (tighter on auth, relaxed on health)
  ▼
Layer 3: Per-resource guards
     Import rate limit (in-memory mutex)
     API key usage tracking
```

### Implementation Details

| Component | Backend | Fallback |
|-----------|---------|----------|
| Rate limit store | Redis (atomic INCR + EXPIRE) | In-memory (DashMap with sliding window) |
| Key strategy | IP address for anonymous; user ID for authenticated | Same |
| Window | Fixed 60-second window per key | Same |
| Headers | `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` | Same |
| Response | 429 with `Retry-After` header | Same |

### Endpoint-Specific Limits

| Endpoint Category | Limit | Override Key |
|-------------------|-------|-------------|
| Auth (login/register) | Tighter | `auth` |
| Health/ready | Relaxed | `health` |
| Import/export | Per-user mutex | N/A |
| General API | Default RPM | Default |

**Source:** `tachyon/crates/server/src/middleware/rate_limit.rs`

---

## Audit Logging Architecture

### Event Flow

```
Handler / Middleware
  │
  │  AuditLogger::log(event)
  ▼
┌──────────────────────────┐
│      AuditLogger          │
│                           │
│  ┌─────────────────────┐ │
│  │ Structured logging   │ │  → tracing (stdout/JSON)
│  │ (tracing::info!)     │ │
│  └─────────────────────┘ │
│                           │
│  ┌─────────────────────┐ │
│  │ Database storage     │ │  → PostgreSQL audit_events table
│  │ (optional)           │ │
│  └─────────────────────┘ │
└──────────────────────────┘
```

### Event Structure

```rust
AuditEvent {
    event_type: AuditEventType,      // 78 categories
    severity: AuditSeverity,         // Low / Medium / High / Critical
    action: String,                  // e.g. "user_login", "document_update"
    description: String,             // Human-readable detail
    actor_id: Option<String>,        // Who performed the action
    resource_type: Option<String>,   // What was acted upon
    resource_id: Option<String>,     // Specific resource identifier
    context: AuditContext {           // Request context
        ip_address,
        user_agent,
        request_id,
        session_id,
        device_id,
        geo_location,
    },
    metadata: BTreeMap<String, String>, // Additional key-value data
    timestamp: DateTime<Utc>,
}
```

### Security Event Helpers

The `audit.rs` module provides typed constructors for common security events:

- `auth_success()`, `auth_failure()`, `auth_locked()`
- `xss_attempt()`, `sql_injection_attempt()`
- `csrf_validation_failure()`, `cors_violation()`
- `rate_limit_exceeded()`, `suspicious_activity()`
- `permission_denied()`, `input_validation_failure()`

**Source:** `tachyon/crates/server/src/audit.rs`
