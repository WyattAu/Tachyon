// Types and state for user routes

use crate::config::GuestConfig;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tachyon_core::{User, UserRole};
use tachyon_database::{DatabasePool, RefreshTokenRepository, UserRepository};
#[allow(unused_imports)]
use utoipa::IntoParams;

// ============================================================================
// JWT Claims
// ============================================================================

/// JWT claims structure for token validation.
///
/// This is compatible with the middleware Claims in `middleware/auth.rs`:
/// both include `permissions` (defaults to empty vec) and `team_id` (defaults to None).
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// Subject (user ID)
    pub(crate) sub: String,
    /// Issuer
    pub(crate) iss: String,
    /// Audience
    pub(crate) aud: String,
    /// Expiration time
    pub(crate) exp: usize,
    /// Issued at
    pub(crate) iat: usize,
    /// User role
    pub(crate) role: String,
    /// Granted permissions (empty for role-based auth)
    #[serde(default)]
    pub(crate) permissions: Vec<String>,
    /// Team ID (for team-scoped tokens)
    #[serde(default)]
    pub(crate) team_id: Option<String>,
}

// ============================================================================
// Application State
// ============================================================================

/// Application state for user routes.
///
/// Holds the database pool and JWT configuration. The `UserRepository`
/// is constructed on-demand from the pool.
#[derive(Clone)]
pub struct UserState {
    /// Database pool
    pub pool: DatabasePool,
    /// JWT secrets for token signing and validation (enables rotation)
    pub jwt_secrets: Vec<String>,
    /// Token expiration in seconds
    pub token_expiration_secs: u64,
    /// JWT issuer
    pub jwt_issuer: String,
    /// JWT audience
    pub jwt_audience: String,
    /// Guest configuration
    pub guest_config: GuestConfig,
}

pub(crate) const REFRESH_TOKEN_EXPIRATION_SECS: u64 = 30 * 24 * 60 * 60;

impl UserState {
    /// Create a new user state with full JWT config and guest config.
    pub fn with_guest_config(
        pool: DatabasePool,
        jwt_secrets: Vec<String>,
        token_expiration_secs: u64,
        jwt_issuer: String,
        jwt_audience: String,
        guest_config: GuestConfig,
    ) -> Self {
        Self {
            pool,
            jwt_secrets,
            token_expiration_secs,
            jwt_issuer,
            jwt_audience,
            guest_config,
        }
    }

    /// Get a UserRepository for this state.
    pub(crate) fn user_repo(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    /// Get a RefreshTokenRepository for this state.
    pub(crate) fn refresh_token_repo(&self) -> RefreshTokenRepository {
        RefreshTokenRepository::new(self.pool.clone())
    }

    pub(crate) fn generate_refresh_token(&self) -> String {
        let bytes: [u8; 32] = rand::thread_rng().gen();
        hex::encode(bytes)
    }

    /// Generate a JWT token for a user.
    ///
    /// The token includes `permissions` and `team_id` fields (defaulting to
    /// empty/None) to maintain compatibility with the auth middleware Claims.
    pub(crate) fn generate_jwt(&self, user_id: &str, role: UserRole) -> Result<String, String> {
        let now = jsonwebtoken::get_current_timestamp();
        let exp = now + self.token_expiration_secs;

        let claims = Claims {
            sub: user_id.to_string(),
            iss: self.jwt_issuer.clone(),
            aud: self.jwt_audience.clone(),
            exp: exp as usize,
            iat: now as usize,
            role: role.to_string(),
            permissions: vec![],
            team_id: None,
        };

        let signing_secret = self
            .jwt_secrets
            .first()
            .map(|s| s.as_str())
            .ok_or_else(|| "JWT secret not configured".to_string())?;

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(signing_secret.as_ref()),
        )
        .map_err(|e| format!("JWT encoding error: {}", e))
    }

    /// Validate a JWT token and return the claims.
    pub(crate) fn validate_jwt(&self, token: &str) -> Result<Claims, String> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.jwt_issuer]);
        validation.set_audience(&[&self.jwt_audience]);

        for secret in &self.jwt_secrets {
            if let Ok(data) = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &validation,
            ) {
                return Ok(data.claims);
            }
        }

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(
                self.jwt_secrets
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("")
                    .as_ref(),
            ),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| format!("JWT validation error: {}", e))
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to register a new user.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// Username (3-50 chars, alphanumeric/underscore/hyphen)
    pub username: String,
    /// Display name (1-100 chars)
    pub display_name: String,
    /// Email address (optional)
    pub email: Option<String>,
    /// Password (8-128 chars)
    pub password: String,
}

/// Request to create a new user (admin-only, can set role).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    /// Username
    pub username: String,
    /// Display name
    pub display_name: String,
    /// User email
    pub email: Option<String>,
    /// User password
    pub password: String,
    /// User role (admin-only)
    #[serde(default)]
    pub role: Option<String>,
}

/// Request to update a user.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateUserRequest {
    /// Display name
    pub display_name: Option<String>,
    /// User email
    pub email: Option<String>,
    /// User role (admin-only)
    pub role: Option<String>,
    /// Active status (admin-only)
    pub is_active: Option<bool>,
}

/// Request to update the current user's own profile.
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    /// Display name
    pub display_name: Option<String>,
    /// Email address
    pub email: Option<String>,
}

/// Request to change password.
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    /// Current password
    pub current_password: String,
    /// New password
    pub new_password: String,
}

/// Request for user authentication.
#[derive(Debug, Deserialize)]
pub struct AuthenticateRequest {
    /// Username or email
    pub username: String,
    /// User password
    pub password: String,
}

/// Request to refresh an access token.
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// Request to logout (revoke refresh token).
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// Refresh token to revoke
    pub refresh_token: Option<String>,
}

/// Authentication response.
#[derive(Debug, Serialize)]
pub struct AuthenticateResponse {
    /// Success flag
    pub success: bool,
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Access token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Refresh token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token type
    pub token_type: String,
    /// Token expires in (seconds)
    pub expires_in: u64,
    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// User info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResponse>,
    /// Whether MFA is required to complete login
    #[serde(default)]
    pub mfa_required: bool,
    /// User ID for MFA completion (set when mfa_required is true)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_user_id: Option<String>,
}

/// User response (never includes password_hash).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Display name
    pub display_name: String,
    /// Email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User type
    pub user_type: String,
    /// Role
    pub role: String,
    /// Is active
    pub is_active: bool,
    /// Created at
    pub created_at: String,
    /// Updated at
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            display_name: user.display_name,
            email: user.email,
            user_type: user.user_type.to_string(),
            role: user.permissions.role.to_string(),
            is_active: user.is_active.unwrap_or(true),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

/// User list response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserListResponse {
    /// List of users
    pub users: Vec<UserResponse>,
    /// Total count
    pub total: i64,
    /// Page number
    pub page: usize,
    /// Page size
    pub page_size: usize,
}

/// Query parameters for user listing.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct UserQuery {
    /// Page number
    pub page: Option<usize>,
    /// Page size
    pub page_size: Option<usize>,
    /// Role filter
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserErrorResponse {
    pub code: String,
    pub message: String,
}

// ============================================================================
// Helpers
// ============================================================================

pub(crate) fn hash_refresh_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    hex::encode(hash)
}
