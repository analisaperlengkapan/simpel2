.PHONY: build up down restart shell-composer shell-php shell-nginx shell-mysql shell-redis logs logs-nginx logs-php logs-mysql logs-redis test setup prune artisan composer migrate-only npm help

# Project settings
COMPOSE := docker-compose
DOCKER_COMPOSE_FILE := docker-compose.yml
ENV_FILE := .env

# Load .env variables if file exists
ifneq (,$(wildcard $(ENV_FILE)))
	include $(ENV_FILE)
	export
endif

# Default target
all: help

## Help
help:
	@echo ""
	@echo "Usage: make <command>"
	@echo ""
	@echo "Available commands:"
	@echo "  build            Build or rebuild Docker services"
	@echo "  up               Start Docker containers in background"
	@echo "  down             Stop and remove containers and networks"
	@echo "  restart          Restart all services (down + up)"
	@echo "  logs             Tail all logs"
	@echo "  logs-php         Tail PHP container logs"
	@echo "  logs-nginx       Tail Nginx container logs"
	@echo "  logs-mysql       Tail MySQL container logs"
	@echo "  logs-redis       Tail Redis container logs"
	@echo "  shell-php        Open PHP container shell (bash)"
	@echo "  shell-nginx      Open Nginx container shell (sh)"
	@echo "  shell-mysql      Open MySQL CLI shell"
	@echo "  shell-redis      Open Redis CLI shell"
	@echo "  shell-composer   Run Composer with arguments"
	@echo "  artisan          Run Laravel artisan command"
	@echo "  composer         Run Composer command"
	@echo "  npm              Run npm inside PHP container"
	@echo "  migrate-only     Run database migrations only"
	@echo "  test             Run PHPUnit tests"
	@echo "  setup            Full Laravel project setup"
	@echo "  prune            Remove unused Docker volumes"
	@echo ""

## Docker
build:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) build

up:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) up -d

down:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) down

restart: down up

## Logs
logs:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) logs -f

logs-php:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) logs -f app

logs-nginx:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) logs -f nginx

logs-mysql:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) logs -f mysql

logs-redis:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) logs -f redis

## Shell access
shell-php:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec app bash

shell-nginx:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec nginx sh

shell-mysql:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec mysql bash -c 'mysql -u$(DB_USERNAME) -p$(DB_PASSWORD) $(DB_DATABASE)'

shell-redis:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec redis redis-cli

shell-composer: composer

## Composer, Artisan, NPM
composer:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec app composer $(filter-out $@,$(MAKECMDGOALS))

artisan:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec app php artisan $(filter-out $@,$(MAKECMDGOALS))

npm:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec app npm $(filter-out $@,$(MAKECMDGOALS))

## Migrate only
migrate-only:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec -T app php artisan migrate --force

## Tests
test:
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec -T app ./vendor/bin/phpunit

## Initial Setup
setup: build up
	@echo "Installing Composer dependencies..."
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec -T app composer install --no-interaction --prefer-dist --optimize-autoloader

	@echo "Generating application key..."
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec -T app php artisan key:generate

	@echo "Running migrations..."
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec -T app php artisan migrate --seed --force

	@echo "Linking storage..."
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec -T app php artisan storage:link

	@echo "Optimizing..."
	$(COMPOSE) -f $(DOCKER_COMPOSE_FILE) exec -T app php artisan optimize

	@echo "Setup complete!"
	@echo "Access the application at http://localhost:8000"

## Prune unused Docker volumes
prune:
	docker volume prune -f

# Allow commands with args
%:
	@:
