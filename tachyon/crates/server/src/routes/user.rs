// User API routes
// Handles user CRUD operations and authentication
//
// All user operations are persisted to PostgreSQL via UserRepository.
// The pluggable AuthProvider trait allows swapping authentication strategies
// (local password, OAuth, API keys) without changing route handlers.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tachyon_core::{User, UserId, UserRole};
use tachyon_database::{DatabasePool, UserRepository};
use tracing::{debug, info, instrument, warn};
use crate::config::GuestConfig;

// ============================================================================
// JWT Claims
// ============================================================================

/// JWT claims structure for token validation.
///
/// This is compatible with the middleware Claims in `middleware/auth.rs`:
/// both include `permissions` (defaults to empty vec) and `team_id` (defaults to None).
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject (user ID)
    sub: String,
    /// Issuer
    iss: String,
    /// Audience
    aud: String,
    /// Expiration time
    exp: usize,
    /// Issued at
    iat: usize,
    /// User role
    role: String,
    /// Granted permissions (empty for role-based auth)
    #[serde(default)]
    permissions: Vec<String>,
    /// Team ID (for team-scoped tokens)
    #[serde(default)]
    team_id: Option<String>,
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
    /// JWT secret for token signing
    pub jwt_secret: String,
    /// Token expiration in seconds
    pub token_expiration_secs: u64,
    /// JWT issuer
    pub jwt_issuer: String,
    /// JWT audience
    pub jwt_audience: String,
    /// Guest configuration
    pub guest_config: GuestConfig,
}

impl UserState {
    /// Create a new user state with full JWT config and guest config.
    pub fn with_guest_config(
        pool: DatabasePool,
        jwt_secret: String,
        token_expiration_secs: u64,
        jwt_issuer: String,
        jwt_audience: String,
        guest_config: GuestConfig,
    ) -> Self {
        Self {
            pool,
            jwt_secret,
            token_expiration_secs,
            jwt_issuer,
            jwt_audience,
            guest_config,
        }
    }

    /// Get a UserRepository for this state.
    fn user_repo(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    /// Generate a JWT token for a user.
    ///
    /// The token includes `permissions` and `team_id` fields (defaulting to
    /// empty/None) to maintain compatibility with the auth middleware Claims.
    fn generate_jwt(&self, user_id: &str, role: UserRole) -> Result<String, String> {
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

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| format!("JWT encoding error: {}", e))
    }

    /// Validate a JWT token and return the claims.
    fn validate_jwt(&self, token: &str) -> Result<Claims, String> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.jwt_issuer]);
        validation.set_audience(&[&self.jwt_audience]);

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
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
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
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
}

/// User response (never includes password_hash).
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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
#[derive(Debug, Deserialize)]
pub struct UserQuery {
    /// Page number
    pub page: Option<usize>,
    /// Page size
    pub page_size: Option<usize>,
    /// Role filter
    pub role: Option<String>,
}

/// Error response.
#[derive(Debug, Serialize)]
pub struct UserErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// Register a new user (public endpoint).
///
/// Creates a user with the default `Reader` role. The password is hashed
/// with Argon2id before storage.
#[instrument(skip(state), fields(username = %req.username))]
pub async fn register(
    State(state): State<UserState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("User registration: {}", req.username);

    // Validate input
    if req.username.len() < 3 || req.username.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Username must be between 3 and 50 characters".into(),
            }),
        ));
    }

    if req.display_name.is_empty() || req.display_name.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Display name must be between 1 and 100 characters".into(),
            }),
        ));
    }

    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Password must be at least 8 characters".into(),
            }),
        ));
    }

    if let Some(ref email) = req.email {
        if !email.contains('@') || !email.contains('.') {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(UserErrorResponse {
                    code: "VALIDATION_ERROR".into(),
                    message: "Invalid email format".into(),
                }),
            ));
        }
    }

    // Build user with hashed password
    let user_id = tachyon_core::generate_user_id();
    let mut user = User::new(user_id, req.username, req.display_name, UserRole::Reader);
    if let Some(email) = req.email {
        user = user.with_email(email);
    }
    if let Err(e) = user.set_password(&req.password) {
        warn!("Failed to hash password during registration: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserErrorResponse {
                code: "PASSWORD_ERROR".into(),
                message: "Failed to process password".into(),
            }),
        ));
    }

    // Persist
    let repo = state.user_repo();
    match repo.create(&user).await {
        Ok(created) => {
            info!("User registered: {} ({})", created.username, created.id);
            Ok(Json(UserResponse::from(created)))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") || msg.contains("unique") {
                Err((
                    StatusCode::CONFLICT,
                    Json(UserErrorResponse {
                        code: "CONFLICT".into(),
                        message: "Username or email already exists".into(),
                    }),
                ))
            } else {
                warn!("User registration failed: {}", msg);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UserErrorResponse {
                        code: "INTERNAL_ERROR".into(),
                        message: "Failed to create user".into(),
                    }),
                ))
            }
        }
    }
}

/// Create a new user (admin-only, can set role).
///
/// Unlike `/auth/register`, this endpoint allows setting an arbitrary role
/// and is intended for admin use.
pub async fn create_user(
    State(state): State<UserState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("Admin creating user: {}", req.username);

    // Validate input
    if req.username.len() < 3 || req.username.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Username must be between 3 and 50 characters".into(),
            }),
        ));
    }

    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Password must be at least 8 characters".into(),
            }),
        ));
    }

    let role = match req.role.as_deref() {
        Some("admin") => UserRole::Admin,
        Some("editor") => UserRole::Editor,
        Some("writer") => UserRole::Writer,
        _ => UserRole::Reader,
    };

    let user_id = tachyon_core::generate_user_id();
    let mut user = User::new(user_id, req.username, req.display_name, role);
    if let Some(email) = req.email {
        user = user.with_email(email);
    }
    if let Err(e) = user.set_password(&req.password) {
        warn!("Failed to hash password: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserErrorResponse {
                code: "PASSWORD_ERROR".into(),
                message: "Failed to process password".into(),
            }),
        ));
    }

    let repo = state.user_repo();
    match repo.create(&user).await {
        Ok(created) => Ok(Json(UserResponse::from(created))),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") || msg.contains("unique") {
                Err((
                    StatusCode::CONFLICT,
                    Json(UserErrorResponse {
                        code: "CONFLICT".into(),
                        message: "Username or email already exists".into(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UserErrorResponse {
                        code: "INTERNAL_ERROR".into(),
                        message: "Failed to create user".into(),
                    }),
                ))
            }
        }
    }
}

/// Get a user by ID.
pub async fn get_user(
    Path(user_id): Path<String>,
    State(state): State<UserState>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    let id = UserId::parse_str(&user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "INVALID_ID".into(),
                message: format!("Invalid user ID: {}", e),
            }),
        )
    })?;

    let repo = state.user_repo();
    match repo.get_by_id(&id).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(UserErrorResponse {
                code: "NOT_FOUND".into(),
                message: format!("User {} not found", user_id),
            }),
        )),
    }
}

/// Update a user.
pub async fn update_user(
    Path(user_id): Path<String>,
    State(state): State<UserState>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    let id = UserId::parse_str(&user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "INVALID_ID".into(),
                message: format!("Invalid user ID: {}", e),
            }),
        )
    })?;

    let role = req.role.as_deref().and_then(|r| match r {
        "admin" => Some(UserRole::Admin),
        "editor" => Some(UserRole::Editor),
        "writer" => Some(UserRole::Writer),
        "reader" => Some(UserRole::Reader),
        _ => None,
    });

    let repo = state.user_repo();
    match repo.update(&id, req.display_name.as_deref(), req.email.as_deref(), role, req.is_active).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(UserErrorResponse {
                code: "NOT_FOUND".into(),
                message: format!("User {} not found", user_id),
            }),
        )),
    }
}

/// Delete a user (soft-delete: sets is_active = false).
pub async fn delete_user(
    Path(user_id): Path<String>,
    State(state): State<UserState>,
) -> Result<StatusCode, (StatusCode, Json<UserErrorResponse>)> {
    let id = UserId::parse_str(&user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "INVALID_ID".into(),
                message: format!("Invalid user ID: {}", e),
            }),
        )
    })?;

    let repo = state.user_repo();
    match repo.deactivate(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(UserErrorResponse {
                code: "NOT_FOUND".into(),
                message: format!("User {} not found", user_id),
            }),
        )),
    }
}

/// List all users with pagination.
pub async fn list_users(
    Query(query): Query<UserQuery>,
    State(state): State<UserState>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<UserErrorResponse>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let repo = state.user_repo();
    match repo.list(page, page_size, query.role.as_deref()).await {
        Ok((users, total)) => {
            let user_responses = users.into_iter().map(UserResponse::from).collect();
            Ok(Json(UserListResponse {
                users: user_responses,
                total,
                page,
                page_size,
            }))
        }
        Err(e) => {
            warn!("Failed to list users: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "INTERNAL_ERROR".into(),
                    message: "Failed to list users".into(),
                }),
            ))
        }
    }
}

/// GET /auth/me — return the current user's profile from the JWT token.
///
/// This endpoint sits under `/auth/` so the middleware bypasses it.
/// We manually extract and validate the JWT from the Authorization header.
pub async fn get_me(
    State(state): State<UserState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "UNAUTHORIZED".into(),
                    message: "Missing or invalid Authorization header".into(),
                }),
            ));
        }
    };

    let claims = match state.validate_jwt(token) {
        Ok(c) => c,
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "UNAUTHORIZED".into(),
                    message: format!("Invalid token: {}", e),
                }),
            ));
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "UNAUTHORIZED".into(),
                    message: "Invalid user ID in token".into(),
                }),
            ));
        }
    };

    let uid = tachyon_core::UserId::from_uuid(user_id);
    let repo = state.user_repo();
    match repo.get_by_id(&uid).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(UserErrorResponse {
                code: "NOT_FOUND".into(),
                message: "User not found".into(),
            }),
        )),
    }
}

/// PUT /auth/me — update the current user's profile.
pub async fn update_me(
    State(state): State<UserState>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "UNAUTHORIZED".into(),
                    message: "Missing or invalid Authorization header".into(),
                }),
            ));
        }
    };

    let claims = match state.validate_jwt(token) {
        Ok(c) => c,
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "UNAUTHORIZED".into(),
                    message: format!("Invalid token: {}", e),
                }),
            ));
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "UNAUTHORIZED".into(),
                    message: "Invalid user ID in token".into(),
                }),
            ));
        }
    };

    let uid = tachyon_core::UserId::from_uuid(user_id);
    let repo = state.user_repo();
    match repo
        .update(&uid, req.display_name.as_deref(), req.email.as_deref(), None, None)
        .await
    {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                Err((
                    StatusCode::CONFLICT,
                    Json(UserErrorResponse {
                        code: "CONFLICT".into(),
                        message: "Email already in use".into(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(UserErrorResponse {
                        code: "NOT_FOUND".into(),
                        message: "User not found".into(),
                    }),
                ))
            }
        }
    }
}

/// Authenticate user (login).
///
/// Looks up the user by username or email in the database, verifies the
/// password with Argon2id, and returns a JWT token on success.
pub async fn authenticate(
    State(state): State<UserState>,
    Json(req): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("Authentication request: {}", req.username);

    if req.username.is_empty() {
        return Ok(Json(AuthenticateResponse {
            success: false,
            user_id: None,
            access_token: None,
            token_type: "Bearer".into(),
            expires_in: state.token_expiration_secs,
            error: Some("Username cannot be empty".into()),
            user: None,
        }));
    }

    if req.password.is_empty() {
        return Ok(Json(AuthenticateResponse {
            success: false,
            user_id: None,
            access_token: None,
            token_type: "Bearer".into(),
            expires_in: state.token_expiration_secs,
            error: Some("Password cannot be empty".into()),
            user: None,
        }));
    }

    let repo = state.user_repo();

    // Look up user by username or email
    let user = match repo.find_by_username_or_email(&req.username).await {
        Ok(u) => u,
        Err(_) => {
            // Use a generic message to prevent username enumeration
            debug!("Authentication failed: user not found");
            return Ok(Json(AuthenticateResponse {
                success: false,
                user_id: None,
                access_token: None,
                token_type: "Bearer".into(),
                expires_in: state.token_expiration_secs,
                error: Some("Invalid username or password".into()),
                user: None,
            }));
        }
    };

    // Check if user is active
    if !user.is_active.unwrap_or(true) {
        debug!("Authentication failed: user {} is inactive", user.username);
        return Ok(Json(AuthenticateResponse {
            success: false,
            user_id: None,
            access_token: None,
            token_type: "Bearer".into(),
            expires_in: state.token_expiration_secs,
            error: Some("Account is disabled".into()),
            user: None,
        }));
    }

    // Verify password
    match user.verify(&req.password) {
        Ok(true) => {}
        Ok(false) | Err(_) => {
            debug!("Authentication failed: invalid password for user {}", user.username);
            return Ok(Json(AuthenticateResponse {
                success: false,
                user_id: None,
                access_token: None,
                token_type: "Bearer".into(),
                expires_in: state.token_expiration_secs,
                error: Some("Invalid username or password".into()),
                user: None,
            }));
        }
    }

    // Generate JWT
    let token = match state.generate_jwt(&user.id.to_string(), user.permissions.role) {
        Ok(t) => t,
        Err(e) => {
            warn!("Failed to generate JWT: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "TOKEN_ERROR".into(),
                    message: "Failed to generate authentication token".into(),
                }),
            ));
        }
    };

    info!("Authentication successful: {} ({})", user.username, user.id);

    Ok(Json(AuthenticateResponse {
        success: true,
        user_id: Some(user.id.to_string()),
        access_token: Some(token),
        token_type: "Bearer".into(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user)),
    }))
}

/// Check authentication status.
pub async fn auth_status(
    State(state): State<UserState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth_str) if auth_str.starts_with("Bearer ") => {
            let token = &auth_str[7..];
            match state.validate_jwt(token) {
                Ok(claims) => {
                    info!("Auth status check: user {} authenticated", claims.sub);
                    Ok(Json(serde_json::json!({
                        "authenticated": true,
                        "user": {
                            "id": claims.sub,
                            "role": claims.role
                        }
                    })))
                }
                Err(e) => {
                    debug!("Auth status check: token validation failed: {}", e);
                    Ok(Json(serde_json::json!({
                        "authenticated": false,
                        "message": "Invalid or expired token"
                    })))
                }
            }
        }
        _ => {
            debug!("Auth status check: no valid Authorization header");
            Ok(Json(serde_json::json!({
                "authenticated": false,
                "message": "No valid session"
            })))
        }
    }
}

/// Logout user.
///
/// Currently a no-op since JWTs are stateless. When session persistence
/// is wired, this will revoke the session in the database.
pub async fn logout(
    State(_state): State<UserState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    info!("User logout");
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// Guest login — auto-authenticate as a guest user.
///
/// Only available when `guest_login_enabled` is true in config.
pub async fn guest_login(
    State(state): State<UserState>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("Guest login request");

    if !state.guest_config.guest_login_enabled {
        warn!("Guest login attempted but guest login is disabled");
        return Err((
            StatusCode::FORBIDDEN,
            Json(UserErrorResponse {
                code: "GUEST_LOGIN_DISABLED".into(),
                message: "Guest login is not enabled".into(),
            }),
        ));
    }

    let guest_user_id = if state.guest_config.guest_user_id.is_empty() {
        "00000000-0000-0000-0000-000000000099".to_string()
    } else {
        state.guest_config.guest_user_id.clone()
    };

    let user_id = match UserId::parse_str(&guest_user_id) {
        Ok(id) => id,
        Err(_) => tachyon_core::generate_user_id(),
    };

    let user = User::new(
        user_id.clone(),
        "guest".to_string(),
        "Guest User".to_string(),
        UserRole::Reader,
    )
    .with_email("guest@tachyon.local".to_string());

    let token = match state.generate_jwt(&user_id.to_string(), UserRole::Reader) {
        Ok(t) => t,
        Err(e) => {
            warn!("Failed to generate guest JWT: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "TOKEN_ERROR".into(),
                    message: "Failed to generate authentication token".into(),
                }),
            ));
        }
    };

    info!("Guest login successful: {}", user_id);
    Ok(Json(AuthenticateResponse {
        success: true,
        user_id: Some(user_id.to_string()),
        access_token: Some(token),
        token_type: "Bearer".into(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user)),
    }))
}

/// Get guest configuration status (public endpoint).
pub async fn guest_status(
    State(state): State<UserState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    Ok(Json(serde_json::json!({
        "guest_login_enabled": state.guest_config.guest_login_enabled,
        "public_notes_enabled": state.guest_config.public_notes_enabled,
    })))
}

// ============================================================================
// Router
// ============================================================================

/// Create the user router.
///
/// Routes:
/// - `POST /auth/register` — public registration (Reader role)
/// - `POST /auth/login` — authenticate (username/email + password)
/// - `POST /auth/guest` — guest login (when enabled)
/// - `GET  /auth/status` — check JWT validity
/// - `POST /auth/logout` — logout (no-op for JWT)
/// - `GET  /auth/me` — get current user profile
/// - `PUT  /auth/me` — update current user profile
/// - `GET  /auth/guest-status` — guest config (public)
/// - `GET  /users` — list users (paginated)
/// - `POST /users` — create user (admin, can set role)
/// - `GET  /users/{user_id}` — get user by ID
/// - `PUT  /users/{user_id}` — update user
/// - `DELETE /users/{user_id}` — deactivate user
pub fn create_user_router() -> axum::Router<UserState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        // Auth routes (public)
        .route("/auth/register", post(register))
        .route("/auth/login", post(authenticate))
        .route("/auth/guest", post(guest_login))
        .route("/auth/status", get(auth_status))
        .route("/auth/logout", post(logout))
        .route("/auth/guest-status", get(guest_status))
        .route("/auth/me", get(get_me))
        .route("/auth/me", put(update_me))
        // User management routes
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{user_id}", get(get_user))
        .route("/users/{user_id}", put(update_user))
        .route("/users/{user_id}", delete(delete_user))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_construction() {
        let req = RegisterRequest {
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            password: "password123".to_string(),
        };
        assert_eq!(req.username, "testuser");
        assert_eq!(req.display_name, "Test User");
    }

    #[test]
    fn test_authenticate_response_serialization() {
        let resp = AuthenticateResponse {
            success: true,
            user_id: Some("user-1".to_string()),
            access_token: Some("token-123".to_string()),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            error: None,
            user: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Bearer"));
        assert!(json.contains("token-123"));
    }

    #[test]
    fn test_user_response_from_user() {
        let user_id = tachyon_core::generate_user_id();
        let user = User::new(
            user_id,
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::Writer,
        );

        let response = UserResponse::from(user);
        assert_eq!(response.username, "testuser");
        assert_eq!(response.role, "writer");
        assert!(response.is_active);
    }

    #[test]
    fn test_user_list_response_serialization() {
        let resp = UserListResponse {
            users: vec![],
            total: 0,
            page: 1,
            page_size: 20,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_update_profile_request_construction() {
        let req = UpdateProfileRequest {
            display_name: Some("New Name".to_string()),
            email: None,
        };
        assert_eq!(req.display_name.as_deref(), Some("New Name"));
        assert!(req.email.is_none());
    }
}
