#!/bin/bash

BASE_URL="http://localhost:8200"

echo "=========================================="
echo "Secreton E2E Test (Memory Backend)"
echo "=========================================="
echo ""

# Note: Using memory backend (raft feature disabled)
# Vault state persists in memory during container lifetime

echo "=== Test 1: Health Check ==="
curl -s $BASE_URL/health | python3 -m json.tool
echo ""

echo "=== Test 2: Version ==="
curl -s $BASE_URL/version | python3 -m json.tool
echo ""

echo "=== Test 3: Seal Status ==="
STATUS=$(curl -s $BASE_URL/v1/sys/seal-status)
echo "$STATUS" | python3 -m json.tool
SEALED=$(echo "$STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['sealed'])")
INITIALIZED=$(echo "$STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['initialized'])")
echo ""

if [ "$INITIALIZED" = "False" ]; then
    echo "=== Test 4: Initialize Vault ==="
    INIT_RESULT=$(curl -s -X POST $BASE_URL/v1/sys/init \
      -H "Content-Type: application/json" \
      -d '{"secret_shares":5,"secret_threshold":3}')
    echo "$INIT_RESULT" | python3 -m json.tool

    KEY1=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][0])")
    KEY2=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][1])")
    KEY3=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][2])")
    ROOT_TOKEN=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['root_token'])")
    echo ""
else
    echo "✓ Vault already initialized"
    echo "  Using test keys (for demo purposes only)"
    # These are dummy keys for testing - in production use real keys
    KEY1="test-key-1"
    KEY2="test-key-2"
    KEY3="test-key-3"
    ROOT_TOKEN="test-root-token"
    echo ""
fi

if [ "$SEALED" = "True" ]; then
    echo "=== Test 5: Unseal Vault (Key 1) ==="
    curl -s -X POST $BASE_URL/v1/sys/unseal \
      -H "Content-Type: application/json" \
      -d "{\"key\":\"$KEY1\"}" | python3 -m json.tool
    echo ""

    echo "=== Test 6: Unseal Vault (Key 2) ==="
    curl -s -X POST $BASE_URL/v1/sys/unseal \
      -H "Content-Type: application/json" \
      -d "{\"key\":\"$KEY2\"}" | python3 -m json.tool
    echo ""

    echo "=== Test 7: Unseal Vault (Key 3 - Final) ==="
    curl -s -X POST $BASE_URL/v1/sys/unseal \
      -H "Content-Type: application/json" \
      -d "{\"key\":\"$KEY3\"}" | python3 -m json.tool
    echo ""
else
    echo "✓ Vault already unsealed"
    echo ""
fi

echo "=== Test 8: System Health ==="
curl -s $BASE_URL/v1/sys/health | python3 -m json.tool
echo ""

echo "=== Test 9: List Secrets Engines ==="
curl -s $BASE_URL/v1/sys/mounts \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool 2>/dev/null || echo "Auth required or not available"
echo ""

echo "=== Test 10: Transit Encrypt ==="
curl -s -X POST $BASE_URL/v1/transit/encrypt/test-key \
  -H "Content-Type: application/json" \
  -d '{"plaintext":"SGVsbG8gV29ybGQ="}' | python3 -m json.tool
echo ""

echo "=== Test 11: KV Write ==="
curl -s -X POST $BASE_URL/v1/kv/test \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret123"}' | python3 -m json.tool
echo ""

echo "=== Test 12: KV Read ==="
curl -s $BASE_URL/v1/kv/test | python3 -m json.tool
echo ""

echo "=== Test 13: Metrics ==="
curl -s $BASE_URL/metrics | head -20
echo "..."
echo ""

echo "=========================================="
echo "✓ E2E Test Complete"
echo "=========================================="
echo ""
echo "Note: Using memory backend (raft feature disabled)"
echo "Data persists only during container lifetime"
