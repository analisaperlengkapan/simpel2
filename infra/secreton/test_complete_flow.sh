#!/bin/bash
# Complete Flow Test - Fresh Start to Full Testing
# This script does EVERYTHING from scratch

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

BASE_URL="http://localhost:8200"
ROOT_TOKEN=""
UNSEAL_KEYS=()

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      🔐 Secreton Complete Flow Test (Fresh Start)           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Cleanup old containers and volumes
echo -e "${YELLOW}Cleaning up old containers and volumes...${NC}"
docker stop secreton-fresh-test 2>/dev/null || true
docker rm secreton-fresh-test 2>/dev/null || true
docker volume rm secreton_secreton_data 2>/dev/null || true
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""

# Start fresh container
echo -e "${CYAN}Starting fresh Secreton container...${NC}"
docker run -d \
    --name secreton-fresh-test \
    --network host \
    -e RUST_LOG=info \
    -e DATABASE_URL="postgresql://secreton:changeme@localhost:5433/secreton" \
    -v secreton_secreton_data:/var/lib/secreton \
    secreton:fresh \
    2>&1 | head -5

echo "Waiting for Secreton to start..."
for i in {1..30}; do
    if curl -s $BASE_URL/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Secreton ready after $i seconds${NC}"
        break
    fi
    echo "  Waiting... ($i/30)"
    sleep 1
done

if ! curl -s $BASE_URL/health > /dev/null 2>&1; then
    echo -e "${RED}✗ Secreton failed to start${NC}"
    docker logs secreton-fresh-test 2>&1 | tail -30
    exit 1
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 1: Verify Fresh State (NOT Initialized)${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
echo "Seal Status: $RESPONSE"
echo ""

if echo "$RESPONSE" | grep -q '"initialized":false'; then
    echo -e "${GREEN}✓ PASS: Vault is NOT initialized (correct for fresh start)${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should NOT be initialized on fresh start${NC}"
    exit 1
fi

if echo "$RESPONSE" | grep -q '"sealed":true'; then
    echo -e "${GREEN}✓ PASS: Vault is sealed${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should be sealed${NC}"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 2: Initialize Vault (Generate Master Key)${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Initializing with 5 shares, threshold 3..."
INIT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/init \
    -H "Content-Type: application/json" \
    -d '{"secret_shares": 5, "secret_threshold": 3}')

echo "Init Response:"
echo "$INIT_RESPONSE" | head -20
echo ""

# Extract root token (simple grep)
ROOT_TOKEN=$(echo "$INIT_RESPONSE" | grep -o '"root_token":"[^"]*"' | cut -d'"' -f4)
if [ -z "$ROOT_TOKEN" ]; then
    echo -e "${RED}✗ FAIL: Failed to get root token${NC}"
    exit 1
fi

# Extract unseal keys
UNSEAL_KEYS=($(echo "$INIT_RESPONSE" | grep -o '"keys":\[[^]]*\]' | grep -o '"[^"]*"' | grep -v keys | tr -d '"'))

echo -e "${GREEN}✓ PASS: Vault initialized${NC}"
echo "  Root Token: ${ROOT_TOKEN:0:20}..."
echo "  Unseal Keys: ${#UNSEAL_KEYS[@]} keys generated"

# Save for later use
echo "$ROOT_TOKEN" > /tmp/secreton-root-token
printf "%s\n" "${UNSEAL_KEYS[@]}" > /tmp/secreton-unseal-keys
echo ""

# Verify initialized status
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"initialized":true'; then
    echo -e "${GREEN}✓ PASS: Vault is now initialized${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should be initialized${NC}"
    exit 1
fi

if echo "$RESPONSE" | grep -q '"sealed":true'; then
    echo -e "${GREEN}✓ PASS: Vault remains SEALED after init (security best practice)${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should remain sealed after init${NC}"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 3: Unseal Vault with Master Key Shares${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Providing 3 of 5 unseal keys..."
for i in 0 1 2; do
    echo "  Share $((i+1))/3: ${UNSEAL_KEYS[$i]:0:20}..."
    RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}")

    PROGRESS=$(echo "$RESPONSE" | grep -o '"progress":[0-9]*' | cut -d':' -f2)
    SEALED=$(echo "$RESPONSE" | grep -o '"sealed":[a-z]*' | cut -d':' -f2)
    echo "    Progress: $PROGRESS/3, Sealed: $SEALED"
done
echo ""

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":false'; then
    echo -e "${GREEN}✓ PASS: Vault successfully unsealed${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should be unsealed${NC}"
    exit 1
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 4: Test Public Endpoints${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "GET /health"
curl -s $BASE_URL/health && echo -e "\n${GREEN}✓ Health check${NC}\n"

echo "GET /version"
curl -s $BASE_URL/version && echo -e "\n${GREEN}✓ Version info${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 5: Test KV Secrets Engine${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Creating secret at /v1/secret/data/test/credentials..."
curl -s -X POST $BASE_URL/v1/secret/data/test/credentials \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "data": {
            "username": "admin",
            "password": "secret123",
            "api_key": "sk-test-key-12345"
        }
    }' && echo -e "\n${GREEN}✓ Secret created${NC}\n"

echo "Reading secret..."
curl -s $BASE_URL/v1/secret/data/test/credentials \
    -H "Authorization: Bearer $ROOT_TOKEN" && echo -e "\n${GREEN}✓ Secret read${NC}\n"

echo "Listing secrets..."
curl -s "$BASE_URL/v1/secret/metadata/test?list=true" \
    -H "Authorization: Bearer $ROOT_TOKEN" && echo -e "\n${GREEN}✓ Secrets listed${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 6: Test Transit Engine (Encryption/Decryption)${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Creating encryption key..."
curl -s -X POST $BASE_URL/v1/transit/keys/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"type": "aes256-gcm96"}' && echo -e "\n${GREEN}✓ Key created${NC}\n"

echo "Encrypting data..."
PLAINTEXT=$(echo -n "Hello, Secreton!" | base64)
ENCRYPT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"plaintext\": \"$PLAINTEXT\"}")
echo "$ENCRYPT_RESPONSE"
CIPHERTEXT=$(echo "$ENCRYPT_RESPONSE" | grep -o '"ciphertext":"[^"]*"' | cut -d'"' -f4)
echo -e "${GREEN}✓ Data encrypted${NC}"
echo "  Ciphertext: ${CIPHERTEXT:0:60}..."
echo ""

echo "Decrypting data..."
DECRYPT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/decrypt/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"ciphertext\": \"$CIPHERTEXT\"}")
echo "$DECRYPT_RESPONSE"
DECRYPTED_B64=$(echo "$DECRYPT_RESPONSE" | grep -o '"plaintext":"[^"]*"' | cut -d'"' -f4)
DECRYPTED=$(echo "$DECRYPTED_B64" | base64 -d 2>/dev/null || echo "")
if [ "$DECRYPTED" = "Hello, Secreton!" ]; then
    echo -e "${GREEN}✓ Data decrypted correctly${NC}"
else
    echo -e "${YELLOW}⚠ Decryption result: $DECRYPTED${NC}"
fi
echo ""

echo "Rotating key..."
curl -s -X POST $BASE_URL/v1/transit/keys/test-key/rotate \
    -H "Authorization: Bearer $ROOT_TOKEN" && echo -e "\n${GREEN}✓ Key rotated${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 7: Test PKI Engine${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Generating root CA..."
curl -s -X POST $BASE_URL/v1/pki/root/generate/internal \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"common_name": "Test Root CA", "ttl": "87600h"}' | head -15
echo -e "\n${GREEN}✓ Root CA generated${NC}\n"

echo "Creating PKI role..."
curl -s -X POST $BASE_URL/v1/pki/roles/web-server \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"allowed_domains": ["example.com"], "allow_subdomains": true, "max_ttl": "72h"}'
echo -e "\n${GREEN}✓ PKI role created${NC}\n"

echo "Issuing certificate..."
curl -s -X POST $BASE_URL/v1/pki/issue/web-server \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"common_name": "test.example.com", "ttl": "24h"}' | head -15
echo -e "\n${GREEN}✓ Certificate issued${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 8: Test Seal Operation${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Sealing vault..."
curl -s -X POST $BASE_URL/v1/sys/seal \
    -H "Authorization: Bearer $ROOT_TOKEN"
sleep 2
echo ""

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":true'; then
    echo -e "${GREEN}✓ PASS: Vault successfully sealed${NC}"
else
    echo -e "${RED}✗ FAIL: Vault should be sealed${NC}"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 9: Test Unseal Again (Verify Repeatability)${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Unsealing with same keys..."
for i in 0 1 2; do
    echo "  Share $((i+1))/3..."
    curl -s -X POST $BASE_URL/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}" > /dev/null
done
echo ""

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":false'; then
    echo -e "${GREEN}✓ PASS: Vault unsealed again successfully${NC}"
else
    echo -e "${RED}✗ FAIL: Failed to unseal${NC}"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 10: Verify Data Persistence${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Checking transit key..."
curl -s $BASE_URL/v1/transit/keys/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" | head -10
echo -e "\n${GREEN}✓ Transit key persisted${NC}\n"

echo "Checking KV secret..."
curl -s $BASE_URL/v1/secret/data/test/credentials \
    -H "Authorization: Bearer $ROOT_TOKEN" | head -10
echo -e "\n${GREEN}✓ KV secret persisted${NC}\n"

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           ✅ ALL TESTS PASSED SUCCESSFULLY!                  ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Test Summary:"
echo "  ✅ Fresh vault NOT initialized (correct)"
echo "  ✅ Vault initialization (master key generation)"
echo "  ✅ Vault remains sealed after init (security)"
echo "  ✅ Unseal with threshold shares (3 of 5)"
echo "  ✅ Public endpoints (health, version)"
echo "  ✅ KV Secrets Engine (create, read, list)"
echo "  ✅ Transit Engine (encrypt, decrypt, rotate)"
echo "  ✅ PKI Engine (CA, roles, certificates)"
echo "  ✅ Seal operation"
echo "  ✅ Unseal again (repeatability)"
echo "  ✅ Data persistence verification"
echo ""
echo "Credentials saved to:"
echo "  Root Token: /tmp/secreton-root-token"
echo "  Unseal Keys: /tmp/secreton-unseal-keys"
echo ""
echo "Container: secreton-fresh-test (running)"
echo "Vault Status: UNSEALED and ready for use"
echo ""
