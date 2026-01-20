# SIMPelv2 - Project Structure

## Top-Level Organization

```
simpelv2/
├── antarmuka/          # Frontend microfrontends (Leptos/WASM)
├── layanan/            # Backend microservices (Axum)
├── infra/              # Infrastructure services
├── scripts/            # Build automation and CLI tools
├── docs/               # Documentation (60+ files)
├── simpel_laravel/     # Legacy Laravel codebase (reference only)
└── test/               # Performance tests
```

## Frontend (`antarmuka/`)

| Directory | Purpose | Port |
|-----------|---------|------|
| `portal/` | Main gateway, SSO integration | 8080 |
| `badiklat/` | Training & Education | 8081 |
| `datun/` | Civil Litigation | 8082 |
| `intel/` | Intelligence & Analytics | 8083 |
| `pemulihan_aset/` | Asset Recovery | 8084 |
| `pengawasan/` | Monitoring & Compliance | 8085 |
| `pidmil/` | Military Criminal Law | 8086 |
| `pidsus/` | Special Crimes | 8087 |
| `pidum/` | General Crimes | 8088 |
| `pembinaan/keuangan/` | Finance Management | 8089 |
| `pembinaan/perencanaan/` | Planning | 8090 |
| `pembinaan/perlengkapan/` | Equipment Management | 8091 |
| `shared/` | Shared component library (40+ components) | - |

## Backend (`layanan/`)

- Domain services mirror frontend modules (badiklat, datun, intel, etc.)
- `shared/` contains cross-cutting services:
  - `ai/` - LLM, RAG, OCR
  - `bantuan/` - Help desk
  - `dasbor/` - Dashboard & metrics
  - `dokumen/` - Document management
  - `integrasi/` - External API integrations
  - `konfigurasi/` - System configuration
  - `laporan/` - Reporting
  - `notifikasi/` - Notifications

## Infrastructure (`infra/`)

| Directory | Purpose | Notes |
|-----------|---------|-------|
| `authenc/` | IAM service | **Separate Cargo workspace** |
| `secreton/` | Secrets management | **Separate Cargo workspace** |
| `gerbang/` | API Gateway (Envoy) | HTTP/gRPC routing |
| `k8s/` | Kubernetes manifests | MicroK8s deployment |
| `nginx/` | Reverse proxy config | SSL termination |
| `proto/` | Protocol Buffers | gRPC definitions |
| `monitoring/` | Prometheus + Grafana | Observability |

## Scripts (`scripts/`)

- `cli/` - Rust CLI tool for project management
- `makefiles/` - Modular Makefile includes
- `backup/` - Backup automation
- `test/` - Test automation
- `tools/` - Development utilities

## Key Files

- `Cargo.toml` - Workspace configuration (all deps centralized)
- `Makefile` - Build orchestration (includes modular makefiles)
- `docker-compose.yml` - Base container config
- `.gitlab-ci.yml` - CI/CD pipeline (9 stages, 12+ security tools)
