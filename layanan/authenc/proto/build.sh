#!/bin/bash
# Optimized gRPC code generation script

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR/generated"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Simpelv2 gRPC Code Generation        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}\n"

# Validate proto files
echo -e "${YELLOW}→ Validating proto files...${NC}"
PROTO_COUNT=0
for proto in "$SCRIPT_DIR"/*.proto; do
    if [ -f "$proto" ]; then
        echo -e "  ✓ $(basename "$proto")"
        ((PROTO_COUNT++))
    fi
done
echo -e "  Found ${GREEN}$PROTO_COUNT${NC} proto files\n"

# Check protoc
if ! command -v protoc &> /dev/null; then
    echo -e "${RED}✗ protoc not found${NC}"
    echo -e "Install: ${YELLOW}sudo apt install protobuf-compiler${NC}"
    exit 1
fi

echo -e "${GREEN}→ Protoc: $(protoc --version)${NC}\n"

# Create output directory
mkdir -p "$OUT_DIR/rust"

# Generate Rust code
echo -e "${YELLOW}→ Generating Rust code...${NC}"
if protoc \
    --proto_path="$SCRIPT_DIR" \
    --rust_out="$OUT_DIR/rust" \
    --tonic_out="$OUT_DIR/rust" \
    "$SCRIPT_DIR"/*.proto 2>&1; then
    
    RUST_FILES=$(find "$OUT_DIR/rust" -name "*.rs" 2>/dev/null | wc -l)
    echo -e "${GREEN}✓ Generated $RUST_FILES Rust files${NC}\n"
else
    echo -e "${RED}✗ Failed${NC}\n"
    exit 1
fi

# Summary
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Complete                              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo -e "\n${GREEN}Output:${NC} $OUT_DIR/rust/"
echo -e "\n${YELLOW}Usage:${NC}"
echo -e "  cp $OUT_DIR/rust/*.rs <project>/src/proto/"
