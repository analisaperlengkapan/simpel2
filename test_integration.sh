#!/bin/bash
set -e

# Configuration
API_URL="http://localhost:8080/api/v1/intel"
SERVICE_NAME="layanan-intel"

echo "Starting Backend Integration Test..."

# 1. Start the backend in the background (using the already built binary)
echo "Starting backend..."
# Use the direct binary path to avoid cargo recompilation checks
./target/debug/layanan-intel &
PID=$!
sleep 5 # Wait for startup

# Ensure cleanup
trap "kill $PID" EXIT

# 2. Check Health
echo "Checking health..."
curl -f "$API_URL/health" || exit 1
echo -e "\nHealth check passed."

# 3. Create Operation
echo "Creating operation..."
RESPONSE=$(curl -s -X POST "$API_URL/operations" \
  -H "Content-Type: application/json" \
  -d '{
    "operation_name": "Test Op Integration",
    "operation_type": "SurveillanceOperation",
    "priority": "High",
    "target_description": "Integration test target",
    "classification_level": "Secret",
    "budget_allocated": 1000000.0
  }')

echo "Create Response: $RESPONSE"
if [[ "$RESPONSE" == *"Test Op Integration"* ]]; then
    echo "Creation successful."
else
    echo "Failed to create operation."
    exit 1
fi

# 4. List Operations and Verify
echo "Listing operations..."
LIST_RESPONSE=$(curl -s "$API_URL/operations")

if [[ "$LIST_RESPONSE" == *"Test Op Integration"* ]]; then
  echo "SUCCESS: Operation found in list."
else
  echo "FAILURE: Operation not found."
  exit 1
fi

# 5. Get Operation Detail (extract ID slightly hackily without jq)
# Assuming ID is at the start or we search by name confirmation again
ID=$(echo $RESPONSE | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Testing Detail for ID: $ID"

DETAIL_RESPONSE=$(curl -s "$API_URL/operations/$ID")
if [[ "$DETAIL_RESPONSE" == *"$ID"* ]]; then
     echo "SUCCESS: Detail retrieved."
else
     echo "FAILURE: Detail retrieval failed."
     exit 1
fi

echo "All integration tests passed."
