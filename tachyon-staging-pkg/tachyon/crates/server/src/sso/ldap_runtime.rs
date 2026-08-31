//! LDAP runtime flow.
//!
//! Provides LDAP authentication and user synchronization:
//! 1. `POST /auth/sso/ldap/login` -- Bind + search to authenticate a user
//! 2. `POST /auth/sso/ldap/sync` -- Synchronize LDAP users into the database

use axum::{extract::State, response::Json};
use ldap3::{LdapConn, Scope, SearchEntry};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use super::ldap::LdapConfig;
use crate::error::ServerError;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct LdapState {
    pub config: LdapConfig,
    pub pool: tachyon_database::DatabasePool,
    pub jwt_secret: String,
}

// ============================================================================
// Request / Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LdapLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LdapLoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LdapSyncResponse {
    pub users_synced: u64,
    pub users_created: u64,
    pub users_updated: u64,
    pub users_deactivated: u64,
    pub errors: Vec<String>,
}

// ============================================================================
// LDAP helpers
// ============================================================================

fn connect_ldap(config: &LdapConfig) -> Result<LdapConn, ServerError> {
    let url = if config.use_tls && !config.server_url.starts_with("ldaps://") {
        config.server_url.replacen("ldap://", "ldaps://", 1)
    } else {
        config.server_url.clone()
    };

    let mut conn = LdapConn::new(&url)
        .map_err(|e| ServerError::internal(format!("LDAP connection failed: {}", e)))?;

    conn.simple_bind(&config.bind_dn, &config.bind_password)
        .map_err(|e| ServerError::internal(format!("LDAP bind failed: {}", e)))?
        .success()
        .map_err(|e| ServerError::internal(format!("LDAP bind error: {}", e)))?;

    Ok(conn)
}

struct LdapUserEntry {
    dn: String,
    username: String,
    email: Option<String>,
    display_name: String,
}

fn search_user(
    conn: &mut LdapConn,
    config: &LdapConfig,
    username: &str,
) -> Result<LdapUserEntry, ServerError> {
    let filter = config.user_filter.replace("{username}", username);
    let attrs = vec![
        "dn".to_string(),
        config.attribute_mapping.username.clone(),
        config.attribute_mapping.email.clone(),
        config.attribute_mapping.display_name.clone(),
    ];

    let (results, _ldap_result) = conn
        .search(&config.base_dn, Scope::Subtree, &filter, attrs)
        .map_err(|e| ServerError::internal(format!("LDAP search failed: {}", e)))?
        .success()
        .map_err(|e| ServerError::internal(format!("LDAP search error: {}", e)))?;

    if results.is_empty() {
        return Err(ServerError::not_found("User", username));
    }

    let entry = SearchEntry::construct(results.into_iter().next().unwrap());

    let username_val = extract_attribute(&entry, &config.attribute_mapping.username)
        .unwrap_or_else(|| username.to_string());
    let email = extract_attribute(&entry, &config.attribute_mapping.email);
    let display_name = extract_attribute(&entry, &config.attribute_mapping.display_name)
        .unwrap_or_else(|| username_val.clone());

    Ok(LdapUserEntry {
        dn: entry.dn,
        username: username_val,
        email,
        display_name,
    })
}

fn search_all_users(
    conn: &mut LdapConn,
    config: &LdapConfig,
) -> Result<Vec<LdapUserEntry>, ServerError> {
    let filter = config.user_filter.replace("{username}", "*");
    let attrs = vec![
        "dn".to_string(),
        config.attribute_mapping.username.clone(),
        config.attribute_mapping.email.clone(),
        config.attribute_mapping.display_name.clone(),
    ];

    let (results, _ldap_result) = conn
        .search(&config.base_dn, Scope::Subtree, &filter, attrs)
        .map_err(|e| ServerError::internal(format!("LDAP search failed: {}", e)))?
        .success()
        .map_err(|e| ServerError::internal(format!("LDAP search error: {}", e)))?;

    let mut entries = Vec::new();
    for result_entry in results {
        let entry = SearchEntry::construct(result_entry);
        let username_val =
            extract_attribute(&entry, &config.attribute_mapping.username).unwrap_or_default();
        let email = extract_attribute(&entry, &config.attribute_mapping.email);
        let display_name = extract_attribute(&entry, &config.attribute_mapping.display_name)
            .unwrap_or_else(|| username_val.clone());

        if username_val.is_empty() {
            continue;
        }

        entries.push(LdapUserEntry {
            dn: entry.dn,
            username: username_val,
            email,
            display_name,
        });
    }

    Ok(entries)
}

fn extract_attribute(entry: &SearchEntry, attr: &str) -> Option<String> {
    entry
        .attrs
        .get(attr)
        .and_then(|v| v.first())
        .cloned()
        .filter(|s| !s.is_empty())
}

async fn upsert_ldap_user(
    pool: &tachyon_database::DatabasePool,
    user: &LdapUserEntry,
) -> Result<bool, ServerError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| ServerError::internal(format!("Database connection failed: {}", e)))?;

    let existing = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM users WHERE sso_provider = $1 AND sso_subject = $2",
    )
    .bind("ldap")
    .bind(&user.dn)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::internal(format!("User lookup failed: {}", e)))?;

    if existing.is_some() {
        sqlx::query(
            "UPDATE users SET display_name = $1, email = COALESCE($2, email), username = $3, updated_at = NOW() WHERE sso_provider = $4 AND sso_subject = $5",
        )
        .bind(&user.display_name)
        .bind(&user.email)
        .bind(&user.username)
        .bind("ldap")
        .bind(&user.dn)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::internal(format!("User update failed: {}", e)))?;

        debug!("Updated existing LDAP user: {}", user.username);
        Ok(false)
    } else {
        let user_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO users (id, username, display_name, email, sso_provider, sso_subject, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"#,
        )
        .bind(&user_id)
        .bind(&user.username)
        .bind(&user.display_name)
        .bind(&user.email)
        .bind("ldap")
        .bind(&user.dn)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::internal(format!("User creation failed: {}", e)))?;

        info!("Created new LDAP user: {}", user.username);
        Ok(true)
    }
}

// ============================================================================
// Handlers
// ============================================================================

#[utoipa::path(
    post,
    path = "/api/v1/auth/sso/ldap/login",
    responses(
        (status = 200, description = "LDAP authentication successful", body = LdapLoginResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 503, description = "LDAP server unavailable"),
    ),
    tag = "auth",
)]
pub async fn ldap_login(
    State(state): State<LdapState>,
    Json(req): Json<LdapLoginRequest>,
) -> Result<Json<LdapLoginResponse>, ServerError> {
    info!("LDAP login attempt for user: {}", req.username);

    let config = state.config.clone();
    let username = req.username.clone();
    let password = req.password.clone();
    let pool = state.pool.clone();

    let user_entry = tokio::task::spawn_blocking(move || {
        let mut conn = connect_ldap(&config)?;

        let user = search_user(&mut conn, &config, &username)?;

        conn.simple_bind(&user.dn, &password)
            .map_err(|e| ServerError::internal(format!("LDAP credential bind failed: {}", e)))?
            .success()
            .map_err(|e| {
                warn!("LDAP bind failed for user {}: {}", username, e);
                ServerError::unauthorized("Invalid LDAP credentials")
            })?;

        Ok::<LdapUserEntry, ServerError>(user)
    })
    .await
    .map_err(|e| ServerError::internal(format!("LDAP task error: {}", e)))??;

    upsert_ldap_user(&pool, &user_entry).await?;

    let now = jsonwebtoken::get_current_timestamp();
    let exp = now + 3600;

    let claims = serde_json::json!({
        "sub": user_entry.username,
        "role": "user",
        "iss": "tachyon",
        "aud": "tachyon-api",
        "exp": exp,
        "iat": now,
        "sso_provider": "ldap",
    });

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::HS256,
            ..Default::default()
        },
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| ServerError::internal(format!("JWT creation failed: {}", e)))?;

    Ok(Json(LdapLoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        username: user_entry.username,
        display_name: user_entry.display_name,
        email: user_entry.email,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/sso/ldap/sync",
    responses(
        (status = 200, description = "LDAP sync complete", body = LdapSyncResponse),
        (status = 503, description = "LDAP server unavailable"),
    ),
    tag = "auth",
)]
pub async fn ldap_sync(
    State(state): State<LdapState>,
) -> Result<Json<LdapSyncResponse>, ServerError> {
    info!(
        "LDAP sync requested: base_dn={}, filter={}",
        state.config.base_dn, state.config.user_filter
    );

    let config = state.config.clone();
    let pool = state.pool.clone();

    let user_entries = tokio::task::spawn_blocking(move || {
        let mut conn = connect_ldap(&config)?;
        search_all_users(&mut conn, &config)
    })
    .await
    .map_err(|e| ServerError::internal(format!("LDAP task error: {}", e)))??;

    let total = user_entries.len() as u64;
    let mut users_created: u64 = 0;
    let mut users_updated: u64 = 0;
    let mut errors: Vec<String> = Vec::new();

    for user in &user_entries {
        match upsert_ldap_user(&pool, user).await {
            Ok(created) => {
                if created {
                    users_created += 1;
                } else {
                    users_updated += 1;
                }
            }
            Err(e) => {
                error!("Failed to sync LDAP user {}: {}", user.username, e);
                errors.push(format!("{}: {}", user.username, e));
            }
        }
    }

    info!(
        "LDAP sync complete: {} total, {} created, {} updated, {} errors",
        total,
        users_created,
        users_updated,
        errors.len()
    );

    Ok(Json(LdapSyncResponse {
        users_synced: total,
        users_created,
        users_updated,
        users_deactivated: 0,
        errors,
    }))
}

// ============================================================================
// Router
// ============================================================================

pub fn create_ldap_router() -> axum::Router<LdapState> {
    use axum::routing::post;
    axum::Router::new()
        .route("/ldap/login", post(ldap_login))
        .route("/ldap/sync", post(ldap_sync))
}
