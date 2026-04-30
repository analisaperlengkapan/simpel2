# Changelog

Semua perubahan penting pada proyek ini akan didokumentasikan di file ini.

Format berdasarkan [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
dan proyek ini mengikuti [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Sistem manajemen aset terintegrasi
- Pengadaan barang/jasa dengan integrasi SPSE
- Manajemen SDM dengan role-based access
- Analisis kebutuhan BMN
- Two-Factor Authentication (2FA)
- QR Code generator untuk aset
- Excel/PDF import/export
- Sistem notifikasi
- Backup dan restore data
- Integrasi dengan sistem eksternal

### Changed
- Migrasi dari sistem lama ke Laravel 10
- Peningkatan keamanan dengan JWT authentication
- Optimasi performa database PostgreSQL
- UI/UX yang lebih modern dan responsif

### Fixed
- Bug pada sistem autentikasi
- Masalah sinkronisasi data
- Error handling yang lebih baik

## [1.0.0] - 2024-06-20

### Added
- **Core System**
  - Laravel 10.48.29 framework
  - PostgreSQL database integration
  - JWT authentication system
  - Two-Factor Authentication (2FA)
  - Role-based access control

- **Asset Management**
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
  - QR Code generation untuk setiap aset
  - Asset tracking dan monitoring

- **Procurement System**
  - Pengadaan barang/jasa
  - Integrasi SPSE (Sistem Pengadaan Secara Elektronik)
  - Distribusi management
  - Kontrak management
  - BAST (Berita Acara Serah Terima)
  - Monitoring pengadaan

- **HR Management**
  - Data pegawai management
  - Role dan permission system
  - Aktivitas tracking
  - Pakaian dinas management
  - Profil pegawai

- **Analysis System**
  - Analisis kebutuhan BMN
  - Analisis kelayakan pengadaan
  - Penyusunan prioritas kebutuhan
  - Sistem pengajuan kebutuhan
  - Monitoring dan evaluasi

- **Technical Features**
  - Excel import/export dengan Maatwebsite Excel
  - PDF generation dengan Laravel MPDF
  - QR Code generation dengan Endroid QR Code
  - File upload dan management
  - Notification system
  - Backup dan restore system
  - Integration logs
  - API endpoints

- **External Integrations**
  - SPSE (Sistem Pengadaan Secara Elektronik)
  - Monsakti (Monitoring Aset Kejaksaan)
  - Mysimkari (Sistem Informasi Kejaksaan)
  - Google reCAPTCHA Enterprise

### Security
- JWT token authentication
- Two-Factor Authentication (2FA)
- CSRF protection
- SQL injection prevention
- XSS protection
- Role-based access control
- Secure file upload validation

### Performance
- Database optimization dengan PostgreSQL
- Caching system
- Asset compression
- API response optimization
- Frontend optimization dengan Vite

## [0.9.0] - 2024-05-15

### Added
- Initial project setup
- Basic authentication system
- Database migrations
- Core models dan controllers

### Changed
- Migrasi dari sistem legacy
- Setup development environment

## [0.8.0] - 2024-04-01

### Added
- Project initialization
- Laravel framework setup
- Basic configuration

---

## Migration Guide

### From v0.9.0 to v1.0.0
1. Update Laravel ke versi 10.48.29
2. Jalankan `composer update`
3. Jalankan `php artisan migrate`
4. Update environment variables
5. Test semua fitur utama

### From v0.8.0 to v0.9.0
1. Install dependencies baru
2. Jalankan database migrations
3. Setup authentication system

---

## Deprecation Notices

- Versi PHP < 8.1 tidak didukung mulai v1.0.0
- MySQL database tidak didukung mulai v1.0.0 (migrasi ke PostgreSQL)
- Laravel < 10 tidak didukung mulai v1.0.0

---

## Breaking Changes

### v1.0.0
- Migrasi database dari MySQL ke PostgreSQL
- Perubahan struktur authentication
- Update API endpoints
- Perubahan format response API

---

## Contributors

Terima kasih kepada semua kontributor yang telah membantu pengembangan proyek ini.

---

## License

Proyek ini dilisensikan di bawah [MIT License](LICENSE).