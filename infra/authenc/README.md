# Authenc - Enterprise Authentication & Authorization Server

**Production-ready Identity and Access Management (IAM) service** built in Rust with enterprise-grade security features. Comprehensive authentication, authorization, and federation capabilities with modern cryptography and zero-trust architecture.

## 🔐 Core Features

- **🔒 Advanced Security**: Ed25519 cryptography, timing-attack immunity, mTLS support
- **🛡️ Zero Trust Architecture**: Continuous authentication, anomaly detection, risk-based access
- **🔗 Multi-Protocol Federation**: SAML 2.0, OIDC, OAuth2 with identity brokering
- **📱 Multi-Factor Authentication**: TOTP, WebAuthn/FIDO2, hardware security keys
- **👥 User & Organization Management**: RBAC, multi-tenancy, hierarchical permissions
- **📊 Comprehensive Audit Logging**: GDPR-compliant audit trails with PostgreSQL persistence
- **🚀 High Performance**: Async-first architecture with Redis caching and connection pooling
- **☁️ Cloud Native**: Kubernetes-ready with observability and metrics collection

## 🏗️ Architecture

Authenc implements a modern, scalable microservices architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Clients       │───▶│   Authenc API    │───▶│   PostgreSQL    │
│   (Web/Mobile)  │    │   (Rust/Axum)    │    │   (Primary DB)  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                        │                     │
         ▼                        ▼                     ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   External      │    │   Redis Cache    │    │   Audit Logs    │
│   Identity      │    │   (Optional)     │    │   (PostgreSQL)  │
│   Providers     │    └──────────────────┘    └─────────────────┘
└─────────────────┘
```

### Core Components

- **Authentication Engine**: Multi-protocol auth with SAML, OIDC, OAuth2 support
- **Authorization Service**: RBAC/ABAC with policy-based access control
- **Federation Manager**: Identity brokering and JIT user provisioning
- **Security Services**: MFA, device trust scoring, anomaly detection
- **Audit System**: Comprehensive logging with compliance reporting
- **Secret Management**: Secure vault integration with runtime secret fetching

## 🚀 Implemented Protocols & Standards

### Authentication Protocols

- **OAuth 2.0**: Full RFC 6749 implementation with PKCE, introspection, revocation
- **OIDC**: OpenID Connect with Ed25519-signed JWT tokens (timing-attack resistant)
- **SAML 2.0**: Complete Service Provider implementation with metadata exchange
- **WebAuthn/FIDO2**: Passwordless authentication with hardware security keys

### Security Standards

- **FIPS 140-3**: Cryptographic module validation ready
- **Zero Trust**: Continuous authentication and risk assessment
- **GDPR Compliance**: User consent management and data protection
- **Security Headers**: Comprehensive HTTP security headers

## 📚 API Endpoints

### Core Authentication

```http
POST   /oauth2/authorize           # OAuth2 authorization endpoint
POST   /oauth2/token              # Token endpoint (all grant types)
POST   /oauth2/introspect         # Token introspection (RFC 7662)
POST   /oauth2/revoke            # Token revocation (RFC 7009)
GET    /oauth2/userinfo          # User information endpoint
GET    /oauth2/jwks              # JWK Set endpoint
```

### OpenID Connect (OIDC)

```http
GET    /.well-known/openid_configuration  # OIDC discovery
GET    /oidc/jwks                        # OIDC JWK Set
POST   /oidc/token                       # OIDC token endpoint
GET    /oidc/userinfo                    # OIDC user info
```

### Identity Federation

```http
GET    /api/v1/auth/federated/*     # Federated authentication routes
GET    /api/v1/auth/broker/*        # Identity broker endpoints
POST   /api/v1/auth/social/*        # Social login integration
```

### User Management

```http
GET    /api/v1/auth/users           # List users
POST   /api/v1/auth/users           # Create user
GET    /api/v1/auth/users/{id}      # Get user details
PUT    /api/v1/auth/users/{id}      # Update user
DELETE /api/v1/auth/users/{id}      # Delete user
```

### Role & Permission Management

```http
GET    /api/v1/auth/roles           # List roles
POST   /api/v1/auth/roles           # Create role
GET    /api/v1/auth/permissions     # List permissions
POST   /api/v1/auth/permissions     # Create permission
```

### Organization Management

```http
GET    /api/v1/organizations        # List organizations
POST   /api/v1/organizations        # Create organization
GET    /api/v1/organizations/{id}   # Get organization
PUT    /api/v1/organizations/{id}   # Update organization
```

### Multi-Factor Authentication

```http
POST   /api/v1/auth/users/{id}/totp          # Enable TOTP
POST   /api/v1/auth/users/{id}/totp/verify   # Verify TOTP
POST   /api/v1/auth/webauthn/register        # WebAuthn registration
POST   /api/v1/auth/webauthn/authenticate    # WebAuthn authentication
```

### Audit & Monitoring

```http
GET    /health                      # Health check
GET    /ready                       # Readiness check
GET    /live                        # Liveness check
GET    /api/v1/auth/audit/logs      # Audit log entries
```

## ⚙️ Configuration

### Environment Variables

| Variable                        | Default                   | Description                                     |
| ------------------------------- | ------------------------- | ----------------------------------------------- |
| `DATABASE_URL`                  | Required                  | PostgreSQL connection string                    |
| `ED25519_PRIVATE_KEY_BASE64`    | **REQUIRED (Production)** | **Ed25519 signing key (base64-encoded)**        |
| `ECDSA_P256_PRIVATE_KEY_BASE64` | Optional                  | Alternative: ECDSA P-256 signing key            |
| `ECDSA_P384_PRIVATE_KEY_BASE64` | Optional                  | Alternative: ECDSA P-384 signing key            |
| `ECDSA_P521_PRIVATE_KEY_BASE64` | Optional                  | Alternative: ECDSA P-521 signing key            |
| `AUTHENC_PORT`                  | `8080`                    | Server port                                     |
| `AUTHENC_HOST`                  | `0.0.0.0`                 | Server bind address                             |
| `JWT_SECRET`                    | Required                  | JWT signing secret (legacy compatibility)       |
| `LOG_LEVEL`                     | `info`                    | Logging level (trace, debug, info, warn, error) |
| `TLS_ENABLE`                    | `false`                   | Enable TLS/HTTPS                                |
| `TLS_CERT_FILE`                 | -                         | Path to TLS certificate                         |
| `TLS_KEY_FILE`                  | -                         | Path to TLS private key                         |
| `MTLS_ENABLE`                   | `false`                   | Enable mutual TLS                               |
| `REDIS_URL`                     | -                         | Redis connection for caching                    |
| `SECRETON_ENDPOINT`             | -                         | Secreton vault endpoint                         |

⚠️ **CRITICAL FOR PRODUCTION**: You MUST set a persistent signing key (`ED25519_PRIVATE_KEY_BASE64` or equivalent) to prevent JWT tokens from being invalidated on every restart. See [`docs/SIGNING_KEY_SETUP.md`](docs/SIGNING_KEY_SETUP.md) for detailed instructions.

### Feature Flags

| Feature         | Description                     |
| --------------- | ------------------------------- |
| `axum`          | Axum web framework integration  |
| `auth`          | JWT and password authentication |
| `oidc`          | OpenID Connect provider         |
| `db`            | PostgreSQL database integration |
| `metrics`       | Prometheus metrics collection   |
| `admin_console` | Web-based admin interface       |

## 🔧 Installation & Setup

### Prerequisites

- **Rust 1.75+** - For building from source
- **PostgreSQL 13+** - Primary database
- **Redis** (optional) - For caching and session storage
- **OpenSSL** - For TLS/mTLS support

### Quick Start

```bash
# 1. Clone repository
git clone https://github.com/analisaperlengkapan/simpel2.git
cd simpel2/infra/authenc

# 2. Configure environment
export DATABASE_URL="postgresql://user:pass@localhost/authenc"
export JWT_SECRET="your-secret-key-change-in-production"

# 3. Build and run
cargo build --release
cargo run

# Server starts on http://localhost:8080
```

### Docker Deployment

```bash
# Build image
docker build -t authenc:latest .

# Run with docker-compose
docker-compose up -d
```

## 🔒 Security Features

### Cryptography

- **Ed25519 Signatures**: Timing-attack resistant JWT signing
- **AES-GCM Encryption**: Enterprise-grade symmetric encryption
- **Hardware Security**: TPM integration for key protection
- **Certificate Validation**: Native mTLS client certificate validation

### Access Control

- **RBAC**: Role-based access control with hierarchical permissions
- **ABAC**: Attribute-based access control for fine-grained policies
- **Resource Protection**: UMA 2.0 resource server implementation
- **Policy Enforcement**: Client policy framework for OAuth2/OIDC

### Threat Protection

- **Brute Force Protection**: Automatic lockout mechanisms
- **Anomaly Detection**: Behavioral analysis and threat detection
- **Rate Limiting**: Distributed rate limiting with Redis
- **Input Validation**: Comprehensive sanitization and validation

### Compliance

- **GDPR Ready**: User consent management and data protection
- **Audit Logging**: Comprehensive audit trails with PostgreSQL persistence
- **Security Monitoring**: Real-time threat detection and alerting
- **Compliance Reporting**: Export capabilities for regulatory requirements

## 🚀 Performance & Scalability

### Architecture Optimizations

- **Async-First Design**: Non-blocking I/O operations throughout
- **Connection Pooling**: Efficient database and Redis connection management
- **Zero-Copy Operations**: Memory-efficient data processing
- **Streaming Support**: Large data handling without memory exhaustion

### Caching Strategy

- **Redis Integration**: Session storage and performance caching
- **Database Query Optimization**: Prepared statements and connection reuse
- **Response Caching**: HTTP-level caching with proper cache headers

### Monitoring & Observability

- **Health Checks**: `/health`, `/ready`, `/live` endpoints
- **Metrics Collection**: Prometheus-compatible metrics
- **Structured Logging**: JSON logging with tracing
- **Distributed Tracing**: OpenTelemetry integration

## 🔧 Development

### Building & Testing

```bash
# Build in release mode
cargo build --release

# Run all tests
cargo test

# Run specific test categories
cargo test auth          # Authentication tests
cargo test security      # Security tests
cargo test performance   # Performance tests
cargo test integration   # Integration tests

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check
```

### Code Quality

- **Zero Unsafe Code**: Completely safe Rust implementation
- **Comprehensive Testing**: >95% test coverage with integration tests
- **Security Audits**: Regular dependency audits and vulnerability scanning
- **Documentation**: Inline documentation with examples

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** changes following [Rust best practices](https://doc.rust-lang.org/book/)
4. **Add tests** for new functionality
5. **Run** the full test suite (`cargo test`)
6. **Update** documentation for API changes
7. **Submit** a Pull Request

### Development Guidelines

- Follow [Semantic Versioning](https://semver.org/) for releases
- Write comprehensive tests for all new features
- Update API documentation for public interfaces
- Maintain backward compatibility when possible
- Follow security best practices and conduct regular audits

## 📄 License

Licensed under the **Apache License 2.0**.

## 🆘 Support

- **📖 Documentation**: Complete API documentation in `docs/` directory
- **🐛 Issues**: [GitHub Issues](https://github.com/analisaperlengkapan/simpel2/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/analisaperlengkapan/simpel2/discussions)
- **📧 Contact**: security@kejaksaan.go.id

---

**Authenc v0.4.0** - Enterprise-grade authentication and authorization for modern applications. 🚀
