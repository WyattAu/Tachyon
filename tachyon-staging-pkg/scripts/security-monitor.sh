#!/bin/bash
# Tachyon Security Monitoring Script
# Runs security checks and alerts on vulnerabilities

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."
ALERT_EMAIL="${ALERT_EMAIL:-admin@example.com}"
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"
LOG_FILE="/var/log/tachyon-security.log"

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

alert() {
    local severity="$1"
    local message="$2"
    
    echo -e "${severity}: $message"
    log "[$severity] $message"
    
    # Send email alert
    if command -v mail &> /dev/null; then
        echo "$message" | mail -s "Tachyon Security Alert: $severity" "$ALERT_EMAIL"
    fi
    
    # Send Slack alert
    if [ -n "$SLACK_WEBHOOK" ]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"🚨 Tachyon Security Alert\\nSeverity: $severity\\nMessage: $message\"}" \
            "$SLACK_WEBHOOK" 2>/dev/null || true
    fi
}

# Check for cargo-audit
check_dependencies() {
    if ! command -v cargo-audit &> /dev/null; then
        log "Installing cargo-audit..."
        cargo install cargo-audit
    fi
}

# Run security audit
run_audit() {
    log "Running cargo audit..."
    
    cd "$PROJECT_ROOT/tachyon"
    
    # Run audit and capture output
    local audit_output
    local audit_exit=0
    
    audit_output=$(cargo audit --json 2>/dev/null) || audit_exit=$?
    
    if [ $audit_exit -ne 0 ]; then
        log "Security vulnerabilities found"
        
        # Parse JSON output
        local vulnerabilities
        vulnerabilities=$(echo "$audit_output" | jq -r '.vulnerabilities.list[] | "\(.advisory.id): \(.advisory.title) (Severity: \(.advisory.severity // \"unknown\"))"' 2>/dev/null || echo "Failed to parse audit output")
        
        # Check for critical vulnerabilities
        local critical_count
        critical_count=$(echo "$audit_output" | jq '[.vulnerabilities.list[] | select(.advisory.severity == "critical")] | length' 2>/dev/null || echo "0")
        
        if [ "$critical_count" -gt 0 ]; then
            alert "CRITICAL" "Found $critical_count critical vulnerabilities:\n$vulnerabilities"
            return 1
        fi
        
        # Check for high vulnerabilities
        local high_count
        high_count=$(echo "$audit_output" | jq '[.vulnerabilities.list[] | select(.advisory.severity == "high")] | length' 2>/dev/null || echo "0")
        
        if [ "$high_count" -gt 0 ]; then
            alert "HIGH" "Found $high_count high severity vulnerabilities:\n$vulnerabilities"
        fi
        
        # Log medium/low for record
        local medium_count
        medium_count=$(echo "$audit_output" | jq '[.vulnerabilities.list[] | select(.advisory.severity == "medium")] | length' 2>/dev/null || echo "0")
        
        local low_count
        low_count=$(echo "$audit_output" | jq '[.vulnerabilities.list[] | select(.advisory.severity == "low")] | length' 2>/dev/null || echo "0")
        
        log "Vulnerability summary: Critical: $critical_count, High: $high_count, Medium: $medium_count, Low: $low_count"
    else
        log "No vulnerabilities found"
    fi
}

# Check for outdated dependencies
check_outdated() {
    log "Checking for outdated dependencies..."
    
    cd "$PROJECT_ROOT/tachyon"
    
    if ! command -v cargo-outdated &> /dev/null; then
        log "Installing cargo-outdated..."
        cargo install cargo-outdated
    fi
    
    local outdated
    outdated=$(cargo outdated --workspace --exclude tachyon-testing -R 2>/dev/null || true)
    
    if [ -n "$outdated" ]; then
        log "Outdated dependencies found:"
        echo "$outdated" | tee -a "$LOG_FILE"
        
        # Alert if major version updates available
        if echo "$outdated" | grep -q "Yes"; then
            alert "MEDIUM" "Major version updates available for dependencies. Review recommended."
        fi
    else
        log "All dependencies are up to date"
    fi
}

# Check file permissions
check_permissions() {
    log "Checking file permissions..."
    
    local issues=0
    
    # Check for world-writable files
    local world_writable
    world_writable=$(find "$PROJECT_ROOT" -type f -perm -002 ! -path "*/target/*" ! -path "*/.git/*" 2>/dev/null || true)
    
    if [ -n "$world_writable" ]; then
        alert "HIGH" "World-writable files found:\n$world_writable"
        issues=$((issues + 1))
    fi
    
    # Check for files with passwords/secrets
    local suspicious_files
    suspicious_files=$(grep -r -l "password\|secret\|api_key" "$PROJECT_ROOT/config" 2>/dev/null | grep -v ".example" | grep -v ".md" || true)
    
    if [ -n "$suspicious_files" ]; then
        alert "MEDIUM" "Files containing potential secrets:\n$suspicious_files"
        issues=$((issues + 1))
    fi
    
    if [ $issues -eq 0 ]; then
        log "File permissions check passed"
    fi
}

# Check SSL certificate expiration
check_ssl_certs() {
    log "Checking SSL certificates..."
    
    local cert_dir="$PROJECT_ROOT/scripts/ssl"
    
    if [ ! -d "$cert_dir" ]; then
        log "SSL directory not found, skipping certificate check"
        return
    fi
    
    for cert in "$cert_dir"/*.pem; do
        if [ -f "$cert" ]; then
            local expiry
            expiry=$(openssl x509 -enddate -noout -in "$cert" 2>/dev/null | cut -d= -f2)
            
            if [ -n "$expiry" ]; then
                local expiry_epoch
                expiry_epoch=$(date -d "$expiry" +%s 2>/dev/null || echo "0")
                local now
                now=$(date +%s)
                local days_until_expiry
                days_until_expiry=$(( (expiry_epoch - now) / 86400 ))
                
                if [ $days_until_expiry -lt 7 ]; then
                    alert "CRITICAL" "SSL certificate expires in $days_until_expiry days: $cert"
                elif [ $days_until_expiry -lt 30 ]; then
                    alert "HIGH" "SSL certificate expires in $days_until_expiry days: $cert"
                else
                    log "Certificate valid for $days_until_expiry days: $(basename "$cert")"
                fi
            fi
        fi
    done
}

# Check container security
check_containers() {
    log "Checking container security..."
    
    if ! command -v docker &> /dev/null; then
        log "Docker not found, skipping container check"
        return
    fi
    
    # Check for containers running as root
    local root_containers
    root_containers=$(docker ps --format "table {{.Names}}\t{{.Image}}" | grep -E "tachyon" | awk '{print $1}' | while read container; do
        if docker inspect "$container" --format='{{.Config.User}}' | grep -q "root"; then
            echo "$container"
        fi
    done || true)
    
    if [ -n "$root_containers" ]; then
        alert "MEDIUM" "Containers running as root:\n$root_containers"
    fi
    
    # Check for containers with privileged mode
    local privileged
    privileged=$(docker ps --format "table {{.Names}}\t{{.Image}}" | grep -E "tachyon" | awk '{print $1}' | while read container; do
        if docker inspect "$container" --format='{{.HostConfig.Privileged}}' | grep -q "true"; then
            echo "$container"
        fi
    done || true)
    
    if [ -n "$privileged" ]; then
        alert "HIGH" "Containers running in privileged mode:\n$privileged"
    fi
}

# Generate security report
generate_report() {
    local report_file="/tmp/tachyon-security-report-$(date +%Y%m%d).txt"
    
    cat > "$report_file" << EOF
Tachyon Security Report
Generated: $(date)
==================================================

VULNERABILITY SCAN:
$(cd "$PROJECT_ROOT/tachyon" && cargo audit 2>&1 || true)

OUTDATED DEPENDENCIES:
$(cd "$PROJECT_ROOT/tachyon" && cargo outdated --workspace --exclude tachyon-testing 2>&1 || true)

FILE PERMISSIONS:
World-writable files: $(find "$PROJECT_ROOT" -type f -perm -002 ! -path "*/target/*" ! -path "*/.git/*" 2>/dev/null | wc -l)

SSL CERTIFICATES:
$(for cert in "$PROJECT_ROOT/scripts/ssl"/*.pem 2>/dev/null; do
    if [ -f "$cert" ]; then
        echo "$(basename "$cert"): $(openssl x509 -enddate -noout -in "$cert" 2>/dev/null | cut -d= -f2)"
    fi
done)

CONTAINER STATUS:
$(docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null | grep -E "tachyon" || echo "No containers running")

==================================================
Report complete.
EOF

    log "Security report generated: $report_file"
    
    # Email report
    if command -v mail &> /dev/null; then
        mail -s "Tachyon Security Report - $(date +%Y-%m-%d)" "$ALERT_EMAIL" < "$report_file"
    fi
}

# Main execution
main() {
    log "Starting security monitoring check"
    
    check_dependencies
    run_audit
    check_outdated
    check_permissions
    check_ssl_certs
    check_containers
    generate_report
    
    log "Security monitoring check complete"
}

# Run if executed directly
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi
