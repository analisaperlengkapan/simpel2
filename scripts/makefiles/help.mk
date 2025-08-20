# ====== SIMPelv2 Makefile Help System ======
# Comprehensive help and documentation

# ====== MAIN HELP ======
.PHONY: help help-all help-build help-dev help-ops help-advanced

help: ## Show main help menu
	@echo ""
	@echo "🛠️  SIMPelv2 Platform Controller v$(PROJECT_VERSION)"
	@echo "=================================================="
	@echo ""
	@echo "📖 Main Categories:"
	@echo "  make help-build     - Build and compilation targets"
	@echo "  make help-dev       - Development environment targets"
	@echo "  make help-ops       - Operations and deployment targets" 
	@echo "  make help-advanced  - Advanced features and automation"
	@echo "  make help-all       - Show all available targets"
	@echo ""
	@echo "🚀 Quick Commands:"
	@echo "  make build          - Build entire project"
	@echo "  make dev            - Start development environment"
	@echo "  make deploy         - Deploy to development"
	@echo "  make test           - Run all tests"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "💡 Examples:"
	@echo "  make build-mf MF=badiklat    - Build specific microfrontend"
	@echo "  make dev-single MF=intel     - Start single dev server"
	@echo "  make deploy-prod             - Deploy to production"
	@echo "  make ultimate-build          - Ultimate build workflow"
	@echo ""
	@echo "📚 Documentation:"
	@echo "  Use 'make help-<category>' for detailed help on each category"

help-build: ## Show build targets help
	@echo ""
	@echo "🔨 Build Targets"
	@echo "==============="
	@$(call show_help_for_category,build)

help-dev: ## Show development targets help
	@echo ""
	@echo "🚀 Development Targets"
	@echo "====================="
	@$(call show_help_for_category,dev)

help-ops: ## Show operations targets help
	@echo ""
	@echo "⚙️  Operations Targets"
	@echo "====================="
	@$(call show_help_for_category,ops)

help-advanced: ## Show advanced features help
	@echo ""
	@echo "🤖 Advanced Features"
	@echo "===================="
	@$(call show_help_for_category,advanced)

help-all: ## Show all available targets
	@echo ""
	@echo "📋 All Available Targets"
	@echo "========================"
	@echo ""
	@echo "🔨 BUILD TARGETS:"
	@$(call show_help_for_category,build)
	@echo ""
	@echo "🚀 DEVELOPMENT TARGETS:"
	@$(call show_help_for_category,dev)
	@echo ""
	@echo "⚙️  OPERATIONS TARGETS:"
	@$(call show_help_for_category,ops)
	@echo ""
	@echo "🤖 ADVANCED TARGETS:"
	@$(call show_help_for_category,advanced)

# ====== HELP UTILITIES ======
define show_help_for_category
	@grep -h "^[a-zA-Z_-]*:" scripts/makefiles/$(1).mk 2>/dev/null | \
		grep "##" | \
		sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' || \
		echo "  No targets found for category: $(1)"
endef

# ====== TARGET DISCOVERY ======
.PHONY: list-targets list-categories list-workflows

list-targets: ## List all available targets
	@echo "📋 Available Targets:"
	@$(MAKE) -qp | awk -F':' '/^[a-zA-Z0-9][^$$#\/\t=]*:([^=]|$$)/ {split($$1,A,/ /); for(i in A)print A[i]}' | sort -u

list-categories: ## List target categories
	@echo "📂 Target Categories:"
	@echo "  • build     - Building and compilation"
	@echo "  • dev       - Development environment"
	@echo "  • ops       - Operations and deployment"
	@echo "  • advanced  - Advanced features and AI"

list-workflows: ## List available workflows
	@echo "🔄 Available Workflows:"
	@echo "  • workflow-build     - Complete build workflow"
	@echo "  • workflow-deploy    - Complete deployment workflow"
	@echo "  • workflow-test      - Complete testing workflow"
	@echo "  • workflow-release   - Complete release workflow"
	@echo "  • ultimate-build     - Ultimate build with optimization"
	@echo "  • ultimate-dev       - Ultimate development environment"
	@echo "  • ultimate-qa        - Ultimate quality assurance"
	@echo "  • ultimate-deploy    - Ultimate deployment workflow"

# ====== INTERACTIVE HELP ======
.PHONY: help-interactive menu

help-interactive: menu ## Interactive help menu

menu: ## Show interactive menu
	@echo ""
	@echo "🎯 SIMPelv2 Interactive Menu"
	@echo "============================"
	@echo ""
	@echo "Choose a category:"
	@echo "  1) 🔨 Build targets"
	@echo "  2) 🚀 Development targets"  
	@echo "  3) ⚙️  Operations targets"
	@echo "  4) 🤖 Advanced features"
	@echo "  5) 📋 List all targets"
	@echo "  6) 🚪 Exit"
	@echo ""
	@read -p "Enter your choice (1-6): " choice; \
	case $$choice in \
		1) $(MAKE) help-build ;; \
		2) $(MAKE) help-dev ;; \
		3) $(MAKE) help-ops ;; \
		4) $(MAKE) help-advanced ;; \
		5) $(MAKE) help-all ;; \
		6) echo "👋 Goodbye!" ;; \
		*) echo "❌ Invalid choice. Please enter 1-6." ;; \
	esac

# ====== QUICK REFERENCE ======
.PHONY: quick-ref examples troubleshooting

quick-ref: ## Quick reference guide
	@echo ""
	@echo "⚡ Quick Reference"
	@echo "================="
	@echo ""
	@echo "🔥 Most Used Commands:"
	@echo "  make build          # Build everything"
	@echo "  make dev            # Start dev environment"
	@echo "  make test           # Run tests"
	@echo "  make clean          # Clean artifacts"
	@echo "  make deploy         # Deploy to dev"
	@echo ""
	@echo "🎯 Specific Tasks:"
	@echo "  make build-mf MF=<name>      # Build single microfrontend"
	@echo "  make dev-single MF=<name>    # Start single dev server"
	@echo "  make deploy-prod             # Deploy to production"
	@echo "  make security-scan           # Security vulnerability scan"
	@echo ""
	@echo "🔧 Maintenance:"
	@echo "  make deep-clean             # Deep clean workspace"
	@echo "  make optimize-workspace     # Optimize everything"
	@echo "  make full-update           # Update all components"

examples: ## Show usage examples
	@echo ""
	@echo "💡 Usage Examples"
	@echo "=================="
	@echo ""
	@echo "🚀 Development Workflow:"
	@echo "  make dev-setup              # First time setup"
	@echo "  make build-dev              # Build for development"
	@echo "  make dev-all                # Start all dev servers"
	@echo "  make dev-test-watch         # Watch and run tests"
	@echo ""
	@echo "🏗️  Production Workflow:"
	@echo "  make ultimate-qa            # Quality assurance"
	@echo "  make build-release          # Production build"
	@echo "  make docker-build           # Build Docker images"
	@echo "  make deploy-prod            # Deploy to production"
	@echo ""
	@echo "🔍 Debugging & Analysis:"
	@echo "  make health-check           # System health check"
	@echo "  make security-audit         # Security audit"
	@echo "  make perf-benchmark         # Performance benchmark"
	@echo "  make ai-analyze             # AI-powered analysis"

troubleshooting: ## Show troubleshooting guide
	@echo ""
	@echo "🔧 Troubleshooting Guide"
	@echo "======================="
	@echo ""
	@echo "❌ Common Issues:"
	@echo ""
	@echo "Build failures:"
	@echo "  • Run 'make clean' then 'make build'"
	@echo "  • Check 'make validate-workspace'"
	@echo "  • Update deps with 'make deps-update'"
	@echo ""
	@echo "Dev server issues:"
	@echo "  • Stop with 'make dev-stop'"
	@echo "  • Clear cache 'make dev-cache-clear'"
	@echo "  • Restart with 'make dev-restart'"
	@echo ""
	@echo "Docker problems:"
	@echo "  • Clean Docker with 'make dev-clean'"
	@echo "  • Rebuild with 'make docker-build'"
	@echo "  • Check logs with 'make docker-logs'"
	@echo ""
	@echo "Performance issues:"
	@echo "  • Run 'make optimize-workspace'"
	@echo "  • Check 'make perf-benchmark'"
	@echo "  • Use 'make deep-clean'"

# ====== STATUS AND INFO ======
.PHONY: show-config show-paths show-env

show-config: ## Show current configuration
	@echo ""
	@echo "⚙️  Current Configuration"
	@echo "========================"
	@echo "Project: $(PROJECT_NAME) v$(PROJECT_VERSION)"
	@echo "Environment: $(ENV)"
	@echo "Build Date: $(BUILD_DATE)"
	@echo "Workspace: $(WORKSPACE_ROOT)"
	@echo "Parallel Build: $(PARALLEL_BUILD)"
	@echo "Hot Reload: $(HOT_RELOAD)"
	@echo "Docker Registry: $(DOCKER_REGISTRY)"
	@echo "Compose File: $(COMPOSE_FILE)"

show-paths: ## Show important paths
	@echo ""
	@echo "📁 Important Paths"
	@echo "=================="
	@echo "Workspace Root: $(WORKSPACE_ROOT)"
	@echo "Scripts Dir: $(SCRIPTS_DIR)"
	@echo "Modules Dir: $(MODULES_DIR)"
	@echo "Build Dir: $(BUILD_DIR)"
	@echo "Target Dir: $(TARGET_DIR)"
	@echo "Backup Location: $(BACKUP_LOCATION)"

show-env: ## Show environment variables
	@echo ""
	@echo "🌍 Environment Variables"
	@echo "======================="
	@env | grep -E "(RUST_|CARGO_|PROJECT_)" | sort || echo "No relevant environment variables found"
