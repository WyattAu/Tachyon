# ADR-081: Compliance Monitoring

**Status:** Accepted
**Date:** 2026-02-12
**Decision Type:** Technical
**Context:** Phase 8.5 - Supply Chain Monitoring

---

## Context and Problem Statement

### Context
The Tachyon project must comply with multiple security and privacy standards (NIST SP 800-53, ISO/IEC 27001:2022, GDPR, CCPA, OWASP Top 10). The compliance matrix ([`.adrs/ identifies 173 controls with 0% current implementation rate. The license compliance report ([`.adrs/ provides baseline license analysis.

### Problem Statement
We lack automated compliance monitoring that provides:
1. Continuous monitoring of compliance requirements
2. Automated license checking for all dependencies
3. Real-time compliance status tracking
4. Automated violation detection and alerting
5. Compliance audit trail generation
6. Compliance reporting for stakeholders

Current challenges:
- Manual compliance verification
- No automated license policy enforcement
- Difficult to track compliance posture
- No real-time compliance dashboard
- Risk of non-compliant dependencies entering codebase

---

## Decision Drivers

### Compliance Requirements
From [`.adrs/
- **NIST-AU-02:** Audit Events
- **NIST-SI-07:** Software and Information Integrity
- **NIST-SI-10:** Information Input Validation
- **NIST-SA-12:** Supply Chain Protection

### Business Requirements
- **Compliance Rate:** Target 100% for all critical controls
- **Audit Trail:** All compliance checks must be logged
- **Automated Enforcement:** Policies must be enforced automatically
- **Real-Time Visibility:** Compliance status must be visible
- **False Positive Rate:** <2% for compliance violations

### Legal Requirements
- **License Compliance:** All dependencies must use allowed licenses
- **Attribution:** Copyright notices must be preserved
- **Policy Documentation:** License policies must be documented

---

## Considered Alternatives

### Alternative 1: SaaS Compliance Platforms (Snyk, WhiteSource, Sonatype)
**Description:** Use commercial SaaS platforms for compliance monitoring

**Pros:**
- Comprehensive compliance coverage
- Rich UI and reporting
- Automated policy enforcement
- Integrated vulnerability scanning
- Continuous monitoring and alerting
- Ready-made compliance reports

**Cons:**
- Recurring licensing costs ($15K-$50K/year)
- Limited control over compliance rules
- Vendor lock-in
- Potential data residency concerns
- Learning curve for team

**Rejection:** Cost-prohibitive for current scale, loss of control

### Alternative 2: Manual Compliance Only
**Description:** Rely on manual compliance audits and reviews

**Pros:**
- Zero additional cost
- Full control over compliance process
- Deep understanding of requirements
- Can use any tools

**Cons:**
- High human error and omission risk
- Slow detection of violations
- Inconsistent coverage
- Difficult to maintain audit trail
- Does not meet continuous monitoring requirements

**Rejection:** Insufficient for regulatory compliance

### Alternative 3: CI/CD Only Compliance
**Description:** Rely solely on CI/CD pipeline for compliance checks

**Pros:**
- Zero additional cost
- Native integration with development workflow
- Automated enforcement at merge time
- Minimal configuration overhead

**Cons:**
- Limited to CI/CD context
- No real-time compliance dashboard
- Limited to build-time checks
- No ongoing compliance monitoring
- Difficult to track compliance posture between builds

**Rejection:** Insufficient for continuous compliance requirements

### Alternative 4: Multi-Tool Open-Source Compliance (Chosen)
**Description:** Implement comprehensive compliance monitoring using open-source tools

**Pros:**
- Zero licensing cost for open-source tools
- Full control over compliance policies
- Real-time monitoring and dashboard
- Automated policy enforcement
- Native CI/CD integration
- Comprehensive audit trail
- Support for multiple compliance frameworks

**Cons:**
- Requires tool installation and configuration
- Multiple tools to maintain and update
- Complex workflow orchestration
- Requires script development

**Acceptance:** Best balance of automation, control, and cost

---

## Decision

### Chosen Approach: Multi-Tool Open-Source Compliance

We will implement compliance monitoring using:

**License Compliance:**
- **cargo-deny:** Automated license checking for Rust dependencies
- **license-checker:** NPM license verification
- **Custom policy engine:** Enforce allowed/prohibited license lists

**SBOM Compliance:**
- **Syft:** SBOM completeness verification
- **SPDX tools:** SBOM format validation
- **Integrity checks:** Hash verification and signature validation

**Compliance Dashboard:**
- **Real-time status:** Live compliance posture display
- **Audit logging:** All compliance checks logged
- **Reporting:** Scheduled compliance reports
- **KPI tracking:** Compliance rate and trend analysis

### Compliance Monitoring Architecture

```
┌───────────────────────────────────────────────────────────────────────────┐
│                  Compliance Monitoring System                         │
├─────────────────────────────────────────────────────────────────────────────┤
│  Data Collection Layer                                              │
│  - License Scanning (Cargo, NPM)                                  │
│  - SBOM Verification (Completeness, Integrity)                      │
│  - Policy Checks (Allowed/Prohibited Lists)                          │
│  - Dependency Audit Trail                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  Policy Enforcement Layer                                              │
│  - License Policy Engine                                             │
│  - SBOM Policy Engine                                              │
│  - Compliance Rules (NIST, ISO, GDPR, CCPA)                      │
│  - Automated Blocking (CI/CD)                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  Monitoring Layer                                                     │
│  - Real-Time Dashboard                                              │
│  - Compliance Status Tracking                                        │
│  - Alert Generation (Violations)                                   │
│  - Reporting (Daily, Weekly, Monthly)                              │
│  - KPI Metrics (Compliance Rate, Audit Trail)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  Reporting Layer                                                      │
│  - Compliance Reports (NIST, ISO, GDPR, CCPA)                      │
│  - Audit Logs (All Compliance Events)                                  │
│  - Stakeholder Notifications                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### License Compliance Engine

**Allowed Licenses:**
```yaml
# config/allowed-licenses.yml
allowed_licenses:
  rust:
    - MIT
    - Apache-2.0
    - BSD-2-Clause
    - BSD-3-Clause
    - ISC
    - Unicode-DFS-2016
    - 0BSD
    
  npm:
    - MIT
    - Apache-2.0
    - BSD-2-Clause
    - BSD-3-Clause
    - ISC
    - 0BSD
    
  prohibited:
    - GPL-2.0
    - GPL-3.0
    - AGPL-3.0
    - LGPL-2.1
    - LGPL-3.0
```

**Policy Enforcement:**
```bash
#!/bin/bash
# enforce-license-policy.sh

check_license() {
    local package=$1
    local license=$2
    local allowed=$3
    
    if [[ ! " ${allowed[*]} " =~ " ${license} " ]]; then
        echo "VIOLATION: License '${license}' not allowed for '${package}'"
        return 1
    fi
    
    return 0
}

# Check all Rust dependencies
cargo deny check licenses --deny ${CARGO_DENY_CONFIG}

# Check all NPM dependencies
cd tachyon/web
license-checker --production --failOn "GPL;AGPL"
```

### SBOM Compliance

**Completeness Verification:**
```bash
#!/bin/bash
# verify-sbom-completeness.sh

SBOM_FILE="${1:-.spdx/tachyon-server.spdx.json}"

# Check for required fields
required_fields=("spdxId" "name" "versionInfo" "licenseConcluded" "downloadLocation")

for field in "${required_fields[@]}"; do
    if ! jq -e ".packages[].${field}" "${SBOM_FILE}" > /dev/null; then
        echo "ERROR: Required field '${field}' missing from SBOM"
        exit 1
    fi

echo "SBOM completeness verification passed"
```

**Integrity Verification:**
```bash
#!/bin/bash
# verify-sbom-integrity.sh

SBOM_FILE="${1:-.spdx/tachyon-server.spdx.json}"
EXPECTED_HASH="${2}"

# Calculate current hash
CALCULATED_HASH=$(sha256sum "${SBOM_FILE}" | cut -d ' ' -f1)

# Compare hashes
if [ "${CALCULATED_HASH}" != "${EXPECTED_HASH}" ]; then
    echo "VIOLATION: SBOM hash mismatch"
    echo "Expected: ${EXPECTED_HASH}"
    echo "Actual: ${CALCULATED_HASH}"
    exit 1
fi

echo "SBOM integrity verification passed"
```

### Compliance Dashboard

**Dashboard Metrics:**
1. **Overall Compliance Rate:** Percentage of passed compliance checks
2. **License Compliance:** License policy adherence
3. **SBOM Compliance:** SBOM completeness and integrity
4. **Violation Trend:** Weekly/monthly violation count
5. **Open Violations:** Currently unresolved compliance issues
6. **Audit Trail:** Total compliance checks logged

**Dashboard API:**
```python
#!/usr/bin/env python3
"""
Compliance Dashboard API
"""

from flask import Flask, jsonify
from datetime import datetime, timedelta
import json

app = Flask(__name__)

@app.route('/api/compliance/status')
def get_compliance_status():
    """Get current compliance status"""
    return jsonify({
        'overall_compliance_rate': calculate_compliance_rate(),
        'license_compliance': get_license_compliance(),
        'sbom_compliance': get_sbom_compliance(),
        'open_violations': get_open_violations(),
        'violation_trend': get_violation_trend(),
        'last_updated': datetime.now().isoformat()
    })

@app.route('/api/compliance/audit')
def get_audit_trail(days=30):
    """Get audit trail for specified days"""
    return jsonify(get_audit_log(days=days))

def calculate_compliance_rate():
    """Calculate overall compliance percentage"""
    total = get_total_checks()
    passed = get_passed_checks()
    return (passed / total * 100) if total > 0 else 100

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

---

## Compliance Reporting

### NIST SP 800-53 Report
**Frequency:** Quarterly
**Content:**
- Control implementation status
- Compliance rate by control family
- Open violations and remediation status
- Audit trail summary
- Recommendations for improvement

### ISO/IEC 27001:2022 Report
**Frequency:** Semi-Annual
**Content:**
- ISMS implementation status
- Compliance with Annex A controls
- Risk assessment results
- Internal audit results
- Management review findings

### GDPR Report
**Frequency:** Annual
**Content:**
- Data processing activities
- Consent management
- Subject rights fulfillment
- Data breach notifications
- DPO contact information

---

## Consequences

### Positive Consequences
1. **Regulatory Compliance:** Automated monitoring ensures NIST, ISO, and GDPR compliance
2. **License Enforcement:** Automated policy enforcement prevents non-compliant dependencies
3. **Real-Time Visibility:** Dashboard provides immediate compliance status
4. **Audit Trail:** Comprehensive logging for compliance and forensics
5. **Risk Mitigation:** Proactive violation detection reduces compliance risk
6. **Cost Efficiency:** Open-source tools avoid SaaS licensing costs
7. **Stakeholder Confidence:** Regular reporting builds trust with stakeholders

### Negative Consequences
1. **Maintenance Overhead:** Ongoing maintenance of compliance tools
2. **Complexity:** Multiple compliance frameworks require management
3. **False Positives:** Automated checks may flag issues requiring manual review
4. **Initial Setup:** ~3 weeks for tool integration and configuration
5. **Policy Evolution:** Updating policies requires careful consideration

---

## Compliance and Standards Alignment

### NIST SP 800-53 Controls
- **AU-02:** Audit Events - Addressed by comprehensive logging
- **AU-06:** Audit Review, Analysis, and Reporting - Addressed by scheduled reports
- **SI-07:** Software and Information Integrity - Addressed by license enforcement
- **SA-12:** Supply Chain Protection - Enhanced by SBOM verification

### ISO/IEC 27001:2022 Controls
- **5.19-5.21:** Supplier Relationships and ICT Supply Chain Security
- **8.8.9:** Logging and Monitoring
- **8.10:** Monitoring and Measurement
- **8.24:** Cryptographic Controls

### Executive Order 14028 Requirements
- **SBOM Generation:** Comprehensive SBOM compliance monitoring
- **Vulnerability Disclosure:** Integrated with compliance monitoring
- **Automated Testing:** CI/CD integration for compliance checks

### GDPR Requirements
- **Article 30:** Record of processing activities
- **Article 32:** Security of processing
- **Article 33:** Data breach notification

---

## Related Decisions

- **ADR-077:** Supply Chain Monitoring ([`.adrs/adr-077-supply-chain-monitoring.md`](.adrs/adr-077-supply-chain-monitoring.md))
- **ADR-054:** SBOM Automation ([`.adrs/adr-054-sbom-automation.md`](.adrs/adr-054-sbom-automation.md))
- **ADR-079:** Dependency Updates ([`.adrs/adr-079-dependency-updates.md`](.adrs/adr-079-dependency-updates.md))

---

## References

**Internal Documents:**
- [`.adrs/ - Monitoring Strategy
- [`.adrs/ - Alerting Rules
- [`.adrs/ - License Compliance
- [`.adrs/ - Compliance Matrix
- [`.adrs/ - CI/CD Pipeline

**External Tools:**
- cargo-deny: https://github.com/EmbarkStudios/cargo-deny
- license-checker: https://github.com/davglock/license-checker
- syft: https://github.com/anchore/syft

**External Standards:**
- NIST SP 800-53: Security and Privacy Controls
- ISO/IEC 27001:2022: Information Security Management
- Executive Order 14028: Improving Software Supply Chain Security
- GDPR: https://gdpr-info.eu
- SPDX 2.3 Specification: https://spdx.dev

---

## Approval

**Approved By:** Security Team Lead
**Approval Date:** 2026-02-12
**Reviewers:** Security Team, Legal Team, DevOps Team
**Implementation Status:** Approved for Implementation
