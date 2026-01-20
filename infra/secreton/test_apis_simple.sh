#!/bin/bash

BASE_URL="http://localhost:8200"

# Note: Vault is already initialized and sealed
# For testing purposes, we'll use a mock token
# In production, you would unseal first and use real token

echo "=========================================="
echo "Secreton API Test (Sealed Vault)"
echo "Testing Public Endpoints"
echo "=========================================="
echo ""

echo "=== Test 1: Health Check ==="
curl -s $BASE_URL/health | python3 -m json.tool
echo ""

echo "=== Test 2: Version ==="
curl -s $BASE_URL/version | python3 -m json.tool
echo ""

echo "=== Test 3: Seal Status ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

echo "=== Test 4: Metrics ==="
curl -s $BASE_URL/metrics | head -20
echo "..."
echo ""

echo "=== Test 5: TLS Metrics ==="
curl -s $BASE_URL/metrics/tls | python3 -m json.tool
echo ""

echo "=========================================="
echo "Note: Vault is SEALED"
echo "To test authenticated endpoints:"
echo "1. Unseal vault with 3 of 5 keys"
echo "2. Use root token for authentication"
echo "=========================================="
echo ""

echo "Vault Status:"
docker logs secreton-api-test 2>&1 | grep -E "(SEALED|UNSEALED|initialized)" | tail -5
