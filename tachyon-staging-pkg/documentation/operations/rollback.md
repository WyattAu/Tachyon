---
title: Automated Rollback Procedures
description: Automated deployment rollback strategies for Tachyon
date: 2026-05-23
category: operations
order: 2
---

# Automated Rollback Procedures

Automated rollback mechanisms for Tachyon deployments.

## Rollback Triggers

| Trigger | Condition | Action |
|---------|-----------|--------|
| Health check failure | `/health` returns non-200 for 3 consecutive checks | Rollback to previous image |
| Error rate spike | >10% 5xx errors over 2-minute window | Rollback + alert |
| Response time degradation | p99 > 5s for 5 consecutive minutes | Rollback + alert |
| Database migration failure | Migration returns non-zero exit | Block deploy, alert |
| Container crash loop | Container restarts >3 times in 5 minutes | Rollback to previous image |

## Docker Compose Rollback

```bash
#!/bin/bash
# rollback.sh — Automated rollback to previous deployment
set -euo pipefail

DEPLOY_DIR="/opt/tachyon"
BACKUP_DIR="/opt/tachyon/backups"
CURRENT_VERSION=$(cat "$DEPLOY_DIR/.current-version")
PREVIOUS_VERSION=$(cat "$DEPLOY_DIR/.previous-version")

echo "Rolling back from $CURRENT_VERSION to $PREVIOUS_VERSION"

# 1. Tag current as failed
echo "$CURRENT_VERSION" > "$BACKUP_DIR/.failed-version"
echo "$(date -Iseconds) ROLLBACK $CURRENT_VERSION -> $PREVIOUS_VERSION" >> "$BACKUP_DIR/rollback.log"

# 2. Update docker-compose to previous image
cd "$DEPLOY_DIR"
sed -i "s|image:.*tachyon:.*|image: ghcr.io/wyattau/tachyon:$PREVIOUS_VERSION|" docker-compose.yml

# 3. Rollback database if migration backup exists
MIGRATION_BACKUP="$BACKUP_DIR/pre-migration-$CURRENT_VERSION.sql.gz"
if [ -f "$MIGRATION_BACKUP" ]; then
    echo "Restoring database from $MIGRATION_BACKUP"
    gunzip -c "$MIGRATION_BACKUP" | docker compose exec -T db psql -U tachyon tachyon
fi

# 4. Restart with previous version
docker compose down
docker compose up -d

# 5. Verify health
echo "Waiting for health check..."
for i in $(seq 1 30); do
    if curl -sf http://localhost:8080/health | grep -q "ok"; then
        echo "Rollback successful: $PREVIOUS_VERSION is healthy"
        echo "$PREVIOUS_VERSION" > "$DEPLOY_DIR/.current-version"
        exit 0
    fi
    sleep 2
done

echo "CRITICAL: Rollback failed — $PREVIOUS_VERSION is not healthy"
exit 1
```

## CI/CD Rollback Gate

The CI pipeline includes an automatic rollback gate after deployment:

```yaml
# In deploy stage
- name: Verify Deployment
  run: |
    for i in $(seq 1 20); do
      if curl -sf $HEALTH_ENDPOINT | grep -q '"status":"ok"'; then
        echo "Deployment verified"
        exit 0
      fi
      sleep 5
    done
    echo "Deployment verification failed — triggering rollback"
    ./scripts/rollback.sh
    exit 1
```

## Database Migration Rollback

Each migration must have a corresponding down migration:

```bash
# Pre-migration backup (run before deploy)
pg_dump -Fc tachyon | gzip > "backup-pre-$VERSION.sql.gz"

# Rollback specific migration
sqlx migrate revert
```

## Configuration Rollback

Configuration is version-controlled. Rollback:

```bash
git log --oneline -5 -- docker-compose.yml .env.production
git checkout HEAD~1 -- docker-compose.yml .env.production
docker compose up -d
```

## Post-Rollback Checklist

- [ ] Health endpoint returns 200
- [ ] Error rate < 1% for 10 minutes
- [ ] Database connectivity verified
- [ ] No new error patterns in logs
- [ ] Stakeholders notified
- [ ] Incident ticket created for root cause analysis
