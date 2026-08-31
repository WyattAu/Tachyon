---
title: Anomalous Pattern Alerting
description: Alerting rules and thresholds for detecting anomalous behavior in Tachyon
date: 2026-05-23
category: operations
order: 3
---

# Anomalous Pattern Alerting

Alerting rules for detecting and responding to anomalous behavior.

## Alert Severity Levels

| Level | Meaning | Response Time | Notification |
|-------|---------|---------------|--------------|
| P1 — Critical | Service down or data loss | < 15 min | PagerDuty + Slack |
| P2 — High | Degraded performance or security event | < 1 hour | Slack + Email |
| P3 — Medium | Non-critical anomaly | < 4 hours | Slack |
| P4 — Low | Informational | Next business day | Dashboard |

## Metric Thresholds

### Application Metrics

| Metric | Warning | Critical | Window |
|--------|---------|----------|--------|
| 5xx Error Rate | > 3% | > 10% | 2 min rolling |
| p50 Response Time | > 500ms | > 2s | 5 min rolling |
| p99 Response Time | > 2s | > 10s | 5 min rolling |
| Request Rate Drop | > 50% drop from baseline | > 80% drop | 5 min |
| WebSocket Connections | > 10,000 concurrent | > 50,000 concurrent | Instant |
| Rate Limit Triggers | > 100/min | > 1,000/min | 1 min rolling |

### Infrastructure Metrics

| Metric | Warning | Critical | Window |
|--------|---------|----------|--------|
| CPU Usage | > 70% | > 90% | 5 min |
| Memory Usage | > 80% | > 95% | 5 min |
| Disk Usage | > 75% | > 90% | 15 min |
| Database Connections | > 80% pool | > 95% pool | 2 min |
| Database Query Time p99 | > 500ms | > 2s | 5 min |

### Security Anomalies

| Pattern | Threshold | Severity | Action |
|---------|-----------|----------|--------|
| Failed auth attempts (single IP) | > 20/min | P2 | Auto-block IP 15 min |
| SQL injection patterns | > 0 | P1 | Log + alert + review |
| XSS attempt patterns | > 5/min from single IP | P2 | Log + alert |
| Path traversal attempts | > 0 | P2 | Log + alert |
| Token reuse from different IPs | > 3 distinct IPs | P2 | Invalidate token + alert |
| Unusual API key usage pattern | > 10x baseline | P3 | Log + review |

## Prometheus Alert Rules

```yaml
groups:
  - name: tachyon_application
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[2m]) / rate(http_requests_total[2m]) > 0.10
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "High 5xx error rate ({{ $value | humanizePercentage }})"

      - alert: ElevatedErrorRate
        expr: rate(http_requests_total{status=~"5.."}[2m]) / rate(http_requests_total[2m]) > 0.03
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "Elevated 5xx error rate ({{ $value | humanizePercentage }})"

      - alert: HighResponseTime
        expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "p99 response time above 10s"

      - alert: RequestRateDrop
        expr: rate(http_requests_total[5m]) < (avg_over_time(rate(http_requests_total[5m])[1d]) * 0.2)
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Request rate dropped >80% from daily baseline"

      - alert: HighWebSocketConnections
        expr: tachyon_websocket_connections_total > 50000
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "WebSocket connections above 50,000"

      - alert: RateLimitTriggered
        expr: rate(tachyon_rate_limit_exceeded_total[1m]) > 1000
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Rate limiting triggered >1,000/min"

  - name: tachyon_security
    rules:
      - alert: SQLInjectionAttempt
        expr: rate(tachyon_security_events{type="sql_injection"}[1m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "SQL injection attempt detected"

      - alert: BruteForceAttempt
        expr: rate(tachyon_auth_failures_total{ip="group"}[1m]) > 20
        for: 1m
        labels:
          severity: high
        annotations:
          summary: "Possible brute force from {{ $labels.ip }}"
```

## Anomaly Detection

Beyond threshold-based alerting, monitor for:

1. **Time-of-day patterns**: Compare current metrics against same-day/hour historical baseline
2. **Deployment correlation**: Flag metric changes within 30 minutes of deployment
3. **Cascading failures**: Monitor dependency health (PostgreSQL, Redis, SMTP)
4. **Slow drift**: Track week-over-week trends for memory, latency, error rates
