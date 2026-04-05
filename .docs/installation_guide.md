# Tachyon Installation Guide

**Document ID:** TACHYON-IG-V1.0
**Date:** 2026-02-11
**Version:** 0.2.0-beta
**Status:** Released
**Accessibility:** WCAG 2.1 AA Compliant

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [System Requirements](#2-system-requirements)
3. [Desktop Application Installation](#3-desktop-application-installation)
4. [Server Mode Installation](#4-server-mode-installation)
5. [Docker Deployment](#5-docker-deployment)
6. [Source Installation](#6-source-installation)
7. [Post-Installation Setup](#7-post-installation-setup)
8. [Upgrading](#8-upgrading)
9. [Uninstallation](#9-uninstallation)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Introduction

This guide provides comprehensive installation instructions for Tachyon across all supported platforms and deployment modes.

### 1.1. Installation Modes

Tachyon supports three deployment modes:

| Mode | Description | Use Case |
|-------|-------------|--------|
| Desktop | Native application with GUI | Personal knowledge management |
| Server | Headless web server | Team collaboration and centralized documentation |
| Static Export | CLI tool for static site generation | CI/CD pipelines |

### 1.2. Installation Methods

| Method | Platform | Difficulty |
|---------|-----------|-----------|
| Binary Installer | Windows, macOS, Linux | Easy |
| Docker Container | All platforms | Medium |
| Source Build | All platforms | Advanced |
| Package Manager | Nix | Advanced |

---

## 2. System Requirements

### 2.1. Desktop Mode

| Requirement | Minimum | Recommended |
|-----------|-----------|--------------|
| Operating System | Windows 10+, macOS 11+, Linux Kernel 5.4+ | Windows 11, macOS 13, Ubuntu 22.04 |
| RAM | 4 GB | 8 GB |
| Disk Space | 500 MB free | 2 GB free |
| Network | None required | Internet for updates |

### 2.2. Server Mode

| Requirement | Minimum | Recommended |
|-----------|-----------|--------------|
| Operating System | Same as Desktop | Same as Desktop |
| RAM | 4 GB | 8 GB |
| Disk Space | 500 MB free | 2 GB free |
| Network | Required | Internet for updates |
| Authentication | Kanidm or LDAP | Kanidm or LDAP |
| Database | SQLite | SQLite with bundled rusqlite |

### 2.3. Static Export Mode

| Requirement | Minimum | Recommended |
|-----------|-----------|--------------|
| Operating System | Same as Desktop | Same as Desktop |
| RAM | 2 GB | 4 GB |
| Disk Space | 1 GB free | 2 GB free |
| Network | None required | None |
| Build Tools | Rust toolchain | Rust stable toolchain |

---

## 3. Desktop Application Installation

### 3.1. Windows

#### Download

Download from the official repository:

```
https://github.com/tachyon-org/tachyon/releases/download/v0.2.0/tachyon_setup_x64.exe
```

#### Installation

1. Double-click `tachyon_setup_x64.exe`
2. Follow the installation wizard
3. Select installation directory (default: `C:\Program Files\Tachyon`)
4. Complete installation

#### Verification

After installation, verify:

```bash
tachyon --version
```

Expected output: `tachyon 0.2.0-beta`

### 3.2. macOS

#### Download

Download from the official repository:

```
https://github.com/tachyon-org/tachyon/releases/download/v0.2.0/Tachyon.dmg
```

#### Installation

1. Double-click `Tachyon.dmg`
2. Drag Tachyon to Applications folder
3. Right-click and select "Open"

#### Verification

After installation, verify:

```bash
tachyon --version
```

Expected output: `tachyon 0.2.0-beta`

#### First Launch

On first launch, macOS may prompt for:

1. **Disk Access:** Allow Tachyon to access documents
2. **Network Access:** Allow incoming connections
3. **Accessibility Access:** Allow screen recording

### 3.3. Linux

#### Download

Download from the official repository:

```
https://github.com/tachyon-org/tachyon/releases/download/v0.2.0/tachyon_amd64.deb
```

#### Installation

```bash
sudo dpkg -i tachyon_amd64.deb
```

#### Verification

After installation, verify:

```bash
tachyon --version
```

Expected output: `tachyon 0.2.0-beta`

---

## 4. Server Mode Installation

### 4.1. Build from Source

#### Prerequisites

Install Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2.0 https://sh.rustup.rs | sh -sSf -y | rustup-init.sh
source ~/.cargo/env
rustup default stable
```

Install dependencies:

```bash
sudo apt update
sudo apt install -y git libssl-dev pkg-config
```

#### Build

Clone and build:

```bash
git clone https://github.com/tachyon-org/tachyon.git
cd tachyon
cargo build --release --no-default-features --features "server-mode"
```

#### Install

```bash
sudo install target/release/tachyon /usr/local/bin/
```

### 4.2. Docker Deployment

#### Pull Docker Image

```bash
docker pull tachyon-org/tachyon-server:latest
```

#### Run Container

```bash
docker run -d \
  --name tachyon-server \
  -p 8080:8080 \
  -v $(pwd)/docs:/data \
  tachyon-org/tachyon-server:latest
```

### 4.3. Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  tachyon:
    image: tachyon-org/tachyon-server:latest
    ports:
      - "8080:8080"
    volumes:
      - ./docs:/data
      - ./tachyon.toml:/app/config
    environment:
      - TACHYON_MODE: server
      - TACHYON_PORT: 8080
```

Run:

```bash
docker-compose up -d
```

---

## 5. Source Installation

### 5.1. Nix Installation

#### Prerequisites

Install Nix:

```bash
curl -L https://nixos.org/nix/install | sh -sSf -y | nix
```

#### Flake Installation

Clone the repository and enter the development environment:

```bash
git clone https://github.com/tachyon-org/tachyon.git
cd tachyon
nix develop
```

The first time may take longer to download dependencies.

---

## 6. Post-Installation Setup

### 6.1. First Launch

When you first launch Tachyon, you will be prompted to:

1. **Select Repository:** Choose an existing Git repository or create a new one
2. **Configuration:** Select default settings (theme, editor preferences)
3. **Authentication (Server only):** Configure identity provider

### 6.2. Configuration File

Create `tachyon.toml` in your repository root:

```toml
[system]
mode = "desktop"
watch_interval_ms = 100

[server]
port = 8080
bind = "0.0.0.0"
workers = 4

[rendering]
math_engine = "katex"
syntax_theme = "axiom-dark"
enable_diagrams = true

[search]
max_results = 100
index_batch_size = 100

[cache]
max_entries = 1000
eviction_policy = "lru"
```

### 6.3. Repository Initialization

Initialize a Git repository:

```bash
mkdir -p my-docs
cd my-docs
git init
```

Create initial document:

```bash
cat > README.md << 'EOF'
# My Documentation

Welcome to Tachyon!
