# layanan-integrasi

**layanan-integrasi** adalah microservice yang menangani sinkronisasi data dan interopabilitas antara SIMPEL v2 dengan sistem eksternal seperti MONSAKTI, SIMAN, dan MySIMKARI.

---

## 🎯 Tujuan

- Sinkronisasi data aset dan persediaan dari MONSAKTI.
- Integrasi data master pegawai dari MySIMKARI.
- Pelaporan data aset ke sistem SIMAN (DJKN).
- Menjamin konsistensi data antar platform pemerintah.

---

## ⚙️ Teknologi

- **Bahasa**: Rust
- **Framework**: Axum
- **Penjadwal**: tokio-cron-scheduler
- **Integrasi**: gRPC & REST API

---

## 📦 Lokasi Kode

`layanan/integrasi/`

---

## 📝 Catatan Implementasi

Layanan ini sudah diimplementasikan sebagai microservice terpisah berbasis Rust. Dokumentasi teknis lebih lanjut mengenai sinkronisasi spesifik tersedia di folder `layanan/integrasi/`.
