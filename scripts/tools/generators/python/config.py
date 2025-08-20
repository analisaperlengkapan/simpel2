"""
Configuration module for Kubernetes YAML Generator

Contains all constants, paths, and configuration settings.
"""

import os
from dotenv import dotenv_values

# --- Paths ---
SCRIPT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE_DIR = os.path.dirname(SCRIPT_DIR)  # Root project
COMPOSE_FILE = os.path.join(BASE_DIR, "docker-compose.yml")  # Use root compose file
K8S_DIR = os.path.join(BASE_DIR, "infra", "k8s")  # Updated to use infra folder

# --- Constants ---
DEFAULT_NAMESPACE = "simpelv2"
DEFAULT_PORT = 3000
ENVIRONMENTS = ["dev", "staging", "prod"]

# --- Environment Variables ---
ENV = os.environ.get("ENV", "dev").lower()
TAG = os.environ.get("TAG", "latest")

# Handle prod versioning
if ENV == "prod":
    TAG = os.environ.get("VERSION", "latest")

# --- Domain Configuration ---
ROOT_ENV_PATH = os.path.join(BASE_DIR, ".env")
ROOT_ENV = dotenv_values(ROOT_ENV_PATH) if os.path.exists(ROOT_ENV_PATH) else {}
DOMAIN = ROOT_ENV.get(f"DOMAIN_{ENV.upper()}", "simpelv2.local")

# --- Service Specific Configs ---
EXCLUDE_MONITORING_SERVICES = ["db-simpelv2", "nginx", "antarmuka"]
SKIP_VOLUME_SERVICES = ["nginx", "db-simpelv2"]
SKIP_COMMAND_SERVICES = ["nginx", "antarmuka", "db-simpelv2"]

# --- Database Configs ---
DB_PORT = 5432

# --- Monitoring Configs ---
PROMETHEUS_SCRAPE_INTERVAL = "15s"
PROMETHEUS_EVALUATION_INTERVAL = "15s"
PROMETHEUS_SCRAPE_TIMEOUT = "10s"
PROMETHEUS_METRICS_PATH = "/metrics"

# --- Health Check Configs ---
DEFAULT_HEALTH_CHECK_INTERVAL = "30s"
DEFAULT_HEALTH_CHECK_TIMEOUT = "5s"
DEFAULT_HEALTH_CHECK_RETRIES = 3
DEFAULT_HEALTH_PATH = "/health"

# --- Image Policy ---
def get_image_pull_policy():
    """Get image pull policy based on environment"""
    return "Always" if ENV == "prod" else "IfNotPresent"

# --- Validation ---
def validate_environment():
    """Validate current environment"""
    if ENV not in ENVIRONMENTS:
        raise ValueError(f"Environment '{ENV}' tidak valid. Pilih: {ENVIRONMENTS}")
    return True 