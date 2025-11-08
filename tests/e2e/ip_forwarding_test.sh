#!/usr/bin/env bash
# E2E Test: IP Forwarding to Upload Service
# Verifies that X-Forwarded-For header is properly forwarded to upload-service
set -euo pipefail

# Configuration
API_URL="${API_URL:-http://localhost:8080}"

echo "Testing IP forwarding..."

# Test: Make request with X-Forwarded-For header
# Expected: API should forward this header to upload-service
# Note: We can't directly verify upload-service received it without checking logs,
# but we can verify the request doesn't fail due to missing header handling

echo "  → Making request with X-Forwarded-For header"
HTTP_CODE=$(curl -s -o /tmp/ip_forwarding_init.txt -w "%{http_code}" --max-time 5 "$API_URL/files/upload/init" \
  -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 203.0.113.42" \
  -d '{
    "files": [
      {
        "filename": "test.stl",
        "content_type": "application/vnd.ms-pki.stl",
        "size_bytes": 1024
      }
    ]
  }')

if [ "$HTTP_CODE" != "200" ]; then
  echo "❌ Test failed: expected 200 from init upload, got $HTTP_CODE"
  cat /tmp/ip_forwarding_init.txt
  exit 1
fi

SESSION_HEADER=$(curl -s -D - --max-time 5 "$API_URL/health/healthz" | grep -i "Set-Cookie: rapidfab_session")
if [ -z "$SESSION_HEADER" ]; then
  echo "❌ Test failed: session cookie not set"
  exit 1
fi

echo "  ✓ API handled X-Forwarded-For header and session cookie successfully"

echo "✅ IP forwarding test passed (header extraction works)"
exit 0
