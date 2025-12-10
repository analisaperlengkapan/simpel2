# Secreton Secure Configuration System

## 🔐 Overview

Secreton now uses a **Vault-like secure configuration system** with two layers:

1. **Bootstrap Config** (`secreton.toml`) - Infrastructure only, **NO SECRETS**
2. **Application Config** (encrypted in storage) - All sensitive settings, protected by master key

---

## Quick Start

### 1. Initialize Vault

```bash
# Copy example config
cp secreton.toml.example secreton.toml

# Initialize (generates Shamir shares)
secreton init --shares 5 --threshold 3

# IMPORTANT: Save shares securely!
```

### 2. Unseal Vault

```bash
# Provide 3 of 5 shares
secreton unseal  # Share 1
secreton unseal  # Share 2
secreton unseal  # Share 3 ✅ UNSEALED
```

### 3. Start Server

```bash
secreton server
```

---

## Configuration Files

### `secreton.toml` (Bootstrap - Safe to Commit)

```toml
[storage]
backend = "raft"  # or "file", "postgres"
path = "/var/lib/secreton/raft"
node_id = "node1"

[listener.http]
address = "0.0.0.0:8200"
tls_enabled = true

[seal]
type = "shamir"  # or "aws-kms", "gcp-kms", "azure-kv"
shares = 5
threshold = 3
```

**Contains:** Storage type, listeners, seal configuration
**Does NOT contain:** Database passwords, API keys, JWT secrets

### Application Config (Encrypted in Storage)

Stored encrypted after unsealing:
- Auth settings (JWT secrets)
- Database credentials
- MFA policies
- Rate limiting
- CORS configuration
- Audit settings

---

## Migration from Legacy Config

```bash
# Unseal vault first
secreton unseal

# Migrate (creates automatic backup)
secreton migrate --from config/

# Verify
secreton status

# After verification, cleanup
secreton migrate --cleanup
```

---

## Deployment

### Development
```bash
# Start with file backend
secreton server
```

### Production
```bash
# With Raft HA + TLS + Shamir
systemctl start secreton
secreton unseal  # 3 operators
```

### Production HA (Auto-Unseal with AWS KMS)
```toml
[seal]
type = "aws-kms"
region = "us-east-1"
kms_key_id = "arn:aws:kms:..."
```

---

## CLI Commands

- `secreton init` - Initialize vault, generate Shamir shares
- `secreton seal` - Seal vault (blocks all operations)
- `secreton unseal` - Unseal with share
- `secreton status` - Check seal status
- `secreton rekey` - Change shares/threshold
- `secreton migrate` - Migrate legacy config
- `secreton server` - Start server

---

## Security Features

✅ **AES-256-GCM** encryption for all config
✅ **Shamir Secret Sharing** (5 shares, 3 threshold default)
✅ **Master key** never written to disk unencrypted
✅ **Zeroize** - Secure memory cleanup
✅ **Vault starts SEALED** - Manual or KMS auto-unseal
✅ **Optional cloud KMS** - AWS/GCP/Azure auto-unseal

---

## Documentation

- **Quick Reference:** `docs/CONFIG_QUICK_REFERENCE.md`
- **Deployment Guide:** `docs/DEPLOYMENT.md`
- **Example Config:** `secreton.toml.example`

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                 secreton.toml                            │
│            (Bootstrap - NO SECRETS)                      │
│  • Storage: raft/file/postgres                           │
│  • Listeners: HTTP/gRPC + TLS                            │
│  • Seal: shamir/aws-kms/gcp-kms/azure-kv                 │
└───────────────────┬──────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────────────┐
│                SealService                               │
│  • Master key protected by Shamir SSS                    │
│  • Zeroize sensitive data from memory                    │
│  • Optional KMS auto-unseal                              │
└───────────────────┬──────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────────────┐
│            ApplicationConfig                             │
│         (Encrypted in Storage)                           │
│  • Auth config (JWT secrets encrypted)                   │
│  • Database config (URLs encrypted)                      │
│  • MFA, Rate Limit, CORS, Audit                          │
│                                                           │
│  Encryption: AES-256-GCM with master key                 │
└──────────────────────────────────────────────────────────┘
```

---

## FAQ

**Q: Where are my secrets stored?**
A: Encrypted in the storage backend (Raft/File/Postgres), protected by the master key.

**Q: What happens if I lose Shamir shares?**
A: You cannot unseal the vault. Keep shares in separate secure locations (password managers, HSMs).

**Q: Can I change the number of shares?**
A: Yes, use `secreton rekey init --shares X --threshold Y`.

**Q: Do I need to unseal after restart?**
A: With Shamir: Yes (manual). With KMS auto-unseal: No (automatic).

**Q: Is the bootstrap config secret?**
A: No, it contains NO secrets and can be committed to git (like Vault's config).
