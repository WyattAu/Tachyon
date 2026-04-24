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

#[derive(Clone)]
pub struct CommentRepository {
    pool: DatabasePool,
}

impl CommentRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

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
            .bind(&req.anchor_line_start)
            .bind(&req.anchor_line_end)
            .bind(&req.anchor_selection)
            .bind(&req.parent_id)
            .bind(&mentions)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))?;

        sqlx::query("UPDATE documents SET comment_count = comment_count + 1 WHERE id = $1::uuid")
            .bind(&req.document_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))?;

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
                existing.resolved_at.as_ref().and_then(|s| s.parse::<DateTime<Utc>>().ok()),
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
            .map_err(|e| DatabaseError::query_error(&e.to_string()))?;

        if status == "resolved" && existing.status != "resolved" {
            sqlx::query("UPDATE documents SET comment_count = GREATEST(comment_count - 1, 0) WHERE id = $1::uuid")
                .bind(&existing.document_id)
                .execute(&mut *conn)
                .await
                .map_err(|e| DatabaseError::query_error(&e.to_string()))?;
        }

        Ok(comment)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let existing = self.get_by_id(id).await?;

        let sql = "DELETE FROM document_comments WHERE id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        let result = sqlx::query(sql)
            .bind(id)
            .execute(&mut *conn)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("comment", id));
        }

        if existing.status != "resolved" {
            sqlx::query("UPDATE documents SET comment_count = GREATEST(comment_count - 1, 0) WHERE id = $1::uuid")
                .bind(&existing.document_id)
                .execute(&mut *conn)
                .await
                .map_err(|e| DatabaseError::query_error(&e.to_string()))?;
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
