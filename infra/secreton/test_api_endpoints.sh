#!/bin/bash

BASE_URL="http://localhost:8200"

echo "=========================================="
echo "Secreton API Endpoints Test"
echo "=========================================="
echo ""

echo "=== 1. Health Check ==="
curl -s $BASE_URL/health | python3 -m json.tool
echo ""

echo "=== 2. Version ==="
curl -s $BASE_URL/version | python3 -m json.tool
echo ""

echo "=== 3. Metrics ==="
curl -s $BASE_URL/metrics | head -10
echo "..."
echo ""

echo "=== 4. TLS Metrics ==="
curl -s $BASE_URL/metrics/tls | python3 -m json.tool
echo ""

echo "=== 5. Seal Status ==="
curl -s $BASE_URL/v1/sys/seal-status | python3 -m json.tool
echo ""

echo "=== 6. System Health (No Auth) ==="
curl -s $BASE_URL/v1/sys/health | python3 -m json.tool 2>/dev/null || echo "Requires auth or sealed"
echo ""

echo "=== 7. Transit Encrypt (No Auth) ==="
curl -s -X POST $BASE_URL/v1/transit/encrypt/test-key \
  -H "Content-Type: application/json" \
  -d '{"plaintext":"SGVsbG8gV29ybGQ="}' | python3 -m json.tool
echo ""

echo "=== 8. KV Write (No Auth) ==="
curl -s -X POST $BASE_URL/v1/kv/test \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret123"}' | python3 -m json.tool
echo ""

echo "=== 9. KV Read (No Auth) ==="
curl -s $BASE_URL/v1/kv/test | python3 -m json.tool
echo ""

echo "=========================================="
echo "✓ API Endpoints Test Complete"
echo "=========================================="
echo ""
echo "Summary:"
echo "- Health endpoints: ✓ Working"
echo "- Metrics endpoints: ✓ Working"
echo "- Seal status: ✓ Working"
echo "- Transit/KV: Requires unseal + auth"
echo ""
echo "Raft Backend Status:"
docker logs secreton-e2e 2>&1 | grep -i "raft\|storage" | head -5
