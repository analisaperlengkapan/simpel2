SHELL := /bin/bash

DOCKER_DEV := docker-compose.dev.yml
DOCKER_PROD := docker-compose.prod.yml

# ----------------------
# Cek dan Setup Docker
# ----------------------
check-docker:
	@echo "🔍 Mengecek Docker Compose V2..."
	@if ! docker compose version > /dev/null 2>&1; then \
		echo "⚠️  Docker Compose V2 tidak ditemukan. Menjalankan setup-docker.sh..."; \
		./scripts/setup-docker.sh; \
	else \
		echo "✅ Docker Compose V2 tersedia."; \
	fi

# ----------------------
# Development Mode
# ----------------------
dev: check-docker
	@echo "🚀 Menjalankan SIMPELv2 development mode..."
	docker compose -f $(DOCKER_DEV) up --build

stop-dev:
	@echo "🛑 Menghentikan container dev..."
	docker compose -f $(DOCKER_DEV) down

logs-dev:
	docker compose -f $(DOCKER_DEV) logs -f

# ----------------------
# Production Mode
# ----------------------
build:
	@echo "🔧 Membuild image dan frontend (npm run build)..."
	docker compose -f $(DOCKER_PROD) build
	docker compose -f $(DOCKER_PROD) run --rm frontend npm run build

prod: check-docker
	@echo "🚀 Menjalankan SIMPELv2 production mode..."
	docker compose -f $(DOCKER_PROD) up -d --build

stop-prod:
	@echo "🛑 Menghentikan container prod..."
	docker compose -f $(DOCKER_PROD) down

logs-prod:
	docker compose -f $(DOCKER_PROD) logs -f

# ----------------------
# Rilis Sekaligus (Build + Prod)
# ----------------------
release: build prod

# ----------------------
# Stop Semua
# ----------------------
stop-all:
	@echo "🛑 Menghentikan semua container..."
	-docker compose -f $(DOCKER_DEV) down
	-docker compose -f $(DOCKER_PROD) down
