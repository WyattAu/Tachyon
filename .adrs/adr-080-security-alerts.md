# ADR-080: Security Alerts

**Status:** Accepted
**Date:** 2026-02-12
**Decision Type:** Technical
**Context:** Phase 8.5 - Supply Chain Monitoring

---

## Context and Problem Statement

### Context
The supply chain monitoring system ([`.adrs/ requires an alerting mechanism to notify teams of security incidents, compliance issues, and supply chain threats. The alerting rules ([`.adrs/ define the alert conditions and severity levels.

### Problem Statement
We lack an automated security alerting system that provides:
1. Multi-channel notification (Slack, PagerDuty, Email, GitHub)
2. Severity-based alert routing
3. Escalation management
4. Alert deduplication and aggregation
5. Integration with monitoring systems
6. Alert acknowledgment and tracking

Current challenges:
- Manual notification of security incidents
- No central alerting dashboard
- Inconsistent response times
- No alert history or audit trail
- Risk of missed or delayed alerts

---

## Decision Drivers

### Security Requirements
From [`.adrs/
- **NIST-AU-02:** Audit Events
- **NIST-AU-06:** Audit Review, Analysis, and Reporting
- **NIST-IR-04:** Incident Handling

### Business Requirements
- **Response SLA:** <30 minutes acknowledgment for Critical alerts
- **Resolution SLA:** <4 hours for Critical alerts
- **Multi-Channel:** Alerts must reach teams via multiple channels
- **Escalation:** Automatic escalation for unacknowledged critical alerts
- **Audit Trail:** All alerts must be logged and tracked

### Operational Constraints
- **False Positive Rate:** <5% to prevent alert fatigue
- **Alert Throughput:** System must handle 1000+ alerts/day
- **Channel Reliability:** 99.9%+ delivery reliability
- **Dashboard Availability:** Real-time alert visualization

---

## Considered Alternatives

### Alternative 1: SaaS Alerting (PagerDuty, VictorOps, Datadog)
**Description:** Use commercial SaaS platforms for security alerting

**Pros:**
- Mature, battle-tested alerting infrastructure
- Rich UI and analytics
- Advanced routing and escalation
- High reliability (99.9%+ SLA)
- Integrated on-call management
- Comprehensive alert history and metrics

**Cons:**
- Recurring licensing costs ($5K-$20K/year)
- Limited control over alerting rules
- Vendor lock-in and learning curve
- Potential data residency concerns
- Additional external dependencies

**Rejection:** Cost-prohibitive for current scale, loss of control

### Alternative 2: GitHub Issues Only
**Description:** Rely solely on GitHub Issues for security alerts

**Pros:**
- Zero additional cost
- Native GitHub integration
- Simple setup and maintenance
- Good for code review workflow

**Cons:**
- No real-time notification
- NoPagerDuty for critical alerts
- Limited to GitHub ecosystem
- No advanced routing or escalation
- No centralized alerting
- Difficult to track response times

**Rejection:** Insufficient for critical security incidents

### Alternative 3: Email-Only Alerting
**Description:** Use email for all security alerts

**Pros:**
- Zero additional cost
- Universal access
- Simple implementation
- Good for detailed alert content

**Cons:**
- Slow delivery (email delays)
- No real-time notification
- No escalation capability
- Risk of emails being missed or delayed
- No central alerting dashboard

**Rejection:** Does not meet response SLA requirements

### Alternative 4: Multi-Channel Open-Source Alerting (Chosen)
**Description:** Implement open-source alerting system with multiple notification channels

**Pros:**
- Zero licensing cost
- Full control over alerting rules
- Multi-channel routing (Slack, PagerDuty, Email, GitHub)
- Native integration with monitoring systems
- Custom escalation policies
- Real-time dashboard
- Comprehensive audit trail
- No vendor lock-in

**Cons:**
- Requires infrastructure setup and maintenance
- Multiple integrations to configure
- Channel reliability management required
- Alert deduplication complexity
- Requires ongoing script maintenance

**Acceptance:** Best balance of cost, control, and reliability

---

## Decision

### Chosen Approach: Multi-Channel Open-Source Alerting

We will implement security alerting using:

**Notification Channels:**
- **PagerDuty:** Critical alerts only, on-call rotation
- **Slack:** All alert types, team channels
- **Email:** All alert types, detailed reports
- **GitHub Issues:** Automated issue creation for actionable alerts

**Alerting Engine:**
- Python-based rule engine for alert generation
- Webhook-based integration with notification services
- Alert aggregation and deduplication
- Response time tracking and SLA monitoring
- Escalation automation

### Alerting Architecture

```
┌───────────────────────────────────────────────────────────────────────────┐
│                    Security Alerting System                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  Alert Generation Layer                                              │
│  - Rule-Based Processing                                            │
│  - Severity Classification                                          │
│  - Alert Aggregation                                              │
│  - Deduplication                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│  Routing Layer                                                       │
│  - PagerDuty (Critical)                                            │
│  - Slack (All Severities)                                          │
│  - Email (All Severities)                                         │
│  - GitHub Issues (Actionable)                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  Escalation Layer                                                    │
│  - Auto-Escalation (Unacknowledged Critical)                          │
│  - Manual Escalation (Level 2, Level 3)                           │
│  - On-Call Rotation                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  Tracking Layer                                                     │
│  - Alert Dashboard                                                 │
│  - Response Time Metrics                                              │
│  - Audit Trail                                                     │
│  - SLA Compliance                                                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### Alert Rules Engine

**Rule Configuration:**
```yaml
# config/alert-rules.yml
rules:
  - id: VULN-CRIT-001
    trigger:
      severity: CRITICAL
      type: vulnerability
    action:
      channels: [pagerduty, slack, email, github]
      escalation: auto
      
  - id: VULN-HIGH-001
    trigger:
      severity: HIGH
      type: vulnerability
    action:
      channels: [slack, email, github]
      escalation: manual
      
  - id: LIC-PROH-001
    trigger:
      license: prohibited
    action:
      channels: [pagerduty, slack, email, github]
      escalation: auto
      
  - id: SBOM-INT-001
    trigger:
      type: sbom_integrity
    action:
      channels: [pagerduty, slack, email, github]
      escalation: auto
```

**Alert Processing Pipeline:**
```python
#!/usr/bin/env python3
"""
Security Alert Processing Pipeline
"""

import json
import logging
from datetime import datetime

class AlertProcessor:
    def __init__(self, rules_file):
        self.rules = self.load_rules(rules_file)
        self.alert_history = []
        
    def process_event(self, event):
        """Process security event and generate alerts"""
        matching_rules = self.find_matching_rules(event)
        
        for rule in matching_rules:
            alert = self.generate_alert(event, rule)
            self.route_alert(alert)
            self.track_alert(alert)
            
    def find_matching_rules(self, event):
        """Find rules matching event conditions"""
        return [r for r in self.rules if r.matches(event)]
        
    def generate_alert(self, event, rule):
        """Generate alert object"""
        return {
            'id': f'ALERT-{datetime.now().isoformat()}',
            'severity': rule.severity,
            'type': event.type,
            'source': event.source,
            'message': self.format_message(event, rule),
            'metadata': event.metadata,
            'created_at': datetime.now().isoformat(),
            'rule_id': rule.id
        }
        
    def route_alert(self, alert):
        """Route alert to configured channels"""
        for channel in alert['channels']:
            self.send_to_channel(channel, alert)
            
    def send_to_channel(self, channel, alert):
        """Send alert to specific channel"""
        if channel == 'pagerduty':
            self.send_pagerduty(alert)
        elif channel == 'slack':
            self.send_slack(alert)
        elif channel == 'email':
            self.send_email(alert)
        elif channel == 'github':
            self.create_github_issue(alert)
            
    def send_pagerduty(self, alert):
        """Send alert to PagerDuty"""
        from pdpy import EventsV2
        
        pd = EventsV2(self.get_pagerduty_token())
        pd.trigger(
            summary=alert['message'],
            severity=alert['severity'],
            custom_details=json.dumps(alert['metadata'])
        )
        
    def send_slack(self, alert):
        """Send alert to Slack"""
        import slack_sdk
        
        slack = slack_sdk.WebClient(token=self.get_slack_token())
        slack.chat_postMessage(
            channel=self.get_slack_channel(alert['severity']),
            text=self.format_slack_message(alert),
            attachments=[self.format_slack_attachment(alert)]
        )
        
    def send_email(self, alert):
        """Send alert via email"""
        import smtplib
        from email.mime.text import MIMEText
        
        msg = MIMEText(self.format_email_body(alert))
        msg['Subject'] = self.format_email_subject(alert)
        msg['From'] = self.get_from_address()
        msg['To'] = self.get_recipients(alert)
        
        with smtplib.SMTP(self.get_smtp_server()) as server:
            server.send_message(msg)
            
    def create_github_issue(self, alert):
        """Create GitHub issue for actionable alerts"""
        from github import Github
        
        gh = Github(self.get_github_token())
        repo = gh.get_repo(self.get_repo_name())
        
        issue = repo.create_issue(
            title=self.format_issue_title(alert),
            body=self.format_issue_body(alert),
            labels=self.get_issue_labels(alert),
            assignees=self.get_assignees(alert)
        )
```

### Notification Channels

**PagerDuty Configuration:**
```yaml
# config/pagerduty.yml
service_key: ${PAGERDUTY_SERVICE_KEY}
api_key: ${PAGERDUTY_API_KEY}
escalation_policy:
  critical:
    timeout_minutes: 5
    escalation_level: 2
  high:
    timeout_minutes: 30
    escalation_level: 3
```

**Slack Configuration:**
```yaml
# config/slack.yml
channels:
  critical: "#security-alerts"
  high: "#security-alerts"
  medium: "#devops"
  low: "#notifications"
  
webhook_url: ${SLACK_WEBHOOK_URL}
bot_name: "tachyon-security"
bot_icon: ":warning:"
```

### Escalation Policy

**Critical Alert Escalation:**
```
Time 0: Initial alert sent
Time +5 min: No acknowledgment? Escalate to Level 2 (Engineering Manager)
Time +30 min: No acknowledgment? Escalate to Level 3 (CTO)
Time +4 hours: No resolution? Executive notification
```

**High/Medium/Low Alert Escalation:**
```
Time 0: Initial alert sent
Time +4 hours: No acknowledgment? Escalate to Level 2 (Team Lead)
Time +24 hours: No resolution? Weekly summary
```

---

## Alert Dashboard

**Dashboard Features:**
1. **Real-time Alert Feed:** Live stream of incoming alerts
2. **Alert History:** Searchable and filterable alert log
3. **Response Time Metrics:** MTTR, MTTA per severity
4. **SLA Compliance:** SLA adherence percentage
5. **Active Incidents:** Currently open critical/high incidents
6. **Alert Trends:** Weekly/monthly alert volume and patterns
7. **Team Performance:** Response times by team member

**Dashboard Architecture:**
```yaml
# dashboard-config.yml
components:
  - name: alert_feed
    type: real_time_stream
    source: alert_processing
    
  - name: alert_history
    type: table
    columns:
      - timestamp
      - severity
      - type
      - message
      - status
      
  - name: response_metrics
    type: charts
    metrics:
      - mtt_by_severity
      - mta_by_severity
      - sla_compliance
      
  - name: active_incidents
    type: list
    filter: open_critical_or_high
```

---

## Consequences

### Positive Consequences
1. **Improved Response Time:** Multi-channel routing ensures rapid notification
2. **Reduced Risk:** Escalation ensures critical alerts get attention
3. **Better Visibility:** Dashboard provides real-time situational awareness
4. **Audit Trail:** Comprehensive logging for compliance and forensics
5. **SLA Compliance:** Automated tracking and reporting of response times
6. **Cost Efficiency:** Open-source solution avoids SaaS licensing costs
7. **Team Accountability:** Response time metrics enable performance tracking

### Negative Consequences
1. **Infrastructure Overhead:** Requires PagerDuty and Slack setup
2. **Maintenance Burden:** Ongoing maintenance of alerting rules and integrations
3. **Alert Fatigue Risk:** Poor tuning may generate excessive alerts
4. **False Positives:** May generate alerts requiring manual triage
5. **Channel Failure Risk:** Dependency on external notification services

---

## Compliance and Standards Alignment

### NIST SP 800-53 Controls
- **AU-02:** Audit Events - Addressed by comprehensive alert logging
- **AU-04:** Audit Review, Analysis, and Reporting - Addressed by dashboard
- **AU-06:** Audit Reduction - Addressed by alert aggregation
- **IR-04:** Incident Handling - Addressed by escalation automation

### NIST SP 800-161 Requirements
- **Continuous Monitoring:** Alerting system provides ongoing visibility
- **Risk Assessment:** Alert severity classification enables risk-based response

### OWASP Top 10
- **A09:** Logging and Monitoring Failures - Addressed by comprehensive alerting

---

## Related Decisions

- **ADR-077:** Supply Chain Monitoring ([`.adrs/adr-077-supply-chain-monitoring.md`](.adrs/adr-077-supply-chain-monitoring.md))
- **ADR-078:** Vulnerability Scanning ([`.adrs/adr-078-vulnerability-scanning.md`](.adrs/adr-078-vulnerability-scanning.md))
- **ADR-082:** Supply Chain Attack Detection ([`.adrs/adr-082-supply-chain-attack-detection.md`](.adrs/adr-082-supply-chain-attack-detection.md))

---

## References

**Internal Documents:**
- [`.adrs/ - Monitoring Strategy
- [`.adrs/ - Alerting Rules
- [`.adrs/ - Threat Model
- [`.adrs/ - Compliance Matrix

**External Services:**
- PagerDuty: https://www.pagerduty.com
- Slack: https://api.slack.com
- GitHub API: https://docs.github.com/en/rest

**External Libraries:**
- slack-sdk: https://github.com/slackapi/python-slack-sdk
- pdpy: https://github.com/PagerDuty/pdpy
- PyGithub: https://github.com/PyGithub/PyGithub

**External Standards:**
- NIST SP 800-53: Security and Privacy Controls
- NIST SP 800-161: Supply Chain Risk Management

---

## Approval

**Approved By:** Security Team Lead
**Approval Date:** 2026-02-12
**Reviewers:** Security Team, DevOps Team, Infrastructure Team
**Implementation Status:** Approved for Implementation
