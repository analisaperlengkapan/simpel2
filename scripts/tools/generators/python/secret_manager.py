"""
Secret Manager integration for SIMPelv2 generator

Support: HashiCorp Vault (implementasi), AWS/GCP/Azure (stub)
"""
import requests
import logging

logger = logging.getLogger(__name__)

# --- Main dispatch function ---
def get_secret(provider, path, **kwargs):
    """
    Ambil secret dari provider tertentu.
    provider: 'vault', 'aws', 'gcp', 'azure'
    path: path/key secret di provider
    kwargs: argumen tambahan (token, url, region, dsb)
    """
    if provider == 'vault':
        return get_secret_vault(path, **kwargs)
    elif provider == 'aws':
        return get_secret_aws(path, **kwargs)
    elif provider == 'gcp':
        return get_secret_gcp(path, **kwargs)
    elif provider == 'azure':
        return get_secret_azure(path, **kwargs)
    else:
        logger.error(f"Provider secret tidak didukung: {provider}")
        return None

# --- HashiCorp Vault ---
def get_secret_vault(path, token=None, url=None, mount='secret'):
    """
    Ambil secret dari HashiCorp Vault (KV v2/v1).
    path: path secret (misal: secret/data/myapp/dev)
    token: Vault token
    url: Vault base URL (misal: http://localhost:8200)
    mount: mount point (default 'secret')
    """
    if not token or not url:
        logger.error("Vault token dan url harus diisi!")
        return None
    # Support KV v2 (secret/data/...) dan v1 (secret/...)
    if '/data/' in path:
        api_path = f"{url}/v1/{path}"
    else:
        api_path = f"{url}/v1/{mount}/{path}"
    headers = {"X-Vault-Token": token}
    try:
        resp = requests.get(api_path, headers=headers, timeout=10)
        resp.raise_for_status()
        data = resp.json()
        # KV v2: data['data']['data'], KV v1: data['data']
        if 'data' in data and 'data' in data['data']:
            return data['data']['data']
        elif 'data' in data:
            return data['data']
        else:
            logger.error(f"Format response Vault tidak dikenali: {data}")
            return None
    except Exception as e:
        logger.error(f"Gagal ambil secret dari Vault: {e}")
        return None

# --- AWS Secrets Manager (stub) ---
def get_secret_aws(path, region=None, **kwargs):
    """
    Ambil secret dari AWS Secrets Manager (stub).
    path: nama secret
    region: region AWS
    """
    logger.warning("get_secret_aws belum diimplementasi. Gunakan boto3 untuk implementasi.")
    return None

# --- GCP Secret Manager (stub) ---
def get_secret_gcp(path, project_id=None, **kwargs):
    """
    Ambil secret dari GCP Secret Manager (stub).
    path: nama secret
    project_id: GCP project id
    """
    logger.warning("get_secret_gcp belum diimplementasi. Gunakan google-cloud-secret-manager untuk implementasi.")
    return None

# --- Azure Key Vault (stub) ---
def get_secret_azure(path, vault_url=None, **kwargs):
    """
    Ambil secret dari Azure Key Vault (stub).
    path: nama secret
    vault_url: URL Key Vault
    """
    logger.warning("get_secret_azure belum diimplementasi. Gunakan azure-identity dan azure-keyvault-secrets untuk implementasi.")
    return None 