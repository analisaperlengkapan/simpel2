# layanan-aset (bank_aset)

**layanan-aset** (diimplementasikan sebagai modul `bank_aset` dalam `layanan/perlengkapan`) adalah modul utama yang mengelola master data aset Barang Milik Negara (BMN) dalam sistem SIMPEL. Layanan ini menjadi fondasi utama integritas data aset nasional, mulai dari identitas aset, status kepemilikan, klasifikasi, hingga histori mutasi.

---

## 🎯 Tujuan

- Menyediakan API CRUD untuk entitas aset BMN
- Menjamin integritas dan konsistensi data aset
- Menyediakan fitur validasi klasifikasi dan kondisi aset
- Mendukung klasifikasi otomatis aset dengan AI (terintegrasi)

---

## 🧱 Teknologi

- **Bahasa**: Rust (menggantikan rencana awal Go)
- **Framework**: Axum
- **Database**: PostgreSQL (schema: `aset`)
- **ORM**: tokio-postgres / deadpool-postgres
- **Testing**: Rust native test framework

---

## 📦 Struktur Direktori (Implementasi Saat Ini)

Modul ini berada di `layanan/perlengkapan/src/bank_aset/`.

```
layanan/perlengkapan/src/bank_aset/
├── mod.rs          # Handler HTTP dan routing
├── repository/     # Logika akses database
├── service/        # Logika bisnis & validasi
└── types/          # Definisi struct dan entitas
```

---

## 🔐 Keamanan

- Middleware JWT untuk otentikasi (via authenc)
- Validasi input menggunakan `garde`
- Role-based Access Control (RBAC)

---

## 📝 Catatan Implementasi

Dokumen ini telah diperbarui untuk mencerminkan transisi dari rencana microservice berbasis Go ke modul terintegrasi dalam monolit `layanan/perlengkapan` berbasis Rust untuk efisiensi resource dan konsistensi codebase.
