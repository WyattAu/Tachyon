use crate::error::ServerError;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::DocumentState;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchOperation {
    pub action: BatchAction,
    pub document_id: String,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BatchAction {
    Archive,
    Publish,
    Delete,
    UpdateTags,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchRequest {
    pub operations: Vec<BatchOperation>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BatchOperationResult {
    pub document_id: String,
    pub status: BatchOperationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BatchOperationStatus {
    Ok,
    Error,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BatchResponse {
    pub results: Vec<BatchOperationResult>,
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/batch",
    request_body = BatchRequest,
    responses(
        (status = 200, description = "Batch operation results", body = BatchResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "documents",
)]
pub async fn batch_operations(
    State(state): State<DocumentState>,
    Json(req): Json<BatchRequest>,
) -> Result<(StatusCode, Json<BatchResponse>), ServerError> {
    if req.operations.len() > 100 {
        return Err(ServerError::bad_request(
            "Batch operation limit exceeded: maximum 100 operations per request",
        ));
    }

    debug!("Processing batch of {} operations", req.operations.len());

    let mut results = Vec::with_capacity(req.operations.len());

    for op in req.operations {
        let result = execute_batch_operation(&state, op).await;
        results.push(result);
    }

    debug!(
        "Batch complete: {} ok, {} errors",
        results
            .iter()
            .filter(|r| matches!(r.status, BatchOperationStatus::Ok))
            .count(),
        results
            .iter()
            .filter(|r| matches!(r.status, BatchOperationStatus::Error))
            .count(),
    );

    state.api_cache.invalidate_documents().await;

    Ok((StatusCode::OK, Json(BatchResponse { results })))
}

async fn execute_batch_operation(
    state: &DocumentState,
    op: BatchOperation,
) -> BatchOperationResult {
    let doc_id = match tachyon_core::DocumentId::parse_str(&op.document_id) {
        Ok(id) => id,
        Err(e) => {
            return BatchOperationResult {
                document_id: op.document_id,
                status: BatchOperationStatus::Error,
                error: Some(format!("Invalid document ID: {}", e)),
            };
        }
    };

    let mut metadata = match state.repository.get_by_id(&doc_id).await {
        Ok(m) => m,
        Err(e) => {
            return BatchOperationResult {
                document_id: op.document_id,
                status: BatchOperationStatus::Error,
                error: Some(format!("not found: {}", e)),
            };
        }
    };

    match op.action {
        BatchAction::Archive => {
            metadata.status = "archived".to_string();
            metadata.updated_at = chrono::Utc::now();
            if let Err(e) = state.repository.update(metadata).await {
                warn!("Batch archive failed for {}: {}", op.document_id, e);
                return BatchOperationResult {
                    document_id: op.document_id,
                    status: BatchOperationStatus::Error,
                    error: Some(format!("Archive failed: {}", e)),
                };
            }
        }
        BatchAction::Publish => {
            metadata.status = "published".to_string();
            metadata.updated_at = chrono::Utc::now();
            metadata.published_at = Some(chrono::Utc::now());
            if let Err(e) = state.repository.update(metadata).await {
                warn!("Batch publish failed for {}: {}", op.document_id, e);
                return BatchOperationResult {
                    document_id: op.document_id,
                    status: BatchOperationStatus::Error,
                    error: Some(format!("Publish failed: {}", e)),
                };
            }
        }
        BatchAction::Delete => {
            if let Err(e) = state.repository.delete(&doc_id).await {
                warn!("Batch delete failed for {}: {}", op.document_id, e);
                return BatchOperationResult {
                    document_id: op.document_id,
                    status: BatchOperationStatus::Error,
                    error: Some(format!("Delete failed: {}", e)),
                };
            }
            state.delete_from_tantivy(&op.document_id).await;
        }
        BatchAction::UpdateTags => {
            let tags = op.tags.unwrap_or_default();
            metadata.tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            metadata.updated_at = chrono::Utc::now();
            if let Err(e) = state.repository.update(metadata).await {
                warn!("Batch update_tags failed for {}: {}", op.document_id, e);
                return BatchOperationResult {
                    document_id: op.document_id,
                    status: BatchOperationStatus::Error,
                    error: Some(format!("Update tags failed: {}", e)),
                };
            }
        }
    }

    BatchOperationResult {
        document_id: op.document_id,
        status: BatchOperationStatus::Ok,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_request_deserialization() {
        let json = r#"{
            "operations": [
                {"action": "archive", "document_id": "doc-1"},
                {"action": "delete", "document_id": "doc-2"},
                {"action": "update_tags", "document_id": "doc-3", "tags": ["rust", "async"]}
            ]
        }"#;
        let req: BatchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.operations.len(), 3);
        assert!(matches!(req.operations[0].action, BatchAction::Archive));
        assert!(matches!(req.operations[1].action, BatchAction::Delete));
        assert!(matches!(req.operations[2].action, BatchAction::UpdateTags));
        assert_eq!(
            req.operations[2].tags.as_ref().unwrap(),
            &vec!["rust".to_string(), "async".to_string()]
        );
    }

    #[test]
    fn test_batch_response_serialization() {
        let response = BatchResponse {
            results: vec![
                BatchOperationResult {
                    document_id: "doc-1".to_string(),
                    status: BatchOperationStatus::Ok,
                    error: None,
                },
                BatchOperationResult {
                    document_id: "doc-2".to_string(),
                    status: BatchOperationStatus::Error,
                    error: Some("not found".to_string()),
                },
            ],
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["results"].as_array().unwrap().len(), 2);
        assert_eq!(json["results"][0]["document_id"], "doc-1");
        assert_eq!(json["results"][0]["status"], "ok");
        assert_eq!(json["results"][1]["status"], "error");
        assert_eq!(json["results"][1]["error"], "not found");
    }

    #[test]
    fn test_batch_action_serialization() {
        let action = BatchAction::Archive;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"archive\"");

        let action: BatchAction = serde_json::from_str("\"publish\"").unwrap();
        assert!(matches!(action, BatchAction::Publish));

        let action: BatchAction = serde_json::from_str("\"delete\"").unwrap();
        assert!(matches!(action, BatchAction::Delete));

        let action: BatchAction = serde_json::from_str("\"update_tags\"").unwrap();
        assert!(matches!(action, BatchAction::UpdateTags));
    }

    #[test]
    fn test_empty_batch_request() {
        let json = r#"{"operations": []}"#;
        let req: BatchRequest = serde_json::from_str(json).unwrap();
        assert!(req.operations.is_empty());
    }

    #[test]
    fn test_batch_operation_status_serialization() {
        let ok_status = BatchOperationStatus::Ok;
        let json = serde_json::to_string(&ok_status).unwrap();
        assert_eq!(json, "\"ok\"");

        let error_status = BatchOperationStatus::Error;
        let json = serde_json::to_string(&error_status).unwrap();
        assert_eq!(json, "\"error\"");
    }
}
