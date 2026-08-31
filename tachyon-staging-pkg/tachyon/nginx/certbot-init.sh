#!/usr/bin/env bash
set -euo pipefail

DOMAIN="${CERTBOT_DOMAIN:?CERTBOT_DOMAIN is required}"
EMAIL="${CERTBOT_EMAIL:-admin@${DOMAIN}}"
STAGING="${CERTBOT_STAGING:-false}"
CERT_DIR="/etc/letsencrypt/live/${DOMAIN}"

echo "=== Tachyon SSL Certificate Setup ==="
echo "Domain: ${DOMAIN}"
echo "Email:  ${EMAIL}"
echo "Staging: ${STAGING}"

STAGING_FLAG=""
if [ "${STAGING}" = "true" ]; then
    echo "WARNING: Using Let's Encrypt staging server (certificates will not be trusted by browsers)"
    STAGING_FLAG="--staging"
fi

if [ -d "${CERT_DIR}" ] && [ -f "${CERT_DIR}/fullchain.pem" ]; then
    echo "Certificate already exists at ${CERT_DIR}"
    echo "To renew manually, run: docker compose exec certbot certbot renew"
    exit 0
fi

if [ "${STAGING}" != "true" ]; then
    echo ""
    echo "=== Obtaining production certificate ==="
    echo "If this fails with rate-limit errors, set CERTBOT_STAGING=true and retry."
    echo ""
    sleep 3
fi

certbot certonly \
    --webroot \
    --webroot-path="/var/www/certbot" \
    --email "${EMAIL}" \
    --agree-tos \
    --no-eff-email \
    --non-interactive \
    ${STAGING_FLAG} \
    -d "${DOMAIN}" \
    -d "www.${DOMAIN}" 2>/dev/null || \
certbot certonly \
    --webroot \
    --webroot-path="/var/www/certbot" \
    --email "${EMAIL}" \
    --agree-tos \
    --no-eff-email \
    --non-interactive \
    ${STAGING_FLAG} \
    -d "${DOMAIN}"

if [ ! -f "${CERT_DIR}/fullchain.pem" ]; then
    echo "ERROR: Certificate was not issued. Check certbot logs above."
    exit 1
fi

echo ""
echo "=== Certificate obtained successfully ==="
echo "Certificate: ${CERT_DIR}/fullchain.pem"
echo "Private key: ${CERT_DIR}/privkey.pem"
echo ""
echo "=== Setting up automatic renewal cron ==="
echo "0 3 * * * certbot renew --quiet --deploy-hook \"nginx -s reload\"" > /etc/cron.d/certbot-renew
chmod 0644 /etc/cron.d/certbot-renew

echo "Renewal cron installed at /etc/cron.d/certbot-renew"
echo "Certificates will be auto-renewed daily at 3:00 AM UTC"
echo ""
echo "Done. Reload nginx to pick up the new certificates:"
echo "  docker compose exec nginx nginx -s reload"
