use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use tachyon_database::DatabasePool;
use tracing::{info, warn};

use crate::middleware::auth::Claims;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct PasswordResetState {
    pub pool: DatabasePool,
    pub client: reqwest::Client,
}

pub fn create_password_reset_router() -> Router<PasswordResetState> {
    Router::new()
        .route("/auth/password-reset/request", post(request_password_reset))
        .route("/auth/password-reset/confirm", post(confirm_password_reset))
        .route("/auth/email-verify/request", post(request_email_verification))
        .route("/auth/email-verify/confirm", post(confirm_email_verification))
}

// ============================================================================
// Request / Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailVerifyRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailVerifyConfirm {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
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
    let webhook_url = std::env::var("TACHYON_EMAIL_WEBHOOK_URL")
        .unwrap_or_else(|_| "/dev/null".to_string());

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

pub async fn request_password_reset(
    State(state): State<PasswordResetState>,
    Json(body): Json<PasswordResetRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_repo = tachyon_database::UserRepository::new(state.pool.clone());
    let reset_repo = tachyon_database::PasswordResetRepository::new(state.pool.clone());

    let user = match user_repo.get_by_email(&body.email).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(Json(MessageResponse {
                message: "If an account with that email exists, a password reset link has been sent.".to_string(),
            }));
        }
    };

    let token = generate_token();
    let token_hash = hash_token(&token);

    reset_repo
        .create_reset_token(&user.id.as_str(), &token_hash, 1)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "TOKEN_ERROR".to_string(),
                    message: format!("Failed to create reset token: {}", e),
                }),
            )
        })?;

    let reset_link = format!(
        "{}/reset-password?token={}",
        std::env::var("TACHYON_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        token
    );

    send_email_webhook(
        &state.client,
        &body.email,
        "Tachyon Password Reset",
        &format!(
            "You requested a password reset. Click the link below to reset your password:\n\n{}\n\n\
             This link expires in 1 hour. If you did not request this, you can safely ignore this email.",
            reset_link
        ),
    )
    .await;

    info!(email = %body.email, "Password reset email sent");

    Ok(Json(MessageResponse {
        message: "If an account with that email exists, a password reset link has been sent.".to_string(),
    }))
}

pub async fn confirm_password_reset(
    State(state): State<PasswordResetState>,
    Json(body): Json<PasswordResetConfirm>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token_hash = hash_token(&body.token);

    let reset_repo = tachyon_database::PasswordResetRepository::new(state.pool.clone());
    let token = reset_repo
        .consume_reset_token(&token_hash)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "TOKEN_ERROR".to_string(),
                    message: format!("Failed to validate reset token: {}", e),
                }),
            )
        })?;

    let token = token.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "INVALID_TOKEN".to_string(),
                message: "Token is invalid, expired, or already used.".to_string(),
            }),
        )
    })?;

    let new_hash = tachyon_core::types::user::User::hash_password(&body.new_password)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "HASH_ERROR".to_string(),
                    message: format!("Failed to hash password: {}", e),
                }),
            )
        })?;

    let user_id = tachyon_core::id::UserId::parse_str(&token.user_id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "INVALID_USER_ID".to_string(),
                    message: format!("Invalid user ID: {}", e),
                }),
            )
        })?;

    let user_repo = tachyon_database::UserRepository::new(state.pool.clone());
    user_repo
        .update_password(&user_id, &new_hash)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "UPDATE_ERROR".to_string(),
                    message: format!("Failed to update password: {}", e),
                }),
            )
        })?;

    info!(user_id = %token.user_id, "Password reset successful");

    Ok(Json(MessageResponse {
        message: "Password has been reset successfully.".to_string(),
    }))
}

pub async fn request_email_verification(
    State(state): State<PasswordResetState>,
    headers: HeaderMap,
    Json(body): Json<EmailVerifyRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let jwt_secret = match std::env::var("TACHYON_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "CONFIG_ERROR".to_string(),
                    message: "JWT secret not configured".to_string(),
                }),
            ));
        }
    };

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    code: "UNAUTHORIZED".to_string(),
                    message: "Authentication required".to_string(),
                }),
            )
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                code: "UNAUTHORIZED".to_string(),
                message: "Invalid authorization header format".to_string(),
            }),
        ));
    }

    let token_str = &auth_header[7..];
    let validation = Validation::default();
    decode::<Claims>(token_str, &DecodingKey::from_secret(jwt_secret.as_bytes()), &validation)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    code: "UNAUTHORIZED".to_string(),
                    message: "Invalid or expired token".to_string(),
                }),
            )
        })?;

    let reset_repo = tachyon_database::PasswordResetRepository::new(state.pool.clone());
    let user_repo = tachyon_database::UserRepository::new(state.pool.clone());

    let user = match user_repo.get_by_email(&body.email).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(Json(MessageResponse {
                message: "If an account with that email exists, a verification link has been sent.".to_string(),
            }));
        }
    };

    let token = generate_token();
    let token_hash = hash_token(&token);

    reset_repo
        .create_verification_token(&user.id.as_str(), &body.email, &token_hash, 24)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "TOKEN_ERROR".to_string(),
                    message: format!("Failed to create verification token: {}", e),
                }),
            )
        })?;

    let verify_link = format!(
        "{}/verify-email?token={}",
        std::env::var("TACHYON_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        token
    );

    send_email_webhook(
        &state.client,
        &body.email,
        "Tachyon Email Verification",
        &format!(
            "Please verify your email address by clicking the link below:\n\n{}\n\n\
             This link expires in 24 hours.",
            verify_link
        ),
    )
    .await;

    info!(email = %body.email, "Email verification sent");

    Ok(Json(MessageResponse {
        message: "If an account with that email exists, a verification link has been sent.".to_string(),
    }))
}

pub async fn confirm_email_verification(
    State(state): State<PasswordResetState>,
    Json(body): Json<EmailVerifyConfirm>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token_hash = hash_token(&body.token);

    let reset_repo = tachyon_database::PasswordResetRepository::new(state.pool.clone());
    let token = reset_repo
        .consume_verification_token(&token_hash)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "TOKEN_ERROR".to_string(),
                    message: format!("Failed to validate verification token: {}", e),
                }),
            )
        })?;

    let token = token.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "INVALID_TOKEN".to_string(),
                message: "Token is invalid, expired, or already used.".to_string(),
            }),
        )
    })?;

    let user_id = tachyon_core::id::UserId::parse_str(&token.user_id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "INVALID_USER_ID".to_string(),
                    message: format!("Invalid user ID: {}", e),
                }),
            )
        })?;

    let user_repo = tachyon_database::UserRepository::new(state.pool.clone());
    user_repo
        .update(&user_id, None, Some(&token.email), None, None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "UPDATE_ERROR".to_string(),
                    message: format!("Failed to update email: {}", e),
                }),
            )
        })?;

    info!(user_id = %token.user_id, email = %token.email, "Email verified successfully");

    Ok(Json(MessageResponse {
        message: "Email verified successfully.".to_string(),
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
        let hash1 = hash_token("test-token");
        let hash2 = hash_token("test-token");
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
    fn test_message_response_serialization() {
        let resp = MessageResponse {
            message: "test".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("test"));
    }

    #[test]
    fn test_hash_token_empty_string() {
        let hash = hash_token("");
        assert_eq!(hash.len(), 64);
        let hash2 = hash_token("");
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_token_is_valid_hex() {
        let hash = hash_token("some-token-value");
        assert!(hex::decode(&hash).is_ok(), "Token hash should be valid hex");
    }

    #[test]
    fn test_generate_token_length() {
        let token = generate_token();
        assert_eq!(token.len(), 32, "UUID without dashes should be 32 chars");
    }

    #[test]
    fn test_generate_token_no_dashes() {
        let token = generate_token();
        assert!(!token.contains('-'));
    }

    #[test]
    fn test_error_response_serialization() {
        let resp = ErrorResponse {
            code: "INVALID_TOKEN".to_string(),
            message: "Token is invalid".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("INVALID_TOKEN"));
        assert!(json.contains("Token is invalid"));
    }

    #[test]
    fn test_password_reset_request_deserialization() {
        let json = r#"{"email":"user@example.com"}"#;
        let req: PasswordResetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "user@example.com");
    }

    #[test]
    fn test_password_reset_confirm_deserialization() {
        let json = r#"{"token":"sometoken","new_password":"NewPass123!"}"#;
        let req: PasswordResetConfirm = serde_json::from_str(json).unwrap();
        assert_eq!(req.token, "sometoken");
        assert_eq!(req.new_password, "NewPass123!");
    }

    #[test]
    fn test_email_verify_request_deserialization() {
        let json = r#"{"email":"user@example.com"}"#;
        let req: EmailVerifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "user@example.com");
    }

    #[test]
    fn test_email_verify_confirm_deserialization() {
        let json = r#"{"token":"verifytoken"}"#;
        let req: EmailVerifyConfirm = serde_json::from_str(json).unwrap();
        assert_eq!(req.token, "verifytoken");
    }

    #[test]
    fn test_message_response_roundtrip() {
        let original = MessageResponse {
            message: "Password reset sent".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["message"], original.message);
    }

    #[test]
    fn test_error_response_roundtrip() {
        let original = ErrorResponse {
            code: "TOKEN_ERROR".to_string(),
            message: "Failed to create token".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["code"], original.code);
        assert_eq!(parsed["message"], original.message);
    }

    #[test]
    fn test_hash_token_long_input() {
        let long_token = "a".repeat(1000);
        let hash = hash_token(&long_token);
        assert_eq!(hash.len(), 64);
    }
}
