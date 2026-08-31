//! End-to-end encryption routes for document-level encryption.
//! Uses client-side encryption with server-side key metadata storage.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Encryption key metadata stored server-side (key material never stored).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionKeyMeta {
    pub id: Uuid,
    pub document_id: Uuid,
    pub owner_id: Uuid,
    pub key_algorithm: String,
    pub public_key_fingerprint: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterKeyRequest {
    pub document_id: Uuid,
    pub key_algorithm: String,
    pub public_key_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateKeyRequest {
    pub key_algorithm: Option<String>,
    pub public_key_fingerprint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EncryptionStatus {
    pub document_id: String,
    pub encrypted: bool,
    pub key_algorithm: Option<String>,
    pub key_fingerprint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KeyListResponse {
    pub keys: Vec<EncryptionKeyMeta>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct DeleteKeyResponse {
    pub deleted: bool,
    pub key_id: String,
}

#[derive(Clone)]
pub struct E2eState {
    pub pool: tachyon_database::DatabasePool,
}

/// Register or update an encryption key for a document.
pub async fn register_encryption_key(
    axum::extract::State(state): axum::extract::State<E2eState>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
    axum::Json(req): axum::Json<RegisterKeyRequest>,
) -> Result<axum::Json<serde_json::Value>, crate::error::ServerError> {
    let row = sqlx::query_as::<_, EncryptionKeyMeta>(
        r#"INSERT INTO document_encryption_keys (document_id, owner_id, key_algorithm, public_key_fingerprint)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (document_id) DO UPDATE SET key_algorithm = $3, public_key_fingerprint = $4
           RETURNING *"#,
    )
    .bind(req.document_id)
    .bind(user_id)
    .bind(&req.key_algorithm)
    .bind(&req.public_key_fingerprint)
    .fetch_one(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;
    Ok(axum::Json(
        serde_json::json!({"key_id": row.id, "registered": true}),
    ))
}

/// Get the encryption key metadata for a document.
pub async fn get_encryption_key(
    axum::extract::State(state): axum::extract::State<E2eState>,
    axum::extract::Path(document_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<serde_json::Value>, crate::error::ServerError> {
    let row = sqlx::query_as::<_, EncryptionKeyMeta>(
        "SELECT * FROM document_encryption_keys WHERE document_id = $1",
    )
    .bind(document_id)
    .fetch_optional(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;

    match row {
        Some(key) => Ok(axum::Json(serde_json::json!({
            "id": key.id,
            "document_id": key.document_id,
            "owner_id": key.owner_id,
            "key_algorithm": key.key_algorithm,
            "public_key_fingerprint": key.public_key_fingerprint,
            "created_at": key.created_at,
        }))),
        None => Ok(axum::Json(serde_json::json!({
            "error": "Encryption key not found",
            "document_id": document_id,
        }))),
    }
}

/// Get encryption status for a document.
pub async fn get_encryption_status(
    axum::extract::State(state): axum::extract::State<E2eState>,
    axum::extract::Path(document_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<EncryptionStatus>, crate::error::ServerError> {
    let row = sqlx::query_as::<_, EncryptionKeyMeta>(
        "SELECT * FROM document_encryption_keys WHERE document_id = $1",
    )
    .bind(document_id)
    .fetch_optional(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;
    Ok(axum::Json(EncryptionStatus {
        document_id: document_id.to_string(),
        encrypted: row.is_some(),
        key_algorithm: row.as_ref().map(|r| r.key_algorithm.clone()),
        key_fingerprint: row.as_ref().map(|r| r.public_key_fingerprint.clone()),
    }))
}

/// Delete an encryption key for a document.
pub async fn delete_encryption_key(
    axum::extract::State(state): axum::extract::State<E2eState>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
    axum::extract::Path(document_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<DeleteKeyResponse>, crate::error::ServerError> {
    let result = sqlx::query(
        "DELETE FROM document_encryption_keys WHERE document_id = $1 AND owner_id = $2",
    )
    .bind(document_id)
    .bind(user_id)
    .execute(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;

    let deleted = result.rows_affected() > 0;
    Ok(axum::Json(DeleteKeyResponse {
        deleted,
        key_id: document_id.to_string(),
    }))
}

/// List all encryption keys owned by the authenticated user.
pub async fn list_encryption_keys(
    axum::extract::State(state): axum::extract::State<E2eState>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
) -> Result<axum::Json<KeyListResponse>, crate::error::ServerError> {
    let rows = sqlx::query_as::<_, EncryptionKeyMeta>(
        "SELECT * FROM document_encryption_keys WHERE owner_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;

    let total = rows.len();
    Ok(axum::Json(KeyListResponse { keys: rows, total }))
}

/// Create the E2E encryption router.
pub fn create_e2e_router() -> axum::Router<E2eState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route(
            "/e2e/keys",
            post(register_encryption_key).get(list_encryption_keys),
        )
        .route(
            "/e2e/keys/{document_id}",
            get(get_encryption_key).delete(delete_encryption_key),
        )
        .route("/e2e/status/{document_id}", get(get_encryption_status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_status_serialization() {
        let status = EncryptionStatus {
            document_id: Uuid::new_v4().to_string(),
            encrypted: true,
            key_algorithm: Some("aes-256-gcm".to_string()),
            key_fingerprint: Some("sha256:abc123".to_string()),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("aes-256-gcm"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_encryption_status_unencrypted() {
        let status = EncryptionStatus {
            document_id: "123".to_string(),
            encrypted: false,
            key_algorithm: None,
            key_fingerprint: None,
        };
        assert!(!status.encrypted);
    }

    #[test]
    fn test_register_key_request() {
        let req = RegisterKeyRequest {
            document_id: Uuid::nil(),
            key_algorithm: "xchacha20-poly1305".to_string(),
            public_key_fingerprint: "fp123".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("xchacha20-poly1305"));
    }

    #[test]
    fn test_delete_key_response() {
        let resp = DeleteKeyResponse {
            deleted: true,
            key_id: "abc".to_string(),
        };
        assert!(resp.deleted);
    }

    #[test]
    fn test_key_list_response() {
        let resp = KeyListResponse {
            keys: vec![],
            total: 0,
        };
        assert_eq!(resp.total, 0);
    }

    #[test]
    fn test_update_key_request() {
        let req = UpdateKeyRequest {
            key_algorithm: Some("aes-256-gcm".to_string()),
            public_key_fingerprint: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("aes-256-gcm"));
    }
}
