#!/bin/bash
set -euo pipefail

usage() {
    cat >&2 <<'EOF'
Usage: backup.sh [OPTIONS]

Create a PostgreSQL backup with compression and optional S3 upload.

Options:
  --schema-only        Backup schema only (no data)
  --data-only          Backup data only (no schema)
  --no-compress        Skip gzip compression
  --retention N        Keep last N backups (default: 7)
  --output DIR         Output directory (default: $BACKUP_DIR or ./backups)
  --upload-s3 BUCKET   Upload to S3 bucket (requires aws CLI)
  --database-url URL   PostgreSQL connection URL
  --help, -h           Show this help

Environment:
  DATABASE_URL         PostgreSQL connection URL
  POSTGRES_HOST        Database host (fallback)
  POSTGRES_PORT        Database port (default: 5432)
  POSTGRES_USER        Database user (fallback)
  POSTGRES_PASSWORD    Database password (fallback)
  POSTGRES_DB          Database name (fallback)
  BACKUP_DIR           Output directory (default: ./backups)
  BACKUP_RETENTION     Number of backups to keep (default: 7)
  S3_BUCKET            S3 bucket for upload

Exit codes:
  0  Success
  1  pg_dump failed
  2  Compression failed
  3  S3 upload failed
EOF
    exit 0
}

log() {
    echo "[backup] $(date +%Y-%m-%dT%H:%M:%S%z) $*" >&2
}

MODE="full"
COMPRESS=true
RETENTION="${BACKUP_RETENTION:-7}"
OUTPUT_DIR="${BACKUP_DIR:-./backups}"
S3_BUCKET="${S3_BUCKET:-}"
DB_URL="${DATABASE_URL:-}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --schema-only)   MODE="schema" ;;
        --data-only)     MODE="data" ;;
        --no-compress)   COMPRESS=false ;;
        --retention)     RETENTION="$2"; shift ;;
        --output)        OUTPUT_DIR="$2"; shift ;;
        --upload-s3)     S3_BUCKET="$2"; shift ;;
        --database-url)  DB_URL="$2"; shift ;;
        --help|-h)       usage ;;
        *)               log "ERROR: Unknown option: $1"; exit 1 ;;
    esac
    shift
done

if [ -z "$DB_URL" ]; then
    DB_HOST="${POSTGRES_HOST:-localhost}"
    DB_PORT="${POSTGRES_PORT:-5432}"
    DB_USER="${POSTGRES_USER:-tachyon}"
    DB_PASS="${POSTGRES_PASSWORD:-tachyon}"
    DB_NAME="${POSTGRES_DB:-tachyon}"
    DB_URL="postgresql://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
fi

if ! command -v pg_dump &>/dev/null; then
    log "ERROR: pg_dump not found. Install PostgreSQL client."
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RAW_NAME="tachyon_backup_${TIMESTAMP}.sql"
RAW_PATH="${OUTPUT_DIR}/${RAW_NAME}"
if [ "$COMPRESS" = true ]; then
    FINAL_NAME="${RAW_NAME}.gz"
else
    FINAL_NAME="$RAW_NAME"
fi

PG_ARGS=()
case "$MODE" in
    schema) PG_ARGS+=(--schema-only) ;;
    data)   PG_ARGS+=(--data-only) ;;
esac
PG_ARGS+=(-Fc -f "$RAW_PATH" "$DB_URL")

log "Starting ${MODE} backup to ${FINAL_NAME}..."

if ! pg_dump "${PG_ARGS[@]}" 2>&1 | while IFS= read -r line; do log "pg_dump: $line"; done; then
    log "ERROR: pg_dump failed"
    rm -f "$RAW_PATH"
    exit 1
fi

if [ ! -f "$RAW_PATH" ] || [ ! -s "$RAW_PATH" ]; then
    log "ERROR: pg_dump produced empty output"
    exit 1
fi

if [ "$COMPRESS" = true ]; then
    log "Compressing backup..."
    if ! gzip -f "$RAW_PATH"; then
        log "ERROR: compression failed"
        exit 2
    fi
fi

FINAL_PATH="${OUTPUT_DIR}/${FINAL_NAME}"
log "Backup created: ${FINAL_PATH} ($(du -h "$FINAL_PATH" | cut -f1))"

log "Applying retention policy (keep=${RETENTION})..."
REMOVED=0
ls -1t "${OUTPUT_DIR}"/tachyon_backup_* 2>/dev/null | tail -n +$((RETENTION + 1)) | while read -r old; do
    rm -f "$old"
    REMOVED=$((REMOVED + 1))
    log "Removed old backup: $old"
done

if [ -n "$S3_BUCKET" ]; then
    if command -v aws &>/dev/null; then
        log "Uploading to s3://${S3_BUCKET}/${FINAL_NAME}..."
        if ! aws s3 cp "$FINAL_PATH" "s3://${S3_BUCKET}/${FINAL_NAME}"; then
            log "ERROR: S3 upload failed"
            exit 3
        fi
        log "Upload complete"
    else
        log "WARN: aws CLI not found, skipping S3 upload"
    fi
fi

log "Backup completed successfully"
exit 0
