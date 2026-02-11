# Requirements Document: SIMPEL - Sistem Informasi Manajemen Perlengkapan

## 1. Introduction

### 1.1 Purpose

This document defines the complete requirements for SIMPEL (Sistem Informasi Manajemen Perlengkapan), a comprehensive Barang Milik Negara (BMN) management system for the Indonesian Attorney General's Office (Kejaksaan RI). SIMPEL manages non-SBSK BMN requirements, complementing the SIMAN v2 RKBMN module which handles SBSK-compliant BMN.

### 1.2 Scope

SIMPEL consists of nine microservices handling:
1. Master data and reference management
2. BMN requirements analysis (including pakaian dinas and roadmap)
3. BMN usage permits (vehicles, housing, laptops)
4. Workflow engine for approvals
5. Document generation (SK Penghapusan, permits, reports)
6. Dashboard and analytics
7. Integration gateway (MonSAKTI, MySIMKARI)
8. Multi-channel notifications
9. Authentication and authorization

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
| Workflow | workflow | 0% | Centralized workflow engine |
| Dokumen | dokumen | 0% | Document generation (SK, permits, reports) |
| Dashboard | dashboard | 50% | Analytics and visualization |
| Integrasi | integrasi | 30% | MonSAKTI/MySIMKARI adapter and cache |
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

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-K001 | High | System SHALL auto-calculate kebutuhan based on standar jumlah minus existing good condition BMN | System |
| REQ-K002 | High | System SHALL provide period management for kebutuhan collection with deadlines | Admin Pusat |
| REQ-K003 | High | System SHALL allow Operator to revise auto-calculation with justification | Operator |
| REQ-K004 | High | System SHALL support multi-level approval workflow | All |
| REQ-K005 | High | System SHALL provide kebutuhan data bank with filters | All |
| REQ-K006 | High | System SHALL manage pakaian dinas pengajuan with Admin Pusat creating periods | Admin Pusat |
| REQ-K007 | High | System SHALL support satker selection via hierarchical tree based on MonSAKTI wilayah codes (0100=Kejagung, 0200=Jabar, 3400=Sulbar, etc.) with multi-select | Admin Pusat |
| REQ-K008 | High | System SHALL allow Pelaksana Satker to input pegawai ukuran per pakaian type with gender-specific options | Operator |
| REQ-K009 | High | System SHALL implement 3-level hierarchical approval workflow (Kejari→Kejati→Kejagung) with state tracking | Verifikator, Pimpinan |
| REQ-K010 | High | System SHALL generate Laporan Daftar (individual pegawai list) with columns: No, NIP, Nama, Pangkat, Jabatan, Eselon, Jenis Pegawai, Ukuran per Pakaian | Admin, Operator |
| REQ-K011 | High | System SHALL generate Laporan Rekap (aggregated summary by ukuran) with L/P breakdown per satker | Admin Pusat |
| REQ-K012 | High | System SHALL update pegawai_pakaian_dinas master data on workflow completion (COMPLETED state) | System |
| REQ-K013 | High | System SHALL support revision workflow returning to appropriate level (Kejati revision→Kejari, Kejagung revision→Kejati/Kejari) | Verifikator |
| REQ-K014 | High | System SHALL support filters for reports (jenis pegawai: TU/Jaksa, eselon, jenis_kelamin: L/P) | Admin, Operator |
| REQ-K015 | High | System SHALL handle pusat satker (ms_satker_id='00') with unit_kerja mapping (ms_satker_pusat_id) | System |
| REQ-K016 | High | System SHALL group ukuran by category (BAJU, CELANA, SEPATU) per pakaian specification | System |
| REQ-K017 | High | System SHALL support hijab option (with_hijab flag) for female employees | Operator |
| REQ-K018 | High | System SHALL snapshot pegawai data at submission time with eselon, pangkat, jabatan | System |
| REQ-K019 | Medium | System SHALL cache satker tree (1 hour TTL) and pegawai list (30 minutes TTL) | System |
| REQ-K020 | Medium | System SHALL use CTE-based SQL for report generation performance | System |
| REQ-K008 | Medium | System SHALL provide 5-year roadmap sarpras feature | Admin, Pimpinan |
| REQ-K009 | Medium | System SHALL sync fulfillment realization with MonSAKTI | System |
| REQ-K010 | Medium | System SHALL provide gap analysis visualization | Admin, Pimpinan |
| REQ-K011 | Medium | System SHALL provide trend analysis year-over-year | Admin, Pimpinan |
| REQ-K012 | Medium | System SHALL lock data after period closure | Admin Pusat |
| REQ-K013 | High | System SHALL snapshot pegawai data at submission time | System |
| REQ-K014 | Medium | System SHALL support export to Excel | All |
| REQ-K015 | Low | System SHALL provide benchmarking between similar satkers | Admin Pusat |
| REQ-K016 | High | System SHALL differentiate new procurement vs replacement | Operator |

### 5.3 Pemakaian BMN Service (pemakaian)

**Status:** 0% complete - new module

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-P001 | High | System SHALL provide dynamic forms for different BMN types | Pegawai, Operator |
| REQ-P002 | High | System SHALL display available BMN (no active permit) | Pegawai, Operator |
| REQ-P003 | High | System SHALL validate one BMN = one active permit | System |
| REQ-P004 | High | System SHALL support configurable approval workflow | Verifikator, Pimpinan |
| REQ-P005 | High | System SHALL auto-generate permit numbers | System |
| REQ-P006 | High | System SHALL generate permit documents via dokumen service | System |
| REQ-P007 | High | System SHALL send reminders at H-30, H-14, H-7 before expiry | System |
| REQ-P008 | High | System SHALL support permit renewal with history tracking | Pegawai, Operator |
| REQ-P009 | High | System SHALL support permit revocation with reason | Pimpinan |
| REQ-P010 | High | System SHALL auto-expire permits after end date | System |
| REQ-P011 | Medium | System SHALL provide active usage monitoring dashboard | Admin, Pimpinan |
| REQ-P012 | Medium | System SHALL provide usage history per BMN and per pegawai | Admin, Pimpinan |
| REQ-P013 | Medium | System SHALL provide BMN utilization report | Admin, Pimpinan |
| REQ-P014 | Medium | System SHALL support document upload (SK, etc.) | Pegawai, Operator |
| REQ-P015 | Low | System SHALL notify on pegawai mutation/retirement | System |
| REQ-P016 | High | System SHALL audit log all permit status changes | System |

### 5.4 Workflow Service (workflow)

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

### 5.5 Dokumen Service (dokumen)

**Status:** 0% complete - new module

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-D001 | High | System SHALL provide template-based document generation | System |
| REQ-D002 | High | System SHALL generate SK Penghapusan BMN with official format | Admin Pusat |
| REQ-D003 | High | System SHALL auto-generate permits after workflow completion | System |
| REQ-D004 | High | System SHALL generate PDF with official letterhead | System |
| REQ-D005 | High | System SHALL provide API for document generation requests | System |
| REQ-D006 | Medium | System SHALL provide CRUD for document templates with preview | Admin Pusat |
| REQ-D007 | Medium | System SHALL store documents in object storage with SHA-256 checksum | System |
| REQ-D008 | Medium | System SHALL support document versioning | Admin Pusat |
| REQ-D009 | Medium | System SHALL provide document search | All |
| REQ-D010 | Medium | System SHALL auto-generate document numbers | System |
| REQ-D011 | Low | System SHALL generate rekapitulasi in PDF and Excel | Admin Pusat |
| REQ-D012 | Low | System SHALL support document retention policy | System |
| REQ-D013 | High | System SHALL audit log document operations | System |

### 5.6 Dashboard Service (dashboard)

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

### 5.7 Integrasi Service (integrasi)

**Status:** 30% complete - schema exists, needs implementation

#### Functional Requirements

| ID | Priority | Requirement | Actor |
|----|----------|-------------|-------|
| REQ-I001 | High | System SHALL sync BMN data from MonSAKTI daily | System |
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

### 5.8 Notifikasi Service (notifikasi)

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

### 5.9 Authenc Service (authenc)

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

| Consumer ↓ / Provider → | master | kebutuhan | pemakaian | workflow | dokumen | dashboard | integrasi | notifikasi | authenc |
|-------------------------|--------|-----------|-----------|----------|---------|-----------|-----------|------------|---------|
| master | — | | | | | | ✓ | ✓ | ✓ |
| kebutuhan | ✓ | — | | ✓ | ✓ | | ✓ | ✓ | ✓ |
| pemakaian | ✓ | | — | ✓ | ✓ | | ✓ | ✓ | ✓ |
| workflow | | | | — | | | | ✓ | ✓ |
| dokumen | | | | | — | | | ✓ | ✓ |
| dashboard | ✓ | ✓ | ✓ | ✓ | | — | ✓ | | ✓ |
| integrasi | | | | | | | — | ✓ | ✓ |
| notifikasi | | | | | | | | — | ✓ |
| authenc | | | | | | | ✓ | | — |

### 7.2 Deployment Order

Based on dependencies:
1. **authenc** - no SIMPEL dependencies
2. **notifikasi** - depends on authenc only
3. **integrasi** - depends on authenc, notifikasi
4. **master** - depends on authenc, integrasi, notifikasi
5. **workflow** - depends on authenc, notifikasi
6. **dokumen** - depends on authenc, notifikasi
7. **kebutuhan** - depends on many services
8. **pemakaian** - depends on many services
9. **dashboard** - depends on almost all services (deploy last)

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
| Surat Perintah Pencairan Dana | SP2D | Payment order |
| Service Level Agreement | SLA | Process completion time limit |
| Konstruksi Dalam Pengerjaan | KDP | Construction in progress |
| Kartu Inventaris Barang | KIB | Asset inventory card |
| Time-based One-Time Password | TOTP | Two-factor authentication method |
| JSON Web Token | JWT | Authentication token standard |
| Role-Based Access Control | RBAC | Role-based access control model |

---

**Document Status:** DRAFT v2.0.0
**Date:** February 9, 2026
**Classification:** Internal - Kejaksaan Republik Indonesia
