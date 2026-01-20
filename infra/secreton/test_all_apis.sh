#!/bin/bash
set -e

BASE_URL="http://localhost:8200"
KEYS_FILE="/tmp/vault_keys_api_test.json"

echo "=========================================="
echo "Secreton Complete API Test"
echo "Secret, Transit, KV, PKI APIs"
echo "=========================================="
echo ""

# Start fresh container
echo "=== Setup: Start Container ==="
docker rm -f secreton-api-test 2>/dev/null || true
sudo rm -rf data 2>/dev/null || true
mkdir -p data/raft

docker run -d --name secreton-api-test \
  -p 8200:8200 -p 8201:8201 \
  -v $(pwd)/secreton.toml:/app/secreton.toml:ro \
  -v $(pwd)/data:/var/lib/secreton \
  secreton:fresh

sleep 5
echo "✓ Container started"
echo ""

# Initialize and unseal
echo "=== Setup: Initialize Vault ==="
INIT_RESULT=$(curl -s -X POST $BASE_URL/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{"secret_shares":5,"secret_threshold":3}')

if echo "$INIT_RESULT" | grep -q "already initialized"; then
    echo "⚠️  Vault already initialized, using test token"
    ROOT_TOKEN="test-root-token"
else
    echo "$INIT_RESULT" | python3 -m json.tool > $KEYS_FILE
    KEY1=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][0])")
    KEY2=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][1])")
    KEY3=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][2])")
    ROOT_TOKEN=$(echo "$INIT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['root_token'])")

    echo "Unsealing..."
    curl -s -X POST $BASE_URL/v1/sys/unseal -H "Content-Type: application/json" -d "{\"key\":\"$KEY1\"}" > /dev/null
    curl -s -X POST $BASE_URL/v1/sys/unseal -H "Content-Type: application/json" -d "{\"key\":\"$KEY2\"}" > /dev/null
    curl -s -X POST $BASE_URL/v1/sys/unseal -H "Content-Type: application/json" -d "{\"key\":\"$KEY3\"}" > /dev/null
    echo "✓ Vault unsealed"
fi

echo "Root Token: ${ROOT_TOKEN:0:30}..."
echo ""

# ==========================================
# KV SECRETS ENGINE TESTS
# ==========================================
echo "=========================================="
echo "KV SECRETS ENGINE TESTS"
echo "=========================================="
echo ""

echo "=== KV Test 1: Write Secret ==="
curl -s -X POST $BASE_URL/v1/kv/myapp/database \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "host": "db.example.com",
    "port": 5432,
    "username": "admin",
    "password": "supersecret123"
  }' | python3 -m json.tool
echo ""

echo "=== KV Test 2: Read Secret ==="
curl -s $BASE_URL/v1/kv/myapp/database \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== KV Test 3: Write Nested Secret ==="
curl -s -X POST $BASE_URL/v1/kv/myapp/api/keys \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "api_key": "abc-def-ghi-jkl",
    "api_secret": "xyz-123-456-789",
    "environment": "production"
  }' | python3 -m json.tool
echo ""

echo "=== KV Test 4: List Secrets ==="
curl -s $BASE_URL/v1/kv/myapp \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== KV Test 5: Update Secret ==="
curl -s -X POST $BASE_URL/v1/kv/myapp/database \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "host": "db.example.com",
    "port": 5432,
    "username": "admin",
    "password": "newsecret456"
  }' | python3 -m json.tool
echo ""

echo "=== KV Test 6: Delete Secret ==="
curl -s -X DELETE $BASE_URL/v1/kv/myapp/api/keys \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

# ==========================================
# TRANSIT ENCRYPTION ENGINE TESTS
# ==========================================
echo "======================================"
echo "TRANSIT ENCRYPTION ENGINE TESTS"
echo "=========================================="
echo ""

echo "=== Transit Test 1: Create Encryption Key ==="
curl -s -X POST $BASE_URL/v1/transit/keys/my-app-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "aes256-gcm96"
  }' | python3 -m json.tool
echo ""

echo "=== Transit Test 2: Read Key Info ==="
curl -s $BASE_URL/v1/transit/keys/my-app-key \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Transit Test 3: Encrypt Data ==="
PLAINTEXT=$(echo -n "Hello, Secreton! This is sensitive data." | base64)
ENCRYPT_RESULT=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/my-app-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"plaintext\":\"$PLAINTEXT\"}")
echo "$ENCRYPT_RESULT" | python3 -m json.tool
CIPHERTEXT=$(echo "$ENCRYPT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['ciphertext'])")
echo ""

echo "=== Transit Test 4: Decrypt Data ==="
DECRYPT_RESULT=$(curl -s -X POST $BASE_URL/v1/transit/decrypt/my-app-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"ciphertext\":\"$CIPHERTEXT\"}")
echo "$DECRYPT_RESULT" | python3 -m json.tool
DECRYPTED=$(echo "$DECRYPT_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['plaintext'])")
echo "Decrypted text: $(echo $DECRYPTED | base64 -d)"
echo ""

echo "=== Transit Test 5: Encrypt with Context ==="
CONTEXT=$(echo -n "user-id-12345" | base64)
ENCRYPT_CTX=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/my-app-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"plaintext\":\"$PLAINTEXT\",\"context\":\"$CONTEXT\"}")
echo "$ENCRYPT_CTX" | python3 -m json.tool
CIPHERTEXT_CTX=$(echo "$ENCRYPT_CTX" | python3 -c "import sys,json; print(json.load(sys.stdin)['ciphertext'])")
echo ""

echo "=== Transit Test 6: Decrypt with Context ==="
curl -s -X POST $BASE_URL/v1/transit/decrypt/my-app-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"ciphertext\":\"$CIPHERTEXT_CTX\",\"context\":\"$CONTEXT\"}" | python3 -m json.tool
echo ""

echo "=== Transit Test 7: Rotate Key ==="
curl -s -X POST $BASE_URL/v1/transit/keys/my-app-key/rotate \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Transit Test 8: Rewrap Data (Re-encrypt with new key version) ==="
curl -s -X POST $BASE_URL/v1/transit/rewrap/my-app-key \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"ciphertext\":\"$CIPHERTEXT\"}" | python3 -m json.tool
echo ""

echo "=== Transit Test 9: Generate Random Bytes ==="
curl -s -X POST $BASE_URL/v1/transit/random/32 \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Transit Test 10: Generate Hash ==="
curl -s -X POST $BASE_URL/v1/transit/hash/sha2-256 \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"input\":\"$PLAINTEXT\"}" | python3 -m json.tool
echo ""

echo "=== Transit Test 11: Generate HMAC ==="
curl -s -X POST $BASE_URL/v1/transit/hmac/my-app-key/sha2-256 \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"input\":\"$PLAINTEXT\"}" | python3 -m json.tool
echo ""

echo "=== Transit Test 12: Sign Data ==="
SIGN_RESULT=$(curl -s -X POST $BASE_URL/v1/transit/sign/my-app-key/sha2-256 \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"input\":\"$PLAINTEXT\"}")
echo "$SIGN_RESULT" | python3 -m json.tool
SIGNATURE=$(echo "$SIGN_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('signature',''))" 2>/dev/null || echo "")
echo ""

if [ -n "$SIGNATURE" ]; then
    echo "=== Transit Test 13: Verify Signature ==="
    curl -s -X POST $BASE_URL/v1/transit/verify/my-app-key/sha2-256 \
      -H "X-Vault-Token: $ROOT_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"input\":\"$PLAINTEXT\",\"signature\":\"$SIGNATURE\"}" | python3 -m json.tool
    echo ""
fi

# ==========================================
# SECRET ENGINE (V2) TESTS
# ==========================================
echo "=========================================="
echo "SECRET ENGINE (KV V2) TESTS"
echo "=========================================="
echo ""

echo "=== Secret Test 1: Write Secret V2 ==="
curl -s -X POST $BASE_URL/v1/secret/data/app/config \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "username": "admin",
      "password": "secret123",
      "api_key": "abc-def-ghi"
    }
  }' | python3 -m json.tool
echo ""

echo "=== Secret Test 2: Read Secret V2 ==="
curl -s $BASE_URL/v1/secret/data/app/config \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Secret Test 3: Write Secret V2 (Version 2) ==="
curl -s -X POST $BASE_URL/v1/secret/data/app/config \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "username": "admin",
      "password": "newsecret456",
      "api_key": "xyz-123-456"
    }
  }' | python3 -m json.tool
echo ""

echo "=== Secret Test 4: Read Secret Metadata ==="
curl -s $BASE_URL/v1/secret/metadata/app/config \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Secret Test 5: Read Specific Version ==="
curl -s $BASE_URL/v1/secret/data/app/config?version=1 \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Secret Test 6: List Secrets ==="
curl -s $BASE_URL/v1/secret/metadata/app \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Secret Test 7: Delete Latest Version ==="
curl -s -X DELETE $BASE_URL/v1/secret/data/app/config \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== Secret Test 8: Undelete Version ==="
curl -s -X POST $BASE_URL/v1/secret/undelete/app/config \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"versions": [2]}' | python3 -m json.tool
echo ""

echo "=== Secret Test 9: Destroy Version (Permanent) ==="
curl -s -X POST $BASE_URL/v1/secret/destroy/app/config \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"versions": [1]}' | python3 -m json.tool
echo ""

# ==========================================
# PKI ENGINE TESTS
# ==========================================
echo "=========================================="
echo "PKI ENGINE TESTS"
echo "=========================================="
echo ""

echo "=== PKI Test 1: Generate Root CA ==="
ROOT_CA=$(curl -s -X POST $BASE_URL/v1/pki/root/generate/internal \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "common_name": "Example Root CA",
    "ttl": "87600h",
    "key_type": "rsa",
    "key_bits": 2048
  }')
echo "$ROOT_CA" | python3 -m json.tool
echo ""

echo "=== PKI Test 2: Read CA Certificate ==="
curl -s $BASE_URL/v1/pki/ca/pem \
  -H "X-Vault-Token: $ROOT_TOKEN"
echo ""

echo "=== PKI Test 3: Configure CA URLs ==="
curl -s -X POST $BASE_URL/v1/pki/config/urls \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "issuing_certificates": ["http://vault.example.com:8200/v1/pki/ca"],
    "crl_distribution_points": ["http://vault.example.com:8200/v1/pki/crl"]
  }' | python3 -m json.tool
echo ""

echo "=== PKI Test 4: Create Role ==="
curl -s -X POST $BASE_URL/v1/pki/roles/example-dot-com \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "allowed_domains": ["example.com"],
    "allow_subdomains": true,
    "max_ttl": "72h"
  }' | python3 -m json.tool
echo ""

echo "=== PKI Test 5: Issue Certificate ==="
CERT=$(curl -s -X POST $BASE_URL/v1/pki/issue/example-dot-com \
  -H "X-Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "common_name": "test.example.com",
    "ttl": "24h"
  }')
echo "$CERT" | python3 -m json.tool
SERIAL=$(echo "$CERT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('serial_number',''))" 2>/dev/null || echo "")
echo ""

echo "=== PKI Test 6: Read Certificate ==="
if [ -n "$SERIAL" ]; then
    curl -s $BASE_URL/v1/pki/cert/$SERIAL \
      -H "X-Vault-Token: $ROOT_TOKEN"
    echo ""
fi
echo ""

echo "=== PKI Test 7: List Certificates ==="
curl -s $BASE_URL/v1/pki/certs \
  -H "X-Vault-Token: $ROOT_TOKEN" | python3 -m json.tool
echo ""

echo "=== PKI Test 8: Revoke Certificate ==="
if [ -n "$SERIAL" ]; then
    curl -s -X POST $BASE_URL/v1/pki/revoke \
      -H "X-Vault-Token: $ROOT_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"serial_number\":\"$SERIAL\"}" | python3 -m json.tool
    echo ""
fi

echo "=== PKI Test 9: Read CRL ==="
curl -s $BASE_URL/v1/pki/crl/pem \
  -H "X-Vault-Token: $ROOT_TOKEN"
echo ""

echo "=========================================="
echo "✅ ALL API TESTS COMPLETE!"
echo "=========================================="
echo ""

echo "Summary:"
echo "  ✓ KV Engine: 6 tests"
echo "  ✓ Transit Engine: 13 tests"
echo "  ✓ Secret Engine (V2): 9 tests"
echo "  ✓ PKI Engine: 9 tests"
echo "  ✓ Total: 37 API tests"
echo ""

echo "Container: secreton-api-test (still running)"
echo "To stop: docker rm -f secreton-api-test"

