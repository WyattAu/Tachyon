use crate::audit::{AuditContext, api_key_created, api_key_revoked};
use crate::error::ServerError;
use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tracing::info;

#[derive(Debug, Clone)]
pub struct ApiKeyState {
    pub pool: tachyon_database::DatabasePool,
    pub audit_logger: crate::audit::AuditLogger,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub expires_in_days: Option<i64>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub api_key: String,
    pub key_info: ApiKeyResponse,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyListResponse {
    pub keys: Vec<ApiKeyResponse>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeyQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

fn generate_api_key() -> (String, String, String) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let bytes: [u8; 32] = rng.r#gen();
    let hex = hex::encode(bytes);
    let api_key = format!("tk_{}", hex);

    let key_prefix = format!("tk_{}", &hex[..8]);

    let hash = sha2::Sha256::digest(api_key.as_bytes());
    let key_hash = hex::encode(hash);

    (api_key, key_prefix, key_hash)
}

pub async fn create_api_key(
    Extension(auth): Extension<crate::middleware::AuthContext>,
    State(state): State<ApiKeyState>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, ServerError> {
    let user_id = auth.user_id.clone();
    let scopes = req.scopes.unwrap_or_else(|| vec!["read".to_string()]);

    if req.name.trim().is_empty() {
        return Err(ServerError::bad_request("API key name is required"));
    }
    if scopes.is_empty() {
        return Err(ServerError::bad_request("At least one scope is required"));
    }

    let (api_key, key_prefix, key_hash) = generate_api_key();

    let expires_at = req
        .expires_in_days
        .map(|days| chrono::Utc::now() + chrono::Duration::days(days));

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let row: (
        String,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(
        r#"INSERT INTO api_keys (name, key_prefix, key_hash, user_id, scopes, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, key_prefix, name, created_at, expires_at"#,
    )
    .bind(&req.name)
    .bind(&key_prefix)
    .bind(&key_hash)
    .bind(&user_id)
    .bind(&scopes)
    .bind(expires_at)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let _ = state
        .audit_logger
        .log(api_key_created(&user_id, &key_prefix, AuditContext::new()))
        .await;

    info!(user_id = %user_id, key_prefix = %key_prefix, "API key created");

    Ok(Json(CreateApiKeyResponse {
        api_key,
        key_info: ApiKeyResponse {
            id: row.0,
            name: row.2,
            key_prefix: row.1,
            scopes,
            created_at: row.3.to_rfc3339(),
            expires_at: row.4.map(|d| d.to_rfc3339()),
            last_used_at: None,
        },
    }))
}

pub async fn list_api_keys(
    Extension(auth): Extension<crate::middleware::AuthContext>,
    State(state): State<ApiKeyState>,
    Query(params): Query<ApiKeyQuery>,
) -> Result<Json<ApiKeyListResponse>, ServerError> {
    let user_id = auth.user_id.clone();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let rows: Vec<(
        String,
        String,
        String,
        Vec<String>,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    )> = sqlx::query_as(
        r#"SELECT id, key_prefix, name, scopes, created_at, expires_at, last_used_at
           FROM api_keys
           WHERE user_id = $1 AND is_active = true
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(&user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let total: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM api_keys WHERE user_id = $1 AND is_active = true")
            .bind(&user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| ServerError::database(e.to_string()))?;

    let keys = rows
        .into_iter()
        .map(|row| ApiKeyResponse {
            id: row.0,
            name: row.2,
            key_prefix: row.1,
            scopes: row.3,
            created_at: row.4.to_rfc3339(),
            expires_at: row.5.map(|d| d.to_rfc3339()),
            last_used_at: row.6.map(|d| d.to_rfc3339()),
        })
        .collect();

    Ok(Json(ApiKeyListResponse {
        keys,
        total: total.0,
    }))
}

pub async fn revoke_api_key(
    Extension(auth): Extension<crate::middleware::AuthContext>,
    State(state): State<ApiKeyState>,
    Path(key_id): Path<String>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let user_id = auth.user_id.clone();

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT id, key_prefix FROM api_keys WHERE id = $1 AND user_id = $2 AND is_active = true",
    )
    .bind(&key_id)
    .bind(&user_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let (id, key_prefix) = row.ok_or_else(|| ServerError::not_found("API key", &key_id))?;

    sqlx::query("UPDATE api_keys SET is_active = false, revoked_at = NOW() WHERE id = $1")
        .bind(&id)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let _ = state
        .audit_logger
        .log(api_key_revoked(&user_id, &key_prefix, AuditContext::new()))
        .await;

    info!(user_id = %user_id, key_id = %key_id, "API key revoked");

    Ok(Json(serde_json::json!({ "revoked": true, "id": id })))
}

pub fn create_api_key_router() -> axum::Router<ApiKeyState> {
    axum::Router::new()
        .route(
            "/api-keys",
            axum::routing::post(create_api_key).get(list_api_keys),
        )
        .route("/api-keys/{key_id}", axum::routing::delete(revoke_api_key))
}
