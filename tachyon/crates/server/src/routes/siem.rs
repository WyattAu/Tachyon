//! SIEM (Security Information and Event Management) audit export endpoints.

use crate::error::ServerError;
use crate::middleware::AuthContext;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::DatabasePool;

#[derive(Clone)]
pub struct SiemState {
    pub pool: DatabasePool,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub severity: Option<String>,
    pub event_type: Option<String>,
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
        .route(
            "/admin/audit/export",
            axum::routing::get(export_audit_events),
        )
        .route("/admin/audit/stats", axum::routing::get(get_audit_stats))
}
