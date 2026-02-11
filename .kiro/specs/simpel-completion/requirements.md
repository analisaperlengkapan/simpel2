# Requirements Document: SIMPEL - Sistem Informasi Manajemen Perlengkapan

## 1. Introduction

### 1.1 Purpose

This document defines the complete requirements for SIMPEL (Sistem Informasi Manajemen Perlengkapan), a comprehensive Barang Milik Negara (BMN) management system for the Indonesian Attorney General's Office (Kejaksaan RI). SIMPEL manages non-SBSK BMN requirements, complementing the SIMAN v2 RKBMN module which handles SBSK-compliant BMN.

### 1.2 Scope

SIMPEL consists of ten microservices handling:
1. Master data and reference management
2. BMN requirements analysis (including pakaian dinas and roadmap)
3. BMN usage permits (vehicles, housing, laptops)
4. SK Penghapusan BMN workflow and document generation
5. Workflow engine for approvals
6. Document generation (SK Penghapusan, permits, reports)
7. Dashboard and analytics
8. Integration gateway (SIMAN, MySIMKARI)
9. Multi-channel notifications
10. Authentication and authorization

### 1.3 Current Status

**Implemented (70-75% complete):**
- Infrastructure: Authenc (50K LOC), Secreton (40K LOC), K8s
- Backend: `layanan/portal`, `layanan/perlengkapan/crates/api`
- Frontend: `antarmuka/portal`, `antarmuka/perlengkapan`
- Libraries: `lib-common` (5K LOC), `lib-ui` (15K LOC), `lib-perlengkapan` (1K LOC)
- Integration schema defined

**Needs Implementation:**
- SIMAN/MySIMKARI API integration
- Workflow engine
- Document/notification services
- Pemakaian BMN module
- Advanced features (search, export, batch ops)
- SK Penghapusan BMN generation

### 1.4 References

- Peraturan Pemerintah Nomor 27 Tahun 2014 tentang Pengelolaan BMN
- Peraturan Menteri Keuangan tentang SBSK
- Peraturan internal Kejaksaan RI terkait BMN
- Full specification: `docs/SIMPEL_Requirements.md`

## 2. System Architecture

### 2.1 Microservices Overview

| Service | Code | Status | Description |
|---------|------|--------|-------------|
| Master Data | master | 60% | Kodefikasi, standar spesifikasi, standar jumlah, mapping |
| Kebutuhan BMN | kebutuhan | 70% | BMN requirements, pakaian dinas, roadmap sarpras |
| Pemakaian BMN | pemakaian | 0% | Usage permits (vehicles, housing, laptops) |
| Penghapusan BMN | penghapusan | 0% | SK Penghapusan BMN workflow and document generation |
| Workflow | workflow | 0% | Centralized workflow engine |
| Dokumen | dokumen | 0% | Document generation (SK, permits, reports) |
| Dashboard | dashboard | 50% | Analytics and visualization |
| Integrasi | integrasi | 30% | SIMAN/MySIMKARI adapter and cache |
| Notifikasi | notifikasi | 0% | Multi-channel notifications |
| Authenc | authenc | 100% | Authentication and authorization (existing) |

### 2.2 Technology Stack

- **Language:** Rust (Edition 2024, MSRV 1.90+)
- **Backend:** Axum 0.8.x (REST), Tonic 0.14.x (gRPC)
- **Frontend:** Leptos 0.8.x (WASM CSR)
- **Database:** PostgreSQL (schema per service)
- **Cache:** Redis
- **Message Broker:** NATS / RabbitMQ
- **Object Storage:** MinIO / S3-compatible
- **Identity:** Authenc (existing)
- **Secrets:** Secreton (existing)

### 2.3 Communication Patterns

- **Frontend → Backend:** REST API (JSON/HTTP)
- **Backend ↔ Backend:** gRPC (Protobuf)
- **Backend → Authenc/Secreton:** gRPC (mTLS)
- **Async Events:** Message broker (NATS/RabbitMQ)

## 3. Actors

| Code | Actor | Description | Access Level |
|------|-------|-------------|--------------|
| A01 | Super Admin | System administrator | Full access |
| A02 | Admin Pusat | BMN manager at Kejaksaan Agung | Master data, verification, national reports |
| A03 | Admin Wilayah | BMN manager at Kejaksaan Tinggi | Wilayah verification, regional reports |
| A04 | Operator Satker | BMN manager at Kejari/Cabjari | Data input, submissions, satker reports |
| A05 | Verifikator | Approval officer | Verification and approval |
| A06 | Pimpinan Satker | Head of Kejari/Cabjari | Satker-level approval, dashboard |
| A07 | Pimpinan Wilayah | Head of Kejati | Wilayah-level approval, monitoring |
| A08 | Pimpinan Pusat | Eselon I/II at Kejagung | National monitoring, strategic decisions |
| A09 | PPBMN | BMN administration officer | BMN register maintenance |
| A10 | Auditor | Internal/external auditor | Read-only access, audit trail |
| A11 | Pegawai | Kejaksaan employee | BMN usage requests, status tracking |

## 4. Core Entities

| Code | Entity | Description | Owner Service |
|------|--------|-------------|---------------|
| E01 | Satuan Kerja | Organizational unit | authenc |
| E02 | Pegawai | Employee data (from MySIMKARI) | integrasi |
| E03 | Barang Milik Negara | BMN assets (from MonSAKTI) | integrasi |
| E04 | Kode Barang | BMN classification codes | master |
| E05 | Standar Spesifikasi | Technical specifications | master |
| E06 | Standar Jumlah | Quantity standards | master |
| E07 | Kebutuhan BMN | BMN requirements | kebutuhan |
| E08 | Kebutuhan Pakaian Dinas | Uniform requirements | kebutuhan |
| E09 | Roadmap Sarpras | 5-year infrastructure roadmap | kebutuhan |
| E10 | Izin Pemakaian | Usage permits | pemakaian |
| E11 | Workflow Instance | Workflow process instance | workflow |
| E12 | Dokumen | Generated documents | dokumen |
| E13 | Template Dokumen | Document templates | dokumen |
| E14 | Mapping Kodefikasi | Code mapping | master |
| E15 | Riwayat Pemenuhan | Fulfillment history | kebutuhan |

## 5. Requirements by Service

### 5.1 Master Data Service (master)

**Status:** 60% complete - needs refactoring and mapping kodefikasi

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-M001 | High | System SHALL provide CRUD for kode barang with BMN format validation | Admin Pusat |
| REQ-M002 | High | System SHALL support autocomplete search for kode barang by code, name, group | All |
| REQ-M003 | High | System SHALL provide CRUD for standar spesifikasi with dynamic JSONB fields | Admin Pusat |
| REQ-M004 | High | System SHALL support versioning of standar spesifikasi per fiscal year | Admin Pusat |
| REQ-M005 | High | System SHALL provide CRUD for standar jumlah with multiple calculation types | Admin Pusat |
| REQ-M006 | High | System SHALL simulate kebutuhan calculation based on standar jumlah | Admin Pusat |
| REQ-M007 | High | System SHALL auto-detect non-standard kode barang from MonSAKTI | System |
| REQ-M008 | Medium | System SHALL provide UI for proposing and verifying kode barang mapping | Operator, Admin |
| REQ-M009 | Medium | System SHALL provide dashboard for mapping kodefikasi progress | Admin |
| REQ-M010 | Medium | System SHALL support bulk import from Excel/CSV with validation | Admin Pusat |
| REQ-M011 | Medium | System SHALL support export to Excel | Admin |
| REQ-M012 | High | System SHALL publish events when reference data changes | System |
| REQ-M013 | Low | System SHALL provide API for version comparison (diff view) | Admin Pusat |
| REQ-M014 | High | System SHALL audit log all master data changes | System |
| REQ-M015 | Medium | System SHALL filter kode barang by is_sbsk attribute | All |

### 5.2 Kebutuhan BMN Service (kebutuhan)

**Status:** 70% complete - needs workflow integration, pakaian dinas, roadmap

#### Functional Requirements

##### 5.2.1 Kebutuhan BMN General Workflow

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-K001 | High | System SHALL allow Validator Pusat to initiate kebutuhan BMN period with start date, end date, and deadline | Validator Pusat |
| REQ-K002 | High | System SHALL allow Validator Pusat to select eligible BMN items filtered by standar kodefikasi | Validator Pusat |
| REQ-K003 | High | System SHALL allow Validator Pusat to select eligible satkers that can submit kebutuhan BMN | Validator Pusat |
| REQ-K004 | High | System SHALL allow Operator Satker to submit kebutuhan BMN within period constraints set by Validator Pusat | Operator Satker |
| REQ-K005 | High | System SHALL require Operator Satker to provide justification text for each kebutuhan BMN item | Operator Satker |
| REQ-K006 | High | System SHALL allow Operator Satker to upload supporting documents (surat permohonan and other attachments) | Operator Satker |
| REQ-K007 | High | System SHALL allow Operator Satker to submit kebutuhan BMN to Validator Wilayah | Operator Satker |
| REQ-K008 | High | System SHALL allow Validator Wilayah to review and forward kebutuhan BMN to Validator Pusat | Validator Wilayah |
| REQ-K009 | High | System SHALL allow Validator Wilayah to return kebutuhan BMN to Operator Satker with revision notes | Validator Wilayah |
| REQ-K010 | High | System SHALL display existing BMN data from SIMAN integration to Validator Pusat during analysis | Validator Pusat |
| REQ-K011 | High | System SHALL display pegawai summary from MySIMKARI (eselon count, non-eselon by golongan/pangkat, jaksa/non-jaksa breakdown) to Validator Pusat | Validator Pusat |
| REQ-K012 | High | System SHALL allow Validator Pusat to approve or reject kebutuhan BMN (not return to Validator Wilayah) | Validator Pusat |
| REQ-K013 | High | System SHALL mark approved kebutuhan BMN as priority for further processing | System |
| REQ-K014 | High | System SHALL generate analysis report in PDF, DOCX, and XLSX formats | Validator Pusat |
| REQ-K015 | High | System SHALL track workflow state transitions (DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → APPROVED/REJECTED) | System |
| REQ-K016 | High | System SHALL send notifications at each workflow transition | System |
| REQ-K017 | High | System SHALL prevent submission outside of period deadline | System |
| REQ-K018 | High | System SHALL validate BMN items against eligible list set by Validator Pusat | System |
| REQ-K019 | High | System SHALL validate satker eligibility against list set by Validator Pusat | System |
| REQ-K020 | Medium | System SHALL provide dashboard showing kebutuhan BMN status by satker and wilayah | Validator Pusat, Validator Wilayah |

##### 5.2.2 Pakaian Dinas Workflow

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-K021 | High | System SHALL allow Validator Pusat to create pakaian dinas period with start date, end date, and deadline | Validator Pusat |
| REQ-K022 | High | System SHALL support satker selection via hierarchical tree based on MonSAKTI wilayah codes (0100=Kejagung, 0200=Jabar, 3400=Sulbar, etc.) with multi-select | Validator Pusat |
| REQ-K023 | High | System SHALL allow Validator Pusat to select pakaian dinas types and specifications for the period | Validator Pusat |
| REQ-K024 | High | System SHALL allow Operator Satker to input pegawai ukuran per pakaian type with gender-specific options | Operator Satker |
| REQ-K025 | High | System SHALL support hijab option (with_hijab flag) for female employees | Operator Satker |
| REQ-K026 | High | System SHALL group ukuran by category (BAJU, CELANA, SEPATU) per pakaian specification | System |
| REQ-K027 | High | System SHALL implement 3-level hierarchical approval workflow (Kejari→Kejati→Kejagung) with state tracking | Validator Wilayah, Validator Pusat |
| REQ-K028 | High | System SHALL support revision workflow returning to appropriate level (Kejati revision→Kejari, Kejagung revision→Kejati/Kejari) | Validator Wilayah, Validator Pusat |
| REQ-K029 | High | System SHALL snapshot pegawai data at submission time with eselon, pangkat, jabatan, golongan | System |
| REQ-K030 | High | System SHALL handle pusat satker (ms_satker_id='00') with unit_kerja mapping (ms_satker_pusat_id) | System |
| REQ-K031 | High | System SHALL update pegawai_pakaian_dinas master data on workflow completion (COMPLETED state) | System |
| REQ-K032 | High | System SHALL generate Laporan Daftar (individual pegawai list) with columns: No, NIP, Nama, Pangkat, Jabatan, Eselon, Jenis Pegawai, Ukuran per Pakaian | Validator Pusat, Operator Satker |
| REQ-K033 | High | System SHALL generate Laporan Rekap (aggregated summary by ukuran) with L/P breakdown per satker | Validator Pusat |
| REQ-K034 | High | System SHALL support filters for reports (jenis pegawai: TU/Jaksa, eselon, jenis_kelamin: L/P) | Validator Pusat, Operator Satker |
| REQ-K035 | High | System SHALL generate reports in PDF and Excel formats matching simpel_web-main format | Validator Pusat |
| REQ-K036 | Medium | System SHALL cache satker tree (1 hour TTL) and pegawai list (30 minutes TTL) | System |
| REQ-K037 | Medium | System SHALL use CTE-based SQL for report generation performance | System |
| REQ-K038 | Medium | System SHALL lock data after period closure | Validator Pusat |

##### 5.2.3 Roadmap Sarpras

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-K039 | Medium | System SHALL provide 5-year roadmap sarpras feature | Validator Pusat, Pimpinan |
| REQ-K040 | Medium | System SHALL sync fulfillment realization with SIMAN | System |
| REQ-K041 | Medium | System SHALL provide gap analysis visualization | Validator Pusat, Pimpinan |
| REQ-K042 | Medium | System SHALL provide trend analysis year-over-year | Validator Pusat, Pimpinan |
| REQ-K043 | Low | System SHALL provide benchmarking between similar satkers | Validator Pusat |
| REQ-K044 | High | System SHALL differentiate new procurement vs replacement | Operator Satker |

### 5.3 Pemakaian BMN Service (pemakaian)

**Status:** 0% complete - new module

#### Functional Requirements

##### 5.3.1 Pemakaian BMN Workflow

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-P001 | High | System SHALL allow Operator Satker to create izin pemakaian BMN with pegawai selection from MySIMKARI integration | Operator Satker |
| REQ-P002 | High | System SHALL allow Operator Satker to select multiple BMN items (nama barang and NUP) from SIMAN integration for single pegawai | Operator Satker |
| REQ-P003 | High | System SHALL validate BMN availability (not currently used by another pegawai with active permit) | System |
| REQ-P004 | High | System SHALL validate one BMN = one active permit (prevent duplicate active permits) | System |
| REQ-P005 | High | System SHALL allow Operator Satker to specify usage period (start date and end date) | Operator Satker |
| REQ-P006 | High | System SHALL generate DOCX draft permit document with pegawai identity and photo on page 1 | System |
| REQ-P007 | High | System SHALL generate DOCX draft permit with BMN details table (nama barang, NUP, etc.) on page 2+ | System |
| REQ-P008 | High | System SHALL allow Operator Satker to upload signed PDF permit after Pimpinan signature | Operator Satker |
| REQ-P009 | High | System SHALL mark workflow as COMPLETED after PDF upload | System |
| REQ-P010 | High | System SHALL auto-generate permit numbers with format IZN/{YEAR}/{SATKER}/{SEQUENCE} | System |
| REQ-P011 | High | System SHALL send reminders at H-30, H-14, H-7 before permit expiry | System |
| REQ-P012 | High | System SHALL auto-expire permits after end date | System |
| REQ-P013 | High | System SHALL support permit renewal with history tracking and link to previous permit | Operator Satker |
| REQ-P014 | High | System SHALL support permit revocation by Admin with reason and approval | Admin |
| REQ-P015 | High | System SHALL allow Validator Wilayah and Validator Pusat to view all active permits | Validator Wilayah, Validator Pusat |
| REQ-P016 | High | System SHALL provide monitoring dashboard showing active permits by BMN, pegawai, and satker | Validator Wilayah, Validator Pusat |
| REQ-P017 | High | System SHALL display permit usage period and expiry status | Validator Wilayah, Validator Pusat |
| REQ-P018 | High | System SHALL audit log all permit status changes (created, uploaded, expired, revoked, renewed) | System |
| REQ-P019 | Medium | System SHALL provide usage history per BMN showing all permits (past and present) | Admin, Validator Pusat |
| REQ-P020 | Medium | System SHALL provide usage history per pegawai showing all permits | Admin, Validator Pusat |
| REQ-P021 | Medium | System SHALL provide BMN utilization report (percentage of BMN with active permits) | Admin, Validator Pusat |
| REQ-P022 | Low | System SHALL notify on pegawai mutation/retirement for permit review | System |

##### 5.3.2 Pemakaian BMN Document Format

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-P023 | High | System SHALL generate permit page 1 with pegawai identity (NIP, nama, jabatan, pangkat, golongan) | System |
| REQ-P024 | High | System SHALL include pegawai photo from MySIMKARI on permit page 1 | System |
| REQ-P025 | High | System SHALL generate permit page 2+ with BMN details table (columns: No, Nama Barang, NUP, Kondisi, Tahun Perolehan) | System |
| REQ-P026 | High | System SHALL include usage period (start date and end date) in permit document | System |
| REQ-P027 | High | System SHALL include permit number and generation date in document header | System |
| REQ-P028 | High | System SHALL include signature placeholder for Pimpinan Satker | System |

### 5.4 Penghapusan BMN Service (penghapusan)

**Status:** 0% complete - new module (frontend exists, backend workflow missing)

#### Functional Requirements

##### 5.4.1 SK Penghapusan BMN Workflow

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-PH001 | High | System SHALL allow Operator Satker to initiate SK Penghapusan BMN request | Operator Satker |
| REQ-PH002 | High | System SHALL allow Operator Satker to select BMN items for deletion from SIMAN integration | Operator Satker |
| REQ-PH003 | High | System SHALL require Operator Satker to upload supporting documents (persyaratan penghapusan) | Operator Satker |
| REQ-PH004 | High | System SHALL allow Operator Satker to submit SK Penghapusan request to Validator Wilayah | Operator Satker |
| REQ-PH005 | High | System SHALL allow Validator Wilayah to review SK Penghapusan request | Validator Wilayah |
| REQ-PH006 | High | System SHALL allow Validator Wilayah to forward request to Validator Pusat or return to Operator Satker with revision notes | Validator Wilayah |
| REQ-PH007 | High | System SHALL allow Validator Pusat to review SK Penghapusan request | Validator Pusat |
| REQ-PH008 | High | System SHALL allow Validator Pusat to generate DOCX draft SK Penghapusan BMN | Validator Pusat |
| REQ-PH009 | High | System SHALL generate SK Penghapusan with official format including BMN details table | System |
| REQ-PH010 | High | System SHALL allow Validator Pusat to upload signed PDF SK Penghapusan after Pimpinan signature | Validator Pusat |
| REQ-PH011 | High | System SHALL mark workflow as COMPLETED after PDF upload | System |
| REQ-PH012 | High | System SHALL allow Operator Satker and Validator Wilayah to view uploaded PDF SK Penghapusan | Operator Satker, Validator Wilayah |
| REQ-PH013 | High | System SHALL track workflow state transitions (DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → DOCUMENT_GENERATED → COMPLETED) | System |
| REQ-PH014 | High | System SHALL send notifications at each workflow transition | System |
| REQ-PH015 | High | System SHALL auto-generate SK number with format SK/{YEAR}/{SEQUENCE} | System |
| REQ-PH016 | High | System SHALL audit log all SK Penghapusan workflow actions | System |
| REQ-PH017 | Medium | System SHALL provide dashboard showing SK Penghapusan status by satker and wilayah | Validator Pusat, Validator Wilayah |
| REQ-PH018 | Medium | System SHALL store supporting documents in object storage with SHA-256 checksum | System |
| REQ-PH019 | Medium | System SHALL validate BMN items are not currently in active use (no active izin pemakaian) | System |
| REQ-PH020 | Low | System SHALL support multiple supporting document uploads per request | Operator Satker |

##### 5.4.2 SK Penghapusan BMN Document Format

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-PH021 | High | System SHALL generate SK Penghapusan with official letterhead | System |
| REQ-PH022 | High | System SHALL include SK number and date in document header | System |
| REQ-PH023 | High | System SHALL include satker information (nama satker, kode satker) | System |
| REQ-PH024 | High | System SHALL include BMN details table (columns: No, Nama Barang, NUP, Kondisi, Tahun Perolehan, Nilai Perolehan, Alasan Penghapusan) | System |
| REQ-PH025 | High | System SHALL include legal basis for deletion (dasar hukum) | System |
| REQ-PH026 | High | System SHALL include signature placeholder for authorized official | System |
| REQ-PH027 | Medium | System SHALL include total value of BMN to be deleted | System |

### 5.5 Workflow Service (workflow)

**Status:** 0% complete - new module

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-W001 | High | System SHALL provide generic configurable workflow engine | Admin Pusat |
| REQ-W002 | High | System SHALL configure steps with role, SLA, actions, escalation | Admin Pusat |
| REQ-W003 | High | System SHALL auto-escalate on SLA breach with notification | System |
| REQ-W004 | High | System SHALL provide consistent API for workflow operations | System |
| REQ-W005 | High | System SHALL immutably log all workflow actions | System |
| REQ-W006 | Medium | System SHALL support delegation | Approver |
| REQ-W007 | Medium | System SHALL support parallel approval | System |
| REQ-W008 | Medium | System SHALL provide workflow monitoring dashboard | Admin Pusat |
| REQ-W009 | Medium | System SHALL support workflow definition versioning | System |
| REQ-W010 | Low | System SHALL support conditional branching | System |
| REQ-W011 | High | System SHALL publish events on workflow status changes | System |
| REQ-W012 | Medium | System SHALL provide inbox/task list for approvers | Approver |
| REQ-W013 | High | System SHALL integrate with dokumen service for automatic document generation on workflow completion | System |
| REQ-W014 | High | System SHALL integrate with notifikasi service for workflow transition notifications | System |

### 5.6 Dokumen Service (dokumen)

**Status:** 0% complete - new module

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-D001 | High | System SHALL provide template-based document generation | System |
| REQ-D002 | High | System SHALL generate SK Penghapusan BMN with official format | Validator Pusat |
| REQ-D003 | High | System SHALL generate izin pemakaian BMN documents with pegawai photo and BMN table | System |
| REQ-D004 | High | System SHALL generate kebutuhan BMN analysis reports in PDF, DOCX, and XLSX formats | Validator Pusat |
| REQ-D005 | High | System SHALL generate PDF with official letterhead | System |
| REQ-D006 | High | System SHALL provide API for document generation requests from workflow service | System |
| REQ-D007 | Medium | System SHALL provide CRUD for document templates with preview | Admin Pusat |
| REQ-D008 | Medium | System SHALL store documents in object storage with SHA-256 checksum | System |
| REQ-D009 | Medium | System SHALL support document versioning | Admin Pusat |
| REQ-D010 | Medium | System SHALL provide document search | All |
| REQ-D011 | Medium | System SHALL auto-generate document numbers | System |
| REQ-D012 | Low | System SHALL generate rekapitulasi in PDF and Excel | Admin Pusat |
| REQ-D013 | Low | System SHALL support document retention policy | System |
| REQ-D014 | High | System SHALL audit log document operations | System |
| REQ-D015 | High | System SHALL integrate with kebutuhan service for analysis report generation | System |
| REQ-D016 | High | System SHALL integrate with pemakaian service for permit document generation | System |
| REQ-D017 | High | System SHALL integrate with penghapusan service for SK document generation | System |

### 5.7 Dashboard Service (dashboard)

**Status:** 50% complete - needs separation and advanced features

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-DB001 | High | System SHALL provide executive dashboard with national KPIs | Pimpinan |
| REQ-DB002 | High | System SHALL support drill-down: national → wilayah → satker | All |
| REQ-DB003 | High | System SHALL visualize roadmap vs realization | Pimpinan, Admin |
| REQ-DB004 | High | System SHALL visualize gap analysis per BMN type and satker | Admin, Pimpinan |
| REQ-DB005 | Medium | System SHALL provide heatmap of kebutuhan by region | Pimpinan |
| REQ-DB006 | Medium | System SHALL provide trend analysis charts | Admin Pusat |
| REQ-DB007 | Medium | System SHALL provide workflow monitoring dashboard | Admin Pusat |
| REQ-DB008 | Medium | System SHALL provide BMN utilization report | Admin, Pimpinan |
| REQ-DB009 | Medium | System SHALL support export to PDF and Excel | All |
| REQ-DB010 | Medium | System SHALL provide consistent filters across dashboards | All |
| REQ-DB011 | Low | System SHALL support saved filters/bookmarks | All |
| REQ-DB012 | Low | System SHALL support auto-refresh | System |
| REQ-DB013 | High | System SHALL display near real-time data (max 1 hour delay) | System |

### 5.8 Integrasi Service (integrasi)

**Status:** 30% complete - schema exists, needs implementation

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-I001 | High | System SHALL sync BMN data from SIMAN daily | System |
| REQ-I002 | High | System SHALL sync pegawai data from MySIMKARI daily | System |
| REQ-I003 | High | System SHALL provide consistent internal API for cached data | System |
| REQ-I004 | High | System SHALL implement retry with exponential backoff | System |
| REQ-I005 | High | System SHALL implement circuit breaker | System |
| REQ-I006 | High | System SHALL support incremental sync | System |
| REQ-I007 | Medium | System SHALL support on-demand sync per satker | Admin |
| REQ-I008 | Medium | System SHALL log detailed sync history | System |
| REQ-I009 | Medium | System SHALL provide sync status monitoring dashboard | Admin Pusat |
| REQ-I010 | Medium | System SHALL transform data to internal format | System |
| REQ-I011 | Medium | System SHALL support configurable satker code mapping | Admin Pusat |
| REQ-I012 | Low | System SHALL store raw data for debugging | System |
| REQ-I013 | High | System SHALL publish events on successful sync | System |
| REQ-I014 | Medium | System SHALL handle API format changes via configuration | Admin Pusat |
| REQ-I015 | High | System SHALL provide gRPC API for kebutuhan service to fetch BMN data | System |
| REQ-I016 | High | System SHALL provide gRPC API for kebutuhan service to fetch pegawai summary | System |
| REQ-I017 | High | System SHALL provide gRPC API for pemakaian service to fetch BMN details | System |
| REQ-I018 | High | System SHALL provide gRPC API for pemakaian service to fetch pegawai details with photo | System |
| REQ-I019 | High | System SHALL provide gRPC API for penghapusan service to fetch BMN details | System |

### 5.9 Notifikasi Service (notifikasi)

**Status:** 0% complete - new module

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-N001 | High | System SHALL provide in-app notification center | All |
| REQ-N002 | High | System SHALL support in-app and email channels | System |
| REQ-N003 | High | System SHALL provide API for sending notifications | System |
| REQ-N004 | High | System SHALL support action-required notifications with deep links | System |
| REQ-N005 | Medium | System SHALL support notifications to roles | System |
| REQ-N006 | Medium | System SHALL support per-user notification preferences | All |
| REQ-N007 | Medium | System SHALL provide manageable notification templates | Admin Pusat |
| REQ-N008 | Medium | System SHALL send auto-reminders based on events | System |
| REQ-N009 | Low | System SHALL support digest mode | System |
| REQ-N010 | Low | System SHALL log delivery status per channel | System |
| REQ-N011 | High | System SHALL integrate with workflow service for workflow transition notifications | System |
| REQ-N012 | High | System SHALL send notifications for kebutuhan BMN workflow transitions | System |
| REQ-N013 | High | System SHALL send notifications for pemakaian BMN expiry reminders | System |
| REQ-N014 | High | System SHALL send notifications for SK Penghapusan workflow transitions | System |

### 5.10 Authenc Service (authenc)

**Status:** 100% complete - existing service

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-A001 | High | System SHALL authenticate with username/password using Argon2id | All |
| REQ-A002 | High | System SHALL enforce password policy (12+ chars, complexity) | All |
| REQ-A003 | High | System SHALL lock account after 5 failed login attempts | System |
| REQ-A004 | High | System SHALL use JWT with 15min access token, 7day refresh token | System |
| REQ-A005 | High | System SHALL implement RBAC with permission-based access | System |
| REQ-A006 | High | System SHALL enforce hierarchical satker access scope | System |
| REQ-A007 | High | System SHALL immutably audit log all significant actions | System |
| REQ-A008 | High | System SHALL provide user CRUD with MySIMKARI NIP linking | Admin |
| REQ-A009 | Medium | System SHALL support TOTP MFA | All |
| REQ-A010 | Medium | System SHALL enforce 90-day password rotation | System |
| REQ-A011 | Medium | System SHALL provide audit trail search and filter | Admin, Auditor |
| REQ-A012 | Medium | System SHALL support audit trail export | Auditor |
| REQ-A013 | Medium | System SHALL provide session management | All |
| REQ-A014 | Medium | System SHALL rate-limit login endpoint | System |
| REQ-A015 | Low | System SHALL support SSO for future integration | System |
| REQ-A016 | High | System SHALL make audit trail immutable | System |
| REQ-A017 | Medium | System SHALL provide navigable satker hierarchy | Admin |

### 5.10 Authenc Service (authenc)

**Status:** 100% complete - existing service

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-A001 | High | System SHALL authenticate with username/password using Argon2id | All |
| REQ-A002 | High | System SHALL enforce password policy (12+ chars, complexity) | All |
| REQ-A003 | High | System SHALL lock account after 5 failed login attempts | System |
| REQ-A004 | High | System SHALL use JWT with 15min access token, 7day refresh token | System |
| REQ-A005 | High | System SHALL implement RBAC with permission-based access | System |
| REQ-A006 | High | System SHALL enforce hierarchical satker access scope | System |
| REQ-A007 | High | System SHALL immutably audit log all significant actions | System |
| REQ-A008 | High | System SHALL provide user CRUD with MySIMKARI NIP linking | Admin |
| REQ-A009 | Medium | System SHALL support TOTP MFA | All |
| REQ-A010 | Medium | System SHALL enforce 90-day password rotation | System |
| REQ-A011 | Medium | System SHALL provide audit trail search and filter | Admin, Auditor |
| REQ-A012 | Medium | System SHALL support audit trail export | Auditor |
| REQ-A013 | Medium | System SHALL provide session management | All |
| REQ-A014 | Medium | System SHALL rate-limit login endpoint | System |
| REQ-A015 | Low | System SHALL support SSO for future integration | System |
| REQ-A016 | High | System SHALL make audit trail immutable | System |
| REQ-A017 | Medium | System SHALL provide navigable satker hierarchy | Admin |

### 5.11 End-to-End Integration Requirements

**Status:** 0% complete - critical for production deployment

#### Cross-Module Integration

| ID | Priority | Requirement | Modules Involved |
|----|----------|-------------|------------------|
| REQ-E001 | High | System SHALL integrate kebutuhan BMN workflow with dokumen service for analysis report generation | kebutuhan, dokumen, workflow |
| REQ-E002 | High | System SHALL integrate kebutuhan BMN workflow with notifikasi service for state transition notifications | kebutuhan, notifikasi, workflow |
| REQ-E003 | High | System SHALL integrate kebutuhan BMN analysis with integrasi service for SIMAN BMN data | kebutuhan, integrasi |
| REQ-E004 | High | System SHALL integrate kebutuhan BMN analysis with integrasi service for MySIMKARI pegawai summary | kebutuhan, integrasi |
| REQ-E005 | High | System SHALL integrate pemakaian BMN with integrasi service for BMN availability check | pemakaian, integrasi |
| REQ-E006 | High | System SHALL integrate pemakaian BMN with integrasi service for pegawai data and photo | pemakaian, integrasi |
| REQ-E007 | High | System SHALL integrate pemakaian BMN with dokumen service for permit document generation | pemakaian, dokumen |
| REQ-E008 | High | System SHALL integrate pemakaian BMN with notifikasi service for expiry reminders | pemakaian, notifikasi |
| REQ-E009 | High | System SHALL integrate penghapusan BMN workflow with dokumen service for SK generation | penghapusan, dokumen, workflow |
| REQ-E010 | High | System SHALL integrate penghapusan BMN workflow with notifikasi service for state transition notifications | penghapusan, notifikasi, workflow |
| REQ-E011 | High | System SHALL integrate penghapusan BMN with integrasi service for BMN details | penghapusan, integrasi |
| REQ-E012 | High | System SHALL integrate penghapusan BMN with pemakaian service to validate BMN not in active use | penghapusan, pemakaian |
| REQ-E013 | High | System SHALL integrate workflow service with dokumen service for automatic document generation on approval | workflow, dokumen |
| REQ-E014 | High | System SHALL integrate workflow service with notifikasi service for workflow transition notifications | workflow, notifikasi |
| REQ-E015 | High | System SHALL ensure frontend microfrontend calls backend REST API (never direct gRPC to infrastructure) | antarmuka/perlengkapan, layanan/perlengkapan |
| REQ-E016 | High | System SHALL ensure backend services use gRPC for inter-service communication | layanan/perlengkapan crates |
| REQ-E017 | High | System SHALL ensure all services authenticate via Authenc gRPC | All services, authenc |
| REQ-E018 | High | System SHALL ensure all services fetch secrets via Secreton gRPC | All services, secreton |

#### Frontend-Backend Integration

| ID | Priority | Requirement | Components |
|----|----------|-------------|------------|
| REQ-E019 | High | System SHALL provide REST API endpoints for kebutuhan BMN workflow in layanan/perlengkapan | layanan/perlengkapan/crates/api |
| REQ-E020 | High | System SHALL provide REST API endpoints for pemakaian BMN workflow in layanan/perlengkapan | layanan/perlengkapan/crates/api |
| REQ-E021 | High | System SHALL provide REST API endpoints for penghapusan BMN workflow in layanan/perlengkapan | layanan/perlengkapan/crates/api |
| REQ-E022 | High | System SHALL implement kebutuhan BMN UI components in antarmuka/perlengkapan using Leptos 0.8.x | antarmuka/pembinaan/perlengkapan |
| REQ-E023 | High | System SHALL implement pemakaian BMN UI components in antarmuka/perlengkapan using Leptos 0.8.x | antarmuka/pembinaan/perlengkapan |
| REQ-E024 | High | System SHALL implement penghapusan BMN UI components in antarmuka/perlengkapan using Leptos 0.8.x | antarmuka/pembinaan/perlengkapan |
| REQ-E025 | High | System SHALL use lib-ui shared components for consistent UI across microfrontends | antarmuka/pembinaan/perlengkapan, lib/ui |
| REQ-E026 | High | System SHALL use lib-common for shared types and utilities across backend services | layanan/perlengkapan crates, lib/common |

#### Data Flow Integration

| ID | Priority | Requirement | Flow |
|----|----------|-------------|------|
| REQ-E027 | High | System SHALL ensure kebutuhan BMN data flows: Frontend → REST API → Kebutuhan Service → Integrasi gRPC → SIMAN/MySIMKARI | Full stack |
| REQ-E028 | High | System SHALL ensure pemakaian BMN data flows: Frontend → REST API → Pemakaian Service → Integrasi gRPC → SIMAN/MySIMKARI | Full stack |
| REQ-E029 | High | System SHALL ensure penghapusan BMN data flows: Frontend → REST API → Penghapusan Service → Workflow gRPC → Dokumen gRPC | Full stack |
| REQ-E030 | High | System SHALL ensure document generation flows: Workflow Service → Dokumen gRPC → Object Storage → Frontend download | Full stack |
| REQ-E031 | High | System SHALL ensure notification flows: Workflow Service → Notifikasi gRPC → Email/In-app → Frontend display | Full stack |

## 6. Non-Functional Requirements

### 6.1 Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-P001 | Web page response time (90th percentile) | ≤ 2 seconds |
| NFR-P002 | API response time (95th percentile) | ≤ 500ms |
| NFR-P003 | PDF generation time | ≤ 5 seconds per document |
| NFR-P004 | Dashboard load time (national data) | ≤ 5 seconds |
| NFR-P005 | API throughput | ≥ 100 requests/second |
| NFR-P006 | Dashboard data delay | ≤ 1 hour from source |
| NFR-P007 | MonSAKTI full sync time | ≤ 4 hours (off-peak) |
| NFR-P008 | Incremental sync time | ≤ 30 minutes |

### 6.2 Availability & Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-A001 | System uptime | 99.5% (excluding scheduled maintenance) |
| NFR-A002 | Maintenance window | ≤ 4 hours/month, outside business hours |
| NFR-A003 | Recovery Time Objective (RTO) | ≤ 4 hours |
| NFR-A004 | Recovery Point Objective (RPO) | ≤ 1 hour |
| NFR-A005 | Database backup | Daily full + continuous WAL archiving |

### 6.3 Security

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-S001 | Data in transit encryption | TLS 1.2+ |
| NFR-S002 | Data at rest encryption | AES-256 for sensitive data |
| NFR-S003 | Password hashing | Argon2id |
| NFR-S004 | Session management | JWT (15min access, 7day refresh) |
| NFR-S005 | OWASP Top 10 | Mitigate all vulnerabilities |
| NFR-S006 | Audit trail retention | ≥ 5 years, immutable |
| NFR-S007 | Rate limiting | Login: 5/min per IP; API: 100/min per user |
| NFR-S008 | Input validation | Server-side validation, output sanitization |

### 6.4 Scalability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-SC001 | Concurrent users | ≥ 500 users |
| NFR-SC002 | BMN records | ≥ 1 million records |
| NFR-SC003 | Pegawai records | ≥ 50,000 records |
| NFR-SC004 | Horizontal scaling | All microservices independently scalable |

### 6.5 Usability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-U001 | Responsive design | Desktop (≥1024px), Tablet (≥768px) |
| NFR-U002 | Browser support | Chrome, Firefox, Edge (2 latest versions) |
| NFR-U003 | Interface language | Bahasa Indonesia |
| NFR-U004 | Accessibility | WCAG 2.1 Level AA |
| NFR-U005 | Training time | ≤ 2 hours for basic tasks |

### 6.6 Maintainability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-M001 | API documentation | OpenAPI/Swagger for all services |
| NFR-M002 | Logging | Structured JSON logging to centralized system |
| NFR-M003 | Health checks | /health endpoint for all services |
| NFR-M004 | Monitoring | Prometheus + Grafana metrics |
| NFR-M005 | Database migrations | Versioned migration scripts |
| NFR-M006 | Code coverage | ≥ 70% unit test coverage |

## 7. Service Dependencies

### 7.1 Dependency Matrix

| Consumer ↓ / Provider → | master | kebutuhan | pemakaian | penghapusan | workflow | dokumen | dashboard | integrasi | notifikasi | authenc |
|-------------------------|--------|-----------|-----------|-------------|----------|---------|-----------|-----------|------------|---------|
| master | — | | | | | | | ✓ | ✓ | ✓ |
| kebutuhan | ✓ | — | | | ✓ | ✓ | | ✓ | ✓ | ✓ |
| pemakaian | ✓ | | — | | ✓ | ✓ | | ✓ | ✓ | ✓ |
| penghapusan | ✓ | | ✓ | — | ✓ | ✓ | | ✓ | ✓ | ✓ |
| workflow | | | | | — | ✓ | | | ✓ | ✓ |
| dokumen | | | | | | — | | | ✓ | ✓ |
| dashboard | ✓ | ✓ | ✓ | ✓ | ✓ | | — | ✓ | | ✓ |
| integrasi | | | | | | | | — | ✓ | ✓ |
| notifikasi | | | | | | | | | — | ✓ |
| authenc | | | | | | | | ✓ | | — |

### 7.2 Deployment Order

Based on dependencies:
1. **authenc** - no SIMPEL dependencies
2. **notifikasi** - depends on authenc only
3. **integrasi** - depends on authenc, notifikasi
4. **master** - depends on authenc, integrasi, notifikasi
5. **workflow** - depends on authenc, notifikasi
6. **dokumen** - depends on authenc, notifikasi, workflow
7. **kebutuhan** - depends on master, workflow, dokumen, integrasi, notifikasi, authenc
8. **pemakaian** - depends on master, workflow, dokumen, integrasi, notifikasi, authenc
9. **penghapusan** - depends on master, pemakaian, workflow, dokumen, integrasi, notifikasi, authenc
10. **dashboard** - depends on almost all services (deploy last)

## 8. Glossary

| Term | Abbreviation | Definition |
|------|--------------|------------|
| Barang Milik Negara | BMN | State-owned assets |
| Standar Barang dan Standar Kebutuhan | SBSK | Standards set by Ministry of Finance |
| Rencana Kebutuhan BMN | RKBMN | BMN requirements planning document |
| Sistem Informasi Manajemen Aset Negara | SIMAN | National asset management system (DJKN) |
| Monitoring SAKTI | MonSAKTI | Budget execution monitoring system |
| MySIMKARI | — | Kejaksaan RI personnel management system |
| Nomor Urut Pendaftaran | NUP | BMN registration number |
| Satuan Kerja | Satker | Organizational work unit |
| Pakaian Dinas Harian | PDH | Daily uniform |
| Pakaian Dinas Lapangan | PDL | Field uniform |
| Pejabat Penatausahaan BMN | PPBMN | BMN administration officer |
| Surat Keputusan Penghapusan | SK Penghapusan | Decree for BMN deletion/disposal |
| Surat Perintah Pencairan Dana | SP2D | Payment order |
| Service Level Agreement | SLA | Process completion time limit |
| Konstruksi Dalam Pengerjaan | KDP | Construction in progress |
| Kartu Inventaris Barang | KIB | Asset inventory card |
| Time-based One-Time Password | TOTP | Two-factor authentication method |
| JSON Web Token | JWT | Authentication token standard |
| Role-Based Access Control | RBAC | Role-based access control model |
| Validator Pusat | — | Central validator at Kejaksaan Agung |
| Validator Wilayah | — | Regional validator at Kejaksaan Tinggi |

---

**Document Status:** DRAFT v3.0.0
**Date:** February 11, 2026
**Classification:** Internal - Kejaksaan Republik Indonesia
