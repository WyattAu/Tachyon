# Security Architecture

Documentation of Tachyon's security architecture and controls.

## Overview

Tachyon implements defense-in-depth security:

1. **Memory Safety**: Rust's ownership system
2. **Input Validation**: All inputs sanitized
3. **Authentication**: Multiple providers supported
4. **Authorization**: Role-based access control
5. **Encryption**: TLS for transport
6. **Audit**: Complete access logging

## Threat Model

### Assets

| Asset | Sensitivity |
|-------|-------------|
| Documents | High |
| User credentials | Critical |
| Session tokens | High |
| Audit logs | Medium |
| Configuration | Medium |

### Threats

| Threat | Mitigation |
|--------|------------|
| Unauthorized access | Authentication + RBAC |
| Data interception | TLS encryption |
| Injection attacks | Input validation, parameterized queries |
| XSS | Output encoding, CSP |
| CSRF | Token validation |
| DoS | Rate limiting |

## Authentication

### Supported Providers

| Provider | Use Case |
|----------|----------|
| Kanidm | Enterprise SSO |
| OAuth 2.0 | Social login |
| LDAP | Corporate directory |
| Local | Development |

### Token-Based Auth

```rust
pub struct Token {
    pub user_id: UserId,
    pub exp: i64,      // Expiration
    pub iat: i64,      // Issued at
    pub scope: Vec<Scope>,
}

pub fn verify_token(token: &str) -> Result<Token> {
    let decoded = decode::<Token>(
        token,
        &DECODING_KEY,
        &Validation::new(Algorithm::RS256),
    )?;
    
    if decoded.claims.exp < Utc::now().timestamp() {
        return Err(Error::TokenExpired);
    }
    
    Ok(decoded.claims)
}
```

### Session Management

```rust
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Session {
    pub fn is_valid(&self) -> bool {
        self.expires_at > Utc::now()
    }
}
```

## Authorization

### Role-Based Access Control

```rust
pub enum Role {
    Viewer,     // Read public documents
    Commenter,  // + Add comments
    Editor,     // + Create/edit documents
    Reviewer,   // + Approve changes
    Admin,      // Full access
}

impl Role {
    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::Viewer => vec![Permission::ReadPublic],
            Role::Editor => vec![
                Permission::ReadPublic,
                Permission::CreateDocument,
                Permission::EditDocument,
            ],
            // ...
        }
    }
}
```

### Permission Check

```rust
pub fn check_access(
    user: &User,
    document: &Document,
    action: Action,
) -> Result<()> {
    // Check document-level access
    if !document.is_accessible_by(user) {
        return Err(Error::Forbidden);
    }
    
    // Check role permissions
    if !user.role.allows(action) {
        return Err(Error::Forbidden);
    }
    
    // Check group membership for restricted docs
    if document.visibility == Visibility::Restricted {
        let user_groups: HashSet<_> = user.groups.iter().collect();
        let doc_groups: HashSet<_> = document.allowed_groups.iter().collect();
        
        if user_groups.is_disjoint(&doc_groups) {
            return Err(Error::Forbidden);
        }
    }
    
    Ok(())
}
```

### Block-Level Redaction

```rust
pub fn redact_blocks(
    document: &Document,
    user: &User,
) -> Document {
    let mut redacted = document.clone();
    
    for block in &mut redacted.blocks {
        if block.is_restricted() {
            if !user.groups.contains(block.required_group()) {
                block.content = String::new();
                block.redacted = true;
            }
        }
    }
    
    redacted
}
```

## Input Validation

### Document Validation

```rust
pub fn validate_document(doc: &CreateDocumentRequest) -> Result<()> {
    // Title validation
    if doc.title.is_empty() {
        return Err(Error::Validation("Title required"));
    }
    if doc.title.len() > 200 {
        return Err(Error::Validation("Title too long"));
    }
    
    // Content validation
    if doc.content.len() > 10_000_000 {
        return Err(Error::Validation("Content too large"));
    }
    
    // Path traversal prevention
    if doc.path.contains("..") || doc.path.contains('/') {
        return Err(Error::Validation("Invalid path"));
    }
    
    Ok(())
}
```

### SQL Injection Prevention

```rust
// Always use parameterized queries
pub fn get_document(db: &Database, id: &str) -> Result<Document> {
    db.query_row(
        "SELECT * FROM documents WHERE id = ?",
        params![id],  // Parameterized
        |row| Document::from_row(row)
    )
}
```

### XSS Prevention

```rust
pub fn sanitize_html(input: &str) -> String {
    let mut clean = String::new();
    
    for c in input.chars() {
        match c {
            '<' => clean.push_str("&lt;"),
            '>' => clean.push_str("&gt;"),
            '&' => clean.push_str("&amp;"),
            '"' => clean.push_str("&quot;"),
            '\'' => clean.push_str("&#x27;"),
            _ => clean.push(c),
        }
    }
    
    clean
}
```

## Transport Security

### TLS Configuration

```rust
pub fn create_tls_config() -> Result<ServerConfig> {
    let cert = load_cert("cert.pem")?;
    let key = load_key("key.pem")?;
    
    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert, key)?;
    
    // Modern TLS settings
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    
    Ok(config)
}
```

### Security Headers

```rust
pub fn security_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        "nosniff".parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        "DENY".parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        "1; mode=block".parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        "default-src 'self'".parse().unwrap(),
    );
    
    headers
}
```

## Rate Limiting

```rust
pub struct RateLimiter {
    requests: DashMap<IpAddr, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn check(&self, ip: &IpAddr) -> Result<()> {
        let now = Instant::now();
        let window_start = now - self.window;
        
        let mut requests = self.requests.entry(ip.clone()).or_default();
        
        // Remove old requests
        requests.retain(|&t| t > window_start);
        
        if requests.len() >= self.max_requests {
            return Err(Error::RateLimited);
        }
        
        requests.push(now);
        Ok(())
    }
}
```

## Audit Logging

### Audit Events

```rust
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<UserId>,
    pub action: Action,
    pub resource_type: ResourceType,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub success: bool,
}

pub enum Action {
    DocumentCreate,
    DocumentRead,
    DocumentUpdate,
    DocumentDelete,
    UserLogin,
    UserLogout,
    PermissionChange,
}
```

### Logging Implementation

```rust
impl AuditLogger {
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        // Log to database
        self.db.execute(
            "INSERT INTO audit_log (...) VALUES (...)",
            params![...]
        )?;
        
        // Log to file
        info!(
            target: "audit",
            "{}: {} {} {}",
            event.user_id.unwrap_or_default(),
            event.action,
            event.resource_type,
            event.resource_id.unwrap_or_default()
        );
        
        Ok(())
    }
}
```

## Secrets Management

### Environment Variables

```bash
# Required secrets
DATABASE_URL=sqlite:/data/tachyon.db
JWT_SECRET_KEY=xxx
OAUTH_CLIENT_SECRET=xxx
```

### Configuration

```toml
[secrets]
# Never hardcode secrets
jwt_secret = "${JWT_SECRET_KEY}"
oauth_secret = "${OAUTH_CLIENT_SECRET}"
```

### Loading Secrets

```rust
pub fn load_secrets() -> Result<Secrets> {
    Ok(Secrets {
        jwt_key: env::var("JWT_SECRET_KEY")
            .map_err(|_| Error::MissingSecret("JWT_SECRET_KEY"))?,
        oauth_secret: env::var("OAUTH_CLIENT_SECRET")
            .map_err(|_| Error::MissingSecret("OAUTH_CLIENT_SECRET"))?,
    })
}
```

## Security Checklist

### Deployment

- [ ] TLS enabled with valid certificate
- [ ] Secrets stored in environment/secrets manager
- [ ] Database encrypted at rest
- [ ] Firewall rules configured
- [ ] Rate limiting enabled
- [ ] Audit logging enabled
- [ ] Security headers configured

### Development

- [ ] No hardcoded secrets
- [ ] Input validation on all endpoints
- [ ] Parameterized queries
- [ ] Output encoding
- [ ] Error messages don't leak information
- [ ] Dependencies audited

## Vulnerability Reporting

Report security issues to: security@tachyon.example.com

Do not open public issues for security vulnerabilities.

## Compliance

- **ISO/IEC 27001**: Information security management
- **SOC 2**: Service organization controls
- **GDPR**: Data protection (if applicable)
- **No telemetry**: All data stays on your infrastructure
