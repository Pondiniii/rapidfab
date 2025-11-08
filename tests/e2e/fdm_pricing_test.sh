#!/usr/bin/env bash
set -euo pipefail

# FDM Pricing Service E2E Test
# Tests health check and basic API contract

PRICING_URL="${PRICING_URL:-http://localhost:8083}"
TIMEOUT="${TIMEOUT:-5}"

echo "Testing FDM Pricing Service..."

# Test 1: Health check
echo "  → Test 1: Health check"
HEALTH_RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$PRICING_URL/health" || echo "FAIL")

if [ "$HEALTH_RESPONSE" != "OK" ]; then
  echo "❌ Health check failed: expected 'OK', got '$HEALTH_RESPONSE'"
  exit 1
fi

echo "  ✓ Health check passed"

# Test 2: Quote endpoint rejects invalid request
echo "  → Test 2: Invalid request validation"
INVALID_RESPONSE=$(curl -s --max-time "$TIMEOUT" -w "\n%{http_code}" "$PRICING_URL/internal/pricing/fdm/quote" \
  -H "Content-Type: application/json" \
  -d '{"file_url":"","material":"invalid","infill":200,"layer_thickness":999}') || echo "FAIL"

HTTP_CODE=$(echo "$INVALID_RESPONSE" | tail -n1)

if [ "$HTTP_CODE" != "400" ]; then
  echo "❌ Expected HTTP 400 for invalid request, got $HTTP_CODE"
  exit 1
fi

echo "  ✓ Invalid request rejected correctly"

# Test 3: Quote endpoint accepts valid structure (will fail on file download, which is expected)
echo "  → Test 3: Valid request structure"
VALID_RESPONSE=$(curl -s --max-time "$TIMEOUT" -w "\n%{http_code}" "$PRICING_URL/internal/pricing/fdm/quote" \
  -H "Content-Type: application/json" \
  -d '{
    "file_url":"https://example.com/test.stl",
    "material":"pla",
    "infill":20,
    "layer_thickness":200
  }') || echo "FAIL"

HTTP_CODE=$(echo "$VALID_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$VALID_RESPONSE" | head -n -1)

# Should fail with 400 (download failed) - that's expected for invalid URL
if [ "$HTTP_CODE" != "400" ]; then
  # If it somehow succeeds (unlikely), check response structure
  if echo "$RESPONSE_BODY" | grep -q '"quote_id"'; then
    echo "  ✓ Valid request structure accepted (unexpected success)"
  else
    echo "  ⚠ Unexpected response: HTTP $HTTP_CODE"
  fi
else
  # Check error message mentions download failure
  if echo "$RESPONSE_BODY" | grep -q "Failed to download"; then
    echo "  ✓ Valid request structure accepted (download failed as expected)"
  else
    echo "  ⚠ Unexpected error message: $RESPONSE_BODY"
  fi
fi

# Test 4: Material validation
echo "  → Test 4: Material validation"
for material in pla abs petg nylon; do
  RESPONSE=$(curl -s --max-time "$TIMEOUT" -w "\n%{http_code}" "$PRICING_URL/internal/pricing/fdm/quote" \
    -H "Content-Type: application/json" \
    -d "{\"file_url\":\"https://example.com/test.stl\",\"material\":\"$material\",\"infill\":20,\"layer_thickness\":200}") || echo "FAIL"

  HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

  # Should fail on download (400), not validation (400 with "Invalid material")
  if [ "$HTTP_CODE" = "400" ]; then
    BODY=$(echo "$RESPONSE" | head -n -1)
    if echo "$BODY" | grep -q "Invalid material"; then
      echo "❌ Material $material should be valid but was rejected"
      exit 1
    fi
  fi
done

echo "  ✓ Material validation passed"

echo "✅ FDM Pricing Service E2E tests passed"
exit 0
