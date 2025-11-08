#!/usr/bin/env bash
# E2E Test: Upload flow using a real fixture file (anonymous user)
# Defaults to tests/fixtures/3DBenchy.3mf but can be overridden via env.
set -euo pipefail

DEFAULT_FIXTURE="tests/fixtures/3DBenchy.3mf"
FIXTURE_PATH="${UPLOAD_FIXTURE_PATH:-$DEFAULT_FIXTURE}"
if [ ! -f "$FIXTURE_PATH" ]; then
  echo "⚠ Skipping upload_fixture_test.sh – fixture not found at $FIXTURE_PATH"
  exit 0
fi

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
TIMEOUT="${TIMEOUT:-5}"
CONTENT_TYPE="${UPLOAD_FIXTURE_CONTENT_TYPE:-application/octet-stream}"
COOKIE_FILE="$(mktemp)"
trap 'rm -f "$COOKIE_FILE" /tmp/upload_fixture_put.txt /tmp/upload_fixture_confirm.txt' EXIT

FILE_SIZE="$(stat -c%s "$FIXTURE_PATH")"
if [ "$FILE_SIZE" -le 0 ]; then
  echo "❌ Fixture file is empty: $FIXTURE_PATH"
  exit 1
fi

INIT_PAYLOAD=$(
  jq -n \
    --arg filename "$(basename "$FIXTURE_PATH")" \
    --arg content_type "$CONTENT_TYPE" \
    --argjson size "$FILE_SIZE" \
    '{files: [{filename: $filename, content_type: $content_type, size_bytes: $size}]}'
)

echo "Testing upload flow with fixture: $FIXTURE_PATH ($FILE_SIZE bytes)"

# Step 1: Init
echo "  → Step 1: Init upload"
INIT_RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$API_URL/files/upload/init" \
  -c "$COOKIE_FILE" \
  -H "Content-Type: application/json" \
  -d "$INIT_PAYLOAD")

UPLOAD_ID=$(echo "$INIT_RESPONSE" | jq -r '.upload_id')
if [ -z "$UPLOAD_ID" ] || [ "$UPLOAD_ID" = "null" ]; then
  echo "❌ Init failed. Response: $INIT_RESPONSE"
  exit 1
fi
echo "  ✓ Init successful (upload_id=$UPLOAD_ID)"

# Step 2: Signed URLs
echo "  → Step 2: Get signed URLs"
URLS_RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$API_URL/files/upload/$UPLOAD_ID/urls" \
  -b "$COOKIE_FILE" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{}')
UPLOAD_URL=$(echo "$URLS_RESPONSE" | jq -r '.urls[0].upload_url')
if [ -z "$UPLOAD_URL" ] || [ "$UPLOAD_URL" = "null" ]; then
  echo "❌ Signed URLs missing. Response: $URLS_RESPONSE"
  exit 1
fi
echo "  ✓ Signed URL received"

# Step 3: PUT file
echo "  → Step 3: Upload file to S3"
PUT_STATUS=$(curl -s -o /tmp/upload_fixture_put.txt -w "%{http_code}" -X PUT \
  -H "Content-Type: $CONTENT_TYPE" \
  --data-binary @"$FIXTURE_PATH" \
  "$UPLOAD_URL")
if [ "$PUT_STATUS" != "200" ] && [ "$PUT_STATUS" != "204" ]; then
  echo "❌ Upload failed with status $PUT_STATUS"
  cat /tmp/upload_fixture_put.txt
  exit 1
fi
echo "  ✓ File uploaded"

# Step 4: Confirm
echo "  → Step 4: Confirm upload"
CONFIRM_RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$API_URL/files/upload/$UPLOAD_ID/confirm" \
  -b "$COOKIE_FILE" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{}')

if ! echo "$CONFIRM_RESPONSE" | jq -e '.status == "completed"' >/dev/null; then
  echo "❌ Confirm failed. Response: $CONFIRM_RESPONSE"
  exit 1
fi
echo "  ✓ Upload confirmed"

echo "✅ Fixture upload test passed (upload_id=$UPLOAD_ID)"
