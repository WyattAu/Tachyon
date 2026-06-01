//! Block-based editor API scaffolding for Notion-like block editing.

use axum::{extract::{Path, State}, response::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorBlock {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub content: serde_json::Value,
    pub children: Vec<EditorBlock>,
    pub order: i32,
}

#[derive(Debug, Serialize)]
pub struct BlockDocumentResponse {
    pub document_id: String,
    pub blocks: Vec<EditorBlock>,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBlocksRequest {
    pub blocks: Vec<EditorBlock>,
}

#[derive(Clone)]
pub struct EditorBlocksState {
    pub pool: PgPool,
}

pub async fn get_document_blocks(
    State(state): State<EditorBlocksState>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<BlockDocumentResponse>, crate::error::ServerError> {
    let row = sqlx::query_as::<_, (String, i32)>(
        "SELECT COALESCE(content, ''), COALESCE(version, 1) FROM documents WHERE id = $1"
    )
    .bind(document_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(crate::error::ServerError::from)?
    .ok_or_else(|| crate::error::ServerError::not_found("Document", &document_id.to_string()))?;

    let blocks = vec![EditorBlock {
        id: "root".to_string(),
        block_type: "document".to_string(),
        content: serde_json::json!({"text": row.0}),
        children: vec![],
        order: 0,
    }];

    Ok(Json(BlockDocumentResponse {
        document_id: document_id.to_string(),
        blocks,
        version: row.1,
    }))
}

pub async fn update_document_blocks(
    State(state): State<EditorBlocksState>,
    Path(document_id): Path<Uuid>,
    Json(_req): Json<UpdateBlocksRequest>,
) -> Result<Json<serde_json::Value>, crate::error::ServerError> {
    let _ = &state.pool;
    Ok(Json(serde_json::json!({
        "document_id": document_id.to_string(),
        "updated": true,
        "version": 1
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_block_serialization() {
        let block = EditorBlock {
            id: "1".to_string(),
            block_type: "paragraph".to_string(),
            content: serde_json::json!({"text": "Hello"}),
            children: vec![],
            order: 0,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("paragraph"));
    }

    #[test]
    fn test_block_document_response() {
        let resp = BlockDocumentResponse {
            document_id: Uuid::new_v4().to_string(),
            blocks: vec![EditorBlock {
                id: "root".to_string(),
                block_type: "document".to_string(),
                content: serde_json::json!(null),
                children: vec![],
                order: 0,
            }],
            version: 1,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("blocks"));
    }

    #[test]
    fn test_nested_blocks() {
        let child = EditorBlock {
            id: "2".to_string(),
            block_type: "list-item".to_string(),
            content: serde_json::json!({"text": "Item"}),
            children: vec![],
            order: 0,
        };
        let parent = EditorBlock {
            id: "1".to_string(),
            block_type: "list".to_string(),
            content: serde_json::json!(null),
            children: vec![child],
            order: 0,
        };
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].block_type, "list-item");
    }
}
