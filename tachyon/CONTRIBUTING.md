# Contributing to Tachyon

Thank you for your interest in contributing! This guide covers everything you need to set up the development environment, understand the codebase, and submit changes.

For a deeper dive into the architecture and Leptos patterns, see [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md).

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Building](#building)
5. [Testing](#testing)
6. [Code Style](#code-style)
7. [Commit Messages](#commit-messages)
8. [Pull Request Process](#pull-request-process)
9. [Architecture Overview](#architecture-overview)
10. [Common Tasks](#common-tasks)

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust (stable) | 1.85+ | `rustup default stable` |
| Trunk | latest | `cargo install trunk` (for WASM builds) |
| PostgreSQL | 16+ | Local or Docker |
| Node.js | 18+ | Required for Trunk and Playwright |
| Docker | latest | Optional — for infrastructure services |
| Nix | latest | Optional but recommended — see below |

### Optional: Nix Development Environment

The repository includes a Nix flake that provisions the full toolchain (Rust with WASM target, Trunk, PostgreSQL 16, Node.js, Bun, system libraries for Tauri):

```bash
# From repo root
nix develop
```

This drops you into a shell with everything pre-installed. See [flake.nix](../flake.nix) for details.

**Important:** When running inside the Nix shell, cargo sometimes fails due to `/tmp` permission issues. Prefix cargo commands with `TMPDIR=/tmp`:

```bash
TMPDIR=/tmp cargo build
TMPDIR=/tmp cargo test
```

CI also uses this prefix for benchmark jobs.

---

## Development Setup

### 1. Clone the repository

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
```

### 2a. Nix setup (recommended)

```bash
# From repo root
nix develop
cd tachyon
```

### 2b. Manual setup

```bash
# Install Rust components
rustup component add rustfmt clippy

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install build tools
cargo install trunk cargo-watch cargo-audit

# Verify
rustc --version
trunk --version
```

### 3. Start infrastructure

```bash
# Start PostgreSQL and Redis via Docker
docker compose up -d postgres redis
```

Or use a local PostgreSQL 16 instance. Create the database and user:

```sql
CREATE USER tachyon WITH PASSWORD 'tachyon';
CREATE DATABASE tachyon OWNER tachyon;
```

### 4. Configure environment

```bash
cp .env.example .env
# Edit .env — set at minimum:
#   DATABASE_URL=postgres://tachyon:tachyon@localhost:5432/tachyon
#   JWT_SECRET=change-me-to-a-random-64-char-string
```

See [.env.example](.env.example) for all available variables and their defaults.

### 5. Run database migrations

```bash
cargo run -p tachyon-server -- migrate
```

### 6. Build and run

```bash
cargo build --workspace
cargo run -p tachyon-server          # Backend on http://127.0.0.1:8080
cd crates/frontend && trunk serve   # Frontend dev server
```

Use `just setup` to automate steps 2b–5 in one command.

---

## Project Structure

Tachyon is a Cargo workspace with 16 crates under `crates/`:

| Crate | Path | Description |
|-------|------|-------------|
| `tachyon-core` | `crates/core` | Shared domain types, auth (argon2), error handling, business logic |
| `tachyon-server` | `crates/server` | Axum HTTP server, REST API (29 routes), WebSocket, middleware, OAuth2, billing |
| `tachyon-database` | `crates/database` | PostgreSQL schema, migrations (28), repository layer, connection pooling |
| `tachyon-frontend` | `crates/frontend` | Leptos 0.8 WASM frontend (CSR), Tailwind CSS, 90+ components |
| `tachyon-rbac` | `crates/rbac` | Role-based access control engine (policies, permissions, hierarchy) |
| `tachyon-search` | `crates/search` | Tantivy full-text search with faceted filtering |
| `tachyon-renderer` | `crates/renderer` | Markdown-to-HTML rendering pipeline (pulldown-cmark, XSS sanitization) |
| `tachyon-cli` | `crates/cli` | CLI tool for administrative tasks |
| `tachyon-desktop` | `crates/desktop` | Tauri 2.0 beta desktop application wrapper |
| `tachyon-testing` | `crates/testing` | Shared test utilities, fixtures, mocks, fuzzing harness, benchmarks |
| `tachyon-plugin-runtime` | `crates/plugin-runtime` | WASM plugin runtime (Wasmtime/WASI) |
| `tachyon-editor` | `crates/editor` | Native Rust text editor engine (ropey, CRDT via yrs) |
| `tachyon-import-export` | `crates/import-export` | Document import/export (ZIP archives, YAML) |
| `tachyon-ssg` | `crates/ssg` | Static site generator with multi-language, RTL, sitemap, RSS |
| `tachyon-storage` | `crates/storage` | Pluggable storage backends (SQLite, S3) |
| `tachyon-benchmarks` | `crates/benchmarks` | Criterion benchmarks (search, renderer, RBAC, database) |

---

## Building

```bash
# Build all workspace members (debug)
cargo build --workspace

# Build backend in release mode
cargo build --release -p tachyon-server

# Build frontend WASM (production)
cd crates/frontend && trunk build --release

# Build everything in release mode
just build-release

# Build desktop app (Tauri)
just build-desktop

# Check WASM compilation without full build
cargo check --workspace --target wasm32-unknown-unknown
```

The release profile is optimized for WASM bundle size (`opt-level = "z"`, LTO, single codegen unit, `panic = "abort"`).

---

## Testing

### Unit tests

```bash
# All workspace tests (excludes frontend and testing crate)
cargo test --workspace --exclude tachyon-frontend --exclude tachyon-testing

# Specific crate
cargo test -p tachyon-core

# Specific test by name
cargo test test_permission_level -- --nocapture

# With output
cargo test -- --nocapture
```

### Integration tests (requires PostgreSQL)

Integration tests need a running PostgreSQL instance. Set the database URL:

```bash
export DATABASE_URL=postgres://tachyon:tachyon@localhost:5432/tachyon_test
export TEST_DATABASE_URL=postgres://tachyon:tachyon@localhost:5432/tachyon_test

# Run integration test suite
cargo test --workspace --exclude tachyon-frontend --exclude tachyon-testing --test integration
```

CI runs integration tests against a PostgreSQL 16 service container.

### E2E tests (Playwright)

E2E tests live outside the Rust workspace and require a running server:

```bash
npx playwright install
npx playwright test
```

### Coverage

```bash
# Enforce minimum coverage threshold (reads .coverage.toml)
just test-coverage

# Report without enforcing threshold
just test-coverage-report
```

### Benchmarks

```bash
# Run Criterion benchmarks
TMPDIR=/tmp cargo bench -p tachyon-benchmarks
```

---

## Code Style

All code must pass the following checks (these run in CI):

```bash
# Format check
cargo fmt --all -- --check

# Auto-format
cargo fmt --all

# Clippy with warnings as errors
cargo clippy --workspace --all-targets -- -D warnings
```

Run both at once with `just lint`.

### Conventions

- **Formatting:** `rustfmt` with default configuration. Run `cargo fmt` before committing.
- **Clippy:** All warnings are treated as errors (`-D warnings`). Fix all clippy lints before pushing.
- **Naming:** Follow standard Rust conventions (`UpperCamelCase` for types, `snake_case` for functions and variables, `SCREAMING_SNAKE_CASE` for constants).
- **Error handling:** Prefer `thiserror` for library error types, `anyhow` for application errors. Avoid `unwrap()` in non-test production code.
- **Dependencies:** Add new workspace dependencies to `[workspace.dependencies]` in the root `Cargo.toml`, then reference with `{ workspace = true }` in crate-level `Cargo.toml` files.

---

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) style:

```
type(scope): description

[optional body]
```

**Types:**

| Type | Purpose |
|------|---------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting, no code change |
| `refactor` | Code restructuring, no behavior change |
| `test` | Adding or updating tests |
| `chore` | Tooling, CI, dependencies |
| `perf` | Performance improvement |

**Examples:**

```
feat(server): add document review workflow endpoint
fix(rbac): resolve permission inheritance for nested roles
test(database): add integration tests for migration rollback
chore: update axum to 0.8
```

---

## Pull Request Process

### Before submitting

1. **Create a branch** from `develop` (or `main` for hotfixes)
2. **Run the full CI check locally:**

   ```bash
   just ci
   ```

   This runs: format check, clippy, tests, backend build, frontend build.

3. **Write tests** for new functionality. Coverage is tracked via `cargo tarpaulin`.
4. **Update documentation** if you changed public APIs, added routes, or modified configuration.

### CI requirements

All PRs must pass these CI jobs:

| Job | What it does |
|-----|-------------|
| **Check** | `cargo check` for workspace + WASM target |
| **Lint** | `cargo fmt --check` + `cargo clippy -D warnings` |
| **Test** | Unit tests (`--lib`) + integration tests (with PostgreSQL 16) |
| **Build** | Release build of `tachyon-server` |
| **Build Frontend** | `trunk build --release` for WASM frontend |
| **Benchmarks** | `cargo bench` (on push to main only) |
| **Docker Build** | Multi-stage Docker build (on push to main only) |

### PR checklist

- [ ] All CI jobs pass
- [ ] New features have tests
- [ ] No `unwrap()` in new production code
- [ ] Clippy clean (`-D warnings`)
- [ ] Documentation updated if needed
- [ ] Commit messages follow conventional commits

---

## Architecture Overview

### Key design decisions

**Axum + Leptos WASM (CSR):** The backend is an Axum 0.8 HTTP server serving a REST API. The frontend compiles to WebAssembly via Leptos 0.8 in client-side rendering mode, built with Trunk. Communication is HTTP/JSON for REST and WebSocket for real-time collaboration.

**RBAC (Role-Based Access Control):** A dedicated `tachyon-rbac` crate implements a policy-based access control engine with role hierarchy (admin > editor > writer > reader), resource-level policies, permission inheritance, and audit logging. Enforced via middleware on all API routes.

**CRDT (yrs):** Real-time collaborative editing uses Yrs (Rust port of Yjs) for CRDT-based conflict resolution. The WebSocket handler manages document rooms, broadcasts edits, and handles client connect/disconnect.

**Plugin Runtime (Wasmtime/WASI):** `tachyon-plugin-runtime` provides a sandboxed WASM plugin system using Wasmtime. Plugins run in WASI sandboxes and are invoked via hooks from the server.

**Multi-tenancy:** Organization/team/space hierarchy with member management, per-space isolation, and resource-level access control.

**Communication flow:**

```
Browser (WASM) --HTTP/JSON--> Axum Server --SQLx--> PostgreSQL
Browser (WASM) --WebSocket--> Axum Server
                      |
                   Redis (rate limiting, caching)
                   Tantivy (search index — not yet wired to API)
```

### Middleware stack

Requests pass through 7 middleware layers: `request_id`, `security_headers`, `cors`, `rate_limit`, `cache_control`, `auth`, `audit`.

---

## Common Tasks

### Adding a new API route

1. Create or edit a handler file in `crates/server/src/routes/`
2. Register the route in the router in `crates/server/src/routes/mod.rs`
3. Add any new database queries in `crates/database/src/` (repository pattern)
4. Add corresponding frontend API methods in `crates/frontend/src/api/`
5. Write unit tests in the route module and integration tests in `tests/integration/`

### Adding a new workspace crate

1. Create the crate: `cargo new --lib crates/my-crate`
2. Add it to `members` in the root `tachyon/Cargo.toml`
3. If other crates depend on it, add `tachyon-my-crate = { path = "../my-crate" }` to their `Cargo.toml`
4. Run `cargo check --workspace` to verify

### Adding a database migration

1. Create a new SQL file in `crates/database/migrations/` with a timestamped name:

   ```
   crates/database/migrations/YYYYMMDDHHMMSS_description.sql
   ```

2. Write the `CREATE TABLE`, `ALTER TABLE`, or `CREATE INDEX` statements
3. The migration is picked up automatically by the migration runner at startup
4. Add corresponding repository methods in the appropriate module under `crates/database/src/`

### Adding a test

**Unit test** — add inside the source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_feature() {
        // ...
    }
}
```

**Integration test** — add to `crates/server/tests/integration/` (for server tests) or `crates/testing/src/integration/` (for cross-crate integration tests). Integration tests that hit the database require PostgreSQL to be running.

**Shared test utilities** — use the `tachyon-testing` crate for `TestApp`, database helpers, and mock factories.
