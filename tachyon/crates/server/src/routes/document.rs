// Document API routes
// Handles document CRUD operations and search

use axum::{
    extract::{DefaultBodyLimit, Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::{Json, IntoResponse},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tachyon_core::{compute_content_hash, Document, DocumentContent, DocumentId, DocumentStatus, DocumentVisibility};
use tachyon_core::id::{RepositoryId, UserId};
use tachyon_database::{
    ActivityRepository, AttachmentRepository, CreateActivityEvent, CreateAttachmentRequest,
    CreateTemplateRequest, CreateVersionRequest, DocumentVersionRepository, DatabasePool,
    DocumentRepository, TemplateRepository, UpdateTemplateRequest,
};
use tachyon_renderer::{RenderConfig, Renderer, MarkdownParser};
use tachyon_search::{IndexManager, SearchDocument};
use tracing::{debug, info, warn};
use crate::config::GuestConfig;

/// Application state for document routes
#[derive(Clone)]
pub struct DocumentState {
    pub pool: DatabasePool,
    pub repository: DocumentRepository,
    pub guest_config: GuestConfig,
    pub index_manager: Option<Arc<Mutex<IndexManager>>>,
}

impl DocumentState {
    pub fn new(pool: DatabasePool) -> Self {
        let repository = DocumentRepository::new(pool.clone());
        Self {
            pool,
            repository,
            guest_config: GuestConfig::default(),
            index_manager: None,
        }
    }

    pub fn with_guest_config(pool: DatabasePool, guest_config: GuestConfig) -> Self {
        let repository = DocumentRepository::new(pool.clone());
        Self {
            pool,
            repository,
            guest_config,
            index_manager: None,
        }
    }

    pub fn with_index_manager(mut self, index_manager: Arc<Mutex<IndexManager>>) -> Self {
        self.index_manager = Some(index_manager);
        self
    }

    pub fn is_public_access_enabled(&self) -> bool {
        self.guest_config.public_notes_enabled
    }

    async fn index_in_tantivy(&self, search_doc: SearchDocument) {
        if let Some(ref mgr) = self.index_manager {
            let guard = mgr.lock().await;
            if let Err(e) = guard.index_document(&search_doc).await {
                warn!("Failed to index document in Tantivy: {}", e);
            }
        }
    }

    async fn delete_from_tantivy(&self, doc_id: &str) {
        if let Some(ref mgr) = self.index_manager {
            let guard = mgr.lock().await;
            if let Err(e) = guard.delete_document(doc_id).await {
                warn!("Failed to delete document from Tantivy: {}", e);
            }
        }
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
    /// Project ID filter
    pub project_id: Option<String>,
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
    /// Project ID
    pub project_id: Option<String>,
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
    /// Repository ID (maps to project_id in database)
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
    auth: Option<Extension<crate::middleware::AuthContext>>,
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

    // Get author_id from auth context, falling back to a generated ID for guest access
    let author_id: tachyon_core::id::UserId = auth
        .and_then(|Extension(ctx)| tachyon_core::id::UserId::parse_str(&ctx.user_id).ok())
        .unwrap_or_else(tachyon_core::generate_user_id);

    // Create document
    let doc_id = tachyon_core::generate_document_id();
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

    // Set repository ID from project_id in request
    if let Some(ref repo_id) = req.project_id {
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
        project_id: doc.repository_id.map(|id| id.to_string()),
        visibility: visibility_str.to_string(),
        status: status_str.to_string(),
        content_type: "markdown".to_string(),
        word_count: doc.stats.word_count as i32,
        character_count: doc.stats.character_count as i32,
        read_count: 0,
        edit_count: 1,
        content: Some(doc.content.as_text().unwrap_or("").to_string()),
        html: None,
        created_at: doc.metadata.created_at,
        updated_at: doc.metadata.updated_at,
        published_at: doc.metadata.published_at,
        content_hash: Some(compute_content_hash(doc.content.as_text().unwrap_or(""))),
        conflict_detected: Some(false),
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

    // Extract and store outgoing wikilinks
    {
        let outgoing_links = MarkdownParser::extract_wikilinks(&req.content);
        let links_json = serde_json::to_value(&outgoing_links).unwrap_or(serde_json::json!([]));
        if let Ok(mut conn) = state.pool.acquire().await {
            if let Err(e) = sqlx::query("UPDATE documents SET outgoing_links = $1 WHERE id = $2")
                .bind(&links_json)
                .bind(&doc.id.to_string())
                .execute(&mut *conn)
                .await
            {
                warn!("Failed to update outgoing links for document {}: {}", doc.id, e);
            }
        }
    }

    // Update search index for full-text search
    if let Err(e) = state.repository.update_search_index(
        &doc.id,
        &doc.metadata.title,
        doc.content.as_text().unwrap_or(""),
        &doc.metadata.tags,
    ).await {
        warn!("Failed to update search index for document {}: {}", doc.id, e);
    }

    state.index_in_tantivy(SearchDocument {
        id: doc.id.clone(),
        title: doc.metadata.title.clone(),
        content: doc.content.as_text().unwrap_or("").to_string(),
        author_id: author_id.clone(),
        repository_id: doc.repository_id.clone(),
        tags: doc.metadata.tags.clone(),
        created_at: doc.metadata.created_at,
        updated_at: doc.metadata.updated_at,
        custom_fields: HashMap::new(),
    }).await;

    if let Err(e) = ActivityRepository::create(&state.pool, CreateActivityEvent {
        actor_id: uuid::Uuid::parse_str(&author_id.to_string()).unwrap_or_default(),
        event_type: "document_created".to_string(),
        target_type: "document".to_string(),
        target_id: uuid::Uuid::parse_str(&doc.id.to_string()).unwrap_or_default(),
        description: format!("Created document: {}", req.title),
        metadata: None,
    }).await {
        warn!("Failed to emit activity event for document creation: {}", e);
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

    let _ = crate::webhook_delivery::deliver_event(state.pool.clone(), "document_created", &serde_json::json!({
        "document_id": response.id.clone(),
        "title": response.title.clone(),
    }));

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
        Ok(metadata) => {
            let tags = metadata.parse_tags().unwrap_or_default();
            let response = DocumentResponse {
                id: metadata.id,
                title: metadata.title,
                slug: metadata.slug,
                html: metadata.html,
                content: metadata.content.unwrap_or_default(),
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
            };
            Ok(Json(response))
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
    State(state): State<DocumentState>,
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

    // Fetch existing document
    let mut metadata = state.repository.get_by_id(&doc_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Document {} not found: {}", document_id, e),
                details: None,
            }),
        )
    })?;

    // Apply updates
    if let Some(title) = req.title {
        metadata.title = title;
    }
    if let Some(content) = req.content {
        // Auto-version: snapshot current content before overwriting
        if let Some(ref current_content) = metadata.content {
            if current_content != &content {
                let version_repo = DocumentVersionRepository::new(state.pool.clone());
                let user_id = tachyon_core::generate_user_id();
                if let Err(e) = version_repo.create(CreateVersionRequest {
                    document_id: document_id.clone(),
                    content: current_content.clone(),
                    commit_message: Some("Auto-snapshot before update".to_string()),
                    created_by: user_id.to_string(),
                }).await {
                    warn!("Failed to auto-version document {}: {}", document_id, e);
                }
            }
        }

        metadata.content = Some(content.clone());
        metadata.content_hash = Some(compute_content_hash(&content));
        // Render markdown to HTML
        let renderer = Renderer::new(RenderConfig::default());
        match renderer.render(&content, None) {
            Ok(render_result) => {
                metadata.html = Some(render_result.content);
                metadata.word_count = render_result.metadata.word_count as i32;
                metadata.character_count = render_result.metadata.char_count as i32;
            }
            Err(e) => {
                warn!("Failed to render markdown: {}", e);
            }
        }
    }
    if let Some(vis) = req.visibility {
        metadata.visibility = vis;
    }
    if let Some(status) = req.status {
        metadata.status = status;
    }
    if let Some(tags) = req.tags {
        metadata.tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
    }
    metadata.updated_at = chrono::Utc::now();

    // Save to database
    state.repository.update(metadata.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "UPDATE_ERROR".to_string(),
                message: format!("Failed to update document: {}", e),
                details: None,
            }),
        )
    })?;

    // Extract and store outgoing wikilinks
    {
        let content = metadata.content.as_deref().unwrap_or("");
        let outgoing_links = MarkdownParser::extract_wikilinks(content);
        let links_json = serde_json::to_value(&outgoing_links).unwrap_or(serde_json::json!([]));
        if let Ok(mut conn) = state.pool.acquire().await {
            if let Err(e) = sqlx::query("UPDATE documents SET outgoing_links = $1 WHERE id = $2")
                .bind(&links_json)
                .bind(&document_id)
                .execute(&mut *conn)
                .await
            {
                warn!("Failed to update outgoing links for document {}: {}", document_id, e);
            }
        }
    }

    // Update search index for full-text search
    if let Err(e) = state.repository.update_search_index(
        &doc_id,
        &metadata.title,
        metadata.content.as_deref().unwrap_or(""),
        &metadata.parse_tags().unwrap_or_default(),
    ).await {
        warn!("Failed to update search index for document {}: {}", doc_id, e);
    }

    state.index_in_tantivy(SearchDocument {
        id: doc_id.clone(),
        title: metadata.title.clone(),
        content: metadata.content.clone().unwrap_or_default(),
        author_id: UserId::parse_str(&metadata.author_id).unwrap_or_default(),
        repository_id: metadata.project_id.as_ref().and_then(|id| RepositoryId::parse_str(id).ok()),
        tags: metadata.parse_tags().unwrap_or_default(),
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        custom_fields: HashMap::new(),
    }).await;

    if let Err(e) = ActivityRepository::create(&state.pool, CreateActivityEvent {
        actor_id: uuid::Uuid::parse_str(&metadata.author_id).unwrap_or_default(),
        event_type: "document_updated".to_string(),
        target_type: "document".to_string(),
        target_id: uuid::Uuid::parse_str(&document_id).unwrap_or_default(),
        description: format!("Updated document: {}", metadata.title),
        metadata: None,
    }).await {
        warn!("Failed to emit activity event for document update: {}", e);
    }

    let tags = metadata.parse_tags().unwrap_or_default();
    let response = DocumentResponse {
        id: metadata.id,
        title: metadata.title,
        slug: metadata.slug,
        html: metadata.html,
        content: metadata.content.unwrap_or_default(),
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
    };

    info!("Document updated successfully: {}", document_id);

    let _ = crate::webhook_delivery::deliver_event(state.pool.clone(), "document_updated", &serde_json::json!({
        "document_id": document_id.clone(),
        "title": response.title.clone(),
    }));

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
            state.delete_from_tantivy(&document_id).await;
            info!("Document deleted: {}", document_id);

            let _ = crate::webhook_delivery::deliver_event(state.pool.clone(), "document_deleted", &serde_json::json!({
                "document_id": document_id.clone(),
            }));

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
    } else if let Some(project_id_str) = query.project_id {
        state
            .repository
            .list_by_project(&project_id_str, Some(page_size as i64), Some(offset as i64))
            .await
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
                        repository_id: m.project_id, // Map project_id to repository_id for API compatibility
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
            // Fetch actual documents by ID
            let mut results = Vec::new();
            for id in document_ids {
                if let Ok(doc_id) = DocumentId::parse_str(&id) {
                    if let Ok(metadata) = state.repository.get_by_id(&doc_id).await {
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

// ============================================================================
// Document Version Endpoints
// ============================================================================

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub content: String,
    pub commit_message: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

impl From<tachyon_database::DocumentVersion> for VersionResponse {
    fn from(v: tachyon_database::DocumentVersion) -> Self {
        Self {
            id: v.id,
            document_id: v.document_id,
            version_number: v.version_number,
            content: v.content,
            commit_message: v.commit_message,
            created_at: v.created_at.to_rfc3339(),
            created_by: v.created_by,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateVersionBody {
    pub content: String,
    pub commit_message: Option<String>,
}

pub async fn list_versions(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<Vec<VersionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = DocumentVersionRepository::new(state.pool.clone());
    let versions = repo.list_by_document(&document_id, Some(50)).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "QUERY_ERROR".to_string(),
                    message: format!("Failed to list versions: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(Json(versions.into_iter().map(VersionResponse::from).collect()))
}

pub async fn get_version(
    Path((document_id, version_number)): Path<(String, i32)>,
    State(state): State<DocumentState>,
) -> Result<Json<VersionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = DocumentVersionRepository::new(state.pool.clone());
    let version = repo.get_by_version_number(&document_id, version_number).await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Version {} not found: {}", version_number, e),
                    details: None,
                }),
            )
        })?;

    Ok(Json(VersionResponse::from(version)))
}

pub async fn create_version(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
    Json(body): Json<CreateVersionBody>,
) -> Result<Json<VersionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = tachyon_core::generate_user_id();
    let repo = DocumentVersionRepository::new(state.pool.clone());
    
    let version = repo.create(CreateVersionRequest {
        document_id: document_id.clone(),
        content: body.content,
        commit_message: body.commit_message,
        created_by: user_id.to_string(),
    }).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "CREATE_ERROR".to_string(),
                message: format!("Failed to create version: {}", e),
                details: None,
            }),
        )
    })?;

    info!("Created version {} for document {}", version.version_number, document_id);
    Ok(Json(VersionResponse::from(version)))
}

// ============================================================================
// Server-Side Diff Endpoint
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DiffLine {
    pub content: String,
    pub line_type: String, // "unchanged", "added", "removed"
}

#[derive(Debug, Serialize)]
pub struct DocumentDiffResponse {
    pub old_lines: Vec<DiffLine>,
    pub new_lines: Vec<DiffLine>,
    pub stats: DiffStats,
}

#[derive(Debug, Serialize)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

pub async fn diff_versions(
    Path((document_id, v1, v2)): Path<(String, i32, i32)>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentDiffResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = DocumentVersionRepository::new(state.pool.clone());

    let ver1 = repo.get_by_version_number(&document_id, v1).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Version {} not found: {}", v1, e),
                details: None,
            }),
        )
    })?;

    let ver2 = repo.get_by_version_number(&document_id, v2).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Version {} not found: {}", v2, e),
                details: None,
            }),
        )
    })?;

    let diff = compute_line_diff(&ver1.content, &ver2.content);
    Ok(Json(diff))
}

/// Server-side LCS line diff computation
fn compute_line_diff(old_content: &str, new_content: &str) -> DocumentDiffResponse {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let lcs = longest_common_subsequence(&old_lines, &new_lines);

    let mut old_idx = 0usize;
    let mut new_idx = 0usize;
    let mut lcs_idx = 0usize;
    let mut result_old = Vec::new();
    let mut result_new = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut unchanged = 0usize;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if lcs_idx < lcs.len() {
            let lcs_line = lcs[lcs_idx];

            while old_idx < old_lines.len() && old_lines[old_idx] != lcs_line {
                result_old.push(DiffLine { content: old_lines[old_idx].to_string(), line_type: "removed".to_string() });
                result_new.push(DiffLine { content: String::new(), line_type: "unchanged".to_string() });
                removed += 1;
                old_idx += 1;
            }

            while new_idx < new_lines.len() && new_lines[new_idx] != lcs_line {
                result_old.push(DiffLine { content: String::new(), line_type: "unchanged".to_string() });
                result_new.push(DiffLine { content: new_lines[new_idx].to_string(), line_type: "added".to_string() });
                added += 1;
                new_idx += 1;
            }

            if old_idx < old_lines.len() && new_idx < new_lines.len() {
                result_old.push(DiffLine { content: old_lines[old_idx].to_string(), line_type: "unchanged".to_string() });
                result_new.push(DiffLine { content: new_lines[new_idx].to_string(), line_type: "unchanged".to_string() });
                unchanged += 1;
                old_idx += 1;
                new_idx += 1;
                lcs_idx += 1;
            }
        } else {
            while old_idx < old_lines.len() {
                result_old.push(DiffLine { content: old_lines[old_idx].to_string(), line_type: "removed".to_string() });
                result_new.push(DiffLine { content: String::new(), line_type: "unchanged".to_string() });
                removed += 1;
                old_idx += 1;
            }
            while new_idx < new_lines.len() {
                result_old.push(DiffLine { content: String::new(), line_type: "unchanged".to_string() });
                result_new.push(DiffLine { content: new_lines[new_idx].to_string(), line_type: "added".to_string() });
                added += 1;
                new_idx += 1;
            }
        }
    }

    DocumentDiffResponse {
        old_lines: result_old,
        new_lines: result_new,
        stats: DiffStats { added, removed, unchanged },
    }
}

fn longest_common_subsequence<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<&'a str> {
    let m = old.len();
    let n = new.len();
    if m == 0 || n == 0 { return Vec::new(); }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            result.push(old[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

// ============================================================================
// Attachment Endpoints
// ============================================================================

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

pub async fn list_attachments(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<Vec<AttachmentResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = AttachmentRepository::new(state.pool.clone());
    let attachments = repo.list_by_document(&document_id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "QUERY_ERROR".to_string(),
                    message: format!("Failed to list attachments: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(Json(attachments.into_iter().map(AttachmentResponse::from).collect()))
}

pub async fn upload_attachment(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
    mut multipart: Multipart,
) -> Result<Json<AttachmentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = tachyon_core::generate_user_id();
    let repo = AttachmentRepository::new(state.pool.clone());

    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        
        let content = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "UPLOAD_ERROR".to_string(),
                    message: format!("Failed to read file: {}", e),
                    details: None,
                }),
            )
        })?;

        let attachment = repo.create(CreateAttachmentRequest {
            document_id: document_id.clone(),
            filename,
            mime_type,
            content: content.to_vec(),
            created_by: user_id.to_string(),
        }).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "CREATE_ERROR".to_string(),
                    message: format!("Failed to create attachment: {}", e),
                    details: None,
                }),
            )
        })?;

        return Ok(Json(AttachmentResponse::from(attachment)));
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: "NO_FILE".to_string(),
            message: "No file provided".to_string(),
            details: None,
        }),
    ))
}

pub async fn download_attachment(
    Path((_document_id, attachment_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let repo = AttachmentRepository::new(state.pool.clone());
    let (attachment, content) = repo.get_content(&attachment_id).await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Attachment not found: {}", e),
                    details: None,
                }),
            )
        })?;

    let headers = [
        ("Content-Type", attachment.mime_type.clone()),
        ("Content-Disposition", format!("attachment; filename=\"{}\"", attachment.filename)),
    ];

    Ok((headers, content))
}

pub async fn delete_attachment(
    Path((_document_id, attachment_id)): Path<(String, String)>,
    State(state): State<DocumentState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = AttachmentRepository::new(state.pool.clone());
    repo.delete(&attachment_id).await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Attachment not found: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Template Endpoints
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
}

impl From<tachyon_database::DocumentTemplate> for TemplateResponse {
    fn from(t: tachyon_database::DocumentTemplate) -> Self {
        let tags = t.parse_tags().unwrap_or_default();
        Self {
            id: t.id,
            name: t.name,
            description: t.description,
            content: t.content,
            category: t.category,
            tags,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
            created_by: t.created_by,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateBody {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateQuery {
    pub category: Option<String>,
}

pub async fn list_templates(
    Query(query): Query<TemplateQuery>,
    State(state): State<DocumentState>,
) -> Result<Json<Vec<TemplateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = TemplateRepository::new(state.pool.clone());
    let templates = repo.list(query.category.as_deref(), Some(50), None).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "QUERY_ERROR".to_string(),
                    message: format!("Failed to list templates: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(Json(templates.into_iter().map(TemplateResponse::from).collect()))
}

pub async fn get_template(
    Path(template_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = TemplateRepository::new(state.pool.clone());
    let template = repo.get_by_id(&template_id).await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Template not found: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(Json(TemplateResponse::from(template)))
}

pub async fn create_template(
    State(state): State<DocumentState>,
    Json(body): Json<CreateTemplateBody>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = tachyon_core::generate_user_id();
    let repo = TemplateRepository::new(state.pool.clone());
    
    let template = repo.create(CreateTemplateRequest {
        name: body.name,
        description: body.description,
        content: body.content,
        category: body.category,
        tags: body.tags,
        created_by: user_id.to_string(),
    }).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "CREATE_ERROR".to_string(),
                message: format!("Failed to create template: {}", e),
                details: None,
            }),
        )
    })?;

    info!("Created template: {}", template.name);
    Ok(Json(TemplateResponse::from(template)))
}

pub async fn update_template(
    Path(template_id): Path<String>,
    State(state): State<DocumentState>,
    Json(body): Json<UpdateTemplateBody>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = TemplateRepository::new(state.pool.clone());
    
    let template = repo.update(&template_id, UpdateTemplateRequest {
        name: body.name,
        description: body.description,
        content: body.content,
        category: body.category,
        tags: body.tags,
    }).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "UPDATE_ERROR".to_string(),
                message: format!("Failed to update template: {}", e),
                details: None,
            }),
        )
    })?;

    Ok(Json(TemplateResponse::from(template)))
}

pub async fn delete_template(
    Path(template_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = TemplateRepository::new(state.pool.clone());
    repo.delete(&template_id).await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Template not found: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Backlinks Endpoint
// ============================================================================

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
        .route("/documents/{document_id}/versions", get(list_versions))
        .route("/documents/{document_id}/versions", post(create_version))
        .route("/documents/{document_id}/versions/{version_number}", get(get_version))
        .route("/documents/{document_id}/versions/{v1}/diff/{v2}", get(diff_versions))
        .route("/documents/{document_id}/attachments", get(list_attachments))
        .route("/documents/{document_id}/attachments", post(upload_attachment).layer(DefaultBodyLimit::max(50 * 1024 * 1024)))
        .route("/documents/{document_id}/attachments/{attachment_id}", delete(delete_attachment))
        .route("/documents/{document_id}/backlinks", get(get_backlinks))
        .route("/templates", get(list_templates))
        .route("/templates", post(create_template))
        .route("/templates/{template_id}", get(get_template))
        .route("/templates/{template_id}", put(update_template))
        .route("/templates/{template_id}", delete(delete_template))
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
            project_id: None,
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
            project_id: None,
            author_id: None,
        };

        assert_eq!(query.page, Some(1));
        assert_eq!(query.search, Some("test".to_string()));
    }

    #[test]
    fn test_backlink_item_serialization() {
        let item = BacklinkItem {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            title: "Test Doc".to_string(),
            slug: "test-doc".to_string(),
            updated_at: "2026-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Test Doc"));
        assert!(json.contains("test-doc"));
    }

    #[test]
    fn test_backlinks_response_serialization() {
        let response = BacklinksResponse {
            backlinks: vec![BacklinkItem {
                id: "1".to_string(),
                title: "Doc A".to_string(),
                slug: "doc-a".to_string(),
                updated_at: "2026-01-01T00:00:00+00:00".to_string(),
            }],
            count: 1,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"count\":1"));
        assert!(json.contains("Doc A"));
    }
}
