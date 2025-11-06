# SIMAN API Quick Reference

## Setup (2 minutes)

```bash
# 1. Add to .env
SIMAN_CLIENT_ID=simanv2.kejagung
SIMAN_CLIENT_SECRET=your_secret
SIMAN_BA_KEY=your_satker_code

# 2. Run example
cargo run --example siman_example
```

## Common Usage

### Get Total Count

```rust
use simpelv2_integrasi::siman::{SimanAssetCategory, get_row_count};

let count = get_row_count(&mut client, SimanAssetCategory::Tanah).await?;
```

### Fetch Data (Paginated)

```rust
use simpelv2_integrasi::siman::get_aset_by_category;

let data = get_aset_by_category(
    &mut client,
    SimanAssetCategory::Tanah,
    1,    // start
    100   // end
).await?;
```

### Fetch All (Auto-pagination)

```rust
use simpelv2_integrasi::siman::fetch_all_aset_paginated;

let all_data = fetch_all_aset_paginated(
    &mut client,
    SimanAssetCategory::GedungBangunan,
    1000  // chunk size
).await?;
```

### Direct Category Functions

```rust
use simpelv2_integrasi::siman::{
    get_aset_tanah,
    get_aset_angkutan_bermotor,
    get_aset_gedung_bangunan,
};

let tanah = get_aset_tanah(&mut client, 1, 100).await?;
let kendaraan = get_aset_angkutan_bermotor(&mut client, 1, 100).await?;
```

## Available Categories

| Category            | Function                        | Description              |
| ------------------- | ------------------------------- | ------------------------ |
| `AlatBesar`         | `get_aset_alat_besar()`         | Heavy Equipment          |
| `AngkutanBermotor`  | `get_aset_angkutan_bermotor()`  | Vehicles                 |
| `AlatPersenjataan`  | `get_aset_alat_persenjataan()`  | Weapons                  |
| `TakBerwujud`       | `get_aset_tak_berwujud()`       | Intangible Assets        |
| `BangunanAir`       | `get_aset_bangunan_air()`       | Water Buildings          |
| `GedungBangunan`    | `get_aset_gedung_bangunan()`    | Buildings                |
| `InstalasiJaringan` | `get_aset_instalasi_jaringan()` | Installations            |
| `JalandanJembatan`  | `get_aset_jalan_jembatan()`     | Roads & Bridges          |
| `NonTIK`            | `get_aset_non_tik()`            | Non-IT Equipment         |
| `Rumah`             | `get_aset_rumah()`              | Houses                   |
| `Tanah`             | `get_aset_tanah()`              | Land                     |
| `TetapLainnya`      | `get_aset_tetap_lainnya()`      | Other Fixed Assets       |
| `KDP`               | `get_aset_kdp()`                | Construction in Progress |
| `KhususTIK`         | `get_aset_khusus_tik()`         | IT Equipment             |
| `TetapRenovasi`     | `get_aset_tetap_renovasi()`     | Renovation Assets        |

## Iterate All Categories

```rust
for category in SimanAssetCategory::all() {
    let count = get_row_count(&mut client, category).await?;
    println!("{}: {} records", category.description(), count);
}
```

## Save Results

```rust
let json = serde_json::to_value(&data)?;

// JSON file
client.save_to_json(&json, "siman_data.json").await?;

// CSV file
client.save_to_csv(&json, "siman_data.csv").await?;

// PostgreSQL
client.save_to_postgres("siman_tanah", &json).await?;
```

## Error Handling

```rust
match get_aset_tanah(&mut client, 1, 100).await {
    Ok(data) => println!("Got {} records", data.len()),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Full Documentation

- **Complete Guide**: `dokumentasi_siman.md`
- **Example Code**: `examples/siman_example.rs`
- **Implementation**: `SIMAN_IMPLEMENTATION_SUMMARY.md`
