# README.md - layanan-rekomendasi

**Layanan Rekomendasi** adalah salah satu komponen inti dalam SIMPelv2 yang bertanggung jawab untuk menghasilkan rekomendasi jumlah, spesifikasi, dan prioritas kebutuhan Barang Milik Negara (BMN) berdasarkan data historis, standar, dan konteks kebutuhan instansi.

---

## 🎯 Tujuan

Memberikan rekomendasi otomatis dan berbasis AI kepada pengguna dalam proses perencanaan pengadaan atau pemenuhan kebutuhan BMN.

---

## 🧩 Fungsi Utama

- Rekomendasi jumlah ideal berdasarkan pola penggunaan dan standar
- Rekomendasi spesifikasi berdasarkan klasifikasi barang
- Penentuan prioritas pemenuhan berdasarkan urgensi dan kondisi
- Integrasi dengan standar dari `layanan-standar`
- Dukungan AI: XGBoost, Rule-based system, DecisionTree

---

## ⚙️ Teknologi

- Bahasa: Go (Gin)
- Database: PostgreSQL (skema `rekomendasi`)
- AI: XGBoost, Rule Engine (pada `layanan-ai`)
- Autentikasi: Middleware dari `authenc`
- Logging & Audit: via `layanan-audit`

---

## 📦 Struktur Direktori

```
layanan-rekomendasi/
├── cmd/                      # Entry point service
├── internal/
│   ├── handler/              # HTTP handler & routing
│   ├── service/              # Logika rekomendasi
│   ├── repository/           # Akses DB PostgreSQL
│   ├── model/                # Struct dan DTO
│   └── ruleset/              # Rule-based logic
├── config/                   # Konfigurasi environment
├── sql/                      # Query sqlc & migrasi
├── docs/                     # Dokumentasi teknis
├── main.go                   # Entry utama
└── Dockerfile                # Build container
```

---

## 🔌 API Endpoint

| Metode | Endpoint                       | Deskripsi                                |
|--------|--------------------------------|-------------------------------------------|
| GET    | `/rekomendasi/barang/:id`     | Rekomendasi jumlah dan spesifikasi        |
| POST   | `/rekomendasi/generate`       | Generate rekomendasi dari input kebutuhan |
| GET    | `/rekomendasi/prioritas`      | Daftar prioritas kebutuhan instansi       |


---

## 🧠 Integrasi AI

Layanan ini bekerja sama dengan `layanan-ai` untuk:

- Mendapatkan hasil prediksi jumlah ideal berdasarkan tren
- Mengakses hasil inference klasifikasi barang
- Menggabungkan hasil Rule-based dengan model ML untuk rekomendasi akhir

---

## 📑 Contoh Output

```json
{
  "barang_id": "BRG-001",
  "rekomendasi_jumlah": 5,
  "spesifikasi_disarankan": "Laptop i5 RAM 16GB SSD 512GB",
  "tingkat_prioritas": "Tinggi"
}
```

---

## 🛡️ Keamanan & Validasi

- Semua endpoint menggunakan middleware otentikasi JWT
- Validasi data masukan dengan skema JSONSchema
- Akses data berdasarkan peran dan unit kerja pengguna

---

## 🧪 Testing & Validasi

- Unit test pada service logika rekomendasi
- Integration test dengan mock DB & mock AI API
- Linting dengan `golangci-lint`

---

## 👥 Kontribusi

Ikuti standar kontribusi SIMPelv2. Untuk kontribusi AI, buat branch:

```
ai/rekomendasi-jumlah-xgboost
```

Dan sertakan hasil evaluasi model (akurasi, F1, latency) di folder `evaluasi/`.

---

## 📝 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia – Dilarang menyebarluaskan kode ini tanpa izin tertulis.

---

> Layanan ini mendukung pengambilan keputusan berbasis data dan AI dalam pengelolaan kebutuhan BMN.
