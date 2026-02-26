#!/bin/bash
set -e

export SECRETON_ADDR="http://localhost:8200"

echo "=========================================="
echo " Starting Secreton API Verification Tests "
echo "=========================================="

if [ ! -f credentials.json ]; then
  echo "[1] Engine is not initialized. Initializing..."
  INIT_PAYLOAD='{"secret_shares": 5, "secret_threshold": 3}'
  INIT_RES=$(curl -s -X POST $SECRETON_ADDR/v1/sys/init -d "$INIT_PAYLOAD" -H "Content-Type: application/json")
  echo "Initialization Response: $INIT_RES"

  ROOT_TOKEN=$(echo "$INIT_RES" | grep -o '"root_token":"[^"]*' | grep -o '[^"]*$')

  if [ -z "$ROOT_TOKEN" ]; then
    echo "Initialization failed or root token not found! Perhaps already initialized?"
    exit 1
  fi
  echo "$INIT_RES" > credentials.json
  echo "Root token saved to credentials.json."
else
  echo "[1] Found credentials.json. Using existing token."
  ROOT_TOKEN=$(cat credentials.json | grep -o '"root_token":"[^"]*' | grep -o '[^"]*$')
fi

echo "[2] Unsealing engine..."
if [ -f credentials.json ]; then
  UNSEAL_KEY_1=$(cat credentials.json | grep -o '"keys":\[[^]]*\]' | grep -o '"[a-zA-Z0-9\/+]*"' | tr -d '"' | sed -n 1p)
  UNSEAL_KEY_2=$(cat credentials.json | grep -o '"keys":\[[^]]*\]' | grep -o '"[a-zA-Z0-9\/+]*"' | tr -d '"' | sed -n 2p)
  UNSEAL_KEY_3=$(cat credentials.json | grep -o '"keys":\[[^]]*\]' | grep -o '"[a-zA-Z0-9\/+]*"' | tr -d '"' | sed -n 3p)

  echo "Sending Key 1..."
  curl -s -X POST $SECRETON_ADDR/v1/sys/unseal -d "{\"key\": \"$UNSEAL_KEY_1\"}" -H "Content-Type: application/json"
  echo ""
  echo "Sending Key 2..."
  curl -s -X POST $SECRETON_ADDR/v1/sys/unseal -d "{\"key\": \"$UNSEAL_KEY_2\"}" -H "Content-Type: application/json"
  echo ""
  echo "Sending Key 3..."
  curl -s -X POST $SECRETON_ADDR/v1/sys/unseal -d "{\"key\": \"$UNSEAL_KEY_3\"}" -H "Content-Type: application/json"
  echo ""
fi

echo "[3] Running Core KV Engine Tests..."
echo "Creating a test secret..."
CREATE_KV=$(curl -s -X POST $SECRETON_ADDR/v1/secret/data/test -H "X-Secreton-Token: $ROOT_TOKEN" -H "Content-Type: application/json" -d '{"data":{"hello":"world"}}')
echo "KV Create: $CREATE_KV"

echo "Reading the test secret..."
READ_KV=$(curl -s $SECRETON_ADDR/v1/secret/data/test -H "X-Secreton-Token: $ROOT_TOKEN")
echo "KV Read: $READ_KV"

echo "[4] Testing System Info Endpoint..."
SYS_INFO=$(curl -s $SECRETON_ADDR/v1/sys/info -H "X-Secreton-Token: $ROOT_TOKEN")
echo "SYS INFO: $SYS_INFO"

echo "[5] Testing Secrets Engine Enable Endpoint..."
ENABLE_ENGINE=$(curl -s -X POST $SECRETON_ADDR/v1/sys/mounts/kv2 -H "X-Secreton-Token: $ROOT_TOKEN" -H "Content-Type: application/json" -d '{"type":"kv","description":"A new KV engine v2 mount"}')
echo "Enable Engine Response: $ENABLE_ENGINE"

echo "=========================================="
echo " All automated tests completed! "
echo "=========================================="
