# TACHYON: COMMAND LINE INTERFACE REFERENCE

**Document ID:** TACHYON-USER-005-V1.0
**Date:** February 2026
**Status:** Approved for Publication
**Classification:** User Documentation
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [CLI Overview](#2-cli-overview)
3. [Command Reference](#3-command-reference)
4. [Command Usage Patterns](#4-command-usage-patterns)
5. [Troubleshooting](#5-troubleshooting)
6. [References](#6-references)

---

## 1. INTRODUCTION

### 1.1. Purpose and Scope

This document provides a comprehensive reference for the Tachyon Command Line Interface (CLI). The Tachyon CLI serves as the primary interface for interacting with the Tachyon toolchain, enabling users to perform repository initialization, server management, desktop application launching, and documentation building from the command line.

The CLI reference is designed for:
- System administrators managing Tachyon deployments
- Developers integrating Tachyon into automated workflows
- Power users requiring efficient command-line access to all Tachyon features
- DevOps engineers scripting Tachyon operations

### 1.2. Document Dependencies

This document depends on the following documents:
- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-ADR-001](../../.specs/02_adrs/ADR-001-rust_language_selection.md) - Rust Language Selection
- [TACHYON-REQ-V1.0](../../.specs/06_requirements/requirements.md) - Requirements Specification
- [Configuration Guide](configuration_guide.md) - Configuration reference

### 1.3. Conventions Used in This Document

The following conventions are used throughout this document:

| Convention | Meaning |
|-----------|---------|
| `tachyon <command>` | Command syntax in monospace font |
| `[<option>]` | Optional parameter in square brackets |
| `<parameter>` | Required parameter in angle brackets |
| `--option=<value>` | Option with value assignment |

### 1.4. Exit Codes

The Tachyon CLI uses standard Unix exit codes to indicate execution status:

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success - operation completed successfully |
| 1 | General error - unspecified failure |
| 2 | Invalid usage - command syntax or argument error |
| 3 | Network error - connection or transmission failure |
| 4 | File system error - read/write/permission error |
| 5 | Authentication error - invalid credentials |
| 6 | Authorization error - insufficient permissions |
| 7 | Configuration error - invalid or missing configuration |
| 8 | Conflict error - resource conflict detected |
| 9 | Timeout error - operation exceeded time limit |
| 128 | Signal received - process terminated by signal |

---

## 2. CLI OVERVIEW

### 2.1. Command Structure

The Tachyon CLI follows a hierarchical command structure:

```
tachyon [<command>] [<options>] [<arguments>]
```

The available commands include:
- **init**: Initialize a new Tachyon repository
- **serve**: Start HTTP/2 and WebSocket servers
- **gui**: Launch Tauri desktop application
- **build**: Build documentation and bundle assets

### 2.2. Invocation

The Tachyon CLI is invoked through the `tachyon` executable, which must be available in the system PATH or referenced with its full path.

**Basic invocation:**
```bash
tachyon --help
```

**Full path invocation:**
```bash
/usr/local/bin/tachyon --help
```

### 2.3. Command Discovery

The CLI provides built-in help facilities for command discovery:

- `tachyon --help`: Display global help and available commands
- `tachyon <command> --help`: Display detailed help for a specific command

### 2.4. Configuration Sources

The Tachyon CLI reads configuration from multiple sources in the following precedence order (highest to lowest):

1. **Command-line options**: Directly specified options
2. **Environment variables**: Variables prefixed with `TACHYON_`
3. **Configuration files**: See [Configuration Guide](configuration_guide.md)
4. **Built-in defaults**: Default values embedded in the CLI

Configuration values from higher-priority sources override those from lower-priority sources.

### 2.5. Environment Variables

The following environment variables are supported:

| Variable | Description |
|----------|-------------|
| `TACHYON_REPO_PATH` | Default repository path |
| `TACHYON_SERVER_HOST` | Default server host |
| `TACHYON_SERVER_PORT` | Default server port |
| `TACHYON_LOG_LEVEL` | Log level (trace, debug, info, warn, error) |

### 2.6. Error Handling

The CLI provides comprehensive error handling with clear, actionable error messages:

- **Error codes**: Each error includes a unique error code for reference
- **Error messages**: Human-readable descriptions of the error condition
- **Recovery suggestions**: Recommended actions to resolve the error
- **Context information**: Relevant state and configuration details

---

## 3. COMMAND REFERENCE

### 3.1. `init` - Initialize Repository

Initialize a new Tachyon repository with Git and database setup.

**Syntax:**
```
tachyon init [OPTIONS]
```

**Options:**
| Option | Short | Type | Default | Description |
|--------|--------|------|----------|-------------|
| `--path` | `-p` | Path | Current directory | Path to initialize repository at |
| `--name` | `-n` | String | None | Repository name |
| `--skip-git` | | Flag | false | Skip git initialization |
| `--skip-database` | | Flag | false | Skip database initialization |
| `--force` | | Flag | false | Force initialization even if directory exists |
| `--interactive` | | Flag | false | Interactive setup wizard |

**Examples:**

Initialize in current directory:
```bash
tachyon init
```

Initialize with custom name:
```bash
tachyon init --name my-knowledge-base
```

Initialize at specific path:
```bash
tachyon init --path ~/Documents/tachyon-repo
```

Initialize with interactive wizard:
```bash
tachyon init --interactive
```

**Exit Codes:**
- `0`: Repository initialized successfully
- `1`: General initialization error
- `4`: File system error (permission denied, directory not empty)

**Related Implementation:** [`tachyon/crates/cli/src/commands/init.rs`](../../tachyon/crates/cli/src/commands/init.rs)

---

### 3.2. `serve` - Start Server

Start HTTP/2 and WebSocket servers for the Tachyon knowledge base.

**Syntax:**
```
tachyon serve [OPTIONS]
```

**Options:**
| Option | Short | Type | Default | Description |
|--------|--------|------|----------|-------------|
| `--host` | `-h` | String | "127.0.0.1" | Host address to bind to |
| `--http-port` | `-p` | Integer | 8080 | HTTP port |
| `--ws-port` | | Integer | 8081 | WebSocket port |
| `--tls-enabled` | | Flag | false | Enable TLS |
| `--tls-cert` | | Path | None | TLS certificate path |
| `--tls-key` | | Path | None | TLS key path |
| `--repo-path` | `-r` | Path | ".tachyon" | Repository path |
| `--max-body-size` | | Integer | None | Maximum request body size in bytes |
| `--timeout` | | Integer | None | Request timeout in seconds |

**Examples:**

Start server with default settings:
```bash
tachyon serve
```

Start server on all interfaces:
```bash
tachyon serve --host 0.0.0.0
```

Start server with custom ports:
```bash
tachyon serve --http-port 8443 --ws-port 8444
```

Start server with TLS:
```bash
tachyon serve --tls-enabled --tls-cert /path/to/cert.pem --tls-key /path/to/key.pem
```

**Exit Codes:**
- `0`: Server started successfully
- `1`: Server failed to start
- `3`: Network error (port already in use, address not available)
- `4`: File system error (repository not found)

**Related Implementation:** [`tachyon/crates/cli/src/commands/serve.rs`](../../tachyon/crates/cli/src/commands/serve.rs)

---

### 3.3. `gui` - Launch Desktop Application

Launch the Tauri desktop application for local-first knowledge base management.

**Syntax:**
```
tachyon gui [OPTIONS]
```

**Options:**
| Option | Short | Type | Default | Description |
|--------|--------|------|----------|-------------|
| `--repo-path` | `-r` | Path | ".tachyon" | Repository path |
| `--dev-tools` | | Flag | false | Enable dev tools |
| `--window-width` | | Integer | None | Window width |
| `--window-height` | | Integer | None | Window height |
| `--start-maximized` | | Flag | false | Start maximized |
| `--start-minimized` | | Flag | false | Start minimized |
| `--server-host` | | String | None | Server host to connect to |
| `--server-port` | | Integer | None | Server port to connect to |

**Examples:**

Launch GUI with default settings:
```bash
tachyon gui
```

Launch GUI with custom window size:
```bash
tachyon gui --window-width 1920 --window-height 1080
```

Launch GUI connected to remote server:
```bash
tachyon gui --server-host tachyon.example.com --server-port 8443
```

Launch GUI with dev tools enabled:
```bash
tachyon gui --dev-tools
```

**Exit Codes:**
- `0`: GUI launched successfully
- `1`: GUI failed to launch
- `4`: File system error (repository not found)

**Related Implementation:** [`tachyon/crates/cli/src/commands/gui.rs`](../../tachyon/crates/cli/src/commands/gui.rs)

---

### 3.4. `build` - Build Documentation

Build documentation and bundle assets for deployment.

**Syntax:**
```
tachyon build [OPTIONS]
```

**Options:**
| Option | Short | Type | Default | Description |
|--------|--------|------|----------|-------------|
| `--repo-path` | `-r` | Path | ".tachyon" | Repository path |
| `--output-dir` | `-o` | Path | None | Output directory |
| `--gen-docs` | | Flag | false | Generate documentation |
| `--minify` | | Flag | false | Minify assets |
| `--source-maps` | | Flag | false | Generate source maps |
| `--clean` | | Flag | false | Clean build (remove output directory first) |
| `--verbose` | `-v` | Flag | false | Verbose output |

**Examples:**

Build with default settings:
```bash
tachyon build
```

Build with documentation generation:
```bash
tachyon build --gen-docs
```

Build with minification:
```bash
tachyon build --minify --source-maps
```

Build to custom output directory:
```bash
tachyon build --output-dir ./dist
```

Clean build with verbose output:
```bash
tachyon build --clean --verbose
```

**Exit Codes:**
- `0`: Build completed successfully
- `1`: Build failed
- `4`: File system error

**Related Implementation:** [`tachyon/crates/cli/src/commands/build.rs`](../../tachyon/crates/cli/src/commands/build.rs)

---

### 3.5. `--help` - Display Help

Display help information for the CLI or specific commands.

**Syntax:**
```
tachyon --help
tachyon <command> --help
```

**Examples:**

Display global help:
```bash
tachyon --help
```

Display help for specific command:
```bash
tachyon init --help
```

**Exit Codes:**
- `0`: Help displayed successfully

---

### 3.6. `--version` - Display Version

Display version information for the Tachyon CLI.

**Syntax:**
```
tachyon --version
tachyon -V
```

**Output:**
```
Tachyon CLI v1.0.0
```

**Exit Codes:**
- `0`: Version displayed successfully

---

## 4. COMMAND USAGE PATTERNS

### 4.1. Typical Workflows

#### 4.1.1. Local Development Workflow

```bash
# Initialize a new repository
tachyon init --name my-knowledge-base --interactive

# Start the desktop application
tachyon gui

# Build for deployment
tachyon build --gen-docs --minify
```

#### 4.1.2. Server Deployment Workflow

```bash
# Initialize repository on server
tachyon init --name production-repo --path /var/lib/tachyon

# Start server with TLS
tachyon serve \
  --host 0.0.0.0 \
  --http-port 8443 \
  --tls-enabled \
  --tls-cert /etc/ssl/certs/tachyon.crt \
  --tls-key /etc/ssl/private/tachyon.key \
  --repo-path /var/lib/tachyon
```

#### 4.1.3. Collaborative Workflow

```bash
# Initialize repository
tachyon init --name team-knowledge

# Start server for team access
tachyon serve --host 0.0.0.0 --http-port 8443

# Team members connect via GUI
tachyon gui --server-host tachyon.example.com --server-port 8443
```

### 4.2. Configuration Management

Configuration is managed through the following methods:

1. **Command-line options**: Highest priority
2. **Environment variables**: Prefix `TACHYON_`
3. **Configuration files**: See [Configuration Guide](configuration_guide.md)

---

## 5. TROUBLESHOOTING

### 5.1. Common Issues

#### 5.1.1. Initialization Fails

**Symptom:** `tachyon init` fails with directory not empty error

**Solution:**
```bash
# Use force flag to initialize in existing directory
tachyon init --force

# Or initialize in a new directory
tachyon init --path ./new-repo
```

#### 5.1.2. Server Port Already in Use

**Symptom:** `tachyon serve` fails with address already in use error

**Solution:**
```bash
# Find process using the port
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows

# Kill the process or use a different port
tachyon serve --http-port 8081
```

#### 5.1.3. Repository Not Found

**Symptom:** GUI or serve command fails with repository not found error

**Solution:**
```bash
# Specify the correct repository path
tachyon gui --repo-path /path/to/repo
tachyon serve --repo-path /path/to/repo

# Or initialize a new repository
tachyon init --path /path/to/new-repo
```

---

## 6. REFERENCES

### 6.1. Related Documentation

- [User Guide](user_guide.md) - Comprehensive user guide
- [Configuration Guide](configuration_guide.md) - Configuration reference
- [FAQ](faq.md) - Frequently asked questions
- [Troubleshooting Guide](troubleshooting_guide.md) - Detailed troubleshooting

### 6.2. Implementation Details

- [CLI Implementation](../../tachyon/crates/cli/src/) - CLI source code
- [Core Library](../../tachyon/crates/core/src/) - Core types and utilities
- [Server Implementation](../../tachyon/crates/server/src/) - Server implementation
- [Desktop Implementation](../../tachyon/crates/desktop/src/) - Desktop implementation

### 6.3. Architecture Documents

- [System Architecture Overview](../architecture/system_architecture_overview.md)
- [Deployment Architecture](../architecture/deployment_architecture.md)
- [Security Architecture](../security/security_architecture.md)

---

**End of Document**
