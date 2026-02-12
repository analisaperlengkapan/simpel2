# Implementation Tasks: SIMPEL Completion - Total Refactor

## Overview

This task list implements the complete SIMPEL system based on the approved requirements and design documents. The implementation follows a phased approach, starting with foundational infrastructure and progressing through core business modules to advanced features and end-to-end testing.

**Total Estimated Effort:** ~320 hours (8 weeks with 2 developers)

**Priority Legend:**
- 🔴 Critical Path - Must complete before dependent tasks
- 🟡 High Priority - Core functionality
- 🟢 Medium Priority - Important but not blocking
- 🔵 Low Priority - Nice to have, can defer

---

## Phase 1: Foundation & Infrastructure (40 hours)

### 1. Database Schema & Migrations
- [ ] 1.1 Create database migration scripts for all schemas (8h) 🔴
  - [ ] 1.1.1 Create perlengkapan schema with naming conventions
  - [ ] 1.1.2 Create integrasi schema for SIMAN/MySIMKARI data
  - [ ] 1.1.3 Create workflow tables (ms_aktivitas_bmn, workflow_instances)
  - [ ] 1.1.4 Create kebutuhan BMN tables (period, submission, items, attachments)
  - [ ] 1.1.5 Create pemakaian BMN tables (permits, items)
  - [ ] 1.1.6 Create penghapusan BMN tables (requests, items, attachments)
  - [ ] 1.1.7 Create indexes and constraints
  - [ ] 1.1.8 Add table and column comments
- [ ] 1.2 Create database views for dashboards (4h) 🟡
  - [ ] 1.2.1 Create v_gap_analysis view
  - [ ] 1.2.2 Create v_workflow_metrics view
  - [ ] 1.2.3 Create v_asset_utilization view

### 2. Shared Libraries Enhancement
- [ ] 2.1 Enhance lib-common with new modules (12h) 🔴
  - [ ] 2.1.1 Implement audit logging module (AuditEvent types, AuditLogger)
  - [ ] 2.1.2 Implement cache module (CacheManager with Redis, TTL, sensitivity levels)
  - [ ] 2.1.3 Implement workflow module (WorkflowEngine, state transitions)
  - [ ] 2.1.4 Implement notification module (NotificationService, types)
  - [ ] 2.1.5 Implement storage module (S3Client for MinIO)
- [ ] 2.2 Enhance lib-perlengkapan with domain logic (8h) 🔴
  - [ ] 2.2.1 Implement gap analysis algorithm (GapAnalyzer)
  - [ ] 2.2.2 Implement prioritization engine (PrioritizationEngine, scoring)
  - [ ] 2.2.3 Implement domain models (KebutuhanBmn, PakaianDinas, etc.)
  - [ ] 2.2.4 Implement kode barang utilities
- [ ] 2.3 Enhance lib-ui with new components (8h) 🟡
  - [ ] 2.3.1 Create MetricCard component
  - [ ] 2.3.2 Create BarChart and PieChart components
  - [ ] 2.3.3 Create GapAnalysisTable component
  - [ ] 2.3.4 Create WorkflowStatusBadge component
  - [ ] 2.3.5 Create FileUpload component with progress

---

## Phase 2: Integration Services (32 hours)

### 3. Integrasi Service Implementation
\- [ ] 3.1 Create SIMAN API client (8h) 🔴
  - [ ] 3.1.1 Implement SimanClient with authentication (Secreton integration)
  - [ ] 3.1.2 Implement get_assets_by_satker endpoint
  - [ ] 3.1.3 Implement get_assets_incremental endpoint
  - [ ] 3.1.4 Implement retry logic with exponential backoff
  - [ ] 3.1.5 Implement circuit breaker pattern
- [ ] 3.2 Create MySIMKARI API client (6h) 🔴
  - [ ] 3.2.1 Implement MySIMKARIClient with authentication
  - [ ] 3.2.2 Implement get_pegawai_by_satker endpoint
  - [ ] 3.2.3 Implement get_pegawai_summary endpoint
  - [ ] 3.2.4 Implement get_pegawai_with_photo endpoint
- [ ] 3.3 Implement sync services (10h) 🔴
  - [ ] 3.3.1 Implement SimanSyncService with cron scheduler
  - [ ] 3.3.2 Implement full sync (daily at 02:00 WIB)
  - [ ] 3.3.3 Implement incremental sync (every 6 hours)
  - [ ] 3.3.4 Implement MySIMKARISyncService with cron scheduler
  - [ ] 3.3.5 Implement API call logging to integrasi.api_call_log
- [ ] 3.4 Implement gRPC service (8h) 🔴
  - [ ] 3.4.1 Define proto files (integrasi.proto)
  - [ ] 3.4.2 Implement GetSimanAssets RPC
  - [ ] 3.4.3 Implement GetMySIMKARIPegawai RPC
  - [ ] 3.4.4 Implement GetMySIMKARIPegawaiSummary RPC
  - [ ] 3.4.5 Implement GetSyncStatus RPC
  - [ ] 3.4.6 Implement TriggerSync RPC

---

## Phase 3: Core Services (48 hours)

### 4. Workflow Service Implementation
- [ ] 4.1 Implement workflow engine (12h) 🔴
  - [ ] 4.1.1 Implement WorkflowConfig with state transitions
  - [ ] 4.1.2 Implement WorkflowEngine with validation
  - [ ] 4.1.3 Implement transition method with role validation
  - [ ] 4.1.4 Implement SLA checking and escalation
  - [ ] 4.1.5 Integrate with audit logging
  - [ ] 4.1.6 Integrate with notification service
- [ ] 4.2 Implement parallel approval engine (6h) 🟡
  - [ ] 4.2.1 Implement ParallelApprovalEngine
  - [ ] 4.2.2 Implement create_parallel_approval
  - [ ] 4.2.3 Implement record_approval with threshold checking
- [ ] 4.3 Implement workflow gRPC service (6h) 🔴
  - [ ] 4.3.1 Define proto files (workflow.proto)
  - [ ] 4.3.2 Implement CreateWorkflowInstance RPC
  - [ ] 4.3.3 Implement TransitionWorkflow RPC
  - [ ] 4.3.4 Implement GetWorkflowStatus RPC

### 5. Dokumen Service Implementation
- [ ] 5.1 Implement template engine (8h) 🔴
  - [ ] 5.1.1 Implement TemplateEngine with Tera
  - [ ] 5.1.2 Load templates from database
  - [ ] 5.1.3 Implement template rendering with context
- [ ] 5.2 Implement document generators (10h) 🔴
  - [ ] 5.2.1 Implement PdfGenerator with headless Chrome
  - [ ] 5.2.2 Implement ExcelGenerator with rust_xlsxwriter
  - [ ] 5.2.3 Implement DOCX generator for permits and SK
  - [ ] 5.2.4 Create templates for kebutuhan BMN analysis reports
  - [ ] 5.2.5 Create templates for pemakaian BMN permits
  - [ ] 5.2.6 Create templates for SK Penghapusan BMN
- [ ] 5.3 Implement storage integration (4h) 🔴
  - [ ] 5.3.1 Implement DocumentStorage with MinIO/S3
  - [ ] 5.3.2 Implement store_document with SHA-256 checksum
  - [ ] 5.3.3 Implement get_document
- [ ] 5.4 Implement dokumen gRPC service (6h) 🔴
  - [ ] 5.4.1 Define proto files (dokumen.proto)
  - [ ] 5.4.2 Implement GenerateDocument RPC
  - [ ] 5.4.3 Implement GetDocument RPC
  - [ ] 5.4.4 Implement ListDocuments RPC

### 6. Notifikasi Service Implementation
- [ ] 6.1 Implement notification channels (10h) 🔴
  - [ ] 6.1.1 Implement InAppNotificationChannel
  - [ ] 6.1.2 Implement EmailNotificationChannel with SMTP
  - [ ] 6.1.3 Implement notification preferences
  - [ ] 6.1.4 Implement mark_as_read and get_unread_count
- [ ] 6.2 Implement notification scheduler (6h) 🟡
  - [ ] 6.2.1 Implement NotificationScheduler with cron
  - [ ] 6.2.2 Implement daily digest (08:00 WIB)
  - [ ] 6.2.3 Implement izin expiry reminders (H-30, H-14, H-7)
- [ ] 6.3 Implement notifikasi gRPC service (4h) 🔴
  - [ ] 6.3.1 Define proto files (notifikasi.proto)
  - [ ] 6.3.2 Implement SendNotification RPC
  - [ ] 6.3.3 Implement GetNotifications RPC
  - [ ] 6.3.4 Implement MarkAsRead RPC

---

## Phase 4: Business Modules (80 hours)

### 7. Kebutuhan BMN Module
- [ ] 7.1 Implement period management (12h) 🔴
  - [ ] 7.1.1 Implement POST /api/v1/kebutuhan/periods (create period)
  - [ ] 7.1.2 Implement POST /api/v1/kebutuhan/periods/{id}/eligible-bmn
  - [ ] 7.1.3 Implement POST /api/v1/kebutuhan/periods/{id}/eligible-satkers
  - [ ] 7.1.4 Implement GET /api/v1/kebutuhan/periods/active
  - [ ] 7.1.5 Implement period validation (dates, constraints)
- [ ] 7.2 Implement submission workflow (16h) 🔴
  - [ ] 7.2.1 Implement POST /api/v1/kebutuhan/submissions (create submission)
  - [ ] 7.2.2 Implement POST /api/v1/kebutuhan/submissions/{id}/items
  - [ ] 7.2.3 Implement POST /api/v1/kebutuhan/submissions/{id}/attachments
  - [ ] 7.2.4 Implement POST /api/v1/kebutuhan/submissions/{id}/submit
  - [ ] 7.2.5 Implement validation (deadline, eligible BMN, eligible satker)
  - [ ] 7.2.6 Integrate with workflow service (DRAFT → SUBMITTED)
- [ ] 7.3 Implement review workflow (12h) 🔴
  - [ ] 7.3.1 Implement POST /api/v1/kebutuhan/submissions/{id}/forward (Validator Wilayah)
  - [ ] 7.3.2 Implement POST /api/v1/kebutuhan/submissions/{id}/return (Validator Wilayah)
  - [ ] 7.3.3 Implement GET /api/v1/kebutuhan/submissions/{id}/analysis (Validator Pusat)
  - [ ] 7.3.4 Integrate with integrasi service (SIMAN, MySIMKARI data)
  - [ ] 7.3.5 Implement POST /api/v1/kebutuhan/submissions/{id}/approve
  - [ ] 7.3.6 Implement POST /api/v1/kebutuhan/submissions/{id}/reject
- [ ] 7.4 Implement report generation (8h) 🟡
  - [ ] 7.4.1 Implement POST /api/v1/kebutuhan/reports/generate
  - [ ] 7.4.2 Integrate with dokumen service (PDF, DOCX, XLSX)
  - [ ] 7.4.3 Implement report templates
- [ ] 7.5 Implement frontend UI (16h) 🔴
  - [ ] 7.5.1 Create period management page (Validator Pusat)
  - [ ] 7.5.2 Create submission form page (Operator Satker)
  - [ ] 7.5.3 Create review page (Validator Wilayah)
  - [ ] 7.5.4 Create analysis page (Validator Pusat)
  - [ ] 7.5.5 Integrate with REST API endpoints
  - [ ] 7.5.6 Implement file upload with progress
- [ ] 7.6 Implement pakaian dinas workflow (16h) 🟡
  - [ ] 7.6.1 Implement period creation with satker tree selection
  - [ ] 7.6.2 Implement pegawai ukuran input (gender-specific, hijab option)
  - [ ] 7.6.3 Implement 3-level hierarchical approval (Kejari→Kejati→Kejagung)
  - [ ] 7.6.4 Implement revision workflow
  - [ ] 7.6.5 Implement report generation (Laporan Daftar, Laporan Rekap)
  - [ ] 7.6.6 Implement frontend UI for pakaian dinas

### 8. Pemakaian BMN Module
- [ ] 8.1 Implement permit creation workflow (14h) 🔴
  - [ ] 8.1.1 Implement POST /api/v1/pemakaian/permits (create permit)
  - [ ] 8.1.2 Implement POST /api/v1/pemakaian/permits/{id}/pegawai
  - [ ] 8.1.3 Integrate with integrasi service (MySIMKARI pegawai data + photo)
  - [ ] 8.1.4 Implement GET /api/v1/pemakaian/bmn/available (with active permit check)
  - [ ] 8.1.5 Implement POST /api/v1/pemakaian/permits/{id}/bmn (multiple BMN)
  - [ ] 8.1.6 Implement validation (one BMN = one active permit)
  - [ ] 8.1.7 Implement POST /api/v1/pemakaian/permits/{id}/period
- [ ] 8.2 Implement document generation (10h) 🔴
  - [ ] 8.2.1 Implement POST /api/v1/pemakaian/permits/{id}/generate-document
  - [ ] 8.2.2 Integrate with dokumen service (DOCX generation)
  - [ ] 8.2.3 Implement permit template (page 1: pegawai + photo, page 2+: BMN table)
  - [ ] 8.2.4 Implement auto-generate permit number (IZN/{YEAR}/{SATKER}/{SEQUENCE})
  - [ ] 8.2.5 Implement POST /api/v1/pemakaian/permits/{id}/upload-signed
- [ ] 8.3 Implement expiry management (8h) 🔴
  - [ ] 8.3.1 Implement cron job for expiry checks (daily at 00:00 WIB)
  - [ ] 8.3.2 Implement H-30, H-14, H-7 reminders
  - [ ] 8.3.3 Implement auto-expire permits past end_date
  - [ ] 8.3.4 Integrate with notifikasi service
- [ ] 8.4 Implement renewal and revocation (8h) 🟡
  - [ ] 8.4.1 Implement POST /api/v1/pemakaian/permits/{id}/renew
  - [ ] 8.4.2 Implement POST /api/v1/pemakaian/permits/{id}/revoke
  - [ ] 8.4.3 Implement validation and history tracking
- [ ] 8.5 Implement monitoring dashboard (6h) 🟡
  - [ ] 8.5.1 Implement GET /api/v1/pemakaian/reports/active-permits
  - [ ] 8.5.2 Implement GET /api/v1/pemakaian/reports/utilization
  - [ ] 8.5.3 Implement GET /api/v1/pemakaian/reports/history/bmn/{nup}
  - [ ] 8.5.4 Implement GET /api/v1/pemakaian/reports/history/pegawai/{nip}
- [ ] 8.6 Implement frontend UI (12h) 🔴
  - [ ] 8.6.1 Create permit creation page
  - [ ] 8.6.2 Create BMN selection with availability check
  - [ ] 8.6.3 Create document download and upload page
  - [ ] 8.6.4 Create monitoring dashboard (Validator Wilayah, Validator Pusat)
  - [ ] 8.6.5 Integrate with REST API endpoints
- [ ] 8.7 Implement pemakaian gRPC service (4h) 🔴
  - [ ] 8.7.1 Define proto files (pemakaian.proto)
  - [ ] 8.7.2 Implement CheckBMNActivePermit RPC (for penghapusan integration)

### 9. Penghapusan BMN Module
- [ ] 9.1 Implement request workflow (14h) 🔴
  - [ ] 9.1.1 Implement POST /api/v1/penghapusan/requests (create request)
  - [ ] 9.1.2 Implement POST /api/v1/penghapusan/requests/{id}/bmn
  - [ ] 9.1.3 Integrate with pemakaian service (validate BMN not in active use)
  - [ ] 9.1.4 Implement POST /api/v1/penghapusan/requests/{id}/attachments
  - [ ] 9.1.5 Implement POST /api/v1/penghapusan/requests/{id}/submit
  - [ ] 9.1.6 Integrate with workflow service (DRAFT → SUBMITTED)
- [ ] 9.2 Implement review workflow (10h) 🔴
  - [ ] 9.2.1 Implement POST /api/v1/penghapusan/requests/{id}/forward (Validator Wilayah)
  - [ ] 9.2.2 Implement POST /api/v1/penghapusan/requests/{id}/return (Validator Wilayah)
  - [ ] 9.2.3 Implement POST /api/v1/penghapusan/requests/{id}/approve (Validator Pusat)
  - [ ] 9.2.4 Integrate with integrasi service (SIMAN BMN details)
- [ ] 9.3 Implement SK document generation (10h) 🔴
  - [ ] 9.3.1 Implement POST /api/v1/penghapusan/requests/{id}/generate-sk
  - [ ] 9.3.2 Integrate with dokumen service (DOCX generation)
  - [ ] 9.3.3 Implement SK template (official format, BMN table, legal basis)
  - [ ] 9.3.4 Implement auto-generate SK number (SK/{YEAR}/{SEQUENCE})
  - [ ] 9.3.5 Implement POST /api/v1/penghapusan/requests/{id}/upload-signed-sk
  - [ ] 9.3.6 Implement GET /api/v1/penghapusan/requests/{id}/signed-sk
- [ ] 9.4 Implement frontend UI (12h) 🔴
  - [ ] 9.4.1 Create request creation page (Operator Satker)
  - [ ] 9.4.2 Create review page (Validator Wilayah)
  - [ ] 9.4.3 Create SK generation page (Validator Pusat)
  - [ ] 9.4.4 Create signed SK view page (Operator Satker, Validator Wilayah)
  - [ ] 9.4.5 Integrate with REST API endpoints

---

## Phase 5: Dashboard & Advanced Features (40 hours)

### 10. Dashboard Implementation
- [ ] 10.1 Implement portal dashboard (8h) 🟡
  - [ ] 10.1.1 Implement GET /api/v1/dashboard/portal (backend)
  - [ ] 10.1.2 Fetch system metrics, cross-domain metrics, auth metrics
  - [ ] 10.1.3 Fetch integration health from integrasi service
  - [ ] 10.1.4 Create portal dashboard UI (frontend)
- [ ] 10.2 Implement perlengkapan dashboard (12h) 🟡
  - [ ] 10.2.1 Implement GET /api/v1/dashboard/perlengkapan (backend)
  - [ ] 10.2.2 Fetch kebutuhan metrics, gap analysis, workflow metrics
  - [ ] 10.2.3 Fetch asset utilization from integrasi service
  - [ ] 10.2.4 Create perlengkapan dashboard UI (frontend)
  - [ ] 10.2.5 Implement drill-down (national → wilayah → satker)
- [ ] 10.3 Implement real-time updates (6h) 🟢
  - [ ] 10.3.1 Implement WebSocket support for dashboard updates
  - [ ] 10.3.2 Implement broadcast_dashboard_update
  - [ ] 10.3.3 Integrate with frontend for real-time metrics

### 11. Search Implementation
- [ ] 11.1 Implement full-text search (8h) 🟡
  - [ ] 11.1.1 Enable pg_trgm extension in PostgreSQL
  - [ ] 11.1.2 Implement SearchEngine with SearchQuery
  - [ ] 11.1.3 Implement search_kebutuhan with filters and sorting
  - [ ] 11.1.4 Implement search_pemakaian
  - [ ] 11.1.5 Implement search_penghapusan
- [ ] 11.2 Implement search UI (6h) 🟡
  - [ ] 11.2.1 Create global search component
  - [ ] 11.2.2 Implement search results page with pagination
  - [ ] 11.2.3 Implement filters (status, tahun, satker)

### 12. Export Features
- [ ] 12.1 Implement export endpoints (6h) 🟢
  - [ ] 12.1.1 Implement GET /api/v1/kebutuhan/export (Excel, CSV)
  - [ ] 12.1.2 Implement GET /api/v1/pemakaian/export
  - [ ] 12.1.3 Implement GET /api/v1/penghapusan/export
  - [ ] 12.1.4 Integrate with dokumen service

---

## Phase 6: Testing & Quality Assurance (80 hours)

### 13. Unit Tests
- [ ] 13.1 Write unit tests for lib-common (8h) 🟡
  - [ ] 13.1.1 Test audit logging module
  - [ ] 13.1.2 Test cache module
  - [ ] 13.1.3 Test workflow module
- [ ] 13.2 Write unit tests for lib-perlengkapan (6h) 🟡
  - [ ] 13.2.1 Test gap analysis algorithm
  - [ ] 13.2.2 Test prioritization engine
- [ ] 13.3 Write unit tests for services (16h) 🟡
  - [ ] 13.3.1 Test integrasi service (SIMAN, MySIMKARI clients)
  - [ ] 13.3.2 Test workflow service
  - [ ] 13.3.3 Test dokumen service
  - [ ] 13.3.4 Test notifikasi service
  - [ ] 13.3.5 Test kebutuhan service
  - [ ] 13.3.6 Test pemakaian service
  - [ ] 13.3.7 Test penghapusan service

### 14. Integration Tests
- [ ] 14.1 Write integration tests for workflows (12h) 🟡
  - [ ] 14.1.1 Test kebutuhan BMN complete workflow
  - [ ] 14.1.2 Test pemakaian BMN complete workflow
  - [ ] 14.1.3 Test penghapusan BMN complete workflow
- [ ] 14.2 Write integration tests for gRPC services (8h) 🟡
  - [ ] 14.2.1 Test integrasi gRPC endpoints
  - [ ] 14.2.2 Test workflow gRPC endpoints
  - [ ] 14.2.3 Test dokumen gRPC endpoints
  - [ ] 14.2.4 Test notifikasi gRPC endpoints
  - [ ] 14.2.5 Test pemakaian gRPC endpoints

### 15. End-to-End Tests with Playwright
- [ ] 15.1 Setup Playwright test environment (4h) 🔴
  - [ ] 15.1.1 Install Playwright and dependencies
  - [ ] 15.1.2 Create test user fixtures (operator, validator wilayah, validator pusat, admin)
  - [ ] 15.1.3 Create mock integration data (SIMAN, MySIMKARI)
  - [ ] 15.1.4 Setup database seeding scripts
- [ ] 15.2 Write E2E tests for Kebutuhan BMN (10h) 🔴
  - [ ] 15.2.1 Test Phase 1: Validator Pusat initiates period and configures eligibility
  - [ ] 15.2.2 Test Phase 2: Operator Satker submits kebutuhan BMN
  - [ ] 15.2.3 Test Phase 3: Validator Wilayah reviews and forwards
  - [ ] 15.2.4 Test Phase 4: Validator Pusat analyzes with integrated data and approves
  - [ ] 15.2.5 Test Phase 5: Validator Pusat generates analysis report
- [ ] 15.3 Write E2E tests for Kebutuhan Pakaian Dinas (8h) 🟡
  - [ ] 15.3.1 Test period creation with satker tree selection
  - [ ] 15.3.2 Test pegawai ukuran input (gender-specific, hijab option)
  - [ ] 15.3.3 Test 3-level hierarchical approval workflow
  - [ ] 15.3.4 Test report generation (Laporan Daftar, Laporan Rekap)
- [ ] 15.4 Write E2E tests for Pemakaian BMN (8h) 🔴
  - [ ] 15.4.1 Test permit creation with multiple BMN items
  - [ ] 15.4.2 Test BMN availability check (one BMN = one active permit)
  - [ ] 15.4.3 Test document generation (DOCX with pegawai photo and BMN table)
  - [ ] 15.4.4 Test permit renewal workflow
  - [ ] 15.4.5 Test permit revocation workflow
- [ ] 15.5 Write E2E tests for SK Penghapusan BMN (8h) 🔴
  - [ ] 15.5.1 Test request creation with BMN selection
  - [ ] 15.5.2 Test validation (BMN not in active use)
  - [ ] 15.5.3 Test review workflow (Validator Wilayah → Validator Pusat)
  - [ ] 15.5.4 Test SK document generation
  - [ ] 15.5.5 Test signed SK upload and view
- [ ] 15.6 Configure CI/CD for E2E tests (4h) 🟡
  - [ ] 15.6.1 Create GitHub Actions workflow for Playwright tests
  - [ ] 15.6.2 Setup test database and mock services
  - [ ] 15.6.3 Configure test reporting and artifacts

---

## Phase 7: Deployment & Documentation (20 hours)

### 16. Deployment Configuration
- [ ] 16.1 Create Kubernetes manifests (8h) 🔴
  - [ ] 16.1.1 Create base manifests for all services
  - [ ] 16.1.2 Create staging overlay
  - [ ] 16.1.3 Create production overlay
  - [ ] 16.1.4 Configure Istio routing
  - [ ] 16.1.5 Configure MetalLB IP pools
- [ ] 16.2 Create Docker images (4h) 🔴
  - [ ] 16.2.1 Create Dockerfiles for all services
  - [ ] 16.2.2 Build and push images to registry
  - [ ] 16.2.3 Test images in staging

### 17. Documentation
- [ ] 17.1 Write API documentation (4h) 🟡
  - [ ] 17.1.1 Generate OpenAPI/Swagger specs for all REST endpoints
  - [ ] 17.1.2 Document gRPC proto files
- [ ] 17.2 Write user documentation (4h) 🟡
  - [ ] 17.2.1 Create user guide for Operator Satker
  - [ ] 17.2.2 Create user guide for Validator Wilayah
  - [ ] 17.2.3 Create user guide for Validator Pusat
  - [ ] 17.2.4 Create admin guide

---

## Summary

**Total Tasks:** 17 major tasks, 150+ subtasks
**Estimated Effort:** ~320 hours (8 weeks with 2 developers)

**Critical Path:**
1. Phase 1: Foundation & Infrastructure (40h)
2. Phase 2: Integration Services (32h)
3. Phase 3: Core Services (48h)
4. Phase 4: Business Modules (80h)
5. Phase 6: E2E Testing (50h)
6. Phase 7: Deployment (12h)

**Parallel Work Opportunities:**
- Frontend UI can be developed in parallel with backend API (after API contracts are defined)
- Unit tests can be written in parallel with implementation
- Documentation can be written in parallel with implementation

**Dependencies:**
- Phase 2 depends on Phase 1 (database schema)
- Phase 3 depends on Phase 2 (integration services)
- Phase 4 depends on Phase 3 (core services)
- Phase 6 depends on Phase 4 (business modules)
- Phase 7 depends on Phase 6 (testing complete)

---

**Document Version:** 1.0.0
**Created:** February 11, 2026
**Status:** Ready for Implementation
