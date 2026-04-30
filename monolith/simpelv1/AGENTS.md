# 🤖 AGENTS.md - Monolith simpelv1 (Laravel) Developer Guide

> **Notice to Agents**: This file contains the domain-specific guidelines for the `monolith/simpelv1` directory, which houses the legacy/monolithic Laravel application integrated into the wider Rust-based SIMPEL ecosystem.

## 🌍 Domain Context
Direktori ini berisi kode untuk **Aplikasi Monolitik simpelv1 berbasis PHP/Laravel**. Aplikasi ini merupakan bagian dari ekosistem Kejaksaan RI dan dipertahankan dalam monorepo Rust untuk memudahkan orkestrasi, CI/CD, dan integrasi bertahap.

Aplikasi ini bertindak sebagai *web application* tradisional dan tidak boleh dicampur adukkan logikanya dengan microservices Rust di direktori `layanan/` atau framework WASM leptos di `antarmuka/`.

## 🏗️ Architecture & Roles
- **Framework**: Laravel (PHP)
- **Role**: Aplikasi Web Monolitik & Legacy Services.
- **Integration Strategy**: Deployment ke microk8s. Integrasi dengan sistem core (seperti `authenc` atau `secreton`) di-proxy-kan melalui *Gateway* (K8s Sidecar atau Rust Gateway Proxy) alih-alih melakukan panggilan gRPC secara langsung, agar performa tetap terjaga dan arsitektur tetap bersih.

## 📏 Critical Conventions (Absolute Rules)

1. **Isolation**: 
   - **TIDAK BOLEH** ada campuran *tooling* Rust (`Cargo.toml`) di dalam folder ini.
   - Folder ini dikelola menggunakan *standard tooling* PHP seperti `composer` dan ekosistem terkait Laravel (`artisan`).
2. **Ketergantungan Ekosistem**: 
   - Direktori `monolith/simpelv1` berdiri secara independen. Jangan menulis referensi dependensi PHP ke *root* Git monorepo.
3. **Database**: Menggunakan preferensi database aplikasi monolith (PostgreSQL atau MySQL/MariaDB yang ada). Pastikan environment variables / `.env` Laravel diarahkan ke persistensi yang aman (dalam _production_, dikelola via Secrets K8s).

## ⚠️ Common Pitfalls (DO & DON'T)

❌ **DON'T:**
- Menghubungkan langsung aplikasi Laravel dengan Core gRPC Rust tanpa pola K8s Sidecar/Gateway. Eksekusi mTLS RPC berulang di tiap siklus `php-fpm` dapat sangat merusak performa.
- Mengeksekusi command Cargo, Trunk, atau *tools* Rust di folder ini.
- Membocorkan *secrets* di file `.env` ke *commit* history.

✅ **DO:**
- Gunakan perintah `composer install`, `php artisan`, dsb. hanya di dalam direktori `monolith/simpelv1` jika sedang berfokus pada Laravel backend.
- Konfigurasi *Docker build* di root harus mampu membangun modul ini secara spesifik pada image Docker yang mendukung eksekusi PHP (FPM/Octane).
- Menerapkan *Zero-Trust Security* terhadap *request* API yang ke dan dari *service* monolith ini menggunakan JWT Authentication sesuai protokol Authenc.