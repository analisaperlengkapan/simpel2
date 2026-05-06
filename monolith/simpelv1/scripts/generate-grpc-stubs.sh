#!/bin/bash
# Generate PHP gRPC stubs from proto files
# Usage: ./scripts/generate-grpc-stubs.sh

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Generating PHP gRPC stubs for SIMPEL services...${NC}"

# Directory setup
PROTO_DIR="layanan/proto"
OUTPUT_DIR="monolith/simpelv1/app/Services/Grpc/Generated"
VENDOR_BIN="monolith/simpelv1/vendor/bin"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo -e "${GREEN}✓ Output directory created${NC}"

# Install protobuf compiler and PHP plugin if not present
if ! command -v protoc &> /dev/null; then
    echo -e "${BLUE}Installing protoc...${NC}"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install protobuf
    else
        apt-get update && apt-get install -y protobuf-compiler
    fi
fi

echo -e "${GREEN}✓ protoc installed${NC}"

# Generate stubs for each service
declare -a services=("authenc" "secreton" "integrasi")

for service in "${services[@]}"; do
    echo -e "${BLUE}Generating stubs for $service...${NC}"
    
    proto_files=$(find "$PROTO_DIR" -name "*.proto" | grep -E "(common|$service)" || true)
    
    if [ -z "$proto_files" ]; then
        echo -e "${BLUE}No proto files found for $service, skipping...${NC}"
        continue
    fi
    
    # Generate PHP stubs using protoc
    protoc \
        --php_out="$OUTPUT_DIR" \
        --php-grpc_out="$OUTPUT_DIR" \
        --plugin=protoc-gen-php-grpc="$VENDOR_BIN/grpc_php_plugin" \
        -I"$PROTO_DIR" \
        $proto_files
    
    echo -e "${GREEN}✓ Stubs generated for $service${NC}"
done

echo -e "${GREEN}✓ All gRPC stubs generated successfully${NC}"
