# Simpel Web - Sistem Manajemen Aset dan Pengadaan

Simpel Web adalah aplikasi berbasis Laravel untuk manajemen aset, pengadaan barang/jasa, SDM, dan kebutuhan organisasi Kejaksaan Republik Indonesia.

## 🚀 Fitur Utama

### 📊 Manajemen Aset

- **Asset Management**: Pengelolaan berbagai jenis aset (Gedung, Tanah, Kendaraan, dll)
- **QR Code Generator**: Generate QR code untuk setiap aset
- **Asset Tracking**: Pelacakan lokasi dan status aset
- **Asset Categories**:
  - Asset TIK (Teknologi Informasi & Komunikasi)
  - Asset Non-TIK
  - Asset Konstruksi
  - Asset Jaringan
  - Asset Senjata
  - Asset Angkutan
  - Asset Bangunan Air
  - Asset Jalan & Jembatan
  - Asset Rumah
  - Asset Lainnya

### 🛒 Pengadaan Barang/Jasa

- **Pengadaan Management**: Sistem pengadaan terintegrasi
- **SPSE Integration**: Integrasi dengan sistem SPSE
- **Distribusi**: Manajemen distribusi barang
- **Kontrak Management**: Pengelolaan kontrak pengadaan
- **BAST Management**: Berita Acara Serah Terima

### 👥 SDM (Sumber Daya Manusia)

- **Pegawai Management**: Data pegawai dan profil
- **Role Management**: Sistem role dan permission
- **Aktivitas Tracking**: Log aktivitas pengguna
- **Pakaian Dinas**: Manajemen pakaian dinas pegawai

### 📋 Analisis Kebutuhan

- **BMN Analysis**: Analisis kebutuhan Barang Milik Negara
- **Kelayakan Analysis**: Analisis kelayakan pengadaan
- **Prioritas Management**: Penyusunan prioritas kebutuhan
- **Pengajuan System**: Sistem pengajuan kebutuhan

### 🔧 Fitur Teknis

- **Two-Factor Authentication (2FA)**: Keamanan tambahan
- **JWT Authentication**: API authentication
- **Excel Import/Export**: Import dan export data Excel
- **PDF Generation**: Generate laporan PDF
- **QR Code Generation**: Generate QR code untuk aset
- **File Management**: Upload dan manajemen file
- **Notification System**: Sistem notifikasi
- **Backup System**: Sistem pencadangan data
- **Integration Logs**: Log integrasi eksternal

## 🛠️ Tech Stack

### Backend

- **Laravel 10.48.29** - PHP Framework
- **PHP 8.1+** - Programming Language
- **PostgreSQL** - Database
- **JWT Auth** - API Authentication
- **Laravel Sanctum** - API Token Authentication
- **Google 2FA** - Two-Factor Authentication

### Frontend

- **Blade Templates** - Template Engine
- **Vite** - Build Tool
- **Axios** - HTTP Client
- **Prettier** - Code Formatter

### Libraries & Packages

- **Maatwebsite Excel** - Excel Import/Export
- **Laravel MPDF** - PDF Generation
- **Endroid QR Code** - QR Code Generation
- **Bacon QR Code** - QR Code Library
- **PHP Spreadsheet** - Excel Processing
- **PHP Word** - Word Document Processing
- **Guzzle HTTP** - HTTP Client
- **League CSV** - CSV Processing

## 📦 Instalasi

### Prerequisites

- PHP 8.1+
- Composer
- Node.js 18+
- PostgreSQL
- Git

### Langkah Instalasi

1. **Clone repository:**

   ```bash
   git clone <repo-url>
   cd simpel_web
   ```

2. **Install dependensi PHP:**

   ```bash
   composer install
   ```

3. **Install dependensi Node.js:**

   ```bash
   npm install
   # atau
   pnpm install
   ```

4. **Setup environment:**

   ```bash
   cp .env.example .env
   # Edit .env sesuai konfigurasi database dan aplikasi
   php artisan key:generate
   ```

5. **Konfigurasi database:**

   ```bash
   # Edit .env dengan konfigurasi PostgreSQL
   DB_CONNECTION=pgsql
   DB_HOST=127.0.0.1
   DB_PORT=5432
   DB_DATABASE=simpel_web
   DB_USERNAME=your_username
   DB_PASSWORD=your_password
   ```

6. **Migrasi database:**

   ```bash
   php artisan migrate
   ```

7. **Seed database (opsional):**

   ```bash
   php artisan db:seed
   ```

8. **Build assets:**

   ```bash
   npm run build
   ```

9. **Jalankan aplikasi:**

   ```bash
   php artisan serve
   ```

## 🧪 Testing

```bash
# Jalankan semua test
php artisan test

# Jalankan test dengan coverage
php artisan test --coverage

# Jalankan test spesifik
php artisan test --filter=AssetTest
```

## 🚀 Deployment

```bash
# Push ke kedua repository
git pushall

# Build untuk production
npm run build

# Optimize untuk production
php artisan config:cache
php artisan route:cache
php artisan view:cache
```

## 🔄 CI/CD

![Pipeline](https://gitlab.com/analisiskebutuhan/simpel_web/badge/main/pipeline.svg)

Pipeline CI/CD menggunakan GitLab CI dengan tahapan:

- **Setup**: Install dependensi PHP dan Node.js
- **Test**: Jalankan test dengan database PostgreSQL
- **Build**: Build aplikasi untuk production
- **Deploy**: Deploy ke production server (manual)

## 📁 Struktur Proyek

```
simpel_web/
├── app/
│   ├── Http/Controllers/     # Controllers
│   │   ├── Asset/           # Asset Management
│   │   ├── Bmn/             # BMN Management
│   │   ├── Pengadaan/       # Procurement
│   │   ├── Sdm/             # HR Management
│   │   └── ...
│   ├── Models/              # Eloquent Models
│   ├── Services/            # Business Logic
│   └── ...
├── config/                  # Configuration Files
├── database/               # Migrations & Seeders
├── resources/              # Views & Assets
├── routes/                 # Route Definitions
└── storage/                # File Storage
```

## 🔐 Keamanan

- **Two-Factor Authentication (2FA)**
- **JWT Token Authentication**
- **Role-based Access Control**
- **CSRF Protection**
- **SQL Injection Prevention**
- **XSS Protection**

## 📊 Database Schema

Aplikasi menggunakan PostgreSQL dengan tabel utama:

- `users` - Data pengguna
- `assets` - Data aset
- `bmn_*` - Tabel BMN
- `pengadaan_*` - Tabel pengadaan
- `sdm_*` - Tabel SDM
- `master_*` - Tabel master data

## 🔗 Integrasi

- **SPSE (Sistem Pengadaan Secara Elektronik)**
- **Monsakti (Monitoring Aset Kejaksaan)**
- **Mysimkari (Sistem Informasi Kejaksaan)**
- **Google reCAPTCHA Enterprise**

## 👥 Kontribusi

Lihat [CONTRIBUTING.md](CONTRIBUTING.md) untuk panduan kontribusi.

## 📝 Changelog

Lihat [CHANGELOG.md](CHANGELOG.md) untuk catatan perubahan.

## 📄 Lisensi

Lihat [LICENSE](LICENSE) untuk detail lisensi.

## 🚀 Workflow Development

1. Buat branch baru: `git checkout -b feature/nama-fiturnya`
2. Commit perubahan: `git commit -m "feat: deskripsi perubahan"`
3. Push ke kedua repository: `git pushall`
4. Buat Merge Request di GitLab

## 📞 Support

Untuk bantuan teknis atau pertanyaan, silakan buat issue di repository ini.

---

**Simpel Web** - Sistem Manajemen Aset dan Pengadaan Kejaksaan Republik Indonesia
