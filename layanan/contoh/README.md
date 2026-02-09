# 📦 Layanan Contoh - Mockup Backend Services

## 🎯 Purpose

Direktori `layanan/contoh/` berisi **mockup backend service** sebagai placeholder untuk layanan-layanan yang belum diimplementasi atau dalam tahap perencanaan.

> **Note**: Ini adalah mockup untuk menunjukkan struktur standar. Implementasi akhir bisa berbeda sesuai kebutuhan.

## 📂 Available Mockups

| Mockup | Description |
|--------|-------------|
| `badiklat/` | Training & Education service |
| `datun/` | Civil Litigation service |
| `intel/` | Intelligence service |
| `pembinaan/` | Pembinaan services |
| `pemulihan_aset/` | Asset Recovery service |
| `pengawasan/` | Supervision service |
| `pidmil/` | Military Criminal service |
| `pidsus/` | Special Crimes service |
| `pidum/` | General Criminal service |

## 🏗️ Standard Structure

Setiap mockup mengikuti struktur standar:

```
contoh/{service}/
├── Cargo.toml          # Dependencies
├── src/
│   ├── main.rs        # Entry point
│   ├── config.rs      # Configuration
│   ├── database.rs    # Database connection
│   ├── routes.rs      # API routes
│   ├── handlers/      # Request handlers
│   ├── models/        # Data models
│   └── services/      # Business logic
└── migrations/        # Database migrations
```

---

> **Superapps**: Jika diperlukan integrasi ke SIMKARI Superapps, mockup ini bisa dijadikan referensi struktur untuk layanan baru.
