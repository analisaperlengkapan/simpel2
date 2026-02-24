#!/bin/bash
# Test script to verify vault initialization status behavior
# CRITICAL: Vault should NOT be initialized on fresh start

set -e

echo "=========================================="
echo "Testing Vault Initialization Status"
echo "=========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Clean up function
cleanup() {
    echo ""
    echo "Cleaning up..."
    docker-compose down -v 2>/dev/null || true
    rm -rf /tmp/secreton-test-data 2>/dev/null || true
}

# Set trap to cleanup on exit
trap cleanup EXIT

echo "Step 1: Clean environment"
echo "----------------------------------------"
cleanup
mkdir -p /tmp/secreton-test-data
echo "✓ Environment cleaned"
echo ""

echo "Step 2: Build Docker image"
echo "----------------------------------------"
docker-compose build secreton
echo "✓ Image built"
echo ""

echo "Step 3: Start fresh Secreton instance"
echo "----------------------------------------"
docker-compose up -d secreton postgres redis
echo "Waiting for services to be ready..."
sleep 5
echo "✓ Services started"
echo ""

echo "Step 4: Check seal status (should be NOT initialized)"
echo "----------------------------------------"
RESPONSE=$(curl -s http://localhost:8200/v1/sys/seal-status)
echo "Response: $RESPONSE"
echo ""

# Parse JSON response
INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')

echo "Parsed values:"
echo "  initialized: $INITIALIZED"
echo "  sealed: $SEALED"
echo ""

# CRITICAL TEST: Vault should NOT be initialized on fresh start
if [ "$INITIALIZED" = "false" ]; then
    echo -e "${GREEN}✓ PASS: Vault is NOT initialized (correct behavior)${NC}"
else
    echo -e "${RED}✗ FAIL: Vault shows as initialized on fresh start!${NC}"
    echo -e "${RED}  This is a CRITICAL SECURITY BUG${NC}"
    exit 1
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS: Vault is sealed (correct behavior)${NC}"
else
    echo -e "${YELLOW}⚠ WARNING: Vault is not sealed${NC}"
fi

echo ""
echo "Step 5: Initialize vault"
echo "----------------------------------------"
INIT_RESPONSE=$(curl -s -X POST http://localhost:8200/v1/sys/init \
    -H "Content-Type: application/json" \
    -d '{
        "secret_shares": 5,
        "secret_threshold": 3
    }')

echo "Init response received"
KEYS=$(echo "$INIT_RESPONSE" | jq -r '.keys[]' | head -3)
ROOT_TOKEN=$(echo "$INIT_RESPONSE" | jq -r '.root_token')

if [ -z "$ROOT_TOKEN" ] || [ "$ROOT_TOKEN" = "null" ]; then
    echo -e "${RED}✗ FAIL: Failed to initialize vault${NC}"
    echo "Response: $INIT_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✓ Vault initialized successfully${NC}"
echo ""

echo "Step 6: Check seal status after init (should be initialized but sealed)"
echo "----------------------------------------"
RESPONSE=$(curl -s http://localhost:8200/v1/sys/seal-status)
echo "Response: $RESPONSE"
echo ""

INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')

echo "Parsed values:"
echo "  initialized: $INITIALIZED"
echo "  sealed: $SEALED"
echo ""

if [ "$INITIALIZED" = "true" ]; then
    echo -e "${GREEN}✓ PASS: Vault is now initialized${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should be initialized after init${NC}"
    exit 1
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS: Vault remains sealed after init (correct security behavior)${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should remain sealed after init${NC}"
    exit 1
fi

echo ""
echo "Step 7: Unseal vault with threshold shares"
echo "----------------------------------------"
SHARE_COUNT=0
for KEY in $KEYS; do
    SHARE_COUNT=$((SHARE_COUNT + 1))
    echo "Providing share $SHARE_COUNT/3..."

    UNSEAL_RESPONSE=$(curl -s -X POST http://localhost:8200/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"$KEY\"}")

    SEALED=$(echo "$UNSEAL_RESPONSE" | jq -r '.data.sealed')
    PROGRESS=$(echo "$UNSEAL_RESPONSE" | jq -r '.data.progress')

    echo "  Progress: $PROGRESS/3, Sealed: $SEALED"
done

echo ""
if [ "$SEALED" = "false" ]; then
    echo -e "${GREEN}✓ PASS: Vault unsealed successfully${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should be unsealed after providing threshold shares${NC}"
    exit 1
fi

echo ""
echo "Step 8: Restart container and verify persistence"
echo "----------------------------------------"
docker-compose restart secreton
echo "Waiting for restart..."
sleep 5

RESPONSE=$(curl -s http://localhost:8200/v1/sys/seal-status)
INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')

echo "After restart:"
echo "  initialized: $INITIALIZED"
echo "  sealed: $SEALED"
echo ""

if [ "$INITIALIZED" = "true" ]; then
    echo -e "${GREEN}✓ PASS: Vault remains initialized after restart${NC}"
else
    echo -e "${RED}✗ FAIL: Vault lost initialization state${NC}"
    exit 1
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS: Vault is sealed after restart (correct security behavior)${NC}"
else
    echo -e "${YELLOW}⚠ WARNING: Vault should be sealed after restart${NC}"
fi

echo ""
echo "=========================================="
echo -e "${GREEN}ALL TESTS PASSED!${NC}"
echo "=========================================="
echo ""
echo "Summary:"
echo "  ✓ Fresh vault starts as NOT initialized"
echo "  ✓ Vault can be initialized"
echo "  ✓ Vault remains sealed after initialization"
echo "  ✓ Vault can be unsealed with threshold shares"
echo "  ✓ Initialization state persists across restarts"
echo ""
