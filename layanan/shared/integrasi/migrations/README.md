# Database Migrations - MonSAKTI & MySIMKARI API

Struktur database sesuai dengan dokumentasi API MonSAKTI v1.4 (Update: 5 Mei 2023) dan MySIMKARI API

## File Migration

1. **001_create_adm_tables.sql** - Modul Administrasi (ADM)
2. **002_create_ang_tables.sql** - Modul Penganggaran (ANG)
3. **003_create_pem_tables.sql** - Modul Pembayaran (PEM)
4. **004_create_ben_tables.sql** - Modul Bendahara (BEN)
5. **005_create_kom_tables.sql** - Modul Komitmen (KOM)
6. **006_create_ast_per_tables.sql** - Modul Aset (AST) & Persediaan (PER)
7. **007_create_glp_tables.sql** - Modul Pelaporan (GLP)
8. **008_create_triggers.sql** - Triggers untuk auto-update timestamp
9. **009_rollback.sql** - Script untuk rollback semua perubahan
10. **010_create_mysimkari_tables.sql** - Modul MySIMKARI (Satker & Pegawai)

## Cara Menjalankan Migration

### Menggunakan psql (PostgreSQL)

```bash
# Jalankan semua migration secara berurutan
psql -U username -d database_name -f migrations/001_create_adm_tables.sql
psql -U username -d database_name -f migrations/002_create_ang_tables.sql
psql -U username -d database_name -f migrations/003_create_pem_tables.sql
psql -U username -d database_name -f migrations/004_create_ben_tables.sql
psql -U username -d database_name -f migrations/005_create_kom_tables.sql
psql -U username -d database_name -f migrations/006_create_ast_per_tables.sql
psql -U username -d database_name -f migrations/007_create_glp_tables.sql
psql -U username -d database_name -f migrations/008_create_triggers.sql
psql -U username -d database_name -f migrations/010_create_mysimkari_tables.sql
```

### Atau jalankan semua sekaligus:

```bash
cat migrations/00*.sql | psql -U username -d database_name
```

### Rollback (Hapus semua tabel):

```bash
psql -U username -d database_name -f migrations/009_rollback.sql
```

## Struktur Tabel Per Modul

### ADM (Administrasi)
- `adm_ref_admin` - Referensi data satker
- `adm_pejabat` - Data pejabat satker
- `adm_ref_jns_spp` - Referensi jenis SPP

### ANG (Penganggaran)
- `ang_data_ang` - Data transaksi penganggaran (47 kolom)
- `ang_ref_sts` - Referensi status history

### PEM (Pembayaran)
- `pem_realisasi` - Realisasi 16 segmen COA
- `pem_spp_header` - Data SPP, SPM, SP2D (Primary Key: id_spp)
- `pem_spp_pengeluaran` - Distribusi COA pengeluaran
- `pem_spp_potongan` - Distribusi COA potongan
- `pem_penerima_spm` - Data penerima/supplier

### BEN (Bendahara)
- `ben_kas_tunai` - Transaksi kas tunai
- `ben_kas_bank` - Transaksi kas bank
- `ben_spby` - Surat Perintah Bayar
- `ben_kuitansi` - Kuitansi pembayaran
- `ben_drpp` - Daftar Rincian Permintaan Pembayaran
- `ben_pungut_pajak` - Pungutan pajak
- `ben_setor_pajak` - Setoran pajak
- `ben_pnbp` - PNBP

### KOM (Komitmen)
- `kom_capaian_ro` - Capaian realisasi output
- `kom_kontrak_header` - Data kontrak (Primary Key: id_kontrak)
- `kom_kontrak_line` - Data kontrak per line
- `kom_kontrak_termin` - Data kontrak per termin
- `kom_kontrak_coa` - Distribusi COA kontrak
- `kom_bast_kontrak_header` - BAST kontraktual
- `kom_bast_kontrak_detail_barang` - Detail barang BAST kontraktual
- `kom_bast_kontrak_coa` - COA BAST kontraktual
- `kom_bast_non_kontrak_header` - BAST non kontraktual
- `kom_bast_non_kontrak_detail_barang` - Detail barang BAST non kontraktual
- `kom_bast_non_kontrak_coa` - COA BAST non kontraktual
- `kom_supplier_header` - Data supplier (Primary Key: id_supplier)
- `kom_supplier_address` - Alamat supplier
- `kom_supplier_bank` - Rekening bank supplier

### AST (Aset)
- `ast_aset_trx` - Transaksi aset (35 kolom)

### PER (Persediaan)
- `per_persedia_trx` - Transaksi persediaan

### GLP (Pelaporan)
- `glp_buku_besar` - Jurnal transaksi (60+ kolom)
- `glp_neraca_sawal` - Saldo awal neraca
- `glp_fa_detail` - Detail transaksi FA akrual

### MySIMKARI (Data Kepegawaian)
- `mysimkari_satker` - Data satker kejaksaan (id, nama, alamat, koordinat, dll)
- `mysimkari_pegawai` - Data pegawai per satker (nama, nip, jabatan, golongan, dll)

## Relasi Antar Tabel

### Foreign Keys yang Penting:

1. **pem_spp_pengeluaran** → pem_spp_header (id_spp)
2. **pem_spp_potongan** → pem_spp_header (id_spp)
3. **pem_penerima_spm** → pem_spp_header (id_spp)
4. **kom_kontrak_line** → kom_kontrak_header (id_kontrak)
5. **kom_bast_kontrak_detail_barang** → kom_bast_kontrak_header (id_bast)
6. **kom_bast_kontrak_coa** → kom_bast_kontrak_header (id_bast)
7. **kom_bast_non_kontrak_detail_barang** → kom_bast_non_kontrak_header (id_bast)
8. **kom_bast_non_kontrak_coa** → kom_bast_non_kontrak_header (id_bast)
9. **kom_supplier_address** → kom_supplier_header (id_supplier)

### Join Logis (tanpa FK):

- **ang_data_ang.cons_item** = **pem_realisasi.kode_item**
- **ben_kuitansi.no_drpp_id** = **ben_drpp.api_id**
- **ben_drpp.id_spp** = **pem_spp_header.id_spp**

## Indexes

Semua tabel memiliki index pada kolom:
- `kdsatker` - untuk filter per satker
- Primary keys dan foreign keys
- Kolom yang sering digunakan untuk join

## Triggers

Semua tabel memiliki trigger `update_updated_at_column()` yang otomatis mengupdate kolom `updated_at` setiap kali ada UPDATE.

## Catatan Penting

1. **Tipe Data NUMERIC** digunakan untuk ID yang besar (dari API menggunakan NUMBER(19,0))
2. **VARCHAR(4000)** untuk kolom yang bisa berisi text panjang
3. **UNIQUE constraints** pada kombinasi kolom yang harus unik
4. **Kolom api_id** untuk menyimpan ID asli dari API (jika ada)
5. **Timestamps** otomatis: `created_at` dan `updated_at`

## Verifikasi

Setelah menjalankan migration, verifikasi dengan:

```sql
-- Cek jumlah tabel
SELECT COUNT(*) FROM information_schema.tables 
WHERE table_schema = 'public' AND table_type = 'BASE TABLE';

-- Cek semua tabel
SELECT table_name FROM information_schema.tables 
WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
ORDER BY table_name;

-- Cek semua triggers
SELECT trigger_name, event_object_table 
FROM information_schema.triggers 
WHERE trigger_schema = 'public'
ORDER BY event_object_table;
```

## Maintenance

### Backup Database:
```bash
pg_dump -U username database_name > backup_$(date +%Y%m%d).sql
```

### Restore Database:
```bash
psql -U username database_name < backup_20250101.sql
```

---

**Dokumentasi Referensi**: 
- API MonSAKTI v1.4 (5 Mei 2023)
- API MySIMKARI (Oktober 2025)
