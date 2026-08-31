#!/bin/bash

# Tachyon Test Runner Script
# Runs all test suites: unit, integration, and E2E

set -e

echo "=== Tachyon Test Suite Runner ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_E2E=false
RUN_COVERAGE=false

for arg in "$@"; do
    case $arg in
        --unit-only)
            RUN_INTEGRATION=false
            RUN_E2E=false
            ;;
        --integration-only)
            RUN_UNIT=false
            RUN_E2E=false
            ;;
        --e2e-only)
            RUN_UNIT=false
            RUN_INTEGRATION=false
            RUN_E2E=true
            ;;
        --with-e2e)
            RUN_E2E=true
            ;;
        --coverage)
            RUN_COVERAGE=true
            ;;
        --all)
            RUN_E2E=true
            RUN_COVERAGE=true
            ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 [--unit-only|--integration-only|--e2e-only|--with-e2e|--coverage|--all]"
            exit 1
            ;;
    esac
done

# Function to run backend tests
run_backend_tests() {
    echo -e "${YELLOW}Running Backend Tests...${NC}"
    
    # Unit tests
    if [ "$RUN_UNIT" = true ]; then
        echo "  → Unit tests..."
        cargo test --lib --all-features --no-fail-fast -- --test-threads=4
    fi
    
    # Integration tests
    if [ "$RUN_INTEGRATION" = true ]; then
        echo "  → Integration tests..."
        cargo test --test '*' --all-features --no-fail-fast -- --test-threads=2
    fi
    
    echo -e "${GREEN}Backend tests complete!${NC}"
}

# Function to run frontend tests
run_frontend_tests() {
    echo -e "${YELLOW}Running Frontend Tests...${NC}"
    
    if command -v wasm-pack &> /dev/null; then
        echo "  → WASM tests (Chrome)..."
        cd tachyon/crates/frontend
        wasm-pack test --headless --chrome || true
        
        echo "  → WASM tests (Firefox)..."
        wasm-pack test --headless --firefox || true
        
        cd ../..
        echo -e "${GREEN}Frontend tests complete!${NC}"
    else
        echo -e "${YELLOW}wasm-pack not found, skipping frontend tests${NC}"
        echo "Install with: cargo install wasm-pack"
    fi
}

# Function to run E2E tests
run_e2e_tests() {
    if [ "$RUN_E2E" = true ]; then
        echo -e "${YELLOW}Running E2E Tests...${NC}"
        
        cd tachyon/tests/e2e
        
        if [ ! -d "node_modules" ]; then
            echo "  → Installing dependencies..."
            npm ci
        fi
        
        echo "  → Installing Playwright..."
        npx playwright install --with-deps
        
        echo "  → Running Playwright tests..."
        npm test
        
        cd ../../..
        echo -e "${GREEN}E2E tests complete!${NC}"
    fi
}

# Function to run coverage
run_coverage() {
    if [ "$RUN_COVERAGE" = true ]; then
        echo -e "${YELLOW}Generating Coverage Report...${NC}"
        
        if command -v cargo-tarpaulin &> /dev/null; then
            cargo tarpaulin --out Html --out Xml --out Lcov --all-features
            
            echo -e "${GREEN}Coverage report generated!${NC}"
            echo "Open tarpaulin-report.html to view"
        else
            echo -e "${YELLOW}cargo-tarpaulin not found, skipping coverage${NC}"
            echo "Install with: cargo install cargo-tarpaulin"
        fi
    fi
}

# Main execution
main() {
    # Check for test database
    if [ -z "$TEST_DATABASE_URL" ]; then
        export TEST_DATABASE_URL="postgres://tachyon:tachyon@localhost:5432/tachyon_test"
        echo -e "${YELLOW}Using default test database: $TEST_DATABASE_URL${NC}"
    fi
    
    # Run tests
    run_backend_tests
    echo ""
    
    run_frontend_tests
    echo ""
    
    run_e2e_tests
    echo ""
    
    run_coverage
    echo ""
    
    echo -e "${GREEN}=== All tests complete! ===${NC}"
}

# Run main function
main
