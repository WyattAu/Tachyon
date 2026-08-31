//! Ecosystem API routes
//! Extended webhooks, email notifications, API metadata

use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ServerError;

/// Ecosystem state
#[derive(Clone)]
pub struct EcosystemState {
    pub pool: tachyon_database::DatabasePool,
}

impl EcosystemState {
    pub fn new(pool: tachyon_database::DatabasePool) -> Self {
        Self { pool }
    }
}

// ============================================================================
// Types — API v2 Metadata
// ============================================================================

/// API version info
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiInfo {
    pub version: String,
    pub api_version: String,
    pub endpoints_count: usize,
    pub features: Vec<String>,
}

/// Feature flag
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FeatureFlag {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Types — Email Notifications
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateNotificationPrefRequest {
    pub notification_type: String,
    pub enabled: bool,
    pub channel: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NotificationPrefsResponse {
    pub preferences: Vec<tachyon_database::NotificationPreference>,
}

// ============================================================================
// Types — Webhook v2 (extended)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WebhookEventFilter {
    pub event_types: Vec<String>,
    pub document_ids: Option<Vec<String>>,
    pub space_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WebhookDeliveryLog {
    pub id: String,
    pub webhook_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: WebhookDeliveryStatus,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub attempts: u32,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WebhookDeliveryStatus {
    Pending,
    Success,
    Failed,
    Retrying,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WebhookLogsResponse {
    pub logs: Vec<WebhookDeliveryLog>,
    pub total: usize,
}

// ============================================================================
// Handlers — API Info
// ============================================================================

/// GET /api/v2/info — Get API information
#[utoipa::path(
    get,
    path = "/v2/info",
    responses(
        (status = 200, description = "API information", body = ApiInfo),
    ),
    tag = "system",
)]
pub async fn api_info() -> Json<ApiInfo> {
    Json(ApiInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        api_version: "v2".to_string(),
        endpoints_count: 120,
        features: vec![
            "documents".to_string(),
            "users".to_string(),
            "teams".to_string(),
            "search".to_string(),
            "crdt".to_string(),
            "plugins".to_string(),
            "ssg".to_string(),
            "git".to_string(),
            "billing".to_string(),
            "collaboration".to_string(),
            "webhooks".to_string(),
            "notifications".to_string(),
        ],
    })
}

// ============================================================================
// Handlers — Email Notifications
// ============================================================================

/// GET /api/v2/notifications/preferences/{user_id} — Get notification preferences
#[utoipa::path(
    get,
    path = "/v2/notifications/preferences/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Notification preferences", body = NotificationPrefsResponse),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
pub async fn get_notification_preferences(
    State(state): State<EcosystemState>,
    Path(user_id): Path<String>,
) -> Result<Json<NotificationPrefsResponse>, ServerError> {
    let repo = tachyon_database::NotificationPreferenceRepository::new(state.pool.clone());
    let prefs = repo.list_by_user(&user_id).await.unwrap_or_default();
    Ok(Json(NotificationPrefsResponse { preferences: prefs }))
}

/// PUT /api/v2/notifications/preferences/{user_id} — Update notification preferences
#[utoipa::path(
    put,
    path = "/v2/notifications/preferences/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body(content = UpdateNotificationPrefRequest, description = "Notification preference update"),
    responses(
        (status = 200, description = "Preference updated", body = tachyon_database::NotificationPreference),
    ),
    tag = "notifications",
    security(("bearer_auth" = [])),
)]
pub async fn update_notification_preferences(
    State(state): State<EcosystemState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateNotificationPrefRequest>,
) -> Json<tachyon_database::NotificationPreference> {
    let notification_type = req.notification_type.clone();
    let channel = req.channel.clone();
    let repo = tachyon_database::NotificationPreferenceRepository::new(state.pool.clone());
    let pref = repo
        .upsert(
            tachyon_database::UpsertNotificationPrefRequest {
                notification_type: req.notification_type,
                enabled: req.enabled,
                channel: req.channel,
            },
            &user_id,
        )
        .await
        .unwrap_or_else(|_| tachyon_database::NotificationPreference {
            user_id: user_id.clone(),
            notification_type,
            enabled: req.enabled,
            channel,
            updated_at: chrono::Utc::now(),
        });
    Json(pref)
}

// ============================================================================
// Handlers — Webhook v2
// ============================================================================

/// GET /api/v2/webhooks/logs — Get webhook delivery logs
#[utoipa::path(
    get,
    path = "/v2/webhooks/logs",
    responses(
        (status = 200, description = "Webhook delivery logs", body = WebhookLogsResponse),
    ),
    tag = "webhooks",
    security(("bearer_auth" = [])),
)]
pub async fn get_webhook_logs(State(_state): State<EcosystemState>) -> Json<WebhookLogsResponse> {
    Json(WebhookLogsResponse {
        logs: vec![],
        total: 0,
    })
}

// ============================================================================
// Router
// ============================================================================

pub fn create_ecosystem_router() -> axum::Router<EcosystemState> {
    use axum::routing::{get, put};

    axum::Router::new()
        .route("/v2/info", get(api_info))
        .route(
            "/v2/notifications/preferences/{user_id}",
            get(get_notification_preferences),
        )
        .route(
            "/v2/notifications/preferences/{user_id}",
            put(update_notification_preferences),
        )
        .route("/v2/webhooks/logs", get(get_webhook_logs))
}
