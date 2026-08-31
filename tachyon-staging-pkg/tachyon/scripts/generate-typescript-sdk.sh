#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${PROJECT_ROOT}/crates/frontend/sdk"
SPEC_URL="${1:-http://localhost:8080/api/docs/openapi.json}"

echo "=== Tachyon TypeScript SDK Generator ==="
echo ""

if ! command -v openapi-typescript &>/dev/null; then
    echo "Installing openapi-typescript..."
    npm install -g openapi-typescript@latest
fi

mkdir -p "$OUTPUT_DIR"

echo "Generating TypeScript types from: $SPEC_URL"
openapi-typescript "$SPEC_URL" \
    --output "$OUTPUT_DIR/api-types.ts" \
    --export-type \
    --immutable-types \
    2>&1

if [ $? -eq 0 ]; then
    echo "Generated: $OUTPUT_DIR/api-types.ts"
    
    cat > "$OUTPUT_DIR/index.ts" << 'BAREOF'
/**
 * Tachyon API TypeScript SDK
 * Auto-generated from OpenAPI specification.
 * Do not edit manually — regenerate with: scripts/generate-typescript-sdk.sh
 */
export * from './api-types';
BAREOF
    echo "Generated: $OUTPUT_DIR/index.ts"

    SIZE=$(wc -c < "$OUTPUT_DIR/api-types.ts" | tr -d ' ')
    LINES=$(wc -l < "$OUTPUT_DIR/api-types.ts" | tr -d ' ')
    echo ""
    echo "SDK Stats:"
    echo "  File: api-types.ts"
    echo "  Size: $(numfmt --to=iec "$SIZE")"
    echo "  Lines: $LINES"
else
    echo "Failed to generate TypeScript types"
    echo "   Make sure the server is running at $SPEC_URL"
    exit 1
fi
