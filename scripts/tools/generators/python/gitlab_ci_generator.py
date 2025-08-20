"""
GitLab CI/CD Pipeline Generator

Mendukung:
- GitLab CI induk di root (.gitlab-ci.yml)
- GitLab CI per layanan (service-specific CI)
- Multi-environment deployment
- Security scanning integration
- Vault integration
"""

import os
import yaml
import json
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
from .config import *
from .utils import *

logger = logging.getLogger(__name__)

# Default configuration
DEFAULT_CI_CONFIG = {
    "ci_file": ".gitlab-ci.yml",
    "base_image": "python:3.10",
    "base_dir": "/var/www/simpelv2",
    "compose_file": "docker-compose.secure.yml",
    "backup_retention_days": 30,
    "security_includes": [
        {"template": "Security/SAST.gitlab-ci.yml"},
        {"template": "Security/Dependency-Scanning.gitlab-ci.yml"},
        {"template": "Security/Container-Scanning.gitlab-ci.yml"},
        {"template": "Security/License-Management.gitlab-ci.yml"},
        {"template": "Security/Secret-Detection.gitlab-ci.yml"},
        {"template": "Security/DAST.gitlab-ci.yml"},
        {"template": "Security/SCA.gitlab-ci.yml"}
    ],
    "stages": [
        "lint", "validate", "build", "seal", "test", "deploy", "release", "cleanup"
    ],
    "deployment_config": {
        "dev": {"branch": "main", "manual": True, "rollback": True},
        "staging": {"branch": "/^release\\/.*$/", "manual": True, "rollback": True},
        "prod": {"branch": "tags", "manual": True, "rollback": True}
    }
}

def load_ci_config(config_file: Optional[str] = None) -> Dict[str, Any]:
    """Load CI configuration dari file atau default"""
    if config_file and os.path.exists(config_file):
        try:
            with open(config_file, 'r') as f:
                config = json.load(f)
                logger.info(f"✅ Loaded CI config dari {config_file}")
                return {**DEFAULT_CI_CONFIG, **config}
        except Exception as e:
            logger.error(f"❌ Error loading config {config_file}: {e}")
            return DEFAULT_CI_CONFIG
    else:
        logger.info("📝 Using default CI configuration")
        return DEFAULT_CI_CONFIG

def backup_ci_file(ci_file: str) -> bool:
    """Backup existing CI file"""
    if not os.path.exists(ci_file):
        return True
    
    try:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_file = f"{ci_file}.backup.{timestamp}"
        shutil.copy2(ci_file, backup_file)
        logger.info(f"✅ Backup created: {backup_file}")
        
        # Cleanup old backups
        cleanup_old_backups(ci_file)
        return True
    except Exception as e:
        logger.error(f"❌ Error creating backup: {e}")
        return False

def cleanup_old_backups(ci_file: str, retention_days: int = 30):
    """Cleanup old backup files"""
    try:
        backup_dir = os.path.dirname(ci_file) or "."
        backup_pattern = f"{os.path.basename(ci_file)}.backup.*"
        
        cutoff_date = datetime.now() - timedelta(days=retention_days)
        
        for backup_file in Path(backup_dir).glob(backup_pattern):
            file_time = datetime.fromtimestamp(backup_file.stat().st_mtime)
            if file_time < cutoff_date:
                backup_file.unlink()
                logger.info(f"🗑️ Cleaned up old backup: {backup_file}")
    except Exception as e:
        logger.warning(f"⚠️ Error cleaning up backups: {e}")

def build_main_ci_config(config: Dict[str, Any]) -> Dict[str, Any]:
    """Build main CI configuration"""
    ci_config = {
        "include": config["security_includes"],
        "stages": config["stages"],
        "variables": {
            "BASE_DIR": config["base_dir"],
            "COMPOSE_FILE": config["compose_file"],
            "ENV": "${CI_COMMIT_REF_NAME}",
            "TAG": "${CI_COMMIT_TAG}"
        },
        "default": {
            "image": config["base_image"],
            "before_script": [
                "apt-get update && apt-get install -y curl docker.io yamllint git make",
                "pip install python-dotenv pyyaml ruamel.yaml"
            ]
        },
        "cache": {
            "paths": [".cache/pip", "backend/pkg/mod"]
        }
    }
    
    # Add jobs
    ci_config.update(build_ci_jobs(config))
    
    return ci_config

def build_ci_jobs(config: Dict[str, Any]) -> Dict[str, Any]:
    """Build CI jobs"""
    jobs = {}
    
    # Lint jobs
    jobs.update({
        "lint:yaml": {
            "stage": "lint",
            "script": [
                "echo \"🔍 Melakukan lint YAML...\"",
                "make lint-yaml"
            ]
        }
    })
    
    # Validate jobs
    jobs.update({
        "validate:all": {
            "stage": "validate",
            "script": [
                "echo \"✅ Validasi semua konfigurasi...\"",
                "make validate-env",
                "make validate-all"
            ]
        }
    })
    
    # Build jobs
    jobs.update({
        "generate:k8s": {
            "stage": "build",
            "script": [
                "echo \"🔧 Generate YAML K8s dari docker-compose...\"",
                "make generate-k8s"
            ]
        }
    })
    
    # Seal jobs
    jobs.update({
        "seal:secrets": {
            "stage": "seal",
            "script": [
                "echo \"🔐 Install kubeseal...\"",
                "curl -sL https://github.com/bitnami-labs/sealed-secrets/releases/latest/download/kubeseal-linux-amd64 -o kubeseal",
                "chmod +x kubeseal && mv kubeseal /usr/local/bin/kubeseal",
                "echo \"🔐 Menyegel semua secret layanan...\"",
                "make seal-secret",
                "echo \"🔍 Validasi hasil SealedSecrets...\"",
                "make validate-sealed"
            ]
        }
    })
    
    # Test jobs
    jobs.update({
        "test:simulate": {
            "stage": "test",
            "script": [
                "echo \"🧪 Simulasi pod run semua image...\"",
                "make simulate-pod-run"
            ]
        },
        "coverage:go": {
            "stage": "test",
            "script": [
                "cd backend || exit 0",
                "go test -coverprofile=coverage.out ./...",
                "go tool cover -func=coverage.out"
            ],
            "coverage": "/total:\\s+\\(statements\\)\\s+(\\d+\\.\\d+%)/",
            "allow_failure": True
        }
    })
    
    # Deploy jobs
    for env, env_config in config["deployment_config"].items():
        jobs.update({
            f"deploy:{env}": {
                "stage": "deploy",
                "script": [
                    f"echo \"🚀 Deploy ke {env}...\"",
                    f"make deploy-{env}"
                ],
                "environment": {
                    "name": env,
                    "url": f"https://{env}.simpelv2.local"
                },
                "when": "manual" if env_config.get("manual", False) else "on_success",
                "only": [env_config["branch"]]
            }
        })
    
    # Release jobs
    jobs.update({
        "release:tag": {
            "stage": "release",
            "script": [
                "echo \"🏷️ Tagging release...\"",
                "git tag -a $CI_COMMIT_TAG -m \"Release $CI_COMMIT_TAG\"",
                "git push origin $CI_COMMIT_TAG"
            ],
            "only": ["tags"],
            "when": "manual"
        }
    })
    
    # Cleanup jobs
    jobs.update({
        "cleanup:old-backups": {
            "stage": "cleanup",
            "script": [
                "echo \"🧹 Cleanup old backups...\"",
                "find . -name '*.backup.*' -mtime +30 -delete"
            ],
            "when": "manual"
        }
    })
    
    return jobs

def build_service_ci_config(service_name: str, service_config: Dict[str, Any], config: Dict[str, Any]) -> Dict[str, Any]:
    """Build service-specific CI configuration"""
    service_ci_file = f".gitlab-ci.{service_name}.yml"
    
    # Determine service type
    service_type = detect_service_type(service_config)
    
    service_ci = {
        "stages": ["build", "test", "deploy"],
        "variables": {
            "SERVICE_NAME": service_name,
            "SERVICE_TYPE": service_type,
            "BASE_DIR": config["base_dir"]
        },
        "default": {
            "image": get_service_image(service_type),
            "before_script": get_service_before_script(service_type)
        }
    }
    
    # Add service-specific jobs
    service_ci.update(build_service_jobs(service_name, service_config, service_type))
    
    return service_ci

def detect_service_type(service_config: Dict[str, Any]) -> str:
    """Detect service type based on configuration"""
    if "build" in service_config:
        if "context" in service_config["build"]:
            context = service_config["build"]["context"]
            if "antarmuka" in context:
                return "frontend"
            elif "backend" in context or "layanan" in context:
                return "backend"
    return "generic"

def get_service_image(service_type: str) -> str:
    """Get appropriate base image for service type"""
    images = {
        "frontend": "node:18-alpine",
        "backend": "golang:1.21-alpine",
        "generic": "python:3.10"
    }
    return images.get(service_type, "python:3.10")

def get_service_before_script(service_type: str) -> List[str]:
    """Get before_script for service type"""
    scripts = {
        "frontend": [
            "apk add --no-cache git make",
            "npm install -g npm@latest"
        ],
        "backend": [
            "apk add --no-cache git make",
            "go version"
        ],
        "generic": [
            "apt-get update && apt-get install -y curl git make",
            "pip install python-dotenv pyyaml"
        ]
    }
    return scripts.get(service_type, scripts["generic"])

def build_service_jobs(service_name: str, service_config: Dict[str, Any], service_type: str) -> Dict[str, Any]:
    """Build service-specific jobs"""
    jobs = {}
    
    # Build job
    jobs.update({
        f"build:{service_name}": {
            "stage": "build",
            "script": [
                f"echo \"🔨 Building {service_name}...\"",
                f"docker build -t {service_name}:$CI_COMMIT_SHA .",
                f"docker tag {service_name}:$CI_COMMIT_SHA {service_name}:latest"
            ],
            "artifacts": {
                "paths": [f"dist/{service_name}/"],
                "expire_in": "1 week"
            }
        }
    })
    
    # Test job
    jobs.update({
        f"test:{service_name}": {
            "stage": "test",
            "script": [
                f"echo \"🧪 Testing {service_name}...\"",
                f"make test-{service_name}"
            ],
            "allow_failure": True
        }
    })
    
    # Deploy job
    jobs.update({
        f"deploy:{service_name}": {
            "stage": "deploy",
            "script": [
                f"echo \"🚀 Deploying {service_name}...\"",
                f"make deploy-{service_name}"
            ],
            "environment": {
                "name": f"{service_name}-dev",
                "url": f"https://{service_name}.dev.simpelv2.local"
            },
            "when": "manual"
        }
    })
    
    return jobs

def validate_ci_config(ci_config: Dict[str, Any]) -> bool:
    """Validate CI configuration"""
    try:
        # Test YAML generation
        yaml.dump(ci_config, sort_keys=False, default_flow_style=False)
        
        # Validate required fields
        required_fields = ["stages", "variables", "default"]
        for field in required_fields:
            if field not in ci_config:
                logger.error(f"❌ Missing required field: {field}")
                return False
        
        logger.info("✅ CI configuration validation passed")
        return True
    except Exception as e:
        logger.error(f"❌ CI configuration validation failed: {e}")
        return False

def test_ci_config(ci_config: Dict[str, Any]) -> bool:
    """Test CI configuration by generating YAML"""
    try:
        yaml.dump(ci_config, sort_keys=False, default_flow_style=False)
        logger.info("✅ YAML generation test passed")
        return True
    except Exception as e:
        logger.error(f"❌ YAML generation test failed: {e}")
        return False

def generate_main_ci(config_file: Optional[str] = None, dry_run: bool = False, force: bool = False) -> bool:
    """Generate main GitLab CI configuration"""
    try:
        # Load configuration
        config = load_ci_config(config_file)
        
        # Build CI configuration
        ci_config = build_main_ci_config(config)
        
        # Validate and test
        if not validate_ci_config(ci_config):
            return False
        
        if not test_ci_config(ci_config):
            return False
        
        # Backup existing file
        ci_file = config["ci_file"]
        if not dry_run and os.path.exists(ci_file):
            if not force:
                response = input(f"File {ci_file} exists. Overwrite? (y/N): ")
                if response.lower() != 'y':
                    logger.info("❌ Operation cancelled")
                    return False
            if not backup_ci_file(ci_file):
                return False
        
        # Write configuration
        if not dry_run:
            with open(ci_file, 'w') as f:
                yaml.dump(ci_config, f, sort_keys=False, default_flow_style=False)
            logger.info(f"✅ Main CI configuration written to {ci_file}")
        else:
            logger.info("🔍 Dry run - CI configuration would be written")
            logger.info(f"Configuration: {json.dumps(ci_config, indent=2)}")
        
        return True
        
    except Exception as e:
        logger.error(f"❌ Error generating main CI: {e}")
        return False

def backup_service_ci_file(service_ci_file: str) -> bool:
    """Backup service CI file sebelum overwrite"""
    try:
        if os.path.exists(service_ci_file):
            # Buat backup dengan timestamp
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            backup_file = f"{service_ci_file}.backup.{timestamp}"
            
            shutil.copy2(service_ci_file, backup_file)
            logger.info(f"✅ Backup created: {backup_file}")
            
            # Cleanup old backups (retention 30 hari)
            cleanup_old_backups(service_ci_file, 30)
            
        return True
    except Exception as e:
        logger.error(f"❌ Error backing up {service_ci_file}: {e}")
        return False

def generate_service_ci(services: Dict[str, Any], config_file: Optional[str] = None, dry_run: bool = False) -> bool:
    """Generate service-specific CI configurations di folder masing-masing service"""
    try:
        config = load_ci_config(config_file)
        
        for service_name, service_config in services.items():
            logger.info(f"🔧 Generating CI for service: {service_name}")
            
            # Build service CI
            service_ci = build_service_ci_config(service_name, service_config, config)
            
            # Validate
            if not validate_ci_config(service_ci):
                logger.error(f"❌ Service CI validation failed for {service_name}")
                continue
            
            # Tentukan path service berdasarkan build context
            service_path = None
            if 'build' in service_config and 'context' in service_config['build']:
                service_path = service_config['build']['context']
                
                # Validasi path benar-benar ada
                if not os.path.exists(service_path):
                    logger.warning(f"⚠️ Build context path tidak ada: {service_path}, skip service {service_name}")
                    continue
                    
                # Skip folder yang bukan service individual
                if service_path in ['.', './antarmuka', './layanan']:
                    logger.warning(f"⚠️ Skip folder non-service: {service_path}")
                    continue
                    
            else:
                # Fallback: cari folder service dengan validasi ketat
                for root_dir in ['.', 'antarmuka', 'layanan']:
                    potential_path = os.path.join(root_dir, service_name.replace('-', '_'))
                    if os.path.exists(potential_path) and os.path.isdir(potential_path):
                        # Validasi ini adalah service folder (ada Dockerfile atau package.json)
                        service_files = ['Dockerfile', 'package.json', 'Cargo.toml', 'go.mod']
                        if any(os.path.exists(os.path.join(potential_path, f)) for f in service_files):
                            service_path = potential_path
                            break
                
                if not service_path:
                    logger.warning(f"⚠️ Tidak dapat menentukan path valid untuk service {service_name}, skip.")
                    continue
            
            # Validasi final: pastikan ini bukan folder parent
            if service_path in ['.', './antarmuka', './layanan', 'antarmuka', 'layanan']:
                logger.warning(f"⚠️ Skip folder parent: {service_path}")
                continue
            
            # Write service CI file di folder service
            service_ci_file = os.path.join(service_path, ".gitlab-ci.yml")
            
            if not dry_run:
                # Backup file lama jika ada
                if not backup_service_ci_file(service_ci_file):
                    logger.warning(f"⚠️ Gagal backup {service_ci_file}, lanjutkan tanpa backup")
                
                # Buat direktori jika tidak ada
                os.makedirs(service_path, exist_ok=True)
                
                with open(service_ci_file, 'w') as f:
                    yaml.dump(service_ci, f, sort_keys=False, default_flow_style=False)
                logger.info(f"✅ Service CI written to {service_ci_file}")
            else:
                logger.info(f"🔍 Dry run - Service CI would be written to {service_ci_file}")
        
        return True
        
    except Exception as e:
        logger.error(f"❌ Error generating service CI: {e}")
        return False

def generate_ci(config_file: Optional[str] = None, dry_run: bool = False, force: bool = False) -> bool:
    """Generate both main and service-specific CI configurations"""
    try:
        # Load services from docker-compose
        compose = load_docker_compose()
        services = compose.get("services", {})
        
        if not services:
            logger.error("❌ No services found in docker-compose")
            return False
        
        # Generate main CI
        main_ok = generate_main_ci(config_file, dry_run, force)
        if not main_ok:
            return False
        
        # Generate service CI
        service_ok = generate_service_ci(services, config_file, dry_run)
        if not service_ok:
            return False
        
        logger.info("✅ All CI configurations generated successfully")
        return True
        
    except Exception as e:
        logger.error(f"❌ Error generating CI configurations: {e}")
        return False

def parse_arguments():
    """Parse command line arguments"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Generate GitLab CI/CD pipeline configuration",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 scripts/generate.py gitlab-ci                    # Generate with default config
  python3 scripts/generate.py gitlab-ci --config ci.json   # Generate with custom config
  python3 scripts/generate.py gitlab-ci --dry-run          # Test without writing file
        """
    )
    
    parser.add_argument("--config", help="Configuration file (JSON format)")
    parser.add_argument("--dry-run", action="store_true", help="Generate configuration without writing to file")
    parser.add_argument("--force", action="store_true", help="Force overwrite without confirmation")
    parser.add_argument("--version", action="version", version="GitLab CI Generator v2.0.0")
    
    return parser.parse_args() 