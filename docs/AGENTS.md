# 🤖 AGENTS.md - Dokumentasi & ADR

> **Notice to Agents**: File ini adalah pedoman (Level 2 Archetype) untuk semua operasi terkait penulisan dokumentasi dan ADR di direktori `docs/`.

## 📑 Daftar Isi (Table of Contents)
1. 🗺️ Tujuan Direktori
2. 📖 Standar Penulisan Dokumentasi
3. ⚠️ Aturan AI untuk Penulisan Dokumen

## 🗺️ Tujuan Direktori `docs/`
Direktori ini berfungsi sebagai pusat pengetahuan (*knowledge base*) statis proyek.
1. **Architecture Decision Records (ADR)**: Dokumentasi keputusan teknis penting.
2. **Pedoman Pengguna (User Guides)**: Dokumentasi cara menggunakan aplikasi.
3. **Desain Teknis (Technical Specs)**: Rincian desain API, skema database, atau alur bisnis.

*(Untuk aturan pemrograman AI/Developer, referensinya ada di `AGENTS.md` pada masing-masing direktori layanan, BUKAN di sini).*

## 📖 Standar Penulisan Dokumentasi

### 1. Format dan Standar Bahasa
- Gunakan standar Markdown (GitHub Flavored Markdown).
- Dokumentasi ditulis dalam **Bahasa Indonesia** yang baku dan profesional, kecuali istilah teknis umum (*database*, *deployment*, *cache*).
- Gunakan [Mermaid.js](https://mermaid-js.github.io/) untuk diagram arsitektur.

### 2. Architecture Decision Records (ADR)
Setiap keputusan arsitektural besar WAJIB didokumentasikan di folder `docs/adr/`.
Format: **Konteks** → **Keputusan** → **Konsekuensi**.

### 3. Integritas Informasi
- Jangan *copy-paste* blok kode panjang ke dokumentasi (cepat usang). Gunakan tautan referensi.
- Pastikan diagram arsitektur selaras dengan `AGENTS.md` root.

## ⚠️ Aturan AI untuk Penulisan Dokumen
❌ **DON'T:**
- Jangan "berhalusinasi" menciptakan arsitektur atau tabel database yang tidak ada di repositori.
- Jangan gunakan format selain Markdown (`.md`) kecuali diminta spesifik.

✅ **DO:**
- Gunakan tautan internal relatif yang akurat.
- Pastikan setiap dokumen memiliki judul (`# Judul`) dan TOC jika terlalu panjang.
