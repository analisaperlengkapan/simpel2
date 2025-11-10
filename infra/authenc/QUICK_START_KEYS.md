# 🚀 Quick Start: Production Deployment with Signing Keys

## ⚡ 5-Minute Setup

### Step 1: Generate Signing Key

```bash
cd /srv/proyek/simpelv2/infra/authenc
cargo run --bin generate-signing-keys --algorithm ed25519 --format env-var
```

**Output example**:

```
ED25519_PRIVATE_KEY_BASE64="WKp2x3Yz... (base64 string)"
```

### Step 2: Set Environment Variable

**Docker Compose**:

```yaml
services:
  authenc:
    environment:
      - ED25519_PRIVATE_KEY_BASE64=WKp2x3Yz...
```

**Kubernetes**:

```bash
kubectl create secret generic authenc-keys \
  --from-literal=ed25519-key='WKp2x3Yz...'
```

### Step 3: Deploy & Verify

**Check logs for**:

```
✅ Ed25519 signing key loaded from ED25519_PRIVATE_KEY_BASE64
```

✅ **Done!** Tokens will now persist across restarts.

---

## 📋 Environment Variables Reference

| Variable                        | Required             | Description                        |
| ------------------------------- | -------------------- | ---------------------------------- |
| `ED25519_PRIVATE_KEY_BASE64`    | **YES (Production)** | Base64-encoded Ed25519 private key |
| `ED25519_PRIVATE_KEY_PATH`      | Alternative          | File path to Ed25519 key           |
| `ECDSA_P256_PRIVATE_KEY_BASE64` | Alternative          | ECDSA P-256 key                    |
| `ECDSA_P384_PRIVATE_KEY_BASE64` | Alternative          | ECDSA P-384 key                    |
| `ECDSA_P521_PRIVATE_KEY_BASE64` | Alternative          | ECDSA P-521 key                    |

⚠️ **Choose ONE algorithm** - Ed25519 is recommended for best performance.

---

## 🔒 Key Storage Best Practices

### Kubernetes (Recommended)

```bash
# Create secret
kubectl create secret generic authenc-signing-keys \
  --from-literal=ed25519-private-key='<base64-key>' \
  -n authenc

# Use in deployment
env:
- name: ED25519_PRIVATE_KEY_BASE64
  valueFrom:
    secretKeyRef:
      name: authenc-signing-keys
      key: ed25519-private-key
```

### Docker Swarm

```bash
# Create secret
echo '<base64-key>' | docker secret create authenc_ed25519_key -

# Use in stack
secrets:
  - authenc_ed25519_key
environment:
  - ED25519_PRIVATE_KEY_BASE64=/run/secrets/authenc_ed25519_key
```

---

## ✅ Verification Checklist

Before going to production:

- [ ] Generated signing key using official tool
- [ ] Key stored securely (Kubernetes Secret / Vault)
- [ ] Environment variable configured in deployment
- [ ] Service starts successfully
- [ ] Logs show: `✅ Ed25519 signing key loaded`
- [ ] JWT token remains valid after service restart
- [ ] Key backed up in secure location

---

## ⚠️ Production Warnings

### ❌ DO NOT Deploy Without Keys!

If you see this in logs:

```
⚠️  ED25519_PRIVATE_KEY_BASE64 not set
⚠️  Generating EPHEMERAL Ed25519 signing key
⚠️  NOT SUITABLE FOR PRODUCTION!
```

**Action**: STOP deployment and configure persistent keys immediately.

**Impact of missing keys**:

- ❌ All tokens invalidated on every restart
- ❌ Users must re-login after every deployment
- ❌ SSO/federation broken
- ❌ Rolling updates impossible

---

## 🔄 Key Rotation (Quarterly)

### Quick Rotation Procedure

1. **Generate new key**:

```bash
cargo run --bin generate-signing-keys --algorithm ed25519 > new-key.txt
```

2. **Update secret**:

```bash
kubectl create secret generic authenc-signing-keys-new \
  --from-literal=ed25519-private-key='<new-key>'
```

3. **Rolling update**:

```bash
kubectl set env deployment/authenc \
  ED25519_PRIVATE_KEY_BASE64=$(cat new-key.txt)
kubectl rollout status deployment/authenc
```

4. **Verify** logs show new key loaded

5. **Wait 7 days** (grace period for old tokens)

6. **Done!**

---

## 🆘 Troubleshooting

### Problem: "Failed to load Ed25519 key from environment"

**Check**:

```bash
# Verify key is set
kubectl exec deployment/authenc -- env | grep ED25519

# Check key length (should output 32 for Ed25519)
echo $ED25519_PRIVATE_KEY_BASE64 | base64 -d | wc -c
```

**Fix**: Regenerate key using official tool.

---

### Problem: Tokens invalid after restart

**Check logs**:

```bash
kubectl logs deployment/authenc | grep "signing key"
```

**Expected**: `✅ Ed25519 signing key loaded`

**If see ephemeral warning**: Keys not configured properly.

---

## 📚 Full Documentation

For detailed information, see:

- **Setup Guide**: [`docs/SIGNING_KEY_SETUP.md`](docs/SIGNING_KEY_SETUP.md)
- **Fix Details**: [`CRITICAL_BLOCKER_FIX.md`](CRITICAL_BLOCKER_FIX.md)

---

## 🎯 Quick Commands

```bash
# Generate Ed25519 key
cargo run --bin generate-signing-keys --algorithm ed25519

# Generate P-256 key
cargo run --bin generate-signing-keys --algorithm p256

# Output for Kubernetes
cargo run --bin generate-signing-keys --algorithm ed25519 --format k8s-secret

# Output for Docker
cargo run --bin generate-signing-keys --algorithm ed25519 --format docker-secret

# Verify service logs
docker logs authenc | grep "signing key"
kubectl logs -f deployment/authenc | grep "signing key"
```

---

**🚨 REMEMBER**: Without persistent signing keys, **PRODUCTION DEPLOYMENT WILL FAIL**. Always verify keys are loaded successfully in logs!
