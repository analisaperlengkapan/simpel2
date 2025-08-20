#!/bin/bash
# WASM optimization script for microfrontends

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_DIR="$PROJECT_ROOT/target/wasm32-unknown-unknown"

echo "🎯 WASM Build Optimization Script"

# Build specific microfrontend with optimizations
build_microfrontend() {
    local package_name="$1"
    local build_mode="${2:-release}"
    
    echo "🔨 Building $package_name in $build_mode mode..."
    
    # Set WASM-specific environment
    export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="-C opt-level=s -C lto=thin"
    
    # Build command with specific profile
    if [[ "$build_mode" == "release" ]]; then
        cargo build --release --target=wasm32-unknown-unknown --package="$package_name"
    else
        cargo build --target=wasm32-unknown-unknown --package="$package_name"
    fi
    
    # Post-process WASM file
    optimize_wasm_file "$package_name" "$build_mode"
}

# Optimize individual WASM file
optimize_wasm_file() {
    local package_name="$1"
    local build_mode="$2"
    
    local wasm_file="$WASM_DIR/$build_mode/${package_name}.wasm"
    
    if [[ -f "$wasm_file" ]]; then
        local original_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file")
        
        echo "📊 Original size: $(( original_size / 1024 ))KB"
        
        # Run wasm-opt if available
        if command -v wasm-opt &> /dev/null; then
            echo "⚡ Optimizing with wasm-opt..."
            wasm-opt -Os --enable-bulk-memory "$wasm_file" -o "${wasm_file}.opt"
            mv "${wasm_file}.opt" "$wasm_file"
            
            local optimized_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file")
            local savings=$(( (original_size - optimized_size) * 100 / original_size ))
            
            echo "✅ Optimized size: $(( optimized_size / 1024 ))KB (${savings}% reduction)"
        fi
    else
        echo "⚠️  WASM file not found: $wasm_file"
    fi
}

# Build all microfrontends
build_all_microfrontends() {
    local build_mode="${1:-release}"
    
    echo "🏗️  Building all microfrontends in $build_mode mode..."
    
    local microfrontends=(
        "badiklat-microfrontend"
        "datun-microfrontend" 
        "intel-microfrontend"
        "pemulihan-aset-microfrontend"
        "pengawasan-microfrontend"
        "pidmil-microfrontend"
        "pidsus-microfrontend"
        "pidum-microfrontend"
        "portal-microfrontend"
        "keuangan-microfrontend"
        "perencanaan-microfrontend"
        "perlengkapan-microfrontend"
    )
    
    for microfrontend in "${microfrontends[@]}"; do
        build_microfrontend "$microfrontend" "$build_mode"
    done
}

# Clean WASM builds
clean_wasm() {
    echo "🧹 Cleaning WASM artifacts..."
    rm -rf "$WASM_DIR/debug"/*.wasm "$WASM_DIR/release"/*.wasm 2>/dev/null || true
    echo "✅ WASM artifacts cleaned"
}

# Show WASM statistics
show_wasm_stats() {
    echo "📊 WASM Build Statistics:"
    
    if [[ -d "$WASM_DIR" ]]; then
        echo "Release builds:"
        find "$WASM_DIR/release" -name "*.wasm" -exec ls -lh {} \; 2>/dev/null | while read -r line; do
            echo "  $line"
        done
        
        echo ""
        echo "Debug builds:"
        find "$WASM_DIR/debug" -name "*.wasm" -exec ls -lh {} \; 2>/dev/null | head -5 | while read -r line; do
            echo "  $line"
        done
        
        echo ""
        echo "Total WASM directory size:"
        du -sh "$WASM_DIR" 2>/dev/null || echo "Directory not found"
    else
        echo "No WASM builds found"
    fi
}

# Main execution
main() {
    cd "$PROJECT_ROOT"
    
    case "${1:-build}" in
        "build")
            shift
            if [[ $# -gt 0 ]]; then
                build_microfrontend "$1" "${2:-release}"
            else
                build_all_microfrontends "release"
            fi
            ;;
        "build-debug")
            shift
            if [[ $# -gt 0 ]]; then
                build_microfrontend "$1" "debug"
            else
                build_all_microfrontends "debug"
            fi
            ;;
        "clean")
            clean_wasm
            ;;
        "stats")
            show_wasm_stats
            ;;
        *)
            echo "Usage: $0 {build|build-debug|clean|stats} [package-name]"
            echo "  build [pkg]       - Build microfrontend(s) in release mode"
            echo "  build-debug [pkg] - Build microfrontend(s) in debug mode"
            echo "  clean             - Clean WASM artifacts"
            echo "  stats             - Show build statistics"
            echo ""
            echo "Examples:"
            echo "  $0 build badiklat-microfrontend"
            echo "  $0 build-debug"
            echo "  $0 stats"
            exit 1
            ;;
    esac
}

main "$@"
