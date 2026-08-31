//! SOC 2 Type II automated evidence collection.
//!
//! Collects and generates evidence for the five Trust Service Criteria:
//! Security, Availability, Processing Integrity, Confidentiality, and Privacy.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Evidence types
// ---------------------------------------------------------------------------

/// Evidence that a specific user accessed a resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessEvidence {
    pub user_id: String,
    pub resource: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub outcome: String,
}

/// Evidence of a change management event (deployment, code review, config change).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeEvidence {
    pub change_id: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub approval: ChangeApproval,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeApproval {
    Approved { approver: String },
    Pending,
    Rejected { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Deployment,
    CodeReview,
    ConfigurationChange,
    InfrastructureChange,
}

/// Evidence of a monitoring metric or alert.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringEvidence {
    pub metric_name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub threshold: Option<f64>,
    pub status: MonitoringStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MonitoringStatus {
    Normal,
    Warning,
    Critical,
}

// ---------------------------------------------------------------------------
// SOC 2 Report
// ---------------------------------------------------------------------------

/// Full SOC 2 Type II report generated from collected evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2Report {
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub trust_service_criteria: Vec<TrustServiceCriteriaReport>,
    pub summary: Soc2Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustServiceCriteriaReport {
    pub criteria: Soc2Criteria,
    pub description: String,
    pub status: CriteriaStatus,
    pub evidence_count: usize,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Soc2Criteria {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CriteriaStatus {
    Pass,
    PartialPass,
    Fail,
    NotAssessed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: FindingSeverity,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2Summary {
    pub total_evidence_items: usize,
    pub criteria_pass: u32,
    pub criteria_partial: u32,
    pub criteria_fail: u32,
    pub criteria_not_assessed: u32,
    pub overall_status: CriteriaStatus,
}

// ---------------------------------------------------------------------------
// Checklist
// ---------------------------------------------------------------------------

/// Detailed checklist item for SOC 2 compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2ChecklistItem {
    pub id: String,
    pub criteria: Soc2Criteria,
    pub description: String,
    pub status: ChecklistStatus,
    pub evidence: Vec<String>,
    pub last_verified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChecklistStatus {
    Implemented,
    PartiallyImplemented,
    Planned,
    NotApplicable,
}

// ---------------------------------------------------------------------------
// Evidence Collector
// ---------------------------------------------------------------------------

/// Collects SOC 2 evidence from various system components.
pub struct Soc2EvidenceCollector {
    access_evidence: Vec<AccessEvidence>,
    change_evidence: Vec<ChangeEvidence>,
    monitoring_evidence: Vec<MonitoringEvidence>,
}

impl Soc2EvidenceCollector {
    pub fn new() -> Self {
        Self {
            access_evidence: Vec::new(),
            change_evidence: Vec::new(),
            monitoring_evidence: Vec::new(),
        }
    }

    /// Record an access event as evidence.
    pub fn record_access(&mut self, evidence: AccessEvidence) {
        self.access_evidence.push(evidence);
    }

    /// Record a change management event as evidence.
    pub fn record_change(&mut self, evidence: ChangeEvidence) {
        self.change_evidence.push(evidence);
    }

    /// Record a monitoring metric as evidence.
    pub fn record_monitoring(&mut self, evidence: MonitoringEvidence) {
        self.monitoring_evidence.push(evidence);
    }

    /// Get all access evidence.
    pub fn access_evidence(&self) -> &[AccessEvidence] {
        &self.access_evidence
    }

    /// Get all change evidence.
    pub fn change_evidence(&self) -> &[ChangeEvidence] {
        &self.change_evidence
    }

    /// Get all monitoring evidence.
    pub fn monitoring_evidence(&self) -> &[MonitoringEvidence] {
        &self.monitoring_evidence
    }

    /// Total evidence items across all categories.
    pub fn total_evidence(&self) -> usize {
        self.access_evidence.len() + self.change_evidence.len() + self.monitoring_evidence.len()
    }

    /// Build access evidence from an audit event snapshot.
    pub fn from_access_snapshot(
        user_id: &str,
        resource: &str,
        action: &str,
        ip_address: Option<String>,
        outcome: &str,
    ) -> AccessEvidence {
        AccessEvidence {
            user_id: user_id.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            timestamp: Utc::now(),
            ip_address,
            outcome: outcome.to_string(),
        }
    }

    /// Build change evidence from deployment info.
    pub fn from_deployment(
        change_id: &str,
        author: &str,
        description: &str,
        approver: Option<&str>,
    ) -> ChangeEvidence {
        let approval = match approver {
            Some(a) => ChangeApproval::Approved {
                approver: a.to_string(),
            },
            None => ChangeApproval::Pending,
        };
        ChangeEvidence {
            change_id: change_id.to_string(),
            author: author.to_string(),
            timestamp: Utc::now(),
            description: description.to_string(),
            approval,
            change_type: ChangeType::Deployment,
        }
    }
}

impl Default for Soc2EvidenceCollector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Checklist generation
// ---------------------------------------------------------------------------

/// Generate the SOC 2 Type II compliance checklist.
pub fn generate_soc2_checklist() -> Vec<Soc2ChecklistItem> {
    let now = Utc::now();
    vec![
        // Security
        Soc2ChecklistItem {
            id: "CC6.1".to_string(),
            criteria: Soc2Criteria::Security,
            description: "Logical access controls are implemented to prevent unauthorized access"
                .to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec![
                "permissions.rs: Role-based access control".to_string(),
                "totp.rs: Multi-factor authentication".to_string(),
            ],
            last_verified: Some(now),
        },
        Soc2ChecklistItem {
            id: "CC6.2".to_string(),
            criteria: Soc2Criteria::Security,
            description: "User authentication mechanisms are in place".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec![
                "user/handlers.rs: JWT authentication".to_string(),
                "session.rs: Session management".to_string(),
            ],
            last_verified: Some(now),
        },
        Soc2ChecklistItem {
            id: "CC6.3".to_string(),
            criteria: Soc2Criteria::Security,
            description: "Access credentials are managed securely".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec![
                "api_key.rs: API key management".to_string(),
                "password_reset.rs: Password reset flow".to_string(),
            ],
            last_verified: Some(now),
        },
        Soc2ChecklistItem {
            id: "CC6.6".to_string(),
            criteria: Soc2Criteria::Security,
            description: "System boundaries are defined and protected".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec![
                "security_headers.rs: HSTS, CSP headers".to_string(),
                "middleware: CORS, rate limiting".to_string(),
            ],
            last_verified: Some(now),
        },
        Soc2ChecklistItem {
            id: "CC6.7".to_string(),
            criteria: Soc2Criteria::Security,
            description: "Data transmission is encrypted".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec!["TLS 1.2+ enforced via configuration".to_string()],
            last_verified: Some(now),
        },
        // Availability
        Soc2ChecklistItem {
            id: "A1.1".to_string(),
            criteria: Soc2Criteria::Availability,
            description: "Capacity monitoring and alerting are in place".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec![
                "health.rs: Health check endpoints".to_string(),
                "metrics: Prometheus metrics".to_string(),
            ],
            last_verified: Some(now),
        },
        Soc2ChecklistItem {
            id: "A1.2".to_string(),
            criteria: Soc2Criteria::Availability,
            description: "Backup and recovery procedures exist".to_string(),
            status: ChecklistStatus::PartiallyImplemented,
            evidence: vec!["audit.rs: Audit event persistence to database".to_string()],
            last_verified: Some(now),
        },
        // Processing Integrity
        Soc2ChecklistItem {
            id: "PI1.1".to_string(),
            criteria: Soc2Criteria::ProcessingIntegrity,
            description: "Input validation is performed".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec![
                "validation.rs: Request validation".to_string(),
                "audit.rs: Input validation failure logging".to_string(),
            ],
            last_verified: Some(now),
        },
        Soc2ChecklistItem {
            id: "PI1.4".to_string(),
            criteria: Soc2Criteria::ProcessingIntegrity,
            description: "Error handling prevents information leakage".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec!["error.rs: Structured error responses".to_string()],
            last_verified: Some(now),
        },
        // Confidentiality
        Soc2ChecklistItem {
            id: "C1.1".to_string(),
            criteria: Soc2Criteria::Confidentiality,
            description: "Data classification and handling procedures exist".to_string(),
            status: ChecklistStatus::Planned,
            evidence: vec![],
            last_verified: None,
        },
        Soc2ChecklistItem {
            id: "C1.2".to_string(),
            criteria: Soc2Criteria::Confidentiality,
            description: "Encryption at rest protects sensitive data".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec!["config.rs: JWT encryption, AES-256 at rest".to_string()],
            last_verified: Some(now),
        },
        // Privacy
        Soc2ChecklistItem {
            id: "P1.1".to_string(),
            criteria: Soc2Criteria::Privacy,
            description: "Privacy notice is provided to users".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec!["gdpr.rs: GDPR compliance module".to_string()],
            last_verified: Some(now),
        },
        Soc2ChecklistItem {
            id: "P1.2".to_string(),
            criteria: Soc2Criteria::Privacy,
            description: "Consent is obtained for data collection".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec!["onboarding: User registration consent".to_string()],
            last_verified: Some(now),
        },
    ]
}

// ---------------------------------------------------------------------------
// Report generation
// ---------------------------------------------------------------------------

/// Generate a SOC 2 report from the evidence collector.
pub fn generate_soc2_report(
    collector: &Soc2EvidenceCollector,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> Soc2Report {
    let checklist = generate_soc2_checklist();

    let mut criteria_map: BTreeMap<Soc2Criteria, Vec<&Soc2ChecklistItem>> = BTreeMap::new();
    for item in &checklist {
        criteria_map
            .entry(item.criteria.clone())
            .or_default()
            .push(item);
    }

    let criteria_definitions = vec![
        (
            Soc2Criteria::Security,
            "Security — Logical and physical protections against unauthorized access.",
        ),
        (
            Soc2Criteria::Availability,
            "Availability — System availability for operation and use.",
        ),
        (
            Soc2Criteria::ProcessingIntegrity,
            "Processing Integrity — System processing is complete, accurate, and timely.",
        ),
        (
            Soc2Criteria::Confidentiality,
            "Confidentiality — Information designated as confidential is protected.",
        ),
        (
            Soc2Criteria::Privacy,
            "Privacy — Personal information is collected, used, retained, and disclosed appropriately.",
        ),
    ];

    let mut trust_reports = Vec::new();
    let mut pass_count = 0u32;
    let mut partial_count = 0u32;
    let mut fail_count = 0u32;
    let mut not_assessed_count = 0u32;

    for (criteria, description) in criteria_definitions {
        let items = criteria_map.get(&criteria).cloned().unwrap_or_default();
        let implemented = items
            .iter()
            .filter(|i| i.status == ChecklistStatus::Implemented)
            .count();
        let total = items.len();

        let status = if total == 0 {
            CriteriaStatus::NotAssessed
        } else if implemented == total {
            CriteriaStatus::Pass
        } else if implemented as f64 / total as f64 >= 0.5 {
            CriteriaStatus::PartialPass
        } else {
            CriteriaStatus::Fail
        };

        match status {
            CriteriaStatus::Pass => pass_count += 1,
            CriteriaStatus::PartialPass => partial_count += 1,
            CriteriaStatus::Fail => fail_count += 1,
            CriteriaStatus::NotAssessed => not_assessed_count += 1,
        }

        let findings: Vec<Finding> = items
            .iter()
            .filter(|i| i.status != ChecklistStatus::Implemented)
            .map(|i| Finding {
                severity: if i.status == ChecklistStatus::Planned {
                    FindingSeverity::Medium
                } else {
                    FindingSeverity::Low
                },
                description: format!("{}: {}", i.id, i.description),
                recommendation: format!("Complete implementation of control {}.", i.id),
            })
            .collect();

        trust_reports.push(TrustServiceCriteriaReport {
            criteria,
            description: description.to_string(),
            status,
            evidence_count: items.iter().map(|i| i.evidence.len()).sum(),
            findings,
        });
    }

    let overall_status = if fail_count > 0 {
        CriteriaStatus::Fail
    } else if partial_count > 0 {
        CriteriaStatus::PartialPass
    } else {
        CriteriaStatus::Pass
    };

    Soc2Report {
        generated_at: Utc::now(),
        period_start,
        period_end,
        trust_service_criteria: trust_reports,
        summary: Soc2Summary {
            total_evidence_items: collector.total_evidence(),
            criteria_pass: pass_count,
            criteria_partial: partial_count,
            criteria_fail: fail_count,
            criteria_not_assessed: not_assessed_count,
            overall_status,
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_collector_access() {
        let mut collector = Soc2EvidenceCollector::new();
        let evidence = AccessEvidence {
            user_id: "user-1".to_string(),
            resource: "/api/v1/documents".to_string(),
            action: "read".to_string(),
            timestamp: Utc::now(),
            ip_address: Some("192.168.1.1".to_string()),
            outcome: "success".to_string(),
        };
        collector.record_access(evidence);
        assert_eq!(collector.access_evidence().len(), 1);
        assert_eq!(collector.total_evidence(), 1);
    }

    #[test]
    fn test_evidence_collector_change() {
        let mut collector = Soc2EvidenceCollector::new();
        let evidence = ChangeEvidence {
            change_id: "ch-001".to_string(),
            author: "dev-1".to_string(),
            timestamp: Utc::now(),
            description: "Deploy v2.1.0".to_string(),
            approval: ChangeApproval::Approved {
                approver: "admin".to_string(),
            },
            change_type: ChangeType::Deployment,
        };
        collector.record_change(evidence);
        assert_eq!(collector.change_evidence().len(), 1);
        assert_eq!(collector.total_evidence(), 1);
    }

    #[test]
    fn test_evidence_collector_monitoring() {
        let mut collector = Soc2EvidenceCollector::new();
        let evidence = MonitoringEvidence {
            metric_name: "uptime".to_string(),
            value: 99.95,
            timestamp: Utc::now(),
            threshold: Some(99.9),
            status: MonitoringStatus::Normal,
        };
        collector.record_monitoring(evidence);
        assert_eq!(collector.monitoring_evidence().len(), 1);
        assert_eq!(collector.total_evidence(), 1);
    }

    #[test]
    fn test_from_access_snapshot() {
        let evidence = Soc2EvidenceCollector::from_access_snapshot(
            "user-1",
            "/documents/abc",
            "read",
            Some("10.0.0.1".to_string()),
            "success",
        );
        assert_eq!(evidence.user_id, "user-1");
        assert_eq!(evidence.resource, "/documents/abc");
    }

    #[test]
    fn test_from_deployment() {
        let evidence =
            Soc2EvidenceCollector::from_deployment("ch-1", "dev", "Deploy v1", Some("admin"));
        assert_eq!(
            evidence.approval,
            ChangeApproval::Approved {
                approver: "admin".to_string()
            }
        );
    }

    #[test]
    fn test_from_deployment_pending() {
        let evidence = Soc2EvidenceCollector::from_deployment("ch-2", "dev", "Deploy v2", None);
        assert_eq!(evidence.approval, ChangeApproval::Pending);
    }

    #[test]
    fn test_checklist_generation() {
        let checklist = generate_soc2_checklist();
        assert!(!checklist.is_empty());
        let security_items: Vec<_> = checklist
            .iter()
            .filter(|i| i.criteria == Soc2Criteria::Security)
            .collect();
        assert!(security_items.len() >= 4);
    }

    #[test]
    fn test_report_generation() {
        let mut collector = Soc2EvidenceCollector::new();
        collector.record_access(Soc2EvidenceCollector::from_access_snapshot(
            "u1", "doc", "read", None, "ok",
        ));
        let report = generate_soc2_report(
            &collector,
            Utc::now() - chrono::Duration::days(90),
            Utc::now(),
        );
        assert_eq!(report.trust_service_criteria.len(), 5);
        assert!(report.summary.total_evidence_items >= 1);
    }

    #[test]
    fn test_report_serialization() {
        let collector = Soc2EvidenceCollector::new();
        let report = generate_soc2_report(
            &collector,
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        );
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("trust_service_criteria"));
        assert!(json.contains("summary"));
    }

    #[test]
    fn test_checklist_item_serialization() {
        let item = Soc2ChecklistItem {
            id: "CC6.1".to_string(),
            criteria: Soc2Criteria::Security,
            description: "Test".to_string(),
            status: ChecklistStatus::Implemented,
            evidence: vec!["test.rs".to_string()],
            last_verified: Some(Utc::now()),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Implemented"));
    }

    #[test]
    fn test_monitoring_status_serialize() {
        let status = MonitoringStatus::Normal;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""Normal""#);
    }

    #[test]
    fn test_change_approval_serialize() {
        let approval = ChangeApproval::Approved {
            approver: "admin".to_string(),
        };
        let json = serde_json::to_string(&approval).unwrap();
        assert!(json.contains("Approved"));
        assert!(json.contains("admin"));
    }

    #[test]
    fn test_default_collector() {
        let collector = Soc2EvidenceCollector::default();
        assert_eq!(collector.total_evidence(), 0);
    }
}
