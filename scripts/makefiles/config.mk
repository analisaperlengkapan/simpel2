# ====== SIMPelv2 Configuration ======
# Project configuration with sensible defaults
# Override these values by creating config.local.mk

# ====== PROJECT INFO ======
PROJECT_NAME ?= simpelv2
PROJECT_VERSION ?= 2.1.0
BUILD_DATE ?= $(shell date '+%Y.%m.%d')

# ====== ENVIRONMENT CONFIGURATION ======
ENV ?= dev
TAG ?= latest
VERSION ?= v$(PROJECT_VERSION)

# ====== PATHS AND DIRECTORIES ======
WORKSPACE_ROOT ?= /var/www/simpelv2
SCRIPTS_DIR ?= $(WORKSPACE_ROOT)/scripts
MODULES_DIR ?= $(SCRIPTS_DIR)/modules

# New organized structure
TOOLS_DIR ?= $(SCRIPTS_DIR)/tools
TEST_DIR ?= $(SCRIPTS_DIR)/test

# Legacy compatibility
LEGACY_MODULES_DIR ?= $(MODULES_DIR)

BUILD_DIR ?= .build
DIST_DIR ?= dist
TARGET_DIR ?= target

# ====== SERVICE CONFIGURATION ======
MICROFRONTENDS ?= $(shell find antarmuka -maxdepth 1 -type d -name '*' ! -path 'antarmuka' | xargs -n1 basename)
MICROSERVICES ?= $(shell find layanan -maxdepth 1 -type d -name '*' ! -path 'layanan' | xargs -n1 basename)

# ====== BUILD CONFIGURATION ======
RUST_LOG ?= info
CARGO_BUILD_JOBS ?= $(shell nproc)
PARALLEL_BUILD ?= true
BUILD_CACHE ?= true
OPTIMIZE_IMAGES ?= true

# ====== DOCKER CONFIGURATION ======
COMPOSE_FILE ?= docker-compose.yml
COMPOSE_PROJECT_NAME ?= $(PROJECT_NAME)
DOCKER_REGISTRY ?= $(PROJECT_NAME)
DOCKER_TAG ?= $(TAG)

# ====== DEVELOPMENT CONFIGURATION ======
HOT_RELOAD ?= true
DEBUG_MODE ?= false
VERBOSE_LOGGING ?= false
DEV_SERVER_PORT ?= 3000

# ====== MONITORING CONFIGURATION ======
ENABLE_MONITORING ?= true
METRICS_ENABLED ?= true
HEALTH_CHECK_ENABLED ?= true

# ====== SECURITY CONFIGURATION ======
ENABLE_SECURITY_SCAN ?= true
ENABLE_AUDIT_LOGGING ?= true

# ====== PERFORMANCE CONFIGURATION ======
ENABLE_PROFILING ?= false
BENCHMARK_ENABLED ?= false
PERFORMANCE_TRACKING ?= true

# ====== LOGGING CONFIGURATION ======
LOG_LEVEL ?= INFO
LOG_FORMAT ?= json
LOG_RETENTION_DAYS ?= 7

# ====== RESOURCE LIMITS ======
CPU_LIMIT ?= 1000m
MEMORY_LIMIT ?= 1Gi
STORAGE_LIMIT ?= 10Gi

# ====== FEATURE FLAGS ======
ENABLE_AI_TOOLS ?= true
ENABLE_EDGE_COMPUTING ?= false
ENABLE_ADVANCED_MONITORING ?= true

# ====== NOTIFICATION CONFIGURATION ======
SLACK_WEBHOOK ?= 
EMAIL_NOTIFICATIONS ?= false
NOTIFICATION_ENABLED ?= false

# ====== BACKUP CONFIGURATION ======
BACKUP_ENABLED ?= false
BACKUP_RETENTION_DAYS ?= 30
BACKUP_LOCATION ?= ./backups

# ====== CUSTOM SCRIPTS ======
# Override these to point to your custom scripts
PRE_BUILD_SCRIPT ?= scripts/hooks/pre_build.sh
POST_BUILD_SCRIPT ?= scripts/hooks/post_build.sh
PRE_DEPLOY_SCRIPT ?= scripts/hooks/pre_deploy.sh
POST_DEPLOY_SCRIPT ?= scripts/hooks/post_deploy.sh
HEALTH_CHECK_SCRIPT ?= scripts/hooks/health_check.sh

# ====== CONDITIONAL INCLUDES ======
# Include local configuration if it exists
-include scripts/makefiles/config.local.mk
