# README – layanan-integrasi

## 🔗 Deskripsi Singkat
`layanan-integrasi` merupakan layanan mikro SIMPelv2 yang bertugas menjembatani pertukaran data antara SIMPelv2 dan sistem eksternal pemerintah seperti SIMAN, MONSAKTI, MySimkari, dan SIPEDE. Layanan ini dirancang untuk menjaga integritas, keamanan, dan keteraturan proses sinkronisasi data.

---

## 🎯 Tujuan Utama
- Menyediakan konektivitas aman dengan sistem eksternal
- Menarik dan menyimpan data secara periodik dan on-demand
- Menyediakan API internal yang distandarisasi untuk data eksternal
- Melakukan validasi, normalisasi, dan transformasi data sesuai skema SIMPelv2
- Menyimpan histori sinkronisasi dan audit terpusat

---

## 🔌 Sistem Terintegrasi
| Sistem Eksternal | Frekuensi       | Jenis Data yang Ditarik                    |
|------------------|-----------------|--------------------------------------------|
| SIMAN            | Mingguan        | Aset, pemanfaatan, penghapusan BMN         |
| MONSAKTI         | Harian (00:30)  | Realisasi anggaran pemeliharaan            |
| MySimkari        | Harian/manual   | Data kepegawaian                           |
| SIPEDE           | Bulanan         | Distribusi dan pemindahtanganan wilayah    |

---

## 🧱 Fitur Utama
- 🔄 Sinkronisasi otomatis berbasis cron dan manual trigger
- 🔒 Dukungan API Key, OAuth2, dan whitelist IP
- 🔁 Normalisasi struktur & mapping antar format JSON/XML/CSV
- 📊 Statistik integrasi dan pelaporan kegagalan
- 🧾 Logging lengkap transaksi & audit sinkronisasi

---

## ⚙️ Teknologi
- Backend: Go (Gin) + SQLC
- Database: PostgreSQL (skema `integrasi`)
- Scheduler: Cron job internal & queue-based worker
- Eksternal API: REST, SOAP, dan SFTP (CSV/XML)
- Middleware: Token validator + IP filter
- Secrets: Sealed Secrets untuk token API

---

## 📁 Struktur Direktori
```
layanan-integrasi/
├── api/                # Handler endpoint & routing internal
├── handler/            # Middleware & routing
├── service/            # Logika sinkronisasi & transformasi
├── job/                # Cron & background worker
├── connector/          # Client eksternal: SIMAN, MONSAKTI, dll
├── repository/         # Query SQL dan transaksi data
├── model/              # Struktur data internal & eksternal
├── config/             # Konfigurasi sistem & koneksi
├── util/               # Parser, formatter, validator
└── main.go             # Entry point layanan
```

---

## 🔄 Contoh Endpoint Internal
| Metode | Endpoint                        | Fungsi                                     |
|--------|----------------------------------|--------------------------------------------|
| POST   | `/sinkron/siman`                | Tarik data aset dari SIMAN                 |
| POST   | `/sinkron/monsakti`             | Tarik realisasi anggaran dari MONSAKTI     |
| POST   | `/sinkron/mysimkari`            | Tarik data kepegawaian dari MySimkari      |
| POST   | `/sinkron/sipede`               | Tarik distribusi dari SIPEDE               |
| GET    | `/sinkron/histori`              | Lihat histori dan status integrasi         |

---

## 🔐 Keamanan & Validasi
- Koneksi terenkripsi (HTTPS, SFTP, VPN bila diperlukan)
- API Key unik per sistem eksternal
- Validasi format JSON/XML dengan schema
- Validasi checksum file untuk integritas
- Audit log otomatis via `layanan-audit`

---

## 📦 Contoh .env Konfigurasi
```env
SIMAN_API_URL=https://siman.kemenkeu.go.id/api
SIMAN_API_KEY=secretsiman
MONSAKTI_SFTP_HOST=sftp.monsakti.go.id
MONSAKTI_SFTP_USER=simpelv2
MONSAKTI_SFTP_PASS=secret
MYSIMKARI_API_URL=https://mysimkari.setneg.go.id/api
MYSIMKARI_API_TOKEN=xxxxx
SIPEDE_ENDPOINT=https://sipede.setjen.kemendagri.go.id/api
```

---

## 📌 Integrasi Terkait Layanan SIMPelv2
| Layanan               | Fungsi Integrasi                            |
|------------------------|---------------------------------------------|
| `layanan-aset`         | Pembaruan aset dari SIMAN                   |
| `layanan-pemeliharaan` | Realisasi anggaran dari MONSAKTI            |
| `layanan-konfigurasi`  | Referensi wilayah dan instansi dari SIPEDE  |
| `layanan-audit`        | Audit sinkronisasi dan histori API          |
| `authenc`     | Validasi autentikasi token integrasi        |

---

## 🚦 Standar Implementasi
- Struktur response mengikuti format OpenAPI
- Semua konfigurasi dikelola via YAML dan `.env`
- Cron disesuaikan dengan tingkat sensitivitas data
- Sinkronisasi otomatis dapat dibatalkan oleh admin
- Validasi data dilakukan sebelum disimpan ke DB

---

## 📝 Lisensi
Hak Cipta © 2025 Kejaksaan Republik Indonesia – SIMPelv2 Internal Use Only

> Layanan Integrasi adalah penghubung krusial yang memastikan bahwa SIMPelv2 tetap selaras dengan sistem nasional dalam pengelolaan Barang Milik Negara.
