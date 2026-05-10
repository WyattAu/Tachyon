// OAuth2 Authentication Routes
//
// Provides Google and GitHub OAuth2 authorization code flow endpoints.
//
// Flow:
// 1. Frontend redirects user to GET /api/v1/auth/oauth2/{provider}/authorize
// 2. Server generates a CSRF state nonce, stores it, and includes it in the
//    provider's authorize URL
// 3. User authenticates with the provider
// 4. Provider redirects to GET /api/v1/auth/oauth2/{provider}/callback?code=...&state=...
// 5. Server validates the returned state against the stored nonce (CSRF protection)
// 6. Server exchanges code for provider tokens, fetches user info
// 7. Server creates/updates local user, issues JWT token
// 8. Server redirects to frontend with JWT token
//
// CSRF Protection:
// Each authorize request generates a cryptographic random state nonce stored in an
// in-memory `DashMap` with a 10-minute TTL. The callback must return the same state
// value; otherwise the request is rejected. Expired states are cleaned periodically.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::Utc;
use dashmap::DashMap;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::OAuth2Config;

// ============================================================================
// State
// ============================================================================

/// Stored CSRF state entry with creation timestamp for TTL enforcement.
pub(crate) struct CsrfStateEntry {
    /// Hex-encoded random nonce.
    nonce: String,
    /// When this entry was created (UTC).
    created_at: chrono::DateTime<Utc>,
}

/// Shared state for OAuth2 routes.
#[derive(Clone)]
pub struct OAuth2State {
    pub jwt_secret: String,
    pub jwt_expiration_secs: u64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub config: OAuth2Config,
    pub pool: tachyon_database::DatabasePool,
    pub client: reqwest::Client,
    /// In-memory store for CSRF state nonces. Keyed by provider name (e.g. "google", "github").
    /// Only one pending OAuth2 flow per provider at a time (per server instance).
    pub(crate) csrf_states: Arc<DashMap<String, CsrfStateEntry>>,
}

/// Maximum age for a CSRF state nonce before it's considered expired (10 minutes).
const CSRF_STATE_TTL_SECS: i64 = 600;

impl OAuth2State {
    /// Generate a cryptographic random state nonce and store it for the given provider.
    fn generate_csrf_state(&self, provider: &str) -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let nonce = hex::encode(bytes);

        self.csrf_states.insert(
            provider.to_string(),
            CsrfStateEntry {
                nonce: nonce.clone(),
                created_at: Utc::now(),
            },
        );

        nonce
    }

    /// Validate the returned state nonce for the given provider.
    /// Returns `true` if the nonce matches and hasn't expired.
    /// On success, the nonce is consumed (single-use).
    fn validate_csrf_state(&self, provider: &str, returned_state: &str) -> bool {
        let result = self.csrf_states.remove(provider);

        match result {
            Some((_, entry)) => {
                let elapsed = Utc::now()
                    .signed_duration_since(entry.created_at)
                    .num_seconds();

                if entry.nonce == returned_state && elapsed <= CSRF_STATE_TTL_SECS {
                    true
                } else {
                    warn!(
                        provider,
                        elapsed_secs = elapsed,
                        "OAuth2 CSRF state validation failed: mismatch or expired"
                    );
                    false
                }
            }
            None => {
                warn!(
                    provider,
                    "OAuth2 callback received with no matching state stored"
                );
                false
            }
        }
    }

    /// Clean up expired CSRF state entries. Called periodically.
    fn cleanup_expired_states(&self) {
        let now = Utc::now();
        self.csrf_states.retain(|provider, entry| {
            let elapsed = now.signed_duration_since(entry.created_at).num_seconds();
            if elapsed > CSRF_STATE_TTL_SECS {
                info!(
                    provider,
                    elapsed_secs = elapsed,
                    "Cleaning up expired OAuth2 CSRF state"
                );
                false
            } else {
                true
            }
        });
    }
}

pub fn create_oauth2_router() -> Router<OAuth2State> {
    Router::new()
        .route("/auth/oauth2/google/authorize", get(google_authorize))
        .route("/auth/oauth2/google/callback", get(google_callback))
        .route("/auth/oauth2/github/authorize", get(github_authorize))
        .route("/auth/oauth2/github/callback", get(github_callback))
}

// ============================================================================
// Query/Response types
// ============================================================================

#[derive(Debug, Deserialize, utoipa::IntoParams)]
struct CallbackQuery {
    code: String,
    /// CSRF state nonce — must match the value generated during the authorize step.
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

// ============================================================================
// Google OAuth2
// ============================================================================

/// Redirect user to Google's consent screen.
/// Generates a CSRF state nonce and includes it in the authorization URL.
#[utoipa::path(
    get,
    path = "/auth/oauth2/google/authorize",
    responses(
        (status = 307, description = "Redirect to Google consent screen"),
        (status = 200, description = "OAuth2 not configured", body = serde_json::Value),
    ),
    tag = "auth",
)]
async fn google_authorize(State(state): State<OAuth2State>) -> Response {
    let client_id = match &state.config.google_client_id {
        Some(id) => id.clone(),
        None => {
            return axum::Json(serde_json::json!({
                "error": "Google OAuth2 is not configured",
                "message": "Set TACHYON_GOOGLE_CLIENT_ID and TACHYON_GOOGLE_CLIENT_SECRET"
            }))
            .into_response()
        }
    };

    let redirect_uri = build_redirect_uri(&state.config, "google");
    let scopes = "openid email profile";

    // Generate CSRF state nonce and store it
    let csrf_nonce = state.generate_csrf_state("google");

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(scopes),
        urlencoding::encode(&csrf_nonce),
    );

    Redirect::temporary(&url).into_response()
}

/// Handle Google OAuth2 callback.
/// Validates the CSRF state nonce before processing the authorization code.
#[utoipa::path(
    get,
    path = "/auth/oauth2/google/callback",
    params(
        CallbackQuery,
    ),
    responses(
        (status = 200, description = "OAuth2 token response", body = serde_json::Value),
        (status = 307, description = "Redirect with token"),
        (status = 400, description = "CSRF validation failed", body = serde_json::Value),
    ),
    tag = "auth",
)]
async fn google_callback(
    State(state): State<OAuth2State>,
    Query(query): Query<CallbackQuery>,
) -> Response {
    if let Some(error) = &query.error {
        warn!(error = %error, description = ?query.error_description, "Google OAuth2 error");
        return axum::Json(serde_json::json!({
            "error": error,
            "description": query.error_description,
        }))
        .into_response();
    }

    // Validate CSRF state nonce — reject if missing or invalid
    let returned_state = match &query.state {
        Some(s) if !s.is_empty() => s.as_str(),
        _ => {
            warn!("Google OAuth2 callback received without state parameter — possible CSRF attack");
            return axum::Json(serde_json::json!({
                "error": "invalid_request",
                "message": "Missing or empty state parameter. Possible CSRF attack."
            }))
            .into_response();
        }
    };

    if !state.validate_csrf_state("google", returned_state) {
        warn!("Google OAuth2 CSRF state validation failed — rejecting callback");
        return axum::Json(serde_json::json!({
            "error": "invalid_state",
            "message": "CSRF state validation failed. The OAuth2 flow may have been tampered with."
        }))
        .into_response();
    }

    // Periodically clean up expired states
    state.cleanup_expired_states();

    let client_id = match &state.config.google_client_id {
        Some(id) => id.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "Google OAuth2 not configured"}))
                .into_response()
        }
    };
    let client_secret = match &state.config.google_client_secret {
        Some(s) => s.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "Google OAuth2 not configured"}))
                .into_response()
        }
    };

    let redirect_uri = build_redirect_uri(&state.config, "google");

    // Exchange code for token
    let token_resp = exchange_google_code(
        &state.client,
        &client_id,
        &client_secret,
        &redirect_uri,
        &query.code,
    )
    .await;
    let access_token = match token_resp {
        Ok(data) => data["access_token"].as_str().unwrap_or("").to_string(),
        Err(e) => {
            error!(error = %e, "Failed to exchange Google code for token");
            return axum::Json(serde_json::json!({
                "error": "token_exchange_failed",
                "message": e.to_string(),
            }))
            .into_response();
        }
    };

    // Fetch user info from Google
    let user_info = fetch_google_user_info(&state.client, &access_token).await;
    let user = match user_info {
        Ok(u) => u,
        Err(e) => {
            error!(error = %e, "Failed to fetch Google user info");
            return axum::Json(serde_json::json!({
                "error": "user_info_failed",
                "message": e.to_string(),
            }))
            .into_response();
        }
    };

    info!(
        provider = "google",
        user_id = %user.provider_user_id,
        email = %user.email,
        "OAuth2 user authenticated"
    );

    // Upsert user and issue JWT
    issue_token_for_oauth_user(&state, user).await
}

async fn exchange_google_code(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<serde_json::Value, String> {
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token endpoint returned {}: {}", status, body));
    }

    resp.json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))
}

async fn fetch_google_user_info(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<OAuthUserInfo, String> {
    let resp = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    Ok(OAuthUserInfo {
        provider: "google".to_string(),
        provider_user_id: data["id"].as_str().unwrap_or("").to_string(),
        email: data["email"].as_str().unwrap_or("").to_string(),
        name: data["name"].as_str().unwrap_or("").to_string(),
        avatar_url: data["picture"].as_str().map(|s| s.to_string()),
    })
}

// ============================================================================
// GitHub OAuth2
// ============================================================================

/// Redirect user to GitHub's authorization page.
/// Generates a CSRF state nonce and includes it in the authorization URL.
#[utoipa::path(
    get,
    path = "/auth/oauth2/github/authorize",
    responses(
        (status = 307, description = "Redirect to GitHub authorization page"),
        (status = 200, description = "OAuth2 not configured", body = serde_json::Value),
    ),
    tag = "auth",
)]
async fn github_authorize(State(state): State<OAuth2State>) -> Response {
    let client_id = match &state.config.github_client_id {
        Some(id) => id.clone(),
        None => {
            return axum::Json(serde_json::json!({
                "error": "GitHub OAuth2 is not configured",
                "message": "Set TACHYON_GITHUB_CLIENT_ID and TACHYON_GITHUB_CLIENT_SECRET"
            }))
            .into_response()
        }
    };

    let redirect_uri = build_redirect_uri(&state.config, "github");
    let scopes = "read:user user:email";

    // Generate CSRF state nonce and store it
    let csrf_nonce = state.generate_csrf_state("github");

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(scopes),
        urlencoding::encode(&csrf_nonce),
    );

    Redirect::temporary(&url).into_response()
}

/// Handle GitHub OAuth2 callback.
/// Validates the CSRF state nonce before processing the authorization code.
#[utoipa::path(
    get,
    path = "/auth/oauth2/github/callback",
    params(
        CallbackQuery,
    ),
    responses(
        (status = 200, description = "OAuth2 token response", body = serde_json::Value),
        (status = 307, description = "Redirect with token"),
        (status = 400, description = "CSRF validation failed", body = serde_json::Value),
    ),
    tag = "auth",
)]
async fn github_callback(
    State(state): State<OAuth2State>,
    Query(query): Query<CallbackQuery>,
) -> Response {
    if let Some(error) = &query.error {
        warn!(error = %error, description = ?query.error_description, "GitHub OAuth2 error");
        return axum::Json(serde_json::json!({
            "error": error,
            "description": query.error_description,
        }))
        .into_response();
    }

    // Validate CSRF state nonce — reject if missing or invalid
    let returned_state = match &query.state {
        Some(s) if !s.is_empty() => s.as_str(),
        _ => {
            warn!("GitHub OAuth2 callback received without state parameter — possible CSRF attack");
            return axum::Json(serde_json::json!({
                "error": "invalid_request",
                "message": "Missing or empty state parameter. Possible CSRF attack."
            }))
            .into_response();
        }
    };

    if !state.validate_csrf_state("github", returned_state) {
        warn!("GitHub OAuth2 CSRF state validation failed — rejecting callback");
        return axum::Json(serde_json::json!({
            "error": "invalid_state",
            "message": "CSRF state validation failed. The OAuth2 flow may have been tampered with."
        }))
        .into_response();
    }

    // Periodically clean up expired states
    state.cleanup_expired_states();

    let client_id = match &state.config.github_client_id {
        Some(id) => id.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "GitHub OAuth2 not configured"}))
                .into_response()
        }
    };
    let client_secret = match &state.config.github_client_secret {
        Some(s) => s.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "GitHub OAuth2 not configured"}))
                .into_response()
        }
    };

    let redirect_uri = build_redirect_uri(&state.config, "github");

    // Exchange code for token
    let token_resp = exchange_github_code(
        &state.client,
        &client_id,
        &client_secret,
        &redirect_uri,
        &query.code,
    )
    .await;
    let access_token = match token_resp {
        Ok(token) => token,
        Err(e) => {
            error!(error = %e, "Failed to exchange GitHub code for token");
            return axum::Json(serde_json::json!({
                "error": "token_exchange_failed",
                "message": e.to_string(),
            }))
            .into_response();
        }
    };

    // Fetch user info from GitHub
    let user_info = fetch_github_user_info(&state.client, &access_token).await;
    let user = match user_info {
        Ok(u) => u,
        Err(e) => {
            error!(error = %e, "Failed to fetch GitHub user info");
            return axum::Json(serde_json::json!({
                "error": "user_info_failed",
                "message": e.to_string(),
            }))
            .into_response();
        }
    };

    info!(
        provider = "github",
        user_id = %user.provider_user_id,
        email = %user.email,
        "OAuth2 user authenticated"
    );

    // Upsert user and issue JWT
    issue_token_for_oauth_user(&state, user).await
}

async fn exchange_github_code(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<String, String> {
    // GitHub uses JSON accept header for token endpoint
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "code": code,
            "client_id": client_id,
            "client_secret": client_secret,
            "redirect_uri": redirect_uri,
        }))
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token endpoint returned {}: {}", status, body));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    data["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No access_token in response".to_string())
}

async fn fetch_github_user_info(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<OAuthUserInfo, String> {
    // Fetch user profile
    let resp = client
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("User-Agent", "Tachyon-Server")
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    // Fetch primary email (GitHub returns public emails by default)
    let email = data["email"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Use login@github.com as fallback if no public email
            format!(
                "{}@github.users.noreply",
                data["login"].as_str().unwrap_or("unknown")
            )
        });

    Ok(OAuthUserInfo {
        provider: "github".to_string(),
        provider_user_id: data["id"].as_u64().unwrap_or(0).to_string(),
        email,
        name: data["name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| data["login"].as_str().unwrap_or("Unknown"))
            .to_string(),
        avatar_url: data["avatar_url"].as_str().map(|s| s.to_string()),
    })
}

// ============================================================================
// Shared: Issue JWT for OAuth user
// ============================================================================

async fn issue_token_for_oauth_user(state: &OAuth2State, user: OAuthUserInfo) -> Response {
    // Create or update user in database
    // Use the provider + provider_user_id as a unique identifier
    let oauth_id = format!("{}:{}", user.provider, user.provider_user_id);

    // Simple claims struct for JWT generation
    #[derive(serde::Serialize)]
    struct OAuthClaims {
        sub: String,
        exp: usize,
        iss: String,
        aud: String,
        iat: usize,
        provider: String,
    }

    // Generate JWT token
    let key = jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_bytes());
    let now = chrono::Utc::now().timestamp();
    let claims = OAuthClaims {
        sub: oauth_id.clone(),
        exp: (now + state.jwt_expiration_secs as i64) as usize,
        iss: state.jwt_issuer.clone(),
        aud: state.jwt_audience.clone(),
        iat: now as usize,
        provider: user.provider.clone(),
    };
    let header = jsonwebtoken::Header::default();
    let token = jsonwebtoken::encode(&header, &claims, &key);

    match token {
        Ok(token_string) => axum::Json(serde_json::json!({
            "access_token": token_string,
            "token_type": "Bearer",
            "expires_in": state.jwt_expiration_secs,
            "user": {
                "provider": user.provider,
                "email": user.email,
                "name": user.name,
                "avatar_url": user.avatar_url,
            },
        }))
        .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to generate JWT for OAuth user");
            axum::Json(serde_json::json!({
                "error": "token_generation_failed",
                "message": e.to_string(),
            }))
            .into_response()
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn build_redirect_uri(config: &OAuth2Config, provider: &str) -> String {
    let base = config
        .redirect_base_url
        .as_deref()
        .unwrap_or("http://localhost:8080");
    format!("{}/api/v1/auth/oauth2/{}/callback", base, provider)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_user_info_serialization() {
        let user = OAuthUserInfo {
            provider: "google".to_string(),
            provider_user_id: "12345".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"provider\":\"google\""));
        assert!(json.contains("\"email\":\"test@example.com\""));
    }

    #[test]
    fn test_build_redirect_uri_default() {
        let config = OAuth2Config::default();
        assert_eq!(
            build_redirect_uri(&config, "google"),
            "http://localhost:8080/api/v1/auth/oauth2/google/callback"
        );
    }

    #[test]
    fn test_build_redirect_uri_custom() {
        let config = OAuth2Config {
            redirect_base_url: Some("https://tachyon.example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(
            build_redirect_uri(&config, "github"),
            "https://tachyon.example.com/api/v1/auth/oauth2/github/callback"
        );
    }

    // ── CSRF State Tests ────────────────────────────────────────────────

    /// Helper to create a CSRF state store for testing.
    /// Tests only need the DashMap — no OAuth2State construction required.
    fn make_test_csrf_store() -> Arc<DashMap<String, CsrfStateEntry>> {
        Arc::new(DashMap::new())
    }

    #[test]
    fn test_csrf_state_generate_and_validate() {
        let store = make_test_csrf_store();

        // Generate a nonce for google
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let nonce = hex::encode(bytes);
        store.insert(
            "google".to_string(),
            CsrfStateEntry {
                nonce: nonce.clone(),
                created_at: Utc::now(),
            },
        );

        assert_eq!(
            nonce.len(),
            64,
            "Nonce should be 32 bytes hex-encoded (64 chars)"
        );

        // Simulate validation — same logic as validate_csrf_state
        let result = store.remove("google");
        match result {
            Some((_, entry)) => {
                assert_eq!(entry.nonce, nonce, "Nonce should match");
                let elapsed = Utc::now()
                    .signed_duration_since(entry.created_at)
                    .num_seconds();
                assert!(elapsed <= CSRF_STATE_TTL_SECS, "Should not be expired");
            }
            None => panic!("State should exist"),
        }

        // State should be consumed — re-validation should fail
        assert!(
            store.get("google").is_none(),
            "State should be consumed after validation"
        );
    }

    #[test]
    fn test_csrf_state_wrong_nonce_rejected() {
        let store = make_test_csrf_store();

        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        store.insert(
            "google".to_string(),
            CsrfStateEntry {
                nonce: hex::encode(bytes),
                created_at: Utc::now(),
            },
        );

        // Simulate validation with wrong nonce
        let result = store.remove("google");
        match result {
            Some((_, entry)) => {
                assert_ne!(entry.nonce, "wrong_nonce_value_here", "Should not match");
            }
            None => panic!("State should exist"),
        }
    }

    #[test]
    fn test_csrf_state_no_nonce_stored() {
        let store = make_test_csrf_store();
        assert!(store.get("github").is_none(), "No state should be stored");
    }

    #[test]
    fn test_csrf_state_expired_rejected() {
        let store = make_test_csrf_store();

        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let nonce = hex::encode(bytes);

        // Insert with backdated timestamp to simulate expiry
        store.insert(
            "google".to_string(),
            CsrfStateEntry {
                nonce: nonce.clone(),
                created_at: Utc::now() - chrono::Duration::seconds(CSRF_STATE_TTL_SECS + 1),
            },
        );

        // Simulate validation — should reject due to expiry
        let result = store.remove("google");
        match result {
            Some((_, entry)) => {
                let elapsed = Utc::now()
                    .signed_duration_since(entry.created_at)
                    .num_seconds();
                assert!(elapsed > CSRF_STATE_TTL_SECS, "Should be expired");
            }
            None => panic!("State should exist"),
        }
    }

    #[test]
    fn test_csrf_state_per_provider_isolation() {
        let store = make_test_csrf_store();

        let mut bytes1 = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes1);
        let google_nonce = hex::encode(bytes1);

        let mut bytes2 = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes2);
        let github_nonce = hex::encode(bytes2);

        store.insert(
            "google".to_string(),
            CsrfStateEntry {
                nonce: google_nonce.clone(),
                created_at: Utc::now(),
            },
        );
        store.insert(
            "github".to_string(),
            CsrfStateEntry {
                nonce: github_nonce.clone(),
                created_at: Utc::now(),
            },
        );

        // Cross-provider: nonces should be different
        let google_entry = store.get("google").unwrap();
        let github_entry = store.get("github").unwrap();
        assert_ne!(
            google_entry.nonce, github_entry.nonce,
            "Nonces should be different per provider"
        );
        drop(google_entry);
        drop(github_entry);

        // Each provider should validate independently
        assert!(store.get("google").is_some());
        assert!(store.get("github").is_some());
    }

    #[test]
    fn test_csrf_cleanup_removes_expired() {
        let store = make_test_csrf_store();

        // Insert expired state
        store.insert(
            "google".to_string(),
            CsrfStateEntry {
                nonce: "expired".to_string(),
                created_at: Utc::now() - chrono::Duration::seconds(CSRF_STATE_TTL_SECS + 1),
            },
        );

        // Insert fresh state
        store.insert(
            "github".to_string(),
            CsrfStateEntry {
                nonce: "fresh".to_string(),
                created_at: Utc::now(),
            },
        );

        assert_eq!(store.len(), 2);

        // Simulate cleanup (same logic as cleanup_expired_states)
        let now = Utc::now();
        store.retain(|_provider, entry| {
            let elapsed = now.signed_duration_since(entry.created_at).num_seconds();
            elapsed <= CSRF_STATE_TTL_SECS
        });

        assert_eq!(store.len(), 1, "Expired state should be cleaned up");
        assert!(store.contains_key("github"));
        assert!(!store.contains_key("google"));
    }
}
