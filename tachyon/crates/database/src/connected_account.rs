// Connected Accounts Repository
// Manages OAuth2 provider connections for user accounts

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow};
use uuid::Uuid;

/// A connected OAuth2 account.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConnectedAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub provider_username: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub connected_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// Create a new connected account.
#[derive(Debug, Deserialize)]
pub struct CreateConnectedAccount {
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub provider_username: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
}

pub struct ConnectedAccountRepository;

impl ConnectedAccountRepository {
    /// Create a new connected account link.
    /// Returns the existing account if the provider+provider_user_id already exists.
    pub async fn create(pool: &DatabasePool, account: CreateConnectedAccount) -> DatabaseResult<ConnectedAccount> {
        let mut conn = pool.acquire().await?;
        let result = query_as::<_, ConnectedAccount>(
            r#"INSERT INTO connected_accounts (user_id, provider, provider_user_id, provider_email, provider_username, avatar_url, access_token, refresh_token, token_expires_at)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
              ON CONFLICT (provider, provider_user_id) DO UPDATE
                SET user_id = EXCLUDED.user_id,
                    provider_email = EXCLUDED.provider_email,
                    provider_username = EXCLUDED.provider_username,
                    avatar_url = EXCLUDED.avatar_url,
                    access_token = EXCLUDED.access_token,
                    refresh_token = EXCLUDED.refresh_token,
                    token_expires_at = EXCLUDED.token_expires_at,
                    last_used_at = NOW()
              RETURNING id, user_id, provider, provider_user_id, provider_email, provider_username, avatar_url, access_token, refresh_token, token_expires_at, connected_at, last_used_at"#
        )
        .bind(account.user_id)
        .bind(&account.provider)
        .bind(&account.provider_user_id)
        .bind(&account.provider_email)
        .bind(&account.provider_username)
        .bind(&account.avatar_url)
        .bind(&account.access_token)
        .bind(&account.refresh_token)
        .bind(account.token_expires_at)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result)
    }

    /// List all connected accounts for a user.
    pub async fn list_for_user(pool: &DatabasePool, user_id: Uuid) -> DatabaseResult<Vec<ConnectedAccount>> {
        let mut conn = pool.acquire().await?;
        let results = query_as::<_, ConnectedAccount>(
            r#"SELECT id, user_id, provider, provider_user_id, provider_email, provider_username, avatar_url, access_token, refresh_token, token_expires_at, connected_at, last_used_at
              FROM connected_accounts
              WHERE user_id = $1
              ORDER BY connected_at DESC"#
        )
        .bind(user_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(results)
    }

    /// Find a connected account by provider and provider user ID.
    pub async fn find_by_provider(
        pool: &DatabasePool,
        provider: &str,
        provider_user_id: &str,
    ) -> DatabaseResult<Option<ConnectedAccount>> {
        let mut conn = pool.acquire().await?;
        let result = query_as::<_, ConnectedAccount>(
            r#"SELECT id, user_id, provider, provider_user_id, provider_email, provider_username, avatar_url, access_token, refresh_token, token_expires_at, connected_at, last_used_at
              FROM connected_accounts
              WHERE provider = $1 AND provider_user_id = $2"#
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result)
    }

    /// Find a connected account by its ID.
    pub async fn find_by_id(pool: &DatabasePool, account_id: Uuid) -> DatabaseResult<Option<ConnectedAccount>> {
        let mut conn = pool.acquire().await?;
        let result = query_as::<_, ConnectedAccount>(
            r#"SELECT id, user_id, provider, provider_user_id, provider_email, provider_username, avatar_url, access_token, refresh_token, token_expires_at, connected_at, last_used_at
              FROM connected_accounts
              WHERE id = $1"#
        )
        .bind(account_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result)
    }

    /// Delete (disconnect) a connected account by ID.
    /// Only allows deletion if the account belongs to the given user.
    pub async fn delete(pool: &DatabasePool, account_id: Uuid, user_id: Uuid) -> DatabaseResult<bool> {
        let mut conn = pool.acquire().await?;
        let result = sqlx::query(
            r#"DELETE FROM connected_accounts WHERE id = $1 AND user_id = $2"#
        )
        .bind(account_id)
        .bind(user_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_connected_account_request() {
        let req = CreateConnectedAccount {
            user_id: Uuid::nil(),
            provider: "google".to_string(),
            provider_user_id: "12345".to_string(),
            provider_email: Some("test@example.com".to_string()),
            provider_username: Some("testuser".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
        };
        assert_eq!(req.provider, "google");
        assert_eq!(req.provider_user_id, "12345");
    }
}
