use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhook {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

pub struct WebhookRepository;

impl WebhookRepository {
    pub async fn create(pool: &DatabasePool, webhook: CreateWebhook) -> DatabaseResult<Webhook> {
        let mut conn = pool.acquire().await?;
        let result = query_as::<_, Webhook>(
            r#"INSERT INTO webhooks (url, events, secret)
              VALUES ($1, $2, $3)
              RETURNING id, url, events, secret, active, created_at, last_triggered_at"#,
        )
        .bind(&webhook.url)
        .bind(&webhook.events)
        .bind(&webhook.secret)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result)
    }

    pub async fn list(pool: &DatabasePool) -> DatabaseResult<Vec<Webhook>> {
        let mut conn = pool.acquire().await?;
        let results = query_as::<_, Webhook>(
            r#"SELECT id, url, events, secret, active, created_at, last_triggered_at
              FROM webhooks
              ORDER BY created_at DESC LIMIT 100"#,
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(results)
    }

    pub async fn delete(pool: &DatabasePool, id: Uuid) -> DatabaseResult<bool> {
        let mut conn = pool.acquire().await?;
        let result = sqlx::query("DELETE FROM webhooks WHERE id = $1")
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_active_by_event(
        pool: &DatabasePool,
        event: &str,
    ) -> DatabaseResult<Vec<Webhook>> {
        let mut conn = pool.acquire().await?;
        let results = query_as::<_, Webhook>(
            r#"SELECT id, url, events, secret, active, created_at, last_triggered_at
              FROM webhooks
              WHERE active = true AND $1 = ANY(events) LIMIT 50"#,
        )
        .bind(event)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(results)
    }

    pub async fn update_last_triggered(pool: &DatabasePool, id: Uuid) -> DatabaseResult<()> {
        let mut conn = pool.acquire().await?;
        sqlx::query("UPDATE webhooks SET last_triggered_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_webhook_deserialization() {
        let json = r#"{
            "url": "https://example.com/webhook",
            "events": ["document_created", "document_updated"],
            "secret": "my-secret"
        }"#;
        let webhook: CreateWebhook = serde_json::from_str(json).unwrap();
        assert_eq!(webhook.url, "https://example.com/webhook");
        assert_eq!(webhook.events.len(), 2);
        assert_eq!(webhook.secret, Some("my-secret".to_string()));
    }

    #[test]
    fn test_create_webhook_without_secret() {
        let json = r#"{
            "url": "https://example.com/webhook",
            "events": ["document_deleted"]
        }"#;
        let webhook: CreateWebhook = serde_json::from_str(json).unwrap();
        assert_eq!(webhook.url, "https://example.com/webhook");
        assert!(webhook.secret.is_none());
    }

    #[test]
    fn test_webhook_serialization() {
        let webhook = Webhook {
            id: Uuid::nil(),
            url: "https://example.com/hook".to_string(),
            events: vec!["document_created".to_string()],
            secret: Some("s".to_string()),
            active: true,
            created_at: Utc::now(),
            last_triggered_at: None,
        };
        let json = serde_json::to_string(&webhook).unwrap();
        assert!(json.contains("document_created"));
        assert!(json.contains("https://example.com/hook"));
    }
}
