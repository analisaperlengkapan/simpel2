import time
import subprocess
import os
from cryptography.fernet import Fernet

KEY_FILE = "infra/vault/keys/konci.key"
UNSEAL_FILE = "infra/vault/keys/unseal_keys.enc"

# Set Vault address to HTTPS
os.environ["VAULT_ADDR"] = "https://127.0.0.1:8200"
os.environ["VAULT_SKIP_VERIFY"] = "true"  # Skip SSL verification for self-signed cert

fernet = Fernet(open(KEY_FILE, "rb").read())
unseal_keys = fernet.decrypt(open(UNSEAL_FILE, "rb").read()).decode().splitlines()

print("🔓 Mengirim unseal key ke Vault...")
for key in unseal_keys[:5]:  # Use all 5 keys for better fault tolerance
    subprocess.run(["vault", "operator", "unseal", key])
    time.sleep(1)

print("✅ Vault telah unseal.")
