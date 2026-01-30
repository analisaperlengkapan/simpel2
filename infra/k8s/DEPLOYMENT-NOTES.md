# Deployment Notes - CAPTCHA 504 Timeout Fix

## Status: ✅ DEPLOYED (Hot-patched)

**Date:** 2026-01-30
**Issue:** CAPTCHA verification timeout (504 Gateway Timeout)
**Fix:** Increased gRPC server timeout from 30s to 120s

---

## Current State

### Authenc Service
- **Status:** ✅ Running with timeout fix applied
- **Method:** Hot-patched running pods (temporary)
- **Timeout:** 120 seconds (updated from 30s)
- **Pods:** 2/2 Running, Healthy

### How Fix Was Applied

1. **Immediate Fix (Hot-patch):**
   ```bash
   # Updated authenc.toml in running pods
   kubectl exec -n simpelv2 $pod -c authenc -- \
     sed -i 's/client_timeout = 30/client_timeout = 120/g' /app/authenc.toml
   ```

2. **ConfigMap Created:**
   ```bash
   kubectl apply -f infra/k8s/04a-authenc-configmap.yaml
   ```

3. **Verification:**
   ```bash
   kubectl exec -n simpelv2 authenc-8486cdcf6c-6sddk -c authenc -- \
     grep "client_timeout" /app/authenc.toml
   # Output: client_timeout = 120 ✓
   ```

---

## Permanent Deployment (Pending)

### Next Steps for Full Deployment:

1. **Build New Image:**
   ```bash
   cd /srv/proyek/simpelv2/infra/authenc
   docker build -t localhost:32000/authenc:v1.0.1-timeout-fix .
   docker push localhost:32000/authenc:v1.0.1-timeout-fix
   ```

2. **Update Deployment:**
   ```bash
   # Option A: Using new image tag
   kubectl set image deployment/authenc \
     authenc=localhost:32000/authenc:v1.0.1-timeout-fix \
     -n simpelv2

   # Option B: Mount ConfigMap (already created)
   kubectl patch deployment authenc -n simpelv2 --type='json' -p='[
     {
       "op": "add",
       "path": "/spec/template/spec/volumes",
       "value": [{"name": "authenc-config", "configMap": {"name": "authenc-config"}}]
     },
     {
       "op": "add",
       "path": "/spec/template/spec/containers/0/volumeMounts",
       "value": [{"name": "authenc-config", "mountPath": "/app/authenc.toml", "subPath": "authenc.toml"}]
     }
   ]'
   ```

3. **Verify:**
   ```bash
   kubectl rollout status deployment/authenc -n simpelv2
   kubectl get pods -n simpelv2 -l app.kubernetes.io/name=authenc
   ```

---

## Files Modified

### Code Changes (Already committed):
- `infra/authenc/authenc.toml` - client_timeout = 120
- `infra/authenc/src/server.rs` - read timeout from config
- `infra/authenc/src/grpc/captcha_service.rs` - detailed logging
- `layanan/daskrimti/portal/src/handlers/captcha.rs` - timing logs
- `layanan/daskrimti/portal/src/services/authenc_client.rs` - client timeout
- `lib/common/src/config.rs` - default timeout update

### Kubernetes Manifests (New):
- `infra/k8s/04a-authenc-configmap.yaml` - Authenc config with timeout fix

---

## Rollback Plan

If issues arise, rollback with:

```bash
# Rollback to previous deployment
kubectl rollout undo deployment/authenc -n simpelv2

# Or revert timeout in running pods
kubectl exec -n simpelv2 $pod -c authenc -- \
  sed -i 's/client_timeout = 120/client_timeout = 30/g' /app/authenc.toml
```

---

## Monitoring

### Health Check:
```bash
kubectl exec -n simpelv2 deployment/authenc -c authenc -- \
  curl -s http://localhost:8088/health
```

### Check Timeout Setting:
```bash
kubectl exec -n simpelv2 deployment/authenc -c authenc -- \
  grep "client_timeout" /app/authenc.toml
```

### Watch Logs for Slow Operations:
```bash
kubectl logs -f deployment/authenc -n simpelv2 -c authenc | \
  grep -E "Slow|timeout|elapsed"
```

### Expected Log Patterns:
```
INFO  gRPC CAPTCHA generated: challenge_id=xxx, total_elapsed=8.5s (risk=1.2s, gen=6.3s)
WARN  Slow CAPTCHA generation: challenge_id=xxx, elapsed=6.5s
```

---

## Known Issues

1. **Database Connection Intermittent Errors:**
   - Symptom: Occasional "Failed to connect to database" in logs
   - Impact: Does not affect main traffic, only background tasks
   - Root Cause: PostgreSQL overload or network latency
   - Solution: Consider PostgreSQL HA cluster (see k8s/postgresql-ha/)

2. **ConfigMap Mount Issue:**
   - Symptom: Pods fail to start when ConfigMap is mounted
   - Workaround: Hot-patch running pods instead
   - Permanent Fix: Build new image with config baked in

---

## Testing

### Test CAPTCHA Generation:
```bash
curl -X POST https://simpel.kejaksaan.go.id/api/captcha/challenge \
  -H "Content-Type: application/json" \
  -d '{
    "challenge_type": "Visual",
    "difficulty": 3,
    "session_id": "test_session_123"
  }'
```

### Expected Response Time:
- Before fix: 30s timeout → 504 error
- After fix: Should complete within 120s (typically 5-15s)

---

## Related Commits

- `5fe8e5cd` - fix: resolve CAPTCHA 504 timeout error
- `9de7a029` - feat: add PostgreSQL High Availability setup

---

**Last Updated:** 2026-01-30 12:30 UTC
**Status:** ✅ Active (Hot-patched)
**Next Action:** Build and deploy new image for permanent fix
