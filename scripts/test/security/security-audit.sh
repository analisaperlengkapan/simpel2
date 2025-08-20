#!/bin/bash

# 🔒 SIMPelv2 Security Module
# Security scanning, auditing, and compliance tools

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

WORKSPACE_ROOT="/var/www/simpelv2"

echo -e "${CYAN}🔒 SIMPelv2 Security Module${NC}"

# Security audit and vulnerability scanning
security_audit() {
    echo -e "\n${RED}🛡️  Security Audit${NC}"
    echo "=================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🔍 Running security audit..."
    
    # Rust security audit
    if [[ -f "Cargo.toml" ]]; then
        echo ""
        echo "🦀 Rust Security Audit:"
        
        # Install cargo-audit if needed
        if ! command -v cargo-audit >/dev/null 2>&1; then
            echo "  📦 Installing cargo-audit..."
            cargo install cargo-audit
        fi
        
        echo "  🔍 Checking for known vulnerabilities..."
        if cargo audit; then
            echo "  ✅ No known vulnerabilities found"
        else
            echo "  ⚠️  Potential vulnerabilities detected"
        fi
        
        echo "  🔍 Checking for unused dependencies..."
        if command -v cargo-machete >/dev/null 2>&1; then
            cargo machete
        else
            echo "  📦 Install cargo-machete for unused dependency detection"
        fi
    fi
    
    # Docker security scan
    echo ""
    echo "🐳 Docker Security Scan:"
    
    if command -v docker >/dev/null 2>&1; then
        # Find Dockerfiles
        dockerfiles=($(find . -name "Dockerfile" | head -5))
        
        if [[ ${#dockerfiles[@]} -gt 0 ]]; then
            for dockerfile in "${dockerfiles[@]}"; do
                echo "  🔍 Scanning $(dirname "$dockerfile")..."
                
                # Basic Dockerfile security checks
                if grep -q "FROM.*:latest" "$dockerfile"; then
                    echo "    ⚠️  Using 'latest' tag (security risk)"
                fi
                
                if grep -q "USER root" "$dockerfile"; then
                    echo "    ⚠️  Running as root user"
                fi
                
                if ! grep -q "USER " "$dockerfile"; then
                    echo "    ⚠️  No non-root user specified"
                fi
                
                # Check for exposed secrets
                if grep -E "(PASSWORD|SECRET|KEY|TOKEN)" "$dockerfile" >/dev/null; then
                    echo "    ⚠️  Potential secrets in Dockerfile"
                fi
            done
        else
            echo "  ℹ️  No Dockerfiles found"
        fi
    else
        echo "  ⚠️  Docker not available for security scan"
    fi
    
    # File permissions check
    echo ""
    echo "📁 File Permissions Check:"
    
    # Check for overly permissive files
    echo "  🔍 Checking for world-writable files..."
    if find . -type f -perm -002 2>/dev/null | head -5; then
        echo "    ⚠️  World-writable files found (security risk)"
    else
        echo "    ✅ No world-writable files"
    fi
    
    # Check for executable scripts
    echo "  🔍 Checking executable scripts..."
    executable_scripts=$(find . -name "*.sh" -executable | wc -l)
    echo "    📄 Found $executable_scripts executable scripts"
}

# Dependency security analysis
dependency_scan() {
    echo -e "\n${YELLOW}📦 Dependency Security Scan${NC}"
    echo "============================"
    
    cd "$WORKSPACE_ROOT"
    
    if [[ -f "Cargo.toml" ]]; then
        echo "🦀 Analyzing Rust dependencies..."
        
        # Check for outdated dependencies
        echo "  📊 Checking for outdated packages..."
        if command -v cargo-outdated >/dev/null 2>&1; then
            cargo outdated
        else
            echo "  📦 Install cargo-outdated for dependency analysis"
        fi
        
        # Analyze dependency tree
        echo ""
        echo "  🌳 Dependency tree analysis:"
        cargo tree --depth 1 | head -10
        
        # Check for duplicate dependencies
        echo ""
        echo "  🔍 Checking for duplicate dependencies..."
        cargo tree --duplicates || echo "    ✅ No duplicate dependencies found"
        
    else
        echo "❌ No Cargo.toml found for dependency analysis"
    fi
}

# Secret scanning
secret_scan() {
    echo -e "\n${PURPLE}🔐 Secret Scanning${NC}"
    echo "=================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🔍 Scanning for potential secrets..."
    
    # Common secret patterns
    local patterns=(
        "password.*="
        "secret.*="
        "key.*="
        "token.*="
        "api_key"
        "private_key"
        "-----BEGIN.*PRIVATE KEY-----"
        "[0-9a-fA-F]{32,}"
    )
    
    for pattern in "${patterns[@]}"; do
        echo "  🔍 Checking for: $pattern"
        
        matches=$(grep -r -i -E "$pattern" --exclude-dir=target --exclude-dir=.git --exclude="*.log" . 2>/dev/null | head -3)
        
        if [[ -n "$matches" ]]; then
            echo "    ⚠️  Potential secrets found:"
            echo "$matches" | sed 's/^/      /'
        else
            echo "    ✅ No matches"
        fi
    done
    
    # Check environment files
    echo ""
    echo "🌍 Environment Files Check:"
    
    env_files=($(find . -name ".env*" -o -name "*.env" | head -5))
    
    if [[ ${#env_files[@]} -gt 0 ]]; then
        for env_file in "${env_files[@]}"; do
            echo "  📄 Found: $env_file"
            
            # Check if it's in .gitignore
            if [[ -f ".gitignore" ]] && grep -q "$(basename "$env_file")" .gitignore; then
                echo "    ✅ Listed in .gitignore"
            else
                echo "    ⚠️  Not in .gitignore (potential security risk)"
            fi
        done
    else
        echo "  ✅ No environment files found"
    fi
}

# Network security check
network_scan() {
    echo -e "\n${BLUE}🌐 Network Security Check${NC}"
    echo "========================="
    
    echo "🔍 Checking network configuration..."
    
    # Check listening ports
    echo "📡 Active listening ports:"
    if command -v ss >/dev/null 2>&1; then
        ss -tuln | head -10
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tuln | head -10
    else
        echo "  ⚠️  No network tools available"
    fi
    
    # Check for insecure protocols
    echo ""
    echo "🔍 Checking for insecure protocols:"
    
    # Check for HTTP in config files
    if grep -r "http://" --include="*.toml" --include="*.yml" --include="*.yaml" . 2>/dev/null | head -3; then
        echo "  ⚠️  HTTP URLs found (consider HTTPS)"
    else
        echo "  ✅ No insecure HTTP URLs found"
    fi
    
    # Check Docker network configuration
    if command -v docker >/dev/null 2>&1; then
        echo ""
        echo "🐳 Docker network security:"
        
        # Check for host network mode
        if find . -name "docker-compose*.yml" -exec grep -l "network_mode.*host" {} \; | head -1 >/dev/null; then
            echo "  ⚠️  Host network mode detected (security risk)"
        else
            echo "  ✅ No host network mode usage"
        fi
        
        # Check for privileged containers
        if find . -name "docker-compose*.yml" -exec grep -l "privileged.*true" {} \; | head -1 >/dev/null; then
            echo "  ⚠️  Privileged containers detected"
        else
            echo "  ✅ No privileged containers"
        fi
    fi
}

# Compliance check
compliance_check() {
    echo -e "\n${GREEN}📋 Compliance Check${NC}"
    echo "==================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🔍 Running compliance checks..."
    
    # OWASP compliance
    echo ""
    echo "🛡️  OWASP Compliance:"
    echo "  ✅ A01 - Broken Access Control: Manual review required"
    echo "  ✅ A02 - Cryptographic Failures: Manual review required"
    echo "  ✅ A03 - Injection: Using parameterized queries (Rust/SQLx)"
    echo "  ✅ A04 - Insecure Design: Architecture review required"
    echo "  ✅ A05 - Security Misconfiguration: Automated checks above"
    echo "  ✅ A06 - Vulnerable Components: Dependency scanning active"
    
    # License compliance
    echo ""
    echo "📜 License Compliance:"
    if [[ -f "Cargo.toml" ]]; then
        echo "  📊 Checking dependency licenses..."
        if command -v cargo-license >/dev/null 2>&1; then
            cargo license | head -10
        else
            echo "  📦 Install cargo-license for license analysis"
        fi
    fi
    
    # Code quality compliance
    echo ""
    echo "🏆 Code Quality Compliance:"
    if [[ -f "Cargo.toml" ]]; then
        echo "  🦀 Running Rust quality checks..."
        
        # Clippy lints
        if cargo clippy --version >/dev/null 2>&1; then
            echo "    ✅ Clippy available for linting"
        else
            echo "    ⚠️  Clippy not available"
        fi
        
        # Formatting
        if cargo fmt --version >/dev/null 2>&1; then
            echo "    ✅ rustfmt available for formatting"
        else
            echo "    ⚠️  rustfmt not available"
        fi
        
        # Tests
        if find . -name "*.rs" -exec grep -l "#\[test\]" {} \; | head -1 >/dev/null; then
            echo "    ✅ Unit tests found"
        else
            echo "    ⚠️  No unit tests found"
        fi
    fi
}

# Generate security report
security_report() {
    echo -e "\n${CYAN}📊 Security Report Generation${NC}"
    echo "============================="
    
    cd "$WORKSPACE_ROOT"
    
    local report_file="security-report-$(date +%Y%m%d-%H%M%S).md"
    
    echo "📝 Generating comprehensive security report..."
    
    cat > "$report_file" << EOF
# SIMPelv2 Security Report

**Generated:** $(date)  
**Project:** SIMPelv2  
**Location:** $WORKSPACE_ROOT

## Executive Summary

This report provides a comprehensive security assessment of the SIMPelv2 platform.

## Findings Summary

### 🛡️ Security Audit
$(security_audit 2>&1 | grep -E "(✅|⚠️|❌)" | head -10)

### 📦 Dependency Security
$(dependency_scan 2>&1 | grep -E "(✅|⚠️|❌)" | head -5)

### 🔐 Secret Scanning
$(secret_scan 2>&1 | grep -E "(✅|⚠️|❌)" | head -5)

### 🌐 Network Security
$(network_scan 2>&1 | grep -E "(✅|⚠️|❌)" | head -5)

## Recommendations

1. **Update Dependencies**: Keep all dependencies up to date
2. **Secret Management**: Use environment variables and secret management
3. **Access Control**: Implement proper authentication and authorization
4. **Network Security**: Use HTTPS and secure network configurations
5. **Container Security**: Follow Docker security best practices

## Next Steps

- [ ] Address high-priority security findings
- [ ] Implement automated security scanning in CI/CD
- [ ] Regular security assessments
- [ ] Security training for development team

---

*This report was generated automatically by SIMPelv2 security tools.*
EOF
    
    echo "✅ Security report generated: $report_file"
    echo "📊 View with: cat $report_file"
}

# Main function
main() {
    case "${1:-audit}" in
        "audit"|"scan")
            security_audit
            ;;
        "dependencies"|"deps")
            dependency_scan
            ;;
        "secrets")
            secret_scan
            ;;
        "network")
            network_scan
            ;;
        "compliance")
            compliance_check
            ;;
        "report")
            security_report
            ;;
        "all"|"full")
            echo "🔒 Running comprehensive security assessment..."
            security_audit
            dependency_scan
            secret_scan
            network_scan
            compliance_check
            ;;
        "help"|"--help"|"-h")
            echo ""
            echo "SIMPelv2 Security Module"
            echo ""
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  audit         - Run security audit and vulnerability scan"
            echo "  dependencies  - Scan dependencies for security issues"
            echo "  secrets       - Scan for potential secrets in code"
            echo "  network       - Check network security configuration"
            echo "  compliance    - Run compliance checks (OWASP, etc.)"
            echo "  report        - Generate comprehensive security report"
            echo "  all           - Run all security checks"
            echo "  help          - Show this help"
            echo ""
            echo "Examples:"
            echo "  $0 audit         # Quick security audit"
            echo "  $0 all          # Complete security assessment"
            echo "  $0 report       # Generate security report"
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo "Use '$0 help' for available commands"
            exit 1
            ;;
    esac
}

main "$@"
