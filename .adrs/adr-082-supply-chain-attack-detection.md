# ADR-082: Supply Chain Attack Detection

**Status:** Accepted
**Date:** 2026-02-12
**Decision Type:** Technical
**Context:** Phase 8.5 - Supply Chain Monitoring

---

## Context and Problem Statement

### Context
The threat model ([`.adrs/ identifies multiple supply chain attack vectors (SC-SUP-001 through SC-SUP-004). The monitoring strategy ([`.adrs/ requires real-time detection of supply chain attacks.

### Problem Statement
We lack automated supply chain attack detection that provides:
1. Real-time detection of dependency confusion attacks
2. Typosquatting attack prevention
3. Malicious update detection
4. Compromised dependency identification
5. Behavioral anomaly detection
6. Attack indicator correlation and analysis
7. Automated containment and remediation

Current challenges:
- Manual detection of supply chain threats
- No automated correlation of attack indicators
- Slow response to supply chain incidents
- Difficult to distinguish false positives from real attacks
- No comprehensive attack signature database

---

## Decision Drivers

### Security Requirements
From [`.adrs/
- **SC-SUP-001:** Malicious dependency injection
- **SC-SUP-002:** Vulnerability disclosure
- **SC-SUP-003:** License non-compliance
- **SC-SUP-004:** Supply chain backdoor

### Business Requirements
- **Detection Time:** <5 minutes for confirmed supply chain attacks
- **False Positive Rate:** <1% for attack detection
- **Containment:** Automated blocking of malicious packages
- **Investigation:** Root cause analysis for all suspected attacks
- **Reporting:** Comprehensive incident documentation

---

## Considered Alternatives

### Alternative 1: SaaS Threat Intelligence (Snyk, Sonatype, ReversingLabs)
**Description:** Use commercial threat intelligence platforms for attack detection

**Pros:**
- Comprehensive threat intelligence database
- Real-time attack feeds
- Automated correlation and analysis
- Rich incident management workflow
- Advanced behavioral analytics
- ML-based anomaly detection

**Cons:**
- Recurring licensing costs ($20K-$100K/year)
- Limited control over detection rules
- Vendor lock-in
- Potential data residency concerns
- Learning curve for team

**Rejection:** Cost-prohibitive for current scale, loss of control

### Alternative 2: Manual Detection Only
**Description:** Rely on manual security reviews and threat analysis

**Pros:**
- Zero additional cost
- Full control over detection process
- Deep understanding of attack vectors
- Can use any tools and techniques

**Cons:**
- High human error and omission risk
- Slow detection of threats
- Inconsistent coverage
- Difficult to maintain attack signatures
- Does not meet real-time detection requirements

**Rejection:** Insufficient for security posture

### Alternative 3: CI/CD Basic Checks Only
**Description:** Rely solely on CI/CD pipeline for basic integrity checks

**Pros:**
- Zero additional cost
- Native integration with build process
- Automated build verification

**Cons:**
- No behavioral analysis
- No real-time detection
- Limited to build-time checks
- No ongoing monitoring
- Difficult to correlate attack indicators

**Rejection:** Insufficient for comprehensive supply chain security

### Alternative 4: Multi-Tool Open-Source Detection (Chosen)
**Description:** Implement comprehensive attack detection using open-source tools

**Pros:**
- Zero licensing cost for open-source tools
- Full control over detection rules
- Real-time monitoring and detection
- Behavioral anomaly detection
- Multi-source correlation
- Native CI/CD integration
- Custom attack signature management

**Cons:**
- Requires tool installation and configuration
- Multiple tools to maintain and update
- Complex workflow orchestration
- Requires signature maintenance
- Ongoing research of new attack vectors

**Acceptance:** Best balance of detection capability, control, and cost

---

## Decision

### Chosen Approach: Multi-Tool Open-Source Detection

We will implement supply chain attack detection using:

**Detection Layers:**
- **Dependency Confusion:** Package name conflict detection
- **Typosquatting:** Similar package name detection
- **Malicious Updates:** Unusual version change detection
- **Compromised Dependencies:** Advisory database integration
- **Behavioral Analysis:** Anomaly detection in dependency behavior
- **Integrity Verification:** Hash and signature verification

**Detection Tools:**
- **Custom Scripts:** Python-based detection engine
- **GitHub API:** Package metadata analysis
- **Cargo-deny:** License and advisory checking
- **npm audit:** NPM vulnerability and dependency analysis
- **SBOM Verification:** Integrity and completeness checks

### Attack Detection Architecture

```
┌───────────────────────────────────────────────────────────────────────────┐
│              Supply Chain Attack Detection System                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  Data Collection Layer                                               │
│  - Dependency Metadata Analysis                                       │
│  - Advisory Database Integration                                    │
│  - Behavioral Baseline Establishment                                │
│  - Version Change Monitoring                                        │
│  - Integrity Verification                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  Detection Engine                                                    │
│  - Dependency Confusion Detection                                    │
│  - Typosquatting Detection                                         │
│  - Malicious Update Detection                                      │
│  - Compromised Dependency Detection                                 │
│  - Behavioral Anomaly Detection                                     │
│  - Attack Correlation                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  Response Layer                                                       │
│  - Automated Containment (CI/CD Blocking)                          │
│  - Alert Generation (Multi-Channel)                                │
│  - Incident Creation (GitHub Issues)                                │
│  - Investigation Workflow                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  Analysis Layer                                                       │
│  - Attack Pattern Analysis                                         │
│  - Indicator Correlation                                          │
│  - Signature Maintenance                                          │
│  - Threat Intelligence Integration                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  Reporting Layer                                                      │
│  - Attack Dashboard                                                 │
│  - Incident Metrics (MTTD, MTTR)                                  │
│  - Attack Trend Analysis                                             │
│  - Remediation Tracking                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### Dependency Confusion Detection

**Detection Logic:**
```python
#!/usr/bin/env python3
"""
Dependency Confusion Detection
"""

import json
from packaging.version import Version

def detect_dependency_confusion(package_name, registry, expected_source):
    """Detect potential dependency confusion attacks"""
    
    # Check for internal vs external package name conflicts
    is_internal = package_name.startswith('@tachyon/')
    if is_internal:
        # Check if public package with same name exists
        public_exists = check_public_registry(package_name.replace('@tachyon/', ''))
        if public_exists:
            return {
                'attack_type': 'dependency_confusion',
                'severity': 'critical',
                'package': package_name,
                'expected_source': 'internal',
                'actual_source': 'external_public',
                'confidence': 'high'
            }
    
    return None

def check_public_registry(package_name):
    """Check if package exists in public registry"""
    # Implementation would query crates.io, npm registry
    # Simplified for example
    return True  # Assume check
```

### Typosquatting Detection

**Detection Logic:**
```python
#!/usr/bin/env python3
"""
Typosquatting Detection
"""

import Levenshtein

def detect_typosquatting(package_name, popular_packages):
    """Detect potential typosquatting attacks"""
    
    min_distance = 2  # Threshold for similarity
    
    for popular_pkg in popular_packages:
        distance = Levenshtein.distance(package_name.lower(), popular_pkg.lower())
        if distance <= min_distance:
            return {
                'attack_type': 'typosquatting',
                'severity': 'critical',
                'package': package_name,
                'suspicious_package': popular_pkg,
                'similarity': 1 - (distance / len(popular_pkg)),
                'confidence': 'high'
            }
    
    return None

# Popular package list would be maintained for each ecosystem
RUST_POPULAR_PACKAGES = ['tokio', 'serde', 'axum', 'async-trait', 'hyper']
NPM_POPULAR_PACKAGES = ['react', 'vue', 'lodash', 'express', 'axios']
```

### Malicious Update Detection

**Detection Logic:**
```python
#!/usr/bin/env python3
"""
Malicious Update Detection
"""

from datetime import datetime, timedelta

def detect_malicious_update(package_name, previous_version, new_version, update_time):
    """Detect suspicious dependency updates"""
    
    # Check for suspicious patterns
    flags = []
    
    # Sudden major version bump without release notes
    version_diff = Version(new_version) - Version(previous_version)
    if version_diff.major >= 1 and not has_release_notes(new_version):
        flags.append('sudden_major_bump')
    
    # Update outside normal release window
    hour = update_time.hour
    if not (9 <= hour <= 17):  # 9 AM - 5 PM
        flags.append('unusual_time')
    
    # Multiple releases in short period
    recent_releases = get_recent_releases(package_name, days=30)
    if new_version in recent_releases:
        flags.append('multiple_releases')
    
    # Maintainer account change
    if maintainer_changed(package_name, update_time):
        flags.append('maintainer_change')
    
    if flags:
        confidence = 'high' if len(flags) >= 2 else 'medium'
        return {
            'attack_type': 'malicious_update',
            'severity': 'critical' if confidence == 'high' else 'medium',
            'package': package_name,
            'previous_version': str(previous_version),
            'new_version': str(new_version),
            'update_time': update_time.isoformat(),
            'flags': flags,
            'confidence': confidence
        }
    
    return None
```

### Compromised Dependency Detection

**Detection Logic:**
```python
#!/usr/bin/env python3
"""
Compromised Dependency Detection
"""

import requests

ADVISORY_DATABASES = [
    'https://rustsec.org/advisories',
    'https://github.com/advisories',
    'https://www.npmjs.com/advisories'
]

def check_compromised_database(package_name, version):
    """Check if package is in compromised advisory database"""
    
    for db_url in ADVISORY_DATABASES:
        try:
            response = requests.get(f'{db_url}/{package_name}', timeout=10)
            if response.status_code == 200:
                advisories = response.json()
                for advisory in advisories:
                    if advisory.get('affected_versions', '').startswith(str(version)):
                        return {
                            'attack_type': 'compromised_dependency',
                            'severity': 'critical',
                            'package': package_name,
                            'version': version,
                            'advisory_id': advisory.get('id'),
                            'published_date': advisory.get('published'),
                            'source': db_url,
                            'confidence': 'critical'
                        }
        except requests.RequestException:
            continue
    
    return None
```

### Behavioral Anomaly Detection

**Detection Logic:**
```python
#!/usr/bin/env python3
"""
Behavioral Anomaly Detection
"""

from statistics import stdev
from datetime import datetime, timedelta

class BehavioralAnalyzer:
    def __init__(self):
        self.baseline = {}
        
    def establish_baseline(self, package_metrics):
        """Establish behavioral baseline for packages"""
        for package, metrics in package_metrics.items():
            self.baseline[package] = {
                'avg_download_count': metrics.get('download_count', 0),
                'avg_stars': metrics.get('stars', 0),
                'avg_open_issues': metrics.get('open_issues', 0),
                'avg_last_update_days': metrics.get('days_since_last_update', 0)
            }
            
    def detect_anomaly(self, package, current_metrics):
        """Detect behavioral anomalies"""
        baseline = self.baseline.get(package)
        
        if not baseline:
            return None
        
        anomalies = []
        
        # Download count anomaly (sudden spike)
        if current_metrics.get('download_count', 0) > baseline['avg_download_count'] * 5:
            anomalies.append({
                'type': 'download_spike',
                'severity': 'medium',
                'baseline': baseline['avg_download_count'],
                'current': current_metrics.get('download_count', 0),
                'threshold_exceeded': 5.0
            })
        
        # Star count anomaly (sudden drop)
        if current_metrics.get('stars', 0) < baseline['avg_stars'] * 0.5:
            anomalies.append({
                'type': 'star_drop',
                'severity': 'medium',
                'baseline': baseline['avg_stars'],
                'current': current_metrics.get('stars', 0),
                'threshold_exceeded': 0.5
            })
        
        # Open issues anomaly (sudden increase)
        if current_metrics.get('open_issues', 0) > baseline['avg_open_issues'] + 10:
            anomalies.append({
                'type': 'open_issues_spike',
                'severity': 'low',
                'baseline': baseline['avg_open_issues'],
                'current': current_metrics.get('open_issues', 0),
                'threshold_exceeded': 10.0
            })
        
        if anomalies:
            return {
                'package': package,
                'anomalies': anomalies,
                'detected_at': datetime.now().isoformat(),
                'confidence': 'high' if len(anomalies) >= 2 else 'medium'
            }
        
        return None
```

---

## Consequences

### Positive Consequences
1. **Enhanced Security Posture:** Real-time detection reduces supply chain attack exposure
2. **Early Threat Detection:** Behavioral analysis detects suspicious patterns before attacks succeed
3. **Automated Response:** Faster containment and remediation of malicious packages
4. **Comprehensive Coverage:** Multiple detection layers cover various attack vectors
5. **Cost Efficiency:** Open-source tools avoid SaaS licensing costs
6. **Audit Trail:** Comprehensive incident logging for forensics
7. **Proactive Defense:** Correlation of attack indicators enables threat hunting

### Negative Consequences
1. **False Positive Risk:** Automated detection may flag legitimate updates as suspicious
2. **Maintenance Overhead:** Ongoing maintenance of detection tools and signatures
3. **Complexity:** Multiple detection layers require orchestration
4. **Research Burden:** Keeping up with new attack vectors requires ongoing research
5. **Initial Setup:** ~4 weeks for tool integration and configuration
6. **Alert Fatigue:** Poor tuning may generate excessive alerts

---

## Compliance and Standards Alignment

### NIST SP 800-53 Controls
- **SI-07:** Software and Information Integrity - Addressed by attack detection
- **SA-12:** Supply Chain Protection - Enhanced by comprehensive monitoring
- **IR-04:** Incident Handling - Addressed by automated response

### NIST SP 800-161 Requirements
- **Continuous Monitoring:** Real-time attack detection meets requirements
- **Risk Assessment:** Behavioral analysis enables risk-based decisions

### Executive Order 14028 Requirements
- **SBOM Generation:** SBOM verification enables attack detection
- **Vulnerability Disclosure:** Advisory database integration
- **Automated Testing:** CI/CD integration for attack detection

### CISA SBOM Guidelines
- **Supply Chain Risk Management:** Comprehensive monitoring addresses CISA requirements
- **Threat Detection:** Real-time detection of supply chain threats

---

## Related Decisions

- **ADR-077:** Supply Chain Monitoring ([`.adrs/adr-077-supply-chain-monitoring.md`](.adrs/adr-077-supply-chain-monitoring.md))
- **ADR-078:** Vulnerability Scanning ([`.adrs/adr-078-vulnerability-scanning.md`](.adrs/adr-078-vulnerability-scanning.md))
- **ADR-080:** Security Alerts ([`.adrs/adr-080-security-alerts.md`](.adrs/adr-080-security-alerts.md))

---

## References

**Internal Documents:**
- [`.adrs/ - Monitoring Strategy
- [`.adrs/ - Alerting Rules
- [`.adrs/ - Threat Model
- [`.adrs/ - CI/CD Pipeline

**External Resources:**
- CISA SBOM Guidance: https://www.cisa.gov/sbom
- RustSec Advisory Database: https://rustsec.org/advisories
- GitHub Advisory Database: https://github.com/advisories
- NPM Advisory Database: https://www.npmjs.com/advisories

**External Standards:**
- NIST SP 800-53: Security and Privacy Controls
- NIST SP 800-161: Supply Chain Risk Management
- Executive Order 14028: Improving Software Supply Chain Security

---

## Approval

**Approved By:** Security Team Lead
**Approval Date:** 2026-02-12
**Reviewers:** Security Team, DevOps Team, Infrastructure Team
**Implementation Status:** Approved for Implementation
