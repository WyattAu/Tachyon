#!/usr/bin/env bash
# Roll back the Tachyon staging service to a previously built release.
# Usage: ./scripts/tachyon-staging-rollback.sh [ssh-host] [revision]

set -euo pipefail

SSH_HOST="${1:-${TACHYON_STAGING_HOST:-wyatt@192.168.1.191}}"
REVISION="${2:-}"
REMOTE_ROOT="${TACHYON_STAGING_ROOT:-$HOME/tachyon-staging}"
APP_URL="${TACHYON_STAGING_URL:-http://192.168.1.191:8082}"

log() { printf '[rollback] %s\n' "$*"; }
die() { printf '[rollback] ERROR: %s\n' "$*" >&2; exit 1; }

command -v ssh >/dev/null || die "ssh is required"
[[ -n "$REVISION" ]] || die "usage: $0 [ssh-host] revision"

RELEASE_DIR="$REMOTE_ROOT/releases/$REVISION"

log "Checking release ${REVISION} on ${SSH_HOST}"
ssh -o ConnectTimeout=10 "$SSH_HOST" "test -x '$RELEASE_DIR/tachyon/target/release/tachyon-server'"

log "Switching current release"
ssh "$SSH_HOST" "ln -sfn '$RELEASE_DIR' '$REMOTE_ROOT/current'; systemctl --user daemon-reload; systemctl --user restart tachyon-staging.service"

log "Waiting for staging health checks"
for attempt in $(seq 1 30); do
  if curl -fsS "$APP_URL/health" >/dev/null 2>&1 \
    && curl -fsS "$APP_URL/ready" >/dev/null 2>&1 \
    && curl -fsS "$APP_URL/metrics/prometheus" >/dev/null 2>&1; then
    log "Rollback is healthy: $APP_URL -> $REVISION"
    exit 0
  fi
  sleep 2
done

die "rollback did not become healthy; inspect systemctl --user status tachyon-staging"
