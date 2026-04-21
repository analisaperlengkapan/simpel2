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

**Overall System Completion: 65-70%**

#### Fully Implemented (100%)

| Component | LOC | Status | Notes |
|-----------|-----|--------|-------|
| Authenc Service | 50K | ✅ Complete | Authentication, RBAC, MFA, audit logging |
| Secreton Service | 40K | ✅ Complete | Secrets management, HSM, transit encryption |
| lib-common | 5K | ✅ Complete | Shared types, database config, crypto utilities |
| lib-ui | 15K | ✅ Complete | Leptos shared UI components |
| lib-perlengkapan | 1K | ✅ Complete | Domain types |
| Infrastructure | — | ✅ Complete | K8s manifests, monitoring, CI/CD |

#### Partially Implemented

| Service/Module | Completion | Status | What's Working | What's Missing |
|----------------|------------|--------|----------------|----------------|
| **Kebutuhan BMN** | 70% | 🟡 Partial | Core workflow, period management, submission flow, basic reporting | Workflow integration, pakaian dinas (60%), roadmap sarpras (30%), advanced analytics |
| **Pemakaian BMN** | 80% | 🟡 Partial | Core workflow, permit creation, BMN selection, basic validation | Document generation integration, expiry reminders, renewal workflow |
| **Pakaian Dinas** | 60% | 🟡 Partial | Basic data structure, satker tree, ukuran input | 3-level approval workflow, report generation, pegawai snapshot |
| **Dashboard** | 50% | 🟡 Partial | Basic KPI display, satker filtering | Drill-down functionality, advanced analytics, real-time updates |
| **Bantuan Service** | 100% | ✅ Complete | FAQ, helpdesk, chatbot fully functional | — |
| **Penghapusan BMN** | 40% | 🟡 Partial | Frontend UI exists, basic data structure | Backend workflow, document generation, validation logic |
| **Workflow Engine** | 70% | 🟡 Partial | Core engine, state machine, SLA tracking, delegation, monitoring, parallel approval | Admin UI, auto-escalation scheduler, monitoring dashboard UI |
| **Notifikasi** | 60% | 🟡 Partial | Basic notification schema, in-app display | Workflow integration, email delivery, reminder scheduling |
| **Dokumen** | 50% | 🟡 Partial | Basic document storage, object storage integration | Template system, DOCX/PDF generation, auto-numbering |
| **Integrasi** | 30% | 🟡 Partial | Database schema defined, gRPC stubs | SIMAN API calls, MySIMKARI API calls, sync scheduling, error handling |
| **Roadmap Sarpras** | 30% | 🟡 Partial | Basic structure, database schema | 5-year planning, gap analysis, trend visualization |

#### Not Started (0%)

| Component | Priority | Blocker Status |
|-----------|----------|----------------|
| **Master Data Service** | High | 🔴 Critical - Kodefikasi mapping CRUD missing |
| **Advanced Search/Export UI** | Medium | — |
| **Real-time WebSocket Updates** | Low | — |

#### Critical Blockers (Must Fix for Production)

1. **🔴 Integrasi Service (30%)** - SIMAN/MySIMKARI API integration incomplete
   - Blocks: Kebutuhan BMN analysis, Pemakaian BMN validation, Penghapusan BMN validation
   - Impact: All data-dependent features cannot function without external data
   - Required: Complete API client implementation, sync scheduling, error handling

2. **🔴 Notifikasi Integration (60%)** - Not wired to workflow transitions
   - Blocks: User notifications for workflow state changes
   - Impact: Users don't receive timely updates on approval status
   - Required: Event-driven integration with workflow service

3. **🔴 Penghapusan BMN Workflow (40%)** - Backend workflow incomplete
   - Blocks: SK Penghapusan BMN generation and approval flow
   - Impact: Cannot complete BMN deletion process
   - Required: Complete workflow implementation, document generation integration

#### Implementation Priority (Based on Dependencies)

**Phase 1 (Critical - Unblock Core Features):**
1. Complete Integrasi Service (SIMAN/MySIMKARI API calls)
2. Complete Workflow Engine UI and auto-escalation (core engine already 70% complete)
3. Wire Notifikasi to Workflow events

**Phase 2 (High Priority - Complete Core Modules):**
4. Complete Penghapusan BMN workflow
5. Complete Dokumen template system and generation
6. Complete Pakaian Dinas approval workflow
7. Implement Master Data kodefikasi mapping

**Phase 3 (Medium Priority - Enhanced Features):**
8. Complete Dashboard drill-down and analytics
9. Complete Roadmap Sarpras 5-year planning
10. Implement advanced search and export

**Phase 4 (Low Priority - Nice-to-Have):**
11. Real-time WebSocket updates
12. Advanced analytics and benchmarking

### 1.4 References

- Peraturan Pemerintah Nomor 27 Tahun 2014 tentang Pengelolaan BMN
- Peraturan Menteri Keuangan tentang SBSK
- Peraturan internal Kejaksaan RI terkait BMN
- Full specification: `docs/SIMPEL_Requirements.md`

## 2. System Architecture

### 2.1 Microservices Overview

| Service | Code | Status | Codebase Location | Description |
|---------|------|--------|-------------------|-------------|
| Master Data | master | 0% 🔴 | `layanan/perlengkapan/crates/master` (planned) | Kodefikasi mapping CRUD - NOT STARTED |
| Kebutuhan BMN | kebutuhan | 70% 🟡 | `layanan/perlengkapan/crates/api` | Core workflow ✅, pakaian dinas 60%, roadmap 30% |
| Pemakaian BMN | pemakaian | 80% 🟡 | `layanan/perlengkapan/crates/api` | Core workflow ✅, needs document integration |
| Penghapusan BMN | penghapusan | 40% 🟡 | `antarmuka/perlengkapan` (frontend only) | Frontend exists, backend workflow missing |
| Workflow | workflow | 70% 🟡 | `layanan/perlengkapan/crates/api/src/workflow` | Core engine ✅, needs admin UI |
| Dokumen | dokumen | 50% 🟡 | `layanan/perlengkapan/crates/dokumen` | Storage ✅, template system missing |
| Dashboard | dashboard | 50% 🟡 | `layanan/perlengkapan/crates/api` + `antarmuka/perlengkapan` | Basic KPIs ✅, drill-down missing |
| Integrasi | integrasi | 30% 🔴 | `layanan/integrasi` | Schema ✅, API calls missing - CRITICAL BLOCKER |
| Notifikasi | notifikasi | 60% 🟡 | `layanan/perlengkapan/crates/notifikasi` | Schema ✅, workflow integration missing |
| Bantuan | bantuan | 100% ✅ | `layanan/perlengkapan/crates/bantuan` | FAQ, helpdesk, chatbot fully functional |
| Authenc | authenc | 100% ✅ | `layanan/authenc` | Authentication, RBAC, MFA, audit (50K LOC) |
| Secreton | secreton | 100% ✅ | `layanan/secreton` | Secrets management, HSM, transit (40K LOC) |

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

**Status:** 0% complete - NOT STARTED (CRITICAL BLOCKER)

**Codebase Location:** `layanan/perlengkapan/crates/master` (planned, does not exist yet)

**Critical Gap:** Kodefikasi mapping CRUD is completely missing. This blocks the ability to map non-standard BMN codes from MonSAKTI to standard codes.

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

**Status:** 70% complete - core workflow functional, needs workflow integration and advanced features

**Codebase Location:** `layanan/perlengkapan/crates/api` (backend), `antarmuka/perlengkapan` (frontend)

**What's Working:**
- ✅ Core kebutuhan BMN workflow (period creation, submission, basic approval)
- ✅ Basic reporting and data display
- ✅ Satker selection and filtering
- ✅ BMN item selection

**What's Missing:**
- ❌ Generic workflow engine integration (currently hardcoded)
- ❌ Pakaian dinas 3-level approval workflow (60% complete - basic structure exists)
- ❌ Roadmap sarpras 5-year planning (30% complete - basic structure only)
- ❌ Advanced analytics and gap analysis
- ❌ Integration with dokumen service for report generation
- ❌ Integration with notifikasi service for workflow notifications

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

**Status:** 80% complete - core workflow functional, needs document generation integration

**Codebase Location:** `layanan/perlengkapan/crates/api` (backend), `antarmuka/perlengkapan` (frontend)

**What's Working:**
- ✅ Core pemakaian BMN workflow (permit creation, submission)
- ✅ Pegawai selection from MySIMKARI integration
- ✅ BMN selection from SIMAN integration
- ✅ Basic validation (availability, duplicate permits)
- ✅ Usage period specification
- ✅ Permit status tracking

**What's Missing:**
- ❌ DOCX draft permit document generation (integration with dokumen service)
- ❌ PDF upload and workflow completion
- ❌ Auto-generated permit numbers
- ❌ Expiry reminders (H-30, H-14, H-7) via notifikasi service
- ❌ Auto-expiry after end date
- ❌ Permit renewal workflow with history tracking
- ❌ Permit revocation workflow
- ❌ Monitoring dashboard for active permits
- ❌ Usage history reports

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

**Status:** 40% complete - frontend exists, backend workflow missing (CRITICAL BLOCKER)

**Codebase Location:** `antarmuka/perlengkapan` (frontend UI exists), backend workflow NOT IMPLEMENTED

**What's Working:**
- ✅ Frontend UI components for SK Penghapusan BMN
- ✅ Basic data structure and forms
- ✅ BMN selection interface

**What's Missing (CRITICAL):**
- ❌ Backend workflow implementation (DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → DOCUMENT_GENERATED → COMPLETED)
- ❌ Integration with workflow service for approval flow
- ❌ Integration with dokumen service for SK generation
- ❌ Integration with integrasi service for BMN details
- ❌ Integration with pemakaian service to validate BMN not in active use
- ❌ Supporting document upload and storage
- ❌ SK number auto-generation
- ❌ Notification integration for workflow transitions
- ❌ Audit logging
- ❌ Dashboard for SK Penghapusan status tracking

**Priority:** HIGH - This is a critical gap blocking the BMN deletion process

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

**Status:** 70% complete - core functional, needs UI and auto-escalation

**Codebase Location:** `layanan/perlengkapan/crates/api/src/workflow`

**What's Working:**
- ✅ Core WorkflowEngine with state machine (engine.rs - 86 lines, full implementation)
- ✅ Workflow configurations for kebutuhan_bmn, pemakaian_bmn, penghapusan_bmn (config.rs)
- ✅ SLA tracking and breach detection (sla.rs)
- ✅ Delegation support (delegation.rs)
- ✅ Document service client integration hooks (dokumen_client.rs)
- ✅ Notification service client integration hooks (notifikasi_client.rs)
- ✅ Workflow monitoring and metrics (monitoring.rs)
- ✅ Parallel approval support (parallel.rs)
- ✅ Integration with KebutuhanBmnService, PemakaianBmnService, PenghapusanBmnService

**What's Missing (30%):**
- ❌ Admin UI for workflow configuration management
- ❌ SLA breach auto-escalation scheduler (detection exists, auto-escalation missing)
- ❌ Workflow monitoring dashboard UI (backend metrics exist, frontend missing)
- ❌ Workflow definition versioning
- ❌ Conditional branching support

**Priority:** MEDIUM - Core engine functional, missing features are enhancements

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

**Status:** 50% complete - storage functional, template system missing

**Codebase Location:** `layanan/perlengkapan/crates/dokumen`

**What's Working:**
- ✅ Basic document storage in object storage
- ✅ SHA-256 checksum validation
- ✅ Document retrieval API
- ✅ Object storage integration (MinIO/S3-compatible)

**What's Missing:**
- ❌ Template-based document generation system
- ❌ DOCX generation for SK Penghapusan BMN
- ❌ DOCX generation for izin pemakaian BMN with pegawai photo and BMN table
- ❌ PDF generation with official letterhead
- ❌ Excel generation for kebutuhan BMN analysis reports
- ❌ CRUD for document templates with preview
- ❌ Document versioning
- ❌ Auto-generated document numbers
- ❌ Integration with workflow service for automatic document generation on approval
- ❌ Document search functionality

**Priority:** HIGH - Required for Pemakaian BMN and Penghapusan BMN workflows

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

**Status:** 50% complete - basic KPIs functional, drill-down and analytics missing

**Codebase Location:** `layanan/perlengkapan/crates/api` (backend), `antarmuka/perlengkapan` (frontend)

**What's Working:**
- ✅ Executive dashboard with basic national KPIs
- ✅ Basic satker filtering
- ✅ Simple data visualization (charts, tables)
- ✅ Basic export to PDF and Excel

**What's Missing:**
- ❌ Drill-down functionality (national → wilayah → satker)
- ❌ Roadmap vs realization visualization
- ❌ Gap analysis per BMN type and satker
- ❌ Heatmap of kebutuhan by region
- ❌ Trend analysis charts (year-over-year)
- ❌ Workflow monitoring dashboard
- ❌ BMN utilization report
- ❌ Consistent filters across dashboards
- ❌ Saved filters/bookmarks
- ❌ Auto-refresh functionality
- ❌ Real-time data updates (currently max 1 hour delay)

**Priority:** MEDIUM - Enhanced features for better decision-making

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

**Status:** 30% complete - schema exists, API calls missing (CRITICAL BLOCKER)

**Codebase Location:** `layanan/integrasi`

**What's Working:**
- ✅ Database schema defined for BMN and pegawai caching
- ✅ gRPC service stubs and protobuf definitions
- ✅ Basic data transformation logic

**What's Missing (CRITICAL):**
- ❌ SIMAN API client implementation (BMN data sync)
- ❌ MySIMKARI API client implementation (pegawai data sync)
- ❌ Daily sync scheduling
- ❌ Incremental sync logic
- ❌ Retry with exponential backoff
- ❌ Circuit breaker pattern
- ❌ On-demand sync per satker
- ❌ Detailed sync history logging
- ❌ Sync status monitoring dashboard
- ❌ Configurable satker code mapping
- ❌ Raw data storage for debugging
- ❌ Event publishing on successful sync
- ❌ API format change handling via configuration

**Impact:** This is a CRITICAL BLOCKER. Without SIMAN/MySIMKARI integration:
- Kebutuhan BMN cannot fetch existing BMN data for analysis
- Kebutuhan BMN cannot fetch pegawai summary for validation
- Pemakaian BMN cannot validate BMN availability
- Pemakaian BMN cannot fetch pegawai details with photo
- Penghapusan BMN cannot fetch BMN details for deletion

**Priority:** CRITICAL - Must implement immediately to unblock all data-dependent features

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

**Status:** 60% complete - schema functional, workflow integration missing

**Codebase Location:** `layanan/perlengkapan/crates/notifikasi`

**What's Working:**
- ✅ Database schema for notifications
- ✅ In-app notification center UI
- ✅ Basic notification display in frontend
- ✅ Notification storage and retrieval API

**What's Missing:**
- ❌ Integration with workflow service for workflow transition notifications
- ❌ Email delivery channel (currently only in-app)
- ❌ Action-required notifications with deep links
- ❌ Notifications to roles (currently only to individual users)
- ❌ Per-user notification preferences
- ❌ Manageable notification templates
- ❌ Auto-reminders based on events (e.g., permit expiry H-30, H-14, H-7)
- ❌ Digest mode for batched notifications
- ❌ Delivery status logging per channel

**Priority:** HIGH - Required for workflow notifications and user engagement

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

### 5.10 Bantuan Service (bantuan)

**Status:** 100% complete - fully functional

**Codebase Location:** `layanan/perlengkapan/crates/bantuan`

**What's Working:**
- ✅ FAQ management (CRUD)
- ✅ Helpdesk ticketing system
- ✅ Chatbot integration
- ✅ Search functionality
- ✅ Category management
- ✅ User feedback system

**Priority:** COMPLETE - No further work required

### 5.11 Authenc Service (authenc)

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

### 5.12 End-to-End Integration Requirements

**Status:** 30% complete - critical integrations missing

**Current State:**
- ✅ Frontend → Backend REST API communication working
- ✅ Backend → Authenc gRPC authentication working
- ✅ Backend → Secreton gRPC secrets management working
- ❌ Backend → Integrasi gRPC (blocked by Integrasi service incompleteness)
- ❌ Backend → Workflow gRPC (blocked by Workflow service not existing)
- ❌ Backend → Dokumen gRPC (partially working, template system missing)
- ❌ Backend → Notifikasi gRPC (partially working, workflow integration missing)

**Priority:** CRITICAL - These integrations are essential for end-to-end functionality

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

## 6. Implementation Guidance

### 6.1 Codebase Structure Alignment

All requirements MUST align with the actual codebase structure:

| Service | Backend Location | Frontend Location | Database Schema |
|---------|------------------|-------------------|-----------------|
| Kebutuhan BMN | `layanan/perlengkapan/crates/api` | `antarmuka/perlengkapan/src/pages/kebutuhan/` | `perlengkapan.kebutuhan_*` tables |
| Pemakaian BMN | `layanan/perlengkapan/crates/api` | `antarmuka/perlengkapan/src/pages/pemakaian/` | `perlengkapan.pemakaian_*` tables |
| Penghapusan BMN | `layanan/perlengkapan/crates/api` (TO BE CREATED) | `antarmuka/perlengkapan/src/pages/penghapusan/` | `perlengkapan.penghapusan_*` tables |
| Pakaian Dinas | `layanan/perlengkapan/crates/api` | `antarmuka/perlengkapan/src/pages/pakaian_dinas/` | `perlengkapan.pakaian_dinas_*` tables |
| Roadmap Sarpras | `layanan/perlengkapan/crates/api` | `antarmuka/perlengkapan/src/pages/roadmap/` | `perlengkapan.roadmap_*` tables |
| Master Data | `layanan/perlengkapan/crates/master` (TO BE CREATED) | `antarmuka/perlengkapan/src/pages/master/` | `perlengkapan.master_*` tables |
| Workflow | `layanan/perlengkapan/crates/workflow` (TO BE CREATED) | N/A (backend only) | `perlengkapan.workflow_*` tables |
| Dokumen | `layanan/perlengkapan/crates/dokumen` | N/A (backend only) | `perlengkapan.dokumen_*` tables |
| Notifikasi | `layanan/perlengkapan/crates/notifikasi` | `antarmuka/perlengkapan/src/components/notifications/` | `perlengkapan.notifikasi_*` tables |
| Bantuan | `layanan/perlengkapan/crates/bantuan` | `antarmuka/perlengkapan/src/pages/bantuan/` | `perlengkapan.bantuan_*` tables |
| Integrasi | `layanan/integrasi` | N/A (backend only) | `integrasi.*` tables |

### 6.2 Critical Implementation Priorities

#### Phase 1: Unblock Core Features (Weeks 1-4)

**1.1 Complete Integrasi Service (CRITICAL)**
- Location: `layanan/integrasi/src/`
- Tasks:
  - Implement SIMAN API client for BMN data sync
  - Implement MySIMKARI API client for pegawai data sync
  - Implement daily sync scheduler with cron
  - Implement retry logic with exponential backoff
  - Implement circuit breaker pattern
  - Add sync status monitoring dashboard
- Blockers Removed: Kebutuhan BMN analysis, Pemakaian BMN validation, Penghapusan BMN validation

**1.2 Complete Workflow Engine UI (MEDIUM)**
- Location: `layanan/perlengkapan/crates/api/src/workflow/` (ALREADY EXISTS - 70% complete)
- Tasks:
  - Implement admin UI for workflow configuration management
  - Implement SLA breach auto-escalation scheduler (detection already exists)
  - Implement workflow monitoring dashboard UI (backend metrics already exist)
  - Add workflow definition versioning
  - Add conditional branching support (optional)
- Blockers Removed: Workflow configuration management

**1.3 Wire Notifikasi to Workflow Events (HIGH)**
- Location: `layanan/perlengkapan/crates/notifikasi/src/`
- Tasks:
  - Subscribe to workflow state transition events
  - Implement notification template system
  - Implement email delivery channel
  - Implement action-required notifications with deep links
  - Implement auto-reminders (permit expiry H-30, H-14, H-7)
- Blockers Removed: User notifications for workflow state changes

#### Phase 2: Complete Core Modules (Weeks 5-8)

**2.1 Complete Penghapusan BMN Workflow (HIGH)**
- Location: `layanan/perlengkapan/crates/api/src/penghapusan/` (NEW MODULE)
- Tasks:
  - Implement backend workflow (DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → DOCUMENT_GENERATED → COMPLETED)
  - Integrate with workflow service
  - Integrate with dokumen service for SK generation
  - Integrate with integrasi service for BMN details
  - Integrate with pemakaian service to validate BMN not in active use
  - Implement supporting document upload and storage
  - Implement SK number auto-generation
  - Add audit logging

**2.2 Complete Dokumen Template System (HIGH)**
- Location: `layanan/perlengkapan/crates/dokumen/src/`
- Tasks:
  - Implement template-based document generation
  - Implement DOCX generation for SK Penghapusan BMN
  - Implement DOCX generation for izin pemakaian BMN with pegawai photo and BMN table
  - Implement PDF generation with official letterhead
  - Implement Excel generation for kebutuhan BMN analysis reports
  - Implement CRUD for document templates with preview
  - Implement document versioning
  - Implement auto-generated document numbers

**2.3 Complete Pakaian Dinas Approval Workflow (HIGH)**
- Location: `layanan/perlengkapan/crates/api/src/pakaian_dinas/`
- Tasks:
  - Implement 3-level hierarchical approval workflow (Kejari→Kejati→Kejagung)
  - Implement revision workflow returning to appropriate level
  - Implement pegawai data snapshot at submission time
  - Implement pegawai_pakaian_dinas master data update on workflow completion
  - Implement Laporan Daftar (individual pegawai list) generation
  - Implement Laporan Rekap (aggregated summary by ukuran) generation
  - Implement report filters (jenis pegawai, eselon, jenis_kelamin)

**2.4 Implement Master Data Kodefikasi Mapping (HIGH)**
- Location: `layanan/perlengkapan/crates/master/` (NEW CRATE)
- Tasks:
  - Implement kode barang CRUD with BMN format validation
  - Implement standar spesifikasi CRUD with dynamic JSONB fields
  - Implement standar jumlah CRUD with multiple calculation types
  - Implement kodefikasi mapping CRUD (non-standard → standard)
  - Implement auto-detection of non-standard kode barang from MonSAKTI
  - Implement UI for proposing and verifying kode barang mapping
  - Implement dashboard for mapping kodefikasi progress

#### Phase 3: Enhanced Features (Weeks 9-12)

**3.1 Complete Dashboard Drill-Down and Analytics (MEDIUM)**
- Location: `layanan/perlengkapan/crates/api/src/dashboard/`, `antarmuka/perlengkapan/src/pages/dashboard/`
- Tasks:
  - Implement drill-down functionality (national → wilayah → satker)
  - Implement roadmap vs realization visualization
  - Implement gap analysis per BMN type and satker
  - Implement heatmap of kebutuhan by region
  - Implement trend analysis charts (year-over-year)
  - Implement workflow monitoring dashboard
  - Implement BMN utilization report
  - Implement consistent filters across dashboards
  - Implement saved filters/bookmarks

**3.2 Complete Roadmap Sarpras 5-Year Planning (MEDIUM)**
- Location: `layanan/perlengkapan/crates/api/src/roadmap/`, `antarmuka/perlengkapan/src/pages/roadmap/`
- Tasks:
  - Implement 5-year roadmap sarpras feature
  - Implement sync fulfillment realization with SIMAN
  - Implement gap analysis visualization
  - Implement trend analysis year-over-year
  - Implement benchmarking between similar satkers
  - Implement differentiation between new procurement vs replacement

**3.3 Implement Advanced Search and Export (MEDIUM)**
- Location: `antarmuka/perlengkapan/src/components/search/`
- Tasks:
  - Implement advanced search UI with multiple filters
  - Implement export to Excel for all major reports
  - Implement export to PDF for all major reports
  - Implement batch operations UI

#### Phase 4: Nice-to-Have Features (Weeks 13+)

**4.1 Real-time WebSocket Updates (LOW)**
- Location: `layanan/perlengkapan/crates/api/src/websocket/`
- Tasks:
  - Implement WebSocket server for real-time updates
  - Implement frontend WebSocket client
  - Implement real-time notification delivery
  - Implement real-time dashboard updates

**4.2 Advanced Analytics and Benchmarking (LOW)**
- Location: `layanan/perlengkapan/crates/api/src/analytics/`
- Tasks:
  - Implement advanced analytics algorithms
  - Implement benchmarking between similar satkers
  - Implement predictive analytics for BMN requirements

### 6.3 Testing Strategy

#### Unit Tests
- Location: `layanan/perlengkapan/crates/*/tests/`
- Target: ≥ 70% code coverage
- Focus: Business logic, validation, data transformation

#### Integration Tests
- Location: `tests/integration/`
- Focus: Service-to-service communication (gRPC), database operations, external API calls

#### End-to-End Tests
- Location: `tests/e2e/`
- Focus: Complete user workflows (kebutuhan BMN submission, pemakaian BMN permit creation, penghapusan BMN SK generation)

#### Load Tests
- Location: `tests/load/`
- Focus: Performance under load (500 concurrent users, 100 requests/second)

### 6.4 Deployment Strategy

#### Deployment Order (Based on Dependencies)
1. **authenc** - no SIMPEL dependencies (ALREADY DEPLOYED)
2. **secreton** - no SIMPEL dependencies (ALREADY DEPLOYED)
3. **notifikasi** - depends on authenc only
4. **integrasi** - depends on authenc, notifikasi
5. **master** - depends on authenc, integrasi, notifikasi
6. **workflow** - depends on authenc, notifikasi
7. **dokumen** - depends on authenc, notifikasi, workflow
8. **bantuan** - depends on authenc (ALREADY DEPLOYED)
9. **kebutuhan** - depends on master, workflow, dokumen, integrasi, notifikasi, authenc
10. **pemakaian** - depends on master, workflow, dokumen, integrasi, notifikasi, authenc
11. **penghapusan** - depends on master, pemakaian, workflow, dokumen, integrasi, notifikasi, authenc
12. **dashboard** - depends on almost all services (deploy last)

#### Rollout Strategy
- **Blue-Green Deployment:** Zero-downtime deployment with traffic switching
- **Canary Deployment:** Gradual rollout to 10% → 50% → 100% of users
- **Feature Flags:** Enable/disable features without redeployment
- **Database Migrations:** Versioned migrations with rollback capability

## 7. Non-Functional Requirements

### 7.1 Performance

| ID | Requirement | Target | Current Status |
|----|-------------|--------|----------------|
| NFR-P001 | Web page response time (90th percentile) | ≤ 2 seconds | ✅ Meeting target |
| NFR-P002 | API response time (95th percentile) | ≤ 500ms | ✅ Meeting target |
| NFR-P003 | PDF generation time | ≤ 5 seconds per document | ⚠️ Not tested (dokumen service incomplete) |
| NFR-P004 | Dashboard load time (national data) | ≤ 5 seconds | ⚠️ Partially meeting (drill-down not implemented) |
| NFR-P005 | API throughput | ≥ 100 requests/second | ✅ Meeting target |
| NFR-P006 | Dashboard data delay | ≤ 1 hour from source | ⚠️ Not tested (integrasi sync not implemented) |
| NFR-P007 | MonSAKTI full sync time | ≤ 4 hours (off-peak) | ❌ Not implemented |
| NFR-P008 | Incremental sync time | ≤ 30 minutes | ❌ Not implemented |

### 7.2 Availability & Reliability

| ID | Requirement | Target | Current Status |
|----|-------------|--------|----------------|
| NFR-A001 | System uptime | 99.5% (excluding scheduled maintenance) | ✅ Meeting target (infrastructure ready) |
| NFR-A002 | Maintenance window | ≤ 4 hours/month, outside business hours | ✅ Meeting target |
| NFR-A003 | Recovery Time Objective (RTO) | ≤ 4 hours | ✅ Meeting target (K8s auto-recovery) |
| NFR-A004 | Recovery Point Objective (RPO) | ≤ 1 hour | ✅ Meeting target (continuous WAL archiving) |
| NFR-A005 | Database backup | Daily full + continuous WAL archiving | ✅ Implemented |

### 7.3 Security

| ID | Requirement | Target | Current Status |
|----|-------------|--------|----------------|
| NFR-S001 | Data in transit encryption | TLS 1.2+ | ✅ Implemented (mTLS for gRPC) |
| NFR-S002 | Data at rest encryption | AES-256 for sensitive data | ✅ Implemented (Secreton) |
| NFR-S003 | Password hashing | Argon2id | ✅ Implemented (Authenc) |
| NFR-S004 | Session management | JWT (15min access, 7day refresh) | ✅ Implemented (Authenc) |
| NFR-S005 | OWASP Top 10 | Mitigate all vulnerabilities | ✅ Implemented |
| NFR-S006 | Audit trail retention | ≥ 5 years, immutable | ✅ Implemented (Authenc) |
| NFR-S007 | Rate limiting | Login: 5/min per IP; API: 100/min per user | ✅ Implemented |
| NFR-S008 | Input validation | Server-side validation, output sanitization | ✅ Implemented |

### 7.4 Scalability

| ID | Requirement | Target | Current Status |
|----|-------------|--------|----------------|
| NFR-SC001 | Concurrent users | ≥ 500 users | ✅ Meeting target (tested) |
| NFR-SC002 | BMN records | ≥ 1 million records | ⚠️ Not tested (integrasi sync not implemented) |
| NFR-SC003 | Pegawai records | ≥ 50,000 records | ⚠️ Not tested (integrasi sync not implemented) |
| NFR-SC004 | Horizontal scaling | All microservices independently scalable | ✅ Implemented (K8s) |

### 7.5 Usability

| ID | Requirement | Target | Current Status |
|----|-------------|--------|----------------|
| NFR-U001 | Responsive design | Desktop (≥1024px), Tablet (≥768px) | ✅ Implemented (Tailwind CSS) |
| NFR-U002 | Browser support | Chrome, Firefox, Edge (2 latest versions) | ✅ Implemented (WASM CSR) |
| NFR-U003 | Interface language | Bahasa Indonesia | ✅ Implemented |
| NFR-U004 | Accessibility | WCAG 2.1 Level AA | ⚠️ Partially implemented (needs audit) |
| NFR-U005 | Training time | ≤ 2 hours for basic tasks | ⚠️ Not tested (user training pending) |

### 7.6 Maintainability

| ID | Requirement | Target | Current Status |
|----|-------------|--------|----------------|
| NFR-M001 | API documentation | OpenAPI/Swagger for all services | ⚠️ Partially implemented (needs completion) |
| NFR-M002 | Logging | Structured JSON logging to centralized system | ✅ Implemented (Prometheus + Grafana) |
| NFR-M003 | Health checks | /health endpoint for all services | ✅ Implemented |
| NFR-M004 | Monitoring | Prometheus + Grafana metrics | ✅ Implemented |
| NFR-M005 | Database migrations | Versioned migration scripts | ✅ Implemented |
| NFR-M006 | Code coverage | ≥ 70% unit test coverage | ⚠️ Partially meeting (needs improvement) |

## 8. Service Dependencies

### 8.1 Dependency Matrix

| Consumer ↓ / Provider → | master | kebutuhan | pemakaian | penghapusan | workflow | dokumen | dashboard | integrasi | notifikasi | bantuan | authenc |
|-------------------------|--------|-----------|-----------|-------------|----------|---------|-----------|-----------|------------|---------|---------|
| master | — | | | | | | | ✓ | ✓ | | ✓ |
| kebutuhan | ✓ | — | | | ✓ | ✓ | | ✓ | ✓ | | ✓ |
| pemakaian | ✓ | | — | | ✓ | ✓ | | ✓ | ✓ | | ✓ |
| penghapusan | ✓ | | ✓ | — | ✓ | ✓ | | ✓ | ✓ | | ✓ |
| workflow | | | | | — | ✓ | | | ✓ | | ✓ |
| dokumen | | | | | | — | | | ✓ | | ✓ |
| dashboard | ✓ | ✓ | ✓ | ✓ | ✓ | | — | ✓ | | | ✓ |
| integrasi | | | | | | | | — | ✓ | | ✓ |
| notifikasi | | | | | | | | | — | | ✓ |
| bantuan | | | | | | | | | ✓ | — | ✓ |
| authenc | | | | | | | | ✓ | | | — |

### 8.2 Deployment Order

Based on dependencies and current implementation status:

**Already Deployed (100% Complete):**
1. **authenc** - no SIMPEL dependencies
2. **secreton** - no SIMPEL dependencies
3. **bantuan** - depends on authenc only

**Phase 1 (Critical - Weeks 1-4):**
4. **notifikasi** - depends on authenc only (60% → 100%)
5. **integrasi** - depends on authenc, notifikasi (30% → 100%)
6. **workflow** - depends on authenc, notifikasi (70% → 100% - complete UI and auto-escalation)

**Phase 2 (High Priority - Weeks 5-8):**
7. **master** - depends on authenc, integrasi, notifikasi (0% → 100%)
8. **dokumen** - depends on authenc, notifikasi, workflow (50% → 100%)
9. **kebutuhan** - depends on master, workflow, dokumen, integrasi, notifikasi, authenc (70% → 100%)
10. **pemakaian** - depends on master, workflow, dokumen, integrasi, notifikasi, authenc (80% → 100%)

**Phase 3 (Medium Priority - Weeks 9-12):**
11. **penghapusan** - depends on master, pemakaian, workflow, dokumen, integrasi, notifikasi, authenc (40% → 100%)
12. **dashboard** - depends on almost all services (50% → 100%)

## 9. Glossary

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

**Document Status:** UPDATED v4.0.0 - Comprehensive Implementation Analysis
**Date:** February 11, 2026
**Classification:** Internal - Kejaksaan Republik Indonesia
**Overall System Completion:** 65-70%

**Critical Blockers:**
1. 🔴 Integrasi Service (30%) - SIMAN/MySIMKARI API integration incomplete
2. 🔴 Workflow Engine (0%) - Generic workflow service missing
3. 🔴 Notifikasi Integration (60%) - Not wired to workflow transitions
4. 🔴 Penghapusan BMN Workflow (40%) - Backend workflow incomplete

**Next Steps:**
- Phase 1 (Weeks 1-4): Complete Integrasi, Workflow, Notifikasi integration
- Phase 2 (Weeks 5-8): Complete Penghapusan, Dokumen, Pakaian Dinas, Master Data
- Phase 3 (Weeks 9-12): Complete Dashboard, Roadmap Sarpras, Advanced Features

