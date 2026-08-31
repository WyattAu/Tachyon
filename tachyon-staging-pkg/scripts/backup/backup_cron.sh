#!/usr/bin/env bash
# Tachyon Backup Cron Wrapper
#
# Runs pg_backup.sh at configured intervals and sends notifications on failure.
#
# Usage: backup_cron.sh [--hourly|--daily|--weekly]
#
# Environment:
#   BACKUP_INTERVAL  cron interval: hourly|daily|weekly (default: daily)
#   NOTIFY_WEBHOOK   URL for notification webhook (Slack/Discord/Google Chat)
#   NOTIFY_EMAIL     Email address for failure notifications
#   BACKUP_DIR       Local backup directory
#   DATABASE_URL     PostgreSQL connection string

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BACKUP_INTERVAL="${BACKUP_INTERVAL:-daily}"
NOTIFY_WEBHOOK="${NOTIFY_WEBHOOK:-}"
NOTIFY_EMAIL="${NOTIFY_EMAIL:-}"
LOG_FILE="${LOG_FILE:-${BACKUP_DIR:-${PROJECT_ROOT}/backups}/cron.log}"

for arg in "$@"; do
    case "$arg" in
        --hourly) BACKUP_INTERVAL="hourly" ;;
        --daily)  BACKUP_INTERVAL="daily" ;;
        --weekly) BACKUP_INTERVAL="weekly" ;;
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
log_error() { _log "ERROR" "$@"; }

# ── Notification ───────────────────────────────────────────────────────────────

send_notification() {
    local subject="$1"
    local body="$2"

    if [ -n "$NOTIFY_WEBHOOK" ]; then
        local payload
        payload=$(printf '{"text":"[%s] %s: %s"}' "$(hostname)" "$subject" "$body")
        if command -v curl &>/dev/null; then
            curl -s -X POST -H 'Content-Type: application/json' -d "$payload" "$NOTIFY_WEBHOOK" &>/dev/null || true
        fi
    fi

    if [ -n "$NOTIFY_EMAIL" ] && command -v mail &>/dev/null; then
        echo "$body" | mail -s "[Tachyon Backup] $subject on $(hostname)" "$NOTIFY_EMAIL" 2>/dev/null || true
    fi
}

# ── Check interval ─────────────────────────────────────────────────────────────

check_interval() {
    local marker_file="${BACKUP_DIR:-${PROJECT_ROOT}/backups}/.last_${BACKUP_INTERVAL}"
    local now
    now=$(date +%s)

    if [ -f "$marker_file" ]; then
        local last_run
        last_run=$(stat -c %Y "$marker_file" 2>/dev/null || echo 0)
        local elapsed=$(( now - last_run ))

        case "$BACKUP_INTERVAL" in
            hourly)  min_seconds=3600 ;;
            daily)   min_seconds=86400 ;;
            weekly)  min_seconds=604800 ;;
            *)       min_seconds=86400 ;;
        esac

        if [ "$elapsed" -lt "$min_seconds" ]; then
            log_info "Skipping: last ${BACKUP_INTERVAL} backup was $(( elapsed / 60 )) minutes ago"
            return 1
        fi
    fi

    return 0
}

# ── Main ───────────────────────────────────────────────────────────────────────

main() {
    log_info "=== Tachyon Backup Cron (${BACKUP_INTERVAL}) ==="

    if ! check_interval; then
        exit 0
    fi

    log_info "Running ${BACKUP_INTERVAL} backup..."

    local backup_script="${SCRIPT_DIR}/pg_backup.sh"
    if [ ! -x "$backup_script" ]; then
        log_error "Backup script not found or not executable: $backup_script"
        send_notification "BACKUP FAILED" "Script not executable: $backup_script"
        exit 1
    fi

    local start_time
    start_time=$(date +%s)

    if "$backup_script" 2>&1 | tee -a "$LOG_FILE"; then
        local end_time
        end_time=$(date +%s)
        local duration=$(( end_time - start_time ))
        log_ok "Backup succeeded in ${duration}s"
        touch "${BACKUP_DIR:-${PROJECT_ROOT}/backups}/.last_${BACKUP_INTERVAL}"
    else
        local end_time
        end_time=$(date +%s)
        local duration=$(( end_time - start_time ))
        log_error "Backup failed after ${duration}s"
        send_notification "BACKUP FAILED" "Backup failed after ${duration}s on $(hostname). Check logs: $LOG_FILE"
        exit 1
    fi
}

main
