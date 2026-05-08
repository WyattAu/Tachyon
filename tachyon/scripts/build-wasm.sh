#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TACHYON_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONTEND_DIR="$TACHYON_DIR/crates/frontend"
DIST_DIR="$FRONTEND_DIR/dist"
WASM_FILE="$DIST_DIR/tachyon_frontend.js"
SIZE_THRESHOLD="${WASM_SIZE_THRESHOLD:-5242880}"

export TMPDIR=/tmp

echo "=== Tachyon WASM Build ==="
echo "Working directory: $FRONTEND_DIR"
echo "Size threshold: $(numfmt --to=iec "$SIZE_THRESHOLD")"
echo ""

echo "[1/4] Building WASM with trunk..."
cd "$FRONTEND_DIR"
TMPDIR=/tmp trunk build --release --public-url "${PUBLIC_URL:-/}"

echo ""
echo "[2/4] Running wasm-opt (if available)..."
if command -v wasm-opt &>/dev/null; then
    WASM_OUTPUT="$DIST_DIR/tachyon_frontend_bg.wasm"
    if [ -f "$WASM_OUTPUT" ]; then
        ORIGINAL_SIZE=$(stat -c%s "$WASM_OUTPUT")
        wasm-opt -Oz -o "$WASM_OUTPUT.opt" "$WASM_OUTPUT" 2>/dev/null && \
            mv "$WASM_OUTPUT.opt" "$WASM_OUTPUT" && \
            OPTIMIZED_SIZE=$(stat -c%s "$WASM_OUTPUT") && \
            SAVED=$((ORIGINAL_SIZE - OPTIMIZED_SIZE)) && \
            echo "  wasm-opt saved $(numfmt --to=iec "$SAVED") ($(numfmt --to=iec "$ORIGINAL_SIZE") -> $(numfmt --to=iec "$OPTIMIZED_SIZE"))"
    else
        echo "  No .wasm file found at $WASM_OUTPUT, skipping wasm-opt"
    fi
else
    echo "  wasm-opt not found (install binaryen for additional size reduction)"
fi

echo ""
echo "[3/4] Bundle size report..."
TOTAL_SIZE=0
if [ -d "$DIST_DIR" ]; then
    echo "  File sizes:"
    while IFS= read -r -d '' file; do
        FILE_SIZE=$(stat -c%s "$file")
        TOTAL_SIZE=$((TOTAL_SIZE + FILE_SIZE))
        REL_PATH="${file#$DIST_DIR/}"
        printf "    %-50s %s\n" "$REL_PATH" "$(numfmt --to=iec "$FILE_SIZE")"
    done < <(find "$DIST_DIR" -type f -print0 | sort -z)
fi

echo ""
echo "[4/4] Size check..."
echo "  Total bundle size: $(numfmt --to=iec "$TOTAL_SIZE") ($TOTAL_SIZE bytes)"
echo "  Threshold:         $(numfmt --to=iec "$SIZE_THRESHOLD") ($SIZE_THRESHOLD bytes)"

if [ "$TOTAL_SIZE" -gt "$SIZE_THRESHOLD" ]; then
    EXCEEDED=$((TOTAL_SIZE - SIZE_THRESHOLD))
    echo "  FAIL: Bundle exceeds threshold by $(numfmt --to=iec "$EXCEEDED")"
    exit 1
else
    REMAINING=$((SIZE_THRESHOLD - TOTAL_SIZE))
    echo "  PASS: $(numfmt --to=iec "$REMAINING") under threshold"
fi

echo ""
echo "=== Build complete ==="
