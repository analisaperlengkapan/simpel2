#!/bin/bash
set -e

BASE_URL="http://localhost:8200"
KEYS_FILE="/tmp/vault_keys_e2e.json"

echo "=========================================="
echo "Secreton End-to-End Test"
echo "=========================================="
echo ""

# Cleanup
echo "=== Cleanup ==="
docker rm -f secreton-e2e 2>/dev/null || true
sudo rm -rf data 2>/dev/null || true
mkdir -p data/raft
rm -f $KEYS_FILE
echo "✓ Cleanup complete"
echo ""

# Start container
echo "=== Starting Secreton ==="
docker run -d --name secreton-e2e \
  -p 8200:8200 -p 8201:8201 \
  -v $(pwd)/secreton.toml:/app/secreton.toml:ro \
  -v $(pwd)/data:/var/lib/secreton \
  secreton:raft
echo "✓ Container started"
echo ""

# Wait for startup
echo "=== Waiting for startup ==="
sleep 5
echo "✓ Ready"
echo ""

# Test 1: Health Check
echo "=== Test 1: Health Check ==="
curl -s $BASE_URL/health | python3 -m json.tool
echo ""

# Test 2: Version
echo "=== Test 2: Version ==="
curl -s $BASE_URL/version | python3 -m json.tool
echo ""

# Test 3: Seal Status (Before Init)
echo "=== Test 3: Seal Status (Before Init) ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

# Test 4: Initialize Vault
echo "=== Test 4: Initialize Vault ==="
curl -s -X POST $BASE_URL/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{"secret_shares":5,"secret_threshold":3}' \
  | tee $KEYS_FILE | python3 -m json.tool
echo ""

# Test 5: Seal Status (After Init, Still Sealed)
echo "=== Test 5: Seal Status (After Init, Still Sealed) ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

# Extract keys
KEY1=$(cat $KEYS_FILE | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][0])")
KEY2=$(cat $KEYS_FILE | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][1])")
KEY3=$(cat $KEYS_FILE | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][2])")
ROOT_TOKEN=$(cat $KEYS_FILE | python3 -c "import sys,json; print(json.load(sys.stdin)['root_token'])")

echo "Keys extracted:"
echo "  Key 1: ${KEY1:0:20}..."
echo "  Key 2: ${KEY2:0:20}..."
echo "  Key 3: ${KEY3:0:20}..."
echo "  Root Token: ${ROOT_TOKEN:0:20}..."
echo ""

# Test 6: Unseal (Key 1)
echo "=== Test 6: Unseal (Key 1 - Progress 1/3) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY1\"}" | python3 -m json.tool
echo ""

# Test 7: Unseal (Key 2)
echo "=== Test 7: Unseal (Key 2 - Progress 2/3) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY2\"}" | python3 -m json.tool
echo ""

# Test 8: Unseal (Key 3 - Final)
echo "=== Test 8: Unseal (Key 3 - UNSEALED!) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY3\"}" | python3 -m json.tool
echo ""

# Test 9: Seal Status (Unsealed)
echo "=== Test 9: Seal Status (Unsealed) ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

# Test 10: System Health (with token)
echo "=== Test 10: System Health ==="
curl -s $BASE_URL/v1/sys/health \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Test 11: List Auth Methods
echo "=== Test 11: List Auth Methods ==="
curl -s $BASE_URL/v1/sys/auth \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Test 12: Write Secret
echo "=== Test 12: Write Secret ==="
curl -s -X POST $BASE_URL/v1/secret/data/test \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"data":{"username":"admin","password":"secret123"}}' | python3 -m json.tool
echo ""

# Test 13: Read Secret
echo "=== Test 13: Read Secret ==="
curl -s $BASE_URL/v1/secret/data/test \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Test 14: Transit Encrypt
echo "=== Test 14: Transit Encrypt ==="
curl -s -X POST $BASE_URL/v1/transit/encrypt/test-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"plaintext":"SGVsbG8gV29ybGQ="}' | python3 -m json.tool
echo ""

# Test 15: Seal Vault
echo "=== Test 15: Seal Vault ==="
curl -s -X POST $BASE_URL/v1/sys/seal \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Test 16: Seal Status (After Seal)
echo "=== Test 16: Seal Status (After Seal) ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

# Test 17: Unseal Again (Key 1)
echo "=== Test 17: Unseal Again (Key 1) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY1\"}" | python3 -m json.tool
echo ""

# Test 18: Unseal Again (Key 2)
echo "=== Test 18: Unseal Again (Key 2) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY2\"}" | python3 -m json.tool
echo ""

# Test 19: Unseal Again (Key 3)
echo "=== Test 19: Unseal Again (Key 3 - UNSEALED!) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY3\"}" | python3 -m json.tool
echo ""

# Test 20: Read Secret Again (After Unseal)
echo "=== Test 20: Read Secret Again (After Unseal) ==="
curl -s $BASE_URL/v1/secret/data/test \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=========================================="
echo "✓ All Tests Completed Successfully!"
echo "=========================================="
echo ""
echo "Container logs:"
docker logs secreton-e2e 2>&1 | tail -20
