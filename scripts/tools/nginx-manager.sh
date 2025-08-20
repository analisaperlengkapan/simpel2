#!/bin/bash
# Enhanced Nginx Configuration Management Script
# Integrates with existing SIMPelv2 infrastructure

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

show_help() {
    cat << EOF
Enhanced Nginx Configuration Management Script

USAGE:
    $0 [COMMAND] [OPTIONS]

COMMANDS:
    generate-all     Generate nginx configs for all microfrontends
    generate         Generate config for specific service
    validate         Validate generated nginx configurations
    update-infra     Update infrastructure nginx configuration
    deploy           Deploy nginx configurations to containers
    clean            Clean generated nginx configurations
    status           Show nginx configuration status
    help             Show this help message

OPTIONS:
    --service NAME   Specify service name (for generate command)
    --validate       Validate configurations after generation
    --force          Force regeneration/overwrite existing files
    --dry-run        Show what would be done without making changes

EXAMPLES:
    $0 generate-all --validate
    $0 generate --service portal
    $0 update-infra
    $0 validate
    $0 deploy --dry-run

EOF
}

check_dependencies() {
    local missing_deps=()
    
    if ! command -v python3 &> /dev/null; then
        missing_deps+=("python3")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        error "Missing dependencies: ${missing_deps[*]}"
        info "Please install the missing dependencies and try again"
        exit 1
    fi
}

generate_all_configs() {
    local validate_flag=""
    [[ "$1" == "--validate" ]] && validate_flag="--validate"
    
    info "Generating nginx configurations for all microfrontends..."
    
    python3 "$SCRIPT_DIR/nginx-config-generator.py" \
        --base-dir "$BASE_DIR" \
        --generate-all \
        --report \
        $validate_flag
        
    success "All nginx configurations generated successfully"
}

generate_service_config() {
    local service_name="$1"
    local validate_flag=""
    [[ "$2" == "--validate" ]] && validate_flag="--validate"
    
    if [[ -z "$service_name" ]]; then
        error "Service name is required for generate command"
        info "Usage: $0 generate --service <service_name>"
        exit 1
    fi
    
    info "Generating nginx configuration for service: $service_name"
    
    python3 "$SCRIPT_DIR/nginx-config-generator.py" \
        --base-dir "$BASE_DIR" \
        --service "$service_name" \
        $validate_flag
        
    success "Nginx configuration generated for $service_name"
}

validate_configs() {
    info "Validating generated nginx configurations..."
    
    local generated_dir="$BASE_DIR/antarmuka/shared/nginx/generated"
    local validation_failed=0
    
    if [[ ! -d "$generated_dir" ]]; then
        warning "No generated configurations found. Run generate-all first."
        return 0
    fi
    
    # Enhanced nginx syntax validation - check structure and syntax
    for config_file in "$generated_dir"/*.conf; do
        if [[ -f "$config_file" ]]; then
            local basename=$(basename "$config_file")
            local syntax_ok=true
            
            # Check for balanced braces
            local open_braces=$(grep -o '{' "$config_file" | wc -l)
            local close_braces=$(grep -o '}' "$config_file" | wc -l)
            
            if [[ $open_braces -ne $close_braces ]]; then
                error "Unbalanced braces in $basename (open: $open_braces, close: $close_braces)"
                syntax_ok=false
            fi
            
            # Check for required directives
            if ! grep -q "server {" "$config_file"; then
                error "Missing server block in $basename"
                syntax_ok=false
            fi
            
            if ! grep -q "listen" "$config_file"; then
                error "Missing listen directive in $basename"
                syntax_ok=false
            fi
            
            # Check for required locations
            if ! grep -q "location /" "$config_file"; then
                error "Missing root location block in $basename"
                syntax_ok=false
            fi
            
            # Check for WASM optimization (specific to our microfrontends)
            if ! grep -q "location.*\.wasm" "$config_file"; then
                warning "Missing WASM location block in $basename"
            fi
            
            # Check for Docker-specific configurations
            if grep -q "proxy_pass.*gerbang" "$config_file"; then
                info "Docker container reference detected in $basename (normal for containerized deployment)"
            fi
            
            # Check for health endpoint
            if ! grep -q "location /health" "$config_file"; then
                warning "Missing health check endpoint in $basename"
            fi
            
            if [[ $syntax_ok == true ]]; then
                success "Syntax validation passed: $basename"
            else
                error "Syntax validation failed: $basename"
                validation_failed=1
            fi
        fi
    done
    
    if [[ $validation_failed -eq 0 ]]; then
        success "All nginx configurations passed syntax validation"
        info "Note: Full nginx -t validation requires running Docker environment for hostname resolution"
    else
        error "Some nginx configurations have syntax errors"
        exit 1
    fi
}

update_infrastructure() {
    info "Updating infrastructure nginx configuration..."
    
    python3 "$SCRIPT_DIR/nginx-config-generator.py" \
        --base-dir "$BASE_DIR" \
        --update-infra
        
    success "Infrastructure nginx configuration updated"
}

deploy_configurations() {
    local dry_run=""
    [[ "$1" == "--dry-run" ]] && dry_run="true"
    
    info "Deploying nginx configurations..."
    
    if [[ "$dry_run" == "true" ]]; then
        info "DRY RUN MODE - showing what would be deployed:"
        echo
        echo "Generated configurations that would be deployed:"
        find "$BASE_DIR/antarmuka/shared/nginx/generated" -name "*.conf" -exec basename {} \; 2>/dev/null | sort
        return 0
    fi
    
    # Check if Docker Compose is running
    if docker compose -f "$BASE_DIR/docker-compose.yml" ps | grep -q nginx; then
        info "Reloading nginx configuration in running containers..."
        docker compose -f "$BASE_DIR/docker-compose.yml" exec nginx nginx -s reload || {
            warning "Failed to reload nginx, attempting restart..."
            docker compose -f "$BASE_DIR/docker-compose.yml" restart nginx
        }
    else
        info "No running nginx containers found. Configuration will be applied on next startup."
    fi
    
    success "Nginx configurations deployed"
}

clean_generated_configs() {
    info "Cleaning generated nginx configurations..."
    
    local generated_dir="$BASE_DIR/antarmuka/shared/nginx/generated"
    if [[ -d "$generated_dir" ]]; then
        rm -f "$generated_dir"/*.conf
        success "Generated nginx configurations cleaned"
    else
        info "No generated configurations to clean"
    fi
}

show_status() {
    info "Nginx Configuration Status"
    echo "=========================="
    
    # Show discovered microfrontends
    local services=($(python3 "$SCRIPT_DIR/nginx-config-generator.py" --base-dir "$BASE_DIR" 2>&1 | grep "Discovered.*microfrontends:" | sed 's/.*: //' | tr ',' '\n' | xargs))
    echo "Discovered microfrontends: ${#services[@]}"
    printf '  - %s\n' "${services[@]}"
    echo
    
    # Show generated configurations
    local generated_dir="$BASE_DIR/antarmuka/shared/nginx/generated"
    if [[ -d "$generated_dir" ]]; then
        local generated_count=$(find "$generated_dir" -name "*.conf" | wc -l)
        echo "Generated configurations: $generated_count"
        find "$generated_dir" -name "*.conf" -exec basename {} \; | sed 's/^/  - /'
    else
        echo "Generated configurations: 0"
    fi
    echo
    
    # Show template status
    local template_file="$BASE_DIR/antarmuka/shared/nginx/microfrontend.conf"
    if [[ -f "$template_file" ]]; then
        echo "Template file: ✅ Present"
        echo "  Location: antarmuka/shared/nginx/microfrontend.conf"
        echo "  Size: $(wc -l < "$template_file") lines"
    else
        echo "Template file: ❌ Missing"
    fi
    echo
    
    # Show infrastructure nginx status
    local infra_nginx="$BASE_DIR/infra/nginx/nginx.conf"
    if [[ -f "$infra_nginx" ]]; then
        echo "Infrastructure nginx: ✅ Present"
        echo "  Location: infra/nginx/nginx.conf"
        echo "  Size: $(wc -l < "$infra_nginx") lines"
    else
        echo "Infrastructure nginx: ❌ Missing"
    fi
}

main() {
    check_dependencies
    
    local command="$1"
    shift || true
    
    case "$command" in
        "generate-all")
            local validate=""
            while [[ $# -gt 0 ]]; do
                case $1 in
                    --validate) validate="--validate"; shift ;;
                    *) shift ;;
                esac
            done
            generate_all_configs "$validate"
            ;;
        "generate")
            local service_name=""
            local validate=""
            while [[ $# -gt 0 ]]; do
                case $1 in
                    --service) service_name="$2"; shift 2 ;;
                    --validate) validate="--validate"; shift ;;
                    *) shift ;;
                esac
            done
            generate_service_config "$service_name" "$validate"
            ;;
        "validate")
            validate_configs
            ;;
        "update-infra")
            update_infrastructure
            ;;
        "deploy")
            local dry_run=""
            while [[ $# -gt 0 ]]; do
                case $1 in
                    --dry-run) dry_run="--dry-run"; shift ;;
                    *) shift ;;
                esac
            done
            deploy_configurations "$dry_run"
            ;;
        "clean")
            clean_generated_configs
            ;;
        "status")
            show_status
            ;;
        "help"|"--help"|"-h"|"")
            show_help
            ;;
        *)
            error "Unknown command: $command"
            info "Run '$0 help' for available commands"
            exit 1
            ;;
    esac
}

main "$@"
