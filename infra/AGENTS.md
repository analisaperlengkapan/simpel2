# 🤖 AGENTS.md - Infrastruktur & DevOps

> **Notice to Agents**: File ini adalah pedoman (Level 2 Archetype) untuk semua operasi terkait Infrastruktur, Deployment, dan DevOps di SIMPEL.

## 📑 Daftar Isi

1. 🗺️ Domain Routing
2. 🌍 Strategi Infrastruktur Global
3. 🔐 Secret Management Zero-Trust (Secreton + Kubernetes Auth)
4. 🔒 TLS Certificate (DigiCert)
5. ⏰ CronJob Schedule (layanan-integrasi)
6. ⚠️ Aturan AI untuk Operasi Infrastruktur

## 🗺️ Domain Routing

- ☸️ **Kubernetes & Deployments (Helm)**: lihat [`infra/helm/README.md`](helm/README.md).
- 📦 **MetalLB IP pools**: lihat [`infra/helm/metallb/`](helm/metallb/).
- 📊 **Monitoring (Prometheus/Grafana)**: ServiceMonitor dirender oleh chart `simpel` saat `monitoring.serviceMonitors.enabled=true`.
- 🌐 **Nginx/Ingress**: routing eksternal lewat Istio Gateway/VirtualService (di-template oleh chart). Sidecar nginx hanya dipakai sebagai static-file server di portal & perlengkapan.

## 🌍 Strategi Infrastruktur Global

Infrastruktur SIMPEL berfokus pada **Keamanan Tingkat Tinggi (Zero-Trust)** dan **Ketersediaan (High Availability)**.

### 1. Prinsip Zero-Trust

- Komunikasi internal antar-layanan WAJIB pakai mTLS (Istio PeerAuthentication; production = STRICT).
- Secret tidak boleh masuk git plain-text. Chart `simpel` **default-nya tidak merender Secret object** (`secrets.bootstrap=false`). Workflow:
  - **Staging**: bootstrap satu kali via `values-secrets.yaml` (gitignored), lalu rotasi manual atau via Secreton SecretSync.
  - **Production**: WAJIB Secreton SecretSync CRD (lihat `layanan/secreton/crates/k8s-operator/README.md`).

### 2. GitOps & Helm

- Seluruh manifest Kubernetes dideklarasikan oleh chart Helm di `infra/helm/`.
  - `infra/helm/simpel/` — chart aplikasi (umbrella).
  - `infra/helm/metallb/` — chart cluster-scoped untuk MetalLB.
- DILARANG mengubah resource langsung di cluster (`kubectl edit`, `kubectl patch`). Semua perubahan WAJIB melalui `values-<env>.yaml` atau template di repo ini, lalu `helm upgrade`.
- Per-MR review apps memakai `./infra/helm/deploy.sh review-<slug> install` (lihat `.gitlab-ci.yml` job `deploy:review`).

### 3. Pemisahan Lingkungan

- **Staging** (`simpelv2-staging`): 1 replica, log `debug`, mTLS PERMISSIVE, image tag **semver immutable** (`v0.1.0`), semua pod di-pin ke node `simple02`.
- **Production** (`simpelv2-production`): 3 replica HA, log `info`, mTLS STRICT, ResourceQuota & LimitRange aktif, image tag **semver immutable** (`v0.1.0` / `vMAJOR.MINOR.PATCH`), scheduling fleksibel, HTTPS redirect on.

#### Alur deploy WAJIB: staging → promote → production (DILARANG prod-only)

> **Aturan tetap (berlaku untuk AI & kontributor):** JANGAN pernah deploy/`helm upgrade` langsung ke `simpelv2-production` tanpa lebih dulu lolos di `simpelv2-staging`. `simpel.kejaksaan.go.id` adalah sistem pemerintah — perubahan harus tervalidasi di staging dulu.

Urutan baku:

1. **Build & push image** via tag git `v*.*.*` → `release.yml` (semver immutable, SBOM + provenance). Untuk RC pakai suffix `-rcN`.
2. **Deploy STAGING**: `helm upgrade` `simpel` di `simpelv2-staging` (`values-staging.yaml`).
3. **Uji STAGING**: smoke + Playwright e2e + `cargo test` terhadap staging. Bila gagal → perbaiki, naikkan RC, ulang dari (1).
4. **Promote**: `promote.yml` me-_retag_ image **yang sama** (digest identik, **tanpa rebuild**) dari `-rcN` → tag final.
5. **Deploy PRODUCTION**: `helm upgrade` `simpel` di `simpelv2-production` (`values-production.yaml`) dengan tag final.
6. **Uji PRODUCTION**: smoke + e2e. Bila regresi → `helm rollback`.

Prasyarat production sekali-jalan: bootstrap+unseal Secreton, cert DigiCert di `istio-system`, MetalLB IP pool (lihat section terkait di bawah).

### 4. Image Registry & Tag

- **Registry resmi**: `ghcr.io/analisaperlengkapan/simpel2/<service>`. Pull dengan k8s Secret `ghcr-pull` (docker-registry type) per namespace; PAT scope `read:packages`.
- **Tag**: WAJIB SemVer `vMAJOR.MINOR.PATCH` (mis. `v0.1.0`). DILARANG mutable tag (`latest`, `stag`, `prod`, kosong) — schema `values.schema.json` reject saat `helm lint`.
- **`imagePullPolicy`**: `IfNotPresent` di staging & production (combined with immutable tag). `Never` di-deprecate (legacy era `localhost:32000` registry).
- **Build**: Tag git `v*.*.*` → `release.yml` GitHub Actions matrix build & push 7 image (portal, perlengkapan, authenc, layanan-integrasi, layanan-perlengkapan, secreton, simpelv1) dengan SBOM + provenance.

## 🔐 Secret Management Zero-Trust (Secreton + Kubernetes Auth)

> **Source of truth produksi & staging**: `kv/<service>/...` di Secreton. **DILARANG** menyimpan secret di k8s Secret object atau `.env` di repo.

### Arsitektur (PR 3, default OFF, opt-in via `secretonAuth.enabled=true` di values)

- Setiap service punya **ServiceAccount** di namespace, dengan annotation `secreton.simpel.io/role: <name>`.
- Pod project SA token via `projected.serviceAccountToken` (audience `secreton`, TTL 3600s) ke `/var/run/secrets/tokens/secreton-token`.
- Service authenticate ke Secreton via `POST /v1/auth/kubernetes/login` dengan JWT projected token → terima Secreton client token (renewable).
- Service fetch secret pakai client token tsb dari path yang diizinkan policy (lihat `values.yaml` `secretonAuth.policies`).
- **simpelv1 (Laravel/PHP)** tidak punya client native → init container `fetch-secrets` (script di `infra/helm/simpel/files/fetch-secrets.sh`) generate `/app/runtime/.env` dari Secreton sebelum main container start.

### Bootstrap (sekali per environment)

1. Helm install dengan `secretonAuth.enabled=false` dulu (default).
2. Init Secreton: `kubectl exec secreton-0 -- secreton operator init -shamir-shares=5 -shamir-threshold=3` (simpan unseal keys offline).
3. Unseal: `secreton operator unseal <key>` × 3.
4. Migrasi `.env` ke Secreton: `SECRETON_TOKEN=<root> ./scripts/migrate-env-to-secreton.sh --env-file <path> --kv-prefix <prefix>`.
5. Bootstrap auth backend + role + policy: `SECRETON_TOKEN=<root> ./infra/helm/bootstrap-secreton.sh <staging|production>`.
6. Flip `secretonAuth.enabled=true` & `helm upgrade` → service mulai pakai SA token, k8s Secret `simpelv1-secrets` & `integration-secrets` di-deprecate.
7. Revoke root token: `./infra/helm/bootstrap-secreton.sh <env> --revoke-root`.

### Yang masih k8s Secret (unavoidable)

- `ghcr-pull` (image pull) — kubelet butuh sebelum pod start.
- `simpel-tls` di `istio-system` — Istio gateway baca dari k8s Secret.
- Bootstrap unseal keys → **TIDAK** disimpan di k8s; offline-only (password manager / KMS terpisah).

### Rotasi Secret Runtime (F2H 2H-B)

Tujuan: kredensial berputar **tanpa** secret statik berumur panjang, dan rotasi
tidak mematikan layanan.

- **Rust services (perlengkapan dulu, pola sama utk lainnya) — dynamic DB
  credentials (Vault-style lease).** Set env `SECRETON_DB_ROLE=<role>` (opsional
  `SECRETON_DB_TTL_SECONDS`) → saat start service memanggil
  `GenerateDatabaseCredentials` → bangun pool dari DSN dinamis. Background task
  memperpanjang lease (`RenewLease`) di **½ TTL**; perpanjangan menjaga role
  Postgres yang sama tetap valid sehingga pool jalan terus **tanpa swap** (tiap
  service memegang clone `Pool`, jadi hot-swap in-process = rewrite besar — sengaja
  dihindari). Di **batas rotasi keras** (max renewals tercapai / renew gagal)
  proses **exit** → Kubernetes restart pod → `main` ambil kredensial baru.
  **Prasyarat F6 (secreton-side):** `ConfigureDatabaseConnection` + `CreateDatabaseRole`
  untuk role tsb, lalu set `SECRETON_DB_ROLE` di `values.yaml` (`<svc>.env`).
  Tanpa `SECRETON_DB_ROLE` → tetap pakai `DATABASE_URL` statik (perilaku lama).
- **simpelv1 (PHP-FPM, env tak hot-reload).** Dua lapis: (1) kredensial yang
  di-fetch runtime via gateway/sidecar tetap on-demand; (2) secret boot (APP_KEY,
  DB via init container `fetch-secrets.sh`) → **rolling-restart-on-rotation** dgn
  **Stakater Reloader**. **Prasyarat F6:** install controller Reloader, lalu
  anotasi Deployment simpelv1 `secret.reloader.stakater.com/reload: "simpelv1-secrets"`
  agar pod restart otomatis saat k8s Secret `simpelv1-secrets` berubah.
- **⚠️ Asimetri APP_KEY (simpelv1).** Rotasi `APP_KEY` Laravel **membatalkan
  sesi Redis terenkripsi** (semua user ter-logout) — beda dgn DB/SMTP yang rotasi-nya
  transparan. Jadwalkan saat maintenance, atau pakai strategi dua-kunci
  (`APP_PREVIOUS_KEYS`) bila perlu zero-logout.

## 🔒 TLS Certificate (DigiCert)

- **Source**: Sertifikat DigiCert untuk `simpel.kejaksaan.go.id` disimpan di luar repo (private, oleh tim SecOps).
- **Apply ke cluster**:

  ```bash
  kubectl -n istio-system create secret tls simpel-tls \
    --cert=/path/to/digicert-fullchain.pem \
    --key=/path/to/private-key.pem \
    --dry-run=client -o yaml | kubectl apply -f -
  ```

- **Verifikasi sebelum apply**:

  ```bash
  openssl x509 -in /path/to/digicert-fullchain.pem -noout -subject -issuer -dates
  ```

- **Rotation**: Re-apply secret dengan PEM baru (Istio gateway pickup hot-reload via SDS, tidak perlu restart pod). Reminder rotate H-30 sebelum expire.
- **Staging**: pakai self-signed atau biarkan plain HTTP via `10.1.7.121` (httpsRedirect: false di gateway).

## ⏰ CronJob Schedule (layanan-integrasi)

`layanan-integrasi` punya **gRPC server** (Deployment, selalu jalan) + **scheduler binary** yang dipanggil per-provider via K8s CronJob:

| Provider | Schedule | Time Zone | Staging | Production |
|----------|----------|-----------|---------|------------|
| **SIMAN** | `0 4 * * 0` (Minggu 04:00 WIB) | Asia/Jakarta | `suspend: true` | `suspend: false` |
| **MySIMKARI** | `0 */6 * * *` (tiap 6 jam) | Asia/Jakarta | `suspend: true` | `suspend: false` |
| **MonSAKTI** | `0 3 * * *` (tiap hari 03:00 WIB) | Asia/Jakarta | `suspend: true` | `suspend: false` |

**Alasan suspend di staging**: rate-limit token API eksternal yang dipakai bersama dengan production scheduler. Penarikan data di staging dilakukan **manual** untuk testing:

```bash
kubectl -n simpelv2-staging create job --from=cronjob/layanan-integrasi-mysimkari manual-$(date +%s)
kubectl -n simpelv2-staging logs job/manual-XXXX --tail=200
```

Override schedule per-env di `values-staging.yaml` / `values-production.yaml`:

```yaml
layananIntegrasi:
  cronjobs:
    siman:     { suspend: false, schedule: "0 4 * * 0" }
    mysimkari: { suspend: false }
    monsakti:  { suspend: false }
```

## ⚠️ Aturan AI untuk Operasi Infrastruktur

❌ **DON'T:**

- Tambahkan secret sensitif sebagai plaintext ke `values.yaml` mana pun yang akan di-commit. **Migrasi ke Secreton** (lihat section "Secret Management Zero-Trust").
- Edit chart lewat `kubectl edit` di cluster. Lakukan perubahan di `values-*.yaml` lalu `helm upgrade`.
- Buat manifest plain YAML baru di luar chart (mis. `kubectl apply -f foo.yaml`). Pendekatan plain-yaml/standalone sudah dihapus saat migrasi dari Kustomize.
- Hardcode tag image di template; set lewat `global.imageTag` atau `<komponen>.image.tag`.
- **Pakai mutable image tag** (`latest`, `stag`, `prod`, kosong). Schema validation reject ini saat `helm lint`.
- **Pakai k8s Secret untuk APP_KEY / token API** saat `secretonAuth.enabled=true`. Secret production WAJIB dari Secreton.
- **Deploy langsung ke production tanpa lewat staging.** Patuhi alur staging → promote → production (lihat "Pemisahan Lingkungan" → "Alur deploy WAJIB").

✅ **DO:**

- `./infra/helm/deploy.sh staging template` sebelum `install` untuk meninjau diff.
- `./infra/helm/deploy.sh staging diff` (helm-diff plugin) sebelum apply ke production.
- Patuhi label standar Kubernetes (`app.kubernetes.io/{name,component,part-of,managed-by,version,environment}`); chart `simpel` sudah meng-injectnya via `_helpers.tpl`.
- `helm rollback simpel -n simpelv2-<env>` saat regresi.
- Jalankan `bootstrap-secreton.sh` SEKALI per env saat fresh deploy (sebelum flip `secretonAuth.enabled=true`).
- Pakai semver tag (`v0.1.0`, `v1.2.3-rc1`); release via tag `git push origin v<MAJOR>.<MINOR>.<PATCH>` → trigger `release.yml`.

## 📋 Common Tasks

### 1. Deploy ke environment microk8s baru

Prerequisite di host: microk8s ≥ 1.30, kubectl alias, snap `helm`,
plugin `helm-diff`.

```bash
# 1. Enable addons (dns/storage/istio/metallb/longhorn cukup untuk staging)
microk8s enable dns storage istio metallb:10.64.140.43-10.64.140.49

# 2. MetalLB pre-bootstrap (chart-driven, idempoten)
./infra/helm/deploy.sh staging metallb-install

# 3. Helm install — values-staging.yaml dengan `secretonAuth.enabled=false`
#    untuk first boot (bootstrap loop)
./infra/helm/deploy.sh staging template     # ← review dulu
./infra/helm/deploy.sh staging install

# 4. Tunggu sampai pods Ready (≤ 5 menit di staging)
kubectl -n simpelv2-staging get pods -w

# 5. Bootstrap Secreton (init + unseal + K8s auth + seed KV)
./infra/helm/bootstrap-secreton.sh staging

# 6. Flip `secretonAuth.enabled=true` di values-staging.yaml lalu upgrade
./infra/helm/deploy.sh staging upgrade

# 7. Verify zero-trust: pods restart, log "fetched secret from secreton"
kubectl -n simpelv2-staging logs deploy/layanan-perlengkapan | grep -i secreton
```

Production identik kecuali: backup 5 Shamir share **dulu** ke 5 lokasi
terpisah sebelum `bootstrap-secreton.sh` dijalankan; revoke root
token hanya setelah konfirmasi unseal share tersimpan dengan benar.

### 2. Rollback Helm release

```bash
# Lihat history:
helm history simpel -n simpelv2-staging

# Rollback ke revision tertentu (nilai dari history --output table):
helm rollback simpel <revision> -n simpelv2-staging --wait

# Atau ke previous revision saja:
helm rollback simpel -n simpelv2-staging --wait
```

CronJob, scheduler, dan StatefulSet (`secreton`, `postgres`) di-roll
dengan PVC tetap utuh — rollback tidak menghapus data, hanya
mengembalikan spek manifest. Untuk schema migration regression, perlu
**manual** rollback DB migration via `refinery` CLI; chart tidak
mengontrol skema DB.

### 3. Update values tanpa restart workload yang tidak butuh

Default `helm upgrade` me-rolling-restart semua Deployment yang
manifest-nya berubah. Untuk perubahan **ConfigMap saja** (mis. update
log level, feature flag) yang sudah punya hot-reload, hindari restart
dengan template hook eksplisit:

```yaml
# Di template Deployment, hapus checksum annotation untuk komponen yang
# punya reload:
metadata:
  annotations:
    # checksum/config: {{ include (print $.Template.BasePath "/configmap.yaml") . | sha256sum }}
```

Hapus baris ini supaya ubah ConfigMap tidak men-trigger pod hash
mismatch. Lebih aman: simpan dynamic config di Secreton dan baca via
client polling (`SecretonClient::get_secret`) — pod tidak perlu
restart sama sekali.

### 4. Debug pod yang gagal start setelah deploy

```bash
# Step 1: lihat events scheduler-level (image pull, OOM, dll)
kubectl -n simpelv2-<env> describe pod <pod-name> | tail -40

# Step 2: log init container (kebanyakan masalah di sini — fetch-secrets,
# migration)
kubectl -n simpelv2-<env> logs <pod-name> -c <init-container-name>

# Step 3: main container
kubectl -n simpelv2-<env> logs <pod-name>

# Step 4: jika fetch-secrets fail karena Secreton sealed, unseal dulu:
kubectl -n simpelv2-<env> exec -it secreton-0 -- secreton operator unseal
```

`fetch-secrets` init container sengaja FAIL daripada start dengan
stale cred — bukan bug, melainkan defense in depth.
