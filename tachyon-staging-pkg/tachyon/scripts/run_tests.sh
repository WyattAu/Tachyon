#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

cleanup() {
    log_info "Cleaning up..."
    docker compose -f "$PROJECT_ROOT/docker-compose.test.yml" down --remove-orphans 2>/dev/null || true
}
trap cleanup EXIT

run_unit_tests() {
    log_info "Running unit tests..."
    cd "$PROJECT_ROOT"
    
    cargo test --workspace --lib --no-fail-fast -- --nocapture 2>&1 | tee test-results-unit.log
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "Unit tests passed"
        return 0
    else
        log_error "Unit tests failed"
        return 1
    fi
}

run_integration_tests() {
    log_info "Running integration tests..."
    cd "$PROJECT_ROOT"
    
    if [ -z "$TEST_DATABASE_URL" ]; then
        log_warn "TEST_DATABASE_URL not set, skipping integration tests"
        log_warn "Set TEST_DATABASE_URL to run integration tests"
        return 0
    fi
    
    cargo test --workspace --test '*' --no-fail-fast -- --nocapture 2>&1 | tee test-results-integration.log
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "Integration tests passed"
        return 0
    else
        log_error "Integration tests failed"
        return 1
    fi
}

run_doc_tests() {
    log_info "Running documentation tests..."
    cd "$PROJECT_ROOT"
    
    cargo test --workspace --doc --no-fail-fast 2>&1 | tee test-results-doc.log
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "Documentation tests passed"
        return 0
    else
        log_error "Documentation tests failed"
        return 1
    fi
}

generate_coverage_report() {
    log_info "Generating coverage report..."
    cd "$PROJECT_ROOT"
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_warn "cargo-tarpaulin not installed, skipping coverage"
        log_warn "Install with: cargo install cargo-tarpaulin"
        return 0
    fi
    
    cargo tarpaulin --workspace --out Html --out Stdout --output-dir target/coverage 2>&1 | tee coverage.log
    
    if [ -f "target/coverage/index.html" ]; then
        log_success "Coverage report generated at target/coverage/index.html"
    else
        log_warn "Coverage report not generated"
    fi
}

run_e2e_tests() {
    log_info "Running E2E tests..."
    cd "$PROJECT_ROOT/e2e"
    
    if [ ! -f "package.json" ]; then
        log_warn "E2E tests not configured, skipping"
        return 0
    fi
    
    if ! command -v npm &> /dev/null; then
        log_warn "npm not installed, skipping E2E tests"
        return 0
    fi
    
    log_info "Installing E2E dependencies..."
    npm install
    
    log_info "Installing Playwright browsers..."
    npx playwright install --with-deps
    
    log_info "Running Playwright tests..."
    npm run test 2>&1 | tee ../test-results-e2e.log
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "E2E tests passed"
        return 0
    else
        log_error "E2E tests failed"
        return 1
    fi
}

run_linting() {
    log_info "Running linting checks..."
    cd "$PROJECT_ROOT"
    
    cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee lint-results.log
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "Linting passed"
        return 0
    else
        log_error "Linting failed"
        return 1
    fi
}

run_format_check() {
    log_info "Checking code formatting..."
    cd "$PROJECT_ROOT"
    
    cargo fmt --all -- --check 2>&1 | tee format-results.log
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "Code formatting check passed"
        return 0
    else
        log_error "Code formatting check failed. Run 'cargo fmt' to fix."
        return 1
    fi
}

print_summary() {
    echo ""
    echo "========================================="
    echo "           TEST SUMMARY"
    echo "========================================="
    
    if [ -f "$PROJECT_ROOT/test-results-unit.log" ]; then
        echo "Unit tests:       Completed"
    fi
    
    if [ -f "$PROJECT_ROOT/test-results-integration.log" ]; then
        echo "Integration tests: Completed"
    fi
    
    if [ -f "$PROJECT_ROOT/test-results-doc.log" ]; then
        echo "Doc tests:        Completed"
    fi
    
    if [ -f "$PROJECT_ROOT/test-results-e2e.log" ]; then
        echo "E2E tests:        Completed"
    fi
    
    if [ -f "$PROJECT_ROOT/lint-results.log" ]; then
        echo "Linting:          Completed"
    fi
    
    if [ -f "$PROJECT_ROOT/target/coverage/index.html" ]; then
        echo "Coverage:         target/coverage/index.html"
    fi
    
    echo "========================================="
}

main() {
    local run_all=false
    local run_unit=false
    local run_integration=false
    local run_doc=false
    local run_e2e=false
    local run_coverage=false
    local run_lint=false
    local run_format=false
    
    if [ $# -eq 0 ]; then
        run_all=true
    fi
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all|-a)
                run_all=true
                shift
                ;;
            --unit|-u)
                run_unit=true
                shift
                ;;
            --integration|-i)
                run_integration=true
                shift
                ;;
            --doc|-d)
                run_doc=true
                shift
                ;;
            --e2e|-e)
                run_e2e=true
                shift
                ;;
            --coverage|-c)
                run_coverage=true
                shift
                ;;
            --lint|-l)
                run_lint=true
                shift
                ;;
            --format|-f)
                run_format=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --all, -a       Run all tests (default)"
                echo "  --unit, -u      Run unit tests only"
                echo "  --integration,-i Run integration tests only"
                echo "  --doc, -d       Run documentation tests only"
                echo "  --e2e, -e       Run E2E tests only"
                echo "  --coverage, -c  Generate coverage report"
                echo "  --lint, -l      Run linting checks"
                echo "  --format, -f    Check code formatting"
                echo "  --help, -h      Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    if [ "$run_all" = true ]; then
        run_unit=true
        run_integration=true
        run_doc=true
        run_lint=true
        run_format=true
    fi
    
    local exit_code=0
    
    if [ "$run_format" = true ]; then
        run_format_check || exit_code=1
    fi
    
    if [ "$run_lint" = true ]; then
        run_linting || exit_code=1
    fi
    
    if [ "$run_unit" = true ]; then
        run_unit_tests || exit_code=1
    fi
    
    if [ "$run_integration" = true ]; then
        run_integration_tests || exit_code=1
    fi
    
    if [ "$run_doc" = true ]; then
        run_doc_tests || exit_code=1
    fi
    
    if [ "$run_coverage" = true ]; then
        generate_coverage_report
    fi
    
    if [ "$run_e2e" = true ]; then
        run_e2e_tests || exit_code=1
    fi
    
    print_summary
    
    exit $exit_code
}

main "$@"
