# Secreton-Authenc-Portal Integration Deployment Guide

## Overview

This document provides comprehensive deployment instructions for the integrated Secreton vault, Authenc IAM, and Portal microfrontend system.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Production Deployment                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │   Portal     │  OAuth2 │   Authenc    │                 │
│  │ (WASM/Leptos)│ ◄─────► │     IAM      │                 │
│  │  Port: 8080  │   JWT   │  Port: 8088  │                 │
│  └──────────────┘         └──────┬───────┘                 │
│         │                         │                          │
│         │                         │ Bearer Token             │
│         │                         ▼                          │
│         │                  ┌──────────────┐                 │
│         │                  │   Secreton   │                 │
│         └─────────────────►│    Vault     │                 │
│          (Admin UI)         │  Port: 8200  │                 │
│                             └──────────────┘                 │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

### System Requirements

- **OS**: Linux (Ubuntu 22.04+ recommended)
- **RAM**: 4GB minimum, 8GB recommended
- **CPU**: 2 cores minimum, 4 cores recommended
- **Disk**: 20GB minimum
- **Rust**: 1.70+ with wasm32-unknown-unknown target
- **Trunk**: Latest version for WASM builds
- **PostgreSQL**: 14+ (for production KVEngine storage)
- **Redis**: 6+ (for session management)

### Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib

# Install Redis
sudo apt install redis-server
```

## Configuration

### 1. Environment Variables

Create `.env` file in project root:

```bash
# Authenc Configuration
AUTHENC_API_URL=http://localhost:8088
AUTHENC_JWT_SECRET=<generate-secure-random-secret>
AUTHENC_JWT_EXPIRATION=3600
AUTHENC_ISSUER=simpelv2-authenc
AUTHENC_AUDIENCE=portal,secreton

# Secreton Configuration
SECRETON_API_URL=http://localhost:8200
SECRETON_TOKEN=<generate-secure-admin-token>
SECRETON_MAX_AUDIT_EVENTS=100000

# Database
DATABASE_URL=postgresql://authenc:password@localhost/authenc_db
SECRETON_DATABASE_URL=postgresql://secreton:password@localhost/secreton_db

# Redis
REDIS_URL=redis://localhost:6379

# Portal Configuration
PORTAL_URL=http://localhost:8080
PORTAL_API_TIMEOUT=30

# Security
CORS_ALLOWED_ORIGINS=http://localhost:8080,https://simpelv2.kejaksaan.go.id
ENABLE_RATE_LIMITING=true
RATE_LIMIT_REQUESTS_PER_MINUTE=100

# Monitoring
ENABLE_METRICS=true
PROMETHEUS_PORT=9090
LOG_LEVEL=info
```

### 2. Database Setup

```sql
-- Create authenc database
CREATE DATABASE authenc_db;
CREATE USER authenc WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE authenc_db TO authenc;

-- Create secreton database
CREATE DATABASE secreton_db;
CREATE USER secreton WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE secreton_db TO secreton;
```

### 3. Secrets Management

Generate secure secrets:

```bash
# Generate JWT secret (256-bit)
openssl rand -base64 32

# Generate admin token for secreton
openssl rand -hex 32
```

## Build Process

### Development Build

```bash
# Build all services
make build-dev

# Or individually:
cargo build --bin authenc
cargo build -p portal-microfrontend
cd infra/secreton && cargo build
```

### Production Build

```bash
# Build with optimizations
make build-prod

# Or individually:
cargo build --bin authenc --release
trunk build --release antarmuka/portal/index.html
cd infra/secreton && cargo build --release
```

### WASM Optimization

```bash
# Optimize WASM binaries
./scripts/wasm-optimizer.sh build

# This reduces bundle size by ~60%
```

## Deployment Steps

### 1. Deploy Authenc IAM

```bash
# Run database migrations
cd infra/authenc
sqlx migrate run

# Start service
./target/release/authenc --config config/production.toml

# Verify health
curl http://localhost:8088/health
```

### 2. Deploy Secreton Vault

```bash
# Initialize vault
cd infra/secreton
./target/release/secreton init

# Start service
./target/release/secreton server --config config/production.toml

# Verify health
curl http://localhost:8200/v1/sys/health
```

### 3. Deploy Portal Microfrontend

```bash
# Copy WASM artifacts to web server
cp -r antarmuka/portal/dist/* /var/www/simpelv2/portal/

# Configure nginx (see nginx.conf example below)
sudo systemctl reload nginx

# Verify
curl http://localhost:8080/
```

## Nginx Configuration

```nginx
# /etc/nginx/sites-available/simpelv2

# Portal frontend
server {
    listen 80;
    server_name simpelv2.kejaksaan.go.id;

    root /var/www/simpelv2/portal;
    index index.html;

    # WASM MIME types
    types {
        application/wasm wasm;
    }

    location / {
        try_files $uri $uri/ /index.html;
    }

    # Proxy to Authenc API
    location /api/auth/ {
        proxy_pass http://localhost:8088/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
}

# Authenc API (internal)
upstream authenc_backend {
    server localhost:8088;
}

# Secreton API (internal only)
upstream secreton_backend {
    server localhost:8200;
}
```

## systemd Service Files

### authenc.service

```ini
[Unit]
Description=Authenc IAM Service
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=simpelv2
Group=simpelv2
WorkingDirectory=/opt/simpelv2/authenc
Environment="RUST_LOG=info"
EnvironmentFile=/opt/simpelv2/.env
ExecStart=/opt/simpelv2/bin/authenc --config /opt/simpelv2/config/authenc.toml
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/simpelv2/data

[Install]
WantedBy=multi-user.target
```

### secreton.service

```ini
[Unit]
Description=Secreton Vault Service
After=network.target postgresql.service

[Service]
Type=simple
User=simpelv2
Group=simpelv2
WorkingDirectory=/opt/simpelv2/secreton
Environment="RUST_LOG=info"
EnvironmentFile=/opt/simpelv2/.env
ExecStart=/opt/simpelv2/bin/secreton server --config /opt/simpelv2/config/secreton.toml
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/simpelv2/vault

[Install]
WantedBy=multi-user.target
```

## Enable and Start Services

```bash
# Copy service files
sudo cp *.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable services
sudo systemctl enable authenc secreton

# Start services
sudo systemctl start authenc secreton

# Check status
sudo systemctl status authenc secreton
```

## Integration Testing

### 1. Test Secreton Health

```bash
curl http://localhost:8200/v1/sys/health
# Expected: 200 OK
```

### 2. Test Authenc Authentication

```bash
# Get token
curl -X POST http://localhost:8088/realms/simpel/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password&client_id=portal&username=admin&password=admin123"

# Expected: {"access_token": "...", "token_type": "Bearer", ...}
```

### 3. Test Vault Integration

```bash
# Store secret (requires token from step 2)
curl -X POST http://localhost:8200/v1/secret/data/test/api-key \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"data": "secret-value-123"}'

# Retrieve secret
curl http://localhost:8200/v1/secret/data/test/api-key \
  -H "Authorization: Bearer $TOKEN"
```

### 4. Test Portal Login

```bash
# Open browser
xdg-open http://localhost:8080

# Login with:
# Username: admin
# Password: admin123

# Verify JWT token in localStorage
```

## Monitoring

### Prometheus Metrics

```yaml
# /etc/prometheus/prometheus.yml
scrape_configs:
  - job_name: "authenc"
    static_configs:
      - targets: ["localhost:8088"]
    metrics_path: "/metrics"

  - job_name: "secreton"
    static_configs:
      - targets: ["localhost:8200"]
    metrics_path: "/metrics"
```

### Grafana Dashboards

Import dashboards from:

- `docs/monitoring/authenc-dashboard.json`
- `docs/monitoring/secreton-dashboard.json`

### Log Aggregation

```bash
# View authenc logs
sudo journalctl -u authenc -f

# View secreton logs
sudo journalctl -u secreton -f

# Aggregate with Loki (optional)
# See docs/monitoring/loki-config.yaml
```

## Security Hardening

### 1. Firewall Rules

```bash
# Allow only necessary ports
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# Block direct access to backend services
sudo ufw deny 8088/tcp
sudo ufw deny 8200/tcp
```

### 2. TLS Certificates

```bash
# Use Let's Encrypt
sudo certbot --nginx -d simpelv2.kejaksaan.go.id
```

### 3. Rate Limiting

Enable in Authenc config:

```toml
[security]
enable_rate_limiting = true
requests_per_minute = 100
burst_size = 20
```

### 4. Audit Log Retention

```toml
[audit]
enabled = true
max_events = 1000000
rotation_days = 90
export_to_syslog = true
```

## Backup and Recovery

### Database Backup

```bash
# Backup authenc database
pg_dump authenc_db > authenc_backup_$(date +%Y%m%d).sql

# Backup secreton database
pg_dump secreton_db > secreton_backup_$(date +%Y%m%d).sql

# Automated daily backups (cron)
0 2 * * * /opt/simpelv2/scripts/backup-databases.sh
```

### Vault Data Backup

```bash
# Backup vault storage
tar -czf vault_backup_$(date +%Y%m%d).tar.gz /opt/simpelv2/vault/
```

### Recovery

```bash
# Restore database
psql authenc_db < authenc_backup_20251007.sql

# Restore vault
tar -xzf vault_backup_20251007.tar.gz -C /
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs
sudo journalctl -u authenc -n 50 --no-pager

# Check port conflicts
sudo lsof -i :8088

# Verify configuration
./target/release/authenc --config config/production.toml --check
```

### Authentication Failures

```bash
# Verify JWT secret matches
grep JWT_SECRET /opt/simpelv2/.env

# Check token expiration
# Use jwt.io to decode token

# Verify database connectivity
psql -h localhost -U authenc authenc_db -c "SELECT version();"
```

### Vault Connection Issues

```bash
# Test vault connectivity
curl -v http://localhost:8200/v1/sys/health

# Check bearer token
echo $SECRETON_TOKEN

# Verify secreton logs
sudo journalctl -u secreton -f
```

## Performance Tuning

### Database Connection Pooling

```toml
[database]
max_connections = 100
min_connections = 10
connection_timeout = 30
```

### WASM Loading Optimization

```javascript
// Preload WASM module
const wasmPromise = import("./portal_microfrontend.js");
```

### Redis Caching

```toml
[cache]
enabled = true
ttl = 3600
max_entries = 10000
```

## Upgrade Path (Phase 3)

### PostgreSQL Storage for Secreton

```sql
CREATE TABLE secrets (
    id SERIAL PRIMARY KEY,
    path VARCHAR(255) NOT NULL,
    version INT NOT NULL,
    data BYTEA NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(path, version)
);

CREATE INDEX idx_secrets_path ON secrets(path);
CREATE INDEX idx_secrets_created ON secrets(created_at);
```

### OAuth2 Authorization Code Flow

See `docs/oauth2-authorization-code.md` for implementation guide.

## Support and Resources

- **Documentation**: `/docs/`
- **API Reference**: `/docs/api/`
- **Issue Tracker**: GitLab Issues
- **Contact**: simpelv2@kejaksaan.go.id

## Changelog

### v1.0.0 (2025-10-07)

- Initial integration release
- Secreton vault with in-memory KVEngine
- Authenc IAM with JWT authentication
- Portal OAuth2 password grant flow
- Audit logging for vault operations
- 409 automated tests passing

---

**Deployment completed successfully!**
