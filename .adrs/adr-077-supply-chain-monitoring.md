# ADR-077: Supply Chain Monitoring

**Status:** Accepted
**Date:** 2026-02-12
**Decision Type:** Architectural
**Context:** Phase 8.5 - Supply Chain Monitoring

---

## Context and Problem Statement

### Context
The Tachyon project relies on a complex supply chain with multiple dependencies across Rust (Cargo), Node.js (npm), and container images (Docker). The software supply chain represents a significant attack surface and compliance risk. According to the threat model ([`.adrs/ supply chain attacks (SC-SUP-001 through SC-SUP-004) are identified as high-severity threats.

### Problem Statement
We lack a comprehensive, automated supply chain monitoring system that provides:
1. Real-time visibility into dependency health and vulnerability status
2. Automated detection of supply chain attacks
3. Continuous compliance monitoring
4. Timely alerting for security incidents
5. Traceability from dependency to deployment

Current state:
- Manual vulnerability scanning is required ([`.adrs/
- License compliance is documented but not automated ([`.adrs/
- SBOM generation is defined ([`.adrs/ but lacks continuous monitoring

---

## Decision Drivers

### Requirements
From [`.adrs/
- **NIST-SI-07:** Software and Information Integrity
- **NIST-SA-12:** Supply Chain Protection
- **NIST-AU-02:** Audit Events
- **ISO-5.19/5.20/5.21:** Supplier Relationships, Agreements, ICT Supply Chain Security

### Security Requirements
From [`.adrs/
- **SC-SUP-001:** Malicious dependency injection detection
- **SC-SUP-002:** Vulnerability disclosure monitoring
- **SC-SUP-003:** License compliance enforcement

### Business Constraints
- **Operational Cost:** Minimal additional infrastructure
- **Maintenance Burden:** Automated where possible
- **False Positive Rate:** <5% to prevent alert fatigue
- **Response Time:** <24 hours for Critical vulnerabilities

---

## Considered Alternatives

### Alternative 1: Third-Party SaaS Solutions
**Description:** Use SaaS platforms like Snyk, Dependabot Pro, or WhiteSource

**Pros:**
- Fast implementation
- Managed updates
- Rich UI and reporting
- Integrated with many tools

**Cons:**
- Recurring cost ($10K-$50K/year)
- Limited control over configuration
- Dependency on external service availability
- Potential data residency concerns

**Rejection:** Too expensive for current scale

### Alternative 2: Manual Monitoring
**Description:** Continue with manual ad-hoc scanning and manual alerting

**Pros:**
- Zero additional cost
- Full control over process

**Cons:**
- High human error risk
- Slow response times
- Inconsistent coverage
- Difficult to maintain audit trail

**Rejection:** Does not meet security requirements (NIST-SI-07 requires continuous monitoring)

### Alternative 3: Minimal Automation Only
**Description:** Automate only critical vulnerability scanning, manual process for everything else

**Pros:**
- Reduced alert fatigue
- Focused on high-priority issues
- Lower maintenance overhead

**Cons:**
- Misses compliance violations
- Misses supply chain attack indicators
- Does not meet NIST-SA-12 requirements
- Limited visibility into supply chain health

**Rejection:** Insufficient for comprehensive supply chain security

### Alternative 4: Full In-House Automation (Chosen)
**Description:** Implement comprehensive in-house monitoring using open-source tools and GitHub Actions

**Pros:**
- Zero licensing cost
- Full control over configuration
- Complete visibility into supply chain
- Automated compliance enforcement
- Integration with existing CI/CD
- Meets all NIST and ISO requirements
- No dependency on external services
- Scalable with project growth

**Cons:**
- Higher initial implementation effort
- Requires maintenance of automation scripts
- Need to manage multiple tool integrations

**Acceptance:** Best balance of security, compliance, cost, and control

---

## Decision

### Chosen Approach: Full In-House Automation

We will implement a comprehensive, automated supply chain monitoring system using:
1. **GitHub Actions workflows** for scheduled and on-demand scanning
2. **Open-source tools**: cargo-audit, cargo-deny, npm audit, Trivy, Grype
3. **Custom alerting scripts** for rule-based alert generation
4. **Multi-channel notifications**: Slack, PagerDuty, Email, GitHub Issues

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Supply Chain Monitoring System                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  Monitoring Layer                                                  │
│  - Vulnerability Scanning (Rust, NPM, Container)                │
│  - License Compliance Checking                                       │
│  - SBOM Integrity Verification                                        │
│  - Dependency Health Monitoring                                      │
│  - Supply Chain Attack Detection                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  Alerting Layer                                                    │
│  - Rule-Based Alert Generation                                         │
│  - Severity-Based Routing                                            │
│  - Multi-Channel Notification (Slack, PagerDuty, Email, GitHub)   │
│  - Escalation Management                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  Reporting Layer                                                    │
│  - Real-Time Dashboard                                              │
│  - Scheduled Reports (Daily, Weekly, Monthly)                           │
│  - Audit Trail                                                     │
│  - Metrics and KPIs                                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 1: Infrastructure Setup (Week 1)
1. Create GitHub Actions workflow for scheduled scanning
2. Configure Slack webhook and PagerDuty integration
3. Set up monitoring dashboard
4. Configure repository secrets

### Phase 2: Tool Integration (Week 2)
1. Integrate cargo-audit for Rust vulnerability scanning
2. Integrate cargo-deny for license compliance
3. Integrate npm audit for NPM vulnerability scanning
4. Integrate Trivy for container vulnerability scanning
5. Create custom alerting scripts

### Phase 3: Alert Configuration (Week 3)
1. Define alert rules ([`.adrs/
2. Configure severity-based routing
3. Set up escalation paths
4. Test alert delivery to all channels

### Phase 4: Reporting and Dashboard (Week 4)
1. Build monitoring dashboard
2. Create scheduled report generation
3. Configure metrics collection
4. Set up KPI tracking

---

## Consequences

### Positive Consequences
1. **Improved Security Posture:** Real-time vulnerability detection reduces exposure window
2. **Compliance Assurance:** Automated checks ensure NIST and ISO compliance
3. **Reduced Response Time:** Automated alerting enables <24 hour response for Critical vulnerabilities
4. **Cost Efficiency:** Open-source tools avoid SaaS licensing costs
5. **Audit Trail:** Comprehensive logging meets NIST-AU-02 requirements
6. **Scalability:** System grows with project without additional licensing costs

### Negative Consequences
1. **Initial Implementation Effort:** ~4 weeks of development time
2. **Maintenance Overhead:** Ongoing maintenance of automation scripts and configurations
3. **Alert Fatigue Risk:** Without proper tuning, may generate excessive alerts
4. **False Positives:** Automated systems may generate false positives requiring manual review

---

## Compliance and Standards Alignment

### NIST SP 800-53 Controls
- **SI-07:** Software and Information Integrity - Addressed by continuous vulnerability monitoring
- **SA-12:** Supply Chain Protection - Addressed by comprehensive monitoring
- **AU-02:** Audit Events - Addressed by alert logging and incident tracking

### NIST SP 800-161 Requirements
- **Continuous Monitoring:** System provides ongoing visibility into supply chain
- **Risk Assessment:** Automated detection and scoring of supply chain risks
- **Supplier Monitoring:** Dependency health and freshness tracking

### ISO/IEC 27001:2022 Controls
- **5.19-5.21:** Supplier relationships and ICT supply chain security
- **8.8-8.9:** Evidence collection and monitoring

### Executive Order 14028 Requirements
- **SBOM Generation:** Automated for all builds
- **Vulnerability Disclosure:** Automated scanning and reporting
- **Automated Testing:** Integrated into CI/CD pipeline

---

## Related Decisions

- **ADR-054:** SBOM Automation ([`.adrs/adr-054-sbom-automation.md`](.adrs/adr-054-sbom-automation.md))
- **ADR-055:** Performance Regression ([`.adrs/adr-055-performance-regression.md`](.adrs/adr-055-performance-regression.md))
- **ADR-056:** Formal Verification ([`.adrs/adr-056-formal-verification.md`](.adrs/adr-056-formal-verification.md))
- **ADR-057:** Quality Gates ([`.adrs/adr-057-quality-gates.md`](.adrs/adr-057-quality-gates.md))

---

## References

**Internal Documents:**
- [`.adrs/ - Monitoring Strategy
- [`.adrs/ - Alerting Rules
- [`.adrs/ - Threat Model
- [`.adrs/ - Compliance Matrix
- [`.adrs/ - CI/CD Configuration

**External Standards:**
- NIST SP 800-53: Security and Privacy Controls
- NIST SP 800-161: Supply Chain Risk Management
- Executive Order 14028: Improving Software Supply Chain Security
- ISO/IEC 27001:2022: Information Security Management

---

## Approval

**Approved By:** Security Team Lead
**Approval Date:** 2026-02-12
**Reviewers:** Security Team, DevOps Team, Legal Team
**Implementation Status:** Approved for Implementation
