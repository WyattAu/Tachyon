# ADR-098: Standards Updates Monitoring

## Status

**Status:** Accepted  
**Date:** 2026-02-12  
**Decision Date:** 2026-02-12

## Context

The Tachyon project must continuously monitor updates to applicable standards and regulations (IEEE, ISO, NIST, OWASP) to maintain compliance and adapt to changes that may impact the project.

## Problem

How do we establish an automated, continuous monitoring system for standards and regulatory updates that provides timely detection and impact analysis of changes?

## Decision

### Automated Standards Monitoring System

The Tachyon project implements an automated standards monitoring system with the following components:

1. **Standards Scrapers**
   - IEEE Standards Scraper
   - ISO/IEC Standards Scraper
   - NIST Publications Scraper
   - OWASP Standards Scraper
   - MITRE (CWE/CVE) Scraper

2. **Change Detection Engine**
   - Version comparison and change detection
   - Change classification (Minor, Major, Critical)
   - Impact assessment (Low, Medium, High, Critical)

3. **Alerting System**
   - Severity-based alerts (P2-P4)
   - Routing to appropriate channels
   - Escalation as needed

4. **Impact Analysis**
   - Gap analysis for each standard
   - Remediation planning
   - Timeline tracking

### Monitored Standards

**Technical Standards:**

| Standard | Organization | Current Version | Monitoring Frequency | Last Review |
|----------|--------------|-----------------|-----------------|-------------|
| IEEE 1016-2009 | IEEE | 2009 | Daily | 2026-02-12 |
| IEEE 730 | IEEE | 2014 | Daily | 2026-02-12 |
| IEEE 829 | IEEE | 2008 | Daily | 2026-02-12 |
| IEEE 1471 | IEEE | 2000 | Daily | 2026-02-12 |
| ISO/IEC 25010 | ISO/IEC | 2011 | Daily | 2026-02-12 |
| ISO/IEC 27001 | ISO/IEC | 2022 | Daily | 2026-02-12 |
| ISO/IEC 27002 | ISO/IEC | 2022 | Daily | 2026-02-12 |

**Security Standards:**

| Standard | Organization | Current Version | Monitoring Frequency | Last Review |
|----------|--------------|-----------------|-----------------|-------------|
| NIST SP 800-53 | NIST | Rev. 5 | Daily | 2026-02-12 |
| NIST SP 800-137 | NIST | 2011 | Daily | 2026-02-12 |
| OWASP Top 10 | OWASP | 2021 | Daily | 2026-02-12 |
| OWASP ASVS | OWASP | 4.0.3 | Daily | 2026-02-12 |
| OWASP MASVS | OWASP | 1.2.1 | Daily | 2026-02-12 |
| CWE Top 25 | MITRE | 2024 | Daily | 2026-02-12 |

**Industry Regulations:**

| Regulation | Jurisdiction | Last Review | Monitoring Frequency |
|------------|--------------|-------------|-----------------|
| GDPR | EU | 2026-02-12 | Daily |
| CCPA/CPRA | California, USA | 2026-02-12 | Daily |
| SOC 2 Type II | USA | 2026-02-12 | Weekly |
| ISO 27001 | International | 2026-02-12 | Weekly |
| HIPAA | USA | 2026-02-12 | Weekly |

### Response Timelines

| Change Type | Response Time | Action Timeline |
|-------------|---------------|-----------------|
| Minor Update | 30 days | 30 days to assess and update if needed |
| Major Update | 60 days | 60 days to assess, plan, and implement |
| Critical Update | 14 days | 14 days to assess, plan, and implement |
| Critical Change (Security-Related) | 7 days | 7 days to assess and implement |

### Change Classification

**Minor Update:**
- Editorial changes
- Clarifications
- Formatting changes
- No new requirements
- Impact: Low
- Response: 30 days

**Major Update:**
- New requirements
- Significant changes to existing requirements
- Structural changes
- Impact: Medium
- Response: 60 days

**Critical Update:**
- Security-relevant changes
- Mandatory compliance requirements
- Significant impact on compliance
- Impact: Critical
- Response: 14 days

**Deprecation:**
- Standard withdrawn
- Standard replaced
- No longer applicable
- Impact: High
- Response: 14 days

**New Standard:**
- New standard published
- Potentially applicable
- Impact assessment required
- Impact: Medium
- Response: 30 days

## Consequences

### Positive Consequences

- Early awareness of standard changes
- Proactive compliance adaptation
- Reduced risk of non-compliance
- Improved audit readiness
- Documented change history
- Knowledge base of standard impacts

### Negative Consequences

- Monitoring and maintenance overhead
- Potential for change notification fatigue
- Risk of missing critical updates
- Complexity of change impact analysis
- Resource requirements for remediation

## Alternatives Considered

1. **Manual Monitoring Only:** Rejected - insufficient coverage and slow response
2. **Periodic Manual Reviews:** Rejected - risk of missing critical updates
3. **External Monitoring Service:** Rejected - cost and data sovereignty concerns
4. **Simplified Alerting Only:** Rejected - lacks impact analysis and planning

## Implementation

### Architecture

```
Standards Scrapers
       |
       v
Change Detection
       |
       v
Change Classification
       |
       v
Impact Analysis
       |
       v
Alerting
       |
       v
Documentation
       |
       v
Standards Registry
```

### Scraper Configuration

```yaml
standards_scrapers:
  ieee:
    enabled: true
    url: "https://standards.ieee.org"
    rss_feed: "https://standards.ieee.org/rss.xml"
    api_url: "https://standards.ieee.org/api"
    api_key: "${IEEE_API_KEY}"
    scrape_interval: "24h"
    monitored_standards:
      - "IEEE 1016"
      - "IEEE 730"
      - "IEEE 829"
      - "IEEE 1471"

  iso:
    enabled: true
    url: "https://www.iso.org"
    rss_feed: "https://www.iso.org/feed.xml"
    api_url: "https://www.iso.org/api"
    scrape_interval: "24h"
    monitored_standards:
      - "ISO/IEC 25010"
      - "ISO/IEC 27001"
      - "ISO/IEC 27002"

  nist:
    enabled: true
    url: "https://csrc.nist.gov"
    rss_feed: "https://csrc.nist.gov/News/rss.xml"
    api_url: "https://csrc.nist.gov/api/publications"
    api_key: "${NIST_API_KEY}"
    scrape_interval: "24h"
    monitored_publications:
      - "SP 800-53"
      - "SP 800-137"

  owasp:
    enabled: true
    url: "https://owasp.org"
    rss_feed: "https://owasp.org/blog/feed.xml"
    scrape_interval: "24h"
    monitored_standards:
      - "OWASP Top 10"
      - "OWASP ASVS"
      - "OWASP MASVS"

  mitre:
    enabled: true
    url: "https://cwe.mitre.org"
    api_url: "https://cwe.mitre.org/api/v1"
    scrape_interval: "24h"
    monitored_standards:
      - "CWE Top 25"
      - "CWE"
```

### Change Detection Algorithm

```rust
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardUpdate {
    pub standard_id: String,
    pub standard_name: String,
    pub organization: String,
    pub old_version: Option<String>,
    pub new_version: String,
    pub change_type: ChangeType,
    pub impact_level: ImpactLevel,
    pub published_at: DateTime<Utc>,
    pub detected_at: DateTime<Utc>,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    MinorUpdate,
    MajorUpdate,
    CriticalUpdate,
    Deprecation,
    NewStandard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

// Change Detection
pub fn detect_standard_updates(
    scrapers: Vec<StandardScraper>,
    current_registry: &StandardsRegistry,
) -> Result<Vec<StandardUpdate>, Error> {
    let mut updates = Vec::new();

    for scraper in scrapers {
        let latest = scraper.fetch_latest().await?;
        let current = current_registry.get(&scraper.standard_id);

        if let Some(current_version) = current {
            let update = StandardUpdate {
                standard_id: scraper.standard_id.clone(),
                standard_name: scraper.standard_name.clone(),
                organization: scraper.organization.clone(),
                old_version: Some(current_version.clone()),
                new_version: latest.version.clone(),
                change_type: classify_change(&current_version, &latest.version, &latest.description),
                impact_level: assess_impact(&latest.description),
                published_at: latest.published_at,
                detected_at: Utc::now(),
                url: latest.url.clone(),
                description: latest.description.clone(),
            };
            updates.push(update);

            // Update registry
            current_registry.update(&scraper.standard_id, &latest.version);
        }
    }

    Ok(updates)
}

fn classify_change(old: &str, new: &str, description: &str) -> ChangeType {
    // Check for critical keywords
    let critical_keywords = vec!["security", "vulnerability", "mandatory", "compliance"];
    let has_critical = critical_keywords.iter()
        .any(|kw| description.to_lowercase().contains(kw));

    if has_critical {
        ChangeType::CriticalUpdate
    } else if old != new {
        // Version change detected
        let old_parts: Vec<_> = old.split('.').collect();
        let new_parts: Vec<_> = new.split('.').collect();
        let major_version_change = old_parts.get(0) != new_parts.get(0);

        if major_version_change {
            ChangeType::MajorUpdate
        } else {
            ChangeType::MinorUpdate
        }
    } else {
        ChangeType::NewStandard
    }
}

fn assess_impact(description: &str) -> ImpactLevel {
    // Use keyword-based impact assessment
    let high_impact_keywords = vec!["requirement", "shall", "control"];
    let has_high_impact = high_impact_keywords.iter()
        .any(|kw| description.to_lowercase().contains(kw));

    if has_high_impact {
        ImpactLevel::High
    } else if description.to_lowercase().contains("security") {
        ImpactLevel::Critical
    } else {
        ImpactLevel::Low
    }
}
```

### Alert Configuration

```yaml
# Alert Configuration for Standards Updates
alerting:
  standard_updates:
    enabled: true
    rules:
      - name: "critical_change"
        severity: "P2"
        conditions:
          change_type: ["CriticalUpdate", "Deprecation"]
        channels: ["slack"]
        recipients: ["#tachyon-standards", "engineering-manager@example.com"]

      - name: "major_update"
        severity: "P2"
        conditions:
          change_type: ["MajorUpdate"]
        channels: ["slack", "email"]
        recipients: ["#tachyon-standards", "engineering-manager@example.com"]

      - name: "minor_update"
        severity: "P4"
        conditions:
          change_type: ["MinorUpdate"]
        channels: ["slack"]
        recipients: ["#tachyon-standards"]

      - name: "new_standard"
        severity: "P3"
        conditions:
          change_type: ["NewStandard"]
        channels: ["slack", "email"]
        recipients: ["#tachyon-standards", "engineering-team@example.com"]
```

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Standard Update Detection Time | < 24 hours | Time from publication to detection |
| Alert Generation Time | < 1 hour | Time from detection to alert |
| Change Classification Accuracy | > 95% | Correct classifications / Total changes |
| Impact Assessment Accuracy | > 90% | Correct impact levels / Total changes |
| Standards Registry Accuracy | 100% | Registry matches actual versions |

## Related Decisions

- [ADR-097](adr-097-monitoring-strategy.md) - Continuous Monitoring Strategy
- [`.specs/11_continuous_monitoring/standard_updates.md`](../.specs/11_continuous_monitoring/standard_updates.md) - Standard Updates Monitoring Specification

## References

- IEEE Standards Portal: https://standards.ieee.org
- ISO Standards Portal: https://www.iso.org
- NIST Publications: https://csrc.nist.gov/publications
- OWASP: https://owasp.org
- MITRE CWE: https://cwe.mitre.org

---

**Document Status:** COMPLETE
**Owner:** Monitoring Engineer
**Reviewers:** TBD
**Approved By:** TBD
