# 🤖 AGENTS.md - Panduan Komponen Antarmuka (`lib-ui`)

> **Domain Konteks**: Leptos WebAssembly (WASM) UI Component Library

---

## 🖼️ Kerangka Kerja UI (The AI Directives)

Gunakan pustaka independen UI bawaan ini untuk Microfrontends SIMPEL. **DILARANG KERAS** menulis spesifikasi UI/Komponen dari awal jika komponen Atom/Molekulnya sudah terdaftar di sini!

```mermaid
stateDiagram-v2
    [*] --> Periksa_Komponen_Ada
    Periksa_Komponen_Ada --> Re-Use_Komponen : Ada
    Periksa_Komponen_Ada --> Rancang_Komponen_Baru_Agnostik : Tidak Ada
    Rancang_Komponen_Baru_Agnostik --> Tambahkan_Props_Reaktif
    Tambahkan_Props_Reaktif --> Implementasi_di_Aplikasi
```

## 🛠 Aturan Pengembangan (Design System)

1. **Agnostik Bisnis Seisinya! (Domain-Agnostic)**
   - Saat mendesain elemen tabel, namakan `DataTable` atau `Pagination`, **bukan** `TabelBarangBMN`. Letakkan abstraksi tabel *TabelBarangBMN* di proyek `antarmuka/perlengkapan/`, bukan di sini.

2. **Gunakan Sinyal (*Signals*) dengan Hati-Hati!**
   - Dalam Leptos v0.8.x, kita menggunakan `signal()` dan tipe reaktif pasif lainnya, BUKAN `create_signal()` iterasi lama yang memicu mutabilitas manual kompleks. Ikuti pola reaktivitas modern.

3. **Komponen Bersih tanpa Efek Samping (Side-Effects)**
   - Komponen UI sebisa mungkin tidak menyimpan integrasi URL langsung ke *backend* menggunakan HTTP Client/Gloo. Komponen UI itu *"bodoh - dumb components"* dan meminta properti seperti `on_submit` atau parameter *props value* dari penginduknya.

4. **Kaidah Modular CSS Styling**
   - Di `lib-ui`, gunakan *Utility-Classes* (Tailwind-style) yang terdaftar di konfigurasi proyek. Hindari menulis gaya *inline* seperti `style="color: red"`.

5. **Pengepakan WASM**
   - `lib-ui` akan ditautkan dan dikompilasi secara dinamis (*cdylib*/ *rlib*). Setiap penambahan pustaka Cargo pihak ketiga yang Anda berikan harus kompatibel dengan OS arsitektur WASM (`wasm32-unknown-unknown`).
