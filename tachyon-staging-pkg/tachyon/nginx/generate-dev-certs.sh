#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERTS_DIR="${SCRIPT_DIR}/certs"
mkdir -p "${CERTS_DIR}"

KEY_FILE="${CERTS_DIR}/localhost.key"
CERT_FILE="${CERTS_DIR}/localhost.crt"

if [ -f "${KEY_FILE}" ] && [ -f "${CERT_FILE}" ]; then
    echo "Development certificates already exist at:"
    echo "  Key:  ${KEY_FILE}"
    echo "  Cert: ${CERT_FILE}"
    echo "Delete them manually to regenerate."
    exit 0
fi

echo "Generating self-signed development certificates for localhost..."

openssl req -x509 -nodes -days 365 \
    -newkey rsa:2048 \
    -keyout "${KEY_FILE}" \
    -out "${CERT_FILE}" \
    -subj "/C=US/ST=Development/L=Development/O=Tachyon Dev/CN=localhost" \
    -addext "subjectAltName=DNS:localhost,IP:127.0.0.1,IP:::1" \
    2>/dev/null

chmod 600 "${KEY_FILE}"
chmod 644 "${CERT_FILE}"

echo "Development certificates generated:"
echo "  Key:  ${KEY_FILE}"
echo "  Cert: ${CERT_FILE}"
echo ""
echo "WARNING: These certificates are self-signed and only for local development."
echo "Browsers will show security warnings. Do not use in production."
