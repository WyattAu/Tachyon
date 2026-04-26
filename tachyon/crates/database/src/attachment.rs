// Document Attachment Repository
// File attachment management for documents

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, Row};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument};

const ATTACHMENT_SELECT_SQL: &str = r#"
    SELECT 
        id::text as id,
        document_id::text as document_id,
        filename,
        mime_type,
        size,
        storage_path,
        created_at,
        created_by::text as created_by
    FROM document_attachments
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: String,
    pub document_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachmentRequest {
    pub document_id: String,
    pub filename: String,
    pub mime_type: String,
    pub content: Vec<u8>,
    pub created_by: String,
}

#[derive(Clone)]
pub struct AttachmentRepository {
    pool: DatabasePool,
    upload_dir: PathBuf,
}

impl AttachmentRepository {
    pub fn new(pool: DatabasePool) -> Self {
        let upload_dir = PathBuf::from("/uploads");
        Self { pool, upload_dir }
    }

    pub fn with_upload_dir(pool: DatabasePool, upload_dir: PathBuf) -> Self {
        Self { pool, upload_dir }
    }

    async fn ensure_upload_dir(&self) -> DatabaseResult<()> {
        if !self.upload_dir.exists() {
            fs::create_dir_all(&self.upload_dir)
                .await
                .map_err(|e| DatabaseError::InternalError(format!("Failed to create upload directory: {}", e)))?;
        }
        Ok(())
    }

    #[instrument(skip(self, req))]
    pub async fn create(&self, req: CreateAttachmentRequest) -> DatabaseResult<Attachment> {
        self.ensure_upload_dir().await?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let size = req.content.len() as i64;

        let storage_path = format!(
            "{}/{}_{}",
            self.upload_dir.display(),
            req.document_id,
            sanitize_filename(&req.filename)
        );

        let mut file = fs::File::create(&storage_path)
            .await
            .map_err(|e| DatabaseError::InternalError(format!("Failed to create file: {}", e)))?;
        
        file.write_all(&req.content)
            .await
            .map_err(|e| DatabaseError::InternalError(format!("Failed to write file: {}", e)))?;

        let insert_sql = r#"
            INSERT INTO document_attachments (
                id, document_id, filename, mime_type, size, storage_path, created_at, created_by
            ) VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7, $8::uuid)
            RETURNING id::text as id, document_id::text as document_id, filename, mime_type, size, storage_path, created_at, created_by::text as created_by
        "#;

        let mut conn = self.pool.acquire().await?;
        let attachment: Attachment = query_as(insert_sql)
            .bind(&id)
            .bind(&req.document_id)
            .bind(&req.filename)
            .bind(&req.mime_type)
            .bind(size)
            .bind(&storage_path)
            .bind(now)
            .bind(&req.created_by)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Attachment created: {} for document {}", req.filename, req.document_id);
        Ok(attachment)
    }

    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Attachment> {
        let select_sql = format!("{} WHERE id = $1::uuid", ATTACHMENT_SELECT_SQL);
        
        let mut conn = self.pool.acquire().await?;
        let attachment: Option<Attachment> = query_as(&select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        attachment.ok_or_else(|| DatabaseError::not_found("attachment", id))
    }

    #[instrument(skip(self))]
    pub async fn list_by_document(&self, document_id: &str) -> DatabaseResult<Vec<Attachment>> {
        let select_sql = format!(
            "{} WHERE document_id = $1::uuid ORDER BY created_at DESC",
            ATTACHMENT_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let attachments: Vec<Attachment> = query_as(&select_sql)
            .bind(document_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!("Found {} attachments for document {}", attachments.len(), document_id);
        Ok(attachments)
    }

    #[instrument(skip(self))]
    pub async fn get_content(&self, id: &str) -> DatabaseResult<(Attachment, Vec<u8>)> {
        let attachment = self.get_by_id(id).await?;
        
        let content = fs::read(&attachment.storage_path)
            .await
            .map_err(|e| DatabaseError::InternalError(format!("Failed to read file: {}", e)))?;

        Ok((attachment, content))
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let attachment = self.get_by_id(id).await?;

        if fs::metadata(&attachment.storage_path).await.is_ok() {
            fs::remove_file(&attachment.storage_path)
                .await
                .map_err(|e| DatabaseError::InternalError(format!("Failed to delete file: {}", e)))?;
        }

        let delete_sql = "DELETE FROM document_attachments WHERE id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("attachment", id));
        }

        info!("Attachment deleted: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete_by_document(&self, document_id: &str) -> DatabaseResult<()> {
        let attachments = self.list_by_document(document_id).await?;

        for attachment in attachments {
            if fs::metadata(&attachment.storage_path).await.is_ok() {
                let _ = fs::remove_file(&attachment.storage_path).await;
            }
        }

        let delete_sql = "DELETE FROM document_attachments WHERE document_id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        query(delete_sql)
            .bind(document_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("All attachments deleted for document: {}", document_id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn total_size_by_document(&self, document_id: &str) -> DatabaseResult<i64> {
        let sum_sql = "SELECT COALESCE(SUM(size), 0) as total_size FROM document_attachments WHERE document_id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(sum_sql)
            .bind(document_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("total_size"))
    }
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
