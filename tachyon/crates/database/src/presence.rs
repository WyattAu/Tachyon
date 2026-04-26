//! Presence repository for real-time collaboration
//!
//! Presence records are ephemeral — they represent who is currently viewing
//! a document. Queries filter by `last_seen_at` to return only "live" presence.
//! A periodic cleanup task should call `purge_stale()` to remove expired rows.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

const PRESENCE_SELECT_SQL: &str = r#"
    SELECT
        id::TEXT as id,
        user_id::TEXT as user_id,
        user_name,
        document_id::TEXT as document_id,
        status,
        cursor_section,
        cursor_line,
        cursor_selection,
        connected_at::TEXT as connected_at,
        last_seen_at::TEXT as last_seen_at
    FROM document_presence
"#;

/// A presence record represents a user actively viewing a document
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Presence {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub document_id: String,
    pub status: String,
    pub cursor_section: Option<String>,
    pub cursor_line: Option<i32>,
    pub cursor_selection: Option<String>,
    pub connected_at: String,
    pub last_seen_at: String,
}

/// Request to upsert presence (create or update)
#[derive(Debug, Deserialize)]
pub struct UpsertPresenceRequest {
    pub user_id: String,
    pub user_name: String,
    pub document_id: String,
    pub status: Option<String>,
    pub cursor_section: Option<String>,
    pub cursor_line: Option<i32>,
    pub cursor_selection: Option<String>,
}

/// Request to update an existing presence record
#[derive(Debug, Deserialize)]
pub struct UpdatePresenceRequest {
    pub status: Option<String>,
    pub cursor_section: Option<String>,
    pub cursor_line: Option<i32>,
    pub cursor_selection: Option<String>,
}

#[derive(Clone)]
pub struct PresenceRepository {
    pool: DatabasePool,
}

/// Default TTL for presence records (5 minutes)
pub const PRESENCE_TTL_SECS: i64 = 300;

impl PresenceRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Upsert presence: create if new, update if user already has presence on this document.
    /// Uses ON CONFLICT for atomic upsert — the most common operation.
    #[instrument(skip(self, req))]
    pub async fn upsert(&self, req: UpsertPresenceRequest) -> DatabaseResult<Presence> {
        let status = req.status.unwrap_or_else(|| "active".to_string());

        let sql = r#"
            INSERT INTO document_presence
                (user_id, user_name, document_id, status,
                 cursor_section, cursor_line, cursor_selection,
                 connected_at, last_seen_at)
            VALUES ($1::uuid, $2, $3::uuid, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (user_id, document_id) DO UPDATE SET
                user_name      = EXCLUDED.user_name,
                status         = EXCLUDED.status,
                cursor_section = EXCLUDED.cursor_section,
                cursor_line    = EXCLUDED.cursor_line,
                cursor_selection = EXCLUDED.cursor_selection,
                last_seen_at   = NOW()
            RETURNING
                id::TEXT as id,
                user_id::TEXT as user_id,
                user_name,
                document_id::TEXT as document_id,
                status,
                cursor_section,
                cursor_line,
                cursor_selection,
                connected_at::TEXT as connected_at,
                last_seen_at::TEXT as last_seen_at
        "#;

        sqlx::query_as::<_, Presence>(sql)
            .bind(&req.user_id)
            .bind(&req.user_name)
            .bind(&req.document_id)
            .bind(&status)
            .bind(&req.cursor_section)
            .bind(req.cursor_line)
            .bind(&req.cursor_selection)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))
    }

    /// Touch a presence record — update only `last_seen_at` to keep it alive.
    /// This is a lightweight heartbeat operation.
    #[instrument(skip(self))]
    pub async fn touch(&self, user_id: &str, document_id: &str) -> DatabaseResult<()> {
        let sql = r#"
            UPDATE document_presence
            SET last_seen_at = NOW()
            WHERE user_id = $1::uuid AND document_id = $2::uuid
        "#;

        sqlx::query(sql)
            .bind(user_id)
            .bind(document_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(())
    }

    /// Update specific fields on an existing presence record.
    #[instrument(skip(self, req))]
    pub async fn update(
        &self,
        user_id: &str,
        document_id: &str,
        req: UpdatePresenceRequest,
    ) -> DatabaseResult<Presence> {
        // First check the record exists
        let existing = self
            .get_by_user_and_document(user_id, document_id)
            .await
            .map_err(|_| DatabaseError::not_found("presence", format!("{}/{}", user_id, document_id)))?;

        let status = req.status.unwrap_or(existing.status);
        let cursor_section = req.cursor_section.or(existing.cursor_section);
        let cursor_line = req.cursor_line.or(existing.cursor_line);
        let cursor_selection = req.cursor_selection.or(existing.cursor_selection);

        let sql = r#"
            UPDATE document_presence SET
                status = $3,
                cursor_section = $4,
                cursor_line = $5,
                cursor_selection = $6,
                last_seen_at = NOW()
            WHERE user_id = $1::uuid AND document_id = $2::uuid
            RETURNING
                id::TEXT as id,
                user_id::TEXT as user_id,
                user_name,
                document_id::TEXT as document_id,
                status,
                cursor_section,
                cursor_line,
                cursor_selection,
                connected_at::TEXT as connected_at,
                last_seen_at::TEXT as last_seen_at
        "#;

        sqlx::query_as::<_, Presence>(sql)
            .bind(user_id)
            .bind(document_id)
            .bind(&status)
            .bind(&cursor_section)
            .bind(cursor_line)
            .bind(&cursor_selection)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))
    }

    /// Get a single presence record by user + document
    #[instrument(skip(self))]
    pub async fn get_by_user_and_document(
        &self,
        user_id: &str,
        document_id: &str,
    ) -> DatabaseResult<Presence> {
        let sql = format!(
            "{} WHERE user_id = $1::uuid AND document_id = $2::uuid",
            PRESENCE_SELECT_SQL
        );

        sqlx::query_as::<_, Presence>(&sql)
            .bind(user_id)
            .bind(document_id)
            .fetch_optional(self.pool.inner())
            .await?
            .ok_or_else(|| {
                DatabaseError::not_found("presence", format!("{}/{}", user_id, document_id))
            })
    }

    /// List all "live" presence records for a document.
    /// Only returns records where `last_seen_at` is within the TTL window.
    #[instrument(skip(self))]
    pub async fn list_by_document(&self, document_id: &str) -> DatabaseResult<Vec<Presence>> {
        let sql = format!(
            "{} WHERE document_id = $1::uuid AND last_seen_at > NOW() - INTERVAL '{} seconds' \
             ORDER BY last_seen_at DESC LIMIT 200",
            PRESENCE_SELECT_SQL, PRESENCE_TTL_SECS
        );

        let rows = sqlx::query_as::<_, Presence>(&sql)
            .bind(document_id)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(rows)
    }

    /// List all documents a user is currently present on
    #[instrument(skip(self))]
    pub async fn list_by_user(&self, user_id: &str) -> DatabaseResult<Vec<Presence>> {
        let sql = format!(
            "{} WHERE user_id = $1::uuid AND last_seen_at > NOW() - INTERVAL '{} seconds' \
             ORDER BY last_seen_at DESC LIMIT 100",
            PRESENCE_SELECT_SQL, PRESENCE_TTL_SECS
        );

        let rows = sqlx::query_as::<_, Presence>(&sql)
            .bind(user_id)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(rows)
    }

    /// Remove a user's presence from a specific document.
    /// Returns true if a record was actually deleted.
    #[instrument(skip(self))]
    pub async fn remove(&self, user_id: &str, document_id: &str) -> DatabaseResult<bool> {
        let sql = "DELETE FROM document_presence WHERE user_id = $1::uuid AND document_id = $2::uuid";

        let result = sqlx::query(sql)
            .bind(user_id)
            .bind(document_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Remove all presence for a user (e.g., on logout/disconnect).
    /// Returns the number of records removed.
    #[instrument(skip(self))]
    pub async fn remove_all_for_user(&self, user_id: &str) -> DatabaseResult<u64> {
        let sql = "DELETE FROM document_presence WHERE user_id = $1::uuid";

        let result = sqlx::query(sql)
            .bind(user_id)
            .execute(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Purge all presence records older than the TTL.
    /// Should be called periodically (e.g., every 60 seconds by a background task).
    /// Returns the number of purged records.
    #[instrument(skip(self))]
    pub async fn purge_stale(&self) -> DatabaseResult<u64> {
        let sql = format!(
            "DELETE FROM document_presence WHERE last_seen_at < NOW() - INTERVAL '{} seconds'",
            PRESENCE_TTL_SECS
        );

        let result = sqlx::query(&sql)
            .execute(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Count of live presence records for a specific document
    #[instrument(skip(self))]
    pub async fn count_by_document(&self, document_id: &str) -> DatabaseResult<i64> {
        let sql = format!(
            "SELECT COUNT(*) as count FROM document_presence \
             WHERE document_id = $1::uuid AND last_seen_at > NOW() - INTERVAL '{} seconds'",
            PRESENCE_TTL_SECS
        );

        let count = sqlx::query_scalar::<_, i64>(&sql)
            .bind(document_id)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(count)
    }
}
