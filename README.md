## 🚀 SIMPelv2

[![GitLab CI](https://gitlab.com/analisiskebutuhan/simpelv2_web/badges/main/pipeline.svg)](https://gitlab.com/analisiskebutuhan/simpelv2_web/pipelines)
![Lint Status](https://img.shields.io/badge/lint-passing-brightgreen)
![Security Scan](https://img.shields.io/badge/security-enabled-blue)
![License](https://img.shields.io/badge/license-MIT-blue)
[![Coverage](https://gitlab.com/analisiskebutuhan/simpelv2_web/badges/main/coverage.svg)](https://gitlab.com/analisiskebutuhan/simpelv2_web/-/graphs/main/charts)

**SIMPelv2** (Sistem Informasi Manajemen Barang Milik Negara Versi 2) adalah platform modern berbasis web untuk mengelola siklus hidup Barang Milik Negara (BMN), dibangun dengan arsitektur **microservices**, pendekatan **AI modular**, serta mematuhi standar **ISO dan keamanan informasi tingkat tinggi**.

> SIMPelv2 mendukung efisiensi, akuntabilitas, dan transparansi dalam manajemen BMN secara menyeluruh, dengan keamanan dan teknologi terkini.

---

## 📑 Daftar Isi

* [Fitur Unggulan](#fitur-unggulan)
* [Teknologi Inti](#teknologi-inti)
* [Instalasi & Setup](#instalasi--setup)
* [Struktur Microservices & Folder](#struktur-microservices--folder)
* [AI & Otomasi](#ai--otomasi)
* [Monitoring & Observabilitas](#monitoring--observabilitas)
* [Perintah Makefile](#perintah-makefile)
* [CI/CD GitLab](#cicd-gitlab)
* [Keamanan Sistem](#keamanan-sistem)
* [Kepatuhan & ISO](#kepatuhan--iso)
* [Kontribusi](#kontribusi)
* [Lisensi](#lisensi)

---

## ✅ Fitur Unggulan

* ✅ Microservices modular berbasis Go (Gin)
* ✅ Frontend SPA modern dengan React + Vite
* ✅ AI terintegrasi: OCR, klasifikasi, rekomendasi, summarization, label
* ✅ Keamanan berlapis: TLS, Sealed Secret, RBAC, AI UEBA
* ✅ Monitoring terpusat: Grafana, Prometheus, Loki, Alertmanager
* ✅ Integrasi resmi: SIMAN, MONSAKTI, MySimkari, SIPEDE
* ✅ Audit trail lengkap, dasbor interaktif, dan pelaporan otomatis
* ✅ Kepatuhan terhadap ISO 27001, 20000-1, 55001, 9001, 25010, 38500

---

## 🧱 Teknologi Inti

| Komponen        | Teknologi                                               |
| --------------- | ------------------------------------------------------- |
| Backend         | Go (Gin), SQLC, REST API                                |
| Frontend        | React, Vite, Tailwind CSS                               |
| Database        | PostgreSQL (multi-schema) + TLS                         |
| Gateway & Proxy | Fiber + NGINX (TLS, static, reverse proxy)              |
| Orkestrasi      | Docker Compose, MicroK8s, Kubernetes                    |
| CI/CD           | GitLab CI, Drone CI (opsional), Makefile                |
| AI Engine       | LLaMA3, Phi-2, Gemma, Donut, Pix2Struct, spaCy, XGBoost |
| Observability   | Prometheus, Grafana, Loki, Alertmanager                 |

---

## ⚙️ Instalasi & Setup

### 📌 Prasyarat

* Docker & Docker Compose
* MicroK8s (dengan `kubectl`, `kustomize`, `helm`, `ingress`)
* Python 3.10+ (`pip install -r requirements.txt`)
* Tools tambahan: `kubeseal`, `yamllint`, `ruamel.yaml`

### 🔧 Langkah Setup

```bash
git clone https://gitlab.com/analisiskebutuhan/simpelv2_web.git
cd simpelv2
cp .env.example .env
make build
make up
```

Untuk Kubernetes dev/staging/prod:

```bash
make build-and-import
make generate-k8s
make deploy-dev
# atau:
make deploy-staging
make deploy-prod VERSION=v1.0.0
```

---

## 🧱 Struktur Microservices & Folder

```text
simpelv2/
├── antarmuka/                 # Frontend React Vite
├── gerbang/                   # Gateway Fiber (routing, middleware, RBAC)
├── nginx/                     # TLS, reverse proxy, static file
├── scripts/                   # Script Python generate_k8s, validator
├── k8s/                       # Struktur YAML Kustomize
│   ├── base/                  # Template dasar Deployment/Service/Ingress
│   ├── overlay/dev/           # Kustomisasi dev
│   ├── overlay/staging/       # Kustomisasi staging
│   └── overlay/prod/          # Kustomisasi production
├── layanan-aset/              # Master aset BMN (AI: klasifikasi, validasi)
├── layanan-usulan/            # Usulan kebutuhan (AI: estimasi kebutuhan)
├── layanan-rekomendasi/       # Rekomendasi jumlah & spesifikasi (AI: XGBoost)
├── layanan-standar/           # Standar BMN nasional
├── layanan-roadmap/           # Perencanaan jangka panjang (AI: simulasi tren)
├── layanan-pemakaian/         # Permohonan dan izin (AI: pola anomali)
├── layanan-pengalihan/        # Optimalisasi & alih fungsi BMN
├── layanan-distribusi/        # Distribusi fisik (AI: jalur & jadwal estimasi)
├── layanan-pemeliharaan/      # Anggaran & siklus perawatan (AI: prediksi)
├── layanan-penilaian/         # Penilaian aset (AI: interpretasi hasil)
├── layanan-hibah/             # Pengajuan dan pemberian hibah
├── layanan-dokumen/           # OCR, klasifikasi, arsip digital
├── layanan-audit/             # Log & audit (AI: korelasi & deteksi)
├── layanan-keamanan/          # Autentikasi & otorisasi pengguna
├── layanan-konfigurasi/       # Metadata, referensi, dan konfigurasi
├── layanan-bantuan/           # Q&A berbasis LLM, FAQ, panduan
├── layanan-dasbor/            # Dashboard lintas layanan
├── layanan-laporan/           # Pelaporan dinamis dan tren real-time
├── layanan-integrasi/         # Konektor SIMAN, MONSAKTI, MySimkari, SIPEDE
├── layanan-ai/                # AI utama: OCR, NER, summarization, rekomendasi
```

---

## 🧠 AI & Otomasi

| Modul/Fungsi           | Teknologi Utama            |
| ---------------------- | -------------------------- |
| OCR dokumen            | PaddleOCR                  |
| Ekstraksi + NER        | spaCy                      |
| Klasifikasi narasi     | Phi-2 / Gemma-2B           |
| Rekomendasi jumlah BMN | XGBoost + rule engine      |
| Ringkasan dokumen      | LLaMA3 / Mistral           |
| Label dari PDF/foto    | Donut / Pix2Struct         |
| AI RAG untuk Q\&A      | LLaMA3-3B + Qdrant         |
| Analisis perilaku user | UEBA (behavior anomaly AI) |

---

## 📊 Monitoring & Observabilitas

* **Grafana** → `http://localhost:3000`
* **Prometheus** → `http://localhost:9090`
* **Loki (logs)** → `http://localhost:3100`
* **Alertmanager** → `http://localhost:9093`

Logging dikumpulkan dari semua layanan ke stack observabilitas.

---

## 📦 Perintah Makefile

| Perintah                | Fungsi                                                 |
| ----------------------- | ------------------------------------------------------ |
| `make build`            | Build seluruh container dari Dockerfile                |
| `make up`               | Jalankan semua container (Docker Compose)              |
| `make build-and-import` | Build, export, lalu `ctr import` ke MicroK8s           |
| `make generate-k8s`     | Generate file YAML dari Compose & .env                 |
| `make deploy-dev`       | Deploy ke namespace `dev` di K8s                       |
| `make deploy-prod`      | Deploy ke `production`                                 |
| `make validate-k8s`     | Validasi hasil YAML (`kubectl apply --dry-run=client`) |
| `make destroy-k8s`      | Hapus seluruh resource Kubernetes yang aktif           |
| `make seal-secret`      | Enkripsi `.env` menjadi SealedSecret                   |
| `make check-status`     | Cek semua status Pod dalam namespace                   |

---

## 🔄 CI/CD GitLab

### 📌 Perintah CI/CD yang Umum Digunakan

```bash
# 1. Push perubahan ke branch baru
git checkout -b fitur/layanan-baru
# ...edit file
git add . && git commit -m "feat: tambah layanan baru"
git push origin fitur/layanan-baru

# 2. Buat Merge Request di GitLab Web ke branch main
# Pipeline otomatis jalan dan validasi YAML, .env, dll

# 3. (Opsional) Jalankan pipeline ulang secara manual jika gagal
# Klik tombol retry di GitLab UI

# 4. Setelah merge ke main, pipeline akan auto-deploy ke dev
```

### 🧬 Diagram Alur CI/CD (GitLab)

```mermaid
graph TB
  A[📤 Commit/Push ke GitLab] --> B[🧪 Lint YAML & Validasi .env]
  B --> C[⚙️ Generate YAML & 🔐 Seal Secret]
  C --> D[🚦 Simulasi Pod (dry-run)]
  D --> E{Branch = main?}
  E -- Ya --> F[🚀 Deploy ke Dev]
  F --> G[🏷️ Otomatisasi Tag & Release]
  E -- Tidak --> H[🛑 Review Manual MR]
```

---

## 🔐 Keamanan Sistem

* 🔐 TLS Digicert (Letakkan di `nginx/certs/`)
* 🔐 Sealed Secrets terenkripsi (Kubernetes)
* 🔐 Fiber middleware: RBAC, otorisasi token, audit
* 🔐 Semua traffic diatur melalui reverse proxy NGINX
* 🔐 Logging & pemantauan anomali dengan AI (UEBA)

---

## 🛡️ Kepatuhan & ISO

| Standar ISO   | Kaitan dengan SIMPelv2                                                               |
| ------------- | ------------------------------------------------------------------------------------ |
| ISO 27001     | Perlindungan terhadap keamanan informasi pengguna, aset, dan infrastruktur TI        |
| ISO 20000-1   | Tata kelola manajemen layanan TI lintas layanan SIMPelv2                             |
| ISO 55001     | Pengelolaan aset (BMN) secara menyeluruh, akuntabel, dan efisien                     |
| ISO 25010     | Memastikan kualitas perangkat lunak seperti usability, maintainability, dan security |
| ISO 38500     | Tata kelola dan pengambilan keputusan strategis dalam pengembangan TI                |
| ISO 9001      | Sistem manajemen mutu dalam siklus pengembangan, dukungan, dan pemeliharaan SIMPelv2 |
| PCI DSS / BSI | Standar keamanan tambahan untuk transaksi dan kontrol akses                          |

---

## 🤝 Kontribusi

Panduan kontribusi lengkap tersedia di [`CONTRIBUTING.md`](./CONTRIBUTING.md).

Beberapa kontribusi yang kami dukung:

* 👩‍💻 Kode (fitur, bug, refactor, infra)
* 🤖 AI & pipeline model
* 📚 Dokumentasi & panduan
* 🛡️ Validasi keamanan & kepatuhan

Jika Anda ingin berkontribusi, pastikan:

* Membaca & mengikuti struktur branch
* Menjalankan validasi (`make validate-k8s`, `yamllint`, dll)
* Menggunakan penamaan commit yang jelas
* Menyertakan deskripsi lengkap saat membuat Merge Request

Jika menemukan bug atau ingin diskusi fitur, gunakan Issue Tracker atau email ke `biro.perlengkapan@kejaksaan.go.id`

---

## 📝 Lisensi

Hak Cipta © 2025 **Kejaksaan Republik Indonesia**
Penggunaan terbatas untuk manajemen Barang Milik Negara.
Dilarang memperjualbelikan ulang tanpa izin resmi.
