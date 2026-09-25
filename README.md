# 🏛️ SIMPEL (Sistem Informasi Perlengkapan)

[![Rust](https://img.shields.io/badge/rust-1.97%2B-orange.svg)](https://rustlang.org)
[![Leptos](https://img.shields.io/badge/leptos-0.8.20-green.svg)](https://leptos.dev)
[![Axum](https://img.shields.io/badge/axum-0.8.9-blue.svg)](https://github.com/tokio-rs/axum)
[![Kubernetes](https://img.shields.io/badge/kubernetes-ready-brightgreen.svg)](https://kubernetes.io)

**SIMPEL** adalah platform untuk manajemen Barang Milik Negara (BMN) di lingkungan Kejaksaan Republik Indonesia. Dibangun sepenuhnya menggunakan **Rust** dengan arsitektur workspace tunggal yang terdiri dari microfrontend (WebAssembly), backend services, dan layanan infrastruktur inti.

---

## ✨ Fitur Utama

- 🛡️ **Keamanan & Autentikasi**: IAM kustom (*Authenc*) dengan OAuth2/OIDC, MFA, RBAC, SAML, WebAuthn, dan manajemen rahasia (*Secreton*)
- 🧩 **Microfrontend (WASM)**: Antarmuka reaktif dibangun dengan Leptos 0.8 — Portal (SSO Gateway) dan Perlengkapan (BMN)
- ⚙️ **Backend Services**: API asinkron berperforma tinggi dengan Axum 0.8, komunikasi antar-layanan via gRPC (Tonic)
- 📦 **Workspace Terintegrasi**: Seluruh ~34 crate dikelola dalam satu `Cargo.toml` workspace dengan dependensi terpusat

---

## 🏗️ Arsitektur Sistem

```mermaid
graph TD
    User([Pengguna]) --> LB[Load Balancer / Nginx]

    subgraph Antarmuka [Microfrontends — Leptos WASM]
        LB --> Portal[Portal Gateway & SSO]
        LB --> Perlengkapan_UI[Aplikasi Perlengkapan]
    end

    subgraph Layanan [Backend & Core Services]
        Portal --> Authenc[Authenc — Identity Provider]
        Perlengkapan_UI --> Perlengkapan_API[Perlengkapan API]
        Perlengkapan_API --> Authenc

        Authenc -. "Kriptografi & Rahasia" .-> Secreton[Secreton — Vault]
        Perlengkapan_API -. "Kriptografi & Rahasia" .-> Secreton
    end

    subgraph Penyimpanan [Infrastruktur Data]
        Perlengkapan_API --> DB[(PostgreSQL)]
        Authenc --> DB
        Authenc --> Redis[(Redis)]
        Perlengkapan_API --> Redis
    end

    classDef frontend fill:#3b82f6,color:#fff,stroke:#1d4ed8;
    classDef backend fill:#10b981,color:#fff,stroke:#047857;
    classDef infra fill:#6366f1,color:#fff,stroke:#4338ca;

    class Portal,Perlengkapan_UI frontend;
    class Authenc,Perlengkapan_API,Secreton backend;
    class DB,Redis infra;
```

---

## 📁 Struktur Proyek

```bash
simpel2/
├── Cargo.toml               # Workspace manifest (single source of truth)
├── antarmuka/               # Microfrontend applications (Leptos WASM)
│   ├── portal/              #   Portal Gateway & SSO
│   └── perlengkapan/        #   Modul operasional BMN
├── layanan/                 # Backend & Core Services
│   ├── perlengkapan/        #   Layanan backend BMN terpadu (kebutuhan, dokumen, notifikasi, bantuan, dll)
│   ├── integrasi/           #   Integrasi layanan eksternal (MySIMKARI, SIMAN)
│   ├── authenc/crates/      #   10 crates: types, core, crypto, storage, api, iam-api,
│   │                        #              grpc, mfa, federation, webauthn
│   └── secreton/crates/     #   14 crates: core, api, storage, crypto, types, agent,
│                            #              cli, grpc, hsm, k8s-operator, auto-unseal,
│                            #              backup, health, replication
├── lib/                     # Shared Libraries
│   ├── ui/                  #   Komponen Antarmuka (Leptos)
│   ├── core/                #   Tipe data aman WASM (WASM-safe types)
│   ├── backend/             #   Infrastruktur backend (database, gRPC, middleware)
│   ├── crypto/              #   Primitif kriptografi bersama
│   └── perlengkapan/        #   Tipe domain perlengkapan
├── tests/                   # Integration & E2E tests
├── docs/                    # Dokumentasi engineering
└── infra/                   # Infrastruktur (K8s, Monitoring, Nginx)
```

---

## 🚀 Tumpukan Teknologi

| Lapisan | Teknologi | Versi | Tujuan |
|---------|-----------|-------|--------|
| **Bahasa** | Rust | 1.97+ (Edition 2024) | Memory safety & performa tinggi |
| **Frontend** | Leptos | 0.8.20 | Reaktivitas WASM (Client-Side Rendering) |
| **Backend HTTP** | Axum | 0.8.9 | REST API asinkron |
| **Backend gRPC** | Tonic + Prost | 0.14.x | Komunikasi antar-layanan terproteksi mTLS |
| **Database** | PostgreSQL | 15+ | Persistensi data relasional |
| **Cache** | Redis | — | Caching in-memory & session store |
| **Kriptografi** | Ed25519, ChaCha20 | — | Signing, enkripsi modern |
| **Orkestrasi** | Kubernetes | — | Manajemen container |

---

## 📸 Galeri Tampilan Aplikasi

Bagian ini dihasilkan otomatis dari aplikasi yang **benar-benar berjalan** — bukan
mockup dan bukan tangkapan tangan yang cepat basi. Alur pengambilan: satu stack
nyata dinaikkan (PostgreSQL, Redis, Authenc, Secreton, layanan-integrasi,
layanan-perlengkapan, kedua microfrontend), sesi dibuat lewat **login sungguhan**
ke Authenc, lalu setiap rute yang terdaftar di `routes.rs` kedua FE dibuka dan
diambil gambarnya oleh Playwright.

Tiap gambar melewati dua penjaga sebelum boleh tampil di sini:

1. `check.py` menolak gambar yang **kosong, putih polos, atau seragam** — sebuah
   galeri mengklaim setiap halaman merender sesuatu, dan PNG kosong membuat klaim
   itu bohong.
2. `capture.mjs` menandai setiap halaman yang menampilkan **404, halaman login di
   rute ter-autentikasi, panic WASM, enum mentah, UUID tak berlabel, atau timestamp
   UTC mentah**. Galeri tidak akan diterbitkan selama masih ada temuan yang belum
   diperbaiki.

Rute yang memang **wajib ditolak** (operator membuka halaman admin) ikut
didokumentasikan apa adanya: halaman "Akses Ditolak" di bawah ini adalah bukti
bahwa gerbang RBAC bekerja, bukan cacat tampilan.

<!-- BEGIN AUTOGENERATED SCREENSHOTS — jangan sunting manual; jalankan tests/e2e/screenshots/gallery.py -->

<!-- Generated by tests/e2e/screenshots/gallery.py — do not edit by hand. -->

Seluruh **70 tampilan** di bawah ini diambil otomatis dari aplikasi yang benar-benar berjalan (satu origin, sama seperti produksi), memakai sesi hasil login nyata ke *Authenc*. Setiap gambar telah diperiksa: tidak ada yang kosong, putih polos, atau berisi halaman 404.

| Cara memperbarui | Perintah |
|---|---|
| Ambil ulang seluruh gambar | `node tests/e2e/screenshots/capture.mjs` |
| Periksa kualitas gambar | `python3 tests/e2e/screenshots/check.py` |
| Bentuk ulang galeri ini (termasuk README) | `python3 tests/e2e/screenshots/gallery.py --prefix docs/assets/screenshots/ --embed README.md` |

### Portal

Katalog microfrontend dan pintasan modul.

**`/portal/`**

![/portal/](docs/assets/screenshots/portal__portal__.png)

<sub>1600×1003 piksel</sub>

**`/portal/apps`**

![/portal/apps](docs/assets/screenshots/portal__portal__apps.png)

<sub>1600×1089 piksel</sub>

**`/portal/dashboard`**

![/portal/dashboard](docs/assets/screenshots/portal__portal__dashboard.png)

<sub>1600×1089 piksel</sub>

**`/portal/logged-out`**

![/portal/logged-out](docs/assets/screenshots/portal__portal__logged-out.png)

<sub>1600×1000 piksel</sub>

**`/portal/logged-out (anon)`**

![/portal/logged-out (anon)](docs/assets/screenshots/portal__anon__portal__logged-out.png)

<sub>1600×1000 piksel</sub>

**`/portal/notifications`**

![/portal/notifications](docs/assets/screenshots/portal__portal__notifications.png)

<sub>1600×1089 piksel</sub>

### Masuk & Sesi

Titik masuk SSO dan hasil akhir sebuah sesi.

**`/perlengkapan/simpel/v2/login`**

![/perlengkapan/simpel/v2/login](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__login.png)

<sub>1600×3764 piksel</sub>

**`/perlengkapan/simpel/v2/login (anon)`**

![/perlengkapan/simpel/v2/login (anon)](docs/assets/screenshots/perlengkapan__anon__perlengkapan__simpel__v2__login.png)

<sub>1600×1000 piksel</sub>

**`/portal/callback`**

![/portal/callback](docs/assets/screenshots/portal__portal__callback.png)

<sub>1600×1000 piksel</sub>

**`/portal/login`**

![/portal/login](docs/assets/screenshots/portal__portal__login.png)

<sub>1600×1000 piksel</sub>

**`/portal/login (anon)`**

![/portal/login (anon)](docs/assets/screenshots/portal__anon__portal__login.png)

<sub>1600×1000 piksel</sub>

### Autentikasi Dua Faktor

Enrolment, verifikasi, dan kode pemulihan.

**`/portal/mfa/backup-codes`**

![/portal/mfa/backup-codes](docs/assets/screenshots/portal__portal__mfa__backup-codes.png)

<sub>1600×1089 piksel</sub>

**`/portal/mfa/backup-verify`**

![/portal/mfa/backup-verify](docs/assets/screenshots/portal__portal__mfa__backup-verify.png)

<sub>1600×1000 piksel</sub>

**`/portal/mfa/setup`**

![/portal/mfa/setup](docs/assets/screenshots/portal__portal__mfa__setup.png)

<sub>1600×1526 piksel</sub>

**`/portal/mfa/verify`**

![/portal/mfa/verify](docs/assets/screenshots/portal__portal__mfa__verify.png)

<sub>1600×1000 piksel</sub>

### Akun

Identitas dan pengaturan akun pengguna.

**`/portal/passkeys`**

![/portal/passkeys](docs/assets/screenshots/portal__portal__passkeys.png)

<sub>1600×1089 piksel</sub>

**`/portal/password`**

![/portal/password](docs/assets/screenshots/portal__portal__password.png)

<sub>1600×1089 piksel</sub>

**`/portal/profile`**

![/portal/profile](docs/assets/screenshots/portal__portal__profile.png)

<sub>1600×1107 piksel</sub>

**`/portal/sessions`**

![/portal/sessions](docs/assets/screenshots/portal__portal__sessions.png)

<sub>1600×1089 piksel</sub>

**`/portal/settings`**

![/portal/settings](docs/assets/screenshots/portal__portal__settings.png)

<sub>1600×1089 piksel</sub>

### Administrasi Portal

IAM: pengguna, peran, klien OAuth2, audit.

**`/portal/admin`**

![/portal/admin](docs/assets/screenshots/portal__portal__admin.png)

<sub>1600×1089 piksel</sub>

**`/portal/admin/audit`**

![/portal/admin/audit](docs/assets/screenshots/portal__portal__admin__audit.png)

<sub>1600×2506 piksel</sub>

**`/portal/admin/clients`**

![/portal/admin/clients](docs/assets/screenshots/portal__portal__admin__clients.png)

<sub>1600×1089 piksel</sub>

**`/portal/admin/clients/:id`**

![/portal/admin/clients/:id](docs/assets/screenshots/portal__portal__admin__clients__id.png)

<sub>1600×1089 piksel</sub>

**`/portal/admin/roles`**

![/portal/admin/roles](docs/assets/screenshots/portal__portal__admin__roles.png)

<sub>1600×1089 piksel</sub>

**`/portal/admin/users`**

![/portal/admin/users](docs/assets/screenshots/portal__portal__admin__users.png)

<sub>1600×1089 piksel</sub>

**`/portal/admin/users (operator)` — ditolak oleh guard**

![/portal/admin/users (operator)](docs/assets/screenshots/guard__operator__portal__admin__users.png)

<sub>1600×1000 piksel</sub>

**`/portal/admin/users/:id`**

![/portal/admin/users/:id](docs/assets/screenshots/portal__portal__admin__users__id.png)

<sub>1600×1089 piksel</sub>

### Dashboard & Beranda

Beranda aplikasi dan indeks modul.

**`/perlengkapan/simpel/v2/`**

![/perlengkapan/simpel/v2/](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__.png)

<sub>1600×3764 piksel</sub>

**`/perlengkapan/simpel/v2/dashboard`**

![/perlengkapan/simpel/v2/dashboard](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__dashboard.png)

<sub>1600×3764 piksel</sub>

### Bank Aset

Katalog BMN: daftar, dashboard, sebaran, dan QR.

**`/perlengkapan/simpel/v2/bank-aset/daftar`**

![/perlengkapan/simpel/v2/bank-aset/daftar](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bank-aset__daftar.png)

<sub>1600×1131 piksel</sub>

**`/perlengkapan/simpel/v2/bank-aset/daftar/:id`**

![/perlengkapan/simpel/v2/bank-aset/daftar/:id](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bank-aset__daftar__id.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/bank-aset/dashboard`**

![/perlengkapan/simpel/v2/bank-aset/dashboard](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bank-aset__dashboard.png)

<sub>1600×1569 piksel</sub>

**`/perlengkapan/simpel/v2/bank-aset/qrcode`**

![/perlengkapan/simpel/v2/bank-aset/qrcode](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bank-aset__qrcode.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/bank-aset/sebaran`**

![/perlengkapan/simpel/v2/bank-aset/sebaran](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bank-aset__sebaran.png)

<sub>1600×1000 piksel</sub>

### Kebutuhan BMN

Usulan kebutuhan per satker dan alur persetujuannya.

**`/perlengkapan/simpel/v2/kebutuhan-bmn/:id/edit`**

![/perlengkapan/simpel/v2/kebutuhan-bmn/:id/edit](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__kebutuhan-bmn__id__edit.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/kebutuhan-bmn/buat`**

![/perlengkapan/simpel/v2/kebutuhan-bmn/buat](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__kebutuhan-bmn__buat.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/kebutuhan-bmn/daftar`**

![/perlengkapan/simpel/v2/kebutuhan-bmn/daftar](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__kebutuhan-bmn__daftar.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/kebutuhan-bmn/detail/:id`**

![/perlengkapan/simpel/v2/kebutuhan-bmn/detail/:id](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__kebutuhan-bmn__detail__id.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/kebutuhan-bmn/laporan`**

![/perlengkapan/simpel/v2/kebutuhan-bmn/laporan](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__kebutuhan-bmn__laporan.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/kebutuhan-bmn/periode`**

![/perlengkapan/simpel/v2/kebutuhan-bmn/periode](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__kebutuhan-bmn__periode.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/kebutuhan-bmn/satker/:satker_id`**

![/perlengkapan/simpel/v2/kebutuhan-bmn/satker/:satker_id](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__kebutuhan-bmn__satker__satker_id.png)

<sub>1600×1077 piksel</sub>

### Pakaian Dinas

Jenis, campaign pengajuan, ukuran, dan laporan.

**`/perlengkapan/simpel/v2/pakaian-dinas/jenis`**

![/perlengkapan/simpel/v2/pakaian-dinas/jenis](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pakaian-dinas__jenis.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pakaian-dinas/jenis/:id/spesifikasi`**

![/perlengkapan/simpel/v2/pakaian-dinas/jenis/:id/spesifikasi](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pakaian-dinas__jenis__id__spesifikasi.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pakaian-dinas/laporan`**

![/perlengkapan/simpel/v2/pakaian-dinas/laporan](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pakaian-dinas__laporan.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pakaian-dinas/pengajuan`**

![/perlengkapan/simpel/v2/pakaian-dinas/pengajuan](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pakaian-dinas__pengajuan.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pakaian-dinas/pengajuan/:pengajuan_id/satker`**

![/perlengkapan/simpel/v2/pakaian-dinas/pengajuan/:pengajuan_id/satker](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pakaian-dinas__pengajuan__pengajuan_id__satker.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pakaian-dinas/pengajuan/:pengajuan_id/satker/:satker_code/isi`**

![/perlengkapan/simpel/v2/pakaian-dinas/pengajuan/:pengajuan_id/satker/:satker_code/isi](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pakaian-dinas__pengajuan__pengajuan_id__satker__satker_code__isi.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pakaian-dinas/ukuran`**

![/perlengkapan/simpel/v2/pakaian-dinas/ukuran](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pakaian-dinas__ukuran.png)

<sub>1600×1000 piksel</sub>

### Pemakaian BMN

Izin pemakaian beserta rantai persetujuannya.

**`/perlengkapan/simpel/v2/pengelolaan/pemakaian`**

![/perlengkapan/simpel/v2/pengelolaan/pemakaian](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pengelolaan__pemakaian.png)

<sub>1600×1059 piksel</sub>

**`/perlengkapan/simpel/v2/pengelolaan/pemakaian/buat`**

![/perlengkapan/simpel/v2/pengelolaan/pemakaian/buat](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pengelolaan__pemakaian__buat.png)

<sub>1600×1619 piksel</sub>

**`/perlengkapan/simpel/v2/pengelolaan/pemakaian/detail/:id`**

![/perlengkapan/simpel/v2/pengelolaan/pemakaian/detail/:id](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pengelolaan__pemakaian__detail__id.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pengelolaan/pemakaian/monitoring`**

![/perlengkapan/simpel/v2/pengelolaan/pemakaian/monitoring](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pengelolaan__pemakaian__monitoring.png)

<sub>1600×1273 piksel</sub>

### Penghapusan BMN

Usulan penghapusan sampai Konsep SK.

**`/perlengkapan/simpel/v2/pengelolaan/penghapusan`**

![/perlengkapan/simpel/v2/pengelolaan/penghapusan](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pengelolaan__penghapusan.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/pengelolaan/penghapusan/buat`**

![/perlengkapan/simpel/v2/pengelolaan/penghapusan/buat](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pengelolaan__penghapusan__buat.png)

<sub>1600×1328 piksel</sub>

**`/perlengkapan/simpel/v2/pengelolaan/penghapusan/detail/:id`**

![/perlengkapan/simpel/v2/pengelolaan/penghapusan/detail/:id](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__pengelolaan__penghapusan__detail__id.png)

<sub>1600×1690 piksel</sub>

### Analitik & Roadmap

Proyeksi kebutuhan sarana-prasarana.

**`/perlengkapan/simpel/v2/analitik/roadmap`**

![/perlengkapan/simpel/v2/analitik/roadmap](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__analitik__roadmap.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/analitik/roadmap/buat`**

![/perlengkapan/simpel/v2/analitik/roadmap/buat](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__analitik__roadmap__buat.png)

<sub>1600×1000 piksel</sub>

### Notifikasi

Kotak masuk notifikasi Perlengkapan.

**`/perlengkapan/simpel/v2/notifikasi`**

![/perlengkapan/simpel/v2/notifikasi](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__notifikasi.png)

<sub>1600×1000 piksel</sub>

### Bantuan

Panduan, FAQ, dan helpdesk.

**`/perlengkapan/simpel/v2/bantuan/faq`**

![/perlengkapan/simpel/v2/bantuan/faq](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bantuan__faq.png)

<sub>1600×1031 piksel</sub>

**`/perlengkapan/simpel/v2/bantuan/helpdesk`**

![/perlengkapan/simpel/v2/bantuan/helpdesk](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bantuan__helpdesk.png)

<sub>1600×1274 piksel</sub>

**`/perlengkapan/simpel/v2/bantuan/panduan`**

![/perlengkapan/simpel/v2/bantuan/panduan](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__bantuan__panduan.png)

<sub>1600×1409 piksel</sub>

### Administrasi Perlengkapan

Peran dan hak akses.

**`/perlengkapan/simpel/v2/admin/audit`**

![/perlengkapan/simpel/v2/admin/audit](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__admin__audit.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/admin/master`**

![/perlengkapan/simpel/v2/admin/master](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__admin__master.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/admin/roles`**

![/perlengkapan/simpel/v2/admin/roles](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__admin__roles.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/admin/roles (operator)` — ditolak oleh guard**

![/perlengkapan/simpel/v2/admin/roles (operator)](docs/assets/screenshots/guard__operator__perlengkapan__simpel__v2__admin__roles.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/admin/templates`**

![/perlengkapan/simpel/v2/admin/templates](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__admin__templates.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/admin/workflow`**

![/perlengkapan/simpel/v2/admin/workflow](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__admin__workflow.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/admin/workflow-delegation`**

![/perlengkapan/simpel/v2/admin/workflow-delegation](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__admin__workflow-delegation.png)

<sub>1600×1000 piksel</sub>

**`/perlengkapan/simpel/v2/admin/workflow-monitoring`**

![/perlengkapan/simpel/v2/admin/workflow-monitoring](docs/assets/screenshots/perlengkapan__perlengkapan__simpel__v2__admin__workflow-monitoring.png)

<sub>1600×1285 piksel</sub>

<!-- END AUTOGENERATED SCREENSHOTS -->

---

## 🏁 Memulai Pengembangan

### 1. Persyaratan Sistem

- **Rust 1.97+** (`rustup` dengan target `wasm32-unknown-unknown`)
- **Trunk** (`cargo install trunk` atau `cargo binstall trunk`)
- **Docker & Docker Compose** (PostgreSQL & Redis lokal)

### 2. Instalasi

```bash
git clone <URL_REPOSITORY> simpel
cd simpel
cp .env.example .env
docker compose up -d postgres redis
```

### 3. Menjalankan Service

**Frontend (WASM + hot-reload):**

```bash
cd antarmuka/portal && trunk serve --port 8080 --open
cd antarmuka/perlengkapan && trunk serve --port 8081 --open
```

**Backend API:**

```bash
cargo run --bin layanan-perlengkapan
cargo run --bin authenc
cargo run --bin api_server
```

### 4. Verifikasi Workspace

```bash
cargo check --workspace                                    # Cek kompilasi
cargo clippy --workspace --all-targets -- -D warnings      # Linter
cargo fmt --all                                            # Format kode
cargo test --workspace                                     # Jalankan semua tes
```

---

## 🛡️ Kebijakan Keamanan

SIMPEL mematuhi paradigma **Security-by-Design**:

- Arsitektur autentikasi berbasis token modern (OAuth2/OIDC) dengan rotasi kunci via Secreton
- Kriptografi modern: Ed25519 (signing), ChaCha20-Poly1305 (enkripsi), Argon2id (password hashing)
- Strict `unsafe_code = "forbid"` pada level workspace
- Zero-trust: semua komunikasi antar-layanan via mTLS (gRPC)

---

## 🤝 Panduan Kontribusi

1. Buat cabang (*branch*) dari `main` berdasarkan penugasan spesifik
2. Pastikan kode selaras: `cargo fmt --all`
3. Pastikan tidak ada warning: `cargo clippy --workspace --all-targets -- -D warnings`
4. Commit dengan format [Conventional Commits](https://www.conventionalcommits.org/)

Panduan selengkapnya: 📖 [CONTRIBUTING.md](CONTRIBUTING.md) · Panduan AI Agent: 🤖 [AGENTS.md](AGENTS.md)

---

## ☎️ Bantuan & Dukungan

Lihat `docs/` untuk panduan arsitektur mendalam dan pemahaman logika lintas layanan.

> **SIMPEL (Sistem Informasi Perlengkapan)**
> Hak Cipta © Kejaksaan Agung Republik Indonesia. Semua Hak Dilindungi Undang-Undang.
