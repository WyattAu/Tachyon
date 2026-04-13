// Sync Queue — Offline→Online operation journal
//
// Records document mutations made while offline so they can be
// replayed against the remote server when connectivity is restored.
//
// Each entry captures:
// - The operation type (create, update_content, update_metadata, delete)
// - The document ID (for updates/deletes) or the full document snapshot (for creates)
// - A JSON payload with the mutation details
// - Timestamp and retry count

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, Row};
use std::path::Path;
use std::str::FromStr;
use tachyon_core::types::storage::StorageError;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The kind of document mutation that was recorded while offline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncOperation {
    /// A brand-new document was created locally.
    Create,
    /// The content of an existing document was edited.
    UpdateContent,
    /// The metadata (title, tags, slug, etc.) was changed.
    UpdateMetadata,
    /// The document was soft-deleted.
    Delete,
    /// The document was permanently deleted.
    PermanentDelete,
}

impl std::fmt::Display for SyncOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::UpdateContent => write!(f, "update_content"),
            Self::UpdateMetadata => write!(f, "update_metadata"),
            Self::Delete => write!(f, "delete"),
            Self::PermanentDelete => write!(f, "permanent_delete"),
        }
    }
}

impl SyncOperation {
    /// Parse from the string stored in SQLite.
    fn from_str_lossy(s: &str) -> Self {
        match s {
            "create" => Self::Create,
            "update_content" => Self::UpdateContent,
            "update_metadata" => Self::UpdateMetadata,
            "delete" => Self::Delete,
            "permanent_delete" => Self::PermanentDelete,
            _ => {
                warn!("Unknown sync operation '{}', defaulting to update_content", s);
                Self::UpdateContent
            }
        }
    }
}

/// Processing status of a queued sync entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntryStatus {
    /// Waiting to be synced.
    Pending,
    /// Currently being synced (in-flight).
    InFlight,
    /// Successfully synced with the remote server.
    Synced,
    /// Failed — will be retried.
    Failed,
}

impl std::fmt::Display for SyncEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InFlight => write!(f, "in_flight"),
            Self::Synced => write!(f, "synced"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl SyncEntryStatus {
    fn from_str_lossy(s: &str) -> Self {
        match s {
            "in_flight" => Self::InFlight,
            "synced" => Self::Synced,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

/// A single queued mutation awaiting sync to the remote server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueEntry {
    /// Unique entry ID (UUID v7 for time-ordering).
    pub id: String,
    /// The type of mutation.
    pub operation: SyncOperation,
    /// The document ID this mutation targets.
    /// For `Create` operations this is the locally-assigned ID.
    pub document_id: String,
    /// JSON payload containing the mutation data.
    /// - Create: full `DocumentMetadata` + `DocumentContent`
    /// - UpdateContent: `{ "content": "..." }`
    /// - UpdateMetadata: full `DocumentMetadata`
    /// - Delete / PermanentDelete: `null`
    pub payload: Option<String>,
    /// Current processing status.
    pub status: SyncEntryStatus,
    /// How many times we've attempted to sync this entry.
    pub retry_count: u32,
    /// When the entry was created (locally).
    pub created_at: DateTime<Utc>,
    /// Last attempt timestamp (if any).
    pub last_attempt_at: Option<DateTime<Utc>>,
    /// Error message from the last failed attempt.
    pub last_error: Option<String>,
}

/// Summary of the sync queue state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncQueueSummary {
    /// Number of entries waiting to be synced.
    pub pending_count: usize,
    /// Number of entries currently in-flight.
    pub in_flight_count: usize,
    /// Number of successfully synced entries.
    pub synced_count: usize,
    /// Number of failed entries awaiting retry.
    pub failed_count: usize,
    /// Total entries in the queue (including synced, kept for audit).
    pub total_count: usize,
}

/// Result of a flush (sync batch) operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushResult {
    /// Number of entries successfully synced in this batch.
    pub synced: usize,
    /// Number of entries that failed.
    pub failed: usize,
    /// Number of entries skipped (already synced or in-flight).
    pub skipped: usize,
}

// ---------------------------------------------------------------------------
// SyncQueue
// ---------------------------------------------------------------------------

/// Persistent sync queue backed by a SQLite table.
///
/// Shares the same SQLite database as `SqliteStore` but uses its own
/// `sync_queue` table. Entries are ordered by `created_at` so the oldest
/// mutations are replayed first.
pub struct SyncQueue {
    pool: sqlx::SqlitePool,
}

impl SyncQueue {
    /// Open (or create) the sync queue backed by the SQLite database at `path`.
    ///
    /// If the database already exists the `sync_queue` table is created
    /// idempotently (CREATE TABLE IF NOT EXISTS).
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path_str))
            .map_err(|e| StorageError::Unavailable {
                reason: format!("Invalid SQLite path: {}", e),
            })?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = sqlx::SqlitePool::connect_with(options)
            .await
            .map_err(|e| StorageError::Unavailable {
                reason: format!("Failed to open SQLite database for sync queue: {}", e),
            })?;

        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    /// Create an in-memory sync queue (for tests).
    pub async fn in_memory() -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|e| StorageError::Unavailable {
                reason: format!("Invalid SQLite URI: {}", e),
            })?
            .create_if_missing(true);

        let pool = sqlx::SqlitePool::connect_with(options)
            .await
            .map_err(|e| StorageError::Unavailable {
                reason: format!("Failed to open in-memory SQLite: {}", e),
            })?;

        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_schema(pool: &sqlx::SqlitePool) -> Result<(), StorageError> {
        let mut conn = pool.acquire().await.map_err(|e| StorageError::Internal {
            message: format!("Failed to acquire connection: {}", e),
        })?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sync_queue (
                id              TEXT PRIMARY KEY,
                operation       TEXT NOT NULL,
                document_id     TEXT NOT NULL,
                payload         TEXT,
                status          TEXT NOT NULL DEFAULT 'pending',
                retry_count     INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL,
                last_attempt_at TEXT,
                last_error      TEXT
            )",
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to create sync_queue table: {}", e),
        })?;

        // Index for efficient pending-entry queries
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_sync_queue_status
             ON sync_queue(status, created_at)",
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to create sync_queue index: {}", e),
        })?;

        info!("sync_queue schema initialized");
        Ok(())
    }

    // ---- Enqueue ----

    /// Enqueue a mutation for later sync.
    pub async fn enqueue(
        &self,
        operation: SyncOperation,
        document_id: impl AsRef<str>,
        payload: Option<String>,
    ) -> Result<String, StorageError> {
        let id = uuid::Uuid::now_v7().to_string();
        let doc_id = document_id.as_ref();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO sync_queue (id, operation, document_id, payload, status, retry_count, created_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', 0, ?5)",
        )
        .bind(&id)
        .bind(operation.to_string())
        .bind(doc_id)
        .bind(&payload)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to enqueue sync entry: {}", e),
        })?;

        Ok(id)
    }

    // ---- Query ----

    /// Get the current queue summary.
    pub async fn summary(&self) -> Result<SyncQueueSummary, StorageError> {
        let row = sqlx::query(
            "SELECT
                COALESCE(SUM(CASE WHEN status = 'pending'  THEN 1 ELSE 0 END), 0) AS pending,
                COALESCE(SUM(CASE WHEN status = 'in_flight' THEN 1 ELSE 0 END), 0) AS in_flight,
                COALESCE(SUM(CASE WHEN status = 'synced'   THEN 1 ELSE 0 END), 0) AS synced,
                COALESCE(SUM(CASE WHEN status = 'failed'   THEN 1 ELSE 0 END), 0) AS failed,
                COUNT(*) AS total
             FROM sync_queue",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to get sync queue summary: {}", e),
        })?;

        Ok(SyncQueueSummary {
            pending_count: row.try_get::<i64, _>("pending").unwrap_or(0) as usize,
            in_flight_count: row.try_get::<i64, _>("in_flight").unwrap_or(0) as usize,
            synced_count: row.try_get::<i64, _>("synced").unwrap_or(0) as usize,
            failed_count: row.try_get::<i64, _>("failed").unwrap_or(0) as usize,
            total_count: row.try_get::<i64, _>("total").unwrap_or(0) as usize,
        })
    }

    /// Fetch pending entries (oldest first), up to `limit`.
    pub async fn pending_entries(&self, limit: usize) -> Result<Vec<SyncQueueEntry>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, operation, document_id, payload, status, retry_count,
                    created_at, last_attempt_at, last_error
             FROM sync_queue
             WHERE status IN ('pending', 'failed')
             ORDER BY created_at ASC
             LIMIT ?1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to fetch pending entries: {}", e),
        })?;

        rows.iter().map(|r| self.row_to_entry(r)).collect()
    }

    /// Fetch all entries (for debugging / inspection).
    pub async fn all_entries(&self) -> Result<Vec<SyncQueueEntry>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, operation, document_id, payload, status, retry_count,
                    created_at, last_attempt_at, last_error
             FROM sync_queue
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to fetch all entries: {}", e),
        })?;

        rows.iter().map(|r| self.row_to_entry(r)).collect()
    }

    // ---- State transitions ----

    /// Mark an entry as in-flight (being synced).
    pub async fn mark_in_flight(&self, entry_id: &str) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE sync_queue SET status = 'in_flight', last_attempt_at = ?1
             WHERE id = ?2 AND status IN ('pending', 'failed')",
        )
        .bind(&now)
        .bind(entry_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to mark entry in-flight: {}", e),
        })?;
        Ok(())
    }

    /// Mark an entry as successfully synced.
    pub async fn mark_synced(&self, entry_id: &str) -> Result<(), StorageError> {
        sqlx::query(
            "UPDATE sync_queue SET status = 'synced' WHERE id = ?1",
        )
        .bind(entry_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to mark entry synced: {}", e),
        })?;
        Ok(())
    }

    /// Mark an entry as failed with an error message.
    pub async fn mark_failed(&self, entry_id: &str, error: &str) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE sync_queue
             SET status = 'failed',
                 retry_count = retry_count + 1,
                 last_attempt_at = ?1,
                 last_error = ?2
             WHERE id = ?3",
        )
        .bind(&now)
        .bind(error)
        .bind(entry_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to mark entry failed: {}", e),
        })?;
        Ok(())
    }

    // ---- Maintenance ----

    /// Remove entries that have been synced (free space).
    pub async fn purge_synced(&self) -> Result<u64, StorageError> {
        let result = sqlx::query("DELETE FROM sync_queue WHERE status = 'synced'")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to purge synced entries: {}", e),
            })?;
        Ok(result.rows_affected())
    }

    /// Clear the entire queue (for testing / reset).
    pub async fn clear(&self) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM sync_queue")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to clear sync queue: {}", e),
            })?;
        Ok(())
    }

    /// Check if the queue is available.
    pub async fn is_available(&self) -> bool {
        sqlx::query("SELECT 1 FROM sync_queue LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .is_ok()
    }

    // ---- Internal ----

    fn row_to_entry(&self, row: &sqlx::sqlite::SqliteRow) -> Result<SyncQueueEntry, StorageError> {
        let created_at_str: &str = row.try_get("created_at").unwrap_or("");
        let created_at = DateTime::parse_from_rfc3339(created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let last_attempt_str: Option<&str> = row.try_get("last_attempt_at").unwrap_or(None);
        let last_attempt_at = last_attempt_str.and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        Ok(SyncQueueEntry {
            id: row.try_get::<&str, _>("id").unwrap_or_default().to_string(),
            operation: SyncOperation::from_str_lossy(row.try_get::<&str, _>("operation").unwrap_or("")),
            document_id: row.try_get::<&str, _>("document_id").unwrap_or_default().to_string(),
            payload: row.try_get::<Option<&str>, _>("payload").unwrap_or(None).map(|s| s.to_string()),
            status: SyncEntryStatus::from_str_lossy(row.try_get::<&str, _>("status").unwrap_or("")),
            retry_count: row.try_get::<i32, _>("retry_count").unwrap_or(0) as u32,
            created_at,
            last_attempt_at,
            last_error: row.try_get::<Option<&str>, _>("last_error").unwrap_or(None).map(|s| s.to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_and_pending() {
        let queue = SyncQueue::in_memory().await.unwrap();
        assert!(queue.is_available().await);

        let id = queue.enqueue(SyncOperation::Create, "doc-1", Some(r#"{"title":"Hello"}"#.to_string())).await.unwrap();
        assert!(!id.is_empty());

        let entries = queue.pending_entries(10).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].document_id, "doc-1");
        assert_eq!(entries[0].operation, SyncOperation::Create);
        assert_eq!(entries[0].status, SyncEntryStatus::Pending);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let queue = SyncQueue::in_memory().await.unwrap();
        let id = queue.enqueue(SyncOperation::UpdateContent, "doc-2", None).await.unwrap();

        // Pending → InFlight
        queue.mark_in_flight(&id).await.unwrap();
        let entries = queue.pending_entries(10).await.unwrap();
        assert!(entries.is_empty()); // no longer pending

        let all = queue.all_entries().await.unwrap();
        assert_eq!(all[0].status, SyncEntryStatus::InFlight);

        // InFlight → Synced
        queue.mark_synced(&id).await.unwrap();
        let summary = queue.summary().await.unwrap();
        assert_eq!(summary.synced_count, 1);
        assert_eq!(summary.pending_count, 0);
    }

    #[tokio::test]
    async fn test_failure_and_retry() {
        let queue = SyncQueue::in_memory().await.unwrap();
        let id = queue.enqueue(SyncOperation::Delete, "doc-3", None).await.unwrap();

        queue.mark_in_flight(&id).await.unwrap();
        queue.mark_failed(&id, "connection refused").await.unwrap();

        let all = queue.all_entries().await.unwrap();
        assert_eq!(all[0].status, SyncEntryStatus::Failed);
        assert_eq!(all[0].retry_count, 1);
        assert_eq!(all[0].last_error.as_deref(), Some("connection refused"));
        assert!(all[0].last_attempt_at.is_some());

        // Failed entries should show up in pending_entries for retry
        let pending = queue.pending_entries(10).await.unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn test_summary() {
        let queue = SyncQueue::in_memory().await.unwrap();

        queue.enqueue(SyncOperation::Create, "d1", None).await.unwrap();
        queue.enqueue(SyncOperation::UpdateContent, "d2", None).await.unwrap();
        queue.enqueue(SyncOperation::Delete, "d3", None).await.unwrap();

        let summary = queue.summary().await.unwrap();
        assert_eq!(summary.pending_count, 3);
        assert_eq!(summary.total_count, 3);

        // Purge synced (none yet)
        let purged = queue.purge_synced().await.unwrap();
        assert_eq!(purged, 0);
    }

    #[tokio::test]
    async fn test_purge_synced() {
        let queue = SyncQueue::in_memory().await.unwrap();

        let id = queue.enqueue(SyncOperation::Create, "d1", None).await.unwrap();
        queue.mark_in_flight(&id).await.unwrap();
        queue.mark_synced(&id).await.unwrap();

        let purged = queue.purge_synced().await.unwrap();
        assert_eq!(purged, 1);

        let summary = queue.summary().await.unwrap();
        assert_eq!(summary.total_count, 0);
    }

    #[tokio::test]
    async fn test_clear() {
        let queue = SyncQueue::in_memory().await.unwrap();
        queue.enqueue(SyncOperation::Create, "d1", None).await.unwrap();
        queue.enqueue(SyncOperation::Delete, "d2", None).await.unwrap();

        queue.clear().await.unwrap();
        let summary = queue.summary().await.unwrap();
        assert_eq!(summary.total_count, 0);
    }

    #[tokio::test]
    async fn test_multiple_operations() {
        let queue = SyncQueue::in_memory().await.unwrap();

        queue.enqueue(SyncOperation::Create, "new-doc", Some(r#"{"title":"New"}"#.to_string())).await.unwrap();
        queue.enqueue(SyncOperation::UpdateContent, "existing-doc", Some(r#"{"content":"edited"}"#.to_string())).await.unwrap();
        queue.enqueue(SyncOperation::UpdateMetadata, "existing-doc", Some(r#"{"title":"Renamed"}"#.to_string())).await.unwrap();
        queue.enqueue(SyncOperation::Delete, "old-doc", None).await.unwrap();
        queue.enqueue(SyncOperation::PermanentDelete, "gone-doc", None).await.unwrap();

        let entries = queue.pending_entries(10).await.unwrap();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].operation, SyncOperation::Create);
        assert_eq!(entries[4].operation, SyncOperation::PermanentDelete);
    }
}
