# Tachyon Grafana Dashboards

## Overview

This directory contains Grafana dashboard definitions for monitoring the Tachyon platform.

## Dashboards

### API Overview (`api-overview.json`)

The primary operational dashboard for monitoring API health and performance.

| Panel | Type | Description |
|-------|------|-------------|
| Request Rate | Graph | Requests per second by method and endpoint |
| Error Rate (%) | Graph | Percentage of 5xx responses (threshold: >1%) |
| P50 / P95 / P99 Latency | Graph | Request latency percentiles in milliseconds |
| Active WebSocket Connections | Graph | Currently active WebSocket connections |
| Database Pool Utilization (%) | Graph | Connection pool usage (threshold: >80%) |
| Cache Hit Rate (%) | Graph | Cache effectiveness (threshold: <70% warning) |
| HTTP Status Code Distribution | Pie Chart | Breakdown of HTTP status codes |
| Top 10 Endpoints by Request Rate | Bar Gauge | Most-requested endpoints |

**Auto-refresh**: 30 seconds
**Time range**: Last 1 hour

## Key Metrics

### Application Metrics (Prometheus format)

```
tachyon_requests_total          # Total HTTP requests
tachyon_requests_successful      # Successful (non-error) requests
tachyon_requests_failed          # Failed (error) requests
tachyon_request_duration_avg_ms  # Average request duration
tachyon_uptime_seconds           # Server uptime
tachyon_version_info             # Server version (label: version)
```

### Infrastructure Metrics

```
http_requests_total{method, endpoint, status}              # Per-endpoint request counts
http_request_duration_seconds_bucket{le, method, endpoint}  # Latency histogram
db_pool_connections_active                                   # Active database connections
db_pool_connections_max                                      # Maximum pool size
websocket_connections_active                                 # Active WebSocket connections
cache_hits_total                                             # Cache hits
cache_requests_total                                         # Total cache requests
```

## Dashboards to Add

- **System Resources**: CPU, memory, disk, network per instance
- **Database Deep Dive**: Query performance, lock contention, replication lag
- **Search Index**: Tantivy index size, query latency, reindex status
- **Business Metrics**: Document creation rate, active users, search usage
- **SLA Dashboard**: Uptime percentage, error budget remaining
