#!/bin/bash
# Secret Rotation Procedure for Tachyon
# This script helps rotate secrets with minimal downtime

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$PROJECT_ROOT/.env.secrets"
BACKUP_DIR="$PROJECT_ROOT/.secrets_backup"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

generate_random_string() {
    local length=$1
    openssl rand -base64 "$length" | tr -d '/+=' | head -c "$length"
}

generate_hex() {
    local bytes=$1
    openssl rand -hex "$bytes"
}

backup_current_secrets() {
    if [ -f "$ENV_FILE" ]; then
        mkdir -p "$BACKUP_DIR"
        local timestamp=$(date +%Y%m%d_%H%M%S)
        local backup_file="$BACKUP_DIR/.env.secrets.$timestamp"
        cp "$ENV_FILE" "$backup_file"
        chmod 600 "$backup_file"
        log_info "Backed up current secrets to $backup_file"
    fi
}

rotate_jwt_secret() {
    local new_secret=$(generate_random_string 64)
    if [ ${#new_secret} -lt 32 ]; then
        new_secret="${new_secret}$(generate_random_string 32)"
    fi
    
    if [ -f "$ENV_FILE" ]; then
        if grep -q "TACHYON_JWT_SECRET=" "$ENV_FILE"; then
            sed -i "s|^TACHYON_JWT_SECRET=.*|TACHYON_JWT_SECRET=${new_secret}|" "$ENV_FILE"
        else
            echo "TACHYON_JWT_SECRET=${new_secret}" >> "$ENV_FILE"
        fi
    fi
    
    echo "$new_secret"
}

rotate_api_key() {
    local new_key="tchk_$(generate_hex 32)"
    
    if [ -f "$ENV_FILE" ]; then
        if grep -q "TACHYON_ADMIN_API_KEY=" "$ENV_FILE"; then
            sed -i "s|^TACHYON_ADMIN_API_KEY=.*|TACHYON_ADMIN_API_KEY=${new_key}|" "$ENV_FILE"
        else
            echo "TACHYON_ADMIN_API_KEY=${new_key}" >> "$ENV_FILE"
        fi
    fi
    
    echo "$new_key"
}

rotate_database_password() {
    local new_password=$(generate_random_string 24)
    
    if [ -f "$ENV_FILE" ]; then
        if grep -q "DATABASE_PASSWORD=" "$ENV_FILE"; then
            sed -i "s|^DATABASE_PASSWORD=.*|DATABASE_PASSWORD=${new_password}|" "$ENV_FILE"
        else
            echo "DATABASE_PASSWORD=${new_password}" >> "$ENV_FILE"
        fi
    fi
    
    echo "$new_password"
}

rotate_session_secret() {
    local new_secret=$(generate_random_string 64)
    
    if [ -f "$ENV_FILE" ]; then
        if grep -q "TACHYON_SESSION_SECRET=" "$ENV_FILE"; then
            sed -i "s|^TACHYON_SESSION_SECRET=.*|TACHYON_SESSION_SECRET=${new_secret}|" "$ENV_FILE"
        else
            echo "TACHYON_SESSION_SECRET=${new_secret}" >> "$ENV_FILE"
        fi
    fi
    
    echo "$new_secret"
}

rotate_encryption_key() {
    local new_key=$(generate_hex 32)
    
    if [ -f "$ENV_FILE" ]; then
        if grep -q "TACHYON_ENCRYPTION_KEY=" "$ENV_FILE"; then
            sed -i "s|^TACHYON_ENCRYPTION_KEY=.*|TACHYON_ENCRYPTION_KEY=${new_key}|" "$ENV_FILE"
        else
            echo "TACHYON_ENCRYPTION_KEY=${new_key}" >> "$ENV_FILE"
        fi
    fi
    
    echo "$new_key"
}

rotate_all_secrets() {
    log_info "Rotating all secrets..."
    
    backup_current_secrets
    
    local jwt=$(rotate_jwt_secret)
    local api_key=$(rotate_api_key)
    local db_pass=$(rotate_database_password)
    local session=$(rotate_session_secret)
    local encryption=$(rotate_encryption_key)
    
    echo ""
    log_info "All secrets rotated successfully!"
    echo ""
    echo "New secrets:"
    echo "  - JWT Secret: ${jwt:0:20}..."
    echo "  - Admin API Key: ${api_key:0:20}..."
    echo "  - Database Password: ${db_pass:0:10}..."
    echo "  - Session Secret: ${session:0:20}..."
    echo "  - Encryption Key: ${encryption:0:16}..."
}

show_menu() {
    echo "=== Tachyon Secret Rotation ==="
    echo ""
    echo "Select secret to rotate:"
    echo "  1) JWT Secret"
    echo "  2) Admin API Key"
    echo "  3) Database Password"
    echo "  4) Session Secret"
    echo "  5) Encryption Key"
    echo "  6) Rotate ALL secrets"
    echo "  7) Exit"
    echo ""
    read -p "Enter choice [1-7]: " choice
}

case "${1:-}" in
    --all)
        rotate_all_secrets
        ;;
    --jwt)
        backup_current_secrets
        new_secret=$(rotate_jwt_secret)
        log_info "JWT Secret rotated: ${new_secret:0:20}..."
        ;;
    --api-key)
        backup_current_secrets
        new_key=$(rotate_api_key)
        log_info "Admin API Key rotated: ${new_key:0:20}..."
        ;;
    --database)
        backup_current_secrets
        new_pass=$(rotate_database_password)
        log_info "Database Password rotated: ${new_pass:0:10}..."
        log_warn "You must also update the database user password!"
        ;;
    --session)
        backup_current_secrets
        new_secret=$(rotate_session_secret)
        log_info "Session Secret rotated: ${new_secret:0:20}..."
        ;;
    --encryption)
        backup_current_secrets
        new_key=$(rotate_encryption_key)
        log_info "Encryption Key rotated: ${new_key:0:16}..."
        log_warn "Data encrypted with old key must be re-encrypted!"
        ;;
    --help|-h)
        echo "Usage: $0 [OPTION]"
        echo ""
        echo "Options:"
        echo "  --all         Rotate all secrets"
        echo "  --jwt         Rotate JWT secret only"
        echo "  --api-key     Rotate admin API key only"
        echo "  --database    Rotate database password only"
        echo "  --session     Rotate session secret only"
        echo "  --encryption  Rotate encryption key only"
        echo "  --help, -h    Show this help message"
        echo ""
        echo "Run without arguments for interactive mode."
        ;;
    *)
        while true; do
            show_menu
            case $choice in
                1)
                    backup_current_secrets
                    new_secret=$(rotate_jwt_secret)
                    log_info "JWT Secret rotated: ${new_secret:0:20}..."
                    log_warn "All existing sessions will be invalidated!"
                    ;;
                2)
                    backup_current_secrets
                    new_key=$(rotate_api_key)
                    log_info "Admin API Key rotated: ${new_key:0:20}..."
                    ;;
                3)
                    backup_current_secrets
                    new_pass=$(rotate_database_password)
                    log_info "Database Password rotated: ${new_pass:0:10}..."
                    log_warn "You must also update the database user password!"
                    echo "Run: ALTER USER tachyon WITH PASSWORD 'new_password';"
                    ;;
                4)
                    backup_current_secrets
                    new_secret=$(rotate_session_secret)
                    log_info "Session Secret rotated: ${new_secret:0:20}..."
                    log_warn "All existing sessions will be invalidated!"
                    ;;
                5)
                    backup_current_secrets
                    new_key=$(rotate_encryption_key)
                    log_info "Encryption Key rotated: ${new_key:0:16}..."
                    log_warn "Data encrypted with old key must be re-encrypted!"
                    ;;
                6)
                    rotate_all_secrets
                    ;;
                7)
                    log_info "Exiting..."
                    exit 0
                    ;;
                *)
                    log_error "Invalid choice. Please enter 1-7."
                    ;;
            esac
            echo ""
            read -p "Press Enter to continue..."
        done
        ;;
esac

echo ""
log_info "Secret rotation complete."
echo ""
log_warn "IMPORTANT: After rotating secrets:"
echo "  1. Restart the application to load new secrets"
echo "  2. Invalidate any cached sessions if JWT/Session secrets changed"
echo "  3. Update any external systems with new credentials"
echo "  4. Verify the application starts correctly"
echo "  5. Monitor logs for authentication errors"
