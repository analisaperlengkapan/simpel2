# layanan-pengalihan

> **Catatan untuk Pengembangan Masa Depan:** Dokumen ini mendeskripsikan rencana pengembangan layanan ini yang saat ini belum diimplementasikan secara penuh dalam codebase. Dokumen ini dapat digunakan sebagai acuan untuk pengembangan ke depan agar tetap selaras dengan visi arsitektur SIMPEL v2 yang efektif, efisien, dan optimal.


**Layanan Pengalihan** adalah komponen dari SIMPEL yang menangani proses pengalihan fungsi, penggunaan, atau pemanfaatan Barang Milik Negara (BMN) antar unit atau instansi. Layanan ini dirancang untuk memastikan pengalihan dilakukan secara tertib, akuntabel, dan sesuai kebijakan pengelolaan aset negara.

---

## 🎯 Tujuan

- Memfasilitasi permohonan pengalihan fungsi atau pemanfaatan aset
- Mencatat dan melacak riwayat pengalihan
- Memastikan kepatuhan terhadap regulasi dan standar BMN

---

## ⚙️ Fitur Utama

- Pengajuan pengalihan oleh satuan kerja
- Validasi dan persetujuan berjenjang
- Tracking status pengalihan secara real-time
- Riwayat lengkap pengalihan aset
- Integrasi dengan layanan-aset dan layanan-roadmap
- Rekomendasi optimalisasi pemanfaatan berbasis AI

---

## 🧠 Komponen AI

| Fungsi AI                         | Teknologi                  |
|----------------------------------|-----------------------------|
| Analisis optimalisasi aset       | XGBoost + rule-based       |
| Prediksi potensi pengalihan      | Klasifikasi historis       |
| Anomali dalam pengalihan         | Deteksi pola tidak lazim   |
| Ringkasan laporan pengalihan     | LLM (Phi-2 / Gemma-2B)     |

---

## 🗂️ Struktur Direktori

```
layanan-pengalihan/
├── cmd/                        # Entrypoint aplikasi
├── internal/                   # Logika domain dan layanan
│   ├── handler/                # Handler HTTP
│   ├── service/                # Logika bisnis
│   ├── repository/             # Akses data
│   └── model/                  # Definisi model data
├── migrations/                 # Skema database SQL
├── openapi/                    # Spesifikasi API (OpenAPI 3)
├── docs/                       # Dokumentasi teknis
├── tests/                      # Unit dan integrasi test
├── Dockerfile                  # Image Docker layanan
└── README.md                   # Dokumentasi layanan ini
```

---

## 🔐 Keamanan

- Validasi JWT melalui middleware `authenc`
- RBAC berdasarkan peran pengguna dan unit kerja
- Logging aktivitas dan audit trail otomatis

---

## 📦 API Utama

| Endpoint                       | Metode | Deskripsi                              |
|-------------------------------|--------|----------------------------------------|
| `/pengalihan`                | GET    | Daftar semua pengalihan aset           |
| `/pengalihan`                | POST   | Ajukan pengalihan baru                 |
| `/pengalihan/{id}`           | PUT    | Perbarui pengajuan pengalihan          |
| `/pengalihan/{id}`           | GET    | Ambil detail pengalihan tertentu       |
| `/pengalihan/{id}/riwayat`   | GET    | Riwayat perubahan status               |

---

## 🧪 Pengujian

- Unit test menggunakan `testing` bawaan Go
- Tes integrasi via `httptest` dan `sqlmock`
- Pastikan semua endpoint diuji dan divalidasi oleh CI pipeline

---

## 🔄 Integrasi

- `layanan-aset`: validasi aset yang akan dialihkan
- `layanan-roadmap`: penyesuaian perencanaan setelah pengalihan
- `layanan-dokumen`: penyimpanan surat permohonan dan persetujuan
- `layanan-ai`: rekomendasi aset yang perlu dialihkan

---

## 📌 Catatan

- Semua pengalihan harus dicatat lengkap dengan alasan, nilai aset, dan tujuan pengalihan
- Dokumen digital harus diunggah dan tervalidasi format serta tandatangannya

---

## 📝 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia
