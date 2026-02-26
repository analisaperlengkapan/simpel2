# SIMPEL — Sistem Informasi Manajemen Perlengkapan

## Dokumen Spesifikasi Kebutuhan Perangkat Lunak (Software Requirements Specification)

**Versi:** 1.0.0-draft  
**Tanggal:** 9 Februari 2026  
**Klasifikasi:** Internal — Kejaksaan Republik Indonesia  
**Format:** EARLS (Entities, Activities, Requirements, Lifecycle, Stories)

---

## Daftar Isi

1. [Pendahuluan](#1-pendahuluan)
2. [Ringkasan Sistem](#2-ringkasan-sistem)
3. [Arsitektur Tingkat Tinggi](#3-arsitektur-tingkat-tinggi)
4. [Entitas & Aktor (Entities)](#4-entitas--aktor-entities)
5. [Layanan 1 — Master Data & Referensi (`master`)](#5-layanan-1--master-data--referensi-master)
6. [Layanan 2 — Analisis Kebutuhan BMN (`kebutuhan`)](#6-layanan-2--analisis-kebutuhan-bmn-kebutuhan)
7. [Layanan 3 — Pemakaian BMN (`pemakaian`)](#7-layanan-3--pemakaian-bmn-pemakaian)
8. [Layanan 4 — Workflow (`workflow`)](#8-layanan-4--workflow-workflow)
9. [Layanan 5 — Dokumen (`dokumen`)](#9-layanan-5--dokumen-dokumen)
10. [Layanan 6 — Dashboard & Analitik (`dashboard`)](#10-layanan-6--dashboard--analitik-dashboard)
11. [Layanan 7 — Integrasi (`integrasi`)](#11-layanan-7--integrasi-integrasi)
12. [Layanan 8 — Notifikasi (`notifikasi`)](#12-layanan-8--notifikasi-notifikasi)
13. [Layanan 9 — Autentikasi & Otorisasi (`authenc`)](#13-layanan-9--autentikasi--otorisasi-authenc)
14. [Kebutuhan Non-Fungsional](#14-kebutuhan-non-fungsional)
15. [Matriks Ketergantungan Antar Layanan](#15-matriks-ketergantungan-antar-layanan)
16. [Glosarium](#16-glosarium)

---

## 1. Pendahuluan

### 1.1 Tujuan Dokumen

Dokumen ini mendefinisikan kebutuhan perangkat lunak secara lengkap dan komprehensif untuk aplikasi SIMPEL (Sistem Informasi Manajemen Perlengkapan) yang dibangun untuk lingkungan Kejaksaan Republik Indonesia. Dokumen ini menjadi acuan utama bagi tim pengembang, penguji, dan pemangku kepentingan selama siklus hidup pengembangan perangkat lunak.

### 1.2 Ruang Lingkup Produk

SIMPEL adalah aplikasi berbasis web untuk manajemen Barang Milik Negara (BMN) di lingkungan Kejaksaan RI. Aplikasi ini menangani kebutuhan BMN yang **tidak tercakup dalam Standar Barang dan Standar Kebutuhan (SBSK)** sebagaimana diatur dalam Peraturan Menteri Keuangan. Dengan demikian, SIMPEL melengkapi — bukan menggantikan — modul RKBMN pada SIMAN v2 yang hanya menangani BMN ber-SBSK. SIMPEL mengelola standar spesifikasi dan kebutuhan BMN yang diatur secara internal oleh Kejaksaan.

### 1.3 Batasan dan Asumsi

**Batasan:**

- SIMPEL hanya menangani kebutuhan BMN non-SBSK; kebutuhan BMN yang sudah memiliki SBSK tetap dikelola melalui SIMAN v2.
- Data BMN eksisting diperoleh melalui integrasi dengan MonSAKTI; SIMPEL tidak menjadi sumber kebenaran (source of truth) untuk data akuntansi BMN.
- Data kepegawaian diperoleh melalui integrasi dengan MySIMKARI; SIMPEL tidak mengelola data kepegawaian secara mandiri.
- Ketersediaan dan stabilitas API MonSAKTI serta MySIMKARI berada di luar kendali tim pengembang SIMPEL.

**Asumsi:**

- Setiap satuan kerja (satker) memiliki koneksi internet yang memadai untuk mengakses aplikasi.
- MonSAKTI dan MySIMKARI menyediakan API atau mekanisme ekspor data yang dapat diintegrasikan.
- Struktur organisasi Kejaksaan yang berlaku meliputi Kejaksaan Agung, Kejaksaan Tinggi (Kejati), Kejaksaan Negeri (Kejari), dan Cabang Kejaksaan Negeri (Cabjari).
- Pengguna memiliki literasi digital dasar untuk mengoperasikan aplikasi berbasis web.

### 1.4 Referensi Regulasi

- Peraturan Pemerintah Nomor 27 Tahun 2014 tentang Pengelolaan Barang Milik Negara/Daerah beserta perubahannya.
- Peraturan Menteri Keuangan tentang Standar Barang dan Standar Kebutuhan (SBSK).
- Peraturan Menteri Keuangan tentang Penatausahaan Barang Milik Negara.
- Peraturan internal Kejaksaan RI terkait pengelolaan perlengkapan dan BMN.
- Peraturan Jaksa Agung terkait standar spesifikasi dan kebutuhan BMN internal.

---

## 2. Ringkasan Sistem

### 2.1 Deskripsi Umum

SIMPEL terdiri dari sembilan microservice yang saling berkomunikasi melalui API internal. Setiap layanan memiliki tanggung jawab tunggal (single responsibility) dan dapat di-deploy, di-scale, serta di-maintain secara independen.

### 2.2 Daftar Layanan

| Kode Layanan | Nama Layanan | Deskripsi Singkat |
|---|---|---|
| `master` | Master Data & Referensi | Pengelolaan data referensi: kodefikasi BMN, standar spesifikasi, standar jumlah, dan mapping kodefikasi |
| `kebutuhan` | Analisis Kebutuhan BMN | Pengumpulan, pengolahan, analisis, dan penyajian kebutuhan BMN non-SBSK, termasuk kebutuhan pakaian dinas dan roadmap sarpras |
| `pemakaian` | Pemakaian BMN | Pengelolaan izin pemakaian BMN (kendaraan, rumah negara, laptop, dll.) |
| `workflow` | Workflow Engine | Mesin alur kerja terpusat untuk proses persetujuan dan eskalasi |
| `dokumen` | Manajemen Dokumen | Pembuatan, penyimpanan, dan pengelolaan dokumen administratif BMN |
| `dashboard` | Dashboard & Analitik | Visualisasi data, pelaporan, dan analisis lintas layanan |
| `integrasi` | Integration Gateway | Adapter dan cache untuk sistem eksternal (MonSAKTI, MySIMKARI) |
| `notifikasi` | Notifikasi | Pengiriman notifikasi multi-kanal (email, in-app, push) |
| `authenc` | Autentikasi & Otorisasi | Identity and Access Management, audit trail |

### 2.3 Perbedaan dengan SIMAN v2

| Aspek | SIMAN v2 (RKBMN) | SIMPEL |
|---|---|---|
| Cakupan BMN | BMN yang memiliki SBSK sesuai PMK | BMN non-SBSK yang diatur internal Kejaksaan |
| Dasar Perhitungan | SBSK dari PMK | Standar spesifikasi dan standar jumlah internal Kejaksaan |
| Sumber Standar | Peraturan Menteri Keuangan | Peraturan/Keputusan internal Kejaksaan |
| Scope Fungsional | Perencanaan kebutuhan (RKBMN) | Perencanaan, pemakaian, kodefikasi, roadmap, dan dokumen |

---

## 3. Arsitektur Tingkat Tinggi

### 3.1 Tech Stack

| Komponen | Teknologi | Keterangan |
|---|---|---|
| Frontend | Leptos (Rust) | Framework web reaktif berbasis Rust dengan WebAssembly |
| Backend | Axum (Rust) | Framework HTTP untuk setiap microservice |
| Inter-service Communication | gRPC (tonic) + REST | gRPC untuk komunikasi sinkron antar layanan; REST untuk API publik/frontend |
| Message Broker | NATS / RabbitMQ | Komunikasi asinkron dan event-driven antar layanan |
| Database | PostgreSQL | Satu instance dengan schema separation per layanan, atau database terpisah |
| Cache | Redis | Caching data referensi, session, dan data integrasi |
| Object Storage | MinIO / S3-compatible | Penyimpanan dokumen dan lampiran |
| Containerization | Docker + Kubernetes | Orkestrasi dan deployment |
| CI/CD | GitHub Actions / GitLab CI | Pipeline otomatis |

### 3.2 Pola Arsitektur

- **Database-per-service (logical):** Setiap microservice memiliki schema PostgreSQL sendiri. Tidak ada akses langsung antar schema; komunikasi hanya melalui API.
- **API Gateway:** Satu titik masuk untuk semua request dari frontend, menangani routing ke microservice yang tepat.
- **Event-driven:** Perubahan state penting dipublikasikan sebagai event melalui message broker (contoh: "izin pemakaian disetujui" → trigger notifikasi dan pembuatan dokumen).
- **CQRS (opsional):** Untuk layanan dashboard, digunakan read-model terpisah yang dioptimalkan untuk query analitik.

### 3.3 Diagram Konteks

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PENGGUNA SIMPEL                              │
│  (Operator Satker, Verifikator, Admin Pusat, Pimpinan, PPBMN)      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ HTTPS
                           ▼
                    ┌──────────────┐
                    │  API Gateway │
                    │   (Reverse   │
                    │    Proxy)    │
                    └──────┬───────┘
                           │
          ┌────────────────┼────────────────────────────────┐
          │                │                                │
    ┌─────▼─────┐   ┌─────▼──────┐   ┌──────────┐   ┌─────▼──────┐
    │  authenc   │   │  master    │   │kebutuhan │   │ pemakaian  │
    └───────────┘   └────────────┘   └──────────┘   └────────────┘
          │                │                │               │
    ┌─────▼─────┐   ┌─────▼──────┐   ┌─────▼────┐   ┌─────▼──────┐
    │ workflow   │   │  dokumen   │   │dashboard │   │ notifikasi │
    └───────────┘   └────────────┘   └──────────┘   └────────────┘
                           │
                    ┌──────▼───────┐
                    │  integrasi   │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────────┐
              ▼                             ▼
       ┌─────────────┐              ┌──────────────┐
       │  MonSAKTI    │              │  MySIMKARI   │
       │ (Data BMN)   │              │(Data Pegawai)│
       └─────────────┘              └──────────────┘
```

---

## 4. Entitas & Aktor (Entities)

### 4.1 Aktor Sistem

| Kode Aktor | Nama Aktor | Deskripsi | Level Akses |
|---|---|---|---|
| `A01` | Super Admin | Administrator sistem secara keseluruhan | Akses penuh ke semua layanan dan konfigurasi sistem |
| `A02` | Admin Pusat | Pengelola BMN di tingkat Kejaksaan Agung (Biro Perlengkapan) | Kelola master data, verifikasi akhir, penerbitan dokumen tingkat pusat |
| `A03` | Admin Wilayah | Pengelola BMN di tingkat Kejaksaan Tinggi | Verifikasi data satker di wilayahnya, rekapitulasi wilayah |
| `A04` | Operator Satker | Pengelola BMN di tingkat satuan kerja (Kejari/Cabjari) | Input data kebutuhan, pengajuan pemakaian, pelaporan kondisi BMN |
| `A05` | Verifikator | Pejabat yang melakukan verifikasi dan persetujuan | Verifikasi pengajuan, persetujuan/penolakan |
| `A06` | Pimpinan Satker | Kepala Kejari/Cabjari atau pejabat yang berwenang | Persetujuan akhir di level satker, melihat dashboard |
| `A07` | Pimpinan Wilayah | Kepala Kejaksaan Tinggi atau pejabat yang berwenang | Monitoring wilayah, persetujuan level wilayah |
| `A08` | Pimpinan Pusat | Pejabat eselon I/II di Kejaksaan Agung | Monitoring nasional, keputusan strategis |
| `A09` | PPBMN | Pejabat Penatausahaan BMN | Penatausahaan, pemeliharaan register BMN |
| `A10` | Auditor | Pemeriksa internal/eksternal | Akses baca ke data dan audit trail |
| `A11` | Pegawai | Pegawai Kejaksaan (sebagai pengguna/peminjam BMN) | Pengajuan pemakaian BMN untuk diri sendiri, melihat status pengajuan |

### 4.2 Entitas Data Utama

| Kode Entitas | Nama Entitas | Deskripsi | Layanan Pemilik |
|---|---|---|---|
| `E01` | Satuan Kerja | Unit organisasi Kejaksaan (Kejagung, Kejati, Kejari, Cabjari) | `authenc` |
| `E02` | Pegawai | Data pegawai Kejaksaan (sumber: MySIMKARI) | `integrasi` |
| `E03` | Barang Milik Negara | Data aset/barang milik negara (sumber: MonSAKTI) | `integrasi` |
| `E04` | Kode Barang | Kodefikasi standar BMN berdasarkan SIMAK BMN dan internal | `master` |
| `E05` | Standar Spesifikasi | Spesifikasi teknis standar per jenis BMN | `master` |
| `E06` | Standar Jumlah | Kebutuhan jumlah standar BMN per satker/per pegawai | `master` |
| `E07` | Kebutuhan BMN | Usulan kebutuhan BMN dari satker | `kebutuhan` |
| `E08` | Kebutuhan Pakaian Dinas | Usulan kebutuhan pakaian dinas per pegawai | `kebutuhan` |
| `E09` | Roadmap Sarpras | Rencana kebutuhan sarana prasarana 5 tahunan | `kebutuhan` |
| `E10` | Izin Pemakaian | Izin penggunaan BMN (kendaraan, rumah negara, laptop) | `pemakaian` |
| `E11` | Workflow Instance | Instance proses persetujuan yang sedang berjalan | `workflow` |
| `E12` | Dokumen | Dokumen administratif yang dihasilkan (SK, surat izin, dll.) | `dokumen` |
| `E13` | Template Dokumen | Template untuk pembuatan dokumen otomatis | `dokumen` |
| `E14` | Mapping Kodefikasi | Pemetaan kode barang non-standar ke kode standar | `master` |
| `E15` | Riwayat Pemenuhan | Data realisasi pemenuhan kebutuhan BMN per tahun | `kebutuhan` |

### 4.3 Hierarki Organisasi

```
Kejaksaan Agung (Pusat)
├── Biro Perlengkapan (Pengelola BMN Pusat)
├── Kejaksaan Tinggi (Kejati) — 34 provinsi
│   ├── Bagian Tata Usaha / Pembinaan
│   ├── Kejaksaan Negeri (Kejari)
│   │   ├── Sub Bagian Pembinaan
│   │   └── Cabang Kejaksaan Negeri (Cabjari)
```

Hierarki ini menentukan:

- Alur eskalasi persetujuan (Cabjari → Kejari → Kejati → Kejagung)
- Agregasi data pelaporan (bottom-up)
- Cakupan akses data (seorang Operator Satker hanya melihat data satkernya sendiri)

---

## 5. Layanan 1 — Master Data & Referensi (`master`)

### 5.1 Deskripsi Layanan

Layanan Master Data & Referensi bertanggung jawab atas pengelolaan seluruh data referensi yang digunakan oleh layanan-layanan lain dalam ekosistem SIMPEL. Layanan ini bersifat sebagai sumber kebenaran (source of truth) untuk standar internal Kejaksaan terkait BMN, mencakup kodefikasi barang, standar spesifikasi teknis, dan standar jumlah kebutuhan.

### 5.2 Entitas (Entities)

#### E04 — Kode Barang

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode_barang` | String(30) | Ya | Kode barang sesuai standar SIMAK BMN |
| `kode_barang_internal` | String(30) | Tidak | Kode barang internal Kejaksaan (jika berbeda) |
| `nama_barang` | String(255) | Ya | Nama resmi barang |
| `kelompok` | String(100) | Ya | Kelompok barang (Tanah, Peralatan dan Mesin, Gedung dan Bangunan, Jalan Irigasi dan Jaringan, Aset Tetap Lainnya, KDP) |
| `sub_kelompok` | String(100) | Tidak | Sub kelompok barang |
| `sub_sub_kelompok` | String(100) | Tidak | Sub-sub kelompok barang |
| `satuan` | String(20) | Ya | Satuan pengukuran (unit, buah, set, m², dll.) |
| `is_aktif` | Boolean | Ya | Status aktif/tidak aktif |
| `is_sbsk` | Boolean | Ya | Apakah termasuk SBSK PMK (true = dikelola SIMAN, false = dikelola SIMPEL) |
| `tahun_berlaku_mulai` | Integer | Ya | Tahun mulai berlaku |
| `tahun_berlaku_akhir` | Integer | Tidak | Tahun akhir berlaku (null = masih berlaku) |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |
| `updated_by` | UUID | Ya | ID pengguna pengubah terakhir |
| `version` | Integer | Ya | Versi data (optimistic locking) |

#### E05 — Standar Spesifikasi

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode_barang_id` | UUID (FK→E04) | Ya | Referensi ke kode barang |
| `nama_standar` | String(255) | Ya | Nama standar spesifikasi |
| `deskripsi` | Text | Tidak | Deskripsi lengkap standar |
| `spesifikasi_teknis` | JSONB | Ya | Detail spesifikasi teknis (fleksibel per jenis barang) |
| `satuan` | String(20) | Ya | Satuan pengukuran |
| `peruntukan` | String(100) | Tidak | Peruntukan (mis: eselon I, eselon II, staf) |
| `dasar_hukum` | String(500) | Tidak | Referensi peraturan internal yang mendasari |
| `masa_pakai_bulan` | Integer | Tidak | Masa pakai standar dalam bulan |
| `tahun_berlaku` | Integer | Ya | Tahun anggaran berlaku |
| `is_aktif` | Boolean | Ya | Status aktif |
| `catatan` | Text | Tidak | Catatan tambahan |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |
| `version` | Integer | Ya | Versi data |

#### E06 — Standar Jumlah

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode_barang_id` | UUID (FK→E04) | Ya | Referensi ke kode barang |
| `standar_spesifikasi_id` | UUID (FK→E05) | Tidak | Referensi ke standar spesifikasi (jika terkait) |
| `tipe_perhitungan` | Enum | Ya | Basis perhitungan: `PER_SATKER`, `PER_PEGAWAI`, `PER_JABATAN`, `PER_GOLONGAN`, `PER_LUAS_KANTOR` |
| `jumlah_standar` | Decimal | Ya | Jumlah kebutuhan standar |
| `satuan` | String(20) | Ya | Satuan |
| `kualifikasi_satker` | JSONB | Tidak | Filter satker yang berlaku (mis: hanya Kejari tipe A) |
| `kualifikasi_pegawai` | JSONB | Tidak | Filter pegawai yang berlaku (mis: golongan III ke atas) |
| `dasar_hukum` | String(500) | Tidak | Referensi peraturan internal |
| `tahun_berlaku` | Integer | Ya | Tahun anggaran berlaku |
| `is_aktif` | Boolean | Ya | Status aktif |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |
| `version` | Integer | Ya | Versi data |

#### E14 — Mapping Kodefikasi

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `satker_id` | UUID | Ya | Satker yang memiliki kodefikasi non-standar |
| `kode_barang_lama` | String(30) | Ya | Kode barang non-standar yang saat ini digunakan |
| `nama_barang_lama` | String(255) | Ya | Nama barang dengan kode lama |
| `kode_barang_baru_id` | UUID (FK→E04) | Tidak | Referensi ke kode barang standar (null jika belum di-mapping) |
| `status_mapping` | Enum | Ya | `BELUM_MAPPING`, `DIUSULKAN`, `DIVERIFIKASI`, `DITERAPKAN`, `DITOLAK` |
| `catatan_mapping` | Text | Tidak | Keterangan atau alasan mapping |
| `nup` | String(20) | Tidak | Nomor Urut Pendaftaran BMN terkait |
| `diusulkan_oleh` | UUID | Tidak | ID pengguna pengusul |
| `diusulkan_pada` | Timestamp | Tidak | Waktu pengajuan usulan |
| `diverifikasi_oleh` | UUID | Tidak | ID pengguna verifikator |
| `diverifikasi_pada` | Timestamp | Tidak | Waktu verifikasi |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |

### 5.3 Aktivitas (Activities)

#### ACT-M01: Pengelolaan Kodefikasi BMN

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-M01.1 | Tambah kode barang baru | Admin Pusat | Menambahkan kode barang baru ke dalam register kodefikasi standar |
| ACT-M01.2 | Ubah kode barang | Admin Pusat | Mengubah detail kode barang yang sudah ada |
| ACT-M01.3 | Nonaktifkan kode barang | Admin Pusat | Menonaktifkan kode barang yang sudah tidak berlaku |
| ACT-M01.4 | Impor kode barang | Admin Pusat | Impor massal kode barang dari file (Excel/CSV) |
| ACT-M01.5 | Lihat daftar kode barang | Semua Pengguna | Melihat dan mencari kode barang |
| ACT-M01.6 | Unduh daftar kode barang | Admin Pusat, Admin Wilayah | Mengunduh daftar kode barang dalam format Excel |

#### ACT-M02: Pengelolaan Standar Spesifikasi

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-M02.1 | Tambah standar spesifikasi | Admin Pusat | Mendefinisikan standar spesifikasi baru untuk jenis BMN tertentu |
| ACT-M02.2 | Ubah standar spesifikasi | Admin Pusat | Mengubah standar spesifikasi yang ada |
| ACT-M02.3 | Versioning standar | Admin Pusat | Membuat versi baru standar untuk tahun anggaran baru (standar lama tetap tersimpan sebagai histori) |
| ACT-M02.4 | Lihat standar spesifikasi | Semua Pengguna | Melihat dan mencari standar spesifikasi berdasarkan jenis BMN |

#### ACT-M03: Pengelolaan Standar Jumlah

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-M03.1 | Tambah standar jumlah | Admin Pusat | Mendefinisikan standar jumlah kebutuhan BMN |
| ACT-M03.2 | Ubah standar jumlah | Admin Pusat | Mengubah standar jumlah |
| ACT-M03.3 | Simulasi perhitungan | Admin Pusat, Admin Wilayah | Menjalankan simulasi perhitungan kebutuhan berdasarkan standar jumlah terhadap data satker/pegawai |
| ACT-M03.4 | Lihat standar jumlah | Semua Pengguna | Melihat standar jumlah berdasarkan jenis BMN dan peruntukan |

#### ACT-M04: Mapping Kodefikasi Non-Standar

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-M04.1 | Identifikasi kode non-standar | Sistem (otomatis) | Mendeteksi kode barang dari MonSAKTI yang tidak sesuai standar |
| ACT-M04.2 | Usulkan mapping | Operator Satker | Mengusulkan pemetaan kode barang lama ke kode standar |
| ACT-M04.3 | Verifikasi mapping | Admin Pusat | Memverifikasi dan menyetujui/menolak usulan mapping |
| ACT-M04.4 | Terapkan mapping | Admin Pusat | Menerapkan mapping yang sudah diverifikasi |
| ACT-M04.5 | Monitoring mapping | Admin Pusat, Admin Wilayah | Melihat progress mapping kodefikasi seluruh satker |

### 5.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-M001 | Tinggi | Sistem harus menyediakan CRUD lengkap untuk data kode barang dengan validasi format kode sesuai standar SIMAK BMN | Admin Pusat |
| REQ-M002 | Tinggi | Sistem harus mendukung pencarian kode barang berdasarkan kode, nama, kelompok, dan sub kelompok dengan fitur autocomplete | Semua |
| REQ-M003 | Tinggi | Sistem harus menyediakan CRUD lengkap untuk standar spesifikasi dengan dukungan field dinamis (JSONB) untuk mengakomodasi variasi spesifikasi antar jenis BMN | Admin Pusat |
| REQ-M004 | Tinggi | Sistem harus mendukung versioning standar spesifikasi per tahun anggaran, sehingga standar lama tetap dapat diakses sebagai histori | Admin Pusat |
| REQ-M005 | Tinggi | Sistem harus menyediakan CRUD lengkap untuk standar jumlah dengan dukungan berbagai tipe perhitungan (per satker, per pegawai, per jabatan, per golongan, per luas kantor) | Admin Pusat |
| REQ-M006 | Tinggi | Sistem harus dapat menjalankan simulasi perhitungan kebutuhan berdasarkan standar jumlah terhadap data aktual satker dan pegawai dari layanan integrasi | Admin Pusat |
| REQ-M007 | Tinggi | Sistem harus mendeteksi secara otomatis kode barang dari data MonSAKTI yang tidak sesuai dengan kodefikasi standar yang terdaftar di SIMPEL | Sistem |
| REQ-M008 | Sedang | Sistem harus menyediakan antarmuka untuk mengusulkan dan memverifikasi mapping kode barang non-standar ke kode standar | Operator Satker, Admin Pusat |
| REQ-M009 | Sedang | Sistem harus menyediakan dashboard monitoring progress mapping kodefikasi per satker dan per wilayah | Admin Pusat, Admin Wilayah |
| REQ-M010 | Sedang | Sistem harus mendukung impor massal data referensi dari file Excel/CSV dengan validasi data dan laporan error | Admin Pusat |
| REQ-M011 | Sedang | Sistem harus mendukung ekspor data referensi ke format Excel untuk kebutuhan offline | Admin Pusat, Admin Wilayah |
| REQ-M012 | Tinggi | Sistem harus mempublikasikan event ketika terjadi perubahan pada data referensi (standar spesifikasi, standar jumlah, kodefikasi) agar layanan konsumer dapat memperbarui data lokalnya | Sistem |
| REQ-M013 | Rendah | Sistem harus menyediakan API untuk perbandingan standar antar versi tahun (diff view) | Admin Pusat |
| REQ-M014 | Tinggi | Setiap perubahan data master harus tercatat dalam log audit (siapa, kapan, perubahan apa) | Sistem |
| REQ-M015 | Sedang | Sistem harus menyediakan fitur filter pada kode barang berdasarkan atribut `is_sbsk` untuk membedakan kode barang yang dikelola SIMPEL dan yang dikelola SIMAN | Semua |

### 5.5 Siklus Hidup (Lifecycle)

#### Lifecycle Standar Spesifikasi/Jumlah

```
DRAFT → BERLAKU → DIREVISI → BERLAKU (versi baru)
                → TIDAK_BERLAKU (jika dicabut tanpa pengganti)
```

#### Lifecycle Mapping Kodefikasi

```
BELUM_MAPPING → DIUSULKAN → DIVERIFIKASI → DITERAPKAN
                           → DITOLAK → DIUSULKAN (revisi)
```

### 5.6 User Stories

| ID | Story |
|---|---|
| US-M001 | Sebagai Admin Pusat, saya ingin menambahkan kode barang baru ke dalam register kodefikasi sehingga seluruh satker dapat menggunakan kode yang seragam. |
| US-M002 | Sebagai Admin Pusat, saya ingin mendefinisikan standar spesifikasi untuk jenis BMN tertentu (misal: laptop jaksa fungsional) sehingga seluruh satker memiliki acuan spesifikasi yang sama saat mengajukan kebutuhan. |
| US-M003 | Sebagai Admin Pusat, saya ingin menetapkan standar jumlah kebutuhan laptop per jaksa sehingga perhitungan kebutuhan dapat diotomatisasi berdasarkan jumlah jaksa di setiap satker. |
| US-M004 | Sebagai Admin Pusat, saya ingin membuat versi baru standar spesifikasi untuk tahun anggaran berikutnya tanpa menghapus standar tahun sebelumnya sehingga ada histori perubahan standar. |
| US-M005 | Sebagai Operator Satker, saya ingin mengusulkan pemetaan kode barang yang non-standar di satker saya ke kode standar sehingga data BMN satker menjadi konsisten secara nasional. |
| US-M006 | Sebagai Admin Pusat, saya ingin melihat dashboard progress mapping kodefikasi per wilayah sehingga saya tahu satker mana yang masih memiliki banyak kode non-standar. |
| US-M007 | Sebagai Admin Pusat, saya ingin menjalankan simulasi perhitungan kebutuhan berdasarkan standar jumlah terhadap data aktual sehingga saya bisa memvalidasi apakah standar yang ditetapkan sudah realistis. |
| US-M008 | Sebagai pengguna, saya ingin mencari kode barang dengan autocomplete sehingga saya tidak perlu menghafal kode dan bisa menemukan barang dengan cepat. |
| US-M009 | Sebagai Admin Pusat, saya ingin mengimpor data kodefikasi dari file Excel sehingga saya tidak perlu menginput satu per satu ketika ada pembaruan massal. |
| US-M010 | Sebagai Auditor, saya ingin melihat riwayat perubahan data master sehingga saya bisa memastikan perubahan standar dilakukan oleh pihak yang berwenang. |

---

## 6. Layanan 2 — Analisis Kebutuhan BMN (`kebutuhan`)

### 6.1 Deskripsi Layanan

Layanan Analisis Kebutuhan BMN bertanggung jawab atas pengumpulan, pengolahan, analisis, dan penyajian data kebutuhan BMN non-SBSK di seluruh satuan kerja Kejaksaan. Layanan ini mencakup tiga sub-domain utama: kebutuhan BMN umum, kebutuhan pakaian dinas pegawai, dan roadmap pembangunan sarana dan prasarana 5 tahunan. Kebutuhan BMN yang sudah memiliki SBSK sesuai PMK tetap dikelola melalui SIMAN v2.

### 6.2 Entitas (Entities)

#### E07 — Kebutuhan BMN

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `satker_id` | UUID | Ya | Referensi ke satuan kerja |
| `tahun_anggaran` | Integer | Ya | Tahun anggaran kebutuhan |
| `kode_barang_id` | UUID (FK→E04) | Ya | Referensi ke kode barang |
| `standar_spesifikasi_id` | UUID (FK→E05) | Tidak | Referensi ke standar spesifikasi |
| `standar_jumlah_id` | UUID (FK→E06) | Tidak | Referensi ke standar jumlah yang digunakan |
| `jumlah_standar` | Decimal | Ya | Jumlah sesuai standar (hasil perhitungan otomatis) |
| `jumlah_eksisting` | Decimal | Ya | Jumlah BMN yang sudah ada (dari MonSAKTI) |
| `jumlah_kondisi_baik` | Decimal | Tidak | Jumlah BMN eksisting dengan kondisi baik |
| `jumlah_kondisi_rusak_ringan` | Decimal | Tidak | Jumlah BMN eksisting dengan kondisi rusak ringan |
| `jumlah_kondisi_rusak_berat` | Decimal | Tidak | Jumlah BMN eksisting dengan kondisi rusak berat |
| `jumlah_kurang` | Decimal | Ya | Selisih kebutuhan (standar - eksisting kondisi baik) |
| `jumlah_diusulkan` | Decimal | Ya | Jumlah yang diusulkan oleh satker |
| `justifikasi` | Text | Tidak | Alasan/justifikasi kebutuhan |
| `prioritas` | Enum | Ya | `SANGAT_MENDESAK`, `MENDESAK`, `NORMAL`, `DAPAT_DITUNDA` |
| `status` | Enum | Ya | Status pengajuan kebutuhan |
| `catatan_verifikator` | Text | Tidak | Catatan dari verifikator |
| `jumlah_disetujui` | Decimal | Tidak | Jumlah yang disetujui (setelah verifikasi) |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |
| `version` | Integer | Ya | Versi data |

#### E08 — Kebutuhan Pakaian Dinas

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `satker_id` | UUID | Ya | Referensi ke satuan kerja |
| `tahun_anggaran` | Integer | Ya | Tahun anggaran |
| `pegawai_id` | UUID | Ya | Referensi ke data pegawai (MySIMKARI) |
| `nip` | String(18) | Ya | NIP pegawai |
| `nama_pegawai` | String(255) | Ya | Nama pegawai (snapshot saat pengajuan) |
| `jabatan` | String(100) | Ya | Jabatan saat pengajuan |
| `golongan` | String(10) | Ya | Golongan/pangkat saat pengajuan |
| `jenis_pakaian_dinas` | Enum | Ya | `PDH`, `PDL`, `TOGA`, `PAKAIAN_SIDANG`, `PAKAIAN_UPACARA`, `PAKAIAN_DINAS_KHUSUS`, `LAINNYA` |
| `ukuran` | JSONB | Tidak | Detail ukuran (baju, celana, sepatu, dll.) |
| `terakhir_diterima` | Date | Tidak | Tanggal terakhir menerima pakaian dinas jenis ini |
| `masa_pakai_bulan` | Integer | Tidak | Masa pakai standar (dari standar spesifikasi) |
| `is_eligible` | Boolean | Ya | Apakah pegawai memenuhi syarat (berdasarkan masa pakai) |
| `jumlah_diusulkan` | Integer | Ya | Jumlah yang diusulkan (biasanya 1 stel) |
| `status` | Enum | Ya | Status pengajuan |
| `prioritas` | Enum | Ya | Tingkat prioritas |
| `catatan` | Text | Tidak | Catatan tambahan |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |

#### E09 — Roadmap Sarana Prasarana

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `satker_id` | UUID | Ya | Referensi ke satuan kerja |
| `periode_mulai` | Integer | Ya | Tahun awal periode 5 tahunan |
| `periode_akhir` | Integer | Ya | Tahun akhir periode 5 tahunan |
| `kode_barang_id` | UUID (FK→E04) | Ya | Referensi ke kode barang |
| `kebutuhan_bmn_id` | UUID (FK→E07) | Tidak | Referensi ke data kebutuhan BMN sumber |
| `tahun_rencana` | Integer | Ya | Tahun rencana pemenuhan dalam periode |
| `jumlah_kebutuhan` | Decimal | Ya | Jumlah kebutuhan pada tahun rencana |
| `jumlah_terpenuhi` | Decimal | Tidak | Jumlah yang terpenuhi (realisasi, diisi setelah tahun berjalan) |
| `sumber_dana` | Enum | Tidak | `APBN`, `HIBAH`, `PNBP`, `LAINNYA` |
| `estimasi_anggaran` | Decimal | Tidak | Estimasi anggaran yang dibutuhkan |
| `realisasi_anggaran` | Decimal | Tidak | Realisasi anggaran (diisi setelah tahun berjalan) |
| `status_pemenuhan` | Enum | Ya | `BELUM`, `SEBAGIAN`, `TERPENUHI`, `DIBATALKAN` |
| `catatan` | Text | Tidak | Catatan |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |

#### E15 — Riwayat Pemenuhan

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kebutuhan_bmn_id` | UUID (FK→E07) | Tidak | Referensi ke kebutuhan BMN |
| `roadmap_id` | UUID (FK→E09) | Tidak | Referensi ke roadmap |
| `satker_id` | UUID | Ya | Referensi ke satuan kerja |
| `tahun_anggaran` | Integer | Ya | Tahun anggaran pemenuhan |
| `kode_barang_id` | UUID (FK→E04) | Ya | Referensi ke kode barang |
| `jumlah_terpenuhi` | Decimal | Ya | Jumlah yang terpenuhi |
| `sumber_data` | Enum | Ya | `MONSAKTI`, `INPUT_MANUAL`, `SIMAN` |
| `nomor_sp2d` | String(50) | Tidak | Nomor SP2D (jika dari MonSAKTI) |
| `tanggal_pemenuhan` | Date | Ya | Tanggal pemenuhan/perolehan |
| `catatan` | Text | Tidak | Catatan |
| `created_at` | Timestamp | Ya | Waktu pembuatan |

### 6.3 Aktivitas (Activities)

#### ACT-K01: Pengumpulan Kebutuhan BMN

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-K01.1 | Inisiasi periode pengumpulan | Admin Pusat | Membuka periode pengumpulan kebutuhan BMN untuk tahun anggaran tertentu |
| ACT-K01.2 | Perhitungan otomatis kebutuhan | Sistem | Menghitung kebutuhan berdasarkan standar jumlah dikurangi eksisting dari MonSAKTI |
| ACT-K01.3 | Input/revisi kebutuhan satker | Operator Satker | Merevisi hasil perhitungan otomatis dengan justifikasi |
| ACT-K01.4 | Pengajuan kebutuhan satker | Operator Satker | Mengajukan kebutuhan satker ke verifikator |
| ACT-K01.5 | Verifikasi kebutuhan | Verifikator, Admin Wilayah | Memverifikasi kewajaran dan kelengkapan data kebutuhan |
| ACT-K01.6 | Rekapitulasi kebutuhan wilayah | Admin Wilayah | Merekapitulasi kebutuhan seluruh satker di wilayah |
| ACT-K01.7 | Konsolidasi kebutuhan nasional | Admin Pusat | Mengonsolidasikan kebutuhan seluruh wilayah |
| ACT-K01.8 | Penutupan periode pengumpulan | Admin Pusat | Menutup periode pengumpulan dan memfinalisasi data |

#### ACT-K02: Kebutuhan Pakaian Dinas

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-K02.1 | Inisiasi periode pakaian dinas | Admin Pusat | Membuka periode pengumpulan kebutuhan pakaian dinas |
| ACT-K02.2 | Perhitungan eligibilitas otomatis | Sistem | Menghitung eligibilitas pegawai berdasarkan masa pakai terakhir dan jabatan/golongan dari MySIMKARI |
| ACT-K02.3 | Input data kebutuhan pakaian dinas | Operator Satker | Menginput/merevisi kebutuhan pakaian dinas per pegawai, termasuk ukuran |
| ACT-K02.4 | Pengajuan kebutuhan pakaian dinas | Operator Satker | Mengajukan kebutuhan pakaian dinas satker |
| ACT-K02.5 | Verifikasi kebutuhan pakaian dinas | Verifikator | Memverifikasi kebutuhan pakaian dinas |
| ACT-K02.6 | Rekapitulasi per jenis dan ukuran | Admin Pusat | Merekapitulasi kebutuhan per jenis pakaian dinas dan per ukuran untuk kebutuhan pengadaan |

#### ACT-K03: Roadmap Sarana Prasarana

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-K03.1 | Penyusunan roadmap 5 tahunan | Admin Pusat | Menyusun roadmap berdasarkan data kebutuhan dan prioritas |
| ACT-K03.2 | Distribusi target tahunan | Admin Pusat | Mendistribusikan target pemenuhan per tahun dalam periode 5 tahun |
| ACT-K03.3 | Input realisasi pemenuhan | Operator Satker, Admin Pusat | Menginput data realisasi pemenuhan BMN pada tahun berjalan |
| ACT-K03.4 | Sinkronisasi dengan MonSAKTI | Sistem | Mencocokkan data realisasi dengan data perolehan BMN baru dari MonSAKTI |
| ACT-K03.5 | Monitoring capaian roadmap | Admin Pusat, Pimpinan | Memantau progress capaian roadmap: target vs realisasi |
| ACT-K03.6 | Revisi roadmap | Admin Pusat | Merevisi roadmap berdasarkan kondisi terkini |

#### ACT-K04: Analisis Data Kebutuhan

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-K04.1 | Analisis gap kebutuhan vs eksisting | Sistem | Menganalisis selisih antara kebutuhan standar dan BMN eksisting per satker |
| ACT-K04.2 | Analisis tren kebutuhan | Sistem | Menganalisis tren kebutuhan dari tahun ke tahun |
| ACT-K04.3 | Analisis prioritas pemenuhan | Sistem | Mengurutkan kebutuhan berdasarkan prioritas dan urgensi |
| ACT-K04.4 | Penyajian bank data kebutuhan | Sistem | Menampilkan bank data kebutuhan terkini seluruh satker yang dapat difilter dan di-drill-down |

### 6.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-K001 | Tinggi | Sistem harus mampu menghitung kebutuhan BMN secara otomatis berdasarkan standar jumlah (dari layanan master) dikurangi jumlah eksisting kondisi baik (dari MonSAKTI via layanan integrasi) | Sistem |
| REQ-K002 | Tinggi | Sistem harus menyediakan mekanisme periode pengumpulan kebutuhan yang dapat dibuka dan ditutup oleh Admin Pusat, dengan batas waktu (deadline) per satker | Admin Pusat |
| REQ-K003 | Tinggi | Operator Satker harus dapat merevisi hasil perhitungan otomatis dengan kewajiban memberikan justifikasi tertulis untuk setiap penyimpangan dari standar | Operator Satker |
| REQ-K004 | Tinggi | Sistem harus mendukung alur persetujuan berjenjang: Operator Satker → Verifikator Satker → Admin Wilayah (Kejati) → Admin Pusat (Kejagung) melalui layanan workflow | Semua |
| REQ-K005 | Tinggi | Sistem harus menyediakan bank data kebutuhan BMN terkini seluruh satker yang dapat difilter berdasarkan wilayah, jenis BMN, prioritas, status, dan tahun anggaran | Semua |
| REQ-K006 | Tinggi | Sistem harus menghitung eligibilitas pegawai untuk menerima pakaian dinas berdasarkan jabatan, golongan, dan masa pakai terakhir secara otomatis | Sistem |
| REQ-K007 | Tinggi | Sistem harus menyediakan rekapitulasi kebutuhan pakaian dinas per jenis, per ukuran, dan per satker untuk mendukung proses pengadaan | Admin Pusat |
| REQ-K008 | Sedang | Sistem harus menyediakan fitur roadmap 5 tahunan yang menampilkan rencana pemenuhan kebutuhan per tahun dan perbandingannya dengan realisasi | Admin Pusat, Pimpinan |
| REQ-K009 | Sedang | Sistem harus dapat menyinkronkan data realisasi pemenuhan BMN dengan data perolehan BMN baru dari MonSAKTI secara periodik | Sistem |
| REQ-K010 | Sedang | Sistem harus menyediakan analisis gap kebutuhan vs eksisting dengan visualisasi grafik per satker, per wilayah, dan secara nasional | Admin Pusat, Pimpinan |
| REQ-K011 | Sedang | Sistem harus menyediakan analisis tren kebutuhan dari tahun ke tahun untuk mendukung perencanaan jangka panjang | Admin Pusat, Pimpinan |
| REQ-K012 | Sedang | Sistem harus mendukung fitur lock data setelah periode pengumpulan ditutup; revisi setelah penutupan hanya dapat dilakukan dengan persetujuan Admin Pusat | Admin Pusat |
| REQ-K013 | Tinggi | Sistem harus menyimpan snapshot data pegawai (jabatan, golongan) pada saat pengajuan kebutuhan pakaian dinas untuk keperluan audit, karena data pegawai dapat berubah sewaktu-waktu di MySIMKARI | Sistem |
| REQ-K014 | Sedang | Sistem harus mendukung ekspor data kebutuhan ke format Excel untuk kebutuhan pengolahan offline dan pelaporan | Semua |
| REQ-K015 | Rendah | Sistem harus menyediakan fitur perbandingan kebutuhan antar satker sejenis (benchmarking) untuk deteksi anomali | Admin Pusat |
| REQ-K016 | Tinggi | Sistem harus menyediakan mekanisme untuk membedakan kebutuhan BMN pengadaan baru (belum pernah ada) dan kebutuhan BMN penggantian (menggantikan yang rusak berat) | Operator Satker |

### 6.5 Siklus Hidup (Lifecycle)

#### Lifecycle Kebutuhan BMN

```
DRAFT → DIAJUKAN → VERIFIKASI_WILAYAH → VERIFIKASI_PUSAT → DISETUJUI
                  → DIKEMBALIKAN → DRAFT (revisi)
                                   → DITOLAK

DISETUJUI → TERPENUHI_SEBAGIAN → TERPENUHI
          → DIBATALKAN
```

#### Lifecycle Kebutuhan Pakaian Dinas

```
DRAFT → DIAJUKAN → DIVERIFIKASI → DISETUJUI → DIDISTRIBUSIKAN
                  → DIKEMBALIKAN → DRAFT
                  → DITOLAK
```

#### Lifecycle Roadmap Sarpras

```
PENYUSUNAN → DITETAPKAN → BERJALAN → EVALUASI_TAHUNAN → BERJALAN (tahun berikutnya)
                                                       → DIREVISI → BERJALAN
           → SELESAI (periode berakhir)
```

### 6.6 User Stories

| ID | Story |
|---|---|
| US-K001 | Sebagai Admin Pusat, saya ingin membuka periode pengumpulan kebutuhan BMN untuk tahun anggaran 2027 dengan batas waktu pengajuan 31 Maret 2026 sehingga seluruh satker memiliki waktu yang cukup untuk menyusun kebutuhan. |
| US-K002 | Sebagai Operator Satker, saya ingin melihat hasil perhitungan otomatis kebutuhan BMN satker saya berdasarkan standar jumlah dikurangi eksisting sehingga saya memiliki baseline yang akurat untuk diajukan. |
| US-K003 | Sebagai Operator Satker, saya ingin merevisi jumlah yang diusulkan dari perhitungan otomatis dengan menyertakan justifikasi sehingga kebutuhan riil di lapangan dapat terakomodasi. |
| US-K004 | Sebagai Admin Wilayah (Kejati), saya ingin melihat rekapitulasi kebutuhan seluruh Kejari dan Cabjari di wilayah saya sehingga saya dapat memverifikasi kewajaran data sebelum diteruskan ke pusat. |
| US-K005 | Sebagai Admin Pusat, saya ingin melihat bank data kebutuhan BMN seluruh satker secara nasional yang dapat saya filter berdasarkan jenis BMN, wilayah, dan prioritas sehingga saya dapat mengambil keputusan alokasi anggaran yang tepat. |
| US-K006 | Sebagai Operator Satker, saya ingin menginput kebutuhan pakaian dinas per pegawai beserta ukurannya sehingga pengadaan pakaian dinas sesuai kebutuhan masing-masing pegawai. |
| US-K007 | Sebagai Sistem, saya ingin menghitung secara otomatis pegawai mana yang eligible menerima pakaian dinas baru berdasarkan tanggal terakhir penerimaan dan masa pakai standar sehingga tidak ada pegawai yang menerima sebelum waktunya. |
| US-K008 | Sebagai Admin Pusat, saya ingin merekapitulasi kebutuhan pakaian dinas per jenis dan per ukuran secara nasional sehingga proses pengadaan dapat dilakukan secara efisien. |
| US-K009 | Sebagai Admin Pusat, saya ingin menyusun roadmap pemenuhan sarpras 5 tahunan yang menunjukkan rencana per tahun sehingga perencanaan anggaran jangka menengah menjadi lebih terarah. |
| US-K010 | Sebagai Pimpinan, saya ingin melihat perbandingan target roadmap vs realisasi per tahun sehingga saya dapat mengevaluasi efektivitas perencanaan. |
| US-K011 | Sebagai Operator Satker, saya ingin membedakan antara kebutuhan BMN baru dan kebutuhan penggantian sehingga prioritas pengadaan dapat ditentukan dengan tepat. |

---

## 7. Layanan 3 — Pemakaian BMN (`pemakaian`)

### 7.1 Deskripsi Layanan

Layanan Pemakaian BMN mengelola seluruh siklus hidup izin pemakaian Barang Milik Negara oleh pegawai Kejaksaan. BMN yang termasuk dalam cakupan layanan ini meliputi kendaraan dinas, rumah negara/rumah dinas, laptop/komputer, dan BMN lainnya yang memerlukan izin pemakaian individual. Layanan ini menangani pengajuan, persetujuan/penolakan, penerbitan izin, perpanjangan izin, pencabutan izin, dan monitoring pemakaian.

### 7.2 Entitas (Entities)

#### E10 — Izin Pemakaian BMN

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `nomor_izin` | String(50) | Ya (auto) | Nomor izin pemakaian (di-generate otomatis) |
| `jenis_bmn` | Enum | Ya | `KENDARAAN_DINAS`, `RUMAH_NEGARA`, `LAPTOP`, `KOMPUTER_DESKTOP`, `PERALATAN_LAINNYA` |
| `bmn_id` | String(50) | Ya | Identifier BMN di MonSAKTI (kode barang + NUP) |
| `kode_barang` | String(30) | Ya | Kode barang |
| `nup` | String(20) | Ya | Nomor Urut Pendaftaran |
| `deskripsi_bmn` | String(500) | Ya | Deskripsi BMN (misal: Toyota Avanza 2020, Nopol B 1234 ABC) |
| `satker_id` | UUID | Ya | Satker pemilik BMN |
| `pemohon_id` | UUID | Ya | ID pegawai pemohon |
| `pemohon_nip` | String(18) | Ya | NIP pemohon |
| `pemohon_nama` | String(255) | Ya | Nama pemohon |
| `pemohon_jabatan` | String(100) | Ya | Jabatan pemohon saat pengajuan |
| `pemohon_golongan` | String(10) | Ya | Golongan pemohon saat pengajuan |
| `tujuan_pemakaian` | Text | Ya | Tujuan/alasan pemakaian |
| `tanggal_mulai` | Date | Ya | Tanggal mulai pemakaian |
| `tanggal_berakhir` | Date | Ya | Tanggal berakhir izin pemakaian |
| `status` | Enum | Ya | Status izin pemakaian |
| `tanggal_disetujui` | Timestamp | Tidak | Tanggal disetujui |
| `disetujui_oleh` | UUID | Tidak | ID pejabat yang menyetujui |
| `tanggal_dicabut` | Timestamp | Tidak | Tanggal pencabutan (jika dicabut) |
| `dicabut_oleh` | UUID | Tidak | ID pejabat yang mencabut |
| `alasan_pencabutan` | Text | Tidak | Alasan pencabutan |
| `alasan_penolakan` | Text | Tidak | Alasan penolakan (jika ditolak) |
| `izin_sebelumnya_id` | UUID (FK→self) | Tidak | Referensi ke izin sebelumnya (untuk perpanjangan) |
| `nomor_dokumen_izin` | String(100) | Tidak | Nomor surat/dokumen izin yang diterbitkan |
| `dokumen_id` | UUID (FK→E12) | Tidak | Referensi ke dokumen izin yang di-generate |
| `catatan` | Text | Tidak | Catatan tambahan |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |
| `version` | Integer | Ya | Versi data |

#### Sub-entitas: Detail Pemakaian Kendaraan

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `izin_pemakaian_id` | UUID (FK→E10) | Ya | Referensi ke izin pemakaian |
| `nomor_polisi` | String(20) | Ya | Nomor polisi kendaraan |
| `jenis_kendaraan` | Enum | Ya | `RODA_4_JABATAN`, `RODA_4_OPERASIONAL`, `RODA_2` |
| `merk_tipe` | String(100) | Ya | Merk dan tipe kendaraan |
| `tahun_pembuatan` | Integer | Ya | Tahun pembuatan kendaraan |
| `nomor_rangka` | String(50) | Tidak | Nomor rangka |
| `nomor_mesin` | String(50) | Tidak | Nomor mesin |
| `kilometer_awal` | Integer | Tidak | Kilometer saat mulai pemakaian |

#### Sub-entitas: Detail Pemakaian Rumah Negara

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `izin_pemakaian_id` | UUID (FK→E10) | Ya | Referensi ke izin pemakaian |
| `alamat` | Text | Ya | Alamat lengkap rumah negara |
| `golongan_rumah` | Enum | Ya | `GOLONGAN_I`, `GOLONGAN_II`, `GOLONGAN_III` |
| `luas_tanah_m2` | Decimal | Tidak | Luas tanah (m²) |
| `luas_bangunan_m2` | Decimal | Tidak | Luas bangunan (m²) |
| `jumlah_penghuni` | Integer | Tidak | Jumlah penghuni |
| `sewa_sip` | Decimal | Tidak | Sewa/Surat Izin Penghunian |

### 7.3 Aktivitas (Activities)

#### ACT-P01: Pengajuan Pemakaian BMN

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-P01.1 | Cek ketersediaan BMN | Pegawai, Operator Satker | Memeriksa BMN yang tersedia dan belum dipakai/dipinjamkan |
| ACT-P01.2 | Buat pengajuan pemakaian | Pegawai, Operator Satker | Membuat pengajuan pemakaian BMN dengan mengisi formulir lengkap |
| ACT-P01.3 | Unggah dokumen pendukung | Pegawai, Operator Satker | Mengunggah dokumen pendukung (SK jabatan, dll.) |
| ACT-P01.4 | Kirim pengajuan | Pegawai, Operator Satker | Mengirimkan pengajuan ke verifikator melalui workflow |

#### ACT-P02: Persetujuan/Penolakan

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-P02.1 | Review pengajuan | Verifikator | Memeriksa kelengkapan dan kewajaran pengajuan |
| ACT-P02.2 | Cek eligibilitas pemohon | Verifikator, Sistem | Memverifikasi hak pemohon atas BMN tersebut (jabatan, golongan) |
| ACT-P02.3 | Setujui pengajuan | Verifikator, Pimpinan | Menyetujui pengajuan pemakaian |
| ACT-P02.4 | Tolak pengajuan | Verifikator, Pimpinan | Menolak pengajuan dengan menyertakan alasan |
| ACT-P02.5 | Kembalikan pengajuan | Verifikator | Mengembalikan pengajuan untuk diperbaiki |

#### ACT-P03: Penerbitan Izin

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-P03.1 | Generate nomor izin | Sistem | Membuat nomor izin pemakaian secara otomatis |
| ACT-P03.2 | Generate dokumen izin | Sistem | Membuat dokumen surat izin pemakaian melalui layanan dokumen |
| ACT-P03.3 | Tandatangani dokumen | Pimpinan | Menandatangani dokumen izin (digital atau manual) |
| ACT-P03.4 | Kirim notifikasi | Sistem | Mengirim notifikasi ke pemohon bahwa izin telah terbit |

#### ACT-P04: Perpanjangan Izin

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-P04.1 | Reminder masa berlaku | Sistem | Mengirim reminder otomatis sebelum izin berakhir (H-30, H-14, H-7) |
| ACT-P04.2 | Ajukan perpanjangan | Pegawai, Operator Satker | Mengajukan perpanjangan izin pemakaian |
| ACT-P04.3 | Proses perpanjangan | Verifikator, Pimpinan | Memproses perpanjangan melalui workflow persetujuan |
| ACT-P04.4 | Terbitkan izin baru | Sistem | Menerbitkan izin baru yang merujuk pada izin sebelumnya |

#### ACT-P05: Pencabutan Izin

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-P05.1 | Inisiasi pencabutan | Admin Pusat, Admin Wilayah, Pimpinan | Menginisiasi pencabutan izin pemakaian |
| ACT-P05.2 | Notifikasi pencabutan | Sistem | Mengirim notifikasi ke pemakai tentang rencana pencabutan |
| ACT-P05.3 | Proses pencabutan | Pimpinan | Memproses dan mengesahkan pencabutan |
| ACT-P05.4 | Generate surat pencabutan | Sistem | Membuat surat pencabutan izin pemakaian |
| ACT-P05.5 | Update status BMN | Sistem | Mengubah status BMN menjadi tersedia kembali |

#### ACT-P06: Monitoring Pemakaian

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-P06.1 | Lihat daftar pemakaian aktif | Admin, Pimpinan | Melihat seluruh izin pemakaian yang masih aktif |
| ACT-P06.2 | Lihat riwayat pemakaian per BMN | Admin, Pimpinan | Melihat riwayat siapa saja yang pernah memakai suatu BMN |
| ACT-P06.3 | Lihat riwayat pemakaian per pegawai | Admin, Pimpinan | Melihat BMN apa saja yang pernah/sedang dipakai oleh seorang pegawai |
| ACT-P06.4 | Deteksi izin kadaluarsa | Sistem | Mendeteksi dan menandai izin yang sudah melewati masa berlaku tanpa perpanjangan |
| ACT-P06.5 | Laporan utilisasi BMN | Admin, Pimpinan | Menghasilkan laporan utilisasi BMN (berapa persen yang sedang dipakai vs idle) |

### 7.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-P001 | Tinggi | Sistem harus menyediakan formulir pengajuan pemakaian BMN yang dinamis sesuai jenis BMN (kendaraan, rumah negara, laptop) dengan field yang berbeda-beda | Pegawai, Operator |
| REQ-P002 | Tinggi | Sistem harus menampilkan daftar BMN yang tersedia (belum ada izin pemakaian aktif) untuk dipilih saat pengajuan, berdasarkan data dari MonSAKTI | Pegawai, Operator |
| REQ-P003 | Tinggi | Sistem harus memvalidasi bahwa satu BMN hanya dapat memiliki satu izin pemakaian aktif pada satu waktu (kecuali untuk BMN yang memang bisa dipakai bersama) | Sistem |
| REQ-P004 | Tinggi | Sistem harus mendukung alur persetujuan yang dapat dikonfigurasi melalui layanan workflow, dengan minimal dua level approval | Verifikator, Pimpinan |
| REQ-P005 | Tinggi | Sistem harus men-generate nomor izin pemakaian secara otomatis dengan format yang konsisten dan unik | Sistem |
| REQ-P006 | Tinggi | Sistem harus men-generate dokumen surat izin pemakaian BMN dalam format PDF melalui layanan dokumen, dengan data yang terisi otomatis dari pengajuan | Sistem |
| REQ-P007 | Tinggi | Sistem harus mengirim reminder otomatis kepada pemegang izin pada H-30, H-14, dan H-7 sebelum masa berlaku izin berakhir melalui layanan notifikasi | Sistem |
| REQ-P008 | Tinggi | Sistem harus menyediakan mekanisme perpanjangan izin yang merujuk pada izin sebelumnya sehingga riwayat perpanjangan dapat dilacak | Pegawai, Operator |
| REQ-P009 | Tinggi | Sistem harus menyediakan mekanisme pencabutan izin dengan kewajiban menyertakan alasan pencabutan dan penerbitan surat pencabutan | Pimpinan |
| REQ-P010 | Tinggi | Sistem harus secara otomatis mengubah status izin menjadi `KADALUARSA` jika melewati tanggal berakhir tanpa perpanjangan | Sistem |
| REQ-P011 | Sedang | Sistem harus menyediakan dashboard monitoring pemakaian BMN aktif yang dapat difilter berdasarkan jenis BMN, satker, dan status | Admin, Pimpinan |
| REQ-P012 | Sedang | Sistem harus menyediakan riwayat pemakaian per BMN dan per pegawai yang lengkap dan dapat diunduh | Admin, Pimpinan |
| REQ-P013 | Sedang | Sistem harus menyediakan laporan utilisasi BMN yang menunjukkan rasio BMN yang dipakai vs BMN idle per satker | Admin, Pimpinan |
| REQ-P014 | Sedang | Sistem harus mendukung unggah dokumen pendukung (scan SK jabatan, surat permohonan manual) sebagai lampiran pengajuan | Pegawai, Operator |
| REQ-P015 | Rendah | Sistem harus menyediakan fitur notifikasi jika terjadi mutasi/pensiun pegawai pemegang izin (dari data MySIMKARI) sehingga izin dapat segera dicabut atau dialihkan | Sistem |
| REQ-P016 | Tinggi | Setiap perubahan status izin pemakaian harus tercatat dalam audit trail (siapa, kapan, dari status apa ke status apa, alasan) | Sistem |

### 7.5 Siklus Hidup (Lifecycle)

#### Lifecycle Izin Pemakaian BMN

```
DRAFT → DIAJUKAN → DALAM_REVIEW → DISETUJUI → IZIN_TERBIT → AKTIF
                                 → DITOLAK
                  → DIKEMBALIKAN → DRAFT

AKTIF → PERPANJANGAN_DIAJUKAN → DALAM_REVIEW → DIPERPANJANG (AKTIF, izin baru)
                                              → DITOLAK_PERPANJANGAN → AKAN_BERAKHIR
      → AKAN_BERAKHIR (H-30) → KADALUARSA
      → DICABUT
```

### 7.6 User Stories

| ID | Story |
|---|---|
| US-P001 | Sebagai Pegawai, saya ingin mengajukan pemakaian kendaraan dinas roda empat yang tersedia di satker saya sehingga saya memiliki kendaraan untuk mendukung tugas kedinasan. |
| US-P002 | Sebagai Operator Satker, saya ingin melihat daftar kendaraan dinas yang belum memiliki izin pemakaian aktif sehingga saya tahu BMN mana yang dapat dialokasikan. |
| US-P003 | Sebagai Verifikator, saya ingin memeriksa kelayakan pengajuan pemakaian rumah negara berdasarkan golongan dan jabatan pemohon sehingga distribusi rumah negara sesuai ketentuan. |
| US-P004 | Sebagai Pimpinan Satker, saya ingin menyetujui atau menolak pengajuan pemakaian BMN dengan satu klik sehingga proses persetujuan tidak memakan waktu lama. |
| US-P005 | Sebagai Pegawai, saya ingin menerima notifikasi 30 hari sebelum izin pemakaian laptop saya berakhir sehingga saya dapat mengajukan perpanjangan tepat waktu. |
| US-P006 | Sebagai Pegawai, saya ingin mengajukan perpanjangan izin pemakaian tanpa harus mengisi ulang seluruh data dari awal sehingga proses perpanjangan lebih efisien. |
| US-P007 | Sebagai Pimpinan, saya ingin mencabut izin pemakaian kendaraan dinas pegawai yang telah pensiun dan menerbitkan surat pencabutan sehingga kendaraan dapat dialokasikan ke pegawai lain. |
| US-P008 | Sebagai Admin Pusat, saya ingin melihat seluruh izin pemakaian aktif secara nasional dengan filter per jenis BMN dan per wilayah sehingga saya memiliki gambaran distribusi pemakaian BMN. |
| US-P009 | Sebagai Auditor, saya ingin melihat riwayat lengkap pemakaian suatu kendaraan dinas dari pertama kali dipinjamkan hingga saat ini sehingga saya bisa memverifikasi kepatuhan prosedur. |

---

## 8. Layanan 4 — Workflow (`workflow`)

### 8.1 Deskripsi Layanan

Layanan Workflow menyediakan mesin alur kerja terpusat (centralized workflow engine) yang digunakan oleh layanan-layanan lain untuk mengelola proses persetujuan berjenjang, eskalasi, dan automasi tugas. Layanan ini bersifat generik dan dapat dikonfigurasi untuk berbagai jenis proses bisnis tanpa perubahan kode.

### 8.2 Entitas (Entities)

#### E11 — Workflow Definition

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode` | String(50) | Ya | Kode unik workflow (misal: `WF_PEMAKAIAN_KENDARAAN`) |
| `nama` | String(255) | Ya | Nama deskriptif |
| `deskripsi` | Text | Tidak | Deskripsi alur kerja |
| `versi` | Integer | Ya | Versi definisi workflow |
| `layanan_asal` | String(50) | Ya | Nama layanan yang menggunakan workflow ini |
| `steps` | JSONB | Ya | Definisi langkah-langkah (step) dalam workflow |
| `is_aktif` | Boolean | Ya | Status aktif |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |

**Struktur `steps` (JSONB):**

```json
[
  {
    "step_order": 1,
    "nama": "Verifikasi Operator",
    "tipe": "APPROVAL",
    "approver_role": "VERIFIKATOR_SATKER",
    "approver_level": "SATKER",
    "sla_jam": 48,
    "aksi_tersedia": ["SETUJUI", "TOLAK", "KEMBALIKAN"],
    "eskalasi": {
      "setelah_jam": 72,
      "ke_role": "PIMPINAN_SATKER"
    }
  },
  {
    "step_order": 2,
    "nama": "Persetujuan Pimpinan Satker",
    "tipe": "APPROVAL",
    "approver_role": "PIMPINAN_SATKER",
    "approver_level": "SATKER",
    "sla_jam": 72,
    "aksi_tersedia": ["SETUJUI", "TOLAK"]
  }
]
```

#### Workflow Instance

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `workflow_definition_id` | UUID (FK) | Ya | Referensi ke definisi workflow |
| `referensi_id` | UUID | Ya | ID entitas yang sedang dalam proses (misal: ID izin pemakaian) |
| `referensi_tipe` | String(50) | Ya | Tipe entitas (misal: `IZIN_PEMAKAIAN`, `KEBUTUHAN_BMN`) |
| `satker_id` | UUID | Ya | Satker terkait |
| `current_step` | Integer | Ya | Step saat ini |
| `status` | Enum | Ya | `BERJALAN`, `SELESAI_DISETUJUI`, `SELESAI_DITOLAK`, `DIBATALKAN`, `KADALUARSA` |
| `initiated_by` | UUID | Ya | ID pengguna yang memulai |
| `initiated_at` | Timestamp | Ya | Waktu dimulai |
| `completed_at` | Timestamp | Tidak | Waktu selesai |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |

#### Workflow Action Log

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `workflow_instance_id` | UUID (FK) | Ya | Referensi ke instance workflow |
| `step_order` | Integer | Ya | Nomor step |
| `aksi` | Enum | Ya | `SETUJUI`, `TOLAK`, `KEMBALIKAN`, `ESKALASI`, `DELEGASI`, `CATATAN` |
| `dilakukan_oleh` | UUID | Ya | ID pengguna yang melakukan aksi |
| `dilakukan_pada` | Timestamp | Ya | Waktu aksi dilakukan |
| `catatan` | Text | Tidak | Catatan dari approver |
| `dari_status` | String(50) | Ya | Status sebelum aksi |
| `ke_status` | String(50) | Ya | Status setelah aksi |

### 8.3 Aktivitas (Activities)

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-W01 | Definisi workflow baru | Admin Pusat | Mendefinisikan workflow baru dengan langkah-langkah persetujuan |
| ACT-W02 | Ubah definisi workflow | Admin Pusat | Mengubah definisi workflow (membuat versi baru) |
| ACT-W03 | Inisiasi workflow instance | Sistem (dari layanan lain) | Membuat instance workflow baru saat ada pengajuan |
| ACT-W04 | Proses approval | Verifikator, Pimpinan | Menyetujui, menolak, atau mengembalikan pada step tertentu |
| ACT-W05 | Eskalasi otomatis | Sistem | Mengeskalasi ke level di atasnya jika SLA terlampaui |
| ACT-W06 | Delegasi | Approver | Mendelegasikan tugas approval ke orang lain |
| ACT-W07 | Monitoring workflow | Admin Pusat | Memonitor seluruh workflow yang berjalan |
| ACT-W08 | Pembatalan workflow | Admin Pusat, pemohon | Membatalkan workflow yang sedang berjalan |

### 8.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-W001 | Tinggi | Sistem harus menyediakan mesin workflow generik yang dapat dikonfigurasi tanpa perubahan kode untuk menangani berbagai jenis proses persetujuan | Admin Pusat |
| REQ-W002 | Tinggi | Setiap step dalam workflow harus dapat dikonfigurasi dengan role approver, SLA waktu proses, aksi yang tersedia, dan aturan eskalasi | Admin Pusat |
| REQ-W003 | Tinggi | Sistem harus menjalankan eskalasi otomatis jika SLA terlampaui, dengan notifikasi ke atasan approver | Sistem |
| REQ-W004 | Tinggi | Sistem harus menyediakan API yang konsisten untuk layanan lain memulai, mengambil status, dan menerima callback dari workflow | Sistem |
| REQ-W005 | Tinggi | Setiap aksi dalam workflow harus tercatat dalam log yang immutable (tidak dapat diubah/dihapus) untuk keperluan audit | Sistem |
| REQ-W006 | Sedang | Sistem harus mendukung fitur delegasi, di mana seorang approver dapat mendelegasikan tugas approval ke rekan sejawat atau bawahan | Approver |
| REQ-W007 | Sedang | Sistem harus mendukung parallel approval (jika dibutuhkan) di mana beberapa approver harus menyetujui secara paralel pada satu step | Sistem |
| REQ-W008 | Sedang | Sistem harus menyediakan dashboard monitoring seluruh workflow yang sedang berjalan dengan informasi step saat ini, SLA, dan aging | Admin Pusat |
| REQ-W009 | Sedang | Sistem harus mendukung versioning definisi workflow sehingga instance yang sedang berjalan tetap menggunakan versi definisi saat instance dimulai | Sistem |
| REQ-W010 | Rendah | Sistem harus mendukung conditional branching di mana step berikutnya dapat berbeda berdasarkan kondisi tertentu (misal: jika nilai di atas threshold, perlu approval tambahan) | Sistem |
| REQ-W011 | Tinggi | Sistem harus mempublikasikan event saat terjadi perubahan status workflow (dimulai, disetujui, ditolak, dieskalasi) agar layanan lain dapat bereaksi | Sistem |
| REQ-W012 | Sedang | Sistem harus menyediakan inbox/task list untuk setiap approver yang menampilkan daftar pengajuan yang menunggu aksi mereka | Approver |

### 8.5 Siklus Hidup (Lifecycle)

#### Lifecycle Workflow Instance

```
BERJALAN → STEP_N_DISETUJUI → STEP_N+1 (next step) → ... → SELESAI_DISETUJUI
         → STEP_N_DITOLAK → SELESAI_DITOLAK
         → STEP_N_DIKEMBALIKAN → STEP_N-1 / DRAFT (ke pemohon)
         → DIESKALASI → STEP_N (approver baru)
         → DIBATALKAN
         → KADALUARSA (jika tidak ada aksi dalam batas waktu keseluruhan)
```

### 8.6 User Stories

| ID | Story |
|---|---|
| US-W001 | Sebagai Admin Pusat, saya ingin mendefinisikan alur persetujuan pemakaian kendaraan dinas yang terdiri dari: verifikasi operator → persetujuan Kabag TU → persetujuan Kajari/Kajati sehingga setiap pengajuan melewati rantai persetujuan yang benar. |
| US-W002 | Sebagai Verifikator, saya ingin melihat daftar pengajuan yang menunggu aksi saya (inbox) dengan informasi SLA sehingga saya bisa memprioritaskan yang hampir melewati batas waktu. |
| US-W003 | Sebagai Sistem, saya ingin mengeskalasi pengajuan yang sudah melewati SLA 48 jam ke Pimpinan Satker dengan notifikasi sehingga pengajuan tidak tertunda terlalu lama. |
| US-W004 | Sebagai Verifikator, saya ingin mendelegasikan tugas approval ke rekan saya ketika saya sedang cuti sehingga pengajuan tetap dapat diproses. |
| US-W005 | Sebagai Admin Pusat, saya ingin melihat dashboard seluruh workflow yang berjalan dengan indikator SLA sehingga saya bisa mengidentifikasi bottleneck proses. |
| US-W006 | Sebagai Admin Pusat, saya ingin mengubah definisi workflow tanpa mempengaruhi workflow yang sedang berjalan sehingga perubahan kebijakan tidak mengganggu proses yang sudah dimulai. |

---

## 9. Layanan 5 — Dokumen (`dokumen`)

### 9.1 Deskripsi Layanan

Layanan Dokumen bertanggung jawab atas pembuatan, penyimpanan, pengelolaan, dan distribusi seluruh dokumen administratif yang dihasilkan oleh SIMPEL. Layanan ini menerima permintaan dari layanan lain untuk men-generate dokumen berdasarkan template yang telah didefinisikan, menyimpan dokumen yang dihasilkan, dan menyediakan akses untuk pengunduhan. Salah satu dokumen utama yang dihasilkan adalah SK Penghapusan BMN.

### 9.2 Entitas (Entities)

#### E12 — Dokumen

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `nomor_dokumen` | String(100) | Ya | Nomor dokumen resmi |
| `jenis_dokumen` | Enum | Ya | Jenis dokumen (lihat daftar di bawah) |
| `judul` | String(500) | Ya | Judul dokumen |
| `deskripsi` | Text | Tidak | Deskripsi/ringkasan dokumen |
| `template_id` | UUID (FK→E13) | Tidak | Template yang digunakan |
| `referensi_id` | UUID | Tidak | ID entitas terkait (izin pemakaian, kebutuhan BMN, dll.) |
| `referensi_tipe` | String(50) | Tidak | Tipe entitas terkait |
| `satker_id` | UUID | Ya | Satker yang menerbitkan |
| `tanggal_dokumen` | Date | Ya | Tanggal dokumen |
| `tanggal_berlaku` | Date | Tidak | Tanggal mulai berlaku |
| `tanggal_berakhir` | Date | Tidak | Tanggal berakhir berlaku |
| `penandatangan` | String(255) | Ya | Nama dan jabatan penandatangan |
| `penandatangan_id` | UUID | Tidak | ID pengguna penandatangan |
| `file_path` | String(500) | Ya | Path file di object storage |
| `file_format` | Enum | Ya | `PDF`, `DOCX` |
| `file_size_bytes` | BigInteger | Ya | Ukuran file dalam bytes |
| `checksum` | String(64) | Ya | SHA-256 checksum untuk integritas file |
| `status` | Enum | Ya | `DRAFT`, `FINAL`, `DITANDATANGANI`, `DIBATALKAN`, `DIREVISI` |
| `versi` | Integer | Ya | Versi dokumen |
| `dokumen_revisi_dari` | UUID (FK→self) | Tidak | Dokumen versi sebelumnya (jika revisi) |
| `metadata` | JSONB | Tidak | Metadata tambahan |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |

**Jenis Dokumen:**

| Kode | Jenis Dokumen | Layanan Peminta |
|---|---|---|
| `SK_PENGHAPUSAN_BMN` | Surat Keputusan Penghapusan BMN | Kebutuhan / Master |
| `SURAT_IZIN_PEMAKAIAN` | Surat Izin Pemakaian BMN | Pemakaian |
| `SURAT_PENCABUTAN_IZIN` | Surat Pencabutan Izin Pemakaian | Pemakaian |
| `SURAT_PERPANJANGAN_IZIN` | Surat Perpanjangan Izin Pemakaian | Pemakaian |
| `REKAPITULASI_KEBUTUHAN` | Rekapitulasi Kebutuhan BMN | Kebutuhan |
| `LAPORAN_KEBUTUHAN_PAKDIN` | Laporan Kebutuhan Pakaian Dinas | Kebutuhan |
| `BERITA_ACARA` | Berita Acara (umum) | Berbagai |
| `KARTU_INVENTARIS_BARANG` | Kartu Inventaris Barang (KIB) | Master / Integrasi |
| `LAPORAN_ROADMAP` | Laporan Roadmap Sarpras | Kebutuhan |
| `LAPORAN_PEMAKAIAN` | Laporan Pemakaian BMN | Pemakaian |
| `DAFTAR_BARANG` | Daftar Barang per Satker | Master / Integrasi |
| `SURAT_UMUM` | Surat Umum | Berbagai |

#### E13 — Template Dokumen

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode` | String(50) | Ya | Kode unik template |
| `nama` | String(255) | Ya | Nama template |
| `jenis_dokumen` | Enum | Ya | Jenis dokumen yang dihasilkan |
| `deskripsi` | Text | Tidak | Deskripsi template |
| `format_output` | Enum | Ya | `PDF`, `DOCX` |
| `template_engine` | Enum | Ya | `HANDLEBARS`, `TERA`, `TYPST` |
| `template_content` | Text | Ya | Konten template dengan placeholder |
| `schema_data` | JSONB | Ya | Definisi data yang dibutuhkan oleh template (JSON Schema) |
| `kop_surat` | Boolean | Ya | Apakah menggunakan kop surat resmi |
| `versi` | Integer | Ya | Versi template |
| `is_aktif` | Boolean | Ya | Status aktif |
| `contoh_data` | JSONB | Tidak | Data contoh untuk preview template |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |

### 9.3 Aktivitas (Activities)

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-D01 | Kelola template dokumen | Admin Pusat | CRUD template dokumen, termasuk desain dan pengujian |
| ACT-D02 | Preview template | Admin Pusat | Melihat preview dokumen dengan data contoh |
| ACT-D03 | Generate dokumen otomatis | Sistem | Men-generate dokumen berdasarkan request dari layanan lain |
| ACT-D04 | Generate SK Penghapusan BMN | Admin Pusat | Membuat SK Penghapusan BMN berdasarkan data BMN yang akan dihapus |
| ACT-D05 | Revisi dokumen | Admin Pusat | Membuat revisi dokumen yang sudah ada (versi baru) |
| ACT-D06 | Unduh dokumen | Semua Pengguna | Mengunduh dokumen yang sudah di-generate |
| ACT-D07 | Cari dokumen | Semua Pengguna | Mencari dokumen berdasarkan nomor, jenis, satker, atau tanggal |
| ACT-D08 | Pembatalan dokumen | Admin Pusat | Membatalkan dokumen yang sudah terbit |
| ACT-D09 | Verifikasi integritas | Sistem | Memverifikasi bahwa dokumen tidak dimodifikasi setelah diterbitkan (checksum) |
| ACT-D10 | Arsip dokumen | Sistem | Mengarsipkan dokumen lama sesuai kebijakan retensi |

### 9.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-D001 | Tinggi | Sistem harus menyediakan mekanisme template-based document generation di mana dokumen dihasilkan dari template yang diisi dengan data dinamis | Sistem |
| REQ-D002 | Tinggi | Sistem harus mendukung pembuatan SK Penghapusan BMN yang mencakup data BMN yang akan dihapus, dasar hukum, pertimbangan, dan format SK resmi Kejaksaan | Admin Pusat |
| REQ-D003 | Tinggi | Sistem harus mendukung pembuatan surat izin pemakaian BMN, surat pencabutan, dan surat perpanjangan secara otomatis saat workflow terkait selesai | Sistem |
| REQ-D004 | Tinggi | Sistem harus menghasilkan dokumen dalam format PDF dengan kop surat resmi Kejaksaan | Sistem |
| REQ-D005 | Tinggi | Sistem harus menyediakan API untuk layanan lain agar dapat meminta pembuatan dokumen dengan mengirimkan jenis template dan data yang diperlukan | Sistem |
| REQ-D006 | Sedang | Sistem harus menyediakan CRUD template dokumen dengan fitur preview menggunakan data contoh | Admin Pusat |
| REQ-D007 | Sedang | Sistem harus menyimpan setiap dokumen yang dihasilkan di object storage dengan checksum SHA-256 untuk verifikasi integritas | Sistem |
| REQ-D008 | Sedang | Sistem harus mendukung versioning dokumen di mana revisi menghasilkan versi baru tanpa menghapus versi lama | Admin Pusat |
| REQ-D009 | Sedang | Sistem harus menyediakan pencarian dokumen berdasarkan nomor dokumen, jenis, satker, rentang tanggal, dan status | Semua |
| REQ-D010 | Sedang | Sistem harus mendukung penomoran dokumen otomatis sesuai format yang berlaku di Kejaksaan | Sistem |
| REQ-D011 | Rendah | Sistem harus mendukung pembuatan dokumen rekapitulasi kebutuhan BMN dalam format yang siap cetak (PDF) dan siap olah (Excel) | Admin Pusat |
| REQ-D012 | Rendah | Sistem harus mendukung kebijakan retensi dokumen di mana dokumen lama dapat diarsipkan setelah periode tertentu | Sistem |
| REQ-D013 | Tinggi | Sistem harus mencatat setiap pembuatan, pengunduhan, dan pembatalan dokumen dalam audit trail | Sistem |

### 9.5 Siklus Hidup (Lifecycle)

#### Lifecycle Dokumen

```
DRAFT → FINAL → DITANDATANGANI → (DIREVISI → versi baru DRAFT)
                                → DIBATALKAN
```

### 9.6 User Stories

| ID | Story |
|---|---|
| US-D001 | Sebagai Admin Pusat, saya ingin membuat SK Penghapusan BMN dengan memilih daftar BMN yang akan dihapus dan sistem secara otomatis men-generate dokumen SK lengkap dengan kop surat, konsiderans, diktum, dan lampiran daftar barang sehingga saya tidak perlu mengetik manual. |
| US-D002 | Sebagai Sistem, saya ingin secara otomatis men-generate surat izin pemakaian kendaraan dinas setelah workflow persetujuan selesai sehingga dokumen langsung tersedia untuk diunduh oleh pemohon. |
| US-D003 | Sebagai Admin Pusat, saya ingin mengelola template dokumen (menambah, mengubah, menonaktifkan) dan melihat preview hasilnya sehingga format dokumen dapat disesuaikan tanpa mengubah kode program. |
| US-D004 | Sebagai Operator Satker, saya ingin mencari dan mengunduh dokumen surat izin pemakaian yang sudah diterbitkan untuk satker saya sehingga saya dapat mencetaknya untuk arsip fisik. |
| US-D005 | Sebagai Auditor, saya ingin memverifikasi bahwa dokumen yang tersimpan di sistem tidak pernah dimodifikasi setelah diterbitkan sehingga integritas dokumen terjamin. |
| US-D006 | Sebagai Admin Pusat, saya ingin meng-generate rekapitulasi kebutuhan BMN nasional dalam format PDF (siap cetak) dan Excel (siap olah) sehingga data dapat didistribusikan ke pimpinan dan unit terkait. |

---

## 10. Layanan 6 — Dashboard & Analitik (`dashboard`)

### 10.1 Deskripsi Layanan

Layanan Dashboard & Analitik menyediakan visualisasi data, pelaporan, dan analisis lintas layanan untuk mendukung pengambilan keputusan. Layanan ini mengonsumsi data dari layanan-layanan lain (melalui read-model atau event) dan menyajikannya dalam bentuk dashboard interaktif, grafik, dan laporan.

### 10.2 Aktivitas (Activities)

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-DB01 | Lihat dashboard eksekutif | Pimpinan Pusat | Melihat ringkasan nasional: total kebutuhan, realisasi, pemakaian aktif, mapping kodefikasi |
| ACT-DB02 | Lihat dashboard wilayah | Pimpinan Wilayah, Admin Wilayah | Melihat ringkasan wilayah dengan drill-down ke satker |
| ACT-DB03 | Lihat dashboard satker | Pimpinan Satker, Operator | Melihat detail data satker |
| ACT-DB04 | Lihat roadmap vs realisasi | Pimpinan, Admin Pusat | Melihat perbandingan target roadmap dengan realisasi per tahun |
| ACT-DB05 | Lihat analisis gap | Admin Pusat, Pimpinan | Melihat gap antara kebutuhan standar dan kondisi eksisting |
| ACT-DB06 | Lihat tren kebutuhan | Admin Pusat, Pimpinan | Melihat tren kebutuhan BMN dari tahun ke tahun |
| ACT-DB07 | Lihat status workflow | Admin Pusat | Melihat status seluruh workflow yang berjalan |
| ACT-DB08 | Lihat utilisasi BMN | Admin Pusat, Pimpinan | Melihat rasio pemakaian BMN vs ketersediaan |
| ACT-DB09 | Lihat progress mapping kodefikasi | Admin Pusat | Melihat progress pemetaan kodefikasi per satker/wilayah |
| ACT-DB10 | Unduh laporan | Semua Pengguna | Mengunduh data dari dashboard dalam format Excel/PDF |
| ACT-DB11 | Filter dan drill-down | Semua Pengguna | Memfilter data berdasarkan parameter dan melakukan drill-down ke level detail |

### 10.3 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-DB001 | Tinggi | Sistem harus menyediakan dashboard eksekutif yang menampilkan KPI utama: total kebutuhan BMN nasional, persentase pemenuhan, jumlah izin pemakaian aktif, dan progress mapping kodefikasi | Pimpinan |
| REQ-DB002 | Tinggi | Sistem harus mendukung drill-down dari level nasional → wilayah (Kejati) → satker (Kejari/Cabjari) pada semua dashboard | Semua |
| REQ-DB003 | Tinggi | Sistem harus menyediakan visualisasi roadmap vs realisasi dalam bentuk chart yang menampilkan target 5 tahunan dan pencapaian per tahun | Pimpinan, Admin |
| REQ-DB004 | Tinggi | Sistem harus menyediakan visualisasi gap analysis (kebutuhan standar - eksisting) per jenis BMN dan per satker | Admin Pusat, Pimpinan |
| REQ-DB005 | Sedang | Sistem harus menyediakan peta sebaran (heatmap) kebutuhan BMN per wilayah di peta Indonesia | Pimpinan |
| REQ-DB006 | Sedang | Sistem harus menyediakan grafik tren kebutuhan BMN dari tahun ke tahun | Admin Pusat |
| REQ-DB007 | Sedang | Sistem harus menyediakan dashboard monitoring workflow yang menampilkan jumlah pengajuan pending, rata-rata waktu proses, dan bottleneck | Admin Pusat |
| REQ-DB008 | Sedang | Sistem harus menyediakan laporan utilisasi BMN per jenis dan per satker | Admin Pusat, Pimpinan |
| REQ-DB009 | Sedang | Sistem harus mendukung ekspor setiap dashboard ke format PDF (laporan) dan Excel (data mentah) | Semua |
| REQ-DB010 | Sedang | Sistem harus menyediakan fitur filter yang konsisten di semua dashboard: berdasarkan tahun anggaran, wilayah, jenis BMN, dan satker | Semua |
| REQ-DB011 | Rendah | Sistem harus menyediakan fitur saved filter/bookmark sehingga pengguna tidak perlu mengatur filter berulang kali | Semua |
| REQ-DB012 | Rendah | Sistem harus mendukung auto-refresh dashboard pada interval yang dapat dikonfigurasi | Sistem |
| REQ-DB013 | Tinggi | Data pada dashboard harus mencerminkan kondisi terkini dengan delay maksimum sesuai SLA per jenis data (lihat kebutuhan non-fungsional) | Sistem |

### 10.4 User Stories

| ID | Story |
|---|---|
| US-DB001 | Sebagai Pimpinan Pusat, saya ingin melihat ringkasan eksekutif dalam satu halaman yang menampilkan total kebutuhan BMN nasional, persentase pemenuhan, dan area yang membutuhkan perhatian sehingga saya dapat mengambil keputusan strategis. |
| US-DB002 | Sebagai Pimpinan Wilayah, saya ingin melihat dashboard yang menampilkan kebutuhan BMN seluruh Kejari di wilayah saya dengan kemampuan drill-down ke masing-masing satker sehingga saya bisa mengidentifikasi satker yang paling membutuhkan. |
| US-DB003 | Sebagai Admin Pusat, saya ingin melihat grafik roadmap vs realisasi sarpras 5 tahunan sehingga saya bisa mengevaluasi sejauh mana rencana sudah tercapai. |
| US-DB004 | Sebagai Admin Pusat, saya ingin mengekspor data gap analysis ke Excel sehingga saya bisa mengolah data lebih lanjut untuk bahan rapat. |
| US-DB005 | Sebagai Pimpinan, saya ingin melihat peta heatmap kebutuhan BMN per provinsi sehingga saya bisa melihat daerah mana yang paling kekurangan BMN secara visual. |

---

## 11. Layanan 7 — Integrasi (`integrasi`)

### 11.1 Deskripsi Layanan

Layanan Integrasi (Integration Gateway) bertanggung jawab sebagai adapter dan cache tunggal untuk seluruh interaksi dengan sistem eksternal. Layanan ini memastikan bahwa layanan-layanan lain di SIMPEL tidak perlu mengetahui detail teknis koneksi ke MonSAKTI atau MySIMKARI. Layanan ini menangani penarikan data, transformasi format, caching, retry logic, circuit breaking, dan error handling untuk semua integrasi eksternal.

### 11.2 Entitas (Entities)

#### Cached Data — BMN dari MonSAKTI (E03)

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik di SIMPEL |
| `id_monsakti` | String(50) | Ya | Identifier asli di MonSAKTI |
| `satker_id` | UUID | Ya | Satker pemilik |
| `kode_satker_monsakti` | String(20) | Ya | Kode satker di MonSAKTI |
| `kode_barang` | String(30) | Ya | Kode barang SIMAK BMN |
| `nup` | String(20) | Ya | Nomor Urut Pendaftaran |
| `nama_barang` | String(500) | Ya | Nama/uraian barang |
| `merk_tipe` | String(255) | Tidak | Merk/tipe barang |
| `nomor_seri` | String(100) | Tidak | Nomor seri/registrasi |
| `tahun_perolehan` | Integer | Ya | Tahun perolehan |
| `nilai_perolehan` | Decimal | Ya | Nilai perolehan (Rp) |
| `nilai_buku` | Decimal | Tidak | Nilai buku setelah penyusutan |
| `akumulasi_penyusutan` | Decimal | Tidak | Akumulasi penyusutan |
| `kondisi` | Enum | Ya | `BAIK`, `RUSAK_RINGAN`, `RUSAK_BERAT` |
| `status_penggunaan` | Enum | Ya | `DIGUNAKAN`, `IDLE`, `DALAM_PROSES_PENGHAPUSAN` |
| `kuantitas` | Decimal | Ya | Jumlah/kuantitas |
| `satuan` | String(20) | Ya | Satuan |
| `lokasi` | String(500) | Tidak | Lokasi BMN |
| `last_synced_at` | Timestamp | Ya | Waktu sinkronisasi terakhir |
| `sync_status` | Enum | Ya | `SYNCED`, `PENDING`, `ERROR` |
| `raw_data` | JSONB | Tidak | Data mentah asli dari MonSAKTI (untuk debug) |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |

#### Cached Data — Pegawai dari MySIMKARI (E02)

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik di SIMPEL |
| `id_mysimkari` | String(50) | Ya | Identifier asli di MySIMKARI |
| `nip` | String(18) | Ya | NIP pegawai |
| `nama` | String(255) | Ya | Nama lengkap |
| `gelar_depan` | String(50) | Tidak | Gelar depan |
| `gelar_belakang` | String(50) | Tidak | Gelar belakang |
| `jabatan` | String(200) | Ya | Jabatan saat ini |
| `jenis_jabatan` | Enum | Ya | `STRUKTURAL`, `FUNGSIONAL_JAKSA`, `FUNGSIONAL_UMUM`, `PELAKSANA` |
| `eselon` | String(10) | Tidak | Eselon (jika jabatan struktural) |
| `golongan` | String(10) | Ya | Golongan/pangkat |
| `satker_id` | UUID | Ya | Satker saat ini |
| `kode_satker_mysimkari` | String(20) | Ya | Kode satker di MySIMKARI |
| `status_pegawai` | Enum | Ya | `AKTIF`, `CUTI`, `TUGAS_BELAJAR`, `PENSIUN`, `MUTASI`, `BERHENTI` |
| `tanggal_lahir` | Date | Tidak | Tanggal lahir |
| `jenis_kelamin` | Enum | Ya | `LAKI_LAKI`, `PEREMPUAN` |
| `pendidikan_terakhir` | String(50) | Tidak | Pendidikan terakhir |
| `email` | String(255) | Tidak | Email dinas |
| `nomor_telepon` | String(20) | Tidak | Nomor telepon |
| `last_synced_at` | Timestamp | Ya | Waktu sinkronisasi terakhir |
| `sync_status` | Enum | Ya | `SYNCED`, `PENDING`, `ERROR` |
| `raw_data` | JSONB | Tidak | Data mentah asli dari MySIMKARI |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |

#### Sync Log

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `sumber_data` | Enum | Ya | `MONSAKTI`, `MYSIMKARI` |
| `tipe_sinkronisasi` | Enum | Ya | `FULL`, `INCREMENTAL`, `ON_DEMAND` |
| `waktu_mulai` | Timestamp | Ya | Waktu mulai sinkronisasi |
| `waktu_selesai` | Timestamp | Tidak | Waktu selesai |
| `status` | Enum | Ya | `BERJALAN`, `BERHASIL`, `GAGAL`, `SEBAGIAN` |
| `jumlah_total` | Integer | Tidak | Total record yang diproses |
| `jumlah_baru` | Integer | Tidak | Record baru ditambahkan |
| `jumlah_diperbarui` | Integer | Tidak | Record yang diperbarui |
| `jumlah_error` | Integer | Tidak | Record yang gagal |
| `error_detail` | JSONB | Tidak | Detail error per record |
| `triggered_by` | Enum | Ya | `JADWAL`, `MANUAL`, `EVENT` |
| `triggered_by_user` | UUID | Tidak | User yang memicu (jika manual) |

### 11.3 Aktivitas (Activities)

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-I01 | Sinkronisasi penuh (full sync) MonSAKTI | Sistem (terjadwal), Admin Pusat | Menarik seluruh data BMN dari MonSAKTI |
| ACT-I02 | Sinkronisasi inkremental MonSAKTI | Sistem (terjadwal) | Menarik data BMN yang berubah sejak sinkronisasi terakhir |
| ACT-I03 | Sinkronisasi penuh MySIMKARI | Sistem (terjadwal), Admin Pusat | Menarik seluruh data pegawai dari MySIMKARI |
| ACT-I04 | Sinkronisasi inkremental MySIMKARI | Sistem (terjadwal) | Menarik data pegawai yang berubah sejak sinkronisasi terakhir |
| ACT-I05 | Sinkronisasi on-demand per satker | Admin Wilayah, Admin Pusat | Memicu sinkronisasi manual untuk satker tertentu |
| ACT-I06 | Monitoring status sinkronisasi | Admin Pusat | Memonitor status dan riwayat sinkronisasi |
| ACT-I07 | Konfigurasi koneksi | Super Admin | Mengatur parameter koneksi ke sistem eksternal |
| ACT-I08 | Pengelolaan mapping satker | Admin Pusat | Memetakan kode satker antara SIMPEL, MonSAKTI, dan MySIMKARI |
| ACT-I09 | Penanganan error sinkronisasi | Admin Pusat | Menangani dan meremediasi error sinkronisasi |

### 11.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-I001 | Tinggi | Sistem harus menyediakan adapter untuk menarik data BMN dari MonSAKTI secara terjadwal (minimal harian) dan menyimpannya dalam cache lokal | Sistem |
| REQ-I002 | Tinggi | Sistem harus menyediakan adapter untuk menarik data pegawai dari MySIMKARI secara terjadwal (minimal harian) dan menyimpannya dalam cache lokal | Sistem |
| REQ-I003 | Tinggi | Sistem harus menyediakan API internal yang konsisten untuk layanan lain agar dapat mengakses data BMN dan pegawai tanpa perlu mengetahui detail sistem sumber | Sistem |
| REQ-I004 | Tinggi | Sistem harus menerapkan retry logic dengan exponential backoff untuk menangani kegagalan koneksi ke sistem eksternal | Sistem |
| REQ-I005 | Tinggi | Sistem harus menerapkan circuit breaker untuk mencegah cascading failure jika sistem eksternal tidak tersedia dalam waktu lama | Sistem |
| REQ-I006 | Tinggi | Sistem harus menyediakan mekanisme sinkronisasi inkremental (hanya data yang berubah) untuk efisiensi bandwidth dan waktu proses | Sistem |
| REQ-I007 | Sedang | Sistem harus mendukung sinkronisasi on-demand per satker yang dapat dipicu secara manual oleh Admin | Admin Pusat, Admin Wilayah |
| REQ-I008 | Sedang | Sistem harus mencatat log sinkronisasi yang detail, mencakup jumlah data yang diproses, waktu eksekusi, dan error yang terjadi | Sistem |
| REQ-I009 | Sedang | Sistem harus menyediakan dashboard monitoring status sinkronisasi yang menampilkan waktu sinkronisasi terakhir per sumber data, status, dan jumlah error | Admin Pusat |
| REQ-I010 | Sedang | Sistem harus menangani transformasi dan normalisasi data dari format sistem sumber ke format internal SIMPEL | Sistem |
| REQ-I011 | Sedang | Sistem harus mendukung mapping kode satker antar sistem (SIMPEL ↔ MonSAKTI ↔ MySIMKARI) yang dapat dikonfigurasi | Admin Pusat |
| REQ-I012 | Rendah | Sistem harus menyimpan raw data asli dari sistem sumber (sebagai JSONB) untuk keperluan debugging dan audit | Sistem |
| REQ-I013 | Tinggi | Sistem harus mempublikasikan event saat data berhasil disinkronisasi agar layanan konsumer dapat bereaksi (misal: recalculate kebutuhan) | Sistem |
| REQ-I014 | Sedang | Sistem harus menangani perbedaan struktur data jika MonSAKTI atau MySIMKARI mengubah format API/ekspor data mereka, dengan konfigurasi mapping yang dapat diubah tanpa deploy ulang | Admin Pusat |

### 11.5 User Stories

| ID | Story |
|---|---|
| US-I001 | Sebagai Sistem, saya ingin menarik data BMN dari MonSAKTI setiap malam secara otomatis sehingga data BMN di SIMPEL selalu terkini. |
| US-I002 | Sebagai Sistem, saya ingin menarik data pegawai dari MySIMKARI setiap malam secara otomatis sehingga data pegawai (jabatan, golongan, satker) selalu terkini. |
| US-I003 | Sebagai Admin Pusat, saya ingin memicu sinkronisasi manual untuk satker tertentu jika saya tahu ada perubahan data besar di MonSAKTI sehingga data langsung terupdate tanpa menunggu jadwal. |
| US-I004 | Sebagai Admin Pusat, saya ingin melihat status sinkronisasi terakhir dan riwayat error sehingga saya tahu jika ada masalah koneksi yang perlu ditangani. |
| US-I005 | Sebagai Layanan Kebutuhan, saya ingin memanggil API integrasi untuk mendapatkan jumlah BMN eksisting per satker per kode barang sehingga saya bisa menghitung gap kebutuhan tanpa perlu tahu cara koneksi ke MonSAKTI. |
| US-I006 | Sebagai Layanan Kebutuhan, saya ingin memanggil API integrasi untuk mendapatkan daftar pegawai per satker beserta jabatan dan golongan sehingga saya bisa menghitung eligibilitas pakaian dinas tanpa perlu tahu cara koneksi ke MySIMKARI. |

---

## 12. Layanan 8 — Notifikasi (`notifikasi`)

### 12.1 Deskripsi Layanan

Layanan Notifikasi bertanggung jawab atas pengiriman notifikasi multi-kanal kepada pengguna SIMPEL. Layanan ini menerima permintaan notifikasi dari layanan lain dan mengirimkannya melalui kanal yang sesuai. Layanan ini juga mengelola preferensi notifikasi pengguna dan menyediakan pusat notifikasi (notification center) dalam aplikasi.

### 12.2 Entitas (Entities)

#### Notifikasi

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `penerima_id` | UUID | Ya | ID pengguna penerima |
| `judul` | String(255) | Ya | Judul notifikasi |
| `pesan` | Text | Ya | Isi pesan notifikasi |
| `tipe` | Enum | Ya | `INFO`, `PERINGATAN`, `AKSI_DIPERLUKAN`, `REMINDER`, `SUKSES`, `ERROR` |
| `kanal` | Array(Enum) | Ya | Kanal pengiriman: `IN_APP`, `EMAIL`, `PUSH` |
| `prioritas` | Enum | Ya | `TINGGI`, `NORMAL`, `RENDAH` |
| `sumber_layanan` | String(50) | Ya | Layanan yang mengirim notifikasi |
| `referensi_id` | UUID | Tidak | ID entitas terkait |
| `referensi_tipe` | String(50) | Tidak | Tipe entitas terkait |
| `url_aksi` | String(500) | Tidak | URL untuk aksi (misal: link ke halaman approval) |
| `label_aksi` | String(100) | Tidak | Label tombol aksi (misal: "Lihat Pengajuan") |
| `is_dibaca` | Boolean | Ya | Status sudah dibaca |
| `dibaca_pada` | Timestamp | Tidak | Waktu dibaca |
| `status_kirim` | JSONB | Ya | Status pengiriman per kanal |
| `created_at` | Timestamp | Ya | Waktu pembuatan |

#### Preferensi Notifikasi

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `user_id` | UUID | Ya | ID pengguna |
| `kanal_aktif` | Array(Enum) | Ya | Kanal yang diaktifkan: `IN_APP`, `EMAIL`, `PUSH` |
| `quiet_hours_start` | Time | Tidak | Jam mulai mode senyap |
| `quiet_hours_end` | Time | Tidak | Jam akhir mode senyap |
| `digest_mode` | Boolean | Ya | Apakah notifikasi di-batch menjadi digest |
| `digest_jadwal` | Enum | Tidak | `HARIAN`, `MINGGUAN` |
| `filter_tipe` | JSONB | Tidak | Filter notifikasi per tipe yang ingin diterima |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |

#### Template Notifikasi

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode` | String(50) | Ya | Kode unik template (misal: `NOTIF_IZIN_REMINDER_H30`) |
| `nama` | String(255) | Ya | Nama template |
| `judul_template` | String(255) | Ya | Template judul dengan placeholder |
| `pesan_template` | Text | Ya | Template pesan dengan placeholder |
| `tipe` | Enum | Ya | Tipe notifikasi |
| `kanal_default` | Array(Enum) | Ya | Kanal default |
| `is_aktif` | Boolean | Ya | Status aktif |

### 12.3 Aktivitas (Activities)

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-N01 | Kirim notifikasi | Sistem (dari layanan lain) | Mengirim notifikasi ke pengguna melalui kanal yang dikonfigurasi |
| ACT-N02 | Lihat notifikasi in-app | Semua Pengguna | Melihat daftar notifikasi di notification center |
| ACT-N03 | Tandai sudah dibaca | Semua Pengguna | Menandai notifikasi sebagai sudah dibaca |
| ACT-N04 | Kelola preferensi | Semua Pengguna | Mengatur preferensi notifikasi (kanal, quiet hours, digest) |
| ACT-N05 | Kelola template notifikasi | Admin Pusat | CRUD template notifikasi |
| ACT-N06 | Kirim reminder terjadwal | Sistem | Mengirim reminder otomatis berdasarkan jadwal (izin akan expired, deadline pengumpulan, dll.) |
| ACT-N07 | Monitoring pengiriman | Admin Pusat | Memonitor status pengiriman notifikasi |

### 12.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-N001 | Tinggi | Sistem harus menyediakan notification center (in-app) yang menampilkan daftar notifikasi pengguna dengan indikator belum dibaca | Semua |
| REQ-N002 | Tinggi | Sistem harus mendukung pengiriman notifikasi melalui minimal dua kanal: in-app dan email | Sistem |
| REQ-N003 | Tinggi | Sistem harus menyediakan API untuk layanan lain agar dapat mengirim notifikasi dengan menyebutkan template, penerima, dan data dinamis | Sistem |
| REQ-N004 | Tinggi | Sistem harus mendukung notifikasi bertipe `AKSI_DIPERLUKAN` yang menyertakan link langsung ke halaman yang membutuhkan aksi (deep link) | Sistem |
| REQ-N005 | Sedang | Sistem harus mendukung pengiriman notifikasi ke role tertentu (misal: semua Verifikator di satker X) bukan hanya ke individu | Sistem |
| REQ-N006 | Sedang | Sistem harus mendukung preferensi notifikasi per pengguna (kanal aktif, quiet hours, digest mode) | Semua |
| REQ-N007 | Sedang | Sistem harus menyediakan template notifikasi yang dapat dikelola oleh Admin Pusat sehingga konten notifikasi dapat diubah tanpa deploy ulang | Admin Pusat |
| REQ-N008 | Sedang | Sistem harus mengirim reminder otomatis berdasarkan event dari layanan lain (izin H-30, deadline pengumpulan H-7, SLA workflow terlampaui) | Sistem |
| REQ-N009 | Rendah | Sistem harus mendukung digest mode di mana notifikasi non-kritis dikumpulkan dan dikirim sebagai rangkuman harian/mingguan | Sistem |
| REQ-N010 | Rendah | Sistem harus mencatat status pengiriman per kanal (terkirim, gagal, dibaca) untuk keperluan monitoring | Sistem |

### 12.5 User Stories

| ID | Story |
|---|---|
| US-N001 | Sebagai Verifikator, saya ingin menerima notifikasi in-app dan email ketika ada pengajuan pemakaian BMN baru yang menunggu review saya, beserta link langsung ke halaman review, sehingga saya dapat segera memprosesnya. |
| US-N002 | Sebagai Pegawai, saya ingin menerima reminder otomatis 30 hari sebelum izin pemakaian saya berakhir sehingga saya bisa mengajukan perpanjangan tepat waktu. |
| US-N003 | Sebagai Pengguna, saya ingin melihat semua notifikasi saya di satu tempat (notification center) dengan indikator belum dibaca sehingga saya tidak melewatkan informasi penting. |
| US-N004 | Sebagai Pengguna, saya ingin mengatur preferensi notifikasi (misalnya mematikan notifikasi email untuk notifikasi INFO) sehingga saya tidak terganggu oleh notifikasi yang tidak penting. |
| US-N005 | Sebagai Admin Pusat, saya ingin mengirim notifikasi broadcast ke seluruh Operator Satker ketika periode pengumpulan kebutuhan BMN dibuka sehingga semua satker mengetahui. |

---

## 13. Layanan 9 — Autentikasi & Otorisasi (`authenc`)

### 13.1 Deskripsi Layanan

Layanan Autentikasi & Otorisasi (authenc) mengelola identitas pengguna, proses login, manajemen sesi, pengelolaan role dan permission, serta pencatatan audit trail. Layanan ini menjadi gerbang keamanan untuk seluruh layanan lain dalam ekosistem SIMPEL.

### 13.2 Entitas (Entities)

#### Pengguna (User)

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `username` | String(50) | Ya | Username untuk login |
| `password_hash` | String(255) | Ya | Hash password (Argon2id) |
| `nip` | String(18) | Ya | NIP pegawai (link ke MySIMKARI) |
| `nama_lengkap` | String(255) | Ya | Nama lengkap |
| `email` | String(255) | Ya | Email dinas |
| `nomor_telepon` | String(20) | Tidak | Nomor telepon |
| `satker_id` | UUID (FK→E01) | Ya | Satker pengguna |
| `is_aktif` | Boolean | Ya | Status aktif |
| `is_locked` | Boolean | Ya | Status terkunci (karena gagal login berulang) |
| `locked_until` | Timestamp | Tidak | Waktu unlock otomatis |
| `last_login` | Timestamp | Tidak | Waktu login terakhir |
| `last_password_change` | Timestamp | Ya | Waktu perubahan password terakhir |
| `force_password_change` | Boolean | Ya | Apakah harus ganti password saat login berikutnya |
| `mfa_enabled` | Boolean | Ya | Apakah MFA diaktifkan |
| `mfa_secret` | String(255) | Tidak | Secret untuk TOTP |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |
| `created_by` | UUID | Ya | ID pengguna pembuat |

#### Satuan Kerja (E01)

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode_satker` | String(20) | Ya | Kode satker internal SIMPEL |
| `kode_satker_monsakti` | String(20) | Tidak | Kode satker di MonSAKTI |
| `kode_satker_mysimkari` | String(20) | Tidak | Kode satker di MySIMKARI |
| `nama` | String(255) | Ya | Nama satker |
| `tipe` | Enum | Ya | `KEJAGUNG`, `KEJATI`, `KEJARI`, `CABJARI`, `UPT` |
| `parent_id` | UUID (FK→self) | Tidak | Satker induk (Kejati untuk Kejari, Kejari untuk Cabjari) |
| `alamat` | Text | Tidak | Alamat satker |
| `provinsi` | String(50) | Tidak | Provinsi |
| `kabupaten_kota` | String(100) | Tidak | Kabupaten/Kota |
| `is_aktif` | Boolean | Ya | Status aktif |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |

#### Role

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `kode` | String(50) | Ya | Kode role unik |
| `nama` | String(100) | Ya | Nama role |
| `deskripsi` | Text | Tidak | Deskripsi role |
| `level` | Enum | Ya | `PUSAT`, `WILAYAH`, `SATKER` |
| `is_system` | Boolean | Ya | Apakah role bawaan sistem (tidak bisa dihapus) |
| `permissions` | Array(String) | Ya | Daftar permission yang dimiliki role |
| `created_at` | Timestamp | Ya | Waktu pembuatan |
| `updated_at` | Timestamp | Ya | Waktu pembaruan terakhir |

#### User-Role Assignment

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `user_id` | UUID (FK) | Ya | ID pengguna |
| `role_id` | UUID (FK) | Ya | ID role |
| `satker_scope_id` | UUID (FK→E01) | Tidak | Satker di mana role ini berlaku (null = global) |
| `assigned_by` | UUID | Ya | ID pengguna yang memberikan role |
| `assigned_at` | Timestamp | Ya | Waktu pemberian role |
| `valid_from` | Date | Ya | Tanggal mulai berlaku |
| `valid_until` | Date | Tidak | Tanggal berakhir (null = tidak terbatas) |
| `is_aktif` | Boolean | Ya | Status aktif |

#### Audit Trail

| Atribut | Tipe Data | Wajib | Deskripsi |
|---|---|---|---|
| `id` | UUID | Ya | Identifier unik |
| `user_id` | UUID | Ya | ID pengguna yang melakukan aksi |
| `username` | String(50) | Ya | Username (disimpan terpisah untuk immutability) |
| `aksi` | String(50) | Ya | Jenis aksi (CREATE, READ, UPDATE, DELETE, LOGIN, LOGOUT, APPROVE, REJECT, dll.) |
| `layanan` | String(50) | Ya | Nama layanan di mana aksi dilakukan |
| `entitas_tipe` | String(50) | Ya | Tipe entitas yang terdampak |
| `entitas_id` | UUID | Tidak | ID entitas yang terdampak |
| `data_sebelum` | JSONB | Tidak | Snapshot data sebelum perubahan |
| `data_sesudah` | JSONB | Tidak | Snapshot data setelah perubahan |
| `ip_address` | String(45) | Ya | IP address pengguna |
| `user_agent` | String(500) | Tidak | User agent browser |
| `satker_id` | UUID | Tidak | Satker pengguna saat aksi dilakukan |
| `timestamp` | Timestamp | Ya | Waktu aksi |

### 13.3 Aktivitas (Activities)

| ID | Aktivitas | Aktor | Deskripsi |
|---|---|---|---|
| ACT-A01 | Login | Semua Pengguna | Masuk ke sistem dengan username dan password |
| ACT-A02 | Logout | Semua Pengguna | Keluar dari sistem |
| ACT-A03 | Ganti password | Semua Pengguna | Mengubah password sendiri |
| ACT-A04 | Reset password | Admin Pusat, Admin Wilayah | Mereset password pengguna |
| ACT-A05 | Kelola pengguna | Admin Pusat, Admin Wilayah | CRUD pengguna (tambah, ubah, nonaktifkan) |
| ACT-A06 | Kelola role | Super Admin, Admin Pusat | CRUD role dan permission |
| ACT-A07 | Assign/revoke role | Admin Pusat, Admin Wilayah | Memberikan atau mencabut role pengguna |
| ACT-A08 | Kelola satker | Super Admin, Admin Pusat | CRUD data satuan kerja |
| ACT-A09 | Lihat audit trail | Admin Pusat, Auditor | Melihat dan mencari log audit trail |
| ACT-A10 | Konfigurasi MFA | Semua Pengguna, Admin Pusat | Mengaktifkan/menonaktifkan MFA |
| ACT-A11 | Kelola sesi | Semua Pengguna | Melihat dan mengakhiri sesi aktif |

### 13.4 Kebutuhan Fungsional (Requirements)

| ID | Prioritas | Kebutuhan | Aktor Terkait |
|---|---|---|---|
| REQ-A001 | Tinggi | Sistem harus menyediakan autentikasi berbasis username/password dengan hashing Argon2id | Semua |
| REQ-A002 | Tinggi | Sistem harus menerapkan kebijakan password: minimal 12 karakter, kombinasi huruf besar-kecil-angka-simbol, tidak boleh sama dengan 5 password terakhir | Semua |
| REQ-A003 | Tinggi | Sistem harus mengunci akun setelah 5 kali gagal login berturut-turut selama 30 menit, dengan notifikasi ke email pengguna | Sistem |
| REQ-A004 | Tinggi | Sistem harus menggunakan JWT (JSON Web Token) dengan refresh token untuk manajemen sesi, dengan access token berumur pendek (15 menit) dan refresh token berumur panjang (7 hari) | Sistem |
| REQ-A005 | Tinggi | Sistem harus menerapkan Role-Based Access Control (RBAC) di mana setiap API endpoint dilindungi berdasarkan permission yang dimiliki role pengguna | Sistem |
| REQ-A006 | Tinggi | Sistem harus menerapkan scope access berdasarkan hierarki satker: pengguna hanya dapat mengakses data satker sendiri dan satker di bawahnya (jika ada) | Sistem |
| REQ-A007 | Tinggi | Sistem harus mencatat setiap aksi signifikan (login, logout, CRUD, approval, perubahan data) dalam audit trail yang immutable | Sistem |
| REQ-A008 | Tinggi | Sistem harus menyediakan CRUD pengguna dengan kemampuan mengaitkan pengguna dengan data pegawai dari MySIMKARI melalui NIP | Admin |
| REQ-A009 | Sedang | Sistem harus mendukung Multi-Factor Authentication (MFA) menggunakan TOTP (Time-based One-Time Password) | Semua |
| REQ-A010 | Sedang | Sistem harus menerapkan wajib ganti password setiap 90 hari dengan notifikasi pengingat H-14 sebelumnya | Sistem |
| REQ-A011 | Sedang | Sistem harus menyediakan fitur pencarian dan filter audit trail berdasarkan pengguna, aksi, layanan, entitas, dan rentang waktu | Admin, Auditor |
| REQ-A012 | Sedang | Sistem harus mendukung ekspor audit trail ke format Excel/CSV untuk keperluan audit eksternal | Auditor |
| REQ-A013 | Sedang | Sistem harus menerapkan session management di mana pengguna dapat melihat sesi aktifnya dan mengakhiri sesi di perangkat lain | Semua |
| REQ-A014 | Sedang | Sistem harus menerapkan rate limiting pada endpoint login untuk mencegah brute force attack | Sistem |
| REQ-A015 | Rendah | Sistem harus mendukung SSO (Single Sign-On) untuk integrasi dengan sistem identitas terpusat Kejaksaan jika tersedia di masa mendatang | Sistem |
| REQ-A016 | Tinggi | Audit trail tidak boleh dapat diubah atau dihapus oleh siapapun termasuk Super Admin | Sistem |
| REQ-A017 | Sedang | Sistem harus menyediakan hierarki satker yang navigable (tree view) untuk memudahkan Admin dalam mengelola pengguna dan role per satker | Admin |

### 13.5 Siklus Hidup (Lifecycle)

#### Lifecycle Pengguna

```
DIBUAT → AKTIF → TERKUNCI (gagal login) → AKTIF (auto-unlock / manual unlock)
       → AKTIF → NONAKTIF (dinonaktifkan admin / pegawai pensiun/mutasi)
                → AKTIF (diaktifkan kembali)
```

#### Lifecycle Sesi

```
LOGIN → AKTIF → EXPIRED (timeout) → ENDED
              → LOGOUT → ENDED
              → REVOKED (admin / perangkat lain) → ENDED
```

### 13.6 User Stories

| ID | Story |
|---|---|
| US-A001 | Sebagai Pengguna, saya ingin login ke SIMPEL menggunakan username dan password sehingga saya dapat mengakses fitur sesuai role saya. |
| US-A002 | Sebagai Admin Pusat, saya ingin membuat akun pengguna baru dengan menautkannya ke NIP pegawai dari MySIMKARI sehingga data pengguna konsisten dengan data kepegawaian. |
| US-A003 | Sebagai Admin Wilayah, saya ingin memberikan role "Operator Satker" kepada pengguna di Kejari dalam wilayah saya sehingga mereka dapat menginput data kebutuhan BMN. |
| US-A004 | Sebagai Pengguna, saya ingin mengubah password saya sendiri dan mengaktifkan MFA sehingga akun saya lebih aman. |
| US-A005 | Sebagai Auditor, saya ingin mencari audit trail berdasarkan rentang waktu dan jenis aksi untuk satker tertentu sehingga saya dapat memeriksa kepatuhan prosedur. |
| US-A006 | Sebagai Admin Pusat, saya ingin melihat hierarki satker dalam bentuk tree view dan mengelola pengguna per satker sehingga pengelolaan akses lebih terstruktur. |
| US-A007 | Sebagai Pengguna, saya ingin melihat sesi aktif saya di perangkat lain dan mengakhirinya jika perlu sehingga saya bisa mengamankan akun jika perangkat hilang. |
| US-A008 | Sebagai Sistem, saya ingin secara otomatis menonaktifkan akun pengguna yang berstatus pensiun di MySIMKARI sehingga tidak ada akun orphan yang masih aktif. |

---

## 14. Kebutuhan Non-Fungsional

### 14.1 Performa

| ID | Kebutuhan | Target |
|---|---|---|
| NFR-P001 | Response time halaman web pada kondisi normal | ≤ 2 detik (90th percentile) |
| NFR-P002 | Response time API pada kondisi normal | ≤ 500ms (95th percentile) |
| NFR-P003 | Waktu generate dokumen PDF | ≤ 5 detik per dokumen |
| NFR-P004 | Waktu load dashboard dengan data nasional | ≤ 5 detik |
| NFR-P005 | Throughput API | Minimal 100 request/detik secara bersamaan |
| NFR-P006 | Delay data pada dashboard | Maksimal 1 jam dari data sumber (near real-time) |
| NFR-P007 | Waktu sinkronisasi penuh MonSAKTI | ≤ 4 jam (saat off-peak) |
| NFR-P008 | Waktu sinkronisasi inkremental | ≤ 30 menit |

### 14.2 Ketersediaan & Keandalan

| ID | Kebutuhan | Target |
|---|---|---|
| NFR-A001 | Uptime sistem | 99.5% (tidak termasuk maintenance terjadwal) |
| NFR-A002 | Maintenance window | Maksimal 4 jam per bulan, di luar jam kerja |
| NFR-A003 | Recovery Time Objective (RTO) | ≤ 4 jam |
| NFR-A004 | Recovery Point Objective (RPO) | ≤ 1 jam (kehilangan data maksimal 1 jam terakhir) |
| NFR-A005 | Backup database | Harian (full backup) + continuous WAL archiving |

### 14.3 Keamanan

| ID | Kebutuhan | Target |
|---|---|---|
| NFR-S001 | Enkripsi data in transit | TLS 1.2 atau lebih tinggi |
| NFR-S002 | Enkripsi data at rest | AES-256 untuk data sensitif |
| NFR-S003 | Password hashing | Argon2id |
| NFR-S004 | Session management | JWT dengan access token 15 menit, refresh token 7 hari |
| NFR-S005 | OWASP Top 10 | Mitigasi seluruh kerentanan OWASP Top 10 |
| NFR-S006 | Audit trail retensi | Minimal 5 tahun, immutable |
| NFR-S007 | Rate limiting | Login: 5 attempt/menit per IP; API: 100 request/menit per user |
| NFR-S008 | Input validation | Validasi semua input di sisi server; sanitasi output |

### 14.4 Skalabilitas

| ID | Kebutuhan | Target |
|---|---|---|
| NFR-SC001 | Jumlah pengguna concurrent | Minimal 500 pengguna bersamaan |
| NFR-SC002 | Jumlah data BMN | Mampu mengelola ≥ 1 juta record BMN |
| NFR-SC003 | Jumlah data pegawai | Mampu mengelola ≥ 50.000 record pegawai |
| NFR-SC004 | Horizontal scaling | Setiap microservice harus dapat di-scale horizontal secara independen |

### 14.5 Usability

| ID | Kebutuhan | Target |
|---|---|---|
| NFR-U001 | Responsive design | Mendukung desktop (≥ 1024px) dan tablet (≥ 768px) |
| NFR-U002 | Browser support | Chrome, Firefox, Edge (2 versi terbaru) |
| NFR-U003 | Bahasa antarmuka | Bahasa Indonesia |
| NFR-U004 | Aksesibilitas | WCAG 2.1 Level AA |
| NFR-U005 | Waktu pelatihan pengguna | Pengguna baru mampu melakukan tugas dasar setelah ≤ 2 jam pelatihan |

### 14.6 Maintainability

| ID | Kebutuhan | Target |
|---|---|---|
| NFR-M001 | Dokumentasi API | Setiap layanan memiliki dokumentasi OpenAPI/Swagger |
| NFR-M002 | Logging | Structured logging (JSON) ke centralized log management |
| NFR-M003 | Health check | Setiap layanan menyediakan endpoint health check (`/health`) |
| NFR-M004 | Monitoring | Metrics Prometheus + Grafana untuk setiap layanan |
| NFR-M005 | Database migration | Setiap perubahan schema melalui migration script yang versioned |
| NFR-M006 | Code coverage | Minimal 70% unit test coverage |

---

## 15. Matriks Ketergantungan Antar Layanan

### 15.1 Matriks Dependensi (Baris mengkonsumsi layanan Kolom)

| Konsumer ↓ / Provider → | master | kebutuhan | pemakaian | workflow | dokumen | dashboard | integrasi | notifikasi | authenc |
|---|---|---|---|---|---|---|---|---|---|
| **master** | — | | | | | | ✓ | | ✓ |
| **kebutuhan** | ✓ | — | | ✓ | ✓ | | ✓ | ✓ | ✓ |
| **pemakaian** | ✓ | | — | ✓ | ✓ | | ✓ | ✓ | ✓ |
| **workflow** | | | | — | | | | ✓ | ✓ |
| **dokumen** | | | | | — | | | | ✓ |
| **dashboard** | ✓ | ✓ | ✓ | ✓ | | — | ✓ | | ✓ |
| **integrasi** | | | | | | | — | ✓ | ✓ |
| **notifikasi** | | | | | | | | — | ✓ |
| **authenc** | | | | | | | ✓ | | — |

### 15.2 Urutan Deployment

Berdasarkan dependensi, urutan deployment yang direkomendasikan:

1. `authenc` — tidak bergantung pada layanan SIMPEL lain
2. `notifikasi` — hanya bergantung pada authenc
3. `integrasi` — bergantung pada authenc dan notifikasi
4. `master` — bergantung pada authenc dan integrasi
5. `workflow` — bergantung pada authenc dan notifikasi
6. `dokumen` — bergantung pada authenc
7. `kebutuhan` — bergantung pada banyak layanan
8. `pemakaian` — bergantung pada banyak layanan
9. `dashboard` — bergantung pada hampir semua layanan (deploy terakhir)

---

## 16. Glosarium

| Istilah | Singkatan | Definisi |
|---|---|---|
| Barang Milik Negara | BMN | Semua barang yang dibeli atau diperoleh atas beban APBN atau berasal dari perolehan lainnya yang sah |
| Standar Barang dan Standar Kebutuhan | SBSK | Standar barang dan kebutuhan yang ditetapkan melalui Peraturan Menteri Keuangan |
| Rencana Kebutuhan BMN | RKBMN | Dokumen perencanaan kebutuhan BMN untuk periode tertentu |
| Sistem Informasi Manajemen Aset Negara | SIMAN | Aplikasi pengelolaan aset negara yang dikelola oleh DJKN Kementerian Keuangan |
| Monitoring SAKTI | MonSAKTI | Sistem monitoring pelaksanaan anggaran Kementerian Keuangan |
| MySIMKARI | — | Sistem informasi manajemen kepegawaian Kejaksaan RI |
| Nomor Urut Pendaftaran | NUP | Nomor urut pencatatan BMN dalam register |
| Satuan Kerja | Satker | Unit organisasi yang mengelola anggaran dan BMN |
| Pakaian Dinas Harian | PDH | Pakaian dinas untuk kegiatan sehari-hari |
| Pakaian Dinas Lapangan | PDL | Pakaian dinas untuk kegiatan di lapangan |
| Pejabat Penatausahaan BMN | PPBMN | Pejabat yang bertanggung jawab atas penatausahaan BMN di satker |
| Surat Perintah Pencairan Dana | SP2D | Surat perintah yang diterbitkan oleh KPPN untuk pencairan anggaran |
| Service Level Agreement | SLA | Batas waktu yang ditetapkan untuk penyelesaian suatu proses |
| Konstruksi Dalam Pengerjaan | KDP | Aset tetap yang masih dalam proses pembangunan |
| Kartu Inventaris Barang | KIB | Kartu pencatatan detail setiap unit BMN |
| Time-based One-Time Password | TOTP | Metode autentikasi dua faktor berbasis waktu |
| JSON Web Token | JWT | Standar terbuka untuk token autentikasi |
| Role-Based Access Control | RBAC | Model kontrol akses berdasarkan role/peran |

---

**Catatan Dokumen:**

Dokumen ini bersifat draft dan memerlukan validasi dari pemangku kepentingan terkait, khususnya Biro Perlengkapan Kejaksaan Agung RI, sebelum dijadikan acuan pengembangan. Perubahan kebutuhan di masa mendatang akan didokumentasikan melalui mekanisme change request yang terpisah.

---

*Disusun oleh: Tim Pengembang SIMPEL*  
*Tanggal: 9 Februari 2026*  
*Status: DRAFT v1.0.0*
