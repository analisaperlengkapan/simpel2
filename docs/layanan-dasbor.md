# README – layanan-dasbor

## 🧾 Deskripsi Singkat
`layanan-dasbor` adalah layanan mikro dalam platform **SIMPelv2** yang bertanggung jawab atas penyajian data secara visual, ringkas, dan interaktif dari berbagai layanan terkait pengelolaan Barang Milik Negara (BMN). Dasbor ini menjadi titik sentral monitoring dan pengambilan keputusan oleh pimpinan maupun operator teknis.

---

## 🎯 Tujuan
- Memberikan tampilan real-time kondisi pengelolaan BMN secara menyeluruh
- Mendukung pengambilan keputusan berbasis data
- Mendeteksi anomali dan progres layanan
- Menyajikan insight dari data operasional, usulan, roadmap, hingga audit

---

## 🧱 Fitur Utama
- 📊 Ringkasan jumlah dan status aset, usulan, distribusi, pemakaian, dan roadmap
- 🧠 Ringkasan otomatis berbasis AI dari narasi dan laporan
- 📈 Visualisasi tren kebutuhan dan pemanfaatan BMN
- 🔍 Fitur pencarian lintas layanan (dengan filter dan sort)
- 🗺️ Peta distribusi aset secara geografis
- 📡 Sinkronisasi rutin dari semua layanan mikro lainnya

---

## ⚙️ Teknologi
- Backend: Go (Gin) + sqlc
- Database: PostgreSQL (skema `dasbor`), read-only dari layanan lain
- Frontend: Komponen React Vite (`antarmuka/`) + Recharts/D3
- AI: Summarization & deteksi outlier menggunakan LLM ringan (Gemma, Phi-2)
- Sinkronisasi: Scheduled job via `job/` + Redis cache (opsional)

---

## 📁 Struktur Direktori
```
layanan-dasbor/
├── api/             # Endpoint REST
├── service/         # Logika penyatuan dan agregasi data
├── query/           # SQL read-only untuk visualisasi
├── model/           # Struktur dan definisi response dasbor
├── dashboard/       # Template JSON visual layout
├── config/          # Konfigurasi dan metadata dasbor
├── summarizer/      # Modul ringkasan naratif (AI)
├── job/             # Sinkronisasi berkala lintas layanan
├── cache/           # Cache Redis (jika digunakan)
├── main.go
└── README.md
```

---

## 🔄 Contoh Endpoint
| Metode | Endpoint                    | Keterangan                            |
|--------|-----------------------------|---------------------------------------|
| GET    | `/dasbor`                  | Agregasi data dari semua layanan      |
| GET    | `/dasbor/aset`             | Statistik aset                        |
| GET    | `/dasbor/usulan`           | Statistik usulan kebutuhan            |
| GET    | `/dasbor/pemakaian`        | Status pemakaian & permohonan         |
| GET    | `/dasbor/ringkasan`        | Ringkasan naratif seluruh data        |
| POST   | `/dasbor/sinkronisasi`     | Inisiasi tarik ulang data             |

---

## 📈 Visualisasi yang Didukung
- Line chart, bar chart, pie chart
- KPI widget dengan indikator warna
- Heatmap dan distribusi spasial
- Tabel interaktif dan filter

---

## 🤖 Kontribusi AI (Opsional)
- **Summarization**: ringkasan otomatis laporan satker
- **Trend prediction**: identifikasi tren kebutuhan
- **Anomaly detection**: deteksi progres abnormal
- **Visual embedding**: clustering narasi berdasarkan kemiripan

---

## 🔐 Akses & Validasi
- Hanya pengguna role `admin`, `monitoring`, atau `pimpinan` yang dapat mengakses
- Semua akses dicatat dalam `layanan-audit`
- Query dibatasi ke data yang diotorisasi oleh `layanan-keamanan`

---

## 🔗 Integrasi Layanan
| Layanan Terkait        | Data yang Diambil                         |
|------------------------|-------------------------------------------|
| `layanan-aset`         | Status, lokasi, nilai aset                |
| `layanan-usulan`       | Jumlah dan jenis usulan                   |
| `layanan-roadmap`      | Progress per tahap                       |
| `layanan-hibah`        | Nilai dan arah hibah                     |
| `layanan-pemeliharaan` | Realisasi teknis dan anggaran            |
| `layanan-laporan`      | Narasi bulanan untuk diringkas           |
| `layanan-audit`        | Aktivitas lintas layanan                 |

---

## 📦 Konfigurasi `.env`
```env
SUMMARIZER_API=http://layanan-ai:8080/summarize
CACHE_REDIS_URL=redis://redis:6379
SINKRONISASI_CRON=*/15 * * * *
```

---

## 📚 Tips Pengembangan
- Gunakan view SQL untuk agregasi kompleks
- Gunakan cache untuk menurunkan latency beban tinggi
- Format visualisasi disimpan sebagai JSON layout agar dinamis
- Ringkasan naratif dapat disesuaikan berdasarkan peran pengguna

---

## 📝 Lisensi
Hak Cipta © 2025 Kejaksaan Republik Indonesia – SIMPelv2 Internal Use Only
