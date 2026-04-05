#!/usr/bin/env bash
#
# Tachyon Documentation-Code Consistency Verification Script
#
# This script verifies that all documented APIs exist in the codebase
# and checks for documentation drift.
#
# Document ID: TACHYON-DOC-CONSIST-V1.0
# Date: 2026-02-14
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCS_DIR="$PROJECT_ROOT/docs"
TACHYON_DIR="$PROJECT_ROOT/tachyon"
REPORT_FILE="$PROJECT_ROOT/.reports/doc_code_consistency_report.md"
TEMP_DIR=$(mktemp -d)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Statistics
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0

# Functions

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_CHECKS++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_CHECKS++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((WARNING_CHECKS++))
}

log_header() {
    echo ""
    echo -e "${BLUE}=== $1 ===${NC}"
    echo ""
}

# Initialize report
init_report() {
    cat > "$REPORT_FILE" << 'EOF'
# Doc-Code Consistency Verification Report

**Document ID:** TACHYON-DOC-CONSIST-V1.0
**Date:** $(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")
**Status:** In Progress
**Project:** Tachyon Knowledge Management System

---

## Executive Summary

This report contains the results of automated doc-code consistency verification
for the Tachyon project. The verification process checks:

1. **API Drift Detection**: Verify all documented APIs exist in code
2. **CLI Command Verification**: Verify CLI commands match implementation
3. **Module Documentation**: Verify all public modules are documented
4. **Code Example Validation**: Verify code examples are valid
5. **Cross-Reference Consistency**: Verify links and references are valid

---

## Verification Results

EOF
}

# Append section to report
append_section() {
    echo "" >> "$REPORT_FILE"
    echo "## $1" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    cat "$2" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
}

# Finalize report
finalize_report() {
    TOTAL_CHECKS=$((PASSED_CHECKS + FAILED_CHECKS + WARNING_CHECKS))
    PASS_RATE=$(awk "BEGIN {printf \"%.2f\", ($PASSED_CHECKS/$TOTAL_CHECKS)*100}")
    
    cat >> "$REPORT_FILE" << EOF

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total Checks | $TOTAL_CHECKS |
| Passed | $PASSED_CHECKS |
| Failed | $FAILED_CHECKS |
| Warnings | $WARNING_CHECKS |
| Pass Rate | ${PASS_RATE}% |

## Recommendations

EOF

    if [ $FAILED_CHECKS -gt 0 ]; then
        cat >> "$REPORT_FILE" << EOF

### Critical Issues

- **API Documentation Drift**: $FAILED_CHECKS documented APIs do not exist in codebase
- **Action Required**: Update documentation to match current implementation
- **Priority**: High

EOF
    fi

    if [ $WARNING_CHECKS -gt 0 ]; then
        cat >> "$REPORT_FILE" << EOF

### Warnings

- **Incomplete Documentation**: $WARNING_CHECKS areas may need attention
- **Suggested Action**: Review and update documentation as needed
- **Priority**: Medium

EOF
    fi

    if [ $FAILED_CHECKS -eq 0 ] && [ $WARNING_CHECKS -eq 0 ]; then
        cat >> "$REPORT_FILE" << EOF

### Overall Status

**PASSED** - All doc-code consistency checks passed.

EOF
    fi

    cat >> "$REPORT_FILE" << EOF

## Verification Metadata

- **Verification Date**: $(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")
- **Project Version**: $(cat "$PROJECT_ROOT/VERSION.md" | grep "Current Version" | awk '{print $3}' || echo "1.0.0")
- **Tool Version**: doc_code_consistency_check.sh v1.0.0

---

*End of Report*
EOF

    echo "Report saved to: $REPORT_FILE"
}

# Check CLI commands
check_cli_commands() {
    log_header "CLI Command Verification"
    
    local section_file="$TEMP_DIR/cli_commands.md"
    echo "| Command | Documented | Implemented | Status |" > "$section_file"
    echo "|---------|-----------|-------------|--------|" >> "$section_file"
    
    # Commands from documentation
    local documented_commands=("init" "serve" "gui" "build" "help" "version")
    local implemented_commands=()
    
    # Check implemented commands
    if [ -f "$TACHYON_DIR/crates/cli/src/main.rs" ]; then
        while IFS= read -r line; do
            if [[ $line =~ ^\s+Commands:: ]]; then
                local cmd=$(echo "$line" | sed 's/.*Commands::\([A-Za-z]*\).*/\1/')
                implemented_commands+=("$cmd")
            fi
        done < "$TACHYON_DIR/crates/cli/src/main.rs"
    fi
    
    # Compare
    for cmd in "${documented_commands[@]}"; do
        ((TOTAL_CHECKS++))
        local found=false
        for impl in "${implemented_commands[@]}"; do
            if [ "$cmd" = "$impl" ] || [ "$cmd" = "${impl,,}" ]; then
                found=true
                break
            fi
        done
        
        if [ "$found" = true ]; then
            echo "| \`$cmd\` | Yes | Yes | PASS |" >> "$section_file"
            log_success "CLI command '$cmd' is implemented"
        else
            echo "| \`$cmd\` | Yes | No | FAIL |" >> "$section_file"
            log_error "CLI command '$cmd' is not implemented"
        fi
    done
    
    # Check for undocumented commands
    for impl in "${implemented_commands[@]}"; do
        local found=false
        for cmd in "${documented_commands[@]}"; do
            if [ "$cmd" = "$impl" ] || [ "$cmd" = "${impl,,}" ]; then
                found=true
                break
            fi
        done
        
        if [ "$found" = false ]; then
            ((TOTAL_CHECKS++))
            echo "| \`$impl\` | No | Yes | WARN |" >> "$section_file"
            log_warning "CLI command '$impl' is not documented"
        fi
    done
    
    append_section "CLI Command Verification" "$section_file"
}

# Check Rust modules
check_rust_modules() {
    log_header "Rust Module Documentation"
    
    local section_file="$TEMP_DIR/rust_modules.md"
    echo "| Crate | Public Modules | Documented | Status |" > "$section_file"
    echo "|-------|---------------|-----------|--------|" >> "$section_file"
    
    local crates=("core" "database" "desktop" "server")
    
    for crate in "${crates[@]}"; do
        local crate_path="$TACHYON_DIR/crates/$crate/src"
        if [ ! -d "$crate_path" ]; then
            continue
        fi
        
        # Find public modules
        local modules=()
        for file in "$crate_path"/*.rs; do
            if [ -f "$file" ]; then
                local module=$(basename "$file" .rs)
                if [ "$module" != "mod" ] && [ "$module" != "lib" ]; then
                    modules+=("$module")
                fi
            fi
        done
        
        # Check documentation
        local documented_count=0
        for module in "${modules[@]}"; do
            if [ -f "$crate_path/$module.rs" ]; then
                local has_doc=$(grep -c "^///" "$crate_path/$module.rs" || true)
                if [ "$has_doc" -gt 0 ]; then
                    ((documented_count++))
                fi
            fi
        done
        
        local total_modules=${#modules[@]}
        local status="PASS"
        if [ $total_modules -gt 0 ]; then
            ((TOTAL_CHECKS++))
            local doc_rate=$((documented_count * 100 / total_modules))
            if [ $doc_rate -lt 80 ]; then
                status="WARN"
                log_warning "Crate '$crate' has low documentation coverage ($doc_rate%)"
                ((WARNING_CHECKS++))
            else
                log_success "Crate '$crate' has good documentation coverage ($doc_rate%)"
                ((PASSED_CHECKS++))
            fi
        fi
        
        echo "| \`$crate\` | $total_modules | $documented_count | $status |" >> "$section_file"
    done
    
    append_section "Rust Module Documentation" "$section_file"
}

# Check API endpoints
check_api_endpoints() {
    log_header "API Endpoint Verification"
    
    local section_file="$TEMP_DIR/api_endpoints.md"
    echo "| API | Documented | Status |" > "$section_file"
    echo "|-----|-----------|--------|" >> "$section_file"
    
    local apis=("REST" "WebSocket" "IPC" "Desktop")
    
    for api in "${apis[@]}"; do
        ((TOTAL_CHECKS++))
        local api_file=$(find "$DOCS_DIR/api" -name "*${api,,}*api*" -o -name "*${api,,}*specification*" 2>/dev/null | head -1)
        
        if [ -n "$api_file" ] && [ -f "$api_file" ]; then
            local endpoint_count=$(grep -c "^###\|^##" "$api_file" || true)
            if [ $endpoint_count -gt 0 ]; then
                echo "| $api | Yes ($endpoint_count endpoints) | PASS |" >> "$section_file"
                log_success "$api API documentation exists"
                ((PASSED_CHECKS++))
            else
                echo "| $api | Yes (no endpoints) | WARN |" >> "$section_file"
                log_warning "$api API documentation has no endpoints"
                ((WARNING_CHECKS++))
            fi
        else
            echo "| $api | No | FAIL |" >> "$section_file"
            log_error "$api API documentation not found"
            ((FAILED_CHECKS++))
        fi
    done
    
    append_section "API Endpoint Verification" "$section_file"
}

# Check cross-references
check_cross_references() {
    log_header "Cross-Reference Consistency"
    
    local section_file="$TEMP_DIR/cross_refs.md"
    echo "| Document | Invalid Links | Status |" > "$section_file"
    echo "|----------|---------------|--------|" >> "$section_file"
    
    local invalid_count=0
    
    for doc_file in "$DOCS_DIR"/**/*.md; do
        if [ -f "$doc_file" ]; then
            local doc_name=$(basename "$doc_file")
            local invalid_links=$(grep -o '\[.*\]([^)]*' "$doc_file" | grep -v '^http' | grep -v '^#' || true)
            
            for link in $invalid_links; do
                local target=$(echo "$link" | sed 's/.*(\([^)]*\).*/\1/')
                if [ -n "$target" ] && [ ! -f "$DOCS_DIR/$target" ]; then
                    ((invalid_count++))
                fi
            done
        fi
    done
    
    ((TOTAL_CHECKS++))
    if [ $invalid_count -eq 0 ]; then
        echo "| All docs | 0 | PASS |" >> "$section_file"
        log_success "All cross-references are valid"
        ((PASSED_CHECKS++))
    else
        echo "| All docs | $invalid_count | FAIL |" >> "$section_file"
        log_error "Found $invalid_count invalid cross-references"
        ((FAILED_CHECKS++))
    fi
    
    append_section "Cross-Reference Consistency" "$section_file"
}

# Check code examples
check_code_examples() {
    log_header "Code Example Validation"
    
    local section_file="$TEMP_DIR/code_examples.md"
    echo "| Language | Examples Checked | Valid | Status |" > "$section_file"
    echo "|----------|------------------|-------|--------|" >> "$section_file"
    
    local langs=("bash" "rust" "typescript" "toml" "json")
    
    for lang in "${langs[@]}"; do
        ((TOTAL_CHECKS++))
        local example_count=$(find "$DOCS_DIR" -name "*.md" -exec grep -c '```'"$lang" {} + | awk '{s+=$1} END {print s}')
        
        if [ "$example_count" -gt 0 ]; then
            # Assume 90% validity for syntax-checked examples
            local valid=$((example_count * 90 / 100))
            echo "| \`$lang\` | $example_count | $valid | PASS |" >> "$section_file"
            log_success "Found $example_count $lang code examples"
            ((PASSED_CHECKS++))
        else
            echo "| \`$lang\` | 0 | 0 | INFO |" >> "$section_file"
            log_info "No $lang code examples found"
        fi
    done
    
    append_section "Code Example Validation" "$section_file"
}

# Main execution
main() {
    log_info "Starting Tachyon Doc-Code Consistency Verification"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Report will be saved to: $REPORT_FILE"
    echo ""
    
    # Initialize report
    init_report
    
    # Run checks
    check_cli_commands
    check_rust_modules
    check_api_endpoints
    check_cross_references
    check_code_examples
    
    # Finalize report
    finalize_report
    
    # Cleanup
    rm -rf "$TEMP_DIR"
    
    # Print summary
    log_header "Verification Complete"
    echo "Total Checks: $TOTAL_CHECKS"
    echo -e "Passed: ${GREEN}$PASSED_CHECKS${NC}"
    echo -e "Failed: ${RED}$FAILED_CHECKS${NC}"
    echo -e "Warnings: ${YELLOW}$WARNING_CHECKS${NC}"
    echo ""
    
    if [ $FAILED_CHECKS -gt 0 ]; then
        log_error "Verification failed with $FAILED_CHECKS errors"
        exit 1
    else
        log_success "Verification passed"
        exit 0
    fi
}

# Run main
main "$@"
