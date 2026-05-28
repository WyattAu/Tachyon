// Project/Service Catalog Routes
// Backstage-like catalog API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tachyon_database::{
    CatalogRepository, Component, CreateComponentRequest, CreateProjectRequest, Project,
    ProjectMember,
};
use tracing::{debug, info, instrument};

use crate::audit::{AuditEvent, AuditEventType, AuditLogger, AuditSeverity};
use crate::error::ServerError;
use crate::pagination::{CursorPage, CursorParams};

/// Response wrapper for paginated catalog API responses.
type CatalogResult<T> = Result<(StatusCode, Json<ApiResponse<T>>), ServerError>;

/// Catalog state for request handling
#[derive(Clone)]
pub struct CatalogState {
    repo: Arc<CatalogRepository>,
    api_cache: crate::middleware::api_cache::ApiCache,
    pub audit_logger: AuditLogger,
}

impl CatalogState {
    pub fn new(pool: tachyon_database::DatabasePool) -> Self {
        Self {
            repo: Arc::new(CatalogRepository::new(pool)),
            api_cache: crate::middleware::api_cache::ApiCache::new(std::time::Duration::from_secs(
                60,
            )),
            audit_logger: AuditLogger::disabled(),
        }
    }

    pub fn repo(&self) -> &CatalogRepository {
        &self.repo
    }

    pub fn with_audit_logger(mut self, logger: AuditLogger) -> Self {
        self.audit_logger = logger;
        self
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

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CatalogCursorPage {
    pub data: Vec<Project>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub total_count: Option<i64>,
}

impl From<CursorPage<Project>> for CatalogCursorPage {
    fn from(page: CursorPage<Project>) -> Self {
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
) -> Result<(StatusCode, Json<ApiResponse<Project>>), ServerError> {
    info!("Creating project: {}", request.name);

    let id = uuid::Uuid::new_v4().to_string();
    let project = request.to_project(id);

    state.repo().create_project(&project).await.map_err(|e| {
        debug!("Failed to create project: {}", e);
        ServerError::bad_request(e.to_string())
    })?;

    info!("Project created successfully: {}", project.id);
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ProjectCreated,
                AuditSeverity::Low,
                "project_create",
                format!("Project '{}' created", project.name),
            )
            .with_target(&project.id, "project"),
        )
        .await;
    Ok((StatusCode::CREATED, Json(ApiResponse::success(project))))
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
) -> CatalogResult<Vec<Project>> {
    debug!("Listing projects");

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

    let projects = projects.map_err(|e| {
        debug!("Failed to list projects: {}", e);
        ServerError::internal(e.to_string())
    })?;

    Ok((StatusCode::OK, Json(ApiResponse::success(projects))))
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
) -> Result<(StatusCode, Json<ApiResponse<Project>>), ServerError> {
    debug!("Getting project: {}", id);

    let project = state.repo().get_project(&id).await.map_err(|e| {
        debug!("Project not found: {}", e);
        ServerError::not_found("Project", &id)
    })?;

    Ok((StatusCode::OK, Json(ApiResponse::success(project))))
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
) -> Result<(StatusCode, Json<ApiResponse<Project>>), ServerError> {
    debug!("Getting project by slug: {}", slug);

    let project = state.repo().get_project_by_slug(&slug).await.map_err(|e| {
        debug!("Project not found: {}", e);
        ServerError::not_found("Project", &slug)
    })?;

    Ok((StatusCode::OK, Json(ApiResponse::success(project))))
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
) -> Result<(StatusCode, Json<ApiResponse<Project>>), ServerError> {
    info!("Updating project: {}", id);

    request.id = id.clone();
    request.updated_at = Utc::now();

    state.repo().update_project(&request).await.map_err(|e| {
        debug!("Failed to update project: {}", e);
        ServerError::bad_request(e.to_string())
    })?;

    info!("Project updated successfully: {}", id);
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ProjectUpdated,
                AuditSeverity::Low,
                "project_update",
                format!("Project '{}' updated", id),
            )
            .with_target(&id, "project"),
        )
        .await;
    Ok((StatusCode::OK, Json(ApiResponse::success(request))))
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
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ServerError> {
    info!("Deleting project: {}", id);

    state.repo().delete_project(&id).await.map_err(|e| {
        debug!("Failed to delete project: {}", e);
        ServerError::not_found("Project", &id)
    })?;

    info!("Project deleted successfully: {}", id);
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ProjectDeleted,
                AuditSeverity::Medium,
                "project_delete",
                format!("Project '{}' deleted", id),
            )
            .with_target(&id, "project"),
        )
        .await;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({ "deleted": true }))),
    ))
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
) -> Result<(StatusCode, Json<ApiResponse<Component>>), ServerError> {
    info!(
        "Creating component: {} for project: {}",
        request.name, request.project_id
    );

    let id = uuid::Uuid::new_v4().to_string();
    let component = request.to_component(id);

    state
        .repo()
        .create_component(&component)
        .await
        .map_err(|e| {
            debug!("Failed to create component: {}", e);
            ServerError::bad_request(e.to_string())
        })?;

    info!("Component created successfully: {}", component.id);
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ComponentCreated,
                AuditSeverity::Low,
                "component_create",
                format!("Component '{}' created", component.name),
            )
            .with_target(&component.id, "component"),
        )
        .await;
    Ok((StatusCode::CREATED, Json(ApiResponse::success(component))))
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
) -> Result<(StatusCode, Json<ApiResponse<Component>>), ServerError> {
    debug!("Getting component: {}", id);

    let component = state.repo().get_component(&id).await.map_err(|e| {
        debug!("Component not found: {}", e);
        ServerError::not_found("Component", &id)
    })?;

    Ok((StatusCode::OK, Json(ApiResponse::success(component))))
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
) -> CatalogResult<Vec<Component>> {
    debug!("Listing components for project: {}", project_id);

    let components = state
        .repo()
        .list_components_by_project(&project_id)
        .await
        .map_err(|e| {
            debug!("Failed to list components: {}", e);
            ServerError::internal(e.to_string())
        })?;

    Ok((StatusCode::OK, Json(ApiResponse::success(components))))
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
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ServerError> {
    info!("Deleting component: {}", id);

    state.repo().delete_component(&id).await.map_err(|e| {
        debug!("Failed to delete component: {}", e);
        ServerError::not_found("Component", &id)
    })?;

    info!("Component deleted successfully: {}", id);
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ComponentDeleted,
                AuditSeverity::Medium,
                "component_delete",
                format!("Component '{}' deleted", id),
            )
            .with_target(&id, "component"),
        )
        .await;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({ "deleted": true }))),
    ))
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
) -> Result<(StatusCode, Json<ApiResponse<ProjectMember>>), ServerError> {
    info!(
        "Adding member {} to project {}",
        request.user_id, project_id
    );

    let member = ProjectMember {
        id: 0,
        project_id: project_id.clone(),
        user_id: request.user_id.clone(),
        role: request.role,
        added_by: request.added_by,
        added_at: Utc::now(),
    };

    state
        .repo()
        .add_project_member(&member)
        .await
        .map_err(|e| {
            debug!("Failed to add member: {}", e);
            ServerError::bad_request(e.to_string())
        })?;

    info!("Member added successfully");
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::CatalogMemberAdded,
                AuditSeverity::Low,
                "catalog_member_add",
                format!(
                    "Member '{}' added to project '{}'",
                    request.user_id, project_id
                ),
            )
            .with_target(&project_id, "project"),
        )
        .await;
    Ok((StatusCode::CREATED, Json(ApiResponse::success(member))))
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
) -> CatalogResult<Vec<ProjectMember>> {
    debug!("Listing members for project: {}", project_id);

    let members = state
        .repo()
        .list_project_members(&project_id)
        .await
        .map_err(|e| {
            debug!("Failed to list members: {}", e);
            ServerError::internal(e.to_string())
        })?;

    Ok((StatusCode::OK, Json(ApiResponse::success(members))))
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
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ServerError> {
    info!("Removing member {} from project {}", user_id, project_id);

    state
        .repo()
        .remove_project_member(&project_id, &user_id)
        .await
        .map_err(|e| {
            debug!("Failed to remove member: {}", e);
            ServerError::not_found("Member", &user_id)
        })?;

    info!("Member removed successfully");
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::CatalogMemberRemoved,
                AuditSeverity::Low,
                "catalog_member_remove",
                format!("Member '{}' removed from project '{}'", user_id, project_id),
            )
            .with_target(&project_id, "project"),
        )
        .await;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({ "removed": true }))),
    ))
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

    let stats = match state.repo().get_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            debug!("Failed to get stats: {}", e);
            return ServerError::internal(e.to_string()).into_response();
        }
    };

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

/// Request to add a member
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role: String,
    pub added_by: Option<String>,
}

#[utoipa::path(
    get,
    path = "/projects/cursor",
    params(CursorParams),
    responses(
        (status = 200, description = "Projects (cursor paginated)", body = serde_json::Value),
        (status = 500, description = "Internal server error"),
    ),
    tag = "projects",
    security(("bearer_auth" = [])),
)]
#[instrument(skip(state))]
pub async fn list_projects_cursor(
    State(state): State<CatalogState>,
    Query(params): Query<CursorParams>,
) -> CatalogResult<CatalogCursorPage> {
    let limit = params.limit();
    let direction = params.direction();
    let cursor_str = params.after.as_deref().or(params.before.as_deref());
    let fetch_limit = (limit + 1) as i64;

    let projects = state
        .repo()
        .list_projects(None, None, None, Some(fetch_limit), Some(0))
        .await
        .map_err(|e| {
            debug!("Failed to list projects (cursor): {}", e);
            ServerError::internal(e.to_string())
        })?;

    let has_extra = projects.len() > limit;
    let mut projects = projects;
    if has_extra {
        projects.truncate(limit);
    }

    let first_id = projects.first().map(|p| p.id.clone());
    let last_id = projects.last().map(|p| p.id.clone());
    let has_prev = cursor_str.is_some();

    let page = CursorPage::new(projects, has_extra, has_prev).with_cursors(
        first_id.as_deref(),
        last_id.as_deref(),
        direction,
    );

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(CatalogCursorPage::from(page))),
    ))
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
        .route("/projects/cursor", get(list_projects_cursor))
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
