# Portal Cleanup Plan - Redundansi Removal

## Status Saat Ini

### File Aktif (KEEP):
- `src/app.rs` - File utama yang digunakan lib.rs (153 lines)
- `style/main.scss` - SCSS file yang dikonfigurasi di Cargo.toml (204 lines)
- `index.html` - HTML template utama
- `lib.rs` - Entry point menggunakan app.rs

### File Redundant (REMOVE):
- `src/app_backup.rs` - Backup dengan syntax error (181 lines)
- `src/app_new.rs` - Versi alternatif tidak digunakan (155 lines)
- `styles/` folder - CSS minified tidak digunakan
- `styles/main.css` - File CSS generik tidak tereferensi

### Alasan Pembersihan:

#### 1. App Files:
- `app_backup.rs`: Corrupt headers, duplikasi import `use leptos::*;` + `use leptos::prelude::*;`
- `app_new.rs`: Versi alternatif dengan imports berlebihan tidak digunakan
- `app.rs`: Clean, menggunakan minimal shared components, syntax correct

#### 2. Style Folders:
- `style/`: SCSS dengan variabel Kejaksaan yang proper, dikonfigurasi di Cargo.toml
- `styles/`: CSS minified generik tidak sesuai design system

## Action Items:

### Phase 1: Safe Backup
```bash
# Backup files sebelum menghapus
cp src/app_backup.rs ../backup/
cp src/app_new.rs ../backup/
cp -r styles ../backup/
```

### Phase 2: Remove Redundant Files
```bash
rm src/app_backup.rs
rm src/app_new.rs
rm -rf styles/
```

### Phase 3: Validation
- Test build: `trunk build`
- Test serve: `trunk serve --port 8080`
- Verify styling: Check Kejaksaan theme rendering

## Expected Results:
- Portal size reduction: ~400 lines of redundant code
- Build clarity: Single app.rs entry point
- Style consistency: Only SCSS with Kejaksaan design system
- Cleaner file structure for maintenance
