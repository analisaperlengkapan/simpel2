# Signing Key Setup for Production

## Overview

Authenc uses cryptographic signing keys for JWT token generation. **These keys MUST be persistent across service restarts** to prevent invalidating all active tokens.

## Critical Requirements

⚠️ **PRODUCTION DEPLOYMENT BLOCKER**: Failing to set persistent signing keys will cause:

- All JWT tokens to become invalid on every restart
- Users forced to re-authenticate after every deployment
- SSO/federation failures due to signature mismatches
- Complete service disruption during rolling updates

## Environment Variables

### Primary Signing Key (Required for Production)

Choose **ONE** of the following algorithms and set the corresponding environment variable:

#### Ed25519 (Recommended - Fast & Secure)

```bash
export ED25519_PRIVATE_KEY_BASE64="<base64-encoded-32-byte-key>"
```

**OR** load from file:

```bash
export ED25519_PRIVATE_KEY_PATH="/path/to/ed25519.key"
```

#### ECDSA P-256 (Alternative)

```bash
export ECDSA_P256_PRIVATE_KEY_BASE64="<base64-encoded-key>"
```

#### ECDSA P-384 (High Security)

```bash
export ECDSA_P384_PRIVATE_KEY_BASE64="<base64-encoded-key>"
```

#### ECDSA P-521 (Maximum Security)

```bash
export ECDSA_P521_PRIVATE_KEY_BASE64="<base64-encoded-key>"
```

## Key Generation

### Using the Built-in Tool

We provide a key generation tool:

```bash
# Generate Ed25519 keypair (recommended)
cargo run --bin generate-signing-keys -- --algorithm ed25519

# Generate ECDSA P-256 keypair
cargo run --bin generate-signing-keys -- --algorithm p256

# Generate ECDSA P-384 keypair
cargo run --bin generate-signing-keys -- --algorithm p384

# Generate ECDSA P-521 keypair
cargo run --bin generate-signing-keys -- --algorithm p521
```

The tool will output:

1. Base64-encoded private key (for environment variable)
2. Raw private key file (for file-based loading)
3. Public key in JWK format (for verification)

### Manual Generation

#### Ed25519 (using openssl)

```bash
# Generate private key
openssl genpkey -algorithm ED25519 -out ed25519-private.pem

# Extract raw key bytes and encode to base64
openssl pkey -in ed25519-private.pem -outform DER | \
    tail -c 32 | base64 | tr -d '\n'
```

#### ECDSA P-256

```bash
# Generate private key
openssl ecparam -genkey -name prime256v1 -noout -out p256-private.pem

# Extract raw key bytes and encode to base64
openssl ec -in p256-private.pem -outform DER | \
    openssl asn1parse -inform DER | \
    grep -A1 "OCTET STRING" | tail -1 | \
    cut -d: -f4 | xxd -r -p | base64 | tr -d '\n'
```

## Deployment Methods

### 1. Environment Variables (Recommended for Kubernetes)

```yaml
# kubernetes/deployment.yaml
apiVersion: v1
kind: Secret
metadata:
  name: authenc-signing-keys
type: Opaque
data:
  ed25519-private-key: <base64-encoded-key>
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: authenc
spec:
  template:
    spec:
      containers:
        - name: authenc
          env:
            - name: ED25519_PRIVATE_KEY_BASE64
              valueFrom:
                secretKeyRef:
                  name: authenc-signing-keys
                  key: ed25519-private-key
```

### 2. Docker Secrets

```bash
# Create secret
echo "<base64-encoded-key>" | docker secret create authenc_ed25519_key -

# Use in docker-compose.yml
version: '3.8'
services:
  authenc:
    image: authenc:latest
    secrets:
      - authenc_ed25519_key
    environment:
      - ED25519_PRIVATE_KEY_BASE64=/run/secrets/authenc_ed25519_key
```

### 3. Secreton Integration (Future)

```bash
# Store key in Secreton vault
secreton set authenc/signing-keys/ed25519 "<base64-key>"

# Authenc will automatically fetch from Secreton if configured
export SECRETON_ENDPOINT="https://secreton.internal"
export SECRETON_TOKEN="<token>"
```

## Key Rotation

### When to Rotate Keys

- **Scheduled**: Every 90-180 days (recommended)
- **Compromised**: Immediately if key exposure suspected
- **Compliance**: Per organizational security policy

### Rotation Procedure

1. **Generate new key** (using tool or manual method)
2. **Deploy with both old and new keys**:
   ```bash
   # Keep old key for grace period
   export ED25519_PRIVATE_KEY_BASE64="<new-key>"
   export ED25519_PRIVATE_KEY_OLD_BASE64="<old-key>"  # Optional
   ```
3. **Wait for grace period** (default: 7 days)
4. **Remove old key** after all tokens issued with old key expire

### Automated Rotation (Future)

The key rotation service can be enabled in `config/production.toml`:

```toml
[key_rotation]
enabled = true
rotation_interval_days = 90
grace_period_days = 7
enable_notifications = true
```

## Verification

### Check Key Loading on Startup

Look for these log messages on service start:

✅ **Success**:

```
✅ Ed25519 signing key loaded from ED25519_PRIVATE_KEY_BASE64
```

⚠️ **Warning (Development)**:

```
⚠️  ED25519_PRIVATE_KEY_BASE64 or ED25519_PRIVATE_KEY_PATH not set
⚠️  Generating EPHEMERAL Ed25519 signing key
⚠️  ALL JWT TOKENS WILL BE INVALIDATED ON RESTART
⚠️  THIS IS NOT SUITABLE FOR PRODUCTION!
```

### Test Token Persistence

```bash
# Get token before restart
TOKEN=$(curl -X POST http://localhost:8088/oauth2/token \
  -d "grant_type=password" \
  -d "username=test" \
  -d "password=test123" | jq -r .access_token)

# Restart service
docker restart authenc

# Verify token still valid
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8088/oauth2/userinfo
```

If token is rejected after restart, keys were not persisted correctly.

## Security Best Practices

1. **Never commit keys to git**

   - Add to `.gitignore`: `*.key`, `*.pem`, `ed25519-private.*`

2. **Restrict access**

   - File permissions: `chmod 600 <keyfile>`
   - Kubernetes RBAC for secrets

3. **Backup keys securely**

   - Encrypt with strong passphrase
   - Store in multiple secure locations
   - Document recovery procedure

4. **Monitor key usage**

   - Enable audit logging for key operations
   - Alert on key loading failures
   - Track key rotation events

5. **Use hardware security modules (HSM)** for maximum security
   - TPM integration available
   - PKCS#11 support (future)

## Troubleshooting

### Problem: "Failed to load Ed25519 key from environment"

**Cause**: Invalid base64 encoding or wrong key length

**Solution**:

1. Verify base64 encoding: `echo $ED25519_PRIVATE_KEY_BASE64 | base64 -d | wc -c`
   - Should output: `32` (for Ed25519)
2. Regenerate key using provided tool
3. Check for whitespace/newlines in environment variable

### Problem: Tokens invalid after restart despite setting key

**Cause**: Key not loaded (check logs), or different key used

**Solution**:

1. Check startup logs for "✅ Ed25519 signing key loaded"
2. Verify environment variable is set in container: `docker exec authenc env | grep ED25519`
3. Compare public key before/after restart: `curl http://localhost:8088/.well-known/jwks`

### Problem: Key rotation not working

**Cause**: Key rotation service not enabled or Secreton not configured

**Solution**:

1. Enable in config: `key_rotation.enabled = true`
2. Configure Secreton endpoint
3. Check key rotation logs: `docker logs authenc | grep "key rotation"`

## Production Checklist

Before deploying to production:

- [ ] Generate signing key using provided tool
- [ ] Store key securely (Kubernetes Secret / Docker Secret / Vault)
- [ ] Set environment variable (ED25519_PRIVATE_KEY_BASE64 or equivalent)
- [ ] Verify key loads successfully in logs
- [ ] Test token persistence across restarts
- [ ] Document key backup procedure
- [ ] Configure key rotation schedule
- [ ] Setup monitoring/alerting for key operations
- [ ] Review security audit logs

## Support

For questions or issues:

- Review logs: `docker logs authenc | grep -i "signing key"`
- Check GitHub issues: https://github.com/analisaperlengkapan/simpel2/issues
- Contact: security@kejaksaan.go.id
