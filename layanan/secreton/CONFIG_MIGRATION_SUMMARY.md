# Config Migration Summary

## What Changed?

Secreton has migrated from a legacy single-file config system to a **secure two-layer config system** following HashiCorp Secret Vault's security model.

## Old System (REMOVED)

```
config/
├── default.toml      ❌ Contained ALL config including secrets
└── production.toml   ❌ Contained ALL config including secrets
```

## New System (ACTIVE)

```
secreton.toml         ✅ Bootstrap config (infrastructure only, NO SECRETS)
                      ✅ Application config encrypted in storage backend
```

## Quick Start

### 1. Create Bootstrap Config

```bash
cp secreton.toml.example secreton.toml
# Edit secreton.toml - configure storage, listeners, seal
```

### 2. Start Secreton

```bash
cargo run --bin api_server
# or
docker-compose up -d
```

### 3. Initialize Secret Vault (First Time Only)

```bash
curl -X POST http://localhost:8200/v1/sys/init \
  -d '{"secret_shares": 5, "secret_threshold": 3}'
```

Save the unseal keys and root token securely!

### 4. Unseal Secret Vault

```bash
# Provide 3 of 5 uns
rl -X POST http://localhost:8200/v1/sys/unseal \
  -d '{"key": "UNSEAL_KEY_1"}'
curl -X POST http://localhost:8200/v1/sys/unseal \
  -d '{"key": "UNSEAL_KEY_2"}'
curl -X POST http://localhost:8200/v1/sys/unseal \
  -d '{"key": "UNSEAL_KEY_3"}'
```

### 5. Application Config Loaded Automatically

Once unsealed, application config (auth, database, MFA) is loaded from encrypted storage.

## Configuration Files

### `secreton.toml` (Bootstrap - Safe to Commit)

```toml
[storage]
backend = "raft"

[storage.raft]
path = "/var/lib/secreton/raft"
node_id = "node1"

[listener.http]
address = "0.0.0.0:8200"
tls_enabled = false

[listener.grpc]
enabled = true
address = "0.0.0.0:8201"

[seal]
type = "shamir"

[seal.shamir]
shares = 5
threshold = 3
```

**Contains:** Storage, listeners, seal config
**Does NOT contain:** Secrets, passwords, API keys

### Application Config (Encrypted in Storage)

- Auth settings (JWT secrets)
- Database credentials
- MFA policies
- Rate limiting
- CORS configuration
- Audit settings

**Stored:** Encrypted in storage backend
**Accessible:** Only when engine is unsealed
**Protected by:** Master key + Shamir Secret Sharing

## Environment Variables

```bash
# Bootstrap config path (optional)
export SECRETON_CONFIG=/path/to/secreton.toml

# Runtime overrides (optional)
export HTTP_PORT=8200
export GRPC_PORT=8201
export LOG_LEVEL=info
```

## Docker Deployment

```yaml
services:
  secreton:
    image: secreton:latest
    environment:
      - SECRETON_CONFIG=/app/secreton.toml
      - HTTP_PORT=8200
      - GRPC_PORT=8201
    volumes:
      - ./secreton.toml:/app/secreton.toml:ro
      - secreton_data:/var/lib/secreton
```

## Kubernetes Deployment

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: secreton-config
data:
  secreton.toml: |
    [storage]
    backend = "raft"
    ...
---
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: secreton
    volumeMounts:
    - name: config
      mountPath: /app/secreton.toml
      subPath: secreton.toml
  volumes:
  - name: config
    secret:
      secretName: secreton-config
```

## Security Benefits

| Feature | Old System | New System |
|---------|-----------|------------|
| Secrets in config files | ❌ Plain text | ✅ Encrypted |
| Master key protection | ❌ None | ✅ Shamir SSS |
| Startup state | ❌ Always open | ✅ Sealed by default |
| Config in git | ❌ Risky | ✅ Safe (no secrets) |
| Secret rotation | ❌ Manual file edit | ✅ API-driven |

## Migration Notes

- **No migration needed** - Secreton was never used in production
- Legacy `config/` directory has been removed
- All new deployments use the secure config system
- See `MIGRATION_COMPLETE.md` for technical details

## Documentation

- Full guide: `docs/SECURE_CONFIG_README.md`
- Quick reference: `docs/CONFIG_QUICK_REFERENCE.md`
- Deployment: `docs/DEPLOYMENT.md`
- Example: `secreton.toml.example`
