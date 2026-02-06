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
    *   `attributes`: JSON fleksibel untuk menyimpan kebijakan tambahan (misal: *legacy* `SecretonAccessPolicy` atau preferensi aplikasi spesifik).
    *   `realm_id`: Mendukung multi-tenancy (lihat bagian Multi-Tenancy).

### Peran & Wewenang (Role & Authority)
Sistem *Role-Based Access Control* (RBAC) di `authenc` bersifat dinamis dan hierarkis.

*   **Lokasi Kode**: `infra/authenc/src/models/role.rs`
*   **Struktur Role**:
    *   `name`: Nama peran (misal: `staff_keuangan`, `admin_wilayah`).
    *   `scope`: Cakupan wewenang. Mendukung struktur hierarkis Kejaksaan:
        *   `global`: Wewenang sistem secara umum (Super Admin).
        *   `pusat`: Wewenang nasional (Kejaksaan Agung/Eselon I).
        *   `wilayah:{KODE}`: Wewenang tingkat regional (Kejaksaan Tinggi). Mencakup semua satker di bawah wilayah tersebut.
        *   `satker:{KODE}`: Wewenang terbatas pada satu Satker (Kejaksaan Negeri/Cabjari).
    *   `managed_by`: Menentukan level admin yang boleh mengelola role ini (mencegah *privilege escalation*).
    *   `permissions`: Daftar izin granular (`resource`, `action`, `scope`) yang melekat pada role.

### Tingkatan (Levels) & Fleksibilitas
Tingkatan (*Level*) diturunkan secara dinamis dari kombinasi `Role` dan `Scope` dalam token, bukan *hardcoded* enum.
*   **Fleksibilitas Custom**: Atribut `attributes` (JSONB) pada `User` dan `Role` memungkinkan penambahan metadata kebijakan kustom tanpa mengubah skema database, mengakomodasi kebutuhan bisnis yang dinamis.

---

## 2. Infra/Secreton (Secrets Engine / Vault)

`infra/secreton` bertindak sebagai *Resource Server* yang menyimpan rahasia (secrets). Ia tidak menyimpan database pengguna penuh, melainkan menggunakan model pengguna yang disederhanakan (*canonical user*).

### Model Pengguna (User)
Model ini lebih ringkas dan berfokus pada akses teknis.

*   **Lokasi Kode**: `infra/secreton/crates/core/src/models/user.rs`
*   **Atribut Kunci**:
    *   `id`: UUID (Sesuai dengan Authenc).
    *   `is_superuser`: Boolean (Akses root ke Vault).
    *   `roles`: `HashSet<String>` (Hanya nama role).
    *   `policies`: `HashSet<String>` (Nama kebijakan akses eksplisit).
    *   `namespace`: Isolasi multi-tenant (Sesuai dengan `realm_id` di Authenc).

### Kebijakan (Policy)
Secreton menggunakan *Policy-Based Access Control* yang lebih berorientasi pada *Path* (jalur secret).

*   **Lokasi Kode**: `infra/secreton/crates/core/src/models/policy.rs`
*   **Struktur Policy**:
    *   `role`: Mengaitkan policy dengan role string dari User.
    *   `path`: Pola Glob (misal: `secret/data/satker/KJA001/*`).
    *   `action`: `read`, `write`, `list`.
    *   `effect`: `allow` atau `deny`.

---

## 3. Multi-Tenancy & Isolation (Realm & Namespace)

Untuk memastikan fleksibilitas pengaturan role dan isolasi data yang baik (Best Practice), sistem ini mengadopsi konsep:
*   **Authenc: Realm**: Mengelompokkan user, role, dan konfigurasi authentication dalam domain terisolasi (misal: `internal-kejaksaan`, `vendor-eksternal`).
*   **Secreton: Namespace**: Memisahkan secrets dan policy dalam ruang nama berbeda.
*   **Rekomendasi**: Penggunaan Realm dan Namespace sangat disarankan untuk mencegah kebocoran akses antar konteks yang berbeda, memberikan fleksibilitas maksimal dalam manajemen kewenangan.

---

## 4. Standar Integrasi & Sinkronisasi

Sistem mengadopsi pendekatan **Stateless** dan **Just-In-Time (JIT)**, serta standar komunikasi modern.

### Protokol Komunikasi (gRPC vs HTTP)
*   **Antar Backend (Service-to-Service)**: Komunikasi antara `authenc`, `secreton`, dan layanan lain (`layanan/perlengkapan`, `layanan/integrasi`) diutamakan menggunakan **gRPC** (Protobuf) untuk performa tinggi, *type-safety*, dan latensi rendah.
*   **Token Validation**: Endpoint validasi token juga dapat diakses via gRPC untuk efisiensi tinggi dalam volume request besar.

### Mekanisme Sinkronisasi (Federated Validation)
Mekanisme ini diimplementasikan di `infra/secreton/crates/core/src/auth/authenc_provider.rs`.

1.  **Login**: Pengguna login ke `authenc` -> JWT Token (berisi `sub`, `roles`, `realm_id`, `scope`).
2.  **Request**: API Call ke `secreton` dengan header `Authorization: Bearer <TOKEN>`.
3.  **Trust & Mapping**:
    *   `secreton` memvalidasi token ke `authenc` (via gRPC/HTTP).
    *   `authenc` mengembalikan User Info terkini (termasuk update role/scope yang baru diubah).
    *   `secreton` memetakan data tersebut ke sesi internal memori (`TokenCache`).

### Kesimpulan Alur Data
*   **Source of Truth**: `infra/authenc`.
*   **JIT Consumer**: `infra/secreton` dan layanan mikro lainnya.
*   **Keuntungan**: Sinkronisasi instan, konsistensi data terjamin, dan overhead manajemen data yang minimal.
