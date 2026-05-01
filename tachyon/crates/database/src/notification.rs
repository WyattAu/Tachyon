use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow};
use uuid::Uuid;

/// A user-facing notification stored in the `notifications` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    /// Primary key (UUID).
    pub id: Uuid,
    /// Target user ID.
    pub user_id: Uuid,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    /// Notification kind (e.g. `review_requested`, `system`).
    pub notification_type: String,
    /// Short summary shown in the UI.
    pub title: String,
    /// Optional longer body text.
    pub body: Option<String>,
    /// Optional deep-link URL.
    pub link: Option<String>,
    /// Whether the user has read this notification.
    pub read: bool,
    /// Arbitrary JSON payload tied to the notification.
    pub metadata: serde_json::Value,
    /// Row-creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Payload for creating a new notification.
#[derive(Debug, Deserialize)]
pub struct CreateNotification {
    /// Target user ID.
    pub user_id: Uuid,
    #[serde(rename = "type")]
    /// Notification kind.
    pub notification_type: String,
    /// Short summary.
    pub title: String,
    /// Optional body text.
    pub body: Option<String>,
    /// Optional deep-link URL.
    pub link: Option<String>,
    /// Arbitrary JSON payload.
    pub metadata: Option<serde_json::Value>,
}

/// Stateless repository for notification persistence (takes `&DatabasePool` per call).
pub struct NotificationRepository;

impl NotificationRepository {
    /// Insert a new notification and return the persisted row.
    pub async fn create(
        pool: &DatabasePool,
        notification: CreateNotification,
    ) -> DatabaseResult<Notification> {
        let metadata = notification.metadata.unwrap_or(serde_json::json!({}));
        let mut conn = pool.acquire().await?;
        let result = query_as::<_, Notification>(
            r#"INSERT INTO notifications (user_id, type, title, body, link, metadata)
              VALUES ($1, $2, $3, $4, $5, $6)
              RETURNING id, user_id, type, title, body, link, read, metadata, created_at"#,
        )
        .bind(notification.user_id)
        .bind(&notification.notification_type)
        .bind(&notification.title)
        .bind(&notification.body)
        .bind(&notification.link)
        .bind(&metadata)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result)
    }

    /// List notifications for a user, newest first.
    ///
    /// When `include_read` is `false`, only unread notifications are returned.
    pub async fn list_for_user(
        pool: &DatabasePool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        include_read: bool,
    ) -> DatabaseResult<Vec<Notification>> {
        let mut conn = pool.acquire().await?;
        if include_read {
            let results = query_as::<_, Notification>(
                r#"SELECT id, user_id, type, title, body, link, read, metadata, created_at
                  FROM notifications
                  WHERE user_id = $1
                  ORDER BY created_at DESC
                  LIMIT $2 OFFSET $3"#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            Ok(results)
        } else {
            let results = query_as::<_, Notification>(
                r#"SELECT id, user_id, type, title, body, link, read, metadata, created_at
                  FROM notifications
                  WHERE user_id = $1 AND read = false
                  ORDER BY created_at DESC
                  LIMIT $2 OFFSET $3"#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            Ok(results)
        }
    }

    /// Count unread notifications for a user.
    pub async fn get_unread_count(pool: &DatabasePool, user_id: Uuid) -> DatabaseResult<i64> {
        let mut conn = pool.acquire().await?;
        let row: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) as count FROM notifications WHERE user_id = $1 AND read = false"#,
        )
        .bind(user_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(row.0)
    }

    /// Mark a single notification as read. Returns `true` if the row was updated.
    pub async fn mark_read(
        pool: &DatabasePool,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> DatabaseResult<bool> {
        let mut conn = pool.acquire().await?;
        let result = sqlx::query(
            r#"UPDATE notifications SET read = true WHERE id = $1 AND user_id = $2 AND read = false"#
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }

    /// Mark all unread notifications for a user as read. Returns the number of rows updated.
    pub async fn mark_all_read(pool: &DatabasePool, user_id: Uuid) -> DatabaseResult<u64> {
        let mut conn = pool.acquire().await?;
        let result = sqlx::query(
            r#"UPDATE notifications SET read = true WHERE user_id = $1 AND read = false"#,
        )
        .bind(user_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_deserialization() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "user_id": "660e8400-e29b-41d4-a716-446655440001",
            "type": "review_requested",
            "title": "Review requested on Getting Started",
            "body": "A new review has been requested",
            "link": "/documents/abc123",
            "read": false,
            "metadata": {"document_id": "abc123"},
            "created_at": "2026-04-10T12:00:00Z"
        }"#;
        let notification: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notification.notification_type, "review_requested");
        assert_eq!(notification.title, "Review requested on Getting Started");
        assert_eq!(notification.metadata["document_id"], "abc123");
        assert!(!notification.read);
    }

    #[test]
    fn test_create_notification_deserialization() {
        let json = r#"{
            "user_id": "660e8400-e29b-41d4-a716-446655440001",
            "type": "review_approved",
            "title": "Review approved for Getting Started",
            "body": "Your review has been approved",
            "metadata": {"review_id": "rev-123"}
        }"#;
        let notification: CreateNotification = serde_json::from_str(json).unwrap();
        assert_eq!(notification.notification_type, "review_approved");
        assert!(notification.metadata.is_some());
        assert_eq!(notification.metadata.unwrap()["review_id"], "rev-123");
    }

    #[test]
    fn test_create_notification_without_optional_fields() {
        let json = r#"{
            "user_id": "660e8400-e29b-41d4-a716-446655440001",
            "type": "system",
            "title": "System maintenance notice"
        }"#;
        let notification: CreateNotification = serde_json::from_str(json).unwrap();
        assert_eq!(notification.notification_type, "system");
        assert!(notification.body.is_none());
        assert!(notification.link.is_none());
        assert!(notification.metadata.is_none());
    }
}
