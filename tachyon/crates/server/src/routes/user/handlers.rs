// Route handler functions for user endpoints

use super::types::{
    AuthenticateRequest, AuthenticateResponse, CreateUserRequest, LogoutRequest,
    REFRESH_TOKEN_EXPIRATION_SECS, RefreshRequest, RegisterRequest, UpdateProfileRequest,
    UpdateUserRequest, UserCursorPage, UserErrorResponse, UserListResponse, UserQuery,
    UserResponse, UserState, hash_refresh_token,
};
use crate::audit::{AuditEvent, AuditEventType, AuditSeverity};
use crate::error::ServerError;
use crate::pagination::{CursorPage, CursorParams};
use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use sqlx::Row;
use tachyon_core::{User, UserId, UserRole};
use tracing::{debug, info, instrument, warn};

// ============================================================================
// Route Handlers
// ============================================================================

/// Register a new user.
///
/// `POST /auth/register`
///
/// Request body: JSON with `username` (3-50 chars), `display_name` (1-100 chars), `password` (8+ chars), optional `email`.
/// Rate limit: 3 requests per minute per IP.
/// Response: 200 with `AuthenticateResponse` (includes `access_token`, `refresh_token`, `user`), or 400/409/500 on error.
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered", body = AuthenticateResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Username or email already exists"),
    ),
    tag = "auth",
)]
#[instrument(skip(state), fields(username = %req.username))]
pub async fn register(
    State(state): State<UserState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<UserErrorResponse>)> {
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

    if let Some(ref email) = req.email
        && (!email.contains('@') || !email.contains('.')) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(UserErrorResponse {
                    code: "VALIDATION_ERROR".into(),
                    message: "Invalid email format".into(),
                }),
            ));
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

            let _ = state
                .audit_logger
                .log(
                    AuditEvent::new(
                        AuditEventType::UserRegistered,
                        AuditSeverity::Medium,
                        "user_register",
                        format!("User '{}' registered", created.username),
                    )
                    .with_target(created.id.to_string(), "user"),
                )
                .await;

            let token = match state.generate_jwt(&created.id.to_string(), created.permissions.role)
            {
                Ok(t) => t,
                Err(e) => {
                    warn!("Failed to generate JWT after registration: {}", e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UserErrorResponse {
                            code: "TOKEN_ERROR".into(),
                            message: "Failed to generate authentication token".into(),
                        }),
                    ));
                }
            };

            let refresh_repo = state.refresh_token_repo();
            let raw_refresh = state.generate_refresh_token();
            let refresh_hash = hash_refresh_token(&raw_refresh);
            if let Err(e) = refresh_repo
                .create(
                    &created.id.to_string(),
                    &refresh_hash,
                    REFRESH_TOKEN_EXPIRATION_SECS as i64,
                )
                .await
            {
                warn!("Failed to create refresh token after registration: {}", e);
            }

            Ok(Json(AuthenticateResponse {
                success: true,
                user_id: Some(created.id.to_string()),
                access_token: Some(token),
                refresh_token: Some(raw_refresh),
                token_type: "Bearer".into(),
                expires_in: state.token_expiration_secs,
                error: None,
                user: Some(UserResponse::from(created)),
                mfa_required: false,
                mfa_user_id: None,
            }))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") || msg.contains("unique")
            {
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
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Conflict"),
    ),
    tag = "users",
)]
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
        Ok(created) => {
            let _ = state
                .audit_logger
                .log(
                    AuditEvent::new(
                        AuditEventType::UserRegistered,
                        AuditSeverity::Medium,
                        "user_create",
                        format!("Admin created user '{}'", created.username),
                    )
                    .with_target(created.id.to_string(), "user"),
                )
                .await;
            Ok(Json(UserResponse::from(created)))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") || msg.contains("unique")
            {
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
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found"),
    ),
    tag = "users",
)]
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
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 404, description = "User not found"),
    ),
    tag = "users",
)]
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
    match repo
        .update(
            &id,
            req.display_name.as_deref(),
            req.email.as_deref(),
            role,
            req.is_active,
        )
        .await
    {
        Ok(user) => {
            let _ = state
                .audit_logger
                .log(
                    AuditEvent::new(
                        AuditEventType::TeamMemberUpdated,
                        AuditSeverity::Low,
                        "user_update",
                        format!("User '{}' updated", user_id),
                    )
                    .with_target(&user_id, "user"),
                )
                .await;
            Ok(Json(UserResponse::from(user)))
        }
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
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "User deactivated"),
        (status = 400, description = "Invalid user ID"),
        (status = 404, description = "User not found"),
    ),
    tag = "users",
)]
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
        Ok(()) => {
            let _ = state
                .audit_logger
                .log(
                    AuditEvent::new(
                        AuditEventType::TeamMemberRemoved,
                        AuditSeverity::Medium,
                        "user_delete",
                        format!("User '{}' deactivated", user_id),
                    )
                    .with_target(&user_id, "user"),
                )
                .await;
            Ok(StatusCode::NO_CONTENT)
        }
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
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(UserQuery),
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 500, description = "Internal error"),
    ),
    tag = "users",
)]
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

/// Get the current authenticated user's profile.
///
/// `GET /auth/me`
///
/// Requires `Authorization: Bearer <token>` header.
/// Response: 200 with `UserResponse`, or 401/404 on error.
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "users",
)]
pub async fn get_me(
    State(state): State<UserState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

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

/// Update the current authenticated user's profile.
///
/// `PUT /auth/me`
///
/// Requires `Authorization: Bearer <token>` header.
/// Request body: JSON with optional `display_name`, `email`.
/// Response: 200 with updated `UserResponse`, or 401/409/404 on error.
#[utoipa::path(
    put,
    path = "/auth/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
pub async fn update_me(
    State(state): State<UserState>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<UserErrorResponse>)> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

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
        .update(
            &uid,
            req.display_name.as_deref(),
            req.email.as_deref(),
            None,
            None,
        )
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

/// Authenticate a user with username/email and password.
///
/// `POST /auth/login`
///
/// Request body: JSON with `username` (or email), `password`.
/// Rate limit: 5 requests per minute per IP.
/// Response: 200 with `AuthenticateResponse`. Returns `mfa_required: true` if MFA is enabled.
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = AuthenticateRequest,
    responses(
        (status = 200, description = "Authentication result", body = AuthenticateResponse),
        (status = 500, description = "Internal error"),
    ),
    tag = "auth",
)]
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
            refresh_token: None,
            token_type: "Bearer".into(),
            expires_in: state.token_expiration_secs,
            error: Some("Username cannot be empty".into()),
            user: None,
            mfa_required: false,
            mfa_user_id: None,
        }));
    }

    if req.password.is_empty() {
        return Ok(Json(AuthenticateResponse {
            success: false,
            user_id: None,
            access_token: None,
            refresh_token: None,
            token_type: "Bearer".into(),
            expires_in: state.token_expiration_secs,
            error: Some("Password cannot be empty".into()),
            user: None,
            mfa_required: false,
            mfa_user_id: None,
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
                refresh_token: None,
                token_type: "Bearer".into(),
                expires_in: state.token_expiration_secs,
                error: Some("Invalid username or password".into()),
                user: None,
                mfa_required: false,
                mfa_user_id: None,
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
            refresh_token: None,
            token_type: "Bearer".into(),
            expires_in: state.token_expiration_secs,
            error: Some("Account is disabled".into()),
            user: None,
            mfa_required: false,
            mfa_user_id: None,
        }));
    }

    // Verify password
    match user.verify(&req.password) {
        Ok(true) => {}
        Ok(false) | Err(_) => {
            debug!(
                "Authentication failed: invalid password for user {}",
                user.username
            );
            return Ok(Json(AuthenticateResponse {
                success: false,
                user_id: None,
                access_token: None,
                refresh_token: None,
                token_type: "Bearer".into(),
                expires_in: state.token_expiration_secs,
                error: Some("Invalid username or password".into()),
                user: None,
                mfa_required: false,
                mfa_user_id: None,
            }));
        }
    }

    // Check if MFA is enabled
    let mut conn = state.pool.acquire().await.map_err(|e| {
        warn!("Failed to acquire connection for MFA check: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserErrorResponse {
                code: "INTERNAL_ERROR".into(),
                message: "Database error".into(),
            }),
        )
    })?;
    let totp_row = sqlx::query("SELECT totp_enabled FROM users WHERE id = $1")
        .bind(user.id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| {
            warn!("Failed to check MFA status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "INTERNAL_ERROR".into(),
                    message: "Database error".into(),
                }),
            )
        })?;

    if let Some(row) = totp_row {
        let totp_enabled: bool = row.get("totp_enabled");
        if totp_enabled {
            info!("MFA required for user {} ({})", user.username, user.id);
            return Ok(Json(AuthenticateResponse {
                success: true,
                user_id: Some(user.id.to_string()),
                access_token: None,
                refresh_token: None,
                token_type: "Bearer".into(),
                expires_in: state.token_expiration_secs,
                error: None,
                user: Some(UserResponse::from(user.clone())),
                mfa_required: true,
                mfa_user_id: Some(user.id.to_string()),
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

    // Generate refresh token
    let refresh_repo = state.refresh_token_repo();
    let raw_refresh = state.generate_refresh_token();
    let refresh_hash = hash_refresh_token(&raw_refresh);
    if let Err(e) = refresh_repo
        .create(
            &user.id.to_string(),
            &refresh_hash,
            REFRESH_TOKEN_EXPIRATION_SECS as i64,
        )
        .await
    {
        warn!("Failed to create refresh token: {}", e);
    }

    info!("Authentication successful: {} ({})", user.username, user.id);

    Ok(Json(AuthenticateResponse {
        success: true,
        user_id: Some(user.id.to_string()),
        access_token: Some(token),
        refresh_token: Some(raw_refresh),
        token_type: "Bearer".into(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user)),
        mfa_required: false,
        mfa_user_id: None,
    }))
}

/// Check authentication status.
#[utoipa::path(
    get,
    path = "/auth/status",
    responses(
        (status = 200, description = "Auth status"),
    ),
    tag = "auth",
)]
pub async fn auth_status(
    State(state): State<UserState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

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
/// Revokes the provided refresh token. If no refresh token is provided,
/// returns success (backward compatible with stateless JWT logout).
#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = Option<LogoutRequest>,
    responses(
        (status = 200, description = "Logged out"),
    ),
    tag = "auth",
)]
pub async fn logout(
    State(state): State<UserState>,
    body: Option<Json<LogoutRequest>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    info!("User logout");

    if let Some(Json(req)) = body
        && let Some(ref token) = req.refresh_token {
            let hash = hash_refresh_token(token);
            let repo = state.refresh_token_repo();
            if let Err(e) = repo.revoke(&hash).await {
                warn!("Failed to revoke refresh token: {}", e);
            }
        }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// Refresh an access token using a refresh token.
///
/// `POST /auth/refresh`
///
/// Request body: JSON with `refresh_token`.
/// Rate limit: 10 requests per minute per IP.
/// Revokes the old refresh token and issues a new access + refresh token pair.
/// Response: 200 with `AuthenticateResponse`, or 401/500 on error.
#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = AuthenticateResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 500, description = "Internal error"),
    ),
    tag = "auth",
)]
pub async fn refresh_token_handler(
    State(state): State<UserState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<UserErrorResponse>)> {
    info!("Token refresh request");

    let token_hash = hash_refresh_token(&req.refresh_token);

    let repo = state.refresh_token_repo();
    let stored = match repo.find_valid_by_hash(&token_hash).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "INVALID_TOKEN".into(),
                    message: "Invalid or expired refresh token".into(),
                }),
            ));
        }
        Err(e) => {
            warn!("Failed to validate refresh token: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "INTERNAL_ERROR".into(),
                    message: "Failed to validate refresh token".into(),
                }),
            ));
        }
    };

    let user_id = match UserId::parse_str(&stored.user_id) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "INTERNAL_ERROR".into(),
                    message: "Invalid user ID in refresh token".into(),
                }),
            ));
        }
    };

    let user_repo = state.user_repo();
    let user = match user_repo.get_by_id(&user_id).await {
        Ok(u) => u,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(UserErrorResponse {
                    code: "USER_NOT_FOUND".into(),
                    message: "User not found".into(),
                }),
            ));
        }
    };

    if !user.is_active.unwrap_or(true) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(UserErrorResponse {
                code: "ACCOUNT_DISABLED".into(),
                message: "Account is disabled".into(),
            }),
        ));
    }

    let _ = repo.revoke(&token_hash).await;

    let access_token = match state.generate_jwt(&user.id.to_string(), user.permissions.role) {
        Ok(t) => t,
        Err(e) => {
            warn!("Failed to generate JWT on refresh: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserErrorResponse {
                    code: "TOKEN_ERROR".into(),
                    message: "Failed to generate access token".into(),
                }),
            ));
        }
    };

    let new_raw_refresh = state.generate_refresh_token();
    let new_hash = hash_refresh_token(&new_raw_refresh);
    if let Err(e) = repo
        .create(
            &user.id.to_string(),
            &new_hash,
            REFRESH_TOKEN_EXPIRATION_SECS as i64,
        )
        .await
    {
        warn!("Failed to create refresh token: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserErrorResponse {
                code: "TOKEN_ERROR".into(),
                message: "Failed to generate refresh token".into(),
            }),
        ));
    }

    info!("Token refresh successful: user {}", user.id);

    Ok(Json(AuthenticateResponse {
        success: true,
        user_id: Some(user.id.to_string()),
        access_token: Some(access_token),
        refresh_token: Some(new_raw_refresh),
        token_type: "Bearer".into(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user)),
        mfa_required: false,
        mfa_user_id: None,
    }))
}

/// POST /auth/guest
/// Rate limit: 3 requests per minute per IP
#[utoipa::path(
    post,
    path = "/auth/guest",
    responses(
        (status = 200, description = "Guest token", body = AuthenticateResponse),
        (status = 403, description = "Guest login disabled"),
        (status = 500, description = "Internal error"),
    ),
    tag = "auth",
)]
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
        user_id,
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
        refresh_token: None,
        token_type: "Bearer".into(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user)),
        mfa_required: false,
        mfa_user_id: None,
    }))
}

/// Get guest configuration status (public endpoint).
#[utoipa::path(
    get,
    path = "/auth/guest-status",
    responses(
        (status = 200, description = "Guest configuration"),
    ),
    tag = "auth",
)]
pub async fn guest_status(
    State(state): State<UserState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<UserErrorResponse>)> {
    Ok(Json(serde_json::json!({
        "guest_login_enabled": state.guest_config.guest_login_enabled,
        "public_notes_enabled": state.guest_config.public_notes_enabled,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/cursor",
    params(CursorParams),
    responses(
        (status = 200, description = "Users (cursor paginated)", body = UserCursorPage),
        (status = 500, description = "Internal error"),
    ),
    tag = "users",
)]
pub async fn list_users_cursor(
    Query(params): Query<CursorParams>,
    State(state): State<UserState>,
) -> Result<Json<UserCursorPage>, (StatusCode, Json<UserErrorResponse>)> {
    let limit = params.limit();
    let direction = params.direction();
    let cursor_str = params.after.as_deref().or(params.before.as_deref());
    let fetch_limit = limit + 1;

    let repo = state.user_repo();
    match repo.list(1, fetch_limit, None).await {
        Ok((users, total)) => {
            let user_responses: Vec<UserResponse> =
                users.into_iter().map(UserResponse::from).collect();

            let has_extra = user_responses.len() > limit;
            let mut user_responses = user_responses;
            if has_extra {
                user_responses.truncate(limit);
            }

            let first_id = user_responses.first().map(|u| u.id.clone());
            let last_id = user_responses.last().map(|u| u.id.clone());
            let has_prev = cursor_str.is_some();

            let page = CursorPage::new(user_responses, has_extra, has_prev)
                .with_cursors(first_id.as_deref(), last_id.as_deref(), direction)
                .with_total_count(total);

            Ok(Json(UserCursorPage::from(page)))
        }
        Err(e) => {
            warn!("Failed to list users (cursor): {}", e);
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

// ============================================================================
// Router
// ============================================================================

/// Export all user data (GDPR Article 20: Right to data portability).
pub async fn export_my_data(
    Extension(auth_context): Extension<crate::middleware::AuthContext>,
    State(state): State<UserState>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = auth_context.user_id.clone();
    info!(user_id = %user_id, "Exporting user data for GDPR compliance");

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    #[allow(clippy::type_complexity)]
    let profile_row: Option<(
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    )> = sqlx::query_as(
        "SELECT id, username, display_name, email, role, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(&user_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let mut builder = tachyon_import_export::GdprExportBuilder::new(user_id.clone());

    if let Some((id, username, display_name, email, role, created_at, updated_at)) = profile_row {
        builder = builder.profile(tachyon_import_export::UserProfile {
            id,
            username,
            display_name,
            email,
            role,
            created_at: created_at.map(|d| d.to_rfc3339()),
            updated_at: updated_at.map(|d| d.to_rfc3339()),
        });
    }

    #[allow(clippy::type_complexity)]
    let doc_rows: Vec<(
        String,
        String,
        String,
        String,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    )> = sqlx::query_as(
        "SELECT id, title, content, slug, created_at, updated_at
         FROM documents WHERE author_id = $1 AND deleted_at IS NULL
         ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let docs: Vec<tachyon_import_export::UserDocument> = doc_rows
        .into_iter()
        .map(|(id, title, content, slug, created_at, updated_at)| {
            tachyon_import_export::UserDocument {
                id,
                title,
                content,
                slug,
                tags: vec![],
                created_at: created_at.map(|d| d.to_rfc3339()),
                updated_at: updated_at.map(|d| d.to_rfc3339()),
            }
        })
        .collect();

    builder = builder.documents(docs);

    #[allow(clippy::type_complexity)]
    let comment_rows: Vec<(
        String,
        String,
        String,
        Option<chrono::DateTime<chrono::Utc>>,
        bool,
    )> = sqlx::query_as(
        "SELECT id, document_id, content, created_at, COALESCE(is_resolved, false)
         FROM document_comments WHERE author_id = $1 AND deleted_at IS NULL
         ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let comments: Vec<tachyon_import_export::UserComment> = comment_rows
        .into_iter()
        .map(|(id, document_id, content, created_at, is_resolved)| {
            tachyon_import_export::UserComment {
                id,
                document_id,
                content,
                created_at: created_at.map(|d| d.to_rfc3339()),
                is_resolved,
            }
        })
        .collect();

    builder = builder.comments(comments);

    #[allow(clippy::type_complexity)]
    let activity_rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
    )> = sqlx::query_as(
        "SELECT id, event_type, target_type, target_id, description, created_at
         FROM activity_events WHERE actor_id = $1
         ORDER BY created_at DESC LIMIT 1000",
    )
    .bind(&user_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let activities: Vec<tachyon_import_export::UserActivity> = activity_rows
        .into_iter()
        .map(
            |(id, event_type, target_type, target_id, description, created_at)| {
                tachyon_import_export::UserActivity {
                    id,
                    event_type,
                    target_type,
                    target_id,
                    description,
                    created_at: created_at.map(|d| d.to_rfc3339()),
                }
            },
        )
        .collect();

    builder = builder.activities(activities);

    let bytes = builder
        .build()
        .map_err(|e| ServerError::internal(e.to_string()))?;

    let headers = HeaderMap::from_iter([
        (
            axum::http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        ),
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"tachyon-export-{}.json\"", user_id)
                .parse()
                .unwrap(),
        ),
    ]);

    Ok((headers, bytes))
}

// ============================================================================
// Router
// ============================================================================

/// Create the user router.
///
/// Routes:
/// - `POST /auth/register` — public registration (Reader role)
/// - `POST /auth/login` — authenticate (username/email + password)
/// - `POST /auth/refresh` — refresh access + refresh tokens
/// - `POST /auth/guest` — guest login (when enabled)
/// - `GET  /auth/status` — check JWT validity
/// - `POST /auth/logout` — logout (revokes refresh token if provided)
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
        .route("/auth/refresh", post(refresh_token_handler))
        .route("/auth/guest", post(guest_login))
        .route("/auth/status", get(auth_status))
        .route("/auth/logout", post(logout))
        .route("/auth/guest-status", get(guest_status))
        .route("/auth/me", get(get_me))
        .route("/auth/me", put(update_me))
        .route("/auth/me/export", get(export_my_data))
        // User management routes
        .route("/users", get(list_users))
        .route("/users/cursor", get(list_users_cursor))
        .route("/users", post(create_user))
        .route("/users/me", get(get_me))
        .route("/users/{user_id}", get(get_user))
        .route("/users/{user_id}", put(update_user))
        .route("/users/{user_id}", delete(delete_user))
}
