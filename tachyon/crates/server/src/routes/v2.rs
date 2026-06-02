//! API v2 routes.
//!
//! Provides the v2 API surface with a consistent response envelope:
//! - Success: `{"data": ..., "meta": {"version": "2.0", "timestamp": "..."}}`
//! - Error:   `{"error": {"code": "...", "message": "..."}}`

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{debug, warn};

use crate::routes::document::DocumentState;

#[derive(Clone)]
pub struct V2State {
    pub document_state: DocumentState,
}

pub fn create_v2_router(state: V2State) -> axum::Router<()> {
    axum::Router::new()
        .route("/health", get(v2_health))
        .route("/documents", get(v2_list_documents))
        .route("/documents/{document_id}", get(v2_get_document))
        .with_state(state)
}

pub fn v2_routes() -> axum::Router<()> {
    axum::Router::new().route("/health", get(v2_health))
}

fn v2_ok(data: Value) -> Value {
    json!({
        "data": data,
        "meta": {
            "version": "2.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    })
}

fn v2_error(code: &str, message: &str, status: StatusCode) -> (StatusCode, Json<Value>) {
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

pub async fn v2_health() -> Json<Value> {
    Json(v2_ok(json!({
        "status": "ok",
        "version": "2.0.0",
        "message": "Tachyon API v2"
    })))
}

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

    match documents {
        Ok(metas) => {
            let items: Vec<Value> = metas
                .into_iter()
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
            v2_error("DATABASE_ERROR", &format!("Failed to list documents: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        }
    }
}

pub async fn v2_get_document(
    Path(document_id): Path<String>,
    State(state): State<V2State>,
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
}
