# Implementation Plan: Sistem Analisis Kebutuhan BMN

## Overview

Implementasi sistem analisis kebutuhan BMN dengan pendekatan incremental, dimulai dari data models dan database schema, kemudian backend services, workflow engine, analysis engine, dan terakhir frontend components. Setiap tahap divalidasi dengan unit tests dan property-based tests.

## Tasks

- [ ] 1. Setup Database Schema dan Core Models
  - [ ] 1.1 Create database migration for kebutuhan_bmn schema
    - Create schema, enum types, and all tables (periode, sbsk, kebutuhan, analysis_result, workflow_history)
    - Add indexes for performance optimization
    - _Requirements: 1.1, 3.1, 4.1, 10.1_

  - [ ] 1.2 Implement core data models in Rust
    - Create models.rs with KebutuhanBmn, Sbsk, Periode, AnalysisResult, WorkflowHistory structs
    - Implement enums: KebutuhanStatus, Priority, OrganizationLevel, PeriodeStatus, AnalysisCategory
    - Add serde and sqlx derives
    - _Requirements: 1.1, 3.1, 4.1_

  - [ ]* 1.3 Write property test for model serialization round-trip
    - **Property: Model Serialization Round-Trip**
    - For any valid model instance, serializing to JSON and deserializing back should produce equivalent object
    - _Requirements: Data integrity_

- [ ] 2. Implement BMN Master Data and SBSK Configuration
  - [ ] 2.1 Create BMN master data service
    - Implement BmnMasterService with lookup by code
    - Add caching for frequently accessed codes
    - Implement auto-population of name, category, unit from code
    - _Requirements: 1.2_

  - [ ]* 2.2 Write property test for BMN code auto-population
    - **Property 1: BMN Code Auto-Population**
    - **Validates: Requirements 1.2**

  - [ ] 2.3 Implement SBSK CRUD service
    - Create SbskService with create, read, update, list operations
    - Implement versioning on update (create new version, preserve old)
    - Add filtering by kode_bmn and organization level
    - _Requirements: 3.2, 3.3, 3.5_

  - [ ]* 2.4 Write property test for SBSK versioning
    - **Property 10: SBSK Versioning**
    - **Validates: Requirements 3.4**

  - [ ] 2.5 Implement SBSK API handlers
    - GET /api/v1/sbsk - list with pagination and filters
    - POST /api/v1/sbsk - create new SBSK
    - PUT /api/v1/sbsk/{id} - update (creates new version)
    - GET /api/v1/sbsk/bmn/{kode} - get SBSK for specific BMN code
    - _Requirements: 3.1, 3.6_

- [ ] 3. Checkpoint - Ensure database and SBSK module tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 4. Implement Kebutuhan BMN Core Service
  - [ ] 4.1 Create validation service
    - Implement input validation using garde crate
    - Validate required fields: kode_bmn, jumlah_diusulkan, justifikasi, prioritas
    - Validate BMN code against master data
    - Validate file size for attachments (max 10MB)
    - _Requirements: 1.3, 1.6, 1.8_

  - [ ]* 4.2 Write property test for required field validation
    - **Property 2: Required Field Validation**
    - **Validates: Requirements 1.3, 1.6**

  - [ ]* 4.3 Write property test for file size validation
    - **Property 5: File Upload Size Validation**
    - **Validates: Requirements 1.8**

  - [ ] 4.4 Implement tracking number generator
    - Generate unique tracking number format: KB-{YEAR}-{SATKER}-{SEQ}
    - Ensure uniqueness with database constraint
    - _Requirements: 1.7_

  - [ ]* 4.5 Write property test for unique tracking number
    - **Property 4: Unique Tracking Number Generation**
    - **Validates: Requirements 1.7**

  - [ ] 4.6 Implement KebutuhanService CRUD operations
    - Create, read, update, delete operations
    - Filter by satker, kejati, status, periode
    - Pagination support
    - _Requirements: 1.1, 1.3_

  - [ ] 4.7 Implement Kebutuhan API handlers
    - POST /api/v1/kebutuhan - create new request
    - GET /api/v1/kebutuhan - list with filters
    - GET /api/v1/kebutuhan/{id} - get by ID
    - PUT /api/v1/kebutuhan/{id} - update (draft only)
    - DELETE /api/v1/kebutuhan/{id} - delete (draft only)
    - _Requirements: 1.1, 1.3_

- [ ] 5. Implement Integration Services
  - [ ] 5.1 Create MySIMKARI integration service
    - Implement fetch employee count by satker
    - Add caching with configurable TTL
    - Handle API unavailability with fallback to cache
    - _Requirements: 1.5, 6.1, 6.3_

  - [ ] 5.2 Create SIMAN integration service
    - Implement fetch existing BMN data by satker
    - Add caching with configurable TTL
    - Handle API unavailability with fallback to cache
    - _Requirements: 1.4, 6.2, 6.3_

  - [ ]* 5.3 Write property test for integration data fetch
    - **Property 3: Integration Data Fetch on Submission**
    - **Validates: Requirements 1.4, 1.5**

  - [ ] 5.4 Implement data discrepancy detection
    - Compare data between systems
    - Log discrepancies for reconciliation
    - _Requirements: 6.5_

- [ ] 6. Checkpoint - Ensure core services and integration tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Implement Workflow Engine
  - [ ] 7.1 Create workflow routing logic
    - Determine next approver based on organizational hierarchy
    - Route Satker under Kejati to Validator_Wilayah
    - Route Biro/Pusat Pembinaan directly to Biro_Perlengkapan
    - Route Sekretariat Bidang directly to Biro_Perlengkapan
    - _Requirements: 2.1, 2.2, 2.3_

  - [ ]* 7.2 Write property test for workflow routing
    - **Property 6: Workflow Routing Based on Organizational Unit**
    - **Validates: Requirements 2.1, 2.2, 2.3**

  - [ ] 7.3 Implement state transition logic
    - Submit: draft -> submitted/pending_wilayah/pending_pusat
    - Approve (wilayah): pending_wilayah -> pending_pusat
    - Approve (pusat): pending_pusat -> approved
    - Reject: any pending -> rejected
    - Request revision: any pending -> revision
    - _Requirements: 2.4, 2.5, 2.6_

  - [ ]* 7.4 Write property test for approval state transitions
    - **Property 7: Approval State Transitions**
    - **Validates: Requirements 2.4, 2.6**

  - [ ]* 7.5 Write property test for rejection state transition
    - **Property 8: Rejection State Transition**
    - **Validates: Requirements 2.5**

  - [ ] 7.6 Implement workflow history logging
    - Record all workflow actions with timestamp, actor, notes
    - Create immutable audit trail
    - _Requirements: 2.8, 10.1, 10.2, 10.3_

  - [ ]* 7.7 Write property test for audit logging
    - **Property 9: Comprehensive Audit Logging**
    - **Validates: Requirements 2.8, 10.1, 10.2, 10.3**

  - [ ] 7.8 Implement workflow API handlers
    - POST /api/v1/kebutuhan/{id}/submit
    - POST /api/v1/kebutuhan/{id}/approve
    - POST /api/v1/kebutuhan/{id}/reject
    - POST /api/v1/kebutuhan/{id}/revise
    - _Requirements: 2.4, 2.5, 2.6_

- [ ] 8. Implement Analysis Engine
  - [ ] 8.1 Implement gap analysis calculation
    - Calculate gap_quantity = requested - existing
    - Calculate gap_percentage = ((requested - existing) / existing) * 100
    - Handle edge cases (existing = 0)
    - _Requirements: 4.2_

  - [ ]* 8.2 Write property test for gap analysis calculation
    - **Property 11: Gap Analysis Calculation**
    - **Validates: Requirements 4.2**

  - [ ] 8.3 Implement ratio analysis calculation
    - Calculate BMN-to-employee ratio
    - Compare against SBSK ratio standard
    - _Requirements: 4.4_

  - [ ]* 8.4 Write property test for ratio analysis
    - **Property 12: Ratio Analysis Calculation**
    - **Validates: Requirements 4.4**

  - [ ] 8.5 Implement SBSK compliance check
    - Compare requested specs against SBSK min/recommended/max
    - Check quantity against SBSK limits
    - _Requirements: 4.3, 4.7_

  - [ ] 8.6 Implement priority score calculation
    - Gap severity weight: 40%
    - SBSK compliance weight: 30%
    - Priority level weight: 30%
    - Score range: 0-100
    - _Requirements: 4.5_

  - [ ]* 8.7 Write property test for priority score calculation
    - **Property 13: Priority Score Calculation**
    - **Validates: Requirements 4.5**

  - [ ] 8.8 Implement categorization logic
    - Urgent: score >= 80
    - Normal: score >= 50 and < 80
    - Low Priority: score < 50
    - Over-standard: exceeds SBSK maximum
    - _Requirements: 4.6, 4.7_

  - [ ]* 8.9 Write property test for over-standard categorization
    - **Property 14: Over-Standard Categorization**
    - **Validates: Requirements 4.7**

  - [ ] 8.10 Implement batch analysis
    - Process multiple requests simultaneously
    - Optimize database queries
    - _Requirements: 4.8_

  - [ ] 8.11 Implement analysis API handlers
    - POST /api/v1/analisis/run
    - GET /api/v1/analisis/results
    - _Requirements: 4.1, 4.8_

- [ ] 9. Checkpoint - Ensure workflow and analysis engine tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Implement Authorization and Security
  - [ ] 10.1 Implement role-based data filtering
    - Operator_Satker: filter by own satker_id
    - Validator_Wilayah: filter by kejati_id
    - Validator_Pusat: no filter (all data)
    - _Requirements: 7.3, 7.4, 7.5_

  - [ ]* 10.2 Write property test for role-based access
    - **Property 15: Role-Based Data Access Restriction**
    - **Validates: Requirements 7.3, 7.4, 7.5**

  - [ ] 10.3 Implement unauthorized access handling
    - Deny access to out-of-scope data
    - Log unauthorized access attempts
    - _Requirements: 7.8_

  - [ ]* 10.4 Write property test for unauthorized access denial
    - **Property 16: Unauthorized Access Denial**
    - **Validates: Requirements 7.8**

  - [ ] 10.5 Implement Authenc integration
    - JWT token validation middleware
    - Role extraction from token claims
    - _Requirements: 7.1, 7.2_

  - [ ] 10.6 Implement Secreton integration
    - Fetch API keys for external integrations
    - Secure credential management
    - _Requirements: 7.6_

  - [ ] 10.7 Implement audit record immutability
    - Prevent modification of workflow_history records
    - Database-level constraints
    - _Requirements: 10.7_

  - [ ]* 10.8 Write property test for audit immutability
    - **Property 19: Audit Record Immutability**
    - **Validates: Requirements 10.7**

- [ ] 11. Implement Period Management
  - [ ] 11.1 Implement Periode CRUD service
    - Create, read, update, list operations
    - Status transitions: draft -> active -> closed -> archived
    - _Requirements: 9.1_

  - [ ] 11.2 Implement period-based submission control
    - Allow submission only during active period
    - Block submission after period ends
    - _Requirements: 9.2, 9.3_

  - [ ]* 11.3 Write property test for period-based submission
    - **Property 18: Period-Based Submission Control**
    - **Validates: Requirements 9.2, 9.3**

  - [ ] 11.4 Implement deadline extension
    - Allow Validator_Pusat to extend deadline for specific satker
    - Require justification
    - _Requirements: 9.6_

  - [ ] 11.5 Implement completion rate calculation
    - Calculate submission status per satker
    - Generate completion report
    - _Requirements: 9.7_

  - [ ] 11.6 Implement Periode API handlers
    - GET /api/v1/periode
    - POST /api/v1/periode
    - PUT /api/v1/periode/{id}
    - POST /api/v1/periode/{id}/extend
    - _Requirements: 9.1, 9.6_

- [ ] 12. Implement Notification Service
  - [ ] 12.1 Create notification service
    - Support in-app and email notifications
    - Queue-based delivery with retry
    - _Requirements: 8.4_

  - [ ] 12.2 Implement event-triggered notifications
    - New request requires validation -> notify validator
    - Request approved/rejected -> notify operator
    - Analysis results available -> notify stakeholders
    - _Requirements: 8.1, 8.2, 8.5_

  - [ ]* 12.3 Write property test for event-triggered notifications
    - **Property 17: Event-Triggered Notifications**
    - **Validates: Requirements 8.1, 8.2, 8.5**

  - [ ] 12.4 Implement notification preferences
    - User-configurable notification settings
    - _Requirements: 8.6_

  - [ ] 12.5 Implement notification history
    - Store and retrieve notification history
    - _Requirements: 8.7_

- [ ] 13. Checkpoint - Ensure all backend tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 14. Implement Reporting Module
  - [ ] 14.1 Implement dashboard statistics
    - Total needs by category, region, priority
    - Pending approvals count
    - Completion rates
    - _Requirements: 5.1, 5.2_

  - [ ] 14.2 Implement gap analysis report
    - Compare needs vs existing vs standards
    - Aggregate by region and category
    - _Requirements: 5.3_

  - [ ] 14.3 Implement trend analysis
    - Compare current period with historical data
    - _Requirements: 5.5_

  - [ ] 14.4 Implement compliance report
    - SBSK adherence percentage per region
    - _Requirements: 5.8_

  - [ ] 14.5 Implement report export
    - PDF export using printpdf crate
    - Excel export using calamine/xlsxwriter crate
    - _Requirements: 5.6_

  - [ ] 14.6 Implement audit report
    - Filter by date range
    - Include all workflow actions
    - _Requirements: 10.4, 10.6_

  - [ ] 14.7 Implement report API handlers
    - GET /api/v1/dashboard/stats
    - GET /api/v1/dashboard/summary
    - GET /api/v1/analisis/report/{type}
    - _Requirements: 5.1, 5.2, 5.3, 5.5, 5.8_

- [ ] 15. Implement Frontend - Core Setup
  - [ ] 15.1 Setup frontend module structure
    - Create pages, components, api directories
    - Configure routing in lib.rs
    - _Requirements: Frontend architecture_

  - [ ] 15.2 Create API client service
    - HTTP client with auth token injection
    - Error handling and response parsing
    - _Requirements: Frontend-backend integration_

  - [ ] 15.3 Create shared types for frontend
    - Mirror backend models for TypeScript-like safety
    - Implement From/Into traits for API responses
    - _Requirements: Type safety_

- [ ] 16. Implement Frontend - Kebutuhan BMN Pages
  - [ ] 16.1 Create KebutuhanListPage component
    - Data table with pagination
    - Filters by status, periode, kategori
    - Role-based data display
    - _Requirements: 1.1, 7.3, 7.4, 7.5_

  - [ ] 16.2 Create KebutuhanFormPage component
    - BMN code selector with auto-population
    - File upload for supporting documents
    - Validation feedback
    - _Requirements: 1.1, 1.2, 1.3, 1.8_

  - [ ] 16.3 Create KebutuhanDetailPage component
    - Display all request details
    - Show comparison data (existing, employee count)
    - Workflow action buttons based on role
    - _Requirements: 1.4, 1.5, 2.4, 2.5, 2.6_

  - [ ] 16.4 Create ApprovalQueuePage component
    - List pending requests for validator
    - Bulk approval support
    - _Requirements: 2.1, 2.4, 2.5_

- [ ] 17. Implement Frontend - SBSK Management Pages
  - [ ] 17.1 Create SbskListPage component
    - List SBSK configurations
    - Filter by kategori, level organisasi
    - _Requirements: 3.1_

  - [ ] 17.2 Create SbskFormPage component
    - Create/edit SBSK configuration
    - Specification and ratio inputs
    - _Requirements: 3.2, 3.3_

- [ ] 18. Implement Frontend - Analysis Pages
  - [ ] 18.1 Create AnalisisDashboardPage component
    - Key metrics cards
    - Charts for distribution by category/region
    - _Requirements: 5.1, 5.2_

  - [ ] 18.2 Create AnalisisResultsPage component
    - Analysis results table
    - Priority score and category display
    - Drill-down to detail
    - _Requirements: 4.6, 5.7_

  - [ ] 18.3 Create ReportPage component
    - Report type selection
    - Date range filter
    - Export buttons (PDF, Excel)
    - _Requirements: 5.3, 5.5, 5.6, 5.8_

- [ ] 19. Implement Frontend - Period and Notification
  - [ ] 19.1 Create PeriodeListPage component
    - List collection periods
    - Status indicators
    - _Requirements: 9.1_

  - [ ] 19.2 Create PeriodeFormPage component
    - Create/edit period
    - Date range selection
    - _Requirements: 9.1_

  - [ ] 19.3 Create deadline countdown component
    - Display on dashboard
    - Visual indicator for approaching deadline
    - _Requirements: 9.4_

  - [ ] 19.4 Create notification center component
    - In-app notification display
    - Notification history
    - Preference settings
    - _Requirements: 8.4, 8.6, 8.7_

- [ ] 20. Implement Frontend - Dashboard Integration
  - [ ] 20.1 Update DashboardHome with real data
    - Connect to dashboard API
    - Display actual statistics
    - _Requirements: 5.1_

  - [ ] 20.2 Update navigation and routing
    - Add menu items for new pages
    - Configure protected routes
    - _Requirements: 7.1, 7.2_

  - [ ] 20.3 Integrate with Portal authentication
    - Use shared auth components
    - Handle session management
    - _Requirements: 7.1_

- [ ] 21. Final Checkpoint - Full Integration Testing
  - Ensure all tests pass, ask the user if questions arise.
  - Run end-to-end workflow tests
  - Verify all API endpoints work correctly
  - Test role-based access across all pages

## Notes

- Tasks marked with `*` are optional property-based tests that can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- Frontend tasks depend on backend API completion
