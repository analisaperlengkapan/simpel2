# Secreton Vault Initialization & Seal/Unseal Guide

## Overview

Secreton follows **HashiCorp Vault best practices** for vault initialization and seal/unseal operations. This ensures production-grade security and operational reliability.

## Workflow

### Phase 1: Initialization (First Time Only)

When Secreton starts for the first time, the vault is **SEALED** and **UNINITIALIZED**.

```
┌─────────────────────────────────────────────────────────┐
│  Vault State: SEALED + UNINITIALIZED                    │
│  Status: ❌ Not ready for operations                     │
│  Action Required: Initialize vault                       │
└─────────────────────────────────────────────────────────┘
```

#### Step 1: Check Initialization Status

```bash
curl http://localhost:8200/v1/sys/seal-status
```

Response (uninitialized):
```json
{
  "seal_type": "shamir",
  "initialized": false,
  "sealed": true,
  "t": 0,
  "n": 0,
  "progress": 0,
  "nonce": "",
  "version": "0.1.0"
}
```

#### Step 2: Initialize Vault (Generate Master Key Shares)

Initialize vault with **Shamir Secret Sharing**:
- `secret_shares`: Total number of key shares to generate (e.g., 5)
- `secret_threshold`: Minimum shares needed to unseal (e.g., 3)

**Recommended**: 5 shares with 3 threshold (3-of-5 scheme)

```bash
curl -X POST http://localhost:8200/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{
    "secret_shares": 5,
    "secret_threshold": 3
  }'
```

Response:
```json
{
  "success": true,
  "data": {
    "keys": [
      "base64_encoded_share_1",
      "base64_encoded_share_2",
      "base64_encoded_share_3",
      "base64_encoded_share_4",
      "base64_encoded_share_5"
    ],
    "root_token": "base64_encoded_root_token"
  }
}
```

**⚠️ CRITICAL SECURITY STEPS:**

1. **Save the shares securely**:
   - Store each share in a separate secure location
   - Use different people/systems to hold shares
   - Never store all shares in one place
   - Consider using hardware security modules (HSM)

2. **Save the root token**:
   - Store in a secure vault/password manager
   - This is the initial authentication token
   - Can be revoked after setting up proper auth

3. **Verify initialization**:
   ```bash
   curl http://localhost:8200/v1/sys/seal-status
   ```

   Response should show:
   ```json
   {
     "initialized": true,
     "sealed": true,
     "t": 5,
     "n": 3
   }
   ```

### Phase 2: Unsealing (Every Restart)

After initialization, the vault remains **SEALED** until unsealed with threshold shares.

```
┌─────────────────────────────────────────────────────────┐
│  Vault State: SEALED + INITIALIZED                      │
│  Status: ⚠️  Sealed - operations blocked                 │
│  Action Required: Provide unseal shares                  │
└─────────────────────────────────────────────────────────┘
```

#### Step 1: Check Seal Status

```bash
curl http://localhost:8200/v1/sys/seal-status
```

Response (sealed):
```json
{
  "sealed": true,
  "progress": 0,
  "t": 5,
  "n": 3,
  "nonce": "session_nonce_12345"
}
```

#### Step 2: Provide Unseal Shares

Provide shares one at a time until threshold is reached:

```bash
# Share 1
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "base64_encoded_share_1"}'

# Share 2
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "base64_encoded_share_2"}'

# Share 3 (threshold reached - vault unseals)
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "base64_encoded_share_3"}'
```

Response after each share:
```json
{
  "sealed": true,
  "progress": 1,
  "t": 5,
  "n": 3
}
```

Response when threshold reached:
```json
{
  "sealed": false,
  "progress": 0,
  "t": 5,
  "n": 3
}
```

#### Step 3: Verify Unsealed Status

```bash
curl http://localhost:8200/v1/sys/seal-status
```

Response (unsealed):
```json
{
  "sealed": false,
  "initialized": true,
  "progress": 0
}
```

### Phase 3: Normal Operations

Once unsealed, vault is ready for operations:

```
┌─────────────────────────────────────────────────────────┐
│  Vault State: UNSEALED + INITIALIZED                    │
│  Status: ✅ Ready for operations                         │
│  Actions: Create/read/update/delete secrets              │
└─────────────────────────────────────────────────────────┘
```

#### Create Secret

```bash
curl -X POST http://localhost:8200/v1/secret/my-app/db \
  -H "Content-Type: application/json" \
  -H "X-Vault-Token: root_token" \
  -d '{
    "data": {
      "username": "admin",
      "password": "secret123"
    }
  }'
```

#### Read Secret

```bash
curl http://localhost:8200/v1/secret/my-app/db \
  -H "X-Vault-Token: root_token"
```

#### List Secrets

```bash
curl http://localhost:8200/v1/secrets \
  -H "X-Vault-Token: root_token"
```

### Phase 4: Sealing (Maintenance)

To seal the vault (e.g., for maintenance):

```bash
curl -X POST http://localhost:8200/v1/sys/seal \
  -H "X-Vault-Token: root_token"
```

⚠️ **WARNING**: Sealing immediately blocks all operations until unsealed again.

## Security Best Practices

### 1. Shamir Secret Sharing

- **Never use 1-of-1**: Always require multiple shares
- **Recommended**: 5-of-3 (5 shares, 3 threshold)
- **High Security**: 7-of-4 (7 shares, 4 threshold)
- **Distribute shares**: Give to different people/locations

### 2. Master Key Management

- **Store securely**: Use HSM, encrypted vaults, or secure storage
- **Rotate regularly**: Implement key rotation policy
- **Audit access**: Log all unseal operations
- **Separate duties**: Different people hold different shares

### 3. Root Token

- **Revoke after setup**: Don't use for daily operations
- **Create auth methods**: Use AppRole, JWT, OIDC, etc.
- **Audit token usage**: Monitor root token access
- **Rotate tokens**: Implement token rotation policy

### 4. Unseal Operations

- **Rate limiting**: 10 attempts per 60 seconds per IP (built-in)
- **Audit logging**: All unseal attempts logged
- **Secure transport**: Always use HTTPS in production
- **Timeout**: Unseal session expires after inactivity

### 5. Seal/Unseal Automation

For production, consider:

```bash
# Automated unseal with cloud KMS
# Example: AWS KMS, Google Cloud KMS, Azure Key Vault

# Or use Vault Enterprise auto-unseal
# Or implement custom unseal orchestration
```

## Troubleshooting

### Vault won't initialize

```bash
# Check if already initialized
curl http://localhost:8200/v1/sys/seal-status

# If initialized=true, vault is already initialized
# Cannot reinitialize without reset
```

### Unseal fails with "Invalid unseal key"

- Verify key is base64-encoded correctly
- Check key hasn't been corrupted
- Ensure using shares from same initialization

### Unseal progress resets

```bash
# Reset unseal session if needed
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"reset": true}'
```

### Rate limited on unseal

- Wait 60 seconds before retrying
- Limit is 10 attempts per 60 seconds per IP
- Check logs for suspicious activity

## Production Checklist

- [ ] Generate and securely store Shamir shares
- [ ] Store root token in secure vault
- [ ] Distribute shares to authorized personnel
- [ ] Test unseal procedure
- [ ] Set up audit logging
- [ ] Configure TLS certificates
- [ ] Set up monitoring/alerting
- [ ] Document recovery procedures
- [ ] Train operators on unseal process
- [ ] Implement key rotation policy
- [ ] Set up automated backups
- [ ] Configure HA/clustering if needed

## Comparison with HashiCorp Vault

| Feature | Secreton | Vault |
|---------|----------|-------|
| Initialization | ✅ Shamir shares | ✅ Shamir shares |
| Seal/Unseal | ✅ Manual | ✅ Manual/Auto |
| Rate Limiting | ✅ 10/60s | ✅ Configurable |
| Audit Logging | ✅ All operations | ✅ All operations |
| Root Token | ✅ Generated | ✅ Generated |
| Auto-Unseal | ❌ Not yet | ✅ Enterprise |
| HA Clustering | ❌ Not yet | ✅ Yes |

## API Reference

### GET /v1/sys/seal-status
Returns current seal status (accessible when sealed)

### POST /v1/sys/init
Initialize vault with Shamir shares

### POST /v1/sys/unseal
Provide unseal key/share

### POST /v1/sys/seal
Seal the vault (requires authentication)

### POST /v1/sys/rekey/init
Start rekey operation

### POST /v1/sys/rekey/update
Provide shares for rekey

## Examples

### Complete Initialization & Unseal Workflow

```bash
#!/bin/bash

BASE_URL="http://localhost:8200"

# Step 1: Initialize
echo "Initializing vault..."
INIT_RESPONSE=$(curl -s -X POST $BASE_URL/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{
    "secret_shares": 5,
    "secret_threshold": 3
  }')

# Extract shares and token
SHARES=$(echo $INIT_RESPONSE | grep -o '"keys":\[.*\]' | cut -d'[' -f2 | cut -d']' -f1)
ROOT_TOKEN=$(echo $INIT_RESPONSE | grep -o '"root_token":"[^"]*' | cut -d'"' -f4)

echo "Root Token: $ROOT_TOKEN"
echo "Shares: $SHARES"

# Step 2: Unseal (provide 3 shares)
echo "Unsealing vault..."
for i in {1..3}; do
  SHARE=$(echo $SHARES | cut -d',' -f$i | tr -d ' "')
  curl -s -X POST $BASE_URL/v1/sys/unseal \
    -H "Content-Type: application/json" \
    -d "{\"key\": \"$SHARE\"}" | grep -o '"sealed":[^,]*'
done

# Step 3: Verify unsealed
curl -s $BASE_URL/v1/sys/seal-status | grep -o '"sealed":[^,]*'
```

## References

- [HashiCorp Vault Initialization](https://www.vaultproject.io/api-docs/system/init)
- [Shamir Secret Sharing](https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing)
- [Vault Security Model](https://www.vaultproject.io/docs/internals/security)
- [Vault Best Practices](https://www.vaultproject.io/docs/platform/security)
