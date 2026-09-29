# Audit PR #930 — `ci/hang-proof-dependency-checks` (2026-09-29)

Laporan ini adalah audit kritis atas PR #930 beserta seluruh modul yang disentuh dan
yang bertetangga dengannya, ditulis apa adanya. Bagian **Status penanganan** di bawah
mencatat, temuan demi temuan, apa yang **sudah diperbaiki di branch ini**, apa yang
**sengaja dibuat opt-in (default OFF)**, dan apa yang **belum dikerjakan** beserta
alasannya. Bagian temuan sesudahnya (A–H) menggambarkan kondisi **saat audit**, sebelum
perbaikan — jangan dibaca sebagai kondisi kode saat ini.

## Batas audit (baca dulu)

- **Dalam**: seluruh route perlengkapan (matriks statis 173 route + ±25 handler dibaca),
  authenc IAM/auth/OAuth, wrapper CI dependency-check, portal & perlengkapan FE,
  Helm/Istio/nginx.
- **Dangkal**: `layanan/integrasi` (gRPC tanpa interceptor; bergantung pada mTLS) dan
  `layanan/secreton` (204 route, 14 crate — hanya survei arsitektur). Keduanya butuh audit
  tersendiri; ketiadaan temuan di sana **bukan** jaminan.
- Tidak dibaca baris-per-baris: ±55 ribu LOC perlengkapan.
- **Ambang nilai persetujuan Pengelola Barang** (PMK penghapusan/RKBMN) tidak berhasil
  diverifikasi; harus dikonfirmasi ke DJKN/Biro Perlengkapan sebelum dijadikan aturan sistem.
- Bukti: **[T]** dijalankan · **[K]** pembacaan kode · **[S]** screenshot.

## Status penanganan

Legenda: ✅ diperbaiki di branch ini · 🟡 sebagian / opt-in default OFF · ⏳ belum dikerjakan ·
🚫 keputusan produk: dibiarkan.

### CI (bagian C)

| # | Status | Catatan |
|---|--------|---------|
| C1 | ✅ | Pola jaringan dipersempit; kegagalan deterministik tak lagi dihitung HANG. Self-test 37 kasus (regresi false-green + kontrol positif). |
| C2 | ✅ | Kegagalan sertifikat/TLS = **merah** (integritas, bukan ketersediaan). |
| C3 | ✅ | Job terjadwal fail-closed (`DEP_CHECK_HANG_IS_RED`); PR/push tetap hijau + `::warning`. `cargo-audit` dan Verus kini diverifikasi SHA-256 (trust-on-first-use, sama dengan `cargo-deny`); satu installer bersama untuk `security.yml` dan `maintenance.yml`. |
| C4 | ✅ | Repair loop: `git fetch origin "$GITHUB_SHA"` eksplisit + pengecekan checkout sebelum `exit 0`. |
| C5 | ✅ | Self-test memuat kasus false-green C1/C2. |

### Keamanan & RBAC (bagian A)

| # | Status | Catatan |
|---|--------|---------|
| A1 | ✅ | IP diparse ke `IpAddr` sebelum masuk kolom `inet`; nilai tak valid tidak lagi menggugurkan INSERT audit. |
| A2 | ✅ | `lib_backend::client_ip`: hanya hop dari proxy tepercaya (`TRUSTED_PROXY_CIDRS`), dibaca dari kanan. |
| A3 | ✅ | Token `mfa_pending` ditolak di validate/introspect/gRPC `ValidateToken`/guard IAM. |
| A4 | ✅ | Konsol IAM = role `admin` persis. Admin terakhir tak bisa dicabut/dinonaktifkan; sesi dicabut saat role dicabut. Tidak ada guard eskalasi terpisah pada `assign_role`: karena hanya `admin` yang dapat memanggilnya, ia tidak lagi meningkatkan hak siapa pun di atas `admin`. |
| A5 | ✅ | Backend memakai himpunan role dari token (`RoleSet`), pemuatan role deterministik. |
| A6 | ✅ | Aksi admin (user/role/MFA/password) diaudit dengan aktor, target, perubahan; label UI diselaraskan ke nama event yang benar. |
| A7 | ✅ | Kode pemulihan MFA butuh TOTP saat ini (step-up); portal meminta kodenya. `totp/disable` **belum** step-up (lihat ⏳). |
| A8 | ✅ | Semua daftar role-literal dimigrasi ke `lib_core::authz`; CI menolak yang baru (`check-authz-guards.py`, dengan `--self-test`). |
| A9 | 🟡 | Kapabilitas aditif; `View` = punya ≥1 role; `approver_satker`/`validator_satker` masuk katalog. `require_permission` tetap tidak dipasang di semua route — matriks route ditegakkan lewat guard CI dan tes BFLA. |
| A10 | 🟡 | Halaman Peran perlengkapan diselaraskan dengan role satker & label. Tabel DB `role_permissions`/`capabilities` **masih tidak ditegakkan siapa pun** (⏳). |
| A11 | 🟡 | Klaim `groups` (RFC 9068) dari hirarki satker + role aktif per sesi (DSD) — keduanya di balik flag, default OFF. Realm/group CRUD **tidak** dipulihkan. |
| A12 | ✅ | Tombol validator tak lagi menampilkan admin; override → `validator_pusat`. |
| A13 | ⏳ | Seed `002_seed.sql` (user NIP dengan role admin) **tidak disentuh**; perlu dipastikan dirotasi/dihapus di produksi oleh operator. |

### Tampilan (bagian B, H)

| # | Status | Catatan |
|---|--------|---------|
| B1 | ✅ | `primary` = navy di kedua config Tailwind; guard shade kini gagal untuk token yang dipakai tapi tak didefinisikan. |
| B2 | ⏳ | Halaman MFA portal masih berbahasa Inggris; kolom NAMA daftar pengguna belum diperiksa ulang. |
| B3 | ⏳ | Kolom Detail audit masih menyembunyikan `request_id`/`user_agent`/`suspicious_indicators`. |
| B4 | 🟡 | CSP `script-src` berbasis hash tersedia (`CSP_HASH_MODE`), default OFF; `style-src 'unsafe-inline'` dan `connect-src https:` tetap; salinan CSP Helm vs nginx in-image belum disatukan. |
| B5 | ⏳ | `nginx.conf` masih dimiliki uid nginx (untuk `sed -i` entrypoint). |
| H (kontras) | 🟡 | Token yang dideklarasikan dinaikkan (`--text-dim` 3,5→≥4,5:1; `--state-danger`; ring fokus portal dua-warna 2,2→≥3:1) dan dijaga CI (`check-color-contrast.py`, `--self-test`). Kelas utilitas seperti `text-gray-400` di latar putih (≈700 pemakaian tersebar) **tidak** diubah. Glyph tofu dan galeri screenshot **tidak** diambil ulang. |

### Otorisasi perlengkapan (bagian E) dan desain (bagian F)

| # | Status | Catatan |
|---|--------|---------|
| E1–E3 | ✅ | Jalur pakaian dinas & kebutuhan BMN digerbangi per role + scope satker; dibuktikan matriks route (CI) dan tes BFLA/BOLA (20 kasus). |
| E4 | 🟡 | Ekspor dibatasi per satker + kepemilikan job. Ekspor roadmap/riwayat **tidak dapat** dibatasi per satker (hanya lintas satker). |
| E5 | 🟡 | Endpoint analisis dibatasi per satker. `search`/`suggestions` tingkat kampanye diperlakukan sebagai data nasional — belum dipilah. |
| E6 | ✅ | `PemakaianBmnPolicy` dipanggil di semua jalur; maker-checker (pengaju ≠ penyetuju). |
| E7 | ✅ | Jalur generik `/pemakaian-bmn/{id}/transition` kini hanya melayani *submit* dan *cancel*, masing-masing diperiksa policy yang sama dengan jalur khusus; sisanya ditolak (400) agar tak melewati versi optimistic-lock, catatan wajib, dan stempel maker-checker. Catatan: engine masih membolehkan state tanpa `required_role` (mis. ACTIVE/EXPIRED/CANCELLED) — aman hanya karena jalur ke sana ditutup di handler. |
| E8 | ✅ | `auto-expire` butuh kapabilitas `Administer`; scheduler berjalan in-process. |
| E9 | 🟡 | Delegasi divalidasi (delegator harus memegang role, penerima harus ada) dan API/UI menyatakan **belum berlaku**. Delegasi **tetap tidak memberi kewenangan** apa pun. |
| E10 | 🟡 | Kewenangan penetap SK diturunkan server dari nilai perolehan (fail-closed), lookup SIMAN dibatasi scope. Batas nilai itu adalah konfigurasi yang **belum diverifikasi** terhadap regulasi. Kolom baru (Pengelola Barang, BA pemusnahan) **tidak** ditambahkan. |
| E11 | 🟡 | Validasi unggahan (allowlist tipe + magic byte + ukuran, gerbang role). TTL 1 tahun pada URL dokumen yang tersimpan **belum** diubah. |
| E12 | ⏳ | `/dashboard/ws` tidak diubah (latent: belum ada producer). |
| Rate limiter | ✅ | Kini benar-benar per-user (auth layer menyisipkan `Claims`). |
| F1 | 🟡 | PVC dokumen tersedia (`layananPerlengkapan.persistence`), default OFF. Selama OFF, dokumen tetap di emptyDir. |
| F2–F7 | ⏳ | Validasi token lokal/JWKS, satu sumber workflow, penyatuan kosakata otorisasi (DB vs kode), kebersihan API (alias/verba), optimistic lock di semua modul, konflik dokumen DB-per-layanan: desain lintas-PR, butuh ADR. |
| F8 | ✅ | Dokumentasi authenc dikoreksi; DCR jujur (`501`), PKCE hanya S256. |
| F9 | 🟡 | Allowlist CIDR untuk `/api/v1/iam/*` (`istio.iamAllowlist`, default OFF). **Tidak ada** Istio `AuthorizationPolicy` antar-service; mTLS tetap `PERMISSIVE` di base. |
| F10 | ⏳ | `gateway` bind `0.0.0.0` dan `/v1/secrets/*` tanpa autentikasi: **tidak disentuh** (mengganti default bind berisiko memutus deployment yang tak menyetel env; perlu verifikasi per lingkungan). |
| F11 | ✅ | CORS fail-closed; `*` hanya bila dinyatakan eksplisit dan tercatat. |
| F12 | 🟡 | PKCE `plain` dihapus; `mfa_pending` ditolak di introspect. `/introspect` **masih tanpa** autentikasi klien (RFC 7662 §2.1). |
| F13 | ⏳ | Dua design system dan token di `localStorage` bersama: keputusan arsitektur, tidak diubah. |

### Proses bisnis (bagian G)

| # | Status | Catatan |
|---|--------|---------|
| G1 | 🚫 | Istilah "Pengguna Barang"/"Kuasa Pengguna Barang" dibiarkan sesuai keputusan produk; risiko label pada dokumen resmi tetap dicatat di sini. |
| G2 | ⏳ | Tahap "diusulkan ke Pengelola Barang" pada RKBMN belum ada; butuh keputusan proses bisnis. |
| G3 | 🟡 | Jalur kewenangan tidak lagi dipilih pengusul. Persetujuan Pengelola Barang, Berita Acara, klasifikasi alasan, tautan SIMAN/SAKTI **belum ada**. |
| G4 | ✅ | `COMPLETED` penghapusan = `validator_pusat`, bukan admin TI. |
| G5 | ✅ | Bypass admin dihapus dari policy/engine/`require_any_role`. Break-glass lewat jalur teraudit (`V012__break_glass_log.sql`). **Notifikasi push** ke otoritas bisnis belum ada — hanya endpoint log. |
| G6, G8–G11 | ⏳ | BAST pemakaian, UU PDP pakaian dinas (dasar pemrosesan/retensi), eskalasi SLA, kontrak SIMAN nyata di staging, `is_staff` = admin TI: belum dikerjakan. |
| G7 | ✅ | Maker-checker berdasarkan **orang**, bukan hanya nama role (Pemakaian BMN). |

### Belum dikerjakan (ringkas)

Step-up `totp/disable`; rotasi refresh-token + deteksi reuse; cek revokasi pada
`extract_user_from_token`; autentikasi klien `/introspect`; Istio `AuthorizationPolicy`
antar-service; bind `gateway`; halaman MFA berbahasa Indonesia; kontras kelas utilitas;
delegasi yang benar-benar berlaku; migrasi kolom penghapusan; audit `integrasi` dan `secreton`.

## Temuan A–D (kondisi saat audit)

Cakupan: 149 file, 23 commit, +9.172/−853 baris. Judul & deskripsi PR hanya membahas CI dependency-check,
tetapi diff juga memuat: pustaka otorisasi bersama (`lib/core/src/authz.rs`), perubahan guard admin authenc,
middleware audit authenc, backend & frontend perlengkapan, frontend portal (auth/MFA/admin), nginx/CSP/Helm,
tooling screenshot + 73 PNG (14 MB).

## A. Keamanan & RBAC

| # | Sev | Temuan | Bukti |
|---|-----|--------|-------|
| A1 | Tinggi (regresi PR) | `audit_logs.ip_address` diisi lewat `$8::text::inet`. Komentar kode bilang nilai tak valid "dibuang jadi NULL" — salah: Postgres **melempar error**, seluruh INSERT gagal, dan error hanya di-log. Header `X-Forwarded-For: x` (atau `ip:port`, `[v6]:port`, `unknown`) dari penyerang tak terautentikasi membuat event login/authz **tidak tercatat**. | [T] Postgres 16: `garbage`, `1.2.3.4:5678`, `unknown`, `[2001:db8::1]:443`, `a, b` → `invalid input syntax for type inet` |
| A2 | Tinggi (regresi PR + pra-ada di perlengkapan) | `forwarded_client_ip` mempercayai entri **paling kiri** XFF — dikontrol klien. IP di audit dapat dipalsukan. MDN/OWASP: hanya percayai hop yang ditambahkan proxy tepercaya (mulai dari kanan). | [K] + rujukan standar |
| A3 | Tinggi (pra-ada, di middleware yang diubah PR) | `admin_auth_middleware` memverifikasi tanda tangan JWT tapi **tidak menolak token `mfa_pending`**. Token itu tak berisi `realm_access`, sehingga jatuh ke cabang "roles kosong → ambil dari DB" dan user admin lolos ke seluruh `/api/v1/iam/*` (buat user, reset password, matikan MFA, assign role) hanya dengan password. gRPC `ValidateToken` juga tidak menolaknya. Bertentangan dengan aturan `layanan/authenc/AGENTS.md`. | [K] `grep mfa_pending` di `iam-api`/`grpc`/`perlengkapan` = 0 hasil |
| A4 | Tinggi (keputusan desain) | PR melebarkan guard IAM authenc dari `admin` → `admin_pusat`/`superadmin`. IAM = IdP; `assign_role_to_user` tak punya guard eskalasi (bisa assign `admin` ke diri sendiri), tak ada last-admin protection, dan **tidak diaudit**. Kedua role baru itu tidak ada di seed (`002`, `004`) dan tak ada API pembuat role. | [K] |
| A5 | Tinggi | Backend perlengkapan memakai `resp.realm_roles.first()` sebagai satu-satunya role. Query role authenc **tanpa `ORDER BY`** → urutan tak tentu. Frontend (PR) menghitung kapabilitas dari **gabungan** semua role. Klaim PR "UI dan API sepakat by construction" hanya benar untuk allowlist admin, tidak untuk user multi-role (mis. seed baseline: operator+validator_wilayah+validator_pusat+admin). Akibatnya guard `enforce_no_admin_revoke` bisa lolos/terblokir tergantung urutan. Seed e2e sengaja "single role per user agar JWT role tidak ambigu". | [K] `stores/user_store.rs:24-29`, `middleware/mod.rs:165` |
| A6 | Sedang | Mutasi IAM (create/update/delete user, reset password, enable/disable MFA, assign/remove role) **tidak menulis audit**. Yang tertulis hanya AUTH_SUCCESS/AUTH_FAILURE/AUTH_ATTEMPT/AUTHZ_FAILURE/SUSPICIOUS_ACTIVITY; baris AUTHZ_FAILURE tanpa `user_id`. Label UI `USER_CREATE`, `ROLE_ASSIGN`, `CLIENT_*`, dll. adalah event yang tak pernah ditulis, sedangkan `AUTHZ_FAILURE` & `SUSPICIOUS_ACTIVITY` tampil mentah dan berwarna biru (info). OWASP Logging: perubahan privilege wajib dicatat dengan actor/target/before-after. | [K]+[S] |
| A7 | Sedang | `POST /mfa/backup-codes {action: generate}` mengganti seluruh kode pemulihan hanya dengan access token — tanpa step-up (TOTP/password). Sesi yang dicuri bisa menanam kode pemulihan sendiri. | [K] |
| A8 | Sedang | Klaim "satu definisi admin" belum terwujud: ≥9 salinan literal tersisa (`policy.rs` ×3, `penghapusan_bmn/handlers.rs:683`, `bantuan/ticket.rs:77`, `workflow/engine/transition.rs:258` (+`system`), `middleware/mod.rs:308`, FE `admin_roles.rs:164`). `kebutuhan_bmn/.../workflow.rs:534 is_admin_user` **masih prefix `admin_`** (menerima `admin_master_read_only`, `super_admin`, `administrator`; menolak `superadmin`) untuk *override darurat* keputusan Validator Pusat — persis anti-pola yang dilarang test PR. | [K] |
| A9 | Sedang | `Capability::View` menghasilkan **false untuk semua non-admin**; `approver_satker` & `validator_satker` = 0 kapabilitas. "Carve-out" ViewAudit/ManageTickets hanya dekoratif (keduanya = ADMIN_ROLES). 6 dari 8 kapabilitas tak dipakai di luar test; `require_permission` tak ter-mount; test `iam-api/integration_tests.rs` menguji tabel `lib_core`, bukan middleware. Doc menyebut `RoleSet::is_authenticated` yang tidak ada. `approver_satker` tak ada di `ROLE_CATALOG`/`PRIMARY_ROLE_PRIORITY`. | [T] scratch test |
| A10 | Sedang | Tiga kosakata otorisasi yang tak terhubung: (1) DB `roles`/`role_permissions`/`capabilities`/`role_capabilities`/`role_hierarchy`; (2) `lib_core::authz`; (3) `policy.rs` per-workflow. Halaman **Peran portal** menampilkan "N permission" dari DB yang tidak ditegakkan siapa pun; halaman **Pengaturan Role perlengkapan** statis, berjudul "Konfigurasi", memuat role hantu (`superadmin`, `admin_pusat` — tak ada di DB) dan tidak memuat `validator_satker`/`approver_satker` (ada di DB, dipakai rantai Pemakaian BMN). Test `catalog_covers_satker_roles` hanya memeriksa 3 dari 5. | [S]+[K] |
| A11 | Info | Realm/group/sub-group: skema ada (`realms`, `groups.parent_id`, `group_roles`, `group_attributes`, `role_hierarchy`) tetapi route-nya dihapus (#45), tak ada seed, tak ada di token, tak ada UI. Scope dihitung dari `satker_code` + role. Model Keycloak (rujukan) = grup hierarkis mewariskan role. | [K] |
| A12 | Sedang | UI "Cabut Izin" ditawarkan ke **admin** (screenshot), padahal mandat stakeholder melarang admin mencabut (`enforce_no_admin_revoke` → 403). Penyempitan `allowed_transitions` memakai `authorize()` yang mem-bypass admin. | [S]+[K] |
| A13 | Info (pra-ada) | `002_seed.sql` (dijalankan `authenc-migrate` di semua lingkungan) membuat user `199203142014031001` (password = NIP-nya, terlihat di repo publik) berperan operator+validator_wilayah+validator_pusat+**admin**, `require_password_change=true`. Perlu dipastikan sudah dirotasi/dihapus di prod. Seed e2e memakai hash sama dan header-nya menyebut bisa dijalankan ke staging. | [K] |

## B. Tampilan / alur

| # | Sev | Temuan | Bukti |
|---|-----|--------|-------|
| B1 | Sedang | Tombol **"+ Tambah Pengguna"** (`bg-primary-600 text-white`) tak terlihat: portal tidak mendefinisikan palet `primary` (hanya `navy`, `gold`) → Tailwind tidak menghasilkan apa pun. ≈85 pemakaian `primary-*` di portal (16× `bg-primary-600`). Guard `check-tailwind-shades-exist.py` lulus karena `if defined is None: continue` — buta terhadap palet yang sama sekali tak ada, padahal docstring-nya mengklaim menutup kasus `bg-primary` (#903). | [S]+[T] |
| B2 | Rendah | Halaman MFA portal (setup, verify, backup-codes) berbahasa Inggris; sisa aplikasi Indonesia. Kolom NAMA di daftar pengguna kosong semua. | [S] |
| B3 | Rendah | Audit UI menyembunyikan `request_id`, `suspicious_indicators`, `user_agent` dari kolom Detail — bertentangan dengan komentar "audit adalah bukti, tak boleh ada yang dibuang". | [K] |
| B4 | Sedang | CSP: `script-src 'unsafe-inline'` (dibutuhkan skrip boot inline Trunk) + `connect-src https:` + token JWT di `localStorage` (aturan repo). Satu XSS = pencurian token. RFC 10017 / OWASP CSP menganjurkan nonce/hash + `strict-dynamic`, dan BFF/token di memori. Ini menyelaraskan salinan compose dengan Helm (yang sudah begitu), bukan regresi baru. CSP/XFO Helm ≠ nginx di image (`SAMEORIGIN` vs `DENY`, font/CDN) → e2e/screenshot menguji konfigurasi yang bukan konfigurasi produksi. | [K] |
| B5 | Rendah | `chown nginx:nginx /etc/nginx/nginx.conf` membuat config bisa ditulis proses nginx (untuk `sed -i` entrypoint). Alternatif: render ke path tulis terpisah. | [K] |
| B6 | Positif | Penghapusan "Ganti Role" tepat (UI tidak boleh menyatakan identitas yang tak dimiliki); perbaikan kontrak MFA (`/mfa/setup`, `/mfa/verify`, refresh token) memperbaiki bug nyata; envelope `ApiResponse` dikonsolidasi; perbaikan pewarisan `add_header` nginx tepat. | |

## C. CI (inti PR)

| # | Sev | Temuan | Bukti |
|---|-----|--------|-------|
| C1 | Sedang | `NETWORK_PATTERNS` terlalu lebar → kegagalan **deterministik** dihitung HANG=hijau: banner `Updating crates.io index` + gagal resolve versi; docker `manifest unknown` (tag terpin hilang/salah — permanen); nama crate berisi `timeout` (`hyper-timeout`, `tokio-io-timeout` ada di pohon dependensi) + kegagalan lain; "crates.io" di teks panic. Melanggar aturan yang ditulis di skrip sendiri. Pola `--locked … needs to be updated` mati (pesan asli menyisipkan path). | [T] 5 dari 6 kasus → HANG |
| C2 | Sedang | `certificate verify failed` / `schannel` dihitung HANG=hijau: itu kegagalan integritas (MITM/CA), bukan ketersediaan. | [K]+[T] |
| C3 | Sedang (kebijakan) | HANG=hijau tanpa kontrol kompensasi; tidak konsisten (unduh alat gagal = merah; jalan alat gagal = hijau); `cargo-audit` & Verus diunduh tanpa checksum sementara `cargo-deny` dipin sha256; versi `cargo-audit` di `maintenance.yml` di-hardcode terpisah. | [K] |
| C4 | Rendah | Loop "repair" checkout gitleaks belum pernah dieksekusi di CI. Disimulasikan: merge commit PR ikut terambil (aman), tapi tak dijamin; tanpa `set -e` kegagalan `git checkout` tertutup `exit 0`. Perbaikan: `git fetch origin "$GITHUB_SHA"` eksplisit (teruji jalan). | [T] simulasi |
| C5 | Info | `self-test` wrapper (24 kasus) lulus; tetapi tidak memuat kasus false-green C1/C2. | [T] |

## D. Higiene PR
PR memuat ≥6 topik independen; deskripsi tidak menyebut perubahan authz/IAM/nginx/FE. Reviewer tidak dapat menilai blast radius dari judulnya. Pertimbangkan pemecahan (perubahan authz/IAM = perlu review keamanan tersendiri).

## Rujukan standar
- NIST/ANSI INCITS 359-2012 RBAC: Core, Hierarchical, SSD (saat penetapan), DSD (saat aktivasi role) — https://csrc.nist.gov/projects/role-based-access-control/faqs
- OWASP Authorization Cheat Sheet (deny by default, least privilege, server-side) — https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html
- OWASP Logging Cheat Sheet (aksi admin, perubahan privilege) — https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html
- OWASP CSP Cheat Sheet — https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html
- MDN X-Forwarded-For (hop tepercaya dari kanan) — https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/X-Forwarded-For
- RFC 9068 (klaim `groups`/`roles`/`entitlements`, SCIM RFC 7643) — https://datatracker.ietf.org/doc/html/rfc9068
- RFC 10017 OAuth 2.0 for Browser-Based Applications — https://datatracker.ietf.org/doc/html/rfc10017
- Keycloak Server Admin Guide (realm, grup hierarkis, composite role) — https://www.keycloak.org/docs/latest/server_admin/index.html

## Perluasan cakupan — semua modul

Metode: matriks statis 173 route perlengkapan (`scratchpad/route_matrix.py`) → baca handler+service yang mencurigakan;
router authenc, gateway, Helm/Istio; 30+ screenshot; pengukuran kontras dari piksel screenshot; riset regulasi.

Kedalaman: **dalam** = perlengkapan (semua route, ±25 handler dibaca), authenc IAM/auth/OAuth, gateway, ingress/Helm.
**Dangkal** = integrasi (gRPC tanpa interceptor; bergantung mTLS), secreton (204 route, 14 crate — hanya survei arsitektur,
butuh audit khusus). Tidak dibaca baris-per-baris: ±55 ribu LOC perlengkapan. Bukti: **[T]** dijalankan · **[K]** pembacaan kode · **[S]** screenshot.

## E. Otorisasi perlengkapan: autentikasi universal, otorisasi opsional (BFLA/BOLA — OWASP API1/API5:2023)

Router tidak punya lapisan auth; gerbangnya extractor `Claims` per-handler. 172/173 route memakai `Claims` (autentikasi OK), tetapi
**31 handler membuang `Claims` (`_claims`)** dan banyak lainnya memakainya hanya untuk `user_id`. Akibatnya setiap user
terautentikasi (role apa pun, termasuk fallback `"user"`) dapat:

| # | Sev | Endpoint | Dampak | Bukti |
|---|-----|----------|--------|-------|
| E1 | Kritis | `POST /pakaian-dinas/pengajuan/{id}/submit\|approve\|reject` | Mengubah status pengajuan pakaian dinas (DRAFT→SUBMITTED→SELESAI/DITOLAK) tanpa cek role/scope. Jalur lain (`/validator-action`) punya gerbang; ini jalur paralel yang tidak. | [K] handlers.rs:757-820, services.rs:768-860 (service tak menerima role) |
| E2 | Kritis | `PUT /kebutuhan-bmn/barang/{id}/approval` | Set `jml_setuju` (kuantitas disetujui RKBMN) pada barang mana pun, status apa pun, satker mana pun. Tanpa role, scope, cek state. | [K] barang.rs:38-56, satker_barang.rs:144-168 |
| E3 | Tinggi | `POST /kebutuhan-bmn/prioritas`, `PUT/DELETE /kebutuhan-bmn/pengajuan/{id}`, `POST …/pengajuan/{id}/satker` | Ubah prioritas / edit / hapus periode nasional / tambah satker — tanpa role. `delete_pengajuan` tak cek "draft only" di service. | [K] |
| E4 | Kritis | `GET /export/excel` (+ `/export/jobs/{id}/status\|download`) | `_claims` diabaikan; ekspor s.d. 50.000 baris `kebutuhan_bmn`/`pakaian_dinas` (memuat data pegawai) **tanpa scope**; job diunduh via UUID tanpa cek kepemilikan. UU PDP 27/2022 (pembatasan akses/minimisasi). | [K] export/handlers.rs |
| E5 | Tinggi | `GET /kebutuhan-bmn/search`, `/analisis`, `/forecast*`, `/workflow/monitoring/*` | `_claims` diabaikan → operator satker melihat data lintas satker/nasional (filter `satker_id` dipilih pemanggil sendiri). | [K] |
| E6 | Tinggi | `POST /pemakaian-bmn` (create), `PUT`, `/activate`, `/renew`, `/generate-konsep-surat`, `/upload-signed-pdf` | `PemakaianBmnPolicy` tidak dipanggil; role apa pun dalam scope dapat membuat/mengubah/mengaktifkan. | [K] handlers.rs:55-176 |
| E7 | Tinggi | `POST /pemakaian-bmn/{id}/transition` (jalur generik) | Aturan berbeda dari policy: `required_roles` hanya memetakan APPROVED/REJECTED/REVOKED/… — untuk target **CANCELLED, EXPIRED, ACTIVE** engine berkata "anyone can transition". Operator dapat meng-EXPIRE izin ACTIVE (padahal itu tugas scheduler); approver dapat REJECT langsung dari SUBMITTED (melewati validator); jalur ini juga tanpa `expected_version` (optimistic lock) yang dipakai jalur spesifik. | [K] config.rs:152-236, transition.rs:245-270 |
| E8 | Sedang | `POST /pemakaian-bmn/auto-expire` | Operasi batch global (untuk cron) terbuka bagi semua user (`_claims`). | [K] handlers.rs:615 |
| E9 | Tinggi | `POST /workflow/delegations` | Komentar bilang "middleware JWT menegakkan admin" — **middleware itu tidak ada**. Siapa pun membuat delegasi dengan `role` bebas-string tanpa dicek bahwa delegator memegang role itu. **Dan tak satu pun keputusan otorisasi membaca tabel delegasi** → fitur & halaman "Delegasi Workflow" dekoratif: user mengira wewenang sudah didelegasikan padahal tidak. | [K] grep `delegat` di luar modul delegasi = 0 |
| E10 | Tinggi | `POST /penghapusan-bmn` (create) | (a) `kewenangan_penetap_sk` (PUSAT/WILAYAH = siapa yang menerbitkan SK) **dipilih operator sendiri** — komentar kode: "seharusnya divalidasi di service (plan §6.3)"; (b) lookup nilai SIMAN memakai `AsetScope::All` → operator satker A dapat mengusulkan penghapusan BMN milik satker B; (c) bila query SIMAN error, "graceful fallback" memakai **nilai dari operator**. | [K] services.rs:85-140, repository.rs:45 |
| E11 | Sedang | `POST /penghapusan-bmn/{id}/lampiran` | Tanpa cek role; tipe file dipercaya dari header klien; tanpa allowlist tipe / sniff magic-byte / AV (OWASP File Upload); URL "presigned" TTL 1 tahun disimpan di DB. | [K] handlers.rs:534-620 |
| E12 | Sedang | `GET /dashboard/ws` | Token via query-string (tercatat di log proxy); tak divalidasi ulang selama koneksi hidup; broadcast tak difilter scope. Saat ini belum ada producer (latent). | [K] websocket.rs |

**Akar masalah (desain):** keamanan "gagal-terbuka karena kelalaian" — handler baru tanpa `Claims` = publik ke internet
(VirtualService `hosts: ["*"]`); handler baru dengan `Claims` tapi tanpa guard = terbuka untuk semua role. Tidak ada
deny-by-default (OWASP Authorization), tidak ada matriks uji RBAC (probe yang ada hanya menguji "tanpa token → bukan 2xx").

**Rate limiter tidak bekerja seperti klaimnya.** `rate_limit_middleware` membaca `Claims` dari *request extensions* ("set by auth
middleware") — tak ada yang menyisipkannya (grep `extensions_mut().insert` = 0) → semua request memakai `Uuid::nil()` →
"per-user limit" = **satu bucket global bersama**: satu klien berat mengunci semua pengguna; tak ada isolasi per-user. [K]

## F. Desain / arsitektur

| # | Sev | Kritik |
|---|-----|--------|
| F1 | Tinggi | **Durabilitas dokumen hukum.** SK penghapusan, surat izin, lampiran ditulis ke `/tmp/perlengkapan/docs` (default `DOCUMENT_STORAGE_PATH`) = `emptyDir` 128 Mi di Helm (`tmpVolumeSizeLimit: 128Mi`, tanpa PVC, tanpa env). Hilang saat pod restart/evict; melewati 128 Mi → pod di-evict; >1 replika → 404 di replika lain. Berkas kearsipan negara (UU 43/2009) di storage fana. [K] |
| F2 | Sedang | Token divalidasi lewat gRPC ke authenc **pada setiap request**, tanpa cache; authenc mati = seluruh API 500. Respons ValidateToken lossy (satu role). Alternatif standar: verifikasi JWT lokal via JWKS + daftar revokasi ber-TTL pendek (RFC 9068/8725). |
| F3 | Sedang | **Dua sumber kebenaran workflow**: `WorkflowConfig::default_*` di kode, tabel `ms_workflow_status`, dan `enum` status per modul; Pakaian memakai integer 1000-an warisan v1, modul lain string. Halaman "Konfigurasi Workflow" read-only (judul menyesatkan). |
| F4 | Sedang | **Tiga kosakata otorisasi** (DB `role_permissions`/`capabilities`, `lib_core::authz`, `policy.rs`+`required_roles`) + admin-bypass di masing-masing → tak ada satu pun yang bisa diaudit BPK sebagai "matriks". `policy.rs` sendiri mengakui tujuan itu tapi hanya Pemakaian yang memakainya. |
| F5 | Sedang | API: verba di URL (`/submit-wilayah`, `/forward-pusat`), **alias ganda** (`/generate-sk` ≡ `/generate-konsep-sk`, `/keputusan-pusat` ≡ `/validator-pusat`), jalur generik `/transition` berdampingan dengan jalur spesifik yang aturannya berbeda (E7), amplop `{success,data,message}` redundan dengan status HTTP, paginasi tak seragam. Tanpa OpenAPI/kontrak yang ditegakkan CI selain skrip drift FE/BE. |
| F6 | Sedang | Optimistic locking (`expected_version`) **hanya ada di jalur spesifik Pemakaian BMN** (baik). Kebutuhan, Penghapusan, Pakaian Dinas, dan jalur generik `/transition` tanpa versi/idempotency-key (hanya satu `FOR UPDATE` di `parallel.rs`) → klik ganda/retry = transisi ganda atau lost update. |
| F7 | Sedang | Root AGENTS.md wajibkan "setiap layanan DB terpisah"; layanan/AGENTS & authenc AGENTS menyatakan skema di **satu `dbsimpelv2`** dengan FK lintas skema (`perlengkapan → authenc.users`). Dokumen bertentangan; kopling data nyata. |
| F8 | Sedang | Dokumentasi berbohong: AGENTS.md authenc menandai DCR "✅ Production" padahal `/register*` adalah **stub yang membalas 201 Created** (POST) dan **204 No Content** (DELETE tanpa menghapus); "PKCE optional" padahal wajib; "HS256" padahal EdDSA. |
| F9 | Sedang | Zero-trust yang diklaim: mTLS `PERMISSIVE` di base/staging; **tidak ada Istio `AuthorizationPolicy`** (hanya NetworkPolicy L3/L4) → setiap pod backend berlabel dapat memanggil gRPC apa pun di authenc/integrasi (pegawai MySIMKARI = PII). `/api/v1/iam` (konsol admin IdP) dirutekan ke internet lewat ingress `hosts:["*"]`. |
| F10 | Sedang | `gateway` (sidecar simpelv1) mem-bind `0.0.0.0` padahal dokumentasi "localhost"; mengekspos `GET/PUT/DELETE /v1/secrets/{*path}` **tanpa autentikasi** dengan identitas Secreton milik gateway. |
| F11 | Sedang | CORS default `*` (origin/header/method Any) bila env tak diset — fail-open. |
| F12 | Sedang | OAuth: `code_challenge_method=plain` diterima (RFC 9700: S256); `/introspect` & `/auth/validate` tanpa autentikasi klien (RFC 7662 §2.1 MUST); keduanya menganggap token `mfa_pending` aktif. |
| F13 | Info | Dua design system (portal terang, perlengkapan gelap) untuk satu suite SSO; SSO via `localStorage` bersama → satu XSS mengambil alih kedua aplikasi + konsol IAM. |

## G. Proses bisnis — kritik terhadap regulasi & logika

Rujukan (hasil riset web, lihat tautan di bawah): PP 27/2014 jo. PP 28/2020; **PMK 153/PMK.06/2021** (perencanaan kebutuhan/RKBMN:
KPB menyusun → Pengguna Barang konsolidasi & teliti → *Pengelola Barang* menelaah/menyetujui); **PMK 83/PMK.06/2016** (pemusnahan &
penghapusan — masih berlaku, sedang direvisi; penghapusan oleh Pengguna Barang butuh persetujuan Pengelola Barang untuk kategori
tertentu); **PMK 40/2024** (tata cara penggunaan, pengganti PMK 246/2014); **PMK 207/PMK.06/2021** (pengawasan & pengendalian:
Pembantu Pengguna Barang Wilayah/Eselon I = pemantauan); **PMK 181/PMK.06/2016** (penatausahaan: Pengurus Barang, Penyimpan Barang, KPB).
*Catatan: ambang nilai persetujuan Pengelola Barang tidak saya temukan/verifikasi — wajib dikonfirmasi ke DJKN/Biro Perlengkapan.*

| # | Sev | Kritik |
|---|-----|--------|
| G1 | Tinggi | **Istilah kewenangan salah.** UI/kode menyebut Validator Pusat "Pengguna Barang" (persetujuan akhir) dan Approver Satker "Pengguna Barang Satker". Menurut PP 27/2014: Pengguna Barang = Jaksa Agung; kepala satker = **Kuasa Pengguna Barang**; Kejati = **Pembantu Pengguna Barang Wilayah**; unit pusat = Pembantu Pengguna Barang Eselon I; **Pengelola Barang = Menkeu (DJKN)**. Salah label di dokumen resmi/SK = cacat hukum & temuan audit. |
| G2 | Tinggi | **Kebutuhan BMN: "APPROVED/COMPLETED" di Validator Pusat tampil final**, padahal menurut PMK 153/2021 persetujuan RKBMN ada pada Pengelola Barang setelah konsolidasi Pengguna Barang. Tak ada state/field "diusulkan ke DJKN", hasil penelaahan, nomor/tanggal persetujuan. Keputusan internal direkam sebagai keputusan final. |
| G3 | Tinggi | **Penghapusan: jalur kewenangan dipilih pengusul.** Tak ada aturan (jenis BMN/nilai) yang menentukan PUSAT vs WILAYAH; tak ada langkah/isian **persetujuan Pengelola Barang** (PMK 83/2016), **Berita Acara Pemusnahan**, alasan terklasifikasi (dijual/dihibahkan/dimusnahkan/sebab lain), maupun tautan ke penghapusan di SIMAN/SAKTI (register resmi). "COMPLETED" hanya status lokal. |
| G4 | Tinggi | **Langkah terakhir dilakukan Admin TI.** `COMPLETED` mensyaratkan `admin_pusat` (role yang bahkan tak ada di seed) atau bypass admin/`system`. Finalisasi penghapusan adalah tindakan bisnis, bukan administrasi sistem → pemisahan tugas rusak di titik paling kritis. |
| G5 | Tinggi | **Admin-bypass di setiap kebijakan** (`authorize`, engine, `require_any_role`, `require_validator_wilayah`, penghapusan, monitoring): admin TI dapat menyetujui/menolak langkah bisnis (kecuali pencabutan izin — pengecualian, bukan prinsip). Break-glass tanpa dua-orang, tanpa notifikasi ke otoritas bisnis, hanya `tracing::warn`. Override darurat Kebutuhan: alasan ≥20 karakter + `is_admin_user` (prefix `admin_`). |
| G6 | Sedang | **Pemakaian BMN**: tak ada tahap **serah-terima/pengembalian** (BAST) — state akhir EXPIRED/REVOKED tanpa konfirmasi BMN kembali; tak ada rujukan dasar hukum/SOP internal pada SK izin (PMK 40/2024 mengatur *penggunaan* oleh K/L, bukan izin pemakaian pegawai). SLA aktivasi 8 jam, tetapi aktivasi dapat dipicu siapa pun (E6). |
| G7 | Sedang | **Maker-checker**: pengaju dan penyetuju dapat orang yang sama (user multi-role/admin); rantai Pemakaian meminta 3 aktor berbeda *menurut nama role*, bukan *menurut orang*. |
| G8 | Sedang | **Pakaian Dinas**: ukuran badan, jenis kelamin, `with_hijab` (inferensi agama) per pegawai; ekspor & roster tanpa pembatasan (E4); tak ada dasar pemrosesan/retensi/notice UU PDP; model status integer warisan v1 dan jalur approve/reject tak terjaga (E1). |
| G9 | Sedang | **SLA hanya pencatat**: `sla_minutes` + penjadwal ada, tetapi tak ada eskalasi ke atasan/delegasi (yang dekoratif, E9). |
| G10 | Sedang | **Bank Aset**: sumber tunggal SIMAN via cache `integrasi.siman_aset` (±624 ribu baris) — baik; tetapi staging memakai data mock yang "tidak memvalidasi bentuk API nyata" (AGENTS integrasi) → kontrak nyata pertama kali teruji di produksi. |
| G11 | Info | Helpdesk: `is_staff` = admin TI → triase tiket memakai hak berlebih. |

## H. Tampilan / aksesibilitas (diukur dari piksel screenshot repo)

Kontras teks vs latar pada `docs/assets/screenshots` (WCAG 2.1 AA: 4,5:1 teks normal) **[T]**:

| Elemen | Warna | Kontras |
|---|---|---|
| perlengkapan: label section sidebar ("MODUL UTAMA") | #475569 / #0d1527 | **2,40:1** ✗ |
| perlengkapan: footer kanan, "v0.1.0" | #475569 / #061423 | **2,43–2,45:1** ✗ |
| perlengkapan: label field ("PEMOHON"), footer kiri | #64748b / #101829 | **3,72–3,90:1** ✗ |
| portal: teks helper login, meta kartu role | #9ca3af / #ffffff | **2,54:1** ✗ |
| portal: tombol "+ Tambah Pengguna" | putih / #f9fafb | **1,05:1** ✗✗ (palet `primary` tak ada) |

Temuan lain **[S]**: glyph `▯` (tofu) pada Sesi Aktif, tombol Cari/Filter, kartu Peran; portal dashboard tertangkap saat animasi
belum selesai (kartu pudar) → galeri README menyesatkan; label peran mentah (`admin`) di Profil vs "Administrator Global" di header vs
"Administrator" di chip; kode satker mentah (`0100000`); halaman MFA berbahasa Inggris; "Konfigurasi Role/Workflow" hanya-baca;
halaman Delegasi menawarkan "Buat Delegasi" untuk fitur tak berlaku; kolom NAMA daftar pengguna kosong; MFA admin "Belum Aktif"
walau seed `mfa_policies` menyatakan enforce.

## Rujukan tambahan
- PMK 153/PMK.06/2021 Perencanaan Kebutuhan BMN — https://jdih.kemenkeu.go.id/dok/153-pmk-06-2021/summary
- PMK 83/PMK.06/2016 Pemusnahan & Penghapusan BMN — https://peraturan.bpk.go.id/Home/Details/121081/pmk-no-83pmk062016
- Penyempurnaan regulasi pemusnahan/penghapusan — https://djpp.kemenkum.go.id/publikasi/indeks-berita/pemerintah-sempurnakan-regulasi-pemusnahan-dan-penghapusan-barang-milik-negara-untuk-optimalisasi-pengelolaan-aset
- PMK 40/2024 Penggunaan BMN — https://jdih.kemenkeu.go.id/dok/pmk-40-tahun-2024
- PMK 207/PMK.06/2021 Pengawasan & Pengendalian BMN — https://jdih.kemenkeu.go.id/dok/207-pmk-06-2021/view
- PMK 181/PMK.06/2016 Penatausahaan BMN — https://peraturan.bpk.go.id/Details/121291/pmk-no-181pmk062016
- UU 27/2022 Pelindungan Data Pribadi — https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022
- RFC 9700 / RFC 7662 / OWASP File Upload & API Security Top 10 (2023)
