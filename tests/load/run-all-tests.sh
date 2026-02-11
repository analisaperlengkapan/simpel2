#!/bin/bash

# Run all k6 load tests
# Usage: ./tests/load/run-all-tests.sh [base_url] [username] [password]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="${1:-${BASE_URL:-http://localhost:3020}}"
TEST_USERNAME="${2:-${TEST_USERNAME:-test_operator}}"
TEST_PASSWORD="${3:-${TEST_PASSWORD:-test_password}}"
RESULTS_DIR="tests/load/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}SIMPEL Load Testing Suite${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Configuration:"
echo "  Base URL: $BASE_URL"
echo "  Username: $TEST_USERNAME"
echo "  Results: $RESULTS_DIR"
echo ""

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo -e "${RED}Error: k6 is not installed${NC}"
    echo "Please install k6: https://k6.io/docs/get-started/installation/"
    exit 1
fi

# Check if server is reachable
echo -e "${YELLOW}Checking server connectivity...${NC}"
if ! curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health" | grep -q "200\|404"; then
    echo -e "${RED}Warning: Server at $BASE_URL may not be reachable${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Function to run a test
run_test() {
    local test_name=$1
    local test_file=$2
    local output_file="$RESULTS_DIR/${test_name}_${TIMESTAMP}.json"

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Running: $test_name${NC}"
    echo -e "${GREEN}========================================${NC}"

    if k6 run \
        -e BASE_URL="$BASE_URL" \
        -e TEST_USERNAME="$TEST_USERNAME" \
        -e TEST_PASSWORD="$TEST_PASSWORD" \
        --out json="$output_file" \
        "$test_file"; then
        echo -e "${GREEN}✓ $test_name completed successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ $test_name failed${NC}"
        return 1
    fi
}

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test 1: Dashboard Load Test
TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test "dashboard-load-test" "tests/load/dashboard-load-test.js"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# Test 2: API Load Test
TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test "api-load-test" "tests/load/api-load-test.js"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# Test 3: Search Load Test
TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test "search-load-test" "tests/load/search-load-test.js"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# Summary
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Test Summary${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Total tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
fi
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""

# Generate summary report
SUMMARY_FILE="$RESULTS_DIR/summary_${TIMESTAMP}.txt"
cat > "$SUMMARY_FILE" << EOF
SIMPEL Load Testing Summary
===========================

Date: $(date)
Base URL: $BASE_URL

Test Results:
-------------
Total: $TOTAL_TESTS
Passed: $PASSED_TESTS
Failed: $FAILED_TESTS

Individual Test Results:
EOF

for result_file in "$RESULTS_DIR"/*_${TIMESTAMP}.json; do
    if [ -f "$result_file" ]; then
        test_name=$(basename "$result_file" "_${TIMESTAMP}.json")
        echo "" >> "$SUMMARY_FILE"
        echo "$test_name:" >> "$SUMMARY_FILE"

        # Extract key metrics from JSON (requires jq)
        if command -v jq &> /dev/null; then
            echo "  Requests: $(jq -r '.metrics.http_reqs.values.count // "N/A"' "$result_file")" >> "$SUMMARY_FILE"
            echo "  Failed: $(jq -r '.metrics.http_req_failed.values.rate // "N/A"' "$result_file")" >> "$SUMMARY_FILE"
            echo "  Avg Duration: $(jq -r '.metrics.http_req_duration.values.avg // "N/A"' "$result_file")ms" >> "$SUMMARY_FILE"
            echo "  P95 Duration: $(jq -r '.metrics.http_req_duration.values["p(95)"] // "N/A"' "$result_file")ms" >> "$SUMMARY_FILE"
        fi
    fi
done

echo "Summary report: $SUMMARY_FILE"

# Exit with appropriate code
if [ $FAILED_TESTS -gt 0 ]; then
    exit 1
else
    exit 0
fi
