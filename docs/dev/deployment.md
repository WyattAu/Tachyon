# Deployment Guide

Guide to deploying Tachyon in production environments.

## Deployment Options

| Option | Use Case |
|--------|----------|
| Docker | Recommended for most deployments |
| Binary | Direct deployment |
| Kubernetes | Scalable cloud deployments |
| Nix | Reproducible deployments |

## Docker Deployment

### Quick Start

```bash
docker pull tachyon-org/tachyon-server:latest
docker run -d \
  --name tachyon \
  -p 8080:8080 \
  -v /data/tachyon:/data \
  tachyon-org/tachyon-server:latest
```

### Docker Compose

```yaml
version: '3.8'

services:
  tachyon:
    image: tachyon-org/tachyon-server:latest
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./data:/data
      - ./config:/config
    environment:
      - TACHYON_CONFIG=/config/tachyon.toml
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Custom Dockerfile

```dockerfile
FROM rust:1.77 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p tachyon-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tachyon-server /usr/local/bin/
EXPOSE 8080
CMD ["tachyon-server"]
```

## Binary Deployment

### Download

```bash
# Linux
curl -LO https://github.com/tachyon-org/tachyon/releases/latest/download/tachyon-server-linux-x86_64.tar.gz
tar xzf tachyon-server-linux-x86_64.tar.gz

# macOS
curl -LO https://github.com/tachyon-org/tachyon/releases/latest/download/tachyon-server-macos-x86_64.tar.gz
tar xzf tachyon-server-macos-x86_64.tar.gz
```

### Systemd Service

```ini
# /etc/systemd/system/tachyon.service
[Unit]
Description=Tachyon Documentation Server
After=network.target

[Service]
Type=simple
User=tachyon
Group=tachyon
WorkingDirectory=/opt/tachyon
ExecStart=/usr/local/bin/tachyon-server --config /etc/tachyon/tachyon.toml
Restart=on-failure
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable tachyon
sudo systemctl start tachyon
```

### Configuration

```toml
# /etc/tachyon/tachyon.toml
[system]
mode = "server"
data_dir = "/var/lib/tachyon"

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
path = "/var/lib/tachyon/tachyon.db"

[auth]
provider = "kanidm"
enable_sso = true

[security]
tls_enabled = true
cert_path = "/etc/tls/cert.pem"
key_path = "/etc/tls/key.pem"
```

## Kubernetes Deployment

### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tachyon
  labels:
    app: tachyon
spec:
  replicas: 3
  selector:
    matchLabels:
      app: tachyon
  template:
    metadata:
      labels:
        app: tachyon
    spec:
      containers:
      - name: tachyon
        image: tachyon-org/tachyon-server:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /config
          readOnly: true
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: tachyon-data
      - name: config
        configMap:
          name: tachyon-config
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: tachyon
spec:
  selector:
    app: tachyon
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: tachyon-config
data:
  tachyon.toml: |
    [system]
    mode = "server"
    
    [server]
    host = "0.0.0.0"
    port = 8080
    
    [database]
    path = "/data/tachyon.db"
```

### Helm Chart

```yaml
# Chart.yaml
apiVersion: v2
name: tachyon
version: 0.1.0
description: Tachyon Documentation Server

# values.yaml
replicaCount: 3

image:
  repository: tachyon-org/tachyon-server
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 80

resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "1Gi"
    cpu: "1000m"

persistence:
  enabled: true
  size: 10Gi
  storageClass: standard
```

## Reverse Proxy

### Nginx

```nginx
upstream tachyon {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name docs.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name docs.example.com;

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;

    client_max_body_size 100M;

    location / {
        proxy_pass http://tachyon;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /ws {
        proxy_pass http://tachyon;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }
}
```

### Caddy

```
docs.example.com {
    reverse_proxy localhost:8080 {
        header_up Host {host}
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
    }
}
```

### Traefik

```yaml
# docker-compose.yml
services:
  tachyon:
    image: tachyon-org/tachyon-server:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.tachyon.rule=Host(`docs.example.com`)"
      - "traefik.http.routers.tachyon.tls.certresolver=letsencrypt"
      - "traefik.http.services.tachyon.loadbalancer.server.port=8080"
```

## TLS Configuration

### Let's Encrypt (Certbot)

```bash
# Install certbot
sudo apt install certbot

# Obtain certificate
sudo certbot certonly --standalone -d docs.example.com

# Auto-renewal
sudo systemctl enable certbot.timer
```

### Self-Signed (Development)

```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

### Configuration

```toml
[security]
tls_enabled = true
cert_path = "/etc/tls/cert.pem"
key_path = "/etc/tls/key.pem"
```

## Authentication Setup

### Kanidm

```toml
[auth]
provider = "kanidm"

[auth.kanidm]
url = "https://kanidm.example.com"
client_id = "tachyon"
client_secret = "${KANIDM_CLIENT_SECRET}"
```

### OAuth (GitHub)

```toml
[auth]
provider = "oauth"

[auth.oauth]
provider = "github"
client_id = "${GITHUB_CLIENT_ID}"
client_secret = "${GITHUB_CLIENT_SECRET}"
callback_url = "https://docs.example.com/auth/callback"
```

### LDAP

```toml
[auth]
provider = "ldap"

[auth.ldap]
url = "ldap://ldap.example.com"
base_dn = "ou=users,dc=example,dc=com"
bind_dn = "cn=admin,dc=example,dc=com"
bind_password = "${LDAP_PASSWORD}"
```

## Monitoring

### Health Check

```bash
curl http://localhost:8080/health
# {"status":"healthy","version":"0.2.0"}
```

### Prometheus Metrics

```toml
[monitoring]
metrics_enabled = true
metrics_port = 9090
```

Metrics endpoint: `http://localhost:9090/metrics`

### Grafana Dashboard

Import the provided dashboard JSON for visualization.

### Log Aggregation

```toml
[logging]
format = "json"
level = "info"
output = "/var/log/tachyon/server.log"
```

Forward to ELK, Loki, or similar.

## Backup and Recovery

### Database Backup

```bash
# Backup
sqlite3 /var/lib/tachyon/tachyon.db ".backup /backup/tachyon-$(date +%Y%m%d).db"

# Restore
cp /backup/tachyon-20240115.db /var/lib/tachyon/tachyon.db
```

### Document Backup

```bash
# Backup document repository
tar czf /backup/docs-$(date +%Y%m%d).tar.gz /var/lib/tachyon/docs/

# Restore
tar xzf /backup/docs-20240115.tar.gz -C /
```

### Automated Backups

```bash
# /etc/cron.daily/tachyon-backup
#!/bin/bash
BACKUP_DIR="/backup/tachyon"
DATE=$(date +%Y%m%d)

mkdir -p $BACKUP_DIR

# Database
sqlite3 /var/lib/tachyon/tachyon.db ".backup $BACKUP_DIR/db-$DATE.db"

# Documents
tar czf $BACKUP_DIR/docs-$DATE.tar.gz -C /var/lib/tachyon docs/

# Keep last 30 days
find $BACKUP_DIR -type f -mtime +30 -delete
```

## Scaling

### Vertical Scaling

Increase resources:
```toml
[server]
workers = 8  # Match CPU cores

[cache]
max_size = 10000
ttl_seconds = 3600
```

### Horizontal Scaling

1. Use shared storage (NFS, S3)
2. Configure load balancer
3. Enable session affinity for WebSocket

```yaml
# Multiple replicas with shared storage
apiVersion: apps/v1
kind: Deployment
spec:
  replicas: 5
  # ... shared volume configuration
```

## Troubleshooting

### Common Issues

**Port in use:**
```bash
lsof -i :8080
kill -9 <PID>
```

**Permission denied:**
```bash
chown -R tachyon:tachyon /var/lib/tachyon
chmod 750 /var/lib/tachyon
```

**Database locked:**
```bash
# Check connections
lsof /var/lib/tachyon/tachyon.db
# Restart service
systemctl restart tachyon
```

### Logs

```bash
# Systemd logs
journalctl -u tachyon -f

# Application logs
tail -f /var/log/tachyon/server.log
```

### Debug Mode

```bash
RUST_LOG=debug tachyon-server --config tachyon.toml
```

## Security Checklist

- [ ] TLS enabled with valid certificate
- [ ] Authentication configured
- [ ] Firewall rules in place
- [ ] Regular backups scheduled
- [ ] Security headers configured
- [ ] Rate limiting enabled
- [ ] Audit logging enabled
- [ ] Secrets stored securely
- [ ] Dependencies up to date
- [ ] File permissions correct
