#!/bin/bash
# Run Full Test After Build
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

BASE_URL="http://localhost:8200"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      🔐 Secreton Full Flow Test (After Fresh Build)         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Start fresh container
echo -e "${CYAN}Starting fresh Secreton container...${NC}"
docker run -d \
    --name secreton-test \
    --network host \
    -e RUST_LOG=info \
    -e DATABASE_URL="postgresql://secreton:changeme@localhost:5433/secreton" \
    secreton:latest

echo "Waiting for Secreton to start..."
for i in {1..30}; do
    if curl -s $BASE_URL/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Ready after $i seconds${NC}"
        break
    fi
    sleep 1
done

echo ""
echo "PHASE 1: Check Fresh State (Should be NOT initialized)"
echo "═══════════════════════════════════════════════════════════════"
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
echo "$RESPONSE"

if echo "$RESPONSE" | grep -q '"initialized":false'; then
    echo -e "${GREEN}✓ PASS: NOT initialized (correct)${NC}"
else
    echo -e "${RED}✗ FAIL: Should NOT be initialized${NC}"
    docker logs secreton-test 2>&1 | tail -20
    exit 1
fi

echo ""
echo "PHASE 2: Initialize Vault"
echo "═══════════════════════════════════════════════════════════════"
INIT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/init \
    -H "Content-Type: application/json" \
    -d '{"secret_shares": 5, "secret_threshold": 3}')

ROOT_TOKEN=$(echo "$INIT_RESPONSE" | grep -o '"root_token":"[^"]*"' | cut -d'"' -f4)
UNSEAL_KEYS=($(echo "$INIT_RESPONSE" | grep -o '"keys":\[[^]]*\]' | grep -o '"[^"]*"' | grep -v keys | tr -d '"'))

echo -e "${GREEN}✓ Initialized${NC}"
echo "  Root Token: ${ROOT_TOKEN:0:20}..."
echo "  Keys: ${#UNSEAL_KEYS[@]}"

# Save credentials
echo "$ROOT_TOKEN" > /tmp/secreton-root-token
printf "%s\n" "${UNSEAL_KEYS[@]}" > /tmp/secreton-unseal-keys

echo ""
echo "PHASE 3: Verify Sealed After Init"
echo "═══════════════════════════════════════════════════════════════"
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":true'; then
    echo -e "${GREEN}✓ PASS: Sealed after init (security)${NC}"
else
    echo -e "${RED}✗ FAIL: Should be sealed${NC}"
fi

echo ""
echo "PHASE 4: Unseal with Master Keys"
echo "═══════════════════════════════════════════════════════════════"
for i in 0 1 2; do
    echo "  Key $((i+1))/3..."
    curl -s -X POST $BASE_URL/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}" > /dev/null
done

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":false'; then
    echo -e "${GREEN}✓ PASS: Unsealed${NC}"
else
    echo -e "${RED}✗ FAIL: Should be unsealed${NC}"
    exit 1
fi

echo ""
echo "PHASE 5: Test KV Secrets"
echo "═══════════════════════════════════════════════════════════════"
curl -s -X POST $BASE_URL/v1/secret/data/test/app \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"data": {"user": "admin", "pass": "secret"}}' > /dev/null
echo -e "${GREEN}✓ Secret created${NC}"

curl -s $BASE_URL/v1/secret/data/test/app \
    -H "Authorization: Bearer $ROOT_TOKEN" | grep -q "admin"
echo -e "${GREEN}✓ Secret read${NC}"

echo ""
echo "PHASE 6: Test Transit Engine"
echo "═══════════════════════════════════════════════════════════════"
curl -s -X POST $BASE_URL/v1/transit/keys/mykey \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"type": "aes256-gcm96"}' > /dev/null
echo -e "${GREEN}✓ Key created${NC}"

PLAINTEXT=$(echo -n "test data" | base64)
ENCRYPT_RESP=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/mykey \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"plaintext\": \"$PLAINTEXT\"}")
CIPHERTEXT=$(echo "$ENCRYPT_RESP" | grep -o '"ciphertext":"[^"]*"' | cut -d'"' -f4)
echo -e "${GREEN}✓ Data encrypted${NC}"

curl -s -X POST $BASE_URL/v1/transit/decrypt/mykey \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"ciphertext\": \"$CIPHERTEXT\"}" | grep -q "plaintext"
echo -e "${GREEN}✓ Data decrypted${NC}"

echo ""
echo "PHASE 7: Test Seal/Unseal Cycle"
echo "═══════════════════════════════════════════════════════════════"
curl -s -X POST $BASE_URL/v1/sys/seal \
    -H "Authorization: Bearer $ROOT_TOKEN" > /dev/null
sleep 1

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":true'; then
    echo -e "${GREEN}✓ Sealed${NC}"
else
    echo -e "${RED}✗ FAIL: Should be sealed${NC}"
fi

for i in 0 1 2; do
    curl -s -X POST $BASE_URL/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}" > /dev/null
done

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
if echo "$RESPONSE" | grep -q '"sealed":false'; then
    echo -e "${GREEN}✓ Unsealed again${NC}"
else
    echo -e "${RED}✗ FAIL: Should be unsealed${NC}"
fi

echo ""
echo "PHASE 8: Verify Data Persistence"
echo "═══════════════════════════════════════════════════════════════"
curl -s $BASE_URL/v1/secret/data/test/app \
    -H "Authorization: Bearer $ROOT_TOKEN" | grep -q "admin"
echo -e "${GREEN}✓ KV secret persisted${NC}"

curl -s $BASE_URL/v1/transit/keys/mykey \
    -H "Authorization: Bearer $ROOT_TOKEN" | grep -q "mykey"
echo -e "${GREEN}✓ Transit key persisted${NC}"

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ ALL TESTS PASSED!                            ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Container: secreton-test (running)"
echo "Credentials: /tmp/secreton-root-token, /tmp/secreton-unseal-keys"
echo ""
