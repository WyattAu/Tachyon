# ADR-101: Security Monitoring

## Status

**Status:** Accepted  
**Date:** 2026-02-12  
**Decision Date:** 2026-02-12

## Context

The Tachyon project requires continuous security monitoring to detect vulnerabilities, threats, and security incidents in real-time, ensuring the security posture is maintained throughout the operational lifecycle.

## Problem

How do we implement comprehensive security scanning and threat detection that provides early warning of vulnerabilities and active threats?

## Decision

### Automated Security Monitoring System

The Tachyon project implements an automated security monitoring system with the following components:

1. **Vulnerability Scanning**
   - SAST (Static Application Security Testing) - Semgrep, SonarQube
   - DAST (Dynamic Application Security Testing) - OWASP ZAP
   - SCA (Software Composition Analysis) - Snyk, Cargo Audit
   - Container Scanning - Trivy, Clair
   - Secret Scanning - Gitleaks, Trivy

2. **Threat Detection**
   - Web Application Firewall (WAF) monitoring
   - Rate limiting and anomaly detection
   - Authentication abuse detection
   - Data exfiltration detection
   - Malware and intrusion detection

3. **Vulnerability Management**
   - CVSS-based severity classification
   - SLA-based remediation (7 days for critical)
   - Automated tracking and alerting
   - Remediation workflow management

### Scan Types and Frequencies

| Scan Type | Tool | Frequency | Trigger | Coverage |
|---------|------|-----------|----------|----------|
| SAST | Semgrep | On commit | Push to main/develop | Source code |
| SAST | SonarQube | On PR | PR to main/develop | Source code |
| DAST | OWASP ZAP | Daily | Scheduled | Running application |
| SCA | Snyk | On dependency update | Cargo.toml/package.json | Dependencies |
| SCA | Cargo Audit | On commit | Cargo.toml change | Rust dependencies |
| Container | Trivy | On build | Dockerfile change | Docker images |
| Secrets | Gitleaks | On push | All branches | Repository |

### Severity Classification

**CVSS Score Mapping:**

| CVSS Score | Severity | Response Time | Remediation SLA |
|------------|-----------|---------------|-----------------|
| 0.0 - 3.9 | Low | 30 days | 60 days |
| 4.0 - 6.9 | Medium | 14 days | 30 days |
| 7.0 - 8.9 | High | 7 days | 14 days |
| 9.0 - 10.0 | Critical | 24 hours | 7 days |

### Alerting Strategy

**Alert Classification:**

| Vulnerability Severity | Response Time | Channels |
|---------------------|-----------|----------|
| Critical (CVSS >= 9.0) | P1 | < 5 minutes | PagerDuty, Slack, Email, Phone |
| High (CVSS 7.0-8.9) | P2 | < 15 minutes | Slack, Email, PagerDuty |
| Medium (CVSS 4.0-6.9) | P3 | < 60 minutes | Slack, Email |
| Low (CVSS 0.0-3.9) | P4 | < 4 hours | Slack |
| Active Threat | P1 | < 5 minutes | PagerDuty, Slack, Email, Phone |

**Routing:**

| Alert Type | Primary Channel | Secondary Channels | Escalation |
|-----------|----------------|-------------------|------------|
| Critical Vulnerability | PagerDuty | Slack, Email, Phone | CTO (30 min) |
| High Vulnerability | Slack, Email, PagerDuty | Engineering Manager (60 min) |
| Active Threat | PagerDuty | Slack, Email, Phone | CTO (30 min) |
| Medium Vulnerability | Slack, Email | Engineering Team | - |
| Low Vulnerability | Slack | Engineering Team | - |

### Reporting Strategy

**Security Reports:**

| Report Type | Frequency | Recipients | Purpose |
|-------------|-----------|------------|---------|
| Daily Summary | Daily (08:00 UTC) | Engineering Team | Daily operational status |
| Weekly Report | Weekly (Monday 09:00 UTC) | All Stakeholders | Weekly trends and issues |
| Monthly Trend | Monthly (1st day 09:00 UTC) | Executives | Monthly analysis and KPIs |
| Quarterly Audit | Quarterly (last day of quarter) | Board, Auditors | Quarterly security audit |

**Report Content:**
- Vulnerability trends
- Threat activity
- Security posture assessment
- Remediation progress
- Compliance status
- Recommendations
- Action items

## Consequences

### Positive Consequences

- Early detection of security vulnerabilities
- Real-time threat detection
- Proactive vulnerability remediation
- Reduced risk exposure
- Improved security posture
- Comprehensive security audit trail
- Automated compliance verification
- Data-driven security decisions

### Negative Consequences

- Potential for false positives and alert fatigue
- Increased system complexity
- Security monitoring overhead
- Alert desensitization potential
- Storage and infrastructure costs
- Maintenance requirements for scanning infrastructure
- Performance impact from security scanning

## Alternatives Considered

1. **Manual Security Scanning Only:** Rejected - insufficient frequency and coverage
2. **Periodic Security Audits Only:** Rejected - risk of issues between audits
3. **External Security Service:** Rejected - cost and data sovereignty concerns
4. **Simplified Alerting Only:** Rejected - lacks context and prioritization

## Implementation

### SAST Configuration

```yaml
# Semgrep Configuration
semgrep:
  rules:
    - id: sql-injection
      languages: [rust, javascript]
      patterns:
        - pattern: execute(query)
          message: "Potential SQL injection vulnerability"
          severity: ERROR
        - pattern: $EXEC($QUERY)
          message: "Potential SQL injection using EXEC macro"
          severity: ERROR

    - id: xss-vulnerability
      languages: [rust, javascript]
      patterns:
        - pattern: innerHTML($INPUT)
          message: "Potential XSS vulnerability"
          severity: WARNING
        - pattern: eval($INPUT)
          message: "Potential eval injection"
          severity: ERROR

    - id: hardcoded-secrets
      languages: [rust, javascript]
      patterns:
        - pattern: $KEY = "$SECRET"
          message: "Potential hardcoded secret"
          severity: ERROR
        - pattern: password = "$CREDENTIAL"
          message: "Potential hardcoded password"
          severity: ERROR
```

### DAST Configuration

```yaml
# OWASP ZAP Configuration
zap:
  target: "https://tachyon.example.com"
  scan_policy: "full-scan"
  alert_thresholds:
    high: 0
    medium: 5
    low: 10
    info: 20

  authentication:
    type: "form-based"
    login_url: "/auth/login"
    username_field: "username"
    password_field: "password"

  scan_schedule:
    enabled: true
    frequency: "daily"
    time: "02:00 UTC"

  reporting:
    format: ["json", "html", "xml"]
    output_dir: "/var/reports/zap/"
```

### SCA Configuration

```yaml
# Snyk Configuration
snyk:
  organization: "tachyon-io"
  project: "tachyon"
  severity_threshold:
    low: false
    medium: true
    high: true
    critical: true

  vulnerability_policy:
    - type: "cvss-score"
      threshold: ">= 7.0"
      action: "fail"

  dependency_update:
    auto_patch: false
    notify: true
    channels:
      - slack
      - email

  scan_schedule:
    - trigger: "dependency-update"
    - trigger: "scheduled"
      frequency: "daily"
      time: "03:00 UTC"
```

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Vulnerability Detection Time | < 24 hours | Time from disclosure to detection |
| Critical Vulnerability Remediation | 95% within SLA | % remediated on time |
| Mean Time to Remediation (MTTR) | < 7 days | Average time to fix |
| False Positive Rate | < 5% | False alerts / Total alerts |
| True Positive Rate | > 95% | Real threats / Total threats |
| Security Test Coverage | > 90% | Code covered by security tests |

## Related Decisions

- [ADR-097](adr-097-monitoring-strategy.md) - Continuous Monitoring Strategy
- [`.adrs/ - Security Monitoring Specification
- [`.adrs/ - Threat Model
- [`.adrs/ - Security Test Plan

## References

- OWASP Top 10: https://owasp.org/www-project-top-ten
- CWE Top 25: https://cwe.mitre.org/Top25
- NVD: https://nvd.nist.gov
- RustSec: https://rustsec.org
- Snyk: https://snyk.io

---

**Document Status:** COMPLETE
**Owner:** Monitoring Engineer
**Reviewers:** TBD
**Approved By:** TBD
