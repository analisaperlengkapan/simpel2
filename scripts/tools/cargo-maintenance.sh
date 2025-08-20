#!/bin/bash
# Cargo maintenance and optimization script

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🔧 Cargo Maintenance Script"

# Update dependencies and development tools
update_dependencies() {
    echo "📦 Updating dependencies and development tools..."

    cd "$PROJECT_ROOT"

    echo "🦀 Updating Rust dependencies..."

    # Update Cargo.lock
    cargo update

    # Check for outdated dependencies
    if command -v cargo-outdated &> /dev/null; then
        echo "📊 Checking for outdated dependencies:"
        cargo outdated --root-deps-only
    else
        echo "💡 Install cargo-outdated for dependency analysis: cargo install cargo-outdated"
    fi

    # Update development tools
    echo ""
    echo "🛠️  Updating development tools..."

    local tools=("cargo-watch" "trunk" "wasm-pack" "cargo-outdated" "cargo-audit")
    for tool in "${tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            echo "  ⬆️  Updating $tool..."
            cargo install "$tool" --force
        else
            echo "  📦 Installing $tool..."
            cargo install "$tool" || echo "  ⚠️  Failed to install $tool"
        fi
    done

    echo "✅ Dependencies and development tools updated"
}

# Analyze duplicates
analyze_duplicates() {
    echo "🔍 Analyzing duplicate dependencies..."

    cd "$PROJECT_ROOT"

    # Show duplicate dependencies
    cargo tree --duplicates --format "{p}" | sort | uniq -c | sort -nr | head -20

    echo ""
    echo "🔍 Top duplicate dependencies found above"
}

# Audit security vulnerabilities
audit_security() {
    echo "🔒 Security audit..."

    if command -v cargo-audit &> /dev/null; then
        cargo audit
    else
        echo "💡 Install cargo-audit for security scanning: cargo install cargo-audit"
    fi
}

# Check unused dependencies
check_unused() {
    echo "🧹 Checking for unused dependencies..."

    if command -v cargo-machete &> /dev/null; then
        cargo machete
    else
        echo "💡 Install cargo-machete for unused dependency detection: cargo install cargo-machete"
    fi
}

# Workspace health check
health_check() {
    echo "🏥 Workspace health check..."

    cd "$PROJECT_ROOT"

    # Check workspace configuration
    echo "✅ Workspace members:"
    cargo metadata --format-version 1 | jq -r '.workspace_members[]' | wc -l | xargs echo "   "

    # Check for profile issues
    echo "⚠️  Checking for profile conflicts:"
    find . -name "Cargo.toml" -not -path "./Cargo.toml" -exec grep -l "\[profile\." {} \; | head -5

    # Target directory size
    echo "💾 Target directory size:"
    du -sh target/ 2>/dev/null || echo "   Target directory not found"

    # Cache directory size if exists
    if [[ -d ".sccache" ]]; then
        echo "💾 Sccache directory size:"
        du -sh .sccache/
    fi
}

# Fix common issues
fix_issues() {
    echo "🔧 Fixing common issues..."

    cd "$PROJECT_ROOT"

    # Remove individual Cargo.lock files (should use workspace lock)
    echo "📝 Cleaning individual Cargo.lock files..."
    find . -name "Cargo.lock" -not -path "./Cargo.lock" -delete

    # Remove local target directories
    echo "📁 Removing local target directories..."
    find . -name "target" -not -path "./target" -type d -exec rm -rf {} + 2>/dev/null || true

    # Fix formatting
    if command -v cargo-fmt &> /dev/null; then
        echo "📝 Formatting code..."
        cargo fmt --all
    fi

    echo "✅ Issues fixed"
}

# Generate optimization report
generate_report() {
    echo "📋 Generating optimization report..."

    local report_file="$PROJECT_ROOT/CARGO_OPTIMIZATION_REPORT.md"

    cat > "$report_file" << EOF
# Cargo Optimization Report

Generated on: $(date)

## Workspace Overview
- Total workspace members: $(cargo metadata --format-version 1 | jq -r '.workspace_members[]' | wc -l)
- Rust version: $(rustc --version)
- Cargo version: $(cargo --version)

## Build Statistics
- Target directory size: $(du -sh target/ 2>/dev/null | cut -f1 || echo "Not found")
- WASM files count: $(find target/ -name "*.wasm" 2>/dev/null | wc -l || echo "0")

## Top Dependencies
\`\`\`
$(cargo tree --depth 1 | head -20)
\`\`\`

## Duplicate Dependencies
\`\`\`
$(cargo tree --duplicates --format "{p}" | sort | uniq -c | sort -nr | head -10)
\`\`\`

## Recommendations
1. ✅ Use workspace dependencies for version consistency
2. ✅ Enable sccache for build caching
3. ✅ Use profile inheritance to avoid duplication
4. ✅ Regular dependency audits and updates
5. ✅ WASM optimization with wasm-opt

## Next Steps
- Run \`./scripts/cache-config.sh setup\` to enable caching
- Run \`./scripts/wasm-optimizer.sh stats\` for WASM analysis
- Consider using \`cargo-outdated\` for dependency updates
EOF

    echo "📋 Report generated: $report_file"
}

# Main execution
main() {
    cd "$PROJECT_ROOT"

    case "${1:-health}" in
        "update")
            update_dependencies
            ;;
        "duplicates")
            analyze_duplicates
            ;;
        "audit")
            audit_security
            ;;
        "unused")
            check_unused
            ;;
        "health")
            health_check
            ;;
        "fix")
            fix_issues
            ;;
        "report")
            generate_report
            ;;
        "all")
            health_check
            analyze_duplicates
            fix_issues
            generate_report
            ;;
        *)
            echo "Usage: $0 {update|duplicates|audit|unused|health|fix|report|all}"
            echo ""
            echo "Commands:"
            echo "  update      - Update dependencies"
            echo "  duplicates  - Analyze duplicate dependencies"
            echo "  audit       - Security audit"
            echo "  unused      - Check unused dependencies"
            echo "  health      - Workspace health check"
            echo "  fix         - Fix common issues"
            echo "  report      - Generate optimization report"
            echo "  all         - Run all checks and generate report"
            exit 1
            ;;
    esac
}

main "$@"
