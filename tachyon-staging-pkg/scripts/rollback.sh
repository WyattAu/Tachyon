#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEFAULT_ENV="staging"
ENVIRONMENT="${1:-$DEFAULT_ENV}"
ROLLBACK_VERSION="${2:-}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

validate_environment() {
    case "$ENVIRONMENT" in
        staging|production)
            log_info "Rolling back $ENVIRONMENT environment"
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT"
            exit 1
            ;;
    esac
}

get_previous_version() {
    if [ -n "$ROLLBACK_VERSION" ]; then
        echo "$ROLLBACK_VERSION"
        return
    fi
    
    local versions_file="$PROJECT_ROOT/deploy/.versions"
    if [ -f "$versions_file" ]; then
        tail -n 2 "$versions_file" | head -n 1 | cut -d' ' -f2
    else
        log_error "No version history found"
        exit 1
    fi
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    command -v docker >/dev/null 2>&1 || { log_error "Docker required"; exit 1; }
    command -v docker-compose >/dev/null 2>&1 || { log_error "Docker Compose required"; exit 1; }
    
    log_success "Prerequisites met"
}

backup_current_state() {
    log_info "Backing up current state..."
    
    local backup_dir="$PROJECT_ROOT/deploy/backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$backup_dir"
    
    if docker ps --format '{{.Names}}' | grep -q "tachyon"; then
        docker exec tachyon-backend cp -r /data "$backup_dir/data" 2>/dev/null || true
    fi
    
    echo "$(date +%Y%m%d_%H%M%S) $(get_current_version)" >> "$PROJECT_ROOT/deploy/.rollback_history"
    
    log_success "Backup created at $backup_dir"
}

get_current_version() {
    docker inspect tachyon-backend --format '{{.Config.Image}}' 2>/dev/null | cut -d':' -f2 || echo "unknown"
}

perform_rollback() {
    local target_version="$1"
    
    log_info "Rolling back to version: $target_version"
    
    if [ "$ENVIRONMENT" == "production" ]; then
        log_warning "You are about to rollback PRODUCTION"
        read -p "Type 'rollback' to confirm: " confirm
        if [ "$confirm" != "rollback" ]; then
            log_error "Rollback cancelled"
            exit 1
        fi
    fi
    
    cd "$PROJECT_ROOT"
    
    export VERSION="$target_version"
    export COMPOSE_PROJECT_NAME="tachyon-$ENVIRONMENT"
    
    log_info "Pulling previous version images..."
    docker-compose -f docker-compose.yml -f docker-compose.prod.yml pull 2>/dev/null || true
    
    log_info "Stopping current containers..."
    docker-compose -f docker-compose.yml -f docker-compose.prod.yml down --timeout 30
    
    log_info "Starting containers with version $target_version..."
    docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
    
    log_info "Waiting for health checks..."
    sleep 10
    
    local retries=0
    local max_retries=30
    
    while [ $retries -lt $max_retries ]; do
        if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
            log_success "Health check passed"
            break
        fi
        retries=$((retries + 1))
        log_info "Health check attempt $retries/$max_retries..."
        sleep 2
    done
    
    if [ $retries -eq $max_retries ]; then
        log_error "Health check failed after rollback"
        log_error "Manual intervention required"
        exit 1
    fi
}

verify_rollback() {
    log_info "Verifying rollback..."
    
    local current=$(get_current_version)
    local expected="${ROLLBACK_VERSION:-$(get_previous_version)}"
    
    if [[ "$current" == *"$expected"* ]]; then
        log_success "Rollback verified: running $current"
    else
        log_warning "Version mismatch: expected $expected, got $current"
    fi
    
    if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
        log_success "Service is healthy"
    else
        log_error "Service health check failed"
        exit 1
    fi
}

notify_rollback() {
    log_info "Rollback completed at $(date)"
    
    if [ -n "${SLACK_WEBHOOK:-}" ]; then
        curl -s -X POST "$SLACK_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{\"text\":\"Tachyon rollback to ${ROLLBACK_VERSION:-previous} in $ENVIRONMENT completed\"}" \
            >/dev/null 2>&1 || true
    fi
}

main() {
    log_info "Starting Tachyon Rollback"
    log_info "Environment: $ENVIRONMENT"
    
    validate_environment
    check_prerequisites
    
    local target_version
    target_version=$(get_previous_version)
    log_info "Target version: $target_version"
    
    backup_current_state
    perform_rollback "$target_version"
    verify_rollback
    notify_rollback
    
    log_success "Rollback completed successfully!"
}

main "$@"
