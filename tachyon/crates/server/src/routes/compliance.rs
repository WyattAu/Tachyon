//! SOC 2 Type II compliance scaffolding — audit trails and evidence collection.
//!
//! **SCAFFOLDING STATUS:** The checklist is a static list of items with hardcoded
//! statuses. The SOC 2 report generation in `soc2.rs` does use real audit events,
//! but the compliance controls themselves are not enforced programmatically.
//! Achieving real SOC 2 Type II compliance requires external auditor engagement
//! and verified control implementation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Soc2Category {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceChecklist {
    pub category: Soc2Category,
    pub items: Vec<ComplianceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceItem {
    pub id: String,
    pub description: String,
    pub status: String,
    pub evidence: Vec<String>,
}

pub fn generate_soc2_checklist() -> Vec<ComplianceChecklist> {
    vec![
        ComplianceChecklist {
            category: Soc2Category::Security,
            items: vec![
                ComplianceItem {
                    id: "SEC-001".to_string(),
                    description: "Encryption at rest (AES-256)".to_string(),
                    status: "implemented".to_string(),
                    evidence: vec!["config.rs: jwt encryption".to_string()],
                },
                ComplianceItem {
                    id: "SEC-002".to_string(),
                    description: "Encryption in transit (TLS 1.2+)".to_string(),
                    status: "implemented".to_string(),
                    evidence: vec!["security_headers.rs: HSTS".to_string()],
                },
                ComplianceItem {
                    id: "SEC-003".to_string(),
                    description: "Multi-factor authentication".to_string(),
                    status: "implemented".to_string(),
                    evidence: vec!["totp.rs".to_string()],
                },
                ComplianceItem {
                    id: "SEC-004".to_string(),
                    description: "Role-based access control".to_string(),
                    status: "implemented".to_string(),
                    evidence: vec!["permissions.rs".to_string()],
                },
                ComplianceItem {
                    id: "SEC-005".to_string(),
                    description: "Audit logging".to_string(),
                    status: "implemented".to_string(),
                    evidence: vec!["audit.rs".to_string()],
                },
            ],
        },
        ComplianceChecklist {
            category: Soc2Category::Availability,
            items: vec![
                ComplianceItem {
                    id: "AVL-001".to_string(),
                    description: "Health check endpoints".to_string(),
                    status: "implemented".to_string(),
                    evidence: vec!["health.rs".to_string()],
                },
                ComplianceItem {
                    id: "AVL-002".to_string(),
                    description: "Horizontal scaling".to_string(),
                    status: "implemented".to_string(),
                    evidence: vec!["redis_relay.rs".to_string()],
                },
            ],
        },
        ComplianceChecklist {
            category: Soc2Category::Confidentiality,
            items: vec![ComplianceItem {
                id: "CNF-001".to_string(),
                description: "Data classification".to_string(),
                status: "planned".to_string(),
                evidence: vec![],
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soc2_checklist_generation() {
        let checklist = generate_soc2_checklist();
        assert!(!checklist.is_empty());
        let security = checklist
            .iter()
            .find(|c| c.category == Soc2Category::Security)
            .unwrap();
        assert!(security.items.len() >= 5);
    }

    #[test]
    fn test_soc2_category_serialization() {
        assert_eq!(
            serde_json::to_string(&Soc2Category::Security).unwrap(),
            r#""Security""#
        );
    }

    #[test]
    fn test_compliance_item() {
        let item = ComplianceItem {
            id: "SEC-001".to_string(),
            description: "Test".to_string(),
            status: "implemented".to_string(),
            evidence: vec!["audit.rs".to_string()],
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("implemented"));
    }
}
