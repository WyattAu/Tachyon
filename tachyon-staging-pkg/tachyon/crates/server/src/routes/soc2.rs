//! SOC 2 compliance API endpoints.
//!
//! Provides evidence collection, checklist status, and report generation
//! for SOC 2 Type II compliance.

use crate::audit::AuditLogger;
use crate::compliance::soc2::*;
use crate::error::ServerError;
use crate::middleware::AuthContext;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Soc2State {
    pub audit_logger: AuditLogger,
}

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EvidenceQuery {
    pub evidence_type: Option<String>,
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct Soc2ReportResponse {
    pub report: Soc2Report,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct Soc2EvidenceResponse {
    pub access_evidence: Vec<AccessEvidence>,
    pub change_evidence: Vec<ChangeEvidence>,
    pub monitoring_evidence: Vec<MonitoringEvidence>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct Soc2ChecklistResponse {
    pub checklist: Vec<Soc2ChecklistItem>,
    pub summary: ChecklistSummary,
}

#[derive(Debug, Serialize)]
pub struct ChecklistSummary {
    pub total_items: usize,
    pub implemented: usize,
    pub partially_implemented: usize,
    pub planned: usize,
    pub not_applicable: usize,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/compliance/soc2/report` — Generate a SOC 2 report.
pub async fn generate_report(
    State(state): State<Soc2State>,
    Extension(auth): Extension<AuthContext>,
    axum::extract::Query(params): axum::extract::Query<ReportQuery>,
) -> Result<Json<Soc2ReportResponse>, ServerError> {
    if !auth.is_admin() {
        return Err(ServerError::forbidden(
            "Admin access required for SOC 2 report",
        ));
    }

    let period_end = params
        .to
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let period_start = params
        .from
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| period_end - chrono::Duration::days(90));

    // Collect evidence from the audit logger
    let mut collector = Soc2EvidenceCollector::new();

    let events = state.audit_logger.get_events(None).await;
    for event in &events {
        let ip = event.context.ip_address.clone();
        let outcome = format!("{:?}", event.outcome).to_lowercase();
        let evidence = Soc2EvidenceCollector::from_access_snapshot(
            event.actor_id.as_deref().unwrap_or("system"),
            &event
                .target_id
                .clone()
                .unwrap_or_else(|| event.action.clone()),
            &event.action,
            ip,
            &outcome,
        );
        collector.record_access(evidence);
    }

    let report = generate_soc2_report(&collector, period_start, period_end);

    Ok(Json(Soc2ReportResponse {
        report,
        generated_at: Utc::now().to_rfc3339(),
    }))
}

/// `GET /api/v1/compliance/soc2/evidence` — List collected evidence.
pub async fn list_evidence(
    State(state): State<Soc2State>,
    Extension(auth): Extension<AuthContext>,
    axum::extract::Query(params): axum::extract::Query<EvidenceQuery>,
) -> Result<Json<Soc2EvidenceResponse>, ServerError> {
    if !auth.is_admin() {
        return Err(ServerError::forbidden(
            "Admin access required for SOC 2 evidence",
        ));
    }

    let limit = params.limit.unwrap_or(100).min(1000);
    let evidence_type = params.evidence_type.as_deref().unwrap_or("all");

    let mut collector = Soc2EvidenceCollector::new();

    let events = state.audit_logger.get_events(Some(limit)).await;
    for event in &events {
        let ip = event.context.ip_address.clone();
        let outcome = format!("{:?}", event.outcome).to_lowercase();
        let evidence = Soc2EvidenceCollector::from_access_snapshot(
            event.actor_id.as_deref().unwrap_or("system"),
            &event
                .target_id
                .clone()
                .unwrap_or_else(|| event.action.clone()),
            &event.action,
            ip,
            &outcome,
        );
        collector.record_access(evidence);
    }

    let (access, change, monitoring) = match evidence_type {
        "access" => (collector.access_evidence().to_vec(), vec![], vec![]),
        "change" => (vec![], collector.change_evidence().to_vec(), vec![]),
        "monitoring" => (vec![], vec![], collector.monitoring_evidence().to_vec()),
        _ => (
            collector.access_evidence().to_vec(),
            collector.change_evidence().to_vec(),
            collector.monitoring_evidence().to_vec(),
        ),
    };

    let total = access.len() + change.len() + monitoring.len();

    Ok(Json(Soc2EvidenceResponse {
        access_evidence: access,
        change_evidence: change,
        monitoring_evidence: monitoring,
        total,
    }))
}

/// `GET /api/v1/compliance/soc2/checklist` — Get checklist status.
pub async fn get_checklist(
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Soc2ChecklistResponse>, ServerError> {
    if !auth.is_admin() {
        return Err(ServerError::forbidden(
            "Admin access required for SOC 2 checklist",
        ));
    }

    let checklist = generate_soc2_checklist();

    let summary = ChecklistSummary {
        total_items: checklist.len(),
        implemented: checklist
            .iter()
            .filter(|i| i.status == ChecklistStatus::Implemented)
            .count(),
        partially_implemented: checklist
            .iter()
            .filter(|i| i.status == ChecklistStatus::PartiallyImplemented)
            .count(),
        planned: checklist
            .iter()
            .filter(|i| i.status == ChecklistStatus::Planned)
            .count(),
        not_applicable: checklist
            .iter()
            .filter(|i| i.status == ChecklistStatus::NotApplicable)
            .count(),
    };

    Ok(Json(Soc2ChecklistResponse { checklist, summary }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn create_soc2_router() -> axum::Router<Soc2State> {
    axum::Router::new()
        .route(
            "/compliance/soc2/report",
            axum::routing::get(generate_report),
        )
        .route(
            "/compliance/soc2/evidence",
            axum::routing::get(list_evidence),
        )
        .route(
            "/compliance/soc2/checklist",
            axum::routing::get(get_checklist),
        )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soc2_report_response_serialization() {
        let collector = Soc2EvidenceCollector::new();
        let report = generate_soc2_report(
            &collector,
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        );
        let response = Soc2ReportResponse {
            report,
            generated_at: Utc::now().to_rfc3339(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("report"));
        assert!(json.contains("generated_at"));
    }

    #[test]
    fn test_checklist_summary() {
        let checklist = generate_soc2_checklist();
        let summary = ChecklistSummary {
            total_items: checklist.len(),
            implemented: checklist
                .iter()
                .filter(|i| i.status == ChecklistStatus::Implemented)
                .count(),
            partially_implemented: checklist
                .iter()
                .filter(|i| i.status == ChecklistStatus::PartiallyImplemented)
                .count(),
            planned: checklist
                .iter()
                .filter(|i| i.status == ChecklistStatus::Planned)
                .count(),
            not_applicable: checklist
                .iter()
                .filter(|i| i.status == ChecklistStatus::NotApplicable)
                .count(),
        };
        assert!(summary.total_items > 0);
        assert!(summary.implemented > 0);
    }

    #[test]
    fn test_evidence_response_serialization() {
        let response = Soc2EvidenceResponse {
            access_evidence: vec![],
            change_evidence: vec![],
            monitoring_evidence: vec![],
            total: 0,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("total"));
    }
}
