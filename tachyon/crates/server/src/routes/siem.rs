//! SIEM (Security Information and Event Management) audit export endpoints.

use crate::audit::AuditLogger;
use crate::error::ServerError;
use crate::middleware::AuthContext;
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tachyon_database::DatabasePool;

#[derive(Clone)]
pub struct SiemState {
    pub pool: DatabasePool,
    pub audit_logger: AuditLogger,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub severity: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub action: Option<String>,
    pub actor_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEventExport {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub severity: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub description: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct AuditStatsResponse {
    pub total_events: u64,
    pub events_by_severity: serde_json::Value,
    pub events_by_type: serde_json::Value,
    pub period_from: Option<String>,
    pub period_to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditListResponse {
    pub entries: serde_json::Value,
    pub total: usize,
    pub page: u32,
    pub page_size: u32,
}

fn to_cef(event: &AuditEventExport) -> String {
    let severity_num = match event.severity.as_str() {
        "critical" => "10",
        "high" => "8",
        "medium" => "5",
        "low" => "3",
        _ => "1",
    };
    format!(
        "CEF:0|Tachyon|Audit|1.0|{}|{}|{}|src={} dst=Tachyon",
        event.event_type,
        event.description,
        severity_num,
        event.ip_address.as_deref().unwrap_or("-"),
    )
}

fn to_leef(event: &AuditEventExport) -> String {
    format!(
        "LEEF:2.0|Tachyon|Audit|1.0|{}|sev={} src={} usrName={} msg={}",
        event.event_type,
        event.severity,
        event.ip_address.as_deref().unwrap_or("-"),
        event.user_id.as_deref().unwrap_or("-"),
        event.description,
    )
}

/// Paginated audit log list endpoint.
/// Reads from in-memory ring buffer, supports filtering by action and actor.
pub async fn list_audit_logs(
    State(state): State<SiemState>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<AuditListQuery>,
) -> Result<Json<serde_json::Value>, ServerError> {
    if !auth.is_admin() {
        return Err(ServerError::forbidden(
            "Admin access required for audit logs",
        ));
    }

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let action = params.action.as_deref();
    let actor_id = params.actor_id.as_deref();

    let events = state.audit_logger.get_events(None).await; // None = use default limit (10000)

    // Apply filters
    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| {
            if let Some(a) = action
                && e.action != a
            {
                return false;
            }
            if let Some(aid) = actor_id
                && e.actor_id.as_deref() != Some(aid)
            {
                return false;
            }
            true
        })
        .collect();

    let total = filtered.len();
    let skip = ((page - 1) * page_size) as usize;
    let entries: Vec<_> = filtered
        .into_iter()
        .skip(skip)
        .take(page_size as usize)
        .collect();

    let response = serde_json::json!({
        "entries": entries,
        "total": total,
        "page": page,
        "page_size": page_size,
    });

    Ok(Json(response))
}

pub async fn export_audit_events(
    State(_state): State<SiemState>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<ExportQuery>,
) -> Result<Response, ServerError> {
    if !auth.is_admin() {
        return Err(ServerError::forbidden(
            "Admin access required for audit export",
        ));
    }

    let format = params.format.as_deref().unwrap_or("json").to_lowercase();

    let sample_events = Vec::<AuditEventExport>::new();

    match format.as_str() {
        "cef" => {
            let body: String = sample_events
                .iter()
                .map(to_cef)
                .collect::<Vec<_>>()
                .join("\n");
            Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                body,
            )
                .into_response())
        }
        "leef" => {
            let body: String = sample_events
                .iter()
                .map(to_leef)
                .collect::<Vec<_>>()
                .join("\n");
            Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                body,
            )
                .into_response())
        }
        _ => Ok(Json(sample_events).into_response()),
    }
}

pub async fn get_audit_stats(
    State(_state): State<SiemState>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<ExportQuery>,
) -> Result<Response, ServerError> {
    if !auth.is_admin() {
        return Err(ServerError::forbidden(
            "Admin access required for audit stats",
        ));
    }

    let stats = AuditStatsResponse {
        total_events: 0,
        events_by_severity: serde_json::json!({}),
        events_by_type: serde_json::json!({}),
        period_from: params.from,
        period_to: params.to,
    };

    Ok(Json(stats).into_response())
}

pub fn create_siem_router() -> axum::Router<SiemState> {
    axum::Router::new()
        .route("/admin/audit", axum::routing::get(list_audit_logs))
        .route(
            "/admin/audit/export",
            axum::routing::get(export_audit_events),
        )
        .route("/admin/audit/stats", axum::routing::get(get_audit_stats))
}
