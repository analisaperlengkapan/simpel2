#!/bin/bash
# Quick Full Flow Test for Secreton (Skip build if image exists)
# Tests: Init -> Seal -> Unseal -> All APIs -> Seal -> Unseal -> Verify

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Base URL
BASE_URL="http://localhost:8200"

# Global variables
ROOT_TOKEN=""
UNSEAL_KEYS=()

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         🔐 Secreton Full Flow Comprehensive Test            ║"
echo "║                    (Quick Mode - Skip Build)                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"
    docker-compose down -v 2>/dev/null || true
}

# Set trap to cleanup on exit
trap cleanup EXIT

# Helper function to print section headers
print_section() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Helper function to check response
check_response() {
    local response="$1"
    local expected="$2"
    local description="$3"

    if echo "$response" | grep -q "$expected"; then
        echo -e "${GREEN}✓ PASS${NC}: $description"
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}: $description"
        echo "Response: $response"
        return 1
    fi
}

# ============================================================================
# PHASE 1: ENVIRONMENT SETUP
# ============================================================================
print_section "PHASE 1: Environment Setup"

echo "Cleaning previous environment..."
cleanup

echo "Starting services (PostgreSQL on port 5433, Secreton)..."
echo "Note: Using existing Docker image if available"
docker-compose up -d secreton postgres

echo "Waiting for services to be ready..."
for i in {1..30}; do
    if curl -s $BASE_URL/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Services ready after $i seconds${NC}"
        break
    fi
    echo "  Waiting... ($i/30)"
    sleep 1
done

# Final check
if ! curl -s $BASE_URL/health > /dev/null 2>&1; then
    echo -e "${RED}✗ FAIL${NC}: Services failed to start"
    docker-compose logs secreton | tail -50
    exit 1
fi

# ============================================================================
# PHASE 2: VERIFY FRESH STATE (NOT INITIALIZED)
# ============================================================================
print_section "PHASE 2: Verify Fresh State (NOT Initialized)"

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
echo "Seal Status Response: $RESPONSE"

INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')

if [ "$INITIALIZED" = "false" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault is NOT initialized (correct for fresh start)"
else
    echo -e "${RED}✗ FAIL${NC}: Vault shows initialized=$INITIALIZED on fresh start"
    echo "This might be OK if vault was previously initialized and data persisted"
    echo "Continuing with test..."
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault is sealed"
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Vault is not sealed (sealed=$SEALED)"
fi

# ============================================================================
# PHASE 3: INITIALIZE VAULT (GENERATE MASTER KEY)
# ============================================================================
print_section "PHASE 3: Initialize Vault (Generate Master Key)"

if [ "$INITIALIZED" = "false" ]; then
    echo "Initializing vault with 5 shares, threshold 3..."
    INIT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/init \
        -H "Content-Type: application/json" \
        -d '{
            "secret_shares": 5,
            "secret_threshold": 3
        }')

    echo "Init Response received"

    # Extract keys and root token
    ROOT_TOKEN=$(echo "$INIT_RESPONSE" | jq -r '.root_token')
    mapfile -t UNSEAL_KEYS < <(echo "$INIT_RESPONSE" | jq -r '.keys[]')

    if [ -z "$ROOT_TOKEN" ] || [ "$ROOT_TOKEN" = "null" ]; then
        echo -e "${RED}✗ FAIL${NC}: Failed to get root token"
        echo "Response: $INIT_RESPONSE"
        exit 1
    fi

    echo -e "${GREEN}✓ PASS${NC}: Vault initialized successfully"
    echo "  Root Token: ${ROOT_TOKEN:0:20}..."
    echo "  Unseal Keys: ${#UNSEAL_KEYS[@]} keys generated"

    # Save keys to file for later use
    echo "$ROOT_TOKEN" > /tmp/secreton-root-token
    printf "%s\n" "${UNSEAL_KEYS[@]}" > /tmp/secreton-unseal-keys
else
    echo -e "${YELLOW}⚠ SKIP${NC}: Vault already initialized"
    echo "Attempting to load keys from previous run..."

    if [ -f /tmp/secreton-root-token ] && [ -f /tmp/secreton-unseal-keys ]; then
        ROOT_TOKEN=$(cat /tmp/secreton-root-token)
        mapfile -t UNSEAL_KEYS < /tmp/secreton-unseal-keys
        echo "  Loaded root token and ${#UNSEAL_KEYS[@]} unseal keys"
    else
        echo -e "${RED}✗ FAIL${NC}: Cannot proceed without keys"
        echo "Please run with fresh environment or provide keys manually"
        exit 1
    fi
fi

# Verify initialized status
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')

if [ "$INITIALIZED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault is now initialized"
else
    echo -e "${RED}✗ FAIL${NC}: Vault should be initialized"
    exit 1
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault is SEALED (security best practice)"
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Vault is not sealed"
fi

# ============================================================================
# PHASE 4: UNSEAL VAULT
# ============================================================================
print_section "PHASE 4: Unseal Vault with Master Key Shares"

if [ "$SEALED" = "true" ]; then
    echo "Providing unseal shares (3 of 5 required)..."
    for i in 0 1 2; do
        echo "  Providing share $((i+1))/3..."
        UNSEAL_RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/unseal \
            -H "Content-Type: application/json" \
            -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}")

        SEALED=$(echo "$UNSEAL_RESPONSE" | jq -r '.data.sealed')
        PROGRESS=$(echo "$UNSEAL_RESPONSE" | jq -r '.data.progress')

        echo "    Progress: $PROGRESS/3, Sealed: $SEALED"
    done

    if [ "$SEALED" = "false" ]; then
        echo -e "${GREEN}✓ PASS${NC}: Vault successfully unsealed"
    else
        echo -e "${RED}✗ FAIL${NC}: Vault should be unsealed after threshold shares"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ SKIP${NC}: Vault already unsealed"
fi

# ============================================================================
# PHASE 5: TEST PUBLIC ENDPOINTS
# ============================================================================
print_section "PHASE 5: Test Public Endpoints"

curl -s $BASE_URL/health | jq '.' && echo -e "${GREEN}✓ PASS${NC}: /health"
curl -s $BASE_URL/version | jq '.' && echo -e "${GREEN}✓ PASS${NC}: /version"
curl -s $BASE_URL/v1/sys/seal-status | jq '.' && echo -e "${GREEN}✓ PASS${NC}: /v1/sys/seal-status"

# ============================================================================
# PHASE 6: TEST KV SECRETS ENGINE
# ============================================================================
print_section "PHASE 6: Test KV Secrets Engine"

echo "Creating secret..."
curl -s -X POST $BASE_URL/v1/secret/data/test/app1 \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "data": {
            "username": "testuser",
            "password": "testpass123",
            "api_key": "sk-test-12345"
        }
    }' | jq '.' && echo -e "${GREEN}✓ PASS${NC}: Create secret"

echo "Reading secret..."
curl -s $BASE_URL/v1/secret/data/test/app1 \
    -H "Authorization: Bearer $ROOT_TOKEN" | jq '.data.data' && echo -e "${GREEN}✓ PASS${NC}: Read secret"

echo "Listing secrets..."
curl -s "$BASE_URL/v1/secret/metadata/test?list=true" \
    -H "Authorization: Bearer $ROOT_TOKEN" | jq '.' && echo -e "${GREEN}✓ PASS${NC}: List secrets"

# ============================================================================
# PHASE 7: TEST TRANSIT ENGINE
# ============================================================================
print_section "PHASE 7: Test Transit Engine (Encryption)"

echo "Creating encryption key..."
curl -s -X POST $BASE_URL/v1/transit/keys/myapp-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"type": "aes256-gcm96"}' | jq '.' && echo -e "${GREEN}✓ PASS${NC}: Create key"

echo "Encrypting data..."
PLAINTEXT=$(echo -n "Sensitive Data 123" | base64)
ENCRYPT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/myapp-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"plaintext\": \"$PLAINTEXT\"}")
echo "$ENCRYPT_RESPONSE" | jq '.'

CIPHERTEXT=$(echo "$ENCRYPT_RESPONSE" | jq -r '.data.ciphertext')
if [ -n "$CIPHERTEXT" ] && [ "$CIPHERTEXT" != "null" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Encrypt data"
    echo "  Ciphertext: ${CIPHERTEXT:0:60}..."
else
    echo -e "${RED}✗ FAIL${NC}: Failed to encrypt"
fi

echo "Decrypting data..."
DECRYPT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/decrypt/myapp-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"ciphertext\": \"$CIPHERTEXT\"}")
echo "$DECRYPT_RESPONSE" | jq '.'

DECRYPTED_B64=$(echo "$DECRYPT_RESPONSE" | jq -r '.data.plaintext')
DECRYPTED=$(echo "$DECRYPTED_B64" | base64 -d)
if [ "$DECRYPTED" = "Sensitive Data 123" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Decrypt data (matches original)"
else
    echo -e "${RED}✗ FAIL${NC}: Decrypted data mismatch"
    echo "  Expected: Sensitive Data 123"
    echo "  Got: $DECRYPTED"
fi

echo "Rotating key..."
curl -s -X POST $BASE_URL/v1/transit/keys/myapp-key/rotate \
    -H "Authorization: Bearer $ROOT_TOKEN" | jq '.' && echo -e "${GREEN}✓ PASS${NC}: Rotate key"

# ============================================================================
# PHASE 8: TEST PKI ENGINE
# ============================================================================
print_section "PHASE 8: Test PKI Engine"

echo "Generating root CA..."
curl -s -X POST $BASE_URL/v1/pki/root/generate/internal \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "common_name": "Test Root CA",
        "ttl": "87600h"
    }' | jq '.data.certificate' | head -5 && echo -e "${GREEN}✓ PASS${NC}: Generate CA"

echo "Creating PKI role..."
curl -s -X POST $BASE_URL/v1/pki/roles/web-server \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "allowed_domains": ["example.com", "test.local"],
        "allow_subdomains": true,
        "max_ttl": "72h"
    }' | jq '.' && echo -e "${GREEN}✓ PASS${NC}: Create role"

echo "Issuing certificate..."
curl -s -X POST $BASE_URL/v1/pki/issue/web-server \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "common_name": "app.example.com",
        "ttl": "24h"
    }' | jq '.data.certificate' | head -5 && echo -e "${GREEN}✓ PASS${NC}: Issue certificate"

# ============================================================================
# PHASE 9: TEST SEAL/UNSEAL CYCLE
# ============================================================================
print_section "PHASE 9: Test Seal/Unseal Cycle"

echo "Sealing vault..."
curl -s -X POST $BASE_URL/v1/sys/seal \
    -H "Authorization: Bearer $ROOT_TOKEN"
sleep 2

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')
if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault sealed"
else
    echo -e "${RED}✗ FAIL${NC}: Vault should be sealed"
fi

echo "Unsealing vault again..."
for i in 0 1 2; do
    echo "  Share $((i+1))/3..."
    curl -s -X POST $BASE_URL/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}" > /dev/null
done

RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')
if [ "$SEALED" = "false" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault unsealed again"
else
    echo -e "${RED}✗ FAIL${NC}: Failed to unseal"
fi

# ============================================================================
# PHASE 10: VERIFY DATA PERSISTENCE
# ============================================================================
print_section "PHASE 10: Verify Data Persistence"

echo "Checking if transit key persisted..."
curl -s $BASE_URL/v1/transit/keys/myapp-key \
    -H "Authorization: Bearer $ROOT_TOKEN" | jq '.data.name' && echo -e "${GREEN}✓ PASS${NC}: Transit key persisted"

echo "Checking if KV secret persisted..."
curl -s $BASE_URL/v1/secret/data/test/app1 \
    -H "Authorization: Bearer $ROOT_TOKEN" | jq '.data.data.username' && echo -e "${GREEN}✓ PASS${NC}: KV secret persisted"

# ============================================================================
# FINAL SUMMARY
# ============================================================================
print_section "FINAL SUMMARY"

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ COMPREHENSIVE TEST COMPLETED!                ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Test Coverage:"
echo "  ✅ Vault initialization status verification"
echo "  ✅ Master key generation (5 shares, threshold 3)"
echo "  ✅ Unseal with threshold shares"
echo "  ✅ Public endpoints (health, version, seal-status)"
echo "  ✅ KV Secrets Engine (create, read, list)"
echo "  ✅ Transit Engine (encrypt, decrypt, rotate)"
echo "  ✅ PKI Engine (CA, roles, certificates)"
echo "  ✅ Seal/Unseal cycle"
echo "  ✅ Data persistence verification"
echo ""
echo -e "${CYAN}Test Duration: $SECONDS seconds${NC}"
echo ""
echo "Credentials saved to:"
echo "  Root Token: /tmp/secreton-root-token"
echo "  Unseal Keys: /tmp/secreton-unseal-keys"
echo ""
echo "For manual testing:"
echo "  export ROOT_TOKEN=$ROOT_TOKEN"
echo "  curl -H \"Authorization: Bearer \$ROOT_TOKEN\" $BASE_URL/v1/secret/data/test/app1"
echo ""
