use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_core::DocumentId;
use tracing::{debug, warn};

use super::{
    DocumentQuery, DocumentResponse, DocumentSearchResponse, DocumentState, ErrorResponse,
};

pub async fn search_documents(
    Query(query): Query<DocumentQuery>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("Searching documents: {:?}", query.search);

    let search_query = match query.search {
        Some(ref q) if !q.is_empty() => q,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "MISSING_QUERY".to_string(),
                    message: "Search query is required".to_string(),
                    details: None,
                }),
            ));
        }
    };

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    match state
        .repository
        .search(search_query, Some(page_size as i64))
        .await
    {
        Ok(document_ids) => {
            let doc_ids: Vec<DocumentId> = document_ids
                .iter()
                .filter_map(|id| DocumentId::parse_str(id).ok())
                .collect();

            let mut results = Vec::new();
            if let Ok(docs) = state.repository.get_by_ids_batch(&doc_ids).await {
                for metadata in docs {
                    let tags = metadata.parse_tags().unwrap_or_default();
                    results.push(DocumentResponse {
                        id: metadata.id,
                        title: metadata.title,
                        slug: metadata.slug,
                        html: None,
                        content: String::new(),
                        status: metadata.status,
                        visibility: metadata.visibility,
                        tags,
                        author_id: metadata.author_id,
                        repository_id: metadata.project_id,
                        word_count: metadata.word_count as usize,
                        character_count: metadata.character_count as usize,
                        created_at: metadata.created_at.to_rfc3339(),
                        updated_at: metadata.updated_at.to_rfc3339(),
                        published_at: metadata.published_at.map(|t| t.to_rfc3339()),
                    });
                }
            }

            let total = results.len();

            Ok(Json(DocumentSearchResponse {
                results,
                total,
                page,
                page_size,
            }))
        }
        Err(e) => {
            warn!("Search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SEARCH_ERROR".to_string(),
                    message: format!("Search failed: {}", e),
                    details: None,
                }),
            ))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BacklinksResponse {
    pub backlinks: Vec<BacklinkItem>,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BacklinkItem {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub updated_at: String,
}

pub async fn get_backlinks(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<BacklinksResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting backlinks for document: {}", document_id);

    let doc_id = DocumentId::parse_str(&document_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "INVALID_ID".to_string(),
                message: format!("Invalid document ID: {}", e),
                details: None,
            }),
        )
    })?;

    let metadata = state.repository.get_by_id(&doc_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Document {} not found: {}", document_id, e),
                details: None,
            }),
        )
    })?;

    let search_json = serde_json::json!([metadata.title]);
    let mut conn = state.pool.acquire().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "QUERY_ERROR".to_string(),
                message: format!("Failed to query backlinks: {}", e),
                details: None,
            }),
        )
    })?;
    let rows: Vec<(String, String, Option<String>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, title, slug, updated_at FROM documents WHERE outgoing_links @> $1::jsonb AND id != $2 ORDER BY updated_at DESC LIMIT 50"
    )
    .bind(&search_json)
    .bind(&document_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "QUERY_ERROR".to_string(),
                message: format!("Failed to query backlinks: {}", e),
                details: None,
            }),
        )
    })?;

    let backlinks: Vec<BacklinkItem> = rows
        .into_iter()
        .map(|(id, title, slug, updated_at)| BacklinkItem {
            id,
            title,
            slug: slug.unwrap_or_default(),
            updated_at: updated_at.to_string(),
        })
        .collect();

    let count = backlinks.len();

    Ok(Json(BacklinksResponse { backlinks, count }))
}
