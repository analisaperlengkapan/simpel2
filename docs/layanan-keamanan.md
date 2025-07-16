# README - layanan-keamanan

**Layanan Keamanan** dalam SIMPelv2 bertanggung jawab atas pengelolaan autentikasi, otorisasi, kontrol akses berbasis peran (RBAC), dan keamanan API lintas layanan. Layanan ini merupakan fondasi keamanan seluruh sistem.

---

## 📌 Fungsi Utama

- Autentikasi pengguna (login, token JWT/OPA)
- Otorisasi berbasis peran dan skema
- Manajemen sesi dan token refresh
- Penyimpanan dan enkripsi credential
- Middleware keamanan untuk seluruh layanan
- Deteksi anomali login (opsional via AI-UEBA)

---

## 🧱 Teknologi yang Digunakan

- Bahasa: Go (Gin Framework)
- DB: PostgreSQL (skema: `keamanan`)
- Middleware: JWT, RBAC, OPA/OPA-lite
- Enkripsi: bcrypt, AES-256, TLS 1.3
- Logging: Forward ke Loki (via Fiber Gateway)
- AI: UEBA (User Behavior Analytics, via layanan-ai)

---

## 🔐 Struktur Modul

```
layanan-keamanan/
├── cmd/                    # Entrypoint aplikasi
├── handler/                # HTTP handler untuk login, roles, dsb
├── middleware/             # Middleware otentikasi, audit, dll
├── model/                  # Skema data dan ORM
├── repository/             # Akses database dan query sqlc
├── service/                # Logika bisnis autentikasi dan otorisasi
├── schema.sql              # Skema SQL (sqlc)
├── seed/                   # Data default (admin, peran dasar)
└── main.go
```

---

## ⚙️ API Endpoint Penting

| Method | Endpoint               | Deskripsi                            |
|--------|------------------------|--------------------------------------|
| POST   | `/login`               | Login user, generate JWT             |
| POST   | `/logout`              | Logout dan revoke token              |
| GET    | `/me`                  | Informasi user login                 |
| GET    | `/roles`               | List peran dan hak akses             |
| POST   | `/roles`               | Tambah/mutakhirkan peran             |
| POST   | `/validate-token`      | Validasi token JWT                   |

Semua endpoint dilindungi middleware autentikasi kecuali `/login`.

---

## 🧪 Testing dan Validasi

- Unit test untuk `service/*` dan `handler/*`
- Integrasi test terhadap token dan RBAC
- Fuzzer untuk middleware validasi JWT
- Linting Go dan validasi YAML

---

## 🔍 Observabilitas

- Semua login/logout dicatat di layanan-audit
- Anomali sesi dan brute force dianalisis oleh AI UEBA
- Token expired otomatis di-blacklist

---

## 🚨 Keamanan Tambahan

- Semua password disimpan hashed dengan bcrypt
- Semua koneksi antar layanan pakai TLS mutual auth
- Token memiliki expiry singkat dan refresh
- Admin user hanya bisa dibuat via seed

---

## 🤝 Interkoneksi

Digunakan oleh:
- `gerbang`: Middleware autentikasi API
- `layanan-dasbor`: Identitas pengguna
- `layanan-audit`: Log keamanan
- `layanan-ai`: Deteksi anomali UEBA

---

## 📁 ENV yang Digunakan

```
JWT_SECRET=xxxxxxxxxx
TOKEN_EXPIRY_MIN=15
REFRESH_EXPIRY_HOURS=24
ENCRYPTION_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

## 📄 Lisensi & Akses

Layanan ini bersifat **internal tertutup** dan hanya digunakan dalam sistem SIMPelv2. Semua kontribusi wajib menjaga kerahasiaan pengguna dan informasi sensitif. Pelanggaran dapat berakibat pemutusan akses dan tindakan hukum.

---

> Untuk kontribusi, silakan lihat: `../CONTRIBUTING.md`
