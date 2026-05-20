# Authenc Getting Started Guide

> **Comprehensive end-to-end documentation for Authenc - Enterprise Authentication & Authorization Server**
>
> Version: 0.4.0 | Last Updated: November 10, 2025

## Table of Contents

1. [Overview & Architecture](#1-overview--architecture)
2. [Prerequisites & System Requirements](#2-prerequisites--system-requirements)
3. [Database Setup](#3-database-setup)
4. [Initial Configuration](#4-initial-configuration)
5. [First-Time Setup](#5-first-time-setup)
6. [Authentication Flows](#6-authentication-flows)
7. [Multi-Factor Authentication (MFA)](#7-multi-factor-authentication-mfa)
8. [OAuth2 & OpenID Connect Integration](#8-oauth2--openid-connect-integration)
9. [Role-Based Access Control (RBAC)](#9-role-based-access-control-rbac)
10. [API Reference](#10-api-reference)
11. [Microfrontend Integration](#11-microfrontend-integration)
12. [Monitoring & Operations](#12-monitoring--operations)
13. [Troubleshooting](#13-troubleshooting)
14. [Security Best Practices](#14-security-best-practices)
15. [Quick Reference](#15-quick-reference)

---

## 1. Overview & Architecture

### 1.1 What is Authenc?

Authenc is a **production-ready Identity and Access Management (IAM) service** built in Rust, designed specifically for the Indonesian Attorney General's Office (Kejaksaan Agung RI) SIMPEL system. It provides:

- **Enterprise-grade authentication** with OAuth2, OIDC, and SAML 2.0
- **Multi-factor authentication** (TOTP, WebAuthn/FIDO2, SMS, Email)
- **Zero-trust security architecture** with continuous authentication
- **Hierarchical RBAC** for Attorney General's organizational structure
- **Post-quantum cryptography** with Ed25519 and ML-DSA signatures
- **Comprehensive audit logging** with PostgreSQL persistence
- **High availability** with Redis caching and connection pooling

### 1.2 Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        Authenc Architecture                       │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐     │
│  │   Portal    │      │  Badiklat   │      │   Datun     │     │
│  │Microfrontend│      │Microfrontend│      │Microfrontend│     │
│  └──────┬──────┘      └──────┬──────┘      └──────┬──────┘     │
│         │                     │                     │             │
│         │   OAuth2/OIDC       │   OAuth2/OIDC      │             │
│         └─────────────────────┼─────────────────────┘             │
│                               │                                   │
│                               ▼                                   │
│                  ┌────────────────────────┐                       │
│                  │    Authenc Server      │                       │
│                  │  (Rust/Axum - Port     │                       │
│                  │       8088)            │                       │
│                  └────────────────────────┘                       │
│                               │                                   │
│         ┌─────────────────────┼─────────────────────┐             │
│         │                     │                     │             │
│         ▼                     ▼                     ▼             │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐     │
│  │ PostgreSQL  │      │    Redis    │      │  Secreton   │     │
│  │  (Primary)  │      │   (Cache)   │      │   (Vault)   │     │
│  │  Port: 5432 │      │ Port: 6379  │      │ Port: 8200  │     │
│  └─────────────┘      └─────────────┘      └─────────────┘     │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### 1.3 Core Components

| Component          | Purpose                                   | Port |
| ------------------ | ----------------------------------------- | ---- |
| **Authenc Server** | Main authentication service               | 8088 |
| **PostgreSQL**     | User data, roles, permissions, audit logs | 5432 |
| **Redis**          | Session caching, rate limiting            | 6379 |
| **Secreton**       | MFA secrets, encryption keys              | 8200 |

### 1.4 Key Features

#### Authentication Methods

- ✅ **Username/Password** with bcrypt hashing
- ✅ **OAuth2** (Authorization Code, PKCE, Client Credentials)
- ✅ **OpenID Connect** with Ed25519 JWT
- ✅ **SAML 2.0** Service Provider
- ✅ **Social Login** (Google, Microsoft, GitHub)
- ✅ **WebAuthn/FIDO2** passwordless authentication

#### Authorization & Access Control

- ✅ **RBAC** (Role-Based Access Control)
- ✅ **ABAC** (Attribute-Based Access Control)
- ✅ **UMA 2.0** (User-Managed Access)
- ✅ **Hierarchical Permissions** (Satker → Wilayah → Pusat)
- ✅ **Policy-Based Authorization**

#### Security Features

- ✅ **Ed25519 Signatures** (timing-attack resistant)
- ✅ **Post-Quantum Crypto** (ML-DSA ready)
- ✅ **mTLS** support
- ✅ **CAPTCHA** integration
- ✅ **Rate Limiting** with Redis
- ✅ **Brute Force Protection**
- ✅ **Anomaly Detection**

### 1.5 Use Cases

**For Government Employees:**

- Login to Portal with CAPTCHA and MFA
- Access multiple microfrontends with SSO
- Manage personal security settings
- View audit trail of account activity

**For System Administrators:**

- Create and manage user accounts
- Configure roles and permissions
- Set up OAuth2 clients
- Monitor system health and security events
- Generate compliance reports

**For Application Developers:**

- Integrate microfrontends with OAuth2
- Implement authentication flows
- Access user information via OIDC
- Enforce authorization policies

---

## 2. Prerequisites & System Requirements

### 2.1 Operating System

**Supported:**

- ✅ Ubuntu 22.04 LTS or higher
- ✅ Debian 11 or higher
- ✅ RHEL/CentOS 8 or higher
- ✅ macOS 12+ (development only)

**Minimum Requirements:**

- **RAM:** 4 GB (8 GB recommended for production)
- **CPU:** 2 cores (4 cores recommended)
- **Disk:** 20 GB free space
- **Network:** 100 Mbps connection

### 2.2 Required Software

#### 2.2.1 PostgreSQL 13+

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# RHEL/CentOS
sudo dnf install postgresql-server postgresql-contrib
sudo postgresql-setup --initdb
sudo systemctl enable postgresql
sudo systemctl start postgresql

# Verify installation
psql --version
# Expected: psql (PostgreSQL) 13.x or higher
```

#### 2.2.2 Redis 6+ (Optional but Recommended)

```bash
# Ubuntu/Debian
sudo apt install redis-server

# RHEL/CentOS
sudo dnf install redis
sudo systemctl enable redis
sudo systemctl start redis

# Verify installation
redis-cli ping
# Expected: PONG
```

#### 2.2.3 Rust 1.75+ (For Building from Source)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
# Expected: rustc 1.75.0 or higher
```

#### 2.2.4 OpenSSL Development Libraries

```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# RHEL/CentOS
sudo dnf install openssl-devel
```

### 2.3 Network Requirements

**Required Ports:**

| Port | Service | Direction | Description |
|------|---------|-----------|-------------|
| 8088 | Authenc HTTP | Inbound | Main API endpoint |
| 8089 | Authenc HTTPS | Inbound | TLS-secured API (optional) |
| 5432 | PostgreSQL | Internal | Database connection |
| 6379 | Redis | Internal | Cache connection |
| 8200 | Secreton | Internal | MFA secrets vault |

**Firewall Configuration:**

```bash
# Allow Authenc HTTP (public)
sudo ufw allow 8088/tcp

# Allow Authenc HTTPS (public, if using TLS)
sudo ufw allow 8089/tcp

# PostgreSQL and Redis should be internal only
# (no external firewall rules needed)
```

### 2.4 DNS Configuration

For production deployment:

```bash
# Add DNS records
authenc.simpel.kejaksaan.go.id   IN A     <server-ip>
portal.simpel.kejaksaan.go.id    IN A     <portal-ip>
```

### 2.5 TLS Certificates (Production Only)

**Option 1: Let's Encrypt (Recommended)**

```bash
# Install certbot
sudo apt install certbot

# Generate certificate
sudo certbot certonly --standalone \
  -d authenc.simpel.kejaksaan.go.id \
  --email admin@kejaksaan.go.id \
  --agree-tos

# Certificates will be in:
# /etc/letsencrypt/live/authenc.simpel.kejaksaan.go.id/
```

**Option 2: Self-Signed (Development/Testing)**

```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 \
  -keyout authenc-key.pem \
  -out authenc-cert.pem \
  -days 365 -nodes \
  -subj "/C=ID/ST=Jakarta/L=Jakarta/O=Kejaksaan Agung RI/CN=authenc.local"
```

### 2.6 System Tuning

**PostgreSQL Performance:**

Edit `/etc/postgresql/13/main/postgresql.conf`:

```conf
# Memory settings (for 8GB RAM server)
shared_buffers = 2GB
effective_cache_size = 6GB
maintenance_work_mem = 512MB
work_mem = 64MB

# Connection settings
max_connections = 200
```

**Redis Performance:**

Edit `/etc/redis/redis.conf`:

```conf
# Memory limit
maxmemory 1gb
maxmemory-policy allkeys-lru

# Persistence (optional)
save 900 1
save 300 10
save 60 10000
```

**System Limits:**

Edit `/etc/security/limits.conf`:

```conf
*    soft    nofile    65536
*    hard    nofile    65536
```

---

## 3. Database Setup

### 3.1 Create Authenc Database

**Step 1: Connect to PostgreSQL**

```bash
# Switch to postgres user
sudo -u postgres psql
```

**Step 2: Create Database and User**

```sql
-- Create authenc database
CREATE DATABASE authenc
    WITH ENCODING 'UTF8'
    LC_COLLATE = 'en_US.UTF-8'
    LC_CTYPE = 'en_US.UTF-8'
    TEMPLATE template0;

-- Create authenc user with strong password
CREATE USER authenc WITH ENCRYPTED PASSWORD 'CHANGE_THIS_PASSWORD_IN_PRODUCTION';

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE authenc TO authenc;

-- Connect to authenc database
\c authenc

-- Grant schema privileges
GRANT ALL ON SCHEMA public TO authenc;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO authenc;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO authenc;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Verify setup
\l authenc
\du authenc
```

**Expected Output:**

```
                                  List of databases
   Name    |  Owner   | Encoding |   Collate   |    Ctype    |
-----------+----------+----------+-------------+-------------+
 authenc   | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |

                                   List of roles
 Role name | Attributes
-----------+------------
 authenc   |
```

### 3.2 Run Database Migrations

Authenc uses SQL migrations located in `layanan/authenc/migrations/`. These must be run in order.

**Option 1: Manual Migration (Development)**

```bash
# Navigate to authenc directory
cd /srv/proyek/simpelv2/layanan/authenc

# Set database URL
export DATABASE_URL="postgresql://authenc:YOUR_PASSWORD@localhost/authenc"

# Run migrations manually
for migration in migrations/*.sql; do
    echo "Running $migration..."
    psql $DATABASE_URL -f "$migration"
done
```

**Option 2: Using Refinery (Recommended)**

Authenc uses the `refinery` crate for migrations. This is handled automatically on first startup, but you can run manually:

```bash
# Install refinery CLI
cargo install refinery_cli

# Run migrations
cd /srv/proyek/simpelv2/layanan/authenc
refinery migrate -e DATABASE_URL
```

**Expected Output:**

```
Applying migration: 001_initial_schema.sql
Applying migration: 002_realms_and_clients.sql
Applying migration: 003_permissions.sql
...
Applying migration: 031_satker_hierarchy.sql
Migration completed successfully!
```

### 3.3 Verify Database Schema

```sql
-- Connect to authenc database
psql "postgresql://authenc:YOUR_PASSWORD@localhost/authenc"

-- List all tables
\dt

-- Expected tables:
-- users, realms, roles, permissions, user_roles, role_permissions
-- oauth2_clients, oauth2_authorization_codes, sessions
-- audit_logs, mfa_configs, satker_hierarchy
-- and many more...

-- Check users table structure
\d users

-- Sample query to verify empty state
SELECT COUNT(*) FROM users;
-- Expected: 0 (no users yet)
```

### 3.4 Database Backup Configuration

**Setup Automated Backups:**

```bash
# Create backup script
sudo nano /usr/local/bin/authenc-db-backup.sh
```

```bash
#!/bin/bash
# Authenc Database Backup Script

BACKUP_DIR="/var/backups/authenc"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="authenc"
DB_USER="authenc"

mkdir -p "$BACKUP_DIR"

# Backup database
pg_dump -U "$DB_USER" "$DB_NAME" | gzip > "$BACKUP_DIR/authenc_$DATE.sql.gz"

# Keep only last 7 days of backups
find "$BACKUP_DIR" -name "authenc_*.sql.gz" -mtime +7 -delete

echo "Backup completed: authenc_$DATE.sql.gz"
```

```bash
# Make executable
sudo chmod +x /usr/local/bin/authenc-db-backup.sh

# Add to crontab (daily at 2 AM)
sudo crontab -e
# Add line:
0 2 * * * /usr/local/bin/authenc-db-backup.sh >> /var/log/authenc-backup.log 2>&1
```

### 3.5 Create Audit Log Database (Optional Separation)

For enhanced security and performance, you can separate audit logs:

```sql
-- Create separate audit database
CREATE DATABASE authenc_audit
    WITH ENCODING 'UTF8'
    TEMPLATE template0;

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE authenc_audit TO authenc;

-- Connect and create audit table
\c authenc_audit

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    event_type VARCHAR(100) NOT NULL,
    user_id UUID,
    username VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    resource_type VARCHAR(100),
    resource_id VARCHAR(255),
    action VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    details JSONB,
    realm_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_status ON audit_logs(status);

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO authenc;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO authenc;
```

Then configure Authenc to use separate audit database:

```bash
# In .env file
DATABASE_URL=postgresql://authenc:password@localhost/authenc
AUDIT_LOG_URL=postgresql://authenc:password@localhost/authenc_audit
```

---

## 4. Initial Configuration

### 4.1 Generate Required Secrets

Before starting Authenc, you need to generate secure cryptographic keys and secrets.

#### 4.1.1 JWT Secret

```bash
# Generate 256-bit JWT secret
openssl rand -base64 32

# Example output:
# 7jK9mN2pQ4rS6tU8vW0xY2zA4bC6dE8fG0hI2jK4lM6n
```

Save this as `JWT_SECRET` in your configuration.

#### 4.1.2 Ed25519 Signing Key (Recommended)

```bash
# Generate Ed25519 key pair
openssl genpkey -algorithm ed25519 -out authenc-ed25519.pem

# Extract private key in base64 format
openssl pkey -in authenc-ed25519.pem -outform DER | base64 -w 0

# Example output:
# MC4CAQAwBQYDK2VwBCIEIJ5w8... (longer string)
```

Save this as `ED25519_PRIVATE_KEY_BASE64` in your configuration.

⚠️ **CRITICAL FOR PRODUCTION**: You MUST set a persistent signing key to prevent JWT tokens from being invalidated on every restart!

#### 4.1.3 Secreton Admin Token

```bash
# Generate Secreton admin token
openssl rand -hex 32

# Example output:
# a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a7b8c9d0e1f2
```

Save this as `SECRETON_TOKEN`.

### 4.2 Configuration File

Create configuration file at `/etc/authenc/config.toml`:

```bash
# Create directory
sudo mkdir -p /etc/authenc

# Create config file
sudo nano /etc/authenc/config.toml
```

**Complete Configuration Template:**

```toml
# /etc/authenc/config.toml
# Authenc Configuration File - Production Example

[server]
# Server binding address
host = "0.0.0.0"
port = 8088

# Request timeout in seconds
request_timeout = 30

# Enable graceful shutdown
graceful_shutdown_timeout = 30

[database]
# PostgreSQL connection
host = "localhost"
port = 5432
username = "authenc"
password = "YOUR_SECURE_PASSWORD"
database = "authenc"

# Connection pool settings
max_connections = 100
min_connections = 10
connection_timeout = 30

# Connection lifetimes in seconds
idle_timeout = 600      # 10 minutes
max_lifetime = 1800     # 30 minutes

# Optional: Separate audit log database
# audit_log_url = "postgresql://authenc:password@localhost/authenc_audit"

[redis]
# Redis connection (optional but recommended)
enabled = true
url = "redis://localhost:6379"

# Cache TTL settings (in seconds)
session_ttl = 3600              # 1 hour
mfa_cache_ttl = 300             # 5 minutes
otp_verification_ttl = 300      # 5 minutes

# Redis pool settings
max_connections = 50
connection_timeout = 5

[security]
# JWT configuration
jwt_secret = "7jK9mN2pQ4rS6tU8vW0xY2zA4bC6dE8fG0hI2jK4lM6n"
jwt_expiration = 3600           # 1 hour (in seconds)
refresh_token_expiration = 604800  # 7 days

# Issuer and audience for JWT
issuer = "authenc.simpel.kejaksaan.go.id"
audience = "portal,badiklat,datun,intel"

# Ed25519 signing key (base64 encoded)
ed25519_private_key_base64 = "MC4CAQAwBQYDK2VwBCIEIJ5w8..."

# Password hashing
bcrypt_cost = 12

# Rate limiting
enable_rate_limiting = true
rate_limit_requests_per_minute = 100
rate_limit_burst = 20

# CORS settings
cors_allowed_origins = [
    "http://localhost:8080",
    "https://portal.simpel.kejaksaan.go.id",
    "https://badiklat.simpel.kejaksaan.go.id",
    "https://datun.simpel.kejaksaan.go.id"
]

# Session configuration
session_timeout = 3600          # 1 hour
session_absolute_timeout = 28800  # 8 hours

[mfa]
# MFA configuration
issuer = "SIMPEL Kejaksaan RI"
totp_period = 30
totp_digits = 6
backup_codes_count = 10

# MFA enforcement policy
enforce_mfa_for_admins = true
enforce_mfa_for_all_users = false

[oauth2]
# OAuth2 configuration
authorization_code_ttl = 600    # 10 minutes
access_token_ttl = 3600         # 1 hour
refresh_token_ttl = 2592000     # 30 days

# PKCE enforcement
require_pkce = true

[oidc]
# OpenID Connect configuration
enabled = true
discovery_enabled = true

# Token signing algorithm (EdDSA for Ed25519)
signing_algorithm = "EdDSA"

[saml]
# SAML 2.0 configuration
enabled = false
entity_id = "authenc.simpel.kejaksaan.go.id"
# sso_url = "https://authenc.simpel.kejaksaan.go.id/saml/sso"
# certificate = "/etc/authenc/saml-cert.pem"

[secreton]
# Secreton vault integration
endpoint = "http://localhost:8200"
token = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"

# MFA secret storage
mfa_mount_path = "secret/mfa"

[features]
# Feature flags
enable_registration = false      # Self-registration disabled
enable_password_reset = true
enable_account_recovery = true
enable_social_login = false      # Social login disabled by default
enable_webauthn = true          # WebAuthn/FIDO2 enabled

[observability]
# Logging configuration
log_level = "info"              # trace, debug, info, warn, error
log_format = "json"             # json or text
log_file = "/var/log/authenc/authenc.log"

# Metrics
enable_metrics = true
prometheus_port = 9090

# Distributed tracing
enable_tracing = false
# jaeger_endpoint = "http://localhost:14268/api/traces"

[audit]
# Audit logging
enabled = true
log_all_events = true
retention_days = 365

# Sensitive data masking
mask_passwords = true
mask_tokens = true

[tls]
# TLS configuration (optional)
enabled = false
# cert_file = "/etc/letsencrypt/live/authenc.simpel.kejaksaan.go.id/fullchain.pem"
# key_file = "/etc/letsencrypt/live/authenc.simpel.kejaksaan.go.id/privkey.pem"

# mTLS (mutual TLS)
mtls_enabled = false
# client_ca_file = "/etc/authenc/client-ca.pem"

[events]
# Event system configuration
user_event_retention_days = 90
admin_event_retention_days = 365
```

### 4.3 Environment Variables

Alternatively, you can use environment variables (takes precedence over config file):

```bash
# Create environment file
sudo nano /etc/authenc/authenc.env
```

```bash
# /etc/authenc/authenc.env
# Authenc Environment Variables

# Server Configuration
AUTHENC_HOST=0.0.0.0
AUTHENC_PORT=8088

# Database
DATABASE_URL=postgresql://authenc:YOUR_PASSWORD@localhost/authenc

# Security
JWT_SECRET=7jK9mN2pQ4rS6tU8vW0xY2zA4bC6dE8fG0hI2jK4lM6n
ED25519_PRIVATE_KEY_BASE64=MC4CAQAwBQYDK2VwBCIEIJ5w8...

# Redis
REDIS_URL=redis://localhost:6379

# Secreton
SECRETON_ENDPOINT=http://localhost:8200
SECRETON_TOKEN=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# CORS
CORS_ALLOWED_ORIGINS=http://localhost:8080,https://portal.simpel.kejaksaan.go.id
```

### 4.4 Build Authenc

#### 4.4.1 From Source (Development)

```bash
# Clone repository (if not already)
cd /srv/proyek/simpelv2

# Build authenc
cd layanan/authenc
cargo build --release

# Binary will be at:
# target/release/authenc
```

#### 4.4.2 Using Docker (Production)

```bash
# Build Docker image
cd /srv/proyek/simpelv2/layanan/authenc
docker build -t authenc:latest .

# Or use docker-compose
cd /srv/proyek/simpelv2
docker-compose -f docker-compose.yml -f docker-compose.prod.yml build authenc
```

### 4.5 Create Systemd Service

For production deployment with systemd:

```bash
# Create service file
sudo nano /etc/systemd/system/authenc.service
```

```ini
[Unit]
Description=Authenc - Enterprise Authentication & Authorization Server
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=authenc
Group=authenc
WorkingDirectory=/opt/authenc

# Environment
EnvironmentFile=/etc/authenc/authenc.env

# Main process
ExecStart=/opt/authenc/authenc
ExecReload=/bin/kill -HUP $MAINPID

# Restart policy
Restart=on-failure
RestartSec=10s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/authenc

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=authenc

[Install]
WantedBy=multi-user.target
```

**Setup service:**

```bash
# Create authenc user
sudo useradd -r -s /bin/false authenc

# Create directories
sudo mkdir -p /opt/authenc
sudo mkdir -p /var/log/authenc
sudo mkdir -p /etc/authenc

# Copy binary
sudo cp target/release/authenc /opt/authenc/
sudo chown -R authenc:authenc /opt/authenc
sudo chown -R authenc:authenc /var/log/authenc

# Set permissions
sudo chmod 755 /opt/authenc/authenc
sudo chmod 600 /etc/authenc/config.toml
sudo chmod 600 /etc/authenc/authenc.env

# Reload systemd
sudo systemctl daemon-reload

# Enable service
sudo systemctl enable authenc

# Start service
sudo systemctl start authenc

# Check status
sudo systemctl status authenc
```

### 4.6 Verify Installation

**Check if Authenc is running:**

```bash
# Check service status
sudo systemctl status authenc

# Check if port is listening
sudo netstat -tlnp | grep 8088

# Check logs
sudo journalctl -u authenc -f
```

**Test health endpoint:**

```bash
# Health check
curl -f http://localhost:8088/health

# Expected response:
{
  "status": "healthy",
  "version": "0.4.0",
  "timestamp": "2025-11-10T10:30:00Z"
}

# Readiness check
curl -f http://localhost:8088/ready

# Liveness check
curl -f http://localhost:8088/live
```

**Test OIDC discovery:**

```bash
# OIDC discovery endpoint
curl -s http://localhost:8088/.well-known/openid-configuration | jq

# Expected response includes:
{
  "issuer": "http://localhost:8088",
  "authorization_endpoint": "http://localhost:8088/oauth2/authorize",
  "token_endpoint": "http://localhost:8088/oauth2/token",
  "jwks_uri": "http://localhost:8088/oauth2/jwks",
  ...
}
```

---

## 5. First-Time Setup

### 5.1 Create Default Realm

Realms in Authenc provide multi-tenancy. The default realm for SIMPEL is `simpel`.

**Using API:**

```bash
# Create simpel realm
curl -X POST http://localhost:8088/api/v1/realms \
  -H "Content-Type: application/json" \
  -d '{
    "name": "simpel",
    "display_name": "SIMPEL Kejaksaan RI",
    "enabled": true,
    "registration_allowed": false,
    "registration_email_as_username": false,
    "login_with_email_allowed": true,
    "duplicate_emails_allowed": false,
    "ssl_required": "external",
    "password_policy": {
      "minimum_length": 12,
      "require_uppercase": true,
      "require_lowercase": true,
      "require_digits": true,
      "require_special_chars": true,
      "expiration_days": 90
    }
  }'
```

**Expected Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "simpel",
  "display_name": "SIMPEL Kejaksaan RI",
  "enabled": true,
  "created_at": "2025-11-10T10:30:00Z"
}
```

### 5.2 Create First Admin User

#### 5.2.1 Direct Database Insert (Quick Start)

```sql
-- Connect to authenc database
psql "postgresql://authenc:YOUR_PASSWORD@localhost/authenc"

-- Insert admin user
INSERT INTO users (
    id,
    realm_id,
    username,
    email,
    email_verified,
    enabled,
    created_at,
    updated_at,
    nip,
    nama_lengkap,
    admin_level
) VALUES (
    gen_random_uuid(),
    '550e8400-e29b-41d4-a716-446655440000',  -- simpel realm ID
    'admin',
    'admin@kejaksaan.go.id',
    true,
    true,
    NOW(),
    NOW(),
    '199901012025011001',  -- Example NIP
    'Administrator Sistem',
    'AdminPusat'
);

-- Set password (bcrypt hash for "Admin@123456")
UPDATE users
SET password_hash = '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5RA0WzL8z.QJ6'
WHERE username = 'admin';

-- Verify user created
SELECT id, username, email, admin_level FROM users WHERE username = 'admin';
```

#### 5.2.2 Using API (Recommended)

```bash
# Create admin user via API
curl -X POST http://localhost:8088/api/v1/auth/users \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "email": "admin@kejaksaan.go.id",
    "password": "Admin@123456",
    "nip": "199901012025011001",
    "nama_lengkap": "Administrator Sistem",
    "admin_level": "AdminPusat",
    "satker_code": "PUSAT",
    "enabled": true,
    "email_verified": true
  }'
```

⚠️ **IMPORTANT**: Change the default password immediately after first login!

### 5.3 Create Default Roles

```bash
# Create Admin Pusat role
curl -X POST http://localhost:8088/api/v1/realms/simpel/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "AdminPusat",
    "description": "Administrator Tingkat Pusat - Full System Access",
    "composite": false,
    "client_role": false
  }'

# Create Admin Wilayah role
curl -X POST http://localhost:8088/api/v1/realms/simpel/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "AdminWilayah",
    "description": "Administrator Tingkat Wilayah",
    "composite": false,
    "client_role": false
  }'

# Create Admin Satker role
curl -X POST http://localhost:8088/api/v1/realms/simpel/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "AdminSatker",
    "description": "Administrator Tingkat Satker",
    "composite": false,
    "client_role": false
  }'

# Create Jaksa role
curl -X POST http://localhost:8088/api/v1/realms/simpel/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Jaksa",
    "description": "Jaksa Penuntut Umum",
    "composite": false,
    "client_role": false
  }'

# Create Staff role
curl -X POST http://localhost:8088/api/v1/realms/simpel/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Staff",
    "description": "Staff Administrasi",
    "composite": false,
    "client_role": false
  }'
```

### 5.4 Create Default Permissions

```bash
# Create system permissions
PERMISSIONS=(
  "system:admin"
  "users:read"
  "users:write"
  "users:delete"
  "roles:read"
  "roles:write"
  "audit:read"
  "secrets:read"
  "secrets:write"
  "cases:read"
  "cases:write"
  "evidence:read"
  "evidence:write"
)

for perm in "${PERMISSIONS[@]}"; do
  curl -X POST http://localhost:8088/api/v1/realms/simpel/permissions \
    -H "Content-Type: application/json" \
    -d "{
      \"name\": \"$perm\",
      \"description\": \"Permission for $perm\",
      \"resource_type\": \"${perm%%:*}\",
      \"action\": \"${perm##*:}\"
    }"
done
```

### 5.5 Assign Permissions to Roles

```bash
# Assign all permissions to AdminPusat
for perm in system:admin users:read users:write users:delete roles:read roles:write audit:read secrets:read secrets:write cases:read cases:write evidence:read evidence:write; do
  curl -X POST "http://localhost:8088/api/v1/realms/simpel/roles/AdminPusat/permissions/$perm"
done

# Assign limited permissions to Jaksa
for perm in cases:read cases:write evidence:read evidence:write; do
  curl -X POST "http://localhost:8088/api/v1/realms/simpel/roles/Jaksa/permissions/$perm"
done
```

### 5.6 Create OAuth2 Clients for Microfrontends

```bash
# Portal client
curl -X POST http://localhost:8088/api/v1/realms/simpel/clients \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "portal",
    "client_name": "Portal Microfrontend",
    "enabled": true,
    "public_client": true,
    "redirect_uris": [
      "http://localhost:8080/callback",
      "https://portal.simpel.kejaksaan.go.id/callback"
    ],
    "web_origins": [
      "http://localhost:8080",
      "https://portal.simpel.kejaksaan.go.id"
    ],
    "protocol": "openid-connect",
    "standard_flow_enabled": true,
    "implicit_flow_enabled": false,
    "direct_access_grants_enabled": false,
    "service_accounts_enabled": false
  }'

# Badiklat client
curl -X POST http://localhost:8088/api/v1/realms/simpel/clients \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "badiklat",
    "client_name": "Badiklat Microfrontend",
    "enabled": true,
    "public_client": true,
    "redirect_uris": [
      "http://localhost:8081/callback",
      "https://badiklat.simpel.kejaksaan.go.id/callback"
    ],
    "web_origins": [
      "http://localhost:8081",
      "https://badiklat.simpel.kejaksaan.go.id"
    ],
    "protocol": "openid-connect",
    "standard_flow_enabled": true
  }'

# Repeat for other microfrontends: datun, intel, pembinaan,
# pemulihan_aset, pengawasan, pidmil, pidsus, pidum
```

### 5.7 Test Admin Login

```bash
# Login as admin
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "Admin@123456",
    "realm": "simpel"
  }'
```

**Expected Response:**

```json
{
  "access_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "session_state": "550e8400-e29b-41d4-a716-446655440000"
}
```

🎉 **Congratulations!** Authenc is now set up and ready for use.

---

## 6. Authentication Flows

### 6.1 Username/Password Authentication

Basic authentication flow with username and password.

**Endpoint:** `POST /api/v1/auth/login`

**Request:**

```bash
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user@kejaksaan.go.id",
    "password": "SecurePassword123!",
    "realm": "simpel"
  }'
```

**Response (Success):**

```json
{
  "access_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiLCJpYXQiOjE2OTk2NTAwMDAsImV4cCI6MTY5OTY1MzYwMCwiaXNzIjoiYXV0aGVuYy5zaW1wZWwua2VqYWtzYWFuLmdvLmlkIiwiYXVkIjpbInBvcnRhbCIsImJhZGlrbGF0Il0sInJlYWxtIjoic2ltcGVsIiwicm9sZXMiOlsiSmFrc2EiXSwicGVybWlzc2lvbnMiOlsiY2FzZXM6cmVhZCIsImNhc2VzOndyaXRlIl19.signature",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "session_state": "abc123-session-id",
  "scope": "openid profile email"
}
```

**Response (MFA Required):**

```json
{
  "status": "mfa_required",
  "mfa_token": "temp_mfa_token_abc123",
  "mfa_methods": ["totp", "email"],
  "message": "Multi-factor authentication required"
}
```

### 6.2 OAuth2 Authorization Code Flow (PKCE)

Complete OAuth2 flow for microfrontend authentication.

#### Step 1: Generate PKCE Challenge

```javascript
// In your frontend JavaScript
function generateCodeVerifier() {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64UrlEncode(array);
}

function generateCodeChallenge(verifier) {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  return crypto.subtle.digest("SHA-256", data).then((hash) => {
    return base64UrlEncode(new Uint8Array(hash));
  });
}

const codeVerifier = generateCodeVerifier();
const codeChallenge = await generateCodeChallenge(codeVerifier);

// Store codeVerifier in sessionStorage
sessionStorage.setItem("pkce_code_verifier", codeVerifier);
```

#### Step 2: Authorization Request

```bash
# User clicks "Login" button, redirect to:
GET http://localhost:8088/oauth2/authorize?
  response_type=code&
  client_id=badiklat&
  redirect_uri=http://localhost:8081/callback&
  state=random_state_abc123&
  code_challenge=BASE64URL(SHA256(code_verifier))&
  code_challenge_method=S256&
  scope=openid profile email
```

**Parameters:**

- `response_type`: Always `code` for Authorization Code flow
- `client_id`: Your registered client ID
- `redirect_uri`: Must match registered redirect URI
- `state`: Random CSRF protection token (store in sessionStorage)
- `code_challenge`: SHA256 hash of code_verifier (base64url encoded)
- `code_challenge_method`: `S256` for SHA256, `plain` for no hashing
- `scope`: Space-separated scopes (openid profile email)

#### Step 3: User Authentication

User is redirected to Portal login page. After successful login (including CAPTCHA and MFA if required), Portal redirects back:

```
http://localhost:8081/callback?
  code=authorization_code_abc123&
  state=random_state_abc123
```

#### Step 4: Token Exchange

```bash
# In your callback handler, exchange code for tokens
curl -X POST http://localhost:8088/oauth2/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=authorization_code_abc123" \
  -d "client_id=badiklat" \
  -d "redirect_uri=http://localhost:8081/callback" \
  -d "code_verifier=STORED_CODE_VERIFIER"
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "id_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "scope": "openid profile email"
}
```

### 6.3 Token Refresh

When access token expires, use refresh token to get new tokens:

```bash
curl -X POST http://localhost:8088/oauth2/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=refresh_token" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  -d "client_id=badiklat"
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "scope": "openid profile email"
}
```

### 6.4 Token Introspection

Validate and inspect token details:

```bash
curl -X POST http://localhost:8088/oauth2/introspect \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "token=TOKEN_TO_INTROSPECT"
```

**Response:**

```json
{
  "active": true,
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "username": "user@kejaksaan.go.id",
  "client_id": "badiklat",
  "token_type": "Bearer",
  "exp": 1699653600,
  "iat": 1699650000,
  "nbf": 1699650000,
  "scope": "openid profile email",
  "realm": "simpel",
  "roles": ["Jaksa"],
  "permissions": ["cases:read", "cases:write"]
}
```

### 6.5 Token Revocation

Revoke access or refresh token:

```bash
curl -X POST http://localhost:8088/oauth2/revoke \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "token=TOKEN_TO_REVOKE" \
  -d "token_type_hint=refresh_token"
```

**Response:** `204 No Content` (success)

### 6.6 Logout

```bash
# Logout (invalidate session)
curl -X POST http://localhost:8088/api/v1/auth/logout \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

**Response:**

```json
{
  "message": "Logged out successfully"
}
```

---

## 7. Multi-Factor Authentication (MFA)

### 7.1 MFA Setup Flow

#### Step 1: Initiate MFA Setup

After initial login, user should setup MFA:

```bash
# Setup TOTP (Time-based One-Time Password)
curl -X POST http://localhost:8088/api/v1/auth/mfa/setup \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "method": "totp"
  }'
```

**Response:**

```json
{
  "qr_code_url": "otpauth://totp/SIMPEL%20Kejaksaan%20RI:user@kejaksaan.go.id?secret=JBSWY3DPEHPK3PXP2AB4CDEFGHIJKLMN&issuer=SIMPEL%20Kejaksaan%20RI&algorithm=SHA1&digits=6&period=30",
  "secret_key": "JBSWY3DPEHPK3PXP2AB4CDEFGHIJKLMN",
  "backup_codes": [
    "12345678",
    "87654321",
    "11223344",
    "44332211",
    "55667788",
    "88776655",
    "99887766",
    "66778899",
    "33445566",
    "66554433"
  ]
}
```

#### Step 2: User Scans QR Code

User opens authenticator app (Google Authenticator, Microsoft Authenticator, FreeOTP) and scans QR code.

**Recommended Authenticator Apps:**

- **Google Authenticator** (iOS/Android)
- **Microsoft Authenticator** (iOS/Android)
- **FreeOTP** (iOS/Android) - Open source
- **Authy** (iOS/Android/Desktop)

#### Step 3: Verify MFA Setup

User enters first OTP code from authenticator app:

```bash
curl -X POST http://localhost:8088/api/v1/auth/mfa/verify-setup \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "123456"
  }'
```

**Response (Success):**

```json
{
  "message": "MFA setup completed successfully",
  "mfa_enabled": true
}
```

**Response (Invalid Code):**

```json
{
  "error": "invalid_code",
  "message": "The verification code is invalid or has expired"
}
```

### 7.2 MFA Login Flow

When MFA is enabled, login requires additional verification step.

#### Step 1: Initial Login

```bash
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user@kejaksaan.go.id",
    "password": "SecurePassword123!",
    "realm": "simpel"
  }'
```

**Response (MFA Required):**

```json
{
  "status": "mfa_required",
  "mfa_token": "temp_mfa_token_abc123xyz",
  "mfa_methods": ["totp"],
  "message": "Please provide MFA verification code"
}
```

#### Step 2: Verify MFA Code

```bash
curl -X POST http://localhost:8088/api/v1/auth/mfa/verify \
  -H "Content-Type: application/json" \
  -d '{
    "mfa_token": "temp_mfa_token_abc123xyz",
    "code": "123456"
  }'
```

**Response (Success):**

```json
{
  "access_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "session_state": "abc123-session-id"
}
```

### 7.3 MFA Recovery with Backup Codes

If user loses access to authenticator app, use backup codes:

```bash
curl -X POST http://localhost:8088/api/v1/auth/mfa/verify \
  -H "Content-Type: application/json" \
  -d '{
    "mfa_token": "temp_mfa_token_abc123xyz",
    "backup_code": "12345678"
  }'
```

**Note:** Each backup code can only be used once.

### 7.4 Disable MFA

```bash
curl -X POST http://localhost:8088/api/v1/auth/mfa/disable \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "password": "SecurePassword123!",
    "code": "123456"
  }'
```

**Response:**

```json
{
  "message": "MFA disabled successfully",
  "mfa_enabled": false
}
```

### 7.5 MFA Status Check

```bash
curl -X GET http://localhost:8088/api/v1/auth/mfa/status \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

**Response:**

```json
{
  "mfa_enabled": true,
  "methods": ["totp"],
  "backup_codes_remaining": 8,
  "configured_at": "2025-11-10T10:30:00Z"
}
```

### 7.6 WebAuthn/FIDO2 Setup (Advanced)

#### Register Security Key

```bash
# Step 1: Request registration challenge
curl -X POST http://localhost:8088/api/v1/auth/webauthn/register/begin \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "YubiKey 5C NFC"
  }'
```

**Response:**

```json
{
  "challenge": "BASE64_CHALLENGE_STRING",
  "rp": {
    "name": "SIMPEL Kejaksaan RI",
    "id": "simpel.kejaksaan.go.id"
  },
  "user": {
    "id": "BASE64_USER_ID",
    "name": "user@kejaksaan.go.id",
    "displayName": "User Name"
  },
  "pubKeyCredParams": [
    { "type": "public-key", "alg": -7 },
    { "type": "public-key", "alg": -257 }
  ],
  "timeout": 60000,
  "authenticatorSelection": {
    "authenticatorAttachment": "cross-platform",
    "requireResidentKey": false,
    "userVerification": "preferred"
  }
}
```

#### Complete Registration

```bash
# Step 2: Send credential from authenticator
curl -X POST http://localhost:8088/api/v1/auth/webauthn/register/complete \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "credential": {
      "id": "CREDENTIAL_ID",
      "rawId": "BASE64_RAW_ID",
      "response": {
        "clientDataJSON": "BASE64_CLIENT_DATA",
        "attestationObject": "BASE64_ATTESTATION"
      },
      "type": "public-key"
    }
  }'
```

**Response:**

```json
{
  "message": "WebAuthn credential registered successfully",
  "credential_id": "CREDENTIAL_ID"
}
```

---

## 8. OAuth2 & OpenID Connect Integration

### 8.1 OIDC Discovery

OpenID Connect provides automatic configuration discovery:

```bash
curl -s http://localhost:8088/.well-known/openid-configuration | jq
```

**Response:**

```json
{
  "issuer": "http://localhost:8088",
  "authorization_endpoint": "http://localhost:8088/oauth2/authorize",
  "token_endpoint": "http://localhost:8088/oauth2/token",
  "userinfo_endpoint": "http://localhost:8088/oauth2/userinfo",
  "jwks_uri": "http://localhost:8088/oauth2/jwks",
  "registration_endpoint": "http://localhost:8088/oauth2/register",
  "introspection_endpoint": "http://localhost:8088/oauth2/introspect",
  "revocation_endpoint": "http://localhost:8088/oauth2/revoke",
  "end_session_endpoint": "http://localhost:8088/oidc/logout",
  "scopes_supported": ["openid", "profile", "email", "address", "phone"],
  "response_types_supported": [
    "code",
    "token",
    "id_token",
    "code token",
    "code id_token",
    "token id_token",
    "code token id_token"
  ],
  "grant_types_supported": [
    "authorization_code",
    "refresh_token",
    "client_credentials"
  ],
  "subject_types_supported": ["public"],
  "id_token_signing_alg_values_supported": ["EdDSA", "ES256", "RS256"],
  "token_endpoint_auth_methods_supported": [
    "client_secret_basic",
    "client_secret_post",
    "none"
  ],
  "code_challenge_methods_supported": ["S256", "plain"],
  "claims_supported": [
    "sub",
    "iss",
    "auth_time",
    "name",
    "given_name",
    "family_name",
    "preferred_username",
    "email",
    "email_verified"
  ]
}
```

### 8.2 Get User Info

Retrieve authenticated user information:

```bash
curl -X GET http://localhost:8088/oauth2/userinfo \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

**Response:**

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "preferred_username": "user@kejaksaan.go.id",
  "name": "User Full Name",
  "given_name": "User",
  "family_name": "Name",
  "email": "user@kejaksaan.go.id",
  "email_verified": true,
  "nip": "199901012025011001",
  "satker_code": "KJA001",
  "admin_level": "AdminSatker",
  "realm": "simpel"
}
```

### 8.3 JWKS (JSON Web Key Set)

Public keys for JWT signature verification:

```bash
curl -s http://localhost:8088/oauth2/jwks | jq
```

**Response:**

```json
{
  "keys": [
    {
      "kty": "OKP",
      "use": "sig",
      "crv": "Ed25519",
      "kid": "authenc-ed25519-2025-11-10",
      "x": "BASE64_PUBLIC_KEY",
      "alg": "EdDSA"
    }
  ]
}
```

### 8.4 Client Registration (Dynamic)

Register new OAuth2 client dynamically:

```bash
curl -X POST http://localhost:8088/oauth2/register \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer INITIAL_ACCESS_TOKEN" \
  -d '{
    "client_name": "My Application",
    "redirect_uris": [
      "https://myapp.example.com/callback"
    ],
    "grant_types": ["authorization_code", "refresh_token"],
    "response_types": ["code"],
    "token_endpoint_auth_method": "none",
    "application_type": "web",
    "contacts": ["admin@example.com"]
  }'
```

**Response:**

```json
{
  "client_id": "generated-client-id",
  "client_name": "My Application",
  "client_id_issued_at": 1699650000,
  "redirect_uris": ["https://myapp.example.com/callback"],
  "grant_types": ["authorization_code", "refresh_token"],
  "response_types": ["code"],
  "token_endpoint_auth_method": "none"
}
```

---

## 9. Role-Based Access Control (RBAC)

### 9.1 Hierarchical Role Structure

Authenc implements a hierarchical RBAC system for Attorney General's Office:

```
┌─────────────────────────────────────────────────┐
│           Attorney General RBAC Hierarchy        │
├─────────────────────────────────────────────────┤
│                                                  │
│  AdminPusat (Central Administrator)             │
│  ├── Can manage all wilayah and satker         │
│  ├── Full system access                         │
│  └── Can create AdminWilayah                    │
│                                                  │
│  AdminWilayah (Regional Administrator)          │
│  ├── Can manage satker in their region         │
│  ├── Regional-level access                      │
│  └── Can create AdminSatker                     │
│                                                  │
│  AdminSatker (Unit Administrator)               │
│  ├── Can manage users in their unit            │
│  ├── Unit-level access                          │
│  └── Can create Jaksa and Staff roles           │
│                                                  │
│  Jaksa (Prosecutor)                             │
│  ├── Case management                            │
│  ├── Evidence handling                          │
│  └── Read/write access to case data             │
│                                                  │
│  Staff (Administrative Staff)                   │
│  ├── Basic document access                      │
│  └── Limited read-only permissions              │
│                                                  │
└─────────────────────────────────────────────────┘
```

### 9.2 Create Custom Role

```bash
curl -X POST http://localhost:8088/api/v1/realms/simpel/roles \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "JaksaPenyidik",
    "description": "Jaksa yang menangani penyidikan kasus",
    "composite": false,
    "client_role": false,
    "attributes": {
      "department": "pidsus",
      "level": "senior"
    }
  }'
```

### 9.3 List All Roles

```bash
curl -X GET "http://localhost:8088/api/v1/realms/simpel/roles" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Response:**

```json
{
  "roles": [
    {
      "id": "role-id-1",
      "name": "AdminPusat",
      "description": "Administrator Tingkat Pusat",
      "composite": false,
      "client_role": false,
      "created_at": "2025-11-10T10:00:00Z"
    },
    {
      "id": "role-id-2",
      "name": "Jaksa",
      "description": "Jaksa Penuntut Umum",
      "composite": false,
      "client_role": false,
      "created_at": "2025-11-10T10:01:00Z"
    }
  ],
  "total": 2
}
```

### 9.4 Assign Role to User

```bash
curl -X POST "http://localhost:8088/api/v1/realms/simpel/users/{user_id}/roles" \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "Jaksa"
  }'
```

### 9.5 Remove Role from User

```bash
curl -X DELETE "http://localhost:8088/api/v1/realms/simpel/users/{user_id}/roles/Jaksa" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

### 9.6 Get User Roles

```bash
curl -X GET "http://localhost:8088/api/v1/realms/simpel/users/{user_id}/roles" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Response:**

```json
{
  "roles": [
    {
      "name": "Jaksa",
      "description": "Jaksa Penuntut Umum",
      "assigned_at": "2025-11-10T11:00:00Z"
    },
    {
      "name": "Staff",
      "description": "Staff Administrasi",
      "assigned_at": "2025-11-10T11:00:00Z"
    }
  ]
}
```

### 9.7 Create Permission

```bash
curl -X POST http://localhost:8088/api/v1/realms/simpel/permissions \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "cases:delete",
    "description": "Permission to delete case files",
    "resource_type": "cases",
    "action": "delete"
  }'
```

### 9.8 Assign Permission to Role

```bash
curl -X POST "http://localhost:8088/api/v1/realms/simpel/roles/Jaksa/permissions/cases:delete" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

### 9.9 Check User Permission

```bash
curl -X POST http://localhost:8088/api/v1/auth/permissions/check \
  -H "Authorization: Bearer USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "resource": "cases",
    "action": "delete"
  }'
```

**Response:**

```json
{
  "allowed": true,
  "reason": "User has permission via role: Jaksa"
}
```

### 9.10 Get User Permissions

```bash
curl -X GET "http://localhost:8088/api/v1/realms/simpel/users/{user_id}/permissions" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Response:**

```json
{
  "permissions": [
    "cases:read",
    "cases:write",
    "cases:delete",
    "evidence:read",
    "evidence:write"
  ],
  "inherited_from_roles": [
    {
      "role": "Jaksa",
      "permissions": [
        "cases:read",
        "cases:write",
        "cases:delete",
        "evidence:read",
        "evidence:write"
      ]
    }
  ]
}
```

---

## 10. API Reference

### 10.1 Authentication Endpoints

| Method | Endpoint                        | Description                   | Auth Required      |
| ------ | ------------------------------- | ----------------------------- | ------------------ |
| POST   | `/api/v1/auth/login`            | Login with username/password  | No                 |
| POST   | `/api/v1/auth/logout`           | Logout and invalidate session | Yes                |
| POST   | `/api/v1/auth/refresh`          | Refresh access token          | No (refresh token) |
| POST   | `/api/v1/auth/mfa/setup`        | Setup MFA for user            | Yes                |
| POST   | `/api/v1/auth/mfa/verify-setup` | Verify MFA setup              | Yes                |
| POST   | `/api/v1/auth/mfa/verify`       | Verify MFA code during login  | No (MFA token)     |
| POST   | `/api/v1/auth/mfa/disable`      | Disable MFA                   | Yes                |
| GET    | `/api/v1/auth/mfa/status`       | Get MFA status                | Yes                |

### 10.2 User Management Endpoints

| Method | Endpoint                                 | Description          | Auth Required |
| ------ | ---------------------------------------- | -------------------- | ------------- |
| GET    | `/api/v1/auth/users`                     | List all users       | Yes (admin)   |
| POST   | `/api/v1/auth/users`                     | Create new user      | Yes (admin)   |
| GET    | `/api/v1/auth/users/{id}`                | Get user details     | Yes           |
| PUT    | `/api/v1/auth/users/{id}`                | Update user          | Yes           |
| DELETE | `/api/v1/auth/users/{id}`                | Delete user          | Yes (admin)   |
| POST   | `/api/v1/auth/users/{id}/password`       | Change user password | Yes           |
| POST   | `/api/v1/auth/users/{id}/reset-password` | Reset password       | Yes (admin)   |

### 10.3 Role Management Endpoints

| Method | Endpoint                                         | Description           | Auth Required |
| ------ | ------------------------------------------------ | --------------------- | ------------- |
| GET    | `/api/v1/realms/{realm}/roles`                   | List roles            | Yes (admin)   |
| POST   | `/api/v1/realms/{realm}/roles`                   | Create role           | Yes (admin)   |
| GET    | `/api/v1/realms/{realm}/roles/{name}`            | Get role details      | Yes (admin)   |
| PUT    | `/api/v1/realms/{realm}/roles/{name}`            | Update role           | Yes (admin)   |
| DELETE | `/api/v1/realms/{realm}/roles/{name}`            | Delete role           | Yes (admin)   |
| POST   | `/api/v1/realms/{realm}/users/{id}/roles`        | Assign role to user   | Yes (admin)   |
| DELETE | `/api/v1/realms/{realm}/users/{id}/roles/{role}` | Remove role from user | Yes (admin)   |
| GET    | `/api/v1/realms/{realm}/users/{id}/roles`        | Get user roles        | Yes           |

### 10.4 Permission Management Endpoints

| Method | Endpoint                                                 | Description               | Auth Required |
| ------ | -------------------------------------------------------- | ------------------------- | ------------- |
| GET    | `/api/v1/realms/{realm}/permissions`                     | List permissions          | Yes (admin)   |
| POST   | `/api/v1/realms/{realm}/permissions`                     | Create permission         | Yes (admin)   |
| DELETE | `/api/v1/realms/{realm}/permissions/{name}`              | Delete permission         | Yes (admin)   |
| POST   | `/api/v1/realms/{realm}/roles/{role}/permissions/{perm}` | Assign permission to role | Yes (admin)   |
| GET    | `/api/v1/realms/{realm}/users/{id}/permissions`          | Get user permissions      | Yes           |
| POST   | `/api/v1/auth/permissions/check`                         | Check permission          | Yes           |

### 10.5 OAuth2/OIDC Endpoints

| Method | Endpoint                            | Description            | Auth Required        |
| ------ | ----------------------------------- | ---------------------- | -------------------- |
| GET    | `/.well-known/openid-configuration` | OIDC discovery         | No                   |
| GET    | `/oauth2/authorize`                 | Authorization endpoint | No (user login)      |
| POST   | `/oauth2/token`                     | Token endpoint         | No (client auth)     |
| GET    | `/oauth2/userinfo`                  | User info endpoint     | Yes                  |
| GET    | `/oauth2/jwks`                      | JWK Set endpoint       | No                   |
| POST   | `/oauth2/introspect`                | Token introspection    | Yes                  |
| POST   | `/oauth2/revoke`                    | Token revocation       | Yes                  |
| POST   | `/oauth2/register`                  | Client registration    | Yes (initial access) |

### 10.6 Client Management Endpoints

| Method | Endpoint                              | Description         | Auth Required |
| ------ | ------------------------------------- | ------------------- | ------------- |
| GET    | `/api/v1/realms/{realm}/clients`      | List OAuth2 clients | Yes (admin)   |
| POST   | `/api/v1/realms/{realm}/clients`      | Create client       | Yes (admin)   |
| GET    | `/api/v1/realms/{realm}/clients/{id}` | Get client details  | Yes (admin)   |
| PUT    | `/api/v1/realms/{realm}/clients/{id}` | Update client       | Yes (admin)   |
| DELETE | `/api/v1/realms/{realm}/clients/{id}` | Delete client       | Yes (admin)   |

### 10.7 Realm Management Endpoints

| Method | Endpoint                | Description       | Auth Required |
| ------ | ----------------------- | ----------------- | ------------- |
| GET    | `/api/v1/realms`        | List realms       | Yes (admin)   |
| POST   | `/api/v1/realms`        | Create realm      | Yes (admin)   |
| GET    | `/api/v1/realms/{name}` | Get realm details | Yes (admin)   |
| PUT    | `/api/v1/realms/{name}` | Update realm      | Yes (admin)   |
| DELETE | `/api/v1/realms/{name}` | Delete realm      | Yes (admin)   |

### 10.8 Audit Log Endpoints

| Method | Endpoint                       | Description       | Auth Required |
| ------ | ------------------------------ | ----------------- | ------------- |
| GET    | `/api/v1/auth/audit/logs`      | Get audit logs    | Yes (admin)   |
| GET    | `/api/v1/auth/audit/logs/{id}` | Get specific log  | Yes (admin)   |
| GET    | `/api/v1/auth/audit/events`    | Get event history | Yes (admin)   |
| POST   | `/api/v1/auth/audit/query`     | Query audit logs  | Yes (admin)   |

### 10.9 Health & Monitoring Endpoints

| Method | Endpoint          | Description         | Auth Required |
| ------ | ----------------- | ------------------- | ------------- |
| GET    | `/health`         | Health check        | No            |
| GET    | `/ready`          | Readiness check     | No            |
| GET    | `/live`           | Liveness check      | No            |
| GET    | `/metrics`        | Prometheus metrics  | No            |
| GET    | `/health/metrics` | Health with metrics | No            |

---

## 11. Microfrontend Integration

### 11.1 Portal Integration (Central Authentication)

Portal acts as the central authentication gateway. All microfrontends redirect to Portal for login.

**Architecture:**

```
User → Microfrontend (unauthenticated)
     → Redirect to Portal (/login?return_url=...)
     → CAPTCHA verification
     → MFA verification (if enabled)
     → OAuth2 authorization
     → Redirect back to Microfrontend (/callback?code=...)
     → Token exchange
     → Authenticated session
```

### 11.2 Example: Badiklat Microfrontend Integration

**Step 1: Check Authentication**

```rust
// In Badiklat main app component
use shared_microfrontend::hooks::use_auth;
use shared_microfrontend::components::auth::ProtectedRoute;

#[component]
pub fn App() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Router>
            <Routes>
                // Public route - login redirect
                <Route path="/" view=LoginRedirectPage />

                // OAuth2 callback handler
                <Route path="/callback" view=CallbackPage />

                // Protected routes - require authentication
                <Route path="/dashboard" view=|| {
                    view! {
                        <ProtectedRoute>
                            <Dashboard />
                        </ProtectedRoute>
                    }
                }/>

                <Route path="/courses" view=|| {
                    view! {
                        <ProtectedRoute>
                            <CoursesPage />
                        </ProtectedRoute>
                    }
                }/>
            </Routes>
        </Router>
    }
}
```

**Step 2: Login Redirect Page**

```rust
use shared_microfrontend::components::auth::OAuth2LoginButton;

#[component]
pub fn LoginRedirectPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    // If already authenticated, redirect to dashboard
    Effect::new(move |_| {
        if auth.is_authenticated() {
            navigate("/dashboard", Default::default());
        }
    });

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="max-w-md w-full space-y-8 p-8 bg-white rounded-lg shadow">
                <div class="text-center">
                    <h2 class="text-3xl font-bold text-gray-900">
                        "Badiklat SIMPEL"
                    </h2>
                    <p class="mt-2 text-sm text-gray-600">
                        "Login untuk mengakses sistem pelatihan dan pendidikan"
                    </p>
                </div>

                <OAuth2LoginButton
                    client_id="badiklat"
                    redirect_uri="http://localhost:8081/callback"
                    button_text="Masuk dengan Portal"
                />
            </div>
        </div>
    }
}
```

**Step 3: OAuth2 Callback Handler**

```rust
use shared_microfrontend::features::oauth::OAuthClient;
use shared_microfrontend::features::auth::AuthService;

#[component]
pub fn CallbackPage() -> impl IntoView {
    let (status, set_status) = signal(CallbackStatus::Processing);
    let (error, set_error) = signal(None::<String>);
    let navigate = use_navigate();

    Effect::new(move |_| {
        spawn_local(async move {
            // Extract code from URL
            let params = parse_url_params();
            match params.get("code") {
                Some(code) => {
                    // Exchange code for token
                    let oauth_client = OAuthClient::new(
                        "http://localhost:8088",
                        "simpel",
                        "badiklat",
                        "http://localhost:8081/callback"
                    );

                    match oauth_client.exchange_code(code).await {
                        Ok(token_response) => {
                            // Decode JWT and save session
                            match AuthService::decode_jwt_claims(&token_response.access_token) {
                                Ok(mut session) => {
                                    session.access_token = token_response.access_token;
                                    session.refresh_token = token_response.refresh_token;

                                    AuthService::save_session(&session);
                                    set_status.set(CallbackStatus::Success);

                                    // Redirect to dashboard
                                    navigate("/dashboard", Default::default());
                                }
                                Err(e) => {
                                    set_status.set(CallbackStatus::Error);
                                    set_error.set(Some(format!("Token decode error: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            set_status.set(CallbackStatus::Error);
                            set_error.set(Some(format!("Token exchange error: {}", e)));
                        }
                    }
                }
                None => {
                    set_status.set(CallbackStatus::Error);
                    set_error.set(Some("Missing authorization code".to_string()));
                }
            }
        });
    });

    view! {
        <div class="min-h-screen flex items-center justify-center">
            <Show when=move || status.get() == CallbackStatus::Processing>
                <div class="text-center">
                    <div class="spinner"></div>
                    <p>"Processing authentication..."</p>
                </div>
            </Show>

            <Show when=move || status.get() == CallbackStatus::Error>
                <div class="text-center text-red-600">
                    <p>"Authentication failed"</p>
                    <p class="text-sm">{move || error.get()}</p>
                    <a href="/" class="btn-primary">"Try Again"</a>
                </div>
            </Show>
        </div>
    }
}
```

### 11.3 Using Protected API Calls

```rust
use shared_microfrontend::hooks::use_auth;

#[component]
pub fn CoursesPage() -> impl IntoView {
    let auth = use_auth();
    let (courses, set_courses) = signal(Vec::new());
    let (loading, set_loading) = signal(true);

    // Fetch courses with authentication
    Effect::new(move |_| {
        spawn_local(async move {
            if let Some(token) = auth.get_access_token() {
                let client = reqwest::Client::new();
                let response = client
                    .get("http://localhost:3001/api/courses")
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<Vec<Course>>().await {
                            set_courses.set(data);
                        }
                    }
                    Err(_) => {
                        // Handle error - possibly token expired
                        // Try to refresh token
                        if let Ok(new_token) = auth.refresh_token().await {
                            // Retry request with new token
                            // ...
                        }
                    }
                }
            }
            set_loading.set(false);
        });
    });

    view! {
        <div class="container mx-auto p-4">
            <h1 class="text-2xl font-bold">"Daftar Pelatihan"</h1>

            <Show when=move || loading.get()>
                <div class="spinner"></div>
            </Show>

            <Show when=move || !loading.get()>
                <div class="grid grid-cols-3 gap-4 mt-4">
                    <For
                        each=move || courses.get()
                        key=|course| course.id
                        children=|course| {
                            view! {
                                <CourseCard course=course />
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
```

### 11.4 SSO Cookie Configuration

All microfrontends share SSO cookies for seamless authentication:

```rust
// SSO cookie name: sso_session
// Domain: .simpel.kejaksaan.go.id (shared across subdomains)
// Secure: true (HTTPS only in production)
// HttpOnly: true (prevent XSS)
// SameSite: Lax (CSRF protection)
// Path: /
// MaxAge: 8 hours

// Check SSO cookie on page load
if let Some(sso_session) = get_cookie("sso_session") {
    // User has active session in another microfrontend
    // Try silent token refresh
    match auth.silent_refresh().await {
        Ok(session) => {
            // Successfully authenticated via SSO
            AuthService::save_session(&session);
        }
        Err(_) => {
            // SSO session expired, redirect to login
            redirect_to_login();
        }
    }
}
```

### 11.5 Logout Propagation

When user logs out from one microfrontend, all tabs should be notified:

```rust
// Logout function
pub async fn logout() {
    let auth = use_auth();

    // Call logout API
    let _ = reqwest::Client::new()
        .post("http://localhost:8088/api/v1/auth/logout")
        .header("Authorization", format!("Bearer {}", auth.get_access_token().unwrap()))
        .send()
        .await;

    // Clear local session
    AuthService::clear_session();

    // Broadcast logout event to other tabs
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.set_item("logout_event", &Date::now().to_string());
    }

    // Redirect to login
    window().location().set_href("/").unwrap();
}

// Listen for logout events from other tabs
window().add_event_listener_with_callback("storage", move |event: StorageEvent| {
    if event.key() == Some("logout_event".to_string()) {
        // Another tab logged out, clear session and redirect
        AuthService::clear_session();
        window().location().set_href("/").unwrap();
    }
});
```

---

## 12. Monitoring & Operations

### 12.1 Health Checks

**Basic Health Check:**

```bash
curl -f http://localhost:8088/health
```

**Readiness Check (for K8s):**

```bash
curl -f http://localhost:8088/ready
```

**Liveness Check (for K8s):**

```bash
curl -f http://localhost:8088/live
```

### 12.2 Prometheus Metrics

Authenc exposes Prometheus metrics at `/metrics`:

```bash
curl -s http://localhost:8088/metrics | grep authenc
```

**Available Metrics:**

```
# Authentication metrics
authenc_login_attempts_total{status="success|failure",realm="simpel"}
authenc_mfa_verifications_total{status="success|failure",method="totp|webauthn"}
authenc_token_issued_total{grant_type="authorization_code|refresh_token"}
authenc_token_refreshed_total
authenc_token_revoked_total

# Performance metrics
authenc_http_request_duration_seconds{method="GET|POST",endpoint="/oauth2/token"}
authenc_http_requests_total{method="GET|POST",status="200|401|500"}

# Database metrics
authenc_db_connections_active
authenc_db_connections_idle
authenc_db_query_duration_seconds

# Cache metrics (Redis)
authenc_cache_hits_total
authenc_cache_misses_total
authenc_cache_size_bytes

# Security metrics
authenc_brute_force_attempts_total
authenc_rate_limit_exceeded_total
authenc_suspicious_activities_total
```

**Prometheus Configuration:**

```yaml
# prometheus.yml
scrape_configs:
  - job_name: "authenc"
    static_configs:
      - targets: ["localhost:8088"]
    metrics_path: "/metrics"
    scrape_interval: 15s
```

### 12.3 Logging

**View Logs:**

```bash
# Systemd logs
sudo journalctl -u authenc -f

# Follow latest 100 lines
sudo journalctl -u authenc -n 100 -f

# Filter by log level
sudo journalctl -u authenc | grep "ERROR"
sudo journalctl -u authenc | grep "WARN"

# View logs from specific time
sudo journalctl -u authenc --since "2025-11-10 10:00:00"
```

**Log Format (JSON):**

```json
{
  "timestamp": "2025-11-10T10:30:15.123Z",
  "level": "INFO",
  "target": "authenc::handlers::auth",
  "message": "User login successful",
  "fields": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "user@kejaksaan.go.id",
    "realm": "simpel",
    "ip_address": "192.168.1.100",
    "user_agent": "Mozilla/5.0..."
  }
}
```

### 12.4 Audit Log Query Examples

**Get all login attempts today:**

```bash
curl -X POST http://localhost:8088/api/v1/auth/audit/query \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "login",
    "start_date": "2025-11-10T00:00:00Z",
    "end_date": "2025-11-10T23:59:59Z"
  }'
```

**Get failed login attempts for specific user:**

```bash
curl -X POST http://localhost:8088/api/v1/auth/audit/query \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "login",
    "status": "failure",
    "username": "user@kejaksaan.go.id",
    "limit": 50
  }'
```

**Get all admin actions:**

```bash
curl -X POST http://localhost:8088/api/v1/auth/audit/query \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "resource_type": "user",
    "action": "create|update|delete",
    "start_date": "2025-11-01T00:00:00Z"
  }'
```

### 12.5 Database Maintenance

**Backup:**

```bash
# Create backup
sudo -u postgres pg_dump authenc | gzip > authenc_backup_$(date +%Y%m%d_%H%M%S).sql.gz
```

**Restore:**

```bash
# Restore from backup
gunzip -c authenc_backup_20251110_100000.sql.gz | sudo -u postgres psql authenc
```

**Vacuum (cleanup):**

```sql
-- Connect to database
psql -U authenc authenc

-- Vacuum all tables
VACUUM ANALYZE;

-- Specific table
VACUUM ANALYZE users;
```

**Check Table Sizes:**

```sql
SELECT
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

---

## 13. Troubleshooting

### 13.1 Common Issues

#### Issue 1: "Connection refused" when starting Authenc

**Symptoms:**

```
Error: Failed to connect to database
Connection refused at localhost:5432
```

**Solution:**

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Start PostgreSQL if not running
sudo systemctl start postgresql

# Enable on boot
sudo systemctl enable postgresql
```

#### Issue 2: JWT tokens invalidated after restart

**Symptoms:**

- Users get "Invalid token" errors after Authenc restart
- All sessions lost on restart

**Solution:**

Set a persistent Ed25519 signing key:

```bash
# Generate key once
openssl genpkey -algorithm ed25519 -out authenc-ed25519.pem
openssl pkey -in authenc-ed25519.pem -outform DER | base64 -w 0 > ed25519-key.base64

# Add to config
export ED25519_PRIVATE_KEY_BASE64=$(cat ed25519-key.base64)
```

#### Issue 3: MFA setup fails with "Secreton connection error"

**Symptoms:**

```
Error: Failed to setup MFA
Secreton endpoint unreachable
```

**Solution:**

```bash
# Check if Secreton is running
curl -f http://localhost:8200/health

# Start Secreton
cd /srv/proyek/simpelv2/layanan/secreton
cargo run

# Check Secreton token is correct
echo $SECRETON_TOKEN
```

#### Issue 4: High database connection count

**Symptoms:**

```
FATAL:  sorry, too many clients already
```

**Solution:**

```sql
-- Check current connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'authenc';

-- Increase max_connections in postgresql.conf
max_connections = 200

-- Restart PostgreSQL
sudo systemctl restart postgresql

-- Or reduce connection pool size in Authenc config.toml
[database]
max_connections = 50
```

#### Issue 5: CORS errors in browser

**Symptoms:**

```
Access to fetch at 'http://localhost:8088/oauth2/token' from origin
'http://localhost:8080' has been blocked by CORS policy
```

**Solution:**

```toml
# Add origin to cors_allowed_origins in config.toml
[security]
cors_allowed_origins = [
    "http://localhost:8080",
    "http://localhost:8081",
    "https://portal.simpel.kejaksaan.go.id"
]
```

### 13.2 Debug Mode

Enable debug logging for detailed troubleshooting:

```bash
# Temporary (current session)
export LOG_LEVEL=debug
systemctl restart authenc

# View debug logs
sudo journalctl -u authenc -f
```

### 13.3 Performance Troubleshooting

**Check slow queries:**

```sql
-- Enable slow query logging
ALTER SYSTEM SET log_min_duration_statement = 1000;  -- Log queries > 1 second
SELECT pg_reload_conf();

-- View pg_stat_statements
SELECT
  query,
  calls,
  total_time,
  mean_time,
  max_time
FROM pg_stat_statements
WHERE query LIKE '%authenc%'
ORDER BY mean_time DESC
LIMIT 10;
```

**Check Redis performance:**

```bash
# Redis info
redis-cli info stats

# Monitor commands
redis-cli monitor

# Check slow log
redis-cli slowlog get 10
```

### 13.4 Support & Contact

**Documentation:**

- API Documentation: `/docs/AUTHENC_API_DOCUMENTATION.md`
- MFA Guide: `/docs/MFA_ARCHITECTURE_DOCUMENTATION.md`
- Architecture: `/docs/SIMKARI_ARCHITECTURE_DOCUMENTATION.md`

**Issue Reporting:**

- GitHub Issues: https://github.com/analisaperlengkapan/simpel2/issues
- Email: security@kejaksaan.go.id

**Community:**

- GitHub Discussions: https://github.com/analisaperlengkapan/simpel2/discussions

---

## 14. Security Best Practices

### 14.1 Production Deployment Checklist

- [ ] **Use HTTPS/TLS** for all production endpoints
- [ ] **Set persistent Ed25519 signing key** in environment
- [ ] **Use strong JWT_SECRET** (256-bit minimum)
- [ ] **Enable MFA** for all admin accounts
- [ ] **Configure CORS** with specific allowed origins
- [ ] **Enable rate limiting** to prevent brute force attacks
- [ ] **Use strong database passwords** (20+ characters, mixed case, special chars)
- [ ] **Enable audit logging** for all sensitive operations
- [ ] **Set up database backups** (automated daily backups)
- [ ] **Configure log rotation** to prevent disk space issues
- [ ] **Use Redis** for session caching and performance
- [ ] **Enable firewall** rules (restrict database and Redis to localhost)
- [ ] **Set resource limits** in systemd service file
- [ ] **Secure secrets management** (use Secreton vault)
- [ ] **Regular security updates** for dependencies
- [ ] **Monitor metrics** with Prometheus and Grafana
- [ ] **Set up alerting** for critical events
- [ ] **Implement backup and disaster recovery** plan
- [ ] **Review and update** security policies quarterly

### 14.2 Password Policies

**Enforce strong passwords:**

```toml
# config.toml
[security]
password_min_length = 12
password_require_uppercase = true
password_require_lowercase = true
password_require_digits = true
password_require_special_chars = true
password_expiration_days = 90
password_history_count = 5  # Prevent reusing last 5 passwords
```

### 14.3 Session Management

**Best practices:**

```toml
[security]
session_timeout = 3600              # 1 hour idle timeout
session_absolute_timeout = 28800    # 8 hours absolute timeout
session_single_device = false       # Allow multiple sessions
session_ip_binding = true           # Bind session to IP address
```

### 14.4 Rate Limiting

**Configure rate limits:**

```toml
[security]
enable_rate_limiting = true
rate_limit_requests_per_minute = 100
rate_limit_burst = 20

# Stricter limits for sensitive endpoints
auth_login_rate_limit = 5           # 5 attempts per minute
mfa_verify_rate_limit = 3           # 3 attempts per minute
```

### 14.5 Audit & Compliance

**GDPR Compliance:**

- Log all user data access
- Provide data export functionality
- Implement data deletion on user request
- Maintain audit trail for 1 year minimum

**Security Monitoring:**

```bash
# Monitor for suspicious activities
curl -X POST http://localhost:8088/api/v1/auth/audit/query \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "login",
    "status": "failure",
    "limit": 100
  }' | jq '.[] | select(.attempts > 5)'
```

---

## 15. Quick Reference

### 15.1 Essential Commands

```bash
# Start Authenc
sudo systemctl start authenc

# Stop Authenc
sudo systemctl stop authenc

# Restart Authenc
sudo systemctl restart authenc

# Check status
sudo systemctl status authenc

# View logs
sudo journalctl -u authenc -f

# Health check
curl -f http://localhost:8088/health

# Database backup
sudo -u postgres pg_dump authenc | gzip > authenc_backup.sql.gz

# Database restore
gunzip -c authenc_backup.sql.gz | sudo -u postgres psql authenc
```

### 15.2 Configuration Files

| File            | Purpose               | Location                              |
| --------------- | --------------------- | ------------------------------------- |
| config.toml     | Main configuration    | `/etc/authenc/config.toml`            |
| authenc.env     | Environment variables | `/etc/authenc/authenc.env`            |
| authenc.service | Systemd service       | `/etc/systemd/system/authenc.service` |
| authenc.log     | Application logs      | `/var/log/authenc/authenc.log`        |
| ed25519.pem     | Signing key           | `/etc/authenc/ed25519.pem`            |

### 15.3 Default Ports

| Service       | Port | Description       |
| ------------- | ---- | ----------------- |
| Authenc HTTP  | 8088 | Main API endpoint |
| Authenc HTTPS | 8089 | TLS-secured API   |
| PostgreSQL    | 5432 | Database          |
| Redis         | 6379 | Cache             |
| Prometheus    | 9090 | Metrics           |

### 15.4 Environment Variables Quick Reference

```bash
# Core settings
DATABASE_URL=postgresql://authenc:password@localhost/authenc
JWT_SECRET=your-256-bit-secret
ED25519_PRIVATE_KEY_BASE64=your-base64-encoded-key

# Network
AUTHENC_HOST=0.0.0.0
AUTHENC_PORT=8088

# Integrations
REDIS_URL=redis://localhost:6379
SECRETON_ENDPOINT=http://localhost:8200
SECRETON_TOKEN=your-secreton-admin-token

# Security
CORS_ALLOWED_ORIGINS=http://localhost:8080
ENABLE_RATE_LIMITING=true

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
```

### 15.5 Common cURL Examples

**Login:**

```bash
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass","realm":"simpel"}'
```

**Create User:**

```bash
curl -X POST http://localhost:8088/api/v1/auth/users \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"newuser","email":"user@example.com","password":"SecurePass123!"}'
```

**Assign Role:**

```bash
curl -X POST "http://localhost:8088/api/v1/realms/simpel/users/{user_id}/roles" \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"role_name":"Jaksa"}'
```

**Check Permission:**

```bash
curl -X POST http://localhost:8088/api/v1/auth/permissions/check \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"resource":"cases","action":"write"}'
```

---

## Appendix A: Glossary

- **Authenc**: Authentication and Authorization Engine for SIMPEL
- **OAuth2**: Open Authorization 2.0 protocol for delegated access
- **OIDC**: OpenID Connect, identity layer on top of OAuth2
- **JWT**: JSON Web Token, compact token format for claims
- **MFA**: Multi-Factor Authentication
- **TOTP**: Time-based One-Time Password (RFC 6238)
- **PKCE**: Proof Key for Code Exchange (RFC 7636)
- **RBAC**: Role-Based Access Control
- **Ed25519**: EdDSA signature scheme using Curve25519
- **JWKS**: JSON Web Key Set
- **Realm**: Multi-tenancy isolation unit
- **Satker**: Satuan Kerja (organizational unit)
- **Wilayah**: Regional administrative area
- **NIP**: Nomor Induk Pegawai (employee ID number)

## Appendix B: Related Documentation

- `SECRETON_GETTING_STARTED_GUIDE.md` - Secreton vault setup guide
- `MFA_ARCHITECTURE_DOCUMENTATION.md` - Detailed MFA implementation
- `MFA_USER_GUIDE.md` - End-user MFA guide (Bahasa Indonesia)
- `MFA_ADMIN_GUIDE.md` - MFA administration guide
- `DEPLOYMENT_INTEGRATION_GUIDE.md` - Full stack deployment
- `ATTORNEY_GENERAL_SECURITY_CONSIDERATIONS.md` - Security requirements
- `SIMKARI_ARCHITECTURE_DOCUMENTATION.md` - Overall system architecture

## Appendix C: Version History

| Version | Date       | Changes                             |
| ------- | ---------- | ----------------------------------- |
| 0.4.0   | 2025-11-10 | Initial comprehensive documentation |

---

**End of Authenc Getting Started Guide**

For questions, issues, or contributions:

- GitHub: https://github.com/analisaperlengkapan/simpel2
- Email: security@kejaksaan.go.id

---

© 2025 Kejaksaan Agung Republik Indonesia. All rights reserved.
