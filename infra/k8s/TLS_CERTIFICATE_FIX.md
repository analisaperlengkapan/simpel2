# Panduan Perbaikan SSL Certificate Error

## Masalah
Browser menampilkan error **ERR_CERT_AUTHORITY_INVALID** dengan certificate:
- **CN**: Kubernetes Ingress Controller Fake Certificate
- **O**: Acme Co

## Root Cause
TLS secret `simpelv2-tls-secret` tidak ditemukan di namespace yang benar:
- **Gateway mencari di**: `istio-system` 
- **Secret didefinisikan di**: `istio-ingress` ❌

Ketika Istio Gateway tidak menemukan secret, ia menggunakan **fake certificate** sebagai fallback.

## Solusi yang Diterapkan

### 1. Perbaikan Namespace
File yang diubah:
- `01-sealed-secrets.yaml` - line 303: `namespace: istio-ingress` → `namespace: istio-system`
- `02-regular-secrets.yaml` - line 63: `namespace: istio-ingress` → `namespace: istio-system`

### 2. Sertifikat Asli
Sertifikat asli DigiCert tersedia di: `/etc/ssl/simpel.kejaksaan.go.id/`
- **Certificate**: `fullchain.pem` (CN=*.kejaksaan.go.id)
- **Private Key**: `privkey.pem`
- **Issuer**: DigiCert Global G2 TLS RSA SHA256 2020 CA1
- **Valid Until**: 12 Januari 2026

### 3. Langkah Deployment

#### A. **RECOMMENDED: Deploy Sertifikat Asli (Production Ready)**

Gunakan script otomatis yang sudah disediakan:

```bash
# Deploy sertifikat asli dari /etc/ssl/simpel.kejaksaan.go.id/
cd /srv/proyek/simpelv2/infra/k8s
sudo ./deploy-real-certificate.sh
```

Script ini akan:
1. ✓ Verifikasi file sertifikat dan expiry date
2. ✓ Hapus secret lama dari namespace yang salah
3. ✓ Buat TLS secret baru di namespace `istio-system`
4. ✓ Restart Istio Ingress Gateway
5. ✓ Verifikasi deployment berhasil

Setelah deployment, verifikasi dengan:
```bash
./verify-certificate.sh
```

#### B. Manual Deployment (jika script tidak bisa digunakan)

```bash
# 1. Hapus secret lama jika ada
kubectl delete secret simpelv2-tls-secret -n istio-ingress --ignore-not-found
kubectl delete secret simpelv2-tls-secret -n istio-system --ignore-not-found

# 2. Buat secret dari sertifikat asli
sudo kubectl create secret tls simpelv2-tls-secret \
  --cert=/etc/ssl/simpel.kejaksaan.go.id/fullchain.pem \
  --key=/etc/ssl/simpel.kejaksaan.go.id/privkey.pem \
  -n istio-system

# 3. Verifikasi secret sudah ada
kubectl get secret simpelv2-tls-secret -n istio-system

# 4. Restart Istio Ingress Gateway untuk reload certificate
kubectl rollout restart deployment istio-ingressgateway -n istio-system

# 5. Tunggu hingga pod ready
kubectl rollout status deployment istio-ingressgateway -n istio-system

# 6. Test koneksi
curl -vI https://simpel.kejaksaan.go.id
```

#### C. Untuk Development/Testing (menggunakan self-signed certificate)

**HANYA untuk development lokal, JANGAN untuk production!**

```bash
kubectl apply -f /srv/proyek/simpelv2/infra/k8s/02-regular-secrets.yaml
kubectl rollout restart deployment istio-ingressgateway -n istio-system
```

#### D. Untuk Production dengan Sealed Secrets (Optional - Advanced)

**Hanya jika Anda ingin menggunakan SealedSecrets untuk GitOps workflow**

```bash
# 1. Buat secret dari sertifikat asli (temporary)
sudo kubectl create secret tls simpelv2-tls-secret \
  --cert=/etc/ssl/simpel.kejaksaan.go.id/fullchain.pem \
  --key=/etc/ssl/simpel.kejaksaan.go.id/privkey.pem \
  -n istio-system \
  --dry-run=client -o yaml > tls-secret-temp.yaml

# 2. Seal secret menggunakan kubeseal
kubeseal --format=yaml \
  --cert=pub-cert.pem \
  < tls-secret-temp.yaml \
  > tls-sealed-secret.yaml

# 3. Update encryptedData di 01-sealed-secrets.yaml
# Copy nilai encryptedData dari tls-sealed-secret.yaml ke:
# - Line 310: tls.crt
# - Line 311: tls.key

# 4. Deploy sealed secret
kubectl apply -f /srv/proyek/simpelv2/infra/k8s/01-sealed-secrets.yaml

# 5. Verifikasi secret ter-unseal dengan benar
kubectl get secret simpelv2-tls-secret -n istio-system

# 6. Restart Istio Ingress Gateway
kubectl rollout restart deployment istio-ingressgateway -n istio-system

# 7. Hapus file temporary
rm tls-secret-temp.yaml tls-sealed-secret.yaml
```

### 3. Verifikasi Certificate

```bash
# Check certificate details
echo | openssl s_client -connect simpel.kejaksaan.go.id:443 -servername simpel.kejaksaan.go.id 2>/dev/null | openssl x509 -noout -subject -issuer -dates

# Expected output untuk production:
# subject=CN = simpel.kejaksaan.go.id
# issuer=CN = DigiCert TLS RSA SHA256 2020 CA1, O = DigiCert Inc, C = US
# notBefore=...
# notAfter=...
```

### 4. Troubleshooting

#### Certificate masih menampilkan "Fake Certificate"
```bash
# Check apakah secret ada di namespace yang benar
kubectl get secret simpelv2-tls-secret -n istio-system

# Check isi secret
kubectl get secret simpelv2-tls-secret -n istio-system -o yaml

# Check Gateway configuration
kubectl get gateway simpelv2-gateway -n istio-system -o yaml

# Check Istio Ingress Gateway logs
kubectl logs -n istio-system -l istio=ingressgateway --tail=100
```

#### Secret tidak ter-unseal (untuk sealed secrets)
```bash
# Check sealed-secrets controller logs
kubectl logs -n sealed-secrets -l app.kubernetes.io/name=sealed-secrets-controller

# Verify sealed-secrets controller is running
kubectl get pods -n sealed-secrets
```

#### HSTS Error di Browser
Jika browser masih menampilkan HSTS error setelah certificate diperbaiki:
1. Clear browser cache dan cookies untuk `simpel.kejaksaan.go.id`
2. Chrome: `chrome://net-internals/#hsts` → Delete domain security policies
3. Edge: Settings → Privacy → Clear browsing data → Cached images and files

## Checklist Deployment

- [ ] Namespace TLS secret sudah diperbaiki ke `istio-system`
- [ ] Certificate asli dari DigiCert sudah disiapkan (production)
- [ ] Secret sudah di-deploy di namespace `istio-system`
- [ ] Istio Ingress Gateway sudah di-restart
- [ ] Certificate verification berhasil menampilkan CN yang benar
- [ ] Browser bisa akses https://simpel.kejaksaan.go.id tanpa error
- [ ] HSTS header berfungsi dengan baik

## Catatan Penting

1. **Self-signed certificate** di `02-regular-secrets.yaml` hanya untuk development/testing
2. **Production** harus menggunakan certificate asli dari CA yang trusted (DigiCert)
3. Certificate harus mencakup:
   - `simpel.kejaksaan.go.id` (primary domain)
   - `*.simpel.kejaksaan.go.id` (wildcard untuk subdomain)
4. Pastikan private key tidak pernah di-commit ke git
5. Gunakan sealed-secrets untuk production deployment
