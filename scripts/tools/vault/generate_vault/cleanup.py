"""
Cleanup module for Vault setup
Handles file cleanup and deployment cleanup operations
"""

import shutil
from pathlib import Path
from .config import VAULT_DIR, K8S_DIR, CLEANUP_FILES, SCRIPT_PATH

def cleanup_files():
    """Hapus semua file yang dibuat oleh script ini"""
    print("🧹 Membersihkan file hasil setup...")
    
    # Hapus direktori VAULT_DIR dan isinya
    if VAULT_DIR.exists():
        shutil.rmtree(VAULT_DIR, ignore_errors=True)
        print(f"🗑️ Menghapus direktori Vault: {VAULT_DIR}")
    
    # Hapus file di K8S_DIR
    for file_name in CLEANUP_FILES:
        file_path = K8S_DIR / file_name
        if file_path.exists():
            file_path.unlink()
            print(f"🗑️ Menghapus file: {file_path}")

def self_destruct():
    """Hapus script ini sendiri setelah eksekusi sukses"""
    print(f"🧹 Menghapus script: {SCRIPT_PATH}")
    try:
        SCRIPT_PATH.unlink()
        print("✅ Script berhasil dihapus")
    except Exception as e:
        print(f"❌ Gagal menghapus script: {str(e)}")

def ensure_dirs():
    """Ensure all required directories exist"""
    from .config import DIRECTORIES
    
    for directory in DIRECTORIES:
        dir_path = VAULT_DIR / directory
        dir_path.mkdir(parents=True, exist_ok=True)
    
    # Ensure K8S directory exists
    K8S_DIR.mkdir(parents=True, exist_ok=True)
    
    print("✅ All directories created/verified")

def cleanup_on_failure():
    """Cleanup on failure - remove generated files but keep script"""
    print("🔄 Cleanup on failure...")
    cleanup_files()
    print("✅ Cleanup completed")

def cleanup_on_success():
    """Cleanup on success - remove all files including script"""
    print("🧹 Cleanup on success...")
    cleanup_files()
    self_destruct()
    print("✅ Complete cleanup finished") 