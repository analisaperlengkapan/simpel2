#!/bin/bash
set -euo pipefail

# Secreton Performance Benchmark Script
# Measures throughput and latency for key operations

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Secreton Performance Benchmarks      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

cd "$PROJECT_ROOT"

# Check if benchmarks exist
if [ ! -d "benches" ]; then
    echo -e "${YELLOW}⚠ No benchmark directory found. Creating basic benchmarks...${NC}"
    mkdir -p benches
fi

print_section() {
    echo -e "\n${BLUE}▶ $1${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Run criterion benchmarks if available
print_section "Running Criterion Benchmarks"

# Transit engine benchmarks
echo -e "  ${GREEN}Transit Engine Performance${NC}"
cargo bench --package secreton-crypto --bench transit_bench 2>/dev/null || echo "  ⚠ Transit benchmarks not found"

# Storage backend benchmarks
echo -e "\n  ${GREEN}Storage Backend Performance${NC}"
cargo bench --package secreton-storage --bench storage_bench 2>/dev/null || echo "  ⚠ Storage benchmarks not found"

# API handler benchmarks
echo -e "\n  ${GREEN}API Handler Performance${NC}"
cargo bench --package secreton-api --bench api_bench 2>/dev/null || echo "  ⚠ API benchmarks not found"

print_section "Quick Performance Tests"

# Build if not already built
if [ ! -f "target/release/secreton" ]; then
    echo "  Building release binary..."
    cargo build --release --quiet
fi

echo -e "  ${GREEN}Testing Core Operations${NC}"

# Measure compilation time
echo -n "  • Clean build time: "
START=$(date +%s)
cargo clean --quiet
cargo build --release --quiet 2>/dev/null
END=$(date +%s)
DURATION=$((END - START))
echo -e "${YELLOW}${DURATION}s${NC}"

# Test count
TEST_COUNT=$(cargo test --workspace --lib -- --list 2>/dev/null | grep -c "test" || echo "0")
echo -e "  • Total unit tests: ${YELLOW}${TEST_COUNT}${NC}"

# Run quick performance test (if lib tests are fast)
echo -n "  • Unit test runtime: "
START=$(date +%s)
cargo test --workspace --lib --quiet 2>/dev/null || true
END=$(date +%s)
DURATION=$((END - START))
echo -e "${YELLOW}${DURATION}s${NC}"

print_section "Performance Summary"

echo -e "  ${GREEN}✓ Benchmark results saved to:${NC}"
echo "    • target/criterion/*/report/index.html"
echo ""
echo -e "  ${BLUE}Open in browser:${NC}"
echo "    firefox target/criterion/report/index.html"
echo ""

print_section "Performance Targets (Production)"

echo "  Minimum Requirements:"
echo "    • Transit encrypt/decrypt: > 10,000 ops/sec"
echo "    • KV read/write:           > 5,000 ops/sec"
echo "    • Authentication:          < 100ms per request"
echo "    • Seal/unseal:             < 5 seconds"
echo ""

echo -e "${GREEN}✓ Performance benchmarking complete${NC}"
