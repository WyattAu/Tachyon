//! OpenID Connect runtime flow.
//!
//! Provides the authorize/callback flow for OIDC authentication:
//! 1. `GET /auth/sso/oidc/{provider}/authorize` -- redirect user to IdP
//! 2. `GET /auth/sso/oidc/{provider}/callback` -- exchange code, validate, create user, issue JWT

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::oidc::{OidcConfig, OidcDiscovery, OidcTokenResponse, OidcUserInfo};
use crate::csrf_store::CsrfStore;
use crate::error::ServerError;

// ============================================================================
// State
// ============================================================================

/// State for OIDC SSO operations.
#[derive(Clone)]
pub struct OidcState {
    pub configs: std::collections::HashMap<String, OidcConfig>,
    pub pool: tachyon_database::DatabasePool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub(crate) csrf_store: crate::csrf_store::CsrfStoreType,
}

impl OidcState {
    async fn generate_csrf_state(&self, state_token: &str, redirect_url: Option<String>) {
        self.csrf_store.store(state_token, state_token, redirect_url).await;
    }

    async fn validate_csrf_state(&self, returned_state: &str) -> Result<Option<String>, ServerError> {
        match self.csrf_store.retrieve_and_consume(returned_state).await {
            Some((nonce, redirect_url)) => {
                if nonce == returned_state {
                    Ok(redirect_url)
                } else {
                    warn!(
                        "OIDC CSRF state validation failed: nonce mismatch"
                    );
                    Err(ServerError::bad_request(
                        "Invalid or expired CSRF state parameter",
                    ))
                }
            }
            None => {
                warn!("OIDC callback received with no matching state stored");
                Err(ServerError::bad_request(
                    "Invalid or expired CSRF state parameter",
                ))
            }
        }
    }
}

// ============================================================================
// Query / Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OidcAuthorizeQuery {
    pub redirect_url: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OidcAuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OidcCallbackResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: OidcUserInfo,
}

// ============================================================================
// Handlers
// ============================================================================

/// OIDC authorize: redirect user to the identity provider.
///
/// `GET /api/v1/auth/sso/oidc/{provider}/authorize`
#[utoipa::path(
    get,
    path = "/api/v1/auth/sso/oidc/{provider}/authorize",
    params(
        ("provider" = String, Path, description = "OIDC provider name"),
        ("redirect_url" = Option<String>, Query, description = "Post-login redirect URL"),
        ("state" = Option<String>, Query, description = "CSRF state parameter"),
    ),
    responses(
        (status = 200, description = "Authorization URL and state", body = OidcAuthorizeResponse),
        (status = 404, description = "Provider not configured"),
    ),
    tag = "auth",
)]
pub async fn oidc_authorize(
    Path(provider): Path<String>,
    Query(_params): Query<OidcAuthorizeQuery>,
    State(state): State<OidcState>,
) -> Result<Json<OidcAuthorizeResponse>, ServerError> {
    info!("OIDC authorize request for provider: {}", provider);

    let config = state.configs.get(&provider).ok_or_else(|| {
        ServerError::not_found(
            "SSO Provider",
            &format!("No OIDC config for '{}'", provider),
        )
    })?;

    // Periodic cleanup (no-op for Redis which handles TTL natively)
    state.csrf_store.cleanup_expired().await;

    // Fetch discovery document
    let discovery = fetch_discovery(&state.http_client, &config.discovery_url).await?;

    // Build authorization URL
    let state_token = uuid::Uuid::new_v4().to_string();
    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        discovery.authorization_endpoint,
        config.client_id,
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(&config.scope.join(" ")),
        urlencoding::encode(&state_token),
    );

    // Store CSRF state with 10-minute TTL
    state.generate_csrf_state(&state_token, _params.redirect_url.clone()).await;

    debug!("Generated OIDC auth URL for {}", provider);

    Ok(Json(OidcAuthorizeResponse {
        authorization_url: auth_url,
        state: state_token,
    }))
}

/// OIDC callback: exchange authorization code for tokens, validate, create/update user.
///
/// `GET /api/v1/auth/sso/oidc/{provider}/callback`
#[utoipa::path(
    get,
    path = "/api/v1/auth/sso/oidc/{provider}/callback",
    params(
        ("provider" = String, Path, description = "OIDC provider name"),
        ("code" = String, Query, description = "Authorization code from IdP"),
        ("state" = Option<String>, Query, description = "CSRF state parameter"),
    ),
    responses(
        (status = 200, description = "Authentication successful", body = OidcCallbackResponse),
        (status = 400, description = "OAuth error or invalid code"),
        (status = 404, description = "Provider not configured"),
    ),
    tag = "auth",
)]
pub async fn oidc_callback(
    Path(provider): Path<String>,
    Query(params): Query<OidcCallbackQuery>,
    State(state): State<OidcState>,
) -> Result<Json<OidcCallbackResponse>, ServerError> {
    info!("OIDC callback for provider: {}", provider);

    // Check for IdP-reported errors
    if let Some(error) = &params.error {
        return Err(ServerError::bad_request(format!(
            "OIDC error: {}{}",
            error,
            params
                .error_description
                .as_ref()
                .map(|d| format!(": {}", d))
                .unwrap_or_default()
        )));
    }

    // Validate CSRF state
    let returned_state = params.state.as_deref().ok_or_else(|| {
        ServerError::bad_request("Missing required state parameter for CSRF validation")
    })?;
    state.validate_csrf_state(returned_state).await?;

    let config = state.configs.get(&provider).ok_or_else(|| {
        ServerError::not_found(
            "SSO Provider",
            &format!("No OIDC config for '{}'", provider),
        )
    })?;

    // Fetch discovery document
    let discovery = fetch_discovery(&state.http_client, &config.discovery_url).await?;

    // Exchange authorization code for tokens
    let token_response = exchange_code(
        &state.http_client,
        &discovery.token_endpoint,
        config,
        &params.code,
    )
    .await?;

    // Fetch user info
    let user_info = fetch_user_info(
        &state.http_client,
        &discovery.userinfo_endpoint,
        &token_response.access_token,
    )
    .await
    .unwrap_or_else(|e| {
        warn!("Failed to fetch userinfo, using fallback: {}", e);
        OidcUserInfo {
            sub: "unknown".to_string(),
            email: None,
            name: None,
            given_name: None,
            family_name: None,
            picture: None,
        }
    });

    debug!(
        "OIDC user authenticated: sub={}, email={:?}",
        user_info.sub, user_info.email
    );

    // Upsert user in database
    upsert_sso_user(&state.pool, &provider, &user_info).await?;

    // Issue JWT using jsonwebtoken (same approach as UserState::generate_jwt)
    let now = jsonwebtoken::get_current_timestamp();
    let exp = now + 3600;

    let claims = jsonwebtoken::Header {
        alg: jsonwebtoken::Algorithm::HS256,
        ..Default::default()
    };

    let token_claims = serde_json::json!({
        "sub": user_info.sub,
        "email": user_info.email,
        "name": user_info.name,
        "role": "user",
        "iss": "tachyon",
        "aud": "tachyon-api",
        "exp": exp,
        "iat": now,
        "sso_provider": format!("oidc:{}", provider),
    });

    let token = jsonwebtoken::encode(
        &claims,
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| ServerError::internal(format!("JWT creation failed: {}", e)))?;

    let expires_in = token_response.expires_in.unwrap_or(3600);

    Ok(Json(OidcCallbackResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: user_info,
    }))
}

// ============================================================================
// Internal helpers
// ============================================================================

async fn fetch_discovery(
    client: &reqwest::Client,
    discovery_url: &str,
) -> Result<OidcDiscovery, ServerError> {
    let url = if discovery_url.ends_with('/') {
        format!("{}.well-known/openid-configuration", discovery_url)
    } else {
        format!("{}/.well-known/openid-configuration", discovery_url)
    };

    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ServerError::bad_request(format!("Discovery request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerError::bad_request(format!(
            "Discovery endpoint returned status {}",
            response.status()
        )));
    }

    response
        .json::<OidcDiscovery>()
        .await
        .map_err(|e| ServerError::bad_request(format!("Failed to parse discovery document: {}", e)))
}

async fn exchange_code(
    client: &reqwest::Client,
    token_endpoint: &str,
    config: &OidcConfig,
    code: &str,
) -> Result<OidcTokenResponse, ServerError> {
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &config.redirect_uri),
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
    ];

    let response = client
        .post(token_endpoint)
        .form(&params)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ServerError::bad_request(format!("Token exchange failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ServerError::bad_request(format!(
            "Token endpoint returned {}: {}",
            status, body
        )));
    }

    response
        .json::<OidcTokenResponse>()
        .await
        .map_err(|e| ServerError::bad_request(format!("Failed to parse token response: {}", e)))
}

async fn fetch_user_info(
    client: &reqwest::Client,
    userinfo_endpoint: &str,
    access_token: &str,
) -> Result<OidcUserInfo, ServerError> {
    let response = client
        .get(userinfo_endpoint)
        .bearer_auth(access_token)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ServerError::bad_request(format!("Userinfo request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerError::bad_request(format!(
            "Userinfo endpoint returned status {}",
            response.status()
        )));
    }

    response
        .json::<OidcUserInfo>()
        .await
        .map_err(|e| ServerError::bad_request(format!("Failed to parse userinfo: {}", e)))
}

/// Upsert an SSO-authenticated user into the database.
///
/// Creates a new user if one with this SSO provider+subject doesn't exist,
/// otherwise updates the existing user's metadata.
async fn upsert_sso_user(
    pool: &tachyon_database::DatabasePool,
    provider: &str,
    user_info: &OidcUserInfo,
) -> Result<(), ServerError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| ServerError::internal(format!("Database connection failed: {}", e)))?;

    let sso_provider = format!("oidc:{}", provider);
    let display_name = user_info.name.as_deref().unwrap_or(&user_info.sub);

    // Try to find existing user by SSO provider + subject
    // Uses the core user table pattern: query by sso_provider and sso_subject
    let result = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM users WHERE sso_provider = $1 AND sso_subject = $2",
    )
    .bind(&sso_provider)
    .bind(&user_info.sub)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::internal(format!("User lookup failed: {}", e)))?;

    if result.is_some() {
        // Update existing user
        sqlx::query(
            "UPDATE users SET display_name = $1, email = COALESCE($2, email), updated_at = NOW() WHERE sso_provider = $3 AND sso_subject = $4",
        )
        .bind(display_name)
        .bind(&user_info.email)
        .bind(&sso_provider)
        .bind(&user_info.sub)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::internal(format!("User update failed: {}", e)))?;

        debug!(
            "Updated existing SSO user: {} ({})",
            user_info.sub, provider
        );
    } else {
        // Create new user
        let user_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO users (id, username, display_name, email, sso_provider, sso_subject, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"#,
        )
        .bind(&user_id)
        .bind(&user_info.sub)
        .bind(display_name)
        .bind(&user_info.email)
        .bind(&sso_provider)
        .bind(&user_info.sub)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::internal(format!("User creation failed: {}", e)))?;

        info!("Created new SSO user: {} ({})", user_info.sub, provider);
    }

    Ok(())
}

// ============================================================================
// Router
// ============================================================================

pub fn create_oidc_router() -> axum::Router<OidcState> {
    use axum::routing::get;
    axum::Router::new()
        .route("/oidc/{provider}/authorize", get(oidc_authorize))
        .route("/oidc/{provider}/callback", get(oidc_callback))
}
