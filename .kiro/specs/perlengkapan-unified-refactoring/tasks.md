# Implementation Plan: Unified Perlengkapan System

## Overview

This implementation plan consolidates the perlengkapan system from a multi-crate architecture into a single unified backend service (`layanan-perlengkapan`) with clear internal module boundaries. The system manages Barang Milik Negara (BMN) for Kejaksaan RI, providing production-ready functionality from day one.

**Key Objectives:**
- Consolidate existing crates (api, dokumen, notifikasi, bantuan) into a single unified crate
- Implement 11 business modules with consistent patterns
- Build comprehensive workflow engine with approval chains
- Integrate with Authenc, Secreton, and Integrasi services
- Achieve sub-200ms API response times with Redis caching
- Implement 14 property-based tests for correctness guarantees
- Deploy production-ready service with monitoring and observability

**Technology Stack:** Rust Edition 2024, Axum 0.8.x, PostgreSQL 15+, Redis, Tonic/Prost gRPC, Leptos 0.8.x frontend

**Timeline:** 12 weeks (7 phases)

## Tasks

### Phase 1: Foundation & Infrastructure (Week 1-2)

- [ ] 1. Set up unified crate structure and workspace integration
  - [ ] 1.1 Create new unified crate structure in layanan/perlengkapan
    - Remove old crate entries from root Cargo.toml workspace members
    - Create src/ directory with main.rs, lib.rs, config.rs
    - Create infrastructure/, models/, and module directories
    - Create migrations/ directory for database schemas
    - Create tests/ directory with common/ subdirectory
    - Update layanan/perlengkapan/Cargo.toml to use workspace dependencies only
    - _Requirements: 1.1, 1.2, 1.3, 25.1, 25.2_

  - [ ] 1.2 Implement configuration management system
    - Create Config struct with nested configuration sections (ServerConfig, DatabaseConfig, RedisConfig, AuthencConfig, SecretonConfig, IntegrasiConfig, CorsConfig, RateLimitConfig, FeatureFlags)
    - Implement configuration loading from environment variables using dotenvy for local development
    - Implement configuration loading from Secreton vault for production using Kubernetes auth backend
    - Implement configuration validation at startup with descriptive errors
    - Add configuration documentation in README.md
    - _Requirements: 2.1, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.8_

  - [ ]* 1.3 Write property test for configuration round-trip
    - **Property 3: Configuration Loading Preserves Values**
    - **Validates: Requirements 1.7, 7.1, 7.2, 7.4**
    - Generate arbitrary valid Config structs
    - Convert to environment variables, load back, verify equality
    - Test all configuration sections independently
    - _Requirements: 1.7, 7.1, 7.2, 7.4_

- [ ] 2. Implement infrastructure layer components
  - [ ] 2.1 Implement database connection pooling
    - Create DatabasePool struct wrapping deadpool-postgres
    - Implement connection pool initialization with configuration
    - Implement health check method for readiness probes
    - Implement connection timeout and retry logic
    - Add connection pool metrics (active, idle, waiting)
    - _Requirements: 3.1, 3.2, 3.7, 3.9, 16.3_

  - [ ] 2.2 Implement database migration system
    - Set up refinery for migration management
    - Create V001__initial_schema.sql with core tables (users, satkers, categories)
    - Implement migration runner function
    - Add migration logging and error handling
    - Document migration execution strategy
    - _Requirements: 3.3, 3.4_

  - [ ]* 2.3 Write property test for database transaction atomicity
    - **Property 10: Database Transaction Atomicity**
    - **Validates: Requirements 3.6**
    - Generate sequences of database operations with simulated failures
    - Verify database state unchanged after transaction rollback
    - Test with various failure points in transaction
    - _Requirements: 3.6_

  - [ ] 2.4 Implement gRPC client management
    - Create GrpcClients struct with Authenc, Secreton, Integrasi clients
    - Implement AuthencClient with validate_token, check_permission, get_user_info methods
    - Implement SecretonClient with Kubernetes auth backend support
    - Implement IntegrasiClient for MySIMKARI and SIMAN sync
    - Configure mTLS for secure gRPC communication
    - Implement connection pooling and health checks for gRPC clients
    - Add retry logic with exponential backoff
    - Implement circuit breaker pattern for fault tolerance
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.9, 4.10_

  - [ ] 2.5 Implement Redis cache client
    - Create RedisClient wrapper with connection management
    - Implement get, set, delete, exists methods
    - Implement TTL management for cache entries
    - Add cache metrics (hits, misses, evictions)
    - Implement cache key namespacing strategy
    - _Requirements: 16.2_

  - [ ]* 2.6 Write property test for cache consistency
    - **Property 11: Redis Cache Consistency**
    - **Validates: Requirements 16.2**
    - Generate arbitrary cacheable data
    - Verify cache hit returns same data as cache miss (database query)
    - Test cache invalidation correctness
    - _Requirements: 16.2_

- [ ] 3. Implement error handling and middleware
  - [ ] 3.1 Create comprehensive error type hierarchy
    - Define AppError enum with all error variants (Database, Grpc, Validation, Unauthorized, Forbidden, NotFound, Conflict, Internal, etc.)
    - Implement From conversions for common error types
    - Implement IntoResponse for AppError with HTTP status code mapping
    - Add error logging with appropriate levels
    - Implement error context pattern with anyhow
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.8_

  - [ ] 3.2 Implement authentication middleware
    - Create auth_middleware function to extract JWT from Authorization header
    - Validate JWT via Authenc gRPC service
    - Inject UserContext into request extensions
    - Return 401 for invalid/expired tokens
    - Log authentication failures with context
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.8, 6.10_

  - [ ]* 3.3 Write property test for JWT token extraction
    - **Property 4: JWT Token Extraction**
    - **Validates: Requirements 6.2, 6.5**
    - Generate arbitrary valid JWT tokens
    - Test extraction from various Authorization header formats
    - Verify extracted token matches original
    - _Requirements: 6.2, 6.5_

  - [ ]* 3.4 Write property test for RBAC permission consistency
    - **Property 5: RBAC Permission Consistency**
    - **Validates: Requirements 6.6, 6.9**
    - Generate arbitrary user/role/permission combinations
    - Verify permission checks are deterministic
    - Test same user/permission returns same result
    - _Requirements: 6.6, 6.9_

  - [ ]* 3.5 Write property test for authentication failure logging
    - **Property 6: Authentication Failure Logging**
    - **Validates: Requirements 6.10, 8.6, 8.7**
    - Generate arbitrary authentication failures
    - Verify log entries created with context (user_id, reason, timestamp)
    - Test log sanitization of sensitive data
    - _Requirements: 6.10, 8.6, 8.7_

  - [ ] 3.6 Implement CORS middleware
    - Configure allowed origins from configuration
    - Support preflight OPTIONS requests
    - Add secure headers (X-Frame-Options, X-Content-Type-Options, etc.)
    - _Requirements: 5.6_

  - [ ] 3.7 Implement rate limiting middleware
    - Create Redis-backed rate limiter
    - Implement per-user and per-IP rate limiting
    - Configure rate limits from configuration
    - Return 429 Too Many Requests when limit exceeded
    - _Requirements: 5.8_

  - [ ] 3.8 Implement request logging middleware
    - Log requests with correlation IDs
    - Log response status and duration
    - Sanitize sensitive data from logs (passwords, tokens)
    - Add performance metrics
    - _Requirements: 5.10, 8.5, 8.6, 8.7, 8.9_

- [ ] 4. Create shared domain models
  - [ ] 4.1 Implement core domain models
    - Create Asset model with AssetCondition and AssetStatus enums
    - Create WorkflowInstance model with WorkflowStatus enum
    - Create WorkflowStep model with ApprovalAction enum
    - Create Document model with DocumentClassification enum
    - Create Notification model with NotificationType and NotificationPriority enums
    - Create Ticket model with TicketCategory, TicketPriority, TicketStatus enums
    - Add serde Serialize/Deserialize derives
    - Add validation derives where appropriate
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [ ]* 4.2 Write property test for JSON serialization round-trip
    - **Property 1: JSON Serialization Round-Trip**
    - **Validates: Requirements 21.1, 21.2, 21.3, 21.4**
    - Generate arbitrary valid domain models (Asset, WorkflowInstance, Document, Notification, Ticket)
    - Serialize to JSON then deserialize
    - Verify deserialized object equals original
    - Test all domain models independently
    - _Requirements: 21.1, 21.2, 21.3, 21.4_

  - [ ] 4.3 Implement common types and utilities
    - Create Pagination struct for list endpoints
    - Create PaginationParams for request parsing
    - Create common filter types
    - Create UserContext struct for authenticated requests
    - _Requirements: 16.4_

  - [ ]* 4.4 Write property test for pagination completeness
    - **Property 12: Pagination Completeness**
    - **Validates: Requirements 16.4**
    - Generate arbitrary result sets with various sizes
    - Test pagination with different page sizes
    - Verify combining all pages produces complete result set
    - _Requirements: 16.4_

- [ ] 5. Set up application entry point and router
  - [ ] 5.1 Implement main.rs application initialization
    - Load configuration from environment and Secreton
    - Set up database connection pool
    - Create gRPC clients (Authenc, Secreton, Integrasi)
    - Initialize Redis cache
    - Run database migrations
    - Build Axum router with all module routes
    - Configure middleware stack
    - Start HTTP server with graceful shutdown
    - _Requirements: 1.3, 1.4, 1.5, 1.6, 5.2_

  - [ ] 5.2 Implement health check endpoints
    - Create /health endpoint for basic health check
    - Create /health/ready endpoint for readiness probe (check DB, Redis, gRPC connections)
    - Create /health/live endpoint for liveness probe
    - Create /metrics endpoint for Prometheus metrics
    - _Requirements: 19.3, 19.4_

  - [ ] 5.3 Implement graceful shutdown
    - Handle SIGTERM signal
    - Complete in-flight requests before shutdown
    - Close database connections gracefully
    - Close gRPC connections gracefully
    - _Requirements: 16.7, 19.4_

- [ ] 6. Checkpoint - Foundation complete
  - Ensure all tests pass, ask the user if questions arise.

### Phase 2: Core Business Modules (Week 3-4)

- [ ] 7. Implement Bank Aset (Asset Management) module
  - [ ] 7.1 Create Bank Aset module structure
    - Create src/bank_aset/ directory with mod.rs, handlers.rs, models.rs, repository.rs, services.rs, validation.rs
    - Define module public API in mod.rs
    - Create routes() function returning Router with asset endpoints
    - _Requirements: 2.1, 13.1_

  - [ ] 7.2 Implement Bank Aset database schema
    - Create V002__bank_aset.sql migration
    - Define assets table with all required columns
    - Define asset_categories table with hierarchical structure
    - Define asset_history table for audit trail
    - Create indexes for performance (satker_id, kategori_id, status, created_at)
    - _Requirements: 13.1_

  - [ ] 7.3 Implement Bank Aset models and validation
    - Create CreateAssetRequest, UpdateAssetRequest, AssetResponse DTOs
    - Add validation rules (length, format, business constraints)
    - Create AssetFilters for list queries
    - _Requirements: 13.1, 21.6, 21.7_

  - [ ]* 7.4 Write property test for input validation consistency
    - **Property 14: Input Validation Consistency**
    - **Validates: Requirements 21.5, 21.6, 21.7**
    - Generate arbitrary invalid asset requests
    - Verify validation fails with descriptive error
    - Test all validation rules independently
    - _Requirements: 21.5, 21.6, 21.7_

  - [ ] 7.5 Implement Bank Aset repository layer
    - Implement create, find_by_id, list, update, delete methods
    - Use prepared statements for all queries
    - Implement transaction support for multi-step operations
    - Add query optimization with proper indexes
    - _Requirements: 13.1, 3.5_

  - [ ] 7.6 Implement Bank Aset service layer
    - Implement business logic for asset operations
    - Add permission checks via Authenc gRPC
    - Implement cache management (invalidation on updates)
    - Add audit logging for all modifications
    - _Requirements: 13.1, 6.6, 13.10_

  - [ ] 7.7 Implement Bank Aset HTTP handlers
    - Implement list_assets handler with pagination and filtering
    - Implement create_asset handler with validation
    - Implement get_asset handler
    - Implement update_asset handler
    - Implement delete_asset handler
    - Implement get_asset_history handler
    - Return structured JSON responses with appropriate status codes
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 13.1_

  - [ ] 7.8 Implement Excel import/export for assets
    - Implement bulk_import_assets handler using calamine
    - Implement export_assets handler using rust_xlsxwriter
    - Add validation for imported data
    - _Requirements: 13.1, 21.8, 21.9_

  - [ ]* 7.9 Write property test for Excel data round-trip
    - **Property 2: Excel Data Round-Trip**
    - **Validates: Requirements 21.8, 21.9, 21.10**
    - Generate arbitrary asset lists
    - Export to Excel then import
    - Verify imported data equals original
    - _Requirements: 21.8, 21.9, 21.10_

- [ ] 8. Implement Kebutuhan BMN (Needs Planning) module
  - [ ] 8.1 Create Kebutuhan BMN module structure and schema
    - Create src/kebutuhan_bmn/ directory with standard module files
    - Create V003__kebutuhan_bmn.sql migration
    - Define kebutuhan_bmn table with planning fields
    - Create indexes for performance
    - _Requirements: 2.1, 13.2_

  - [ ] 8.2 Implement Kebutuhan BMN models, repository, and services
    - Create request/response DTOs with validation
    - Implement repository methods (create, find_by_id, list, update, delete)
    - Implement service layer with business logic and permission checks
    - Add workflow integration for approval submission
    - _Requirements: 13.2_

  - [ ] 8.3 Implement Kebutuhan BMN HTTP handlers
    - Implement CRUD handlers for needs planning
    - Implement submit_for_approval handler to trigger workflow
    - Add pagination and filtering support
    - _Requirements: 5.1, 5.2, 13.2_

- [ ] 9. Implement Pemakaian BMN (Usage Tracking) module
  - [ ] 9.1 Create Pemakaian BMN module structure and schema
    - Create src/pemakaian_bmn/ directory with standard module files
    - Create V004__pemakaian_bmn.sql migration
    - Define pemakaian_bmn table with usage tracking fields
    - Create indexes for performance
    - _Requirements: 2.1, 13.3_

  - [ ] 9.2 Implement Pemakaian BMN models, repository, and services
    - Create request/response DTOs with validation
    - Implement repository methods
    - Implement service layer with asset status updates
    - Add integration with Bank Aset for asset availability checks
    - _Requirements: 13.3_

  - [ ] 9.3 Implement Pemakaian BMN HTTP handlers
    - Implement CRUD handlers for usage tracking
    - Implement return_asset handler to mark asset as returned
    - Update asset status in Bank Aset when borrowed/returned
    - _Requirements: 5.1, 5.2, 13.3_

- [ ] 10. Implement Penghapusan BMN (Asset Disposal) module
  - [ ] 10.1 Create Penghapusan BMN module structure and schema
    - Create src/penghapusan_bmn/ directory with standard module files
    - Create V005__penghapusan_bmn.sql migration
    - Define penghapusan_bmn table with disposal request fields
    - Create indexes for performance
    - _Requirements: 2.1, 13.4_

  - [ ] 10.2 Implement Penghapusan BMN models, repository, and services
    - Create request/response DTOs with validation
    - Implement repository methods
    - Implement service layer with workflow integration
    - Add business rules for disposal eligibility
    - _Requirements: 13.4_

  - [ ] 10.3 Implement Penghapusan BMN HTTP handlers
    - Implement CRUD handlers for disposal requests
    - Implement submit_for_approval handler
    - Add document attachment support
    - _Requirements: 5.1, 5.2, 13.4_

- [ ] 11. Implement Pakaian Dinas (Uniform Management) module
  - [ ] 11.1 Create Pakaian Dinas module structure and schema
    - Create src/pakaian_dinas/ directory with standard module files
    - Create V006__pakaian_dinas.sql migration
    - Define pakaian_dinas table with uniform management fields
    - Create indexes for performance
    - _Requirements: 2.1, 13.5_

  - [ ] 11.2 Implement Pakaian Dinas models, repository, and services
    - Create request/response DTOs with validation
    - Implement repository methods
    - Implement service layer with distribution logic
    - Add inventory tracking for uniform stock
    - _Requirements: 13.5_

  - [ ] 11.3 Implement Pakaian Dinas HTTP handlers
    - Implement CRUD handlers for uniform management
    - Implement distribute_uniforms handler for bulk distribution
    - Add size and quantity tracking
    - _Requirements: 5.1, 5.2, 13.5_

- [ ] 12. Checkpoint - Core modules complete
  - Ensure all tests pass, ask the user if questions arise.

### Phase 3: Workflow Engine (Week 5)

- [ ] 13. Implement workflow engine core
  - [ ] 13.1 Create workflow module structure and schema
    - Create src/workflow/ directory with mod.rs, handlers.rs, models.rs, repository.rs, services.rs, engine.rs, rules.rs
    - Create V007__workflow.sql migration
    - Define workflow_definitions table with configurable steps
    - Define workflow_instances table for tracking execution
    - Define workflow_steps table for approval tracking
    - Create indexes for performance (status, approver_id, sla_deadline)
    - _Requirements: 2.1, 13.6, 14.1_

  - [ ] 13.2 Implement workflow definition models
    - Create WorkflowDefinition model with steps configuration
    - Create WorkflowStep model with approval requirements
    - Add support for sequential and parallel approval steps
    - Implement conditional routing rules
    - _Requirements: 14.1, 14.2, 14.8_

  - [ ] 13.3 Implement workflow instance models
    - Create WorkflowInstance model with current state
    - Create WorkflowStepExecution model for tracking approvals
    - Add status tracking (Pending, InProgress, Approved, Rejected, Cancelled)
    - _Requirements: 14.3_

  - [ ]* 13.4 Write property test for workflow state invariants
    - **Property 7: Workflow State Invariants**
    - **Validates: Requirements 14.1, 14.2, 14.3, 14.7**
    - Generate arbitrary workflow definitions and action sequences
    - Verify state transitions preserve invariants (valid current_step, status matches completion, monotonic timestamps)
    - Test with various approval actions
    - _Requirements: 14.1, 14.2, 14.3, 14.7_

  - [ ]* 13.5 Write property test for workflow conditional routing
    - **Property 8: Workflow Conditional Routing**
    - **Validates: Requirements 14.8**
    - Generate arbitrary workflow definitions with conditionals
    - Verify routing decisions are deterministic
    - Test same input always routes to same next step
    - _Requirements: 14.8_

  - [ ] 13.6 Implement workflow engine
    - Create WorkflowEngine for executing workflow logic
    - Implement workflow instance creation
    - Implement step activation and completion
    - Implement approval action processing (approve, reject, delegate, request changes)
    - Add support for parallel approval steps
    - Implement conditional routing based on business rules
    - _Requirements: 14.2, 14.3, 14.4, 14.8_

  - [ ] 13.7 Implement SLA monitoring and escalation
    - Add SLA deadline tracking for workflow steps
    - Implement background job for SLA monitoring
    - Implement escalation when SLA is breached
    - Send notifications for approaching deadlines
    - _Requirements: 14.5, 14.9_

  - [ ] 13.8 Implement workflow notification integration
    - Trigger notifications when approval is required
    - Send notifications on workflow completion
    - Send notifications on rejection or delegation
    - _Requirements: 14.6_

  - [ ]* 13.9 Write property test for workflow notification triggering
    - **Property 9: Workflow Notification Triggering**
    - **Validates: Requirements 14.6**
    - Generate arbitrary workflows with approval steps
    - Verify notification sent to approver when step becomes active
    - Test notification content includes required information
    - _Requirements: 14.6_

  - [ ] 13.10 Implement workflow repository layer
    - Implement methods for workflow definition CRUD
    - Implement methods for workflow instance management
    - Implement methods for workflow step tracking
    - Add transaction support for atomic state updates
    - _Requirements: 14.1, 14.3_

  - [ ] 13.11 Implement workflow service layer
    - Implement business logic for workflow operations
    - Add permission checks for workflow actions
    - Implement workflow history and audit trail
    - Add caching for workflow definitions
    - _Requirements: 14.7, 14.10_

  - [ ] 13.12 Implement workflow HTTP handlers
    - Implement list_workflow_definitions handler
    - Implement get_workflow_definition handler
    - Implement create_workflow_definition handler
    - Implement update_workflow_definition handler
    - Implement list_workflow_instances handler
    - Implement get_workflow_instance handler
    - Implement approve_workflow_step handler
    - Implement reject_workflow_step handler
    - Implement delegate_workflow_step handler
    - Implement get_pending_approvals handler
    - _Requirements: 5.1, 5.2, 13.6_

- [ ] 14. Checkpoint - Workflow engine complete
  - Ensure all tests pass, ask the user if questions arise.

### Phase 4: Supporting Modules (Week 6-7)

- [ ] 15. Implement Dokumen (Document Management) module
  - [ ] 15.1 Create Dokumen module structure and schema
    - Create src/dokumen/ directory with mod.rs, handlers.rs, models.rs, repository.rs, services.rs, storage.rs, templates.rs
    - Create src/dokumen/generators/ subdirectory with pdf.rs, excel.rs
    - Create V008__dokumen.sql migration
    - Define documents table with metadata and storage path
    - Create indexes for performance (entity_type, entity_id, uploaded_by)
    - _Requirements: 2.1, 9.1, 9.2_

  - [ ] 15.2 Implement document storage integration
    - Implement S3-compatible object storage client
    - Implement upload_to_storage method
    - Implement download_from_storage method
    - Implement delete_from_storage method
    - Add storage path generation strategy
    - _Requirements: 9.8_

  - [ ] 15.3 Implement document models and validation
    - Create UploadDocumentRequest, DocumentResponse DTOs
    - Add file type and size validation
    - Implement document classification (Public, Internal, Confidential, Secret)
    - Add metadata extraction
    - _Requirements: 9.10, 9.6_

  - [ ] 15.4 Implement document repository and services
    - Implement repository methods for document metadata
    - Implement service layer with storage integration
    - Add permission checks for document access
    - Implement document archival and retention policies
    - _Requirements: 9.7, 9.9_

  - [ ] 15.5 Implement document HTTP handlers
    - Implement list_documents handler with filtering
    - Implement upload_document handler with multipart support
    - Implement get_document_metadata handler
    - Implement download_document handler
    - Implement delete_document handler
    - Implement search_documents handler
    - _Requirements: 9.2, 5.1, 5.2_

  - [ ] 15.6 Implement document template system
    - Create document template models
    - Implement template variable substitution
    - Add template validation
    - _Requirements: 9.3_

  - [ ] 15.7 Implement PDF generation
    - Implement generate_pdf_from_template using printpdf
    - Add support for common document types (reports, certificates)
    - Implement PDF styling and formatting
    - _Requirements: 9.4_

  - [ ] 15.8 Implement Excel generation
    - Implement generate_excel_report using rust_xlsxwriter
    - Add support for data tables and charts
    - Implement Excel styling and formatting
    - _Requirements: 9.5_

- [ ] 16. Implement Notifikasi (Notification System) module
  - [ ] 16.1 Create Notifikasi module structure and schema
    - Create src/notifikasi/ directory with mod.rs, handlers.rs, models.rs, repository.rs, services.rs, templates.rs, websocket.rs
    - Create src/notifikasi/channels/ subdirectory with email.rs, sms.rs, whatsapp.rs, push.rs
    - Create V009__notifikasi.sql migration
    - Define notifications table with user_id, type, priority, read status
    - Define notification_preferences table for user settings
    - Create indexes for performance (user_id, read, created_at)
    - _Requirements: 2.1, 10.1, 10.2_

  - [ ] 16.2 Implement notification models
    - Create CreateNotificationRequest, NotificationResponse DTOs
    - Create NotificationPreferences model
    - Add notification types (WorkflowApproval, WorkflowCompleted, AssetUpdate, SystemAlert)
    - Add priority levels (Low, Medium, High, Critical)
    - _Requirements: 10.2_

  - [ ] 16.3 Implement notification channels
    - Implement email channel using lettre
    - Implement SMS channel integration
    - Implement WhatsApp channel integration
    - Implement push notification channel
    - Add channel selection based on user preferences
    - _Requirements: 10.3_

  - [ ] 16.4 Implement notification templates
    - Create notification template system with variable substitution
    - Implement templates for each notification type
    - Add multi-language support
    - _Requirements: 10.4_

  - [ ] 16.5 Implement notification queue and delivery
    - Implement asynchronous notification queue
    - Implement notification scheduling for delayed delivery
    - Add retry logic for failed deliveries
    - Implement rate limiting per user
    - Log all delivery attempts with status
    - _Requirements: 10.5, 10.6, 10.8, 10.10_

  - [ ] 16.6 Implement WebSocket support for real-time notifications
    - Implement WebSocket handler for notification streaming
    - Add connection management for WebSocket clients
    - Implement notification broadcasting to connected clients
    - Add heartbeat/ping-pong for connection health
    - _Requirements: 10.9_

  - [ ] 16.7 Implement notification repository and services
    - Implement repository methods for notification CRUD
    - Implement service layer with channel integration
    - Add user preference management
    - _Requirements: 10.2, 10.7_

  - [ ] 16.8 Implement notification HTTP handlers
    - Implement list_notifications handler with pagination
    - Implement get_unread_count handler
    - Implement mark_as_read handler
    - Implement mark_all_as_read handler
    - Implement get_preferences handler
    - Implement update_preferences handler
    - _Requirements: 10.2, 5.1, 5.2_

- [ ] 17. Implement Bantuan (Help/Ticket System) module
  - [ ] 17.1 Create Bantuan module structure and schema
    - Create src/bantuan/ directory with mod.rs, handlers.rs, models.rs, repository.rs, services.rs, workflow.rs, faq.rs
    - Create V010__bantuan.sql migration
    - Define tickets table with title, description, category, priority, status
    - Define ticket_comments table for conversation tracking
    - Define faq table for knowledge base
    - Define kb_articles table for help articles
    - Create indexes for performance (assigned_to, status, created_by)
    - _Requirements: 2.1, 11.1, 11.2_

  - [ ] 17.2 Implement ticket models and validation
    - Create CreateTicketRequest, UpdateTicketRequest, TicketResponse DTOs
    - Add validation rules for ticket fields
    - Create ticket categories (Technical, DataEntry, Workflow, General)
    - Create priority levels (Low, Medium, High, Critical)
    - Create status values (Open, InProgress, Resolved, Closed)
    - _Requirements: 11.2, 11.5_

  - [ ] 17.3 Implement ticket workflow
    - Implement ticket state machine (open → in_progress → resolved → closed)
    - Implement ticket assignment logic
    - Add automatic assignment based on category
    - Implement ticket escalation for high priority
    - _Requirements: 11.3, 11.4_

  - [ ] 17.4 Implement ticket repository and services
    - Implement repository methods for ticket CRUD
    - Implement comment management
    - Implement service layer with workflow integration
    - Add notification integration for status changes
    - Implement ticket analytics
    - _Requirements: 11.2, 11.8, 11.9_

  - [ ] 17.5 Implement ticket HTTP handlers
    - Implement list_tickets handler with filtering
    - Implement create_ticket handler with CAPTCHA validation
    - Implement get_ticket handler
    - Implement update_ticket handler
    - Implement close_ticket handler
    - Implement add_comment handler
    - _Requirements: 11.2, 11.10, 5.1, 5.2_

  - [ ] 17.6 Implement FAQ and knowledge base
    - Implement FAQ management (create, update, delete, search)
    - Implement knowledge base article management
    - Add category organization for articles
    - Implement search functionality with relevance ranking
    - _Requirements: 11.6, 11.7_

  - [ ] 17.7 Implement FAQ and knowledge base HTTP handlers
    - Implement list_faq handler
    - Implement get_faq handler
    - Implement search_faq handler
    - Implement list_kb_articles handler
    - Implement get_kb_article handler
    - _Requirements: 11.6, 11.7, 5.1, 5.2_

- [ ] 18. Implement Dashboard and Reporting module
  - [ ] 18.1 Create Dashboard module structure
    - Create src/dashboard/ directory with mod.rs, handlers.rs, models.rs, services.rs, aggregations.rs
    - _Requirements: 2.1, 13.7_

  - [ ] 18.2 Implement dashboard data aggregation
    - Implement asset statistics aggregation (total, by category, by status)
    - Implement workflow statistics (pending, approved, rejected)
    - Implement KPI calculations (asset utilization, approval turnaround time)
    - Add caching for dashboard data with appropriate TTL
    - _Requirements: 15.1, 15.7, 15.9_

  - [ ] 18.3 Implement dashboard HTTP handlers
    - Implement get_dashboard handler with aggregated statistics
    - Implement get_kpis handler
    - Implement get_chart_data handler for visualizations
    - Add role-based dashboard views
    - _Requirements: 15.1, 15.8, 5.1, 5.2_

  - [ ] 18.4 Implement report generation
    - Implement report template system
    - Implement PDF report generation
    - Implement Excel report generation
    - Add support for custom date ranges
    - Implement scheduled report generation using tokio-cron-scheduler
    - _Requirements: 15.3, 15.4, 15.5, 15.10_

  - [ ] 18.5 Implement report HTTP handlers
    - Implement list_reports handler
    - Implement generate_report handler
    - Implement download_report handler
    - _Requirements: 15.3, 5.1, 5.2_

  - [ ] 18.6 Implement real-time dashboard updates
    - Integrate WebSocket for real-time statistics
    - Implement dashboard event broadcasting
    - Add automatic refresh for stale data
    - _Requirements: 15.2_

- [ ] 19. Implement Admin and User Management module
  - [ ] 19.1 Create Admin module structure
    - Create src/admin/ directory with mod.rs, handlers.rs, models.rs, repository.rs, services.rs
    - _Requirements: 2.1, 13.8_

  - [ ] 19.2 Implement user management
    - Implement list_users handler with pagination and filtering
    - Implement create_user handler (delegates to Authenc)
    - Implement get_user handler
    - Implement update_user handler
    - Implement delete_user handler
    - Add permission checks for admin operations
    - _Requirements: 13.8_

  - [ ] 19.3 Implement role management
    - Implement list_roles handler
    - Implement create_role handler (delegates to Authenc)
    - Implement update_role handler
    - Add role-permission mapping
    - _Requirements: 13.8_

- [ ] 20. Checkpoint - Supporting modules complete
  - Ensure all tests pass, ask the user if questions arise.

### Phase 5: Integration & Testing (Week 8)

- [ ] 21. Implement external service integrations
  - [ ] 21.1 Integrate with Authenc service
    - Implement JWT validation in auth middleware
    - Implement permission checking in service layers
    - Add user info retrieval for admin functions
    - Test authentication flows end-to-end
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.6, 6.7_

  - [ ] 21.2 Integrate with Secreton service
    - Implement Kubernetes auth backend for Secreton
    - Fetch database credentials from Secreton at startup
    - Fetch Redis credentials from Secreton
    - Fetch S3 credentials from Secreton
    - Fetch SMTP credentials from Secreton
    - Test secret retrieval in different environments
    - _Requirements: 7.2, 7.9_

  - [ ] 21.3 Integrate with Integrasi service
    - Implement asset sync to MySIMKARI after create/update
    - Implement asset sync to SIMAN after create/update
    - Add sync status tracking
    - Implement retry logic for failed syncs
    - Test integration with mock services
    - _Requirements: 13.9_

  - [ ]* 21.4 Write property test for stateless request handling
    - **Property 13: Stateless Request Handling**
    - **Validates: Requirements 16.8**
    - Generate arbitrary API requests
    - Test same request on multiple backend instances
    - Verify responses are identical (no instance-specific state)
    - _Requirements: 16.8_

- [ ] 22. Implement comprehensive testing
  - [ ] 22.1 Write unit tests for all modules
    - Write unit tests for repository methods
    - Write unit tests for service layer business logic
    - Write unit tests for validation logic
    - Write unit tests for error handling
    - Achieve minimum 80% code coverage
    - _Requirements: 17.1, 17.5_

  - [ ] 22.2 Write integration tests for API endpoints
    - Write integration tests for all REST endpoints
    - Test authentication and authorization flows
    - Test error responses and status codes
    - Test pagination and filtering
    - _Requirements: 17.2, 17.8_

  - [ ] 22.3 Write database integration tests
    - Write tests for database operations with test fixtures
    - Test transaction rollback on errors
    - Test concurrent access scenarios
    - _Requirements: 17.3_

  - [ ] 22.4 Write gRPC client tests
    - Write tests for Authenc client with mocks
    - Write tests for Secreton client with mocks
    - Write tests for Integrasi client with mocks
    - Test retry logic and circuit breaker
    - _Requirements: 17.4_

  - [ ] 22.5 Write contract tests for API stability
    - Define API contracts for all endpoints
    - Write tests to verify contract compliance
    - Test backward compatibility
    - _Requirements: 17.8_

  - [ ] 22.6 Write load and performance tests
    - Write load tests using criterion for benchmarking
    - Test API response times under load
    - Test database connection pool under load
    - Test cache performance
    - Verify 95th percentile response time < 200ms
    - _Requirements: 16.1, 17.7_

  - [ ] 22.7 Generate test coverage reports
    - Configure test coverage reporting
    - Generate coverage reports for all modules
    - Verify coverage meets 80% threshold
    - _Requirements: 17.5, 17.10_

- [ ] 23. Checkpoint - Integration and testing complete
  - Ensure all tests pass, ask the user if questions arise.

### Phase 6: Deployment & Documentation (Week 9-10)

- [ ] 24. Create deployment artifacts
  - [ ] 24.1 Create production Docker image
    - Create multi-stage Dockerfile for optimized image size
    - Use Rust release profile for production builds
    - Include only necessary runtime dependencies
    - Add health check configuration
    - Test Docker image locally
    - _Requirements: 19.1, 19.2_

  - [ ] 24.2 Create Kubernetes manifests
    - Create Deployment manifest with resource limits
    - Create Service manifest for load balancing
    - Create ConfigMap for non-secret configuration
    - Create init container for database migrations
    - Add liveness and readiness probes
    - Configure horizontal pod autoscaling
    - _Requirements: 19.3, 19.5, 19.6, 19.8_

  - [ ] 24.3 Create Helm charts
    - Create Helm chart for perlengkapan service
    - Add values.yaml with configurable parameters
    - Add templates for all Kubernetes resources
    - Test Helm chart installation
    - _Requirements: 19.5_

  - [ ] 24.4 Configure secrets management
    - Configure Secreton integration for production
    - Set up Kubernetes service account for Secreton auth
    - Document secret paths and access patterns
    - Test secret retrieval in staging environment
    - _Requirements: 19.9_

  - [ ] 24.5 Set up CI/CD pipeline
    - Update .github/workflows/ci.yml for unified crate on self-hosted runners
    - Add build and test stages
    - Add Docker image build and push
    - Add Helm chart packaging
    - Configure automated deployment to staging
    - _Requirements: 25.9, 26.1_

  - [ ] 24.6 Update dependency management
    - Update .github/dependabot.yml to monitor layanan/perlengkapan
    - Remove old crate paths from dependabot configuration
    - _Requirements: 26.2_

- [ ] 25. Create comprehensive documentation
  - [ ] 25.1 Write README.md
    - Add project overview and architecture description
    - Add setup instructions for local development
    - Add configuration documentation
    - Add build and deployment instructions
    - Add troubleshooting guide
    - _Requirements: 20.1, 20.4_

  - [ ] 25.2 Write AGENTS.md
    - Document development guidelines and patterns
    - Document module structure and boundaries
    - Document testing strategies
    - Add code examples for common tasks
    - _Requirements: 20.2, 20.5, 20.10_

  - [ ] 25.3 Create API documentation
    - Document all REST API endpoints with request/response examples
    - Add authentication and authorization documentation
    - Add error code reference
    - Generate OpenAPI/Swagger specification
    - _Requirements: 20.3, 5.9_

  - [ ] 25.4 Create architecture diagrams
    - Create high-level system architecture diagram using Mermaid
    - Create module dependency diagram
    - Create database schema diagram
    - Create deployment architecture diagram
    - _Requirements: 20.6_

  - [ ] 25.5 Document database schema
    - Document all database tables with column descriptions
    - Document entity relationships
    - Document indexes and performance considerations
    - Add migration strategy documentation
    - _Requirements: 20.7_

  - [ ] 25.6 Create deployment documentation
    - Document deployment procedures for staging and production
    - Document rollback procedures
    - Document scaling procedures
    - Add environment-specific configuration guide
    - _Requirements: 20.8_

  - [ ] 25.7 Create operational runbooks
    - Document common operational tasks
    - Document troubleshooting procedures
    - Document monitoring and alerting setup
    - Add incident response procedures
    - _Requirements: 20.9_

- [ ] 26. Checkpoint - Deployment and documentation complete
  - Ensure all tests pass, ask the user if questions arise.

### Phase 7: Production Readiness (Week 11-12)

- [ ] 27. Implement monitoring and observability
  - [ ] 27.1 Set up Prometheus metrics
    - Implement request rate, latency, and error rate metrics
    - Add database connection pool metrics
    - Add gRPC client metrics
    - Add cache hit/miss metrics
    - Add business metrics (transactions, approvals, active users)
    - Expose /metrics endpoint
    - _Requirements: 16.1, 24.1_

  - [ ] 27.2 Set up distributed tracing
    - Implement OpenTelemetry tracing
    - Add correlation IDs to all logs and traces
    - Configure trace sampling
    - Integrate with tracing backend (Jaeger/Tempo)
    - _Requirements: 24.2, 24.3_

  - [ ] 27.3 Set up structured logging
    - Configure JSON logging for production
    - Add log levels (ERROR, WARN, INFO, DEBUG)
    - Implement log sanitization for sensitive data
    - Configure log aggregation (ELK/Loki)
    - _Requirements: 24.4, 24.5_

  - [ ] 27.4 Set up alerting
    - Configure alerts for critical errors
    - Configure alerts for high latency
    - Configure alerts for database connection issues
    - Configure alerts for gRPC client failures
    - Configure alerts for SLA breaches
    - _Requirements: 24.7_

  - [ ] 27.5 Create monitoring dashboards
    - Create Grafana dashboard for service health
    - Create dashboard for API performance
    - Create dashboard for business metrics
    - Create dashboard for infrastructure metrics
    - _Requirements: 24.8_

  - [ ] 27.6 Integrate with Sentry for error tracking
    - Configure Sentry integration
    - Add error context and breadcrumbs
    - Configure error sampling
    - Test error reporting
    - _Requirements: 24.10_

- [ ] 28. Perform security hardening
  - [ ] 28.1 Implement security best practices
    - Use Argon2 for password hashing
    - Use Ed25519 for cryptographic signatures
    - Implement input sanitization to prevent injection attacks
    - Implement CSRF protection for state-changing operations
    - Add security headers (X-Frame-Options, CSP, etc.)
    - _Requirements: 23.3, 23.4, 23.5, 23.6_

  - [ ] 28.2 Implement rate limiting
    - Configure rate limits per user and per IP
    - Implement distributed rate limiting with Redis
    - Add rate limit headers in responses
    - _Requirements: 23.7_

  - [ ] 28.3 Implement audit logging
    - Log all authentication events
    - Log all authorization failures
    - Log all data modifications
    - Log all security-relevant events
    - _Requirements: 23.8, 13.10_

  - [ ] 28.4 Implement data encryption
    - Encrypt sensitive data at rest in database
    - Use TLS for all network communication
    - Implement field-level encryption for sensitive fields
    - _Requirements: 23.9, 23.10_

  - [ ] 28.5 Conduct security audit
    - Run security scanning tools (cargo audit, trivy)
    - Review authentication and authorization flows
    - Review input validation and sanitization
    - Review error handling and information disclosure
    - Address all critical and high severity findings
    - _Requirements: 23.1, 23.2_

- [ ] 29. Optimize performance
  - [ ] 29.1 Optimize database queries
    - Review and optimize slow queries
    - Add missing indexes
    - Implement query result caching
    - Use prepared statements for all queries
    - _Requirements: 16.5, 16.6_

  - [ ] 29.2 Optimize caching strategy
    - Implement cache warming for frequently accessed data
    - Implement cache invalidation strategy
    - Configure appropriate TTLs for different data types
    - Monitor cache hit rates
    - _Requirements: 16.2, 16.7_

  - [ ] 29.3 Optimize connection pooling
    - Configure optimal database connection pool size
    - Configure optimal gRPC connection pool size
    - Monitor connection pool utilization
    - _Requirements: 16.3, 24.6_

  - [ ] 29.4 Implement response compression
    - Enable gzip compression for API responses
    - Configure compression levels
    - Test compression performance
    - _Requirements: 16.1_

  - [ ] 29.5 Implement background job processing
    - Set up tokio-cron-scheduler for scheduled tasks
    - Implement async notification delivery
    - Implement async report generation
    - Implement async external system sync
    - _Requirements: 16.7_

- [ ] 30. Prepare for production deployment
  - [ ] 30.1 Create disaster recovery plan
    - Document backup procedures for database
    - Document backup procedures for object storage
    - Document restore procedures
    - Test backup and restore procedures
    - _Requirements: 19.9_

  - [ ] 30.2 Conduct load testing
    - Run load tests with production-like traffic
    - Verify system handles 1000 concurrent users
    - Verify 99.9% uptime under load
    - Identify and address bottlenecks
    - _Requirements: 16.1, 16.8_

  - [ ] 30.3 Conduct failover testing
    - Test database failover
    - Test Redis failover
    - Test pod restart and recovery
    - Test graceful shutdown
    - _Requirements: 16.7, 19.4_

  - [ ] 30.4 Create deployment checklist
    - Document pre-deployment checks
    - Document deployment steps
    - Document post-deployment verification
    - Document rollback procedures
    - _Requirements: 20.8_

  - [ ] 30.5 Conduct staging deployment
    - Deploy to staging environment
    - Run smoke tests in staging
    - Verify all integrations working
    - Verify monitoring and alerting
    - Get stakeholder approval
    - _Requirements: 19.5_

  - [ ] 30.6 Conduct production deployment
    - Execute deployment checklist
    - Deploy to production environment
    - Run smoke tests in production
    - Monitor metrics and logs
    - Verify all functionality working
    - _Requirements: 19.10_

- [ ] 31. Code cleanup and optimization
  - [ ] 31.1 Remove duplicate code
    - Identify and remove code duplication across modules
    - Extract common patterns into shared utilities
    - Refactor complex functions into smaller units
    - _Requirements: 18.1, 18.10_

  - [ ] 31.2 Remove unused code and dependencies
    - Remove dead code and unused functions
    - Remove unused dependencies from Cargo.toml
    - Run cargo-udeps to identify unused dependencies
    - _Requirements: 18.2, 18.3_

  - [ ] 31.3 Apply code formatting and linting
    - Run cargo fmt --all to format code
    - Run cargo clippy --workspace and fix all warnings
    - Apply idiomatic Rust patterns
    - _Requirements: 18.4, 18.5, 18.6_

  - [ ] 31.4 Add documentation comments
    - Add rustdoc comments for all public APIs
    - Add module-level documentation
    - Add examples in documentation
    - Generate and review rustdoc output
    - _Requirements: 18.7_

  - [ ] 31.5 Optimize error handling
    - Replace all unwrap() calls with proper error handling
    - Add context to errors using anyhow
    - Ensure all errors are logged appropriately
    - _Requirements: 18.8_

  - [ ] 31.6 Optimize database operations
    - Minimize database round trips
    - Use batch operations where possible
    - Optimize query patterns
    - _Requirements: 18.9_

- [ ] 32. Update workspace configuration
  - [ ] 32.1 Update root Cargo.toml workspace members
    - Remove old crate entries: layanan/perlengkapan/crates/api, layanan/perlengkapan/crates/dokumen, layanan/perlengkapan/crates/notifikasi, layanan/perlengkapan/crates/bantuan
    - Ensure layanan/perlengkapan is registered as single member
    - Verify workspace compiles successfully
    - _Requirements: 25.1, 25.2, 25.3_

  - [ ] 32.2 Update CI/CD workflows
    - Update .github/workflows/ci.yml to use self-hosted runners
    - Update build commands for unified crate
    - Update test commands for unified crate
    - Update Docker build for unified crate
    - _Requirements: 26.1_

  - [ ] 32.3 Update release workflow
    - Update .github/workflows/release.yml for unified crate on self-hosted runners
    - Update artifact packaging
    - Update deployment automation
    - _Requirements: 26.1_

  - [ ] 32.4 Update GitLab CI configuration
    - Update .gitlab-ci.yml for unified crate structure
    - Update build and test stages
    - Update deployment stages
    - _Requirements: 26.3_

  - [ ] 32.5 Update tooling configurations
    - Update .gitleaks.toml if needed
    - Update .editorconfig if needed
    - Update any other tool configurations
    - _Requirements: 26.4_

  - [ ] 32.6 Clean up old crate directories
    - Remove layanan/perlengkapan/crates/api directory
    - Remove layanan/perlengkapan/crates/dokumen directory
    - Remove layanan/perlengkapan/crates/notifikasi directory
    - Remove layanan/perlengkapan/crates/bantuan directory
    - Update documentation references
    - _Requirements: 25.1, 25.2_

- [ ] 33. Update project documentation
  - [ ] 33.1 Update root AGENTS.md
    - Update perlengkapan service description
    - Update routing guide for unified structure
    - Remove references to old crate structure
    - _Requirements: 20.1, 20.2_

  - [ ] 33.2 Update root README.md
    - Update project structure documentation
    - Update build instructions
    - Update workspace member list
    - _Requirements: 20.1_

  - [ ] 33.3 Update CONTRIBUTING.md
    - Update development workflow for unified crate
    - Update testing guidelines
    - Update code review guidelines
    - _Requirements: 20.2_

  - [ ] 33.4 Update documentation links
    - Update all documentation links to reflect new structure
    - Update references in other services
    - Update references in frontend code
    - _Requirements: 20.1, 20.2, 20.3_

- [ ] 34. Final checkpoint - Production ready
  - Ensure all tests pass, ask the user if questions arise.

## Notes

### Property-Based Tests Summary

This implementation includes **14 property-based tests** that validate correctness properties across the system:

1. **Property 1: JSON Serialization Round-Trip** (Task 4.2) - Validates Requirements 21.1-21.4
2. **Property 2: Excel Data Round-Trip** (Task 7.9) - Validates Requirements 21.8-21.10
3. **Property 3: Configuration Loading Preserves Values** (Task 1.3) - Validates Requirements 1.7, 7.1, 7.2, 7.4
4. **Property 4: JWT Token Extraction** (Task 3.3) - Validates Requirements 6.2, 6.5
5. **Property 5: RBAC Permission Consistency** (Task 3.4) - Validates Requirements 6.6, 6.9
6. **Property 6: Authentication Failure Logging** (Task 3.5) - Validates Requirements 6.10, 8.6, 8.7
7. **Property 7: Workflow State Invariants** (Task 13.4) - Validates Requirements 14.1-14.3, 14.7
8. **Property 8: Workflow Conditional Routing** (Task 13.5) - Validates Requirements 14.8
9. **Property 9: Workflow Notification Triggering** (Task 13.9) - Validates Requirements 14.6
10. **Property 10: Database Transaction Atomicity** (Task 2.3) - Validates Requirements 3.6
11. **Property 11: Redis Cache Consistency** (Task 2.6) - Validates Requirements 16.2
12. **Property 12: Pagination Completeness** (Task 4.4) - Validates Requirements 16.4
13. **Property 13: Stateless Request Handling** (Task 21.4) - Validates Requirements 16.8
14. **Property 14: Input Validation Consistency** (Task 7.4) - Validates Requirements 21.5-21.7

### Optional Tasks

Tasks marked with `*` are optional and can be skipped for faster MVP delivery:
- All property-based test tasks (14 tasks)
- These tests provide strong correctness guarantees but are not required for basic functionality

### Implementation Strategy

**Incremental Development:**
- Each task builds on previous tasks
- Checkpoints ensure validation at key milestones
- Property tests catch errors early in development

**Module Independence:**
- Modules follow consistent patterns (handlers → services → repository)
- Shared infrastructure enables code reuse
- Clear boundaries prevent tight coupling

**Production Readiness:**
- Built for production from day one
- Comprehensive monitoring and observability
- Security hardening throughout
- Performance optimization at every layer

### Key Technical Decisions

1. **Single Crate Architecture**: Simplifies deployment and dependency management while maintaining clear module boundaries
2. **Rust Edition 2024**: Latest language features and performance improvements
3. **tokio-postgres + deadpool**: Async connection pooling without SQLx complexity
4. **refinery**: Simple, reliable database migrations
5. **Axum 0.8.x**: Modern, performant HTTP framework
6. **Tonic/Prost**: Type-safe gRPC communication
7. **Redis**: High-performance caching layer
8. **Property-Based Testing**: Strong correctness guarantees

### Success Criteria

**Technical:**
- ✅ Crate compiles without errors or warnings
- ✅ All tests passing with > 80% coverage
- ✅ API response times < 200ms (95th percentile)
- ✅ Zero critical security vulnerabilities
- ✅ Complete API and developer documentation

**Business:**
- ✅ All 11 business modules operational
- ✅ 99.9% uptime in production
- ✅ Supports 1000 concurrent users
- ✅ Clear code structure and documentation
- ✅ Positive user feedback

**Operational:**
- ✅ Automated CI/CD pipeline
- ✅ Comprehensive metrics and logging
- ✅ Timely alerts for issues
- ✅ Tested disaster recovery plan
- ✅ Operational runbooks

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 0,
      "tasks": ["1.1", "1.2"]
    },
    {
      "id": 1,
      "tasks": ["1.3", "2.1", "2.2", "2.4", "2.5", "3.1"]
    },
    {
      "id": 2,
      "tasks": ["2.3", "2.6", "3.2", "3.6", "3.7", "3.8", "4.1"]
    },
    {
      "id": 3,
      "tasks": ["3.3", "3.4", "3.5", "4.2", "4.3"]
    },
    {
      "id": 4,
      "tasks": ["4.4", "5.1", "5.2", "5.3"]
    },
    {
      "id": 5,
      "tasks": ["7.1", "8.1", "9.1", "10.1", "11.1"]
    },
    {
      "id": 6,
      "tasks": ["7.2", "7.3", "8.2", "9.2", "10.2", "11.2"]
    },
    {
      "id": 7,
      "tasks": ["7.4", "7.5", "8.3", "9.3", "10.3", "11.3"]
    },
    {
      "id": 8,
      "tasks": ["7.6", "7.7", "7.8"]
    },
    {
      "id": 9,
      "tasks": ["7.9"]
    },
    {
      "id": 10,
      "tasks": ["13.1", "13.2", "13.3"]
    },
    {
      "id": 11,
      "tasks": ["13.4", "13.5", "13.6", "13.7", "13.8"]
    },
    {
      "id": 12,
      "tasks": ["13.9", "13.10", "13.11"]
    },
    {
      "id": 13,
      "tasks": ["13.12"]
    },
    {
      "id": 14,
      "tasks": ["15.1", "15.2", "15.3", "16.1", "16.2", "17.1", "17.2", "18.1", "19.1"]
    },
    {
      "id": 15,
      "tasks": ["15.4", "15.6", "16.3", "16.4", "17.3", "18.2", "19.2"]
    },
    {
      "id": 16,
      "tasks": ["15.5", "15.7", "15.8", "16.5", "16.7", "17.4", "18.3", "19.3"]
    },
    {
      "id": 17,
      "tasks": ["16.6", "16.8", "17.5", "17.6", "18.4"]
    },
    {
      "id": 18,
      "tasks": ["17.7", "18.5", "18.6"]
    },
    {
      "id": 19,
      "tasks": ["21.1", "21.2", "21.3"]
    },
    {
      "id": 20,
      "tasks": ["21.4", "22.1", "22.2", "22.3", "22.4"]
    },
    {
      "id": 21,
      "tasks": ["22.5", "22.6", "22.7"]
    },
    {
      "id": 22,
      "tasks": ["24.1", "24.2", "24.3", "24.4"]
    },
    {
      "id": 23,
      "tasks": ["24.5", "24.6", "25.1", "25.2", "25.3"]
    },
    {
      "id": 24,
      "tasks": ["25.4", "25.5", "25.6", "25.7"]
    },
    {
      "id": 25,
      "tasks": ["27.1", "27.2", "27.3", "28.1", "29.1"]
    },
    {
      "id": 26,
      "tasks": ["27.4", "27.5", "27.6", "28.2", "28.3", "29.2", "29.3"]
    },
    {
      "id": 27,
      "tasks": ["28.4", "28.5", "29.4", "29.5"]
    },
    {
      "id": 28,
      "tasks": ["30.1", "30.2", "30.3", "30.4"]
    },
    {
      "id": 29,
      "tasks": ["30.5"]
    },
    {
      "id": 30,
      "tasks": ["30.6"]
    },
    {
      "id": 31,
      "tasks": ["31.1", "31.2", "31.3", "31.4", "31.5", "31.6"]
    },
    {
      "id": 32,
      "tasks": ["32.1", "32.2", "32.3", "32.4", "32.5"]
    },
    {
      "id": 33,
      "tasks": ["32.6", "33.1", "33.2", "33.3"]
    },
    {
      "id": 34,
      "tasks": ["33.4"]
    }
  ]
}
```
