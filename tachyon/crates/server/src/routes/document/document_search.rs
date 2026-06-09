use crate::error::ServerError;
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_core::DocumentId;
use tracing::{debug, warn};

use super::{DocumentQuery, DocumentResponse, DocumentSearchResponse, DocumentState};

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SemanticSearchParams {
    /// The query text to embed and search with.
    pub q: String,
    /// Maximum number of results (1-100, default 20).
    pub limit: Option<i64>,
    /// Minimum cosine similarity threshold (0.0-1.0, default 0.5).
    pub threshold: Option<f32>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SemanticSearchResponse {
    pub results: Vec<DocumentResponse>,
    pub query: String,
    pub limit: i64,
    pub threshold: f32,
}

/// Search documents by semantic similarity using pgvector.
///
/// `GET /api/v1/documents/semantic-search`
///
/// Embeds the query using the configured AI provider and searches the
/// `embedding` column via cosine distance. Requires AI to be configured
/// and the `pgvector` extension to be installed.
#[utoipa::path(
    get,
    path = "/api/v1/documents/semantic-search",
    params(SemanticSearchParams),
    responses(
        (status = 200, description = "Semantic search results", body = SemanticSearchResponse),
        (status = 400, description = "Missing or empty query"),
        (status = 503, description = "AI not configured"),
    ),
    tag = "documents",
)]
pub async fn semantic_search(
    Query(params): Query<SemanticSearchParams>,
    State(state): State<DocumentState>,
) -> Result<Json<SemanticSearchResponse>, ServerError> {
    let query = params.q.trim();
    if query.is_empty() {
        return Err(ServerError::bad_request("Query parameter 'q' is required"));
    }

    let ai = state
        .ai_manager
        .as_ref()
        .ok_or_else(|| ServerError::internal("AI provider not configured"))?;

    if !ai.is_available() {
        return Err(ServerError::internal("AI provider not available"));
    }

    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let threshold = params.threshold.unwrap_or(0.5).clamp(0.0, 1.0);

    let embedding = ai
        .embed(query)
        .await
        .map_err(|e| ServerError::internal(format!("Embedding generation failed: {}", e)))?;

    let documents = state
        .repository
        .search_semantic(embedding, limit, threshold)
        .await
        .map_err(|e| ServerError::database(format!("Semantic search failed: {}", e)))?;

    let results: Vec<DocumentResponse> = documents
        .into_iter()
        .map(|metadata| {
            let tags = metadata.parse_tags().unwrap_or_default();
            DocumentResponse {
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
            }
        })
        .collect();

    Ok(Json(SemanticSearchResponse {
        query: query.to_string(),
        limit,
        threshold,
        results,
    }))
}

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
    /// Short excerpt from the linking document (first 200 chars of content).
    pub excerpt: Option<String>,
    /// The surrounding text (up to 80 chars) where the wikilink appears in the linking document.
    pub link_context: Option<String>,
}

/// Get documents that link to the given document.
///
/// `GET /api/v1/documents/{document_id}/backlinks`
///
/// Queries the `outgoing_links` JSONB column to find all documents referencing
/// this document by title. Returns up to 50 backlinks ordered by update time,
/// each with a short excerpt and link context showing where the link appears.
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
    type BacklinkRow = (
        String,
        String,
        Option<String>,
        Option<String>,
        chrono::DateTime<chrono::Utc>,
    );
    let rows: Vec<BacklinkRow> = sqlx::query_as(
        "SELECT id, title, slug, content, updated_at FROM documents WHERE outgoing_links @> $1::jsonb AND id != $2 ORDER BY updated_at DESC LIMIT 50"
    )
    .bind(&search_json)
    .bind(&document_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| {
        ServerError::database(format!("Failed to query backlinks: {}", e))
    })?;

    let link_pattern = format!("[[{}]]", metadata.title);
    let backlinks: Vec<BacklinkItem> = rows
        .into_iter()
        .map(|(id, title, slug, content, updated_at)| {
            let excerpt = content.as_ref().map(|c| {
                let trimmed = c.trim();
                if trimmed.len() > 200 {
                    format!("{}...", &trimmed[..200])
                } else {
                    trimmed.to_string()
                }
            });

            let link_context = content
                .as_ref()
                .and_then(|c| find_link_context(c, &link_pattern));

            BacklinkItem {
                id,
                title,
                slug: slug.unwrap_or_default(),
                updated_at: updated_at.to_rfc3339(),
                excerpt,
                link_context,
            }
        })
        .collect();

    let count = backlinks.len();

    Ok(Json(BacklinksResponse { backlinks, count }))
}

/// Find the surrounding text where a link pattern appears in content.
/// Returns up to 80 characters of context centered around the match.
fn find_link_context(content: &str, pattern: &str) -> Option<String> {
    let lower_content = content.to_lowercase();
    let lower_pattern = pattern.to_lowercase();
    let pos = lower_content.find(&lower_pattern)?;

    let start = pos.saturating_sub(40);
    let end = (pos + pattern.len() + 40).min(content.len());

    let mut ctx_start = start;
    let mut ctx_end = end;

    // Try to break at word boundaries
    if ctx_start > 0 {
        while ctx_start < ctx_end && !content.is_char_boundary(ctx_start) {
            ctx_start += 1;
        }
        // Move to next word boundary
        while ctx_start < ctx_end
            && !content[ctx_start..].starts_with(char::is_whitespace)
            && !content[ctx_start..].starts_with([',', '.', '!'])
        {
            ctx_start += 1;
        }
        // Skip whitespace
        while ctx_start < ctx_end && content[ctx_start..].starts_with(char::is_whitespace) {
            ctx_start += 1;
        }
    }

    if ctx_end < content.len() {
        while ctx_end > ctx_start && !content.is_char_boundary(ctx_end) {
            ctx_end -= 1;
        }
        // Move to previous word boundary
        while ctx_end > ctx_start
            && !content[..ctx_end].ends_with(char::is_whitespace)
            && !content[..ctx_end].ends_with([',', '.', '!'])
        {
            ctx_end -= 1;
        }
    }

    let ctx = content[ctx_start..ctx_end].trim().to_string();
    if ctx.is_empty() { None } else { Some(ctx) }
}
