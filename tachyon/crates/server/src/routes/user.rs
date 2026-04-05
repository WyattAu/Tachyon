// User API routes
// Handles user CRUD operations and authentication

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tachyon_core::{User, UserId, UserRole};
use tachyon_database::DatabasePool;
use tracing::{debug, info, warn};
use crate::config::GuestConfig;

/// JWT claims structure for token validation
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
}

/// Application state for user routes
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
    /// Create a new user state
    pub fn new(pool: DatabasePool, jwt_secret: String, token_expiration_secs: u64) -> Self {
        Self {
            pool,
            jwt_secret,
            token_expiration_secs,
            jwt_issuer: "tachyon-server".to_string(),
            jwt_audience: "tachyon-client".to_string(),
            guest_config: GuestConfig::default(),
        }
    }

    /// Create a new user state with full JWT config
    pub fn with_jwt_config(
        pool: DatabasePool,
        jwt_secret: String,
        token_expiration_secs: u64,
        jwt_issuer: String,
        jwt_audience: String,
    ) -> Self {
        Self {
            pool,
            jwt_secret,
            token_expiration_secs,
            jwt_issuer,
            jwt_audience,
            guest_config: GuestConfig::default(),
        }
    }

    /// Create a new user state with guest config
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

    /// Generate a JWT token for a user
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
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| format!("JWT encoding error: {}", e))
    }

    /// Validate a JWT token and return the claims
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

/// Request to create a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// User name
    pub username: String,
    /// Display name
    pub display_name: String,
    /// User email
    pub email: Option<String>,
    /// User password
    pub password: String,
    /// User role
    #[serde(default)]
    pub role: Option<String>,
}

/// Request to update a user
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    /// User name
    pub username: Option<String>,
    /// Display name
    pub display_name: Option<String>,
    /// User email
    pub email: Option<String>,
    /// User role
    pub role: Option<String>,
    /// Active status
    pub is_active: Option<bool>,
}

/// Request for user authentication
#[derive(Debug, Deserialize)]
pub struct AuthenticateRequest {
    /// Username or email
    pub username: String,
    /// User password
    pub password: String,
}

/// Authentication response
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

/// User response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Display name
    pub display_name: String,
    /// Email
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

/// User list response
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    /// List of users
    pub users: Vec<UserResponse>,
    /// Total count
    pub total: usize,
    /// Page number
    pub page: usize,
    /// Page size
    pub page_size: usize,
}

/// Query parameters for user listing
#[derive(Debug, Deserialize)]
pub struct UserQuery {
    /// Page number
    pub page: Option<usize>,
    /// Page size
    pub page_size: Option<usize>,
    /// Role filter
    pub role: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct UserErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// Create a new user
pub async fn create_user(
    State(_state): State<UserState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("Creating new user: {}", req.username);

    // Validate username
    if req.username.len() < 3 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Username must be at least 3 characters".to_string(),
            }),
        ));
    }

    if req.username.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Username cannot exceed 50 characters".to_string(),
            }),
        ));
    }

    // Validate password
    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Password must be at least 8 characters".to_string(),
            }),
        ));
    }

    // Parse role
    let role = match req.role.as_deref() {
        Some("admin") => UserRole::Admin,
        Some("editor") => UserRole::Editor,
        Some("writer") => UserRole::Writer,
        _ => UserRole::Reader,
    };

    // Create user
    let user_id = tachyon_core::generate_user_id();
    let mut user = User::new(user_id, req.username, req.display_name, role);

    // Set email if provided
    if let Some(email) = req.email {
        if !email.contains('@') {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(UserErrorResponse {
                    code: "VALIDATION_ERROR".to_string(),
                    message: "Invalid email format".to_string(),
                }),
            ));
        }
        user = user.with_email(email);
    }

    // Set password
    if let Err(e) = user.set_password(&req.password) {
        warn!("Failed to set password: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserErrorResponse {
                code: "PASSWORD_ERROR".to_string(),
                message: "Failed to set password".to_string(),
            }),
        ));
    }

    // Validate user
    if let Err(e) = user.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: e.to_string(),
            }),
        ));
    }

    info!("User created successfully: {}", user.username);

    Ok(Json(UserResponse::from(user)))
}

/// Get a user by ID
pub async fn get_user(
    Path(user_id): Path<String>,
    State(_state): State<UserState>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    debug!("Getting user: {}", user_id);

    // Parse user ID
    let _id = UserId::parse_str(&user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "INVALID_ID".to_string(),
                message: format!("Invalid user ID: {}", e),
            }),
        )
    })?;

    // TODO: Fetch from database
    Err((
        StatusCode::NOT_FOUND,
        Json(UserErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: format!("User {} not found", user_id),
        }),
    ))
}

/// Update a user
pub async fn update_user(
    Path(user_id): Path<String>,
    State(_state): State<UserState>,
    Json(_req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    debug!("Updating user: {}", user_id);

    // Parse user ID
    let _id = UserId::parse_str(&user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "INVALID_ID".to_string(),
                message: format!("Invalid user ID: {}", e),
            }),
        )
    })?;

    // TODO: Fetch from database, update, and save

    // Return placeholder response
    Err((
        StatusCode::NOT_FOUND,
        Json(UserErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: format!("User {} not found", user_id),
        }),
    ))
}

/// Delete a user
pub async fn delete_user(
    Path(user_id): Path<String>,
    State(_state): State<UserState>,
) -> Result<StatusCode, (StatusCode, Json<UserErrorResponse>)> {
    debug!("Deleting user: {}", user_id);

    // Parse user ID
    let _id = UserId::parse_str(&user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UserErrorResponse {
                code: "INVALID_ID".to_string(),
                message: format!("Invalid user ID: {}", e),
            }),
        )
    })?;

    // TODO: Delete from database

    Ok(StatusCode::NO_CONTENT)
}

/// List all users
pub async fn list_users(
    Query(query): Query<UserQuery>,
    State(_state): State<UserState>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<UserErrorResponse>)> {
    debug!(
        "Listing users (page: {:?}, size: {:?})",
        query.page, query.page_size
    );

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    // TODO: Fetch from database

    Ok(Json(UserListResponse {
        users: vec![],
        total: 0,
        page,
        page_size,
    }))
}

/// Authenticate user
pub async fn authenticate(
    State(state): State<UserState>,
    Json(req): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("User authentication request: {}", req.username);

    // Validate input
    if req.username.is_empty() {
        return Ok(Json(AuthenticateResponse {
            success: false,
            user_id: None,
            access_token: None,
            token_type: "Bearer".to_string(),
            expires_in: state.token_expiration_secs,
            error: Some("Username cannot be empty".to_string()),
            user: None,
        }));
    }

    if req.password.is_empty() {
        return Ok(Json(AuthenticateResponse {
            success: false,
            user_id: None,
            access_token: None,
            token_type: "Bearer".to_string(),
            expires_in: state.token_expiration_secs,
            error: Some("Password cannot be empty".to_string()),
            user: None,
        }));
    }

    // TODO: Fetch user from database and verify password
    // For now, support local-first mode with demo users

    // Define seed users for local-first / demo mode
    // These users are available without database setup
    let seed_users = [
        ("admin", "admin123", "Administrator", UserRole::Admin, "admin@tachyon.local"),
        ("guest", "guest", "Guest User", UserRole::Reader, "guest@tachyon.local"),
        ("editor", "editor123", "Editor User", UserRole::Editor, "editor@tachyon.local"),
    ];

    for (username, password, display_name, role, email) in seed_users {
        if req.username == username && req.password == password {
            let user_id = tachyon_core::generate_user_id();
            let user = User::new(
                user_id.clone(),
                username.to_string(),
                display_name.to_string(),
                role,
            )
            .with_email(email.to_string());

            // Generate a proper JWT token
            let token = match state.generate_jwt(&user_id.to_string(), role) {
                Ok(t) => t,
                Err(e) => {
                    warn!("Failed to generate JWT token: {}", e);
                    return Ok(Json(AuthenticateResponse {
                        success: false,
                        user_id: None,
                        access_token: None,
                        token_type: "Bearer".to_string(),
                        expires_in: state.token_expiration_secs,
                        error: Some("Failed to generate authentication token".to_string()),
                        user: None,
                    }));
                }
            };

            info!("JWT authentication successful for user: {}", req.username);

            return Ok(Json(AuthenticateResponse {
                success: true,
                user_id: Some(user_id.to_string()),
                access_token: Some(token),
                token_type: "Bearer".to_string(),
                expires_in: state.token_expiration_secs,
                error: None,
                user: Some(UserResponse::from(user)),
            }));
        }
    }

    // Demo: Create a temporary user for local-first mode (legacy - keeping for backwards compatibility)
    if req.username == "admin" && req.password == "admin123" {
        let user_id = tachyon_core::generate_user_id();
        let user = User::new(
            user_id.clone(),
            "admin".to_string(),
            "Administrator".to_string(),
            UserRole::Admin,
        )
        .with_email("admin@tachyon.local".to_string());

        // Generate a simple token (in production, use proper JWT)
        let token = format!("local_{}", uuid::Uuid::new_v4());

        info!("Local authentication successful for user: {}", req.username);

        return Ok(Json(AuthenticateResponse {
            success: true,
            user_id: Some(user_id.to_string()),
            access_token: Some(token),
            token_type: "Bearer".to_string(),
            expires_in: state.token_expiration_secs,
            error: None,
            user: Some(UserResponse::from(user)),
        }));
    }

    // Authentication failed
    warn!("Authentication failed for user: {}", req.username);

    Ok(Json(AuthenticateResponse {
        success: false,
        user_id: None,
        access_token: None,
        token_type: "Bearer".to_string(),
        expires_in: state.token_expiration_secs,
        error: Some("Invalid username or password".to_string()),
        user: None,
    }))
}

/// Check authentication status
pub async fn auth_status(
    State(state): State<UserState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth_str) if auth_str.starts_with("Bearer ") => {
            let token = &auth_str[7..]; // Skip "Bearer "

            // Validate the JWT token
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

/// Logout user
pub async fn logout(
    State(_state): State<UserState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    info!("User logout");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// Guest login - auto-authenticate as guest user
/// This endpoint is only available when guest_login_enabled is true
pub async fn guest_login(
    State(state): State<UserState>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("Guest login request");

    // Check if guest login is enabled
    if !state.guest_config.guest_login_enabled {
        warn!("Guest login attempted but guest login is disabled");
        return Err((
            StatusCode::FORBIDDEN,
            Json(UserErrorResponse {
                code: "GUEST_LOGIN_DISABLED".to_string(),
                message: "Guest login is not enabled".to_string(),
            }),
        ));
    }

    // Create guest user
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

    // Generate JWT token for guest
    let token = match state.generate_jwt(&user_id.to_string(), UserRole::Reader) {
        Ok(t) => t,
        Err(e) => {
            warn!("Failed to generate guest JWT token: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "TOKEN_ERROR".to_string(),
                    message: "Failed to generate authentication token".to_string(),
                }),
            ));
        }
    };

    info!("Guest login successful for user: {}", user_id);

    Ok(Json(AuthenticateResponse {
        success: true,
        user_id: Some(user_id.to_string()),
        access_token: Some(token),
        token_type: "Bearer".to_string(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user)),
    }))
}

/// Get guest configuration status (public endpoint)
pub async fn guest_status(
    State(state): State<UserState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    Ok(Json(serde_json::json!({
        "guest_login_enabled": state.guest_config.guest_login_enabled,
        "public_notes_enabled": state.guest_config.public_notes_enabled,
    })))
}

/// Create the user router (without state - caller must use .with_state())
pub fn create_user_router() -> axum::Router<UserState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{user_id}", get(get_user))
        .route("/users/{user_id}", put(update_user))
        .route("/users/{user_id}", delete(delete_user))
        .route("/auth/login", post(authenticate))
        .route("/auth/guest", post(guest_login))
        .route("/auth/status", get(auth_status))
        .route("/auth/logout", post(logout))
        .route("/auth/guest-status", get(guest_status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_request_construction() {
        // Note: CreateUserRequest is Deserialize only (for incoming requests)
        let req = CreateUserRequest {
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            password: "password123".to_string(),
            role: Some("writer".to_string()),
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
    }
}
