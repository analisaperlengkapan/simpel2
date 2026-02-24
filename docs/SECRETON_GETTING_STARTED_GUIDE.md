# Secreton - Getting Started Guide

**Complete Guide: From Zero to Production**

> Panduan lengkap penggunaan Secreton dari instalasi awal hingga operasional production. Dokumen ini mencakup setup, konfigurasi, operasi harian, dan troubleshooting.

**Versi:** 1.0
**Tanggal:** November 2025
**Target Audience:** DevOps Engineers, System Administrators, Security Engineers

---

## 📋 Table of Contents

1. [Overview](#1-overview)
2. [Prerequisites](#2-prerequisites)
3. [Quick Start (Development)](#3-quick-start-development)
4. [Production Setup](#4-production-setup)
5. [Initial Configuration](#5-initial-configuration)
6. [First-Time Initialization](#6-first-time-initialization)
7. [Unseal Process](#7-unseal-process)
8. [Basic Operations](#8-basic-operations)
9. [Advanced Features](#9-advanced-features)
10. [Monitoring & Maintenance](#10-monitoring--maintenance)
11. [Troubleshooting](#11-troubleshooting)
12. [Security Best Practices](#12-security-best-practices)

---

## 1. Overview

### 1.1 What is Secreton?

Secreton adalah enterprise-grade security vault system yang menyediakan:

- **Secret Management** - Penyimpanan aman untuk credentials, API keys, certificates
- **Encryption-as-a-Service** - Transit encryption untuk aplikasi
- **Key Management** - Lifecycle management untuk cryptographic keys
- **Audit Trail** - Comprehensive logging untuk compliance
- **Post-Quantum Ready** - ML-KEM dan ML-DSA support

### 1.2 Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Client Applications                   │
│         (REST API / gRPC / CLI / Web Interface)         │
└────────────────────┬────────────────────────────────────┘
                     │ HTTP/HTTPS (TLS 1.3)
┌────────────────────▼────────────────────────────────────┐
│                   Secreton API Server                    │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐ │
│  │ Transit     │  │ KV Secrets   │  │ Key Management │ │
│  │ Engine      │  │ Engine       │  │ (KMIP)         │ │
│  └─────────────┘  └──────────────┘  └────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐│
│  │          Seal/Unseal Service (Shamir)               ││
│  └─────────────────────────────────────────────────────┘│
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              Storage Backend (Pluggable)                 │
│  ┌──────────┐  ┌────────┐  ┌──────┐  ┌───────────────┐│
│  │   File   │  │ Consul │  │ Raft │  │  PostgreSQL   ││
│  │ (Default)│  │  (HA)  │  │(Cons)│  │   (Legacy)    ││
│  └──────────┘  └────────┘  └──────┘  └───────────────┘│
└─────────────────────────────────────────────────────────┘
```

### 1.3 Key Concepts

**Seal/Unseal:**

- **Sealed** - Vault tidak dapat diakses, master key di-encrypt
- **Unsealed** - Vault aktif, master key di-load ke memory
- **Shamir Secret Sharing** - Master key dibagi menjadi N shares, butuh M shares untuk unseal

**Storage Backends:**

- **File** - Local filesystem (development, single-node)
- **Consul** - Distributed HA (production, multi-datacenter)
- **Raft** - Consensus-based (production, integrated HA)
- **PostgreSQL** - Relational database (legacy support)

**Engines:**

- **Transit Engine** - Encryption/decryption as a service
- **KV Engine** - Key-value secret storage dengan versioning
- **Transform Engine** - Tokenization, FPE, data masking

---

## 2. Prerequisites

### 2.1 System Requirements

**Minimum (Development):**

- CPU: 2 cores
- RAM: 2 GB
- Disk: 10 GB
- OS: Linux, macOS, Windows

**Recommended (Production):**

- CPU: 4+ cores
- RAM: 8+ GB
- Disk: 100+ GB SSD
- OS: Linux (Ubuntu 22.04 LTS, RHEL 9)

### 2.2 Software Dependencies

**Required:**

```bash
# Rust toolchain (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build tools
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

**Optional (for specific backends):**

```bash
# Consul (untuk HA backend)
wget https://releases.hashicorp.com/consul/1.17.0/consul_1.17.0_linux_amd64.zip
unzip consul_1.17.0_linux_amd64.zip
sudo mv consul /usr/local/bin/

# PostgreSQL (untuk legacy backend)
sudo apt-get install -y postgresql-client libpq-dev
```

### 2.3 Network Requirements

**Ports:**

- `8080` - HTTP API (development)
- `8443` - HTTPS API (production)
- `50051` - gRPC server (optional)

**Firewall Rules:**

```bash
# Allow HTTP/HTTPS
sudo ufw allow 8080/tcp
sudo ufw allow 8443/tcp

# Allow gRPC (optional)
sudo ufw allow 50051/tcp
```

---

## 3. Quick Start (Development)

**Time Required:** 5-10 minutes

### 3.1 Build from Source

```bash
# Clone repository
cd /srv/proyek/simpelv2/layanan/secreton

# Build API server
cargo build --release --bin api_server

# Binary location
ls -lh target/release/api_server
```

### 3.2 Start Server (Development Mode)

```bash
# Start with default file backend
./target/release/api_server

# Server akan start di http://127.0.0.1:8080
# Storage di /var/lib/secreton/data (default)
```

**Expected Output:**

```
2025-11-10T10:00:00.000Z INFO  Secreton API Server starting...
2025-11-10T10:00:00.100Z INFO  Storage backend: File (/var/lib/secreton/data)
2025-11-10T10:00:00.200Z INFO  🔒 Vault is SEALED at startup
2025-11-10T10:00:00.201Z WARN     All secret operations blocked until unsealed
2025-11-10T10:00:00.202Z WARN     Use /v1/sys/unseal endpoint with shares
2025-11-10T10:00:00.300Z INFO  HTTP server listening on 127.0.0.1:8080
2025-11-10T10:00:00.301Z INFO  ✅ Secreton ready!
```

### 3.3 Initialize Vault (First Time)

```bash
# Initialize dengan default config (5 shares, 3 threshold)
curl -X POST http://127.0.0.1:8080/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{
    "secret_shares": 5,
    "secret_threshold": 3
  }' | jq '.' > secreton-init-keys.json

# Output tersimpan di secreton-init-keys.json
cat secreton-init-keys.json
```

**Response Structure:**

```json
{
  "keys": [
    "base64-encoded-share-1",
    "base64-encoded-share-2",
    "base64-encoded-share-3",
    "base64-encoded-share-4",
    "base64-encoded-share-5"
  ],
  "root_token": "hvs.XXXXXXXXXXXXXXXXXXXXX"
}
```

⚠️ **CRITICAL SECURITY:**

- **BACKUP** file `secreton-init-keys.json` di lokasi aman
- **JANGAN COMMIT** ke git atau share via insecure channel
- **DISTRIBUTE** shares ke different operators
- **STORE** root_token securely (password manager, HSM)

### 3.4 Unseal Vault

```bash
# Extract shares dari init output
SHARE_1=$(cat secreton-init-keys.json | jq -r '.keys[0]')
SHARE_2=$(cat secreton-init-keys.json | jq -r '.keys[1]')
SHARE_3=$(cat secreton-init-keys.json | jq -r '.keys[2]')

# Unseal dengan 3 shares (threshold)
curl -X POST http://127.0.0.1:8080/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\": \"$SHARE_1\"}"

curl -X POST http://127.0.0.1:8080/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\": \"$SHARE_2\"}"

curl -X POST http://127.0.0.1:8080/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d "{\"key\": \"$SHARE_3\"}"
```

**Check Seal Status:**

```bash
curl http://127.0.0.1:8080/v1/sys/seal-status | jq '.'
```

**Expected Response (Unsealed):**

```json
{
  "sealed": false,
  "threshold": 3,
  "shares": 5,
  "progress": 0
}
```

### 3.5 Test Basic Operations

```bash
# Extract root token
ROOT_TOKEN=$(cat secreton-init-keys.json | jq -r '.root_token')

# Create a secret
curl -X POST http://127.0.0.1:8080/v1/secret/data/my-app/config \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "api_key": "my-secret-key-12345",
      "database_url": "postgresql://user:pass@localhost/db"
    }
  }'

# Read the secret
curl http://127.0.0.1:8080/v1/secret/data/my-app/config \
  -H "Authorization: Bearer $ROOT_TOKEN" | jq '.'

# Encrypt data
curl -X POST http://127.0.0.1:8080/v1/transit/encrypt/my-key \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "plaintext": "SGVsbG8gU2VjcmV0b24="
  }' | jq '.'
```

🎉 **Congratulations!** Secreton sudah berjalan di development mode.

---

## 4. Production Setup

### 4.1 Directory Structure

```bash
# Create production directories
sudo mkdir -p /etc/secreton
sudo mkdir -p /var/lib/secreton/data
sudo mkdir -p /var/log/secreton
sudo mkdir -p /opt/secreton/bin

# Set permissions
sudo chown -R secreton:secreton /etc/secreton
sudo chown -R secreton:secreton /var/lib/secreton
sudo chown -R secreton:secreton /var/log/secreton
sudo chmod 700 /var/lib/secreton/data
```

### 4.2 Create System User

```bash
# Create dedicated user
sudo useradd -r -s /bin/false -d /var/lib/secreton secreton

# Verify
id secreton
```

### 4.3 Install Binary

```bash
# Build production binary
cd /srv/proyek/simpelv2/layanan/secreton
cargo build --release --bin api_server

# Install
sudo cp target/release/api_server /opt/secreton/bin/
sudo chown secreton:secreton /opt/secreton/bin/api_server
sudo chmod 755 /opt/secreton/bin/api_server
```

### 4.4 TLS Certificates

**Generate Self-Signed (Development/Testing):**

```bash
# Generate private key
openssl genrsa -out /etc/secreton/secreton-key.pem 4096

# Generate certificate (10 years)
openssl req -new -x509 -sha256 \
  -key /etc/secreton/secreton-key.pem \
  -out /etc/secreton/secreton-cert.pem \
  -days 3650 \
  -subj "/C=ID/ST=Jakarta/L=Jakarta/O=Kejaksaan Agung RI/CN=secreton.kejaksaan.go.id"

# Set permissions
sudo chown secreton:secreton /etc/secreton/*.pem
sudo chmod 600 /etc/secreton/*.pem
```

**Production (Let's Encrypt):**

```bash
# Install certbot
sudo apt-get install -y certbot

# Get certificate
sudo certbot certonly --standalone \
  -d secreton.kejaksaan.go.id \
  --email admin@kejaksaan.go.id

# Symlink to secreton directory
sudo ln -s /etc/letsencrypt/live/secreton.kejaksaan.go.id/fullchain.pem \
  /etc/secreton/secreton-cert.pem
sudo ln -s /etc/letsencrypt/live/secreton.kejaksaan.go.id/privkey.pem \
  /etc/secreton/secreton-key.pem
```

---

## 5. Initial Configuration

### 5.1 Configuration File

Create `/etc/secreton/config.toml`:

```toml
# Secreton Production Configuration
# Location: /etc/secreton/config.toml

[http]
bind_address = "0.0.0.0:8443"
timeout_secs = 30
max_body_size = 10485760  # 10 MB
keep_alive_secs = 60
compression = true

[grpc]
enabled = true
bind_address = "0.0.0.0:50051"
timeout_secs = 30
max_message_size = 4194304  # 4 MB
reflection = true
health_check = true

[tls]
enabled = true
cert_file = "/etc/secreton/secreton-cert.pem"
key_file = "/etc/secreton/secreton-key.pem"
# min_version = "1.3"  # TLS 1.3 only

[auth.jwt]
secret = "change-this-to-random-256-bit-secret"
expiration_secs = 3600  # 1 hour
refresh_expiration_secs = 604800  # 7 days
algorithm = "HS256"
issuer = "secreton.kejaksaan.go.id"
audience = "secreton-clients"

[rate_limit]
enabled = true
requests_per_second = 100
burst_size = 200

[logging]
level = "info"
format = "json"
output = "file"
file_path = "/var/log/secreton/secreton.log"
rotation = "daily"
max_files = 30

[monitoring]
metrics_enabled = true
metrics_port = 9090
health_check_interval_secs = 30

[storage]
backend = "file"  # Options: file, consul, raft, postgres

# File backend configuration
[storage.file]
path = "/var/lib/secreton/data"
sync_writes = true
permissions = 0o600
```

### 5.2 Environment Variables

Create `/etc/secreton/secreton.env`:

```bash
# Secreton Environment Variables
# Location: /etc/secreton/secreton.env

# Storage Backend
SECRETON_STORAGE_BACKEND=file

# HTTP Server
SECRETON_HTTP_BIND=0.0.0.0:8443
SECRETON_HTTP_TIMEOUT=30

# TLS Configuration
SECRETON_TLS_ENABLED=true
SECRETON_TLS_CERT=/etc/secreton/secreton-cert.pem
SECRETON_TLS_KEY=/etc/secreton/secreton-key.pem

# Logging
SECRETON_LOG_LEVEL=info
SECRETON_LOG_FORMAT=json
SECRETON_LOG_FILE=/var/log/secreton/secreton.log

# Authentication
SECRETON_JWT_SECRET=your-256-bit-secret-here

# Rate Limiting
SECRETON_RATE_LIMIT_RPS=100
SECRETON_RATE_LIMIT_BURST=200

# Database (if using postgres backend)
# DATABASE_URL=postgresql://secreton:password@localhost/secreton_db

# Consul (if using consul backend)
# CONSUL_ADDRESS=http://localhost:8500
# CONSUL_TOKEN=your-consul-token

# TPM (if using TPM for seal)
# SECRETON_TPM_ENABLED=false
# SECRETON_TPM_DEVICE=/dev/tpm0
```

**Set Secure Permissions:**

```bash
sudo chmod 600 /etc/secreton/secreton.env
sudo chown secreton:secreton /etc/secreton/secreton.env
```

### 5.3 Systemd Service

Create `/etc/systemd/system/secreton.service`:

```ini
[Unit]
Description=Secreton Security Vault System
Documentation=https://github.com/analisaperlengkapan/simpel2/tree/main/layanan/secreton
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=secreton
Group=secreton
ExecStart=/opt/secreton/bin/api_server
EnvironmentFile=/etc/secreton/secreton.env
WorkingDirectory=/var/lib/secreton

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/secreton /var/log/secreton
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

# Restart policy
Restart=on-failure
RestartSec=5s
StartLimitInterval=60s
StartLimitBurst=3

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=secreton

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

**Enable and Start:**

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable auto-start
sudo systemctl enable secreton

# Start service
sudo systemctl start secreton

# Check status
sudo systemctl status secreton

# View logs
sudo journalctl -u secreton -f
```

---

## 6. First-Time Initialization

### 6.1 Initialize Vault

**CRITICAL:** This can only be done ONCE. Save the output securely!

```bash
# Initialize with production settings
curl -X POST https://secreton.kejaksaan.go.id:8443/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{
    "secret_shares": 7,
    "secret_threshold": 4
  }' \
  --cacert /etc/secreton/secreton-cert.pem \
  | jq '.' > /root/secreton-init-keys-$(date +%Y%m%d-%H%M%S).json

# Secure the file
chmod 600 /root/secreton-init-keys-*.json
```

**Response Structure:**

```json
{
  "keys": [
    "share-1-base64...",
    "share-2-base64...",
    "share-3-base64...",
    "share-4-base64...",
    "share-5-base64...",
    "share-6-base64...",
    "share-7-base64..."
  ],
  "root_token": "hvs.CAES..."
}
```

### 6.2 Distribute Shamir Shares

**Security Protocol:**

1. **Print Shares** - Print each share on separate paper
2. **Distribute** - Give to different trusted operators:

   - Share 1 → Kepala Kejaksaan
   - Share 2 → Kepala IT Security
   - Share 3 → Senior SysAdmin 1
   - Share 4 → Senior SysAdmin 2
   - Share 5 → Backup Officer 1
   - Share 6 → Backup Officer 2
   - Share 7 → Backup Officer 3

3. **Secure Storage** - Each operator stores share in:

   - Physical safe
   - Password manager (encrypted)
   - HSM device (production)

4. **Documentation** - Record who has which share (without revealing share content)

### 6.3 Store Root Token

**Options:**

**Option 1: Password Manager**

```bash
# Store in KeePass, 1Password, LastPass, etc.
# Entry: "Secreton Root Token - Production"
# Username: root
# Password: <root_token>
```

**Option 2: Hardware Security Module (HSM)**

```bash
# Store in TPM, YubiKey, or enterprise HSM
# Requires HSM integration setup
```

**Option 3: Encrypted File (Temporary)**

```bash
# Encrypt with GPG
gpg --symmetric --cipher-algo AES256 \
  /root/secreton-init-keys-*.json

# Store encrypted version
mv /root/secreton-init-keys-*.json.gpg /secure/backup/

# Delete plaintext
shred -u /root/secreton-init-keys-*.json
```

---

## 7. Unseal Process

### 7.1 Manual Unseal (After Server Restart)

**Scenario:** Server reboot, service restart, atau seal operation.

```bash
# Check seal status
curl -k https://secreton.kejaksaan.go.id:8443/v1/sys/seal-status | jq '.'

# Response:
# {
#   "sealed": true,
#   "threshold": 4,
#   "shares": 7,
#   "progress": 0
# }
```

**Unseal dengan 4 shares:**

```bash
# Operator 1 provides share 1
curl -X POST https://secreton.kejaksaan.go.id:8443/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "share-1-base64..."}' \
  --cacert /etc/secreton/secreton-cert.pem

# Response: {"sealed": true, "progress": 1, "threshold": 4}

# Operator 2 provides share 2
curl -X POST https://secreton.kejaksaan.go.id:8443/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "share-2-base64..."}' \
  --cacert /etc/secreton/secreton-cert.pem

# Response: {"sealed": true, "progress": 2, "threshold": 4}

# Operator 3 provides share 3
curl -X POST https://secreton.kejaksaan.go.id:8443/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "share-3-base64..."}' \
  --cacert /etc/secreton/secreton-cert.pem

# Response: {"sealed": true, "progress": 3, "threshold": 4}

# Operator 4 provides share 4
curl -X POST https://secreton.kejaksaan.go.id:8443/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "share-4-base64..."}' \
  --cacert /etc/secreton/secreton-cert.pem

# Response: {"sealed": false, "progress": 0, "threshold": 4}
# ✅ VAULT UNSEALED!
```

### 7.2 Automated Unseal (Production)

**Option 1: Transit Auto-Unseal (Recommended)**

Uses another Secreton instance to auto-unseal:

```toml
# /etc/secreton/config.toml
[seal]
type = "transit"

[seal.transit]
address = "https://master-secreton.kejaksaan.go.id:8443"
token = "hvs.XXXXX"
mount_path = "transit"
key_name = "autounseal"
```

**Option 2: AWS KMS Auto-Unseal**

```toml
[seal]
type = "awskms"

[seal.awskms]
region = "ap-southeast-1"
kms_key_id = "arn:aws:kms:ap-southeast-1:123456789:key/xxx"
```

**Option 3: Azure Key Vault Auto-Unseal**

```toml
[seal]
type = "azurekeyvault"

[seal.azurekeyvault]
tenant_id = "your-tenant-id"
client_id = "your-client-id"
client_secret = "your-secret"
vault_name = "secreton-vault"
key_name = "secreton-unseal-key"
```

### 7.3 Unseal Script (Development Only)

**⚠️ WARNING:** Only for development/testing. DO NOT use in production!

```bash
#!/bin/bash
# File: /opt/secreton/scripts/dev-unseal.sh
# DEVELOPMENT ONLY - NOT FOR PRODUCTION

SECRETON_URL="http://localhost:8080"
KEYS_FILE="/root/secreton-dev-keys.json"

if [ ! -f "$KEYS_FILE" ]; then
    echo "Error: Keys file not found: $KEYS_FILE"
    exit 1
fi

# Extract shares
SHARE_1=$(jq -r '.keys[0]' "$KEYS_FILE")
SHARE_2=$(jq -r '.keys[1]' "$KEYS_FILE")
SHARE_3=$(jq -r '.keys[2]' "$KEYS_FILE")

echo "🔓 Unsealing Secreton (DEV MODE)..."

# Unseal with 3 shares
curl -X POST "$SECRETON_URL/v1/sys/unseal" \
  -H "Content-Type: application/json" \
  -d "{\"key\": \"$SHARE_1\"}" -s > /dev/null

curl -X POST "$SECRETON_URL/v1/sys/unseal" \
  -H "Content-Type: application/json" \
  -d "{\"key\": \"$SHARE_2\"}" -s > /dev/null

curl -X POST "$SECRETON_URL/v1/sys/unseal" \
  -H "Content-Type: application/json" \
  -d "{\"key\": \"$SHARE_3\"}" -s > /dev/null

# Check status
STATUS=$(curl -s "$SECRETON_URL/v1/sys/seal-status" | jq -r '.sealed')

if [ "$STATUS" = "false" ]; then
    echo "✅ Vault unsealed successfully!"
else
    echo "❌ Failed to unseal vault"
    exit 1
fi
```

---

## 8. Basic Operations

### 8.1 Secret Management

#### 8.1.1 Store Secret

```bash
# Set root token
export ROOT_TOKEN="hvs.CAES..."
export SECRETON_URL="https://secreton.kejaksaan.go.id:8443"

# Create secret
curl -X POST "$SECRETON_URL/v1/secret/data/applications/myapp/config" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "database_url": "postgresql://user:pass@db.kejaksaan.go.id/myapp",
      "api_key": "sk_live_51H...",
      "encryption_key": "base64-encoded-key...",
      "smtp_password": "secret123"
    },
    "metadata": {
      "environment": "production",
      "owner": "tim-aplikasi",
      "created_by": "admin@kejaksaan.go.id"
    }
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

**Response:**

```json
{
  "success": true,
  "data": {
    "version": 1,
    "created_at": "2025-11-10T10:00:00Z"
  }
}
```

#### 8.1.2 Read Secret

```bash
# Retrieve secret
curl "$SECRETON_URL/v1/secret/data/applications/myapp/config" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

**Response:**

```json
{
  "success": true,
  "data": {
    "path": "applications/myapp/config",
    "data": {
      "database_url": "postgresql://user:pass@db.kejaksaan.go.id/myapp",
      "api_key": "sk_live_51H...",
      "encryption_key": "base64-encoded-key...",
      "smtp_password": "secret123"
    },
    "metadata": {
      "version": 1,
      "created_at": "2025-11-10T10:00:00Z",
      "updated_at": "2025-11-10T10:00:00Z"
    }
  }
}
```

#### 8.1.3 Update Secret

```bash
# Update creates new version
curl -X PUT "$SECRETON_URL/v1/secret/data/applications/myapp/config" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "database_url": "postgresql://user:newpass@db.kejaksaan.go.id/myapp",
      "api_key": "sk_live_51H...",
      "encryption_key": "base64-encoded-key...",
      "smtp_password": "newsecret456"
    }
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

**Response:**

```json
{
  "success": true,
  "data": {
    "version": 2,
    "created_at": "2025-11-10T11:00:00Z"
  }
}
```

#### 8.1.4 Delete Secret

```bash
# Soft delete (can be restored)
curl -X DELETE "$SECRETON_URL/v1/secret/data/applications/myapp/config" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  --cacert /etc/secreton/secreton-cert.pem
```

#### 8.1.5 List Secrets

```bash
# List all secrets under path
curl "$SECRETON_URL/v1/secret?prefix=applications/" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "path": "applications/myapp/config",
      "version": 2,
      "created_at": "2025-11-10T10:00:00Z",
      "updated_at": "2025-11-10T11:00:00Z"
    },
    {
      "path": "applications/otherapp/credentials",
      "version": 1,
      "created_at": "2025-11-10T09:00:00Z",
      "updated_at": "2025-11-10T09:00:00Z"
    }
  ]
}
```

### 8.2 Encryption Operations

#### 8.2.1 Encrypt Data

```bash
# Encrypt plaintext
curl -X POST "$SECRETON_URL/v1/transit/encrypt/my-encryption-key" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "plaintext": "SGVsbG8gU2VjcmV0b24gLSBLZWpha3NhYW4gQWd1bmcgUkk="
  }' \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

**Response:**

```json
{
  "success": true,
  "data": {
    "ciphertext": "vault:v1:8SDd3WHDOjf7mq69CyCqYjBXAiQQAVZRkFM62EM6kLqK...",
    "key_version": 1
  }
}
```

#### 8.2.2 Decrypt Data

```bash
# Decrypt ciphertext
curl -X POST "$SECRETON_URL/v1/transit/decrypt/my-encryption-key" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "ciphertext": "vault:v1:8SDd3WHDOjf7mq69CyCqYjBXAiQQAVZRkFM62EM6kLqK..."
  }' \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

**Response:**

```json
{
  "success": true,
  "data": {
    "plaintext": "SGVsbG8gU2VjcmV0b24gLSBLZWpha3NhYW4gQWd1bmcgUkk="
  }
}
```

#### 8.2.3 Bulk Encryption

```bash
# Encrypt multiple items
curl -X POST "$SECRETON_URL/v1/transit/batch/encrypt" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "key_name": "my-encryption-key",
    "operations": [
      {
        "id": "op1",
        "data": "cGxhaW50ZXh0LTE=",
        "context": "YXV0aF9wYXJhbQ=="
      },
      {
        "id": "op2",
        "data": "cGxhaW50ZXh0LTI=",
        "context": "cGFyYW1fYXV0aA=="
      }
    ]
  }' \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

**Response:**

```json
{
  "success": true,
  "data": {
    "results": [
      {
        "id": "op1",
        "success": true,
        "ciphertext": "vault:v1:..."
      },
      {
        "id": "op2",
        "success": true,
        "ciphertext": "vault:v1:..."
      }
    ]
  }
}
```

### 8.3 Key Management

#### 8.3.1 Create Encryption Key

```bash
# Create new encryption key
curl -X POST "$SECRETON_URL/v1/keys" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "app-data-encryption-key",
    "type": "aes256-gcm96",
    "metadata": {
      "purpose": "application data encryption",
      "environment": "production"
    }
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

#### 8.3.2 Rotate Key

```bash
# Rotate encryption key (creates new version)
curl -X POST "$SECRETON_URL/v1/keys/app-data-encryption-key/rotate" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  --cacert /etc/secreton/secreton-cert.pem
```

#### 8.3.3 List Keys

```bash
# Get all encryption keys
curl "$SECRETON_URL/v1/keys" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

---

## 9. Advanced Features

### 9.1 Storage Backend Configuration

#### 9.1.1 File Backend (Default)

**Use Case:** Development, single-node, edge deployments

**Configuration:**

```toml
[storage]
backend = "file"

[storage.file]
path = "/var/lib/secreton/data"
sync_writes = true
permissions = 0o600
dir_permissions = 0o700
```

**Pros:**

- ✅ Zero dependencies
- ✅ Simple setup
- ✅ Fast local access
- ✅ No network latency

**Cons:**

- ❌ No HA support
- ❌ Single point of failure
- ❌ Manual backup required

#### 9.1.2 Consul Backend (High Availability)

**Use Case:** Production multi-node, distributed systems, multi-datacenter

**Prerequisites:**

```bash
# Install Consul
wget https://releases.hashicorp.com/consul/1.17.0/consul_1.17.0_linux_amd64.zip
unzip consul_1.17.0_linux_amd64.zip
sudo mv consul /usr/local/bin/

# Start Consul server (dev mode)
consul agent -dev

# Or start production cluster
consul agent -server -bootstrap-expect=3 \
  -data-dir=/var/consul/data \
  -bind=10.0.1.10 \
  -client=0.0.0.0
```

**Secreton Configuration:**

```toml
[storage]
backend = "consul"

[storage.consul]
address = "http://consul.kejaksaan.go.id:8500"
path = "secreton/"
token = "your-consul-acl-token"
scheme = "https"
tls_skip_verify = false
tls_ca_file = "/etc/consul/ca.pem"
tls_cert_file = "/etc/consul/client-cert.pem"
tls_key_file = "/etc/consul/client-key.pem"
```

**Environment Variables:**

```bash
export CONSUL_HTTP_ADDR=http://consul.kejaksaan.go.id:8500
export CONSUL_HTTP_TOKEN=your-consul-acl-token
export CONSUL_HTTP_SSL=true
```

**Verify Consul Connection:**

```bash
# Check Consul health
curl http://consul.kejaksaan.go.id:8500/v1/agent/self

# List Secreton keys
curl http://consul.kejaksaan.go.id:8500/v1/kv/secreton/?keys
```

**Pros:**

- ✅ High availability
- ✅ Automatic leader election
- ✅ Multi-datacenter support
- ✅ Built-in health checking
- ✅ Service discovery

**Cons:**

- ❌ Additional infrastructure
- ❌ Network dependency
- ❌ Consul maintenance overhead

#### 9.1.3 Raft Backend (Integrated HA)

**Use Case:** Production HA without external dependencies

**Configuration:**

```toml
[storage]
backend = "raft"

[storage.raft]
node_id = "secreton-node-1"
path = "/var/lib/secreton/raft"
bind_address = "10.0.1.10:8201"

# Cluster members
[[storage.raft.peers]]
node_id = "secreton-node-2"
address = "10.0.1.11:8201"

[[storage.raft.peers]]
node_id = "secreton-node-3"
address = "10.0.1.12:8201"
```

**Initialize Cluster:**

```bash
# Node 1 (leader)
./api_server --config /etc/secreton/config.toml

# Node 2
./api_server --config /etc/secreton/config-node2.toml --join 10.0.1.10:8201

# Node 3
./api_server --config /etc/secreton/config-node3.toml --join 10.0.1.10:8201
```

**Check Raft Status:**

```bash
curl https://secreton-node-1:8443/v1/sys/storage/raft/configuration | jq '.'
```

**Pros:**

- ✅ Integrated HA (no external dependencies)
- ✅ Strong consistency
- ✅ Automatic failover
- ✅ Simpler than Consul

**Cons:**

- ❌ Requires 3+ nodes
- ❌ More complex than file backend
- ❌ Network partition handling

#### 9.1.4 PostgreSQL Backend (Legacy)

**Use Case:** Existing PostgreSQL infrastructure

**Prerequisites:**

```bash
# Install PostgreSQL
sudo apt-get install -y postgresql postgresql-contrib

# Create database
sudo -u postgres psql
CREATE DATABASE secreton_db;
CREATE USER secreton WITH ENCRYPTED PASSWORD 'secure-password';
GRANT ALL PRIVILEGES ON DATABASE secreton_db TO secreton;
\q
```

**Configuration:**

```toml
[storage]
backend = "postgres"

[storage.postgres]
connection_url = "postgresql://secreton:secure-password@localhost/secreton_db"
table_name = "vault_kv_store"
max_connections = 10
```

**Environment Variable:**

```bash
export DATABASE_URL="postgresql://secreton:secure-password@localhost/secreton_db"
```

**Pros:**

- ✅ Familiar database technology
- ✅ ACID transactions
- ✅ Backup tools available

**Cons:**

- ❌ Database maintenance overhead
- ❌ Performance overhead vs. file/Consul
- ❌ Single point of failure (without replication)

### 9.2 Tokenization & Data Masking

#### 9.2.1 Format-Preserving Encryption (FPE)

```bash
# Tokenize credit card number (maintains format)
curl -X POST "$SECRETON_URL/v1/transform/encode/credit-card" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": "4532-1234-5678-9010",
    "transformation": "fpe"
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

**Response:**

```json
{
  "success": true,
  "data": {
    "encoded_value": "8471-6892-3451-7623",
    "token_id": "tok_AbCdEf123456"
  }
}
```

#### 9.2.2 Tokenization

```bash
# Generate token for sensitive data
curl -X POST "$SECRETON_URL/v1/transform/tokenize" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": "john.doe@example.com",
    "template": "email"
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

**Detokenize:**

```bash
curl -X POST "$SECRETON_URL/v1/transform/detokenize" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "token": "tok_AbCdEf123456"
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

#### 9.2.3 Data Masking

```bash
# Mask phone number (show last 4 digits)
curl -X POST "$SECRETON_URL/v1/transform/mask" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "value": "0812-3456-7890",
    "template": "****-****-{4}"
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

**Response:**

```json
{
  "success": true,
  "data": {
    "masked_value": "****-****-7890"
  }
}
```

### 9.3 TOTP/MFA Operations

#### 9.3.1 Generate TOTP Secret

```bash
# Create TOTP secret for user
curl -X POST "$SECRETON_URL/v1/mfa/totp/keys" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user@kejaksaan.go.id",
    "issuer": "Kejaksaan Agung RI",
    "account_name": "user@kejaksaan.go.id"
  }' \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

**Response:**

```json
{
  "success": true,
  "data": {
    "secret": "JBSWY3DPEHPK3PXP",
    "qr_code": "data:image/png;base64,iVBORw0KG...",
    "url": "otpauth://totp/Kejaksaan%20Agung%20RI:user@kejaksaan.go.id?secret=JBSWY3DPEHPK3PXP&issuer=Kejaksaan%20Agung%20RI"
  }
}
```

#### 9.3.2 Verify TOTP Code

```bash
# Verify user's TOTP code
curl -X POST "$SECRETON_URL/v1/mfa/totp/verify" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user@kejaksaan.go.id",
    "code": "123456"
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

**Response:**

```json
{
  "success": true,
  "data": {
    "valid": true
  }
}
```

### 9.4 Response Wrapping (Zero-Knowledge)

```bash
# Wrap secret with one-time token
curl -X POST "$SECRETON_URL/v1/sys/wrapping/wrap" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "password": "super-secret-password"
    },
    "ttl": 300
  }' \
  --cacert /etc/secreton/secreton-cert.pem
```

**Response:**

```json
{
  "success": true,
  "data": {
    "token": "hvs.WRAP.8SDd3WHDOjf7mq69CyCqYjBXAi...",
    "ttl": 300,
    "creation_time": "2025-11-10T10:00:00Z",
    "accessor": "hvs.ACC.xyz123"
  }
}
```

**Unwrap (One-Time Use):**

```bash
# Use token to retrieve secret (destroys token)
curl -X POST "$SECRETON_URL/v1/sys/wrapping/unwrap" \
  -H "X-Vault-Token: hvs.WRAP.8SDd3WHDOjf7mq69CyCqYjBXAi..." \
  --cacert /etc/secreton/secreton-cert.pem
```

---

## 10. Monitoring & Maintenance

### 10.1 Health Checks

#### 10.1.1 Basic Health Check

```bash
# Simple health endpoint
curl https://secreton.kejaksaan.go.id:8443/health
```

**Response (Healthy):**

```json
{
  "status": "healthy",
  "timestamp": "2025-11-10T10:00:00Z",
  "version": "1.0.0",
  "sealed": false
}
```

**Response (Sealed):**

```json
{
  "status": "sealed",
  "timestamp": "2025-11-10T10:00:00Z",
  "version": "1.0.0",
  "sealed": true
}
```

#### 10.1.2 Detailed System Status

```bash
# Comprehensive status
curl https://secreton.kejaksaan.go.id:8443/v1/sys/health \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  --cacert /etc/secreton/secreton-cert.pem | jq '.'
```

**Response:**

```json
{
  "initialized": true,
  "sealed": false,
  "standby": false,
  "performance_standby": false,
  "replication_performance_mode": "disabled",
  "replication_dr_mode": "disabled",
  "server_time_utc": 1699617600,
  "version": "1.0.0",
  "cluster_name": "secreton-cluster",
  "cluster_id": "d2c1e3f4-5a6b-7c8d-9e0f-1a2b3c4d5e6f"
}
```

#### 10.1.3 Storage Backend Health

```bash
# Check storage backend status
curl https://secreton.kejaksaan.go.id:8443/v1/sys/storage/health \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  --cacert /etc/secreton/secreton-cert.pem
```

### 10.2 Metrics & Monitoring

#### 10.2.1 Prometheus Metrics

```bash
# Expose metrics endpoint
curl https://secreton.kejaksaan.go.id:9090/metrics
```

**Sample Metrics:**

```
# HELP secreton_requests_total Total number of requests
# TYPE secreton_requests_total counter
secreton_requests_total{method="GET",path="/v1/secret/data/*",status="200"} 1234

# HELP secreton_request_duration_seconds Request duration in seconds
# TYPE secreton_request_duration_seconds histogram
secreton_request_duration_seconds_bucket{le="0.005"} 100
secreton_request_duration_seconds_bucket{le="0.01"} 200
secreton_request_duration_seconds_bucket{le="0.05"} 450

# HELP secreton_seal_status Vault seal status (0=unsealed, 1=sealed)
# TYPE secreton_seal_status gauge
secreton_seal_status 0

# HELP secreton_storage_operations_total Storage operations count
# TYPE secreton_storage_operations_total counter
secreton_storage_operations_total{operation="get",backend="file"} 5678
secreton_storage_operations_total{operation="put",backend="file"} 2345
```

#### 10.2.2 Prometheus Configuration

**prometheus.yml:**

```yaml
scrape_configs:
  - job_name: "secreton"
    static_configs:
      - targets: ["secreton.kejaksaan.go.id:9090"]
    scrape_interval: 15s
    metrics_path: "/metrics"
    scheme: https
    tls_config:
      ca_file: /etc/prometheus/secreton-ca.pem
      cert_file: /etc/prometheus/prometheus-cert.pem
      key_file: /etc/prometheus/prometheus-key.pem
```

#### 10.2.3 Grafana Dashboard

**Import Dashboard JSON:**

```json
{
  "dashboard": {
    "title": "Secreton Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(secreton_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Request Duration (p95)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, secreton_request_duration_seconds)"
          }
        ]
      },
      {
        "title": "Seal Status",
        "targets": [
          {
            "expr": "secreton_seal_status"
          }
        ]
      }
    ]
  }
}
```

### 10.3 Logging

#### 10.3.1 View Logs

```bash
# Systemd journal
sudo journalctl -u secreton -f

# Log file
tail -f /var/log/secreton/secreton.log

# Filter by level
jq 'select(.level == "ERROR")' /var/log/secreton/secreton.log
```

#### 10.3.2 Log Rotation

**/etc/logrotate.d/secreton:**

```
/var/log/secreton/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    create 0640 secreton secreton
    sharedscripts
    postrotate
        systemctl reload secreton > /dev/null 2>&1 || true
    endscript
}
```

### 10.4 Backup & Recovery

#### 10.4.1 Backup Vault Data

**File Backend:**

```bash
#!/bin/bash
# Backup script for file backend

BACKUP_DIR="/backup/secreton"
DATA_DIR="/var/lib/secreton/data"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Stop Secreton (optional, untuk consistency)
# sudo systemctl stop secreton

# Create backup
tar -czf "$BACKUP_DIR/secreton-data-$TIMESTAMP.tar.gz" \
  -C "$(dirname $DATA_DIR)" "$(basename $DATA_DIR)"

# Encrypt backup
gpg --symmetric --cipher-algo AES256 \
  "$BACKUP_DIR/secreton-data-$TIMESTAMP.tar.gz"

# Remove unencrypted backup
rm "$BACKUP_DIR/secreton-data-$TIMESTAMP.tar.gz"

# Start Secreton
# sudo systemctl start secreton

# Keep last 30 days
find "$BACKUP_DIR" -name "secreton-data-*.tar.gz.gpg" -mtime +30 -delete

echo "✅ Backup completed: secreton-data-$TIMESTAMP.tar.gz.gpg"
```

**Consul Backend:**

```bash
# Backup Consul snapshot
consul snapshot save /backup/consul/consul-snapshot-$(date +%Y%m%d).snap

# Encrypt
gpg --symmetric --cipher-algo AES256 \
  /backup/consul/consul-snapshot-$(date +%Y%m%d).snap
```

#### 10.4.2 Restore from Backup

**File Backend:**

```bash
#!/bin/bash
# Restore script

BACKUP_FILE="/backup/secreton/secreton-data-20251110-100000.tar.gz.gpg"
DATA_DIR="/var/lib/secreton/data"

# Stop Secreton
sudo systemctl stop secreton

# Decrypt backup
gpg --decrypt "$BACKUP_FILE" > /tmp/secreton-restore.tar.gz

# Restore data
sudo rm -rf "$DATA_DIR"
sudo tar -xzf /tmp/secreton-restore.tar.gz -C "$(dirname $DATA_DIR)"

# Set permissions
sudo chown -R secreton:secreton "$DATA_DIR"
sudo chmod 700 "$DATA_DIR"

# Clean temp
rm /tmp/secreton-restore.tar.gz

# Start Secreton
sudo systemctl start secreton

echo "✅ Restore completed"
```

**Consul Backend:**

```bash
# Decrypt snapshot
gpg --decrypt /backup/consul/consul-snapshot-20251110.snap.gpg \
  > /tmp/consul-snapshot.snap

# Restore snapshot
consul snapshot restore /tmp/consul-snapshot.snap

# Clean temp
rm /tmp/consul-snapshot.snap
```

#### 10.4.3 Disaster Recovery

**Scenario: Complete data loss**

```bash
# 1. Restore storage backend data (dari backup)
./restore-backup.sh

# 2. Start Secreton
sudo systemctl start secreton

# 3. Check seal status
curl https://secreton.kejaksaan.go.id:8443/v1/sys/seal-status

# 4. Unseal dengan Shamir shares
# (Koordinasi dengan operators untuk provide shares)
curl -X POST https://secreton.kejaksaan.go.id:8443/v1/sys/unseal \
  -d '{"key": "share-1..."}'
# ... ulangi sampai threshold terpenuhi

# 5. Verify operations
curl https://secreton.kejaksaan.go.id:8443/v1/sys/health \
  -H "Authorization: Bearer $ROOT_TOKEN"
```

---

## 11. Troubleshooting

### 11.1 Common Issues

#### Issue 1: Vault Sealed After Restart

**Symptoms:**

```
HTTP 503 Service Unavailable
{"error": "Vault is sealed"}
```

**Solution:**

```bash
# Check seal status
curl https://secreton.kejaksaan.go.id:8443/v1/sys/seal-status

# Unseal dengan threshold shares
# Koordinasi dengan operators
```

#### Issue 2: Storage Backend Connection Failed

**Symptoms:**

```
ERROR Failed to connect to storage backend: Connection refused
```

**Solutions:**

**For Consul:**

```bash
# Check Consul service
systemctl status consul

# Test connection
curl http://consul.kejaksaan.go.id:8500/v1/agent/self

# Check Secreton logs
journalctl -u secreton | grep consul
```

**For PostgreSQL:**

```bash
# Check PostgreSQL service
systemctl status postgresql

# Test connection
psql -h localhost -U secreton -d secreton_db -c "SELECT 1;"

# Check connection string
echo $DATABASE_URL
```

#### Issue 3: TLS Certificate Errors

**Symptoms:**

```
x509: certificate signed by unknown authority
```

**Solutions:**

```bash
# Check certificate validity
openssl x509 -in /etc/secreton/secreton-cert.pem -text -noout

# Verify certificate chain
openssl verify -CAfile /etc/secreton/ca.pem \
  /etc/secreton/secreton-cert.pem

# Test TLS connection
openssl s_client -connect secreton.kejaksaan.go.id:8443 \
  -CAfile /etc/secreton/ca.pem
```

#### Issue 4: High Memory Usage

**Symptoms:**

```
OOMKilled by systemd
Memory usage constantly increasing
```

**Solutions:**

```bash
# Check memory stats
ps aux | grep api_server

# Set memory limits in systemd
# Edit /etc/systemd/system/secreton.service
[Service]
MemoryLimit=2G
MemoryMax=4G

# Reload and restart
sudo systemctl daemon-reload
sudo systemctl restart secreton

# Monitor memory
watch -n 1 'ps -o rss= -p $(pgrep api_server)'
```

#### Issue 5: Permission Denied on Storage

**Symptoms:**

```
ERROR Failed to write to storage: Permission denied
```

**Solutions:**

```bash
# Check directory permissions
ls -la /var/lib/secreton/data

# Fix ownership
sudo chown -R secreton:secreton /var/lib/secreton

# Fix permissions
sudo chmod 700 /var/lib/secreton/data
sudo chmod 600 /var/lib/secreton/data/*

# Verify user
sudo -u secreton ls /var/lib/secreton/data
```

### 11.2 Performance Tuning

#### 11.2.1 HTTP Server Tuning

```toml
[http]
bind_address = "0.0.0.0:8443"
timeout_secs = 60  # Increase for slow operations
max_body_size = 52428800  # 50 MB for large secrets
keep_alive_secs = 120  # Longer keep-alive
compression = true  # Enable gzip
```

#### 11.2.2 Storage Backend Optimization

**File Backend:**

```toml
[storage.file]
sync_writes = false  # Disable untuk performance (trade-off: durability)
```

**Consul Backend:**

```toml
[storage.consul]
max_parallel = 128  # Increase parallelism
consistency_mode = "default"  # Options: default, consistent, stale
```

#### 11.2.3 System Limits

**/etc/security/limits.conf:**

```
secreton soft nofile 65536
secreton hard nofile 65536
secreton soft nproc 4096
secreton hard nproc 4096
```

**/etc/sysctl.conf:**

```
# Network tuning
net.core.somaxconn = 4096
net.ipv4.tcp_max_syn_backlog = 8192
net.ipv4.ip_local_port_range = 10000 65535

# File descriptors
fs.file-max = 2097152
```

Apply:

```bash
sudo sysctl -p
```

### 11.3 Debug Mode

```bash
# Enable debug logging
export RUST_LOG=secreton=debug,secreton_core=debug,secreton_api=debug

# Start with debug
./api_server --log-level debug

# Or via environment
SECRETON_LOG_LEVEL=debug systemctl restart secreton
```

### 11.4 Getting Help

**Internal Resources:**

- Documentation: `/srv/proyek/simpelv2/layanan/secreton/docs/`
- Source code: `/srv/proyek/simpelv2/layanan/secreton/`
- Issue tracker: GitHub Issues

**External Resources:**

- Rust Crypto: https://github.com/RustCrypto
- NIST Post-Quantum: https://csrc.nist.gov/projects/post-quantum-cryptography

---

## 12. Security Best Practices

### 12.1 Shamir Shares Management

✅ **DO:**

- Store shares in different physical locations
- Use hardware security modules (HSM) for production
- Implement split-knowledge principle (no single person knows all shares)
- Encrypt shares when storing digitally
- Regular audit of who has access to shares
- Test unseal process quarterly

❌ **DON'T:**

- Store all shares in same location
- Email shares in plaintext
- Store shares in code repositories
- Share shares via insecure channels
- Allow single person to hold threshold shares

### 12.2 Root Token Management

✅ **DO:**

- Revoke root token after initial setup
- Use short-lived tokens for operations
- Rotate root token periodically
- Store in hardware security module
- Enable audit logging for root token usage

❌ **DON'T:**

- Use root token for daily operations
- Share root token with multiple users
- Store root token in plaintext files
- Commit root token to version control

### 12.3 Network Security

✅ **DO:**

- Always use TLS in production
- Enable mTLS for service-to-service communication
- Implement IP whitelisting
- Use firewall rules to restrict access
- Enable rate limiting
- Monitor for suspicious traffic patterns

❌ **DON'T:**

- Expose Secreton directly to public internet
- Use self-signed certificates in production (use proper CA)
- Disable TLS verification
- Allow weak cipher suites

### 12.4 Audit & Compliance

✅ **DO:**

- Enable comprehensive audit logging
- Store audit logs in tamper-proof storage
- Regularly review audit logs
- Implement log retention policies
- Set up alerting for suspicious activities
- Conduct regular security audits

❌ **DON'T:**

- Disable audit logging
- Store audit logs on same server as Secreton
- Ignore audit log alerts
- Delete audit logs prematurely

### 12.5 Access Control

✅ **DO:**

- Implement principle of least privilege
- Use namespaces for multi-tenancy
- Create role-based policies
- Regularly review and revoke unused tokens
- Implement token TTL limits
- Enable MFA for sensitive operations

❌ **DON'T:**

- Grant admin access to all users
- Use same token for multiple applications
- Set infinite token TTLs
- Skip authentication in development

### 12.6 Backup & Disaster Recovery

✅ **DO:**

- Automate daily backups
- Encrypt all backups
- Store backups in multiple locations
- Test restore procedures regularly
- Document disaster recovery plan
- Keep backup of Shamir shares offline

❌ **DON'T:**

- Store backups unencrypted
- Keep only single backup copy
- Skip testing restore procedures
- Store backups on same infrastructure

### 12.7 Post-Quantum Readiness

✅ **DO:**

- Enable hybrid encryption (classical + post-quantum)
- Use ML-KEM for key encapsulation
- Use ML-DSA for digital signatures
- Plan migration path to quantum-safe algorithms
- Monitor NIST post-quantum standardization

**Enable Post-Quantum:**

```toml
[crypto]
enable_post_quantum = true
pq_algorithm = "mlkem768"  # NIST Level 3
hybrid_mode = true  # Classical + PQ
```

---

## 13. Quick Reference

### 13.1 Common Commands

```bash
# Server Management
systemctl start secreton
systemctl stop secreton
systemctl restart secreton
systemctl status secreton
journalctl -u secreton -f

# Seal/Unseal
curl POST /v1/sys/seal        # Seal vault
curl POST /v1/sys/unseal      # Unseal vault
curl GET  /v1/sys/seal-status # Check status

# Secret Operations
curl POST /v1/secret/data/{path}     # Create secret
curl GET  /v1/secret/data/{path}     # Read secret
curl PUT  /v1/secret/data/{path}     # Update secret
curl DEL  /v1/secret/data/{path}     # Delete secret

# Encryption
curl POST /v1/transit/encrypt/{key}  # Encrypt
curl POST /v1/transit/decrypt/{key}  # Decrypt

# Health & Monitoring
curl GET /health                     # Basic health
curl GET /v1/sys/health              # Detailed health
curl GET /metrics                    # Prometheus metrics
```

### 13.2 Environment Variables

```bash
# Core
SECRETON_STORAGE_BACKEND=file|consul|raft|postgres
SECRETON_HTTP_BIND=0.0.0.0:8443
SECRETON_LOG_LEVEL=info|debug|warn|error

# TLS
SECRETON_TLS_ENABLED=true|false
SECRETON_TLS_CERT=/path/to/cert.pem
SECRETON_TLS_KEY=/path/to/key.pem

# Storage
DATABASE_URL=postgresql://...       # PostgreSQL
CONSUL_HTTP_ADDR=http://...         # Consul
CONSUL_HTTP_TOKEN=token             # Consul token

# Authentication
SECRETON_JWT_SECRET=secret-key

# Monitoring
SECRETON_METRICS_ENABLED=true|false
SECRETON_METRICS_PORT=9090
```

### 13.3 File Locations

```
/opt/secreton/bin/api_server          # Binary
/etc/secreton/config.toml             # Configuration
/etc/secreton/secreton.env            # Environment vars
/etc/secreton/secreton-cert.pem       # TLS certificate
/etc/secreton/secreton-key.pem        # TLS private key
/var/lib/secreton/data/               # Storage (file backend)
/var/log/secreton/secreton.log        # Log file
/etc/systemd/system/secreton.service  # Systemd unit
```

### 13.4 Default Ports

```
8080  - HTTP (development)
8443  - HTTPS (production)
9090  - Metrics (Prometheus)
50051 - gRPC
```

---

## 14. Appendix

### 14.1 Architecture Comparison: Secreton vs Sandi Data

| Feature           | Sandi Data    | Secreton       | Notes                      |
| ----------------- | ------------- | -------------- | -------------------------- |
| Key Hierarchy     | MK→KEK→SK→DEK | ✅ Implemented | Full 4-layer hierarchy     |
| Bulk Encryption   | ✅            | ✅             | With AAD support           |
| TOTP/MFA          | ✅            | ✅             | RFC 6238 compliant         |
| Tokenization      | ✅            | ✅             | FPE, tokenization, masking |
| HSM Support       | ✅            | ✅             | TPM 2.0 integration        |
| Post-Quantum      | ❌            | ✅             | ML-KEM, ML-DSA             |
| High Availability | ❌            | ✅             | Consul, Raft               |
| Response Wrapping | ❌            | ✅             | Zero-knowledge delivery    |

### 14.2 Performance Benchmarks

**Hardware:** 4 CPU cores, 8GB RAM, SSD storage

| Operation                | Throughput   | Latency (p95) |
| ------------------------ | ------------ | ------------- |
| Secret Read              | 5,000 req/s  | 15ms          |
| Secret Write             | 2,500 req/s  | 25ms          |
| Encryption (AES-GCM)     | 10,000 req/s | 8ms           |
| Decryption (AES-GCM)     | 12,000 req/s | 6ms           |
| TOTP Generation          | 15,000 req/s | 5ms           |
| Bulk Encrypt (100 items) | 150 batch/s  | 450ms         |

### 14.3 Glossary

- **Seal/Unseal**: Process of protecting/accessing master key
- **Shamir Secret Sharing**: Algorithm untuk split key menjadi N shares
- **Threshold**: Minimum shares needed untuk reconstruct key
- **Master Key**: Root encryption key untuk encrypt KEKs
- **KEK**: Key Encryption Key untuk encrypt DEKs
- **DEK**: Data Encryption Key untuk encrypt actual data
- **Transit Engine**: Encryption-as-a-service untuk aplikasi
- **KV Engine**: Key-value secret storage dengan versioning
- **AAD**: Additional Authenticated Data untuk AEAD encryption
- **FPE**: Format-Preserving Encryption (maintains data format)
- **KMIP**: Key Management Interoperability Protocol
- **mTLS**: Mutual TLS (two-way authentication)
- **HSM**: Hardware Security Module
- **TPM**: Trusted Platform Module

---

## 15. Getting Support

### Internal Support

**Email:** tim-keamanan@kejaksaan.go.id
**Slack:** #secreton-support
**Documentation:** `/srv/proyek/simpelv2/layanan/secreton/docs/`

### Issue Reporting

**GitHub:** https://github.com/analisaperlengkapan/simpel2/issues
**Template:**

```markdown
**Environment:**

- Secreton Version: x.x.x
- OS: Ubuntu 22.04
- Storage Backend: file/consul/raft
- Deployment: development/production

**Issue Description:**
[Clear description of the problem]

**Steps to Reproduce:**

1. ...
2. ...
3. ...

**Expected Behavior:**
[What should happen]

**Actual Behavior:**
[What actually happens]

**Logs:**
```

[Paste relevant logs]

```

**Configuration:**
[Relevant configuration (redact secrets!)]
```

---

**Document Version:** 1.0
**Last Updated:** November 10, 2025
**Maintained By:** Tim Keamanan IT - Kejaksaan Agung RI

---

**© 2025 Kejaksaan Agung Republik Indonesia. All rights reserved.**
