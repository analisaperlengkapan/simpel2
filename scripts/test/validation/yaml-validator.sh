#!/bin/bash

# 🔍 YAML Validation & Linting Script
# Validates YAML files using yamllint with custom configuration
# Part of SIMPelv2 Testing & Quality Assurance Suite

set -euo pipefail

# ====== CONFIGURATION ======
WORKSPACE_ROOT="${WORKSPACE_ROOT:-/var/www/simpelv2}"
TEST_DIR="$WORKSPACE_ROOT/scripts/test"
VALIDATION_DIR="$TEST_DIR/validation"
YAMLLINT_CONFIG="$VALIDATION_DIR/yamllint.yaml"

# Directories to validate
YAML_DIRS=(
    "infra/k8s"
    "infra/docker-compose*.yml"
    ".github/workflows"
    "antarmuka/*/Trunk.toml"
    "layanan/*/Cargo.toml"
)

# Color definitions
declare -A COLORS=(
    [RED]='\033[0;31m'
    [GREEN]='\033[0;32m'
    [YELLOW]='\033[1;33m'
    [BLUE]='\033[0;34m'
    [CYAN]='\033[0;36m'
    [BOLD]='\033[1m'
    [NC]='\033[0m'
)

# ====== UTILITY FUNCTIONS ======
log() {
    local level="$1"
    shift
    local color="${COLORS[${level^^}]:-${COLORS[NC]}}"
    echo -e "${color}[$level]${COLORS[NC]} $*"
}

success() { log "green" "✅ $*"; }
error() { log "red" "❌ $*"; }
warn() { log "yellow" "⚠️  $*"; }
info() { log "blue" "ℹ️  $*"; }

banner() {
    echo -e "${COLORS[CYAN]}"
    echo "╔══════════════════════════════════════════════════════════════════╗"
    echo "║   🔍 YAML Validation & Linting - SIMPelv2 Quality Assurance     ║"
    echo "║   📋 Configuration Validation • 🛠️ Infrastructure Checks        ║"
    echo "╚══════════════════════════════════════════════════════════════════╝"
    echo -e "${COLORS[NC]}"
}

# ====== VALIDATION FUNCTIONS ======
check_yamllint_installed() {
    if ! command -v yamllint &> /dev/null; then
        warn "yamllint not found, installing..."
        if command -v pip3 &> /dev/null; then
            pip3 install yamllint
        elif command -v apt &> /dev/null; then
            sudo apt update && sudo apt install -y yamllint
        else
            error "Cannot install yamllint automatically. Please install manually."
            return 1
        fi
    fi
    success "yamllint is available"
}

validate_yamllint_config() {
    if [[ ! -f "$YAMLLINT_CONFIG" ]]; then
        error "yamllint configuration not found at: $YAMLLINT_CONFIG"
        return 1
    fi
    success "yamllint configuration found"
    
    # Test configuration syntax
    if yamllint -d "$YAMLLINT_CONFIG" --print-config &> /dev/null; then
        success "yamllint configuration is valid"
    else
        error "yamllint configuration has syntax errors"
        return 1
    fi
}

find_yaml_files() {
    local files=()
    
    # Find YAML files in specified directories
    for pattern in "${YAML_DIRS[@]}"; do
        if [[ "$pattern" == *"*"* ]]; then
            # Handle glob patterns
            while IFS= read -r -d '' file; do
                files+=("$file")
            done < <(find "$WORKSPACE_ROOT" -path "*/$pattern" -type f -print0 2>/dev/null || true)
        elif [[ -d "$WORKSPACE_ROOT/$pattern" ]]; then
            # Handle directories
            while IFS= read -r -d '' file; do
                files+=("$file")
            done < <(find "$WORKSPACE_ROOT/$pattern" -name "*.yml" -o -name "*.yaml" -type f -print0 2>/dev/null || true)
        fi
    done
    
    # Add common YAML files
    local common_files=(
        "docker-compose.yml"
        "docker-compose.dev.yml"
        "docker-compose.prod.yml"
        ".gitlab-ci.yml"
        ".github/workflows/*.yml"
        ".github/workflows/*.yaml"
    )
    
    for pattern in "${common_files[@]}"; do
        while IFS= read -r -d '' file; do
            files+=("$file")
        done < <(find "$WORKSPACE_ROOT" -path "*/$pattern" -type f -print0 2>/dev/null || true)
    done
    
    printf '%s\n' "${files[@]}" | sort -u
}

validate_yaml_files() {
    local files=()
    readarray -t files < <(find_yaml_files)
    
    if [[ ${#files[@]} -eq 0 ]]; then
        warn "No YAML files found to validate"
        return 0
    fi
    
    info "Found ${#files[@]} YAML files to validate"
    
    local errors=0
    local warnings=0
    local success_count=0
    
    for file in "${files[@]}"; do
        if [[ -f "$file" ]]; then
            echo -e "\n${COLORS[BLUE]}📄 Validating: ${file#$WORKSPACE_ROOT/}${COLORS[NC]}"
            
            if yamllint -d "$YAMLLINT_CONFIG" "$file"; then
                success "✅ Valid"
                ((success_count++))
            else
                local exit_code=$?
                if [[ $exit_code -eq 1 ]]; then
                    warn "⚠️  Has warnings"
                    ((warnings++))
                else
                    error "❌ Has errors"
                    ((errors++))
                fi
            fi
        fi
    done
    
    echo -e "\n${COLORS[BOLD]}📊 YAML Validation Summary:${COLORS[NC]}"
    echo "================================="
    echo -e "${COLORS[GREEN]}✅ Valid files:${COLORS[NC]}     $success_count"
    echo -e "${COLORS[YELLOW]}⚠️  Files with warnings:${COLORS[NC]} $warnings"
    echo -e "${COLORS[RED]}❌ Files with errors:${COLORS[NC]}   $errors"
    echo "================================="
    echo "Total files processed: ${#files[@]}"
    
    if [[ $errors -gt 0 ]]; then
        error "YAML validation failed with $errors errors"
        return 1
    elif [[ $warnings -gt 0 ]]; then
        warn "YAML validation completed with $warnings warnings"
        return 0
    else
        success "All YAML files are valid!"
        return 0
    fi
}

validate_specific_file() {
    local file="$1"
    
    if [[ ! -f "$file" ]]; then
        error "File not found: $file"
        return 1
    fi
    
    info "Validating single file: $file"
    
    if yamllint -d "$YAMLLINT_CONFIG" "$file"; then
        success "File is valid: $file"
        return 0
    else
        local exit_code=$?
        if [[ $exit_code -eq 1 ]]; then
            warn "File has warnings: $file"
            return 1
        else
            error "File has errors: $file"
            return 1
        fi
    fi
}

show_help() {
    banner
    echo ""
    echo -e "${COLORS[BOLD]}Usage:${COLORS[NC]} $0 [command] [options]"
    echo ""
    echo -e "${COLORS[GREEN]}COMMANDS:${COLORS[NC]}"
    echo "  validate-all            - Validate all YAML files in the project"
    echo "  validate-file <file>    - Validate specific YAML file"
    echo "  list-files              - List all YAML files that will be validated"
    echo "  check-config            - Validate yamllint configuration"
    echo "  install                 - Install yamllint if not available"
    echo ""
    echo -e "${COLORS[BLUE]}EXAMPLES:${COLORS[NC]}"
    echo "  $0 validate-all                           # Validate all YAML files"
    echo "  $0 validate-file docker-compose.yml      # Validate specific file"
    echo "  $0 list-files                            # List files to be validated"
    echo ""
    echo -e "${COLORS[YELLOW]}OPTIONS:${COLORS[NC]}"
    echo "  --config <file>         - Use custom yamllint config file"
    echo "  --help, -h              - Show this help message"
}

# ====== MAIN FUNCTION ======
main() {
    local command="${1:-validate-all}"
    
    case "$command" in
        "validate-all"|"all")
            banner
            check_yamllint_installed
            validate_yamllint_config
            validate_yaml_files
            ;;
        "validate-file"|"file")
            if [[ -z "${2:-}" ]]; then
                error "Please specify a file to validate"
                exit 1
            fi
            banner
            check_yamllint_installed
            validate_yamllint_config
            validate_specific_file "$2"
            ;;
        "list-files"|"list")
            info "YAML files that will be validated:"
            find_yaml_files | sed 's|^.*/||' | sort
            ;;
        "check-config"|"config")
            banner
            check_yamllint_installed
            validate_yamllint_config
            ;;
        "install")
            check_yamllint_installed
            ;;
        "help"|"--help"|"-h")
            show_help
            ;;
        *)
            error "Unknown command: $command"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Execute main function if script is run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
