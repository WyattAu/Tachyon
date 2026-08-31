#!/bin/bash
# Deploy latest Tachyon to server
# Run from repo root: bash deploy.sh

SERVER="wyatt@192.168.1.191"
REMOTE_DIR="/home/wyatt/tachyon-server"

echo "=== Deploying Tachyon to ${SERVER} ==="

# 1. Sync source code
echo "[1/5] Syncing source code..."
rsync -avz --exclude target --exclude node_modules --exclude .git --exclude traversal-results \
  tachyon/ ${SERVER}:${REMOTE_DIR}/

# 2. Build server binary on remote
echo "[2/5] Building server binary (this may take a few minutes)..."
ssh ${SERVER} "cd ${REMOTE_DIR} && cargo build --release -p tachyon-server"

# 3. Build WASM frontend on remote
echo "[3/5] Building WASM frontend..."
ssh ${SERVER} "cd ${REMOTE_DIR}/crates/frontend && ~/.cargo/bin/trunk build --release"

# 4. Restart server
echo "[4/5] Restarting server..."
ssh ${SERVER} "killall -9 tachyon-server 2>/dev/null; sleep 2; \
  DATABASE_URL='postgres://tachyon:tachyon@localhost:5434/tachyon' \
  TACHYON_JWT_SECRET='deploy-secret-key-for-cachyos-server-at-least-64-chars-long-for-production-use' \
  TACHYON_JWT_SECRETS='deploy-secret-key-for-cachyos-server-at-least-64-chars-long-for-production-use' \
  TACHYON_SERVER_HOST=0.0.0.0 \
  TACHYON_SERVER_PORT=8080 \
  RUST_LOG=info \
  TACHYON_ADMIN_PASSWORD=admin123 \
  TACHYON_STATIC_DIR=${REMOTE_DIR}/crates/frontend/dist \
  nohup ${REMOTE_DIR}/target/release/tachyon-server > /tmp/tachyon-server.log 2>&1 &"

# 5. Verify
echo "[5/5] Verifying server health..."
sleep 5
ssh ${SERVER} "curl -sf http://localhost:8080/health | python3 -m json.tool"

echo ""
echo "=== Deployment complete ==="
echo "Server: http://192.168.1.191:8080"
echo "Health: http://192.168.1.191:8080/health"
