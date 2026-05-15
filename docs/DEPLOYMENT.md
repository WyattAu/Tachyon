# Tachyon Production Deployment Guide

This guide covers deploying Tachyon to production environments with security, monitoring, and high availability considerations.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Deployment Options](#deployment-options)
4. [Configuration](#configuration)
5. [Security](#security)
6. [Monitoring](#monitoring)
7. [Backup and Recovery](#backup-and-recovery)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **OS**: Linux (Ubuntu 22.04 LTS recommended)
- **CPU**: 2+ cores
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 20GB minimum, SSD recommended
- **Network**: Stable internet connection for TLS certificates

### Software Requirements

- Docker 24.0+
- Docker Compose 2.20+
- Git 2.30+
- OpenSSL 3.0+

### Domain Requirements

- Registered domain name
- DNS A/AAAA records pointing to server
- Access to DNS management panel

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd tachyon
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env with your production values
nano .env
```

### 3. Deploy

```bash
./scripts/deploy.sh production v1.0.0
```

## Deployment Options

### Option 1: Docker Compose (Recommended)

Best for single-server deployments with built-in monitoring.

```bash
# Production deployment
export COMPOSE_PROJECT_NAME=tachyon-production
export SERVER_PORT=80
export VERSION=1.0.0

docker-compose -f scripts/docker-compose.yml up -d
```

### Option 2: Bare Metal

Best for maximum performance or specialized hardware requirements.

```bash
# Build release binary
cargo build --release --workspace --exclude tachyon-testing

# Copy binary and config
sudo cp target/release/tachyon-server /usr/local/bin/
sudo cp config/production.toml /etc/tachyon/config.toml

# Create systemd service
sudo cp scripts/tachyon.service /etc/systemd/system/
sudo systemctl enable tachyon
sudo systemctl start tachyon
```

### Option 3: Kubernetes

Best for high-availability, multi-node deployments.

See `scripts/k8s/` directory for Kubernetes manifests.

```bash
kubectl apply -f scripts/k8s/namespace.yaml
kubectl apply -f scripts/k8s/configmap.yaml
kubectl apply -f scripts/k8s/deployment.yaml
kubectl apply -f scripts/k8s/service.yaml
kubectl apply -f scripts/k8s/ingress.yaml
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (error, warn, info, debug, trace) |
| `DATABASE_PATH` | `/data/tachyon.db` | SQLite database file path |
| `SERVER_HOST` | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | `8080` | Server port |
| `JWT_SECRET` | Required | Secret key for JWT tokens |
| `JWT_EXPIRATION` | `86400` | JWT token expiration in seconds |
| `MAX_UPLOAD_SIZE` | `10485760` | Max file upload size in bytes (10MB) |

### Production Configuration File

Create `config/production.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4
request_timeout = 30

[database]
path = "/data/tachyon.db"
max_connections = 100
connection_timeout = 30

[security]
jwt_secret = "${JWT_SECRET}"
jwt_expiration = 86400
password_hash_cost = 12

[search]
index_path = "/data/search"
max_results = 1000
highlight_fragments = 3

[renderer]
cache_size = 1000
enable_math = true
enable_syntax_highlighting = true

[logging]
level = "info"
format = "json"
output = "stdout"
```

## Security

### TLS/SSL Configuration

#### Using Let's Encrypt

```bash
# Install certbot
docker run -it --rm \
  -v "$(pwd)/ssl:/etc/letsencrypt" \
  -v "$(pwd)/certbot-data:/var/lib/letsencrypt" \
  certbot/certbot certonly \
  --standalone \
  -d your-domain.com \
  --agree-tos \
  --email your-email@example.com
```

#### Using Custom Certificates

```bash
# Place certificates in ssl/
cp your-cert.pem ssl/cert.pem
cp your-key.pem ssl/key.pem
```

### Security Best Practices

1. **Change Default Secrets**
   ```bash
   # Generate strong JWT secret
   openssl rand -base64 64
   ```

2. **Enable Firewall**
   ```bash
   sudo ufw allow 80/tcp
   sudo ufw allow 443/tcp
   sudo ufw enable
   ```

3. **Regular Security Updates**
   ```bash
   # Set up automated security updates
   sudo apt install unattended-upgrades
   sudo dpkg-reconfigure -plow unattended-upgrades
   ```

4. **Monitor Security Advisories**
   ```bash
   # Run security audit
   cargo audit
   ```

### RBAC Security Considerations

[WARN] **Important**: The RBAC module has 11 failing tests related to business logic. Review these before using in production:

- Policy precedence rules
- Permission matching logic
- Cache expiration
- Authorization decision logic

## Monitoring

### Prometheus Metrics

Access Prometheus at `http://your-domain:9090`

Key metrics:
- `tachyon_requests_total` - Total HTTP requests
- `tachyon_request_duration_seconds` - Request latency
- `tachyon_active_connections` - Active WebSocket connections
- `tachyon_database_query_duration_seconds` - Database query latency

### Grafana Dashboards

Access Grafana at `http://your-domain:3000`

Default credentials:
- Username: admin
- Password: (set in .env file)

### Health Checks

```bash
# Server health
curl http://localhost:8080/health

# Detailed health
curl http://localhost:8080/health/detailed
```

### Log Aggregation

Logs are aggregated using Loki and viewable in Grafana.

Key log locations:
- Application: `/app/logs/tachyon.log`
- Nginx: `/var/log/nginx/access.log`
- System: `journalctl -u tachyon`

## Backup and Recovery

### Database Backup

```bash
# Automated daily backup
0 2 * * * /app/scripts/backup.sh

# Manual backup
sqlite3 /data/tachyon.db ".backup '/backup/tachyon-$(date +%Y%m%d).db'"
```

### Configuration Backup

```bash
# Backup configuration
tar -czf config-backup-$(date +%Y%m%d).tar.gz config/
```

### Disaster Recovery

1. **Stop services**
   ```bash
   docker-compose -f scripts/docker-compose.yml down
   ```

2. **Restore database**
   ```bash
   cp backup/tachyon-YYYYMMDD.db /data/tachyon.db
   ```

3. **Restart services**
   ```bash
   docker-compose -f scripts/docker-compose.yml up -d
   ```

## Troubleshooting

### Common Issues

#### 1. Port Already in Use

```bash
# Find process using port
sudo lsof -i :8080

# Kill process
sudo kill -9 <PID>
```

#### 2. Permission Denied

```bash
# Fix data directory permissions
sudo chown -R 1000:1000 /data/tachyon
```

#### 3. Database Locked

```bash
# Check for stuck connections
lsof /data/tachyon.db

# Restart services
docker-compose -f scripts/docker-compose.yml restart
```

#### 4. High Memory Usage

```bash
# Monitor memory
docker stats tachyon-server

# Adjust memory limits in docker-compose.yml
```

### Log Analysis

```bash
# View recent logs
docker logs -f tachyon-server

# Search for errors
docker logs tachyon-server 2>&1 | grep ERROR

# Filter by time
docker logs --since 10m tachyon-server
```

### Performance Tuning

1. **Database Optimization**
   ```sql
   PRAGMA journal_mode = WAL;
   PRAGMA synchronous = NORMAL;
   PRAGMA cache_size = -64000; -- 64MB
   ```

2. **Connection Pooling**
   ```toml
   [database]
   max_connections = 100
   min_connections = 10
   ```

3. **Caching**
   ```toml
   [renderer]
   cache_size = 10000
   ```

## Maintenance

### Regular Tasks

- **Daily**: Check health endpoints
- **Weekly**: Review security advisories
- **Monthly**: Update dependencies
- **Quarterly**: Performance review

### Update Process

```bash
# 1. Pull latest code
git pull origin main

# 2. Run tests
cargo test --workspace --lib --exclude tachyon-testing

# 3. Deploy
./scripts/deploy.sh production v1.1.0

# 4. Verify
curl http://localhost/health
```

## Support

- **Documentation**: https://wyattau.github.io/Tachyon
- **Issues**: https://github.com/WyattAu/Tachyon/issues
- **Security**: https://github.com/WyattAu/Tachyon/security

## License

MIT OR Apache-2.0
