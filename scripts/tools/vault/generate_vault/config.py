"""
Configuration module for Vault setup
Contains all constants, environment variables, and configuration settings
"""

import os
from pathlib import Path

# Base paths
BASE_DIR = Path(__file__).resolve().parents[3]
VAULT_DIR = BASE_DIR / "infra" / "vault"
K8S_DIR = BASE_DIR / "infra" / "vault" / "k8s"  # Isolated Vault K8s manifests
SCRIPT_PATH = Path(__file__).resolve().parents[2] / "setup_vault.py"

# Environment-based configuration
NAMESPACE = os.getenv("VAULT_NAMESPACE", "simpelv2")
SECRET_NAME = os.getenv("VAULT_SECRET_NAME", "vault-fernet-key")
MAX_RETRIES = int(os.getenv("VAULT_MAX_RETRIES", "5"))
RETRY_DELAY = int(os.getenv("VAULT_RETRY_DELAY", "5"))
VAULT_PORT = int(os.getenv("VAULT_PORT", "8200"))
KUBERNETES_CONTEXT = os.getenv("KUBERNETES_CONTEXT", "microk8s")

# Vault configuration
VAULT_IMAGE = "hashicorp/vault:1.15"
VAULT_UI_ENABLED = True
VAULT_DISABLE_MLOCK = True

# TLS configuration
TLS_MIN_VERSION = "tls12"
TLS_CIPHER_SUITES = "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"

# Certificate configuration
CERT_ORGANIZATION = "SimpelV2"
CERT_COUNTRY = "ID"
CERT_STATE = "Jakarta"
CERT_LOCALITY = "Jakarta"
CERT_COMMON_NAME = "vault.simpelv2.local"
CERT_VALIDITY_DAYS = 365

# Kubernetes configuration
K8S_RESOURCE_LIMITS = {
    "memory": "1Gi",
    "cpu": "500m"
}
K8S_RESOURCE_REQUESTS = {
    "memory": "512Mi",
    "cpu": "250m"
}

# Security configuration
SECURITY_USER = 1000
SECURITY_GROUP = 1000
SECURITY_FS_GROUP = 1000

# File permissions
KEY_FILE_PERMISSIONS = 0o600
CERT_FILE_PERMISSIONS = 0o644

# Directory structure
DIRECTORIES = [
    "keys",
    "config", 
    "certs",
    "backup",
    "audit"
]

# Kubernetes manifests to generate
K8S_MANIFESTS = [
    'pvc.yaml',
    'deployment.yaml',
    'service.yaml',
    'vault-env-configmap.yaml',
    'configmap.yaml',
    'network-policy.yaml',
    'tls-secret.yaml'
]

# Files to clean up
CLEANUP_FILES = [
    'pvc.yaml',
    'deployment.yaml',
    'service.yaml',
    'vault-env-configmap.yaml',
    'configmap.yaml',
    'fernet-sealedsecret.yaml',
    'kustomization.yaml',
    'network-policy.yaml',
    'pod-security.yaml',
    'tls-secret.yaml'
] 