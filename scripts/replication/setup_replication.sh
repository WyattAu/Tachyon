#!/usr/bin/env bash
# Tachyon PostgreSQL Streaming Replication Setup
#
# Configures a primary + standby streaming replication with replication slots
# and WAL archiving.
#
# Usage: setup_replication.sh <primary|standby> [options]
#
# Primary mode:
#   setup_replication.sh primary --replication-user <user> --replication-pass <pass>
#
# Standby mode:
#   setup_replication.sh standby --primary-host <host> --replication-user <user> --replication-pass <pass>
#
# Environment:
#   DATABASE_URL              Primary PostgreSQL connection string
#   REPLICATION_USER          Replication user (default: replicator)
#   REPLICATION_PASSWORD      Replication user password
#   PRIMARY_HOST              Primary host address (for standby mode)
#   WAL_ARCHIVE_DIR           WAL archive directory
#   PGDATA                    PostgreSQL data directory

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Defaults ───────────────────────────────────────────────────────────────────

DATABASE_URL="${DATABASE_URL:-postgres://tachyon:tachyon@localhost:5432/tachyon}"
REPLICATION_USER="${REPLICATION_USER:-replicator}"
REPLICATION_PASSWORD="${REPLICATION_PASSWORD:-}"
PRIMARY_HOST="${PRIMARY_HOST:-}"
WAL_ARCHIVE_DIR="${WAL_ARCHIVE_DIR:-/var/lib/postgresql/wal_archive}"
PGDATA="${PGDATA:-/var/lib/postgresql/data}"

MODE=""

# ── Argument parsing ──────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        primary|standby)
            MODE="$1"
            shift
            ;;
        --replication-user)
            REPLICATION_USER="${2:?}"
            shift 2
            ;;
        --replication-pass)
            REPLICATION_PASSWORD="${2:?}"
            shift 2
            ;;
        --primary-host)
            PRIMARY_HOST="${2:?}"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

if [ -z "$MODE" ]; then
    echo "Usage: setup_replication.sh <primary|standby> [options]" >&2
    exit 1
fi

# ── Logging ────────────────────────────────────────────────────────────────────

_log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$1] $2"; }
log_info()  { _log "INFO"  "$*"; }
log_ok()    { _log "OK"    "$*"; }
log_error() { _log "ERROR" "$*" >&2; }

die() { log_error "$@"; exit 1; }

# ── Helpers ────────────────────────────────────────────────────────────────────

check_deps() {
    for cmd in psql; do
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

# ── Primary setup ──────────────────────────────────────────────────────────────

setup_primary() {
    parse_db_url
    check_deps

    if [ -z "$REPLICATION_PASSWORD" ]; then
        die "REPLICATION_PASSWORD is required for primary setup"
    fi

    log_info "Setting up PRIMARY node..."

    # Create replication user
    log_info "Creating replication user: ${REPLICATION_USER}"
    psql_cmd --dbname="postgres" -c \
        "CREATE USER ${REPLICATION_USER} WITH REPLICATION ENCRYPTED PASSWORD '${REPLICATION_PASSWORD}';" \
        2>/dev/null || log_info "User ${REPLICATION_USER} may already exist"

    # Create replication slot
    local slot_name="tachyon_standby_1"
    log_info "Creating replication slot: ${slot_name}"
    psql_cmd --dbname="postgres" -c \
        "SELECT pg_create_physical_replication_slot('${slot_name}');" \
        2>/dev/null || log_info "Replication slot may already exist"

    # Configure WAL archiving
    log_info "Configuring WAL archiving..."
    mkdir -p "$WAL_ARCHIVE_DIR"

    psql_cmd --dbname="$DB_NAME" <<'SQL' 2>/dev/null || log_info "Some settings may require postgresql.conf edit"
ALTER SYSTEM SET wal_level = replica;
ALTER SYSTEM SET max_wal_senders = 5;
ALTER SYSTEM SET wal_keep_size = '256MB';
ALTER SYSTEM SET max_replication_slots = 5;
ALTER SYSTEM SET archive_mode = on;
ALTER SYSTEM SET archive_command = 'test ! -f ${WAL_ARCHIVE_DIR}/%f && cp %p ${WAL_ARCHIVE_DIR}/%f';
ALTER SYSTEM SET archive_timeout = 60;
ALTER SYSTEM SET hot_standby = on;
SELECT pg_reload_conf();
SQL

    log_ok "Primary node configured"
    log_info ""
    log_info "PostgreSQL configuration applied via ALTER SYSTEM."
    log_info "Verify in postgresql.conf or pg_settings:"
    log_info "  wal_level = replica"
    log_info "  max_wal_senders = 5"
    log_info "  archive_mode = on"
    log_info "  archive_command = 'cp %p ${WAL_ARCHIVE_DIR}/%f'"
    log_info ""
    log_info "Replication slot: ${slot_name}"
    log_info "Replication user: ${REPLICATION_USER}"
    log_info ""
    log_info "To add a standby, run on the standby host:"
    log_info "  $0 standby --primary-host ${DB_HOST} --replication-user ${REPLICATION_USER} --replication-pass <password>"
}

# ── Standby setup ──────────────────────────────────────────────────────────────

setup_standby() {
    parse_db_url

    if [ -z "$PRIMARY_HOST" ]; then
        die "PRIMARY_HOST is required for standby setup (use --primary-host)"
    fi

    if [ -z "$REPLICATION_PASSWORD" ]; then
        die "REPLICATION_PASSWORD is required for standby setup"
    fi

    log_info "Setting up STANDBY node..."
    log_info "Primary host: ${PRIMARY_HOST}"

    # Create .pgpass for replication
    local pgpass_dir="${HOME:-/var/lib/postgresql}"
    log_info "Creating .pgpass entry for replication..."
    echo "${PRIMARY_HOST}:5432:replication:${REPLICATION_USER}:${REPLICATION_PASSWORD}" >> "${pgpass_dir}/.pgpass"
    chmod 600 "${pgpass_dir}/.pgpass" 2>/dev/null || true

    # Take base backup from primary
    local backup_dir="${PGDATA}_basebackup"
    log_info "Taking base backup from primary to ${backup_dir}..."

    if command -v pg_basebackup &>/dev/null; then
        PGPASSWORD="$REPLICATION_PASSWORD" pg_basebackup \
            --host="$PRIMARY_HOST" \
            --port="${DB_PORT:-5432}" \
            --username="$REPLICATION_USER" \
            --pgdata="$backup_dir" \
            --format=plain \
            --write-recovery-conf \
            --progress \
            --verbose \
            --checkpoint=fast \
            || die "pg_basebackup failed"

        log_ok "Base backup complete"

        # Configure recovery
        if [ -f "${backup_dir}/postgresql.auto.conf" ]; then
            cat >> "${backup_dir}/postgresql.auto.conf" <<EOF
primary_conninfo = 'host=${PRIMARY_HOST} port=${DB_PORT:-5432} user=${REPLICATION_USER} password=${REPLICATION_PASSWORD} sslmode=prefer'
primary_slot_name = 'tachyon_standby_1'
hot_standby = on
EOF
        fi

        # Create standby.signal
        touch "${backup_dir}/standby.signal"

        log_ok "Standby configuration written to ${backup_dir}"
        log_info ""
        log_info "To activate the standby:"
        log_info "  1. Stop PostgreSQL"
        log_info "  2. Move current PGDATA: mv ${PGDATA} ${PGDATA}.old"
        log_info "  3. Move base backup:    mv ${backup_dir} ${PGDATA}"
        log_info "  4. Start PostgreSQL"
        log_info ""
        log_info "Verify replication on primary:"
        log_info "  SELECT * FROM pg_stat_replication;"
    else
        die "pg_basebackup not found. Install PostgreSQL client tools."
    fi
}

# ── Main ───────────────────────────────────────────────────────────────────────

case "$MODE" in
    primary) setup_primary ;;
    standby) setup_standby ;;
    *)       die "Unknown mode: $MODE" ;;
esac
