# Layanan Distribusi - SIMPEL

**Layanan Distribusi** bertanggung jawab atas proses pendistribusian fisik Barang Milik Negara (BMN) dari pusat ke satuan kerja (satker), antar satker, maupun ke pihak penerima yang berwenang.

Layanan ini memfasilitasi permintaan distribusi, penjadwalan, pelacakan, dan dokumentasi distribusi barang secara efisien dan aman.

---

## 🚚 Fungsi Utama

- Mengelola permintaan distribusi BMN
- Penjadwalan pengiriman barang
- Pencatatan status pengiriman (dalam pengiriman, diterima, gagal kirim)
- Integrasi dengan layanan-pemakaian dan layanan-hibah
- Pelacakan posisi distribusi jika terhubung GPS (opsional)
- Dokumentasi tanda terima, surat jalan, dan bukti fisik

---

## 🔐 Keamanan dan Akses

- Hanya pengguna berperan "Distribusi BMN" yang dapat mengakses fitur utama
- Verifikasi dua langkah untuk permintaan distribusi bernilai besar
- Semua distribusi dicatat dalam layanan-audit

---

## 🧠 Integrasi AI

| Fitur                          | Implementasi AI                          |
|-------------------------------|-------------------------------------------|
| Estimasi waktu pengiriman     | Model prediksi berdasarkan data historis |
| Prioritas jalur distribusi    | AI memilih rute optimal (rule-based + ML)|
| Anomali distribusi            | Deteksi pola pengiriman yang tidak wajar |
| Ringkasan laporan distribusi  | LLM (Phi-2, Gemma) untuk auto-summary     |

---

## 📁 Struktur Direktori

```
layanan-distribusi/
├── api/                 # Handler dan route distribusi
├── model/              # Skema data dan entitas
├── service/            # Logika bisnis distribusi
├── repository/         # Akses ke database
├── middleware/         # Middleware autentikasi dan validasi
├── docs/               # Dokumentasi API
├── tests/              # Unit test dan integrasi
└── README.md
```

---

## 📦 API Endpoint

| Method | Endpoint                  | Deskripsi                                 |
|--------|---------------------------|-------------------------------------------|
| GET    | `/distribusi`             | Daftar permintaan distribusi              |
| POST   | `/distribusi`             | Buat permintaan distribusi baru           |
| PUT    | `/distribusi/{id}`        | Perbarui status distribusi                |
| GET    | `/distribusi/{id}/dokumen`| Ambil dokumen pendukung distribusi        |
| GET    | `/distribusi/lacak/{id}`  | Lacak status distribusi                   |

---

## 🧪 Pengujian

- Unit test dan integrasi berada di folder `tests/`
- Gunakan perintah:

```bash
make test-layanan-distribusi
```

---

## 🧩 Ketergantungan

- `layanan-aset` untuk validasi barang
- `layanan-pemakaian` untuk permintaan distribusi
- `layanan-dokumen` untuk penyimpanan dokumen
- `layanan-audit` untuk pencatatan histori distribusi

---

## 📌 Catatan

Distribusi BMN memegang peran penting dalam menjamin efektivitas penyaluran aset negara. Oleh karena itu, setiap transaksi distribusi harus tervalidasi, terdokumentasi, dan dapat diaudit secara menyeluruh.

---

© 2025 Kejaksaan Republik Indonesia – SIMPEL
