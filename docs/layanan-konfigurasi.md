---

> **Catatan untuk Pengembangan Masa Depan:** Dokumen ini mendeskripsikan rencana pengembangan layanan ini yang saat ini belum diimplementasikan secara penuh dalam codebase. Dokumen ini dapat digunakan sebagai acuan untuk pengembangan ke depan agar tetap selaras dengan visi arsitektur SIMPEL v2 yang efektif, efisien, dan optimal.


### 📄 `layanan-konfigurasi/README.md`

```markdown
# Layanan Konfigurasi – SIMPEL

Layanan **Konfigurasi** adalah pusat pengelolaan referensi, konfigurasi sistem, dan template standar di SIMPEL. Layanan ini menyediakan data referensi yang digunakan oleh seluruh layanan lain, seperti satuan barang, jenis BMN, kategori penggunaan, tahun anggaran, dan lainnya.

---

## 🎯 Tujuan

Menjadi sumber kebenaran tunggal (Single Source of Truth) untuk seluruh konfigurasi dan referensi operasional yang diperlukan oleh sistem SIMPEL secara konsisten dan terpusat.

---

## 🧱 Fitur Utama

- 📚 Referensi jenis, kelompok, dan satuan BMN
- 📆 Referensi tahun anggaran, periode, semester
- 📊 Referensi status, kategori, dan kode klasifikasi
- 📦 Template output laporan, dokumen, dan tabel
- 🧩 Konfigurasi sistem per lingkungan (`dev`, `prod`)
- 🔁 Sinkronisasi berkala dari sumber eksternal (mis: SIMAN)
- 🛡️ Middleware RBAC untuk kontrol pengubahan

---

## ⚙️ Teknologi

- Bahasa: Go (Gin) + sqlc
- Database: PostgreSQL (skema `konfigurasi`)
- Validasi input: JSON Schema
- Caching: Redis (opsional)
- Logging: Terintegrasi dengan layanan-audit

---

## 📁 Struktur Folder

```
layanan-konfigurasi/
├── api/                  # Handler endpoint
├── model/                # Struktur entitas konfigurasi
├── repository/           # Query SQL (via sqlc)
├── service/              # Logika bisnis konfigurasi
├── sync/                 # Modul sinkronisasi referensi eksternal
├── config/               # Konfigurasi runtime
├── middleware/           # Validasi otorisasi perubahan
├── main.go               # Entry point
└── go.mod / go.sum       # Dependensi
```

---

## 🔄 Contoh Endpoint

| Metode | Endpoint                    | Keterangan                                |
|--------|-----------------------------|-------------------------------------------|
| GET    | `/referensi/satuan`         | Daftar satuan barang                      |
| GET    | `/referensi/jenis-bmn`      | Daftar jenis barang milik negara          |
| GET    | `/konfigurasi/tahun`        | Tahun aktif dan arsip                     |
| POST   | `/konfigurasi/template`     | Tambah template dokumen                   |
| PUT    | `/konfigurasi/sistem`       | Ubah konfigurasi sistem (admin only)      |
| GET    | `/sinkron/siman`            | Trigger sinkronisasi referensi dari SIMAN |

---

## 🧠 Peran AI (Opsional)

- Rekomendasi konfigurasi awal untuk instansi baru
- Deteksi perubahan referensi yang inkonsisten
- Suggestion template berdasarkan jenis laporan
- Analisis perubahan referensi dan dampaknya lintas layanan

---

## 🔧 Konfigurasi .env

```env
REFERENSI_CACHE_ENABLED=true
SIMAN_API_KEY=xxx
```

---

## 🔐 Akses & Validasi

- Perubahan referensi hanya oleh pengguna dengan role `admin-konfigurasi`
- Semua perubahan tercatat di `layanan-audit`
- Token JWT dari `authenc` diperlukan untuk semua request

---

## 📌 Best Practices

- Jangan hardcode referensi dalam layanan lain
- Gunakan endpoint konfigurasi secara dinamis (mis. saat dropdown load)
- Validasi eksternal seperti SIMAN hanya dilakukan saat sinkronisasi berkala

---

## 📚 Terkait

- [`layanan-standar`](../layanan-standar) – menggunakan referensi barang
- [`layanan-usulan`](../layanan-usulan) – membaca referensi satuan & tahun

---

© 2025 – Kejaksaan RI | Divisi Teknologi SIMPEL

```

---
