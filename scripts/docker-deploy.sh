#!/bin/bash
# SIMPelv2 Docker Deployment Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if .env exists
check_env() {
    if [ ! -f .env ]; then
        print_warning ".env file not found. Creating from .env.example..."
        cp .env.example .env
        print_success ".env file created. Please review and update as needed."
    fi
}

# Build services
build_services() {
    print_header "Building Docker Images"

    case "$1" in
        portal)
            print_warning "Building Portal stack (Frontend + Backend + Authenc + Secreton)..."
            docker compose -f docker-compose.portal.yml build
            ;;
        full)
            print_warning "Building full stack (All services)..."
            docker compose -f docker-compose.full.yml build
            ;;
        authenc)
            print_warning "Building Authenc..."
            docker compose -f infra/authenc/docker-compose.yml build
            ;;
        secreton)
            print_warning "Building Secreton..."
            docker compose -f infra/secreton/docker-compose.yml build
            ;;
        *)
            print_error "Unknown target: $1"
            echo "Usage: $0 build [portal|full|authenc|secreton]"
            exit 1
            ;;
    esac

    print_success "Build completed!"
}

# Start services
start_services() {
    print_header "Starting Services"

    case "$1" in
        portal)
            print_warning "Starting Portal stack..."
            docker compose -f docker-compose.portal.yml up -d
            ;;
        full)
            print_warning "Starting full stack..."
            docker compose -f docker-compose.full.yml up -d
            ;;
        authenc)
            print_warning "Starting Authenc..."
            docker compose -f infra/authenc/docker-compose.yml up -d
            ;;
        secreton)
            print_warning "Starting Secreton..."
            docker compose -f infra/secreton/docker-compose.yml up -d
            ;;
        *)
            print_error "Unknown target: $1"
            echo "Usage: $0 start [portal|full|authenc|secreton]"
            exit 1
            ;;
    esac

    print_success "Services started!"
}

# Stop services
stop_services() {
    print_header "Stopping Services"

    case "$1" in
        portal)
            docker compose -f docker-compose.portal.yml down
            ;;
        full)
            docker compose -f docker-compose.full.yml down
            ;;
        authenc)
            docker compose -f infra/authenc/docker-compose.yml down
            ;;
        secreton)
            docker compose -f infra/secreton/docker-compose.yml down
            ;;
        all)
            print_warning "Stopping all services..."
            docker compose -f docker-compose.portal.yml down 2>/dev/null || true
            docker compose -f docker-compose.full.yml down 2>/dev/null || true
            docker compose -f infra/authenc/docker-compose.yml down 2>/dev/null || true
            docker compose -f infra/secreton/docker-compose.yml down 2>/dev/null || true
            ;;
        *)
            print_error "Unknown target: $1"
            echo "Usage: $0 stop [portal|full|authenc|secreton|all]"
            exit 1
            ;;
    esac

    print_success "Services stopped!"
}

# Show logs
show_logs() {
    print_header "Showing Logs"

    case "$1" in
        portal)
            docker compose -f docker-compose.portal.yml logs -f
            ;;
        full)
            docker compose -f docker-compose.full.yml logs -f
            ;;
        authenc)
            docker logs -f authenc
            ;;
        secreton)
            docker logs -f secreton
            ;;
        *)
            docker logs -f "$1"
            ;;
    esac
}

# Check service health
check_health() {
    print_header "Checking Service Health"

    services=(
        "authenc:8088"
        "secreton:8200"
        "layanan-portal:8081"
    )

    for service in "${services[@]}"; do
        IFS=':' read -r name port <<< "$service"

        if curl -f -s "http://localhost:$port/health" > /dev/null 2>&1; then
            print_success "$name is healthy on port $port"
        else
            print_error "$name is not responding on port $port"
        fi
    done
}

# Run integration tests
run_tests() {
    print_header "Running Integration Tests"

    print_warning "Testing Authenc..."
    curl -X POST http://localhost:8088/api/v1/auth/register \
        -H "Content-Type: application/json" \
        -d '{"email":"test@example.com","password":"TestPassword123!"}' || true

    print_warning "Testing Secreton..."
    curl -X GET http://localhost:8200/health || true

    print_warning "Testing Portal Backend..."
    curl -X GET http://localhost:8081/health || true

    print_success "Integration tests completed!"
}

# Main script
main() {
    check_env

    case "$1" in
        build)
            build_services "$2"
            ;;
        start|up)
            start_services "$2"
            ;;
        stop|down)
            stop_services "$2"
            ;;
        restart)
            stop_services "$2"
            start_services "$2"
            ;;
        logs)
            show_logs "$2"
            ;;
        health)
            check_health
            ;;
        test)
            run_tests
            ;;
        *)
            echo "SIMPelv2 Docker Deployment Tool"
            echo ""
            echo "Usage: $0 COMMAND [TARGET]"
            echo ""
            echo "Commands:"
            echo "  build    Build Docker images"
            echo "  start    Start services"
            echo "  stop     Stop services"
            echo "  restart  Restart services"
            echo "  logs     Show service logs"
            echo "  health   Check service health"
            echo "  test     Run integration tests"
            echo ""
            echo "Targets:"
            echo "  portal   Portal stack (Frontend + Backend + Authenc + Secreton)"
            echo "  full     Full stack (All services)"
            echo "  authenc  Authenc only"
            echo "  secreton Secreton only"
            echo "  all      All services (for stop command)"
            echo ""
            echo "Examples:"
            echo "  $0 build portal      # Build portal stack"
            echo "  $0 start portal      # Start portal stack"
            echo "  $0 logs authenc      # Show authenc logs"
            echo "  $0 health            # Check all service health"
            echo "  $0 test              # Run integration tests"
            exit 1
            ;;
    esac
}

main "$@"
