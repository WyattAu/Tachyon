use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, utoipa::ToSchema)]
pub struct ActivityEvent {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub event_type: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub description: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateActivityEvent {
    pub actor_id: Uuid,
    pub event_type: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct ActivityRepository;

impl ActivityRepository {
    pub async fn create(
        pool: &DatabasePool,
        event: CreateActivityEvent,
    ) -> DatabaseResult<ActivityEvent> {
        let metadata = event.metadata.unwrap_or(serde_json::json!({}));
        let mut conn = pool.acquire().await?;
        let result = query_as::<_, ActivityEvent>(
            r#"INSERT INTO activity_events (actor_id, event_type, target_type, target_id, description, metadata)
              VALUES ($1, $2, $3, $4, $5, $6)
              RETURNING id, actor_id, event_type, target_type, target_id, description, metadata, created_at"#
        )
        .bind(event.actor_id)
        .bind(&event.event_type)
        .bind(&event.target_type)
        .bind(event.target_id)
        .bind(&event.description)
        .bind(&metadata)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result)
    }

    pub async fn list_recent(
        pool: &DatabasePool,
        limit: i64,
        offset: i64,
    ) -> DatabaseResult<Vec<ActivityEvent>> {
        let mut conn = pool.acquire().await?;
        let results = query_as::<_, ActivityEvent>(
            r#"SELECT id, actor_id, event_type, target_type, target_id, description, metadata, created_at
              FROM activity_events
              ORDER BY created_at DESC
              LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(results)
    }

    pub async fn list_by_target(
        pool: &DatabasePool,
        target_type: &str,
        target_id: Uuid,
        limit: i64,
    ) -> DatabaseResult<Vec<ActivityEvent>> {
        let mut conn = pool.acquire().await?;
        let results = query_as::<_, ActivityEvent>(
            r#"SELECT id, actor_id, event_type, target_type, target_id, description, metadata, created_at
              FROM activity_events
              WHERE target_type = $1 AND target_id = $2
              ORDER BY created_at DESC
              LIMIT $3"#
        )
        .bind(target_type)
        .bind(target_id)
        .bind(limit)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(results)
    }

    pub async fn list_by_actor(
        pool: &DatabasePool,
        actor_id: Uuid,
        limit: i64,
    ) -> DatabaseResult<Vec<ActivityEvent>> {
        let mut conn = pool.acquire().await?;
        let results = query_as::<_, ActivityEvent>(
            r#"SELECT id, actor_id, event_type, target_type, target_id, description, metadata, created_at
              FROM activity_events
              WHERE actor_id = $1
              ORDER BY created_at DESC
              LIMIT $2"#
        )
        .bind(actor_id)
        .bind(limit)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(results)
    }

    pub async fn list_after_cursor(
        pool: &DatabasePool,
        limit: i64,
        cursor: Option<&str>,
    ) -> DatabaseResult<Vec<ActivityEvent>> {
        let mut conn = pool.acquire().await?;

        if let Some(cursor_str) = cursor {
            let parts: Vec<&str> = cursor_str.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(DatabaseError::ValidationError(
                    "Invalid cursor format: expected {{id}}:{{direction}}".to_string(),
                ));
            }
            let cursor_id = parts[0];

            let results = query_as::<_, ActivityEvent>(
                r#"SELECT id, actor_id, event_type, target_type, target_id, description, metadata, created_at
                  FROM activity_events
                  WHERE id < $1::uuid
                  ORDER BY created_at DESC
                  LIMIT $2"#
            )
            .bind(cursor_id)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            Ok(results)
        } else {
            let results = query_as::<_, ActivityEvent>(
                r#"SELECT id, actor_id, event_type, target_type, target_id, description, metadata, created_at
                  FROM activity_events
                  ORDER BY created_at DESC
                  LIMIT $1"#
            )
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            Ok(results)
        }
    }

    pub async fn count(pool: &DatabasePool) -> DatabaseResult<i64> {
        let mut conn = pool.acquire().await?;
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_events")
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(row.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_event_deserialization() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "actor_id": "660e8400-e29b-41d4-a716-446655440001",
            "event_type": "document_created",
            "target_type": "document",
            "target_id": "770e8400-e29b-41d4-a716-446655440002",
            "description": "Created document: Getting Started",
            "metadata": {"title": "Getting Started"},
            "created_at": "2026-04-10T12:00:00Z"
        }"#;
        let event: ActivityEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "document_created");
        assert_eq!(event.target_type, "document");
        assert_eq!(event.description, "Created document: Getting Started");
        assert_eq!(event.metadata["title"], "Getting Started");
    }

    #[test]
    fn test_create_activity_event_deserialization() {
        let json = r#"{
            "actor_id": "660e8400-e29b-41d4-a716-446655440001",
            "event_type": "document_updated",
            "target_type": "document",
            "target_id": "770e8400-e29b-41d4-a716-446655440002",
            "description": "Updated document: Getting Started",
            "metadata": {"fields_changed": ["title"]}
        }"#;
        let event: CreateActivityEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "document_updated");
        assert!(event.metadata.is_some());
        assert_eq!(
            event.metadata.unwrap()["fields_changed"],
            serde_json::json!(["title"])
        );
    }

    #[test]
    fn test_create_activity_event_without_metadata() {
        let json = r#"{
            "actor_id": "660e8400-e29b-41d4-a716-446655440001",
            "event_type": "document_deleted",
            "target_type": "document",
            "target_id": "770e8400-e29b-41d4-a716-446655440002",
            "description": "Deleted document: Old Draft"
        }"#;
        let event: CreateActivityEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "document_deleted");
        assert!(event.metadata.is_none());
    }
}
