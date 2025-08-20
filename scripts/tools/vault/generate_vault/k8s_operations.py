"""
Kubernetes Operations module
Handles Kubernetes deployment, cleanup, and management operations
"""

import subprocess
import time
import threading
from .config import NAMESPACE, KUBERNETES_CONTEXT, MAX_RETRIES, RETRY_DELAY

def cleanup_previous_deployment():
    """Hapus deployment sebelumnya untuk menghindari konflik"""
    print("🧹 Membersihkan deployment sebelumnya...")
    
    cleanup_commands = [
        ["delete", "deployment", "vault", "-n", NAMESPACE, "--ignore-not-found"],
        ["delete", "pvc", "vault-pvc", "-n", NAMESPACE, "--ignore-not-found"],
        ["-n", NAMESPACE, "delete", "configmap", "vault-config", "--ignore-not-found"],
        ["-n", NAMESPACE, "delete", "configmap", "vault-env", "--ignore-not-found"],
        ["-n", NAMESPACE, "delete", "secret", "vault-tls-certs", "--ignore-not-found"],
        ["-n", NAMESPACE, "delete", "networkpolicy", "vault-network-policy", "--ignore-not-found"],
    ]
    
    for cmd in cleanup_commands:
        subprocess.run(
            [KUBERNETES_CONTEXT, "kubectl"] + cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
    
    time.sleep(5)  # Beri waktu untuk penghapusan
    print("✅ Cleanup completed")

def create_namespace():
    """Create namespace if it doesn't exist"""
    print(f"📦 Creating namespace: {NAMESPACE}")
    subprocess.run(
        [KUBERNETES_CONTEXT, "kubectl", "create", "namespace", NAMESPACE],
        stderr=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        check=False
    )

def deploy_to_k8s():
    """Deploy Vault to Kubernetes"""
    from .config import K8S_DIR
    
    cleanup_previous_deployment()
    create_namespace()

    print("🚀 Deploying to Kubernetes...")
    subprocess.run(
        [KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE, "apply", "-k", str(K8S_DIR)],
        check=True
    )

    print("🔄 Menunggu deployment Vault siap...")
    stream_vault_logs(NAMESPACE)

    # Menunggu deployment dengan timeout lebih lama
    try:
        subprocess.run(
            [
                KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
                "wait", "--for=condition=available",
                "deployment/vault", "--timeout=300s"
            ],
            check=True
        )
        print("✅ Deployment Vault siap")
        return True
    except subprocess.CalledProcessError:
        print("❌ Gagal menunggu deployment Vault siap")
        return False

def stream_vault_logs(namespace, label_selector="app=vault"):
    """Stream Vault logs in background"""
    def _stream():
        try:
            get_pod_cmd = [
                KUBERNETES_CONTEXT, "kubectl", "-n", namespace,
                "get", "pods",
                "-l", label_selector,
                "-o", "jsonpath={.items[0].metadata.name}"
            ]
            result = subprocess.run(get_pod_cmd, capture_output=True, text=True)
            pod_name = result.stdout.strip()
            if not pod_name:
                print("❗ Tidak bisa menemukan pod Vault untuk streaming log.")
                return

            print(f"📺 Streaming log dari pod: {pod_name}")
            subprocess.run([KUBERNETES_CONTEXT, "kubectl", "-n", namespace, "logs", "-f", pod_name])
        except Exception as e:
            print(f"⚠️ Gagal streaming log Vault: {e}")
    
    threading.Thread(target=_stream, daemon=True).start()

def port_forward():
    """Setup port forwarding for Vault"""
    from .config import VAULT_PORT
    
    try:
        proc = subprocess.Popen(
            [
                KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
                "port-forward", "deployment/vault", f"{VAULT_PORT}:{VAULT_PORT}"
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        time.sleep(3)  # Beri waktu agar port-forward terbentuk
        return proc
    except Exception as e:
        print(f"❌ Gagal port-forward: {e}")
        raise SystemExit(1)

def check_deployment_status():
    """Check deployment status"""
    try:
        result = subprocess.run([
            KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
            "get", "deployment", "vault", "-o", "jsonpath={.status.conditions[?(@.type=='Available')].status}"
        ], capture_output=True, text=True)
        
        return result.stdout.strip() == "True"
    except Exception:
        return False

def get_pod_name():
    """Get Vault pod name"""
    try:
        result = subprocess.run([
            KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
            "get", "pods", "-l", "app=vault", "-o", "jsonpath={.items[0].metadata.name}"
        ], capture_output=True, text=True)
        
        return result.stdout.strip()
    except Exception:
        return None 