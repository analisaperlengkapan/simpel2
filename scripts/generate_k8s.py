import os
import json
import base64
import yaml
import shutil
from pathlib import Path
from dotenv import dotenv_values
from yaml.representer import SafeRepresenter

BASE_DIR = "/var/www/simpelv2"
COMPOSE_FILE = os.path.join(BASE_DIR, "docker-compose.secure.yml")
K8S_DIR = os.path.join(BASE_DIR, "k8s")
DEFAULT_NAMESPACE = "simpelv2"
DEFAULT_PORT = 3000
DEFAULT_CMD = ["./main"]
ENVIRONMENTS = ["dev", "staging", "prod"]

ENV = os.environ.get("ENV", "dev")
TAG = os.environ.get("TAG", "dev")

errors = []

class QuotedString(str): pass
def quoted_scalar(dumper, data):
    return dumper.represent_scalar("tag:yaml.org,2002:str", data, style='"')

yaml.add_representer(QuotedString, quoted_scalar)
yaml.add_representer(str, SafeRepresenter.represent_str)

def simpan_yaml(obj, path):
    def quote_keys(d):
        if isinstance(d, dict):
            return {QuotedString(k): quote_keys(v) for k, v in d.items()}
        elif isinstance(d, list):
            return [quote_keys(i) for i in d]
        elif isinstance(d, str):
            if len(d) > 120 and "\n" not in d:
                return QuotedString(d)
            return d
        else:
            return d

    obj = quote_keys(obj)
    with open(path, "w") as f:
        yaml.dump(obj, f, sort_keys=False, default_flow_style=False, width=120)

def load_docker_compose():
    if not os.path.exists(COMPOSE_FILE):
        raise FileNotFoundError("❌ docker-compose.secure.yml tidak ditemukan.")
    with open(COMPOSE_FILE) as f:
        return yaml.safe_load(f)

def generate_namespace():
    return {
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {"name": DEFAULT_NAMESPACE}
    }

def generate_secret_from_env(service_name):
    env_path = os.path.join(BASE_DIR, service_name, ".env")
    if not os.path.exists(env_path):
        # fallback untuk db-simpelv2 di root
        if service_name == "db-simpelv2":
            env_path = os.path.join(BASE_DIR, ".env")
        if not os.path.exists(env_path):
            print(f"⚠️  Tidak ditemukan .env untuk {service_name}, lewati secret.")
            return None

    env_data_raw = dotenv_values(env_path)
    env_data = {k: str(v) for k, v in env_data_raw.items() if v is not None}
    return {
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": f"{service_name}-secret",
            "namespace": DEFAULT_NAMESPACE
        },
        "type": "Opaque",
        "stringData": env_data
    }

def generate_ingress_tls_secret():
    cert_path = Path(BASE_DIR) / "nginx" / "certs" / "fullchain.pem"
    key_path = Path(BASE_DIR) / "nginx" / "certs" / "privkey.pem"

    if not cert_path.exists() or not key_path.exists():
        print("⚠️  TLS cert/key tidak ditemukan di nginx/certs/, lewati TLS secret.")
        return None

    with open(cert_path, "rb") as f:
        cert_data = base64.b64encode(f.read()).decode()
    with open(key_path, "rb") as f:
        key_data = base64.b64encode(f.read()).decode()

    return {
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": "simpelv2-tls",
            "namespace": DEFAULT_NAMESPACE
        },
        "type": "kubernetes.io/tls",
        "data": {
            "tls.crt": cert_data,
            "tls.key": key_data
        }
    }

def generate_db_pvc():
    return {
        "apiVersion": "v1",
        "kind": "PersistentVolumeClaim",
        "metadata": {
            "name": "db-simpelv2-pvc",
            "namespace": DEFAULT_NAMESPACE
        },
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": {"requests": {"storage": "1Gi"}}
        }
    }

def generate_deployment(name, image, port, command, envs, volumes):
    volume_mounts, volume_defs = [], []

    for idx, vol in enumerate(volumes):
        if not isinstance(vol, str) or ":" not in vol:
            errors.append(f"❌ Format volume tidak dikenali: {vol} di service {name}")
            continue

        src, dst, *opt = vol.split(":")
        readonly = True if (opt and opt[0] == "ro") else False
        vol_name = f"{name}-vol-{idx}"

        volume_mounts.append({
            "name": vol_name,
            "mountPath": dst,
            "readOnly": readonly
        })

        volume_defs.append({
            "name": vol_name,
            "hostPath": {
                "path": os.path.abspath(src),
                "type": "DirectoryOrCreate"
            }
        })

    env_items = []
    for k, v in envs.items():
        if isinstance(v, (list, dict)):
            errors.append(f"❌ ENV `{k}` di service `{name}` invalid: {v} ({type(v).__name__})")
            continue
        env_items.append({
            "name": str(k),
            "value": str(v)
        })

    container_spec = {
        "name": name,
        "image": image,
        "imagePullPolicy": "IfNotPresent",
        "ports": [{"containerPort": port}],
        **({"command": command} if command else {}),
        "volumeMounts": volume_mounts
    }

    if env_items:
        container_spec["env"] = env_items
    secret_obj = generate_secret_from_env(name)
    if secret_obj:
        container_spec.setdefault("envFrom", []).append({
            "secretRef": {"name": f"{name}-secret"}
        })

    return {
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {"name": name, "namespace": DEFAULT_NAMESPACE},
        "spec": {
            "replicas": 1,
            "selector": {"matchLabels": {"app": name}},
            "template": {
                "metadata": {"labels": {"app": name}},
                "spec": {
                    "restartPolicy": "Always",
                    "containers": [container_spec],
                    "volumes": volume_defs
                }
            }
        }
    }

def generate_service(name, port):
    return {
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {"name": name, "namespace": DEFAULT_NAMESPACE},
        "spec": {
            "selector": {"app": name},
            "ports": [{"port": port, "targetPort": port}]
        }
    }

def generate_ingress():
    ingress = {
        "apiVersion": "networking.k8s.io/v1",
        "kind": "Ingress",
        "metadata": {
            "name": "simpelv2-ingress",
            "namespace": DEFAULT_NAMESPACE,
            "annotations": {
                "nginx.ingress.kubernetes.io/rewrite-target": "/$1"
            }
        },
        "spec": {
            "rules": [ {
                "host": "simpelv2.local",
                "http": {
                    "paths": [ {
                        "path": "/(.*)",
                        "pathType": "Prefix",
                        "backend": {
                            "service": {
                                "name": "gerbang",
                                "port": {"number": 3000}
                            }
                        }
                    }]
                }
            }]
        }
    }

    if (Path(BASE_DIR) / "nginx" / "certs" / "fullchain.pem").exists():
        ingress["spec"]["tls"] = [{
            "hosts": ["simpelv2.local"],
            "secretName": "simpelv2-tls"
        }]
    return ingress

def setup_directories():
    shutil.rmtree(K8S_DIR, ignore_errors=True)
    for sub in ["base", "monitoring", "secrets", "ingress"] + [f"overlays/{e}" for e in ENVIRONMENTS]:
        Path(os.path.join(K8S_DIR, sub)).mkdir(parents=True, exist_ok=True)

def generate_monitoring_resources():
    prometheus = {
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {"name": "prometheus-config", "namespace": DEFAULT_NAMESPACE},
        "data": {"prometheus.yml": """
global:
  scrape_interval: 15s
scrape_configs:
  - job_name: 'simpelv2'
    static_configs:
      - targets: ['localhost:9090']
"""}
    }

    grafana = {
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {"name": "grafana-datasource", "namespace": DEFAULT_NAMESPACE},
        "data": {"datasource.yml": """
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
"""}
    }

    paths = []
    for obj, name in [(prometheus, "prometheus.yaml"), (grafana, "grafana.yaml")]:
        full_path = os.path.join(K8S_DIR, "monitoring", name)
        simpan_yaml(obj, full_path)
        paths.append(full_path)
    return paths

def validate_generated_yaml(path):
    print("🔍 Validasi hasil kustomize build dengan microk8s kubectl dry-run...")
    ret = os.system(f"microk8s kubectl apply --dry-run=client -k {path} > /dev/null")
    if ret != 0:
        print(f"❌ Validasi gagal untuk hasil kustomization di: {path}")
        exit(1)
    print("✅ Semua YAML valid (kustomize build success)")

def main():
    print(f"🔧 Generate YAML untuk environment: {ENV}, tag: {TAG}")
    compose = load_docker_compose()
    services = compose.get("services", {})
    setup_directories()
    Path(os.path.join(K8S_DIR, "secrets/sealed")).mkdir(parents=True, exist_ok=True)
    overlay_dir = os.path.join(K8S_DIR, f"overlays/{ENV}")
    all_files = []

    if os.system("which kubeseal > /dev/null") != 0:
        print("❌ kubeseal tidak ditemukan. Harap install terlebih dahulu (https://github.com/bitnami-labs/sealed-secrets).")
        exit(1)

    def simpan(obj, filename, subdir):
        path = os.path.join(K8S_DIR, subdir, filename)
        simpan_yaml(obj, path)
        copied = os.path.join(overlay_dir, filename)
        shutil.copyfile(path, copied)
        all_files.append(filename)

    simpan(generate_namespace(), "namespace.yaml", "base")
    simpan(generate_db_pvc(), "db-simpelv2-pvc.yaml", "base")
    simpan(generate_ingress(), "ingress.yaml", "ingress")

    for monitor_path in generate_monitoring_resources():
        fname = os.path.basename(monitor_path)
        shutil.copyfile(monitor_path, os.path.join(overlay_dir, fname))
        all_files.append(fname)

    for name, config in services.items():
        image = config.get("image", f"simpelv2/{name}:{TAG}")
        if image.startswith("simpelv2/") and ":" not in image:
            image = f"{image}:{TAG}"
        expose = config.get("expose", [])
        ports = config.get("ports", [])
        environment = config.get("environment", {})
        volumes = config.get("volumes", [])

        env_path = os.path.join(BASE_DIR, name, ".env")
        if os.path.exists(env_path):
            dotenv_env = dotenv_values(env_path)
            # environment.update({k: str(v) for k, v in dotenv_env.items() if v is not None})

        invalid_env = False
        for k, v in environment.items():
            if isinstance(v, (list, dict)):
                errors.append(f"❌ ENV `{k}` di service `{name}` invalid: {v} ({type(v).__name__})")
                invalid_env = True

        if invalid_env:
            print(f"⚠️  Lewati service `{name}` karena ENV tidak valid.")
            for f in [
                f"{name}-deployment.yaml",
                f"{name}-service.yaml",
                f"{name}-secret.yaml"
            ]:
                for folder in ["base", "secrets", f"overlays/{ENV}"]:
                    path = os.path.join(K8S_DIR, folder, f)
                    if os.path.exists(path):
                        os.remove(path)
            continue

        command = config.get("command", DEFAULT_CMD)
        port = int(expose[0]) if expose else int(str(ports[0]).split(":")[-1]) if ports else DEFAULT_PORT

        if name == "db-simpelv2":
            port = 5432
            command = None
        elif name in ["nginx", "antarmuka"]:
            command = None

        if not isinstance(command, list) and command is not None:
            command = DEFAULT_CMD
        if not isinstance(port, int):
            errors.append(f"❌ Port tidak valid: {port} di {name}")
            continue

        simpan(generate_deployment(name, image, port, command, environment, volumes), f"{name}-deployment.yaml", "base")
        simpan(generate_service(name, port), f"{name}-service.yaml", "base")

        secret_obj = generate_secret_from_env(name)
        if secret_obj:
            seal_path = seal_env_to_sealed_secret(secret_obj, name)
            if seal_path:
                print(f"✅ SealedSecret untuk {name} disimpan di {seal_path}")

    if errors:
        print("\n🚫 Error ditemukan saat generate YAML:")
        for e in errors:
            print(" -", e)
        print("❌ Gagal generate karena environment tidak valid.")
        exit(1)

    simpan_yaml({
        "apiVersion": "kustomize.config.k8s.io/v1beta1",
        "kind": "Kustomization",
        "namespace": DEFAULT_NAMESPACE,
        "resources": all_files
    }, os.path.join(overlay_dir, "kustomization.yaml"))

    tls_secret = generate_ingress_tls_secret()
    if tls_secret:
        tls_secret_path = os.path.join(K8S_DIR, "secrets", "ingress-tls.yaml")
        simpan_yaml(tls_secret, tls_secret_path)
        overlay_tls_copy = os.path.join(overlay_dir, "ingress-tls.yaml")
        shutil.copyfile(tls_secret_path, overlay_tls_copy)
        all_files.append("ingress-tls.yaml")

    validate_generated_yaml(overlay_dir)
    sealed_dir = Path(K8S_DIR) / "secrets" / "sealed"
    for sealed_file in sealed_dir.glob("*.yaml"):
        dst = Path(overlay_dir) / sealed_file.name
        shutil.copy(sealed_file, dst)
        all_files.append(sealed_file.name)
    print(f"✅ YAML untuk environment `{ENV}` berhasil digenerate di `{overlay_dir}`")

def seal_env_to_sealed_secret(secret_obj, name):
    from subprocess import run, PIPE

    tmp_plain_path = Path("/tmp") / f"{name}-secret.yaml"
    sealed_out_path = Path(K8S_DIR) / "secrets" / "sealed" / f"{name}-sealed.yaml"

    simpan_yaml(secret_obj, tmp_plain_path)

    result = run([
        "kubeseal",
        "--controller-namespace", "kube-system",
        "--format", "yaml",
        "-o", "yaml",
        "-f", str(tmp_plain_path)
    ], stdout=PIPE, stderr=PIPE)

    if result.returncode != 0:
        print(f"❌ Gagal mengenkripsi secret {name}: {result.stderr.decode().strip()}")
        return None

    sealed_data = yaml.safe_load(result.stdout)
    simpan_yaml(sealed_data, sealed_out_path)
    return sealed_out_path

if __name__ == "__main__":
    main()
