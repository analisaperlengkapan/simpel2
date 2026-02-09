# Layanan Integrasi - Scheduler

Sistem penjadwalan otomatis untuk penarikan data rutin dari API eksternal (MonSAKTI, MySIMKARI, dan SIMAN).

## 📋 Fitur Utama

- **Penjadwalan Fleksibel**: Menggunakan cron expression untuk konfigurasi jadwal yang fleksibel
- **Multi-Source**: Mendukung MonSAKTI, MySIMKARI, dan SIMAN secara bersamaan
- **Konfigurasi via Environment**: Semua pengaturan dapat dikonfigurasi melalui file `.env`
- **Auto-Retry**: Otomatis mencoba ulang jika terjadi kegagalan
- **Timezone Support**: Mendukung pengaturan timezone (default: Asia/Jakarta)
- **Logging**: Logging lengkap untuk monitoring dan debugging

## 🚀 Quick Start

### 1. Konfigurasi Environment

Edit file `.env`:

```bash
# Enable scheduler
SCHEDULER_ENABLED=true

# MonSAKTI - Jadwal penarikan data (setiap hari jam 02:00 WIB)
MONSAKTI_SCHEDULE=0 2 * * *

# MySIMKARI - Jadwal penarikan data (setiap hari jam 02:00 WIB)
MYSIMKARI_SCHEDULE=0 2 * * *

# SIMAN - Jadwal penarikan data (setiap hari Minggu jam 03:00 WIB)
SIMAN_SCHEDULE=0 3 * * 0

# Timezone
SCHEDULER_TIMEZONE=Asia/Jakarta

# Storage configuration
STORAGE_TYPE=json
OUTPUT_DIR=./output
```

### 2. Jalankan Scheduler

```bash
# Menjalankan scheduler sebagai daemon
cargo run --bin scheduler

# Atau build dan run binary
cargo build --bin scheduler --release
./target/release/scheduler
```

## ⏰ Cron Expression Format

Scheduler menggunakan format cron expression standar dengan 5 field:

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6) (Sunday to Saturday)
│ │ │ │ │
│ │ │ │ │
* * * * *
```

### Contoh Cron Expression

| Expression | Deskripsi |
|------------|-----------|
| `0 2 * * *` | Setiap hari jam 02:00 |
| `0 3 * * 0` | Setiap hari Minggu jam 03:00 |
| `0 0 */6 * *` | Setiap 6 jam |
| `0 0 12 * * MON-FRI` | Setiap hari kerja jam 12:00 |
| `30 14 * * *` | Setiap hari jam 14:30 |
| `0 0 1 * *` | Setiap tanggal 1 jam 00:00 |
| `0 22 * * 1-5` | Setiap hari kerja jam 22:00 |

## 🛠️ Konfigurasi Detail

### Environment Variables

| Variable | Default | Deskripsi |
|----------|---------|-----------|
| `SCHEDULER_ENABLED` | `true` | Enable/disable scheduler |
| `MONSAKTI_SCHEDULE` | `0 2 * * *` | Jadwal penarikan MonSAKTI |
| `MYSIMKARI_SCHEDULE` | `0 2 * * *` | Jadwal penarikan MySIMKARI |
| `SIMAN_SCHEDULE` | `0 3 * * 0` | Jadwal penarikan SIMAN |
| `SCHEDULER_TIMEZONE` | `Asia/Jakarta` | Timezone untuk scheduler |
| `STORAGE_TYPE` | `json` | Tipe penyimpanan: `database`, `json`, `csv` |
| `OUTPUT_DIR` | `./output` | Direktori output untuk file export |

### Skenario Konfigurasi

#### Skenario 1: Penarikan Harian (Jam Kerja)
```bash
MONSAKTI_SCHEDULE=0 8 * * *      # Jam 08:00 setiap hari
MYSIMKARI_SCHEDULE=0 9 * * *     # Jam 09:00 setiap hari
SIMAN_SCHEDULE=0 10 * * 1        # Jam 10:00 setiap Senin
```

#### Skenario 2: Penarikan Malam (Load Rendah)
```bash
MONSAKTI_SCHEDULE=0 2 * * *      # Jam 02:00 setiap hari
MYSIMKARI_SCHEDULE=0 2 * * *     # Jam 02:00 setiap hari
SIMAN_SCHEDULE=0 3 * * 0         # Jam 03:00 setiap Minggu
```

#### Skenario 3: Penarikan Berkala (Setiap 6 Jam)
```bash
MONSAKTI_SCHEDULE=0 0 */6 * *    # Setiap 6 jam
MYSIMKARI_SCHEDULE=0 0 */6 * *   # Setiap 6 jam
SIMAN_SCHEDULE=0 0 */12 * *      # Setiap 12 jam
```

## 📊 Monitoring

Scheduler akan memberikan log yang detail untuk setiap aktivitas:

```
INFO 🚀 SIMPelv2 - Integration Scheduler Starting...
INFO 📋 Scheduler Configuration:
INFO   • Timezone: Asia/Jakarta
INFO   • MonSAKTI Schedule: 0 2 * * *
INFO   • MySIMKARI Schedule: 0 2 * * *
INFO   • SIMAN Schedule: 0 3 * * 0
INFO ✅ Scheduler started successfully
INFO ⏰ Scheduler is now running. Press Ctrl+C to stop.

INFO 🚀 Starting MonSAKTI scheduled data fetch...
INFO 📊 Fetching MonSAKTI data for all modules...
INFO   → Fetching ADM module data...
INFO   ✓ ADM: 150 records
INFO   💾 Saved 150 records to ./output/monsakti_adm.json
INFO ✅ MonSAKTI data fetch completed successfully
```

## 🔧 Troubleshooting

### Scheduler Tidak Berjalan

1. **Periksa SCHEDULER_ENABLED**
   ```bash
   SCHEDULER_ENABLED=true  # Harus true
   ```

2. **Periksa Cron Expression**
   ```bash
   # Pastikan format cron expression valid
   # Gunakan https://crontab.guru untuk validasi
   ```

3. **Periksa Log Level**
   ```bash
   LOG_LEVEL=info
   RUST_LOG=info
   ```

### Token Expired

Jika token MonSAKTI expired, scheduler akan mencoba reset otomatis. Jika gagal:

1. Login ke portal MonSAKTI
2. Generate token baru untuk setiap module
3. Update di file `.env`

### Storage Error

**Database Connection Failed:**
```bash
# Periksa DATABASE_URL
DATABASE_URL=postgres://user:password@localhost:5432/simpelv2

# Atau gunakan file storage
STORAGE_TYPE=json
OUTPUT_DIR=./output
```

## 🐳 Deploy dengan Docker

### Dockerfile

```dockerfile
FROM rust:1.90 AS builder
WORKDIR /app
COPY . .
RUN cargo build --bin scheduler --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/scheduler /usr/local/bin/scheduler
CMD ["scheduler"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  scheduler:
    build: .
    environment:
      - SCHEDULER_ENABLED=true
      - MONSAKTI_SCHEDULE=0 2 * * *
      - MYSIMKARI_SCHEDULE=0 2 * * *
      - SIMAN_SCHEDULE=0 3 * * 0
      - SCHEDULER_TIMEZONE=Asia/Jakarta
      - DATABASE_URL=postgres://user:password@postgres:5432/simpelv2
    volumes:
      - ./output:/app/output
    restart: unless-stopped
```

## 🔒 Security Best Practices

1. **Environment Variables**: Jangan commit file `.env` ke repository
2. **Token Rotation**: Rotate token secara berkala
3. **Access Control**: Batasi akses ke file output/database
4. **Logging**: Monitor log untuk aktivitas mencurigakan
5. **Rate Limiting**: Gunakan jadwal yang wajar untuk menghindari rate limiting

## 📝 API Coverage

### MonSAKTI
- ✅ ADM (Administrasi)
- ✅ ANG (Anggaran)
- ✅ AST (Aset)
- ✅ BEN (Bendahara)
- ✅ GLP (GAJI/LEMBUR/PERJALANAN DINAS)
- ✅ KOM (Komitmen)
- ✅ PEM (Pembayaran)
- ✅ PER (Perbendaharaan)

### MySIMKARI
- ✅ Satker Data
- ✅ Pegawai Data

### SIMAN
- ✅ All Categories (via getRowCount endpoints)

## 🤝 Contributing

Kontribusi sangat diterima! Silakan buat issue atau pull request untuk improvement.

## 📄 License

MIT License - lihat file LICENSE untuk detail.
