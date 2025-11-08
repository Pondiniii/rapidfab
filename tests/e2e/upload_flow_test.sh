#!/usr/bin/env bash
# E2E Test: Upload Flow (anonymous user)
# Tests the complete upload flow including ticket generation, init, signed URLs
set -euo pipefail

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
UPLOAD_URL="${UPLOAD_URL:-http://localhost:8082}"
TIMEOUT="${TIMEOUT:-5}"
CONTENT_TYPE="application/vnd.ms-pki.stl"
TMP_FILE="$(mktemp)"
trap 'rm -f "$TMP_FILE" /tmp/upload_test_cookies.txt /tmp/upload_put.txt /tmp/confirm_response.txt' EXIT

# Prepare deterministic payload (256 bytes) to keep size in sync with presigned headers
dd if=/dev/zero bs=256 count=1 status=none > "$TMP_FILE"
FILE_SIZE="$(wc -c < "$TMP_FILE" | tr -d ' ')"
INIT_PAYLOAD=$(
  jq -n \
    --arg filename "test.stl" \
    --arg content_type "$CONTENT_TYPE" \
    --argjson size "$FILE_SIZE" \
    '{files: [{filename: $filename, content_type: $content_type, size_bytes: $size}]}'
)

echo "Testing upload flow..."

# Step 1: Init upload (API generates ticket and forwards to upload service)
echo "  → Step 1: Init upload"
INIT_RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$API_URL/files/upload/init" \
  -c /tmp/upload_test_cookies.txt \
  -H "Content-Type: application/json" \
  -d "$INIT_PAYLOAD")

if [ -z "$INIT_RESPONSE" ]; then
  echo "❌ Test failed: empty response from init"
  exit 1
fi

# Extract upload_id from response
UPLOAD_ID=$(echo "$INIT_RESPONSE" | grep -o '"upload_id":"[^"]*"' | cut -d'"' -f4)

if [ -z "$UPLOAD_ID" ]; then
  echo "❌ Test failed: no upload_id in response"
  echo "Response: $INIT_RESPONSE"
  exit 1
fi

echo "  ✓ Init successful, upload_id: $UPLOAD_ID"

# Step 2: Get signed URLs (API forwards to upload service)
echo "  → Step 2: Get signed URLs"
URLS_RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$API_URL/files/upload/$UPLOAD_ID/urls" \
  -b /tmp/upload_test_cookies.txt \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{}')

if [ -z "$URLS_RESPONSE" ]; then
  echo "❌ Test failed: empty response from signed-urls"
  exit 1
fi

# Check if response contains upload_url
if ! echo "$URLS_RESPONSE" | grep -q "upload_url"; then
  echo "❌ Test failed: no upload_url in response"
  echo "Response: $URLS_RESPONSE"
  exit 1
fi

echo "  ✓ Signed URLs generated"

# Step 3: Upload file to presigned URL
echo "  → Step 3: Upload file to S3"
UPLOAD_URL=$(python - <<'PY' "$URLS_RESPONSE"
import json, sys
data = json.loads(sys.argv[1])
print(data["urls"][0]["upload_url"])
PY
)

if [ -z "$UPLOAD_URL" ]; then
  echo "❌ Test failed: missing upload_url"
  exit 1
fi

PUT_STATUS=$(curl -s -o /tmp/upload_put.txt -w "%{http_code}" -X PUT \
  -H "Content-Type: $CONTENT_TYPE" \
  --data-binary @"$TMP_FILE" \
  "$UPLOAD_URL")

if [ "$PUT_STATUS" != "200" ] && [ "$PUT_STATUS" != "204" ]; then
  echo "❌ Test failed: uploading to presigned URL failed (status $PUT_STATUS)"
  cat /tmp/upload_put.txt
  exit 1
fi

echo "  ✓ File uploaded"

# Step 4: Confirm upload
echo "  → Step 4: Confirm upload"
CONFIRM_RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$API_URL/files/upload/$UPLOAD_ID/confirm" \
  -b /tmp/upload_test_cookies.txt \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{}')

if ! echo "$CONFIRM_RESPONSE" | grep -q '"status":"completed"'; then
  echo "❌ Test failed: confirm upload did not succeed"
  echo "Response: $CONFIRM_RESPONSE"
  exit 1
fi

echo "  ✓ Upload confirmed"

echo "✅ Upload flow test passed"
exit 0
