//! Collaboration API routes
//! Presence tracking, inline comments, and @mentions

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::ServerError;
use crate::websocket::ConnectionManager;
use tachyon_database::error::DatabaseError;
use tachyon_database::CommentRepository;
use tachyon_database::CreateDocumentCommentRequest as DbCreateCommentRequest;
use tachyon_database::PresenceRepository;
use tachyon_database::UpdateDocumentCommentRequest as DbUpdateCommentRequest;
use tachyon_database::UpsertPresenceRequest as DbUpsertPresenceRequest;

/// Collaboration state
#[derive(Clone)]
pub struct CollaborationState {
    pub pool: tachyon_database::DatabasePool,
    pub connection_manager: ConnectionManager,
}

impl CollaborationState {
    pub fn new(
        pool: tachyon_database::DatabasePool,
        connection_manager: ConnectionManager,
    ) -> Self {
        Self {
            pool,
            connection_manager,
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PresenceInfo {
    pub user_id: String,
    pub user_name: String,
    pub document_id: String,
    pub status: PresenceStatus,
    pub cursor_position: Option<CursorInfo>,
    pub connected_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PresenceStatus {
    Active,
    Idle,
    Away,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CursorInfo {
    pub section: Option<String>,
    pub line: Option<u32>,
    pub selection: Option<String>,
}

/// Inline comment on a document
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Comment {
    pub id: String,
    pub document_id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub anchor: Option<CommentAnchor>,
    pub status: CommentStatus,
    pub parent_id: Option<String>,
    pub thread_id: Option<String>,
    pub start_offset: Option<i32>,
    pub end_offset: Option<i32>,
    pub mentions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Open,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CommentAnchor {
    pub section: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub selection_text: Option<String>,
}

// ============================================================================
// Request/Response types
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdatePresenceRequest {
    pub document_id: String,
    pub user_id: String,
    pub user_name: String,
    pub status: Option<PresenceStatus>,
    pub cursor_position: Option<CursorInfo>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PresenceResponse {
    pub document_id: String,
    pub users: Vec<PresenceInfo>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateCommentRequest {
    pub document_id: String,
    pub content: String,
    pub anchor: Option<CommentAnchor>,
    pub parent_id: Option<String>,
    pub thread_id: Option<String>,
    pub start_offset: Option<i32>,
    pub end_offset: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub status: Option<CommentStatus>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CommentsResponse {
    pub comments: Vec<Comment>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MentionsResponse {
    pub mentions: Vec<MentionNotification>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MentionNotification {
    pub id: String,
    pub user_id: String,
    pub document_id: String,
    pub document_title: String,
    pub mentioned_by: String,
    pub mentioned_by_name: String,
    pub comment_content: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

// ============================================================================
// Helpers
// ============================================================================

/// Convert a database presence row into the API PresenceInfo type
fn db_presence_to_info(db: tachyon_database::presence::Presence) -> PresenceInfo {
    let status = match db.status.as_str() {
        "idle" => PresenceStatus::Idle,
        "away" => PresenceStatus::Away,
        _ => PresenceStatus::Active,
    };
    let cursor_position =
        if db.cursor_section.is_some() || db.cursor_line.is_some() || db.cursor_selection.is_some()
        {
            Some(CursorInfo {
                section: db.cursor_section,
                line: db.cursor_line.map(|v| v as u32),
                selection: db.cursor_selection,
            })
        } else {
            None
        };
    let connected_at = db
        .connected_at
        .parse::<DateTime<Utc>>()
        .unwrap_or_else(|_| Utc::now());
    let last_seen_at = db
        .last_seen_at
        .parse::<DateTime<Utc>>()
        .unwrap_or_else(|_| Utc::now());

    PresenceInfo {
        user_id: db.user_id,
        user_name: db.user_name,
        document_id: db.document_id,
        status,
        cursor_position,
        connected_at,
        last_seen_at,
    }
}

/// Convert a PresenceStatus to the database string representation
fn status_to_db(status: &PresenceStatus) -> String {
    match status {
        PresenceStatus::Idle => "idle".to_string(),
        PresenceStatus::Away => "away".to_string(),
        PresenceStatus::Active => "active".to_string(),
    }
}

/// Parse a DateTime<Utc> from a string, falling back to now
fn parse_datetime(s: &str) -> DateTime<Utc> {
    s.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now())
}

// ============================================================================
// Handlers — Presence (database-backed)
// ============================================================================

/// PUT /api/v1/collaboration/presence — Update user presence
#[utoipa::path(
    put,
    path = "/collaboration/presence",
    request_body(content = UpdatePresenceRequest, description = "Presence update request"),
    responses(
        (status = 200, description = "Presence updated", body = PresenceResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn update_presence(
    State(state): State<CollaborationState>,
    Json(req): Json<UpdatePresenceRequest>,
) -> Result<Json<PresenceResponse>, ServerError> {
    let db_req = DbUpsertPresenceRequest {
        user_id: req.user_id.clone(),
        user_name: req.user_name.clone(),
        document_id: req.document_id.clone(),
        status: req.status.as_ref().map(status_to_db),
        cursor_section: req.cursor_position.as_ref().and_then(|c| c.section.clone()),
        cursor_line: req
            .cursor_position
            .as_ref()
            .and_then(|c| c.line.map(|v| v as i32)),
        cursor_selection: req
            .cursor_position
            .as_ref()
            .and_then(|c| c.selection.clone()),
    };

    let repo = PresenceRepository::new(state.pool.clone());
    repo.upsert(db_req).await?;

    // Broadcast presence update to WebSocket clients viewing this document
    let presence_user = crate::websocket::types::PresenceUser {
        user_id: req.user_id.clone(),
        user_name: req.user_name.clone(),
        cursor_position: req
            .cursor_position
            .as_ref()
            .map(|c| c.line.unwrap_or(0) as usize)
            .unwrap_or(0),
        selection: None,
        color: None,
    };
    let presence_msg = crate::websocket::types::WebSocketMessage::presence(
        req.document_id.clone(),
        vec![presence_user],
    );
    let _ = state
        .connection_manager
        .broadcast_to_room(&format!("doc:{}", req.document_id), presence_msg)
        .await;

    // Return all live presence for this document
    let users = repo
        .list_by_document(&req.document_id)
        .await?
        .into_iter()
        .map(db_presence_to_info)
        .collect();

    Ok(Json(PresenceResponse {
        document_id: req.document_id,
        users,
    }))
}

/// GET /api/v1/collaboration/presence/{document_id} — Get presence for a document
#[utoipa::path(
    get,
    path = "/collaboration/presence/{document_id}",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "Presence for document", body = PresenceResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn get_presence(
    State(state): State<CollaborationState>,
    Path(document_id): Path<String>,
) -> Result<Json<PresenceResponse>, ServerError> {
    let repo = PresenceRepository::new(state.pool.clone());
    let users = repo
        .list_by_document(&document_id)
        .await?
        .into_iter()
        .map(db_presence_to_info)
        .collect();

    Ok(Json(PresenceResponse { document_id, users }))
}

/// DELETE /api/v1/collaboration/presence/{document_id}/{user_id} — Remove user presence
#[utoipa::path(
    delete,
    path = "/collaboration/presence/{document_id}/{user_id}",
    params(
        ("document_id" = String, Path, description = "Document ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "Presence removed"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn remove_presence(
    State(state): State<CollaborationState>,
    Path((document_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, ServerError> {
    let repo = PresenceRepository::new(state.pool.clone());
    repo.remove(&user_id, &document_id).await?;

    // Broadcast leave event to WebSocket clients
    let leave_msg = crate::websocket::types::WebSocketMessage::leave(document_id.clone(), user_id);
    let _ = state
        .connection_manager
        .broadcast_to_room(&format!("doc:{}", document_id), leave_msg)
        .await;

    Ok(StatusCode::NO_CONTENT)
}

fn db_error(e: DatabaseError) -> ServerError {
    if matches!(e, DatabaseError::NotFound { .. }) {
        ServerError::not_found("resource", &e.to_string())
    } else {
        ServerError::database(e.to_string())
    }
}

// ============================================================================
// Handlers — Comments
// ============================================================================

fn db_comment_to_comment(db: tachyon_database::comment::Comment) -> Comment {
    let mentions: Vec<String> = serde_json::from_str(&db.mentions).unwrap_or_default();
    let status = if db.status == "resolved" {
        CommentStatus::Resolved
    } else {
        CommentStatus::Open
    };
    let created_at = parse_datetime(&db.created_at);
    let updated_at = parse_datetime(&db.updated_at);
    let resolved_at = db
        .resolved_at
        .as_ref()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok());

    Comment {
        id: db.id,
        document_id: db.document_id,
        author_id: db.author_id,
        author_name: db.author_name,
        content: db.content,
        anchor: if db.anchor_section.is_some() || db.anchor_line_start.is_some() {
            Some(CommentAnchor {
                section: db.anchor_section,
                line_start: db.anchor_line_start.map(|v| v as u32),
                line_end: db.anchor_line_end.map(|v| v as u32),
                selection_text: db.anchor_selection,
            })
        } else {
            None
        },
        status,
        parent_id: db.parent_id,
        thread_id: db.thread_id,
        start_offset: db.start_offset,
        end_offset: db.end_offset,
        mentions,
        created_at,
        updated_at,
        resolved_at,
        resolved_by: db.resolved_by,
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListCommentsQuery {
    pub threaded: Option<String>,
}

/// GET /api/v1/collaboration/comments/{document_id} — List comments
#[utoipa::path(
    get,
    path = "/collaboration/documents/{document_id}/comments",
    params(
        ("document_id" = String, Path, description = "Document ID"),
        ("threaded" = Option<String>, Query, description = "Set to 'true' for threaded organization"),
    ),
    responses(
        (status = 200, description = "Document comments", body = CommentsResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn list_comments(
    State(state): State<CollaborationState>,
    Path(document_id): Path<String>,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<CommentsResponse>, ServerError> {
    let repo = CommentRepository::new(state.pool.clone());
    let comments = repo
        .list_by_document(&document_id, true, None, 100, 0)
        .await?;

    let total = comments.len();
    let mut comments: Vec<Comment> = comments.into_iter().map(db_comment_to_comment).collect();

    if query.threaded.as_deref() == Some("true") {
        comments.sort_by(|a, b| {
            let a_thread = a
                .thread_id
                .as_ref()
                .or(a.parent_id.as_ref())
                .unwrap_or(&a.id);
            let b_thread = b
                .thread_id
                .as_ref()
                .or(b.parent_id.as_ref())
                .unwrap_or(&b.id);
            a_thread
                .cmp(b_thread)
                .then_with(|| a.parent_id.is_some().cmp(&b.parent_id.is_some()))
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
    }

    Ok(Json(CommentsResponse { comments, total }))
}

/// POST /api/v1/collaboration/comments — Create a comment
#[utoipa::path(
    post,
    path = "/collaboration/comments",
    request_body(content = CreateCommentRequest, description = "Comment creation request"),
    responses(
        (status = 200, description = "Comment created", body = Comment),
        (status = 500, description = "Internal server error"),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn create_comment(
    State(state): State<CollaborationState>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, ServerError> {
    let db_req = DbCreateCommentRequest {
        document_id: req.document_id.clone(),
        author_id: "user".to_string(),
        author_name: "User".to_string(),
        content: req.content.clone(),
        anchor_section: req.anchor.as_ref().and_then(|a| a.section.clone()),
        anchor_line_start: req
            .anchor
            .as_ref()
            .and_then(|a| a.line_start.map(|v| v as i32)),
        anchor_line_end: req
            .anchor
            .as_ref()
            .and_then(|a| a.line_end.map(|v| v as i32)),
        anchor_selection: req.anchor.as_ref().and_then(|a| a.selection_text.clone()),
        parent_id: req.parent_id.clone(),
        thread_id: req.thread_id.clone(),
        start_offset: req.start_offset,
        end_offset: req.end_offset,
        mentions: None,
    };

    let repo = CommentRepository::new(state.pool.clone());
    let comment = repo.create(db_req).await?;

    info!(
        "Comment created on {} (mentions: {:?})",
        req.document_id, comment.mentions
    );

    // Broadcast comment activity to WebSocket clients viewing this document
    let activity = crate::websocket::types::ActivityUpdate {
        activity_type: "comment_created".to_string(),
        description: format!("New comment on {}", req.document_id),
        metadata: None,
    };
    let activity_msg = crate::websocket::types::WebSocketMessage::activity(
        req.document_id.clone(),
        "system".to_string(),
        activity,
    );
    let _ = state
        .connection_manager
        .broadcast_to_room(&format!("doc:{}", req.document_id), activity_msg)
        .await;

    Ok(Json(db_comment_to_comment(comment)))
}

/// PUT /api/v1/collaboration/comments/{comment_id} — Update a comment
#[utoipa::path(
    put,
    path = "/collaboration/comments/{comment_id}",
    params(
        ("comment_id" = String, Path, description = "Comment ID"),
    ),
    request_body(content = UpdateCommentRequest, description = "Comment update request"),
    responses(
        (status = 200, description = "Comment updated", body = Comment),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn update_comment(
    State(state): State<CollaborationState>,
    Path(comment_id): Path<String>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<Comment>, ServerError> {
    let db_req = DbUpdateCommentRequest {
        content: req.content,
        status: req.status.map(|s| format!("{:?}", s).to_lowercase()),
        resolved_by: None,
    };

    let repo = CommentRepository::new(state.pool.clone());
    let comment = repo.update(&comment_id, db_req).await.map_err(db_error)?;

    Ok(Json(db_comment_to_comment(comment)))
}

/// DELETE /api/v1/collaboration/comments/{comment_id} — Delete a comment
#[utoipa::path(
    delete,
    path = "/collaboration/comments/{comment_id}",
    params(
        ("comment_id" = String, Path, description = "Comment ID"),
    ),
    responses(
        (status = 204, description = "Comment deleted"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn delete_comment(
    State(state): State<CollaborationState>,
    Path(comment_id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let repo = CommentRepository::new(state.pool.clone());
    repo.delete(&comment_id).await.map_err(db_error)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Handlers — Mentions
// ============================================================================

/// GET /api/v1/collaboration/mentions/{user_id} — Get mentions for a user
#[utoipa::path(
    get,
    path = "/collaboration/mentions/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User mentions", body = MentionsResponse),
    ),
    tag = "collaboration",
    security(("bearer_auth" = [])),
)]
pub async fn get_mentions(
    State(_state): State<CollaborationState>,
    Path(_user_id): Path<String>,
) -> Json<MentionsResponse> {
    Json(MentionsResponse { mentions: vec![] })
}

// ============================================================================
// Router
// ============================================================================

pub fn create_collaboration_router() -> axum::Router<CollaborationState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/collaboration/presence", put(update_presence))
        .route("/collaboration/presence/{document_id}", get(get_presence))
        .route(
            "/collaboration/presence/{document_id}/{user_id}",
            delete(remove_presence),
        )
        // Comment CRUD: list by document, create, update, delete
        .route(
            "/collaboration/documents/{document_id}/comments",
            get(list_comments),
        )
        .route("/collaboration/comments", post(create_comment))
        .route("/collaboration/comments/{comment_id}", put(update_comment))
        .route(
            "/collaboration/comments/{comment_id}",
            delete(delete_comment),
        )
        .route("/collaboration/mentions/{user_id}", get(get_mentions))
}
