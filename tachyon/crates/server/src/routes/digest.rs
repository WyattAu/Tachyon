//! Email digest subscriptions — daily/weekly document updates.

use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tachyon_database::DatabasePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DigestSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub frequency: String,
    pub last_sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub frequency: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub frequency: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub frequency: String,
    pub is_active: bool,
    pub last_sent_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DigestContent {
    pub new_documents: Vec<DigestDocumentItem>,
    pub updated_documents: Vec<DigestDocumentItem>,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Serialize)]
pub struct DigestDocumentItem {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub updated_at: String,
    pub author: Option<String>,
}

#[derive(Clone)]
pub struct DigestState {
    pub pool: DatabasePool,
}

pub fn create_digest_router() -> Router<DigestState> {
    Router::new()
        .route("/digest/subscribe", post(subscribe))
        .route("/digest/subscriptions", get(list_subscriptions))
        .route("/digest/subscriptions/{id}", post(update_subscription))
        .route("/digest/unsubscribe/{id}", post(unsubscribe))
}

async fn subscribe(
    State(state): State<DigestState>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
    axum::Json(req): axum::Json<CreateSubscriptionRequest>,
) -> Result<Json<SubscriptionResponse>, crate::error::ServerError> {
    if !matches!(req.frequency.as_str(), "daily" | "weekly") {
        return Err(crate::error::ServerError::bad_request(
            "frequency must be 'daily' or 'weekly'",
        ));
    }
    let row = sqlx::query_as::<_, DigestSubscription>(
        r#"INSERT INTO digest_subscriptions (user_id, frequency) VALUES ($1, $2)
           ON CONFLICT (user_id) DO UPDATE SET frequency = $2, is_active = true
           RETURNING id, user_id, frequency, last_sent_at, is_active, created_at"#,
    )
    .bind(user_id)
    .bind(&req.frequency)
    .fetch_one(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;
    Ok(Json(SubscriptionResponse {
        id: row.id.to_string(),
        frequency: row.frequency,
        is_active: row.is_active,
        last_sent_at: row.last_sent_at.map(|t| t.to_rfc3339()),
    }))
}

async fn list_subscriptions(
    State(state): State<DigestState>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
) -> Result<Json<Vec<SubscriptionResponse>>, crate::error::ServerError> {
    let rows = sqlx::query_as::<_, DigestSubscription>(
        "SELECT * FROM digest_subscriptions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;
    Ok(Json(
        rows.into_iter()
            .map(|r| SubscriptionResponse {
                id: r.id.to_string(),
                frequency: r.frequency,
                is_active: r.is_active,
                last_sent_at: r.last_sent_at.map(|t| t.to_rfc3339()),
            })
            .collect(),
    ))
}

async fn update_subscription(
    State(state): State<DigestState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    axum::Json(req): axum::Json<UpdateSubscriptionRequest>,
) -> Result<Json<SubscriptionResponse>, crate::error::ServerError> {
    if let Some(ref f) = req.frequency
        && !matches!(f.as_str(), "daily" | "weekly")
    {
        return Err(crate::error::ServerError::bad_request(
            "frequency must be 'daily' or 'weekly'",
        ));
    }
    let row = sqlx::query_as::<_, DigestSubscription>(
        r#"UPDATE digest_subscriptions SET
           frequency = COALESCE($2, frequency),
           is_active = COALESCE($3, is_active)
           WHERE id = $1 RETURNING *"#,
    )
    .bind(id)
    .bind(&req.frequency)
    .bind(req.is_active)
    .fetch_one(state.pool.inner())
    .await
    .map_err(crate::error::ServerError::from)?;
    Ok(Json(SubscriptionResponse {
        id: row.id.to_string(),
        frequency: row.frequency,
        is_active: row.is_active,
        last_sent_at: row.last_sent_at.map(|t| t.to_rfc3339()),
    }))
}

async fn unsubscribe(
    State(state): State<DigestState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, crate::error::ServerError> {
    sqlx::query("UPDATE digest_subscriptions SET is_active = false WHERE id = $1")
        .bind(id)
        .execute(state.pool.inner())
        .await
        .map_err(crate::error::ServerError::from)?;
    Ok(Json(serde_json::json!({"unsubscribed": true})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_validation() {
        assert!(matches!("daily".to_string().as_str(), "daily" | "weekly"));
        assert!(matches!("weekly".to_string().as_str(), "daily" | "weekly"));
        assert!(!matches!(
            "monthly".to_string().as_str(),
            "daily" | "weekly"
        ));
    }

    #[test]
    fn test_subscription_response_serialization() {
        let resp = SubscriptionResponse {
            id: Uuid::new_v4().to_string(),
            frequency: "daily".to_string(),
            is_active: true,
            last_sent_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("daily"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_digest_content_serialization() {
        let content = DigestContent {
            new_documents: vec![DigestDocumentItem {
                id: "1".to_string(),
                title: "Test".to_string(),
                slug: "test".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                author: Some("user".to_string()),
            }],
            updated_documents: vec![],
            period_start: "2026-01-01".to_string(),
            period_end: "2026-01-02".to_string(),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("Test"));
    }
}
