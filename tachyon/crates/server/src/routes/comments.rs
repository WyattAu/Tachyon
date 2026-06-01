//! Document comment API.
//!
//! Inline comments on documents with threading (replies) and resolution tracking.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::ServerError;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateCommentRequest {
    pub content: String,
    pub anchor_type: String,
    pub anchor_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub is_resolved: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CommentResponse {
    pub id: String,
    pub document_id: String,
    pub parent_id: Option<String>,
    pub author_id: String,
    pub content: String,
    pub anchor_type: String,
    pub anchor_value: Option<String>,
    pub depth: i32,
    pub is_resolved: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct CommentQuery {
    pub document_id: Option<String>,
    pub parent_id: Option<String>,
    pub resolved: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponse>,
    pub total: i64,
}

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct CommentState {
    pub pool: tachyon_database::DatabasePool,
}

impl CommentState {
    pub fn new(pool: tachyon_database::DatabasePool) -> Self {
        Self { pool }
    }
}

// ============================================================================
// Helpers
// ============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
struct CommentRow {
    id: String,
    document_id: String,
    parent_id: Option<String>,
    author_id: String,
    content: String,
    anchor_type: String,
    anchor_value: Option<String>,
    depth: i32,
    is_resolved: bool,
    resolved_by: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

fn row_to_response(r: CommentRow) -> CommentResponse {
    CommentResponse {
        id: r.id,
        document_id: r.document_id,
        parent_id: r.parent_id,
        author_id: r.author_id,
        content: r.content,
        anchor_type: r.anchor_type,
        anchor_value: r.anchor_value,
        depth: r.depth,
        is_resolved: r.is_resolved,
        created_at: r.created_at.to_rfc3339(),
        updated_at: r.updated_at.to_rfc3339(),
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// List comments filtered by document.
///
/// `GET /api/v1/documents/{document_id}/comments`
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/comments",
    params(
        ("document_id" = String, Path, description = "Document ID"),
        ("parent_id" = Option<String>, Query, description = "Filter by parent comment"),
        ("resolved" = Option<bool>, Query, description = "Filter by resolution status"),
        ("limit" = Option<usize>, Query, description = "Page size"),
        ("offset" = Option<usize>, Query, description = "Page offset"),
    ),
    responses(
        (status = 200, description = "Comment list", body = CommentListResponse),
    ),
    tag = "comments",
    security(("bearer_auth" = [])),
)]
pub async fn list_comments(
    Path(document_id): Path<String>,
    Query(params): Query<CommentQuery>,
    State(state): State<CommentState>,
) -> Result<Json<CommentListResponse>, ServerError> {
    info!("Listing comments for document: {}", document_id);

    let limit = params.limit.unwrap_or(50).min(100) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let rows: Vec<CommentRow> = if params.parent_id.is_some() {
        sqlx::query_as(
            r#"SELECT id, document_id, parent_id, author_id, content, anchor_type, anchor_value,
                      depth, is_resolved, resolved_by, created_at, updated_at
               FROM document_comments
               WHERE document_id = $1 AND parent_id = $2 AND deleted_at IS NULL
               ORDER BY created_at ASC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(&document_id)
        .bind(params.parent_id.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default()
    } else if let Some(resolved) = params.resolved {
        sqlx::query_as(
            r#"SELECT id, document_id, parent_id, author_id, content, anchor_type, anchor_value,
                      depth, is_resolved, resolved_by, created_at, updated_at
               FROM document_comments
               WHERE document_id = $1 AND is_resolved = $2 AND deleted_at IS NULL
               ORDER BY created_at ASC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(&document_id)
        .bind(resolved)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            r#"SELECT id, document_id, parent_id, author_id, content, anchor_type, anchor_value,
                      depth, is_resolved, resolved_by, created_at, updated_at
               FROM document_comments
               WHERE document_id = $1 AND deleted_at IS NULL
               ORDER BY created_at ASC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(&document_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default()
    };

    let total = rows.len() as i64;
    let comments = rows.into_iter().map(row_to_response).collect();

    Ok(Json(CommentListResponse { comments, total }))
}

/// Create a new comment on a document.
///
/// `POST /api/v1/documents/{document_id}/comments`
#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/comments",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CommentResponse),
    ),
    tag = "comments",
    security(("bearer_auth" = [])),
)]
pub async fn create_comment(
    Path(document_id): Path<String>,
    State(state): State<CommentState>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<(axum::http::StatusCode, Json<CommentResponse>), ServerError> {
    info!("Creating comment on document: {}", document_id);

    let sql = r#"
        INSERT INTO document_comments (document_id, author_id, content, anchor_type, anchor_value, depth)
        VALUES ($1, 'system', $2, $3, $4, 0)
        RETURNING id, document_id, parent_id, author_id, content, anchor_type, anchor_value, depth, is_resolved, resolved_by, created_at, updated_at
    "#;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let row: CommentRow = sqlx::query_as(sql)
        .bind(&document_id)
        .bind(&req.content)
        .bind(&req.anchor_type)
        .bind(&req.anchor_value)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    Ok((axum::http::StatusCode::CREATED, Json(row_to_response(row))))
}

/// Update a comment (content or resolution status).
///
/// `PUT /api/v1/comments/{comment_id}`
#[utoipa::path(
    put,
    path = "/api/v1/comments/{comment_id}",
    params(
        ("comment_id" = String, Path, description = "Comment ID"),
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Comment updated", body = CommentResponse),
    ),
    tag = "comments",
    security(("bearer_auth" = [])),
)]
pub async fn update_comment(
    Path(comment_id): Path<String>,
    State(state): State<CommentState>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<CommentResponse>, ServerError> {
    info!("Updating comment: {}", comment_id);

    let sql = r#"
        UPDATE document_comments
        SET content = COALESCE($1, content),
            is_resolved = COALESCE($2, is_resolved),
            updated_at = NOW()
        WHERE id = $3 AND deleted_at IS NULL
        RETURNING id, document_id, parent_id, author_id, content, anchor_type, anchor_value, depth, is_resolved, resolved_by, created_at, updated_at
    "#;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let row: CommentRow = sqlx::query_as(sql)
        .bind(&req.content)
        .bind(req.is_resolved)
        .bind(&comment_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| ServerError::not_found("Comment", &format!("Update failed: {}", e)))?;

    Ok(Json(row_to_response(row)))
}

/// Soft-delete a comment.
///
/// `DELETE /api/v1/comments/{comment_id}`
#[utoipa::path(
    delete,
    path = "/api/v1/comments/{comment_id}",
    params(
        ("comment_id" = String, Path, description = "Comment ID"),
    ),
    responses(
        (status = 204, description = "Comment deleted"),
    ),
    tag = "comments",
    security(("bearer_auth" = [])),
)]
pub async fn delete_comment(
    Path(comment_id): Path<String>,
    State(state): State<CommentState>,
) -> Result<axum::http::StatusCode, ServerError> {
    info!("Deleting comment: {}", comment_id);

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let result = sqlx::query(
        "UPDATE document_comments SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(&comment_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerError::not_found("Comment", &comment_id));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ============================================================================
// Router
// ============================================================================

pub fn create_comment_router() -> axum::Router<CommentState> {
    use axum::routing::{get, put};

    axum::Router::new()
        .route(
            "/documents/{document_id}/comments",
            get(list_comments).post(create_comment),
        )
        .route(
            "/comments/{comment_id}",
            put(update_comment).delete(delete_comment),
        )
}
