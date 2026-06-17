# layanan-audit

**layanan-audit** (diimplementasikan sebagai modul `audit` dalam `layanan/perlengkapan`) bertanggung jawab untuk mencatat dan mengelola jejak audit (audit trail) serta kepatuhan pengelolaan BMN.

---

## 🎯 Tujuan

- Pencatatan otomatis setiap perubahan data kritikal (aset, usulan, hibah).
- Penyediaan dashboard kepatuhan untuk auditor.
- Deteksi anomali pada transaksi BMN.

---

## 📦 Lokasi Kode

`layanan/perlengkapan/src/audit/`

---

## 📝 Catatan Implementasi

Implementasi saat ini menggunakan modul Rust di dalam `layanan/perlengkapan` untuk memastikan pencatatan audit yang cepat dan terintegrasi erat dengan operasi bisnis utama.
