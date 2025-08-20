"""
Vault Operations module
Handles Vault initialization, unsealing, and health checks
"""

import os
import json
import time
import subprocess
import requests
from cryptography.fernet import Fernet
from .config import VAULT_DIR, VAULT_PORT, MAX_RETRIES, RETRY_DELAY
from .k8s_operations import port_forward

def check_vault_ready():
    """Cek kesehatan Vault dengan HTTP request langsung"""
    url = f"https://127.0.0.1:{VAULT_PORT}/v1/sys/health"
    try:
        response = requests.get(url, timeout=5, verify=False)  # Disable SSL verification for self-signed cert
        print(f"🌡️ Status Vault: {response.status_code}")
        return response.status_code in [200, 429, 501, 503]
    except requests.exceptions.RequestException as e:
        print(f"🔌 Koneksi gagal: {str(e)}")
        return False

def wait_for_vault_ready(timeout=120):
    """Tunggu sampai Vault merespons"""
    start_time = time.time()
    while time.time() - start_time < timeout:
        if check_vault_ready():
            return True
        print("⏳ Menunggu Vault siap...")
        time.sleep(5)
    return False

def init_vault():
    """Initialize Vault"""
    os.environ["VAULT_ADDR"] = f"https://127.0.0.1:{VAULT_PORT}"
    
    fernet_path = VAULT_DIR / "keys" / "konci.key"
    if not fernet_path.exists():
        print("❌ Kunci Fernet tidak ditemukan!")
        raise FileNotFoundError("Fernet key missing")
    
    fernet_key = fernet_path.read_bytes()
    fernet = Fernet(fernet_key)

    print("🔐 Inisialisasi Vault...")
    pf = port_forward()
    
    try:
        # Tunggu sampai Vault merespons
        if not wait_for_vault_ready():
            print("❌ Vault tidak merespons setelah port-forward")
            return

        # Coba inisialisasi
        for attempt in range(MAX_RETRIES):
            try:
                result = subprocess.run(
                    ["vault", "operator", "init", "-format=json"],
                    capture_output=True,
                    text=True,
                    timeout=30
                )

                if result.returncode == 0:
                    enc_data = fernet.encrypt(result.stdout.encode())
                    (VAULT_DIR / "keys" / "unseal_keys.enc").write_bytes(enc_data)
                    print("✅ Vault berhasil diinisialisasi dan dienkripsi.")
                    return
                else:
                    if "already initialized" in result.stderr:
                        print("ℹ️ Vault sudah diinisialisasi sebelumnya.")
                        return
                    else:
                        print(f"⚠️ Percobaan {attempt+1}/{MAX_RETRIES} gagal: {result.stderr}")
            except Exception as e:
                print(f"⚠️ Error pada percobaan {attempt+1}: {str(e)}")
            
            time.sleep(RETRY_DELAY)
        
        print("❌ Gagal menginisialisasi Vault setelah beberapa percobaan")
        raise SystemExit(1)
                
    finally:
        pf.terminate()
        pf.wait()
        print("🔌 Port-forward dihentikan.")

def unseal_vault():
    """Unseal Vault using stored keys"""
    os.environ["VAULT_ADDR"] = f"https://127.0.0.1:{VAULT_PORT}"
    
    print("🔓 Mencoba auto-unseal Vault...")
    
    # Cek status Vault terlebih dahulu
    try:
        health = requests.get(f"https://127.0.0.1:{VAULT_PORT}/v1/sys/health", timeout=5, verify=False)
        if health.status_code in [200, 429]:
            print("🔓 Vault sudah dalam keadaan unsealed.")
            return
    except:
        pass
    
    fernet_path = VAULT_DIR / "keys" / "konci.key"
    if not fernet_path.exists():
        print("❌ Kunci Fernet tidak ditemukan!")
        return
    
    fernet_key = fernet_path.read_bytes()
    fernet = Fernet(fernet_key)
    enc_path = VAULT_DIR / "keys" / "unseal_keys.enc"
    
    if not enc_path.exists():
        print("❗ Belum ada keys/unseal_keys.enc, lewati auto-unseal.")
        return

    try:
        decrypted = fernet.decrypt(enc_path.read_bytes())
        data = json.loads(decrypted)
        keys = data.get("unseal_keys_b64", [])[:5]  # Use all 5 keys for better fault tolerance
    except Exception as e:
        print(f"❌ Error dekripsi unseal_keys: {str(e)}")
        return

    pf = port_forward()
    try:
        # Tunggu sampai Vault siap
        if not wait_for_vault_ready():
            print("❌ Vault tidak merespons setelah port-forward")
            return
            
        print(f"🔓 Using {len(keys)} unseal keys for fault tolerance")
        for i, key in enumerate(keys, 1):
            print(f"🔑 Proses unseal ({i}/{len(keys)})...")
            for attempt in range(MAX_RETRIES):
                try:
                    result = subprocess.run(
                        ["vault", "operator", "unseal", key],
                        capture_output=True,
                        text=True,
                        timeout=10
                    )
                    if result.returncode == 0:
                        print(f"✅ Key {i} berhasil digunakan")
                        break
                    else:
                        print(f"⚠️ Percobaan {attempt+1}/{MAX_RETRIES} gagal: {result.stderr}")
                except Exception as e:
                    print(f"⚠️ Error pada percobaan {attempt+1}: {str(e)}")
                
                time.sleep(RETRY_DELAY)
            else:
                print(f"❌ Gagal unseal dengan key {i} setelah {MAX_RETRIES} percobaan")
        
        print("✅ Proses unseal selesai.")
    except Exception as e:
        print(f"❌ Error selama unseal: {str(e)}")
    finally:
        pf.terminate()
        pf.wait()

def validate_vault_status():
    """Validate Vault is properly initialized and unsealed"""
    try:
        # Check if Vault is responding
        if not check_vault_ready():
            print("❌ Vault is not responding")
            return False
        
        # Check if Vault is unsealed
        health = requests.get(f"https://127.0.0.1:{VAULT_PORT}/v1/sys/health", timeout=5, verify=False)
        if health.status_code == 200:
            print("✅ Vault is initialized and unsealed")
            return True
        elif health.status_code == 501:
            print("❌ Vault is not initialized")
            return False
        elif health.status_code == 503:
            print("❌ Vault is sealed")
            return False
        else:
            print(f"❌ Unexpected Vault status: {health.status_code}")
            return False
            
    except Exception as e:
        print(f"❌ Error validating Vault status: {e}")
        return False 