# Secreton Configuration System - Quick Reference

**Status:** ✅ Implemented
**Version:** 1.0

---

## Overview

Secreton uses a **Secret Vault-like secure configuration system** with two layers:

1. **Bootstrap Config** (`secreton.toml`) - Infrastructure only, NO SECRETS
2. **Application Config** (encrypted in storage) - All sensitive settings

---

## Quick Start

### 1. Development Setup

```bash
# Copy example config
cp secreton.toml.example secreton.toml

# Edit for your environment (still no secrets!)
vim secreton.toml

# Initialize engine (generates Shamir shares)
secreton init --shares 5 --threshold 3

# Save the shares securely!
# Secret Vault is now SEALED

# Unseal engine (need 3 of 5 shares)
secreton unseal  # Enter share 1
secreton unseal  # Enter share 2
secreton unseal  # Enter share 3 -> UNSEALED!

# Start server
secreton server
```

### 2. Migrating from Old Config

```bash
# IMPORTANT: Unseal engine first!
secreton unseal

# Migrate (creates backup automatically)
secreton migrate --from config/

# Verify migration worked
secreton status

# After verification, cleanup old files
secreton migrate --cleanup
```

---

## Configuration Files

### Bootstrap Config (`secreton.toml`)

**Location:** Repository root
**Committed:** ✅ Yes (contains NO secrets)
**Purpose:** Infrastructure configuration

**Contains:**
- Storage backend type (Raft/File/Postgres)
- Listener addresses (HTTP/gRPC)
- Seal type (Shamir/AWS KMS/GCP KMS/Azure KV)
- Telemetry settings
- Log level

**Does NOT contain:**
- Database passwords
- JWT secrets
- API keys
- Any sensitive credentials

### Application Config (Encrypted)

**Location:** Storage backend (encrypted)
**Committed:** ❌ Never
**Purpose:** All application settings

**Contains (encrypted):**
- Auth settings (JWT secrets)
- Database connection URLs
- MFA policies
- Rate limiting
- CORS settings
- Audit configuration
- Backup schedules

---

## Common Operations

### Check Seal Status

```bash
secreton status

# Output:
# 🟢 Secret Vault Status: UNSEALED
# Initialized: Yes
# Seal Type: shamir
# Total Shares: 5
# Threshold: 3
```

### Seal Secret Vault

```bash
secreton seal

# Secret Vault is now SEALED
# All operations blocked until unsealed
```

### Unseal Secret Vault

```bash
# Method 1: Interactive (secure)
secreton unseal
# Enter unseal key: ****

# Method 2: With key argument (less secure)
secreton unseal --key <base64-share>

# Method 3: Reset and start over
secreton unseal --reset
```

### Rekey (Change Shares/Threshold)

```bash
# Start rekey operation
secreton rekey init --shares 7 --threshold 4

# Provide old keys (need current threshold)
secreton rekey update
secreton rekey update
secreton rekey update

# New shares generated when threshold met
```

---

## Storage Backends

### Raft (Recommended for Production)

```toml
[storage]
backend = "raft"

[storage.raft]
path = "/var/lib/secreton/raft"
node_id = "node1"

# For HA cluster:
[[storage.raft.retry_join]]
leader_api_addr = "https://node2:8200"
```

**Pros:** Built-in HA, no external dependencies
**Cons:** Requires 3+ nodes for HA

### File (Development/Testing)

```toml
[storage]
backend = "file"

[storage.file]
path = "./data/secreton"
```

**Pros:** Simple, no dependencies
**Cons:** Single node only

### PostgreSQL (Legacy)

```toml
[storage]
backend = "postgres"

[storage.postgres]
max_connections = 50
```

**Note:** Connection URL from `SECRETON_STORAGE_URL` env var only!

---

## Seal Types

### Shamir Secret Sharing (Default)

```toml
[seal]
type = "shamir"

[seal.shamir]
shares = 5
threshold = 3
```

**Pros:** No external dependencies, proven security
**Cons:** Manual unseal required

### AWS KMS Auto-Unseal

```toml
[seal]
type = "aws-kms"

[seal.aws_kms]
region = "us-east-1"
kms_key_id = "arn:aws:kms:..."
```

**Pros:** Auto-unseal on restart
**Cons:** AWS dependency, cost

### GCP KMS Auto-Unseal

```toml
[seal]
type = "gcp-kms"

[seal.gcp_kms]
project = "my-project"
region = "global"
key_ring = "secreton-keyring"
crypto_key = "seal-key"
```

### Azure Key Secret Vault Auto-Unseal

```toml
[seal]
type = "azure-kv"

[seal.azure_kv]
engine_name = "secreton-kv"
key_name = "seal-key"
```

---

## Environment Variables

### Storage

- `SECRETON_STORAGE_URL` - PostgreSQL connection (PostgreSQL backend only)

### Auto-Unseal

**AWS KMS:**
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_REGION` (optional, overrides config)

**GCP KMS:**
- `GOOGLE_APPLICATION_CREDENTIALS` (path to service account JSON)

**Azure Key Secret Vault:**
- `AZURE_TENANT_ID`
- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`

---

## Security Best Practices

### ✅ DO

- Keep Shamir shares in separate secure locations
- Use auto-unseal (KMS) in production if possible
- Enable TLS for all listeners
- Use Raft storage for HA deployments
- Backup config files before migration
- Rotate unseal keys periodically (rekey)

### ❌ DON'T

- Commit `secreton.toml` with production values
- Store all Shamir shares together
- Use `memory` backend in production
- Disable TLS in production
- Skip backups before migration
- Share unseal keys via insecure channels

---

## Troubleshooting

### "Secret Vault is sealed"

```bash
# Check status
secreton status

# Unseal with shares
secreton unseal
secreton unseal
secreton unseal
```

### "Config not found in storage"

```bash
# Secret Vault may not be initialized
secreton init

# Or engine is sealed
secreton unseal

# Or migration not completed
secreton migrate --from config/
```

### "Failed to get master key"

```bash
# Secret Vault is sealed, unseal first
secreton unseal
```

### Migration failed

```bash
# Check backup exists
ls -la config_backup_*/

# Secret Vault must be unsealed
secreton status
secreton unseal

# Try migration again
secreton migrate --from config/
```

---

## File Structure

```
secreton/
├── secreton.toml.example    # Example config (commit)
├── secreton.toml             # Actual config (gitignored)
│
├── config/                   # LEGACY (to be deleted)
│   ├── default.toml          # OLD - delete after migration
│   ├── production.toml       # OLD - delete after migration
│   ├── raft.toml             # OLD - delete after migration
│   └── engine.toml            # OLD - delete after migration
│
├── config_backup_*/          # Migration backups (gitignored)
│
└── crates/core/src/config/
    ├── bootstrap.rs          # Bootstrap config parser
    ├── application.rs        # Encrypted config
    └── migrate.rs            # Migration utilities
```

---

## API Endpoints

All seal/unseal operations available via REST API:

- `GET /v1/sys/seal-status` - Check engine status
- `POST /v1/sys/init` - Initialize engine
- `POST /v1/sys/seal` - Seal engine (requires auth)
- `POST /v1/sys/unseal` - Unseal engine
- `POST /v1/sys/rekey/init` - Start rekey
- `POST /v1/sys/rekey/update` - Rekey progress

---

## Need Help?

- Documentation: `docs/configuration.md`
- Migration guide: `docs/migration.md`
- Examples: `secreton.toml.example`
- CLI help: `secreton --help`
