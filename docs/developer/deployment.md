# Deployment Guide

This guide covers deploying Tachyon to production environments.

## Overview

Tachyon can be deployed using:
- Docker containers (recommended)
- Binary deployment
- Kubernetes
- Cloud platforms (AWS, GCP, Azure)

## Prerequisites

- PostgreSQL 12+ database
- Domain name (for production)
- TLS certificate (for HTTPS)
- SMTP server (optional, for emails)

## Docker Deployment

### Using Docker Compose

1. Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  tachyon:
    image: tachyon-org/tachyon-server:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://tachyon:${DB_PASSWORD}@postgres:5432/tachyon
      - TACHYON_JWT_SECRET=${JWT_SECRET}
      - TACHYON_HOST=0.0.0.0
      - TACHYON_PORT=8080
    volumes:
      - ./docs:/docs
      - ./data:/data
    depends_on:
      - postgres
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:14-alpine
    environment:
      - POSTGRES_USER=tachyon
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=tachyon
    volumes:
      - postgres-data:/var/lib/postgresql/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U tachyon"]
      interval: 10s
      timeout: 5s
      retries: 5

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - tachyon
    restart: unless-stopped

volumes:
  postgres-data:
```

2. Create `.env` file:

```env
DB_PASSWORD=your-secure-password
JWT_SECRET=your-jwt-secret-min-32-characters
```

3. Deploy:

```bash
docker-compose up -d
```

### Manual Docker Commands

```bash
# Build image
docker build -t tachyon-server .

# Run container
docker run -d \
  --name tachyon \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://... \
  -e TACHYON_JWT_SECRET=... \
  -v /path/to/docs:/docs \
  tachyon-server
```

## Binary Deployment

### Build Binary

```bash
cargo build --release --no-default-features --features "server-mode"
```

### Install Binary

```bash
# Copy to system location
sudo cp target/release/tachyon-server /usr/local/bin/

# Set permissions
sudo chmod +x /usr/local/bin/tachyon-server
```

### Create Systemd Service

Create `/etc/systemd/system/tachyon.service`:

```ini
[Unit]
Description=Tachyon Server
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=tachyon
Group=tachyon
WorkingDirectory=/opt/tachyon

Environment="DATABASE_URL=postgres://tachyon:password@localhost:5432/tachyon"
Environment="TACHYON_JWT_SECRET=your-secret"
Environment="TACHYON_HOST=0.0.0.0"
Environment="TACHYON_PORT=8080"

ExecStart=/usr/local/bin/tachyon-server
ExecReload=/bin/kill -s HUP $MAINPID
Restart=on-failure
RestartSec=5

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/tachyon/data

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable tachyon
sudo systemctl start tachyon
```

## Kubernetes Deployment

### Deployment YAML

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
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: tachyon-secrets
              key: database-url
        - name: TACHYON_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: tachyon-secrets
              key: jwt-secret
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
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
---
apiVersion: v1
kind: Secret
metadata:
  name: tachyon-secrets
type: Opaque
stringData:
  database-url: postgres://tachyon:password@postgres:5432/tachyon
  jwt-secret: your-jwt-secret-min-32-characters
```

### Deploy to Kubernetes

```bash
kubectl apply -f k8s-deployment.yaml
```

## Reverse Proxy Configuration

### Nginx

```nginx
# /etc/nginx/sites-available/tachyon
upstream tachyon {
    server localhost:8080;
    keepalive 64;
}

server {
    listen 80;
    server_name docs.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name docs.example.com;

    ssl_certificate /etc/letsencrypt/live/docs.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/docs.example.com/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;

    client_max_body_size 10M;

    location / {
        proxy_pass http://tachyon;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /ws {
        proxy_pass http://tachyon;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }
}
```

### Caddy

```caddyfile
docs.example.com {
    reverse_proxy localhost:8080 {
        header_up Host {host}
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
    }
    
    reverse_proxy /ws localhost:8080 {
        header_up Host {host}
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
    }
}
```

## TLS Certificates

### Let's Encrypt (Certbot)

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d docs.example.com

# Auto-renewal
sudo certbot renew --dry-run
```

### Manual Certificate

```bash
# Generate self-signed (development only)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Use in config
TACHYON_TLS_ENABLED=true
TACHYON_TLS_CERT_PATH=/path/to/cert.pem
TACHYON_TLS_KEY_PATH=/path/to/key.pem
```

## Database Setup

### PostgreSQL

```bash
# Create database
sudo -u postgres createdb tachyon

# Create user
sudo -u postgres psql -c "CREATE USER tachyon WITH PASSWORD 'password';"

# Grant permissions
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE tachyon TO tachyon;"

# Run migrations
DATABASE_URL=postgres://tachyon:password@localhost/tachyon sqlx migrate run
```

### Database Backups

```bash
# Backup script
#!/bin/bash
BACKUP_DIR="/backups/postgres"
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump -h localhost -U tachyon tachyon | gzip > "$BACKUP_DIR/tachyon_$DATE.sql.gz"

# Keep only last 7 days
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete
```

### Cron job

```bash
# Daily backup at 2 AM
0 2 * * * /opt/tachyon/scripts/backup.sh
```

## Monitoring

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Expected response
{
  "status": "healthy",
  "timestamp": "2026-03-09T12:00:00Z",
  "version": "0.2.0"
}
```

### Prometheus Metrics

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'tachyon'
    static_configs:
      - targets: ['localhost:8080']
```

### Logging

Configure structured logging:

```bash
# JSON logging
RUST_LOG=tachyon=info,tower_http=debug
```

## Scaling

### Horizontal Scaling

1. Use external PostgreSQL
2. Use Redis for rate limiting
3. Use load balancer

```yaml
# docker-compose.scale.yml
services:
  tachyon:
    deploy:
      replicas: 3
```

### Load Balancing

Use Nginx or cloud load balancer:

```nginx
upstream tachyon {
    least_conn;
    server tachyon1:8080;
    server tachyon2:8080;
    server tachyon3:8080;
}
```

## Security Checklist

- [ ] Use strong JWT secret (32+ characters)
- [ ] Enable HTTPS/TLS
- [ ] Configure CORS properly
- [ ] Enable rate limiting
- [ ] Set secure headers
- [ ] Use environment variables for secrets
- [ ] Regular database backups
- [ ] Keep dependencies updated
- [ ] Monitor logs for anomalies
- [ ] Set up alerts for downtime

## Troubleshooting

### Server Won't Start

```bash
# Check logs
docker logs tachyon

# Check database connection
psql $DATABASE_URL

# Check port availability
lsof -i :8080
```

### Database Connection Issues

```bash
# Verify PostgreSQL is running
sudo systemctl status postgresql

# Check connection string
echo $DATABASE_URL

# Test connection
psql $DATABASE_URL -c "SELECT 1"
```

### Performance Issues

```bash
# Check resource usage
docker stats tachyon

# Check database performance
psql $DATABASE_URL -c "SELECT * FROM pg_stat_activity"

# Increase resources
# Edit docker-compose.yml or deployment.yaml
```

## Maintenance

### Updates

```bash
# Pull latest image
docker pull tachyon-org/tachyon-server:latest

# Restart with new image
docker-compose up -d

# Run migrations
docker-compose exec tachyon /app/migrate
```

### Rollback

```bash
# Use specific version
docker pull tachyon-org/tachyon-server:0.2.0
docker-compose up -d
```

## Next Steps

- [Configuration Guide](../user-guide/configuration.md) - Configuration options
- [Architecture](architecture.md) - System architecture
- [API Guide](api.md) - API documentation
