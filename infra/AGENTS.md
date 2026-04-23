# 🤖 AGENTS.md - Infrastruktur & DevOps

> **Notice to Agents**: File ini adalah pedoman (Level 2 Archetype) untuk semua operasi terkait Infrastruktur, Deployment, dan DevOps di SIMPEL.

## 📑 Daftar Isi (Table of Contents)
1. 🗺️ Domain Routing
2. 🌍 Strategi Infrastruktur Global
3. ⚠️ Aturan AI untuk Operasi Infrastruktur

## 🗺️ Domain Routing
Jika Anda bekerja di subdirektori spesifik, baca aturan detailnya di sini:
- ☸️ **Kubernetes & Deployments**: Baca `infra/k8s/AGENTS.md`.
- 📊 **Monitoring (Prometheus/Grafana)**: (Terintegrasi dengan kustomize).
- 🌐 **Nginx/Ingress**: (Terintegrasi dengan Istio/K8s).

## 🌍 Strategi Infrastruktur Global

Infrastruktur SIMPEL berfokus pada **Keamanan Tingkat Tinggi (Zero-Trust)** dan **Ketersediaan (High Availability)**.

### 1. Prinsip Zero-Trust
- Semua komunikasi internal antar-layanan (backend-to-backend) **WAJIB** menggunakan mTLS (dikelola oleh Istio atau bawaan gRPC).
- Rahasia (Secrets) tidak boleh dibiarkan secara statis sebagai plain-text. Gunakan mekanisme injeksi *secrets* Kustomize (sebagai *CHANGEME*) dan hindari menyimpan kredensial asli di git.

### 2. GitOps & Kustomize
- Semua *manifests* Kubernetes dikelola secara deklaratif menggunakan **Kustomize** (`infra/k8s/base` dan `infra/k8s/overlays`).
- DILARANG mengedit resources langsung di dalam cluster (misal dengan `kubectl edit`). Semua perubahan **WAJIB** melalui repositori ini.

### 3. Pemisahan Lingkungan (Environments)
- **Staging** (`simpelv2-staging`): 1 Replica, log tingkat *debug*, dan aturan mTLS yang lebih permisif (PERMISSIVE).
- **Production** (`simpelv2-production`): Minimal 3 Replica (HA), log tingkat *info*, aturan mTLS ketat (STRICT), dan PodSecurity standards yang terbatas (*restricted*).

## ⚠️ Aturan AI untuk Operasi Infrastruktur
❌ **DON'T:**
- Jangan pernah mengubah atau menambahkan secret sensitif secara *plaintext* ke dalam file YAML apa pun.
- Jangan mengubah arsitektur jaringan (Network Policies) tanpa mengecek implikasinya pada layanan.

✅ **DO:**
- Gunakan skrip `deploy.sh` jika memungkinkan untuk menguji *dry-run* Kustomize secara lokal.
- Selalu patuhi penamaan *labels* standar Kubernetes (`app.kubernetes.io/name`, dll).
