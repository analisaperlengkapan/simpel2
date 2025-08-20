# 🏛️ SIMPelv2 - Sistem Informasi Manajemen Pengelolaan BMN

**SIMPelv2** adalah platform modern untuk pengelolaan Barang Milik Negara (BMN) yang dibangun dengan arsitektur microservices menggunakan **Rust** untuk performa dan keamanan maksimal.

## 🎯 **Overview**

SIMPelv2 adalah sistem terintegrasi yang menyediakan solusi lengkap untuk pengelolaan BMN, mulai dari perencanaan, pengadaan, distribusi, hingga pelaporan. Dibangun dengan teknologi modern dan mengikuti standar keamanan enterprise.

## 🏗️ **Architecture**

### **🦀 Technology Stack**
- **Backend**: Rust (Axum) untuk performa dan keamanan
- **Frontend**: React + Vite + Tailwind CSS
- **Database**: PostgreSQL (multi-schema)
- **Gateway**: Envoy Proxy + Nginx
- **Security**: HashiCorp Vault + JWT + MFA
- **AI/ML**: Rust-Bert + Tch + Qdrant
- **Orchestration**: Docker Compose + Kubernetes
- **Monitoring**: Prometheus + Grafana + Loki

### **🔐 Security Features**
- **Zero-Trust Architecture**: Tidak ada implicit trust
- **Immutable Audit Trail**: Logging yang tidak dapat diubah
- **Multi-Factor Authentication**: TOTP-based security
- **Role-Based Access Control**: Granular permissions
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

## 📁 **Project Structure**

```
simpelv2/
├── 📁 antarmuka/                 # Frontend React Vite
├── 📁 gerbang/                   # API Gateway (Envoy)
├── 📁 nginx/                     # Reverse Proxy
├── 📁 layanan/                   # Microservices (Rust)
│   ├── 🔐 keamanan/             # Security Service
│   ├── 🤖 ai/                   # AI/ML Service
│   ├── 📄 dokumen/              # Document Management
│   ├── ⚙️ konfigurasi/          # Configuration Service
│   ├── 🆘 bantuan/              # Help & Support
│   ├── 📊 dasbor/               # Dashboard Service
│   ├── 📋 laporan/              # Reporting Service
│   ├── 🔗 integrasi/            # External Integration
│   └── 🔔 notifikasi/           # Notification Service
├── 📁 infra/                     # Infrastructure
│   ├── 📁 k8s/                  # Kubernetes manifests
│   ├── 📁 vault/                # HashiCorp Vault
│   └── 📁 monitoring/           # Observability stack
├── 📁 scripts/                   # Automation scripts
├── 📁 docs/                      # Documentation
└── 📁 dist/                      # Build artifacts
```

## 🔧 **Services**

### **🔐 Security Service (Rust)**
- **JWT Authentication**: Token management
- **Multi-Factor Authentication**: TOTP-based security
- **Role-Based Access Control**: Granular permissions
- **HashiCorp Vault Integration**: Secret management
- **Immutable Audit Trail**: Forensic capabilities

**Port**: `3001`
**Health Check**: `http://localhost:3001/health`

### **🤖 AI Service (Rust)**
- **LLM Integration**: Internal fine-tuned models
- **RAG System**: Retrieval-Augmented Generation
- **OCR Processing**: Document text extraction
- **Supervised Learning**: Traditional ML models
- **RLHF**: Reinforcement Learning from Human Feedback

**Port**: `3002`
**Health Check**: `http://localhost:3002/health`

### **📄 Document Service**
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
- **Pivot Tables**: Data analysis
- **Data Warehouse**: Filtered reporting
- **AI Integration**: Automated insights

### **🔗 Integration Service**
- **External APIs**: SIMAN, MONSAKTI, MySimkari, SIPEDE
- **Webhook Support**: Real-time synchronization
- **Scheduled Sync**: Automated data updates
- **Mutual TLS**: Secure communication

### **🔔 Notification Service**
- **Multi-channel**: Email, WhatsApp, Push notifications
- **Dynamic Templates**: Event-based messaging
- **Service Integration**: Used by all other services

## 🛡️ **Security Architecture**

### **🔐 Authentication Flow**
```
User Login → JWT Token → MFA Verification → Role Assignment → Access Control
```

### **🛡️ Security Layers**
1. **Network Security**: Envoy Gateway + Nginx
2. **Application Security**: Rust memory safety
3. **Authentication**: JWT + MFA + RBAC
4. **Secret Management**: HashiCorp Vault
5. **Audit Trail**: Immutable logging
6. **Threat Detection**: Honeytrap service

### **📊 Compliance**
- **ISO 27001**: Information security management
- **PCI DSS**: Payment card industry standards
- **GDPR**: Data protection regulations
- **SOX**: Financial reporting compliance

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

### **Development Environment**
```bash
# Start development
make up-dev

# View logs
make logs

# Stop services
make down
```

### **Production Environment**
```bash
# Deploy to production
make deploy-prod

# Monitor services
make logs-prod

# Scale services
make scale-prod
```

### **Kubernetes Deployment**
```bash
# Deploy to Kubernetes
make generate-k8s
make deploy-k8s

# Monitor with Grafana
make monitor
```

## 📚 **Documentation**

### **📖 Service Documentation**
- [Security Service](layanan/keamanan/README.md)
- [AI Service](layanan/ai/README.md)
- [Architecture Decisions](DOCKER_COMPOSE_STRATEGY.md)
- [Contributing Guidelines](CONTRIBUTING.md)

### **📋 API Documentation**
- **OpenAPI 3.0**: Interactive API docs
- **Postman Collection**: API testing
- **Swagger UI**: Visual API explorer

### **🏗️ Architecture Documentation**
- [System Design](docs/architecture/)
- [Security Architecture](docs/security/)
- [Deployment Guides](docs/deployment/)

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

## 🙏 **Acknowledgments**

- **Rust Community**: For the amazing language and ecosystem
- **Axum Team**: For the high-performance web framework
- **HashiCorp**: For enterprise security tools
- **Open Source Community**: For all the amazing libraries

---

**🏛️ Built with ❤️ and Rust for maximum security and performance**

*SIMPelv2 - Sistem Informasi Manajemen Pengelolaan BMN*
