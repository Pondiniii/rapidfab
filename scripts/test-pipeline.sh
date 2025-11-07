#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.yml"
API_URL="http://localhost:8080"
MAX_WAIT=60
TESTS_DIR="tests/e2e"

echo -e "${YELLOW}=== RapidFab Testing Pipeline ===${NC}"
echo "Testing as containers (prod-like environment)"
echo ""

# Step 1: Cleanup previous runs
echo -e "${YELLOW}[1/5] Cleanup previous containers...${NC}"
docker-compose -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""

# Step 2: Build Docker images (compilation pipeline)
echo -e "${YELLOW}[2/5] Building Docker images (compilation pipeline)...${NC}"
docker-compose -f "$COMPOSE_FILE" build --no-cache
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Step 3: Start services (deployment pipeline)
echo -e "${YELLOW}[3/5] Starting services (deployment pipeline)...${NC}"
docker-compose -f "$COMPOSE_FILE" up -d

# Wait for API to be healthy
echo -n "Waiting for API to be ready"
ELAPSED=0
while [ $ELAPSED -lt $MAX_WAIT ]; do
    if curl -sf "$API_URL/health/healthz" > /dev/null 2>&1; then
        echo ""
        echo -e "${GREEN}✓ API is ready${NC}"
        break
    fi
    echo -n "."
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

if [ $ELAPSED -ge $MAX_WAIT ]; then
    echo ""
    echo -e "${RED}✗ API failed to start within ${MAX_WAIT}s${NC}"
    echo "Container logs:"
    docker-compose -f "$COMPOSE_FILE" logs api
    docker-compose -f "$COMPOSE_FILE" down -v
    exit 1
fi
echo ""

# Step 4: Run E2E tests
echo -e "${YELLOW}[4/5] Running E2E tests...${NC}"
FAILED_TESTS=0
PASSED_TESTS=0

if [ -d "$TESTS_DIR" ]; then
    for test_file in "$TESTS_DIR"/*.sh; do
        if [ -f "$test_file" ]; then
            test_name=$(basename "$test_file")
            echo -e "${YELLOW}Running: $test_name${NC}"

            if bash "$test_file"; then
                echo -e "${GREEN}✓ $test_name PASSED${NC}"
                PASSED_TESTS=$((PASSED_TESTS + 1))
            else
                echo -e "${RED}✗ $test_name FAILED${NC}"
                FAILED_TESTS=$((FAILED_TESTS + 1))
            fi
            echo ""
        fi
    done
else
    echo -e "${RED}✗ E2E tests directory not found: $TESTS_DIR${NC}"
    docker-compose -f "$COMPOSE_FILE" down -v
    exit 1
fi

# Step 5: Cleanup
echo -e "${YELLOW}[5/5] Cleanup...${NC}"
docker-compose -f "$COMPOSE_FILE" down -v
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""

# Summary
echo -e "${YELLOW}=== Test Results ===${NC}"
echo "Passed: $PASSED_TESTS"
echo "Failed: $FAILED_TESTS"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
