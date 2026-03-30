# Refactoring Portal, SSO, dan IAM Integration

Goal: Melakukan refaktor total pada `antarmuka/portal` agar sesuai standar dan best practice, memastikan integrasi Single Sign-On (SSO) dan IAM dengan `authenc`, serta mengintegrasikan data `satker` dan `pegawai` secara penuh dari `layanan-integrasi`.

## Flow Diagrams

### 1. SSO Login, MFA, & Activation Flow (Comprehensive)
```mermaid
sequenceDiagram
    participant User
    participant Portal (Frontend)
    participant Authenc (IAM Backend)
    participant Integrasi (Backend)
    participant MySIMKARI (External API)

    %% Sinkronisasi Data Pegawai Background
    loop Background Sync
        Integrasi->>MySIMKARI: Fetch Pegawai & Satker Data
        MySIMKARI-->>Integrasi: Transformed Data (NIP, Nama, Jabatan, Satker)
        Integrasi-->>Integrasi: Upsert to DB `mysimkari_pegawai`
    end

    %% Login Initialization
    User->>Portal (Frontend): Akses Halaman Home (`/home`) -> Login (`/login`)
    User->>Portal (Frontend): Lengkapi Verifikasi Keamanan CAPTCHA
    User->>Portal (Frontend): Input NIP & Password (Atau Passkeys)
    Portal (Frontend)->>Authenc (IAM Backend): POST /api/v1/auth/login (Dengan Token CAPTCHA)

    %% Registration & Activation Checks
    Authenc (IAM Backend)->>Integrasi (Backend): gRPC GetMysimkariPegawai(NIP)
    Integrasi (Backend)-->>Authenc (IAM Backend): Data Pegawai

    alt Akun Belum Terdaftar di IAM
        Authenc (IAM Backend)->>Authenc (IAM Backend): Auto-provision (Config: User=NIP, Pass=NIP, Enabled=false)
        Authenc (IAM Backend)-->>Portal (Frontend): Gagal (Akun Belum Aktif)
        Portal (Frontend)-->>User: "Akun Anda belum aktif. Hubungi Admin."
    else Akun Terdaftar Tapi Nonaktif
        Authenc (IAM Backend)-->>Portal (Frontend): Gagal (Akun Belum Aktif)
        Portal (Frontend)-->>User: "Akun Anda belum aktif. Hubungi Admin."
    else Akun Aktif & Kredensial Valid
        Authenc (IAM Backend)->>Authenc (IAM Backend): Cek Persyaratan MFA

        alt MFA Diperlukan tapi Belum Setup
            Authenc (IAM Backend)-->>Portal (Frontend): Response (Require MFA Setup + Temp Token)
            Portal (Frontend)-->>User: Redirect ke MFA Setup (`/mfa/setup`)
            User->>Portal (Frontend): Scan QR, Input OTP
            Portal (Frontend)->>Authenc (IAM Backend): Verifikasi Setup (/auth/mfa/verify-setup)
            Authenc (IAM Backend)-->>Portal (Frontend): Sukses + Temp Kode Backup
        else MFA Sudah Setup (Standard Login)
            Authenc (IAM Backend)-->>Portal (Frontend): Response (Require OTP + Temp Token)
            Portal (Frontend)-->>User: Redirect ke MFA Verify (`/mfa/verify`)

            alt User Input OTP (Authenticator App)
                User->>Portal (Frontend): Input 6-digit OTP
                Portal (Frontend)->>Authenc (IAM Backend): POST /auth/mfa/verify
            else User Pilih Backup Code
                User->>Portal (Frontend): Pilih "Gunakan Kode Cadangan" -> Redirect (`/mfa/backup-verify`)
                User->>Portal (Frontend): Input 8-digit Backup Code
                Portal (Frontend)->>Authenc (IAM Backend): POST /auth/mfa/verify-recovery
            end

            Authenc (IAM Backend)-->>Portal (Frontend): Sukses (JWT Access Token)
        else Autentikasi WebAuthn / Passkeys
            Authenc (IAM Backend)-->>Portal (Frontend): Passkeys Challenge
            Portal (Frontend)-->>User: Prompt Biometrik / Hardware Key
            User->>Portal (Frontend): Approve Challenge
            Portal (Frontend)->>Authenc (IAM Backend): Passkeys Response Validation
            Authenc (IAM Backend)-->>Portal (Frontend): Sukses (JWT Access Token)
        end

        %% Finalization
        Portal (Frontend)->>Portal (Frontend): Simpan Session (localStorage via `auth.rs`)
        Portal (Frontend)-->>User: Redirect ke Beranda Portal (`/dashboard`)
    end
```

### 2. Portal App Navigation Flow (Pages Exhaustive Mapping)
```mermaid
graph TD
    %% Public Access Pages
    Home[Landing Page `/home`] --> Lgn(Login Page `/login`)
    OAuth[OAuth Callback `/callback`] --> Lgn
    Lgn -->|Sukses Autentikasi| Dash(Dashboard Utama `/dashboard`)

    %% Interceptor / MFA Routing
    Lgn -.->|Butuh Setup MFA| MfaS[MFA Setup `/mfa/setup`]
    Lgn -.->|Butuh Verifikasi| MfaV[MFA Verifikasi `/mfa/verify`]
    MfaV -.->|Lupa OTP| MfaB[MFA Backup `/mfa/backup-verify`]
    MfaS --> Dash
    MfaV --> Dash
    MfaB --> Dash

    %% Main Application Layout
    Dash --> App1[Aplikasi Perlengkapan `/apps/perlengkapan`]
    Dash --> Admin[Admin Portal IAM `/admin/*`]

    %% Self-Service & Profiling Sub-pages
    Dash --> Profile[Profil Pengguna `/profile`]
    Dash --> Settings[Pengaturan Tampilan `/settings`]
    Dash --> Pwd[Ubah Kata Sandi `/password-change`]
    Dash --> PK[Manajemen Passkeys `/passkeys`]
    Dash --> Sess[Manajemen Sesi `/sessions`]
    Dash --> BCGen[Generate Backup Codes `/mfa/backup-codes`]

    %% Logout Flow
    Dash --> LogoutAct([Action: Klik Logout])
    Profile --> LogoutAct
    LogoutAct --> LoggedOut[Halaman Logged Out `/logged-out`]
    LoggedOut --> Home

    %% Universal Not Found
    AnyPage((Halaman Apapun)) -.->|URL Salah| NotFound[404 Not Found `/not-found`]

    %% Halaman yang Dihapus / Refactor Out
    Dash -.-x D1[Pembinaan `/pembinaan`]
    Dash -.-x D2[Monitoring `/monitoring`]
    Dash -.-x D3[App Mockup: PIDSUS, PIDUM, dll]
```

---

## Proposed Changes

### 1. Refactor Portal (`antarmuka/portal/src/pages/*`)
> [!NOTE]
> Halaman statis atau fitur prototipe yang tidak memiliki back-end terintegrasi akan dihapus, sedangkan halaman core IAM Identity dan Pengaturan Mandiri Pengguna (Self-Service) akan dipertahankan dengan refinement.

- **[DELETE] Halaman Statis/Halu**:
  - `src/pages/pembinaan.rs`
  - `src/pages/monitoring.rs`
  - *List microfrontends (Apps) halu seperti PIDUM, PIDSUS, dll pada `src/features/microfrontends.rs` dihapus, menyisakan Perlengkapan dan Admin*.
- **[RETAIN & MODIFY] Core Login & Landing**:
  - `src/pages/home.rs`: Halaman landing page tetap dipertahankan dengan UI resmi Kejaksaan RI.
  - `src/pages/login.rs`: Tetap dipertahankan, namun UX-nya disesuaikan untuk lebih eksplisit meminta NIP. Mendukung redireksi untuk status Akun Non-aktif. **Sudah terintegrasi dengan verifikasi keamanan CAPTCHA yang wajib (mandatory) sebelum request login dikirim.**
  - `src/pages/logged_out.rs`: Dipertahankan untuk transisi Sign-Out.
  - `src/pages/not_found.rs`: Dipertahankan sebagai catch-all fallback 404 router.
  - `src/pages/callback.rs`: Dipertahankan untuk delegasi flow otentikasi OAuth2 internal (jika SSO di-switch on).
- **[RETAIN & MODIFY] Flow MFA (Multi-Factor Auth)**:
  - `src/pages/mfa_setup.rs` & `src/pages/mfa_verification.rs`: Dipertahankan, mendukung logic token temporary session backend dengan flow interceptor login.
  - `src/pages/mfa_backup_codes.rs` & `src/pages/mfa_backup_verification.rs`: Dipertahankan, memfasilitasi pengguna yang app Authenticator-nya hilang.
- **[RETAIN & MODIFY] Self-Service / Profiling Pengguna**:
  - `src/pages/passkeys.rs`: Dipertahankan sebagai front-end untuk WebAuthn (Login Sidik Jari/Wajah).
  - `src/pages/password_change.rs`: Dipertahankan untuk user mengganti password NIP default sistem mereka.
  - `src/pages/profile.rs`, `src/pages/sessions.rs`, `src/pages/settings.rs`: Dipertahankan dan dipastikan sinkron dengan user claim yang baru dari database layanan integrasi (Nama, Satker, Jabatan).
- **[MODIFY] Dashboard Utama**:
  - `src/pages/dashboard.rs`: Dihapus koneksinya ke widget Monitoring mock. Ganti dengan widgets/summary valid (misal: Sesi Login Aktif terakhir, atau Info Data Satker).

### 2. Seeding Admin Authenc & Aktivasi Akun
> [!IMPORTANT]
> Admin akan mendapat Default Password: `199203142014031001` untuk saat ini sesuai request. Akun yang auto-provison akan dibuat **nonaktif** (`enabled: false`), dan harus diaktivasi via manajemen admin.

- **[MODIFY]** `layanan/authenc/crates/core/src/init/database.rs` (atau file startup yang ekuivalen):
  - Tambahkan skrip seeding saat backend startup untuk mengecek keberadaan admin `199203142014031001`.
  - Jika belum ada, lakukan `INSERT` otomatis ke tabel user IAM default sistem dengan Role Admin dan status `enabled: true`.

### 3. Integrasi gRPC ke Layanan Integrasi
Mengoptimalkan data di dalam Authenc berdasar tarikan gRPC.
- **[MODIFY]** `layanan/authenc/crates/federation/src/mysimkari_sync.rs`:
  - Hapus stub mock data. Panggil gRPC Client.
  - Ketika token digenerate dan data user belum ada / butuh sync ulang:
    - Tarik data `nip`, `nama`, `satker_id`, `jabatan`, dan `status_pegawai` menggunakan endpoint gRPC `GetMysimkariPegawai`.
    - Simpan info ini ke dalam Database / Claims user.
    - Set default user `enabled = false` saat pertama kali di-*upsert*.
- **[MODIFY]** `layanan/authenc/crates/core/src/config/mod.rs`:
  - Inject environment parameter untuk host `layanan-integrasi-grpc:50051`.

### 4. Aktivasi User di Portal Admin (`antarmuka/portal/src/pages/admin/users.rs`)
- **[MODIFY]** `src/pages/admin/users.rs`:
  - Fitur *Aksi* -> *Aktifkan* / *Nonaktifkan* sudah/sedang diimplementasikan untuk trigger ke endpoint API IAM Authenc (`PUT /api/v1/iam/users/{id}`).
  - Halaman ini juga perlu menampung data pencarian baru (berdasarkan *username, email, nip, nama*) dan *pagination* yang telah ditambahkan pada *backend store*.
  - Pastikan User Experience (UX) menekankan bahwa user baru dari sinkronisasi MySIMKARI akan muncul dengan status **Nonaktif** dan membutuhkan *approval* klik **Aktifkan** oleh Admin untuk mengizinkan login.

## Verification Plan

### Automated Tests
- Menjalankan testing `authenc` untuk memastikan akun yang di-*provisioning* otomatis terekam dengan `enabled: false`.
- Eksekusi cargo check di folder portal setelah komponen mock dihapus.

### Manual Verification
- Jalankan portal secara lokal (termasuk db, redis, layanan-integrasi).
- Coba login dengan NIP baru (Gagal - butuh aktivasi).
- Login dengan akun admin default `199203142014031001` dan passwordnya.
- Aktifkan akun test NIP lewat UI Admin Portal.
- Login dengan NIP test, verifikasi berhasil diarahkan ke Dashboard.
