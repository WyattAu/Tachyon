# Tachyon Quickstart Guide

This guide helps you get started with Tachyon quickly by cloning, building, and running the project with a single command.

## Prerequisites

Before starting, ensure you have the following installed:

| Requirement | Version | Install |
|-------------|---------|---------|
| **Rust** | 1.77+ | [rustup.rs](https://rustup.rs) |
| **Cargo** | 1.77+ | Comes with Rust |
| **Bun** or **Node.js** | Bun 1.2+ / Node 18+ | [bun.sh](https://bun.sh) or [nodejs.org](https://nodejs.org) |
| **Git** | 2.30+ | Package manager |

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/tachyon.git
cd tachyon
```

### 2. Run Setup

```bash
./scripts/quickstart.sh setup
```

This will:
- Check all prerequisites
- Build the Rust workspace
- Install web frontend dependencies
- Create a starter template at `/tmp/tachyon-starter`
- Copy `.env.example` to `.env` with default settings

### 3. Start the Server

```bash
./scripts/quickstart.sh start
```

The server will be available at:
- **API**: http://localhost:8080
- **Health**: http://localhost:8080/health
- **Metrics**: http://localhost:8080/metrics

### 4. Run Tests

```bash
./scripts/quickstart.sh test
```

This runs the comprehensive event-triggering crawl bot that:
- Tests all UI interactions
- Triggers all application events
- Captures runtime errors
- Generates detailed reports

## Available Commands

```bash
./scripts/quickstart.sh [command]
```

| Command | Description |
|---------|-------------|
| `setup` | Clone and build the project (first-time setup) |
| `start` | Start the development server |
| `stop` | Stop the development server |
| `test` | Run the event-triggering crawl bot |
| `status` | Show project status |
| `clean` | Clean all build artifacts |
| `help` | Show help message |

## Using Makefile

Alternatively, you can use the Makefile:

```bash
# First-time setup
make build

# Start development server
make serve

# Initialize example repository
make init-example

# Run tests
make test

# Run full CI pipeline
make ci
```

## Starter Template

The starter template provides a basic project structure for Tachyon:

```
/tmp/tachyon-starter/
├── nodes/         # Node data files
├── edges/         # Edge relationship files
├── documents/     # Document content files
├── db/            # SQLite database
├── cache/         # Cached data
├── logs/          # Log files
├── backup/        # Database backups
├── .gitignore
├── README.md
└── tachyon.toml   # Configuration
```

### Create a Custom Project

```bash
# Using CLI
cd tachyon && cargo run -p tachyon-cli -- init --path ~/my-project --name "My Project"

# Using Makefile
make init
```

## Event Crawl Bot

The event crawl bot (`tachyon/web/event-crawler.ts`) is a comprehensive testing tool that:

### Features
- **Page Testing**: Tests all major pages for errors
- **Event Triggering**: Triggers all UI events (clicks, keyboard shortcuts, form inputs)
- **Error Capture**: Captures console errors, page errors, network failures, and stored app errors
- **Screenshot Recording**: Takes before/after screenshots of each test
- **Detailed Reports**: Generates JSON reports with all captured data

### Event Categories Tested
1. **Theme Events**: Toggle dark/light mode
2. **Search Events**: Focus, input, clear
3. **Navigation Events**: Menu toggles, link clicks, dropdowns
4. **Keyboard Shortcuts**: Ctrl+S, Ctrl+N, Ctrl+Shift+D
5. **Editor Events**: Focus, content changes
6. **Auth Events**: Login form, authentication
7. **Form Events**: Validation, submission
8. **Scroll Events**: Page scroll, scroll to top
9. **Modal Events**: Open, close dialogs
10. **HTMX Events**: Dynamic content loading
11. **Responsive Events**: Viewport resizing
12. **Accessibility Events**: Tab navigation, Enter key

### Running the Crawl Bot

```bash
# Ensure server is running
./scripts/quickstart.sh start

# Run crawl bot
./scripts/quickstart.sh test

# Or directly
cd tachyon/web && bun run event-crawler.ts
```

### Reports Location

Reports are saved to `tachyon/web/crawl-results/`:
- `crawl-report-{timestamp}.json` - Timestamped report
- `crawl-report-latest.json` - Latest report (overwritten each run)
- `screenshots/` - Before/after screenshots

## Troubleshooting

### Server Won't Start

```bash
# Check if port is in use
lsof -i :8080

# Kill existing process
kill -9 $(lsof -t -i:8080)

# Try again
./scripts/quickstart.sh start
```

### Build Fails

```bash
# Clean and rebuild
./scripts/quickstart.sh clean
./scripts/quickstart.sh setup
```

### Tests Fail

```bash
# Check server health
curl http://localhost:8080/health

# View logs
cat logs/server.log

# Run with verbose output
cd tachyon && RUST_LOG=debug cargo run -p tachyon-server
```

## Next Steps

1. **Explore the API**: Visit http://localhost:8080/api/v1/
2. **Create Documents**: Use the web interface or API
3. **Configure Authentication**: Edit `.env` for production settings
4. **Deploy**: See `docs/DEPLOYMENT.md` for production deployment

## Additional Resources

- [User Guide](../docs/user/user_guide.md)
- [API Documentation](../docs/api/)
- [Architecture Overview](../docs/architecture/)
- [Deployment Guide](../docs/DEPLOYMENT.md)
