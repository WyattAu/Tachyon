//! Data Loss Prevention engine.
//! Scans content for sensitive data patterns before storage.

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SensitivityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DlpAction {
    Log,
    Warn,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub action: DlpAction,
    pub sensitivity: SensitivityLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpViolation {
    pub rule_id: String,
    pub rule_name: String,
    pub matched_text: String,
    pub action: DlpAction,
    pub sensitivity: SensitivityLevel,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpScanResult {
    pub violations: Vec<DlpViolation>,
    pub max_sensitivity: Option<SensitivityLevel>,
    pub is_blocked: bool,
}

pub struct DlpEngine {
    rules: Vec<(DlpRule, Regex)>,
}

impl Default for DlpEngine {
    fn default() -> Self {
        let mut engine = Self { rules: Vec::new() };
        engine.add_default_rules();
        engine
    }
}

impl DlpEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_default_rules(&mut self) {
        self.add_rule(DlpRule {
            id: "dlp-credit-card".into(),
            name: "Credit Card Number".into(),
            pattern: r"\b4\d{12}(?:\d{3})?\b|\b5[1-5]\d{14}\b".into(),
            action: DlpAction::Warn,
            sensitivity: SensitivityLevel::Restricted,
            description: "Detects Visa/Mastercard card numbers".into(),
        });
        self.add_rule(DlpRule {
            id: "dlp-ssn".into(),
            name: "Social Security Number".into(),
            pattern: r"\b\d{3}-\d{2}-\d{4}\b".into(),
            action: DlpAction::Block,
            sensitivity: SensitivityLevel::Restricted,
            description: "Detects US SSN format".into(),
        });
        self.add_rule(DlpRule {
            id: "dlp-api-key".into(),
            name: "API Key".into(),
            pattern:
                r#"(?i)(api[_-]?key|secret|token|password)\s*[:=]\s*['"]?[a-zA-Z0-9]{20,}['"]?"#
                    .into(),
            action: DlpAction::Warn,
            sensitivity: SensitivityLevel::Confidential,
            description: "Detects potential API keys/secrets".into(),
        });
    }

    fn add_rule(&mut self, rule: DlpRule) {
        if let Ok(re) = Regex::new(&rule.pattern) {
            self.rules.push((rule, re));
        }
    }

    pub fn scan(&self, content: &str) -> DlpScanResult {
        let mut violations = Vec::new();
        let mut max_sensitivity: Option<SensitivityLevel> = None;
        let mut is_blocked = false;

        for (rule, re) in &self.rules {
            for m in re.find_iter(content) {
                if rule.action == DlpAction::Block {
                    is_blocked = true;
                }
                let sens = &rule.sensitivity;
                max_sensitivity = Some(match &max_sensitivity {
                    None => sens.clone(),
                    Some(current) => {
                        if self.sensitivity_rank(sens) > self.sensitivity_rank(current) {
                            sens.clone()
                        } else {
                            current.clone()
                        }
                    }
                });
                violations.push(DlpViolation {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    matched_text: m.as_str().to_string(),
                    action: rule.action.clone(),
                    sensitivity: rule.sensitivity.clone(),
                    position: m.start(),
                });
            }
        }

        DlpScanResult {
            violations,
            max_sensitivity,
            is_blocked,
        }
    }

    fn sensitivity_rank(&self, level: &SensitivityLevel) -> u8 {
        match level {
            SensitivityLevel::Public => 0,
            SensitivityLevel::Internal => 1,
            SensitivityLevel::Confidential => 2,
            SensitivityLevel::Restricted => 3,
        }
    }
}
