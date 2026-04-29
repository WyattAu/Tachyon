use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

const SELECT_SQL: &str = r#"
    SELECT
        id::TEXT as id,
        user_id::TEXT as user_id,
        token_hash,
        expires_at::TEXT as expires_at,
        revoked,
        created_at::TEXT as created_at
    FROM refresh_tokens
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: String,
    pub revoked: bool,
    pub created_at: String,
}

#[derive(Clone)]
pub struct RefreshTokenRepository {
    pool: DatabasePool,
}

impl RefreshTokenRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create(
        &self,
        user_id: &str,
        token_hash: &str,
        expires_in_secs: i64,
    ) -> DatabaseResult<RefreshToken> {
        let id = uuid::Uuid::new_v4().to_string();

        let sql = r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
            VALUES ($1::uuid, $2::uuid, $3, NOW() + INTERVAL '1 second' * $4)
            RETURNING
                id::TEXT as id,
                user_id::TEXT as user_id,
                token_hash,
                expires_at::TEXT as expires_at,
                revoked,
                created_at::TEXT as created_at
        "#;

        sqlx::query_as::<_, RefreshToken>(sql)
            .bind(&id)
            .bind(user_id)
            .bind(token_hash)
            .bind(expires_in_secs)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn find_valid_by_hash(
        &self,
        token_hash: &str,
    ) -> DatabaseResult<Option<RefreshToken>> {
        let sql = format!(
            "{} WHERE token_hash = $1 AND revoked = FALSE AND expires_at > NOW()",
            SELECT_SQL
        );

        sqlx::query_as::<_, RefreshToken>(&sql)
            .bind(token_hash)
            .fetch_optional(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn revoke(&self, token_hash: &str) -> DatabaseResult<bool> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked = TRUE WHERE token_hash = $1 AND revoked = FALSE",
        )
        .bind(token_hash)
        .execute(self.pool.inner())
        .await
        .map_err(|e| DatabaseError::query_error(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(self))]
    pub async fn revoke_all_for_user(&self, user_id: &str) -> DatabaseResult<u64> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = $1::uuid AND revoked = FALSE",
        )
        .bind(user_id)
        .execute(self.pool.inner())
        .await
        .map_err(|e| DatabaseError::query_error(e.to_string()))?;
        Ok(result.rows_affected())
    }

    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> DatabaseResult<u64> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refresh_token_struct_fields() {
        let token = RefreshToken {
            id: "1".into(),
            user_id: "user-1".into(),
            token_hash: "hash123".into(),
            expires_at: "2024-02-01T00:00:00Z".into(),
            revoked: false,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert_eq!(token.user_id, "user-1");
        assert_eq!(token.token_hash, "hash123");
        assert!(!token.revoked);
    }

    #[test]
    fn test_refresh_token_revoked() {
        let token = RefreshToken {
            id: "1".into(),
            user_id: "user-1".into(),
            token_hash: "hash123".into(),
            expires_at: "2024-02-01T00:00:00Z".into(),
            revoked: true,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert!(token.revoked);
    }
}
