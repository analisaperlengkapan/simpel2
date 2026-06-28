# 🤖 AGENTS.md - Direktori Monolith

> **Notice to Agents**: File ini menjelaskan arsitektur makro dari direktori `/monolith/`. Direktori ini **bukan** tempat untuk microservices Rust (`layanan/`), melainkan inkubator khusus untuk menampung aplikasi legacy atau berarsitektur monolitik.

## 🌍 Domain Context

Direktori `monolith/` digunakan sebagai *namespace* untuk memisahkan aplikasi sistem eksternal, lawas (legacy), atau monolitik yang terintegrasi (atau akan diintegrasikan) ke dalam ekosistem SIMPEL.

Pemisahan ini berfungsi untuk memastikan *engineer* dan *build tools* lingkungan Rust (seperti Cargo dan Trunk) bisa mengecualikan dan memberi batasan tegas terhadap pola perancangan non-Rust ini.

## 📦 Isi Direktori

Saat ini, proyek monolitik utama yang aktif berada di:

- 🐘 `monolith/simpelv1/`: Versi pertama dari sistem web SIMPEL (dibangun menggunakan framework PHP Laravel).
- 🌐 `monolith/APP-2026/`: Halaman statis permintaan data pegawai untuk rencana pengadaan Alat Perlengkapan Personil (APP) Tahun 2026 (dua halaman: `index.html` = Kejati, `index2.html` = Kejagung). Dibangun dengan HTML/CSS/JS murni, di-serve via Nginx, diakses di path `/APP-2026`. **Image-nya mengikuti pipeline rilis yang sama dengan service lain** — di-build oleh `release.yml`/`ci.yml` dari `monolith/APP-2026/Dockerfile` (context = repo root) ke `ghcr.io/<owner>/simpelv2-app-2026` dengan tag SemVer immutable (`global.imageTag`), lalu di-deploy **staging → promote → production** via Helm (`app-2026` di `values*.yaml`). DILARANG registry lokal / tag mutable.

Untuk regulasi implementasi teknis spesifik pada proyek-proyek di dalam sub-folder ini, Anda **DIWAJIBKAN** membaca file `AGENTS.md` yang berada tepat di dalam tiap proyek (contoh: `monolith/simpelv1/AGENTS.md`).

## 📏 Aturan Integrasi Global Direktori Monolith

1. **Isolasi Penuh**: DILARANG KERAS menautkan kode, dependensi, maupun inisialisasi lingkungan (misal: referensi path symlink) yang saling merusak/mencampur antara ekosistem rust di `/layanan` dengan apa yang ada di dalam `monolith/`.
2. **Komunikasi antar Domain**: Setiap aplikasi monolith harus berkomunikasi dengan dunia luar (layanan Rust SIMPEL) secara eksklusif menggunakan jaringan (HTTP API atau Proxy gRPC) lewat K8s Service Mesh, BUKAN secara FFI atau internal *linking*.
3. **Deployment**: Deployment diurus via Dockerfile dan kubernetes secara mandiri, terlepas dari binari *Rust services*.
