# Tachyon

**Version:** 0.2.0-beta
**Classification:** Universal Documentation Engine / Internal Developer Portal
**Architecture:** Cross-Platform (Rust/Tokio)

## 1. Abstract

Tachyon is a deterministic, high-performance knowledge management platform designed for hybrid operational environments. It functions simultaneously as a local-first desktop application for individual knowledge capture and a centralized Just-In-Time (JIT) documentation server for enterprise deployment.

Architected in Rust on the Tokio asynchronous runtime, Tachyon eliminates the "build step" latency inherent in traditional Static Site Generators (SSG). It operates directly upon the host file system or Git repository as its single source of truth, utilizing a reactive file-watching architecture to provide instantaneous rendering synchronization with external editors (e.g., VS Code, Neovim, JetBrains).

## 2. System Architecture

The Tachyon operational pipeline is structured into four logic layers:

1. **Runtime Layer:** Utilizes `tokio` for cross-platform asynchronous I/O (IOCP on Windows, Kqueue on macOS, Epoll/io_uring on Linux), ensuring native performance parity across operating systems.
2. **Reactive Layer:** Implements kernel-level file system monitoring via the `notify` crate. Modifications to the repository by external processes trigger immediate cache invalidation and WebSocket-driven UI updates.
3. **Processing Layer:**
   - **Data Access:** Direct interface with the `.git` object database via `git2-rs`.
   - **JIT Rendering:** SIMD-accelerated parsing (`pulldown-cmark`) and template compilation (`minijinja`).
   - **Mathematics:** Server-side LaTeX rendering via `katex-rs` to eliminate client-side layout shifts.
4. **Presentation Layer:**
   - **Desktop:** Encapsulated via `tauri` using the native OS WebView (WebView2/WebKit).
   - **Server:** Headless `axum` HTTP/2 server for network distribution.

## 3. Technical Specifications

| Metric                 | Specification                             |
| :--------------------- | :---------------------------------------- |
| **Language**           | Rust (2024 Edition)                       |
| **Concurrency Model**  | Multi-threaded Work-stealing (Tokio)      |
| **GUI Framework**      | Tauri (Native WebView)                    |
| **Hot-Reload Latency** | < 15ms (File Save to Render)              |
| **Storage Engine**     | Local File System / Git Repository        |
| **Transport Protocol** | HTTP/2 (Server Mode) / IPC (Desktop Mode) |

## 4. Prerequisites

- **Windows:** Windows 10 (Build 1903+) or Windows 11.
- **macOS:** macOS 11 (Big Sur) or later (Intel/Apple Silicon).
- **Linux:** Kernel ≥ 5.4 (GTK3 required for Desktop Mode).

## 5. Installation

### 5.1. Desktop Application (End User)

Execute the platform-specific installer. This installs the Tachyon binary and registers the URL protocol handler.

- **Windows:** `tachyon_setup_x64.exe`
- **macOS:** `Tachyon.dmg`
- **Linux:** `tachyon_amd64.deb` / `tachyon-x86_64.AppImage`

### 5.2. Server Daemon (SysAdmin)

For centralized hosting, compile from source or utilize the Docker container.

```bash
cargo build --release --no-default-features --features "server-mode"
# or
docker pull tachyon-org/tachyon-server:latest
```

## 6. Operation Modes

Tachyon provides three mutually exclusive execution modes derived from a single binary.

### 6.1. Desktop Mode (GUI)

The default execution mode. Launches a graphical interface wrapping the JIT engine.

- **Behavior:** Starts a local loopback server on a randomized port.
- **Usage:** Personal knowledge base, drafting, and local review.
- **Sync:** Commits to the local Git repository; "Publish" executes `git push`.

### 6.2. Server Mode (Daemon)

Headless execution for hosting internal documentation portals.

```bash
tachyon serve --port 8080 --config ./tachyon.toml
```

- **Behavior:** Binds to 0.0.0.0. Enforces Authentication and RBAC.
- **Usage:** Enterprise intranet, team documentation.

### 6.3. Static Export Mode (CI/CD)

Generates standard HTML artifacts for generic HTTP hosting.

```bash
tachyon build --output ./dist
```

- **Behavior:** Traverses the repository and serializes all JIT views to disk.
- **Usage:** GitHub Pages, Cloudflare Pages, Netlify.

## 7. External Editor Integration

Tachyon supports a "Bring Your Own Editor" (BYOE) workflow. It does not enforce exclusive file locks.

1. **Workflow:** Open the documentation repository in an IDE (VS Code, IntelliJ) and Tachyon simultaneously.
2. **Synchronization:**
   - **IDE -> Tachyon:** Saving a file in the IDE triggers the kernel watcher. Tachyon invalidates the cache and pushes a WebSocket refresh to the view.
   - **Tachyon -> IDE:** Editing within Tachyon writes to the file system. The IDE detects the change and updates its buffer.
3. **Conflict Resolution:** Relies on standard OS file locking and Git merge strategies.

## 8. Configuration

Configuration is managed via `tachyon.toml` in the repository root.

```toml
[system]
mode = "hybrid"          # desktop | server | static
watch_interval_ms = 100  # File system polling rate

[server]
auth_provider = "kanidm" # Required for Server Mode
enable_sso = true

[rendering]
math_engine = "katex"    # katex | mathjax (client-side)
syntax_theme = "axiom-dark"
enable_diagrams = true   # Mermaid.js support

[security]
# Define file patterns to exclude from serving
exclude = [".env", "*.secret.md", "private/"]
```

## 9. Access Control Implementation

Tachyon implements Role-Based Access Control (RBAC) at the parsing level. This is active in **Server Mode** only.

### 9.1. Frontmatter Directives

```yaml
---
title: Deployment Protocols
access: restricted
groups: [devops, sysadmin]
---
```

Requests for this document by unprivileged users result in a `404 Not Found` (Security through obscurity) or `403 Forbidden` response, configurable via TOML.

### 9.2. Block Redaction

```markdown
::: internal
**SENSITIVE:** API Keys for Production...
:::
```

The parser excises these blocks from the Abstract Syntax Tree (AST) before HTML generation if the requesting session lacks the required clearance.

## 10. Standards Compliance

- **ISO/IEC 25010:** Adheres to software quality models for performance efficiency and compatibility.
- **IEEE 829:** Documentation generation supports standard test documentation formats.
- **Data Sovereignty:** Tachyon performs no telemetry. All data processing occurs on the host hardware.

## 11. Licensing

Copyright © 2026. Licensed under the Apache License, Version 2.0. See `LICENSE` for terms.
