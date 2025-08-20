# 🔐 Layanan Keamanan - SIMPelv2

**Layanan Keamanan** adalah microservice utama dalam SIMPelv2 yang bertanggung jawab atas autentikasi, otorisasi, dan keamanan sistem. Dibangun dengan **Rust** untuk performa dan keamanan maksimal.

## 🎯 Fitur Utama

### 🔐 Authentication & Authorization
- **JWT Token Management**: Access token dan refresh token
- **Multi-Factor Authentication (MFA)**: TOTP-based authentication
- **Role-Based Access Control (RBAC)**: Granular permission system
- **Session Management**: Token revocation dan tracking

### 🛡️ Security Features
- **HashiCorp Vault Integration**: Secret management
- **Audit Trail Immutable**: Logging yang tidak dapat diubah
- **Honeytrap Service**: Deteksi intrusi
- **Rate Limiting**: Protection against brute force
- **CORS Protection**: Cross-origin resource sharing

### 📊 Monitoring & Compliance
- **Real-time Audit Logging**: Semua aktivitas tercatat
- **Security Analytics**: Deteksi anomali
- **Compliance Reporting**: ISO 27001, PCI DSS
- **Forensic Capabilities**: Digital forensics support

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   API Gateway   │    │  Security Svc   │
│   (React)       │───▶│   (Envoy)       │───▶│   (Rust)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                                                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Vault         │    │   PostgreSQL    │    │   Audit Logs    │
│   (Secrets)     │◀───│   (Database)    │───▶│   (Immutable)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+
- PostgreSQL 15+
- HashiCorp Vault
- Docker & Docker Compose

### Installation

```bash
# Clone repository
cd layanan/keamanan

# Install dependencies
cargo build --release

# Setup environment
cp .env.example .env
# Edit .env with your configuration

# Run database migrations
psql -d simpelv2 -f db/schema.sql

# Start service
cargo run
```

### Docker Deployment

```bash
# Build image
docker build -t simpelv2-keamanan .

# Run container
docker run -d \
  --name keamanan \
  -p 3001:3001 \
  -e DATABASE_URL=postgres://user:pass@host:5432/db \
  -e JWT_SECRET=your-secret-key \
  -e VAULT_URL=http://vault:8200 \
  simpelv2-keamanan
```

## 📡 API Endpoints

### Authentication
```http
POST /auth/login
POST /auth/logout
POST /auth/refresh
POST /auth/mfa/setup
POST /auth/mfa/verify
```

### User Management
```http
GET  /users/me
GET  /users/{id}
POST /users
PUT  /users/{id}
DELETE /users/{id}
```

### Role Management
```http
GET  /roles
POST /roles
PUT  /roles/{id}
DELETE /roles/{id}
```

### Permission Management
```http
GET  /permissions
POST /permissions
PUT  /permissions/{id}
DELETE /permissions/{id}
```

### Audit & Monitoring
```http
GET  /audit/logs
GET  /audit/analytics
GET  /security/health
```

## 🔧 Configuration

### Environment Variables
```env
# Database
DATABASE_URL=postgres://user:pass@host:5432/simpelv2

# JWT
JWT_SECRET=your-super-secret-jwt-key

# Vault
VAULT_URL=http://localhost:8200
VAULT_TOKEN=your-vault-token

# Server
SERVER_PORT=3001
SERVER_HOST=0.0.0.0

# Security
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_DURATION=60

# CORS
CORS_ORIGINS=http://localhost:3000,http://localhost:8080
```

## 🗄️ Database Schema

### Core Tables
- `users`: User accounts dan credentials
- `roles`: Role definitions
- `permissions`: Permission definitions
- `user_roles`: User-role relationships
- `role_permissions`: Role-permission relationships
- `audit_logs`: Immutable audit trail
- `sessions`: Token management
- `failed_logins`: Brute force protection

### Security Features
- **Immutable Audit Logs**: Hash-based integrity
- **Automatic Timestamps**: Created/updated tracking
- **Foreign Key Constraints**: Data integrity
- **Indexes**: Performance optimization

## 🔐 Security Features

### JWT Authentication
```rust
// Generate JWT token
let token = auth_service.generate_access_token(&user, &roles, &permissions)?;

// Verify JWT token
let claims = auth_service.verify_token(token)?;
```

### MFA Implementation
```rust
// Setup MFA
let mfa_response = auth_service.setup_mfa(user_id)?;

// Verify MFA
let is_valid = auth_service.verify_mfa(secret, code)?;
```

### RBAC Authorization
```rust
// Check permission
let has_access = rbac_service.check_permission(user_id, "read:users")?;

// Validate access
let is_authorized = rbac_service.validate_access(user_id, "users", "read")?;
```

### Vault Integration
```rust
// Get secret from Vault
let secret = vault_client.get_secret("database/credentials").await?;

// Rotate secret
let new_secret = vault_client.rotate_secret("api/keys").await?;
```

## 📊 Monitoring & Observability

### Health Checks
```bash
# Service health
curl http://localhost:3001/health

# Database health
curl http://localhost:3001/health/db

# Vault health
curl http://localhost:3001/health/vault
```

### Metrics
- Authentication success/failure rates
- Token generation/validation metrics
- MFA usage statistics
- Audit log volume
- Security event counts

### Logging
```rust
// Structured logging
info!("User logged in successfully", user_id = user.id);
error!("Failed login attempt", username = username, ip = ip);
```

## 🧪 Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test integration
```

### Security Tests
```bash
# Run security scan
cargo audit

# Run fuzzing tests
cargo fuzz run
```

## 🔄 CI/CD

### GitHub Actions
```yaml
name: Security Service CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: cargo clippy
      - run: cargo audit
```

## 📈 Performance

### Benchmarks
- **JWT Generation**: ~1ms per token
- **Password Verification**: ~10ms per verification
- **MFA Verification**: ~5ms per code
- **Audit Logging**: ~2ms per log entry

### Optimization
- **Connection Pooling**: Database connection reuse
- **Caching**: Redis for session data
- **Async Processing**: Non-blocking operations
- **Compression**: Gzip for API responses

## 🔒 Security Best Practices

### Code Security
- **Memory Safety**: Rust's ownership system
- **No Runtime**: No garbage collection overhead
- **Type Safety**: Compile-time error checking
- **Zero-cost Abstractions**: High performance

### Operational Security
- **Principle of Least Privilege**: Minimal permissions
- **Defense in Depth**: Multiple security layers
- **Secure by Default**: Safe defaults
- **Regular Updates**: Security patches

### Compliance
- **ISO 27001**: Information security management
- **PCI DSS**: Payment card industry standards
- **GDPR**: Data protection regulations
- **SOX**: Financial reporting compliance

## 🤝 Contributing

### Development Setup
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
cargo install sqlx-cli
cargo install cargo-audit

# Setup development database
sqlx database create
sqlx migrate run
```

### Code Style
- Follow Rust conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write comprehensive tests

## 📚 Documentation

### API Documentation
- OpenAPI 3.0 specification
- Interactive Swagger UI
- Postman collection
- Example requests/responses

### Architecture Documentation
- System design documents
- Security architecture
- Deployment guides
- Troubleshooting guides

## 🆘 Support

### Getting Help
- **Documentation**: Comprehensive guides
- **Issues**: GitHub issue tracker
- **Discussions**: Community forum
- **Security**: Responsible disclosure

### Contact
- **Email**: security@simpelv2.go.id
- **Slack**: #simpelv2-security
- **GitHub**: Issues and discussions

---

**🔐 Built with ❤️ and Rust for maximum security and performance** 