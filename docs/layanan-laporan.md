# README – layanan-laporan

## 🧾 Deskripsi Singkat
`layanan-laporan` adalah layanan mikro dalam platform **SIMPEL** yang bertanggung jawab atas penyusunan laporan berkala, rekapitulasi lintas layanan, dan pelaporan tematik atas aktivitas pengelolaan Barang Milik Negara (BMN). Laporan ini mendukung pengambilan keputusan strategis, pemantauan kinerja, dan kepatuhan regulasi.

---

## 🎯 Tujuan Utama
- Menyediakan laporan bulanan, triwulan, semesteran, dan tahunan secara otomatis
- Menghasilkan rekapitulasi lintas layanan dan unit kerja
- Mendukung ekspor laporan dalam format PDF, Excel, dan CSV
- Memberikan ringkasan naratif berbasis AI dari seluruh data

---

## 🧱 Fitur Utama
- 📄 Laporan agregat semua layanan SIMPEL
- 🧠 Ringkasan berbasis LLM dari indikator dan kinerja
- 📊 Ekspor laporan ke PDF, Excel, dan CSV
- 📆 Jadwal otomatis pembuatan laporan
- 🔍 Filter laporan berdasarkan satker, kategori, atau waktu
- 🧩 Rekomendasi kebijakan berbasis pola data

---

## ⚙️ Teknologi
- Backend: Go (Gin) + sqlc
- Database: PostgreSQL (join antar skema layanan)
- AI: Gemma-2B, LLaMA3-3B (summarization & insight extraction)
- Ekspor Dokumen: `gofpdf`, `excelize`, `csv-writer`
- Scheduler: Cron job / penjadwalan manual
- Frontend (melalui antarmuka): Komponen visualisasi laporan

---

## 📁 Struktur Direktori
```
layanan-laporan/
├── api/                # Endpoint REST
├── internal/           # Logika layanan dan model
│   ├── laporan/        # Agregator laporan per jenis
│   ├── summary/        # Modul summarization dengan AI
│   └── exporter/       # PDF, Excel, CSV exporter
├── repository/         # Query SQLC
├── schemas/            # Skema request/response
├── job/                # Penjadwalan laporan rutin
├── config/             # Konfigurasi kategori, periode, format
└── main.go
```

---

## 🔄 Contoh Endpoint
| Metode | Endpoint                    | Keterangan                              |
|--------|-----------------------------|-----------------------------------------|
| GET    | `/laporan/ringkasan`        | Ringkasan indikator semua layanan       |
| GET    | `/laporan/bulanan`          | Laporan bulanan seluruh unit kerja      |
| POST   | `/laporan/export/pdf`       | Ekspor laporan ke PDF                   |
| POST   | `/laporan/export/excel`     | Ekspor laporan ke Excel                 |
| POST   | `/laporan/export/csv`       | Ekspor laporan ke CSV                   |
| GET    | `/laporan/:id`              | Lihat laporan yang telah dibuat         |

---

## 📈 Tipe Laporan
- **Laporan Realisasi**: Pemakaian, distribusi, pemeliharaan
- **Laporan Perencanaan**: Usulan, roadmap, kebutuhan mendatang
- **Laporan Kinerja**: Indikator strategis dan indikator teknis
- **Laporan Audit**: Aktivitas pengguna dan transaksi penting
- **Laporan Tematik**: Fokus per wilayah/satker/periode tertentu

---

## 🤖 Kontribusi AI/ML
| Fungsi Laporan         | Teknologi AI/ML                            |
|------------------------|--------------------------------------------|
| Ringkasan laporan      | LLM: Gemma-2B, LLaMA3-3B                   |
| Analisis narasi        | Phi-2 + klasifikasi naratif               |
| Deteksi pola anomali   | ML klasifikasi + rule-based patterns      |
| Rekomendasi kebijakan  | Model summarization + knowledge rule      |
| Insight deviasi tren   | Time-series anomaly detector              |

---

## 🔐 Akses & Validasi
- Hanya pengguna dengan role `admin`, `monitoring`, atau `pimpinan` yang dapat mengakses dan mengekspor
- Validasi parameter periode, satker, dan format secara ketat
- Semua aktivitas dicatat oleh `layanan-audit`

---

## 🔗 Integrasi Lintas Layanan
| Layanan Terkait         | Data yang Digunakan                    |
|-------------------------|----------------------------------------|
| `layanan-usulan`        | Jumlah dan status usulan               |
| `layanan-pemakaian`     | Rekapitulasi pemakaian                 |
| `layanan-pengalihan`    | Status pengalihan                      |
| `layanan-hibah`         | Rangkuman pemberian hibah             |
| `layanan-aset`          | Status dan kategori aset               |
| `layanan-rekomendasi`   | Jumlah & jenis yang disarankan         |
| `layanan-roadmap`       | Data rencana dan kebutuhan mendatang   |
| `layanan-penilaian`     | Hasil nilai aset dan kesimpulan        |
| `layanan-audit`         | Log aktivitas pengguna                 |
| `layanan-konfigurasi`   | Kategori laporan, format, dan referensi|
| `layanan-dasbor`        | Ringkasan tren                         |
| `layanan-ai`            | Ringkasan naratif dan analitik AI      |

---

## 🧪 Testing
- Unit test untuk setiap modul generator dan exporter
- Integration test dengan mock data dari layanan lain
- Benchmark waktu query, proses AI, dan rendering dokumen

---

## 🚀 Rencana Pengembangan
- Integrasi ke `layanan-integrasi` untuk penarikan laporan eksternal
- Auto-notifikasi laporan penting ke pengguna terkait
- Template laporan dinamis (drag-and-drop + builder)
- Pelabelan otomatis laporan berdasarkan isi

---

## 📩 Kontak & Dukungan
Jika menemukan bug, ide, atau masukan:
- Gunakan GitLab Issues untuk laporan dan diskusi
- Email tim teknis: `simpelv2-support@kejaksaan.go.id`

---

## 📝 Lisensi
Hak Cipta © 2025 Kejaksaan Republik Indonesia – SIMPEL Internal Use Only
