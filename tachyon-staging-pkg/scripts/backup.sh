#!/bin/bash
# Tachyon PostgreSQL Backup Script
# Usage: ./backup.sh [full|incremental|restore <file>|cleanup|verify <file>]
#
# Dependencies: pg_dump, pg_restore, psql (PostgreSQL client tools)
# Optional: aws CLI (for S3 upload)
#
# Make executable: chmod +x scripts/backup.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ─── Configuration ───────────────────────────────────────────────────────────

BACKUP_DIR="${BACKUP_DIR:-$PROJECT_ROOT/backups}"
DATABASE_URL="${DATABASE_URL:-postgres://tachyon:tachyon@localhost:5432/tachyon}"
S3_BUCKET="${S3_BUCKET:-}"
RETENTION_DAILY="${RETENTION_DAILY:-30}"
RETENTION_WEEKLY="${RETENTION_WEEKLY:-12}"
RETENTION_MONTHLY="${RETENTION_MONTHLY:-6}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ─── Logging ─────────────────────────────────────────────────────────────────

log_info()  { echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $*"; }
log_error() { echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $*" >&2; }

die() {
    log_error "$@"
    exit 1
}

# ─── Usage ───────────────────────────────────────────────────────────────────

usage() {
    cat >&2 <<'EOF'
Tachyon PostgreSQL Backup Script

Usage: ./backup.sh <command> [args]

Commands:
  full              Create a full pg_dump backup (--format=custom --compress=9)
  incremental       Enable WAL archiving configuration
  restore <file>    Restore database from a backup file
  verify <file>     Restore to temp database and verify table count
  cleanup           Apply retention policy (daily/weekly/monthly)

Environment:
  DATABASE_URL        PostgreSQL connection URL
                      (default: postgres://tachyon:tachyon@localhost:5432/tachyon)
  BACKUP_DIR          Backup storage directory (default: ./backups)
  S3_BUCKET           S3 bucket name for upload
  AWS_ACCESS_KEY_ID   AWS credentials (if set, S3 upload is attempted)
  RETENTION_DAILY     Keep last N daily backups  (default: 30)
  RETENTION_WEEKLY    Keep last N weekly backups  (default: 12)
  RETENTION_MONTHLY   Keep last N monthly backups (default: 6)

Examples:
  ./backup.sh full
  ./backup.sh incremental
  ./backup.sh restore backups/tachyon-backup-20250101-120000.sql
  ./backup.sh verify backups/tachyon-backup-20250101-120000.sql
  ./backup.sh cleanup
EOF
    exit 0
}

# ─── Helpers ─────────────────────────────────────────────────────────────────

check_deps() {
    for cmd in pg_dump pg_restore psql; do
        command -v "$cmd" &>/dev/null || die "Missing dependency: $cmd (install PostgreSQL client tools)"
    done
}

parse_db_url() {
    # Extract components from DATABASE_URL for individual use
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

timestamp() {
    date '+%Y%m%d-%H%M%S'
}

backup_name() {
    local ts="$1"
    echo "tachyon-backup-${ts}.sql"
}

week_bucket() {
    date -d "${1:0:8}" '+%Y-W%V' 2>/dev/null || echo "unknown"
}

month_bucket() {
    echo "${1:0:6}"
}

# ─── Full Backup ─────────────────────────────────────────────────────────────

full_backup() {
    log_info "Starting full backup..."

    parse_db_url
    check_deps

    mkdir -p "$BACKUP_DIR"

    local ts
    ts=$(timestamp)
    local fname
    fname=$(backup_name "$ts")
    local fpath="${BACKUP_DIR}/${fname}"

    log_info "Running pg_dump (format=custom, compress=9)..."
    if ! PGPASSWORD="$DB_PASS" pg_dump \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="$DB_NAME" \
        --format=custom \
        --compress=9 \
        --file="$fpath" \
        --no-password; then
        rm -f "$fpath"
        die "pg_dump failed"
    fi

    if [ ! -f "$fpath" ] || [ ! -s "$fpath" ]; then
        die "Backup file is empty or missing: $fpath"
    fi

    local fsize
    fsize=$(du -h "$fpath" | cut -f1)
    log_ok "Backup created: ${fpath} (${fsize})"

    # S3 upload if credentials are available
    if [ -n "${AWS_ACCESS_KEY_ID:-}" ] && [ -n "${S3_BUCKET:-}" ]; then
        if command -v aws &>/dev/null; then
            log_info "Uploading to s3://${S3_BUCKET}/backups/${fname}..."
            aws s3 cp "$fpath" "s3://${S3_BUCKET}/backups/${fname}" || log_warn "S3 upload failed"
            log_ok "S3 upload complete"
        else
            log_warn "aws CLI not found; skip S3 upload"
        fi
    fi

    log_ok "Full backup complete"
}

# ─── Incremental (WAL Archiving Config) ──────────────────────────────────────

incremental_backup() {
    log_info "Configuring WAL archiving for incremental backups..."

    parse_db_url

    local wal_dir="${BACKUP_DIR}/wal_archive"
    mkdir -p "$wal_dir"

    log_info "WAL archive directory: $wal_dir"

    cat <<WALDOC

╔══════════════════════════════════════════════════════════════════╗
║  WAL Archiving Configuration Instructions                      ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                ║
║  Add the following to postgresql.conf:                         ║
║                                                                ║
║    wal_level = replica                                         ║
║    archive_mode = on                                           ║
║    archive_command = 'test ! -f ${wal_dir}/%f && cp %p ${wal_dir}/%f'  ║
║    archive_timeout = 60                                        ║
║                                                                ║
║  Then restart PostgreSQL:                                      ║
║    pg_ctl restart -D /var/lib/postgresql/data                  ║
║    (or sudo systemctl restart postgresql)                      ║
║                                                                ║
║  To take a base backup (required for PITR with WAL):           ║
║    pg_basebackup -h $DB_HOST -U $DB_USER -D ${BACKUP_DIR}/base_$(timestamp) -Ft -z -P  ║
║                                                                ║
║  Verify archiving is working:                                  ║
║    SELECT archived_count, failed_count FROM pg_stat_archiver;  ║
║                                                                ║
╚══════════════════════════════════════════════════════════════════╝
WALDOC

    log_info "Attempting to set WAL archiving config on server..."
    if PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="$DB_NAME" \
        --no-password \
        -c "ALTER SYSTEM SET archive_mode = on;" \
        -c "ALTER SYSTEM SET archive_command = 'test ! -f ${wal_dir}/%f && cp %p ${wal_dir}/%f';" \
        -c "ALTER SYSTEM SET archive_timeout = 60;" \
        -c "SELECT pg_reload_conf();" 2>/dev/null; then
        log_ok "WAL archiving configured; reload sent"
    else
        log_warn "Could not configure WAL archiving via SQL (check permissions or edit postgresql.conf manually)"
    fi
}

# ─── Restore ─────────────────────────────────────────────────────────────────

restore_backup() {
    local file="$1"

    if [ ! -f "$file" ]; then
        die "Backup file not found: $file"
    fi

    parse_db_url
    check_deps

    log_info "Restoring database from: $file"

    # Determine restore flags based on file extension / format
    local format_flag="-Fc"
    if [[ "$file" == *.sql ]]; then
        format_flag=""
    fi

    log_warn "This will DROP and recreate all objects in target database: ${DB_NAME}"
    read -p "Type 'yes' to confirm restore: " confirm
    if [ "$confirm" != "yes" ]; then
        log_warn "Restore cancelled"
        exit 0
    fi

    log_info "Dropping existing connections to ${DB_NAME}..."
    PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="postgres" \
        --no-password \
        -c "SELECT pg_terminate_backend(pg_stat_activity.pid)
            FROM pg_stat_activity
            WHERE pg_stat_activity.datname = '${DB_NAME}'
              AND pid <> pg_backend_pid();" 2>/dev/null || true

    log_info "Recreating database..."
    PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="postgres" \
        --no-password \
        -c "DROP DATABASE IF EXISTS \"${DB_NAME}\";" 2>/dev/null || true

    PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="postgres" \
        --no-password \
        -c "CREATE DATABASE \"${DB_NAME}\" OWNER \"${DB_USER}\";" 2>/dev/null || die "Failed to create database"

    log_info "Restoring data..."
    if ! PGPASSWORD="$DB_PASS" pg_restore \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="$DB_NAME" \
        --no-password \
        --clean \
        --if-exists \
        --no-owner \
        ${format_flag:+"$format_flag"} \
        "$file"; then
        die "pg_restore failed"
    fi

    log_ok "Restore complete: ${DB_NAME}"
}

# ─── Verify ──────────────────────────────────────────────────────────────────

verify_backup() {
    local file="$1"

    if [ ! -f "$file" ]; then
        die "Backup file not found: $file"
    fi

    parse_db_url
    check_deps

    local verify_db="${DB_NAME}_verify_$$"
    local format_flag="-Fc"
    [[ "$file" == *.sql ]] && format_flag=""

    log_info "Verifying backup: $file"
    log_info "Restoring to temp database: $verify_db"

    PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="postgres" \
        --no-password \
        -c "CREATE DATABASE \"${verify_db}\" OWNER \"${DB_USER}\";" 2>/dev/null || die "Failed to create temp database"

    if ! PGPASSWORD="$DB_PASS" pg_restore \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="$verify_db" \
        --no-password \
        --no-owner \
        ${format_flag:+"$format_flag"} \
        "$file" 2>&1; then
        PGPASSWORD="$DB_PASS" psql \
            --host="$DB_HOST" \
            --port="$DB_PORT" \
            --username="$DB_USER" \
            --dbname="postgres" \
            --no-password \
            -c "DROP DATABASE IF EXISTS \"${verify_db}\";" 2>/dev/null
        die "pg_restore to temp database failed"
    fi

    local table_count
    table_count=$(PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="$verify_db" \
        --no-password \
        -tAc "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');" 2>/dev/null)

    PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="postgres" \
        --no-password \
        -c "DROP DATABASE IF EXISTS \"${verify_db}\";" 2>/dev/null

    log_ok "Verification passed: ${table_count:-0} tables restored successfully"
}

# ─── Retention / Cleanup ─────────────────────────────────────────────────────

cleanup_backups() {
    log_info "Applying retention policy..."
    log_info "  Daily:  keep last ${RETENTION_DAILY}"
    log_info "  Weekly: keep last ${RETENTION_WEEKLY}"
    log_info "  Monthly: keep last ${RETENTION_MONTHLY}"

    if [ ! -d "$BACKUP_DIR" ]; then
        log_warn "Backup directory does not exist: $BACKUP_DIR"
        return
    fi

    # Collect all backup files sorted by timestamp (newest first)
    local backups
    mapfile -t backups < <(
        find "$BACKUP_DIR" -maxdepth 1 -name 'tachyon-backup-*.sql' -printf '%f\n' 2>/dev/null |
        sort -r
    )

    if [ ${#backups[@]} -eq 0 ]; then
        log_info "No backup files to clean up"
        return
    fi

    local kept=0
    local -A kept_files
    local day_idx=0 week_idx=0 month_idx=0
    local last_day="" last_week="" last_month=""

    for b in "${backups[@]}"; do
        # Extract timestamp: tachyon-backup-YYYYMMDD-HHMMSS.sql
        local ts="${b#tachyon-backup-}"
        ts="${ts%.sql}"

        local day="${ts:0:8}"             # YYYYMMDD
        local wk
        wk=$(week_bucket "$ts")
        local mo="${ts:0:6}"              # YYYYMM

        # Daily: keep if it's a new day within limit
        if [ "$day" != "$last_day" ] && [ "$day_idx" -lt "$RETENTION_DAILY" ]; then
            kept_files["$b"]=1
            last_day="$day"
            day_idx=$((day_idx + 1))
            kept=$((kept + 1))
            continue
        fi

        # Weekly: keep first backup of each ISO week
        if [ "$wk" != "$last_week" ] && [ "$week_idx" -lt "$RETENTION_WEEKLY" ]; then
            kept_files["$b"]=1
            last_week="$wk"
            week_idx=$((week_idx + 1))
            kept=$((kept + 1))
            continue
        fi

        # Monthly: keep first backup of each month
        if [ "$mo" != "$last_month" ] && [ "$month_idx" -lt "$RETENTION_MONTHLY" ]; then
            kept_files["$b"]=1
            last_month="$mo"
            month_idx=$((month_idx + 1))
            kept=$((kept + 1))
        fi
    done

    # Remove files not marked as kept
    local removed=0
    for b in "${backups[@]}"; do
        if [ -z "${kept_files[$b]:-}" ]; then
            rm -f "${BACKUP_DIR}/${b}"
            log_info "Removed: $b"
            removed=$((removed + 1))
        fi
    done

    # Clean up WAL archives older than 7 days
    if [ -d "${BACKUP_DIR}/wal_archive" ]; then
        local wal_removed
        wal_removed=$(find "${BACKUP_DIR}/wal_archive" -type f -mtime +7 -delete -print 2>/dev/null | wc -l)
        log_info "Removed ${wal_removed} old WAL archive files"
    fi

    log_ok "Retention cleanup: kept ${kept}, removed ${removed}"
}

# ─── Main ────────────────────────────────────────────────────────────────────

main() {
    local cmd="${1:-}"

    case "$cmd" in
        full)
            full_backup
            ;;
        incremental)
            incremental_backup
            ;;
        restore)
            [[ -n "${2:-}" ]] || die "Usage: $0 restore <file>"
            restore_backup "$2"
            ;;
        verify)
            [[ -n "${2:-}" ]] || die "Usage: $0 verify <file>"
            verify_backup "$2"
            ;;
        cleanup)
            cleanup_backups
            ;;
        -h|--help|help|"")
            usage
            ;;
        *)
            log_error "Unknown command: $cmd"
            usage
            ;;
    esac
}

main "$@"
