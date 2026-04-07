// Authentication middleware
// Handles JWT token validation, API key authentication, and RBAC

use crate::config::ServerConfig;
use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use tachyon_core::{UserAction, UserRole};
use tachyon_database::{DatabasePool, Permission};
use tracing::{debug, warn};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct Claims {
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

#[derive(Debug)]
pub enum AuthError {
    MissingAuthHeader,
    InvalidTokenFormat,
    TokenExpired,
    InvalidSignature,
    InvalidApiKey,
    UserNotFound,
    InsufficientPermissions,
    InternalError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingAuthHeader => write!(f, "Missing authentication header"),
            Self::InvalidTokenFormat => write!(f, "Invalid token format"),
            Self::TokenExpired => write!(f, "Token expired"),
            Self::InvalidSignature => write!(f, "Invalid token signature"),
            Self::InvalidApiKey => write!(f, "Invalid API key"),
            Self::UserNotFound => write!(f, "User not found"),
            Self::InsufficientPermissions => write!(f, "Insufficient permissions"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

#[derive(Clone)]
pub struct AuthState {
    config: Arc<ServerConfig>,
    pool: DatabasePool,
}

impl AuthState {
    pub fn new(config: ServerConfig, pool: DatabasePool) -> Self {
        Self {
            config: Arc::new(config),
            pool,
        }
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

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt.secret.as_ref()),
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

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt.secret.as_ref()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => AuthError::InvalidSignature,
            _ => AuthError::InvalidTokenFormat,
        })?;

        Ok(token_data.claims)
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<(String, UserRole), AuthError> {
        if api_key.len() < 12 {
            return Err(AuthError::InvalidApiKey);
        }

        let prefix = &api_key[..12];
        let hash = Sha256::digest(api_key.as_bytes());
        let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

        let mut conn = self.pool.acquire().await
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

    pub async fn extract_auth_context(&self, headers: &HeaderMap) -> Result<AuthContext, AuthError> {
        if let Some(auth_header) = headers.get("authorization") {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| AuthError::InvalidTokenFormat)?;

            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
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
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = request.headers();
    let path = request.uri().path();

    if path == "/health" || path == "/metrics" || path.starts_with("/api/v1/auth/") {
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
                AuthError::MissingAuthHeader | AuthError::InvalidTokenFormat => {
                    StatusCode::UNAUTHORIZED
                }
                AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
                AuthError::InvalidSignature | AuthError::InvalidApiKey => StatusCode::FORBIDDEN,
                AuthError::UserNotFound => StatusCode::NOT_FOUND,
                AuthError::InsufficientPermissions => StatusCode::FORBIDDEN,
                AuthError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };

            let body = serde_json::json!({
                "error": e.to_string(),
            });

            Err((status, Json(body)))
        }
    }
}

pub async fn require_permission_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(auth_context) = request.extensions().get::<AuthContext>() {
        if auth_context.role == UserRole::Admin {
            return Ok(next.run(request).await);
        }
        
        if auth_context.has_permission(Permission::Admin) {
            return Ok(next.run(request).await);
        }
        
        Err(StatusCode::FORBIDDEN)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn require_permission(permission: Permission) -> impl Fn(&AuthContext) -> bool {
    move |auth_context: &AuthContext| {
        auth_context.has_permission(permission)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires database pool"]
    async fn test_jwt_generation() {
        let config = ServerConfig::default();
        let pool = DatabasePool::new("postgres://localhost:5432/tachyon_test").await.unwrap();
        let state = AuthState::new(config, pool);

        let user_id = "test-user-id";
        let role = UserRole::Admin;

        let token = state.generate_jwt(user_id, role);
        assert!(token.is_ok());

        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires database pool"]
    async fn test_jwt_validation() {
        let config = ServerConfig::default();
        let pool = DatabasePool::new("postgres://localhost:5432/tachyon_test").await.unwrap();
        let state = AuthState::new(config, pool);

        let user_id = "test-user-id";
        let role = UserRole::Writer;

        let token = state.generate_jwt(user_id, role).unwrap();
        let claims = state.validate_jwt(&token);

        assert!(claims.is_ok());
        let validated_claims = claims.unwrap();
        assert_eq!(validated_claims.sub, user_id);
        assert_eq!(validated_claims.role, "writer");
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
            "Missing authentication header"
        );
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
    }
}
