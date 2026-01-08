# Requirements Document

## Introduction

Sistem Analisis Kebutuhan BMN (Barang Milik Negara) adalah modul untuk mengelola proses pengumpulan, validasi, dan analisis kebutuhan BMN di lingkungan Kejaksaan Republik Indonesia. Sistem ini mendukung alur kerja berjenjang dari satuan kerja hingga Biro Perlengkapan, dengan kemampuan membandingkan data eksisting, usulan kebutuhan, dan standar yang berlaku sesuai ketentuan Peraturan Menteri Keuangan.

## Glossary

- **BMN**: Barang Milik Negara - aset yang dimiliki oleh pemerintah
- **Satker**: Satuan Kerja - unit organisasi terkecil yang mengelola BMN
- **Kejati**: Kejaksaan Tinggi - tingkat wilayah yang membawahi beberapa satker
- **Biro_Perlengkapan**: Unit pusat yang mengelola dan menganalisis kebutuhan BMN seluruh Kejaksaan
- **Kodefikasi_BMN**: Sistem penomoran BMN sesuai Peraturan Menteri Keuangan
- **SBSK**: Standar Barang dan Standar Kebutuhan - standar spesifikasi dan jumlah BMN
- **Operator_Satker**: Pengguna yang menginput kebutuhan BMN di tingkat satker
- **Validator_Wilayah**: Pengguna yang memvalidasi kebutuhan BMN di tingkat Kejati
- **Validator_Pusat**: Pengguna yang memvalidasi dan menganalisis kebutuhan BMN di Biro Perlengkapan
- **Data_Eksisting**: Data BMN yang saat ini dimiliki oleh satker
- **Data_Usulan**: Data kebutuhan BMN yang diusulkan oleh satker
- **Gap_Analysis**: Analisis selisih antara kebutuhan dengan kondisi eksisting
- **Workflow_Engine**: Mesin yang mengelola alur persetujuan berjenjang
- **MySIMKARI**: Sistem informasi kepegawaian Kejaksaan untuk data pegawai
- **SIMAN**: Sistem informasi manajemen aset negara untuk data BMN eksisting

## Requirements

### Requirement 1: Pengumpulan Data Kebutuhan BMN

**User Story:** As an Operator_Satker, I want to submit BMN needs data for my work unit, so that the needs can be processed and analyzed by higher levels.

#### Acceptance Criteria

1. WHEN an Operator_Satker accesses the needs submission form, THE System SHALL display a form with BMN code selection based on Kodefikasi_BMN from Peraturan Menteri Keuangan
2. WHEN an Operator_Satker selects a BMN code, THE System SHALL auto-populate the BMN name, category, and unit of measurement
3. WHEN an Operator_Satker submits a needs request, THE System SHALL validate that all required fields are filled including BMN code, quantity, justification, and priority level
4. WHEN a needs request is submitted, THE System SHALL automatically fetch existing BMN data from SIMAN integration for comparison
5. WHEN a needs request is submitted, THE System SHALL automatically fetch employee count from MySIMKARI integration for ratio calculation
6. IF a needs request contains invalid BMN code, THEN THE System SHALL reject the submission with a descriptive error message
7. WHEN a needs request is successfully submitted, THE System SHALL assign a unique tracking number and set status to "draft"
8. THE System SHALL allow Operator_Satker to attach supporting documents (PDF, images) up to 10MB per file

### Requirement 2: Alur Persetujuan Berjenjang

**User Story:** As a system administrator, I want hierarchical approval workflow, so that needs requests are properly validated at each organizational level.

#### Acceptance Criteria

1. WHEN a needs request from Satker under Kejati is submitted, THE Workflow_Engine SHALL route it to Validator_Wilayah at the corresponding Kejati
2. WHEN a needs request from Biro/Pusat at Kejaksaan Agung (Bidang Pembinaan) is submitted, THE Workflow_Engine SHALL route it directly to Biro_Perlengkapan
3. WHEN a needs request from Sekretariat Bidang (non-Pembinaan) at Kejaksaan Agung is submitted, THE Workflow_Engine SHALL route it directly to Biro_Perlengkapan
4. WHEN a Validator_Wilayah approves a request, THE Workflow_Engine SHALL forward it to Validator_Pusat at Biro_Perlengkapan
5. WHEN a Validator_Wilayah rejects a request, THE System SHALL return it to Operator_Satker with rejection reason
6. WHEN a Validator_Pusat approves a request, THE System SHALL mark it as "approved" and include it in analysis pool
7. IF a request is pending for more than 7 days, THEN THE System SHALL send reminder notifications to the responsible validator
8. THE System SHALL maintain complete audit trail of all approval actions with timestamp and user information

### Requirement 3: Standar Spesifikasi dan Standar Jumlah (SBSK)

**User Story:** As a Validator_Pusat, I want to configure BMN standards, so that needs analysis can be performed against established benchmarks.

#### Acceptance Criteria

1. WHEN a Validator_Pusat accesses SBSK configuration, THE System SHALL display a management interface for standards
2. THE System SHALL allow configuration of standard specifications per BMN category including minimum specs, recommended specs, and maximum specs
3. THE System SHALL allow configuration of standard quantity ratios per BMN type (e.g., 1 computer per 1 employee, 1 vehicle per 10 employees)
4. WHEN SBSK is updated, THE System SHALL version the changes and maintain history
5. THE System SHALL support different SBSK profiles for different organizational levels (Satker, Kejati, Kejaksaan Agung)
6. WHEN a BMN code is selected in needs form, THE System SHALL display applicable SBSK information as reference
7. IF no SBSK is configured for a BMN type, THEN THE System SHALL flag it for manual review

### Requirement 4: Analisis Kebutuhan BMN

**User Story:** As a Validator_Pusat, I want to analyze BMN needs against standards and existing data, so that I can make informed decisions on procurement priorities.

#### Acceptance Criteria

1. WHEN Validator_Pusat initiates analysis, THE System SHALL compare submitted needs against Data_Eksisting from SIMAN
2. THE System SHALL calculate gap between current BMN count and requested quantity per satker
3. THE System SHALL compare requested specifications against SBSK minimum and recommended specs
4. THE System SHALL calculate BMN-to-employee ratio using MySIMKARI data and compare against SBSK ratio standards
5. WHEN analysis is complete, THE System SHALL generate a priority score based on gap severity, SBSK compliance, and justification strength
6. THE System SHALL categorize analysis results into: "Urgent", "Normal", "Low Priority", and "Over-standard"
7. IF requested quantity exceeds SBSK maximum, THEN THE System SHALL flag as "Over-standard" requiring additional justification
8. THE System SHALL support batch analysis for multiple satker requests simultaneously

### Requirement 5: Pelaporan dan Visualisasi

**User Story:** As a Validator_Pusat, I want comprehensive reports and visualizations, so that I can present analysis results to leadership.

#### Acceptance Criteria

1. WHEN Validator_Pusat accesses reporting module, THE System SHALL display dashboard with key metrics
2. THE System SHALL generate summary report showing total needs by BMN category, region, and priority
3. THE System SHALL generate gap analysis report comparing needs vs existing vs standards
4. THE System SHALL provide geographic visualization showing needs distribution across Kejati regions
5. THE System SHALL generate trend analysis comparing current period needs with historical data
6. WHEN a report is generated, THE System SHALL allow export to PDF and Excel formats
7. THE System SHALL provide drill-down capability from summary to detailed satker-level data
8. THE System SHALL generate compliance report showing SBSK adherence percentage per region

### Requirement 6: Integrasi Data Eksternal

**User Story:** As a system architect, I want seamless integration with external systems, so that data is accurate and up-to-date.

#### Acceptance Criteria

1. THE System SHALL integrate with MySIMKARI API to fetch employee data per satker
2. THE System SHALL integrate with SIMAN API to fetch existing BMN inventory data
3. WHEN external API is unavailable, THE System SHALL use cached data with clear indication of data freshness
4. THE System SHALL synchronize BMN master data (Kodefikasi_BMN) from authoritative source
5. WHEN data discrepancy is detected between systems, THE System SHALL log the discrepancy for reconciliation
6. THE System SHALL support manual data entry as fallback when integration is unavailable
7. THE System SHALL refresh integration data at configurable intervals (default: daily)

### Requirement 7: Manajemen Pengguna dan Otorisasi

**User Story:** As a system administrator, I want role-based access control, so that users can only access features appropriate to their role.

#### Acceptance Criteria

1. THE System SHALL authenticate users via Authenc IAM integration
2. THE System SHALL support three primary roles: Operator_Satker, Validator_Wilayah, and Validator_Pusat
3. WHEN an Operator_Satker logs in, THE System SHALL restrict access to only their satker's data
4. WHEN a Validator_Wilayah logs in, THE System SHALL provide access to all satker data within their Kejati
5. WHEN a Validator_Pusat logs in, THE System SHALL provide access to all data nationwide
6. THE System SHALL integrate with Secreton for secure credential and API key management
7. THE System SHALL log all user actions for audit purposes
8. IF a user attempts unauthorized access, THEN THE System SHALL deny access and log the attempt

### Requirement 8: Notifikasi dan Pengingat

**User Story:** As a user, I want to receive notifications about pending actions, so that I can respond in a timely manner.

#### Acceptance Criteria

1. WHEN a new request requires validation, THE System SHALL notify the responsible validator
2. WHEN a request is approved or rejected, THE System SHALL notify the submitting Operator_Satker
3. WHEN a request approaches deadline, THE System SHALL send reminder notification
4. THE System SHALL support notification via in-app notification and email
5. WHEN analysis results are available, THE System SHALL notify relevant stakeholders
6. THE System SHALL allow users to configure notification preferences
7. THE System SHALL maintain notification history accessible to users

### Requirement 9: Periode Pengumpulan dan Deadline

**User Story:** As a Validator_Pusat, I want to manage collection periods, so that needs data is gathered in organized cycles.

#### Acceptance Criteria

1. THE System SHALL support configurable collection periods (e.g., annual, quarterly)
2. WHEN a collection period is active, THE System SHALL allow Operator_Satker to submit needs
3. WHEN a collection period ends, THE System SHALL prevent new submissions and lock existing data
4. THE System SHALL display countdown to deadline on user dashboard
5. WHEN deadline approaches (7 days), THE System SHALL send reminder to satker with incomplete submissions
6. THE System SHALL allow Validator_Pusat to extend deadline for specific satker with justification
7. THE System SHALL generate completion rate report showing submission status per satker

### Requirement 10: Riwayat dan Audit Trail

**User Story:** As an auditor, I want complete history of all transactions, so that I can verify compliance and trace decisions.

#### Acceptance Criteria

1. THE System SHALL record all create, update, and delete operations with timestamp and user ID
2. THE System SHALL maintain version history of needs requests showing all changes
3. WHEN a request status changes, THE System SHALL log the transition with reason
4. THE System SHALL provide audit log search and filter capabilities
5. THE System SHALL retain audit data for minimum 5 years
6. THE System SHALL generate audit report for specified date range
7. THE System SHALL prevent modification of audit records

