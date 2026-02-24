# 🤖 AGENTS.md - Panduan `lib-common`

> **Domain Konteks**: Utilitas Lintas Domain. Kode di sini harus **Agnostik** terhadap proses bisnis (seperti Keuangan, BMN, dll). Jika kode hanya berlaku untuk "Barang", letakkan di tempat lain!

---

## 🏗️ Topologi Fitur Cargo

Sebagai AI Agent, wajib hukumnya untuk memahami bagaimana dependensi disusun di `Cargo.toml`:

```mermaid
graph TD
    Base[Core Rust / Agnostic]
    Backend[Fitur "backend": tokio, axum]
    Crypto[Fitur "crypto", "encryption": dalek, aes, argon]

    Backend --> DB[Fitur "db": tokio-postgres]
    Backend --> Redis[Fitur "redis-cache": redis]
    Backend --> JWT[Fitur "jwt": jsonwebtoken]
    Backend --> GRPC[Fitur "grpc": tonic]

    Base --> Backend
    Base --> Crypto

    classDef wasm fill:#0284c7,color:#fff,stroke:#0369a1;
    classDef server fill:#b91c1c,color:#fff,stroke:#991b1b;

    class Base,Crypto wasm;
    class Backend,DB,Redis,JWT,GRPC server;
```

**Keterangan**:
- Kotak Biru (`wasm`): Bisa digunakan di Frontend (WASM).
- Kotak Merah (`server`): Hanya untuk Layanan Backend, karena WASM tidak dapat mengeksekusi I/O sistem (seperti soket memori `tokio`).

---

## 🚨 Instruksi Modifikasi untuk AI

### 1. Menjaga Kompabilitas WebAssembly (WASM)
Jika Anda ditugaskan memperbaiki masalah atau fungsi di `lib-common`:
- PASTIKAN tipe data yang diekspor kompatibel dengan WASM.
- **DILARANG** menambahkan dependensi berat yang bisa membengkakkan ukuran binari eksekutabel frontend kecuali disembunyikan di balik *feature flags*.
- Setiap penambahan *middleware* harus dibungkus dengan makro:
  ```rust
  #[cfg(feature = "backend")]
  pub async fn my_axum_interceptor(...) { ... }
  ```

### 2. Modul Autentikasi (`auth.rs`, `jwt.rs`)
Autentikasi di proyek ini berpusat di layanan IAM Kustom (`Authenc`).
- `lib-common` HANYA menyimpan rutinitas *parsing JWT* primitif atau utilitas penyimpulan klaim (claims inference).
- Segala bentuk autentikasi dan komunikasi mTLS dengan `Authenc` dan `Secreton` wajib dijalin lewat antarmuka gRPC (Lihat `lib-common/grpc/`).

### 3. Utilitas Kriptografi (`crypto/`)
Murni menggunakan algoritme paling termutakhir dan disetujui (Ed25519 untuk penandatanganan, AES-GCM-256 / ChaCha20Poly1305 untuk enkripsi sirkular). Jangan mengimpor `RSA` klasik tanpa instruksi eksplisit via `Secreton`.
