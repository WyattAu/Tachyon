#!/bin/bash
set -euo pipefail

usage() {
    cat >&2 <<'EOF'
Usage: restore.sh [OPTIONS] --file BACKUP_FILE

Restore a PostgreSQL backup.

Options:
  --file, -f PATH       Path to backup file (required)
  --schema-only         Restore schema only (no data)
  --list, -l            List backup contents
  --force, -y           Skip confirmation prompt
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
  0  Success
  1  Validation or restore failed
  2  Restore failed
EOF
    exit 0
}

log() {
    echo "[restore] $(date +%Y-%m-%dT%H:%M:%S%z) $*" >&2
}

BACKUP_FILE=""
DB_URL="${DATABASE_URL:-}"
SCHEMA_ONLY=false
LIST_CONTENTS=false
FORCE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --file|-f)        BACKUP_FILE="$2"; shift ;;
        --schema-only)    SCHEMA_ONLY=true ;;
        --list|-l)        LIST_CONTENTS=true ;;
        --force|-y)       FORCE=true ;;
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

if [ ! -s "$BACKUP_FILE" ]; then
    log "ERROR: Backup file is empty: $BACKUP_FILE"
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

if [ "$LIST_CONTENTS" = true ]; then
    log "Listing contents of $BACKUP_FILE..."
    if [[ "$BACKUP_FILE" == *.gz ]]; then
        gunzip -c "$BACKUP_FILE" | pg_restore --list
    else
        pg_restore --list "$BACKUP_FILE"
    fi
    exit 0
fi

if [ "$FORCE" = false ] && [ "$SCHEMA_ONLY" = false ]; then
    echo "WARNING: This will overwrite existing data in the database!" >&2
    echo "  Backup file: $BACKUP_FILE" >&2
    echo "  Database:    $DB_URL" >&2
    read -r -p "Type 'yes' to confirm: " confirm
    if [ "$confirm" != "yes" ]; then
        log "Restore cancelled by user"
        exit 0
    fi
fi

RESTORE_ARGS=()
if [ "$SCHEMA_ONLY" = true ]; then
    RESTORE_ARGS+=(--schema-only)
fi
RESTORE_ARGS+=(--no-owner --no-privileges -d "$DB_URL")

log "Restoring backup from $BACKUP_FILE..."
if [[ "$BACKUP_FILE" == *.gz ]]; then
    if ! gunzip -c "$BACKUP_FILE" | pg_restore "${RESTORE_ARGS[@]}"; then
        log "ERROR: restore failed"
        exit 2
    fi
else
    if ! pg_restore "${RESTORE_ARGS[@]}" "$BACKUP_FILE"; then
        log "ERROR: restore failed"
        exit 2
    fi
fi

log "Restore completed successfully"
exit 0
