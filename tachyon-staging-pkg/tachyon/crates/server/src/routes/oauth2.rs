// OAuth2 Authentication Routes
//
// Provides Google and GitHub OAuth2 authorization code flow endpoints.
//
// Flow:
// 1. Frontend redirects user to GET /api/v1/auth/oauth2/{provider}/authorize
// 2. Server generates a CSRF state nonce, stores it in Redis/DashMap, and includes it in the
//    provider's authorize URL
// 3. User authenticates with the provider
// 4. Provider redirects to GET /api/v1/auth/oauth2/{provider}/callback?code=...&state=...
// 5. Server validates the returned state against the stored nonce (CSRF protection)
// 6. Server exchanges code for provider tokens, fetches user info
// 7. Server creates/updates local user, issues JWT token
// 8. Server redirects to frontend with JWT token
//
// CSRF Protection:
// Each authorize request generates a cryptographic random state nonce stored via
// CsrfStore (Redis-backed when available, in-memory DashMap fallback) with a
// 10-minute TTL. The callback must return the same state value; otherwise the
// request is rejected.

use axum::{
    Router,
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::config::OAuth2Config;
use crate::csrf_store::CsrfStore;

// ============================================================================
// State
// ============================================================================

/// Shared state for OAuth2 routes.
#[derive(Clone)]
pub struct OAuth2State {
    pub jwt_secrets: Vec<String>,
    pub jwt_expiration_secs: u64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub config: OAuth2Config,
    pub pool: tachyon_database::DatabasePool,
    pub client: reqwest::Client,
    /// CSRF state store. Keyed by provider name (e.g. "google", "github").
    /// Backed by Redis when available, falls back to in-memory DashMap.
    pub(crate) csrf_store: crate::csrf_store::CsrfStoreType,
}

impl OAuth2State {
    /// Generate a cryptographic random state nonce and store it for the given provider.
    async fn generate_csrf_state(&self, provider: &str) -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let nonce = hex::encode(bytes);

        self.csrf_store.store(provider, &nonce, None).await;

        nonce
    }

    /// Validate the returned state nonce for the given provider.
    /// Returns `true` if the nonce matches. On success, the nonce is consumed (single-use).
    async fn validate_csrf_state(&self, provider: &str, returned_state: &str) -> bool {
        match self.csrf_store.retrieve_and_consume(provider).await {
            Some((stored_nonce, _redirect_url)) => {
                if stored_nonce == returned_state {
                    true
                } else {
                    warn!(
                        provider,
                        "OAuth2 CSRF state validation failed: nonce mismatch"
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
            .into_response();
        }
    };

    let redirect_uri = build_redirect_uri(&state.config, "google");
    let scopes = "openid email profile";

    // Generate CSRF state nonce and store it
    let csrf_nonce = state.generate_csrf_state("google").await;

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

    if !state.validate_csrf_state("google", returned_state).await {
        warn!("Google OAuth2 CSRF state validation failed — rejecting callback");
        return axum::Json(serde_json::json!({
            "error": "invalid_state",
            "message": "CSRF state validation failed. The OAuth2 flow may have been tampered with."
        }))
        .into_response();
    }

    // Periodic cleanup (no-op for Redis)
    state.csrf_store.cleanup_expired().await;

    let client_id = match &state.config.google_client_id {
        Some(id) => id.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "Google OAuth2 not configured"}))
                .into_response();
        }
    };
    let client_secret = match &state.config.google_client_secret {
        Some(s) => s.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "Google OAuth2 not configured"}))
                .into_response();
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
            .into_response();
        }
    };

    let redirect_uri = build_redirect_uri(&state.config, "github");
    let scopes = "read:user user:email";

    // Generate CSRF state nonce and store it
    let csrf_nonce = state.generate_csrf_state("github").await;

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

    if !state.validate_csrf_state("github", returned_state).await {
        warn!("GitHub OAuth2 CSRF state validation failed — rejecting callback");
        return axum::Json(serde_json::json!({
            "error": "invalid_state",
            "message": "CSRF state validation failed. The OAuth2 flow may have been tampered with."
        }))
        .into_response();
    }

    // Periodic cleanup (no-op for Redis)
    state.csrf_store.cleanup_expired().await;

    let client_id = match &state.config.github_client_id {
        Some(id) => id.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "GitHub OAuth2 not configured"}))
                .into_response();
        }
    };
    let client_secret = match &state.config.github_client_secret {
        Some(s) => s.clone(),
        None => {
            return axum::Json(serde_json::json!({"error": "GitHub OAuth2 not configured"}))
                .into_response();
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
    let signing_secret = state.jwt_secrets.first().map(|s| s.as_str()).unwrap_or("");
    let key = jsonwebtoken::EncodingKey::from_secret(signing_secret.as_bytes());
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

    /// Helper to create a CSRF store for testing.
    fn make_test_csrf_store() -> crate::csrf_store::CsrfStoreType {
        crate::csrf_store::CsrfStoreType::Memory(crate::csrf_store::MemoryCsrfStore::new())
    }

    #[tokio::test]
    async fn test_csrf_state_generate_and_validate() {
        let store = make_test_csrf_store();

        // Generate a nonce for google
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let nonce = hex::encode(bytes);

        store.store("google", &nonce, None).await;

        assert_eq!(
            nonce.len(),
            64,
            "Nonce should be 32 bytes hex-encoded (64 chars)"
        );

        // Retrieve and consume
        let result = store.retrieve_and_consume("google").await;
        assert!(result.is_some());
        let (retrieved_nonce, _redirect) = result.unwrap();
        assert_eq!(retrieved_nonce, nonce, "Nonce should match");

        // State should be consumed -- re-validation should fail
        let result = store.retrieve_and_consume("google").await;
        assert!(
            result.is_none(),
            "State should be consumed after validation"
        );
    }

    #[tokio::test]
    async fn test_csrf_state_wrong_nonce_rejected() {
        let store = make_test_csrf_store();

        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        store.store("google", &hex::encode(bytes), None).await;

        // Retrieve and compare with wrong nonce
        let result = store.retrieve_and_consume("google").await;
        assert!(result.is_some());
        let (retrieved_nonce, _) = result.unwrap();
        assert_ne!(
            retrieved_nonce, "wrong_nonce_value_here",
            "Should not match"
        );
    }

    #[tokio::test]
    async fn test_csrf_state_no_nonce_stored() {
        let store = make_test_csrf_store();
        let result = store.retrieve_and_consume("github").await;
        assert!(result.is_none(), "No state should be stored");
    }

    #[tokio::test]
    async fn test_csrf_state_expired_rejected() {
        let store = make_test_csrf_store();
        // Insert directly with backdated timestamp to simulate expiry
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let nonce = hex::encode(bytes);

        // Use the internal DashMap directly to backdate
        if let crate::csrf_store::CsrfStoreType::Memory(mem) = &store {
            mem.entries.insert(
                "google".to_string(),
                (
                    nonce.clone(),
                    None,
                    chrono::Utc::now() - chrono::Duration::seconds(601),
                ),
            );
        }

        // Should return None because expired
        let result = store.retrieve_and_consume("google").await;
        assert!(result.is_none(), "Expired state should be rejected");
    }

    #[tokio::test]
    async fn test_csrf_state_per_provider_isolation() {
        let store = make_test_csrf_store();

        let mut bytes1 = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes1);
        let google_nonce = hex::encode(bytes1);

        let mut bytes2 = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes2);
        let github_nonce = hex::encode(bytes2);

        store.store("google", &google_nonce, None).await;
        store.store("github", &github_nonce, None).await;

        // Retrieve both
        let g = store.retrieve_and_consume("google").await.unwrap();
        let h = store.retrieve_and_consume("github").await.unwrap();
        assert_ne!(g.0, h.0, "Nonces should be different per provider");
    }

    #[tokio::test]
    async fn test_csrf_cleanup_removes_expired() {
        let store = make_test_csrf_store();

        if let crate::csrf_store::CsrfStoreType::Memory(mem) = &store {
            // Insert expired state
            mem.entries.insert(
                "google".to_string(),
                (
                    "expired".to_string(),
                    None,
                    chrono::Utc::now() - chrono::Duration::seconds(601),
                ),
            );
            // Insert fresh state
            mem.entries.insert(
                "github".to_string(),
                ("fresh".to_string(), None, chrono::Utc::now()),
            );
            assert_eq!(mem.entries.len(), 2);
        }

        store.cleanup_expired().await;

        if let crate::csrf_store::CsrfStoreType::Memory(mem) = &store {
            assert_eq!(mem.entries.len(), 1, "Expired state should be cleaned up");
            assert!(mem.entries.contains_key("github"));
            assert!(!mem.entries.contains_key("google"));
        }
    }
}
