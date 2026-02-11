# 🏛️ SIMPelv2 - Sistem Informasi Manajemen Pengelolaan BMN

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://rustlang.org)
[![Leptos](https://img.shields.io/badge/leptos-0.8.12-green.svg)](https://leptos.dev)
[![Kubernetes](https://img.shields.io/badge/kubernetes-ready-brightgreen.svg)](https://kubernetes.io)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**SIMPelv2** adalah platform enterprise-grade untuk pengelolaan Barang Milik Negara (BMN) Kejaksaan Agung Republik Indonesia, dibangun dengan arsitektur **microfrontend + microservices** menggunakan **Rust** untuk performa, keamanan, dan skalabilitas maksimal.

## 🎯 Overview

SIMPelv2 menyediakan solusi terintegrasi untuk pengelolaan BMN, mulai dari perencanaan, pengadaan, distribusi, hingga pelaporan. Platform ini menggunakan:

- **Zero-Trust Security Architecture** dengan custom IAM (Authenc) dan secrets management (Secreton)
- **12 Microfrontend Independen** + shared component library
- **17 Backend Microservices** dengan gRPC internal communication
- **Enterprise CI/CD** dengan 12+ security tools

### 📦 Mockup Directories

| Directory | Purpose |
|-----------|---------|
| `antarmuka/contoh/*` | Mockup microfrontend (placeholder untuk modul yang belum diimplementasi) |
| `layanan/contoh/*` | Mockup backend service (placeholder untuk layanan yang belum diimplementasi) |

> **Superapps Ready**: SIMPelv2 dirancang dengan arsitektur modular yang siap diintegrasikan menjadi SIMKARI Superapps jika diperlukan. Lihat [AGENTS.md](AGENTS.md) untuk detail.

## 🌐 Production Deployment

### Live URLs

| Service         | URL                                                                                    | Status  |
| --------------- | -------------------------------------------------------------------------------------- | ------- |
| **Main Portal** | [https://simpel.kejaksaan.go.id/](https://simpel.kejaksaan.go.id/)                     | ✅ Live |
| **Badiklat**    | [https://simpel.kejaksaan.go.id/badiklat](https://simpel.kejaksaan.go.id/badiklat)     | ✅ Live |
| **Datun**       | [https://simpel.kejaksaan.go.id/datun](https://simpel.kejaksaan.go.id/datun)           | ✅ Live |
| **Intel**       | [https://simpel.kejaksaan.go.id/intel](https://simpel.kejaksaan.go.id/intel)           | ✅ Live |
| **Pengawasan**  | [https://simpel.kejaksaan.go.id/pengawasan](https://simpel.kejaksaan.go.id/pengawasan) | ✅ Live |
| **Pidum**       | [https://simpel.kejaksaan.go.id/pidum](https://simpel.kejaksaan.go.id/pidum)           | ✅ Live |
| **Pidsus**      | [https://simpel.kejaksaan.go.id/pidsus](https://simpel.kejaksaan.go.id/pidsus)         | ✅ Live |
| **Pidmil**      | [https://simpel.kejaksaan.go.id/pidmil](https://simpel.kejaksaan.go.id/pidmil)         | ✅ Live |

### Security Features

- ✅ **SSL/TLS**: DigiCert certificate dengan HTTP/2 support
- ✅ **HSTS**: Strict Transport Security headers
- ✅ **CSP**: Content Security Policy implementation
- ✅ **Zero-Trust**: All traffic encrypted dan authenticated

---

## 🏗️ Architecture

### Technology Stack

| Layer         | Technology         | Version              | Purpose                              |
| ------------- | ------------------ | -------------------- | ------------------------------------ |
| **Language**  | Rust               | 1.90+ (Edition 2024) | Memory safety & performance          |
| **Frontend**  | Leptos             | 0.8.12               | Reactive WASM framework              |
| **Backend**   | Axum               | 0.8.6                | High-performance web framework       |
| **Database**  | PostgreSQL         | 15+                  | Multi-schema dengan deadpool pooling |
| **Cache**     | Redis              | 7+                   | Session & data caching               |
| **Build**     | Trunk              | Latest               | WASM bundling & optimization         |
| **Container** | MicroK8s           | Latest               | Kubernetes orchestration             |
| **Security**  | Authenc + Secreton | Custom               | Zero-trust IAM & secrets             |
| **AI/ML**     | Qdrant + tiktoken  | Latest               | Vector search & tokenization         |

### Microfrontend Architecture

SIMPelv2 menggunakan **12 microfrontend independen** yang didukung shared component library:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Load Balancer (Nginx)                     │
│                       SSL Termination / HTTP2                    │
└───────────────────────────────┬─────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│    Portal     │    │   Badiklat    │    │    Datun      │
│  (Gateway)    │    │  (Training)   │    │  (Criminal)   │
└───────────────┘    └───────────────┘    └───────────────┘
        │
┌───────────────────────────────────────────────────────────┐
│                    Pembinaan Division                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │
│  │  Keuangan   │ │ Perencanaan │ │    Perlengkapan     │  │
│  │ (Finance)   │ │ (Planning)  │ │    (Equipment)      │  │
│  └─────────────┘ └─────────────┘ └─────────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

| Modul                      | Fungsi                    | Dev Port | Status        |
| -------------------------- | ------------------------- | -------- | ------------- |
| **Portal**                 | Gateway & SSO Integration | 8080     | ✅ Production |
| **Badiklat**               | Training & Education      | 8081     | ✅ Production |
| **Datun**                  | Civil Litigation          | 8082     | ✅ Production |
| **Intel**                  | Intelligence & Analytics  | 8083     | ✅ Production |
| **Pemulihan Aset**         | Asset Recovery            | 8084     | ✅ Production |
| **Pengawasan**             | Monitoring & Compliance   | 8085     | ✅ Production |
| **Pidmil**                 | Military Criminal Law     | 8086     | ✅ Production |
| **Pidsus**                 | Special Crimes            | 8087     | ✅ Production |
| **Pidum**                  | General Crimes            | 8088     | ✅ Production |
| **Pembinaan/Keuangan**     | Finance Management        | 8089     | ✅ Production |
| **Pembinaan/Perencanaan**  | Planning                  | 8090     | ✅ Production |
| **Pembinaan/Perlengkapan** | Equipment Management      | 8091     | ✅ Production |

### Shared Component Library v1.0.0

Production-ready UI library dengan Leptos 0.8.x compatibility:

- **40+ Components**: Button, Input, Modal, Table, Navigation, Form, Toast, dll.
- **Thread Safety**: Complete `Send + Sync` implementation
- **Modern Patterns**: Signal-based reactivity, callback methods
- **Government Branding**: Kejaksaan RI design system
- **60% Smaller**: Refactored dari 7,637 → ~2,800 lines

```rust
use lib_ui::prelude::*;
use lib_ui::components::auth::ProtectedRoute;
```

### Backend Microservices Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   API Gateway (Gerbang)                      │
│                  HTTP:8080 / gRPC:9080                       │
└─────────────────────────────┬───────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Authenc     │    │   Secreton    │    │   Layanan     │
│   (IAM)       │    │   (Secrets)   │    │   Domain      │
└───────────────┘    └───────────────┘    └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
                    ▼                   ▼
             ┌───────────┐       ┌───────────┐
             │ PostgreSQL│       │   Redis   │
             │  (15+)    │       │   (7+)    │
             └───────────┘       └───────────┘
```

#### Domain Services (`layanan/`)

| Service                    | Function            | Port |
| -------------------------- | ------------------- | ---- |
| **badiklat**               | Training management | 3001 |
| **datun**                  | Civil litigation    | 3002 |
| **intel**                  | Intelligence        | 3003 |
| **pemulihan_aset**         | Asset recovery      | 3004 |
| **pengawasan**             | Monitoring          | 3005 |
| **pidmil**                 | Military criminal   | 3006 |
| **pidsus**                 | Special crimes      | 3007 |
| **pidum**                  | General crimes      | 3008 |
| **pembinaan/perlengkapan** | Equipment           | 3009 |

#### Shared Services (`layanan/shared/`)

| Service         | Function            | Port |
| --------------- | ------------------- | ---- |
| **ai**          | LLM, RAG, OCR       | 3010 |
| **bantuan**     | Help desk & FAQ     | 3011 |
| **dasbor**      | Dashboard & metrics | 3012 |
| **dokumen**     | Document management | 3013 |
| **integrasi**   | External APIs       | 3014 |
| **konfigurasi** | System config       | 3015 |
| **laporan**     | Reporting           | 3016 |
| **notifikasi**  | Notifications       | 3017 |

#### Infrastructure Services (`infra/`)

| Service        | Function                     | Notes              |
| -------------- | ---------------------------- | ------------------ |
| **authenc**    | Identity & Access Management | Separate workspace |
| **secreton**   | Secret management            | Separate workspace |
| **gerbang**    | API Gateway (Envoy)          | HTTP/gRPC routing  |
| **monitoring** | Prometheus + Grafana         | Observability      |

> **Note**: `authenc` dan `secreton` adalah **separate Cargo workspaces** untuk architectural independence. Mereka berkomunikasi via mTLS, bukan shared code.

### Security Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Zero-Trust Model                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐  │
│  │  User   │───▶│ Authenc │───▶│   JWT   │───▶│ Service │  │
│  │         │    │  (IAM)  │    │  Token  │    │         │  │
│  └─────────┘    └────┬────┘    └─────────┘    └─────────┘  │
│                      │                                      │
│                      ▼                                      │
│               ┌─────────────┐                               │
│               │  Secreton   │                               │
│               │  (Secrets)  │                               │
│               └─────────────┘                               │
├─────────────────────────────────────────────────────────────┤
│  • Ed25519 signing (NOT RSA)                                │
│  • ChaCha20-Poly1305 encryption                             │
│  • TOTP-based MFA                                           │
│  • Immutable audit logs                                     │
│  • Post-quantum ready design                                │
└─────────────────────────────────────────────────────────────┘
```

**Security Features:**

- **Zero-Trust Architecture**: No implicit trust between services
- **Ed25519 Cryptography**: Faster dan more secure than RSA
- **Multi-Factor Authentication**: TOTP-based with authenticator apps
- **RBAC**: Granular permissions per microfrontend
- **Immutable Audit Trail**: PostgreSQL-backed forensic logging
- **Content Security Policy**: CSP headers untuk setiap module

---

## 📁 Project Structure

```
simpelv2/
├── 📁 antarmuka/                    # Microfrontend Applications
│   ├── 🏛️ portal/                  # Main Gateway & SSO
│   ├── 🎓 badiklat/                 # Training & Education
│   ├── ⚖️ datun/                    # Civil Litigation
│   ├── 🔍 intel/                    # Intelligence Analytics
│   ├── 💰 pemulihan_aset/           # Asset Recovery
│   ├── 🛡️ pengawasan/               # Monitoring & Compliance
│   ├── 🪖 pidmil/                   # Military Criminal Law
│   ├── 🔒 pidsus/                   # Special Crimes
│   ├── 📜 pidum/                    # General Crimes
│   ├── 📁 pembinaan/                # Pembinaan Division
│   │   ├── 💵 keuangan/             # Finance Management
│   │   ├── 📋 perencanaan/          # Planning
│   │   └── 🛠️ perlengkapan/         # Equipment Management
│   └── 🧩 shared/                   # Shared Component Library v1.0.0
│
├── 📁 layanan/                      # Backend Microservices
│   ├── 🎓 badiklat/                 # Training Service
│   ├── ⚖️ datun/                    # Civil Litigation Service
│   ├── 🔍 intel/                    # Intelligence Service
│   ├── 💰 pemulihan_aset/           # Asset Recovery Service
│   ├── 🛡️ pengawasan/               # Monitoring Service
│   ├── 🪖 pidmil/                   # Military Criminal Service
│   ├── 🔒 pidsus/                   # Special Crimes Service
│   ├── 📜 pidum/                    # General Crimes Service
│   ├── 📁 pembinaan/                # Pembinaan Services
│   │   └── 🛠️ perlengkapan/         # Equipment Service
│   └── 📁 shared/                   # Shared Services
│       ├── 🤖 ai/                   # AI/ML Service
│       ├── 🆘 bantuan/              # Help Desk Service
│       ├── 📊 dasbor/               # Dashboard Service
│       ├── 📄 dokumen/              # Document Service
│       ├── 🔗 integrasi/            # Integration Service
│       ├── ⚙️ konfigurasi/          # Configuration Service
│       ├── 📋 laporan/              # Reporting Service
│       └── 🔔 notifikasi/           # Notification Service
│
├── 📁 infra/                        # Infrastructure
│   ├── 🔐 authenc/                  # IAM Service (Separate Workspace)
│   ├── 🗝️ secreton/                 # Secret Management (Separate Workspace)
│   ├── 🚪 gerbang/                  # API Gateway (Envoy)
│   ├── ☸️ k8s/                      # Kubernetes Manifests
│   ├── 🌐 nginx/                    # Reverse Proxy Config
│   ├── 📡 proto/                    # Protocol Buffers
│   └── 📊 monitoring/               # Prometheus + Grafana
│
├── 📁 scripts/                      # Build & Automation
│   ├── 📁 cli/                      # CLI Tool
│   ├── 📁 makefiles/                # Modular Makefiles
│   ├── 📁 backup/                   # Backup Scripts
│   ├── 📁 test/                     # Test Automation
│   └── 📁 tools/                    # Development Tools
│
├── 📁 docs/                         # Documentation (60+ files)
│   ├── 📖 MFA_*.md                  # MFA Documentation
│   ├── 📖 AUTHENC_*.md              # Authenc Documentation
│   ├── 📖 SECRETON_*.md             # Secreton Documentation
│   ├── 📖 CAPTCHA_*.md              # CAPTCHA Documentation
│   └── 📖 layanan-*.md              # Service Documentation
│
├── 📄 Cargo.toml                    # Workspace Configuration
├── 📄 Makefile                      # Build Commands
├── 🐳 docker-compose.yml            # Base Docker Config
├── 🐳 docker-compose.dev.yml        # Development Override
├── 🐳 docker-compose.prod.yml       # Production Override
└── 📄 .gitlab-ci.yml                # CI/CD Pipeline
```

---

## 🚀 Quick Start

### Prerequisites

```bash
# System Requirements
- Docker & Docker Compose (v2 — prefer `docker compose`; see `docs/DOCKER_COMPOSE_MIGRATION.md`)
- Rust 1.90+ (for development)
- Trunk (for WASM builds)
- PostgreSQL 15+ (or use Docker)
- Redis 7+ (or use Docker)
```

### Installation

```bash
# Clone repository
git clone https://gitlab.com/analisiskebutuhan/simpelv2_web.git
cd simpelv2

# Setup environment
cp .env.example .env
# Edit .env with your configuration

# Start infrastructure services
docker compose up -d postgres redis

# Build and run (development)
make up-dev

# Or build specific microfrontend
cd antarmuka/portal && trunk serve --port 8080 --open
```

### Development Commands

```bash
# ===== Frontend Development =====
make build-all-fe          # Build all microfrontends
cd antarmuka/portal && trunk serve --open  # Serve portal with hot reload

# ===== Backend Development =====
cargo build --bin layanan-dasbor  # Build specific service
cargo build                       # Build all workspace members

# ===== Testing =====
cargo test                        # Run all tests
cargo test --package layanan-ai   # Test specific service
make rust-test                    # Comprehensive test suite

# ===== Quality =====
make rust-fmt                     # Format all code
make rust-clippy                  # Lint all code
cargo doc --open                  # Generate documentation

# ===== Infrastructure =====
make up-dev                       # Start development environment
make logs                         # View service logs
make down                         # Stop all services
```

### Building Microfrontends

```bash
# Build single microfrontend
cd antarmuka/portal
trunk build --release

# Build all microfrontends (parallel)
make build-all-fe

# Build with optimization
trunk build --release --config Trunk.toml
```

### Building Backend Services

```bash
# Build specific service
cargo build --bin layanan-dasbor
cargo build --bin layanan-ai

# Build all services
cargo build --workspace

# Release build
cargo build --release
```

---

## 🔄 CI/CD Pipeline

### GitLab CI/CD (9 Stages)

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Preparation │───▶│   Quality   │───▶│  Security   │
│  (Toolchain)│    │(Fmt/Clippy) │    │(Audit/SAST) │
└─────────────┘    └─────────────┘    └─────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│    Build    │───▶│    Test     │───▶│Security-Scan│
│(Parallel)   │    │(Unit/Integ) │    │(Trivy/IaC)  │
└─────────────┘    └─────────────┘    └─────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│    SBOM     │───▶│   Deploy    │───▶│   Cleanup   │
│(CycloneDX)  │    │(Dev/Prod)   │    │  (Cache)    │
└─────────────┘    └─────────────┘    └─────────────┘
```

| Stage             | Jobs                                    | Description                             |
| ----------------- | --------------------------------------- | --------------------------------------- |
| **Preparation**   | Dependencies                            | Rust toolchain, cargo tools, WASM tools |
| **Quality**       | Format, Clippy, Docs                    | Code quality dan documentation          |
| **Security**      | Audit, Deny, Geiger, Miri, Vet, SAST    | Static security analysis                |
| **Build**         | Chef, Rust Backend, Leptos Frontend     | Parallel builds dengan sccache          |
| **Test**          | Unit, Integration, Coverage, Fuzz       | Comprehensive testing suite             |
| **Security-Scan** | Trivy, License, Secrets, IaC, Container | Runtime security scanning               |
| **SBOM**          | Generate SBOM, Policy Validation        | Supply chain security                   |
| **Deploy**        | Dev, Staging, Production                | Multi-environment deployment            |
| **Cleanup**       | Cache, Security Summary                 | Cleanup dan reporting                   |

### Security Tools Integrated

| Tool             | Purpose                             |
| ---------------- | ----------------------------------- |
| **Cargo Audit**  | Vulnerability database scanning     |
| **Cargo Deny**   | Dependency policy enforcement       |
| **Cargo Geiger** | Unsafe code analysis                |
| **Cargo Vet**    | Supply chain verification           |
| **Semgrep**      | Static Application Security Testing |
| **Trivy**        | Filesystem & container scanning     |
| **TruffleHog**   | Secret detection                    |
| **Checkov**      | Infrastructure as Code security     |
| **Miri**         | Undefined behavior detection        |
| **OPA**          | Policy validation engine            |
| **Cosign**       | Container image signing             |
| **CycloneDX**    | SBOM generation                     |

---

## 📊 Monitoring & Observability

### Metrics Stack

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Services   │───▶│ Prometheus  │───▶│   Grafana   │
│  (Metrics)  │    │ (Scraping)  │    │ (Dashboard) │
└─────────────┘    └─────────────┘    └─────────────┘
       │                                     │
       │          ┌─────────────┐            │
       └─────────▶│    Loki     │◀───────────┘
                  │  (Logging)  │
                  └─────────────┘
```

### Collected Metrics

- **Application**: Response times, throughput, error rates
- **Infrastructure**: CPU, memory, disk, network
- **Business**: User engagement, feature adoption
- **Security**: Authentication events, threat detection
- **Microfrontend**: Load times, bundle sizes, user flows

### Alerting

- Service health monitoring
- Performance threshold alerts
- Security incident response
- Business process alerts

---

## 🔐 Authentication Flow

### Portal SSO Integration

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────────┐
│  User   │───▶│ Portal  │───▶│ Authenc │───▶│ Secreton    │
│         │    │ (Login) │    │  (IAM)  │    │ (MFA Keys)  │
└─────────┘    └────┬────┘    └────┬────┘    └─────────────┘
                    │              │
                    │   JWT Token  │
                    │◀─────────────┘
                    │
                    ▼
            ┌───────────────┐
            │ Microfrontend │
            │ (Protected)   │
            └───────────────┘
```

**Authentication Pattern:**

```rust
// Microfrontend integration
use lib_ui::hooks::use_auth;
use lib_ui::components::auth::ProtectedRoute;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=LoginRedirectPage />
                <Route path="/dashboard" view=|| {
                    view! { <ProtectedRoute><Dashboard /></ProtectedRoute> }
                }/>
            </Routes>
        </Router>
    }
}
```

> **Important**: Microfrontends NEVER handle credentials directly. All authentication goes through Portal + Authenc.

---

## 🧪 Testing

### Test Strategy

```bash
# Unit Tests
cargo test                           # All workspace tests
cargo test --package layanan-ai      # Specific service

# Integration Tests
make test-integration                # Full integration suite

# Frontend Tests
cd antarmuka/portal && trunk test    # WASM tests

# Security Tests
./scripts/security_validation.sh     # Security validation

# Performance Tests
cargo bench                          # Benchmarks
make load-test                       # Load testing
```

### Quality Metrics

- **Code Coverage**: Minimum 85% for production
- **Performance**: Response time <100ms average
- **Reliability**: 99.9% uptime SLA
- **Security**: Zero critical vulnerabilities

---

## ☸️ Kubernetes Deployment

### MicroK8s Production

```bash
# Build Portal microfrontend
cd antarmuka/portal
trunk build --release

# Deploy to MicroK8s
microk8s kubectl apply -f infra/k8s/microfrontends/portal-deployment.yaml
microk8s kubectl apply -f infra/k8s/ingress/ingress.yaml

# Check deployment
microk8s kubectl get pods,svc,ingress -n simpelv2

# Test endpoints
curl -k https://simpel.kejaksaan.go.id/ -I
```

### Namespace Organization

```
simpelv2/                    # Main namespace
simpelv2-frontend/           # Frontend microfrontends
simpelv2-backend/            # Backend microservices
simpelv2-infra/              # Infrastructure services
simpelv2-monitoring/         # Monitoring stack
```

---

## 🤝 Contributing

### Development Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.90
rustup component add clippy rustfmt

# Install frontend tools
cargo install trunk
cargo install wasm-pack

# Install development tools
cargo install cargo-audit
cargo install cargo-watch

# Setup project
git clone https://gitlab.com/analisiskebutuhan/simpelv2_web.git
cd simpelv2
make setup-dev
```

### Code Standards

- **Rust Conventions**: Follow Rust idioms + clippy linting
- **Security First**: Security-by-design approach
- **Comprehensive Testing**: Unit, integration, e2e tests
- **Documentation**: Clear comments + API documentation

### Contribution Process

1. **Create Issue**: Detailed issue dengan requirements
2. **Create Branch**: Feature branch dari main
3. **Implement**: Code dengan tests + documentation
4. **Quality Checks**: `make rust-fmt && make rust-clippy`
5. **Submit MR**: Merge request dengan description
6. **Review**: Peer review + automated checks
7. **Merge**: Automated deployment pipeline

---

## 📚 Documentation

### Core Documentation

| Document                                                                           | Description                 |
| ---------------------------------------------------------------------------------- | --------------------------- |
| [CONTRIBUTING.md](CONTRIBUTING.md)                                                 | Contribution guidelines     |
| [COMPLETE_AUTH_FLOW_ARCHITECTURE.md](antarmuka/COMPLETE_AUTH_FLOW_ARCHITECTURE.md) | Authentication architecture |
| [MFA_ARCHITECTURE_DOCUMENTATION.md](docs/MFA_ARCHITECTURE_DOCUMENTATION.md)        | MFA implementation          |
| [MICROFRONTEND_INTEGRATION.md](antarmuka/shared/MICROFRONTEND_INTEGRATION.md)      | Frontend integration guide  |

### Service Documentation

Located in `docs/` folder:

- `layanan-*.md` - Individual service documentation
- `MFA_*.md` - Multi-factor authentication docs
- `AUTHENC_*.md` - IAM service documentation
- `SECRETON_*.md` - Secret management docs
- `CAPTCHA_*.md` - CAPTCHA integration docs

---

## 🆘 Support

### Getting Help

- **Documentation**: `docs/` folder (60+ files)
- **Issues**: GitLab issue tracker
- **Architecture**: See `antarmuka/COMPLETE_AUTH_FLOW_ARCHITECTURE.md`

### Contact

- **Repository**: [gitlab.com/analisiskebutuhan/simpelv2_web](https://gitlab.com/analisiskebutuhan/simpelv2_web)
- **Production**: [simpel.kejaksaan.go.id](https://simpel.kejaksaan.go.id)

---

## 📄 License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

---

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for full change history.

**Recent Updates:**

- ✅ Rust 1.90+ (Edition 2024) migration
- ✅ Leptos 0.8.12 upgrade
- ✅ Shared component library v1.0.0 refactor
- ✅ Authenc + Secreton separate workspace architecture
- ✅ 9-stage CI/CD pipeline with 12+ security tools
- ✅ Zero-trust security implementation

---

## 🙏 Acknowledgments

- **Rust Community**: Amazing language dan ecosystem
- **Leptos Team**: High-performance reactive framework
- **Open Source Community**: All the amazing libraries

---

**🏛️ Built with ❤️ and Rust for Kejaksaan Agung Republik Indonesia**

_SIMPelv2 - Sistem Informasi Manajemen Pengelolaan BMN_
