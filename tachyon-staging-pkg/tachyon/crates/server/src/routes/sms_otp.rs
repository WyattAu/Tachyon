use crate::audit::{AuditEvent, AuditEventType, AuditLogger, AuditOutcome, AuditSeverity};
use crate::error::ServerError;
use axum::{Router, extract::State, response::Json, routing::post};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tachyon_database::{DatabasePool, RefreshTokenRepository, SmsOtpRepository, UserRepository};
use tracing::{info, warn};

use crate::routes::user::UserState;
use crate::routes::user::types::{
    AuthenticateResponse, REFRESH_TOKEN_EXPIRATION_SECS, UserResponse, hash_refresh_token,
};
use crate::sms::SmsProvider;

#[derive(Clone)]
pub struct SmsOtpRouteState {
    pub pool: DatabasePool,
    pub client: reqwest::Client,
    pub audit_logger: AuditLogger,
    pub jwt_secrets: Vec<String>,
    pub token_expiration_secs: u64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub ttl_secs: i64,
    pub sms_provider: Option<std::sync::Arc<dyn SmsProvider>>,
}

pub fn create_sms_otp_router() -> Router<SmsOtpRouteState> {
    Router::new()
        .route("/auth/sms-otp/request", post(request_sms_otp))
        .route("/auth/sms-otp/verify", post(verify_sms_otp))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SmsOtpRequest {
    pub phone: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SmsOtpVerify {
    pub phone: String,
    pub code: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SmsOtpMessageResponse {
    pub message: String,
}

fn is_valid_e164(phone: &str) -> bool {
    phone.len() >= 8
        && phone.len() <= 15
        && phone.starts_with('+')
        && phone[1..].chars().all(|c| c.is_ascii_digit())
}

fn generate_otp_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6).map(|_| rng.gen_range(0..=9).to_string()).collect()
}

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

/// POST /auth/sms-otp/request
/// Rate limit: 3 requests per minute per IP
#[utoipa::path(
    post,
    path = "/auth/sms-otp/request",
    request_body(content = SmsOtpRequest, description = "SMS OTP request"),
    responses(
        (status = 200, description = "If account exists, OTP sent", body = SmsOtpMessageResponse),
        (status = 400, description = "Invalid phone format"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth",
)]
pub async fn request_sms_otp(
    State(state): State<SmsOtpRouteState>,
    Json(body): Json<SmsOtpRequest>,
) -> Result<Json<SmsOtpMessageResponse>, ServerError> {
    let phone = body.phone.trim().to_string();

    if !is_valid_e164(&phone) {
        return Err(ServerError::bad_request(
            "Invalid phone format. Must be E.164 (e.g., +1234567890)",
        ));
    }

    let user_repo = UserRepository::new(state.pool.clone());

    let user = match user_repo.get_by_phone(&phone).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(Json(SmsOtpMessageResponse {
                message: "If an account exists with that phone, a code has been sent.".to_string(),
            }));
        }
    };

    let code = generate_otp_code();
    let code_hash = hash_code(&code);

    let sms_repo = SmsOtpRepository::new(state.pool.clone());
    sms_repo
        .create_token(&user.id.as_str(), &phone, &code_hash, state.ttl_secs, None)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to create SMS OTP token: {}", e)))?;

    if let Some(ref provider) = state.sms_provider {
        let message = format!(
            "Your Tachyon verification code is: {}. It expires in {} minutes. Do not share this code.",
            code,
            state.ttl_secs / 60
        );

        if let Err(e) = provider.send_sms(&phone, &message).await {
            warn!(phone = %phone, error = %e, "Failed to send SMS OTP, returning 500");
            return Err(ServerError::internal(format!(
                "Failed to send SMS verification code: {}",
                e
            )));
        }
    } else {
        info!(phone = %phone, "SMS provider not configured, OTP code not sent");
    }

    info!(phone = %phone, "SMS OTP requested");
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                AuditSeverity::Medium,
                "sms_otp_request",
                "SMS OTP login requested",
            )
            .with_metadata("phone", serde_json::json!(phone))
            .with_outcome(AuditOutcome::Success),
        )
        .await;

    Ok(Json(SmsOtpMessageResponse {
        message: "If an account exists with that phone, a code has been sent.".to_string(),
    }))
}

/// POST /auth/sms-otp/verify
/// Rate limit: 10 requests per minute per IP
#[utoipa::path(
    post,
    path = "/auth/sms-otp/verify",
    request_body(content = SmsOtpVerify, description = "SMS OTP verification"),
    responses(
        (status = 200, description = "Authenticated", body = AuthenticateResponse),
        (status = 400, description = "Invalid or expired code"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth",
)]
pub async fn verify_sms_otp(
    State(state): State<SmsOtpRouteState>,
    Json(body): Json<SmsOtpVerify>,
) -> Result<Json<AuthenticateResponse>, ServerError> {
    let phone = body.phone.trim().to_string();
    let code = body.code.trim().to_string();

    if !is_valid_e164(&phone) {
        return Err(ServerError::bad_request(
            "Invalid phone format. Must be E.164 (e.g., +1234567890)",
        ));
    }

    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServerError::bad_request("Code must be a 6-digit number"));
    }

    let code_hash = hash_code(&code);

    let sms_repo = SmsOtpRepository::new(state.pool.clone());
    let sms_token = sms_repo
        .consume_token(&code_hash)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to validate SMS OTP: {}", e)))?;

    let sms_token = sms_token
        .ok_or_else(|| ServerError::bad_request("Code is invalid, expired, or already used"))?;

    let user_repo = UserRepository::new(state.pool.clone());
    let user_id = tachyon_core::id::UserId::parse_str(&sms_token.user_id)
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
            "Failed to create refresh token during SMS OTP verify: {}",
            e
        );
    }

    info!(user_id = %user.id, "SMS OTP login successful");
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                AuditSeverity::Medium,
                "sms_otp_verify",
                format!("SMS OTP login for user '{}'", user.id),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_e164() {
        assert!(is_valid_e164("+1234567890"));
        assert!(is_valid_e164("+447911123456"));
        assert!(is_valid_e164("+15551234567"));
        assert!(!is_valid_e164("1234567890"));
        assert!(!is_valid_e164("+123"));
        assert!(!is_valid_e164("+abc1234567"));
        assert!(!is_valid_e164(""));
        assert!(!is_valid_e164("+1234567890123456"));
    }

    #[test]
    fn test_generate_otp_code_length() {
        let code = generate_otp_code();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_generate_otp_code_numeric() {
        let code = generate_otp_code();
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_otp_code_range() {
        for _ in 0..100 {
            let code = generate_otp_code();
            let num: u32 = code.parse().unwrap();
            assert!(num <= 999999);
        }
    }

    #[test]
    fn test_generate_otp_code_randomness() {
        let c1 = generate_otp_code();
        let c2 = generate_otp_code();
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_hash_code_deterministic() {
        let h1 = hash_code("123456");
        let h2 = hash_code("123456");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_hash_code_different_inputs() {
        let h1 = hash_code("000000");
        let h2 = hash_code("999999");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_code_is_valid_hex() {
        let hash = hash_code("654321");
        assert!(hex::decode(&hash).is_ok());
    }

    #[test]
    fn test_sms_otp_request_deserialization() {
        let json = r#"{"phone":"+1234567890"}"#;
        let req: SmsOtpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.phone, "+1234567890");
    }

    #[test]
    fn test_sms_otp_verify_deserialization() {
        let json = r#"{"phone":"+1234567890","code":"123456"}"#;
        let req: SmsOtpVerify = serde_json::from_str(json).unwrap();
        assert_eq!(req.phone, "+1234567890");
        assert_eq!(req.code, "123456");
    }

    #[test]
    fn test_sms_otp_message_response_serialization() {
        let resp = SmsOtpMessageResponse {
            message: "If an account exists, a code has been sent.".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("code has been sent"));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["message"], resp.message);
    }
}
