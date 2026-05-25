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
3. **Database**: PostgreSQL (shared dengan layanan Rust). Konfigurasi DB host/port/database via ConfigMap `simpelv1-env`; **password & APP_KEY dari Secreton** (lihat section "Secret Fetching" di bawah).
4. **Secret Fetching (Zero-Trust, saat `secretonAuth.enabled=true`)**:
   - simpelv1 tidak punya client Secreton native — pakai **init container `fetch-secrets`** yang:
     1. Auth ke Secreton via projected ServiceAccount JWT (audience `secreton`).
     2. Fetch dari `kv/simpelv1/app` (APP_KEY) dan `kv/postgres/simpelv2` (DB credentials).
     3. Render `/app/runtime/.env` (emptyDir volume yang shared dengan main container).
   - Script: [`monolith/simpelv1/fetch-secrets.sh`](fetch-secrets.sh) (di-mount via ConfigMap `simpelv1-fetch-secrets`).
   - Laravel main container baca `.env` dari `/app/runtime/.env` (env `APP_ENV_FILE`).
   - DILARANG commit `.env` real ke repo (gitignored). Hanya `.env.example` boleh tracked.
5. **APP_KEY Rotation**:
   - Generate baru via `php artisan key:generate --show` (lokal/staging dulu, validasi).
   - Push ke Secreton: `secreton kv put kv/simpelv1/app app_key=<base64-key>`.
   - Restart pod simpelv1 (`kubectl rollout restart deploy/simpelv1`) — init container fetch ulang.
   - **Hati-hati**: rotasi APP_KEY invalidate semua session existing (user logout). Lakukan di luar jam puncak.

## ⚠️ Common Pitfalls (DO & DON'T)

❌ **DON'T:**

- Menghubungkan langsung aplikasi Laravel dengan Core gRPC Rust tanpa pola K8s Sidecar/Gateway. Eksekusi mTLS RPC berulang di tiap siklus `php-fpm` dapat sangat merusak performa.
- Mengeksekusi command Cargo, Trunk, atau *tools* Rust di folder ini.
- Membocorkan *secrets* di file `.env` ke *commit* history (audit via `git log -p -- '*.env'` jika curiga; rotate semua secret kalau ditemukan).
- Pakai k8s Secret untuk APP_KEY/DB_PASSWORD saat `secretonAuth.enabled=true`. Source of truth = Secreton.

✅ **DO:**

- Gunakan perintah `composer install`, `php artisan`, dsb. hanya di dalam direktori `monolith/simpelv1` jika sedang berfokus pada Laravel backend.
- Konfigurasi *Docker build* di root harus mampu membangun modul ini secara spesifik pada image Docker yang mendukung eksekusi PHP (FPM/Octane).
- Menerapkan *Zero-Trust Security* terhadap *request* API yang ke dan dari *service* monolith ini menggunakan JWT Authentication sesuai protokol Authenc.

## 🧭 v1 ↔ v2 Boundary

Status per Mei 2026: v2 (Rust) **belum** production-ready secara
end-to-end. simpelv1 + simpelv2 jalan **paralel** di staging — user
masih mengandalkan v1 untuk fitur yang stabil sambil v2 menyusul fitur
demi fitur.

**Default**: bug yang ditemukan operator → fix di-tempatnya di
simpelv1 (PHP). Jangan menunggu rewrite ke v2; tidak ada timeline
hard-cutover.

**Modul yang TETAP di v1 (no migration scheduled)**:

| Modul | Alasan |
|-------|--------|
| Pengadaan tender | Workflow stabil, user familiar. Rebuild di v2 di-defer per keputusan Mei 2026. |
| BMN Wasdal (10 jenis: Sewa, Pinjam, KSP, BSG, BGS, dll) | Per-jenis semantics berbeda; konsolidasi ke polymorphic state-machine di v2 prematur. |
| Pakaian Dinas (modul stabil v1) | Sudah jalan baik di v1. Versi v2 (`layanan/perlengkapan/src/pakaian_dinas/`) sedang dilengkapi tapi belum jadi default user. |
| Asset CRUD (18 controller) | Mature, banyak data historis. Migrasi data masih open question. |
| Master data | Sumber kebenaran tetap di v1 sementara. |

**Modul YANG sudah ada di v2** (paralel, user dapat akses keduanya):
Kebutuhan BMN, Pemakaian BMN, Penghapusan BMN, Analitik Roadmap
Sarpras, Notifikasi, Workflow (definitions / monitoring / delegation).

**Kapan port ke v2**: hanya kalau ada keputusan eksplisit dari product
owner. Jangan port sendiri "karena rapi" — kontrak data v1 ↔ v2 belum
final (mis. lookup tabel master di v2 belum lengkap), risiko data
drift tinggi.

## 📋 Common Tasks

### 1. Fix bug di simpelv1 tanpa breaking session

Session storage Laravel v1 = Redis (`SESSION_DRIVER=redis`,
`SESSION_CONNECTION=default`). Aturan amannya:

1. **Jangan ubah `APP_KEY`** sebagai bagian dari bugfix biasa —
   rotation invalidate semua session (lihat section 5 di atas).
2. **Migrasi DB**: pakai `php artisan migrate --pretend` dulu untuk
   review SQL, lalu jalankan di staging sebelum production.
3. **Cache clear setelah deploy**: `php artisan config:clear` +
   `php artisan view:clear` + (kalau OPcache enabled) restart
   php-fpm. Tanpa cache clear, perubahan config bisa kelihatan
   tidak nyata.
4. **OPcache**: `simpelv1` Dockerfile umumnya enable OPcache
   `validate_timestamps=0` di production → kode lama "stuck" sampai
   restart container. Kalau bugfix kelihatan tidak efektif, ini
   tersangka pertama.
5. **Asset compilation**: `npm run build` (Mix/Vite) kalau perubahan
   menyangkut `resources/js` atau `resources/sass`. Asset hash baru
   wajib agar browser fetch ulang (jangan andalkan F5).

Untuk hot-fix produksi cepat: `kubectl exec` ke pod simpelv1 →
`php artisan tinker` untuk diagnosa interaktif, **tapi** semua
perubahan akhir tetap harus melalui git + helm upgrade (tidak boleh
patch in-place di pod).
