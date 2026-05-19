#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ $# -eq 0 ]; then
    TARGETS=("$ROOT_DIR/docs" "$ROOT_DIR/documentation")
else
    TARGETS=("$@")
fi

DRY_RUN="${DRY_RUN:-0}"
SED_CMD="sed -i"
if [ "$DRY_RUN" = "1" ]; then
    SED_CMD="sed"
    echo "=== DRY RUN MODE ==="
fi

count=0

for target in "${TARGETS[@]}"; do
    [ -d "$target" ] || { echo "Skipping non-existent directory: $target"; continue; }

    while IFS= read -r -d '' file; do
        changed=0

        if grep -qE '\]\(\.specs/01_standards/|\]\(\.specs/02_adrs/|\]\(\.specs/04_future_state/|\]\(\.specs/08_roadmap/|\]\(\.specs/' "$file"; then
            $SED_CMD -E 's|\]\(\.specs/01_standards/|\]\(.adrs/|g; s|\]\(\.specs/02_adrs/|\]\(.adrs/|g; s|\]\(\.specs/04_future_state/|\]\(.adrs/|g; s|\]\(\.specs/08_roadmap/|\]\(.adrs/|g; s|\]\(\.specs/|\]\(.adrs/|g' "$file"
            changed=1
        fi

        if grep -q '\.\./specs/' "$file"; then
            $SED_CMD 's|\.\./specs/|../.adrs/|g' "$file"
            changed=1
        fi

        if grep -q '\.\./\.\./specs/' "$file"; then
            $SED_CMD 's|\.\./\.\./specs/|../../.adrs/|g' "$file"
            changed=1
        fi

        if grep -q '\.docs/' "$file"; then
            $SED_CMD 's|\.docs/|docs/|g' "$file"
            changed=1
        fi

        if grep -q '\.\./user-guide/' "$file"; then
            $SED_CMD 's|\.\./user-guide/|../user/|g' "$file"
            changed=1
        fi

        if [ "$changed" -eq 1 ]; then
            echo "Fixed: $file"
            count=$((count + 1))
        fi

    done < <(find "$target" -name '*.md' -print0 | sort -z)
done

echo "Done. Fixed $count file(s)."
