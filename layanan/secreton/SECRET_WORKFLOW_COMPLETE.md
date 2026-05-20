# Secreton Secret Vault Complete Workflow - HashiCorp Secret Vault Best Practices

## Overview

Secreton implements **HashiCorp Secret Vault-compatible** engine initialization and seal/unseal operations using **Shamir Secret Sharing** for secure master key distribution.

## Architecture

### Seal States

```
┌─────────────────────────────────────────────────────────┐
│                  VAULT LIFECYCLE                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. SEALED + UNINITIALIZED (Fresh Start)                │
│     └─> POST /v1/sys/init                               │
│         └─> Generates Shamir shares + root token        │
│                                                          │
│  2. SEALED + INITIALIZED (Normal State)                 │
│     └─> POST /v1/sys/unseal (provide shares)            │
│         └─> Accumulates shares until threshold reached  │
│                                                          │
│  3. UNSEALED + INITIALIZED (Operational)                │
│     └─> All secret operations available                 │
│     └─> POST /v1/sys/seal to seal again                 │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Shamir Secret Sharing

Secreton uses **Shamir's Secret Sharing** scheme:

- **Master Key**: The root encryption key (generated during init)
- **Shares**: Split master key into N shares
- **Threshold**: Minimum M shares needed to reconstruct master key
- **Security**: Any M shares can reconstruct, but M-1 shares reveal nothing

**Recommended Configuration**: 5-of-3 (5 shares, 3 threshold)

- Distribute 5 shares to 5 different people/locations
- Any 3 people can unseal the engine
- Protects against loss of 2 shares

## Complete Workflow

### Phase 1: Fresh Deployment

#### 1.1 Start Container

```bash
docker run -d --name secreton \
  -p 8200:8200 \
  -p 8201:8201 \
  -e JWT_SECRET="your-secret-key" \
  -e SECRETON_ENV="production" \
  secreton:latest
```

#### 1.2 Verify Uninitialized Status

```bash
curl http://localhost:8200/v1/sys/seal-status
```

Response:

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

#### 1.3 Initialize Secret Vault

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

1. **Save shares securely**:

   ```bash
   # Each share to separate secure location
   echo "share_1" | gpg --encrypt --recipient admin@company.com > share_1.gpg
   echo "share_2" | gpg --encrypt --recipient ops@company.com > share_2.gpg
   echo "share_3" | gpg --encrypt --recipient security@company.com > share_3.gpg
   # ... etc
   ```

2. **Save root token**:

   ```bash
   echo "root_token" | gpg --encrypt --recipient admin@company.com > root_token.gpg
   ```

3. **Store in secure engine**:
   - Use password manager (1Password, LastPass, Secret Vault)
   - Or use HSM (Hardware Security Module)
   - Or use cloud KMS (AWS KMS, GCP KMS, Azure Key Secret Vault)

### Phase 2: Unsealing (Every Restart)

#### 2.1 Check Seal Status

```bash
curl http://localhost:8200/v1/sys/seal-status
```

Response (sealed):

```json
{
  "sealed": true,
  "initialized": true,
  "progress": 0,
  "t": 5,
  "n": 3,
  "nonce": "session_nonce_abc123"
}
```

#### 2.2 Provide Unseal Shares

Provide shares one at a time. Each person provides their share:

**Person 1:**

```bash
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "base64_encoded_share_1"}'
```

Response:

```json
{
  "sealed": true,
  "initialized": true,
  "progress": 1,
  "t": 5,
  "n": 3
}
```

**Person 2:**

```bash
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "base64_encoded_share_2"}'
```

Response:

```json
{
  "sealed": true,
  "initialized": true,
  "progress": 2,
  "t": 5,
  "n": 3
}
```

**Person 3 (Threshold Reached):**

```bash
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key": "base64_encoded_share_3"}'
```

Response:

```json
{
  "sealed": false,
  "initialized": true,
  "progress": 0,
  "t": 5,
  "n": 3
}
```

#### 2.3 Verify Unsealed

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

Once unsealed, engine is ready for operations:

#### 3.1 Create Secret

```bash
curl -X POST http://localhost:8200/v1/secret/my-app/db \
  -H "Content-Type: application/json" \
  -H "X-Secret Vault-Token: root_token" \
  -d '{
    "data": {
      "username": "admin",
      "password": "secret123",
      "host": "db.example.com",
      "port": 5432
    }
  }'
```

#### 3.2 Read Secret

```bash
curl http://localhost:8200/v1/secret/my-app/db \
  -H "X-Secret Vault-Token: root_token"
```

Response:

```json
{
  "success": true,
  "data": {
    "username": "admin",
    "password": "secret123",
    "host": "db.example.com",
    "port": 5432
  }
}
```

#### 3.3 List Secrets

```bash
curl http://localhost:8200/v1/secrets \
  -H "X-Secret Vault-Token: root_token"
```

#### 3.4 Delete Secret

```bash
curl -X DELETE http://localhost:8200/v1/secret/my-app/db \
  -H "X-Secret Vault-Token: root_token"
```

### Phase 4: Sealing (Maintenance)

To seal engine for maintenance:

```bash
curl -X POST http://localhost:8200/v1/sys/seal \
  -H "X-Secret Vault-Token: root_token"
```

⚠️ **WARNING**: Sealing immediately blocks all operations until unsealed.

## Security Best Practices

### 1. Key Distribution

**Recommended**: 5-of-3 Shamir scheme

```
Person 1: Share 1 + Share 2
Person 2: Share 3 + Share 4
Person 3: Share 5
```

Why this distribution:

- No single person has all shares
- Any 3 people can unseal
- Requires coordination (security feature)
- Protects against insider threats

### 2. Root Token Management

**DO:**

- ✅ Store in secure engine
- ✅ Rotate regularly
- ✅ Audit all usage
- ✅ Revoke after setup

**DON'T:**

- ❌ Commit to git
- ❌ Store in plain text
- ❌ Share via email
- ❌ Use for daily operations

### 3. Unseal Operations

**DO:**

- ✅ Rate limit attempts (10/60s built-in)
- ✅ Log all attempts
- ✅ Use HTTPS in production
- ✅ Verify share authenticity

**DON'T:**

- ❌ Provide all shares at once
- ❌ Store shares together
- ❌ Use over unencrypted channels
- ❌ Automate without safeguards

### 4. Monitoring & Alerting

```bash
# Monitor seal status
watch -n 5 'curl -s http://localhost:8200/v1/sys/seal-status | jq'

# Alert on seal changes
# Alert on unseal failures
# Alert on rate limit hits
# Alert on unauthorized access
```

### 5. Disaster Recovery

**Backup Strategy:**

1. Backup engine state regularly
2. Store backups securely (encrypted)
3. Test restore procedures
4. Document recovery steps

**Key Rotation:**

1. Implement rekey operation
2. Rotate shares periodically
3. Update distribution
4. Audit rotation events

## Comparison: Secreton vs HashiCorp Secret Vault

| Feature | Secreton | Secret Vault |
|---------|----------|-------|
| **Initialization** | ✅ Shamir shares | ✅ Shamir shares |
| **Seal/Unseal** | ✅ Manual | ✅ Manual/Auto |
| **Rate Limiting** | ✅ 10/60s | ✅ Configurable |
| **Audit Logging** | ✅ All operations | ✅ All operations |
| **Root Token** | ✅ Generated | ✅ Generated |
| **Auto-Unseal** | ❌ Not yet | ✅ Enterprise |
| **HA Clustering** | ❌ Not yet | ✅ Yes |
| **Rekey** | ⚠️ Partial | ✅ Full |
| **Backup/Restore** | ⚠️ Manual | ✅ Built-in |

## API Reference

### Seal/Unseal Endpoints

#### GET /v1/sys/seal-status

Returns current seal status (accessible when sealed)

**Response:**

```json
{
  "seal_type": "shamir",
  "initialized": true,
  "sealed": false,
  "t": 5,
  "n": 3,
  "progress": 0,
  "nonce": "session_nonce",
  "version": "0.1.0"
}
```

#### POST /v1/sys/init

Initialize engine with Shamir shares

**Request:**

```json
{
  "secret_shares": 5,
  "secret_threshold": 3
}
```

**Response:**

```json
{
  "keys": ["share1", "share2", "share3", "share4", "share5"],
  "root_token": "root_token_base64"
}
```

#### POST /v1/sys/unseal

Provide unseal key/share

**Request:**

```json
{
  "key": "base64_encoded_share",
  "reset": false
}
```

**Response:**

```json
{
  "sealed": true,
  "initialized": true,
  "progress": 1,
  "t": 5,
  "n": 3,
  "nonce": "session_nonce"
}
```

#### POST /v1/sys/seal

Seal the engine (requires authentication)

**Response:** 204 No Content

#### POST /v1/sys/rekey/init

Start rekey operation

#### POST /v1/sys/rekey/update

Provide shares for rekey

## Troubleshooting

### Issue: "Secret Vault is already initialized"

**Cause**: Secret Vault was already initialized in previous run
**Solution**:

- For development: Restart container
- For production: Use rekey operation

### Issue: "Invalid unseal key"

**Cause**: Wrong share provided or corrupted
**Solution**:

- Verify share is base64-encoded correctly
- Check share hasn't been corrupted
- Ensure using shares from same initialization

### Issue: "Rate limit exceeded"

**Cause**: Too many unseal attempts
**Solution**:

- Wait 60 seconds before retrying
- Limit is 10 attempts per 60 seconds per IP
- Check logs for suspicious activity

### Issue: "Threshold not met"

**Cause**: Not enough shares provided yet
**Solution**:

- Continue providing shares until threshold reached
- Check progress in seal-status response

## Production Checklist

- [ ] Generate and securely store Shamir shares
- [ ] Store root token in secure engine
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
- [ ] Implement auto-unseal (future)
- [ ] Set up persistent storage backend

## Example: Complete Workflow Script

```bash
#!/bin/bash

BASE_URL="http://localhost:8200"

# Initialize
echo "Initializing engine..."
INIT=$(curl -s -X POST $BASE_URL/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{"secret_shares": 5, "secret_threshold": 3}')

# Extract and save shares
ROOT_TOKEN=$(echo $INIT | jq -r '.data.root_token')
SHARES=$(echo $INIT | jq -r '.data.keys[]')

echo "Root Token: $ROOT_TOKEN"
echo "Shares:"
echo "$SHARES" | nl

# Unseal
echo "Unsealing engine..."
for share in $(echo "$SHARES" | head -3); do
  curl -s -X POST $BASE_URL/v1/sys/unseal \
    -H "Content-Type: application/json" \
    -d "{\"key\": \"$share\"}" | jq '.progress'
done

# Verify unsealed
curl -s $BASE_URL/v1/sys/seal-status | jq '.sealed'

# Create secret
echo "Creating secret..."
curl -s -X POST $BASE_URL/v1/secret/test \
  -H "X-Secret Vault-Token: $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"data": {"key": "value"}}'

# Read secret
echo "Reading secret..."
curl -s $BASE_URL/v1/secret/test \
  -H "X-Secret Vault-Token: $ROOT_TOKEN" | jq '.data'
```

## References

- [HashiCorp Secret Vault Init API](https://www.engineproject.io/api-docs/system/init)
- [HashiCorp Secret Vault Unseal API](https://www.engineproject.io/api-docs/system/unseal)
- [Shamir Secret Sharing](https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing)
- [Secret Vault Security Model](https://www.engineproject.io/docs/internals/security)
- [Secret Vault Best Practices](https://www.engineproject.io/docs/platform/security)
