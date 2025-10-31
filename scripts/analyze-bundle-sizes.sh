#!/bin/bash
# Bundle Size Analysis Script for SIMPelv2
# Analyzes WASM bundle sizes across all microfrontends

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Thresholds (in KB)
WARN_THRESHOLD=400
ERROR_THRESHOLD=500

echo -e "${BLUE}📦 SIMPelv2 Bundle Size Analysis${NC}"
echo "=================================="
echo ""

# Function to get file size in KB
get_size_kb() {
    local file=$1
    if [ -f "$file" ]; then
        # Get size in bytes, convert to KB
        local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
        echo $((size / 1024))
    else
        echo "0"
    fi
}

# Function to analyze a microfrontend
analyze_mf() {
    local name=$1
    local path=$2

    if [ ! -d "$path" ]; then
        echo -e "${YELLOW}⚠️  $name: Directory not found${NC}"
        return
    fi

    local wasm_file=$(find "$path/dist" -name "*.wasm" 2>/dev/null | head -1)
    local js_file=$(find "$path/dist" -name "*.js" -not -name "*-*.js" 2>/dev/null | head -1)

    if [ -z "$wasm_file" ]; then
        echo -e "${YELLOW}⚠️  $name: Not built yet (run 'trunk build --release')${NC}"
        return
    fi

    local wasm_size=$(get_size_kb "$wasm_file")
    local js_size=$(get_size_kb "$js_file")
    local total_size=$((wasm_size + js_size))

    # Determine status
    local status="${GREEN}✅${NC}"
    if [ $total_size -gt $ERROR_THRESHOLD ]; then
        status="${RED}❌${NC}"
    elif [ $total_size -gt $WARN_THRESHOLD ]; then
        status="${YELLOW}⚠️${NC}"
    fi

    printf "%-20s %s  WASM: %4d KB  JS: %4d KB  Total: %4d KB\n" \
        "$name" "$status" "$wasm_size" "$js_size" "$total_size"

    # Return total size for summary (use safe filename)
    local safe_name=$(echo "$name" | tr ' /' '_')
    echo "$total_size" > /tmp/bundle_size_$safe_name
}

# Analyze all microfrontends
echo "Analyzing microfrontends..."
echo ""

analyze_mf "Portal" "antarmuka/portal"
analyze_mf "Badiklat" "antarmuka/badiklat"
analyze_mf "Datun" "antarmuka/datun"
analyze_mf "Intel" "antarmuka/intel"
analyze_mf "Pidum" "antarmuka/pidum"
analyze_mf "Pidsus" "antarmuka/pidsus"
analyze_mf "Pidmil" "antarmuka/pidmil"
analyze_mf "Pengawasan" "antarmuka/pengawasan"
analyze_mf "Pemulihan Aset" "antarmuka/pemulihan_aset"
analyze_mf "Pembinaan/Keuangan" "antarmuka/pembinaan/keuangan"
analyze_mf "Pembinaan/Perencanaan" "antarmuka/pembinaan/perencanaan"
analyze_mf "Pembinaan/Perlengkapan" "antarmuka/pembinaan/perlengkapan"

echo ""
echo "=================================="

# Calculate total size
total=0
count=0
for file in /tmp/bundle_size_*; do
    if [ -f "$file" ]; then
        size=$(cat "$file")
        total=$((total + size))
        count=$((count + 1))
        rm "$file"
    fi
done

if [ $count -gt 0 ]; then
    avg=$((total / count))
    echo -e "${BLUE}Summary:${NC}"
    echo "  Total bundles analyzed: $count"
    echo "  Average bundle size: ${avg} KB"
    echo "  Total size (all bundles): ${total} KB"
    echo ""

    if [ $avg -lt $WARN_THRESHOLD ]; then
        echo -e "${GREEN}✅ All bundles are within optimal size!${NC}"
    elif [ $avg -lt $ERROR_THRESHOLD ]; then
        echo -e "${YELLOW}⚠️  Some bundles exceed recommended size${NC}"
        echo "   Consider optimization techniques:"
        echo "   - Enable LTO in Cargo.toml"
        echo "   - Use -Oz optimization in Trunk.toml"
        echo "   - Remove unused dependencies"
    else
        echo -e "${RED}❌ Bundles are too large!${NC}"
        echo "   Immediate action required:"
        echo "   - Review and remove unused dependencies"
        echo "   - Split large components"
        echo "   - Enable all optimization flags"
    fi
else
    echo -e "${YELLOW}⚠️  No bundles found. Build microfrontends first:${NC}"
    echo "   cd antarmuka/portal && trunk build --release"
fi

echo ""
echo "Thresholds:"
echo "  ✅ Optimal: < ${WARN_THRESHOLD} KB"
echo "  ⚠️  Warning: ${WARN_THRESHOLD}-${ERROR_THRESHOLD} KB"
echo "  ❌ Error: > ${ERROR_THRESHOLD} KB"
