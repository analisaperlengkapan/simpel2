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
9. [Backup & Restore (Velero + MinIO)](#8-backup--restore-velero--minio)
10. [Troubleshooting](#9-troubleshooting)

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

**Prasyarat klaster (sekali per klaster, bukan per-environment).** StorageClass
`longhorn-retain` harus ada SEBELUM `helm install`, karena `volumeClaimTemplates`
StatefulSet immutable — kalau PVC production lahir di kelas yang salah,
memperbaikinya berarti membongkar volume, bukan `helm upgrade`.

```bash
./infra/helm/deploy.sh production storage-install
kubectl get sc longhorn-retain \
  -o custom-columns='NAME:.metadata.name,RECLAIM:.reclaimPolicy,REPL:.parameters.numberOfReplicas'
# harapan: longhorn-retain   Retain   2
```

Sekalian rapikan setelan replika global Longhorn agar `degraded` kembali bermakna
di klaster 2-node (lihat `infra/AGENTS.md` → "Ketersediaan database di klaster
2-node"). Tanpa ini setiap volume permanen `degraded` dan status itu berhenti
membedakan sehat vs rusak:

```bash
# Default untuk volume baru yang kelasnya TIDAK memaksakan angka sendiri.
kubectl -n longhorn-system patch setting default-replica-count \
  --type=merge -p '{"value":"{\"v1\":\"2\",\"v2\":\"2\"}"}'

# volume yang SUDAH ada (aman: tiap volume sudah punya 2 replika sehat,
# satu per node; yang dilepas hanya replika ketiga yang tak pernah terjadwal)
for v in $(kubectl -n longhorn-system get volumes.longhorn.io -o name); do
  kubectl -n longhorn-system patch "$v" --type=merge -p '{"spec":{"numberOfReplicas":2}}'
done

kubectl -n longhorn-system get volumes.longhorn.io \
  -o custom-columns='NAME:.metadata.name,ROBUST:.status.robustness,REPL:.spec.numberOfReplicas'
# harapan: semua `healthy`, REPL=2
```

> ⚠️ **Setelan global itu TIDAK menjangkau StorageClass `longhorn` bawaan.** Kelas
> itu menuliskan `parameters.numberOfReplicas: "3"` pada dirinya sendiri, dan
> parameter kelas selalu menang atas `default-replica-count`. Jadi volume baru
> yang lahir di kelas default tetap meminta 3 dan tetap lahir `degraded` di
> klaster 2-node. Kelas itu milik rilis Helm `longhorn` — mengeditnya di sini
> akan tertimpa saat Longhorn di-upgrade. Perbaikannya bukan menambal kelas
> bawaan, melainkan **memakai `longhorn-retain`** (yang menuliskan 2 pada dirinya
> sendiri) untuk semua volume SIMPel.
>
> Terverifikasi di klaster 2026-08-18: setelah kedua perintah di atas, ketiga
> volume staging jadi `healthy` dengan 2 replika, tetap tersebar satu per node.

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

# Port-forward dulu (tak ada CLI di image — lihat §7).
kubectl -n simpelv2-production port-forward svc/secreton 8200:8200 >/dev/null 2>&1 &
PF=$!; trap 'kill $PF' EXIT

# Path & nama field HARUS persis seperti yang dibaca fetch-secrets.sh:
# `simpelv1/app` → field `app_key` (tanpa prefix `base64:`).
jq -nc --arg v "${NEW_KEY#base64:}" '{data:{app_key:$v}}' \
  | curl -fsS -X PUT http://127.0.0.1:8200/v1/secret/data/simpelv1/app \
      -H "X-Secreton-Token: $SECRETON_TOKEN" \
      -H 'Content-Type: application/json' --data @-

# Verifikasi tulisan mendarat sebelum restart.
curl -fsS -H "X-Secreton-Token: $SECRETON_TOKEN" \
  http://127.0.0.1:8200/v1/secret/data/simpelv1/app | jq '.data.data | keys'

kubectl -n simpelv2-production rollout restart deploy/simpelv1
```

⚠️ **Hati-hati**: rotasi APP_KEY invalidate semua user session.

### 5.3 PostgreSQL password

Multi-step (butuh koordinasi):

1. `ALTER USER <db-user> WITH PASSWORD '<new>';` di Postgres.
2. Push ke Secreton. Path yang dibaca `fetch-secrets.sh` = `postgres/simpelv2`,
   field `username` + `password`. **Tulis KEDUA field sekaligus** — `PUT`
   mengganti seluruh objek `data`, jadi mengirim `password` saja akan
   menghilangkan `username` dan pod gagal render `.env`:

   ```bash
   jq -nc --arg u "<user>" --arg p "<new>" '{data:{username:$u,password:$p}}' \
     | curl -fsS -X PUT http://127.0.0.1:8200/v1/secret/data/postgres/simpelv2 \
         -H "X-Secreton-Token: $SECRETON_TOKEN" \
         -H 'Content-Type: application/json' --data @-

   curl -fsS -H "X-Secreton-Token: $SECRETON_TOKEN" \
     http://127.0.0.1:8200/v1/secret/data/postgres/simpelv2 | jq '.data.data | keys'
   ```

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

# 7.2 Port-forward. TIDAK ADA biner `secreton` di image (cuma `api_server`) —
#     semua operasi lewat REST :8200. Port-forward, bukan `kubectl exec`, supaya
#     key material tidak melewati scrollback exec dan supaya jq ada di tangan.
kubectl -n simpelv2-<env> port-forward svc/secreton 8200:8200 >/dev/null 2>&1 &
PF=$!; trap 'kill $PF' EXIT
until curl -fsS http://127.0.0.1:8200/v1/sys/seal-status >/dev/null 2>&1; do sleep 1; done
curl -fsS http://127.0.0.1:8200/v1/sys/seal-status | jq   # pastikan initialized=false

# 7.3 Init Secreton (sekali per fresh cluster)
curl -fsS -X POST http://127.0.0.1:8200/v1/sys/init \
  -H 'Content-Type: application/json' \
  -d '{"secret_shares":5,"secret_threshold":3}' | jq
# OUTPUT: Simpan 5 unseal keys & root token OFFLINE — satu-satunya saat keduanya
# ada, tidak bisa dipulihkan. Distribusi keys ke 5 holder berbeda (Shamir).

# 7.4 Unseal × 3 (threshold)
for key in <key1> <key2> <key3>; do
  curl -fsS -X POST http://127.0.0.1:8200/v1/sys/unseal \
    -H 'Content-Type: application/json' \
    -d "$(jq -nc --arg k "$key" '{key:$k}')" | jq -r '.sealed'
done
curl -fsS http://127.0.0.1:8200/v1/sys/seal-status | jq -r '.initialized, .sealed'

# 7.5 Migrasi .env → Secreton (sekali per file)
SECRETON_TOKEN=<root-from-7.3> \
  ./layanan/secreton/scripts/migrate-env-to-secreton.sh \
  --env-file layanan/integrasi/.env \
  --kv-prefix integrasi/tokens

# 7.6 Bootstrap auth backend & policies + roles
#     (script ini sudah pakai port-forward + curl /v1/... sendiri)
SECRETON_TOKEN=<root-from-7.3> \
  ./infra/helm/bootstrap-secreton.sh <staging|production>

# 7.7 Verify (REST)
curl -fsS -H "X-Secreton-Token: <root>" http://127.0.0.1:8200/v1/sys/auth | jq 'keys'
curl -fsS -H "X-Secreton-Token: <root>" http://127.0.0.1:8200/v1/sys/policies | jq

# 7.8 Revoke root token (cleanup)
SECRETON_TOKEN=<root> \
  ./infra/helm/bootstrap-secreton.sh <env> --revoke-root
```

---

## 8. Backup & Restore (Velero + MinIO)

> **Backup yang belum pernah di-restore belum terbukti jadi backup.** Bagian
> restore di bawah bukan lampiran — ia bagian dari pemasangan.

Arsitektur: Velero (namespace `velero`) menulis ke MinIO (namespace
`simpelv2-backup`) lewat API S3. Datanya disalin per-file dengan kopia
(File System Backup), bukan snapshot CSI — snapshot CSI butuh
snapshot-controller + VolumeSnapshotClass Longhorn yang belum terpasang.

Namespace object store sengaja terpisah dari namespace aplikasi. Kegagalan yang
sudah terbukti terjadi di klaster ini menghabisi **satu** namespace; backup yang
tinggal di namespace yang sama akan ikut mati bersama data yang seharusnya ia
selamatkan.

**Batasnya, ditulis supaya tak dilupakan:** MinIO ini berdiri di atas disk
klaster yang sama. Ia melindungi dari kesalahan operasi dan penghapusan objek
Kubernetes, **tidak** dari hilangnya kedua node atau site. Pindah ke object store
di luar klaster = ubah `configuration.backupStorageLocation` di
`infra/velero/values.yaml` lalu jalankan ulang `install.sh`.

### 8.1 Pasang (sekali per klaster)

```bash
# 1. Kredensial MinIO — bangkitkan, jangan karang.
cp infra/helm/backup/values-secrets.example.yaml infra/helm/backup/values-secrets.yaml
openssl rand -base64 36          # tempel ke rootPassword; file ini ter-gitignore

# 2. Object store + bucket
./infra/helm/deploy.sh production backup-install \
  -f infra/helm/backup/values-secrets.yaml --set secrets.bootstrap=true
kubectl -n simpelv2-backup rollout status statefulset/minio --timeout=5m
kubectl -n simpelv2-backup get job minio-bucket-init

# 3. Velero (pakai kredensial yang SAMA)
MINIO_ROOT_USER=<user> MINIO_ROOT_PASSWORD=<password> ./infra/velero/install.sh
```

`install.sh` berhenti dengan galat kalau BackupStorageLocation tidak mencapai
`Available`. Itu disengaja: BSL yang `Unavailable` membuat SETIAP backup gagal,
dan bentuk kegagalan itu mudah tak terlihat karena yang merah adalah objek
Velero, bukan deploy-nya.

### 8.2 Backup manual sebelum operasi berisiko

Wajib sebelum `helm uninstall`, migrasi besar, atau uji destruktif.

```bash
velero backup create pra-<alasan>-$(date +%Y%m%d-%H%M) \
  --include-namespaces simpelv2-production --wait
velero backup describe <nama> --details
```

Perhatikan `Phase`. `PartiallyFailed` **bukan** sukses — paling sering artinya
hook `pg_dumpall` gagal, jadi yang tersimpan hanya direktori data
crash-consistent tanpa dump logis.

### 8.3 Verifikasi terjadwal

```bash
velero schedule get
velero backup get
kubectl -n velero get backupstoragelocation default \
  -o custom-columns='NAME:.metadata.name,PHASE:.status.phase,LAST:.status.lastValidationTime'
```

Jadwal: staging 01:00 WIB (simpan 14 hari), production 00:00 WIB (simpan 30 hari).

### 8.4 Latihan restore (WAJIB, dan wajib DIULANG)

Restore ke namespace **baru**, jangan ke namespace hidup — tujuannya membuktikan
backup-nya utuh, bukan mempertaruhkan yang sedang berjalan.

```bash
velero restore create drill-$(date +%Y%m%d) \
  --from-backup <nama-backup> \
  --namespace-mappings simpelv2-production:simpelv2-restore-drill --wait

velero restore describe drill-<tanggal> --details
kubectl -n simpelv2-restore-drill get pods,pvc

# Bukti yang sebenarnya: datanya ADA, bukan sekadar pod-nya Running.
kubectl -n simpelv2-restore-drill exec statefulset/postgres -- \
  psql -U <POSTGRES_USER> -d dbsimpelv2 -c '\dt perlengkapan.*'
kubectl -n simpelv2-restore-drill exec statefulset/postgres -- \
  ls -la /var/lib/postgresql/data/backup/dumpall.sql

# Bersihkan setelah selesai.
kubectl delete namespace simpelv2-restore-drill
```

Catat tanggal latihan terakhir. Backup yang terakhir diuji berbulan-bulan lalu
adalah asumsi, bukan jaminan.

**Latihan terakhir: 2026-08-18 — LULUS.** Bukti, bukan kesan:

| Yang diuji | Hasil |
|---|---|
| Backup `simpelv2-staging` | `Completed`, 139/139 item, 0 error, 0 warning |
| Restore → `simpelv2-restore-drill` | PVC ter-bind ke PV BARU, ukuran benar (20/5/10Gi) |
| Database yang kembali | `dbsimpelv1`, `dbsimpelv2`, `postgres`, `secreton` |
| Jumlah baris vs sumber hidup | `authenc.users` 2, `mysimkari_satker` 539, `siman_aset` **624.528**, 47 tabel `perlengkapan` — **identik** |
| Checksum isi (md5 atas 539 baris satker) | `34335b79…` di **kedua** sisi |
| Staging setelah drill dibongkar | 12 pod Running, 3 PV asli utuh |

**Restore dilaporkan `PartiallyFailed`, dan itu BUKAN kehilangan data.** 19 dari 74
PodVolumeRestore dibatalkan; semuanya volume scratch (`istio-envoy`,
`nginx-cache`, `var-run`, …) yang pod pembantunya ditolak saat klaster sibuk.
Nol volume data gagal. Sejak itu chart mengecualikan seluruh emptyDir dari
File System Backup dan CI menjaganya (`infra/scripts/check-velero-volume-excludes.py`),
supaya `PartiallyFailed` kembali berarti "ada yang salah".

**Yang BELUM diuji** — jangan mengaku lebih dari yang dibuktikan:

- restore ke namespace ASLI (bencana sungguhan), bukan ke namespace drill;
- unseal Secreton setelah restore;
- restore production (belum pernah ada backup production).

Jadwalkan ketiganya di jendela uji destruktif F5-E.

### 8.5 Restore sungguhan

Sama seperti latihan, tanpa `--namespace-mappings`, dan **hanya** setelah
namespace tujuan benar-benar kosong (Velero tidak menimpa objek yang sudah ada).
Untuk kehilangan data logis (bukan hilangnya seluruh namespace), lebih cepat dan
lebih aman: restore ke namespace drill, ambil `dumpall.sql` dari sana, `psql`
kembali ke database yang hidup.

---

## 9. Troubleshooting

### 9.1 Pod CrashLoopBackOff

```bash
kubectl -n simpelv2-<env> logs <pod> --previous --tail=100
kubectl -n simpelv2-<env> describe pod <pod>
kubectl -n simpelv2-<env> get events --sort-by='.lastTimestamp' | tail -50
```

Common causes:

- Image pull error → cek `ghcr-pull` secret di namespace.
- Secret fetch error → cek Secreton sealed/unsealed status, cek SA annotation.
- Health probe failed → cek path & port di values, lihat container log.

### 9.2 Helm install / upgrade gagal

```bash
helm history simpel -n simpelv2-<env>
helm get values simpel -n simpelv2-<env>
helm get manifest simpel -n simpelv2-<env> | head -100
```

Jika `--atomic` rollback otomatis: cek `kubectl events` untuk root cause.

### 9.3 Secreton sealed setelah restart

```bash
kubectl -n simpelv2-<env> exec secreton-0 -- secreton status
# Jika Sealed: true, unseal × 3 (lihat §7.3)
```

### 9.4 Token API rate-limit

Cek log integrasi:

```bash
kubectl -n simpelv2-production logs deploy/layanan-integrasi --tail=200 | grep -iE 'rate|429|throttle'
```

Action: tunda manual trigger staging, contact provider untuk increase quota.

### 9.5 Frontend not reachable

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
