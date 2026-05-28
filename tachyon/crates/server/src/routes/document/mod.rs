pub mod document_attachments;
pub mod document_batch;
pub mod document_crud;
pub mod document_search;
pub mod document_templates;
pub mod document_versions;

use axum::extract::DefaultBodyLimit;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tachyon_core::{Document, DocumentContent, DocumentStatus, DocumentVisibility};
use tachyon_database::DatabasePool;
use tachyon_search::IndexManager;
use tokio::sync::Mutex;
use tracing::warn;

use crate::audit::AuditLogger;
use crate::config::GuestConfig;

#[derive(Clone)]
pub struct DocumentState {
    pub pool: DatabasePool,
    pub repository: tachyon_database::DocumentRepository,
    pub guest_config: GuestConfig,
    pub index_manager: Option<Arc<Mutex<IndexManager>>>,
    pub http_client: reqwest::Client,
    pub api_cache: crate::middleware::api_cache::ApiCache,
    pub audit_logger: AuditLogger,
    pub ai_manager: Option<Arc<crate::ai::AiManager>>,
}

impl DocumentState {
    pub fn new(pool: DatabasePool, http_client: reqwest::Client) -> Self {
        let repository = tachyon_database::DocumentRepository::new(pool.clone());
        Self {
            pool,
            repository,
            guest_config: GuestConfig::default(),
            index_manager: None,
            http_client,
            api_cache: crate::middleware::api_cache::ApiCache::new(std::time::Duration::from_secs(
                60,
            )),
            audit_logger: AuditLogger::disabled(),
            ai_manager: None,
        }
    }

    pub fn with_guest_config(
        pool: DatabasePool,
        guest_config: GuestConfig,
        http_client: reqwest::Client,
    ) -> Self {
        let repository = tachyon_database::DocumentRepository::new(pool.clone());
        Self {
            pool,
            repository,
            guest_config,
            index_manager: None,
            http_client,
            api_cache: crate::middleware::api_cache::ApiCache::new(std::time::Duration::from_secs(
                60,
            )),
            audit_logger: AuditLogger::disabled(),
            ai_manager: None,
        }
    }

    pub fn with_index_manager(mut self, index_manager: Arc<Mutex<IndexManager>>) -> Self {
        self.index_manager = Some(index_manager);
        self
    }

    pub fn with_audit_logger(mut self, logger: AuditLogger) -> Self {
        self.audit_logger = logger;
        self
    }

    pub fn with_ai_manager(mut self, ai_manager: Arc<crate::ai::AiManager>) -> Self {
        self.ai_manager = Some(ai_manager);
        self
    }

    pub fn is_public_access_enabled(&self) -> bool {
        self.guest_config.public_notes_enabled
    }

    pub(crate) async fn index_in_tantivy(&self, search_doc: tachyon_search::SearchDocument) {
        if let Some(ref mgr) = self.index_manager {
            let guard = mgr.lock().await;
            if let Err(e) = guard.index_document(&search_doc).await {
                warn!("Failed to index document in Tantivy: {}", e);
            }
        }
    }

    pub(crate) async fn delete_from_tantivy(&self, doc_id: &str) {
        if let Some(ref mgr) = self.index_manager {
            let guard = mgr.lock().await;
            if let Err(e) = guard.delete_document(doc_id).await {
                warn!("Failed to delete document from Tantivy: {}", e);
            }
        }
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DocumentQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub project_id: Option<String>,
    pub author_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DocumentResponse {
    pub id: String,
    pub title: String,
    pub slug: Option<String>,
    pub html: Option<String>,
    pub content: String,
    pub status: String,
    pub visibility: String,
    pub tags: Vec<String>,
    pub author_id: String,
    pub repository_id: Option<String>,
    pub word_count: usize,
    pub character_count: usize,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        let (content, html) = match &doc.content {
            DocumentContent::Markdown { content } => {
                let html = None;
                (content.clone(), html)
            }
            DocumentContent::Text { content } => (content.clone(), None),
            DocumentContent::Binary { .. } => ("[Binary content]".to_string(), None),
        };

        let status = match doc.status {
            DocumentStatus::Draft => "draft",
            DocumentStatus::Published => "published",
            DocumentStatus::Archived => "archived",
            DocumentStatus::Deleted => "deleted",
        };

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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DocumentSearchResponse {
    pub results: Vec<DocumentResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DocumentCursorPage {
    pub data: Vec<DocumentResponse>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub total_count: Option<i64>,
}

impl From<crate::pagination::CursorPage<DocumentResponse>> for DocumentCursorPage {
    fn from(page: crate::pagination::CursorPage<DocumentResponse>) -> Self {
        Self {
            data: page.data,
            has_next: page.has_next,
            has_prev: page.has_prev,
            next_cursor: page.next_cursor,
            prev_cursor: page.prev_cursor,
            total_count: page.total_count,
        }
    }
}

pub use document_attachments::{
    delete_attachment, download_attachment, list_attachments, upload_attachment, AttachmentResponse,
};
pub use document_batch::{
    batch_operations, BatchAction, BatchOperation, BatchOperationResult, BatchOperationStatus,
    BatchRequest, BatchResponse,
};
pub use document_crud::{
    create_document, delete_document, get_document, get_document_metadata, list_documents,
    list_documents_cursor, render_markdown, update_document, CreateDocumentRequest,
    UpdateDocumentRequest,
};
pub use document_search::{
    get_backlinks, search_documents, semantic_search, BacklinkItem, BacklinksResponse,
    SemanticSearchParams, SemanticSearchResponse,
};
pub use document_templates::{
    create_template, delete_template, get_template, list_templates, update_template,
    CreateTemplateBody, TemplateQuery, TemplateResponse, UpdateTemplateBody,
};
pub use document_versions::{
    create_version, diff_versions, get_version, list_versions, restore_version, CreateVersionBody,
    DiffLine, DiffStats, DocumentDiffResponse, RestoreVersionResponse, VersionResponse,
};

pub fn create_document_router() -> axum::Router<DocumentState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/documents", get(list_documents))
        .route("/documents/cursor", get(list_documents_cursor))
        .route("/documents", post(create_document))
        .route("/documents/batch", post(document_batch::batch_operations))
        .route("/documents/search", get(search_documents))
        .route("/documents/semantic-search", get(semantic_search))
        .route("/documents/{document_id}", get(get_document))
        .route("/documents/{document_id}", put(update_document))
        .route("/documents/{document_id}", delete(delete_document))
        .route(
            "/documents/{document_id}/metadata",
            get(get_document_metadata),
        )
        .route("/documents/{document_id}/versions", get(list_versions))
        .route("/documents/{document_id}/versions", post(create_version))
        .route(
            "/documents/{document_id}/versions/{version_number}",
            get(get_version),
        )
        .route(
            "/documents/{document_id}/versions/{v1}/diff/{v2}",
            get(diff_versions),
        )
        .route(
            "/documents/{document_id}/versions/{version_id}/restore",
            post(restore_version),
        )
        .route(
            "/documents/{document_id}/attachments",
            get(list_attachments),
        )
        .route(
            "/documents/{document_id}/attachments",
            post(upload_attachment).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .route(
            "/documents/{document_id}/attachments/{attachment_id}",
            delete(delete_attachment),
        )
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

    #[test]
    fn test_cursor_params_empty_returns_first_page() {
        let params = crate::pagination::CursorParams::default();
        assert_eq!(params.limit(), 20);
        assert_eq!(params.direction(), "asc");
        assert!(params.after.is_none());
        assert!(params.before.is_none());
    }

    #[test]
    fn test_cursor_params_after_returns_next_page() {
        let params = crate::pagination::CursorParams {
            after: Some("doc123:asc".to_string()),
            before: None,
            limit: Some(10),
        };
        assert_eq!(params.limit(), 10);
        assert_eq!(params.direction(), "asc");
    }

    #[test]
    fn test_cursor_params_before_returns_prev_page() {
        let params = crate::pagination::CursorParams {
            after: None,
            before: Some("doc456:desc".to_string()),
            limit: None,
        };
        assert_eq!(params.limit(), 20);
        assert_eq!(params.direction(), "desc");
    }

    #[test]
    fn test_cursor_params_limit_clamping() {
        let params_zero = crate::pagination::CursorParams {
            limit: Some(0),
            ..Default::default()
        };
        assert_eq!(params_zero.limit(), 1);

        let params_huge = crate::pagination::CursorParams {
            limit: Some(500),
            ..Default::default()
        };
        assert_eq!(params_huge.limit(), 100);
    }

    #[test]
    fn test_cursor_page_serialization() {
        let page = crate::pagination::CursorPage::new(
            vec![DocumentResponse {
                id: "doc1".to_string(),
                title: "First".to_string(),
                slug: Some("first".to_string()),
                html: None,
                content: "content".to_string(),
                status: "draft".to_string(),
                visibility: "private".to_string(),
                tags: vec![],
                author_id: "user1".to_string(),
                repository_id: None,
                word_count: 1,
                character_count: 7,
                created_at: "2026-01-01T00:00:00+00:00".to_string(),
                updated_at: "2026-01-01T00:00:00+00:00".to_string(),
                published_at: None,
            }],
            true,
            false,
        )
        .with_cursors(Some("doc1"), Some("doc1"), "asc");

        assert!(page.has_next);
        assert!(!page.has_prev);
        assert_eq!(page.data.len(), 1);
        assert!(page.next_cursor.is_some());
    }

    #[test]
    fn test_document_cursor_page_from_pagination() {
        let pagination_page = crate::pagination::CursorPage::new(
            vec![DocumentResponse {
                id: "doc1".to_string(),
                title: "Test".to_string(),
                slug: None,
                html: None,
                content: "".to_string(),
                status: "draft".to_string(),
                visibility: "private".to_string(),
                tags: vec![],
                author_id: "user1".to_string(),
                repository_id: None,
                word_count: 0,
                character_count: 0,
                created_at: "2026-01-01T00:00:00+00:00".to_string(),
                updated_at: "2026-01-01T00:00:00+00:00".to_string(),
                published_at: None,
            }],
            false,
            false,
        );

        let doc_page = DocumentCursorPage::from(pagination_page);
        assert_eq!(doc_page.data.len(), 1);
        assert!(!doc_page.has_next);
        assert!(!doc_page.has_prev);
        assert!(doc_page.next_cursor.is_none());
        assert!(doc_page.prev_cursor.is_none());
    }
}
