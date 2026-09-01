#!/usr/bin/env bash
# Tachyon 24-hour soak test
# Hits API endpoints at varying intervals, logs response times and errors
# Run: ./soak-test.sh [BASE_URL] [DURATION_HOURS]
set -uo pipefail

BASE_URL="${1:-http://localhost:8082}"
DURATION_HOURS="${2:-24}"
END_TIME=$(( $(date +%s) + DURATION_HOURS * 3600 ))
LOG_DIR="/opt/tachyon/backups/soak-test"
mkdir -p "${LOG_DIR}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="${LOG_DIR}/soak-${TIMESTAMP}.log"
METRICS_FILE="${LOG_DIR}/metrics-${TIMESTAMP}.csv"

# CSV header
echo "timestamp,endpoint,status_code,response_time_ms" > "${METRICS_FILE}"

endpoints=(
    "/health"
    "/ready"
    "/api/v1/documents?page=1&limit=10"
    "/api/v1/documents?page=1&limit=50"
    "/api/v1/search?q=test"
    "/metrics/prometheus"
    "/graphql"
)

echo "[$(date -Iseconds)] Starting soak test: ${BASE_URL} for ${DURATION_HOURS}h" | tee -a "${LOG_FILE}"
echo "Log: ${LOG_FILE}" | tee -a "${LOG_FILE}"
echo "Metrics CSV: ${METRICS_FILE}" | tee -a "${LOG_FILE}"

REQUEST_COUNT=0
ERROR_COUNT=0
SLOW_COUNT=0

while [ "$(date +%s)" -lt "${END_TIME}" ]; do
    for ep in "${endpoints[@]}"; do
        START_NS=$(date +%s%N)
        HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}${ep}" 2>/dev/null || echo "000")
        END_NS=$(date +%s%N)

        ELAPSED_MS=$(( (END_NS - START_NS) / 1000000 ))

        REQUEST_COUNT=$((REQUEST_COUNT + 1))

        echo "$(date -Iseconds),${ep},${HTTP_CODE},${ELAPSED_MS}" >> "${METRICS_FILE}"

        if [ "${HTTP_CODE}" = "000" ]; then
            ERROR_COUNT=$((ERROR_COUNT + 1))
            echo "[$(date -Iseconds)] ERROR: ${ep} -> connection failed" >> "${LOG_FILE}"
        elif [ "${HTTP_CODE}" -ge 500 ]; then
            ERROR_COUNT=$((ERROR_COUNT + 1))
            echo "[$(date -Iseconds)] ERROR: ${ep} -> ${HTTP_CODE}" >> "${LOG_FILE}"
        fi

        if [ "${ELAPSED_MS}" -gt 2000 ]; then
            SLOW_COUNT=$((SLOW_COUNT + 1))
            echo "[$(date -Iseconds)] SLOW: ${ep} -> ${ELAPSED_MS}ms (${HTTP_CODE})" >> "${LOG_FILE}"
        fi

        # Check server health every 100 requests
        if [ $((REQUEST_COUNT % 100)) -eq 0 ]; then
            CONNS=$(ss -tnp 2>/dev/null | grep -c tachyon || echo "N/A")
            echo "[$(date -Iseconds)] CHECKPOINT #${REQUEST_COUNT}: conns=${CONNS} errors=${ERROR_COUNT} slow=${SLOW_COUNT}" | tee -a "${LOG_FILE}"
        fi

        # Random delay 100ms-2s
        DELAY_MS=$(( (RANDOM % 1900) + 100 ))
        sleep "$(awk "BEGIN{printf \"%.3f\", ${DELAY_MS}/1000}")"
    done
done

echo "" | tee -a "${LOG_FILE}"
echo "[$(date -Iseconds)] === SOAK TEST COMPLETE ===" | tee -a "${LOG_FILE}"
echo "Total requests: ${REQUEST_COUNT}" | tee -a "${LOG_FILE}"
echo "Total errors: ${ERROR_COUNT}" | tee -a "${LOG_FILE}"
echo "Slow requests (>2s): ${SLOW_COUNT}" | tee -a "${LOG_FILE}"

if [ "${REQUEST_COUNT}" -gt 0 ]; then
    echo "" | tee -a "${LOG_FILE}"
    echo "Response time distribution:" | tee -a "${LOG_FILE}"
    awk -F',' 'NR>1 && $4>0 {print $4}' "${METRICS_FILE}" | sort -n | awk '
    {a[NR]=$1; sum+=$1}
    END {
        if(NR>0) {
            printf "  p50: %dms\n  p90: %dms\n  p99: %dms\n  max: %dms\n  avg: %dms\n",
                a[int(NR*0.5)], a[int(NR*0.9)], a[int(NR*0.99)], a[NR], sum/NR
        }
    }' | tee -a "${LOG_FILE}"
fi
