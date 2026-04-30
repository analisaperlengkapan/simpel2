# 🤖 AGENTS.md - Direktori Monolith

> **Notice to Agents**: File ini menjelaskan arsitektur makro dari direktori `/monolith/`. Direktori ini **bukan** tempat untuk microservices Rust (`layanan/`), melainkan inkubator khusus untuk menampung aplikasi legacy atau berarsitektur monolitik.

## 🌍 Domain Context
Direktori `monolith/` digunakan sebagai *namespace* untuk memisahkan aplikasi sistem eksternal, lawas (legacy), atau monolitik yang terintegrasi (atau akan diintegrasikan) ke dalam ekosistem SIMPEL.

Pemisahan ini berfungsi untuk memastikan *engineer* dan *build tools* lingkungan Rust (seperti Cargo dan Trunk) bisa mengecualikan dan memberi batasan tegas terhadap pola perancangan non-Rust ini.

## 📦 Isi Direktori
Saat ini, proyek monolitik utama yang aktif berada di:
- 🐘 `monolith/simpelv1/`: Versi pertama dari sistem web SIMPEL (dibangun menggunakan framework PHP Laravel).

Untuk regulasi implementasi teknis spesifik pada proyek-proyek di dalam sub-folder ini, Anda **DIWAJIBKAN** membaca file `AGENTS.md` yang berada tepat di dalam tiap proyek (contoh: `monolith/simpelv1/AGENTS.md`).

## 📏 Aturan Integrasi Global Direktori Monolith
1. **Isolasi Penuh**: DILARANG KERAS menautkan kode, dependensi, maupun inisialisasi lingkungan (misal: referensi path symlink) yang saling merusak/mencampur antara ekosistem rust di `/layanan` dengan apa yang ada di dalam `monolith/`.
2. **Komunikasi antar Domain**: Setiap aplikasi monolith harus berkomunikasi dengan dunia luar (layanan Rust SIMPEL) secara eksklusif menggunakan jaringan (HTTP API atau Proxy gRPC) lewat K8s Service Mesh, BUKAN secara FFI atau internal *linking*.
3. **Deployment**: Deployment diurus via Dockerfile dan kubernetes secara mandiri, terlepas dari binari *Rust services*.