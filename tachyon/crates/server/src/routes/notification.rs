use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{sse::Event as SseEvent, Json, Sse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tachyon_database::{DatabasePool, Notification, NotificationRepository};
use tokio::sync::broadcast;

use crate::pagination::{CursorPage, CursorParams};

#[derive(Clone)]
pub struct NotificationState {
    pub pool: DatabasePool,
    pub sse_tx: Arc<broadcast::Sender<String>>,
}

impl NotificationState {
    pub fn new(pool: DatabasePool) -> Self {
        let (sse_tx, _) = broadcast::channel(256);
        Self {
            pool,
            sse_tx: Arc::new(sse_tx),
        }
    }

    pub fn broadcast_sse(&self, data: &serde_json::Value) {
        let _ = self
            .sse_tx
            .send(serde_json::to_string(data).unwrap_or_default());
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

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListNotificationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub include_read: Option<bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NotificationListResponse {
    pub notifications: Vec<Notification>,
    pub count: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UnreadCountResponse {
    pub count: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MarkReadResponse {
    pub read: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MarkAllReadResponse {
    pub updated: u64,
}

/// List notifications for the current user.
///
/// `GET /api/v1/notifications`
///
/// Supports `limit`, `offset`, and `include_read` query parameters.
#[utoipa::path(
    get,
    path = "/notifications",
    params(
        ListNotificationsQuery,
    ),
    responses(
        (status = 200, description = "List of notifications", body = NotificationListResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
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
#[utoipa::path(
    get,
    path = "/notifications/unread-count",
    responses(
        (status = 200, description = "Unread notification count", body = UnreadCountResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
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
#[utoipa::path(
    post,
    path = "/notifications/{id}/read",
    params(
        ("id" = String, Path, description = "Notification ID"),
    ),
    responses(
        (status = 200, description = "Notification marked as read", body = MarkReadResponse),
        (status = 400, description = "Invalid notification ID"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
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
#[utoipa::path(
    post,
    path = "/notifications/read-all",
    responses(
        (status = 200, description = "All notifications marked as read", body = MarkAllReadResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
pub async fn mark_all_read(
    State(state): State<NotificationState>,
) -> Result<Json<MarkAllReadResponse>, (StatusCode, String)> {
    let updated = NotificationRepository::mark_all_read(&state.pool, uuid::Uuid::nil())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(MarkAllReadResponse { updated }))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NotificationCursorPage {
    pub data: Vec<Notification>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub total_count: Option<i64>,
}

impl From<CursorPage<Notification>> for NotificationCursorPage {
    fn from(page: CursorPage<Notification>) -> Self {
        Self {
            data: page.data,
            has_next: page.has_next,
            has_prev: page.has_prev,
            next_cursor: page.next_cursor,
            prev_cursor: page.prev_cursor,
            total_count: page.total_count,
        }
    }
}

#[utoipa::path(
    get,
    path = "/notifications/cursor",
    params(CursorParams),
    responses(
        (status = 200, description = "Cursor-paginated notifications", body = NotificationCursorPage),
        (status = 400, description = "Invalid cursor"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
pub async fn list_notifications_cursor(
    State(state): State<NotificationState>,
    Query(params): Query<CursorParams>,
) -> Result<Json<CursorPage<Notification>>, (StatusCode, String)> {
    let limit = params.limit();
    let direction = params.direction();
    let fetch_limit = (limit + 1) as i64;
    let cursor_str = params.after.as_deref().or(params.before.as_deref());

    let notifications = NotificationRepository::list_after_cursor(
        &state.pool,
        uuid::Uuid::nil(),
        fetch_limit,
        cursor_str,
        true,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_count = NotificationRepository::count_for_user(&state.pool, uuid::Uuid::nil())
        .await
        .unwrap_or(0);

    let mut items = notifications;
    let has_extra = items.len() > limit;
    if has_extra {
        items.truncate(limit);
    }

    let has_next = if direction == "asc" {
        has_extra
    } else {
        cursor_str.is_some()
    };
    let has_prev = if direction == "asc" {
        cursor_str.is_some()
    } else {
        has_extra
    };

    let first_id = items.first().map(|n| n.id.to_string());
    let last_id = items.last().map(|n| n.id.to_string());

    let page = CursorPage::new(items, has_next, has_prev)
        .with_cursors(first_id.as_deref(), last_id.as_deref(), direction)
        .with_total_count(total_count);

    Ok(Json(page))
}

/// Server-Sent Events stream for notifications.
///
/// `GET /api/v1/notifications/stream`
///
/// Fallback for clients that cannot use WebSocket. Sends `data: {json}\n\n` for each
/// new notification event published to the internal broadcast channel.
type NotificationSseStream =
    std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<SseEvent, axum::Error>> + Send>>;

fn sse_receiver_stream(rx: broadcast::Receiver<String>) -> NotificationSseStream {
    Box::pin(futures_util::stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(json) => Some((Ok(SseEvent::default().data(json)), rx)),
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                match rx.recv().await {
                    Ok(json) => Some((Ok(SseEvent::default().data(json)), rx)),
                    Err(_) => None,
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => None,
        }
    }))
}

#[utoipa::path(
    get,
    path = "/notifications/stream",
    responses(
        (status = 200, description = "SSE stream of notifications"),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
pub async fn notification_stream(
    State(state): State<NotificationState>,
) -> Sse<NotificationSseStream> {
    let rx = state.sse_tx.subscribe();
    Sse::new(sse_receiver_stream(rx))
}

pub fn create_notification_router() -> axum::Router<NotificationState> {
    axum::Router::new()
        .route("/notifications", get(list_notifications))
        .route("/notifications/cursor", get(list_notifications_cursor))
        .route("/notifications/unread-count", get(unread_count))
        .route("/notifications/read-all", post(mark_all_read))
        .route("/notifications/{id}/read", post(mark_notification_read))
        .route("/notifications/stream", get(notification_stream))
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
