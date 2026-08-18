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
- Per-MR review apps memakai `./infra/helm/deploy.sh review-<slug> install` (GitLab CI — job `deploy:review` di `.gitlab-ci.yml`; **alur rilis utama staging→prod = GitHub Actions `release.yml`/`promote.yml`**).

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

> **Skill:** prosedur lengkap + guardrail + perintah ada di Skill
> **`deploy-to-environment`** (`.claude/skills/`). Pakai saat rilis/deploy/promote/rollback.
> Untuk diagnosa/troubleshoot cluster (CrashLoop/Pending/Istio/Calico/ARC/MetalLB/
> Longhorn) → Skill **`kubectl-cluster-ops`** (read/diagnose + emergency; mutasi tetap via Helm).

Prasyarat production sekali-jalan: bootstrap+unseal Secreton, cert DigiCert di `istio-system`, MetalLB IP pool (lihat section terkait di bawah).

#### 🛡️ Keamanan data & lifecycle (WAJIB — insiden 2026-06-17)

> **`helm uninstall` = OPERASI BERBAHAYA, bukan rutin.** Insiden 2026-06-17:
> `helm uninstall simpel -n simpelv2-staging` meng-cascade-delete SELURUH namespace
> dan semua data (chart me-render `templates/namespace.yaml` sbg Namespace ber-manage
> Helm → hapus ns → cascade semua PVC/Secret; Longhorn reclaim=`Delete`; tanpa Velero
> → **tak terpulihkan**). Lihat memori `project-helm-namespace-footgun-safety`.

- **Lifecycle = UPGRADE-ONLY.** Perubahan environment lewat `helm upgrade`; `helm
  uninstall` hanya eksepsional, **selalu didahului backup (Velero/`pg_dump`)**, dan
  untuk production **di-guard** (`deploy.sh uninstall` menolak prod tanpa
  `SIMPEL_CONFIRM_DESTROY=yes`).
- **Sebelum `helm uninstall` apa pun, cek apakah chart me-render Namespace** —
  kalau ya, ns + isinya akan ke-cascade-delete.
- **Proteksi data ber-lapis di chart (sudah terpasang):** namespace.yaml + redis-pvc
  `helm.sh/resource-policy: keep`; postgres+secreton STS
  `persistentVolumeClaimRetentionPolicy: {whenDeleted: Retain, whenScaled: Retain}`.
- **StorageClass `longhorn-retain` (chart `infra/helm/storage`, `deploy.sh <env>
  storage-install`)** — `reclaimPolicy: Retain` + `numberOfReplicas: 2`. Retain
  adalah **satu-satunya** lapisan yang menahan skenario 2026-06-17: dua proteksi
  di atas menahan _Helm_ agar tak menghapus, tapi tak berdaya bila **namespace**-nya
  yang dihapus — cascade delete berjalan di sisi Kubernetes, bukan Helm. Dengan
  Retain, PV berubah jadi `Released` dengan data utuh, bukan lenyap.
- **Kelas penyimpanan ditulis di SATU tempat: `global.storageClass`.** Komponen
  stateful menurunkannya (`_workload.tpl` menyuntikkannya ke `volumeClaimTemplates`
  yang tak menyebutkannya). Dijaga CI: `values*.yaml` chart simpel yang menyebut
  `storageClassName` = job `Helm Lint & kube-linter` merah.
- **Mengganti kelas pada environment yang SUDAH punya PVC = tidak bisa in-place.**
  `volumeClaimTemplates` StatefulSet immutable → `helm upgrade` ditolak API server.
  Peralihan hanya saat bootstrap ulang. Karena itu staging masih `longhorn`
  (pengecualian bertanggal di `values-staging.yaml`) sementara production lahir
  langsung di `longhorn-retain`.
- **Velero + MinIO (chart `infra/helm/backup` + `infra/velero/`)** —
  `deploy.sh <env> backup-install` lalu `infra/velero/install.sh`. Object store
  hidup di namespace **sendiri** (`simpelv2-backup`): kegagalan yang terbukti
  terjadi di sini menghabisi SATU namespace, jadi backup yang tinggal serumah
  dengan datanya akan ikut mati bersama insiden yang seharusnya ia selamatkan.
  Prosedur + jadwal + latihan restore: `infra/helm/RUNBOOK.md` §8.
- **Postgres punya hook pre/post-backup** (`postgres.veleroBackupHook.enabled`):
  `pg_dumpall` konsisten-logis ditulis ke PVC tepat sebelum salinan file-system
  diambil, lalu dibuang lagi. Tanpa itu yang ter-backup hanya direktori data
  crash-consistent — biasanya bisa di-recover, tak pernah dijanjikan bisa.
- **Batas yang jangan dilupakan:** MinIO ini di disk klaster yang sama. Ia
  menahan kesalahan operasi & penghapusan objek Kubernetes, **bukan** hilangnya
  kedua node/site. Ganti `configuration.backupStorageLocation` di
  `infra/velero/values.yaml` begitu ada object store di luar klaster.
- **UTANG YANG BELUM LUNAS: latihan restore belum pernah dijalankan.** Sampai
  §8.4 benar-benar dieksekusi dan hasilnya dicatat, yang kita punya adalah
  mekanisme backup, bukan kemampuan pulih. Backup yang belum pernah di-restore
  belum terbukti jadi backup.
- **Data eksternal:** staging pakai data **mock/sintetis** (seed `integrasi.*`,
  sync OFF); token asli MySIMKARI/SIMAN/Monsakti **HANYA di production** (lihat
  memori `project-staging-mock-external-data` + `layanan/integrasi/AGENTS.md`).

#### 🗄️ Ketersediaan database di klaster 2-node (putusan 2026-08-18)

> **HA otomatis TIDAK bisa dicapai di klaster ini, dan tak boleh dijanjikan.**
> Bukan karena Postgres-nya, tapi karena lapisan di bawahnya.

Fakta terverifikasi (`kubectl get nodes -o wide`, 2026-08-18): klaster punya **dua**
node — `simpel.kejaksaan.go.id` (satu-satunya control plane) dan `simple02` (worker).
User mengonfirmasi node ke-3 **tidak memungkinkan saat ini**.

Konsekuensinya berantai:

- microk8s HA butuh **3 control plane** (kuorum dqlite). Dengan 1, hilangnya node
  itu = API server hilang.
- **Semua** operator Postgres HA (CNPG, Patroni, Zalando) memilih primary lewat
  **API Kubernetes**. API server mati ⇒ tak ada yang bisa mempromosikan standby.
  Jadi memasang operator HA di atas 1 control plane = membangun failover otomatis
  yang justru mati bersama penyebab downtime-nya.
- Longhorn `replica-soft-anti-affinity: false` (ketat) ⇒ maksimum 2 replika. Meminta
  3 membuat SETIAP volume permanen `degraded`, sehingga statusnya berhenti jadi
  sinyal. Kelas `longhorn-retain` meminta **2** — lihat bagian di atas.

**Yang berlaku sekarang (Tahap 0 — durabilitas, bukan ketersediaan):** risiko yang
sudah TERBUKTI di sistem ini adalah **kehilangan data** (insiden 2026-06-17), bukan
downtime. Karena itu urutannya: `longhorn-retain` ✅ → **Velero (#50)** → WAL
archiving/PITR → **latihan restore yang benar-benar dijalankan** (backup yang belum
pernah di-restore belum terbukti jadi backup).

**Kalau nanti butuh lebih:**

| Tahap | Prasyarat | Yang didapat | Yang TIDAK didapat |
|---|---|---|---|
| **0** (sekarang) | — | Data tahan hilangnya 1 node; PV tahan cascade-delete namespace; restore ber-RPO = jarak backup | Failover apa pun |
| **1.5** | node tetap 2 | CNPG 2 instance: replikasi streaming, standby panas, **promosi MANUAL** hitungan detik, RPO≈0 | Promosi otomatis |
| **1** | **node ke-3** | microk8s HA + CNPG 3 instance sinkron, failover otomatis, RPO 0 | — |

**Aturan sampai Tahap 1 tercapai:**

- **`postgres.replicas` WAJIB 1.** Image `postgres` stock tidak mereplikasi; >1
  replica di balik satu Service headless = N database TERPISAH yang bercabang
  diam-diam. Chart sudah `fail` keras kalau dinaikkan (`infrastructure/postgres.yaml`).
- **Jangan pasang scaffolding HA yang tak menyala.** Sisa Patroni (Role/RoleBinding
  `endpoints`+`pods`, port 8008, Service `postgres-replicas`, ServiceMonitor
  `port: patroni`) sudah dihapus 2026-08-18: selama 199 hari ia hanya memberi kesan
  ada HA sekaligus memberi hak tulis API ke SA yang tak memakainya.
- **Kalau pindah ke CNPG**, lakukan **sebelum** production pertama — migrasinya
  dump/restore, dan production belum pernah ter-deploy, jadi sekarang paling murah.
  CRD CNPG **sudah ada** di klaster (tanpa operator); operator Zalando berjalan
  tanpa satu pun cluster — keduanya perlu dibereskan saat keputusan diambil.

#### Release Train — kadens 3-lajur (TARGET-STATE; mekanik di-codify di P3/F-REL)

> **Status:** kebijakan/model di bawah = keputusan arah (2026-06-16). **Mekanik
> persis** (perubahan `release.yml`/`promote.yml`/`train.yml`) di-implement di
> **P3/F-REL — SETELAH staging terbukti manual (P2/F5-E)**, mengikuti disiplin
> _"automate the proven, jangan prove the automated"_. JANGAN tulis otomasi yang
> menyentuh cluster sebelum jalur manualnya hijau sekali. Rilis pertama `v0.1.0`
> tetap lewat jalur manual F5→F6.

Tiga lajur, **kadens memicu, mekanik tetap tag/digest-driven** (kalender hidup di
SATU tempat = "conductor" terjadwal, bukan tersebar di logika deploy):

| Lajur | Kadens | Aksi | Namespace | Tag | Gagal-uji |
|------|--------|------|-----------|-----|-----------|
| **Nightly** | tiap push ke `main` **+** `schedule:@daily` | per-push = gerbang lantai-CI (blocking, `main` selalu buildable); harian = suite komprehensif (integration+e2e+docker) | — (TIDAK deploy) | — | gate merah = blokir merge / notifikasi |
| **Beta (RC)** | `schedule:` ~2 minggu (conductor cut `vX.Y.Z-rcN` dari `main`) | `release.yml` build digest **D** sekali (cosign+SBOM+Trivy) → Velero backup → `helm upgrade` staging by-digest → uji komprehensif **+ destruktif** | `simpelv2-staging` | `vX.Y.Z-rcN` | **auto `helm rollback` staging** + TIDAK ada RC baru + notifikasi; retry manual dlm masa toleransi ≤1 minggu; tak sukses → tertunda ke periode berikut (alami: tak ada cut sukses = tak ada rilis) |
| **Stable** | `schedule:` ~triwulan (conductor buka issue/dispatch promote) | `promote.yml` promote **digest D yang SAMA** (tanpa rebuild) → `helm upgrade` prod → uji komprehensif **non-destruktif** | `simpelv2-production` | `vX.Y.Z` (final, dibuat **TERAKHIR**) | **IMMEDIATE auto `helm rollback` prod** (DB restore = last-resort; expand/contract membuat rollback app tanpa restore) + TIDAK buat tag stable + toleransi ≤1 minggu → tak sukses → tunda triwulan berikut |

Prinsip yang ditegakkan (selain "Alur deploy WAJIB" di atas):

- **Build-once / promote-digest:** RC build digest D; prod pakai D identik, tanpa rebuild.
- **Tag IMMUTABLE, tak pernah dihapus:** tag final `vX.Y.Z` = stempel SERTIFIKASI dibuat
  HANYA setelah prod hijau (bukan trigger). Gagal = rollback + fix-forward tag baru.
- **Auto-rollback = YA; auto-bikin-stable = TIDAK.** Kegagalan uji staging/prod →
  `helm rollback` otomatis (deterministik). Tapi **menerbitkan stable baru WAJIB
  human-gate** (GitHub Environment required reviewers) — `simpel.kejaksaan.go.id`
  sistem pemerintah, jangan full-auto ke prod.
- **Grace/defer TIDAK di-hardcode** sebagai logika kalender stateful di CI. Conductor
  _attempt + notify_; sukses → artefak; gagal → tak ada artefak (defer alami). Retry =
  dispatch manual dalam masa toleransi.
- **Jalur hotfix off-cycle** WAJIB tetap ada (security tak menunggu triwulan).
- "**nightly**" ≠ "tiap push": per-push = gerbang merge cepat; nightly = run terjadwal
  komprehensif. Keduanya lokal/CI saja (tanpa deploy namespace).

Prasyarat make-or-break: **Velero** (#50) terpasang; **disiplin migrasi expand/contract**
(#52 ✅) agar `helm rollback` tak butuh restore DB.

### 4. Image Registry & Tag

- **Registry resmi**: `ghcr.io/analisaperlengkapan/simpel2/<service>`. Pull dengan k8s Secret `ghcr-pull` (docker-registry type) per namespace; PAT scope `read:packages`.
- **Tag**: WAJIB SemVer `vMAJOR.MINOR.PATCH` (mis. `v0.1.0`). DILARANG mutable tag (`latest`, `stag`, `prod`, kosong) — schema `values.schema.json` reject saat `helm lint`.
- **`imagePullPolicy`**: `IfNotPresent` di staging & production (combined with immutable tag). `Never` di-deprecate (legacy era `localhost:32000` registry).
- **Build**: Tag git `v*.*.*` → `release.yml` GitHub Actions matrix build & push 7 image (portal, perlengkapan, authenc, layanan-integrasi, layanan-perlengkapan, secreton, simpelv1) dengan SBOM + provenance.
- **Image runner ARC** (`simpel2-arc-runner`) di luar matrix itu — ia bukan bagian rilis produk, tapi tetap dibangun CI (`arc-runner-image.yml`), **bukan tangan**. Tag diturunkan dari `infra/helm/arc/values-runner-set.yaml`; workflow menolak PR yang mengubah `runner-image/` tanpa menaikkan tag. Prosedur bump + alasan versinya kritis: `infra/helm/arc/README.md`.

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
- `simpelv2-tls-secret` di `istio-system` — Istio gateway baca dari k8s Secret.
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
  kubectl -n istio-system create secret tls simpelv2-tls-secret \
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
- **Loloskan temuan `kube-linter`.** Job `Helm Lint & kube-linter` (`security.yml`) = **BLOCKING** (di set `BLOCKING` Security Summary, path-gated `infra/helm/**`): `helm lint` + `kube-linter` atas rendered manifest (staging+production) WAJIB 0 temuan. Tiap workload WAJIB liveness+readiness (boot-lambat → startupProbe via `_probes.tpl`+values), `resources.requests/limits`, securityContext non-root + `readOnlyRootFilesystem` (scratch via emptyDir) + drop ALL caps + seccomp, PDB ber-`unhealthyPodEvictionPolicy`, anti-affinity (soft di single-node), tanpa mutable tag. **False-positive di-suppress per-objek** via annotation `ignore-check.kube-linter.io/<check>: "alasan"` di template (BUKAN blanket `exclude` di `infra/lint/.kube-linter.yaml`). Cek lokal: `helm template … | kube-linter lint`.
- **Pakai k8s Secret untuk APP_KEY / token API** saat `secretonAuth.enabled=true`. Secret production WAJIB dari Secreton.
- **Deploy langsung ke production tanpa lewat staging.** Patuhi alur staging → promote → production (lihat "Pemisahan Lingkungan" → "Alur deploy WAJIB").
- **Taruh penjaga di atas hal yang ia jaga.** Setiap workflow di repo ini `runs-on: arc-simpel`, jadi apa pun yang memantau kesehatan CI harus `runs-on: ubuntu-latest` — kalau tidak, ia mati bersama yang dipantaunya dan diam. Insiden 2026-08-10: runner yang di-deprecate mengunci ARC, job hanya menggantung `queued` (tak pernah merah — `required_status_checks` tak bisa menilai check yang tak pernah mulai), dan CI mati **4 hari tanpa satu pun indikator**. Berlaku sama untuk `ci-heartbeat.yml` dan `arc-runner-image.yml`.
- **Anggap kredensial yang hanya dipakai saat cache-miss sebagai teruji.** `imagePullPolicy: IfNotPresent` berarti `imagePullSecrets` hanya benar-benar dijalankan sekali, saat image pertama mendarat di node. Kredensial yang kedaluwarsa setelah itu tak terlihat sampai bump image berikutnya — lalu SEMUA runner `ImagePullBackOff` sekaligus, tepat di tengah pemulihan. Bila sebuah paket boleh publik (image tanpa kode SIMPel), lebih baik publik daripada memelihara PAT yang tak pernah diuji.

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
mengembalikan spek manifest.

#### 🔒 Migrasi WAJIB expand/contract (agar `helm rollback` aman tanpa restore DB)

> **Prinsip (F-REL):** **app rollback = `helm rollback`** (revisi N→N-1);
> **DB restore = LAST RESORT manual** (Velero / `pg_dump`, human-gated). Agar
> rollback aplikasi **tak pernah** butuh restore DB, **setiap migrasi WAJIB
> expand/contract** — additive & backward-compatible dalam satu rilis.

- **Expand dulu (rilis ini):** hanya perubahan **aditif & kompatibel-mundur** —
  `ADD COLUMN` nullable / ber-default, tabel/index baru, `CREATE … IF NOT EXISTS`.
  Kode versi LAMA (target rollback) **harus tetap jalan** di atas skema BARU.
- **Contract nanti (rilis BERIKUTNYA):** perubahan destruktif (`DROP COLUMN`,
  `NOT NULL`, rename, `DROP TABLE`) hanya **setelah** versi yang masih memakainya
  tak lagi di-deploy/di-rollback-able. Pisahkan minimal satu rilis.
- **DILARANG** menggabung expand+contract destruktif dalam rilis yang sama —
  itu membuat `helm rollback` butuh restore DB (= jalur kehilangan data).
- **Rename = 3 langkah lintas-rilis:** (1) add kolom baru + dual-write; (2)
  backfill + baca kolom baru; (3) drop kolom lama. Bukan `ALTER … RENAME` sekali jalan.
- **Migrasi idempotent + reversibel-secara-data:** hook `*-migrate-job.yaml` boleh
  jalan ulang (pre-upgrade); jangan ada operasi yang merusak saat re-run.
- Konsekuensi: pada rollback app, **JANGAN** rollback DB. Skema BARU (aditif)
  tetap kompatibel dengan kode LAMA. Restore DB hanya bila migrasi melanggar aturan
  ini (insiden) — gunakan Velero/`pg_dump` terbaru, dengan human gate.
- Aturan **authoring** migrasi (file `V###`/baseline, no-ALTER-after-merge,
  search_path, penamaan) ada di Skill `add-backend-feature` + `layanan/AGENTS.md`.

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
