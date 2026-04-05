// Search API Routes
// Full-text search with faceted filtering and saved searches

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tachyon_database::{
    DatabasePool, SearchRepository, SavedSearchRepository,
    CreateSavedSearchRequest, UpdateSavedSearchRequest, SavedSearch,
    SearchFilters, GlobalSearchResponse,
};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct SearchState {
    pub pool: DatabasePool,
    pub search_repo: SearchRepository,
    pub saved_search_repo: SavedSearchRepository,
}

impl SearchState {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            search_repo: SearchRepository::new(pool.clone()),
            saved_search_repo: SavedSearchRepository::new(pool.clone()),
            pool,
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

    match state.search_repo.search(&query.q, &filters, page, page_size).await {
        Ok(response) => {
            let results: Vec<SearchResultItem> = response
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
                content_types: response.facets.content_types.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                statuses: response.facets.statuses.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                visibilities: response.facets.visibilities.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
                tags: response.facets.tags.into_iter().map(|f| FacetItem { value: f.value, count: f.count }).collect(),
            };

            Ok(Json(SearchResultsResponse {
                results,
                total: response.total,
                page: response.page,
                page_size: response.page_size,
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

pub fn create_search_router() -> axum::Router<SearchState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/search", get(search))
        .route("/search/global", get(global_search))
        .route("/search/saved", post(create_saved_search))
        .route("/search/saved", get(list_saved_searches))
        .route("/search/saved/{id}", get(get_saved_search))
        .route("/search/saved/{id}", put(update_saved_search))
        .route("/search/saved/{id}", delete(delete_saved_search))
}
