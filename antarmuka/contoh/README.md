# 📦 Antarmuka Contoh - Mockup Microfrontends

## 🎯 Purpose

Direktori `antarmuka/contoh/` berisi **mockup microfrontend** sebagai placeholder untuk modul-modul yang belum diimplementasi atau dalam tahap perencanaan.

> **Note**: Ini adalah mockup untuk menunjukkan struktur standar. Implementasi akhir bisa berbeda sesuai kebutuhan.

## 📂 Available Mockups

| Mockup | Description |
|--------|-------------|
| `badiklat/` | Training & Education |
| `datun/` | Civil Litigation |
| `intel/` | Intelligence & Surveillance |
| `pembinaan/keuangan/` | Financial Management |
| `pembinaan/perencanaan/` | Strategic Planning |
| `pemulihan_aset/` | Asset Recovery |
| `pengawasan/` | Supervision & Oversight |
| `pidmil/` | Military Criminal Law |
| `pidsus/` | Special Crimes |
| `pidum/` | General Criminal |

## 🏗️ Standard Structure

Setiap mockup mengikuti struktur standar:

```
contoh/{module}/
├── Cargo.toml          # Dependencies
├── Trunk.toml          # WASM build config
├── index.html          # HTML template
├── src/
│   ├── lib.rs         # Library entry
│   ├── app.rs         # Main component
│   ├── components/    # UI components
│   └── pages/         # Page components
└── styles/            # CSS files
```

---

> **Superapps**: Jika diperlukan integrasi ke SIMKARI Superapps, mockup ini bisa dijadikan referensi struktur untuk modul baru.
