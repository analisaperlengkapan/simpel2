# Dokumentasi Integrasi SIMAN API v2.0

## Tentang SIMAN

**SIMAN (Sistem Informasi Manajemen Aset Negara)** adalah sistem yang dikelola oleh Kementerian Keuangan Republik Indonesia untuk pengelolaan Barang Milik Negara (BMN). API v2.0 menyediakan akses terstruktur ke berbagai kategori aset milik instansi pemerintah.

## Arsitektur Integrasi

### Autentikasi

SIMAN API menggunakan **OAuth2 Client Credentials Flow** melalui SSO Kemenkeu:

```
1. Request Token: POST https://sso.kemenkeu.go.id/connect/token
   Body: client_id, client_secret, grant_type=client_credentials

2. Receive Token: access_token (Bearer), expires_in (seconds)

3. API Calls: Authorization: Bearer {access_token}
```

Token management dilakukan otomatis oleh `MonsaktiClient` dengan:

- Caching token yang valid
- Auto-refresh saat token expired (60 detik sebelum expire)
- Thread-safe token sharing untuk parallel requests

### Endpoint Structure

**Base URL:** `https://api-gw.kemenkeu.go.id/gateway/SLDKSimanKL/2.0/`

**Alternate URL (dari Panduan PDF):** `https://apigateway.kemenkeu.go.id/gateway/SLDKSimanKL/2.0/`

**Catatan:** Kedua URL tersebut valid (alias), konfigurasi default menggunakan `api-gw.kemenkeu.go.id`

**Available Endpoints:**

1. **getRowCount** (GET)

   - Path: `/getRowCount/{BA_KEY}/{TABLE_NAME}`
   - Mendapatkan total jumlah record

2. **getAset{Category}** (POST)
   - Path: `/getAset{Category}`
   - Body: `BA_KEY`, `ID_1` (start), `ID_2` (end)
   - Pagination-based data retrieval

## Kategori Aset

SIMAN API v2.0 menyediakan 15 kategori aset:

| Kategori             | Endpoint                   | Tabel Database                      |
| -------------------- | -------------------------- | ----------------------------------- |
| Alat Besar           | `getAsetAlatBesar`         | `SIMAN2_M_ASET_ALAT_BESAR`          |
| Angkutan Bermotor    | `getAsetAngkutanBermotor`  | `SIMAN2_M_ASET_ANGKUTAN_BERMOTOR`   |
| Alat Persenjataan    | `getAsetAlatPersenjataan`  | `SIMAN2_M_ASET_ALAT_PERSENJATAAN`   |
| Tak Berwujud         | `getAsetTakBerwujud`       | `SIMAN2_M_ASET_ASET_TAK_BERWUJUD`   |
| Bangunan Air         | `getAsetBangunanAir`       | `SIMAN2_M_ASET_BANGUNAN_AIR`        |
| Gedung & Bangunan    | `getAsetGedungBangunan`    | `SIMAN2_M_ASET_GEDUNG_BANGUNAN`     |
| Instalasi & Jaringan | `getAsetInstalasiJaringan` | `SIMAN2_M_ASET_INSTALASI_JARINGAN`  |
| Jalan & Jembatan     | `getAsetJalandanJembatan`  | `SIMAN2_M_ASET_JALAN_DAN_JEMBATAN`  |
| Peralatan Non-TIK    | `getAsetNonTIK`            | `SIMAN2_M_ASET_NON_TIK`             |
| Rumah                | `getAsetRumah`             | `SIMAN2_M_ASET_RUMAH`               |
| Tanah                | `getAsetTanah`             | `SIMAN2_M_ASET_ASET_TANAH`          |
| Aset Tetap Lainnya   | `getAsetTetapLainnya`      | `SIMAN2_M_ASET_ASET_TETAP_LAINNYA`  |
| KDP                  | `getAsetKDP`               | `SIMAN2_M_ASET_KDP`                 |
| Khusus TIK           | `getAsetKhususTIK`         | `SIMAN2_M_ASET_KHUSUS_TIK`          |
| Tetap Renovasi       | `getAsetTetapRenovasi`     | `SIMAN2_M_ASET_ASET_TETAP_RENOVASI` |

**Catatan Penting:** Beberapa nama tabel memiliki prefix "ASET\_" ganda (sesuai Panduan SLDK resmi).

## Konfigurasi

### Environment Variables

Tambahkan ke file `.env`:

```env
# SIMAN API v2.0 Configuration
SIMAN_BASE_URL=https://api-gw.kemenkeu.go.id
SIMAN_TOKEN_URL=https://sso.kemenkeu.go.id/connect/token
SIMAN_CLIENT_ID=simanv2.kejagung
SIMAN_CLIENT_SECRET=your_secret_here
SIMAN_BA_KEY=your_satker_code
```

**Penjelasan:**

- `SIMAN_CLIENT_ID`: Client ID dari Kemenkeu (contoh: `simanv2.kejagung`)
- `SIMAN_CLIENT_SECRET`: Secret key untuk OAuth2
- `SIMAN_BA_KEY`: Kode satuan kerja (BA_KEY) untuk Kejaksaan RI

### Cara Mendapatkan Credentials

1. Hubungi **Biro TI Kejaksaan Agung** atau **Kementerian Keuangan**
2. Request akses SIMAN API v2.0 untuk satker Anda
3. Dapatkan: `client_id`, `client_secret`, dan `BA_KEY`

## Penggunaan

### Basic Usage

```rust
use layanan_integrasi::{Config, MonsaktiClient};
use layanan_integrasi::siman::{SimanAssetCategory, get_row_count, get_aset_by_category};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = Config::from_env()?;
    let mut client = MonsaktiClient::new(config).await?;

    // Get total count
    let count = get_row_count(&mut client, SimanAssetCategory::Tanah).await?;
    println!("Total Tanah: {}", count);

    // Fetch data (pagination: records 1-100)
    let data = get_aset_by_category(
        &mut client,
        SimanAssetCategory::Tanah,
        1,
        100
    ).await?;

    println!("Retrieved {} records", data.len());
    Ok(())
}
```

### Automatic Pagination

```rust
use layanan_integrasi::siman::fetch_all_aset_paginated;

// Fetch all data with auto-pagination (chunks of 1000)
let all_data = fetch_all_aset_paginated(
    &mut client,
    SimanAssetCategory::GedungBangunan,
    1000
).await?;

println!("Total records: {}", all_data.len());
```

### Using Convenience Functions

```rust
use layanan_integrasi::siman::{
    get_aset_tanah,
    get_aset_angkutan_bermotor,
    get_aset_gedung_bangunan,
};

// Direct category functions
let tanah = get_aset_tanah(&mut client, 1, 100).await?;
let kendaraan = get_aset_angkutan_bermotor(&mut client, 1, 100).await?;
let gedung = get_aset_gedung_bangunan(&mut client, 1, 100).await?;
```

### Iterate All Categories

```rust
use layanan_integrasi::siman::SimanAssetCategory;

for category in SimanAssetCategory::all() {
    println!("Processing: {}", category.description());

    let count = get_row_count(&mut client, category).await?;
    println!("  Total: {} records", count);

    if count > 0 {
        let data = get_aset_by_category(&mut client, category, 1, 100).await?;
        // Process data...
    }
}
```

### Save to File

```rust
// Save to JSON
let json_data = serde_json::to_value(&data)?;
client.save_to_json(&json_data, "siman_tanah.json").await?;

// Save to CSV
client.save_to_csv(&json_data, "siman_tanah.csv").await?;
```

## Response Structure

### Row Count Response

```json
[
  {
    "table_name": "SIMAN2_M_ASET_TANAH",
    "row_count": 1523
  }
]
```

### Asset Data Response

```json
{
  "status": 200,
  "message": "Success",
  "data": [
    {
      "ID": 1,
      "KODE_BARANG": "01.01.01.001",
      "NAMA_BARANG": "Tanah Kantor",
      "LUAS": 5000,
      "SATUAN": "M2",
      "NILAI_PEROLEHAN": 50000000000
      // ... other fields
    }
    // ... more records
  ]
}
```

## Error Handling

```rust
use layanan_integrasi::error::MonsaktiError;

match get_aset_tanah(&mut client, 1, 100).await {
    Ok(data) => {
        println!("Success: {} records", data.len());
    }
    Err(MonsaktiError::TokenExpired) => {
        // Token akan di-refresh otomatis, retry akan dilakukan
        println!("Token expired, will auto-retry");
    }
    Err(MonsaktiError::ApiError(msg)) => {
        println!("API Error: {}", msg);
    }
    Err(MonsaktiError::ConfigError(msg)) => {
        println!("Config Error: {}", msg);
        // Check your .env file
    }
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```

## Best Practices

### 1. Pagination Strategy

Untuk dataset besar, gunakan pagination dengan chunk size optimal:

```rust
// Baik: chunk 1000 untuk stabilitas
let data = fetch_all_aset_paginated(&mut client, category, 1000).await?;

// Hindari: chunk terlalu besar (timeout risk)
// let data = fetch_all_aset_paginated(&mut client, category, 10000).await?;
```

### 2. Rate Limiting

Implementasi delay untuk menghindari rate limiting:

```rust
use tokio::time::{sleep, Duration};

for category in SimanAssetCategory::all() {
    let data = get_aset_by_category(&mut client, category, 1, 1000).await?;
    // Process data...

    sleep(Duration::from_millis(500)).await; // 500ms delay
}
```

### 3. Parallel Processing

Untuk multiple categories, gunakan parallel processing:

```rust
use tokio::task;

let mut handles = vec![];

for category in SimanAssetCategory::all() {
    let mut client_clone = client.clone();

    let handle = task::spawn(async move {
        fetch_all_aset_paginated(&mut client_clone, category, 1000).await
    });

    handles.push((category, handle));
}

for (category, handle) in handles {
    match handle.await? {
        Ok(data) => println!("{}: {} records", category.description(), data.len()),
        Err(e) => println!("{}: Error - {}", category.description(), e),
    }
}
```

### 4. Data Validation

Selalu validasi data sebelum processing:

```rust
let data = get_aset_tanah(&mut client, 1, 100).await?;

for record in data {
    // Validate required fields
    let kode_barang = record.get("KODE_BARANG")
        .and_then(|v| v.as_str())
        .ok_or("Missing KODE_BARANG")?;

    let nilai = record.get("NILAI_PEROLEHAN")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // Process validated data...
}
```

## Troubleshooting

### Issue: "SIMAN_CLIENT_ID not configured"

**Solusi:** Pastikan `.env` file sudah dikonfigurasi dengan benar:

```bash
cp .env.example .env
# Edit .env dan isi credentials SIMAN
```

### Issue: Token expired terus-menerus

**Penyebab:** Client secret mungkin salah atau expired

**Solusi:**

1. Verifikasi `SIMAN_CLIENT_SECRET` di `.env`
2. Hubungi admin untuk regenerate credentials
3. Pastikan system time server sudah benar (untuk token validation)

### Issue: Empty data atau row count = 0

**Penyebab:** BA_KEY tidak valid atau tidak ada data untuk satker

**Solusi:**

1. Verifikasi `SIMAN_BA_KEY` benar (hubungi admin)
2. Cek apakah satker memiliki data di kategori tersebut
3. Test dengan kategori lain (misal: Tanah biasanya ada)

### Issue: HTTP 403 Forbidden

**Penyebab:** Token tidak valid atau tidak ada permission

**Solusi:**

1. Pastikan token OAuth2 berhasil didapat (cek logs)
2. Verifikasi client_id dan client_secret
3. Pastikan satker Anda sudah terdaftar di SIMAN

## Performance Considerations

### Recommended Pagination Sizes

| Total Records    | Recommended Chunk Size |
| ---------------- | ---------------------- |
| < 1,000          | 500                    |
| 1,000 - 10,000   | 1,000                  |
| 10,000 - 100,000 | 1,000                  |
| > 100,000        | 1,000 dengan parallel  |

### Memory Usage

```rust
// Untuk dataset besar, process per chunk alih-alih load all
let total = get_row_count(&mut client, category).await?;
let chunk_size = 1000u32;

for start in (1..=total as u32).step_by(chunk_size as usize) {
    let end = (start + chunk_size - 1).min(total as u32);
    let chunk = get_aset_by_category(&mut client, category, start, end).await?;

    // Process chunk immediately
    process_chunk(chunk).await?;

    // Chunk is dropped here, freeing memory
}
```

## Integration dengan SIMPEL

SIMAN integration dapat digunakan untuk:

1. **Sinkronisasi Data BMN**: Update data aset dari SIMAN ke database lokal
2. **Validasi Aset**: Cross-check data lokal dengan data SIMAN pusat
3. **Reporting**: Generate laporan BMN berbasis data real-time SIMAN
4. **Dashboard**: Display statistik aset dari berbagai sumber

### Contoh: Sync Service

```rust
pub async fn sync_siman_to_local(
    siman_client: &mut MonsaktiClient,
    local_db: &tokio_postgres::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    for category in SimanAssetCategory::all() {
        tracing::info!("Syncing {}", category.description());

        let data = fetch_all_aset_paginated(
            siman_client,
            category,
            1000
        ).await?;

        // Save to local database
        let table_name = format!("siman_{}", category.table_name().to_lowercase());
        let json_data = serde_json::to_value(&data)?;

        siman_client.save_to_postgres(&table_name, &json_data).await?;

        tracing::info!("Synced {} records for {}", data.len(), category.description());
    }

    Ok(())
}
```

## Referensi

- **Postman Collection**: `API Siman v2.0 - Kejaksaan RI.postman_collection.json`
- **Official Guide**: `Panduan Penggunaan Web Service SLDK-Kejaksaan RI.pdf`
- **Source Code**: `layanan/shared/integrasi/src/siman/`
- **Examples**: `layanan/shared/integrasi/examples/siman_example.rs`

## Support

Untuk pertanyaan atau masalah:

1. Check dokumentasi di `layanan/shared/integrasi/README.md`
2. Review example code di `examples/siman_example.rs`
3. Hubungi team development SIMPEL
4. Untuk credential issues: hubungi Biro TI Kejaksaan Agung

---

**Version:** 1.0.0
**Last Updated:** November 2025
**Maintained by:** SIMPEL Development Team
