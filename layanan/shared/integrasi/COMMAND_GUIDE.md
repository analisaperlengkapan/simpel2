# Panduan Command SIMPelv2 Layanan Integrasi

## Ringkasan Perubahan

**FIXED**: Mode `complete` sekarang **TIDAK menarik MySIMKARI** saat menggunakan `--source monsakti`.

### Sebelumnya (Bug)

```bash
# ❌ BUG: Ini menarik MySIMKARI juga meskipun --source monsakti
cargo run --release -- --source monsakti --mode complete
```

### Setelah Diperbaiki (Fixed)

```bash
# ✅ BENAR: Hanya menarik MonSAKTI (global refs + all satker)
cargo run --release -- --source monsakti --mode complete

# ✅ BENAR: Menarik SEMUA (MySIMKARI + MonSAKTI)
cargo run --release -- --source all --mode complete
```

---

## Command Reference

### 1. **MonSAKTI Only** (`--source monsakti`)

#### Mode: COMPLETE (Global + All Satker)

```bash
cargo run --release -- --source monsakti --mode complete
```

**Output:**

- ✅ Global reference data (ADM, KOM, dll)
- ✅ MonSAKTI data untuk semua satker (568 satker)
- ❌ MySIMKARI (TIDAK termasuk)

#### Mode: GLOBAL (Reference Data Only)

```bash
cargo run --release -- --source monsakti --mode global
```

**Output:**

- ✅ Global reference data saja
- ❌ Per-satker data (skip)
- ❌ MySIMKARI (skip)

#### Mode: SATKER (Per-Satker Only)

```bash
cargo run --release -- --source monsakti --mode satker
```

**Output:**

- ❌ Global reference (skip)
- ✅ MonSAKTI data untuk semua satker
- ❌ MySIMKARI (skip)

#### Mode: SINGLE (Test Single Satker)

```bash
cargo run --release -- --source monsakti --mode single --test-satker 414830
```

**Output:**

- ✅ Data untuk satker 414830 saja

#### Mode: LIST (List Satker Codes)

```bash
cargo run --release -- --source monsakti --mode list
```

**Output:**

- ✅ List kode satker (568 satker)
- ❌ Tidak fetch data

---

### 2. **MySIMKARI Only** (`--source mysimkari`)

```bash
cargo run --release -- --source mysimkari
```

**Output:**

- ✅ Data pegawai dari semua satker (542 satker)
- ❌ MonSAKTI (skip)

---

### 3. **ALL Sources** (`--source all`)

#### Mode: COMPLETE (Everything)

```bash
cargo run --release -- --source all --mode complete
```

**Output:**

- ✅ Global reference data (MonSAKTI)
- ✅ MySIMKARI data (semua pegawai)
- ✅ MonSAKTI per-satker data

**Execution Order:**

1. Global references (MonSAKTI)
2. MySIMKARI data
3. MonSAKTI per-satker data

#### Mode: GLOBAL/SATKER/SINGLE

```bash
# Akan proses MonSAKTI dulu, kemudian MySIMKARI secara terpisah
cargo run --release -- --source all --mode global
```

---

## Background Execution

### Timeout 20 menit dengan log

```bash
timeout 1200 cargo run --release -- --source monsakti --mode complete 2>&1 | tee /tmp/monsakti_$(date +%H%M%S).log &
```

### Monitor Progress

```bash
# Monitor real-time
tail -f /tmp/monsakti_*.log

# Check for MonSAKTI activity
grep -i "monsakti\|fetching.*API" /tmp/monsakti_*.log | tail -20

# Check for MySIMKARI activity
grep -i "mysimkari\|pegawai" /tmp/monsakti_*.log | tail -20
```

---

## Common Use Cases

### Production Data Pull (MonSAKTI Only)

```bash
# Recommended for production MonSAKTI sync
timeout 1200 cargo run --release -- --source monsakti --mode complete
```

### Full System Sync (Everything)

```bash
# Pull EVERYTHING (MySIMKARI + MonSAKTI)
timeout 2400 cargo run --release -- --source all --mode complete
```

### Testing Single Satker

```bash
# Test dengan satker Kejaksaan Agung
cargo run --release -- --source monsakti --mode single --test-satker 414830
```

### Check Available Satker

```bash
# List all satker codes
cargo run --release -- --source monsakti --mode list | tee satker_list.txt
```

---

## Architecture Changes

### File Modified

1. `src/batch/orchestrator.rs`

   - Split `fetch_all_data()` → MonSAKTI only
   - Added `fetch_all_data_with_mysimkari()` → ALL sources

2. `src/main.rs`

   - Updated `process_monsakti()` → Uses `fetch_all_data()`
   - Updated `Source::All` logic → Uses `fetch_all_data_with_mysimkari()`

3. `src/lib.rs`
   - Exported `fetch_all_data_with_mysimkari`

### Function Mapping

| Command                             | Function Called                   | MySIMKARI? |
| ----------------------------------- | --------------------------------- | ---------- |
| `--source monsakti --mode complete` | `fetch_all_data()`                | ❌ NO      |
| `--source all --mode complete`      | `fetch_all_data_with_mysimkari()` | ✅ YES     |
| `--source mysimkari`                | `fetch_mysimkari()`               | ✅ YES     |

---

## Performance Notes

- **Global references**: ~2-3 menit
- **MySIMKARI**: ~5-8 menit (542 satker)
- **MonSAKTI per-satker**: ~10-15 menit (568 satker)
- **Total (ALL)**: ~20-30 menit

## Token Management

- Automatic token refresh on 403 Forbidden
- Token saved to database
- Retry with new token after reset
- Bearer token from `.env` file

---

## Examples

### Example 1: Quick Test

```bash
# Test global references only (fastest)
cargo run --release -- --source monsakti --mode global
```

### Example 2: Production MonSAKTI Sync

```bash
# Full MonSAKTI sync with logging
timeout 1200 cargo run --release -- \
  --source monsakti \
  --mode complete \
  2>&1 | tee /tmp/monsakti_sync_$(date +%Y%m%d_%H%M%S).log
```

### Example 3: Full System Sync

```bash
# Everything including MySIMKARI
timeout 2400 cargo run --release -- \
  --source all \
  --mode complete \
  2>&1 | tee /tmp/full_sync_$(date +%Y%m%d_%H%M%S).log
```

---

## Troubleshooting

### MySIMKARI tetap muncul saat `--source monsakti`

- Pastikan build terbaru: `cargo build --release`
- Periksa log: harus ada "MonSAKTI only" bukan "ALL sources"

### Token expired errors

- Token auto-refresh aktif
- Check `.env` untuk Bearer token
- Periksa database table `api_tokens`

### Rate limiting

- Gunakan sequential mode (default)
- Avoid `--parallel` flag untuk production

---

## Version Info

- **Fixed**: November 6, 2025
- **Issue**: Mode complete menarik MySIMKARI meskipun --source monsakti
- **Solution**: Split fetch_all_data into two functions
- **Impact**: Breaking change - behavior now matches documentation
