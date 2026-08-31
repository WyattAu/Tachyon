#!/usr/bin/env bash
# Reproducible Tachyon staging deployment to the authorized CachyOS host.
# Usage: ./deploy.sh [ssh-host]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SSH_HOST="${1:-${TACHYON_STAGING_HOST:-wyatt@192.168.1.191}}"
REMOTE_ROOT="${TACHYON_STAGING_ROOT:-$HOME/tachyon-staging}"
BASE_REVISION="$(git -C "$SCRIPT_DIR" rev-parse --short HEAD)"
if git -C "$SCRIPT_DIR" diff --quiet && git -C "$SCRIPT_DIR" diff --cached --quiet; then
  REVISION="$BASE_REVISION"
else
  REVISION="${BASE_REVISION}-dirty-$(date +%s)"
fi
BUILD_LOG="${REMOTE_ROOT}/build-${REVISION}.log"
BUILD_STATUS="${REMOTE_ROOT}/build-${REVISION}.status"
RELEASE_DIR="${REMOTE_ROOT}/releases/${REVISION}"
SERVICE_FILE="${SCRIPT_DIR}/scripts/tachyon-staging.service"
APP_URL="${TACHYON_STAGING_URL:-http://192.168.1.191:8082}"

log() { printf '[deploy] %s\n' "$*"; }
die() { printf '[deploy] ERROR: %s\n' "$*" >&2; exit 1; }

command -v ssh >/dev/null || die "ssh is required"
command -v scp >/dev/null || die "scp is required"
command -v tar >/dev/null || die "tar is required"
[[ -f "$SERVICE_FILE" ]] || die "missing service unit: $SERVICE_FILE"
[[ -d "$SCRIPT_DIR/tachyon" ]] || die "run from the Tachyon repository root"

log "Checking SSH access to ${SSH_HOST}"
ssh -o ConnectTimeout=10 "$SSH_HOST" true

log "Preparing release ${REVISION} on ${SSH_HOST}"
ssh "$SSH_HOST" "mkdir -p '$RELEASE_DIR' '$REMOTE_ROOT/releases' \"\$HOME/.config/systemd/user\""

log "Packaging Tachyon workspace and local dependencies"
# Cargo.toml uses ../../{salting,tokenkit,cryptkit}; preserve that layout remotely.
tar czf - \
  --exclude='tachyon/target' \
  --exclude='tachyon/crates/frontend/dist' \
  --exclude='tachyon/node_modules' \
  --exclude='tachyon/.tachyon/search_index' \
  --exclude='tachyon/reports/*.log' \
  --exclude='.git' \
  --exclude='*/target' \
  --exclude='*/.direnv' \
  -C "$SCRIPT_DIR" tachyon \
  | ssh "$SSH_HOST" "tar xzf - -C '$RELEASE_DIR'"
tar czf - \
  --exclude='.git' \
  --exclude='*/target' \
  --exclude='*/.direnv' \
  -C "$(dirname "$SCRIPT_DIR")" salting tokenkit cryptkit \
  | ssh "$SSH_HOST" "tar xzf - -C '$REMOTE_ROOT/releases'"

log "Installing deployment environment without exposing secrets"
ssh "$SSH_HOST" "install -m 600 /tmp/tachyon-staging.env \"\$HOME/.config/tachyon-staging.env\""
ssh "$SSH_HOST" "test -s \"\$HOME/.config/tachyon-staging.env\""

log "Building release binary on CachyOS"
if ! ssh "$SSH_HOST" "test -x '$RELEASE_DIR/tachyon/target/release/tachyon-server'"; then
  BUILD_STATE="$(ssh "$SSH_HOST" "cat '$BUILD_STATUS' 2>/dev/null || true")"
  if [[ "$BUILD_STATE" == "failed" ]]; then
    ssh "$SSH_HOST" "tail -80 '$BUILD_LOG'" || true
    die "remote release build previously failed"
  fi
  if [[ "$BUILD_STATE" != "started" ]]; then
    ssh "$SSH_HOST" "rm -f '$BUILD_STATUS' '$BUILD_LOG'; printf started > '$BUILD_STATUS'; nohup bash -c 'cd \"$RELEASE_DIR/tachyon\" && cargo build --release -p tachyon-server --bin tachyon-server >\"$BUILD_LOG\" 2>&1; rc=\$?; if [ \"\$rc\" -eq 0 ]; then printf success >\"$BUILD_STATUS\"; else printf failed >\"$BUILD_STATUS\"; fi' </dev/null >/dev/null 2>&1 &"
  fi
  for attempt in $(seq 1 180); do
    if ssh "$SSH_HOST" "test -x '$RELEASE_DIR/tachyon/target/release/tachyon-server'"; then
      break
    fi
    BUILD_STATE="$(ssh "$SSH_HOST" "cat '$BUILD_STATUS' 2>/dev/null || true")"
    if [[ "$BUILD_STATE" == "failed" ]]; then
      ssh "$SSH_HOST" "tail -80 '$BUILD_LOG'" || true
      die "remote release build failed"
    fi
    sleep 2
    [[ "$attempt" -lt 180 ]] || die "remote release build did not finish; resume with ./deploy.sh"
  done
fi

log "Installing persistent user service"
scp -q "$SERVICE_FILE" "$SSH_HOST:/tmp/tachyon-staging.service"
ssh "$SSH_HOST" "install -m 644 /tmp/tachyon-staging.service \"\$HOME/.config/systemd/user/tachyon-staging.service\""
ssh "$SSH_HOST" "ln -sfn '$RELEASE_DIR' '$REMOTE_ROOT/current'"
ssh "$SSH_HOST" "systemctl --user daemon-reload"
ssh "$SSH_HOST" "systemctl --user enable tachyon-staging.service"
ssh "$SSH_HOST" "systemctl --user restart tachyon-staging.service"

log "Waiting for staging health checks"
for attempt in $(seq 1 30); do
  if curl -fsS "$APP_URL/health" >/dev/null 2>&1 \
    && curl -fsS "$APP_URL/ready" >/dev/null 2>&1 \
    && curl -fsS "$APP_URL/metrics/prometheus" >/dev/null 2>&1; then
    log "Staging is healthy: $APP_URL"
    exit 0
  fi
  sleep 2
  [[ "$attempt" -lt 30 ]] || die "staging did not become healthy; inspect systemctl --user status tachyon-staging"
done
