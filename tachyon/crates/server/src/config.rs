// Server configuration module
// Manages server configuration for HTTP/2, TLS, and authentication

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Database URL (PostgreSQL connection string)
    pub database_url: String,
    /// Legacy database path (for backwards compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_path: Option<String>,
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Enable TLS
    pub enable_tls: bool,
    /// TLS certificate path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_cert_path: Option<String>,
    /// TLS key path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_key_path: Option<String>,
    /// JWT configuration
    pub jwt: JwtConfig,
    /// API key configuration
    pub api_keys: ApiKeyConfig,
    /// CORS configuration
    pub cors: CorsConfig,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
    /// Guest login configuration
    pub guest: GuestConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Security headers configuration
    pub security: SecurityConfig,
    /// Site configuration for SEO and SSR
    pub site: SiteConfig,
    /// Logging configuration
    pub log: LogConfig,
    /// OAuth2 configuration
    pub oauth2: OAuth2Config,
    /// TrueLayer payment configuration
    pub truelayer: TrueLayerConfig,
    /// SMTP URL for email delivery (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smtp_url: Option<String>,
    /// From address for outgoing emails (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smtp_from: Option<String>,
}

/// JWT configuration for token-based authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key for signing tokens
    pub secret: String,
    /// Token expiration time in seconds
    pub expiration_secs: u64,
    /// Issuer for JWT claims
    pub issuer: String,
    /// Audience for JWT claims
    pub audience: String,
}

/// API key configuration for service account authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Enable API key authentication
    pub enabled: bool,
    /// Header name for API key (default: "X-API-Key")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_name: Option<String>,
    /// Prefix for API keys (e.g., "tchk_")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_prefix: Option<String>,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    pub enabled: bool,
    /// Allowed origins (use "*" for any)
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Exposed headers
    pub exposed_headers: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
    /// Max age for preflight requests (seconds)
    pub max_age_secs: Option<u64>,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Enable WebSocket support
    pub enabled: bool,
    /// WebSocket path
    pub path: String,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Maximum concurrent connections
    pub max_connections: usize,
}

/// Guest login and public access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestConfig {
    /// Enable guest login (auto-authenticate as guest user)
    pub guest_login_enabled: bool,
    /// Enable public notes access (no authentication required)
    pub public_notes_enabled: bool,
    /// Guest user ID (for auto-authentication)
    pub guest_user_id: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Redis URL for distributed rate limiting (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,
    /// Default requests per minute
    pub default_requests_per_minute: u32,
    /// Cleanup interval for in-memory store (seconds)
    pub cleanup_interval_secs: u64,
    /// Per-endpoint rate limits
    #[serde(default)]
    pub endpoint_limits: BTreeMap<String, EndpointRateLimit>,
}

/// Per-endpoint rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateLimit {
    /// Maximum requests allowed
    pub max_requests: u32,
    /// Time window in seconds
    pub window_secs: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security headers
    pub enable_security_headers: bool,
    /// Whether the server is in development mode (affects CSP and other headers)
    #[serde(default = "default_true")]
    pub development: bool,
    /// Environment mode (affects CSP and other headers)
    pub environment: String,
    /// Enable HSTS (Strict Transport Security)
    #[serde(default = "default_true")]
    pub hsts_enabled: bool,
    /// Enable HSTS (Strict Transport Security) — legacy alias
    #[serde(default)]
    pub enable_hsts: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u64,
    /// HSTS include subdomains
    pub hsts_include_subdomains: bool,
    /// HSTS preload
    pub hsts_preload: bool,
    /// Content Security Policy report-only mode
    pub csp_report_only: bool,
    /// Custom CSP directives (override defaults)
    #[serde(default)]
    pub csp_directives: BTreeMap<String, String>,
    /// Enable Content-Security-Policy header
    #[serde(default = "default_true")]
    pub csp_enabled: bool,
    /// Override default CSP with a custom value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp_custom: Option<String>,
    /// Enable Permissions-Policy header
    #[serde(default = "default_true")]
    pub permissions_policy: bool,
    /// Enable Cross-Origin-Embedder-Policy header
    pub coep_enabled: bool,
    /// Allowed frame ancestors for CSP (e.g., "'none'", "'self'", "https://example.com")
    #[serde(default = "default_frame_ancestors")]
    pub frame_ancestors: String,
    /// Trusted origins for CORS (in addition to configured origins)
    #[serde(default)]
    pub trusted_origins: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_frame_ancestors() -> String {
    "'none'".to_string()
}

impl SecurityConfig {
    pub fn is_hsts_enabled(&self) -> bool {
        self.hsts_enabled || self.enable_hsts
    }

    pub fn is_development(&self) -> bool {
        self.development || self.environment == "development"
    }
}

/// Site configuration for SEO and server-side rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Site title (e.g., "Tachyon")
    pub title: String,
    /// Site description for meta tags
    pub description: String,
    /// Canonical base URL (e.g., "https://tachyon.dev")
    pub base_url: String,
    /// Theme color for mobile browsers
    pub theme_color: String,
    /// OG image URL (default site-wide image)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,
    /// Custom template directory path (overrides defaults)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_dir: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log format: "text" (default) or "json" (for production)
    pub format: String,
    /// Log level override (e.g., "info", "debug", "warn")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

/// TrueLayer configuration for open banking payments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrueLayerConfig {
    /// Enable TrueLayer payment processing
    pub enabled: bool,
    /// TrueLayer client ID
    pub client_id: String,
    /// TrueLayer client secret
    pub client_secret: String,
    /// Environment: "sandbox" or "production"
    pub environment: String,
    /// TrueLayer merchant account ID
    pub merchant_account_id: String,
    /// Webhook secret for signature verification
    pub webhook_secret: String,
}

impl Default for TrueLayerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_id: String::new(),
            client_secret: String::new(),
            environment: "sandbox".to_string(),
            merchant_account_id: String::new(),
            webhook_secret: String::new(),
        }
    }
}

/// OAuth2 provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OAuth2Config {
    /// Enable OAuth2 authentication
    pub enabled: bool,
    /// Google OAuth2 client ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_client_id: Option<String>,
    /// Google OAuth2 client secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_client_secret: Option<String>,
    /// GitHub OAuth2 client ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_client_id: Option<String>,
    /// GitHub OAuth2 client secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_client_secret: Option<String>,
    /// OAuth2 redirect base URL (e.g., "http://localhost:8080")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_base_url: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            database_url: "postgres://tachyon:tachyon@127.0.0.1:5433/tachyon".to_string(),
            database_path: None,
            cache_size_mb: 256,
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            jwt: JwtConfig::default(),
            api_keys: ApiKeyConfig::default(),
            cors: CorsConfig::default(),
            websocket: WebSocketConfig::default(),
            guest: GuestConfig::default(),
            rate_limit: RateLimitConfig::default(),
            security: SecurityConfig::default(),
            site: SiteConfig::default(),
            log: LogConfig::default(),
            oauth2: OAuth2Config::default(),
            truelayer: TrueLayerConfig::default(),
            smtp_url: None,
            smtp_from: None,
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-this-secret-key-in-production".to_string(),
            expiration_secs: 24 * 60 * 60, // 24 hours
            issuer: "tachyon-server".to_string(),
            audience: "tachyon-client".to_string(),
        }
    }
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            header_name: Some("X-API-Key".to_string()),
            key_prefix: Some("tchk_".to_string()),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "Accept".to_string(),
                "X-API-Key".to_string(),
                "X-Request-ID".to_string(),
            ],
            exposed_headers: vec![],
            allow_credentials: false,
            max_age_secs: Some(3600), // 1 hour
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/ws".to_string(),
            max_message_size: 10 * 1024 * 1024, // 10MB
            connection_timeout_secs: 300,       // 5 minutes
            heartbeat_interval_secs: 30,        // 30 seconds
            max_connections: 1000,
        }
    }
}

impl Default for GuestConfig {
    fn default() -> Self {
        Self {
            guest_login_enabled: false,
            public_notes_enabled: false,
            guest_user_id: "00000000-0000-0000-0000-000000000099".to_string(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut endpoint_limits = BTreeMap::new();

        endpoint_limits.insert(
            "/api/v1/auth/login".to_string(),
            EndpointRateLimit {
                max_requests: 5,
                window_secs: 60,
            },
        );
        endpoint_limits.insert(
            "/api/v1/auth/register".to_string(),
            EndpointRateLimit {
                max_requests: 3,
                window_secs: 60,
            },
        );
        endpoint_limits.insert(
            "/api/v1/auth/refresh".to_string(),
            EndpointRateLimit {
                max_requests: 10,
                window_secs: 60,
            },
        );
        endpoint_limits.insert(
            "/api/v1/auth/guest".to_string(),
            EndpointRateLimit {
                max_requests: 3,
                window_secs: 60,
            },
        );
        endpoint_limits.insert(
            "/api/v1/auth/password-reset".to_string(),
            EndpointRateLimit {
                max_requests: 3,
                window_secs: 60,
            },
        );
        endpoint_limits.insert(
            "/api/v1/documents".to_string(),
            EndpointRateLimit {
                max_requests: 100,
                window_secs: 60,
            },
        );

        Self {
            enabled: true,
            redis_url: None,
            default_requests_per_minute: 1000,
            cleanup_interval_secs: 60,
            endpoint_limits,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_security_headers: true,
            development: true,
            environment: "development".to_string(),
            hsts_enabled: true,
            enable_hsts: true,
            hsts_max_age: 31536000,
            hsts_include_subdomains: true,
            hsts_preload: true,
            csp_report_only: false,
            csp_directives: BTreeMap::new(),
            csp_enabled: true,
            csp_custom: None,
            permissions_policy: true,
            coep_enabled: true,
            frame_ancestors: "'none'".to_string(),
            trusted_origins: Vec::new(),
        }
    }
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Tachyon".to_string(),
            description: "A deterministic, high-performance knowledge management system."
                .to_string(),
            base_url: "http://localhost:8080".to_string(),
            theme_color: "#2563eb".to_string(),
            og_image: None,
            template_dir: None,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            format: "text".to_string(),
            level: None,
        }
    }
}

pub fn static_dir() -> String {
    std::env::var("TACHYON_STATIC_DIR").unwrap_or_else(|_| "dist".to_string())
}

impl ServerConfig {
    /// Create a new server configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get the WebSocket full path
    pub fn websocket_path(&self) -> String {
        self.websocket.path.clone()
    }

    /// Get JWT token expiration as Duration
    pub fn jwt_expiration(&self) -> Duration {
        Duration::from_secs(self.jwt.expiration_secs)
    }

    /// Get API key header name
    pub fn api_key_header(&self) -> String {
        self.api_keys
            .header_name
            .clone()
            .unwrap_or_else(|| "X-API-Key".to_string())
    }

    /// Validate configuration
    ///
    /// # Returns
    /// `Ok(())` if all checks pass, `Err(Vec<String>)` with all error messages otherwise.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if self.host.is_empty() {
            errors.push("Host cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("Port must be greater than 0".to_string());
        }

        if !self.database_url.is_empty()
            && !self.database_url.starts_with("postgres://")
            && !self.database_url.starts_with("postgresql://")
        {
            errors.push(
                "Database URL must start with postgres:// or postgresql://".to_string(),
            );
        }

        if self.enable_tls {
            if self.tls_cert_path.is_none()
                || self.tls_cert_path.as_ref().is_none_or(|p| p.is_empty())
            {
                errors.push(
                    "TLS certificate path required when TLS is enabled".to_string(),
                );
            }
            if self.tls_key_path.is_none()
                || self.tls_key_path.as_ref().is_none_or(|p| p.is_empty())
            {
                errors.push("TLS key path required when TLS is enabled".to_string());
            }
        }

        if self.jwt.secret.len() < 32 {
            errors.push("JWT secret must be at least 32 characters".to_string());
        } else if self.jwt.secret.len() < 64 {
            warnings.push(
                "JWT secret is less than 64 characters; consider using a longer secret for better security".to_string(),
            );
        }

        if self.jwt.expiration_secs == 0 {
            errors.push("JWT expiration must be greater than 0".to_string());
        }

        if self.cache_size_mb == 0 {
            errors.push("Cache size must be greater than 0".to_string());
        }

        if self.jwt.secret == "change-this-secret-key-in-production" {
            errors.push(
                "JWT secret must be changed from default value. Set TACHYON_JWT_SECRET environment variable.".to_string(),
            );
        }

        if self.cors.enabled {
            for origin in &self.cors.allowed_origins {
                if origin != "*"
                    && !origin.starts_with("http://")
                    && !origin.starts_with("https://")
                {
                    warnings.push(format!(
                        "CORS origin '{}' does not look like a valid URL",
                        origin
                    ));
                }
            }
            if self.cors.allowed_origins.contains(&"*".to_string()) {
                warnings.push(
                    "CORS is enabled with wildcard origin - this should be restricted in production"
                        .to_string(),
                );
            }
        }

        if let Some(ref level) = self.log.level {
            const VALID_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];
            if !VALID_LEVELS.contains(&level.as_str()) {
                warnings.push(format!(
                    "Log level '{}' is not a standard value (expected: trace, debug, info, warn, error)",
                    level
                ));
            }
        }

        for w in &warnings {
            tracing::warn!(config_warning = %w, "Configuration warning");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Load configuration from environment variables
///
/// # Returns
/// Server configuration loaded from environment
impl ServerConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("TACHYON_HOST") {
            config.host = host;
        }

        if let Ok(port) = std::env::var("TACHYON_PORT") {
            if let Ok(p) = port.parse::<u16>() {
                config.port = p;
            }
        }

        if let Ok(db_path) = std::env::var("TACHYON_DATABASE_PATH") {
            config.database_path = Some(db_path);
        }

        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(tls_enabled) = std::env::var("TACHYON_TLS_ENABLED") {
            config.enable_tls = tls_enabled == "1" || tls_enabled == "true";
        }

        if let Ok(cert_path) = std::env::var("TACHYON_TLS_CERT_PATH") {
            config.tls_cert_path = Some(cert_path);
        }

        if let Ok(key_path) = std::env::var("TACHYON_TLS_KEY_PATH") {
            config.tls_key_path = Some(key_path);
        }

        if let Ok(jwt_secret) = std::env::var("TACHYON_JWT_SECRET") {
            config.jwt.secret = jwt_secret;
        }

        if let Ok(jwt_expiration) = std::env::var("TACHYON_JWT_EXPIRATION") {
            if let Ok(exp) = jwt_expiration.parse::<u64>() {
                config.jwt.expiration_secs = exp;
            }
        }

        if let Ok(guest_login) = std::env::var("TACHYON_GUEST_LOGIN_ENABLED") {
            config.guest.guest_login_enabled = guest_login == "1" || guest_login == "true";
        }

        if let Ok(public_notes) = std::env::var("TACHYON_PUBLIC_NOTES_ENABLED") {
            config.guest.public_notes_enabled = public_notes == "1" || public_notes == "true";
        }

        if let Ok(guest_user_id) = std::env::var("TACHYON_GUEST_USER_ID") {
            config.guest.guest_user_id = guest_user_id;
        }

        if let Ok(rate_limit_enabled) = std::env::var("TACHYON_RATE_LIMIT_ENABLED") {
            config.rate_limit.enabled = rate_limit_enabled != "0" && rate_limit_enabled != "false";
        }

        if let Ok(site_title) = std::env::var("TACHYON_SITE_TITLE") {
            config.site.title = site_title;
        }

        if let Ok(site_description) = std::env::var("TACHYON_SITE_DESCRIPTION") {
            config.site.description = site_description;
        }

        if let Ok(base_url) = std::env::var("TACHYON_BASE_URL") {
            config.site.base_url = base_url;
        }

        if let Ok(template_dir) = std::env::var("TACHYON_TEMPLATE_DIR") {
            config.site.template_dir = Some(template_dir);
        }

        // Logging configuration
        if let Ok(log_format) = std::env::var("LOG_FORMAT") {
            config.log.format = log_format;
        }
        if let Ok(log_level) = std::env::var("TACHYON_LOG_LEVEL") {
            config.log.level = Some(log_level);
        }

        // OAuth2 configuration
        if let Ok(enabled) = std::env::var("TACHYON_OAUTH2_ENABLED") {
            config.oauth2.enabled = enabled == "1" || enabled == "true";
        }
        if let Ok(id) = std::env::var("TACHYON_GOOGLE_CLIENT_ID") {
            config.oauth2.google_client_id = Some(id);
        }
        if let Ok(secret) = std::env::var("TACHYON_GOOGLE_CLIENT_SECRET") {
            config.oauth2.google_client_secret = Some(secret);
        }
        if let Ok(id) = std::env::var("TACHYON_GITHUB_CLIENT_ID") {
            config.oauth2.github_client_id = Some(id);
        }
        if let Ok(secret) = std::env::var("TACHYON_GITHUB_CLIENT_SECRET") {
            config.oauth2.github_client_secret = Some(secret);
        }
        if let Ok(url) = std::env::var("TACHYON_OAUTH2_REDIRECT_BASE_URL") {
            config.oauth2.redirect_base_url = Some(url);
        }

        // TrueLayer configuration
        if let Ok(enabled) = std::env::var("TRUELAYER_ENABLED") {
            config.truelayer.enabled = enabled == "1" || enabled == "true";
        }
        if let Ok(id) = std::env::var("TRUELAYER_CLIENT_ID") {
            config.truelayer.client_id = id;
        }
        if let Ok(secret) = std::env::var("TRUELAYER_CLIENT_SECRET") {
            config.truelayer.client_secret = secret;
        }
        if let Ok(env) = std::env::var("TRUELAYER_ENV") {
            config.truelayer.environment = env;
        }
        if let Ok(id) = std::env::var("TRUELAYER_MERCHANT_ACCOUNT_ID") {
            config.truelayer.merchant_account_id = id;
        }
        if let Ok(secret) = std::env::var("TRUELAYER_WEBHOOK_SECRET") {
            config.truelayer.webhook_secret = secret;
        }

        // Security configuration
        if let Ok(val) = std::env::var("TACHYON_SMTP_URL") {
            config.smtp_url = Some(val);
        }
        if let Ok(val) = std::env::var("TACHYON_SMTP_FROM") {
            config.smtp_from = Some(val);
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_CSP_ENABLED") {
            config.security.csp_enabled = val != "0" && val != "false";
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_CSP_CUSTOM") {
            config.security.csp_custom = Some(val);
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_PERMISSIONS_POLICY") {
            config.security.permissions_policy = val != "0" && val != "false";
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_COEP_ENABLED") {
            config.security.coep_enabled = val == "1" || val == "true";
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_HSTS_ENABLED") {
            config.security.hsts_enabled = val != "0" && val != "false";
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_DEVELOPMENT") {
            config.security.development = val != "0" && val != "false";
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_FRAME_ANCESTORS") {
            config.security.frame_ancestors = val;
        }
        if let Ok(val) = std::env::var("TACHYON_SECURITY_TRUSTED_ORIGINS") {
            config.security.trusted_origins = val
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_bind_address() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ..Default::default()
        };
        assert_eq!(config.bind_address(), "127.0.0.1:3000");
    }

    #[test]
    fn test_config_validation_valid() {
        let mut config = ServerConfig::default();
        config.jwt.secret = "a-properly-long-secret-key-for-testing".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_host() {
        let mut config = ServerConfig::default();
        config.jwt.secret = "a-properly-long-secret-key-for-testing".to_string();
        config.host = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_tls_missing_cert() {
        let config = ServerConfig {
            enable_tls: true,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_jwt_expiration_duration() {
        let config = ServerConfig::default();
        let duration = config.jwt_expiration();
        assert_eq!(duration, Duration::from_secs(24 * 60 * 60));
    }

    #[test]
    fn test_api_key_header() {
        let config = ServerConfig::default();
        assert_eq!(config.api_key_header(), "X-API-Key");
    }

    #[test]
    fn test_websocket_path() {
        let config = ServerConfig::default();
        assert_eq!(config.websocket_path(), "/ws");
    }

    #[test]
    fn test_static_dir_default() {
        std::env::remove_var("TACHYON_STATIC_DIR");
        assert_eq!(static_dir(), "dist");
    }

    #[test]
    fn test_static_dir_from_env() {
        std::env::set_var("TACHYON_STATIC_DIR", "/var/www/html");
        assert_eq!(static_dir(), "/var/www/html");
        std::env::remove_var("TACHYON_STATIC_DIR");
    }
}
