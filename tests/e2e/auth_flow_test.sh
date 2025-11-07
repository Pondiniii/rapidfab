#!/bin/bash
set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
EMAIL="test-$(date +%s)@example.com"
PASSWORD="testpass123"

echo "=== E2E Test: Auth Flow ==="
echo "Base URL: $BASE_URL"
echo "Email: $EMAIL"

# Test 1: Health check
echo -n "Test 1: Health check... "
HEALTH=$(curl -s "$BASE_URL/health/healthz")
if echo "$HEALTH" | grep -q "healthy"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
    exit 1
fi

# Test 2: Register
echo -n "Test 2: Register user... "
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\",\"full_name\":\"Test User\"}")

TOKEN=$(echo "$REGISTER_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "❌ FAIL - No token received"
    echo "Response: $REGISTER_RESPONSE"
    exit 1
else
    echo "✅ PASS"
fi

# Test 3: Get profile
echo -n "Test 3: Get user profile... "
PROFILE=$(curl -s "$BASE_URL/users/me" \
    -H "Authorization: Bearer $TOKEN")

if echo "$PROFILE" | grep -q "$EMAIL"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
    echo "Response: $PROFILE"
    exit 1
fi

# Test 4: Logout
echo -n "Test 4: Logout... "
LOGOUT_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/auth/logout" \
    -H "Authorization: Bearer $TOKEN")

if [ "$LOGOUT_STATUS" = "204" ]; then
    echo "✅ PASS"
else
    echo "❌ FAIL (status: $LOGOUT_STATUS)"
    exit 1
fi

# Test 5: Try accessing profile after logout (should fail)
echo -n "Test 5: Profile access after logout (should fail)... "
PROFILE_AFTER_LOGOUT_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/users/me" \
    -H "Authorization: Bearer $TOKEN")

if [ "$PROFILE_AFTER_LOGOUT_STATUS" = "401" ]; then
    echo "✅ PASS"
else
    echo "❌ FAIL (expected 401, got $PROFILE_AFTER_LOGOUT_STATUS)"
    exit 1
fi

# Test 6: Login with credentials
echo -n "Test 6: Login with credentials... "
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")

NEW_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$NEW_TOKEN" ]; then
    echo "❌ FAIL - No token received"
    exit 1
else
    echo "✅ PASS"
fi

echo ""
echo "=== All E2E tests passed! ✅ ==="
