# Authenc Enterprise Features - Implementation Tasks

## Overview

This task list implements the Authenc Enterprise Features specification to achieve full Keycloak parity and production readiness for Indonesian government deployment. The implementation is organized into 4 phases over 9 months, addressing 31 functional requirements (FR-1 through FR-31) and 66 non-functional requirements (NFR-1 through NFR-66).

**Feature**: authenc-keycloak-parity
**Total Estimated Duration**: 9 months (36 weeks)
**Priority**: CRITICAL for production deployment

---

## Phase 1: Critical Features (Q1 2026 - 3 months)

**Goal**: Make Authenc production-ready with full management capabilities

### 0. Prerequisite: UI Architecture Decision - CRITICAL

**Addresses**: Code organization and maintainability
**Decision**: UI Authenc akan digabung ke `antarmuka/portal/` untuk efektivitas, efisiensi, dan optimalisasi

#### 0.1 Remove Old admin_console Folder (if exists)

- [ ] 0.1.1 Check if `infra/authenc/src/admin_console/` folder exists
- [ ] 0.1.2 If exists, delete folder and all its contents
- [ ] 0.1.3 Remove `admin_console` module reference from `infra/authenc/src/lib.rs` if exists
- [ ] 0.1.4 Remove `admin_console` feature flag from `infra/authenc/Cargo.toml` if exists
- [ ] 0.1.5 Verify build succeeds: `cargo check --bin authenc`

#### 0.2 Verify Portal Structure

- [ ] 0.2.1 Verify `antarmuka/portal/` exists and is properly configured
- [ ] 0.2.2 Verify `lib-ui` is accessible from portal workspace
- [ ] 0.2.3 Check existing portal routing structure
- [ ] 0.2.4 Verify portal can call Authenc REST API endpoints

### 1. Admin Console UI (8 weeks) - CRITICAL

**Addresses**: FR-1 through FR-6, NFR-1, NFR-19-24, NFR-25-29, NFR-30-34

#### 1.1 Admin Console Architecture & Setup (Week 1-2)

- [ ] 1.1.1 Create Authenc UI module in portal: `antarmuka/portal/src/pages/authenc/`
- [ ] 1.1.2 Create admin console folder: `antarmuka/portal/src/pages/authenc/admin/`
- [ ] 1.1.3 Create account console folder: `antarmuka/portal/src/pages/authenc/account/`
- [ ] 1.1.4 Create auth pages folder: `antarmuka/portal/src/pages/authenc/auth/` (login, register, etc.)
- [ ] 1.1.5 Implement base layout component with navigation sidebar using `lib-ui`
- [ ] 1.1.6 Add Authenc routes to portal router (Leptos Router)
- [ ] 1.1.7 Implement authentication check and redirect to login
- [ ] 1.1.8 Verify Tailwind CSS integration via lib-ui works
- [ ] 1.1.9 Use shared UI components from `lib-ui/components/` (buttons, forms, tables, modals)
- [ ] 1.1.10 Implement error boundary and loading states
- [ ] 1.1.11 Set up i18n support (Indonesian and English) - reuse portal's i18n
- [ ] 1.1.12 Write unit tests for routing and layout components

#### 1.2 Realm Management UI (Week 3)

- [ ] 1.2.1 Create realm list page with search and filtering
- [ ] 1.2.2 Implement realm creation form with validation (FR-1.1)
- [ ] 1.2.3 Create realm settings page (enable/disable toggle) (FR-1.2)
- [ ] 1.2.4 Implement realm deletion with confirmation dialog (FR-1.4)
- [ ] 1.2.5 Add realm name validation (alphanumeric, no spaces) (FR-1.3)
- [ ] 1.2.6 Create REST API endpoints in Axum backend for realm CRUD
- [ ] 1.2.7 Write property-based test for Property 1 (realm enable/disable persistence)
- [ ] 1.2.8 Write property-based test for Property 2 (realm name validation)
- [ ] 1.2.9 Write property-based test for Property 3 (realm deletion protection)
- [ ] 1.2.10 Write integration tests for realm management flow

#### 1.3 User Management UI (Week 4)

- [ ] 1.3.1 Create user list page with pagination (50 per page)
- [ ] 1.3.2 Implement user search by username, email, NIP (FR-2.2)
- [ ] 1.3.3 Create user creation form with validation (FR-2.1)
- [ ] 1.3.4 Implement user edit page (profile, attributes, roles)
- [ ] 1.3.5 Add user enable/disable toggle
- [ ] 1.3.6 Create user deletion with confirmation
- [ ] 1.3.7 Implement bulk user operations (import CSV, bulk enable/disable) (FR-2.3)
- [ ] 1.3.8 Add user session viewer with termination capability (FR-2.4)
- [ ] 1.3.9 Implement admin password reset functionality (FR-2.5)
- [ ] 1.3.10 Write property-based test for Property 31 (multi-field search)
- [ ] 1.3.11 Write property-based test for Property 32 (search result accuracy)
- [ ] 1.3.12 Write integration tests for user management flow

#### 1.4 Client Management UI (Week 5)

- [ ] 1.4.1 Create client list page with search
- [ ] 1.4.2 Implement client registration form (FR-3.1)
- [ ] 1.4.3 Add client secret generation with secure display (FR-3.2)
- [ ] 1.4.4 Create redirect URI management (add/remove multiple URIs) (FR-3.3, FR-3.4)
- [ ] 1.4.5 Implement client secret rotation functionality (FR-3.5)
- [ ] 1.4.6 Add client enable/disable toggle
- [ ] 1.4.7 Create client deletion with confirmation
- [ ] 1.4.8 Implement protocol configuration (OAuth2, OIDC, SAML)
- [ ] 1.4.9 Write property-based test for Property 40 (client secret security)
- [ ] 1.4.10 Write property-based test for Property 41 (redirect URI validation)
- [ ] 1.4.11 Write property-based test for Property 42 (client secret rotation)
- [ ] 1.4.12 Write integration tests for client management flow

#### 1.5 Role Management UI (Week 6)

- [ ] 1.5.1 Create realm roles list page
- [ ] 1.5.2 Create client roles list page
- [ ] 1.5.3 Implement role creation form (FR-4.1)
- [ ] 1.5.4 Add role assignment to users interface (FR-4.4)
- [ ] 1.5.5 Add role assignment to groups interface (FR-4.4)
- [ ] 1.5.6 Create role deletion with confirmation
- [ ] 1.5.7 Implement role description editing
- [ ] 1.5.8 Add role search and filtering
- [ ] 1.5.9 Write integration tests for role management flow

#### 1.6 Audit Log Viewer (Week 7)

- [ ] 1.6.1 Create admin events viewer page with pagination (FR-5.1)
- [ ] 1.6.2 Create user events viewer page with pagination (FR-5.1)
- [ ] 1.6.3 Implement event filtering by type, user, date, IP (FR-5.2)
- [ ] 1.6.4 Add event export to CSV functionality (FR-5.3)
- [ ] 1.6.5 Add event export to JSON functionality (FR-5.3)
- [ ] 1.6.6 Implement event search with multiple filters
- [ ] 1.6.7 Add security event highlighting (FR-5.5)
- [ ] 1.6.8 Create event detail modal view
- [ ] 1.6.9 Write property-based test for Property 43 (admin event immutability)
- [ ] 1.6.10 Write property-based test for Property 44 (event filtering accuracy)
- [ ] 1.6.11 Write property-based test for Property 45 (event export format)
- [ ] 1.6.12 Write integration tests for audit log viewer

#### 1.7 System Monitoring Dashboard (Week 8)

- [ ] 1.7.1 Create dashboard page with metrics grid (FR-6.1)
- [ ] 1.7.2 Implement real-time active users counter
- [ ] 1.7.3 Add active sessions counter
- [ ] 1.7.4 Display requests per second metric
- [ ] 1.7.5 Show database connection pool status (FR-6.2)
- [ ] 1.7.6 Display Redis cache statistics (FR-6.3)
- [ ] 1.7.7 Show recent errors and warnings (FR-6.4)
- [ ] 1.7.8 Implement auto-refresh every 30 seconds (FR-6.6)
- [ ] 1.7.9 Add manual refresh button
- [ ] 1.7.10 Create charts for historical metrics
- [ ] 1.7.11 Write property-based test for Property 46 (metrics accuracy)
- [ ] 1.7.12 Write property-based test for Property 47 (auto-refresh consistency)
- [ ] 1.7.13 Write integration tests for monitoring dashboard

### 2. Enhanced Event System (3 weeks) - CRITICAL

**Addresses**: FR-13 through FR-15

#### 2.1 Event Database Schema (Week 1)

- [ ] 2.1.1 Create migration for admin_events table (FR-13.1)
- [ ] 2.1.2 Create migration for user_events table (FR-13.1)
- [ ] 2.1.3 Create migration for system_events table (FR-13.1)
- [ ] 2.1.4 Create migration for event_config table (FR-14.1)
- [ ] 2.1.5 Add indexes for efficient event querying
- [ ] 2.1.6 Create AdminEventStore with CRUD operations
- [ ] 2.1.7 Create UserEventStore with CRUD operations
- [ ] 2.1.8 Create SystemEventStore with CRUD operations
- [ ] 2.1.9 Write property-based test for Property 4 (event category separation)
- [ ] 2.1.10 Write property-based test for Property 5 (admin event completeness)
- [ ] 2.1.11 Write property-based test for Property 6 (user event completeness)

#### 2.2 Event Service Implementation (Week 2)

- [ ] 2.2.1 Implement EventService with log_admin_event method (FR-13.2)
- [ ] 2.2.2 Implement log_user_event method (FR-13.3)
- [ ] 2.2.3 Implement log_system_event method (FR-13.4)
- [ ] 2.2.4 Add event filtering by type, user, date, IP
- [ ] 2.2.5 Implement event search functionality
- [ ] 2.2.6 Create event retention policy enforcement (FR-14.2)
- [ ] 2.2.7 Implement cleanup_expired_events scheduled task (FR-14.3)
- [ ] 2.2.8 Add event statistics aggregation
- [ ] 2.2.9 Write property-based test for Property 7 (event retention enforcement)
- [ ] 2.2.10 Write integration tests for event service

#### 2.3 Event Export Integration (Week 3)

- [ ] 2.3.1 Implement Kafka producer for event streaming (FR-15.1)
- [ ] 2.3.2 Add Kafka export configuration per realm (FR-15.3)
- [ ] 2.3.3 Implement Elasticsearch client for event indexing (FR-15.2)
- [ ] 2.3.4 Add Elasticsearch export configuration per realm (FR-15.3)
- [ ] 2.3.5 Implement retry logic with exponential backoff (FR-15.4)
- [ ] 2.3.6 Add export status monitoring (FR-15.5)
- [ ] 2.3.7 Create event export configuration UI in Admin Console
- [ ] 2.3.8 Write integration tests for Kafka export
- [ ] 2.3.9 Write integration tests for Elasticsearch export

### 3. Required Actions Framework (2 weeks) - CRITICAL

**Addresses**: FR-17

#### 3.1 Required Actions Database Schema (Week 1)

- [ ] 3.1.1 Create migration for required_actions table
- [ ] 3.1.2 Create migration for user_required_actions table
- [ ] 3.1.3 Add indexes for efficient querying
- [ ] 3.1.4 Create RequiredActionStore with CRUD operations
- [ ] 3.1.5 Implement built-in required actions (verify email, update password, configure MFA, accept terms)
- [ ] 3.1.6 Add government-specific required actions (update NIP, verify identity, accept privacy policy, update Satker)

#### 3.2 Required Actions Service & UI (Week 2)

- [ ] 3.2.1 Implement RequiredActionService with enable/disable methods
- [ ] 3.2.2 Add set_default_action method for new users
- [ ] 3.2.3 Implement assign_required_action_to_user method
- [ ] 3.2.4 Create required actions configuration UI in Admin Console
- [ ] 3.2.5 Implement required actions blocking logic in authentication flow
- [ ] 3.2.6 Create required actions completion UI for users
- [ ] 3.2.7 Add required action priority/ordering support
- [ ] 3.2.8 Write integration tests for required actions flow

### 4. User Account Console (4 weeks) - CRITICAL

**Addresses**: FR-7 through FR-12, NFR-19-24
**Location**: `antarmuka/portal/src/pages/authenc/account/`

#### 4.1 Profile Management (Week 1)

- [ ] 4.1.1 Create profile management pages in `antarmuka/portal/src/pages/authenc/account/`
- [ ] 4.1.2 Implement profile view page (FR-7.1)
- [ ] 4.1.3 Create profile edit form with validation using `lib-ui` components (FR-7.2)
- [ ] 4.1.4 Add read-only display for system fields (username, NIP) (FR-7.3)
- [ ] 4.1.5 Implement profile update validation (FR-7.4)
- [ ] 4.1.6 Add confirmation email after profile changes (FR-7.5)
- [ ] 4.1.7 Create REST API endpoints in Authenc backend for profile management
- [ ] 4.1.8 Write property-based test for Property 33 (profile update validation)
- [ ] 4.1.9 Write integration tests for profile management

#### 4.2 Password Management (Week 1)

- [ ] 4.2.1 Create password change form (FR-8.1)
- [ ] 4.2.2 Implement current password verification (FR-8.1)
- [ ] 4.2.3 Add password complexity validation (FR-8.2)
- [ ] 4.2.4 Implement session termination after password change (FR-8.3)
- [ ] 4.2.5 Add notification email after password change (FR-8.4)
- [ ] 4.2.6 Implement password reuse prevention (last 5 passwords) (FR-8.5)
- [ ] 4.2.7 Write property-based test for Property 34 (password change session termination)
- [ ] 4.2.8 Write integration tests for password management

#### 4.3 MFA Device Management (Week 2)

- [ ] 4.3.1 Create MFA devices list page (FR-9.1)
- [ ] 4.3.2 Implement TOTP device registration with QR code (FR-9.2)
- [ ] 4.3.3 Add WebAuthn device registration (FR-9.3)
- [ ] 4.3.4 Implement device removal with password confirmation (FR-9.4)
- [ ] 4.3.5 Add protection against removing all devices when MFA required (FR-9.5)
- [ ] 4.3.6 Implement backup code generation and display (FR-9.6)
- [ ] 4.3.7 Add device nickname editing
- [ ] 4.3.8 Write property-based test for Property 35 (MFA device removal protection)
- [ ] 4.3.9 Write property-based test for Property 48 (backup code uniqueness)
- [ ] 4.3.10 Write property-based test for Property 49 (backup code single-use)
- [ ] 4.3.11 Write integration tests for MFA device management

#### 4.4 Session Management (Week 3)

- [ ] 4.4.1 Create active sessions list page (FR-10.1)
- [ ] 4.4.2 Display session details (device, location, last activity) (FR-10.1)
- [ ] 4.4.3 Mark current session clearly (FR-10.2)
- [ ] 4.4.4 Implement individual session termination (FR-10.3)
- [ ] 4.4.5 Add "terminate all other sessions" functionality (FR-10.4)
- [ ] 4.4.6 Send notification when session terminated remotely (FR-10.4)
- [ ] 4.4.7 Write property-based test for Property 36 (session termination notification)
- [ ] 4.4.8 Write integration tests for session management

#### 4.5 Activity Log & Data Management (Week 4)

- [ ] 4.5.1 Create activity log viewer page (FR-11.1)
- [ ] 4.5.2 Display login history (FR-11.1)
- [ ] 4.5.3 Show password changes (FR-11.2)
- [ ] 4.5.4 Display MFA device changes (FR-11.3)
- [ ] 4.5.5 Show profile updates (FR-11.4)
- [ ] 4.5.6 Include timestamp, IP, device for each event (FR-11.5)
- [ ] 4.5.7 Implement pagination (50 events per page) (FR-11.6)
- [ ] 4.5.8 Create personal data export functionality (FR-12.1)
- [ ] 4.5.9 Implement export link delivery via email (FR-12.2)
- [ ] 4.5.10 Add export link expiration (7 days) (FR-12.3)
- [ ] 4.5.11 Create account deletion request form (FR-12.4)
- [ ] 4.5.12 Implement 30-day grace period for deletion (FR-12.5)
- [ ] 4.5.13 Add deletion cancellation functionality (FR-12.6)
- [ ] 4.5.14 Write property-based test for Property 37 (data export completeness)
- [ ] 4.5.15 Write property-based test for Property 38 (deletion grace period)
- [ ] 4.5.16 Write property-based test for Property 39 (deletion cancellation)
- [ ] 4.5.17 Write property-based test for Property 50 (activity log completeness)
- [ ] 4.5.18 Write property-based test for Property 51 (activity log pagination)
- [ ] 4.5.19 Write integration tests for activity log and data management

---

## Phase 2: High-Priority Features (Q2 2026 - 3 months)

**Goal**: Improve usability and integration

### 5. Authentication Flows Customization (4 weeks) - HIGH

**Addresses**: FR-16

#### 5.1 Authentication Flow Database Schema (Week 1)

- [ ] 5.1.1 Create migration for authentication_flows table
- [ ] 5.1.2 Create migration for authentication_executions table
- [ ] 5.1.3 Add indexes for efficient querying
- [ ] 5.1.4 Create AuthenticationFlowStore with CRUD operations
- [ ] 5.1.5 Create AuthenticationExecutionStore with CRUD operations
- [ ] 5.1.6 Seed built-in flows (browser, direct grant, registration, reset credentials)

#### 5.2 Authentication Flow Service (Week 2)

- [ ] 5.2.1 Implement AuthenticationFlowService with create_flow method (FR-16.1)
- [ ] 5.2.2 Add add_execution method (FR-16.2)
- [ ] 5.2.3 Implement remove_execution method (FR-16.2)
- [ ] 5.2.4 Add reorder_executions method (FR-16.3)
- [ ] 5.2.5 Implement copy_flow method (FR-16.4)
- [ ] 5.2.6 Add delete_flow method with built-in protection (FR-16.5)
- [ ] 5.2.7 Implement execute_flow method with requirement logic
- [ ] 5.2.8 Write property-based test for Property 8 (flow execution ordering)
- [ ] 5.2.9 Write property-based test for Property 9 (required execution enforcement)
- [ ] 5.2.10 Write property-based test for Property 10 (alternative execution logic)

#### 5.3 Authentication Flow UI (Week 3-4)

- [ ] 5.3.1 Create authentication flows list page in Admin Console
- [ ] 5.3.2 Implement flow creation form (FR-16.1)
- [ ] 5.3.3 Create flow editor with execution list (FR-16.2)
- [ ] 5.3.4 Add drag-and-drop for execution reordering (FR-16.3)
- [ ] 5.3.5 Implement execution requirement selector (REQUIRED, ALTERNATIVE, DISABLED, CONDITIONAL)
- [ ] 5.3.6 Add flow copy functionality (FR-16.4)
- [ ] 5.3.7 Implement flow deletion with built-in protection (FR-16.5)
- [ ] 5.3.8 Create client flow override configuration (FR-16.6)
- [ ] 5.3.9 Add flow testing/preview functionality
- [ ] 5.3.10 Write integration tests for authentication flow management

### 6. Composite Roles System (2 weeks) - HIGH

**Addresses**: FR-18, FR-19

#### 6.1 Composite Roles Database Schema (Week 1)

- [ ] 6.1.1 Create migration for composite_roles table
- [ ] 6.1.2 Add composite boolean field to roles table
- [ ] 6.1.3 Add role_type field (realm/client) to roles table
- [ ] 6.1.4 Create indexes for efficient querying
- [ ] 6.1.5 Add circular dependency check constraint
- [ ] 6.1.6 Create CompositeRoleStore with CRUD operations

#### 6.2 Composite Roles Service & UI (Week 2)

- [ ] 6.2.1 Implement add_composite_role method (FR-18.1)
- [ ] 6.2.2 Add remove_composite_role method (FR-18.2)
- [ ] 6.2.3 Implement get_composite_roles method (FR-18.1)
- [ ] 6.2.4 Add get_effective_roles method with transitive closure (FR-18.2)
- [ ] 6.2.5 Implement circular dependency detection (FR-18.4)
- [ ] 6.2.6 Create composite role management UI in Admin Console
- [ ] 6.2.7 Add role hierarchy visualization (FR-18.5)
- [ ] 6.2.8 Implement realm role vs client role scoping (FR-19.1, FR-19.2)
- [ ] 6.2.9 Add role scope mapping UI (FR-19.3)
- [ ] 6.2.10 Implement token role filtering by client (FR-19.4)
- [ ] 6.2.11 Write property-based test for Property 11 (circular dependency detection)
- [ ] 6.2.12 Write property-based test for Property 12 (composite role inheritance)
- [ ] 6.2.13 Write property-based test for Property 13 (role type scoping)
- [ ] 6.2.14 Write integration tests for composite roles

### 7. User Storage Federation (3 weeks) - HIGH

**Addresses**: FR-20, FR-21, FR-29

#### 7.1 User Storage Provider SPI (Week 1)

- [ ] 7.1.1 Define UserStorageProvider trait (FR-20.1)
- [ ] 7.1.2 Create migration for user_storage_providers table
- [ ] 7.1.3 Implement provider priority ordering (FR-20.1)
- [ ] 7.1.4 Add read-only vs read-write provider support (FR-20.2)
- [ ] 7.1.5 Implement credential validation delegation (FR-20.4)
- [ ] 7.1.6 Add attribute mapping configuration (FR-20.5)
- [ ] 7.1.7 Implement user caching for federated users (FR-20.6)
- [ ] 7.1.8 Write property-based test for Property 14 (credential delegation)
- [ ] 7.1.9 Write property-based test for Property 15 (provider priority ordering)

#### 7.2 MySIMKARI Integration (Week 2)

- [ ] 7.2.1 Implement MysimkariUserStorageProvider (FR-21.1)
- [ ] 7.2.2 Add NIP validation via MySIMKARI gRPC (FR-21.2, FR-29.2)
- [ ] 7.2.3 Implement auto-populate user attributes from MySIMKARI (FR-21.3)
- [ ] 7.2.4 Add MySIMKARI response caching (5 minutes) (FR-21.4, FR-29.4)
- [ ] 7.2.5 Implement read-only mode for MySIMKARI users (FR-21.5)
- [ ] 7.2.6 Add NIP format validation (18 digits) (FR-29.1)
- [ ] 7.2.7 Implement error handling for MySIMKARI unavailability (FR-29.5)
- [ ] 7.2.8 Write property-based test for Property 16 (NIP validation with MySIMKARI)
- [ ] 7.2.9 Write property-based test for Property 28 (NIP format validation)
- [ ] 7.2.10 Write property-based test for Property 52 (MySIMKARI cache validity)

#### 7.3 User Storage Provider UI & Testing (Week 3)

- [ ] 7.3.1 Create user storage provider configuration UI in Admin Console
- [ ] 7.3.2 Add provider priority management UI
- [ ] 7.3.3 Implement provider enable/disable toggle
- [ ] 7.3.4 Add provider testing functionality (test connection)
- [ ] 7.3.5 Create attribute mapping configuration UI
- [ ] 7.3.6 Write integration tests for user storage federation
- [ ] 7.3.7 Write integration tests for MySIMKARI integration

### 8. User Self-Registration (3 weeks) - HIGH

**Addresses**: FR-24, FR-25

#### 8.1 Registration Configuration (Week 1)

- [ ] 8.1.1 Create migration for registration_config table
- [ ] 8.1.2 Create migration for registration_fields table
- [ ] 8.1.3 Implement RegistrationConfigStore with CRUD operations
- [ ] 8.1.4 Add government-specific registration fields (NIP, Satker, phone)
- [ ] 8.1.5 Implement field validation rules (regex patterns) (FR-25.4)
- [ ] 8.1.6 Add select field options configuration (FR-25.5)

#### 8.2 Registration Page & Logic (Week 2)

- [ ] 8.2.1 Create public registration page (FR-24.1)
- [ ] 8.2.2 Implement NIP validation during registration (FR-24.2)
- [ ] 8.2.3 Add email verification requirement (FR-24.3)
- [ ] 8.2.4 Implement CAPTCHA protection (FR-24.4)
- [ ] 8.2.5 Add terms and conditions acceptance (FR-24.5)
- [ ] 8.2.6 Implement auto-assign default roles (FR-24.6)
- [ ] 8.2.7 Add admin notification for new registrations (FR-24.7)
- [ ] 8.2.8 Write property-based test for Property 20 (email verification requirement)
- [ ] 8.2.9 Write property-based test for Property 21 (NIP validation during registration)
- [ ] 8.2.10 Write property-based test for Property 22 (default role assignment)

#### 8.3 Registration Customization UI (Week 3)

- [ ] 8.3.1 Create registration configuration UI in Admin Console
- [ ] 8.3.2 Add custom field management UI (FR-25.1)
- [ ] 8.3.3 Implement field type selector (text, email, select, checkbox) (FR-25.2)
- [ ] 8.3.4 Add required/optional field configuration (FR-25.3)
- [ ] 8.3.5 Create validation rule editor (FR-25.4)
- [ ] 8.3.6 Implement select field options editor (FR-25.5)
- [ ] 8.3.7 Add registration preview functionality
- [ ] 8.3.8 Write integration tests for user registration flow

### 9. Email Template System (2 weeks) - HIGH

**Addresses**: FR-26

#### 9.1 Email Template Engine (Week 1)

- [ ] 9.1.1 Create migration for email_templates table
- [ ] 9.1.2 Implement EmailTemplateStore with CRUD operations
- [ ] 9.1.3 Create default templates (verification, password reset, password changed, account updated, MFA enabled, login alert)
- [ ] 9.1.4 Implement template variable substitution (user name, NIP, Satker, links, IP, timestamp) (FR-26.2)
- [ ] 9.1.5 Add multi-language support (Indonesian, English) (FR-26.3)
- [ ] 9.1.6 Implement HTML and plain text versions (FR-26.4)
- [ ] 9.1.7 Add government branding to templates (FR-26.7)
- [ ] 9.1.8 Write property-based test for Property 23 (template variable substitution)
- [ ] 9.1.9 Write property-based test for Property 24 (template language selection)

#### 9.2 Email Template UI (Week 2)

- [ ] 9.2.1 Create email templates list page in Admin Console
- [ ] 9.2.2 Implement template editor with syntax highlighting
- [ ] 9.2.3 Add template preview functionality (FR-26.5)
- [ ] 9.2.4 Implement test email sending (FR-26.6)
- [ ] 9.2.5 Add template variable documentation
- [ ] 9.2.6 Create template reset to default functionality
- [ ] 9.2.7 Write integration tests for email template system

---

## Phase 3: Medium-Priority Features (Q3 2026 - 3 months)

**Goal**: Operational excellence and flexibility

### 10. Session Management Enhancement (2 weeks) - MEDIUM

**Addresses**: FR-22, FR-23

#### 10.1 Session Policies (Week 1)

- [ ] 10.1.1 Implement maximum sessions per user limit (FR-22.1)
- [ ] 10.1.2 Add oldest session revocation when limit reached (FR-22.2)
- [ ] 10.1.3 Implement configurable session idle timeout (FR-22.3)
- [ ] 10.1.4 Add configurable session maximum lifespan (FR-22.4)
- [ ] 10.1.5 Implement "Remember Me" functionality (FR-22.5)
- [ ] 10.1.6 Add offline sessions support for mobile apps (FR-22.6)
- [ ] 10.1.7 Write property-based test for Property 17 (session limit enforcement)
- [ ] 10.1.8 Write property-based test for Property 18 (session idle timeout)
- [ ] 10.1.9 Write property-based test for Property 19 (session max lifespan)

#### 10.2 Session Cleanup & UI (Week 2)

- [ ] 10.2.1 Implement automatic session cleanup (hourly) (FR-23.1)
- [ ] 10.2.2 Add idle session deletion (FR-23.2)
- [ ] 10.2.3 Implement max lifespan session deletion (FR-23.3)
- [ ] 10.2.4 Add cleanup statistics logging (FR-23.4)
- [ ] 10.2.5 Create manual cleanup trigger in Admin Console (FR-23.5)
- [ ] 10.2.6 Add session policy configuration UI
- [ ] 10.2.7 Write integration tests for session management

### 11. Import/Export System (3 weeks) - MEDIUM

**Addresses**: FR-27, FR-28

#### 11.1 Realm Export (Week 1)

- [ ] 11.1.1 Implement realm configuration export to JSON (FR-27.1)
- [ ] 11.1.2 Add include/exclude users option (FR-27.2)
- [ ] 11.1.3 Add include/exclude credentials option (FR-27.3)
- [ ] 11.1.4 Implement large export file splitting (FR-27.4)
- [ ] 11.1.5 Export clients, roles, groups, identity providers, flows, scopes (FR-27.5)
- [ ] 11.1.6 Write property-based test for Property 25 (realm export completeness)
- [ ] 11.1.7 Write property-based test for Property 26 (export inclusion options)

#### 11.2 Realm Import (Week 2)

- [ ] 11.2.1 Implement realm configuration import from JSON (FR-28.1)
- [ ] 11.2.2 Add overwrite or create new realm option (FR-28.2)
- [ ] 11.2.3 Implement import validation before applying (FR-28.3)
- [ ] 11.2.4 Add clear error reporting (FR-28.4)
- [ ] 11.2.5 Implement import rollback on failure (FR-28.5)
- [ ] 11.2.6 Add import progress display (FR-28.6)
- [ ] 11.2.7 Write property-based test for Property 27 (import/export round-trip)

#### 11.3 Import/Export UI (Week 3)

- [ ] 11.3.1 Create realm export page in Admin Console
- [ ] 11.3.2 Add export options configuration UI
- [ ] 11.3.3 Implement export download functionality
- [ ] 11.3.4 Create realm import page
- [ ] 11.3.5 Add import file upload
- [ ] 11.3.6 Implement import preview/validation
- [ ] 11.3.7 Add import progress indicator
- [ ] 11.3.8 Write integration tests for import/export

### 12. Government-Specific Features (3 weeks) - HIGH

**Addresses**: FR-29, FR-30, FR-31

#### 12.1 Satker Hierarchy (Week 1)

- [ ] 12.1.1 Create migration for satker_hierarchy table
- [ ] 12.1.2 Implement SatkerStore with CRUD operations
- [ ] 12.1.3 Add user Satker assignment storage (FR-30.1)
- [ ] 12.1.4 Load Satker hierarchy from database (FR-30.2)
- [ ] 12.1.5 Implement hierarchical access control enforcement (FR-30.3)
- [ ] 12.1.6 Add child Satker access logic (FR-30.4)
- [ ] 12.1.7 Implement special permissions to override Satker restrictions (FR-30.5)
- [ ] 12.1.8 Include Satker path in JWT tokens (FR-30.6)
- [ ] 12.1.9 Write property-based test for Property 29 (Satker hierarchy access control)
- [ ] 12.1.10 Write property-based test for Property 30 (Satker assignment persistence)
- [ ] 12.1.11 Write property-based test for Property 53 (Satker hierarchy transitivity)
- [ ] 12.1.12 Write property-based test for Property 54 (Satker access control inheritance)

#### 12.2 Audit Reports (Week 2)

- [ ] 12.2.1 Implement audit report generation for date ranges (FR-31.1)
- [ ] 12.2.2 Add total users metric
- [ ] 12.2.3 Add logins metric
- [ ] 12.2.4 Add failed logins metric
- [ ] 12.2.5 Add MFA adoption rate metric
- [ ] 12.2.6 Add admin actions metric
- [ ] 12.2.7 Add security incidents metric
- [ ] 12.2.8 Add compliance status metric
- [ ] 12.2.9 Implement PDF export with government branding (FR-31.2, FR-31.3)
- [ ] 12.2.10 Write property-based test for Property 55 (audit report completeness)
- [ ] 12.2.11 Write property-based test for Property 56 (audit report PDF generation)

#### 12.3 Government Features UI (Week 3)

- [ ] 12.3.1 Create Satker hierarchy management UI in Admin Console
- [ ] 12.3.2 Add Satker assignment UI for users
- [ ] 12.3.3 Implement Satker access control configuration UI
- [ ] 12.3.4 Create audit report generation UI
- [ ] 12.3.5 Add report date range selector
- [ ] 12.3.6 Implement report preview
- [ ] 12.3.7 Add PDF download functionality
- [ ] 12.3.8 Write integration tests for government-specific features

---

## Phase 4: Polish & Documentation (Ongoing)

### 13. Testing & Quality Assurance

- [ ] 13.1 Ensure all 56 property-based tests are implemented and passing
- [ ] 13.2 Achieve >80% unit test coverage (NFR-37)
- [ ] 13.3 Achieve >70% integration test coverage (NFR-38)
- [ ] 13.4 Implement E2E tests for critical user flows (NFR-39)
- [ ] 13.5 Run cargo clippy without warnings (NFR-40)
- [ ] 13.6 Format all code with cargo fmt (NFR-41)
- [ ] 13.7 Conduct security audit and penetration testing
- [ ] 13.8 Perform load testing (1000+ concurrent users) (NFR-4)
- [ ] 13.9 Verify performance targets (p95 <100ms) (NFR-2)
- [ ] 13.10 Test WCAG 2.1 Level AA compliance (NFR-19)

### 14. Documentation

- [ ] 14.1 Write Admin Console user guide
- [ ] 14.2 Write User Account Console user guide
- [ ] 14.3 Create API documentation for all REST endpoints
- [ ] 14.4 Document all gRPC services
- [ ] 14.5 Write deployment guide
- [ ] 14.6 Create troubleshooting guide
- [ ] 14.7 Document all configuration options
- [ ] 14.8 Write migration guide from Keycloak (if needed)
- [ ] 14.9 Create video tutorials for common tasks
- [ ] 14.10 Translate documentation to Indonesian

### 15. Deployment & Operations

- [ ] 15.1 Create Docker images <300MB (NFR-59)
- [ ] 15.2 Write Kubernetes manifests (NFR-60)
- [ ] 15.3 Create Helm charts (NFR-61)
- [ ] 15.4 Configure CI/CD pipeline (NFR-62)
- [ ] 15.5 Set up blue-green deployment (NFR-63)
- [ ] 15.6 Configure canary deployment (NFR-64)
- [ ] 15.7 Automate database migrations (NFR-65)
- [ ] 15.8 Verify startup time <5 seconds (NFR-66)
- [ ] 15.9 Set up monitoring and alerting
- [ ] 15.10 Configure backup and disaster recovery

---

## Success Criteria

### Feature Completeness
- [ ] All 31 functional requirements (FR-1 through FR-31) implemented
- [ ] All 66 non-functional requirements (NFR-1 through NFR-66) met
- [ ] All 56 correctness properties validated through property-based tests
- [ ] Admin Console fully functional with all management capabilities
- [ ] User Account Console fully functional with all self-service features

### Quality Metrics
- [ ] Test coverage >80% (unit tests)
- [ ] Test coverage >70% (integration tests)
- [ ] Zero critical security vulnerabilities
- [ ] <5 high-priority bugs in production
- [ ] Performance targets met (p95 <100ms, <2s page load)
- [ ] WCAG 2.1 Level AA compliance achieved

### Operational Readiness
- [ ] 99.9% uptime achieved in staging
- [ ] Successful deployment in 1+ government agency (Satker)
- [ ] Documentation complete and translated
- [ ] Support team trained
- [ ] Disaster recovery tested

---

## Notes

**UI Architecture Decision**:
UI Authenc digabung ke `antarmuka/portal/` untuk efektivitas, efisiensi, dan optimalisasi:
- **Location**: `antarmuka/portal/src/pages/authenc/`
  - `admin/` - Admin Console (realm, user, client, role management)
  - `account/` - User Account Console (profile, password, MFA, sessions)
  - `auth/` - Authentication Pages (login, register, forgot password, MFA challenges)
- **Benefits**:
  - Maximal code reuse dengan `lib-ui` components
  - Consistent styling dengan portal lainnya
  - Single build system (Trunk + Leptos 0.8.x)
  - Shared i18n, routing, dan authentication logic
  - Simplified deployment (satu portal untuk semua UI)

**Communication Pattern**:
- Portal UI → Authenc REST API (JSON/HTTP) → Authenc backend (Axum)
- Authenc backend → Secreton gRPC (untuk secrets)
- Authenc backend → MySIMKARI gRPC (untuk NIP validation)

**Security Considerations**:
- Admin Console routes protected dengan role-based access control
- User Account Console accessible untuk authenticated users
- Auth pages (login, register) public accessible
- All API calls dari portal ke Authenc backend menggunakan JWT authentication

**Coding Standards**:
- All tasks should follow SIMPelv2 coding standards (see AGENTS.md)
- Use Leptos 0.8.x for all UI components (signal(), not create_signal!)
- All REST API endpoints use Axum 0.8.x patterns
- All gRPC services use Tonic 0.14.x + Prost 0.14.x
- Property-based tests use proptest with minimum 100 iterations
- Each property test must be tagged with: `// Feature: authenc-keycloak-parity, Property {number}: {property_text}`
- All database migrations use refinery
- All secrets must be stored in Secreton (never environment variables)
- All external service calls (MySIMKARI, Secreton) use gRPC with mTLS
- Microfrontends call backend REST API only (never gRPC directly)

---

**Document Version**: 1.0
**Last Updated**: 2026-02-18
**Status**: READY FOR IMPLEMENTATION
