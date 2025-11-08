#!/usr/bin/env bash
# RapidFab.xyz E2E Test Runner
# Auto-discovers and runs all tests from tests/e2e/*_test.sh
# Convention: tests must end with _test.sh and be executable
# Exit code: 0 = all passed, non-zero = failures

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TESTS_DIR="tests/e2e"
TEST_PATTERN="*_test.sh"
FAILED_TESTS=0
PASSED_TESTS=0
TOTAL_TESTS=0

# Change to project root (script can be called from anywhere)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "${BLUE}  E2E Test Runner (Auto-Discovery)${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo ""
echo -e "Tests directory: ${YELLOW}$TESTS_DIR${NC}"
echo -e "Test pattern: ${YELLOW}$TEST_PATTERN${NC}"
echo ""

# Check if tests directory exists
if [ ! -d "$TESTS_DIR" ]; then
    echo -e "${RED}✗ Tests directory not found: $TESTS_DIR${NC}"
    exit 1
fi

# Discover test files
echo -e "${YELLOW}Discovering tests...${NC}"
TEST_FILES=()
while IFS= read -r -d '' test_file; do
    TEST_FILES+=("$test_file")
done < <(find "$TESTS_DIR" -maxdepth 1 -name "$TEST_PATTERN" -type f -print0 | sort -z)

TOTAL_TESTS=${#TEST_FILES[@]}

if [ $TOTAL_TESTS -eq 0 ]; then
    echo -e "${YELLOW}⚠ No tests found matching pattern: $TEST_PATTERN${NC}"
    echo -e "${YELLOW}⚠ To add tests, create files: tests/e2e/your_feature_test.sh${NC}"
    echo ""
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo -e "${GREEN}✅ 0 tests found (not an error)${NC}"
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    exit 0
fi

echo -e "${GREEN}✓ Found $TOTAL_TESTS test(s)${NC}"
echo ""

# Run each test
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "${BLUE}  Running Tests${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo ""

for test_file in "${TEST_FILES[@]}"; do
    test_name=$(basename "$test_file")
    test_num=$((PASSED_TESTS + FAILED_TESTS + 1))

    echo -e "${YELLOW}[$test_num/$TOTAL_TESTS] Running: $test_name${NC}"

    # Make sure test is executable
    if [ ! -x "$test_file" ]; then
        echo -e "${YELLOW}  ⚠ Making test executable...${NC}"
        chmod +x "$test_file"
    fi

    # Run test and capture output
    if bash "$test_file" 2>&1; then
        echo -e "${GREEN}  ✅ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}  ❌ FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
done

# Summary
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "${BLUE}  Test Results${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo ""
echo -e "Total:  ${BLUE}$TOTAL_TESTS${NC}"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED_TESTS test(s) failed${NC}"
    exit 1
fi
