from cryptography.fernet import Fernet

KEY_FILE = "infra/vault/keys/konci.key"
TOKEN_FILE = "infra/vault/keys/root_token.enc"

fernet = Fernet(open(KEY_FILE, "rb").read())
token = fernet.decrypt(open(TOKEN_FILE, "rb").read()).decode()

print(f"🔑 Root Token Vault:\n{token}")
