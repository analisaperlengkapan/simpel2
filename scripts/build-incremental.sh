#!/bin/bash
# SIMPelv2 - Incremental Build Script
# Build images in priority order with proper error handling

set -euo pipefail

REGISTRY="${REGISTRY:-localhost:32000}"
TAG="${TAG:-latest}"
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Detect container runtime
if command -v podman &> /dev/null; then
    CMD="podman"
elif command -v docker &> /dev/null; then
    CMD="docker"
    if ! docker ps &> /dev/null 2>&1; then
        CMD="sudo docker"
    fi
else
    echo "Error: Neither docker nor podman found"
    exit 1
fi

echo -e "${BLUE}Using: $CMD${NC}"
echo -e "${BLUE}Registry: $REGISTRY${NC}"
echo -e "${BLUE}Workspace: $WORKSPACE_ROOT${NC}"
echo ""

cd "$WORKSPACE_ROOT"

# Function to build and push
build_service() {
    local name=$1
    local dockerfile=$2
    local context=$3
    
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Building: ${name}${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    if $CMD build -t "${REGISTRY}/${name}:${TAG}" -f "$dockerfile" "$context"; then
        echo -e "${GREEN}✅ Built: ${name}${NC}"
        
        if $CMD push "${REGISTRY}/${name}:${TAG}"; then
            echo -e "${GREEN}✅ Pushed: ${name}${NC}"
            return 0
        else
            echo -e "${RED}❌ Failed to push: ${name}${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ Failed to build: ${name}${NC}"
        return 1
    fi
}

# TIER 1: Critical Infrastructure
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}   TIER 1: Critical Infrastructure${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"

build_service "gerbang" "infra/gerbang/Dockerfile" "infra/gerbang" || true
build_service "authenc" "infra/authenc/Dockerfile" "infra/authenc" || true
build_service "secreton" "infra/secreton/Dockerfile" "infra/secreton" || true

# TIER 2: Backend Services
echo -e "\n${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}   TIER 2: Backend Services${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"

for service in ai bantuan dasbor dokumen integrasi konfigurasi laporan notifikasi; do
    if [[ -f "layanan/shared/${service}/Dockerfile" ]]; then
        build_service "layanan-${service}" "layanan/shared/${service}/Dockerfile" "." || true
    fi
done

# TIER 3: Frontend Microfrontends
echo -e "\n${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}   TIER 3: Frontend Microfrontends${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"

# Portal first (main entry point)
build_service "portal" "antarmuka/portal/Dockerfile" "." || true

# Other frontends
for frontend in badiklat datun intel pidmil pidsus pidum pemulihan_aset pengawasan; do
    if [[ -f "antarmuka/${frontend}/Dockerfile" ]]; then
        build_service "${frontend}" "antarmuka/${frontend}/Dockerfile" "." || true
    fi
done

# Pembinaan frontends
for frontend in keuangan perencanaan perlengkapan; do
    if [[ -f "antarmuka/pembinaan/${frontend}/Dockerfile" ]]; then
        build_service "${frontend}" "antarmuka/pembinaan/${frontend}/Dockerfile" "." || true
    fi
done

echo -e "\n${GREEN}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}   Build Process Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo ""
echo "Check images with: $CMD images | grep ${REGISTRY}"
echo "Check registry with: curl http://localhost:32000/v2/_catalog"
