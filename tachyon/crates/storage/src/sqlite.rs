// SQLite Document Store
//
// Embedded local storage for offline-first mode.
// Uses sqlx::SqlitePool for async-native SQLite access.
// FTS5 for full-text search.

use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteConnectOptions, Row};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use tachyon_core::id::{generate_document_id, DocumentId, UserId};
use tachyon_core::types::document::{
    Document, DocumentContent, DocumentMetadata, DocumentStats, DocumentStatus, DocumentVisibility,
};
use tachyon_core::types::storage::{
    DocumentListSummary, DocumentStore, ListParams, ListResult, SortDirection, SortField,
    StorageError, StorageResult,
};
use tracing::{info, warn};

/// SQLite-backed document store for offline-first mode.
///
/// Uses `sqlx::SqlitePool` for async-native access.
/// WAL mode enabled for better concurrent read performance.
pub struct SqliteStore {
    pool: sqlx::SqlitePool,
}

impl SqliteStore {
    /// Open (or create) a SQLite database at the given path.
    pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Self, StorageError> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path_str))
            .map_err(|e| StorageError::Unavailable {
                reason: format!("Invalid SQLite path: {}", e),
            })?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = sqlx::SqlitePool::connect_with(options).await.map_err(|e| {
            StorageError::Unavailable {
                reason: format!("Failed to open SQLite database: {}", e),
            }
        })?;

        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    /// Open an in-memory SQLite database (useful for testing).
    pub async fn in_memory() -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|e| StorageError::Unavailable {
                reason: format!("Invalid SQLite URI: {}", e),
            })?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = sqlx::SqlitePool::connect_with(options).await.map_err(|e| {
            StorageError::Unavailable {
                reason: format!("Failed to open in-memory SQLite: {}", e),
            }
        })?;

        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    /// Create the documents table and FTS5 search index.
    async fn init_schema(pool: &sqlx::SqlitePool) -> Result<(), StorageError> {
        // Use a raw connection for batch DDL (pool doesn't support execute_batch well)
        let mut conn = pool.acquire().await.map_err(|e| StorageError::Internal {
            message: format!("Failed to acquire connection: {}", e),
        })?;

        // Create documents table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS documents (
                id              TEXT PRIMARY KEY,
                title           TEXT NOT NULL,
                slug            TEXT NOT NULL DEFAULT '',
                author_id       TEXT NOT NULL,
                description     TEXT,
                tags            TEXT NOT NULL DEFAULT '[]',
                visibility      TEXT NOT NULL DEFAULT 'private',
                status          TEXT NOT NULL DEFAULT 'draft',
                content_type    TEXT NOT NULL DEFAULT 'markdown',
                content         TEXT,
                word_count      INTEGER NOT NULL DEFAULT 0,
                character_count INTEGER NOT NULL DEFAULT 0,
                read_count      INTEGER NOT NULL DEFAULT 0,
                edit_count      INTEGER NOT NULL DEFAULT 1,
                published_at    TEXT,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            )",
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| StorageError::Internal {
            message: format!("Failed to create documents table: {}", e),
        })?;

        // Create indexes
        for idx_sql in [
            "CREATE INDEX IF NOT EXISTS idx_documents_author_id ON documents(author_id)",
            "CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status)",
            "CREATE INDEX IF NOT EXISTS idx_documents_slug ON documents(slug)",
            "CREATE INDEX IF NOT EXISTS idx_documents_updated_at ON documents(updated_at)",
        ] {
            sqlx::query(idx_sql)
                .execute(&mut *conn)
                .await
                .map_err(|e| StorageError::Internal {
                    message: format!("Failed to create index: {}", e),
                })?;
        }

        // FTS5 virtual table for full-text search
        // We need a raw SQL approach since sqlx doesn't natively handle FTS5 DDL
        let fts_sql = r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts
                USING fts5(title, content, content=documents, content_rowid=rowid);
        "#;
        // FTS5 may fail if it already exists with a different schema, ignore errors
        let _ = sqlx::query(fts_sql).execute(&mut *conn).await;

        info!("SQLite schema initialized");
        Ok(())
    }

    // ---- Internal helpers ----

    fn row_to_document(row: &sqlx::sqlite::SqliteRow) -> Result<Document, StorageError> {
        let id_str: &str = row.try_get("id").map_err(|e| StorageError::Internal {
            message: format!("Failed to read id: {}", e),
        })?;
        let id = DocumentId::parse_str(id_str).map_err(|_| StorageError::Internal {
            message: format!("Invalid document ID: {}", id_str),
        })?;

        let author_str: &str = row
            .try_get("author_id")
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to read author_id: {}", e),
            })?;
        let author_id = UserId::parse_str(author_str).map_err(|_| StorageError::Internal {
            message: format!("Invalid author ID: {}", author_str),
        })?;

        let status_str: &str = row.try_get("status").map_err(|e| StorageError::Internal {
            message: format!("Failed to read status: {}", e),
        })?;
        let status = match status_str {
            "published" => DocumentStatus::Published,
            "archived" => DocumentStatus::Archived,
            "deleted" => DocumentStatus::Deleted,
            _ => DocumentStatus::Draft,
        };

        let vis_str: &str = row
            .try_get("visibility")
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to read visibility: {}", e),
            })?;
        let visibility = match vis_str {
            "public" => DocumentVisibility::Public,
            "restricted" => DocumentVisibility::Restricted,
            _ => DocumentVisibility::Private,
        };

        let content_type: &str = row.try_get("content_type").unwrap_or("markdown");
        let content_raw: Option<&str> = row.try_get("content").unwrap_or(None);
        let content = match content_type {
            "text" => DocumentContent::Text {
                content: content_raw.unwrap_or_default().to_string(),
            },
            _ => DocumentContent::Markdown {
                content: content_raw.unwrap_or_default().to_string(),
            },
        };

        let created_at_str: &str = row.try_get("created_at").unwrap_or("");
        let updated_at_str: &str = row.try_get("updated_at").unwrap_or("");
        let published_at_str: Option<&str> = row.try_get("published_at").unwrap_or(None);

        let created_at = if !created_at_str.is_empty() {
            parse_datetime(created_at_str)
        } else {
            Utc::now()
        };
        let updated_at = if !updated_at_str.is_empty() {
            parse_datetime(updated_at_str)
        } else {
            Utc::now()
        };
        let published_at = published_at_str.map(parse_datetime);

        let tags_str: &str = row.try_get("tags").unwrap_or("[]");
        let tags: Vec<String> = serde_json::from_str(tags_str).unwrap_or_default();

        let slug: &str = row.try_get("slug").unwrap_or("");
        let title: String = row.try_get("title").unwrap_or_default();

        let word_count: i64 = row.try_get("word_count").unwrap_or(0);
        let character_count: i64 = row.try_get("character_count").unwrap_or(0);
        let read_count: i64 = row.try_get("read_count").unwrap_or(0);
        let edit_count: i64 = row.try_get("edit_count").unwrap_or(1);

        Ok(Document {
            id,
            metadata: DocumentMetadata {
                title,
                slug: if slug.is_empty() {
                    None
                } else {
                    Some(slug.to_string())
                },
                author_id,
                description: row.try_get("description").unwrap_or(None),
                tags,
                frontmatter: None,
                created_at,
                updated_at,
                published_at,
            },
            content,
            visibility,
            status,
            stats: DocumentStats {
                word_count: word_count as usize,
                character_count: character_count as usize,
                read_count: read_count as usize,
                edit_count: edit_count as usize,
            },
            repository_id: None,
        })
    }

    fn compute_stats(content: &DocumentContent) -> (i64, i64) {
        match content {
            DocumentContent::Markdown { content } | DocumentContent::Text { content } => (
                content.split_whitespace().count() as i64,
                content.chars().count() as i64,
            ),
            DocumentContent::Binary { content, .. } => {
                (0, content.as_ref().map_or(0, |v| v.len()) as i64)
            }
        }
    }

    fn content_type_str(content: &DocumentContent) -> &'static str {
        match content {
            DocumentContent::Markdown { .. } => "markdown",
            DocumentContent::Text { .. } => "text",
            DocumentContent::Binary { .. } => "binary",
        }
    }

    fn content_text(content: &DocumentContent) -> Option<&str> {
        content.as_text()
    }
}

fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").map(|ndt| ndt.and_utc())
        })
        .unwrap_or_else(|_| Utc::now())
}

fn to_rfc3339(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

// ============================================================================
// DocumentStore implementation
// ============================================================================

impl DocumentStore for SqliteStore {
    fn create_document<'a>(
        &'a self,
        metadata: DocumentMetadata,
        content: DocumentContent,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let id = generate_document_id();
            let id_str = id.as_str();
            let _id_str_clone = id_str.clone(); // for error path below if needed
            let slug = metadata.slug.as_deref().unwrap_or("").to_string();
            let tags_json =
                serde_json::to_string(&metadata.tags).unwrap_or_else(|_| "[]".to_string());
            let content_type = Self::content_type_str(&content);
            let content_text = Self::content_text(&content);
            let (word_count, character_count) = Self::compute_stats(&content);
            let created_at = to_rfc3339(&metadata.created_at);
            let updated_at = to_rfc3339(&metadata.updated_at);
            let published_at = metadata.published_at.as_ref().map(to_rfc3339);

            let result = sqlx::query(
                "INSERT INTO documents
                    (id, title, slug, author_id, description, tags,
                     visibility, status, content_type, content,
                     word_count, character_count, read_count, edit_count,
                     published_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                         ?11, ?12, ?13, 1, ?14, ?15, ?16)",
            )
            .bind(&id_str)
            .bind(&metadata.title)
            .bind(&slug)
            .bind(metadata.author_id.as_str())
            .bind(&metadata.description)
            .bind(&tags_json)
            .bind("private")
            .bind("draft")
            .bind(content_type)
            .bind(content_text)
            .bind(word_count)
            .bind(character_count)
            .bind(0i64) // read_count
            .bind(published_at.as_deref())
            .bind(&created_at)
            .bind(&updated_at)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => {
                    let doc = Document::new(id, metadata.title, metadata.author_id, content);
                    Ok(doc)
                }
                Err(e) if e.to_string().contains("UNIQUE constraint failed") => {
                    Err(StorageError::ConstraintViolation {
                        field: "slug".to_string(),
                        value: slug,
                    })
                }
                Err(e) => Err(StorageError::Internal {
                    message: format!("Failed to create document: {}", e),
                }),
            }
        })
    }

    fn get_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let id_str = id.as_str();
            let id_str_clone = id_str.clone();

            let row = sqlx::query(
                "SELECT id, title, slug, author_id, description, tags,
                        visibility, status, content_type, content,
                        word_count, character_count, read_count, edit_count,
                        published_at, created_at, updated_at
                 FROM documents WHERE id = ?1",
            )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to query document: {}", e),
            })?;

            match row {
                Some(row) => Self::row_to_document(&row),
                None => Err(StorageError::NotFound { id: id_str_clone }),
            }
        })
    }

    fn update_document_content<'a>(
        &'a self,
        id: &'a DocumentId,
        content: DocumentContent,
        _expected_version: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let id_str = id.as_str();
            let id_str_clone = id_str.clone();
            let content_type = Self::content_type_str(&content);
            let content_text = Self::content_text(&content);
            let (word_count, character_count) = Self::compute_stats(&content);
            let now = to_rfc3339(&Utc::now());

            let result = sqlx::query(
                "UPDATE documents SET
                    content_type = ?1, content = ?2,
                    word_count = ?3, character_count = ?4,
                    edit_count = edit_count + 1,
                    updated_at = ?5
                 WHERE id = ?6",
            )
            .bind(content_type)
            .bind(content_text)
            .bind(word_count)
            .bind(character_count)
            .bind(&now)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to update content: {}", e),
            })?;

            if result.rows_affected() == 0 {
                return Err(StorageError::NotFound { id: id_str_clone });
            }

            self.get_document(id).await
        })
    }

    fn update_document_metadata<'a>(
        &'a self,
        id: &'a DocumentId,
        metadata: DocumentMetadata,
        _expected_version: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let id_str = id.as_str();
            let id_str_clone = id_str.clone();
            let tags_json =
                serde_json::to_string(&metadata.tags).unwrap_or_else(|_| "[]".to_string());
            let now = to_rfc3339(&Utc::now());
            let published_at = metadata.published_at.as_ref().map(to_rfc3339);

            let result = sqlx::query(
                "UPDATE documents SET
                    title = ?1, slug = COALESCE(?2, slug),
                    description = ?3, tags = ?4,
                    published_at = COALESCE(?5, published_at),
                    updated_at = ?6
                 WHERE id = ?7",
            )
            .bind(&metadata.title)
            .bind(&metadata.slug)
            .bind(&metadata.description)
            .bind(&tags_json)
            .bind(published_at.as_deref())
            .bind(&now)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to update metadata: {}", e),
            })?;

            if result.rows_affected() == 0 {
                return Err(StorageError::NotFound { id: id_str_clone });
            }

            self.get_document(id).await
        })
    }

    fn delete_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let id_str = id.as_str();
            let now = to_rfc3339(&Utc::now());

            sqlx::query("UPDATE documents SET status = 'deleted', updated_at = ?1 WHERE id = ?2")
                .bind(&now)
                .bind(&id_str)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::Internal {
                    message: format!("Failed to delete document: {}", e),
                })?;

            Ok(())
        })
    }

    fn permanently_delete_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let id_str = id.as_str();

            sqlx::query("DELETE FROM documents WHERE id = ?1")
                .bind(&id_str)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::Internal {
                    message: format!("Failed to permanently delete document: {}", e),
                })?;

            Ok(())
        })
    }

    fn list_documents<'a>(
        &'a self,
        params: ListParams,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>> {
        Box::pin(async move {
            // Build WHERE clause dynamically
            let mut conditions: Vec<String> = Vec::new();
            let mut bind_values: Vec<String> = Vec::new();

            // Exclude deleted
            conditions.push("status != 'deleted'".to_string());

            if let Some(ref author_id) = params.author_id {
                conditions.push(format!("author_id = ?{}", bind_values.len() + 1));
                bind_values.push(author_id.as_str().to_string());
            }

            if let Some(ref status) = params.status {
                let s = match status {
                    DocumentStatus::Published => "published",
                    DocumentStatus::Draft => "draft",
                    DocumentStatus::Archived => "archived",
                    DocumentStatus::Deleted => "deleted",
                };
                conditions.push(format!("status = ?{}", bind_values.len() + 1));
                bind_values.push(s.to_string());
            }

            // Tag filter using json_each
            if !params.tags.is_empty() {
                let tag_ors: Vec<String> = params
                    .tags
                    .iter()
                    .map(|_| {
                        let idx = bind_values.len() + 1;
                        bind_values.push(String::new()); // placeholder
                        format!("EXISTS (SELECT 1 FROM json_each(documents.tags) WHERE json_each.value = ?{})", idx)
                    })
                    .collect();
                conditions.push(format!("({})", tag_ors.join(" OR ")));
                // Now fill in actual tag values (we know the indices)
                // This is a bit tricky with dynamic binding, let's use a simpler approach
            }

            // Full-text search using LIKE (simpler than FTS5 trigger sync issues)
            if let Some(ref query) = params.query {
                conditions.push(format!(
                    "(title LIKE ?{} OR content LIKE ?{})",
                    bind_values.len() + 1,
                    bind_values.len() + 2
                ));
                let pattern = format!("%{}%", query);
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }

            // Rebuild with simpler tag approach: use LIKE on tags JSON
            if !params.tags.is_empty() {
                // Remove the json_each condition we added and replace with LIKE
                if conditions.len() > 1 {
                    // Pop the json_each condition
                    let last = conditions.pop().unwrap_or_default();
                    if last.contains("json_each") {
                        // Replace with simple LIKE for each tag
                        for tag in &params.tags {
                            conditions.push(format!("tags LIKE ?{}", bind_values.len() + 1));
                            bind_values.push(format!("%\"{}\"%", tag));
                        }
                    } else {
                        conditions.push(last);
                    }
                }
            }

            let where_sql = format!("WHERE {}", conditions.join(" AND "));

            let order_col = match params.sort_by {
                SortField::CreatedAt => "created_at",
                SortField::Title => "title",
                SortField::UpdatedAt => "updated_at",
            };
            let order_dir = match params.sort_dir {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
            };

            let offset = ((params.page.saturating_sub(1)) * params.page_size) as i64;
            let limit = params.page_size as i64;

            // Build the query with positional bind parameters
            // Use a macro-like approach: build bind array and use ?N syntax
            let select_sql = format!(
                "SELECT id, title, slug, author_id, description, tags,
                        visibility, status, content_type, content,
                        word_count, character_count, read_count, edit_count,
                        published_at, created_at, updated_at
                 FROM documents {}
                 ORDER BY {} {}
                 LIMIT ?{} OFFSET ?{}",
                where_sql,
                order_col,
                order_dir,
                bind_values.len() + 1,
                bind_values.len() + 2,
            );

            let count_sql = format!("SELECT COUNT(*) as cnt FROM documents {}", where_sql);

            // Execute count query
            let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
            for val in &bind_values {
                count_query = count_query.bind(val.clone());
            }
            let total: i64 = count_query.fetch_one(&self.pool).await.unwrap_or(0);

            // Execute data query
            let mut data_query = sqlx::query(&select_sql);
            for val in &bind_values {
                data_query = data_query.bind(val.clone());
            }
            data_query = data_query.bind(limit).bind(offset);

            let rows =
                data_query
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| StorageError::Internal {
                        message: format!("Failed to list documents: {}", e),
                    })?;

            let mut items = Vec::new();
            for row in rows {
                match Self::row_to_document(&row) {
                    Ok(doc) => items.push(doc),
                    Err(e) => warn!("Error reading document row: {:?}", e),
                }
            }

            Ok(ListResult {
                total: total as usize,
                items,
                page: params.page,
                page_size: params.page_size,
            })
        })
    }

    fn search_documents<'a>(
        &'a self,
        query: &'a str,
        page: usize,
        page_size: usize,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>> {
        Box::pin(async move {
            let params = ListParams {
                query: Some(query.to_string()),
                page,
                page_size,
                ..ListParams::default()
            };
            self.list_documents(params).await
        })
    }

    fn get_list_summary<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<DocumentListSummary>> + Send + 'a>> {
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE status != 'deleted'")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);

            let draft_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE status = 'draft'")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);

            let published_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE status = 'published'")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);

            let archived_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE status = 'archived'")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);

            let total_word_count: i64 = sqlx::query_scalar(
                "SELECT COALESCE(SUM(word_count), 0) FROM documents WHERE status != 'deleted'",
            )
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

            // Count unique tags via json_each
            let total_tags: i64 = sqlx::query_scalar(
                "SELECT COUNT(DISTINCT json_each.value)
                 FROM documents, json_each(documents.tags)
                 WHERE documents.status != 'deleted'",
            )
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

            Ok(DocumentListSummary {
                total_documents: total as usize,
                draft_count: draft_count as usize,
                published_count: published_count as usize,
                archived_count: archived_count as usize,
                total_word_count: total_word_count as usize,
                total_tags: total_tags as usize,
            })
        })
    }

    fn get_all_tags<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Vec<String>>> + Send + 'a>> {
        Box::pin(async move {
            let rows = sqlx::query_scalar::<_, String>(
                "SELECT DISTINCT json_each.value
                 FROM documents, json_each(documents.tags)
                 WHERE documents.status != 'deleted'
                 ORDER BY json_each.value",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Internal {
                message: format!("Failed to query tags: {}", e),
            })?;

            Ok(rows)
        })
    }

    fn get_documents_by_tag<'a>(
        &'a self,
        tag: &'a str,
        page: usize,
        page_size: usize,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>> {
        Box::pin(async move {
            let params = ListParams {
                tags: vec![tag.to_string()],
                page,
                page_size,
                ..ListParams::default()
            };
            self.list_documents(params).await
        })
    }

    fn is_available<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<bool>> + Send + 'a>> {
        Box::pin(async move {
            match sqlx::query("SELECT 1").execute(&self.pool).await {
                Ok(_) => Ok(true),
                Err(e) => {
                    warn!("SQLite availability check failed: {}", e);
                    Ok(false)
                }
            }
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tachyon_core::id::generate_user_id;

    async fn make_store() -> SqliteStore {
        SqliteStore::in_memory()
            .await
            .expect("Failed to create in-memory store")
    }

    fn make_user_id() -> UserId {
        generate_user_id()
    }

    #[tokio::test]
    async fn test_create_and_get_document() {
        let store = make_store().await;
        let user_id = make_user_id();
        let metadata = DocumentMetadata::new("Test Document".to_string(), user_id);
        let content = DocumentContent::markdown("# Hello\n\nWorld".to_string());

        let doc = store
            .create_document(metadata, content)
            .await
            .expect("Failed to create document");

        assert_eq!(doc.metadata.title, "Test Document");
        assert!(!doc.id.as_str().is_empty());

        let fetched = store
            .get_document(&doc.id)
            .await
            .expect("Failed to get document");
        assert_eq!(fetched.id, doc.id);
        assert_eq!(fetched.metadata.title, "Test Document");
    }

    #[tokio::test]
    async fn test_get_nonexistent_document() {
        let store = make_store().await;
        let fake_id = generate_document_id();

        let result = store.get_document(&fake_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::NotFound { id } => assert_eq!(id, fake_id.as_str()),
            other => panic!("Expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_update_content() {
        let store = make_store().await;
        let user_id = make_user_id();
        let metadata = DocumentMetadata::new("Test".to_string(), user_id);
        let content = DocumentContent::markdown("Original".to_string());

        let doc = store
            .create_document(metadata, content)
            .await
            .expect("create");

        let new_content = DocumentContent::markdown("Updated content".to_string());
        let updated = store
            .update_document_content(&doc.id, new_content, None)
            .await
            .expect("update");

        assert_eq!(updated.content.as_text(), Some("Updated content"));
    }

    #[tokio::test]
    async fn test_update_metadata() {
        let store = make_store().await;
        let user_id = make_user_id();
        let metadata = DocumentMetadata::new("Old Title".to_string(), user_id);
        let content = DocumentContent::markdown("Content".to_string());

        let doc = store
            .create_document(metadata, content)
            .await
            .expect("create");

        let mut new_meta = doc.metadata.clone();
        new_meta.title = "New Title".to_string();
        new_meta.description = Some("A description".to_string());
        new_meta.tags = vec!["rust".to_string(), "test".to_string()];

        let updated = store
            .update_document_metadata(&doc.id, new_meta, None)
            .await
            .expect("update");

        assert_eq!(updated.metadata.title, "New Title");
        assert_eq!(
            updated.metadata.description,
            Some("A description".to_string())
        );
        assert_eq!(updated.metadata.tags.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_document() {
        let store = make_store().await;
        let user_id = make_user_id();
        let metadata = DocumentMetadata::new("To Delete".to_string(), user_id);
        let content = DocumentContent::markdown("Content".to_string());

        let doc = store
            .create_document(metadata, content)
            .await
            .expect("create");
        store.delete_document(&doc.id).await.expect("delete");

        let result = store
            .list_documents(ListParams::default())
            .await
            .expect("list");
        assert_eq!(result.items.len(), 0);
    }

    #[tokio::test]
    async fn test_permanent_delete() {
        let store = make_store().await;
        let user_id = make_user_id();
        let metadata = DocumentMetadata::new("Gone".to_string(), user_id);
        let content = DocumentContent::markdown("Content".to_string());

        let doc = store
            .create_document(metadata, content)
            .await
            .expect("create");
        store
            .permanently_delete_document(&doc.id)
            .await
            .expect("permanent delete");

        let result = store.get_document(&doc.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_documents() {
        let store = make_store().await;
        let user_id = make_user_id();

        for i in 0..5 {
            let metadata = DocumentMetadata::new(format!("Doc {}", i), user_id);
            let content = DocumentContent::markdown(format!("Content {}", i));
            store.create_document(metadata, content).await.unwrap();
        }

        let result = store
            .list_documents(ListParams::default())
            .await
            .expect("list");
        assert_eq!(result.total, 5);
        assert_eq!(result.items.len(), 5);
    }

    #[tokio::test]
    async fn test_pagination() {
        let store = make_store().await;
        let user_id = make_user_id();

        for i in 0..10 {
            let metadata = DocumentMetadata::new(format!("Doc {:02}", i), user_id);
            let content = DocumentContent::markdown(format!("Content {}", i));
            store.create_document(metadata, content).await.unwrap();
        }

        let page1 = store
            .list_documents(ListParams {
                page: 1,
                page_size: 3,
                ..ListParams::default()
            })
            .await
            .unwrap();
        assert_eq!(page1.total, 10);
        assert_eq!(page1.items.len(), 3);
    }

    #[tokio::test]
    async fn test_search_documents() {
        let store = make_store().await;
        let user_id = make_user_id();

        let m1 = DocumentMetadata::new("Rust Programming".to_string(), user_id);
        store
            .create_document(
                m1,
                DocumentContent::markdown("Rust is a systems language".to_string()),
            )
            .await
            .unwrap();

        let m2 = DocumentMetadata::new("Python Guide".to_string(), user_id);
        store
            .create_document(
                m2,
                DocumentContent::markdown("Python is easy to learn".to_string()),
            )
            .await
            .unwrap();

        let results = store.search_documents("Rust", 1, 10).await.expect("search");
        assert_eq!(results.total, 1);
        assert_eq!(results.items[0].metadata.title, "Rust Programming");
    }

    #[tokio::test]
    async fn test_tags() {
        let store = make_store().await;
        let user_id = make_user_id();

        let mut m1 = DocumentMetadata::new("Doc 1".to_string(), user_id);
        m1.tags = vec!["rust".to_string(), "programming".to_string()];
        store
            .create_document(m1, DocumentContent::markdown("Content 1".to_string()))
            .await
            .unwrap();

        let mut m2 = DocumentMetadata::new("Doc 2".to_string(), user_id);
        m2.tags = vec!["rust".to_string(), "systems".to_string()];
        store
            .create_document(m2, DocumentContent::markdown("Content 2".to_string()))
            .await
            .unwrap();

        let tags = store.get_all_tags().await.expect("tags");
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"programming".to_string()));

        let by_tag = store
            .get_documents_by_tag("rust", 1, 10)
            .await
            .expect("by tag");
        assert_eq!(by_tag.total, 2);
    }

    #[tokio::test]
    async fn test_list_summary() {
        let store = make_store().await;
        let user_id = make_user_id();

        let m = DocumentMetadata::new("Summary Test".to_string(), user_id);
        store
            .create_document(m, DocumentContent::markdown("Hello world test".to_string()))
            .await
            .unwrap();

        let summary = store.get_list_summary().await.expect("summary");
        assert_eq!(summary.total_documents, 1);
        assert_eq!(summary.draft_count, 1);
        assert_eq!(summary.total_word_count, 3);
    }

    #[tokio::test]
    async fn test_is_available() {
        let store = make_store().await;
        assert!(store.is_available().await.expect("available"));
    }

    #[tokio::test]
    async fn test_filter_by_author() {
        let store = make_store().await;
        let user1 = make_user_id();
        let user2 = make_user_id();

        let m1 = DocumentMetadata::new("User1 Doc".to_string(), user1);
        store
            .create_document(m1, DocumentContent::markdown("c1".to_string()))
            .await
            .unwrap();

        let m2 = DocumentMetadata::new("User2 Doc".to_string(), user2);
        store
            .create_document(m2, DocumentContent::markdown("c2".to_string()))
            .await
            .unwrap();

        let results = store
            .list_documents(ListParams {
                author_id: Some(user1),
                ..ListParams::default()
            })
            .await
            .unwrap();
        assert_eq!(results.total, 1);
        assert_eq!(results.items[0].metadata.title, "User1 Doc");
    }

    #[tokio::test]
    async fn test_sort_order() {
        let store = make_store().await;
        let user_id = make_user_id();

        let m1 = DocumentMetadata::new("Zebra".to_string(), user_id);
        store
            .create_document(m1, DocumentContent::markdown("z".to_string()))
            .await
            .unwrap();

        let m2 = DocumentMetadata::new("Alpha".to_string(), user_id);
        store
            .create_document(m2, DocumentContent::markdown("a".to_string()))
            .await
            .unwrap();

        let results = store
            .list_documents(ListParams {
                sort_by: SortField::Title,
                sort_dir: SortDirection::Asc,
                ..ListParams::default()
            })
            .await
            .unwrap();

        assert_eq!(results.items[0].metadata.title, "Alpha");
        assert_eq!(results.items[1].metadata.title, "Zebra");
    }
}
