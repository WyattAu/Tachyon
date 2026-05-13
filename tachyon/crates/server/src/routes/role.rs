// Role API routes
// Handles role management operations (admin only)

use crate::error::ServerError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{DatabasePool, RoleRecord, RoleRepository};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct RoleState {
    pub pool: DatabasePool,
    pub repo: RoleRepository,
}

impl RoleState {
    pub fn new(pool: DatabasePool) -> Self {
        let repo = RoleRepository::new(pool.clone());
        Self { pool, repo }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RoleResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_system: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RoleRecord> for RoleResponse {
    fn from(role: RoleRecord) -> Self {
        let permissions = role.parse_permissions().unwrap_or_default();
        Self {
            id: role.id,
            name: role.name,
            description: role.description,
            permissions,
            is_system: role.is_system,
            created_at: role.created_at.to_rfc3339(),
            updated_at: role.updated_at.to_rfc3339(),
        }
    }
}

/// List all roles.
///
/// `GET /api/v1/roles`
#[utoipa::path(
    get,
    path = "/roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<RoleResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "roles",
    security(("bearer_auth" = [])),
)]
pub async fn list_roles(
    State(state): State<RoleState>,
) -> Result<Json<Vec<RoleResponse>>, ServerError> {
    debug!("Listing roles");

    let roles = state
        .repo
        .list_all()
        .await
        .map_err(|e| ServerError::database(format!("Failed to list roles: {}", e)))?;

    Ok(Json(roles.into_iter().map(RoleResponse::from).collect()))
}

/// Get a role by ID.
///
/// `GET /api/v1/roles/{role_id}`
#[utoipa::path(
    get,
    path = "/roles/{role_id}",
    params(
        ("role_id" = i64, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role details", body = RoleResponse),
        (status = 404, description = "Role not found"),
    ),
    tag = "roles",
    security(("bearer_auth" = [])),
)]
pub async fn get_role(
    Path(role_id): Path<i64>,
    State(state): State<RoleState>,
) -> Result<Json<RoleResponse>, ServerError> {
    debug!("Getting role: {}", role_id);

    let role = state
        .repo
        .get_by_id(role_id)
        .await
        .map_err(|e| ServerError::not_found("role", &e.to_string()))?;

    Ok(Json(RoleResponse::from(role)))
}

/// Create a new role.
///
/// `POST /api/v1/roles`
///
/// Validates that the role name is 1–50 characters.
#[utoipa::path(
    post,
    path = "/roles",
    request_body(content = CreateRoleRequest, description = "Role creation request"),
    responses(
        (status = 200, description = "Role created", body = RoleResponse),
        (status = 400, description = "Invalid role name"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "roles",
    security(("bearer_auth" = [])),
)]
pub async fn create_role(
    State(state): State<RoleState>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<RoleResponse>, ServerError> {
    info!("Creating role: {}", req.name);

    if req.name.is_empty() || req.name.len() > 50 {
        return Err(ServerError::bad_request(
            "Role name must be between 1 and 50 characters",
        ));
    }

    let permissions = serde_json::to_value(&req.permissions).unwrap_or(serde_json::json!([]));
    let mut role = RoleRecord::new(req.name, permissions);
    if let Some(desc) = req.description {
        role = role.with_description(desc);
    }

    let created = state.repo.create(&role).await.map_err(|e| {
        warn!("Failed to create role: {}", e);
        ServerError::database(format!("Failed to create role: {}", e))
    })?;

    Ok(Json(RoleResponse::from(created)))
}

/// Update a role.
///
/// `PUT /api/v1/roles/{role_id}`
///
/// System roles cannot be modified.
#[utoipa::path(
    put,
    path = "/roles/{role_id}",
    params(
        ("role_id" = i64, Path, description = "Role ID"),
    ),
    request_body(content = UpdateRoleRequest, description = "Role update request"),
    responses(
        (status = 200, description = "Role updated", body = RoleResponse),
        (status = 403, description = "Cannot modify system roles"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "roles",
    security(("bearer_auth" = [])),
)]
pub async fn update_role(
    Path(role_id): Path<i64>,
    State(state): State<RoleState>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<RoleResponse>, ServerError> {
    debug!("Updating role: {}", role_id);

    let mut role = state
        .repo
        .get_by_id(role_id)
        .await
        .map_err(|e| ServerError::not_found("role", &e.to_string()))?;

    if role.is_system {
        return Err(ServerError::forbidden("Cannot modify system roles"));
    }

    if let Some(name) = req.name {
        role.name = name;
    }
    if let Some(description) = req.description {
        role.description = Some(description);
    }
    if let Some(permissions) = req.permissions {
        role.permissions = serde_json::to_value(permissions).unwrap_or(serde_json::json!([]));
    }
    role.updated_at = chrono::Utc::now();

    let updated = state
        .repo
        .update(&role)
        .await
        .map_err(|e| ServerError::database(format!("Failed to update role: {}", e)))?;

    Ok(Json(RoleResponse::from(updated)))
}

/// Delete a role.
///
/// `DELETE /api/v1/roles/{role_id}`
///
/// System roles and roles still assigned to users cannot be deleted.
#[utoipa::path(
    delete,
    path = "/roles/{role_id}",
    params(
        ("role_id" = i64, Path, description = "Role ID"),
    ),
    responses(
        (status = 204, description = "Role deleted"),
        (status = 403, description = "Cannot delete role"),
    ),
    tag = "roles",
    security(("bearer_auth" = [])),
)]
pub async fn delete_role(
    Path(role_id): Path<i64>,
    State(state): State<RoleState>,
) -> Result<StatusCode, ServerError> {
    debug!("Deleting role: {}", role_id);

    state
        .repo
        .delete(role_id)
        .await
        .map_err(|e| ServerError::forbidden(format!("Cannot delete role: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Seed default roles into the database.
///
/// `POST /api/v1/roles/seed`
///
/// Creates the built-in system roles (owner, admin, editor, viewer) if they do not exist.
#[utoipa::path(
    post,
    path = "/roles/seed",
    responses(
        (status = 200, description = "Default roles seeded", body = serde_json::Value),
        (status = 500, description = "Internal server error"),
    ),
    tag = "roles",
    security(("bearer_auth" = [])),
)]
pub async fn seed_default_roles(
    State(state): State<RoleState>,
) -> Result<Json<serde_json::Value>, ServerError> {
    info!("Seeding default roles");

    state
        .repo
        .seed_default_roles()
        .await
        .map_err(|e| ServerError::database(format!("Failed to seed roles: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Default roles seeded successfully"
    })))
}

pub fn create_role_router() -> axum::Router<RoleState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/roles", get(list_roles))
        .route("/roles", post(create_role))
        .route("/roles/seed", post(seed_default_roles))
        .route("/roles/{role_id}", get(get_role))
        .route("/roles/{role_id}", put(update_role))
        .route("/roles/{role_id}", delete(delete_role))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role_request() {
        let req = CreateRoleRequest {
            name: "custom".to_string(),
            description: Some("Custom role".to_string()),
            permissions: vec!["read".to_string(), "write".to_string()],
        };
        assert_eq!(req.name, "custom");
        assert_eq!(req.permissions.len(), 2);
    }

    #[test]
    fn test_role_response_from_record() {
        let role = RoleRecord::new("test".to_string(), serde_json::json!(["read"]));
        let response = RoleResponse::from(role);
        assert_eq!(response.name, "test");
        assert_eq!(response.permissions, vec!["read"]);
    }
}
