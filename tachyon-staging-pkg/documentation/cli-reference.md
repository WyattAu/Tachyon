---
title: CLI Reference
description: Command-line interface for Tachyon
order: 9
tags: [cli, reference]
---

# CLI Reference

The `tachyon-cli` binary provides commands for building, serving, and managing Tachyon.

## Installation

```bash
cargo install --path tachyon/crates/cli
```

## Commands

### `init`

Initialize a new Tachyon project:

```bash
tachyon init [path]
```

Creates directory structure, `README.md`, and `.gitignore`.

### `serve`

Start the backend server:

```bash
tachyon serve [options]
```

Options:

| Flag | Default | Description |
|------|---------|-------------|
| `--host` | `0.0.0.0` | Bind address |
| `--port` | `8080` | Bind port |
| `--database-url` | env var | PostgreSQL connection string |
| `--jwt-secret` | env var | JWT signing secret |

### `build`

Build the frontend for production:

```bash
tachyon build [path]
```

Scans markdown files, processes frontmatter, and generates output.

### `gui`

Launch the desktop application:

```bash
tachyon gui [options]
```

Options:

| Flag | Default | Description |
|------|---------|-------------|
| `--server-host` | `127.0.0.1` | Embedded server host |
| `--server-port` | `8080` | Embedded server port |

## Using with `just`

The project includes a `justfile` for common tasks:

```bash
just build    # Build all crates
just test     # Run backend tests
just lint     # Check formatting + clippy
just dev      # Lint + test + build
```

## Using with `make`

The root Makefile provides additional targets:

```bash
make test             # Run all tests
make coverage         # Generate coverage report
make docker-build     # Build Docker image
make quickstart       # Quick setup with Docker
```

## Further Reading

- [Getting Started](getting-started.html) - Setup guide
- [Configuration](configuration.html) - Environment variables
