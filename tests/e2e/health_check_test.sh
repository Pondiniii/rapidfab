#!/usr/bin/env bash
# E2E Test: Health Check
# Tests that all services are healthy and responding
set -euo pipefail

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
UPLOAD_URL="${UPLOAD_URL:-http://localhost:8082}"
TIMEOUT="${TIMEOUT:-5}"

echo "Testing health endpoints..."

# Test API health
echo "  → API health check"
API_HEALTH=$(curl -sf --max-time "$TIMEOUT" "$API_URL/health/healthz")

if [ -z "$API_HEALTH" ]; then
  echo "❌ Test failed: API health check returned empty response"
  exit 1
fi

if ! echo "$API_HEALTH" | grep -q "healthy"; then
  echo "❌ Test failed: API not healthy"
  echo "Response: $API_HEALTH"
  exit 1
fi

echo "  ✓ API healthy"

# Test Upload service health
echo "  → Upload service health check"
UPLOAD_HEALTH=$(curl -sf --max-time "$TIMEOUT" "$UPLOAD_URL/health")

if [ -z "$UPLOAD_HEALTH" ]; then
  echo "❌ Test failed: Upload service health check returned empty response"
  exit 1
fi

if ! echo "$UPLOAD_HEALTH" | grep -q "healthy"; then
  echo "❌ Test failed: Upload service not healthy"
  echo "Response: $UPLOAD_HEALTH"
  exit 1
fi

echo "  ✓ Upload service healthy"

echo "✅ Health check test passed"
exit 0
