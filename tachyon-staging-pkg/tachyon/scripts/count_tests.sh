#!/usr/bin/env bash
# Count total tests across the workspace and update VERSION.md
set -euo pipefail

cd "$(dirname "$0")"

# Count tests by running --list and counting lines
TOTAL=$(cargo test --workspace -- --list 2>/dev/null | grep -c ':: ' || echo "0")

echo "Total tests: $TOTAL"

# Update VERSION.md
if [ -f VERSION.md ]; then
    sed -i "s/[0-9]\{1,\} tests/${TOTAL} tests/g" VERSION.md
    sed -i "s/[0-9]\{1,\}\+ tests/${TOTAL}+ tests/g" VERSION.md
    echo "Updated VERSION.md with ${TOTAL} tests"
fi
