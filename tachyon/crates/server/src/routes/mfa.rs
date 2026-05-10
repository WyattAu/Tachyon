use crate::middleware::auth::AuthContext;
use crate::routes::user::{
    hash_refresh_token, AuthenticateResponse, UserErrorResponse, UserResponse, UserState,
    REFRESH_TOKEN_EXPIRATION_SECS,
};
use crate::totp::{generate_backup_codes, generate_otpauth_uri, generate_secret, verify_totp};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::{info, instrument, warn};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MfaEnableResponse {
    pub secret: String,
    pub qr_code_uri: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct MfaVerifyRequest {
    pub code: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct MfaDisableRequest {
    pub code: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct MfaAuthRequest {
    pub user_id: String,
    pub mfa_code: String,
    pub backup_code: Option<String>,
}

type Error = (StatusCode, Json<UserErrorResponse>);

fn bad_request(code: &str, message: &str) -> Error {
    (
        StatusCode::BAD_REQUEST,
        Json(UserErrorResponse {
            code: code.into(),
            message: message.into(),
        }),
    )
}

fn unauthorized(code: &str, message: &str) -> Error {
    (
        StatusCode::UNAUTHORIZED,
        Json(UserErrorResponse {
            code: code.into(),
            message: message.into(),
        }),
    )
}

fn internal_error() -> Error {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(UserErrorResponse {
            code: "INTERNAL_ERROR".into(),
            message: "Internal server error".into(),
        }),
    )
}

fn db_error(e: impl std::fmt::Display) -> Error {
    warn!("Database error: {}", e);
    internal_error()
}

/// Initiate TOTP MFA setup for the authenticated user.
///
/// `POST /api/v1/auth/mfa/enable`
///
/// Generates a new TOTP secret, QR code URI, and 10 backup codes.
/// MFA is not active until the user verifies a code via the verify endpoint.
#[utoipa::path(
    post,
    path = "/auth/mfa/enable",
    responses(
        (status = 200, description = "MFA setup initiated", body = MfaEnableResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal error"),
    ),
    tag = "auth",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn enable_mfa(
    State(state): State<UserState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<MfaEnableResponse>, Error> {
    info!("MFA enable request: user {}", auth.user_id);

    let user_id = tachyon_core::UserId::parse_str(&auth.user_id)
        .map_err(|_| bad_request("INVALID_USER", "Invalid user ID"))?;

    let repo = state.user_repo();
    let user = repo
        .get_by_id(&user_id)
        .await
        .map_err(|_| bad_request("USER_NOT_FOUND", "User not found"))?;

    let email = user.email.as_deref().unwrap_or(&user.username);
    let secret = generate_secret();
    let backup_codes = generate_backup_codes(10);
    let qr_uri = generate_otpauth_uri(&secret, email, "Tachyon");

    let mut conn = state.pool.acquire().await.map_err(db_error)?;
    sqlx::query(
        "UPDATE users SET totp_secret = $1, totp_backup_codes = $2, totp_enabled = false WHERE id = $3",
    )
    .bind(&secret)
    .bind(&backup_codes)
    .bind(user.id.as_uuid())
    .execute(&mut *conn)
    .await
    .map_err(db_error)?;

    info!("MFA setup initiated for user {}", auth.user_id);

    Ok(Json(MfaEnableResponse {
        secret,
        qr_code_uri: qr_uri,
        backup_codes: backup_codes.clone(),
    }))
}

/// Verify a TOTP code to complete MFA setup.
///
/// `POST /api/v1/auth/mfa/verify`
///
/// Accepts a 6-digit TOTP code. On success, enables MFA for the user.
#[utoipa::path(
    post,
    path = "/auth/mfa/verify",
    request_body = MfaVerifyRequest,
    responses(
        (status = 200, description = "MFA enabled", body = serde_json::Value),
        (status = 400, description = "Invalid code or MFA not setup"),
        (status = 500, description = "Internal error"),
    ),
    tag = "auth",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn verify_mfa(
    State(state): State<UserState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    info!("MFA verify request: user {}", auth.user_id);

    let code: u32 = req
        .code
        .parse()
        .map_err(|_| bad_request("INVALID_CODE", "Code must be 6 digits"))?;

    let user_id = tachyon_core::UserId::parse_str(&auth.user_id)
        .map_err(|_| bad_request("INVALID_USER", "Invalid user ID"))?;

    let mut conn = state.pool.acquire().await.map_err(db_error)?;
    let row = sqlx::query("SELECT totp_secret FROM users WHERE id = $1")
        .bind(user_id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| bad_request("MFA_NOT_SETUP", "TOTP secret not found"))?;

    let secret: Option<String> = row.get("totp_secret");
    let secret = secret.ok_or_else(|| bad_request("MFA_NOT_SETUP", "TOTP secret not found"))?;

    if !verify_totp(&secret, code)
        .map_err(|_| bad_request("INVALID_CODE", "TOTP verification error"))?
    {
        return Err(bad_request("INVALID_CODE", "Invalid TOTP code"));
    }

    sqlx::query("UPDATE users SET totp_enabled = true, totp_verified_at = NOW() WHERE id = $1")
        .bind(user_id.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(db_error)?;

    info!("MFA enabled for user {}", auth.user_id);

    Ok(Json(serde_json::json!({"enabled": true})))
}

/// Disable MFA for the authenticated user.
///
/// `POST /api/v1/auth/mfa/disable`
///
/// Requires a valid TOTP code to confirm the action. Clears the secret and backup codes.
#[utoipa::path(
    post,
    path = "/auth/mfa/disable",
    request_body = MfaDisableRequest,
    responses(
        (status = 200, description = "MFA disabled", body = serde_json::Value),
        (status = 400, description = "Invalid code or MFA not enabled"),
        (status = 500, description = "Internal error"),
    ),
    tag = "auth",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn disable_mfa(
    State(state): State<UserState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MfaDisableRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    info!("MFA disable request: user {}", auth.user_id);

    let code: u32 = req
        .code
        .parse()
        .map_err(|_| bad_request("INVALID_CODE", "Code must be 6 digits"))?;

    let user_id = tachyon_core::UserId::parse_str(&auth.user_id)
        .map_err(|_| bad_request("INVALID_USER", "Invalid user ID"))?;

    let mut conn = state.pool.acquire().await.map_err(db_error)?;
    let row = sqlx::query("SELECT totp_secret, totp_enabled FROM users WHERE id = $1")
        .bind(user_id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| bad_request("MFA_NOT_SETUP", "TOTP not enabled"))?;

    let totp_enabled: bool = row.get("totp_enabled");
    if !totp_enabled {
        return Err(bad_request("MFA_NOT_SETUP", "TOTP not enabled"));
    }

    let secret: Option<String> = row.get("totp_secret");
    let secret = secret.ok_or_else(|| bad_request("MFA_NOT_SETUP", "TOTP secret not found"))?;

    if !verify_totp(&secret, code)
        .map_err(|_| bad_request("INVALID_CODE", "TOTP verification error"))?
    {
        return Err(bad_request("INVALID_CODE", "Invalid TOTP code"));
    }

    sqlx::query(
        "UPDATE users SET totp_enabled = false, totp_secret = NULL, totp_backup_codes = NULL, totp_verified_at = NULL WHERE id = $1",
    )
    .bind(user_id.as_uuid())
    .execute(&mut *conn)
    .await
    .map_err(db_error)?;

    info!("MFA disabled for user {}", auth.user_id);

    Ok(Json(serde_json::json!({"enabled": false})))
}

/// Authenticate with MFA during login.
///
/// `POST /api/v1/auth/mfa/authenticate`
///
/// Accepts a TOTP code or a backup code. On success, issues access and refresh tokens.
#[utoipa::path(
    post,
    path = "/auth/mfa/authenticate",
    request_body = MfaAuthRequest,
    responses(
        (status = 200, description = "MFA authentication successful", body = serde_json::Value),
        (status = 400, description = "Invalid code or MFA not setup"),
        (status = 401, description = "Invalid MFA code"),
        (status = 500, description = "Internal error"),
    ),
    tag = "auth",
)]
#[instrument(skip(state))]
pub async fn mfa_authenticate(
    State(state): State<UserState>,
    Json(req): Json<MfaAuthRequest>,
) -> Result<Json<AuthenticateResponse>, Error> {
    info!("MFA authenticate request");

    let user_id = tachyon_core::UserId::parse_str(&req.user_id)
        .map_err(|_| bad_request("INVALID_USER", "Invalid user ID"))?;

    let repo = state.user_repo();
    let user = repo
        .get_by_id(&user_id)
        .await
        .map_err(|_| unauthorized("INVALID_USER", "Invalid user ID"))?;

    if !user.is_active.unwrap_or(true) {
        return Err(unauthorized("ACCOUNT_DISABLED", "Account is disabled"));
    }

    let mut conn = state.pool.acquire().await.map_err(db_error)?;
    let row =
        sqlx::query("SELECT totp_enabled, totp_secret, totp_backup_codes FROM users WHERE id = $1")
            .bind(user_id.as_uuid())
            .fetch_optional(&mut *conn)
            .await
            .map_err(db_error)?
            .ok_or_else(|| bad_request("MFA_NOT_SETUP", "TOTP not enabled"))?;

    let totp_enabled: bool = row.get("totp_enabled");
    if !totp_enabled {
        return Err(bad_request(
            "MFA_NOT_SETUP",
            "MFA is not enabled for this user",
        ));
    }

    if let Some(ref backup) = req.backup_code {
        let codes: Option<Vec<String>> = row.get("totp_backup_codes");
        let stored_codes = codes.as_deref().unwrap_or(&[]);

        if stored_codes.iter().any(|c| c == backup) {
            let remaining: Vec<String> = stored_codes
                .iter()
                .filter(|c| *c != backup)
                .cloned()
                .collect();

            sqlx::query("UPDATE users SET totp_backup_codes = $1 WHERE id = $2")
                .bind(&remaining)
                .bind(user_id.as_uuid())
                .execute(&mut *conn)
                .await
                .map_err(db_error)?;

            info!("MFA backup code used for user {}", req.user_id);
            return issue_tokens(&state, &user).await;
        }
    }

    let code: u32 = req
        .mfa_code
        .parse()
        .map_err(|_| bad_request("INVALID_CODE", "Code must be 6 digits"))?;

    let secret: Option<String> = row.get("totp_secret");
    let secret = secret.ok_or_else(|| bad_request("MFA_NOT_SETUP", "TOTP secret not found"))?;

    if !verify_totp(&secret, code)
        .map_err(|_| bad_request("INVALID_CODE", "TOTP verification error"))?
    {
        return Err(unauthorized("INVALID_MFA_CODE", "Invalid MFA code"));
    }

    info!("MFA authentication successful for user {}", req.user_id);
    issue_tokens(&state, &user).await
}

async fn issue_tokens(
    state: &UserState,
    user: &tachyon_core::User,
) -> Result<Json<AuthenticateResponse>, Error> {
    let token = state
        .generate_jwt(&user.id.to_string(), user.permissions.role)
        .map_err(|e| {
            warn!("Failed to generate JWT: {}", e);
            internal_error()
        })?;

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

    Ok(Json(AuthenticateResponse {
        success: true,
        user_id: Some(user.id.to_string()),
        access_token: Some(token),
        refresh_token: Some(raw_refresh),
        token_type: "Bearer".into(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user.clone())),
        mfa_required: false,
        mfa_user_id: None,
    }))
}

pub fn create_mfa_router() -> axum::Router<UserState> {
    use axum::routing::post;
    axum::Router::new()
        .route("/auth/mfa/enable", post(enable_mfa))
        .route("/auth/mfa/verify", post(verify_mfa))
        .route("/auth/mfa/disable", post(disable_mfa))
        .route("/auth/mfa/authenticate", post(mfa_authenticate))
}
