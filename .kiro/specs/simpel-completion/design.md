# Design Document: SIMPEL Completion

## 1. Overview

### 1.1 Purpose

This design document provides a comprehensive technical specification for completing SIMPEL (Sistem Informasi Manajemen Perlengkapan) based on **actual codebase analysis**. This document reflects what is **actually implemented** versus what is missing, providing a realistic roadmap to production.

**Document Status:** Updated based on codebase inspection (migrations, routes, handlers, services)

**Key Focus Areas:**
1. Document actual database schema (from migration files)
2. Document actual API endpoints (from routes.rs)
3. Identify implementation gaps with evidence
4. Provide minimal, actionable completion guidance
5. Reference actual file paths and code

### 1.2 Current Implementation Status (Evidence-Based)

**Codebase Location:** `layanan/perlengkapan/crates/api/`

#### ✅ Fully Implemented Modules

| Module | Evidence | Status |
|--------|----------|--------|
| **Kebutuhan BMN** | `src/kebutuhan_bmn/` (handlers, services, repository, models) | 70% - Core CRUD + workflow |
| **Pakaian Dinas** | `src/pakaian_dinas/` (handlers, services, repository, export) | 60% - Data structure + basic workflow |
| **Pemakaian BMN** | `src/pemakaian_bmn/` (handlers, services, scheduler) | 80% - Core workflow + scheduler |
| **Penghapusan BMN** | `src/penghapusan_bmn/` (handlers, services, repository) | 40% - Handlers exist, workflow incomplete |
| **Dashboard** | `src/dashboard/` (handlers, websocket, services) | 50% - Basic metrics + WebSocket |
| **Mapping Kodefikasi** | `src/mapping_kodefikasi/` (handlers, services) | Read-only detection |
| **Roadmap Sarpras** | `src/roadmap_sarpras/` (handlers, services) | 30% - Basic structure |
| **Workflow Engine** | `src/workflow/` (engine, config, SLA, delegation) | **70% - EXISTS!** |

#### 🔴 Critical Discovery: Workflow Engine EXISTS

**Location:** `layanan/perlengkapan/crates/api/src/workflow/`

**Files Found:**
- `engine.rs` - WorkflowEngine with transition logic
- `config.rs` - Workflow configuration
- `sla.rs` - SLA tracking
- `delegation.rs` - Delegation support
- `dokumen_client.rs` - Document service client
- `notifikasi_client.rs` - Notification service client
- `monitoring.rs` - Workflow monitoring
- `parallel.rs` - Parallel approval support

**Status:** Workflow engine is **70% complete**, NOT 0% as requirements stated!

### 1.3 Technology Stack

| Component | Technology | Version | Location |
|-----------|-----------|---------|----------|
| Language | Rust | Edition 2024, MSRV 1.90+ | Workspace |
| Backend HTTP | Axum | 0.8.7 | `layanan/perlengkapan/crates/api` |
| Backend gRPC | Tonic + Prost | 0.14.x | Authenc/Secreton clients |
| Frontend | Leptos | 0.8.14 | `antarmuka/perlengkapan` |
| Database | PostgreSQL | 15+ | `perlengkapan` schema |
| Caching | Redis | - | Connection pooling |
| Identity | Authenc | gRPC | `layanan/authenc` |
| Secrets | Secreton | gRPC | `layanan/secreton` |

## 2. Database Design (Actual Schema)

### 2.1 Schema Overview

**Database:** `perlengkapan`
**Migration Files:** `layanan/perlengkapan/crates/api/migrations/`

**Schemas:**
- `perlengkapan` - Main application schema
- `integrasi` - External system integration data (SIMAN, MySIMKARI)

### 2.2 Kebutuhan BMN Tables (ACTUAL)

**Source:** `20260202_create_kebutuhan_bmn_tables.sql`

**Main Tables:**
1. `perlengkapan.ms_aktivitas_bmn` - Workflow status codes (2000-2009)
2. `perlengkapan.pengajuan_kebutuhan_bmn` - Main request entity
3. `perlengkapan.pengajuan_kebutuhan_bmn_asset` - Asset types per request
4. `perlengkapan.pengajuan_kebutuhan_bmn_satker` - Per-satker tracking
5. `perlengkapan.pengajuan_kebutuhan_bmn_satker_barang` - Individual goods per satker
6. `perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas` - Workflow history

**Key Fields:**
```sql
-- pengajuan_kebutuhan_bmn
id UUID PRIMARY KEY
nama VARCHAR(255) NOT NULL
tahun INTEGER NOT NULL
tgl_mulai DATE NOT NULL
tgl_selesai DATE NOT NULL
pilihan_satker VARCHAR(20) -- 'semua' | 'sebagian'
id_jenis_asset JSONB DEFAULT '[]'::jsonb
status_kode INTEGER NOT NULL DEFAULT 2000 -- FK to ms_aktivitas_bmn
is_appv_daskrimti BOOLEAN DEFAULT FALSE
created_by UUID
version INTEGER DEFAULT 1 -- Optimistic locking

-- pengajuan_kebutuhan_bmn_satker
id UUID PRIMARY KEY
pengajuan_id UUID NOT NULL -- FK
ms_satker_id VARCHAR(20) NOT NULL
status_kode INTEGER NOT NULL DEFAULT 2001
prioritas INTEGER DEFAULT 0

-- pengajuan_kebutuhan_bmn_satker_barang
id UUID PRIMARY KEY
pengajuan_satker_id UUID NOT NULL -- FK
nama VARCHAR(255) NOT NULL
kode_barang VARCHAR(50)
jumlah INTEGER NOT NULL CHECK (jumlah > 0)
jml_setuju INTEGER DEFAULT 0 CHECK (jml_setuju >= 0)
prioritas INTEGER DEFAULT 0
skor NUMERIC(10,2) DEFAULT 0
file_pendukung JSONB DEFAULT '[]'::jsonb
existing_count INTEGER DEFAULT 0 -- From SIMAN
```

**Indexes:**
- `idx_pkb_tahun` ON tahun
- `idx_pkb_status` ON status_kode
- `idx_pkb_satker_pengajuan` ON pengajuan_id
- `idx_pkb_barang_prioritas` ON prioritas
- `idx_pkb_barang_skor` ON skor DESC

**View:**
- `vw_kebutuhan_bmn_summary` - Dashboard statistics

### 2.3 Pakaian Dinas Tables (ACTUAL)

**Source:** `20260202_create_pakaian_dinas_tables.sql`

**Master Tables:**
1. `ms_jenis_pakaian_dinas` - Uniform types (PDH, PDL, Toga)
2. `ms_spesifikasi_pakaian_dinas` - Specifications with size groups
3. `ms_subspesifikasi_pakaian_dinas` - Sub-specifications
4. `ms_ukuran` - Size master (BAJU, CELANA, SEPATU)

**Transaction Tables:**
1. `pengajuan_pakaian_dinas` - Main request header
2. `pengajuan_pakaian_dinas_satker_terpilih` - Selected satkers
3. `pengajuan_pakaian_dinas_pakaian` - Clothing items in request
4. `pengajuan_pakaian_dinas_satker` - Per-satker submission
5. `pengajuan_pakaian_dinas_satker_pegawai` - Employee data snapshot
6. `pengajuan_pakaian_dinas_satker_pegawai_ukuran` - Employee sizes
7. `pengajuan_pakaian_dinas_satker_aktivitas` - Workflow history
8. `pegawai_pakaian_dinas` - Persistent employee sizes

**Key Fields:**
```sql
-- pengajuan_pakaian_dinas
id UUID PRIMARY KEY
nama VARCHAR(255) NOT NULL
tahun INTEGER NOT NULL
pilihan_satker VARCHAR(20) CHECK (pilihan_satker IN ('all', 'sebagian'))
dengan_unit_kerja BOOLEAN DEFAULT FALSE
jenis_pakaian_dinas_id UUID
aktivitas_id INTEGER NOT NULL DEFAULT 1000

-- pengajuan_pakaian_dinas_satker_pegawai
nip VARCHAR(30) NOT NULL
nama VARCHAR(500) NOT NULL
jenis_kelamin VARCHAR(1) CHECK (jenis_kelamin IN ('L', 'P'))
with_hijab BOOLEAN DEFAULT FALSE -- For female employees
eselon VARCHAR(20)
jenis VARCHAR(100) -- TU/Jaksa

-- ms_ukuran (Pre-populated)
ukuran VARCHAR(20) NOT NULL
"group" VARCHAR(50) CHECK ("group" IN ('BAJU', 'CELANA', 'SEPATU'))
urutan INTEGER DEFAULT 0
PRIMARY KEY (ukuran, "group")
```

**Indexes:**
- `idx_pengajuan_pakaian_dinas_tahun` ON tahun
- `idx_ppd_satker_pengajuan` ON pengajuan_id
- `idx_ppd_satker_pegawai_nip` ON nip

### 2.4 Pemakaian BMN Tables (ACTUAL)

**Source:** `20260211_add_document_fields_to_pemakaian_bmn.sql` (references base table)

**Main Table:** `perlengkapan.izin_pemakaian_bmn`

**Key Fields:**
```sql
id UUID PRIMARY KEY
satker_id UUID NOT NULL
pegawai_nip VARCHAR(30) NOT NULL
pegawai_nama VARCHAR(255) NOT NULL
bmn_nup VARCHAR(50) NOT NULL -- From SIMAN
bmn_nama VARCHAR(255) NOT NULL
tanggal_mulai DATE NOT NULL
tanggal_selesai DATE NOT NULL
status VARCHAR(50) NOT NULL -- DRAFT, ACTIVE, EXPIRED, REVOKED
document_id UUID -- Generated permit document
document_url TEXT -- Download URL
created_by UUID NOT NULL
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
```

**Indexes:**
- `idx_izin_satker` ON satker_id
- `idx_izin_pegawai` ON pegawai_nip
- `idx_izin_bmn` ON bmn_nup
- `idx_izin_status` ON status
- `idx_izin_document_id` ON document_id

### 2.5 Penghapusan BMN Tables (ACTUAL)

**Source:** `20260211_create_penghapusan_bmn_tables.sql`

**Main Tables:**
1. `perlengkapan.penghapusan_bmn` - Deletion requests
2. `perlengkapan.penghapusan_bmn_aktivitas` - Workflow history

**Key Fields:**
```sql
-- penghapusan_bmn
id UUID PRIMARY KEY
satker_id UUID NOT NULL
asset_id UUID NOT NULL
kode_barang VARCHAR(50) NOT NULL
nama_barang VARCHAR(255) NOT NULL
nup VARCHAR(50) NOT NULL
tanggal_penghapusan DATE NOT NULL
alasan TEXT NOT NULL
metode_penghapusan VARCHAR(100) NOT NULL -- DIJUAL, DIHIBAHKAN, DIMUSNAHKAN
nilai_residu DECIMAL(15,2)
status VARCHAR(50) NOT NULL DEFAULT 'DRAFT' -- DRAFT, SUBMITTED, REVIEWED, APPROVED, REJECTED, CANCELLED
document_id UUID -- SK Penghapusan document
document_url TEXT
created_by UUID NOT NULL

-- penghapusan_bmn_aktivitas
id UUID PRIMARY KEY
penghapusan_id UUID NOT NULL -- FK
aktivitas_id INTEGER NOT NULL -- FK to ms_aktivitas_bmn
user_id UUID NOT NULL
catatan TEXT
document_id UUID -- Document generated during activity
document_url TEXT
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
```

**Indexes:**
- `idx_penghapusan_bmn_satker` ON satker_id
- `idx_penghapusan_bmn_asset` ON asset_id
- `idx_penghapusan_bmn_status` ON status
- `idx_penghapusan_bmn_tahun` ON EXTRACT(YEAR FROM tanggal_penghapusan)

### 2.6 Integration Schema (ACTUAL)

**Source:** `20260209_create_integration_schema.sql`

**Schema:** `integrasi`

**SIMAN Asset Tables:**
1. `integrasi.siman_aset_tanah` - Land assets
2. `integrasi.siman_aset_gedung_bangunan` - Buildings
3. `integrasi.siman_aset_alat_besar` - Heavy equipment
4. `integrasi.siman_aset_angkutan_bermotor` - Motor vehicles

**Common SIMAN Fields:**
```sql
id UUID PRIMARY KEY
nup VARCHAR(50) NOT NULL UNIQUE -- Nomor Urut Pendaftaran
kode_barang VARCHAR(50) NOT NULL
nama_barang VARCHAR(255) NOT NULL
kondisi VARCHAR(50) -- BAIK, RUSAK RINGAN, RUSAK BERAT
tahun_perolehan INTEGER
nilai_perolehan NUMERIC(15,2)
satker_code VARCHAR(50)
satker_nama VARCHAR(255)
raw_data JSONB NOT NULL -- Complete API response
synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
```

**MySIMKARI Table:**
```sql
-- integrasi.mysimkari_pegawai
id UUID PRIMARY KEY
nip VARCHAR(20) NOT NULL UNIQUE
nama VARCHAR(255) NOT NULL
satker_id UUID
satker_code VARCHAR(50)
satker_nama VARCHAR(255)
jabatan VARCHAR(255)
golongan VARCHAR(10)
pangkat VARCHAR(100)
eselon VARCHAR(10)
status_pegawai VARCHAR(50)
jenis_pegawai VARCHAR(50) -- TU/Jaksa
email VARCHAR(255)
no_hp VARCHAR(20)
tempat_lahir VARCHAR(100)
tanggal_lahir DATE
jenis_kelamin VARCHAR(10)
tmt_cpns DATE
tmt_pns DATE
raw_data JSONB NOT NULL
synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
```

**Sync Tracking Tables:**
```sql
-- integrasi.api_call_log
id BIGSERIAL PRIMARY KEY
service_name VARCHAR(50) NOT NULL -- SIMAN, MySIMKARI, MonSAKTI
endpoint VARCHAR(255) NOT NULL
method VARCHAR(10) NOT NULL
request_params JSONB
status_code INTEGER
duration_ms INTEGER
error_message TEXT
retry_count INTEGER DEFAULT 0
called_at TIMESTAMPTZ NOT NULL DEFAULT NOW()

-- integrasi.sync_status
id UUID PRIMARY KEY
service_name VARCHAR(50) NOT NULL
sync_type VARCHAR(20) CHECK (sync_type IN ('FULL', 'INCREMENTAL', 'ON_DEMAND'))
entity_type VARCHAR(50) NOT NULL -- aset_tanah, pegawai, etc.
satker_code VARCHAR(50)
status VARCHAR(20) CHECK (status IN ('RUNNING', 'COMPLETED', 'FAILED', 'CANCELLED'))
total_records INTEGER DEFAULT 0
processed_records INTEGER DEFAULT 0
success_records INTEGER DEFAULT 0
failed_records INTEGER DEFAULT 0
started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
completed_at TIMESTAMPTZ
duration_seconds INTEGER
error_message TEXT
```

**Indexes:**
- GIN indexes on all `raw_data` JSONB columns
- Trigram indexes on `nama` fields for full-text search
- Composite indexes on (service_name, called_at) for monitoring

**Views:**
- `v_latest_sync_status` - Latest sync per service/entity
- `v_api_call_statistics` - Hourly API call stats (24h)
- `v_integration_health` - Overall health status

**Helper Functions:**
- `get_asset_count_by_satker(satker_code, kode_barang, kondisi)` - Asset counts
- `get_employee_count_by_satker(satker_code, status_pegawai)` - Employee counts
- `cleanup_old_api_logs()` - Delete logs older than 90 days
- `cleanup_old_sync_status()` - Delete sync records older than 180 days

### 2.7 Workflow Status Codes (ACTUAL)

**Source:** `ms_aktivitas_bmn` table

**Kebutuhan BMN Workflow:**
- 2000: DRAFT - Initial state
- 2001: INPUT_BARANG - Operator entering goods
- 2002: SUBMIT_SATKER - Submitted to validator
- 2003: REVISI_SATKER - Returned for revision
- 2004: ANALISIS_KELAYAKAN - Feasibility analysis
- 2005: PENYUSUNAN_PRIORITAS - Priority setting
- 2006: APPROVED - Approved
- 2007: REJECTED - Rejected
- 2008: COMPLETED - Process complete
- 2009: CANCELLED - Cancelled

**Pakaian Dinas Workflow:**
- 1000: DRAFT (default aktivitas_id)
- Additional states defined in workflow engine

**Penghapusan BMN Workflow:**
- Status stored as VARCHAR in `status` column
- DRAFT, SUBMITTED, REVIEWED, APPROVED, REJECTED, CANCELLED

### 2.8 Missing Tables (Identified Gaps)

**Not Found in Migrations:**
1. Master data tables for kode barang (kodefikasi mapping CRUD)
2. Roadmap sarpras detailed tables (basic structure exists in `20260209_create_new_entity_tables.sql`)
3. Document template tables
4. Notification queue tables (schema may exist in separate notifikasi crate)

**Action Required:**
- Check if master data tables exist in separate migration files
- Verify roadmap_sarpras table structure
- Confirm dokumen and notifikasi schemas

## 3. API Design (Actual Endpoints)

### 3.1 API Routes Overview

**Source:** `layanan/perlengkapan/crates/api/src/routes.rs`

**Base URL:** `/api/v1` (assumed, not explicitly in routes.rs)

### 3.2 Kebutuhan BMN Endpoints (IMPLEMENTED)

**Dashboard:**
- `GET /kebutuhan-bmn/dashboard` - Dashboard statistics

**Pengajuan CRUD:**
- `GET /kebutuhan-bmn/pengajuan` - List all pengajuan
- `POST /kebutuhan-bmn/pengajuan` - Create pengajuan
- `GET /kebutuhan-bmn/pengajuan/{id}` - Get pengajuan by ID
- `PUT /kebutuhan-bmn/pengajuan/{id}` - Update pengajuan
- `DELETE /kebutuhan-bmn/pengajuan/{id}` - Delete pengajuan

**Workflow:**
- `POST /kebutuhan-bmn/pengajuan/{id}/transition` - Transition status
- `GET /kebutuhan-bmn/pengajuan/{id}/export` - Export pengajuan

**Satker Operations:**
- `GET /kebutuhan-bmn/pengajuan/{id}/satker` - Get satkers for pengajuan
- `POST /kebutuhan-bmn/pengajuan/{id}/satker` - Add satker to pengajuan
- `GET /kebutuhan-bmn/satker/{id}` - Get satker detail
- `POST /kebutuhan-bmn/satker/{id}/transition` - Transition satker status
- `GET /kebutuhan-bmn/satker/{id}/aktivitas` - Get satker activity history
- `GET /kebutuhan-bmn/satker/{id}/analisis` - Get feasibility analysis

**Validator Actions:**
- `POST /kebutuhan-bmn/satker/{id}/submit-wilayah` - Submit to wilayah validator
- `POST /kebutuhan-bmn/satker/{id}/validator-wilayah` - Wilayah validator action
- `POST /kebutuhan-bmn/satker/{id}/keputusan-pusat` - Pusat validator decision

**Barang Operations:**
- `POST /kebutuhan-bmn/satker/{id}/barang` - Create barang
- `PUT /kebutuhan-bmn/barang/{id}/approval` - Update barang approval
- `DELETE /kebutuhan-bmn/barang/{id}` - Delete barang

**Priority & Analysis:**
- `POST /kebutuhan-bmn/prioritas` - Set barang priority

**SIMAN Integration:**
- `GET /kebutuhan-bmn/siman/search` - Search SIMAN assets
- `GET /kebutuhan-bmn/siman/summary/{satker_id}` - Get SIMAN satker summary

**Advanced Features:**
- `GET /kebutuhan-bmn/search` - Advanced search
- `GET /kebutuhan-bmn/search/suggestions` - Search suggestions

**Batch Operations:**
- `POST /kebutuhan-bmn/batch/approve` - Batch approve
- `POST /kebutuhan-bmn/batch/reject` - Batch reject
- `POST /kebutuhan-bmn/batch/update-status` - Batch update status

### 3.3 Pakaian Dinas Endpoints (IMPLEMENTED)

**Master Data:**
- `GET /pakaian-dinas/jenis` - List jenis pakaian
- `POST /pakaian-dinas/jenis` - Create jenis pakaian
- `GET /pakaian-dinas/jenis/{id}` - Get jenis by ID
- `PUT /pakaian-dinas/jenis/{id}` - Update jenis
- `DELETE /pakaian-dinas/jenis/{id}` - Delete jenis

- `GET /pakaian-dinas/spesifikasi` - List spesifikasi
- `POST /pakaian-dinas/spesifikasi` - Create spesifikasi
- `GET /pakaian-dinas/spesifikasi/{id}` - Get spesifikasi by ID
- `PUT /pakaian-dinas/spesifikasi/{id}` - Update spesifikasi
- `DELETE /pakaian-dinas/spesifikasi/{id}` - Delete spesifikasi

- `GET /pakaian-dinas/subspesifikasi` - List subspesifikasi
- `POST /pakaian-dinas/subspesifikasi` - Create subspesifikasi
- `GET /pakaian-dinas/subspesifikasi/{id}` - Get subspesifikasi by ID
- `DELETE /pakaian-dinas/subspesifikasi/{id}` - Delete subspesifikasi

- `GET /pakaian-dinas/ukuran` - List ukuran

**Pengajuan:**
- `GET /pakaian-dinas/pengajuan` - List pengajuan
- `POST /pakaian-dinas/pengajuan` - Create pengajuan
- `GET /pakaian-dinas/pengajuan/{id}` - Get pengajuan by ID
- `DELETE /pakaian-dinas/pengajuan/{id}` - Delete pengajuan

**Satker:**
- `GET /pakaian-dinas/pengajuan/{pengajuan_id}/satker` - List satker for pengajuan
- `GET /pakaian-dinas/satker/{id}` - Get satker by ID

**Workflow:**
- `POST /pakaian-dinas/validator-action` - Validator action
- `POST /pakaian-dinas/pengajuan/{id}/submit` - Submit pengajuan
- `POST /pakaian-dinas/pengajuan/{id}/approve` - Approve pengajuan
- `POST /pakaian-dinas/pengajuan/{id}/reject` - Reject pengajuan

**Personal Sizes:**
- `GET /pakaian-dinas/ukuran-pakaian-pegawai` - Get personal sizes
- `POST /pakaian-dinas/ukuran-pakaian-pegawai` - Update personal sizes

**MySIMKARI Integration:**
- `GET /pakaian-dinas/pegawai-satker/{satker_id}` - Get pegawai by satker
- `GET /pakaian-dinas/pegawai-satker/{satker_id}/with-sizes` - Get pegawai with sizes

**Reports:**
- `GET /pakaian-dinas/laporan/rekap-ukuran` - Size summary report
- `GET /pakaian-dinas/laporan/daftar-pegawai` - Employee list report
- `GET /pakaian-dinas/laporan/cetak` - Print report
- `GET /pakaian-dinas/pengajuan/{id}/rekapitulasi` - Download rekapitulasi

### 3.4 Pemakaian BMN Endpoints (IMPLEMENTED)

**Permit CRUD:**
- `GET /pemakaian-bmn` - List permits
- `POST /pemakaian-bmn` - Create permit
- `GET /pemakaian-bmn/{id}` - Get permit by ID
- `PUT /pemakaian-bmn/{id}` - Update permit

**Workflow:**
- `POST /pemakaian-bmn/{id}/transition` - Transition permit status
- `POST /pemakaian-bmn/{id}/activate` - Activate permit
- `GET /pemakaian-bmn/{id}/document` - Get permit document

**Document Generation:**
- `POST /pemakaian-bmn/{id}/generate-konsep-surat` - Generate draft permit
- `POST /pemakaian-bmn/{id}/upload-signed-pdf` - Upload signed PDF

**Permit Management:**
- `POST /pemakaian-bmn/{id}/revoke` - Revoke permit
- `POST /pemakaian-bmn/{id}/renew` - Renew permit

**BMN Tracking:**
- `GET /pemakaian-bmn/bmn/{bmn_nup}/availability` - Check BMN availability
- `GET /pemakaian-bmn/bmn/{bmn_nup}/history` - Get BMN usage history

**Pegawai Tracking:**
- `GET /pemakaian-bmn/pegawai/{pegawai_nip}/history` - Get pegawai usage history

**Expiry Management:**
- `GET /pemakaian-bmn/expiring` - Get expiring permits
- `POST /pemakaian-bmn/auto-expire` - Auto-expire permits

**Monitoring:**
- `GET /pemakaian-bmn/monitoring/active-usage` - Active usage dashboard
- `GET /pemakaian-bmn/monitoring/utilization-report` - BMN utilization report

### 3.5 Penghapusan BMN Endpoints (IMPLEMENTED)

**CRUD:**
- `GET /penghapusan-bmn` - List penghapusan
- `POST /penghapusan-bmn` - Create penghapusan
- `GET /penghapusan-bmn/{id}` - Get penghapusan by ID
- `PUT /penghapusan-bmn/{id}` - Update penghapusan
- `DELETE /penghapusan-bmn/{id}` - Delete penghapusan
- `GET /penghapusan-bmn/{id}/detail` - Get detailed view

**Workflow:**
- `POST /penghapusan-bmn/{id}/transition` - Transition status
- `POST /penghapusan-bmn/{id}/submit-wilayah` - Submit to wilayah
- `POST /penghapusan-bmn/{id}/validator-wilayah` - Wilayah validator action

**Document Generation:**
- `POST /penghapusan-bmn/{id}/generate-sk` - Generate SK draft
- `POST /penghapusan-bmn/{id}/upload-signed-sk` - Upload signed SK
- `GET /penghapusan-bmn/{id}/document` - Get SK document

### 3.6 Dashboard Endpoints (IMPLEMENTED)

**General:**
- `GET /health` - Health check
- `GET /dashboard/stats` - General dashboard stats

**Perlengkapan Dashboard:**
- `GET /dashboard/perlengkapan` - Perlengkapan metrics
- `GET /dashboard/ws` - WebSocket for real-time updates
- `GET /dashboard/perlengkapan/export/excel` - Export to Excel
- `GET /dashboard/perlengkapan/export/pdf` - Export to PDF

### 3.7 Mapping Kodefikasi Endpoints (READ-ONLY)

**Detection & Analysis:**
- `GET /mapping/detect` - Detect non-standard codes
- `GET /mapping/standard` - List standard codes
- `GET /mapping/suggestions` - Get mapping suggestions
- `GET /mapping/progress` - Get mapping progress
- `GET /mapping/progress/satker` - Get progress by satker
- `GET /mapping/export` - Export mapping data

### 3.8 Roadmap Sarpras Endpoints (BASIC)

**Forecast:**
- `GET /forecast` - Get forecast
- `GET /forecast/summary` - Get forecast summary
- `GET /forecast/compare` - Compare forecasts
- `GET /forecast/export` - Export forecast

### 3.9 Export Endpoints (IMPLEMENTED)

**General Export:**
- `GET /export/excel` - Export to Excel
- `GET /export/jobs/{id}/status` - Get export job status
- `GET /export/jobs/{id}/download` - Download export job

### 3.10 Missing Endpoints (Identified Gaps)

**Not Found in routes.rs:**
1. Master data CRUD endpoints (kode barang, standar spesifikasi, standar jumlah)
2. Notifikasi endpoints (may be in separate notifikasi crate)
3. Dokumen template management endpoints (may be in separate dokumen crate)
4. Workflow configuration endpoints (workflow engine config)
5. Integration sync trigger endpoints (manual sync, sync status)

**Action Required:**
- Check if notifikasi and dokumen crates have separate route definitions
- Verify if master data endpoints exist in a separate module
- Confirm workflow configuration API

**Example - Audit Module:**
```rust
// lib/common/src/audit.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    SecretStored { path: String, version: u32, user: String },
    SecretRetrieved { path: String, version: u32 },
    UserCreated { user_id: Uuid, username: String },
    UserUpdated { user_id: Uuid, changes: Vec<String> },
    WorkflowTransition { entity_id: Uuid, from_state: String, to_state: String },
    BatchOperation { batch_id: Uuid, operation: String, count: usize },
}

pub struct AuditLogger {
    db_pool: deadpool_postgres::Pool,
}

impl AuditLogger {
    pub async fn log(&self, event: AuditEvent, user_id: Uuid, ip_address: String) -> Result<()> {
        let query = r#"
            INSERT INTO audit_log (event_type, event_data, user_id, ip_address, created_at)
            VALUES ($1, $2, $3, $4, NOW())
        "#;

        let event_type = match &event {
            AuditEvent::SecretStored { .. } => "secret_stored",
            AuditEvent::SecretRetrieved { .. } => "secret_retrieved",
            AuditEvent::UserCreated { .. } => "user_created",
            AuditEvent::UserUpdated { .. } => "user_updated",
            AuditEvent::WorkflowTransition { .. } => "workflow_transition",
            AuditEvent::BatchOperation { .. } => "batch_operation",
        };

        let event_data = serde_json::to_value(&event)?;

        self.db_pool.get().await?
            .execute(query, &[&event_type, &event_data, &user_id, &ip_address])
            .await?;

        Ok(())
    }
}
```

**Example - Cache Module:**
```rust
// lib/common/src/cache.rs
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};

pub enum SensitivityLevel {
    Public,      // 1 hour TTL
    Internal,    // 30 minutes TTL
    Confidential, // 5 minutes TTL
}

pub struct CacheManager {
    redis: ConnectionManager,
}

impl CacheManager {
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut self.redis.clone())
            .await?;

        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        sensitivity: SensitivityLevel,
    ) -> Result<()> {
        let ttl = match sensitivity {
            SensitivityLevel::Public => 3600,
            SensitivityLevel::Internal => 1800,
            SensitivityLevel::Confidential => 300,
        };

        let serialized = serde_json::to_string(value)?;

        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl)
            .arg(serialized)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    pub async fn invalidate(&self, pattern: &str) -> Result<()> {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut self.redis.clone())
            .await?;

        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(&keys)
                .query_async(&mut self.redis.clone())
                .await?;
        }

        Ok(())
    }
}
```

### 3.2 lib-perlengkapan Structure

**Location:** `lib/perlengkapan/`

**Modules:**
```rust
// lib/perlengkapan/src/lib.rs
pub mod models;         // Domain models (Kebutuhan, PakaianDinas, Asset, Roadmap, RiwayatPemenuhan)
pub mod gap_analysis;   // Gap analysis algorithm
pub mod prioritization; // Prioritization scoring algorithm
pub mod validation;     // Domain-specific validation
pub mod kode_barang;    // Kode barang utilities
pub mod search;         // Search implementation
pub mod client;         // API client for frontends (no separate client library needed)
```

**Example - Models:**
```rust
// lib/perlengkapan/src/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KebutuhanBmn {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah_kebutuhan: i32,
    pub tahun_anggaran: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PakaianDinas {
    pub id: Uuid,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub jenis_pakaian: String,
    pub ukuran: String,
    pub jumlah: i32,
    pub tahun_anggaran: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapSarpras {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub periode_mulai: i32,
    pub periode_akhir: i32,
    pub kode_barang: String,
    pub tahun_rencana: i32,
    pub jumlah_kebutuhan: i32,
    pub jumlah_terpenuhi: i32,
    pub estimasi_anggaran: Option<f64>,
    pub realisasi_anggaran: Option<f64>,
    pub status_pemenuhan: String,
}
```

**Example - Gap Analysis:**
```rust
// lib/perlengkapan/src/gap_analysis.rs
use crate::models::KebutuhanBmn;

#[derive(Debug, Clone)]
pub struct GapAnalysisResult {
    pub kode_barang: String,
    pub nama_barang: String,
    pub standard_quantity: i32,
    pub existing_good_quantity: i32,
    pub gap: i32,
}

pub struct GapAnalyzer {
    cache: lib_common::cache::CacheManager,
}

impl GapAnalyzer {
    pub async fn calculate_gap(
        &self,
        satker_id: Uuid,
        kode_barang: &str,
        standard_quantity: i32,
        siman_client: &SimanGrpcClient,
    ) -> Result<GapAnalysisResult> {
        // Check cache first
        let cache_key = format!("gap:{}:{}", satker_id, kode_barang);
        if let Some(cached) = self.cache.get::<GapAnalysisResult>(&cache_key).await? {
            return Ok(cached);
        }

        // Fetch existing assets from SIMAN via gRPC
        let existing_assets = siman_client
            .get_assets_by_kode(GetAssetsByKodeRequest {
                satker_id: satker_id.to_string(),
                kode_barang: kode_barang.to_string(),
            })
            .await?
            .into_inner()
            .assets;

        // Count only assets in good condition
        let existing_good_quantity = existing_assets
            .iter()
            .filter(|a| a.kondisi == "BAIK")
            .count() as i32;

        let gap = standard_quantity - existing_good_quantity;

        let result = GapAnalysisResult {
            kode_barang: kode_barang.to_string(),
            nama_barang: "".to_string(), // Fetch from master data
            standard_quantity,
            existing_good_quantity,
            gap,
        };

        // Cache for 1 hour
        self.cache.set(&cache_key, &result, lib_common::cache::SensitivityLevel::Internal).await?;

        Ok(result)
    }
}
```

**Example - Prioritization:**
```rust
// lib/perlengkapan/src/prioritization.rs

#[derive(Debug, Clone)]
pub struct PriorityScore {
    pub kebutuhan_id: Uuid,
    pub score: f64,
    pub breakdown: ScoreBreakdown,
}

#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub gap_magnitude_score: f64,      // 40%
    pub asset_criticality_score: f64,  // 30%
    pub satker_type_score: f64,        // 20%
    pub justification_score: f64,      // 10%
}

pub struct PrioritizationEngine;

impl PrioritizationEngine {
    pub fn calculate_priority(
        &self,
        kebutuhan: &KebutuhanBmn,
        gap: i32,
        is_critical_infrastructure: bool,
        satker_type: &str,
        justification_length: usize,
    ) -> PriorityScore {
        // Gap magnitude (40%) - linear scale
        let gap_magnitude_score = (gap as f64 / 100.0).min(1.0) * 40.0;

        // Asset criticality (30%)
        let asset_criticality_score = if is_critical_infrastructure { 30.0 } else { 0.0 };

        // Satker type (20%)
        let satker_type_score = match satker_type {
            "Cabjari" => 20.0,
            "Kejari_C" => 20.0,
            "Kejari_B" => 15.0,
            "Kejari_A" => 10.0,
            "Kejati" => 5.0,
            _ => 0.0,
        };

        // Justification quality (10%)
        let justification_score = if justification_length > 200 { 10.0 } else { 0.0 };

        let total_score = gap_magnitude_score + asset_criticality_score + satker_type_score + justification_score;

        PriorityScore {
            kebutuhan_id: kebutuhan.id,
            score: total_score,
            breakdown: ScoreBreakdown {
                gap_magnitude_score,
                asset_criticality_score,
                satker_type_score,
                justification_score,
            },
        }
    }
}
```

### 3.3 lib-ui Structure

**Location:** `lib/ui/`

**Components:**
```rust
// lib/ui/src/lib.rs
pub mod components;     // Reusable UI components
pub mod forms;          // Form components
pub mod tables;         // Table components
pub mod charts;         // Chart components
pub mod modals;         // Modal components
pub mod dashboard;      // Dashboard widgets
pub mod layouts;        // Layout components
```

**Example - Dashboard Widget:**
```rust
// lib/ui/src/dashboard/metric_card.rs
use leptos::prelude::*;

#[component]
pub fn MetricCard(
    title: String,
    value: String,
    change: Option<f64>,
    icon: String,
) -> impl IntoView {
    let change_class = move || {
        change.map(|c| if c >= 0.0 { "text-green-600" } else { "text-red-600" })
    };

    view! {
        <div class="bg-white rounded-lg shadow p-6">
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm text-gray-600">{title}</p>
                    <p class="text-2xl font-bold mt-2">{value}</p>
                    {change.map(|c| view! {
                        <p class={change_class()}>
                            {if c >= 0.0 { "↑" } else { "↓" }}
                            {format!("{:.1}%", c.abs())}
                        </p>
                    })}
                </div>
                <div class="text-4xl">{icon}</div>
            </div>
        </div>
    }
}
```

## 4. Integration Services Design

### 4.1 Integration Service Architecture

**Location:** `layanan/perlengkapan/crates/integrasi/`

**Structure:**
```
layanan/perlengkapan/crates/integrasi/
├── Cargo.toml
├── src/
│   ├── main.rs              # Service entry point
│   ├── lib.rs               # Module exports
│   ├── config.rs            # Configuration
│   ├── clients/             # API clients
│   │   ├── siman.rs         # SIMAN API client
│   │   ├── mysimkari.rs     # MySIMKARI API client
│   │   └── monsakti.rs      # MonSAKTI API client (optional)
│   ├── sync/                # Sync logic
│   │   ├── scheduler.rs     # Cron scheduler
│   │   ├── siman_sync.rs    # SIMAN sync
│   │   └── mysimkari_sync.rs # MySIMKARI sync
│   ├── grpc/                # gRPC service
│   │   └── service.rs       # Integration gRPC API
│   └── error.rs             # Error types
└── proto/
    └── integrasi.proto      # gRPC proto
```

### 4.2 SIMAN Integration

**API Client:**
```rust
// layanan/perlengkapan/crates/integrasi/src/clients/siman.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SimanClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SimanAsset {
    pub nup: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub kondisi: String,
    pub tahun_perolehan: i32,
    pub nilai_perolehan: f64,
}

impl SimanClient {
    pub async fn new(secreton_client: &SecretonGrpcClient) -> Result<Self> {
        // Fetch API credentials from Secreton
        let api_key = secreton_client
            .get_secret(GetSecretRequest {
                path: "integration/siman/api_key".to_string(),
                version: None,
            })
            .await?
            .into_inner()
            .data;

        let api_key = String::from_utf8(api_key)?;

        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            base_url: "https://siman.djkn.kemenkeu.go.id/api/v2".to_string(),
            api_key,
        })
    }

    pub async fn get_assets_by_satker(
        &self,
        satker_code: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<SimanAsset>> {
        let response = self.client
            .get(&format!("{}/assets", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("satker_code", satker_code),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(IntegrationError::ApiError(format!(
                "SIMAN API returned status: {}",
                response.status()
            )));
        }

        let assets: Vec<SimanAsset> = response.json().await?;
        Ok(assets)
    }

    pub async fn get_assets_incremental(
        &self,
        satker_code: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<SimanAsset>> {
        let response = self.client
            .get(&format!("{}/assets/incremental", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("satker_code", satker_code),
                ("since", &since.to_rfc3339()),
            ])
            .send()
            .await?;

        let assets: Vec<SimanAsset> = response.json().await?;
        Ok(assets)
    }
}
```

**Sync Logic:**
```rust
// layanan/perlengkapan/crates/integrasi/src/sync/siman_sync.rs
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct SimanSyncService {
    client: SimanClient,
    db_pool: deadpool_postgres::Pool,
    audit_logger: lib_common::audit::AuditLogger,
}

impl SimanSyncService {
    pub async fn start_scheduler(&self) -> Result<()> {
        let scheduler = JobScheduler::new().await?;

        // Full sync daily at 02:00 WIB
        let full_sync_job = Job::new_async("0 0 2 * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = self.full_sync().await {
                    tracing::error!("Full sync failed: {}", e);
                }
            })
        })?;

        // Incremental sync every 6 hours
        let incremental_sync_job = Job::new_async("0 0 */6 * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = self.incremental_sync().await {
                    tracing::error!("Incremental sync failed: {}", e);
                }
            })
        })?;

        scheduler.add(full_sync_job).await?;
        scheduler.add(incremental_sync_job).await?;
        scheduler.start().await?;

        Ok(())
    }

    pub async fn full_sync(&self) -> Result<()> {
        tracing::info!("Starting SIMAN full sync");

        // Get all satkers
        let satkers = self.get_all_satkers().await?;

        for satker in satkers {
            let mut page = 1;
            let per_page = 100;

            loop {
                // Fetch assets with retry logic
                let assets = self.retry_with_backoff(|| async {
                    self.client.get_assets_by_satker(&satker.code, page, per_page).await
                }).await?;

                if assets.is_empty() {
                    break;
                }

                // Store in database
                self.store_assets(&assets).await?;

                // Log API call
                self.log_api_call("SIMAN", "get_assets_by_satker", 200, None).await?;

                page += 1;
            }
        }

        tracing::info!("SIMAN full sync completed");
        Ok(())
    }

    pub async fn incremental_sync(&self) -> Result<()> {
        tracing::info!("Starting SIMAN incremental sync");

        // Get last sync time
        let last_sync = self.get_last_sync_time("siman").await?;

        let satkers = self.get_all_satkers().await?;

        for satker in satkers {
            let assets = self.retry_with_backoff(|| async {
                self.client.get_assets_incremental(&satker.code, last_sync).await
            }).await?;

            if !assets.is_empty() {
                self.store_assets(&assets).await?;
            }
        }

        // Update last sync time
        self.update_last_sync_time("siman").await?;

        tracing::info!("SIMAN incremental sync completed");
        Ok(())
    }

    async fn retry_with_backoff<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        let max_retries = 5;

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    retries += 1;
                    let backoff = Duration::from_secs(2u64.pow(retries));
                    tracing::warn!("Retry {} after {:?}: {}", retries, backoff, e);
                    tokio::time::sleep(backoff).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn store_assets(&self, assets: &[SimanAsset]) -> Result<()> {
        let client = self.db_pool.get().await?;
        let tx = client.transaction().await?;

        for asset in assets {
            let query = r#"
                INSERT INTO integrasi.siman_aset_tanah
                (nup, kode_barang, nama_barang, kondisi, raw_data, synced_at)
                VALUES ($1, $2, $3, $4, $5, NOW())
                ON CONFLICT (nup) DO UPDATE SET
                    kode_barang = EXCLUDED.kode_barang,
                    nama_barang = EXCLUDED.nama_barang,
                    kondisi = EXCLUDED.kondisi,
                    raw_data = EXCLUDED.raw_data,
                    synced_at = NOW()
            "#;

            let raw_data = serde_json::to_value(asset)?;

            tx.execute(
                query,
                &[&asset.nup, &asset.kode_barang, &asset.nama_barang, &asset.kondisi, &raw_data],
            ).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
```

### 4.3 MySIMKARI Integration

**API Client:**
```rust
// layanan/perlengkapan/crates/integrasi/src/clients/mysimkari.rs

#[derive(Debug, Clone)]
pub struct MySIMKARIClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct MySIMKARIPegawai {
    pub nip: String,
    pub nama: String,
    pub satker_code: String,
    pub jabatan: String,
    pub golongan: String,
    pub status_pegawai: String,
}

impl MySIMKARIClient {
    pub async fn new(secreton_client: &SecretonGrpcClient) -> Result<Self> {
        let api_key = secreton_client
            .get_secret(GetSecretRequest {
                path: "integration/mysimkari/api_key".to_string(),
                version: None,
            })
            .await?
            .into_inner()
            .data;

        let api_key = String::from_utf8(api_key)?;

        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            base_url: "https://mysimkari.kejaksaan.go.id/api/v1".to_string(),
            api_key,
        })
    }

    pub async fn get_pegawai_by_satker(
        &self,
        satker_code: &str,
    ) -> Result<Vec<MySIMKARIPegawai>> {
        let response = self.client
            .get(&format!("{}/pegawai", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[("satker_code", satker_code)])
            .send()
            .await?;

        let pegawai: Vec<MySIMKARIPegawai> = response.json().await?;
        Ok(pegawai)
    }
}
```

**Sync Logic:**
```rust
// layanan/perlengkapan/crates/integrasi/src/sync/mysimkari_sync.rs

pub struct MySIMKARISyncService {
    client: MySIMKARIClient,
    db_pool: deadpool_postgres::Pool,
}

impl MySIMKARISyncService {
    pub async fn start_scheduler(&self) -> Result<()> {
        let scheduler = JobScheduler::new().await?;

        // Full sync daily at 03:00 WIB
        let full_sync_job = Job::new_async("0 0 3 * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = self.full_sync().await {
                    tracing::error!("MySIMKARI full sync failed: {}", e);
                }
            })
        })?;

        // Incremental sync every 4 hours
        let incremental_sync_job = Job::new_async("0 0 */4 * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = self.incremental_sync().await {
                    tracing::error!("MySIMKARI incremental sync failed: {}", e);
                }
            })
        })?;

        scheduler.add(full_sync_job).await?;
        scheduler.add(incremental_sync_job).await?;
        scheduler.start().await?;

        Ok(())
    }

    pub async fn full_sync(&self) -> Result<()> {
        tracing::info!("Starting MySIMKARI full sync");

        let satkers = self.get_all_satkers().await?;

        for satker in satkers {
            let pegawai = self.client.get_pegawai_by_satker(&satker.code).await?;
            self.store_pegawai(&pegawai).await?;
        }

        tracing::info!("MySIMKARI full sync completed");
        Ok(())
    }
}
```

### 4.4 Integration gRPC Service

**Proto Definition:**
```protobuf
// layanan/perlengkapan/crates/integrasi/proto/integrasi.proto
syntax = "proto3";

package integrasi;

service IntegrationService {
    rpc GetSimanAssets(GetSimanAssetsRequest) returns (GetSimanAssetsResponse);
    rpc GetMySIMKARIPegawai(GetMySIMKARIPegawaiRequest) returns (GetMySIMKARIPegawaiResponse);
    rpc GetSyncStatus(GetSyncStatusRequest) returns (GetSyncStatusResponse);
    rpc TriggerSync(TriggerSyncRequest) returns (TriggerSyncResponse);
}

message GetSimanAssetsRequest {
    string satker_id = 1;
    string kode_barang = 2;
}

message GetSimanAssetsResponse {
    repeated SimanAsset assets = 1;
}

message SimanAsset {
    string nup = 1;
    string kode_barang = 2;
    string nama_barang = 3;
    string kondisi = 4;
    int32 tahun_perolehan = 5;
    double nilai_perolehan = 6;
}

message GetSyncStatusRequest {
    string service_name = 1; // "siman" or "mysimkari"
}

message GetSyncStatusResponse {
    string service_name = 1;
    string last_sync_time = 2;
    string status = 3; // "healthy", "error"
    string error_message = 4;
}
```

**gRPC Implementation:**
```rust
// layanan/perlengkapan/crates/integrasi/src/grpc/service.rs
use tonic::{Request, Response, Status};

pub mod integrasi_proto {
    tonic::include_proto!("integrasi");
}

use integrasi_proto::integration_service_server::{IntegrationService, IntegrationServiceServer};

#[derive(Debug)]
pub struct IntegrationServiceImpl {
    db_pool: deadpool_postgres::Pool,
}

#[tonic::async_trait]
impl IntegrationService for IntegrationServiceImpl {
    async fn get_siman_assets(
        &self,
        request: Request<GetSimanAssetsRequest>,
    ) -> Result<Response<GetSimanAssetsResponse>, Status> {
        let req = request.into_inner();

        let query = r#"
            SELECT nup, kode_barang, nama_barang, kondisi,
                   (raw_data->>'tahun_perolehan')::int as tahun_perolehan,
                   (raw_data->>'nilai_perolehan')::float as nilai_perolehan
            FROM integrasi.siman_aset_tanah
            WHERE satker_id = $1 AND kode_barang = $2
        "#;

        let client = self.db_pool.get().await
            .map_err(|e| Status::internal(e.to_string()))?;

        let rows = client.query(query, &[&req.satker_id, &req.kode_barang]).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let assets = rows.into_iter().map(|row| SimanAsset {
            nup: row.get("nup"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            kondisi: row.get("kondisi"),
            tahun_perolehan: row.get("tahun_perolehan"),
            nilai_perolehan: row.get("nilai_perolehan"),
        }).collect();

        Ok(Response::new(GetSimanAssetsResponse { assets }))
    }

    async fn get_sync_status(
        &self,
        request: Request<GetSyncStatusRequest>,
    ) -> Result<Response<GetSyncStatusResponse>, Status> {
        let req = request.into_inner();

        let query = r#"
            SELECT last_sync_time, status, error_message
            FROM integrasi.sync_status
            WHERE service_name = $1
        "#;

        let client = self.db_pool.get().await
            .map_err(|e| Status::internal(e.to_string()))?;

        let row = client.query_one(query, &[&req.service_name]).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetSyncStatusResponse {
            service_name: req.service_name,
            last_sync_time: row.get::<_, DateTime<Utc>>("last_sync_time").to_rfc3339(),
            status: row.get("status"),
            error_message: row.get("error_message"),
        }))
    }
}
```

## 5. Workflow Engine Design

### 5.1 Workflow Configuration

**Location:** `layanan/perlengkapan/crates/api/src/workflow/`

**State Transition Rules:**
```rust
// layanan/perlengkapan/crates/api/src/workflow/config.rs
use std::collections::HashMap;

pub struct WorkflowConfig {
    pub transitions: HashMap<String, Vec<String>>,
    pub sla_minutes: HashMap<String, u32>,
}

impl WorkflowConfig {
    pub fn default_kebutuhan_bmn() -> Self {
        let mut transitions = HashMap::new();

        // DRAFT can transition to INPUT_BARANG or CANCELLED
        transitions.insert("DRAFT".to_string(), vec![
            "INPUT_BARANG".to_string(),
            "CANCELLED".to_string(),
        ]);

        // INPUT_BARANG can transition to SUBMITTED or DRAFT
        transitions.insert("INPUT_BARANG".to_string(), vec![
            "SUBMITTED".to_string(),
            "DRAFT".to_string(),
        ]);

        // SUBMITTED can transition to REVIEWED or REJECTED
        transitions.insert("SUBMITTED".to_string(), vec![
            "REVIEWED".to_string(),
            "REJECTED".to_string(),
        ]);

        // REVIEWED can transition to APPROVED, REVISION_REQUIRED, or REJECTED
        transitions.insert("REVIEWED".to_string(), vec![
            "APPROVED".to_string(),
            "REVISION_REQUIRED".to_string(),
            "REJECTED".to_string(),
        ]);

        // REVISION_REQUIRED can transition back to SUBMITTED
        transitions.insert("REVISION_REQUIRED".to_string(), vec![
            "SUBMITTED".to_string(),
        ]);

        // APPROVED can transition to COMPLETED
        transitions.insert("APPROVED".to_string(), vec![
            "COMPLETED".to_string(),
        ]);

        // COMPLETED can transition to ARCHIVED
        transitions.insert("COMPLETED".to_string(), vec![
            "ARCHIVED".to_string(),
        ]);

        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMITTED".to_string(), 2880);  // 2 days
        sla_minutes.insert("REVIEWED".to_string(), 1440);   // 1 day
        sla_minutes.insert("APPROVED".to_string(), 4320);   // 3 days

        Self { transitions, sla_minutes }
    }
}
```

### 5.2 Workflow Engine Implementation

```rust
// layanan/perlengkapan/crates/api/src/workflow/engine.rs
use lib_common::audit::AuditLogger;
use lib_common::notification::NotificationService;

pub struct WorkflowEngine {
    config: WorkflowConfig,
    db_pool: deadpool_postgres::Pool,
    audit_logger: AuditLogger,
    notification_service: NotificationService,
    authenc_client: AuthencGrpcClient,
}

impl WorkflowEngine {
    pub async fn transition(
        &self,
        entity_id: Uuid,
        from_state: &str,
        to_state: &str,
        user_id: Uuid,
        catatan: Option<String>,
    ) -> Result<()> {
        // 1. Validate transition
        self.validate_transition(from_state, to_state)?;

        // 2. Validate approver role
        self.validate_approver_role(user_id, to_state).await?;

        // 3. Update entity state
        let client = self.db_pool.get().await?;
        let tx = client.transaction().await?;

        let update_query = r#"
            UPDATE perlengkapan.kebutuhan_bmn
            SET status = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        tx.execute(update_query, &[&to_state, &entity_id]).await?;

        // 4. Record transition in aktivitas table
        let aktivitas_id = self.get_aktivitas_id(to_state).await?;

        let insert_query = r#"
            INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
            (pengajuan_id, aktivitas_id, user_id, catatan, created_at)
            VALUES ($1, $2, $3, $4, NOW())
        "#;

        tx.execute(insert_query, &[&entity_id, &aktivitas_id, &user_id, &catatan]).await?;

        tx.commit().await?;

        // 5. Audit log
        self.audit_logger.log(
            lib_common::audit::AuditEvent::WorkflowTransition {
                entity_id,
                from_state: from_state.to_string(),
                to_state: to_state.to_string(),
            },
            user_id,
            "".to_string(), // IP address
        ).await?;

        // 6. Send notification
        self.notification_service.send_workflow_notification(
            entity_id,
            to_state,
            user_id,
        ).await?;

        // 7. Check SLA
        self.check_sla(entity_id, to_state).await?;

        Ok(())
    }

    fn validate_transition(&self, from_state: &str, to_state: &str) -> Result<()> {
        let allowed_transitions = self.config.transitions.get(from_state)
            .ok_or_else(|| WorkflowError::InvalidState(from_state.to_string()))?;

        if !allowed_transitions.contains(&to_state.to_string()) {
            return Err(WorkflowError::InvalidTransition {
                from: from_state.to_string(),
                to: to_state.to_string(),
            });
        }

        Ok(())
    }

    async fn validate_approver_role(&self, user_id: Uuid, to_state: &str) -> Result<()> {
        // Call Authenc to get user roles
        let response = self.authenc_client
            .get_user_roles(GetUserRolesRequest {
                user_id: user_id.to_string(),
            })
            .await?;

        let roles: Vec<String> = response.into_inner().roles;

        // Check if user has required role for this transition
        let required_role = match to_state {
            "REVIEWED" => "operator",
            "APPROVED" => "admin_pusat",
            "REJECTED" => "admin_pusat",
            _ => return Ok(()), // No role check for other states
        };

        if !roles.contains(&required_role.to_string()) {
            return Err(WorkflowError::InsufficientPermissions {
                required_role: required_role.to_string(),
            });
        }

        Ok(())
    }

    async fn check_sla(&self, entity_id: Uuid, current_state: &str) -> Result<()> {
        if let Some(sla_minutes) = self.config.sla_minutes.get(current_state) {
            let query = r#"
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = $1 AND aktivitas_id = (
                    SELECT id FROM perlengkapan.ms_aktivitas_bmn WHERE kode = $2
                )
                ORDER BY created_at DESC
                LIMIT 1
            "#;

            let client = self.db_pool.get().await?;
            let row = client.query_one(query, &[&entity_id, &current_state]).await?;
            let created_at: DateTime<Utc> = row.get("created_at");

            let elapsed = Utc::now().signed_duration_since(created_at);
            let sla_duration = chrono::Duration::minutes(*sla_minutes as i64);

            if elapsed > sla_duration {
                // SLA breached - send escalation notification
                self.notification_service.send_sla_breach_notification(
                    entity_id,
                    current_state,
                    elapsed.num_minutes(),
                ).await?;
            }
        }

        Ok(())
    }
}
```

### 5.3 Parallel Approvals

```rust
// layanan/perlengkapan/crates/api/src/workflow/parallel.rs

pub struct ParallelApprovalEngine {
    workflow_engine: WorkflowEngine,
    db_pool: deadpool_postgres::Pool,
}

impl ParallelApprovalEngine {
    pub async fn create_parallel_approval(
        &self,
        entity_id: Uuid,
        approvers: Vec<Uuid>,
        required_approvals: usize,
    ) -> Result<Uuid> {
        let approval_id = Uuid::new_v4();

        let query = r#"
            INSERT INTO perlengkapan.parallel_approvals
            (id, entity_id, approvers, required_approvals, current_approvals, status, created_at)
            VALUES ($1, $2, $3, $4, 0, 'PENDING', NOW())
        "#;

        self.db_pool.get().await?
            .execute(query, &[&approval_id, &entity_id, &approvers, &(required_approvals as i32)])
            .await?;

        Ok(approval_id)
    }

    pub async fn record_approval(
        &self,
        approval_id: Uuid,
        approver_id: Uuid,
        approved: bool,
        catatan: Option<String>,
    ) -> Result<()> {
        let query = r#"
            INSERT INTO perlengkapan.parallel_approval_votes
            (approval_id, approver_id, approved, catatan, created_at)
            VALUES ($1, $2, $3, $4, NOW())
        "#;

        self.db_pool.get().await?
            .execute(query, &[&approval_id, &approver_id, &approved, &catatan])
            .await?;

        // Check if threshold reached
        self.check_approval_threshold(approval_id).await?;

        Ok(())
    }

    async fn check_approval_threshold(&self, approval_id: Uuid) -> Result<()> {
        let query = r#"
            SELECT entity_id, required_approvals,
                   (SELECT COUNT(*) FROM perlengkapan.parallel_approval_votes
                    WHERE approval_id = $1 AND approved = true) as current_approvals
            FROM perlengkapan.parallel_approvals
            WHERE id = $1
        "#;

        let client = self.db_pool.get().await?;
        let row = client.query_one(query, &[&approval_id]).await?;

        let entity_id: Uuid = row.get("entity_id");
        let required_approvals: i32 = row.get("required_approvals");
        let current_approvals: i64 = row.get("current_approvals");

        if current_approvals >= required_approvals as i64 {
            // Threshold reached - transition to next state
            self.workflow_engine.transition(
                entity_id,
                "REVIEWED",
                "APPROVED",
                Uuid::nil(), // System user
                Some("Parallel approval threshold reached".to_string()),
            ).await?;

            // Update parallel approval status
            let update_query = r#"
                UPDATE perlengkapan.parallel_approvals
                SET status = 'APPROVED', updated_at = NOW()
                WHERE id = $1
            "#;

            client.execute(update_query, &[&approval_id]).await?;
        }

        Ok(())
    }
}
```

## 6. Document Generation Service

### 6.1 Service Architecture

**Location:** `layanan/perlengkapan/crates/dokumen/`

**Structure:**
```
layanan/perlengkapan/crates/dokumen/
├── Cargo.toml
├── src/
│   ├── main.rs              # Service entry point
│   ├── lib.rs               # Module exports
│   ├── config.rs            # Configuration
│   ├── templates/           # Template management
│   │   ├── engine.rs        # Tera template engine
│   │   └── loader.rs        # Template loader
│   ├── generators/          # Document generators
│   │   ├── pdf.rs           # PDF generation
│   │   └── excel.rs         # Excel generation
│   ├── storage.rs           # MinIO/S3 storage
│   ├── grpc/                # gRPC service
│   │   └── service.rs       # Document gRPC API
│   └── error.rs             # Error types
└── templates/               # Template files
    ├── rekapitulasi_kebutuhan.html
    ├── surat_izin.html
    └── roadmap_report.html
```

### 6.2 Template Engine

```rust
// layanan/perlengkapan/crates/dokumen/src/templates/engine.rs
use tera::{Tera, Context};

pub struct TemplateEngine {
    tera: Tera,
    db_pool: deadpool_postgres::Pool,
}

impl TemplateEngine {
    pub async fn new(db_pool: deadpool_postgres::Pool) -> Result<Self> {
        // Load templates from database
        let templates = Self::load_templates_from_db(&db_pool).await?;

        let mut tera = Tera::default();

        for (name, content) in templates {
            tera.add_raw_template(&name, &content)?;
        }

        Ok(Self { tera, db_pool })
    }

    async fn load_templates_from_db(
        db_pool: &deadpool_postgres::Pool,
    ) -> Result<Vec<(String, String)>> {
        let query = r#"
            SELECT name, content
            FROM dokumen.templates
            WHERE enabled = true
        "#;

        let client = db_pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let templates = rows.into_iter()
            .map(|row| (row.get("name"), row.get("content")))
            .collect();

        Ok(templates)
    }

    pub fn render(&self, template_name: &str, context: &Context) -> Result<String> {
        let rendered = self.tera.render(template_name, context)?;
        Ok(rendered)
    }
}
```

### 6.3 PDF Generator

```rust
// layanan/perlengkapan/crates/dokumen/src/generators/pdf.rs
use headless_chrome::{Browser, LaunchOptions};

pub struct PdfGenerator {
    template_engine: TemplateEngine,
}

impl PdfGenerator {
    pub async fn generate(
        &self,
        template_name: &str,
        data: serde_json::Value,
    ) -> Result<Vec<u8>> {
        // 1. Render HTML from template
        let mut context = Context::new();
        context.insert("data", &data);

        let html = self.template_engine.render(template_name, &context)?;

        // 2. Convert HTML to PDF using headless Chrome
        let browser = Browser::new(LaunchOptions::default())?;
        let tab = browser.new_tab()?;

        tab.navigate_to(&format!("data:text/html,{}", urlencoding::encode(&html)))?;
        tab.wait_until_navigated()?;

        let pdf_data = tab.print_to_pdf(None)?;

        Ok(pdf_data)
    }
}
```

### 6.4 Excel Generator

```rust
// layanan/perlengkapan/crates/dokumen/src/generators/excel.rs
use rust_xlsxwriter::{Workbook, Worksheet, Format};

pub struct ExcelGenerator;

impl ExcelGenerator {
    pub fn generate_rekapitulasi_kebutuhan(
        &self,
        data: Vec<KebutuhanBmn>,
    ) -> Result<Vec<u8>> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Header format
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF");

        // Write headers
        worksheet.write_string_with_format(0, 0, "No", &header_format)?;
        worksheet.write_string_with_format(0, 1, "Satker", &header_format)?;
        worksheet.write_string_with_format(0, 2, "Kode Barang", &header_format)?;
        worksheet.write_string_with_format(0, 3, "Nama Barang", &header_format)?;
        worksheet.write_string_with_format(0, 4, "Jumlah Kebutuhan", &header_format)?;
        worksheet.write_string_with_format(0, 5, "Tahun Anggaran", &header_format)?;
        worksheet.write_string_with_format(0, 6, "Status", &header_format)?;

        // Write data
        for (idx, kebutuhan) in data.iter().enumerate() {
            let row = (idx + 1) as u32;
            worksheet.write_number(row, 0, (idx + 1) as f64)?;
            worksheet.write_string(row, 1, &kebutuhan.satker_nama)?;
            worksheet.write_string(row, 2, &kebutuhan.kode_barang)?;
            worksheet.write_string(row, 3, &kebutuhan.nama_barang)?;
            worksheet.write_number(row, 4, kebutuhan.jumlah_kebutuhan as f64)?;
            worksheet.write_number(row, 5, kebutuhan.tahun_anggaran as f64)?;
            worksheet.write_string(row, 6, &kebutuhan.status)?;
        }

        // Auto-fit columns
        worksheet.autofit();

        // Save to buffer
        let mut buffer = Vec::new();
        workbook.save_to_buffer(&mut buffer)?;

        Ok(buffer)
    }
}
```

### 6.5 Storage Integration

```rust
// layanan/perlengkapan/crates/dokumen/src/storage.rs
use lib_common::storage::S3Client;
use sha2::{Sha256, Digest};

pub struct DocumentStorage {
    s3_client: S3Client,
    db_pool: deadpool_postgres::Pool,
}

impl DocumentStorage {
    pub async fn store_document(
        &self,
        document_type: &str,
        filename: &str,
        content: Vec<u8>,
        metadata: DocumentMetadata,
    ) -> Result<Uuid> {
        let document_id = Uuid::new_v4();

        // 1. Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let checksum = format!("{:x}", hasher.finalize());

        // 2. Upload to MinIO
        let s3_key = format!("documents/{}/{}/{}", document_type, document_id, filename);
        self.s3_client.put_object(&s3_key, content).await?;

        // 3. Store metadata in database
        let query = r#"
            INSERT INTO dokumen.documents
            (id, document_type, filename, s3_key, checksum, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#;

        let metadata_json = serde_json::to_value(&metadata)?;

        self.db_pool.get().await?
            .execute(query, &[
                &document_id,
                &document_type,
                &filename,
                &s3_key,
                &checksum,
                &metadata_json,
            ])
            .await?;

        Ok(document_id)
    }

    pub async fn get_document(&self, document_id: Uuid) -> Result<Vec<u8>> {
        // 1. Get S3 key from database
        let query = r#"
            SELECT s3_key
            FROM dokumen.documents
            WHERE id = $1
        "#;

        let client = self.db_pool.get().await?;
        let row = client.query_one(query, &[&document_id]).await?;
        let s3_key: String = row.get("s3_key");

        // 2. Download from MinIO
        let content = self.s3_client.get_object(&s3_key).await?;

        Ok(content)
    }
}
```

### 6.6 gRPC Service

```rust
// layanan/perlengkapan/crates/dokumen/src/grpc/service.rs
use tonic::{Request, Response, Status};

pub mod dokumen_proto {
    tonic::include_proto!("dokumen");
}

use dokumen_proto::document_service_server::{DocumentService, DocumentServiceServer};

#[derive(Debug)]
pub struct DocumentServiceImpl {
    pdf_generator: PdfGenerator,
    excel_generator: ExcelGenerator,
    storage: DocumentStorage,
    notification_service: NotificationService,
}

#[tonic::async_trait]
impl DocumentService for DocumentServiceImpl {
    async fn generate_document(
        &self,
        request: Request<GenerateDocumentRequest>,
    ) -> Result<Response<GenerateDocumentResponse>, Status> {
        let req = request.into_inner();

        // Generate document based on type
        let content = match req.document_type.as_str() {
            "rekapitulasi_kebutuhan_pdf" => {
                let data = serde_json::from_str(&req.data)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?;
                self.pdf_generator.generate("rekapitulasi_kebutuhan", data).await
                    .map_err(|e| Status::internal(e.to_string()))?
            }
            "rekapitulasi_kebutuhan_excel" => {
                let data: Vec<KebutuhanBmn> = serde_json::from_str(&req.data)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?;
                self.excel_generator.generate_rekapitulasi_kebutuhan(data)
                    .map_err(|e| Status::internal(e.to_string()))?
            }
            _ => return Err(Status::invalid_argument("Unknown document type")),
        };

        // Store document
        let document_id = self.storage.store_document(
            &req.document_type,
            &req.filename,
            content,
            DocumentMetadata {
                created_by: req.user_id,
                entity_id: req.entity_id,
            },
        ).await.map_err(|e| Status::internal(e.to_string()))?;

        // Send notification
        self.notification_service.send_document_ready_notification(
            req.user_id,
            document_id,
        ).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GenerateDocumentResponse {
            document_id: document_id.to_string(),
            status: "completed".to_string(),
        }))
    }
}
```

## 7. Notification Service

### 7.1 Service Architecture

**Location:** `layanan/perlengkapan/crates/notifikasi/`

**Structure:**
```
layanan/perlengkapan/crates/notifikasi/
├── Cargo.toml
├── src/
│   ├── main.rs              # Service entry point
│   ├── lib.rs               # Module exports
│   ├── config.rs            # Configuration
│   ├── channels/            # Delivery channels
│   │   ├── in_app.rs        # In-app notifications
│   │   ├── email.rs         # Email notifications
│   │   └── sms.rs           # SMS notifications (optional)
│   ├── templates/           # Notification templates
│   │   └── engine.rs        # Template engine
│   ├── scheduler.rs         # Batch notification scheduler
│   ├── grpc/                # gRPC service
│   │   └── service.rs       # Notification gRPC API
│   └── error.rs             # Error types
```

### 7.2 Notification Types

```rust
// layanan/perlengkapan/crates/notifikasi/src/lib.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    WorkflowStateChange {
        entity_id: Uuid,
        entity_type: String,
        from_state: String,
        to_state: String,
    },
    DocumentReady {
        document_id: Uuid,
        document_type: String,
    },
    SLABreach {
        entity_id: Uuid,
        current_state: String,
        elapsed_minutes: i64,
    },
    IzinExpiry {
        izin_id: Uuid,
        days_until_expiry: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,      // Batch daily digest
    Normal,   // Send immediately
    High,     // Send immediately + SMS
    Urgent,   // Send immediately + SMS + Email
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub batch_non_urgent: bool,
}
```

### 7.3 In-App Notifications

```rust
// layanan/perlengkapan/crates/notifikasi/src/channels/in_app.rs

pub struct InAppNotificationChannel {
    db_pool: deadpool_postgres::Pool,
}

impl InAppNotificationChannel {
    pub async fn send(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
        priority: NotificationPriority,
    ) -> Result<Uuid> {
        let notification_id = Uuid::new_v4();

        let (title, message) = self.format_notification(&notification_type);

        let query = r#"
            INSERT INTO notifikasi.notifications
            (id, user_id, notification_type, title, message, priority, read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, false, NOW())
        "#;

        let notification_type_str = match notification_type {
            NotificationType::WorkflowStateChange { .. } => "workflow_state_change",
            NotificationType::DocumentReady { .. } => "document_ready",
            NotificationType::SLABreach { .. } => "sla_breach",
            NotificationType::IzinExpiry { .. } => "izin_expiry",
        };

        let priority_str = match priority {
            NotificationPriority::Low => "low",
            NotificationPriority::Normal => "normal",
            NotificationPriority::High => "high",
            NotificationPriority::Urgent => "urgent",
        };

        self.db_pool.get().await?
            .execute(query, &[
                &notification_id,
                &user_id,
                &notification_type_str,
                &title,
                &message,
                &priority_str,
            ])
            .await?;

        Ok(notification_id)
    }

    fn format_notification(&self, notification_type: &NotificationType) -> (String, String) {
        match notification_type {
            NotificationType::WorkflowStateChange { entity_type, to_state, .. } => {
                let title = format!("{} Status Changed", entity_type);
                let message = format!("Status changed to: {}", to_state);
                (title, message)
            }
            NotificationType::DocumentReady { document_type, .. } => {
                let title = "Document Ready".to_string();
                let message = format!("Your {} is ready for download", document_type);
                (title, message)
            }
            NotificationType::SLABreach { current_state, elapsed_minutes, .. } => {
                let title = "SLA Breach".to_string();
                let message = format!(
                    "Item in {} state has exceeded SLA by {} minutes",
                    current_state, elapsed_minutes
                );
                (title, message)
            }
            NotificationType::IzinExpiry { days_until_expiry, .. } => {
                let title = "Izin Expiring Soon".to_string();
                let message = format!("Izin will expire in {} days", days_until_expiry);
                (title, message)
            }
        }
    }

    pub async fn get_unread_count(&self, user_id: Uuid) -> Result<i64> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM notifikasi.notifications
            WHERE user_id = $1 AND read = false
        "#;

        let client = self.db_pool.get().await?;
        let row = client.query_one(query, &[&user_id]).await?;
        let count: i64 = row.get("count");

        Ok(count)
    }

    pub async fn mark_as_read(&self, notification_id: Uuid) -> Result<()> {
        let query = r#"
            UPDATE notifikasi.notifications
            SET read = true, read_at = NOW()
            WHERE id = $1
        "#;

        self.db_pool.get().await?
            .execute(query, &[&notification_id])
            .await?;

        Ok(())
    }
}
```

### 7.4 Email Notifications

```rust
// layanan/perlengkapan/crates/notifikasi/src/channels/email.rs
use lettre::{
    Message, SmtpTransport, Transport,
    transport::smtp::authentication::Credentials,
};

pub struct EmailNotificationChannel {
    smtp_transport: SmtpTransport,
    from_address: String,
    template_engine: TemplateEngine,
}

impl EmailNotificationChannel {
    pub async fn new(secreton_client: &SecretonGrpcClient) -> Result<Self> {
        // Fetch SMTP credentials from Secreton
        let smtp_username = secreton_client
            .get_secret(GetSecretRequest {
                path: "notification/smtp/username".to_string(),
                version: None,
            })
            .await?
            .into_inner()
            .data;

        let smtp_password = secreton_client
            .get_secret(GetSecretRequest {
                path: "notification/smtp/password".to_string(),
                version: None,
            })
            .await?
            .into_inner()
            .data;

        let credentials = Credentials::new(
            String::from_utf8(smtp_username)?,
            String::from_utf8(smtp_password)?,
        );

        let smtp_transport = SmtpTransport::relay("smtp.gmail.com")?
            .credentials(credentials)
            .build();

        Ok(Self {
            smtp_transport,
            from_address: "noreply@kejaksaan.go.id".to_string(),
            template_engine: TemplateEngine::new().await?,
        })
    }

    pub async fn send(
        &self,
        to_address: &str,
        notification_type: NotificationType,
    ) -> Result<()> {
        let (subject, body) = self.render_email(&notification_type)?;

        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(to_address.parse()?)
            .subject(subject)
            .body(body)?;

        // Send with retry logic
        let mut retries = 0;
        let max_retries = 3;

        loop {
            match self.smtp_transport.send(&email) {
                Ok(_) => {
                    tracing::info!("Email sent to {}", to_address);
                    return Ok(());
                }
                Err(e) if retries < max_retries => {
                    retries += 1;
                    tracing::warn!("Email send retry {} for {}: {}", retries, to_address, e);
                    tokio::time::sleep(Duration::from_secs(2u64.pow(retries))).await;
                }
                Err(e) => {
                    tracing::error!("Email send failed after {} retries: {}", max_retries, e);
                    return Err(NotificationError::EmailSendFailed(e.to_string()));
                }
            }
        }
    }

    fn render_email(&self, notification_type: &NotificationType) -> Result<(String, String)> {
        match notification_type {
            NotificationType::WorkflowStateChange { entity_type, to_state, .. } => {
                let subject = format!("{} Status Update", entity_type);
                let body = format!(
                    "Your {} has been updated to status: {}",
                    entity_type, to_state
                );
                Ok((subject, body))
            }
            NotificationType::DocumentReady { document_type, document_id } => {
                let subject = "Document Ready for Download".to_string();
                let body = format!(
                    "Your {} is ready. Download at: https://simpel.kejaksaan.go.id/documents/{}",
                    document_type, document_id
                );
                Ok((subject, body))
            }
            _ => {
                let subject = "Notification from SIMPEL".to_string();
                let body = "You have a new notification".to_string();
                Ok((subject, body))
            }
        }
    }
}
```

### 7.5 Batch Notification Scheduler

```rust
// layanan/perlengkapan/crates/notifikasi/src/scheduler.rs
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct NotificationScheduler {
    db_pool: deadpool_postgres::Pool,
    email_channel: EmailNotificationChannel,
}

impl NotificationScheduler {
    pub async fn start(&self) -> Result<()> {
        let scheduler = JobScheduler::new().await?;

        // Daily digest at 08:00 WIB
        let daily_digest_job = Job::new_async("0 0 8 * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = self.send_daily_digest().await {
                    tracing::error!("Daily digest failed: {}", e);
                }
            })
        })?;

        // Izin expiry reminders (H-30, H-14, H-7)
        let izin_reminder_job = Job::new_async("0 0 9 * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = self.send_izin_expiry_reminders().await {
                    tracing::error!("Izin expiry reminders failed: {}", e);
                }
            })
        })?;

        scheduler.add(daily_digest_job).await?;
        scheduler.add(izin_reminder_job).await?;
        scheduler.start().await?;

        Ok(())
    }

    async fn send_daily_digest(&self) -> Result<()> {
        tracing::info!("Sending daily digest");

        // Get users with batch_non_urgent preference
        let query = r#"
            SELECT DISTINCT n.user_id, u.email
            FROM notifikasi.notifications n
            JOIN authenc.users u ON n.user_id = u.id
            WHERE n.priority = 'low'
              AND n.read = false
              AND n.created_at >= NOW() - INTERVAL '24 hours'
              AND u.notification_preferences->>'batch_non_urgent' = 'true'
        "#;

        let client = self.db_pool.get().await?;
        let rows = client.query(query, &[]).await?;

        for row in rows {
            let user_id: Uuid = row.get("user_id");
            let email: String = row.get("email");

            // Get all unread notifications for this user
            let notifications = self.get_unread_notifications(user_id).await?;

            // Send digest email
            self.send_digest_email(&email, notifications).await?;
        }

        Ok(())
    }

    async fn send_izin_expiry_reminders(&self) -> Result<()> {
        tracing::info!("Sending izin expiry reminders");

        // Find izin expiring in 30, 14, or 7 days
        let query = r#"
            SELECT id, user_id, tanggal_berakhir
            FROM perlengkapan.izin_bmn
            WHERE tanggal_berakhir IN (
                CURRENT_DATE + INTERVAL '30 days',
                CURRENT_DATE + INTERVAL '14 days',
                CURRENT_DATE + INTERVAL '7 days'
            )
        "#;

        let client = self.db_pool.get().await?;
        let rows = client.query(query, &[]).await?;

        for row in rows {
            let izin_id: Uuid = row.get("id");
            let user_id: Uuid = row.get("user_id");
            let tanggal_berakhir: chrono::NaiveDate = row.get("tanggal_berakhir");

            let days_until_expiry = (tanggal_berakhir - chrono::Local::now().date_naive()).num_days();

            // Send notification
            let notification_type = NotificationType::IzinExpiry {
                izin_id,
                days_until_expiry: days_until_expiry as i32,
            };

            self.send_notification(user_id, notification_type, NotificationPriority::Normal).await?;
        }

        Ok(())
    }
}
```

### 7.6 gRPC Service

```rust
// layanan/perlengkapan/crates/notifikasi/src/grpc/service.rs
use tonic::{Request, Response, Status};

pub mod notifikasi_proto {
    tonic::include_proto!("notifikasi");
}

use notifikasi_proto::notification_service_server::{NotificationService, NotificationServiceServer};

#[derive(Debug)]
pub struct NotificationServiceImpl {
    in_app_channel: InAppNotificationChannel,
    email_channel: EmailNotificationChannel,
    sms_channel: Option<SmsNotificationChannel>,
    authenc_client: AuthencGrpcClient,
}

#[tonic::async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> Result<Response<SendNotificationResponse>, Status> {
        let req = request.into_inner();

        // Get user preferences from Authenc
        let preferences = self.authenc_client
            .get_user_notification_preferences(GetUserPreferencesRequest {
                user_id: req.user_id.clone(),
            })
            .await?
            .into_inner();

        let notification_type: NotificationType = serde_json::from_str(&req.notification_data)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let priority = match req.priority.as_str() {
            "low" => NotificationPriority::Low,
            "normal" => NotificationPriority::Normal,
            "high" => NotificationPriority::High,
            "urgent" => NotificationPriority::Urgent,
            _ => NotificationPriority::Normal,
        };

        // Send via enabled channels
        if preferences.in_app_enabled {
            self.in_app_channel.send(
                Uuid::parse_str(&req.user_id)?,
                notification_type.clone(),
                priority.clone(),
            ).await.map_err(|e| Status::internal(e.to_string()))?;
        }

        if preferences.email_enabled && !matches!(priority, NotificationPriority::Low) {
            self.email_channel.send(
                &preferences.email,
                notification_type.clone(),
            ).await.map_err(|e| Status::internal(e.to_string()))?;
        }

        if preferences.sms_enabled && matches!(priority, NotificationPriority::High | NotificationPriority::Urgent) {
            if let Some(sms_channel) = &self.sms_channel {
                sms_channel.send(
                    &preferences.phone_number,
                    notification_type,
                ).await.map_err(|e| Status::internal(e.to_string()))?;
            }
        }

        Ok(Response::new(SendNotificationResponse {
            success: true,
            message: "Notification sent".to_string(),
        }))
    }
}
```

## 8. Dashboard Design

### 8.1 Portal Dashboard (General)

**Location:** `layanan/portal/src/modules/dasbor.rs` (backend)
**Frontend:** `antarmuka/portal/src/pages/dashboard.rs`

**Metrics:**
```rust
// layanan/portal/src/modules/dasbor.rs
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PortalDashboardMetrics {
    pub system_metrics: SystemMetrics,
    pub cross_domain_metrics: CrossDomainMetrics,
    pub auth_metrics: AuthMetrics,
    pub integration_health: IntegrationHealth,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub total_users: i64,
    pub active_sessions: i64,
    pub system_health: String, // "healthy", "degraded", "down"
    pub uptime_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CrossDomainMetrics {
    pub total_documents: i64,
    pub notifications_sent_today: i64,
    pub api_calls_today: i64,
    pub average_response_time_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct AuthMetrics {
    pub login_success_rate: f64,
    pub mfa_usage_percentage: f64,
    pub failed_attempts_today: i64,
}

#[derive(Debug, Serialize)]
pub struct IntegrationHealth {
    pub siman_status: String,
    pub siman_last_sync: String,
    pub mysimkari_status: String,
    pub mysimkari_last_sync: String,
}

pub async fn get_portal_dashboard_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PortalDashboardMetrics>, AppError> {
    // Fetch system metrics
    let system_metrics = fetch_system_metrics(&state.db_pool).await?;

    // Fetch cross-domain metrics
    let cross_domain_metrics = fetch_cross_domain_metrics(&state.db_pool).await?;

    // Fetch auth metrics from Authenc
    let auth_metrics = fetch_auth_metrics(&state.authenc_client).await?;

    // Fetch integration health from Integration Service
    let integration_health = fetch_integration_health(&state.integration_client).await?;

    Ok(Json(PortalDashboardMetrics {
        system_metrics,
        cross_domain_metrics,
        auth_metrics,
        integration_health,
    }))
}

async fn fetch_system_metrics(db_pool: &deadpool_postgres::Pool) -> Result<SystemMetrics> {
    let query = r#"
        SELECT
            (SELECT COUNT(*) FROM authenc.users) as total_users,
            (SELECT COUNT(*) FROM authenc.sessions WHERE expires_at > NOW()) as active_sessions
    "#;

    let client = db_pool.get().await?;
    let row = client.query_one(query, &[]).await?;

    Ok(SystemMetrics {
        total_users: row.get("total_users"),
        active_sessions: row.get("active_sessions"),
        system_health: "healthy".to_string(),
        uptime_percentage: 99.9,
    })
}
```

**Frontend Component:**
```rust
// antarmuka/portal/src/pages/dashboard.rs
use leptos::prelude::*;
use lib_ui::dashboard::MetricCard;

#[component]
pub fn PortalDashboard() -> impl IntoView {
    let metrics = Resource::new(
        || (),
        |_| async move {
            gloo_net::http::Request::get("/api/v1/dashboard/portal")
                .send()
                .await?
                .json::<PortalDashboardMetrics>()
                .await
        }
    );

    view! {
        <div class="dashboard-container">
            <h1 class="text-2xl font-bold mb-6">"Portal Dashboard"</h1>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || metrics.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                            <MetricCard
                                title="Total Users".to_string()
                                value=data.system_metrics.total_users.to_string()
                                change=None
                                icon="👥".to_string()
                            />
                            <MetricCard
                                title="Active Sessions".to_string()
                                value=data.system_metrics.active_sessions.to_string()
                                change=None
                                icon="🔐".to_string()
                            />
                            <MetricCard
                                title="Documents".to_string()
                                value=data.cross_domain_metrics.total_documents.to_string()
                                change=None
                                icon="📄".to_string()
                            />
                            <MetricCard
                                title="API Calls Today".to_string()
                                value=data.cross_domain_metrics.api_calls_today.to_string()
                                change=None
                                icon="📊".to_string()
                            />
                        </div>

                        <div class="mt-8">
                            <h2 class="text-xl font-bold mb-4">"Integration Health"</h2>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <IntegrationStatusCard
                                    name="SIMAN".to_string()
                                    status=data.integration_health.siman_status.clone()
                                    last_sync=data.integration_health.siman_last_sync.clone()
                                />
                                <IntegrationStatusCard
                                    name="MySIMKARI".to_string()
                                    status=data.integration_health.mysimkari_status.clone()
                                    last_sync=data.integration_health.mysimkari_last_sync.clone()
                                />
                            </div>
                        </div>
                    }.into_any(),
                    Err(e) => view! { <ErrorDisplay error=e.to_string() /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
```

### 8.2 Perlengkapan Dashboard (Domain-Specific)

**Location:** `layanan/perlengkapan/crates/api/src/handlers/dashboard.rs` (backend)
**Frontend:** `antarmuka/perlengkapan/src/pages/dashboard.rs`

**Metrics:**
```rust
// layanan/perlengkapan/crates/api/src/handlers/dashboard.rs

#[derive(Debug, Serialize)]
pub struct PerlengkapanDashboardMetrics {
    pub kebutuhan_metrics: KebutuhanMetrics,
    pub gap_analysis: Vec<GapAnalysisResult>,
    pub pakaian_dinas_metrics: PakaianDinasMetrics,
    pub workflow_metrics: WorkflowMetrics,
    pub asset_utilization: AssetUtilization,
}

#[derive(Debug, Serialize)]
pub struct KebutuhanMetrics {
    pub total_by_status: HashMap<String, i64>,
    pub total_by_satker: Vec<SatkerCount>,
    pub total_by_tahun: HashMap<i32, i64>,
}

#[derive(Debug, Serialize)]
pub struct SatkerCount {
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct WorkflowMetrics {
    pub average_processing_time_hours: f64,
    pub bottlenecks: Vec<BottleneckInfo>,
    pub sla_breaches_today: i64,
}

#[derive(Debug, Serialize)]
pub struct BottleneckInfo {
    pub state: String,
    pub average_time_hours: f64,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AssetUtilization {
    pub total_assets: i64,
    pub assets_in_good_condition: i64,
    pub utilization_percentage: f64,
}

pub async fn get_perlengkapan_dashboard_metrics(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DashboardParams>,
) -> Result<Json<PerlengkapanDashboardMetrics>, AppError> {
    // Fetch kebutuhan metrics
    let kebutuhan_metrics = fetch_kebutuhan_metrics(&state.db_pool, &params).await?;

    // Fetch gap analysis (top 10 asset types)
    let gap_analysis = fetch_gap_analysis(&state.db_pool, &state.integration_client, 10).await?;

    // Fetch pakaian dinas metrics
    let pakaian_dinas_metrics = fetch_pakaian_dinas_metrics(&state.db_pool, &params).await?;

    // Fetch workflow metrics
    let workflow_metrics = fetch_workflow_metrics(&state.db_pool).await?;

    // Fetch asset utilization from SIMAN
    let asset_utilization = fetch_asset_utilization(&state.integration_client).await?;

    Ok(Json(PerlengkapanDashboardMetrics {
        kebutuhan_metrics,
        gap_analysis,
        pakaian_dinas_metrics,
        workflow_metrics,
        asset_utilization,
    }))
}

async fn fetch_kebutuhan_metrics(
    db_pool: &deadpool_postgres::Pool,
    params: &DashboardParams,
) -> Result<KebutuhanMetrics> {
    // Total by status
    let status_query = r#"
        SELECT status, COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn
        WHERE tahun_anggaran = $1
        GROUP BY status
    "#;

    let client = db_pool.get().await?;
    let rows = client.query(status_query, &[&params.tahun_anggaran]).await?;

    let total_by_status: HashMap<String, i64> = rows.into_iter()
        .map(|row| (row.get("status"), row.get("count")))
        .collect();

    // Total by satker (top 10)
    let satker_query = r#"
        SELECT k.satker_id, s.nama as satker_nama, COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn k
        JOIN authenc.satkers s ON k.satker_id = s.id
        WHERE k.tahun_anggaran = $1
        GROUP BY k.satker_id, s.nama
        ORDER BY count DESC
        LIMIT 10
    "#;

    let rows = client.query(satker_query, &[&params.tahun_anggaran]).await?;

    let total_by_satker: Vec<SatkerCount> = rows.into_iter()
        .map(|row| SatkerCount {
            satker_id: row.get("satker_id"),
            satker_nama: row.get("satker_nama"),
            count: row.get("count"),
        })
        .collect();

    // Total by tahun
    let tahun_query = r#"
        SELECT tahun_anggaran, COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn
        GROUP BY tahun_anggaran
        ORDER BY tahun_anggaran DESC
        LIMIT 5
    "#;

    let rows = client.query(tahun_query, &[]).await?;

    let total_by_tahun: HashMap<i32, i64> = rows.into_iter()
        .map(|row| (row.get("tahun_anggaran"), row.get("count")))
        .collect();

    Ok(KebutuhanMetrics {
        total_by_status,
        total_by_satker,
        total_by_tahun,
    })
}

async fn fetch_workflow_metrics(db_pool: &deadpool_postgres::Pool) -> Result<WorkflowMetrics> {
    // Average processing time
    let avg_time_query = r#"
        SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) / 3600) as avg_hours
        FROM perlengkapan.kebutuhan_bmn
        WHERE status = 'COMPLETED'
    "#;

    let client = db_pool.get().await?;
    let row = client.query_one(avg_time_query, &[]).await?;
    let average_processing_time_hours: f64 = row.get("avg_hours");

    // Bottlenecks (states with longest average time)
    let bottleneck_query = r#"
        SELECT
            a.kode as state,
            AVG(EXTRACT(EPOCH FROM (
                LEAD(ka.created_at) OVER (PARTITION BY ka.pengajuan_id ORDER BY ka.created_at) - ka.created_at
            )) / 3600) as avg_hours,
            COUNT(*) as count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas ka
        JOIN perlengkapan.ms_aktivitas_bmn a ON ka.aktivitas_id = a.id
        GROUP BY a.kode
        HAVING AVG(EXTRACT(EPOCH FROM (
            LEAD(ka.created_at) OVER (PARTITION BY ka.pengajuan_id ORDER BY ka.created_at) - ka.created_at
        )) / 3600) > 24
        ORDER BY avg_hours DESC
        LIMIT 5
    "#;

    let rows = client.query(bottleneck_query, &[]).await?;

    let bottlenecks: Vec<BottleneckInfo> = rows.into_iter()
        .map(|row| BottleneckInfo {
            state: row.get("state"),
            average_time_hours: row.get("avg_hours"),
            count: row.get("count"),
        })
        .collect();

    // SLA breaches today
    let sla_breach_query = r#"
        SELECT COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn k
        JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas ka ON k.id = ka.pengajuan_id
        WHERE ka.created_at < NOW() - INTERVAL '2 days'
          AND k.status NOT IN ('COMPLETED', 'ARCHIVED', 'CANCELLED')
    "#;

    let row = client.query_one(sla_breach_query, &[]).await?;
    let sla_breaches_today: i64 = row.get("count");

    Ok(WorkflowMetrics {
        average_processing_time_hours,
        bottlenecks,
        sla_breaches_today,
    })
}
```

**Frontend Component:**
```rust
// antarmuka/perlengkapan/src/pages/dashboard.rs
use leptos::prelude::*;
use lib_ui::dashboard::{MetricCard, BarChart, PieChart};

#[component]
pub fn PerlengkapanDashboard() -> impl IntoView {
    let (tahun_anggaran, set_tahun_anggaran) = signal(2026);

    let metrics = Resource::new(
        move || tahun_anggaran.get(),
        |tahun| async move {
            gloo_net::http::Request::get(&format!(
                "/api/v1/dashboard/perlengkapan?tahun_anggaran={}",
                tahun
            ))
            .send()
            .await?
            .json::<PerlengkapanDashboardMetrics>()
            .await
        }
    );

    view! {
        <div class="dashboard-container">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold">"Perlengkapan Dashboard"</h1>
                <select
                    class="form-select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev).parse::<i32>().unwrap_or(2026);
                        set_tahun_anggaran.set(value);
                    }
                >
                    <option value="2024">"2024"</option>
                    <option value="2025">"2025"</option>
                    <option value="2026" selected>"2026"</option>
                </select>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || metrics.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="space-y-8">
                            // Kebutuhan metrics
                            <div>
                                <h2 class="text-xl font-bold mb-4">"Kebutuhan BMN"</h2>
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                    <PieChart
                                        title="By Status".to_string()
                                        data=data.kebutuhan_metrics.total_by_status.clone()
                                    />
                                    <BarChart
                                        title="Top 10 Satker".to_string()
                                        data=data.kebutuhan_metrics.total_by_satker.clone()
                                    />
                                </div>
                            </div>

                            // Gap analysis
                            <div>
                                <h2 class="text-xl font-bold mb-4">"Gap Analysis (Top 10)"</h2>
                                <GapAnalysisTable data=data.gap_analysis.clone() />
                            </div>

                            // Workflow metrics
                            <div>
                                <h2 class="text-xl font-bold mb-4">"Workflow Performance"</h2>
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                    <MetricCard
                                        title="Avg Processing Time".to_string()
                                        value=format!("{:.1} hours", data.workflow_metrics.average_processing_time_hours)
                                        change=None
                                        icon="⏱️".to_string()
                                    />
                                    <MetricCard
                                        title="SLA Breaches Today".to_string()
                                        value=data.workflow_metrics.sla_breaches_today.to_string()
                                        change=None
                                        icon="⚠️".to_string()
                                    />
                                    <MetricCard
                                        title="Asset Utilization".to_string()
                                        value=format!("{:.1}%", data.asset_utilization.utilization_percentage)
                                        change=None
                                        icon="📦".to_string()
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    Err(e) => view! { <ErrorDisplay error=e.to_string() /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
```

### 8.3 Real-Time Updates

**WebSocket Support:**
```rust
// layanan/perlengkapan/crates/api/src/handlers/websocket.rs
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::Response,
};
use tokio::sync::broadcast;

pub async fn dashboard_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_dashboard_socket(socket, state))
}

async fn handle_dashboard_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.dashboard_updates.subscribe();

    while let Ok(update) = rx.recv().await {
        let message = serde_json::to_string(&update).unwrap();

        if socket.send(axum::extract::ws::Message::Text(message)).await.is_err() {
            break;
        }
    }
}

// Broadcast dashboard updates
pub async fn broadcast_dashboard_update(
    tx: &broadcast::Sender<DashboardUpdate>,
    update: DashboardUpdate,
) {
    let _ = tx.send(update);
}
```

## 9. Advanced Features

### 9.1 Search Implementation

**Location:** `lib/perlengkapan/src/search.rs`

**Full-Text Search with pg_trgm:**
```rust
// lib/perlengkapan/src/search.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub filters: SearchFilters,
    pub sort_by: SortBy,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub status: Option<Vec<String>>,
    pub tahun_anggaran: Option<i32>,
    pub satker_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    DateDesc,
    DateAsc,
    Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

pub struct SearchEngine {
    db_pool: deadpool_postgres::Pool,
}

impl SearchEngine {
    pub async fn search_kebutuhan(
        &self,
        query: SearchQuery,
    ) -> Result<SearchResult<KebutuhanBmn>> {
        let mut sql = String::from(r#"
            SELECT k.*,
                   ts_rank(
                       to_tsvector('indonesian', k.nama_barang || ' ' || COALESCE(k.deskripsi, '')),
                       plainto_tsquery('indonesian', $1)
                   ) as relevance
            FROM perlengkapan.kebutuhan_bmn k
            WHERE 1=1
        "#);

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = vec![
            Box::new(query.query.clone()),
        ];
        let mut param_idx = 2;

        // Add filters
        if let Some(status_list) = &query.filters.status {
            sql.push_str(&format!(" AND k.status = ANY(${})", param_idx));
            params.push(Box::new(status_list.clone()));
            param_idx += 1;
        }

        if let Some(tahun) = query.filters.tahun_anggaran {
            sql.push_str(&format!(" AND k.tahun_anggaran = ${}", param_idx));
            params.push(Box::new(tahun));
            param_idx += 1;
        }

        if let Some(satker_id) = query.filters.satker_id {
            sql.push_str(&format!(" AND k.satker_id = ${}", param_idx));
            params.push(Box::new(satker_id));
            param_idx += 1;
        }

        // Add full-text search condition
        sql.push_str(&format!(
            " AND (to_tsvector('indonesian', k.nama_barang || ' ' || COALESCE(k.deskripsi, '')) @@ plainto_tsquery('indonesian', $1)
               OR k.kode_barang ILIKE '%' || $1 || '%')"
        ));

        // Add sorting
        match query.sort_by {
            SortBy::Relevance => sql.push_str(" ORDER BY relevance DESC"),
            SortBy::DateDesc => sql.push_str(" ORDER BY k.created_at DESC"),
            SortBy::DateAsc => sql.push_str(" ORDER BY k.created_at ASC"),
            SortBy::Priority => sql.push_str(" ORDER BY k.priority_score DESC"),
        }

        // Add pagination
        let offset = (query.page - 1) * query.page_size;
        sql.push_str(&format!(" LIMIT {} OFFSET {}", query.page_size, offset));

        // Execute query
        let client = self.db_pool.get().await?;
        let rows = client.query(&sql, &params.iter().map(|p| p.as_ref()).collect::<Vec<_>>()).await?;

        let items: Vec<KebutuhanBmn> = rows.into_iter()
            .map(|row| KebutuhanBmn::from_row(&row))
            .collect::<Result<Vec<_>>>()?;

        // Get total count
        let count_sql = format!(
            "SELECT COUNT(*) FROM ({}) as subquery",
            sql.split("LIMIT").next().unwrap()
        );

        let count_row = client.query_one(&count_sql, &params.iter().map(|p| p.as_ref()).collect::<Vec<_>>()).await?;
        let total: i64 = count_row.get(0);

        let total_pages = ((total as f64) / (query.page_size as f64)).ceil() as u32;

        Ok(SearchResult {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
            total_pages,
        })
    }

    pub async fn highlight_search_terms(
        &self,
        text: &str,
        query: &str,
    ) -> String {
        // Simple highlighting - wrap matching terms in <mark> tags
        let terms: Vec<&str> = query.split_whitespace().collect();
        let mut highlighted = text.to_string();

        for term in terms {
            let pattern = regex::Regex::new(&format!("(?i){}", regex::escape(term))).unwrap();
            highlighted = pattern.replace_all(&highlighted, "<mark>$0</mark>").to_string();
        }

        highlighted
    }
}
```

**REST API Handler:**
```rust
// layanan/perlengkapan/crates/api/src/handlers/search.rs
use axum::{extract::{Query, State}, Json};

pub async fn search_kebutuhan(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResult<KebutuhanBmn>>, AppError> {
    let search_engine = lib_perlengkapan::search::SearchEngine::new(state.db_pool.clone());

    let results = search_engine.search_kebutuhan(query).await?;

    Ok(Json(results))
}
```

### 9.2 Export Functionality

**Location:** `layanan/perlengkapan/crates/api/src/handlers/export.rs`

**Excel Export:**
```rust
// layanan/perlengkapan/crates/api/src/handlers/export.rs
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    http::{header, StatusCode},
};
use rust_xlsxwriter::Workbook;

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub entity_type: String,
    pub filters: Option<String>, // JSON-encoded filters
    pub limit: Option<u32>,
}

pub async fn export_to_excel(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    // Limit export size
    let limit = query.limit.unwrap_or(50000).min(50000);

    // Check if async export needed (>1000 rows)
    if limit > 1000 {
        // Queue async export job
        let job_id = queue_export_job(&state, query).await?;

        return Ok((
            StatusCode::ACCEPTED,
            Json(json!({
                "job_id": job_id,
                "status": "queued",
                "message": "Export job queued. You will be notified when ready."
            }))
        ).into_response());
    }

    // Synchronous export for small datasets
    let data = fetch_export_data(&state, &query).await?;

    let excel_data = match query.entity_type.as_str() {
        "kebutuhan_bmn" => generate_kebutuhan_excel(data)?,
        "pakaian_dinas" => generate_pakaian_dinas_excel(data)?,
        _ => return Err(AppError::InvalidInput("Unknown entity type".to_string())),
    };

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}.xlsx\"", query.entity_type)),
        ],
        excel_data,
    ).into_response())
}

fn generate_kebutuhan_excel(data: Vec<KebutuhanBmn>) -> Result<Vec<u8>> {
    let mut workbook = Workbook::new();

    // Data sheet
    let data_sheet = workbook.add_worksheet();
    data_sheet.set_name("Data")?;

    // Header
    let headers = vec![
        "No", "Satker", "Kode Barang", "Nama Barang",
        "Jumlah Kebutuhan", "Tahun Anggaran", "Status", "Created At"
    ];

    for (col, header) in headers.iter().enumerate() {
        data_sheet.write_string(0, col as u16, header)?;
    }

    // Data rows
    for (idx, kebutuhan) in data.iter().enumerate() {
        let row = (idx + 1) as u32;
        data_sheet.write_number(row, 0, (idx + 1) as f64)?;
        data_sheet.write_string(row, 1, &kebutuhan.satker_nama)?;
        data_sheet.write_string(row, 2, &kebutuhan.kode_barang)?;
        data_sheet.write_string(row, 3, &kebutuhan.nama_barang)?;
        data_sheet.write_number(row, 4, kebutuhan.jumlah_kebutuhan as f64)?;
        data_sheet.write_number(row, 5, kebutuhan.tahun_anggaran as f64)?;
        data_sheet.write_string(row, 6, &kebutuhan.status)?;
        data_sheet.write_string(row, 7, &kebutuhan.created_at.to_rfc3339())?;
    }

    // Metadata sheet
    let metadata_sheet = workbook.add_worksheet();
    metadata_sheet.set_name("Metadata")?;
    metadata_sheet.write_string(0, 0, "Export Date")?;
    metadata_sheet.write_string(0, 1, &chrono::Utc::now().to_rfc3339())?;
    metadata_sheet.write_string(1, 0, "Total Rows")?;
    metadata_sheet.write_number(1, 1, data.len() as f64)?;

    // Save to buffer
    let mut buffer = Vec::new();
    workbook.save_to_buffer(&mut buffer)?;

    Ok(buffer)
}

async fn queue_export_job(
    state: &Arc<AppState>,
    query: ExportQuery,
) -> Result<Uuid> {
    let job_id = Uuid::new_v4();

    // Store job in database
    let sql = r#"
        INSERT INTO perlengkapan.export_jobs
        (id, entity_type, filters, status, created_at)
        VALUES ($1, $2, $3, 'queued', NOW())
    "#;

    state.db_pool.get().await?
        .execute(sql, &[&job_id, &query.entity_type, &query.filters])
        .await?;

    // Queue background job
    tokio::spawn(async move {
        if let Err(e) = process_export_job(state.clone(), job_id).await {
            tracing::error!("Export job {} failed: {}", job_id, e);
        }
    });

    Ok(job_id)
}

async fn process_export_job(state: Arc<AppState>, job_id: Uuid) -> Result<()> {
    // Update status to processing
    update_job_status(&state, job_id, "processing").await?;

    // Fetch data
    let data = fetch_export_data_by_job_id(&state, job_id).await?;

    // Generate Excel
    let excel_data = generate_kebutuhan_excel(data)?;

    // Store in document service
    let document_id = state.document_client
        .generate_document(GenerateDocumentRequest {
            document_type: "export".to_string(),
            filename: format!("export_{}.xlsx", job_id),
            data: excel_data,
            user_id: "system".to_string(),
            entity_id: job_id.to_string(),
        })
        .await?
        .into_inner()
        .document_id;

    // Update job status
    update_job_status(&state, job_id, "completed").await?;

    // Send notification
    state.notification_client
        .send_notification(SendNotificationRequest {
            user_id: "user_id".to_string(), // Get from job
            notification_data: serde_json::to_string(&NotificationType::DocumentReady {
                document_id: Uuid::parse_str(&document_id)?,
                document_type: "export".to_string(),
            })?,
            priority: "normal".to_string(),
        })
        .await?;

    Ok(())
}
```

### 9.3 Batch Operations

**Location:** `layanan/perlengkapan/crates/api/src/handlers/batch.rs`

**Batch Approval:**
```rust
// layanan/perlengkapan/crates/api/src/handlers/batch.rs
use axum::{extract::State, Json};

#[derive(Debug, Deserialize)]
pub struct BatchApprovalRequest {
    pub kebutuhan_ids: Vec<Uuid>,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchOperationResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub failures: Vec<BatchFailure>,
}

#[derive(Debug, Serialize)]
pub struct BatchFailure {
    pub id: Uuid,
    pub error: String,
}

pub async fn batch_approve_kebutuhan(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BatchApprovalRequest>,
) -> Result<Json<BatchOperationResult>, AppError> {
    // Validate batch size
    if request.kebutuhan_ids.len() > 500 {
        return Err(AppError::InvalidInput("Batch size exceeds 500".to_string()));
    }

    let batch_id = Uuid::new_v4();
    let mut successful = 0;
    let mut failures = Vec::new();

    // Process each item
    for kebutuhan_id in &request.kebutuhan_ids {
        // Use transaction per item (not all-or-nothing for entire batch)
        let result = state.db_pool.transaction(|tx| async move {
            // Validate current state
            let current_state = get_kebutuhan_status(tx, kebutuhan_id).await?;

            if current_state != "REVIEWED" {
                return Err(AppError::InvalidState(format!(
                    "Cannot approve from state: {}",
                    current_state
                )));
            }

            // Transition to APPROVED
            state.workflow_engine.transition(
                *kebutuhan_id,
                &current_state,
                "APPROVED",
                state.current_user_id,
                request.catatan.clone(),
            ).await?;

            Ok(())
        }).await;

        match result {
            Ok(_) => successful += 1,
            Err(e) => failures.push(BatchFailure {
                id: *kebutuhan_id,
                error: e.to_string(),
            }),
        }
    }

    // Record batch operation in audit trail
    state.audit_logger.log(
        lib_common::audit::AuditEvent::BatchOperation {
            batch_id,
            operation: "batch_approve".to_string(),
            count: request.kebutuhan_ids.len(),
        },
        state.current_user_id,
        state.ip_address.clone(),
    ).await?;

    // Send summary notification
    state.notification_client
        .send_notification(SendNotificationRequest {
            user_id: state.current_user_id.to_string(),
            notification_data: serde_json::to_string(&json!({
                "type": "batch_operation_complete",
                "operation": "approve",
                "total": request.kebutuhan_ids.len(),
                "successful": successful,
                "failed": failures.len(),
            }))?,
            priority: "normal".to_string(),
        })
        .await?;

    Ok(Json(BatchOperationResult {
        total: request.kebutuhan_ids.len(),
        successful,
        failed: failures.len(),
        failures,
    }))
}

pub async fn batch_reject_kebutuhan(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BatchRejectionRequest>,
) -> Result<Json<BatchOperationResult>, AppError> {
    // Similar implementation to batch_approve_kebutuhan
    // but transitions to REJECTED state
    todo!()
}

pub async fn batch_update_status(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BatchStatusUpdateRequest>,
) -> Result<Json<BatchOperationResult>, AppError> {
    // Similar implementation for status updates
    todo!()
}
```

## 10. Performance Optimization

### 10.1 Database Optimization

**Indexes:**
```sql
-- Foreign key indexes
CREATE INDEX idx_kebutuhan_bmn_satker ON perlengkapan.kebutuhan_bmn(satker_id);
CREATE INDEX idx_kebutuhan_bmn_tahun ON perlengkapan.kebutuhan_bmn(tahun_anggaran);
CREATE INDEX idx_kebutuhan_bmn_status ON perlengkapan.kebutuhan_bmn(status);
CREATE INDEX idx_kebutuhan_bmn_kode ON perlengkapan.kebutuhan_bmn(kode_barang);

-- Full-text search indexes
CREATE INDEX idx_kebutuhan_bmn_fts ON perlengkapan.kebutuhan_bmn
USING GIN(to_tsvector('indonesian', nama_barang || ' ' || COALESCE(deskripsi, '')));

-- JSONB indexes
CREATE INDEX idx_siman_aset_raw ON integrasi.siman_aset_tanah USING GIN(raw_data);

-- Composite indexes for common queries
CREATE INDEX idx_kebutuhan_bmn_satker_tahun_status
ON perlengkapan.kebutuhan_bmn(satker_id, tahun_anggaran, status);
```

**Database Views:**
```sql
-- Materialized view for dashboard (refresh every 5 minutes)
CREATE MATERIALIZED VIEW perlengkapan.mv_dashboard_metrics AS
SELECT
    COUNT(*) as total_kebutuhan,
    COUNT(*) FILTER (WHERE status = 'DRAFT') as draft_count,
    COUNT(*) FILTER (WHERE status = 'SUBMITTED') as submitted_count,
    COUNT(*) FILTER (WHERE status = 'APPROVED') as approved_count,
    AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) / 3600) as avg_processing_hours
FROM perlengkapan.kebutuhan_bmn
WHERE tahun_anggaran = EXTRACT(YEAR FROM CURRENT_DATE);

CREATE UNIQUE INDEX idx_mv_dashboard_metrics ON perlengkapan.mv_dashboard_metrics((1));

-- Refresh job (run every 5 minutes)
CREATE OR REPLACE FUNCTION refresh_dashboard_metrics()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_dashboard_metrics;
END;
$$ LANGUAGE plpgsql;
```

### 10.2 Caching Strategy

**Redis Caching:**
```rust
// lib/common/src/cache.rs - Extended implementation

impl CacheManager {
    // Cache reference data (1 hour TTL)
    pub async fn cache_reference_data(&self) -> Result<()> {
        // Cache master barang
        let barang_list = fetch_master_barang().await?;
        self.set("ref:master_barang", &barang_list, SensitivityLevel::Public).await?;

        // Cache satker list
        let satker_list = fetch_satker_list().await?;
        self.set("ref:satker_list", &satker_list, SensitivityLevel::Internal).await?;

        // Cache workflow states
        let workflow_states = fetch_workflow_states().await?;
        self.set("ref:workflow_states", &workflow_states, SensitivityLevel::Public).await?;

        Ok(())
    }

    // Cache gap analysis results (1 hour TTL)
    pub async fn cache_gap_analysis(
        &self,
        satker_id: Uuid,
        results: &Vec<GapAnalysisResult>,
    ) -> Result<()> {
        let key = format!("gap_analysis:{}", satker_id);
        self.set(&key, results, SensitivityLevel::Internal).await
    }

    // Invalidate cache on data changes
    pub async fn invalidate_kebutuhan_cache(&self, satker_id: Uuid) -> Result<()> {
        self.invalidate(&format!("gap_analysis:{}*", satker_id)).await?;
        self.invalidate("dashboard:*").await?;
        Ok(())
    }
}
```

**Cache Middleware:**
```rust
// layanan/perlengkapan/crates/api/src/middleware/cache.rs
use axum::{
    middleware::Next,
    response::Response,
    extract::Request,
};

pub async fn cache_middleware(
    req: Request,
    next: Next,
) -> Response {
    let cache_key = format!("http:{}:{}", req.method(), req.uri());

    // Check cache
    if let Ok(Some(cached_response)) = CACHE.get::<CachedResponse>(&cache_key).await {
        return cached_response.into_response();
    }

    // Execute request
    let response = next.run(req).await;

    // Cache GET requests only
    if req.method() == Method::GET && response.status().is_success() {
        let cached = CachedResponse::from_response(&response);
        let _ = CACHE.set(&cache_key, &cached, SensitivityLevel::Internal).await;
    }

    response
}
```

### 10.3 Connection Pooling

**Database Pool Configuration:**
```rust
// lib/common/src/database.rs
use deadpool_postgres::{Config, Pool, Runtime};

pub async fn create_pool(database_url: &str) -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.url = Some(database_url.to_string());
    cfg.pool = Some(deadpool_postgres::PoolConfig {
        max_size: 50,
        min_size: 10,
        timeouts: deadpool_postgres::Timeouts {
            wait: Some(Duration::from_secs(5)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(5)),
        },
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

    Ok(pool)
}
```

### 10.4 Rate Limiting

**Rate Limiter:**
```rust
// layanan/perlengkapan/crates/api/src/middleware/rate_limit.rs
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};

pub fn rate_limit_layer() -> GovernorLayer<'static> {
    let config = Box::new(
        GovernorConfigBuilder::default()
            .per_second(100)  // 100 requests per second
            .burst_size(200)  // Allow burst of 200
            .finish()
            .unwrap()
    );

    GovernorLayer {
        config: Box::leak(config),
    }
}
```

### 10.5 Prepared Statements

**Prepared Statement Cache:**
```rust
// lib/common/src/database.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PreparedStatementCache {
    cache: Arc<RwLock<HashMap<String, tokio_postgres::Statement>>>,
}

impl PreparedStatementCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_prepare(
        &self,
        client: &tokio_postgres::Client,
        sql: &str,
    ) -> Result<tokio_postgres::Statement> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(stmt) = cache.get(sql) {
                return Ok(stmt.clone());
            }
        }

        // Prepare statement
        let stmt = client.prepare(sql).await?;

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(sql.to_string(), stmt.clone());
        }

        Ok(stmt)
    }
}
```

## 11. Testing Strategy

### 11.1 Unit Tests

**Example - Gap Analysis Tests:**
```rust
// lib/perlengkapan/src/gap_analysis.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_gap_positive() {
        let analyzer = GapAnalyzer::new_mock();

        let result = analyzer.calculate_gap(
            Uuid::new_v4(),
            "KB001",
            100,
            &mock_siman_client_with_assets(80),
        ).await.unwrap();

        assert_eq!(result.gap, 20);
        assert_eq!(result.existing_good_quantity, 80);
    }

    #[tokio::test]
    async fn test_calculate_gap_negative() {
        let analyzer = GapAnalyzer::new_mock();

        let result = analyzer.calculate_gap(
            Uuid::new_v4(),
            "KB001",
            100,
            &mock_siman_client_with_assets(120),
        ).await.unwrap();

        assert_eq!(result.gap, -20);
        assert_eq!(result.existing_good_quantity, 120);
    }

    #[tokio::test]
    async fn test_calculate_gap_only_good_condition() {
        let analyzer = GapAnalyzer::new_mock();

        // Mock returns 100 assets: 80 BAIK, 20 RUSAK
        let result = analyzer.calculate_gap(
            Uuid::new_v4(),
            "KB001",
            100,
            &mock_siman_client_with_mixed_condition(80, 20),
        ).await.unwrap();

        // Should only count BAIK condition
        assert_eq!(result.existing_good_quantity, 80);
        assert_eq!(result.gap, 20);
    }
}
```

### 11.2 Integration Tests

**Example - Workflow Engine Tests:**
```rust
// layanan/perlengkapan/crates/api/tests/workflow_tests.rs
use testcontainers::*;

#[tokio::test]
async fn test_workflow_transition_valid() {
    let docker = clients::Cli::default();
    let postgres = docker.run(images::postgres::Postgres::default());

    let db_pool = setup_test_database(&postgres).await;
    let workflow_engine = WorkflowEngine::new(db_pool.clone());

    // Create test kebutuhan in DRAFT state
    let kebutuhan_id = create_test_kebutuhan(&db_pool, "DRAFT").await;

    // Transition to INPUT_BARANG
    let result = workflow_engine.transition(
        kebutuhan_id,
        "DRAFT",
        "INPUT_BARANG",
        Uuid::new_v4(),
        None,
    ).await;

    assert!(result.is_ok());

    // Verify state changed
    let current_state = get_kebutuhan_status(&db_pool, kebutuhan_id).await;
    assert_eq!(current_state, "INPUT_BARANG");
}

#[tokio::test]
async fn test_workflow_transition_invalid() {
    let docker = clients::Cli::default();
    let postgres = docker.run(images::postgres::Postgres::default());

    let db_pool = setup_test_database(&postgres).await;
    let workflow_engine = WorkflowEngine::new(db_pool.clone());

    let kebutuhan_id = create_test_kebutuhan(&db_pool, "DRAFT").await;

    // Try invalid transition: DRAFT -> APPROVED (should fail)
    let result = workflow_engine.transition(
        kebutuhan_id,
        "DRAFT",
        "APPROVED",
        Uuid::new_v4(),
        None,
    ).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WorkflowError::InvalidTransition { .. }));
}
```

### 11.3 Load Tests

**Example - k6 Load Test:**
```javascript
// tests/load/dashboard_load_test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
    stages: [
        { duration: '2m', target: 100 },  // Ramp up to 100 users
        { duration: '5m', target: 100 },  // Stay at 100 users
        { duration: '2m', target: 0 },    // Ramp down
    ],
    thresholds: {
        http_req_duration: ['p(95)<500'],  // 95% of requests < 500ms
    },
};

export default function() {
    let response = http.get('http://localhost:3020/api/v1/dashboard/perlengkapan?tahun_anggaran=2026');

    check(response, {
        'status is 200': (r) => r.status === 200,
        'response time < 500ms': (r) => r.timings.duration < 500,
    });

    sleep(1);
}
```

## 12. Deployment Strategy

### 12.1 Migration Plan

**Phase 1: Database Refactoring (Week 1-2)**
1. Create new standardized tables
2. Migrate existing data
3. Update indexes and views
4. Test data integrity

**Phase 2: Shared Libraries (Week 3-4)**
1. Extract common code to `lib-common`
2. Extract domain logic to `lib-perlengkapan`
3. Extract UI components to `lib-ui`
4. Update all services to use shared libraries

**Phase 3: Integration Services (Week 5-6)**
1. Implement SIMAN integration
2. Implement MySIMKARI integration
3. Test sync logic
4. Deploy integration service

**Phase 4: Workflow Engine (Week 7-8)**
1. Implement workflow engine
2. Migrate existing workflows
3. Test state transitions
4. Deploy workflow engine

**Phase 5: Document & Notification Services (Week 9-10)**
1. Implement document generation
2. Implement notification delivery
3. Test multi-channel notifications
4. Deploy services

**Phase 6: Dashboard Separation (Week 11-12)**
1. Implement portal dashboard
2. Implement perlengkapan dashboard
3. Test real-time updates
4. Deploy dashboards

**Phase 7: Advanced Features (Week 13-14)**
1. Implement search
2. Implement export
3. Implement batch operations
4. Deploy features

**Phase 8: Testing & Optimization (Week 15-16)**
1. Load testing
2. Performance optimization
3. Security audit
4. Final deployment

### 12.2 Rollback Plan

**Database Rollback:**
```sql
-- Backup before migration
pg_dump -h localhost -U simpelv2 perlengkapan > backup_pre_migration.sql

-- Rollback script
BEGIN;
-- Drop new tables
DROP TABLE IF EXISTS perlengkapan.roadmap_sarpras CASCADE;
DROP TABLE IF EXISTS perlengkapan.mapping_kodefikasi CASCADE;
DROP TABLE IF EXISTS perlengkapan.riwayat_pemenuhan CASCADE;
-- Restore old schema
\i backup_pre_migration.sql
COMMIT;
```

**Service Rollback:**
```bash
# Kubernetes rollback
kubectl rollout undo deployment/layanan-perlengkapan -n simpelv2-production

# Docker rollback
docker service update --rollback layanan-perlengkapan
```

## 13. Monitoring & Observability

### 13.1 Metrics

**Prometheus Metrics:**
```rust
// layanan/perlengkapan/crates/api/src/metrics.rs
use prometheus::{
    register_histogram_vec, register_int_counter_vec,
    HistogramVec, IntCounterVec,
};

lazy_static! {
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint", "status"]
    ).unwrap();

    pub static ref WORKFLOW_TRANSITIONS: IntCounterVec = register_int_counter_vec!(
        "workflow_transitions_total",
        "Total workflow transitions",
        &["from_state", "to_state", "success"]
    ).unwrap();

    pub static ref INTEGRATION_SYNC_DURATION: HistogramVec = register_histogram_vec!(
        "integration_sync_duration_seconds",
        "Integration sync duration in seconds",
        &["service", "sync_type"]
    ).unwrap();
}
```

### 13.2 Logging

**Structured Logging:**
```rust
// Use tracing for structured logging
tracing::info!(
    user_id = %user_id,
    kebutuhan_id = %kebutuhan_id,
    from_state = %from_state,
    to_state = %to_state,
    "Workflow transition completed"
);

tracing::error!(
    error = %e,
    service = "siman",
    "Integration sync failed"
);
```

### 13.3 Health Checks

**Health Check Endpoint:**
```rust
// layanan/perlengkapan/crates/api/src/handlers/health.rs
use axum::{extract::State, Json};

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub redis: String,
    pub integrations: HashMap<String, String>,
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Json<HealthStatus> {
    let db_status = check_database(&state.db_pool).await;
    let redis_status = check_redis(&state.redis).await;
    let integration_status = check_integrations(&state.integration_client).await;

    let overall_status = if db_status == "healthy" && redis_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    Json(HealthStatus {
        status: overall_status.to_string(),
        database: db_status,
        redis: redis_status,
        integrations: integration_status,
    })
}
```

## 14. Security Considerations

### 14.1 Authentication & Authorization

- All API endpoints require JWT authentication via Authenc
- Role-based access control (RBAC) for workflow transitions
- Audit logging for all sensitive operations

### 14.2 Data Protection

- Encryption at rest (PostgreSQL encryption)
- Encryption in transit (TLS 1.3)
- Sensitive data stored in Secreton
- PII handling compliance

### 14.3 Input Validation

- All user inputs validated using `lib-common::validation`
- SQL injection prevention via prepared statements
- XSS prevention in frontend
- CSRF protection

## 15. Correctness Properties

### Property 1: Gap Analysis Correctness
**Validates: Requirements 10.1-10.10**

The gap calculation must always equal standard quantity minus existing good condition quantity.

```rust
#[cfg(test)]
mod gap_analysis_properties {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_gap_equals_standard_minus_existing(
            standard_qty in 0i32..1000,
            existing_good in 0i32..1000,
        ) {
            let gap = calculate_gap(standard_qty, existing_good);
            prop_assert_eq!(gap, standard_qty - existing_good);
        }

        #[test]
        fn prop_only_good_condition_counted(
            good_assets in prop::collection::vec(any::<Asset>(), 0..100),
            bad_assets in prop::collection::vec(any::<Asset>(), 0..100),
        ) {
            let all_assets = [good_assets.clone(), bad_assets].concat();
            let counted = count_good_condition_assets(&all_assets);
            prop_assert_eq!(counted, good_assets.len() as i32);
        }
    }
}
```

### Property 2: Workflow Transition Validity
**Validates: Requirements 6.1-6.10**

All workflow transitions must follow configured rules.

```rust
proptest! {
    #[test]
    fn prop_workflow_transitions_follow_rules(
        from_state in workflow_state_strategy(),
        to_state in workflow_state_strategy(),
    ) {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let is_valid = config.is_valid_transition(&from_state, &to_state);

        if is_valid {
            // If valid, transition should succeed
            let result = workflow_engine.transition(entity_id, &from_state, &to_state, user_id, None).await;
            prop_assert!(result.is_ok());
        } else {
            // If invalid, transition should fail
            let result = workflow_engine.transition(entity_id, &from_state, &to_state, user_id, None).await;
            prop_assert!(result.is_err());
        }
    }
}
```

### Property 3: Batch Operation Atomicity
**Validates: Requirements 12.1-12.10**

Each item in a batch operation must be processed independently.

```rust
proptest! {
    #[test]
    fn prop_batch_operations_independent(
        items in prop::collection::vec(any::<Uuid>(), 1..100),
    ) {
        let result = batch_approve(items.clone()).await;

        // Total processed = successful + failed
        prop_assert_eq!(
            result.total,
            result.successful + result.failed
        );

        // Each item appears exactly once in results
        for item_id in items {
            let in_success = result.successful_ids.contains(&item_id);
            let in_failure = result.failures.iter().any(|f| f.id == item_id);
            prop_assert!(in_success ^ in_failure); // XOR: exactly one
        }
    }
}
```

---

## 16. Pemakaian BMN Module Design

### 16.1 Overview

The Pemakaian BMN module manages usage permits for BMN assets (vehicles, housing, laptops). This is a new module (0% complete) that needs full implementation.

**Requirements Coverage:** REQ-P001 to REQ-P016

### 16.2 Database Schema

```sql
-- Izin Pemakaian BMN table
CREATE TABLE perlengkapan.izin_pemakaian_bmn (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bmn_nup VARCHAR(50) NOT NULL,
    bmn_kode_barang VARCHAR(50) NOT NULL,
    bmn_nama_barang VARCHAR(255) NOT NULL,
    jenis_izin VARCHAR(50) NOT NULL, -- KENDARAAN, RUMAH_DINAS, LAPTOP
    pegawai_nip VARCHAR(20) NOT NULL,
    pegawai_nama VARCHAR(255) NOT NULL,
    satker_id UUID NOT NULL REFERENCES authenc.satkers(id),
    tanggal_mulai DATE NOT NULL,
    tanggal_berakhir DATE NOT NULL,
    nomor_izin VARCHAR(100) UNIQUE,
    status VARCHAR(50) NOT NULL, -- DRAFT, SUBMITTED, APPROVED, REJECTED, ACTIVE, EXPIRED, REVOKED
    alasan_penggunaan TEXT,
    dokumen_pendukung_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_tanggal_valid CHECK (tanggal_berakhir > tanggal_mulai)
);

CREATE INDEX idx_izin_bmn_nup ON perlengkapan.izin_pemakaian_bmn(bmn_nup);
CREATE INDEX idx_izin_bmn_pegawai ON perlengkapan.izin_pemakaian_bmn(pegawai_nip);
CREATE INDEX idx_izin_bmn_status ON perlengkapan.izin_pemakaian_bmn(status);
CREATE INDEX idx_izin_bmn_tanggal_berakhir ON perlengkapan.izin_pemakaian_bmn(tanggal_berakhir);

-- Riwayat Izin table
CREATE TABLE perlengkapan.riwayat_izin_pemakaian (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    izin_id UUID NOT NULL REFERENCES perlengkapan.izin_pemakaian_bmn(id),
    status_lama VARCHAR(50),
    status_baru VARCHAR(50) NOT NULL,
    user_id UUID NOT NULL REFERENCES authenc.users(id),
    catatan TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_riwayat_izin ON perlengkapan.riwayat_izin_pemakaian(izin_id);
```

### 16.3 Business Logic

**Dynamic Form Generation:**
```rust
// layanan/perlengkapan/crates/api/src/handlers/pemakaian.rs
use axum::{extract::{State, Path}, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct IzinForm {
    pub jenis_izin: String,
    pub fields: Vec<FormField>,
}

#[derive(Debug, Serialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: String, // text, date, select, file
    pub required: bool,
    pub options: Option<Vec<String>>,
}

pub async fn get_izin_form(
    State(state): State<Arc<AppState>>,
    Path(jenis_izin): Path<String>,
) -> Result<Json<IzinForm>, AppError> {
    let fields = match jenis_izin.as_str() {
        "KENDARAAN" => vec![
            FormField {
                name: "bmn_nup".to_string(),
                label: "NUP Kendaraan".to_string(),
                field_type: "select".to_string(),
                required: true,
                options: Some(get_available_vehicles(&state.db_pool).await?),
            },
            FormField {
                name: "alasan_penggunaan".to_string(),
                label: "Alasan Penggunaan".to_string(),
                field_type: "textarea".to_string(),
                required: true,
                options: None,
            },
            FormField {
                name: "tanggal_mulai".to_string(),
                label: "Tanggal Mulai".to_string(),
                field_type: "date".to_string(),
                required: true,
                options: None,
            },
            FormField {
                name: "tanggal_berakhir".to_string(),
                label: "Tanggal Berakhir".to_string(),
                field_type: "date".to_string(),
                required: true,
                options: None,
            },
        ],
        "RUMAH_DINAS" => vec![
            FormField {
                name: "bmn_nup".to_string(),
                label: "NUP Rumah Dinas".to_string(),
                field_type: "select".to_string(),
                required: true,
                options: Some(get_available_housing(&state.db_pool).await?),
            },
            FormField {
                name: "jumlah_penghuni".to_string(),
                label: "Jumlah Penghuni".to_string(),
                field_type: "number".to_string(),
                required: true,
                options: None,
            },
        ],
        _ => return Err(AppError::InvalidInput("Invalid jenis_izin".to_string())),
    };

    Ok(Json(IzinForm {
        jenis_izin,
        fields,
    }))
}

async fn get_available_vehicles(db_pool: &deadpool_postgres::Pool) -> Result<Vec<String>> {
    let query = r#"
        SELECT sa.nup, sa.nama_barang
        FROM integrasi.siman_aset_angkutan_bermotor sa
        LEFT JOIN perlengkapan.izin_pemakaian_bmn ip
            ON sa.nup = ip.bmn_nup AND ip.status = 'ACTIVE'
        WHERE ip.id IS NULL AND sa.kondisi = 'BAIK'
    "#;

    let client = db_pool.get().await?;
    let rows = client.query(query, &[]).await?;

    Ok(rows.into_iter()
        .map(|row| format!("{} - {}", row.get::<_, String>("nup"), row.get::<_, String>("nama_barang")))
        .collect())
}
```

### 16.4 Permit Expiry Automation

```rust
// layanan/perlengkapan/crates/api/src/jobs/izin_expiry.rs
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct IzinExpiryJob {
    db_pool: deadpool_postgres::Pool,
    notification_client: NotificationGrpcClient,
}

impl IzinExpiryJob {
    pub async fn start(&self) -> Result<()> {
        let scheduler = JobScheduler::new().await?;

        // Check expiry daily at 00:00 WIB
        let expiry_check_job = Job::new_async("0 0 0 * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = self.check_and_expire_permits().await {
                    tracing::error!("Permit expiry check failed: {}", e);
                }
            })
        })?;

        scheduler.add(expiry_check_job).await?;
        scheduler.start().await?;

        Ok(())
    }

    async fn check_and_expire_permits(&self) -> Result<()> {
        tracing::info!("Checking for expired permits");

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET status = 'EXPIRED', updated_at = NOW()
            WHERE status = 'ACTIVE' AND tanggal_berakhir < CURRENT_DATE
            RETURNING id, pegawai_nip, bmn_nama_barang
        "#;

        let client = self.db_pool.get().await?;
        let rows = client.query(query, &[]).await?;

        for row in rows {
            let izin_id: Uuid = row.get("id");
            let pegawai_nip: String = row.get("pegawai_nip");
            let bmn_nama: String = row.get("bmn_nama_barang");

            // Send notification
            self.notification_client
                .send_notification(SendNotificationRequest {
                    user_id: pegawai_nip.clone(),
                    notification_type: "IZIN_EXPIRED".to_string(),
                    notification_data: serde_json::json!({
                        "izin_id": izin_id,
                        "bmn_nama": bmn_nama,
                    }).to_string(),
                    priority: "normal".to_string(),
                })
                .await?;

            tracing::info!("Expired permit {} for pegawai {}", izin_id, pegawai_nip);
        }

        Ok(())
    }
}
```

## 17. MonSAKTI Integration (Optional)

### 17.1 Overview

MonSAKTI integration is optional and provides budget execution monitoring data. This integration is lower priority than SIMAN/MySIMKARI.

**Requirements Coverage:** REQ-I001 (optional), REQ-K009

### 17.2 Database Schema

```sql
-- MonSAKTI Realisasi Anggaran table
CREATE TABLE integrasi.monsakti_realisasi_anggaran (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    satker_id UUID NOT NULL REFERENCES authenc.satkers(id),
    tahun_anggaran INTEGER NOT NULL,
    kode_akun VARCHAR(50) NOT NULL,
    nama_akun VARCHAR(255) NOT NULL,
    pagu DECIMAL(15,2) NOT NULL,
    realisasi DECIMAL(15,2) NOT NULL,
    sisa DECIMAL(15,2) NOT NULL,
    persentase_realisasi DECIMAL(5,2),
    raw_data JSONB NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_monsakti_satker ON integrasi.monsakti_realisasi_anggaran(satker_id);
CREATE INDEX idx_monsakti_tahun ON integrasi.monsakti_realisasi_anggaran(tahun_anggaran);
CREATE INDEX idx_monsakti_akun ON integrasi.monsakti_realisasi_anggaran(kode_akun);
```

### 17.3 API Client

```rust
// layanan/perlengkapan/crates/integrasi/src/clients/monsakti.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MonSAKTIClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct MonSAKTIRealisasi {
    pub satker_code: String,
    pub tahun_anggaran: i32,
    pub kode_akun: String,
    pub nama_akun: String,
    pub pagu: f64,
    pub realisasi: f64,
    pub sisa: f64,
}

impl MonSAKTIClient {
    pub async fn new(secreton_client: &SecretonGrpcClient) -> Result<Self> {
        let api_key = secreton_client
            .get_secret(GetSecretRequest {
                path: "integration/monsakti/api_key".to_string(),
                version: None,
            })
            .await?
            .into_inner()
            .data;

        let api_key = String::from_utf8(api_key)?;

        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            base_url: "https://monsakti.kemenkeu.go.id/api/v1".to_string(),
            api_key,
        })
    }

    pub async fn get_realisasi_anggaran(
        &self,
        satker_code: &str,
        tahun_anggaran: i32,
    ) -> Result<Vec<MonSAKTIRealisasi>> {
        let response = self.client
            .get(&format!("{}/realisasi", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("satker_code", satker_code),
                ("tahun", &tahun_anggaran.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(IntegrationError::ApiError(format!(
                "MonSAKTI API returned status: {}",
                response.status()
            )));
        }

        let realisasi: Vec<MonSAKTIRealisasi> = response.json().await?;
        Ok(realisasi)
    }
}
```

## 18. SK Penghapusan BMN Generation

### 18.1 Overview

SK (Surat Keputusan) Penghapusan BMN is an official document for BMN deletion/disposal. This feature generates formatted PDF documents with official letterhead.

**Requirements Coverage:** REQ-D002

### 18.2 Document Template

```rust
// layanan/perlengkapan/crates/dokumen/src/templates/sk_penghapusan.rs
use tera::{Tera, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SkPenghapusanData {
    pub nomor_sk: String,
    pub tanggal_sk: String,
    pub satker_nama: String,
    pub satker_alamat: String,
    pub pejabat_nama: String,
    pub pejabat_jabatan: String,
    pub bmn_list: Vec<BmnPenghapusan>,
    pub alasan_penghapusan: String,
    pub dasar_hukum: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BmnPenghapusan {
    pub nup: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub tahun_perolehan: i32,
    pub nilai_perolehan: f64,
    pub kondisi: String,
    pub alasan: String,
}

pub struct SkPenghapusanGenerator {
    tera: Tera,
}

impl SkPenghapusanGenerator {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("sk_penghapusan", SK_PENGHAPUSAN_TEMPLATE)?;
        Ok(Self { tera })
    }

    pub fn generate_html(&self, data: &SkPenghapusanData) -> Result<String> {
        let mut context = Context::new();
        context.insert("data", data);

        let html = self.tera.render("sk_penghapusan", &context)?;
        Ok(html)
    }
}

const SK_PENGHAPUSAN_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        @page { size: A4; margin: 2cm; }
        body { font-family: 'Times New Roman', serif; font-size: 12pt; }
        .header { text-align: center; margin-bottom: 2cm; }
        .logo { width: 100px; }
        .title { font-weight: bold; text-transform: uppercase; margin-top: 1cm; }
        .nomor { margin-top: 0.5cm; }
        .content { text-align: justify; line-height: 1.5; }
        .table { width: 100%; border-collapse: collapse; margin: 1cm 0; }
        .table th, .table td { border: 1px solid black; padding: 8px; }
        .signature { margin-top: 2cm; float: right; text-align: center; }
    </style>
</head>
<body>
    <div class="header">
        <img src="/assets/logo-kejaksaan.png" class="logo" alt="Logo Kejaksaan RI">
        <h2>KEJAKSAAN REPUBLIK INDONESIA</h2>
        <h3>{{ data.satker_nama }}</h3>
        <p>{{ data.satker_alamat }}</p>
    </div>

    <div class="title">
        <p>SURAT KEPUTUSAN</p>
        <p>TENTANG</p>
        <p>PENGHAPUSAN BARANG MILIK NEGARA</p>
    </div>

    <div class="nomor">
        <p>Nomor: {{ data.nomor_sk }}</p>
    </div>

    <div class="content">
        <p><strong>MENIMBANG:</strong></p>
        <ol type="a">
            <li>Bahwa {{ data.alasan_penghapusan }}</li>
            <li>Bahwa berdasarkan pertimbangan sebagaimana dimaksud huruf a, perlu menetapkan Surat Keputusan tentang Penghapusan Barang Milik Negara</li>
        </ol>

        <p><strong>MENGINGAT:</strong></p>
        <ol>
            {% for dasar in data.dasar_hukum %}
            <li>{{ dasar }}</li>
            {% endfor %}
        </ol>

        <p><strong>MEMUTUSKAN:</strong></p>
        <p><strong>MENETAPKAN:</strong></p>

        <p><strong>KESATU:</strong> Menghapus Barang Milik Negara sebagaimana tercantum dalam lampiran Surat Keputusan ini.</p>

        <p><strong>KEDUA:</strong> Surat Keputusan ini mulai berlaku pada tanggal ditetapkan.</p>

        <table class="table">
            <thead>
                <tr>
                    <th>No</th>
                    <th>NUP</th>
                    <th>Kode Barang</th>
                    <th>Nama Barang</th>
                    <th>Tahun Perolehan</th>
                    <th>Nilai Perolehan</th>
                    <th>Kondisi</th>
                    <th>Alasan</th>
                </tr>
            </thead>
            <tbody>
                {% for bmn in data.bmn_list %}
                <tr>
                    <td>{{ loop.index }}</td>
                    <td>{{ bmn.nup }}</td>
                    <td>{{ bmn.kode_barang }}</td>
                    <td>{{ bmn.nama_barang }}</td>
                    <td>{{ bmn.tahun_perolehan }}</td>
                    <td>Rp {{ bmn.nilai_perolehan | number_format }}</td>
                    <td>{{ bmn.kondisi }}</td>
                    <td>{{ bmn.alasan }}</td>
                </tr>
                {% endfor %}
            </tbody>
        </table>
    </div>

    <div class="signature">
        <p>Ditetapkan di: {{ data.satker_nama }}</p>
        <p>Pada tanggal: {{ data.tanggal_sk }}</p>
        <p>{{ data.pejabat_jabatan }}</p>
        <br><br><br>
        <p><strong>{{ data.pejabat_nama }}</strong></p>
    </div>
</body>
</html>
"#;
```

## 19. Roadmap Sarpras Business Logic

### 19.1 Overview

Roadmap Sarpras provides 5-year infrastructure planning with realization tracking. The database schema exists but business logic needs implementation.

**Requirements Coverage:** REQ-K008, REQ-DB003

### 19.2 Roadmap Creation Logic

```rust
// lib/perlengkapan/src/roadmap.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapSarpras {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub periode_mulai: i32,
    pub periode_akhir: i32,
    pub items: Vec<RoadmapItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItem {
    pub kode_barang: String,
    pub nama_barang: String,
    pub tahun_rencana: i32,
    pub jumlah_kebutuhan: i32,
    pub estimasi_anggaran: f64,
    pub prioritas: i32, // 1-5, 1 = highest
}

pub struct RoadmapEngine {
    db_pool: deadpool_postgres::Pool,
}

impl RoadmapEngine {
    pub async fn create_roadmap(
        &self,
        satker_id: Uuid,
        periode_mulai: i32,
        periode_akhir: i32,
        items: Vec<RoadmapItem>,
    ) -> Result<Uuid> {
        // Validate period (must be 5 years)
        if periode_akhir - periode_mulai != 4 {
            return Err(RoadmapError::InvalidPeriod("Period must be exactly 5 years".to_string()));
        }

        // Validate all items are within period
        for item in &items {
            if item.tahun_rencana < periode_mulai || item.tahun_rencana > periode_akhir {
                return Err(RoadmapError::InvalidYear(format!(
                    "Year {} is outside period {}-{}",
                    item.tahun_rencana, periode_mulai, periode_akhir
                )));
            }
        }

        let client = self.db_pool.get().await?;
        let tx = client.transaction().await?;

        // Insert roadmap items
        for item in items {
            let query = r#"
                INSERT INTO perlengkapan.roadmap_sarpras
                (satker_id, periode_mulai, periode_akhir, kode_barang, tahun_rencana,
                 jumlah_kebutuhan, estimasi_anggaran, status_pemenuhan)
                VALUES ($1, $2, $3, $4, $5, $6, $7, 'PLANNED')
            "#;

            tx.execute(
                query,
                &[
                    &satker_id,
                    &periode_mulai,
                    &periode_akhir,
                    &item.kode_barang,
                    &item.tahun_rencana,
                    &item.jumlah_kebutuhan,
                    &item.estimasi_anggaran,
                ],
            ).await?;
        }

        tx.commit().await?;
        Ok(Uuid::new_v4())
    }

    pub async fn update_realization(
        &self,
        roadmap_id: Uuid,
        tahun_anggaran: i32,
        jumlah_terpenuhi: i32,
        realisasi_anggaran: f64,
    ) -> Result<()> {
        let query = r#"
            UPDATE perlengkapan.roadmap_sarpras
            SET jumlah_terpenuhi = jumlah_terpenuhi + $1,
                realisasi_anggaran = COALESCE(realisasi_anggaran, 0) + $2,
                status_pemenuhan = CASE
                    WHEN jumlah_terpenuhi + $1 >= jumlah_kebutuhan THEN 'COMPLETED'
                    WHEN jumlah_terpenuhi + $1 > 0 THEN 'PARTIAL'
                    ELSE 'PLANNED'
                END,
                updated_at = NOW()
            WHERE id = $3 AND tahun_rencana = $4
        "#;

        let client = self.db_pool.get().await?;
        client.execute(query, &[&jumlah_terpenuhi, &realisasi_anggaran, &roadmap_id, &tahun_anggaran]).await?;

        Ok(())
    }

    pub async fn get_roadmap_vs_realization(
        &self,
        satker_id: Uuid,
        periode_mulai: i32,
        periode_akhir: i32,
    ) -> Result<Vec<RoadmapComparison>> {
        let query = r#"
            SELECT
                tahun_rencana,
                kode_barang,
                nama_barang,
                jumlah_kebutuhan,
                jumlah_terpenuhi,
                estimasi_anggaran,
                realisasi_anggaran,
                status_pemenuhan,
                ROUND((jumlah_terpenuhi::DECIMAL / jumlah_kebutuhan * 100), 2) as persentase_pemenuhan
            FROM perlengkapan.roadmap_sarpras
            WHERE satker_id = $1
              AND periode_mulai = $2
              AND periode_akhir = $3
            ORDER BY tahun_rencana, kode_barang
        "#;

        let client = self.db_pool.get().await?;
        let rows = client.query(query, &[&satker_id, &periode_mulai, &periode_akhir]).await?;

        let comparisons = rows.into_iter().map(|row| RoadmapComparison {
            tahun_rencana: row.get("tahun_rencana"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            jumlah_kebutuhan: row.get("jumlah_kebutuhan"),
            jumlah_terpenuhi: row.get("jumlah_terpenuhi"),
            estimasi_anggaran: row.get("estimasi_anggaran"),
            realisasi_anggaran: row.get("realisasi_anggaran"),
            status_pemenuhan: row.get("status_pemenuhan"),
            persentase_pemenuhan: row.get("persentase_pemenuhan"),
        }).collect();

        Ok(comparisons)
    }
}

#[derive(Debug, Serialize)]
pub struct RoadmapComparison {
    pub tahun_rencana: i32,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah_kebutuhan: i32,
    pub jumlah_terpenuhi: i32,
    pub estimasi_anggaran: f64,
    pub realisasi_anggaran: Option<f64>,
    pub status_pemenuhan: String,
    pub persentase_pemenuhan: f64,
}
```

## 20. Mapping Kodefikasi UI Workflow

### 20.1 Overview

Mapping Kodefikasi provides a UI workflow for mapping non-standard kode barang from MonSAKTI to standard codes. This includes auto-detection, proposal, and verification.

**Requirements Coverage:** REQ-M007, REQ-M008, REQ-M009

### 20.2 Auto-Detection Logic

```rust
// layanan/perlengkapan/crates/api/src/handlers/mapping_kodefikasi.rs
use axum::{extract::State, Json};

pub struct MappingDetector {
    db_pool: deadpool_postgres::Pool,
}

impl MappingDetector {
    pub async fn detect_non_standard_codes(&self) -> Result<Vec<NonStandardCode>> {
        let query = r#"
            SELECT DISTINCT
                sa.kode_barang as kode_lama,
                sa.nama_barang as nama_lama,
                sa.satker_id,
                COUNT(*) as jumlah_aset
            FROM integrasi.siman_aset_tanah sa
            LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
            WHERE mb.id IS NULL
            GROUP BY sa.kode_barang, sa.nama_barang, sa.satker_id
            ORDER BY jumlah_aset DESC
        "#;

        let client = self.db_pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let non_standard = rows.into_iter().map(|row| NonStandardCode {
            kode_lama: row.get("kode_lama"),
            nama_lama: row.get("nama_lama"),
            satker_id: row.get("satker_id"),
            jumlah_aset: row.get("jumlah_aset"),
            suggested_mapping: None,
        }).collect();

        Ok(non_standard)
    }

    pub async fn suggest_mapping(&self, kode_lama: &str, nama_lama: &str) -> Result<Vec<MappingSuggestion>> {
        // Use fuzzy matching to suggest standard codes
        let query = r#"
            SELECT
                id,
                kode,
                nama,
                similarity(nama, $1) as score
            FROM perlengkapan.ms_barang
            WHERE similarity(nama, $1) > 0.3
            ORDER BY score DESC
            LIMIT 5
        "#;

        let client = self.db_pool.get().await?;
        let rows = client.query(query, &[&nama_lama]).await?;

        let suggestions = rows.into_iter().map(|row| MappingSuggestion {
            barang_id: row.get("id"),
            kode_baru: row.get("kode"),
            nama_baru: row.get("nama"),
            similarity_score: row.get("score"),
        }).collect();

        Ok(suggestions)
    }
}

#[derive(Debug, Serialize)]
pub struct NonStandardCode {
    pub kode_lama: String,
    pub nama_lama: String,
    pub satker_id: Uuid,
    pub jumlah_aset: i64,
    pub suggested_mapping: Option<MappingSuggestion>,
}

#[derive(Debug, Serialize)]
pub struct MappingSuggestion {
    pub barang_id: Uuid,
    pub kode_baru: String,
    pub nama_baru: String,
    pub similarity_score: f64,
}
```

### 20.3 Mapping Proposal & Verification

```rust
// Proposal submission
pub async fn submit_mapping_proposal(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MappingProposalRequest>,
) -> Result<Json<MappingProposal>, AppError> {
    let proposal_id = Uuid::new_v4();

    let query = r#"
        INSERT INTO perlengkapan.mapping_kodefikasi
        (id, satker_id, kode_barang_lama, nama_barang_lama, kode_barang_baru_id,
         status_mapping, catatan_mapping, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'PROPOSED', $6, NOW(), NOW())
        RETURNING *
    "#;

    let client = state.db_pool.get().await?;
    let row = client.query_one(
        query,
        &[
            &proposal_id,
            &request.satker_id,
            &request.kode_lama,
            &request.nama_lama,
            &request.kode_baru_id,
            &request.catatan,
        ],
    ).await?;

    // Send notification to Admin Pusat for verification
    state.notification_client
        .send_notification(SendNotificationRequest {
            user_id: "admin-pusat".to_string(),
            notification_type: "MAPPING_PROPOSAL".to_string(),
            notification_data: serde_json::json!({
                "proposal_id": proposal_id,
                "kode_lama": request.kode_lama,
                "kode_baru_id": request.kode_baru_id,
            }).to_string(),
            priority: "normal".to_string(),
        })
        .await?;

    Ok(Json(row.into()))
}

// Verification by Admin Pusat
pub async fn verify_mapping_proposal(
    State(state): State<Arc<AppState>>,
    Path(proposal_id): Path<Uuid>,
    Json(request): Json<VerifyMappingRequest>,
) -> Result<Json<MappingProposal>, AppError> {
    let new_status = if request.approved {
        "VERIFIED"
    } else {
        "REJECTED"
    };

    let query = r#"
        UPDATE perlengkapan.mapping_kodefikasi
        SET status_mapping = $1,
            catatan_mapping = COALESCE($2, catatan_mapping),
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
    "#;

    let client = state.db_pool.get().await?;
    let row = client.query_one(
        query,
        &[&new_status, &request.catatan_verifikasi, &proposal_id],
    ).await?;

    // If approved, update all assets with old code to use new code
    if request.approved {
        apply_mapping(&state.db_pool, proposal_id).await?;
    }

    Ok(Json(row.into()))
}

async fn apply_mapping(db_pool: &deadpool_postgres::Pool, proposal_id: Uuid) -> Result<()> {
    let query = r#"
        UPDATE integrasi.siman_aset_tanah sa
        SET kode_barang = mb.kode
        FROM perlengkapan.mapping_kodefikasi mk
        JOIN perlengkapan.ms_barang mb ON mk.kode_barang_baru_id = mb.id
        WHERE mk.id = $1
          AND sa.kode_barang = mk.kode_barang_lama
          AND sa.satker_id = mk.satker_id
    "#;

    let client = db_pool.get().await?;
    client.execute(query, &[&proposal_id]).await?;

    Ok(())
}
```

### 20.4 Mapping Progress Dashboard

```rust
// Frontend component
// antarmuka/perlengkapan/src/pages/mapping_kodefikasi.rs
use leptos::prelude::*;

#[component]
pub fn MappingKodefikasiDashboard() -> impl IntoView {
    let progress = Resource::new(
        || (),
        |_| async move {
            gloo_net::http::Request::get("/api/v1/mapping/progress")
                .send()
                .await?
                .json::<MappingProgress>()
                .await
        }
    );

    view! {
        <div class="mapping-dashboard">
            <h1 class="text-2xl font-bold mb-6">"Mapping Kodefikasi Progress"</h1>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || progress.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                            <MetricCard
                                title="Total Non-Standard Codes".to_string()
                                value=data.total_non_standard.to_string()
                                change=None
                                icon="⚠️".to_string()
                            />
                            <MetricCard
                                title="Mapped".to_string()
                                value=data.total_mapped.to_string()
                                change=Some(data.mapping_percentage)
                                icon="✅".to_string()
                            />
                            <MetricCard
                                title="Pending Verification".to_string()
                                value=data.pending_verification.to_string()
                                change=None
                                icon="⏳".to_string()
                            />
                        </div>

                        <div class="mt-8">
                            <h2 class="text-xl font-bold mb-4">"Non-Standard Codes"</h2>
                            <MappingTable items=data.non_standard_codes />
                        </div>
                    }.into_any(),
                    Err(e) => view! { <ErrorDisplay error=e.to_string() /> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
```

---

**End of Design Document**
**Document Version:** 2.0.0
**Last Updated:** February 9, 2026
**Status:** Complete - Ready for Implementation


## 21. Kebutuhan BMN Business Process Flow

### 21.1 Overview

This section provides detailed business process flows for the Kebutuhan BMN module, covering the complete workflow from period initiation by Validator Pusat through submission by Operator Satker, review by Validator Wilayah, and final analysis and approval by Validator Pusat.

**Key Requirements Addressed:** REQ-K001 through REQ-K020

### 21.2 Actors and Roles

| Actor | Role | Responsibilities |
|-------|------|------------------|
| Validator Pusat | Central validator at Kejaksaan Agung | Initiate period, configure eligible BMN/satkers, analyze with SIMAN/MySIMKARI data, approve/reject |
| Validator Wilayah | Regional validator at Kejaksaan Tinggi | Review submissions from satkers, forward to Pusat or return for revision |
| Operator Satker | BMN manager at Kejari/Cabjari | Submit kebutuhan BMN with justification and supporting documents |

### 21.3 Workflow States

```
DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → APPROVED/REJECTED
```

**State Definitions:**
- `DRAFT`: Initial state, operator satker is preparing submission
- `SUBMITTED`: Operator satker has submitted to validator wilayah
- `REVIEWED_WILAYAH`: Validator wilayah has reviewed and forwarded to validator pusat
- `REVIEWED_PUSAT`: Validator pusat is analyzing with integrated data
- `APPROVED`: Validator pusat has approved (marked as priority)
- `REJECTED`: Validator pusat has rejected

### 21.4 Sequence Diagram: Complete Kebutuhan BMN Flow

```mermaid
sequenceDiagram
    participant VP as Validator Pusat
    participant VW as Validator Wilayah
    participant OS as Operator Satker
    participant API as layanan-perlengkapan API
    participant WF as Workflow Service
    participant INT as Integrasi Service
    participant SIMAN as SIMAN API
    participant MySIMKARI as MySIMKARI API
    participant DOK as Dokumen Service
    participant NOT as Notifikasi Service

    Note over VP,NOT: Phase 1: Period Initiation by Validator Pusat
    VP->>API: POST /api/v1/kebutuhan/periods
    API->>API: Create period (start_date, end_date, deadline)
    VP->>API: POST /api/v1/kebutuhan/periods/{id}/eligible-bmn
    API->>API: Configure eligible BMN items (filtered by standar kodefikasi)
    VP->>API: POST /api/v1/kebutuhan/periods/{id}/eligible-satkers
    API->>API: Configure eligible satkers
    API->>NOT: Send notification to eligible satkers
    NOT-->>OS: Notification: "Kebutuhan BMN period opened"

    Note over VP,NOT: Phase 2: Submission by Operator Satker
    OS->>API: GET /api/v1/kebutuhan/periods/active
    API-->>OS: Return active period with constraints
    OS->>API: POST /api/v1/kebutuhan/submissions
    API->>API: Validate: within deadline, eligible BMN, eligible satker
    API->>WF: Create workflow instance (state: DRAFT)
    OS->>API: POST /api/v1/kebutuhan/submissions/{id}/items
    API->>API: Add BMN items with justification
    OS->>API: POST /api/v1/kebutuhan/submissions/{id}/attachments
    API->>API: Upload supporting documents (surat permohonan, etc.)
    OS->>API: POST /api/v1/kebutuhan/submissions/{id}/submit
    API->>WF: Transition: DRAFT → SUBMITTED
    WF->>NOT: Send notification to Validator Wilayah
    NOT-->>VW: Notification: "New kebutuhan BMN submission"

    Note over VP,NOT: Phase 3: Review by Validator Wilayah
    VW->>API: GET /api/v1/kebutuhan/submissions?status=SUBMITTED
    API-->>VW: Return pending submissions
    VW->>API: GET /api/v1/kebutuhan/submissions/{id}
    API-->>VW: Return submission details

    alt Forward to Validator Pusat
        VW->>API: POST /api/v1/kebutuhan/submissions/{id}/forward
        API->>WF: Transition: SUBMITTED → REVIEWED_WILAYAH
        WF->>NOT: Send notification to Validator Pusat
        NOT-->>VP: Notification: "Kebutuhan BMN ready for analysis"
    else Return to Operator Satker for Revision
        VW->>API: POST /api/v1/kebutuhan/submissions/{id}/return
        API->>WF: Transition: SUBMITTED → DRAFT (with revision notes)
        WF->>NOT: Send notification to Operator Satker
        NOT-->>OS: Notification: "Revision required"
        Note over OS: Operator revises and resubmits (back to Phase 2)
    end

    Note over VP,NOT: Phase 4: Analysis by Validator Pusat with Integrated Data
    VP->>API: GET /api/v1/kebutuhan/submissions?status=REVIEWED_WILAYAH
    API-->>VP: Return submissions ready for analysis
    VP->>API: GET /api/v1/kebutuhan/submissions/{id}/analysis
    API->>INT: gRPC: GetSimanAssets(satker_id, kode_barang)
    INT->>SIMAN: GET /api/v2/assets?satker_code=...
    SIMAN-->>INT: Return existing BMN data
    INT-->>API: Return BMN data (kondisi, jumlah)
    API->>INT: gRPC: GetMySIMKARIPegawaiSummary(satker_id)
    INT->>MySIMKARI: GET /api/v1/pegawai/summary?satker_code=...
    MySIMKARI-->>INT: Return pegawai summary (eselon count, golongan breakdown)
    INT-->>API: Return pegawai summary
    API-->>VP: Return analysis data (existing BMN + pegawai summary)

    alt Approve
        VP->>API: POST /api/v1/kebutuhan/submissions/{id}/approve
        API->>WF: Transition: REVIEWED_WILAYAH → APPROVED
        API->>API: Mark as priority for further processing
        WF->>NOT: Send notification to Operator Satker
        NOT-->>OS: Notification: "Kebutuhan BMN approved"
    else Reject
        VP->>API: POST /api/v1/kebutuhan/submissions/{id}/reject
        API->>WF: Transition: REVIEWED_WILAYAH → REJECTED
        WF->>NOT: Send notification to Operator Satker
        NOT-->>OS: Notification: "Kebutuhan BMN rejected"
    end

    Note over VP,NOT: Phase 5: Analysis Report Generation
    VP->>API: POST /api/v1/kebutuhan/reports/generate
    API->>DOK: gRPC: GenerateDocument(type: analysis_report, format: PDF/DOCX/XLSX)
    DOK->>DOK: Generate report with analysis data
    DOK-->>API: Return document_id
    API->>NOT: Send notification to Validator Pusat
    NOT-->>VP: Notification: "Analysis report ready"
    VP->>API: GET /api/v1/documents/{document_id}/download
    API->>DOK: gRPC: GetDocument(document_id)
    DOK-->>API: Return document binary
    API-->>VP: Download PDF/DOCX/XLSX
```

### 21.5 REST API Endpoints

**Period Management (Validator Pusat only):**
```
POST   /api/v1/kebutuhan/periods
GET    /api/v1/kebutuhan/periods
GET    /api/v1/kebutuhan/periods/{id}
PUT    /api/v1/kebutuhan/periods/{id}
POST   /api/v1/kebutuhan/periods/{id}/eligible-bmn
POST   /api/v1/kebutuhan/periods/{id}/eligible-satkers
GET    /api/v1/kebutuhan/periods/active
```

**Submission Management:**
```
POST   /api/v1/kebutuhan/submissions
GET    /api/v1/kebutuhan/submissions
GET    /api/v1/kebutuhan/submissions/{id}
PUT    /api/v1/kebutuhan/submissions/{id}
POST   /api/v1/kebutuhan/submissions/{id}/items
POST   /api/v1/kebutuhan/submissions/{id}/attachments
POST   /api/v1/kebutuhan/submissions/{id}/submit
POST   /api/v1/kebutuhan/submissions/{id}/forward
POST   /api/v1/kebutuhan/submissions/{id}/return
POST   /api/v1/kebutuhan/submissions/{id}/approve
POST   /api/v1/kebutuhan/submissions/{id}/reject
GET    /api/v1/kebutuhan/submissions/{id}/analysis
```

**Report Generation:**
```
POST   /api/v1/kebutuhan/reports/generate
GET    /api/v1/kebutuhan/reports
```

### 21.6 Database Schema

**Period Configuration:**
```sql
CREATE TABLE perlengkapan.kebutuhan_bmn_period (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama_period VARCHAR(255) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    deadline TIMESTAMPTZ NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE', -- ACTIVE, CLOSED
    created_by UUID NOT NULL REFERENCES authenc.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE perlengkapan.kebutuhan_bmn_period_eligible_bmn (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_id UUID NOT NULL REFERENCES perlengkapan.kebutuhan_bmn_period(id),
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE perlengkapan.kebutuhan_bmn_period_eligible_satker (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_id UUID NOT NULL REFERENCES perlengkapan.kebutuhan_bmn_period(id),
    satker_id UUID NOT NULL REFERENCES authenc.satkers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Submission Data:**
```sql
CREATE TABLE perlengkapan.kebutuhan_bmn_submission (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_id UUID NOT NULL REFERENCES perlengkapan.kebutuhan_bmn_period(id),
    satker_id UUID NOT NULL REFERENCES authenc.satkers(id),
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT',
    workflow_instance_id UUID,
    submitted_at TIMESTAMPTZ,
    reviewed_wilayah_at TIMESTAMPTZ,
    reviewed_pusat_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    is_priority BOOLEAN DEFAULT FALSE,
    created_by UUID NOT NULL REFERENCES authenc.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE perlengkapan.kebutuhan_bmn_submission_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES perlengkapan.kebutuhan_bmn_submission(id),
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,
    jumlah_kebutuhan INTEGER NOT NULL,
    justification TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE perlengkapan.kebutuhan_bmn_submission_attachment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES perlengkapan.kebutuhan_bmn_submission(id),
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES authenc.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 21.7 Integration Points

**SIMAN Integration (via Integrasi Service gRPC):**
```protobuf
service IntegrasiService {
    rpc GetSimanAssets(GetSimanAssetsRequest) returns (GetSimanAssetsResponse);
}

message GetSimanAssetsRequest {
    string satker_id = 1;
    string kode_barang = 2;
}

message GetSimanAssetsResponse {
    repeated SimanAsset assets = 1;
}

message SimanAsset {
    string nup = 1;
    string kode_barang = 2;
    string nama_barang = 3;
    string kondisi = 4; // BAIK, RUSAK_RINGAN, RUSAK_BERAT
    int32 tahun_perolehan = 5;
    double nilai_perolehan = 6;
}
```

**MySIMKARI Integration (via Integrasi Service gRPC):**
```protobuf
service IntegrasiService {
    rpc GetMySIMKARIPegawaiSummary(GetPegawaiSummaryRequest) returns (GetPegawaiSummaryResponse);
}

message GetPegawaiSummaryRequest {
    string satker_id = 1;
}

message GetPegawaiSummaryResponse {
    int32 total_pegawai = 1;
    int32 total_eselon = 2;
    repeated GolonganBreakdown golongan_breakdown = 3;
    int32 total_jaksa = 4;
    int32 total_non_jaksa = 5;
}

message GolonganBreakdown {
    string golongan = 1; // I, II, III, IV
    int32 count = 2;
}
```

### 21.8 Validation Rules

**Period Validation:**
- Start date must be before end date
- Deadline must be after start date
- Only one active period allowed at a time
- Eligible BMN must exist in ms_barang
- Eligible satkers must exist in authenc.satkers

**Submission Validation:**
- Must be within active period
- Submission date must be before deadline
- Satker must be in eligible satkers list
- BMN items must be in eligible BMN list
- Justification must be at least 50 characters
- At least one supporting document required

**Workflow Transition Validation:**
- DRAFT → SUBMITTED: Requires at least one item and one attachment
- SUBMITTED → REVIEWED_WILAYAH: Only Validator Wilayah can forward
- SUBMITTED → DRAFT: Only Validator Wilayah can return with notes
- REVIEWED_WILAYAH → APPROVED/REJECTED: Only Validator Pusat can approve/reject


## 22. Pemakaian BMN Business Process Flow

### 22.1 Overview

This section provides detailed business process flows for the Pemakaian BMN (BMN Usage Permit) module, covering permit creation, document generation, expiry management, renewal, and revocation workflows.

**Key Requirements Addressed:** REQ-P001 through REQ-P028

### 22.2 Actors and Roles

| Actor | Role | Responsibilities |
|-------|------|------------------|
| Operator Satker | BMN manager at Kejari/Cabjari | Create permit requests, upload signed permits |
| Validator Wilayah | Regional validator at Kejaksaan Tinggi | Monitor permits in region |
| Validator Pusat | Central validator at Kejaksaan Agung | Monitor all permits, view reports |
| Pimpinan Satker | Head of Kejari/Cabjari | Sign permit documents |

### 22.3 Workflow States

```
DRAFT → DOCUMENT_GENERATED → COMPLETED → EXPIRED/REVOKED
```

**State Definitions:**
- `DRAFT`: Operator satker is preparing permit request
- `DOCUMENT_GENERATED`: System has generated DOCX draft permit
- `COMPLETED`: Operator satker has uploaded signed PDF permit
- `EXPIRED`: Permit has passed end date (auto-transition)
- `REVOKED`: Admin has revoked permit before expiry

### 22.4 Sequence Diagram: Pemakaian BMN Permit Creation Flow

```mermaid
sequenceDiagram
    participant OS as Operator Satker
    participant API as layanan-perlengkapan API
    participant INT as Integrasi Service
    participant SIMAN as SIMAN API
    participant MySIMKARI as MySIMKARI API
    participant DOK as Dokumen Service
    participant NOT as Notifikasi Service
    participant PS as Pimpinan Satker

    Note over OS,PS: Phase 1: Permit Request Creation
    OS->>API: POST /api/v1/pemakaian/permits
    API->>API: Create permit (state: DRAFT)
    OS->>API: POST /api/v1/pemakaian/permits/{id}/pegawai
    API->>INT: gRPC: GetMySIMKARIPegawai(nip)
    INT->>MySIMKARI: GET /api/v1/pegawai/{nip}
    MySIMKARI-->>INT: Return pegawai data (nama, jabatan, pangkat, golongan, photo)
    INT-->>API: Return pegawai data
    API->>API: Validate: pegawai exists, belongs to satker
    API-->>OS: Pegawai added to permit

    Note over OS,PS: Phase 2: BMN Selection with Availability Check
    OS->>API: GET /api/v1/pemakaian/bmn/available?satker_id=...
    API->>INT: gRPC: GetSimanAssets(satker_id)
    INT->>SIMAN: GET /api/v2/assets?satker_code=...
    SIMAN-->>INT: Return BMN list
    INT-->>API: Return BMN list
    API->>API: Check active permits (filter out BMN with active permits)
    API-->>OS: Return available BMN list

    OS->>API: POST /api/v1/pemakaian/permits/{id}/bmn
    API->>API: Validate: BMN not in active permit (one BMN = one active permit)
    API->>API: Add BMN to permit
    Note over OS: Operator can add multiple BMN for single pegawai
    OS->>API: POST /api/v1/pemakaian/permits/{id}/bmn (repeat for each BMN)

    OS->>API: POST /api/v1/pemakaian/permits/{id}/period
    API->>API: Set usage period (start_date, end_date)

    Note over OS,PS: Phase 3: Document Generation
    OS->>API: POST /api/v1/pemakaian/permits/{id}/generate-document
    API->>DOK: gRPC: GenerateDocument(type: izin_pemakaian, format: DOCX)
    DOK->>DOK: Generate DOCX with:
    DOK->>DOK: - Page 1: Pegawai identity + photo
    DOK->>DOK: - Page 2+: BMN details table
    DOK->>DOK: - Auto-generate permit number: IZN/{YEAR}/{SATKER}/{SEQUENCE}
    DOK-->>API: Return document_id
    API->>API: Transition: DRAFT → DOCUMENT_GENERATED
    API->>NOT: Send notification to Operator Satker
    NOT-->>OS: Notification: "Draft permit document ready"
    OS->>API: GET /api/v1/documents/{document_id}/download
    API->>DOK: gRPC: GetDocument(document_id)
    DOK-->>API: Return DOCX binary
    API-->>OS: Download DOCX draft

    Note over OS,PS: Phase 4: Signature and Upload
    OS->>PS: Send DOCX for signature (offline process)
    PS->>PS: Review and sign document
    PS->>OS: Return signed PDF

    OS->>API: POST /api/v1/pemakaian/permits/{id}/upload-signed
    API->>API: Upload signed PDF to object storage
    API->>API: Transition: DOCUMENT_GENERATED → COMPLETED
    API->>NOT: Send notification to Validator Wilayah, Validator Pusat
    NOT-->>OS: Notification: "Permit completed"

    Note over OS,PS: Phase 5: Expiry Management (Automated)
    loop Daily Cron Job
        API->>API: Check permits expiring in 30 days
        API->>NOT: Send reminder notification (H-30)
        NOT-->>OS: Notification: "Permit expiring in 30 days"
        API->>API: Check permits expiring in 14 days
        API->>NOT: Send reminder notification (H-14)
        NOT-->>OS: Notification: "Permit expiring in 14 days"
        API->>API: Check permits expiring in 7 days
        API->>NOT: Send reminder notification (H-7)
        NOT-->>OS: Notification: "Permit expiring in 7 days"
        API->>API: Check permits past end_date
        API->>API: Transition: COMPLETED → EXPIRED
        API->>NOT: Send notification
        NOT-->>OS: Notification: "Permit expired"
    end
```

### 22.5 Sequence Diagram: Permit Renewal Flow

```mermaid
sequenceDiagram
    participant OS as Operator Satker
    participant API as layanan-perlengkapan API
    participant DOK as Dokumen Service
    participant NOT as Notifikasi Service

    Note over OS,NOT: Permit Renewal Process
    OS->>API: POST /api/v1/pemakaian/permits/{old_permit_id}/renew
    API->>API: Validate: old permit exists and is COMPLETED or EXPIRED
    API->>API: Create new permit (copy pegawai, BMN from old permit)
    API->>API: Link to previous permit (previous_permit_id)
    API->>API: Set new usage period
    API->>API: State: DRAFT
    API-->>OS: Return new permit_id

    OS->>API: POST /api/v1/pemakaian/permits/{new_permit_id}/generate-document
    API->>DOK: gRPC: GenerateDocument(type: izin_pemakaian_renewal)
    DOK->>DOK: Generate DOCX with renewal note
    DOK-->>API: Return document_id
    API->>API: Transition: DRAFT → DOCUMENT_GENERATED
    API-->>OS: Download DOCX draft

    Note over OS: Signature process (same as creation)
    OS->>API: POST /api/v1/pemakaian/permits/{new_permit_id}/upload-signed
    API->>API: Transition: DOCUMENT_GENERATED → COMPLETED
    API->>NOT: Send notification
    NOT-->>OS: Notification: "Permit renewed"
```

### 22.6 Sequence Diagram: Permit Revocation Flow

```mermaid
sequenceDiagram
    participant Admin as Admin/Validator Pusat
    participant API as layanan-perlengkapan API
    participant NOT as Notifikasi Service
    participant OS as Operator Satker

    Note over Admin,OS: Permit Revocation Process
    Admin->>API: POST /api/v1/pemakaian/permits/{permit_id}/revoke
    API->>API: Validate: permit is COMPLETED (not already EXPIRED or REVOKED)
    API->>API: Require revocation reason
    API->>API: Transition: COMPLETED → REVOKED
    API->>API: Record revocation metadata (revoked_by, revoked_at, reason)
    API->>NOT: Send notification to Operator Satker
    NOT-->>OS: Notification: "Permit revoked: {reason}"
    API-->>Admin: Revocation successful
```

### 22.7 REST API Endpoints

**Permit Management:**
```
POST   /api/v1/pemakaian/permits
GET    /api/v1/pemakaian/permits
GET    /api/v1/pemakaian/permits/{id}
PUT    /api/v1/pemakaian/permits/{id}
DELETE /api/v1/pemakaian/permits/{id}
POST   /api/v1/pemakaian/permits/{id}/pegawai
POST   /api/v1/pemakaian/permits/{id}/bmn
POST   /api/v1/pemakaian/permits/{id}/period
POST   /api/v1/pemakaian/permits/{id}/generate-document
POST   /api/v1/pemakaian/permits/{id}/upload-signed
POST   /api/v1/pemakaian/permits/{id}/renew
POST   /api/v1/pemakaian/permits/{id}/revoke
```

**BMN Availability:**
```
GET    /api/v1/pemakaian/bmn/available
GET    /api/v1/pemakaian/bmn/{nup}/status
```

**Monitoring and Reports:**
```
GET    /api/v1/pemakaian/permits/expiring
GET    /api/v1/pemakaian/reports/active-permits
GET    /api/v1/pemakaian/reports/utilization
GET    /api/v1/pemakaian/reports/history/bmn/{nup}
GET    /api/v1/pemakaian/reports/history/pegawai/{nip}
```

### 22.8 Database Schema

**Permit Data:**
```sql
CREATE TABLE perlengkapan.izin_pemakaian_bmn (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permit_number VARCHAR(100) UNIQUE NOT NULL, -- IZN/{YEAR}/{SATKER}/{SEQUENCE}
    satker_id UUID NOT NULL REFERENCES authenc.satkers(id),
    pegawai_nip VARCHAR(20) NOT NULL,
    pegawai_nama VARCHAR(255) NOT NULL,
    pegawai_jabatan VARCHAR(255),
    pegawai_pangkat VARCHAR(50),
    pegawai_golongan VARCHAR(10),
    pegawai_photo_url VARCHAR(500),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT', -- DRAFT, DOCUMENT_GENERATED, COMPLETED, EXPIRED, REVOKED
    draft_document_id UUID,
    signed_document_id UUID,
    previous_permit_id UUID REFERENCES perlengkapan.izin_pemakaian_bmn(id), -- For renewals
    revoked_by UUID REFERENCES authenc.users(id),
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT,
    created_by UUID NOT NULL REFERENCES authenc.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_end_after_start CHECK (end_date > start_date)
);

CREATE TABLE perlengkapan.izin_pemakaian_bmn_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permit_id UUID NOT NULL REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE CASCADE,
    nup VARCHAR(50) NOT NULL,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,
    kondisi VARCHAR(50),
    tahun_perolehan INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(permit_id, nup)
);

-- Index for active permit check (one BMN = one active permit)
CREATE INDEX idx_izin_pemakaian_active ON perlengkapan.izin_pemakaian_bmn(status)
    WHERE status = 'COMPLETED';

CREATE INDEX idx_izin_pemakaian_bmn_item_nup ON perlengkapan.izin_pemakaian_bmn_item(nup);
CREATE INDEX idx_izin_pemakaian_pegawai ON perlengkapan.izin_pemakaian_bmn(pegawai_nip);
CREATE INDEX idx_izin_pemakaian_satker ON perlengkapan.izin_pemakaian_bmn(satker_id);
CREATE INDEX idx_izin_pemakaian_end_date ON perlengkapan.izin_pemakaian_bmn(end_date);
```

### 22.9 Validation Rules

**Permit Creation Validation:**
- Pegawai must exist in MySIMKARI
- Pegawai must belong to the satker
- BMN must exist in SIMAN
- BMN must not have active permit (status = COMPLETED)
- End date must be after start date
- At least one BMN item required

**BMN Availability Check:**
```sql
-- Check if BMN has active permit
SELECT COUNT(*) FROM perlengkapan.izin_pemakaian_bmn p
JOIN perlengkapan.izin_pemakaian_bmn_item i ON p.id = i.permit_id
WHERE i.nup = $1
  AND p.status = 'COMPLETED'
  AND p.end_date >= CURRENT_DATE;
```

**Renewal Validation:**
- Previous permit must exist
- Previous permit must be COMPLETED or EXPIRED
- Can create renewal before expiry (for continuity)

**Revocation Validation:**
- Permit must be COMPLETED (not already EXPIRED or REVOKED)
- Revocation reason required (minimum 20 characters)
- Only Admin or Validator Pusat can revoke

### 22.10 Document Format Specification

**Page 1: Pegawai Identity**
```
SURAT IZIN PEMAKAIAN BARANG MILIK NEGARA
Nomor: {permit_number}

Foto Pegawai: [Photo from MySIMKARI]

Identitas Pegawai:
NIP         : {pegawai_nip}
Nama        : {pegawai_nama}
Jabatan     : {pegawai_jabatan}
Pangkat     : {pegawai_pangkat}
Golongan    : {pegawai_golongan}
Satuan Kerja: {satker_nama}

Periode Pemakaian:
Tanggal Mulai : {start_date}
Tanggal Akhir : {end_date}
```

**Page 2+: BMN Details Table**
```
RINCIAN BARANG MILIK NEGARA

| No | NUP | Nama Barang | Kode Barang | Kondisi | Tahun Perolehan |
|----|-----|-------------|-------------|---------|-----------------|
| 1  | ... | ...         | ...         | ...     | ...             |
| 2  | ... | ...         | ...         | ...     | ...             |
| n  | ... | ...         | ...         | ...     | ...             |

Catatan:
- Pegawai bertanggung jawab atas pemeliharaan BMN yang digunakan
- Izin ini berlaku sampai dengan {end_date}
- Perpanjangan izin harus diajukan sebelum masa berlaku berakhir

                                        {Tempat}, {Tanggal}
                                        Pimpinan Satuan Kerja,


                                        {Nama Pimpinan}
                                        NIP. {NIP Pimpinan}
```

### 22.11 Expiry Reminder Schedule

**Automated Cron Job (Daily at 00:00 WIB):**
```rust
// layanan/perlengkapan/crates/api/src/cron/pemakaian_expiry.rs
pub async fn check_expiring_permits() -> Result<()> {
    let today = Utc::now().date_naive();

    // H-30 reminder
    let expiring_30 = get_permits_expiring_on(today + Duration::days(30)).await?;
    for permit in expiring_30 {
        send_expiry_reminder(permit, 30).await?;
    }

    // H-14 reminder
    let expiring_14 = get_permits_expiring_on(today + Duration::days(14)).await?;
    for permit in expiring_14 {
        send_expiry_reminder(permit, 14).await?;
    }

    // H-7 reminder
    let expiring_7 = get_permits_expiring_on(today + Duration::days(7)).await?;
    for permit in expiring_7 {
        send_expiry_reminder(permit, 7).await?;
    }

    // Auto-expire permits past end_date
    let expired = get_permits_past_end_date(today).await?;
    for permit in expired {
        expire_permit(permit).await?;
    }

    Ok(())
}
```


## 23. SK Penghapusan BMN Business Process Flow

### 23.1 Overview

This section provides detailed business process flows for the SK Penghapusan BMN (BMN Deletion Decree) module, covering the complete workflow from initiation by Operator Satker through review by Validator Wilayah and Validator Pusat, document generation, and final completion.

**Key Requirements Addressed:** REQ-PH001 through REQ-PH027

### 23.2 Actors and Roles

| Actor | Role | Responsibilities |
|-------|------|------------------|
| Operator Satker | BMN manager at Kejari/Cabjari | Initiate SK Penghapusan request, select BMN, upload supporting documents |
| Validator Wilayah | Regional validator at Kejaksaan Tinggi | Review requests, forward to Pusat or return for revision |
| Validator Pusat | Central validator at Kejaksaan Agung | Final review, generate SK document, upload signed SK |
| Pimpinan | Authorized official | Sign SK Penghapusan document |

### 23.3 Workflow States

```
DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → DOCUMENT_GENERATED → COMPLETED
```

**State Definitions:**
- `DRAFT`: Operator satker is preparing request
- `SUBMITTED`: Operator satker has submitted to validator wilayah
- `REVIEWED_WILAYAH`: Validator wilayah has reviewed and forwarded to validator pusat
- `REVIEWED_PUSAT`: Validator pusat has reviewed and approved for SK generation
- `DOCUMENT_GENERATED`: System has generated DOCX draft SK
- `COMPLETED`: Validator pusat has uploaded signed PDF SK

### 23.4 Sequence Diagram: Complete SK Penghapusan BMN Flow

```mermaid
sequenceDiagram
    participant OS as Operator Satker
    participant VW as Validator Wilayah
    participant VP as Validator Pusat
    participant API as layanan-perlengkapan API
    participant WF as Workflow Service
    participant INT as Integrasi Service
    participant SIMAN as SIMAN API
    participant PEM as Pemakaian Service
    participant DOK as Dokumen Service
    participant NOT as Notifikasi Service
    participant PIM as Pimpinan

    Note over OS,PIM: Phase 1: Request Initiation by Operator Satker
    OS->>API: POST /api/v1/penghapusan/requests
    API->>API: Create request (state: DRAFT)
    API->>WF: Create workflow instance
    OS->>API: GET /api/v1/penghapusan/bmn/available?satker_id=...
    API->>INT: gRPC: GetSimanAssets(satker_id)
    INT->>SIMAN: GET /api/v2/assets?satker_code=...
    SIMAN-->>INT: Return BMN list
    INT-->>API: Return BMN list
    API-->>OS: Return available BMN for deletion

    OS->>API: POST /api/v1/penghapusan/requests/{id}/bmn
    API->>PEM: gRPC: CheckBMNActivePermit(nup)
    PEM-->>API: Return active permit status
    API->>API: Validate: BMN not in active use
    API->>API: Add BMN to request
    Note over OS: Operator can add multiple BMN items

    OS->>API: POST /api/v1/penghapusan/requests/{id}/attachments
    API->>API: Upload supporting documents (persyaratan penghapusan)
    API->>API: Store in object storage with SHA-256 checksum
    API-->>OS: Attachment uploaded

    OS->>API: POST /api/v1/penghapusan/requests/{id}/submit
    API->>API: Validate: at least one BMN, at least one attachment
    API->>WF: Transition: DRAFT → SUBMITTED
    WF->>NOT: Send notification to Validator Wilayah
    NOT-->>VW: Notification: "New SK Penghapusan request"

    Note over OS,PIM: Phase 2: Review by Validator Wilayah
    VW->>API: GET /api/v1/penghapusan/requests?status=SUBMITTED
    API-->>VW: Return pending requests
    VW->>API: GET /api/v1/penghapusan/requests/{id}
    API-->>VW: Return request details (BMN list, attachments)

    alt Forward to Validator Pusat
        VW->>API: POST /api/v1/penghapusan/requests/{id}/forward
        API->>WF: Transition: SUBMITTED → REVIEWED_WILAYAH
        WF->>NOT: Send notification to Validator Pusat
        NOT-->>VP: Notification: "SK Penghapusan ready for review"
    else Return to Operator Satker for Revision
        VW->>API: POST /api/v1/penghapusan/requests/{id}/return
        API->>WF: Transition: SUBMITTED → DRAFT (with revision notes)
        WF->>NOT: Send notification to Operator Satker
        NOT-->>OS: Notification: "Revision required"
        Note over OS: Operator revises and resubmits (back to Phase 1)
    end

    Note over OS,PIM: Phase 3: Review by Validator Pusat
    VP->>API: GET /api/v1/penghapusan/requests?status=REVIEWED_WILAYAH
    API-->>VP: Return requests ready for review
    VP->>API: GET /api/v1/penghapusan/requests/{id}
    API-->>VP: Return request details
    VP->>API: GET /api/v1/penghapusan/requests/{id}/bmn-details
    API->>INT: gRPC: GetSimanAssetDetails(nup_list)
    INT->>SIMAN: GET /api/v2/assets/details
    SIMAN-->>INT: Return detailed BMN data
    INT-->>API: Return BMN details
    API-->>VP: Return BMN details for analysis

    VP->>API: POST /api/v1/penghapusan/requests/{id}/approve
    API->>WF: Transition: REVIEWED_WILAYAH → REVIEWED_PUSAT
    WF->>NOT: Send notification
    NOT-->>VP: Notification: "Request approved for SK generation"

    Note over OS,PIM: Phase 4: SK Document Generation
    VP->>API: POST /api/v1/penghapusan/requests/{id}/generate-sk
    API->>DOK: gRPC: GenerateDocument(type: sk_penghapusan, format: DOCX)
    DOK->>DOK: Generate DOCX with:
    DOK->>DOK: - Official letterhead
    DOK->>DOK: - SK number: SK/{YEAR}/{SEQUENCE}
    DOK->>DOK: - Satker information
    DOK->>DOK: - BMN details table
    DOK->>DOK: - Legal basis (dasar hukum)
    DOK->>DOK: - Total value of BMN to be deleted
    DOK-->>API: Return document_id
    API->>WF: Transition: REVIEWED_PUSAT → DOCUMENT_GENERATED
    API->>NOT: Send notification to Validator Pusat
    NOT-->>VP: Notification: "Draft SK document ready"
    VP->>API: GET /api/v1/documents/{document_id}/download
    API->>DOK: gRPC: GetDocument(document_id)
    DOK-->>API: Return DOCX binary
    API-->>VP: Download DOCX draft

    Note over OS,PIM: Phase 5: Signature and Upload
    VP->>PIM: Send DOCX for signature (offline process)
    PIM->>PIM: Review and sign document
    PIM->>VP: Return signed PDF

    VP->>API: POST /api/v1/penghapusan/requests/{id}/upload-signed-sk
    API->>API: Upload signed PDF to object storage
    API->>API: Calculate SHA-256 checksum
    API->>WF: Transition: DOCUMENT_GENERATED → COMPLETED
    API->>NOT: Send notification to Operator Satker, Validator Wilayah
    NOT-->>OS: Notification: "SK Penghapusan completed"
    NOT-->>VW: Notification: "SK Penghapusan completed"

    Note over OS,PIM: Phase 6: View Signed SK
    OS->>API: GET /api/v1/penghapusan/requests/{id}/signed-sk
    API->>DOK: gRPC: GetDocument(signed_document_id)
    DOK-->>API: Return PDF binary
    API-->>OS: Download signed PDF SK

    VW->>API: GET /api/v1/penghapusan/requests/{id}/signed-sk
    API->>DOK: gRPC: GetDocument(signed_document_id)
    DOK-->>API: Return PDF binary
    API-->>VW: Download signed PDF SK
```

### 23.5 REST API Endpoints

**Request Management:**
```
POST   /api/v1/penghapusan/requests
GET    /api/v1/penghapusan/requests
GET    /api/v1/penghapusan/requests/{id}
PUT    /api/v1/penghapusan/requests/{id}
DELETE /api/v1/penghapusan/requests/{id}
POST   /api/v1/penghapusan/requests/{id}/bmn
POST   /api/v1/penghapusan/requests/{id}/attachments
POST   /api/v1/penghapusan/requests/{id}/submit
POST   /api/v1/penghapusan/requests/{id}/forward
POST   /api/v1/penghapusan/requests/{id}/return
POST   /api/v1/penghapusan/requests/{id}/approve
POST   /api/v1/penghapusan/requests/{id}/generate-sk
POST   /api/v1/penghapusan/requests/{id}/upload-signed-sk
GET    /api/v1/penghapusan/requests/{id}/signed-sk
GET    /api/v1/penghapusan/requests/{id}/bmn-details
```

**BMN Availability:**
```
GET    /api/v1/penghapusan/bmn/available
GET    /api/v1/penghapusan/bmn/{nup}/eligibility
```

**Monitoring and Reports:**
```
GET    /api/v1/penghapusan/reports/by-satker
GET    /api/v1/penghapusan/reports/by-wilayah
GET    /api/v1/penghapusan/reports/summary
```

### 23.6 Database Schema

**Request Data:**
```sql
CREATE TABLE perlengkapan.sk_penghapusan_bmn_request (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sk_number VARCHAR(100) UNIQUE, -- SK/{YEAR}/{SEQUENCE}
    satker_id UUID NOT NULL REFERENCES authenc.satkers(id),
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT',
    workflow_instance_id UUID,
    submitted_at TIMESTAMPTZ,
    reviewed_wilayah_at TIMESTAMPTZ,
    reviewed_wilayah_by UUID REFERENCES authenc.users(id),
    reviewed_pusat_at TIMESTAMPTZ,
    reviewed_pusat_by UUID REFERENCES authenc.users(id),
    document_generated_at TIMESTAMPTZ,
    draft_document_id UUID,
    signed_document_id UUID,
    completed_at TIMESTAMPTZ,
    revision_notes TEXT,
    created_by UUID NOT NULL REFERENCES authenc.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE perlengkapan.sk_penghapusan_bmn_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES perlengkapan.sk_penghapusan_bmn_request(id) ON DELETE CASCADE,
    nup VARCHAR(50) NOT NULL,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,
    kondisi VARCHAR(50),
    tahun_perolehan INTEGER,
    nilai_perolehan DECIMAL(15,2),
    alasan_penghapusan TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(request_id, nup)
);

CREATE TABLE perlengkapan.sk_penghapusan_bmn_attachment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES perlengkapan.sk_penghapusan_bmn_request(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    checksum VARCHAR(64) NOT NULL, -- SHA-256
    uploaded_by UUID NOT NULL REFERENCES authenc.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sk_penghapusan_satker ON perlengkapan.sk_penghapusan_bmn_request(satker_id);
CREATE INDEX idx_sk_penghapusan_status ON perlengkapan.sk_penghapusan_bmn_request(status);
CREATE INDEX idx_sk_penghapusan_item_nup ON perlengkapan.sk_penghapusan_bmn_item(nup);
```

### 23.7 Validation Rules

**Request Creation Validation:**
- Satker must exist in authenc.satkers
- At least one BMN item required
- At least one supporting document required
- BMN must exist in SIMAN
- BMN must not have active izin pemakaian (status = COMPLETED)

**BMN Eligibility Check:**
```sql
-- Check if BMN has active permit (should NOT be in active use)
SELECT COUNT(*) FROM perlengkapan.izin_pemakaian_bmn p
JOIN perlengkapan.izin_pemakaian_bmn_item i ON p.id = i.permit_id
WHERE i.nup = $1
  AND p.status = 'COMPLETED'
  AND p.end_date >= CURRENT_DATE;

-- If count > 0, BMN is in active use and cannot be deleted
```

**Workflow Transition Validation:**
- DRAFT → SUBMITTED: Requires at least one BMN item and one attachment
- SUBMITTED → REVIEWED_WILAYAH: Only Validator Wilayah can forward
- SUBMITTED → DRAFT: Only Validator Wilayah can return with revision notes
- REVIEWED_WILAYAH → REVIEWED_PUSAT: Only Validator Pusat can approve
- REVIEWED_PUSAT → DOCUMENT_GENERATED: Only Validator Pusat can generate SK
- DOCUMENT_GENERATED → COMPLETED: Only Validator Pusat can upload signed SK

**Supporting Document Validation:**
- File size limit: 10 MB per file
- Allowed MIME types: application/pdf, image/jpeg, image/png
- SHA-256 checksum calculated and stored for integrity verification

### 23.8 Document Format Specification

**SK Penghapusan BMN Format:**
```
[Official Letterhead - Kejaksaan Republik Indonesia]

SURAT KEPUTUSAN PENGHAPUSAN BARANG MILIK NEGARA
Nomor: {sk_number}

KEPALA {SATKER_NAMA}

Menimbang:
a. bahwa berdasarkan hasil inventarisasi dan penilaian kondisi Barang Milik Negara...
b. bahwa Barang Milik Negara sebagaimana dimaksud pada huruf a sudah tidak layak pakai...
c. bahwa berdasarkan pertimbangan sebagaimana dimaksud pada huruf a dan b...

Mengingat:
1. Peraturan Pemerintah Nomor 27 Tahun 2014 tentang Pengelolaan Barang Milik Negara/Daerah
2. Peraturan Menteri Keuangan tentang Tata Cara Pelaksanaan Penghapusan BMN
3. [Additional legal basis]

MEMUTUSKAN:

Menetapkan:
PERTAMA   : Menghapus Barang Milik Negara sebagaimana tercantum dalam lampiran
            Surat Keputusan ini dari Daftar Barang Milik Negara.

KEDUA     : Penghapusan sebagaimana dimaksud dalam Diktum PERTAMA dilakukan
            dengan cara [pemusnahan/penjualan/hibah].

KETIGA    : Surat Keputusan ini mulai berlaku pada tanggal ditetapkan.

LAMPIRAN: DAFTAR BARANG MILIK NEGARA YANG DIHAPUS

| No | NUP | Nama Barang | Kode Barang | Kondisi | Tahun Perolehan | Nilai Perolehan | Alasan Penghapusan |
|----|-----|-------------|-------------|---------|-----------------|-----------------|-------------------|
| 1  | ... | ...         | ...         | ...     | ...             | Rp ...          | ...               |
| 2  | ... | ...         | ...         | ...     | ...             | Rp ...          | ...               |
| n  | ... | ...         | ...         | ...     | ...             | Rp ...          | ...               |

TOTAL NILAI PEROLEHAN: Rp {total_nilai_perolehan}

                                        Ditetapkan di {Tempat}
                                        pada tanggal {Tanggal}
                                        {Jabatan Pimpinan},


                                        {Nama Pimpinan}
                                        NIP. {NIP Pimpinan}
```

### 23.9 Integration with Pemakaian Service

**gRPC Service Definition:**
```protobuf
// layanan/perlengkapan/crates/pemakaian/proto/pemakaian.proto
service PemakaianService {
    rpc CheckBMNActivePermit(CheckBMNActivePermitRequest) returns (CheckBMNActivePermitResponse);
}

message CheckBMNActivePermitRequest {
    string nup = 1;
}

message CheckBMNActivePermitResponse {
    bool has_active_permit = 1;
    string permit_id = 2;
    string permit_number = 3;
    string pegawai_nama = 4;
    string end_date = 5;
}
```

**Implementation:**
```rust
// layanan/perlengkapan/crates/pemakaian/src/grpc/service.rs
#[tonic::async_trait]
impl PemakaianService for PemakaianServiceImpl {
    async fn check_bmn_active_permit(
        &self,
        request: Request<CheckBMNActivePermitRequest>,
    ) -> Result<Response<CheckBMNActivePermitResponse>, Status> {
        let nup = request.into_inner().nup;

        let query = r#"
            SELECT p.id, p.permit_number, p.pegawai_nama, p.end_date
            FROM perlengkapan.izin_pemakaian_bmn p
            JOIN perlengkapan.izin_pemakaian_bmn_item i ON p.id = i.permit_id
            WHERE i.nup = $1
              AND p.status = 'COMPLETED'
              AND p.end_date >= CURRENT_DATE
            LIMIT 1
        "#;

        let client = self.db_pool.get().await
            .map_err(|e| Status::internal(e.to_string()))?;

        match client.query_opt(query, &[&nup]).await {
            Ok(Some(row)) => {
                Ok(Response::new(CheckBMNActivePermitResponse {
                    has_active_permit: true,
                    permit_id: row.get::<_, Uuid>("id").to_string(),
                    permit_number: row.get("permit_number"),
                    pegawai_nama: row.get("pegawai_nama"),
                    end_date: row.get::<_, NaiveDate>("end_date").to_string(),
                }))
            }
            Ok(None) => {
                Ok(Response::new(CheckBMNActivePermitResponse {
                    has_active_permit: false,
                    permit_id: String::new(),
                    permit_number: String::new(),
                    pegawai_nama: String::new(),
                    end_date: String::new(),
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
```

### 23.10 Monitoring Dashboard

**Dashboard Metrics:**
- Total SK Penghapusan requests by status
- Requests by satker and wilayah
- Average processing time per workflow stage
- Total value of BMN to be deleted
- Pending requests requiring action

**Dashboard Queries:**
```sql
-- Summary by status
SELECT status, COUNT(*) as count
FROM perlengkapan.sk_penghapusan_bmn_request
GROUP BY status;

-- Summary by wilayah
SELECT s.wilayah_code, s.wilayah_nama, COUNT(r.id) as request_count
FROM perlengkapan.sk_penghapusan_bmn_request r
JOIN authenc.satkers s ON r.satker_id = s.id
GROUP BY s.wilayah_code, s.wilayah_nama;

-- Total value to be deleted
SELECT SUM(i.nilai_perolehan) as total_nilai
FROM perlengkapan.sk_penghapusan_bmn_item i
JOIN perlengkapan.sk_penghapusan_bmn_request r ON i.request_id = r.id
WHERE r.status IN ('REVIEWED_PUSAT', 'DOCUMENT_GENERATED', 'COMPLETED');

-- Average processing time
SELECT
    AVG(EXTRACT(EPOCH FROM (reviewed_wilayah_at - submitted_at))/3600) as avg_hours_wilayah_review,
    AVG(EXTRACT(EPOCH FROM (reviewed_pusat_at - reviewed_wilayah_at))/3600) as avg_hours_pusat_review,
    AVG(EXTRACT(EPOCH FROM (completed_at - document_generated_at))/3600) as avg_hours_signature
FROM perlengkapan.sk_penghapusan_bmn_request
WHERE completed_at IS NOT NULL;
```

---

**End of Business Process Flow Sections**
**Document Version:** 3.0.0
**Last Updated:** February 11, 2026
**Status:** Complete with Detailed Business Process Flows


## 24. End-to-End Testing Strategy with Playwright

### 24.1 Overview

This section defines comprehensive end-to-end testing strategy using Playwright to validate all business process flows across the entire system, from frontend microfrontend through backend services to database and external integrations.

**Testing Scope:**
1. Kebutuhan BMN complete workflow
2. Kebutuhan Pakaian Dinas complete workflow
3. Pemakaian BMN complete workflow (creation, renewal, revocation)
4. SK Penghapusan BMN complete workflow
5. Cross-module integration validation
6. Role-based access control validation

### 24.2 Test User Setup

**Mock Users for Testing:**
```typescript
// tests/e2e/fixtures/users.ts
export const testUsers = {
  operatorSatker: {
    username: 'operator.kejari.jakarta',
    password: 'Test123!@#',
    nip: '199001012020121001',
    nama: 'Budi Santoso',
    role: 'operator_satker',
    satker_id: 'kejari-jakarta-pusat',
    satker_nama: 'Kejaksaan Negeri Jakarta Pusat',
    wilayah_code: '0200', // DKI Jakarta
    permissions: [
      'kebutuhan_bmn.create',
      'kebutuhan_bmn.submit',
      'pemakaian_bmn.create',
      'penghapusan_bmn.create',
    ]
  },

  validatorWilayah: {
    username: 'validator.kejati.jakarta',
    password: 'Test123!@#',
    nip: '198501012015121001',
    nama: 'Siti Aminah',
    role: 'validator_wilayah',
    satker_id: 'kejati-dki-jakarta',
    satker_nama: 'Kejaksaan Tinggi DKI Jakarta',
    wilayah_code: '0200',
    permissions: [
      'kebutuhan_bmn.review_wilayah',
      'kebutuhan_bmn.forward',
      'kebutuhan_bmn.return',
      'pemakaian_bmn.view_wilayah',
      'penghapusan_bmn.review_wilayah',
    ]
  },

  validatorPusat: {
    username: 'validator.kejagung',
    password: 'Test123!@#',
    nip: '198001012010121001',
    nama: 'Ahmad Hidayat',
    role: 'validator_pusat',
    satker_id: 'kejaksaan-agung',
    satker_nama: 'Kejaksaan Agung Republik Indonesia',
    wilayah_code: '0100',
    permissions: [
      'kebutuhan_bmn.initiate_period',
      'kebutuhan_bmn.configure_eligible',
      'kebutuhan_bmn.review_pusat',
      'kebutuhan_bmn.approve',
      'kebutuhan_bmn.reject',
      'kebutuhan_bmn.generate_report',
      'pemakaian_bmn.view_all',
      'penghapusan_bmn.review_pusat',
      'penghapusan_bmn.generate_sk',
    ]
  },

  admin: {
    username: 'admin.simpel',
    password: 'Admin123!@#',
    nip: '197501012005121001',
    nama: 'Super Admin',
    role: 'admin',
    satker_id: 'kejaksaan-agung',
    satker_nama: 'Kejaksaan Agung Republik Indonesia',
    wilayah_code: '0100',
    permissions: ['*'] // All permissions
  }
};
```

### 24.3 Test Data Setup

**Mock Integration Data:**
```typescript
// tests/e2e/fixtures/integration-data.ts
export const mockSimanData = {
  assets: [
    {
      nup: '0001.01.01.001',
      kode_barang: '01.01.01.001',
      nama_barang: 'Meja Kerja Kayu',
      kondisi: 'BAIK',
      tahun_perolehan: 2020,
      nilai_perolehan: 2500000,
      satker_code: 'kejari-jakarta-pusat'
    },
    {
      nup: '0001.01.01.002',
      kode_barang: '01.01.01.002',
      nama_barang: 'Kursi Kerja Putar',
      kondisi: 'RUSAK_RINGAN',
      tahun_perolehan: 2019,
      nilai_perolehan: 1500000,
      satker_code: 'kejari-jakarta-pusat'
    },
    {
      nup: '0002.03.01.001',
      kode_barang: '02.03.01.001',
      nama_barang: 'Laptop Dell Latitude',
      kondisi: 'BAIK',
      tahun_perolehan: 2022,
      nilai_perolehan: 15000000,
      satker_code: 'kejari-jakarta-pusat'
    }
  ]
};

export const mockMySIMKARIData = {
  pegawai: [
    {
      nip: '199001012020121001',
      nama: 'Budi Santoso',
      satker_code: 'kejari-jakarta-pusat',
      jabatan: 'Kepala Sub Bagian Umum',
      pangkat: 'Penata Muda Tk.I',
      golongan: 'III/b',
      eselon: null,
      jenis_pegawai: 'TU',
      jenis_kelamin: 'L',
      photo_url: 'https://mysimkari.test/photos/199001012020121001.jpg'
    },
    {
      nip: '198501012015121002',
      nama: 'Dewi Lestari',
      satker_code: 'kejari-jakarta-pusat',
      jabatan: 'Jaksa Penuntut Umum',
      pangkat: 'Penata',
      golongan: 'III/c',
      eselon: null,
      jenis_pegawai: 'Jaksa',
      jenis_kelamin: 'P',
      photo_url: 'https://mysimkari.test/photos/198501012015121002.jpg'
    }
  ],
  summary: {
    total_pegawai: 45,
    total_eselon: 3,
    golongan_breakdown: [
      { golongan: 'I', count: 2 },
      { golongan: 'II', count: 8 },
      { golongan: 'III', count: 25 },
      { golongan: 'IV', count: 10 }
    ],
    total_jaksa: 20,
    total_non_jaksa: 25
  }
};
```

### 24.4 Test Suite 1: Kebutuhan BMN Complete Flow

```typescript
// tests/e2e/kebutuhan-bmn.spec.ts
import { test, expect } from '@playwright/test';
import { testUsers } from './fixtures/users';
import { mockSimanData, mockMySIMKARIData } from './fixtures/integration-data';

test.describe('Kebutuhan BMN - Complete Workflow', () => {

  test.beforeEach(async ({ page }) => {
    // Setup mock API responses for SIMAN and MySIMKARI
    await page.route('**/api/integration/siman/**', route => {
      route.fulfill({ json: mockSimanData });
    });

    await page.route('**/api/integration/mysimkari/**', route => {
      route.fulfill({ json: mockMySIMKARIData });
    });
  });

  test('Phase 1: Validator Pusat initiates period and configures eligibility', async ({ page }) => {
    // Login as Validator Pusat
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorPusat.username);
    await page.fill('input[name="password"]', testUsers.validatorPusat.password);
    await page.click('button[type="submit"]');

    // Wait for dashboard
    await expect(page).toHaveURL('/dashboard');

    // Navigate to Kebutuhan BMN Period Management
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Kelola Periode');

    // Create new period
    await page.click('button:has-text("Buat Periode Baru")');
    await page.fill('input[name="nama_period"]', 'Kebutuhan BMN TA 2026');
    await page.fill('input[name="start_date"]', '2026-03-01');
    await page.fill('input[name="end_date"]', '2026-06-30');
    await page.fill('input[name="deadline"]', '2026-06-30T23:59');
    await page.click('button:has-text("Simpan")');

    // Verify period created
    await expect(page.locator('text=Kebutuhan BMN TA 2026')).toBeVisible();

    // Configure eligible BMN
    await page.click('button:has-text("Konfigurasi BMN")');
    await page.click('input[type="checkbox"][value="01.01.01.001"]'); // Meja Kerja
    await page.click('input[type="checkbox"][value="02.03.01.001"]'); // Laptop
    await page.click('button:has-text("Simpan BMN Eligible")');

    // Verify eligible BMN configured
    await expect(page.locator('text=2 BMN dipilih')).toBeVisible();

    // Configure eligible satkers
    await page.click('button:has-text("Konfigurasi Satker")');
    await page.click('text=DKI Jakarta'); // Expand wilayah
    await page.click('input[type="checkbox"][value="kejari-jakarta-pusat"]');
    await page.click('input[type="checkbox"][value="kejari-jakarta-selatan"]');
    await page.click('button:has-text("Simpan Satker Eligible")');

    // Verify eligible satkers configured
    await expect(page.locator('text=2 Satker dipilih')).toBeVisible();

    // Activate period
    await page.click('button:has-text("Aktifkan Periode")');
    await expect(page.locator('text=Status: ACTIVE')).toBeVisible();
  });

  test('Phase 2: Operator Satker submits kebutuhan BMN', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to Kebutuhan BMN Submission
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Ajukan Kebutuhan');

    // Verify active period is shown
    await expect(page.locator('text=Kebutuhan BMN TA 2026')).toBeVisible();
    await expect(page.locator('text=Deadline: 30 Juni 2026')).toBeVisible();

    // Create new submission
    await page.click('button:has-text("Buat Pengajuan Baru")');

    // Add BMN items
    await page.click('button:has-text("Tambah BMN")');
    await page.selectOption('select[name="kode_barang"]', '01.01.01.001'); // Meja Kerja
    await page.fill('input[name="jumlah_kebutuhan"]', '10');
    await page.fill('textarea[name="justification"]',
      'Kebutuhan meja kerja untuk ruang kerja baru yang akan dibangun. ' +
      'Saat ini hanya memiliki 5 meja dalam kondisi baik, sedangkan kebutuhan ' +
      'standar untuk 15 pegawai adalah 15 meja. Gap: 10 meja.'
    );
    await page.click('button:has-text("Simpan Item")');

    // Verify item added
    await expect(page.locator('text=Meja Kerja Kayu')).toBeVisible();
    await expect(page.locator('text=Jumlah: 10')).toBeVisible();

    // Add another BMN item
    await page.click('button:has-text("Tambah BMN")');
    await page.selectOption('select[name="kode_barang"]', '02.03.01.001'); // Laptop
    await page.fill('input[name="jumlah_kebutuhan"]', '5');
    await page.fill('textarea[name="justification"]',
      'Kebutuhan laptop untuk jaksa penuntut umum yang baru. ' +
      'Saat ini hanya memiliki 15 laptop, kebutuhan untuk 20 jaksa adalah 20 laptop. Gap: 5 laptop.'
    );
    await page.click('button:has-text("Simpan Item")');

    // Upload supporting documents
    await page.click('button:has-text("Upload Lampiran")');
    await page.setInputFiles('input[type="file"]', 'tests/fixtures/surat-permohonan.pdf');
    await page.click('button:has-text("Upload")');

    // Verify attachment uploaded
    await expect(page.locator('text=surat-permohonan.pdf')).toBeVisible();

    // Submit to Validator Wilayah
    await page.click('button:has-text("Kirim ke Validator Wilayah")');
    await page.click('button:has-text("Ya, Kirim")'); // Confirmation dialog

    // Verify submission successful
    await expect(page.locator('text=Status: SUBMITTED')).toBeVisible();
    await expect(page.locator('text=Pengajuan berhasil dikirim')).toBeVisible();
  });

  test('Phase 3: Validator Wilayah reviews and forwards to Validator Pusat', async ({ page }) => {
    // Login as Validator Wilayah
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorWilayah.username);
    await page.fill('input[name="password"]', testUsers.validatorWilayah.password);
    await page.click('button[type="submit"]');

    // Navigate to pending submissions
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Review Pengajuan');

    // Filter by status SUBMITTED
    await page.selectOption('select[name="status"]', 'SUBMITTED');

    // Open submission detail
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');

    // Review submission details
    await expect(page.locator('text=Meja Kerja Kayu')).toBeVisible();
    await expect(page.locator('text=Laptop Dell Latitude')).toBeVisible();
    await expect(page.locator('text=surat-permohonan.pdf')).toBeVisible();

    // Forward to Validator Pusat
    await page.click('button:has-text("Teruskan ke Validator Pusat")');
    await page.fill('textarea[name="catatan"]', 'Pengajuan sudah sesuai dan lengkap.');
    await page.click('button:has-text("Ya, Teruskan")');

    // Verify forwarded
    await expect(page.locator('text=Status: REVIEWED_WILAYAH')).toBeVisible();
    await expect(page.locator('text=Berhasil diteruskan')).toBeVisible();
  });

  test('Phase 4: Validator Pusat analyzes with integrated data and approves', async ({ page }) => {
    // Login as Validator Pusat
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorPusat.username);
    await page.fill('input[name="password"]', testUsers.validatorPusat.password);
    await page.click('button[type="submit"]');

    // Navigate to analysis
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Analisis Kebutuhan');

    // Filter by status REVIEWED_WILAYAH
    await page.selectOption('select[name="status"]', 'REVIEWED_WILAYAH');

    // Open submission for analysis
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');

    // View integrated data
    await page.click('button:has-text("Lihat Data Analisis")');

    // Verify SIMAN data displayed
    await expect(page.locator('text=Data BMN Existing (SIMAN)')).toBeVisible();
    await expect(page.locator('text=Meja Kerja: 5 unit (Kondisi Baik)')).toBeVisible();
    await expect(page.locator('text=Laptop: 15 unit (Kondisi Baik)')).toBeVisible();

    // Verify MySIMKARI data displayed
    await expect(page.locator('text=Data Pegawai (MySIMKARI)')).toBeVisible();
    await expect(page.locator('text=Total Pegawai: 45')).toBeVisible();
    await expect(page.locator('text=Eselon: 3')).toBeVisible();
    await expect(page.locator('text=Jaksa: 20')).toBeVisible();
    await expect(page.locator('text=Non-Jaksa: 25')).toBeVisible();

    // Verify gap analysis
    await expect(page.locator('text=Gap Meja Kerja: 10 unit')).toBeVisible();
    await expect(page.locator('text=Gap Laptop: 5 unit')).toBeVisible();

    // Approve submission
    await page.click('button:has-text("Setujui")');
    await page.fill('textarea[name="catatan_approval"]',
      'Disetujui berdasarkan analisis gap dan data pegawai. ' +
      'Kebutuhan sesuai dengan standar.'
    );
    await page.click('button:has-text("Ya, Setujui")');

    // Verify approved and marked as priority
    await expect(page.locator('text=Status: APPROVED')).toBeVisible();
    await expect(page.locator('text=Prioritas: Ya')).toBeVisible();
  });

  test('Phase 5: Validator Pusat generates analysis report', async ({ page }) => {
    // Login as Validator Pusat
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorPusat.username);
    await page.fill('input[name="password"]', testUsers.validatorPusat.password);
    await page.click('button[type="submit"]');

    // Navigate to reports
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Laporan Analisis');

    // Generate report
    await page.click('button:has-text("Generate Laporan")');
    await page.selectOption('select[name="format"]', 'PDF');
    await page.selectOption('select[name="wilayah"]', '0200'); // DKI Jakarta
    await page.click('button:has-text("Generate")');

    // Wait for document generation
    await expect(page.locator('text=Laporan sedang diproses')).toBeVisible();
    await page.waitForSelector('text=Laporan siap diunduh', { timeout: 10000 });

    // Download report
    const downloadPromise = page.waitForEvent('download');
    await page.click('button:has-text("Unduh PDF")');
    const download = await downloadPromise;

    // Verify download
    expect(download.suggestedFilename()).toContain('laporan-analisis-kebutuhan');
    expect(download.suggestedFilename()).toContain('.pdf');
  });
});
```

### 24.5 Test Suite 2: Pemakaian BMN Complete Flow

```typescript
// tests/e2e/pemakaian-bmn.spec.ts
import { test, expect } from '@playwright/test';
import { testUsers } from './fixtures/users';
import { mockSimanData, mockMySIMKARIData } from './fixtures/integration-data';

test.describe('Pemakaian BMN - Complete Workflow', () => {

  test('Scenario 1: Create permit with multiple BMN items', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to Pemakaian BMN
    await page.click('text=Pemakaian BMN');
    await page.click('text=Buat Izin Baru');

    // Select pegawai
    await page.fill('input[name="nip"]', '199001012020121001');
    await page.click('button:has-text("Cari Pegawai")');

    // Verify pegawai data loaded from MySIMKARI
    await expect(page.locator('text=Budi Santoso')).toBeVisible();
    await expect(page.locator('text=Kepala Sub Bagian Umum')).toBeVisible();
    await expect(page.locator('text=Penata Muda Tk.I')).toBeVisible();
    await expect(page.locator('img[alt="Foto Pegawai"]')).toBeVisible();

    // Add first BMN item
    await page.click('button:has-text("Tambah BMN")');
    await page.fill('input[name="nup_search"]', '0002.03.01.001');
    await page.click('button:has-text("Cari BMN")');

    // Verify BMN availability check
    await expect(page.locator('text=Laptop Dell Latitude')).toBeVisible();
    await expect(page.locator('text=Status: Tersedia')).toBeVisible();
    await page.click('button:has-text("Pilih BMN")');

    // Add second BMN item
    await page.click('button:has-text("Tambah BMN")');
    await page.fill('input[name="nup_search"]', '0001.01.01.001');
    await page.click('button:has-text("Cari BMN")');
    await expect(page.locator('text=Meja Kerja Kayu')).toBeVisible();
    await page.click('button:has-text("Pilih BMN")');

    // Verify multiple BMN added
    await expect(page.locator('text=2 BMN dipilih')).toBeVisible();

    // Set usage period
    await page.fill('input[name="start_date"]', '2026-03-01');
    await page.fill('input[name="end_date"]', '2027-02-28'); // 1 year

    // Generate draft document
    await page.click('button:has-text("Generate Dokumen")');

    // Wait for document generation
    await expect(page.locator('text=Dokumen sedang diproses')).toBeVisible();
    await page.waitForSelector('text=Dokumen siap diunduh', { timeout: 10000 });

    // Verify permit number generated
    await expect(page.locator('text=IZN/2026/KEJARI-JAKARTA-PUSAT/')).toBeVisible();

    // Download draft DOCX
    const downloadPromise = page.waitForEvent('download');
    await page.click('button:has-text("Unduh Draft DOCX")');
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.docx');

    // Verify status changed to DOCUMENT_GENERATED
    await expect(page.locator('text=Status: DOCUMENT_GENERATED')).toBeVisible();
  });

  test('Scenario 2: Upload signed permit and complete', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to pending permits
    await page.click('text=Pemakaian BMN');
    await page.click('text=Daftar Izin');
    await page.selectOption('select[name="status"]', 'DOCUMENT_GENERATED');

    // Open permit
    await page.click('text=Budi Santoso');

    // Upload signed PDF
    await page.click('button:has-text("Upload PDF Bertanda Tangan")');
    await page.setInputFiles('input[type="file"]', 'tests/fixtures/izin-pemakaian-signed.pdf');
    await page.click('button:has-text("Upload")');

    // Verify upload successful
    await expect(page.locator('text=Status: COMPLETED')).toBeVisible();
    await expect(page.locator('text=PDF berhasil diupload')).toBeVisible();

    // Verify signed document can be downloaded
    await expect(page.locator('button:has-text("Unduh PDF Bertanda Tangan")')).toBeVisible();
  });

  test('Scenario 3: Validator Wilayah monitors active permits', async ({ page }) => {
    // Login as Validator Wilayah
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorWilayah.username);
    await page.fill('input[name="password"]', testUsers.validatorWilayah.password);
    await page.click('button[type="submit"]');

    // Navigate to monitoring
    await page.click('text=Pemakaian BMN');
    await page.click('text=Monitoring Izin');

    // Filter by wilayah
    await page.selectOption('select[name="wilayah"]', '0200'); // DKI Jakarta
    await page.selectOption('select[name="status"]', 'COMPLETED');

    // Verify can see active permits
    await expect(page.locator('text=Budi Santoso')).toBeVisible();
    await expect(page.locator('text=Laptop Dell Latitude')).toBeVisible();
    await expect(page.locator('text=Meja Kerja Kayu')).toBeVisible();

    // View permit details
    await page.click('text=Budi Santoso');
    await expect(page.locator('text=Periode: 01 Mar 2026 - 28 Feb 2027')).toBeVisible();
    await expect(page.locator('text=Sisa Waktu: 365 hari')).toBeVisible();
  });

  test('Scenario 4: Renew permit before expiry', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to permits
    await page.click('text=Pemakaian BMN');
    await page.click('text=Daftar Izin');

    // Open permit
    await page.click('text=Budi Santoso');

    // Renew permit
    await page.click('button:has-text("Perpanjang Izin")');

    // Verify renewal form pre-filled
    await expect(page.locator('text=Perpanjangan dari: IZN/2026/')).toBeVisible();
    await expect(page.locator('text=Budi Santoso')).toBeVisible();
    await expect(page.locator('text=2 BMN')).toBeVisible();

    // Set new period
    await page.fill('input[name="start_date"]', '2027-03-01');
    await page.fill('input[name="end_date"]', '2028-02-29');

    // Generate renewal document
    await page.click('button:has-text("Generate Dokumen Perpanjangan")');
    await page.waitForSelector('text=Dokumen siap diunduh', { timeout: 10000 });

    // Verify new permit created
    await expect(page.locator('text=IZN/2027/KEJARI-JAKARTA-PUSAT/')).toBeVisible();
    await expect(page.locator('text=Status: DOCUMENT_GENERATED')).toBeVisible();
  });

  test('Scenario 5: Admin revokes permit', async ({ page }) => {
    // Login as Admin
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.admin.username);
    await page.fill('input[name="password"]', testUsers.admin.password);
    await page.click('button[type="submit"]');

    // Navigate to permits
    await page.click('text=Pemakaian BMN');
    await page.click('text=Daftar Izin');
    await page.selectOption('select[name="status"]', 'COMPLETED');

    // Open permit
    await page.click('text=Budi Santoso');

    // Revoke permit
    await page.click('button:has-text("Cabut Izin")');
    await page.fill('textarea[name="revocation_reason"]',
      'Pegawai dimutasi ke satker lain. BMN dikembalikan ke pool.'
    );
    await page.click('button:has-text("Ya, Cabut Izin")');

    // Verify revoked
    await expect(page.locator('text=Status: REVOKED')).toBeVisible();
    await expect(page.locator('text=Alasan: Pegawai dimutasi')).toBeVisible();

    // Verify BMN now available for others
    await page.goto('/pemakaian-bmn/buat-izin');
    await page.fill('input[name="nip"]', '198501012015121002'); // Different pegawai
    await page.click('button:has-text("Cari Pegawai")');
    await page.click('button:has-text("Tambah BMN")');
    await page.fill('input[name="nup_search"]', '0002.03.01.001'); // Previously used laptop
    await page.click('button:has-text("Cari BMN")');

    // Verify BMN is now available
    await expect(page.locator('text=Status: Tersedia')).toBeVisible();
  });

  test('Scenario 6: Automated expiry reminders', async ({ page }) => {
    // This test simulates the cron job behavior
    // In real implementation, this would be tested via API or background job testing

    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Check notifications
    await page.click('button[aria-label="Notifications"]');

    // Verify expiry reminder notifications
    await expect(page.locator('text=Izin pemakaian akan berakhir dalam 30 hari')).toBeVisible();
    await expect(page.locator('text=Laptop Dell Latitude')).toBeVisible();

    // Navigate to expiring permits
    await page.click('text=Pemakaian BMN');
    await page.click('text=Izin Akan Berakhir');

    // Verify expiring permits listed
    await expect(page.locator('text=Berakhir dalam 30 hari')).toBeVisible();
  });
});
```

### 24.6 Test Suite 3: SK Penghapusan BMN Complete Flow

```typescript
// tests/e2e/sk-penghapusan-bmn.spec.ts
import { test, expect } from '@playwright/test';
import { testUsers } from './fixtures/users';
import { mockSimanData } from './fixtures/integration-data';

test.describe('SK Penghapusan BMN - Complete Workflow', () => {

  test('Scenario 1: Operator Satker creates penghapusan request', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to SK Penghapusan
    await page.click('text=SK Penghapusan BMN');
    await page.click('text=Buat Pengajuan Baru');

    // Add BMN for deletion
    await page.click('button:has-text("Tambah BMN")');
    await page.fill('input[name="nup_search"]', '0001.01.01.002'); // Kursi Rusak
    await page.click('button:has-text("Cari BMN")');

    // Verify BMN details from SIMAN
    await expect(page.locator('text=Kursi Kerja Putar')).toBeVisible();
    await expect(page.locator('text=Kondisi: RUSAK_RINGAN')).toBeVisible();
    await expect(page.locator('text=Tahun: 2019')).toBeVisible();

    // Verify BMN not in active use
    await expect(page.locator('text=Status Pemakaian: Tidak Digunakan')).toBeVisible();

    // Add alasan penghapusan
    await page.fill('textarea[name="alasan_penghapusan"]',
      'BMN dalam kondisi rusak berat dan tidak ekonomis untuk diperbaiki. ' +
      'Sudah melewati masa manfaat dan mengganggu estetika ruangan.'
    );
    await page.click('button:has-text("Pilih BMN")');

    // Verify BMN added
    await expect(page.locator('text=1 BMN dipilih')).toBeVisible();

    // Upload supporting documents
    await page.click('button:has-text("Upload Persyaratan")');
    await page.setInputFiles('input[type="file"]', 'tests/fixtures/berita-acara-penghapusan.pdf');
    await page.click('button:has-text("Upload")');

    // Verify attachment uploaded
    await expect(page.locator('text=berita-acara-penghapusan.pdf')).toBeVisible();

    // Submit to Validator Wilayah
    await page.click('button:has-text("Kirim ke Validator Wilayah")');
    await page.click('button:has-text("Ya, Kirim")');

    // Verify submission successful
    await expect(page.locator('text=Status: SUBMITTED')).toBeVisible();
    await expect(page.locator('text=Pengajuan berhasil dikirim')).toBeVisible();
  });

  test('Scenario 2: Validator Wilayah reviews and forwards', async ({ page }) => {
    // Login as Validator Wilayah
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorWilayah.username);
    await page.fill('input[name="password"]', testUsers.validatorWilayah.password);
    await page.click('button[type="submit"]');

    // Navigate to pending requests
    await page.click('text=SK Penghapusan BMN');
    await page.click('text=Review Pengajuan');
    await page.selectOption('select[name="status"]', 'SUBMITTED');

    // Open request
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');

    // Review details
    await expect(page.locator('text=Kursi Kerja Putar')).toBeVisible();
    await expect(page.locator('text=Kondisi: RUSAK_RINGAN')).toBeVisible();
    await expect(page.locator('text=berita-acara-penghapusan.pdf')).toBeVisible();

    // Download and verify attachment
    const downloadPromise = page.waitForEvent('download');
    await page.click('button:has-text("Unduh Lampiran")');
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe('berita-acara-penghapusan.pdf');

    // Forward to Validator Pusat
    await page.click('button:has-text("Teruskan ke Validator Pusat")');
    await page.fill('textarea[name="catatan"]',
      'Dokumen lengkap dan alasan penghapusan valid. Direkomendasikan untuk disetujui.'
    );
    await page.click('button:has-text("Ya, Teruskan")');

    // Verify forwarded
    await expect(page.locator('text=Status: REVIEWED_WILAYAH')).toBeVisible();
  });

  test('Scenario 3: Validator Pusat reviews and generates SK', async ({ page }) => {
    // Login as Validator Pusat
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorPusat.username);
    await page.fill('input[name="password"]', testUsers.validatorPusat.password);
    await page.click('button[type="submit"]');

    // Navigate to pending requests
    await page.click('text=SK Penghapusan BMN');
    await page.click('text=Review Pengajuan');
    await page.selectOption('select[name="status"]', 'REVIEWED_WILAYAH');

    // Open request
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');

    // View BMN details
    await page.click('button:has-text("Lihat Detail BMN")');
    await expect(page.locator('text=NUP: 0001.01.01.002')).toBeVisible();
    await expect(page.locator('text=Nilai Perolehan: Rp 1.500.000')).toBeVisible();

    // Approve for SK generation
    await page.click('button:has-text("Setujui untuk SK")');
    await page.click('button:has-text("Ya, Setujui")');

    // Verify status changed
    await expect(page.locator('text=Status: REVIEWED_PUSAT')).toBeVisible();

    // Generate SK document
    await page.click('button:has-text("Generate SK Penghapusan")');

    // Wait for document generation
    await expect(page.locator('text=SK sedang diproses')).toBeVisible();
    await page.waitForSelector('text=SK siap diunduh', { timeout: 10000 });

    // Verify SK number generated
    await expect(page.locator('text=SK/2026/')).toBeVisible();

    // Download draft SK DOCX
    const downloadPromise = page.waitForEvent('download');
    await page.click('button:has-text("Unduh Draft SK DOCX")');
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('SK-Penghapusan');
    expect(download.suggestedFilename()).toContain('.docx');

    // Verify status changed
    await expect(page.locator('text=Status: DOCUMENT_GENERATED')).toBeVisible();
  });

  test('Scenario 4: Validator Pusat uploads signed SK', async ({ page }) => {
    // Login as Validator Pusat
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorPusat.username);
    await page.fill('input[name="password"]', testUsers.validatorPusat.password);
    await page.click('button[type="submit"]');

    // Navigate to requests
    await page.click('text=SK Penghapusan BMN');
    await page.click('text=Daftar Pengajuan');
    await page.selectOption('select[name="status"]', 'DOCUMENT_GENERATED');

    // Open request
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');

    // Upload signed SK PDF
    await page.click('button:has-text("Upload SK Bertanda Tangan")');
    await page.setInputFiles('input[type="file"]', 'tests/fixtures/sk-penghapusan-signed.pdf');
    await page.click('button:has-text("Upload")');

    // Verify upload successful
    await expect(page.locator('text=Status: COMPLETED')).toBeVisible();
    await expect(page.locator('text=SK berhasil diupload')).toBeVisible();

    // Verify signed SK can be downloaded
    await expect(page.locator('button:has-text("Unduh SK Bertanda Tangan")')).toBeVisible();
  });

  test('Scenario 5: Operator Satker and Validator Wilayah view signed SK', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to completed requests
    await page.click('text=SK Penghapusan BMN');
    await page.click('text=Daftar Pengajuan');
    await page.selectOption('select[name="status"]', 'COMPLETED');

    // Open request
    await page.click('text=Kursi Kerja Putar');

    // Verify can see SK details
    await expect(page.locator('text=Status: COMPLETED')).toBeVisible();
    await expect(page.locator('text=SK/2026/')).toBeVisible();

    // Download signed SK
    const downloadPromise = page.waitForEvent('download');
    await page.click('button:has-text("Unduh SK Bertanda Tangan")');
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('SK-Penghapusan');
    expect(download.suggestedFilename()).toContain('.pdf');

    // Logout and login as Validator Wilayah
    await page.click('button[aria-label="User menu"]');
    await page.click('text=Logout');

    await page.fill('input[name="username"]', testUsers.validatorWilayah.username);
    await page.fill('input[name="password"]', testUsers.validatorWilayah.password);
    await page.click('button[type="submit"]');

    // Navigate to completed requests
    await page.click('text=SK Penghapusan BMN');
    await page.click('text=Daftar Pengajuan');
    await page.selectOption('select[name="status"]', 'COMPLETED');

    // Verify can also see and download SK
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');
    await expect(page.locator('button:has-text("Unduh SK Bertanda Tangan")')).toBeVisible();
  });

  test('Scenario 6: Validation - Cannot delete BMN in active use', async ({ page }) => {
    // First, create an active permit for a BMN
    // (This would be done in setup or previous test)

    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Try to create penghapusan for BMN in active use
    await page.click('text=SK Penghapusan BMN');
    await page.click('text=Buat Pengajuan Baru');
    await page.click('button:has-text("Tambah BMN")');
    await page.fill('input[name="nup_search"]', '0002.03.01.001'); // Laptop with active permit
    await page.click('button:has-text("Cari BMN")');

    // Verify validation error
    await expect(page.locator('text=BMN sedang digunakan')).toBeVisible();
    await expect(page.locator('text=Izin Pemakaian: IZN/2026/')).toBeVisible();
    await expect(page.locator('text=Pegawai: Budi Santoso')).toBeVisible();
    await expect(page.locator('text=Berakhir: 28 Feb 2027')).toBeVisible();

    // Verify cannot select BMN
    await expect(page.locator('button:has-text("Pilih BMN")')).toBeDisabled();
  });
});
```

### 24.7 Test Suite 4: Kebutuhan Pakaian Dinas Complete Flow

```typescript
// tests/e2e/kebutuhan-pakaian-dinas.spec.ts
import { test, expect } from '@playwright/test';
import { testUsers } from './fixtures/users';
import { mockMySIMKARIData } from './fixtures/integration-data';

test.describe('Kebutuhan Pakaian Dinas - Complete Workflow', () => {

  test('Scenario 1: Validator Pusat creates pakaian dinas period', async ({ page }) => {
    // Login as Validator Pusat
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorPusat.username);
    await page.fill('input[name="password"]', testUsers.validatorPusat.password);
    await page.click('button[type="submit"]');

    // Navigate to Pakaian Dinas Period Management
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Pakaian Dinas');
    await page.click('text=Kelola Periode');

    // Create new period
    await page.click('button:has-text("Buat Periode Baru")');
    await page.fill('input[name="nama_period"]', 'Pakaian Dinas TA 2026');
    await page.fill('input[name="start_date"]', '2026-03-01');
    await page.fill('input[name="end_date"]', '2026-06-30');
    await page.fill('input[name="deadline"]', '2026-06-30T23:59');
    await page.click('button:has-text("Simpan")');

    // Configure eligible satkers using hierarchical tree
    await page.click('button:has-text("Konfigurasi Satker")');

    // Expand wilayah tree
    await page.click('text=0200 - DKI Jakarta'); // Expand
    await page.click('input[type="checkbox"][value="kejari-jakarta-pusat"]');
    await page.click('input[type="checkbox"][value="kejari-jakarta-selatan"]');

    await page.click('text=0300 - Jawa Barat'); // Expand
    await page.click('input[type="checkbox"][value="kejari-bandung"]');

    await page.click('button:has-text("Simpan Satker Eligible")');
    await expect(page.locator('text=3 Satker dipilih')).toBeVisible();

    // Configure pakaian types and specifications
    await page.click('button:has-text("Konfigurasi Jenis Pakaian")');

    // Add PDH (Pakaian Dinas Harian)
    await page.click('button:has-text("Tambah Jenis")');
    await page.fill('input[name="nama_pakaian"]', 'PDH (Pakaian Dinas Harian)');

    // Add specifications with categories
    await page.click('button:has-text("Tambah Spesifikasi")');
    await page.fill('input[name="nama_spesifikasi"]', 'Kemeja PDH');
    await page.selectOption('select[name="kategori"]', 'BAJU');
    await page.fill('input[name="ukuran_options"]', 'S, M, L, XL, XXL, XXXL');
    await page.click('button:has-text("Simpan Spesifikasi")');

    await page.click('button:has-text("Tambah Spesifikasi")');
    await page.fill('input[name="nama_spesifikasi"]', 'Celana PDH');
    await page.selectOption('select[name="kategori"]', 'CELANA');
    await page.fill('input[name="ukuran_options"]', '28, 30, 32, 34, 36, 38, 40');
    await page.click('button:has-text("Simpan Spesifikasi")');

    await page.click('button:has-text("Tambah Spesifikasi")');
    await page.fill('input[name="nama_spesifikasi"]', 'Sepatu PDH');
    await page.selectOption('select[name="kategori"]', 'SEPATU');
    await page.fill('input[name="ukuran_options"]', '38, 39, 40, 41, 42, 43, 44');
    await page.click('button:has-text("Simpan Spesifikasi")');

    await page.click('button:has-text("Simpan Jenis Pakaian")');

    // Activate period
    await page.click('button:has-text("Aktifkan Periode")');
    await expect(page.locator('text=Status: ACTIVE')).toBeVisible();
  });

  test('Scenario 2: Operator Satker inputs pegawai ukuran', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to Pakaian Dinas submission
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Pakaian Dinas');
    await page.click('text=Input Ukuran');

    // Verify active period shown
    await expect(page.locator('text=Pakaian Dinas TA 2026')).toBeVisible();

    // Load pegawai list from MySIMKARI
    await page.click('button:has-text("Muat Data Pegawai")');
    await expect(page.locator('text=45 pegawai dimuat')).toBeVisible();

    // Filter pegawai
    await page.selectOption('select[name="jenis_pegawai"]', 'Jaksa');
    await page.selectOption('select[name="jenis_kelamin"]', 'L');

    // Input ukuran for first pegawai (male)
    await page.click('tr:has-text("Budi Santoso") button:has-text("Input Ukuran")');

    // Verify pegawai data snapshot
    await expect(page.locator('text=NIP: 199001012020121001')).toBeVisible();
    await expect(page.locator('text=Jabatan: Kepala Sub Bagian Umum')).toBeVisible();
    await expect(page.locator('text=Pangkat: Penata Muda Tk.I')).toBeVisible();
    await expect(page.locator('text=Golongan: III/b')).toBeVisible();

    // Input ukuran by category
    await page.selectOption('select[name="ukuran_kemeja"]', 'L');
    await page.selectOption('select[name="ukuran_celana"]', '32');
    await page.selectOption('select[name="ukuran_sepatu"]', '42');

    // No hijab option for male
    await expect(page.locator('input[name="with_hijab"]')).not.toBeVisible();

    await page.click('button:has-text("Simpan Ukuran")');
    await expect(page.locator('text=Ukuran berhasil disimpan')).toBeVisible();

    // Input ukuran for female pegawai
    await page.selectOption('select[name="jenis_kelamin"]', 'P');
    await page.click('tr:has-text("Dewi Lestari") button:has-text("Input Ukuran")');

    // Input ukuran with hijab option
    await page.selectOption('select[name="ukuran_kemeja"]', 'M');
    await page.selectOption('select[name="ukuran_celana"]', '30');
    await page.selectOption('select[name="ukuran_sepatu"]', '38');
    await page.check('input[name="with_hijab"]'); // Hijab option for female

    await page.click('button:has-text("Simpan Ukuran")');

    // Verify progress
    await expect(page.locator('text=2 dari 45 pegawai sudah diinput')).toBeVisible();
  });

  test('Scenario 3: Operator Satker submits to Validator Wilayah', async ({ page }) => {
    // Login as Operator Satker
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Navigate to submission
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Pakaian Dinas');
    await page.click('text=Input Ukuran');

    // Verify all pegawai have ukuran
    await expect(page.locator('text=45 dari 45 pegawai sudah diinput')).toBeVisible();

    // Submit to Validator Wilayah (Kejati)
    await page.click('button:has-text("Kirim ke Kejati")');
    await page.click('button:has-text("Ya, Kirim")');

    // Verify submission successful
    await expect(page.locator('text=Status: SUBMITTED_TO_KEJATI')).toBeVisible();
    await expect(page.locator('text=Berhasil dikirim ke Kejati')).toBeVisible();
  });

  test('Scenario 4: Validator Wilayah reviews and forwards to Validator Pusat', async ({ page }) => {
    // Login as Validator Wilayah
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorWilayah.username);
    await page.fill('input[name="password"]', testUsers.validatorWilayah.password);
    await page.click('button[type="submit"]');

    // Navigate to review
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Pakaian Dinas');
    await page.click('text=Review Pengajuan');

    // Filter by status
    await page.selectOption('select[name="status"]', 'SUBMITTED_TO_KEJATI');

    // Open submission
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');

    // Review data
    await expect(page.locator('text=45 pegawai')).toBeVisible();
    await expect(page.locator('text=Jaksa: 20')).toBeVisible();
    await expect(page.locator('text=TU: 25')).toBeVisible();

    // View detailed list
    await page.click('button:has-text("Lihat Daftar Pegawai")');
    await expect(page.locator('text=Budi Santoso')).toBeVisible();
    await expect(page.locator('text=Kemeja: L, Celana: 32, Sepatu: 42')).toBeVisible();

    // Forward to Validator Pusat (Kejagung)
    await page.click('button:has-text("Teruskan ke Kejagung")');
    await page.fill('textarea[name="catatan"]', 'Data sudah lengkap dan sesuai.');
    await page.click('button:has-text("Ya, Teruskan")');

    // Verify forwarded
    await expect(page.locator('text=Status: SUBMITTED_TO_KEJAGUNG')).toBeVisible();
  });

  test('Scenario 5: Validator Pusat approves and generates reports', async ({ page }) => {
    // Login as Validator Pusat
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorPusat.username);
    await page.fill('input[name="password"]', testUsers.validatorPusat.password);
    await page.click('button[type="submit"]');

    // Navigate to review
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Pakaian Dinas');
    await page.click('text=Review Pengajuan');

    // Filter by status
    await page.selectOption('select[name="status"]', 'SUBMITTED_TO_KEJAGUNG');

    // Open submission
    await page.click('text=Kejaksaan Negeri Jakarta Pusat');

    // Approve
    await page.click('button:has-text("Setujui")');
    await page.click('button:has-text("Ya, Setujui")');

    // Verify approved and data updated to master
    await expect(page.locator('text=Status: COMPLETED')).toBeVisible();
    await expect(page.locator('text=Data berhasil diupdate ke master pegawai_pakaian_dinas')).toBeVisible();

    // Generate Laporan Daftar (individual list)
    await page.click('button:has-text("Generate Laporan")');
    await page.selectOption('select[name="jenis_laporan"]', 'DAFTAR');
    await page.selectOption('select[name="format"]', 'PDF');

    // Apply filters
    await page.selectOption('select[name="jenis_pegawai"]', 'Jaksa');
    await page.selectOption('select[name="jenis_kelamin"]', 'L');

    await page.click('button:has-text("Generate")');
    await page.waitForSelector('text=Laporan siap diunduh', { timeout: 10000 });

    // Download Laporan Daftar
    const downloadDaftar = page.waitForEvent('download');
    await page.click('button:has-text("Unduh PDF")');
    const daftarPdf = await downloadDaftar;
    expect(daftarPdf.suggestedFilename()).toContain('Laporan-Daftar');

    // Generate Laporan Rekap (aggregated summary)
    await page.click('button:has-text("Generate Laporan")');
    await page.selectOption('select[name="jenis_laporan"]', 'REKAP');
    await page.selectOption('select[name="format"]', 'EXCEL');
    await page.click('button:has-text("Generate")');
    await page.waitForSelector('text=Laporan siap diunduh', { timeout: 10000 });

    // Download Laporan Rekap
    const downloadRekap = page.waitForEvent('download');
    await page.click('button:has-text("Unduh Excel")');
    const rekapExcel = await downloadRekap;
    expect(rekapExcel.suggestedFilename()).toContain('Laporan-Rekap');
    expect(rekapExcel.suggestedFilename()).toContain('.xlsx');
  });

  test('Scenario 6: Validator Wilayah returns for revision', async ({ page }) => {
    // Login as Validator Wilayah
    await page.goto('/login');
    await page.fill('input[name="username"]', testUsers.validatorWilayah.username);
    await page.fill('input[name="password"]', testUsers.validatorWilayah.password);
    await page.click('button[type="submit"]');

    // Navigate to review
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Pakaian Dinas');
    await page.click('text=Review Pengajuan');

    // Open submission
    await page.click('text=Kejaksaan Negeri Jakarta Selatan');

    // Return for revision
    await page.click('button:has-text("Kembalikan untuk Revisi")');
    await page.fill('textarea[name="catatan_revisi"]',
      'Masih ada 5 pegawai yang belum diinput ukurannya. ' +
      'Mohon dilengkapi sebelum diajukan kembali.'
    );
    await page.click('button:has-text("Ya, Kembalikan")');

    // Verify returned
    await expect(page.locator('text=Status: REVISION_REQUIRED')).toBeVisible();

    // Logout and login as Operator Satker
    await page.click('button[aria-label="User menu"]');
    await page.click('text=Logout');

    await page.fill('input[name="username"]', testUsers.operatorSatker.username);
    await page.fill('input[name="password"]', testUsers.operatorSatker.password);
    await page.click('button[type="submit"]');

    // Check notification
    await page.click('button[aria-label="Notifications"]');
    await expect(page.locator('text=Pengajuan pakaian dinas dikembalikan untuk revisi')).toBeVisible();

    // Navigate to submission
    await page.click('text=Kebutuhan BMN');
    await page.click('text=Pakaian Dinas');
    await page.click('text=Input Ukuran');

    // View revision notes
    await expect(page.locator('text=Catatan Revisi:')).toBeVisible();
    await expect(page.locator('text=Masih ada 5 pegawai yang belum diinput')).toBeVisible();

    // Complete missing data and resubmit
    // (Implementation continues...)
  });
});
```

### 24.8 Test Configuration and Setup

**Playwright Configuration:**
```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false, // Run sequentially for workflow tests
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : 1,
  reporter: [
    ['html'],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/junit.xml' }]
  ],

  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
  ],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
```

**Test Database Setup:**
```bash
# tests/e2e/setup/database.sh
#!/bin/bash

# Create test database
psql -U postgres -c "CREATE DATABASE simpelv2_test;"

# Run migrations
cd layanan/perlengkapan
DATABASE_URL=postgres://postgres:password@localhost:5432/simpelv2_test cargo run --bin migrate

# Seed test data
psql -U postgres -d simpelv2_test -f tests/e2e/fixtures/seed-data.sql
```

**Test Data Seeding:**
```sql
-- tests/e2e/fixtures/seed-data.sql

-- Insert test users
INSERT INTO authenc.users (id, username, password_hash, nip, nama, role, satker_id, created_at)
VALUES
  ('11111111-1111-1111-1111-111111111111', 'operator.kejari.jakarta', '$argon2id$...', '199001012020121001', 'Budi Santoso', 'operator_satker', 'kejari-jakarta-pusat', NOW()),
  ('22222222-2222-2222-2222-222222222222', 'validator.kejati.jakarta', '$argon2id$...', '198501012015121001', 'Siti Aminah', 'validator_wilayah', 'kejati-dki-jakarta', NOW()),
  ('33333333-3333-3333-3333-333333333333', 'validator.kejagung', '$argon2id$...', '198001012010121001', 'Ahmad Hidayat', 'validator_pusat', 'kejaksaan-agung', NOW()),
  ('44444444-4444-4444-4444-444444444444', 'admin.simpel', '$argon2id$...', '197501012005121001', 'Super Admin', 'admin', 'kejaksaan-agung', NOW());

-- Insert test satkers
INSERT INTO authenc.satkers (id, kode, nama, wilayah_code, wilayah_nama, created_at)
VALUES
  ('kejari-jakarta-pusat', 'KEJARI-JKT-PST', 'Kejaksaan Negeri Jakarta Pusat', '0200', 'DKI Jakarta', NOW()),
  ('kejari-jakarta-selatan', 'KEJARI-JKT-SEL', 'Kejaksaan Negeri Jakarta Selatan', '0200', 'DKI Jakarta', NOW()),
  ('kejati-dki-jakarta', 'KEJATI-DKI', 'Kejaksaan Tinggi DKI Jakarta', '0200', 'DKI Jakarta', NOW()),
  ('kejaksaan-agung', 'KEJAGUNG', 'Kejaksaan Agung Republik Indonesia', '0100', 'Pusat', NOW());

-- Insert test BMN master data
INSERT INTO perlengkapan.ms_barang (id, kode, nama, kategori, created_at)
VALUES
  (gen_random_uuid(), '01.01.01.001', 'Meja Kerja Kayu', 'Peralatan Kantor', NOW()),
  (gen_random_uuid(), '01.01.01.002', 'Kursi Kerja Putar', 'Peralatan Kantor', NOW()),
  (gen_random_uuid(), '02.03.01.001', 'Laptop Dell Latitude', 'Peralatan Komputer', NOW());
```

### 24.9 CI/CD Integration

**GitHub Actions Workflow:**
```yaml
# .github/workflows/e2e-tests.yml
name: E2E Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  e2e-tests:
    runs-on: self-hosted

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'

      - name: Install dependencies
        run: |
          npm install
          cargo build --release

      - name: Setup test database
        run: |
          ./tests/e2e/setup/database.sh
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/simpelv2_test

      - name: Install Playwright
        run: npx playwright install --with-deps

      - name: Run E2E tests
        run: npx playwright test
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/simpelv2_test
          REDIS_URL: redis://localhost:6379

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 30
```

---

**End of End-to-End Testing Section**
**Document Version:** 3.1.0
**Last Updated:** February 11, 2026
**Status:** Complete with Comprehensive E2E Testing Strategy
