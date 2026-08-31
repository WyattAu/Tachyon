#!/bin/bash
set -euo pipefail

usage() {
    cat >&2 <<'EOF'
Usage: verify-backup.sh --file BACKUP_FILE [OPTIONS]

Verify a backup by restoring it to a temporary database and comparing schemas.

Options:
  --file, -f PATH       Path to backup file (required)
  --database-url URL    PostgreSQL connection URL
  --help, -h            Show this help

Environment:
  DATABASE_URL          PostgreSQL connection URL
  POSTGRES_HOST         Database host (fallback)
  POSTGRES_PORT         Database port (default: 5432)
  POSTGRES_USER         Database user (fallback)
  POSTGRES_PASSWORD     Database password (fallback)
  POSTGRES_DB           Database name (fallback)

Exit codes:
  0  Verification passed (schemas match)
  1  Verification failed or error
EOF
    exit 0
}

log() {
    echo "[verify] $(date +%Y-%m-%dT%H:%M:%S%z) $*" >&2
}

BACKUP_FILE=""
DB_URL="${DATABASE_URL:-}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --file|-f)        BACKUP_FILE="$2"; shift ;;
        --database-url)   DB_URL="$2"; shift ;;
        --help|-h)        usage ;;
        *)                log "ERROR: Unknown option: $1"; exit 1 ;;
    esac
    shift
done

if [ -z "$BACKUP_FILE" ]; then
    log "ERROR: --file is required"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    log "ERROR: Backup file not found: $BACKUP_FILE"
    exit 1
fi

if [ -z "$DB_URL" ]; then
    DB_HOST="${POSTGRES_HOST:-localhost}"
    DB_PORT="${POSTGRES_PORT:-5432}"
    DB_USER="${POSTGRES_USER:-tachyon}"
    DB_PASS="${POSTGRES_PASSWORD:-tachyon}"
    DB_NAME="${POSTGRES_DB:-tachyon}"
    DB_URL="postgresql://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
fi

if ! command -v pg_dump &>/dev/null || ! command -v pg_restore &>/dev/null; then
    log "ERROR: pg_dump and pg_restore are required"
    exit 1
fi

TEMP_DB="tachyon_verify_${RANDOM}_$$"
TEMP_URL="${DB_URL%/*}/${TEMP_DB}"

cleanup() {
    log "Dropping temporary database: ${TEMP_DB}..."
    psql "$DB_URL" -c "DROP DATABASE IF EXISTS \"${TEMP_DB}\"" 2>/dev/null || true
}
trap cleanup EXIT

log "Creating temporary database: ${TEMP_DB}..."
psql "$DB_URL" -c "CREATE DATABASE \"${TEMP_DB}\"" >/dev/null 2>&1

log "Restoring backup to temporary database..."
if [[ "$BACKUP_FILE" == *.gz ]]; then
    gunzip -c "$BACKUP_FILE" | pg_restore --no-owner --no-privileges -d "$TEMP_URL" 2>/dev/null
else
    pg_restore --no-owner --no-privileges -d "$TEMP_URL" "$BACKUP_FILE" 2>/dev/null
fi

log "Comparing schemas..."

CURRENT_SCHEMA=$(pg_dump --schema-only "$DB_URL" 2>/dev/null \
    | grep -v '^--' \
    | grep -v '^$' \
    | grep -v '^SET ' \
    | grep -v '^SELECT ' \
    | sort)

BACKUP_SCHEMA=$(pg_dump --schema-only "$TEMP_URL" 2>/dev/null \
    | grep -v '^--' \
    | grep -v '^$' \
    | grep -v '^SET ' \
    | grep -v '^SELECT ' \
    | sort)

if [ "$CURRENT_SCHEMA" = "$BACKUP_SCHEMA" ]; then
    log "PASS: Schemas match - no drift detected"
    exit 0
else
    log "WARN: Schema drift detected"
    echo "" >&2
    echo "=== Schema Differences ===" >&2
    diff <(echo "$CURRENT_SCHEMA") <(echo "$BACKUP_SCHEMA") >&2 || true
    echo "" >&2
    log "FAIL: Schemas do not match"
    exit 1
fi
