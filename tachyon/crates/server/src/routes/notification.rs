use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tachyon_database::{DatabasePool, Notification, NotificationRepository};

#[derive(Clone)]
pub struct NotificationState {
    pub pool: DatabasePool,
}

impl NotificationState {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    pub fn create_notification_email(
        email: crate::email::EmailService,
        to: &str,
        notification_type: &str,
        title: &str,
        body: &str,
    ) {
        let to = to.to_string();
        let notification_type = notification_type.to_string();
        let title = title.to_string();
        let body = body.to_string();
        tokio::spawn(async move {
            if let Err(e) = email
                .send_notification(&to, &notification_type, &title, &body, None)
                .await
            {
                tracing::warn!("Failed to send notification email: {}", e);
            }
        });
    }
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub include_read: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub notifications: Vec<Notification>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct MarkReadResponse {
    pub read: bool,
}

#[derive(Debug, Serialize)]
pub struct MarkAllReadResponse {
    pub updated: u64,
}

/// List notifications for the current user.
///
/// `GET /api/v1/notifications`
///
/// Supports `limit`, `offset`, and `include_read` query parameters.
pub async fn list_notifications(
    State(state): State<NotificationState>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<NotificationListResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);
    let include_read = query.include_read.unwrap_or(true);

    let notifications = NotificationRepository::list_for_user(
        &state.pool,
        uuid::Uuid::nil(),
        limit,
        offset,
        include_read,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count = notifications.len();
    Ok(Json(NotificationListResponse {
        notifications,
        count,
    }))
}

/// Get the unread notification count.
///
/// `GET /api/v1/notifications/unread-count`
pub async fn unread_count(
    State(state): State<NotificationState>,
) -> Result<Json<UnreadCountResponse>, (StatusCode, String)> {
    let count = NotificationRepository::get_unread_count(&state.pool, uuid::Uuid::nil())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(UnreadCountResponse { count }))
}

/// Mark a single notification as read.
///
/// `POST /api/v1/notifications/{id}/read`
pub async fn mark_notification_read(
    Path(notification_id): Path<String>,
    State(state): State<NotificationState>,
) -> Result<Json<MarkReadResponse>, (StatusCode, String)> {
    let id = uuid::Uuid::parse_str(&notification_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid notification ID: {}", e),
        )
    })?;
    let read = NotificationRepository::mark_read(&state.pool, id, uuid::Uuid::nil())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(MarkReadResponse { read }))
}

/// Mark all notifications as read.
///
/// `POST /api/v1/notifications/read-all`
pub async fn mark_all_read(
    State(state): State<NotificationState>,
) -> Result<Json<MarkAllReadResponse>, (StatusCode, String)> {
    let updated = NotificationRepository::mark_all_read(&state.pool, uuid::Uuid::nil())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(MarkAllReadResponse { updated }))
}

pub fn create_notification_router() -> axum::Router<NotificationState> {
    axum::Router::new()
        .route("/notifications", get(list_notifications))
        .route("/notifications/unread-count", get(unread_count))
        .route("/notifications/read-all", post(mark_all_read))
        .route("/notifications/{id}/read", post(mark_notification_read))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_notifications_query_deserialization() {
        let json = r#"{"limit": 25, "offset": 10, "include_read": false}"#;
        let query: ListNotificationsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.offset, Some(10));
        assert_eq!(query.include_read, Some(false));
    }

    #[test]
    fn test_notification_list_response_construction() {
        let response = NotificationListResponse {
            notifications: vec![],
            count: 0,
        };
        assert_eq!(response.count, 0);
        assert!(response.notifications.is_empty());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("\"notifications\":[]"));
    }
}
