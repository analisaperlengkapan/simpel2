#!/bin/bash
set -euo pipefail

# Secreton Integration Test Script
# Runs comprehensive end-to-end tests with reporting

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Secreton Integration Test Suite     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Function to print test section
print_section() {
    echo -e "\n${BLUE}▶ $1${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Function to run test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"

    ((TOTAL_TESTS++))
    echo -n "  Testing: $test_name..."

    if eval "$test_command" > /dev/null 2>&1; then
        echo -e " ${GREEN}✓ PASS${NC}"
        ((PASSED_TESTS++))
        return 0
    else
        echo -e " ${RED}✗ FAIL${NC}"
        ((FAILED_TESTS++))
        return 1
    fi
}

# Change to project root
cd "$PROJECT_ROOT"

print_section "Phase 1: Build Verification"
run_test "Workspace builds cleanly" "cargo build --workspace --release"
run_test "All targets compile" "cargo check --workspace --all-targets"

print_section "Phase 2: Unit Tests"
run_test "Core library tests" "cargo test --package secreton-core --lib"
run_test "Crypto library tests" "cargo test --package secreton-crypto --lib"
run_test "Storage library tests" "cargo test --package secreton-storage --lib"
run_test "API library tests" "cargo test --package secreton-api --lib"

print_section "Phase 3: Integration Tests"
run_test "Transit engine integration" "cargo test --test integration_tests transit"
run_test "KV secrets engine integration" "cargo test --test integration_tests kv"
run_test "Authentication integration" "cargo test --test auth_methods_test"
run_test "Dynamic secrets integration" "cargo test --test dynamic_secrets_test"

print_section "Phase 4: End-to-End Workflows"
run_test "Complete secret lifecycle" "cargo test --test comprehensive_integration_tests test_end_to_end_secret_encryption_workflow"
run_test "Cross-component data flow" "cargo test --test comprehensive_integration_tests test_cross_component_data_flow"
run_test "Policy enforcement" "cargo test --test comprehensive_integration_tests test_policy_enforcement_across_components"

print_section "Phase 5: Performance & Stress Tests"
run_test "Concurrent operations" "cargo test --test comprehensive_integration_tests test_concurrent_multi_component_operations"
run_test "System resilience" "cargo test --test comprehensive_integration_tests test_system_recovery_and_resilience"

print_section "Phase 6: Security Tests"
run_test "Error propagation" "cargo test --test comprehensive_integration_tests test_error_propagation_across_components"
run_test "Security validation" "cargo test --test security_validation"

print_section "Phase 7: Storage Backend Tests"
run_test "Memory backend" "cargo test --package secreton-storage memory"
run_test "File backend" "cargo test --package secreton-storage file"

# Print summary
echo ""
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Test Summary                          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Tests:   ${TOTAL_TESTS}"
echo -e "  Passed:        ${GREEN}${PASSED_TESTS}${NC}"
echo -e "  Failed:        ${RED}${FAILED_TESTS}${NC}"
echo ""

# Calculate pass rate
if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$(( (PASSED_TESTS * 100) / TOTAL_TESTS ))
    echo -e "  Pass Rate:     ${PASS_RATE}%"
    echo ""

    if [ $PASS_RATE -eq 100 ]; then
        echo -e "${GREEN}✓ All tests passed! System is production ready.${NC}"
        exit 0
    elif [ $PASS_RATE -ge 80 ]; then
        echo -e "${YELLOW}⚠ Most tests passed. Review failures before deployment.${NC}"
        exit 1
    else
        echo -e "${RED}✗ Critical failures detected. System not ready for production.${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ No tests were run!${NC}"
    exit 1
fi
