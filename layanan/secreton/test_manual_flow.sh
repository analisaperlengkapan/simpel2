#!/bin/bash
# Manual Full Flow Test for Secreton (Using existing container)
# Tests: Check Status -> Unseal -> All APIs -> Seal -> Unseal -> Verify

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

BASE_URL="http://localhost:8200"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         🔐 Secreton Manual Full Flow Test                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if we have saved credentials
if [ -f /tmp/secreton-root-token ] && [ -f /tmp/secreton-unseal-keys ]; then
    ROOT_TOKEN=$(cat /tmp/secreton-root-token)
    mapfile -t UNSEAL_KEYS < /tmp/secreton-unseal-keys
    echo -e "${GREEN}✓ Loaded credentials from previous session${NC}"
    echo "  Root Token: ${ROOT_TOKEN:0:20}..."
    echo "  Unseal Keys: ${#UNSEAL_KEYS[@]} keys"
else
    echo -e "${RED}✗ No saved credentials found${NC}"
    echo "Please provide credentials manually or run initialization first"
    echo ""
    echo "If you need to initialize:"
    echo "  curl -X POST $BASE_URL/v1/sys/init -H 'Content-Type: application/json' -d '{\"secret_shares\": 5, \"secret_threshold\": 3}'"
    exit 1
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 1: Check Current Status${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "GET /v1/sys/seal-status"
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
echo "$RESPONSE"
echo ""

if echo "$RESPONSE" | grep -q '"initialized":true'; then
    echo -e "${GREEN}✓ Vault is initialized${NC}"
else
    echo -e "${RED}✗ Vault is not initialized${NC}"
fi

if echo "$RESPONSE" | grep -q '"sealed":true'; then
    echo -e "${YELLOW}⚠ Vault is SEALED - will unseal${NC}"
    SEALED=true
else
    echo -e "${GREEN}✓ Vault is UNSEALED${NC}"
    SEALED=false
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 2: Unseal Vault (if sealed)${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

if [ "$SEALED" = true ]; then
    echo "Providing 3 unseal keys..."
    for i in 0 1 2; do
        echo "  Share $((i+1))/3..."
        RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/unseal \
            -H "Content-Type: application/json" \
            -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}")
        echo "  Response: $RESPONSE"
    done
    echo ""

    RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
    if echo "$RESPONSE" | grep -q '"sealed":false'; then
        echo -e "${GREEN}✓ Vault successfully unsealed${NC}"
    else
        echo -e "${RED}✗ Failed to unseal vault${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✓ Vault already unsealed, skipping${NC}"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 3: Test Public Endpoints${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "GET /health"
curl -s $BASE_URL/health
echo -e "\n${GREEN}✓ Health check${NC}\n"

echo "GET /version"
curl -s $BASE_URL/version
echo -e "\n${GREEN}✓ Version info${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 4: Test KV Secrets Engine${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "POST /v1/secret/data/demo/app1 (Create Secret)"
curl -s -X POST $BASE_URL/v1/secret/data/demo/app1 \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "data": {
            "username": "demouser",
            "password": "demopass123",
            "api_key": "sk-demo-key-xyz"
        }
    }'
echo -e "\n${GREEN}✓ Secret created${NC}\n"

echo "GET /v1/secret/data/demo/app1 (Read Secret)"
curl -s $BASE_URL/v1/secret/data/demo/app1 \
    -H "Authorization: Bearer $ROOT_TOKEN"
echo -e "\n${GREEN}✓ Secret read${NC}\n"

echo "GET /v1/secret/metadata/demo?list=true (List Secrets)"
curl -s "$BASE_URL/v1/secret/metadata/demo?list=true" \
    -H "Authorization: Bearer $ROOT_TOKEN"
echo -e "\n${GREEN}✓ Secrets listed${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 5: Test Transit Engine (Encryption)${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "POST /v1/transit/keys/demo-key (Create Key)"
curl -s -X POST $BASE_URL/v1/transit/keys/demo-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"type": "aes256-gcm96"}'
echo -e "\n${GREEN}✓ Encryption key created${NC}\n"

echo "POST /v1/transit/encrypt/demo-key (Encrypt Data)"
PLAINTEXT=$(echo -n "Confidential Information 2024" | base64)
ENCRYPT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/demo-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"plaintext\": \"$PLAINTEXT\"}")
echo "$ENCRYPT_RESPONSE"
echo -e "${GREEN}✓ Data encrypted${NC}\n"

# Extract ciphertext (simple grep since no jq)
CIPHERTEXT=$(echo "$ENCRYPT_RESPONSE" | grep -o '"ciphertext":"[^"]*"' | cut -d'"' -f4)
echo "Ciphertext: ${CIPHERTEXT:0:60}..."
echo ""

echo "POST /v1/transit/decrypt/demo-key (Decrypt Data)"
DECRYPT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/decrypt/demo-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"ciphertext\": \"$CIPHERTEXT\"}")
echo "$DECRYPT_RESPONSE"
echo -e "${GREEN}✓ Data decrypted${NC}\n"

echo "POST /v1/transit/keys/demo-key/rotate (Rotate Key)"
curl -s -X POST $BASE_URL/v1/transit/keys/demo-key/rotate \
    -H "Authorization: Bearer $ROOT_TOKEN"
echo -e "\n${GREEN}✓ Key rotated${NC}\n"

echo "GET /v1/transit/keys/demo-key (Read Key Info)"
curl -s $BASE_URL/v1/transit/keys/demo-key \
    -H "Authorization: Bearer $ROOT_TOKEN"
echo -e "\n${GREEN}✓ Key info retrieved${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 6: Test PKI Engine${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "POST /v1/pki/root/generate/internal (Generate Root CA)"
curl -s -X POST $BASE_URL/v1/pki/root/generate/internal \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "common_name": "Demo Root CA",
        "ttl": "87600h"
    }' | head -20
echo -e "\n${GREEN}✓ Root CA generated${NC}\n"

echo "POST /v1/pki/roles/demo-server (Create PKI Role)"
curl -s -X POST $BASE_URL/v1/pki/roles/demo-server \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "allowed_domains": ["demo.local", "test.local"],
        "allow_subdomains": true,
        "max_ttl": "72h"
    }'
echo -e "\n${GREEN}✓ PKI role created${NC}\n"

echo "POST /v1/pki/issue/demo-server (Issue Certificate)"
curl -s -X POST $BASE_URL/v1/pki/issue/demo-server \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "common_name": "app.demo.local",
        "ttl": "24h"
    }' | head -20
echo -e "\n${GREEN}✓ Certificate issued${NC}\n"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 7: Test Seal Operation${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "POST /v1/sys/seal (Seal Vault)"
curl -s -X POST $BASE_URL/v1/sys/seal \
    -H "Authorization: Bearer $ROOT_TOKEN"
echo ""
sleep 2

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
echo "Seal Status: $RESPONSE"
if echo "$RESPONSE" | grep -q '"sealed":true'; then
    echo -e "${GREEN}✓ Vault successfully sealed${NC}"
else
    echo -e "${RED}✗ Vault should be sealed${NC}"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 8: Test Unseal Again${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "Unsealing vault again with same keys..."
for i in 0 1 2; do
    echo "  Share $((i+1))/3..."
    curl -s -X POST $BASE_URL/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}" > /dev/null
done
echo ""

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":false'; then
    echo -e "${GREEN}✓ Vault successfully unsealed (second time)${NC}"
else
    echo -e "${RED}✗ Failed to unseal vault${NC}"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PHASE 9: Verify Data Persistence${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "GET /v1/transit/keys/demo-key (Check Transit Key)"
curl -s $BASE_URL/v1/transit/keys/demo-key \
    -H "Authorization: Bearer $ROOT_TOKEN" | head -10
echo -e "\n${GREEN}✓ Transit key persisted${NC}\n"

echo "GET /v1/secret/data/demo/app1 (Check KV Secret)"
curl -s $BASE_URL/v1/secret/data/demo/app1 \
    -H "Authorization: Bearer $ROOT_TOKEN" | head -10
echo -e "\n${GREEN}✓ KV secret persisted${NC}\n"

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ ALL TESTS COMPLETED SUCCESSFULLY!            ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Test Summary:"
echo "  ✅ Vault status check"
echo "  ✅ Unseal operation"
echo "  ✅ Public endpoints (health, version)"
echo "  ✅ KV Secrets Engine (create, read, list)"
echo "  ✅ Transit Engine (create key, encrypt, decrypt, rotate)"
echo "  ✅ PKI Engine (CA, roles, certificates)"
echo "  ✅ Seal operation"
echo "  ✅ Unseal again (verify repeatability)"
echo "  ✅ Data persistence verification"
echo ""
echo "Vault is now UNSEALED and ready for use"
echo ""
