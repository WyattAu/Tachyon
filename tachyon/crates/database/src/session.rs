// Session Persistence
// Session storage and management for PostgreSQL

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::SessionRecord;
use chrono::{DateTime, Duration, Utc};
use sqlx::{query, query_as, Row};
use tachyon_core::id::SessionId;
use tachyon_core::types::session::Session;
use tracing::{debug, info, instrument};

fn parse_uuid(s: &str, field: &str) -> Result<uuid::Uuid, DatabaseError> {
    uuid::Uuid::parse_str(s)
        .map_err(|e| DatabaseError::ValidationError(format!("Invalid {} UUID: {}", field, e)))
}

/// Session repository for persistence operations
pub struct SessionRepository {
    pool: DatabasePool,
}

impl SessionRepository {
    /// Create a new session repository
    ///
    /// # Arguments
    /// * `pool` - Database pool
    ///
    /// # Returns
    /// New SessionRepository instance
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a new session
    ///
    /// # Arguments
    /// * `session` - Session to persist
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn create(&self, session: &Session) -> DatabaseResult<()> {
        let insert_sql = r#"
            INSERT INTO sessions (
                id, user_id, session_type, status, token_value, token_type,
                ip_address, user_agent, device_info, created_at, expires_at, last_activity
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(parse_uuid(&session.id.as_str(), "session_id")?)
            .bind(parse_uuid(&session.user_id().as_str(), "user_id")?)
            .bind(session.session_type().to_string())
            .bind(format!("{:?}", session.status))
            .bind(&session.token.value)
            .bind(format!("{:?}", session.token.token_type))
            .bind(session.metadata.ip_address.as_deref())
            .bind(session.metadata.user_agent.as_deref())
            .bind(session.metadata.device_info.as_deref())
            .bind(session.created_at)
            .bind(session.expires_at)
            .bind(session.last_activity)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    DatabaseError::duplicate(
                        "session",
                        format!("Session ID {} already exists", session.id),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!("Session created: {}", session.id.as_str());
        Ok(())
    }

    /// Get a session by ID
    ///
    /// # Arguments
    /// * `id` - Session ID
    ///
    /// # Returns
    /// Result containing SessionRecord or error
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &SessionId) -> DatabaseResult<SessionRecord> {
        let select_sql = "SELECT * FROM sessions WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query_as::<_, SessionRecord>(select_sql)
            .bind(parse_uuid(&id.as_str(), "session_id")?)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        result.ok_or_else(|| DatabaseError::session_not_found(id.as_str()))
    }

    /// Get a session by token value
    ///
    /// # Arguments
    /// * `token_value` - Token value
    ///
    /// # Returns
    /// Result containing SessionRecord or error
    #[instrument(skip(self))]
    pub async fn get_by_token(&self, token_value: &str) -> DatabaseResult<SessionRecord> {
        let select_sql = "SELECT * FROM sessions WHERE token_value = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query_as::<_, SessionRecord>(select_sql)
            .bind(token_value)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        result.ok_or_else(|| DatabaseError::session_not_found(token_value))
    }

    /// Get all sessions for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `active_only` - Only return active sessions
    ///
    /// # Returns
    /// Result containing vector of SessionRecords or error
    pub async fn get_by_user(
        &self,
        user_id: &str,
        active_only: bool,
    ) -> DatabaseResult<Vec<SessionRecord>> {
        let select_sql = if active_only {
            "SELECT * FROM sessions WHERE user_id = $1 AND status = 'Active' ORDER BY last_activity DESC"
        } else {
            "SELECT * FROM sessions WHERE user_id = $1 ORDER BY last_activity DESC"
        };

        let mut conn = self.pool.acquire().await?;
        let sessions = query_as::<_, SessionRecord>(select_sql)
            .bind(parse_uuid(user_id, "user_id")?)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(sessions)
    }

    /// Update session last activity
    ///
    /// # Arguments
    /// * `id` - Session ID
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn update_activity(&self, id: &SessionId) -> DatabaseResult<()> {
        let update_sql = "UPDATE sessions SET last_activity = NOW() WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(parse_uuid(&id.as_str(), "session_id")?)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::session_not_found(id.as_str()));
        }

        debug!("Session activity updated: {}", id.as_str());
        Ok(())
    }

    /// Revoke a session
    ///
    /// # Arguments
    /// * `id` - Session ID
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn revoke(&self, id: &SessionId) -> DatabaseResult<()> {
        let update_sql = "UPDATE sessions SET status = 'Revoked' WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(parse_uuid(&id.as_str(), "session_id")?)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::session_not_found(id.as_str()));
        }

        info!("Session revoked: {}", id.as_str());
        Ok(())
    }

    /// Revoke all sessions for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// Result containing number of revoked sessions or error
    #[instrument(skip(self))]
    pub async fn revoke_all_for_user(&self, user_id: &str) -> DatabaseResult<u64> {
        let update_sql =
            "UPDATE sessions SET status = 'Revoked' WHERE user_id = $1 AND status = 'Active'";

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(parse_uuid(user_id, "user_id")?)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Revoked {} sessions for user: {}",
            result.rows_affected(),
            user_id
        );
        Ok(result.rows_affected())
    }

    /// Revoke all sessions except the specified one
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `except_session_id` - Session ID to exclude from revocation
    ///
    /// # Returns
    /// Result containing number of revoked sessions or error
    #[instrument(skip(self))]
    pub async fn revoke_all_except(
        &self,
        user_id: &str,
        except_session_id: &SessionId,
    ) -> DatabaseResult<u64> {
        let update_sql = r#"
            UPDATE sessions
            SET status = 'Revoked'
            WHERE user_id = $1 AND status = 'Active' AND id != $2
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(parse_uuid(user_id, "user_id")?)
            .bind(parse_uuid(&except_session_id.as_str(), "session_id")?)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Revoked {} sessions for user: {} (except: {})",
            result.rows_affected(),
            user_id,
            except_session_id.as_str()
        );
        Ok(result.rows_affected())
    }

    /// Delete a session
    ///
    /// # Arguments
    /// * `id` - Session ID
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &SessionId) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM sessions WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(parse_uuid(&id.as_str(), "session_id")?)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::session_not_found(id.as_str()));
        }

        debug!("Session deleted: {}", id.as_str());
        Ok(())
    }

    /// Delete expired sessions
    ///
    /// # Arguments
    /// * `before_date` - Delete sessions expired before this date
    ///
    /// # Returns
    /// Result containing number of deleted sessions or error
    #[instrument(skip(self))]
    pub async fn delete_expired(&self, before_date: DateTime<Utc>) -> DatabaseResult<u64> {
        let delete_sql = "DELETE FROM sessions WHERE expires_at < $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(before_date)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Deleted {} expired sessions", result.rows_affected());
        Ok(result.rows_affected())
    }

    /// Clean up old sessions (expired and revoked)
    ///
    /// # Arguments
    /// * `days_old` - Delete sessions older than this many days
    ///
    /// # Returns
    /// Result containing number of deleted sessions or error
    #[instrument(skip(self))]
    pub async fn cleanup_old_sessions(&self, days_old: i64) -> DatabaseResult<u64> {
        let cutoff_date = Utc::now() - Duration::days(days_old);
        let delete_sql = r#"
            DELETE FROM sessions
            WHERE created_at < $1
            AND (status = 'Expired' OR status = 'Revoked')
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(cutoff_date)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Deleted {} old sessions", result.rows_affected());
        Ok(result.rows_affected())
    }

    /// Mark expired sessions
    ///
    /// # Returns
    /// Result containing number of marked sessions or error
    #[instrument(skip(self))]
    pub async fn mark_expired_sessions(&self) -> DatabaseResult<u64> {
        let update_sql = r#"
            UPDATE sessions
            SET status = 'Expired'
            WHERE expires_at < NOW() AND status = 'Active'
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() > 0 {
            info!("Marked {} sessions as expired", result.rows_affected());
        }

        Ok(result.rows_affected())
    }

    /// Count active sessions for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// Result containing session count or error
    pub async fn count_active(&self, user_id: &str) -> DatabaseResult<i64> {
        let count_sql = r#"
            SELECT COUNT(*) as count
            FROM sessions
            WHERE user_id = $1 AND status = 'Active' AND expires_at > NOW()
        "#;

        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(count_sql)
            .bind(parse_uuid(user_id, "user_id")?)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Count all sessions (including expired)
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// Result containing session count or error
    pub async fn count_all(&self, user_id: &str) -> DatabaseResult<i64> {
        let count_sql = "SELECT COUNT(*) as count FROM sessions WHERE user_id = $1";

        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(count_sql)
            .bind(parse_uuid(user_id, "user_id")?)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Get session statistics
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// Result containing statistics HashMap or error
    pub async fn get_statistics(&self, user_id: &str) -> DatabaseResult<serde_json::Value> {
        let mut stats = serde_json::Map::new();

        let active_count = self.count_active(user_id).await?;
        stats.insert(
            "active_sessions".to_string(),
            serde_json::json!(active_count),
        );

        let all_count = self.count_all(user_id).await?;
        stats.insert("total_sessions".to_string(), serde_json::json!(all_count));

        let expired_count = all_count - active_count;
        stats.insert(
            "expired_sessions".to_string(),
            serde_json::json!(expired_count),
        );

        Ok(serde_json::Value::Object(stats))
    }

    /// Validate a session (check if valid and update activity)
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// Result indicating if session is valid or error
    #[instrument(skip(self))]
    pub async fn validate_session(&self, session_id: &SessionId) -> DatabaseResult<bool> {
        let session: SessionRecord = self.get_by_id(session_id).await?;

        if !session.is_valid() {
            if session.is_expired() {
                // Mark as expired
                let update_sql = "UPDATE sessions SET status = 'Expired' WHERE id = $1";
                let mut conn = self.pool.acquire().await?;
                query(update_sql)
                    .bind(parse_uuid(&session_id.as_str(), "session_id")?)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
                return Err(DatabaseError::session_expired(session_id.as_str()));
            } else {
                return Err(DatabaseError::session_not_found(session_id.as_str()));
            }
        }

        // Update last activity
        self.update_activity(session_id).await?;
        Ok(true)
    }

    /// Extend session expiration
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    /// * `additional_duration` - Additional duration to add
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn extend_expiration(
        &self,
        session_id: &SessionId,
        additional_duration: Duration,
    ) -> DatabaseResult<()> {
        let new_expires_at = Utc::now() + additional_duration;
        let update_sql = "UPDATE sessions SET expires_at = $1 WHERE id = $2";

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(new_expires_at)
            .bind(parse_uuid(&session_id.as_str(), "session_id")?)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::session_not_found(session_id.as_str()));
        }

        info!("Session expiration extended: {}", session_id.as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session_record(status: &str, expires_at: DateTime<Utc>) -> SessionRecord {
        SessionRecord {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            session_type: "web".to_string(),
            status: status.to_string(),
            token_value: "token-abc".to_string(),
            token_type: "jwt".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            device_info: None,
            created_at: Utc::now() - Duration::hours(1),
            expires_at,
            last_activity: Utc::now(),
        }
    }

    #[test]
    fn test_session_record_is_expired_true() {
        let record = make_session_record("Active", Utc::now() - Duration::hours(1));
        assert!(record.is_expired());
    }

    #[test]
    fn test_session_record_is_expired_false() {
        let record = make_session_record("Active", Utc::now() + Duration::hours(1));
        assert!(!record.is_expired());
    }

    #[test]
    fn test_session_record_is_valid_active_and_not_expired() {
        let record = make_session_record("Active", Utc::now() + Duration::hours(1));
        assert!(record.is_valid());
    }

    #[test]
    fn test_session_record_is_invalid_when_expired() {
        let record = make_session_record("Active", Utc::now() - Duration::seconds(1));
        assert!(!record.is_valid());
    }

    #[test]
    fn test_session_record_is_invalid_when_revoked() {
        let record = make_session_record("Revoked", Utc::now() + Duration::hours(1));
        assert!(!record.is_valid());
    }

    #[test]
    fn test_session_record_is_invalid_when_status_not_active() {
        let record = make_session_record("Expired", Utc::now() + Duration::hours(1));
        assert!(!record.is_valid());
    }

    #[test]
    fn test_session_record_fields() {
        let record = make_session_record("Active", Utc::now() + Duration::hours(1));
        assert_eq!(record.id, "sess-1");
        assert_eq!(record.user_id, "user-1");
        assert_eq!(record.session_type, "web");
        assert_eq!(record.token_value, "token-abc");
        assert_eq!(record.ip_address.as_deref(), Some("127.0.0.1"));
        assert!(record.device_info.is_none());
    }

    #[test]
    fn test_database_error_session_expired() {
        let err = DatabaseError::session_expired("sess-123");
        let msg = err.to_string();
        assert!(msg.contains("sess-123"));
        assert!(msg.contains("expired"));
    }

    #[test]
    fn test_database_error_session_not_found() {
        let err = DatabaseError::session_not_found("sess-456");
        let msg = err.to_string();
        assert!(msg.contains("sess-456"));
        assert!(msg.contains("not found"));
    }
}
