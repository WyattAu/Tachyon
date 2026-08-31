# Tachyon Docker Deployment

This directory contains Docker configuration for building and deploying Tachyon.

## Quick Start

### Development

```bash
# Start all services (backend, frontend, postgres, redis)
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

Services will be available at:
- Frontend: http://localhost:8080
- Backend API: http://localhost:3000
- PostgreSQL: localhost:5432
- Redis: localhost:6379

### Production

```bash
# Create .env file from example
cp .env.example .env
# Edit .env with your production values

# Deploy production stack
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

## Architecture

### Services

1. **Backend** (`tachyon-server`)
   - Rust/Axum HTTP server
   - REST API and WebSocket support
   - Connects to PostgreSQL and Redis
   - Health check: `/health`

2. **Frontend** (`tachyon-frontend`)
   - Leptos/WASM application
   - Served via nginx
   - Proxies API requests to backend

3. **PostgreSQL**
   - Primary database
   - Persistent storage via Docker volume

4. **Redis** (optional)
   - Caching layer
   - Session storage

### Docker Images

#### Backend Image

Multi-stage build optimized for size:
- Builder stage: Compiles Rust code with musl for static linking
- Runtime stage: Alpine Linux with minimal dependencies
- Includes health check
- Runs as non-root user

Size: ~50-80 MB

#### Frontend Image

Multi-stage build for WASM:
- Builder stage: Compiles Rust to WebAssembly using trunk
- Runtime stage: nginx serving static files
- Includes gzip compression and caching
- Proxies API requests to backend

Size: ~20-40 MB

## Configuration

### Environment Variables

#### Backend

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `info` |
| `TACHYON_DATABASE_URL` | PostgreSQL connection URL | Required |
| `TACHYON_REDIS_URL` | Redis connection URL | Optional |
| `TACHYON_BIND_ADDRESS` | Server bind address | `0.0.0.0:3000` |
| `TACHYON_JWT_SECRET` | JWT signing secret | Required |
| `TACHYON_CORS_ORIGINS` | Allowed CORS origins | Required |

#### Frontend (nginx)

Nginx configuration is in `crates/frontend/nginx.conf` and includes:
- Gzip compression
- Static asset caching
- API proxy to backend
- WebSocket support
- Security headers

### Volumes

- `postgres_data`: PostgreSQL data persistence
- `redis_data`: Redis data persistence
- `./data`: Application data (repositories, etc.)

### Networks

All services run on the `tachyon-network` bridge network for isolation and communication.

## Building Images

### Manual Build

```bash
# Build backend
docker build -f Dockerfile -t tachyon/server:latest .

# Build frontend
docker build -f Dockerfile.frontend -t tachyon/frontend:latest .
```

### Using Justfile

```bash
# Development build
just docker-build

# Production build
just docker-build-prod
```

## Health Checks

All services include health checks:

- **Backend**: HTTP GET `/health` every 30s
- **Frontend**: HTTP GET `/` every 30s
- **PostgreSQL**: `pg_isready` every 10s
- **Redis**: `redis-cli ping` every 10s

## Monitoring

### Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
```

### Resource Usage

```bash
# Live stats
docker stats

# Container details
docker-compose ps
```

## Troubleshooting

### Common Issues

1. **Port already in use**
   ```bash
   # Check what's using the port
   lsof -i :3000
   # Stop conflicting services
   docker-compose down
   ```

2. **Database connection failed**
   ```bash
   # Check PostgreSQL logs
   docker-compose logs postgres
   # Verify database is ready
   docker-compose exec postgres pg_isready
   ```

3. **Frontend not loading**
   ```bash
   # Check nginx logs
   docker-compose logs frontend
   # Verify backend is accessible
   curl http://localhost:3000/health
   ```

### Reset Everything

```bash
# Stop and remove all containers, networks, volumes
docker-compose down -v

# Clean build artifacts
docker-compose build --no-cache

# Start fresh
docker-compose up -d
```

## Production Deployment

### Prerequisites

1. Docker and Docker Compose installed
2. Domain name with DNS configured
3. SSL certificates (recommended: use Traefik or Caddy)
4. Production `.env` file

### Deployment Steps

1. **Prepare Environment**
   ```bash
   cp .env.example .env
   # Edit .env with production values
   ```

2. **Pull Images**
   ```bash
   docker-compose -f docker-compose.yml -f docker-compose.prod.yml pull
   ```

3. **Deploy**
   ```bash
   docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```

4. **Verify**
   ```bash
   # Check all services are running
   docker-compose ps
   
   # Check health
   curl https://your-domain.com/health
   ```

### Scaling

To scale the backend service:

```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale backend=3
```

Note: You'll need a load balancer (e.g., Traefik, nginx) in front for proper distribution.

### Updates

```bash
# Pull latest images
docker-compose -f docker-compose.yml -f docker-compose.prod.yml pull

# Recreate containers with new images
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Backup

```bash
# Backup PostgreSQL
docker-compose exec postgres pg_dump -U tachyon tachyon > backup.sql

# Backup volumes
docker run --rm -v tachyon_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres-backup.tar.gz /data
```

## Security Considerations

1. **Change default passwords** in `.env`
2. **Use secrets management** for production (Docker secrets, HashiCorp Vault)
3. **Enable TLS** with reverse proxy
4. **Restrict network access** with firewall rules
5. **Keep images updated** for security patches
6. **Review security scan** results in CI/CD

## CI/CD Integration

The GitHub Actions workflows automatically:

1. **CI Pipeline** (`ci-new.yml`)
   - Runs on every push/PR
   - Builds and tests code
   - Builds Docker images
   - Runs security scans

2. **CD Pipeline** (`cd-new.yml`)
   - Runs on release tags
   - Builds and pushes images to registry
   - Deploys to staging/production

3. **Security Pipeline** (`security-new.yml`)
   - Dependency audits
   - Container scanning
   - SAST analysis
   - Secret detection

See `.github/workflows/` for details.
