"""
Utilities module for Kubernetes YAML Generator

Contains helper functions for YAML handling, logging, and common operations.
"""

import os
import yaml
import logging
import re
from pathlib import Path
from yaml.representer import SafeRepresenter
from .config import *

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Global errors list
errors = []

# --- YAML Utilities ---
class QuotedString(str): 
    pass

def quoted_scalar(dumper, data):
    """Custom YAML representer for quoted strings"""
    return dumper.represent_scalar("tag:yaml.org,2002:str", data, style='"')

# Register custom representers
yaml.add_representer(QuotedString, quoted_scalar)
yaml.add_representer(str, SafeRepresenter.represent_str)

def simpan_yaml(obj, path):
    """Menyimpan objek ke file YAML dengan formatting yang baik"""
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

    try:
        obj = quote_keys(obj)
        with open(path, "w") as f:
            yaml.dump(obj, f, sort_keys=False, default_flow_style=False, width=120)
        logger.info(f"✅ Berhasil menyimpan YAML ke {path}")
    except Exception as e:
        logger.error(f"❌ Gagal menyimpan YAML ke {path}: {e}")
        raise

def load_docker_compose():
    """Load dan parse docker-compose file dengan error handling yang lebih baik"""
    if not os.path.exists(COMPOSE_FILE):
        raise FileNotFoundError(f"❌ {COMPOSE_FILE} tidak ditemukan.")
    
    try:
        with open(COMPOSE_FILE) as f:
            compose_data = yaml.safe_load(f)
        
        if not compose_data:
            raise ValueError("File docker-compose kosong atau tidak valid")
        
        return compose_data
    except yaml.YAMLError as e:
        raise ValueError(f"❌ Error parsing YAML: {e}")
    except Exception as e:
        raise Exception(f"❌ Gagal memuat docker-compose: {e}")

# --- Port Utilities ---
def extract_port_from_config(config):
    """Extract port dari konfigurasi service dengan validasi yang lebih baik"""
    try:
        # Coba dari expose terlebih dahulu
        expose = config.get("expose", [])
        if expose and len(expose) > 0:
            port_str = str(expose[0])
            if port_str.isdigit():
                return int(port_str)
        
        # Coba dari ports
        ports = config.get("ports", [])
        if ports and len(ports) > 0:
            port_mapping = str(ports[0])
            
            # Handle berbagai format port mapping
            if port_mapping.isdigit():
                return int(port_mapping)
            elif ":" in port_mapping:
                parts = port_mapping.split(":")
                if len(parts) >= 2:
                    target_port = parts[-1]
                    if target_port.isdigit():
                        return int(target_port)
            
            # Jika format tidak dikenali, coba parse sebagai integer
            try:
                return int(port_mapping)
            except ValueError:
                pass
        
        # Fallback ke default port
        logger.warning(f"⚠️ Tidak dapat menentukan port, menggunakan default: {DEFAULT_PORT}")
        return DEFAULT_PORT
        
    except Exception as e:
        logger.error(f"❌ Error extracting port: {e}")
        return DEFAULT_PORT

# --- Service Name Utilities ---
def validate_service_name(service_name):
    """Validasi nama service untuk Kubernetes compatibility"""
    # Kubernetes naming rules: lowercase, alphanumeric, -, .
    if not re.match(r'^[a-z0-9]([a-z0-9\-\.]*[a-z0-9])?$', service_name):
        logger.warning(f"⚠️ Nama service '{service_name}' mungkin tidak kompatibel dengan Kubernetes")
        # Sanitize nama untuk Kubernetes
        sanitized = re.sub(r'[^a-z0-9\-\.]', '-', service_name.lower())
        sanitized = re.sub(r'^[^a-z0-9]|[^a-z0-9]$', '', sanitized)
        return sanitized
    return service_name

def sanitize_service_name_for_path(service_name):
    """Sanitize service name untuk digunakan dalam ingress path"""
    return re.sub(r'[^a-z0-9\-]', '-', service_name.lower())

# --- Volume Utilities ---
def parse_volume_mount(vol, service_name, idx):
    """Parse volume mount dengan support untuk berbagai format"""
    try:
        if isinstance(vol, str):
            if ":" in vol:
                # Format: source:target[:ro]
                parts = vol.split(":")
                if len(parts) >= 2:
                    src = parts[0]
                    dst = parts[1]
                    readonly = len(parts) > 2 and parts[2] == "ro"
                    
                    vol_name = f"{service_name}-vol-{idx}"
                    
                    return {
                        "mount": {
                            "name": vol_name,
                            "mountPath": dst,
                            "readOnly": readonly
                        },
                        "volume": {
                            "name": vol_name,
                            "hostPath": {
                                "path": os.path.abspath(os.path.join(BASE_DIR, src)),
                                "type": "DirectoryOrCreate"
                            }
                        }
                    }
            else:
                # Named volume
                vol_name = f"{service_name}-{vol}-{idx}"
                return {
                    "mount": {
                        "name": vol_name,
                        "mountPath": f"/{vol}"
                    },
                    "volume": {
                        "name": vol_name,
                        "emptyDir": {}
                    }
                }
        elif isinstance(vol, dict):
            # Volume dengan konfigurasi kompleks
            vol_name = f"{service_name}-vol-{idx}"
            mount_config = {
                "name": vol_name,
                "mountPath": vol.get("target", f"/{vol_name}")
            }
            
            if vol.get("read_only"):
                mount_config["readOnly"] = True
            
            volume_config = {"name": vol_name}
            
            if "host_path" in vol:
                volume_config["hostPath"] = {
                    "path": vol["host_path"],
                    "type": vol.get("type", "DirectoryOrCreate")
                }
            elif "empty_dir" in vol:
                volume_config["emptyDir"] = {}
            else:
                volume_config["emptyDir"] = {}
            
            return {
                "mount": mount_config,
                "volume": volume_config
            }
        
        return None
    except Exception as e:
        logger.error(f"❌ Error parsing volume {vol} untuk service {service_name}: {e}")
        return None

# --- Health Check Utilities ---
def parse_health_check_url(test_cmd):
    """Parse health check URL dari test command"""
    health_path = DEFAULT_HEALTH_PATH
    
    for item in test_cmd:
        if isinstance(item, str) and ("http://" in item or "https://" in item):
            try:
                # Parse URL untuk mendapatkan path
                if "http://" in item:
                    url_part = item.split("http://")[1]
                else:
                    url_part = item.split("https://")[1]
                
                if "/" in url_part:
                    health_path = "/" + url_part.split("/", 1)[1]
                else:
                    health_path = "/"
            except Exception as e:
                logger.warning(f"⚠️ Error parsing healthcheck URL: {e}")
    
    return health_path

def create_health_check_probe(healthcheck, port):
    """Create health check probe configuration"""
    test_cmd = healthcheck.get("test", [])
    
    if not test_cmd or not isinstance(test_cmd, list) or len(test_cmd) == 0:
        return None
    
    first_cmd = test_cmd[0]
    
    # HTTP healthcheck
    if "curl" in first_cmd or "wget" in first_cmd:
        health_path = parse_health_check_url(test_cmd)
        
        return {
            "httpGet": {
                "path": health_path,
                "port": port
            },
            "initialDelaySeconds": int(healthcheck.get("interval", DEFAULT_HEALTH_CHECK_INTERVAL).rstrip('s') or 30),
            "periodSeconds": int(healthcheck.get("interval", DEFAULT_HEALTH_CHECK_INTERVAL).rstrip('s') or 30),
            "timeoutSeconds": int(healthcheck.get("timeout", DEFAULT_HEALTH_CHECK_TIMEOUT).rstrip('s') or 5),
            "failureThreshold": healthcheck.get("retries", DEFAULT_HEALTH_CHECK_RETRIES)
        }
    
    # TCP healthcheck
    elif first_cmd == "CMD-SHELL" and len(test_cmd) > 1:
        tcp_cmd = test_cmd[1]
        if "nc" in tcp_cmd or "telnet" in tcp_cmd:
            return {
                "tcpSocket": {
                    "port": port
                },
                "initialDelaySeconds": int(healthcheck.get("interval", DEFAULT_HEALTH_CHECK_INTERVAL).rstrip('s') or 30),
                "periodSeconds": int(healthcheck.get("interval", DEFAULT_HEALTH_CHECK_INTERVAL).rstrip('s') or 30),
                "timeoutSeconds": int(healthcheck.get("timeout", DEFAULT_HEALTH_CHECK_TIMEOUT).rstrip('s') or 5),
                "failureThreshold": healthcheck.get("retries", DEFAULT_HEALTH_CHECK_RETRIES)
            }
    
    return None

# --- Error Handling ---
def add_error(error_msg):
    """Add error to global errors list"""
    errors.append(error_msg)

def get_errors():
    """Get all errors"""
    return errors.copy()

def clear_errors():
    """Clear all errors"""
    global errors
    errors = []

def has_errors():
    """Check if there are any errors"""
    return len(errors) > 0 