"""
Utilities module for Vault setup
Common functions, validation, and helper utilities
"""

import os
import sys
import subprocess
from pathlib import Path
from .config import VAULT_DIR

def check_prerequisites():
    """Check if all prerequisites are installed"""
    print("🔍 Checking prerequisites...")
    
    # Add common binary paths
    os.environ["PATH"] += os.pathsep + "/snap/bin" + os.pathsep + "/usr/local/bin"

    # Check kubeseal
    if subprocess.run(["which", "kubeseal"], capture_output=True).returncode != 0:
        print("❌ kubeseal tidak ditemukan. Harap install SealedSecrets CLI")
        return False
        
    # Check vault CLI
    if subprocess.run(["which", "vault"], capture_output=True).returncode != 0:
        print("❌ vault CLI tidak ditemukan. Harap install HashiCorp Vault CLI")
        return False
    
    print("✅ All prerequisites are available")
    return True

def check_permissions():
    """Check directory permissions"""
    try:
        if not os.access(os.path.dirname(VAULT_DIR), os.W_OK):
            print(f"❌ Tidak bisa menulis ke direktori: {os.path.dirname(VAULT_DIR)}")
            print("💡 Jalankan perintah ini sekali untuk memperbaiki permission:")
            print(f"   sudo chown -R $(whoami):$(id -gn) {VAULT_DIR}")
            return False
        return True
    except Exception as e:
        print(f"❌ Error saat cek akses folder Vault: {e}")
        return False

def validate_environment():
    """Validate environment configuration"""
    print("🔍 Validating environment configuration...")
    
    from .config import (
        NAMESPACE, VAULT_PORT, KUBERNETES_CONTEXT,
        MAX_RETRIES, RETRY_DELAY
    )
    
    # Validate namespace
    if not NAMESPACE or len(NAMESPACE) < 1:
        print("❌ Invalid namespace configuration")
        return False
    
    # Validate port
    if not (1024 <= VAULT_PORT <= 65535):
        print(f"❌ Invalid port number: {VAULT_PORT}")
        return False
    
    # Validate Kubernetes context
    if not KUBERNETES_CONTEXT:
        print("❌ Invalid Kubernetes context")
        return False
    
    # Validate retry configuration
    if MAX_RETRIES < 1 or RETRY_DELAY < 1:
        print("❌ Invalid retry configuration")
        return False
    
    print("✅ Environment configuration is valid")
    return True

def print_configuration():
    """Print current configuration"""
    print("📋 Current Configuration:")
    print("=" * 40)
    
    from .config import (
        NAMESPACE, VAULT_PORT, KUBERNETES_CONTEXT,
        MAX_RETRIES, RETRY_DELAY, VAULT_IMAGE
    )
    
    config_items = [
        ("Namespace", NAMESPACE),
        ("Vault Port", VAULT_PORT),
        ("Kubernetes Context", KUBERNETES_CONTEXT),
        ("Max Retries", MAX_RETRIES),
        ("Retry Delay", RETRY_DELAY),
        ("Vault Image", VAULT_IMAGE),
    ]
    
    for key, value in config_items:
        print(f"  {key}: {value}")
    
    print("=" * 40)

def handle_error(error, cleanup_func=None):
    """Handle errors with optional cleanup"""
    print(f"❌ Error occurred: {error}")
    
    if cleanup_func:
        try:
            cleanup_func()
        except Exception as cleanup_error:
            print(f"⚠️ Cleanup failed: {cleanup_error}")
    
    sys.exit(1) 