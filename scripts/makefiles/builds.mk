# ====== SIMPelv2 Unified Build Operations ======
# Merged from build.mk and build-core.mk for comprehensive build system
# Version: 4.1.0 - Consolidated & Optimized

# ====== CONFIGURATION ======
SCRIPTS_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
WORKSPACE_ROOT := $(shell cd $(SCRIPTS_DIR)/../.. && pwd)

# Working services (excluding problematic ones)
BACKEND_SERVICES := layanan/keamanan layanan/konfigurasi layanan/laporan layanan/ai antarmuka/shared
FRONTEND_SERVICES := antarmuka/portal antarmuka/badiklat antarmuka/datun antarmuka/intel

# Colors
GREEN := \033[0;32m
BLUE := \033[0;34m
CYAN := \033[0;36m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

# ====== PRIMARY BUILD TARGETS ======
.PHONY: build build-all build-parallel build-release

build: build-parallel ## Build all components (default)
	@echo -e "$(GREEN)✅ All components built successfully$(NC)"

build-all: build-parallel ## Build all components (alias)

build-parallel: rust-backend leptos-frontend ## Build all components in parallel
	@echo -e "$(GREEN)🎉 Parallel build completed$(NC)"

build-release: rust-backend-release leptos-frontend ## Build all components (release mode)
	@echo -e "$(GREEN)🚀 Release build completed$(NC)"

# ====== BACKEND BUILD TARGETS ======
.PHONY: rust-backend rust-backend-release rust-service-%

rust-backend: ## Build all Rust backend services (debug)
	@echo -e "$(CYAN)🦀 Building Rust backend services (debug)...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	for service in $(BACKEND_SERVICES); do \
		if [[ -f "$$service/Cargo.toml" ]]; then \
			echo -e "$(BLUE)Building $$service...$(NC)"; \
			cargo build --manifest-path "$$service/Cargo.toml"; \
		fi; \
	done
	@echo -e "$(GREEN)✅ Backend build completed$(NC)"

rust-backend-release: ## Build all Rust backend services (release)
	@echo -e "$(CYAN)🦀 Building Rust backend services (release)...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	for service in $(BACKEND_SERVICES); do \
		if [[ -f "$$service/Cargo.toml" ]]; then \
			echo -e "$(BLUE)Building $$service (release)...$(NC)"; \
			cargo build --manifest-path "$$service/Cargo.toml" --release; \
		fi; \
	done
	@echo -e "$(GREEN)✅ Backend release build completed$(NC)"

rust-service-%: ## Build specific Rust service (e.g., rust-service-keamanan)
	@echo -e "$(CYAN)🦀 Building layanan/$* service...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	if [[ -f "layanan/$*/Cargo.toml" ]]; then \
		cargo build --manifest-path "layanan/$*/Cargo.toml" --release; \
		echo -e "$(GREEN)✅ Service layanan/$* built successfully$(NC)"; \
	else \
		echo -e "$(RED)❌ Service layanan/$* not found$(NC)"; \
		exit 1; \
	fi

# ====== FRONTEND BUILD TARGETS ======
.PHONY: leptos-frontend leptos-shared leptos-microfrontend-%

leptos-shared: ## Build shared Leptos components first
	@echo -e "$(BLUE)🎨 Building shared Leptos components...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	if [[ -f "antarmuka/shared/Cargo.toml" ]]; then \
		cargo build --manifest-path "antarmuka/shared/Cargo.toml" --target wasm32-unknown-unknown --release; \
		echo -e "$(GREEN)✅ Shared components built$(NC)"; \
	fi

leptos-frontend: leptos-shared ## Build all Leptos microfrontends
	@echo -e "$(BLUE)🎨 Building Leptos microfrontends...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	for frontend in $(FRONTEND_SERVICES); do \
		if [[ -f "$$frontend/Cargo.toml" ]] && [[ -f "$$frontend/Trunk.toml" ]]; then \
			echo -e "$(CYAN)Building $$frontend...$(NC)"; \
			cd "$$frontend" && trunk build --release && cd "$(WORKSPACE_ROOT)"; \
		fi; \
	done
	@echo -e "$(GREEN)✅ Frontend build completed$(NC)"

leptos-microfrontend-%: leptos-shared ## Build specific microfrontend (e.g., leptos-microfrontend-portal)
	@echo -e "$(BLUE)🎨 Building antarmuka/$* microfrontend...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	if [[ -f "antarmuka/$*/Cargo.toml" ]] && [[ -f "antarmuka/$*/Trunk.toml" ]]; then \
		cd "antarmuka/$*" && trunk build --release; \
		echo -e "$(GREEN)✅ Microfrontend antarmuka/$* built successfully$(NC)"; \
	else \
		echo -e "$(RED)❌ Microfrontend antarmuka/$* not found or missing Trunk.toml$(NC)"; \
		exit 1; \
	fi

# ====== DOCUMENTATION BUILD ======
.PHONY: build-docs docs-api docs-book

build-docs: docs-api docs-book ## Build all documentation

docs-api: ## Generate API documentation
	@echo -e "$(BLUE)📚 Generating API documentation...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo doc --no-deps --open

docs-book: ## Build documentation book
	@echo -e "$(CYAN)📖 Building documentation book...$(NC)"
	@if command -v mdbook >/dev/null 2>&1; then \
		cd docs && mdbook build; \
	else \
		echo -e "$(YELLOW)⚠️  mdbook not installed, skipping book generation$(NC)"; \
	fi

# ====== OPTIMIZATION BUILDS ======
.PHONY: build-wasm-opt build-size-opt

build-wasm-opt: leptos-frontend ## Build with WASM optimization
	@echo -e "$(PURPLE)⚡ Optimizing WASM bundles...$(NC)"
	@if [[ -f "$(SCRIPTS_DIR)/../core/wasm-optimizer.sh" ]]; then \
		$(SCRIPTS_DIR)/../core/wasm-optimizer.sh optimize; \
	fi

build-size-opt: ## Build with size optimization
	@echo -e "$(YELLOW)📦 Size-optimized build...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	cargo build --release --profile minsize

# ====== CLEAN TARGETS ======
.PHONY: clean clean-all clean-cargo clean-trunk clean-docs

clean: clean-cargo clean-trunk ## Standard clean

clean-all: clean clean-docs ## Deep clean including docs
	@echo -e "$(GREEN)✅ Complete cleanup finished$(NC)"

clean-cargo: ## Clean Cargo build artifacts
	@echo -e "$(YELLOW)🧹 Cleaning Cargo artifacts...$(NC)"
	@cd $(WORKSPACE_ROOT) && cargo clean

clean-trunk: ## Clean Trunk build artifacts
	@echo -e "$(YELLOW)🧹 Cleaning Trunk artifacts...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	find . -name "dist" -type d -exec rm -rf {} + 2>/dev/null || true; \
	find . -name "pkg" -type d -exec rm -rf {} + 2>/dev/null || true

clean-docs: ## Clean documentation artifacts
	@echo -e "$(YELLOW)🧹 Cleaning documentation...$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	rm -rf target/doc 2>/dev/null || true; \
	rm -rf docs/book 2>/dev/null || true

# ====== MAINTENANCE & CHECK TARGETS ======
.PHONY: check-services check-rust-services check-frontend-services build-status

check-services: check-rust-services check-frontend-services ## Check all services

check-rust-services: ## Check which Rust services are available
	@echo -e "$(CYAN)📋 Available Rust Services:$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	for service in $(BACKEND_SERVICES); do \
		if [[ -f "$$service/Cargo.toml" ]]; then \
			echo -e "  $(GREEN)✅ $$service$(NC)"; \
		else \
			echo -e "  $(RED)❌ $$service$(NC)"; \
		fi; \
	done

check-frontend-services: ## Check which frontend services are available
	@echo -e "$(BLUE)📋 Available Frontend Services:$(NC)"
	@cd $(WORKSPACE_ROOT) && \
	for frontend in $(FRONTEND_SERVICES); do \
		if [[ -f "$$frontend/Cargo.toml" ]] && [[ -f "$$frontend/Trunk.toml" ]]; then \
			echo -e "  $(GREEN)✅ $$frontend$(NC)"; \
		else \
			echo -e "  $(RED)❌ $$frontend (missing Cargo.toml or Trunk.toml)$(NC)"; \
		fi; \
	done

build-status: ## Show build system status
	@echo -e "$(CYAN)🔧 Build System Status$(NC)"
	@echo "=============================="
	@echo "Workspace: $(WORKSPACE_ROOT)"
	@echo "Scripts: $(SCRIPTS_DIR)"
	@echo ""
	@echo -e "$(GREEN)🔍 Tool Check:$(NC)"
	@command -v cargo >/dev/null 2>&1 && echo "  ✅ Cargo available" || echo "  ❌ Cargo not found"
	@command -v trunk >/dev/null 2>&1 && echo "  ✅ Trunk available" || echo "  ❌ Trunk not found"
	@command -v wasm-pack >/dev/null 2>&1 && echo "  ✅ wasm-pack available" || echo "  ⚠️  wasm-pack optional"

# ====== HELP TARGET ======
help-builds: ## Show build operations help
	@echo -e "$(CYAN)🔨 Build Operations$(NC)"
	@echo "===================="
	@echo ""
	@echo -e "$(YELLOW)🎯 Primary Builds:$(NC)"
	@echo "  make build                  - Build all components (default)"
	@echo "  make build-all              - Build all components (alias)"
	@echo "  make build-parallel         - Parallel build"
	@echo "  make build-release          - Release build"
	@echo ""
	@echo -e "$(CYAN)🦀 Backend (Rust):$(NC)"
	@echo "  make rust-backend           - All backend services (debug)"
	@echo "  make rust-backend-release   - All backend services (release)"
	@echo "  make rust-service-<name>    - Specific service"
	@echo ""
	@echo -e "$(BLUE)🎨 Frontend (Leptos):$(NC)"
	@echo "  make leptos-shared          - Shared components"
	@echo "  make leptos-frontend        - All microfrontends"
	@echo "  make leptos-microfrontend-<name> - Specific microfrontend"
	@echo ""
	@echo -e "$(PURPLE)📚 Documentation:$(NC)"
	@echo "  make build-docs             - All documentation"
	@echo "  make docs-api               - API documentation"
	@echo "  make docs-book              - Documentation book"
	@echo ""
	@echo -e "$(YELLOW)⚡ Optimization:$(NC)"
	@echo "  make build-wasm-opt         - WASM optimized build"
	@echo "  make build-size-opt         - Size optimized build"
	@echo ""
	@echo -e "$(RED)🧹 Cleanup:$(NC)"
	@echo "  make clean                  - Standard cleanup"
	@echo "  make clean-all              - Deep cleanup"
	@echo "  make clean-cargo            - Cargo artifacts"
	@echo "  make clean-trunk            - Trunk artifacts"
	@echo "  make clean-docs             - Documentation"
	@echo ""
	@echo -e "$(GREEN)📋 Maintenance:$(NC)"
	@echo "  make check-services         - Check all services"
	@echo "  make check-rust-services    - Check Rust services"
	@echo "  make check-frontend-services - Check frontend services"
	@echo "  make build-status           - Build system status"
