#!/usr/bin/env bash
set -euo pipefail

# Tachyon staging database backup script
# Runs pg_dump inside the tachyon-postgres Docker container
# Designed for cron: 0 2 * * * /opt/tachyon/scripts/backup-db-staging.sh

BACKUP_DIR="/opt/tachyon/backups/postgres"
CONTAINER="tachyon-postgres"
DB_USER="tachyon"
DB_NAME="tachyon"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/tachyon_${DATE}.sql.gz"

mkdir -p "${BACKUP_DIR}"

echo "[$(date -Iseconds)] Starting backup: ${DB_NAME} -> ${BACKUP_FILE}"

# Run pg_dump inside the container, compress on the host
docker exec "${CONTAINER}" \
    pg_dump -U "${DB_USER}" -d "${DB_NAME}" \
    --no-owner --no-privileges --verbose 2>/dev/null \
    | gzip > "${BACKUP_FILE}"

FILESIZE=$(du -h "${BACKUP_FILE}" | cut -f1)
echo "[$(date -Iseconds)] Backup complete: ${FILESIZE}"

# Prune old backups
PRUNED=$(find "${BACKUP_DIR}" -name "tachyon_*.sql.gz" -mtime +${RETENTION_DAYS} -delete -print | wc -l)
if [ "${PRUNED}" -gt 0 ]; then
    echo "[$(date -Iseconds)] Pruned ${PRUNED} backups older than ${RETENTION_DAYS} days"
fi

# Quick integrity check: re-inflate to verify gzip is valid
if gzip -t "${BACKUP_FILE}" 2>/dev/null; then
    echo "[$(date -Iseconds)] Integrity check passed"
else
    echo "[$(date -Iseconds)] ERROR: Backup integrity check FAILED" >&2
    exit 1
fi

# Report total backup size
TOTAL_SIZE=$(du -sh "${BACKUP_DIR}" | cut -f1)
BACKUP_COUNT=$(find "${BACKUP_DIR}" -name "tachyon_*.sql.gz" | wc -l)
echo "[$(date -Iseconds)] Total: ${BACKUP_COUNT} backups, ${TOTAL_SIZE} on disk"
