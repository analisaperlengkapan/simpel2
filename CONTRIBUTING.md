# 🛠️ Panduan Kontribusi SIMPelv2

SIMPelv2 adalah proyek Kejaksaan yang terbuka untuk pegawai Kejaksaan dalam kontribusi baik  berpengalaman maupun pemula yang ingin belajar sambil membangun.

💡 **Tidak harus langsung ahli. Kami percaya bahwa kontribusi terbaik sering kali lahir dari proses belajar bersama.**

Kontributor akan mendapatkan:

* Paparan terhadap arsitektur microservices skala nasional
* Akses ke teknologi AI, CI/CD, K8s, dan observabilitas modern
* Lingkungan yang suportif
* Kesempatan untuk menciptakan dampak nyata pada tata kelola BMN negara

Mari mulai langkah kecil Anda hari ini—baik dari menulis dokumentasi, memperbaiki UI, menguji layanan, maupun menyumbangkan ide atau model AI.

Terima kasih atas ketertarikan Anda untuk berkontribusi dalam proyek **SIMPelv2**. Dokumen ini akan memandu Anda dari awal hingga pengajuan kontribusi ke sistem manajemen Barang Milik Negara berbasis teknologi dan AI ini.

---

## 📑 Daftar Isi

* [🧭 Alur Kontribusi](#-alur-kontribusi)

  * [👩‍💻 Persiapan Awal untuk Pengguna Windows (Pemula)](#-persiapan-awal-untuk-pengguna-windows-pemula)
* [🌿 Struktur Branch](#-struktur-branch)
* [📛 Format Commit](#-format-commit)
* [📋 Template Merge Request](#-template-merge-request)
* [🤖 Bentuk Kontribusi yang Didukung](#-bentuk-kontribusi-yang-didukung)

  * [🔄 Diagram Kolaborasi Peran](#-diagram-kolaborasi-peran)
  * [Peran dan Kemampuan Kontributor](#peran-dan-kemampuan-kontributor)
  * [🚀 Pemanfaatan Maksimal AI dalam Kontribusi](#-pemanfaatan-maksimal-ai-dalam-kontribusi)
* [💡 Tips: Gunakan AI untuk Mendukung Proses Kontribusi](#-tips-gunakan-ai-untuk-mendukung-proses-kontribusi)
* [🧩 Memaksimalkan Fitur GitLab](#-memaksimalkan-fitur-gitlab)
* [🔒 Kerahasiaan](#-kerahasiaan)
* [🛡️ Pelaporan Keamanan](#-pelaporan-keamanan)
* [💬 Bantuan & Diskusi](#-bantuan--diskusi)

---

## 🧭 Alur Kontribusi

### 👩‍💻 Persiapan Awal untuk Pengguna Windows (Pemula)

1. **Install WSL (Windows Subsystem for Linux)**
2. **Install VS Code + Ekstensi**
3. **Install Git & konfigurasi awal**
4. **Clone proyek untuk eksplorasi awal**
5. **Install Python, Docker, Make (opsional)**
6. **Jalankan proyek secara lokal**
7. **Setelah paham, lakukan fork & mulai kontribusi**

### Langkah Umum Kontribusi

1. Fork repositori → clone → checkout ke `dev`
2. Buat branch baru (`fitur/nama-fitur` atau `bugfix/deskripsi`)
3. Lakukan perubahan
4. Linting & validasi (`make validate-k8s`, `yamllint`, dll)
5. Commit dengan format baku → push → buat Merge Request ke `dev`

> ⚠️ Semua penggabungan dilakukan ke `dev`. Branch `main` hanya untuk rilis stabil.

---

## 🌿 Struktur Branch

```
main ← produksi (stabil)
└── dev ← pengembangan
    ├── fitur/*
    ├── bugfix/*
    ├── docs/*
    ├── ai/*
    ├── refactor/*
    ├── infra/*
    └── hotfix/*
```

---

## 📛 Format Commit

```
[fitur] Tambah dashboard pengguna
[bugfix] Perbaiki validasi login
[docs] Panduan instalasi di staging
[ai] Pipeline OCR pretraining
```

---

## 📋 Template Merge Request

```
### Ringkasan
- Tambahkan [fitur/deskripsi]
- Perbaiki [bug/lint]

### Checklist
- [x] Sudah diuji lokal
- [x] Validasi YAML lolos
- [x] Dokumentasi diperbarui

### Catatan Tambahan
- Tidak ada / Penyesuaian minor config
```

---

## 🤖 Bentuk Kontribusi yang Didukung

### 🔄 Diagram Kolaborasi Peran

```
[🎨 UI/UX] → [💻 Frontend] → [👩‍💻 Backend] → [🧠 AI]
       ↑         ↓                    ↓             ↓
  [📊 Analyst] ← [🧪 QA Tester] ← [🔐 Security] ← [🧠 AI Output]
```

### Peran dan Kemampuan Kontributor

Semua peran terbuka untuk pemula & profesional:

* **👩‍💻 Backend Developer** — Go, REST, CI/CD
* **💻 Frontend Developer** — React, Tailwind, Zustand
* **🧠 AI Engineer** — Python, XGBoost, NLP, OCR
* **📊 Data Analyst** — SQL, Grafana, statistik
* **🧪 QA Tester** — Manual test, Postman
* **🎨 UI/UX Designer** — Figma, usability audit
* **📖 Dokumentasi** — Markdown, arsitektur sistem
* **🖥️ Admin Infrastruktur** — Docker, K8s, TLS
* **🔐 DevSecOps** — RBAC, audit log, SealedSecrets

---

### 🚀 Pemanfaatan Maksimal AI dalam Kontribusi

Kontributor AI dapat berkontribusi dalam:

* Pembuatan pipeline baru (OCR, RAG, summarization)
* Pelatihan dan integrasi model ke layanan SIMPelv2
* Evaluasi, explainability, dan monitoring model AI
* Human-in-the-loop labeling & validasi
* Pengembangan REST API / microservice AI
* Data-centric AI & feedback loop

**Pendekatan AI yang dapat digunakan:**

| Pendekatan               | Deskripsi                                 | Contoh di SIMPelv2                   |
| ------------------------ | ----------------------------------------- | ------------------------------------ |
| Supervised Learning      | Belajar dari data berlabel                | Prediksi jumlah aset                 |
| Unsupervised Learning    | Klaster tanpa label                       | Segmentasi aset, analisis log        |
| Rule-based System        | Aturan eksplisit                          | Validasi distribusi                  |
| Active Learning          | Belajar dari feedback manusia             | Labeling narasi satker               |
| Transfer Learning        | Fine-tune dari model besar                | LLaMA3, Donut, spaCy                 |
| Reinforcement Learning   | Reward-based learning                     | (eksperimen) distribusi aset dinamis |
| RAG (Retrieval + LLM)    | Gabungan pencarian dan LLM                | Q\&A dokumen aset & hukum            |
| Explainable AI (XAI)     | Visualisasi dan penjelasan model          | SHAP, LIME, dashboard                |
| Human-in-the-loop (HITL) | Kolaborasi AI dan manusia dalam inferensi | Validasi klasifikasi & rekomendasi   |

---

## 💡 Tips: Gunakan AI untuk Mendukung Proses Kontribusi

Manfaatkan alat bantu AI untuk meningkatkan efisiensi:

* **ChatGPT / Gemini** — Refactor kode, generate schema
* **GitHub Copilot / Codeium** — Saran kode otomatis
* **LangChain, Ollama** — Prototipe pipeline AI
* **ExplainPaper, Elicit** — Ringkasan literatur teknis

> Tetap lakukan validasi manual. AI adalah alat bantu, bukan pengganti tanggung jawab kontribusi.

---

## 🧩 Memaksimalkan Fitur GitLab

### 🐞 GitLab Issues

* Gunakan untuk pelaporan bug & diskusi
* Gunakan label seperti `bug`, `ai-module`, `good first issue`

### 🎯 Milestone

* Kaitkan issue & MR ke milestone (mis. `v1.0.0`, `AI-Q3`)

### ✅ Merge Request (MR)

* Gunakan template MR
* Tambahkan reviewer sesuai `CODEOWNERS`
* Tambahkan label status (`ready`, `needs review`)

#### ℹ️ Apa itu CODEOWNERS?

File `CODEOWNERS` otomatis menetapkan reviewer berdasarkan path file. Contoh:

```
/layanan-usulan/  @tim-usulan
/layanan-ai/      @tim-ai
/docs/            @dokumensimpelv2
```

### 🔁 CI/CD

* Pipeline otomatis via GitLab CI & Drone CI
* Validasi kode: `build`, `test`, `lint`, `validate`

### 📈 Board & Roadmap

* Gunakan board untuk melihat progres issue
* Cek roadmap fitur untuk tahu rencana jangka menengah

---

## 🔒 Kerahasiaan

* Dilarang menyebarkan kode/data tanpa izin
* Semua data bersifat internal dan rahasia
* Pelanggaran akan dikenai sanksi (hukum & etika)


---

## 🛡️ Pelaporan Keamanan, Bantuan, & Diskusi

Jika menemukan potensi kerentanan, butuh bantuan, dan diskusi silahkan gunakan GitLab Issues untuk laporkan potensi kerentanan dan tanya jawab
* Email tim: `biro.perlengkapan@kejaksaan.go.id`

---

Terima kasih telah berkontribusi untuk membangun manajemen BMN yang lebih efisien dan cerdas 🇮🇩
