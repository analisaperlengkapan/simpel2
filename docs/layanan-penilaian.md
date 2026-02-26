# README - layanan-penilaian

**layanan-penilaian** adalah layanan mikro dalam SIMPEL yang bertanggung jawab mendokumentasikan hasil penilaian Barang Milik Negara (BMN) oleh penilai pemerintah. Layanan ini menyimpan, mengelola, dan menyajikan data hasil penilaian untuk kepentingan pengelolaan aset, laporan, serta akuntabilitas publik.

---

## 🎯 Tujuan Layanan

- Menyimpan hasil penilaian aset BMN secara elektronik
- Menyediakan data penilaian untuk layanan lain seperti pelaporan dan roadmap
- Menyajikan ringkasan dan interpretasi nilai aset
- Mendukung transparansi dan akuntabilitas proses penilaian

---

## 🧱 Teknologi

- Bahasa Pemrograman: Go (Gin Framework)
- Database: PostgreSQL (skema: `penilaian`)
- Otentikasi & Otorisasi: JWT + RBAC via `authenc`
- Integrasi: `layanan-aset`, `layanan-laporan`, `layanan-roadmap`
- AI: NLP summarization & interpretasi dokumen penilai

---

## 📁 Struktur Folder

```
layanan-penilaian/
├── cmd/                    # Entry point aplikasi
├── internal/
│   ├── handler/            # HTTP handler
│   ├── usecase/            # Logika bisnis
│   ├── repository/         # Akses data PostgreSQL
│   ├── model/              # Definisi model data
│   └── middleware/         # Middleware otentikasi dan validasi
├── api/
│   └── penilaian.yaml      # Spesifikasi OpenAPI
├── docs/                   # Dokumentasi layanan
├── test/                   # Unit dan integration test
├── sql/                    # Skema, migrasi, dan seed SQL
└── main.go
```

---

## 🔐 Keamanan

- Token JWT diverifikasi dengan middleware `authenc`
- Role-based access control: hanya pengguna tertentu yang bisa unggah/edit hasil
- Data sensitif seperti nilai estimasi dan catatan penilai dilindungi dengan enkripsi saat disimpan

---

## 🔗 Integrasi

- `layanan-aset`: menyinkronkan ID dan metadata aset
- `layanan-roadmap`: menggunakan nilai penilaian untuk perencanaan pengalihan/pemanfaatan
- `layanan-laporan`: menarik data rekapitulasi hasil penilaian untuk pelaporan berkala

---

## 🔍 Endpoint API (Contoh)

| Method | Endpoint                         | Deskripsi                           |
|--------|----------------------------------|-------------------------------------|
| GET    | `/penilaian/:id`                | Ambil data penilaian berdasarkan ID |
| POST   | `/penilaian`                    | Tambah hasil penilaian baru         |
| PUT    | `/penilaian/:id`                | Perbarui data penilaian             |
| GET    | `/penilaian/aset/:id_aset`      | Ambil semua penilaian untuk aset    |
| GET    | `/penilaian/ringkasan/:id_aset` | Ringkasan AI hasil penilaian        |

---

## 🧠 Fitur AI (Opsional)

- **Summarization otomatis** hasil dokumen penilai pemerintah
- **Ekstraksi entitas** seperti nilai pasar, kondisi, pendekatan penilaian
- **Klasifikasi penilaian** berdasarkan metode dan keperluan hukum

Model yang digunakan: Gemma 2B (summarizer), spaCy NER, dan Phi-2 untuk klasifikasi pendekatan.

---

## 🧪 Pengujian

Jalankan semua tes:

```bash
make test
```

Pastikan koneksi ke database lokal tersedia (dengan seed).

---

## 📜 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia

Layanan ini merupakan bagian dari sistem internal SIMPEL dan tidak untuk distribusi publik tanpa izin.

---

> Pastikan semua hasil penilaian BMN terdokumentasi dan dapat ditelusuri untuk mendukung pengelolaan aset negara yang transparan dan dapat dipertanggungjawabkan.
