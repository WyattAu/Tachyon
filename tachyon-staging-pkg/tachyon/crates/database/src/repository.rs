// Repository CRUD Operations
// Document and repository management

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::*;
use sqlx::{Row, query};
use tachyon_core::id::{DocumentId, RepositoryId};
use tracing::{debug, info, instrument};

/// Document SELECT SQL with UUID casting for PostgreSQL
const DOCUMENT_SELECT_SQL: &str = r#"
    SELECT 
        id::text as id,
        title,
        slug,
        author_id::text as author_id,
        description,
        tags::text as tags,
        frontmatter::text as frontmatter,
        project_id::text as project_id,
        visibility,
        status,
        content_type,
        word_count,
        character_count,
        read_count,
        edit_count,
        content,
        html,
        created_at,
        updated_at,
        published_at,
        content_hash,
        conflict_detected
    FROM documents
"#;

/// Lightweight document SELECT excluding heavy content/html columns for list queries.
const DOCUMENT_SUMMARY_SELECT_SQL: &str = r#"
    SELECT 
        id::text as id,
        title,
        slug,
        author_id::text as author_id,
        description,
        tags::text as tags,
        frontmatter::text as frontmatter,
        project_id::text as project_id,
        visibility,
        status,
        content_type,
        word_count,
        character_count,
        read_count,
        edit_count,
        NULL::text as content,
        NULL::text as html,
        created_at,
        updated_at,
        published_at,
        content_hash,
        conflict_detected
    FROM documents
"#;

/// Repository SELECT SQL with UUID casting for PostgreSQL
const REPOSITORY_SELECT_SQL: &str = r#"
    SELECT 
        id::text as id,
        name,
        slug,
        description,
        repository_type,
        owner_id::text as owner_id,
        visibility,
        status,
        default_branch,
        auto_sync,
        sync_interval_seconds,
        file_watching_enabled,
        remote_url,
        last_commit_hash,
        current_branch,
        commits_ahead,
        commits_behind,
        document_count,
        total_storage_bytes,
        member_count,
        local_path,
        created_at,
        updated_at
    FROM repositories
"#;

/// Helper to construct DocumentMetadata from a database row
fn row_to_document_metadata(row: sqlx::postgres::PgRow) -> DatabaseResult<DocumentMetadata> {
    Ok(DocumentMetadata {
        id: row.get("id"),
        title: row.get("title"),
        slug: row.get("slug"),
        author_id: row.get("author_id"),
        description: row.get("description"),
        tags: row.get("tags"),
        frontmatter: row.get("frontmatter"),
        project_id: row.get("project_id"),
        visibility: row.get("visibility"),
        status: row.get("status"),
        content_type: row.get("content_type"),
        word_count: row.get("word_count"),
        character_count: row.get("character_count"),
        read_count: row.get("read_count"),
        edit_count: row.get("edit_count"),
        content: row.get("content"),
        html: row.get("html"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        published_at: row.get("published_at"),
        content_hash: row.get("content_hash"),
        conflict_detected: row.get("conflict_detected"),
    })
}

/// Helper to construct RepositoryMetadata from a database row
fn row_to_repository_metadata(row: sqlx::postgres::PgRow) -> DatabaseResult<RepositoryMetadata> {
    Ok(RepositoryMetadata {
        id: row.get("id"),
        name: row.get("name"),
        slug: row.get("slug"),
        description: row.get("description"),
        repository_type: row.get("repository_type"),
        owner_id: row.get("owner_id"),
        visibility: row.get("visibility"),
        status: row.get("status"),
        default_branch: row.get("default_branch"),
        auto_sync: row.get("auto_sync"),
        sync_interval_seconds: row.get("sync_interval_seconds"),
        file_watching_enabled: row.get("file_watching_enabled"),
        remote_url: row.get("remote_url"),
        last_commit_hash: row.get("last_commit_hash"),
        current_branch: row.get("current_branch"),
        commits_ahead: row.get("commits_ahead"),
        commits_behind: row.get("commits_behind"),
        document_count: row.get("document_count"),
        total_storage_bytes: row.get("total_storage_bytes"),
        member_count: row.get("member_count"),
        local_path: row.get("local_path"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

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
                project_id, visibility, status, content_type,
                word_count, character_count, read_count, edit_count,
                content, html,
                created_at, updated_at, published_at,
                content_hash, conflict_detected
            ) VALUES ($1::uuid, $2, $3, $4::uuid, $5, $6::jsonb, $7::jsonb, $8::uuid, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
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
            .bind(&metadata.project_id)
            .bind(&metadata.visibility)
            .bind(&metadata.status)
            .bind(&metadata.content_type)
            .bind(metadata.word_count)
            .bind(metadata.character_count)
            .bind(metadata.read_count)
            .bind(metadata.edit_count)
            .bind(&metadata.content)
            .bind(&metadata.html)
            .bind(metadata.created_at)
            .bind(metadata.updated_at)
            .bind(metadata.published_at)
            .bind(metadata.content_hash.as_deref())
            .bind(metadata.conflict_detected.unwrap_or(false))
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("UNIQUE constraint")
                {
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
        let select_sql = format!("{} WHERE id = $1::uuid", DOCUMENT_SELECT_SQL);

        let mut conn = self.pool.acquire().await?;
        let row = query(&select_sql)
            .bind(id.as_str())
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match row {
            Some(r) => row_to_document_metadata(r),
            None => Err(DatabaseError::not_found("document", id.as_str())),
        }
    }

    /// Get multiple documents by ID in a single query.
    ///
    /// Returns an empty vec if `ids` is empty.
    pub async fn get_by_ids_batch(
        &self,
        ids: &[DocumentId],
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let select_sql = format!("{} WHERE id = ANY($1::uuid[])", DOCUMENT_SELECT_SQL);
        let id_strs: Vec<String> = ids.iter().map(|id| id.as_str()).collect();

        let mut conn = self.pool.acquire().await?;
        let rows = query(&select_sql)
            .bind(&id_strs)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        rows.into_iter().map(row_to_document_metadata).collect()
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
                title = $1, slug = $2, description = $3, tags = $4::jsonb, frontmatter = $5::jsonb,
                project_id = $6::uuid, visibility = $7, status = $8, content_type = $9,
                word_count = $10, character_count = $11, read_count = $12, edit_count = $13,
                content = $16, html = $17,
                updated_at = $14, published_at = $15,
                content_hash = COALESCE($19, content_hash), conflict_detected = COALESCE($20, conflict_detected)
            WHERE id = $18::uuid
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(&metadata.title)
            .bind(&metadata.slug)
            .bind(&metadata.description)
            .bind(&tags_json)
            .bind(&frontmatter_json)
            .bind(&metadata.project_id)
            .bind(&metadata.visibility)
            .bind(&metadata.status)
            .bind(&metadata.content_type)
            .bind(metadata.word_count)
            .bind(metadata.character_count)
            .bind(metadata.read_count)
            .bind(metadata.edit_count)
            .bind(metadata.updated_at)
            .bind(metadata.published_at)
            .bind(&metadata.content)
            .bind(&metadata.html)
            .bind(&metadata.id)
            .bind(metadata.content_hash.as_deref())
            .bind(metadata.conflict_detected)
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
        let delete_sql = "DELETE FROM documents WHERE id = $1::uuid";

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
        let base_sql = format!(
            "{} WHERE author_id = $1::uuid ORDER BY updated_at DESC",
            DOCUMENT_SUMMARY_SELECT_SQL
        );
        let (sql, limit_val, offset_val) = apply_pagination(&base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = sqlx::query(&sql).bind(author_id);
        if let Some(l) = limit_val {
            query_builder = query_builder.bind(l);
        }
        if let Some(o) = offset_val {
            query_builder = query_builder.bind(o);
        }

        let rows = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        rows.into_iter().map(row_to_document_metadata).collect()
    }

    /// List documents by project
    ///
    /// # Arguments
    /// * `project_id` - Project ID
    /// * `limit` - Maximum number of documents to return
    /// * `offset` - Offset for pagination
    ///
    /// # Returns
    /// Result containing vector of documents or error
    pub async fn list_by_project(
        &self,
        project_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        let base_sql = format!(
            "{} WHERE project_id = $1::uuid ORDER BY updated_at DESC",
            DOCUMENT_SUMMARY_SELECT_SQL
        );
        let (sql, limit_val, offset_val) = apply_pagination(&base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = sqlx::query(&sql).bind(project_id);
        if let Some(l) = limit_val {
            query_builder = query_builder.bind(l);
        }
        if let Some(o) = offset_val {
            query_builder = query_builder.bind(o);
        }

        let rows = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        rows.into_iter().map(row_to_document_metadata).collect()
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
        let base_sql = format!("{} ORDER BY updated_at DESC", DOCUMENT_SUMMARY_SELECT_SQL);
        let (sql, limit_val, offset_val) = apply_pagination(&base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = sqlx::query(&sql);
        if let Some(l) = limit_val {
            query_builder = query_builder.bind(l);
        }
        if let Some(o) = offset_val {
            query_builder = query_builder.bind(o);
        }

        let rows = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        rows.into_iter().map(row_to_document_metadata).collect()
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

        // Build a single query that matches ANY of the tags using OR conditions
        let tag_conditions: Vec<String> = tags
            .iter()
            .enumerate()
            .map(|(i, _)| format!("tags::jsonb @> ${}::jsonb", i + 1))
            .collect();
        let where_clause = tag_conditions.join(" OR ");

        let select_sql = format!(
            "{} WHERE {} ORDER BY updated_at DESC LIMIT ${}",
            DOCUMENT_SELECT_SQL,
            where_clause,
            tags.len() + 1
        );

        let mut conn = self.pool.acquire().await?;
        let mut q = query(&select_sql);
        for tag in tags {
            let tag_json = serde_json::to_string(&vec![tag])
                .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
            q = q.bind(tag_json);
        }
        q = q.bind(limit);

        let rows = q
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let documents = rows
            .into_iter()
            .map(row_to_document_metadata)
            .collect::<Result<Vec<_>, _>>()?;

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
        let delete_sql = "DELETE FROM search_index WHERE document_id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        query(delete_sql)
            .bind(document_id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let insert_sql = r#"
            INSERT INTO search_index (document_id, content_type, content, weight)
            VALUES ($1::uuid, 'title', $2, 2.0),
                   ($1::uuid, 'content', $3, 1.0)
        "#;
        query(insert_sql)
            .bind(document_id.as_str())
            .bind(title)
            .bind(content)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Batch-insert all tags in a single query using UNNEST
        if !tags.is_empty() {
            let tag_insert = r#"
                INSERT INTO search_index (document_id, content_type, content, weight)
                SELECT $1::uuid, 'tag', unnest($2::text[]), 1.5
            "#;
            let tag_array: Vec<String> = tags.to_vec();
            query(tag_insert)
                .bind(document_id.as_str())
                .bind(&tag_array)
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
        // PostgreSQL full-text search using to_tsvector and to_tsquery
        let select_sql = r#"
            SELECT DISTINCT document_id::text as document_id FROM search_index
            WHERE to_tsvector('english', content) @@ to_tsquery('english', $1)
            ORDER BY document_id
            LIMIT $2
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

    /// Find a document by its slug.
    pub async fn get_by_slug(&self, slug: &str) -> DatabaseResult<Option<DocumentMetadata>> {
        let select_sql = format!("{} WHERE slug = $1 LIMIT 1", DOCUMENT_SELECT_SQL);
        let mut conn = self.pool.acquire().await?;
        let row = query(&select_sql)
            .bind(slug)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        match row {
            Some(r) => Ok(Some(row_to_document_metadata(r)?)),
            None => Ok(None),
        }
    }

    /// Find multiple documents by their slugs in a single query.
    pub async fn get_by_slugs_batch(
        &self,
        slugs: &[String],
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        if slugs.is_empty() {
            return Ok(vec![]);
        }

        let select_sql = format!("{} WHERE slug = ANY($1::text[])", DOCUMENT_SELECT_SQL);
        let mut conn = self.pool.acquire().await?;
        let rows = query(&select_sql)
            .bind(slugs)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        rows.into_iter().map(row_to_document_metadata).collect()
    }

    /// Find a document by its content hash (for deduplication).
    pub async fn get_by_content_hash(
        &self,
        content_hash: &str,
    ) -> DatabaseResult<Option<DocumentMetadata>> {
        let select_sql = format!("{} WHERE content_hash = $1 LIMIT 1", DOCUMENT_SELECT_SQL);
        let mut conn = self.pool.acquire().await?;
        let row = query(&select_sql)
            .bind(content_hash)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        match row {
            Some(r) => Ok(Some(row_to_document_metadata(r)?)),
            None => Ok(None),
        }
    }

    /// Clear the conflict_detected flag for a document.
    pub async fn clear_conflict(&self, id: &DocumentId) -> DatabaseResult<()> {
        let sql = "UPDATE documents SET conflict_detected = false, updated_at = NOW() WHERE id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("document", id.as_str()));
        }
        Ok(())
    }

    /// Bulk-update the status of multiple documents in a single query.
    pub async fn batch_update_status(
        &self,
        ids: &[DocumentId],
        status: &str,
        published_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> DatabaseResult<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        let id_strs: Vec<String> = ids.iter().map(|id| id.as_str()).collect();
        let sql = r#"
            UPDATE documents
            SET status = $1, updated_at = NOW(), published_at = COALESCE($2, published_at)
            WHERE id = ANY($3::uuid[])
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(status)
            .bind(published_at)
            .bind(&id_strs)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    /// List documents using cursor-based pagination.
    ///
    /// When `cursor` is `Some`, the cursor string is decoded (format: `{id}:{direction}`)
    /// and rows are fetched with `WHERE id < $cursor_id ORDER BY id DESC LIMIT $limit`.
    /// When `cursor` is `None`, rows are fetched with
    /// `ORDER BY updated_at DESC, id DESC LIMIT $limit`.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of documents to return
    /// * `cursor` - Optional opaque cursor string (`{id}:{direction}`)
    ///
    /// # Returns
    /// Result containing vector of documents or error
    pub async fn list_after_cursor(
        &self,
        limit: i64,
        cursor: Option<&str>,
    ) -> DatabaseResult<Vec<DocumentMetadata>> {
        let mut conn = self.pool.acquire().await?;

        if let Some(cursor_str) = cursor {
            let parts: Vec<&str> = cursor_str.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(DatabaseError::ValidationError(
                    "Invalid cursor format: expected {{id}}:{{direction}}".to_string(),
                ));
            }
            let cursor_id = parts[0];

            let sql = format!(
                "{} WHERE id < $1::uuid ORDER BY id DESC LIMIT $2",
                DOCUMENT_SUMMARY_SELECT_SQL
            );
            let rows = query(&sql)
                .bind(cursor_id)
                .bind(limit)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            rows.into_iter().map(row_to_document_metadata).collect()
        } else {
            let sql = format!(
                "{} ORDER BY updated_at DESC, id DESC LIMIT $1",
                DOCUMENT_SUMMARY_SELECT_SQL
            );
            let rows = query(&sql)
                .bind(limit)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            rows.into_iter().map(row_to_document_metadata).collect()
        }
    }

    /// Count all documents.
    ///
    /// # Returns
    /// Result containing total document count or error
    pub async fn count_documents(&self) -> DatabaseResult<i64> {
        let count_sql = "SELECT COUNT(*) as count FROM documents";

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Update the embedding vector for a document.
    pub async fn update_embedding(
        &self,
        id: &str,
        embedding: Vec<f32>,
    ) -> Result<(), DatabaseError> {
        let emb_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let mut conn = self.pool.acquire().await?;
        query("UPDATE documents SET embedding = $1::vector WHERE id = $2")
            .bind(&emb_str)
            .bind(id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    /// Search documents by semantic similarity using cosine distance.
    pub async fn search_semantic(
        &self,
        embedding: Vec<f32>,
        limit: i64,
        threshold: f32,
    ) -> Result<Vec<DocumentMetadata>, DatabaseError> {
        let emb_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let mut conn = self.pool.acquire().await?;
        let rows = query(&format!(
            "SELECT {} FROM documents WHERE embedding IS NOT NULL AND 1 - (embedding <=> $1::vector) > $2 ORDER BY embedding <=> $1::vector LIMIT $3",
            DOCUMENT_SUMMARY_SELECT_SQL
        ))
        .bind(&emb_str)
        .bind(threshold)
        .bind(limit)
        .fetch_all(&mut *conn)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| row_to_document_metadata(r).ok())
            .collect())
    }

    /// Count documents by author
    ///
    /// # Arguments
    /// * `author_id` - Author user ID
    ///
    /// # Returns
    /// Result containing document count or error
    pub async fn count_by_author(&self, author_id: &str) -> DatabaseResult<i64> {
        let count_sql = "SELECT COUNT(*) as count FROM documents WHERE author_id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(author_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Count documents by project
    ///
    /// # Arguments
    /// * `project_id` - Project ID
    ///
    /// # Returns
    /// Result containing document count or error
    pub async fn count_by_project(&self, project_id: &str) -> DatabaseResult<i64> {
        let count_sql = "SELECT COUNT(*) as count FROM documents WHERE project_id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(project_id)
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
            ) VALUES ($1::uuid, $2, $3, $4, $5, $6::uuid, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
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
            .bind(metadata.auto_sync)
            .bind(metadata.sync_interval_seconds)
            .bind(metadata.file_watching_enabled)
            .bind(&metadata.remote_url)
            .bind(&metadata.last_commit_hash)
            .bind(&metadata.current_branch)
            .bind(metadata.commits_ahead)
            .bind(metadata.commits_behind)
            .bind(metadata.document_count)
            .bind(metadata.total_storage_bytes)
            .bind(metadata.member_count)
            .bind(&metadata.local_path)
            .bind(metadata.created_at)
            .bind(metadata.updated_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("UNIQUE constraint")
                {
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
        let select_sql = format!("{} WHERE id = $1::uuid", REPOSITORY_SELECT_SQL);

        let mut conn = self.pool.acquire().await?;
        let row = query(&select_sql)
            .bind(id.as_str())
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match row {
            Some(r) => row_to_repository_metadata(r),
            None => Err(DatabaseError::not_found("repository", id.as_str())),
        }
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
                name = $1, slug = $2, description = $3, repository_type = $4, owner_id = $5::uuid,
                visibility = $6, status = $7, default_branch = $8, auto_sync = $9, sync_interval_seconds = $10,
                file_watching_enabled = $11, remote_url = $12, last_commit_hash = $13, current_branch = $14,
                commits_ahead = $15, commits_behind = $16, document_count = $17, total_storage_bytes = $18,
                member_count = $19, local_path = $20, updated_at = $21
            WHERE id = $22::uuid
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
            .bind(metadata.auto_sync)
            .bind(metadata.sync_interval_seconds)
            .bind(metadata.file_watching_enabled)
            .bind(&metadata.remote_url)
            .bind(&metadata.last_commit_hash)
            .bind(&metadata.current_branch)
            .bind(metadata.commits_ahead)
            .bind(metadata.commits_behind)
            .bind(metadata.document_count)
            .bind(metadata.total_storage_bytes)
            .bind(metadata.member_count)
            .bind(&metadata.local_path)
            .bind(metadata.updated_at)
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
        let delete_sql = "DELETE FROM repositories WHERE id = $1::uuid";

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
        let base_sql = format!(
            "{} WHERE owner_id = $1::uuid ORDER BY updated_at DESC",
            REPOSITORY_SELECT_SQL
        );
        let (sql, limit_val, offset_val) = apply_pagination(&base_sql, limit, offset);

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = sqlx::query(&sql).bind(owner_id);
        if let Some(l) = limit_val {
            query_builder = query_builder.bind(l);
        }
        if let Some(o) = offset_val {
            query_builder = query_builder.bind(o);
        }

        let rows = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        rows.into_iter().map(row_to_repository_metadata).collect()
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
        let update_sql =
            "UPDATE repositories SET document_count = document_count + $1 WHERE id = $2::uuid";

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
        let update_sql = "UPDATE repositories SET total_storage_bytes = $1 WHERE id = $2::uuid";

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
    let effective_limit = limit.unwrap_or(100);
    let effective_offset = offset.unwrap_or(0);
    (
        format!(
            "{} LIMIT ${} OFFSET ${}",
            base_sql,
            count_placeholders(base_sql) + 1,
            count_placeholders(base_sql) + 2
        ),
        Some(effective_limit),
        Some(effective_offset),
    )
}

/// Count the number of $N placeholders in a SQL query
fn count_placeholders(sql: &str) -> usize {
    let mut count = 0;
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                chars.next();
            }
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_placeholders_empty() {
        assert_eq!(count_placeholders("SELECT 1"), 0);
    }

    #[test]
    fn test_count_placeholders_single() {
        assert_eq!(count_placeholders("SELECT * FROM t WHERE id = $1"), 1);
    }

    #[test]
    fn test_count_placeholders_multiple() {
        assert_eq!(
            count_placeholders("SELECT * FROM t WHERE a = $1 AND b = $2 LIMIT $3"),
            3
        );
    }

    #[test]
    fn test_apply_pagination_defaults() {
        let base = "SELECT * FROM t ORDER BY id".to_string();
        let (sql, limit, offset) = apply_pagination(&base, None, None);
        assert_eq!(limit, Some(100));
        assert_eq!(offset, Some(0));
        assert!(sql.contains("LIMIT $1 OFFSET $2"));
    }

    #[test]
    fn test_apply_pagination_with_values() {
        let base = "SELECT * FROM t WHERE x = $1 ORDER BY id".to_string();
        let (sql, limit, offset) = apply_pagination(&base, Some(25), Some(50));
        assert_eq!(limit, Some(25));
        assert_eq!(offset, Some(50));
        assert!(sql.contains("LIMIT $2 OFFSET $3"));
    }

    #[test]
    fn test_cursor_decode_valid() {
        let cursor = "abc123:asc";
        let parts: Vec<&str> = cursor.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "abc123");
        assert_eq!(parts[1], "asc");
    }

    #[test]
    fn test_cursor_decode_no_colon() {
        let cursor = "abc123";
        let parts: Vec<&str> = cursor.splitn(2, ':').collect();
        assert_eq!(parts.len(), 1);
    }

    #[tokio::test]
    async fn test_list_after_cursor_invalid_format() {
        let pool = DatabasePool::new("postgres://localhost:5432/test").await;
        if pool.is_err() {
            return;
        }
        let repo = DocumentRepository::new(pool.unwrap());
        let result = repo.list_after_cursor(10, Some("invalidcursor")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_after_cursor_no_db() {
        let pool = DatabasePool::new("postgres://localhost:5432/nonexistent_test").await;
        if pool.is_err() {
            return;
        }
        let repo = DocumentRepository::new(pool.unwrap());
        let result = repo.list_after_cursor(10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_count_documents_no_db() {
        let pool = DatabasePool::new("postgres://localhost:5432/nonexistent_test").await;
        if pool.is_err() {
            return;
        }
        let repo = DocumentRepository::new(pool.unwrap());
        let result = repo.count_documents().await;
        assert!(result.is_err());
    }
}
