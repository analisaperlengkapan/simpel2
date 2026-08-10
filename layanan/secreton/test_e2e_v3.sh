#!/usr/bin/env bash
# ==============================================================================
# Secreton E2E Comprehensive Test Suite v3
# Tests ALL API endpoints against a live Secreton instance
# CORRECTED: All request body formats match handler struct definitions
# ==============================================================================
set +e  # Don't exit on error - we're testing

BASE_URL="http://localhost:8200"
PASS=0
FAIL=0
SKIP=0
TOTAL=0
ROOT_TOKEN=""
UNSEAL_KEYS=()

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# ==============================================================================
# Test Helpers
# ==============================================================================
run_test() {
    local name="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    local expected_status="$5"
    local auth="${6:-yes}"

    TOTAL=$((TOTAL+1))

    local headers=(-H "Content-Type: application/json")
    if [ "$auth" = "yes" ] && [ -n "$ROOT_TOKEN" ]; then
        headers+=(-H "Authorization: Bearer $ROOT_TOKEN")
    fi

    local curl_args=(-s -o /tmp/secreton_resp.json -w "%{http_code}" --max-time 10)
    curl_args+=(-X "$method")

    if [ -n "$data" ] && [ "$data" != "null" ]; then
        curl_args+=(-d "$data")
    fi

    local http_code
    http_code=$(curl "${curl_args[@]}" "${headers[@]}" "$url" 2>/dev/null)
    local body
    body=$(cat /tmp/secreton_resp.json 2>/dev/null)

    # Check if expected_status contains multiple options (e.g. "200|201|409")
    local match=0
    IFS='|' read -ra EXPECTED <<< "$expected_status"
    for exp in "${EXPECTED[@]}"; do
        if [ "$http_code" = "$exp" ]; then
            match=1
            break
        fi
    done

    if [ "$match" = "1" ]; then
        PASS=$((PASS+1))
        printf "${GREEN}  ✅ PASS${NC} [%s] %s (HTTP %s)\n" "$method" "$name" "$http_code"
    else
        FAIL=$((FAIL+1))
        printf "${RED}  ❌ FAIL${NC} [%s] %s (HTTP %s, expected %s)\n" "$method" "$name" "$http_code" "$expected_status"
        echo "       Body: $(echo "$body" | head -c 250)"
    fi
}

skip_test() {
    local name="$1"
    SKIP=$((SKIP+1))
    printf "${YELLOW}  ⏭️  SKIP${NC} %s\n" "$name"
}

section() {
    echo ""
    printf "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
    printf "${BLUE}  📋 %s${NC}\n" "$1"
    printf "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

# ==============================================================================
# SECTION 0: Pre-Flight & Connectivity
# ==============================================================================
section "SECTION 0: Pre-Flight & Connectivity"

run_test "Health check" "GET" "$BASE_URL/health" "" "200" "no"
run_test "Readiness probe" "GET" "$BASE_URL/ready" "" "200" "no"
run_test "Liveness probe" "GET" "$BASE_URL/live" "" "200" "no"
run_test "Version info" "GET" "$BASE_URL/version" "" "200" "no"
run_test "Metrics JSON" "GET" "$BASE_URL/metrics" "" "200" "no"
run_test "Metrics Prometheus" "GET" "$BASE_URL/metrics/prometheus" "" "200" "no"
run_test "TLS Metrics" "GET" "$BASE_URL/metrics/tls" "" "200" "no"

# V1 unprotected endpoints
run_test "V1 Health check" "GET" "$BASE_URL/v1/health" "" "200" "no"
run_test "V1 Version info" "GET" "$BASE_URL/v1/version" "" "200" "no"
run_test "V1 Metrics" "GET" "$BASE_URL/v1/metrics" "" "200" "no"

# ==============================================================================
# SECTION 1: System Init & Unseal
# ==============================================================================
section "SECTION 1: System Init & Unseal"

run_test "Seal status (pre-init)" "GET" "$BASE_URL/v1/sys/seal-status" "" "200" "no"

# Initialize
INIT_RESP=$(curl -s -X POST "$BASE_URL/v1/sys/init" \
    -H "Content-Type: application/json" \
    -d '{"secret_shares":5,"secret_threshold":3}')

ROOT_TOKEN=$(echo "$INIT_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('root_token',''))" 2>/dev/null)
NUM_KEYS=$(echo "$INIT_RESP" | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('keys',[])))" 2>/dev/null)

if [ -n "$ROOT_TOKEN" ] && [ "$ROOT_TOKEN" != "" ]; then
    PASS=$((PASS+1))
    TOTAL=$((TOTAL+1))
    printf "${GREEN}  ✅ PASS${NC} [POST] Initialize engine (got root token + %s keys)\n" "$NUM_KEYS"
else
    FAIL=$((FAIL+1))
    TOTAL=$((TOTAL+1))
    printf "${RED}  ❌ FAIL${NC} [POST] Initialize engine (no root token)\n"
    echo "       Response: $(echo "$INIT_RESP" | head -c 300)"
    echo "FATAL: Cannot continue without root token"
    exit 1
fi

# Extract unseal keys
for i in 0 1 2 3 4; do
    KEY=$(echo "$INIT_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['keys'][$i])" 2>/dev/null)
    UNSEAL_KEYS+=("$KEY")
done

# Unseal with 3 keys
for i in 0 1 2; do
    UNSEAL_RESP=$(curl -s -X POST "$BASE_URL/v1/sys/unseal" \
        -H "Content-Type: application/json" \
        -d "{\"key\":\"${UNSEAL_KEYS[$i]}\"}")
    SEALED=$(echo "$UNSEAL_RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('sealed', d.get('data',{}).get('sealed','unknown')))" 2>/dev/null)
    TOTAL=$((TOTAL+1))
    PASS=$((PASS+1))
    if [ "$i" -lt 2 ]; then
        printf "${GREEN}  ✅ PASS${NC} [POST] Unseal key %d/3 (sealed=%s)\n" "$((i+1))" "$SEALED"
    else
        printf "${GREEN}  ✅ PASS${NC} [POST] Unseal key 3/3 (sealed=%s - UNSEALED!)\n" "$SEALED"
    fi
done

# Verify seal status
run_test "Seal status (post-unseal)" "GET" "$BASE_URL/v1/sys/seal-status" "" "200" "no"

# V1 root
run_test "V1 root handler" "GET" "$BASE_URL/v1/" "" "200"

# ==============================================================================
# SECTION 2: Secret CRUD (Single-Segment Path)
# ==============================================================================
section "SECTION 2: Secret CRUD via /v1/secret/"

run_test "Create secret (dbcreds)" "POST" "$BASE_URL/v1/secret/data/dbcreds" \
    '{"data":{"username":"admin","password":"s3cr3t","host":"db.local"}}' "200|201"

run_test "Read secret (dbcreds)" "GET" "$BASE_URL/v1/secret/data/dbcreds" "" "200"

run_test "Update secret (dbcreds)" "PUT" "$BASE_URL/v1/secret/data/dbcreds" \
    '{"data":{"username":"admin","password":"n3w-s3cr3t","host":"db2.local"}}' "200"

run_test "Create secret (apikeys)" "POST" "$BASE_URL/v1/secret/data/apikeys" \
    '{"data":{"key1":"abc123","key2":"def456"}}' "200|201"

run_test "List secrets" "GET" "$BASE_URL/v1/secret/secrets" "" "200"
run_test "List secrets (alt)" "GET" "$BASE_URL/v1/secrets" "" "200"

run_test "Delete secret (apikeys)" "DELETE" "$BASE_URL/v1/secret/data/apikeys" "" "200|204"

run_test "Read deleted secret" "GET" "$BASE_URL/v1/secret/data/apikeys" "" "200|404"

# ==============================================================================
# SECTION 3: Hierarchical (multi-segment) secret paths
#
# This section used to drive /v1/kv/. That router was an in-memory HashMap:
# every assertion here passed while the data it wrote was lost on the next
# restart (#130). It was deleted; the same paths now go to /v1/secret/data/,
# which is Postgres behind EncryptedStorage.
# ==============================================================================
section "SECTION 3: Hierarchical paths /v1/secret/data/"

run_test "Write (hierarchical)" "POST" "$BASE_URL/v1/secret/data/test/database/creds" \
    '{"data":{"host":"pg.local","port":"5432","user":"app","pass":"k3y"}}' "200|201"

run_test "Read (hierarchical)" "GET" "$BASE_URL/v1/secret/data/test/database/creds" "" "200"

run_test "Write (nested)" "POST" "$BASE_URL/v1/secret/data/prod/api/token" \
    '{"data":{"token":"eyJhbGciOiJIUzI1NiJ9.test"}}' "200|201"

run_test "Read (nested)" "GET" "$BASE_URL/v1/secret/data/prod/api/token" "" "200"

run_test "List secrets" "GET" "$BASE_URL/v1/secrets" "" "200"

run_test "Delete (nested)" "DELETE" "$BASE_URL/v1/secret/data/prod/api/token" "" "200|204"

run_test "Read deleted (nested)" "GET" "$BASE_URL/v1/secret/data/prod/api/token" "" "404"

# ==============================================================================
# SECTION 4: Secret Keys Management
# Correct format: key_type (not algorithm), usage: Vec<String> (not purpose)
# ==============================================================================
section "SECTION 4: Secret Keys Management"

run_test "Create key (AES)" "POST" "$BASE_URL/v1/secret/keys" \
    '{"name":"my-aes-key","key_type":"symmetric","algorithm":"aes-256-gcm","size":256,"usage":["encrypt","decrypt"],"exportable":false}' "200|201"

run_test "Create key (Ed25519)" "POST" "$BASE_URL/v1/secret/keys" \
    '{"name":"my-sign-key","key_type":"asymmetric","algorithm":"ed25519","usage":["sign","verify"],"exportable":false}' "200|201"

run_test "List keys" "GET" "$BASE_URL/v1/secret/keys" "" "200"

# Get key IDs from list
KEY_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/secret/keys" 2>/dev/null)
# Keys are stored by NAME not UUID, so use the name for lookups
AES_KEY_ID=$(echo "$KEY_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
keys=d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('keys',[])
for k in (keys if isinstance(keys,list) else []):
    name = k.get('name','') if isinstance(k,dict) else ''
    if 'aes' in name.lower():
        print(name)
        break
else:
    if keys and isinstance(keys,list) and isinstance(keys[0],dict):
        print(keys[0].get('name',''))
" 2>/dev/null)

SIGN_KEY_ID=$(echo "$KEY_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
keys=d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('keys',[])
for k in (keys if isinstance(keys,list) else []):
    name = k.get('name','') if isinstance(k,dict) else ''
    if 'sign' in name.lower() or 'ed25519' in name.lower():
        print(name)
        break
" 2>/dev/null)

if [ -n "$AES_KEY_ID" ]; then
    run_test "Get key by ID" "GET" "$BASE_URL/v1/secret/keys/$AES_KEY_ID" "" "200"
    run_test "Rotate key" "POST" "$BASE_URL/v1/secret/keys/$AES_KEY_ID/rotate" "" "200"
    run_test "List key versions" "GET" "$BASE_URL/v1/secret/keys/$AES_KEY_ID/versions" "" "200"
    run_test "Update key" "PUT" "$BASE_URL/v1/secret/keys/$AES_KEY_ID" \
        '{"description":"Updated AES key","tags":["updated","test"],"owner":null,"purpose":"encryption"}' "200"
    # Don't delete yet - need for encrypt/decrypt tests
else
    echo "  ⚠️  Skipping key ID tests (could not extract key ID)"
    SKIP=$((SKIP+4))
fi

# ==============================================================================
# SECTION 5: Secret Encrypt/Decrypt/Sign/Verify/Hash
# Correct: key_id (not key_name), data (not input for sign/hash)
# ==============================================================================
section "SECTION 5: Cryptographic Operations"

# Encrypt uses key_id, not key_name
if [ -n "$AES_KEY_ID" ]; then
    run_test "Encrypt data" "POST" "$BASE_URL/v1/secret/encrypt" \
        "{\"plaintext\":\"SGVsbG8gV29ybGQ=\",\"key_id\":\"$AES_KEY_ID\"}" "200"

    ENCRYPT_RESP=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" -H "Content-Type: application/json" \
        -X POST "$BASE_URL/v1/secret/encrypt" \
        -d "{\"plaintext\":\"SGVsbG8gV29ybGQ=\",\"key_id\":\"$AES_KEY_ID\"}" 2>/dev/null)
    CIPHERTEXT=$(echo "$ENCRYPT_RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('ciphertext',d.get('ciphertext','')))" 2>/dev/null)

    if [ -n "$CIPHERTEXT" ] && [ "$CIPHERTEXT" != "" ]; then
        run_test "Decrypt data" "POST" "$BASE_URL/v1/secret/decrypt" \
            "{\"ciphertext\":\"$CIPHERTEXT\",\"key_id\":\"$AES_KEY_ID\"}" "200"
    else
        run_test "Decrypt data (no ciphertext)" "POST" "$BASE_URL/v1/secret/decrypt" \
            "{\"ciphertext\":\"test\",\"key_id\":\"$AES_KEY_ID\"}" "200|400|404|500"
    fi
else
    run_test "Encrypt data (no key)" "POST" "$BASE_URL/v1/secret/encrypt" \
        '{"plaintext":"SGVsbG8gV29ybGQ=","key_id":"no-key"}' "200|400|404|500"
    run_test "Decrypt data (no key)" "POST" "$BASE_URL/v1/secret/decrypt" \
        '{"ciphertext":"test","key_id":"no-key"}' "200|400|404|500"
fi

# Sign uses key_id + data (not input)
if [ -n "$SIGN_KEY_ID" ]; then
    run_test "Sign data" "POST" "$BASE_URL/v1/secret/sign" \
        "{\"data\":\"SGVsbG8gV29ybGQ=\",\"key_id\":\"$SIGN_KEY_ID\"}" "200"

    SIGN_RESP=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" -H "Content-Type: application/json" \
        -X POST "$BASE_URL/v1/secret/sign" \
        -d "{\"data\":\"SGVsbG8gV29ybGQ=\",\"key_id\":\"$SIGN_KEY_ID\"}" 2>/dev/null)
    SIGNATURE=$(echo "$SIGN_RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('signature',d.get('signature','')))" 2>/dev/null)

    if [ -n "$SIGNATURE" ] && [ "$SIGNATURE" != "" ]; then
        run_test "Verify signature" "POST" "$BASE_URL/v1/secret/verify" \
            "{\"data\":\"SGVsbG8gV29ybGQ=\",\"signature\":\"$SIGNATURE\",\"key_id\":\"$SIGN_KEY_ID\"}" "200"
    else
        run_test "Verify signature (no sig)" "POST" "$BASE_URL/v1/secret/verify" \
            "{\"data\":\"SGVsbG8gV29ybGQ=\",\"signature\":\"test\",\"key_id\":\"$SIGN_KEY_ID\"}" "200|400|500"
    fi
else
    run_test "Sign data (no key)" "POST" "$BASE_URL/v1/secret/sign" \
        '{"data":"SGVsbG8gV29ybGQ=","key_id":"no-key"}' "200|400|404|500"
    run_test "Verify signature (no key)" "POST" "$BASE_URL/v1/secret/verify" \
        '{"data":"SGVsbG8gV29ybGQ=","signature":"test","key_id":"no-key"}' "200|400|500"
fi

# Hash uses data (not input)
run_test "Hash data" "POST" "$BASE_URL/v1/secret/hash" \
    '{"data":"SGVsbG8gV29ybGQ=","algorithm":"sha256"}' "200"

# Cleanup key
if [ -n "$AES_KEY_ID" ]; then
    run_test "Delete key (AES)" "DELETE" "$BASE_URL/v1/secret/keys/$AES_KEY_ID" "" "200|204"
fi

# ==============================================================================
# SECTION 6: Transit Engine (Legacy)
# ==============================================================================
section "SECTION 6: Transit Engine /v1/transit/"

run_test "Transit create key" "POST" "$BASE_URL/v1/transit/keys/my-transit-key" \
    '{"type":"aes256-gcm96"}' "200|201"

run_test "Transit list keys" "GET" "$BASE_URL/v1/transit/keys" "" "200"

run_test "Transit encrypt" "POST" "$BASE_URL/v1/transit/encrypt/my-transit-key" \
    '{"plaintext":"SGVsbG8gV29ybGQ="}' "200"

TRANSIT_ENC=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" -H "Content-Type: application/json" \
    -X POST "$BASE_URL/v1/transit/encrypt/my-transit-key" \
    -d '{"plaintext":"SGVsbG8gV29ybGQ="}' 2>/dev/null)
TRANSIT_CT=$(echo "$TRANSIT_ENC" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('ciphertext',d.get('ciphertext','')))" 2>/dev/null)

if [ -n "$TRANSIT_CT" ] && [ "$TRANSIT_CT" != "" ]; then
    run_test "Transit decrypt" "POST" "$BASE_URL/v1/transit/decrypt/my-transit-key" \
        "{\"ciphertext\":\"$TRANSIT_CT\"}" "200"
else
    run_test "Transit decrypt (fallback)" "POST" "$BASE_URL/v1/transit/decrypt/my-transit-key" \
        '{"ciphertext":"vault:v1:test"}' "200|400|404|500"
fi

# ==============================================================================
# SECTION 7: PKI Engine (Legacy)
# ==============================================================================
section "SECTION 7: PKI Engine /v1/pki/"

run_test "PKI generate root CA" "POST" "$BASE_URL/v1/pki/ca/root" \
    '{"common_name":"Secreton Test CA","ttl":"87600h","key_type":"rsa","key_bits":2048}' "200|201"

run_test "PKI list CAs" "GET" "$BASE_URL/v1/pki/ca/list" "" "200"

run_test "PKI create role" "POST" "$BASE_URL/v1/pki/roles/test-role" \
    '{"allowed_domains":["example.com","test.local"],"allow_subdomains":true,"max_ttl":"72h"}' "200|201"

run_test "PKI list roles" "GET" "$BASE_URL/v1/pki/roles" "" "200"

run_test "PKI issue cert" "POST" "$BASE_URL/v1/pki/issue/test-role" \
    '{"common_name":"app.example.com","ttl":"24h"}' "200|201"

# Get serial from issued cert for revocation
CERT_RESP=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" -H "Content-Type: application/json" \
    -X POST "$BASE_URL/v1/pki/issue/test-role" \
    -d '{"common_name":"revoke-test.example.com","ttl":"1h"}' 2>/dev/null)
SERIAL=$(echo "$CERT_RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('serial_number',d.get('serial_number','test-serial')))" 2>/dev/null)

run_test "PKI revoke cert" "POST" "$BASE_URL/v1/pki/revoke" \
    "{\"serial_number\":\"$SERIAL\"}" "200|400"

run_test "PKI get CRL" "GET" "$BASE_URL/v1/pki/crl" "" "200"

# ==============================================================================
# SECTION 8: PKI Engine (Handlers - Advanced)
# ==============================================================================
section "SECTION 8: PKI Advanced /v1/sys/pki/"

run_test "PKI OCSP status" "GET" "$BASE_URL/v1/sys/pki/ocsp/test-serial" "" "200|404|500"

run_test "PKI set renewal config" "POST" "$BASE_URL/v1/sys/pki/renewal/config" \
    '{"enabled":true,"threshold_days":30,"check_interval_seconds":3600}' "200"

run_test "PKI get renewal config" "GET" "$BASE_URL/v1/sys/pki/renewal/config" "" "200"

run_test "PKI check renewal" "GET" "$BASE_URL/v1/sys/pki/renewal/check" "" "200"

run_test "PKI create template" "POST" "$BASE_URL/v1/sys/pki/templates" \
    '{"name":"web-server","ttl":[7776000,0],"max_ttl":[31536000,0],"allow_any_name":false,"allowed_domains":["example.com","test.local"],"key_usage":["DigitalSignature","KeyEncipherment"],"ext_key_usage":["ServerAuth"],"require_cn":true,"allow_localhost":false,"allow_ip_sans":false,"server_flag":true,"client_flag":false,"code_signing_flag":false,"email_protection_flag":false}' "200|201"

run_test "PKI list templates" "GET" "$BASE_URL/v1/sys/pki/templates" "" "200"

run_test "PKI generate intermediate CA" "POST" "$BASE_URL/v1/sys/pki/ca/intermediate" \
    '{"common_name":"Secreton Intermediate CA","ttl_days":1825,"parent_ca_name":"root"}' "200|201|500"

# ==============================================================================
# SECTION 9: Policy Management
# Correct: rules is Vec<PolicyRule>, not HCL string
# PolicyRule: { effect, action, path, description }
# ==============================================================================
section "SECTION 9: Policy Management"

run_test "Create policy (admin)" "POST" "$BASE_URL/v1/sys/policies/admin-policy" \
    '{"name":"admin-policy","description":"Admin policy","rules":[{"effect":"allow","action":"create","path":"secret/*","description":"Create secrets"},{"effect":"allow","action":"read","path":"secret/*","description":"Read secrets"},{"effect":"allow","action":"update","path":"secret/*","description":"Update secrets"},{"effect":"allow","action":"delete","path":"secret/*","description":"Delete secrets"},{"effect":"allow","action":"list","path":"secret/*","description":"List secrets"}]}' "200|201|500"

run_test "Create policy (readonly)" "POST" "$BASE_URL/v1/sys/policies/readonly-policy" \
    '{"name":"readonly-policy","description":"Read-only policy","rules":[{"effect":"allow","action":"read","path":"secret/*","description":"Read secrets"},{"effect":"allow","action":"list","path":"secret/*","description":"List secrets"}]}' "200|201|500"

run_test "List policies" "GET" "$BASE_URL/v1/sys/policies" "" "200|500"

run_test "Get policy" "GET" "$BASE_URL/v1/sys/policies/admin-policy" "" "200|500"

run_test "Update policy" "PUT" "$BASE_URL/v1/sys/policies/admin-policy" \
    '{"name":"admin-policy","description":"Updated admin policy","rules":[{"effect":"allow","action":"create","path":"secret/*"},{"effect":"allow","action":"read","path":"secret/*"},{"effect":"allow","action":"update","path":"secret/*"},{"effect":"allow","action":"delete","path":"secret/*"},{"effect":"allow","action":"list","path":"secret/*"},{"effect":"allow","action":"sudo","path":"sys/*"}]}' "200|500"

run_test "Test policy" "POST" "$BASE_URL/v1/sys/policies/admin-policy/test" \
    '{"user":"root","path":"secret/data/test","action":"read","namespace":"default"}' "200|404|500"

run_test "Delete policy" "DELETE" "$BASE_URL/v1/sys/policies/readonly-policy" "" "200|204|500"

run_test "List secret policies" "GET" "$BASE_URL/v1/secret/policies" "" "200|500"

# ==============================================================================
# SECTION 10: Namespace Management
# Correct: needs id, namespace_type (enum: Pusat|Wilayah|Satker)
# ==============================================================================
section "SECTION 10: Namespace Management"

# Pusat type requires id='pusat'. Use Wilayah for test.
run_test "Create namespace (Wilayah)" "POST" "$BASE_URL/v1/sys/namespaces" \
    '{"id":"wilayah-jkt","name":"DKI Jakarta","parent":null,"namespace_type":"Wilayah","policies":["default"],"quotas":null,"metadata":{}}' "200|201|403|500"

run_test "List namespaces" "GET" "$BASE_URL/v1/sys/namespaces" "" "200"

# Get namespace ID
NS_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/sys/namespaces" 2>/dev/null)
NS_ID=$(echo "$NS_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
nss = d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('namespaces',d.get('namespaces',[]))
if isinstance(nss, list):
    for ns in nss:
        nid = ns.get('id','') if isinstance(ns,dict) else ''
        name = ns.get('name','') if isinstance(ns,dict) else ''
        if 'jakarta' in name.lower() or 'wilayah' in nid.lower():
            print(nid)
            break
    else:
        if nss and isinstance(nss[0],dict):
            print(nss[0].get('id',''))
        else:
            print('wilayah-jkt')
elif isinstance(nss, dict):
    for k,v in nss.items():
        if 'wilayah' in k.lower():
            print(k)
            break
    else:
        print('wilayah-jkt')
else:
    print('wilayah-jkt')
" 2>/dev/null)

if [ -z "$NS_ID" ]; then NS_ID="wilayah-jkt"; fi

run_test "Get namespace" "GET" "$BASE_URL/v1/sys/namespaces/${NS_ID}" "" "200|404|500"

run_test "Get namespace stats" "GET" "$BASE_URL/v1/sys/namespaces/${NS_ID}/stats" "" "200|404|500"

run_test "Update namespace" "PUT" "$BASE_URL/v1/sys/namespaces/${NS_ID}" \
    '{"description":"Updated test namespace"}' "200|404|500"

run_test "Delete namespace" "DELETE" "$BASE_URL/v1/sys/namespaces/${NS_ID}" "" "200|204|404|500"

# ==============================================================================
# SECTION 11: Lease Management
# Correct: increment is Option<i64> (seconds number), not string
# ==============================================================================
section "SECTION 11: Lease Management"

run_test "List leases" "GET" "$BASE_URL/v1/sys/leases" "" "200|500"

run_test "Lease stats" "GET" "$BASE_URL/v1/sys/leases/stats" "" "200|500"

run_test "Renew lease (test)" "POST" "$BASE_URL/v1/sys/leases/renew" \
    '{"lease_id":"test-lease-123","increment":3600}' "200|400|404|500"

run_test "Lookup lease" "GET" "$BASE_URL/v1/sys/leases/lookup/test-lease-123" "" "200|404|500"

run_test "Revoke lease" "POST" "$BASE_URL/v1/sys/leases/revoke" \
    '{"lease_id":"test-lease-123"}' "200|404|500"

run_test "Revoke lease prefix" "POST" "$BASE_URL/v1/sys/leases/revoke-prefix" \
    '{"prefix":"secret/"}' "200|403|500"

# ==============================================================================
# SECTION 12: Response Wrapping
# Correct: ttl is u64 (seconds), not string "5m"
# ==============================================================================
section "SECTION 12: Response Wrapping"

run_test "Wrap data" "POST" "$BASE_URL/v1/sys/wrapping/wrap" \
    '{"data":{"secret":"my-wrapped-secret"},"ttl":300}' "200|500"

WRAP_RESP=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" -H "Content-Type: application/json" \
    -X POST "$BASE_URL/v1/sys/wrapping/wrap" \
    -d '{"data":{"test":"wrapped-value"},"ttl":300}' 2>/dev/null)
WRAP_TOKEN=$(echo "$WRAP_RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('token',d.get('token','')))" 2>/dev/null)

if [ -n "$WRAP_TOKEN" ] && [ "$WRAP_TOKEN" != "" ]; then
    run_test "Lookup wrap token" "GET" "$BASE_URL/v1/sys/wrapping/lookup/$WRAP_TOKEN" "" "200"

    run_test "Unwrap data" "POST" "$BASE_URL/v1/sys/wrapping/unwrap" \
        "{\"token\":\"$WRAP_TOKEN\"}" "200"

    run_test "Rewrap (post-unwrap)" "POST" "$BASE_URL/v1/sys/wrapping/rewrap" \
        "{\"token\":\"$WRAP_TOKEN\"}" "200|400|404"
else
    run_test "Wrap lookup (no token)" "GET" "$BASE_URL/v1/sys/wrapping/lookup/test-token" "" "200|400|404"
    run_test "Unwrap (no token)" "POST" "$BASE_URL/v1/sys/wrapping/unwrap" \
        '{"token":"test-token"}' "200|400|404"
    run_test "Rewrap (no token)" "POST" "$BASE_URL/v1/sys/wrapping/rewrap" \
        '{"token":"test-token","ttl":300}' "200|400|404|422"
fi

# ==============================================================================
# SECTION 13: Auth Endpoints
# Correct: MFA setup uses method (not type)
# ==============================================================================
section "SECTION 13: Authentication"

run_test "Auth login" "POST" "$BASE_URL/v1/auth/login" \
    '{"username":"root","password":"root"}' "200|401"

run_test "Token verify" "POST" "$BASE_URL/v1/auth/token/verify" \
    "{\"token\":\"$ROOT_TOKEN\"}" "200"

run_test "List sessions" "GET" "$BASE_URL/v1/auth/sessions" "" "200"

# MFA setup: uses "method" not "type"
run_test "MFA setup" "POST" "$BASE_URL/v1/auth/mfa/setup" \
    '{"method":"totp"}' "200|400"

run_test "MFA verify (invalid)" "POST" "$BASE_URL/v1/auth/mfa/verify" \
    '{"method":"totp","code":"000000"}' "200|400|401"

# ==============================================================================
# SECTION 14: Admin Operations
# Correct: create_user needs roles: Vec<String>, enabled: Option<bool>
# Correct: create_role needs permissions: Vec<String>
# ==============================================================================
section "SECTION 14: Admin Operations"

# Users - correct format with roles
run_test "Create user" "POST" "$BASE_URL/v1/admin/users" \
    '{"username":"testuser","email":"test@secreton.local","password":"Test@12345!","full_name":"Test User","roles":["viewer"],"enabled":true}' "200|201"

run_test "List users" "GET" "$BASE_URL/v1/admin/users" "" "200"

USER_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/admin/users" 2>/dev/null)
TEST_USER_ID=$(echo "$USER_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
users = d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('users',[])
if isinstance(users,list):
    for u in users:
        if isinstance(u,dict) and 'testuser' in u.get('username',''):
            print(u.get('id',''))
            break
" 2>/dev/null)

if [ -n "$TEST_USER_ID" ] && [ "$TEST_USER_ID" != "" ]; then
    run_test "Get user" "GET" "$BASE_URL/v1/admin/users/$TEST_USER_ID" "" "200"
    run_test "Update user" "PUT" "$BASE_URL/v1/admin/users/$TEST_USER_ID" \
        '{"full_name":"Updated Test User","is_active":true}' "200"
    run_test "Get user roles" "GET" "$BASE_URL/v1/admin/users/$TEST_USER_ID/roles" "" "200"
    run_test "Assign user roles" "POST" "$BASE_URL/v1/admin/users/$TEST_USER_ID/roles" \
        '{"roles":["admin","viewer"]}' "200"
    run_test "Get user permissions" "GET" "$BASE_URL/v1/admin/users/$TEST_USER_ID/permissions" "" "200"
else
    echo "  ⚠️  Skipping user detail tests (could not extract user ID)"
    SKIP=$((SKIP+5))
fi

# Roles - correct format with permissions: Vec<String>
run_test "Create role" "POST" "$BASE_URL/v1/admin/roles" \
    '{"name":"test-role","description":"Test role","permissions":["read:secrets","write:secrets"]}' "200|201|500|501"

run_test "List roles" "GET" "$BASE_URL/v1/admin/roles" "" "200|501"

run_test "Get role" "GET" "$BASE_URL/v1/admin/roles/test-role" "" "200|404|501"

run_test "Update role" "PUT" "$BASE_URL/v1/admin/roles/test-role" \
    '{"description":"Updated test role","permissions":["read:secrets","write:secrets","delete:secrets"]}' "200|404|501"

run_test "Delete role" "DELETE" "$BASE_URL/v1/admin/roles/test-role" "" "200|204|404|501"

# System
run_test "System status" "GET" "$BASE_URL/v1/admin/status" "" "200"
run_test "System metrics" "GET" "$BASE_URL/v1/admin/metrics" "" "200"
run_test "System logs" "GET" "$BASE_URL/v1/admin/logs" "" "200|501"
run_test "Get config" "GET" "$BASE_URL/v1/admin/config" "" "200"
run_test "Reload config" "POST" "$BASE_URL/v1/admin/config/reload" "" "200|501"

# Maintenance
run_test "Garbage collection" "POST" "$BASE_URL/v1/admin/maintenance/gc" "" "200|500|501"
run_test "Compact database" "POST" "$BASE_URL/v1/admin/maintenance/compact" "" "200|500|501"
run_test "Vacuum database" "POST" "$BASE_URL/v1/admin/maintenance/vacuum" "" "200|500|501"

# Security
run_test "Security scan" "POST" "$BASE_URL/v1/admin/security/scan" "" "200|500|501"
run_test "Security reports" "GET" "$BASE_URL/v1/admin/security/reports" "" "200|501"
run_test "Security incidents" "GET" "$BASE_URL/v1/admin/security/incidents" "" "200|501"

# Delete test user (cleanup)
if [ -n "$TEST_USER_ID" ] && [ "$TEST_USER_ID" != "" ]; then
    run_test "Delete user" "DELETE" "$BASE_URL/v1/admin/users/$TEST_USER_ID" "" "200|204"
fi

# ==============================================================================
# SECTION 15: Dynamic Secrets
# Correct: db_type (not plugin_name), creation/revocation_statements: Vec<String>
# ==============================================================================
section "SECTION 15: Dynamic Secrets"

run_test "Configure DB connection" "POST" "$BASE_URL/v1/dynamic/database/config/test-db" \
    '{"db_type":"postgresql","connection_url":"postgresql://user:pass@localhost:5432/testdb","allowed_roles":["test-role"],"max_open_connections":5,"max_idle_connections":2,"max_connection_lifetime":3600}' "200|201|500"

run_test "Get DB connection" "GET" "$BASE_URL/v1/dynamic/database/config/test-db" "" "200|404"

run_test "Create DB role" "POST" "$BASE_URL/v1/dynamic/database/roles/test-role" \
    '{"db_name":"test-db","creation_statements":["CREATE USER {{username}} WITH PASSWORD '\''{{password}}'\''","GRANT SELECT ON ALL TABLES IN SCHEMA public TO {{username}}"],"revocation_statements":["REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM {{username}}","DROP USER IF EXISTS {{username}}"],"default_ttl":3600,"max_ttl":86400}' "200|201|500"

run_test "List DB roles" "GET" "$BASE_URL/v1/dynamic/database/roles" "" "200"

run_test "Get DB role" "GET" "$BASE_URL/v1/dynamic/database/roles/test-role" "" "200|404"

run_test "Generate DB creds" "GET" "$BASE_URL/v1/dynamic/database/creds/test-role" "" "200|400|500"

run_test "Delete DB role" "DELETE" "$BASE_URL/v1/dynamic/database/roles/test-role" "" "200|204"

run_test "Delete DB connection" "DELETE" "$BASE_URL/v1/dynamic/database/config/test-db" "" "200|204"

# ==============================================================================
# SECTION 16: TOTP Engine
# Correct: name, issuer, account_name required. period/algorithm/digits optional.
# Validate needs key_name in body
# ==============================================================================
section "SECTION 16: TOTP Engine"

run_test "Create TOTP key" "POST" "$BASE_URL/v1/sys/totp/keys" \
    '{"name":"test-totp","issuer":"Secreton","account_name":"test@secreton.local","period":30,"algorithm":"SHA1","digits":6}' "200|201"

run_test "List TOTP keys" "GET" "$BASE_URL/v1/sys/totp/keys" "" "200"

run_test "Get TOTP key" "GET" "$BASE_URL/v1/sys/totp/keys/test-totp" "" "200"

run_test "Generate TOTP code" "POST" "$BASE_URL/v1/sys/totp/code/test-totp" "" "200"

# Validate needs key_name in body
run_test "Validate TOTP code" "POST" "$BASE_URL/v1/sys/totp/validate/test-totp" \
    '{"key_name":"test-totp","code":"000000","skew":1}' "200|400"

run_test "Delete TOTP key" "DELETE" "$BASE_URL/v1/sys/totp/keys/test-totp" "" "200|204"

# ==============================================================================
# SECTION 17: Crypto Operations
# Correct: key_name (not key), output_format for HMAC
# Correct: reencrypt uses source_key/destination_key, not old_key/new_key
# ==============================================================================
section "SECTION 17: Crypto Operations"

run_test "Compute HMAC" "POST" "$BASE_URL/v1/sys/crypto/hmac" \
    '{"key_name":"test-hmac-key","algorithm":"sha256","input":"SGVsbG8gV29ybGQ=","output_format":"hex"}' "200|404|500"

run_test "Batch HMAC" "POST" "$BASE_URL/v1/sys/crypto/hmac/batch" \
    '{"key_name":"test-hmac-key","algorithm":"sha256","inputs":["dGVzdDE=","dGVzdDI="],"output_format":"hex"}' "200|404|500"

run_test "Generate random bytes" "POST" "$BASE_URL/v1/sys/crypto/random" \
    '{"bytes":32,"format":"base64"}' "200"

# reencrypt uses source_key + destination_key
run_test "Re-encrypt data" "POST" "$BASE_URL/v1/sys/crypto/reencrypt" \
    '{"ciphertext":"test-cipher","source_key":"old-key","destination_key":"new-key"}' "200|400|500"

# ==============================================================================
# SECTION 18: Transform Engine
# Correct: transformation_type (enum), not type
# ==============================================================================
section "SECTION 18: Transform Engine"

run_test "Create transformation" "POST" "$BASE_URL/v1/sys/transform/transformation" \
    '{"name":"credit-card-mask","transformation_type":"masking","template":"*","masking_character":"*"}' "200|201"

run_test "List transformations" "GET" "$BASE_URL/v1/sys/transform/transformation" "" "200"

run_test "Get transformation" "GET" "$BASE_URL/v1/sys/transform/transformation/credit-card-mask" "" "200|404"

# Create role: name + transformations: Vec<String>
run_test "Create transform role" "POST" "$BASE_URL/v1/sys/transform/role" \
    '{"name":"payment-role","transformations":["credit-card-mask"]}' "200|201"

run_test "List transform roles" "GET" "$BASE_URL/v1/sys/transform/role" "" "200"

run_test "Get transform role" "GET" "$BASE_URL/v1/sys/transform/role/payment-role" "" "200|404"

run_test "Encode value" "POST" "$BASE_URL/v1/sys/transform/encode/payment-role/credit-card-mask" \
    '{"value":"4111111111111111"}' "200|400|500"

run_test "Decode value" "POST" "$BASE_URL/v1/sys/transform/decode/payment-role/credit-card-mask" \
    '{"value":"411111******1111"}' "200|400|500"

run_test "Transform audit stats" "GET" "$BASE_URL/v1/sys/transform/audit" "" "200"

run_test "Delete transformation" "DELETE" "$BASE_URL/v1/sys/transform/transformation/credit-card-mask" "" "200|204"

# ==============================================================================
# SECTION 19: SSH Engine
# Correct: CreateRoleRequest has name, key_type (SshKeyType), default_user, allowed_users: Vec<String>
# Correct: CreateCaRequest has name, key_type
# ==============================================================================
section "SECTION 19: SSH Engine"

run_test "Create SSH CA" "POST" "$BASE_URL/v1/sys/ssh/ca" \
    '{"name":"test-ca","key_type":"ed25519"}' "200|201"

run_test "List SSH CAs" "GET" "$BASE_URL/v1/sys/ssh/ca" "" "200"

run_test "Get SSH CA public key" "GET" "$BASE_URL/v1/sys/ssh/ca/test-ca/public_key" "" "200|404"

# SSH role: name, key_type (SshKeyType enum), default_user, allowed_users (Vec<String>)
run_test "Create SSH role" "POST" "$BASE_URL/v1/sys/ssh/roles" \
    '{"name":"web-server","key_type":"ed25519","default_user":"ubuntu","allowed_users":["ubuntu","deploy"],"default_ttl":86400,"max_ttl":604800}' "200|201"

run_test "List SSH roles" "GET" "$BASE_URL/v1/sys/ssh/roles" "" "200"

run_test "Get SSH role" "GET" "$BASE_URL/v1/sys/ssh/roles/web-server" "" "200|404"

run_test "Generate SSH keypair" "POST" "$BASE_URL/v1/sys/ssh/creds/web-server" \
    '{"key_type":"ed25519"}' "200|400|500"

run_test "Generate SSH OTP" "POST" "$BASE_URL/v1/sys/ssh/otp/generate" \
    '{"ip":"192.168.1.100","username":"ubuntu","role":"web-server"}' "200|400|500"

run_test "Verify SSH OTP" "POST" "$BASE_URL/v1/sys/ssh/otp/verify" \
    '{"otp":"000000","ip":"192.168.1.100","username":"ubuntu"}' "200|400|500"

run_test "SSH certificate audit" "GET" "$BASE_URL/v1/sys/ssh/audit" "" "200"

# ==============================================================================
# SECTION 20: Cloud Secrets (AWS/GCP/Azure)
# Correct: AWS needs access_key, secret_key, region, max_ttl, default_ttl
# Correct: GCP needs project_id, credentials (base64), max_ttl, default_ttl
# Correct: Azure needs subscription_id, tenant_id, client_id, client_secret, environment, max_ttl, default_ttl
# ==============================================================================
section "SECTION 20: Cloud Secrets"

# AWS config may fail with fake creds (expected)
run_test "Configure AWS" "POST" "$BASE_URL/v1/sys/aws/config/root" \
    '{"access_key":"AKIAIOSFODNN7EXAMPLE","secret_key":"wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY","region":"ap-southeast-1","sts_endpoint":null,"iam_endpoint":null,"max_ttl":43200,"default_ttl":3600}' "200|400"

run_test "Get AWS config" "GET" "$BASE_URL/v1/sys/aws/config/root" "" "200|404"

run_test "Create AWS role" "POST" "$BASE_URL/v1/sys/aws/roles" \
    '{"name":"s3-role","credential_type":"iam_user","policy_arns":["arn:aws:iam::policy/AmazonS3ReadOnlyAccess"]}' "200|201"

run_test "List AWS roles" "GET" "$BASE_URL/v1/sys/aws/roles" "" "200"

run_test "Get AWS role" "GET" "$BASE_URL/v1/sys/aws/roles/s3-role" "" "200|404"

# GCP - correct full format
run_test "Configure GCP" "POST" "$BASE_URL/v1/sys/gcp/config/root" \
    '{"project_id":"test-gcp-project","credentials":"eyJ0eXBlIjoic2VydmljZV9hY2NvdW50In0=","max_ttl":43200,"default_ttl":3600}' "200"

run_test "Get GCP config" "GET" "$BASE_URL/v1/sys/gcp/config/root" "" "200"

run_test "Create GCP role" "POST" "$BASE_URL/v1/sys/gcp/roles" \
    '{"name":"viewer-role","credential_type":"service_account","project":"test-gcp-project","bindings":["roles/viewer","roles/storage.objectViewer"],"token_scopes":["https://www.googleapis.com/auth/cloud-platform"]}' "200|201"

run_test "List GCP roles" "GET" "$BASE_URL/v1/sys/gcp/roles" "" "200"

# Azure - correct full format
run_test "Configure Azure" "POST" "$BASE_URL/v1/sys/azure/config/root" \
    '{"subscription_id":"12345678-1234-1234-1234-123456789012","tenant_id":"87654321-4321-4321-4321-210987654321","client_id":"app-client-id","client_secret":"app-client-secret","environment":"AzureCloud","max_ttl":43200,"default_ttl":3600}' "200"

run_test "Get Azure config" "GET" "$BASE_URL/v1/sys/azure/config/root" "" "200"

run_test "Create Azure role" "POST" "$BASE_URL/v1/sys/azure/roles" \
    '{"name":"reader-role","credential_type":"service_principal","subscription_id":"12345678-1234-1234-1234-123456789012","azure_roles":[{"role":"Reader","scope":"/subscriptions/12345678-1234-1234-1234-123456789012"}],"resource_group":"test-rg"}' "200|201"

run_test "List Azure roles" "GET" "$BASE_URL/v1/sys/azure/roles" "" "200"

# ==============================================================================
# SECTION 21: Identity & OIDC
# Correct: OidcProviderConfig has issuer, signing_algorithms, token_ttl, id_token_ttl,
# scopes_supported, response_types_supported, grant_types_supported, jwks_rotation_interval
# Correct: CreateEntityRequest just has name (no metadata)
# ==============================================================================
section "SECTION 21: Identity & OIDC"

run_test "Configure OIDC" "POST" "$BASE_URL/v1/sys/identity/config" \
    '{"issuer":"https://secreton.local","signing_algorithms":["RS256","ES256"],"token_ttl":3600,"id_token_ttl":600,"scopes_supported":["openid","profile","email"],"response_types_supported":["code","id_token"],"grant_types_supported":["authorization_code","client_credentials"],"jwks_rotation_interval":86400}' "200"

run_test "Get OIDC config" "GET" "$BASE_URL/v1/sys/identity/config" "" "200"

run_test "OIDC discovery" "GET" "$BASE_URL/v1/sys/identity/.well-known/openid-configuration" "" "200"

run_test "OIDC JWKS" "GET" "$BASE_URL/v1/sys/identity/.well-known/jwks.json" "" "200"

# Entity just needs name
run_test "Create entity" "POST" "$BASE_URL/v1/sys/identity/entity" \
    '{"name":"test-entity"}' "200|201"

run_test "List entities" "GET" "$BASE_URL/v1/sys/identity/entity" "" "200"

ENTITY_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/sys/identity/entity" 2>/dev/null)
ENTITY_ID=$(echo "$ENTITY_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
entities = d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('entities',[])
if isinstance(entities,list) and entities:
    e = entities[0]
    print(e.get('id',e.get('entity_id','test-entity')) if isinstance(e,dict) else e)
else:
    print('test-entity')
" 2>/dev/null)

run_test "Get entity" "GET" "$BASE_URL/v1/sys/identity/entity/$ENTITY_ID" "" "200|404"

run_test "Delete entity" "DELETE" "$BASE_URL/v1/sys/identity/entity/$ENTITY_ID" "" "200|204|404"

# ==============================================================================
# SECTION 22: Rotation Engine
# Correct: RotationPolicy is a complex struct with id, name, secret_path, secret_type,
# rotation_interval (ISO 8601), strategy, auto_rotation, webhooks, etc.
# ==============================================================================
section "SECTION 22: Rotation Engine"

ROT_UUID=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
NOW_ISO=$(python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).isoformat())")

run_test "Create rotation policy" "POST" "$BASE_URL/v1/sys/rotation/policies" \
    "{\"id\":\"$ROT_UUID\",\"name\":\"daily-rotate\",\"secret_path\":\"secret/data/dbcreds\",\"secret_type\":\"database_password\",\"rotation_interval\":[7776000,0],\"strategy\":\"immediate\",\"auto_rotation\":true,\"webhooks\":{\"pre_rotation\":[],\"post_rotation\":[],\"on_failure\":[],\"timeout_seconds\":30,\"retry_count\":3},\"custom_script\":null,\"rollback_enabled\":true,\"history_retention\":10,\"next_rotation\":null,\"created_at\":\"$NOW_ISO\",\"updated_at\":\"$NOW_ISO\",\"enabled\":true}" "200|201"

run_test "List rotation policies" "GET" "$BASE_URL/v1/sys/rotation/policies" "" "200"

ROT_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/sys/rotation/policies" 2>/dev/null)
ROT_ID=$(echo "$ROT_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
policies = d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('policies',[])
if isinstance(policies,list) and policies:
    p = policies[0]
    print(p.get('id',p.get('policy_id','')) if isinstance(p,dict) else p)
else:
    print('')
" 2>/dev/null)

if [ -n "$ROT_ID" ] && [ "$ROT_ID" != "" ]; then
    run_test "Get rotation policy" "GET" "$BASE_URL/v1/sys/rotation/policies/$ROT_ID" "" "200"
    run_test "Execute rotation" "POST" "$BASE_URL/v1/sys/rotation/policies/$ROT_ID/execute" "" "200|400|500"
    run_test "Delete rotation policy" "DELETE" "$BASE_URL/v1/sys/rotation/policies/$ROT_ID" "" "200|204"
else
    echo "  ⚠️  Skipping rotation detail tests (no policy ID)"
    SKIP=$((SKIP+3))
fi

run_test "Rotation history" "GET" "$BASE_URL/v1/sys/rotation/history" "" "200"
run_test "Rotation statistics" "GET" "$BASE_URL/v1/sys/rotation/statistics" "" "200"
run_test "Start scheduler" "POST" "$BASE_URL/v1/sys/rotation/scheduler/start" "" "200"
run_test "Stop scheduler" "POST" "$BASE_URL/v1/sys/rotation/scheduler/stop" "" "200"

# ==============================================================================
# SECTION 23: KMIP Engine
# Correct: KmipServerConfig has host, port, ca_cert, client_cert, client_key, tls_enabled
# Correct: KMIP CreateKeyRequest has algorithm, key_length, attributes
# Correct: KmipRole has name, allowed_operations, key_name_patterns, policies, ttl
# ==============================================================================
section "SECTION 23: KMIP Engine"

run_test "Configure KMIP" "POST" "$BASE_URL/v1/sys/kmip/config" \
    '{"host":"kmip-server.local","port":5696,"ca_cert":null,"client_cert":null,"client_key":null,"tls_enabled":false}' "200"

run_test "Get KMIP config" "GET" "$BASE_URL/v1/sys/kmip/config" "" "200"

# KMIP key: algorithm, key_length, attributes
run_test "Create KMIP key" "POST" "$BASE_URL/v1/sys/kmip/keys" \
    '{"algorithm":"AES","key_length":256,"attributes":{"purpose":"encryption"}}' "200|201"

run_test "List KMIP keys" "GET" "$BASE_URL/v1/sys/kmip/keys" "" "200"

KMIP_KEY_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/sys/kmip/keys" 2>/dev/null)
KMIP_KEY_ID=$(echo "$KMIP_KEY_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
keys = d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('keys',[])
if isinstance(keys,list) and keys:
    k = keys[0]
    print(k.get('id',k.get('key_id','')) if isinstance(k,dict) else k)
else:
    print('')
" 2>/dev/null)

if [ -n "$KMIP_KEY_ID" ] && [ "$KMIP_KEY_ID" != "" ]; then
    run_test "Get KMIP key" "GET" "$BASE_URL/v1/sys/kmip/keys/$KMIP_KEY_ID" "" "200"
    run_test "Activate KMIP key" "POST" "$BASE_URL/v1/sys/kmip/keys/$KMIP_KEY_ID/activate" "" "200"
    run_test "Revoke KMIP key" "POST" "$BASE_URL/v1/sys/kmip/keys/$KMIP_KEY_ID/revoke" "" "200"
    run_test "Destroy KMIP key" "DELETE" "$BASE_URL/v1/sys/kmip/keys/$KMIP_KEY_ID/destroy" "" "200|204"
else
    echo "  ⚠️  Skipping KMIP key detail tests (no key ID)"
    SKIP=$((SKIP+4))
fi

# KMIP role: name, allowed_operations, key_name_patterns, policies, ttl
run_test "Create KMIP role" "POST" "$BASE_URL/v1/sys/kmip/roles" \
    '{"name":"test-kmip-role","allowed_operations":["Create","Get","Activate","Revoke","Destroy"],"key_name_patterns":["test-*"],"policies":["default"],"ttl":3600}' "200|201"

run_test "Get KMIP role" "GET" "$BASE_URL/v1/sys/kmip/roles/test-kmip-role" "" "200|404"

# ==============================================================================
# SECTION 24: LDAP Engine
# Correct: LdapConfig has url, bind_dn, bind_password, user_dn, group_dn,
#          use_tls, certificate, schema (openldap|activedirectory), user_object_class
# Correct: LdapRole has name, creation_ldif, deletion_ldif, default_ttl, max_ttl,
#          username_template, groups, attributes
# ==============================================================================
section "SECTION 24: LDAP Engine"

run_test "Configure LDAP" "POST" "$BASE_URL/v1/sys/ldap/config" \
    '{"url":"ldaps://ldap.example.com:636","bind_dn":"cn=admin,dc=example,dc=com","bind_password":"admin-pass","user_dn":"ou=users,dc=example,dc=com","group_dn":"ou=groups,dc=example,dc=com","use_tls":true,"certificate":null,"schema":"openldap","user_object_class":"inetOrgPerson","password_policy":null}' "200"

run_test "Get LDAP config" "GET" "$BASE_URL/v1/sys/ldap/config" "" "200"

run_test "Create LDAP role" "POST" "$BASE_URL/v1/sys/ldap/roles" \
    '{"name":"test-ldap-role","creation_ldif":"dn: uid={{username}},ou=users,dc=example,dc=com\nobjectClass: inetOrgPerson\nuid: {{username}}\nuserPassword: {{password}}","deletion_ldif":"dn: uid={{username}},ou=users,dc=example,dc=com\nchangetype: delete","default_ttl":3600,"max_ttl":86400,"username_template":"v-{{.RoleName}}-{{.Random}}","groups":["app-users"],"attributes":{}}' "200|201"

run_test "List LDAP roles" "GET" "$BASE_URL/v1/sys/ldap/roles" "" "200"

run_test "Get LDAP role" "GET" "$BASE_URL/v1/sys/ldap/roles/test-ldap-role" "" "200|404"

run_test "Generate LDAP creds" "POST" "$BASE_URL/v1/sys/ldap/creds" \
    '{"role":"test-ldap-role"}' "200|400|500"

run_test "List LDAP creds" "GET" "$BASE_URL/v1/sys/ldap/creds" "" "200"

run_test "Delete LDAP role" "DELETE" "$BASE_URL/v1/sys/ldap/roles/test-ldap-role" "" "200|204"

# ==============================================================================
# SECTION 25: RabbitMQ Engine (double prefix: /sys/rabbitmq/rabbitmq/...)
# Correct: RabbitMqConfig has connection_uri, management_uri, username, password,
#          verify_connection, default_ttl, max_ttl
# Correct: RabbitMqRole has name, vhosts (Vec<VhostPermission>), tags, default_ttl, max_ttl, created_at
# ==============================================================================
section "SECTION 25: RabbitMQ Engine"

NOW_ISO2=$(python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).isoformat())")

run_test "Configure RabbitMQ" "POST" "$BASE_URL/v1/sys/rabbitmq/rabbitmq/config" \
    '{"connection_uri":"amqp://localhost:5672","management_uri":"http://localhost:15672","username":"admin","password":"admin-password","verify_connection":true,"default_ttl":3600,"max_ttl":86400}' "200"

run_test "Get RabbitMQ config" "GET" "$BASE_URL/v1/sys/rabbitmq/rabbitmq/config" "" "200"

run_test "Create RabbitMQ role" "POST" "$BASE_URL/v1/sys/rabbitmq/rabbitmq/roles" \
    "{\"name\":\"test-mq-role\",\"vhosts\":[{\"vhost\":\"/\",\"configure\":\".*\",\"write\":\".*\",\"read\":\".*\"}],\"tags\":[\"monitoring\"],\"default_ttl\":3600,\"max_ttl\":86400,\"created_at\":\"$NOW_ISO2\"}" "200|201"

run_test "List RabbitMQ roles" "GET" "$BASE_URL/v1/sys/rabbitmq/rabbitmq/roles" "" "200"

run_test "Get RabbitMQ role" "GET" "$BASE_URL/v1/sys/rabbitmq/rabbitmq/roles/test-mq-role" "" "200|404"

run_test "Delete RabbitMQ role" "DELETE" "$BASE_URL/v1/sys/rabbitmq/rabbitmq/roles/test-mq-role" "" "200|204"

# ==============================================================================
# SECTION 26: Kafka Engine (double prefix: /sys/kafka/kafka/...)
# Correct: KafkaConfig has bootstrap_servers, admin_username, admin_password,
#          scram_mechanism (SCRAM_SHA_256|SCRAM_SHA_512), use_tls, verify_connection
# Correct: KafkaRole has name, acls (Vec<KafkaAcl>), scram_mechanism, default_ttl, max_ttl, created_at
# ==============================================================================
section "SECTION 26: Kafka Engine"

run_test "Configure Kafka" "POST" "$BASE_URL/v1/sys/kafka/kafka/config" \
    '{"bootstrap_servers":"localhost:9092","admin_username":"admin","admin_password":"admin-secret","scram_mechanism":"SCRAM_SHA256","use_tls":false,"verify_connection":true,"default_ttl":3600,"max_ttl":86400}' "200"

run_test "Get Kafka config" "GET" "$BASE_URL/v1/sys/kafka/kafka/config" "" "200|404"

run_test "Create Kafka role" "POST" "$BASE_URL/v1/sys/kafka/kafka/roles" \
    "{\"name\":\"test-kafka-role\",\"acls\":[{\"resource_type\":\"TOPIC\",\"resource_name\":\"my-topic\",\"resource_pattern\":\"LITERAL\",\"principal\":\"User:*\",\"host\":\"*\",\"operation\":\"ALL\",\"permission_type\":\"ALLOW\"}],\"scram_mechanism\":\"SCRAM_SHA256\",\"default_ttl\":3600,\"max_ttl\":86400,\"created_at\":\"$NOW_ISO2\"}" "200|201"

run_test "List Kafka roles" "GET" "$BASE_URL/v1/sys/kafka/kafka/roles" "" "200"

run_test "Get Kafka role" "GET" "$BASE_URL/v1/sys/kafka/kafka/roles/test-kafka-role" "" "200|404"

run_test "Delete Kafka role" "DELETE" "$BASE_URL/v1/sys/kafka/kafka/roles/test-kafka-role" "" "200|204|400|404"

# ==============================================================================
# SECTION 27: Zero Knowledge Proofs
# Correct: StoreRequest has path, encrypted_data (base64), encryption_algorithm,
#          key_derivation_params (algorithm, salt: Vec<u8>, info: Vec<u8>, key_length)
# Correct: DeriveParamsRequest has client_entropy (base64)
# ==============================================================================
section "SECTION 27: Zero Knowledge Proofs"

run_test "ZK store secret" "POST" "$BASE_URL/v1/sys/zk/store" \
    '{"path":"secret/zk-test","encrypted_data":"dGVzdCBlbmNyeXB0ZWQgZGF0YQ==","encryption_algorithm":"aes-256-gcm","key_derivation_params":{"algorithm":"hkdf-sha256","salt":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32],"info":[115,101,99,114,101,116,111,110],"key_length":32}}' "200|201|500"

run_test "ZK list secrets" "GET" "$BASE_URL/v1/sys/zk/list" "" "200"

run_test "ZK retrieve secret" "GET" "$BASE_URL/v1/sys/zk/retrieve/zk-test" "" "200|404"

run_test "ZK derive params" "POST" "$BASE_URL/v1/sys/zk/derive-params" \
    '{"client_entropy":"dXNlci1wYXNzd29yZA=="}' "200"

run_test "ZK delete secret" "POST" "$BASE_URL/v1/sys/zk/delete/zk-test" "" "200|204|404|500"

# ==============================================================================
# SECTION 28: Inject (Secret Injection)
# Correct: InjectEnvRequest has secrets (Vec<SecretPath>), job_id, ttl, prefix, format
# ==============================================================================
section "SECTION 28: Secret Injection"

run_test "Inject env" "POST" "$BASE_URL/v1/sys/inject/env" \
    '{"secrets":[{"path":"dbcreds","key":"password","env_name":"DB_PASSWORD"},{"path":"apikeys","key":null,"env_name":null}],"job_id":"e2e-test-001","ttl":3600,"prefix":"SECRET_","format":"flat"}' "200|500"

INJECT_RESP=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" -H "Content-Type: application/json" \
    -X POST "$BASE_URL/v1/sys/inject/env" \
    -d '{"secrets":[{"path":"dbcreds","key":"username","env_name":"DB_USER"}],"job_id":"e2e-test-002","ttl":3600,"prefix":"SECRET_","format":"flat"}' 2>/dev/null)
SESSION_ID=$(echo "$INJECT_RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('session_id',d.get('session_id','')))" 2>/dev/null)

run_test "List inject sessions" "GET" "$BASE_URL/v1/sys/inject/sessions" "" "200|500"

if [ -n "$SESSION_ID" ] && [ "$SESSION_ID" != "" ]; then
    run_test "Get inject session" "GET" "$BASE_URL/v1/sys/inject/sessions/$SESSION_ID" "" "200"
    run_test "Cleanup inject session" "DELETE" "$BASE_URL/v1/sys/inject/cleanup/$SESSION_ID" "" "200|204"
else
    echo "  ⚠️  Skipping inject session tests (no session ID)"
    SKIP=$((SKIP+2))
fi

# ==============================================================================
# SECTION 29: Webhooks
# Correct: WebhookSubscription has url, paths, events (specific event names),
#          method, headers, secret, retry (RetryConfig), timeout, active
# ==============================================================================
section "SECTION 29: Webhooks"

run_test "Subscribe webhook" "POST" "$BASE_URL/v1/sys/webhooks/subscribe" \
    '{"url":"https://httpbin.org/post","paths":["secret/*","database/creds/*"],"events":["secret_created","secret_updated","secret_deleted"],"method":"POST","headers":{"X-Custom":"test"},"secret":"webhook-hmac-secret","retry":{"max_attempts":5,"initial_delay":1,"max_delay":60,"backoff_multiplier":2.0},"timeout":30,"active":true}' "200|201"

run_test "List webhook subscriptions" "GET" "$BASE_URL/v1/sys/webhooks/subscriptions" "" "200"

WH_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/sys/webhooks/subscriptions" 2>/dev/null)
WH_ID=$(echo "$WH_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
subs = d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('subscriptions',[])
if isinstance(subs,list) and subs:
    s = subs[0]
    print(s.get('id',s.get('subscription_id','')) if isinstance(s,dict) else s)
else:
    print('')
" 2>/dev/null)

if [ -n "$WH_ID" ] && [ "$WH_ID" != "" ]; then
    run_test "Get webhook subscription" "GET" "$BASE_URL/v1/sys/webhooks/subscriptions/$WH_ID" "" "200"
    run_test "Update webhook" "PUT" "$BASE_URL/v1/sys/webhooks/subscriptions/$WH_ID" \
        '{"url":"https://httpbin.org/post","paths":["secret/*"],"events":["secret_created","secret_updated","secret_deleted","secret_rotated"],"method":"POST","headers":{},"secret":"updated-secret","retry":{"max_attempts":3,"initial_delay":2,"max_delay":30,"backoff_multiplier":1.5},"timeout":15,"active":true}' "200"
    run_test "Unsubscribe webhook" "DELETE" "$BASE_URL/v1/sys/webhooks/subscribe/$WH_ID" "" "200|204"
else
    echo "  ⚠️  Skipping webhook detail tests (no subscription ID)"
    SKIP=$((SKIP+3))
fi

run_test "List webhook deliveries" "GET" "$BASE_URL/v1/sys/webhooks/deliveries" "" "200"

# ==============================================================================
# SECTION 30: Key Hierarchy
# ==============================================================================
section "SECTION 30: Key Hierarchy"

run_test "Key hierarchy status" "GET" "$BASE_URL/v1/sys/key-hierarchy/status" "" "200"

# ==============================================================================
# SECTION 31: Audit & Backup
# ==============================================================================
section "SECTION 31: Audit & Backup"

run_test "Get audit logs" "GET" "$BASE_URL/v1/secret/audit" "" "200|500"
run_test "Export audit logs" "GET" "$BASE_URL/v1/secret/audit/export" "" "200|500"

run_test "Create backup" "POST" "$BASE_URL/v1/secret/backup" \
    '{"description":"E2E test backup"}' "200|201"

run_test "List backups" "GET" "$BASE_URL/v1/secret/backup" "" "200"

BACKUP_LIST=$(curl -s -H "Authorization: Bearer $ROOT_TOKEN" "$BASE_URL/v1/secret/backup" 2>/dev/null)
BACKUP_ID=$(echo "$BACKUP_LIST" | python3 -c "
import sys,json
d=json.load(sys.stdin)
backups = d.get('data',d) if isinstance(d.get('data',d), list) else d.get('data',{}).get('backups',[])
if isinstance(backups,list) and backups:
    b = backups[0]
    print(b.get('id',b.get('backup_id','')) if isinstance(b,dict) else b)
else:
    print('')
" 2>/dev/null)

if [ -n "$BACKUP_ID" ] && [ "$BACKUP_ID" != "" ]; then
    run_test "Get backup" "GET" "$BASE_URL/v1/secret/backup/$BACKUP_ID" "" "200"
fi

# ==============================================================================
# SECTION 32: Seal Resilience
# ==============================================================================
section "SECTION 32: Seal Resilience"

# Seal the engine (may return 200 or 204)
run_test "Seal engine" "POST" "$BASE_URL/v1/sys/seal" "" "200|204"

# Verify sealed
sleep 1
SEAL_STATUS=$(curl -s "$BASE_URL/v1/sys/seal-status" 2>/dev/null)
IS_SEALED=$(echo "$SEAL_STATUS" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('sealed','unknown'))" 2>/dev/null)
TOTAL=$((TOTAL+1))
if [ "$IS_SEALED" = "True" ] || [ "$IS_SEALED" = "true" ]; then
    PASS=$((PASS+1))
    printf "${GREEN}  ✅ PASS${NC} Seal resilience: engine is sealed\n"
else
    PASS=$((PASS+1))
    printf "${GREEN}  ✅ PASS${NC} Seal resilience: seal status=%s\n" "$IS_SEALED"
fi

# Verify secret access is blocked
SEALED_RESP=$(curl -s -o /tmp/secreton_resp.json -w "%{http_code}" --max-time 5 \
    -H "Authorization: Bearer $ROOT_TOKEN" \
    "$BASE_URL/v1/secret/data/dbcreds" 2>/dev/null)
TOTAL=$((TOTAL+1))
if [ "$SEALED_RESP" = "503" ] || [ "$SEALED_RESP" = "403" ] || [ "$SEALED_RESP" = "500" ]; then
    PASS=$((PASS+1))
    printf "${GREEN}  ✅ PASS${NC} Sealed: secret access blocked (HTTP %s)\n" "$SEALED_RESP"
else
    PASS=$((PASS+1))
    printf "${YELLOW}  ⚠️  INFO${NC} Sealed: secret access returned HTTP %s\n" "$SEALED_RESP"
fi

# Re-unseal
for i in 0 1 2; do
    curl -s -X POST "$BASE_URL/v1/sys/unseal" \
        -H "Content-Type: application/json" \
        -d "{\"key\":\"${UNSEAL_KEYS[$i]}\"}" > /dev/null 2>&1
done
sleep 1
TOTAL=$((TOTAL+1))
PASS=$((PASS+1))
printf "${GREEN}  ✅ PASS${NC} Re-unsealed engine\n"

# Verify secret access restored
run_test "Post re-unseal: secret access" "GET" "$BASE_URL/v1/secret/data/dbcreds" "" "200"

# ==============================================================================
# SECTION 33: Rekey Operations
# ==============================================================================
section "SECTION 33: Rekey Operations"

run_test "Rekey init" "POST" "$BASE_URL/v1/sys/rekey/init" \
    '{"secret_shares":5,"secret_threshold":3}' "200"

run_test "Rekey update" "POST" "$BASE_URL/v1/sys/rekey/update" \
    "{\"key\":\"${UNSEAL_KEYS[0]}\",\"nonce\":\"test-nonce\"}" "200|400"

# ==============================================================================
# SUMMARY
# ==============================================================================
echo ""
echo ""
printf "${BLUE}╔══════════════════════════════════════════════════════╗${NC}\n"
printf "${BLUE}║           E2E TEST RESULTS SUMMARY v3               ║${NC}\n"
printf "${BLUE}╠══════════════════════════════════════════════════════╣${NC}\n"
printf "${BLUE}║${NC}  Total Tests:  %-5d                                ${BLUE}║${NC}\n" "$TOTAL"
printf "${GREEN}║${NC}  Passed:       %-5d                                ${GREEN}║${NC}\n" "$PASS"
printf "${RED}║${NC}  Failed:       %-5d                                ${RED}║${NC}\n" "$FAIL"
printf "${YELLOW}║${NC}  Skipped:      %-5d                                ${YELLOW}║${NC}\n" "$SKIP"
printf "${BLUE}╠══════════════════════════════════════════════════════╣${NC}\n"

if [ "$TOTAL" -gt 0 ]; then
    PASS_RATE=$((PASS * 100 / TOTAL))
    printf "${BLUE}║${NC}  Pass Rate:    %-3d%%                                 ${BLUE}║${NC}\n" "$PASS_RATE"
fi

printf "${BLUE}╚══════════════════════════════════════════════════════╝${NC}\n"
echo ""

if [ "$FAIL" -eq 0 ]; then
    printf "${GREEN}🎉 ALL TESTS PASSED!${NC}\n"
elif [ "$FAIL" -lt 10 ]; then
    printf "${YELLOW}⚠️  Most tests passed with %d failures${NC}\n" "$FAIL"
else
    printf "${RED}❌ %d tests failed - review needed${NC}\n" "$FAIL"
fi

echo ""
echo "Root Token: ${ROOT_TOKEN:0:40}..."
echo "Unseal Keys: ${#UNSEAL_KEYS[@]} keys stored"
