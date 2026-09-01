#!/usr/bin/env bash
# Tachyon lightweight security audit
# Tests for OWASP Top 10 issues without external dependencies
set -uo pipefail

BASE_URL="${1:-http://192.168.1.191:8082}"
REPORT_DIR="/opt/tachyon/backups/soak-test"
mkdir -p "${REPORT_DIR}"
REPORT="${REPORT_DIR}/security-audit-$(date +%Y%m%d_%H%M%S).txt"

PASS=0
FAIL=0
WARN=0

check() {
    local name="$1"
    local result="$2"
    local detail="$3"
    
    if [ "${result}" = "PASS" ]; then
        echo "✅ PASS: ${name}" | tee -a "${REPORT}"
        PASS=$((PASS + 1))
    elif [ "${result}" = "WARN" ]; then
        echo "⚠️  WARN: ${name} — ${detail}" | tee -a "${REPORT}"
        WARN=$((WARN + 1))
    else
        echo "❌ FAIL: ${name} — ${detail}" | tee -a "${REPORT}"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== TACHYON SECURITY AUDIT ===" | tee -a "${REPORT}"
echo "Date: $(date -Iseconds)" | tee -a "${REPORT}"
echo "Target: ${BASE_URL}" | tee -a "${REPORT}"
echo "" | tee -a "${REPORT}"

# --- A01: Broken Access Control ---
echo "--- A01: Broken Access Control ---" | tee -a "${REPORT}"

# Test: Unauthenticated access to protected endpoints
for ep in "/api/v1/documents" "/api/v1/analytics/overview" "/api/v1/admin/query-stats" "/api/v1/users" "/api/v1/admin/scim/v2/Users"; do
    CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}${ep}" 2>/dev/null)
    if [ "${CODE}" = "401" ] || [ "${CODE}" = "403" ]; then
        check "Unauth blocked: ${ep}" "PASS" ""
    else
        check "Unauth blocked: ${ep}" "FAIL" "Got HTTP ${CODE}, expected 401/403"
    fi
done

# Test: Horizontal privilege escalation (access other user's docs)
CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/v1/documents/00000000-0000-0000-0000-000000000001" 2>/dev/null)
if [ "${CODE}" = "401" ] || [ "${CODE}" = "403" ] || [ "${CODE}" = "404" ]; then
    check "No horizontal privesc on doc ID" "PASS" ""
else
    check "No horizontal privesc on doc ID" "FAIL" "Got HTTP ${CODE}"
fi

# --- A02: Cryptographic Failures ---
echo "" | tee -a "${REPORT}"
echo "--- A02: Cryptographic Failures ---" | tee -a "${REPORT}"

# Test: HTTP to HTTPS redirect
HTTPS_REDIRECT=$(curl -s -o /dev/null -w "%{http_code}:%{redirect_url}" "http://192.168.1.191:8082/health" 2>/dev/null)
if echo "${HTTPS_REDIRECT}" | grep -q "30[1-8]"; then
    check "HTTP->HTTPS redirect" "PASS" ""
else
    check "HTTP->HTTPS redirect" "WARN" "No redirect (expected for staging; production needs TLS)"
fi

# Test: Security headers
HEADERS=$(curl -sI "${BASE_URL}/health" 2>/dev/null)
for hdr in "strict-transport-security" "x-content-type-options" "x-frame-options" "content-security-policy"; do
    if echo "${HEADERS}" | grep -qi "${hdr}"; then
        check "Header: ${hdr}" "PASS" ""
    else
        check "Header: ${hdr}" "WARN" "Missing"
    fi
done

# --- A03: Injection ---
echo "" | tee -a "${REPORT}"
echo "--- A03: Injection ---" | tee -a "${REPORT}"

# SQL injection attempt on search
SQL_RESP=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/v1/search?q=1'+OR+'1'%3D'1" 2>/dev/null)
if [ "${SQL_RESP}" = "400" ] || [ "${SQL_RESP}" = "401" ] || [ "${SQL_RESP}" = "422" ]; then
    check "SQL injection blocked on search" "PASS" ""
elif [ "${SQL_RESP}" = "200" ]; then
    # Check if the response body is a valid JSON (not SQL error)
    BODY=$(curl -s "${BASE_URL}/api/v1/search?q=1'+OR+'1'%3D'1" 2>/dev/null)
    if echo "${BODY}" | grep -qi "error\|sql\|syntax\|exception"; then
        check "SQL injection blocked on search" "FAIL" "SQL error leaked in response"
    else
        check "SQL injection blocked on search" "PASS" ""
    fi
else
    check "SQL injection blocked on search" "WARN" "Got HTTP ${SQL_RESP}"
fi

# XSS attempt on document creation
XSS_BODY='{"title":"<script>alert(1)</script>","content":"test"}'
XSS_RESP=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -d "${XSS_BODY}" "${BASE_URL}/api/v1/documents" 2>/dev/null)
if [ "${XSS_RESP}" = "401" ] || [ "${XSS_RESP}" = "400" ] || [ "${XSS_RESP}" = "422" ]; then
    check "XSS payload rejected on doc create" "PASS" ""
else
    check "XSS payload rejected on doc create" "WARN" "Got HTTP ${XSS_RESP}"
fi

# --- A04: Insecure Design ---
echo "" | tee -a "${REPORT}"
echo "--- A04: Insecure Design ---" | tee -a "${REPORT}"

# Test: Auth brute-force protection (rate limiting)
echo "Testing rate limit (5 rapid login attempts)..."
for i in 1 2 3 4 5; do
    curl -s -o /dev/null -X POST -H "Content-Type: application/json" \
        -d '{"email":"test@test.com","password":"wrong"}' \
        "${BASE_URL}/api/v1/auth/login" 2>/dev/null
done
# 6th attempt
RATE_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" \
    -d '{"email":"test@test.com","password":"wrong"}' \
    "${BASE_URL}/api/v1/auth/login" 2>/dev/null)
if [ "${RATE_CODE}" = "429" ]; then
    check "Rate limiting on login" "PASS" ""
else
    check "Rate limiting on login" "WARN" "No 429 after 6 attempts (got ${RATE_CODE})"
fi

# --- A05: Security Misconfiguration ---
echo "" | tee -a "${REPORT}"
echo "--- A05: Security Misconfiguration ---" | tee -a "${REPORT}"

# Test: Debug endpoint in production
DEBUG_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/debug/html" 2>/dev/null)
if [ "${DEBUG_CODE}" = "403" ]; then
    check "Debug endpoint blocked in prod" "PASS" ""
elif [ "${DEBUG_CODE}" = "404" ]; then
    check "Debug endpoint blocked in prod" "PASS" "Returns 404"
else
    check "Debug endpoint blocked in prod" "FAIL" "Got HTTP ${DEBUG_CODE}"
fi

# Test: GraphQL introspection disabled in prod
INTRO=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"query":"{ __schema { types { name } } }"}' \
    "${BASE_URL}/graphql" 2>/dev/null)
if echo "${INTRO}" | grep -q "types"; then
    check "GraphQL introspection" "WARN" "Introspection enabled (disable in production)"
else
    check "GraphQL introspection" "PASS" "Disabled or blocked"
fi

# Test: Error messages don't leak stack traces
ERR_BODY=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"query":"invalid"}' \
    "${BASE_URL}/api/v1/auth/login" 2>/dev/null)
if echo "${ERR_BODY}" | grep -qi "stack\|trace\|backtrace\|panic\|thread\|at /"; then
    check "No stack trace in error responses" "FAIL" "Stack trace leaked"
else
    check "No stack trace in error responses" "PASS" ""
fi

# --- A06: Vulnerable and Outdated Components ---
echo "" | tee -a "${REPORT}"
echo "--- A06: Vulnerable Components ---" | tee -a "${REPORT}"
check "Cargo audit" "WARN" "Run 'cargo audit' locally to check for known vulnerabilities"

# --- A07: Identification and Authentication Failures ---
echo "" | tee -a "${REPORT}"
echo "--- A07: Authentication Failures ---" | tee -a "${REPORT}"

# Test: Password complexity enforcement (empty password)
EMPTY_PASS=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" \
    -d '{"email":"test@test.com","password":""}' \
    "${BASE_URL}/api/v1/auth/signup" 2>/dev/null)
if [ "${EMPTY_PASS}" = "400" ] || [ "${EMPTY_PASS}" = "422" ]; then
    check "Empty password rejected" "PASS" ""
else
    check "Empty password rejected" "WARN" "Got HTTP ${EMPTY_PASS}"
fi

# --- A08: Software and Data Integrity ---
echo "" | tee -a "${REPORT}"
echo "--- A08: Data Integrity ---" | tee -a "${REPORT}"

# Check that API returns proper Content-Type
CT=$(curl -sI "${BASE_URL}/health" 2>/dev/null | grep -i "content-type")
if echo "${CT}" | grep -qi "application/json"; then
    check "Content-Type: application/json on /health" "PASS" ""
else
    check "Content-Type: application/json on /health" "WARN" "Got: ${CT}"
fi

# --- A09: Security Logging and Monitoring ---
echo "" | tee -a "${REPORT}"
echo "--- A09: Logging & Monitoring ---" | tee -a "${REPORT}"

# Check /metrics endpoint exists
METRICS_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/metrics/prometheus" 2>/dev/null)
if [ "${METRICS_CODE}" = "200" ]; then
    check "Prometheus metrics endpoint" "PASS" ""
else
    check "Prometheus metrics endpoint" "FAIL" "HTTP ${METRICS_CODE}"
fi

# --- A10: Server-Side Request Forgery ---
echo "" | tee -a "${REPORT}"
echo "--- A10: SSRF ---" | tee -a "${REPORT}"
check "SSRF: requires manual testing with authenticated webhook/file-upload endpoints" "WARN" ""

# --- Summary ---
echo "" | tee -a "${REPORT}"
echo "=== SUMMARY ===" | tee -a "${REPORT}"
echo "✅ PASS: ${PASS}" | tee -a "${REPORT}"
echo "⚠️  WARN: ${WARN}" | tee -a "${REPORT}"
echo "❌ FAIL: ${FAIL}" | tee -a "${REPORT}"
echo "Report: ${REPORT}" | tee -a "${REPORT}"
