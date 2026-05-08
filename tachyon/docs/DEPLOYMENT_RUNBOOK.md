# Tachyon Deployment Runbook

> Comprehensive guide for deploying, operating, and maintaining Tachyon in production.

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Environment Variables](#2-environment-variables)
3. [Database Setup](#3-database-setup)
4. [Configuration](#4-configuration)
5. [Docker Deployment](#5-docker-deployment)
6. [Nginx + SSL Setup](#6-nginx--ssl-setup)
7. [Monitoring Setup](#7-monitoring-setup)
8. [Backup & Restore](#8-backup--restore)
9. [Scaling Guide](#9-scaling-guide)
10. [Rollback Procedure](#10-rollback-procedure)
11. [Health Checks](#11-health-checks)
12. [Troubleshooting](#12-troubleshooting)
13. [Security Checklist](#13-security-checklist)
14. [Performance Tuning](#14-performance-tuning)
15. [Incident Response](#15-incident-response)

---

## 1. Prerequisites

### Hardware (Minimum)

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 4 GB | 8+ GB |
| Disk | 20 GB SSD | 50+ GB SSD |
| Network | 100 Mbps | 1 Gbps |

Per-service resource allocations (from `docker-compose.prod.yml`):

| Service | CPU Limit | Memory Limit | CPU Reserved | Memory Reserved |
|---------|-----------|-------------|-------------|----------------|
| PostgreSQL | 2 | 2 GB | 1 | 1 GB |
| Redis | 1 | 512 MB | 0.5 | 256 MB |
| Backend (per replica) | 2 | 1 GB | 0.5 | 512 MB |
| Frontend | 1 | 256 MB | 0.25 | 128 MB |
| Nginx | default | default | default | default |

### Software

| Software | Minimum Version | Notes |
|----------|----------------|-------|
| Docker | 24.0+ | With compose v2 plugin |
| Docker Compose | v2.20+ | `docker compose` (not `docker-compose`) |
| Git | 2.30+ | For cloning the repository |

### Accounts

| Account | Purpose | Required |
|---------|---------|----------|
| GitHub account with GHCR access | Pull Docker images | Yes |
| Domain name | TLS certificates | Yes (production) |
| Let's Encrypt | SSL certificates | Auto-provisioned |
| AWS/S3 (optional) | Off-site backups | No |
| Slack (optional) | Deployment notifications | No |

### Verify Docker

```bash
docker --version
docker compose version
```

---

## 2. Environment Variables

All variables are defined in `.env.example`. Copy and customize:

```bash
cp .env.example .env
```

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `JWT_SECRET` | JWT signing key (min 32 chars, 64 recommended) | `openssl rand -hex 32` |
| `DATABASE_URL` | PostgreSQL connection string | `postgres://tachyon:pass@postgres:5432/tachyon` |
| `TACHYON_HOST` | Bind address | `0.0.0.0` |
| `TACHYON_PORT` | Bind port | `8080` |

### Docker Compose Variables

These are consumed by `docker-compose.prod.yml`:

| Variable | Description | Example |
|----------|-------------|---------|
| `REGISTRY` | Container registry | `ghcr.io` |
| `IMAGE_NAME` | Image path | `wyattau/tachyon` |
| `VERSION` | Image tag to deploy | `v1.2.3` or `latest` |
| `POSTGRES_USER` | Database user | `tachyon` |
| `POSTGRES_PASSWORD` | Database password | `<generate with openssl>` |
| `POSTGRES_DB` | Database name | `tachyon` |
| `REDIS_PASSWORD` | Redis auth password | `<generate with openssl>` |
| `JWT_SECRET` | JWT signing secret | `<generate with openssl>` |
| `CORS_ORIGINS` | Allowed CORS origins | `https://tachyon.example.com` |
| `CERTBOT_DOMAIN` | Domain for SSL cert | `tachyon.example.com` |
| `CERTBOT_EMAIL` | Email for Let's Encrypt | `admin@example.com` |
| `CERTBOT_STAGING` | Use staging CA (set `true` to avoid rate limits) | `false` |
| `BACKUP_RETENTION` | Number of backups to keep | `7` |
| `BACKUP_S3_BUCKET` | S3 bucket for off-site backups | (optional) |

### Authentication

| Variable | Description | Default |
|----------|-------------|---------|
| `JWT_EXPIRATION` | Token TTL in seconds | `3600` |
| `GUEST_LOGIN_ENABLED` | Allow guest access | `false` |
| `PUBLIC_NOTES_ENABLED` | Allow unauthenticated note reads | `false` |

### OAuth2 (Optional)

| Variable | Description |
|----------|-------------|
| `TACHYON_OAUTH2_ENABLED` | Enable OAuth2 (`true`/`false`) |
| `TACHYON_GOOGLE_CLIENT_ID` | Google OAuth2 client ID |
| `TACHYON_GOOGLE_CLIENT_SECRET` | Google OAuth2 client secret |
| `TACHYON_GITHUB_CLIENT_ID` | GitHub OAuth2 client ID |
| `TACHYON_GITHUB_CLIENT_SECRET` | GitHub OAuth2 client secret |
| `TACHYON_OAUTH2_REDIRECT_BASE_URL` | OAuth2 redirect URL |

### Rate Limiting

| Variable | Description | Default |
|----------|-------------|---------|
| `TACHYON_RATE_LIMIT_ENABLED` | Enable rate limiting | `true` |
| `REDIS_URL` | Redis URL for distributed rate limiting | `redis://localhost:6379` |

### Security Headers

| Variable | Description | Default |
|----------|-------------|---------|
| `TACHYON_SECURITY_DEVELOPMENT` | Development mode (relaxes CSP) | `true` (set `false` in prod) |
| `TACHYON_SECURITY_CSP_ENABLED` | Enable CSP header | `true` |
| `TACHYON_SECURITY_HSTS_ENABLED` | Enable HSTS | `true` |
| `TACHYON_SECURITY_FRAME_ANCESTORS` | CSP frame-ancestors directive | `'none'` |

### Storage

| Variable | Description | Default |
|----------|-------------|---------|
| `TACHYON_FILES_ROOT` | Local file upload directory | `./uploads` |
| `TACHYON_STORAGE_BACKEND` | Storage backend (`local` or `s3`) | `local` |
| `TACHYON_STORAGE_S3_BUCKET` | S3 bucket name | (empty) |
| `TACHYON_STORAGE_S3_REGION` | S3 region | `us-east-1` |
| `TACHYON_STORAGE_S3_ENDPOINT` | S3-compatible endpoint URL | (empty) |
| `TACHYON_STORAGE_S3_ACCESS_KEY` | S3 access key | (empty) |
| `TACHYON_STORAGE_S3_SECRET_KEY` | S3 secret key | (empty) |

### Logging

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Rust log filter | `info` |
| `TACHYON_LOG_FORMAT` | `text` or `json` | `text` |
| `TACHYON_LOG_LEVEL` | Per-module level syntax | `info` |

### Database Pool Tuning

| Variable | Description | Default |
|----------|-------------|---------|
| `TACHYON_DB_MAX_CONNECTIONS` | Max pool connections | `10` |
| `TACHYON_DB_MIN_CONNECTIONS` | Min idle connections | `2` |
| `TACHYON_DB_ACQUIRE_TIMEOUT` | Connection acquire timeout (s) | `30` |
| `TACHYON_DB_IDLE_TIMEOUT` | Idle connection timeout (s) | `600` |
| `TACHYON_DB_MAX_LIFETIME` | Max connection lifetime (s) | `1800` |
| `TACHYON_SLOW_QUERY_THRESHOLD_MS` | Slow query log threshold (ms) | `100` |

---

## 3. Database Setup

### PostgreSQL Initialization

PostgreSQL is initialized automatically by the `postgres:16-alpine` container on first start. The database, user, and password are created from environment variables in `docker-compose.prod.yml`.

### Run Migrations

Migrations run automatically on container startup. To run manually:

```bash
# Run migrations
docker compose -f docker-compose.yml -f docker-compose.prod.yml \
  run --rm backend tachyon-server migrate

# Or via the CD pipeline (happens automatically on deploy):
# docker compose run --rm backend /app/tachyon-server migrate
```

Migration files are bundled in the Docker image at `/app/migrations` (see `Dockerfile:122`).

### Verify Database Health

```bash
# Check PostgreSQL is ready
docker compose exec postgres pg_isready -U tachyon

# Connect to the database
docker compose exec postgres psql -U tachyon -d tachyon

# List tables
\dt

# Check active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'tachyon';
```

### Admin Account

The admin account is created on first run. Configure via environment variables:

```bash
TACHYON_ADMIN_USERNAME=admin
# TACHYON_ADMIN_PASSWORD=   # Leave empty for auto-generated
# TACHYON_ADMIN_EMAIL=admin@example.com
```

The auto-generated password is printed to container logs on first startup.

---

## 4. Configuration

### Server Configuration

The server reads configuration from environment variables (see `tachyon/crates/server/src/config.rs`). Key configuration sections:

- **Database**: `DATABASE_URL`, pool tuning via `TACHYON_DB_*`
- **JWT**: `TACHYON_JWT_SECRET`, `JWT_EXPIRATION`
- **CORS**: `TACHYON_CORS_ORIGINS`
- **WebSocket**: enabled by default on `/ws`, max 1000 connections, 30s heartbeat
- **Rate Limiting**: per-endpoint limits (auth: 5 req/min, documents: 100 req/min)
- **Search**: `TACHYON_SEARCH_BACKEND` (`postgres` or `tantivy`)

### Configuration Validation

The server validates configuration at startup. Common validation errors:

| Error | Fix |
|-------|-----|
| `JWT secret must be at least 32 characters` | Generate a longer secret |
| `JWT secret must be changed from default value` | Set `TACHYON_JWT_SECRET` |
| `Database URL must start with postgres://` | Fix `DATABASE_URL` format |
| `CORS wildcard origin (*) is not allowed in production` | Set `TACHYON_SECURITY_DEVELOPMENT=false` and specify origins |
| `TLS certificate path required when TLS is enabled` | Set `TACHYON_TLS_CERT_PATH` and `TACHYON_TLS_KEY_PATH` |

### Security Configuration

Disable development mode in production:

```bash
TACHYON_SECURITY_DEVELOPMENT=false
TACHYON_SECURITY_HSTS_ENABLED=true
TACHYON_SECURITY_CSP_ENABLED=true
TACHYON_LOG_FORMAT=json
```

### Storage Configuration

Local storage (default):

```bash
TACHYON_STORAGE_BACKEND=local
TACHYON_FILES_ROOT=/app/data/uploads
```

S3-compatible storage:

```bash
TACHYON_STORAGE_BACKEND=s3
TACHYON_STORAGE_S3_BUCKET=tachyon-uploads
TACHYON_STORAGE_S3_REGION=us-east-1
TACHYON_STORAGE_S3_ACCESS_KEY=<key>
TACHYON_STORAGE_S3_SECRET_KEY=<secret>
```

---

## 5. Docker Deployment

### Initial Deployment

```bash
# 1. Clone the repository
git clone https://github.com/WyattAu/Tachyon.git /opt/tachyon
cd /opt/tachyon/tachyon

# 2. Create environment file
cp .env.example .env
# Edit .env with your production values (see Section 2)

# 3. Log in to container registry
echo "$GHCR_TOKEN" | docker login ghcr.io -u <username> --password-stdin

# 4. Pull images
docker compose -f docker-compose.yml -f docker-compose.prod.yml pull

# 5. Start all services
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 6. Verify services are healthy
docker compose -f docker-compose.yml -f docker-compose.prod.yml ps
```

### Deploy a Specific Version

```bash
cd /opt/tachyon/tachyon

# Set the version to deploy
export VERSION=v1.2.3

# Pull new images
docker compose -f docker-compose.yml -f docker-compose.prod.yml pull backend frontend

# Run migrations (before rolling out new version)
docker compose -f docker-compose.yml -f docker-compose.prod.yml \
  run --rm backend tachyon-server migrate

# Deploy with rolling update
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Restart a Single Service

```bash
# Restart backend only
docker compose -f docker-compose.yml -f docker-compose.prod.yml restart backend

# Restart nginx (e.g., after config change)
docker compose -f docker-compose.yml -f docker-compose.prod.yml restart nginx
```

### View Logs

```bash
# All services
docker compose -f docker-compose.yml -f docker-compose.prod.yml logs -f

# Specific service (last 100 lines)
docker compose -f docker-compose.yml -f docker-compose.prod.yml logs --tail 100 backend

# Since a specific time
docker compose -f docker-compose.yml -f docker-compose.prod.yml logs --since 1h backend
```

### Stop Services

```bash
# Graceful stop
docker compose -f docker-compose.yml -f docker-compose.prod.yml down

# Stop and remove volumes (DESTRUCTIVE)
docker compose -f docker-compose.yml -f docker-compose.prod.yml down -v
```

### Production Compose Aliases

For convenience, create an alias:

```bash
# Add to ~/.bashrc or ~/.zshrc
alias tachyon='docker compose -f /opt/tachyon/tachyon/docker-compose.yml -f /opt/tachyon/tachyon/docker-compose.prod.yml'

# Usage
tachyon ps
tachyon logs -f backend
tachyon up -d
tachyon down
```

---

## 6. Nginx + SSL Setup

### Nginx Configuration

The nginx config is at `tachyon/nginx/nginx.conf`. Key settings:

- Listens on port 8080 internally
- Gzip compression enabled for text, JSON, CSS, JS, fonts, SVG
- Rate limiting zones: global (30 r/s), auth (5 r/m), API (60 r/m)
- `client_max_body_size 50m`
- Security headers: X-Frame-Options DENY, X-Content-Type-Options nosniff, HSTS
- WebSocket proxy at `/ws` with 24h timeout
- `server_tokens off`

### SSL with Let's Encrypt

The production stack includes a certbot sidecar (`tachyon/nginx/certbot-init.sh`) that automatically provisions certificates.

**Step 1:** Set environment variables in `.env`:

```bash
CERTBOT_DOMAIN=tachyon.example.com
CERTBOT_EMAIL=admin@example.com
CERTBOT_STAGING=false
```

**Step 2:** First obtain certificates using staging to avoid rate limits:

```bash
# Set staging mode initially
export CERTBOT_STAGING=true

# Start the stack (certbot will provision staging certs)
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d certbot

# Verify staging cert was obtained
docker compose logs certbot
```

**Step 3:** Switch to production certificates:

```bash
# Remove staging certs and re-provision
docker compose -f docker-compose.yml -f docker-compose.prod.yml down
docker volume rm tachyon_certbot_certs 2>/dev/null || true

export CERTBOT_STAGING=false
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

**Step 4:** Use the SSL-enabled nginx template for production:

```bash
# Copy the SSL template and substitute your domain
sed -e 's/${DOMAIN}/tachyon.example.com/g' \
    -e 's|${SSL_CERT_PATH}|/etc/letsencrypt/live/tachyon.example.com/fullchain.pem|g' \
    -e 's|${SSL_KEY_PATH}|/etc/letsencrypt/live/tachyon.example.com/privkey.pem|g' \
    tachyon/nginx/nginx.ssl.conf.template > tachyon/nginx/nginx.conf
```

**Step 5:** Reload nginx:

```bash
docker compose exec nginx nginx -s reload
```

### Certificate Renewal

Certificates are auto-renewed via cron inside the certbot container. To manually renew:

```bash
# Test renewal (dry run)
docker compose exec certbot certbot renew --dry-run

# Force renewal
docker compose exec certbot certbot renew

# Reload nginx after renewal
docker compose exec nginx nginx -s reload
```

### Generate Development Certificates

```bash
cd tachyon/nginx
./generate-dev-certs.sh
# Creates nginx/certs/localhost.crt and nginx/certs/localhost.key
```

---

## 7. Monitoring Setup

### Stack Overview

The monitoring stack is defined in `tachyon/monitoring/docker-compose.monitoring.yml` and includes:

| Service | Image | Port | Purpose |
|---------|-------|------|---------|
| Prometheus | `prom/prometheus:v2.51.0` | 9090 | Metrics collection |
| Grafana | `grafana/grafana:10.4.0` | 3000 | Dashboards and visualization |
| PostgreSQL Exporter | `promcommunity/postgres-exporter:v0.15.0` | 9187 | Database metrics |
| Node Exporter | `prom/node-exporter:v1.8.0` | 9100 | Host system metrics |
| Nginx Exporter | `nginx/nginx-prometheus-exporter:1.1.0` | 9113 | Proxy metrics |

### Start Monitoring

```bash
cd tachyon/monitoring

# Set Grafana credentials
export GF_ADMIN_USER=admin
export GF_ADMIN_PASSWORD=<strong-password>

# Start the monitoring stack
docker compose -f docker-compose.monitoring.yml up -d
```

The monitoring stack connects to the main `tachyon_tachyon-network` Docker network (external).

### Access Dashboards

| Service | URL | Default Credentials |
|---------|-----|-------------------|
| Grafana | `http://<host>:3000` | `admin` / `admin` (change in production) |
| Prometheus | `http://<host>:9090` | none |

### Pre-built Dashboards

Two dashboards are auto-provisioned:

1. **Tachyon Overview** (`tachyon-overview`): Uptime, HTTP rate, latency percentiles, error rate, DB pool, WebSocket connections
2. **Tachyon Database** (`tachyon-database`): Pool gauges, query duration histogram, slow queries, lock contention, cache hit ratio

### Metrics Endpoints

Tachyon exposes metrics at two endpoints:

| Endpoint | Description |
|----------|-------------|
| `/metrics/prometheus` | Global `metrics` crate recorder |
| `/metrics/app` | Custom Tachyon application metrics |

### Configure Alert Notifications

See `tachyon/monitoring/README.md` for Alertmanager configuration examples (Email, Slack, PagerDuty). Add an `alertmanager` service to `docker-compose.monitoring.yml`:

```yaml
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

### Verify Monitoring

```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {job, health, lastScrape}'

# Check Tachyon metrics
curl http://localhost:9090/api/v1/query?query=up{job="tachyon"} | jq

# Force Grafana dashboard reload
curl -X POST http://admin:<password>@localhost:3000/api/admin/provisioning/dashboards/reload
```

---

## 8. Backup & Restore

### Automated Backups

The `db-backup` service in `docker-compose.prod.yml` runs `tachyon/scripts/backup.sh` on a cron schedule:

- Schedule: **3:00 AM UTC daily** (`0 3 * * *`)
- Retention: **7 days** (configurable via `BACKUP_RETENTION`)
- Format: Custom PostgreSQL (`-Fc`) compressed with gzip
- Storage: Docker volume `backup_data` at `/backups`

### Manual Backup

```bash
# Full backup
docker compose exec db-backup /usr/local/bin/backup.sh

# Schema-only backup
docker compose exec db-backup /usr/local/bin/backup.sh --schema-only

# Data-only backup
docker compose exec db-backup /usr/local/bin/backup.sh --data-only

# Custom output directory
docker compose exec db-backup /usr/local/bin/backup.sh --output /backups --retention 30

# Upload to S3
docker compose exec db-backup /usr/local/bin/backup.sh --upload-s3 my-backup-bucket
```

### Local Backup Script

```bash
# Run backup.sh directly (outside Docker)
./tachyon/scripts/backup.sh \
  --database-url "postgres://tachyon:password@localhost:5432/tachyon" \
  --output ./backups \
  --retention 14 \
  --upload-s3 my-backup-bucket
```

### Backup File Naming

Backups are named: `tachyon_backup_YYYYMMDD_HHMMSS.sql.gz`

### Restore from Backup

```bash
# 1. List available backups
docker compose exec db-backup ls -la /backups/

# 2. Stop the backend to prevent writes during restore
docker compose stop backend

# 3. Restore (custom format with gzip)
docker compose exec postgres \
  pg_restore --clean --if-exists -U tachyon -d tachyon \
  /backups/tachyon_backup_20260101_030000.sql.gz

# 4. If using a plain SQL dump instead:
docker compose exec postgres \
  psql -U tachyon -d tachyon < /backups/tachyon_backup_20260101_030000.sql

# 5. Restart the backend
docker compose start backend

# 6. Verify
docker compose exec postgres psql -U tachyon -d tachyon -c "SELECT count(*) FROM pages;"
```

### Restore from S3

```bash
# Download latest backup from S3
aws s3 ls s3://my-backup-bucket/ --sort date | tail -5
aws s3 cp s3://my-backup-bucket/tachyon_backup_20260101_030000.sql.gz /tmp/

# Copy into the postgres container and restore
docker cp /tmp/tachyon_backup_20260101_030000.sql.gz tachyon-postgres:/tmp/
docker compose exec postgres \
  pg_restore --clean --if-exists -U tachyon -d tachyon \
  /tmp/tachyon_backup_20260101_030000.sql.gz
```

---

## 9. Scaling Guide

### Horizontal Scaling with Backend Replicas

The production compose file deploys **2 backend replicas** by default. Adjust via `docker-compose.prod.yml`:

```yaml
backend:
  deploy:
    replicas: 4  # Increase for higher load
```

### Redis for Distributed Features

Redis is required for:

- **Distributed rate limiting**: All backend instances share rate limit counters
- **Session/cache coherence**: Shared state across replicas

Ensure Redis is configured with a password and persistence:

```bash
# In .env
REDIS_PASSWORD=<strong-password>

# Redis is started with: redis-server --appendonly yes --requirepass <password>
# (configured in docker-compose.prod.yml)
```

The backend connects to Redis via `TACHYON_REDIS_URL`:

```bash
TACHYON_REDIS_URL=redis://:<password>@redis:6379
```

If Redis is unavailable, the server falls back to per-instance in-memory rate limiting.

### Connection Pooling

Scale the database pool based on replica count. Formula: `max_connections = (replicas * per_replica_connections) + safety_margin`

| Replicas | Recommended `TACHYON_DB_MAX_CONNECTIONS` | Recommended `TACHYON_DB_MIN_CONNECTIONS` |
|----------|----------------------------------------|----------------------------------------|
| 1 | 10 | 2 |
| 2 | 20 | 4 |
| 4 | 40 | 8 |
| 8 | 60 | 10 |

Also increase PostgreSQL's `max_connections` to be higher than the total pool size:

```bash
# Add to postgres service environment in docker-compose.prod.yml
POSTGRES_INITDB_ARGS: "--max-connections=200"
```

Or via `postgresql.conf`:

```
max_connections = 200
```

### Nginx Load Balancing

The nginx upstream block in `tachyon/nginx/nginx.conf` uses a single backend server. For multiple replicas, Docker's internal DNS round-robins automatically.

To add explicit upstream servers:

```nginx
upstream backend {
    server backend:3000;
    keepalive 64;  # Increase for more replicas
}
```

### Scaling Checklist

- [ ] Increase `deploy.replicas` for the backend service
- [ ] Adjust `TACHYON_DB_MAX_CONNECTIONS` proportionally
- [ ] Increase PostgreSQL `max_connections`
- [ ] Increase nginx `keepalive` count
- [ ] Ensure Redis is running for distributed rate limiting
- [ ] Monitor connection pool metrics in Grafana
- [ ] Set `TACHYON_DB_MAX_LIFETIME` to prevent stale connections under load

---

## 10. Rollback Procedure

### Via CD Pipeline (Recommended)

Use the GitHub Actions workflow dispatch with rollback enabled:

```bash
# Via GitHub CLI
gh workflow run cd.yml \
  -f environment=production \
  -f version=v1.1.0 \
  -f rollback=true
```

Or trigger manually in GitHub Actions: **Actions > CD > Run workflow** with:
- Target environment: `production` or `staging`
- Version: e.g., `v1.1.0`
- Rollback: `true`

### Manual Rollback

```bash
cd /opt/tachyon/tachyon

# 1. Identify the previous version
docker images | grep tachyon-server

# 2. Set the rollback version
export REGISTRY=ghcr.io
export IMAGE_NAME=wyattau/tachyon
export VERSION=v1.1.0  # The version to roll back to

# 3. Pull the previous version
docker compose -f docker-compose.yml -f docker-compose.prod.yml pull backend frontend

# 4. Check if migration rollback is needed
# Review migration files between versions. If destructive migrations were applied,
# you may need to manually reverse them before restoring the old code.
docker compose -f docker-compose.yml -f docker-compose.prod.yml \
  run --rm backend tachyon-server migrate

# 5. Deploy the previous version
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 6. Verify health
sleep 15
curl -sf https://tachyon.example.com/health | jq .

# 7. Verify API
curl -sf https://tachyon.example.com/api/v1/health | jq .
```

### Database Rollback

If the deployment included schema changes that need to be reversed:

```bash
# 1. Stop the backend
docker compose stop backend

# 2. Restore the pre-deployment backup
docker compose exec postgres \
  pg_restore --clean --if-exists -U tachyon -d tachyon \
  /backups/tachyon_backup_<pre-deployment-timestamp>.sql.gz

# 3. Start the rolled-back version
export VERSION=v1.1.0
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d backend
```

### Rollback Checklist

- [ ] Notify the team about the rollback
- [ ] Verify the target version exists in the registry
- [ ] Check if database migrations need to be reversed
- [ ] Pull the target version images
- [ ] Deploy and verify health checks pass
- [ ] Monitor error rates in Grafana for 15 minutes
- [ ] Create a post-incident review if the rollback was due to an incident

---

## 11. Health Checks

### Container Health Checks

All services have built-in health checks defined in `docker-compose.prod.yml`:

| Service | Check | Interval | Timeout | Retries |
|---------|-------|----------|---------|---------|
| PostgreSQL | `pg_isready -U <user>` | 30s | 10s | 3 |
| Backend | `wget http://localhost:3000/health` | 30s | 10s | 3 |
| Frontend | `wget http://localhost:80/` | 30s | 10s | 3 |
| Nginx | `wget http://localhost:80/health` | 30s | 10s | 3 |

### Manual Health Checks

```bash
# Backend health endpoint
curl -sf http://localhost:3000/health | jq .

# Backend readiness endpoint
curl -sf http://localhost:3000/ready | jq .

# API health
curl -sf http://localhost:3000/api/v1/health | jq .

# Frontend
curl -sf -o /dev/null -w '%{http_code}' http://localhost:80/

# Via nginx (production)
curl -sf https://tachyon.example.com/health | jq .
curl -sf https://tachyon.example.com/api/v1/health | jq .
```

### Check All Container Health

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"
```

### What to Monitor

| Metric | Source | Alert Threshold |
|--------|--------|----------------|
| Error rate (>5% failed requests) | `tachyon_requests_failed` | 5% for 5m |
| p99 latency (>2s) | `http_request_duration_seconds` | 2s for 5m |
| DB pool utilization (>90%) | `tachyon_db_pool_active` / `tachyon_db_pool_max` | 90% for 2m |
| Slow queries (>10 in 5m) | `tachyon_slow_queries_total` | 10 in 5m |
| WebSocket disconnect storm (>100/min) | `tachyon_ws_disconnects_total` | 100/min for 1m |
| Service down | `up{job="tachyon"}` | 0 for 1m |
| DB errors (>50% failure rate) | `tachyon_requests_failed` | 50% for 3m |
| Connections waiting in pool | pool size - active - idle | >5 for 2m |

Full alert rules are defined in `tachyon/monitoring/prometheus/alerts.yml`.

---

## 12. Troubleshooting

### Backend Won't Start

```bash
# Check logs
docker compose logs backend --tail 200

# Common causes:
# 1. Database not ready - check postgres health
docker compose ps postgres
docker compose logs postgres --tail 50

# 2. Invalid configuration - look for validation errors
docker compose logs backend 2>&1 | grep -i "error\|validation"

# 3. Port conflict
ss -tlnp | grep 3000

# 4. Database connection refused
docker compose exec postgres pg_isready -U tachyon
```

### Database Connection Issues

```bash
# Verify PostgreSQL is running and healthy
docker compose exec postgres pg_isready -U tachyon

# Check connection count
docker compose exec postgres psql -U tachyon -d tachyon \
  -c "SELECT count(*), state FROM pg_stat_activity WHERE datname='tachyon' GROUP BY state;"

# Check for lock contention
docker compose exec postgres psql -U tachyon -d tachyon \
  -c "SELECT pid, state, query, wait_event_type, wait_event FROM pg_stat_activity WHERE datname='tachyon' AND state != 'idle';"

# Reset connections if pool is exhausted
docker compose restart backend
```

### High Memory Usage

```bash
# Check container resource usage
docker stats --no-stream

# Check PostgreSQL memory
docker compose exec postgres psql -U tachyon -d tachyon \
  -c "SELECT name, setting, unit FROM pg_settings WHERE name IN ('shared_buffers', 'work_mem', 'effective_cache_size', 'maintenance_work_mem');"

# Restart services to free memory
docker compose restart backend
```

### Nginx Issues

```bash
# Check nginx config syntax
docker compose exec nginx nginx -t

# Check nginx error log
docker compose exec nginx cat /var/log/nginx/error.log | tail -50

# Check nginx access log (with response times)
docker compose exec nginx tail -50 /var/log/nginx/access.log

# Reload nginx config
docker compose exec nginx nginx -s reload
```

### SSL Certificate Issues

```bash
# Check certificate expiry
docker compose exec certbot openssl x509 -in \
  /etc/letsencrypt/live/${CERTBOT_DOMAIN}/fullchain.pem \
  -noout -dates

# Force renewal
docker compose exec certbot certbot renew --force-renewal

# Check certbot logs
docker compose logs certbot --tail 50
```

### Redis Connection Issues

```bash
# Check Redis is running
docker compose exec redis redis-cli ping

# If password-protected
docker compose exec redis redis-cli -a "${REDIS_PASSWORD}" ping

# Check Redis info
docker compose exec redis redis-cli info memory | head -20
docker compose exec redis redis-cli info clients

# Restart Redis
docker compose restart redis
```

### CD Pipeline Failures

```bash
# Check GitHub Actions run status
gh run list --workflow=cd.yml --limit 5

# View a specific run
gh run view <run-id> --log-failed

# Re-run a failed workflow
gh run rerun <run-id>
```

---

## 13. Security Checklist

Complete this checklist before every production deployment:

### Secrets

- [ ] `JWT_SECRET` is a cryptographically random string (64+ chars): `openssl rand -hex 32`
- [ ] `POSTGRES_PASSWORD` is a strong unique password
- [ ] `REDIS_PASSWORD` is set (not empty)
- [ ] No default secrets (`change-me-to-a-random-64-char-string`) are in use
- [ ] Secrets are not committed to git (all in `.env` or GitHub Secrets)

### Authentication

- [ ] `GUEST_LOGIN_ENABLED=false` unless intentionally public
- [ ] `PUBLIC_NOTES_ENABLED=false` unless intentionally public
- [ ] `JWT_EXPIRATION` is set to an appropriate value (default: 3600s)
- [ ] OAuth2 secrets are configured securely (if enabled)

### Network

- [ ] Only ports 80 and 443 are exposed externally
- [ ] Backend port (3000/8080) is not publicly accessible
- [ ] PostgreSQL port (5432) is not publicly accessible
- [ ] Redis port (6379) is not publicly accessible
- [ ] Prometheus (9090) and Grafana (3000) are behind auth/firewall
- [ ] Docker network isolation is in place (`tachyon-network` bridge)

### Headers & TLS

- [ ] `TACHYON_SECURITY_DEVELOPMENT=false`
- [ ] `TACHYON_SECURITY_HSTS_ENABLED=true`
- [ ] `TACHYON_SECURITY_CSP_ENABLED=true`
- [ ] TLS is enabled with valid Let's Encrypt certificates
- [ ] `server_tokens off` in nginx (prevents version disclosure)
- [ ] Security headers are present: `X-Frame-Options`, `X-Content-Type-Options`, `Referrer-Policy`

### CORS

- [ ] CORS origins are set to specific domains (not `*`)
- [ ] `TACHYON_CORS_ORIGINS` matches the actual domain

### Logging & Monitoring

- [ ] `TACHYON_LOG_FORMAT=json` for structured logging
- [ ] Audit logging is enabled
- [ ] Monitoring alerts are configured and firing correctly
- [ ] Access logs are being collected

### Backups

- [ ] Automated backups are running (check cron in `db-backup` container)
- [ ] Backup restoration has been tested
- [ ] S3 off-site backup is configured (if applicable)

### File Permissions

- [ ] Upload directory has restricted permissions
- [ ] Database data volume is not world-readable
- [ ] SSL private key has restricted permissions (chmod 600)

---

## 14. Performance Tuning

### Database Connection Pool

Tune based on load. See scaling guide (Section 9) for replica-based sizing.

```bash
# High-traffic production
TACHYON_DB_MAX_CONNECTIONS=50
TACHYON_DB_MIN_CONNECTIONS=5
TACHYON_DB_ACQUIRE_TIMEOUT=15
TACHYON_DB_IDLE_TIMEOUT=300
TACHYON_DB_MAX_LIFETIME=900
```

### PostgreSQL Tuning

Add to the postgres service environment or a custom `postgresql.conf`:

```bash
# In docker-compose.prod.yml postgres service environment:
POSTGRES_INITDB_ARGS: "--max-connections=200"

# Or mount a custom postgresql.conf with:
shared_buffers = 512MB
effective_cache_size = 1.5GB
work_mem = 16MB
maintenance_work_mem = 256MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
min_wal_size = 1GB
max_wal_size = 4GB
```

### Caching

Enable API response caching (enabled by default):

```bash
TACHYON_CACHE_ENABLED=true
```

Set cache size:

```bash
# Default: 256 MB
# Increase for larger working sets
```

### Nginx Compression

Already configured in `tachyon/nginx/nginx.conf`:

```nginx
gzip on;
gzip_comp_level 6;
gzip_min_length 256;
# Covers: text/*, application/json, application/javascript, application/xml, fonts, SVG
```

### Slow Query Logging

Enable and tune the slow query threshold:

```bash
TACHYON_SLOW_QUERY_THRESHOLD_MS=100
```

Monitor slow queries in Grafana (`tachyon-database` dashboard) or via the alert rule (`HighSlowQueryRate`).

### Log Level

Use per-module log levels for production:

```bash
RUST_LOG=info,tachyon_server=warn,tachyon_database=warn
TACHYON_LOG_FORMAT=json
```

For debugging a specific issue:

```bash
RUST_LOG=tachyon_server=debug,tachyon_database=info
```

---

## 15. Incident Response

### Severity Levels

| Level | Description | Response Time |
|-------|-------------|--------------|
| P1 - Critical | Service down, data loss risk | Immediate (15 min) |
| P2 - High | Degraded performance, partial outage | 30 min |
| P3 - Medium | Non-critical feature broken | 2 hours |
| P4 - Low | Minor issue, no user impact | Next business day |

### P1 - Service Down

```bash
# 1. Confirm the incident
curl -sf https://tachyon.example.com/health || echo "DOWN"

# 2. Check all containers
docker compose -f docker-compose.yml -f docker-compose.prod.yml ps

# 3. Check logs for crash evidence
docker compose logs backend --tail 100 --since 10m

# 4. If backend is crashing, check for:
#    - Database connectivity: docker compose exec postgres pg_isready -U tachyon
#    - OOM kills: dmesg | grep -i oom
#    - Disk space: df -h

# 5. Restart services
docker compose -f docker-compose.yml -f docker-compose.prod.yml restart backend

# 6. If restart doesn't fix it, check for recent deployments
git log --oneline -5

# 7. If caused by a bad deploy, initiate rollback (see Section 10)

# 8. If database corruption, restore from backup (see Section 8)
```

### P2 - Degraded Performance

```bash
# 1. Check error rate
curl http://localhost:9090/api/v1/query?query='rate(tachyon_requests_failed[5m])/rate(tachyon_requests_total[5m])'

# 2. Check latency
curl http://localhost:9090/api/v1/query?query='histogram_quantile(0.99,sum(rate(http_request_duration_seconds_bucket[5m]))by(le))'

# 3. Check database pool
docker compose exec postgres psql -U tachyon -d tachyon \
  -c "SELECT count(*) FROM pg_stat_activity WHERE datname='tachyon' AND state='active';"

# 4. Check for long-running queries
docker compose exec postgres psql -U tachyon -d tachyon \
  -c "SELECT pid, now()-query_start AS duration, query FROM pg_stat_activity WHERE datname='tachyon' AND state='active' ORDER BY duration DESC LIMIT 10;"

# 5. Check Redis
docker compose exec redis redis-cli info clients
docker compose exec redis redis-cli info memory | grep used_memory_human

# 6. Check system resources
docker stats --no-stream
```

### P3 - Feature Broken

```bash
# 1. Identify the affected endpoint from logs
docker compose logs backend --since 30m | grep -i "error\|panic\|failed"

# 2. Check for recent config changes
docker compose exec backend env | grep TACHYON

# 3. Check API response directly
curl -v https://tachyon.example.com/api/v1/<endpoint>

# 4. If related to a specific deployment, consider targeted rollback
```

### Communication

```bash
# Notify via Slack (if webhook is configured)
curl -s -X POST "${SLACK_WEBHOOK_URL}" \
  -H 'Content-Type: application/json' \
  -d '{"text":"[INCIDENT] Tachyon production issue - investigating"}'

# Post-resolution
curl -s -X POST "${SLACK_WEBHOOK_URL}" \
  -H 'Content-Type: application/json' \
  -d '{"text":"[RESOLVED] Tachyon production issue - service restored"}'
```

### Post-Incident

1. Create an incident timeline
2. Identify root cause
3. Write up findings
4. Create follow-up tickets for prevention
5. Update this runbook with new troubleshooting steps if applicable

---

## Quick Reference

### Common Commands

```bash
# Start
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Stop
docker compose -f docker-compose.yml -f docker-compose.prod.yml down

# Logs
docker compose -f docker-compose.yml -f docker-compose.prod.yml logs -f <service>

# Status
docker compose -f docker-compose.yml -f docker-compose.prod.yml ps

# Pull latest
docker compose -f docker-compose.yml -f docker-compose.prod.yml pull

# Migrate
docker compose -f docker-compose.yml -f docker-compose.prod.yml run --rm backend tachyon-server migrate

# Backup
docker compose exec db-backup /usr/local/bin/backup.sh

# Health check
curl -sf https://tachyon.example.com/health | jq .
```

### File References

| File | Purpose |
|------|---------|
| `tachyon/.env.example` | Environment variable template |
| `tachyon/Dockerfile` | Backend multi-stage Docker build |
| `tachyon/docker-compose.yml` | Development compose configuration |
| `tachyon/docker-compose.prod.yml` | Production compose overrides |
| `tachyon/nginx/nginx.conf` | Nginx reverse proxy configuration |
| `tachyon/nginx/nginx.ssl.conf.template` | SSL-enabled nginx template |
| `tachyon/nginx/certbot-init.sh` | Let's Encrypt certificate provisioning |
| `tachyon/nginx/generate-dev-certs.sh` | Self-signed dev certificate generator |
| `tachyon/scripts/backup.sh` | Database backup script |
| `tachyon/monitoring/docker-compose.monitoring.yml` | Monitoring stack |
| `tachyon/monitoring/prometheus/prometheus.yml` | Prometheus scrape config |
| `tachyon/monitoring/prometheus/alerts.yml` | Alert rules |
| `tachyon/monitoring/grafana/` | Grafana dashboards and provisioning |
| `.github/workflows/cd.yml` | CI/CD pipeline |
| `tachyon/crates/server/src/config.rs` | Server configuration source |
