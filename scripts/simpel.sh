#!/bin/bash

# 🚀 SIMPelv2 - Unified Script Entry Point
# Single access point for all development, testing, and deployment operations
# Version: 4.0.0 - Simplified & Optimized

set -euo pipefail

# ====== GLOBAL CONFIGURATION ======
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SIMPEL_VERSION="4.0.0"

# Colors for output
declare -A COLORS=(
    [RED]='\033[0;31m'
    [GREEN]='\033[0;32m'
    [YELLOW]='\033[1;33m'
    [BLUE]='\033[0;34m'
    [CYAN]='\033[0;36m'
    [PURPLE]='\033[0;35m'
    [BOLD]='\033[1m'
    [NC]='\033[0m'
)

# ====== UTILITY FUNCTIONS ======
log() {
    local level="$1"; shift
    local color="${COLORS[${level^^}]:-${COLORS[NC]}}"
    echo -e "${color}[${level^^}]${COLORS[NC]} $*"
}

info() { log "info" "ℹ️  $*"; }
success() { log "success" "✅ $*"; }
warn() { log "warn" "⚠️  $*"; }
error() { log "error" "❌ $*"; }

# ====== MAIN MENU ======
show_menu() {
    echo -e "${COLORS[PURPLE]}${COLORS[BOLD]}"
    echo "  ╔══════════════════════════════════════════╗"
    echo "  ║            🚀 SIMPelv2 v$SIMPEL_VERSION            ║"
    echo "  ║      Unified Development Environment     ║"
    echo "  ╚══════════════════════════════════════════╝"
    echo -e "${COLORS[NC]}"
    echo ""
    echo -e "${COLORS[CYAN]}🏗️  BUILD & DEVELOPMENT:${COLORS[NC]}"
    echo "  1) 🔨 Build All (Rust + Frontend)"
    echo "  2) 🎨 Build Frontend Only"
    echo "  3) ⚙️  Build Backend Only"
    echo "  4) 🚀 Start Development Server"
    echo "  5) 🧹 Clean Build Artifacts"
    echo ""
    echo -e "${COLORS[BLUE]}🧪 TESTING & VALIDATION:${COLORS[NC]}"
    echo "  6) 🧪 Run All Tests"
    echo "  7) 📊 Performance Benchmark"
    echo "  8) 🛡️  Security Scan"
    echo "  9) ✅ Environment Validation"
    echo ""
    echo -e "${COLORS[GREEN]}🚀 DEPLOYMENT & OPS:${COLORS[NC]}"
    echo "  10) 🐳 Deploy Development"
    echo "  11) 📈 System Monitoring"
    echo "  12) 💾 Backup & Maintenance"
    echo ""
    echo -e "${COLORS[YELLOW]}🛠️  TOOLS & UTILITIES:${COLORS[NC]}"
    echo "  13) 📦 WASM Optimization"
    echo "  14) 🔧 Cargo Maintenance"
    echo "  15) 🚀 Create New Service/Frontend"
    echo "  16) 📊 Project Statistics"
    echo "  17) 🤖 AI Development Tools"
    echo "  18) ⚙️  VS Code Configuration"
    echo ""
    echo -e "${COLORS[PURPLE]}📚 HELP & INFO:${COLORS[NC]}"
    echo "  h) 📖 Show Help & Documentation"
    echo "  v) 📋 Show Version & Status"
    echo "  q) 🚪 Quit"
    echo ""
    echo -ne "${COLORS[BOLD]}Select option (1-18, h, v, q): ${COLORS[NC]}"
}

# ====== OPERATION HANDLERS ======
handle_build() {
    local target="${1:-all}"
    info "Executing build operation: $target"
    cd "$WORKSPACE_ROOT"
    case "$target" in
        "all") make build-parallel ;;
        "frontend") make leptos-frontend ;;
        "backend") make rust-backend ;;
        "clean") make clean ;;
        *) error "Unknown build target: $target" ;;
    esac
}

handle_test() {
    local type="${1:-all}"
    info "Executing test operation: $type"
    "$SCRIPT_DIR/test/test-runner.sh" "$type"
}

handle_deploy() {
    local env="${1:-dev}"
    info "Executing deployment: $env"
    "$SCRIPT_DIR/core/deploy.sh" "$env"
}

handle_monitoring() {
    info "Starting system monitoring"
    "$SCRIPT_DIR/ops/monitoring.sh" "overview"
}

handle_tools() {
    local tool="$1"; shift
    case "$tool" in
        "wasm") "$SCRIPT_DIR/tools/wasm-optimizer.sh" "$@" ;;
        "cargo") "$SCRIPT_DIR/tools/cargo-maintenance.sh" "$@" ;;
        "ai") "$SCRIPT_DIR/tools/ai/ai-tools.sh" "$@" ;;
        "vscode") "$SCRIPT_DIR/vscode-validator.sh" "$@" ;;
        *) error "Unknown tool: $tool" ;;
    esac
}

# ====== HELP & INFO ======
show_help() {
    echo -e "${COLORS[CYAN]}📚 SIMPelv2 Help & Documentation${COLORS[NC]}"
    echo "======================================="
    echo ""
    echo -e "${COLORS[YELLOW]}📋 DIRECT USAGE:${COLORS[NC]}"
    echo "  ./scripts/simpel.sh                 # Interactive menu"
    echo "  ./scripts/simpel.sh build all       # Build everything"
    echo "  ./scripts/simpel.sh test performance # Run performance tests"
    echo "  ./scripts/simpel.sh deploy dev       # Deploy to development"
    echo ""
    echo -e "${COLORS[GREEN]}📁 SCRIPT STRUCTURE:${COLORS[NC]}"
    echo "  scripts/makefiles/  # Unified build and operations system"
    echo "  scripts/tools/      # Development tools & utilities"
    echo "  scripts/ops/        # Operations & monitoring"
    echo "  scripts/test/       # Testing & validation"
    echo ""
    echo -e "${COLORS[BLUE]}📖 DOCUMENTATION:${COLORS[NC]}"
    echo "  scripts/README.md   # Complete usage guide"
    echo "  docs/               # Detailed documentation"
}

show_status() {
    echo -e "${COLORS[PURPLE]}📋 SIMPelv2 Status Information${COLORS[NC]}"
    echo "=================================="
    echo ""
    echo "Version: $SIMPEL_VERSION"
    echo "Workspace: $WORKSPACE_ROOT"
    echo "Scripts Dir: $SCRIPT_DIR"
    echo ""
    echo -e "${COLORS[GREEN]}🔍 System Check:${COLORS[NC]}"
    command -v cargo >/dev/null 2>&1 && echo "  ✅ Cargo available" || echo "  ❌ Cargo not found"
    command -v docker >/dev/null 2>&1 && echo "  ✅ Docker available" || echo "  ❌ Docker not found"
    command -v trunk >/dev/null 2>&1 && echo "  ✅ Trunk available" || echo "  ❌ Trunk not found"
}

# ====== INTERACTIVE MODE ======
interactive_mode() {
    while true; do
        show_menu
        read -r choice
        echo ""

        case "$choice" in
            # Build operations
            1) handle_build "all" ;;
            2) handle_build "frontend" ;;
            3) handle_build "backend" ;;
            4) "$SCRIPT_DIR/core/dev.sh" "start" ;;
            5) handle_build "clean" ;;

            # Testing operations
            6) handle_test "all" ;;
            7) handle_test "performance" ;;
            8) handle_test "security" ;;
            9) handle_test "validation" ;;

            # Deployment operations
            10) handle_deploy "dev" ;;
            11) handle_monitoring ;;
            12) "$SCRIPT_DIR/ops/backup.sh" "maintenance" ;;

            # Tools
            13) handle_tools "wasm" "stats" ;;
            14) handle_tools "cargo" "status" ;;
            15) handle_project_init ;;
            16) handle_project_stats ;;
            17) handle_tools "ai" "help" ;;
            18) handle_tools "vscode" "validate" ;;

            # Help & info
            h|H) show_help; echo ""; echo "Press Enter to continue..."; read -r ;;
            v|V) show_status; echo ""; echo "Press Enter to continue..."; read -r ;;
            q|Q) success "Goodbye! 👋"; exit 0 ;;

            *) warn "Invalid choice. Please select 1-18, h, v, or q" ;;
        esac

        echo ""
        echo "Press Enter to continue..."
        read -r
        clear
    done
}

# ====== MAIN EXECUTION ======
main() {
    cd "$WORKSPACE_ROOT" 2>/dev/null || {
        error "Cannot access workspace: $WORKSPACE_ROOT"
        exit 1
    }

    # Command line mode
    if [[ $# -gt 0 ]]; then
        case "$1" in
            # Build commands
            "build") handle_build "${2:-all}" ;;
            "dev") "$SCRIPT_DIR/core/dev.sh" "${2:-start}" ;;
            "clean") handle_build "clean" ;;

            # Test commands
            "test") handle_test "${2:-all}" ;;
            "benchmark") handle_test "performance" ;;

            # Deployment commands
            "deploy") handle_deploy "${2:-dev}" ;;
            "monitor") handle_monitoring ;;

            # Tool commands
            "wasm") handle_tools "wasm" "${@:2}" ;;
            "cargo") handle_tools "cargo" "${@:2}" ;;
            "ai") handle_tools "ai" "${@:2}" ;;
            "vscode") handle_tools "vscode" "${@:2}" ;;

            # Help & info
            "help"|"--help"|"-h") show_help ;;
            "version"|"--version"|"-v") show_status ;;

            # Invalid
            *) error "Unknown command: $1"; show_help; exit 1 ;;
        esac
    else
        # Interactive mode
        clear
        interactive_mode
    fi
}

# Execute main function
main "$@"
