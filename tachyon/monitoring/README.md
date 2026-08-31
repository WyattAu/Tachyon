# Tachyon Monitoring Stack

Grafana + Prometheus monitoring for the Tachyon server.

## Quick Start

```bash
cd monitoring

docker compose -f docker-compose.monitoring.yml up -d
```

This starts:
- **Prometheus** (port 9090) — scrapes Tachyon and infrastructure metrics
- **Grafana** (port 3000) — pre-configured dashboards
- **PostgreSQL Exporter** (port 9187) — database metrics
- **Node Exporter** (port 9100) — host system metrics

The optional Nginx exporter and metrics-only Nginx sidecar are enabled together with `--profile metrics-only` when an Nginx metrics endpoint is available. The PostgreSQL exporter expects an already-running PostgreSQL service on the shared `tachyon-network`; it is not defined by this Compose file. For native staging, point Prometheus at the host-reachable staging URL instead of the Docker-only `tachyon-server:8080` target.

## Access

| Service | URL | Default Credentials |
|---------|-----|-------------------|
| Grafana | http://localhost:3000 | `admin` / `admin` |
| Prometheus | http://localhost:9090 | none |
| Prometheus targets | http://localhost:9090/targets | — |

**Change Grafana credentials** before deploying to production:

```bash
GF_ADMIN_USER=myuser GF_ADMIN_PASSWORD=mypassword docker compose -f docker-compose.monitoring.yml up -d grafana
```

## Dashboards

Two pre-configured dashboards are loaded automatically:

### Tachyon Overview (`tachyon-overview`)
- System overview (uptime, CPU, memory, service status)
- HTTP request rate (total, by status class)
- Request duration (avg, p50/p95/p99)
- Error rate
- Database connection pool
- Slow query count
- Active WebSocket connections
- Nginx upstream response time

### Tachyon Database (`tachyon-database`)
- Connection pool gauges (active, idle, waiting, utilization %)
- Query duration histogram (p50/p95/p99)
- Slow query rate
- PostgreSQL transaction rate (commits/rollbacks)
- Lock contention and deadlock rate
- Row operations
- API cache hit/miss/bypass ratio
- Migration status

## Metrics Exposed by Tachyon

Tachyon exposes metrics at two endpoints:

| Endpoint | Format | Metrics |
|----------|--------|---------|
| `/metrics/prometheus` | Prometheus text | Global `metrics` crate recorder |
| `/metrics/app` | Prometheus text | Custom Tachyon metrics |

### Custom Metrics (`/metrics/app`)

| Metric | Type | Description |
|--------|------|-------------|
| `tachyon_requests_total` | counter | Total HTTP requests |
| `tachyon_requests_successful` | counter | Requests with 2xx-3xx status |
| `tachyon_requests_failed` | counter | Requests with 4xx-5xx status |
| `tachyon_request_duration_avg_ms` | gauge | Average request duration (ms) |
| `tachyon_uptime_seconds` | gauge | Server uptime |
| `tachyon_version_info` | gauge | Server version (label: `version`) |

### Metrics Available via `metrics` Crate

These require adding `metrics::counter!()`, `metrics::gauge!()`, `metrics::histogram!()` calls in application code:

| Metric | Type | Where to Add |
|--------|------|-------------|
| `tachyon_db_pool_active` | gauge | `ConnectionManager` or pool wrapper |
| `tachyon_db_pool_idle` | gauge | `ConnectionManager` or pool wrapper |
| `tachyon_db_pool_size` | gauge | `ConnectionManager` or pool wrapper |
| `tachyon_db_pool_max` | gauge | `ConnectionManager` or pool wrapper |
| `tachyon_slow_queries_total` | counter | `SlowQueryLogger::record_query()` |
| `tachyon_db_avg_query_time_ms` | gauge | Periodic export from `SlowQueryLogger` |
| `tachyon_ws_connections_active` | gauge | `ConnectionManager::add_client/remove_client` |
| `tachyon_crdt_connections_active` | gauge | `CrdtConnectionManager` |
| `tachyon_ws_connects_total` | counter | `ConnectionManager::add_client` |
| `tachyon_ws_disconnects_total` | counter | `ConnectionManager::remove_client` |
| `tachyon_api_cache_hits_total` | counter | `api_cache_middleware` (X-Cache: HIT) |
| `tachyon_api_cache_misses_total` | counter | `api_cache_middleware` (X-Cache: MISS) |
| `tachyon_api_cache_bypass_total` | counter | `api_cache_middleware` (X-Cache: BYPASS) |
| `tachyon_migrations_pending` | gauge | Migration runner |
| `tachyon_migrations_applied_total` | counter | Migration runner |
| `http_request_duration_seconds_bucket` | histogram | Request tracing middleware |

## Adding Custom Metrics

In any Rust handler or middleware, use the `metrics` crate:

```rust
use metrics::{counter, gauge, histogram};

counter!("my_custom_counter_total", "label" => "value").increment(1);
gauge!("my_custom_gauge").set(42.0);
histogram!("my_custom_duration_seconds").record(duration.as_secs_f64());
```

The `metrics_exporter_prometheus` recorder is installed at startup (see `lib.rs:install_metrics()`), so all `metrics` crate calls are automatically exported to `/metrics/prometheus`.

## Alerting Rules

Defined in `prometheus/alerts.yml`:

| Alert | Severity | Condition | Duration |
|-------|----------|-----------|----------|
| HighErrorRate | warning | >5% failed requests | 5m |
| HighLatency | warning | p99 >2s | 5m |
| DatabasePoolExhausted | critical | >90% pool utilization | 2m |
| HighSlowQueryRate | warning | >10 slow queries | 5m |
| WebSocketDisconnectStorm | warning | >100 disconnects/min | 1m |
| ServiceDown | critical | target unreachable | 1m |
| DatabaseNotReady | critical | >50% error rate | 3m |
| ConnectionPoolWaiting | warning | >5 waiting connections | 2m |

### Configuring Alert Notifications

To receive alerts, add an Alertmanager service and configure notification channels.

#### Email

```yaml
# alertmanager/alertmanager.yml
global:
  smtp_smarthost: "smtp.example.com:587"
  smtp_from: "alerts@tachyon.example.com"
  smtp_auth_username: "alerts@tachyon.example.com"
  smtp_auth_password: "secret"

route:
  receiver: "email"
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h

receivers:
  - name: "email"
    email_configs:
      - to: "oncall@tachyon.example.com"
        send_resolved: true
```

#### Slack

```yaml
receivers:
  - name: "slack"
    slack_configs:
      - api_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
        channel: "#alerts"
        send_resolved: true
        title: "{{ .GroupLabels.alertname }}"
        text: >-
          {{ range .Alerts }}
          *{{ .Status | toUpper }}*: {{ .Annotations.summary }}
          {{ end }}
```

#### PagerDuty

```yaml
receivers:
  - name: "pagerduty"
    pagerduty_configs:
      - service_key: "YOUR_PAGERDUTY_SERVICE_KEY"
        severity: "{{ .CommonLabels.severity }}"
```

Add Alertmanager to the monitoring stack:

```yaml
# In docker-compose.monitoring.yml
alertmanager:
  image: prom/alertmanager:v0.27.0
  container_name: tachyon-alertmanager
  restart: unless-stopped
  command:
    - "--config.file=/etc/alertmanager/alertmanager.yml"
  ports:
    - "9093:9093"
  volumes:
    - ./alertmanager/alertmanager.yml:/etc/alertmanager/alertmanager.yml:ro
    - alertmanager_data:/alertmanager
  networks:
    - tachyon-monitoring
```

## Data Retention

- **Prometheus**: 30 days, 8GB max (configurable in `prometheus.yml`)
- **Grafana**: Persistent volume (retains dashboards and settings)

## Connecting to an Existing Docker Network

The monitoring stack expects a `tachyon_tachyon-network` network (from the main `docker-compose.yml`). If the network name differs, update `docker-compose.monitoring.yml`:

```yaml
networks:
  tachyon-network:
    external: true
    name: your-actual-network-name
```

## Troubleshooting

```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {job, health, lastScrape}'

# Check if Tachyon metrics are being scraped
curl http://localhost:9090/api/v1/query?query=up{job="tachyon"} | jq

# View Prometheus logs
docker logs tachyon-prometheus --tail 100

# View Grafana logs
docker logs tachyon-grafana --tail 100

# Force dashboard reload
curl -X POST http://admin:admin@localhost:3000/api/admin/provisioning/dashboards/reload
```
