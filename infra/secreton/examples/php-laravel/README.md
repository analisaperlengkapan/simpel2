# Integrasi Secreton dengan Laravel

Panduan ini menjelaskan cara mengintegrasikan aplikasi PHP Laravel dengan **Secreton**, sistem manajemen rahasia (secrets management) enterprise-grade yang digunakan di lingkungan Kejaksaan Agung RI.

## Fitur Secreton

Secreton menawarkan berbagai fitur keamanan canggih yang dapat dimanfaatkan oleh aplikasi Laravel:

1.  **KV Secrets Engine (Penyimpanan Kunci-Nilai)**: Menyimpan rahasia (seperti API keys, kredensial database) secara aman, terenkripsi, dan berversi.
2.  **Environment Injection (`inject_env`)**: Fitur "killer" untuk CI/CD dan deployment. Aplikasi tidak perlu menyimpan file `.env` yang sensitif. Secreton menyuntikkan rahasia langsung sebagai variabel lingkungan (environment variables) saat runtime atau boot.
3.  **Transit Engine**: Encryption-as-a-Service. Aplikasi dapat mengenkripsi data sensitif (misal: NIK, data pribadi) tanpa menyimpan kunci enkripsi di dalam kode atau database aplikasi.
4.  **Audit Logging**: Setiap akses ke rahasia tercatat secara detail (siapa, kapan, apa).
5.  **Policy & RBAC**: Kontrol akses granular. Anda bisa menentukan user/service mana yang boleh membaca rahasia tertentu.

## Skenario Integrasi

Ada dua pendekatan utama untuk mengintegrasikan Laravel dengan Secreton:

### 1. Pendekatan "Maximal" (Bootstrapping / Environment Injection) - **Direkomendasikan**

Pada pendekatan ini, rahasia diambil **sebelum** aplikasi Laravel berjalan penuh. Skrip bootstrap akan menghubungi Secreton, mengambil rahasia, dan menyuntikkannya ke dalam `$_ENV` atau `putenv()`.

**Keuntungan:**
*   **Zero Code Change**: Kode Laravel Anda tetap menggunakan `env('DB_PASSWORD')` atau `config('app.key')` seperti biasa.
*   **Keamanan Tinggi**: Tidak ada file `.env` fisik yang berisi rahasia di server produksi.
*   **Sentralisasi**: Rotasi password database dilakukan di Secreton, dan aplikasi tinggal di-restart untuk mendapatkan nilai baru.

### 2. Pendekatan Runtime (Service Provider)

Pada pendekatan ini, Anda membuat Service Provider di Laravel yang menghubungi API Secreton saat aplikasi sedang berjalan untuk mengambil data spesifik (misalnya untuk Transit Encryption).

**Keuntungan:**
*   **Fleksibel**: Bisa melakukan enkripsi/dekripsi data transaksional (Transit Engine).
*   **Dinamis**: Bisa mengambil kredensial on-the-fly.

---

## Prasyarat

*   **Secreton Server** yang sudah berjalan.
*   **Akun Service** di Secreton (Username & Password) atau Token yang memiliki policy untuk membaca path rahasia yang dibutuhkan.
*   Ekstensi PHP: `curl`, `json`.

## Langkah Implementasi

Lihat contoh kode di direktori ini:

*   `src/SecretonClient.php`: Wrapper sederhana untuk API Secreton.
*   `src/SecretonServiceProvider.php`: Contoh integrasi Service Provider.
*   `bootstrap_secrets.php`: Contoh skrip bootstrapping untuk injection.
*   `config/secreton.php`: Contoh file konfigurasi.

### Cara Menggunakan Bootstrap Injection

1.  Salin `bootstrap_secrets.php` ke root proyek Laravel Anda.
2.  Edit `bootstrap/app.php` (Laravel 11) atau `bootstrap/autoload.php` (Laravel lama) untuk memuat skrip ini di awal.
    ```php
    // Di awal file bootstrap/app.php
    if (file_exists(__DIR__.'/../bootstrap_secrets.php')) {
        require __DIR__.'/../bootstrap_secrets.php';
    }
    ```
3.  Pastikan environment variable dasar untuk koneksi Secreton tersedia (misal dari environment container/k8s):
    ```env
    SECRETON_HOST=http://secreton.internal:8200
    SECRETON_USERNAME=laravel-app
    SECRETON_PASSWORD=very-secure-password
    SECRETON_JOB_ID=deployment-2025-01
    ```

### Cara Menggunakan Runtime Client

1.  Tambahkan `SecretonServiceProvider` ke `config/app.php` (atau biarkan auto-discovery jika dikemas sebagai package).
2.  Gunakan Facade atau Dependency Injection:
    ```php
    use App\Services\SecretonClient;

    public function index(SecretonClient $secreton) {
        // Mengambil secret
        $data = $secreton->getSecret('secret/data/myapp/payment-gateway');

        // Mengenkripsi data (Transit Engine)
        $cipher = $secreton->encrypt('transit/encrypt/myapp-key', 'Data Rahasia');
    }
    ```
