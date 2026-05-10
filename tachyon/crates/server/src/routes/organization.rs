// Organization Routes
// REST API endpoints for organization (multi-tenant) management

use crate::middleware::AuthContext;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{
    AddOrganizationMemberRequest, CreateOrganizationRequest, DatabasePool, OrganizationRepository,
    UpdateOrganizationMemberRequest, UpdateOrganizationRequest,
};
use tracing::info;

#[derive(Clone)]
pub struct OrganizationState {
    pub pool: DatabasePool,
}

// ============================================================================
// Response / Request Types
// ============================================================================

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: String,
    pub logo_url: Option<String>,
    pub owner_id: String,
    pub default_role: String,
    pub max_members: i32,
    pub is_personal: bool,
    pub member_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationMemberResponse {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateOrganizationBody {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub logo_url: Option<String>,
    pub default_role: Option<String>,
    pub max_members: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateOrganizationBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub logo_url: Option<String>,
    pub default_role: Option<String>,
    pub max_members: Option<i32>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct OrganizationQuery {
    pub user_id: Option<String>,
    pub include_personal: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddMemberBody {
    pub user_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateMemberBody {
    pub role: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// List organizations for a user.
///
/// `GET /api/v1/organizations`
///
/// Uses the authenticated user's ID by default. Supports `user_id`, `include_personal`,
/// `limit`, and `offset` query parameters.
#[utoipa::path(
    get,
    path = "/organizations",
    params(
        OrganizationQuery,
    ),
    responses(
        (status = 200, description = "List organizations", body = Vec<OrganizationResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn list_organizations(
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<OrganizationQuery>,
    State(state): State<OrganizationState>,
) -> Result<Json<Vec<OrganizationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = query.user_id.as_deref().unwrap_or(&auth.user_id);
    let include_personal = query.include_personal.unwrap_or(true);

    let repo = OrganizationRepository::new(state.pool.clone());
    let orgs = repo
        .list_for_user(user_id, include_personal, query.limit, query.offset)
        .await
        .map_err(|e| {
            server_error(
                "QUERY_ERROR",
                &format!("Failed to list organizations: {}", e),
            )
        })?;

    let member_counts = repo
        .count_members_batch(&orgs.iter().map(|o| o.id.clone()).collect::<Vec<_>>())
        .await
        .unwrap_or_default();

    let responses: Vec<OrganizationResponse> = orgs
        .into_iter()
        .map(|org| {
            let member_count = member_counts.get(&org.id).copied().unwrap_or(0);
            OrganizationResponse::from_org(org, member_count)
        })
        .collect();

    Ok(Json(responses))
}

/// Get a single organization by ID.
///
/// `GET /api/v1/organizations/{id}`
#[utoipa::path(
    get,
    path = "/organizations/{id}",
    params(
        ("id" = String, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Organization details", body = OrganizationResponse),
        (status = 404, description = "Organization not found"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn get_organization(
    Path(id): Path<String>,
    State(state): State<OrganizationState>,
) -> Result<Json<OrganizationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OrganizationRepository::new(state.pool.clone());
    let org = repo
        .get_by_id(&id)
        .await
        .map_err(|e| not_found(&format!("Organization not found: {}", e)))?;

    let member_count = repo.count_members(&org.id).await.unwrap_or(0);
    Ok(Json(OrganizationResponse::from_org(org, member_count)))
}

/// Create a new organization.
///
/// `POST /api/v1/organizations`
///
/// The authenticated user becomes the organization owner.
#[utoipa::path(
    post,
    path = "/organizations",
    request_body(content = CreateOrganizationBody, description = "Organization creation request"),
    responses(
        (status = 200, description = "Organization created", body = OrganizationResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn create_organization(
    Extension(auth): Extension<AuthContext>,
    State(state): State<OrganizationState>,
    Json(body): Json<CreateOrganizationBody>,
) -> Result<Json<OrganizationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let owner_id = &auth.user_id;

    let repo = OrganizationRepository::new(state.pool.clone());
    let org = repo
        .create(
            owner_id,
            CreateOrganizationRequest {
                name: body.name,
                description: body.description,
                icon: body.icon,
                logo_url: body.logo_url,
                default_role: body.default_role,
                max_members: body.max_members,
            },
        )
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate") {
                bad_request("DUPLICATE", &format!("Organization creation failed: {}", e))
            } else {
                server_error(
                    "CREATE_ERROR",
                    &format!("Failed to create organization: {}", e),
                )
            }
        })?;

    let member_count = repo.count_members(&org.id).await.unwrap_or(0);
    info!("Organization created: {} ({})", org.name, org.slug);
    Ok(Json(OrganizationResponse::from_org(org, member_count)))
}

/// Update an organization.
///
/// `PUT /api/v1/organizations/{id}`
///
/// Accepts partial updates for name, description, icon, logo, default role, max members, and settings.
#[utoipa::path(
    put,
    path = "/organizations/{id}",
    params(
        ("id" = String, Path, description = "Organization ID"),
    ),
    request_body(content = UpdateOrganizationBody, description = "Organization update request"),
    responses(
        (status = 200, description = "Organization updated", body = OrganizationResponse),
        (status = 404, description = "Organization not found"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn update_organization(
    Path(id): Path<String>,
    State(state): State<OrganizationState>,
    Json(body): Json<UpdateOrganizationBody>,
) -> Result<Json<OrganizationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OrganizationRepository::new(state.pool.clone());
    let org = repo
        .update(
            &id,
            UpdateOrganizationRequest {
                name: body.name,
                description: body.description,
                icon: body.icon,
                logo_url: body.logo_url,
                default_role: body.default_role,
                max_members: body.max_members,
                settings: body.settings,
            },
        )
        .await
        .map_err(|e| not_found(&format!("Organization not found: {}", e)))?;

    let member_count = repo.count_members(&org.id).await.unwrap_or(0);
    info!("Organization updated: {}", id);
    Ok(Json(OrganizationResponse::from_org(org, member_count)))
}

/// Delete an organization.
///
/// `DELETE /api/v1/organizations/{id}`
///
/// Cannot delete the personal organization.
#[utoipa::path(
    delete,
    path = "/organizations/{id}",
    params(
        ("id" = String, Path, description = "Organization ID"),
    ),
    responses(
        (status = 204, description = "Organization deleted"),
        (status = 400, description = "Cannot delete personal org"),
        (status = 404, description = "Organization not found"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn delete_organization(
    Path(id): Path<String>,
    State(state): State<OrganizationState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = OrganizationRepository::new(state.pool.clone());
    repo.delete(&id).await.map_err(|e| {
        if e.to_string().contains("personal") {
            bad_request(
                "CANNOT_DELETE_PERSONAL",
                "Cannot delete the personal organization",
            )
        } else {
            not_found(&format!("Organization not found: {}", e))
        }
    })?;

    info!("Organization deleted: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

// -- Member management --

/// List members of an organization.
///
/// `GET /api/v1/organizations/{org_id}/members`
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/members",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Organization members", body = Vec<OrganizationMemberResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn list_members(
    Path(org_id): Path<String>,
    Query(query): Query<OrganizationQuery>,
    State(state): State<OrganizationState>,
) -> Result<Json<Vec<OrganizationMemberResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OrganizationRepository::new(state.pool.clone());
    let members = repo
        .list_members(&org_id, query.limit, query.offset)
        .await
        .map_err(|e| server_error("QUERY_ERROR", &format!("Failed to list members: {}", e)))?;

    Ok(Json(
        members
            .into_iter()
            .map(OrganizationMemberResponse::from)
            .collect(),
    ))
}

/// Add a member to an organization.
///
/// `POST /api/v1/organizations/{org_id}/members`
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/members",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
    ),
    request_body(content = AddMemberBody, description = "Add member request"),
    responses(
        (status = 200, description = "Member added", body = OrganizationMemberResponse),
        (status = 400, description = "Already a member"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn add_member(
    Path(org_id): Path<String>,
    State(state): State<OrganizationState>,
    Json(body): Json<AddMemberBody>,
) -> Result<Json<OrganizationMemberResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OrganizationRepository::new(state.pool.clone());
    let member = repo
        .add_member(
            &org_id,
            AddOrganizationMemberRequest {
                user_id: body.user_id,
                role: body.role,
            },
        )
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate") {
                bad_request(
                    "ALREADY_MEMBER",
                    "User is already a member of this organization",
                )
            } else {
                server_error("ADD_MEMBER_ERROR", &format!("Failed to add member: {}", e))
            }
        })?;

    info!(
        "Member added to organization {}: {}",
        org_id, member.user_id
    );
    Ok(Json(OrganizationMemberResponse::from(member)))
}

/// Update a member's role in an organization.
///
/// `PUT /api/v1/organizations/{org_id}/members/{user_id}`
#[utoipa::path(
    put,
    path = "/organizations/{org_id}/members/{user_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body(content = UpdateMemberBody, description = "Update member role"),
    responses(
        (status = 200, description = "Member role updated", body = OrganizationMemberResponse),
        (status = 404, description = "Member not found"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn update_member(
    Path((org_id, user_id)): Path<(String, String)>,
    State(state): State<OrganizationState>,
    Json(body): Json<UpdateMemberBody>,
) -> Result<Json<OrganizationMemberResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OrganizationRepository::new(state.pool.clone());
    let role = body.role.clone();
    let member = repo
        .update_member(
            &org_id,
            &user_id,
            UpdateOrganizationMemberRequest { role: body.role },
        )
        .await
        .map_err(|e| not_found(&format!("Member not found: {}", e)))?;

    info!(
        "Member {} role updated to {} in org {}",
        user_id, role, org_id
    );
    Ok(Json(OrganizationMemberResponse::from(member)))
}

/// Remove a member from an organization.
///
/// `DELETE /api/v1/organizations/{org_id}/members/{user_id}`
#[utoipa::path(
    delete,
    path = "/organizations/{org_id}/members/{user_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 404, description = "Member not found"),
    ),
    tag = "organizations",
    security(("bearer_auth" = [])),
)]
pub async fn remove_member(
    Path((org_id, user_id)): Path<(String, String)>,
    State(state): State<OrganizationState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = OrganizationRepository::new(state.pool.clone());
    repo.remove_member(&org_id, &user_id)
        .await
        .map_err(|e| not_found(&format!("Member not found: {}", e)))?;

    info!("Member {} removed from organization {}", user_id, org_id);
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Router
// ============================================================================

pub fn create_organization_router() -> axum::Router<OrganizationState> {
    use axum::routing::{get, put};

    axum::Router::new()
        .route(
            "/organizations",
            get(list_organizations).post(create_organization),
        )
        .route(
            "/organizations/{id}",
            get(get_organization)
                .put(update_organization)
                .delete(delete_organization),
        )
        .route(
            "/organizations/{org_id}/members",
            get(list_members).post(add_member),
        )
        .route(
            "/organizations/{org_id}/members/{user_id}",
            put(update_member).delete(remove_member),
        )
}

// ============================================================================
// Conversions
// ============================================================================

impl OrganizationResponse {
    fn from_org(org: tachyon_database::Organization, member_count: i64) -> Self {
        Self {
            id: org.id,
            name: org.name,
            slug: org.slug,
            description: org.description,
            icon: org.icon,
            logo_url: org.logo_url,
            owner_id: org.owner_id,
            default_role: org.default_role,
            max_members: org.max_members,
            is_personal: org.is_personal,
            member_count,
            created_at: org.created_at.to_rfc3339(),
            updated_at: org.updated_at.to_rfc3339(),
        }
    }
}

impl From<tachyon_database::OrganizationMember> for OrganizationMemberResponse {
    fn from(m: tachyon_database::OrganizationMember) -> Self {
        Self {
            id: m.id,
            organization_id: m.organization_id,
            user_id: m.user_id,
            role: m.role,
            username: m.username,
            display_name: m.display_name,
            avatar_url: m.avatar_url,
            joined_at: m.joined_at.to_rfc3339(),
        }
    }
}

// ============================================================================
// Error helpers
// ============================================================================

fn server_error(code: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

fn not_found(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: message.to_string(),
        }),
    )
}

fn bad_request(code: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_organization_body_deserialization() {
        let body = CreateOrganizationBody {
            name: "My Org".to_string(),
            description: Some("A test org".to_string()),
            icon: Some("building".to_string()),
            logo_url: None,
            default_role: Some("editor".to_string()),
            max_members: Some(50),
        };
        assert_eq!(body.name, "My Org");
        assert_eq!(body.default_role.as_deref(), Some("editor"));
    }

    #[test]
    fn test_organization_response_serialization() {
        let resp = OrganizationResponse {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Test Org".to_string(),
            slug: "test-org".to_string(),
            description: Some("A test".to_string()),
            icon: "building".to_string(),
            logo_url: None,
            owner_id: "00000000-0000-0000-0000-000000000001".to_string(),
            default_role: "viewer".to_string(),
            max_members: -1,
            is_personal: false,
            member_count: 5,
            created_at: "2026-04-17T00:00:00+00:00".to_string(),
            updated_at: "2026-04-17T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("test-org"));
        assert!(json.contains("\"member_count\":5"));
        assert!(json.contains("\"is_personal\":false"));
    }

    #[test]
    fn test_member_response_serialization() {
        let resp = OrganizationMemberResponse {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            organization_id: "00000000-0000-0000-0000-000000000001".to_string(),
            user_id: "00000000-0000-0000-0000-000000000002".to_string(),
            role: "editor".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            joined_at: "2026-04-17T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("editor"));
        assert!(json.contains("testuser"));
    }
}
