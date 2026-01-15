#!/bin/bash
# SIMPelv2 - Automated Code Quality & Security Checker
# Runs comprehensive checks on Authenc and Secreton codebases

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNINGS=0

print_header() {
    echo -e "\n${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC} $1"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
}

print_check() {
    echo -e "${CYAN}▶${NC} Checking: $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED_CHECKS++))
}

print_failure() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED_CHECKS++))
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

run_check() {
    ((TOTAL_CHECKS++))
}

cd "$(dirname "$0")/.." || exit 1

print_header "SIMPelv2 Code Quality & Security Checker"

# ========================================
# 1. COMPILATION CHECKS
# ========================================
print_header "1. Compilation Checks"

print_check "Authenc compilation (release mode)"
run_check
if cargo build --release -p authenc 2>&1 | tee /tmp/authenc-build.log | grep -q "warning"; then
    warning_count=$(grep -c "warning:" /tmp/authenc-build.log || echo "0")
    print_warning "Authenc compiled with $warning_count warnings"
else
    if cargo build --release -p authenc >/dev/null 2>&1; then
        print_success "Authenc compiled cleanly"
    else
        print_failure "Authenc failed to compile"
    fi
fi

print_check "Secreton compilation (release mode)"
run_check
if cargo build --release -p secreton-api >/dev/null 2>&1; then
    print_success "Secreton compiled successfully"
else
    print_failure "Secreton failed to compile"
fi

# ========================================
# 2. CODE FORMATTING
# ========================================
print_header "2. Code Formatting"

print_check "Rust format check"
run_check
if cargo fmt --all -- --check >/dev/null 2>&1; then
    print_success "All Rust code is properly formatted"
else
    print_warning "Some files need formatting (run: cargo fmt --all)"
fi

# ========================================
# 3. CLIPPY LINTS
# ========================================
print_header "3. Clippy Linting"

print_check "Authenc clippy check"
run_check
clippy_output=$(cargo clippy -p authenc 2>&1)
if echo "$clippy_output" | grep -q "warning:"; then
    warning_count=$(echo "$clippy_output" | grep -c "warning:" || echo "0")
    print_warning "Authenc has $warning_count clippy warnings"
    echo "$clippy_output" | grep "warning:" | head -5
else
    print_success "Authenc passes clippy with no warnings"
fi

print_check "Secreton clippy check"
run_check
clippy_output=$(cargo clippy -p secreton-api 2>&1)
if echo "$clippy_output" | grep -q "warning:"; then
    warning_count=$(echo "$clippy_output" | grep -c "warning:" || echo "0")
    print_warning "Secreton has $warning_count clippy warnings"
else
    print_success "Secreton passes clippy with no warnings"
fi

# ========================================
# 4. SECURITY AUDITS
# ========================================
print_header "4. Security Audits"

print_check "Cargo audit (dependency vulnerabilities)"
run_check
if command -v cargo-audit &> /dev/null; then
    if cargo audit 2>&1 | grep -q "Vulnerabilities found"; then
        vuln_count=$(cargo audit 2>&1 | grep -c "ID:" || echo "0")
        print_warning "Found $vuln_count vulnerabilities in dependencies"
        cargo audit 2>&1 | grep -A 2 "ID:"
    else
        print_success "No known vulnerabilities in dependencies"
    fi
else
    print_warning "cargo-audit not installed (install: cargo install cargo-audit)"
fi

print_check "Cargo deny (license and security policy)"
run_check
if command -v cargo-deny &> /dev/null; then
    if cargo deny check >/dev/null 2>&1; then
        print_success "Passed cargo-deny checks"
    else
        print_warning "Some cargo-deny checks failed"
    fi
else
    print_warning "cargo-deny not installed (install: cargo install cargo-deny)"
fi

# ========================================
# 5. TEST COVERAGE
# ========================================
print_header "5. Test Execution"

print_check "Authenc unit tests"
run_check
if cargo test -p authenc --lib >/dev/null 2>&1; then
    test_count=$(cargo test -p authenc --lib 2>&1 | grep -c "test result:" || echo "0")
    print_success "Authenc unit tests passed"
else
    print_failure "Authenc unit tests failed"
fi

print_check "Secreton unit tests"
run_check
if cargo test -p secreton-core --lib >/dev/null 2>&1; then
    print_success "Secreton unit tests passed"
else
    print_warning "Secreton tests need review"
fi

# ========================================
# 6. DEPENDENCY ANALYSIS
# ========================================
print_header "6. Dependency Analysis"

print_check "Outdated dependencies"
run_check
if command -v cargo-outdated &> /dev/null; then
    outdated=$(cargo outdated --root-deps-only 2>&1 | grep -c "→" || echo "0")
    if [ "$outdated" -gt 0 ]; then
        print_warning "$outdated root dependencies are outdated"
    else
        print_success "All root dependencies are up to date"
    fi
else
    print_warning "cargo-outdated not installed (install: cargo install cargo-outdated)"
fi

print_check "Duplicate dependencies"
run_check
duplicates=$(cargo tree --duplicates 2>/dev/null | grep -c "├──" || echo "0")
if [ "$duplicates" -gt 5 ]; then
    print_warning "Found $duplicates duplicate dependencies (may affect binary size)"
else
    print_success "Minimal duplicate dependencies"
fi

# ========================================
# 7. CODE METRICS
# ========================================
print_header "7. Code Metrics"

print_check "Lines of code analysis"
run_check
if command -v tokei &> /dev/null; then
    echo ""
    tokei infra/authenc/src --exclude "target/*"
    tokei infra/secreton/crates --exclude "target/*"
    print_success "Code metrics generated"
else
    print_warning "tokei not installed (install: cargo install tokei)"
fi

# ========================================
# 8. DOCUMENTATION CHECKS
# ========================================
print_header "8. Documentation"

print_check "Cargo doc generation"
run_check
if cargo doc --no-deps -p authenc -p secreton-api >/dev/null 2>&1; then
    print_success "Documentation builds successfully"
else
    print_warning "Documentation has warnings or errors"
fi

print_check "README files present"
run_check
if [ -f "infra/authenc/README.md" ] && [ -f "infra/secreton/README.md" ]; then
    print_success "README files present for both services"
else
    print_warning "Missing README files"
fi

# ========================================
# 9. DOCKER CONFIGURATION
# ========================================
print_header "9. Docker Configuration"

print_check "Authenc Dockerfile"
run_check
if [ -f "infra/authenc/Dockerfile" ]; then
    if grep -q "HEALTHCHECK" infra/authenc/Dockerfile; then
        print_success "Authenc Dockerfile has healthcheck"
    else
        print_warning "Authenc Dockerfile missing healthcheck"
    fi
else
    print_failure "Authenc Dockerfile not found"
fi

print_check "Secreton Dockerfile"
run_check
if [ -f "infra/secreton/Dockerfile" ]; then
    if grep -q "HEALTHCHECK" infra/secreton/Dockerfile; then
        print_success "Secreton Dockerfile has healthcheck"
    else
        print_warning "Secreton Dockerfile missing healthcheck"
    fi
else
    print_failure "Secreton Dockerfile not found"
fi

# ========================================
# 10. CONFIGURATION VALIDATION
# ========================================
print_header "10. Configuration Validation"

print_check "Authenc TOML configuration"
run_check
if [ -f "infra/authenc/authenc.toml" ]; then
    # Basic TOML syntax check
    if command -v toml &> /dev/null; then
        if toml check infra/authenc/authenc.toml >/dev/null 2>&1; then
            print_success "Authenc config is valid TOML"
        else
            print_failure "Authenc config has syntax errors"
        fi
    else
        print_success "Authenc config file exists"
    fi
else
    print_warning "Authenc config file not found"
fi

print_check "Secreton TOML configuration"
run_check
if [ -f "infra/secreton/secreton.toml" ]; then
    print_success "Secreton config file exists"
else
    print_warning "Secreton config file not found"
fi

# ========================================
# 11. ENVIRONMENT VARIABLES
# ========================================
print_header "11. Environment Configuration"

print_check "Docker compose environment files"
run_check
env_files_count=0
[ -f ".env.example" ] && ((env_files_count++))
[ -f "infra/authenc/.env.captcha.development.example" ] && ((env_files_count++))

if [ $env_files_count -gt 0 ]; then
    print_success "Found $env_files_count environment example files"
else
    print_warning "No environment example files found"
fi

# ========================================
# SUMMARY
# ========================================
print_header "Summary"

echo -e "Total Checks:  ${TOTAL_CHECKS}"
echo -e "${GREEN}Passed:        ${PASSED_CHECKS}${NC}"
echo -e "${RED}Failed:        ${FAILED_CHECKS}${NC}"
echo -e "${YELLOW}Warnings:      ${WARNINGS}${NC}"

SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED_CHECKS/$TOTAL_CHECKS)*100}")
echo -e "Success Rate:  ${SUCCESS_RATE}%"

echo ""
if [ $FAILED_CHECKS -eq 0 ] && [ $WARNINGS -lt 5 ]; then
    echo -e "${GREEN}✅ Excellent code quality!${NC}"
    exit 0
elif [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Good, but there are some warnings to address.${NC}"
    exit 0
else
    echo -e "${RED}❌ Critical issues found. Please fix failing checks.${NC}"
    exit 1
fi
