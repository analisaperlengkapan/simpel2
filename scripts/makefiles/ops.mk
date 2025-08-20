# ====== SIMPelv2 Updated Operations Targets ======
# Deployment and operations targets adapted to new structure

# ====== DEPLOYMENT TARGETS ======
.PHONY: deploy deploy-all deploy-dev deploy-staging deploy-prod deploy-status deploy-rollback

deploy: deploy-dev ## Deploy to development environment (default)

deploy-all: ## Deploy all services
	@echo "🚀 Deploying all services..."
	@$(TOOLS_DIR)/deploy/deploy-all.sh




deploy-status: ## Check deployment status
	@echo "📋 Checking deployment status..."
	@$(TOOLS_DIR)/deploy/status-check.sh

deploy-rollback: ## Rollback last deployment
	@echo "⏪ Rolling back deployment..."
	@$(TOOLS_DIR)/deploy/rollback.sh

# ====== TESTING OPERATIONS ======
.PHONY: test test-all test-security test-performance test-integration test-unit

test: test-all ## Run all tests





test-unit: ## Run unit tests
	@echo "🧪 Running unit tests..."
	@$(TEST_DIR)/unit/unit-test.sh

# ====== MONITORING AND HEALTH ======
.PHONY: health-check monitor-system validate-env validate-config


monitor-system: ## Monitor system metrics
	@echo "📈 Monitoring system..."
	@$(TEST_DIR)/monitoring/system-monitor.sh

validate-env: ## Validate environment configuration
	@echo "✅ Validating environment..."
	@$(TEST_DIR)/validation/validate-env.py

validate-config: ## Validate system configuration
	@echo "⚙️  Validating configuration..."
	@$(TEST_DIR)/validation/config-validator.sh

# ====== DOCKER OPERATIONS ======
.PHONY: docker-build docker-push docker-pull docker-deploy docker-ps docker-logs docker-clean

docker-build: ## Build Docker images
	@echo "🐳 Building Docker images..."
	@docker-compose build

docker-push: ## Push Docker images to registry
	@echo "📤 Pushing Docker images..."
	@docker-compose push

docker-pull: ## Pull Docker images from registry
	@echo "📥 Pulling Docker images..."
	@docker-compose pull

docker-deploy: docker-build ## Build and deploy Docker containers
	@echo "🐳 Deploying Docker containers..."
	@docker-compose up -d

docker-ps: ## Show running Docker containers
	@echo "📋 Docker container status..."
	@docker-compose ps

docker-logs: ## Show Docker container logs
	@echo "📜 Docker container logs..."
	@docker-compose logs -f

docker-clean: ## Clean Docker images and containers
	@echo "🧹 Cleaning Docker resources..."
	@docker system prune -f

# ====== VAULT OPERATIONS ======
.PHONY: vault-init vault-unseal vault-status vault-config vault-backup

vault-init: ## Initialize Vault
	@echo "🔐 Initializing Vault..."
	@$(TOOLS_DIR)/vault/vault-init.sh

vault-unseal: ## Unseal Vault
	@echo "🔓 Unsealing Vault..."
	@$(TOOLS_DIR)/vault/vault-unseal.sh

vault-status: ## Check Vault status
	@echo "📊 Checking Vault status..."
	@$(TOOLS_DIR)/vault/vault-status.sh

vault-config: ## Configure Vault policies
	@echo "⚙️  Configuring Vault..."
	@$(TOOLS_DIR)/vault/vault-config.sh

vault-backup: ## Backup Vault data
	@echo "💾 Backing up Vault..."
	@$(TOOLS_DIR)/vault/vault-backup.sh

# ====== AI OPERATIONS ======
.PHONY: ai-analysis ai-optimize ai-generate ai-review

ai-analysis: ## Run AI-powered analysis
	@echo "🧠 Running AI analysis..."
	@$(TOOLS_DIR)/ai/ai-tools.sh analyze




# ====== GENERATORS ======
.PHONY: generate-cicd generate-docs generate-api generate-config

generate-cicd: ## Generate CI/CD configurations
	@echo "🤖 Generating CI/CD configurations..."
	@$(TOOLS_DIR)/generators/cicd-generator.sh

generate-docs: ## Generate documentation
	@echo "📚 Generating documentation..."
	@$(TOOLS_DIR)/generators/docs-generator.sh

generate-api: ## Generate API specifications
	@echo "📋 Generating API specs..."
	@$(TOOLS_DIR)/generators/api-generator.sh

generate-config: ## Generate configuration files
	@echo "⚙️  Generating configurations..."
	@$(TOOLS_DIR)/generators/config-generator.sh

# ====== LEGACY COMPATIBILITY ======
# Support old module paths during transition
.PHONY: legacy-deploy legacy-test legacy-monitor

legacy-deploy: ## Legacy deploy (uses old module paths)
	@echo "⚠️  Using legacy deploy path..."
	@if [ -f "$(MODULES_DIR)/deploy.sh" ]; then \
		$(MODULES_DIR)/deploy.sh dev; \
	else \
		echo "❌ Legacy deploy script not found, using new structure"; \
		$(TOOLS_DIR)/deploy/deploy-all.sh dev; \
	fi

legacy-test: ## Legacy test (uses old module paths)
	@echo "⚠️  Using legacy test path..."
	@if [ -f "$(MODULES_DIR)/test.sh" ]; then \
		$(MODULES_DIR)/test.sh; \
	else \
		echo "❌ Legacy test script not found, using new structure"; \
		$(WORKSPACE_ROOT)/scripts/test-suite.sh all; \
	fi

legacy-monitor: ## Legacy monitor (uses old module paths)
	@echo "⚠️  Using legacy monitor path..."
	@if [ -f "$(MODULES_DIR)/monitor.sh" ]; then \
		$(MODULES_DIR)/monitor.sh; \
	else \
		echo "❌ Legacy monitor script not found, using new structure"; \
		$(TEST_DIR)/monitoring/health-check.sh; \
	fi
