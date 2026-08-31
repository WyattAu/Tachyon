use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

const RESET_TOKEN_SELECT_SQL: &str = r#"
    SELECT
        id::TEXT as id,
        user_id::TEXT as user_id,
        token_hash,
        expires_at::TEXT as expires_at,
        used_at::TEXT as used_at,
        created_at::TEXT as created_at
    FROM password_reset_tokens
"#;

const VERIFICATION_TOKEN_SELECT_SQL: &str = r#"
    SELECT
        id::TEXT as id,
        user_id::TEXT as user_id,
        email,
        token_hash,
        expires_at::TEXT as expires_at,
        used_at::TEXT as used_at,
        created_at::TEXT as created_at
    FROM email_verification_tokens
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordResetToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: String,
    pub used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailVerificationToken {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub token_hash: String,
    pub expires_at: String,
    pub used_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct PasswordResetRepository {
    pool: DatabasePool,
}

impl PasswordResetRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create_reset_token(
        &self,
        user_id: &str,
        token_hash: &str,
        expires_in_hours: i64,
    ) -> DatabaseResult<PasswordResetToken> {
        let id = uuid::Uuid::new_v4().to_string();

        let sql = r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at)
            VALUES ($1::uuid, $2::uuid, $3, NOW() + INTERVAL '1 hour' * $4)
            RETURNING
                id::TEXT as id,
                user_id::TEXT as user_id,
                token_hash,
                expires_at::TEXT as expires_at,
                used_at::TEXT as used_at,
                created_at::TEXT as created_at
        "#;

        sqlx::query_as::<_, PasswordResetToken>(sql)
            .bind(&id)
            .bind(user_id)
            .bind(token_hash)
            .bind(expires_in_hours)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn consume_reset_token(
        &self,
        token_hash: &str,
    ) -> DatabaseResult<Option<PasswordResetToken>> {
        let sql = format!(
            "{} WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW() \
             FOR UPDATE",
            RESET_TOKEN_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;

        let token = sqlx::query_as::<_, PasswordResetToken>(&sql)
            .bind(token_hash)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        let token = match token {
            Some(t) => t,
            None => return Ok(None),
        };

        let update_sql = "UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1::uuid";
        sqlx::query(update_sql)
            .bind(&token.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(Some(token))
    }

    #[instrument(skip(self))]
    pub async fn invalidate_user_tokens(&self, user_id: &str) -> DatabaseResult<()> {
        let sql = "UPDATE password_reset_tokens SET used_at = NOW() WHERE user_id = $1::uuid AND used_at IS NULL";

        sqlx::query(sql)
            .bind(user_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn create_verification_token(
        &self,
        user_id: &str,
        email: &str,
        token_hash: &str,
        expires_in_hours: i64,
    ) -> DatabaseResult<EmailVerificationToken> {
        let id = uuid::Uuid::new_v4().to_string();

        let sql = r#"
            INSERT INTO email_verification_tokens (id, user_id, email, token_hash, expires_at)
            VALUES ($1::uuid, $2::uuid, $3, $4, NOW() + INTERVAL '1 hour' * $5)
            RETURNING
                id::TEXT as id,
                user_id::TEXT as user_id,
                email,
                token_hash,
                expires_at::TEXT as expires_at,
                used_at::TEXT as used_at,
                created_at::TEXT as created_at
        "#;

        sqlx::query_as::<_, EmailVerificationToken>(sql)
            .bind(&id)
            .bind(user_id)
            .bind(email)
            .bind(token_hash)
            .bind(expires_in_hours)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn consume_verification_token(
        &self,
        token_hash: &str,
    ) -> DatabaseResult<Option<EmailVerificationToken>> {
        let sql = format!(
            "{} WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW() \
             FOR UPDATE",
            VERIFICATION_TOKEN_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;

        let token = sqlx::query_as::<_, EmailVerificationToken>(&sql)
            .bind(token_hash)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        let token = match token {
            Some(t) => t,
            None => return Ok(None),
        };

        let update_sql = "UPDATE email_verification_tokens SET used_at = NOW() WHERE id = $1::uuid";
        sqlx::query(update_sql)
            .bind(&token.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(Some(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_password_reset_token_struct_fields() {
        let token = PasswordResetToken {
            id: "1".into(),
            user_id: "user-1".into(),
            token_hash: "hash123".into(),
            expires_at: "2024-01-02T00:00:00Z".into(),
            used_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert_eq!(token.user_id, "user-1");
        assert_eq!(token.token_hash, "hash123");
        assert!(token.used_at.is_none());
    }

    #[test]
    fn test_password_reset_token_used() {
        let token = PasswordResetToken {
            id: "1".into(),
            user_id: "user-1".into(),
            token_hash: "hash123".into(),
            expires_at: "2024-01-02T00:00:00Z".into(),
            used_at: Some("2024-01-01T12:00:00Z".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert!(token.used_at.is_some());
        assert_eq!(token.used_at.as_deref(), Some("2024-01-01T12:00:00Z"));
    }

    #[test]
    fn test_email_verification_token_struct_fields() {
        let token = EmailVerificationToken {
            id: "1".into(),
            user_id: "user-1".into(),
            email: "user@example.com".into(),
            token_hash: "hash456".into(),
            expires_at: "2024-01-02T00:00:00Z".into(),
            used_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert_eq!(token.email, "user@example.com");
        assert_eq!(token.token_hash, "hash456");
        assert!(token.used_at.is_none());
    }

    #[test]
    fn test_email_verification_token_used() {
        let token = EmailVerificationToken {
            id: "1".into(),
            user_id: "user-1".into(),
            email: "user@example.com".into(),
            token_hash: "hash456".into(),
            expires_at: "2024-01-02T00:00:00Z".into(),
            used_at: Some("2024-01-01T12:00:00Z".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        assert!(token.used_at.is_some());
    }
}
