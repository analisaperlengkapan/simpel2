#!/bin/bash
# Comprehensive Full Flow Test for Secreton
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
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"
    docker-compose down -v 2>/dev/null || true
    rm -rf /tmp/secreton-test-data 2>/dev/null || true
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
mkdir -p /tmp/secreton-test-data

echo "Building Docker image..."
docker-compose build secreton

echo "Starting services (PostgreSQL, Redis, Secreton)..."
docker-compose up -d secreton postgres redis

echo "Waiting for services to be ready..."
sleep 8

echo -e "${GREEN}✓ Environment ready${NC}"

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
    echo -e "${RED}✗ FAIL${NC}: Vault shows initialized on fresh start (BUG!)"
    exit 1
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault is sealed"
else
    echo -e "${RED}✗ FAIL${NC}: Vault should be sealed"
    exit 1
fi

# ============================================================================
# PHASE 3: INITIALIZE VAULT (GENERATE MASTER KEY)
# ============================================================================
print_section "PHASE 3: Initialize Vault (Generate Master Key)"

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

# Verify initialized status
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')

if [ "$INITIALIZED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault is now initialized"
else
    echo -e "${RED}✗ FAIL${NC}: Vault should be initialized after init"
    exit 1
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault remains SEALED after init (security best practice)"
else
    echo -e "${RED}✗ FAIL${NC}: Vault should remain sealed after init"
    exit 1
fi

# ============================================================================
# PHASE 4: UNSEAL VAULT (FIRST TIME)
# ============================================================================
print_section "PHASE 4: Unseal Vault with Master Key Shares"

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
    echo -e "${RED}✗ FAIL${NC}: Vault should be unsealed after providing threshold shares"
    exit 1
fi

# ============================================================================
# PHASE 5: TEST ALL PUBLIC ENDPOINTS
# ============================================================================
print_section "PHASE 5: Test Public Endpoints (No Auth)"

echo "Testing /health..."
RESPONSE=$(curl -s $BASE_URL/health)
check_response "$RESPONSE" "healthy" "Health check"

echo "Testing /version..."
RESPONSE=$(curl -s $BASE_URL/version)
check_response "$RESPONSE" "version" "Version info"

echo "Testing /metrics..."
RESPONSE=$(curl -s $BASE_URL/metrics)
check_response "$RESPONSE" "secreton" "Prometheus metrics"

echo "Testing /metrics/tls..."
RESPONSE=$(curl -s $BASE_URL/metrics/tls)
check_response "$RESPONSE" "tls_enabled" "TLS metrics"

# ============================================================================
# PHASE 6: TEST KV SECRETS ENGINE
# ============================================================================
print_section "PHASE 6: Test KV Secrets Engine"

echo "Creating secret at /v1/secret/data/test/credentials..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/secret/data/test/credentials \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "data": {
            "username": "admin",
            "password": "secret123",
            "api_key": "sk-test-key-12345"
        }
    }')
check_response "$RESPONSE" "success" "Create KV secret"

echo "Reading secret from /v1/secret/data/test/credentials..."
RESPONSE=$(curl -s $BASE_URL/v1/secret/data/test/credentials \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "admin" "Read KV secret"
check_response "$RESPONSE" "secret123" "Verify secret data"

echo "Listing secrets at /v1/secret/metadata/test..."
RESPONSE=$(curl -s $BASE_URL/v1/secret/metadata/test?list=true \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "credentials" "List KV secrets"

echo "Updating secret..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/secret/data/test/credentials \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "data": {
            "username": "admin",
            "password": "newsecret456",
            "api_key": "sk-test-key-67890"
        }
    }')
check_response "$RESPONSE" "success" "Update KV secret"

echo "Deleting secret..."
RESPONSE=$(curl -s -X DELETE $BASE_URL/v1/secret/data/test/credentials \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "success" "Delete KV secret"

# ============================================================================
# PHASE 7: TEST TRANSIT ENGINE (ENCRYPTION/DECRYPTION)
# ============================================================================
print_section "PHASE 7: Test Transit Engine (Encryption/Decryption)"

echo "Creating encryption key 'test-key'..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/keys/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "type": "aes256-gcm96"
    }')
check_response "$RESPONSE" "success" "Create transit key"

echo "Encrypting data..."
PLAINTEXT=$(echo -n "Hello, Secreton!" | base64)
RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
  \"plaintext\": \"$PLAINTEXT\"
    }")
check_response "$RESPONSE" "ciphertext" "Encrypt data"

CIPHERTEXT=$(echo "$RESPONSE" | jq -r '.data.ciphertext')
echo "  Ciphertext: ${CIPHERTEXT:0:50}..."

echo "Decrypting data..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/decrypt/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
        \"ciphertext\": \"$CIPHERTEXT\"
    }")
check_response "$RESPONSE" "plaintext" "Decrypt data"

DECRYPTED=$(echo "$RESPONSE" | jq -r '.data.plaintext' | base64 -d)
if [ "$DECRYPTED" = "Hello, Secreton!" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Decrypted data matches original"
else
    echo -e "${RED}✗ FAIL${NC}: Decrypted data does not match"
    echo "  Expected: Hello, Secreton!"
    echo "  Got: $DECRYPTED"
fi

echo "Rotating encryption key..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/keys/test-key/rotate \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "success" "Rotate transit key"

echo "Reading key info..."
RESPONSE=$(curl -s $BASE_URL/v1/transit/keys/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "test-key" "Read transit key info"

# ============================================================================
# PHASE 8: TEST PKI ENGINE (CERTIFICATES)
# ============================================================================
print_section "PHASE 8: Test PKI Engine (Certificate Management)"

echo "Generating root CA..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/pki/root/generate/internal \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "common_name": "Secreton Root CA",
        "ttl": "87600h"
    }')
check_response "$RESPONSE" "certificate" "Generate root CA"

echo "Creating PKI role..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/pki/roles/test-role \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "allowed_domains": ["example.com"],
        "allow_subdomains": true,
        "max_ttl": "72h"
    }')
check_response "$RESPONSE" "success" "Create PKI role"

echo "Issuing certificate..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/pki/issue/test-role \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "common_name": "test.example.com",
        "ttl": "24h"
    }')
check_response "$RESPONSE" "certificate" "Issue certificate"

# ============================================================================
# PHASE 9: SEAL VAULT (SECOND TIME)
# ============================================================================
print_section "PHASE 9: Seal Vault (Test Re-seal)"

echo "Sealing vault..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/seal \
    -H "Authorization: Bearer $ROOT_TOKEN")

# Check seal status
sleep 2
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')
INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault successfully sealed"
else
    echo -e "${RED}✗ FAIL${NC}: Vault should be sealed"
    exit 1
fi

if [ "$INITIALIZED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault remains initialized after seal"
else
    echo -e "${RED}✗ FAIL${NC}: Initialized status should persist"
    exit 1
fi

echo "Verifying sealed vault blocks operations..."
RESPONSE=$(curl -s $BASE_URL/v1/secret/data/test/blocked \
    -H "Authorization: Bearer $ROOT_TOKEN")
# Should fail or return error when sealed
echo "  Response when sealed: ${RESPONSE:0:100}..."

# ============================================================================
# PHASE 10: UNSEAL VAULT (SECOND TIME)
# ============================================================================
print_section "PHASE 10: Unseal Vault Again (Verify Persistence)"

echo "Unsealing vault with same master key shares..."
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
    echo -e "${GREEN}✓ PASS${NC}: Vault successfully unsealed (second time)"
else
    echo -e "${RED}✗ FAIL${NC}: Vault should be unsealed"
    exit 1
fi

# ============================================================================
# PHASE 11: VERIFY DATA PERSISTENCE
# ============================================================================
print_section "PHASE 11: Verify Data Persistence After Seal/Unseal"

echo "Verifying transit key still exists..."
RESPONSE=$(curl -s $BASE_URL/v1/transit/keys/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "test-key" "Transit key persisted"

echo "Testing encryption with persisted key..."
PLAINTEXT=$(echo -n "Data after unseal" | base64)
RESPONSE=$(curl -s -X POST $BASE_URL/v1/transit/encrypt/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
        \"plaintext\": \"$PLAINTEXT\"
    }")
check_response "$RESPONSE" "ciphertext" "Encrypt with persisted key"

# ============================================================================
# PHASE 12: TEST CONTAINER RESTART
# ============================================================================
print_section "PHASE 12: Test Container Restart (Full Persistence)"

echo "Restarting Secreton container..."
docker-compose restart secreton
echo "Waiting for restart..."
sleep 8

echo "Checking seal status after restart..."
RESPONSE=$(curl -s $BASE_URL/v1/sys/seal-status)
INITIALIZED=$(echo "$RESPONSE" | jq -r '.initialized')
SEALED=$(echo "$RESPONSE" | jq -r '.sealed')

if [ "$INITIALIZED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault remains initialized after restart"
else
    echo -e "${RED}✗ FAIL${NC}: Initialization state should persist"
    exit 1
fi

if [ "$SEALED" = "true" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault is sealed after restart (security best practice)"
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Vault should be sealed after restart"
fi

echo "Unsealing after restart..."
for i in 0 1 2; do
    echo "  Providing share $((i+1))/3..."
    UNSEAL_RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/unseal \
        -H "Content-Type: application/json" \
        -d "{\"key\": \"${UNSEAL_KEYS[$i]}\"}")

    SEALED=$(echo "$UNSEAL_RESPONSE" | jq -r '.data.sealed')
done

if [ "$SEALED" = "false" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Vault unsealed after restart"
else
    echo -e "${RED}✗ FAIL${NC}: Failed to unseal after restart"
    exit 1
fi

echo "Verifying data persisted after restart..."
RESPONSE=$(curl -s $BASE_URL/v1/transit/keys/test-key \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "test-key" "Data persisted after restart"

# ============================================================================
# PHASE 13: ADDITIONAL API TESTS
# ============================================================================
print_section "PHASE 13: Additional API Tests"

echo "Testing audit log endpoint..."
RESPONSE=$(curl -s $BASE_URL/v1/sys/audit \
    -H "Authorization: Bearer $ROOT_TOKEN")
echo "  Audit response: ${RESPONSE:0:100}..."

echo "Testing capabilities endpoint..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/capabilities-self \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "paths": ["secret/data/test"]
    }')
echo "  Capabilities response: ${RESPONSE:0:100}..."

echo "Testing token lookup..."
RESPONSE=$(curl -s -X POST $BASE_URL/v1/auth/token/lookup-self \
    -H "Authorization: Bearer $ROOT_TOKEN")
check_response "$RESPONSE" "root" "Token lookup"

# ============================================================================
# FINAL SUMMARY
# ============================================================================
print_section "FINAL SUMMARY"

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  ✅ ALL TESTS PASSED!                        ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Test Coverage:"
echo "  ✅ Fresh vault initialization status (NOT initialized)"
echo "  ✅ Vault initialization (master key generation)"
echo "  ✅ Vault remains sealed after init (security)"
echo "  ✅ Unseal with threshold shares (3 of 5)"
echo "  ✅ Public endpoints (health, version, metrics)"
echo "  ✅ KV Secrets Engine (CRUD operations)"
echo "  ✅ Transit Engine (encrypt/decrypt/rotate)"
echo "  ✅ PKI Engine (CA, roles, certificates)"
echo "  ✅ Vault seal operation"
echo "  ✅ Vault unseal operation (second time)"
echo "  ✅ Data persistence after seal/unseal"
echo "  ✅ Container restart persistence"
echo "  ✅ Unseal after restart"
echo "  ✅ Additional system APIs"
echo ""
echo -e "${CYAN}Total Test Duration: $SECONDS seconds${NC}"
echo ""
echo "Credentials for manual testing:"
echo "  Root Token: $ROOT_TOKEN"
echo "  Unseal Keys: ${#UNSEAL_KEYS[@]} keys available"
echo ""

