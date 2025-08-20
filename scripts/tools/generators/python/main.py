"""
Main module for Kubernetes YAML Generator

Contains the main workflow and orchestration logic.
"""

import os
import shutil
import subprocess
from pathlib import Path
from .config import *
from .utils import *
from .generators import *
from .secret_manager import get_secret

def validate_generated_yaml(path):
    """Validasi YAML dengan error handling yang lebih baik"""
    try:
        # Gunakan kubectl dari microk8s jika tersedia
        kubectl_cmd = "microk8s kubectl" if shutil.which("microk8s") else "kubectl"
        
        logger.info(f"🔍 Validasi dengan {kubectl_cmd} dry-run...")
        result = subprocess.run(
            f"{kubectl_cmd} apply --dry-run=client -k {path}",
            shell=True,
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            logger.error(f"❌ Validasi gagal untuk {path}")
            logger.error(result.stderr)
            return False
        
        logger.info("✅ Semua YAML valid")
        return True
    except Exception as e:
        logger.error(f"❌ Error during validation: {e}")
        return False

def setup_directories():
    """Setup direktori untuk output"""
    shutil.rmtree(K8S_DIR, ignore_errors=True)
    for sub in ["base", "monitoring", "secrets", "ingress"] + [f"overlays/{e}" for e in ENVIRONMENTS]:
        Path(os.path.join(K8S_DIR, sub)).mkdir(parents=True, exist_ok=True)
    Path(os.path.join(K8S_DIR, "secrets/sealed")).mkdir(parents=True, exist_ok=True)

def check_dependencies():
    """Check dependencies yang diperlukan"""
    if shutil.which("kubeseal") is None:
        raise RuntimeError("❌ kubeseal tidak ditemukan. Harap install terlebih dahulu (https://github.com/bitnami-labs/sealed-secrets).")

def load_services():
    """Load services dari docker-compose"""
    try:
        compose = load_docker_compose()
        services = compose.get("services", {})
        if not services:
            raise ValueError("❌ Tidak ada services di docker-compose")
        return services
    except Exception as e:
        raise RuntimeError(f"❌ Gagal memuat docker-compose: {str(e)}")

def save_manifest(obj, filename, subdir, overlay_dir, all_files):
    """Save manifest ke file dan copy ke overlay"""
    try:
        dir_path = os.path.join(K8S_DIR, subdir)
        Path(dir_path).mkdir(parents=True, exist_ok=True)
        
        path = os.path.join(dir_path, filename)
        simpan_yaml(obj, path)
        
        # Salin ke overlay environment
        overlay_path = os.path.join(overlay_dir, filename)
        Path(overlay_dir).mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, overlay_path)
        
        all_files.append(filename)
        return path
    except Exception as e:
        logger.error(f"❌ Error menyimpan {filename}: {e}")
        raise

def generate_base_manifests(overlay_dir, all_files):
    """Generate manifest dasar"""
    save_manifest(generate_namespace(), "namespace.yaml", "base", overlay_dir, all_files)
    save_manifest(generate_db_pvc(), "db-pvc.yaml", "base", overlay_dir, all_files)

def generate_monitoring_manifests(services, overlay_dir, all_files):
    """Generate monitoring manifests"""
    monitoring_resources = generate_monitoring_resources(services)
    for i, resource in enumerate(monitoring_resources):
        filename = f"monitoring-{i+1}.yaml"
        save_manifest(resource, filename, "monitoring", overlay_dir, all_files)

def generate_service_manifests(services, overlay_dir, all_files, secrets=None):
    """Generate manifests untuk setiap service"""
    for name, config in services.items():
        logger.info(f"  🛠️  Memproses service: {name}")
        
        # Generate deployment
        try:
            deployment = generate_deployment(name, config)
            save_manifest(deployment, f"{name}-deployment.yaml", "base", overlay_dir, all_files)
        except Exception as e:
            add_error(f"❌ Gagal generate deployment untuk {name}: {str(e)}")
            continue
        
        # Generate service
        try:
            service = generate_service(name, config)
            save_manifest(service, f"{name}-service.yaml", "base", overlay_dir, all_files)
        except Exception as e:
            add_error(f"❌ Gagal generate service untuk {name}: {str(e)}")
            continue
        
        # Generate secret jika ada
        secret_obj = None
        if secrets:
            env_secret = secrets.get(name) if isinstance(secrets, dict) and name in secrets else secrets
            if env_secret and isinstance(env_secret, dict):
                secret_obj = {
                    "apiVersion": "v1",
                    "kind": "Secret",
                    "metadata": {
                        "name": f"{name}-secret",
                        "namespace": DEFAULT_NAMESPACE
                    },
                    "type": "Opaque",
                    "stringData": env_secret
                }
                logger.info(f"Inject secrets dari secret manager ke Secret K8s {name}-secret")
        if not secret_obj:
            secret_obj = generate_secret_from_env(name)
        if secret_obj:
            try:
                secret_path = save_manifest(secret_obj, f"{name}-secret.yaml", "secrets", overlay_dir, all_files)
                seal_path = seal_env_to_sealed_secret(secret_obj, name)
                if seal_path:
                    logger.info(f"✅ SealedSecret untuk {name} disimpan di {seal_path}")
            except Exception as e:
                logger.error(f"❌ Error generating secret untuk {name}: {e}")

def generate_ingress_manifests(services, overlay_dir, all_files):
    """Generate ingress manifests"""
    save_manifest(generate_ingress(services), "ingress.yaml", "ingress", overlay_dir, all_files)

def generate_tls_secret(overlay_dir, all_files):
    """Generate TLS secret jika ada"""
    tls_secret = generate_ingress_tls_secret()
    if tls_secret:
        try:
            tls_secret_path = save_manifest(tls_secret, "ingress-tls.yaml", "secrets", overlay_dir, all_files)
        except Exception as e:
            logger.error(f"❌ Error generating TLS secret: {e}")

def generate_kustomization(overlay_dir, all_files):
    """Generate kustomization file"""
    kustomization = {
        "apiVersion": "kustomize.config.k8s.io/v1beta1",
        "kind": "Kustomization",
        "namespace": DEFAULT_NAMESPACE,
        "resources": all_files
    }
    kustom_path = os.path.join(overlay_dir, "kustomization.yaml")
    simpan_yaml(kustomization, kustom_path)

def copy_sealed_secrets(overlay_dir, all_files):
    """Copy sealed secrets ke overlay"""
    sealed_dir = Path(K8S_DIR) / "secrets" / "sealed"
    for sealed_file in sealed_dir.glob("*.yaml"):
        try:
            dst = Path(overlay_dir) / sealed_file.name
            shutil.copy(sealed_file, dst)
            if sealed_file.name not in all_files:
                all_files.append(sealed_file.name)
        except Exception as e:
            logger.error(f"❌ Error copying sealed secret {sealed_file.name}: {e}")

def handle_errors():
    """Handle dan report errors"""
    if has_errors():
        logger.error("\n🚫 Error ditemukan saat generate YAML:")
        for i, e in enumerate(get_errors(), 1):
            logger.error(f"{i}. {e}")
        logger.error("❌ Gagal generate karena environment tidak valid.")
        return False
    return True

def main():
    """Main workflow dengan error handling yang lebih baik"""
    try:
        # Validasi environment
        validate_environment()
        
        logger.info(f"🔧 Generate YAML untuk environment: {ENV}, tag: {TAG}")
        
        # Setup direktori
        setup_directories()
        
        overlay_dir = os.path.join(K8S_DIR, f"overlays/{ENV}")
        all_files = []

        # Cek dependensi
        check_dependencies()
        
        # Load services
        services = load_services()
        
        # --- Integrasi Secret Manager ---
        import argparse
        import sys
        parser = argparse.ArgumentParser(add_help=False)
        parser.add_argument("--secret-provider", choices=["vault", "aws", "gcp", "azure"], default=None)
        parser.add_argument("--secret-path", default=None)
        parser.add_argument("--secret-token", default=None)
        parser.add_argument("--secret-url", default=None)
        args, _ = parser.parse_known_args()
        secrets = None
        if getattr(args, 'secret_provider', None) and getattr(args, 'secret_path', None):
            logger.info(f"Mengambil secrets dari {args.secret_provider} path={args.secret_path}")
            secret_kwargs = {}
            if args.secret_provider == 'vault':
                secret_kwargs['token'] = args.secret_token
                secret_kwargs['url'] = args.secret_url
            secrets = get_secret(args.secret_provider, args.secret_path, **secret_kwargs)
            if not secrets:
                logger.error("Gagal mengambil secrets dari secret manager, fallback ke .env jika ada.")
        else:
            logger.info("Secret manager tidak digunakan, fallback ke .env.")
        
        # Generate semua manifest
        generate_base_manifests(overlay_dir, all_files)
        generate_monitoring_manifests(services, overlay_dir, all_files)
        generate_ingress_manifests(services, overlay_dir, all_files)
        generate_service_manifests(services, overlay_dir, all_files, secrets=secrets)
        generate_tls_secret(overlay_dir, all_files)
        
        # Handle errors
        if not handle_errors():
            return False
        
        # Generate kustomization dan copy sealed secrets
        generate_kustomization(overlay_dir, all_files)
        copy_sealed_secrets(overlay_dir, all_files)
        
        # Validasi
        if not validate_generated_yaml(overlay_dir):
            logger.error("❌ Validasi YAML gagal")
            return False
            
        logger.info(f"✅ YAML untuk environment `{ENV}` berhasil digenerate di `{overlay_dir}`")
        return True
        
    except Exception as e:
        logger.error(f"❌ Error dalam main workflow: {e}")
        return False

if __name__ == "__main__":
    logger.info("📦 Generate YAML Kubernetes dimulai...")
    success = main()
    if not success:
        exit(1) 