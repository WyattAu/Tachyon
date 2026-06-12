#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/e2e-results"
SCREENSHOTS_DIR="$RESULTS_DIR/screenshots"
REPORT="$RESULTS_DIR/e2e-report.json"
PORT=${PORT:-8080}
HOST=${HOST:-192.168.1.191}
BASE_URL="http://${HOST}:${PORT}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[E2E]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if server is running
check_server() {
  log "Checking server at $BASE_URL..."
  if curl -s -o /dev/null -w "%{http_code}" "$BASE_URL" | grep -qE "^(200|301|302|404)$"; then
    log "Server is running"
    return 0
  fi
  warn "Server not responding, attempting to start..."
  return 1
}

# Try to start the server
start_server() {
  # Check for docker-compose
  if [ -f "$SCRIPT_DIR/docker-compose.yml" ]; then
    log "Starting server via docker-compose..."
    cd "$SCRIPT_DIR"
    docker-compose up -d tachyon 2>/dev/null || docker compose up -d tachyon 2>/dev/null || true
    sleep 5
    
    # Wait for server
    for i in $(seq 1 30); do
      if curl -s -o /dev/null -w "%{http_code}" "$BASE_URL" | grep -qE "^(200|301|302|404)$"; then
        log "Server started successfully"
        return 0
      fi
      sleep 2
    done
    err "Failed to start server"
    return 1
  fi

  # Check for binary
  if [ -f "$SCRIPT_DIR/tachyon/tachyon-server" ]; then
    log "Starting server binary..."
    cd "$SCRIPT_DIR"
    ./tachyon/tachyon-server &
    SERVER_PID=$!
    sleep 3
    
    for i in $(seq 1 15); do
      if curl -s -o /dev/null -w "%{http_code}" "$BASE_URL" | grep -qE "^(200|301|302|404)$"; then
        log "Server started successfully"
        return 0
      fi
      sleep 2
    done
    err "Failed to start server"
    return 1
  fi

  err "No server found to start"
  return 1
}

# Cleanup
cleanup() {
  if [ -n "$SERVER_PID" ]; then
    log "Stopping server (PID: $SERVER_PID)..."
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# Main
main() {
  log "=== Tachyon E2E Test Runner ==="
  log "Target: $BASE_URL"
  log ""

  # Ensure results dir exists
  mkdir -p "$SCREENSHOTS_DIR"

  # Check/start server
  if ! check_server; then
    start_server
  fi

  # Verify Playwright is installed
  if ! node -e "require('playwright')" 2>/dev/null; then
    log "Installing Playwright..."
    npm install playwright 2>/dev/null
    npx playwright install chromium 2>/dev/null
  fi

  # Run E2E tests
  log "Running E2E tests..."
  log ""
  
  TACHYON_URL="$BASE_URL" node "$SCRIPT_DIR/e2e-test.js"
  EXIT_CODE=$?

  # Display results
  log ""
  log "=== Results ==="
  
  if [ -f "$REPORT" ]; then
    node -e "
      const r = require('$REPORT');
      console.log('Tests: ' + r.summary.total);
      console.log('Passed: \x1b[32m' + r.summary.passed + '\x1b[0m');
      console.log('Failed: \x1b[31m' + r.summary.failed + '\x1b[0m');
      console.log('');
      for (const t of r.results) {
        const icon = t.status === 'pass' ? '\x1b[32m✓\x1b[0m' : '\x1b[31m✗\x1b[0m';
        console.log(icon + ' ' + t.test + (t.error ? ' - ' + t.error.substring(0, 80) : ''));
      }
    "
  fi

  log ""
  log "Screenshots: $SCREENSHOTS_DIR"
  log "Full report: $REPORT"

  if [ $EXIT_CODE -ne 0 ]; then
    err "Some tests failed"
    exit 1
  fi

  log "All tests passed!"
}

main "$@"
