#!/bin/bash
# SIMPelv2 - Build and Push All Container Images to Registry
# This script builds all Docker images and pushes them to registry.kejaksaan.go.id
# Supports: Docker, Podman, and MicroK8s registry

set -euo pipefail

# Configuration
REGISTRY="${REGISTRY:-localhost:32000}"  # Default to MicroK8s registry
EXTERNAL_REGISTRY="registry.kejaksaan.go.id/simpelv2"
TAG="${TAG:-latest}"
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
USE_SUDO="${USE_SUDO:-false}"

# Detect container runtime
if command -v podman &> /dev/null; then
    CONTAINER_CMD="podman"
    USE_SUDO=false
elif command -v docker &> /dev/null; then
    CONTAINER_CMD="docker"
    # Check if we need sudo for docker
    if ! docker ps &> /dev/null; then
        if sudo docker ps &> /dev/null 2>&1; then
            USE_SUDO=true
        fi
    fi
else
    echo "Error: Neither docker nor podman found"
    exit 1
fi

# Wrapper function for container commands
container_cmd() {
    if [[ "$USE_SUDO" == "true" ]]; then
        sudo $CONTAINER_CMD "$@"
    else
        $CONTAINER_CMD "$@"
    fi
}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Counters
TOTAL=0
SUCCESS=0
FAILED=0
SKIPPED=0

# Arrays to track results
declare -a FAILED_IMAGES=()
declare -a SUCCESS_IMAGES=()

# Function to print colored output
print_header() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to build and push a Docker image
build_and_push() {
    local service_path=$1
    local image_name=$2
    local dockerfile_path="${service_path}/Dockerfile"
    local build_context=$3  # Optional: specify build context (default: service_path)
    
    TOTAL=$((TOTAL + 1))
    
    if [[ ! -f "$dockerfile_path" ]]; then
        print_warning "Dockerfile not found: $dockerfile_path - SKIPPED"
        SKIPPED=$((SKIPPED + 1))
        return 0
    fi
    
    # Determine build context
    if [[ -z "$build_context" ]]; then
        # For independent workspaces (authenc, secreton, gerbang), use service_path
        # For workspace members, use workspace root
        if [[ "$service_path" =~ ^(infra/authenc|infra/secreton|infra/gerbang)$ ]]; then
            build_context="$service_path"
        else
            build_context="$WORKSPACE_ROOT"
        fi
    fi
    
    print_info "Building: ${REGISTRY}/${image_name}:${TAG}"
    print_info "Context: $build_context | Dockerfile: $dockerfile_path"
    
    # Build the image with correct context
    if container_cmd build -t "${REGISTRY}/${image_name}:${TAG}" -f "$dockerfile_path" "$build_context" 2>&1 | tee "/tmp/build-${image_name}.log"; then
        print_success "Built: ${image_name}"
        
        # Push the image
        print_info "Pushing: ${REGISTRY}/${image_name}:${TAG}"
        if container_cmd push "${REGISTRY}/${image_name}:${TAG}" 2>&1 | tee -a "/tmp/build-${image_name}.log"; then
            print_success "Pushed: ${image_name}"
            SUCCESS=$((SUCCESS + 1))
            SUCCESS_IMAGES+=("${image_name}")
            
            # Also tag and push to external registry if different
            if [[ "$REGISTRY" != "$EXTERNAL_REGISTRY" ]] && [[ -n "$EXTERNAL_REGISTRY" ]]; then
                container_cmd tag "${REGISTRY}/${image_name}:${TAG}" "${EXTERNAL_REGISTRY}/${image_name}:${TAG}"
                print_info "Also tagged for: ${EXTERNAL_REGISTRY}/${image_name}:${TAG}"
            fi
        else
            print_error "Failed to push: ${image_name}"
            FAILED=$((FAILED + 1))
            FAILED_IMAGES+=("${image_name}")
        fi
    else
        print_error "Failed to build: ${image_name}"
        FAILED=$((FAILED + 1))
        FAILED_IMAGES+=("${image_name}")
    fi
    
    echo ""
}

# Check if container runtime is working
print_info "Using container runtime: ${CONTAINER_CMD} (sudo: ${USE_SUDO})"
if ! container_cmd info >/dev/null 2>&1; then
    print_error "${CONTAINER_CMD} is not running or accessible. Please check your setup."
    exit 1
fi

# Check registry connectivity (skip for localhost)
if [[ ! "$REGISTRY" =~ ^localhost ]]; then
    print_info "Checking registry connectivity..."
    if ! container_cmd login "${REGISTRY%%/*}" 2>/dev/null; then
        print_warning "Not logged in to registry. Attempting to continue..."
    fi
fi

cd "$WORKSPACE_ROOT"

print_header "Building and Pushing SIMPelv2 Container Images"
echo -e "Registry: ${CYAN}${REGISTRY}${NC}"
echo -e "Tag: ${CYAN}${TAG}${NC}"
echo -e "Workspace: ${CYAN}${WORKSPACE_ROOT}${NC}"
echo ""

# ====== INFRASTRUCTURE SERVICES ======
print_header "Infrastructure Services"

build_and_push "infra/authenc" "authenc"
build_and_push "infra/secreton" "secreton"
build_and_push "infra/gerbang" "gerbang"

# ====== BACKEND SERVICES ======
print_header "Backend Services"

# Check if layanan directory structure exists
if [[ -d "layanan/shared" ]]; then
    # New structure with shared services
    for service in ai bantuan dasbor dokumen integrasi konfigurasi laporan notifikasi; do
        service_path="layanan/shared/${service}"
        if [[ -d "$service_path" ]]; then
            build_and_push "$service_path" "layanan-${service}"
        else
            print_warning "Service not found: $service_path - SKIPPED"
            SKIPPED=$((SKIPPED + 1))
        fi
    done
else
    # Old structure with individual services
    for service in ai bantuan dasbor dokumen integrasi konfigurasi laporan notifikasi; do
        service_path="layanan/${service}"
        if [[ -d "$service_path" ]]; then
            build_and_push "$service_path" "layanan-${service}"
        else
            print_warning "Service not found: $service_path - SKIPPED"
            SKIPPED=$((SKIPPED + 1))
        fi
    done
fi

# ====== FRONTEND SERVICES ======
print_header "Frontend Microfrontends"

# Portal (main entry point)
build_and_push "antarmuka/portal" "portal"

# Domain-specific microfrontends
build_and_push "antarmuka/badiklat" "badiklat"
build_and_push "antarmuka/datun" "datun"
build_and_push "antarmuka/intel" "intel"
build_and_push "antarmuka/pidmil" "pidmil"
build_and_push "antarmuka/pidsus" "pidsus"
build_and_push "antarmuka/pidum" "pidum"
build_and_push "antarmuka/pemulihan_aset" "pemulihan-aset"
build_and_push "antarmuka/pengawasan" "pengawasan"

# Pembinaan modules
if [[ -d "antarmuka/pembinaan" ]]; then
    build_and_push "antarmuka/pembinaan/keuangan" "keuangan"
    build_and_push "antarmuka/pembinaan/perencanaan" "perencanaan"
    build_and_push "antarmuka/pembinaan/perlengkapan" "perlengkapan"
else
    # Alternative structure
    build_and_push "antarmuka/keuangan" "keuangan"
    build_and_push "antarmuka/perencanaan" "perencanaan"
    build_and_push "antarmuka/perlengkapan" "perlengkapan"
fi

# ====== SUMMARY ======
print_header "Build & Push Summary"

echo -e "${CYAN}Total Images:${NC} $TOTAL"
echo -e "${GREEN}Successful:${NC} $SUCCESS"
echo -e "${RED}Failed:${NC} $FAILED"
echo -e "${YELLOW}Skipped:${NC} $SKIPPED"
echo ""

if [[ ${#SUCCESS_IMAGES[@]} -gt 0 ]]; then
    echo -e "${GREEN}✅ Successfully built and pushed:${NC}"
    for img in "${SUCCESS_IMAGES[@]}"; do
        echo -e "  - ${GREEN}${img}${NC}"
    done
    echo ""
fi

if [[ ${#FAILED_IMAGES[@]} -gt 0 ]]; then
    echo -e "${RED}❌ Failed to build/push:${NC}"
    for img in "${FAILED_IMAGES[@]}"; do
        echo -e "  - ${RED}${img}${NC}"
        echo -e "    Log: /tmp/build-${img}.log"
    done
    echo ""
fi

# Exit with appropriate code
if [[ $FAILED -gt 0 ]]; then
    print_error "Some images failed to build or push. Check logs in /tmp/build-*.log"
    exit 1
else
    print_success "All images built and pushed successfully!"
    exit 0
fi
