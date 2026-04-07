# Tachyon Version Information

## Current Status
- **Version:** 0.1.0
- **Phase:** Launch Ready
- **Status:** Pre-release — all launch blockers resolved
- **Last Updated:** 2026-04-06

## What Changed (Epoch 1-2)

### Epoch 1: Foundation Hardening (Complete)
- [x] CSS Pipeline — Tailwind Play CDN, dark mode, Inter + JetBrains Mono fonts
- [x] Auth Guards — AuthGuard component wrapping 8 protected routes, session restoration from localStorage
- [x] Version Fix — Workspace version aligned to 0.1.0
- [x] Favicon + Meta — SVG favicon, viewport, theme-color, description meta tags
- [x] API Client Unification — 6 team/role methods added to ApiClient, raw gloo_net calls refactored
- [x] Search Index — update_search_index() called after document create/update
- [x] HTML Sanitization — ammonia v4 sanitization on all HTML output paths
- [x] Swagger UI — Re-enabled via utoipa-swagger-ui 9.x (axum 0.8 compatible)
- [x] Orphaned Components — VersionHistory, TemplateSelector, RoleBadge wired up

### Epoch 2: Performance & SEO (Complete)
- [x] ISR/SSR — Full-page HTML rendering with OG, Twitter Cards, JSON-LD (Article + BreadcrumbList)
- [x] robots.txt — Dynamic generation blocking /api/ and /ws
- [x] sitemap.xml — Dynamic generation from document catalog (published/public docs only)
- [x] SSR /docs/:id — Server-side document rendering with SEO metadata from database
- [x] Cache-Control + ETag — Path-aware middleware (SEO=1hr, docs=5min, API=10s SWR)
- [x] WebSocket heartbeat — Client: exponential backoff reconnection (1s→30s, 10 attempts), message queue. Server: 30s Ping frames
- [x] Axum alignment — Workspace axum 0.7→0.8, tower-http 0.5→0.6 with services feature
- [x] Version alignment — All crates at 0.1.0
- [x] Docker port fix — Container health check now on port 8080

### Epoch 3: Beauty & Rigor (Complete)
- [x] Design system tokens — CSS custom properties for colors, spacing, typography
- [x] Semantic HTML — SSR pages use nav/main/article/footer
- [x] Error pages — 404/500 for both API and SSR routes
- [x] JWT secret management — Production documentation in config and env vars

## Services
- **Backend:** http://localhost:8080
- **Frontend:** http://localhost:8080 (WASM served by backend in dev)
- **API:** http://localhost:8080/api/v1/
- **Swagger:** http://localhost:8080/swagger-ui/
- **WebSocket:** ws://localhost:8080/ws
- **Database:** PostgreSQL @ localhost:5432

## Configuration
```bash
# Server
TACHYON_HOST=0.0.0.0
TACHYON_PORT=8080
DATABASE_URL=postgres://tachyon:tachyon@localhost:5432/tachyon
TACHYON_JWT_SECRET=<32+ character secret>

# Auth
TACHYON_GUEST_LOGIN_ENABLED=false
TACHYON_PUBLIC_NOTES_ENABLED=false

# SEO
TACHYON_SITE_TITLE=Tachyon
TACHYON_SITE_DESCRIPTION=A deterministic knowledge management system
TACHYON_BASE_URL=https://tachyon.dev
```
