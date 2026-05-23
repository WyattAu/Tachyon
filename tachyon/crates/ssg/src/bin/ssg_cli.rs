//! tachyon-ssg-cli — Build static documentation sites from markdown files.
//!
//! Usage:
//!   tachyon-ssg-cli build --input ./docs --output ./site
//!   tachyon-ssg-cli build --input ./docs --output ./site --config site.toml
//!
//! Input files are markdown with optional YAML frontmatter:
//!
//!   ---
//!   title: Getting Started
//!   description: Quick start guide for Tachyon
//!   tags: [guide, tutorial]
//!   order: 1
//!   ---
//!
//!   # Getting Started
//!
//!   Content here...

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tachyon_ssg::{ColorTheme, NavLink, SiteConfig, SiteGenerator, SsgDocument};

#[derive(Parser, Debug)]
#[command(
    name = "tachyon-ssg-cli",
    about = "Build static documentation sites from markdown"
)]
enum Cli {
    /// Build a static site from markdown files
    Build {
        /// Input directory containing markdown files
        #[arg(short, long, default_value = "docs")]
        input: PathBuf,

        /// Output directory for generated HTML
        #[arg(short, long, default_value = "site")]
        output: PathBuf,

        /// Path to site configuration TOML file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Site title (overrides config)
        #[arg(long)]
        title: Option<String>,

        /// Site description (overrides config)
        #[arg(long)]
        description: Option<String>,

        /// Base URL for canonical links
        #[arg(long, default_value = "https://wyattau.github.io/Tachyon")]
        base_url: String,

        /// Include a 404.html page
        #[arg(long, default_value = "true")]
        with_404: bool,
    },

    /// Initialize a new docs directory with example files
    Init {
        /// Target directory to create
        #[arg(short, long, default_value = "docs")]
        target: PathBuf,
    },

    /// Serve the site locally with live rebuild on file changes
    Serve {
        /// Input directory containing markdown files
        #[arg(short, long, default_value = "docs")]
        input: PathBuf,

        /// Output directory for generated HTML
        #[arg(short, long, default_value = "site")]
        output: PathBuf,

        /// Path to site configuration TOML file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Base URL for canonical links
        #[arg(long, default_value = "http://localhost:3000")]
        base_url: String,
    },
}

/// Parsed frontmatter from a markdown file.
#[derive(Debug, Default, serde::Deserialize)]
struct Frontmatter {
    title: Option<String>,
    description: Option<String>,
    author: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    order: i32,
    #[serde(default)]
    language: String,
}

/// YAML frontmatter parser using serde_yaml.
fn parse_frontmatter(content: &str) -> Result<(Frontmatter, String), Box<dyn std::error::Error>> {
    let trimmed = content
        .trim_start_matches("---\n")
        .trim_start_matches("---\r\n");
    let end = trimmed
        .find("---")
        .ok_or("Missing closing --- in frontmatter")?;
    let yaml_block = &trimmed[..end];
    let body = trimmed[end + 3..].trim_start().to_string();
    let frontmatter: Frontmatter = serde_yaml::from_str(yaml_block)?;
    Ok((frontmatter, body))
}

/// Derive title from first H1 heading in markdown content.
fn title_from_content(content: &str, filename: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return stripped.to_string();
        }
    }
    // Fall back to filename
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

/// Read a TOML config file using serde.
fn read_config(path: &Path) -> Result<SiteConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let config: SiteConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    Ok(config)
}

/// Collect and parse all markdown files from the input directory.
fn collect_documents(input_dir: &Path) -> Result<Vec<SsgDocument>> {
    let mut docs = Vec::new();

    if !input_dir.exists() {
        anyhow::bail!("Input directory does not exist: {}", input_dir.display());
    }

    for entry in walkdir::WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        match path.extension().and_then(|e| e.to_str()) {
            Some("md") | Some("markdown") => {}
            _ => continue,
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;

        let (fm, body) = parse_frontmatter(&content).map_err(|e| {
            anyhow::anyhow!("Failed to parse frontmatter: {} — {}", path.display(), e)
        })?;

        // Derive slug from relative path
        let rel = path.strip_prefix(input_dir).unwrap_or(path);
        let slug = rel
            .with_extension("")
            .to_str()
            .unwrap_or("untitled")
            .replace('\\', "/");

        let title = fm
            .title
            .unwrap_or_else(|| title_from_content(&body, path.to_str().unwrap_or("untitled")));

        let now = Utc::now();

        docs.push(SsgDocument {
            slug,
            title,
            content: body,
            description: fm.description,
            author: fm.author,
            tags: fm.tags,
            created_at: now,
            updated_at: now,
            order: fm.order,
            language: if fm.language.is_empty() {
                "en".to_string()
            } else {
                fm.language
            },
        });
    }

    // Sort by order, then by slug
    docs.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.slug.cmp(&b.slug)));

    Ok(docs)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli {
        Cli::Build {
            input,
            output,
            config,
            title,
            description,
            base_url,
            with_404,
        } => cmd_build(
            input,
            output,
            config,
            title,
            description,
            &base_url,
            with_404,
        ),
        Cli::Init { target } => cmd_init(&target),
        Cli::Serve {
            input,
            output,
            config,
            port,
            base_url,
        } => cmd_serve(input, output, config, port, &base_url),
    }
}

fn cmd_build(
    input: PathBuf,
    output: PathBuf,
    config: Option<PathBuf>,
    title: Option<String>,
    description: Option<String>,
    base_url: &str,
    with_404: bool,
) -> Result<()> {
    println!("📄 Tachyon SSG — Static Site Generator");
    println!("   Input:  {}", input.display());
    println!("   Output: {}", output.display());
    println!();

    // Load config
    let mut site_config = if let Some(config_path) = config {
        read_config(&config_path)?
    } else {
        let config_path = input.join("site.toml");
        if config_path.exists() {
            read_config(&config_path)?
        } else {
            SiteConfig::default()
        }
    };

    // Apply CLI overrides
    if let Some(t) = title {
        site_config.title = t;
    }
    if let Some(d) = description {
        site_config.description = d;
    }
    site_config.base_url = base_url.to_string();

    // Set sensible defaults for documentation sites
    if site_config.nav_links.is_empty() {
        site_config.nav_links = vec![
            NavLink {
                label: "Home".to_string(),
                href: "./index.html".to_string(),
            },
            NavLink {
                label: "GitHub".to_string(),
                href: "https://github.com/WyattAu/Tachyon".to_string(),
            },
        ];
    }

    site_config.color_theme = Some(ColorTheme {
        primary: "#2563eb".to_string(),
        secondary: "#7c3aed".to_string(),
        accent: "#06b6d4".to_string(),
        code_bg: "#1f2937".to_string(),
        font_family: None,
        heading_font_family: None,
    });

    // Collect documents
    println!("🔍 Scanning for markdown files...");
    let docs = collect_documents(&input)?;
    println!("   Found {} documents", docs.len());

    if docs.is_empty() {
        anyhow::bail!("No markdown files found in {}", input.display());
    }

    for doc in &docs {
        println!("   - {} ({})", doc.title, doc.slug);
    }
    println!();

    // Build
    let image_optimization = site_config.image_optimization_enabled;
    println!("🔨 Building static site...");
    let generator = SiteGenerator::new(site_config);
    let result = generator.build_to_dir(&docs, &output)?;

    println!();
    println!("✅ Build complete!");
    println!("   Pages:        {}", result.pages);
    println!("   Categories:   {}", result.category_pages);
    println!("   Total files:  {}", result.total_files);
    println!("   Languages:    {}", result.languages);
    println!(
        "   Output size:  {:.1} KB",
        result.output_size_bytes as f64 / 1024.0
    );
    println!("   Build time:   {}ms", result.build_time_ms);
    println!();

    // Write 404 page
    if with_404 {
        let not_found = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Not Found</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex; align-items: center; justify-content: center;
            min-height: 100vh; background: #f9fafb; color: #111827;
        }
        .container { text-align: center; padding: 2rem; }
        h1 { font-size: 6rem; font-weight: 700; color: #2563eb; margin-bottom: 0.5rem; }
        p { font-size: 1.25rem; color: #6b7280; margin-bottom: 1.5rem; }
        a { color: #2563eb; text-decoration: none; font-weight: 500; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <div class="container">
        <h1>404</h1>
        <p>Page not found</p>
        <a href="/">← Back to Home</a>
    </div>
</body>
</html>"#;
        let not_found_path = output.join("404.html");
        fs::write(&not_found_path, not_found)?;
        println!("   Wrote 404.html");
    }

    // Write .nojekyll (GitHub Pages requirement)
    fs::write(output.join(".nojekyll"), "")?;
    println!("   Wrote .nojekyll");

    // Copy static assets (images, fonts, CSS, etc.)
    println!();
    let asset_stats = tachyon_ssg::assets::copy_static_assets(&input, &output, image_optimization)?;
    if asset_stats.files_copied > 0 {
        println!(
            "📦 Copied {} static assets ({} images optimized, {:.1}KB saved)",
            asset_stats.files_copied,
            asset_stats.images_optimized,
            asset_stats.bytes_saved as f64 / 1024.0,
        );
    }

    println!();
    println!("🚀 Site ready at: {}", output.display());

    Ok(())
}

fn cmd_init(target: &Path) -> Result<()> {
    if target.exists() {
        anyhow::bail!("Directory already exists: {}", target.display());
    }

    fs::create_dir_all(target)?;
    println!("✅ Created docs directory: {}", target.display());

    let files: BTreeMap<&str, &str> = BTreeMap::from([
        (
            "site.toml",
            r#"title = "Tachyon Documentation"
description = "A fast, offline-first knowledge management platform built with Rust"
base_url = "https://wyattau.github.io/Tachyon"
footer = "Built with Tachyon"
theme = "auto"
group_by_tag = true
"#,
        ),
        (
            "index.md",
            r#"---
title: Welcome to Tachyon
description: A fast, offline-first knowledge management platform built with Rust
order: -1
---

# Welcome to Tachyon

Tachyon is a high-performance knowledge management platform built entirely in Rust. It features real-time collaboration via CRDTs, a native editor, and offline-first architecture.

## Features

- **Markdown-first editing** with rich preview
- **Real-time collaboration** with CRDT sync (Yrs/lib0)
- **Offline-first** — works without network, syncs when connected
- **Knowledge graph** with bidirectional links
- **Full-text search** with Tantivy + PostgreSQL
- **Static site generation** for documentation
- **Plugin system** for extensibility
- **Role-based access control** with teams and spaces

## Quick Start

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
cargo run --release
```

The server starts at `http://localhost:8080`.

## Architecture

Tachyon is built as a Rust workspace with the following crates:

| Crate | Purpose |
|-------|---------|
| `tachyon-server` | Axum HTTP server with JWT auth |
| `tachyon-frontend` | Leptos 0.8 WASM frontend (CSR) |
| `tachyon-editor` | Native Rust editor with CRDT support |
| `tachyon-core` | Shared types and utilities |
| `tachyon-database` | PostgreSQL via sqlx |
| `tachyon-search` | Tantivy + PostgreSQL hybrid search |
| `tachyon-ssg` | Static site generator |
| `tachyon-renderer` | Markdown to HTML renderer |
"#,
        ),
        (
            "getting-started.md",
            r#"---
title: Getting Started
description: Installation, setup, and first steps with Tachyon
order: 0
tags: [guide, setup]
---

# Getting Started

## Prerequisites

- **Rust** 1.75+ (stable toolchain)
- **PostgreSQL** 15+
- **Node.js** 18+ (for Trunk WASM builds)

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
"#,
        ),
        (
            "editor-guide.md",
            r#"---
title: Editor Guide
description: Keyboard shortcuts, features, and configuration for the native editor
order: 1
tags: [guide, editor]
---

# Editor Guide

Tachyon uses a native Rust editor (not CodeMirror or Monaco) built on `ropey` for text management and `yrs` for CRDT sync.

## Keyboard Shortcuts

### Navigation

| Shortcut | Action |
|----------|--------|
| `Ctrl+G` / `Cmd+G` | Go to line |
| `Ctrl+F` / `Cmd+F` | Find in document |
| `Ctrl+H` / `Cmd+H` | Find and replace |
| `Ctrl+Home` | Jump to start of document |
| `Ctrl+End` | Jump to end of document |

### Editing

| Shortcut | Action |
|----------|--------|
| `Tab` | Insert indent |
| `Shift+Tab` | Outdent |
| `Ctrl+Z` / `Cmd+Z` | Undo |
| `Ctrl+Shift+Z` / `Cmd+Shift+Z` | Redo |
| `Ctrl+A` / `Cmd+A` | Select all |
| `Ctrl+D` / `Cmd+D` | Delete line |

### Markdown Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` / `Cmd+B` | Bold (`**text**`) |
| `Ctrl+I` / `Cmd+I` | Italic (`*text*`) |
| `Ctrl+K` / `Cmd+K` | Insert link |
| `Ctrl+Shift+K` | Insert code block |

## Features

### Syntax Highlighting

The editor provides syntax highlighting for:

- Markdown headings, bold, italic, links, code
- Code blocks with language detection
- Lists (ordered and unordered)
- Blockquotes and tables

### Real-time Collaboration

When multiple users edit the same document, changes are synced in real-time via WebSocket using Yrs (Yjs Rust port):

- Character-level conflict resolution
- Cursor presence (see other users' cursors)
- No merge conflicts — CRDTs handle concurrent edits

### Search

Press `Ctrl+F` to open the search panel:

- **Case sensitive** toggle (`Aa`)
- **Whole word** match toggle (`W`)
- **Regex** mode toggle (`.*`)
- Navigate matches with arrow buttons or `Enter`/`Shift+Enter`
- Replace single or all matches

## Markdown Support

Tachyon supports CommonMark markdown with extensions:

- Tables
- Task lists (`- [x] done`)
- Footnotes
- Strikethrough (`~~text~~`)
- Highlight (`==text==`)
- Math (basic inline)
- Mermaid diagrams (via plugin)
"#,
        ),
        (
            "api-reference.md",
            r#"---
title: API Reference
description: REST API endpoints for the Tachyon server
order: 2
tags: [api, reference]
---

# API Reference

All API endpoints are prefixed with `/api/v1`. Authentication uses Bearer JWT tokens.

## Authentication

### Register
```
POST /api/v1/auth/register
Content-Type: application/json

{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

### Login
```
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "string",
  "password": "string"
}

Response: { "token": "jwt...", "user": { ... } }
```

## Documents

### List Documents
```
GET /api/v1/documents?page=1&page_size=20&status=published
Authorization: Bearer <token>
```

### Create Document
```
POST /api/v1/documents
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "string",
  "content": "markdown content",
  "project_id": "uuid (optional)"
}
```

### Get Document
```
GET /api/v1/documents/:id
Authorization: Bearer <token>
```

### Update Document
```
PUT /api/v1/documents/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "string",
  "content": "markdown content",
  "status": "draft|published|archived"
}
```

### Delete Document
```
DELETE /api/v1/documents/:id
Authorization: Bearer <token>
```

## Search

### Search Documents
```
GET /api/v1/search?q=query&status=published&tags=tag1,tag2&page=1&page_size=20
Authorization: Bearer <token>
```

### Global Search
```
GET /api/v1/search/global?q=query
Authorization: Bearer <token>
```

### Autocomplete Suggestions
```
GET /api/v1/search/suggest?q=que&limit=5
Authorization: Bearer <token>
```

## SSG (Static Site Generator)

### Get SSG Config
```
GET /api/v1/ssg/config
Authorization: Bearer <token>
```

### Build Static Site
```
POST /api/v1/ssg/build
Authorization: Bearer <token>

Response: { "build_result": { "pages": 5, "total_files": 8, ... } }
```

### Download Static Site (ZIP)
```
GET /api/v1/ssg/download
Authorization: Bearer <token>

Response: application/zip
```

## Teams

### List Teams
```
GET /api/v1/teams
Authorization: Bearer <token>
```

### Create Team
```
POST /api/v1/teams
Authorization: Bearer <token>
Content-Type: application/json

{ "name": "string", "description": "string" }
```

## Plugins

### List Plugins
```
GET /api/v1/plugins
Authorization: Bearer <token>
```

### Install Plugin
```
POST /api/v1/plugins/install
Authorization: Bearer <token>
Content-Type: application/json

{ "repository_url": "string" }
```
"#,
        ),
        (
            "configuration.md",
            r#"---
title: Configuration
description: Advanced configuration options for Tachyon
order: 3
tags: [config, reference]
---

# Configuration

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | PostgreSQL connection string (required) |
| `JWT_SECRET` | — | Secret key for JWT signing (required) |
| `RUST_LOG` | `info` | Log level filter |
| `SERVER_HOST` | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | `8080` | Server bind port |
| `CORS_ORIGINS` | `*` | Allowed CORS origins |
| `UPLOAD_DIR` | `./uploads` | File upload directory |
| `MAX_UPLOAD_SIZE` | `50MB` | Maximum file upload size |

## Database Setup

```bash
# Create database
createdb tachyon

# Run migrations (sqlx automatically runs on startup)
# Or manually:
sqlx migrate run
```

## Frontend Build Configuration

The frontend is built with Trunk. Key configuration in `crates/frontend/Trunk.toml`:

```toml
[build]
target = "index.html"
dist = "dist"
public_url = "/"

[serve]
addresses = ["127.0.0.1"]
port = 8080
```

### Tailwind CSS

Tachyon uses Tailwind CSS via Play CDN (loaded in `index.html`). For production, consider migrating to a build-time Tailwind pipeline.

## Search Configuration

Tantivy search index is stored at `.tachyon/search_index/` by default. Configure via:

```rust
let config = IndexConfig::new("tachyon")
    .with_index_path("./custom_index");
```

### Reindexing

Trigger a full reindex via the API:
```
POST /api/v1/search/reindex
Authorization: Bearer <token>
```

## SSG Configuration

The static site generator can be configured per-site:

```toml
title = "My Docs"
description = "Documentation for my project"
base_url = "https://docs.example.com"
footer = "Built with Tachyon"
theme = "auto"  # "light", "dark", or "auto"
group_by_tag = true
```

### Building Docs from CLI

```bash
# Build from markdown files
tachyon-ssg-cli build --input ./docs --output ./site

# With custom config
tachyon-ssg-cli build --input ./docs --output ./site --config site.toml

# For GitHub Pages
tachyon-ssg-cli build --input ./docs --output ./site \
  --base-url "https://username.github.io/repo"
```
"#,
        ),
        (
            "deployment.md",
            r#"---
title: Deployment
description: Production deployment guide for Tachyon
order: 4
tags: [guide, deployment]
---

# Deployment

## Docker

```bash
# Build
docker build -t tachyon .

# Run
docker run -d \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@db:5432/tachyon \
  -e JWT_SECRET=your-secret \
  tachyon
```

### Docker Compose

```yaml
version: '3.8'
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_DB: tachyon
      POSTGRES_USER: tachyon
      POSTGRES_PASSWORD: password
    volumes:
      - pgdata:/var/lib/postgresql/data

  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://tachyon:password@db:5432/tachyon
      JWT_SECRET: your-secret
    depends_on:
      - db

volumes:
  pgdata:
```

## Static Site (GitHub Pages)

Tachyon can generate a static documentation site:

```bash
# Build the SSG CLI
cargo build --release -p tachyon-ssg

# Generate static site
./target/release/tachyon-ssg-cli build \
  --input ./docs \
  --output ./site \
  --base-url "https://username.github.io/Tachyon"
```

### GitHub Actions

The project includes a `.github/workflows/docs.yml` workflow that automatically builds and deploys documentation to GitHub Pages on push to `main`.

## Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name docs.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## TLS

Use Let's Encrypt with Certbot:

```bash
sudo certbot --nginx -d docs.example.com
```
"#,
        ),
        (
            "architecture.md",
            r#"---
title: Architecture
description: System architecture and design decisions
order: 5
tags: [architecture, reference]
---

# Architecture

## System Overview

Tachyon is a full-stack Rust application:

```
┌─────────────────────────────────────────────────────┐
│                   Browser (WASM)                     │
│  ┌─────────────────────────────────────────────┐    │
│  │  Leptos 0.8 (CSR) + Tailwind CSS           │    │
│  │  ┌─────────────┐  ┌──────────────────────┐  │    │
│  │  │ Native Editor│  │ WebSocket (CRDT)     │  │    │
│  │  │ (ropey+yrs) │  │ (axum-tungstenite)  │  │    │
│  │  └─────────────┘  └──────────────────────┘  │    │
│  └─────────────────────────────────────────────┘    │
└──────────────────────┬──────────────────────────────┘
                       │ HTTP + WebSocket
┌──────────────────────┴──────────────────────────────┐
│                   Axum Server (:8080)                │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────┐   │
│  │ JWT Auth │ │ REST API │ │ WebSocket Handler │   │
│  └──────────┘ └────┬─────┘ └───────────────────┘   │
│                    │                                 │
│  ┌─────────────────┴────────────────────────────┐   │
│  │         Service Layer (routes/)              │   │
│  │  Documents, Search, Teams, SSG, Plugins...   │   │
│  └─────────────────┬────────────────────────────┘   │
│                    │                                 │
│  ┌────────┐ ┌─────┴──────┐ ┌──────────┐            │
│  │ Tantivy│ │ PostgreSQL │ │ Redis    │            │
│  │ Search │ │ (sqlx)     │ │ (cache)  │            │
│  └────────┘ └────────────┘ └──────────┘            │
└─────────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
tachyon-server
├── tachyon-database → sqlx → PostgreSQL
├── tachyon-core (shared types)
├── tachyon-search → tantivy
├── tachyon-ssg → tachyon-renderer
├── tachyon-rbac (role-based access control)
└── tachyon-auth → jsonwebtoken

tachyon-frontend (WASM)
├── leptos 0.8 + leptos_router
├── tachyon-editor → ropey + yrs
├── gloo-net (HTTP/WebSocket)
└── web-sys
```

## Key Design Decisions

### Why CSR (Client-Side Rendering)?

Tachyon uses Leptos in CSR mode because:
1. The editor needs direct DOM access for cursor management
2. Offline-first requires all logic in the browser
3. CRDT sync runs entirely client-side
4. No server-side rendering complexity for a desktop-first app

### Why Yrs (Yjs Rust Port)?

- Battle-tested CRDT implementation (used by Notion, Figma)
- Rust-native (no WASM bridge needed)
- Character-level conflict resolution
- Efficient binary encoding for network sync

### Why Axum?

- Tokio-native async runtime
- Tower middleware ecosystem
- WebSocket support via axum-tungstenite
- Extractor pattern for clean route handlers

### Why PostgreSQL?

- Full-text search with tsvector
- JSONB for flexible metadata
- Row-level security
- Mature tooling (sqlx, migrations)
"#,
        ),
    ]);

    for (name, content) in &files {
        let path = target.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        println!("   Created {}", path.display());
    }

    println!();
    println!("✨ Next steps:");
    println!("   1. Edit docs in: {}", target.display());
    println!(
        "   2. Build site:  tachyon-ssg-cli build --input {} --output site",
        target.display()
    );
    println!(
        "   3. Preview:      tachyon-ssg-cli serve --input {} --port 3000",
        target.display()
    );

    Ok(())
}

/// Build site from input directory. Returns the SiteConfig used.
fn rebuild_site(
    input: &Path,
    output: &Path,
    config: Option<&Path>,
    base_url: &str,
) -> Result<SiteConfig> {
    let mut site_config = if let Some(config_path) = config {
        read_config(config_path)?
    } else {
        let config_path = input.join("site.toml");
        if config_path.exists() {
            read_config(&config_path)?
        } else {
            SiteConfig::default()
        }
    };
    site_config.base_url = base_url.to_string();

    let docs = collect_documents(input)?;
    if docs.is_empty() {
        println!("   No markdown files found, skipping build");
        return Ok(site_config);
    }

    let generator = SiteGenerator::new(site_config.clone());
    let result = generator.build_to_dir(&docs, output)?;

    println!(
        "   Rebuilt {} pages in {}ms",
        result.pages, result.build_time_ms
    );
    Ok(site_config)
}

/// Serve the site with live rebuild on file changes.
fn cmd_serve(
    input: PathBuf,
    output: PathBuf,
    config: Option<PathBuf>,
    port: u16,
    base_url: &str,
) -> Result<()> {
    println!("🔥 Tachyon SSG — Development Server");
    println!("   Input:  {}", input.display());
    println!("   Output: {}", output.display());
    println!("   URL:    http://localhost:{}", port);
    println!();

    // Initial build
    println!("🔨 Initial build...");
    rebuild_site(&input, &output, config.as_deref(), base_url)?;
    println!();

    // Start HTTP server in background thread
    let output_clone = output.clone();
    let server_handle = std::thread::spawn(move || {
        serve_directory(&output_clone, port);
    });

    // Watch for file changes
    println!("👁  Watching for changes in {}...", input.display());
    println!("   Press Ctrl+C to stop");
    println!();

    use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )?;

    watcher.watch(&input, RecursiveMode::Recursive)?;

    // Debounce: wait for 500ms of no changes before rebuilding
    let debounce = std::time::Duration::from_millis(500);
    let mut last_rebuild = std::time::Instant::now() - debounce;

    while let Ok(_event) = rx.recv() {
        // Drain queued events for debouncing
        while rx.try_recv().is_ok() {}

        let now = std::time::Instant::now();
        if now.duration_since(last_rebuild) < debounce {
            std::thread::sleep(debounce - now.duration_since(last_rebuild));
        }
        last_rebuild = std::time::Instant::now();

        println!("\n🔄 Change detected, rebuilding...");
        match rebuild_site(&input, &output, config.as_deref(), base_url) {
            Ok(_) => println!("   ✅ Site rebuilt — refresh browser"),
            Err(e) => println!("   ❌ Build failed: {}", e),
        }
    }

    drop(server_handle);
    Ok(())
}

/// Simple HTTP file server serving static files from a directory.
fn serve_directory(dir: &Path, port: u16) {
    use std::io::Read;
    use std::net::TcpListener;

    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut buf = [0u8; 4096];
        let n = match stream.read(&mut buf) {
            Ok(0) | Err(_) => continue,
            Ok(n) => n,
        };

        let request = String::from_utf8_lossy(&buf[..n]);
        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/")
            .trim_start_matches('/');

        let file_path = if path.is_empty() || path == "/" {
            dir.join("index.html")
        } else {
            // Prevent path traversal
            let safe = path
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '/');
            if !safe {
                let _ = write_response(&mut stream, 400, "text/plain", b"Bad Request");
                continue;
            }
            dir.join(path)
        };

        // Try exact path, then path + .html, then path/index.html
        let (content_type, body) = if let Ok(data) = fs::read(&file_path) {
            (mime_type(&file_path), data)
        } else if let Ok(data) = fs::read(file_path.with_extension("html")) {
            (b"text/html".to_vec(), data)
        } else if let Ok(data) = fs::read(file_path.join("index.html")) {
            (b"text/html".to_vec(), data)
        } else {
            // Try 404.html
            if let Ok(data) = fs::read(dir.join("404.html")) {
                let _ = write_response(&mut stream, 404, "text/html", &data);
                continue;
            }
            let _ = write_response(&mut stream, 404, "text/plain", b"Not Found");
            continue;
        };

        let _ = write_response(
            &mut stream,
            200,
            unsafe { std::str::from_utf8_unchecked(&content_type) },
            &body,
        );
    }
}

fn mime_type(path: &Path) -> Vec<u8> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => b"text/html".to_vec(),
        Some("css") => b"text/css".to_vec(),
        Some("js") => b"application/javascript".to_vec(),
        Some("json") => b"application/json".to_vec(),
        Some("png") => b"image/png".to_vec(),
        Some("jpg") | Some("jpeg") => b"image/jpeg".to_vec(),
        Some("gif") => b"image/gif".to_vec(),
        Some("svg") => b"image/svg+xml".to_vec(),
        Some("ico") => b"image/x-icon".to_vec(),
        Some("xml") => b"application/xml".to_vec(),
        Some("txt") => b"text/plain".to_vec(),
        Some("wasm") => b"application/wasm".to_vec(),
        Some("woff") | Some("woff2") => b"font/woff2".to_vec(),
        _ => b"application/octet-stream".to_vec(),
    }
}

fn write_response(
    stream: &mut std::net::TcpStream,
    status: u32,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    use std::io::Write;
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Unknown",
    };
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        status_text,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}
