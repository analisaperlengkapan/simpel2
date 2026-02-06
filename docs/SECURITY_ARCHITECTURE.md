# Arsitektur Keamanan & Model Pengguna (Security Architecture)

Dokumen ini menjelaskan secara teknis model pengguna (*User*), peran (*Role*), tingkatan, dan wewenang di layanan `infra/authenc` dan `infra/secreton`, serta mekanisme sinkronisasi antara keduanya.

## 1. Infra/Authenc (Identity Provider)

`infra/authenc` bertindak sebagai *Identity Provider* (IdP) utama dan *Source of Truth* untuk identitas pengguna di seluruh ekosistem SIMKARI.

### Model Pengguna (User)
Definisi pengguna di `authenc` sangat kaya dan disesuaikan dengan struktur organisasi Kejaksaan.

*   **Lokasi Kode**: `infra/authenc/src/models/user.rs`
*   **Atribut Kunci**:
    *   `id`: UUID (Identitas unik global).
    *   `username`, `email`: Kredensial login standar.
    *   `nip`: Nomor Induk Pegawai (Spesifik pemerintahan).
    *   `satker_code`: Kode Satuan Kerja (Menentukan lokasi dinas, misal: Kejaksaan Negeri Medan).
    *   `jabatan`: Posisi struktural/fungsional.
    *   `roles`: Daftar objek `Role` yang kompleks.
    *   `attributes`: JSON fleksibel untuk menyimpan kebijakan tambahan (misal: *legacy* `SecretonAccessPolicy`).

### Peran & Wewenang (Role & Authority)
Sistem *Role-Based Access Control* (RBAC) di `authenc` bersifat dinamis dan hierarkis.

*   **Lokasi Kode**: `infra/authenc/src/models/role.rs`
*   **Struktur Role**:
    *   `name`: Nama peran (misal: `staff_keuangan`).
    *   `scope`: Cakupan wewenang. Ini adalah fitur krusial untuk struktur organisasi.
        *   `pusat`: Wewenang nasional (Kejaksaan Agung).
        *   `satker:{KODE}`: Wewenang terbatas pada satu Satker.
        *   `global`: Wewenang sistem secara umum.
    *   `managed_by`: Menentukan level admin yang boleh mengelola role ini (mencegah *privilege escalation*).
    *   `permissions`: Daftar izin granular (`resource`, `action`, `scope`) yang melekat pada role.

### Tingkatan (Levels)
Tingkatan tidak lagi didefinisikan sebagai Enum statis (`AdminLevel`), melainkan diturunkan dari kombinasi `Role` dan `Scope`.
*   **Super Admin**: Memiliki role dengan scope `global` atau atribut `is_superuser`.
*   **Admin Pusat**: Role dengan scope `pusat`.
*   **Admin Satker**: Role dengan scope `satker:XYZ`.

---

## 2. Infra/Secreton (Secrets Engine / Vault)

`infra/secreton` bertindak sebagai *Resource Server* yang menyimpan rahasia (secrets). Ia tidak menyimpan database pengguna penuh, melainkan menggunakan model pengguna yang disederhanakan (*canonical user*).

### Model Pengguna (User)
Model ini lebih ringkas dan berfokus pada akses teknis.

*   **Lokasi Kode**: `infra/secreton/crates/core/src/models/user.rs`
*   **Atribut Kunci**:
    *   `id`: UUID (Sesuai dengan Authenc).
    *   `is_superuser`: Boolean (Akses root ke Vault).
    *   `roles`: `HashSet<String>` (Hanya nama role, tanpa detail scope yang kompleks).
    *   `policies`: `HashSet<String>` (Nama kebijakan akses eksplisit).
    *   `namespace`: Isolasi multi-tenant.

### Kebijakan (Policy)
Secreton menggunakan *Policy-Based Access Control* yang lebih berorientasi pada *Path* (jalur secret).

*   **Lokasi Kode**: `infra/secreton/crates/core/src/models/policy.rs`
*   **Struktur Policy**:
    *   `role`: Mengaitkan policy dengan role string dari User.
    *   `path`: Pola Glob (misal: `secret/data/satker/KJA001/*`).
    *   `action`: `read`, `write`, `list`.
    *   `effect`: `allow` atau `deny`.

---

## 3. Hubungan & Sinkronisasi (Federated Validation)

Tidak ada sinkronisasi database "fisik" (seperti replikasi tabel) antara Authenc dan Secreton. Hubungan keduanya bersifat **Stateless** dan **Just-In-Time (JIT)** melalui validasi token.

### Mekanisme Kerja
Mekanisme ini diimplementasikan di `infra/secreton/crates/core/src/auth/authenc_provider.rs`.

1.  **Login**: Pengguna login ke `authenc` dan mendapatkan **JWT Token**. Token ini berisi klaim (`sub`, `roles`, `realm_id`).
2.  **Request**: Pengguna mengirim request ke API `secreton` dengan header `Authorization: Bearer <TOKEN>`.
3.  **Validasi (Trust)**:
    *   `secreton` tidak memvalidasi signature lokal saja (opsi via `OidcVerifier`), tetapi utamanya menggunakan **Remote Validation**.
    *   Secreton memanggil endpoint `POST {authenc_url}/v1/auth/validate-token`.
4.  **Mapping (Sinkronisasi On-The-Fly)**:
    *   `authenc` merespons dengan validitas token dan **User Info** terbaru.
    *   Respons berisi: `satker_code`, `roles` (list string), dan `permissions`.
    *   `secreton` secara otomatis memetakan respons ini menjadi objek `User` dan `Claims` internal untuk durasi request tersebut.

### Detail Teknis Integrasi
*   **Authenc Provider**: Struct `AuthencAuthProvider` di Secreton bertugas melakukan panggilan HTTP ke Authenc.
*   **Caching**: Hasil validasi di-cache di memori Secreton (`TokenCache`) selama beberapa menit untuk performa, sehingga tidak setiap request memanggil Authenc.
*   **Otorisasi**:
    *   Role di Authenc (misal: "staff_keuangan") dikirim sebagai string ke Secreton.
    *   Di Secreton, harus ada Policy yang mengizinkan role "staff_keuangan" untuk mengakses path tertentu (misal: `secret/keuangan/*`).

### Kesimpulan Alur Data
*   **Source**: Semua data pengguna dan role dikelola di **Authenc**.
*   **Consumer**: **Secreton** hanya mengonsumsi data tersebut saat runtime (saat ada request).
*   **Keuntungan**: Perubahan role di Authenc (misal: pencabutan akses) berlaku hampir instan di Secreton (setelah cache expired), tanpa perlu job sinkronisasi yang kompleks.
