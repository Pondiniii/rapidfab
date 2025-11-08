#!/usr/bin/env bash
# Upload Service Health Check E2E Test
# Verifies upload service responds and is healthy

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "Testing upload service health..."

# Test 1: Service responds
echo -n "  → Upload service responds... "
if curl -sf http://localhost:8082/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo "Upload service not responding on port 8082"
    exit 1
fi

# Test 2: Health endpoint returns proper JSON
echo -n "  → Health endpoint format... "
HEALTH_RESPONSE=$(curl -sf http://localhost:8082/health)
if echo "$HEALTH_RESPONSE" | grep -q '"status"'; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo "Health response invalid: $HEALTH_RESPONSE"
    exit 1
fi

# Test 3: Status is "healthy"
echo -n "  → Status is healthy... "
if echo "$HEALTH_RESPONSE" | grep -q '"status":"healthy"'; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo "Status not healthy: $HEALTH_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✅ Upload service health check passed${NC}"
