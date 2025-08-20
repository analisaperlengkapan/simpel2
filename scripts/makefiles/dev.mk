# ====== SIMPelv2 Development Targets ======
# Development server and workflow targets

# ====== DEVELOPMENT SERVER TARGETS ======
.PHONY: dev dev-all dev-backend dev-frontend dev-stop dev-status dev-logs

dev: dev-all ## Start complete development environment

dev-all: ## Start all development servers
	@echo "🚀 Starting complete development environment..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh fullstack

dev-backend: ## Start only backend development servers
	@echo "🦀 Starting backend development servers..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh backend

dev-frontend: ## Start only frontend development servers
	@echo "🌐 Starting frontend development servers..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh frontend

dev-single: ## Start single microfrontend dev server (usage: make dev-single MF=badiklat)
	@if [ -z "$(MF)" ]; then \
		echo "❌ Please specify microfrontend: make dev-single MF=badiklat"; \
		exit 1; \
	fi
	@echo "🚀 Starting development server for $(MF)..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh single $(MF)

dev-stop: ## Stop all development servers
	@echo "🛑 Stopping development servers..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh stop

dev-status: ## Show development server status
	@echo "📋 Development server status..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh status

dev-logs: ## Show development server logs
	@echo "📄 Development server logs..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh logs

# ====== DOCKER DEVELOPMENT ======
.PHONY: docker-dev docker-dev-build docker-dev-up docker-dev-down docker-dev-logs

docker-dev: docker-dev-up ## Start Docker development environment

docker-dev-build: ## Build Docker development images
	@echo "🐳 Building Docker development images..."
	@docker-compose -f docker-compose.dev.yml build

docker-dev-up: ## Start Docker development containers
	@echo "🐳 Starting Docker development containers..."
	@docker-compose -f docker-compose.dev.yml up -d

docker-dev-down: ## Stop Docker development containers
	@echo "🐳 Stopping Docker development containers..."
	@docker-compose -f docker-compose.dev.yml down

docker-dev-logs: ## Show Docker development logs
	@echo "📄 Docker development logs..."
	@docker-compose -f docker-compose.dev.yml logs -f

# ====== TESTING IN DEVELOPMENT ======
.PHONY: dev-test dev-test-watch dev-test-coverage

dev-test: ## Run tests in development mode
	@echo "🧪 Running development tests..."
	@cargo test --workspace

dev-test-watch: ## Watch and run tests automatically
	@echo "👀 Watching and running tests..."
	@cargo watch -x "test --workspace"

dev-test-coverage: ## Run tests with coverage
	@echo "📊 Running tests with coverage..."
	@cargo tarpaulin --workspace --out Html --output-dir target/coverage

# ====== DEVELOPMENT TOOLS ======
.PHONY: dev-tools dev-setup dev-install dev-update

dev-tools: ## Install development tools
	@echo "🔧 Installing development tools..."
	@cargo install trunk cargo-watch cargo-tarpaulin
	@rustup component add rustfmt clippy

dev-setup: validate-workspace dev-tools ## Complete development environment setup
	@echo "🚀 Setting up development environment..."
	@$(MAKE) deps-update
	@echo "✅ Development environment ready!"

dev-install: ## Install project dependencies
	@echo "📦 Installing project dependencies..."
	@cargo fetch --target x86_64-unknown-linux-gnu

dev-update: ## Update development environment
	@echo "🔄 Updating development environment..."
	@rustup update
	@$(MAKE) dev-tools
	@$(MAKE) deps-update

# ====== DEVELOPMENT WORKFLOW ======
.PHONY: dev-workflow dev-start dev-restart dev-reset

dev-workflow: ## Complete development workflow
	@echo "🚀 Starting complete development workflow..."
	@$(MAKE) clean
	@$(MAKE) build-dev
	@$(MAKE) dev-all

dev-start: dev-setup build-dev dev-all ## Start fresh development session

dev-restart: dev-stop build-dev dev-all ## Restart development environment

dev-reset: clean dev-start ## Reset and start development environment

# ====== HOT RELOAD AND WATCH ======
.PHONY: hot-reload watch-frontend watch-backend

hot-reload: ## Start hot reload for all components
	@echo "🔥 Starting hot reload..."
	@$(SCRIPTS_DIR)/modules/dev-server.sh hot-reload

watch-frontend: ## Watch frontend changes
	@echo "👀 Watching frontend changes..."
	@for dir in antarmuka/*/; do \
		if [[ -f "$$dir/Trunk.toml" ]]; then \
			echo "👀 Starting watch for $$(basename $$dir)..."; \
			(cd "$$dir" && trunk serve --port $$(( $(DEV_SERVER_PORT) + $$(basename $$dir | wc -c) ))) & \
		fi \
	done
	@wait

watch-backend: ## Watch backend changes
	@echo "👀 Watching backend changes..."
	@cargo watch -x "run --bin server"

# ====== DEVELOPMENT UTILITIES ======
.PHONY: dev-clean dev-logs-clear dev-cache-clear

dev-clean: ## Clean development environment
	@echo "🧹 Cleaning development environment..."
	@$(MAKE) dev-stop
	@$(MAKE) clean
	@docker system prune -f 2>/dev/null || true

dev-logs-clear: ## Clear development logs
	@echo "🗑️ Clearing development logs..."
	@find . -name "*.log" -type f -delete 2>/dev/null || true

dev-cache-clear: ## Clear development caches
	@echo "🗑️ Clearing development caches..."
	@cargo clean
	@find . -name "node_modules" -type d -exec rm -rf {} + 2>/dev/null || true
