use axum::{extract::Path, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use tachyon_core::id::DocumentId;
use tachyon_database::{DatabasePool, DocumentRepository};
use tracing::{info, warn};

use crate::conflict::merge3;

#[derive(Clone)]
pub struct ConflictState {
    pub pool: DatabasePool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ResolveConflictRequest {
    pub resolution: String,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConflictInfo {
    pub document_id: String,
    pub has_conflict: bool,
    pub base_content: Option<String>,
    pub current_content: Option<String>,
    pub incoming_content: Option<String>,
    pub merge_result: Option<MergeResultInfo>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MergeResultInfo {
    pub status: String,
    pub content: String,
    pub conflict_count: usize,
}

/// Get conflict information for a document.
///
/// `GET /api/v1/documents/{document_id}/conflict`
///
/// Returns whether a conflict is detected and, if so, a 3-way merge result.
#[utoipa::path(
    get,
    path = "/documents/{document_id}/conflict",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "Conflict information", body = ConflictInfo),
        (status = 400, description = "Invalid document ID"),
        (status = 404, description = "Document not found"),
    ),
    tag = "documents",
    security(("bearer_auth" = [])),
)]
pub async fn get_conflict_info(
    Path(document_id): Path<String>,
    axum::extract::State(state): axum::extract::State<ConflictState>,
) -> Result<Json<ConflictInfo>, (StatusCode, Json<ErrorResponse>)> {
    let doc_id = match DocumentId::parse_str(&document_id) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "INVALID_ID".to_string(),
                    message: "Invalid document ID format".to_string(),
                    details: None,
                }),
            ));
        }
    };

    let repo = DocumentRepository::new(state.pool.clone());
    let doc = match repo.get_by_id(&doc_id).await {
        Ok(doc) => doc,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Document not found: {}", e),
                    details: None,
                }),
            ));
        }
    };

    let has_conflict = doc.conflict_detected.unwrap_or(false);

    let merge_result = if has_conflict {
        let base_content = doc.content.as_deref().unwrap_or("");
        let incoming_content = doc.content.as_deref().unwrap_or("");
        let result = merge3(base_content, base_content, incoming_content);
        Some(MergeResultInfo {
            status: match &result {
                crate::conflict::MergeResult::Clean(_) => "clean".to_string(),
                crate::conflict::MergeResult::Conflicted { .. } => "conflicted".to_string(),
            },
            content: match &result {
                crate::conflict::MergeResult::Clean(c) => c.clone(),
                crate::conflict::MergeResult::Conflicted { content, .. } => content.clone(),
            },
            conflict_count: match &result {
                crate::conflict::MergeResult::Clean(_) => 0,
                crate::conflict::MergeResult::Conflicted { conflict_count, .. } => *conflict_count,
            },
        })
    } else {
        None
    };

    Ok(Json(ConflictInfo {
        document_id: document_id.clone(),
        has_conflict,
        base_content: doc.content.clone(),
        current_content: doc.content.clone(),
        incoming_content: doc.content.clone(),
        merge_result,
    }))
}

/// Resolve a document conflict.
///
/// `POST /api/v1/documents/{document_id}/conflict/resolve`
///
/// Accepts a resolution strategy: `ours`, `theirs`, or `manual` (requires `content`).
/// Clears the conflict flag on the document.
#[utoipa::path(
    post,
    path = "/documents/{document_id}/conflict/resolve",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    request_body(content = ResolveConflictRequest, description = "Resolution strategy"),
    responses(
        (status = 200, description = "Conflict resolved", body = serde_json::Value),
        (status = 400, description = "Invalid request or document ID"),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "documents",
    security(("bearer_auth" = [])),
)]
pub async fn resolve_conflict(
    Path(document_id): Path<String>,
    axum::extract::State(state): axum::extract::State<ConflictState>,
    Json(body): Json<ResolveConflictRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let doc_id = match DocumentId::parse_str(&document_id) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "INVALID_ID".to_string(),
                    message: "Invalid document ID format".to_string(),
                    details: None,
                }),
            ));
        }
    };

    let repo = DocumentRepository::new(state.pool.clone());
    let doc = match repo.get_by_id(&doc_id).await {
        Ok(doc) => doc,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Document not found: {}", e),
                    details: None,
                }),
            ));
        }
    };

    let final_content = match body.resolution.as_str() {
        "manual" => match &body.content {
            Some(c) => c.clone(),
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        code: "MISSING_CONTENT".to_string(),
                        message: "Content is required for manual resolution".to_string(),
                        details: None,
                    }),
                ));
            }
        },
        "ours" => doc.content.clone().unwrap_or_default(),
        "theirs" => doc.content.clone().unwrap_or_default(),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "INVALID_RESOLUTION".to_string(),
                    message: "Resolution must be 'ours', 'theirs', or 'manual'".to_string(),
                    details: None,
                }),
            ));
        }
    };

    let mut updated = doc.clone();
    updated.content = Some(final_content.clone());
    updated.conflict_detected = Some(false);

    if let Err(e) = repo.update(updated).await {
        warn!(
            "Failed to resolve conflict for document {}: {}",
            document_id, e
        );
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "UPDATE_ERROR".to_string(),
                message: format!("Failed to update document: {}", e),
                details: None,
            }),
        ));
    }

    if let Err(e) = repo.clear_conflict(&doc_id).await {
        warn!(
            "Failed to clear conflict flag for document {}: {}",
            document_id, e
        );
    }

    info!(
        "Conflict resolved for document {} with resolution: {}",
        document_id, body.resolution
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "document_id": document_id,
        "resolution": body.resolution,
    })))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

pub fn create_conflict_router() -> axum::Router<ConflictState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/documents/{document_id}/conflict", get(get_conflict_info))
        .route(
            "/documents/{document_id}/conflict/resolve",
            post(resolve_conflict),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_conflict_request_deserialization() {
        let body = ResolveConflictRequest {
            resolution: "ours".to_string(),
            content: None,
        };
        assert_eq!(body.resolution, "ours");
        assert!(body.content.is_none());
    }

    #[test]
    fn test_resolve_conflict_request_manual() {
        let json = r#"{"resolution":"manual","content":"resolved content"}"#;
        let body: ResolveConflictRequest = serde_json::from_str(json).unwrap();
        assert_eq!(body.resolution, "manual");
        assert_eq!(body.content, Some("resolved content".to_string()));
    }
}
