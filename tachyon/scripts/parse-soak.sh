#!/usr/bin/env bash
# Parse soak test metrics CSV and produce percentile report
CSV="/opt/tachyon/backups/soak-test/metrics-20260901_020727.csv"

echo "=== SOAK TEST RESULTS ==="
echo "Total requests: $(tail -n +2 "$CSV" | wc -l)"
echo ""

# Status code distribution
echo "Status code distribution:"
tail -n +2 "$CSV" | awk -F',' '{print $3}' | sort | uniq -c | sort -rn
echo ""

# Response time percentiles
echo "Response time percentiles (ms):"
tail -n +2 "$CSV" | awk -F',' '$4 > 0 {print $4}' | sort -n | awk '
{a[NR]=$1; sum+=$1; if($1>max) max=$1}
END {
    if(NR>0) {
        printf "  Count: %d\n", NR
        printf "  Min:   %dms\n", a[1]
        printf "  p50:   %dms\n", a[int(NR*0.5)]
        printf "  p90:   %dms\n", a[int(NR*0.9)]
        printf "  p95:   %dms\n", a[int(NR*0.95)]
        printf "  p99:   %dms\n", a[int(NR*0.99)]
        printf "  Max:   %dms\n", max
        printf "  Avg:   %dms\n", sum/NR
    }
}'
echo ""

# Per-endpoint breakdown
echo "Per-endpoint response times (avg ms):"
tail -n +2 "$CSV" | awk -F',' '{sum[$2]+=$4; cnt[$2]++} END {for(ep in sum) printf "  %-50s avg=%dms (n=%d)\n", ep, sum[ep]/cnt[ep], cnt[ep]}' | sort -t= -k2 -n -r
echo ""

# Error analysis
ERRORS=$(tail -n +2 "$CSV" | awk -F',' '$3 >= 500 || $3 == 000' | wc -l)
echo "Errors (5xx or connection failed): ${ERRORS}"

# Slow requests
SLOW=$(tail -n +2 "$CSV" | awk -F',' '$4 > 2000' | wc -l)
echo "Slow requests (>2s): ${SLOW}"

# Check for any errors in log
LOG=$(ls /opt/tachyon/backups/soak-test/soak-*.log 2>/dev/null | head -1)
if [ -n "$LOG" ]; then
    ERR_LINES=$(grep -c "ERROR\|SLOW" "$LOG" 2>/dev/null || echo 0)
    echo "Error/slow lines in log: ${ERR_LINES}"
fi
