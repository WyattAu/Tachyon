use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

const MAGIC_LINK_TOKEN_SELECT_SQL: &str = r#"
    SELECT
        id::TEXT as id,
        user_id::TEXT as user_id,
        token_hash,
        expires_at::TEXT as expires_at,
        consumed_at::TEXT as consumed_at,
        ip_address,
        created_at::TEXT as created_at
    FROM magic_link_tokens
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MagicLinkToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: String,
    pub consumed_at: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct MagicLinkRepository {
    pool: DatabasePool,
}

impl MagicLinkRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create_token(
        &self,
        user_id: &str,
        token_hash: &str,
        expires_in_secs: i64,
        ip_address: Option<&str>,
    ) -> DatabaseResult<MagicLinkToken> {
        let id = uuid::Uuid::new_v4().to_string();

        let sql = r#"
            INSERT INTO magic_link_tokens (id, user_id, token_hash, expires_at, ip_address)
            VALUES ($1::uuid, $2::uuid, $3, NOW() + INTERVAL '1 second' * $4, $5)
            RETURNING
                id::TEXT as id,
                user_id::TEXT as user_id,
                token_hash,
                expires_at::TEXT as expires_at,
                consumed_at::TEXT as consumed_at,
                ip_address,
                created_at::TEXT as created_at
        "#;

        sqlx::query_as::<_, MagicLinkToken>(sql)
            .bind(&id)
            .bind(user_id)
            .bind(token_hash)
            .bind(expires_in_secs)
            .bind(ip_address)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn consume_token(&self, token_hash: &str) -> DatabaseResult<Option<MagicLinkToken>> {
        let sql = format!(
            "{} WHERE token_hash = $1 AND consumed_at IS NULL AND expires_at > NOW() \
             FOR UPDATE",
            MAGIC_LINK_TOKEN_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;

        let token = sqlx::query_as::<_, MagicLinkToken>(&sql)
            .bind(token_hash)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        let token = match token {
            Some(t) => t,
            None => return Ok(None),
        };

        let update_sql = "UPDATE magic_link_tokens SET consumed_at = NOW() WHERE id = $1::uuid";
        sqlx::query(update_sql)
            .bind(&token.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(Some(token))
    }

    #[instrument(skip(self))]
    pub async fn invalidate_user_tokens(&self, user_id: &str) -> DatabaseResult<()> {
        let sql = "UPDATE magic_link_tokens SET consumed_at = NOW() WHERE user_id = $1::uuid AND consumed_at IS NULL";

        sqlx::query(sql)
            .bind(user_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> DatabaseResult<u64> {
        let sql = "DELETE FROM magic_link_tokens WHERE expires_at < NOW()";

        let result = sqlx::query(sql)
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
    fn test_magic_link_token_struct_fields() {
        let token = MagicLinkToken {
            id: "1".into(),
            user_id: "user-1".into(),
            token_hash: "hash123".into(),
            expires_at: "2024-01-02T00:00:00Z".into(),
            consumed_at: None,
            ip_address: Some("127.0.0.1".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert_eq!(token.user_id, "user-1");
        assert_eq!(token.token_hash, "hash123");
        assert!(token.consumed_at.is_none());
        assert_eq!(token.ip_address.as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn test_magic_link_token_consumed() {
        let token = MagicLinkToken {
            id: "1".into(),
            user_id: "user-1".into(),
            token_hash: "hash123".into(),
            expires_at: "2024-01-02T00:00:00Z".into(),
            consumed_at: Some("2024-01-01T12:00:00Z".into()),
            ip_address: None,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert!(token.consumed_at.is_some());
    }

    #[test]
    fn test_magic_link_token_serialization() {
        let token = MagicLinkToken {
            id: "abc123".into(),
            user_id: "user-2".into(),
            token_hash: "h".repeat(64),
            expires_at: "2099-01-01T00:00:00Z".into(),
            consumed_at: None,
            ip_address: Some("10.0.0.1".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("user-2"));
        assert!(json.contains("10.0.0.1"));
        let parsed: MagicLinkToken = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, token.id);
    }
}
