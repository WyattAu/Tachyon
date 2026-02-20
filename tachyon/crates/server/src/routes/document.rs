// Document API routes
// Handles document CRUD operations and search

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tachyon_core::{Document, DocumentContent, DocumentId, DocumentStatus, DocumentVisibility};
use tachyon_database::{DatabasePool, DocumentRepository};
use tachyon_renderer::{RenderConfig, Renderer};
use tracing::{debug, info, warn};

/// Application state for document routes
#[derive(Clone)]
pub struct DocumentState {
    /// Database pool
    pub pool: DatabasePool,
    /// Document repository
    pub repository: DocumentRepository,
}

impl DocumentState {
    /// Create a new document state
    pub fn new(pool: DatabasePool) -> Self {
        let repository = DocumentRepository::new(pool.clone());
        Self { pool, repository }
    }
}

/// Query parameters for document listing
#[derive(Debug, Deserialize)]
pub struct DocumentQuery {
    /// Page number (1-indexed)
    pub page: Option<usize>,
    /// Page size
    pub page_size: Option<usize>,
    /// Search query
    pub search: Option<String>,
    /// Repository ID filter
    pub repository_id: Option<String>,
    /// Author ID filter
    pub author_id: Option<String>,
}

/// Request to create a document
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    /// Document title
    pub title: String,
    /// Document content (markdown)
    pub content: String,
    /// Repository ID
    pub repository_id: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Visibility (public, private, restricted)
    #[serde(default)]
    pub visibility: Option<String>,
}

/// Request to update a document
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    /// Document title
    pub title: Option<String>,
    /// Document content (markdown)
    pub content: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Visibility
    pub visibility: Option<String>,
    /// Status
    pub status: Option<String>,
}

/// Document response
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Document slug
    pub slug: Option<String>,
    /// Rendered HTML content
    pub html: Option<String>,
    /// Raw markdown content
    pub content: String,
    /// Document status
    pub status: String,
    /// Document visibility
    pub visibility: String,
    /// Tags
    pub tags: Vec<String>,
    /// Author ID
    pub author_id: String,
    /// Repository ID
    pub repository_id: Option<String>,
    /// Word count
    pub word_count: usize,
    /// Character count
    pub character_count: usize,
    /// Created at timestamp
    pub created_at: String,
    /// Updated at timestamp
    pub updated_at: String,
    /// Published at timestamp
    pub published_at: Option<String>,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        let (content, html) = match &doc.content {
            DocumentContent::Markdown { content } => {
                // Render markdown to HTML
                let html = None; // Will be rendered on demand
                (content.clone(), html)
            }
            DocumentContent::Text { content } => (content.clone(), None),
            DocumentContent::Binary { .. } => ("[Binary content]".to_string(), None),
        };

        // Convert status to string
        let status = match doc.status {
            DocumentStatus::Draft => "draft",
            DocumentStatus::Published => "published",
            DocumentStatus::Archived => "archived",
            DocumentStatus::Deleted => "deleted",
        };

        // Convert visibility to string
        let visibility = match doc.visibility {
            DocumentVisibility::Public => "public",
            DocumentVisibility::Private => "private",
            DocumentVisibility::Restricted => "restricted",
        };

        Self {
            id: doc.id.to_string(),
            title: doc.metadata.title,
            slug: doc.metadata.slug,
            html,
            content,
            status: status.to_string(),
            visibility: visibility.to_string(),
            tags: doc.metadata.tags,
            author_id: doc.metadata.author_id.to_string(),
            repository_id: doc.repository_id.map(|id| id.to_string()),
            word_count: doc.stats.word_count,
            character_count: doc.stats.character_count,
            created_at: doc.metadata.created_at.to_rfc3339(),
            updated_at: doc.metadata.updated_at.to_rfc3339(),
            published_at: doc.metadata.published_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// Document search response
#[derive(Debug, Serialize)]
pub struct DocumentSearchResponse {
    /// Search results
    pub results: Vec<DocumentResponse>,
    /// Total count
    pub total: usize,
    /// Page number
    pub page: usize,
    /// Page size
    pub page_size: usize,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
}

/// Create a new document
pub async fn create_document(
    State(state): State<DocumentState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating new document: {}", req.title);

    // Validate title
    if req.title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Title cannot be empty".to_string(),
                details: None,
            }),
        ));
    }

    if req.title.len() > 200 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Title must be 200 characters or less".to_string(),
                details: None,
            }),
        ));
    }

    // Create document
    let doc_id = tachyon_core::generate_document_id();
    let author_id = tachyon_core::generate_user_id(); // TODO: Get from auth context

    let content = DocumentContent::markdown(req.content.clone());
    let mut doc = Document::new(doc_id.clone(), req.title.clone(), author_id.clone(), content);

    // Set visibility
    if let Some(ref vis) = req.visibility {
        let visibility = match vis.to_lowercase().as_str() {
            "public" => DocumentVisibility::Public,
            "restricted" => DocumentVisibility::Restricted,
            _ => DocumentVisibility::Private,
        };
        doc.visibility = visibility;
    }

    // Add tags
    for tag in &req.tags {
        if let Err(e) = doc.metadata.add_tag(tag.clone()) {
            warn!("Failed to add tag: {}", e);
        }
    }

    // Set repository ID
    if let Some(ref repo_id) = req.repository_id {
        if let Ok(id) = tachyon_core::id::RepositoryId::parse_str(repo_id) {
            doc.repository_id = Some(id);
        }
    }

    // Validate document
    if let Err(e) = doc.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: e.to_string(),
                details: None,
            }),
        ));
    }

    // Create database metadata
    let status_str = match doc.status {
        DocumentStatus::Draft => "draft",
        DocumentStatus::Published => "published",
        DocumentStatus::Archived => "archived",
        DocumentStatus::Deleted => "deleted",
    };
    
    let visibility_str = match doc.visibility {
        DocumentVisibility::Public => "public",
        DocumentVisibility::Private => "private",
        DocumentVisibility::Restricted => "restricted",
    };

    let metadata = tachyon_database::DocumentMetadata {
        id: doc.id.to_string(),
        title: doc.metadata.title.clone(),
        slug: doc.metadata.slug.clone(),
        author_id: doc.metadata.author_id.to_string(),
        description: None,
        tags: serde_json::to_string(&doc.metadata.tags).unwrap_or_else(|_| "[]".to_string()),
        frontmatter: None,
        repository_id: doc.repository_id.map(|id| id.to_string()),
        visibility: visibility_str.to_string(),
        status: status_str.to_string(),
        content_type: "markdown".to_string(),
        word_count: doc.stats.word_count as i64,
        character_count: doc.stats.character_count as i64,
        read_count: 0,
        edit_count: 1,
        created_at: doc.metadata.created_at,
        updated_at: doc.metadata.updated_at,
        published_at: doc.metadata.published_at,
    };

    // Persist to database
    if let Err(e) = state.repository.create(metadata).await {
        warn!("Failed to persist document: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to create document: {}", e),
                details: None,
            }),
        ));
    }

    // Render markdown to HTML (create renderer on demand due to katex thread-safety issues)
    // Note: Renderer is not Send, so we create it after all await points
    let html_content = {
        let renderer = Renderer::new(RenderConfig::default());
        renderer
            .render(doc.content.as_text().unwrap_or(""), None)
            .map(|r| r.content)
            .unwrap_or_default()
    };

    // Create response
    let mut response = DocumentResponse::from(doc);
    response.html = Some(html_content);

    info!("Document created successfully: {}", response.id);

    Ok(Json(response))
}

/// Get a document by ID
pub async fn get_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting document: {}", document_id);

    // Parse document ID
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

    // Try to get from database
    match state.repository.get_by_id(&doc_id).await {
        Ok(_metadata) => {
            // TODO: Fetch full document content from storage
            // For now, return a placeholder
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Document {} not found", document_id),
                    details: None,
                }),
            ))
        }
        Err(e) => {
            warn!("Failed to get document: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Document {} not found", document_id),
                    details: None,
                }),
            ))
        }
    }
}

/// Update a document
pub async fn update_document(
    Path(document_id): Path<String>,
    State(_state): State<DocumentState>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Updating document: {}", document_id);

    // Parse document ID
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

    // For now, return a mock updated document
    // In a real implementation, we would fetch from storage, update, and save
    let author_id = tachyon_core::generate_user_id();
    let content = DocumentContent::markdown(req.content.unwrap_or_default());
    let mut doc = Document::new(
        doc_id,
        req.title.unwrap_or_else(|| "Untitled".to_string()),
        author_id,
        content,
    );

    // Update visibility if provided
    if let Some(vis) = req.visibility {
        let visibility = match vis.to_lowercase().as_str() {
            "public" => DocumentVisibility::Public,
            "restricted" => DocumentVisibility::Restricted,
            _ => DocumentVisibility::Private,
        };
        doc.visibility = visibility;
    }

    // Update status if provided
    if let Some(status) = req.status {
        match status.to_lowercase().as_str() {
            "published" => {
                let _ = doc.publish();
            }
            "archived" => {
                let _ = doc.archive();
            }
            _ => {}
        }
    }

    // Add tags
    if let Some(tags) = req.tags {
        doc.metadata.tags = tags;
    }

    // Render markdown to HTML (create renderer on demand due to katex thread-safety issues)
    let renderer = Renderer::new(RenderConfig::default());
    let render_result = renderer
        .render(doc.content.as_text().unwrap_or(""), None)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "RENDER_ERROR".to_string(),
                    message: format!("Failed to render document: {}", e),
                    details: None,
                }),
            )
        })?;

    let mut response = DocumentResponse::from(doc);
    response.html = Some(render_result.content);

    info!("Document updated successfully: {}", document_id);

    Ok(Json(response))
}

/// Delete a document
pub async fn delete_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    debug!("Deleting document: {}", document_id);

    // Parse document ID
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

    // Try to delete from database
    match state.repository.delete(&doc_id).await {
        Ok(()) => {
            info!("Document deleted: {}", document_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            warn!("Failed to delete document: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Document {} not found", document_id),
                    details: None,
                }),
            ))
        }
    }
}

/// List documents
pub async fn list_documents(
    Query(query): Query<DocumentQuery>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        "Listing documents (page: {:?}, size: {:?})",
        query.page, query.page_size
    );

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    // Get documents from repository
    let documents = if let Some(author_id) = query.author_id {
        state
            .repository
            .list_by_author(&author_id, Some(page_size as i64), Some(offset as i64))
            .await
    } else if let Some(repo_id_str) = query.repository_id {
        if let Ok(repo_id) = tachyon_core::id::RepositoryId::parse_str(&repo_id_str) {
            state
                .repository
                .list_by_repository(&repo_id, Some(page_size as i64), Some(offset as i64))
                .await
        } else {
            Ok(vec![])
        }
    } else {
        // List all documents
        state
            .repository
            .list_all(Some(page_size as i64), Some(offset as i64))
            .await
    };

    match documents {
        Ok(metas) => {
            // Convert to response format
            // Note: For now, we're returning metadata without full content
            let results: Vec<DocumentResponse> = metas
                .into_iter()
                .map(|m| {
                    let tags = m.parse_tags().unwrap_or_default();
                    DocumentResponse {
                        id: m.id,
                        title: m.title,
                        slug: m.slug,
                        html: None,
                        content: String::new(), // Content would be loaded separately
                        status: m.status,
                        visibility: m.visibility,
                        tags,
                        author_id: m.author_id,
                        repository_id: m.repository_id,
                        word_count: m.word_count as usize,
                        character_count: m.character_count as usize,
                        created_at: m.created_at.to_rfc3339(),
                        updated_at: m.updated_at.to_rfc3339(),
                        published_at: m
                            .published_at
                            .map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
                    }
                })
                .collect();

            let total = results.len();

            Ok(Json(DocumentSearchResponse {
                results,
                total,
                page,
                page_size,
            }))
        }
        Err(e) => {
            warn!("Failed to list documents: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "QUERY_ERROR".to_string(),
                    message: format!("Failed to list documents: {}", e),
                    details: None,
                }),
            ))
        }
    }
}

/// Search documents
pub async fn search_documents(
    Query(query): Query<DocumentQuery>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Searching documents: {:?}", query.search);

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

    // Search documents
    match state
        .repository
        .search(search_query, Some(page_size as i64))
        .await
    {
        Ok(document_ids) => {
            // For each ID, we would fetch the document
            // For now, return placeholder results
            let results: Vec<DocumentResponse> = document_ids
                .into_iter()
                .map(|id: String| DocumentResponse {
                    id,
                    title: "Search Result".to_string(),
                    slug: None,
                    html: None,
                    content: String::new(),
                    status: "published".to_string(),
                    visibility: "private".to_string(),
                    tags: vec![],
                    author_id: "unknown".to_string(),
                    repository_id: None,
                    word_count: 0,
                    character_count: 0,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    published_at: None,
                })
                .collect();

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

/// Get document metadata
pub async fn get_document_metadata(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting document metadata: {}", document_id);

    // Parse document ID
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

    // Get metadata from repository
    match state.repository.get_by_id(&doc_id).await {
        Ok(metadata) => {
            let mut result = HashMap::new();
            result.insert("id".to_string(), metadata.id.clone());
            result.insert("title".to_string(), metadata.title.clone());
            result.insert("status".to_string(), metadata.status.clone());
            result.insert("visibility".to_string(), metadata.visibility.clone());
            result.insert("word_count".to_string(), metadata.word_count.to_string());
            result.insert(
                "character_count".to_string(),
                metadata.character_count.to_string(),
            );
            result.insert("created_at".to_string(), metadata.created_at.to_rfc3339());
            result.insert("updated_at".to_string(), metadata.updated_at.to_rfc3339());
            result.insert(
                "tags".to_string(),
                metadata.parse_tags().unwrap_or_default().join(","),
            );

            Ok(Json(result))
        }
        Err(e) => {
            warn!("Failed to get document metadata: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Document {} not found", document_id),
                    details: None,
                }),
            ))
        }
    }
}

/// Render markdown to HTML
pub async fn render_markdown(
    body: String,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Rendering markdown ({} bytes)", body.len());

    // Create renderer on demand since it's not thread-safe (katex issues)
    let renderer = Renderer::new(RenderConfig::default());

    let result = renderer.render(&body, None).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "RENDER_ERROR".to_string(),
                message: format!("Failed to render markdown: {}", e),
                details: None,
            }),
        )
    })?;

    Ok(Json(serde_json::json!({
        "html": result.content,
        "format": "html",
        "word_count": result.metadata.word_count,
        "character_count": result.metadata.char_count,
        "heading_count": result.metadata.heading_count,
        "code_block_count": result.metadata.code_block_count,
        "render_time_ms": result.stats.render_time_ms,
    })))
}

/// Create the document router (without state - caller must use .with_state())
pub fn create_document_router() -> axum::Router<DocumentState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/documents", get(list_documents))
        .route("/documents", post(create_document))
        .route("/documents/search", get(search_documents))
        .route("/documents/{document_id}", get(get_document))
        .route("/documents/{document_id}", put(update_document))
        .route("/documents/{document_id}", delete(delete_document))
        .route(
            "/documents/{document_id}/metadata",
            get(get_document_metadata),
        )
        .route("/render/markdown", post(render_markdown))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document_request_serialization() {
        // Note: CreateDocumentRequest is Deserialize only (for incoming requests)
        // This test verifies the struct can be constructed properly
        let req = CreateDocumentRequest {
            title: "Test Document".to_string(),
            content: "Test content".to_string(),
            repository_id: None,
            tags: vec![],
            visibility: None,
        };

        assert_eq!(req.title, "Test Document");
        assert_eq!(req.content, "Test content");
    }

    #[test]
    fn test_document_query_serialization() {
        // Note: DocumentQuery is Deserialize only (for query params)
        // This test verifies the struct can be constructed properly
        let query = DocumentQuery {
            page: Some(1),
            page_size: Some(10),
            search: Some("test".to_string()),
            repository_id: None,
            author_id: None,
        };

        assert_eq!(query.page, Some(1));
        assert_eq!(query.search, Some("test".to_string()));
    }
}
