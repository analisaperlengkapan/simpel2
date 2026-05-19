# README - layanan-aset

**layanan-aset** adalah microservice utama yang mengelola master data aset Barang Milik Negara (BMN) dalam sistem SIMPEL. Layanan ini menjadi fondasi utama integritas data aset nasional, mulai dari identitas aset, status kepemilikan, klasifikasi, hingga histori mutasi.

---

## 🎯 Tujuan

- Menyediakan API CRUD untuk entitas aset BMN
- Menjamin integritas dan konsistensi data aset
- Menyediakan fitur validasi klasifikasi dan kondisi aset
- Mendukung klasifikasi otomatis aset dengan AI

---

## 🧱 Teknologi

- **Bahasa**: Go (Gin framework)
- **ORM/Query**: sqlc (typed SQL → Go)
- **Database**: PostgreSQL (schema: `aset`)
- **AI opsional**: LLM + rule-based untuk klasifikasi
- **Testing**: Go test + TestContainer (opsional)

---

## 📦 Struktur Direktori

```
layanan-aset/
├── api/              # Handler HTTP dan routing
├── db/               # Query SQL dan schema PostgreSQL
├── model/            # Struct Go untuk entitas aset
├── service/          # Logika bisnis & validasi
├── middleware/       # Middleware khusus layanan aset
├── ai/               # Opsional: klasifikasi AI untuk jenis/kondisi
├── test/             # Unit test dan integrasi
├── Dockerfile
├── Makefile
└── README.md
```

---

## 📚 API Endpoint (Contoh)

| Method | Endpoint               | Deskripsi                          |
|--------|------------------------|-----------------------------------|
| GET    | /aset                  | List semua aset                    |
| POST   | /aset                  | Tambah aset baru                   |
| GET    | /aset/:id              | Ambil detail aset berdasarkan ID  |
| PUT    | /aset/:id              | Ubah data aset                     |
| DELETE | /aset/:id              | Hapus aset                         |
| POST   | /aset/klasifikasi-ai   | (Opsional) Klasifikasi AI          |

---

## 🔐 Keamanan

- Middleware JWT untuk otentikasi antar layanan
- Validasi input dan sanitasi data
- Hanya pengguna dengan role `admin` atau `verifikator` dapat menghapus atau mengubah aset

---

## 🧠 Fitur AI (Opsional)

Jika AI diaktifkan, layanan ini mendukung:

- Klasifikasi jenis aset berdasarkan nama dan deskripsi (LLM + rule-based)
- Validasi kategori aset berdasarkan katalog BMN
- Penilaian kondisi awal otomatis dari input gambar atau teks (butuh layanan-ai)

---

## 🧪 Testing

Jalankan unit test:

```bash
make test
```

---

## 📄 Environment

Variabel penting:

- `DB_URL` – URL koneksi database PostgreSQL
- `JWT_SECRET` – secret key untuk validasi token
- `AI_SERVICE_URL` – (opsional) endpoint layanan-ai untuk klasifikasi

---

## 📝 Lisensi & Hak Akses

Proyek ini bagian dari sistem internal Kejaksaan Republik Indonesia. Setiap pengakses wajib menjaga kerahasiaan data aset negara. Penyalahgunaan data dikenakan sanksi administratif dan pidana sesuai UU ITE dan peraturan pengelolaan BMN.

---

> Untuk informasi integrasi, dokumentasi OpenAPI tersedia di: `/docs/openapi.yaml`
> Kontak PIC layanan: `aset-support@kejaksaan.go.id`
