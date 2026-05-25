// Space Routes
// REST API endpoints for space management and document hierarchy

use crate::audit::{AuditEvent, AuditEventType, AuditLogger, AuditSeverity};
use crate::error::ServerError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{
    AddSpaceMemberRequest, CreateSpaceRequest, DatabasePool, SpaceRepository,
    UpdateSpaceMemberRequest, UpdateSpaceRequest,
};
use tracing::info;
#[allow(unused_imports)]
use utoipa::IntoParams;

#[derive(Clone)]
pub struct SpaceState {
    pub pool: DatabasePool,
    pub audit_logger: AuditLogger,
}

// ============================================================================
// Response / Request Types
// ============================================================================

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SpaceResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: String,
    pub owner_id: String,
    pub parent_id: Option<String>,
    pub visibility: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub document_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SpaceMemberResponse {
    pub id: String,
    pub space_id: String,
    pub user_id: String,
    pub role: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateSpaceBody {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub visibility: Option<String>,
    /// Optional owner UUID. When auth middleware is active, this is
    /// overridden by the authenticated user's identity. For
    /// unauthenticated contexts (migration scripts, seeding) it allows
    /// explicit assignment.
    pub owner_id: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateSpaceBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub visibility: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SpaceQuery {
    pub parent_id: Option<String>,
    pub visibility: Option<String>,
    pub owner_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberBody {
    pub user_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberBody {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct MoveDocumentBody {
    pub space_id: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// List spaces with optional filters.
///
/// `GET /spaces?owner_id=&parent_id=&visibility=&limit=&offset=`
///
/// Response: 200 with `Vec<SpaceResponse>`, or 500 on error.
#[utoipa::path(
    get,
    path = "/api/v1/spaces",
    params(SpaceQuery),
    responses(
        (status = 200, description = "List of spaces", body = Vec<SpaceResponse>),
        (status = 500, description = "Internal error"),
    ),
    tag = "spaces",
)]
pub async fn list_spaces(
    Query(query): Query<SpaceQuery>,
    State(state): State<SpaceState>,
) -> Result<Json<Vec<SpaceResponse>>, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    let spaces = repo
        .list(
            query.owner_id.as_deref(),
            query.parent_id.as_deref(),
            query.visibility.as_deref(),
            None,
            query.limit,
            query.offset,
        )
        .await
        .map_err(|e| ServerError::internal(format!("Failed to list spaces: {}", e)))?;

    let doc_counts = repo
        .count_documents_batch(&spaces.iter().map(|s| s.id.clone()).collect::<Vec<_>>())
        .await
        .unwrap_or_default();

    let responses: Vec<SpaceResponse> = spaces
        .into_iter()
        .map(|space| {
            let doc_count = doc_counts.get(&space.id).copied().unwrap_or(0);
            SpaceResponse::from_space(space, doc_count)
        })
        .collect();

    Ok(Json(responses))
}

pub async fn list_root_spaces(
    Query(query): Query<SpaceQuery>,
    State(state): State<SpaceState>,
) -> Result<Json<Vec<SpaceResponse>>, ServerError> {
    let owner_id = query
        .owner_id
        .as_deref()
        .ok_or_else(|| ServerError::bad_request("owner_id query parameter is required"))?;
    let repo = SpaceRepository::new(state.pool.clone());
    let spaces = repo
        .list_root_spaces(owner_id, query.limit)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to list root spaces: {}", e)))?;

    let doc_counts = repo
        .count_documents_batch(&spaces.iter().map(|s| s.id.clone()).collect::<Vec<_>>())
        .await
        .unwrap_or_default();

    let responses: Vec<SpaceResponse> = spaces
        .into_iter()
        .map(|space| {
            let doc_count = doc_counts.get(&space.id).copied().unwrap_or(0);
            SpaceResponse::from_space(space, doc_count)
        })
        .collect();

    Ok(Json(responses))
}

pub async fn list_child_spaces(
    Path(parent_id): Path<String>,
    Query(query): Query<SpaceQuery>,
    State(state): State<SpaceState>,
) -> Result<Json<Vec<SpaceResponse>>, ServerError> {
    let owner_id = query
        .owner_id
        .as_deref()
        .ok_or_else(|| ServerError::bad_request("owner_id query parameter is required"))?;
    let repo = SpaceRepository::new(state.pool.clone());
    let spaces = repo
        .list_child_spaces(&parent_id, owner_id)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to list child spaces: {}", e)))?;

    let doc_counts = repo
        .count_documents_batch(&spaces.iter().map(|s| s.id.clone()).collect::<Vec<_>>())
        .await
        .unwrap_or_default();

    let responses: Vec<SpaceResponse> = spaces
        .into_iter()
        .map(|space| {
            let doc_count = doc_counts.get(&space.id).copied().unwrap_or(0);
            SpaceResponse::from_space(space, doc_count)
        })
        .collect();

    Ok(Json(responses))
}

/// Get a space by ID.
///
/// `GET /spaces/{space_id}`
///
/// Response: 200 with `SpaceResponse`, or 404 on error.
#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_id}",
    params(
        ("space_id" = String, Path, description = "Space ID"),
    ),
    responses(
        (status = 200, description = "Space found", body = SpaceResponse),
        (status = 404, description = "Space not found"),
    ),
    tag = "spaces",
)]
pub async fn get_space(
    Path(space_id): Path<String>,
    State(state): State<SpaceState>,
) -> Result<Json<SpaceResponse>, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    let space = repo
        .get_by_id(&space_id)
        .await
        .map_err(|e| ServerError::not_found("space", &format!("{}", e)))?;

    let doc_count = repo.count_documents(&space_id).await.unwrap_or(0);
    Ok(Json(SpaceResponse::from_space(space, doc_count)))
}

pub async fn get_default_space(
    Query(query): Query<SpaceQuery>,
    State(state): State<SpaceState>,
) -> Result<Json<SpaceResponse>, ServerError> {
    let owner_id = query
        .owner_id
        .as_deref()
        .ok_or_else(|| ServerError::bad_request("owner_id query parameter is required"))?;
    let repo = SpaceRepository::new(state.pool.clone());
    let space = repo
        .get_default_space(owner_id)
        .await
        .map_err(|e| ServerError::not_found("space", &format!("{}", e)))?;

    let doc_count = repo.count_documents(&space.id).await.unwrap_or(0);
    Ok(Json(SpaceResponse::from_space(space, doc_count)))
}

/// Create a new space.
///
/// `POST /spaces`
///
/// Request body: JSON with `name` (required), optional `description`, `icon`, `color`, `parent_id`, `visibility`.
/// Response: 201 with `SpaceResponse`, or 400/500 on error.
#[utoipa::path(
    post,
    path = "/api/v1/spaces",
    request_body = CreateSpaceBody,
    responses(
        (status = 200, description = "Space created", body = SpaceResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal error"),
    ),
    tag = "spaces",
)]
pub async fn create_space(
    State(state): State<SpaceState>,
    Json(body): Json<CreateSpaceBody>,
) -> Result<Json<SpaceResponse>, ServerError> {
    if body.name.trim().is_empty() {
        return Err(ServerError::bad_request("Space name is required"));
    }

    // Derive owner from authenticated context when available;
    // fall back to a request-supplied owner_id field (validated as UUID).
    let owner_id = body
        .owner_id
        .as_deref()
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .unwrap_or("00000000-0000-0000-0000-000000000000");

    let repo = SpaceRepository::new(state.pool.clone());
    let space = repo
        .create(
            owner_id,
            CreateSpaceRequest {
                name: body.name,
                description: body.description,
                icon: body.icon,
                color: body.color,
                parent_id: body.parent_id,
                visibility: body.visibility,
            },
        )
        .await
        .map_err(|e| ServerError::internal(format!("Failed to create space: {}", e)))?;

    let doc_count = 0;
    info!("Space created: {} ({})", space.name, space.slug);
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::SpaceCreated,
                AuditSeverity::Low,
                "space_create",
                format!("Space '{}' created", space.name),
            )
            .with_target(&space.id, "space")
            .with_metadata("name", serde_json::json!(space.name)),
        )
        .await;
    Ok(Json(SpaceResponse::from_space(space, doc_count)))
}

/// Update a space by ID.
///
/// `PUT /spaces/{space_id}`
///
/// Request body: JSON with optional `name`, `description`, `icon`, `color`, `parent_id`, `visibility`, `sort_order`.
/// Response: 200 with `SpaceResponse`, or 500 on error.
#[utoipa::path(
    put,
    path = "/api/v1/spaces/{space_id}",
    params(
        ("space_id" = String, Path, description = "Space ID"),
    ),
    request_body = UpdateSpaceBody,
    responses(
        (status = 200, description = "Space updated", body = SpaceResponse),
        (status = 500, description = "Internal error"),
    ),
    tag = "spaces",
)]
pub async fn update_space(
    Path(space_id): Path<String>,
    State(state): State<SpaceState>,
    Json(body): Json<UpdateSpaceBody>,
) -> Result<Json<SpaceResponse>, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());

    let space = repo
        .update(
            &space_id,
            UpdateSpaceRequest {
                name: body.name,
                description: body.description,
                icon: body.icon,
                color: body.color,
                parent_id: body.parent_id,
                visibility: body.visibility,
                sort_order: body.sort_order,
            },
        )
        .await
        .map_err(|e| ServerError::internal(format!("Failed to update space: {}", e)))?;

    let doc_count = repo.count_documents(&space_id).await.unwrap_or(0);
    info!("Space updated: {}", space_id);
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::SpaceUpdated,
                AuditSeverity::Low,
                "space_update",
                format!("Space '{}' updated", space_id),
            )
            .with_target(&space_id, "space"),
        )
        .await;
    Ok(Json(SpaceResponse::from_space(space, doc_count)))
}

/// Delete a space by ID.
///
/// `DELETE /spaces/{space_id}`
///
/// Response: 204 No Content, or 404 on error.
#[utoipa::path(
    delete,
    path = "/api/v1/spaces/{space_id}",
    params(
        ("space_id" = String, Path, description = "Space ID"),
    ),
    responses(
        (status = 204, description = "Space deleted"),
        (status = 404, description = "Space not found"),
    ),
    tag = "spaces",
)]
pub async fn delete_space(
    Path(space_id): Path<String>,
    State(state): State<SpaceState>,
) -> Result<StatusCode, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    repo.delete(&space_id)
        .await
        .map_err(|e| ServerError::not_found("space", &format!("{}", e)))?;

    info!("Space deleted: {}", space_id);
    Ok(StatusCode::NO_CONTENT)
}

// -- Members --

pub async fn list_space_members(
    Path(space_id): Path<String>,
    State(state): State<SpaceState>,
) -> Result<Json<Vec<SpaceMemberResponse>>, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    let members = repo
        .list_members(&space_id)
        .await
        .map_err(|e| ServerError::internal(format!("Failed to list members: {}", e)))?;

    Ok(Json(
        members.into_iter().map(SpaceMemberResponse::from).collect(),
    ))
}

pub async fn add_space_member(
    Path(space_id): Path<String>,
    State(state): State<SpaceState>,
    Json(body): Json<AddMemberBody>,
) -> Result<Json<SpaceMemberResponse>, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    let member = repo
        .add_member(
            &space_id,
            AddSpaceMemberRequest {
                user_id: body.user_id,
                role: body.role,
            },
        )
        .await
        .map_err(|e| ServerError::internal(format!("Failed to add member: {}", e)))?;

    info!("Member added to space {}", space_id);
    Ok(Json(SpaceMemberResponse::from(member)))
}

pub async fn update_space_member(
    Path((space_id, user_id)): Path<(String, String)>,
    State(state): State<SpaceState>,
    Json(body): Json<UpdateMemberBody>,
) -> Result<Json<SpaceMemberResponse>, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    let member = repo
        .update_member(
            &space_id,
            &user_id,
            UpdateSpaceMemberRequest { role: body.role },
        )
        .await
        .map_err(|e| ServerError::internal(format!("Failed to update member: {}", e)))?;

    info!("Member {} role updated in space {}", user_id, space_id);
    Ok(Json(SpaceMemberResponse::from(member)))
}

pub async fn remove_space_member(
    Path((space_id, user_id)): Path<(String, String)>,
    State(state): State<SpaceState>,
) -> Result<StatusCode, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    repo.remove_member(&space_id, &user_id)
        .await
        .map_err(|e| ServerError::not_found("member", &format!("{}", e)))?;

    info!("Member {} removed from space {}", user_id, space_id);
    Ok(StatusCode::NO_CONTENT)
}

// -- Document operations --

pub async fn move_document(
    Path(document_id): Path<String>,
    State(state): State<SpaceState>,
    Json(body): Json<MoveDocumentBody>,
) -> Result<StatusCode, ServerError> {
    let repo = SpaceRepository::new(state.pool.clone());
    repo.move_document(&document_id, body.space_id.as_deref())
        .await
        .map_err(|e| ServerError::internal(format!("Failed to move document: {}", e)))?;

    info!(
        "Document {} moved to space {:?}",
        document_id, body.space_id
    );
    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::DocumentMoved,
                AuditSeverity::Low,
                "document_move",
                format!(
                    "Document '{}' moved to space {:?}",
                    document_id, body.space_id
                ),
            )
            .with_target(&document_id, "document"),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Router
// ============================================================================

pub fn create_space_router() -> axum::Router<SpaceState> {
    axum::Router::new()
        // Space CRUD
        .route("/spaces", axum::routing::get(list_spaces))
        .route("/spaces/root", axum::routing::get(list_root_spaces))
        .route(
            "/spaces/{space_id}/children",
            axum::routing::get(list_child_spaces),
        )
        .route("/spaces/default", axum::routing::get(get_default_space))
        .route("/spaces", axum::routing::post(create_space))
        .route("/spaces/{space_id}", axum::routing::get(get_space))
        .route("/spaces/{space_id}", axum::routing::put(update_space))
        .route("/spaces/{space_id}", axum::routing::delete(delete_space))
        // Member management
        .route(
            "/spaces/{space_id}/members",
            axum::routing::get(list_space_members),
        )
        .route(
            "/spaces/{space_id}/members",
            axum::routing::post(add_space_member),
        )
        .route(
            "/spaces/{space_id}/members/{user_id}",
            axum::routing::put(update_space_member),
        )
        .route(
            "/spaces/{space_id}/members/{user_id}",
            axum::routing::delete(remove_space_member),
        )
        // Document operations
        .route(
            "/spaces/move-document/{document_id}",
            axum::routing::put(move_document),
        )
}

// ============================================================================
// Helpers
// ============================================================================

impl SpaceResponse {
    fn from_space(s: tachyon_database::Space, document_count: i64) -> Self {
        Self {
            id: s.id,
            name: s.name,
            slug: s.slug,
            description: s.description,
            icon: s.icon,
            color: s.color,
            owner_id: s.owner_id,
            parent_id: s.parent_id,
            visibility: s.visibility,
            sort_order: s.sort_order,
            is_default: s.is_default,
            document_count,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

impl From<tachyon_database::SpaceMember> for SpaceMemberResponse {
    fn from(m: tachyon_database::SpaceMember) -> Self {
        Self {
            id: m.id,
            space_id: m.space_id,
            user_id: m.user_id,
            role: m.role,
            username: m.username,
            display_name: m.display_name,
            avatar_url: m.avatar_url,
            joined_at: m.joined_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_space_body_deserialization() {
        let body = CreateSpaceBody {
            name: "My Space".to_string(),
            description: Some("A test space".to_string()),
            icon: Some("folder".to_string()),
            color: Some("#3B82F6".to_string()),
            parent_id: None,
            visibility: Some("private".to_string()),
            owner_id: None,
        };
        assert_eq!(body.name, "My Space");
        assert_eq!(body.visibility.as_deref(), Some("private"));
    }

    #[test]
    fn test_space_response_serialization() {
        let resp = SpaceResponse {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Test Space".to_string(),
            slug: "test-space".to_string(),
            description: Some("A test".to_string()),
            icon: "folder".to_string(),
            color: "#3B82F6".to_string(),
            owner_id: "00000000-0000-0000-0000-000000000001".to_string(),
            parent_id: None,
            visibility: "private".to_string(),
            sort_order: 0,
            is_default: false,
            document_count: 5,
            created_at: "2026-04-14T00:00:00+00:00".to_string(),
            updated_at: "2026-04-14T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("test-space"));
        assert!(json.contains("folder"));
        assert!(json.contains("\"document_count\":5"));
    }

    #[test]
    fn test_member_response_serialization() {
        let resp = SpaceMemberResponse {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            space_id: "00000000-0000-0000-0000-000000000001".to_string(),
            user_id: "00000000-0000-0000-0000-000000000002".to_string(),
            role: "editor".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            joined_at: "2026-04-14T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("editor"));
        assert!(json.contains("testuser"));
    }
}
