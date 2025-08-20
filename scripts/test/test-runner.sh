#!/bin/bash

# 🧪 SIMPelv2 Test Runner
# Unified test execution for all testing types
# Version: 4.0.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# ====== INLINED UTILITY FUNCTIONS ======
# Colors for output
declare -A COLORS=(
    [RED]='\033[0;31m'
    [GREEN]='\033[0;32m'
    [YELLOW]='\033[1;33m'
    [BLUE]='\033[0;34m'
    [CYAN]='\033[0;36m'
    [PURPLE]='\033[0;35m'
    [NC]='\033[0m'
)

# Logging functions
log() {
    local level="$1"; shift
    local color="${COLORS[${level^^}]:-${COLORS[NC]}}"
    echo -e "${color}[${level^^}]${COLORS[NC]} $*" >&2
}

info() { log "info" "ℹ️  $*"; }
success() { log "success" "✅ $*"; }
warn() { log "warn" "⚠️  $*"; }
error() { log "error" "❌ $*"; }

# ====== TEST FUNCTIONS ======
run_unit_tests() {
    info "Running unit tests..."
    cd "$WORKSPACE_ROOT"

    # Test shared components
    if [[ -f "antarmuka/shared/Cargo.toml" ]]; then
        info "Testing shared components..."
        cargo test --manifest-path "antarmuka/shared/Cargo.toml"
    fi

    success "Unit tests completed"
}

run_performance_tests() {
    info "Running performance tests..."
    if [[ -f "$SCRIPT_DIR/performance/benchmark.sh" ]]; then
        "$SCRIPT_DIR/performance/benchmark.sh" benchmark
    else
        warn "Performance test script not found"
    fi
}

run_security_tests() {
    info "Running security tests..."
    if [[ -f "$SCRIPT_DIR/security/security-scan.sh" ]]; then
        "$SCRIPT_DIR/security/security-scan.sh" scan
    else
        warn "Security test script not found"
    fi
}

run_validation_tests() {
    info "Running environment validation..."
    if [[ -f "$SCRIPT_DIR/validation/validate-env.py" ]]; then
        python3 "$SCRIPT_DIR/validation/validate-env.py"
    else
        warn "Validation script not found"
    fi
}

run_all_tests() {
    info "Running all tests..."
    run_unit_tests
    run_validation_tests
    run_performance_tests
    run_security_tests
    success "All tests completed"
}

# ====== MAIN EXECUTION ======
main() {
    local test_type="${1:-all}"

    case "$test_type" in
        "unit"|"cargo")
            run_unit_tests
            ;;
        "performance"|"perf"|"benchmark")
            run_performance_tests
            ;;
        "security"|"sec")
            run_security_tests
            ;;
        "validation"|"env")
            run_validation_tests
            ;;
        "all")
            run_all_tests
            ;;
        *)
            error "Unknown test type: $test_type"
            echo "Usage: $0 {all|unit|performance|security|validation}"
            exit 1
            ;;
    esac
}

main "$@"
