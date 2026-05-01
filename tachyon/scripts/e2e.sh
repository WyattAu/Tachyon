#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONTEND_DIR="$PROJECT_DIR/crates/frontend"
E2E_DIR="$PROJECT_DIR/e2e"
SERVER_PID=""

cleanup() {
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

echo "=== Tachyon E2E Test Runner ==="

# 1. Check prerequisites
echo "[1/7] Checking prerequisites..."
command -v trunk >/dev/null 2>&1 || { echo "ERROR: trunk not found. Install: cargo install trunk"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "ERROR: node not found"; exit 1; }
rustup target list | grep -q wasm32-unknown-unknown || { echo "Installing wasm32-unknown-unknown target..."; rustup target add wasm32-unknown-unknown; }

# 2. Build frontend
echo "[2/7] Building WASM frontend..."
cd "$FRONTEND_DIR"
trunk build --release 2>&1 | tail -3
echo "Frontend built to $FRONTEND_DIR/dist/"

# 3. Start PostgreSQL
echo "[3/7] Ensuring PostgreSQL is running..."
if ! docker ps | grep -q tachyon-pg; then
    docker run -d --name tachyon-pg \
        -e POSTGRES_USER=tachyon_test \
        -e POSTGRES_PASSWORD=tachyon_test \
        -e POSTGRES_DB=tachyon_test \
        -p 5433:5432 \
        postgres:16-alpine >/dev/null
    echo "Waiting for PostgreSQL..."
    sleep 3
fi

# 4. Start server
echo "[4/7] Starting server..."
cd "$PROJECT_DIR"
DATABASE_URL=postgres://tachyon_test:tachyon_test@127.0.0.1:5433/tachyon_test \
TACHYON_STATIC_DIR=crates/frontend/dist \
TACHYON_PORT=8080 \
TACHYON_JWT_SECRET=e2e-test-secret-key-must-be-at-least-32-chars-long \
RUST_LOG=warn \
cargo run -p tachyon-server 2>&1 &
SERVER_PID=$!

# 5. Wait for health
echo "[5/7] Waiting for server health..."
for i in $(seq 1 60); do
    if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
        echo "Server is healthy"
        break
    fi
    if [ $i -eq 60 ]; then
        echo "ERROR: Server failed to start"
        exit 1
    fi
    sleep 1
done

# 6. Install Playwright deps
echo "[6/7] Installing Playwright dependencies..."
cd "$E2E_DIR"
if [ ! -d "node_modules" ]; then
    npm install 2>&1 | tail -3
fi
npx playwright install chromium 2>&1 | tail -3

# 7. Run tests
echo "[7/7] Running E2E tests..."
E2E_BASE_URL=http://localhost:8080 npx playwright test --reporter=list 2>&1
EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "=== E2E Tests: PASSED ==="
else
    echo "=== E2E Tests: FAILED (exit code $EXIT_CODE) ==="
fi

exit $EXIT_CODE
