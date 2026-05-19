# SIMPEL Docker Build Guide

## 📋 Overview

Panduan lengkap untuk build dan push container images SIMPEL ke registry.

## 🏗️ Arsitektur Build

### Workspace Structure

```
simpelv2/
├── Cargo.toml                    # Root workspace
├── infra/
│   ├── authenc/                  # Independent workspace
│   ├── secreton/                 # Independent workspace
│   └── gerbang/                  # Envoy proxy (simple)
├── layanan/
│   └── shared/                   # Workspace members
│       ├── ai/
│       ├── bantuan/
│       ├── dasbor/
│       ├── dokumen/
│       ├── integrasi/
│       ├── konfigurasi/
│       ├── laporan/
│       └── notifikasi/
└── antarmuka/                    # Workspace members
    ├── portal/
    ├── badiklat/
    ├── datun/
    └── ...
```

### Build Context Rules

**Independent Workspaces** (authenc, secreton, gerbang):

- Build context: Service directory
- Command: `podman build -f infra/SERVICE/Dockerfile infra/SERVICE`

**Workspace Members** (layanan/*, antarmuka/*):

- Build context: Workspace root (.)
- Command: `podman build -f layanan/shared/SERVICE/Dockerfile .`

## 🔧 Prerequisites

### System Requirements

- Podman atau Docker
- Rust 1.90+
- 8GB+ RAM (untuk parallel builds)
- 50GB+ disk space

### Registry Configuration

**MicroK8s Registry (localhost:32000)**:

```bash
# Add to /etc/containers/registries.conf
unqualified-search-registries = ["docker.io"]

[[registry]]
location = "localhost:32000"
insecure = true
```

**External Registry (registry.kejaksaan.go.id)**:

```bash
podman login registry.kejaksaan.go.id
```

## 🚀 Build Methods

### Method 1: Incremental Build (Recommended)

Build services in priority tiers:

```bash
cd /srv/proyek/simpelv2
./scripts/build-incremental.sh
```

**Tiers:**

- Tier 1: Critical infrastructure (gerbang, authenc, secreton)
- Tier 2: Backend services (8 layanan)
- Tier 3: Frontend microfrontends (12 antarmuka)

### Method 2: Build All at Once

```bash
./scripts/build-and-push-all.sh
```

**Note**: Memakan waktu lama (2-4 jam) dan resources besar.

### Method 3: Build Individual Service

**Infrastructure Services:**

```bash
# Gerbang (Envoy)
podman build -t localhost:32000/gerbang:latest \
  -f infra/gerbang/Dockerfile infra/gerbang
podman push localhost:32000/gerbang:latest

# Authenc
podman build -t localhost:32000/authenc:latest \
  -f layanan/authenc/Dockerfile layanan/authenc
podman push localhost:32000/authenc:latest

# Secreton
podman build -t localhost:32000/secreton:latest \
  -f layanan/secreton/Dockerfile layanan/secreton
podman push localhost:32000/secreton:latest
```

**Backend Services:**

```bash
# Build from workspace root
podman build -t localhost:32000/layanan-ai:latest \
  -f layanan/shared/ai/Dockerfile .
podman push localhost:32000/layanan-ai:latest
```

**Frontend Services:**

```bash
# Build from workspace root
podman build -t localhost:32000/portal:latest \
  -f antarmuka/portal/Dockerfile .
podman push localhost:32000/portal:latest
```

## 📦 Image List

### Infrastructure (3 images)

- `gerbang` - API Gateway (Envoy)
- `authenc` - Authentication & Authorization
- `secreton` - Secrets Management (Vault)

### Backend Services (8 images)

- `layanan-ai` - AI & ML services
- `layanan-bantuan` - Help & Support
- `layanan-dasbor` - Dashboard
- `layanan-dokumen` - Document Management
- `layanan-integrasi` - Integration Hub
- `layanan-konfigurasi` - Configuration
- `layanan-laporan` - Reporting
- `layanan-notifikasi` - Notifications

### Frontend Microfrontends (12 images)

- `portal` - Main Portal
- `badiklat` - Training
- `datun` - Civil Law
- `intel` - Intelligence
- `keuangan` - Finance
- `pemulihan-aset` - Asset Recovery
- `pengawasan` - Supervision
- `perencanaan` - Planning
- `perlengkapan` - Equipment
- `pidmil` - Military Criminal
- `pidsus` - Special Criminal
- `pidum` - General Criminal

**Total: 23 images**

## 🐛 Troubleshooting

### Build Failures

**Error: "can't find bench at benches/performance.rs"**

```bash
# Solution: Dockerfile sudah di-fix untuk copy benches directory
```

**Error: "OpenSSL not found"**

```bash
# Solution: Dockerfile sudah di-fix dengan install libssl-dev
```

**Error: "short-name did not resolve"**

```bash
# Solution: Add unqualified-search-registries = ["docker.io"]
# to /etc/containers/registries.conf
```

**Error: "http: server gave HTTP response to HTTPS client"**

```bash
# Solution: Add insecure registry configuration
[[registry]]
location = "localhost:32000"
insecure = true
```

### Build Context Issues

**Error: "failed to parse manifest - no targets specified"**

- Cause: Build context salah (dari subdirectory instead of workspace root)
- Solution: Build workspace members dari root directory

### Memory Issues

**Error: "signal: killed" during compilation**

- Cause: Out of memory
- Solution: Build services satu per satu, atau tambah swap

## 📊 Build Times (Approximate)

| Service Type | Build Time | Size |
|-------------|-----------|------|
| Gerbang (Envoy) | 1-2 min | ~200MB |
| Authenc | 15-20 min | ~100MB |
| Secreton | 15-20 min | ~100MB |
| Backend Service | 10-15 min | ~80MB |
| Frontend | 20-30 min | ~50MB |

**Total (all services)**: 2-4 hours

## 🔍 Verification

### Check Built Images

```bash
podman images | grep localhost:32000
```

### Check Registry

```bash
curl http://localhost:32000/v2/_catalog
```

### Test Image

```bash
podman run --rm localhost:32000/gerbang:latest --version
```

### Check in Kubernetes

```bash
microk8s.kubectl get pods -A
microk8s.kubectl describe pod <pod-name> -n <namespace>
```

## 🎯 Best Practices

1. **Build incrementally** - Start with critical services
2. **Use layer caching** - Leverage cargo-chef for Rust builds
3. **Monitor resources** - Watch memory and disk usage
4. **Tag properly** - Use semantic versioning
5. **Test locally** - Run container before pushing
6. **Clean regularly** - Remove unused images

## 🔄 CI/CD Integration

### GitLab CI Example

```yaml
build:
  stage: build
  script:
    - ./scripts/build-incremental.sh
  only:
    - main
```

### GitHub Actions Example

```yaml
- name: Build Images
  run: ./scripts/build-incremental.sh
```

## 📚 References

- [Rust Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Multi-stage Builds](https://docs.docker.com/develop/develop-images/multistage-build/)
- [Cargo Chef](https://github.com/LukeMathWalker/cargo-chef)
- [MicroK8s Registry](https://microk8s.io/docs/registry-built-in)
