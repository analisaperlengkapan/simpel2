#!/usr/bin/env python3
"""
SIMPelv2 Generator CLI - Main Entry Point

This script generates Kubernetes manifests, Docker Compose files, and GitLab CI/CD pipelines.
Usage:
  python3 scripts/generate.py compose         # Generate Compose only
  python3 scripts/generate.py k8s             # Generate K8s only
  python3 scripts/generate.py all             # Generate Compose, K8s, and GitLab CI
  python3 scripts/generate.py gitlab-ci       # Generate GitLab CI only
  python3 scripts/generate.py gitlab-ci-main  # Generate main GitLab CI only
  python3 scripts/generate.py gitlab-ci-service # Generate service-specific CI only
"""

import sys
import os
import argparse
import subprocess
from pathlib import Path

# Add the scripts directory to Python path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Path docker compose utama selalu di root
DOCKER_COMPOSE_MAIN = Path('docker-compose.yml')
DOCKER_COMPOSE_VAULT = Path('infra/docker-compose.vault.yml')

if not DOCKER_COMPOSE_MAIN.exists():
    print(f"[ERROR] File docker-compose.yml utama tidak ditemukan di root: {DOCKER_COMPOSE_MAIN.resolve()}")
    exit(1)

# Tentukan endpoint Vault berdasarkan mode
VAULT_MODE = os.environ.get('VAULT_MODE', 'compose')
if VAULT_MODE == 'k8s':
    VAULT_ADDR = 'http://vault.simpelv2.svc:8200'  # service DNS di K8s
else:
    # Untuk Docker Compose dev, gunakan host.docker.internal (WSL2) atau localhost
    VAULT_ADDR = 'http://host.docker.internal:8200' if 'WSL2' in os.uname().release else 'http://localhost:8200'
    # Otomatis pastikan Vault Compose sudah running
    result = subprocess.run(["docker", "compose", "-f", "infra/docker-compose.vault.yml", "ps", "-q", "vault"], capture_output=True, text=True)
    if not result.stdout.strip():
        print("[INFO] Vault Compose belum aktif, menjalankan infra/docker-compose.vault.yml ...")
        subprocess.run(["docker", "compose", "-f", "infra/docker-compose.vault.yml", "up", "-d"])
    else:
        print("[INFO] Vault Compose sudah aktif.")

# Pastikan semua env/manifest yang butuh VAULT_ADDR menggunakan variabel ini
# Contoh:
# env['VAULT_ADDR'] = VAULT_ADDR

def main():
    parser = argparse.ArgumentParser(description="SIMPelv2 Generator CLI", add_help=True)
    subparsers = parser.add_subparsers(dest="command")

    # Subcommand: compose
    subparsers.add_parser("compose", help="Generate Docker Compose, nginx, envoy")
    
    # Subcommand: k8s
    subparsers.add_parser("k8s", help="Generate Kubernetes manifests")
    
    # Subcommand: all
    subparsers.add_parser("all", help="Generate Compose, K8s, dan GitLab CI sekaligus")
    
    # Subcommand: gitlab-ci (legacy)
    subparsers.add_parser("gitlab-ci", help="Generate GitLab CI/CD pipeline (legacy)")
    
    # New subcommand: gitlab-ci-main
    subparsers.add_parser("gitlab-ci-main", help="Generate main GitLab CI/CD pipeline (.gitlab-ci.yml)")
    
    # New subcommand: gitlab-ci-service
    subparsers.add_parser("gitlab-ci-service", help="Generate service-specific GitLab CI/CD pipelines")

    # Global options (backward compatible)
    parser.add_argument(
        "--env",
        choices=["dev", "staging", "prod"],
        default=None,
        help="Environment to generate for (default: dev)"
    )
    parser.add_argument(
        "--tag",
        help="Docker image tag to use (default: latest for dev/staging, VERSION env var for prod)"
    )
    parser.add_argument(
        "--version",
        action="version",
        version="SIMPelv2 Generator v2.0.0"
    )
    # Secret manager integration
    parser.add_argument(
        "--secret-provider",
        choices=["vault", "aws", "gcp", "azure"],
        default=None,
        help="Provider secret manager (vault/aws/gcp/azure)"
    )
    parser.add_argument(
        "--secret-path",
        default=None,
        help="Path/key secret di provider"
    )
    parser.add_argument(
        "--secret-token",
        default=None,
        help="Token untuk HashiCorp Vault"
    )
    parser.add_argument(
        "--secret-url",
        default=None,
        help="Base URL untuk HashiCorp Vault"
    )
    # GitLab CI specific options
    parser.add_argument(
        "--ci-config",
        default=None,
        help="GitLab CI configuration file (JSON format)"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Generate configuration without writing to file"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Force overwrite without confirmation"
    )

    args, unknown = parser.parse_known_args()

    # Set environment variables from arguments (for all subcommands)
    if args.env:
        os.environ["ENV"] = args.env
    if args.tag:
        os.environ["TAG"] = args.tag

    command = args.command or "k8s"

    try:
        if command == "compose":
            from generator.compose_generator import generate_compose_main
            success = generate_compose_main(args=args)
            sys.exit(0 if success else 1)
            
        elif command == "k8s":
            from generator.generators import main as generate_k8s_main
            success = generate_k8s_main(args=args)
            sys.exit(0 if success else 1)
            
        elif command == "all":
            from generator.compose_generator import generate_compose_main
            from generator.generators import main as generate_k8s_main
            from generator.gitlab_ci_generator import generate_ci
            
            # Jalankan Compose
            compose_ok = generate_compose_main(args=args)
            # Jalankan K8s
            k8s_ok = generate_k8s_main(args=args)
            # Jalankan GitLab CI (main + service)
            gitlab_ok = generate_ci(
                config_file=getattr(args, 'ci_config', None),
                dry_run=getattr(args, 'dry_run', False),
                force=getattr(args, 'force', False)
            )
            
            sys.exit(0 if compose_ok and k8s_ok and gitlab_ok else 1)
            
        elif command == "gitlab-ci":
            # Legacy command - generate both main and service CI
            from generator.gitlab_ci_generator import generate_ci
            success = generate_ci(
                config_file=getattr(args, 'ci_config', None),
                dry_run=getattr(args, 'dry_run', False),
                force=getattr(args, 'force', False)
            )
            sys.exit(0 if success else 1)
            
        elif command == "gitlab-ci-main":
            # Generate only main GitLab CI
            from generator.gitlab_ci_generator import generate_main_ci
            success = generate_main_ci(
                config_file=getattr(args, 'ci_config', None),
                dry_run=getattr(args, 'dry_run', False),
                force=getattr(args, 'force', False)
            )
            sys.exit(0 if success else 1)
            
        elif command == "gitlab-ci-service":
            # Generate only service-specific GitLab CI
            from generator.gitlab_ci_generator import generate_service_ci
            from generator.utils import load_docker_compose
            
            # Load services from docker-compose
            compose = load_docker_compose()
            services = compose.get("services", {})
            
            if not services:
                print("❌ No services found in docker-compose")
                sys.exit(1)
            
            success = generate_service_ci(
                services=services,
                config_file=getattr(args, 'ci_config', None),
                dry_run=getattr(args, 'dry_run', False)
            )
            sys.exit(0 if success else 1)
            
        else:
            print("❌ Unknown command. Use --help for usage.")
            sys.exit(1)
            
    except KeyboardInterrupt:
        print("\n❌ Operation cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Fatal error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 