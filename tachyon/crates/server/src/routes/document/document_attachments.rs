use crate::error::ServerError;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;

use super::DocumentState;

#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub document_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: String,
    pub created_by: String,
}

impl From<tachyon_database::Attachment> for AttachmentResponse {
    fn from(a: tachyon_database::Attachment) -> Self {
        Self {
            id: a.id,
            document_id: a.document_id,
            filename: a.filename,
            mime_type: a.mime_type,
            size: a.size,
            created_at: a.created_at.to_rfc3339(),
            created_by: a.created_by,
        }
    }
}

/// List attachments for a document.
///
/// `GET /api/v1/documents/{document_id}/attachments`
pub async fn list_attachments(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<Vec<AttachmentResponse>>, ServerError> {
    let repo = tachyon_database::AttachmentRepository::new(state.pool.clone());
    let attachments = repo
        .list_by_document(&document_id)
        .await
        .map_err(|e| ServerError::database(format!("Failed to list attachments: {}", e)))?;

    Ok(Json(
        attachments
            .into_iter()
            .map(AttachmentResponse::from)
            .collect(),
    ))
}

/// Upload an attachment to a document.
///
/// `POST /api/v1/documents/{document_id}/attachments`
///
/// Accepts multipart form data with a single file field.
pub async fn upload_attachment(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
    mut multipart: Multipart,
) -> Result<Json<AttachmentResponse>, ServerError> {
    let user_id = tachyon_core::generate_user_id();
    let repo = tachyon_database::AttachmentRepository::new(state.pool.clone());

    if let Some(field) = multipart.next_field().await.ok().flatten() {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let mime_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let content = field
            .bytes()
            .await
            .map_err(|e| ServerError::bad_request(format!("Failed to read file: {}", e)))?;

        let attachment = repo
            .create(tachyon_database::CreateAttachmentRequest {
                document_id: document_id.clone(),
                filename,
                mime_type,
                content: content.to_vec(),
                created_by: user_id.to_string(),
            })
            .await
            .map_err(|e| ServerError::database(format!("Failed to create attachment: {}", e)))?;

        return Ok(Json(AttachmentResponse::from(attachment)));
    }

    Err(ServerError::bad_request("No file provided"))
}

/// Download an attachment.
///
/// `GET /api/v1/documents/{document_id}/attachments/{attachment_id}`
///
/// Returns the raw file content with appropriate `Content-Type` and `Content-Disposition` headers.
pub async fn download_attachment(
    Path((_document_id, attachment_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
) -> Result<impl IntoResponse, ServerError> {
    let repo = tachyon_database::AttachmentRepository::new(state.pool.clone());
    let (attachment, content) = repo
        .get_content(&attachment_id)
        .await
        .map_err(|e| ServerError::not_found("Attachment", &format!("{}", e)))?;

    let headers = [
        ("Content-Type", attachment.mime_type.clone()),
        (
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", attachment.filename),
        ),
    ];

    Ok((headers, content))
}

/// Delete an attachment.
///
/// `DELETE /api/v1/documents/{document_id}/attachments/{attachment_id}`
pub async fn delete_attachment(
    Path((_document_id, attachment_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
) -> Result<StatusCode, ServerError> {
    let repo = tachyon_database::AttachmentRepository::new(state.pool.clone());
    repo.delete(&attachment_id)
        .await
        .map_err(|e| ServerError::not_found("Attachment", &format!("{}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}
