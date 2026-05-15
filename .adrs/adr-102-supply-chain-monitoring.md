# ADR-102: Supply Chain Monitoring

## Status

**Status:** Accepted  
**Date:** 2026-02-12  
**Decision Date:** 2026-02-12

## Context

The Tachyon project requires continuous supply chain monitoring to track dependency vulnerabilities, license compliance, and supply chain threats throughout the operational lifecycle.

## Problem

How do we implement automated dependency vulnerability monitoring, SBOM verification, and license compliance checking to ensure supply chain security?

## Decision

### Automated Supply Chain Monitoring System

The Tachyon project implements an automated supply chain monitoring system with the following components:

1. **Dependency Vulnerability Monitoring**
   - Multi-source vulnerability database aggregation (NVD, GitHub, RustSec, Snyk)
   - CVSS-based severity classification
   - Dependency health assessment
   - Automated vulnerability detection and alerting

2. **SBOM (Software Bill of Materials) Verification**
   - Automated SBOM generation on build
   - Completeness verification
   - Accuracy verification
   - Integrity verification
   - License compliance checking

3. **License Compliance Monitoring**
   - Allowed licenses defined
   - License scanning on dependency update
   - License violation detection and alerting

4. **Supply Chain Threat Detection**
   - Dependency confusion detection (typosquatting)
   - Malicious dependency detection
   - Compromised repository detection
   - Abandoned dependency detection

### Monitoring Sources

**Vulnerability Databases:**

| Source | URL | Frequency | Coverage | Alert Threshold |
|--------|-----|-----------|----------|-----------------|
| NVD (National Vulnerability Database) | https://nvd.nist.gov | Daily | CVSS >= 7.0 |
| GitHub Advisory Database | https://github.com/advisories | Daily | CVSS >= 7.0 |
| RustSec Advisory Database | https://rustsec.org/advisories | Daily | CVSS >= 7.0 |
| Snyk Advisory Database | https://snyk.io | Daily | CVSS >= 7.0 |

**Ecosystem-Specific Sources:**

| Ecosystem | Sources | Frequency | Coverage |
|-----------|---------|----------|----------|
| Rust | Cargo Advisory Database, RustSec, NVD | Daily | 100% |
| npm | npm audit, Snyk, NVD | Daily | 100% |
| System | OS package managers | Weekly | 100% |

### CVSS Severity Classification

| CVSS Score | Severity | Response Time | Remediation SLA |
|------------|-----------|---------------|-----------------|
| 0.0 - 3.9 | Low | 30 days | 60 days |
| 4.0 - 6.9 | Medium | 14 days | 30 days |
| 7.0 - 8.9 | High | 7 days | 14 days |
| 9.0 - 10.0 | Critical | 24 hours | 7 days |

### SBOM Generation

**Generation Triggers:**

| Trigger | Description | Format |
|---------|-------------|--------|
| On build | Generate SBOM for every build | SPDX, CycloneDX |
| On dependency change | Update SBOM when dependencies change | SPDX, CycloneDX |
| On release | Generate release SBOM | SPDX, CycloneDX |
| Scheduled | Daily verification | SPDX, CycloneDX |

**SBOM Format:** SPDX 2.3, CycloneDX 1.4

### License Compliance

**Allowed Licenses:**

| License | Type | Approval Required |
|----------|----------|-----------------|
| MIT | Permissive | Yes | No |
| Apache-2.0 | Permissive | Yes | No |
| BSD-3-Clause | Permissive | Yes | No |
| BSD-2-Clause | Permissive | Yes | No |
| 0BSD | Permissive | Yes | No |
| ISC | Permissive | Yes | No |
| Unlicense | Permissive | Yes | No |

### Alerting Strategy

**Alert Classification:**

| Issue Type | Severity | Response Time | Channels |
|-----------|-----------|---------------|----------|
| Critical Vulnerability (CVSS >= 9.0) | P1 | < 5 minutes | PagerDuty, Slack, Email |
| High Vulnerability (CVSS 7.0-8.9) | P2 | < 15 minutes | Slack, Email, PagerDuty |
| Medium Vulnerability (CVSS 4.0-6.9) | P3 | < 60 minutes | Slack, Email |
| License Violation | P3 | < 60 minutes | Slack, Email |
| Abandoned Dependency | P3 | < 60 minutes | Slack, Email |
| SBOM Inaccuracy | P4 | < 4 hours | Slack |
| Supply Chain Threat | P1 | < 5 minutes | PagerDuty, Slack, Email, Phone |

**Routing:**

| Alert Type | Primary Channel | Secondary Channels | Escalation |
|-----------|----------------|-------------------|------------|
| Critical Vulnerability | PagerDuty | Slack, Email, Phone | CTO (30 min) |
| High Vulnerability | Slack, Email, PagerDuty | Engineering Manager (60 min) |
| Supply Chain Threat | PagerDuty | Slack, Email, Phone | CTO (30 min) |
| License Violation | Slack, Email | Engineering Team | - |
| Abandoned Dependency | Slack, Email | Engineering Team | - |

### Reporting Strategy

**Supply Chain Reports:**

| Report Type | Frequency | Recipients | Purpose |
|-------------|-----------|------------|---------|
| Daily Summary | Daily (08:00 UTC) | Engineering Team | Daily operational status |
| Weekly Report | Weekly (Monday 09:00 UTC) | All Stakeholders | Weekly trends and issues |
| Monthly Trend | Monthly (1st day 09:00 UTC) | Executives | Monthly analysis and KPIs |
| Quarterly Audit | Quarterly (last day of quarter) | Board, Auditors | Quarterly audit |

**Report Content:**
- Vulnerability trends
- Dependency health metrics
- License compliance status
- Supply chain threat activity
- SBOM accuracy and integrity
- Remediation progress
- Recommendations
- Action items

## Consequences

### Positive Consequences

- Early detection of dependency vulnerabilities
- Proactive vulnerability remediation
- Reduced risk from supply chain attacks
- Continuous license compliance verification
- Improved supply chain security
- Comprehensive dependency audit trail
- Data-driven dependency decisions
- Automated SBOM verification

### Negative Consequences

- Potential for false positives
- Increased system complexity
- Alert fatigue potential
- Maintenance overhead
- Storage and infrastructure costs
- Performance impact from scanning

## Alternatives Considered

1. **Manual Dependency Review Only:** Rejected - insufficient frequency and coverage
2. **Periodic SBOM Audits Only:** Rejected - risk of issues between audits
3. **External Supply Chain Service:** Rejected - cost and data sovereignty concerns
4. **Simplified License Checking Only:** Rejected - lacks automation and tracking

## Implementation

### Dependency Vulnerability Monitoring

**Vulnerability Database Aggregation:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityDatabase {
    pub sources: Vec<VulnerabilitySource>,
    pub vulnerabilities: Vec<VulnerabilityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilitySource {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityEntry {
    pub cve_id: String,
    pub cvss_score: f64,
    pub affected_packages: Vec<PackageReference>,
    pub published_at: DateTime<Utc>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageReference {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
}

// Vulnerability Aggregator
pub async fn aggregate_vulnerabilities(
    sources: Vec<VulnerabilitySource>,
) -> Result<VulnerabilityDatabase, Error> {
    let mut db = VulnerabilityDatabase {
        sources: vec![],
        vulnerabilities: vec![],
    };

    for source in sources {
        if source.enabled {
            let vulns = source.fetch_vulnerabilities().await?;
            db.vulnerabilities.extend(vulns);
        }
    }

    Ok(db)
}
```

**CVSS Severity Classification:**

```rust
pub fn classify_severity(cvss_score: f64) -> Severity {
    if cvss_score >= 9.0 {
        Severity::Critical
    } else if cvss_score >= 7.0 {
        Severity::High
    } else if cvss_score >= 4.0 {
        Severity::Medium
    } else if cvss_score >= 0.0 {
        Severity::Low
    } else {
        Severity::Info
    }
}
```

### SBOM Generation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SBOM {
    pub id: String,
    pub document_name: String,
    pub document_namespace: String,
    pub data_license: String,
    spdx_version: String,
    created_at: DateTime<Utc>,
    packages: Vec<SBOMPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SBOMPackage {
    pub spdx_id: String,
    pub name: String,
    pub version: String,
    pub download_location: String,
    license_concluded: String,
    license_declared: String,
    checksum: String,
}

// SBOM Generator
pub async fn generate_sbom(
    project_name: &str,
    cargo_lock_path: &str,
    package_json_path: &str,
    output_path: &str,
) -> Result<SBOM, Error> {
    // Parse Cargo.lock
    let cargo_lock = parse_cargo_lock(cargo_lock_path)?;
    
    // Parse package.json for npm
    let npm_packages = if Path::new(package_json_path).exists() {
        parse_package_json(package_json_path)?
    } else {
        vec![]
    };

    let mut packages = Vec::new();

    for dependency in cargo_lock.dependencies {
        let pkg = SBOMPackage {
            spdx_id: format!("SPDXRef-{}", dependency.name.replace('_', "-")),
            name: dependency.name.clone(),
            version: dependency.version.to_string(),
            download_location: format!(
                "https://crates.io/api/v1/crates/{}",
                dependency.name
            ),
            license_concluded: infer_license(&dependency.license),
            license_declared: dependency.license.clone(),
            checksum: calculate_checksum(&dependency),
        };
        packages.push(pkg);
    }

    let sbom = SBOM {
        id: format!("{}-{}", project_name, Utc::now().timestamp()),
        document_name: project_name.to_string(),
        document_namespace: format!("https://example.com/sbom/tachyon-{}"),
        data_license: "CC0-1.0".to_string(),
        spdx_version: "SPDX-2.3".to_string(),
        created_at: Utc::now(),
        packages,
    };

    write_sbom(&sbom, output_path).await?;

    Ok(sbom)
}
```

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Vulnerability Detection Time | < 24 hours | Time from disclosure to detection |
| Critical Vulnerability Remediation | 95% within SLA | % remediated on time |
| Mean Time to Remediation (MTTR) | < 7 days | Average time to fix |
| SBOM Accuracy | 100% | SBOM matches actual dependencies |
| License Compliance | 100% | All licenses are allowed |
| Supply Chain Risk Score | < 10 | Aggregate risk score |

## Related Decisions

- [ADR-097](adr-097-monitoring-strategy.md) - Continuous Monitoring Strategy
- [`.specs/11_continuous_monitoring/supply_chain_monitoring.md`](../.specs/11_continuous_monitoring/supply_chain_monitoring.md) - Supply Chain Monitoring Specification
- [`.specs/01_5_supply_chain/sbom.spdx`](../.specs/01_5_supply_chain/sbom.spdx) - SBOM
- [`.specs/01_5_supply_chain/vulnerability_report.md`](../.specs/01_5_supply_chain/vulnerability_report.md) - Vulnerability Report

## References

- NVD: https://nvd.nist.gov
- GitHub Advisory: https://github.com/advisories
- RustSec: https://rustsec.org
- Snyk: https://snyk.io
- SPDX Specification: https://spdx.dev

---

**Document Status:** COMPLETE
**Owner:** Monitoring Engineer
**Reviewers:** TBD
**Approved By:** TBD
