---
title: Getting Started
description: Installation, setup, and first steps with Tachyon
order: 0
tags: [guide, setup]
---

# Getting Started

## Prerequisites

- **Rust** 1.75+ (stable toolchain)
- **PostgreSQL** 16+
- **Node.js** 20+ (for Trunk WASM builds)

## Installation

### From Source

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon

# Build the backend
cargo build --release

# Build the frontend (requires trunk)
cargo install trunk
trunk build --release --packages tachyon-frontend
```

### Using Nix (recommended)

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon
nix develop  # or 'use flake' if you have direnv
cargo run --release
```

## Configuration

Create a `.env` file in the `tachyon/` directory:

```env
DATABASE_URL=postgres://user:password@localhost/tachyon
JWT_SECRET=your-secret-key-here
RUST_LOG=tachyon_server=debug
```

## Running

```bash
# Start the server (backend + frontend)
cargo run --release

# Or separately:
cargo run --release --bin tachyon-server    # Backend on :8080
trunk serve --packages tachyon-frontend       # Frontend dev on :8080
```

## Creating Your First Document

1. Open `http://localhost:8080` in your browser
2. Register a new account (first user is admin)
3. Navigate to Documents → + New Document
4. Write in Markdown with live preview
5. The document auto-saves locally and syncs to the server

## What's Next

- [**Editor Guide**](editor-guide.html) — Keyboard shortcuts and editor features
- [**API Reference**](api-reference.html) — REST API documentation
- [**Configuration**](configuration.html) — Advanced configuration options
- [**Deployment**](deployment.html) — Production deployment guide
