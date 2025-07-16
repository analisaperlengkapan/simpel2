# layanan-pemakaian

**Layanan Pemakaian** adalah komponen dalam sistem SIMPelv2 yang menangani seluruh proses permohonan, izin, dan pencatatan pemakaian Barang Milik Negara (BMN) oleh satuan kerja (satker). Layanan ini memastikan bahwa seluruh proses pemanfaatan aset negara terdokumentasi dengan baik, sesuai regulasi, dan dapat dipantau secara akurat.

---

## 🎯 Tujuan

- Memfasilitasi proses permohonan pemakaian BMN oleh pengguna.
- Menyediakan alur persetujuan dan pencatatan izin pemakaian.
- Menjaga transparansi dan akuntabilitas pemanfaatan aset negara.
- Mendukung pelaporan dan audit terhadap penggunaan aset.

---

## 🧩 Fitur Utama

- Pengajuan permohonan pemakaian BMN oleh satker.
- Proses verifikasi dan persetujuan berjenjang.
- Pelacakan status permohonan dan riwayat pemakaian.
- Integrasi dengan layanan-aset dan layanan-audit.
- AI untuk deteksi pola permohonan abnormal dan rekomendasi otomatis.

---

## ⚙️ Struktur Folder

```
layanan-pemakaian/
├── cmd/                    # Entry point aplikasi
├── internal/              # Logika utama layanan
│   ├── handler/           # HTTP handler / controller
│   ├── service/           # Business logic
│   ├── repository/        # Akses ke database
│   ├── middleware/        # Middleware keamanan/logging
│   └── model/             # Struktur data dan entitas
├── db/
│   ├── schema/            # SQL schema dan migrasi
│   └── query/             # File SQL untuk sqlc
├── api/
│   └── openapi.yaml       # Spesifikasi API OpenAPI 3.0
├── test/                  # Unit dan integration test
├── Dockerfile
├── Makefile
└── README.md
```

---

## 📡 API Endpoint (contoh)

| Metode | Endpoint                     | Deskripsi                              |
|--------|------------------------------|----------------------------------------|
| GET    | /pemakaian                   | Daftar permohonan pemakaian            |
| POST   | /pemakaian                   | Ajukan permohonan pemakaian baru       |
| GET    | /pemakaian/{id}             | Detail permohonan tertentu             |
| PUT    | /pemakaian/{id}/verifikasi  | Verifikasi atau setujui permohonan     |
| DELETE | /pemakaian/{id}             | Batalkan permohonan                    |

---

## 🤖 Dukungan AI

- Deteksi anomali dari pola pemakaian tidak wajar (AI-UEBA).
- Rekomendasi otomatis berdasarkan riwayat penggunaan satker.
- Validasi permintaan berdasarkan klasifikasi dan jenis aset.
- Pembelajaran aktif dari feedback verifikator.

---

## 🔐 Keamanan

- Hanya pengguna berotorisasi (RBAC) yang dapat mengakses dan mengubah status permohonan.
- Semua aktivitas tercatat di layanan-audit.
- Middleware validasi JWT dan logging penuh.

---

## 📈 Integrasi Layanan

- `layanan-aset`: Validasi aset yang dimohonkan.
- `layanan-audit`: Pencatatan aktivitas dan status proses.
- `layanan-dokumen`: Penyimpanan lampiran dan bukti penggunaan.
- `layanan-ai`: Analisis pola, rekomendasi, dan deteksi anomali.

---

## 🧪 Pengujian

- Unit test menggunakan Go testing framework.
- Integration test untuk API endpoint utama.
- Mock database untuk pengujian isolated.

---

## 📜 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia – SIMPelv2

---

> Layanan Pemakaian memastikan bahwa aset negara dimanfaatkan secara optimal dan transparan oleh setiap satuan kerja.
