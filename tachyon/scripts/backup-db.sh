#!/usr/bin/env bash
# Tachyon Database Backup Script
# Supports: pg_dump backup, rotation, restore, and verification
#
# Usage:
#   ./scripts/backup-db.sh backup          # Create a backup
#   ./scripts/backup-db.sh restore <file>  # Restore from backup
#   ./scripts/backup-db.sh list            # List available backups
#   ./scripts/backup-db.sh verify <file>   # Verify backup integrity
#   ./scripts/backup-db.sh cleanup         # Remove old backups beyond retention
#
# Environment variables:
#   DATABASE_URL          - PostgreSQL connection string (required)
#   BACKUP_DIR            - Directory to store backups (default: ./backups)
#   BACKUP_RETENTION_DAYS - Days to keep backups (default: 30)
#   BACKUP_COMPRESSION    - Enable gzip compression (default: true)

set -euo pipefail

# --- Configuration ---
BACKUP_DIR="${BACKUP_DIR:-./backups}"
BACKUP_RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"
BACKUP_COMPRESSION="${BACKUP_COMPRESSION:-true}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# --- Validate environment ---
validate_env() {
    if [[ -z "${DATABASE_URL:-}" ]]; then
        log_error "DATABASE_URL is not set."
        echo "Example: export DATABASE_URL='postgresql://user:pass@localhost:5432/tachyon'"
        exit 1
    fi

    if ! command -v pg_dump &>/dev/null; then
        log_error "pg_dump not found. Install postgresql-client."
        exit 1
    fi

    if ! command -v psql &>/dev/null; then
        log_error "psql not found. Install postgresql-client."
        exit 1
    fi
}

# --- Extract connection details from DATABASE_URL ---
parse_db_url() {
    # postgresql://user:password@host:port/database?params
    local url="$DATABASE_URL"
    DB_HOST=$(echo "$url" | sed -n 's|.*@\([^:]*\):\([0-9]*\)/.*|\1|p')
    DB_PORT=$(echo "$url" | sed -n 's|.*@\([^:]*\):\([0-9]*\)/.*|\2|p')
    DB_NAME=$(echo "$url" | sed -n 's|.*\/\([^?]*\).*|\1|p')
    DB_USER=$(echo "$url" | sed -n 's|.*://\([^:]*\):.*|\1|p')

    DB_HOST="${DB_HOST:-localhost}"
    DB_PORT="${DB_PORT:-5432}"
    DB_NAME="${DB_NAME:-tachyon}"
    DB_USER="${DB_USER:-postgres}"
}

# --- Create backup ---
do_backup() {
    validate_env
    parse_db_url

    mkdir -p "$BACKUP_DIR"

    local dump_file="${BACKUP_DIR}/tachyon_${DB_NAME}_${TIMESTAMP}.sql"
    local meta_file="${dump_file}.meta"

    log_info "Starting backup of '${DB_NAME}' on ${DB_HOST}:${DB_PORT}..."

    # Test connection first
    if ! psql "$DATABASE_URL" -c "SELECT 1" &>/dev/null; then
        log_error "Cannot connect to database. Check DATABASE_URL."
        exit 1
    fi

    # Get database size before backup
    local db_size
    db_size=$(psql "$DATABASE_URL" -t -c "SELECT pg_size_pretty(pg_database_size(current_database()))" 2>/dev/null | xargs)
    log_info "Database size: ${db_size:-unknown}"

    # Get current migration version
    local migration_version
    migration_version=$(psql "$DATABASE_URL" -t -c "SELECT COALESCE(MAX(version), 'none') FROM _sqlx_migrations" 2>/dev/null | xargs)

    # Run pg_dump
    local start_time
    start_time=$(date +%s)

    pg_dump "$DATABASE_URL" \
        --no-owner \
        --no-privileges \
        --clean \
        --if-exists \
        --verbose \
        --file="$dump_file" 2>/dev/null

    local end_time
    end_time=$(date +%s)
    local duration=$(( end_time - start_time ))

    # Compress if enabled
    if [[ "$BACKUP_COMPRESSION" == "true" ]]; then
        log_info "Compressing backup..."
        gzip "$dump_file"
        dump_file="${dump_file}.gz"
    fi

    # Write metadata
    local file_size
    file_size=$(du -h "$dump_file" | cut -f1)

    cat > "$meta_file" <<EOF
backup_file=$(basename "$dump_file")
database=$DB_NAME
host=$DB_HOST
port=$DB_PORT
user=$DB_USER
timestamp=$TIMESTAMP
duration_seconds=$duration
db_size=${db_size:-unknown}
migration_version=${migration_version:-unknown}
compressed=$BACKUP_COMPRESSION
created_at=$(date -Iseconds)
EOF

    log_ok "Backup complete: ${dump_file}"
    log_info "  Size: ${file_size}"
    log_info "  Duration: ${duration}s"
    log_info "  Migration version: ${migration_version:-unknown}"
    log_info "  Retention: ${BACKUP_RETENTION_DAYS} days"

    # Cleanup old backups
    cleanup_old_backups
}

# --- Restore from backup ---
do_restore() {
    local backup_file="$1"

    if [[ -z "$backup_file" ]]; then
        log_error "Usage: $0 restore <backup_file>"
        exit 1
    fi

    if [[ ! -f "$backup_file" ]]; then
        log_error "Backup file not found: ${backup_file}"
        exit 1
    fi

    validate_env
    parse_db_url

    log_warn "⚠️  WARNING: This will OVERWRITE the database '${DB_NAME}'!"
    log_warn "   Press Ctrl+C within 5 seconds to cancel."
    sleep 5

    local dump_file="$backup_file"

    # Decompress if gzipped
    if [[ "$backup_file" == *.gz ]]; then
        log_info "Decompressing backup..."
        dump_file="${backup_file%.gz}"
        if [[ ! -f "$dump_file" ]]; then
            gunzip -k "$backup_file"
        fi
    fi

    log_info "Restoring from: ${dump_file}"
    log_info "Target database: ${DB_NAME} on ${DB_HOST}:${DB_PORT}"

    # Create a pre-restore backup
    local pre_restore_backup="${BACKUP_DIR}/tachyon_${DB_NAME}_pre_restore_${TIMESTAMP}.sql"
    log_info "Creating pre-restore backup..."
    pg_dump "$DATABASE_URL" --no-owner --clean --if-exists --file="$pre_restore_backup" 2>/dev/null || true

    # Restore
    local start_time
    start_time=$(date +%s)

    psql "$DATABASE_URL" --file="$dump_file" 2>/dev/null

    local end_time
    end_time=$(date +%s)
    local duration=$(( end_time - start_time ))

    log_ok "Restore complete in ${duration}s"
    log_info "Pre-restore backup saved at: ${pre_restore_backup}"
}

# --- List backups ---
do_list() {
    if [[ ! -d "$BACKUP_DIR" ]]; then
        log_info "No backup directory found at ${BACKUP_DIR}"
        return 0
    fi

    local count=0
    echo ""
    echo "Available backups in ${BACKUP_DIR}:"
    echo "─────────────────────────────────────────────────────────────────"
    printf "%-45s %-8s %s\n" "FILE" "SIZE" "DATE"
    echo "─────────────────────────────────────────────────────────────────"

    for file in "${BACKUP_DIR}"/tachyon_*_*.sql*; do
        [[ -f "$file" ]] || continue
        local fname
        fname=$(basename "$file")
        local fsize
        fsize=$(du -h "$file" | cut -f1)
        local fdate
        fdate=$(stat -c %y "$file" 2>/dev/null | cut -d' ' -f1-2 || stat -f "%Sm" "$file" 2>/dev/null)
        printf "%-45s %-8s %s\n" "$fname" "$fsize" "$fdate"
        ((count++))
    done

    echo "─────────────────────────────────────────────────────────────────"
    echo "Total: ${count} backup(s)"
    echo ""
}

# --- Verify backup ---
do_verify() {
    local backup_file="$1"

    if [[ -z "$backup_file" ]]; then
        log_error "Usage: $0 verify <backup_file>"
        exit 1
    fi

    if [[ ! -f "$backup_file" ]]; then
        log_error "Backup file not found: ${backup_file}"
        exit 1
    fi

    log_info "Verifying backup: ${backup_file}"

    local dump_file="$backup_file"
    local is_gzipped=false

    # Decompress if gzipped
    if [[ "$backup_file" == *.gz ]]; then
        is_gzipped=true
        dump_file="${backup_file%.gz}"
        if [[ ! -f "$dump_file" ]]; then
            log_info "Decompressing for verification..."
            gunzip -k "$backup_file"
        fi
    fi

    # Check file is not empty
    local file_size
    file_size=$(stat -c%s "$dump_file" 2>/dev/null || stat -f%z "$dump_file" 2>/dev/null)
    if [[ "$file_size" -eq 0 ]]; then
        log_error "Backup file is empty!"
        return 1
    fi

    # Check for essential SQL markers
    local has_create=true
    local has_insert=true

    if ! grep -qi "CREATE TABLE" "$dump_file"; then
        has_create=false
    fi
    if ! grep -qi "INSERT INTO" "$dump_file"; then
        has_insert=false
    fi

    log_info "File size: $(du -h "$dump_file" | cut -f1)"
    log_info "Contains CREATE TABLE: ${has_create}"
    log_info "Contains INSERT INTO: ${has_insert}"

    # Clean up decompressed temp file if we created it
    if [[ "$is_gzipped" == true ]] && [[ ! -f "$backup_file".orig ]]; then
        rm -f "$dump_file"
    fi

    if [[ "$has_create" == true ]]; then
        log_ok "Backup verification passed"
        return 0
    else
        log_warn "Backup may be incomplete (no CREATE TABLE found)"
        return 1
    fi
}

# --- Cleanup old backups ---
cleanup_old_backups() {
    if [[ ! -d "$BACKUP_DIR" ]]; then
        return 0
    fi

    log_info "Cleaning up backups older than ${BACKUP_RETENTION_DAYS} days..."

    local deleted=0
    while IFS= read -r -d '' file; do
        rm -f "$file" "${file}.meta"
        ((deleted++))
    done < <(find "$BACKUP_DIR" -name "tachyon_*_*.sql*" -type f -mtime +"$BACKUP_RETENTION_DAYS" -print0 2>/dev/null)

    if [[ $deleted -gt 0 ]]; then
        log_info "Removed ${deleted} old backup(s)"
    fi
}

# --- Cron setup helper ---
do_cron() {
    local script_path
    script_path=$(realpath "$0")

    echo ""
    echo "Add this to your crontab (crontab -e):"
    echo ""
    echo "# Tachyon database backup - daily at 2 AM"
    echo "0 2 * * * DATABASE_URL='${DATABASE_URL:-postgresql://tachyon:tachyon@localhost:5432/tachyon}' BACKUP_DIR='${BACKUP_DIR}' BACKUP_RETENTION_DAYS='${BACKUP_RETENTION_DAYS}' ${script_path} backup >> /var/log/tachyon-backup.log 2>&1"
    echo ""
    echo "# Verify most recent backup - weekly on Sunday at 3 AM"
    echo "0 3 * * 0 ${script_path} verify \$(ls -t ${BACKUP_DIR}/tachyon_*_*.sql* 2>/dev/null | head -1) >> /var/log/tachyon-backup.log 2>&1"
    echo ""
}

# --- Main ---
case "${1:-help}" in
    backup)
        do_backup
        ;;
    restore)
        do_restore "${2:-}"
        ;;
    list)
        do_list
        ;;
    verify)
        do_verify "${2:-}"
        ;;
    cleanup)
        cleanup_old_backups
        ;;
    cron)
        do_cron
        ;;
    help|*)
        echo ""
        echo "Tachyon Database Backup Tool"
        echo ""
        echo "Usage: $0 <command> [args]"
        echo ""
        echo "Commands:"
        echo "  backup           Create a new backup"
        echo "  restore <file>   Restore from a backup file"
        echo "  list             List available backups"
        echo "  verify <file>    Verify backup integrity"
        echo "  cleanup          Remove old backups beyond retention"
        echo "  cron             Show crontab configuration"
        echo ""
        echo "Environment:"
        echo "  DATABASE_URL            PostgreSQL connection string"
        echo "  BACKUP_DIR              Backup directory (default: ./backups)"
        echo "  BACKUP_RETENTION_DAYS   Days to keep (default: 30)"
        echo "  BACKUP_COMPRESSION      gzip compression (default: true)"
        echo ""
        ;;
esac
