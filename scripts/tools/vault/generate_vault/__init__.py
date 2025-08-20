"""
Vault Setup Generation Package
Modular package for generating and deploying Vault with security improvements
"""

__version__ = "2.0.0"
__author__ = "SimpelV2 Team"
__description__ = "Secure Vault Setup with Modular Architecture"

def setup_vault():
    """Main function to setup Vault with all security improvements"""
    import traceback
    
    # Import modules here to avoid circular imports
    from .utils import check_prerequisites, check_permissions, validate_environment, print_configuration
    from .cleanup import ensure_dirs, cleanup_on_failure, cleanup_on_success
    from .vault_config import write_vault_files
    from .k8s_manifests import write_k8s_manifests
    from .k8s_operations import deploy_to_k8s
    from .vault_operations import init_vault, unseal_vault
    
    success = False
    try:
        # Validation phase
        if not check_prerequisites():
            raise RuntimeError("Prerequisites check failed")
        
        if not check_permissions():
            raise RuntimeError("Permissions check failed")
        
        if not validate_environment():
            raise RuntimeError("Environment validation failed")
        
        # Print configuration
        print_configuration()
        
        # Setup phase
        ensure_dirs()
        write_vault_files()
        write_k8s_manifests()
        print("✅ Vault siap dijalankan")
        
        # Deployment phase
        if not deploy_to_k8s():
            raise RuntimeError("Deployment Vault gagal")

        # Streaming log pod Vault secara realtime (opsional, non-blocking)
        import os
        if os.environ.get('SHOW_LOG', '0') == '1':
            from .k8s_operations import KUBERNETES_CONTEXT, NAMESPACE
            import threading, subprocess
            def stream_log():
                subprocess.run([
                    KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE, "logs", "-f", "deployment/vault"
                ])
            t = threading.Thread(target=stream_log, daemon=True)
            t.start()
            print("[INFO] Streaming log Vault berjalan di background thread. Lanjutkan workflow...")
        else:
            print("[INFO] Streaming log Vault tidak dijalankan otomatis. Jalankan manual jika perlu.")

        # Beri waktu tambahan untuk Vault benar-benar siap
        print("⏳ Memberi waktu tambahan untuk Vault start...")
        import time
        time.sleep(30)
        
        # Initialization phase
        init_vault()
        unseal_vault()
        print("🚀 Vault berhasil di-deploy dan di-unseal")
        success = True
        
    except Exception as e:
        print(f"❌ Fatal error: {str(e)}")
        print("🔄 Melakukan rollback...")
        traceback.print_exc()
    finally:
        # Cleanup phase
        if success:
            cleanup_on_success()
        else:
            cleanup_on_failure()
        
        # Exit with appropriate code
        if not success:
            import sys
            sys.exit(1)

# Export main function
__all__ = ['setup_vault'] 