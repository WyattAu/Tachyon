// Document Version Repository
// Version history management for documents

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, Row};
use tracing::{debug, info, instrument};

const VERSION_SELECT_SQL: &str = r#"
    SELECT 
        id::text as id,
        document_id::text as document_id,
        version_number,
        content,
        commit_message,
        created_at,
        created_by::text as created_by
    FROM document_versions
"#;

/// A snapshot of a document at a specific point in time.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentVersion {
    /// Primary key (UUID).
    pub id: String,
    /// Parent document ID.
    pub document_id: String,
    /// Monotonically increasing version number.
    pub version_number: i32,
    /// Full document content at this version.
    pub content: String,
    /// Optional commit message describing the change.
    pub commit_message: Option<String>,
    /// When this version was created.
    pub created_at: DateTime<Utc>,
    /// User ID of the author of this version.
    pub created_by: String,
}

/// Payload for creating a new document version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionRequest {
    /// Parent document ID.
    pub document_id: String,
    /// Full document content to snapshot.
    pub content: String,
    /// Optional commit message.
    pub commit_message: Option<String>,
    /// User ID of the author.
    pub created_by: String,
}

/// Repository for document version history.
#[derive(Clone)]
pub struct DocumentVersionRepository {
    pool: DatabasePool,
}

impl DocumentVersionRepository {
    /// Create a new document version repository backed by `pool`.
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a new version, auto-incrementing the version number.
    #[instrument(skip(self))]
    pub async fn create(&self, req: CreateVersionRequest) -> DatabaseResult<DocumentVersion> {
        let mut conn = self.pool.acquire().await?;

        let current_version: i32 = query(
            "SELECT COALESCE(MAX(version_number), 0) as version FROM document_versions WHERE document_id = $1::uuid"
        )
        .bind(&req.document_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        .get("version");

        let new_version_number = current_version + 1;
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let insert_sql = r#"
            INSERT INTO document_versions (
                id, document_id, version_number, content, commit_message, created_at, created_by
            ) VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7::uuid)
            RETURNING id::text as id, document_id::text as document_id, version_number, content, commit_message, created_at, created_by::text as created_by
        "#;

        let version: DocumentVersion = query_as(insert_sql)
            .bind(&id)
            .bind(&req.document_id)
            .bind(new_version_number)
            .bind(&req.content)
            .bind(&req.commit_message)
            .bind(now)
            .bind(&req.created_by)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Document version created: {} v{}",
            req.document_id, new_version_number
        );
        Ok(version)
    }

    /// Retrieve a version by its UUID.
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<DocumentVersion> {
        let select_sql = format!("{} WHERE id = $1::uuid", VERSION_SELECT_SQL);

        let mut conn = self.pool.acquire().await?;
        let version: Option<DocumentVersion> = query_as(&select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        version.ok_or_else(|| DatabaseError::not_found("document_version", id))
    }

    /// Retrieve a specific version of a document by version number.
    #[instrument(skip(self))]
    pub async fn get_by_version_number(
        &self,
        document_id: &str,
        version_number: i32,
    ) -> DatabaseResult<DocumentVersion> {
        let select_sql = format!(
            "{} WHERE document_id = $1::uuid AND version_number = $2",
            VERSION_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let version: Option<DocumentVersion> = query_as(&select_sql)
            .bind(document_id)
            .bind(version_number)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        version.ok_or_else(|| {
            DatabaseError::not_found(
                "document_version",
                format!("{}:v{}", document_id, version_number),
            )
        })
    }

    /// List versions for a document, newest first.
    #[instrument(skip(self))]
    pub async fn list_by_document(
        &self,
        document_id: &str,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<DocumentVersion>> {
        let limit = limit.unwrap_or(50);
        let select_sql = format!(
            "{} WHERE document_id = $1::uuid ORDER BY version_number DESC LIMIT $2",
            VERSION_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let versions: Vec<DocumentVersion> = query_as(&select_sql)
            .bind(document_id)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!(
            "Found {} versions for document {}",
            versions.len(),
            document_id
        );
        Ok(versions)
    }

    /// Get the most recent version of a document.
    #[instrument(skip(self))]
    pub async fn get_latest(&self, document_id: &str) -> DatabaseResult<DocumentVersion> {
        let select_sql = format!(
            "{} WHERE document_id = $1::uuid ORDER BY version_number DESC LIMIT 1",
            VERSION_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let version: Option<DocumentVersion> = query_as(&select_sql)
            .bind(document_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        version.ok_or_else(|| DatabaseError::not_found("document_version", document_id))
    }

    /// Permanently delete a version by UUID.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM document_versions WHERE id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("document_version", id));
        }

        info!("Document version deleted: {}", id);
        Ok(())
    }

    /// Count versions for a document.
    #[instrument(skip(self))]
    pub async fn count_by_document(&self, document_id: &str) -> DatabaseResult<i64> {
        let count_sql =
            "SELECT COUNT(*) as count FROM document_versions WHERE document_id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(document_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("count"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_document_version_struct_fields() {
        let version = DocumentVersion {
            id: "1".into(),
            document_id: "doc-1".into(),
            version_number: 1,
            content: "# Hello".into(),
            commit_message: Some("Initial commit".into()),
            created_at: chrono::Utc::now(),
            created_by: "user-1".into(),
        };
        assert_eq!(version.version_number, 1);
        assert_eq!(version.document_id, "doc-1");
        assert_eq!(version.commit_message.as_deref(), Some("Initial commit"));
    }

    #[test]
    fn test_document_version_no_commit_message() {
        let version = DocumentVersion {
            id: "1".into(),
            document_id: "doc-1".into(),
            version_number: 2,
            content: "# Updated".into(),
            commit_message: None,
            created_at: chrono::Utc::now(),
            created_by: "user-1".into(),
        };
        assert!(version.commit_message.is_none());
        assert_eq!(version.version_number, 2);
    }

    #[test]
    fn test_create_version_request_fields() {
        let req = CreateVersionRequest {
            document_id: "doc-1".into(),
            content: "content".into(),
            commit_message: Some("save".into()),
            created_by: "user-1".into(),
        };
        assert_eq!(req.document_id, "doc-1");
        assert_eq!(req.commit_message.as_deref(), Some("save"));
    }

    #[test]
    fn test_create_version_request_no_message() {
        let req = CreateVersionRequest {
            document_id: "doc-1".into(),
            content: "content".into(),
            commit_message: None,
            created_by: "user-1".into(),
        };
        assert!(req.commit_message.is_none());
    }
}
