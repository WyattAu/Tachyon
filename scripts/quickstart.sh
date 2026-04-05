#!/bin/bash
# =============================================================================
# Tachyon Quickstart Script
# =============================================================================
# This script provides a one-command setup to clone, build, and run Tachyon
# for testing and development purposes.
#
# Usage:
#   ./scripts/quickstart.sh [command]
#
# Commands:
#   setup     - Clone and build the project (first-time setup)
#   start     - Start the development server
#   stop      - Stop the development server
#   test      - Run the event-triggering crawl bot
#   clean     - Clean all build artifacts
#   status    - Show project status
#   help      - Show this help message
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TACHYON_DIR="$PROJECT_DIR/tachyon"
WEB_DIR="$TACHYON_DIR/web"
DATA_DIR="$TACHYON_DIR/data"
LOG_DIR="$PROJECT_DIR/logs"
PID_FILE="$PROJECT_DIR/.tachyon.pid"
SERVER_PORT=${SERVER_PORT:-8080}
WEB_PORT=${WEB_PORT:-3000}

# Print banner
print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║                    TACHYON QUICKSTART                         ║"
    echo "║         Knowledge Management System - Development             ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Check prerequisites
check_prerequisites() {
    echo -e "${BLUE}[Checking Prerequisites]${NC}"
    
    local missing=()
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        missing+=("rustc (install from https://rustup.rs)")
    else
        echo -e "  ${GREEN}✓${NC} Rust $(rustc --version)"
    fi
    
    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        missing+=("cargo (install from https://rustup.rs)")
    else
        echo -e "  ${GREEN}✓${NC} Cargo $(cargo --version)"
    fi
    
    # Check Bun or Node
    if command -v bun &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} Bun $(bun --version)"
    elif command -v node &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} Node $(node --version)"
    else
        missing+=("bun or node (install bun from https://bun.sh)")
    fi
    
    # Check Git
    if ! command -v git &> /dev/null; then
        missing+=("git")
    else
        echo -e "  ${GREEN}✓${NC} Git $(git --version | cut -d' ' -f3)"
    fi
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "\n${RED}Missing prerequisites:${NC}"
        for item in "${missing[@]}"; do
            echo -e "  ${RED}✗${NC} $item"
        done
        echo -e "\nPlease install missing prerequisites and try again."
        exit 1
    fi
    
    echo ""
}

# Setup the project
setup_project() {
    echo -e "${BLUE}[Setting Up Project]${NC}"
    
    # Create necessary directories
    mkdir -p "$DATA_DIR" "$LOG_DIR"
    
    # Copy environment file if not exists
    if [ ! -f "$PROJECT_DIR/.env" ]; then
        if [ -f "$PROJECT_DIR/.env.example" ]; then
            cp "$PROJECT_DIR/.env.example" "$PROJECT_DIR/.env"
            echo -e "  ${GREEN}✓${NC} Created .env from .env.example"
        fi
    fi
    
    # Build Rust workspace
    echo -e "  ${YELLOW}Building Rust workspace...${NC}"
    cd "$TACHYON_DIR"
    cargo build --workspace --exclude tachyon-testing 2>&1 | while read -r line; do
        if [[ $line == *"error"* ]]; then
            echo -e "    ${RED}$line${NC}"
        elif [[ $line == *"Finished"* ]] || [[ $line == *"Compiling"* ]]; then
            echo -e "    ${BLUE}$line${NC}"
        fi
    done
    echo -e "  ${GREEN}✓${NC} Rust workspace built"
    
    # Install web dependencies
    echo -e "  ${YELLOW}Installing web dependencies...${NC}"
    cd "$WEB_DIR"
    if command -v bun &> /dev/null; then
        bun install --silent
    else
        npm install --silent
    fi
    echo -e "  ${GREEN}✓${NC} Web dependencies installed"
    
    # Initialize example repository
    echo -e "  ${YELLOW}Initializing example repository...${NC}"
    cd "$TACHYON_DIR"
    cargo run -p tachyon-cli -- init --path /tmp/tachyon-starter --name "Starter Template" --force 2>/dev/null || true
    echo -e "  ${GREEN}✓${NC} Starter template created at /tmp/tachyon-starter"
    
    echo -e "\n${GREEN}✓ Setup complete!${NC}"
    echo -e "\nTo start the development server, run:"
    echo -e "  ${BLUE}./scripts/quickstart.sh start${NC}"
}

# Start development server
start_server() {
    echo -e "${BLUE}[Starting Development Server]${NC}"
    
    # Check if already running
    if [ -f "$PID_FILE" ] && kill -0 $(cat "$PID_FILE") 2>/dev/null; then
        echo -e "  ${YELLOW}Server already running (PID: $(cat $PID_FILE))${NC}"
        echo -e "  Run '${BLUE}./scripts/quickstart.sh stop${NC}' first to restart."
        return 1
    fi
    
    # Ensure build exists
    if [ ! -f "$TACHYON_DIR/target/debug/tachyon-server" ]; then
        echo -e "  ${YELLOW}Server binary not found. Building...${NC}"
        cd "$TACHYON_DIR"
        cargo build -p tachyon-server
    fi
    
    # Start server in background
    cd "$TACHYON_DIR"
    TACHYON_LOG=info ./target/debug/tachyon-server > "$LOG_DIR/server.log" 2>&1 &
    SERVER_PID=$!
    echo $SERVER_PID > "$PID_FILE"
    
    # Wait for server to start
    echo -e "  ${YELLOW}Waiting for server to start...${NC}"
    for i in {1..30}; do
        if curl -s "http://localhost:$SERVER_PORT/health" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} Server started on port $SERVER_PORT"
            break
        fi
        sleep 1
    done
    
    # Check if server is actually running
    if ! curl -s "http://localhost:$SERVER_PORT/health" > /dev/null 2>&1; then
        echo -e "  ${RED}✗ Server failed to start. Check logs at $LOG_DIR/server.log${NC}"
        rm -f "$PID_FILE"
        return 1
    fi
    
    echo ""
    echo -e "${GREEN}Server is running!${NC}"
    echo -e "  API:     ${BLUE}http://localhost:$SERVER_PORT${NC}"
    echo -e "  Health:  ${BLUE}http://localhost:$SERVER_PORT/health${NC}"
    echo -e "  Metrics: ${BLUE}http://localhost:$SERVER_PORT/metrics${NC}"
    echo -e "  Logs:    ${BLUE}$LOG_DIR/server.log${NC}"
    echo ""
    echo -e "To stop the server, run: ${BLUE}./scripts/quickstart.sh stop${NC}"
}

# Stop development server
stop_server() {
    echo -e "${BLUE}[Stopping Development Server]${NC}"
    
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 $PID 2>/dev/null; then
            kill $PID
            echo -e "  ${GREEN}✓${NC} Server stopped (PID: $PID)"
        else
            echo -e "  ${YELLOW}Server process not found${NC}"
        fi
        rm -f "$PID_FILE"
    else
        echo -e "  ${YELLOW}No PID file found. Server may not be running.${NC}"
    fi
}

# Run the event-triggering crawl bot
run_crawl_bot() {
    echo -e "${BLUE}[Running Event-Triggering Crawl Bot]${NC}"
    
    # Check if server is running
    if ! curl -s "http://localhost:$SERVER_PORT/health" > /dev/null 2>&1; then
        echo -e "  ${YELLOW}Server not running. Starting...${NC}"
        start_server
        sleep 2
    fi
    
    # Run the crawl bot
    echo -e "  ${YELLOW}Executing crawl bot...${NC}"
    cd "$WEB_DIR"
    
    if command -v bun &> /dev/null; then
        bun run event-crawler.ts
    else
        npx tsx event-crawler.ts
    fi
    
    CRAWL_EXIT=$?
    
    if [ $CRAWL_EXIT -eq 0 ]; then
        echo -e "\n${GREEN}✓ Crawl completed successfully!${NC}"
    else
        echo -e "\n${RED}✗ Crawl completed with errors.${NC}"
    fi
    
    echo -e "\nCrawl reports saved to: ${BLUE}$WEB_DIR/crawl-results/${NC}"
    
    return $CRAWL_EXIT
}

# Show project status
show_status() {
    echo -e "${BLUE}[Project Status]${NC}\n"
    
    # Server status
    if [ -f "$PID_FILE" ] && kill -0 $(cat "$PID_FILE") 2>/dev/null; then
        echo -e "  Server:  ${GREEN}Running${NC} (PID: $(cat $PID_FILE))"
        echo -e "  API:     http://localhost:$SERVER_PORT"
    else
        echo -e "  Server:  ${YELLOW}Stopped${NC}"
    fi
    
    # Build status
    if [ -f "$TACHYON_DIR/target/debug/tachyon-server" ]; then
        echo -e "  Build:   ${GREEN}Ready${NC}"
    else
        echo -e "  Build:   ${YELLOW}Not built${NC}"
    fi
    
    # Web dependencies
    if [ -d "$WEB_DIR/node_modules" ]; then
        echo -e "  Web:     ${GREEN}Dependencies installed${NC}"
    else
        echo -e "  Web:     ${YELLOW}Dependencies not installed${NC}"
    fi
    
    # Starter template
    if [ -d "/tmp/tachyon-starter" ]; then
        echo -e "  Starter: ${GREEN}Available${NC} at /tmp/tachyon-starter"
    else
        echo -e "  Starter: ${YELLOW}Not created${NC}"
    fi
    
    echo ""
    
    # Recent errors from crawl
    if [ -f "$WEB_DIR/crawl-results/crawl-report-latest.json" ]; then
        ERROR_COUNT=$(grep -o '"totalErrors"' "$WEB_DIR/crawl-results/crawl-report-latest.json" | head -1 | grep -o '[0-9]*' || echo "0")
        if [ "$ERROR_COUNT" -gt 0 ]; then
            echo -e "  ${YELLOW}Last crawl had $ERROR_COUNT error(s)${NC}"
            echo -e "  See: $WEB_DIR/crawl-results/crawl-report-latest.json"
        else
            echo -e "  ${GREEN}Last crawl: No errors${NC}"
        fi
    fi
}

# Clean build artifacts
clean_project() {
    echo -e "${BLUE}[Cleaning Project]${NC}"
    
    # Stop server first
    stop_server 2>/dev/null || true
    
    # Clean Rust build
    echo -e "  ${YELLOW}Cleaning Rust build artifacts...${NC}"
    cd "$TACHYON_DIR"
    cargo clean 2>/dev/null || true
    
    # Clean web dependencies
    echo -e "  ${YELLOW}Cleaning web dependencies...${NC}"
    rm -rf "$WEB_DIR/node_modules"
    rm -rf "$WEB_DIR/dist"
    
    # Clean logs
    echo -e "  ${YELLOW}Cleaning logs...${NC}"
    rm -rf "$LOG_DIR"
    
    # Clean crawl results
    rm -rf "$WEB_DIR/crawl-results"
    
    echo -e "  ${GREEN}✓${NC} Clean complete"
}

# Show help
show_help() {
    echo ""
    echo -e "${BLUE}Usage:${NC} ./scripts/quickstart.sh [command]"
    echo ""
    echo -e "${BLUE}Commands:${NC}"
    echo "  setup     Clone and build the project (first-time setup)"
    echo "  start     Start the development server"
    echo "  stop      Stop the development server"
    echo "  test      Run the event-triggering crawl bot"
    echo "  clean     Clean all build artifacts"
    echo "  status    Show project status"
    echo "  help      Show this help message"
    echo ""
    echo -e "${BLUE}Examples:${NC}"
    echo "  ./scripts/quickstart.sh setup    # First-time setup"
    echo "  ./scripts/quickstart.sh start    # Start server"
    echo "  ./scripts/quickstart.sh test     # Run crawl tests"
    echo "  ./scripts/quickstart.sh stop     # Stop server"
    echo ""
}

# Main entry point
main() {
    print_banner
    
    COMMAND=${1:-help}
    
    case $COMMAND in
        setup)
            check_prerequisites
            setup_project
            ;;
        start)
            start_server
            ;;
        stop)
            stop_server
            ;;
        test|crawl)
            run_crawl_bot
            ;;
        clean)
            clean_project
            ;;
        status)
            show_status
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}Unknown command: $COMMAND${NC}"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
