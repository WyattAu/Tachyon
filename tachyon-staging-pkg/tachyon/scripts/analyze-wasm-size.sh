#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TACHYON_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONTEND_DIR="$TACHYON_DIR/crates/frontend"
DIST_DIR="$FRONTEND_DIR/dist"
WASM_FILE="$DIST_DIR/tachyon_frontend_bg.wasm"

export TMPDIR=/tmp

echo "=== Tachyon WASM Size Analysis ==="
echo ""

if [ ! -f "$WASM_FILE" ]; then
    echo "WASM file not found at $WASM_FILE"
    echo "Run scripts/build-wasm.sh first"
    exit 1
fi

WASM_SIZE=$(stat -c%s "$WASM_FILE")
echo "WASM file: $WASM_FILE ($(numfmt --to=iec "$WASM_SIZE"))"
echo ""

echo "--- wasm-opt size analysis ---"
if command -v wasm-opt &>/dev/null; then
    echo "Top 20 largest functions:"
    wasm-opt --metrics --metrics-json "$WASM_FILE" 2>/dev/null | \
        python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    metrics = data.get('code_size', [])
    metrics.sort(key=lambda x: x.get('bytes', 0) if isinstance(x, dict) else 0, reverse=True)
    for i, m in enumerate(metrics[:20]):
        if isinstance(m, dict):
            name = m.get('name', '?')
            b = m.get('bytes', 0)
            print(f'  {i+1:2d}. {name:<60s} {b:>10,} bytes')
except Exception:
    print('  Could not parse wasm-opt metrics')
" 2>/dev/null || echo "  Could not parse wasm-opt output"
else
    echo "  wasm-opt not found (install binaryen)"
fi

echo ""
echo "--- cargo bloat analysis ---"
if command -v cargo-bloat &>/dev/null; then
    echo "Top 20 largest items (uncompressed .wasm section sizes):"
    cd "$TACHYON_DIR"
    TMPDIR=/tmp cargo bloat --release \
        --target wasm32-unknown-unknown \
        -p tachyon-frontend \
        --crates \
        --sorted \
        -n 20 2>/dev/null || echo "  cargo bloat failed"
else
    echo "  cargo-bloat not found (install with: cargo install cargo-bloat)"
    echo "  Run: cargo install cargo-bloat && TMPDIR=/tmp cargo bloat --release --target wasm32-unknown-unknown -p tachyon-frontend --crates -n 20"
fi

echo ""
echo "--- Total bundle breakdown ---"
if [ -d "$DIST_DIR" ]; then
    TOTAL=0
    while IFS= read -r -d '' file; do
        FILE_SIZE=$(stat -c%s "$file")
        TOTAL=$((TOTAL + FILE_SIZE))
    done < <(find "$DIST_DIR" -type f -print0)

    echo "  Total: $(numfmt --to=iec "$TOTAL")"
    echo ""
    echo "  By file type:"
    find "$DIST_DIR" -type f | sed 's/.*\.//' | sort | uniq -c | sort -rn | head -10 | while read -r count ext; do
        TYPE_SIZE=0
        while IFS= read -r -d '' file; do
            FILE_SIZE=$(stat -c%s "$file")
            TYPE_SIZE=$((TYPE_SIZE + FILE_SIZE))
        done < <(find "$DIST_DIR" -type f -name "*.$ext" -print0 2>/dev/null)
        printf "    .%-12s %3d files  %s\n" "$ext" "$count" "$(numfmt --to=iec "$TYPE_SIZE")"
    done
fi

echo ""
echo "--- Optimization suggestions ---"
echo "  - Remove unused web-sys features in crates/frontend/Cargo.toml"
echo "  - Use lazy_static or once_cell for heavy initializations"
echo "  - Consider feature-gating debug-only code behind [cfg(debug_assertions)]"
echo "  - Run 'wasm-opt -Oz' for additional size reduction"
echo "  - Profile with 'wasm-pack build --target web --release' for granular analysis"
echo ""
echo "=== Analysis complete ==="
