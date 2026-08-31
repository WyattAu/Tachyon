//! API v2 routes.
//!
//! Provides the v2 API surface with a consistent response envelope:
//! - Success: `{"data": ..., "meta": {"version": "2.0", "timestamp": "..."}}`
//! - Error:   `{"error": {"code": "...", "message": "..."}}`
//!
//! v2 endpoints:
//! - GET  /health
//! - POST /auth/login
//! - POST /auth/register
//! - POST /auth/refresh
//! - GET  /auth/me
//! - PUT  /auth/me
//! - GET  /documents
//! - POST /documents
//! - GET  /documents/{document_id}
//! - PUT  /documents/{document_id}
//! - DELETE /documents/{document_id}
//! - GET  /search

use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{debug, warn};

use crate::routes::document::DocumentState;
use crate::routes::search::SearchState;
use crate::routes::user::UserState;

use crate::routes::document::document_search::visible_to_caller;
use crate::routes::document::{CreateDocumentRequest, UpdateDocumentRequest};

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct V2State {
    pub document_state: DocumentState,
    pub user_state: UserState,
    pub search_state: SearchState,
}

// ============================================================================
// Router
// ============================================================================

pub fn create_v2_router(state: V2State) -> axum::Router<()> {
    axum::Router::new()
        // Health
        .route("/health", get(v2_health))
        // Auth
        .route("/auth/login", post(v2_login))
        .route("/auth/register", post(v2_register))
        .route("/auth/refresh", post(v2_refresh))
        .route("/auth/me", get(v2_get_me).put(v2_update_me))
        // Documents
        .route(
            "/documents",
            get(v2_list_documents).post(v2_create_document),
        )
        .route(
            "/documents/{document_id}",
            get(v2_get_document)
                .put(v2_update_document)
                .delete(v2_delete_document),
        )
        // Search
        .route("/search", get(v2_search))
        .with_state(state)
}

pub fn v2_routes() -> axum::Router<()> {
    axum::Router::new().route("/health", get(v2_health))
}

// ============================================================================
// Envelope helpers
// ============================================================================

pub(crate) fn v2_ok(data: Value) -> Value {
    json!({
        "data": data,
        "meta": {
            "version": "2.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    })
}

pub(crate) fn v2_error(code: &str, message: &str, status: StatusCode) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "error": {
                "code": code,
                "message": message,
            }
        })),
    )
}

// ============================================================================
// Health
// ============================================================================

pub async fn v2_health() -> Json<Value> {
    Json(v2_ok(json!({
        "status": "ok",
        "version": "2.0.0",
        "message": "Tachyon API v2"
    })))
}

// ============================================================================
// Auth
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct V2LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn v2_login(
    State(state): State<V2State>,
    axum::Json(body): axum::Json<V2LoginRequest>,
) -> Response {
    // Delegate to v1 authenticate handler via UserState
    let req = crate::routes::user::types::AuthenticateRequest {
        username: body.username,
        password: body.password,
    };
    // Use the internal authenticate logic
    match crate::routes::user::handlers::authenticate(
        State(state.user_state.clone()),
        axum::Json(req),
    )
    .await
    {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_else(|_| json!({"success": true}));
            Json(v2_ok(data)).into_response()
        }
        Err((status, Json(err))) => {
            let code = match status {
                StatusCode::UNAUTHORIZED => "UNAUTHORIZED",
                StatusCode::BAD_REQUEST => "BAD_REQUEST",
                StatusCode::FORBIDDEN => "FORBIDDEN",
                StatusCode::TOO_MANY_REQUESTS => "TOO_MANY_REQUESTS",
                _ => "INTERNAL_ERROR",
            };
            v2_error(code, &err.message, status).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct V2RegisterRequest {
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub password: String,
}

pub async fn v2_register(
    State(state): State<V2State>,
    axum::Json(body): axum::Json<V2RegisterRequest>,
) -> Response {
    let req = crate::routes::user::types::RegisterRequest {
        username: body.username,
        display_name: body.display_name,
        email: body.email,
        password: body.password,
    };
    match crate::routes::user::handlers::register(State(state.user_state.clone()), axum::Json(req))
        .await
    {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_else(|_| json!({"success": true}));
            Json(v2_ok(data)).into_response()
        }
        Err((status, Json(err))) => {
            let code = match status {
                StatusCode::CONFLICT => "CONFLICT",
                StatusCode::BAD_REQUEST => "BAD_REQUEST",
                StatusCode::FORBIDDEN => "FORBIDDEN",
                _ => "INTERNAL_ERROR",
            };
            v2_error(code, &err.message, status).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct V2RefreshRequest {
    pub refresh_token: String,
}

pub async fn v2_refresh(
    State(state): State<V2State>,
    axum::Json(body): axum::Json<V2RefreshRequest>,
) -> Response {
    let req = crate::routes::user::types::RefreshRequest {
        refresh_token: body.refresh_token,
    };
    match crate::routes::user::handlers::refresh_token_handler(
        State(state.user_state.clone()),
        axum::Json(req),
    )
    .await
    {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_else(|_| json!({"success": true}));
            Json(v2_ok(data)).into_response()
        }
        Err((status, Json(err))) => {
            let code = match status {
                StatusCode::UNAUTHORIZED => "UNAUTHORIZED",
                StatusCode::BAD_REQUEST => "BAD_REQUEST",
                _ => "INTERNAL_ERROR",
            };
            v2_error(code, &err.message, status).into_response()
        }
    }
}

pub async fn v2_get_me(State(state): State<V2State>, headers: HeaderMap) -> Response {
    match crate::routes::user::handlers::get_me(State(state.user_state.clone()), headers).await {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_default();
            Json(v2_ok(data)).into_response()
        }
        Err((status, Json(err))) => v2_error("UNAUTHORIZED", &err.message, status).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct V2UpdateProfileRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
}

pub async fn v2_update_me(
    State(state): State<V2State>,
    headers: HeaderMap,
    axum::Json(body): axum::Json<V2UpdateProfileRequest>,
) -> Response {
    let req = crate::routes::user::types::UpdateProfileRequest {
        display_name: body.display_name,
        email: body.email,
    };
    match crate::routes::user::handlers::update_me(
        State(state.user_state.clone()),
        headers,
        axum::Json(req),
    )
    .await
    {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_default();
            Json(v2_ok(data)).into_response()
        }
        Err((status, Json(err))) => v2_error("BAD_REQUEST", &err.message, status).into_response(),
    }
}

// ============================================================================
// Documents
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct V2DocumentQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub author_id: Option<String>,
    pub project_id: Option<String>,
}

pub async fn v2_list_documents(
    Query(query): Query<V2DocumentQuery>,
    State(state): State<V2State>,
    auth: Option<Extension<crate::middleware::AuthContext>>,
) -> Response {
    debug!("v2: listing documents");

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let documents = if let Some(ref author_id) = query.author_id {
        state
            .document_state
            .repository
            .list_by_author(author_id, Some(per_page as i64), Some(offset as i64))
            .await
    } else if let Some(ref project_id) = query.project_id {
        state
            .document_state
            .repository
            .list_by_project(project_id, Some(per_page as i64), Some(offset as i64))
            .await
    } else {
        state
            .document_state
            .repository
            .list_all(Some(per_page as i64), Some(offset as i64))
            .await
    };

    let caller_id = auth.as_ref().map(|Extension(ctx)| ctx.user_id.as_str());
    let is_admin = auth.as_ref().is_some_and(|Extension(ctx)| ctx.is_admin());

    match documents {
        Ok(metas) => {
            let items: Vec<Value> = metas
                .into_iter()
                .filter(|m| visible_to_caller(&m.visibility, &m.author_id, caller_id, is_admin))
                .map(|m| {
                    json!({
                        "id": m.id,
                        "title": m.title,
                        "slug": m.slug,
                        "status": m.status,
                        "visibility": m.visibility,
                        "tags": m.parse_tags().unwrap_or_default(),
                        "author_id": m.author_id,
                        "repository_id": m.project_id,
                        "word_count": m.word_count,
                        "character_count": m.character_count,
                        "created_at": m.created_at.to_rfc3339(),
                        "updated_at": m.updated_at.to_rfc3339(),
                        "published_at": m.published_at.map(|t| t.to_rfc3339()),
                    })
                })
                .collect();

            let total = items.len();
            let total_pages = total.div_ceil(per_page);

            Json(v2_ok(json!({
                "items": items,
                "pagination": {
                    "page": page,
                    "per_page": per_page,
                    "total_items": total,
                    "total_pages": total_pages,
                    "has_next": page < total_pages,
                    "has_prev": page > 1,
                }
            })))
            .into_response()
        }
        Err(e) => {
            warn!("v2: failed to list documents: {}", e);
            v2_error(
                "DATABASE_ERROR",
                &format!("Failed to list documents: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct V2CreateDocumentRequest {
    pub title: String,
    pub content: Option<String>,
    pub project_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<String>,
}

pub async fn v2_create_document(
    State(state): State<V2State>,
    axum::Json(body): axum::Json<V2CreateDocumentRequest>,
) -> Response {
    let req = CreateDocumentRequest {
        title: body.title,
        content: body.content.unwrap_or_default(),
        project_id: body.project_id,
        tags: body.tags.unwrap_or_default(),
        visibility: body.visibility,
    };

    match crate::routes::document::create_document(
        State(state.document_state.clone()),
        None,
        axum::Json(req),
    )
    .await
    {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_default();
            Json(v2_ok(data)).into_response()
        }
        Err(e) => v2_error("BAD_REQUEST", &e.to_string(), StatusCode::BAD_REQUEST).into_response(),
    }
}

pub async fn v2_get_document(
    Path(document_id): Path<String>,
    State(state): State<V2State>,
    auth: Option<Extension<crate::middleware::AuthContext>>,
) -> Response {
    debug!("v2: getting document {}", document_id);

    let doc_id = match tachyon_core::DocumentId::parse_str(&document_id) {
        Ok(id) => id,
        Err(e) => {
            return v2_error(
                "VALIDATION_ERROR",
                &format!("Invalid document ID: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    match state.document_state.repository.get_by_id(&doc_id).await {
        Ok(m) => {
            let caller_id = auth.as_ref().map(|Extension(ctx)| ctx.user_id.as_str());
            let is_admin = auth.as_ref().is_some_and(|Extension(ctx)| ctx.is_admin());
            if !visible_to_caller(&m.visibility, &m.author_id, caller_id, is_admin) {
                return v2_error(
                    "FORBIDDEN",
                    "You do not have permission to access this document",
                    StatusCode::FORBIDDEN,
                )
                .into_response();
            }
            let tags = m.parse_tags().unwrap_or_default();
            Json(v2_ok(json!({
                "id": m.id,
                "title": m.title,
                "slug": m.slug,
                "html": m.html,
                "content": m.content.unwrap_or_default(),
                "status": m.status,
                "visibility": m.visibility,
                "tags": tags,
                "author_id": m.author_id,
                "repository_id": m.project_id,
                "word_count": m.word_count,
                "character_count": m.character_count,
                "created_at": m.created_at.to_rfc3339(),
                "updated_at": m.updated_at.to_rfc3339(),
                "published_at": m.published_at.map(|t| t.to_rfc3339()),
            })))
            .into_response()
        }
        Err(e) => {
            warn!("v2: failed to get document {}: {}", document_id, e);
            v2_error(
                "NOT_FOUND",
                &format!("Document '{}' not found", document_id),
                StatusCode::NOT_FOUND,
            )
            .into_response()
        }
    }
}

pub async fn v2_update_document(
    Path(document_id): Path<String>,
    State(state): State<V2State>,
    auth: Option<Extension<crate::middleware::AuthContext>>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> Response {
    debug!("v2: updating document {}", document_id);

    let doc_id = match tachyon_core::DocumentId::parse_str(&document_id) {
        Ok(id) => id,
        Err(e) => {
            return v2_error(
                "VALIDATION_ERROR",
                &format!("Invalid document ID: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    // Build UpdateDocumentRequest from JSON body (flexible field set)
    let req = UpdateDocumentRequest {
        title: body.get("title").and_then(|v| v.as_str()).map(String::from),
        content: body
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from),
        tags: body.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        }),
        visibility: body
            .get("visibility")
            .and_then(|v| v.as_str())
            .map(String::from),
        status: body
            .get("status")
            .and_then(|v| v.as_str())
            .map(String::from),
    };

    match crate::routes::document::update_document(
        Path(doc_id.to_string()),
        State(state.document_state.clone()),
        auth,
        axum::Json(req),
    )
    .await
    {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_default();
            Json(v2_ok(data)).into_response()
        }
        Err(e) => v2_error("BAD_REQUEST", &e.to_string(), StatusCode::BAD_REQUEST).into_response(),
    }
}

pub async fn v2_delete_document(
    Path(document_id): Path<String>,
    State(state): State<V2State>,
    auth: Option<Extension<crate::middleware::AuthContext>>,
) -> Response {
    debug!("v2: deleting document {}", document_id);

    let doc_id = match tachyon_core::DocumentId::parse_str(&document_id) {
        Ok(id) => id,
        Err(e) => {
            return v2_error(
                "VALIDATION_ERROR",
                &format!("Invalid document ID: {}", e),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    match crate::routes::document::delete_document(
        Path(doc_id.to_string()),
        State(state.document_state.clone()),
        auth,
    )
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => v2_error("BAD_REQUEST", &e.to_string(), StatusCode::BAD_REQUEST).into_response(),
    }
}

// ============================================================================
// Search
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct V2SearchQuery {
    pub q: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub project_id: Option<String>,
    pub author_id: Option<String>,
    pub tags: Option<String>,
}

pub async fn v2_search(
    Query(query): Query<V2SearchQuery>,
    State(state): State<V2State>,
) -> Response {
    debug!(query = %query.q, "v2: search");

    let req = crate::routes::search::SearchQuery {
        q: query.q,
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(20),
        content_type: query.content_type,
        status: query.status,
        visibility: query.visibility,
        project_id: query.project_id,
        author_id: query.author_id,
        tags: query.tags,
        date_from: None,
        date_to: None,
    };

    match crate::routes::search::search(Query(req), State(state.search_state.clone())).await {
        Ok(Json(response)) => {
            let data = serde_json::to_value(&response).unwrap_or_default();
            Json(v2_ok(data)).into_response()
        }
        Err(e) => v2_error("SEARCH_ERROR", &e.to_string(), StatusCode::BAD_REQUEST).into_response(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_v2_health_response() {
        let Json(body) = v2_health().await;
        assert_eq!(body["data"]["status"], "ok");
        assert_eq!(body["data"]["version"], "2.0.0");
        assert_eq!(body["data"]["message"], "Tachyon API v2");
        assert_eq!(body["meta"]["version"], "2.0");
        assert!(body["meta"]["timestamp"].is_string());
    }

    #[test]
    fn test_v2_ok_envelope() {
        let body = v2_ok(json!({"foo": "bar"}));
        assert_eq!(body["data"]["foo"], "bar");
        assert_eq!(body["meta"]["version"], "2.0");
        assert!(body["meta"]["timestamp"].is_string());
    }

    #[test]
    fn test_v2_error_envelope() {
        let (status, Json(body)) = v2_error("NOT_FOUND", "thing not found", StatusCode::NOT_FOUND);
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "NOT_FOUND");
        assert_eq!(body["error"]["message"], "thing not found");
    }

    #[test]
    fn test_v2_private_document_access_is_denied_cross_tenant() {
        assert!(!visible_to_caller("private", "owner", Some("other"), false));
        assert!(visible_to_caller("private", "owner", Some("owner"), false));
        assert!(visible_to_caller("private", "owner", Some("other"), true));
    }

    #[test]
    fn test_v2_public_document_access_allows_guests() {
        assert!(visible_to_caller("public", "owner", None, false));
    }

    #[test]
    fn test_v2_document_query_defaults() {
        let query = V2DocumentQuery {
            page: None,
            per_page: None,
            author_id: None,
            project_id: None,
        };
        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.per_page.unwrap_or(20), 20);
    }

    #[test]
    fn test_v2_search_query_defaults() {
        let query = V2SearchQuery {
            q: "test".to_string(),
            page: None,
            page_size: None,
            content_type: None,
            status: None,
            visibility: None,
            project_id: None,
            author_id: None,
            tags: None,
        };
        assert_eq!(query.q, "test");
        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.page_size.unwrap_or(20), 20);
    }

    #[test]
    fn test_v2_login_request_deserialize() {
        let json = r#"{"username":"admin","password":"secret"}"#;
        let req: V2LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "admin");
        assert_eq!(req.password, "secret");
    }

    #[test]
    fn test_v2_register_request_deserialize() {
        let json = r#"{"username":"admin","display_name":"Admin","email":"admin@test.com","password":"secret"}"#;
        let req: V2RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "admin");
        assert_eq!(req.display_name, "Admin");
        assert_eq!(req.email, Some("admin@test.com".to_string()));
        assert_eq!(req.password, "secret");
    }

    #[test]
    fn test_v2_create_document_request_deserialize() {
        let json = r#"{"title":"Hello","content":"world","tags":["test"],"visibility":"public"}"#;
        let req: V2CreateDocumentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.title, "Hello");
        assert_eq!(req.content, Some("world".to_string()));
        assert_eq!(req.tags, Some(vec!["test".to_string()]));
        assert_eq!(req.visibility, Some("public".to_string()));
    }
}
