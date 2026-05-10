use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tachyon_core::id::{SessionId, UserId};
use tachyon_core::types::session::{SessionBuilder, TokenType};
use tachyon_database::{DatabasePool, SessionRepository};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct SessionState {
    pub pool: DatabasePool,
    pub expiration_secs: u64,
}

impl SessionState {
    pub fn new(pool: DatabasePool, expiration_secs: u64) -> Self {
        Self {
            pool,
            expiration_secs,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SessionErrorResponse {
    pub code: String,
    pub message: String,
}

/// Create a new session for a user.
///
/// `POST /sessions`
///
/// Request body: JSON with `user_id` (required), optional `metadata`.
/// Response: 200 with `SessionResponse`, or 400/500 on error.
#[utoipa::path(
    post,
    path = "/sessions",
    request_body(content = CreateSessionRequest, description = "Session creation request"),
    responses(
        (status = 200, description = "Session created", body = SessionResponse),
        (status = 400, description = "Invalid user ID"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "sessions",
    security(("bearer_auth" = [])),
)]
pub async fn create_session(
    State(state): State<SessionState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<SessionErrorResponse>)> {
    info!("Creating new session for user: {}", req.user_id);

    let session_id = SessionId::new();
    let user_id = UserId::parse_str(&req.user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(SessionErrorResponse {
                code: "INVALID_USER_ID".to_string(),
                message: format!("Invalid user ID: {}", e),
            }),
        )
    })?;

    let token_value = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_in = chrono::Duration::seconds(state.expiration_secs as i64);

    let session = SessionBuilder::new(session_id, user_id, token_value)
        .token_type(TokenType::Bearer)
        .expires_in(expires_in)
        .build();

    let repo = SessionRepository::new(state.pool.clone());
    repo.create(&session).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SessionErrorResponse {
                code: "SESSION_CREATE_FAILED".to_string(),
                message: format!("Failed to create session: {}", e),
            }),
        )
    })?;

    let response = SessionResponse {
        id: session.id.as_str(),
        user_id: session.user_id().as_str(),
        metadata: req.metadata,
        created_at: now.to_rfc3339(),
    };

    info!("Session created: {}", response.id);
    Ok(Json(response))
}

/// Get a session by ID.
///
/// `GET /sessions/{session_id}`
///
/// Response: 200 with `SessionResponse`, or 400/404/410 (expired) on error.
#[utoipa::path(
    get,
    path = "/sessions/{session_id}",
    params(
        ("session_id" = String, Path, description = "Session ID"),
    ),
    responses(
        (status = 200, description = "Session details", body = SessionResponse),
        (status = 400, description = "Invalid session ID"),
        (status = 404, description = "Session not found"),
        (status = 410, description = "Session expired"),
    ),
    tag = "sessions",
    security(("bearer_auth" = [])),
)]
pub async fn get_session(
    Path(session_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Getting session: {}", session_id);

    let sid = SessionId::parse_str(&session_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(SessionErrorResponse {
                code: "INVALID_SESSION_ID".to_string(),
                message: format!("Invalid session ID: {}", session_id),
            }),
        )
    })?;

    let repo = SessionRepository::new(state.pool.clone());
    let record = repo.get_by_id(&sid).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(SessionErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Session {} not found: {}", session_id, e),
            }),
        )
    })?;

    if !record.is_valid() {
        warn!("Session expired or inactive: {}", session_id);
        return Err((
            StatusCode::GONE,
            Json(SessionErrorResponse {
                code: "SESSION_EXPIRED".to_string(),
                message: "Session has expired or is inactive".to_string(),
            }),
        ));
    }

    Ok(Json(SessionResponse {
        id: record.id,
        user_id: record.user_id,
        metadata: None,
        created_at: record.created_at.to_rfc3339(),
    }))
}

/// Validate whether a session is still active.
///
/// `GET /sessions/{session_id}/validate`
///
/// Response: 200 with `{"valid": bool, "session_id": string}`.
#[utoipa::path(
    get,
    path = "/sessions/{session_id}/validate",
    params(
        ("session_id" = String, Path, description = "Session ID"),
    ),
    responses(
        (status = 200, description = "Validation result", body = serde_json::Value),
        (status = 400, description = "Invalid session ID"),
    ),
    tag = "sessions",
    security(("bearer_auth" = [])),
)]
pub async fn validate_session(
    Path(session_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<SessionErrorResponse>)> {
    info!("Validating session: {}", session_id);

    let sid = SessionId::parse_str(&session_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(SessionErrorResponse {
                code: "INVALID_SESSION_ID".to_string(),
                message: format!("Invalid session ID: {}", session_id),
            }),
        )
    })?;

    let repo = SessionRepository::new(state.pool.clone());
    match repo.validate_session(&sid).await {
        Ok(true) => Ok(Json(serde_json::json!({
            "valid": true,
            "session_id": session_id,
        }))),
        Ok(false) => Ok(Json(serde_json::json!({
            "valid": false,
            "session_id": session_id,
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "valid": false,
            "session_id": session_id,
            "reason": e.to_string()
        }))),
    }
}

/// Revoke a session by ID.
///
/// `DELETE /sessions/{session_id}`
///
/// Response: 204 No Content, or 400/404 on error.
#[utoipa::path(
    delete,
    path = "/sessions/{session_id}",
    params(
        ("session_id" = String, Path, description = "Session ID"),
    ),
    responses(
        (status = 204, description = "Session revoked"),
        (status = 400, description = "Invalid session ID"),
        (status = 404, description = "Session not found"),
    ),
    tag = "sessions",
    security(("bearer_auth" = [])),
)]
pub async fn revoke_session(
    Path(session_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<StatusCode, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Revoking session: {}", session_id);

    let sid = SessionId::parse_str(&session_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(SessionErrorResponse {
                code: "INVALID_SESSION_ID".to_string(),
                message: format!("Invalid session ID: {}", session_id),
            }),
        )
    })?;

    let repo = SessionRepository::new(state.pool.clone());
    repo.revoke(&sid).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(SessionErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Session {} not found: {}", session_id, e),
            }),
        )
    })?;

    info!("Session revoked: {}", session_id);
    Ok(StatusCode::NO_CONTENT)
}

/// List all active sessions for a user.
///
/// `GET /users/{user_id}/sessions`
///
/// Response: 200 with `SessionListResponse` containing `sessions` and `total`, or 500 on error.
#[utoipa::path(
    get,
    path = "/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "List of active sessions", body = SessionListResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "sessions",
    security(("bearer_auth" = [])),
)]
pub async fn list_sessions(
    Path(user_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<SessionListResponse>, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Listing sessions for user: {}", user_id);

    let repo = SessionRepository::new(state.pool.clone());
    let records = repo.get_by_user(&user_id, true).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SessionErrorResponse {
                code: "LIST_FAILED".to_string(),
                message: format!("Failed to list sessions: {}", e),
            }),
        )
    })?;

    let sessions: Vec<SessionResponse> = records
        .into_iter()
        .map(|r| SessionResponse {
            id: r.id,
            user_id: r.user_id,
            metadata: None,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    let total = sessions.len();

    Ok(Json(SessionListResponse { sessions, total }))
}

/// Revoke all sessions for a user.
///
/// `DELETE /users/{user_id}/sessions`
///
/// Response: 200 with `{"success": true, "revoked_count": N}`, or 500 on error.
#[utoipa::path(
    delete,
    path = "/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "All sessions revoked", body = serde_json::Value),
        (status = 500, description = "Internal server error"),
    ),
    tag = "sessions",
    security(("bearer_auth" = [])),
)]
pub async fn revoke_all_sessions(
    Path(user_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Revoking all sessions for user: {}", user_id);

    let repo = SessionRepository::new(state.pool.clone());
    let revoked_count = repo.revoke_all_for_user(&user_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SessionErrorResponse {
                code: "REVOKE_ALL_FAILED".to_string(),
                message: format!("Failed to revoke sessions: {}", e),
            }),
        )
    })?;

    info!("Revoked {} sessions for user: {}", revoked_count, user_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "revoked_count": revoked_count,
        "message": format!("Revoked {} session(s)", revoked_count)
    })))
}

pub fn create_session_router() -> axum::Router<SessionState> {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route("/sessions", post(create_session))
        .route("/sessions/{session_id}", get(get_session))
        .route("/sessions/{session_id}/validate", get(validate_session))
        .route("/sessions/{session_id}", delete(revoke_session))
        .route("/users/{user_id}/sessions", get(list_sessions))
        .route("/users/{user_id}/sessions", delete(revoke_all_sessions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session_request_construction() {
        let req = CreateSessionRequest {
            user_id: "user-1".to_string(),
            metadata: None,
        };

        assert_eq!(req.user_id, "user-1");
    }

    #[test]
    fn test_session_list_response_serialization() {
        let resp = SessionListResponse {
            sessions: vec![],
            total: 0,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("total"));
    }

    #[test]
    fn test_session_response_construction() {
        let response = SessionResponse {
            id: "sess-123".to_string(),
            user_id: "user-1".to_string(),
            metadata: Some(serde_json::json!({"device": "web"})),
            created_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(response.id, "sess-123");
        assert_eq!(response.user_id, "user-1");
        assert!(response.metadata.is_some());
    }

    #[test]
    fn test_session_error_response_construction() {
        let err = SessionErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: "Session not found".to_string(),
        };

        assert_eq!(err.code, "NOT_FOUND");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NOT_FOUND"));
    }
}
