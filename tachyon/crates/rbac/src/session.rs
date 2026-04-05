// Session Management Module
// Secure session storage and lifecycle management with SQLite persistence

use crate::error::{RbacError, RbacResult};
use crate::{SessionId, UserId};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Session Store
// ============================================================================

/// Session record for database storage
#[derive(Debug, Clone)]
struct SessionRecord {
    /// Session ID
    id: String,
    /// User ID
    user_id: String,
    /// Session type
    session_type: String,
    /// Session status
    status: String,
    /// IP address
    ip_address: Option<String>,
    /// User agent
    user_agent: Option<String>,
    /// Device info
    device_info: Option<String>,
    /// Created at timestamp
    created_at: DateTime<Utc>,
    /// Expires at timestamp
    expires_at: DateTime<Utc>,
    /// Last activity timestamp
    last_activity: DateTime<Utc>,
    /// Token value
    token_value: String,
    /// Token type
    token_type: String,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for SessionRecord {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            session_type: row.try_get("session_type")?,
            status: row.try_get("status")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
            device_info: row.try_get("device_info")?,
            created_at: row.try_get("created_at")?,
            expires_at: row.try_get("expires_at")?,
            last_activity: row.try_get("last_activity")?,
            token_value: row.try_get("token_value")?,
            token_type: row.try_get("token_type")?,
        })
    }
}

// ============================================================================
// Session Manager
// ============================================================================

/// Session manager for secure session lifecycle management
#[derive(Clone)]
pub struct SessionManager {
    /// Database pool
    db_pool: Arc<Pool<Sqlite>>,
    /// In-memory session cache
    cache: Arc<DashMap<SessionId, SessionRecord>>,
    /// Maximum sessions per user
    max_sessions_per_user: usize,
}

impl SessionManager {
    /// Create a new session manager
    ///
    /// # Arguments
    /// * `database_url` - SQLite database URL
    ///
    /// # Returns
    /// Result containing the SessionManager or error
    pub async fn new(database_url: &str) -> RbacResult<Self> {
        let db_pool = Self::create_pool(database_url).await?;
        Self::initialize_database(&db_pool).await?;

        Ok(Self {
            db_pool: Arc::new(db_pool),
            cache: Arc::new(DashMap::new()),
            max_sessions_per_user: 5,
        })
    }

    /// Create a new session manager with custom max sessions per user
    ///
    /// # Arguments
    /// * `database_url` - SQLite database URL
    /// * `max_sessions_per_user` - Maximum sessions per user
    ///
    /// # Returns
    /// Result containing the SessionManager or error
    pub async fn with_max_sessions(
        database_url: &str,
        max_sessions_per_user: usize,
    ) -> RbacResult<Self> {
        let mut manager = Self::new(database_url).await?;
        manager.max_sessions_per_user = max_sessions_per_user;
        Ok(manager)
    }

    /// Create database connection pool
    ///
    /// # Arguments
    /// * `database_url` - SQLite database URL
    ///
    /// # Returns
    /// Result containing the database pool or error
    async fn create_pool(database_url: &str) -> RbacResult<Pool<Sqlite>> {
        Pool::connect(database_url)
            .await
            .map_err(|e| RbacError::database_error(format!("Failed to connect to database: {}", e)))
    }

    /// Initialize database schema
    ///
    /// # Arguments
    /// * `pool` - Database pool
    ///
    /// # Returns
    /// Result indicating success or error
    async fn initialize_database(pool: &Pool<Sqlite>) -> RbacResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY NOT NULL,
                user_id TEXT NOT NULL,
                session_type TEXT NOT NULL,
                status TEXT NOT NULL,
                ip_address TEXT,
                user_agent TEXT,
                device_info TEXT,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_activity TEXT NOT NULL,
                token_value TEXT NOT NULL,
                token_type TEXT NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| RbacError::database_error(format!("Failed to initialize database: {}", e)))?;

        Ok(())
    }

    /// Create a new session
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `session_id` - Session ID
    /// * `session_type` - Session type
    /// * `token_value` - Token value
    /// * `token_type` - Token type
    /// * `expires_in` - Duration until expiration
    /// * `ip_address` - Optional IP address
    /// * `user_agent` - Optional user agent
    /// * `device_info` - Optional device info
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn create_session(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
        session_type: &str,
        token_value: &str,
        token_type: &str,
        expires_in: Duration,
        ip_address: Option<String>,
        user_agent: Option<String>,
        device_info: Option<String>,
    ) -> RbacResult<()> {
        // Check max sessions per user
        let session_count = self.get_user_session_count(user_id).await?;
        if session_count >= self.max_sessions_per_user {
            return Err(RbacError::session_error(format!(
                "Maximum sessions ({}) reached for user",
                self.max_sessions_per_user
            )));
        }

        let now = Utc::now();
        let expires_at = now + expires_in;

        let record = SessionRecord {
            id: session_id.as_str(),
            user_id: user_id.as_str(),
            session_type: session_type.to_string(),
            status: "active".to_string(),
            ip_address,
            user_agent,
            device_info,
            created_at: now,
            expires_at,
            last_activity: now,
            token_value: token_value.to_string(),
            token_type: token_type.to_string(),
        };

        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, user_id, session_type, status,
                ip_address, user_agent, device_info,
                created_at, expires_at, last_activity,
                token_value, token_type
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record.id)
        .bind(&record.user_id)
        .bind(&record.session_type)
        .bind(&record.status)
        .bind(&record.ip_address)
        .bind(&record.user_agent)
        .bind(&record.device_info)
        .bind(&record.created_at)
        .bind(&record.expires_at)
        .bind(&record.last_activity)
        .bind(&record.token_value)
        .bind(&record.token_type)
        .execute(&*self.db_pool)
        .await
        .map_err(|e| RbacError::database_error(format!("Failed to create session: {}", e)))?;

        // Cache the session
        self.cache.insert(session_id.clone(), record);

        Ok(())
    }

    /// Get a session by ID
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// Result containing the session record or error
    pub async fn get_session(&self, session_id: &SessionId) -> RbacResult<SessionRecord> {
        // Check cache first
        if let Some(cached) = self.cache.get(session_id) {
            return Ok(cached.clone());
        }

        let result = sqlx::query_as::<_, SessionRecord>("SELECT * FROM sessions WHERE id = ?")
            .bind(session_id.as_str())
            .fetch_one(&*self.db_pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::RowNotFound = e {
                    RbacError::not_found(format!("Session not found: {}", session_id))
                } else {
                    RbacError::database_error(format!("Failed to get session: {}", e))
                }
            })?;

        // Cache the result
        self.cache.insert(session_id.clone(), result.clone());

        Ok(result)
    }

    /// Get all sessions for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// Result containing the session records or error
    pub async fn get_user_sessions(&self, user_id: &UserId) -> RbacResult<Vec<SessionRecord>> {
        let records = sqlx::query_as::<_, SessionRecord>(
            "SELECT * FROM sessions WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id.as_str())
        .fetch_all(&*self.db_pool)
        .await
        .map_err(|e| RbacError::database_error(format!("Failed to get user sessions: {}", e)))?;

        Ok(records)
    }

    /// Get session count for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// Result containing the session count or error
    async fn get_user_session_count(&self, user_id: &UserId) -> RbacResult<usize> {
        let count: (i64,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM sessions WHERE user_id = ? AND status = 'active'",
        )
        .bind(user_id.as_str())
        .fetch_one(&*self.db_pool)
        .await
        .map_err(|e| RbacError::database_error(format!("Failed to count sessions: {}", e)))?;

        Ok(count.0 as usize)
    }

    /// Update session last activity
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn update_activity(&self, session_id: &SessionId) -> RbacResult<()> {
        let now = Utc::now();

        sqlx::query("UPDATE sessions SET last_activity = ? WHERE id = ?")
            .bind(now)
            .bind(session_id.as_str())
            .execute(&*self.db_pool)
            .await
            .map_err(|e| RbacError::database_error(format!("Failed to update activity: {}", e)))?;

        // Update cache
        if let Some(mut cached) = self.cache.get_mut(session_id) {
            cached.last_activity = now;
        }

        Ok(())
    }

    /// Extend session expiration
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    /// * `additional_time` - Additional duration
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn extend_session(
        &self,
        session_id: &SessionId,
        additional_time: Duration,
    ) -> RbacResult<()> {
        let new_expires_at = Utc::now() + additional_time;

        sqlx::query("UPDATE sessions SET expires_at = ? WHERE id = ?")
            .bind(new_expires_at)
            .bind(session_id.as_str())
            .execute(&*self.db_pool)
            .await
            .map_err(|e| RbacError::database_error(format!("Failed to extend session: {}", e)))?;

        // Update cache
        if let Some(mut cached) = self.cache.get_mut(session_id) {
            cached.expires_at = new_expires_at;
        }

        Ok(())
    }

    /// Revoke a session
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn revoke_session(&self, session_id: &SessionId) -> RbacResult<()> {
        sqlx::query("UPDATE sessions SET status = 'revoked' WHERE id = ?")
            .bind(session_id.as_str())
            .execute(&*self.db_pool)
            .await
            .map_err(|e| RbacError::database_error(format!("Failed to revoke session: {}", e)))?;

        // Update cache
        if let Some(mut cached) = self.cache.get_mut(session_id) {
            cached.status = "revoked".to_string();
        }

        Ok(())
    }

    /// Revoke all sessions for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn revoke_user_sessions(&self, user_id: &UserId) -> RbacResult<()> {
        sqlx::query("UPDATE sessions SET status = 'revoked' WHERE user_id = ?")
            .bind(user_id.as_str())
            .execute(&*self.db_pool)
            .await
            .map_err(|e| {
                RbacError::database_error(format!("Failed to revoke user sessions: {}", e))
            })?;

        // Clear cache for user's sessions
        self.cache
            .retain(|_, record| record.user_id != user_id.as_str());

        Ok(())
    }

    /// Clean up expired sessions
    ///
    /// # Returns
    /// Result containing the number of sessions cleaned or error
    pub async fn cleanup_expired_sessions(&self) -> RbacResult<usize> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE sessions SET status = 'expired' WHERE expires_at < ? AND status = 'active'",
        )
        .bind(now)
        .execute(&*self.db_pool)
        .await
        .map_err(|e| RbacError::database_error(format!("Failed to cleanup sessions: {}", e)))?;

        let count = result.rows_affected();

        // Clear expired sessions from cache
        self.cache
            .retain(|_, record| record.status != "expired" && record.expires_at > now);

        Ok(count as usize)
    }

    /// Validate a session
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// Result indicating if session is valid or error
    pub async fn validate_session(&self, session_id: &SessionId) -> RbacResult<()> {
        let session = self.get_session(session_id).await?;

        if session.status != "active" {
            return Err(RbacError::session_error(format!(
                "Session is {}",
                session.status
            )));
        }

        if session.expires_at < Utc::now() {
            return Err(RbacError::session_error("Session has expired".to_string()));
        }

        Ok(())
    }

    /// Clear session cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// Current cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

// ============================================================================
// Session Store Trait
// ============================================================================

/// Trait for session storage backends
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a session
    async fn create_session(&self, session: SessionRecord) -> RbacResult<()>;

    /// Get a session
    async fn get_session(&self, session_id: &SessionId) -> RbacResult<SessionRecord>;

    /// Update a session
    async fn update_session(&self, session: SessionRecord) -> RbacResult<()>;

    /// Delete a session
    async fn delete_session(&self, session_id: &SessionId) -> RbacResult<()>;

    /// List sessions for user
    async fn list_sessions(&self, user_id: &UserId) -> RbacResult<Vec<SessionRecord>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_record_creation() {
        let user_id = UserId::new();
        let session_id = SessionId::new();

        let record = SessionRecord {
            id: session_id.as_str(),
            user_id: user_id.as_str(),
            session_type: "web".to_string(),
            status: "active".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            device_info: None,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            last_activity: Utc::now(),
            token_value: "test-token".to_string(),
            token_type: "bearer".to_string(),
        };

        assert_eq!(record.status, "active");
        assert_eq!(record.session_type, "web");
    }
}
