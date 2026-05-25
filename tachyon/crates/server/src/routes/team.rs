// Team API routes
// Handles team management operations

use crate::audit::{AuditEvent, AuditEventType, AuditLogger, AuditSeverity};
use crate::error::ServerError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{DatabasePool, RoleRepository, Team, TeamMember, TeamRepository};
use tracing::{debug, info};

#[derive(Clone)]
pub struct TeamState {
    pub pool: DatabasePool,
    pub team_repo: TeamRepository,
    pub role_repo: RoleRepository,
    pub audit_logger: AuditLogger,
}

impl TeamState {
    pub fn new(pool: DatabasePool) -> Self {
        let team_repo = TeamRepository::new(pool.clone());
        let role_repo = RoleRepository::new(pool.clone());
        Self {
            pool,
            team_repo,
            role_repo,
            audit_logger: AuditLogger::disabled(),
        }
    }

    pub fn with_audit_logger(mut self, logger: AuditLogger) -> Self {
        self.audit_logger = logger;
        self
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateTeamRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role_name: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateMemberRequest {
    pub role_name: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TeamQuery {
    pub owner_id: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TeamResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub member_count: Option<i64>,
}

impl From<Team> for TeamResponse {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
            slug: team.slug,
            description: team.description,
            owner_id: team.owner_id,
            avatar_url: team.avatar_url,
            created_at: team.created_at.to_rfc3339(),
            updated_at: team.updated_at.to_rfc3339(),
            member_count: None,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TeamMemberResponse {
    pub id: i64,
    pub team_id: String,
    pub user_id: String,
    pub role_id: i64,
    pub role_name: String,
    pub joined_at: String,
    pub invited_by: Option<String>,
}

impl From<TeamMember> for TeamMemberResponse {
    fn from(member: TeamMember) -> Self {
        Self {
            id: member.id,
            team_id: member.team_id,
            user_id: member.user_id,
            role_id: member.role_id,
            role_name: member.role_name,
            joined_at: member.joined_at.to_rfc3339(),
            invited_by: member.invited_by,
        }
    }
}

/// Create a new team.
///
/// `POST /api/v1/teams`
///
/// Validates the team name (1–100 characters) and slug (alphanumeric and hyphens).
/// The authenticated user becomes the team owner and is assigned the "owner" role.
#[utoipa::path(
    post,
    path = "/teams",
    request_body(content = CreateTeamRequest, description = "Team creation request"),
    responses(
        (status = 200, description = "Team created", body = TeamResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn create_team(
    State(state): State<TeamState>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<TeamResponse>, ServerError> {
    info!("Creating team: {}", req.name);

    if req.name.is_empty() || req.name.len() > 100 {
        return Err(ServerError::bad_request(
            "Team name must be between 1 and 100 characters",
        ));
    }

    if req.slug.is_empty() || !req.slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(ServerError::bad_request(
            "Slug must contain only alphanumeric characters and hyphens",
        ));
    }

    let owner_id = tachyon_core::generate_user_id().to_string();
    let mut team = Team::new(req.name, req.slug, owner_id);
    if let Some(desc) = req.description {
        team = team.with_description(desc);
    }

    let created = state
        .team_repo
        .create(&team)
        .await
        .map_err(|e| ServerError::database(format!("Failed to create team: {}", e)))?;

    let role = state.role_repo.get_by_name("owner").await.ok();
    if let Some(role) = role {
        let member = TeamMember::new(
            created.id.clone(),
            created.owner_id.clone(),
            role.id,
            role.name,
        );
        let _ = state.team_repo.add_member(&member).await;
    }

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::TeamCreated,
                AuditSeverity::Medium,
                "team_create",
                format!("Team '{}' created", created.name),
            )
            .with_target(&created.id, "team")
            .with_metadata("name", serde_json::json!(created.name)),
        )
        .await;

    Ok(Json(TeamResponse::from(created)))
}

/// Get a team by ID.
///
/// `GET /api/v1/teams/{team_id}`
#[utoipa::path(
    get,
    path = "/teams/{team_id}",
    params(
        ("team_id" = String, Path, description = "Team ID"),
    ),
    responses(
        (status = 200, description = "Team details", body = TeamResponse),
        (status = 404, description = "Team not found"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn get_team(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
) -> Result<Json<TeamResponse>, ServerError> {
    debug!("Getting team: {}", team_id);

    let team = state
        .team_repo
        .get_by_id(&team_id)
        .await
        .map_err(|e| ServerError::not_found(r"Team not found:", &e.to_string()))?;

    Ok(Json(TeamResponse::from(team)))
}

/// Get a team by its URL slug.
///
/// `GET /api/v1/teams/slug/{slug}`
#[utoipa::path(
    get,
    path = "/teams/slug/{slug}",
    params(
        ("slug" = String, Path, description = "Team slug"),
    ),
    responses(
        (status = 200, description = "Team details", body = TeamResponse),
        (status = 404, description = "Team not found"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn get_team_by_slug(
    Path(slug): Path<String>,
    State(state): State<TeamState>,
) -> Result<Json<TeamResponse>, ServerError> {
    debug!("Getting team by slug: {}", slug);

    let team = state
        .team_repo
        .get_by_slug(&slug)
        .await
        .map_err(|e| ServerError::not_found(r"Team not found:", &e.to_string()))?;

    Ok(Json(TeamResponse::from(team)))
}

/// List teams.
///
/// `GET /api/v1/teams`
///
/// Supports optional `owner_id` or `user_id` query parameters to filter results.
#[utoipa::path(
    get,
    path = "/teams",
    params(
        TeamQuery,
    ),
    responses(
        (status = 200, description = "List of teams", body = Vec<TeamResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn list_teams(
    Query(query): Query<TeamQuery>,
    State(state): State<TeamState>,
) -> Result<Json<Vec<TeamResponse>>, ServerError> {
    debug!("Listing teams");

    let teams = if let Some(owner_id) = query.owner_id {
        state.team_repo.list_by_owner(&owner_id).await
    } else if let Some(user_id) = query.user_id {
        state.team_repo.list_for_user(&user_id).await
    } else {
        state.team_repo.list_by_owner("").await
    };

    let teams =
        teams.map_err(|e| ServerError::database(format!(r"Failed to list teams: {}", e)))?;

    Ok(Json(teams.into_iter().map(TeamResponse::from).collect()))
}

/// Update a team.
///
/// `PUT /api/v1/teams/{team_id}`
///
/// Accepts partial updates for name, slug, and description.
#[utoipa::path(
    put,
    path = "/teams/{team_id}",
    params(
        ("team_id" = String, Path, description = "Team ID"),
    ),
    request_body(content = UpdateTeamRequest, description = "Team update request"),
    responses(
        (status = 200, description = "Team updated", body = TeamResponse),
        (status = 404, description = "Team not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn update_team(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<Json<TeamResponse>, ServerError> {
    debug!("Updating team: {}", team_id);

    let mut team = state
        .team_repo
        .get_by_id(&team_id)
        .await
        .map_err(|e| ServerError::not_found(r"Team not found:", &e.to_string()))?;

    if let Some(name) = req.name {
        team.name = name;
    }
    if let Some(slug) = req.slug {
        team.slug = slug;
    }
    if let Some(description) = req.description {
        team.description = Some(description);
    }
    team.updated_at = chrono::Utc::now();

    let updated = state
        .team_repo
        .update(&team)
        .await
        .map_err(|e| ServerError::database(format!(r"Failed to update team: {}", e)))?;

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::TeamUpdated,
                AuditSeverity::Low,
                "team_update",
                format!("Team '{}' updated", team_id),
            )
            .with_target(&team_id, "team"),
        )
        .await;

    Ok(Json(TeamResponse::from(updated)))
}

/// Delete a team.
///
/// `DELETE /api/v1/teams/{team_id}`
#[utoipa::path(
    delete,
    path = "/teams/{team_id}",
    params(
        ("team_id" = String, Path, description = "Team ID"),
    ),
    responses(
        (status = 204, description = "Team deleted"),
        (status = 404, description = "Team not found"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn delete_team(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
) -> Result<StatusCode, ServerError> {
    debug!("Deleting team: {}", team_id);

    state
        .team_repo
        .delete(&team_id)
        .await
        .map_err(|e| ServerError::not_found(r"Team not found:", &e.to_string()))?;

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::TeamDeleted,
                AuditSeverity::High,
                "team_delete",
                format!("Team '{}' deleted", team_id),
            )
            .with_target(&team_id, "team"),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

/// List members of a team.
///
/// `GET /api/v1/teams/{team_id}/members`
#[utoipa::path(
    get,
    path = "/teams/{team_id}/members",
    params(
        ("team_id" = String, Path, description = "Team ID"),
    ),
    responses(
        (status = 200, description = "List of team members", body = Vec<TeamMemberResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn list_team_members(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
) -> Result<Json<Vec<TeamMemberResponse>>, ServerError> {
    debug!("Listing members for team: {}", team_id);

    let members = state
        .team_repo
        .list_members(&team_id)
        .await
        .map_err(|e| ServerError::database(format!(r"Failed to list members: {}", e)))?;

    Ok(Json(
        members.into_iter().map(TeamMemberResponse::from).collect(),
    ))
}

/// Add a member to a team.
///
/// `POST /api/v1/teams/{team_id}/members`
///
/// Requires a `user_id` and `role_name` in the request body.
#[utoipa::path(
    post,
    path = "/teams/{team_id}/members",
    params(
        ("team_id" = String, Path, description = "Team ID"),
    ),
    request_body(content = AddMemberRequest, description = "Add member request"),
    responses(
        (status = 200, description = "Member added", body = TeamMemberResponse),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn add_team_member(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<TeamMemberResponse>, ServerError> {
    info!("Adding member {} to team {}", req.user_id, team_id);

    let role = state
        .role_repo
        .get_by_name(&req.role_name)
        .await
        .map_err(|e| ServerError::not_found("role", &e.to_string()))?;

    let member = TeamMember::new(team_id.clone(), req.user_id.clone(), role.id, role.name);
    let created = state
        .team_repo
        .add_member(&member)
        .await
        .map_err(|e| ServerError::database(format!(r"Failed to add member: {}", e)))?;

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::TeamMemberAdded,
                AuditSeverity::Medium,
                "team_member_add",
                format!(
                    "Member '{}' added to team '{}' with role '{}'",
                    req.user_id, team_id, req.role_name
                ),
            )
            .with_target(&team_id, "team")
            .with_metadata("member_id", serde_json::json!(req.user_id)),
        )
        .await;

    Ok(Json(TeamMemberResponse::from(created)))
}

/// Update a team member's role.
///
/// `PUT /api/v1/teams/{team_id}/members/{user_id}`
#[utoipa::path(
    put,
    path = "/teams/{team_id}/members/{user_id}",
    params(
        ("team_id" = String, Path, description = "Team ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body(content = UpdateMemberRequest, description = "Update member role"),
    responses(
        (status = 204, description = "Member role updated"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn update_team_member(
    Path((team_id, user_id)): Path<(String, String)>,
    State(state): State<TeamState>,
    Json(req): Json<UpdateMemberRequest>,
) -> Result<StatusCode, ServerError> {
    debug!("Updating member {} in team {}", user_id, team_id);

    let role = state
        .role_repo
        .get_by_name(&req.role_name)
        .await
        .map_err(|e| ServerError::not_found("role", &e.to_string()))?;

    state
        .team_repo
        .update_member_role(&team_id, &user_id, role.id, &role.name)
        .await
        .map_err(|e| ServerError::database(format!(r"Failed to update member: {}", e)))?;

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::TeamMemberUpdated,
                AuditSeverity::Medium,
                "team_member_update",
                format!("Member '{}' role updated in team '{}'", user_id, team_id),
            )
            .with_target(&team_id, "team")
            .with_metadata("user_id", serde_json::json!(user_id)),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

/// Remove a member from a team.
///
/// `DELETE /api/v1/teams/{team_id}/members/{user_id}`
#[utoipa::path(
    delete,
    path = "/teams/{team_id}/members/{user_id}",
    params(
        ("team_id" = String, Path, description = "Team ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 404, description = "Member not found"),
    ),
    tag = "teams",
    security(("bearer_auth" = [])),
)]
pub async fn remove_team_member(
    Path((team_id, user_id)): Path<(String, String)>,
    State(state): State<TeamState>,
) -> Result<StatusCode, ServerError> {
    debug!("Removing member {} from team {}", user_id, team_id);

    state
        .team_repo
        .remove_member(&team_id, &user_id)
        .await
        .map_err(|e| ServerError::not_found(r"Member not found:", &e.to_string()))?;

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::TeamMemberRemoved,
                AuditSeverity::Medium,
                "team_member_remove",
                format!("Member '{}' removed from team '{}'", user_id, team_id),
            )
            .with_target(&team_id, "team")
            .with_metadata("user_id", serde_json::json!(user_id)),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub fn create_team_router() -> axum::Router<TeamState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/teams", get(list_teams))
        .route("/teams", post(create_team))
        .route("/teams/slug/{slug}", get(get_team_by_slug))
        .route("/teams/{team_id}", get(get_team))
        .route("/teams/{team_id}", put(update_team))
        .route("/teams/{team_id}", delete(delete_team))
        .route("/teams/{team_id}/members", get(list_team_members))
        .route("/teams/{team_id}/members", post(add_team_member))
        .route(
            "/teams/{team_id}/members/{user_id}",
            put(update_team_member),
        )
        .route(
            "/teams/{team_id}/members/{user_id}",
            delete(remove_team_member),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_team_request() {
        let req = CreateTeamRequest {
            name: "Engineering".to_string(),
            slug: "engineering".to_string(),
            description: Some("Engineering team".to_string()),
        };
        assert_eq!(req.name, "Engineering");
    }

    #[test]
    fn test_team_response_from_team() {
        let team = Team::new("Test".to_string(), "test".to_string(), "user-1".to_string());
        let response = TeamResponse::from(team);
        assert_eq!(response.name, "Test");
    }
}
