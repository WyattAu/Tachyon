// Authentication middleware
// Handles JWT token validation, API key authentication, and RBAC

use crate::config::ServerConfig;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tachyon_core::{UserAction, UserRole};
use tachyon_database::{DatabasePool, Permission};
use tachyon_rbac::types::{
    AccessRequest, Action as RbacAction, Resource as RbacResource, Subject as RbacSubject,
};
use tachyon_rbac::AuthContext as RbacAuthContext;
use tachyon_rbac::{Enforcer, EnforcerConfig as RbacEnforcerConfig};
use tachyon_rbac::{SessionId, UserId};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
    role: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    team_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub team_id: Option<String>,
    pub auth_method: AuthMethod,
}

impl AuthContext {
    pub fn has_permission(&self, permission: Permission) -> bool {
        if self.role == UserRole::Admin {
            return true;
        }

        self.permissions.iter().any(|p| {
            if let Some(perm) = Permission::from_str(p) {
                perm.includes(&permission)
            } else {
                false
            }
        })
    }

    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.has_permission(*p))
    }

    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.has_permission(*p))
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin || self.permissions.contains(&"admin".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMethod {
    Jwt,
    ApiKey,
    Bearer,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthHeader,
    #[error("Invalid token format")]
    InvalidTokenFormat,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("User not found")]
    UserNotFound,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Clone)]
pub struct AuthState {
    config: Arc<ServerConfig>,
    pool: DatabasePool,
    enforcer: Arc<RwLock<Enforcer>>,
    secret_usage: Arc<SecretUsageTracker>,
}

#[derive(Debug, Default)]
pub struct SecretUsageTracker {
    counters: Vec<AtomicU64>,
}

impl SecretUsageTracker {
    pub fn new(count: usize) -> Self {
        Self {
            counters: (0..count).map(|_| AtomicU64::new(0)).collect(),
        }
    }

    pub fn record(&self, index: usize) {
        if let Some(counter) = self.counters.get(index) {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn snapshot(&self) -> Vec<(usize, u64)> {
        self.counters
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.load(Ordering::Relaxed)))
            .collect()
    }
}

impl AuthState {
    pub fn new(config: ServerConfig, pool: DatabasePool) -> Self {
        let enforcer = Enforcer::with_config(RbacEnforcerConfig::default());
        let secret_usage = SecretUsageTracker::new(config.jwt.secrets.len());
        Self {
            config: Arc::new(config),
            pool,
            enforcer: Arc::new(RwLock::new(enforcer)),
            secret_usage: Arc::new(secret_usage),
        }
    }

    pub fn secret_usage_snapshot(&self) -> Vec<(usize, u64)> {
        self.secret_usage.snapshot()
    }

    pub fn generate_jwt(&self, user_id: &str, role: UserRole) -> Result<String, AuthError> {
        self.generate_jwt_with_permissions(user_id, role, vec![], None)
    }

    pub fn generate_jwt_with_permissions(
        &self,
        user_id: &str,
        role: UserRole,
        permissions: Vec<String>,
        team_id: Option<String>,
    ) -> Result<String, AuthError> {
        let now = jsonwebtoken::get_current_timestamp();
        let exp = now + self.config.jwt.expiration_secs;

        let claims = Claims {
            sub: user_id.to_string(),
            iss: self.config.jwt.issuer.clone(),
            aud: self.config.jwt.audience.clone(),
            exp: exp as usize,
            iat: now as usize,
            role: role.to_string(),
            permissions,
            team_id,
        };

        let kid = secret_kid(self.config.jwt.signing_secret(), 0);
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some(kid);

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.config.jwt.signing_secret().as_bytes()),
        )
        .map_err(|e| AuthError::InternalError(format!("JWT encoding error: {}", e)))
    }

    #[allow(private_interfaces)]
    pub fn validate_jwt(&self, token: &str) -> Result<Claims, AuthError> {
        let issuer = self.config.jwt.issuer.as_str();
        let audience = self.config.jwt.audience.as_str();

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);

        for (index, secret) in self.config.jwt.secrets.iter().enumerate() {
            match decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &validation,
            ) {
                Ok(token_data) => {
                    if index > 0 {
                        let kid_in_token = token_data.header.kid.as_deref().unwrap_or("none");
                        let current_kid = secret_kid(self.config.jwt.signing_secret(), 0);
                        if self.config.jwt.is_rotation_active() {
                            info!(
                                secret_index = index,
                                token_kid = %kid_in_token,
                                current_kid = %current_kid,
                                "JWT validated with non-primary secret — rotation in progress"
                            );
                        }
                        warn!(
                            secret_index = index,
                            "JWT validated with non-primary secret"
                        );
                    }
                    self.secret_usage.record(index);
                    return Ok(token_data.claims);
                }
                Err(_) => continue,
            }
        }

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt.signing_secret().as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => AuthError::InvalidSignature,
            _ => AuthError::InvalidTokenFormat,
        })
        .map(|td| td.claims)
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<(String, UserRole), AuthError> {
        if api_key.len() < 12 {
            return Err(AuthError::InvalidApiKey);
        }

        let prefix = &api_key[..12];
        let hash = Sha256::digest(api_key.as_bytes());
        let hash_hex = hex::encode(hash);

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AuthError::InternalError(format!("Database error: {}", e)))?;

        let row = sqlx::query(
            "SELECT user_id, expires_at FROM api_keys WHERE key_hash = $1 AND key_prefix = $2 AND is_active = true"
        )
        .bind(&hash_hex)
        .bind(prefix)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| AuthError::InternalError(format!("Database error: {}", e)))?
        .ok_or(AuthError::InvalidApiKey)?;

        let user_id: uuid::Uuid = row.get("user_id");
        let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");

        if let Some(exp) = expires_at {
            if exp < Utc::now() {
                return Err(AuthError::InvalidApiKey);
            }
        }

        let user_row = sqlx::query("SELECT role FROM users WHERE id = $1 AND is_active = true")
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| AuthError::InternalError(format!("Database error: {}", e)))?
            .ok_or(AuthError::UserNotFound)?;

        let role_str: String = user_row.get("role");
        let role = match role_str.as_str() {
            "admin" => UserRole::Admin,
            "editor" => UserRole::Editor,
            "writer" => UserRole::Writer,
            _ => UserRole::Reader,
        };

        let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE key_hash = $1")
            .bind(&hash_hex)
            .execute(&mut *conn)
            .await;

        Ok((user_id.to_string(), role))
    }

    pub async fn extract_auth_context(
        &self,
        headers: &HeaderMap,
    ) -> Result<AuthContext, AuthError> {
        if let Some(auth_header) = headers.get("authorization") {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| AuthError::InvalidTokenFormat)?;

            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let claims = self.validate_jwt(token)?;

                let user_id = claims.sub.clone();
                let role = match claims.role.as_str() {
                    "reader" => UserRole::Reader,
                    "writer" => UserRole::Writer,
                    "editor" => UserRole::Editor,
                    "admin" => UserRole::Admin,
                    _ => return Err(AuthError::InvalidTokenFormat),
                };

                return Ok(AuthContext {
                    user_id,
                    role,
                    permissions: claims.permissions,
                    team_id: claims.team_id,
                    auth_method: AuthMethod::Jwt,
                });
            }
        }

        if self.config.api_keys.enabled {
            if let Some(api_key_header) = headers.get(self.config.api_key_header()) {
                let api_key = api_key_header
                    .to_str()
                    .map_err(|_| AuthError::InvalidTokenFormat)?;

                let (user_id, role) = self.validate_api_key(api_key).await?;

                return Ok(AuthContext {
                    user_id,
                    role,
                    permissions: vec![],
                    team_id: None,
                    auth_method: AuthMethod::ApiKey,
                });
            }
        }

        Err(AuthError::MissingAuthHeader)
    }

    pub async fn check_rbac_permission(
        &self,
        auth_context: &AuthContext,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> bool {
        let subject = RbacSubject::from_role(&auth_context.role.to_string());

        let resource = RbacResource::new(resource_type, resource_id);

        let action = RbacAction::new(action);

        let user_id = UserId::parse_str(&auth_context.user_id).unwrap_or_else(|_| UserId::new());
        let session_id = SessionId::new();
        let rbac_context =
            RbacAuthContext::new(user_id, session_id).with_role(&auth_context.role.to_string());

        let request = AccessRequest::new(subject, resource, action, rbac_context);

        let mut enforcer = self.enforcer.write().await;
        match enforcer.authorize(&request) {
            Ok(decision) => decision.is_allowed(),
            Err(_) => auth_context.role == UserRole::Admin,
        }
    }

    pub fn enforcer(&self) -> &Arc<RwLock<Enforcer>> {
        &self.enforcer
    }
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = request.headers();
    let path = request.uri().path();

    if request.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    let is_public = path == "/api/v1/auth/login"
        || path == "/api/v1/auth/register"
        || path == "/api/v1/auth/guest"
        || path == "/api/v1/auth/refresh"
        || path == "/api/v1/auth/mfa/authenticate"
        || path.starts_with("/api/v1/auth/password-reset/request")
        || path.starts_with("/api/v1/auth/email-verification/request")
        || path == "/api/v1/billing/webhook"
        || path == "/api/health"
        || path == "/api/docs"
        || path.starts_with("/api/static/")
        || path == "/health"
        || path == "/metrics"
        || path == "/";

    if is_public {
        return Ok(next.run(request).await);
    }

    match state.extract_auth_context(headers).await {
        Ok(auth_context) => {
            debug!(
                user_id = %auth_context.user_id,
                method = ?auth_context.auth_method,
                "Authentication successful"
            );

            let mut request = request;
            request.extensions_mut().insert(auth_context);

            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!(error = %e, "Authentication failed");
            let status = match e {
                AuthError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::UNAUTHORIZED,
            };

            let body = serde_json::json!({
                "error": e.to_string(),
            });

            Err((status, Json(body)))
        }
    }
}

pub async fn require_permission_middleware(
    State(state): State<AuthState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(auth_context) = request.extensions().get::<AuthContext>().cloned() {
        if auth_context.role == UserRole::Admin {
            return Ok(next.run(request).await);
        }

        if state
            .check_rbac_permission(&auth_context, "global", "*", "admin")
            .await
        {
            return Ok(next.run(request).await);
        }

        Err(StatusCode::FORBIDDEN)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn require_permission(permission: Permission) -> impl Fn(&AuthContext) -> bool {
    move |auth_context: &AuthContext| auth_context.has_permission(permission)
}

pub fn check_permission(auth_context: &AuthContext, action: UserAction) -> bool {
    match auth_context.role {
        UserRole::Admin => true,
        _ => auth_context.role.can_perform(action),
    }
}

#[derive(Debug, Clone)]
pub struct PermissionGuard {
    pub required_permission: Permission,
}

impl PermissionGuard {
    pub fn new(permission: Permission) -> Self {
        Self {
            required_permission: permission,
        }
    }

    pub fn check(&self, auth_context: &AuthContext) -> bool {
        auth_context.has_permission(self.required_permission)
    }
}

fn secret_kid(secret: &str, index: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();
    let hex = hex::encode(&hash[..8]);
    format!("k{}-{}", index, hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_claims(user_id: &str, role: &str) -> Claims {
        let now = jsonwebtoken::get_current_timestamp();
        Claims {
            sub: user_id.to_string(),
            iss: "tachyon-test".to_string(),
            aud: "tachyon-test".to_string(),
            exp: (now + 86400) as usize,
            iat: now as usize,
            role: role.to_string(),
            permissions: vec![],
            team_id: None,
        }
    }

    fn encode_test_token(claims: &Claims, secret: &str) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .expect("token encoding should succeed")
    }

    fn encode_test_token_with_kid(claims: &Claims, secret: &str, kid: &str) -> String {
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some(kid.to_string());
        encode(&header, claims, &EncodingKey::from_secret(secret.as_ref()))
            .expect("token encoding should succeed")
    }

    fn decode_test_token(
        token: &str,
        secret: &str,
        issuer: &str,
        audience: &str,
    ) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => AuthError::InvalidSignature,
            _ => AuthError::InvalidTokenFormat,
        })
    }

    #[test]
    fn test_jwt_generation() {
        let secret = "test-secret-key-for-testing-only-long-enough";
        let issuer = "tachyon-test";
        let audience = "tachyon-test";
        let user_id = "test-user-123";

        let claims = make_test_claims(user_id, "admin");
        let token = encode_test_token(&claims, secret);

        assert!(!token.is_empty(), "token should not be empty");

        let decoded = decode_test_token(&token, secret, issuer, audience);
        assert!(decoded.is_ok(), "token should decode successfully");
        let decoded_claims = decoded.unwrap();
        assert_eq!(decoded_claims.sub, user_id);
        assert_eq!(decoded_claims.role, "admin");
        assert_eq!(decoded_claims.iss, issuer);
        assert_eq!(decoded_claims.aud, audience);
    }

    #[test]
    fn test_jwt_validation() {
        let secret = "test-secret-key-for-testing-only-long-enough";
        let issuer = "tachyon-test";
        let audience = "tachyon-test";
        let user_id = "test-user-456";

        let claims = make_test_claims(user_id, "writer");
        let token = encode_test_token(&claims, secret);

        assert!(
            decode_test_token(&token, secret, issuer, audience).is_ok(),
            "valid token should decode"
        );

        assert!(
            decode_test_token(&token, "wrong-secret", issuer, audience).is_err(),
            "wrong secret should fail"
        );

        assert!(
            decode_test_token(&token, secret, "wrong-issuer", audience).is_err(),
            "wrong issuer should fail"
        );

        assert!(
            decode_test_token(&token, secret, issuer, "wrong-audience").is_err(),
            "wrong audience should fail"
        );
    }

    #[test]
    fn test_auth_context_permissions() {
        let auth = AuthContext {
            user_id: "test".to_string(),
            role: UserRole::Writer,
            permissions: vec!["read".to_string(), "write".to_string()],
            team_id: None,
            auth_method: AuthMethod::Jwt,
        };

        assert!(auth.has_permission(Permission::Read));
        assert!(auth.has_permission(Permission::Write));
        assert!(!auth.has_permission(Permission::Delete));
    }

    #[test]
    fn test_auth_error_display() {
        assert_eq!(
            AuthError::MissingAuthHeader.to_string(),
            "Missing authorization header"
        );
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
    }

    #[test]
    fn test_jwt_validation_with_rotated_secret() {
        let issuer = "tachyon-test";
        let audience = "tachyon-test";
        let old_secret = "old-rotated-secret-that-is-at-least-32";
        let new_secret = "current-secret-that-is-at-least-32-bytes-long";

        let claims = make_test_claims("user123", "admin");

        let token = encode_test_token(&claims, old_secret);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);

        let secrets: Vec<&str> = vec![new_secret, old_secret];

        let mut result = None;
        for secret in &secrets {
            if let Ok(data) = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(secret.as_ref()),
                &validation,
            ) {
                result = Some(data.claims);
                break;
            }
        }

        assert!(
            result.is_some(),
            "Token signed with old secret should validate with rotated secrets"
        );
        let decoded = result.unwrap();
        assert_eq!(decoded.sub, "user123");
        assert_eq!(decoded.role, "admin");
    }

    #[test]
    fn test_jwt_signed_with_new_secret_fails_with_old_only() {
        let issuer = "tachyon-test";
        let audience = "tachyon-test";
        let old_secret = "old-rotated-secret-that-is-at-least-32";
        let new_secret = "current-secret-that-is-at-least-32-bytes-long";

        let claims = make_test_claims("user456", "writer");

        let token = encode_test_token(&claims, new_secret);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(old_secret.as_ref()),
            &validation,
        );

        assert!(
            result.is_err(),
            "Token signed with new secret should fail with old secret only"
        );
    }

    #[test]
    fn test_secret_kid_format() {
        let kid = secret_kid("test-secret-key-for-testing-only-long-enough", 0);
        assert!(kid.starts_with("k0-"));
        assert!(kid.len() > 5);
    }

    #[test]
    fn test_secret_kid_deterministic() {
        let kid1 = secret_kid("same-secret-key-for-testing-only-long-enough", 0);
        let kid2 = secret_kid("same-secret-key-for-testing-only-long-enough", 0);
        assert_eq!(kid1, kid2);
    }

    #[test]
    fn test_secret_kid_differs_per_index() {
        let kid0 = secret_kid("same-secret-key-for-testing-only-long-enough", 0);
        let kid1 = secret_kid("same-secret-key-for-testing-only-long-enough", 1);
        assert_ne!(kid0, kid1);
    }

    #[test]
    fn test_secret_usage_tracker() {
        let tracker = SecretUsageTracker::new(3);
        tracker.record(0);
        tracker.record(0);
        tracker.record(2);

        let snap = tracker.snapshot();
        assert_eq!(snap[0], (0, 2));
        assert_eq!(snap[1], (1, 0));
        assert_eq!(snap[2], (2, 1));
    }

    #[test]
    fn test_secret_usage_tracker_out_of_bounds() {
        let tracker = SecretUsageTracker::new(2);
        tracker.record(5);
        let snap = tracker.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0], (0, 0));
        assert_eq!(snap[1], (1, 0));
    }

    #[test]
    fn test_jwt_with_kid_header() {
        let secret = "test-secret-key-for-testing-only-long-enough";
        let claims = make_test_claims("user-kid-test", "admin");
        let kid = secret_kid(secret, 0);
        let token = encode_test_token_with_kid(&claims, secret, &kid);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["tachyon-test"]);
        validation.set_audience(&["tachyon-test"]);

        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        );
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().header.kid, Some(kid));
    }

    #[test]
    fn test_rotation_config_default() {
        let config = crate::config::JwtConfig::default();
        assert!(config.rotation_enabled);
        assert!(!config.is_rotation_active());
    }

    #[test]
    fn test_rotation_config_active_with_multiple_secrets() {
        let config = crate::config::JwtConfig {
            secrets: vec![
                "first-secret-that-is-at-least-32-characters".to_string(),
                "second-secret-that-is-at-least-32-characters".to_string(),
            ],
            expiration_secs: 3600,
            issuer: "test".to_string(),
            audience: "test".to_string(),
            rotation_enabled: true,
        };
        assert!(config.is_rotation_active());
    }

    #[test]
    fn test_rotation_config_disabled() {
        let config = crate::config::JwtConfig {
            secrets: vec![
                "first-secret-that-is-at-least-32-characters".to_string(),
                "second-secret-that-is-at-least-32-characters".to_string(),
            ],
            expiration_secs: 3600,
            issuer: "test".to_string(),
            audience: "test".to_string(),
            rotation_enabled: false,
        };
        assert!(!config.is_rotation_active());
    }
}
