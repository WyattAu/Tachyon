// Search API Routes
// Full-text search with faceted filtering and saved searches

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tachyon_core::id::{DocumentId, RepositoryId, UserId};
use tachyon_database::{
    DatabasePool, DocumentRepository, SearchRepository, SavedSearchRepository,
    CreateSavedSearchRequest, UpdateSavedSearchRequest, SavedSearch,
    SearchFilters,
};
use tachyon_search::{IndexManager, QueryEngine, ResultAggregator, SearchDocument, SearchRequest, SearchResponseItem};
use tracing::{info, warn};

#[derive(Clone)]
pub struct SearchState {
    pub pool: DatabasePool,
    pub search_repo: SearchRepository,
    pub saved_search_repo: SavedSearchRepository,
    pub index_manager: Option<Arc<Mutex<IndexManager>>>,
}

impl SearchState {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            search_repo: SearchRepository::new(pool.clone()),
            saved_search_repo: SavedSearchRepository::new(pool.clone()),
            pool,
            index_manager: None,
        }
    }

    pub fn with_index_manager(mut self, index_manager: Arc<Mutex<IndexManager>>) -> Self {
        self.index_manager = Some(index_manager);
        self
    }

    pub async fn index_document(&self, doc: SearchDocument) {
        if let Some(ref mgr) = self.index_manager {
            let guard = mgr.lock().await;
            if let Err(e) = guard.index_document(&doc).await {
                warn!("Failed to index document in Tantivy: {}", e);
            }
        }
    }

    pub async fn delete_from_index(&self, doc_id: &str) {
        if let Some(ref mgr) = self.index_manager {
            let guard = mgr.lock().await;
            if let Err(e) = guard.delete_document(doc_id).await {
                warn!("Failed to delete document from Tantivy: {}", e);
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub project_id: Option<String>,
    pub author_id: Option<String>,
    pub tags: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct SearchResultsResponse {
    pub results: Vec<SearchResultItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub facets: SearchFacetsResponse,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub id: String,
    pub title: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub visibility: String,
    pub tags: Vec<String>,
    pub author_id: String,
    pub project_id: Option<String>,
    pub word_count: i32,
    pub rank: f64,
    pub headline: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SearchFacetsResponse {
    pub content_types: Vec<FacetItem>,
    pub statuses: Vec<FacetItem>,
    pub visibilities: Vec<FacetItem>,
    pub tags: Vec<FacetItem>,
}

#[derive(Debug, Serialize)]
pub struct FacetItem {
    pub value: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct GlobalSearchResultsResponse {
    pub documents: SearchResultsResponse,
    pub projects: Vec<ProjectSearchResultItem>,
}

#[derive(Debug, Serialize)]
pub struct ProjectSearchResultItem {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub project_type: String,
    pub status: String,
    pub rank: f64,
}

#[derive(Debug, Serialize)]
pub struct SavedSearchResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<SavedSearch> for SavedSearchResponse {
    fn from(s: SavedSearch) -> Self {
        let filters = s.parse_filters().ok().flatten();
        Self {
            id: s.id,
            user_id: s.user_id,
            name: s.name,
            query: s.query,
            filters,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSavedSearchBody {
    pub name: String,
    pub query: String,
    pub filters: Option<SearchFilters>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavedSearchBody {
    pub name: Option<String>,
    pub query: Option<String>,
    pub filters: Option<SearchFilters>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

async fn search_tantivy(
    state: &SearchState,
    query: &str,
    page: usize,
    page_size: usize,
) -> Option<Vec<SearchResponseItem>> {
    let mgr_arc = state.index_manager.as_ref()?;
    let guard = mgr_arc.lock().await;
    let engine = QueryEngine::new(guard.clone());
    drop(guard);

    let request = SearchRequest::new(query)
        .with_pagination(page, page_size);

    match engine.search(&request).await {
        Ok(response) => Some(response.results),
        Err(e) => {
            warn!("Tantivy search failed: {}", e);
            None
        }
    }
}

pub async fn search(
    Query(query): Query<SearchQuery>,
    State(state): State<SearchState>,
) -> Result<Json<SearchResultsResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Search request: q='{}', page={}, page_size={}", query.q, query.page, query.page_size);

    let filters = SearchFilters {
        content_type: query.content_type,
        status: query.status,
        visibility: query.visibility,
        project_id: query.project_id,
        author_id: query.author_id,
        tags: query.tags.as_ref().map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
        date_from: query.date_from.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|d| d.with_timezone(&chrono::Utc))),
        date_to: query.date_to.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|d| d.with_timezone(&chrono::Utc))),
    };

    let page = query.page.max(1);
    let page_size = query.page_size.min(100).max(1);
    let fetch_limit = (page_size * 3).min(300) as i64;

    match state.search_repo.search(&query.q, &filters, 1, fetch_limit).await {
        Ok(pg_response) => {
            let total = pg_response.total;
            let facets_source = pg_response.facets;
            let pg_results = pg_response.results;

            let pg_metadata_map: HashMap<String, &_> = pg_results.iter()
                .map(|r| (r.document.id.clone(), r))
                .collect();

            let pg_items: Vec<SearchResponseItem> = pg_results.iter().map(|r| {
                SearchResponseItem {
                    document_id: DocumentId::parse_str(&r.document.id).unwrap_or_default(),
                    title: r.document.title.clone(),
                    snippet: r.headline.clone().unwrap_or_default(),
                    score: r.rank as f32,
                    highlights: Vec::new(),
                    author_id: UserId::parse_str(&r.document.author_id).unwrap_or_default(),
                    repository_id: r.document.project_id.as_ref()
                        .and_then(|id| RepositoryId::parse_str(id).ok()),
                    tags: r.document.parse_tags().unwrap_or_default(),
                    created_at: r.document.created_at,
                }
            }).collect();

            let tantivy_items = search_tantivy(&state, &query.q, 1, fetch_limit as usize).await
                .unwrap_or_default();

            let aggregator = ResultAggregator::default();
            let fused = aggregator.fuse_results(vec![pg_items, tantivy_items]);

            let results: Vec<SearchResultItem> = fused.into_iter().map(|item| {
                let id_str = item.document_id.to_string();
                if let Some(pg_r) = pg_metadata_map.get(&id_str) {
                    let tags = pg_r.document.parse_tags().unwrap_or_default();
                    SearchResultItem {
                        id: pg_r.document.id.clone(),
                        title: pg_r.document.title.clone(),
                        slug: pg_r.document.slug.clone(),
                        description: pg_r.document.description.clone(),
                        status: pg_r.document.status.clone(),
                        visibility: pg_r.document.visibility.clone(),
                        tags,
                        author_id: pg_r.document.author_id.clone(),
                        project_id: pg_r.document.project_id.clone(),
                        word_count: pg_r.document.word_count,
                        rank: item.score as f64,
                        headline: pg_r.headline.clone(),
                        created_at: pg_r.document.created_at.to_rfc3339(),
                        updated_at: pg_r.document.updated_at.to_rfc3339(),
                    }
                } else {
                    SearchResultItem {
                        id: id_str,
                        title: item.title,
                        slug: None,
                        description: if item.snippet.is_empty() { None } else { Some(item.snippet) },
                        status: "draft".to_string(),
                        visibility: "private".to_string(),
                        tags: item.tags,
                        author_id: item.author_id.to_string(),
                        project_id: item.repository_id.map(|id| id.to_string()),
                        word_count: 0,
                        rank: item.score as f64,
                        headline: if item.highlights.is_empty() { None } else { Some(item.highlights.join(" ")) },
                        created_at: item.created_at.to_rfc3339(),
                        updated_at: item.created_at.to_rfc3339(),
                    }
                }
            }).collect();

            let start = ((page - 1) * page_size) as usize;
            let paginated: Vec<SearchResultItem> = results.into_iter()
                .skip(start)
                .take(page_size as usize)
                .collect();

            let facets = SearchFacetsResponse {
                content_types: facets_source.content_types.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                statuses: facets_source.statuses.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                visibilities: facets_source.visibilities.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                tags: facets_source.tags.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
            };

            Ok(Json(SearchResultsResponse {
                results: paginated,
                total,
                page,
                page_size,
                facets,
            }))
        }
        Err(e) => {
            warn!("Search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SEARCH_ERROR".to_string(),
                    message: format!("Search failed: {}", e),
                }),
            ))
        }
    }
}

pub async fn global_search(
    Query(query): Query<SearchQuery>,
    State(state): State<SearchState>,
) -> Result<Json<GlobalSearchResultsResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Global search request: q='{}'", query.q);

    let filters = SearchFilters {
        content_type: query.content_type,
        status: query.status,
        visibility: query.visibility,
        project_id: query.project_id,
        author_id: query.author_id,
        tags: query.tags.as_ref().map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
        date_from: query.date_from.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|d| d.with_timezone(&chrono::Utc))),
        date_to: query.date_to.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|d| d.with_timezone(&chrono::Utc))),
    };

    let page = query.page.max(1);
    let page_size = query.page_size.min(100).max(1);

    match state.search_repo.global_search(&query.q, &filters, page, page_size).await {
        Ok(response) => {
            let doc_results: Vec<SearchResultItem> = response
                .documents
                .results
                .into_iter()
                .map(|r| {
                    let tags = r.document.parse_tags().unwrap_or_default();
                    SearchResultItem {
                        id: r.document.id,
                        title: r.document.title,
                        slug: r.document.slug,
                        description: r.document.description,
                        status: r.document.status,
                        visibility: r.document.visibility,
                        tags,
                        author_id: r.document.author_id,
                        project_id: r.document.project_id,
                        word_count: r.document.word_count,
                        rank: r.rank,
                        headline: r.headline,
                        created_at: r.document.created_at.to_rfc3339(),
                        updated_at: r.document.updated_at.to_rfc3339(),
                    }
                })
                .collect();

            let facets = SearchFacetsResponse {
                content_types: response.documents.facets.content_types.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                statuses: response.documents.facets.statuses.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                visibilities: response.documents.facets.visibilities.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                tags: response.documents.facets.tags.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
            };

            let projects: Vec<ProjectSearchResultItem> = response
                .projects
                .into_iter()
                .map(|p| ProjectSearchResultItem {
                    id: p.id,
                    name: p.name,
                    slug: p.slug,
                    description: p.description,
                    project_type: p.project_type,
                    status: p.status,
                    rank: p.rank,
                })
                .collect();

            Ok(Json(GlobalSearchResultsResponse {
                documents: SearchResultsResponse {
                    results: doc_results,
                    total: response.documents.total,
                    page: response.documents.page,
                    page_size: response.documents.page_size,
                    facets,
                },
                projects,
            }))
        }
        Err(e) => {
            warn!("Global search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SEARCH_ERROR".to_string(),
                    message: format!("Search failed: {}", e),
                }),
            ))
        }
    }
}

pub async fn create_saved_search(
    State(state): State<SearchState>,
    Json(body): Json<CreateSavedSearchBody>,
) -> Result<Json<SavedSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating saved search: {}", body.name);

    let user_id = tachyon_core::generate_user_id();

    let request = CreateSavedSearchRequest {
        user_id: user_id.to_string(),
        name: body.name,
        query: body.query,
        filters: body.filters,
    };

    match state.saved_search_repo.create(request).await {
        Ok(saved) => Ok(Json(SavedSearchResponse::from(saved))),
        Err(e) => {
            warn!("Failed to create saved search: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "CREATE_ERROR".to_string(),
                    message: format!("Failed to create saved search: {}", e),
                }),
            ))
        }
    }
}

pub async fn list_saved_searches(
    State(state): State<SearchState>,
) -> Result<Json<Vec<SavedSearchResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = tachyon_core::generate_user_id();

    match state.saved_search_repo.list_by_user(&user_id.to_string()).await {
        Ok(searches) => {
            let response: Vec<SavedSearchResponse> = searches.into_iter().map(SavedSearchResponse::from).collect();
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Failed to list saved searches: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "QUERY_ERROR".to_string(),
                    message: format!("Failed to list saved searches: {}", e),
                }),
            ))
        }
    }
}

pub async fn get_saved_search(
    Path(id): Path<String>,
    State(state): State<SearchState>,
) -> Result<Json<SavedSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.saved_search_repo.get_by_id(&id).await {
        Ok(saved) => Ok(Json(SavedSearchResponse::from(saved))),
        Err(e) => {
            warn!("Failed to get saved search: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Saved search not found: {}", id),
                }),
            ))
        }
    }
}

pub async fn update_saved_search(
    Path(id): Path<String>,
    State(state): State<SearchState>,
    Json(body): Json<UpdateSavedSearchBody>,
) -> Result<Json<SavedSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Updating saved search: {}", id);

    let request = UpdateSavedSearchRequest {
        name: body.name,
        query: body.query,
        filters: body.filters,
    };

    match state.saved_search_repo.update(&id, request).await {
        Ok(saved) => Ok(Json(SavedSearchResponse::from(saved))),
        Err(e) => {
            warn!("Failed to update saved search: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "UPDATE_ERROR".to_string(),
                    message: format!("Failed to update saved search: {}", e),
                }),
            ))
        }
    }
}

pub async fn delete_saved_search(
    Path(id): Path<String>,
    State(state): State<SearchState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting saved search: {}", id);

    match state.saved_search_repo.delete(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            warn!("Failed to delete saved search: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Saved search not found: {}", id),
                }),
            ))
        }
    }
}

pub async fn reindex_tantivy(
    State(state): State<SearchState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Starting Tantivy reindex");

    let mgr_arc = state.index_manager.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                code: "NO_INDEX".to_string(),
                message: "Tantivy search index is not available".to_string(),
            }),
        )
    })?;

    let doc_repo = DocumentRepository::new(state.pool.clone());
    let documents = doc_repo.list_all(None, None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DB_ERROR".to_string(),
                message: format!("Failed to fetch documents: {}", e),
            }),
        )
    })?;

    let search_docs: Vec<SearchDocument> = documents.iter().filter_map(|m| {
        let doc_id = DocumentId::parse_str(&m.id).ok()?;
        let author_id = UserId::parse_str(&m.author_id).ok()?;
        Some(SearchDocument {
            id: doc_id,
            title: m.title.clone(),
            content: m.content.clone().unwrap_or_default(),
            author_id,
            repository_id: m.project_id.as_ref().and_then(|id| RepositoryId::parse_str(id).ok()),
            tags: m.parse_tags().unwrap_or_default(),
            created_at: m.created_at,
            updated_at: m.updated_at,
            custom_fields: HashMap::new(),
        })
    }).collect();

    let total = search_docs.len();

    let guard = mgr_arc.lock().await;
    if let Err(e) = guard.clear_index().await {
        warn!("Failed to clear Tantivy index: {}", e);
    }

    let indexed = guard.batch_index(&search_docs).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "INDEX_ERROR".to_string(),
                message: format!("Failed to reindex documents: {}", e),
            }),
        )
    })?;

    info!("Tantivy reindex complete: {}/{} documents indexed", indexed, total);

    Ok(Json(serde_json::json!({
        "indexed": indexed,
        "total": total,
        "status": "success",
    })))
}

#[derive(Debug, Serialize)]
pub struct SuggestResponse {
    pub query: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SuggestQuery {
    pub q: String,
    #[serde(default = "default_suggest_limit")]
    pub limit: usize,
}

fn default_suggest_limit() -> usize {
    10
}

pub async fn suggest(
    Query(query): Query<SuggestQuery>,
    State(state): State<SearchState>,
) -> Result<Json<SuggestResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mgr_arc = state.index_manager.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                code: "NO_INDEX".to_string(),
                message: "Tantivy search index is not available".to_string(),
            }),
        )
    })?;

    let guard = mgr_arc.lock().await;
    let engine = QueryEngine::new(guard.clone());
    drop(guard);

    let suggestions = engine.suggest(&query.q, query.limit).await.unwrap_or_default();

    Ok(Json(SuggestResponse {
        query: query.q,
        suggestions,
    }))
}

pub fn create_search_router() -> axum::Router<SearchState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/search", get(search))
        .route("/search/global", get(global_search))
        .route("/search/suggest", get(suggest))
        .route("/search/reindex", post(reindex_tantivy))
        .route("/search/saved", post(create_saved_search))
        .route("/search/saved", get(list_saved_searches))
        .route("/search/saved/{id}", get(get_saved_search))
        .route("/search/saved/{id}", put(update_saved_search))
        .route("/search/saved/{id}", delete(delete_saved_search))
}
