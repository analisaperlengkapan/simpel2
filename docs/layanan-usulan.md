# README - layanan-usulan

**Layanan Usulan** dalam SIMPelv2 menangani proses pencatatan dan pengajuan usulan kebutuhan Barang Milik Negara (BMN), mulai dari perencanaan kebutuhan, referensi standar, hingga pencatatan alokasi awal kebutuhan BMN.

---

## 📌 Deskripsi Singkat

Layanan ini memungkinkan pengguna menyusun dan mengajukan usulan kebutuhan BMN berdasarkan standar yang berlaku, data historis, serta hasil analisis kecerdasan buatan. Setiap usulan akan dianalisis secara kuantitatif dan kualitatif untuk mendukung perencanaan strategis pengadaan BMN.

---

## 📦 Fitur Utama

- Formulir usulan kebutuhan dengan validasi otomatis
- Integrasi referensi standar dan master data
- AI/ML prediksi kebutuhan dan prioritas
- Simulasi kebutuhan jangka pendek dan menengah
- Workflow persetujuan dan validasi
- Rekap dan ekspor usulan (PDF/Excel)

---

## 🧠 Kontribusi AI

| Modul AI               | Teknologi Digunakan                | Fungsi                                                   |
|------------------------|------------------------------------|-----------------------------------------------------------|
| Prediksi kebutuhan     | XGBoost + Historical Data          | Memprediksi kuantitas kebutuhan berdasarkan tren historis |
| Saran standar          | Rule-based + LLM (Gemma/LLAMA3)    | Memberikan saran spesifikasi sesuai regulasi              |
| Klasifikasi jenis usulan| Supervised Learning + spaCy        | Mengklasifikasi usulan ke dalam kategori prioritas        |

---

## 🏗️ Struktur Direktori

```
layanan-usulan/
├── cmd/                       # Entry-point server
├── internal/
│   ├── handler/               # HTTP handler
│   ├── service/               # Logika bisnis
│   ├── repository/            # Query database
│   └── schema.sql             # Skema DB usulan
├── api/
│   └── usulan_openapi.yaml    # Dokumentasi OpenAPI
├── docs/
│   └── contoh-formulir.pdf    # Contoh dokumen
├── test/
│   └── service_test.go        # Pengujian unit
└── README.md
```

---

## 🔐 Keamanan & Akses

- Akses ke endpoint usulan dibatasi per unit kerja dan peran
- Setiap perubahan usulan tercatat dalam histori audit trail
- Input divalidasi sisi klien & server

---

## 🔄 Integrasi Layanan

- **layanan-aset**: mengambil master data aset
- **layanan-standar**: mengambil standar jumlah/spesifikasi
- **layanan-rekomendasi**: untuk perhitungan prioritas
- **authenc**: validasi token & otorisasi pengguna

---

## ✅ API Utama

| Method | Endpoint                  | Deskripsi                            |
|--------|---------------------------|--------------------------------------|
| GET    | `/usulan`                 | Ambil daftar usulan pengguna         |
| POST   | `/usulan`                 | Ajukan usulan baru                   |
| PUT    | `/usulan/{id}`            | Ubah usulan                          |
| DELETE | `/usulan/{id}`            | Hapus usulan                         |
| GET    | `/usulan/pratinjau/{id}`  | Dapatkan ringkasan & analisis usulan|

---

## 🧪 Pengujian

Jalankan semua pengujian unit dengan:

```bash
make test
```

---

## 📝 Catatan Tambahan

- Layanan ini akan terus diperbarui untuk mendukung analisis berbasis realisasi anggaran dan integrasi ke SIMAN.
- AI dilatih ulang setiap bulan menggunakan data hasil usulan yang disetujui.

---

**Layanan ini mendukung efisiensi dan transparansi dalam proses perencanaan kebutuhan BMN melalui otomasi dan analisis cerdas.**
