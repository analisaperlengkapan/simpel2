#!/bin/bash

# Run k6 load tests in Kubernetes using kubectl
# Usage: ./tests/load/run-k8s-load-tests.sh [namespace] [test_type]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
NAMESPACE="${1:-simpelv2-staging}"
TEST_TYPE="${2:-all}"  # all, dashboard, api, search
RESULTS_DIR="tests/load/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}SIMPEL K8s Load Testing${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Configuration:"
echo "  Namespace: $NAMESPACE"
echo "  Test Type: $TEST_TYPE"
echo ""

# Check kubectl
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}Error: kubectl is not installed${NC}"
    exit 1
fi

# Check namespace exists
if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
    echo -e "${RED}Error: Namespace $NAMESPACE does not exist${NC}"
    exit 1
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to create secret if not exists
create_secret_if_needed() {
    if ! kubectl get secret load-test-credentials -n "$NAMESPACE" &> /dev/null; then
        echo -e "${YELLOW}Creating load-test-credentials secret...${NC}"
        read -sp "Enter test password: " TEST_PASSWORD
        echo ""
        kubectl create secret generic load-test-credentials \
            --from-literal=password="$TEST_PASSWORD" \
            -n "$NAMESPACE"
    fi
}

# Function to apply ConfigMap and wait
apply_configmap() {
    echo -e "${BLUE}Applying k6 test scripts ConfigMap...${NC}"
    kubectl apply -f tests/load/k8s-load-test-job.yaml -n "$NAMESPACE" | grep ConfigMap
    sleep 2
}

# Function to run a specific test
run_test() {
    local test_name=$1
    local job_name="k6-${test_name}-load-test"

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Running: ${test_name} load test${NC}"
    echo -e "${GREEN}========================================${NC}"

    # Delete existing job if exists
    if kubectl get job "$job_name" -n "$NAMESPACE" &> /dev/null; then
        echo -e "${YELLOW}Deleting existing job...${NC}"
        kubectl delete job "$job_name" -n "$NAMESPACE" --wait=true
        sleep 2
    fi

    # Create job
    echo -e "${BLUE}Creating job...${NC}"
    kubectl apply -f tests/load/k8s-load-test-job.yaml -n "$NAMESPACE" | grep "job.batch/$job_name"

    # Wait for job to start
    echo -e "${BLUE}Waiting for job to start...${NC}"
    sleep 5

    # Get pod name
    POD_NAME=$(kubectl get pods -n "$NAMESPACE" -l "app=k6-load-test,test-type=${test_name}" \
        --field-selector=status.phase!=Succeeded,status.phase!=Failed \
        -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")

    if [ -z "$POD_NAME" ]; then
        # Try to get any pod from this job
        POD_NAME=$(kubectl get pods -n "$NAMESPACE" -l "job-name=${job_name}" \
            -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    fi

    if [ -z "$POD_NAME" ]; then
        echo -e "${RED}Error: Could not find pod for job $job_name${NC}"
        return 1
    fi

    echo -e "${BLUE}Pod: $POD_NAME${NC}"
    echo -e "${BLUE}Streaming logs...${NC}"
    echo ""

    # Stream logs
    kubectl logs -f "$POD_NAME" -n "$NAMESPACE" 2>&1 | tee "$RESULTS_DIR/${test_name}_${TIMESTAMP}.log"

    # Check job status
    echo ""
    echo -e "${BLUE}Checking job status...${NC}"

    # Wait for job to complete
    kubectl wait --for=condition=complete --timeout=30m "job/$job_name" -n "$NAMESPACE" 2>/dev/null && {
        echo -e "${GREEN}✓ ${test_name} load test completed successfully${NC}"

        # Save results
        kubectl logs "$POD_NAME" -n "$NAMESPACE" > "$RESULTS_DIR/${test_name}_${TIMESTAMP}_full.log"

        return 0
    } || {
        echo -e "${RED}✗ ${test_name} load test failed or timed out${NC}"

        # Get pod status
        kubectl get pod "$POD_NAME" -n "$NAMESPACE"

        # Save error logs
        kubectl logs "$POD_NAME" -n "$NAMESPACE" > "$RESULTS_DIR/${test_name}_${TIMESTAMP}_error.log"

        return 1
    }
}

# Function to cleanup
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"

    # Delete jobs
    kubectl delete jobs -l app=k6-load-test -n "$NAMESPACE" --wait=false 2>/dev/null || true

    echo -e "${GREEN}Cleanup complete${NC}"
}

# Trap cleanup on exit
trap cleanup EXIT

# Create secret if needed
create_secret_if_needed

# Apply ConfigMap
apply_configmap

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Run tests based on type
case "$TEST_TYPE" in
    dashboard)
        TOTAL_TESTS=1
        if run_test "dashboard"; then
            PASSED_TESTS=1
        else
            FAILED_TESTS=1
        fi
        ;;
    api)
        TOTAL_TESTS=1
        if run_test "api"; then
            PASSED_TESTS=1
        else
            FAILED_TESTS=1
        fi
        ;;
    search)
        TOTAL_TESTS=1
        if run_test "search"; then
            PASSED_TESTS=1
        else
            FAILED_TESTS=1
        fi
        ;;
    all)
        # Dashboard test
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        if run_test "dashboard"; then
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi

        sleep 10

        # API test
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        if run_test "api"; then
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi

        sleep 10

        # Search test
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        if run_test "search"; then
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
        ;;
    *)
        echo -e "${RED}Error: Invalid test type: $TEST_TYPE${NC}"
        echo "Valid types: all, dashboard, api, search"
        exit 1
        ;;
esac

# Summary
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Test Summary${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Total tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
fi
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""

# Generate summary
SUMMARY_FILE="$RESULTS_DIR/k8s_summary_${TIMESTAMP}.txt"
cat > "$SUMMARY_FILE" << EOF
SIMPEL K8s Load Testing Summary
================================

Date: $(date)
Namespace: $NAMESPACE
Test Type: $TEST_TYPE

Test Results:
-------------
Total: $TOTAL_TESTS
Passed: $PASSED_TESTS
Failed: $FAILED_TESTS

Logs saved in: $RESULTS_DIR
EOF

echo "Summary: $SUMMARY_FILE"

# Exit with appropriate code
if [ $FAILED_TESTS -gt 0 ]; then
    exit 1
else
    exit 0
fi
