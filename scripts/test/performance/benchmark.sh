#!/bin/bash

# 🏎️ SIMPelv2 Performance Module
# Performance testing, optimization, and monitoring

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

WORKSPACE_ROOT="/var/www/simpelv2"
PERF_RESULTS_DIR="$WORKSPACE_ROOT/.perf-results"

echo -e "${CYAN}🏎️ SIMPelv2 Performance Module${NC}"

# Initialize performance testing
init_perf() {
    echo -e "\n${GREEN}🔧 Initializing Performance Testing${NC}"
    echo "=================================="
    
    cd "$WORKSPACE_ROOT"
    
    # Create performance results directory
    mkdir -p "$PERF_RESULTS_DIR"
    
    echo "📦 Installing performance tools..."
    
    # Install Rust performance tools
    local rust_tools=("cargo-flamegraph" "cargo-benchcmp" "hyperfine")
    
    for tool in "${rust_tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "  📦 Installing $tool..."
            cargo install "$tool"
        else
            echo "  ✅ $tool already installed"
        fi
    done
    
    # Install system performance tools
    echo ""
    echo "🛠️  Checking system performance tools..."
    
    local sys_tools=("wrk" "ab" "htop" "iotop" "perf")
    
    for tool in "${sys_tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            echo "  ✅ $tool available"
        else
            echo "  ⚠️  $tool not available (install with: apt install $tool)"
        fi
    done
    
    echo "✅ Performance testing environment initialized"
}

# Run Rust benchmarks
rust_benchmarks() {
    echo -e "\n${BLUE}🦀 Rust Performance Benchmarks${NC}"
    echo "=============================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🔍 Running Rust benchmarks..."
    
    # Find services with benchmarks
    local services_with_benchmarks=()
    
    find layanan -name "benches" -type d | while read bench_dir; do
        service_name=$(basename "$(dirname "$bench_dir")")
        services_with_benchmarks+=("$service_name")
        
        echo ""
        echo "📊 Running benchmarks for $service_name..."
        
        cd "layanan/$service_name"
        
        # Run benchmarks if they exist
        if [[ -d "benches" && $(find benches -name "*.rs" | wc -l) -gt 0 ]]; then
            echo "  🏃 Running benchmark suite..."
            
            local benchmark_file="$PERF_RESULTS_DIR/bench-$service_name-$(date +%Y%m%d-%H%M%S).txt"
            
            if cargo bench > "$benchmark_file" 2>&1; then
                echo "  ✅ Benchmarks completed: $benchmark_file"
                
                # Show summary
                echo "  📊 Benchmark summary:"
                tail -10 "$benchmark_file" | sed 's/^/    /'
            else
                echo "  ❌ Benchmark failed"
            fi
        else
            echo "  ⚠️  No benchmarks found (create in benches/ directory)"
        fi
        
        cd "$WORKSPACE_ROOT"
    done
    
    if [[ ${#services_with_benchmarks[@]} -eq 0 ]]; then
        echo "⚠️  No services with benchmarks found"
        echo "💡 Create benchmark files in layanan/*/benches/ directories"
    fi
}

# Performance profiling
profile_services() {
    local service="${1:-all}"
    local duration="${2:-30}"
    
    echo -e "\n${PURPLE}🔬 Performance Profiling${NC}"
    echo "======================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🔍 Profiling services for $duration seconds..."
    
    case "$service" in
        "all")
            # Profile all running services
            echo "📊 Profiling all services..."
            
            # Find running Rust processes
            local rust_processes=$(pgrep -f "target.*release" | head -5)
            
            if [[ -n "$rust_processes" ]]; then
                for pid in $rust_processes; do
                    local proc_name=$(ps -p "$pid" -o comm= 2>/dev/null || echo "unknown")
                    echo "  🔍 Profiling process $pid ($proc_name)..."
                    
                    local profile_file="$PERF_RESULTS_DIR/profile-$proc_name-$(date +%Y%m%d-%H%M%S).svg"
                    
                    # Use perf to profile
                    if command -v perf >/dev/null 2>&1; then
                        timeout "$duration"s perf record -p "$pid" -o "$PERF_RESULTS_DIR/perf-$proc_name.data" >/dev/null 2>&1 || true
                        
                        # Generate flamegraph if cargo-flamegraph is available
                        if command -v cargo-flamegraph >/dev/null 2>&1; then
                            perf script -i "$PERF_RESULTS_DIR/perf-$proc_name.data" | \
                                cargo flamegraph --output "$profile_file" >/dev/null 2>&1 || true
                            
                            if [[ -f "$profile_file" ]]; then
                                echo "    ✅ Flamegraph generated: $profile_file"
                            fi
                        fi
                    else
                        echo "    ⚠️  perf not available for profiling"
                    fi
                done
            else
                echo "  ⚠️  No running Rust services found"
            fi
            ;;
            
        *)
            echo "📊 Profiling specific service: $service"
            
            local service_path="layanan/$service"
            if [[ -d "$service_path" && -f "$service_path/Cargo.toml" ]]; then
                cd "$service_path"
                
                echo "  🚀 Starting service with profiling..."
                local profile_file="$PERF_RESULTS_DIR/profile-$service-$(date +%Y%m%d-%H%M%S).svg"
                
                # Run with flamegraph profiling
                if command -v cargo-flamegraph >/dev/null 2>&1; then
                    timeout "$duration"s cargo flamegraph --bin "$service" --output "$profile_file" >/dev/null 2>&1 || true
                    
                    if [[ -f "$profile_file" ]]; then
                        echo "  ✅ Profiling completed: $profile_file"
                    else
                        echo "  ❌ Profiling failed"
                    fi
                else
                    echo "  ❌ cargo-flamegraph not available"
                fi
                
                cd "$WORKSPACE_ROOT"
            else
                echo "❌ Service not found: $service"
                return 1
            fi
            ;;
    esac
}

# Load testing
load_test() {
    local target_url="${1:-http://localhost:8080}"
    local duration="${2:-60s}"
    local connections="${3:-100}"
    
    echo -e "\n${YELLOW}🔥 Load Testing${NC}"
    echo "==============="
    
    echo "🎯 Target: $target_url"
    echo "⏰ Duration: $duration"
    echo "🔗 Connections: $connections"
    echo ""
    
    # Create test results file
    local test_file="$PERF_RESULTS_DIR/load-test-$(date +%Y%m%d-%H%M%S).txt"
    
    echo "🚀 Starting load test..."
    
    # Use wrk if available
    if command -v wrk >/dev/null 2>&1; then
        echo "  🔧 Using wrk for load testing..."
        
        wrk -t4 -c"$connections" -d"$duration" "$target_url" > "$test_file" 2>&1
        
        echo "✅ Load test completed: $test_file"
        echo ""
        echo "📊 Load test summary:"
        grep -E "(Requests/sec|Latency|Transfer/sec)" "$test_file" | sed 's/^/  /'
        
    # Use ab as fallback
    elif command -v ab >/dev/null 2>&1; then
        echo "  🔧 Using Apache Bench (ab) for load testing..."
        
        local requests=$((connections * 10))  # Simple calculation for total requests
        ab -n "$requests" -c "$connections" "$target_url" > "$test_file" 2>&1
        
        echo "✅ Load test completed: $test_file"
        echo ""
        echo "📊 Load test summary:"
        grep -E "(Requests per second|Time per request)" "$test_file" | sed 's/^/  /'
        
    else
        echo "❌ No load testing tools available"
        echo "💡 Install wrk or apache2-utils (ab) for load testing"
        return 1
    fi
}

# Memory usage analysis
memory_analysis() {
    echo -e "\n${GREEN}💾 Memory Usage Analysis${NC}"
    echo "======================="
    
    echo "🔍 Analyzing memory usage..."
    
    # System memory overview
    echo ""
    echo "🖥️  System Memory:"
    free -h | head -2 | sed 's/^/  /'
    
    # Process memory usage
    echo ""
    echo "🦀 Rust Process Memory Usage:"
    
    # Find Rust processes
    if pgrep -f "target.*release" >/dev/null; then
        ps -eo pid,ppid,cmd,pmem,rss --sort=-rss | grep -E "(target.*release|PID)" | head -6 | sed 's/^/  /'
    else
        echo "  ⚠️  No running Rust processes found"
    fi
    
    # Docker container memory (if available)
    if command -v docker >/dev/null 2>&1; then
        echo ""
        echo "🐳 Docker Container Memory:"
        
        if docker ps --format "{{.Names}}" | head -1 >/dev/null; then
            docker stats --no-stream --format "table {{.Container}}\t{{.MemUsage}}\t{{.MemPerc}}" | sed 's/^/  /'
        else
            echo "  ⚠️  No running Docker containers"
        fi
    fi
    
    # Memory leak detection suggestions
    echo ""
    echo "🔍 Memory Leak Detection:"
    echo "  💡 Use valgrind with: valgrind --tool=memcheck --leak-check=full ./target/release/service-name"
    echo "  💡 Use cargo-valgrind: cargo install cargo-valgrind && cargo valgrind run"
    echo "  💡 Monitor with: watch -n 1 'ps -eo pid,cmd,rss --sort=-rss | head -10'"
}

# Performance optimization suggestions
optimize_suggestions() {
    echo -e "\n${CYAN}⚡ Performance Optimization Suggestions${NC}"
    echo "======================================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🔍 Analyzing codebase for optimization opportunities..."
    
    # Cargo.toml optimization analysis
    echo ""
    echo "📦 Cargo Configuration:"
    
    if grep -q "\[profile.release\]" Cargo.toml; then
        echo "  ✅ Release profile configured"
        
        # Check specific optimizations
        if grep -A 5 "\[profile.release\]" Cargo.toml | grep -q "lto = true"; then
            echo "  ✅ Link-time optimization enabled"
        else
            echo "  💡 Consider adding: lto = true"
        fi
        
        if grep -A 5 "\[profile.release\]" Cargo.toml | grep -q "codegen-units = 1"; then
            echo "  ✅ Single codegen unit configured"
        else
            echo "  💡 Consider adding: codegen-units = 1"
        fi
    else
        echo "  ⚠️  No release profile optimization"
        echo "  💡 Add to Cargo.toml:"
        echo "     [profile.release]"
        echo "     lto = true"
        echo "     codegen-units = 1"
        echo "     panic = 'abort'"
    fi
    
    # Code analysis
    echo ""
    echo "🦀 Code Optimization:"
    
    # Check for common performance issues
    local perf_issues=0
    
    if grep -r "clone()" --include="*.rs" layanan/ | wc -l | awk '{if($1>50) print "  ⚠️  High clone() usage ("$1" instances) - consider references"}'; then
        ((perf_issues++))
    fi
    
    if grep -r "String::from" --include="*.rs" layanan/ | wc -l | awk '{if($1>20) print "  ⚠️  Frequent String allocation ("$1" instances) - consider &str"}'; then
        ((perf_issues++))
    fi
    
    if grep -r "unwrap()" --include="*.rs" layanan/ | wc -l | awk '{if($1>30) print "  ⚠️  Frequent unwrap() usage ("$1" instances) - consider proper error handling"}'; then
        ((perf_issues++))
    fi
    
    if [[ $perf_issues -eq 0 ]]; then
        echo "  ✅ No obvious performance issues detected"
    fi
    
    # General optimization tips
    echo ""
    echo "🎯 General Optimization Tips:"
    echo "  1. Use Vec::with_capacity() for known-size collections"
    echo "  2. Prefer &str over String for function parameters"
    echo "  3. Use Arc<str> for shared string data"
    echo "  4. Consider using Box<dyn Trait> for polymorphism"
    echo "  5. Profile with cargo flamegraph for hot paths"
    echo "  6. Use async/await for I/O bound operations"
    echo "  7. Consider SIMD for data-parallel operations"
}

# Generate performance report
performance_report() {
    echo -e "\n${PURPLE}📊 Performance Report Generation${NC}"
    echo "==============================="
    
    cd "$WORKSPACE_ROOT"
    
    local report_file="$PERF_RESULTS_DIR/performance-report-$(date +%Y%m%d-%H%M%S).md"
    
    echo "📝 Generating comprehensive performance report..."
    
    cat > "$report_file" << EOF
# SIMPelv2 Performance Report

**Generated:** $(date)  
**Project:** SIMPelv2  
**Location:** $WORKSPACE_ROOT

## Executive Summary

This report provides a comprehensive performance analysis of the SIMPelv2 platform.

## System Information

**Hardware:**
- CPU: $(cat /proc/cpuinfo | grep "model name" | head -1 | cut -d: -f2 | xargs)
- Memory: $(free -h | grep Mem | awk '{print $2}')
- Storage: $(df -h / | tail -1 | awk '{print $2}')

**Software:**
- OS: $(uname -s) $(uname -r)
- Rust: $(rustc --version 2>/dev/null || echo "Not available")
- Docker: $(docker --version 2>/dev/null || echo "Not available")

## Performance Metrics

### Build Performance
- Workspace build time: $(time cargo build --release 2>&1 | grep real | awk '{print $2}' || echo "N/A")
- Number of services: $(find layanan -maxdepth 1 -type d ! -name layanan | wc -l)
- Number of frontends: $(find antarmuka -maxdepth 1 -type d ! -name antarmuka -a ! -name shared | wc -l)

### Runtime Performance
$(if [[ -d "$PERF_RESULTS_DIR" ]] && ls "$PERF_RESULTS_DIR"/*.txt >/dev/null 2>&1; then
    echo "Recent performance test results:"
    ls -lt "$PERF_RESULTS_DIR"/*.txt | head -3 | while read line; do
        echo "- $(basename "$(echo "$line" | awk '{print $9}')")"
    done
else
    echo "No performance test results available"
fi)

### Memory Usage
$(free -h | grep Mem)

### Optimization Status
$(if grep -q "\[profile.release\]" Cargo.toml; then
    echo "✅ Release profile optimization: Enabled"
else
    echo "⚠️ Release profile optimization: Not configured"
fi)

## Recommendations

1. **Build Optimization**: Ensure release profile is properly configured
2. **Runtime Monitoring**: Implement continuous performance monitoring
3. **Load Testing**: Regular load testing of critical endpoints
4. **Memory Profiling**: Monitor for memory leaks and excessive allocation
5. **Database Performance**: Optimize database queries and connections

## Next Steps

- [ ] Implement automated performance testing in CI/CD
- [ ] Set up performance monitoring dashboards
- [ ] Conduct regular performance reviews
- [ ] Optimize identified bottlenecks

---

*This report was generated automatically by SIMPelv2 performance tools.*
EOF
    
    echo "✅ Performance report generated: $report_file"
    echo "📊 View with: cat $report_file"
}

# Main function
main() {
    case "${1:-help}" in
        "init"|"setup")
            init_perf
            ;;
        "benchmark"|"bench")
            rust_benchmarks
            ;;
        "profile")
            profile_services "${2:-all}" "${3:-30}"
            ;;
        "load"|"load-test")
            load_test "${2:-http://localhost:8080}" "${3:-60s}" "${4:-100}"
            ;;
        "memory"|"mem")
            memory_analysis
            ;;
        "optimize"|"suggestions")
            optimize_suggestions
            ;;
        "report")
            performance_report
            ;;
        "all"|"full")
            echo "🏎️ Running comprehensive performance analysis..."
            init_perf
            rust_benchmarks
            memory_analysis
            optimize_suggestions
            performance_report
            ;;
        "help"|"--help"|"-h")
            echo ""
            echo "SIMPelv2 Performance Module"
            echo ""
            echo "Usage: $0 [command] [options]"
            echo ""
            echo "Commands:"
            echo "  init              - Initialize performance testing environment"
            echo "  benchmark         - Run Rust benchmarks"
            echo "  profile [service] [duration] - Profile service performance"
            echo "  load [url] [duration] [connections] - Run load tests"
            echo "  memory            - Analyze memory usage"
            echo "  optimize          - Show optimization suggestions"
            echo "  report            - Generate performance report"
            echo "  all               - Run complete performance analysis"
            echo "  help              - Show this help"
            echo ""
            echo "Examples:"
            echo "  $0 init                           # Setup performance tools"
            echo "  $0 benchmark                      # Run benchmarks"
            echo "  $0 profile dasbor 60             # Profile service for 60s"
            echo "  $0 load http://localhost:8080 30s 50  # Load test"
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo "Use '$0 help' for available commands"
            exit 1
            ;;
    esac
}

main "$@"
