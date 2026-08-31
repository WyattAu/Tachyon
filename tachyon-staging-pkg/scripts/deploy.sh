#!/bin/bash
# Tachyon Production Deployment Script
# Usage: ./deploy.sh [environment] [version]
# Example: ./deploy.sh production v1.0.0

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_NAME="tachyon"
DEFAULT_ENV="staging"
ENVIRONMENT="${1:-$DEFAULT_ENV}"
VERSION="${2:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validate environment
validate_environment() {
    case "$ENVIRONMENT" in
        staging|production)
            log_info "Deploying to $ENVIRONMENT environment"
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT"
            log_error "Valid environments: staging, production"
            exit 1
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        log_error "Must run from project root directory"
        exit 1
    fi
    
    log_success "All prerequisites met"
}

# Build release binary
build_release() {
    log_info "Building release binary..."
    
    cd "$SCRIPT_DIR/.."
    
    # Build with optimizations
    cargo build --release --workspace --exclude tachyon-testing
    
    # Verify binary was created
    if [ ! -f "target/release/tachyon-server" ]; then
        log_error "Build failed: tachyon-server binary not found"
        exit 1
    fi
    
    log_success "Release build completed"
}

# Run security audit
security_audit() {
    log_info "Running security audit..."
    
    cd "$SCRIPT_DIR/.."
    
    # Check if cargo-audit is installed
    if ! command -v cargo-audit &> /dev/null; then
        log_warning "cargo-audit not installed, installing..."
        cargo install cargo-audit
    fi
    
    # Run audit
    if cargo audit; then
        log_success "Security audit passed"
    else
        log_warning "Security audit found issues - review required"
        read -p "Continue with deployment? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_error "Deployment aborted"
            exit 1
        fi
    fi
}

# Run tests
run_tests() {
    log_info "Running test suite..."
    
    cd "$SCRIPT_DIR/.."
    
    # Run all tests except tachyon-testing
    if cargo test --workspace --lib --exclude tachyon-testing; then
        log_success "All tests passed"
    else
        log_error "Tests failed - deployment aborted"
        exit 1
    fi
}

# Create deployment package
create_package() {
    log_info "Creating deployment package..."
    
    local package_dir="$SCRIPT_DIR/../deploy/packages"
    local package_name="tachyon-${VERSION}-${ENVIRONMENT}"
    
    mkdir -p "$package_dir"
    
    # Create package structure
    mkdir -p "$package_dir/$package_name"
    cp target/release/tachyon-server "$package_dir/$package_name/"
    cp target/release/tachyon "$package_dir/$package_name/" 2>/dev/null || true
    cp -r web/dist "$package_dir/$package_name/" 2>/dev/null || true
    cp "$SCRIPT_DIR/../config/production.toml" "$package_dir/$package_name/config.toml" 2>/dev/null || true
    cp "$SCRIPT_DIR/docker-compose.yml" "$package_dir/$package_name/"
    cp "$SCRIPT_DIR/Dockerfile" "$package_dir/$package_name/"
    
    # Create tarball
    cd "$package_dir"
    tar -czf "${package_name}.tar.gz" "$package_name"
    
    log_success "Package created: ${package_name}.tar.gz"
}

# Deploy to environment
deploy() {
    log_info "Deploying to $ENVIRONMENT..."
    
    # Set environment-specific variables
    case "$ENVIRONMENT" in
        staging)
            export COMPOSE_PROJECT_NAME="tachyon-staging"
            export SERVER_PORT="8080"
            export DATABASE_URL="sqlite:/data/tachyon-staging.db"
            ;;
        production)
            export COMPOSE_PROJECT_NAME="tachyon-production"
            export SERVER_PORT="80"
            export DATABASE_URL="sqlite:/data/tachyon.db"
            ;;
    esac
    
    # Stop existing containers
    log_info "Stopping existing containers..."
    docker-compose -f "$SCRIPT_DIR/docker-compose.yml" down --remove-orphans
    
    # Pull latest images (if using pre-built)
    if [ "$VERSION" != "latest" ]; then
        docker-compose -f "$SCRIPT_DIR/docker-compose.yml" pull
    fi
    
    # Start new containers
    log_info "Starting new containers..."
    docker-compose -f "$SCRIPT_DIR/docker-compose.yml" up -d --build
    
    # Wait for health check
    log_info "Waiting for health check..."
    sleep 5
    
    local retries=0
    local max_retries=30
    
    while [ $retries -lt $max_retries ]; do
        if curl -sf http://localhost:${SERVER_PORT}/health > /dev/null 2>&1; then
            log_success "Health check passed"
            break
        fi
        
        retries=$((retries + 1))
        log_info "Health check attempt $retries/$max_retries..."
        sleep 2
    done
    
    if [ $retries -eq $max_retries ]; then
        log_error "Health check failed after $max_retries attempts"
        log_info "Rolling back..."
        docker-compose -f "$SCRIPT_DIR/docker-compose.yml" down
        exit 1
    fi
    
    log_success "Deployment to $ENVIRONMENT completed successfully"
}

# Cleanup old deployments
cleanup() {
    log_info "Cleaning up old deployments..."
    
    # Keep only last 5 packages
    local package_dir="$SCRIPT_DIR/../deploy/packages"
    if [ -d "$package_dir" ]; then
        cd "$package_dir"
        ls -t *.tar.gz 2>/dev/null | tail -n +6 | xargs -r rm --
        log_success "Old packages cleaned up"
    fi
    
    # Clean up Docker images
    docker image prune -f
    
    log_success "Cleanup completed"
}

# Main deployment flow
main() {
    log_info "Starting Tachyon deployment"
    log_info "Environment: $ENVIRONMENT"
    log_info "Version: $VERSION"
    
    validate_environment
    check_prerequisites
    
    # Confirm production deployment
    if [ "$ENVIRONMENT" == "production" ]; then
        log_warning "You are about to deploy to PRODUCTION"
        read -p "Are you sure? Type 'deploy' to confirm: " confirm
        if [ "$confirm" != "deploy" ]; then
            log_error "Deployment cancelled"
            exit 1
        fi
    fi
    
    # Run deployment steps
    security_audit
    run_tests
    build_release
    create_package
    deploy
    cleanup
    
    log_success "Deployment completed successfully!"
    log_info "Server running at: http://localhost:${SERVER_PORT}"
}

# Run main function
main "$@"
