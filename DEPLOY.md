# Tachyon Staging Deployment Guide

This guide walks through deploying Tachyon to the staging server at `wyatt@192.168.1.191`.

## Prerequisites

- SSH access to `wyatt@192.168.1.191`
- Docker and docker-compose installed on server
- Source code synced to `/home/wyatt/tachyon-server/`

## Step 1: SSH to Server

```bash
ssh wyatt@192.168.1.191
```

## Step 2: Pull Latest Code

```bash
cd /home/wyatt/tachyon-server
git pull origin main
```

## Step 3: Configure Environment

```bash
# Copy the staging env template
cp .env.staging .env

# Edit with secure values
nano .env
```

**Required values to set:**
- `POSTGRES_PASSWORD` - Generate with: `openssl rand -base64 32`
- `TACHYON_JWT_SECRETS` - Generate with: `openssl rand -base64 48`
- `SERVER_NAME` - Your domain name
- `TACHYON_BASE_URL` - Your staging URL

## Step 4: Build Docker Image

```bash
docker compose -f docker-compose.staging.yml build
```

## Step 5: Start Services

```bash
# Start all services in detached mode
docker compose -f docker-compose.staging.yml up -d
```

## Step 6: Verify Health

```bash
# Check all services are running
docker compose -f docker-compose.staging.yml ps

# Check server logs
docker compose -f docker-compose.staging.yml logs -f server

# Test health endpoint
curl -k https://localhost/health
```

## Step 7: Set Up TLS with Certbot

### Initial Certificate Setup

```bash
# Stop nginx temporarily
docker compose -f docker-compose.staging.yml stop nginx

# Get initial certificate (standalone mode)
docker run --rm \
  -v certbot-conf:/etc/letsencrypt \
  -v certbot-www:/var/www/certbot \
  -p 80:80 \
  certbot/certbot certonly \
  --webroot \
  --webroot-path=/var/www/certbot \
  --email your-email@example.com \
  --agree-tos \
  --no-eff-email \
  -d your-domain.com

# Start nginx again
docker compose -f docker-compose.staging.yml start nginx
```

### Auto-Renewal

The certbot service runs automatically and renews certificates every 12 hours.

## Step 8: Verify Deployment

```bash
# Test HTTPS access
curl -I https://your-domain.com

# Check certificate
openssl s_client -connect your-domain.com:443 -servername your-domain.com

# View logs
docker compose -f docker-compose.staging.yml logs -f
```

## Troubleshooting

### Check Service Status
```bash
docker compose -f docker-compose.staging.yml ps
docker compose -f docker-compose.staging.yml logs [service-name]
```

### Restart Services
```bash
docker compose -f docker-compose.staging.yml restart
```

### View Server Logs
```bash
docker compose -f docker-compose.staging.yml logs -f server
```

### Database Access
```bash
# Connect to PostgreSQL
docker compose -f docker-compose.staging.yml exec postgres psql -U tachyon -d tachyon
```

### Rebuild After Code Changes
```bash
docker compose -f docker-compose.staging.yml build --no-cache server
docker compose -f docker-compose.staging.yml up -d server
```

## Port Mapping

| Service    | Internal Port | External Port | Notes                    |
|------------|---------------|---------------|--------------------------|
| nginx      | 80/443        | 80/443        | TLS termination          |
| server     | 8080          | 8080 (local)  | Only accessible locally  |
| postgres   | 5432          | 5434 (local)  | Only accessible locally  |

## Volumes

| Volume            | Purpose                    |
|-------------------|----------------------------|
| pgdata            | PostgreSQL data storage     |
| tachyon-content   | User uploaded files         |
| certbot-conf      | Let's Encrypt certificates  |
| certbot-www       | ACME challenge files        |
