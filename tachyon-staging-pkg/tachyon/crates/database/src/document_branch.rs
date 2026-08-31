// Document Branch Repository
// Branch management operations for document branching workflow

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as};
use tracing::{debug, info, instrument};
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DocumentBranchRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub branch_name: String,
    pub source_content: String,
    pub source_content_hash: String,
    pub source_version: i32,
    pub branched_by: Option<Uuid>,
    pub status: String,
    pub merged_at: Option<DateTime<Utc>>,
    pub merged_by: Option<Uuid>,
    pub merge_conflict: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBranchRow {
    pub document_id: Uuid,
    pub branch_name: String,
    pub source_content: String,
    pub source_content_hash: String,
    pub source_version: i32,
    pub branched_by: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBranchRow {
    pub content: String,
    pub content_hash: String,
}

// ============================================================================
// Repository
// ============================================================================

/// Repository for managing document branches.
#[derive(Clone)]
pub struct DocumentBranchRepository {
    pool: DatabasePool,
}

impl DocumentBranchRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create(&self, branch: &CreateBranchRow) -> DatabaseResult<DocumentBranchRow> {
        let mut conn = self.pool.acquire().await?;

        let row: DocumentBranchRow = query_as(
            r#"INSERT INTO document_branches (document_id, branch_name, source_content, source_content_hash, source_version, branched_by, status)
               VALUES ($1, $2, $3, $4, $5, $6, 'open')
               RETURNING id, document_id, branch_name, source_content, source_content_hash, source_version, branched_by, status, merged_at, merged_by, merge_conflict, created_at, updated_at"#,
        )
        .bind(branch.document_id)
        .bind(&branch.branch_name)
        .bind(&branch.source_content)
        .bind(&branch.source_content_hash)
        .bind(branch.source_version)
        .bind(branch.branched_by)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Branch '{}' created for document {}",
            branch.branch_name, branch.document_id
        );
        Ok(row)
    }

    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: Uuid) -> DatabaseResult<Option<DocumentBranchRow>> {
        let mut conn = self.pool.acquire().await?;

        let row: Option<DocumentBranchRow> =
            query_as("SELECT * FROM document_branches WHERE id = $1")
                .bind(id)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row)
    }

    #[instrument(skip(self))]
    pub async fn list_by_document(
        &self,
        document_id: Uuid,
    ) -> DatabaseResult<Vec<DocumentBranchRow>> {
        let mut conn = self.pool.acquire().await?;

        let rows: Vec<DocumentBranchRow> = query_as(
            "SELECT * FROM document_branches WHERE document_id = $1 ORDER BY created_at DESC",
        )
        .bind(document_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!("Found {} branches for document {}", rows.len(), document_id);
        Ok(rows)
    }

    #[instrument(skip(self))]
    pub async fn list_open_by_document(
        &self,
        document_id: Uuid,
    ) -> DatabaseResult<Vec<DocumentBranchRow>> {
        let mut conn = self.pool.acquire().await?;

        let rows: Vec<DocumentBranchRow> = query_as(
            "SELECT * FROM document_branches WHERE document_id = $1 AND status = 'open' ORDER BY created_at DESC",
        )
        .bind(document_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(rows)
    }

    #[instrument(skip(self))]
    pub async fn update_content(
        &self,
        id: Uuid,
        update: &UpdateBranchRow,
    ) -> DatabaseResult<Option<DocumentBranchRow>> {
        let mut conn = self.pool.acquire().await?;

        let row: Option<DocumentBranchRow> = query_as(
            r#"UPDATE document_branches SET source_content = $1, source_content_hash = $2, updated_at = NOW()
               WHERE id = $3 AND status = 'open'
               RETURNING id, document_id, branch_name, source_content, source_content_hash, source_version, branched_by, status, merged_at, merged_by, merge_conflict, created_at, updated_at"#,
        )
        .bind(&update.content)
        .bind(&update.content_hash)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row)
    }

    #[instrument(skip(self))]
    pub async fn mark_merged(
        &self,
        id: Uuid,
        merged_by: Uuid,
        merge_conflict: Option<String>,
    ) -> DatabaseResult<Option<DocumentBranchRow>> {
        let mut conn = self.pool.acquire().await?;

        let row: Option<DocumentBranchRow> = query_as(
            r#"UPDATE document_branches SET status = 'merged', merged_at = NOW(), merged_by = $2, merge_conflict = $3, updated_at = NOW()
               WHERE id = $1 AND status = 'open'
               RETURNING id, document_id, branch_name, source_content, source_content_hash, source_version, branched_by, status, merged_at, merged_by, merge_conflict, created_at, updated_at"#,
        )
        .bind(id)
        .bind(merged_by)
        .bind(merge_conflict)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Branch {} marked as merged", id);
        Ok(row)
    }

    #[instrument(skip(self))]
    pub async fn mark_abandoned(&self, id: Uuid) -> DatabaseResult<Option<DocumentBranchRow>> {
        let mut conn = self.pool.acquire().await?;

        let row: Option<DocumentBranchRow> = query_as(
            r#"UPDATE document_branches SET status = 'abandoned', updated_at = NOW()
               WHERE id = $1 AND status = 'open'
               RETURNING id, document_id, branch_name, source_content, source_content_hash, source_version, branched_by, status, merged_at, merged_by, merge_conflict, created_at, updated_at"#,
        )
        .bind(id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Branch {} marked as abandoned", id);
        Ok(row)
    }

    #[instrument(skip(self))]
    pub async fn count_open_by_document(&self, document_id: Uuid) -> DatabaseResult<i64> {
        let mut conn = self.pool.acquire().await?;

        let row: (i64,) = query_as(
            "SELECT COUNT(*) FROM document_branches WHERE document_id = $1 AND status = 'open'",
        )
        .bind(document_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.0)
    }

    #[instrument(skip(self))]
    pub async fn find_by_name(
        &self,
        document_id: Uuid,
        branch_name: &str,
    ) -> DatabaseResult<Option<DocumentBranchRow>> {
        let mut conn = self.pool.acquire().await?;

        let row: Option<DocumentBranchRow> = query_as(
            "SELECT * FROM document_branches WHERE document_id = $1 AND branch_name = $2 AND status = 'open'",
        )
        .bind(document_id)
        .bind(branch_name)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_uuid() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_document_branch_row_fields() {
        let id = make_test_uuid();
        let doc_id = make_test_uuid();
        let now = Utc::now();
        let row = DocumentBranchRow {
            id,
            document_id: doc_id,
            branch_name: "feature-x".to_string(),
            source_content: "hello world".to_string(),
            source_content_hash: "abc123".to_string(),
            source_version: 1,
            branched_by: Some(make_test_uuid()),
            status: "open".to_string(),
            merged_at: None,
            merged_by: None,
            merge_conflict: None,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(row.branch_name, "feature-x");
        assert_eq!(row.status, "open");
        assert_eq!(row.source_version, 1);
        assert!(row.merged_at.is_none());
    }

    #[test]
    fn test_document_branch_row_merged() {
        let id = make_test_uuid();
        let doc_id = make_test_uuid();
        let now = Utc::now();
        let merged_by = make_test_uuid();
        let row = DocumentBranchRow {
            id,
            document_id: doc_id,
            branch_name: "feature-y".to_string(),
            source_content: "content".to_string(),
            source_content_hash: "hash".to_string(),
            source_version: 2,
            branched_by: Some(make_test_uuid()),
            status: "merged".to_string(),
            merged_at: Some(now),
            merged_by: Some(merged_by),
            merge_conflict: None,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(row.status, "merged");
        assert!(row.merged_at.is_some());
        assert_eq!(row.merged_by, Some(merged_by));
    }

    #[test]
    fn test_create_branch_row() {
        let doc_id = make_test_uuid();
        let user_id = make_test_uuid();
        let row = CreateBranchRow {
            document_id: doc_id,
            branch_name: "my-branch".to_string(),
            source_content: "content here".to_string(),
            source_content_hash: "sha256hash".to_string(),
            source_version: 3,
            branched_by: user_id,
        };
        assert_eq!(row.branch_name, "my-branch");
        assert_eq!(row.source_version, 3);
    }

    #[test]
    fn test_update_branch_row() {
        let row = UpdateBranchRow {
            content: "updated content".to_string(),
            content_hash: "new_hash".to_string(),
        };
        assert_eq!(row.content, "updated content");
        assert_eq!(row.content_hash, "new_hash");
    }

    #[test]
    fn test_document_branch_row_serialization() {
        let id = make_test_uuid();
        let doc_id = make_test_uuid();
        let now = Utc::now();
        let row = DocumentBranchRow {
            id,
            document_id: doc_id,
            branch_name: "test-branch".to_string(),
            source_content: "content".to_string(),
            source_content_hash: "hash".to_string(),
            source_version: 1,
            branched_by: None,
            status: "open".to_string(),
            merged_at: None,
            merged_by: None,
            merge_conflict: None,
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("test-branch"));
        assert!(json.contains("open"));
    }

    #[test]
    fn test_document_branch_row_deserialization() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "document_id": "00000000-0000-0000-0000-000000000002",
            "branch_name": "feature-z",
            "source_content": "hello",
            "source_content_hash": "h1",
            "source_version": 5,
            "branched_by": null,
            "status": "abandoned",
            "merged_at": null,
            "merged_by": null,
            "merge_conflict": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;
        let row: DocumentBranchRow = serde_json::from_str(json).unwrap();
        assert_eq!(row.branch_name, "feature-z");
        assert_eq!(row.status, "abandoned");
        assert_eq!(row.source_version, 5);
    }
}
