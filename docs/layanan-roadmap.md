# layanan-roadmap

**Layanan Roadmap SIMPelv2** adalah microservice yang menangani perencanaan jangka menengah dan panjang atas kebutuhan serta pengelolaan Barang Milik Negara (BMN). Layanan ini mendukung proses penyusunan roadmap aset strategis untuk optimalisasi pengelolaan BMN lintas tahun.

---

## 🎯 Tujuan

- Merancang roadmap kebutuhan dan alokasi BMN untuk 3–5 tahun ke depan.
- Memberikan proyeksi otomatis berdasarkan histori dan pola usulan.
- Memberikan dasar kebijakan perencanaan jangka panjang yang selaras dengan RENSTRA dan kebutuhan satker.

---

## 📦 Fitur Utama

- Manajemen entri dan revisi roadmap kebutuhan BMN
- Integrasi data historis usulan, pengadaan, dan realisasi
- Proyeksi AI untuk tren kebutuhan BMN per kategori dan wilayah
- Tampilan visualisasi roadmap multi-tahun
- Dukungan rekomendasi prioritas berbasis risiko dan nilai manfaat

---

## ⚙️ Teknologi

- Bahasa: Go (Gin Framework)
- Database: PostgreSQL (skema: `roadmap`)
- Integrasi: layanan-usulan, layanan-aset, layanan-rekomendasi
- AI Opsional: Analisis tren kebutuhan & prediksi roadmap
- Standar: JSON REST API + JWT Auth + RBAC (via authenc)

---

## 🧠 Kontribusi AI

| Fungsi AI                    | Teknologi / Model            |
|-----------------------------|------------------------------|
| Proyeksi kebutuhan jangka panjang | XGBoost, Prophet, Mistral-7B |
| Deteksi perubahan pola kebutuhan | Rule-based + Anomaly ML       |
| Simulasi kebijakan aset     | LLM kecil + skenario berbasis aturan |

---

## 📁 Struktur Folder

```
layanan-roadmap/
├── cmd/                      # Entry point aplikasi
├── internal/
│   ├── handler/              # HTTP handler
│   ├── service/              # Logika bisnis roadmap
│   ├── repository/           # Query SQL (via sqlc)
│   └── model/                # Definisi struktur data
├── db/
│   └── migrations/           # Skrip migrasi roadmap
├── api/                      # Definisi OpenAPI / Swagger
└── README.md
```

---

## 🔐 Keamanan

- Semua endpoint dilindungi JWT dan role dari authenc.
- Audit trail dicatat otomatis via middleware logging.
- Validasi input disertai sanitasi dan pembatasan input numerik.

---

## 📄 API Penting

| Endpoint                      | Method | Deskripsi                          |
|------------------------------|--------|-----------------------------------|
| `/roadmap`                   | GET    | Ambil seluruh roadmap aktif       |
| `/roadmap/:id`               | GET    | Detail satu roadmap               |
| `/roadmap`                   | POST   | Buat roadmap baru                 |
| `/roadmap/:id`               | PUT    | Perbarui roadmap                  |
| `/roadmap/:id`               | DELETE | Hapus roadmap                     |
| `/roadmap/proyeksi`          | GET    | Ambil data hasil proyeksi AI      |
| `/roadmap/simulasi-kebijakan`| POST   | Simulasi skenario kebijakan BMN   |

---

## 📌 Catatan

- Roadmap tidak mengikat, namun menjadi dasar awal perencanaan tahun berjalan.
- Setiap roadmap dapat dikaitkan dengan aset, jenis kebutuhan, satker, dan zona wilayah.
- Perubahan roadmap harus disertai alasan dan disimpan dalam histori perubahan.

---

## 🧪 Pengujian

- Unit test disimpan di `internal/service/` dan `internal/handler/`
- Disarankan coverage >85%
- Jalankan test dengan:

```bash
go test ./...
```

---

## 📬 Kontak Tim

Untuk pertanyaan lebih lanjut, silakan hubungi tim roadmap:

📧 roadmap-support@kejaksaan.go.id

---

> Roadmap BMN adalah fondasi perencanaan strategis jangka panjang Kejaksaan dalam mengelola aset negara secara berkelanjutan dan terintegrasi.
