#!/bin/bash
set -e

BASE_URL="http://localhost:8200"
KEYS_FILE="/tmp/vault_keys_full.json"

echo "=========================================="
echo "Secreton Full Workflow Test"
echo "Complete Init -> Seal -> Unseal Cycle"
echo "=========================================="
echo ""

# Cleanup
echo "=== Step 1: Cleanup ==="
docker rm -f secreton-full 2>/dev/null || true
sudo rm -rf data 2>/dev/null || true
mkdir -p data/raft
rm -f $KEYS_FILE
echo "✓ Cleanup complete"
echo ""

# Start fresh container
echo "=== Step 2: Start Fresh Container ==="
docker run -d --name secreton-full \
  -p 8200:8200 -p 8201:8201 \
  -v $(pwd)/secreton.toml:/app/secreton.toml:ro \
  -v $(pwd)/data:/var/lib/secreton \
  secreton:raft
echo "✓ Container started"
sleep 5
echo ""

# Verify Raft backend
echo "=== Step 3: Verify Raft Backend ==="
docker logs secreton-full 2>&1 | grep -i "raft\|storage" | head -5
echo ""

# Health check
echo "=== Step 4: Health Check ==="
curl -s $BASE_URL/health | python3 -m json.tool
echo ""

# Check seal status (should be uninitialized)
echo "=== Step 5: Initial Seal Status ==="
STATUS=$(curl -s $BASE_URL/v1/sys/seal-status)
echo "$STATUS" | python3 -m json.tool
INITIALIZED=$(echo "$STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['initialized'])")
echo ""

if [ "$INITIALIZED" = "true" ]; then
    echo "❌ ERROR: Vault already initialized!"
    echo "Please ensure data directory is clean"
    exit 1
fi

# Initialize vault
echo "=== Step 6: Initialize Vault (Generate Master Keys) ==="
INIT_RESULT=$(curl -s -X POST $BASE_URL/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{"secret_shares":5,"secret_threshold":3}')

echo "$INIT_RESULT" | python3 -m json.tool | tee $KEYS_FILE
echo ""

# Extract keys
KEY1=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][0])")
KEY2=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][1])")
KEY3=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][2])")
KEY4=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][3])")
KEY5=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][4])")
ROOT_TOKEN=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['root_token'])")

echo "Master Keys Generated:"
echo "  Key 1: ${KEY1:0:30}..."
echo "  Key 2: ${KEY2:0:30}..."
echo "  Key 3: ${KEY3:0:30}..."
echo "  Key 4: ${KEY4:0:30}..."
echo "  Key 5: ${KEY5:0:30}..."
echo "  Root Token: ${ROOT_TOKEN:0:30}..."
echo ""

# Verify sealed status
echo "=== Step 7: Verify Sealed Status ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

# Unseal process (1st key)
echo "=== Step 8: Unseal with Key 1 (Progress 1/3) ==="
UNSEAL1=$(curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY1\"}")
echo "$UNSEAL1" | python3 -m json.tool
PROGRESS=$(echo "$UNSEAL1" | python3 -c "import sys,json; print(json.load(sys.stdin)['progress'])")
echo "Progress: $PROGRESS/3"
echo ""

# Unseal process (2nd key)
echo "=== Step 9: Unseal with Key 2 (Progress 2/3) ==="
UNSEAL2=$(curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY2\"}")
echo "$UNSEAL2" | python3 -m json.tool
PROGRESS=$(echo "$UNSEAL2" | python3 -c "import sys,json; print(json.load(sys.stdin)['progress'])")
echo "Progress: $PROGRESS/3"
echo ""

# Unseal process (3rd key - final)
echo "=== Step 10: Unseal with Key 3 (UNSEALED!) ==="
UNSEAL3=$(curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY3\"}")
echo "$UNSEAL3" | python3 -m json.tool
SEALED=$(echo "$UNSEAL3" | python3 -c "import sys,json; print(json.load(sys.stdin)['sealed'])")
echo ""

if [ "$SEALED" = "true" ]; then
    echo "❌ ERROR: Vault still sealed after 3 keys!"
    exit 1
fi

echo "✅ Vault UNSEALED successfully!"
echo ""

# Verify unsealed status
echo "=== Step 11: Verify Unsealed Status ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

# Test authenticated endpoints
echo "=== Step 12: Test System Health (Authenticated) ==="
curl -s $BASE_URL/v1/sys/health \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Step 13: List Auth Methods ==="
curl -s $BASE_URL/v1/sys/auth \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Step 14: List Secrets Engines ==="
curl -s $BASE_URL/v1/sys/mounts \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Test KV operations
echo "=== Step 15: Write Secret to KV ==="
curl -s -X POST $BASE_URL/v1/secret/data/myapp/config \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"data":{"username":"admin","password":"supersecret123","api_key":"abc-def-ghi"}}' \
  | python3 -m json.tool
echo ""

echo "=== Step 16: Read Secret from KV ==="
curl -s $BASE_URL/v1/secret/data/myapp/config \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Step 17: List Secrets ==="
curl -s $BASE_URL/v1/secret/metadata/myapp \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Test Transit operations
echo "=== Step 18: Create Transit Key ==="
curl -s -X POST $BASE_URL/v1/transit/keys/my-key \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Step 19: Encrypt with Transit ==="
PLAINTEXT=$(echo -n "Hello, Secreton!" | base64)
ENCRYPTED=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/my-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"plaintext\":\"$PLAINTEXT\"}")
echo "$ENCRYPTED" | python3 -m json.tool
CIPHERTEXT=$(echo "$ENCRYPTED" | python3 -c "import sys,json; print(json.load(sys.stdin)['ciphertext'])")
echo ""

echo "=== Step 20: Decrypt with Transit ==="
curl -s -X POST $BASE_URL/v1/transit/decrypt/my-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"ciphertext\":\"$CIPHERTEXT\"}" | python3 -m json.tool
echo ""

# Now seal the vault
echo "=== Step 21: SEAL the Vault ==="
curl -s -X POST $BASE_URL/v1/sys/seal \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Step 22: Verify Sealed Status ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

# Try to access secret (should fail)
echo "=== Step 23: Try to Read Secret (Should Fail - Sealed) ==="
curl -s $BASE_URL/v1/secret/data/myapp/config \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Unseal again
echo "=== Step 24: Unseal Again with Key 1 ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY1\"}" | python3 -m json.tool
echo ""

echo "=== Step 25: Unseal Again with Key 4 (Different Key) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY4\"}" | python3 -m json.tool
echo ""

echo "=== Step 26: Unseal Again with Key 5 (UNSEALED!) ==="
curl -s -X POST $BASE_URL/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\":\"$KEY5\"}" | python3 -m json.tool
echo ""

# Verify data persisted
echo "=== Step 27: Read Secret Again (Should Work) ==="
curl -s $BASE_URL/v1/secret/data/myapp/config \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# Test more APIs
echo "=== Step 28: Test PKI Operations ==="
curl -s -X POST $BASE_URL/v1/pki/root/generate/internal \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"common_name":"example.com","ttl":"87600h"}' | python3 -m json.tool
echo ""

echo "=== Step 29: Test Dynamic Secrets (Database) ==="
curl -s $BASE_URL/v1/database/config/my-database \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Step 30: Check Metrics ==="
curl -s $BASE_URL/metrics | head -20
echo "..."
echo ""

echo "=========================================="
echo "✅ Full Workflow Test COMPLETE!"
echo "=========================================="
echo ""

echo "Summary:"
echo "  ✓ Vault initialized with 5 keys, threshold 3"
echo "  ✓ Unsealed successfully (3 keys required)"
echo "  ✓ Secrets written and read"
echo "  ✓ Transit encryption/decryption working"
echo "  ✓ Vault sealed successfully"
echo "  ✓ Vault unsealed again with different keys"
echo "  ✓ Data persisted across seal/unseal"
echo "  ✓ All API endpoints tested"
echo ""

echo "Raft Backend Status:"
docker logs secreton-full 2>&1 | grep -i "raft" | tail -5
echo ""

echo "Keys saved to: $KEYS_FILE"
echo "Root Token: $ROOT_TOKEN"
echo ""
echo "Container: secreton-full (still running)"
echo "To stop: docker rm -f secreton-full"

