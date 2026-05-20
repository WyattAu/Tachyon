# Upgrade Guide: v10.x to v11.0.0

**Date:** 2026-05-14 | **Classification:** Operations Documentation

## Summary of Changes

v11.0.0 includes security hardening (HTML sanitization via ammonia, `unwrap()` audit across the codebase), operational maturity (readiness checks at `/ready`, structured JSON logging, migration rollback support), performance improvements (Tantivy full-text search fusion with PostgreSQL tsvector, response caching on read-heavy endpoints, load testing infrastructure), testing enhancements (middleware chain integration tests, enforced coverage thresholds), and documentation (API reference, error code catalog, operational runbooks, Grafana monitoring dashboards).

## Breaking Changes

None. All changes are backward-compatible additions.

## New Configuration Variables

| Variable | Type | Default | Description |
|---|---|---|---|
| `TACHYON_LOG_FORMAT` | `json` or `text` | `text` | Structured log output format. Set to `json` in production for log aggregation. |
| `TACHYON_LOG_FILTER` | tracing filter string | `info` | Controls log verbosity. Accepts standard `tracing_subscriber` filter syntax (e.g., `info,tachyon::search=debug`). |

## New Dependencies

- `ammonia` -- HTML sanitization library. Already present as a transitive dependency; promoted to direct.

## Database Migrations

No new migrations required. Schema is unchanged.

## API Changes

- **`/ready`** now returns structured JSON with per-dependency status checks (`database`, `smtp`, `redis`). Previous behavior returned a plain `200 OK`.
- **Response caching** applied to `GET /api/v1/documents`, `GET /api/v1/search`, and `GET /api/v1/spaces` with a 60-second TTL. Cache keys are derived from the full request path and query string. `Cache-Control: max-age=60` header is set on responses.
- **Tantivy search fusion** in `GET /api/v1/search`. Results from Tantivy (BM25) and PostgreSQL tsvector are merged using reciprocal rank fusion (RRF). No changes to request or response schema.

## Deployment Steps

1. Pull the new Docker image or build from source.
2. Set `TACHYON_LOG_FORMAT=json` for production (optional but recommended).
3. Set `TACHYON_LOG_FILTER` as needed (default: `info`).
4. No database migrations needed.
5. Verify the `/ready` endpoint returns `{"status":"ok",...}`.
6. Review Grafana dashboards under `monitoring/grafana/provisioning/`.

## Rollback

Standard Docker rollback:

```
docker-compose down
docker tag tachyon:11.0.0 tachyon:rollback
docker-compose up -d
```

No database rollback is required since the schema is unchanged.

## Verification

- `curl /health` returns `200`.
- `curl /ready` returns `200` with `{"status":"ok","checks":{"database":"ok","smtp":"ok","redis":"ok"}}`.
- All existing API endpoints return expected responses without schema changes.
- No error spikes in application logs post-deployment.
- Grafana dashboards reflect normal metric baselines within one scrape interval.
