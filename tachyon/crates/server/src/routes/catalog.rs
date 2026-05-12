// Project/Service Catalog Routes
// Backstage-like catalog API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tachyon_database::{
    CatalogRepository, CatalogStats, Component, CreateComponentRequest, CreateProjectRequest,
    Project, ProjectMember,
};
use tracing::{debug, info, instrument};

/// Catalog state for request handling
#[derive(Clone)]
pub struct CatalogState {
    repo: Arc<CatalogRepository>,
    api_cache: crate::middleware::api_cache::ApiCache,
}

impl CatalogState {
    pub fn new(pool: tachyon_database::DatabasePool) -> Self {
        Self {
            repo: Arc::new(CatalogRepository::new(pool)),
            api_cache: crate::middleware::api_cache::ApiCache::new(std::time::Duration::from_secs(
                60,
            )),
        }
    }

    pub fn repo(&self) -> &CatalogRepository {
        &self.repo
    }
}

// ============================================================================
// API Response Types
// ============================================================================

/// Generic API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(message.into()),
        }
    }
}

/// Pagination query parameters
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Combined query parameters (pagination + filters)
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProjectListParams {
    // Pagination
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    // Filters
    pub project_type: Option<String>,
    pub owner_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

/// Project filter parameters
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProjectFilters {
    pub project_type: Option<String>,
    pub owner_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

// ============================================================================
// Project Routes
// ============================================================================

/// Create a new project
#[utoipa::path(
    post,
    path = "/projects",
    request_body(content = tachyon_database::CreateProjectRequest, description = "Project creation request"),
    responses(
        (status = 201, description = "Project created", body = serde_json::Value),
        (status = 400, description = "Bad request"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn create_project(
    State(state): State<CatalogState>,
    Json(request): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    info!("Creating project: {}", request.name);

    // Generate UUID for the project
    let id = uuid::Uuid::new_v4().to_string();
    let project = request.to_project(id);

    match state.repo().create_project(&project).await {
        Ok(()) => {
            info!("Project created successfully: {}", project.id);
            (StatusCode::CREATED, Json(ApiResponse::success(project)))
        }
        Err(e) => {
            debug!("Failed to create project: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<Project>::error(e.to_string())),
            )
        }
    }
}

/// List all projects with optional filters
#[utoipa::path(
    get,
    path = "/projects",
    params(
        ProjectListParams,
    ),
    responses(
        (status = 200, description = "List of projects", body = serde_json::Value),
        (status = 500, description = "Internal server error"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn list_projects(
    State(state): State<CatalogState>,
    Query(params): Query<ProjectListParams>,
) -> impl IntoResponse {
    debug!("Listing projects");

    // Handle search vs filter-based listing
    let projects = if let Some(search) = &params.search {
        state.repo().search_projects(search, params.limit).await
    } else {
        state
            .repo()
            .list_projects(
                params.project_type.as_deref(),
                params.owner_id.as_deref(),
                params.status.as_deref(),
                params.limit,
                params.offset,
            )
            .await
    };

    match projects {
        Ok(projects) => (StatusCode::OK, Json(ApiResponse::success(projects))),
        Err(e) => {
            debug!("Failed to list projects: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Project>>::error(e.to_string())),
            )
        }
    }
}

/// Get a project by ID
#[utoipa::path(
    get,
    path = "/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project details", body = serde_json::Value),
        (status = 404, description = "Project not found"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn get_project(
    State(state): State<CatalogState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    debug!("Getting project: {}", id);

    match state.repo().get_project(&id).await {
        Ok(project) => (StatusCode::OK, Json(ApiResponse::success(project))),
        Err(e) => {
            debug!("Project not found: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Project>::error(e.to_string())),
            )
        }
    }
}

/// Get a project by slug
#[utoipa::path(
    get,
    path = "/projects/slug/{slug}",
    params(
        ("slug" = String, Path, description = "Project slug"),
    ),
    responses(
        (status = 200, description = "Project by slug", body = serde_json::Value),
        (status = 404, description = "Project not found"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn get_project_by_slug(
    State(state): State<CatalogState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    debug!("Getting project by slug: {}", slug);

    match state.repo().get_project_by_slug(&slug).await {
        Ok(project) => (StatusCode::OK, Json(ApiResponse::success(project))),
        Err(e) => {
            debug!("Project not found: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Project>::error(e.to_string())),
            )
        }
    }
}

/// Update a project
#[utoipa::path(
    put,
    path = "/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID"),
    ),
    request_body(content = tachyon_database::Project, description = "Project update"),
    responses(
        (status = 200, description = "Project updated", body = serde_json::Value),
        (status = 400, description = "Bad request"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn update_project(
    State(state): State<CatalogState>,
    Path(id): Path<String>,
    Json(mut request): Json<Project>,
) -> impl IntoResponse {
    info!("Updating project: {}", id);

    // Ensure the ID in the path matches the request body
    request.id = id.clone();
    request.updated_at = Utc::now();

    match state.repo().update_project(&request).await {
        Ok(()) => {
            info!("Project updated successfully: {}", id);
            (StatusCode::OK, Json(ApiResponse::success(request)))
        }
        Err(e) => {
            debug!("Failed to update project: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<Project>::error(e.to_string())),
            )
        }
    }
}

/// Delete a project
#[utoipa::path(
    delete,
    path = "/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project deleted", body = serde_json::Value),
        (status = 404, description = "Project not found"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn delete_project(
    State(state): State<CatalogState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting project: {}", id);

    match state.repo().delete_project(&id).await {
        Ok(()) => {
            info!("Project deleted successfully: {}", id);
            (
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({ "deleted": true }))),
            )
        }
        Err(e) => {
            debug!("Failed to delete project: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<serde_json::Value>::error(e.to_string())),
            )
        }
    }
}

// ============================================================================
// Component Routes
// ============================================================================

/// Create a new component
#[utoipa::path(
    post,
    path = "/components",
    request_body(content = tachyon_database::CreateComponentRequest, description = "Component creation request"),
    responses(
        (status = 201, description = "Component created", body = serde_json::Value),
        (status = 400, description = "Bad request"),
    ),
    tag = "components",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn create_component(
    State(state): State<CatalogState>,
    Json(request): Json<CreateComponentRequest>,
) -> impl IntoResponse {
    info!(
        "Creating component: {} for project: {}",
        request.name, request.project_id
    );

    let id = uuid::Uuid::new_v4().to_string();
    let component = request.to_component(id);

    match state.repo().create_component(&component).await {
        Ok(()) => {
            info!("Component created successfully: {}", component.id);
            (StatusCode::CREATED, Json(ApiResponse::success(component)))
        }
        Err(e) => {
            debug!("Failed to create component: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<Component>::error(e.to_string())),
            )
        }
    }
}

/// Get a component by ID
#[utoipa::path(
    get,
    path = "/components/{id}",
    params(
        ("id" = String, Path, description = "Component ID"),
    ),
    responses(
        (status = 200, description = "Component details", body = serde_json::Value),
        (status = 404, description = "Component not found"),
    ),
    tag = "components",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn get_component(
    State(state): State<CatalogState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    debug!("Getting component: {}", id);

    match state.repo().get_component(&id).await {
        Ok(component) => (StatusCode::OK, Json(ApiResponse::success(component))),
        Err(e) => {
            debug!("Component not found: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Component>::error(e.to_string())),
            )
        }
    }
}

/// List components for a project
#[utoipa::path(
    get,
    path = "/projects/{project_id}/components",
    params(
        ("project_id" = String, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project components", body = serde_json::Value),
        (status = 500, description = "Internal server error"),
    ),
    tag = "components",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn list_project_components(
    State(state): State<CatalogState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    debug!("Listing components for project: {}", project_id);

    match state.repo().list_components_by_project(&project_id).await {
        Ok(components) => (StatusCode::OK, Json(ApiResponse::success(components))),
        Err(e) => {
            debug!("Failed to list components: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Component>>::error(e.to_string())),
            )
        }
    }
}

/// Delete a component
#[utoipa::path(
    delete,
    path = "/components/{id}",
    params(
        ("id" = String, Path, description = "Component ID"),
    ),
    responses(
        (status = 200, description = "Component deleted", body = serde_json::Value),
        (status = 404, description = "Component not found"),
    ),
    tag = "components",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn delete_component(
    State(state): State<CatalogState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting component: {}", id);

    match state.repo().delete_component(&id).await {
        Ok(()) => {
            info!("Component deleted successfully: {}", id);
            (
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({ "deleted": true }))),
            )
        }
        Err(e) => {
            debug!("Failed to delete component: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<serde_json::Value>::error(e.to_string())),
            )
        }
    }
}

// ============================================================================
// Member Routes
// ============================================================================

/// Add a member to a project
#[utoipa::path(
    post,
    path = "/projects/{project_id}/members",
    params(
        ("project_id" = String, Path, description = "Project ID"),
    ),
    request_body(content = AddMemberRequest, description = "Add member request"),
    responses(
        (status = 201, description = "Member added", body = serde_json::Value),
        (status = 400, description = "Bad request"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn add_project_member(
    State(state): State<CatalogState>,
    Path(project_id): Path<String>,
    Json(request): Json<AddMemberRequest>,
) -> impl IntoResponse {
    info!(
        "Adding member {} to project {}",
        request.user_id, project_id
    );

    let member = ProjectMember {
        id: 0, // Auto-generated
        project_id,
        user_id: request.user_id,
        role: request.role,
        added_by: request.added_by,
        added_at: Utc::now(),
    };

    match state.repo().add_project_member(&member).await {
        Ok(()) => {
            info!("Member added successfully");
            (StatusCode::CREATED, Json(ApiResponse::success(member)))
        }
        Err(e) => {
            debug!("Failed to add member: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<ProjectMember>::error(e.to_string())),
            )
        }
    }
}

/// List project members
#[utoipa::path(
    get,
    path = "/projects/{project_id}/members",
    params(
        ("project_id" = String, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project members", body = serde_json::Value),
        (status = 500, description = "Internal server error"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn list_project_members(
    State(state): State<CatalogState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    debug!("Listing members for project: {}", project_id);

    match state.repo().list_project_members(&project_id).await {
        Ok(members) => (StatusCode::OK, Json(ApiResponse::success(members))),
        Err(e) => {
            debug!("Failed to list members: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<ProjectMember>>::error(e.to_string())),
            )
        }
    }
}

/// Remove a member from a project
#[utoipa::path(
    delete,
    path = "/projects/{project_id}/members/{user_id}",
    params(
        ("project_id" = String, Path, description = "Project ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Member removed", body = serde_json::Value),
        (status = 404, description = "Member not found"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn remove_project_member(
    State(state): State<CatalogState>,
    Path((project_id, user_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Removing member {} from project {}", user_id, project_id);

    match state
        .repo()
        .remove_project_member(&project_id, &user_id)
        .await
    {
        Ok(()) => {
            info!("Member removed successfully");
            (
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({ "removed": true }))),
            )
        }
        Err(e) => {
            debug!("Failed to remove member: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<serde_json::Value>::error(e.to_string())),
            )
        }
    }
}

// ============================================================================
// Stats Route
// ============================================================================

/// Get catalog statistics
#[utoipa::path(
    get,
    path = "/catalog/stats",
    responses(
        (status = 200, description = "Catalog statistics", body = serde_json::Value),
        (status = 500, description = "Internal server error"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn get_catalog_stats(State(state): State<CatalogState>) -> axum::response::Response {
    use axum::http::{header, HeaderValue};
    use axum::response::{IntoResponse, Response};

    debug!("Getting catalog statistics");

    let key = crate::middleware::api_cache::cache_key("GET", "/api/v1/catalog/stats", None);

    if let Some(hit) = state.api_cache.get_response(&key).await {
        let mut response = Response::new(axum::body::Body::from(hit.data));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response
            .headers_mut()
            .insert("X-Cache-Status", HeaderValue::from_static("HIT"));
        return response;
    }

    match state.repo().get_stats().await {
        Ok(stats) => {
            if let Ok(bytes) = serde_json::to_vec(&ApiResponse::success(&stats)) {
                state
                    .api_cache
                    .set_response(&key, bytes, "application/json", None)
                    .await;
            }
            let mut response = (StatusCode::OK, Json(ApiResponse::success(stats))).into_response();
            response
                .headers_mut()
                .insert("X-Cache-Status", HeaderValue::from_static("MISS"));
            response
        }
        Err(e) => {
            debug!("Failed to get stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<CatalogStats>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

/// Request to add a member
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role: String,
    pub added_by: Option<String>,
}

// ============================================================================
// Router Factory
// ============================================================================

use axum::{
    routing::{delete, get, post, put},
    Router,
};

/// Create the catalog router
pub fn create_catalog_router() -> Router<CatalogState> {
    Router::new()
        // Projects
        .route("/projects", post(create_project))
        .route("/projects", get(list_projects))
        .route("/projects/{id}", get(get_project))
        .route("/projects/{id}", put(update_project))
        .route("/projects/{id}", delete(delete_project))
        .route("/projects/slug/{slug}", get(get_project_by_slug))
        // Components
        .route("/components", post(create_component))
        .route("/components/{id}", get(get_component))
        .route("/components/{id}", delete(delete_component))
        .route(
            "/projects/{project_id}/components",
            get(list_project_components),
        )
        // Members
        .route("/projects/{project_id}/members", post(add_project_member))
        .route("/projects/{project_id}/members", get(list_project_members))
        .route(
            "/projects/{project_id}/members/{user_id}",
            delete(remove_project_member),
        )
        // Stats
        .route("/catalog/stats", get(get_catalog_stats))
}
