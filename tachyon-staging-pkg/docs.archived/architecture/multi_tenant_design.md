# Multi-Tenant SaaS Architecture Design (G.5)

## 1. Overview

Transform the single-tenant Tachyon instance into a multi-tenant SaaS platform. Each tenant (organization) operates in isolation with dedicated configuration, quotas, and branding while sharing the underlying infrastructure.

Core requirements:

- **Isolation**: Tenant data must not be accessible across tenant boundaries
- **Configuration**: Per-tenant feature flags, branding, and rate limits
- **Metering**: Usage tracking for billing and quota enforcement
- **Administration**: Portal for tenant lifecycle management

## 2. Isolation Strategy

### Schema-Level Isolation

Shared PostgreSQL instance with separate schemas per tenant. Each tenant's schema contains the full table structure (documents, users, spaces, permissions).

```
postgres
  ├── public (shared: tenants table, platform config)
  ├── tenant_001 (tenant A: full table set)
  ├── tenant_002 (tenant B: full table set)
  └── tenant_NNN (tenant N: full table set)
```

### Why Schema Over Database

| Approach | Isolation | Cost | Migration | Connection Mgmt |
|----------|-----------|------|-----------|-----------------|
| Separate databases | Strongest | Highest | Per-DB | Separate pools |
| Schema isolation | Strong | Medium | Single migration | Shared pool with switching |
| Row-level (tenant_id) | Weakest | Lowest | Single migration | Single pool |

Schema isolation provides strong data isolation without the operational overhead of managing separate databases per tenant.

### Connection Pooling and Schema Switching

- PgBouncer manages the connection pool
- On each request, the application sets the search path before executing queries:
  ```sql
  SET search_path TO tenant_001, public;
  ```
- Tenant resolution from request (subdomain, header, or JWT claim) occurs in middleware
- Connection is returned to the pool with search_path reset

### Tenant Resolution

Priority order for tenant identification from an incoming request:

1. **Subdomain**: `{tenant}.tachyon.app` -- primary method
2. **Authorization header**: JWT contains `tenant_id` claim -- API clients
3. **Custom domain**: Tenant-provided domain mapped via DNS CNAME -- enterprise tier

## 3. Tenant Configuration

### Feature Flags

Per-tenant feature flags stored in `public.tenant_config`:

```sql
CREATE TABLE public.tenant_config (
  tenant_id      UUID PRIMARY KEY REFERENCES public.tenants(id),
  key            TEXT NOT NULL,
  value          JSONB NOT NULL,
  updated_at     TIMESTAMPTZ DEFAULT now(),
  UNIQUE (tenant_id, key)
);
```

Feature flags control:

- Collaboration (real-time editing, comments, mentions)
- Spaces (shared workspaces, permissions)
- API access (rate limits, API key generation)
- Export formats (PDF, DOCX, HTML)
- Admin features (audit log, SSO)

### Custom Branding

| Setting | Storage | Applied At |
|---------|---------|------------|
| Logo | S3 object URL | Server-rendered HTML, API response |
| Primary/secondary colors | tenant_config | CSS variables injection |
| Custom domain | DNS + TLS cert | Edge router |
| Favicon | S3 object URL | HTML meta tag |
| Custom login page HTML | tenant_config | Server-rendered |

### Rate Limits and Quotas

- Rate limits enforced per tenant, not per user (tenant-level aggregate)
- Quotas tracked in real-time with periodic reconciliation:
  - Storage: sum of document sizes in tenant schema
  - Users: count of active user records
  - API calls: rolling window counter in Redis
- Plan-based limits defined in `public.plans` table

### Plan-Based Feature Gating

```
Free        -> 3 users, 100 documents, 100MB storage
Pro         -> 25 users, 10000 documents, 10GB storage
Business    -> 250 users, unlimited documents, 100GB storage
Enterprise  -> Unlimited users, unlimited documents, custom storage
```

Feature access is a function of plan + feature flags. Enterprise tenants may have custom feature combinations.

## 4. Usage Metering

### Metrics Collected

| Metric | Collection Method | Granularity |
|--------|------------------|-------------|
| API requests | Middleware counter | Per-request, aggregated hourly |
| Storage used | PostgreSQL `pg_relation_size` + S3 stats | Daily |
| Active users | Last-seen timestamp on user records | Daily |
| Documents created | INSERT trigger on documents table | Daily |
| WebSocket connections | Connection count at interval | Hourly |

### Metering Pipeline

```
[Request] -> [Middleware] -> [Redis INCR tenant:{id}:api:{hour}]
    -> [Cron Job (hourly)] -> [Aggregate to tenant_usage table]
    -> [Cron Job (monthly)] -> [Billing aggregation]
```

### Billing Aggregation

- Monthly snapshot of usage metrics per tenant
- Compared against plan quotas to determine overages
- Usage report available to tenant admins in the admin portal
- Webhook notification to billing provider (Stripe) on plan threshold breach

### Storage Calculation

- Document content: `SUM(LENGTH(content))` from documents table
- Attachments: S3 bucket size per tenant prefix (`/{tenant_id}/attachments/`)
- Total storage = document content + attachments, calculated daily

## 5. Admin Portal

### Tenant Management

- CRUD operations for tenants (create, read, update, suspend, delete)
- Tenant suspension blocks all access without data deletion
- Tenant deletion triggers schema drop + S3 prefix deletion (soft-delete with 30-day retention)

### Plan Management

- Plan CRUD with feature matrix definition
- Tenant plan assignment and change
- Proration calculation on plan changes
- Grandfathering of legacy plans

### Usage Dashboards

- Per-tenant usage view with current vs. quota
- Platform-wide usage aggregation (total tenants, total storage, API volume)
- Trend graphs (daily, weekly, monthly)
- Export to CSV for accounting

### Support Integration

- Tenant-scoped support tickets
- Admin notes on tenant records
- Activity log for tenant configuration changes

## 6. Data Migration

### Single-Tenant Import

Existing single-tenant Tachyon instances can be imported as the first tenant:

1. Create tenant record in `public.tenants`
2. Create tenant schema (`tenant_{id}`)
3. Run schema migration to create tables in new schema
4. Copy data from existing `public` schema to tenant schema
5. Verify row counts and data integrity
6. Update application connection to use new schema

### Migration Tooling

- CLI tool: `tachyon-tenant-migrate --from public --to tenant_001`
- Schema migrations applied to all tenant schemas via:
  ```bash
  psql -c "SELECT 'migrate_' || schema_name FROM information_schema.schemata WHERE schema_name LIKE 'tenant_%'"
  ```
- Each migration is idempotent and can be re-run safely
- Migration dry-run mode for validation before execution

## 7. Implementation Priority

| Phase | Scope | Duration |
|-------|-------|----------|
| 1 | Schema isolation: schema-per-tenant, connection pooling, tenant resolution middleware | 2 weeks |
| 2 | Tenant configuration: feature flags, branding, plan model, rate limiting | 2 weeks |
| 3 | Usage metering: collection pipeline, aggregation, storage calculation | 2 weeks |
| 4 | Admin portal: tenant CRUD, plan management, usage dashboards | 2 weeks |

**Total estimated effort: 8 weeks** (1 senior backend engineer)
