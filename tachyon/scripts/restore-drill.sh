#!/usr/bin/env bash
# Tachyon backup restore drill: verify a backup can actually be restored.
set -euo pipefail

CONTAINER="tachyon-postgres"
TEST_DB="tachyon_restore_test"
LATEST=$(ls -t /opt/tachyon/backups/postgres/tachyon_*.sql.gz | head -1)

echo "[$(date -Iseconds)] Restore drill using: ${LATEST}"

# Fresh test database
docker exec "${CONTAINER}" psql -U tachyon -d postgres -c "DROP DATABASE IF EXISTS ${TEST_DB};" >/dev/null
docker exec "${CONTAINER}" psql -U tachyon -d postgres -c "CREATE DATABASE ${TEST_DB};" >/dev/null

# Restore (plain SQL dump, gunzip on host, pipe in)
gzip -dc "${LATEST}" | docker exec -i "${CONTAINER}" psql -U tachyon -d "${TEST_DB}" -q >/dev/null 2>&1

# Verify tables exist
TABLES=$(docker exec "${CONTAINER}" psql -U tachyon -d "${TEST_DB}" -t -A -c \
  "SELECT count(*) FROM information_schema.tables WHERE table_schema='public';")
ROWS=$(docker exec "${CONTAINER}" psql -U tachyon -d "${TEST_DB}" -t -A -c \
  "SELECT coalesce(sum(n_live_tup),0) FROM pg_stat_user_tables;")

echo "[$(date -Iseconds)] Restored ${TABLES} tables, ~${ROWS} rows"

# Cleanup
docker exec "${CONTAINER}" psql -U tachyon -d postgres -c "DROP DATABASE ${TEST_DB};" >/dev/null

if [ "${TABLES}" -gt 0 ]; then
  echo "[$(date -Iseconds)] RESTORE DRILL PASSED"
  exit 0
else
  echo "[$(date -Iseconds)] RESTORE DRILL FAILED: 0 tables restored"
  exit 1
fi
