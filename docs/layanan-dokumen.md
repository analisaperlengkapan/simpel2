# 📁 layanan-dokumen

**layanan-dokumen** adalah layanan SIMPEL yang bertanggung jawab untuk pengelolaan dokumen elektronik, arsip digital, dan lampiran yang berkaitan dengan seluruh siklus pengelolaan BMN (Barang Milik Negara). Layanan ini mendukung pencarian, pelabelan otomatis, ekstraksi informasi, dan integrasi dengan modul AI (OCR, NER, klasifikasi).

---

## 🎯 Tujuan

- Menyediakan repositori terpusat untuk dokumen dan arsip
- Mendukung pencarian cerdas berbasis metadata dan konten
- Memungkinkan label otomatis dan ekstraksi data dari PDF dan gambar
- Menjamin keamanan, integritas, dan keterelusuran dokumen

---

## 🧱 Teknologi yang Digunakan

- **Go (Gin)** untuk API dan manajemen dokumen
- **PostgreSQL** untuk metadata dokumen
- **MinIO** untuk penyimpanan objek (dokumen, arsip, gambar)
- **PaddleOCR**, **spaCy**, dan **Donut** untuk ekstraksi data berbasis AI
- **Qdrant** untuk indexing dokumen dalam RAG (Retrieval-Augmented Generation)

---

## 📂 Struktur Direktori

```
layanan-dokumen/
├── cmd/                    # Entry point aplikasi
├── internal/               # Logika aplikasi
│   ├── handler/            # HTTP handler
│   ├── service/            # Logika layanan dokumen
│   ├── repository/         # Interaksi DB dan MinIO
│   └── ai/                 # Modul integrasi AI (OCR, klasifikasi, NER)
├── docs/                   # Dokumentasi internal layanan
├── api/                    # Definisi OpenAPI/Swagger
├── schema/                 # Skema database SQLC
├── scripts/                # Migrasi, seed, validasi
└── README.md
```

---

## 🧠 Fitur AI & Otomatisasi

| Fitur                         | Teknologi          | Fungsi                                                                 |
|------------------------------|--------------------|------------------------------------------------------------------------|
| OCR Dokumen PDF              | PaddleOCR          | Membaca teks dari hasil scan                                           |
| NER dan Ekstraksi Metadata   | spaCy              | Menemukan nama, tanggal, jenis dokumen, dll.                          |
| Parsing Visual Dokumen       | Donut              | Mengenali struktur layout dokumen berbasis gambar                     |
| Klasifikasi Dokumen Otomatis | LLM + Rule-based   | Menentukan kategori, jenis, dan keperluan dokumen                     |
| Indexing untuk RAG           | Qdrant             | Dokumen dapat dicari kembali oleh LLM dalam sesi percakapan pengguna  |

---

## 🔐 Keamanan

- Token-based Authentication (melalui `authenc`)
- Akses terkontrol melalui RBAC
- Audit trail dari setiap aktivitas akses/modifikasi dokumen
- Objek disimpan terenkripsi via MinIO + TLS

---

## 🔄 Integrasi

Layanan ini digunakan oleh:
- `layanan-usulan` untuk upload lampiran usulan
- `layanan-pemakaian`, `layanan-hibah`, dan lainnya untuk menyimpan SK, dokumen keputusan, dan laporan
- `layanan-ai` untuk ekstraksi dan anotasi berbasis AI

---

## 🧪 Testing & Validasi

- Unit test untuk handler dan service
- Integration test untuk upload/download dokumen
- Validasi schema dengan SQLC dan migrasi
- Linting YAML dan pipeline CI

---

## 👥 Tim Penanggung Jawab

- **Pengembang**: Tim Backend SIMPEL
- **AI & NLP**: Tim AI dan NLU SIMPEL
- **Dokumentasi & Metadata**: Tim Pengelola Arsip BMN

---

> "layanan-dokumen" adalah fondasi interoperabilitas antar-layanan di SIMPEL, mendukung alur kerja berbasis bukti dan pencarian dokumen cerdas melalui integrasi AI.
