#!/usr/bin/env bash
# Tachyon Release Script
#
# Automates the release process:
#   1. Verify clean git status
#   2. Run all tests
#   3. Update version in Cargo.toml files
#   4. Generate changelog from conventional commits
#   5. Create git tag
#   6. Build release artifacts
#   7. Push tag and trigger CI
#
# Usage: release.sh <version> [--skip-tests] [--skip-build] [--dry-run]
#
# Example: release.sh 18.0.0
#          release.sh 18.1.0 --skip-tests --dry-run

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TACHYON_DIR="${PROJECT_ROOT}/tachyon"

VERSION=""
SKIP_TESTS=0
SKIP_BUILD=0
DRY_RUN=0

# ── Argument parsing ──────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --skip-tests) SKIP_TESTS=1; shift ;;
        --skip-build) SKIP_BUILD=1; shift ;;
        --dry-run)    DRY_RUN=1; shift ;;
        -*)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
        *)
            if [ -z "$VERSION" ]; then
                VERSION="$1"
            else
                echo "Unexpected argument: $1" >&2
                exit 1
            fi
            shift
            ;;
    esac
done

# ── Colors / Logging ──────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

_log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $*"; }
_ok()  { echo -e "${GREEN}[OK]${NC} $*"; }
_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
_err() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

die() { _err "$@"; exit 1; }

step() {
    echo ""
    echo -e "${BLUE}━━━ $1 ━━━${NC}"
}

# ── Validation ─────────────────────────────────────────────────────────────────

if [ -z "$VERSION" ]; then
    echo "Usage: release.sh <version> [--skip-tests] [--skip-build] [--dry-run]" >&2
    echo "Example: release.sh 18.0.0" >&2
    exit 1
fi

if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
    die "Invalid semver: $VERSION (expected major.minor.patch[-prerelease])"
fi

cd "$PROJECT_ROOT"

# ── Step 1: Verify clean git status ───────────────────────────────────────────

step "1/7  Verifying clean git status"

if [ "$(git status --porcelain | wc -l)" -ne 0 ]; then
    git status --short
    die "Working tree has uncommitted changes. Commit or stash first."
fi

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    _warn "Not on main branch (current: ${CURRENT_BRANCH}). Continue? [y/N]"
    read -r confirm
    [ "$confirm" = "y" ] || exit 1
fi

TAG="v${VERSION}"
if git tag -l "$TAG" | grep -q .; then
    die "Tag ${TAG} already exists"
fi

_ok "Git status clean, branch: ${CURRENT_BRANCH}"

# ── Step 2: Run tests ─────────────────────────────────────────────────────────

step "2/7  Running tests"

if [ "$SKIP_TESTS" -eq 1 ]; then
    _warn "Tests skipped (--skip-tests)"
else
    _log "Running cargo test..."
    cd "$TACHYON_DIR"

    TEST_FLAGS="--workspace --lib --exclude tachyon-testing --exclude tachyon-frontend --exclude tachyon-desktop --exclude tachyon-desktop-app"

    if cargo test $TEST_FLAGS 2>&1; then
        _ok "All tests passed"
    else
        die "Tests failed. Fix before releasing."
    fi

    cd "$PROJECT_ROOT"
fi

# ── Step 3: Update version in Cargo.toml ──────────────────────────────────────

step "3/7  Updating version to ${VERSION}"

WORKSPACE_TOML="${TACHYON_DIR}/Cargo.toml"
PREV_VERSION=$(grep '^package.version' "$WORKSPACE_TOML" | head -1 | sed 's/.*= *"\(.*\)"/\1/')
_log "Previous version: ${PREV_VERSION}"

if [ "$DRY_RUN" -eq 0 ]; then
    sed -i "s/^package\.version = \"${PREV_VERSION}\"/package.version = \"${VERSION}\"/" "$WORKSPACE_TOML"

    cd "$TACHYON_DIR"
    cargo check --workspace 2>&1 | tail -1 || true
    cd "$PROJECT_ROOT"

    _ok "Version updated to ${VERSION} in workspace Cargo.toml"
else
    _warn "[DRY-RUN] Would update version from ${PREV_VERSION} to ${VERSION}"
fi

# ── Step 4: Generate changelog ────────────────────────────────────────────────

step "4/7  Generating changelog"

PREV_TAG=$(git tag -l 'v*' --sort=-version:refname | head -1)
if [ -n "$PREV_TAG" ]; then
    COMMIT_RANGE="${PREV_TAG}..HEAD"
else
    COMMIT_RANGE="HEAD~50..HEAD"
fi

_log "Generating changelog from ${COMMIT_RANGE}"

CHANGELOG_ENTRY="## [${VERSION}] - $(date '+%Y-%m-%d')

"

# Parse conventional commits
FEAT=""
FIX=""
DOCS=""
OTHER=""

while IFS= read -r line; do
    hash=$(echo "$line" | cut -d' ' -f1)
    subject=$(echo "$line" | cut -d' ' -f2-)

    if echo "$subject" | grep -qE '^feat(\(.+\))?:'; then
        desc=$(echo "$subject" | sed 's/^feat(\{0,1\}[^)]*\{0,1\}): */- /')
        FEAT="${FEAT}${desc} (${hash})
"
    elif echo "$subject" | grep -qE '^fix(\(.+\))?:'; then
        desc=$(echo "$subject" | sed 's/^fix(\{0,1\}[^)]*\{0,1\}): */- /')
        FIX="${FIX}${desc} (${hash})
"
    elif echo "$subject" | grep -qE '^docs(\(.+\))?:'; then
        desc=$(echo "$subject" | sed 's/^docs(\{0,1\}[^)]*\{0,1\}): */- /')
        DOCS="${DOCS}${desc} (${hash})
"
    else
        desc="- ${subject} (${hash})"
        OTHER="${OTHER}${desc}
"
    fi
done < <(git log --format='%h %s' "$COMMIT_RANGE" --no-merges 2>/dev/null || true)

if [ -n "$FEAT" ]; then
    CHANGELOG_ENTRY="${CHANGELOG_ENTRY}### Added

${FEAT}
"
fi

if [ -n "$FIX" ]; then
    CHANGELOG_ENTRY="${CHANGELOG_ENTRY}### Fixed

${FIX}
"
fi

if [ -n "$DOCS" ]; then
    CHANGELOG_ENTRY="${CHANGELOG_ENTRY}### Documentation

${DOCS}
"
fi

if [ -n "$OTHER" ]; then
    CHANGELOG_ENTRY="${CHANGELOG_ENTRY}### Changed

${OTHER}
"
fi

CHANGELOG_FILE="${PROJECT_ROOT}/CHANGELOG.md"

if [ "$DRY_RUN" -eq 0 ]; then
    # Insert after the header
    HEADER_LINE=$(grep -n '^## \[' "$CHANGELOG_FILE" | head -1 | cut -d: -f1)
    if [ -n "$HEADER_LINE" ]; then
        sed -i "${HEADER_LINE}i\\${CHANGELOG_ENTRY}" "$CHANGELOG_FILE"
    else
        echo "${CHANGELOG_ENTRY}" >> "$CHANGELOG_FILE"
    fi
    _ok "Changelog updated"
else
    _warn "[DRY-RUN] Changelog entry:"
    echo "$CHANGELOG_ENTRY"
fi

# ── Step 5: Create git tag ────────────────────────────────────────────────────

step "5/7  Creating git tag ${TAG}"

if [ "$DRY_RUN" -eq 0 ]; then
    git add -A
    git commit -m "release: ${TAG}" || true
    git tag -a "$TAG" -m "Release ${TAG}"
    _ok "Tag ${TAG} created"
else
    _warn "[DRY-RUN] Would create tag ${TAG}"
fi

# ── Step 6: Build release artifacts ───────────────────────────────────────────

step "6/7  Building release artifacts"

if [ "$SKIP_BUILD" -eq 1 ]; then
    _warn "Build skipped (--skip-build)"
else
    cd "$TACHYON_DIR"

    _log "Building release binaries..."
    WORKSPACE_FLAGS="--workspace --exclude tachyon-testing --exclude tachyon-frontend --exclude tachyon-desktop --exclude tachyon-desktop-app --exclude tachyon-benchmarks"

    cargo build --release $WORKSPACE_FLAGS 2>&1 || die "Release build failed"

    _ok "Release build complete"

    # List artifacts
    _log "Artifacts:"
    ls -lh target/release/tachyon-server 2>/dev/null || true
    ls -lh target/release/tachyon-cli 2>/dev/null || true

    cd "$PROJECT_ROOT"
fi

# ── Step 7: Push tag and trigger CI ───────────────────────────────────────────

step "7/7  Pushing tag and triggering CI"

if [ "$DRY_RUN" -eq 0 ]; then
    _log "Pushing commit and tag..."
    git push origin "$CURRENT_BRANCH"
    git push origin "$TAG"
    _ok "Pushed ${TAG} to origin"
    _ok "CI pipeline should be triggered"
else
    _warn "[DRY-RUN] Would push ${TAG} to origin"
fi

# ── Summary ────────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN} Release ${TAG} complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  Version:    ${VERSION}"
echo "  Tag:        ${TAG}"
echo "  Branch:     ${CURRENT_BRANCH}"
echo "  Previous:   ${PREV_VERSION}"
echo ""
