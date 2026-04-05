# TACHYON: DEVELOPER GUIDE

**Document ID:** TACHYON-DEV-001-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Developer Documentation
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Development Framework](#2-development-framework)
3. [Development Environment](#3-development-environment)
4. [Project Structure](#4-project-structure)
5. [Development Workflow](#5-development-workflow)
6. [Desktop Development](#6-desktop-development)
7. [Server Development](#7-server-development)
8. [Web Development](#8-web-development)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides comprehensive guidance for developers contributing to the Tachyon toolchain project. It establishes the development framework, environment setup procedures, project structure, workflow processes, and component-specific development guidelines. The guide serves as the authoritative reference for all development activities and ensures consistency across the development team.

### 1.2. System Overview

Tachyon is a deterministic, high-performance Knowledge Management System (KMS) and Internal Developer Portal (IDP) operating as a hybrid system supporting both local-first desktop usage and centralized server deployment. The system eliminates traditional build step latency through Just-In-Time (JIT) rendering architecture, operating directly upon Git repositories or file systems without preliminary compilation.

The Tachyon toolchain encompasses three primary components:

1. **Desktop Application:** A native desktop application built with Tauri framework, providing local-first knowledge management with optional synchronization to remote Git repositories.
2. **Server Application:** An HTTP/2 server built with Axum framework, providing centralized hosting for documentation repositories with multi-user support, authentication, authorization, and real-time collaboration features.
3. **Web Frontend:** A reactive web application built with Leptos framework, providing user interface for both desktop and server deployment modes with Server-Side Rendering (SSR) and client-side hydration.

### 1.3. Technology Stack

The Tachyon system employs a carefully selected technology stack optimized for performance, safety, and developer experience:

| Component | Technology | Version | Purpose |
|-----------|------------|----------|---------|
| **Primary Language** | Rust | Edition 2024 | Core logic, server components, desktop backend |
| **Async Runtime** | Tokio | 1.x | Asynchronous I/O and concurrent processing |
| **Desktop Framework** | Tauri | 2.x | Native desktop application wrapper |
| **Server Framework** | Axum | 0.7.x | HTTP/2 server and WebSocket support |
| **Web Framework** | Leptos | 0.8.15 | Reactive web frontend with SSR |
| **JavaScript Runtime** | Bun | 7.3.1 | JavaScript execution and build tooling |
| **Markdown Parser** | pulldown-cmark | 0.9.x | CommonMark-compliant parsing with SIMD |
| **Search Engine** | Tantivy | 0.21.x | Full-text search indexing |
| **Git Integration** | git2-rs | 0.18.x | Git repository operations |
| **Database** | SQLite | 0.29.x | Metadata storage and persistence |

### 1.4. Document Scope

This document covers:

- Development environment setup and configuration
- Project structure and module organization
- Development workflow and procedures
- Desktop application development guidelines
- Server application development guidelines
- Web frontend development guidelines
- Testing and quality assurance procedures
- Contribution guidelines and code review processes

Out of scope:

- Detailed API specifications (covered in separate API documentation)
- User-facing documentation (covered in user guides)
- Deployment procedures (covered in operations documentation)

---

## 2. DEVELOPMENT FRAMEWORK

### 2.1. Architectural Principles

The Tachyon system is architected according to the following principles:

#### 2.1.1. Local-First Design

The system prioritizes local-first design, ensuring full functionality without network connectivity in desktop mode. All core operations, including content rendering, search, and Git operations, execute locally on the host hardware. This approach ensures data sovereignty, offline capability, and minimal latency.

**Implementation Requirements:**

- Desktop mode operates without network connectivity
- All content processing occurs locally
- Git operations use local repository with optional remote sync
- No telemetry or data transmission without explicit user consent

#### 2.1.2. Microsecond Latency

The system is architected for microsecond-level latency in rendering and search operations. JIT rendering processes Markdown content to HTML within 15 milliseconds of file modification, enabling real-time updates without perceptible delay.

**Performance Targets:**

| Operation | Target Latency | Measurement Method |
|-----------|-----------------|-------------------|
| JIT Rendering | < 15 ms | File modification to HTML rendering |
| Search Query | < 100 ms | Query submission to result display |
| File Watch | < 100 ms | File system change to cache invalidation |
| WebSocket Update | < 50 ms | Event occurrence to client delivery |

#### 2.1.3. Type Safety

The system leverages Rust's type system for compile-time guarantees of memory safety and thread safety. The ownership system prevents entire classes of memory corruption vulnerabilities including buffer overflows, use-after-free, double-free, and null pointer dereferences.

**Memory Safety Guarantees:**

| Vulnerability | Prevention Mechanism |
|---------------|---------------------|
| Buffer Overflow | Compile-time bounds checking |
| Use-After-Free | Compile-time lifetime tracking |
| Double-Free | Compile-time ownership tracking |
| Null Pointer Dereference | Compile-time null checking |
| Data Races | Compile-time race prevention |
| Memory Leaks | Compile-time RAII |

#### 2.1.4. Asynchronous Processing

The system uses Tokio's asynchronous runtime for non-blocking I/O operations. Async/await syntax with the `Future` trait enables efficient concurrent processing without the complexity of manual thread management.

**Concurrency Model:**

- Async/await syntax for asynchronous operations
- Work-stealing scheduler for multi-threaded execution
- Thread-safe primitives: `Arc<T>`, `Mutex<T>`, `RwLock<T>`
- Multi-producer multi-consumer (MPMC) channels for message passing

#### 2.1.5. Modular Design

The system is architected with clear module boundaries and minimal coupling between components. Each component has well-defined responsibilities and interfaces, enabling independent development, testing, and deployment.

**Module Boundaries:**

- Core Engine: JIT rendering, caching, content processing
- Desktop Component: Tauri application, WebView integration
- Server Component: HTTP/2 serving, WebSocket management
- Web Frontend: Reactive UI, client-side state management
- IPC Component: Inter-process communication between desktop and server

### 2.2. Development Standards

All development activities must adhere to the standards defined in [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md).

#### 2.2.1. Coding Standards

**File Naming Conventions:**

- Rust files: `snake_case.rs`
- TypeScript files: `kebab-case.ts`
- Markdown files: `snake_case.md`
- All files: lowercase with underscores

**Code Style Standards:**

- Rust: `rustfmt` with default configuration
- TypeScript: Prettier with 2-space indentation
- Markdown: CommonMark with 80-character line width

**Documentation Standards:**

- All public functions must have documentation comments
- Rust: `///` doc comments with `# Arguments`, `# Returns`, `# Errors` sections
- TypeScript: JSDoc with `@param`, `@returns`, `@throws` tags
- All documentation must be precise, unambiguous, and verifiable

#### 2.2.2. Quality Standards

**Type Safety Requirements:**

- Use the most restrictive type definitions available
- Explicitly reject implicit type coercion or universal types
- Handle all edge cases explicitly (empty sets, null values, boundary limits)
- All public interfaces must validate input arguments

**Error Handling Requirements:**

- Use explicit error handling with `Result<T, E>` in Rust
- Define custom error types with `thiserror` crate
- Provide clear, actionable error messages
- Log errors securely without exposing sensitive information

**Testing Requirements:**

- Unit tests for all functions with non-trivial logic
- Integration tests for component interactions
- Documentation tests for examples in public APIs
- Minimum 80% code coverage for critical paths

### 2.3. Security Framework

All development activities must adhere to the security architecture defined in [ADR-010: Security Architecture](../../.specs/02_adrs/010_security_architecture.md).

#### 2.3.1. Defense-in-Depth Strategy

The system implements multiple layers of security controls:

1. **Application Layer:** Input validation, output encoding, business logic
2. **Framework Layer:** Memory safety, type safety, IPC security
3. **Communication Layer:** TLS 1.3, authentication, authorization
4. **Data Layer:** Encryption at rest, access controls, audit logging
5. **Infrastructure Layer:** Supply chain security, build security, deployment security

#### 2.3.2. Security Requirements

**Memory Safety:**

- Rust's ownership system prevents memory corruption at compile time
- No use of `unsafe` code unless absolutely necessary
- All `unsafe` blocks must be documented and reviewed

**Input Validation:**

- Validate all user inputs against defined schemas
- Sanitize content to prevent injection attacks
- Use type-safe validation libraries (e.g., `validator` crate)

**Encryption:**

- TLS 1.3 for all network communications
- AES-256 encryption for data at rest
- Secure session tokens (JWT) with proper expiration

**Audit Logging:**

- Log all security-relevant events with timestamps
- Include user identities and action details
- Use structured logging with `tracing` crate

**Fail-Safe Error Handling:**

- Errors must not expose sensitive information
- Default error handling must be secure
- System must fail safely on errors

### 2.4. Compliance Framework

The system must comply with the following standards and regulations:

| Standard | Compliance Level | Implementation |
|----------|------------------|----------------|
| ISO/IEC 26514:2021 | Full | Documentation lifecycle and quality assurance |
| ISO/IEC 12207:2017 | Full | Software lifecycle processes |
| ISO/IEC 25010:2011 | Full | Software quality characteristics |
| IEEE 829-2008 | Full | Test documentation |
| IEEE 1063-2001 | Full | User documentation |
| WCAG 2.1 AA | Full | Web accessibility |
| GDPR | Full | Data protection and privacy |
| ISO 27001 | Partial | Information security management |
| SOC 2 Type II | Partial | Security, availability, processing integrity |

---

## 3. DEVELOPMENT ENVIRONMENT

### 3.1. Prerequisites

#### 3.1.1. System Requirements

**Minimum Hardware Requirements:**

| Component | Minimum | Recommended |
|-----------|-----------|--------------|
| **CPU** | 4 cores (x86_64 or ARM64) | 8 cores (x86_64 or ARM64) |
| **RAM** | 8 GB | 16 GB |
| **Disk Space** | 2 GB | 5 GB |
| **Network** | Not required for development | High-speed for Git operations |

**Supported Operating Systems:**

| Platform | Version | Status | Notes |
|----------|---------|--------|-------|
| **Windows** | 10+ | Tier 1 | Native support with MSVC toolchain |
| **Windows** | 11+ | Tier 1 | Native support with MSVC toolchain |
| **macOS** | 11+ (x86_64) | Tier 1 | Native support with Xcode toolchain |
| **macOS** | 11+ (Apple Silicon) | Tier 1 | Native support with Xcode toolchain |
| **Linux** | Kernel 5.4+ | Tier 1 | Native support with GCC/Clang toolchain |

#### 3.1.2. Software Prerequisites

**Required Software:**

| Software | Minimum Version | Recommended Version | Purpose |
|----------|-----------------|---------------------|---------|
| **Rust** | 1.77.2 | 1.80+ | Primary language and build tool |
| **Bun** | 1.0.0 | Latest stable | JavaScript runtime and build tool |
| **Git** | 2.30.0 | Latest stable | Version control |
| **Node.js** | Not required | Not required | Replaced by Bun |
| **Nix** | 2.18+ | Latest stable | Reproducible builds (optional) |
| **VS Code** | 1.80+ | Latest stable | Recommended IDE |

**Optional Software:**

| Software | Purpose |
|----------|---------|
| **rust-analyzer** | Rust language server for VS Code |
| **Tauri CLI** | Tauri development tools |
| **Leptos CLI** | Leptos development tools |
| **cargo-watch** | Auto-rebuild on file changes |
| **sccache** | Distributed compilation caching |

### 3.2. Installation Procedures

#### 3.2.1. Rust Installation

**Installation on Linux/macOS:**

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version

# Add required components
rustup component add rust-src rustfmt clippy

# Set default toolchain (Rust Edition 2024)
rustup default stable
```

**Installation on Windows:**

```powershell
# Install Rust using rustup
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe

# Verify installation
rustc --version
cargo --version

# Add required components
rustup component add rust-src rustfmt clippy

# Set default toolchain
rustup default stable
```

**Configuration:**

```bash
# Configure Cargo for optimal development
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[build]
jobs = 4  # Use 4 parallel jobs for compilation

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native"]  # Optimize for host CPU

[profile.dev]
opt-level = 0  # Faster compilation for development

[profile.release]
opt-level = 3  # Maximum optimization for release builds
lto = "thin"  # Link-time optimization for smaller binaries
codegen-units = 1  # Better optimization at cost of compilation time
strip = true  # Remove debug symbols from binaries
EOF
```

#### 3.2.2. Bun Installation

**Installation on Linux/macOS:**

```bash
# Install Bun using curl
curl -fsSL https://bun.sh/install | bash

# Verify installation
bun --version

# Enable Bun completions (optional)
echo 'eval "$(bun completions)"' >> ~/.bashrc
```

**Installation on Windows:**

```powershell
# Install Bun using PowerShell
powershell -c "irm bun.sh/install.ps1|iex"

# Verify installation
bun --version
```

**Configuration:**

```bash
# Configure Bun for optimal development
cat > ~/.bunfig.toml << 'EOF'
[install]
# Use global cache for faster installs
cache = true

[build]
# Optimize builds for production
target = "bun"
minify = true
sourcemap = false
EOF
```

#### 3.2.3. Nix Installation (Optional)

**Installation:**

```bash
# Install Nix using single-user install
curl -L https://nixos.org/nix/install | sh

# Enable Nix command
. ~/.nix-profile/etc/profile.d/nix.sh

# Verify installation
nix --version
```

**Configuration:**

```bash
# Enable flakes and nix-command
mkdir -p ~/.config/nix
cat > ~/.config/nix/nix.conf << 'EOF'
experimental-features = nix-command flakes
EOF
```

### 3.3. Project Setup

#### 3.3.1. Repository Cloning

```bash
# Clone the Tachyon repository
git clone https://github.com/your-org/tachyon.git
cd tachyon

# Verify repository structure
ls -la
```

#### 3.3.2. Development Environment Activation

**Using Nix (Recommended):**

```bash
# Enter development shell
nix develop

# Verify environment
which cargo
which bun
```

---

## 6. DESKTOP DEVELOPMENT

### 6.1. Desktop Application Architecture

#### 6.1.1. Tauri Framework

The desktop application uses Tauri framework to wrap the core rendering engine within a native desktop application. Tauri provides WebView integration, native OS access, and IPC communication capabilities.

**Tauri Configuration ([`tachyon/crates/desktop/src-tauri/tauri.conf.json`](../tachyon/crates/desktop/src-tauri/tauri.conf.json)):**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Tachyon",
  "version": "0.1.0",
  "identifier": "com.tachyon.app",
  "build": {
    "beforeDevCommand": "cargo run --bin server",
    "beforeBuildCommand": "cargo build --release --bin server",
    "devUrl": "../dist",
    "frontendDist": "../dist",
    "withGlobalTauri": true
  },
  "tauri": {
    "allowlist": [
      {
        "identifier": "fs:read-file",
        "allow": [{ "path": "$HOME/Documents" }]
      },
      {
        "identifier": "fs:write-file",
        "allow": [{ "path": "$HOME/Documents" }]
      },
      {
        "identifier": "fs:read-dir",
        "allow": [{ "path": "$HOME/Documents" }]
      },
      {
        "identifier": "window:allow-create",
        "allow": [{ "title": true }]
      },
      {
        "identifier": "window:allow-close",
        "allow": [{ "label": "main" }]
      },
      {
        "identifier": "dialog:allow-open",
        "allow": [{ "multiple": false }]
      },
      {
        "identifier": "dialog:allow-save",
        "allow": [{ "multiple": false }]
      },
      {
        "identifier": "notification:allow-send",
        "allow": [{ "title": true }]
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"],
    "publisher": "Tachyon Team"
  },
  "plugins": {}
}
```

#### 6.1.2. Desktop Entry Point

The [`tachyon/crates/desktop/src-tauri/src/main.rs`](../tachyon/crates/desktop/src-tauri/src/main.rs) file is the entry point for the desktop application.

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Create Tauri application
    tauri::Builder::default()
        .setup(|app| {
            // Setup application state
            let state = AppState::new();
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri_commands::handle_command)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Application state shared across Tauri commands
pub struct AppState {
    /// Document cache
    documents: Arc<RwLock<HashMap<String, String>>>,
    /// Git repository state
    git_state: Arc<Mutex<GitState>>,
    /// Search index state
    search_index: Arc<Mutex<SearchIndex>>,
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            git_state: Arc::new(Mutex::new(GitState::new())),
            search_index: Arc::new(Mutex::new(SearchIndex::new())),
        }
    }
}
```

### 6.2. IPC Communication

#### 6.2.1. Tauri Commands

Tauri commands provide IPC communication between the WebView frontend and Rust backend.

**Command Handler ([`tachyon/crates/desktop/src-tauri/src/lib.rs`](../tachyon/crates/desktop/src-tauri/src/lib.rs)):**

```rust
use tauri::{command, State, Window};
use crate::AppState;

/// Handle Tauri IPC commands
#[command]
pub async fn handle_command(
    command: String,
    payload: String,
    state: State<'_, AppState>,
    window: Window,
) -> Result<String, String> {
    tracing::info!(command = %command, "Received command");

    match command.as_str() {
        "get_document" => get_document(&payload, state).await,
        "save_document" => save_document(&payload, state).await,
        "search_documents" => search_documents(&payload, state).await,
        "git_status" => git_status(state).await,
        "git_commit" => git_commit(&payload, state).await,
        _ => Err(format!("Unknown command: {}", command)),
    }
}

/// Get document content
///
/// # Arguments
///
/// * `payload` - Document ID as JSON string
///
/// # Returns
///
/// Document content as JSON string
///
/// # Errors
///
/// Returns error if document not found or read fails
#[command]
async fn get_document(
    payload: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let id: String = serde_json::from_str(&payload)
        .map_err(|e| format!("Invalid payload: {}", e))?;

    let documents = state.documents.read().await;
    match documents.get(&id) {
        Some(content) => Ok(content.clone()),
        None => Err(format!("Document not found: {}", id)),
    }
}

/// Save document content
///
/// # Arguments
///
/// * `payload` - Document ID and content as JSON string
///
/// # Returns
///
/// Unit
///
/// # Errors
///
/// Returns error if save fails
#[command]
async fn save_document(
    payload: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let data: SaveRequest = serde_json::from_str(&payload)
        .map_err(|e| format!("Invalid payload: {}", e))?;

    let mut documents = state.documents.write().await;
    documents.insert(data.id.clone(), data.content);

    tracing::info!(id = %data.id, "Document saved");
    Ok(())
}
```

#### 6.2.2. Frontend IPC Client

The web frontend communicates with the desktop backend using Tauri IPC API.

**IPC Client ([`tachyon/web/src/api/ipc.ts`](../tachyon/web/src/api/ipc.ts)):**

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { IpcMessage } from './types';

/// IPC client for desktop communication
export class IpcClient {
  /// Get document content
  async getDocument(id: string): Promise<string> {
    const response = await invoke<string>('get_document', JSON.stringify({ id }));
    return JSON.parse(response);
  }

  /// Save document content
  async saveDocument(id: string, content: string): Promise<void> {
    const payload = JSON.stringify({ id, content });
    await invoke('save_document', payload);
  }

  /// Search documents
  async searchDocuments(query: string): Promise<SearchResult[]> {
    const response = await invoke<string>('search_documents', JSON.stringify({ query }));
    return JSON.parse(response);
  }

  /// Get Git status
  async getGitStatus(): Promise<GitStatus> {
    const response = await invoke<string>('git_status', '{}');
    return JSON.parse(response);
  }

  /// Commit changes
  async gitCommit(message: string): Promise<void> {
    const payload = JSON.stringify({ message });
    await invoke('git_commit', payload);
  }
}

/// Export singleton instance
export const ipc = new IpcClient();
```

### 6.3. Native OS Integration

#### 6.3.1. File System Access

Tauri provides capability-based access to the file system, ensuring security and user privacy.

**File System Operations ([`tachyon/crates/desktop/src-tauri/src/commands/files.rs`](../tachyon/crates/desktop/src-tauri/src/commands/files.rs)):**

```rust
use std::path::Path;
use tauri::command;

/// Open file dialog
///
/// # Arguments
///
/// * `filters` - File type filters
///
/// # Returns
///
/// Selected file path
///
/// # Errors
///
/// Returns error if dialog fails or user cancels
#[command]
async fn open_file_dialog(filters: Vec<FileFilter>) -> Result<String, String> {
    use tauri::dialog::FileDialogBuilder;

    let mut builder = FileDialogBuilder::new();
    for filter in filters {
        builder = builder.add_filter(&filter.name, &filter.extensions);
    }

    let path = builder.pick_file()
        .map_err(|e| format!("Failed to open file dialog: {}", e))?;

    path.ok_or_else(|| "No file selected".to_string())
}

/// Save file dialog
///
/// # Arguments
///
/// * `default_name` - Default file name
/// * `filters` - File type filters
///
/// # Returns
///
/// Selected file path
///
/// # Errors
///
/// Returns error if dialog fails or user cancels
#[command]
async fn save_file_dialog(
    default_name: String,
    filters: Vec<FileFilter>,
) -> Result<String, String> {
    use tauri::dialog::FileDialogBuilder;

    let mut builder = FileDialogBuilder::new();
    builder = builder.set_file_name(&default_name);
    for filter in filters {
        builder = builder.add_filter(&filter.name, &filter.extensions);
    }

    let path = builder.save_file()
        .map_err(|e| format!("Failed to save file dialog: {}", e))?;

    path.ok_or_else(|| "No file selected".to_string())
}

/// File filter for dialog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    /// Filter name
    pub name: String,
    /// File extensions
    pub extensions: Vec<String>,
}
```

#### 6.3.2. System Notifications

Tauri provides native OS notifications for important events.

**Notification Operations ([`tachyon/crates/desktop/src-tauri/src/commands/notifications.rs`](../tachyon/crates/desktop/src-tauri/src/commands/notifications.rs)):**

```rust
use tauri::command;

/// Send system notification
///
/// # Arguments
///
/// * `title` - Notification title
/// * `body` - Notification body
///
/// # Returns
///
/// Unit
///
/// # Errors
///
/// Returns error if notification fails
#[command]
async fn send_notification(title: String, body: String) -> Result<(), String> {
    use tauri::api::notification;

    notification::Notification::new(&title)
        .body(&body)
        .show()
        .map_err(|e| format!("Failed to send notification: {}", e))?;

    Ok(())
}
```

### 6.4. Desktop-Specific Features

#### 6.4.1. Local Server Management

The desktop application spawns a local Axum server on a randomized loopback port.

**Server Spawning ([`tachyon/crates/desktop/src-tauri/src/server.rs`](../tachyon/crates/desktop/src-tauri/src/server.rs)):**

```rust
use std::process::Child;
use tokio::net::TcpListener;

/// Local server process
pub struct LocalServer {
    /// Server process
    process: Option<Child>,
    /// Server port
    port: u16,
}

impl LocalServer {
    /// Create new local server
    pub fn new() -> Self {
        Self {
            process: None,
            port: 0,
        }
    }

    /// Start local server
    ///
    /// # Errors
    ///
    /// Returns error if server fails to start
    pub async fn start(&mut self) -> Result<(), String> {
        // Find available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind to loopback: {}", e))?;
        self.port = listener.local_addr().port();

        tracing::info!(port = self.port, "Starting local server");

        // Spawn server process
        self.process = Some(
            Command::new("cargo")
                .args(["run", "--bin", "server", "--port", &self.port.to_string()])
                .spawn()
                .map_err(|e| format!("Failed to spawn server: {}", e))?,
        );

        Ok(())
    }

    /// Stop local server
    ///
    /// # Errors
    ///
    /// Returns error if server fails to stop
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut process) = self.process.take() {
            process
                .kill()
                .map_err(|e| format!("Failed to stop server: {}", e))?;
        }
        Ok(())
    }

    /// Get server URL
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}
```

#### 6.4.2. Session Persistence

The desktop application persists session state across restarts.

**Session Management ([`tachyon/crates/desktop/src-tauri/src/session.rs`](../tachyon/crates/desktop/src-tauri/src/session.rs)):**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Open documents
    pub open_documents: Vec<String>,
    /// Current document
    pub current_document: Option<String>,
    /// Scroll positions
    pub scroll_positions: HashMap<String, f64>,
    /// Window state
    pub window_state: WindowState,
}

/// Window state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Window maximized
    pub maximized: bool,
    /// Sidebar state
    pub sidebar_open: bool,
}

/// Session manager
pub struct SessionManager {
    /// Session file path
    session_path: PathBuf,
}

impl SessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        let session_path = dirs::cache_local()
            .unwrap()
            .join("tachyon")
            .join("session.json");

        Self { session_path }
    }

    /// Load session
    ///
    /// # Errors
    ///
    /// Returns error if session file cannot be read
    pub fn load(&self) -> Result<Session, String> {
        if !self.session_path.exists() {
            return Ok(Session::default());
        }

        let content = fs::read_to_string(&self.session_path)
            .map_err(|e| format!("Failed to read session file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse session: {}", e))
    }

    /// Save session
    ///
    /// # Errors
    ///
    /// Returns error if session file cannot be written
    pub fn save(&self, session: &Session) -> Result<(), String> {
        let content = serde_json::to_string_pretty(session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        fs::write(&self.session_path, content)
            .map_err(|e| format!("Failed to write session file: {}", e))?;

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            open_documents: Vec::new(),
            current_document: None,
            scroll_positions: HashMap::new(),
            window_state: WindowState {
                width: 1280,
                height: 720,
                maximized: false,
                sidebar_open: true,
            },
        }
    }
}
```

---

## 7. SERVER DEVELOPMENT

### 7.1. Server Application Architecture

#### 7.1.1. Axum Framework

The server application uses Axum framework to provide HTTP/2 server with WebSocket support for real-time communication.

**Server Entry Point ([`tachyon/crates/server/src/main.rs`](../tachyon/crates/server/src/main.rs)):**

```rust
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
    Router,
};
use tokio::net::TcpListener;
use tracing::info;

mod api;
mod core;
mod db;
mod git;
mod websocket;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create router
    let app = Router::new()
        .route("/", get(root))
        .route("/api", api::router())
        .route("/ws", websocket::handler())
        .route("/health", get(health_check));

    // Bind to address
    let addr = std::env::var("SERVER_ADDRESS")
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|| "3000".to_string())
        .parse()
        .expect("Invalid port");

    let listener = TcpListener::bind(format!("{}:{}", addr, port))
        .await
        .expect("Failed to bind to address");

    info!(addr = %addr, port = port, "Server listening");

    axum::serve(listener, app).await.unwrap();
}

/// Root handler
async fn root() -> &'static str {
    "Tachyon Server API"
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK").into_response()
}
```

### 7.2. API Endpoints

#### 7.2.1. Document Endpoints

Document endpoints provide CRUD operations for document management.

**Document API ([`tachyon/crates/server/src/api/documents.rs`](../tachyon/crates/server/src/api/documents.rs)):**

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put, delete},
    Json,
};
use serde::{Deserialize, Serialize};
use crate::AppState;

/// Get all documents
///
/// # Arguments
///
/// * `state` - Application state
/// * `query` - Query parameters (limit, offset)
///
/// # Returns
///
/// JSON array of documents
///
/// # Errors
///
/// Returns error if retrieval fails
#[get("/api/documents")]
async fn get_documents(
    State(state): State<AppState>,
    Query(query): Query<GetDocumentsQuery>,
) -> Result<Json<Vec<Document>>, StatusCode> {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    let documents = state
        .storage
        .list_documents(limit, offset)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get documents");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(documents))
}

/// Get document by ID
///
/// # Arguments
///
/// * `state` - Application state
/// * `path` - Document ID path parameter
///
/// # Returns
///
/// Document as JSON
///
/// # Errors
///
/// Returns 404 if document not found, 500 if retrieval fails
#[get("/api/documents/:id")]
async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Document>, StatusCode> {
    let document = state
        .storage
        .get_document(&id)
        .await
        .map_err(|e| {
            tracing::error!(id = %id, error = %e, "Failed to get document");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match document {
        Some(doc) => Ok(Json(doc)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create new document
///
/// # Arguments
///
/// * `state` - Application state
/// * `payload` - Document creation payload
///
/// # Returns
///
/// Created document as JSON
///
/// # Errors
///
/// Returns 400 if validation fails, 500 if creation fails
#[post("/api/documents")]
async fn create_document(
    State(state): State<AppState>,
    Json(payload): Json<CreateDocumentPayload>,
) -> Result<Json<Document>, StatusCode> {
    // Validate payload
    if payload.title.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let document = state
        .storage
        .create_document(&payload.title, &payload.content)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create document");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(document))
}

/// Update document
///
/// # Arguments
///
/// * `state` - Application state
/// * `path` - Document ID path parameter
/// * `payload` - Document update payload
///
/// # Returns
///
/// Updated document as JSON
///
/// # Errors
///
/// Returns 404 if document not found, 400 if validation fails, 500 if update fails
#[put("/api/documents/:id")]
async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDocumentPayload>,
) -> Result<Json<Document>, StatusCode> {
    let document = state
        .storage
        .update_document(&id, &payload.title, &payload.content)
        .await
        .map_err(|e| {
            tracing::error!(id = %id, error = %e, "Failed to update document");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match document {
        Some(doc) => Ok(Json(doc)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete document
///
/// # Arguments
///
/// * `state` - Application state
/// * `path` - Document ID path parameter
///
/// # Returns
///
/// Unit
///
/// # Errors
///
/// Returns 404 if document not found, 500 if deletion fails
#[delete("/api/documents/:id")]
async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .storage
        .delete_document(&id)
        .await
        .map_err(|e| {
            tracing::error!(id = %id, error = %e, "Failed to delete document");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get documents query parameters
#[derive(Debug, Deserialize)]
pub struct GetDocumentsQuery {
    /// Number of documents to return
    pub limit: Option<usize>,
    /// Number of documents to skip
    pub offset: Option<usize>,
}

/// Create document payload
#[derive(Debug, Deserialize)]
pub struct CreateDocumentPayload {
    /// Document title
    #[serde(validate(length(min = 1, max = 100))]
    pub title: String,
    /// Document content
    pub content: String,
}

/// Update document payload
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentPayload {
    /// Document title
    #[serde(validate(length(min = 1, max = 100))]
    pub title: Option<String>,
    /// Document content
    pub content: Option<String>,
}

/// Document model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Document content
    pub content: String,
    /// Creation timestamp
    pub created_at: String,
    /// Update timestamp
    pub updated_at: String,
}
```

#### 7.2.2. Search Endpoints

Search endpoints provide full-text search capabilities.

**Search API ([`tachyon/crates/server/src/api/search.rs`](../tachyon/crates/server/src/api/search.rs)):**

```rust
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::AppState;

/// Search documents
///
/// # Arguments
///
/// * `state` - Application state
/// * `query` - Search query parameters
///
/// # Returns
///
/// JSON array of search results
///
/// # Errors
///
/// Returns 400 if query is empty, 500 if search fails
#[get("/api/search")]
async fn search_documents(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    if query.q.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let results = state
        .search_index
        .search(&query.q, query.limit.unwrap_or(20))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to search documents");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(results))
}

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Number of results to return
    pub limit: Option<usize>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Match score
    pub score: f64,
    /// Highlighted snippet
    pub snippet: String,
}
```

### 7.3. WebSocket Communication

#### 7.3.1. WebSocket Handler

WebSocket handler provides real-time communication for collaborative editing.

**WebSocket Handler ([`tachyon/crates/server/src/websocket/handler.rs`](../tachyon/crates/server/src/websocket/handler.rs)):**

```rust
use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
};
use futures_util::{Sink, StreamExt};
use tracing::{error, info};
use crate::AppState;

/// WebSocket upgrade handler
///
/// # Arguments
///
/// * `state` - Application state
/// * `ws` - WebSocket upgrade
///
/// # Returns
///
/// WebSocket response
///
/// # Errors
///
/// Returns error if upgrade fails
pub async fn websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket, mut headers| {
        info!("WebSocket connection established");

        // Create WebSocket connection
        let conn = WebSocketConnection::new(state, socket);

        // Spawn connection handler task
        tokio::spawn(async move {
            if let Err(e) = conn.handle().await {
                error!(error = %e, "WebSocket connection error");
            }
        });
    })
}

/// WebSocket connection
pub struct WebSocketConnection {
    /// Application state
    state: AppState,
    /// WebSocket sender
    sender: SplitSink<axum::extract::ws::Message>,
    /// WebSocket receiver
    receiver: SplitStream<axum::extract::ws::Message>,
}

impl WebSocketConnection {
    /// Create new WebSocket connection
    pub fn new(state: AppState, socket: axum::extract::ws::WebSocket) -> Self {
        let (sender, receiver) = socket.split();

        Self {
            state,
            sender,
            receiver,
        }
    }

    /// Handle WebSocket connection
    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(message) = self.receiver.next().await {
            if let Err(e) = self.handle_message(message).await {
                error!(error = %e, "Failed to handle message");
            }
        }
        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(&mut self, message: axum::extract::ws::Message) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            axum::extract::ws::Message::Text(text) => {
                let event: WsEvent = serde_json::from_str(&text)?;
                self.handle_event(event).await?;
            }
            axum::extract::ws::Message::Close(close_frame) => {
                info!(code = close_frame.code, reason = close_frame.reason, "WebSocket closed");
                return Ok(());
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle WebSocket event
    async fn handle_event(&mut self, event: WsEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            WsEvent::ContentUpdate { id, content } => {
                self.broadcast_content_update(&id, &content).await;
            }
            WsEvent::UserPresence { user_id, document_id } => {
                self.broadcast_user_presence(&user_id, &document_id).await;
            }
            _ => {}
        }
        Ok(())
    }

    /// Broadcast content update
    async fn broadcast_content_update(&self, id: &str, content: &str) {
        let event = WsEvent::ContentUpdate {
            id: id.to_string(),
            content: content.to_string(),
        };

        let message = serde_json::to_string(&event)?;
        self.sender.send(axum::extract::ws::Message::Text(message)).await?;
        Ok(())
    }

    /// Broadcast user presence
    async fn broadcast_user_presence(&self, user_id: &str, document_id: &str) {
        let event = WsEvent::UserPresence {
            user_id: user_id.to_string(),
            document_id: document_id.to_string(),
        };

        let message = serde_json::to_string(&event)?;
        self.sender.send(axum::extract::ws::Message::Text(message)).await?;
        Ok(())
    }
}

/// WebSocket event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsEvent {
    /// Content update event
    ContentUpdate { id: String, content: String },
    /// User presence event
    UserPresence { user_id: String, document_id: String },
}
```

---

## 8. WEB DEVELOPMENT

### 8.1. Leptos Framework

#### 8.1.1. Leptos Architecture

The web frontend uses Leptos framework for reactive UI with Server-Side Rendering (SSR) and client-side hydration.

**Main Component ([`tachyon/web/src/main.rs`](../tachyon/web/src/main.rs)):**

```rust
use leptos::*;
use leptos_router::*;
use leptos_meta::*;
use leptos_axum::*;

#[component(App)]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <nav class="navbar">
                <a href="/" class="nav-item">Home</a>
                <a href="/editor" class="nav-item">Editor</a>
                <a href="/settings" class="nav-item">Settings</a>
            </nav>
            <main>
                <Routes />
            </main>
        </Router>
    }
}

#[component(Home)]
pub fn Home() -> impl IntoView {
    view! {
        <div class="home-container">
            <h1>"Welcome to Tachyon"</h1>
            <p>"Your knowledge management system"</p>
        </div>
    }
}

#[component(Editor)]
pub fn Editor() -> impl IntoView {
    view! {
        <EditorComponent />
    }
}

#[component(Settings)]
pub fn Settings() -> impl IntoView {
    view! {
        <SettingsComponent />
    }
}
```

#### 8.1.2. State Management

Leptos uses signals for reactive state management.

**State Store ([`tachyon/web/src/state/store.rs`](../tachyon/web/src/state/store.rs)):**

```rust
use leptos::*;
use serde::{Deserialize, Serialize};

/// Application state
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    /// Current document ID
    #[default]
    current_document: Option<String>,
    /// Document content
    #[default]
    document_content: Option<String>,
    /// Search query
    #[default]
    search_query: String,
    /// Search results
    #[default]
    search_results: Vec<SearchResult>,
    /// User preferences
    #[default]
    theme: Theme,
    /// Sidebar state
    #[default]
    sidebar_open: bool,
}

/// Theme preference
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Custom theme
    Custom,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

/// Search result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Match score
    pub score: f64,
}

/// Global state store
#[derive(Clone)]
pub struct Store {
    /// Application state
    state: ReadSignal<AppState>,
}

impl Store {
    /// Create new store
    pub fn new() -> Self {
        Self {
            state: create_rw_signal(AppState::default()),
        }
    }

    /// Get application state
    pub fn state(&self) -> ReadSignal<AppState> {
        self.state.read()
    }

    /// Update application state
    pub fn update_state(&self, f: impl FnOnce(&mut AppState)) {
        self.state.update(f);
    }
}

/// Action to set current document
pub fn SetCurrentDocument(document_id: String) -> impl FnOnce<&mut AppState> {
    move |state| {
        state.current_document = Some(document_id);
    }
}

/// Action to set document content
pub fn SetDocumentContent(content: String) -> impl FnOnce<&mut AppState> {
    move |state| {
        state.document_content = Some(content);
    }
}

/// Action to set search query
pub fn SetSearchQuery(query: String) -> impl FnOnce<&mut AppState> {
    move |state| {
        state.search_query = query;
    }
}

/// Action to set search results
pub fn SetSearchResults(results: Vec<SearchResult>) -> impl FnOnce<&mut AppState> {
    move |state| {
        state.search_results = results;
    }
}

/// Action to toggle theme
pub fn ToggleTheme() -> impl FnOnce<&mut AppState> {
    move |state| {
        state.theme = match state.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
            Theme::Custom => Theme::Dark,
        };
    }
}

/// Action to toggle sidebar
pub fn ToggleSidebar() -> impl FnOnce<&mut AppState> {
    move |state| {
        state.sidebar_open = !state.sidebar_open;
    }
}
```

### 8.2. API Client

#### 8.2.1. HTTP Client

The web frontend communicates with the server using HTTP/2.

**API Client ([`tachyon/web/src/api/client.ts`](../tachyon/web/src/api/client.ts)):**

```typescript
import axios from 'axios';
import type {
  Document,
  CreateDocumentPayload,
  UpdateDocumentPayload,
  SearchResult,
  GitStatus,
  CommitRequest,
};

/// API client for server communication
export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = '') {
    this.baseUrl = baseUrl;
  }

  /// Get all documents
  async getDocuments(limit: number = 100, offset: number = 0): Promise<Document[]> {
    const response = await axios.get(`${this.baseUrl}/api/documents`, {
      params: { limit, offset },
    });
    return response.data;
  }

  /// Get document by ID
  async getDocument(id: string): Promise<Document> {
    const response = await axios.get(`${this.baseUrl}/api/documents/${id}`);
    return response.data;
  }

  /// Create document
  async createDocument(payload: CreateDocumentPayload): Promise<Document> {
    const response = await axios.post(`${this.baseUrl}/api/documents`, payload);
    return response.data;
  }

  /// Update document
  async updateDocument(id: string, payload: UpdateDocumentPayload): Promise<Document> {
    const response = await axios.put(`${this.baseUrl}/api/documents/${id}`, payload);
    return response.data;
  }

  /// Delete document
  async deleteDocument(id: string): Promise<void> {
    await axios.delete(`${this.baseUrl}/api/documents/${id}`);
  }

  /// Search documents
  async searchDocuments(query: string, limit: number = 20): Promise<SearchResult[]> {
    const response = await axios.get(`${this.baseUrl}/api/search`, {
      params: { q: query, limit },
    });
    return response.data;
  }

  /// Get Git status
  async getGitStatus(): Promise<GitStatus> {
    const response = await axios.get(`${this.baseUrl}/api/git/status`);
    return response.data;
  }

  /// Commit changes
  async gitCommit(request: CommitRequest): Promise<void> {
    await axios.post(`${this.baseUrl}/api/git/commit`, request);
  }
}

/// Export singleton instance
export const api = new ApiClient();
```

#### 8.2.2. WebSocket Client

The web frontend uses WebSocket for real-time updates.

**WebSocket Client ([`tachyon/web/src/api/websocket.ts`](../tachyon/web/src/api/websocket.ts)):**

```typescript
export type { WsEvent, ContentUpdate, UserPresence };

/// WebSocket client for real-time communication
export class WebSocketClient {
  private ws: WebSocket | null;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 5;
  private reconnectDelay: number = 1000;

  /// Connect to WebSocket
  connect(url: string): void {
    this.ws = new WebSocket(url);
    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };
    this.ws.onmessage = (event: MessageEvent) => {
      this.handleMessage(event.data);
    };
    this.ws.onerror = (error: Event) => {
      console.error('WebSocket error:', error);
    };
    this.ws.onclose = () => {
      console.log('WebSocket closed');
      this.attemptReconnect();
    };
  }

  /// Disconnect from WebSocket
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /// Send message to WebSocket
  send(message: WsEvent): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  /// Handle incoming message
  private handleMessage(data: string): void {
    try {
      const event: WsEvent = JSON.parse(data);
      this.handleEvent(event);
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  }

  /// Handle WebSocket event
  private handleEvent(event: WsEvent): void {
    switch (event.type) {
      case 'ContentUpdate':
        this.onContentUpdate(event as ContentUpdate);
        break;
      case 'UserPresence':
        this.onUserPresence(event as UserPresence);
        break;
    }
  }

  /// Attempt to reconnect
  private attemptReconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      setTimeout(() => {
        this.connect(this.getWebSocketUrl());
      }, this.reconnectDelay * Math.pow(2, this.reconnectAttempts));
    }
  }

  /// Get WebSocket URL
  private getWebSocketUrl(): string {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.hostname;
    const port = window.location.port;
    return `${protocol}//${host}:${port}/ws`;
  }
}

/// Content update event
export interface ContentUpdate {
  type: 'ContentUpdate';
  id: string;
  content: string;
}

/// User presence event
export interface UserPresence {
  type: 'UserPresence';
  user_id: string;
  document_id: string;
}

/// WebSocket event
export type WsEvent = ContentUpdate | UserPresence;
```

### 8.3. Editor Component

#### 8.3.1. Markdown Editor

The editor component provides a content-editable div for Markdown editing with syntax highlighting.

**Editor Component ([`tachyon/web/src/components/editor.rs`](../tachyon/web/src/components/editor.rs)):**

```rust
use leptos::*;
use leptos_meta::*;
use web_sys::HtmlElement;
use wasm_bindgen::prelude::*;

/// Editor component
#[component]
pub fn EditorComponent() -> impl IntoView {
    view! {
        <div class="editor-container">
            <div
                class="editor-toolbar"
                on:click=toggle_bold
                on:click=toggle_italic
                on:click=insert_link>
                <button class="toolbar-button" title="Bold">B</button>
                <button class="toolbar-button" title="Italic">I</button>
                <button class="toolbar-button" title="Insert Link">🔗</button>
            </div>
            <div
                class="editor-content"
                contenteditable="true"
                prop:value={editor_content}
                on:input=on_editor_input
                on:keydown=on_editor_keydown>
            </div>
            <div class="editor-preview">
                <div inner_html={rendered_content} />
            </div>
        </div>
    }
}

/// Editor state
#[derive(Clone, Debug, Default)]
pub struct EditorState {
    /// Editor content
    #[default]
    content: String,
    /// Rendered content
    #[default]
    rendered: String,
    /// Bold mode
    #[default]
    bold: bool,
    /// Italic mode
    #[default]
    italic: bool,
}

/// Toggle bold mode
fn toggle_bold() {
    EditorState::update(|state| state.bold = !state.bold);
}

/// Toggle italic mode
fn toggle_italic() {
    EditorState::update(|state| state.italic = !state.italic);
}

/// Insert link
fn insert_link() {
    EditorState::update(|state| {
        state.content.push_str("[Link]()");
    });
}

/// Handle editor input
fn on_editor_input(event: KeyboardEvent) {
    EditorState::update(|state| {
        state.content = event.target_as_html().unwrap_or_default();
    });
}

/// Handle editor keydown
fn on_editor_keydown(event: KeyboardEvent) {
    match event.key().as_str() {
        "b" if event.ctrl_key() => toggle_bold(),
        "i" if event.ctrl_key() => toggle_italic(),
        "k" if event.ctrl_key() => insert_link(),
        _ => {}
    }
}
```

---

## 9. REFERENCES

### 9.1. Internal References

This document references the following internal project documents:

| Document ID | Title | Location |
|-------------|-------|----------|
| [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) | Coding and Documentation Standards |
| [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) | ADR-001: Rust as Primary Language |
| [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) | ADR-010: Security Architecture |
| [TACHYON-REQ-SYS-V1.0](../../.specs/04_future_state/reqs/system_overview.md) | System Overview Requirements |
| [TACHYON-REQ-DESK-V1.0](../../.specs/04_future_state/reqs/desktop_requirements.md) | Desktop Application Requirements |
| [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) | Server Application Requirements |
| [TACHYON-REQ-WEB-V1.0](../../.specs/04_future_state/reqs/web_requirements.md) | Web Frontend Requirements |

### 9.2. External References

This document references the following external standards and resources:

| Standard | Reference | Purpose |
|----------|----------|---------|
| ISO/IEC 26514:2021 | Systems and Software Engineering Requirements for Designers and Developers of User Documentation |
| ISO/IEC 12207:2017 | Systems and Software Engineering Software Life Cycle Processes |
| ISO/IEC 25010:2011 | Systems and Software Engineering Systems and Software Quality Requirements and Evaluation (SQuaRE) System and Software Quality Models |
| IEEE 829-2008 | Software Test Documentation |
| IEEE 1063-2001 | Standard for Software User Documentation |
| IEEE 1016-2009 | Standard for Information Technology Software Design |
| WCAG 2.1 | Web Content Accessibility Guidelines (WCAG) 2.1 |

### 9.3. Technology References

This document references the following technology documentation:

| Technology | Version | Reference |
|-----------|----------|----------|
| Rust | Edition 2024 | [The Rust Programming Language](https://doc.rust-lang.org/book/) |
| Tokio | 1.x | [Tokio: Asynchronous Runtime for Rust](https://tokio.rs/) |
| Tauri | 2.x | [Tauri: Build smaller, faster binaries for all major desktop platforms](https://tauri.app/) |
| Axum | 0.7.x | [Axum: Ergonomic and Modular Web Framework](https://github.com/tokio-rs/axum) |
| Leptos | 0.8.15 | [Leptos: Build Fast Web Applications with Rust](https://leptos.dev/) |
| Bun | 7.3.1 | [Bun: Incredibly Fast JavaScript Runtime, Package Manager, Bundler, and Test Runner](https://bun.sh/) |
| pulldown-cmark | 0.9.x | [pulldown-cmark: CommonMark parser and renderer for Rust](https://github.com/raphlinstein/pulldown-cmark) |
| Tantivy | 0.21.x | [Tantivy: Full-text search engine library in Rust](https://github.com/quickwit-inc/tantivy) |
| git2-rs | 0.18.x | [git2-rs: Bindings to libgit2 for Rust](https://github.com/rust-lang/git2-rs) |
| SQLite | 0.29.x | [SQLite: Self-contained, Serverless, Zero-Configuration, Transactional SQL Database Engine](https://www.sqlite.org/) |
| serde | 1.x | [serde: Serialization framework for Rust](https://serde.rs/) |
| tracing | 0.1.x | [tracing: Structured, extensible, composable logging and tracing for Rust](https://tokio.rs/tracing) |

### 9.4. Document Change History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| V1.0 | 2026-02-06 | Initial creation |

---

**Document Status:** Approved for Implementation

This document is part of the Tachyon documentation suite and complies with all applicable standards and regulations. For questions or issues related to this document, please refer to the project issue tracker.




which rustc
```

**Using Direnv (Alternative):**

```bash
# Install direnv
cargo install direnv

# Hook direnv into shell
echo 'eval "$(direnv hook bash)"' >> ~/.bashrc

# Allow .envrc
direnv allow
```

**Manual Setup (Fallback):**

```bash
# Install Rust dependencies
cargo install cargo-watch sccache

# Install Bun dependencies
cd tachyon/web
bun install
```

#### 3.3.3. Initial Build

**Build All Components:**

```bash
# Build Rust workspace
cargo build --release

# Build web frontend
cd tachyon/web
bun run build

# Build desktop application
cd tachyon/crates/desktop
cargo tauri build
```

**Verify Build:**

```bash
# Run tests
cargo test --all

# Run linter
cargo clippy --all-targets

# Format code
cargo fmt --all -- --check
```

### 3.4. Development Tools

#### 3.4.1. IDE Configuration

**VS Code Extensions:**

| Extension | Purpose |
|----------|---------|
| **rust-analyzer** | Rust language server |
| **CodeLLDB** | Rust debugger |
| **Even Better TOML** | TOML syntax highlighting |
| **Leptos** | Leptos syntax highlighting |
| **Error Lens** | Inline error display |
| **GitLens** | Git integration |

**VS Code Settings:**

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true,
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-analyzer",
  "[rust]": {
    "editor.defaultFormatter": "rust-analyzer"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

#### 3.4.2. Git Configuration

**Recommended Git Configuration:**

```bash
# Configure Git for Tachyon development
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Configure line endings
git config --global core.autocrlf input

# Configure rebase behavior
git config --global pull.rebase true

# Configure commit signing (optional)
git config --global commit.gpgsign true
```

**Git Hooks:**

```bash
# Install pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Run cargo fmt check
cargo fmt --all -- --check

# Run cargo clippy
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test --all
EOF

chmod +x .git/hooks/pre-commit
```

#### 3.4.3. Build Tools

**Cargo Commands:**

| Command | Purpose |
|---------|---------|
| `cargo build` | Build all crates in debug mode |
| `cargo build --release` | Build all crates in release mode |
| `cargo test` | Run all tests |
| `cargo clippy` | Run linter |
| `cargo fmt` | Format code |
| `cargo doc --open` | Generate and open documentation |
| `cargo watch` | Auto-rebuild on file changes |

**Bun Commands:**

| Command | Purpose |
|---------|---------|
| `bun install` | Install dependencies |
| `bun run dev` | Start development server |
| `bun run build` | Build for production |
| `bun run test` | Run tests |
| `bun run lint` | Run linter |
| `bun run format` | Format code |

**Nix Commands:**

| Command | Purpose |
|---------|---------|
| `nix develop` | Enter development shell |
| `nix build` | Build project |
| `nix flake check` | Check flake configuration |
| `nix flake update` | Update flake inputs |

### 3.5. Environment Variables

#### 3.5.1. Required Variables

```bash
# Rust environment
export RUST_BACKTRACE=1  # Enable backtraces for debugging
export RUST_LOG=debug  # Set logging level

# Bun environment
export NODE_ENV=development  # Set Node environment

# Tachyon environment
export TACHYON_LOG_LEVEL=debug  # Set logging level
export TACHYON_CACHE_DIR=~/.cache/tachyon  # Cache directory
```

#### 3.5.2. Optional Variables

```bash
# Performance tuning
export CARGO_BUILD_JOBS=4  # Number of parallel jobs
export SCCACHE_DIR=~/.cache/sccache  # sccache directory

# Development tools
export EDITOR=code  # Default editor
export PAGER=less  # Default pager

# Git configuration
export GIT_AUTHOR_NAME="Your Name"
export GIT_AUTHOR_EMAIL="your.email@example.com"
```

---

## 4. PROJECT STRUCTURE

### 4.1. Workspace Organization

The Tachyon project is organized as a Cargo workspace containing multiple crates and a web frontend. The workspace structure enables code sharing, unified dependency management, and efficient compilation.

```
tachyon/
├── .specs/                    # Specification and documentation
│   ├── 01_standards/          # Coding and documentation standards
│   ├── 02_adrs/              # Architectural Decision Records
│   ├── 03_current_state/      # Current state analysis
│   ├── 04_future_state/       # Future state design
│   │   ├── reqs/              # Requirements specifications
│   │   ├── design/            # Design documents
│   │   └── test_plan.md       # Test plan
│   └── 05_architecture/        # Architecture documentation
├── .docs/                     # Public documentation
│   ├── architecture/            # Architecture documentation
│   ├── developer/              # Developer documentation
│   ├── quality/                # Quality assurance documentation
│   └── user/                   # User documentation
├── tachyon/                   # Main workspace
│   ├── Cargo.toml              # Workspace configuration
│   ├── Cargo.lock              # Dependency lock file
│   └── crates/                 # Rust crates
│       ├── desktop/            # Desktop application
│       │   └── src-tauri/    # Tauri application
│       │       ├── src/         # Rust backend
│       │       └── tauri.conf.json  # Tauri configuration
│       └── server/             # Server application
│           └── src/         # Rust backend
└── web/                       # Web frontend
    ├── index.ts               # Entry point
    ├── package.json           # Bun dependencies
    ├── tsconfig.json          # TypeScript configuration
    └── README.md              # Web frontend documentation
```

### 4.2. Cargo Workspace

#### 4.2.1. Workspace Configuration

The [`tachyon/Cargo.toml`](../tachyon/Cargo.toml) file defines the workspace structure and shared dependencies:

```toml
[workspace]
resolver = "2"
members = [
    "crates/desktop",
    "crates/server",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.77.2"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
anyhow = "1"
thiserror = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Git integration
git2 = "0.18"

# Database
rusqlite = { version = "0.29", features = ["bundled"] }

# Search
tantivy = "0.21"

# Markdown parsing
pulldown-cmark = { version = "0.9", features = ["simd"] }

# Concurrency
dashmap = "5"

# File system
walkdir = "2"

# Web framework (server only)
axum = { version = "0.7", optional = true }
tower = { version = "0.4", optional = true }
tower-http = { version = "0.4", optional = true }

# Desktop framework (desktop only)
tauri = { version = "2", optional = true }

[workspace.dependencies.validator]
version = "0.18"
features = ["derive"]
```

#### 4.2.2. Desktop Crate

The [`tachyon/crates/desktop/`](../tachyon/crates/desktop) crate implements the desktop application using Tauri.

**Structure:**

```
tachyon/crates/desktop/
├── Cargo.toml              # Desktop crate configuration
└── src-tauri/              # Tauri application
    ├── Cargo.toml          # Tauri Rust dependencies
    ├── build.rs            # Build script
    ├── tauri.conf.json    # Tauri configuration
    ├── capabilities/       # Capability definitions
    │   └── default.json   # Default capabilities
    ├── icons/              # Application icons
    └── src/                # Rust backend
        ├── lib.rs          # Library entry point
        └── main.rs         # Application entry point
```

**Key Responsibilities:**

- Native desktop application lifecycle
- WebView integration and management
- IPC communication with server component
- Native OS integration (file dialogs, notifications)
- System tray integration
- URL protocol handling

#### 4.2.3. Server Crate

The [`tachyon/crates/server/`](../tachyon/crates/server) crate implements the HTTP/2 server using Axum.

**Structure:**

```
tachyon/crates/server/
├── Cargo.toml              # Server crate configuration
└── src/                    # Rust backend
    ├── main.rs             # Application entry point
    ├── lib.rs              # Library entry point
    ├── api/                # API endpoints
    │   ├── mod.rs
    │   ├── documents.rs
    │   ├── search.rs
    │   ├── git.rs
    │   └── auth.rs
    ├── core/               # Core functionality
    │   ├── mod.rs
    │   ├── renderer.rs
    │   ├── cache.rs
    │   └── indexer.rs
    ├── db/                 # Database operations
    │   ├── mod.rs
    │   ├── models.rs
    │   └── schema.rs
    ├── git/                # Git integration
    │   ├── mod.rs
    │   └── operations.rs
    └── websocket/           # WebSocket handling
        ├── mod.rs
        └── handler.rs
```

**Key Responsibilities:**

- HTTP/2 server implementation
- API endpoint handling
- WebSocket communication
- JIT rendering pipeline
- Search indexing and querying
- Git repository management
- Database operations
- Authentication and authorization

### 4.3. Web Frontend

#### 4.3.1. Frontend Structure

The [`tachyon/web/`](../tachyon/web) directory contains the Leptos web frontend.

**Structure:**

```
tachyon/web/
├── index.ts               # Application entry point
├── package.json           # Bun dependencies
├── tsconfig.json          # TypeScript configuration
├── README.md              # Frontend documentation
└── src/                   # Source code
    ├── main.rs             # Leptos main component
    ├── app.rs              # App component
    ├── components/         # UI components
    │   ├── mod.rs
    │   ├── editor.rs
    │   ├── navigation.rs
    │   ├── search.rs
    │   └── document_view.rs
    ├── pages/              # Page components
    │   ├── mod.rs
    │   ├── home.rs
    │   ├── editor.rs
    │   └── settings.rs
    ├── state/              # State management
    │   ├── mod.rs
    │   ├── store.rs
    │   └── actions.rs
    ├── api/                # API client
    │   ├── mod.rs
    │   ├── client.rs
    │   └── types.rs
    └── utils/              # Utilities
        ├── mod.rs
        └── helpers.rs
```

**Key Responsibilities:**

- Reactive UI implementation
- Client-side state management
- API client implementation
- WebSocket client
- Editor component
- Search interface
- Navigation components

#### 4.3.2. Frontend Configuration

**[`tachyon/web/package.json`](../tachyon/web/package.json):**

```json
{
  "name": "tachyon-web",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "bun run --bun vite",
    "build": "bun run --bun vite build",
    "preview": "bun run --bun vite preview",
    "test": "bun test",
    "lint": "bun run --bun eslint .",
    "format": "bun run --bun prettier --write ."
  },
  "dependencies": {
    "leptos": "^0.8.15",
    "leptos_axum": "^0.8.15",
    "leptos_meta": "^0.8.15"
  },
  "devDependencies": {
    "vite": "^7.3.1",
    "typescript": "^5.0.0",
    "prettier": "^3.0.0",
    "eslint": "^9.0.0"
  }
}
```

**[`tachyon/web/tsconfig.json`](../tachyon/web/tsconfig.json):**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "jsx": "react-jsx",
    "jsxImportSource": "leptos",
    "strict": true,
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "types": ["leptos", "vite/client"]
  },
  "include": ["src"],
  "exclude": ["node_modules"]
}
```

### 4.4. Module Boundaries

#### 4.4.1. Core Engine Module

The core engine module provides JIT rendering, caching, and content processing functionality shared across all components.

**Responsibilities:**

- Markdown parsing and rendering
- Code syntax highlighting
- Math equation rendering
- Diagram rendering
- Content sanitization
- LRU cache management
- Frontmatter processing

**Interfaces:**

```rust
/// Core rendering engine for JIT content processing
pub trait Renderer {
    /// Render Markdown content to HTML
    ///
    /// # Arguments
    ///
    /// * `content` - Markdown content to render
    ///
    /// # Returns
    ///
    /// Rendered HTML content
    ///
    /// # Errors
    ///
    /// Returns error if rendering fails
    async fn render(&self, content: &str) -> Result<String, RenderError>;

    /// Invalidate cache for specific content
    ///
    /// # Arguments
    ///
    /// * `content_id` - Identifier of content to invalidate
    fn invalidate_cache(&self, content_id: &str);
}
```

#### 4.4.2. IPC Module

The IPC module provides inter-process communication between desktop and server components.

**Responsibilities:**

- Message serialization and deserialization
- Channel management
- Connection handling
- Error handling and retry logic
- Message queuing and delivery

**Message Types:**

```rust
/// IPC message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Request document content
    GetDocument { id: String },
    /// Document content response
    Document { id: String, content: String },
    /// Notify of content update
    ContentUpdate { id: String, content: String },
    /// Error message
    Error { message: String },
}
```

#### 4.4.3. Storage Module

The storage module provides abstraction layer for file system and Git repository access.

**Responsibilities:**

- File system operations
- Git repository operations
- Database operations
- Search index operations
- Transaction management
- Concurrency control

**Interfaces:**

```rust
/// Storage abstraction layer
pub trait Storage {
    /// Read document content
    ///
    /// # Arguments
    ///
    /// * `id` - Document identifier
    ///
    /// # Returns
    ///
    /// Document content
    ///
    /// # Errors
    ///
    /// Returns error if document not found or read fails
    async fn read_document(&self, id: &str) -> Result<String, StorageError>;

    /// Write document content
    ///
    /// # Arguments
    ///
    /// * `id` - Document identifier
    /// * `content` - Document content
    ///
    /// # Returns
    ///
    /// Unit
    ///
    /// # Errors
    ///
    /// Returns error if write fails
    async fn write_document(&self, id: &str, content: &str) -> Result<(), StorageError>;
}
```

---

## 5. DEVELOPMENT WORKFLOW

### 5.1. Development Process

#### 5.1.1. Feature Development

**Feature Development Lifecycle:**

1. **Planning Phase**
   - Review requirements from [`.specs/04_future_state/reqs/`](../../.specs/04_future_state/reqs/)
   - Review design documents from [`.specs/04_future_state/design/`](../../.specs/04_future_state/design/)
   - Create implementation plan with clear acceptance criteria
   - Estimate effort and identify dependencies

2. **Implementation Phase**
   - Create feature branch from `main`
   - Implement feature following coding standards
   - Write unit tests for all new code
   - Document public APIs with doc comments
   - Run `cargo fmt` and `cargo clippy`

3. **Testing Phase**
   - Run unit tests: `cargo test`
   - Run integration tests: `cargo test --test integration`
   - Verify code coverage: `cargo tarpaulin --out Html`
   - Manual testing of functionality

4. **Review Phase**
   - Submit pull request with clear description
   - Address review feedback
   - Ensure all CI checks pass
   - Update documentation as needed

5. **Merge Phase**
   - Squash commits if necessary
   - Merge to `main` via pull request
   - Delete feature branch
   - Update changelog

**Branch Naming Convention:**

- Feature branches: `feature/<feature-name>`
- Bugfix branches: `bugfix/<bug-description>`
- Hotfix branches: `hotfix/<hotfix-description>`
- Release branches: `release/<version>`

#### 5.1.2. Bug Fixing

**Bug Fixing Process:**

1. **Bug Report**
   - Create issue with detailed description
   - Include steps to reproduce
   - Attach relevant logs and screenshots
   - Assign priority and labels

2. **Investigation**
   - Reproduce bug locally
   - Identify root cause
   - Review related code and tests
   - Check for similar issues

3. **Fix Implementation**
   - Create bugfix branch
   - Implement minimal fix
   - Add regression tests
   - Verify fix resolves issue

4. **Testing**
   - Run affected tests
   - Perform manual verification
   - Check for side effects
   - Update documentation if needed

5. **Deployment**
   - Submit pull request
   - Include issue reference in description
   - Merge after review approval
   - Close related issues

### 5.2. Code Review Process

#### 5.2.1. Review Checklist

**Code Review Criteria:**

- [ ] Code follows coding standards
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] Code passes linter (`cargo clippy`)
- [ ] All tests pass
- [ ] New code has unit tests
- [ ] Public APIs have documentation
- [ ] Error handling is comprehensive
- [ ] Security considerations addressed
- [ ] Performance impact evaluated
- [ ] Documentation updated

#### 5.2.2. Review Guidelines

**For Reviewers:**

- Provide constructive, specific feedback
- Focus on code quality and correctness
- Suggest improvements without being prescriptive
- Verify tests cover edge cases
- Check for security vulnerabilities
- Ensure documentation is clear

**For Authors:**

- Respond to all review comments
- Address feedback systematically
- Explain design decisions
- Update tests as requested
- Keep commits atomic and focused

### 5.3. Testing Strategy

#### 5.3.1. Test Organization

**Test Structure:**

```
tachyon/crates/<crate>/src/
├── lib.rs              # Library code
├── main.rs             # Binary code
└── tests/              # Integration tests
    ├── mod.rs
    ├── test_renderer.rs
    ├── test_cache.rs
    └── test_api.rs
```

**Test Categories:**

| Category | Purpose | Location |
|----------|---------|----------|
| **Unit Tests** | Test individual functions and methods | In same file as code |
| **Integration Tests** | Test component interactions | `tests/` directory |
| **Documentation Tests** | Test code examples | In doc comments |
| **Property Tests** | Test invariants with random inputs | `tests/` directory |

#### 5.3.2. Testing Guidelines

**Unit Test Guidelines:**

- Test all public functions with non-trivial logic
- Test edge cases (empty inputs, boundary values)
- Test error conditions
- Use descriptive test names
- Keep tests focused and independent

**Example Unit Test:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_render_markdown_success() {
        let renderer = Renderer::new();
        let result = renderer.render("# Hello").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<h1>Hello</h1>");
    }

    #[tokio::test]
    async fn test_render_markdown_empty() {
        let renderer = Renderer::new();
        let result = renderer.render("").await;
        assert!(result.is_err());
    }
}
```

**Integration Test Guidelines:**

- Test component interactions
- Test API endpoints
- Test database operations
- Test WebSocket communication
- Use test fixtures for setup/teardown

**Example Integration Test:**

```rust
#[tokio::test]
async fn test_api_get_document() {
    // Setup
    let app = create_test_app().await;
    let storage = create_test_storage().await;
    storage.write_document("test", "# Test").await.unwrap();

    // Test
    let response = app
        .oneshot(Request::builder()
            .uri("/api/documents/test")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("<h1>Test</h1>"));
}
```

### 5.4. Continuous Integration

#### 5.4.1. CI Pipeline

**CI Stages:**

1. **Lint Stage**
   - Run `cargo fmt --all -- --check`
   - Run `cargo clippy --all-targets`
   - Run `bun run lint` (web frontend)

2. **Test Stage**
   - Run `cargo test --all`
   - Run `bun run test` (web frontend)
   - Generate coverage report

3. **Build Stage**
   - Run `cargo build --release`
   - Run `bun run build` (web frontend)

4. **Security Stage**
   - Run `cargo audit`
   - Run `cargo deny check`
   - Scan for vulnerabilities

5. **Documentation Stage**
   - Run `cargo doc --no-deps`
   - Check for broken documentation links

#### 5.4.2. CI Configuration

**GitHub Actions Example:**

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
```

### 5.5. Release Process

#### 5.5.1. Version Management

**Semantic Versioning:**

- **Major version (X.0.0):** Breaking changes
- **Minor version (0.X.0):** New features, backward compatible
- **Patch version (0.0.X):** Bug fixes, backward compatible

**Release Checklist:**

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Version bumped in Cargo.toml
- [ ] Release branch created
- [ ] Release notes prepared
- [ ] CI/CD pipeline verified

#### 5.5.2. Release Procedure

**Release Steps:**

1. **Preparation**
   - Create release branch from `main`
   - Update version numbers
   - Update changelog
   - Tag release with version

2. **Build**
   - Build release artifacts
   - Run full test suite
   - Generate documentation

3. **Verification**
   - Test release artifacts
   - Verify documentation
   - Check for regressions

4. **Publishing**
   - Publish to crates.io
   - Create GitHub release
   - Upload release artifacts
   - Announce release

5. **Post-Release**
   - Merge release to `main`
   - Delete release branch
   - Update documentation website




