# Deployment Guide - Secure Configuration System

**Version:** 1.0
**Prerequisites:** Secreton with secure config system implemented

---

## Quick Deployment Checklist

- [ ] Copy `secreton.toml.example` to `secreton.toml`
- [ ] Configure storage backend (Raft recommended)
- [ ] Configure TLS certificates (production)
- [ ] Initialize vault and save Shamir shares
- [ ] Distribute shares to operators securely
- [ ] (Optional) Migrate from legacy config
- [ ] Test unseal process
- [ ] Configure systemd/supervisor
- [ ] Set up monitoring

---

## 1. Development Deployment

### Setup

```bash
# 1. Copy example config
cd /srv/proyek/simpelv2/infra/secreton
cp secreton.toml.example secreton.toml

# 2. Edit for local development
vim secreton.toml
```

**Minimal dev config:**
```toml
[storage]
backend = "file"
path = "./data/secreton"

[listener.http]
address = "127.0.0.1:8200"

[seal]
type = "shamir"
shares = 3
threshold = 2
```

### Initialize

```bash
# Build
cargo build --release

# Initialize vault (generates Shamir shares)
./target/release/secreton init --shares 3 --threshold 2 --output keys.json

# IMPORTANT: Save keys.json securely!
```

### Start & Unseal

```bash
# Start server (in background or separate terminal)
./target/release/secreton server &

# Unseal (need 2 of 3 shares)
./target/release/secreton unseal  # Enter share 1
./target/release/secreton unseal  # Enter share 2

# Verify
./target/release/secreton status
```

---

## 2. Production Deployment (Single Node)

### Prerequisites

```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y openssl

# Create secreton user
sudo useradd -r -s /bin/false secreton

# Create directories
sudo mkdir -p /var/lib/secreton/{raft,backups}
sudo mkdir -p /etc/secreton/tls
sudo mkdir -p /var/log/secreton
sudo chown -R secreton:secreton /var/lib/secreton /var/log/secreton
```

### TLS Certificates

```bash
# Generate self-signed cert (or use Let's Encrypt)
sudo openssl req -x509 -newkey rsa:4096 \
  -keyout /etc/secreton/tls/key.pem \
  -out /etc/secreton/tls/cert.pem \
  -days 365 -nodes \
  -subj "/CN=secreton.yourdomain.com"

sudo chown secreton:secreton /etc/secreton/tls/*.pem
sudo chmod 600 /etc/secreton/tls/key.pem
```

### Configuration

**/etc/secreton/secreton.toml:**
```toml
[storage]
backend = "raft"
path = "/var/lib/secreton/raft"
node_id = "secreton-prod-01"

[listener.http]
address = "0.0.0.0:8200"
tls_enabled = true

[listener.http.tls]
cert_file = "/etc/secreton/tls/cert.pem"
key_file = "/etc/secreton/tls/key.pem"
min_version = "1.3"

[seal]
type = "shamir"
shares = 5
threshold = 3

[telemetry]
prometheus_enabled = true
metrics_path = "/metrics"

log_level = "info"
log_format = "json"
```

### Systemd Service

**/etc/systemd/system/secreton.service:**
```ini
[Unit]
Description=Secreton Vault
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=secreton
Group=secreton
ExecStart=/usr/local/bin/secreton server --config /etc/secreton/secreton.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/lib/secreton /var/log/secreton

[Install]
WantedBy=multi-user.target
```

### Deploy & Initialize

```bash
# 1. Copy binary
sudo cp target/release/secreton /usr/local/bin/
sudo chown root:root /usr/local/bin/secreton
sudo chmod 755 /usr/local/bin/secreton

# 2. Copy config
sudo cp secreton.toml /etc/secreton/
sudo chown secreton:secreton /etc/secreton/secreton.toml
sudo chmod 640 /etc/secreton/secreton.toml

# 3. Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable secreton
sudo systemctl start secreton

# 4. Initialize (as admin, not secreton user)
secreton init --shares 5 --threshold 3 --output ~/secreton-keys.json

# 5. CRITICAL: Distribute shares to 5 different operators
# Store each share in separate secure locations (password managers, HSMs, etc.)

# 6. Unseal (need 3 operators)
secreton unseal  # Operator 1
secreton unseal  # Operator 2
secreton unseal  # Operator 3

# 7. Verify
secreton status
curl -k https://localhost:8200/v1/sys/seal-status
```

---

## 3. Production HA Cluster (3 Nodes)

### Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  secreton-01    │◄──►│  secreton-02    │◄──►│  secreton-03    │
│  (Leader)       │    │  (Follower)     │    │  (Follower)     │
│  10.0.1.10:8200 │    │  10.0.1.11:8200 │    │  10.0.1.12:8200 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                      │                      │
         └──────────────────────┴──────────────────────┘
                         Raft Consensus
```

### Node 1 Configuration

**/etc/secreton/secreton.toml (node1):**
```toml
[storage]
backend = "raft"
path = "/var/lib/secreton/raft"
node_id = "secreton-01"

[[storage.raft.retry_join]]
leader_api_addr = "https://10.0.1.11:8200"
leader_ca_cert_file = "/etc/secreton/tls/ca.pem"

[[storage.raft.retry_join]]
leader_api_addr = "https://10.0.1.12:8200"
leader_ca_cert_file = "/etc/secreton/tls/ca.pem"

[listener.http]
address = "0.0.0.0:8200"
tls_enabled = true

[listener.http.tls]
cert_file = "/etc/secreton/tls/cert.pem"
key_file = "/etc/secreton/tls/key.pem"

[seal]
type = "shamir"
shares = 7
threshold = 4
```

### Deployment Steps

```bash
# On each node:

# 1. Deploy binary and config
sudo cp target/release/secreton /usr/local/bin/
sudo cp secreton.toml /etc/secreton/
sudo systemctl enable secreton
sudo systemctl start secreton

# 2. Initialize ONLY on node1 (first time)
# On node1:
secreton init --shares 7 --threshold 4

# 3. Unseal ALL nodes (need 4 operators on EACH node)
# On each node:
secreton unseal  # Operator 1
secreton unseal  # Operator 2
secreton unseal  # Operator 3
secreton unseal  # Operator 4

# 4. Verify cluster
secreton status
# Should show: "Cluster: 3 nodes, Leader: secreton-01"
```

---

## 4. Cloud KMS Auto-Unseal (AWS)

### Prerequisites

```bash
# IAM policy for KMS access
aws iam create-policy --policy-name SecretonKMSPolicy --policy-document '{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["kms:Decrypt", "kms:Encrypt", "kms:DescribeKey"],
    "Resource": "arn:aws:kms:us-east-1:123456789012:key/your-key-id"
  }]
}'

# Attach to EC2 instance role
aws iam attach-role-policy \
  --role-name SecretonInstanceRole \
  --policy-arn arn:aws:iam::123456789012:policy/SecretonKMSPolicy
```

### Configuration

```toml
[seal]
type = "aws-kms"

[seal.aws_kms]
region = "us-east-1"
kms_key_id = "arn:aws:kms:us-east-1:123456789012:key/your-key-id"
```

### Environment Variables

```bash
# If not using IAM role:
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
export AWS_REGION=us-east-1
```

**Benefits:**
- Auto-unseal on restart
- No manual operator intervention
- Centralized key management

---

## 5. Migration from Legacy Config

### Before Migration

```bash
# 1. Backup everything
sudo tar czf secreton-backup-$(date +%Y%m%d).tar.gz \
  /etc/secreton/config/ \
  /var/lib/secreton/

# 2. Ensure vault is unsealed
secreton status  # Should show "UNSEALED"
```

### Run Migration

```bash
# Migrate with automatic backup
secreton migrate --from /etc/secreton/config/

# Output:
# 📦 Step 1/4: Backing up legacy configuration...
# 🔄 Step 2/4: Migrating to encrypted storage...
# 🔍 Step 3/4: Verifying migration...
# ✅ Step 4/4: Migration Summary
```

### Verification

```bash
# 1. Restart service
sudo systemctl restart secreton

# 2. Unseal
secreton unseal

# 3. Test functionality
curl -k https://localhost:8200/v1/sys/seal-status

# 4. If everything works, cleanup
secreton migrate --cleanup
```

---

## 6. Monitoring & Maintenance

### Prometheus Metrics

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'secreton'
    scheme: https
    tls_config:
      insecure_skip_verify: true
    static_configs:
      - targets: ['localhost:8200']
    metrics_path: /metrics
```

### Health Checks

```bash
# Systemd health check
sudo systemctl status secreton

# Seal status
curl -k https://localhost:8200/v1/sys/seal-status | jq

# Raft cluster status (if HA)
curl -k https://localhost:8200/v1/sys/storage/raft/configuration | jq
```

### Log Rotation

**/etc/logrotate.d/secreton:**
```
/var/log/secreton/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0640 secreton secreton
    sharedscripts
    postrotate
        systemctl reload secreton > /dev/null 2>&1 || true
    endscript
}
```

---

## 7. Backup & Recovery

### Backup Raft Data

```bash
#!/bin/bash
# /usr/local/bin/backup-secreton.sh

BACKUP_DIR="/var/backups/secreton"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
sudo tar czf "$BACKUP_DIR/secreton-$DATE.tar.gz" \
  /var/lib/secreton/raft \
  /etc/secreton/secreton.toml

# Keep last 30 days
find "$BACKUP_DIR" -name "secreton-*.tar.gz" -mtime +30 -delete

# Cron: Daily at 2 AM
# 0 2 * * * /usr/local/bin/backup-secreton.sh
```

### Disaster Recovery

```bash
# 1. Stop service
sudo systemctl stop secreton

# 2. Restore data
sudo tar xzf secreton-backup.tar.gz -C /

# 3. Start service
sudo systemctl start secreton

# 4. Unseal
secreton unseal  # Need threshold operators
```

---

## 8. Security Checklist

### Production Security

- [x] TLS enabled with valid certificates
- [x] Firewall rules (only 8200 from trusted IPs)
- [x] Shamir shares distributed to separate operators
- [x] No shares stored on same server
- [x] Regular backups to secure location
- [x] Log monitoring and alerting
- [x] Audit logging enabled
- [x] systemd hardening enabled
- [x] File permissions restricted (640 for config)
- [x] Run as non-root user (secreton)

### Operator Checklist

- [ ] Each operator stores share in password manager or HSM
- [ ] Operators from different teams/departments
- [ ] Emergency unseal procedure documented
- [ ] Rekey process tested and documented
- [ ] Seal on security incidents procedure

---

## Troubleshooting

### Service won't start

```bash
# Check logs
sudo journalctl -u secreton -n 50 --no-pager

# Check config syntax
secreton validate-config /etc/secreton/secreton.toml

# Check permissions
ls -la /var/lib/secreton/
```

### Can't unseal

```bash
# Reset unseal progress
secreton unseal --reset

# Check seal status
secreton status

# Verify shares are correct (base64 encoded)
```

### Cluster issues

```bash
# Check Raft status on each node
curl -k https://node1:8200/v1/sys/storage/raft/configuration
curl -k https://node2:8200/v1/sys/storage/raft/configuration
curl -k https://node3:8200/v1/sys/storage/raft/configuration

# Remove failed node
secreton cluster remove-node <node-id>
```

---

## Support

- Logs: `/var/log/secreton/`
- Config: `/etc/secreton/secreton.toml`
- Data: `/var/lib/secreton/raft/`
- Documentation: `docs/`
