# ====== SIMPelv2 Testing Operations Makefile ======
# Replaces test runner scripts with make targets
# Version: 4.0.0 - Converted from shell scripts

# ====== CONFIGURATION ======
SCRIPTS_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
WORKSPACE_ROOT := $(shell cd $(SCRIPTS_DIR)/../.. && pwd)
TEST_DIR := $(SCRIPTS_DIR)/../test

# Colors
GREEN := \033[0;32m
BLUE := \033[0;34m
CYAN := \033[0;36m
YELLOW := \033[1;33m
RED := \033[0;31m
PURPLE := \033[0;35m
NC := \033[0m

# ====== UNIT TEST TARGETS ======
.PHONY: test-rust test-rust-service-% test-leptos

test-rust: ## Run all Rust unit tests
	@echo -e "$(CYAN)🧪 Running Rust unit tests...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo test --workspace --lib

test-rust-service-%: ## Run tests for specific Rust service (e.g., test-rust-service-keamanan)
	@echo -e "$(CYAN)🧪 Running tests for layanan/$*...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	if [[ -f "layanan/$*/Cargo.toml" ]]; then \
		cargo test --manifest-path "layanan/$*/Cargo.toml"; \
	else \
		echo -e "$(RED)❌ Service layanan/$* not found$(NC)"; \
		exit 1; \
	fi

test-leptos: ## Run Leptos frontend tests
	@echo -e "$(BLUE)🎨 Running Leptos frontend tests...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	for frontend in antarmuka/portal antarmuka/badiklat antarmuka/datun antarmuka/intel; do \
		if [[ -f "$$frontend/Cargo.toml" ]]; then \
			echo -e "Testing $$frontend..."; \
			cargo test --manifest-path "$$frontend/Cargo.toml" --target wasm32-unknown-unknown; \
		fi; \
	done

# ====== INTEGRATION TEST TARGETS ======
.PHONY: test-integration test-api test-e2e

test-integration: ## Run integration tests
	@echo -e "$(PURPLE)🔗 Running integration tests...$(NC)"
	@if [[ -f "$(TEST_DIR)/test-runner.sh" ]]; then \
		$(TEST_DIR)/test-runner.sh integration; \
	else \
		echo -e "$(YELLOW)⚠️  Integration test runner not found$(NC)"; \
	fi

test-api: ## Run API tests
	@echo -e "$(GREEN)🌐 Running API tests...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo test --test api_tests --features integration

test-e2e: ## Run end-to-end tests
	@echo -e "$(BLUE)🎭 Running E2E tests...$(NC)"
	@if [[ -f "$(TEST_DIR)/e2e/e2e-runner.sh" ]]; then \
		$(TEST_DIR)/e2e/e2e-runner.sh; \
	else \
		echo -e "$(YELLOW)⚠️  E2E test runner not found$(NC)"; \
	fi

# ====== PERFORMANCE TEST TARGETS ======
.PHONY: test-performance test-benchmark test-load test-stress

test-performance: test-benchmark test-load ## Run all performance tests

test-benchmark: ## Run performance benchmarks
	@echo -e "$(YELLOW)⚡ Running performance benchmarks...$(NC)"
	@if [[ -f "$(TEST_DIR)/performance/benchmark.sh" ]]; then \
		$(TEST_DIR)/performance/benchmark.sh; \
	else \
		echo -e "$(RED)❌ Benchmark script not found$(NC)"; \
	fi

test-load: ## Run load testing
	@echo -e "$(CYAN)📊 Running load tests...$(NC)"
	@if [[ -f "$(TEST_DIR)/performance/load-test.sh" ]]; then \
		$(TEST_DIR)/performance/load-test.sh; \
	else \
		echo -e "$(YELLOW)⚠️  Load test script not found$(NC)"; \
	fi

test-stress: ## Run stress testing
	@echo -e "$(RED)💥 Running stress tests...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo test --test stress_tests --release

# ====== SECURITY TEST TARGETS ======
.PHONY: test-security test-audit test-vulnerability test-dependency-check

test-security: test-audit test-vulnerability test-dependency-check ## Run all security tests

test-audit: ## Run security audit
	@echo -e "$(RED)🔒 Running security audit...$(NC)"
	@if [[ -f "$(TEST_DIR)/security/security-audit.sh" ]]; then \
		$(TEST_DIR)/security/security-audit.sh; \
	else \
		echo -e "$(YELLOW)⚠️  Security audit script not found$(NC)"; \
	fi

test-vulnerability: ## Run vulnerability scan
	@echo -e "$(RED)🛡️  Running vulnerability scan...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo audit

test-dependency-check: ## Check dependencies for security issues
	@echo -e "$(CYAN)📦 Checking dependencies...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo deny check

# ====== VALIDATION TEST TARGETS ======
.PHONY: test-validation test-yaml test-docker test-config

test-validation: test-yaml test-docker test-config ## Run all validation tests

test-yaml: ## Validate YAML files
	@echo -e "$(BLUE)📄 Validating YAML files...$(NC)"
	@if [[ -f "$(TEST_DIR)/validation/yaml-validator.sh" ]]; then \
		$(TEST_DIR)/validation/yaml-validator.sh; \
	else \
		echo -e "$(YELLOW)⚠️  YAML validator not found$(NC)"; \
	fi

test-docker: ## Test Docker configurations
	@echo -e "$(CYAN)🐳 Testing Docker configurations...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	for dockerfile in $$(find . -name "Dockerfile" -not -path "./target/*"); do \
		echo "Validating $$dockerfile"; \
		docker build -t test-build -f "$$dockerfile" . --no-cache || echo "Failed: $$dockerfile"; \
	done

test-config: ## Validate configuration files
	@echo -e "$(GREEN)⚙️  Validating configuration files...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	find . -name "*.toml" -not -path "./target/*" -exec echo "Checking {}" \; -exec toml-cli check {} \; || true

# ====== COVERAGE TARGETS ======
.PHONY: test-coverage test-coverage-html test-coverage-report

test-coverage: ## Generate test coverage
	@echo -e "$(PURPLE)📊 Generating test coverage...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo tarpaulin --out Html --output-dir coverage

test-coverage-html: test-coverage ## Generate HTML coverage report
	@echo -e "$(BLUE)🌐 Coverage report generated in coverage/tarpaulin-report.html$(NC)"

test-coverage-report: ## Show coverage summary
	@echo -e "$(CYAN)📋 Coverage Summary:$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo tarpaulin --out Stdout

# ====== COMPREHENSIVE TEST TARGETS ======
.PHONY: test-all test-quick test-full test-ci

test-quick: test-rust ## Quick test run (unit tests only)
	@echo -e "$(GREEN)✅ Quick tests completed$(NC)"

test-all: test-rust test-leptos test-integration test-performance test-security test-validation ## Run all tests
	@echo -e "$(GREEN)🎉 All tests completed successfully$(NC)"

test-full: test-all test-coverage ## Complete test suite with coverage
	@echo -e "$(PURPLE)🎯 Full test suite with coverage completed$(NC)"

test-ci: test-rust test-integration test-security ## CI/CD optimized test suite
	@echo -e "$(CYAN)🚀 CI test suite completed$(NC)"

# ====== MONITORING & HEALTH CHECK ======
.PHONY: health-check test-health monitoring-check

health-check: ## Run system health checks
	@echo -e "$(GREEN)🏥 Running health checks...$(NC)"
	@if [[ -f "$(TEST_DIR)/monitoring/health-check.sh" ]]; then \
		$(TEST_DIR)/monitoring/health-check.sh; \
	else \
		echo -e "$(YELLOW)⚠️  Health check script not found$(NC)"; \
	fi

test-health: health-check ## Alias for health-check

monitoring-check: ## Check monitoring systems
	@echo -e "$(CYAN)📊 Checking monitoring systems...$(NC)"
	@curl -f http://localhost:3000/health 2>/dev/null && echo -e "$(GREEN)✅ Main service healthy$(NC)" || echo -e "$(RED)❌ Main service down$(NC)"

# ====== HELP TARGET ======
help-test-mk: ## Show testing operations help
	@echo -e "$(CYAN)🧪 Testing Operations$(NC)"
	@echo "======================"
	@echo ""
	@echo -e "$(YELLOW)🦀 Unit Tests:$(NC)"
	@echo "  make test-rust              - All Rust unit tests"
	@echo "  make test-rust-service-<name> - Specific service tests"
	@echo "  make test-leptos            - Leptos frontend tests"
	@echo ""
	@echo -e "$(PURPLE)🔗 Integration Tests:$(NC)"
	@echo "  make test-integration       - Integration tests"
	@echo "  make test-api               - API tests"
	@echo "  make test-e2e               - End-to-end tests"
	@echo ""
	@echo -e "$(YELLOW)⚡ Performance Tests:$(NC)"
	@echo "  make test-performance       - All performance tests"
	@echo "  make test-benchmark         - Benchmarks"
	@echo "  make test-load              - Load testing"
	@echo "  make test-stress            - Stress testing"
	@echo ""
	@echo -e "$(RED)🔒 Security Tests:$(NC)"
	@echo "  make test-security          - All security tests"
	@echo "  make test-audit             - Security audit"
	@echo "  make test-vulnerability     - Vulnerability scan"
	@echo "  make test-dependency-check  - Dependency security check"
	@echo ""
	@echo -e "$(BLUE)✅ Validation Tests:$(NC)"
	@echo "  make test-validation        - All validation tests"
	@echo "  make test-yaml              - YAML validation"
	@echo "  make test-docker            - Docker validation"
	@echo "  make test-config            - Config validation"
	@echo ""
	@echo -e "$(PURPLE)📊 Coverage:$(NC)"
	@echo "  make test-coverage          - Generate coverage"
	@echo "  make test-coverage-html     - HTML coverage report"
	@echo "  make test-coverage-report   - Coverage summary"
	@echo ""
	@echo -e "$(GREEN)🎯 Comprehensive:$(NC)"
	@echo "  make test-quick             - Quick tests (unit only)"
	@echo "  make test-all               - All tests"
	@echo "  make test-full              - Full suite + coverage"
	@echo "  make test-ci                - CI/CD optimized"
	@echo ""
	@echo -e "$(CYAN)🏥 Health:$(NC)"
	@echo "  make health-check           - System health"
	@echo "  make monitoring-check       - Monitoring status"
