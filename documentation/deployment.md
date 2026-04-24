---
title: Deployment
description: Production deployment guide for Tachyon
order: 4
tags: [guide, deployment]
---

# Deployment

## Docker

```bash
# Build
docker build -t tachyon .

# Run
docker run -d \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@db:5432/tachyon \
  -e JWT_SECRET=your-secret \
  tachyon
```

### Docker Compose

```yaml
version: '3.8'
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_DB: tachyon
      POSTGRES_USER: tachyon
      POSTGRES_PASSWORD: password
    volumes:
      - pgdata:/var/lib/postgresql/data

  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://tachyon:password@db:5432/tachyon
      JWT_SECRET: your-secret
    depends_on:
      - db

volumes:
  pgdata:
```

## Static Site (GitHub Pages)

Tachyon can generate a static documentation site:

```bash
# Build the SSG CLI
cargo build --release -p tachyon-ssg

# Generate static site
./target/release/tachyon-ssg-cli build \
  --input ./docs \
  --output ./site \
  --base-url "https://username.github.io/Tachyon"
```

### GitHub Actions

The project includes a `.github/workflows/docs.yml` workflow that automatically builds and deploys documentation to GitHub Pages on push to `main`.

## Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name docs.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## TLS

Use Let's Encrypt with Certbot:

```bash
sudo certbot --nginx -d docs.example.com
```
