#!/bin/bash

# 📊 SIMPelv2 Project Statistics Tool
# Generate project metrics and architecture overview

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

WORKSPACE_ROOT="/var/www/simpelv2"

# Generate project statistics
project_stats() {
    echo -e "${GREEN}📊 Project Statistics${NC}"
    echo "===================="

    cd "$WORKSPACE_ROOT"

    # Code statistics
    echo -e "${BLUE}📏 Code Metrics:${NC}"
    echo "  🦀 Rust files: $(find . -name "*.rs" | wc -l)"
    echo "  📄 TOML files: $(find . -name "*.toml" | wc -l)"
    echo "  🌐 HTML files: $(find . -name "*.html" | wc -l)"
    echo "  🎨 CSS files: $(find . -name "*.css" | wc -l)"
    echo "  📜 Markdown files: $(find . -name "*.md" | wc -l)"
    echo "  🐚 Shell scripts: $(find . -name "*.sh" | wc -l)"
    echo "  🐍 Python files: $(find . -name "*.py" | wc -l)"

    # Project structure
    echo ""
    echo -e "${BLUE}🏗️  Architecture:${NC}"
    echo "  📦 Microservices: $(find layanan -maxdepth 1 -type d ! -name layanan | wc -l)"
    echo "  🌐 Microfrontends: $(find antarmuka -maxdepth 1 -type d ! -name antarmuka -a ! -name shared | wc -l)"
    echo "  📚 Documentation files: $(find docs -name "*.md" 2>/dev/null | wc -l)"
    echo "  🐳 Docker files: $(find . -name "Dockerfile" -o -name "docker-compose*.yml" | wc -l)"
    echo "  ⚙️  CI/CD configs: $(find . -name ".gitlab-ci.yml" -o -name ".github" | wc -l)"

    # Infrastructure
    echo ""
    echo -e "${BLUE}🔧 Infrastructure:${NC}"
    if [[ -d "infra/k8s" ]]; then
        echo "  ☸️  Kubernetes manifests: $(find infra/k8s -name "*.yml" -o -name "*.yaml" | wc -l)"
    fi
    if [[ -d "infra/nginx" ]]; then
        echo "  🌐 Nginx configs: $(find infra/nginx -name "*.conf" | wc -l)"
    fi
    if [[ -d "infra/monitoring" ]]; then
        echo "  📊 Monitoring configs: $(find infra/monitoring -type f | wc -l)"
    fi

    # Scripts analysis
    echo ""
    echo -e "${BLUE}🔨 Scripts & Tools:${NC}"
    if [[ -d "scripts" ]]; then
        echo "  🛠️  Build scripts: $(find scripts/makefiles -name "*.mk" 2>/dev/null | wc -l) makefiles"
        echo "  🧪 Test scripts: $(find scripts/test -name "*.sh" 2>/dev/null | wc -l)"
        echo "  ⚙️  Tool scripts: $(find scripts/tools -name "*.sh" 2>/dev/null | wc -l)"
        echo "  🐍 Python tools: $(find scripts -name "*.py" 2>/dev/null | wc -l)"
    fi

    # Lines of code (if cloc is available)
    echo ""
    if command -v cloc >/dev/null 2>&1; then
        echo -e "${BLUE}📏 Lines of Code (by language):${NC}"
        cloc --quiet --hide-rate --include-lang=Rust,Python,JavaScript,TypeScript,HTML,CSS,YAML,Shell . 2>/dev/null | tail -n +4 || echo "  📊 cloc analysis failed"
    else
        echo -e "${YELLOW}📏 Lines of Code:${NC}"
        echo "  📊 Install 'cloc' for detailed code analysis"
        echo "  🦀 Approximate Rust files: $(find . -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}') lines"
    fi

    # Git statistics (if in git repo)
    echo ""
    if [[ -d ".git" ]]; then
        echo -e "${BLUE}📈 Git Statistics:${NC}"
        echo "  📝 Total commits: $(git rev-list --count HEAD 2>/dev/null || echo "N/A")"
        echo "  🌿 Branches: $(git branch -a 2>/dev/null | wc -l || echo "N/A")"
        echo "  👥 Contributors: $(git shortlog -sn --all 2>/dev/null | wc -l || echo "N/A")"

        # Recent activity
        if git log --oneline -10 >/dev/null 2>&1; then
            echo "  📅 Recent commits:"
            git log --oneline -5 2>/dev/null | sed 's/^/    /' || echo "    No recent commits found"
        fi
    fi

    # Service status (basic check)
    echo ""
    echo -e "${BLUE}🏃 Service Status:${NC}"
    local services_with_dockerfile=0
    local services_with_compose_meta=0

    for service_dir in layanan/*/; do
        if [[ -f "$service_dir/Dockerfile" ]]; then
            ((services_with_dockerfile++))
        fi
        if [[ -f "$service_dir/compose.meta.yaml" ]]; then
            ((services_with_compose_meta++))
        fi
    done

    echo "  🐳 Services with Dockerfile: $services_with_dockerfile"
    echo "  ⚙️  Services with compose meta: $services_with_compose_meta"

    # Frontend status
    local frontends_with_trunk=0
    local frontends_with_cargo=0

    for frontend_dir in antarmuka/*/; do
        [[ "$frontend_dir" == "antarmuka/shared/" ]] && continue
        if [[ -f "$frontend_dir/Trunk.toml" ]]; then
            ((frontends_with_trunk++))
        fi
        if [[ -f "$frontend_dir/Cargo.toml" ]]; then
            ((frontends_with_cargo++))
        fi
    done

    echo "  🌐 Frontends with Trunk config: $frontends_with_trunk"
    echo "  🦀 Frontends with Cargo.toml: $frontends_with_cargo"

    # Summary
    echo ""
    echo -e "${GREEN}🎯 Project Summary:${NC}"
    echo "  📊 This is a large-scale Rust microservices project"
    echo "  🏗️  Architecture: $(find layanan -maxdepth 1 -type d ! -name layanan | wc -l) backend + $(find antarmuka -maxdepth 1 -type d ! -name antarmuka -a ! -name shared | wc -l) frontend services"
    echo "  🔧 Tech Stack: Rust, Leptos, Docker, Kubernetes"
    echo "  📈 Maturity: Production-ready with comprehensive tooling"
}

# Show specific statistics
show_services() {
    echo -e "${CYAN}📦 Microservices Overview:${NC}"
    echo ""

    for service_dir in layanan/*/; do
        service_name=$(basename "$service_dir")
        status_indicators=""

        [[ -f "$service_dir/Dockerfile" ]] && status_indicators+="🐳 "
        [[ -f "$service_dir/Cargo.toml" ]] && status_indicators+="🦀 "
        [[ -f "$service_dir/compose.meta.yaml" ]] && status_indicators+="⚙️ "
        [[ -f "$service_dir/.gitlab-ci.yml" ]] && status_indicators+="🔄 "

        echo "  $service_name $status_indicators"
    done
}

show_frontends() {
    echo -e "${CYAN}🌐 Microfrontends Overview:${NC}"
    echo ""

    for frontend_dir in antarmuka/*/; do
        [[ "$frontend_dir" == "antarmuka/shared/" ]] && continue
        frontend_name=$(basename "$frontend_dir")
        status_indicators=""

        [[ -f "$frontend_dir/Trunk.toml" ]] && status_indicators+="🌐 "
        [[ -f "$frontend_dir/Cargo.toml" ]] && status_indicators+="🦀 "
        [[ -f "$frontend_dir/index.html" ]] && status_indicators+="📄 "
        [[ -f "$frontend_dir/.gitlab-ci.yml" ]] && status_indicators+="🔄 "

        echo "  $frontend_name $status_indicators"
    done
}

# Main execution
main() {
    cd "$WORKSPACE_ROOT" || {
        echo "❌ Cannot access workspace root: $WORKSPACE_ROOT"
        exit 1
    }

    case "${1:-stats}" in
        "stats"|"all")
            project_stats
            ;;
        "services")
            show_services
            ;;
        "frontends")
            show_frontends
            ;;
        "help"|"-h"|"--help")
            echo -e "${CYAN}📊 SIMPelv2 Project Statistics Tool${NC}"
            echo ""
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  stats        Show comprehensive project statistics (default)"
            echo "  services     Show microservices overview"
            echo "  frontends    Show microfrontends overview"
            echo "  help         Show this help"
            echo ""
            echo "Legend:"
            echo "  🐳 Docker    🦀 Rust    ⚙️ Config    🔄 CI/CD"
            echo "  🌐 Trunk    📄 HTML"
            ;;
        *)
            echo "❌ Unknown command: $1"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

main "$@"
