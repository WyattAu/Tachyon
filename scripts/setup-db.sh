#!/bin/bash
# Tachyon Database Setup Script
# Starts PostgreSQL and verifies the connection

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Tachyon PostgreSQL Setup ==="

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is required but not installed"
    exit 1
fi

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "Error: docker-compose is required but not installed"
    exit 1
fi

# Use docker compose if available, otherwise docker-compose
COMPOSE_CMD="docker compose"
if ! docker compose version &> /dev/null; then
    COMPOSE_CMD="docker-compose"
fi

# Load environment variables
if [ -f "$PROJECT_ROOT/.env.development" ]; then
    echo "Loading environment from .env.development..."
    export $(grep -v '^#' "$PROJECT_ROOT/.env.development" | xargs)
fi

# Set defaults
export POSTGRES_USER="${POSTGRES_USER:-tachyon}"
export POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-tachyon_dev_password}"
export POSTGRES_DB="${POSTGRES_DB:-tachyon}"
export POSTGRES_PORT="${POSTGRES_PORT:-5432}"
export PGADMIN_EMAIL="${PGADMIN_EMAIL:-admin@tachyon.local}"
export PGADMIN_PASSWORD="${PGADMIN_PASSWORD:-admin}"
export PGADMIN_PORT="${PGADMIN_PORT:-5050}"

echo "Configuration:"
echo "  PostgreSQL User: $POSTGRES_USER"
echo "  PostgreSQL Database: $POSTGRES_DB"
echo "  PostgreSQL Port: $POSTGRES_PORT"
echo "  pgAdmin Port: $PGADMIN_PORT"
echo ""

# Start PostgreSQL
echo "Starting PostgreSQL..."
$COMPOSE_CMD -f "$SCRIPT_DIR/docker-compose.dev.yml" up -d postgres

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
max_attempts=30
attempt=0
while ! docker exec tachyon-postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB" &> /dev/null; do
    attempt=$((attempt + 1))
    if [ $attempt -ge $max_attempts ]; then
        echo "Error: PostgreSQL did not become ready in time"
        exit 1
    fi
    echo "  Attempt $attempt/$max_attempts..."
    sleep 1
done

echo "PostgreSQL is ready!"

# Test connection
echo "Testing database connection..."
if docker exec tachyon-postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT 1" &> /dev/null; then
    echo "✅ Database connection successful!"
else
    echo "❌ Database connection failed!"
    exit 1
fi

# Check extensions
echo "Checking installed extensions..."
docker exec tachyon-postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT extname FROM pg_extension;"

echo ""
echo "=== PostgreSQL is running ==="
echo ""
echo "Connection string:"
echo "  postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$POSTGRES_PORT/$POSTGRES_DB"
echo ""
echo "To start pgAdmin, run:"
echo "  $COMPOSE_CMD -f $SCRIPT_DIR/docker-compose.dev.yml up -d pgadmin"
echo "  Then open http://localhost:$PGADMIN_PORT"
echo ""
echo "To stop:"
echo "  $COMPOSE_CMD -f $SCRIPT_DIR/docker-compose.dev.yml down"
