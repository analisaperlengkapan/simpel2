# 🤖 AGENTS.md - Infrastruktur & DevOps

> **Notice to Agents**: File ini adalah pedoman (Level 2 Archetype) untuk semua operasi terkait Infrastruktur, Deployment, dan DevOps di SIMPEL.

## 📑 Daftar Isi
1. 🗺️ Domain Routing
2. 🌍 Strategi Infrastruktur Global
3. ⚠️ Aturan AI untuk Operasi Infrastruktur

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
- **Staging** (`simpelv2-staging`): 1 replica, log `debug`, mTLS PERMISSIVE, image tag `:stag`, semua pod di-pin ke node `simple02`.
- **Production** (`simpelv2-production`): 3 replica HA, log `info`, mTLS STRICT, ResourceQuota & LimitRange aktif, image tag `:prod` / `:vX.Y.Z`, scheduling fleksibel, HTTPS redirect on.

## ⚠️ Aturan AI untuk Operasi Infrastruktur

❌ **DON'T:**
- Tambahkan secret sensitif sebagai plaintext ke `values.yaml` mana pun yang akan di-commit. Selalu pakai pola `values-secrets.yaml` (gitignored) atau Secreton SecretSync.
- Edit chart lewat `kubectl edit` di cluster. Lakukan perubahan di `values-*.yaml` lalu `helm upgrade`.
- Buat manifest plain YAML baru di luar chart (mis. `kubectl apply -f foo.yaml`). Pendekatan plain-yaml/standalone sudah dihapus saat migrasi dari Kustomize.
- Hardcode tag image di template; set lewat `global.imageTag` atau `<komponen>.image.tag`.

✅ **DO:**
- `./infra/helm/deploy.sh staging template` sebelum `install` untuk meninjau diff.
- `./infra/helm/deploy.sh staging diff` (helm-diff plugin) sebelum apply ke production.
- Patuhi label standar Kubernetes (`app.kubernetes.io/{name,component,part-of,managed-by,version,environment}`); chart `simpel` sudah meng-injectnya via `_helpers.tpl`.
- `helm rollback simpel -n simpelv2-<env>` saat regresi.
