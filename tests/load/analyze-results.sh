#!/bin/bash

# Analyze k6 load test results and generate report
# Usage: ./tests/load/analyze-results.sh [results_dir]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
RESULTS_DIR="${1:-tests/load/results}"
REPORT_FILE="$RESULTS_DIR/analysis_report_$(date +%Y%m%d_%H%M%S).md"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Load Test Results Analysis${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if results directory exists
if [ ! -d "$RESULTS_DIR" ]; then
    echo -e "${RED}Error: Results directory not found: $RESULTS_DIR${NC}"
    exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}Warning: jq not installed. Install for detailed JSON analysis.${NC}"
    echo "  Ubuntu/Debian: sudo apt-get install jq"
    echo "  macOS: brew install jq"
    echo ""
fi

# Function to extract metrics from log file
extract_metrics() {
    local log_file=$1
    local test_name=$2

    echo "## $test_name Test Results" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"

    # Extract summary metrics
    if grep -q "checks" "$log_file"; then
        echo "### Summary Metrics" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        grep -A 20 "checks" "$log_file" | head -25 >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi

    # Extract thresholds
    if grep -q "✓\|✗" "$log_file"; then
        echo "### Thresholds" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        grep "✓\|✗" "$log_file" | while read line; do
            if [[ $line == *"✓"* ]]; then
                echo "- ✅ $line" >> "$REPORT_FILE"
            else
                echo "- ❌ $line" >> "$REPORT_FILE"
            fi
        done
        echo "" >> "$REPORT_FILE"
    fi
}

# Function to analyze performance
analyze_performance() {
    local log_file=$1
    local test_name=$2

    echo "### Performance Analysis" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"

    # Extract key metrics
    local p95_duration=$(grep "http_req_duration" "$log_file" | grep "p(95)" | awk '{print $2}' | head -1)
    local avg_duration=$(grep "http_req_duration" "$log_file" | grep "avg=" | awk -F'avg=' '{print $2}' | awk '{print $1}' | head -1)
    local error_rate=$(grep "http_req_failed" "$log_file" | grep "rate=" | awk -F'rate=' '{print $2}' | awk '{print $1}' | head -1)
    local throughput=$(grep "http_reqs" "$log_file" | grep "/s" | awk '{print $(NF-1)}' | head -1)

    echo "| Metric | Value | Target | Status |" >> "$REPORT_FILE"
    echo "|--------|-------|--------|--------|" >> "$REPORT_FILE"

    # Response time
    if [ -n "$p95_duration" ]; then
        local status="✅"
        if (( $(echo "$p95_duration > 500" | bc -l 2>/dev/null || echo 0) )); then
            status="❌"
        fi
        echo "| Response Time (p95) | $p95_duration | ≤ 500ms | $status |" >> "$REPORT_FILE"
    fi

    # Error rate
    if [ -n "$error_rate" ]; then
        local status="✅"
        if (( $(echo "$error_rate > 0.05" | bc -l 2>/dev/null || echo 0) )); then
            status="❌"
        fi
        echo "| Error Rate | $error_rate | < 5% | $status |" >> "$REPORT_FILE"
    fi

    # Throughput
    if [ -n "$throughput" ]; then
        local status="✅"
        if (( $(echo "$throughput < 100" | bc -l 2>/dev/null || echo 0) )); then
            status="❌"
        fi
        echo "| Throughput | $throughput | ≥ 100 req/s | $status |" >> "$REPORT_FILE"
    fi

    echo "" >> "$REPORT_FILE"
}

# Function to identify bottlenecks
identify_bottlenecks() {
    local log_file=$1

    echo "### Identified Issues" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"

    local has_issues=false

    # Check for high response times
    if grep -q "http_req_duration.*p(95).*[0-9]\{4,\}" "$log_file"; then
        echo "- ⚠️ **High response times detected** (p95 > 1000ms)" >> "$REPORT_FILE"
        echo "  - Recommendation: Check database query performance" >> "$REPORT_FILE"
        echo "  - Action: Run EXPLAIN ANALYZE on slow queries" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        has_issues=true
    fi

    # Check for errors
    if grep -q "http_req_failed.*rate=[0-9]\+\.[0-9]\+%" "$log_file"; then
        local error_rate=$(grep "http_req_failed" "$log_file" | grep -oP 'rate=\K[0-9.]+' | head -1)
        if (( $(echo "$error_rate > 0.01" | bc -l 2>/dev/null || echo 0) )); then
            echo "- ⚠️ **High error rate detected** ($error_rate%)" >> "$REPORT_FILE"
            echo "  - Recommendation: Check service logs for errors" >> "$REPORT_FILE"
            echo "  - Action: kubectl logs -l app=layanan-perlengkapan" >> "$REPORT_FILE"
            echo "" >> "$REPORT_FILE"
            has_issues=true
        fi
    fi

    # Check for low throughput
    if grep -q "http_reqs.*[0-9]\+/s" "$log_file"; then
        local throughput=$(grep "http_reqs" "$log_file" | grep -oP '[0-9.]+(?=/s)' | head -1)
        if (( $(echo "$throughput < 100" | bc -l 2>/dev/null || echo 0) )); then
            echo "- ⚠️ **Low throughput detected** ($throughput req/s)" >> "$REPORT_FILE"
            echo "  - Recommendation: Check connection pool configuration" >> "$REPORT_FILE"
            echo "  - Action: Increase database pool size" >> "$REPORT_FILE"
            echo "" >> "$REPORT_FILE"
            has_issues=true
        fi
    fi

    if [ "$has_issues" = false ]; then
        echo "- ✅ No significant issues detected" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
}

# Generate report header
cat > "$REPORT_FILE" << EOF
# Load Test Analysis Report

**Generated:** $(date)
**Results Directory:** $RESULTS_DIR

## Executive Summary

EOF

# Analyze each test result
total_tests=0
passed_tests=0
failed_tests=0

for log_file in "$RESULTS_DIR"/*.log; do
    if [ -f "$log_file" ]; then
        total_tests=$((total_tests + 1))

        test_name=$(basename "$log_file" .log | sed 's/_[0-9]\{8\}_[0-9]\{6\}.*//')

        echo -e "${BLUE}Analyzing: $test_name${NC}"

        # Check if test passed
        if grep -q "✓" "$log_file" && ! grep -q "✗.*threshold" "$log_file"; then
            passed_tests=$((passed_tests + 1))
            echo "- ✅ **$test_name**: PASSED" >> "$REPORT_FILE"
        else
            failed_tests=$((failed_tests + 1))
            echo "- ❌ **$test_name**: FAILED" >> "$REPORT_FILE"
        fi
    fi
done

echo "" >> "$REPORT_FILE"
echo "**Total Tests:** $total_tests" >> "$REPORT_FILE"
echo "**Passed:** $passed_tests" >> "$REPORT_FILE"
echo "**Failed:** $failed_tests" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Detailed analysis for each test
for log_file in "$RESULTS_DIR"/*.log; do
    if [ -f "$log_file" ]; then
        test_name=$(basename "$log_file" .log | sed 's/_[0-9]\{8\}_[0-9]\{6\}.*//')

        echo -e "${BLUE}Generating detailed analysis for: $test_name${NC}"

        extract_metrics "$log_file" "$test_name"
        analyze_performance "$log_file" "$test_name"
        identify_bottlenecks "$log_file"

        echo "---" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
done

# Add recommendations section
cat >> "$REPORT_FILE" << 'EOF'
## Optimization Recommendations

### Immediate Actions (Quick Wins)

1. **Add Missing Indexes**
   ```sql
   CREATE INDEX idx_kebutuhan_satker ON kebutuhan_bmn(satker_id);
   CREATE INDEX idx_kebutuhan_tahun ON kebutuhan_bmn(tahun_anggaran);
   CREATE INDEX idx_kebutuhan_status ON kebutuhan_bmn(status);
   ```

2. **Implement Caching**
   - Cache dashboard metrics (5 minutes TTL)
   - Cache reference data (1 hour TTL)
   - Cache gap analysis results (1 hour TTL)

3. **Optimize Connection Pool**
   - Increase pool size from 20 to 50
   - Adjust connection timeout to 30 seconds

### Medium-term Actions

1. **Query Optimization**
   - Identify and optimize N+1 queries
   - Use EXPLAIN ANALYZE for slow queries
   - Implement batch loading

2. **Materialized Views**
   - Create materialized views for dashboard metrics
   - Refresh every 5 minutes

3. **Rate Limiting**
   - Verify rate limiting configuration
   - Adjust limits based on load test results

### Long-term Actions

1. **Horizontal Scaling**
   - Increase replicas from 1 to 3
   - Implement autoscaling (HPA)

2. **Database Optimization**
   - Set up read replicas
   - Implement database partitioning
   - Consider connection pooling at application level

3. **Monitoring & Alerting**
   - Set up Prometheus alerts
   - Create Grafana dashboards
   - Implement distributed tracing

## Next Steps

1. Review this analysis report
2. Prioritize optimization actions
3. Implement quick wins first
4. Re-run load tests to verify improvements
5. Document baseline performance
6. Set up continuous performance monitoring

## References

- [Load Test Analysis Guide](./LOAD_TEST_ANALYSIS.md)
- [K8s Load Testing Guide](./K8S_LOAD_TESTING.md)
- [Performance Requirements](../../.kiro/specs/simpel-completion/requirements.md)
EOF

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Analysis Complete${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Report generated: $REPORT_FILE"
echo ""
echo "Summary:"
echo "  Total tests: $total_tests"
echo -e "  ${GREEN}Passed: $passed_tests${NC}"
if [ $failed_tests -gt 0 ]; then
    echo -e "  ${RED}Failed: $failed_tests${NC}"
fi
echo ""

# Open report if possible
if command -v cat &> /dev/null; then
    echo -e "${BLUE}Report preview:${NC}"
    echo ""
    head -50 "$REPORT_FILE"
    echo ""
    echo "... (see full report in $REPORT_FILE)"
fi
