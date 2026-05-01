// User Preferences Persistence
// JSON preferences storage per user

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use sqlx::Row;
use tachyon_core::id::UserId;
use tracing::instrument;

/// User preferences repository.
pub struct UserPreferencesRepository {
    pool: DatabasePool,
}

impl UserPreferencesRepository {
    /// Create a new user preferences repository backed by `pool`.
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Get preferences for a user. Returns `{}` if none exist.
    #[instrument(skip(self))]
    pub async fn get_preferences(&self, user_id: &UserId) -> DatabaseResult<serde_json::Value> {
        let sql = "SELECT preferences FROM user_preferences WHERE user_id = $1";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .bind(user_id.as_uuid())
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let prefs: serde_json::Value = row.get("preferences");
                Ok(prefs)
            }
            None => Ok(serde_json::json!({})),
        }
    }

    /// Set (upsert) preferences for a user.
    #[instrument(skip(self, prefs))]
    pub async fn set_preferences(
        &self,
        user_id: &UserId,
        prefs: &serde_json::Value,
    ) -> DatabaseResult<()> {
        let sql = r#"
            INSERT INTO user_preferences (user_id, preferences, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id) DO UPDATE SET preferences = $2, updated_at = NOW()
        "#;
        let mut conn = self.pool.acquire().await?;
        sqlx::query(sql)
            .bind(user_id.as_uuid())
            .bind(prefs)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_default_preferences() {
        let prefs = serde_json::json!({});
        assert!(prefs.is_object());
        assert!(prefs.as_object().unwrap().is_empty());
    }
}
