# 🏛️ SIMPelv2 - Sistem Informasi Manajemen Pengelolaan BMN

**SIMPelv2** adalah platform modern untuk pengelolaan Barang Milik Negara (BMN) yang dibangun dengan arsitektur microservices dan microfron├── 📁 docs/                      # Comprehensive Documentation
│   ├── 📚 api/                  # API Documentation
│   ├── 🏗️ architecture/         # Architecture Diagrams
│   ├── 📘 guides/               # Development Guides
│   └── SHARED_OPTIMIZATION_COMPLETE.md # ⭐ Optimization Reports menggunakan **Rust** dan **Leptos** untuk performa, keamanan, dan pengalaman pengguna yang optimal.

## 🎯 **Overview**

SIMPelv2 adalah sistem terintegrasi yang menyediakan solusi lengkap untuk pengelolaan BMN, mulai dari perencanaan, pengadaan, distribusi, hingga pelaporan. Dibangun dengan teknologi modern Rust dan mengikuti standar keamanan enterprise dengan arsitektur microfrontend untuk skalabilitas dan maintainability maksimal.

## 🏗️ **Architecture**

### **🦀 Technology Stack**
- **Backend Services**: Rust (Axum) untuk performa dan keamanan tinggi
- **Frontend Microfrontends**: Leptos 0.7.8 + WebAssembly untuk speed dan type-safety
- **Shared UI Library**: Komponen terpusat dengan Kejaksaan RI branding
- **Build System**: Trunk 0.21.14 dengan optimasi WASM
- **Database**: PostgreSQL (multi-schema) dengan connection pooling
- **Gateway**: Envoy Proxy + Nginx dengan load balancing
- **Security**: HashiCorp Vault + JWT + MFA + Zero-Trust Architecture
- **AI/ML**: Rust-Bert + Tch + Qdrant untuk intelligent processing
- **Orchestration**: Docker Compose + Kubernetes dengan Helm charts
- **Monitoring**: Prometheus + Grafana + Loki + comprehensive observability

### **🌐 Microfrontend Architecture**
SIMPelv2 menggunakan arsitektur microfrontend dengan 11 modul independen yang didukung oleh **shared components library** yang telah dioptimasi dengan Leptos 0.8.x:

| Modul | Fungsi | Port | Status |
|-------|--------|------|--------|
| **Portal** | Gateway & Dashboard Utama | :8080 | ✅ Aktif |
| **Badiklat** | Pelatihan & Pendidikan | :8081 | ✅ Aktif |
| **Datun** | Tindak Pidana Umum | :8082 | ✅ Aktif |
| **Intel** | Intelligence & Analytics | :8083 | ✅ Aktif |
| **Pembinaan** | Manajemen Pembinaan | :8084-8086 | ✅ Aktif |
| **Pemulihan Aset** | Asset Recovery | :8087 | ✅ Aktif |
| **Pengawasan** | Monitoring & Compliance | :8088 | ✅ Aktif |
| **PIDMIL** | Pidana Militer | :8089 | ✅ Aktif |
| **PIDSUS** | Pidana Khusus | :8090 | ✅ Aktif |
| **PIDUM** | Pidana Umum | :8091 | ✅ Aktif |
| **Shared** | Komponen Terpusat | - | ✅ **v0.2.0 Optimized** |

### **🧩 Shared Components Library v0.2.0**
**Production-Ready UI Components dengan Leptos 0.8.x Compatibility**
- **40+ Components**: Button, Input, Modal, Table, Navigation, Form, Toast, Spinner, dll.
- **Zero Compilation Errors**: Full compatibility setelah resolusi 113+ compilation errors
- **Thread Safety**: Complete Send + Sync implementation untuk reactive components
- **Modern Patterns**: Updated signal patterns, callback methods, dan type annotations
- **Government Branding**: Kejaksaan RI design system dan accessibility compliance
- **Performance Optimized**: High-performance WASM dengan zero-copy operations

### **🔐 Security Features**
- **Zero-Trust Architecture**: Tidak ada implicit trust
- **Immutable Audit Trail**: Logging yang tidak dapat diubah
- **Multi-Factor Authentication**: TOTP-based security
- **Role-Based Access Control**: Granular permissions per microfrontend
- **Content Security Policy**: CSP headers untuk setiap modul
- **Honeytrap Service**: Advanced threat detection

### **🤖 AI Capabilities**
- **LLM Integration**: Internal fine-tuned models
- **RAG System**: Retrieval-Augmented Generation
- **OCR Processing**: Document text extraction
- **Supervised Learning**: Traditional ML models
- **RLHF**: Reinforcement Learning from Human Feedback

## 🚀 **Quick Start**

### **Prerequisites**
```bash
# System requirements
- Docker & Docker Compose
- Rust 1.75+ (for development)
- PostgreSQL 15+
- HashiCorp Vault
```

### **Installation**
```bash
# Clone repository
git clone https://gitlab.com/analisiskebutuhan/simpelv2_web.git
cd simpelv2

# Setup environment
cp .env.example .env
# Edit .env with your configuration

# Start services
make up-dev

# Access the application
# Frontend: http://localhost:3000
# API Gateway: http://localhost:8080
# Security Service: http://localhost:3001
# AI Service: http://localhost:3002
```

### **Development**
```bash
# Start development environment
make up-dev

# View logs
make logs

# Stop services
make down

# Build and deploy
make build
make deploy-prod
```

## 🚀 **CI/CD Pipeline**

### **Enterprise-Grade GitLab CI/CD (v1.0.0)**
SIMPelv2 menggunakan pipeline CI/CD modern dengan 9 stages dan 12+ security tools:

| Stage | Jobs | Description |
|-------|------|-------------|
| **Preparation** | Dependencies | Rust toolchain, cargo tools, WASM tools |
| **Quality** | Format, Clippy, Spellcheck, Docs | Code quality dan documentation |
| **Security** | Audit, Deny, Geiger, Miri, Vet, SAST | Static security analysis |
| **Build** | Chef, Rust Backend, Leptos Frontend | Parallel builds dengan caching |
| **Test** | Unit, Integration, Coverage, Performance, Fuzz | Comprehensive testing suite |
| **Security-Scan** | Trivy FS, License, Secrets, IaC, Container | Runtime security scanning |
| **SBOM** | Generate SBOM, Policy Validation, Image Build | Supply chain security |
| **Deploy** | Dev, Review, Staging, Production | Multi-environment deployment |
| **Cleanup** | Cache, Security Summary | Cleanup dan reporting |

### **🔐 Security Tools Integrated**
- **Cargo Audit**: Vulnerability database scanning
- **Cargo Deny**: Dependency policy enforcement
- **Cargo Geiger**: Unsafe code analysis
- **Cargo Vet**: Supply chain verification
- **Semgrep**: Static Application Security Testing (SAST)
- **Trivy**: Filesystem dan container scanning
- **TruffleHog**: Secret detection
- **Checkov**: Infrastructure as Code security
- **Miri**: Undefined behavior detection
- **OPA**: Policy validation engine
- **Cosign**: Container image signing
- **SBOM**: Software Bill of Materials (CycloneDX + SPDX)

### **⚡ Performance Optimizations**
- **Cargo Chef**: Docker build optimization
- **sccache**: Distributed compilation caching (2G cache)
- **Parallel Builds**: Matrix strategy untuk 7 backend + 9 frontend
- **Multi-level Caching**: Cargo.lock fingerprinting
- **Distroless Images**: Minimal attack surface

### **📋 Compliance & Standards**
- **SLSA Level 3**: Supply chain security framework
- **Zero-Trust Architecture**: Security by design
- **OWASP Guidelines**: Web application security
- **ISO/IEC 25010**: Software quality standards

## 📁 **Project Structure**

```
simpelv2/
├── 📁 antarmuka/                 # Microfrontend Leptos Applications
│   ├── 🏛️ portal/              # Main Dashboard & Gateway (8080)
│   ├── 🎓 badiklat/             # Training & Education (8081)
│   ├── ⚖️ datun/               # Criminal Prosecution (8082)
│   ├── � intel/               # Intelligence Analytics (8083)
│   ├── 📋 pembinaan/           # Development Management
│   │   ├── 💰 keuangan/        # Financial Management (8084)
│   │   ├── 📊 perencanaan/     # Planning & Strategy (8085)
│   │   └── 🛠️ perlengkapan/    # Equipment Management (8086)
│   ├── 🔄 pemulihan_aset/      # Asset Recovery (8087)
│   ├── �️ pengawasan/          # Monitoring & Compliance (8088)
│   ├── 🪖 pidmil/             # Military Criminal Law (8089)
│   ├── 🔒 pidsus/             # Special Crimes (8090)
│   ├── 📜 pidum/              # General Crimes (8091)
│   └── 🧩 shared/             # ⭐ Shared UI Components Library v0.2.0
│       ├── src/
│       │   ├── components.rs   # 40+ Optimized Components
│       │   ├── types.rs        # Modern Type System
│       │   ├── constants.rs    # Government Data Constants
│       │   ├── theme.rs        # Kejaksaan RI Design System
│       │   └── utils.rs        # Utility Functions
│       └── Cargo.toml          # Leptos 0.8.x Dependencies
├── 📁 layanan/                   # Backend Microservices (Rust)
│   ├── 🔐 keamanan/             # Security Service (3001)
│   ├── 🤖 ai/                   # AI/ML Service (3002)
│   ├── 📄 dokumen/              # Document Management (3003)
│   ├── ⚙️ konfigurasi/          # Configuration Service (3004)
│   ├── 🆘 bantuan/              # Help & Support (3005)
│   ├── 📊 dasbor/               # Dashboard Service (3006)
│   ├── 📋 laporan/              # Reporting Service (3007)
│   ├── 🔗 integrasi/            # External Integration (3008)
│   └── 🔔 notifikasi/           # Notification Service (3009)
├── 📁 infra/                     # Infrastructure & DevOps
│   ├── 🌐 nginx/                # Nginx Reverse Proxy
│   ├── 🚪 gerbang/              # API Gateway (Envoy)
│   ├── 📁 k8s/                  # Kubernetes Manifests
│   ├── � vault/                # HashiCorp Vault Config
│   └── � monitoring/           # Observability Stack
├── 📁 scripts/                   # Build & Automation Scripts
│   ├── 🔧 tools/                # Development Tools
│   ├── 🧪 test/                 # Test Automation
│   └── 📦 makefiles/            # Make Configurations
├── 📁 docs/                      # Comprehensive Documentation
│   ├── 📚 api/                  # API Documentation
│   ├── 🏗️ architecture/         # Architecture Diagrams
│   └── � guides/               # Development Guides
└── 📁 target/                    # Rust Build Artifacts
    ├── debug/                   # Development Builds
    ├── release/                 # Production Builds
    └── wasm32-unknown-unknown/  # WebAssembly Builds
```

## 🎨 **Frontend Microfrontends**

### **🏛️ Portal Dashboard**
**Primary Gateway & Unified Dashboard**
- **Port**: `:8080`
- **Function**: Main entry point, authentication, navigation
- **Technology**: Leptos 0.7.8 + WebAssembly
- **Features**: Single Sign-On, role-based routing, system overview

### **🎓 Badiklat Training System**
**Training & Education Management**
- **Port**: `:8081`
- **Function**: Training programs, certifications, learning paths
- **Features**: Course management, progress tracking, assessments

### **⚖️ Datun Criminal Prosecution**
**General Criminal Case Management**
- **Port**: `:8082`
- **Function**: Case tracking, prosecution workflow, legal documents
- **Features**: Case assignment, timeline management, evidence tracking

### **🔍 Intel Analytics Platform**
**Intelligence & Data Analytics**
- **Port**: `:8083`
- **Function**: Data visualization, intelligence reports, analytics dashboards
- **Features**: Real-time monitoring, predictive analytics, custom reports
- **Upload/Download**: File management
- **Preview**: PDF/Word document preview
- **Classification**: AI-powered tagging
- **Encryption**: In-transit & at-rest encryption

### **⚙️ Configuration Service**
- **Dynamic Metadata**: Categories, tags, reference codes
- **User Preferences**: Display settings
- **JSONB Config**: Flexible configuration storage

### **🆘 Help Service**
- **FAQ Management**: Knowledge base
- **Ticket System**: Support requests
- **AI Chatbot**: Intelligent assistance
- **Integration**: Notification & document linking

### **📊 Dashboard Service**
- **Performance Summary**: Institutional overview
- **Data Visualization**: Cross-service analytics
- **Microfrontend Integration**: Modular UI components

### **📋 Reporting Service**
- **Dynamic Reports**: User-generated reports
## 🔧 **Backend Microservices**

### **🔐 Security Service (Rust)**
- **JWT Authentication**: Token management & validation
- **Multi-Factor Authentication**: TOTP-based security
- **Role-Based Access Control**: Granular permissions per microfrontend
- **HashiCorp Vault Integration**: Secret management
- **Immutable Audit Trail**: Forensic capabilities

**Port**: `3001` | **Health**: `/health`

### **🤖 AI Service (Rust)**
- **LLM Integration**: Internal fine-tuned models untuk analisa dokumen
- **RAG System**: Retrieval-Augmented Generation untuk Q&A
- **OCR Processing**: Document text extraction dengan AI
- **Supervised Learning**: Traditional ML models
- **RLHF**: Reinforcement Learning from Human Feedback

**Port**: `3002` | **Health**: `/health`

### **📄 Document Service (Rust)**
- **Document Management**: Upload, versioning, storage
- **Full-text Search**: Advanced search capabilities
- **AI Classification**: Automated categorization
- **Digital Signatures**: PKI-based document signing
- **Workflow Integration**: Document approval processes

**Port**: `3003` | **Health**: `/health`

### **� Dashboard Service (Rust)**
- **Real-time Metrics**: Live performance indicators
- **Custom Dashboards**: Per-role dashboard configurations
- **Data Visualization**: Charts, graphs, analytics
- **Pivot Tables**: Interactive data analysis
- **Export Functions**: PDF, Excel, CSV output

**Port**: `3006` | **Health**: `/health`

### **📋 Reporting Service (Rust)**
- **Dynamic Reports**: Template-based report generation
- **Scheduled Reports**: Automated report delivery
- **Data Warehouse**: Filtered reporting capabilities
- **AI Integration**: Automated insights generation
- **Multi-format Output**: PDF, Excel, Word, JSON

**Port**: `3007` | **Health**: `/health`

## 🛡️ **Security Architecture**

### **🔐 Authentication Flow**
```
User Login → JWT Token → MFA Verification → Role Assignment → Microfrontend Access
```

### **🛡️ Multi-Layer Security**
1. **Network Security**: Envoy Gateway + Nginx with SSL termination
2. **Application Security**: Rust memory safety + type checking
3. **Authentication**: JWT + MFA + RBAC per service
4. **Secret Management**: HashiCorp Vault integration
5. **Audit Trail**: Immutable logging across all microfrontends
6. **Content Security Policy**: CSP headers for each frontend module
7. **Threat Detection**: Honeytrap service + anomaly detection

### **📊 Compliance Standards**
- **ISO 27001**: Information security management system
- **PCI DSS**: Payment card industry data security
- **GDPR**: General data protection regulation compliance
- **SOX**: Sarbanes-Oxley financial reporting compliance

## 🤖 **AI Capabilities**

### **🧠 Machine Learning Models**
- **LLM**: Fine-tuned language models
- **OCR**: Document text extraction
- **Classification**: Document categorization
- **Recommendation**: Smart suggestions
- **Anomaly Detection**: Security monitoring

### **🔄 Learning Approaches**
- **Supervised Learning**: Traditional ML models
- **RLHF**: Reinforcement Learning from Human Feedback
- **Active Learning**: Query optimization
- **Transfer Learning**: Model adaptation
- **HITL**: Human-in-the-loop annotation

## 📊 **Performance Metrics**

### **🔐 Security Service**
- **JWT Generation**: ~1ms per token
- **Password Verification**: ~10ms per verification
- **MFA Verification**: ~5ms per code
- **Audit Logging**: ~2ms per log entry

### **🤖 AI Service**
- **LLM Inference**: ~50ms per request
- **OCR Processing**: ~100ms per page
- **RAG Query**: ~20ms per query
- **Model Loading**: ~2s startup time

## 🚀 **Deployment**

## 🚀 **Quick Start**

### **Prerequisites**
```bash
# System requirements
- Docker & Docker Compose 20+
- Rust 1.75+ (for development)
- Node.js 18+ & Trunk 0.21.14 (for microfrontends)
- PostgreSQL 15+
- HashiCorp Vault
```

### **🚀 Development Setup**
```bash
# Clone repository
git clone https://gitlab.com/analisiskebutuhan/simpelv2_web.git
cd simpelv2

# Setup development environment
make setup-dev

# Start all services
make up-dev

# Build microfrontends
make build-frontends

# Access applications
make open-portal        # Opens http://localhost:8080
```

### **🌐 Access Points**
| Service | URL | Description |
|---------|-----|-------------|
| **Portal** | http://localhost:8080 | Main Dashboard & Gateway |
| **Badiklat** | http://localhost:8081 | Training System |
| **Datun** | http://localhost:8082 | Criminal Prosecution |
| **Intel** | http://localhost:8083 | Intelligence Platform |
| **API Gateway** | http://localhost:8000 | Backend API Gateway |
| **Vault UI** | http://localhost:8200 | HashiCorp Vault |
| **Grafana** | http://localhost:3000 | Monitoring Dashboard |

### **🔧 Development Commands**
```bash
# Frontend development
make serve-portal       # Serve portal with hot reload
make build-all-fe      # Build all microfrontends
make test-frontends    # Test all frontend modules

# Backend development
make dev-backend       # Start backend services
make test-backend      # Run backend tests
make clippy           # Rust linting

# Infrastructure
make up-infra         # Start infrastructure only
make logs-all         # View all service logs
make clean-all        # Clean all build artifacts
```

### **📦 Production Deployment**
```bash
# Build production images
make build-prod

# Deploy to staging
make deploy-staging

# Deploy to production
make deploy-prod

# Monitor deployment
make monitor-prod
```

### **☸️ Kubernetes Deployment**
```bash
# Generate K8s manifests
make generate-k8s

# Deploy to cluster
make deploy-k8s

# Check status
kubectl get pods -n simpelv2

# Monitor with Grafana
make monitor-k8s
```

## 📚 **Documentation**

### **📖 Core Documentation**
- [🏗️ Architecture Overview](docs/architecture/README.md)
- [🔐 Security Guide](docs/security/README.md)
- [🤝 Contributing Guidelines](CONTRIBUTING.md)
- [🚀 Deployment Guide](docs/deployment/README.md)
- [📋 API Documentation](docs/api/README.md)

### **📱 Frontend Documentation**
- [🎨 Microfrontend Architecture](docs/MICROFRONTEND_STATUS_REPORT.md)
- [🧩 Shared Components Guide](antarmuka/shared/README.md)
- [🎯 Portal System](antarmuka/portal/README.md)
- [🔧 Build & Optimization](docs/ANTARMUKA_OPTIMIZATION_FINAL_REPORT.md)

### **⚙️ Backend Documentation**
- [🔐 Security Service](layanan/keamanan/README.md)
- [🤖 AI/ML Service](layanan/ai/README.md)
- [📄 Document Service](layanan/dokumen/README.md)
- [📊 Dashboard Service](layanan/dasbor/README.md)

### **📋 API Documentation**
- **OpenAPI 3.0**: Interactive API documentation
- **Postman Collection**: Comprehensive API testing suite
- **Swagger UI**: Visual API explorer dengan examples
- **GraphQL**: Real-time query interface untuk analytics

## 🔍 **Quality Assurance**

### **🧪 Testing Strategy**
- **Unit Tests**: Rust services dengan coverage >90%
- **Integration Tests**: End-to-end microfrontend testing
- **Performance Tests**: Load testing dengan K6
- **Security Tests**: Penetration testing automation
- **UI Tests**: Playwright untuk semua microfrontends

### **📊 Quality Metrics**
- **Code Coverage**: Minimum 85% untuk production
- **Performance**: Response time <100ms average
- **Reliability**: 99.9% uptime SLA
- **Security**: Zero critical vulnerabilities
- **Compliance**: ISO 27001 & SOX compliance

### **🔄 CI/CD Pipeline**
```bash
# Quality checks pipeline
Code Push → Pre-commit Hooks → Unit Tests → Integration Tests
         → Security Scan → Build → Deploy Staging → E2E Tests
         → Deploy Production → Health Check → Monitoring
```

## 📈 **Monitoring & Observability**

### **📊 Comprehensive Metrics**
- **Application Performance**: Response times, throughput, error rates
- **Infrastructure Health**: CPU, memory, disk, network utilization
- **Business Intelligence**: User engagement, feature adoption
- **Security Monitoring**: Authentication events, threat detection
- **Microfrontend Metrics**: Load times, bundle sizes, user flows

### **📝 Structured Logging**
- **Centralized Logging**: Loki + Grafana stack
- **Structured JSON**: Consistent log format across services
- **Immutable Audit Trail**: Security & compliance logging
- **Real-time Monitoring**: Live log streaming & analysis
- **Performance Tracing**: Distributed request tracing

### **⚠️ Intelligent Alerting**
- **Service Health Monitoring**: Automated health checks
- **Performance Thresholds**: SLA-based alerting
- **Security Incident Response**: Automated threat detection
- **Business Process Alerts**: Critical workflow monitoring
- **Predictive Alerts**: AI-powered anomaly detection

## 🤝 **Contributing**

### **🛠️ Development Setup**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add clippy rustfmt

# Install frontend tools
cargo install trunk
npm install -g wasm-pack

# Install development tools
cargo install sqlx-cli
cargo install cargo-audit
cargo install cargo-watch

# Setup project
git clone https://gitlab.com/analisiskebutuhan/simpelv2_web.git
cd simpelv2
make setup-dev
```

### **📋 Development Standards**
- **Code Quality**: Rust conventions + clippy linting
- **Security First**: Security-by-design approach
- **Comprehensive Testing**: Unit, integration, e2e tests
- **Documentation**: Clear comments + API documentation
- **Performance**: Benchmarking + optimization

### **🔄 Contribution Process**
1. **📋 Issue Creation**: Create detailed issue dengan requirements
2. **🌿 Branch Creation**: Feature branch dari main branch
3. **💻 Development**: Implement dengan tests + documentation
4. **🧪 Quality Checks**: Run tests, linting, security scans
5. **📝 Merge Request**: Submit dengan comprehensive description
6. **👁️ Code Review**: Peer review + automated checks
7. **🚀 Merge & Deploy**: Automated deployment pipeline

## 🧪 **Testing**

### **Unit Tests**
```bash
# Run all tests
cargo test

# Run specific service tests
cargo test --package layanan-keamanan
cargo test --package layanan-ai
```

### **Integration Tests**
```bash
# Run integration tests
make test-integration

# Run security tests
make test-security
```

### **Performance Tests**
```bash
# Run benchmarks
cargo bench

# Load testing
make load-test
```

## 🔄 **CI/CD Pipeline**

### **GitLab CI**
- **Build**: Multi-stage Docker builds
- **Test**: Automated testing suite
- **Security**: Vulnerability scanning
- **Deploy**: Automated deployment

### **Quality Gates**
- **Code Coverage**: >80% coverage required
- **Security Scan**: No critical vulnerabilities
- **Performance**: Response time <100ms
- **Compliance**: Security standards met

## 📈 **Monitoring & Observability**

### **Metrics**
- **Application Metrics**: Response times, error rates
- **Infrastructure Metrics**: CPU, memory, disk usage
- **Business Metrics**: User activity, feature usage
- **Security Metrics**: Authentication, authorization events

### **Logging**
- **Structured Logging**: JSON format
- **Centralized Logging**: Loki + Grafana
- **Audit Logging**: Immutable security logs
- **Performance Logging**: Request tracing

### **Alerting**
- **Service Health**: Automatic health checks
- **Performance Alerts**: Response time thresholds
- **Security Alerts**: Suspicious activity detection
- **Business Alerts**: Critical business events

## 🤝 **Contributing**

### **Development Setup**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
cargo install sqlx-cli
cargo install cargo-audit

# Setup development environment
make setup-dev
```

### **Code Standards**
- **Rust**: Follow Rust conventions
- **Security**: Security-first development
- **Testing**: Comprehensive test coverage
- **Documentation**: Clear and concise docs

### **Pull Request Process**
1. **Fork** the repository
2. **Create** feature branch
3. **Implement** changes with tests
4. **Submit** pull request
5. **Review** and merge

## 🆘 **Support**

### **Getting Help**
- **Documentation**: Comprehensive guides
- **Issues**: GitHub issue tracker
- **Discussions**: Community forum
- **Security**: Responsible disclosure

### **Contact**
- **Email**: support@simpelv2.go.id
- **Slack**: #simpelv2-support
- **GitHub**: Issues and discussions

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.



## 📝 **Changelog**

Lihat file `CHANGELOG.md` untuk riwayat perubahan lengkap. Perubahan terakhir:

- Workspace Cargo.toml: Semua layanan backend dan shared sudah terdaftar di `[workspace].members`.
- SIMPelv2.code-workspace: Semua backend dan shared sudah di-link ke Rust Analyzer, serta task build/check/test backend sudah tersedia.
- Struktur workspace kini siap build/test lintas layanan dan kolaborasi tim.

## 🙏 **Acknowledgments**

- **Rust Community**: For the amazing language and ecosystem
- **Axum Team**: For the high-performance web framework
- **HashiCorp**: For enterprise security tools
- **Open Source Community**: For all the amazing libraries

---

**🏛️ Built with ❤️ and Rust for maximum security and performance**

*SIMPelv2 - Sistem Informasi Manajemen Pengelolaan BMN*
