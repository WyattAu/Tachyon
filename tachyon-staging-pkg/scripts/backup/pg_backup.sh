#!/usr/bin/env bash
# Tachyon PostgreSQL Backup Script
#
# Creates compressed pg_dump backups with rotation and optional S3 upload.
#
# Usage: pg_backup.sh [--no-upload] [--no-rotate]
#
# Environment:
#   DATABASE_URL   PostgreSQL connection string
#   BACKUP_DIR     Local backup directory
#   S3_BUCKET      S3-compatible bucket (optional)
#   RETENTION_DAYS Days of backups to keep (default: 30)
#   LOG_FILE       Log file path (default: BACKUP_DIR/backup.log)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

BACKUP_DIR="${BACKUP_DIR:-${SCRIPT_DIR}/../../backups}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
LOG_FILE="${LOG_FILE:-${BACKUP_DIR}/backup.log}"
S3_BUCKET="${S3_BUCKET:-}"
S3_PATH="${S3_PATH:-backups/postgres}"
DATABASE_URL="${DATABASE_URL:-postgres://tachyon:tachyon@localhost:5432/tachyon}"

NO_UPLOAD=0
NO_ROTATE=0

for arg in "$@"; do
    case "$arg" in
        --no-upload) NO_UPLOAD=1 ;;
        --no-rotate) NO_ROTATE=1 ;;
    esac
done

# ── Logging ────────────────────────────────────────────────────────────────────

_log() {
    local level="$1"; shift
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] $*"
    echo "$msg"
    mkdir -p "$(dirname "$LOG_FILE")"
    echo "$msg" >> "$LOG_FILE"
}

log_info()  { _log "INFO"  "$@"; }
log_ok()    { _log "OK"    "$@"; }
log_warn()  { _log "WARN"  "$@"; }
log_error() { _log "ERROR" "$@"; }

die() {
    log_error "$@"
    exit 1
}

# ── Helpers ────────────────────────────────────────────────────────────────────

check_deps() {
    for cmd in pg_dump; do
        command -v "$cmd" &>/dev/null || die "Missing dependency: $cmd"
    done
}

parse_db_url() {
    local url="${DATABASE_URL#*://}"
    DB_USER="${url%%:*}"
    url="${url#*:}"
    DB_PASS="${url%%@*}"
    url="${url#*@}"
    DB_HOST="${url%%:*}"
    url="${url#*:}"
    DB_PORT="${url%%/*}"
    DB_NAME="${url#*/}"
}

# ── Backup ─────────────────────────────────────────────────────────────────────

do_backup() {
    check_deps
    parse_db_url

    mkdir -p "$BACKUP_DIR"

    local ts
    ts=$(date '+%Y%m%d-%H%M%S')
    local fname="tachyon_${DB_NAME}_${ts}.dump.gz"
    local fpath="${BACKUP_DIR}/${fname}"

    log_info "Starting backup of ${DB_NAME} on ${DB_HOST}:${DB_PORT}"

    if ! PGPASSWORD="$DB_PASS" pg_dump \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="$DB_NAME" \
        --format=custom \
        --compress=9 \
        --no-password \
        --verbose \
        2>>"$LOG_FILE" | gzip > "$fpath"; then
        rm -f "$fpath"
        die "pg_dump failed for ${DB_NAME}"
    fi

    if [ ! -s "$fpath" ]; then
        rm -f "$fpath"
        die "Backup file is empty: $fpath"
    fi

    local fsize
    fsize=$(du -h "$fpath" | cut -f1)
    log_ok "Backup created: ${fname} (${fsize})"

    # Verify backup integrity
    log_info "Verifying backup integrity..."
    if gzip -t "$fpath" 2>/dev/null; then
        log_ok "Backup integrity verified"
    else
        rm -f "$fpath"
        die "Backup failed integrity check: $fpath"
    fi

    # S3 upload
    if [ "$NO_UPLOAD" -eq 0 ] && [ -n "${AWS_ACCESS_KEY_ID:-}" ] && [ -n "$S3_BUCKET" ]; then
        if command -v aws &>/dev/null; then
            log_info "Uploading to s3://${S3_BUCKET}/${S3_PATH}/${fname}..."
            if aws s3 cp "$fpath" "s3://${S3_BUCKET}/${S3_PATH}/${fname}" 2>>"$LOG_FILE"; then
                log_ok "S3 upload complete"
            else
                log_warn "S3 upload failed"
            fi
        else
            log_warn "aws CLI not found; skipping S3 upload"
        fi
    fi

    log_ok "Backup complete: ${fname}"
}

# ── Rotation ───────────────────────────────────────────────────────────────────

do_rotate() {
    if [ "$NO_ROTATE" -eq 1 ]; then
        return 0
    fi

    log_info "Rotating backups older than ${RETENTION_DAYS} days..."

    local count=0
    while IFS= read -r -d '' file; do
        rm -f "$file"
        log_info "Removed: $(basename "$file")"
        count=$((count + 1))
    done < <(find "$BACKUP_DIR" -maxdepth 1 -name 'tachyon_*.dump.gz' -type f -mtime +"$RETENTION_DAYS" -print0 2>/dev/null)

    log_ok "Rotation complete: removed ${count} old backup(s)"
}

# ── Main ───────────────────────────────────────────────────────────────────────

main() {
    log_info "=== Tachyon PostgreSQL Backup ==="
    do_backup
    do_rotate
    log_info "=== Backup run finished ==="
}

main
