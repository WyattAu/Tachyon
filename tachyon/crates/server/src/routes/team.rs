// Team API routes
// Handles team management operations

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{DatabasePool, RoleRepository, Team, TeamMember, TeamRepository};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct TeamState {
    pub pool: DatabasePool,
    pub team_repo: TeamRepository,
    pub role_repo: RoleRepository,
}

impl TeamState {
    pub fn new(pool: DatabasePool) -> Self {
        let team_repo = TeamRepository::new(pool.clone());
        let role_repo = RoleRepository::new(pool.clone());
        Self {
            pool,
            team_repo,
            role_repo,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    pub role_name: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamQuery {
    pub owner_id: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

pub async fn create_team(
    State(state): State<TeamState>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<TeamResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating team: {}", req.name);

    if req.name.is_empty() || req.name.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Team name must be between 1 and 100 characters".to_string(),
            }),
        ));
    }

    if req.slug.is_empty() || !req.slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Slug must contain only alphanumeric characters and hyphens".to_string(),
            }),
        ));
    }

    let owner_id = tachyon_core::generate_user_id().to_string();
    let mut team = Team::new(req.name, req.slug, owner_id);
    if let Some(desc) = req.description {
        team = team.with_description(desc);
    }

    let created = state.team_repo.create(&team).await.map_err(|e| {
        warn!("Failed to create team: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to create team: {}", e),
            }),
        )
    })?;

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

    Ok(Json(TeamResponse::from(created)))
}

pub async fn get_team(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
) -> Result<Json<TeamResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting team: {}", team_id);

    let team = state.team_repo.get_by_id(&team_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Team not found: {}", e),
            }),
        )
    })?;

    Ok(Json(TeamResponse::from(team)))
}

pub async fn get_team_by_slug(
    Path(slug): Path<String>,
    State(state): State<TeamState>,
) -> Result<Json<TeamResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting team by slug: {}", slug);

    let team = state.team_repo.get_by_slug(&slug).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Team not found: {}", e),
            }),
        )
    })?;

    Ok(Json(TeamResponse::from(team)))
}

pub async fn list_teams(
    Query(query): Query<TeamQuery>,
    State(state): State<TeamState>,
) -> Result<Json<Vec<TeamResponse>>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Listing teams");

    let teams = if let Some(owner_id) = query.owner_id {
        state.team_repo.list_by_owner(&owner_id).await
    } else if let Some(user_id) = query.user_id {
        state.team_repo.list_for_user(&user_id).await
    } else {
        state.team_repo.list_by_owner("").await
    };

    let teams = teams.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to list teams: {}", e),
            }),
        )
    })?;

    Ok(Json(teams.into_iter().map(TeamResponse::from).collect()))
}

pub async fn update_team(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<Json<TeamResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Updating team: {}", team_id);

    let mut team = state.team_repo.get_by_id(&team_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Team not found: {}", e),
            }),
        )
    })?;

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

    let updated = state.team_repo.update(&team).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to update team: {}", e),
            }),
        )
    })?;

    Ok(Json(TeamResponse::from(updated)))
}

pub async fn delete_team(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    debug!("Deleting team: {}", team_id);

    state.team_repo.delete(&team_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Team not found: {}", e),
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_team_members(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
) -> Result<Json<Vec<TeamMemberResponse>>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Listing members for team: {}", team_id);

    let members = state.team_repo.list_members(&team_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to list members: {}", e),
            }),
        )
    })?;

    Ok(Json(
        members.into_iter().map(TeamMemberResponse::from).collect(),
    ))
}

pub async fn add_team_member(
    Path(team_id): Path<String>,
    State(state): State<TeamState>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<TeamMemberResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Adding member {} to team {}", req.user_id, team_id);

    let role = state
        .role_repo
        .get_by_name(&req.role_name)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "ROLE_NOT_FOUND".to_string(),
                    message: format!("Role not found: {}", e),
                }),
            )
        })?;

    let member = TeamMember::new(team_id.clone(), req.user_id, role.id, role.name);
    let created = state.team_repo.add_member(&member).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to add member: {}", e),
            }),
        )
    })?;

    Ok(Json(TeamMemberResponse::from(created)))
}

pub async fn update_team_member(
    Path((team_id, user_id)): Path<(String, String)>,
    State(state): State<TeamState>,
    Json(req): Json<UpdateMemberRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    debug!("Updating member {} in team {}", user_id, team_id);

    let role = state
        .role_repo
        .get_by_name(&req.role_name)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "ROLE_NOT_FOUND".to_string(),
                    message: format!("Role not found: {}", e),
                }),
            )
        })?;

    state
        .team_repo
        .update_member_role(&team_id, &user_id, role.id, &role.name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "DATABASE_ERROR".to_string(),
                    message: format!("Failed to update member: {}", e),
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_team_member(
    Path((team_id, user_id)): Path<(String, String)>,
    State(state): State<TeamState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    debug!("Removing member {} from team {}", user_id, team_id);

    state
        .team_repo
        .remove_member(&team_id, &user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Member not found: {}", e),
                }),
            )
        })?;

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
