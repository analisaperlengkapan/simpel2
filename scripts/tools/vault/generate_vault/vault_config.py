"""
Vault Configuration module
Handles Vault HCL configuration and environment files
"""

from .config import (
    VAULT_PORT, VAULT_UI_ENABLED, VAULT_DISABLE_MLOCK,
    TLS_MIN_VERSION, TLS_CIPHER_SUITES
)

def generate_vault_hcl_config():
    """Generate Vault HCL configuration"""
    config_hcl = f'''
ui = {str(VAULT_UI_ENABLED).lower()}
listener "tcp" {{
  address     = "0.0.0.0:{VAULT_PORT}"
  tls_cert_file = "/vault/certs/server.crt"
  tls_key_file = "/vault/certs/server.key"
  tls_min_version = "{TLS_MIN_VERSION}"
  tls_cipher_suites = "{TLS_CIPHER_SUITES}"
}}
storage "file" {{
  path = "/vault/data"
}}
disable_mlock = {str(VAULT_DISABLE_MLOCK).lower()}

# Authentication methods
auth "kubernetes" {{
  path = "kubernetes"
  config = {{
    kubernetes_host = "https://kubernetes.default.svc"
    kubernetes_ca_cert = "/var/run/secrets/kubernetes.io/serviceaccount/ca.crt"
    token_reviewer_jwt = "/var/run/secrets/kubernetes.io/serviceaccount/token"
  }}
}}

auth "userpass" {{
  path = "userpass"
}}

# Enable audit logging
audit "file" {{
  path = "/vault/audit/audit.log"
  format = "json"
  log_raw = true
}}
'''
    return config_hcl.strip()

def generate_vault_env_config():
    """Generate Vault environment configuration"""
    vault_env = f'VAULT_ADDR=https://127.0.0.1:{VAULT_PORT}'
    return vault_env.strip()

def write_vault_files():
    """Write Vault configuration files"""
    from .config import VAULT_DIR
    
    # Generate configurations
    config_hcl = generate_vault_hcl_config()
    vault_env = generate_vault_env_config()
    
    # Write HCL configuration
    config_path = VAULT_DIR / "config" / "config.hcl"
    config_path.write_text(config_hcl)
    print(f"✅ Vault HCL configuration written: {config_path}")
    
    # Write environment file
    env_path = VAULT_DIR / "vault.env"
    env_path.write_text(vault_env)
    print(f"✅ Vault environment file written: {env_path}")
    
    return config_path, env_path 