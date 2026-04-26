use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use tachyon_core::{compute_content_hash, Document, DocumentContent, DocumentId, DocumentStatus, DocumentVisibility};
use tachyon_core::id::{RepositoryId, UserId};
use tachyon_database::{ActivityRepository, CreateActivityEvent, CreateVersionRequest, DocumentVersionRepository};
use tachyon_renderer::{RenderConfig, Renderer, MarkdownParser};
use tachyon_search::SearchDocument;
use tracing::{debug, info, warn};

use super::{DocumentState, ErrorResponse, DocumentResponse, DocumentQuery, DocumentSearchResponse};

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<String>,
    pub status: Option<String>,
}

pub async fn create_document(
    State(state): State<DocumentState>,
    auth: Option<Extension<crate::middleware::AuthContext>>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating new document: {}", req.title);

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

    let author_id: tachyon_core::id::UserId = auth
        .and_then(|Extension(ctx)| tachyon_core::id::UserId::parse_str(&ctx.user_id).ok())
        .unwrap_or_else(tachyon_core::generate_user_id);

    let doc_id = tachyon_core::generate_document_id();
    let content = DocumentContent::markdown(req.content.clone());
    let mut doc = Document::new(doc_id, req.title.clone(), author_id, content);

    if let Some(ref vis) = req.visibility {
        let visibility = match vis.to_lowercase().as_str() {
            "public" => DocumentVisibility::Public,
            "restricted" => DocumentVisibility::Restricted,
            _ => DocumentVisibility::Private,
        };
        doc.visibility = visibility;
    }

    for tag in &req.tags {
        if let Err(e) = doc.metadata.add_tag(tag.clone()) {
            warn!("Failed to add tag: {}", e);
        }
    }

    if let Some(ref repo_id) = req.project_id {
        if let Ok(id) = tachyon_core::id::RepositoryId::parse_str(repo_id) {
            doc.repository_id = Some(id);
        }
    }

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
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to create document: {}", e),
                details: None,
            }),
        ));
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
                warn!("Failed to update outgoing links for document {}: {}", doc.id, e);
            }
        }
    }

    if let Err(e) = state.repository.update_search_index(
        &doc.id,
        &doc.metadata.title,
        doc.content.as_text().unwrap_or(""),
        &doc.metadata.tags,
    ).await {
        warn!("Failed to update search index for document {}: {}", doc.id, e);
    }

    state.index_in_tantivy(SearchDocument {
        id: doc.id,
        title: doc.metadata.title.clone(),
        content: doc.content.as_text().unwrap_or("").to_string(),
        author_id,
        repository_id: doc.repository_id,
        tags: doc.metadata.tags.clone(),
        created_at: doc.metadata.created_at,
        updated_at: doc.metadata.updated_at,
        custom_fields: BTreeMap::new(),
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

    {
        let pool = state.pool.clone();
        let client = state.http_client.clone();
        let payload = serde_json::json!({
            "document_id": response.id.clone(),
            "title": response.title.clone(),
        });
        tokio::spawn(async move {
            crate::webhook_delivery::deliver_event(pool, client, "document_created", &payload).await;
        });
    }

    Ok(Json(response))
}

pub async fn get_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting document: {}", document_id);

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

pub async fn update_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Updating document: {}", document_id);

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

    if let Some(title) = req.title {
        metadata.title = title;
    }
    if let Some(content) = req.content {
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

    if let Err(e) = state.repository.update_search_index(
        &doc_id,
        &metadata.title,
        metadata.content.as_deref().unwrap_or(""),
        &metadata.parse_tags().unwrap_or_default(),
    ).await {
        warn!("Failed to update search index for document {}: {}", doc_id, e);
    }

    state.index_in_tantivy(SearchDocument {
        id: doc_id,
        title: metadata.title.clone(),
        content: metadata.content.clone().unwrap_or_default(),
        author_id: UserId::parse_str(&metadata.author_id).unwrap_or_default(),
        repository_id: metadata.project_id.as_ref().and_then(|id| RepositoryId::parse_str(id).ok()),
        tags: metadata.parse_tags().unwrap_or_default(),
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        custom_fields: BTreeMap::new(),
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

    {
        let pool = state.pool.clone();
        let client = state.http_client.clone();
        let payload = serde_json::json!({
            "document_id": document_id.clone(),
            "title": response.title.clone(),
        });
        tokio::spawn(async move {
            crate::webhook_delivery::deliver_event(pool, client, "document_updated", &payload).await;
        });
    }

    Ok(Json(response))
}

pub async fn delete_document(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    debug!("Deleting document: {}", document_id);

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

    match state.repository.delete(&doc_id).await {
        Ok(()) => {
            state.delete_from_tantivy(&document_id).await;
            info!("Document deleted: {}", document_id);

            {
                let pool = state.pool.clone();
                let client = state.http_client.clone();
                let payload = serde_json::json!({
                    "document_id": document_id.clone(),
                });
                tokio::spawn(async move {
                    crate::webhook_delivery::deliver_event(pool, client, "document_deleted", &payload).await;
                });
            }

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

pub async fn get_document_metadata(
    Path(document_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<BTreeMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting document metadata: {}", document_id);

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

pub async fn render_markdown(
    body: String,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Rendering markdown ({} bytes)", body.len());

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
