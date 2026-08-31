# Failed Database Migration Runbook

## Severity: High

A failed database migration can prevent the application from starting or cause data inconsistency. Migrations are run using `sqlx` at application startup and can also be run manually.

## Detection Methods

1. **Application startup failure**: Application exits with `Failed to connect to test database` or migration error
2. **Schema mismatch errors**: Runtime `sqlx::Error` with column/table not found messages
3. **Manual check**: `sqlx migrate info` shows pending or failed migrations
4. **Deployment logs**: CI/CD pipeline fails at the migration step

## Response Procedure

### 1. Identify the Failed Migration (5 min)

```bash
# Check migration status
sqlx migrate info --database-url $DATABASE_URL

# Look for "applied" vs "pending" migrations
# The migration after the last "applied" one is the failure point
```

### 2. Examine the Migration File

```bash
# Find the migration files
ls -la migrations/

# Review the SQL in the failed migration
cat migrations/<timestamp>_<description>.sql
```

### 3. Assess the State

- **Partial application**: The migration file was partially executed (some statements succeeded, some failed)
- **Not applied**: The migration was never run
- **Already applied with errors**: The migration was recorded but the actual schema is incorrect

```bash
# Check current schema state
psql -h $DB_HOST -U $DB_USER -d $DB_NAME -c "\dt"
psql -h $DB_HOST -U $DB_USER -d $DB_NAME -c "\d+ <table_name>"
```

### 4. Recovery Options

#### Option A: Fix and Re-run (preferred)

If the migration was never applied or can be safely re-run:

```bash
# Fix the migration SQL if needed
# Then re-run
sqlx migrate run --database-url $DATABASE_URL
```

#### Option B: Manual Rollback

If the migration was partially applied:

```bash
# 1. Connect to the database
psql -h $DB_HOST -U $DB_USER -d $DB_NAME

# 2. Manually undo the partial changes
#    (reverse the SQL statements from the migration file)

# 3. Remove the migration record from __sqlx_migrations
DELETE FROM __sqlx_migrations WHERE version = '<timestamp>';

# 4. Fix the migration file and re-run
sqlx migrate run --database-url $DATABASE_URL
```

#### Option C: Restore from Backup (last resort)

```bash
# Stop the application
# Restore from the most recent pre-migration backup
pg_restore -h $DB_HOST -U $DB_USER -d $DB_NAME backup.dump

# Verify migration state
sqlx migrate info --database-url $DATABASE_URL

# Re-run migrations
sqlx migrate run --database-url $DATABASE_URL
```

### 5. Verify

```bash
# Confirm all migrations applied
sqlx migrate info --database-url $DATABASE_URL

# Run health check
curl -s http://localhost:8080/health | jq '.checks.database'

# Test critical operations
curl -s http://localhost:8080/api/v1/documents | head
```

### 6. Post-Incident

- Document the root cause
- Fix the migration file to be idempotent (use `IF NOT EXISTS`, `IF EXISTS`)
- Add rollback migration for the fixed migration
- Update migration testing in CI pipeline
- Review all pending migrations for similar issues

## Prevention Measures

- Always test migrations against a copy of production data before deploying
- Make migrations idempotent using `IF EXISTS` / `IF NOT EXISTS`
- Create both up and down migration files
- Back up the database before running migrations in production
- Use transactional DDL (`BEGIN` / `COMMIT`) in migration files
- Lock the schema during deployments to prevent concurrent migration runs
- Run `sqlx migrate info` as a pre-deployment check
