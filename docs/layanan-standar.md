# README - layanan-standar

**`layanan-standar`** adalah microservice dalam SIMPelv2 yang bertanggung jawab untuk mendefinisikan dan mengelola standar jumlah dan spesifikasi Barang Milik Negara (BMN) berdasarkan kategori, jenis pengguna, dan kebutuhan operasional.

---

## 🎯 Tujuan

Menetapkan standar baku untuk perencanaan dan usulan BMN berdasarkan data historis, klasifikasi jabatan, dan jenis satuan kerja.

---

## 🧩 Fitur Utama

- CRUD standar jumlah dan spesifikasi barang
- Penentuan batas minimal-maksimal jumlah berdasarkan tipe satker
- Integrasi dengan `layanan-usulan`, `layanan-rekomendasi`, dan `layanan-aset`
- API untuk validasi usulan terhadap standar
- AI untuk saran spesifikasi barang berdasarkan input kebutuhan

---

## 🛠️ Teknologi

- Go + Gin
- PostgreSQL (skema: `standar`)
- sqlc untuk query database
- JWT middleware dari `layanan-keamanan`
- AI modular (XGBoost, rule-based suggestion)

---

## 🔌 API Endpoint (Contoh)

| Method | Endpoint                     | Deskripsi                               |
|--------|------------------------------|-----------------------------------------|
| GET    | `/api/standar`              | List semua standar aktif                |
| POST   | `/api/standar`              | Tambah standar baru                     |
| PUT    | `/api/standar/:id`          | Update standar berdasarkan ID           |
| GET    | `/api/standar/validasi`     | Validasi input jumlah barang            |
| GET    | `/api/standar/rekomendasi`  | Rekomendasi spesifikasi dari AI         |

---

## 🤖 Dukungan AI

- Rekomendasi spesifikasi barang berdasarkan deskripsi kebutuhan
- Deteksi ketidaksesuaian antara usulan dan standar
- Model ringan berbasis rule-based dan ML (XGBoost)

---

## 🔐 Keamanan

- Hanya role `admin-standar`, `penyusun-standar`, dan `verifikator` yang dapat mengubah data
- Semua endpoint dilindungi JWT
- Aktivitas dicatat ke `layanan-audit`

---

## 🧪 Testing & Validasi

```bash
make test
make lint
make validate-api
```

---

## 🗂️ Struktur Folder

```
layanan-standar/
├── cmd/
├── internal/
│   ├── handler/
│   ├── service/
│   └── repository/
├── db/
│   └── schema.sql
├── docs/
├── main.go
└── go.mod
```

---

## 🧠 Standar ISO yang Relevan

- ISO 55001 → Manajemen aset dan penetapan kebutuhan standar
- ISO 25010 → Kualitas informasi dan spesifikasi teknis
- ISO 20000-1 → Layanan TI yang mendukung proses pengusulan dan penyediaan

---

## 👥 Kontribusi

Gunakan branch `fitur/*` atau `bugfix/*` dan ajukan Merge Request ke `dev`.
Semua kontribusi diverifikasi oleh tim pengelola BMN pusat dan AI engineer.

---

> Microservice ini memastikan bahwa pengadaan BMN tidak melebihi kebutuhan wajar dan sesuai dengan standar nasional yang berlaku.

---
