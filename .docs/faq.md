# Tachyon FAQ

**Document ID:** TACHYON-FAQ-V1.0
**Date:** 2026-02-11
**Version:** 0.2.0-beta
**Status:** Released
**Accessibility:** WCAG 2.1 AA Compliant

---

## Table of Contents

1. [General Questions](#1-general-questions)
2. [Installation](#2-installation)
3. [Configuration](#3-configuration)
4. [Desktop Mode](#4-desktop-mode)
5. [Server Mode](#5-server-mode)
6. [Static Export Mode](#6-static-export-mode)
7. [Authoring](#7-authoring)
8. [Search and Navigation](#8-search-and-navigation)
9. [Performance](#9-performance)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. General Questions

### 1.1. What is Tachyon?

Tachyon is a high-performance knowledge management platform that eliminates build step latency inherent in traditional documentation systems. It provides sub-15ms rendering through Just-In-Time compilation, real-time file watching, and Git-based version control.

### 1.2. What are the system requirements?

**Desktop Mode:**
- Windows 10+ or macOS 11+ or Linux Kernel 5.4+
- 4 GB RAM minimum, 8 GB recommended
- 500 MB free disk space

**Server Mode:**
- Same OS requirements as Desktop
- 4 GB RAM minimum
- Network access for authentication
- SQLite database

**Static Export Mode:**
- Same OS requirements as Desktop
- 2 GB RAM minimum
- Rust toolchain required

### 1.3. Is Tachyon free?

Yes, Tachyon is open-source software licensed under the Apache License 2.0. You can use it freely for personal and commercial purposes.

### 1.4. What programming languages are supported?

Tachyon is written in Rust for the core engine. The web interface uses TypeScript and JavaScript. The desktop application uses Tauri (Rust + WebView).

### 1.5. What databases are supported?

Tachyon uses SQLite (via rusqlite) for session management in Server Mode. Content is stored in Git repositories.

### 1.6. Does Tachyon require internet?

**Desktop Mode:** No internet connection required after installation. Updates can be installed manually.

**Server Mode:** Internet access required for initial setup and authentication. Documentation can be cached locally.

**Static Export Mode:** No internet required for generating static sites.

---

## 2. Installation

### 2.1. How do I install Tachyon on Windows?

Download the installer from the official repository and double-click to run the installation wizard. Follow the prompts to complete installation.

### 2.2. How do I install Tachyon on macOS?

Download the `.dmg` file and drag Tachyon to your Applications folder. On first launch, macOS may prompt for disk and network access permissions.

### 2.3. How do I install Tachyon on Linux?

Download the `.deb` package for Debian/Ubuntu and run:

```bash
sudo dpkg -i tachyon_amd64.deb
```

For other distributions, use the `.AppImage` or build from source.

### 2.4. How do I install Tachyon from source?

Ensure you have Rust toolchain installed:

```bash
curl --proto '=https' --tlsv1.2.0 https://sh.rustup.rs | sh -sSf -y | rustup-init.sh
source ~/.cargo/env
rustup default stable
```

Then clone and build:

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd tachyon
cargo build --release --no-default-features --features "server-mode"
sudo install target/release/tachyon /usr/local/bin/
```

---

## 3. Configuration

### 3.1. Where do I put the configuration file?

Place `tachyon.toml` in the root of your Git repository. This is the recommended location as Tachyon searches for configuration files in the current working directory first.

### 3.2. How do I change the deployment mode?

Modify the `mode` setting in `tachyon.toml`:

```toml
[system]
mode = "desktop"  # or "server" or "static"
```

### 3.3. How do I change the port number?

Modify the `port` setting in the `[server]` section:

```toml
[server]
port = 9000  # or any other valid port
```

### 3.4. How do I enable authentication?

Configure an authentication provider:

```toml
[server]
auth_provider = "kanidm"  # or "ldap"
kanidm_url = "https://your-auth-server.com"
enable_sso = true
```

### 3.5. How do I change the theme?

Modify the `syntax_theme` setting:

```toml
[rendering]
syntax_theme = "nord"  # or any other built-in theme
```

---

## 4. Desktop Mode

### 4.1. Can I use my own editor?

Yes! Tachyon supports Bring Your Own Editor (BYOE) workflow. You can use VS Code, Neovim, JetBrains IDEs, or any text editor while Tachyon provides real-time preview.

### 4.2. How does auto-save work?

Tachyon automatically commits changes to Git after 2 seconds of inactivity. This prevents data loss during crashes or power failures.

### 4.3. Can I work on the same files simultaneously?

Yes, Tachyon does not enforce exclusive file locking. However, if both you and Tachyon modify the same file simultaneously, the last save wins (Last-Write-Wins).

### 4.4. How do I see the Git history?

In Desktop Mode, Tachyon tracks all commits locally. You can view the history using Git commands or through the file watching interface.

### 4.5. Can I use Tachyon offline?

Yes, Desktop Mode works completely offline. No internet connection is required after initial installation.

---

## 5. Server Mode

### 5.1. What authentication providers are supported?

Tachyon supports:

- **Kanidm:** Recommended for enterprise deployments
- **LDAP:** For Active Directory integration
- **Custom:** For custom authentication providers

### 5.2. How do I configure RBAC?

RBAC is configured through frontmatter in documents:

```yaml
---
title: Document Title
access: restricted
groups: [devops, sysadmin]
---
```

Users must be members of the specified groups to access restricted documents.

### 5.3. How do I enable multi-user collaboration?

Server Mode natively supports multiple concurrent users. Conflict resolution uses Last-Write-Wins strategy. Users are notified of conflicts.

### 5.4. Can I use Tachyon Server Mode with Docker?

Yes, see the installation guide for Docker deployment instructions.

---

## 6. Static Export Mode

### 6.1. How do I generate a static site?

Run the static export command:

```bash
tachyon build --output ./dist
```

### 6.2. Can I deploy to GitHub Pages?

Yes, after generating the static site, use:

```bash
ghp-import -n ./dist
git push origin gh-pages
```

### 6.3. Can I deploy to Cloudflare Pages?

Yes, after generating the static site, use:

```bash
npx wrangler pages publish ./dist
```

---

## 7. Authoring

### 7.1. What file formats are supported?

- **Markdown:** CommonMark compliant (`.md`)
- **Frontmatter:** YAML or TOML
- **Images:** PNG, JPG, SVG, WebP
- **Code:** Syntax highlighting for 200+ languages
- **Mathematics:** LaTeX notation via KaTeX
- **Diagrams:** Mermaid.js

### 7.2. How do I create custom directives?

Use the `:::` syntax for special blocks:

```markdown
::: note
This is a note block.
:::

::: warning
This is a warning block.
:::

::: internal
This content is only visible to authorized users.
:::
```

### 7.3. How do I add mathematical formulas?

Use LaTeX notation with `$$` delimiters for block math:

```markdown
Inline math: $E = mc^2$

Block math:
$$
\int_{0}^{\infty} e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

---

## 8. Search and Navigation

### 8.1. How do I search for documents?

Use the search bar in the main interface. Tachyon uses BM25 (Best Matching 25) algorithm for relevance ranking.

### 8.2. How do I navigate the document tree?

The navigation pane on the left displays your Git repository structure. Click folders to expand/collapse, and click files to open them.

### 8.3. What keyboard shortcuts are available?

See the user guide for a complete list of keyboard shortcuts for navigation, editing, and accessibility.

---

## 9. Performance

### 9.1. What are the performance targets?

Tachyon is designed for sub-15ms rendering from file save to HTML generation. Other targets include:

- Cache hit rate: >80%
- Search query time: <100ms
- File watch latency: <10ms

### 9.2. How can I improve performance?

- Increase cache size for frequently accessed documents
- Use SSD storage for better I/O performance
- Reduce syntax theme complexity

---

## 10. Troubleshooting

### 10.1. File watching is not working

See the troubleshooting guide for detailed solutions to common file watching issues on different platforms.

### 10.2. I cannot log in to Server Mode

Verify your authentication credentials and ensure you are a member of the required groups.

### 10.3. Search is not returning results

Rebuild the search index:

```bash
tachyon rebuild-index
```

### 10.4. I see authentication errors

Check your auth provider configuration and network connectivity to the authentication server.

---

## Getting More Help

**Documentation:**
- [Online Documentation](https://docs.tachyon.org)
- [User Guide](./user_guide.md)
- [API Reference](./api_reference.md)
- [Installation Guide](./installation_guide.md)
- [Configuration Guide](./configuration_guide.md)
- [Troubleshooting Guide](./troubleshooting_guide.md)

**Community:**
- [GitHub Issues](https://github.com/WyattAu/Tachyon/issues)
- [Discord Server](https://discord.gg/tachyon)
- [Matrix Room](https://matrix.to/#/tachyon:matrix.org)

**Professional Support:**
- [Enterprise Support](mailto:enterprise@tachyon.org)
- [Security Report](mailto:security@tachyon.org)

---

**Document Control**

| Version | Date | Author | Changes |
|---------|-------|--------|----------|
| 1.0 | 2026-02-11 | Brand Strategist | Initial FAQ from verified implementation |

---

**Accessibility Statement:** This document is WCAG 2.1 AA compliant with proper heading structure, sufficient color contrast, and keyboard navigation support.
