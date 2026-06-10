# SIMPEL Helm RUNBOOK

> Operational runbook untuk deployment, rollback, dan troubleshooting Helm chart `simpel` di staging & production.

## 📑 Daftar Isi

1. [Release Flow (rc → promote)](#0-release-flow-rc--promote)
2. [Fresh Deploy ke Production](#1-fresh-deploy-ke-production)
3. [Deploy / Upgrade ke Staging](#2-deploy--upgrade-ke-staging)
4. [Rollback](#3-rollback)
5. [Manual Trigger Sync Integrasi (staging)](#4-manual-trigger-sync-integrasi-staging)
6. [Rotasi Secret](#5-rotasi-secret)
7. [TLS Cert Renewal (DigiCert)](#6-tls-cert-renewal-digicert)
8. [Bootstrap Secreton (one-shot)](#7-bootstrap-secreton-one-shot)
9. [Troubleshooting](#8-troubleshooting)

---

## 0. Release Flow (rc → promote)

> Pattern: **"build once, promote artifact"**. Image yang lulus QA staging =
> image identik di-deploy production. Cegah cache miss / non-determinism.

### 0.1 Tag pre-release (rc)

```bash
# Sesudah feature merged ke main, tag rc
git checkout main && git pull
git tag -a v0.1.0-rc1 -m "Release candidate 1 untuk v0.1.0"
git push origin v0.1.0-rc1
# Trigger release.yml otomatis: build & push 7 image dengan tag v0.1.0-rc1
gh run watch  # monitor
```

7 image akan ter-publish ke `ghcr.io/<owner>/simpel2/<svc>:v0.1.0-rc1`.

### 0.2 Deploy staging dengan tag rc

```bash
helm upgrade --install simpel infra/helm/simpel \
  -f infra/helm/simpel/values-staging.yaml \
  --namespace simpelv2-staging --create-namespace \
  --atomic --timeout 15m \
  --set global.imageTag=v0.1.0-rc1
```

Smoke test, manual sync integrasi, Playwright e2e (lihat [§2](#2-deploy--upgrade-ke-staging)
dan [§4](#4-manual-trigger-sync-integrasi-staging)).

### 0.3 Promote rc → final tag (kalau staging hijau)

```bash
# Trigger workflow promote-release.yml via gh CLI
gh workflow run promote-release.yml \
  -f source_tag=v0.1.0-rc1 \
  -f target_tag=v0.1.0 \
  -f git_tag=y

gh run watch
```

Workflow akan:

1. Validasi format tag (rc → final dengan base version match).
2. Re-tag image di ghcr.io via `docker buildx imagetools create` (TANPA rebuild).
3. Verify digest source ↔ target identik per image.
4. Create git tag `v0.1.0` di commit yang sama dengan `v0.1.0-rc1`.
5. Publish GitHub Release.

Image `:v0.1.0` sekarang punya digest **identik** dengan `:v0.1.0-rc1`.

### 0.4 Deploy production dengan tag final

```bash
helm install simpel infra/helm/simpel \
  -f infra/helm/simpel/values-production.yaml \
  --namespace simpelv2-production \
  --atomic --timeout 15m \
  --set global.imageTag=v0.1.0
```

Image yang jalan di production **identik** (digest sama) dengan yang sudah lulus
staging — zero risk regresi karena rebuild.

### 0.5 Kalau staging gagal

```bash
# Fix bug → push ke main → tag rc baru
git tag -a v0.1.0-rc2 -m "rc2: fix <issue>"
git push origin v0.1.0-rc2
# Ulang §0.1-0.4 dengan rc2.
# v0.1.0-rc1 di ghcr.io biarkan; tidak di-promote ke v0.1.0.
```

Lihat [Lihat lebih jauh §1 untuk first-time production deploy.](#1-fresh-deploy-ke-production)

---

## 1. Fresh Deploy ke Production

> Asumsi: cluster production fresh, belum ada release sebelumnya. Image `v0.1.0` sudah ter-push ke ghcr.io.

### 1.1 Pre-flight

```bash
kubectl config use-context <production-context>
kubectl get nodes -o wide
```

### 1.2 Buat namespace & ghcr-pull secret

```bash
kubectl create namespace simpelv2-production
kubectl -n simpelv2-production create secret docker-registry ghcr-pull \
  --docker-server=ghcr.io \
  --docker-username=<github-user> \
  --docker-password=<PAT-with-read:packages> \
  --docker-email=<email>
```

### 1.3 Apply DigiCert TLS

```bash
openssl x509 -in /path/to/digicert-fullchain.pem -noout -subject -issuer -dates
# Verify: subject CN matches simpel.kejaksaan.go.id, NotAfter > 30 days

kubectl -n istio-system create secret tls simpelv2-tls-secret \
  --cert=/path/to/digicert-fullchain.pem \
  --key=/path/to/private-key.pem \
  --dry-run=client -o yaml | kubectl apply -f -
```

### 1.4 Pre-render & validate

```bash
./infra/helm/deploy.sh production template > /tmp/prod-rendered.yaml
helm lint infra/helm/simpel -f infra/helm/simpel/values-production.yaml
# Optional: kubeconform -strict -summary /tmp/prod-rendered.yaml
```

### 1.5 Helm install (atomic)

```bash
helm install simpel infra/helm/simpel \
  -f infra/helm/simpel/values-production.yaml \
  --namespace simpelv2-production \
  --atomic --timeout 15m
```

### 1.6 Verifikasi rollout

```bash
kubectl -n simpelv2-production rollout status deploy --timeout=15m
kubectl -n simpelv2-production rollout status statefulset --timeout=15m
kubectl -n simpelv2-production get pods,svc,vs,gw,dr,pdb,hpa,networkpolicy
```

### 1.7 Bootstrap Secreton

Lihat [§7 Bootstrap Secreton](#7-bootstrap-secreton-one-shot).

### 1.8 Flip secretonAuth ON

```bash
helm upgrade simpel infra/helm/simpel \
  -f infra/helm/simpel/values-production.yaml \
  --namespace simpelv2-production \
  --set secretonAuth.enabled=true \
  --atomic --timeout 10m
```

### 1.9 Smoke test

```bash
curl -sf https://simpel.kejaksaan.go.id/api/v1/auth/health
curl -sf https://simpel.kejaksaan.go.id/api/v1/perlengkapan/health
curl -sf https://simpel.kejaksaan.go.id/portal/ | grep -q '<title>'
```

---

## 2. Deploy / Upgrade ke Staging

```bash
kubectl config use-context <staging-context>
./infra/helm/deploy.sh staging template > /tmp/staging-rendered.yaml
helm diff upgrade simpel infra/helm/simpel \
  -f infra/helm/simpel/values-staging.yaml \
  --namespace simpelv2-staging || true

helm upgrade --install simpel infra/helm/simpel \
  -f infra/helm/simpel/values-staging.yaml \
  --namespace simpelv2-staging --create-namespace \
  --atomic --timeout 15m \
  --history-max 10
```

CronJob staging by default `suspend: true`. Untuk testing, lihat [§4 Manual Trigger Sync](#4-manual-trigger-sync-integrasi-staging).

---

## 3. Rollback

```bash
# Lihat history
helm history simpel -n simpelv2-<env>

# Rollback ke revision sebelumnya
helm rollback simpel <REVISION> -n simpelv2-<env> --wait

# Last resort: uninstall + reinstall (state lost!)
helm uninstall simpel -n simpelv2-<env>
kubectl delete namespace simpelv2-<env>   # ⚠️ destroys PVCs
```

---

## 4. Manual Trigger Sync Integrasi (staging)

Staging CronJob `suspend: true` (cegah rate-limit token API). Untuk testing UI/API yang butuh data segar:

```bash
# Trigger MySIMKARI (paling cepat)
kubectl -n simpelv2-staging create job \
  --from=cronjob/layanan-integrasi-mysimkari \
  manual-mysimkari-$(date +%s)

# Tunggu selesai
kubectl -n simpelv2-staging wait \
  --for=condition=complete \
  job/manual-mysimkari-XXXXX \
  --timeout=10m

# Cek log
kubectl -n simpelv2-staging logs \
  -l job-name=manual-mysimkari-XXXXX --tail=200
```

Verifikasi data masuk DB:

```bash
kubectl -n simpelv2-staging exec postgres-0 -- \
  psql -U postgres -d integrasi -c \
  "SELECT module, endpoint, last_success_at, row_count FROM v_recent_successful_calls ORDER BY last_success_at DESC LIMIT 5;"
```

---

## 5. Rotasi Secret

### 5.1 Token API eksternal (SIMAN/MonSAKTI/MySIMKARI)

```bash
SECRETON_TOKEN=<service-token-with-write> \
  ./scripts/migrate-env-to-secreton.sh \
  --env-file <(echo "SIMAN_CLIENT_SECRET=<new-secret>") \
  --kv-prefix integrasi/tokens/siman
```

CronJob berikutnya akan pakai secret baru otomatis (token TTL 1 jam, renewal pull config terbaru).

### 5.2 simpelv1 APP_KEY

```bash
NEW_KEY=$(php artisan key:generate --show)   # lokal, jangan apply ke .env
secreton kv put kv/simpelv1/app app_key="${NEW_KEY#base64:}"
kubectl -n simpelv2-production rollout restart deploy/simpelv1
```

⚠️ **Hati-hati**: rotasi APP_KEY invalidate semua user session.

### 5.3 PostgreSQL password

Multi-step (butuh koordinasi):

1. `ALTER USER <db-user> WITH PASSWORD '<new>';` di Postgres.
2. Push ke Secreton: `secreton kv put kv/postgres/<db> password="<new>" username="<user>"`.
3. Rolling restart consumer service: `kubectl rollout restart deploy/<service>`.

---

## 6. TLS Cert Renewal (DigiCert)

DigiCert cert berlaku 1 tahun. Reminder rotate H-30 sebelum expire.

```bash
# Verify current expiry
kubectl -n istio-system get secret simpelv2-tls-secret -o jsonpath='{.data.tls\.crt}' \
  | base64 -d | openssl x509 -noout -dates

# Apply new PEM (Istio gateway hot-reload via SDS, no restart needed)
kubectl -n istio-system create secret tls simpelv2-tls-secret \
  --cert=/path/to/new-fullchain.pem \
  --key=/path/to/new-private-key.pem \
  --dry-run=client -o yaml | kubectl apply -f -

# Verify
curl -vI https://simpel.kejaksaan.go.id 2>&1 | grep -E '(SSL|expire)'
```

---

## 7. Bootstrap Secreton (one-shot)

```bash
# 7.1 Pastikan secreton pod sudah running tapi sealed
kubectl -n simpelv2-<env> get pod -l app.kubernetes.io/name=secreton

# 7.2 Init Secreton (sekali per fresh cluster)
kubectl -n simpelv2-<env> exec secreton-0 -- \
  secreton operator init -shamir-shares=5 -shamir-threshold=3
# OUTPUT: Simpan 5 unseal keys & root token OFFLINE.
# Distribusi keys ke 5 holder berbeda (Shamir).

# 7.3 Unseal × 3
for key in <key1> <key2> <key3>; do
  kubectl -n simpelv2-<env> exec secreton-0 -- \
    secreton operator unseal "$key"
done

# 7.4 Migrasi .env → Secreton (sekali per file)
SECRETON_TOKEN=<root-from-step-2> \
  ./scripts/migrate-env-to-secreton.sh \
  --env-file layanan/integrasi/.env \
  --kv-prefix integrasi/tokens

# 7.5 Bootstrap auth backend & policies + roles
SECRETON_TOKEN=<root-from-step-2> \
  ./infra/helm/bootstrap-secreton.sh <staging|production>

# 7.6 Verify
kubectl -n simpelv2-<env> exec secreton-0 -- \
  secreton auth list | grep kubernetes
kubectl -n simpelv2-<env> exec secreton-0 -- \
  secreton policy list

# 7.7 Revoke root token (cleanup)
SECRETON_TOKEN=<root> \
  ./infra/helm/bootstrap-secreton.sh <env> --revoke-root
```

---

## 8. Troubleshooting

### 8.1 Pod CrashLoopBackOff

```bash
kubectl -n simpelv2-<env> logs <pod> --previous --tail=100
kubectl -n simpelv2-<env> describe pod <pod>
kubectl -n simpelv2-<env> get events --sort-by='.lastTimestamp' | tail -50
```

Common causes:

- Image pull error → cek `ghcr-pull` secret di namespace.
- Secret fetch error → cek Secreton sealed/unsealed status, cek SA annotation.
- Health probe failed → cek path & port di values, lihat container log.

### 8.2 Helm install / upgrade gagal

```bash
helm history simpel -n simpelv2-<env>
helm get values simpel -n simpelv2-<env>
helm get manifest simpel -n simpelv2-<env> | head -100
```

Jika `--atomic` rollback otomatis: cek `kubectl events` untuk root cause.

### 8.3 Secreton sealed setelah restart

```bash
kubectl -n simpelv2-<env> exec secreton-0 -- secreton status
# Jika Sealed: true, unseal × 3 (lihat §7.3)
```

### 8.4 Token API rate-limit

Cek log integrasi:

```bash
kubectl -n simpelv2-production logs deploy/layanan-integrasi --tail=200 | grep -iE 'rate|429|throttle'
```

Action: tunda manual trigger staging, contact provider untuk increase quota.

### 8.5 Frontend not reachable

```bash
kubectl get gw,vs,dr -A
kubectl -n istio-system get pods -l app=istio-ingressgateway
# Cek listener
kubectl -n istio-system exec deploy/istio-ingressgateway -- \
  pilot-agent request GET listeners | jq '.[] | select(.name == "https.443.https")'
```

---

## Referensi

- [ADR-0001 Secret Zero-Trust](../../docs/adr/0001-secret-management-zero-trust.md)
- [ADR-0002 Helm Refactor](../../docs/adr/0002-helm-refactor.md)
- [infra/AGENTS.md](../AGENTS.md)
- [Plan deployment](../../.claude/plans/build-dan-deploy-antarmuka-portal-peaceful-cocke.md) — internal
