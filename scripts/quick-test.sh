#!/bin/bash
set -e

echo "==================================="
echo "Quick Integration Test - Authenc & Secreton"
echo "==================================="
echo ""

AUTHENC_URL="http://localhost:8088"
SECRETON_URL="http://localhost:8200"
PASSED=0
FAILED=0

test_endpoint() {
    local name="$1"
    local url="$2"
    local expected_field="$3"

    echo -n "Testing $name... "

    if response=$(curl -s -f -m 5 "$url" 2>&1); then
        if echo "$response" | jq -e ".$expected_field" > /dev/null 2>&1; then
            echo "✓ PASSED"
            ((PASSED++))
            return 0
        else
            echo "✗ FAILED - Missing field: $expected_field"
            echo "  Response: $response"
            ((FAILED++))
            return 1
        fi
    else
        echo "✗ FAILED - No response"
        echo "  Error: $response"
        ((FAILED++))
        return 1
    fi
}

echo "╔══════════════════════════════════╗"
echo "║ AUTHENC Tests                    ║"
echo "╚══════════════════════════════════╝"
test_endpoint "Health Endpoint" "$AUTHENC_URL/health" "status"
test_endpoint "Ready Endpoint" "$AUTHENC_URL/ready" "status"

echo ""
echo "╔══════════════════════════════════╗"
echo "║ SECRETON Tests                   ║"
echo "╚══════════════════════════════════╝"
test_endpoint "Health Endpoint" "$SECRETON_URL/health" "status"
test_endpoint "Version Endpoint" "$SECRETON_URL/v1/sys/health" "initialized"

echo ""
echo "==================================="
echo "Results: $PASSED passed, $FAILED failed"
echo "==================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
