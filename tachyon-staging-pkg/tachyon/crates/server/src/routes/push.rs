//! Push notification registration and management.

use crate::error::ServerError;
use crate::push::web_push::{PushPayload, PushSubscription, VapidConfig};
use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tachyon_database::DatabasePool;

#[derive(Clone)]
pub struct PushState {
    pub pool: DatabasePool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterPushRequest {
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RegisterPushResponse {
    pub id: String,
    pub endpoint: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UnregisterPushRequest {
    pub endpoint: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UnregisterPushResponse {
    pub removed: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BroadcastPushRequest {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub url: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BroadcastPushResponse {
    pub sent: usize,
    pub failed: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VapidPublicKeyResponse {
    pub public_key: String,
}

#[utoipa::path(
    get,
    path = "/push/vapid-public-key",
    responses(
        (status = 200, description = "VAPID public key", body = VapidPublicKeyResponse),
    ),
    tag = "push",
)]
pub async fn vapid_public_key() -> Result<Json<VapidPublicKeyResponse>, ServerError> {
    let vapid = VapidConfig::from_env()
        .ok_or_else(|| ServerError::internal("VAPID keys not configured"))?;
    Ok(Json(VapidPublicKeyResponse {
        public_key: vapid.public_key,
    }))
}

#[utoipa::path(
    post,
    path = "/push/subscribe",
    request_body(content = RegisterPushRequest, description = "Push subscription registration"),
    responses(
        (status = 200, description = "Subscription registered", body = RegisterPushResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "push",
    security(("bearer_auth" = [])),
)]
pub async fn subscribe_push(
    State(state): State<PushState>,
    Json(body): Json<RegisterPushRequest>,
) -> Result<Json<RegisterPushResponse>, ServerError> {
    if body.endpoint.trim().is_empty() {
        return Err(ServerError::bad_request("Endpoint is required"));
    }
    if body.p256dh_key.trim().is_empty() {
        return Err(ServerError::bad_request("p256dh_key is required"));
    }
    if body.auth_key.trim().is_empty() {
        return Err(ServerError::bad_request("auth_key is required"));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::internal(format!("Failed to acquire connection: {}", e)))?;

    let row: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh_key, auth_key) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (endpoint) DO UPDATE SET p256dh_key = $4, auth_key = $5 \
         RETURNING id",
    )
    .bind(uuid::Uuid::parse_str(&id).unwrap_or_default())
    .bind(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap_or_default())
    .bind(&body.endpoint)
    .bind(&body.p256dh_key)
    .bind(&body.auth_key)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| ServerError::internal(format!("Failed to register push subscription: {}", e)))?;

    Ok(Json(RegisterPushResponse {
        id: row.0.to_string(),
        endpoint: body.endpoint,
    }))
}

#[utoipa::path(
    post,
    path = "/push/unsubscribe",
    request_body(content = UnregisterPushRequest, description = "Push subscription removal"),
    responses(
        (status = 200, description = "Subscription removed", body = UnregisterPushResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "push",
    security(("bearer_auth" = [])),
)]
pub async fn unsubscribe_push(
    State(state): State<PushState>,
    Json(body): Json<UnregisterPushRequest>,
) -> Result<Json<UnregisterPushResponse>, ServerError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::internal(format!("Failed to acquire connection: {}", e)))?;

    let result = sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = $1")
        .bind(&body.endpoint)
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            ServerError::internal(format!("Failed to unregister push subscription: {}", e))
        })?;

    Ok(Json(UnregisterPushResponse {
        removed: result.rows_affected() > 0,
    }))
}

#[utoipa::path(
    post,
    path = "/admin/push/broadcast",
    request_body(content = BroadcastPushRequest, description = "Broadcast push notification"),
    responses(
        (status = 200, description = "Broadcast sent", body = BroadcastPushResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "push",
    security(("bearer_auth" = [])),
)]
pub async fn broadcast_push(
    State(state): State<PushState>,
    Json(body): Json<BroadcastPushRequest>,
) -> Result<Json<BroadcastPushResponse>, ServerError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::internal(format!("Failed to acquire connection: {}", e)))?;

    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT endpoint, p256dh_key, auth_key FROM push_subscriptions")
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| {
                ServerError::internal(format!("Failed to fetch push subscriptions: {}", e))
            })?;

    let subscriptions: Vec<PushSubscription> = rows
        .into_iter()
        .map(|(endpoint, p256dh_key, auth_key)| PushSubscription {
            endpoint,
            p256dh_key,
            auth_key,
        })
        .collect();

    let payload = PushPayload {
        title: body.title,
        body: body.body,
        icon: body.icon,
        url: body.url,
        tag: body.tag,
    };

    let manager = crate::push::PushManager::new();
    let results = manager.broadcast(&subscriptions, &payload).await;
    let sent = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.len() - sent;

    Ok(Json(BroadcastPushResponse { sent, failed }))
}

pub fn create_push_router() -> Router<PushState> {
    Router::new()
        .route("/push/vapid-public-key", get(vapid_public_key))
        .route("/push/subscribe", post(subscribe_push))
        .route("/push/unsubscribe", post(unsubscribe_push))
        .route("/admin/push/broadcast", post(broadcast_push))
}
