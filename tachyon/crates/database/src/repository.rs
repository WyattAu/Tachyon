// Repository CRUD Operations
// Document and repository management

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::*;
use sqlx::{Row, query, query_as};
use tachyon_core::id::{DocumentId, RepositoryId};
use tracing::{debug, info, instrument};

/// Document repository for CRUD operations
#[derive(Clone)]
pub struct DocumentRepository {
    pool: DatabasePool,
}

impl DocumentRepository {
    /// Create a new document repository
    ///
    /// # Arguments
    /// * `pool` - Database pool
    ///
    /// # Returns
    /// New DocumentRepository instance
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a new document
    ///
    /// # Arguments
    /// * `metadata` - Document metadata
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn create(&self, metadata: DocumentMetadata) -> DatabaseResult<()> {
        let tags_json = DocumentMetadata::serialize_tags(&metadata.parse_tags()?)?;
        let frontmatter_json =
            DocumentMetadata::serialize_frontmatter(&metadata.parse_frontmatter()?)?;

        let insert_sql = r#"
            INSERT INTO documents (
                id, title, slug, author_id, description, tags, frontmatter,
                repository_id, visibility, status, content_type,
                word_count, character_count, read_count, edit_count,
                created_at, updated_at, published_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&metadata.id)
            .bind(&metadata.title)
            .bind(&metadata.slug)
            .bind(&metadata.author_id)
            .bind(&metadata.description)
            .bind(&tags_json)
            .bind(&frontmatter_json)
            .bind(&metadata.repository_id)
            .bind(&metadata.visibility)
            .bind(&metadata.status)
            .bind(&metadata.content_type)
            .bind(metadata.word_count)
            .bind(metadata.character_count)
            .bind(metadata.read_count)
            .bind(metadata.edit_count)
            .bind(&metadata.created_at)
            .bind(&metadata.updated_at)
            .bind(&metadata.published_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    DatabaseError::duplicate(
                        "document",
                        format!("Document ID {} already exists", metadata.id),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!("Document created: {}", metadata.id);
        Ok(())
    }

    /// Get a document by ID
    ///
    /// # Arguments
    /// * `id` - Document ID
    ///
    /// # Returns
    /// Result containing DocumentMetadata or error
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &DocumentId) -> DatabaseResult<DocumentMetadata> {
        let select_sql = "SELECT * FROM documents WHERE id = ?";

        let mut conn = self.pool.acquire().await?;
        let result = query_as::<_, DocumentMetadata>(select_sql)
            .bind(id.as_str())
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        result.ok_or_else(|| DatabaseError::not_found("document", id.as_str()))
    }

    /// Update a document
    ///
    /// # Arguments
    /// * `metadata` - Updated document metadata
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn update(&self, metadata: DocumentMetadata) -> DatabaseResult<()> {
        let tags_json = DocumentMetadata::serialize_tags(&metadata.parse_tags()?)?;
        let frontmatter_json =
            DocumentMetadata::serialize_frontmatter(&metadata.parse_frontmatter()?)?;

        let update_sql = r#"
            UPDATE documents SET
                title = ?, slug = ?, description = ?, tags = ?, frontmatter = ?,
                repository_id = ?, visibility = ?, status = ?, content_type = ?,
                word_count = ?, character_count = ?, read_count = ?, edit_count = ?,
                updated_at = ?, published_at = ?
            WHERE id = ?
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(&metadata.title)
            .bind(&metadata.slug)
            .bind(&metadata.description)
            .bind(&tags_json)
            .bind(&frontmatter_json)
            .bind(&metadata.repository_id)
            .bind(&metadata.visibility)
            .bind(&metadata.status)
            .bind(&metadata.content_type)
            .bind(metadata.word_count)
            .bind(metadata.character_count)
            .bind(metadata.read_count)
            .bind(metadata.edit_count)
            .bind(&metadata.updated_at)
            .bind(&metadata.published_at)
            .bind(&metadata.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("document", &metadata.id));
        }

        info!("Document updated: {}", metadata.id);
        Ok(())
    }

    /// Delete a document
    ///
    /// # Arguments
    /// * `id` - Document ID
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &DocumentId) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM documents WHERE id = ?";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("document", id.as_str()));
        }

        info!("Document deleted: {}", id.as_str());
        Ok(())
    }

    /// List documents by author
    ///
    /// # Arguments
    /// * `author_id` - Author user ID
    /// * `limit` - Maximum number of documents to return
    /// * `offset` - Offset for pagination
    ///
    /// # Returns
    /// Result containing vector of documents or error
    pub async fn list_by_author(
        &self,
        author_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        let base_sql = "SELECT * FROM documents WHERE author_id = ? ORDER BY updated_at DESC";
        let (sql, limit, offset) = apply_pagination(base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, DocumentMetadata>(&sql).bind(author_id);
        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = offset {
            query_builder = query_builder.bind(offset);
        }

        let documents = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(documents)
    }

    /// List documents by repository
    ///
    /// # Arguments
    /// * `repository_id` - Repository ID
    /// * `limit` - Maximum number of documents to return
    /// * `offset` - Offset for pagination
    ///
    /// # Returns
    /// Result containing vector of documents or error
    pub async fn list_by_repository(
        &self,
        repository_id: &RepositoryId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        let base_sql = "SELECT * FROM documents WHERE repository_id = ? ORDER BY updated_at DESC";
        let (sql, limit, offset) = apply_pagination(base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, DocumentMetadata>(&sql).bind(repository_id.as_str());
        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = offset {
            query_builder = query_builder.bind(offset);
        }

        let documents = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(documents)
    }

    /// List all documents with pagination
    ///
    /// # Arguments
    /// * `limit` - Maximum number of documents to return
    /// * `offset` - Offset for pagination
    ///
    /// # Returns
    /// Result containing vector of documents or error
    pub async fn list_all(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        let base_sql = "SELECT * FROM documents ORDER BY updated_at DESC";
        let (sql, limit, offset) = apply_pagination(base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, DocumentMetadata>(&sql);
        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = offset {
            query_builder = query_builder.bind(offset);
        }

        let documents = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(documents)
    }

    /// Search documents by tags
    ///
    /// # Arguments
    /// * `tags` - List of tags to search for
    /// * `limit` - Maximum number of documents to return
    ///
    /// # Returns
    /// Result containing vector of documents or error
    pub async fn search_by_tags(
        &self,
        tags: &[String],
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        let limit = limit.unwrap_or(50);

        let mut documents = Vec::new();
        for tag in tags {
            let select_sql = r#"
                SELECT * FROM documents
                WHERE json_array_contains(tags, ?)
                ORDER BY updated_at DESC
                LIMIT ?
            "#;

            let mut conn = self.pool.acquire().await?;
            let tag_documents = query_as::<_, DocumentMetadata>(select_sql)
                .bind(format!("\"{}\"", tag))
                .bind(limit)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            documents.extend(tag_documents);
        }

        Ok(documents)
    }

    /// Update search index for a document
    ///
    /// # Arguments
    /// * `document_id` - Document ID
    /// * `title` - Document title
    /// * `content` - Document content
    /// * `tags` - Document tags
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn update_search_index(
        &self,
        document_id: &DocumentId,
        title: &str,
        content: &str,
        tags: &[String],
    ) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM search_index WHERE document_id = ?";
        let mut conn = self.pool.acquire().await?;
        query(delete_sql)
            .bind(document_id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let insert_sql = r#"
            INSERT INTO search_index (document_id, content_type, content, weight)
            VALUES (?, 'title', ?, 2.0)
        "#;
        query(insert_sql)
            .bind(document_id.as_str())
            .bind(title)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let insert_sql = r#"
            INSERT INTO search_index (document_id, content_type, content, weight)
            VALUES (?, 'content', ?, 1.0)
        "#;
        query(insert_sql)
            .bind(document_id.as_str())
            .bind(content)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        for tag in tags {
            let insert_sql = r#"
                INSERT INTO search_index (document_id, content_type, content, weight)
                VALUES (?, 'tag', ?, 1.5)
            "#;
            query(insert_sql)
                .bind(document_id.as_str())
                .bind(tag)
                .execute(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        }

        debug!(
            "Search index updated for document: {}",
            document_id.as_str()
        );
        Ok(())
    }

    /// Full-text search
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Result containing vector of document IDs or error
    pub async fn search(
        &self,
        search_query: &str,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<String>> {
        let limit = limit.unwrap_or(50);
        let select_sql = r#"
            SELECT DISTINCT document_id FROM search_index
            WHERE search_index MATCH ?
            ORDER BY rank
            LIMIT ?
        "#;

        let mut conn = self.pool.acquire().await?;
        let results = query(select_sql)
            .bind(search_query)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let document_ids: Vec<String> = results.iter().map(|row| row.get("document_id")).collect();
        Ok(document_ids)
    }

    /// Count documents by author
    ///
    /// # Arguments
    /// * `author_id` - Author user ID
    ///
    /// # Returns
    /// Result containing document count or error
    pub async fn count_by_author(&self, author_id: &str) -> DatabaseResult<i64> {
        let count_sql = "SELECT COUNT(*) as count FROM documents WHERE author_id = ?";

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(author_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Count documents by repository
    ///
    /// # Arguments
    /// * `repository_id` - Repository ID
    ///
    /// # Returns
    /// Result containing document count or error
    pub async fn count_by_repository(&self, repository_id: &RepositoryId) -> DatabaseResult<i64> {
        let count_sql = "SELECT COUNT(*) as count FROM documents WHERE repository_id = ?";

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(repository_id.as_str())
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }
}

/// Repository management repository
pub struct RepositoryRepository {
    pool: DatabasePool,
}

impl RepositoryRepository {
    /// Create a new repository repository
    ///
    /// # Arguments
    /// * `pool` - Database pool
    ///
    /// # Returns
    /// New RepositoryRepository instance
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a new repository
    ///
    /// # Arguments
    /// * `metadata` - Repository metadata
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn create(&self, metadata: RepositoryMetadata) -> DatabaseResult<()> {
        let insert_sql = r#"
            INSERT INTO repositories (
                id, name, slug, description, repository_type, owner_id,
                visibility, status, default_branch, auto_sync, sync_interval_seconds,
                file_watching_enabled, remote_url, last_commit_hash, current_branch,
                commits_ahead, commits_behind, document_count, total_storage_bytes,
                member_count, local_path, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&metadata.id)
            .bind(&metadata.name)
            .bind(&metadata.slug)
            .bind(&metadata.description)
            .bind(&metadata.repository_type)
            .bind(&metadata.owner_id)
            .bind(&metadata.visibility)
            .bind(&metadata.status)
            .bind(&metadata.default_branch)
            .bind(metadata.auto_sync as i64)
            .bind(metadata.sync_interval_seconds)
            .bind(metadata.file_watching_enabled as i64)
            .bind(&metadata.remote_url)
            .bind(&metadata.last_commit_hash)
            .bind(&metadata.current_branch)
            .bind(metadata.commits_ahead)
            .bind(metadata.commits_behind)
            .bind(metadata.document_count)
            .bind(metadata.total_storage_bytes)
            .bind(metadata.member_count)
            .bind(&metadata.local_path)
            .bind(&metadata.created_at)
            .bind(&metadata.updated_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    DatabaseError::duplicate(
                        "repository",
                        format!("Repository ID {} already exists", metadata.id),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!("Repository created: {}", metadata.id);
        Ok(())
    }

    /// Get a repository by ID
    ///
    /// # Arguments
    /// * `id` - Repository ID
    ///
    /// # Returns
    /// Result containing RepositoryMetadata or error
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &RepositoryId) -> DatabaseResult<RepositoryMetadata> {
        let select_sql = "SELECT * FROM repositories WHERE id = ?";

        let mut conn = self.pool.acquire().await?;
        let result = query_as::<_, RepositoryMetadata>(select_sql)
            .bind(id.as_str())
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        result.ok_or_else(|| DatabaseError::not_found("repository", id.as_str()))
    }

    /// Update a repository
    ///
    /// # Arguments
    /// * `metadata` - Updated repository metadata
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn update(&self, metadata: RepositoryMetadata) -> DatabaseResult<()> {
        let update_sql = r#"
            UPDATE repositories SET
                name = ?, slug = ?, description = ?, repository_type = ?, owner_id = ?,
                visibility = ?, status = ?, default_branch = ?, auto_sync = ?, sync_interval_seconds = ?,
                file_watching_enabled = ?, remote_url = ?, last_commit_hash = ?, current_branch = ?,
                commits_ahead = ?, commits_behind = ?, document_count = ?, total_storage_bytes = ?,
                member_count = ?, local_path = ?, updated_at = ?
            WHERE id = ?
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(&metadata.name)
            .bind(&metadata.slug)
            .bind(&metadata.description)
            .bind(&metadata.repository_type)
            .bind(&metadata.owner_id)
            .bind(&metadata.visibility)
            .bind(&metadata.status)
            .bind(&metadata.default_branch)
            .bind(metadata.auto_sync as i64)
            .bind(metadata.sync_interval_seconds)
            .bind(metadata.file_watching_enabled as i64)
            .bind(&metadata.remote_url)
            .bind(&metadata.last_commit_hash)
            .bind(&metadata.current_branch)
            .bind(metadata.commits_ahead)
            .bind(metadata.commits_behind)
            .bind(metadata.document_count)
            .bind(metadata.total_storage_bytes)
            .bind(metadata.member_count)
            .bind(&metadata.local_path)
            .bind(&metadata.updated_at)
            .bind(&metadata.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("repository", &metadata.id));
        }

        info!("Repository updated: {}", metadata.id);
        Ok(())
    }

    /// Delete a repository
    ///
    /// # Arguments
    /// * `id` - Repository ID
    ///
    /// # Returns
    /// Result indicating success or error
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &RepositoryId) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM repositories WHERE id = ?";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("repository", id.as_str()));
        }

        info!("Repository deleted: {}", id.as_str());
        Ok(())
    }

    /// List repositories by owner
    ///
    /// # Arguments
    /// * `owner_id` - Owner user ID
    /// * `limit` - Maximum number of repositories to return
    /// * `offset` - Offset for pagination
    ///
    /// # Returns
    /// Result containing vector of repositories or error
    pub async fn list_by_owner(
        &self,
        owner_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<RepositoryMetadata>> {
        let base_sql = "SELECT * FROM repositories WHERE owner_id = ? ORDER BY updated_at DESC";
        let (sql, limit, offset) = apply_pagination(base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, RepositoryMetadata>(&sql).bind(owner_id);
        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = offset {
            query_builder = query_builder.bind(offset);
        }

        let repositories = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(repositories)
    }

    /// Update repository document count
    ///
    /// # Arguments
    /// * `id` - Repository ID
    /// * `delta` - Change in document count
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn update_document_count(&self, id: &RepositoryId, delta: i64) -> DatabaseResult<()> {
        let update_sql = "UPDATE repositories SET document_count = document_count + ? WHERE id = ?";

        let mut conn = self.pool.acquire().await?;
        query(update_sql)
            .bind(delta)
            .bind(id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(())
    }

    /// Update repository storage bytes
    ///
    /// # Arguments
    /// * `id` - Repository ID
    /// * `bytes` - Total storage bytes
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn update_storage_bytes(&self, id: &RepositoryId, bytes: i64) -> DatabaseResult<()> {
        let update_sql = "UPDATE repositories SET total_storage_bytes = ? WHERE id = ?";

        let mut conn = self.pool.acquire().await?;
        query(update_sql)
            .bind(bytes)
            .bind(id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(())
    }
}

/// Apply pagination to SQL query
///
/// # Arguments
/// * `base_sql` - Base SQL query
/// * `limit` - Optional limit
/// * `offset` - Optional offset
///
/// # Returns
/// Tuple of (sql with pagination, limit, offset)
fn apply_pagination(
    base_sql: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> (String, Option<i64>, Option<i64>) {
    match (limit, offset) {
        (Some(l), Some(o)) => (format!("{} LIMIT ? OFFSET ?", base_sql), Some(l), Some(o)),
        (Some(l), None) => (format!("{} LIMIT ?", base_sql), Some(l), None),
        (None, Some(o)) => (format!("{} OFFSET ?", base_sql), None, Some(o)),
        (None, None) => (base_sql.to_string(), None, None),
    }
}

/// Extension for Option to provide or_else
trait OptionExt<T> {
    fn ok_or_else<F>(self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce() -> DatabaseError;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_else<F>(self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce() -> DatabaseError,
    {
        self.ok_or_else(f)
    }
}
