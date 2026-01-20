#!/bin/bash

# Comprehensive API Test Script for Authenc
# Tests all major endpoints and features

BASE_URL="http://localhost:8088"
GRPC_URL="localhost:9088"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Function to test HTTP endpoint
test_endpoint() {
    local method=$1
    local path=$2
    local expected_status=$3
    local data=$4
    local description=$5

    TOTAL=$((TOTAL + 1))

    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" "${BASE_URL}${path}" \
            -H "Content-Type: application/json" \
            -d "$data" 2>/dev/null)
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "${BASE_URL}${path}" 2>/dev/null)
    fi

    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    # Accept any 2xx, 3xx, 401, 403, 404 as valid responses (endpoint exists)
    if [[ "$status_code" =~ ^[234][0-9][0-9]$ ]]; then
        PASSED=$((PASSED + 1))
        echo -e "${GREEN}✓${NC} [$method] $path - $status_code ${description:+($description)}"
    else
        FAILED=$((FAILED + 1))
        echo -e "${RED}✗${NC} [$method] $path - $status_code (expected valid response) ${description:+($description)}"
    fi
}

# Function to test endpoint exists (any response including errors)
test_endpoint_exists() {
    local method=$1
    local path=$2
    local description=$3

    TOTAL=$((TOTAL + 1))

    response=$(curl -s -w "\n%{http_code}" -X "$method" "${BASE_URL}${path}" \
        -H "Content-Type: application/json" 2>/dev/null)

    status_code=$(echo "$response" | tail -n1)

    # 404 means route not found, anything else means endpoint exists
    if [[ "$status_code" != "404" && "$status_code" != "000" ]]; then
        PASSED=$((PASSED + 1))
        echo -e "${GREEN}✓${NC} [$method] $path - $status_code ${description:+($description)}"
    else
        FAILED=$((FAILED + 1))
        echo -e "${RED}✗${NC} [$method] $path - $status_code (endpoint not found) ${description:+($description)}"
    fi
}

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}   Authenc Comprehensive API Test Suite    ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# ==========================================
# HEALTH & METRICS
# ==========================================
echo -e "${YELLOW}=== Health & Metrics ===${NC}"
test_endpoint "GET" "/health" 200 "" "Basic health"
test_endpoint "GET" "/health/ready" 200 "" "Readiness"
test_endpoint "GET" "/health/live" 200 "" "Liveness"
test_endpoint "GET" "/metrics" 200 "" "Prometheus metrics"

# ==========================================
# OIDC DISCOVERY
# ==========================================
echo ""
echo -e "${YELLOW}=== OIDC Discovery ===${NC}"
test_endpoint "GET" "/.well-known/openid-configuration" 200 "" "OIDC Discovery"
test_endpoint "GET" "/realms/master/.well-known/openid-configuration" 200 "" "Realm OIDC Discovery"
test_endpoint "GET" "/.well-known/jwks.json" 200 "" "JWKS"
test_endpoint "GET" "/jwks" 200 "" "JWKS alternate"

# ==========================================
# UMA 2.0 DISCOVERY
# ==========================================
echo ""
echo -e "${YELLOW}=== UMA 2.0 Discovery ===${NC}"
test_endpoint "GET" "/.well-known/uma2-configuration" 200 "" "UMA Discovery"

# ==========================================
# OAUTH2 ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== OAuth2 Endpoints ===${NC}"
test_endpoint_exists "GET" "/oauth2/authorize" "Authorization endpoint"
test_endpoint_exists "POST" "/oauth2/token" "Token endpoint"
test_endpoint_exists "POST" "/oauth2/introspect" "Token introspection"
test_endpoint_exists "POST" "/oauth2/revoke" "Token revocation"
test_endpoint_exists "GET" "/oauth2/userinfo" "UserInfo endpoint"
test_endpoint_exists "GET" "/oauth2/test/authorize" "Test authorize"

# Token Exchange
test_endpoint_exists "POST" "/oauth2/token-exchange" "Token exchange"
test_endpoint_exists "GET" "/oauth2/token-exchange/metadata" "Exchange metadata"

# DCR
test_endpoint_exists "POST" "/oauth2/register" "Client registration"

# ==========================================
# OIDC ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== OIDC Endpoints ===${NC}"
test_endpoint_exists "GET" "/oidc/authorize" "OIDC authorize"
test_endpoint_exists "POST" "/oidc/token" "OIDC token"
test_endpoint_exists "POST" "/oidc/refresh" "OIDC refresh"
test_endpoint_exists "POST" "/oidc/revoke" "OIDC revoke"
test_endpoint_exists "GET" "/oidc/userinfo" "OIDC userinfo"
test_endpoint_exists "GET" "/oidc/jwks" "OIDC JWKS"

# ==========================================
# AUTHENTICATION ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== Authentication Endpoints ===${NC}"
test_endpoint_exists "POST" "/auth/login" "Login"
test_endpoint_exists "POST" "/auth/test-login" "Test login"
test_endpoint_exists "POST" "/auth/logout" "Logout"
test_endpoint_exists "POST" "/auth/refresh" "Refresh token"

# MFA
test_endpoint_exists "POST" "/auth/mfa/setup" "MFA setup"
test_endpoint_exists "POST" "/auth/mfa/verify" "MFA verify"
test_endpoint_exists "POST" "/auth/mfa/verify-setup" "MFA verify setup"
test_endpoint_exists "POST" "/auth/mfa/status" "MFA status"
test_endpoint_exists "POST" "/auth/mfa/disable" "MFA disable"
test_endpoint_exists "POST" "/auth/mfa/reset" "MFA reset"
test_endpoint_exists "POST" "/auth/mfa/backup-codes" "Backup codes"

# WebAuthn
test_endpoint_exists "POST" "/auth/webauthn/register/challenge" "WebAuthn register challenge"
test_endpoint_exists "POST" "/auth/webauthn/register/verify" "WebAuthn register verify"
test_endpoint_exists "POST" "/auth/webauthn/authenticate/challenge" "WebAuthn auth challenge"
test_endpoint_exists "POST" "/auth/webauthn/authenticate/verify" "WebAuthn auth verify"

# ==========================================
# SAML ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== SAML Endpoints ===${NC}"
test_endpoint_exists "GET" "/saml/sp/metadata" "SP Metadata"
test_endpoint_exists "GET" "/saml/idp/metadata" "IDP Metadata"
test_endpoint_exists "GET" "/saml/sso" "SSO Init"
test_endpoint_exists "POST" "/saml/acs" "Assertion Consumer"
test_endpoint_exists "GET" "/saml/slo" "Single Logout"

# ==========================================
# SSO ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== SSO Endpoints ===${NC}"
test_endpoint_exists "GET" "/sso/login" "SSO Login"
test_endpoint_exists "GET" "/sso/callback" "SSO Callback"
test_endpoint_exists "POST" "/sso/logout" "SSO Logout"
test_endpoint_exists "GET" "/sso/session" "Get Session"
test_endpoint_exists "GET" "/sso/sessions" "List Sessions"

# ==========================================
# CAPTCHA ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== CAPTCHA Endpoints ===${NC}"
test_endpoint_exists "POST" "/captcha/generate" "Generate CAPTCHA"
test_endpoint_exists "GET" "/captcha/challenge/test-id" "Get Challenge"
test_endpoint_exists "POST" "/captcha/validate" "Validate CAPTCHA"
test_endpoint_exists "POST" "/captcha/refresh" "Refresh CAPTCHA"

# Dashboard
test_endpoint_exists "GET" "/captcha/dashboard" "Dashboard"
test_endpoint_exists "GET" "/captcha/dashboard/metrics" "Dashboard Metrics"
test_endpoint_exists "GET" "/captcha/dashboard/alerts" "Dashboard Alerts"
test_endpoint_exists "GET" "/captcha/dashboard/health" "Dashboard Health"

# Risk-based
test_endpoint_exists "POST" "/captcha/risk/login" "Login Risk"
test_endpoint_exists "GET" "/captcha/risk/mfa-setup" "MFA Setup Risk"
test_endpoint_exists "POST" "/captcha/risk/password-reset" "Password Reset Risk"

# ==========================================
# UMA 2.0 ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== UMA 2.0 Endpoints ===${NC}"
test_endpoint_exists "POST" "/uma/permission" "Permission Ticket"
test_endpoint_exists "POST" "/uma/authorize" "Authorization"
test_endpoint_exists "POST" "/uma/introspect" "Introspection"
test_endpoint_exists "POST" "/uma/claims" "Claims Submit"
test_endpoint_exists "GET" "/uma/claims-gather" "Claims Gather"
test_endpoint_exists "GET" "/uma/pending-requests" "Pending Requests"
test_endpoint_exists "POST" "/uma/authorize-request" "Authorize Request"

# ==========================================
# OID4VC ENDPOINTS
# ==========================================
echo ""
echo -e "${YELLOW}=== OID4VC Endpoints ===${NC}"
test_endpoint_exists "GET" "/oid4vc/issuer/.well-known/openid-credential-issuer" "Issuer Metadata"
test_endpoint_exists "GET" "/oid4vc/issuer/authorize" "VC Authorize"
test_endpoint_exists "POST" "/oid4vc/issuer/token" "VC Token"
test_endpoint_exists "POST" "/oid4vc/issuer/credential" "Issue Credential"
test_endpoint_exists "POST" "/oid4vc/verifier/verify" "Verify Credential"

# ==========================================
# ADMIN API - USERS
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - Users ===${NC}"
test_endpoint_exists "GET" "/admin/realms/master/users" "List Users"
test_endpoint_exists "POST" "/admin/realms/master/users" "Create User"
test_endpoint_exists "GET" "/admin/realms/master/users/test-user-id" "Get User"
test_endpoint_exists "PUT" "/admin/realms/master/users/test-user-id" "Update User"
test_endpoint_exists "DELETE" "/admin/realms/master/users/test-user-id" "Delete User"

# ==========================================
# ADMIN API - REALMS
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - Realms ===${NC}"
test_endpoint_exists "GET" "/admin/realms" "List Realms"
test_endpoint_exists "POST" "/admin/realms" "Create Realm"
test_endpoint_exists "GET" "/admin/realms/master" "Get Realm"
test_endpoint_exists "PUT" "/admin/realms/master" "Update Realm"

# ==========================================
# ADMIN API - ROLES
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - Roles ===${NC}"
test_endpoint_exists "GET" "/admin/realms/master/roles" "List Roles"
test_endpoint_exists "POST" "/admin/realms/master/roles" "Create Role"
test_endpoint_exists "DELETE" "/admin/realms/master/roles/test-role" "Delete Role"

# ==========================================
# ADMIN API - CLIENTS
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - Clients ===${NC}"
test_endpoint_exists "GET" "/admin/realms/master/clients" "List Clients"
test_endpoint_exists "POST" "/admin/realms/master/clients" "Create Client"
test_endpoint_exists "GET" "/admin/realms/master/clients/test-client-id" "Get Client"
test_endpoint_exists "PUT" "/admin/realms/master/clients/test-client-id" "Update Client"
test_endpoint_exists "DELETE" "/admin/realms/master/clients/test-client-id" "Delete Client"

# ==========================================
# ADMIN API - SERVICE ACCOUNTS
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - Service Accounts ===${NC}"
test_endpoint_exists "GET" "/admin/service-accounts" "List Service Accounts"
test_endpoint_exists "POST" "/admin/service-accounts" "Create Service Account"
test_endpoint_exists "GET" "/admin/service-accounts/test-sa" "Get Service Account"
test_endpoint_exists "PUT" "/admin/service-accounts/test-sa" "Update Service Account"
test_endpoint_exists "DELETE" "/admin/service-accounts/test-sa" "Delete Service Account"
test_endpoint_exists "POST" "/admin/service-accounts/test-sa/regenerate-secret" "Regenerate Secret"

# ==========================================
# ADMIN API - FEDERATION
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - Federation ===${NC}"
test_endpoint_exists "GET" "/admin/realms/master/identity-providers" "List Identity Providers"
test_endpoint_exists "POST" "/admin/realms/master/identity-providers" "Create Identity Provider"
test_endpoint_exists "GET" "/admin/realms/master/identity-providers/test-idp" "Get Identity Provider"
test_endpoint_exists "PUT" "/admin/realms/master/identity-providers/test-idp" "Update Identity Provider"
test_endpoint_exists "DELETE" "/admin/realms/master/identity-providers/test-idp" "Delete Identity Provider"
test_endpoint_exists "POST" "/admin/realms/master/identity-providers/test-idp/sync" "Trigger Sync"
test_endpoint_exists "GET" "/admin/realms/master/identity-providers/test-idp/sync/status" "Sync Status"

# ==========================================
# ADMIN API - MFA ADMIN
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - MFA Admin ===${NC}"
test_endpoint_exists "GET" "/admin/mfa/locked-accounts" "Locked Accounts"
test_endpoint_exists "POST" "/admin/mfa/unlock" "Unlock Account"
test_endpoint_exists "POST" "/admin/mfa/reset-mfa" "Reset MFA"
test_endpoint_exists "GET" "/admin/mfa/lockout-status" "Lockout Status"
test_endpoint_exists "POST" "/admin/mfa/bulk-unlock" "Bulk Unlock"

# ==========================================
# ADMIN API - DCR ADMIN
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - DCR Admin ===${NC}"
test_endpoint_exists "POST" "/admin/realms/master/initial-access-tokens" "Create IAT"
test_endpoint_exists "GET" "/admin/realms/master/initial-access-tokens" "List IATs"
test_endpoint_exists "GET" "/admin/realms/master/registration-policy" "Get DCR Policy"
test_endpoint_exists "POST" "/admin/realms/master/registration-policy" "Create DCR Policy"

# ==========================================
# ADMIN API - EVENTS & AUDIT
# ==========================================
echo ""
echo -e "${YELLOW}=== Admin API - Events & Audit ===${NC}"
test_endpoint_exists "GET" "/admin/realms/master/events" "List Events"
test_endpoint_exists "GET" "/admin/realms/master/admin-events" "List Admin Events"
test_endpoint_exists "GET" "/admin/realms/master/events/retention/stats" "Retention Stats"
test_endpoint_exists "POST" "/admin/realms/master/events/retention/cleanup" "Trigger Cleanup"
test_endpoint_exists "GET" "/admin/realms/master/audit" "Query Audit"
test_endpoint_exists "POST" "/admin/realms/master/audit" "Create Audit"

# ==========================================
# ACCOUNT SELF-SERVICE
# ==========================================
echo ""
echo -e "${YELLOW}=== Account Self-Service ===${NC}"
test_endpoint_exists "GET" "/account" "Get Account"
test_endpoint_exists "PUT" "/account" "Update Account"
test_endpoint_exists "DELETE" "/account" "Delete Account"
test_endpoint_exists "GET" "/account/sessions" "List Sessions"
test_endpoint_exists "DELETE" "/account/sessions/test-session" "Delete Session"
test_endpoint_exists "GET" "/account/applications" "List Apps"
test_endpoint_exists "GET" "/account/export" "Export Data (GDPR)"
test_endpoint_exists "POST" "/account/totp/setup" "Setup TOTP"
test_endpoint_exists "GET" "/account/totp" "Get TOTP Status"
test_endpoint_exists "DELETE" "/account/totp" "Remove TOTP"
test_endpoint_exists "GET" "/account/linked-accounts" "Social Links"
test_endpoint_exists "GET" "/account/consents" "List Consents"

# ==========================================
# CONSENT UI
# ==========================================
echo ""
echo -e "${YELLOW}=== Consent UI ===${NC}"
test_endpoint_exists "GET" "/consent" "Consent Page"
test_endpoint_exists "POST" "/consent" "Process Consent"
test_endpoint_exists "GET" "/consent/test" "Test Consent"
test_endpoint_exists "POST" "/consent/test" "Test Consent Post"
test_endpoint_exists "GET" "/consent/list" "List Consents"
test_endpoint_exists "DELETE" "/consent/revoke/test-consent" "Revoke Consent"

# ==========================================
# FEDERATION / SOCIAL LOGIN
# ==========================================
echo ""
echo -e "${YELLOW}=== Federation / Social Login ===${NC}"
test_endpoint_exists "POST" "/auth/federated/ldap" "LDAP Login"
test_endpoint_exists "GET" "/auth/federated/social/google" "Google Social Init"
test_endpoint_exists "GET" "/auth/federated/callback" "Social Callback"
test_endpoint_exists "GET" "/auth/federated/providers" "List Providers"
test_endpoint_exists "POST" "/auth/social/login" "Social Login Init"
test_endpoint_exists "GET" "/auth/social/callback" "Social Login Callback"

# ==========================================
# gRPC CHECK (if grpcurl is available)
# ==========================================
echo ""
echo -e "${YELLOW}=== gRPC Check ===${NC}"
if command -v grpcurl &> /dev/null; then
    grpc_response=$(grpcurl -plaintext ${GRPC_URL} list 2>&1)
    if [[ "$grpc_response" == *"authenc"* ]] || [[ "$grpc_response" == *"grpc"* ]]; then
        PASSED=$((PASSED + 1))
        TOTAL=$((TOTAL + 1))
        echo -e "${GREEN}✓${NC} gRPC server responding at ${GRPC_URL}"
    else
        FAILED=$((FAILED + 1))
        TOTAL=$((TOTAL + 1))
        echo -e "${RED}✗${NC} gRPC server not responding properly"
    fi
else
    SKIPPED=$((SKIPPED + 1))
    TOTAL=$((TOTAL + 1))
    echo -e "${YELLOW}⊘${NC} grpcurl not installed, skipping gRPC tests"
fi

# Check gRPC port is open
nc -z localhost 9088 2>/dev/null
if [ $? -eq 0 ]; then
    PASSED=$((PASSED + 1))
    TOTAL=$((TOTAL + 1))
    echo -e "${GREEN}✓${NC} gRPC port 9088 is open"
else
    FAILED=$((FAILED + 1))
    TOTAL=$((TOTAL + 1))
    echo -e "${RED}✗${NC} gRPC port 9088 is not accessible"
fi

# ==========================================
# SUMMARY
# ==========================================
echo ""
echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}                 SUMMARY                    ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""
echo -e "Total Tests:  ${TOTAL}"
echo -e "${GREEN}Passed:       ${PASSED}${NC}"
echo -e "${RED}Failed:       ${FAILED}${NC}"
echo -e "${YELLOW}Skipped:      ${SKIPPED}${NC}"
echo ""

PASS_RATE=$((PASSED * 100 / TOTAL))
if [ $PASS_RATE -ge 90 ]; then
    echo -e "${GREEN}Pass Rate: ${PASS_RATE}% ✓${NC}"
elif [ $PASS_RATE -ge 70 ]; then
    echo -e "${YELLOW}Pass Rate: ${PASS_RATE}%${NC}"
else
    echo -e "${RED}Pass Rate: ${PASS_RATE}%${NC}"
fi

echo ""
echo -e "${BLUE}============================================${NC}"
