# Tachyon Deployment Runbook

This runbook describes the supported deployment path for Tachyon. The repository contains a reproducible native staging deployer and Docker assets for production; production TLS, DNS, provider credentials, and CI secrets must be supplied by the operator.

## 1. CachyOS staging

### Prerequisites

- SSH key access to the staging host
- `ssh`, `scp`, `tar`, and `curl` on the deployment machine
- Rust/Cargo on the staging host
- A staging environment file on the host at:
  `~/.config/tachyon-staging.env`

The environment file must contain, at minimum:

The database pool settings are applied by the server when it initializes the pool. For concurrent staging validation, the checked-in systemd unit additionally sets `TACHYON_DB_MAX_CONNECTIONS=40`, `TACHYON_DB_MIN_CONNECTIONS=5`, and `TACHYON_DB_IDLE_TIMEOUT_SECS=600`; override those only after checking PostgreSQL capacity.

```dotenv
DATABASE_URL=postgres://tachyon:<password>@127.0.0.1:<port>/tachyon
TACHYON_JWT_SECRETS=<64-character-random-secret>
TACHYON_HOST=0.0.0.0
TACHYON_PORT=8082
TACHYON_SECURITY_DEVELOPMENT=false
TACHYON_CORS_ORIGINS=http://<staging-host>:8082
```

Keep this file mode `600`. Do not commit it or print its contents.

### Deploy

From the repository root:

```bash
./deploy.sh
```

Optional parameters:

```bash
TACHYON_STAGING_HOST=wyatt@192.168.1.191 \
TACHYON_STAGING_URL=http://192.168.1.191:8082 \
./deploy.sh
```

The deployer:

1. Creates a revision directory under `~/tachyon-staging/releases/`.
2. Packages the Tachyon workspace and its local `salting`, `tokenkit`, and `cryptkit` dependencies.
3. Builds `tachyon-server` in release mode on CachyOS.
4. Switches `~/tachyon-staging/current` atomically to the new revision.
5. Installs/enables the user-level `tachyon-staging.service`.
6. Restarts the service and verifies `/health`, `/ready`, and `/metrics/prometheus`.

The checked-in unit is `scripts/tachyon-staging.service`; it allows the server’s graceful shutdown window to complete before systemd force-kills it.

### Verify and operate

```bash
ssh 192.168.1.191 systemctl --user status tachyon-staging.service --no-pager
ssh 192.168.1.191 journalctl --user -u tachyon-staging.service -n 100 --no-pager
curl -fsS http://192.168.1.191:8082/health
curl -fsS http://192.168.1.191:8082/ready
curl -fsS http://192.168.1.191:8082/metrics/prometheus >/dev/null
```

To roll back to a previously built revision, use the checked-in helper. First inspect the available revisions:

```bash
ssh 192.168.1.191 'ls -1dt ~/tachyon-staging/releases/*/ | head'
./scripts/tachyon-staging-rollback.sh 192.168.1.191 <revision>
```

The rollback helper validates the binary, switches `current`, restarts the user service, and verifies health/readiness/metrics. Do not delete a revision until the replacement has passed health checks.

## 2. Backups and restore

The PostgreSQL scripts are:

```bash
cd tachyon
./scripts/backup.sh --database-url "$DATABASE_URL" --output ./backups
./scripts/verify-backup.sh --file ./backups/<backup-file> --database-url "$DATABASE_URL"
./scripts/restore.sh --file ./backups/<backup-file> --database-url "$DATABASE_URL"
```

`restore.sh` requires an explicit confirmation unless `--force` is provided. Run verification against an isolated temporary database or an isolated staging database before relying on a backup for recovery.

A production deployment is not ready until:

- automated backups run on a schedule;
- at least one backup is copied to independent storage;
- a restore has been completed and checked; and
- the restore and rollback commands are recorded for the operator team.

## 3. Monitoring

Tachyon exposes:

- `/metrics/prometheus` for the global Prometheus recorder;
- `/metrics/app` for application metrics.

The monitoring configuration is in `tachyon/monitoring/`:

```bash
cd tachyon/monitoring
docker compose -f docker-compose.monitoring.yml up -d
# Add --profile metrics-only only when the optional nginx metrics sidecar is present.
```

Before production use, set non-default Grafana credentials and verify that Prometheus targets resolve to the actual Tachyon service. Review `prometheus.yml` if the service name, network, or port differs from the Docker defaults.

Required alerts include service availability, request errors, latency, database-pool pressure, slow queries, and WebSocket disconnect spikes. Alert delivery itself must be tested; a configured rule without a tested notification path is not operational monitoring.

## 4. TLS, DNS, and CORS

Production must run behind Nginx or Caddy with a real DNS name and HTTPS certificate. Configure:

```dotenv
TACHYON_SECURITY_DEVELOPMENT=false
TACHYON_SECURITY_HSTS_ENABLED=true
TACHYON_SECURITY_CSP_ENABLED=true
TACHYON_CORS_ORIGINS=https://app.example.com
```

Use exact comma-separated origins. `*` is rejected by configuration validation when development mode is disabled. Do not expose the application, PostgreSQL, Redis, Prometheus, or Grafana ports directly to the public internet.

Let's Encrypt requires a real public domain, DNS pointing to the host, ports 80/443 reachable, and an operator email. Those prerequisites are not available for the private-LAN staging host and therefore cannot be completed here.

## 5. Validation gates

Run the existing k6 tests against the deployed URL:

```bash
cd tachyon
BASE_URL=http://192.168.1.191:8082 k6 run load-tests/k6/smoke.js
BASE_URL=http://192.168.1.191:8082 k6 run load-tests/k6/LoadTest.js
BASE_URL=http://192.168.1.191:8082 k6 run load-tests/k6/RateLimitTest.js
# Wait for the configured rate-limit window before a subsequent WebSocket run.
BASE_URL=ws://192.168.1.191:8082 k6 run --vus 20 --duration 20s load-tests/k6/WebSocketStress.js
```

The existing reports under `tachyon/reports/` are LAN/staging evidence only. A production release additionally requires:

- a 24-hour soak with resource measurements;
- a higher-concurrency stress run;
- multi-client, room-scoped WebSocket protocol tests;
- OWASP ZAP against the real HTTPS surface;
- authorization review across REST, GraphQL, uploads, plugins, and WebSockets;
- backup/restore and rollback rehearsals; and
- CI/CD deployment from a tagged commit.

## 6. Production release checklist

- [ ] Domain and DNS configured
- [ ] HTTPS certificate obtained and renewal tested
- [ ] Explicit CORS origins configured
- [ ] Strong secrets stored outside Git
- [ ] PostgreSQL backup and restore verified
- [ ] Redis configured for multi-instance features
- [ ] Prometheus/Grafana dashboards and alerts tested
- [ ] 24-hour soak and stress reports retained
- [ ] OWASP ZAP and manual authorization review complete
- [ ] CI/CD push-to-deploy path verified
- [ ] Rollback rehearsal complete
- [ ] Release notes identify known limitations and scaffolding
