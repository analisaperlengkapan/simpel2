# Requirements Document

## Introduction

Dokumen ini mendefinisikan requirements untuk pembangunan sistem perlengkapan SIMPEL yang unified dan terintegrasi end-to-end. Sistem akan dibangun sebagai single crate backend (layanan-perlengkapan) dengan struktur modular internal yang jelas, terintegrasi dengan frontend Leptos WASM (antarmuka/perlengkapan).

Sistem perlengkapan mengelola Barang Milik Negara (BMN) untuk Kejaksaan RI, mencakup modul Bank Aset, Kebutuhan BMN, Pemakaian BMN, Penghapusan BMN, Pakaian Dinas, Workflow & Approval, Dashboard & Reporting, Admin & User Management, Dokumen Management, Notifikasi System, dan Help/Bantuan System.

## Glossary

- **Perlengkapan_System**: Sistem manajemen perlengkapan BMN Kejaksaan RI
- **Backend_Service**: Layanan backend Axum REST API yang dikonsolidasikan
- **Frontend_Microfrontend**: Aplikasi Leptos WASM di antarmuka/perlengkapan
- **API_Crate**: Crate utama layanan-perlengkapan-api yang menangani REST endpoints
- **Dokumen_Module**: Modul internal untuk manajemen dokumen (sebelumnya crate terpisah)
- **Notifikasi_Module**: Modul internal untuk sistem notifikasi (sebelumnya crate terpisah)
- **Bantuan_Module**: Modul internal untuk help/ticket system (sebelumnya crate terpisah)
- **BMN**: Barang Milik Negara
- **Authenc**: Service gRPC untuk autentikasi dan otorisasi
- **Secreton**: Service gRPC untuk manajemen secrets
- **Integrasi_Service**: Service untuk integrasi dengan sistem eksternal (MySIMKARI, SIMAN)
- **Database_Pool**: Connection pool PostgreSQL menggunakan deadpool-postgres
- **JWT_Token**: JSON Web Token untuk autentikasi user
- **gRPC_Client**: Client untuk komunikasi dengan Authenc dan Secreton
- **REST_API**: HTTP JSON API untuk komunikasi frontend-backend
- **Workspace_Dependencies**: Dependensi yang didefinisikan di root Cargo.toml
- **Module**: Unit organisasi kode internal dalam satu crate (bukan crate terpisah)

## Requirements

### Requirement 1: Unified Backend Crate Structure

**User Story:** As a developer, I want to build a unified backend crate from the start, so that I can avoid code duplication, simplify deployment, and ensure maintainability.

#### Acceptance Criteria

1. THE Backend_Service SHALL be implemented as a single crate named layanan-perlengkapan
2. THE Backend_Service SHALL organize functionality into internal modules (dokumen, notifikasi, bantuan, bank_aset, kebutuhan_bmn, pemakaian_bmn, penghapusan_bmn, pakaian_dinas, workflow, dashboard, admin)
3. THE Backend_Service SHALL maintain a single main.rs entry point that initializes all modules
4. THE Backend_Service SHALL use a single Axum router that includes routes from all modules
5. THE Backend_Service SHALL share a single Database_Pool across all modules
6. THE Backend_Service SHALL share gRPC_Client instances (Authenc, Secreton) across all modules
7. THE Backend_Service SHALL have unified configuration management across all modules
8. THE Backend_Service SHALL use Workspace_Dependencies exclusively (no version specifications in Cargo.toml)
9. THE Backend_Service SHALL compile without errors
10. THE Backend_Service SHALL have comprehensive unit tests for all modules

### Requirement 2: Module Structure Organization

**User Story:** As a developer, I want a clear modular structure within the unified crate, so that I can easily navigate and maintain different functional areas.

#### Acceptance Criteria

1. THE Backend_Service SHALL organize code into modules: bank_aset, kebutuhan_bmn, pemakaian_bmn, penghapusan_bmn, pakaian_dinas, workflow, dashboard, admin, dokumen, notifikasi, bantuan
2. WHEN a module is accessed, THE Backend_Service SHALL expose a clear public API through mod.rs
3. THE Backend_Service SHALL follow the pattern: src/{module}/handlers.rs, src/{module}/models.rs, src/{module}/repository.rs, src/{module}/services.rs
4. THE Backend_Service SHALL place shared infrastructure code in src/infrastructure/ (database, grpc_clients, middleware, errors, config)
5. THE Backend_Service SHALL place shared domain models in src/models/
6. THE Backend_Service SHALL maintain module independence (modules SHALL NOT directly import from each other's internal implementation)
7. WHEN modules need to communicate, THE Backend_Service SHALL use service layer abstractions or events
8. THE Backend_Service SHALL document module boundaries and responsibilities in module-level documentation

### Requirement 3: Database Layer Consolidation

**User Story:** As a developer, I want a unified database layer, so that I can manage connections efficiently and ensure consistent data access patterns.

#### Acceptance Criteria

1. THE Backend_Service SHALL use tokio-postgres with deadpool-postgres for connection pooling
2. THE Backend_Service SHALL create a single Database_Pool instance shared across all modules
3. THE Backend_Service SHALL use refinery for database migrations
4. THE Backend_Service SHALL consolidate all migration files into a single migrations/ directory
5. THE Backend_Service SHALL use prepared statements for all database queries
6. THE Backend_Service SHALL implement proper transaction handling for multi-step operations
7. THE Backend_Service SHALL validate database configuration at startup
8. WHEN database connection fails, THE Backend_Service SHALL return a descriptive error and exit gracefully
9. THE Backend_Service SHALL implement connection health checks for readiness probes
10. THE Backend_Service SHALL log database query performance metrics

### Requirement 4: gRPC Client Integration

**User Story:** As a developer, I want centralized gRPC client management, so that I can efficiently communicate with Authenc and Secreton services.

#### Acceptance Criteria

1. THE Backend_Service SHALL create gRPC_Client instances for Authenc and Secreton at startup
2. THE Backend_Service SHALL share gRPC_Client instances across all modules via application state
3. THE Backend_Service SHALL implement connection pooling for gRPC clients
4. THE Backend_Service SHALL configure mTLS for gRPC connections
5. THE Backend_Service SHALL implement retry logic with exponential backoff for gRPC calls
6. WHEN gRPC connection fails, THE Backend_Service SHALL log the error with context
7. THE Backend_Service SHALL validate JWT_Token via Authenc gRPC on every REST API request
8. THE Backend_Service SHALL fetch secrets from Secreton using Kubernetes auth backend
9. THE Backend_Service SHALL implement health checks for gRPC client connections
10. THE Backend_Service SHALL use connection timeouts to prevent hanging requests

### Requirement 5: REST API Endpoint Consolidation

**User Story:** As a frontend developer, I want all perlengkapan endpoints available from a single service, so that I can simplify API client configuration.

#### Acceptance Criteria

1. THE Backend_Service SHALL expose all REST endpoints under /api/pembinaan/perlengkapan/ prefix
2. THE Backend_Service SHALL consolidate routes from all modules into a single Axum router
3. THE Backend_Service SHALL implement consistent error response format across all endpoints
4. THE Backend_Service SHALL validate request payloads using the validator crate
5. THE Backend_Service SHALL return structured JSON responses with appropriate HTTP status codes
6. THE Backend_Service SHALL implement CORS configuration for frontend origins
7. THE Backend_Service SHALL apply authentication middleware to protected endpoints
8. THE Backend_Service SHALL implement rate limiting per user/IP
9. THE Backend_Service SHALL document all endpoints with OpenAPI/Swagger annotations
10. WHEN an endpoint is called, THE Backend_Service SHALL log request/response with correlation IDs

### Requirement 6: Authentication and Authorization

**User Story:** As a security engineer, I want consistent authentication and authorization across all endpoints, so that I can ensure zero-trust security.

#### Acceptance Criteria

1. THE Backend_Service SHALL validate JWT_Token on every protected REST API request
2. WHEN a request is received, THE Backend_Service SHALL extract JWT_Token from Authorization header
3. WHEN JWT_Token is present, THE Backend_Service SHALL validate it via Authenc gRPC
4. IF JWT_Token is invalid or expired, THEN THE Backend_Service SHALL return 401 Unauthorized
5. THE Backend_Service SHALL extract user_id and roles from validated JWT_Token
6. THE Backend_Service SHALL implement role-based access control (RBAC) for endpoints
7. WHEN user lacks required permissions, THE Backend_Service SHALL return 403 Forbidden
8. THE Backend_Service SHALL implement middleware for authentication checks
9. THE Backend_Service SHALL never trust client-side data for authorization decisions
10. THE Backend_Service SHALL log all authentication and authorization failures with context

### Requirement 7: Configuration Management

**User Story:** As a DevOps engineer, I want centralized configuration management, so that I can easily configure the service for different environments.

#### Acceptance Criteria

1. THE Backend_Service SHALL load configuration from environment variables using dotenvy for local development
2. THE Backend_Service SHALL fetch production secrets from Secreton via gRPC
3. THE Backend_Service SHALL use Kubernetes auth backend for Secreton authentication
4. THE Backend_Service SHALL validate all required configuration at startup
5. WHEN required configuration is missing, THE Backend_Service SHALL return a descriptive error and exit
6. THE Backend_Service SHALL support configuration for: database URL, Authenc endpoint, Secreton endpoint, Redis URL, CORS origins, rate limits
7. THE Backend_Service SHALL implement feature flags for optional functionality
8. THE Backend_Service SHALL log configuration values at startup (excluding secrets)
9. THE Backend_Service SHALL support hot-reload for non-critical configuration changes
10. THE Backend_Service SHALL document all configuration options in README.md

### Requirement 8: Error Handling and Logging

**User Story:** As a developer, I want consistent error handling and logging, so that I can debug issues efficiently.

#### Acceptance Criteria

1. THE Backend_Service SHALL use thiserror for error type definitions
2. THE Backend_Service SHALL implement a custom AppError enum covering all error cases
3. THE Backend_Service SHALL implement From conversions for common error types (database, gRPC, validation)
4. THE Backend_Service SHALL return structured error responses with error codes and messages
5. THE Backend_Service SHALL use tracing for structured logging
6. THE Backend_Service SHALL log errors with appropriate context (request_id, user_id, module)
7. THE Backend_Service SHALL implement log levels: ERROR for failures, WARN for recoverable issues, INFO for important events, DEBUG for detailed diagnostics
8. THE Backend_Service SHALL include stack traces in error logs for debugging
9. THE Backend_Service SHALL sanitize sensitive data (passwords, tokens) from logs
10. THE Backend_Service SHALL export logs in JSON format for production environments

### Requirement 9: Dokumen Module Integration

**User Story:** As a user, I want document management functionality integrated into the main service, so that I can manage documents seamlessly with other perlengkapan features.

#### Acceptance Criteria

1. THE Backend_Service SHALL integrate document management functionality as an internal Dokumen_Module
2. THE Dokumen_Module SHALL provide REST endpoints for: upload, download, list, delete, search documents
3. THE Dokumen_Module SHALL support document templates for workflow-generated documents
4. THE Dokumen_Module SHALL generate PDF documents from templates using printpdf
5. THE Dokumen_Module SHALL generate Excel documents using rust_xlsxwriter
6. THE Dokumen_Module SHALL implement document classification and metadata extraction
7. THE Dokumen_Module SHALL implement document archival and retention policies
8. THE Dokumen_Module SHALL store documents in object storage (S3-compatible)
9. THE Dokumen_Module SHALL implement access control for document operations based on user roles
10. THE Dokumen_Module SHALL validate file types and sizes before accepting uploads

### Requirement 10: Notifikasi Module Integration

**User Story:** As a user, I want notification functionality integrated into the main service, so that I can receive timely updates about perlengkapan activities.

#### Acceptance Criteria

1. THE Backend_Service SHALL integrate notification functionality as an internal Notifikasi_Module
2. THE Notifikasi_Module SHALL provide REST endpoints for: list notifications, mark as read, get unread count, update preferences
3. THE Notifikasi_Module SHALL support multiple notification channels: in-app, email, SMS, WhatsApp, push notifications
4. THE Notifikasi_Module SHALL implement notification templates with variable substitution
5. THE Notifikasi_Module SHALL implement notification queue for asynchronous delivery
6. THE Notifikasi_Module SHALL implement notification scheduling for delayed delivery
7. THE Notifikasi_Module SHALL implement user notification preferences (channel, frequency)
8. THE Notifikasi_Module SHALL implement notification rate limiting per user
9. THE Notifikasi_Module SHALL implement WebSocket support for real-time in-app notifications
10. THE Notifikasi_Module SHALL log all notification delivery attempts with status

### Requirement 11: Bantuan Module Integration

**User Story:** As a user, I want help/ticket system functionality integrated into the main service, so that I can get support for perlengkapan issues.

#### Acceptance Criteria

1. THE Backend_Service SHALL integrate help/ticket system functionality as an internal Bantuan_Module
2. THE Bantuan_Module SHALL provide REST endpoints for: create ticket, list tickets, get ticket detail, update ticket, close ticket
3. THE Bantuan_Module SHALL implement ticket workflow with statuses: open, in_progress, resolved, closed
4. THE Bantuan_Module SHALL implement ticket assignment to support staff
5. THE Bantuan_Module SHALL implement ticket priority levels: low, medium, high, critical
6. THE Bantuan_Module SHALL implement FAQ management with search functionality
7. THE Bantuan_Module SHALL implement knowledge base articles with categories
8. THE Bantuan_Module SHALL implement ticket analytics and reporting
9. THE Bantuan_Module SHALL send notifications when ticket status changes
10. THE Bantuan_Module SHALL implement CAPTCHA for anonymous ticket creation to prevent spam

### Requirement 12: Frontend-Backend Integration

**User Story:** As a frontend developer, I want seamless integration between the Leptos frontend and Axum backend, so that I can build responsive user interfaces.

#### Acceptance Criteria

1. THE Frontend_Microfrontend SHALL communicate with Backend_Service exclusively via REST_API
2. THE Frontend_Microfrontend SHALL include JWT_Token in Authorization header for all authenticated requests
3. THE Frontend_Microfrontend SHALL handle 401 Unauthorized responses by redirecting to login
4. THE Frontend_Microfrontend SHALL handle 403 Forbidden responses by showing permission error
5. THE Frontend_Microfrontend SHALL display user-friendly error messages from API error responses
6. THE Frontend_Microfrontend SHALL implement loading states during API calls
7. THE Frontend_Microfrontend SHALL implement optimistic updates for better UX
8. THE Frontend_Microfrontend SHALL use Leptos Resources for async data fetching
9. THE Frontend_Microfrontend SHALL implement proper error boundaries for component failures
10. THE Frontend_Microfrontend SHALL use WebSocket for real-time notifications

### Requirement 13: Business Module Completeness

**User Story:** As a product owner, I want all business modules fully implemented, so that users can perform all perlengkapan operations.

#### Acceptance Criteria

1. THE Backend_Service SHALL implement complete CRUD operations for Bank Aset (asset management)
2. THE Backend_Service SHALL implement complete CRUD operations for Kebutuhan BMN (needs planning)
3. THE Backend_Service SHALL implement complete CRUD operations for Pemakaian BMN (usage tracking)
4. THE Backend_Service SHALL implement complete CRUD operations for Penghapusan BMN (asset disposal)
5. THE Backend_Service SHALL implement complete CRUD operations for Pakaian Dinas (uniform management)
6. THE Backend_Service SHALL implement workflow engine with approval chains
7. THE Backend_Service SHALL implement dashboard with real-time statistics
8. THE Backend_Service SHALL implement admin functions for user and role management
9. THE Backend_Service SHALL implement integration with Integrasi_Service for MySIMKARI and SIMAN sync
10. THE Backend_Service SHALL implement audit logging for all data modifications

### Requirement 14: Workflow and Approval System

**User Story:** As a manager, I want a flexible workflow and approval system, so that I can control business processes.

#### Acceptance Criteria

1. THE Backend_Service SHALL implement workflow definitions with configurable steps
2. THE Backend_Service SHALL support sequential and parallel approval steps
3. THE Backend_Service SHALL implement workflow instance tracking with current state
4. THE Backend_Service SHALL implement approval actions: approve, reject, delegate, request changes
5. THE Backend_Service SHALL implement SLA monitoring for workflow steps
6. THE Backend_Service SHALL send notifications when approval is required
7. THE Backend_Service SHALL implement workflow history and audit trail
8. THE Backend_Service SHALL support conditional routing based on business rules
9. THE Backend_Service SHALL implement escalation when SLA is breached
10. THE Backend_Service SHALL allow workflow definition updates without code changes

### Requirement 15: Dashboard and Reporting

**User Story:** As a manager, I want comprehensive dashboards and reports, so that I can monitor perlengkapan operations.

#### Acceptance Criteria

1. THE Backend_Service SHALL provide dashboard endpoints with aggregated statistics
2. THE Backend_Service SHALL implement real-time dashboard updates via WebSocket
3. THE Backend_Service SHALL generate reports in PDF and Excel formats
4. THE Backend_Service SHALL implement report templates for common report types
5. THE Backend_Service SHALL support custom date ranges for reports
6. THE Backend_Service SHALL implement data visualization endpoints for charts and graphs
7. THE Backend_Service SHALL cache dashboard data with appropriate TTL
8. THE Backend_Service SHALL implement role-based dashboard views
9. THE Backend_Service SHALL track KPIs: asset utilization, approval turnaround time, pending requests
10. THE Backend_Service SHALL implement scheduled report generation and delivery

### Requirement 16: Performance and Scalability

**User Story:** As a system administrator, I want the service to be performant and scalable, so that it can handle production load.

#### Acceptance Criteria

1. THE Backend_Service SHALL respond to API requests within 200ms for 95th percentile
2. THE Backend_Service SHALL implement Redis caching for frequently accessed data
3. THE Backend_Service SHALL use database connection pooling with appropriate pool size
4. THE Backend_Service SHALL implement pagination for list endpoints
5. THE Backend_Service SHALL use database indexes for frequently queried columns
6. THE Backend_Service SHALL implement query optimization for complex reports
7. THE Backend_Service SHALL implement graceful shutdown to complete in-flight requests
8. THE Backend_Service SHALL implement horizontal scaling support (stateless design)
9. THE Backend_Service SHALL implement circuit breakers for external service calls
10. THE Backend_Service SHALL monitor and expose performance metrics via Prometheus

### Requirement 17: Testing and Quality Assurance

**User Story:** As a developer, I want comprehensive test coverage, so that I can ensure code quality and prevent regressions.

#### Acceptance Criteria

1. THE Backend_Service SHALL implement unit tests for all business logic functions
2. THE Backend_Service SHALL implement integration tests for API endpoints
3. THE Backend_Service SHALL implement database integration tests with test fixtures
4. THE Backend_Service SHALL implement gRPC client mock tests
5. THE Backend_Service SHALL achieve minimum 80% code coverage
6. THE Backend_Service SHALL implement property-based tests for data validation logic
7. THE Backend_Service SHALL implement load tests for performance validation
8. THE Backend_Service SHALL implement contract tests for API stability
9. THE Backend_Service SHALL run tests in CI/CD pipeline before deployment
10. THE Backend_Service SHALL generate test coverage reports

### Requirement 18: Code Cleanup and Optimization

**User Story:** As a developer, I want clean and optimized code, so that the codebase is maintainable and efficient.

#### Acceptance Criteria

1. THE Backend_Service SHALL remove all duplicate code across modules
2. THE Backend_Service SHALL remove unused dependencies from Cargo.toml
3. THE Backend_Service SHALL remove dead code and unused functions
4. THE Backend_Service SHALL apply consistent code formatting using rustfmt
5. THE Backend_Service SHALL pass all clippy lints without warnings
6. THE Backend_Service SHALL use idiomatic Rust patterns throughout
7. THE Backend_Service SHALL document all public APIs with rustdoc comments
8. THE Backend_Service SHALL implement proper error handling (no unwrap in production code)
9. THE Backend_Service SHALL optimize database queries to minimize round trips
10. THE Backend_Service SHALL refactor complex functions into smaller, testable units

### Requirement 19: Deployment and DevOps

**User Story:** As a DevOps engineer, I want streamlined deployment, so that I can deploy the service reliably.

#### Acceptance Criteria

1. THE Backend_Service SHALL build into a single Docker image
2. THE Backend_Service SHALL use multi-stage Docker build for optimized image size
3. THE Backend_Service SHALL implement health check endpoints for liveness and readiness probes
4. THE Backend_Service SHALL implement graceful shutdown on SIGTERM signal
5. THE Backend_Service SHALL support Kubernetes deployment with Helm charts
6. THE Backend_Service SHALL implement proper resource limits and requests
7. THE Backend_Service SHALL use immutable image tags with semantic versioning
8. THE Backend_Service SHALL implement database migration as init container
9. THE Backend_Service SHALL fetch secrets from Secreton at startup using Kubernetes auth
10. THE Backend_Service SHALL export metrics for Prometheus scraping

### Requirement 20: Documentation and Knowledge Transfer

**User Story:** As a new developer, I want comprehensive documentation, so that I can understand and contribute to the codebase.

#### Acceptance Criteria

1. THE Backend_Service SHALL include README.md with architecture overview and setup instructions
2. THE Backend_Service SHALL include AGENTS.md with development guidelines and patterns
3. THE Backend_Service SHALL document all REST API endpoints with request/response examples
4. THE Backend_Service SHALL document all configuration options with default values
5. THE Backend_Service SHALL document module boundaries and responsibilities
6. THE Backend_Service SHALL include architecture diagrams using Mermaid
7. THE Backend_Service SHALL document database schema with entity relationships
8. THE Backend_Service SHALL document deployment procedures for different environments
9. THE Backend_Service SHALL document troubleshooting guides for common issues
10. THE Backend_Service SHALL include code examples for common development tasks

### Requirement 21: Data Serialization and Validation

**User Story:** As a developer, I want robust data serialization and validation, so that I can ensure data integrity across the system.

#### Acceptance Criteria

1. THE Backend_Service SHALL parse JSON request payloads into strongly-typed Rust structs
2. THE Backend_Service SHALL serialize Rust structs into JSON response payloads
3. THE Backend_Service SHALL implement a Pretty_Printer for all data models to format them back to JSON
4. FOR ALL valid data models, parsing then printing then parsing SHALL produce an equivalent object (round-trip property)
5. WHEN invalid JSON is provided, THE Backend_Service SHALL return a descriptive validation error
6. THE Backend_Service SHALL validate all input data using the validator crate
7. THE Backend_Service SHALL implement custom validation rules for business constraints
8. THE Backend_Service SHALL parse Excel files for bulk import operations
9. THE Backend_Service SHALL serialize data to Excel format for export operations
10. FOR ALL valid Excel data, parsing then serializing then parsing SHALL produce equivalent data (round-trip property)

### Requirement 22: Frontend Data Handling

**User Story:** As a frontend developer, I want consistent data handling in the Leptos frontend, so that I can build reliable user interfaces.

#### Acceptance Criteria

1. THE Frontend_Microfrontend SHALL parse JSON API responses into strongly-typed Rust structs
2. THE Frontend_Microfrontend SHALL serialize form data into JSON for API requests
3. THE Frontend_Microfrontend SHALL validate form inputs before submission
4. THE Frontend_Microfrontend SHALL display validation errors inline with form fields
5. THE Frontend_Microfrontend SHALL implement client-side validation matching backend rules
6. THE Frontend_Microfrontend SHALL handle API error responses gracefully
7. THE Frontend_Microfrontend SHALL implement data caching for frequently accessed resources
8. THE Frontend_Microfrontend SHALL implement optimistic updates with rollback on error
9. THE Frontend_Microfrontend SHALL use Leptos signals for reactive state management
10. THE Frontend_Microfrontend SHALL implement proper memory cleanup for WASM resources

### Requirement 23: Security and Compliance

**User Story:** As a security officer, I want the system to meet security and compliance requirements, so that sensitive government data is protected.

#### Acceptance Criteria

1. THE Backend_Service SHALL implement zero-trust security model for all inter-service communication
2. THE Backend_Service SHALL validate JWT_Token on every protected endpoint
3. THE Backend_Service SHALL use Argon2 for password hashing
4. THE Backend_Service SHALL use Ed25519 for cryptographic signatures
5. THE Backend_Service SHALL sanitize all user inputs to prevent injection attacks
6. THE Backend_Service SHALL implement CSRF protection for state-changing operations
7. THE Backend_Service SHALL implement rate limiting to prevent abuse
8. THE Backend_Service SHALL log all security-relevant events (authentication, authorization, data access)
9. THE Backend_Service SHALL encrypt sensitive data at rest in the database
10. THE Backend_Service SHALL use TLS for all network communication

### Requirement 24: Monitoring and Observability

**User Story:** As a system administrator, I want comprehensive monitoring and observability, so that I can detect and resolve issues quickly.

#### Acceptance Criteria

1. THE Backend_Service SHALL export Prometheus metrics for request rate, latency, and error rate
2. THE Backend_Service SHALL implement distributed tracing with OpenTelemetry
3. THE Backend_Service SHALL include correlation IDs in all logs and traces
4. THE Backend_Service SHALL implement structured logging with JSON format
5. THE Backend_Service SHALL monitor database connection pool utilization
6. THE Backend_Service SHALL monitor gRPC client connection health
7. THE Backend_Service SHALL implement alerting for critical errors
8. THE Backend_Service SHALL track business metrics (transactions, approvals, active users)
9. THE Backend_Service SHALL implement performance profiling endpoints for debugging
10. THE Backend_Service SHALL integrate with Sentry for error tracking

### Requirement 25: Workspace Integration

**User Story:** As a developer, I want proper workspace integration, so that the perlengkapan service works seamlessly with other services in the monorepo.

#### Acceptance Criteria

1. THE Backend_Service SHALL be registered in root Cargo.toml workspace members as "layanan/perlengkapan"
2. THE Backend_Service SHALL remove old crate entries from workspace members (layanan/perlengkapan/crates/api, layanan/perlengkapan/crates/dokumen, layanan/perlengkapan/crates/notifikasi, layanan/perlengkapan/crates/bantuan)
3. THE Backend_Service SHALL use workspace dependencies for all external crates
4. THE Backend_Service SHALL compile successfully with `cargo build -p layanan-perlengkapan`
5. THE Backend_Service SHALL run tests successfully with `cargo test -p layanan-perlengkapan`
6. THE Backend_Service SHALL be included in workspace-level commands (cargo fmt --all, cargo clippy --workspace)
7. THE Backend_Service SHALL share the workspace Cargo.lock for dependency resolution
8. THE Backend_Service SHALL follow workspace linting rules defined in [workspace.lints]
9. THE Backend_Service SHALL use workspace package metadata (version, edition, authors, license)
10. THE Backend_Service SHALL integrate with CI/CD pipeline for automated builds and tests

### Requirement 26: CI/CD and Tooling Configuration Updates

**User Story:** As a DevOps engineer, I want all CI/CD and tooling configurations updated to reflect the new crate structure, so that automated workflows continue to function correctly.

#### Acceptance Criteria

1. THE Backend_Service SHALL update .github/workflows/ci.yml to build and test layanan-perlengkapan on self-hosted runners instead of ubuntu-latest
2. THE Backend_Service SHALL update .github/workflows/release.yml to package layanan-perlengkapan as a single artifact on self-hosted runners
3. THE Backend_Service SHALL update .github/dependabot.yml to monitor layanan/perlengkapan directory for dependency updates
4. THE Backend_Service SHALL update .github/workflows/security.yml to scan layanan-perlengkapan for vulnerabilities on self-hosted runners
5. THE Backend_Service SHALL update Docker build workflows to use the unified crate structure on self-hosted runners
6. THE Backend_Service SHALL update any cargo-deny configuration to include layanan-perlengkapan
7. THE Backend_Service SHALL update code coverage workflows to collect coverage from the unified crate on self-hosted runners
8. THE Backend_Service SHALL update deployment scripts to deploy the single layanan-perlengkapan service
9. THE Backend_Service SHALL remove CI/CD configurations for old separate crates (api, dokumen, notifikasi, bantuan)
10. THE Backend_Service SHALL verify all GitHub Actions workflows pass after configuration updates

### Requirement 27: Infrastructure and Deployment Configuration

**User Story:** As a platform engineer, I want infrastructure configurations updated for the unified service, so that deployment and operations work correctly.

#### Acceptance Criteria

1. THE Backend_Service SHALL update Kubernetes manifests to deploy layanan-perlengkapan as a single deployment
2. THE Backend_Service SHALL update Helm charts in infra/helm/simpel/templates/ to reflect the unified service
3. THE Backend_Service SHALL update service discovery configurations to point to the unified service endpoint
4. THE Backend_Service SHALL update monitoring dashboards to track metrics from layanan-perlengkapan
5. THE Backend_Service SHALL update alerting rules to monitor the unified service health
6. THE Backend_Service SHALL update load balancer and ingress configurations for the unified API endpoints
7. THE Backend_Service SHALL update Secreton policies to grant layanan-perlengkapan access to required secrets
8. THE Backend_Service SHALL update resource quotas and limits for the unified service pod
9. THE Backend_Service SHALL update horizontal pod autoscaler (HPA) configuration for the unified service
10. THE Backend_Service SHALL remove infrastructure configurations for old separate services

### Requirement 28: Documentation and Repository Cleanup

**User Story:** As a developer, I want clean repository structure and updated documentation, so that the codebase is easy to understand and maintain.

#### Acceptance Criteria

1. THE Backend_Service SHALL remove layanan/perlengkapan/crates/ directory after code migration to unified structure
2. THE Backend_Service SHALL update root README.md to document the unified perlengkapan service architecture
3. THE Backend_Service SHALL update root AGENTS.md to reflect the unified layanan/perlengkapan structure
4. THE Backend_Service SHALL update CONTRIBUTING.md with guidelines for the unified crate structure
5. THE Backend_Service SHALL update layanan/AGENTS.md to document the unified perlengkapan service pattern
6. THE Backend_Service SHALL update layanan/perlengkapan/AGENTS.md to reflect the new internal module organization and remove references to separate crates
7. THE Backend_Service SHALL update antarmuka/AGENTS.md if needed to reflect new backend API structure
8. THE Backend_Service SHALL remove obsolete documentation referencing separate crates (api, dokumen, notifikasi, bantuan)
9. THE Backend_Service SHALL update architecture diagrams in all AGENTS.md files to show the unified service structure
10. THE Backend_Service SHALL update API documentation to reflect consolidated endpoints
11. THE Backend_Service SHALL add migration notes explaining the transition from planned separate crates to unified structure
12. THE Backend_Service SHALL update developer onboarding documentation with the new structure
13. THE Backend_Service SHALL ensure all code comments and inline documentation reference the correct module paths
14. THE Backend_Service SHALL update any .github/agents/ custom agent definitions that reference perlengkapan structure
15. THE Backend_Service SHALL update any .github/prompts/ that reference perlengkapan development patterns
