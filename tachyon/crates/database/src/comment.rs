use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

const COMMENT_SELECT_SQL: &str = r#"
    SELECT
        id::text as id,
        document_id::text as document_id,
        author_id::text as author_id,
        author_name,
        content,
        anchor_section,
        anchor_line_start,
        anchor_line_end,
        anchor_selection,
        status,
        parent_id::text as parent_id,
        mentions::text as mentions,
        created_at::text as created_at,
        updated_at::text as updated_at,
        resolved_at::text as resolved_at,
        resolved_by::text as resolved_by
    FROM document_comments
"#;

/// A comment on a document, optionally anchored to a specific section or
/// line range. Supports threading via `parent_id` and `@mention` tracking.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: String,
    pub document_id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub anchor_section: Option<String>,
    pub anchor_line_start: Option<i32>,
    pub anchor_line_end: Option<i32>,
    pub anchor_selection: Option<String>,
    pub status: String,
    pub parent_id: Option<String>,
    pub mentions: String,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub document_id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub anchor_section: Option<String>,
    pub anchor_line_start: Option<i32>,
    pub anchor_line_end: Option<i32>,
    pub anchor_selection: Option<String>,
    pub parent_id: Option<String>,
    pub mentions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub status: Option<String>,
    pub resolved_by: Option<String>,
}

/// Repository for creating, querying, and managing document comments.
///
/// Automatically maintains the `comment_count` denormalized counter on
/// the parent `documents` row when comments are created, resolved, or deleted.
#[derive(Clone)]
pub struct CommentRepository {
    pool: DatabasePool,
}

impl CommentRepository {
    /// Create a new comment repository backed by the given connection pool.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a new comment on a document.
    ///
    /// If `mentions` is not provided, `@mentions` are extracted
    /// automatically from the content. Increments the document's
    /// `comment_count` on success.
    ///
    /// # Arguments
    /// * `req` - Comment creation parameters
    ///
    /// # Returns
    /// The persisted `Comment`.
    ///
    /// # Errors
    /// Returns `DatabaseError::SerializationError` if mentions fail to
    /// serialize, or `DatabaseError::QueryError` on SQL failures.
    #[instrument(skip(self, req))]
    pub async fn create(&self, req: CreateCommentRequest) -> DatabaseResult<Comment> {
        let id = uuid::Uuid::new_v4().to_string();
        let mentions = match req.mentions {
            Some(m) => serde_json::to_string(&m)
                .map_err(|e| DatabaseError::SerializationError(e.to_string()))?,
            None => {
                let parsed: Vec<String> = req
                    .content
                    .split_whitespace()
                    .filter(|w| w.starts_with('@'))
                    .map(|w| w.trim_start_matches('@').to_string())
                    .collect();
                serde_json::to_string(&parsed)
                    .map_err(|e| DatabaseError::SerializationError(e.to_string()))?
            }
        };

        let sql = r#"
            INSERT INTO document_comments (id, document_id, author_id, author_name, content,
                                           anchor_section, anchor_line_start, anchor_line_end,
                                           anchor_selection, status, parent_id, mentions,
                                           created_at, updated_at)
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5,
                    $6, $7, $8, $9, 'open', $10::uuid, $11::jsonb,
                    NOW(), NOW())
            RETURNING
                id::text as id,
                document_id::text as document_id,
                author_id::text as author_id,
                author_name,
                content,
                anchor_section,
                anchor_line_start,
                anchor_line_end,
                anchor_selection,
                status,
                parent_id::text as parent_id,
                mentions::text as mentions,
                created_at::text as created_at,
                updated_at::text as updated_at,
                resolved_at::text as resolved_at,
                resolved_by::text as resolved_by
        "#;

        let mut conn = self.pool.acquire().await?;
        let comment = sqlx::query_as::<_, Comment>(sql)
            .bind(&id)
            .bind(&req.document_id)
            .bind(&req.author_id)
            .bind(&req.author_name)
            .bind(&req.content)
            .bind(&req.anchor_section)
            .bind(req.anchor_line_start)
            .bind(req.anchor_line_end)
            .bind(&req.anchor_selection)
            .bind(&req.parent_id)
            .bind(&mentions)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        sqlx::query("UPDATE documents SET comment_count = comment_count + 1 WHERE id = $1::uuid")
            .bind(&req.document_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(comment)
    }

    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Comment> {
        let sql = format!("{} WHERE id = $1::uuid", COMMENT_SELECT_SQL);
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as::<_, Comment>(&sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("comment", id))
    }

    /// List comments for a document with optional filtering.
    ///
    /// # Arguments
    /// * `document_id` - UUID of the document
    /// * `include_resolved` - When `false`, only "open" comments are returned
    /// * `parent_id` - Optional parent comment UUID to filter a thread
    /// * `limit` - Maximum number of results
    /// * `offset` - Number of results to skip
    ///
    /// # Returns
    /// A vector of comments ordered by creation date (newest first).
    #[instrument(skip(self))]
    pub async fn list_by_document(
        &self,
        document_id: &str,
        include_resolved: bool,
        parent_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> DatabaseResult<Vec<Comment>> {
        let mut conditions = vec!["document_id = $1::uuid".to_string()];
        let mut bind_idx = 1u32;

        if !include_resolved {
            bind_idx += 1;
            conditions.push(format!("status = ${}", bind_idx));
        }

        if parent_id.is_some() {
            bind_idx += 1;
            conditions.push(format!("parent_id = ${}::uuid", bind_idx));
        }

        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        let sql = format!(
            "{}{} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            COMMENT_SELECT_SQL, where_clause, limit, offset
        );

        let mut conn = self.pool.acquire().await?;
        let mut query = sqlx::query_as::<_, Comment>(&sql).bind(document_id);

        if !include_resolved {
            query = query.bind("open");
        }
        if let Some(pid) = parent_id {
            query = query.bind(pid);
        }

        let comments = query.fetch_all(&mut *conn).await?;
        Ok(comments)
    }

    /// Update a comment's content, status, or resolution info.
    ///
    /// When transitioning to "resolved", sets `resolved_at` and decrements
    /// the document's `comment_count`.
    ///
    /// # Arguments
    /// * `id` - UUID of the comment
    /// * `req` - Fields to update
    ///
    /// # Returns
    /// The updated `Comment`.
    ///
    /// # Errors
    /// Returns `DatabaseError::NotFound` if the comment does not exist.
    #[instrument(skip(self, req))]
    pub async fn update(&self, id: &str, req: UpdateCommentRequest) -> DatabaseResult<Comment> {
        let existing = self.get_by_id(id).await?;

        let content = req.content.unwrap_or(existing.content);
        let status = req.status.unwrap_or_else(|| existing.status.clone());
        let now = Utc::now();

        let (resolved_at, resolved_by) = if status == "resolved" && existing.status != "resolved" {
            (Some(now), req.resolved_by.clone())
        } else if status != "resolved" {
            (None, None)
        } else {
            (
                existing
                    .resolved_at
                    .as_ref()
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                existing.resolved_by.clone(),
            )
        };

        let sql = r#"
            UPDATE document_comments SET
                content = $2,
                status = $3,
                updated_at = $4,
                resolved_at = $5,
                resolved_by = $6::uuid
            WHERE id = $1::uuid
            RETURNING
                id::text as id,
                document_id::text as document_id,
                author_id::text as author_id,
                author_name,
                content,
                anchor_section,
                anchor_line_start,
                anchor_line_end,
                anchor_selection,
                status,
                parent_id::text as parent_id,
                mentions::text as mentions,
                created_at::text as created_at,
                updated_at::text as updated_at,
                resolved_at::text as resolved_at,
                resolved_by::text as resolved_by
        "#;

        let mut conn = self.pool.acquire().await?;
        let comment = sqlx::query_as::<_, Comment>(sql)
            .bind(id)
            .bind(&content)
            .bind(&status)
            .bind(now)
            .bind(resolved_at)
            .bind(&resolved_by)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        if status == "resolved" && existing.status != "resolved" {
            sqlx::query("UPDATE documents SET comment_count = GREATEST(comment_count - 1, 0) WHERE id = $1::uuid")
                .bind(&existing.document_id)
                .execute(&mut *conn)
                .await
                .map_err(|e| DatabaseError::query_error(e.to_string()))?;
        }

        Ok(comment)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let existing = self.get_by_id(id).await?;

        let sql = "DELETE FROM document_comments WHERE id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        let result = sqlx::query(sql).bind(id).execute(&mut *conn).await?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("comment", id));
        }

        if existing.status != "resolved" {
            sqlx::query("UPDATE documents SET comment_count = GREATEST(comment_count - 1, 0) WHERE id = $1::uuid")
                .bind(&existing.document_id)
                .execute(&mut *conn)
                .await
                .map_err(|e| DatabaseError::query_error(e.to_string()))?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn count_by_document(&self, document_id: &str) -> DatabaseResult<i64> {
        let sql = "SELECT COUNT(*) as count FROM document_comments WHERE document_id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        let count = sqlx::query_scalar::<_, i64>(sql)
            .bind(document_id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn extract_mentions(content: &str) -> Vec<String> {
        content
            .split_whitespace()
            .filter(|w| w.starts_with('@'))
            .map(|w| w.trim_start_matches('@').to_string())
            .collect()
    }

    #[test]
    fn test_extract_mentions_from_content() {
        let content = "Hello @alice and @bob check this out";
        let mentions = extract_mentions(content);
        assert_eq!(mentions, vec!["alice", "bob"]);
    }

    #[test]
    fn test_extract_mentions_no_mentions() {
        let content = "Hello world, no mentions here";
        let mentions = extract_mentions(content);
        assert!(mentions.is_empty());
    }

    #[test]
    fn test_extract_mentions_single() {
        let content = "Hey @carol look at this";
        let mentions = extract_mentions(content);
        assert_eq!(mentions, vec!["carol"]);
    }

    #[test]
    fn test_comment_struct_open_status() {
        let comment = Comment {
            id: "1".into(),
            document_id: "doc-1".into(),
            author_id: "user-1".into(),
            author_name: "Alice".into(),
            content: "Hello".into(),
            anchor_section: Some("intro".into()),
            anchor_line_start: Some(1),
            anchor_line_end: Some(5),
            anchor_selection: None,
            status: "open".into(),
            parent_id: None,
            mentions: r#"["bob"]"#.into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
            resolved_at: None,
            resolved_by: None,
        };
        assert_eq!(comment.status, "open");
        assert_eq!(comment.anchor_line_start, Some(1));
        assert!(comment.parent_id.is_none());
        assert!(comment.resolved_at.is_none());
    }

    #[test]
    fn test_comment_struct_resolved_state() {
        let comment = Comment {
            id: "1".into(),
            document_id: "doc-1".into(),
            author_id: "user-1".into(),
            author_name: "Alice".into(),
            content: "Fixed".into(),
            anchor_section: None,
            anchor_line_start: None,
            anchor_line_end: None,
            anchor_selection: None,
            status: "resolved".into(),
            parent_id: None,
            mentions: "[]".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:01:00Z".into(),
            resolved_at: Some("2024-01-01T00:01:00Z".into()),
            resolved_by: Some("user-2".into()),
        };
        assert_eq!(comment.status, "resolved");
        assert!(comment.resolved_at.is_some());
        assert_eq!(comment.resolved_by.as_deref(), Some("user-2"));
    }

    #[test]
    fn test_create_comment_request_fields() {
        let req = CreateCommentRequest {
            document_id: "doc-1".into(),
            author_id: "user-1".into(),
            author_name: "Alice".into(),
            content: "Great work @bob".into(),
            anchor_section: Some("body".into()),
            anchor_line_start: Some(10),
            anchor_line_end: Some(20),
            anchor_selection: None,
            parent_id: Some("parent-1".into()),
            mentions: Some(vec!["bob".into()]),
        };
        assert_eq!(req.document_id, "doc-1");
        assert_eq!(
            req.mentions.as_deref(),
            Some(["bob".to_string()].as_slice())
        );
        assert_eq!(req.parent_id.as_deref(), Some("parent-1"));
    }

    #[test]
    fn test_update_comment_request_all_none() {
        let req = UpdateCommentRequest {
            content: None,
            status: None,
            resolved_by: None,
        };
        assert!(req.content.is_none());
        assert!(req.status.is_none());
    }
}
