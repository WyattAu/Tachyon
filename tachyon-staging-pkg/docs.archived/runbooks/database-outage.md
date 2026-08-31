# Database Outage Runbook

## Severity: Critical

A PostgreSQL database outage affects all data-dependent operations: document CRUD, authentication, search indexing, billing, and session management.

## Detection Methods

1. **Health endpoint**: `GET /health` returns `{"status": "unhealthy"}` with `"database": {"status": "error", ...}`
2. **Readiness endpoint**: `GET /ready` returns 503 with `{"status": "not ready", "error": "database unreachable"}`
3. **Prometheus alerts**: `DATABASE_ERROR` error code spikes in response metrics
4. **Application logs**: `ERROR Database error: connection refused` / `pool exhausted`
5. **Connection pool metrics**: `db_pool_connections_active` at max, connection timeout errors

## Response Procedure

### 1. Confirm the Outage (5 min)

```bash
# Check if the application can reach the database
curl -s http://localhost:8080/health | jq '.checks.database'

# Check PostgreSQL directly
pg_isready -h $DB_HOST -p $DB_PORT -d $DB_NAME

# Check PostgreSQL logs
docker logs tachyon-db --tail 100  # or journalctl -u postgresql
```

### 2. Identify the Root Cause

- **Connection exhaustion**: Too many active connections exceeding `max_connections`
- **Disk full**: PostgreSQL cannot write to WAL or data directory
- **OOM kill**: PostgreSQL process was killed by the OS
- **Network**: Firewall, DNS, or network partition between app and database
- **Lock contention**: Long-running transaction blocking queries
- **Crash**: PostgreSQL process crashed (check for `postmaster.pid`)

```bash
# Check active connections
psql -h $DB_HOST -U $DB_USER -d $DB_NAME -c "SELECT count(*) FROM pg_stat_activity;"

# Check for blocking locks
psql -c "SELECT pid, state, query, wait_event FROM pg_stat_activity WHERE wait_event IS NOT NULL;"

# Check disk space
df -h /var/lib/postgresql/data

# Check PostgreSQL process
ps aux | grep postgres
```

### 3. Mitigate

- **Connection exhaustion**: Terminate idle connections
  ```bash
  psql -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle' AND query_start < now() - interval '10 minutes';"
  ```

- **Disk full**: Free space or expand volume
  ```bash
  # Find large tables
  psql -c "SELECT relname, pg_size_pretty(pg_total_relation_size(relid)) FROM pg_stat_user_tables ORDER BY pg_total_relation_size(relid) DESC LIMIT 10;"
  ```

- **OOM kill**: Increase memory or reduce `shared_buffers`/`work_mem`

### 4. Restore Service

```bash
# Restart PostgreSQL if crashed
docker restart tachyon-db  # or systemctl restart postgresql

# Verify connectivity
curl -s http://localhost:8080/health | jq '.checks.database'

# Verify readiness
curl -s http://localhost:8080/ready
```

### 5. Verify Data Integrity

```bash
# Check for corrupted indexes
psql -c "SELECT indexrelname, indisvalid FROM pg_index WHERE NOT indisvalid;"

# Rebuild any invalid indexes
psql -c "REINDEX INDEX CONCURRENTLY <index_name>;"

# Run database migrations to ensure schema is current
sqlx migrate run
```

### 6. Post-Incident

- Review application logs for any data inconsistency errors
- Check search index is in sync (POST `/api/v1/search/reindex` if needed)
- Notify stakeholders of resolution
- File incident report

## Prevention Measures

- Configure connection pool with appropriate `max_connections` and timeout values
- Set up automated disk space monitoring with alerts at 80% and 90% thresholds
- Enable PostgreSQL `auto_explain` for slow query logging
- Configure `wal_level = replica` for point-in-time recovery
- Set up regular `pg_dump` backups (see backup recovery guide)
- Configure `shared_preload_libraries` for monitoring extensions
- Test failover procedures quarterly
