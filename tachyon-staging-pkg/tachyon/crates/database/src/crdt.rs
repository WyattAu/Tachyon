//! CRDT document state persistence.
//!
//! Stores binary Yrs document state and update logs in PostgreSQL,
//! enabling CRDT state survival across server restarts and LRU evictions.

use sqlx::PgPool;
use uuid::Uuid;

/// CRDT document state record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CrdtDocumentRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub state_vector: Vec<u8>,
    pub state: Vec<u8>,
    pub version: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// CRDT update log record.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CrdtUpdateRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub update: Vec<u8>,
    pub client_id: Option<Uuid>,
    pub seq: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Save or update CRDT document state (upsert).
pub async fn upsert_crdt_state(
    pool: &PgPool,
    document_id: Uuid,
    state_vector: &[u8],
    state: &[u8],
) -> sqlx::Result<CrdtDocumentRow> {
    let row = sqlx::query_as::<_, CrdtDocumentRow>(
        r#"
        INSERT INTO crdt_documents (document_id, state_vector, state, version, updated_at)
        VALUES ($1, $2, $3, 1, now())
        ON CONFLICT (document_id) DO UPDATE SET
            state_vector = EXCLUDED.state_vector,
            state = EXCLUDED.state,
            version = crdt_documents.version + 1,
            updated_at = now()
        RETURNING *
        "#,
    )
    .bind(document_id)
    .bind(state_vector)
    .bind(state)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Load CRDT document state from the database.
pub async fn load_crdt_state(
    pool: &PgPool,
    document_id: Uuid,
) -> sqlx::Result<Option<CrdtDocumentRow>> {
    sqlx::query_as::<_, CrdtDocumentRow>("SELECT * FROM crdt_documents WHERE document_id = $1")
        .bind(document_id)
        .fetch_optional(pool)
        .await
}

/// Append a CRDT update to the log.
pub async fn append_update(
    pool: &PgPool,
    document_id: Uuid,
    update: &[u8],
    client_id: Option<Uuid>,
) -> sqlx::Result<CrdtUpdateRow> {
    let row = sqlx::query_as::<_, CrdtUpdateRow>(
        r#"
        INSERT INTO crdt_updates (document_id, update, client_id, seq)
        VALUES ($1, $2, $3, COALESCE(
            (SELECT MAX(seq) FROM crdt_updates WHERE document_id = $1), 0
        ) + 1)
        RETURNING *
        "#,
    )
    .bind(document_id)
    .bind(update)
    .bind(client_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Load recent updates for a document since a given sequence number.
pub async fn load_updates_since(
    pool: &PgPool,
    document_id: Uuid,
    since_seq: i64,
    limit: i64,
) -> sqlx::Result<Vec<CrdtUpdateRow>> {
    sqlx::query_as::<_, CrdtUpdateRow>(
        "SELECT * FROM crdt_updates WHERE document_id = $1 AND seq > $2 ORDER BY seq ASC LIMIT $3",
    )
    .bind(document_id)
    .bind(since_seq)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Delete old update log entries (garbage collection).
/// Keeps only the most recent `keep_count` updates per document.
pub async fn gc_updates(pool: &PgPool, document_id: Uuid, keep_count: i64) -> sqlx::Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM crdt_updates
        WHERE document_id = $1
        AND id NOT IN (
            SELECT id FROM crdt_updates
            WHERE document_id = $1
            ORDER BY seq DESC
            LIMIT $2
        )
        "#,
    )
    .bind(document_id)
    .bind(keep_count)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Delete all CRDT state for a document.
pub async fn delete_crdt_state(pool: &PgPool, document_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM crdt_updates WHERE document_id = $1")
        .bind(document_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM crdt_documents WHERE document_id = $1")
        .bind(document_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_document_row_fields() {
        // Verify the struct fields match expected names
        let _ = |row: CrdtDocumentRow| {
            let _ = row.id;
            let _ = row.document_id;
            let _ = row.state_vector;
            let _ = row.state;
            let _ = row.version;
            let _ = row.created_at;
            let _ = row.updated_at;
        };
    }

    #[test]
    fn test_crdt_update_row_fields() {
        let _ = |row: CrdtUpdateRow| {
            let _ = row.id;
            let _ = row.document_id;
            let _ = row.update;
            let _ = row.client_id;
            let _ = row.seq;
            let _ = row.created_at;
        };
    }
}
