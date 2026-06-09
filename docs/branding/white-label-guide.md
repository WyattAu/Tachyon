# White-Label Branding Guide

**Version:** 1.0
**Effective Date:** 2026-06-09

## Overview

Tachyon supports white-label branding through the `BrandConfig` system. This guide covers environment variable configuration, CSS custom properties, and deployment requirements.

## Configuration

### Environment Variables

All branding configuration is done via environment variables with the `TACHYON_BRAND_` prefix:

| Variable | Default | Description |
|----------|---------|-------------|
| `TACHYON_BRAND_COMPANY_NAME` | `Tachyon` | Company/product name displayed in UI |
| `TACHYON_BRAND_LOGO_URL` | _(none)_ | URL to logo image (SVG recommended, 200x50px) |
| `TACHYON_BRAND_FAVICON_URL` | _(none)_ | URL to favicon (SVG or ICO, 32x32px minimum) |
| `TACHYON_BRAND_PRIMARY_COLOR` | `#3B82F6` | Primary brand color (hex format) |
| `TACHYON_BRAND_SECONDARY_COLOR` | `#10B981` | Secondary brand color (hex format) |
| `TACHYON_BRAND_CUSTOM_CSS` | _(none)_ | Custom CSS injected into page `<head>` |
| `TACHYON_BRAND_CUSTOM_DOMAIN` | _(none)_ | Custom domain for white-label deployments |

### Example Configuration

```bash
# .env.branding
TACHYON_BRAND_COMPANY_NAME="Acme Corp"
TACHYON_BRAND_LOGO_URL="https://cdn.example.com/logo.svg"
TACHYON_BRAND_FAVICON_URL="https://cdn.example.com/favicon.svg"
TACHYON_BRAND_PRIMARY_COLOR="#E11D48"
TACHYON_BRAND_SECONDARY_COLOR="#F59E0B"
TACHYON_BRAND_CUSTOM_DOMAIN="docs.acme.example.com"
```

### Rust Configuration

The `BrandConfig` struct is defined in `tachyon/crates/server/src/config.rs`:

```rust
pub struct BrandConfig {
    pub company_name: String,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub custom_css: Option<String>,
    pub custom_domain: Option<String>,
}
```

Access the brand configuration from server code:

```rust
let brand = &config.brand;
let company_name = &brand.company_name;
let primary_color = &brand.primary_color;
```

## CSS Custom Properties

The following CSS custom properties are available for theming:

### Core Brand Properties

```css
:root {
    --brand-primary: #3B82F6;      /* Primary brand color */
    --brand-secondary: #10B981;    /* Secondary brand color */
    --brand-logo: url('/logo.svg'); /* Logo background image */
}
```

### Derived Color Scale

The design system includes a full primary color scale (50-950) that can be customized:

```css
:root {
    --color-primary-50: #eff6ff;
    --color-primary-100: #dbeafe;
    --color-primary-200: #bfdbfe;
    --color-primary-300: #93c5fd;
    --color-primary-400: #60a5fa;
    --color-primary-500: #3b82f6;   /* Matches --brand-primary */
    --color-primary-600: #2563eb;
    --color-primary-700: #1d4ed8;
    --color-primary-800: #1e40af;
    --color-primary-900: #1e3a8a;
    --color-primary-950: #172554;
}
```

### Semantic Colors

```css
:root {
    --color-success: #10b981;
    --color-warning: #f59e0b;
    --color-error: #ef4444;
    --color-info: #3b82f6;
}
```

## Logo and Favicon Requirements

### Logo

- **Format:** SVG preferred (vector, scalable)
- **Dimensions:** 200x50px recommended for horizontal layout
- **Max Size:** 50KB
- **Background:** Transparent
- **Placement:** Top-left navigation, login page

### Favicon

- **Format:** SVG preferred, ICO fallback
- **Dimensions:** 32x32px minimum (support 16x16, 32x32, 192x192)
- **Max Size:** 10KB
- **Visibility:** Browser tab, bookmarks, mobile home screen

### Dark Mode Support

For dark mode compatibility, provide two versions:

```html
<link rel="icon" href="/favicon-light.svg" media="(prefers-color-scheme: light)">
<link rel="icon" href="/favicon-dark.svg" media="(prefers-color-scheme: dark)">
```

## Custom CSS Injection

The `TACHYON_BRAND_CUSTOM_CSS` environment variable allows injecting arbitrary CSS into the page `<head>`. This is useful for:

- Additional color overrides
- Font changes
- Layout adjustments
- Custom animations

### Example

```bash
TACHYON_BRAND_CUSTOM_CSS="body { font-family: 'Custom Font', sans-serif; } .sidebar { background: #1a1a2e; }"
```

### CSS Injection Point

Custom CSS is injected before the closing `</style>` tag in `tachyon/crates/frontend/index.html`.

## Custom Domain Setup

### DNS Configuration

1. Add a CNAME record pointing to your Tachyon deployment
2. Configure SSL/TLS certificate (Let's Encrypt recommended)
3. Update `TACHYON_BRAND_CUSTOM_DOMAIN` environment variable

### Example DNS Record

```
docs.acme.example.com.  CNAME  tachyon.example.com.
```

### SSL/TLS

For custom domains, ensure:

1. SSL certificate covers the custom domain
2. HSTS headers are configured for the domain
3. CORS origins include the custom domain

### Configuration

```bash
TACHYON_BRAND_CUSTOM_DOMAIN="docs.acme.example.com"
TACHYON_BASE_URL="https://docs.acme.example.com"
TACHYON_SECURITY_CORS_ALLOWED_ORIGINS="https://docs.acme.example.com"
```

## Deployment Patterns

### Single-Tenant (SaaS)

```bash
# Default configuration
TACHYON_BRAND_COMPANY_NAME="Your App"
TACHYON_BRAND_LOGO_URL="/your-logo.svg"
TACHYON_BRAND_PRIMARY_COLOR="#6366F1"
```

### Multi-Tenant (White-Label)

Each tenant gets their own deployment with unique branding:

```bash
# Tenant A
TACHYON_BRAND_COMPANY_NAME="Tenant A"
TACHYON_BRAND_PRIMARY_COLOR="#059669"
TACHYON_BRAND_CUSTOM_DOMAIN="a.example.com"

# Tenant B
TACHYON_BRAND_COMPANY_NAME="Tenant B"
TACHYON_BRAND_PRIMARY_COLOR="#DC2626"
TACHYON_BRAND_CUSTOM_DOMAIN="b.example.com"
```

### Self-Hosted

Users can customize branding via environment variables in their deployment:

```yaml
# docker-compose.yml
services:
  tachyon:
    environment:
      - TACHYON_BRAND_COMPANY_NAME=My Company
      - TACHYON_BRAND_PRIMARY_COLOR=#7C3AED
      - TACHYON_BRAND_LOGO_URL=/logo.svg
    volumes:
      - ./logo.svg:/static/logo.svg:ro
```

## API Reference

### Server Configuration

```rust
// Access brand config
let brand = &config.brand;

// Get company name
let name = brand.company_name.clone();

// Get colors
let primary = brand.primary_color.clone();
let secondary = brand.secondary_color.clone();

// Check for custom domain
if let Some(ref domain) = brand.custom_domain {
    // Configure for custom domain
}
```

### Frontend Integration

The CSS custom properties are automatically available in the frontend:

```css
/* Use brand colors in components */
.primary-button {
    background-color: var(--brand-primary);
}

.secondary-button {
    background-color: var(--brand-secondary);
}
```

## Troubleshooting

### Logo Not Displaying

- Verify `TACHYON_BRAND_LOGO_URL` is accessible
- Check CORS headers on the logo URL
- Ensure the URL uses HTTPS

### Colors Not Applying

- Verify hex color format (e.g., `#3B82F6`)
- Check for CSS specificity issues
- Inspect browser developer tools for conflicts

### Custom CSS Not Loading

- Verify the CSS is valid
- Check for syntax errors
- Ensure the environment variable is properly escaped

### Custom Domain Issues

- Verify DNS records are propagated
- Check SSL certificate validity
- Ensure CORS origins include the custom domain
