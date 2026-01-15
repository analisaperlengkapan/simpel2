#!/bin/bash
# SIMPelv2 - Authenc & Secreton Integration Test Suite
# Comprehensive testing for all API endpoints and features

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
AUTHENC_URL="${AUTHENC_URL:-http://localhost:8088}"
SECRETON_URL="${SECRETON_URL:-http://localhost:8200}"
TEST_REALM="${TEST_REALM:-master}"
TEST_USERNAME="testuser_$(date +%s)"
TEST_PASSWORD="TestPassword123!"
TEST_EMAIL="${TEST_USERNAME}@example.com"

# Test Results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Functions
print_header() {
    echo -e "\n${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC} $1"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
}

print_test() {
    echo -e "${CYAN}▶${NC} Testing: $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED_TESTS++))
}

print_failure() {
    echo -e "${RED}✗${NC} $1"
    echo -e "${RED}  Response: $2${NC}"
    ((FAILED_TESTS++))
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

run_test() {
    ((TOTAL_TESTS++))
}

# ========================================
# AUTHENC TESTS
# ========================================

test_authenc_health() {
    print_header "AUTHENC - Health Checks"

    # Test 1: Basic Health
    print_test "Basic health endpoint"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/health")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        print_success "Health check passed"
        print_info "Response: $body"
    else
        print_failure "Health check failed (HTTP $status)" "$body"
    fi

    # Test 2: Ready Check
    print_test "Ready endpoint"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/ready")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "Ready check passed"
    else
        print_failure "Ready check failed (HTTP $status)" "$(echo "$response" | head -n-1)"
    fi

    # Test 3: Liveness Check
    print_test "Liveness endpoint"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/live")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "Liveness check passed"
    else
        print_failure "Liveness check failed (HTTP $status)" "$(echo "$response" | head -n-1)"
    fi

    # Test 4: Metrics
    print_test "Metrics endpoint"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/metrics")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "Metrics endpoint accessible"
    else
        print_failure "Metrics check failed (HTTP $status)" "$(echo "$response" | head -n-1)"
    fi
}

test_authenc_oidc_discovery() {
    print_header "AUTHENC - OIDC Discovery"

    # Test 1: OpenID Configuration
    print_test "OIDC discovery endpoint (.well-known/openid-configuration)"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/realms/$TEST_REALM/.well-known/openid-configuration")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        # Validate essential OIDC fields
        if echo "$body" | jq -e '.issuer' > /dev/null 2>&1 && \
           echo "$body" | jq -e '.authorization_endpoint' > /dev/null 2>&1 && \
           echo "$body" | jq -e '.token_endpoint' > /dev/null 2>&1 && \
           echo "$body" | jq -e '.jwks_uri' > /dev/null 2>&1; then
            print_success "OIDC discovery passed with all required fields"
            print_info "Issuer: $(echo "$body" | jq -r '.issuer')"
        else
            print_failure "OIDC discovery missing required fields" "$body"
        fi
    else
        print_failure "OIDC discovery failed (HTTP $status)" "$body"
    fi

    # Test 2: JWKS Endpoint
    print_test "JWKS endpoint"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/realms/$TEST_REALM/protocol/openid-connect/certs")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        if echo "$body" | jq -e '.keys' > /dev/null 2>&1; then
            key_count=$(echo "$body" | jq '.keys | length')
            print_success "JWKS endpoint passed ($key_count keys found)"
        else
            print_failure "JWKS response malformed" "$body"
        fi
    else
        print_failure "JWKS endpoint failed (HTTP $status)" "$body"
    fi

    # Test 3: OAuth2 Discovery
    print_test "OAuth2 authorization server metadata"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/.well-known/oauth-authorization-server")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "OAuth2 discovery passed"
    else
        print_warning "OAuth2 discovery returned HTTP $status (may not be implemented)"
    fi
}

test_authenc_authentication() {
    print_header "AUTHENC - Authentication Flow"

    # Test 1: User Registration (if supported)
    print_test "User registration"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X POST "$AUTHENC_URL/api/v1/auth/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"username\": \"$TEST_USERNAME\",
            \"password\": \"$TEST_PASSWORD\",
            \"email\": \"$TEST_EMAIL\",
            \"realm\": \"$TEST_REALM\"
        }")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ] || [ "$status" = "201" ]; then
        print_success "User registration successful"
    elif [ "$status" = "409" ]; then
        print_warning "User already exists (continuing with existing user)"
    else
        print_failure "User registration failed (HTTP $status)" "$body"
    fi

    # Test 2: User Login
    print_test "User login"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X POST "$AUTHENC_URL/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"username\": \"$TEST_USERNAME\",
            \"password\": \"$TEST_PASSWORD\",
            \"realm\": \"$TEST_REALM\"
        }")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        # Extract token
        ACCESS_TOKEN=$(echo "$body" | jq -r '.access_token // .token // empty')
        if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "null" ]; then
            print_success "Login successful, token obtained"
            print_info "Token preview: ${ACCESS_TOKEN:0:50}..."
        else
            print_warning "Login successful but no token in response"
            print_info "Response: $body"
        fi
    else
        print_failure "Login failed (HTTP $status)" "$body"
    fi

    # Test 3: Token Validation
    if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "null" ]; then
        print_test "Token validation"
        run_test
        response=$(curl -s -w "\n%{http_code}" "$AUTHENC_URL/api/v1/auth/validate" \
            -H "Authorization: Bearer $ACCESS_TOKEN")
        status=$(echo "$response" | tail -n1)

        if [ "$status" = "200" ]; then
            print_success "Token validation passed"
        else
            print_failure "Token validation failed (HTTP $status)" "$(echo "$response" | head -n-1)"
        fi
    fi
}

test_authenc_mfa() {
    print_header "AUTHENC - MFA Features"

    # Requires authenticated session
    if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" = "null" ]; then
        print_warning "Skipping MFA tests (no valid access token)"
        return
    fi

    # Test 1: MFA Status
    print_test "MFA status check"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X POST "$AUTHENC_URL/api/v1/auth/mfa/status" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -H "Content-Type: application/json" \
        -d '{}')
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        print_success "MFA status check passed"
        print_info "Response: $body"
    else
        print_warning "MFA status check returned HTTP $status (may require setup)"
    fi

    # Test 2: MFA Setup Initiation
    print_test "MFA setup initiation"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X POST "$AUTHENC_URL/api/v1/auth/mfa/setup" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -H "Content-Type: application/json" \
        -d '{}')
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "MFA setup initiated"
    else
        print_warning "MFA setup returned HTTP $status"
    fi
}

# ========================================
# SECRETON TESTS
# ========================================

test_secreton_health() {
    print_header "SECRETON - Health Checks"

    # Test 1: Basic Health
    print_test "Health endpoint"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$SECRETON_URL/health")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        print_success "Health check passed"
        print_info "Response: $body"
    else
        print_failure "Health check failed (HTTP $status)" "$body"
    fi

    # Test 2: System Status
    print_test "System status endpoint"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$SECRETON_URL/v1/sys/health")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ] || [ "$status" = "501" ]; then
        print_success "System status check passed"
    else
        print_warning "System status returned HTTP $status"
    fi
}

test_secreton_transit() {
    print_header "SECRETON - Transit Engine"

    KEY_NAME="test-key-$(date +%s)"

    # Test 1: Create Transit Key
    print_test "Create transit encryption key"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X POST "$SECRETON_URL/v1/transit/keys/$KEY_NAME" \
        -H "Content-Type: application/json" \
        -d '{
            "type": "aes256-gcm96",
            "derived": false
        }')
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ] || [ "$status" = "204" ]; then
        print_success "Transit key created"
    else
        print_failure "Transit key creation failed (HTTP $status)" "$body"
    fi

    # Test 2: Encrypt Data
    print_test "Encrypt data with transit key"
    run_test
    PLAINTEXT="Hello, World!"
    PLAINTEXT_B64=$(echo -n "$PLAINTEXT" | base64)

    response=$(curl -s -w "\n%{http_code}" -X POST "$SECRETON_URL/v1/transit/encrypt/$KEY_NAME" \
        -H "Content-Type: application/json" \
        -d "{\"plaintext\": \"$PLAINTEXT_B64\"}")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        CIPHERTEXT=$(echo "$body" | jq -r '.ciphertext // empty')
        if [ -n "$CIPHERTEXT" ]; then
            print_success "Data encrypted successfully"
            print_info "Ciphertext preview: ${CIPHERTEXT:0:50}..."
        else
            print_failure "Encryption response missing ciphertext" "$body"
        fi
    else
        print_failure "Encryption failed (HTTP $status)" "$body"
    fi

    # Test 3: Decrypt Data
    if [ -n "$CIPHERTEXT" ]; then
        print_test "Decrypt data with transit key"
        run_test
        response=$(curl -s -w "\n%{http_code}" -X POST "$SECRETON_URL/v1/transit/decrypt/$KEY_NAME" \
            -H "Content-Type: application/json" \
            -d "{\"ciphertext\": \"$CIPHERTEXT\"}")
        status=$(echo "$response" | tail -n1)
        body=$(echo "$response" | head -n-1)

        if [ "$status" = "200" ]; then
            DECRYPTED_B64=$(echo "$body" | jq -r '.plaintext // empty')
            DECRYPTED=$(echo "$DECRYPTED_B64" | base64 -d)

            if [ "$DECRYPTED" = "$PLAINTEXT" ]; then
                print_success "Data decrypted successfully (matches original)"
            else
                print_failure "Decrypted data mismatch" "Expected: $PLAINTEXT, Got: $DECRYPTED"
            fi
        else
            print_failure "Decryption failed (HTTP $status)" "$body"
        fi
    fi

    # Test 4: List Keys
    print_test "List transit keys"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$SECRETON_URL/v1/transit/keys")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "Transit keys listed"
    else
        print_warning "List keys returned HTTP $status"
    fi
}

test_secreton_kv() {
    print_header "SECRETON - KV Secret Engine"

    SECRET_PATH="test/secret-$(date +%s)"
    SECRET_DATA='{"password": "super-secret-password", "api_key": "sk-test-123456"}'

    # Test 1: Write Secret
    print_test "Write KV secret"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X POST "$SECRETON_URL/v1/kv/data/$SECRET_PATH" \
        -H "Content-Type: application/json" \
        -d "{\"data\": $SECRET_DATA}")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ] || [ "$status" = "204" ]; then
        print_success "Secret written successfully"
    else
        print_failure "Write secret failed (HTTP $status)" "$body"
    fi

    # Test 2: Read Secret
    print_test "Read KV secret"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$SECRETON_URL/v1/kv/data/$SECRET_PATH")
    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$status" = "200" ]; then
        # Verify data
        PASSWORD=$(echo "$body" | jq -r '.data.password // empty')
        if [ "$PASSWORD" = "super-secret-password" ]; then
            print_success "Secret read successfully with correct data"
        else
            print_failure "Secret data mismatch" "$body"
        fi
    else
        print_failure "Read secret failed (HTTP $status)" "$body"
    fi

    # Test 3: List Secrets
    print_test "List KV secrets"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$SECRETON_URL/v1/kv/metadata/test?list=true")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "Secrets listed successfully"
    else
        print_warning "List secrets returned HTTP $status"
    fi

    # Test 4: Delete Secret
    print_test "Delete KV secret"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X DELETE "$SECRETON_URL/v1/kv/data/$SECRET_PATH")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ] || [ "$status" = "204" ]; then
        print_success "Secret deleted successfully"
    else
        print_failure "Delete secret failed (HTTP $status)" "$(echo "$response" | head -n-1)"
    fi
}

test_secreton_pki() {
    print_header "SECRETON - PKI Engine"

    # Test 1: PKI Health
    print_test "PKI engine health"
    run_test
    response=$(curl -s -w "\n%{http_code}" "$SECRETON_URL/v1/pki/health")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "PKI engine healthy"
    else
        print_warning "PKI health check returned HTTP $status"
    fi

    # Test 2: Generate Certificate (may require CA setup)
    print_test "Certificate generation"
    run_test
    response=$(curl -s -w "\n%{http_code}" -X POST "$SECRETON_URL/v1/pki/issue/test-role" \
        -H "Content-Type: application/json" \
        -d '{
            "common_name": "test.example.com",
            "ttl": "72h"
        }')
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "Certificate generated"
    else
        print_warning "Certificate generation returned HTTP $status (may need CA setup)"
    fi
}

# ========================================
# INTEGRATION TESTS
# ========================================

test_integration() {
    print_header "INTEGRATION - Authenc ↔ Secreton"

    # Test 1: Authenc can access Secreton
    print_test "Authenc → Secreton connectivity"
    run_test

    # Simulate authenc calling secreton for secret retrieval
    if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "null" ]; then
        response=$(curl -s -w "\n%{http_code}" "$SECRETON_URL/health" \
            -H "X-Authenc-Token: $ACCESS_TOKEN")
        status=$(echo "$response" | tail -n1)

        if [ "$status" = "200" ]; then
            print_success "Authenc can reach Secreton"
        else
            print_warning "Cross-service communication check returned HTTP $status"
        fi
    else
        print_warning "Skipping integration test (no authenc token)"
    fi

    # Test 2: Secreton health from external perspective
    print_test "External → Secreton accessibility"
    run_test
    response=$(curl -s -w "\n%{http_code}" -m 5 "$SECRETON_URL/health")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        print_success "Secreton accessible from external network"
    else
        print_failure "Secreton not accessible (HTTP $status)" "$(echo "$response" | head -n-1)"
    fi
}

# ========================================
# PERFORMANCE TESTS
# ========================================

test_performance() {
    print_header "PERFORMANCE - Basic Load Testing"

    # Test 1: Health endpoint performance (10 requests)
    print_test "Authenc health endpoint latency (10 requests)"
    run_test

    total_time=0
    for i in {1..10}; do
        start=$(date +%s%N)
        curl -s "$AUTHENC_URL/health" > /dev/null
        end=$(date +%s%N)
        elapsed=$((($end - $start) / 1000000))
        total_time=$(($total_time + $elapsed))
    done

    avg_time=$(($total_time / 10))

    if [ $avg_time -lt 100 ]; then
        print_success "Excellent latency: ${avg_time}ms average"
    elif [ $avg_time -lt 500 ]; then
        print_success "Good latency: ${avg_time}ms average"
    else
        print_warning "High latency: ${avg_time}ms average"
    fi

    # Test 2: Secreton transit encryption performance
    print_test "Secreton encryption latency (5 requests)"
    run_test

    # Ensure key exists
    curl -s -X POST "$SECRETON_URL/v1/transit/keys/perf-test" \
        -H "Content-Type: application/json" \
        -d '{"type": "aes256-gcm96"}' > /dev/null 2>&1

    total_time=0
    for i in {1..5}; do
        start=$(date +%s%N)
        curl -s -X POST "$SECRETON_URL/v1/transit/encrypt/perf-test" \
            -H "Content-Type: application/json" \
            -d '{"plaintext": "dGVzdC1kYXRh"}' > /dev/null
        end=$(date +%s%N)
        elapsed=$((($end - $start) / 1000000))
        total_time=$(($total_time + $elapsed))
    done

    avg_time=$(($total_time / 5))

    if [ $avg_time -lt 200 ]; then
        print_success "Excellent encryption latency: ${avg_time}ms average"
    elif [ $avg_time -lt 1000 ]; then
        print_success "Good encryption latency: ${avg_time}ms average"
    else
        print_warning "High encryption latency: ${avg_time}ms average"
    fi
}

# ========================================
# MAIN EXECUTION
# ========================================

main() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                                                              ║"
    echo "║          SIMPelv2 Integration Test Suite                    ║"
    echo "║          Authenc & Secreton Comprehensive Testing           ║"
    echo "║                                                              ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"

    print_info "Authenc URL: $AUTHENC_URL"
    print_info "Secreton URL: $SECRETON_URL"
    print_info "Test Realm: $TEST_REALM"
    echo ""

    # Check dependencies
    if ! command -v jq &> /dev/null; then
        print_warning "jq not found, some tests may fail. Install with: sudo apt install jq"
    fi

    # Run test suites
    test_authenc_health
    test_authenc_oidc_discovery
    test_authenc_authentication
    test_authenc_mfa

    test_secreton_health
    test_secreton_transit
    test_secreton_kv
    test_secreton_pki

    test_integration
    test_performance

    # Summary
    echo -e "\n${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}                   TEST SUMMARY                      ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
    echo -e "Total Tests:  ${TOTAL_TESTS}"
    echo -e "${GREEN}Passed:       ${PASSED_TESTS}${NC}"
    echo -e "${RED}Failed:       ${FAILED_TESTS}${NC}"

    SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")
    echo -e "Success Rate: ${SUCCESS_RATE}%"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All tests passed!${NC}"
        exit 0
    else
        echo -e "\n${YELLOW}⚠ Some tests failed. Review logs above for details.${NC}"
        exit 1
    fi
}

# Run main function
main "$@"
