#!/bin/bash
# Tachyon E2E API Test Suite
# Tests the complete user journey via API calls

SERVER="http://192.168.1.191:8080"
PASS=0
FAIL=0
TOKEN=""

echo "=== Tachyon E2E API Test Suite ==="
echo "Target: $SERVER"
echo ""

# Helper function
test_result() {
    local name="$1"
    local expected="$2"
    local actual="$3"
    if [ "$expected" = "$actual" ]; then
        echo "  ✓ $name (expected=$expected)"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $name (expected=$expected, got=$actual)"
        FAIL=$((FAIL + 1))
    fi
}

# Test 1: Health check
echo "[1/10] Health check"
STATUS=$(curl -sf -o /dev/null -w '%{http_code}' "$SERVER/health")
test_result "GET /health returns 200" "200" "$STATUS"

# Test 2: Login
echo "[2/10] Login"
RESPONSE=$(curl -sf -X POST "$SERVER/api/v1/auth/login" \
    -H 'Content-Type: application/json' \
    -d '{"username":"admin","password":"admin123"}')
TOKEN=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)
SUCCESS=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('success',False))" 2>/dev/null)
test_result "POST /api/v1/auth/login returns success" "True" "$SUCCESS"
test_result "Login returns JWT token" "True" "$([ -n "$TOKEN" ] && echo True || echo False)"

# Test 3: Get current user
echo "[3/10] Get current user"
STATUS=$(curl -sf -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $TOKEN" "$SERVER/api/v1/auth/me")
test_result "GET /api/v1/auth/me returns 200" "200" "$STATUS"

# Test 4: Create document
echo "[4/10] Create document"
DOC_RESPONSE=$(curl -sf -X POST "$SERVER/api/v1/documents" \
    -H "Authorization: Bearer $TOKEN" \
    -H 'Content-Type: application/json' \
    -d '{"title":"E2E Test Document","content":"# Test\n\nThis is an E2E test document with [[wiki-links]].","tags":["e2e","test"]}')
DOC_ID=$(echo "$DOC_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))" 2>/dev/null)
test_result "POST /api/v1/documents creates document" "True" "$([ -n "$DOC_ID" ] && echo True || echo False)"
echo "  Document ID: $DOC_ID"

# Test 5: Get document
echo "[5/10] Get document"
STATUS=$(curl -sf -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $TOKEN" "$SERVER/api/v1/documents/$DOC_ID")
test_result "GET /api/v1/documents/$DOC_ID returns 200" "200" "$STATUS"

# Test 6: Update document
echo "[6/10] Update document"
STATUS=$(curl -sf -o /dev/null -w '%{http_code}' -X PUT "$SERVER/api/v1/documents/$DOC_ID" \
    -H "Authorization: Bearer $TOKEN" \
    -H 'Content-Type: application/json' \
    -d '{"title":"E2E Test Document Updated","content":"# Updated\n\nThis document has been updated."}')
test_result "PUT /api/v1/documents/$DOC_ID returns 200" "200" "$STATUS"

# Test 7: Search
echo "[7/10] Search"
SEARCH_RESPONSE=$(curl -sf -H "Authorization: Bearer $TOKEN" "$SERVER/api/v1/search?q=E2E")
SEARCH_COUNT=$(echo "$SEARCH_RESPONSE" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('total',d.get('count',0)))" 2>/dev/null)
test_result "Search finds E2E document" "True" "$([ "$SEARCH_COUNT" -gt 0 ] 2>/dev/null && echo True || echo False)"

# Test 8: List documents
echo "[8/10] List documents"
LIST_RESPONSE=$(curl -sf -H "Authorization: Bearer $TOKEN" "$SERVER/api/v1/documents?page=1&page_size=10")
DOC_COUNT=$(echo "$LIST_RESPONSE" | python3 -c "import sys,json; d=json.load(sys.stdin); items=d.get('results',d.get('items',d.get('documents',[]))); print(len(items))" 2>/dev/null)
test_result "GET /api/v1/documents returns documents" "True" "$([ "$DOC_COUNT" -gt 0 ] 2>/dev/null && echo True || echo False)"

# Test 9: Backlinks
echo "[9/10] Backlinks"
STATUS=$(curl -sf -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $TOKEN" "$SERVER/api/v1/documents/$DOC_ID/backlinks")
test_result "GET /api/v1/documents/$DOC_ID/backlinks returns 200" "200" "$STATUS"

# Test 10: Delete document
echo "[10/10] Delete document"
STATUS=$(curl -sf -o /dev/null -w '%{http_code}' -X DELETE -H "Authorization: Bearer $TOKEN" "$SERVER/api/v1/documents/$DOC_ID")
test_result "DELETE /api/v1/documents/$DOC_ID returns 204" "204" "$STATUS"

# Verify deletion
STATUS=$(curl -sf -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $TOKEN" "$SERVER/api/v1/documents/$DOC_ID")
test_result "Deleted document returns 404" "404" "$STATUS"

# Summary
echo ""
echo "=== RESULTS ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo "Total: $((PASS + FAIL))"
if [ $FAIL -eq 0 ]; then
    echo "STATUS: ALL TESTS PASSED"
else
    echo "STATUS: $FAIL TESTS FAILED"
fi
