---
title: Incident Response Runbook
description: Operational runbook for Tachyon Server incidents
date: 2026-05-21
category: operations
order: 1
---

# Incident Response Runbook

This runbook covers diagnosis and remediation for common Tachyon Server incidents.

## High Error Rate (>5% 5xx)

### Symptoms

- Elevated 5xx responses in load balancer / API gateway metrics
- Alert triggered on error rate threshold
- Users reporting Internal Server Error pages

### Diagnosis

```bash
# Check recent error logs
docker compose logs app --since 5m | grep -i "error\|5xx\|panic" | tail -50

# Check PostgreSQL connectivity
docker compose exec app sh -c 'pg_isready -h $DATABASE_HOST -p $DATABASE_PORT'

# Check resource limits
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"

# Check rate limiter saturation
curl -s http://localhost:8080/health | jq '.rate_limiter'
```

### Remediation

1. If PostgreSQL is unreachable: verify `DATABASE_URL`, restart the database container, check network connectivity
2. If CPU/memory is saturated: scale horizontally (add replicas), increase resource limits in `docker-compose.yml`
3. If rate limiter is saturated: increase `rate_limit.max_requests` in config, check for abusive clients in access logs
4. If a specific endpoint is failing: check for recent deployments that may have introduced a regression

### Prevention

- Set up alerting on 5xx rate > 1% with 2-minute window
- Run load tests before releases (`just load-test`)
- Use circuit breakers for external service calls
- Implement graceful degradation for non-critical features

## Database Connection Pool Exhaustion

### Symptoms

- "connection pool exhausted" or "timeout acquiring connection" in logs
- Requests hanging then returning 503
- Database connections at `max_connections` limit

### Diagnosis

```bash
# Check active PostgreSQL connections
docker compose exec db psql -U tachyon -c "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"

# Check max_connections in PostgreSQL
docker compose exec db psql -U tachyon -c "SHOW max_connections;"

# Check slow queries (> 1 second)
docker compose exec db psql -U tachyon -c "SELECT query, state, duration FROM pg_stat_activity WHERE state = 'active' AND now() - query_start > interval '1 second';"

# Check pool config in Tachyon
grep -r "max_connections\|pool_size" config/ docker-compose*.yml
```

### Remediation

1. Identify and kill long-running queries: `SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'active' AND now() - query_start > interval '30 seconds';`
2. Increase `max_connections` in `config.rs` or environment variable `DATABASE_MAX_CONNECTIONS`
3. Scale pool size: set `DATABASE_MAX_CONNECTIONS` to `2 * CPU_cores + number_of_disks`
4. Add query timeouts: set `STATEMENT_TIMEOUT=10000` (10s) at the database level

### Prevention

- Monitor `pg_stat_activity` connection counts with Prometheus/Grafana
- Set statement timeout in PostgreSQL: `ALTER DATABASE tachyon SET statement_timeout = '10s';`
- Use connection pooling (PgBouncer) for production deployments
- Review slow queries in `pg_stat_statements` regularly

## WebSocket Disconnections

### Symptoms

- Clients reporting real-time sync stops working
- Collaborative editing sessions dropping
- Spikes in WebSocket reconnect attempts

### Diagnosis

```bash
# Check WebSocket connection count
curl -s http://localhost:8080/metrics | grep websocket_connections

# Check heartbeat timeouts in logs
docker compose logs app --since 10m | grep -i "heartbeat\|ws_close\|ws_timeout" | tail -30

# Check CRDT sync backlog
curl -s http://localhost:8080/metrics | grep crdt_sync_backlog

# Check connection manager limits
docker compose logs app --since 10m | grep -i "connection_limit\|max_connections" | tail -20
```

### Remediation

1. If heartbeat timeouts: increase `WS_HEARTBEAT_INTERVAL_MS` and `WS_HEARTBEAT_TIMEOUT_MS` in config
2. If CRDT sync backlog is growing: reduce document size, batch updates, increase sync worker count
3. If connection limit hit: increase `WS_MAX_CONNECTIONS` or scale horizontally with sticky sessions
4. If network instability: check load balancer WebSocket timeout settings (nginx: `proxy_read_timeout`)

### Prevention

- Configure nginx `proxy_read_timeout` and `proxy_send_timeout` to exceed heartbeat interval
- Implement exponential backoff on client reconnects
- Monitor WebSocket connection metrics with dashboards
- Load test WebSocket capacity before releases

## Memory Leak

### Symptoms

- Steadily increasing memory usage over time
- OOM kills in container logs
- Performance degradation as memory pressure increases

### Diagnosis

```bash
# Monitor memory growth over time
docker stats --no-stream app --format "{{.MemUsage}}"

# Check CRDT document cache size
curl -s http://localhost:8080/metrics | grep -i "crdt_cache\|document_cache"

# Check rate limiter store entries
curl -s http://localhost:8080/metrics | grep -i "rate_limit_store"

# Check API response cache
curl -s http://localhost:8080/metrics | grep -i "response_cache"

# Run heap profiling (if enabled)
curl -s http://localhost:8080/debug/heap > heap.prof
```

### Remediation

1. CRDT document cache: reduce `CRDT_CACHE_MAX_ENTRIES` or `CRDT_CACHE_TTL_SECONDS` in config
2. Rate limiter store: ensure cleanup interval is configured (`RATE_LIMIT_CLEANUP_INTERVAL_SECS`)
3. API response cache: reduce `API_CACHE_TTL_SECONDS` or `API_CACHE_MAX_ENTRIES`
4. Emergency: restart the affected container (`docker compose restart app`)
5. After restart, monitor memory growth rate to confirm leak is ongoing

### Prevention

- Set memory limits in Docker Compose (`mem_limit`) and configure OOM kill policy
- Run continuous memory profiling in staging
- Use LRU eviction with size limits on all in-memory caches
- Add memory usage metrics and alerting at 80% threshold

## JWT Validation Failures After Rotation

### Symptoms

- 401 Unauthorized responses for valid tokens
- Spike in authentication failures after key rotation
- Users unable to access authenticated endpoints

### Diagnosis

```bash
# Check current JWT secrets configuration
docker compose logs app --since 10m | grep -i "jwt\|kid\|validation" | tail -30

# Verify TACHYON_JWT_SECRETS env var order (oldest first)
docker compose exec app env | grep TACHYON_JWT

# Check rotation_enabled flag
docker compose exec app env | grep ROTATION_ENABLED

# Check kid header tracking
curl -s http://localhost:8080/metrics | grep -i "jwt_kid"
```

### Remediation

1. Verify `TACHYON_JWT_SECRETS` lists secrets in order: oldest first, newest last
2. Ensure all previously issued tokens have their signing key still in the secrets list
3. If `rotation_enabled` is false, enable it: `ROTATION_ENABLED=true`
4. If kid header mismatch: check that token issuer and Tachyon agree on key identifiers
5. Emergency: temporarily add the old secret back to `TACHYON_JWT_SECRETS`

### Prevention

- Use automated key rotation with overlap period (keep old keys for at least token TTL)
- Log key rotation events with timestamps
- Monitor authentication failure rate as a key metric
- Test key rotation in staging before production

## SSG Build Failure

### Symptoms

- Documentation site not updating after content changes
- Build pipeline failing at SSG step
- Missing or incomplete generated HTML files

### Diagnosis

```bash
# Run SSG build with verbose logging
RUST_LOG=debug cargo run -p tachyon-ssg-cli -- build --input ./docs --output ./site

# Check markdown frontmatter validity
find ./docs -name "*.md" -exec sh -c 'echo "=== $1 ===" && head -5 "$1"' _ {} \;

# Check output directory permissions
ls -la ./site/

# Validate site config
cat ./docs/site.toml
```

### Remediation

1. Invalid frontmatter: ensure all `.md` files have valid YAML frontmatter with required fields (`title`, `date`)
2. Template rendering errors: check error message for missing fields, verify `SiteConfig` has all required fields
3. Output directory permissions: `chmod 755 ./site/` or fix ownership
4. Missing dependencies: run `cargo build -p tachyon-ssg` to check compilation errors

### Prevention

- Validate frontmatter in CI before build (`just lint-docs`)
- Run SSG build as a CI step with non-zero exit on warnings
- Pin markdown renderer version to avoid surprise formatting changes
- Keep a known-good docs snapshot for regression testing

## Rollback Procedures

### Git Revert

```bash
# Identify the bad commit
git log --oneline -10

# Revert the specific commit
git revert <commit-sha>

# Or revert the last deployment tag
git revert <tag-name>

# Push and trigger deployment
git push origin main
```

### Database Migration Rollback

```bash
# Check current migration version
docker compose exec app sh -c 'cat migrations/.version'

# Rollback to previous migration
docker compose exec app sh -c 'sqlx migrate revert'

# Verify rollback
docker compose exec db psql -U tachyon -c "\\dt"

# If manual rollback needed
docker compose exec db psql -U tachyon -f /migrations/backups/<timestamp>.sql
```

### Configuration Rollback

```bash
# Check current config version in git
git log --oneline docker-compose.yml config/ -5

# Restore previous config
git checkout <previous-sha> -- docker-compose.yml config/

# Restart services with old config
docker compose down && docker compose up -d

# Verify health
curl -s http://localhost:8080/health | jq '.'
```
