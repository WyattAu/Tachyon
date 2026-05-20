# Performance Degradation Runbook

## Severity: Medium to High

Performance degradation manifests as increased latency, higher error rates, or resource exhaustion. The goal is to identify the bottleneck and restore acceptable response times.

## Detection Methods

1. **P99 latency > 500ms**: Prometheus alert on `histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[1m]))`
2. **Error rate > 1%**: Alert on 5xx response rate
3. **Response time alerts**: User-reported slowness or frontend timeout errors
4. **Resource metrics**: CPU > 80%, memory > 80%, disk I/O saturation
5. **Database pool exhaustion**: `db_pool_connections_active` near `db_pool_connections_max`
6. **Cache hit rate drop**: `cache_hits_total / cache_requests_total` declining

## Response Procedure

### 1. Identify the Bottleneck (10 min)

```bash
# Check overall system health
curl -s http://localhost:8080/health | jq '.'

# Check Prometheus metrics for overview
curl -s http://localhost:8080/metrics/prometheus | grep tachyon_

# Check application-level metrics
# - Request rate, latency, error counts
# - Database pool utilization
# - Cache hit rates
```

### 2. Categorize the Issue

#### A. Database Bottleneck

```bash
# Check active queries
psql -c "SELECT pid, state, query_start, now() - query_start AS duration, query
         FROM pg_stat_activity
         WHERE state != 'idle'
         ORDER BY duration DESC LIMIT 20;"

# Check for blocking locks
psql -c "SELECT blocked.pid, blocked.query, blocking.pid, blocking.query
         FROM pg_stat_activity blocked
         JOIN pg_locks bl ON bl.pid = blocked.pid
         JOIN pg_locks kl ON kl.locktype = bl.locktype AND kl.database IS NOT DISTINCT FROM bl.database
         AND kl.relation IS NOT DISTINCT FROM bl.relation AND kl.page IS NOT DISTINCT FROM bl.page
         AND kl.tuple IS NOT DISTINCT FROM bl.tuple AND kl.pid != bl.pid
         JOIN pg_stat_activity blocking ON blocking.pid = kl.pid;"

# Check table/index sizes
psql -c "SELECT relname, pg_size_pretty(pg_total_relation_size(relid)) as size
         FROM pg_stat_user_tables ORDER BY pg_total_relation_size(relid) DESC LIMIT 10;"

# Check missing indexes
psql -c "SELECT schemaname, tablename, attname, n_distinct, correlation
         FROM pg_stats WHERE n_distinct > 100 AND correlation < 0.1 ORDER BY abs(correlation);"
```

**Mitigations**:
- Kill long-running queries: `SELECT pg_cancel_backend(pid);`
- Add missing indexes for slow queries
- Run `VACUUM ANALYZE` on bloated tables
- Increase `shared_buffers` or `work_mem` if memory allows

#### B. Search Index Bottleneck

```bash
# Check if Tantivy index is healthy
curl -s http://localhost:8080/health | jq '.checks.tantivy'

# Rebuild the search index if degraded
curl -X POST http://localhost:8080/api/v1/search/reindex \
  -H "Authorization: Bearer <token>"
```

#### C. Memory/CPU Exhaustion

```bash
# Check process resource usage
ps aux | grep tachyon-server
top -p $(pgrep -f tachyon-server)

# Check for memory leaks (growth over time)
# Review heap allocation patterns
```

**Mitigations**:
- Restart the application to reclaim memory
- Scale horizontally (add more instances behind a load balancer)
- Profile with `perf` or `tracing` spans

#### D. Connection Pool Exhaustion

```bash
# Check pool metrics
curl -s http://localhost:8080/metrics/prometheus | grep db_pool

# Identify which endpoints are consuming connections
# Look for queries holding connections open
psql -c "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"
```

**Mitigations**:
- Increase pool size in configuration
- Add connection timeout settings
- Identify and fix slow queries that hold connections
- Implement request queuing for non-critical endpoints

### 3. Verify Recovery

```bash
# Check P99 latency is back to normal
# (via Grafana dashboard or direct Prometheus query)

# Run health check
curl -s http://localhost:8080/health | jq '.status'

# Test critical endpoints
time curl -s http://localhost:8080/api/v1/documents?page=1&page_size=20 -o /dev/null -w "%{http_code} %{time_total}s\n"
time curl -s http://localhost:8080/api/v1/search?q=test -o /dev/null -w "%{http_code} %{time_total}s\n"
```

### 4. Post-Incident

- Document the root cause and resolution
- Add specific alerting for the identified bottleneck
- Update query optimization or add indexes as needed
- Review and adjust pool/connection settings

## Prevention Measures

- Configure database connection pool with appropriate max size and timeouts
- Set up automated `VACUUM` and `ANALYZE` jobs
- Monitor query performance with `pg_stat_statements`
- Implement request-level timeouts on all endpoints
- Use caching for frequently accessed, rarely changing data
- Profile the application regularly under load
- Run load tests before deploying performance-sensitive changes
