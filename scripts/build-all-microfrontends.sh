#!/bin/bash

# Build All Microfrontends Script
# This script builds all Leptos 0.7.8 CSR microfrontends with wasm-opt optimization bypass
# Required due to bulk memory operations compatibility issue in current wasm-opt version

set -e

echo "🔧 Building all Leptos 0.7.8 CSR microfrontends..."

MICROFRONTENDS=(
    "antarmuka/portal"
    "antarmuka/intel"
    "antarmuka/badiklat"
    "antarmuka/datun"
    "antarmuka/pembinaan"
    "antarmuka/pemulihan_aset"
    "antarmuka/pengawasan"
    "antarmuka/pidmil"
    "antarmuka/pidsus"
    "antarmuka/pidum"
)

FAILED_BUILDS=()

for mf in "${MICROFRONTENDS[@]}"; do
    echo "📦 Building $mf..."
    cd "/var/www/simpelv2/$mf"

    if TRUNK_SKIP_WASM_OPT=true trunk build --release; then
        echo "✅ $mf build successful"
    else
        echo "❌ $mf build failed"
        FAILED_BUILDS+=($mf)
    fi

    cd /var/www/simpelv2
done

echo ""
echo "📋 Build Summary:"
echo "Total microfrontends: ${#MICROFRONTENDS[@]}"
echo "Failed builds: ${#FAILED_BUILDS[@]}"

if [ ${#FAILED_BUILDS[@]} -ne 0 ]; then
    echo "⚠️  Failed microfrontends:"
    for failed in "${FAILED_BUILDS[@]}"; do
        echo "  - $failed"
    done
    exit 1
else
    echo "🎉 All microfrontends built successfully!"
    echo ""
    echo "📝 Note: All builds use TRUNK_SKIP_WASM_OPT=true workaround"
    echo "   This is required due to bulk memory operations compatibility"
    echo "   issue with current wasm-opt version and Leptos 0.7.8 CSR builds"
fi
