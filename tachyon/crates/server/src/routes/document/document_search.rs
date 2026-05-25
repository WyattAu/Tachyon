use crate::error::ServerError;
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_core::DocumentId;
use tracing::{debug, warn};

use super::{DocumentQuery, DocumentResponse, DocumentSearchResponse, DocumentState};

/// Search documents by full-text query.
///
/// `GET /api/v1/documents/search`
///
/// Requires a non-empty `search` query parameter. Supports `page` and `page_size` pagination.
#[utoipa::path(
    get,
    path = "/api/v1/documents/search",
    params(
        crate::routes::document::DocumentQuery,
    ),
    responses(
        (status = 200, description = "Search results", body = DocumentSearchResponse),
        (status = 400, description = "Search query is required"),
        (status = 500, description = "Search failed"),
    ),
    tag = "documents",
)]
pub async fn search_documents(
    Query(query): Query<DocumentQuery>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentSearchResponse>, ServerError> {
    tracing::info!("Searching documents: {:?}", query.search);

    let search_query = match query.search {
        Some(ref q) if !q.is_empty() => q,
        _ => {
            return Err(ServerError::bad_request("Search query is required"));
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
            Err(ServerError::Search(format!("Search failed: {}", e)))
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BacklinksResponse {
    pub backlinks: Vec<BacklinkItem>,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BacklinkItem {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub updated_at: String,
}

/// Get documents that link to the given document.
///
/// `GET /api/v1/documents/{document_id}/backlinks`
///
/// Queries the `outgoing_links` JSONB column to find all documents referencing
/// this document by title. Returns up to 50 backlinks ordered by update time.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/backlinks",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "Backlinks found", body = BacklinksResponse),
        (status = 400, description = "Invalid document ID"),
        (status = 404, description = "Document not found"),
    ),
    tag = "documents",
)]
pub async fn get_backlinks(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<BacklinksResponse>, ServerError> {
    debug!("Getting backlinks for document: {}", document_id);

    let doc_id = DocumentId::parse_str(&document_id)
        .map_err(|e| ServerError::bad_request(format!("Invalid document ID: {}", e)))?;

    let metadata = state
        .repository
        .get_by_id(&doc_id)
        .await
        .map_err(|e| ServerError::not_found("Document", &format!("{}: {}", document_id, e)))?;

    let search_json = serde_json::json!([metadata.title]);
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(format!("Failed to query backlinks: {}", e)))?;
    let rows: Vec<(String, String, Option<String>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, title, slug, updated_at FROM documents WHERE outgoing_links @> $1::jsonb AND id != $2 ORDER BY updated_at DESC LIMIT 50"
    )
    .bind(&search_json)
    .bind(&document_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| {
        ServerError::database(format!("Failed to query backlinks: {}", e))
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
