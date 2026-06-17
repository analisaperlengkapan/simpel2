# Layanan Hibah - SIMPEL

> **Catatan untuk Pengembangan Masa Depan:** Dokumen ini mendeskripsikan rencana pengembangan layanan ini yang saat ini belum diimplementasikan secara penuh dalam codebase. Dokumen ini dapat digunakan sebagai acuan untuk pengembangan ke depan agar tetap selaras dengan visi arsitektur SIMPEL v2 yang efektif, efisien, dan optimal.


Layanan ini bertanggung jawab atas pengajuan, penerimaan, pencatatan, dan pelaporan hibah Barang Milik Negara (BMN) dalam Sistem Informasi Manajemen Pengelolaan BMN Versi 2 (**SIMPEL**).

---

## 🎯 Tujuan

- Memfasilitasi pengajuan hibah dari unit/satker.
- Mencatat dan menyetujui hibah dari/ke pihak eksternal.
- Menyediakan riwayat dan laporan hibah BMN.
- Menyediakan dukungan AI untuk analisis dan klasifikasi pola hibah.

---

## 🧱 Struktur Folder

```
layanan-hibah/
├── cmd/                   # Entry point aplikasi
├── internal/              # Logika bisnis
│   ├── handler/           # Handler HTTP
│   ├── service/           # Service business logic
│   ├── repository/        # Koneksi dan kueri DB
│   └── model/             # Struktur data
├── proto/                 # Protobuf untuk gRPC (opsional)
├── openapi/               # Spesifikasi OpenAPI
├── migrations/            # Skema database (SQL)
├── docs/                  # Dokumentasi lokal
├── test/                  # Unit test
└── main.go                # Main program
```

---

## 🔌 API Endpoint Utama

| Metode | Endpoint                     | Deskripsi                             |
|--------|------------------------------|----------------------------------------|
| GET    | `/hibah`                    | Daftar semua hibah                    |
| POST   | `/hibah/pengajuan`          | Buat pengajuan hibah baru             |
| PUT    | `/hibah/persetujuan/{id}`   | Menyetujui / menolak pengajuan        |
| GET    | `/hibah/{id}`               | Detail hibah tertentu                 |
| GET    | `/hibah/riwayat`            | Riwayat hibah per satker              |

---

## 🔐 Keamanan

- Otentikasi JWT dari `authenc`
- Validasi RBAC dari Fiber Gateway (`gerbang`)
- Audit trail otomatis melalui `layanan-audit`

---

## 🤖 Dukungan AI

- Klasifikasi jenis hibah (hibah masuk/keluar)
- Deteksi pola penyaluran abnormal
- Rekomendasi kelayakan hibah berdasarkan data historis

---

## 🔗 Integrasi

- **layanan-aset** → Ambil data BMN yang dapat dihibahkan
- **layanan-dokumen** → Simpan surat pengajuan & persetujuan
- **layanan-audit** → Mencatat aktivitas pengajuan dan persetujuan hibah
- **layanan-dasbor** → Menyediakan data rekap hibah

---

## 🧪 Pengujian Lokal

```bash
docker-compose up -d layanan-hibah
curl http://localhost:8080/hibah
```

---

## 📜 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia
