use crate::audit::{AuditEvent, AuditEventType, AuditSeverity};
use crate::error::ServerError;
use crate::pagination::{CursorPage, CursorParams};
use crate::validation::{ValidatedDocumentTitle, ValidatedTagList};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use tachyon_core::id::{RepositoryId, UserId};
use tachyon_core::{
    compute_content_hash, Document, DocumentContent, DocumentId, DocumentStatus, DocumentVisibility,
};
use tachyon_database::{
    ActivityRepository, CreateActivityEvent, CreateVersionRequest, DocumentVersionRepository,
};
use tachyon_renderer::{MarkdownParser, RenderConfig, Renderer};
use tachyon_search::SearchDocument;
use tracing::{debug, info, warn};

use super::{
    DocumentCursorPage, DocumentQuery, DocumentResponse, DocumentSearchResponse, DocumentState,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateDocumentRequest {
    pub title: String,
    #[serde(default)]
    pub content: String,
    pub project_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<String>,
    pub status: Option<String>,
}

/// Create a new document.
///
/// `POST /api/v1/documents`
///
/// Request body: JSON with `title` (required), `content`, `project_id`, `tags`, `visibility`.
/// Response: 201 with the created `DocumentResponse`, or 400/401/500 on error.
#[utoipa::path(
    post,
    path = "/api/v1/documents",
    request_body = CreateDocumentRequest,
    responses(
        (status = 200, description = "Document created", body = DocumentResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal error"),
    ),
    tag = "documents",
)]
pub async fn create_document(
    State(state): State<DocumentState>,
    auth: Option<Extension<crate::middleware::AuthContext>>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, ServerError> {
    info!("Creating new document: {}", req.title);

    let validated_title = ValidatedDocumentTitle::new(&req.title)
        .map_err(|e| ServerError::bad_request(format!("Invalid title: {}", e)))?;

    let validated_tags = if !req.tags.is_empty() {
        Some(
            ValidatedTagList::new(&req.tags)
                .map_err(|e| ServerError::bad_request(format!("Invalid tags: {}", e)))?,
        )
    } else {
        None
    };

    let author_id: tachyon_core::id::UserId = auth
        .and_then(|Extension(ctx)| tachyon_core::id::UserId::parse_str(&ctx.user_id).ok())
        .unwrap_or_else(tachyon_core::generate_user_id);

    let doc_id = tachyon_core::generate_document_id();
    let content = DocumentContent::markdown(req.content.clone());
    let mut doc = Document::new(
        doc_id,
        validated_title.as_str().to_string(),
        author_id,
        content,
    );

    if let Some(ref vis) = req.visibility {
        let visibility = match vis.to_lowercase().as_str() {
            "public" => DocumentVisibility::Public,
            "restricted" => DocumentVisibility::Restricted,
            _ => DocumentVisibility::Private,
        };
        doc.visibility = visibility;
    }

    for tag in validated_tags.iter().flat_map(|vt| vt.iter()) {
        if let Err(e) = doc.metadata.add_tag(tag.as_str().to_string()) {
            warn!("Failed to add tag: {}", e);
        }
    }

    if let Some(ref repo_id) = req.project_id {
        if let Ok(id) = tachyon_core::id::RepositoryId::parse_str(repo_id) {
            doc.repository_id = Some(id);
        }
    }

    if let Err(e) = doc.validate() {
        return Err(ServerError::bad_request(e.to_string()));
    }

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

    if let Err(e) = state.repository.create(metadata).await {
        warn!("Failed to persist document: {}", e);
        return Err(ServerError::database(format!(
            "Failed to create document: {}",
            e
        )));
    }

    {
        let outgoing_links = MarkdownParser::extract_wikilinks(&req.content);
        let links_json = serde_json::to_value(&outgoing_links).unwrap_or(serde_json::json!([]));
        if let Ok(mut conn) = state.pool.acquire().await {
            if let Err(e) = sqlx::query("UPDATE documents SET outgoing_links = $1 WHERE id = $2")
                .bind(&links_json)
                .bind(doc.id.to_string())
                .execute(&mut *conn)
                .await
            {
                warn!(
                    "Failed to update outgoing links for document {}: {}",
                    doc.id, e
                );
            }
        }
    }

    if let Err(e) = state
        .repository
        .update_search_index(
            &doc.id,
            &doc.metadata.title,
            doc.content.as_text().unwrap_or(""),
            &doc.metadata.tags,
        )
        .await
    {
        warn!(
            "Failed to update search index for document {}: {}",
            doc.id, e
        );
    }

    state
        .index_in_tantivy(SearchDocument {
            id: doc.id,
            title: doc.metadata.title.clone(),
            content: doc.content.as_text().unwrap_or("").to_string(),
            author_id,
            repository_id: doc.repository_id,
            tags: doc.metadata.tags.clone(),
            created_at: doc.metadata.created_at,
            updated_at: doc.metadata.updated_at,
            custom_fields: BTreeMap::new(),
        })
        .await;

    // Generate and persist embedding asynchronously (non-blocking)
    if let Some(ref ai) = state.ai_manager {
        if ai.is_available() {
            let ai = ai.clone();
            let repo = state.repository.clone();
            let doc_id = doc.id.to_string();
            let embed_text = format!(
                "{} {}",
                doc.metadata.title,
                doc.content.as_text().unwrap_or("")
            );
            tokio::spawn(async move {
                match ai.embed(&embed_text).await {
                    Ok(embedding) => {
                        if let Err(e) = repo.update_embedding(&doc_id, embedding).await {
                            warn!("Failed to persist embedding for document {}: {}", doc_id, e);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to generate embedding for document {}: {}",
                            doc_id, e
                        );
                    }
                }
            });
        }
    }

    if let Err(e) = ActivityRepository::create(
        &state.pool,
        CreateActivityEvent {
            actor_id: uuid::Uuid::parse_str(&author_id.to_string()).unwrap_or_default(),
            event_type: "document_created".to_string(),
            target_type: "document".to_string(),
            target_id: uuid::Uuid::parse_str(&doc.id.to_string()).unwrap_or_default(),
            description: format!("Created document: {}", req.title),
            metadata: None,
        },
    )
    .await
    {
        warn!("Failed to emit activity event for document creation: {}", e);
    }

    let html_content = {
        let renderer = Renderer::new(RenderConfig::default());
        renderer
            .render(doc.content.as_text().unwrap_or(""), None)
            .map(|r| r.content)
            .unwrap_or_default()
    };

    let mut response = DocumentResponse::from(doc);
    response.html = Some(html_content);

    info!("Document created successfully: {}", response.id);

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::NodeCreated,
                AuditSeverity::Low,
                "document_create",
                format!("Document '{}' created", response.title),
            )
            .with_target(&response.id, "document"),
        )
        .await;

    state.api_cache.invalidate_documents().await;

    {
        let pool = state.pool.clone();
        let client = state.http_client.clone();
        let payload = serde_json::json!({
            "document_id": response.id.clone(),
            "title": response.title.clone(),
        });
        tokio::spawn(async move {
            crate::webhook_delivery::deliver_event(pool, client, "document_created", &payload)
                .await;
        });
    }

    Ok(Json(response))
}

/// Get a document by ID.
///
/// `GET /api/v1/documents/{document_id}`
///
/// Response: 200 with `DocumentResponse`, or 400 (invalid ID) / 404 on error.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "Document found", body = DocumentResponse),
        (status = 404, description = "Document not found"),
    ),
    tag = "documents",
)]
pub async fn get_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentResponse>, ServerError> {
    debug!("Getting document: {}", document_id);

    let doc_id = DocumentId::parse_str(&document_id)
        .map_err(|e| ServerError::bad_request(format!("Invalid document ID: {}", e)))?;

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
            Err(ServerError::not_found("Document", &document_id))
        }
    }
}

/// Update an existing document.
///
/// `PUT /api/v1/documents/{document_id}`
///
/// Request body: JSON with optional `title`, `content`, `tags`, `visibility`, `status`.
/// Automatically creates a version snapshot when content changes.
/// Response: 200 with updated `DocumentResponse`, or 400/404/500 on error.
#[utoipa::path(
    put,
    path = "/api/v1/documents/{document_id}",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    request_body = UpdateDocumentRequest,
    responses(
        (status = 200, description = "Document updated", body = DocumentResponse),
        (status = 404, description = "Document not found"),
    ),
    tag = "documents",
)]
pub async fn update_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentResponse>, ServerError> {
    debug!("Updating document: {}", document_id);

    let doc_id = DocumentId::parse_str(&document_id)
        .map_err(|e| ServerError::bad_request(format!("Invalid document ID: {}", e)))?;

    let mut metadata = state
        .repository
        .get_by_id(&doc_id)
        .await
        .map_err(|e| ServerError::not_found("Document", &format!("{}: {}", document_id, e)))?;

    if let Some(title) = req.title {
        let validated_title = ValidatedDocumentTitle::new(&title)
            .map_err(|e| ServerError::bad_request(format!("Invalid title: {}", e)))?;
        metadata.title = validated_title.as_str().to_string();
    }
    let mut content_changed = false;
    if let Some(content) = req.content {
        content_changed = true;
        if let Some(ref current_content) = metadata.content {
            if current_content != &content {
                let version_repo = DocumentVersionRepository::new(state.pool.clone());
                let user_id = tachyon_core::generate_user_id();
                if let Err(e) = version_repo
                    .create(CreateVersionRequest {
                        document_id: document_id.clone(),
                        content: current_content.clone(),
                        commit_message: Some("Auto-snapshot before update".to_string()),
                        created_by: user_id.to_string(),
                    })
                    .await
                {
                    warn!("Failed to auto-version document {}: {}", document_id, e);
                }
            }
        }

        metadata.content = Some(content.clone());
        metadata.content_hash = Some(compute_content_hash(&content));
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
        let validated_tags = ValidatedTagList::new(&tags)
            .map_err(|e| ServerError::bad_request(format!("Invalid tags: {}", e)))?;
        metadata.tags = serde_json::to_string(&validated_tags.as_strings())
            .unwrap_or_else(|_| "[]".to_string());
    }
    metadata.updated_at = chrono::Utc::now();

    state
        .repository
        .update(metadata.clone())
        .await
        .map_err(|e| ServerError::database(format!("Failed to update document: {}", e)))?;

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
                warn!(
                    "Failed to update outgoing links for document {}: {}",
                    document_id, e
                );
            }
        }
    }

    if let Err(e) = state
        .repository
        .update_search_index(
            &doc_id,
            &metadata.title,
            metadata.content.as_deref().unwrap_or(""),
            &metadata.parse_tags().unwrap_or_default(),
        )
        .await
    {
        warn!(
            "Failed to update search index for document {}: {}",
            doc_id, e
        );
    }

    state
        .index_in_tantivy(SearchDocument {
            id: doc_id,
            title: metadata.title.clone(),
            content: metadata.content.clone().unwrap_or_default(),
            author_id: UserId::parse_str(&metadata.author_id).unwrap_or_default(),
            repository_id: metadata
                .project_id
                .as_ref()
                .and_then(|id| RepositoryId::parse_str(id).ok()),
            tags: metadata.parse_tags().unwrap_or_default(),
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            custom_fields: BTreeMap::new(),
        })
        .await;

    // Re-generate and persist embedding asynchronously when content changes
    if content_changed {
        if let Some(ref ai) = state.ai_manager {
            if ai.is_available() {
                let ai = ai.clone();
                let repo = state.repository.clone();
                let doc_id = document_id.clone();
                let embed_text = format!(
                    "{} {}",
                    metadata.title,
                    metadata.content.as_deref().unwrap_or("")
                );
                tokio::spawn(async move {
                    match ai.embed(&embed_text).await {
                        Ok(embedding) => {
                            if let Err(e) = repo.update_embedding(&doc_id, embedding).await {
                                warn!("Failed to persist embedding for document {}: {}", doc_id, e);
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to generate embedding for document {}: {}",
                                doc_id, e
                            );
                        }
                    }
                });
            }
        }
    }

    if let Err(e) = ActivityRepository::create(
        &state.pool,
        CreateActivityEvent {
            actor_id: uuid::Uuid::parse_str(&metadata.author_id).unwrap_or_default(),
            event_type: "document_updated".to_string(),
            target_type: "document".to_string(),
            target_id: uuid::Uuid::parse_str(&document_id).unwrap_or_default(),
            description: format!("Updated document: {}", metadata.title),
            metadata: None,
        },
    )
    .await
    {
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

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::NodeUpdated,
                AuditSeverity::Low,
                "document_update",
                format!("Document '{}' updated", document_id),
            )
            .with_target(&document_id, "document"),
        )
        .await;

    state.api_cache.invalidate_documents().await;

    {
        let pool = state.pool.clone();
        let client = state.http_client.clone();
        let payload = serde_json::json!({
            "document_id": document_id.clone(),
            "title": response.title.clone(),
        });
        tokio::spawn(async move {
            crate::webhook_delivery::deliver_event(pool, client, "document_updated", &payload)
                .await;
        });
    }

    Ok(Json(response))
}

/// Delete a document by ID.
///
/// `DELETE /api/v1/documents/{document_id}`
///
/// Removes the document from the database and the Tantivy search index.
/// Response: 204 No Content, or 400 (invalid ID) / 404 on error.
#[utoipa::path(
    delete,
    path = "/api/v1/documents/{document_id}",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 204, description = "Document deleted"),
        (status = 404, description = "Document not found"),
    ),
    tag = "documents",
)]
pub async fn delete_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<StatusCode, ServerError> {
    debug!("Deleting document: {}", document_id);

    let doc_id = DocumentId::parse_str(&document_id)
        .map_err(|e| ServerError::bad_request(format!("Invalid document ID: {}", e)))?;

    match state.repository.delete(&doc_id).await {
        Ok(()) => {
            state.delete_from_tantivy(&document_id).await;
            state.api_cache.invalidate_documents().await;
            info!("Document deleted: {}", document_id);

            let _ = state
                .audit_logger
                .log(
                    AuditEvent::new(
                        AuditEventType::NodeDeleted,
                        AuditSeverity::Medium,
                        "document_delete",
                        format!("Document '{}' deleted", document_id),
                    )
                    .with_target(&document_id, "document"),
                )
                .await;

            {
                let pool = state.pool.clone();
                let client = state.http_client.clone();
                let payload = serde_json::json!({
                    "document_id": document_id.clone(),
                });
                tokio::spawn(async move {
                    crate::webhook_delivery::deliver_event(
                        pool,
                        client,
                        "document_deleted",
                        &payload,
                    )
                    .await;
                });
            }

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            warn!("Failed to delete document: {}", e);
            Err(ServerError::not_found("Document", &document_id))
        }
    }
}

/// List documents with pagination and optional filters.
///
/// `GET /api/v1/documents?page=&page_size=&author_id=&project_id=`
///
/// Query params: `page` (default 1), `page_size` (default 20, max 100), `author_id`, `project_id`.
/// Response: 200 with `DocumentSearchResponse` containing `results`, `total`, `page`, `page_size`.
#[utoipa::path(
    get,
    path = "/api/v1/documents",
    params(
        DocumentQuery,
    ),
    responses(
        (status = 200, description = "List of documents", body = DocumentSearchResponse),
        (status = 500, description = "Internal error"),
    ),
    tag = "documents",
)]
pub async fn list_documents(
    Query(query): Query<DocumentQuery>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentSearchResponse>, ServerError> {
    debug!(
        "Listing documents (page: {:?}, size: {:?})",
        query.page, query.page_size
    );

    let mut query_parts = Vec::new();
    if let Some(ref s) = query.search {
        query_parts.push(format!("search={}", urlencoding::encode(s)));
    }
    if let Some(ref a) = query.author_id {
        query_parts.push(format!("author_id={}", urlencoding::encode(a)));
    }
    if let Some(ref p) = query.project_id {
        query_parts.push(format!("project_id={}", urlencoding::encode(p)));
    }
    if let Some(page) = query.page {
        query_parts.push(format!("page={}", page));
    }
    if let Some(page_size) = query.page_size {
        query_parts.push(format!("page_size={}", page_size));
    }
    let query_string = query_parts.join("&");
    let key = crate::middleware::api_cache::cache_key(
        "GET",
        "/api/v1/documents",
        if query_string.is_empty() {
            None
        } else {
            Some(&query_string)
        },
    );

    if let Some(hit) = state.api_cache.get_response(&key).await {
        if let Ok(parsed) = serde_json::from_slice::<DocumentSearchResponse>(&hit.data) {
            return Ok(Json(parsed));
        }
    }

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

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
        state
            .repository
            .list_all(Some(page_size as i64), Some(offset as i64))
            .await
    };

    match documents {
        Ok(metas) => {
            let results: Vec<DocumentResponse> = metas
                .into_iter()
                .map(|m| {
                    let tags = m.parse_tags().unwrap_or_default();
                    DocumentResponse {
                        id: m.id,
                        title: m.title,
                        slug: m.slug,
                        html: None,
                        content: String::new(),
                        status: m.status,
                        visibility: m.visibility,
                        tags,
                        author_id: m.author_id,
                        repository_id: m.project_id,
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

            let response = DocumentSearchResponse {
                results,
                total,
                page,
                page_size,
            };

            if let Ok(bytes) = serde_json::to_vec(&response) {
                state
                    .api_cache
                    .set_response(&key, bytes, "application/json", None)
                    .await;
            }

            Ok(Json(response))
        }
        Err(e) => {
            warn!("Failed to list documents: {}", e);
            Err(ServerError::database(format!(
                "Failed to list documents: {}",
                e
            )))
        }
    }
}

/// Get document metadata (key-value pairs) by ID.
///
/// `GET /api/v1/documents/{document_id}/metadata`
///
/// Response: 200 with a JSON object of metadata fields, or 400/404 on error.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/metadata",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "Document metadata key-value pairs"),
        (status = 400, description = "Invalid document ID"),
        (status = 404, description = "Document not found"),
    ),
    tag = "documents",
)]
pub async fn get_document_metadata(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<BTreeMap<String, String>>, ServerError> {
    debug!("Getting document metadata: {}", document_id);

    let doc_id = DocumentId::parse_str(&document_id)
        .map_err(|e| ServerError::bad_request(format!("Invalid document ID: {}", e)))?;

    match state.repository.get_by_id(&doc_id).await {
        Ok(metadata) => {
            let mut result = BTreeMap::new();
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
            Err(ServerError::not_found("Document", &document_id))
        }
    }
}

/// Render markdown to HTML.
///
/// `POST /api/v1/documents/render`
///
/// Request body: raw markdown string.
/// Response: 200 with `html`, `word_count`, `character_count`, `heading_count`, `code_block_count`, `render_time_ms`.
#[utoipa::path(
    post,
    path = "/api/v1/documents/render",
    request_body = String,
    responses(
        (status = 200, description = "Rendered markdown"),
        (status = 400, description = "Failed to render markdown"),
    ),
    tag = "rendering",
)]
pub async fn render_markdown(body: String) -> Result<Json<serde_json::Value>, ServerError> {
    debug!("Rendering markdown ({} bytes)", body.len());

    let renderer = Renderer::new(RenderConfig::default());

    let result = renderer
        .render(&body, None)
        .map_err(|e| ServerError::bad_request(format!("Failed to render markdown: {}", e)))?;

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

/// List documents with cursor-based pagination.
///
/// `GET /api/v1/documents/cursor?after=&before=&limit=`
///
/// Query params: `after` (cursor from previous page's `next_cursor`),
///   `before` (cursor from previous page's `prev_cursor`), `limit` (default 20, max 100).
/// Response: 200 with `CursorPage<DocumentResponse>`.
#[utoipa::path(
    get,
    path = "/api/v1/documents/cursor",
    params(CursorParams),
    responses(
        (status = 200, description = "Cursor-paginated list of documents", body = DocumentCursorPage),
        (status = 500, description = "Internal error"),
    ),
    tag = "documents",
)]
pub async fn list_documents_cursor(
    Query(params): Query<CursorParams>,
    State(state): State<DocumentState>,
) -> Result<Json<CursorPage<DocumentResponse>>, ServerError> {
    let limit = params.limit();
    let direction = params.direction();

    let fetch_limit = (limit + 1) as i64;

    let cursor_str = params.after.as_deref().or(params.before.as_deref());

    let all_docs = state
        .repository
        .list_after_cursor(fetch_limit, cursor_str)
        .await
        .map_err(|e| ServerError::database(format!("Failed to list documents: {}", e)))?;

    let total_count = state.repository.count_documents().await.unwrap_or(0);

    let mut items: Vec<DocumentResponse> = all_docs
        .into_iter()
        .map(|m| {
            let tags = m.parse_tags().unwrap_or_default();
            DocumentResponse {
                id: m.id,
                title: m.title,
                slug: m.slug,
                html: None,
                content: String::new(),
                status: m.status,
                visibility: m.visibility,
                tags,
                author_id: m.author_id,
                repository_id: m.project_id,
                word_count: m.word_count as usize,
                character_count: m.character_count as usize,
                created_at: m.created_at.to_rfc3339(),
                updated_at: m.updated_at.to_rfc3339(),
                published_at: m.published_at.map(|t| t.to_rfc3339()),
            }
        })
        .collect();

    let has_extra = items.len() > limit;
    if has_extra {
        items.truncate(limit);
    }

    let has_next = if direction == "asc" {
        has_extra
    } else {
        cursor_str.is_some()
    };
    let has_prev = if direction == "asc" {
        cursor_str.is_some()
    } else {
        has_extra
    };

    let first_id = items.first().map(|d| d.id.clone());
    let last_id = items.last().map(|d| d.id.clone());

    let page = CursorPage::new(items, has_next, has_prev)
        .with_cursors(first_id.as_deref(), last_id.as_deref(), direction)
        .with_total_count(total_count);

    Ok(Json(page))
}
