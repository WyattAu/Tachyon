---
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
| `TACHYON_JWT_SECRET` | — | Secret key for JWT signing (required) |
| `RUST_LOG` | `info` | Log level filter |
| `TACHYON_HOST` | `0.0.0.0` | Server bind address |
| `TACHYON_PORT` | `8080` | Server bind port |
| `TACHYON_CORS_ORIGINS` | `*` | Allowed CORS origins |
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
