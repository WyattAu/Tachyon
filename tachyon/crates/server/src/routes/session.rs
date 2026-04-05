// Session API routes
// Handles session CRUD operations and validation

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Session metadata
    pub metadata: Option<serde_json::Value>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Is active
    pub is_active: bool,
}

/// Application state for session routes
#[derive(Clone)]
pub struct SessionState {
    /// In-memory session store (for demo mode)
    pub sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    /// Session expiration time in seconds
    pub expiration_secs: u64,
}

impl SessionState {
    /// Create a new session state
    pub fn new(expiration_secs: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            expiration_secs,
        }
    }

    /// Create a session state with existing sessions
    pub fn with_sessions(sessions: HashMap<String, SessionData>, expiration_secs: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(sessions)),
            expiration_secs,
        }
    }
}

/// Request to create a session
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// User ID
    pub user_id: String,
    /// Session metadata
    pub metadata: Option<serde_json::Value>,
}

/// Request to validate a session
#[derive(Debug, Deserialize)]
pub struct ValidateSessionRequest {
    /// Session ID
    pub session_id: String,
}

/// Session response
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Session metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Created at
    pub created_at: String,
}

impl From<SessionData> for SessionResponse {
    fn from(session: SessionData) -> Self {
        Self {
            id: session.id,
            user_id: session.user_id,
            metadata: session.metadata,
            created_at: session.created_at.to_rfc3339(),
        }
    }
}

/// Session list response
#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    /// List of sessions
    pub sessions: Vec<SessionResponse>,
    /// Total count
    pub total: usize,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct SessionErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// Create a new session
pub async fn create_session(
    State(state): State<SessionState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<SessionErrorResponse>)> {
    info!("Creating new session for user: {}", req.user_id);

    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(state.expiration_secs as i64);

    let session = SessionData {
        id: format!("sess_{}", Uuid::new_v4()),
        user_id: req.user_id,
        metadata: req.metadata,
        created_at: now,
        updated_at: now,
        expires_at,
        is_active: true,
    };

    let response = SessionResponse::from(session.clone());

    // Store session
    let mut sessions = state.sessions.write().await;
    sessions.insert(session.id.clone(), session);

    info!("Session created: {}", response.id);

    Ok(Json(response))
}

/// Get a session by ID
pub async fn get_session(
    Path(session_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Getting session: {}", session_id);

    let sessions = state.sessions.read().await;
    
    match sessions.get(&session_id) {
        Some(session) if session.is_active && session.expires_at > Utc::now() => {
            Ok(Json(SessionResponse::from(session.clone())))
        }
        Some(_) => {
            warn!("Session expired or inactive: {}", session_id);
            Err((
                StatusCode::GONE,
                Json(SessionErrorResponse {
                    code: "SESSION_EXPIRED".to_string(),
                    message: "Session has expired or is inactive".to_string(),
                }),
            ))
        }
        None => {
            debug!("Session not found: {}", session_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(SessionErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Session {} not found", session_id),
                }),
            ))
        }
    }
}

/// Validate a session
pub async fn validate_session(
    Path(session_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<SessionErrorResponse>)> {
    info!("Validating session: {}", session_id);

    let sessions = state.sessions.read().await;
    
    match sessions.get(&session_id) {
        Some(session) if session.is_active && session.expires_at > Utc::now() => {
            Ok(Json(serde_json::json!({
                "valid": true,
                "session_id": session.id,
                "user_id": session.user_id,
                "expires_at": session.expires_at.to_rfc3339()
            })))
        }
        Some(session) => {
            warn!("Session validation failed - expired or inactive: {}", session_id);
            Ok(Json(serde_json::json!({
                "valid": false,
                "session_id": session_id,
                "reason": if session.is_active { "expired" } else { "inactive" }
            })))
        }
        None => {
            debug!("Session validation failed - not found: {}", session_id);
            Ok(Json(serde_json::json!({
                "valid": false,
                "session_id": session_id,
                "reason": "not_found"
            })))
        }
    }
}

/// Revoke a session
pub async fn revoke_session(
    Path(session_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<StatusCode, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Revoking session: {}", session_id);

    let mut sessions = state.sessions.write().await;
    
    match sessions.get_mut(&session_id) {
        Some(session) => {
            session.is_active = false;
            session.updated_at = Utc::now();
            info!("Session revoked: {}", session_id);
            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            debug!("Session not found for revocation: {}", session_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(SessionErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Session {} not found", session_id),
                }),
            ))
        }
    }
}

/// List all sessions for a user
pub async fn list_sessions(
    Path(user_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<SessionListResponse>, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Listing sessions for user: {}", user_id);

    let sessions = state.sessions.read().await;
    let now = Utc::now();
    
    let user_sessions: Vec<SessionResponse> = sessions
        .values()
        .filter(|s| s.user_id == user_id && s.is_active && s.expires_at > now)
        .map(|s| SessionResponse::from(s.clone()))
        .collect();

    let total = user_sessions.len();

    Ok(Json(SessionListResponse {
        sessions: user_sessions,
        total,
    }))
}

/// Revoke all sessions for a user
pub async fn revoke_all_sessions(
    Path(user_id): Path<String>,
    State(state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<SessionErrorResponse>)> {
    debug!("Revoking all sessions for user: {}", user_id);

    let mut sessions = state.sessions.write().await;
    let now = Utc::now();
    let mut revoked_count = 0;

    for session in sessions.values_mut() {
        if session.user_id == user_id && session.is_active {
            session.is_active = false;
            session.updated_at = now;
            revoked_count += 1;
        }
    }

    info!("Revoked {} sessions for user: {}", revoked_count, user_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "revoked_count": revoked_count,
        "message": format!("Revoked {} session(s)", revoked_count)
    })))
}

/// Create the session router (without state - caller must use .with_state())
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
    fn test_session_data_creation() {
        let now = Utc::now();
        let session = SessionData {
            id: "sess-123".to_string(),
            user_id: "user-1".to_string(),
            metadata: None,
            created_at: now,
            updated_at: now,
            expires_at: now + chrono::Duration::hours(24),
            is_active: true,
        };

        assert_eq!(session.id, "sess-123");
        assert!(session.is_active);
    }

    #[test]
    fn test_session_response_from_session_data() {
        let now = Utc::now();
        let session = SessionData {
            id: "sess-123".to_string(),
            user_id: "user-1".to_string(),
            metadata: Some(serde_json::json!({"device": "web"})),
            created_at: now,
            updated_at: now,
            expires_at: now + chrono::Duration::hours(24),
            is_active: true,
        };

        let response = SessionResponse::from(session);
        assert_eq!(response.id, "sess-123");
        assert_eq!(response.user_id, "user-1");
        assert!(response.metadata.is_some());
    }
}
