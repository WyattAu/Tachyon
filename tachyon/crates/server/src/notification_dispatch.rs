use std::sync::Arc;
use tachyon_database::{CreateNotification, DatabasePool, NotificationRepository};
use tokio::sync::broadcast;
use tracing::debug;
use uuid::Uuid;

use crate::websocket::types::WebSocketMessage;
use crate::websocket::ConnectionManager;

#[derive(Clone)]
pub struct NotificationDispatcher {
    pool: DatabasePool,
    connection_manager: Arc<ConnectionManager>,
    sse_tx: Arc<broadcast::Sender<String>>,
}

impl NotificationDispatcher {
    pub fn new(
        pool: DatabasePool,
        connection_manager: Arc<ConnectionManager>,
        sse_tx: Arc<broadcast::Sender<String>>,
    ) -> Self {
        Self {
            pool,
            connection_manager,
            sse_tx,
        }
    }

    pub async fn dispatch(
        &self,
        recipient_user_id: Uuid,
        notification_type: &str,
        title: &str,
        body: Option<&str>,
        link: Option<&str>,
        metadata: serde_json::Value,
    ) -> Result<(), tachyon_database::error::DatabaseError> {
        let notification = NotificationRepository::create(
            &self.pool,
            CreateNotification {
                user_id: recipient_user_id,
                notification_type: notification_type.to_string(),
                title: title.to_string(),
                body: body.map(|b| b.to_string()),
                link: link.map(|l| l.to_string()),
                metadata: Some(metadata.clone()),
            },
        )
        .await?;

        debug!(
            notification_id = %notification.id,
            user_id = %recipient_user_id,
            notification_type = %notification_type,
            "Notification created, broadcasting to WebSocket and SSE"
        );

        let data = serde_json::json!({
            "id": notification.id,
            "type": notification.notification_type,
            "title": notification.title,
            "body": notification.body,
            "link": notification.link,
            "read": notification.read,
            "created_at": notification.created_at,
            "recipient_user_id": recipient_user_id,
        });

        let room_id = format!("user:{}", recipient_user_id);
        let msg = WebSocketMessage::notification(room_id, data.clone());
        self.connection_manager
            .broadcast_to_room(&format!("user:{}", recipient_user_id), msg)
            .await;

        let _ = self
            .sse_tx
            .send(serde_json::to_string(&data).unwrap_or_default());

        Ok(())
    }

    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn connection_manager(&self) -> &Arc<ConnectionManager> {
        &self.connection_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_data_shape() {
        let data = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "document_shared",
            "title": "Document shared with you",
            "body": null,
            "link": "/documents/abc123",
            "read": false,
            "created_at": "2026-04-10T12:00:00Z",
        });
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("document_shared"));
        assert!(json.contains("Document shared with you"));
        assert!(json.contains("read"));
    }

    #[test]
    fn test_dispatch_creates_correct_room_id() {
        let user_id = Uuid::new_v4();
        let room_id = format!("user:{}", user_id);
        assert!(room_id.starts_with("user:"));
        assert!(room_id.len() > 5);
    }

    #[tokio::test]
    async fn test_websocket_message_notification_serialization() {
        let data = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "review_requested",
            "title": "Review requested",
            "body": null,
            "link": null,
            "read": false,
            "created_at": "2026-04-10T12:00:00Z",
        });
        let msg = WebSocketMessage::notification("user:test-user".to_string(), data);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"notification\""));
        assert!(json.contains("review_requested"));
        assert!(json.contains("\"room_id\":\"user:test-user\""));
    }
}
