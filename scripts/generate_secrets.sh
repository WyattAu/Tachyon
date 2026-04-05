#!/bin/bash
# Generate Secure Secrets for Tachyon
# Run this script to generate production-ready secrets

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$PROJECT_ROOT/.env.secrets"

echo "=== Tachyon Secret Generator ==="
echo ""

generate_random_string() {
    local length=$1
    openssl rand -base64 "$length" | tr -d '/+=' | head -c "$length"
}

generate_hex() {
    local bytes=$1
    openssl rand -hex "$bytes"
}

generate_uuid() {
    uuidgen | tr '[:upper:]' '[:lower:]'
}

echo "Generating secrets..."

JWT_SECRET=$(generate_random_string 64)
if [ ${#JWT_SECRET} -lt 32 ]; then
    JWT_SECRET="${JWT_SECRET}$(generate_random_string 32)"
fi

API_KEY_PREFIX="tchk"
API_KEY_SECRET=$(generate_hex 32)
API_KEY="${API_KEY_PREFIX}_${API_KEY_SECRET}"

DATABASE_PASSWORD=$(generate_random_string 24)

REDIS_PASSWORD=$(generate_random_string 32)

ENCRYPTION_KEY=$(generate_hex 32)

SESSION_SECRET=$(generate_random_string 64)

CSRF_SECRET=$(generate_random_string 32)

ADMIN_API_KEY="${API_KEY_PREFIX}_admin_$(generate_hex 24)"

echo "Writing secrets to $ENV_FILE..."

cat > "$ENV_FILE" << EOF
# Tachyon Security Secrets
# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# WARNING: Do not commit this file to version control!

# JWT Configuration
TACHYON_JWT_SECRET=${JWT_SECRET}
TACHYON_JWT_EXPIRATION=86400

# API Key Configuration
TACHYON_API_KEY_PREFIX=${API_KEY_PREFIX}
TACHYON_ADMIN_API_KEY=${ADMIN_API_KEY}

# Database Configuration
DATABASE_PASSWORD=${DATABASE_PASSWORD}

# Redis Configuration (for rate limiting/caching)
REDIS_PASSWORD=${REDIS_PASSWORD}

# Encryption
TACHYON_ENCRYPTION_KEY=${ENCRYPTION_KEY}

# Session Management
TACHYON_SESSION_SECRET=${SESSION_SECRET}

# CSRF Protection
TACHYON_CSRF_SECRET=${CSRF_SECRET}

# Environment
TACHYON_ENV=production
EOF

chmod 600 "$ENV_FILE"

echo ""
echo "=== Secrets Generated Successfully ==="
echo ""
echo "Files created:"
echo "  - $ENV_FILE (secrets file)"
echo ""
echo "Generated Secrets:"
echo "  - JWT Secret: ${JWT_SECRET:0:20}..."
echo "  - API Key Prefix: $API_KEY_PREFIX"
echo "  - Admin API Key: ${ADMIN_API_KEY:0:20}..."
echo "  - Database Password: ${DATABASE_PASSWORD:0:10}..."
echo "  - Redis Password: ${REDIS_PASSWORD:0:10}..."
echo "  - Encryption Key: ${ENCRYPTION_KEY:0:16}..."
echo "  - Session Secret: ${SESSION_SECRET:0:20}..."
echo "  - CSRF Secret: ${CSRF_SECRET:0:10}..."
echo ""
echo "IMPORTANT:"
echo "  1. Add .env.secrets to .gitignore"
echo "  2. Store a backup in a secure location (e.g., password manager)"
echo "  3. Use environment-specific secrets for each deployment"
echo "  4. Rotate secrets regularly (use scripts/rotate_secrets.sh)"
echo ""

if [ -f "$PROJECT_ROOT/.gitignore" ]; then
    if ! grep -q ".env.secrets" "$PROJECT_ROOT/.gitignore"; then
        echo ".env.secrets" >> "$PROJECT_ROOT/.gitignore"
        echo "Added .env.secrets to .gitignore"
    fi
fi

echo "To use these secrets:"
echo "  source $ENV_FILE"
echo "  # or"
echo "  export \$(cat $ENV_FILE | xargs)"
