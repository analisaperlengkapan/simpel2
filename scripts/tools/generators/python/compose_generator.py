"""
Compose Generator module for SIMPelv2

Berisi seluruh logic generator docker-compose, nginx, envoy, summary, dan helper terkait.
Modular dan siap dipanggil dari workflow utama atau CLI.
"""

import os
import yaml
import logging
import sys
import shutil
import socket
import json
from subprocess import run, CalledProcessError

# Setup logging
logging.basicConfig(level=logging.INFO, format='[%(levelname)s] %(message)s')
logger = logging.getLogger(__name__)

# -------------------- Helper Functions --------------------
def load_meta(service_path):
    """Membaca file compose.meta.yaml jika ada, mengembalikan dict meta."""
    meta_file = os.path.join(service_path, 'compose.meta.yaml')
    if os.path.exists(meta_file):
        with open(meta_file) as mf:
            return yaml.safe_load(mf) or {}
    return {}

def is_port_in_use(port):
    """Cek apakah port sudah digunakan di sistem."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        return s.connect_ex(('localhost', port)) == 0

def find_dockerfile(path):
    """Cari Dockerfile secara rekursif di path service."""
    for root, dirs, files in os.walk(path):
        if 'Dockerfile' in files:
            return os.path.relpath(os.path.join(root, 'Dockerfile'), path)
    return None

def detect_frontend_structure(path):
    """Deteksi framework frontend dan output build-nya."""
    if os.path.exists(os.path.join(path, "Cargo.toml")):
        return {"framework": "leptos", "is_ssr": True, "build_output": "target/site"}
    elif os.path.exists(os.path.join(path, "vite.config.ts")):
        return {"framework": "vite", "is_ssr": False, "build_output": "dist"}
    elif os.path.exists(os.path.join(path, "webpack.mix.js")):
        return {"framework": "laravel", "is_ssr": False, "build_output": "public"}
    elif os.path.exists(os.path.join(path, "next.config.js")):
        return {"framework": "nextjs", "is_ssr": True, "build_output": ".next"}
    elif os.path.exists(os.path.join(path, "angular.json")):
        return {"framework": "angular", "is_ssr": False, "build_output": "dist"}
    elif os.path.exists(os.path.join(path, "nuxt.config.js")):
        return {"framework": "nuxtjs", "is_ssr": True, "build_output": ".output/public"}
    return {"framework": "unknown", "is_ssr": False, "build_output": "dist"}

def write_default_dockerfile(path, is_frontend=True, binary_name="your_binary"):
    """Generate Dockerfile default jika tidak ditemukan."""
    template_frontend = '''FROM node:20
WORKDIR /app
COPY . .
RUN npm install
EXPOSE 3000
CMD ["npm", "run", "dev"]'''
    template_backend = f'''FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
COPY --from=builder /app/target/release /app
WORKDIR /app
EXPOSE 7000
CMD ["./{binary_name}"]'''
    content = template_frontend if is_frontend else template_backend
    dockerfile_path = os.path.join(path, 'Dockerfile')
    with open(dockerfile_path, 'w') as df:
        df.write(content)
    logger.warning(f"Generated default Dockerfile: {dockerfile_path}")
    dockerignore_path = os.path.join(path, '.dockerignore')
    if not os.path.exists(dockerignore_path):
        with open(dockerignore_path, 'w') as dfi:
            dfi.write("target\nnode_modules\n.env\n*.log\n")

def generate_healthcheck(port):
    """Generate healthcheck dict untuk service."""
    return {
        "test": ["CMD", "curl", "-f", f"http://localhost:{port}"],
        "interval": "30s",
        "timeout": "10s",
        "retries": 3,
    }

def generate_openapi_stub(service_name, path, meta):
    """Validasi dan generate stub OpenAPI jika openapi.yaml ada."""
    openapi_file = os.path.join(path, "openapi.yaml")
    if not os.path.exists(openapi_file):
        return
    logger.info(f"Validating OpenAPI spec for {service_name}")
    try:
        res = run(["openapi-generator-cli", "validate", "-i", openapi_file])
        if res.returncode != 0:
            logger.error(f"OpenAPI validation failed for {service_name}, skipping stub generation.")
            return
        output_dir = os.path.join(path, "generated")
        language = meta.get("openapi_codegen", "rust")
        logger.info(f"Generating OpenAPI stub for {service_name} ({language})")
        run([
            "openapi-generator-cli", "generate",
            "-i", openapi_file,
            "-g", language,
            "-o", output_dir,
            "--skip-validate-spec"
        ])
    except CalledProcessError as e:
        logger.error(f"Error during OpenAPI stub generation for {service_name}: {e}")

def backup_file(filepath):
    """Backup file sebelum overwrite."""
    if os.path.exists(filepath):
        backup_path = filepath + ".bak"
        shutil.copy(filepath, backup_path)
        logger.info(f"Backup file lama: {filepath} -> {backup_path}")

def validate_yaml(filepath):
    """Validasi file YAML setelah generate."""
    try:
        with open(filepath) as f:
            yaml.safe_load(f)
        logger.info(f"Validasi YAML sukses: {filepath}")
        return True
    except Exception as e:
        logger.error(f"YAML tidak valid di {filepath}: {e}")
        return False

def validate_nginx_conf(conf_path):
    """Validasi file nginx.conf jika binary tersedia."""
    if shutil.which("nginx") and os.path.exists(conf_path):
        try:
            result = run(["nginx", "-t", "-c", os.path.abspath(conf_path)], capture_output=True, text=True)
            if result.returncode == 0:
                logger.info("nginx.conf valid!")
            else:
                logger.error(f"nginx.conf error: {result.stderr}")
        except Exception as e:
            logger.error(f"Gagal validasi nginx.conf: {e}")
    else:
        logger.warning("nginx binary tidak ditemukan atau nginx.conf tidak ada, skip validasi.")

# -------------------- Generator Functions --------------------
def scan_services(base_dir, port_start, default_internal, is_frontend=True, all_service_names=None, summary=None, args=None):
    """Scan folder service (frontend/backend), generate dict service Compose."""
    services = {}
    port = port_start
    used_ports = set()
    
    # Cek apakah direktori ada
    if not os.path.exists(base_dir):
        logger.warning(f"Direktori {base_dir} tidak ditemukan, skip scanning.")
        if summary is not None:
            summary['warnings'].append(f"Direktori {base_dir} tidak ditemukan, skip scanning.")
        return services
    
    for module in sorted(os.listdir(base_dir)):
        path = os.path.join(base_dir, module)
        if not os.path.isdir(path) or module.startswith('.'):
            continue
        meta = load_meta(path) or {}
        # Skip jika disabled di meta
        if meta.get('disabled', False):
            logger.info(f"Service {module} di-skip karena disabled di meta.")
            if summary is not None:
                summary['warnings'].append(f"Service {module} di-skip karena disabled di meta.")
            continue
        # Filtering by include/exclude
        name = module.lower().replace('_', '-')
        if args and hasattr(args, 'include_service') and args.include_service and name not in args.include_service:
            logger.info(f"Service {name} di-skip karena tidak ada di --include-service.")
            if summary is not None:
                summary['warnings'].append(f"Service {name} di-skip karena tidak ada di --include-service.")
            continue
        if args and hasattr(args, 'exclude_service') and name in args.exclude_service:
            logger.info(f"Service {name} di-skip karena ada di --exclude-service.")
            if summary is not None:
                summary['warnings'].append(f"Service {name} di-skip karena ada di --exclude-service.")
            continue
        internal_port = meta.get('internal_port', default_internal)
        dockerfile_name = meta.get('dockerfile', 'Dockerfile')
        binary_name = meta.get("binary", module.replace("-", "_"))
        dockerfile_path = os.path.join(path, dockerfile_name)
        dockerfile_found = True
        if not os.path.exists(dockerfile_path):
            found = find_dockerfile(path)
            if found:
                dockerfile_name = found
            else:
                try:
                    write_default_dockerfile(path, is_frontend, binary_name)
                except Exception as e:
                    dockerfile_found = False
                    msg = f"Dockerfile tidak ditemukan dan gagal dibuat default untuk {name}: {e}"
                    logger.warning(msg)
                    if summary is not None:
                        summary['warnings'].append(msg)
        # Cek konflik port di internal dict
        while port in used_ports or is_port_in_use(port):
            msg = f"Port {port} sudah digunakan (internal/sistem), mencari port berikutnya..."
            logger.warning(msg)
            if summary is not None:
                summary['warnings'].append(msg)
            port += 1
        used_ports.add(port)
        svc = {
            "build": {"context": path, "dockerfile": dockerfile_name},
            "ports": [f"{port}:{internal_port}"],
            "networks": ["simpelv2", "internal"],
            "labels": {
                "simpelv2.role": "frontend" if is_frontend else "backend",
                "simpelv2.name": name
            }
        }
        # Build args & platform dari meta
        if 'build_args' in meta:
            svc['build']['args'] = meta['build_args']
        if 'platform' in meta:
            svc['build']['platform'] = meta['platform']
        if 'environment' in meta:
            svc['environment'] = meta['environment']
        if 'secrets' in meta:
            svc['secrets'] = meta['secrets']
        # Healthcheck custom
        if 'healthcheck' in meta:
            svc['healthcheck'] = meta['healthcheck']
        else:
            svc['healthcheck'] = generate_healthcheck(internal_port)
        if args and hasattr(args, 'dev') and args.dev:
            svc.setdefault('volumes', []).append(f"{path}:/app")
            svc['command'] = meta.get('dev_command', 'npm run dev' if is_frontend else None)
        if not is_frontend:
            generate_openapi_stub(name, path, meta)
        depends_on = meta.get('depends_on', [])
        # Validasi dependensi
        missing_deps = []
        if all_service_names is not None:
            for dep in depends_on:
                if dep not in all_service_names:
                    missing_deps.append(dep)
        if missing_deps:
            msg = f"Service {name} depends_on {missing_deps} yang tidak ditemukan!"
            logger.warning(msg)
            if summary is not None:
                summary['warnings'].append(msg)
        services[name] = svc
        if summary is not None:
            summary['services'].append({
                "name": name,
                "port": port,
                "internal_port": internal_port,
                "path": path,
                "frontend": is_frontend,
                "depends_on": depends_on,
                "disabled": meta.get('disabled', False),
                "dockerfile_found": dockerfile_found,
                "missing_deps": missing_deps
            })
        port += 1
    return services

def scan_frontend_services(base_dir, port_start, default_internal, all_service_names=None, summary=None, args=None):
    """Scan frontend services."""
    return scan_services(base_dir, port_start, default_internal, is_frontend=True, all_service_names=all_service_names, summary=summary, args=args)

def scan_backend_services(base_dir, port_start, default_internal, all_service_names=None, summary=None, args=None):
    """Scan backend services."""
    return scan_services(base_dir, port_start, default_internal, is_frontend=False, all_service_names=all_service_names, summary=summary, args=args)

def generate_nginx_conf(frontend_services):
    """Generate nginx.conf lines untuk frontend services."""
    conf_lines = [
        "events {",
        "    worker_connections 1024;",
        "}",
        "",
        "http {",
        "    include       /etc/nginx/mime.types;",
        "    default_type  application/octet-stream;",
        "",
        "    sendfile        on;",
        "    keepalive_timeout  65;",
        "",
        "    # Gzip compression",
        "    gzip on;",
        "    gzip_vary on;",
        "    gzip_min_length 1024;",
        "    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;",
        ""
    ]
    
    # Add upstream definitions
    for name, svc in frontend_services.items():
        port = 3000  # Default port
        if 'ports' in svc and svc['ports']:
            port_str = svc['ports'][0]
            if ':' in port_str:
                port = int(port_str.split(':')[1])
            else:
                port = int(port_str)
        
        conf_lines.extend([
            f"    upstream {name} {{",
            f"        server {name}:{port};",
            "    }",
            ""
        ])
    
    # Generate server block with proper routing
    server_locs = []
    for name, svc in frontend_services.items():
        meta = svc.get('_meta', {})
        domain = meta.get('domain')
        path = svc['build']['context']
        info = detect_frontend_structure(path)
        output = info['build_output']
        is_ssr = info['is_ssr']
        
        # Ambil port dari konfigurasi service
        port = 3000  # Default port
        if 'ports' in svc and svc['ports']:
            port_str = svc['ports'][0]
            if ':' in port_str:
                port = int(port_str.split(':')[1])
            else:
                port = int(port_str)
        
        if is_ssr:
            loc = [
                f"        # {name.title()}",
                f"        location /{name} {{",
                f"            proxy_pass http://{name}/;",
                "            proxy_set_header Host $host;",
                "            proxy_set_header X-Real-IP $remote_addr;",
                "            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;",
                "            proxy_set_header X-Forwarded-Proto $scheme;",
                "        }",
                ""
            ]
        else:
            loc = [
                f"        # {name.title()}",
                f"        location /{name} {{",
                f"            root /usr/share/nginx/html/{output};",
                "            try_files $uri $uri/ /index.html;",
                "        }",
                ""
            ]
        
        if domain:
            conf_lines += [
                "    server {",
                "        listen 80;",
                f"        server_name {domain};",
                ""
            ] + loc + ["    }", ""]
        else:
            server_locs.extend(loc)
    
    # Tambahkan security headers di awal blok server utama
    security_headers = [
        '        # Security headers',
        '        add_header Content-Security-Policy "default-src \'self\'" always;',
        '        add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;',
        '        add_header X-Frame-Options "DENY" always;',
        '        add_header X-Content-Type-Options "nosniff" always;',
        '        add_header Referrer-Policy "no-referrer" always;',
        '        add_header Permissions-Policy "geolocation=(), microphone=()" always;',
        ''
    ]
    if server_locs:
        conf_lines += [
            "    server {",
            "        listen 80;",
            "        server_name _;",
            ""
        ] + security_headers + server_locs + [
            "        # Default route - redirect to portal",
            "        location / {",
            "            return 301 /portal;",
            "        }",
            "",
            "        # Health check endpoint",
            "        location /healthz {",
            "            access_log off;",
            "            return 200 \"healthy\\n\";",
            "            add_header Content-Type text/plain;",
            "        }",
            "    }"
        ]
    
    conf_lines.append("}")
    return conf_lines

def generate_envoy_yaml(backend_services):
    """Generate envoy.yaml config untuk backend services dengan konfigurasi advanced."""
    clusters = []
    routes = []
    
    # Tambahkan route default untuk gerbang
    routes.append("""                        - match: { prefix: "/" }
                          route: 
                            cluster: gerbang
                            timeout: 30s
                            retry_policy:
                              retry_on: connect-failure,refused-stream,unavailable,cancelled,retriable-status-codes
                              num_retries: 3
                              per_try_timeout: 5s
                          cors:
                            allow_origin_string_match:
                              - prefix: "*"
                            allow_methods: GET, POST, PUT, DELETE, OPTIONS
                            allow_headers: "*"
                            expose_headers: "*"
                            max_age: "86400"
""")
    
    # Cluster untuk gerbang sebagai entry point
    clusters.append("""
  - name: gerbang
    connect_timeout: 30s
    type: strict_dns
    circuit_breakers:
      thresholds:
        - priority: DEFAULT
          max_requests: 1000
          max_retries: 3
    load_assignment:
      cluster_name: gerbang
      endpoints:
        - lb_endpoints:
            - endpoint:
                address:
                  socket_address: { address: gerbang, port_value: 8080 }
""")
    
    # Tambahkan clusters dan routes untuk semua backend services
    for name, svc in backend_services.items():
        if name == "gerbang":
            continue  # Skip karena sudah ditambahkan di atas
            
        # Ambil port dari konfigurasi service
        port = 8080  # Default port
        if 'ports' in svc and svc['ports']:
            # Ambil port internal dari format "host:internal"
            port_str = svc['ports'][0]
            if ':' in port_str:
                port = int(port_str.split(':')[1])
            else:
                port = int(port_str)
        
        meta = svc.get('_meta', {})
        prefix = meta.get("api_prefix", f"/api/{name}")
        
        # Route untuk backend service
        routes.append(f"""                        - match: {{ prefix: "{prefix}" }}
                          route: 
                            cluster: {name}
                            timeout: 30s
                            retry_policy:
                              retry_on: connect-failure,refused-stream,unavailable,cancelled,retriable-status-codes
                              num_retries: 3
                              per_try_timeout: 5s
""")
        
        # Cluster untuk backend service
        clusters.append(f"""
  - name: {name}
    connect_timeout: 30s
    type: strict_dns
    circuit_breakers:
      thresholds:
        - priority: DEFAULT
          max_requests: 500
          max_retries: 2
    load_assignment:
      cluster_name: {name}
      endpoints:
        - lb_endpoints:
            - endpoint:
                address:
                  socket_address: {{ address: {name}, port_value: {port} }}
""")
    
    yaml_tpl = f"""static_resources:
  listeners:
    - name: listener_0
      address:
        socket_address: {{ address: 0.0.0.0, port_value: 8080 }}
      filter_chains:
        - filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                stat_prefix: ingress_http
                route_config:
                  name: local_route
                  virtual_hosts:
                    - name: backend
                      domains: ["*"]
                      routes:
{chr(10).join(routes)}
                http_filters:
                  - name: envoy.filters.http.cors
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.cors.v3.Cors
                  - name: envoy.filters.http.lua
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.lua.v3.Lua
                      inline_code: |
                        function envoy_on_response(response_handle)
                          response_handle:headers():add(\"Content-Security-Policy\", \"default-src 'self'\")
                          response_handle:headers():add(\"Strict-Transport-Security\", \"max-age=63072000; includeSubDomains; preload\")
                          response_handle:headers():add(\"X-Frame-Options\", \"DENY\")
                          response_handle:headers():add(\"X-Content-Type-Options\", \"nosniff\")
                          response_handle:headers():add(\"Referrer-Policy\", \"no-referrer\")
                          response_handle:headers():add(\"Permissions-Policy\", \"geolocation=(), microphone=()\")
                        end
                  - name: envoy.filters.http.router
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router

  clusters:{chr(10).join(clusters)}
admin:
  access_log_path: "/tmp/admin_access.log"
  address:
    socket_address: {{ address: 0.0.0.0, port_value: 9901 }}
"""
    return yaml_tpl

def scan_dynamic_services(services, frontend_names, backend_names):
    """Scan dan update services secara dinamis untuk layanan baru."""
    updated = False
    
    # Scan frontend services baru
    for name, svc in services.items():
        if name not in frontend_names and svc.get('labels', {}).get('simpelv2.role') == 'frontend':
            frontend_names.append(name)
            updated = True
            logger.info(f"Detected new frontend service: {name}")
    
    # Scan backend services baru
    for name, svc in services.items():
        if name not in backend_names and svc.get('labels', {}).get('simpelv2.role') == 'backend':
            backend_names.append(name)
            updated = True
            logger.info(f"Detected new backend service: {name}")
    
    return updated

def update_configs_for_new_services(services, frontend_names, backend_names):
    """Update nginx.conf dan envoy.yaml untuk layanan baru."""
    # Update nginx.conf
    frontend_svcs = {k: v for k, v in services.items() if k in frontend_names}
    conf_lines = generate_nginx_conf(frontend_svcs)
    
    # Buat folder nginx jika tidak ada
    nginx_dir = "infra/nginx"
    if not os.path.exists(nginx_dir):
        os.makedirs(nginx_dir)
    
    # Tulis nginx.conf di folder nginx
    nginx_conf_path = os.path.join(nginx_dir, "nginx.conf")
    with open(nginx_conf_path, "w") as f:
        f.write("\n".join(conf_lines))
    logger.info(f"Updated nginx.conf at {nginx_conf_path}")
    
    # Update envoy.yaml
    backend_svcs = {k: v for k, v in services.items() if k in backend_names}
    if 'gerbang' in services and 'gerbang' not in backend_svcs:
        backend_svcs['gerbang'] = services['gerbang']
    
    yaml_tpl = generate_envoy_yaml(backend_svcs)
    
    # Tulis envoy.yaml di folder gerbang
    envoy_yaml_path = os.path.join("infra", "gerbang", "envoy.yaml")
    with open(envoy_yaml_path, "w") as f:
        f.write(yaml_tpl.strip())
    logger.info(f"Updated envoy.yaml at {envoy_yaml_path}")

def reload_nginx_config():
    """Reload nginx configuration jika nginx berjalan."""
    try:
        import subprocess
        # Gunakan path nginx.conf yang benar
        nginx_conf_path = "infra/nginx/nginx.conf"
        if os.path.exists(nginx_conf_path):
            # Test config dulu
            test_result = subprocess.run(['nginx', '-t', '-c', nginx_conf_path], capture_output=True, text=True)
            if test_result.returncode == 0:
                # Reload jika test berhasil
                result = subprocess.run(['nginx', '-s', 'reload'], capture_output=True, text=True)
                if result.returncode == 0:
                    logger.info("Nginx configuration reloaded successfully")
                else:
                    logger.warning(f"Nginx reload failed: {result.stderr}")
            else:
                logger.warning(f"Nginx config test failed: {test_result.stderr}")
        else:
            logger.warning(f"Nginx config file not found: {nginx_conf_path}")
    except Exception as e:
        logger.warning(f"Could not reload nginx: {e}")

def reload_envoy_config():
    """Reload envoy configuration jika envoy berjalan."""
    try:
        import subprocess
        # Envoy tidak mendukung reload langsung, perlu restart container
        logger.info("Envoy configuration updated, container restart required")
    except Exception as e:
        logger.warning(f"Could not reload envoy: {e}")

def auto_reload_configs():
    """Auto-reload konfigurasi nginx dan envoy."""
    reload_nginx_config()
    reload_envoy_config()

# -------------------- Workflow Utama --------------------
def generate_compose_main(args=None):
    """
    Fungsi utama generator Compose. Bisa menerima argumen (Namespace/argparse) atau parsing sendiri jika None.
    Menjalankan seluruh workflow Compose: scan service, generate compose, nginx, envoy, summary, validasi, dsb.
    """
    import argparse
    import yaml
    import copy
    from .secret_manager import get_secret
    
    # 1. Parsing argumen jika args None atau tidak memiliki atribut yang diperlukan
    if args is None or not hasattr(args, 'layanan_dir'):
        parser = argparse.ArgumentParser(
            description="Auto-generate docker-compose.yml for SIMPelv2 microservices"
        )
        parser.add_argument("--layanan-dir", default="layanan", help="Path to folder containing backend services")
        parser.add_argument("--antarmuka-dir", default="antarmuka", help="Path to folder containing frontend services")
        parser.add_argument("--output", default="docker-compose.yml", help="Output file name")
        parser.add_argument("--port-frontend-start", type=int, default=3000, help="Starting host port for frontends")
        parser.add_argument("--port-backend-start", type=int, default=7000, help="Starting host port for backends")
        parser.add_argument("--postgres-image", default="postgres:15", help="Docker image for Postgres")
        parser.add_argument("--dev", action="store_true", help="Enable development mode (hot reload, volume mounts)")
        parser.add_argument("--extra-network", action="append", default=[], help="Tambahkan network ke compose (bisa dipanggil berkali-kali)")
        parser.add_argument("--extra-volume", action="append", default=[], help="Tambahkan volume ke compose (bisa dipanggil berkali-kali)")
        parser.add_argument("--include-service", action="append", default=[], help="Hanya generate service tertentu (bisa dipanggil berkali-kali)")
        parser.add_argument("--exclude-service", action="append", default=[], help="Exclude service tertentu (bisa dipanggil berkali-kali)")
        parser.add_argument("--external-compose", action="append", default=[], help="File compose eksternal untuk di-include (bisa dipanggil berkali-kali)")
        parser.add_argument("--summary-format", choices=["json", "markdown", "both"], default="both", help="Format summary output (json/markdown/both)")
        parser.add_argument("--env", choices=["dev", "staging", "prod"], default="dev", help="Environment")
        parser.add_argument("--secret-provider", choices=["vault", "aws", "gcp", "azure"], default=None, help="Provider secret manager (vault/aws/gcp/azure)")
        parser.add_argument("--secret-path", default=None, help="Path/key secret di provider")
        parser.add_argument("--secret-token", default=None, help="Token untuk HashiCorp Vault")
        parser.add_argument("--secret-url", default=None, help="Base URL untuk HashiCorp Vault")
        parser.add_argument("--dry-run", action="store_true", help="Hanya generate file, tidak melakukan reload konfigurasi")
        
        # Jika args sudah ada, gunakan nilai yang ada, jika tidak gunakan default
        if args is not None:
            # Copy existing args ke parser
            for attr, value in vars(args).items():
                if hasattr(parser, attr):
                    setattr(parser, attr, value)
        
        args = parser.parse_args([])  # Parse dengan empty list untuk menggunakan defaults

    # Setup paths
    ROOT_DIR = os.path.abspath('.')
    LAYANAN_DIR = os.path.join(ROOT_DIR, getattr(args, 'layanan_dir', 'layanan'))  # Updated to use layanan folder
    ANTARMUKA_DIR = os.path.join(ROOT_DIR, getattr(args, 'antarmuka_dir', 'antarmuka'))
    INFRA_DIR = os.path.join(ROOT_DIR, 'infra')  # New infra directory
    ENV_FILE = os.path.join(ROOT_DIR, '.env')
    DEFAULT_ENV = {
        "DB_USER": "simpel",
        "DB_PASS": "rahasia",
        "DB_NAME": "simpelv2",
    }
    if not os.path.exists(ENV_FILE):
        with open(ENV_FILE, 'w') as f:
            for key, val in DEFAULT_ENV.items():
                f.write(f"{key}={val}\n")
        logger.info(f"Created .env with defaults: {ENV_FILE}")
    summary = {"services": [], "errors": [], "warnings": []}

    # --- Integrasi Secret Manager ---
    secrets = None
    if getattr(args, 'secret_provider', None) and getattr(args, 'secret_path', None):
        logger.info(f"Mengambil secrets dari {args.secret_provider} path={args.secret_path}")
        secret_kwargs = {}
        if args.secret_provider == 'vault':
            secret_kwargs['token'] = getattr(args, 'secret_token', None)
            secret_kwargs['url'] = getattr(args, 'secret_url', None)
        secrets = get_secret(args.secret_provider, args.secret_path, **secret_kwargs)
        if not secrets:
            logger.error("Gagal mengambil secrets dari secret manager, fallback ke .env jika ada.")
    else:
        logger.info("Secret manager tidak digunakan, fallback ke .env.")

    # 3. Dapatkan semua nama service (untuk validasi dependensi)
    # Scan frontend services secara dinamis
    frontend_names = [d.lower().replace('_', '-') for d in os.listdir(ANTARMUKA_DIR) if os.path.isdir(os.path.join(ANTARMUKA_DIR, d))]
    
    # Scan backend services secara dinamis
    backend_names = []
    
    # Scan gerbang dari infra/
    gerbang_path = os.path.join(INFRA_DIR, 'gerbang')
    if os.path.exists(gerbang_path):
        backend_names.append('gerbang')
    
    # Scan layanan dari folder layanan/ secara dinamis
    layanan_dir = os.path.join(ROOT_DIR, 'layanan')
    if os.path.exists(layanan_dir):
        for item in os.listdir(layanan_dir):
            item_path = os.path.join(layanan_dir, item)
            if os.path.isdir(item_path):
                logger.info(f"Scanning layanan: {item}")
                # Scan subfolder di dalam layanan
                for subitem in os.listdir(item_path):
                    subitem_path = os.path.join(item_path, subitem)
                    if os.path.isdir(subitem_path):
                        service_name = subitem.lower().replace('_', '-')
                        backend_names.append(service_name)
                        logger.info(f"  - Found backend service: {service_name}")
    
    all_service_names = set(frontend_names + backend_names)
    services = {}
    
    # Scan frontend services
    services.update(scan_frontend_services(ANTARMUKA_DIR, getattr(args, 'port_frontend_start', 3000), 3000, all_service_names=all_service_names, summary=summary, args=args))
    
    # Scan backend services
    backend_services = {}
    
    # Scan gerbang dari infra/
    gerbang_path = os.path.join(INFRA_DIR, 'gerbang')
    if os.path.exists(gerbang_path):
        logger.info(f"Scanning backend service: gerbang")
        service_scan = scan_services(gerbang_path, getattr(args, 'port_backend_start', 7000), 7000, is_frontend=False, all_service_names=all_service_names, summary=summary, args=args)
        # Jika tidak ada service yang ditemukan, buat service gerbang manual
        if not service_scan:
            service_scan = {
                'gerbang': {
                    'build': {'context': f'./infra/gerbang', 'dockerfile': 'Dockerfile'},
                    'ports': [f'{getattr(args, "port_backend_start", 7000)}:8080'],
                    'networks': ['simpelv2', 'internal'],
                    'healthcheck': generate_healthcheck(8080),
                    'labels': {
                        'simpelv2.role': 'backend',
                        'simpelv2.name': 'gerbang'
                    }
                }
            }
        backend_services.update(service_scan)
    
    # Scan layanan dari folder layanan/ secara dinamis
    layanan_dir = os.path.join(ROOT_DIR, 'layanan')
    if os.path.exists(layanan_dir):
        for item in os.listdir(layanan_dir):
            item_path = os.path.join(layanan_dir, item)
            if os.path.isdir(item_path):
                logger.info(f"Scanning layanan folder: {item}")
                # Scan subfolder di dalam layanan
                service_scan = scan_services(item_path, getattr(args, 'port_backend_start', 7000), 7000, is_frontend=False, all_service_names=all_service_names, summary=summary, args=args)
                if service_scan:
                    logger.info(f"  - Found services: {list(service_scan.keys())}")
                backend_services.update(service_scan)
    
    services.update(backend_services)

    # --- Inject secrets ke environment service jika ada ---
    if secrets:
        for svc_name, svc in services.items():
            # Jika secret spesifik per service, gunakan itu; jika tidak, inject semua global
            env_secret = secrets.get(svc_name) if isinstance(secrets, dict) and svc_name in secrets else secrets
            if env_secret and isinstance(env_secret, dict):
                svc.setdefault('environment', {}).update(env_secret)
                logger.info(f"Inject secrets ke environment service {svc_name}")

    # 4. Generate infrastructure services
    services['db'] = {
        "image": getattr(args, 'postgres_image', 'postgres:15'),
        "restart": "always",
        "environment": {
            "POSTGRES_USER": "${DB_USER}",
            "POSTGRES_PASSWORD": "${DB_PASS}",
            "POSTGRES_DB": "${DB_NAME}",
        },
        "volumes": ["pgdata:/var/lib/postgresql/data"],
        "networks": ["simpelv2", "internal"],
        "healthcheck": {
            "test": ["CMD-SHELL", "pg_isready -U ${DB_USER}"],
            "interval": "30s",
            "timeout": "10s",
            "retries": 5,
        },
        "labels": {
            "simpelv2.role": "db",
            "simpelv2.name": "db"
        }
    }
    
    # Generate nginx service
    services['nginx'] = {
        "image": "nginx:stable-alpine",
        "volumes": [
            "./infra/nginx/nginx.conf:/etc/nginx/nginx.conf:ro",
            "./nginx/conf.d:/etc/nginx/conf.d:ro",
            "./nginx/certs:/etc/nginx/certs:ro",
            "./antarmuka/dist:/usr/share/nginx/html:ro",
            "./logs/nginx:/var/log/nginx"
        ],
        "ports": ["80:80", "443:443"],
        "networks": ["simpelv2", "internal"],
        "labels": {
            "simpelv2.role": "infrastructure",
            "simpelv2.name": "nginx"
        }
    }
    
    # Note: Semua layanan backend akan di-scan secara dinamis dari folder layanan/
    # Tidak perlu hardcode layanan-keamanan dan layanan-integrasi

    # 5. Generate nginx.conf dan envoy.yaml
    # Gunakan services yang sudah di-scan, bukan frontend_names dan backend_names yang di-scan di awal
    frontend_svcs = {k: v for k, v in services.items() if k in frontend_names}
    backend_svcs = {k: v for k, v in services.items() if k in backend_names}
    
    # Tambahkan gerbang ke backend_svcs jika belum ada
    if 'gerbang' in services and 'gerbang' not in backend_svcs:
        backend_svcs['gerbang'] = services['gerbang']
    
    # Log services yang ditemukan untuk debugging
    logger.info(f"Frontend services found: {list(frontend_svcs.keys())}")
    logger.info(f"Backend services found: {list(backend_svcs.keys())}")
    
    # Generate konfigurasi normal
    conf_lines = generate_nginx_conf(frontend_svcs)
    
    # Buat folder nginx jika tidak ada
    nginx_dir = "infra/nginx"
    if not os.path.exists(nginx_dir):
        os.makedirs(nginx_dir)
    
    # Tulis nginx.conf di folder nginx
    nginx_conf_path = os.path.join(nginx_dir, "nginx.conf")
    with open(nginx_conf_path, "w") as f:
        f.write("\n".join(conf_lines))
    logger.info(f"Generated nginx.conf at {nginx_conf_path}")
    validate_nginx_conf(nginx_conf_path)
    
    # Generate envoy.yaml
    yaml_tpl = generate_envoy_yaml(backend_svcs)
    
    # Tulis envoy.yaml di folder gerbang
    envoy_yaml_path = os.path.join("infra", "gerbang", "envoy.yaml")
    with open(envoy_yaml_path, "w") as f:
        f.write(yaml_tpl.strip())
    logger.info(f"Generated envoy.yaml at {envoy_yaml_path}")
    validate_yaml(envoy_yaml_path)

    # 6. Generate docker-compose.generated.yml
    networks = {"simpelv2": {}, "internal": {"internal": True}}
    for net in args.extra_network:
        networks[net] = {}
    volumes = {"pgdata": {}}
    for vol in args.extra_volume:
        volumes[vol] = {}
    compose = {
        "services": services,
        "networks": networks,
        "volumes": volumes,
    }
    if args.external_compose:
        compose['extends'] = args.external_compose
    backup_file(args.output)
    with open(args.output, 'w') as out:
        yaml.dump(compose, out, sort_keys=False, default_flow_style=False)
    logger.info(f"Generated Docker Compose file: {args.output}")
    validate_yaml(args.output)

    # --- Overlay Multi-Environment Compose ---
    env = getattr(args, 'env', 'dev')
    overlay_file = f"docker-compose.{env}.yml"
    if os.path.exists(overlay_file):
        logger.info(f"Overlay Compose ditemukan: {overlay_file}, akan di-merge ke hasil generate.")
        try:
            with open(args.output) as f:
                base_compose = yaml.safe_load(f)
            with open(overlay_file) as f:
                overlay_compose = yaml.safe_load(f)
            # Merge dict (overlay > base)
            def deep_merge(a, b):
                for k, v in b.items():
                    if k in a and isinstance(a[k], dict) and isinstance(v, dict):
                        deep_merge(a[k], v)
                    else:
                        a[k] = copy.deepcopy(v)
            merged = copy.deepcopy(base_compose)
            deep_merge(merged, overlay_compose)
            with open(args.output, 'w') as f:
                yaml.dump(merged, f, sort_keys=False, default_flow_style=False)
            logger.info(f"Overlay Compose berhasil di-merge ke {args.output}")
        except Exception as e:
            logger.error(f"Gagal merge overlay Compose: {e}")
    else:
        logger.info(f"Tidak ada overlay Compose untuk env {env}.")

    # 7. Validasi output build frontend
    for k in frontend_svcs:
        output_dir = detect_frontend_structure(services[k]['build']['context'])['build_output']
        full_path = os.path.join(services[k]['build']['context'], output_dir)
        if not os.path.exists(full_path):
            logger.warning(f"Output build {full_path} tidak ditemukan untuk service {k}. Pastikan sudah build.")

    # 8. Output summary
    if args.summary_format in ("json", "both"):
        try:
            with open("summary.json", "w") as f:
                json.dump(summary, f, indent=2)
            logger.info("Generated summary.json")
        except Exception as e:
            logger.error(f"Gagal menulis summary.json: {e}")
    if args.summary_format in ("markdown", "both"):
        try:
            with open("summary.md", "w") as f:
                f.write("# Ringkasan Service Compose\n\n")
                for svc in summary['services']:
                    f.write(f"- **{svc['name']}**: port {svc['port']} (internal: {svc['internal_port']}) | frontend: {svc['frontend']} | path: {svc['path']} | depends_on: {svc['depends_on']} | dockerfile: {svc['dockerfile_found']}\n")
                    if svc['missing_deps']:
                        f.write(f"  - ⚠️  missing depends_on: {svc['missing_deps']}\n")
                if summary['warnings']:
                    f.write("\n## Warning\n")
                    for w in summary['warnings']:
                        f.write(f"- {w}\n")
                if summary['errors']:
                    f.write("\n## Error\n")
                    for e in summary['errors']:
                        f.write(f"- {e}\n")
            logger.info("Generated summary.md")
        except Exception as e:
            logger.error(f"Gagal menulis summary.md: {e}")

    # 9. Exit code dan return
    if summary['errors']:
        logger.error("Terdapat error pada proses generate Compose.")
        return False
    logger.info("Generate Compose selesai tanpa error.")
    return True 