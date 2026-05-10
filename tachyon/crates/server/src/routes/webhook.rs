use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{CreateWebhook, DatabasePool, WebhookRepository};
use tracing::info;

#[derive(Clone)]
pub struct WebhookState {
    pub pool: DatabasePool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WebhookResponse {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub created_at: String,
    pub last_triggered_at: Option<String>,
}

impl From<tachyon_database::Webhook> for WebhookResponse {
    fn from(w: tachyon_database::Webhook) -> Self {
        Self {
            id: w.id.to_string(),
            url: w.url,
            events: w.events,
            active: w.active,
            created_at: w.created_at.to_rfc3339(),
            last_triggered_at: w.last_triggered_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWebhookBody {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

/// Create a new outgoing webhook.
///
/// `POST /api/v1/webhooks`
///
/// Requires at least one event type in the `events` array.
#[utoipa::path(
    post,
    path = "/webhooks",
    request_body(content = CreateWebhookBody, description = "Webhook creation request"),
    responses(
        (status = 200, description = "Webhook created", body = WebhookResponse),
        (status = 400, description = "At least one event required"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "webhooks",
    security(("bearer_auth" = [])),
)]
pub async fn create_webhook(
    State(state): State<WebhookState>,
    Json(body): Json<CreateWebhookBody>,
) -> Result<Json<WebhookResponse>, (StatusCode, Json<ErrorResponse>)> {
    if body.events.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "At least one event must be specified".to_string(),
            }),
        ));
    }

    let webhook = WebhookRepository::create(
        &state.pool,
        CreateWebhook {
            url: body.url,
            events: body.events,
            secret: body.secret,
        },
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "CREATE_ERROR".to_string(),
                message: format!("Failed to create webhook: {}", e),
            }),
        )
    })?;

    info!("Webhook created: {} -> {}", webhook.id, webhook.url);
    Ok(Json(WebhookResponse::from(webhook)))
}

/// List all outgoing webhooks.
///
/// `GET /api/v1/webhooks`
#[utoipa::path(
    get,
    path = "/webhooks",
    responses(
        (status = 200, description = "List of webhooks", body = Vec<WebhookResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "webhooks",
    security(("bearer_auth" = [])),
)]
pub async fn list_webhooks(
    State(state): State<WebhookState>,
) -> Result<Json<Vec<WebhookResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let webhooks = WebhookRepository::list(&state.pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "QUERY_ERROR".to_string(),
                message: format!("Failed to list webhooks: {}", e),
            }),
        )
    })?;

    Ok(Json(
        webhooks.into_iter().map(WebhookResponse::from).collect(),
    ))
}

/// Delete an outgoing webhook.
///
/// `DELETE /api/v1/webhooks/{id}`
#[utoipa::path(
    delete,
    path = "/webhooks/{id}",
    params(
        ("id" = String, Path, description = "Webhook ID"),
    ),
    responses(
        (status = 204, description = "Webhook deleted"),
        (status = 400, description = "Invalid webhook ID"),
        (status = 404, description = "Webhook not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "webhooks",
    security(("bearer_auth" = [])),
)]
pub async fn delete_webhook(
    Path(id): Path<String>,
    State(state): State<WebhookState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid::Uuid::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "INVALID_ID".to_string(),
                message: format!("Invalid webhook ID: {}", id),
            }),
        )
    })?;

    let deleted = WebhookRepository::delete(&state.pool, uuid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "DELETE_ERROR".to_string(),
                    message: format!("Failed to delete webhook: {}", e),
                }),
            )
        })?;

    if !deleted {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Webhook {} not found", id),
            }),
        ));
    }

    info!("Webhook deleted: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

pub fn create_webhook_router() -> axum::Router<WebhookState> {
    axum::Router::new()
        .route("/webhooks", axum::routing::post(create_webhook))
        .route("/webhooks", axum::routing::get(list_webhooks))
        .route("/webhooks/{id}", axum::routing::delete(delete_webhook))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_webhook_body_deserialization() {
        let body = CreateWebhookBody {
            url: "https://example.com/hook".to_string(),
            events: vec![
                "document_created".to_string(),
                "document_updated".to_string(),
            ],
            secret: Some("s3cret".to_string()),
        };
        assert_eq!(body.url, "https://example.com/hook");
        assert_eq!(body.events.len(), 2);
    }

    #[test]
    fn test_webhook_response_serialization() {
        let resp = WebhookResponse {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            url: "https://example.com".to_string(),
            events: vec!["document_created".to_string()],
            active: true,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
            last_triggered_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("document_created"));
        assert!(json.contains("https://example.com"));
    }
}
