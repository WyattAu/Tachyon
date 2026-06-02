use crate::audit::{AuditEvent, AuditEventType, AuditLogger, AuditOutcome, AuditSeverity};
use crate::error::ServerError;
use axum::{Router, extract::State, response::Json, routing::post};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tachyon_database::{DatabasePool, MagicLinkRepository, RefreshTokenRepository, UserRepository};
use tracing::{info, warn};

use crate::routes::user::UserState;
use crate::routes::user::types::{
    AuthenticateResponse, REFRESH_TOKEN_EXPIRATION_SECS, UserResponse, hash_refresh_token,
};

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct MagicLinkState {
    pub pool: DatabasePool,
    pub client: reqwest::Client,
    pub audit_logger: AuditLogger,
    pub jwt_secrets: Vec<String>,
    pub token_expiration_secs: u64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub base_url: String,
    pub ttl_secs: i64,
}

pub fn create_magic_link_router() -> Router<MagicLinkState> {
    Router::new()
        .route("/auth/magic-link/request", post(request_magic_link))
        .route("/auth/magic-link/verify", post(verify_magic_link))
}

// ============================================================================
// Request / Response types
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct MagicLinkVerify {
    pub token: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MagicLinkMessageResponse {
    pub message: String,
}

// ============================================================================
// Token helpers
// ============================================================================

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_token() -> String {
    uuid::Uuid::new_v4().to_string().replace('-', "")
}

async fn send_email_webhook(client: &reqwest::Client, to: &str, subject: &str, body: &str) {
    let webhook_url =
        std::env::var("TACHYON_EMAIL_WEBHOOK_URL").unwrap_or_else(|_| "/dev/null".to_string());

    if webhook_url == "/dev/null" {
        info!(to = %to, subject = %subject, "Email webhook not configured, skipping email delivery");
        return;
    }

    let payload = serde_json::json!({
        "to": to,
        "subject": subject,
        "body": body,
    });

    match client.post(&webhook_url).json(&payload).send().await {
        Ok(resp) if resp.status().is_success() => {
            info!(to = %to, subject = %subject, "Email sent via webhook");
        }
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!(status = %status, body = %text, "Email webhook returned non-success");
        }
        Err(e) => {
            warn!(error = %e, "Failed to send email via webhook");
        }
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /auth/magic-link/request
/// Rate limit: 3 requests per minute per IP
#[utoipa::path(
    post,
    path = "/auth/magic-link/request",
    request_body(content = MagicLinkRequest, description = "Magic link request"),
    responses(
        (status = 200, description = "Magic link email sent", body = MagicLinkMessageResponse),
        (status = 400, description = "Invalid email"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth",
)]
pub async fn request_magic_link(
    State(state): State<MagicLinkState>,
    Json(body): Json<MagicLinkRequest>,
) -> Result<Json<MagicLinkMessageResponse>, ServerError> {
    let email = body.email.trim().to_string();

    if email.is_empty() || !email.contains('@') || !email.contains('.') {
        return Err(ServerError::bad_request("Invalid email format"));
    }

    let user_repo = UserRepository::new(state.pool.clone());

    let user = match user_repo.get_by_email(&email).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(Json(MagicLinkMessageResponse {
                message: "If an account with that email exists, a login link has been sent."
                    .to_string(),
            }));
        }
    };

    let token = generate_token();
    let token_hash = hash_token(&token);

    let magic_repo = MagicLinkRepository::new(state.pool.clone());
    magic_repo
        .create_token(&user.id.as_str(), &token_hash, state.ttl_secs, None)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to create magic link token: {}", e)))?;

    let magic_link = format!("{}/auth/magic-link/verify?token={}", state.base_url, token);

    send_email_webhook(
        &state.client,
        &email,
        "Tachyon Magic Link Login",
        &format!(
            "You requested a magic link login. Click the link below to sign in:\n\n{}\n\n\
             This link expires in {} minutes. If you did not request this, you can safely ignore this email.",
            magic_link,
            state.ttl_secs / 60
        ),
    )
    .await;

    info!(email = %email, "Magic link email sent");
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                AuditSeverity::Medium,
                "magic_link_request",
                "Magic link login requested",
            )
            .with_metadata("email", serde_json::json!(email))
            .with_outcome(AuditOutcome::Success),
        )
        .await;

    Ok(Json(MagicLinkMessageResponse {
        message: "If an account with that email exists, a login link has been sent.".to_string(),
    }))
}

/// POST /auth/magic-link/verify
/// Rate limit: 10 requests per minute per IP
#[utoipa::path(
    post,
    path = "/auth/magic-link/verify",
    request_body(content = MagicLinkVerify, description = "Magic link verification"),
    responses(
        (status = 200, description = "Authenticated", body = AuthenticateResponse),
        (status = 400, description = "Invalid or expired token"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth",
)]
pub async fn verify_magic_link(
    State(state): State<MagicLinkState>,
    Json(body): Json<MagicLinkVerify>,
) -> Result<Json<AuthenticateResponse>, ServerError> {
    let token_hash = hash_token(&body.token);

    let magic_repo = MagicLinkRepository::new(state.pool.clone());
    let magic_token = magic_repo.consume_token(&token_hash).await.map_err(|e| {
        ServerError::internal(format!("Failed to validate magic link token: {}", e))
    })?;

    let magic_token = magic_token
        .ok_or_else(|| ServerError::bad_request("Token is invalid, expired, or already used"))?;

    let user_repo = UserRepository::new(state.pool.clone());
    let user_id = tachyon_core::id::UserId::parse_str(&magic_token.user_id)
        .map_err(|e| ServerError::internal(format!("Invalid user ID: {}", e)))?;
    let user = user_repo
        .get_by_id(&user_id)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to look up user: {}", e)))?;

    let user_state = UserState::with_guest_config(
        state.pool.clone(),
        state.jwt_secrets.clone(),
        state.token_expiration_secs,
        state.jwt_issuer.clone(),
        state.jwt_audience.clone(),
        crate::config::GuestConfig::default(),
    );

    let access_token = user_state
        .generate_jwt(&user.id.to_string(), user.permissions.role)
        .map_err(|e| ServerError::internal(format!("Failed to generate JWT: {}", e)))?;

    let refresh_repo = RefreshTokenRepository::new(state.pool.clone());
    let raw_refresh = {
        let bytes: [u8; 32] = rand::random();
        hex::encode(bytes)
    };
    let refresh_hash = hash_refresh_token(&raw_refresh);
    if let Err(e) = refresh_repo
        .create(
            &user.id.to_string(),
            &refresh_hash,
            REFRESH_TOKEN_EXPIRATION_SECS as i64,
        )
        .await
    {
        warn!(
            "Failed to create refresh token during magic link verify: {}",
            e
        );
    }

    info!(user_id = %user.id, "Magic link login successful");
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                AuditSeverity::Medium,
                "magic_link_verify",
                format!("Magic link login for user '{}'", user.id),
            )
            .with_target(user.id.to_string(), "user")
            .with_outcome(AuditOutcome::Success),
        )
        .await;

    Ok(Json(AuthenticateResponse {
        success: true,
        user_id: Some(user.id.to_string()),
        access_token: Some(access_token),
        refresh_token: Some(raw_refresh),
        token_type: "Bearer".into(),
        expires_in: state.token_expiration_secs,
        error: None,
        user: Some(UserResponse::from(user)),
        mfa_required: false,
        mfa_user_id: None,
    }))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_deterministic() {
        let hash1 = hash_token("magic-token-1");
        let hash2 = hash_token("magic-token-1");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = hash_token("token-a");
        let hash2 = hash_token("token-b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_token_unique() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
        assert!(!t1.contains('-'));
    }

    #[test]
    fn test_generate_token_length() {
        let token = generate_token();
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_generate_token_no_dashes() {
        let token = generate_token();
        assert!(!token.contains('-'));
    }

    #[test]
    fn test_hash_token_is_valid_hex() {
        let hash = hash_token("some-magic-token");
        assert!(hex::decode(&hash).is_ok());
    }

    #[test]
    fn test_hash_token_empty_string() {
        let hash = hash_token("");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_token_length() {
        let long_token = "a".repeat(1000);
        let hash = hash_token(&long_token);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_magic_link_request_deserialization() {
        let json = r#"{"email":"user@example.com"}"#;
        let req: MagicLinkRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "user@example.com");
    }

    #[test]
    fn test_magic_link_verify_deserialization() {
        let json = r#"{"token":"sometoken"}"#;
        let req: MagicLinkVerify = serde_json::from_str(json).unwrap();
        assert_eq!(req.token, "sometoken");
    }

    #[test]
    fn test_magic_link_message_response_serialization() {
        let resp = MagicLinkMessageResponse {
            message: "If an account exists, a login link has been sent.".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("login link"));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["message"], resp.message);
    }
}
