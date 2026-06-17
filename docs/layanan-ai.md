# layanan-ai

> **Catatan untuk Pengembangan Masa Depan:** Dokumen ini mendeskripsikan rencana pengembangan layanan AI yang saat ini belum diimplementasikan secara penuh sebagai microservice terpisah. Implementasi saat ini mungkin masih tersebar di layanan lain atau dalam tahap perencanaan. Dokumen ini dapat digunakan sebagai acuan untuk pengembangan ke depan agar tetap selaras dengan visi arsitektur SIMPEL v2 yang efektif, efisien, dan optimal.

**Layanan AI SIMPEL** merupakan inti dari kemampuan kecerdasan buatan dalam sistem SIMPEL. Layanan ini menyediakan antarmuka API untuk berbagai fungsi seperti OCR, klasifikasi, rekomendasi, ringkasan teks, label dokumen, dan pemrosesan visual, yang diimplementasikan secara modular dan efisien.

---

## 🎯 Tujuan

Menyediakan layanan AI/ML yang mendukung pengambilan keputusan dan automasi dalam pengelolaan BMN:

- Ekstraksi informasi dari dokumen PDF/gambar (OCR + NER)
- Klasifikasi jenis barang/usulan dan kondisi
- Rekomendasi kebutuhan dan jumlah berdasarkan pola historis
- Ringkasan narasi laporan satker
- Parsing visual dokumen PDF dan scan (tanpa OCR)
- Layanan Q&A berbasis LLM dengan dukungan RAG (vector DB)

---

## ⚙️ Struktur Direktori

```
layanan-ai/
├── cmd/                # Entrypoint server/API
├── internal/           # Modul fungsional AI
│   ├── ocr/            # Ekstraksi teks dari PDF/gambar (PaddleOCR)
│   ├── ner/            # Named Entity Recognition (spaCy)
│   ├── classify/       # Klasifikasi jenis BMN, kondisi
│   ├── recommend/      # Model rekomendasi jumlah/spesifikasi
│   ├── summary/        # Summarization narasi satker
│   ├── vision/         # Visual parsing dokumen (Donut, Pix2Struct)
│   └── rag/            # Retrieval-Augmented Generation (LLM + Qdrant)
├── model/              # Model terlatih (bin, pt, tokenizer, config)
├── data/               # Dataset pelatihan dan validasi
├── scripts/            # Pipeline training, fine-tuning, evaluation
├── api/                # OpenAPI/gRPC specification
├── Dockerfile
└── README.md
```

---

## 🧠 Teknologi dan Model

| Komponen              | Teknologi/Model               | Fungsi                                |
|----------------------|-------------------------------|----------------------------------------|
| OCR                  | PaddleOCR                     | Ekstraksi teks dari gambar/PDF         |
| NER                  | spaCy                         | Ekstraksi entitas dari teks            |
| Klasifikasi Narasi   | Phi-2 / Gemma-2B              | Kategori usulan, kondisi BMN           |
| Rekomendasi          | XGBoost / DecisionTree        | Jumlah/spesifikasi barang              |
| Summarization        | Gemma, Mistral                | Ringkasan teks narasi                  |
| Visual Parsing       | Donut, Pix2Struct             | Labeling dan parsing langsung visual   |
| LLM Q&A              | LLaMA3-3B + Qdrant (RAG)      | Pertanyaan BMN kontekstual             |

---

## 🧪 Contoh Endpoint

```http
POST /api/ocr
→ JSON { base64_file }

POST /api/classify
→ JSON { teks_usulan }

POST /api/rekomendasi
→ JSON { jenis_bmn, lokasi, kebutuhan }

POST /api/ringkasan
→ JSON { narasi_satker }

POST /api/label-dokumen
→ PDF upload → label & metadata
```

---

## 🔐 Keamanan

- Semua endpoint dibatasi via middleware gateway
- Validasi input otomatis dan manual
- Tidak menyimpan hasil inferensi secara permanen
- Data sensitif hanya diproses jika berasal dari layanan internal

---

## 📈 Standar Implementasi

- Semua pipeline AI dapat direplikasi (reproducible)
- Model dikemas dalam container dan tidak memerlukan internet
- Runtime optimal: 1-2 vCPU, RAM < 2GB per model
- Konfigurasi model dinamis via `.env` dan YAML

---

## 🧩 Integrasi Lintas Layanan

| Layanan Terkait         | Fungsi AI yang Terhubung                   |
|-------------------------|--------------------------------------------|
| `layanan-usulan`        | Klasifikasi jenis & kondisi usulan        |
| `layanan-rekomendasi`   | Sistem rekomendasi jumlah/spesifikasi     |
| `layanan-pemakaian`     | Analisis pola permohonan abnormal         |
| `layanan-penilaian`     | Ringkasan hasil penilai BMN               |
| `layanan-dokumen`       | OCR, labeler, parsing dokumen visual      |
| `layanan-bantuan`       | Q&A BMN via LLM + RAG                      |
| `layanan-audit`         | Analisis korelasi audit, deteksi anomali  |

---

## 🤖 Pendekatan AI/ML yang Digunakan

| Pendekatan                  | Penjelasan & Implementasi SIMPEL                                        |
|----------------------------|----------------------------------------------------------------------------|
| Supervised Learning        | Klasifikasi kondisi BMN, jenis usulan, hasil dokumen                      |
| Transfer Learning          | Fine-tuning LLM (Phi-2, Gemma) untuk teks BMN spesifik                    |
| Instruction Tuning         | Model dilatih dengan instruksi-jawaban → Q&A BMN                         |
| RAG (Retrieval-Augmented)  | Vector DB + LLM menjawab berdasarkan dokumen teknis dan peraturan        |
| Active Learning            | Labeling semi-otomatis berbasis masukan pengguna (feedback loop)         |
| Human-in-the-Loop          | Validasi hasil AI oleh admin teknis, loop pelabelan                      |
| Vision-based Modeling      | Model CV membaca langsung layout visual dokumen (tanpa OCR)              |
| Rule-based Hybrid          | Kombinasi aturan + ML kecil untuk rekomendasi                            |

---

## 👨‍💻 Panduan Kontribusi

- Kontributor wajib membaca [CONTRIBUTING.md](../CONTRIBUTING.md)
- Model harus bisa dilatih dan dijalankan secara lokal
- Berkas besar disimpan dalam folder `model/` atau link eksternal
- Dilarang menggunakan layanan cloud terbuka untuk inferensi

---

## 📝 Lisensi dan Kebijakan Akses

Hak Cipta © 2025 Kejaksaan Republik Indonesia.

> Layanan ini merupakan bagian dari sistem internal tertutup SIMPEL dan memproses informasi sensitif. Setiap kontribusi, akses, atau penggunaan ulang wajib mematuhi NDA dan kebijakan internal Kejaksaan RI. Pelanggaran atas kerahasiaan data atau kebijakan sistem akan dikenakan sanksi administratif dan/atau hukum sesuai peraturan yang berlaku.

---
