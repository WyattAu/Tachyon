#!/usr/bin/env bash
# launch-tachyon.sh — Start the full Tachyon stack (PostgreSQL + Server + Desktop)
# Usage: ./launch-tachyon.sh [start|stop|status|server-only|desktop-only]
set -euo pipefail

# === Configuration ===
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
PGDATA="/tmp/tachyon-pg"
PG_PORT=5433
PG_HOST="127.0.0.1"
PG_USER="tachyon"
PG_PASS="tachyon"
PG_DB="tachyon"

SERVER_PORT=8080
SERVER_HOST="127.0.0.1"
JWT_SECRET="dev-secret-key-at-least-32-chars-long-ok"
ADMIN_USER="admin"
ADMIN_PASS="admin123"

TAURI_BIN="${PROJECT_ROOT}/target/debug/tachyon-desktop-app"
SERVER_BIN="${PROJECT_ROOT}/target/debug/tachyon-server"

# === Helper Functions ===
log() { echo "[$(date +%H:%M:%S)] $*"; }

ensure_pg_running() {
    # Check if PostgreSQL is already running
    if pg_isready -h "$PG_HOST" -p "$PG_PORT" >/dev/null 2>&1; then
        log "PostgreSQL already running on ${PG_HOST}:${PG_PORT}"
        return 0
    fi

    # Initialize if needed
    if [ ! -f "$PGDATA/postgresql.conf" ]; then
        log "Initializing PostgreSQL at $PGDATA..."
        mkdir -p "$PGDATA"
        initdb --auth=trust --no-locale --encoding=UTF8 -D "$PGDATA" >/dev/null 2>&1
        cat >> "$PGDATA/postgresql.conf" <<EOF
port = $PG_PORT
unix_socket_directories = '$PGDATA'
listen_addresses = '$PG_HOST'
EOF
    fi

    # Start PostgreSQL
    log "Starting PostgreSQL..."
    pg_ctl -D "$PGDATA" -l "$PGDATA/pg.log" start -w -t 10 >/dev/null 2>&1

    if pg_isready -h "$PG_HOST" -p "$PG_PORT" >/dev/null 2>&1; then
        log "PostgreSQL started on ${PG_HOST}:${PG_PORT}"
    else
        log "ERROR: PostgreSQL failed to start"
        exit 1
    fi

    # Ensure user and database exist
    psql -h "$PG_HOST" -p "$PG_PORT" -U "$(whoami)" -d postgres -tc \
        "SELECT 1 FROM pg_roles WHERE rolname='$PG_USER'" | grep -q 1 2>/dev/null \
        || psql -h "$PG_HOST" -p "$PG_PORT" -U "$(whoami)" -d postgres -c \
            "CREATE USER $PG_USER WITH PASSWORD '$PG_PASS';" >/dev/null 2>&1

    psql -h "$PG_HOST" -p "$PG_PORT" -U "$(whoami)" -d postgres -tc \
        "SELECT 1 FROM pg_database WHERE datname='$PG_DB'" | grep -q 1 2>/dev/null \
        || psql -h "$PG_HOST" -p "$PG_PORT" -U "$(whoami)" -d postgres -c \
            "CREATE DATABASE $PG_DB OWNER $PG_USER;" >/dev/null 2>&1
}

start_server() {
    # Kill existing server
    fuser -k "${SERVER_PORT}/tcp" 2>/dev/null || true
    sleep 1

    log "Starting tachyon-server on ${SERVER_HOST}:${SERVER_PORT}..."
    DATABASE_URL="postgres://${PG_USER}:${PG_PASS}@${PG_HOST}:${PG_PORT}/${PG_DB}" \
    TACHYON_JWT_SECRET="$JWT_SECRET" \
    TACHYON_HOST="$SERVER_HOST" \
    TACHYON_PORT="$SERVER_PORT" \
    TACHYON_ADMIN_USERNAME="$ADMIN_USER" \
    TACHYON_ADMIN_PASSWORD="$ADMIN_PASS" \
    TACHYON_SECURITY_CSP_ENABLED=false \
    RUST_LOG="tachyon_server=info" \
    nohup "$SERVER_BIN" >/tmp/tachyon-server.log 2>&1 &
    echo $! > /tmp/tachyon-server.pid

    # Wait for server
    for i in $(seq 30); do
        if curl -sf "http://${SERVER_HOST}:${SERVER_PORT}/health" >/dev/null 2>&1; then
            log "Server ready (took ${i}s)"
            log "  Admin: $ADMIN_USER / $ADMIN_PASS"
            log "  API:   http://${SERVER_HOST}:${SERVER_PORT}/api/v1/"
            log "  Health: http://${SERVER_HOST}:${SERVER_PORT}/health"
            return 0
        fi
        sleep 1
    done
    log "ERROR: Server failed to start within 30s"
    log "  Check /tmp/tachyon-server.log"
    exit 1
}

start_desktop() {
    # Kill existing desktop
    pkill -f tachyon-desktop 2>/dev/null || true
    sleep 1

    log "Starting Tauri desktop app..."
    export TACHYON_DEBUG=1
    export TACHYON_API_URL="http://${SERVER_HOST}:${SERVER_PORT}"
    export LD_LIBRARY_PATH="/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

    rm -f /tmp/tachyon-debug.jsonl /tmp/tachyon-desktop-stdout.log /tmp/tachyon-desktop-stderr.log

    nohup "$TAURI_BIN" >/tmp/tachyon-desktop-stdout.log 2>/tmp/tachyon-desktop-stderr.log &
    echo $! > /tmp/tachyon-desktop.pid
    log "Desktop PID: $(cat /tmp/tachyon-desktop.pid)"
    log "Debug log: /tmp/tachyon-debug.jsonl"
    log "Stdout: /tmp/tachyon-desktop-stdout.log"
    log "Stderr: /tmp/tachyon-desktop-stderr.log"
}

stop_all() {
    log "Stopping all Tachyon processes..."
    pkill -f tachyon-desktop 2>/dev/null || true
    pkill -f tachyon-server 2>/dev/null || true
    fuser -k "${SERVER_PORT}/tcp" 2>/dev/null || true
    rm -f /tmp/tachyon-server.pid /tmp/tachyon-desktop.pid
    log "Stopped"
}

show_status() {
    echo "=== Tachyon Stack Status ==="
    if pg_isready -h "$PG_HOST" -p "$PG_PORT" >/dev/null 2>&1; then
        echo "PostgreSQL:  RUNNING (${PG_HOST}:${PG_PORT})"
    else
        echo "PostgreSQL:  STOPPED"
    fi
    if curl -sf "http://${SERVER_HOST}:${SERVER_PORT}/health" >/dev/null 2>&1; then
        echo "Server:      RUNNING (${SERVER_HOST}:${SERVER_PORT})"
    else
        echo "Server:      STOPPED"
    fi
    if [ -f /tmp/tachyon-desktop.pid ] && kill -0 $(cat /tmp/tachyon-desktop.pid) 2>/dev/null; then
        echo "Desktop:     RUNNING (PID $(cat /tmp/tachyon-desktop.pid))"
    else
        echo "Desktop:     STOPPED"
    fi
    echo "Logs:"
    [ -f /tmp/tachyon-server.log ] && echo "  Server:  /tmp/tachyon-server.log"
    [ -f /tmp/tachyon-debug.jsonl ] && echo "  Debug:   /tmp/tachyon-debug.jsonl"
    [ -f /tmp/tachyon-desktop-stderr.log ] && echo "  Stderr:  /tmp/tachyon-desktop-stderr.log"
}

# === Main ===
case "${1:-start}" in
    start)
        ensure_pg_running
        start_server
        start_desktop
        log "Full stack started!"
        ;;
    stop)
        stop_all
        ;;
    status)
        show_status
        ;;
    server-only)
        ensure_pg_running
        start_server
        ;;
    desktop-only)
        start_desktop
        ;;
    restart)
        stop_all
        sleep 2
        ensure_pg_running
        start_server
        start_desktop
        log "Full stack restarted!"
        ;;
    *)
        echo "Usage: $0 {start|stop|status|restart|server-only|desktop-only}"
        echo "  start         — Start PostgreSQL + Server + Desktop (default)"
        echo "  stop          — Stop all processes"
        echo "  status        — Show status of all components"
        echo "  restart       — Stop and restart everything"
        echo "  server-only   — Start only PostgreSQL + Server"
        echo "  desktop-only  — Start only Desktop (assumes server running)"
        exit 1
        ;;
esac
