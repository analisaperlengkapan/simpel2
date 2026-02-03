# Requirements Document - Migrasi Modul Perlengkapan

**Versi:** 1.0  
**Tanggal:** 2 Februari 2026  
**Status:** Draft  
**Project:** SIMPelv2 - Migrasi Modul Perlengkapan dari Laravel ke Rust

## Change History

| Versi | Tanggal | Perubahan | Author |
|-------|---------|-----------|--------|
| 1.0 | 2026-02-02 | Initial requirements document | AI Agent |

## Daftar Isi

1. [Project Overview](#1-project-overview)
2. [User Roles & Permissions](#2-user-roles--permissions)
3. [Functional Requirements](#3-functional-requirements)
4. [Non-Functional Requirements](#4-non-functional-requirements)
5. [Data Requirements](#5-data-requirements)
6. [Integration Requirements](#6-integration-requirements)
7. [Testing Requirements](#7-testing-requirements)
8. [Constraints & Assumptions](#8-constraints--assumptions)
9. [Success Criteria](#9-success-criteria)

---

## 1. Project Overview

### 1.1 Project Goals

Migrasi lengkap modul **Perlengkapan** dari sistem Laravel legacy ke arsitektur Rust modern dengan tujuan:

1. **Performance**: Meningkatkan performa 10x dengan Rust + WASM
2. **Security**: Zero-trust security dengan gRPC mTLS
3. **Scalability**: Microservices architecture untuk horizontal scaling
4. **Maintainability**: Type-safe code dengan Rust compiler guarantees
5. **Compliance**: FIPS, GDPR-aligned, government security standards

### 1.2 Target Users

| User Type | Deskripsi | Jumlah Estimasi |
|-----------|-----------|-----------------|
| Admin Perlengkapan | Pengelola BMN tingkat Kejaksaan | ~500 |
| Kepala Biro Perlengkapan | Approver tingkat tinggi | ~50 |
| Jaksa Agung Muda Pembinaan | Final approver untuk penghapusan | ~10 |
| Operator Satker | Input data di satuan kerja | ~2000 |
| Viewer/Auditor | Read-only access untuk audit | ~100 |

### 1.3 Scope

#### 1.3.1 In Scope

- Migrasi 30+ controllers dari Laravel ke Rust/Axum handlers
- Migrasi 35+ models ke Rust structs dengan PostgreSQL
- Migrasi 80+ routes ke Axum REST API
- Implementasi full CRUD (Create, Read, Update, Delete)
- Workflow approval multi-level
- File attachments (upload/download)
- Report generation (PDF, Excel)
- Unit tests untuk setiap file
- E2E tests dengan Playwright

#### 1.3.2 Out of Scope

- Migrasi data legacy (handled by separate ETL process)
- Mobile application
- Offline mode

### 1.4 Tech Stack

| Layer | Technology | Version |
|-------|------------|---------|
| **Frontend** | Leptos (WASM CSR) | 0.8.x |
| **Backend** | Axum + Tonic | 0.8.x / 0.14.x |
| **Database** | PostgreSQL | 15+ |
| **Cache** | Redis | 7+ |
| **Auth** | Authenc (gRPC) | Internal |
| **Secrets** | Secreton (gRPC) | Internal |
| **Testing** | Playwright + cargo test | Latest |

### 1.5 Project Structure

```
antarmuka/pembinaan/perlengkapan/     # Frontend Microfrontend
├── src/
│   ├── components/                   # UI Components
│   ├── api.rs                        # API Client
│   ├── lib.rs                        # Main entry
│   └── tests.rs                      # Unit tests
├── Cargo.toml
└── Trunk.toml

layanan/pembinaan/perlengkapan/       # Backend Microservice
├── src/
│   ├── handlers.rs                   # API Handlers
│   ├── models.rs                     # Data Models
│   ├── repository.rs                 # Database Layer
│   ├── services.rs                   # Business Logic
│   ├── routes.rs                     # Route Definitions
│   └── tests/                        # Unit & Integration Tests
├── migrations/                       # Database Migrations
└── Cargo.toml

e2e/tests/perlengkapan/               # E2E Tests
├── *.spec.ts                         # Playwright Test Files
└── fixtures/                         # Test Data
```

---

## 2. User Roles & Permissions

### 2.1 Operator Satker

**Deskripsi:** Penginput data di tingkat satuan kerja

| Permission | Create | Read | Update | Delete |
|------------|--------|------|--------|--------|
| Asset | ❌ | ✅ Own Satker | ❌ | ❌ |
| Pengadaan | ✅ Draft | ✅ Own | ✅ Draft | ✅ Draft |
| Pemakaian | ✅ | ✅ Own | ✅ Own | ✅ Own |
| Pemeliharaan | ✅ | ✅ Own | ✅ Own | ❌ |
| Penghapusan | ✅ Draft | ✅ Own | ✅ Draft | ✅ Draft |
| Hibah | ✅ Draft | ✅ Own | ✅ Draft | ✅ Draft |
| Mutasi | ✅ Draft | ✅ Own | ✅ Draft | ✅ Draft |
| Pakaian Dinas | ✅ | ✅ Own | ✅ Own | ❌ |
| Laporan | ❌ | ✅ Own | ❌ | ❌ |

### 2.2 Admin Perlengkapan

**Deskripsi:** Pengelola BMN tingkat Kejaksaan (Wilayah/Pusat)

| Permission | Create | Read | Update | Delete |
|------------|--------|------|--------|--------|
| Asset | ❌ | ✅ All | ❌ | ❌ |
| Pengadaan | ✅ | ✅ All | ✅ Pending | ❌ |
| Pemakaian | ✅ | ✅ All | ✅ | ✅ |
| Pemeliharaan | ✅ | ✅ All | ✅ | ✅ |
| Penghapusan | ✅ | ✅ All | ✅ Pending | ❌ |
| BMN Izin | ✅ | ✅ All | ✅ | ✅ |
| Hibah | ✅ | ✅ All | ✅ Pending | ❌ |
| Mutasi | ✅ | ✅ All | ✅ | ✅ |
| Pakaian Dinas | ✅ | ✅ All | ✅ | ✅ |
| Analisis Kebutuhan | ✅ | ✅ All | ✅ | ✅ |
| Laporan | ✅ | ✅ All | ❌ | ❌ |
| Workflow Approval | ✅ Level 1 | ✅ | ❌ | ❌ |

### 2.3 Kepala Biro Perlengkapan

**Deskripsi:** Approver tingkat tinggi untuk operasi BMN kritis

| Permission | Create | Read | Update | Delete |
|------------|--------|------|--------|--------|
| All Modules | ✅ | ✅ All | ✅ | ✅ |
| Workflow Approval | ✅ Level 2 | ✅ | ❌ | ❌ |
| Penghapusan Final | ✅ Approve/Reject | ✅ | ❌ | ❌ |
| SK Penghapusan | ✅ Generate | ✅ | ❌ | ❌ |

### 2.4 Jaksa Agung Muda Pembinaan

**Deskripsi:** Final approver untuk penghapusan BMN nilai tinggi

| Permission | Create | Read | Update | Delete |
|------------|--------|------|--------|--------|
| Penghapusan BMN | ✅ Final Approval | ✅ All | ❌ | ❌ |
| SK Penghapusan | ✅ Sign | ✅ | ❌ | ❌ |
| Dashboard Executive | ❌ | ✅ | ❌ | ❌ |

### 2.5 Auditor

**Deskripsi:** Read-only access untuk keperluan audit

| Permission | Create | Read | Update | Delete |
|------------|--------|------|--------|--------|
| All Modules | ❌ | ✅ All | ❌ | ❌ |
| Audit Trail | ❌ | ✅ | ❌ | ❌ |
| Report Export | ❌ | ✅ | ❌ | ❌ |

---

## 3. Functional Requirements

### 3.1 Complete CRUD Operations

**Prioritas:** 🔴 Tinggi

Semua entity yang existing harus memiliki full CRUD operations.

#### 3.1.1 Pengadaan CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.1.1 | Create Pengadaan | ✅ Sudah ada | Maintain |
| REQ-3.1.1.2 | Read Pengadaan (List + Detail) | ✅ Sudah ada | Maintain |
| REQ-3.1.1.3 | Update Pengadaan | ❌ Belum ada | **Implement** |
| REQ-3.1.1.4 | Delete Pengadaan (Soft Delete) | ❌ Belum ada | **Implement** |
| REQ-3.1.1.5 | Update Sub-documents (HPS, SKPPBJ, SPK, dll) | ❌ Belum ada | **Implement** |
| REQ-3.1.1.6 | Delete Sub-documents | ❌ Belum ada | **Implement** |

**Acceptance Criteria:**
- PUT `/pengadaan/{id}` returns 200 dengan data updated
- DELETE `/pengadaan/{id}` returns 204 (soft delete, set deleted_at)
- Validasi: Hanya status DRAFT yang bisa di-update/delete
- Audit log tercatat untuk setiap operasi

#### 3.1.2 Analisis Kebutuhan CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.2.1 | Create Analisis | ✅ Sudah ada | Maintain |
| REQ-3.1.2.2 | Read Analisis | ✅ Sudah ada | Maintain |
| REQ-3.1.2.3 | Update Analisis | ❌ Belum ada | **Implement** |
| REQ-3.1.2.4 | Delete Analisis | ❌ Belum ada | **Implement** |

#### 3.1.3 Pemakaian CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.3.1 | Create Pemakaian | ✅ Sudah ada | Maintain |
| REQ-3.1.3.2 | Read Pemakaian | ✅ Sudah ada | Maintain |
| REQ-3.1.3.3 | Update Pemakaian | ❌ Belum ada | **Implement** |
| REQ-3.1.3.4 | Delete Pemakaian | ❌ Belum ada | **Implement** |
| REQ-3.1.3.5 | Return Asset (Pengembalian) | ❌ Belum ada | **Implement** |

#### 3.1.4 Hibah CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.4.1 | Create Hibah | ✅ Sudah ada | Maintain |
| REQ-3.1.4.2 | Read Hibah | ✅ Sudah ada | Maintain |
| REQ-3.1.4.3 | Update Hibah | ❌ Belum ada | **Implement** |
| REQ-3.1.4.4 | Delete Hibah | ❌ Belum ada | **Implement** |

#### 3.1.5 Mutasi CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.5.1 | Create Mutasi | ✅ Sudah ada | Maintain |
| REQ-3.1.5.2 | Read Mutasi | ✅ Sudah ada | Maintain |
| REQ-3.1.5.3 | Update Mutasi | ❌ Belum ada | **Implement** |
| REQ-3.1.5.4 | Delete Mutasi | ❌ Belum ada | **Implement** |
| REQ-3.1.5.5 | Approve/Reject Mutasi | ❌ Belum ada | **Implement** |

#### 3.1.6 Penghapusan CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.6.1 | Create Penghapusan | ✅ Sudah ada | Maintain |
| REQ-3.1.6.2 | Read Penghapusan | ✅ Sudah ada | Maintain |
| REQ-3.1.6.3 | Update Penghapusan | ❌ Belum ada | **Implement** |
| REQ-3.1.6.4 | Delete Penghapusan | ❌ Belum ada | **Implement** |

#### 3.1.7 Pengalihan CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.7.1 | Create Pengalihan | ✅ Sudah ada | Maintain |
| REQ-3.1.7.2 | Read Pengalihan | ✅ Sudah ada | Maintain |
| REQ-3.1.7.3 | Update Pengalihan | ❌ Belum ada | **Implement** |
| REQ-3.1.7.4 | Delete Pengalihan | ❌ Belum ada | **Implement** |

#### 3.1.8 Pemeliharaan CRUD Complete

| ID | Requirement | Status Existing | Target |
|----|-------------|-----------------|--------|
| REQ-3.1.8.1 | Create Pemeliharaan | ✅ Sudah ada | Maintain |
| REQ-3.1.8.2 | Read Pemeliharaan | ✅ Sudah ada | Maintain |
| REQ-3.1.8.3 | Update Pemeliharaan | ❌ Belum ada | **Implement** |
| REQ-3.1.8.4 | Delete Pemeliharaan | ❌ Belum ada | **Implement** |

### 3.2 Workflow Approval System

**Prioritas:** 🔴 Tinggi

#### 3.2.1 Multi-Level Approval Engine

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.2.1.1 | Approval Configuration | Konfigurasi level approval per jenis transaksi |
| REQ-3.2.1.2 | Approval Chain | Definisi urutan approver |
| REQ-3.2.1.3 | Auto-escalation | Eskalasi otomatis jika tidak diproses dalam SLA |
| REQ-3.2.1.4 | Delegation | Delegasi approval ke user lain |
| REQ-3.2.1.5 | Parallel Approval | Approval paralel untuk kasus tertentu |

**Approval Flow States:**
```
DRAFT → SUBMITTED → LEVEL1_PENDING → LEVEL1_APPROVED → 
LEVEL2_PENDING → LEVEL2_APPROVED → FINAL_APPROVED / REJECTED
```

#### 3.2.2 Penghapusan BMN Approval

| ID | Level | Approver | Threshold |
|----|-------|----------|-----------|
| REQ-3.2.2.1 | Level 1 | Admin Perlengkapan | Semua pengajuan |
| REQ-3.2.2.2 | Level 2 | Kepala Biro Perlengkapan | Nilai > 50 Juta |
| REQ-3.2.2.3 | Level 3 | Jaksa Agung Muda Pembinaan | Nilai > 500 Juta |

#### 3.2.3 Hibah BMN Approval

| ID | Level | Approver | Threshold |
|----|-------|----------|-----------|
| REQ-3.2.3.1 | Level 1 | Admin Perlengkapan | Semua pengajuan |
| REQ-3.2.3.2 | Level 2 | Kepala Biro Perlengkapan | Nilai > 100 Juta |

#### 3.2.4 Mutasi BMN Approval

| ID | Level | Approver | Threshold |
|----|-------|----------|-----------|
| REQ-3.2.4.1 | Level 1 | Admin Perlengkapan Asal | Semua pengajuan |
| REQ-3.2.4.2 | Level 2 | Admin Perlengkapan Tujuan | Semua pengajuan |

### 3.3 File Attachments

**Prioritas:** 🔴 Tinggi

#### 3.3.1 File Upload System

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.3.1.1 | Single File Upload | Upload file tunggal per field |
| REQ-3.3.1.2 | Multi File Upload | Upload multiple files per entity |
| REQ-3.3.1.3 | File Type Validation | Validasi: PDF, JPG, PNG, DOCX, XLSX |
| REQ-3.3.1.4 | File Size Limit | Max 10MB per file, 50MB per request |
| REQ-3.3.1.5 | Virus Scan | Scan malware sebelum storage |
| REQ-3.3.1.6 | Secure Storage | Encrypted storage dengan presigned URLs |

#### 3.3.2 File Categories per Module

| Module | File Types Required |
|--------|---------------------|
| Penghapusan | SK, Berita Acara, Foto Kondisi, Dokumen Pendukung |
| Hibah | SK Hibah, Berita Acara Serah Terima, Foto |
| Pengadaan | HPS, SKPPBJ, SPK, Kontrak, BAST, Invoice |
| Pemeliharaan | Laporan Pemeliharaan, Foto Before/After, Invoice |
| BMN Izin | Surat Permohonan, SK Izin, Foto |

#### 3.3.3 File Download & Preview

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.3.3.1 | Secure Download | Download dengan token temporary |
| REQ-3.3.3.2 | PDF Preview | In-browser PDF preview |
| REQ-3.3.3.3 | Image Preview | Thumbnail dan full-size preview |
| REQ-3.3.3.4 | Batch Download | Download ZIP untuk multiple files |

### 3.4 BMN Izin (Izin Penggunaan BMN)

**Prioritas:** 🟡 Sedang

#### 3.4.1 Izin Penggunaan BMN

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.4.1.1 | Create Permohonan Izin | Buat permohonan izin penggunaan BMN |
| REQ-3.4.1.2 | List Permohonan Izin | Daftar permohonan dengan filter & pagination |
| REQ-3.4.1.3 | Detail Permohonan Izin | Detail lengkap permohonan |
| REQ-3.4.1.4 | Update Permohonan Izin | Edit permohonan (status DRAFT) |
| REQ-3.4.1.5 | Cancel Permohonan Izin | Batalkan permohonan |
| REQ-3.4.1.6 | Approve/Reject Izin | Proses approval |
| REQ-3.4.1.7 | Generate SK Izin | Generate surat keputusan izin |

#### 3.4.2 Izin Pegawai (Penggunaan oleh Pegawai)

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.4.2.1 | Create Izin Pegawai | Assign BMN ke pegawai |
| REQ-3.4.2.2 | List Izin Pegawai | Daftar pegawai yang menggunakan BMN |
| REQ-3.4.2.3 | Grid Data Pegawai | Grid data pegawai per pengajuan |
| REQ-3.4.2.4 | Grid Data Aset | Grid data aset per pengajuan |
| REQ-3.4.2.5 | Update Assignment | Update penugasan |
| REQ-3.4.2.6 | Revoke Assignment | Cabut penugasan |

#### 3.4.3 Izin Monitor (Monitoring Izin)

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.4.3.1 | Dashboard Monitoring | Overview status semua izin |
| REQ-3.4.3.2 | Expiring Soon Alert | Notifikasi izin akan berakhir |
| REQ-3.4.3.3 | Expired List | Daftar izin yang sudah expired |
| REQ-3.4.3.4 | Renewal Workflow | Proses perpanjangan izin |
| REQ-3.4.3.5 | History Log | Riwayat perubahan status |

### 3.5 Penghapusan BMN Extended

**Prioritas:** 🔴 Tinggi

#### 3.5.1 Pengajuan Penghapusan BMN

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.5.1.1 | Create Pengajuan | Buat pengajuan penghapusan BMN |
| REQ-3.5.1.2 | Add Asset to Pengajuan | Tambah aset ke pengajuan |
| REQ-3.5.1.3 | Remove Asset from Pengajuan | Hapus aset dari pengajuan |
| REQ-3.5.1.4 | Submit Pengajuan | Kirim pengajuan untuk approval |
| REQ-3.5.1.5 | Grid Data Asset | Grid data aset yang diajukan |
| REQ-3.5.1.6 | Upload Dokumen Pendukung | Upload file pendukung |

**Kategori Penghapusan:**
- Pemindahtanganan (Penjualan, Tukar Menukar, Hibah, PNBP)
- Pemusnahan (Barang rusak berat)
- Sebab Lain (Hilang, Dicuri, Force Majeure)

#### 3.5.2 Permohonan Penghapusan BMN

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.5.2.1 | Create Permohonan | Buat permohonan ke Pengelola Barang |
| REQ-3.5.2.2 | Grid Data Fotocopy | Grid data dokumen fotocopy |
| REQ-3.5.2.3 | Save Fotocopy | Simpan dokumen fotocopy |
| REQ-3.5.2.4 | Save Foto | Simpan foto kondisi BMN |
| REQ-3.5.2.5 | Submit Permohonan | Kirim permohonan |
| REQ-3.5.2.6 | Delete Fotocopy | Hapus dokumen fotocopy |
| REQ-3.5.2.7 | Delete Foto | Hapus foto |
| REQ-3.5.2.8 | Download ZIP | Download semua dokumen sebagai ZIP |

#### 3.5.3 Permohonan Penghapusan BMN SK

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.5.3.1 | Generate SK | Generate surat keputusan penghapusan |
| REQ-3.5.3.2 | Track Aktivitas SK | Tracking aktivitas SK |
| REQ-3.5.3.3 | Grid Data SK | Grid data SK penghapusan |
| REQ-3.5.3.4 | Upload SK Final | Upload SK yang sudah ditandatangani |

#### 3.5.4 Persetujuan Penghapusan BMN Monitor

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.5.4.1 | Dashboard Persetujuan | Dashboard monitoring persetujuan |
| REQ-3.5.4.2 | Grid Data Monitor | Grid data monitoring |
| REQ-3.5.4.3 | Upload File Persetujuan | Upload dokumen persetujuan |
| REQ-3.5.4.4 | Status Tracking | Tracking status approval |

#### 3.5.5 Penghapusan Monitor

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.5.5.1 | Dashboard Penghapusan | Overview semua penghapusan |
| REQ-3.5.5.2 | Grid Data | Grid data monitoring penghapusan |
| REQ-3.5.5.3 | Upload File Monitor | Upload file terkait monitoring |
| REQ-3.5.5.4 | Export Report | Export laporan penghapusan |

### 3.6 Pakaian Dinas

**Prioritas:** 🟡 Sedang

#### 3.6.1 Pengajuan Pakaian Dinas

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.6.1.1 | Create Pengajuan | Buat pengajuan pakaian dinas per satker |
| REQ-3.6.1.2 | List Pengajuan | Daftar pengajuan dengan filter & pagination |
| REQ-3.6.1.3 | Grid Data Satker | Grid data pengajuan per satker |
| REQ-3.6.1.4 | Grid Data Pegawai | Grid data pegawai per satker |
| REQ-3.6.1.5 | Submit Pengajuan | Kirim pengajuan untuk approval |
| REQ-3.6.1.6 | Approve/Reject | Proses approval pengajuan |
| REQ-3.6.1.7 | Track Aktivitas | Tracking aktivitas per satker |

#### 3.6.2 Ukuran Pakaian Pegawai

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.6.2.1 | Input Ukuran | Input ukuran pakaian pegawai |
| REQ-3.6.2.2 | Update Ukuran | Update ukuran yang sudah ada |
| REQ-3.6.2.3 | Bulk Import | Import ukuran dari Excel |
| REQ-3.6.2.4 | Validation | Validasi ukuran (S, M, L, XL, XXL, dll) |

#### 3.6.3 Laporan Pakaian Dinas

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.6.3.1 | Laporan per Tahun | Laporan pengadaan per tahun |
| REQ-3.6.3.2 | Cetak Laporan | Export PDF/Excel |
| REQ-3.6.3.3 | Rekap Satker | Rekapitulasi per satker |
| REQ-3.6.3.4 | Rekap Ukuran | Rekapitulasi per ukuran |

### 3.7 Aset per Kategori

**Prioritas:** 🟢 Rendah (Read-Only dari SIMAN, hanya view spesifik)

#### 3.7.1 Kategori Aset

| ID | Kategori | Deskripsi |
|----|----------|-----------|
| REQ-3.7.1.1 | Tanah | Aset tanah dengan koordinat lokasi |
| REQ-3.7.1.2 | Gedung | Bangunan gedung |
| REQ-3.7.1.3 | Rumah | Rumah dinas |
| REQ-3.7.1.4 | Angkutan | Kendaraan dinas |
| REQ-3.7.1.5 | Senjata | Aset senjata (restricted) |
| REQ-3.7.1.6 | Alat Besar | Alat berat dan mesin |
| REQ-3.7.1.7 | Jalan & Jembatan | Infrastruktur jalan |
| REQ-3.7.1.8 | Bangunan Air | Bangunan pengairan |
| REQ-3.7.1.9 | Jaringan | Instalasi dan jaringan |
| REQ-3.7.1.10 | Konstruksi | Konstruksi dalam pengerjaan |
| REQ-3.7.1.11 | TIK | Aset teknologi informasi |
| REQ-3.7.1.12 | Non-TIK | Peralatan non-TIK |
| REQ-3.7.1.13 | Lainnya | Aset lainnya |
| REQ-3.7.1.14 | Tak Berwujud | Aset tak berwujud (lisensi, dll) |
| REQ-3.7.1.15 | Renovasi | Aset dalam renovasi |

#### 3.7.2 Fitur per Kategori

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.7.2.1 | Grid Data per Kategori | Daftar aset dengan filter spesifik kategori |
| REQ-3.7.2.2 | Detail View | Detail aset dengan field spesifik kategori |
| REQ-3.7.2.3 | Cetak Label | Cetak label/QR Code per aset |
| REQ-3.7.2.4 | Export Excel | Export data ke Excel |
| REQ-3.7.2.5 | Export PDF | Export data ke PDF |
| REQ-3.7.2.6 | Maps Integration | Peta lokasi untuk Tanah & Gedung |

#### 3.7.3 QR Code Generator

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.7.3.1 | Generate QR | Generate QR Code per aset |
| REQ-3.7.3.2 | Batch Print | Cetak batch QR Code |
| REQ-3.7.3.3 | Scan & View | Scan QR untuk melihat detail aset |

#### 3.7.4 Peta Sebaran Aset

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.7.4.1 | Map View | Visualisasi lokasi aset di peta |
| REQ-3.7.4.2 | Cluster View | Clustering aset per wilayah |
| REQ-3.7.4.3 | Filter by Category | Filter berdasarkan kategori |
| REQ-3.7.4.4 | Info Window | Detail aset saat klik marker |

#### 3.7.5 Lapor Masalah Aset

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.7.5.1 | Create Laporan | Buat laporan masalah aset |
| REQ-3.7.5.2 | Attach Photo | Lampirkan foto kondisi |
| REQ-3.7.5.3 | Track Status | Tracking penanganan |
| REQ-3.7.5.4 | Assign Handler | Assign petugas penanganan |

### 3.8 Analisis Kebutuhan BMN Extended

**Prioritas:** 🟡 Sedang

#### 3.8.1 Daftar Kebutuhan BMN

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.8.1.1 | Create Daftar | Buat daftar kebutuhan BMN per satker |
| REQ-3.8.1.2 | Grid Data Satker | Grid data kebutuhan per satker |
| REQ-3.8.1.3 | Grid Data Barang | Grid data barang yang dibutuhkan |
| REQ-3.8.1.4 | Add Barang | Tambah item barang |
| REQ-3.8.1.5 | Remove Barang | Hapus item barang |
| REQ-3.8.1.6 | Submit | Kirim daftar kebutuhan |

#### 3.8.2 Analisis Kelayakan

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.8.2.1 | Grid Data Analisis | Grid data analisis kelayakan |
| REQ-3.8.2.2 | Grid Data Satker | Grid per satker |
| REQ-3.8.2.3 | Grid Data Barang | Grid per barang |
| REQ-3.8.2.4 | Grid Satker Aset | Grid aset per satker |
| REQ-3.8.2.5 | Input Kelayakan | Input hasil analisis kelayakan |

#### 3.8.3 Penyusunan Prioritas

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.8.3.1 | Grid Data Prioritas | Grid data penyusunan prioritas |
| REQ-3.8.3.2 | Save Prioritas | Simpan urutan prioritas |
| REQ-3.8.3.3 | Cetak Excel | Export ke Excel |
| REQ-3.8.3.4 | Cetak Dokumen | Cetak dokumen prioritas |

### 3.9 Pengadaan Extended

**Prioritas:** 🟡 Sedang

#### 3.9.1 Rencana Pengadaan Langsung

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.9.1.1 | Create Rencana | Buat rencana pengadaan langsung |
| REQ-3.9.1.2 | List Rencana | Daftar rencana pengadaan |
| REQ-3.9.1.3 | Detail Rencana | Detail rencana |
| REQ-3.9.1.4 | Update Rencana | Edit rencana |
| REQ-3.9.1.5 | Approve Rencana | Approve rencana pengadaan |

#### 3.9.2 Pengadaan Langsung Non-Kontrak

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.9.2.1 | Create Pengadaan | Buat pengadaan tanpa kontrak |
| REQ-3.9.2.2 | Quick Entry | Entry cepat untuk pembelian kecil |
| REQ-3.9.2.3 | Attach Invoice | Lampirkan invoice/kwitansi |
| REQ-3.9.2.4 | Approval Simple | Approval sederhana 1 level |

### 3.10 Reporting & Export

**Prioritas:** 🟡 Sedang

#### 3.10.1 Report Templates

| ID | Report | Deskripsi |
|----|--------|-----------|
| REQ-3.10.1.1 | Daftar Aset | Laporan daftar aset per kategori/satker |
| REQ-3.10.1.2 | Mutasi Aset | Laporan mutasi aset periode tertentu |
| REQ-3.10.1.3 | Penghapusan | Laporan penghapusan BMN |
| REQ-3.10.1.4 | Hibah | Laporan hibah BMN |
| REQ-3.10.1.5 | Pemeliharaan | Laporan pemeliharaan aset |
| REQ-3.10.1.6 | Pemakaian | Laporan peminjaman/pemakaian |
| REQ-3.10.1.7 | Analisis Kebutuhan | Laporan kebutuhan BMN |
| REQ-3.10.1.8 | Executive Summary | Dashboard eksekutif |

#### 3.10.2 Export Formats

| ID | Format | Deskripsi |
|----|--------|-----------|
| REQ-3.10.2.1 | PDF | Export ke PDF dengan template resmi |
| REQ-3.10.2.2 | Excel | Export ke XLSX dengan styling |
| REQ-3.10.2.3 | CSV | Export raw data ke CSV |
| REQ-3.10.2.4 | Print | Print-friendly HTML view |

#### 3.10.3 Report Scheduling

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| REQ-3.10.3.1 | Schedule Report | Jadwalkan report otomatis |
| REQ-3.10.3.2 | Email Delivery | Kirim report via email |
| REQ-3.10.3.3 | Report History | Riwayat report yang di-generate |

---

## 4. Non-Functional Requirements

### 4.1 Performance

| ID | Requirement | Target | Measurement |
|----|-------------|--------|-------------|
| NFR-4.1.1 | Page Load Time | < 2 detik | First Contentful Paint |
| NFR-4.1.2 | API Response Time | < 500ms (p95) | Latency percentile |
| NFR-4.1.3 | Concurrent Users | 1000 users | Load test |
| NFR-4.1.4 | Database Query | < 100ms | Query execution time |
| NFR-4.1.5 | WASM Bundle Size | < 2MB gzipped | Build output |
| NFR-4.1.6 | Memory Usage | < 256MB per instance | Runtime monitoring |

### 4.2 Security

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| NFR-4.2.1 | Authentication | JWT via Authenc (gRPC mTLS) |
| NFR-4.2.2 | Authorization | Role-based access control (RBAC) |
| NFR-4.2.3 | Data Encryption | TLS 1.3 in transit, AES-256 at rest |
| NFR-4.2.4 | Input Validation | Server-side validation untuk semua input |
| NFR-4.2.5 | SQL Injection Prevention | Parameterized queries only |
| NFR-4.2.6 | XSS Prevention | Content Security Policy, output encoding |
| NFR-4.2.7 | CSRF Protection | Double-submit cookie pattern |
| NFR-4.2.8 | Audit Logging | Immutable audit trail untuk semua operasi |
| NFR-4.2.9 | Secrets Management | Via Secreton (gRPC mTLS), no env vars |
| NFR-4.2.10 | Session Management | Secure, HttpOnly cookies |

### 4.3 Scalability

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| NFR-4.3.1 | Horizontal Scaling | Stateless services, scale via Kubernetes |
| NFR-4.3.2 | Database Scaling | Read replicas untuk query-heavy operations |
| NFR-4.3.3 | Caching | Redis untuk session dan frequently accessed data |
| NFR-4.3.4 | CDN | Static assets via CDN |

### 4.4 Availability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-4.4.1 | Uptime SLA | 99.9% |
| NFR-4.4.2 | RTO | < 1 jam |
| NFR-4.4.3 | RPO | < 15 menit |
| NFR-4.4.4 | Graceful Degradation | Fallback untuk fitur non-critical |

### 4.5 Usability

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| NFR-4.5.1 | Accessibility | WCAG 2.1 AA compliant |
| NFR-4.5.2 | Responsive Design | Desktop, tablet, mobile support |
| NFR-4.5.3 | Browser Support | Chrome 90+, Firefox 90+, Edge 90+ |
| NFR-4.5.4 | Localization | Bahasa Indonesia |
| NFR-4.5.5 | Error Messages | Clear, actionable error messages |
| NFR-4.5.6 | Loading States | Skeleton loading, progress indicators |

### 4.6 Maintainability

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| NFR-4.6.1 | Code Coverage | ≥ 80% unit test coverage |
| NFR-4.6.2 | Documentation | Inline docs, API documentation |
| NFR-4.6.3 | Logging | Structured JSON logging |
| NFR-4.6.4 | Monitoring | Prometheus metrics, Grafana dashboards |
| NFR-4.6.5 | Alerting | PagerDuty integration for critical alerts |

---

## 5. Data Requirements

### 5.1 Data Entities

| ID | Entity | Deskripsi | Source |
|----|--------|-----------|--------|
| DTA-5.1.1 | Asset | Data aset BMN | SIMAN (Read-only) |
| DTA-5.1.2 | Pengadaan | Data pengadaan | Local |
| DTA-5.1.3 | Analisis | Data analisis kebutuhan | Local |
| DTA-5.1.4 | Pemakaian | Data peminjaman/pemakaian | Local |
| DTA-5.1.5 | Hibah | Data hibah BMN | Local |
| DTA-5.1.6 | Mutasi | Data mutasi aset | Local |
| DTA-5.1.7 | Penghapusan | Data penghapusan BMN | Local |
| DTA-5.1.8 | Pengalihan | Data pengalihan | Local |
| DTA-5.1.9 | Pemeliharaan | Data pemeliharaan | Local |
| DTA-5.1.10 | BMN Izin | Data izin penggunaan BMN | Local |
| DTA-5.1.11 | Pakaian Dinas | Data pakaian dinas | Local |
| DTA-5.1.12 | Workflow | Data approval workflow | Local |
| DTA-5.1.13 | File Attachment | Data file lampiran | Local + Object Storage |
| DTA-5.1.14 | Audit Log | Immutable audit trail | Local |

### 5.2 Data Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                        SIMAN (External)                          │
│  ┌─────────┐                                                     │
│  │  Asset  │ ◄──────────── Read-Only Integration                 │
│  └────┬────┘                                                     │
└───────┼─────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Local Database                               │
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐   │
│  │Pengadaan │    │Pemakaian │    │ Hibah    │    │ Mutasi   │   │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘   │
│       │               │               │               │          │
│       ▼               ▼               ▼               ▼          │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │                    Workflow Engine                         │   │
│  │  (Approval States, Activities, Assignments)                │   │
│  └───────────────────────────────────────────────────────────┘   │
│       │                                                          │
│       ▼                                                          │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │                  File Attachments                          │   │
│  └───────────────────────────────────────────────────────────┘   │
│       │                                                          │
│       ▼                                                          │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │                    Audit Log                               │   │
│  └───────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3 Data Validation Rules

| Entity | Field | Validation |
|--------|-------|------------|
| Pengadaan | judul | Required, 1-255 chars |
| Pengadaan | anggaran | Positive number, max 10 trillion |
| Pengadaan | jenis | Enum: BARANG, JASA, KONSULTANSI |
| Penghapusan | kategori | Enum: PEMINDAHTANGANAN, PEMUSNAHAN, SEBAB_LAIN |
| Penghapusan | nilai | Must match asset nilai_perolehan |
| Hibah | penerima | Required, valid organization name |
| File | size | Max 10MB |
| File | type | PDF, JPG, PNG, DOCX, XLSX only |

---

## 6. Integration Requirements

### 6.1 Internal Integrations

| ID | System | Protocol | Purpose |
|----|--------|----------|---------|
| INT-6.1.1 | Authenc | gRPC (mTLS) | Authentication, JWT validation |
| INT-6.1.2 | Secreton | gRPC (mTLS) | Secrets management (DB credentials, API keys) |
| INT-6.1.3 | Portal | REST API | SSO, session management |
| INT-6.1.4 | layanan-integrasi | REST/gRPC | SIMAN data sync |
| INT-6.1.5 | lib-ui | Rust crate | Shared UI components |
| INT-6.1.6 | lib-common | Rust crate | Shared utilities |

### 6.2 External Integrations

| ID | System | Protocol | Purpose |
|----|--------|----------|---------|
| INT-6.2.1 | SIMAN | REST API | Asset data sync (read-only) |
| INT-6.2.2 | Object Storage | S3-compatible | File storage (MinIO/S3) |
| INT-6.2.3 | Email Service | SMTP | Notifications, report delivery |

### 6.3 Integration Patterns

```
┌────────────────────────────────────────────────────────────────┐
│                    Browser (WASM)                               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │            perlengkapan-microfrontend                     │  │
│  │  (uses lib-ui, calls REST API)                            │  │
│  └────────────────────────┬─────────────────────────────────┘  │
└───────────────────────────┼────────────────────────────────────┘
                            │ REST API (JSON/HTTP)
                            ▼
┌────────────────────────────────────────────────────────────────┐
│                    layanan-perlengkapan                         │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  Axum Handlers + Services                               │    │
│  └──┬─────────────────────────────┬───────────────────────┘    │
│     │                             │                             │
│     │ gRPC (mTLS)                 │ gRPC (mTLS)                │
│     ▼                             ▼                             │
│  ┌──────────┐              ┌──────────┐                        │
│  │ Authenc  │              │ Secreton │                        │
│  └──────────┘              └──────────┘                        │
└────────────────────────────────────────────────────────────────┘
```

---

## 7. Testing Requirements

### 7.1 Unit Testing

| ID | Requirement | Target |
|----|-------------|--------|
| TST-7.1.1 | Backend Handler Tests | 100% handlers covered |
| TST-7.1.2 | Backend Service Tests | 100% business logic covered |
| TST-7.1.3 | Backend Model Tests | 100% models with validation tests |
| TST-7.1.4 | Frontend Component Tests | 100% components covered |
| TST-7.1.5 | Frontend API Tests | Mock API responses |
| TST-7.1.6 | Code Coverage | ≥ 80% overall |

**Unit Test File Naming:**
- Backend: `src/tests/{module}_tests.rs`
- Frontend: `src/tests.rs` (module-level tests)

### 7.2 Integration Testing

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| TST-7.2.1 | Database Integration | Test dengan PostgreSQL test container |
| TST-7.2.2 | API Integration | Full request-response cycle tests |
| TST-7.2.3 | Authenc Integration | Mock gRPC client tests |
| TST-7.2.4 | Secreton Integration | Mock gRPC client tests |
| TST-7.2.5 | File Upload Integration | Test file upload/download flow |

### 7.3 End-to-End Testing (Playwright)

| ID | Test Suite | Deskripsi |
|----|------------|-----------|
| TST-7.3.1 | Authentication Flow | Login, logout, session management |
| TST-7.3.2 | Pengadaan CRUD | Create, read, update, delete pengadaan |
| TST-7.3.3 | Workflow Approval | Multi-level approval flow |
| TST-7.3.4 | File Upload | Upload, preview, download files |
| TST-7.3.5 | Penghapusan Flow | Full penghapusan workflow |
| TST-7.3.6 | Hibah Flow | Full hibah workflow |
| TST-7.3.7 | Report Export | Generate and download reports |
| TST-7.3.8 | Responsive Design | Mobile, tablet, desktop views |
| TST-7.3.9 | Error Handling | Error states and recovery |
| TST-7.3.10 | Performance | Page load times, interactions |

**E2E Test Structure:**
```
e2e/tests/perlengkapan/
├── auth.spec.ts
├── pengadaan.spec.ts
├── penghapusan.spec.ts
├── hibah.spec.ts
├── mutasi.spec.ts
├── pemakaian.spec.ts
├── pemeliharaan.spec.ts
├── workflow.spec.ts
├── file-upload.spec.ts
├── reporting.spec.ts
└── fixtures/
    ├── test-data.json
    └── test-files/
```

### 7.4 Test Data Management

| ID | Requirement | Deskripsi |
|----|-------------|-----------|
| TST-7.4.1 | Test Database | Isolated test database per test run |
| TST-7.4.2 | Seed Data | Fixtures untuk initial test data |
| TST-7.4.3 | Data Cleanup | Automatic cleanup after test |
| TST-7.4.4 | Mock Services | Mock Authenc/Secreton untuk testing |

### 7.5 Performance Testing

| ID | Requirement | Target |
|----|-------------|--------|
| TST-7.5.1 | Load Test | 1000 concurrent users |
| TST-7.5.2 | Stress Test | 2000 concurrent users (breaking point) |
| TST-7.5.3 | API Latency | p95 < 500ms |
| TST-7.5.4 | Page Load | < 2 seconds |

---

## 8. Constraints & Assumptions

### 8.1 Technical Constraints

| ID | Constraint | Impact |
|----|------------|--------|
| CON-8.1.1 | Rust Edition 2024 | Must use MSRV 1.90+ |
| CON-8.1.2 | Leptos 0.8.x | CSR-only, no SSR |
| CON-8.1.3 | PostgreSQL 15+ | Required for JSONB features |
| CON-8.1.4 | WASM Browser Support | Chrome/Firefox/Edge 90+ only |
| CON-8.1.5 | gRPC for Infra | Must use gRPC to Authenc/Secreton |
| CON-8.1.6 | REST for Frontend | Microfrontend cannot call gRPC directly |

### 8.2 Business Constraints

| ID | Constraint | Impact |
|----|------------|--------|
| CON-8.2.1 | Government Compliance | Must follow BSSN security standards |
| CON-8.2.2 | Data Sovereignty | All data must remain in Indonesia |
| CON-8.2.3 | Audit Requirements | Complete audit trail required |
| CON-8.2.4 | Approval Authority | Must respect organizational hierarchy |

### 8.3 Assumptions

| ID | Assumption | Risk if Invalid |
|----|------------|-----------------|
| ASM-8.3.1 | SIMAN API available | Asset data unavailable |
| ASM-8.3.2 | Authenc/Secreton deployed | Auth/secrets unavailable |
| ASM-8.3.3 | Network connectivity stable | Service disruption |
| ASM-8.3.4 | Users have modern browsers | Compatibility issues |
| ASM-8.3.5 | Object storage available | File upload fails |

---

## 9. Success Criteria

### 9.1 Functional Completeness

| ID | Criteria | Target | Measurement |
|----|----------|--------|-------------|
| SUC-9.1.1 | All CRUD operations | 100% | Feature checklist |
| SUC-9.1.2 | Workflow approval | 100% | Flow testing |
| SUC-9.1.3 | File attachments | 100% | Upload/download tests |
| SUC-9.1.4 | Report generation | 100% | Export tests |
| SUC-9.1.5 | Laravel parity | ≥ 95% | Feature comparison |

### 9.2 Quality Metrics

| ID | Criteria | Target | Measurement |
|----|----------|--------|-------------|
| SUC-9.2.1 | Unit test coverage | ≥ 80% | Code coverage report |
| SUC-9.2.2 | E2E test pass rate | 100% | Playwright report |
| SUC-9.2.3 | Zero critical bugs | 0 | Bug tracker |
| SUC-9.2.4 | Performance targets | 100% met | Load test report |
| SUC-9.2.5 | Security scan clean | 0 high/critical | Security audit |

### 9.3 User Acceptance

| ID | Criteria | Target | Measurement |
|----|----------|--------|-------------|
| SUC-9.3.1 | User training complete | 100% key users | Training records |
| SUC-9.3.2 | UAT sign-off | All stakeholders | Sign-off document |
| SUC-9.3.3 | User satisfaction | ≥ 4/5 | Survey |

### 9.4 Deployment Readiness

| ID | Criteria | Target | Measurement |
|----|----------|--------|-------------|
| SUC-9.4.1 | CI/CD pipeline | Working | Pipeline status |
| SUC-9.4.2 | Documentation | Complete | Doc review |
| SUC-9.4.3 | Runbook | Complete | Operations review |
| SUC-9.4.4 | Monitoring setup | Complete | Dashboard review |
| SUC-9.4.5 | Rollback plan | Tested | Rollback drill |

---

## Appendix A: Reference IDs Summary

### Functional Requirements (REQ)

| Prefix | Section | Count |
|--------|---------|-------|
| REQ-3.1.x | CRUD Complete | 32 |
| REQ-3.2.x | Workflow Approval | 10 |
| REQ-3.3.x | File Attachments | 10 |
| REQ-3.4.x | BMN Izin | 17 |
| REQ-3.5.x | Penghapusan Extended | 18 |
| REQ-3.6.x | Pakaian Dinas | 14 |
| REQ-3.7.x | Aset Kategori | 27 |
| REQ-3.8.x | Analisis Extended | 11 |
| REQ-3.9.x | Pengadaan Extended | 9 |
| REQ-3.10.x | Reporting | 15 |
| **Total** | | **163** |

### Non-Functional Requirements (NFR)

| Prefix | Section | Count |
|--------|---------|-------|
| NFR-4.1.x | Performance | 6 |
| NFR-4.2.x | Security | 10 |
| NFR-4.3.x | Scalability | 4 |
| NFR-4.4.x | Availability | 4 |
| NFR-4.5.x | Usability | 6 |
| NFR-4.6.x | Maintainability | 5 |
| **Total** | | **35** |

---

**Document End**

*Generated: 2026-02-02*  
*Next Step: Database Design Document*

