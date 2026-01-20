#!/bin/bash

# ============================================
# Authenc Comprehensive API Test Suite v2
# ============================================
# Tests all endpoints based on actual router configuration
# Reference: handlers/mod.rs

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

# Function to test endpoint exists (any response including errors)
test_endpoint_exists() {
    local method=$1
    local path=$2
    local description=$3

    TOTAL=$((TOTAL + 1))

    response=$(curl -s -w "\n%{http_code}" -X "$method" "${BASE_URL}${path}" \
        -H "Content-Type: application/json" 2>/dev/null)

    body=$(echo "$response" | head -n -1)
    status_code=$(echo "$response" | tail -n1)

    # Check if 404 is from route (empty body) vs handler (JSON error body)
    if [[ "$status_code" == "404" ]]; then
        # If body contains "error" key, it's a valid handler response (resource not found)
        if echo "$body" | grep -q '"error"' 2>/dev/null; then
            PASSED=$((PASSED + 1))
            echo -e "${GREEN}✓${NC} [$method] $path - $status_code (resource not found) ${description:+($description)}"
        else
            FAILED=$((FAILED + 1))
            echo -e "${RED}✗${NC} [$method] $path - $status_code (route not found) ${description:+($description)}"
        fi
    elif [[ "$status_code" != "000" ]]; then
        PASSED=$((PASSED + 1))
        echo -e "${GREEN}✓${NC} [$method] $path - $status_code ${description:+($description)}"
    else
        FAILED=$((FAILED + 1))
        echo -e "${RED}✗${NC} [$method] $path - $status_code (connection failed) ${description:+($description)}"
    fi
}

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}   Authenc Comprehensive API Test Suite v2 ${NC}"
echo -e "${BLUE}   Based on actual router configuration    ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# ==========================================
# HEALTH & METRICS
# Based on: create_health_routes()
# ==========================================
echo -e "${YELLOW}=== Health & Metrics ===${NC}"
test_endpoint_exists "GET" "/health" "Basic health"
test_endpoint_exists "GET" "/ready" "Readiness"
test_endpoint_exists "GET" "/live" "Liveness"
test_endpoint_exists "GET" "/metrics" "Prometheus metrics"
test_endpoint_exists "GET" "/health/metrics" "Health with metrics"

# ==========================================
# DISCOVERY ENDPOINTS
# Based on: Router main config
# ==========================================
echo ""
echo -e "${YELLOW}=== Discovery Endpoints ===${NC}"
test_endpoint_exists "GET" "/.well-known/jwks.json" "JWKS"
test_endpoint_exists "GET" "/.well-known/uma2-configuration" "UMA Discovery"
test_endpoint_exists "GET" "/.well-known/openid_configuration" "OIDC Discovery (alt)"
test_endpoint_exists "GET" "/.well-known/oauth-token-exchange" "Token Exchange Metadata"

# ==========================================
# OAUTH2 ENDPOINTS
# Based on: oauth2_comprehensive module
# ==========================================
echo ""
echo -e "${YELLOW}=== OAuth2 Endpoints ===${NC}"
test_endpoint_exists "GET" "/oauth2/authorize" "Authorization endpoint"
test_endpoint_exists "POST" "/oauth2/token" "Token endpoint"
test_endpoint_exists "POST" "/oauth2/introspect" "Token introspection"
test_endpoint_exists "POST" "/oauth2/revoke" "Token revocation"
test_endpoint_exists "GET" "/oauth2/userinfo" "UserInfo endpoint"
test_endpoint_exists "GET" "/oauth2/jwks" "OAuth2 JWKS"
test_endpoint_exists "GET" "/oauth2/authorize/test" "Test authorize"
test_endpoint_exists "POST" "/oauth2/token/test" "Test token"
test_endpoint_exists "POST" "/oauth2/token/exchange" "Token exchange"
test_endpoint_exists "POST" "/oauth2/register" "Client registration (DCR)"

# ==========================================
# OIDC ENDPOINTS
# Based on: oidc_ed25519 module
# ==========================================
echo ""
echo -e "${YELLOW}=== OIDC Endpoints ===${NC}"
test_endpoint_exists "GET" "/oidc/authorize" "OIDC authorize"
test_endpoint_exists "POST" "/oidc/token" "OIDC token"
test_endpoint_exists "POST" "/oidc/refresh" "OIDC refresh"
test_endpoint_exists "POST" "/oidc/revoke" "OIDC revoke"
test_endpoint_exists "GET" "/oidc/userinfo" "OIDC userinfo"
test_endpoint_exists "GET" "/oidc/jwks" "OIDC JWKS"
test_endpoint_exists "GET" "/oidc/logout" "OIDC SSO Logout"

# ==========================================
# SSO ENDPOINTS
# Based on: sso::create_sso_router()
# ==========================================
echo ""
echo -e "${YELLOW}=== SSO Endpoints ===${NC}"
test_endpoint_exists "GET" "/sso/login" "SSO Login"
test_endpoint_exists "GET" "/sso/callback" "SSO Callback"
test_endpoint_exists "POST" "/sso/logout" "SSO Logout"
test_endpoint_exists "GET" "/sso/session" "Get Session"
test_endpoint_exists "GET" "/sso/sessions" "List Sessions"

# ==========================================
# API V1 AUTH ENDPOINTS
# Based on: api::auth module
# ==========================================
echo ""
echo -e "${YELLOW}=== API v1 Authentication Endpoints ===${NC}"
test_endpoint_exists "POST" "/api/v1/auth/login" "Login"
test_endpoint_exists "POST" "/api/v1/auth/logout" "Logout"
test_endpoint_exists "POST" "/api/v1/auth/refresh" "Refresh token"
test_endpoint_exists "POST" "/api/v1/auth/mfa/setup" "MFA setup"
test_endpoint_exists "POST" "/api/v1/auth/mfa/verify" "MFA verify"
test_endpoint_exists "POST" "/api/v1/auth/mfa/verify-setup" "MFA verify setup"
test_endpoint_exists "GET" "/api/v1/auth/mfa/status" "MFA status"
test_endpoint_exists "POST" "/api/v1/auth/mfa/disable" "MFA disable"
test_endpoint_exists "POST" "/api/v1/auth/mfa/reset" "MFA reset"
test_endpoint_exists "POST" "/api/v1/auth/mfa/backup-codes" "Backup codes"

# ==========================================
# WEBAUTHN ENDPOINTS
# Based on: webauthn::create_webauthn_routes()
# ==========================================
echo ""
echo -e "${YELLOW}=== WebAuthn Endpoints ===${NC}"
test_endpoint_exists "POST" "/api/v1/auth/webauthn/register/begin" "WebAuthn register begin"
test_endpoint_exists "POST" "/api/v1/auth/webauthn/register/complete" "WebAuthn register complete"
test_endpoint_exists "POST" "/api/v1/auth/webauthn/authenticate/begin" "WebAuthn auth begin"
test_endpoint_exists "POST" "/api/v1/auth/webauthn/authenticate/complete" "WebAuthn auth complete"

# ==========================================
# REALM ENDPOINTS
# Based on: api::realm module
# ==========================================
echo ""
echo -e "${YELLOW}=== Realm Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/realms" "List Realms"
test_endpoint_exists "POST" "/api/v1/auth/realms" "Create Realm"
test_endpoint_exists "GET" "/api/v1/auth/realms/master" "Get Realm"
test_endpoint_exists "PUT" "/api/v1/auth/realms/master" "Update Realm"
test_endpoint_exists "DELETE" "/api/v1/auth/realms/master" "Delete Realm"

# ==========================================
# USER ENDPOINTS (realm-scoped)
# Based on: api::user module - /realms/{realm}/users
# ==========================================
echo ""
echo -e "${YELLOW}=== User Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/users" "List Users"
test_endpoint_exists "POST" "/api/v1/auth/realms/master/users" "Create User"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/users/00000000-0000-0000-0000-000000000001" "Get User"
test_endpoint_exists "PUT" "/api/v1/auth/realms/master/users/00000000-0000-0000-0000-000000000001" "Update User"
test_endpoint_exists "DELETE" "/api/v1/auth/realms/master/users/00000000-0000-0000-0000-000000000001" "Delete User"

# ==========================================
# ROLE ENDPOINTS (realm-scoped)
# Based on: api::role module - /realms/{realm}/roles
# ==========================================
echo ""
echo -e "${YELLOW}=== Role Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/roles" "List Roles"
test_endpoint_exists "POST" "/api/v1/auth/realms/master/roles" "Create Role"
test_endpoint_exists "DELETE" "/api/v1/auth/realms/master/roles/test-role" "Delete Role"

# ==========================================
# CLIENT ENDPOINTS (realm-scoped)
# Based on: api::client module - /realms/{realm}/clients
# ==========================================
echo ""
echo -e "${YELLOW}=== Client Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/clients" "List Clients"
test_endpoint_exists "POST" "/api/v1/auth/realms/master/clients" "Create Client"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/clients/test-client" "Get Client"
test_endpoint_exists "PUT" "/api/v1/auth/realms/master/clients/test-client" "Update Client"
test_endpoint_exists "DELETE" "/api/v1/auth/realms/master/clients/test-client" "Delete Client"

# ==========================================
# SERVICE ACCOUNT ENDPOINTS (realm-scoped)
# Based on: api::service_account module - /realms/{realm}/service-accounts
# ==========================================
echo ""
echo -e "${YELLOW}=== Service Account Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/service-accounts" "List Service Accounts"
test_endpoint_exists "POST" "/api/v1/auth/realms/master/service-accounts" "Create Service Account"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/service-accounts/00000000-0000-0000-0000-000000000001" "Get Service Account"
test_endpoint_exists "PUT" "/api/v1/auth/realms/master/service-accounts/00000000-0000-0000-0000-000000000001" "Update Service Account"
test_endpoint_exists "DELETE" "/api/v1/auth/realms/master/service-accounts/00000000-0000-0000-0000-000000000001" "Delete Service Account"

# ==========================================
# PERMISSION ENDPOINTS (realm-scoped)
# Based on: api::permission module - /realms/{realm}/permissions
# ==========================================
echo ""
echo -e "${YELLOW}=== Permission Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/permissions" "List Permissions"
test_endpoint_exists "POST" "/api/v1/auth/realms/master/permissions" "Create Permission"
test_endpoint_exists "DELETE" "/api/v1/auth/realms/master/permissions/test-perm" "Delete Permission"

# ==========================================
# RESOURCE ENDPOINTS
# Based on: api::resource and api::resources modules
# ==========================================
echo ""
echo -e "${YELLOW}=== Resource Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/resources" "List Resources"
test_endpoint_exists "POST" "/api/v1/auth/resources" "Create Resource"
test_endpoint_exists "GET" "/api/v1/auth/resources/test-resource" "Get Resource"
test_endpoint_exists "PUT" "/api/v1/auth/resources/test-resource" "Update Resource"
test_endpoint_exists "DELETE" "/api/v1/auth/resources/test-resource" "Delete Resource"

# ==========================================
# PERMISSION CHECK ENDPOINTS
# Based on: api::permission_check module
# ==========================================
echo ""
echo -e "${YELLOW}=== Permission Check Endpoints ===${NC}"
test_endpoint_exists "POST" "/api/v1/auth/check-permission" "Check Permission"
test_endpoint_exists "POST" "/api/v1/auth/check-permissions" "Check Permissions Batch"

# ==========================================
# ACCOUNT SELF-SERVICE
# Based on: api::account module - /api/v1/auth/account
# ==========================================
echo ""
echo -e "${YELLOW}=== Account Self-Service ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/account" "Get Account"
test_endpoint_exists "PUT" "/api/v1/auth/account" "Update Account"
test_endpoint_exists "DELETE" "/api/v1/auth/account" "Delete Account"
test_endpoint_exists "GET" "/api/v1/auth/account/sessions" "List Sessions"
test_endpoint_exists "DELETE" "/api/v1/auth/account/sessions/test-session" "Delete Session"
test_endpoint_exists "GET" "/api/v1/auth/account/applications" "List Apps"
test_endpoint_exists "GET" "/api/v1/auth/account/social" "Linked Social Accounts"
test_endpoint_exists "GET" "/api/v1/auth/account/consents" "List Consents"
test_endpoint_exists "GET" "/api/v1/auth/account/totp" "TOTP Status"
test_endpoint_exists "GET" "/api/v1/auth/account/export" "Export Account Data"

# ==========================================
# CONSENT ENDPOINTS
# Based on: consent_ui module - /oauth2/consent and /consents
# ==========================================
echo ""
echo -e "${YELLOW}=== Consent Endpoints ===${NC}"
test_endpoint_exists "GET" "/oauth2/consent/test" "Test Consent GET"
test_endpoint_exists "POST" "/oauth2/consent/test" "Test Consent POST"
test_endpoint_exists "GET" "/oauth2/consent" "Consent Page"
test_endpoint_exists "POST" "/oauth2/consent" "Process Consent"
test_endpoint_exists "GET" "/consents" "List User Consents"

# ==========================================
# AUDIT ENDPOINTS
# Based on: api::audit module - routes at /api/v1/auth/realms/{realm}/audit
# ==========================================
echo ""
echo -e "${YELLOW}=== Audit Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/realms/master/audit" "Query Audit Logs"
test_endpoint_exists "POST" "/api/v1/auth/realms/master/audit" "Create Audit Log"

# ==========================================
# EVENTS ENDPOINTS
# Based on: api::events module
# ==========================================
echo ""
echo -e "${YELLOW}=== Events Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/events" "List Events"
test_endpoint_exists "POST" "/api/v1/events" "Create Event"

# ==========================================
# CAPTCHA ENDPOINTS
# Based on: api::captcha module
# ==========================================
echo ""
echo -e "${YELLOW}=== CAPTCHA Endpoints ===${NC}"
test_endpoint_exists "POST" "/api/v1/captcha/generate" "Generate CAPTCHA"
test_endpoint_exists "POST" "/api/v1/captcha/validate" "Validate CAPTCHA"
test_endpoint_exists "POST" "/api/v1/captcha/refresh" "Refresh CAPTCHA"
test_endpoint_exists "GET" "/api/v1/captcha/challenge" "Get Challenge"

# ==========================================
# UMA 2.0 ENDPOINTS
# Based on: uma::create_uma_routes()
# ==========================================
echo ""
echo -e "${YELLOW}=== UMA 2.0 Endpoints ===${NC}"
test_endpoint_exists "POST" "/uma/permission" "Permission Ticket"
test_endpoint_exists "POST" "/uma/authorize" "Authorization"
test_endpoint_exists "POST" "/uma/introspect" "Introspection"
test_endpoint_exists "POST" "/uma/claims" "Claims Submit"
test_endpoint_exists "GET" "/uma/pending-requests" "Pending Requests"
test_endpoint_exists "POST" "/uma/authorize-request" "Authorize Request"

# ==========================================
# OID4VC ENDPOINTS
# Based on: oid4vc::create_oid4vc_router()
# ==========================================
echo ""
echo -e "${YELLOW}=== OID4VC Endpoints ===${NC}"
test_endpoint_exists "GET" "/oid4vc/.well-known/openid-credential-issuer" "Issuer Metadata"
test_endpoint_exists "POST" "/oid4vc/credential" "Issue Credential"
test_endpoint_exists "POST" "/oid4vc/token" "VC Token"
test_endpoint_exists "POST" "/vp/verify" "Verify Presentation"

# ==========================================
# MFA ADMIN ENDPOINTS
# Based on: api::mfa_admin module - routes at /api/v1/admin/mfa/
# ==========================================
echo ""
echo -e "${YELLOW}=== MFA Admin Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/admin/mfa/locked-accounts" "Locked Accounts"
test_endpoint_exists "POST" "/api/v1/admin/mfa/unlock-account" "Unlock Account"
test_endpoint_exists "POST" "/api/v1/admin/mfa/reset-mfa" "Reset MFA"
test_endpoint_exists "GET" "/api/v1/admin/mfa/account-status/00000000-0000-0000-0000-000000000001" "Account Status"
test_endpoint_exists "POST" "/api/v1/admin/mfa/bulk-unlock" "Bulk Unlock"

# ==========================================
# MFA MANAGEMENT ENDPOINTS
# Based on: api::mfa_management module - routes at /api/v1/admin/mfa-management/
# ==========================================
echo ""
echo -e "${YELLOW}=== MFA Management Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/admin/mfa-management/users" "List MFA Users Status"
test_endpoint_exists "GET" "/api/v1/admin/mfa-management/organization/status" "Org MFA Status"
test_endpoint_exists "GET" "/api/v1/admin/mfa-management/organization/policy" "Get MFA Policy"
test_endpoint_exists "GET" "/api/v1/admin/mfa-management/reports/adoption" "MFA Adoption Report"

# ==========================================
# MFA TROUBLESHOOTING ENDPOINTS
# Based on: api::mfa_troubleshooting module - routes at /api/v1/mfa-troubleshooting/
# ==========================================
echo ""
echo -e "${YELLOW}=== MFA Troubleshooting Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/mfa-troubleshooting/admin/health-check" "MFA Health Check"
test_endpoint_exists "GET" "/api/v1/mfa-troubleshooting/admin/system-status" "MFA System Status"
test_endpoint_exists "GET" "/api/v1/mfa-troubleshooting/diagnostics/time-sync-check" "Time Sync Check"
test_endpoint_exists "GET" "/api/v1/mfa-troubleshooting/self-service/guided-setup" "Guided Setup Steps"

# ==========================================
# DCR ADMIN ENDPOINTS
# Based on: dcr_admin module - routes at /api/v1/admin/dcr/
# Routes: /initial-access-tokens, /policies/{realm_id}, /software-statement-issuers
# ==========================================
echo ""
echo -e "${YELLOW}=== DCR Admin Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/admin/dcr/initial-access-tokens" "List Initial Access Tokens"
test_endpoint_exists "POST" "/api/v1/admin/dcr/initial-access-tokens" "Create Initial Access Token"
test_endpoint_exists "GET" "/api/v1/admin/dcr/policies/00000000-0000-0000-0000-000000000001" "Get DCR Policy"
test_endpoint_exists "POST" "/api/v1/admin/dcr/policies/00000000-0000-0000-0000-000000000001" "Create DCR Policy"
test_endpoint_exists "GET" "/api/v1/admin/dcr/software-statement-issuers" "List Software Statement Issuers"

# ==========================================
# FEDERATION ADMIN ENDPOINTS
# Based on: federation_admin module - routes at /api/v1/admin/federation/
# Routes: /identity-providers, /sync/trigger/{alias}, /statistics/{alias}
# ==========================================
echo ""
echo -e "${YELLOW}=== Federation Admin Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/admin/federation/identity-providers" "List Identity Providers"
test_endpoint_exists "POST" "/api/v1/admin/federation/identity-providers" "Create Identity Provider"
test_endpoint_exists "GET" "/api/v1/admin/federation/identity-providers/00000000-0000-0000-0000-000000000001" "Get Identity Provider"
test_endpoint_exists "GET" "/api/v1/admin/federation/sync/status" "Sync Status"

# ==========================================
# FEDERATED LOGIN ENDPOINTS
# Based on: federated_login module - routes at /api/v1/auth/federated/
# Routes: /login/ldap, /login/social/authorize, /login/social/callback, /providers
# ==========================================
echo ""
echo -e "${YELLOW}=== Federated Login Endpoints ===${NC}"
test_endpoint_exists "POST" "/api/v1/auth/federated/login/ldap" "LDAP Login"
test_endpoint_exists "GET" "/api/v1/auth/federated/providers" "List Federated Providers"
test_endpoint_exists "GET" "/api/v1/auth/federated/login/social/authorize?provider_alias=google" "Social Authorize"

# ==========================================
# SOCIAL LOGIN ENDPOINTS
# Based on: social module - routes at /api/v1/auth/social/
# ==========================================
echo ""
echo -e "${YELLOW}=== Social Login Endpoints ===${NC}"
test_endpoint_exists "POST" "/api/v1/auth/social/social/initiate" "Social Login Initiate"
test_endpoint_exists "GET" "/api/v1/auth/social/social/callback?code=test&state=test" "Social Callback"
# Note: /social/providers not implemented in social.rs - only federated_login has providers

# ==========================================
# SPI MANAGEMENT ENDPOINTS
# Based on: spi_management module - routes at /api/v1/admin/spi/
# Routes: /spis, /spis/{name}, /spis/{name}/providers
# ==========================================
echo ""
echo -e "${YELLOW}=== SPI Management Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/admin/spi/spis" "List SPIs"
test_endpoint_exists "GET" "/api/v1/admin/spi/spis/organization" "Get SPI Info"
test_endpoint_exists "GET" "/api/v1/admin/spi/spis/organization/providers" "List SPI Providers"

# ==========================================
# AUTH FLOW ENDPOINTS
# Based on: api::auth_flow module - routes at /api/v1/auth/
# Routes: /flows, /flows/{flow_id}, /flows/{flow_id}/executions
# ==========================================
echo ""
echo -e "${YELLOW}=== Auth Flow Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/auth/flows" "List Auth Flows"
test_endpoint_exists "POST" "/api/v1/auth/flows" "Create Auth Flow"
test_endpoint_exists "GET" "/api/v1/auth/flows/00000000-0000-0000-0000-000000000001" "Get Auth Flow"

# ==========================================
# EVENT LISTENERS ENDPOINTS
# Based on: api::event_listeners module - routes at /api/v1/realms/
# Routes: /{realm}/event-listeners, /{realm}/event-log, etc.
# ==========================================
echo ""
echo -e "${YELLOW}=== Event Listeners Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/realms/master/event-listeners" "List Event Listeners"
test_endpoint_exists "POST" "/api/v1/realms/master/event-listeners" "Register Event Listener"
test_endpoint_exists "GET" "/api/v1/realms/master/event-log" "Query Event Log"
test_endpoint_exists "GET" "/api/v1/realms/master/event-statistics" "Event Statistics"

# ==========================================
# PROTOCOL MAPPERS ENDPOINTS
# Based on: api::protocol_mappers module - routes at /api/v1/realms/
# Routes: /{realm}/clients/{client_id}/mappers, /{realm}/mappers
# ==========================================
echo ""
echo -e "${YELLOW}=== Protocol Mappers Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/realms/master/mappers" "List Realm Mappers"
test_endpoint_exists "POST" "/api/v1/realms/master/mappers" "Create Realm Mapper"
test_endpoint_exists "GET" "/api/v1/realms/master/mapper-statistics" "Mapper Statistics"

# ==========================================
# AUTHENTICATORS ENDPOINTS
# Based on: api::authenticators module - routes at /api/v1/realms/
# Routes: /{realm}/authenticators, /{realm}/authentication-flows/{flow_id}/executions
# ==========================================
echo ""
echo -e "${YELLOW}=== Authenticators Endpoints ===${NC}"
test_endpoint_exists "GET" "/api/v1/realms/master/authenticators" "List Authenticators"
test_endpoint_exists "POST" "/api/v1/realms/master/authenticators" "Register Authenticator"

# ==========================================
# gRPC CHECK
# ==========================================
echo ""
echo -e "${YELLOW}=== gRPC Check ===${NC}"
if command -v grpcurl &> /dev/null; then
    grpc_response=$(grpcurl -plaintext ${GRPC_URL} list 2>&1)
    if [[ "$grpc_response" == *"authenc"* ]] || [[ "$grpc_response" == *"grpc"* ]] || [[ "$grpc_response" != *"error"* ]]; then
        PASSED=$((PASSED + 1))
        TOTAL=$((TOTAL + 1))
        echo -e "${GREEN}✓${NC} gRPC server responding at ${GRPC_URL}"
    else
        FAILED=$((FAILED + 1))
        TOTAL=$((TOTAL + 1))
        echo -e "${RED}✗${NC} gRPC server not responding properly: $grpc_response"
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

if [ $TOTAL -gt 0 ]; then
    PASS_RATE=$((PASSED * 100 / TOTAL))
    if [ $PASS_RATE -ge 90 ]; then
        echo -e "${GREEN}Pass Rate: ${PASS_RATE}% ✓${NC}"
    elif [ $PASS_RATE -ge 70 ]; then
        echo -e "${YELLOW}Pass Rate: ${PASS_RATE}%${NC}"
    else
        echo -e "${RED}Pass Rate: ${PASS_RATE}%${NC}"
    fi
fi

echo ""
echo -e "${BLUE}============================================${NC}"

exit $FAILED
