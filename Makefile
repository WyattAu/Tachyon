# Tachyon Monorepo Makefile
# Provides convenient commands for building, testing, and managing the project

.DEFAULT_GOAL := help

.PHONY: help all build build-release build-server build-cli build-desktop build-frontend \
       test test-verbose test-crate-% test-core test-database test-server test-search \
       test-renderer test-rbac test-cli coverage \
       fmt fmt-check lint lint-fix check fix \
       audit audit-json security-monitor \
       doc doc-build \
       clean clean-all \
       serve serve-release run \
       deploy deploy-staging deploy-production \
       docker-build docker-up docker-down docker-logs docker-clean \
       bench \
       install update outdated watch tree \
       ci ci-lite \
       version status \
       init init-example \
       quickstart quickstart-setup quickstart-start quickstart-stop quickstart-test quickstart-status quickstart-clean \
       web-build web-dev \
       backup-db migrate reset-db \
       docs-preview link-check

# Colors for terminal output
BLUE := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
NC := \033[0m # No Color

# Project configuration
PROJECT_NAME := tachyon
NIX := nix develop --command
CARGO := $(NIX) cargo
WORKSPACE_FLAGS := --workspace --exclude tachyon-testing --exclude tachyon-frontend --exclude tachyon-desktop --exclude tachyon-desktop-app
RELEASE_FLAGS := --release $(WORKSPACE_FLAGS)
TEST_FLAGS := --workspace --lib --exclude tachyon-testing --exclude tachyon-frontend --exclude tachyon-desktop --exclude tachyon-desktop-app

# ============================================================================
# Help Target
# ============================================================================

help: ## Show this help message
	@echo "$(BLUE)Tachyon Monorepo - Available Commands$(NC)"
	@echo "========================================"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'

# ============================================================================
# Build Targets
# ============================================================================

all: fmt lint build test ## Run full CI pipeline (format, lint, build, test)

build: ## Build the entire workspace in debug mode
	@echo "$(BLUE)Building workspace (debug)...$(NC)"
	cd tachyon && $(CARGO) build $(WORKSPACE_FLAGS)
	@echo "$(GREEN)✓ Build complete$(NC)"

build-release: ## Build optimized release binaries
	@echo "$(BLUE)Building workspace (release)...$(NC)"
	cd tachyon && $(CARGO) build $(RELEASE_FLAGS)
	@echo "$(GREEN)✓ Release build complete$(NC)"
	@echo "$(YELLOW)Binaries located in: tachyon/target/release/$(NC)"

build-server: ## Build only the server binary
	@echo "$(BLUE)Building server...$(NC)"
	cd tachyon && $(CARGO) build --release -p tachyon-server
	@echo "$(GREEN)✓ Server built$(NC)"

build-cli: ## Build only the CLI binary
	@echo "$(BLUE)Building CLI...$(NC)"
	cd tachyon && $(CARGO) build --release -p tachyon-cli
	@echo "$(GREEN)✓ CLI built$(NC)"

build-desktop: ## Build desktop application (Tauri builds frontend via beforeBuildCommand)
	@echo "$(BLUE)Building desktop application...$(NC)"
	cd tachyon/crates/desktop/src-tauri && $(CARGO) tauri build
	@echo "$(GREEN)✓ Desktop app built$(NC)"

build-frontend: ## Build Leptos frontend WASM via Trunk
	@echo "$(BLUE)Building frontend WASM...$(NC)"
	cd tachyon/crates/frontend && trunk build --release
	@echo "$(GREEN)✓ Frontend built$(NC)"

# ============================================================================
# Test Targets
# ============================================================================

test: ## Run all library tests
	@echo "$(BLUE)Running tests...$(NC)"
	cd tachyon && $(CARGO) test $(TEST_FLAGS)
	@echo "$(GREEN)✓ Tests complete$(NC)"

test-verbose: ## Run tests with verbose output
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	cd tachyon && $(CARGO) test $(TEST_FLAGS) -- --nocapture

test-crate-%: ## Run tests for a specific crate (e.g., make test-crate-core)
	@echo "$(BLUE)Running tests for crate: $*$(NC)"
	cd tachyon && $(CARGO) test -p tachyon-$*

test-core: ## Run tests for tachyon-core
	cd tachyon && $(CARGO) test -p tachyon-core --lib

test-database: ## Run tests for tachyon-database
	cd tachyon && $(CARGO) test -p tachyon-database --lib

test-server: ## Run tests for tachyon-server
	cd tachyon && $(CARGO) test -p tachyon-server --lib

test-search: ## Run tests for tachyon-search
	cd tachyon && $(CARGO) test -p tachyon-search --lib

test-renderer: ## Run tests for tachyon-renderer
	cd tachyon && $(CARGO) test -p tachyon-renderer --lib

test-rbac: ## Run tests for tachyon-rbac
	cd tachyon && $(CARGO) test -p tachyon-rbac --lib

test-cli: ## Run tests for tachyon-cli
	cd tachyon && $(CARGO) test -p tachyon-cli --lib

coverage: ## Generate test coverage report (requires cargo-tarpaulin)
	@echo "$(BLUE)Generating coverage report...$(NC)"
	cd tachyon && cargo tarpaulin --workspace --exclude tachyon-testing --out Html
	@echo "$(GREEN)✓ Coverage report generated: tachyon/tarpaulin-report.html$(NC)"

# ============================================================================
# Code Quality Targets
# ============================================================================

fmt: ## Format all code using rustfmt
	@echo "$(BLUE)Formatting code...$(NC)"
	cd tachyon && $(CARGO) fmt --all
	@echo "$(GREEN)✓ Formatting complete$(NC)"

fmt-check: ## Check if code is properly formatted
	@echo "$(BLUE)Checking code formatting...$(NC)"
	cd tachyon && $(CARGO) fmt --all -- --check
	@echo "$(GREEN)✓ Code is properly formatted$(NC)"

lint: ## Run clippy linter
	@echo "$(BLUE)Running clippy...$(NC)"
	cd tachyon && $(CARGO) clippy $(WORKSPACE_FLAGS) -- -D warnings
	@echo "$(GREEN)✓ Linting complete$(NC)"

lint-fix: ## Run clippy with automatic fixes
	@echo "$(BLUE)Running clippy with fixes...$(NC)"
	cd tachyon && $(CARGO) clippy --fix $(WORKSPACE_FLAGS) --allow-dirty --allow-staged
	@echo "$(GREEN)✓ Linting fixes applied$(NC)"

check: ## Run cargo check (fast compile check)
	@echo "$(BLUE)Running cargo check...$(NC)"
	cd tachyon && $(CARGO) check $(WORKSPACE_FLAGS)
	@echo "$(GREEN)✓ Check complete$(NC)"

fix: ## Automatically fix common issues
	@echo "$(BLUE)Running cargo fix...$(NC)"
	cd tachyon && $(CARGO) fix $(WORKSPACE_FLAGS) --allow-dirty --allow-staged
	@echo "$(GREEN)✓ Fixes applied$(NC)"

# ============================================================================
# Security Targets
# ============================================================================

audit: ## Run security audit (requires cargo-audit)
	@echo "$(BLUE)Running security audit...$(NC)"
	cd tachyon && cargo audit
	@echo "$(GREEN)✓ Audit complete$(NC)"

audit-json: ## Run security audit and output JSON
	cd tachyon && cargo audit --json

security-monitor: ## Run security monitoring script
	@echo "$(BLUE)Running security monitor...$(NC)"
	./scripts/security-monitor.sh
	@echo "$(GREEN)✓ Security monitoring complete$(NC)"

# ============================================================================
# Documentation Targets
# ============================================================================

doc: ## Generate and open documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	cd tachyon && $(CARGO) doc --workspace --exclude tachyon-testing --no-deps --open
	@echo "$(GREEN)✓ Documentation generated$(NC)"

doc-build: ## Build documentation without opening
	cd tachyon && $(CARGO) doc --workspace --exclude tachyon-testing --no-deps

# ============================================================================
# Clean Targets
# ============================================================================

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cd tachyon && $(CARGO) clean
	rm -rf tachyon/target/
	@echo "$(GREEN)✓ Clean complete$(NC)"

clean-all: ## Clean everything including caches
	@echo "$(BLUE)Cleaning everything...$(NC)"
	cd tachyon && $(CARGO) clean
	rm -rf tachyon/target/
	rm -rf ~/.cargo/registry/cache
	rm -rf ~/.cargo/git/checkouts
	@echo "$(GREEN)✓ Deep clean complete$(NC)"

# ============================================================================
# Development Server Targets
# ============================================================================

serve: build ## Build and run the development server
	@echo "$(BLUE)Starting development server...$(NC)"
	cd tachyon && ./target/debug/tachyon-server

serve-release: build-release ## Build and run the release server
	@echo "$(BLUE)Starting release server...$(NC)"
	cd tachyon && ./target/release/tachyon-server

run: ## Run the CLI application
	cd tachyon && $(CARGO) run -p tachyon-cli -- --help

# ============================================================================
# Deployment Targets
# ============================================================================

deploy-staging: ## Deploy to staging environment
	@echo "$(BLUE)Deploying to staging...$(NC)"
	./scripts/deploy.sh staging

deploy-production: ## Deploy to production environment
	@echo "$(RED)Deploying to production...$(NC)"
	./scripts/deploy.sh production

deploy: deploy-staging ## Alias for deploy-staging

# ============================================================================
# Docker Targets
# ============================================================================

docker-build: ## Build Docker images
	@echo "$(BLUE)Building Docker images...$(NC)"
	docker-compose -f scripts/docker-compose.yml build

docker-up: ## Start Docker containers
	@echo "$(BLUE)Starting Docker containers...$(NC)"
	docker-compose -f scripts/docker-compose.yml up -d

docker-down: ## Stop Docker containers
	@echo "$(BLUE)Stopping Docker containers...$(NC)"
	docker-compose -f scripts/docker-compose.yml down

docker-logs: ## View Docker logs
	docker-compose -f scripts/docker-compose.yml logs -f

docker-clean: ## Clean Docker containers and volumes
	@echo "$(BLUE)Cleaning Docker resources...$(NC)"
	docker-compose -f scripts/docker-compose.yml down -v --remove-orphans
	docker system prune -f

# ============================================================================
# Benchmark Targets
# ============================================================================

bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cd tachyon && $(CARGO) bench --workspace --exclude tachyon-testing

# ============================================================================
# Utility Targets
# ============================================================================

install: ## Install cargo tools (audit, tarpaulin, etc.)
	@echo "$(BLUE)Installing development tools...$(NC)"
	cargo install cargo-audit
	cargo install cargo-tarpaulin
	cargo install cargo-outdated
	@echo "$(GREEN)✓ Tools installed$(NC)"

update: ## Update dependencies
	@echo "$(BLUE)Updating dependencies...$(NC)"
	cd tachyon && $(CARGO) update

outdated: ## Check for outdated dependencies
	cd tachyon && cargo outdated --workspace --exclude tachyon-testing

watch: ## Watch for changes and run tests (requires cargo-watch)
	cd tachyon && cargo watch -x "test $(TEST_FLAGS)"

tree: ## Show project structure
	@tree -L 2 -I 'target|node_modules|.git' tachyon/

# ============================================================================
# CI/CD Targets
# ============================================================================

ci: fmt-check lint build test audit ## Run full CI pipeline
	@echo "$(GREEN)✓ CI pipeline complete$(NC)"

ci-lite: check test ## Run lightweight CI (check + test only)
	@echo "$(GREEN)✓ Lite CI complete$(NC)"

# ============================================================================
# Information Targets
# ============================================================================

version: ## Show project version
	@cd tachyon && $(CARGO) --version
	@echo "$(PROJECT_NAME) version:"
	@cd tachyon && grep "^version" Cargo.toml | head -1

status: ## Show git status and project info
	@echo "$(BLUE)Git Status:$(NC)"
	@git status --short
	@echo ""
	@echo "$(BLUE)Last Commit:$(NC)"
	@git log -1 --oneline
	@echo ""
	@echo "$(BLUE)Workspace Status:$(NC)"
	@cd tachyon && $(CARGO) metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sort

# ============================================================================
# Specialized Targets
# ============================================================================

init: ## Initialize a new Tachyon repository
	cd tachyon && $(CARGO) run -p tachyon-cli -- init

init-example: ## Initialize example repository
	cd tachyon && $(CARGO) run -p tachyon-cli -- init --path /tmp/tachyon-example --name "Example Repo"

quickstart-start: ## Start development server via quickstart
	./scripts/quickstart.sh start

quickstart-stop: ## Stop development server
	./scripts/quickstart.sh stop

quickstart-test: ## Run event-triggering crawl bot
	./scripts/quickstart.sh test

quickstart-status: ## Show quickstart status
	./scripts/quickstart.sh status

# event-crawl: ## Run comprehensive event-triggering crawl bot
# Disabled: tachyon/web/ directory does not exist
# 	cd tachyon/web && bun run event-crawler.ts

quickstart-clean: ## Clean all build artifacts
	./scripts/quickstart.sh clean

quickstart: quickstart-setup quickstart-start ## Full quickstart (setup + start)

# ============================================================================
# Documentation Preview Target
# ============================================================================

docs-preview: ## Build docs with SSG and serve locally
	@echo "$(BLUE)Building SSG binary...$(NC)"
	cd tachyon && $(CARGO) build --release -p tachyon-ssg
	@echo "$(BLUE)Generating documentation...$(NC)"
	cd tachyon && ./target/release/tachyon-ssg-cli --input documentation/ --output /tmp/tachyon-docs
	@echo "$(GREEN)✓ Docs generated at /tmp/tachyon-docs$(NC)"
	@echo "$(BLUE)Serving at http://localhost:8080 (Ctrl+C to stop)$(NC)"
	cd /tmp/tachyon-docs && python3 -m http.server 8080

link-check: ## Check for stale .specs/ references in markdown files
	@echo "$(BLUE)Checking for stale .specs/ references...$(NC)"
	@count=$$(rg '\.specs/' docs/ .adrs/ .reports/ .patterns/ --type md --files-with-matches 2>/dev/null | wc -l); \
	if [ "$$count" -gt 0 ]; then \
		echo "$(RED)Found $$count file(s) with stale .specs/ references:$(NC)"; \
		rg '\.specs/' docs/ .adrs/ .reports/ .patterns/ --type md --files-with-matches 2>/dev/null; \
		exit 1; \
	else \
		echo "$(GREEN)✓ No stale .specs/ references found$(NC)"; \
	fi
