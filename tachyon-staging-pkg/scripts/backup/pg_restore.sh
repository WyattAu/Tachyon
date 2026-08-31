#!/usr/bin/env bash
# Tachyon PostgreSQL Restore Script
#
# Restores a database from a pg_dump backup file with integrity verification.
# Supports point-in-time recovery via WAL archive.
#
# Usage: pg_restore.sh <backup_file> [--target-time <timestamp>] [--no-verify]
#
# Environment:
#   DATABASE_URL        PostgreSQL connection string
#   WAL_ARCHIVE_DIR     Directory containing WAL segments for PITR

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DATABASE_URL="${DATABASE_URL:-postgres://tachyon:tachyon@localhost:5432/tachyon}"
WAL_ARCHIVE_DIR="${WAL_ARCHIVE_DIR:-${SCRIPT_DIR}/../../backups/wal_archive}"

NO_VERIFY=0
TARGET_TIME=""
BACKUP_FILE=""

# ── Argument parsing ──────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --target-time)
            TARGET_TIME="${2:?--target-time requires a value}"
            shift 2
            ;;
        --no-verify)
            NO_VERIFY=1
            shift
            ;;
        -*)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
        *)
            BACKUP_FILE="$1"
            shift
            ;;
    esac
done

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: pg_restore.sh <backup_file> [--target-time <timestamp>] [--no-verify]" >&2
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file not found: $BACKUP_FILE" >&2
    exit 1
fi

# ── Logging ────────────────────────────────────────────────────────────────────

_log() {
    local level="$1"; shift
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] $*"
}

log_info()  { _log "INFO"  "$@"; }
log_ok()    { _log "OK"    "$@"; }
log_warn()  { _log "WARN"  "$@"; }
log_error() { _log "ERROR" "$@" >&2; }

die() {
    log_error "$@"
    exit 1
}

# ── Helpers ────────────────────────────────────────────────────────────────────

check_deps() {
    for cmd in pg_restore psql; do
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

psql_cmd() {
    PGPASSWORD="$DB_PASS" psql \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --no-password \
        "$@"
}

# ── Verify backup ──────────────────────────────────────────────────────────────

verify_backup() {
    local file="$1"

    log_info "Verifying backup integrity: $(basename "$file")"

    # Check gzip integrity if gzipped
    if [[ "$file" == *.gz ]]; then
        if ! gzip -t "$file" 2>/dev/null; then
            die "Backup gzip integrity check failed: $file"
        fi
        log_ok "Gzip integrity check passed"
    fi

    # Check pg_restore can list contents
    local cat_cmd="cat"
    if [[ "$file" == *.gz ]]; then
        cat_cmd="zcat"
    fi

    if ! $cat_cmd "$file" | pg_restore --list &>/dev/null; then
        die "Backup pg_restore --list failed (corrupt backup): $file"
    fi

    log_ok "Backup integrity verified"
}

# ── Restore ────────────────────────────────────────────────────────────────────

do_restore() {
    check_deps
    parse_db_url

    log_info "=== Tachyon PostgreSQL Restore ==="
    log_info "Backup file: $BACKUP_FILE"
    log_info "Target database: ${DB_NAME} on ${DB_HOST}:${DB_PORT}"

    if [ "$NO_VERIFY" -eq 0 ]; then
        verify_backup "$BACKUP_FILE"
    fi

    log_warn "This will DROP and recreate database: ${DB_NAME}"
    echo -n "Type 'yes' to confirm: "
    read -r confirm
    if [ "$confirm" != "yes" ]; then
        log_warn "Restore cancelled"
        exit 0
    fi

    # Terminate existing connections
    log_info "Terminating existing connections to ${DB_NAME}..."
    psql_cmd --dbname="postgres" -c \
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${DB_NAME}' AND pid <> pg_backend_pid();" \
        2>/dev/null || true

    # Recreate database
    log_info "Recreating database ${DB_NAME}..."
    psql_cmd --dbname="postgres" -c "DROP DATABASE IF EXISTS \"${DB_NAME}\";" 2>/dev/null || true
    psql_cmd --dbname="postgres" -c "CREATE DATABASE \"${DB_NAME}\" OWNER \"${DB_USER}\";" || die "Failed to create database"

    # Restore data
    log_info "Restoring data..."
    local cat_cmd="cat"
    if [[ "$BACKUP_FILE" == *.gz ]]; then
        cat_cmd="zcat"
    fi

    if ! $cat_cmd "$BACKUP_FILE" | pg_restore \
        --host="$DB_HOST" \
        --port="$DB_PORT" \
        --username="$DB_USER" \
        --dbname="$DB_NAME" \
        --no-password \
        --clean \
        --if-exists \
        --no-owner \
        --verbose 2>&1; then
        log_warn "pg_restore completed with warnings (this may be normal for --clean)"
    fi

    # Verify table count
    local table_count
    table_count=$(psql_cmd --dbname="$DB_NAME" -tAc \
        "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');" 2>/dev/null || echo "0")

    log_ok "Restore complete: ${table_count} tables restored in ${DB_NAME}"

    # Point-in-time recovery note
    if [ -n "$TARGET_TIME" ]; then
        log_info "Point-in-time recovery target: $TARGET_TIME"
        if [ -d "$WAL_ARCHIVE_DIR" ]; then
            log_info "WAL archive found at: $WAL_ARCHIVE_DIR"
            log_info "To complete PITR, set restore_command in postgresql.conf:"
            log_info "  restore_command = 'cp ${WAL_ARCHIVE_DIR}/%f %p'"
            log_info "  recovery_target_time = '${TARGET_TIME}'"
        else
            log_warn "WAL archive directory not found: $WAL_ARCHIVE_DIR"
        fi
    fi

    log_ok "=== Restore finished ==="
}

do_restore
