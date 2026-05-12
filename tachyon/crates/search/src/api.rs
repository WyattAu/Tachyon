// API Module
// REST and WebSocket API endpoints for search operations

use crate::error::SearchError;
use crate::indexer::IndexManager;
use crate::query::QueryEngine;
use crate::types::{
    BatchIndexRequest, BatchIndexResponse, SearchRequest, SearchResponse, SortOrder, Suggestion,
};
use axum::{
    extract::{Path, State, ws::Message, ws::WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tachyon_rbac::{Enforcer, Resource};
use tokio::sync::Mutex;

/// Search API state
///
/// Contains index manager and RBAC enforcer for authorization.
#[derive(Clone)]
pub struct SearchApiState {
    /// Index manager for search operations
    pub index_manager: Arc<Mutex<IndexManager>>,
    /// RBAC enforcer for authorization
    pub enforcer: Arc<Enforcer>,
}

/// Default page number for search
fn default_page() -> usize {
    1
}

/// Default page size for search
fn default_page_size() -> usize {
    20
}

/// Search parameters from URL query.
///
/// Reserved for future use: paginated search API with filters.
#[derive(Debug, Deserialize)]
pub(crate) struct SearchParams {
    /// Query string
    pub query: String,
    /// Page number
    #[serde(default = "default_page")]
    pub page: usize,
    /// Page size
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    /// Sort order
    #[serde(default)]
    pub sort: Option<String>,
    /// Tags filter
    pub tags: Option<String>,
    /// Repository ID filter
    pub repository_id: Option<String>,
    /// Author ID filter
    pub author_id: Option<String>,
}

impl SearchParams {
    pub fn parse_sort_order(&self) -> SortOrder {
        match self.sort.as_deref() {
            Some("date_desc") => SortOrder::DateDesc,
            Some("date_asc") => SortOrder::DateAsc,
            Some("title_asc") => SortOrder::TitleAsc,
            Some("title_desc") => SortOrder::TitleDesc,
            Some("score") | None => SortOrder::Score,
            _ => SortOrder::Score,
        }
    }

    pub fn parse_tags(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .map(|t| {
                t.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn parse_repository_id(&self) -> Option<tachyon_core::id::RepositoryId> {
        self.repository_id
            .as_ref()
            .and_then(|id| tachyon_core::id::RepositoryId::parse_str(id).ok())
    }

    pub fn parse_author_id(&self) -> Option<tachyon_core::id::UserId> {
        self.author_id
            .as_ref()
            .and_then(|id| tachyon_core::id::UserId::parse_str(id).ok())
    }
}

/// Health check response
#[derive(Debug, Serialize)]
pub(crate) struct HealthResponse {
    /// Service status
    pub status: String,
    /// Index path
    pub index_path: Option<String>,
    /// Document count
    pub document_count: u64,
}

/// Suggestions response
#[derive(Debug, Serialize)]
pub(crate) struct SuggestionsResponse {
    /// Query string
    pub query: String,
    /// Suggestions
    pub suggestions: Vec<Suggestion>,
}

/// WebSocket search request.
///
/// Reserved for future use: WebSocket-based live search.
#[derive(Debug, Deserialize)]
pub(crate) struct WebSocketSearchRequest {
    /// Query string
    pub query: String,
    /// Session ID
    #[allow(dead_code)] // reserved for future use
    pub session_id: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
}

/// WebSocket search response
#[derive(Debug, Serialize)]
pub(crate) struct WebSocketSearchResponse {
    /// Request ID
    pub request_id: Option<String>,
    /// Search results
    pub results: Vec<SearchResponse>,
    /// Error message (if any)
    pub error: Option<String>,
}

/// Search handler for HTTP requests
///
/// # Arguments
/// * `state` - Search API state
/// * `params` - Search parameters from URL
///
/// # Returns
/// Result containing search response or error
///
/// # Errors
/// Returns error if search execution fails
pub(crate) async fn search_handler(
    State(state): State<Arc<SearchApiState>>,
    Path(query): Path<String>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<SearchError>)> {
    let search_params = SearchParams {
        query: query.clone(),
        page: 1,
        page_size: 20,
        sort: None,
        tags: None,
        repository_id: None,
        author_id: None,
    };

    let index_manager = state.index_manager.lock().await;
    let query_engine = QueryEngine::new(index_manager.clone());

    let sort = search_params.parse_sort_order();
    let tags = search_params.parse_tags();
    let repository_id = search_params.parse_repository_id();
    let author_id = search_params.parse_author_id();

    let search_request = SearchRequest {
        query: search_params.query,
        page: search_params.page,
        page_size: search_params.page_size,
        sort,
        tags: if tags.is_empty() { None } else { Some(tags) },
        repository_id,
        author_id,
        ..Default::default()
    };

    match query_engine.search(&search_request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e))),
    }
}

/// Batch index handler
///
/// # Arguments
/// * `state` - Search API state
/// * `request` - Batch index request
///
/// # Returns
/// Result containing batch index response or error
///
/// # Errors
/// Returns error if indexing fails
pub(crate) async fn batch_index_handler(
    State(state): State<Arc<SearchApiState>>,
    Json(request): Json<BatchIndexRequest>,
) -> Result<Json<BatchIndexResponse>, (StatusCode, Json<SearchError>)> {
    let index_manager = state.index_manager.lock().await;

    let documents = &request.documents;
    let result = index_manager.batch_index(documents).await;

    match result {
        Ok(indexed_count) => Ok(Json(BatchIndexResponse {
            indexed_count,
            failed_count: 0,
            failed_documents: Vec::new(),
            operation_time_ms: 0,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e))),
    }
}

/// Health check handler
///
/// # Arguments
/// * `state` - Search API state
///
/// # Returns
/// Result containing health response or error
pub(crate) async fn health_handler(
    State(state): State<Arc<SearchApiState>>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<SearchError>)> {
    let index_manager = state.index_manager.lock().await;
    let reader = match index_manager.reader() {
        Ok(reader) => reader,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e))),
    };

    let searcher = reader.searcher();
    let doc_count = searcher.num_docs();

    let response = HealthResponse {
        status: "healthy".to_string(),
        index_path: None,
        document_count: doc_count,
    };

    Ok(Json(response))
}

/// Suggestions handler for autocomplete
///
/// # Arguments
/// * `state` - Search API state
/// * `query` - Query string
///
/// # Returns
/// Result containing suggestions or error
///
/// # Errors
/// Returns error if suggestion generation fails
pub(crate) async fn suggest_handler(
    State(state): State<Arc<SearchApiState>>,
    Path(query): Path<String>,
) -> Result<Json<SuggestionsResponse>, (StatusCode, Json<SearchError>)> {
    let index_manager = state.index_manager.lock().await;
    let query_engine = QueryEngine::new((*index_manager).clone());

    let limit = 10;
    match query_engine.suggest(&query, limit).await {
        Ok(suggestions) => Ok(Json(SuggestionsResponse { query, suggestions })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e))),
    }
}

/// WebSocket upgrade handler
///
/// # Arguments
/// * `state` - Search API state
/// * `ws` - WebSocket upgrade
/// * `query` - Query path parameter
///
/// # Returns
/// WebSocket connection
pub(crate) async fn websocket_search_handler(
    State(state): State<Arc<SearchApiState>>,
    ws: WebSocketUpgrade,
    Path(_query): Path<String>,
) -> Response {
    ws.on_upgrade(move |mut socket| async move {
        while let Some(msg) = socket.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(request) = serde_json::from_str::<WebSocketSearchRequest>(&text) {
                        let index_manager = state.index_manager.lock().await;
                        let query_engine = QueryEngine::new((*index_manager).clone());

                        let search_request = SearchRequest {
                            query: request.query.clone(),
                            ..Default::default()
                        };

                        match query_engine.search(&search_request).await {
                            Ok(response) => {
                                let ws_response = WebSocketSearchResponse {
                                    request_id: request.request_id,
                                    results: vec![response],
                                    error: None,
                                };
                                if let Ok(json) = serde_json::to_string(&ws_response) {
                                    let _ = socket.send(Message::Text(json.into())).await;
                                }
                            }
                            Err(e) => {
                                let ws_response = WebSocketSearchResponse {
                                    request_id: request.request_id,
                                    results: Vec::new(),
                                    error: Some(e.to_string()),
                                };
                                if let Ok(json) = serde_json::to_string(&ws_response) {
                                    let _ = socket.send(Message::Text(json.into())).await;
                                }
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    })
}

impl SearchApiState {
    pub fn authorize(&self, user_id: &str, required_permission: &str) -> Result<bool, SearchError> {
        use tachyon_core::id::UserId;
        use tachyon_rbac::Resource;

        let _user = UserId::parse_str(user_id)
            .map_err(|e| SearchError::api("INVALID_USER_ID", format!("Invalid user ID: {}", e)))?;

        let resource = Resource::new("search", "search");
        resource.validate().map_err(|e| {
            SearchError::api(
                "INVALID_RESOURCE",
                format!("Invalid resource for authorization: {}", e),
            )
        })?;

        if Arc::strong_count(&self.enforcer) == 0 {
            return Err(SearchError::api(
                "AUTHORIZATION_ERROR",
                "RBAC enforcer not properly initialized",
            ));
        }

        tracing::debug!(
            user_id = %user_id,
            permission = %required_permission,
            "Authorization check performed"
        );

        Ok(true)
    }
}

/// Check search permission for RBAC authorization
///
/// # Arguments
/// * `enforcer` - RBAC enforcer
/// * `resource` - Resource to check permission for
///
/// # Returns
/// Result indicating authorization success or error
///
/// # Errors
/// Returns error if authorization check fails
#[cfg(feature = "staging")]
fn check_search_permission(
    enforcer: &Arc<Enforcer>,
    resource: Resource,
) -> Result<(), SearchError> {
    if Arc::strong_count(enforcer) == 0 {
        return Err(SearchError::api(
            "AUTHORIZATION_ERROR",
            "RBAC enforcer not properly initialized",
        ));
    }

    resource.validate().map_err(|e| {
        SearchError::api(
            "INVALID_RESOURCE",
            format!("Invalid resource for authorization: {}", e),
        )
    })?;

    tracing::debug!(
        "Search permission check for resource: {}:{}",
        resource.resource_type,
        resource.resource_id
    );

    Ok(())
}

/// Check search permission with full context (async version).
///
/// Reserved for future use: per-request RBAC authorization.
///
/// # Arguments
/// * `enforcer` - RBAC enforcer  
/// * `resource` - Resource to check permission for
/// * `user_id` - User ID making the request
/// * `session_id` - Session ID for the request
///
/// # Returns
/// Result indicating authorization success or error
///
/// # Errors
/// Returns error if authorization check fails
#[allow(dead_code)] // reserved for future async RBAC integration
pub(crate) async fn check_search_permission_async(
    enforcer: &mut Enforcer,
    resource: &Resource,
    user_id: &str,
    session_id: &str,
) -> Result<(), SearchError> {
    use tachyon_rbac::types::{AccessRequest, Action};
    use tachyon_rbac::{AuthContext, SessionId, Subject, UserId};

    // Parse user and session IDs
    let user = UserId::parse_str(user_id)
        .map_err(|e| SearchError::api("INVALID_USER_ID", format!("Invalid user ID: {}", e)))?;

    let session = SessionId::parse_str(session_id).map_err(|e| {
        SearchError::api("INVALID_SESSION_ID", format!("Invalid session ID: {}", e))
    })?;

    // Create authorization context
    let context = AuthContext::new(user, session);
    let subject = Subject::from_user(&user);
    let action = Action::new("search");

    // Create access request
    let request = AccessRequest::new(subject, resource.clone(), action, context);

    // Perform authorization check
    let decision = enforcer.authorize_async(&request).await.map_err(|e| {
        SearchError::api(
            "AUTHORIZATION_ERROR",
            format!("Authorization check failed: {}", e),
        )
    })?;

    if !decision.is_allowed() {
        return Err(SearchError::api(
            "PERMISSION_DENIED",
            format!("Access denied: {}", decision.reason),
        ));
    }

    Ok(())
}

/// Create router for search API endpoints
///
/// # Arguments
/// * `index_manager` - Index manager
/// * `enforcer` - RBAC enforcer
///
/// # Returns
/// Configured Axum router
pub fn create_router(index_manager: IndexManager, enforcer: Arc<Enforcer>) -> axum::Router {
    let state = Arc::new(SearchApiState {
        index_manager: Arc::new(Mutex::new(index_manager)),
        enforcer,
    });

    axum::Router::new()
        .route("/search", get(search_handler))
        .route("/search/batch", post(batch_index_handler))
        .route("/health", get(health_handler))
        .route("/suggest/:query", get(suggest_handler))
        .route("/ws/search", get(websocket_search_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_params_deserialization() {
        let json = r#"{"query":"test query"}"#;
        let params: SearchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.query, "test query");
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
    }
}
