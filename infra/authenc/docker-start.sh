#!/bin/bash
# ============================================================================
# Authenc Docker Startup Script
# ============================================================================
# Usage: ./docker-start.sh [dev|prod]
# Default: dev (development environment)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_TYPE="${1:-dev}"
ENV_FILE="${SCRIPT_DIR}/.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."

    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed"
        exit 1
    fi

    print_info "Docker and Docker Compose are installed"
}

# Setup environment file
setup_env() {
    if [ ! -f "$ENV_FILE" ]; then
        print_warn ".env file not found"
        print_info "Creating .env from .env.example..."

        if [ ! -f "${SCRIPT_DIR}/.env.example" ]; then
            print_error ".env.example not found"
            exit 1
        fi

        cp "${SCRIPT_DIR}/.env.example" "$ENV_FILE"
        print_warn "Please edit .env file with your settings before continuing"
        print_warn "Run: nano $ENV_FILE"
        exit 1
    fi

    print_info ".env file found"
}

# Start services
start_services() {
    print_info "Starting Authenc services in $ENV_TYPE environment..."

    if [ "$ENV_TYPE" = "prod" ]; then
        print_info "Using production configuration..."
        docker-compose -f "${SCRIPT_DIR}/docker-compose.yml" \
                       -f "${SCRIPT_DIR}/docker-compose.prod.yml" \
                       up -d
    else
        print_info "Using development configuration..."
        docker-compose -f "${SCRIPT_DIR}/docker-compose.yml" up -d
    fi

    print_info "Services starting..."
}

# Wait for services to be healthy
wait_for_services() {
    print_info "Waiting for services to be healthy..."

    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if docker-compose -f "${SCRIPT_DIR}/docker-compose.yml" exec -T postgres pg_isready -U postgres &> /dev/null; then
            print_info "PostgreSQL is ready"
            break
        fi
        attempt=$((attempt + 1))
        echo -n "."
        sleep 1
    done

    if [ $attempt -eq $max_attempts ]; then
        print_error "PostgreSQL failed to start"
        exit 1
    fi

    # Wait for Authenc to be ready
    attempt=0
    while [ $attempt -lt $max_attempts ]; do
        if curl -sf http://localhost:8088/health &> /dev/null; then
            print_info "Authenc is ready"
            break
        fi
        attempt=$((attempt + 1))
        echo -n "."
        sleep 1
    done

    if [ $attempt -eq $max_attempts ]; then
        print_error "Authenc failed to start"
        print_info "Checking logs..."
        docker-compose -f "${SCRIPT_DIR}/docker-compose.yml" logs authenc
        exit 1
    fi

    echo ""
}

# Display service information
display_info() {
    print_info "Services are running!"
    echo ""
    echo "Service URLs:"
    echo "  - Authenc API:  http://localhost:8088"
    echo "  - gRPC:         localhost:9088"
    echo "  - Metrics:      http://localhost:9090/metrics"
    echo "  - Prometheus:   http://localhost:9091"
    echo "  - Grafana:      http://localhost:3000"
    echo ""
    echo "Useful commands:"
    echo "  - View logs:    docker-compose logs -f authenc"
    echo "  - Stop:         docker-compose stop"
    echo "  - Restart:      docker-compose restart"
    echo "  - Status:       docker-compose ps"
    echo ""
}

# Main execution
main() {
    print_info "Authenc Docker Startup Script"
    print_info "Environment: $ENV_TYPE"
    echo ""

    check_prerequisites
    setup_env
    start_services
    wait_for_services
    display_info
}

main "$@"
