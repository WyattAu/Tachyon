# Installation Guide

## System Requirements

### Desktop Application

| Platform | Minimum Requirements |
|----------|---------------------|
| **Windows** | Windows 10 (Build 1903+) or Windows 11 |
| **macOS** | macOS 11 (Big Sur) or later (Intel/Apple Silicon) |
| **Linux** | Kernel ≥ 5.4, GTK3 required |

**Hardware:**
- RAM: 4GB minimum, 8GB recommended
- Disk: 500MB for application
- Display: 1024x768 minimum resolution

### Server Mode

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 2GB | 4GB+ |
| CPU | 2 cores | 4+ cores |
| Disk | 1GB per 1000 documents | SSD recommended |
| Network | 100Mbps | 1Gbps |

## Desktop Installation

### Windows

1. Download `tachyon_setup_x64.exe` from the releases page
2. Run the installer
3. Accept the license agreement
4. Choose installation directory (default: `C:\Program Files\Tachyon`)
5. Select additional components:
   - Desktop shortcut
   - File associations (`.md`, `.markdown`)
   - Context menu integration
6. Complete installation

### macOS

1. Download `Tachyon.dmg` from the releases page
2. Open the DMG file
3. Drag Tachyon to Applications folder
4. On first launch, right-click and select "Open" to bypass Gatekeeper
5. Approve the security prompt

### Linux

**DEB Package (Ubuntu/Debian):**
```bash
sudo dpkg -i tachyon_amd64.deb
sudo apt-get install -f  # Install dependencies
```

**AppImage:**
```bash
chmod +x tachyon-x86_64.AppImage
./tachyon-x86_64.AppImage
```

**Arch Linux (AUR):**
```bash
yay -S tachyon
```

## Server Installation

### Docker (Recommended)

```bash
docker pull tachyon-org/tachyon-server:latest
docker run -d \
  --name tachyon-server \
  -p 8080:8080 \
  -v /path/to/docs:/docs \
  -v /path/to/config:/config \
  tachyon-org/tachyon-server:latest
```

### Docker Compose

```yaml
version: '3.8'
services:
  tachyon:
    image: tachyon-org/tachyon-server:latest
    ports:
      - "8080:8080"
    volumes:
      - ./docs:/docs
      - ./tachyon.toml:/config/tachyon.toml
    environment:
      - TACHYON_MODE=server
      - RUST_LOG=info
```

### Native Binary

```bash
# Download the binary for your platform
curl -LO https://github.com/tachyon-org/tachyon/releases/latest/download/tachyon-server-linux-x86_64.tar.gz
tar xzf tachyon-server-linux-x86_64.tar.gz
sudo mv tachyon /usr/local/bin/

# Create configuration
mkdir -p /etc/tachyon
cat > /etc/tachyon/tachyon.toml << EOF
[system]
mode = "server"

[server]
host = "0.0.0.0"
port = 8080
EOF

# Run
tachyon serve --config /etc/tachyon/tachyon.toml
```

### Build from Source

```bash
git clone https://github.com/tachyon-org/tachyon.git
cd tachyon
cargo build --release --no-default-features --features "server-mode"
```

The binary will be at `target/release/tachyon`.

## Verification

### Desktop

Launch Tachyon and verify:
- Application opens without errors
- File > Open works with a markdown file
- Real-time preview updates on save

### Server

```bash
# Check server is running
curl http://localhost:8080/api/v1/health

# Expected response
{"status":"healthy","version":"0.2.0"}
```

## Configuration

Create a `tachyon.toml` in your project root:

```toml
[system]
mode = "hybrid"          # desktop | server | static
watch_interval_ms = 100  # File system polling rate

[server]
host = "0.0.0.0"
port = 8080
auth_provider = "kanidm"
enable_sso = true

[rendering]
math_engine = "katex"
syntax_theme = "axiom-dark"
enable_diagrams = true

[security]
exclude = [".env", "*.secret.md", "private/"]
```

## Upgrading

### Desktop

Download and install the new version. Settings and recent files are preserved.

### Server (Docker)

```bash
docker pull tachyon-org/tachyon-server:latest
docker-compose up -d
```

### Server (Native)

Replace the binary and restart the service.

## Uninstallation

### Windows

Use "Add or Remove Programs" or run the uninstaller from the installation directory.

### macOS

Remove from Applications folder:
```bash
rm -rf /Applications/Tachyon.app
rm -rf ~/Library/Application\ Support/Tachyon
```

### Linux

**DEB:**
```bash
sudo apt-get remove tachyon
```

**AppImage:**
```bash
rm tachyon-x86_64.AppImage
```

## Next Steps

- [Quick Start Tutorial](quick-start.md)
- [Configuration Guide](configuration_guide.md)
