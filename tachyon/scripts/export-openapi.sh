#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${PROJECT_ROOT}/docs/openapi"

SPEC_URL="${1:-http://localhost:8080/api/docs/openapi.json}"

echo "=== Tachyon OpenAPI Spec Export ==="

mkdir -p "$OUTPUT_DIR"

if command -v curl &>/dev/null; then
    HTTP_CODE=$(curl -s -o "$OUTPUT_DIR/openapi.json" -w "%{http_code}" "$SPEC_URL" 2>/dev/null || echo "000")
    if [ "$HTTP_CODE" = "200" ]; then
        echo "Downloaded OpenAPI spec from $SPEC_URL"
        
        if command -v python3 &>/dev/null; then
            python3 -c "
import json, yaml, sys
with open('$OUTPUT_DIR/openapi.json') as f:
    spec = json.load(f)
with open('$OUTPUT_DIR/openapi.yaml', 'w') as f:
    yaml.dump(spec, f, default_flow_style=False, sort_keys=False)
" 2>/dev/null && echo "Generated openapi.yaml"
        fi
        
        ENDPOINTS=$(python3 -c "
import json, sys
with open('$OUTPUT_DIR/openapi.json') as f:
    spec = json.load(f)
paths = spec.get('paths', {})
print(len(paths))
" 2>/dev/null || echo "?")
        echo "  Endpoints: $ENDPOINTS"
        
        exit 0
    fi
fi

echo "Server not reachable at $SPEC_URL"
echo "   OpenAPI spec will be generated at server startup time."
echo "   To export, start the server first: cargo run --bin tachyon-server"
exit 1
