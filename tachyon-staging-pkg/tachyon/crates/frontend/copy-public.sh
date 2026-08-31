#!/usr/bin/env bash
# Post-trunk-build hook: copies public/ assets to dist/.
# Trunk doesn't auto-copy public/ on `trunk build` — only on `trunk serve`.
# Usage: ./copy-public.sh [frontend-dir]
set -euo pipefail

FRONTEND_DIR="${1:-$(dirname "$0")}"
DIST_DIR="$FRONTEND_DIR/dist"
PUBLIC_DIR="$FRONTEND_DIR/public"

if [ ! -d "$PUBLIC_DIR" ]; then
    echo "No public/ directory found at $PUBLIC_DIR"
    exit 0
fi

if [ ! -d "$DIST_DIR" ]; then
    echo "No dist/ directory found at $DIST_DIR"
    exit 1
fi

# Copy all files from public/ to dist/ (skip directories that already exist)
rsync -a --ignore-existing "$PUBLIC_DIR/" "$DIST_DIR/"
echo "Copied public/ assets to dist/"
