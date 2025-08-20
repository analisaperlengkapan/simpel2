"""
Generators module for Kubernetes YAML Generator

Contains all functions for generating Kubernetes manifests.
"""

import os
import base64
import subprocess
from pathlib import Path
from dotenv import dotenv_values
from .config import *
from .utils import *

def generate_namespace():
    """Generate namespace manifest"""
    return {
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {"name": DEFAULT_NAMESPACE}
    }

def generate_secret_from_env(service_name):
    """Generate secret dari file .env dengan error handling yang lebih baik"""
    env_path = os.path.join(BASE_DIR, service_name, ".env")
    
    # Fallback untuk db-simpelv2
    if service_name == "db-simpelv2" and not os.path.exists(env_path):
        env_path = os.path.join(BASE_DIR, ".env")
    
    if not os.path.exists(env_path):
        logger.info(f"⚠️ Tidak ditemukan .env untuk {service_name}, lewati secret.")
        return None

    try:
        env_data_raw = dotenv_values(env_path)
        env_data = {k: str(v) for k, v in env_data_raw.items() if v is not None}
        
        if not env_data:
            logger.info(f"⚠️ File .env kosong untuk {service_name}, lewati secret.")
            return None

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
    except Exception as e:
        logger.error(f"❌ Error generating secret untuk {service_name}: {e}")
        return None

def generate_ingress_tls_secret():
    """Generate TLS secret dengan validasi yang lebih baik"""
    cert_path = Path(BASE_DIR) / "nginx" / "certs" / "fullchain.pem"
    key_path = Path(BASE_DIR) / "nginx" / "certs" / "privkey.pem"

    if not cert_path.exists() or not key_path.exists():
        logger.info("⚠️ TLS cert/key tidak ditemukan, lewati TLS secret.")
        return None

    try:
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
    except Exception as e:
        logger.error(f"❌ Error generating TLS secret: {e}")
        return None

def generate_db_pvc():
    """Generate Persistent Volume Claim untuk database"""
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

def generate_deployment(service_name, config):
    """Generate deployment dengan error handling dan validasi yang lebih baik"""
    try:
        # Validasi nama service
        service_name = validate_service_name(service_name)
        
        # Penentuan image
        image = config.get("image", f"simpelv2/{service_name}:{TAG}")
        if image.startswith("simpelv2/") and ":" not in image:
            image = f"{image}:{TAG}"
        
        # Penentuan port dengan fungsi yang lebih robust
        port = extract_port_from_config(config)
        
        # Penentuan command
        command = config.get("command")
        if service_name == "db-simpelv2":
            port = DB_PORT
            command = None
        elif service_name in SKIP_COMMAND_SERVICES:
            command = None
        
        # Penanganan volume yang lebih robust
        volume_mounts = []
        volume_defs = []
        volumes = config.get("volumes", [])
        
        # Skip volume untuk service tertentu
        if service_name in SKIP_VOLUME_SERVICES:
            volumes = []
        
        # Handle volumes yang bisa berupa list atau dict
        if isinstance(volumes, list):
            for idx, vol in enumerate(volumes):
                parsed_volume = parse_volume_mount(vol, service_name, idx)
                if parsed_volume:
                    volume_mounts.append(parsed_volume["mount"])
                    volume_defs.append(parsed_volume["volume"])
                else:
                    add_error(f"❌ Format volume tidak dikenali: {vol} di service {service_name}")
        elif isinstance(volumes, dict):
            # Handle volume dict format
            for vol_name, vol_config in volumes.items():
                parsed_volume = parse_volume_mount(vol_config, service_name, vol_name)
                if parsed_volume:
                    volume_mounts.append(parsed_volume["mount"])
                    volume_defs.append(parsed_volume["volume"])
                else:
                    add_error(f"❌ Format volume tidak dikenali: {vol_config} di service {service_name}")
        else:
            logger.warning(f"⚠️ Format volumes tidak dikenali untuk {service_name}: {type(volumes)}")
        
        # Penanganan environment yang lebih robust
        env_items = []
        environment = config.get("environment", {})
        
        # Handle environment yang bisa berupa list atau dict
        if isinstance(environment, list):
            # Convert list format ["KEY=value"] ke dict
            env_dict = {}
            for env_item in environment:
                if isinstance(env_item, str) and "=" in env_item:
                    key, value = env_item.split("=", 1)
                    env_dict[key] = value
                else:
                    logger.warning(f"⚠️ Format environment tidak dikenali: {env_item}")
            environment = env_dict
        elif not isinstance(environment, dict):
            logger.warning(f"⚠️ Format environment tidak dikenali untuk {service_name}: {type(environment)}")
            environment = {}
        
        # Tambahkan environment dari .env jika ada
        env_path = os.path.join(BASE_DIR, service_name, ".env")
        if os.path.exists(env_path):
            try:
                dotenv_env = dotenv_values(env_path)
                environment.update({k: str(v) for k, v in dotenv_env.items() if v is not None})
            except Exception as e:
                logger.warning(f"⚠️ Error loading .env untuk {service_name}: {e}")
        
        for k, v in environment.items():
            if isinstance(v, (list, dict)):
                add_error(f"❌ ENV `{k}` di service `{service_name}` invalid: {v} ({type(v).__name__})")
                continue
            env_items.append({
                "name": str(k),
                "value": str(v)
            })
        
        # Konfigurasi container
        container_spec = {
            "name": service_name,
            "image": image,
            "imagePullPolicy": get_image_pull_policy(),
            "ports": [{"containerPort": port}],
            "volumeMounts": volume_mounts
        }
        
        if command:
            container_spec["command"] = command
        
        if env_items:
            container_spec["env"] = env_items
        
        # Tambahkan healthcheck yang lebih robust
        if "healthcheck" in config:
            healthcheck = config["healthcheck"]
            probe = create_health_check_probe(healthcheck, port)
            if probe:
                container_spec["livenessProbe"] = probe
        
        # Tambahkan secret jika ada
        secret_obj = generate_secret_from_env(service_name)
        if secret_obj:
            container_spec.setdefault("envFrom", []).append({
                "secretRef": {"name": f"{service_name}-secret"}
            })
        
        return {
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {"name": service_name, "namespace": DEFAULT_NAMESPACE},
            "spec": {
                "replicas": 1,
                "selector": {"matchLabels": {"app": service_name}},
                "template": {
                    "metadata": {"labels": {"app": service_name}},
                    "spec": {
                        "restartPolicy": "Always",
                        "containers": [container_spec],
                        "volumes": volume_defs
                    }
                }
            }
        }
    except Exception as e:
        logger.error(f"❌ Error generating deployment untuk {service_name}: {e}")
        raise

def generate_service(service_name, config):
    """Generate service dengan port handling yang lebih robust"""
    try:
        service_name = validate_service_name(service_name)
        port = extract_port_from_config(config)
        
        service_ports = []
        ports = config.get("ports", [])
        
        if ports:
            for idx, port_mapping in enumerate(ports):
                try:
                    if isinstance(port_mapping, int):
                        service_ports.append({
                            "name": f"port-{idx}",
                            "port": port_mapping,
                            "targetPort": port_mapping
                        })
                    else:
                        parts = str(port_mapping).split(":")
                        if len(parts) >= 2:
                            target = int(parts[-1])
                            service_port = int(parts[0]) if len(parts) == 2 else target
                            service_ports.append({
                                "name": f"port-{idx}",
                                "port": service_port,
                                "targetPort": target
                            })
                except (ValueError, IndexError) as e:
                    logger.warning(f"⚠️ Error parsing port mapping {port_mapping}: {e}")
                    continue
        else:
            service_ports.append({
                "name": "default",
                "port": port,
                "targetPort": port
            })
        
        return {
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {"name": service_name, "namespace": DEFAULT_NAMESPACE},
            "spec": {
                "selector": {"app": service_name},
                "ports": service_ports
            }
        }
    except Exception as e:
        logger.error(f"❌ Error generating service untuk {service_name}: {e}")
        raise

def generate_ingress(services):
    """Generate ingress dengan path generation yang lebih robust"""
    try:
        rules = []
        tls_config = []
        
        # Cari service gerbang untuk dijadikan default
        gerbang_port = DEFAULT_PORT
        if "gerbang" in services:
            gerbang_config = services["gerbang"]
            gerbang_port = extract_port_from_config(gerbang_config)
        
        # Generate path untuk setiap layanan
        for service_name, config in services.items():
            # Skip nginx karena dia bertindak sebagai reverse proxy
            if service_name == "nginx":
                continue
            
            service_name = validate_service_name(service_name)
            port = extract_port_from_config(config)
            
            # Generate path yang lebih robust
            safe_service_name = sanitize_service_name_for_path(service_name)
            rules.append({
                "path": f"/{safe_service_name}(/|$)(.*)",
                "pathType": "Prefix",
                "backend": {
                    "service": {
                        "name": service_name,
                        "port": {"number": port}
                    }
                }
            })
        
        # Aturan default ke gerbang
        rules.append({
            "path": "/(.*)",
            "pathType": "Prefix",
            "backend": {
                "service": {
                    "name": "gerbang",
                    "port": {"number": gerbang_port}
                }
            }
        })
        
        # Konfigurasi TLS jika tersedia
        tls_secret = generate_ingress_tls_secret()
        if tls_secret:
            tls_config = [{
                "hosts": [DOMAIN],
                "secretName": "simpelv2-tls"
            }]
        
        return {
            "apiVersion": "networking.k8s.io/v1",
            "kind": "Ingress",
            "metadata": {
                "name": "simpelv2-ingress",
                "namespace": DEFAULT_NAMESPACE,
                "annotations": {
                    "nginx.ingress.kubernetes.io/rewrite-target": "/$1",
                    "nginx.ingress.kubernetes.io/ssl-redirect": "true" if tls_config else "false"
                }
            },
            "spec": {
                "tls": tls_config,
                "rules": [{
                    "host": DOMAIN,
                    "http": {"paths": rules}
                }]
            }
        }
    except Exception as e:
        logger.error(f"❌ Error generating ingress: {e}")
        raise

def generate_monitoring_resources(services):
    """Generate monitoring resources dengan konfigurasi yang lebih fleksibel"""
    try:
        # Kumpulkan target dari layanan yang memiliki expose/ports
        targets = []
        for name, config in services.items():
            if name in EXCLUDE_MONITORING_SERVICES:
                continue
            
            # Cari port yang digunakan layanan
            port = extract_port_from_config(config)
            
            if port:
                targets.append(f"'{name}:{port}'")
        
        # Format targets menjadi string
        if targets:
            targets_str = ",\n          ".join(targets)
            prometheus_config = f"""
global:
  scrape_interval: {PROMETHEUS_SCRAPE_INTERVAL}
  evaluation_interval: {PROMETHEUS_EVALUATION_INTERVAL}
  scrape_timeout: {PROMETHEUS_SCRAPE_TIMEOUT}

rule_files:
  # - "first_rules.yml"
  # - "second_rules.yml"

scrape_configs:
  - job_name: 'simpelv2'
    static_configs:
      - targets: [{targets_str}]
    metrics_path: {PROMETHEUS_METRICS_PATH}
    scrape_interval: {PROMETHEUS_SCRAPE_INTERVAL}
    scrape_timeout: {PROMETHEUS_SCRAPE_TIMEOUT}
"""
        else:
            prometheus_config = f"""
global:
  scrape_interval: {PROMETHEUS_SCRAPE_INTERVAL}
  evaluation_interval: {PROMETHEUS_EVALUATION_INTERVAL}
  scrape_timeout: {PROMETHEUS_SCRAPE_TIMEOUT}

scrape_configs:
  - job_name: 'simpelv2'
    static_configs:
      - targets: ['gerbang:{DEFAULT_PORT}']
    metrics_path: {PROMETHEUS_METRICS_PATH}
    scrape_interval: {PROMETHEUS_SCRAPE_INTERVAL}
    scrape_timeout: {PROMETHEUS_SCRAPE_TIMEOUT}
"""
        
        prometheus = {
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {"name": "prometheus-config", "namespace": DEFAULT_NAMESPACE},
            "data": {"prometheus.yml": prometheus_config}
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
    editable: true
"""}
        }

        return [prometheus, grafana]
    except Exception as e:
        logger.error(f"❌ Error generating monitoring resources: {e}")
        return []

def seal_env_to_sealed_secret(secret_obj, name):
    """Generate sealed secret dengan error handling yang lebih baik"""
    try:
        tmp_plain_path = Path("/tmp") / f"{name}-secret.yaml"
        sealed_out_path = Path(K8S_DIR) / "secrets" / "sealed" / f"{name}-sealed.yaml"
        
        simpan_yaml(secret_obj, tmp_plain_path)
        
        with open(sealed_out_path, "w") as fout:
            result = subprocess.run([
                "kubeseal",
                "--controller-namespace", "kube-system",
                "--format", "yaml",
                "-f", str(tmp_plain_path)
            ], stdout=fout, stderr=subprocess.PIPE, text=True)
        
        try:
            tmp_plain_path.unlink()
        except Exception as e:
            logger.warning(f"⚠️ Gagal menghapus file sementara {tmp_plain_path}: {e}")

        if result.returncode != 0:
            logger.error(f"❌ Gagal mengenkripsi secret {name}: {result.stderr}")
            return None
            
        logger.info(f"✅ SealedSecret untuk {name} berhasil dibuat")
        return sealed_out_path
    except Exception as e:
        logger.error(f"❌ Error creating sealed secret untuk {name}: {e}")
        return None 

def main(args=None):
    """Main function untuk generators.py - entry point dari generate.py"""
    try:
        # Import main dari main.py untuk backward compatibility
        from .main import main as main_workflow
        return main_workflow()
    except Exception as e:
        logger.error(f"❌ Error in generators main: {e}")
        return False

if __name__ == "__main__":
    logger.info("📦 Generate YAML Kubernetes dimulai...")
    success = main()
    if not success:
        exit(1) 