# Peningkatan Sistem Tarik API MonSAKTI

## Ringkasan Perubahan

Sistem tarik API MonSAKTI telah ditingkatkan dengan fitur-fitur berikut:

### 1. Auto-Retry dengan Token Reset ✅

**Masalah Sebelumnya:**
- Ketika token expired, request gagal dan harus manual reset token
- mekanisme retry otomatis

**Solusi:**
- Implementasi `fetch_with_retry()` yang otomatis detect token expired
- Auto-reset token menggunakan endpoint khusus MonSAKTI
- Default menggunakan KL006 untuk Kejaksaan RI
- Retry maksimal 1x setelah reset token

**Endpoint Reset Token yang Digunakan:**
```
https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/{MODULE}/tipedata/KL006
```

Modul yang didukung:
- ADM (Administrasi)
- ANG (Anggaran)
- AST (Aset)
- BEN (Bendahara)
- GLP (General Ledger)
- KOM (Komitmen)
- PEM (Pembayaran)
- PER (Persediaan)

**Kode:**
```rust
// Di client.rs
pub async fn fetch(&mut self, module: &str, tipe_data: &str, variables: Vec<String>) -> Result<MonsaktiResponse, MonsaktiError> {
    self.fetch_with_retry(module, tipe_data, variables, 1).await
}

async fn fetch_with_retry(&mut self, module: &str, tipe_data: &str, variables: Vec<String>, retry_count: u8) -> Result<MonsaktiResponse, MonsaktiError> {
    // ... fetch logic ...

    // Jika token expired, auto-reset dan retry
    if error_msg.contains("Token Expired") && retry_count == 1 {
        if let Ok(_) = self.reset_token_auto(module, tipe_data).await {
            return self.fetch_with_retry(module, tipe_data, variables, retry_count + 1).await;
        }
    }
}

async fn reset_token_auto(&mut self, module: &str, tipe_data: &str) -> Result<String, MonsaktiError> {
    // Default menggunakan KL006 untuk Kejaksaan RI
    self.reset_token(module, tipe_data, "KL006").await
}
```

### 2. Direct Database Storage ✅

**Masalah Sebelumnya:**
- Data hanya disimpan ke file JSON/CSV
- Perlu proses manual untuk import ke database
- Tidak efisien untuk data besar

**Solusi:**
- Implementasi `save_to_database()` untuk langsung simpan ke PostgreSQL
- Bulk insert dengan batch processing (100 rows per batch)
- Automatic column mapping (case-insensitive)
- ON CONFLICT DO NOTHING untuk idempotency
- Error handling per-row (tidak stop jika ada row yang gagal)

**Modul Baru:**
- `src/db.rs` - Helper functions untuk database operations
- `src/batch_db.rs` - Batch processing dengan database storage

**Kode:**
```rust
// Simpan data ke database
let data = adm::pejabat(&mut client, "006", "123456").await?;
client.save_to_database("adm", "pejabat", &data).await?;

// Atau dengan nama tabel eksplisit
client.save_to_postgres("adm_pejabat", &data).await?;
```

### 3. Optimized untuk KL006 ✅

**Konstanta:**
```rust
pub const KL_KEJAKSAAN: &str = "006";
```

**Fungsi Khusus:**
- `fetch_satker_complete_to_db()` - Fetch semua modul untuk satu satker
- `fetch_all_satker_to_db()` - Fetch semua satker untuk KL006
- Auto-delay 2 detik antar satker untuk menghindari rate limiting

### 4. Improved Error Handling ✅

**Fitur:**
- Detailed error messages dengan HTTP status code
- Warning untuk row yang gagal insert (tidak stop execution)
- Logging yang lebih informatif
- Graceful degradation (lanjut ke endpoint berikutnya jika ada yang gagal)

### 5. Database Schema ✅

Sudah tersedia migrations untuk semua tabel:
- `001_create_adm_tables.sql` - Tabel ADM (ref_admin, pejabat, ref_jns_spp)
- `002_create_ang_tables.sql` - Tabel ANG (data_ang, ref_sts)
- `003_create_pem_tables.sql` - Tabel PEM (realisasi, spp_header, dll)
- `004_create_ben_tables.sql` - Tabel BEN (kas_tunai, kas_bank, dll)
- `005_create_kom_tables.sql` - Tabel KOM (kontrak, supplier, dll)
- `006_create_ast_per_tables.sql` - Tabel AST dan PER
- `007_create_glp_tables.sql` - Tabel GLP (buku_besar, neraca, dll)
- `008_create_triggers.sql` - Triggers untuk updated_at
- `010_create_mysimkari_tables.sql` - Tabel MySIMKARI

## Cara Penggunaan

### Setup

1. **Install Dependencies:**
```bash
cd layanan/shared/integrasi
cargo build
```

2. **Setup Database:**
```bash
# Buat database
createdb monsakti

# Jalankan migrations
psql -d monsakti -f migrations/001_create_adm_tables.sql
psql -d monsakti -f migrations/002_create_ang_tables.sql
# ... dst untuk semua migrations
```

3. **Konfigurasi Environment:**
```bash
cp .env.example .env
```

Edit `.env`:
```env
# Database
DATABASE_URL=postgresql://user:password@localhost/monsakti

# MonSAKTI API
MONSAKTI_BASE_URL=https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice

# Tokens untuk setiap modul
MONSAKTI_TOKEN_ADM=your_token_here
MONSAKTI_TOKEN_ANG=your_token_here
MONSAKTI_TOKEN_BEN=your_token_here
MONSAKTI_TOKEN_PEM=your_token_here
MONSAKTI_TOKEN_KOM=your_token_here
MONSAKTI_TOKEN_AST=your_token_here
MONSAKTI_TOKEN_PER=your_token_here
MONSAKTI_TOKEN_GLP=your_token_here

# MySIMKARI
MYSIMKARI_BASE_URL=https://mysimkari.kejaksaan.go.id/api/anbut
MYSIMKARI_TOKEN=your_token_here
```

### Penggunaan

**1. Fetch Satu Satker (Testing):**
```bash
TEST_SATKER=123456 cargo run --example fetch_to_database
```

**2. Fetch Semua Satker (Production):**
```bash
cargo run --example fetch_to_database
```

**3. Programmatic Usage:**
```rust
use monsakti_fetcher::{Config, MonsaktiClient, fetch_satker_complete_to_db, KL_KEJAKSAAN};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let mut client = MonsaktiClient::new(config).await?;

    // Fetch satu satker
    fetch_satker_complete_to_db(&mut client, KL_KEJAKSAAN, "123456").await?;

    Ok(())
}
```

## Performa

**Sebelum:**
- Manual token reset ketika expired
- Simpan ke file, lalu import manual ke database
- Tidak ada retry mechanism

**Sesudah:**
- Auto token reset dan retry
- Direct database insert (no intermediate files)
- Batch processing 100 rows per batch
- Parallel processing untuk multiple endpoints (opsional)
- Rate limiting protection (2s delay antar satker)

## Monitoring

**Logs:**
```
[INFO] Fetching: https://monsakti.kemenkeu.go.id/.../API/ADM/pejabat/KL006/123456 (attempt 1)
[WARN] Token kadaluarsa untuk modul ADM (attempt 1)
[INFO] Mencoba reset token dan retry...
[INFO] Token berhasil direset untuk modul ADM
[INFO] Fetching: https://monsakti.kemenkeu.go.id/.../API/ADM/pejabat/KL006/123456 (attempt 2)
[INFO] 15 baris berhasil diinsert ke tabel adm_pejabat
```

## Troubleshooting

**1. Token Expired:**
- Sistem akan otomatis reset token dan retry
- Jika masih gagal, cek token di .env

**2. Database Connection Error:**
- Pastikan PostgreSQL running
- Cek DATABASE_URL di .env
- Pastikan database sudah dibuat

**3. Rate Limiting:**
- Sistem sudah ada delay 2s antar satker
- Jika masih kena rate limit, tambah delay di `batch_db.rs`

**4. Data Tidak Masuk Database:**
- Cek logs untuk error messages
- Pastikan migrations sudah dijalankan
- Cek struktur tabel sesuai dengan response API

## MySIMKARI Integration

**Fitur:**
- `fetch_mysimkari_to_db()` - Fetch data satker dan pegawai dari MySIMKARI
- `fetch_all_data_to_db()` - Fetch MySIMKARI + MonSAKTI sekaligus
- Auto-fetch pegawai untuk setiap satker
- Rate limiting protection (500ms delay antar satker)

**Tabel Database:**
- `mysimkari_satker` - Data satker dari MySIMKARI
- `mysimkari_pegawai` - Data pegawai dari MySIMKARI

**Usage:**
```rust
// Hanya MySIMKARI
fetch_mysimkari_to_db(&mut client).await?;

// MySIMKARI + MonSAKTI
fetch_all_data_to_db(&mut client, KL_KEJAKSAAN).await?;
```

**Environment Variable:**
```env
MYSIMKARI_BASE_URL=https://mysimkari.kejaksaan.go.id/api/anbut
MYSIMKARI_TOKEN=your_token_here
```

## Next Steps

1. ✅ Auto-retry dengan token reset - **DONE**
2. ✅ Direct database storage - **DONE**
3. ✅ Optimized untuk KL006 - **DONE**
4. ✅ Add MySIMKARI integration ke batch_db - **DONE**
5. 🔄 Add monitoring dashboard
6. 🔄 Add data validation before insert
7. 🔄 Add incremental sync (hanya fetch data baru)

## Kontributor

- Sistem ini dioptimalkan untuk Kejaksaan RI (KL006)
- Auto-retry dan database storage ditambahkan untuk efisiensi operasional

