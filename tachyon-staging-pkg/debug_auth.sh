#!/bin/bash
# Quick diagnostic script for Tachyon auth flow
# Run from the server or any machine that can reach the server

set -e

SERVER="${TACHYON_SERVER:-http://192.168.1.191:8080}"
echo "=== Tachyon Auth Diagnostics ==="
echo "Server: $SERVER"
echo ""

# 1. Test API health
echo "--- 1. API Health ---"
curl -s "$SERVER/api/health" | python3 -m json.tool 2>/dev/null || echo "FAILED"
echo ""

# 2. Test login endpoint
echo "--- 2. Login Endpoint ---"
LOGIN_RESP=$(curl -s -w "\n%{http_code}" -X POST "$SERVER/api/v1/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"admin123"}')
HTTP_CODE=$(echo "$LOGIN_RESP" | tail -1)
BODY=$(echo "$LOGIN_RESP" | head -n -1)
echo "HTTP Status: $HTTP_CODE"
echo "$BODY" | python3 -m json.tool 2>/dev/null || echo "$BODY"
echo ""

# Extract token
TOKEN=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)
if [ -z "$TOKEN" ]; then
  echo "❌ No token received from login"
  exit 1
fi
echo "Token received (length: ${#TOKEN})"
echo ""

# 3. Test authenticated endpoint
echo "--- 3. Authenticated Request ---"
curl -s "$SERVER/api/v1/auth/me" \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool 2>/dev/null || echo "FAILED"
echo ""

# 4. Test SPA route serving (should return index.html)
echo "--- 4. SPA Route Serving ---"
echo "GET /login:"
curl -s -o /dev/null -w "HTTP %{http_code}, Content-Type: %{content_type}, Size: %{size_download}\n" "$SERVER/login"

echo "GET /dashboard:"
curl -s -o /dev/null -w "HTTP %{http_code}, Content-Type: %{content_type}, Size: %{size_download}\n" "$SERVER/dashboard"

echo "GET /documents:"
curl -s -o /dev/null -w "HTTP %{http_code}, Content-Type: %{content_type}, Size: %{size_download}\n" "$SERVER/documents"
echo ""

# 5. Check if auth middleware blocks SPA routes
echo "--- 5. Auth Middleware on SPA Routes ---"
echo "GET /login (no auth header):"
curl -s -o /dev/null -w "HTTP %{http_code}\n" "$SERVER/login"

echo "GET /dashboard (no auth header):"
curl -s -o /dev/null -w "HTTP %{http_code}\n" "$SERVER/dashboard"

echo "GET / (no auth header):"
curl -s -o /dev/null -w "HTTP %{http_code}\n" "$SERVER/"
echo ""

# 6. Test CORS preflight
echo "--- 6. CORS Preflight ---"
curl -s -o /dev/null -w "HTTP %{http_code}\n" -X OPTIONS "$SERVER/api/v1/auth/login" \
  -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type"
echo ""

echo "=== Done ==="
