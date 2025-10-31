# Quick Start Guide - MonSAKTI Data Fetcher

## Setup Cepat (5 Menit)

### 1. Persiapan Database

```bash
# Buat database PostgreSQL
createdb monsakti

# Jalankan migrations
cd layanan/shared/integrasi
for file in migrations/*.sql; do
    echo "Running $file..."
    psql -d monsakti -f "$file"
done
```

### 2. Konfigurasi Token

```bash
# Copy template
cp .env.example .env

# Edit .env dan isi token
nano .env
```

Minimal configuration:
```env
DATABASE_URL=postgresql://localhost/monsakti
MONSAKTI_TOKEN_ADM=your_token_here
MONSAKTI_TOKEN_ANG=your_token_here
MONSAKTI_TOKEN_BEN=your_token_here
```

### 3. Test Satu Satker

```bash
# Test dengan satu satker dulu
TEST_SATKER=123456 cargo run --example fetch_to_database
```

### 4. Production Run

```bash
# Fetch semua (MySIMKARI + MonSAKTI)
cargo run --example fetch_to_database

# Hanya MySIMKARI
FETCH_MODE=mysimkari cargo run --example fetch_to_database

# Hanya MonSAKTI
FETCH_MODE=monsakti cargo run --example fetch_to_database
```

## Fitur Utama

### ✅ Auto Token Reset

Tidak perlu manual reset token lagi! Sistem otomatis:
1. Detect token expired
2. Call endpoint reset token
3. Retry request dengan token baru

### ✅ Direct Database Insert

Data langsung masuk database, tidak perlu:
- Simpan ke file JSON
- Import manual ke database
- Cleanup file temporary

### ✅ Error Recovery

Jika ada error:
- Sistem lanjut ke endpoint berikutnya
- Log detail error untuk debugging
- Tidak stop seluruh proses

## Endpoint Reset Token

Sistem menggunakan endpoint berikut untuk reset token:

```
ADM:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/ADM/tipedata/KL006
ANG:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/ANG/tipedata/KL006
AST:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/AST/tipedata/KL006
BEN:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/BEN/tipedata/KL006
GLP:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/GLP/tipedata/KL006
KOM:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/KOM/tipedata/KL006
PEM:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/PEM/tipedata/KL006
PER:  https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/PER/tipedata/KL006
```

## Monitoring

### Check Logs

```bash
# Lihat progress
tail -f /var/log/simpelv2-integrasi.log

# Atau jika run di terminal
cargo run --example fetch_to_database 2>&1 | tee fetch.log
```

### Check Database

```sql
-- Cek jumlah data per tabel
SELECT
    schemaname,
    tablename,
    n_live_tup as row_count
FROM pg_stat_user_tables
WHERE schemaname = 'public'
ORDER BY n_live_tup DESC;

-- Cek data terbaru
SELECT * FROM adm_pejabat ORDER BY created_at DESC LIMIT 10;
SELECT * FROM ang_data_ang ORDER BY created_at DESC LIMIT 10;
```

## Troubleshooting

### Token Expired Terus

```bash
# Manual reset token via curl
curl "https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/ADM/tipedata/KL006"

# Update token di .env
nano .env
```

### Database Connection Error

```bash
# Test koneksi
psql postgresql://localhost/monsakti -c "SELECT 1"

# Cek DATABASE_URL
echo $DATABASE_URL
```

### Rate Limiting

Edit `src/batch_db.rs` dan tambah delay:

```rust
// Delay untuk menghindari rate limiting
tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; // Dari 2 jadi 5 detik
```

## Advanced Usage

### Fetch Modul Tertentu Saja

```rust
use monsakti_fetcher::{Config, MonsaktiClient, batch_db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let mut client = MonsaktiClient::new(config).await?;

    // Hanya fetch ADM dan ANG
    batch_db::fetch_adm_to_db(&mut client, "006", "123456").await?;
    batch_db::fetch_ang_to_db(&mut client, "006", "123456").await?;

    Ok(())
}
```

### Custom Table Mapping

```rust
// Simpan dengan nama tabel custom
let data = adm::pejabat(&mut client, "006", "123456").await?;
client.save_to_postgres("my_custom_table", &data).await?;
```

### Parallel Processing

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();

for satker in satker_list {
    let mut client_clone = client.clone();
    set.spawn(async move {
        fetch_satker_complete_to_db(&mut client_clone, "006", &satker).await
    });
}

while let Some(result) = set.join_next().await {
    match result {
        Ok(Ok(_)) => println!("Satker completed"),
        Ok(Err(e)) => eprintln!("Satker failed: {}", e),
        Err(e) => eprintln!("Task failed: {}", e),
    }
}
```

## Scheduled Runs

### Cron Job

```bash
# Edit crontab
crontab -e

# Tambahkan (run setiap hari jam 2 pagi)
0 2 * * * cd /path/to/layanan/shared/integrasi && cargo run --release --example fetch_to_database >> /var/log/monsakti-cron.log 2>&1
```

### Systemd Service

```ini
# /etc/systemd/system/simpelv2-integrasi.service
[Unit]
Description=MonSAKTI Data Fetcher
After=network.target postgresql.service

[Service]
Type=oneshot
User=monsakti
WorkingDirectory=/path/to/layanan/shared/integrasi
ExecStart=/usr/bin/cargo run --release --example fetch_to_database
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

```bash
# Enable dan start
sudo systemctl enable simpelv2-integrasi.service
sudo systemctl start simpelv2-integrasi.service

# Check status
sudo systemctl status simpelv2-integrasi.service
```

## Performance Tips

1. **Use Release Build:**
   ```bash
   cargo build --release
   ./target/release/examples/fetch_to_database
   ```

2. **Increase Database Connection Pool:**
   Edit `src/client.rs`:
   ```rust
   .pool_max_idle_per_host(20) // Dari 10 jadi 20
   ```

3. **Batch Size:**
   Edit `src/db.rs`:
   ```rust
   for chunk in data.chunks(200) { // Dari 100 jadi 200
   ```

4. **Parallel Satker Processing:**
   Gunakan `tokio::task::JoinSet` untuk process multiple satker bersamaan

## Support

Untuk pertanyaan atau issue:
1. Check logs untuk error details
2. Verify token masih valid
3. Check database connection
4. Review IMPROVEMENTS.md untuk detail teknis
