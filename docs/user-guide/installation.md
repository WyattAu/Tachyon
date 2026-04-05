# Installation Guide

This guide covers installing Tachyon on different platforms and deployment scenarios.

## System Requirements

### Desktop Application

| Platform | Minimum Requirements |
|----------|---------------------|
| Windows | Windows 10 (Build 1903+) |
| macOS | macOS 11 (Big Sur) |
| Linux | Kernel ≥ 5.4, GTK3 |

### Server Mode

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| RAM | 2GB | 4GB+ |
| CPU | 2 cores | 4+ cores |
| Disk | 1GB/1000 docs | SSD |
| Database | PostgreSQL 12+ | PostgreSQL 14+ |

## Desktop Installation

### Windows

1. Download the installer:
   ```powershell
   # Using PowerShell
   Invoke-WebRequest -Uri "https://github.com/tachyon-org/tachyon/releases/latest/download/tachyon_setup_x64.exe" -OutFile "tachyon_setup.exe"
   ```

2. Run the installer:
   ```powershell
   .\tachyon_setup.exe
   ```

3. Launch Tachyon from the Start Menu

### macOS

1. Download the DMG:
   ```bash
   curl -L -o Tachyon.dmg https://github.com/tachyon-org/tachyon/releases/latest/download/Tachyon.dmg
   ```

2. Install:
   ```bash
   open Tachyon.dmg
   # Drag Tachyon to Applications folder
   ```

3. Launch from Applications

### Linux

#### Debian/Ubuntu

```bash
wget https://github.com/tachyon-org/tachyon/releases/latest/download/tachyon_amd64.deb
sudo dpkg -i tachyon_amd64.deb
sudo apt-get install -f  # Install dependencies
```

#### AppImage

```bash
wget https://github.com/tachyon-org/tachyon/releases/latest/download/tachyon-x86_64.AppImage
chmod +x tachyon-x86_64.AppImage
./tachyon-x86_64.AppImage
```

#### Arch Linux (AUR)

```bash
yay -S tachyon
```

## Server Installation

### Docker (Recommended)

1. Pull the image:
   ```bash
   docker pull tachyon-org/tachyon-server:latest
   ```

2. Run with Docker:
   ```bash
   docker run -d \
     --name tachyon-server \
     -p 8080:8080 \
     -v /path/to/docs:/docs \
     -v /path/to/data:/data \
     -e DATABASE_URL=postgres://user:pass@host:5432/tachyon \
     -e TACHYON_JWT_SECRET=your-secret-key-min-32-chars \
     tachyon-org/tachyon-server:latest
   ```

3. Using Docker Compose:
   ```yaml
   version: '3.8'
   services:
     tachyon:
       image: tachyon-org/tachyon-server:latest
       ports:
         - "8080:8080"
       volumes:
         - ./docs:/docs
         - ./data:/data
       environment:
         - DATABASE_URL=postgres://tachyon:tachyon@postgres:5432/tachyon
         - TACHYON_JWT_SECRET=your-secret-key-min-32-characters
       depends_on:
         - postgres
     
     postgres:
       image: postgres:14-alpine
       environment:
         - POSTGRES_USER=tachyon
         - POSTGRES_PASSWORD=tachyon
         - POSTGRES_DB=tachyon
       volumes:
         - postgres-data:/var/lib/postgresql/data
   
   volumes:
     postgres-data:
   ```

### Binary Installation

1. Download the binary:
   ```bash
   curl -L -o tachyon-server https://github.com/tachyon-org/tachyon/releases/latest/download/tachyon-server-linux-x64
   chmod +x tachyon-server
   ```

2. Create a systemd service:
   ```ini
   # /etc/systemd/system/tachyon.service
   [Unit]
   Description=Tachyon Server
   After=network.target postgresql.service

   [Service]
   Type=simple
   User=tachyon
   Group=tachyon
   Environment="DATABASE_URL=postgres://tachyon:password@localhost:5432/tachyon"
   Environment="TACHYON_JWT_SECRET=your-secret-key-min-32-characters"
   ExecStart=/usr/local/bin/tachyon-server
   Restart=on-failure
   RestartSec=5

   [Install]
   WantedBy=multi-user.target
   ```

3. Enable and start:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable tachyon
   sudo systemctl start tachyon
   ```

### Build from Source

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. Clone and build:
   ```bash
   git clone https://github.com/tachyon-org/tachyon.git
   cd tachyon
   cargo build --release --no-default-features --features "server-mode"
   ```

3. The binary will be at `target/release/tachyon-server`

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `TACHYON_HOST` | Server bind address | `0.0.0.0` |
| `TACHYON_PORT` | Server port | `8080` |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `TACHYON_JWT_SECRET` | JWT signing secret (min 32 chars) | Required |
| `TACHYON_JWT_EXPIRATION` | Token expiration in seconds | `86400` |
| `TACHYON_TLS_ENABLED` | Enable TLS | `false` |
| `TACHYON_TLS_CERT_PATH` | TLS certificate path | - |
| `TACHYON_TLS_KEY_PATH` | TLS key path | - |
| `TACHYON_GUEST_LOGIN_ENABLED` | Enable guest login | `false` |
| `TACHYON_PUBLIC_NOTES_ENABLED` | Enable public access | `false` |

### Configuration File

Create `tachyon.toml` in your working directory:

```toml
[server]
host = "0.0.0.0"
port = 8080
database_url = "postgres://tachyon:password@localhost:5432/tachyon"

[jwt]
secret = "your-secret-key-minimum-32-characters-long"
expiration_secs = 86400
issuer = "tachyon-server"
audience = "tachyon-client"

[cors]
enabled = true
allowed_origins = ["*"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]

[websocket]
enabled = true
path = "/ws"
max_connections = 1000

[guest]
guest_login_enabled = false
public_notes_enabled = false
```

## Verification

### Check Installation

```bash
# Desktop
tachyon --version

# Server
curl http://localhost:8080/health
```

### Expected Response

```json
{
  "status": "healthy",
  "timestamp": "2026-03-09T12:00:00Z",
  "version": "0.2.0-beta"
}
```

## Troubleshooting

### Port Already in Use

```bash
# Check what's using port 8080
lsof -i :8080

# Use a different port
TACHYON_PORT=8081 tachyon-server
```

### Database Connection Failed

```bash
# Test PostgreSQL connection
psql postgres://tachyon:password@localhost:5432/tachyon

# Check PostgreSQL is running
sudo systemctl status postgresql
```

### Permission Denied

```bash
# Ensure proper permissions
chmod +x tachyon-server
chown -R tachyon:tachyon /path/to/docs
```

## Next Steps

- [Configuration Guide](configuration.md) - Detailed configuration options
- [Authentication Setup](authentication.md) - Set up authentication
- [Quick Start](../user/quick-start.md) - Get started in 5 minutes
