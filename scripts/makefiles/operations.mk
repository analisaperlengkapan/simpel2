# ====== SIMPelv2 Operations Makefile ======
# Focused on deployment, testing, and tool operations only
# Version: 4.1.0 - Consolidated (Build operations moved to builds.mk)

# ====== CONFIGURATION ======
SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c
SCRIPTS_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
WORKSPACE_ROOT := $(shell cd $(SCRIPTS_DIR)/../.. && pwd)

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
RED := \033[0;31m
NC := \033[0m

# ====== TEST TARGETS ======
.PHONY: test-performance test-security test-validation


test-performance: ## Run performance benchmarks
	@echo -e "$(BLUE)⚡ Running performance tests...$(NC)"
	@$(SCRIPTS_DIR)/test/performance/benchmark.sh

test-security: ## Run security audits
	@echo -e "$(RED)🔒 Running security audits...$(NC)"
	@$(SCRIPTS_DIR)/test/security/security-audit.sh

test-validation: ## Run validation tests
	@echo -e "$(YELLOW)✅ Running validation tests...$(NC)"
	@$(SCRIPTS_DIR)/test/validation/yaml-validator.sh

# ====== DEPLOYMENT TARGETS ======
.PHONY: deploy-dev deploy-staging deploy-prod monitoring backup

deploy-dev: ## Deploy to development environment
	@echo -e "$(GREEN)🚀 Deploying to development...$(NC)"
	@$(SCRIPTS_DIR)/ops/deploy.sh dev

deploy-staging: ## Deploy to staging environment
	@echo -e "$(YELLOW)🚀 Deploying to staging...$(NC)"
	@$(SCRIPTS_DIR)/ops/deploy.sh staging

deploy-prod: ## Deploy to production environment
	@echo -e "$(RED)🚀 Deploying to production...$(NC)"
	@$(SCRIPTS_DIR)/ops/deploy.sh prod

monitoring: ## Start system monitoring
	@echo -e "$(CYAN)📊 Starting system monitoring...$(NC)"
	@$(SCRIPTS_DIR)/ops/monitoring.sh overview

backup: ## Perform maintenance backup
	@echo -e "$(BLUE)💾 Performing maintenance backup...$(NC)"
	@$(SCRIPTS_DIR)/ops/backup.sh maintenance

# ====== TOOL TARGETS ======
.PHONY: wasm-stats wasm-optimize cargo-status cargo-update ai-generate project-init

wasm-stats: ## Show WASM optimization statistics
	@echo -e "$(PURPLE)📊 WASM Statistics...$(NC)"
	@$(SCRIPTS_DIR)/tools/wasm-optimizer.sh stats

wasm-optimize: ## Optimize WASM bundles
	@echo -e "$(GREEN)⚡ Optimizing WASM bundles...$(NC)"
	@$(SCRIPTS_DIR)/tools/wasm-optimizer.sh optimize

cargo-status: ## Show cargo maintenance status
	@echo -e "$(CYAN)📦 Cargo status...$(NC)"
	@$(SCRIPTS_DIR)/tools/cargo-maintenance.sh status

cargo-update: ## Update cargo dependencies
	@echo -e "$(YELLOW)📦 Updating cargo dependencies...$(NC)"
	@$(SCRIPTS_DIR)/tools/cargo-maintenance.sh update

ai-generate: ## Generate code using AI tools
	@echo -e "$(PURPLE)🤖 AI code generation...$(NC)"
	@$(SCRIPTS_DIR)/tools/ai/ai-tools.sh generate

project-init: ## Initialize new project component
	@echo -e "$(GREEN)🚀 Project initialization...$(NC)"
	@$(SCRIPTS_DIR)/tools/project-init.sh

# ====== STATUS & INFO TARGETS ======
.PHONY: status info workspace-stats

status: ## Show system status
	@echo -e "$(CYAN)📋 SIMPelv2 System Status$(NC)"
	@echo "=================================="
	@echo "Version: 4.0.0"
	@echo "Workspace: $(WORKSPACE_ROOT)"
	@echo "Scripts Dir: $(SCRIPTS_DIR)"
	@echo ""
	@echo -e "$(GREEN)🔍 System Check:$(NC)"
	@command -v cargo >/dev/null 2>&1 && echo "  ✅ Cargo available" || echo "  ❌ Cargo not found"
	@command -v docker >/dev/null 2>&1 && echo "  ✅ Docker available" || echo "  ❌ Docker not found"
	@command -v trunk >/dev/null 2>&1 && echo "  ✅ Trunk available" || echo "  ❌ Trunk not found"

workspace-stats: ## Show workspace statistics
	@echo -e "$(BLUE)📊 Workspace Statistics$(NC)"
	@$(SCRIPTS_DIR)/tools/project-stats.sh

info: status ## Show system information (alias for status)

# ====== HELP TARGET ======
help-simpel: ## Show SIMPelv2 operations help
	@echo -e "$(CYAN)📚 SIMPelv2 Operations$(NC)"
	@echo "=============================="
	@echo ""
	@echo -e "$(YELLOW)🔨 Build Operations:$(NC)"
	@echo "  make build-all          - Build all components"
	@echo "  make build-frontend     - Build frontend only"
	@echo "  make build-backend      - Build backend only"
	@echo "  make build-clean        - Clean build artifacts"
	@echo "  make dev-start          - Start development environment"
	@echo ""
	@echo -e "$(BLUE)🧪 Testing Operations:$(NC)"
	@echo "  make test-all           - Run all tests"
	@echo "  make test-performance   - Performance benchmarks"
	@echo "  make test-security      - Security audits"
	@echo "  make test-validation    - Validation tests"
	@echo ""
	@echo -e "$(GREEN)🚀 Deployment Operations:$(NC)"
	@echo "  make deploy-dev         - Deploy to development"
	@echo "  make deploy-staging     - Deploy to staging"
	@echo "  make deploy-prod        - Deploy to production"
	@echo "  make monitoring         - System monitoring"
	@echo "  make backup             - Maintenance backup"
	@echo ""
	@echo -e "$(PURPLE)🛠️  Tool Operations:$(NC)"
	@echo "  make wasm-stats         - WASM statistics"
	@echo "  make wasm-optimize      - Optimize WASM"
	@echo "  make cargo-status       - Cargo status"
	@echo "  make cargo-update       - Update dependencies"
	@echo "  make ai-generate        - AI code generation"
	@echo "  make project-init       - Initialize new component"
	@echo ""
	@echo -e "$(CYAN)📊 Information:$(NC)"
	@echo "  make status             - System status"
	@echo "  make workspace-stats    - Workspace statistics"
	@echo "  make help-simpel        - This help"
