# README - layanan-pemeliharaan

## 🛠️ Layanan Pemeliharaan

Layanan ini menangani proses **pemeliharaan aset BMN**, termasuk input rencana dan realisasi pemeliharaan, pencatatan siklus kerusakan, serta pelaporan kebutuhan anggaran pemeliharaan barang. Layanan ini juga terhubung dengan MONSAKTI untuk sinkronisasi realisasi anggaran.

---

## 🎯 Tujuan

- Mencatat rencana dan realisasi pemeliharaan aset
- Melacak siklus rusak → pemeliharaan → kembali layak
- Menghitung estimasi biaya berdasarkan riwayat
- Menyinkronkan data realisasi anggaran dari MONSAKTI

---

## 🧱 Teknologi

- **Bahasa Pemrograman**: Go (Gin Framework)
- **ORM**: SQLC
- **Database**: PostgreSQL (skema: `pemeliharaan`)
- **Auth**: JWT, middleware Fiber Gateway
- **API**: RESTful + OpenAPI Spec
- **AI Terintegrasi**:
  - Prediksi siklus kerusakan aset
  - Estimasi rencana pemeliharaan
  - Analisis efektivitas anggaran

---

## 📁 Struktur Folder

```
layanan-pemeliharaan/
├── api/                  # Definisi OpenAPI
├── db/
│   ├── schema.sql        # Skema PostgreSQL
│   └── query.sql         # Query SQL untuk sqlc
├── internal/
│   ├── handler/          # HTTP handler
│   ├── service/          # Business logic
│   ├── model/            # Struct data
│   └── middleware/       # Middleware tambahan (jika perlu)
├── oai/                  # File OpenAPI 3.0 YAML/JSON
├── scripts/              # Seeder, migrasi, tools
└── main.go               # Entry point layanan
```

---

## 🔐 Akses & Keamanan

- Setiap request harus melalui Fiber Gateway
- Validasi JWT dan peran pengguna (RBAC)
- Semua endpoint dilindungi oleh skema otorisasi berbasis peran
- Logging dilakukan oleh layanan-audit

---

## 🔗 Integrasi Eksternal

- **MONSAKTI**: Sinkronisasi realisasi anggaran pemeliharaan
- **layanan-aset**: Validasi aset dan status kondisi
- **layanan-ai**: Prediksi rusak ringan/berat dan estimasi waktu kerusakan

---

## 🔄 Endpoint Utama

| Method | Endpoint                       | Deskripsi                                      |
|--------|--------------------------------|-------------------------------------------------|
| GET    | /rencana                      | Daftar rencana pemeliharaan                    |
| POST   | /rencana                      | Tambah rencana pemeliharaan                    |
| GET    | /realisasi                    | Daftar realisasi pemeliharaan                  |
| POST   | /sinkron/monsakti             | Sinkronisasi dengan MONSAKTI                   |
| GET    | /prediksi/siklus/:id_aset     | Prediksi siklus kerusakan berdasarkan AI       |
| GET    | /anggaran/estimasi/:id_aset   | Estimasi kebutuhan anggaran pemeliharaan       |

---

## 🧠 Fitur AI

- **Prediksi Kerusakan**: Model menganalisis histori kerusakan aset untuk mengestimasi kerusakan berikutnya.
- **Rekomendasi Jadwal**: Sistem menyarankan kapan aset sebaiknya dipelihara untuk menghindari rusak berat.
- **Evaluasi Efisiensi Biaya**: AI menghitung efektivitas biaya pemeliharaan dari tahun ke tahun.

---

## 🧪 Testing

Gunakan Postman atau Insomnia untuk mencoba API berikut:

```http
GET /prediksi/siklus/ID1234
Authorization: Bearer <token>
```

Unit test tersedia di folder `internal/service` dengan framework `testing` bawaan Go.

---

## 🚀 Perintah Penting

```bash
make build              # Build binary
make run                # Jalankan server lokal
make lint               # Cek format dan standar Go
make migrate            # Migrasi skema ke PostgreSQL
```

---

## 📝 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia. Layanan ini bersifat tertutup dan hanya digunakan untuk pengelolaan internal BMN.

---

> "Layanan Pemeliharaan membantu memastikan aset negara tetap dalam kondisi optimal melalui pencatatan, analisis, dan perencanaan yang cerdas."
