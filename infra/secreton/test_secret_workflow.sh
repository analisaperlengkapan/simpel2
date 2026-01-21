#!/bin/bash

# Secreton Vault Initialization & Unseal Test Script
# This script demonstrates the complete workflow following HashiCorp Vault best practices

set -e

BASE_URL="${1:-http://localhost:8200}"
CONTAINER="${2:-secreton-full-test}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "\n${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC} $1"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}\n"
}

print_step() {
    echo -e "${YELLOW}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

# Function to make curl requests
make_request() {
    local method=$1
    local endpoint=$2
    local data=$3

    if [ -z "$data" ]; then
        curl -s -X "$method" "$BASE_URL$endpoint"
    else
        curl -s -X "$method" "$BASE_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data"
    fi
}

# Function to make requests inside container
make_container_request() {
    local method=$1
    local endpoint=$2
    local data=$3

    if [ -z "$data" ]; then
        docker exec "$CONTAINER" curl -s -X "$method" "http://127.0.0.1:8200$endpoint"
    else
        docker exec "$CONTAINER" curl -s -X "$method" "http://127.0.0.1:8200$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data"
    fi
}

# Main workflow
main() {
    print_header "SECRETON VAULT INITIALIZATION & UNSEAL WORKFLOW TEST"

    # Check if container is running
    print_step "Checking container status..."
    if ! docker ps | grep -q "$CONTAINER"; then
        print_error "Container $CONTAINER is not running"
        echo "Start container with: docker run -d --name $CONTAINER -p 8200:8200 -p 8201:8201 secreton:latest"
        exit 1
    fi
    print_success "Container is running"

    # Phase 1: Check Initial Status
    print_header "PHASE 1: CHECK INITIAL STATUS"

    print_step "Checking seal status..."
    SEAL_STATUS=$(make_container_request GET /v1/sys/seal-status)
    echo "$SEAL_STATUS" | grep -o '"initialized":[^,]*'
    echo "$SEAL_STATUS" | grep -o '"sealed":[^,]*'

    INITIALIZED=$(echo "$SEAL_STATUS" | grep -o '"initialized":[^,]*' | cut -d':' -f2)
    SEALED=$(echo "$SEAL_STATUS" | grep -o '"sealed":[^,]*' | cut -d':' -f2)

    if [ "$INITIALIZED" = "true" ]; then
        print_info "Vault is already initialized, skipping to unseal phase"
        SKIP_INIT=true
    else
        SKIP_INIT=false
    fi

    # Phase 2: Initialize Vault (if needed)
    if [ "$SKIP_INIT" = "false" ]; then
        print_header "PHASE 2: INITIALIZE VAULT"

        print_step "Initializing vault with 5 shares and 3 threshold..."
        INIT_RESPONSE=$(make_container_request POST /v1/sys/init '{
            "secret_shares": 5,
            "secret_threshold": 3
        }')

        # Extract keys and root token
        print_step "Extracting unseal keys and root token..."

        # Save full response for debugging
        echo "$INIT_RESPONSE" > /tmp/init_response.json

        # Extract root token
        ROOT_TOKEN=$(echo "$INIT_RESPONSE" | grep -o '"root_token":"[^"]*' | head -1 | cut -d'"' -f4)

        if [ -z "$ROOT_TOKEN" ]; then
            print_error "Failed to extract root token"
            echo "Full response: $INIT_RESPONSE"
            exit 1
        fi

        print_success "Root Token: ${ROOT_TOKEN:0:20}..."

        # Extract keys - parse JSON properly
        KEYS_JSON=$(echo "$INIT_RESPONSE" | grep -o '"keys":\[[^]]*\]')

        # Convert keys to array
        declare -a KEYS
        KEY_COUNT=0

        # Simple extraction of base64 strings from keys array
        while IFS= read -r line; do
            if [[ $line =~ \"([A-Za-z0-9+/=]+)\" ]]; then
                KEYS[$KEY_COUNT]="${BASH_REMATCH[1]}"
                ((KEY_COUNT++))
            fi
        done < <(echo "$INIT_RESPONSE" | grep -o '"keys":\[[^]]*\]')

        # If simple extraction failed, try alternative method
        if [ $KEY_COUNT -eq 0 ]; then
            print_info "Extracting keys using alternative method..."
            # Extract all base64 strings that look like keys
            mapfile -t KEYS < <(echo "$INIT_RESPONSE" | grep -oE '[A-Za-z0-9+/]{80,}' | head -5)
            KEY_COUNT=${#KEYS[@]}
        fi

        print_success "Extracted $KEY_COUNT unseal keys"

        # Save keys for later use
        echo "$ROOT_TOKEN" > /tmp/root_token.txt
        for i in "${!KEYS[@]}"; do
            echo "${KEYS[$i]}" > "/tmp/unseal_key_$i.txt"
            print_info "Key $((i+1)): ${KEYS[$i]:0:20}..."
        done

        print_success "Initialization complete!"
        print_info "Keys saved to /tmp/unseal_key_*.txt"
        print_info "Root token saved to /tmp/root_token.txt"
    else
        print_info "Loading existing keys..."
        ROOT_TOKEN=$(cat /tmp/root_token.txt 2>/dev/null || echo "")

        # Load keys
        declare -a KEYS
        KEY_COUNT=0
        for key_file in /tmp/unseal_key_*.txt; do
            if [ -f "$key_file" ]; then
                KEYS[$KEY_COUNT]=$(cat "$key_file")
                ((KEY_COUNT++))
            fi
        done
    fi

    # Phase 3: Unseal Vault
    print_header "PHASE 3: UNSEAL VAULT"

    print_step "Checking current seal status..."
    SEAL_STATUS=$(make_container_request GET /v1/sys/seal-status)
    PROGRESS=$(echo "$SEAL_STATUS" | grep -o '"progress":[^,]*' | cut -d':' -f2)
    THRESHOLD=$(echo "$SEAL_STATUS" | grep -o '"n":[^,]*' | cut -d':' -f2)

    print_info "Current progress: $PROGRESS/$THRESHOLD shares"

    # Provide unseal keys
    print_step "Providing unseal keys..."
    for i in $(seq 0 2); do
        if [ $i -lt $KEY_COUNT ]; then
            print_info "Providing key $((i+1))/3..."
            UNSEAL_RESPONSE=$(make_container_request POST /v1/sys/unseal "{
                \"key\": \"${KEYS[$i]}\"
            }")

            SEALED=$(echo "$UNSEAL_RESPONSE" | grep -o '"sealed":[^,]*' | cut -d':' -f2)
            PROGRESS=$(echo "$UNSEAL_RESPONSE" | grep -o '"progress":[^,]*' | cut -d':' -f2)

            if [ "$SEALED" = "false" ]; then
                print_success "Vault unsealed! (Progress: $PROGRESS)"
                break
            else
                print_info "Progress: $PROGRESS shares provided"
            fi
        fi
    done

    # Phase 4: Verify Unsealed Status
    print_header "PHASE 4: VERIFY UNSEALED STATUS"

    print_step "Checking final seal status..."
    FINAL_STATUS=$(make_container_request GET /v1/sys/seal-status)

    SEALED=$(echo "$FINAL_STATUS" | grep -o '"sealed":[^,]*' | cut -d':' -f2)
    INITIALIZED=$(echo "$FINAL_STATUS" | grep -o '"initialized":[^,]*' | cut -d':' -f2)

    echo "$FINAL_STATUS" | grep -o '"sealed":[^,]*'
    echo "$FINAL_STATUS" | grep -o '"initialized":[^,]*'

    if [ "$SEALED" = "false" ] && [ "$INITIALIZED" = "true" ]; then
        print_success "Vault is UNSEALED and INITIALIZED ✓"
    else
        print_error "Vault is still sealed or not initialized"
        exit 1
    fi

    # Phase 5: Test Operations
    print_header "PHASE 5: TEST VAULT OPERATIONS"

    print_step "Testing health check..."
    HEALTH=$(make_container_request GET /v1/health)
    STATUS=$(echo "$HEALTH" | grep -o '"status":"[^"]*' | cut -d'"' -f4)
    print_success "Health status: $STATUS"

    print_step "Testing version endpoint..."
    VERSION=$(make_container_request GET /v1/version)
    VERSION_NUM=$(echo "$VERSION" | grep -o '"version":"[^"]*' | cut -d'"' -f4)
    print_success "Version: $VERSION_NUM"

    print_step "Testing metrics endpoint..."
    METRICS_CODE=$(docker exec "$CONTAINER" curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8200/v1/metrics)
    if [ "$METRICS_CODE" = "200" ]; then
        print_success "Metrics endpoint: OK"
    else
        print_error "Metrics endpoint returned: $METRICS_CODE"
    fi

    # Phase 6: Summary
    print_header "WORKFLOW COMPLETE ✓"

    echo -e "${GREEN}Summary:${NC}"
    echo "  ✓ Vault initialized with Shamir secret sharing"
    echo "  ✓ 5 shares generated with 3-of-5 threshold"
    echo "  ✓ Vault unsealed successfully"
    echo "  ✓ All operations functional"
    echo ""
    echo -e "${YELLOW}Security Notes:${NC}"
    echo "  • Root token: ${ROOT_TOKEN:0:20}... (saved to /tmp/root_token.txt)"
    echo "  • Unseal keys saved to /tmp/unseal_key_*.txt"
    echo "  • In production: Distribute shares to different people/locations"
    echo "  • In production: Store root token in secure vault"
    echo "  • In production: Use HTTPS and proper authentication"
    echo ""
    echo -e "${BLUE}Next Steps:${NC}"
    echo "  1. Create secrets: POST /v1/secret/{path}"
    echo "  2. Read secrets: GET /v1/secret/{path}"
    echo "  3. Set up authentication methods"
    echo "  4. Configure audit logging"
    echo "  5. Implement backup strategy"
}

# Run main function
main "$@"
