"""
SIMPelv2 Generator Package

Modul untuk generate Kubernetes manifests, Docker Compose, dan GitLab CI/CD pipelines.
"""

from .config import *
from .utils import *
from .generators import *
from .compose_generator import *
from .secret_manager import *
from .gitlab_ci_generator import *

__version__ = "2.0.0"
__author__ = "SIMPelv2 Team"

# Export main functions
__all__ = [
    # Config
    "DEFAULT_NAMESPACE", "ENVIRONMENTS", "TAG", "DOMAIN",
    
    # Utils
    "simpan_yaml", "load_docker_compose", "extract_port_from_config",
    "validate_service_name", "parse_volume_mount", "create_health_check_probe",
    
    # Generators
    "generate_namespace", "generate_deployment", "generate_service", 
    "generate_ingress", "generate_secret_from_env", "generate_db_pvc",
    
    # Compose Generator
    "generate_compose_main", "generate_nginx_conf", "generate_envoy_yaml",
    
    # Secret Manager
    "get_secret", "get_secret_vault",
    
    # GitLab CI Generator
    "generate_ci", "generate_main_ci", "generate_service_ci", "parse_arguments"
] 