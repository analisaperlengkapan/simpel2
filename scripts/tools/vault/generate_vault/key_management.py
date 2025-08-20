"""
Key Management module for Vault
Handles Fernet key generation, backup, and SealedSecret creation
"""

import os
import base64
import hashlib
import json
import subprocess
from pathlib import Path
from cryptography.fernet import Fernet
from .config import (
    VAULT_DIR, NAMESPACE, SECRET_NAME, KEY_FILE_PERMISSIONS
)

def generate_and_store_fernet_key():
    """Generate and store Fernet key with backup"""
    fernet_path = VAULT_DIR / "keys" / "konci.key"

    if not fernet_path.exists():
        fernet_key = Fernet.generate_key()
        fernet_path.write_bytes(fernet_key)
        os.chmod(fernet_path, KEY_FILE_PERMISSIONS)
        print("🔑 Kunci Fernet baru berhasil dibuat")
        
        # Create backup
        backup_fernet_key(fernet_key)
    else:
        print("ℹ️ Kunci Fernet sudah ada")

    create_sealed_secret(fernet_path)
    return fernet_path

def backup_fernet_key(fernet_key):
    """Backup Fernet key to secure location"""
    print("💾 Creating key backup...")
    
    try:
        # Create backup directory
        backup_dir = VAULT_DIR / "backup"
        backup_dir.mkdir(exist_ok=True)
        
        # Create backup key using hash of original key
        backup_key = hashlib.sha256(fernet_key).digest()
        
        # Encrypt with backup key
        backup_fernet = Fernet(base64.urlsafe_b64encode(backup_key))
        encrypted_backup = backup_fernet.encrypt(fernet_key)
        
        # Store backup
        backup_path = backup_dir / "vault-key.enc"
        backup_path.write_bytes(encrypted_backup)
        os.chmod(backup_path, KEY_FILE_PERMISSIONS)
        
        print(f"✅ Key backup created: {backup_path}")
        return backup_path
    except Exception as e:
        print(f"⚠️ Failed to create key backup: {e}")
        return None

def create_sealed_secret(fernet_path):
    """Create SealedSecret for Fernet key"""
    fernet_key = fernet_path.read_bytes()

    secret_manifest = {
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": SECRET_NAME,
            "namespace": NAMESPACE
        },
        "data": {
            "konci.key": base64.b64encode(fernet_key).decode('utf-8')
        }
    }

    temp_secret_path = VAULT_DIR / "temp-secret.yaml"
    with open(temp_secret_path, 'w') as f:
        json.dump(secret_manifest, f)

    sealed_secret_path = VAULT_DIR / "k8s" / "fernet-sealedsecret.yaml"
    try:
        subprocess.run([
            "kubeseal",
            "--format", "yaml",
            "--namespace", NAMESPACE,
            "-f", str(temp_secret_path),
            "-w", str(sealed_secret_path)
        ], check=True)
        print(f"🔐 SealedSecret berhasil dibuat: {sealed_secret_path}")
    except subprocess.CalledProcessError as e:
        print(f"❌ Gagal membuat SealedSecret: {e}")
        raise
    finally:
        temp_secret_path.unlink(missing_ok=True)

def create_tls_secret(cert_path, key_path):
    """Create TLS certificate secret for Vault"""
    print("🔐 Creating TLS certificate secret...")
    
    cert_data = cert_path.read_bytes()
    key_data = key_path.read_bytes()
    
    secret_manifest = {
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": "vault-tls-certs",
            "namespace": NAMESPACE
        },
        "type": "kubernetes.io/tls",
        "data": {
            "tls.crt": base64.b64encode(cert_data).decode('utf-8'),
            "tls.key": base64.b64encode(key_data).decode('utf-8')
        }
    }
    
    tls_secret_path = VAULT_DIR / "k8s" / "tls-secret.yaml"
    with open(tls_secret_path, 'w') as f:
        json.dump(secret_manifest, f, indent=2)
    
    print(f"✅ TLS secret created: {tls_secret_path}")
    return tls_secret_path

def validate_key_backup():
    """Validate key backup exists and is accessible"""
    backup_path = VAULT_DIR / "backup" / "vault-key.enc"
    if backup_path.exists():
        print("✅ Key backup exists and is accessible")
        return True
    else:
        print("❌ Key backup not found")
        return False 