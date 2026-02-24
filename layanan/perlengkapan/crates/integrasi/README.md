# SIMPEL - Layanan Integrasi

Layanan integrasi untuk external APIs dalam ekosistem SIMPEL. Menangani integrasi dengan:

- **MonSAKTI** - Sistem Aplikasi Keuangan Tingkat Instansi (Kementerian Keuangan RI)
- **MySIMKARI** - Sistem Informasi Kepegawaian Kejaksaan RI
- **SIMAN v2.0** - Sistem Informasi Manajemen Aset Negara (Kementerian Keuangan RI)

## 🆕 What's New - Peningkatan Terbaru

### ✨ Auto Token Reset & Retry

- **Otomatis detect token expired** dan reset menggunakan endpoint khusus
- **Auto-retry** setelah token direset
- **Optimized untuk KL006** (Kejaksaan RI)
- Tidak perlu manual reset token lagi!

### 💾 Direct Database Storage

- **Langsung simpan ke PostgreSQL** tanpa file intermediary
- **Batch insert** dengan performa tinggi (100 rows per batch)
- **Error recovery** - lanjut ke endpoint berikutnya jika ada yang gagal
- Tidak perlu import manual lagi!

### 📚 Dokumentasi Lengkap

- **[QUICK_START.md](QUICK_START.md)** - Setup dalam 5 menit
- **[IMPROVEMENTS.md](IMPROVEMENTS.md)** - Detail teknis peningkatan
- **[dokumentasi_monsakti.md](dokumentasi_monsakti.md)** - Referensi API lengkap

**Quick Start:**

```bash
# Setup database dan token
cp .env.example .env
nano .env

# Test dengan satu satker
TEST_SATKER=123456 cargo run --example fetch_to_database

# Production: fetch semua satker KL006
cargo run --example fetch_to_database
```

## 📋 Deskripsi

Library ini menyediakan interface yang mudah digunakan untuk berinteraksi dengan multiple government APIs:

### MonSAKTI v1.4

Mendukung semua modul:

- **ADM** - Administrasi
- **ANG** - Penganggaran
- **PEM** - Pembayaran
- **BEN** - Bendahara
- **KOM** - Komitmen
- **AST** - Aset Tetap
- **PER** - Persediaan
- **GLP** - Pelaporan

### MySIMKARI

- Data satker
- Data pegawai per satker

### SIMAN v2.0 (NEW!)

15 kategori aset Barang Milik Negara (BMN):

- Tanah, Gedung & Bangunan, Rumah
- Angkutan Bermotor, Alat Besar, Alat Persenjataan
- Peralatan TIK & Non-TIK
- Instalasi & Jaringan, Jalan & Jembatan
- Dan lainnya...

## ✨ Fitur

### MonSAKTI

- ✅ Implementasi lengkap 43 endpoint API MonSAKTI
- ✅ **Auto token reset & retry** ketika token expired
- ✅ **Direct database storage** tanpa file intermediary
- ✅ **Optimized untuk KL006** (Kejaksaan RI)

### SIMAN v2.0 (NEW!)

- ✅ **OAuth2 authentication** dengan SSO Kemenkeu
- ✅ **Auto token refresh** sebelum expired
- ✅ **15 kategori aset BMN** lengkap
- ✅ **Auto pagination** untuk dataset besar
- ✅ Convenience functions untuk setiap kategori

### MySIMKARI

- ✅ Integrasi data kepegawaian
- ✅ Sinkronisasi satker dan pegawai

### General Features

- ✅ Token management otomatis
- ✅ Parallel & sequential processing untuk batch operations
- ✅ Export ke JSON, CSV, dan PostgreSQL
- ✅ Type-safe dengan Rust
- ✅ Async/await dengan Tokio
- ✅ Logging dengan tracing
- ✅ Error handling yang robust

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 atau lebih baru
- PostgreSQL (opsional, untuk database storage)

### Instalasi

```bash
git clone <repository-url>
cd api_monsakti
cargo build --release
```

### Konfigurasi

1. Copy file `.env.example` ke `.env`:

```bash
cp .env.example .env
```

2. Edit `.env` dengan kredensial Anda:

```env
MONSAKTI_BASE_URL=https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice

# Token untuk setiap modul
MONSAKTI_TOKEN_ADM=your_token_here
MONSAKTI_TOKEN_ANG=your_token_here
MONSAKTI_TOKEN_PEM=your_token_here
MONSAKTI_TOKEN_BEN=your_token_here
MONSAKTI_TOKEN_KOM=your_token_here
MONSAKTI_TOKEN_AST=your_token_here
MONSAKTI_TOKEN_PER=your_token_here
MONSAKTI_TOKEN_GLP=your_token_here

# Output directory
OUTPUT_DIR=./data

# Database (optional)
DATABASE_URL=postgresql://user:password@localhost/monsakti

# Processing mode
USE_PARALLEL=true
```

### Penggunaan Dasar

```rust
use monsakti_fetcher::{Config, MonsaktiClient};
use monsakti_fetcher::modul::adm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load konfigurasi
    let config = Config::from_env()?;
    let mut client = MonsaktiClient::new(config).await?;

    // Ambil data referensi admin
    let data = adm::ref_admin(&mut client, "006", "").await?;

    // Simpan ke JSON
    client.save_to_json(&data, "output/ref_admin.json").await?;

    Ok(())
}
```

## 📚 Dokumentasi API

### 🆕 SIMAN API v2.0 - Barang Milik Negara

```rust
use layanan_integrasi::{Config, MonsaktiClient};
use layanan_integrasi::siman::{SimanAssetCategory, get_row_count, get_aset_by_category};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let mut client = MonsaktiClient::new(config).await?;

    // Get total count
    let count = get_row_count(&mut client, SimanAssetCategory::Tanah).await?;
    println!("Total Tanah: {}", count);

    // Fetch data with pagination
    let data = get_aset_by_category(
        &mut client,
        SimanAssetCategory::Tanah,
        1,    // start ID
        100   // end ID
    ).await?;

    println!("Retrieved {} records", data.len());
    Ok(())
}
```

**Available Categories:**

- `AlatBesar`, `AngkutanBermotor`, `AlatPersenjataan`
- `TakBerwujud`, `BangunanAir`, `GedungBangunan`
- `InstalasiJaringan`, `JalandanJembatan`, `NonTIK`
- `Rumah`, `Tanah`, `TetapLainnya`
- `KDP`, `KhususTIK`, `TetapRenovasi`

**Convenience Functions:**

```rust
use layanan_integrasi::siman::{
    get_aset_tanah,
    get_aset_angkutan_bermotor,
    get_aset_gedung_bangunan,
    fetch_all_aset_paginated,  // Auto pagination!
};

// Direct category access
let tanah = get_aset_tanah(&mut client, 1, 100).await?;

// Fetch all with auto pagination
let all_gedung = fetch_all_aset_paginated(
    &mut client,
    SimanAssetCategory::GedungBangunan,
    1000  // chunk size
).await?;
```

📖 **Full documentation:** [dokumentasi_siman.md](dokumentasi_siman.md)
💡 **Example code:** [examples/siman_example.rs](examples/siman_example.rs)

### Modul Administrasi (ADM)

```rust
use monsakti_fetcher::modul::adm;

// Referensi uraian (program, kegiatan, output, suboutput, komponen, akun)
let data = adm::ref_uraian(&mut client, "006", "program", "").await?;

// Referensi satker
let data = adm::ref_admin(&mut client, "006", "").await?;

// Data pejabat
let data = adm::pejabat(&mut client, "006", "123456").await?;

// Referensi bank
let data = adm::ref_bank(&mut client, "006").await?;

// Referensi jenis SPP
let data = adm::ref_jns_spp(&mut client, "006", "").await?;

// Referensi aset
let data = adm::ref_aset(&mut client, "006", "KDGOL", "").await?;
```

### Modul Penganggaran (ANG)

```rust
use monsakti_fetcher::modul::ang;

// Data anggaran
let data = ang::data_ang(&mut client, "006", "123456", "B00").await?;

// Referensi status history
let data = ang::ref_sts(&mut client, "006", "123456").await?;

// Data pendapatan
let data = ang::pendapatan(&mut client, "006", "123456", "B00").await?;
```

### Modul Pembayaran (PEM)

```rust
use monsakti_fetcher::modul::pem;

// Realisasi
let data = pem::realisasi(&mut client, "006", "123456", "231", "00001T").await?;

// SPP Header
let data = pem::spp_header(&mut client, "006", "123456", "", "").await?;

// SPP Pengeluaran (by ID_SPP)
let data = pem::spp_pengeluaran(&mut client, "006", "12345").await?;

// SPP Potongan
let data = pem::spp_potongan(&mut client, "006", "12345").await?;

// Penerima SPM
let data = pem::penerima_spm(&mut client, "006", "12345").await?;
```

### Modul Bendahara (BEN)

```rust
use monsakti_fetcher::modul::ben;

// Kas Tunai
let data = ben::kas_tunai(&mut client, "006", "123456").await?;

// Kas Bank
let data = ben::kas_bank(&mut client, "006", "123456").await?;

// SPBY
let data = ben::spby(&mut client, "006", "123456").await?;

// Kuitansi
let data = ben::kuitansi(&mut client, "006", "123456").await?;

// DRPP
let data = ben::drpp(&mut client, "006", "123456").await?;

// Pungut Pajak
let data = ben::pungut_pajak(&mut client, "006", "123456").await?;

// Setor Pajak
let data = ben::setor_pajak(&mut client, "006", "123456").await?;

// PNBP
let data = ben::pnbp(&mut client, "006", "123456").await?;

// TUP
let data = ben::tup(&mut client, "006", "123456").await?;

// Pengembalian
let data = ben::pengembalian(&mut client, "006", "123456").await?;
```

### Modul Komitmen (KOM)

```rust
use monsakti_fetcher::modul::kom;

// Capaian RO
let data = kom::capaian_ro(&mut client, "006", "123456", "2025-01").await?;

// Kontrak Header
let data = kom::kontrak_header(&mut client, "006", "123456").await?;

// Kontrak Line
let data = kom::kontrak_line(&mut client, "006", "123456", "1", "1").await?;

// Kontrak Termin
let data = kom::kontrak_termin(&mut client, "006", "123456", "1", "1", "1").await?;

// Kontrak COA
let data = kom::kontrak_coa(&mut client, "006", "123456", "1", "1", "1").await?;

// BAST Kontrak
let data = kom::bast_kontrak_header(&mut client, "006", "123456", "").await?;
let data = kom::bast_kontrak_detail_barang(&mut client, "006", "123456", "1").await?;
let data = kom::bast_kontrak_coa(&mut client, "006", "123456", "1").await?;

// BAST Non Kontrak
let data = kom::bast_non_kontrak_header(&mut client, "006", "123456", "").await?;
let data = kom::bast_non_kontrak_detail_barang(&mut client, "006", "123456", "1").await?;
let data = kom::bast_non_kontrak_coa(&mut client, "006", "123456", "1").await?;

// Supplier
let data = kom::supplier_header(&mut client, "006").await?;
let data = kom::supplier_address(&mut client, "006", "1").await?;
let data = kom::supplier_bank(&mut client, "006", "1", "1", "").await?;
```

### Modul Aset (AST)

```rust
use monsakti_fetcher::modul::ast;

// Transaksi Aset
let data = ast::aset_trx(
    &mut client,
    "006",      // kode_kl
    "123456",   // kdsatker
    "1",        // kdgol
    "101",      // kdbid
    "",         // kdkel (opsional)
    "",         // kdskel (opsional)
    ""          // kdbrg (opsional)
).await?;
```

### Modul Persediaan (PER)

```rust
use monsakti_fetcher::modul::per;

// Transaksi Persediaan
let data = per::persedia_trx(
    &mut client,
    "006",      // kode_kl
    "123456",   // kdsatker
    "1",        // kdgol (opsional)
    "101",      // kdbid (opsional)
    "",         // kdkel (opsional)
    "",         // kdskel (opsional)
    ""          // kdbrg (opsional)
).await?;
```

### Modul Pelaporan (GLP)

```rust
use monsakti_fetcher::modul::glp;

// Buku Besar
let data = glp::buku_besar(&mut client, "006", "123456", "01", "").await?;

// Neraca Saldo Awal
let data = glp::neraca_sawal(&mut client, "006", "123456").await?;

// FA Detail
let data = glp::fa_detail(&mut client, "006", "123456", "01").await?;
```

## 🔄 Batch Processing

### Sequential Mode (Lebih Aman)

```rust
use monsakti_fetcher::fetch_satker_complete_sequential;

fetch_satker_complete_sequential(&mut client, "006", "123456").await?;
```

### Parallel Mode (Lebih Cepat)

```rust
use monsakti_fetcher::fetch_satker_complete_parallel;

fetch_satker_complete_parallel(&mut client, "006", "123456").await?;
```

### Batch All Satker

```rust
use monsakti_fetcher::batch_fetch_all_satker;

let satker_list = batch_fetch_all_satker(&mut client, "006").await?;

for kdsatker in satker_list {
    fetch_satker_complete_parallel(&mut client, "006", &kdsatker).await?;
}
```

## 💾 Export Data

### JSON

```rust
client.save_to_json(&data, "output/data.json").await?;
```

### CSV

```rust
client.save_to_csv(&data, "output/data.csv").await?;
```

### PostgreSQL

```rust
client.save_to_postgres("table_name", &data).await?;
```

## 🗄️ Database Setup

Jalankan migration untuk membuat tabel database:

```bash
# Jalankan semua migration
cat migrations/00*.sql | psql -U username -d database_name

# Atau satu per satu
psql -U username -d database_name -f migrations/001_create_adm_tables.sql
psql -U username -d database_name -f migrations/002_create_ang_tables.sql
# ... dst
```

Lihat [migrations/README.md](migrations/README.md) untuk detail lengkap.

## 🧪 Testing

### Test Modul AST

```bash
cargo run --example test_ast
```

### Run Main Program

```bash
cargo run --release
```

## 📖 Struktur Proyek

```
api_monsakti/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── client.rs            # HTTP client & token management
│   ├── config.rs            # Configuration
│   ├── error.rs             # Error types
│   ├── response.rs          # Response types
│   ├── batch.rs             # Batch processing
│   └── modul/               # API modules
│       ├── mod.rs
│       ├── adm.rs           # Administrasi
│       ├── ang.rs           # Penganggaran
│       ├── pem.rs           # Pembayaran
│       ├── ben.rs           # Bendahara
│       ├── kom.rs           # Komitmen
│       ├── ast.rs           # Aset
│       ├── per.rs           # Persediaan
│       └── glp.rs           # Pelaporan
├── migrations/              # Database migrations
├── examples/                # Example code
├── .env                     # Configuration file
└── Cargo.toml              # Dependencies
```

## 🔐 Token Management

Token MonSAKTI hanya dapat digunakan **1x hit endpoint**. Token baru akan ada di response data. Library ini menangani token management secara otomatis:

1. Token baru dari response disimpan otomatis
2. Jika token expired, gunakan `reset_token()`:

```rust
client.reset_token("AST", "asetTrx", "KL006").await?;
```

## ⚠️ Catatan Penting

1. **Format Kode KL**: Gunakan 3 digit tanpa prefix "KL" (contoh: "006" bukan "KL006")
2. **KDSATKER**: 6 digit kode satker (contoh: "123456")
3. **Token Per Modul**: Setiap modul memiliki token sendiri
4. **Token Per Tahun**: Token berbeda untuk setiap tahun anggaran
5. **Rate Limiting**: Hindari request terlalu cepat, gunakan delay jika perlu
6. **Data Besar**: Untuk aset/persediaan, gunakan filter KDKEL/KDSKEL untuk menghindari timeout

## 🐛 Troubleshooting

### Token Expired

```rust
// Reset token jika expired
match client.reset_token("AST", "asetTrx", "KL006").await {
    Ok(new_token) => println!("Token baru: {}", new_token),
    Err(e) => eprintln!("Gagal reset token: {}", e),
}
```

### Data Terlalu Besar

Untuk endpoint dengan data besar (asetTrx, persediaTrx), gunakan filter:

```rust
// Gunakan filter sampai KDKEL atau KDSKEL
let data = ast::aset_trx(&mut client, "006", "123456", "1", "101", "10101", "", "").await?;
```

### Connection Timeout

Tingkatkan timeout di `client.rs`:

```rust
let client = Client::builder()
    .timeout(std::time::Duration::from_secs(300)) // 5 menit
    .build()?;
```

## 📝 Logging

Set log level dengan environment variable:

```bash
# Debug level
RUST_LOG=debug cargo run

# Info level (default)
RUST_LOG=info cargo run

# Error only
RUST_LOG=error cargo run
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License.

## 📞 Support

Untuk pertanyaan terkait API MonSAKTI, silakan hubungi:

- Email: sitp.perbendaharaan@kemenkeu.go.id
- Website: hai.kemenkeu.go.id

## 🔗 Referensi

- [Dokumentasi API MonSAKTI v1.4](dokumentasi_monsakti.md)
- [Struktur Data Lengkap](STRUKTUR_DATA_MONSAKTI.md)
- [Database Migrations](migrations/README.md)

---

**Version**: 1.0.0
**Last Updated**: 2025
**API Version**: MonSAKTI v1.4 (5 Mei 2023)
