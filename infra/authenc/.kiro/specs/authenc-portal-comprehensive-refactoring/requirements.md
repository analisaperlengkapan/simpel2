# Requirements Document: Authenc & Portal IAM Comprehensive Refactoring

## 1. Overview

### 1.1 Purpose
This document specifies the functional and non-functional requirements for the comprehensive refactoring of the SIMPelv2 authentication system, derived from the approved design document.

### 1.2 Scope
The requirements cover:
1. **Authenc Multi-Crate Architecture**: Decomposition of monolithic authenc service into modular crates
2. **Portal IAM Microfrontend**: Complete rebuild as modern IAM portal with direct REST API integration
3. **Elimination of layanan-portal**: Direct communication between portal and Authenc

### 1.3 Stakeholders
- **Development Team**: Implementation and maintenance
- **Security Team**: Security audit and compliance
- **Operations Team**: Deployment and monitoring
- **End Users**: Authentication and self-service
- **Administrators**: IAM administration and configuration

---

## 2. Functional Requirements

### 2.1 Authentication Requirements

#### 2.1.1 User Authentication
**REQ-AUTH-001**: The system SHALL support username/password authentication
- Username: 3-50 characters, alphanumeric + underscore
- Password: Minimum 8 characters with complexity requirements
- Password hashing: Argon2id algorithm

**REQ-AUTH-002**: The system SHALL support multi-factor authentication (MFA)
- TOTP (Time-based One-Time Password)
- SMS-based OTP (optional)
- Email-based OTP (optional)
- WebAuthn/FIDO2 Passkeys (MANDATORY - PRIMARY authentication method)

**REQ-AUTH-003**: The system SHALL implement brute force protection
- Maximum 5 failed attempts before account lockout
- Lockout duration: 15 minutes
- CAPTCHA requirement after 3 failed attempts
- Exponential backoff for repeated failures

**REQ-AUTH-004**: The system SHALL support session management
- Session creation on successful authentication
- Session timeout: 15 minutes idle, 8 hours absolute
- Session invalidation on logout
- Multiple concurrent sessions per user

**REQ-AUTH-005**: The system SHALL support passwordless authentication (MANDATORY)
- WebAuthn/FIDO2 Passkeys as PRIMARY authentication method
- Usernameless authentication (discoverable credentials)
- Multi-device passkey support (sync via platform providers)
- Platform authenticator support (Touch ID, Face ID, Windows Hello)
- Security key support (YubiKey, Titan Key, etc.)
- Phishing-resistant authentication
- Origin-bound credentials
- Replay attack prevention via credential counter

#### 2.1.2 Token Management
**REQ-TOKEN-001**: The system SHALL generate JWT access tokens
- Algorithm: Ed25519 signature
- Token lifetime: 15 minutes
- Claims: sub (user ID), exp (expiry), iss (issuer), scope, realm

**REQ-TOKEN-002**: The system SHALL generate refresh tokens
- Token lifetime: 7 days
- Secure storage in database
- Token rotation on refresh

**REQ-TOKEN-003**: The system SHALL validate JWT tokens
- Signature verification
- Expiration check
- Issuer validation
- Revocation list check

**REQ-TOKEN-004**: The system SHALL support token revocation
- Individual token revocation
- User-level token revocation (all tokens)
- Revocation list with TTL

---

### 2.2 User Management Requirements

#### 2.2.1 User CRUD Operations
**REQ-USER-001**: The system SHALL support user creation
- Required fields: username, email, password
- Optional fields: first name, last name, phone number
- Email verification workflow
- Welcome email notification

**REQ-USER-002**: The system SHALL support user profile updates
- Self-service profile editing
- Admin-initiated profile updates
- Audit logging for all changes

**REQ-USER-003**: The system SHALL support user deletion
- Soft delete with retention period
- Hard delete after retention period
- Cascade deletion of related data
- GDPR compliance (right to be forgotten)

**REQ-USER-004**: The system SHALL support user search and filtering
- Search by username, email, name
- Filter by status (enabled/disabled)
- Filter by realm
- Filter by role
- Pagination support (20 items per page)

#### 2.2.2 Password Management
**REQ-PASS-001**: The system SHALL support password changes
- Current password verification required
- New password complexity validation
- Password history check (last 5 passwords)
- Notification on password change

**REQ-PASS-002**: The system SHALL support password reset
- Email-based reset workflow
- Reset token expiration: 1 hour
- One-time use reset tokens
- Notification on password reset

**REQ-PASS-003**: The system SHALL enforce password policies
- Minimum length: 8 characters
- Complexity: uppercase, lowercase, digit, special character
- Password expiration (optional, configurable per realm)
- Password reuse prevention

---

### 2.3 Realm Management Requirements

**REQ-REALM-001**: The system SHALL support multi-realm architecture
- Realm isolation for users, clients, and sessions
- Realm-specific configuration
- Realm-specific branding

**REQ-REALM-002**: The system SHALL support realm CRUD operations
- Create realm with unique name
- Update realm configuration
- Enable/disable realm
- Delete realm (with confirmation)

**REQ-REALM-003**: The system SHALL support realm configuration
- Authentication policies
- Token lifetimes
- MFA requirements
- Password policies
- Session policies

---

### 2.4 OAuth2/OIDC Requirements

#### 2.4.1 OAuth2 Flows
**REQ-OAUTH-001**: The system SHALL support Authorization Code flow
- PKCE enforcement for public clients (OAuth 2.1)
- Authorization code expiration: 10 minutes
- Single-use authorization codes
- Redirect URI validation

**REQ-OAUTH-002**: The system SHALL support Client Credentials flow
- Service-to-service authentication
- Client authentication via client_secret
- Scope-based authorization

**REQ-OAUTH-003**: The system SHALL support Refresh Token flow
- Refresh token rotation
- Refresh token expiration: 7 days
- Revocation on security events

#### 2.4.2 OIDC Support
**REQ-OIDC-001**: The system SHALL support OpenID Connect
- UserInfo endpoint
- ID Token generation
- Standard OIDC claims (sub, name, email, etc.)
- Discovery endpoint (/.well-known/openid-configuration)

**REQ-OIDC-002**: The system SHALL support OIDC scopes
- openid (required)
- profile (name, given_name, family_name, etc.)
- email (email, email_verified)
- Custom scopes per realm

#### 2.4.3 Client Management
**REQ-CLIENT-001**: The system SHALL support OAuth2 client registration
- Client ID generation
- Client secret generation (for confidential clients)
- Redirect URI whitelist
- Allowed scopes configuration

**REQ-CLIENT-002**: The system SHALL support client types
- Public clients (no client secret, PKCE required)
- Confidential clients (client secret required)
- Service accounts (client credentials flow)

**REQ-CLIENT-003**: The system SHALL support client policies
- OAuth 2.1 compliance enforcement
- FAPI-1 compliance (optional)
- FAPI-2 compliance (optional)
- Custom policy executors

---

### 2.5 Federation/SSO Requirements

**REQ-FED-001**: The system SHALL support external identity providers
- OIDC providers (Google, Microsoft, etc.)
- SAML 2.0 providers
- LDAP/Active Directory (optional)

**REQ-FED-002**: The system SHALL support identity brokering
- User account linking
- Attribute mapping
- Just-in-time provisioning
- Identity provider chaining

**REQ-FED-003**: The system SHALL support SSO flows
- SP-initiated SSO
- IdP-initiated SSO
- Single Logout (SLO)

---

### 2.6 MFA Requirements

**REQ-MFA-001**: The system SHALL support TOTP setup
- QR code generation for authenticator apps
- Secret storage in Secreton
- Backup codes generation (10 codes)

**REQ-MFA-002**: The system SHALL support TOTP verification
- 6-digit code validation
- Time window tolerance: ±1 period (30 seconds)
- Rate limiting: 3 attempts per minute

**REQ-MFA-003**: The system SHALL support backup codes
- One-time use codes
- 10 codes per user
- Regeneration on demand
- Notification on backup code usage

**REQ-MFA-004**: The system SHALL support WebAuthn/FIDO2 Passkeys (MANDATORY)
- Passkey registration
- Passkey authentication
- Multiple passkeys per user
- Platform authenticator support
- Security key support
- Usernameless authentication

---

### 2.7 WebAuthn/Passkeys Requirements (MANDATORY)

**REQ-WEBAUTHN-001**: The system SHALL support passkey registration
- Generate WebAuthn creation challenge
- Verify attestation response
- Store credential in database
- Exclude existing credentials from registration
- Support credential nickname assignment

**REQ-WEBAUTHN-002**: The system SHALL support passkey authentication
- Generate WebAuthn authentication challenge
- Verify assertion response
- Support usernameless authentication (discoverable credentials)
- Update credential counter on successful authentication
- Update last used timestamp

**REQ-WEBAUTHN-003**: The system SHALL support credential management
- List user's registered passkeys
- Delete passkey
- Update passkey nickname
- View passkey metadata (created date, last used date)

**REQ-WEBAUTHN-004**: The system SHALL prevent replay attacks
- Maintain credential counter per passkey
- Verify counter increments on each authentication
- Reject authentication if counter does not increment

**REQ-WEBAUTHN-005**: The system SHALL support multi-device passkeys
- Sync passkeys via platform providers (iCloud Keychain, Google Password Manager)
- Support cross-device authentication
- Support both platform and roaming authenticators

**REQ-WEBAUTHN-006**: The system SHALL support platform authenticators
- Touch ID (macOS, iOS)
- Face ID (iOS, iPadOS)
- Windows Hello (Windows)
- Android biometric authentication

**REQ-WEBAUTHN-007**: The system SHALL support security keys
- FIDO2 security keys (YubiKey, Titan Key, etc.)
- USB, NFC, and Bluetooth security keys
- Multiple security keys per user

**REQ-WEBAUTHN-008**: The system SHALL enforce origin binding
- Verify relying party ID matches expected value
- Verify origin matches expected value
- Prevent credential use on different origins

**REQ-WEBAUTHN-009**: The system SHALL support attestation (optional)
- Verify authenticator attestation
- Support attestation formats (packed, fido-u2f, etc.)
- Store attestation metadata

**REQ-WEBAUTHN-010**: The system SHALL provide user verification
- Require user verification for sensitive operations
- Support biometric verification
- Support PIN verification
- Configurable user verification policy

---

### 2.8 Role and Permission Requirements

**REQ-ROLE-001**: The system SHALL support role-based access control (RBAC)
- Role creation and management
- Role assignment to users
- Role hierarchy (optional)
- Realm-specific roles

**REQ-ROLE-002**: The system SHALL support permission management
- Permission definition
- Permission assignment to roles
- Permission checking in API endpoints
- Permission-based UI rendering

**REQ-ROLE-003**: The system SHALL support attribute-based access control (ABAC)
- User attributes
- Resource attributes
- Environment attributes
- Policy evaluation engine

---

### 2.8 Audit and Logging Requirements

**REQ-AUDIT-001**: The system SHALL log all authentication events
- Login attempts (success/failure)
- Logout events
- MFA verification attempts
- Token generation and validation

**REQ-AUDIT-002**: The system SHALL log all administrative actions
- User CRUD operations
- Realm configuration changes
- Client registration/updates
- Role and permission changes

**REQ-AUDIT-003**: The system SHALL support audit log querying
- Filter by event type
- Filter by user
- Filter by date range
- Filter by realm
- Export to CSV/JSON

**REQ-AUDIT-004**: The system SHALL retain audit logs
- Retention period: 90 days (configurable)
- Archival to external storage
- Compliance with government regulations

---

### 2.9 FAPI Compliance Requirements (OPTIONAL - Phased Implementation)

#### 2.9.1 OAuth 2.1 Baseline (CURRENT - Already Implemented)
**REQ-FAPI-001**: The system SHALL support OAuth 2.1 baseline security
- PKCE enforcement for all clients (public and confidential)
- Authorization Code flow only (implicit flow disabled)
- Refresh token rotation
- HTTPS enforcement for all endpoints
- Redirect URI exact matching

#### 2.9.2 FAPI-1 Implementation (OPTIONAL - Phase 2)
**REQ-FAPI-002**: The system MAY support FAPI-1 Advanced Profile (optional)
- mTLS for token endpoint (client authentication)
- Signed Request Objects (JAR - JWT-secured Authorization Requests)
- ID Token as detached signature
- Enhanced token validation
- Request object encryption (optional)
- JARM (JWT-secured Authorization Response Mode)

**Acceptance Criteria for FAPI-1**:
- AC-FAPI-001: Token endpoint accepts mTLS client certificates
- AC-FAPI-002: Authorization endpoint accepts signed request objects
- AC-FAPI-003: ID tokens include c_hash and s_hash claims
- AC-FAPI-004: Authorization responses use JARM format

#### 2.9.3 FAPI-2 Implementation (OPTIONAL - Phase 3)
**REQ-FAPI-003**: The system MAY support FAPI-2 Security Profile (optional)
- Pushed Authorization Requests (PAR)
- DPoP (Demonstrating Proof-of-Possession)
- Sender-constrained access tokens
- Enhanced client authentication methods
- Grant Management API
- RAR (Rich Authorization Requests)

**Acceptance Criteria for FAPI-2**:
- AC-FAPI-005: PAR endpoint accepts authorization requests
- AC-FAPI-006: DPoP proof validation for token requests
- AC-FAPI-007: Access tokens bound to DPoP keys
- AC-FAPI-008: Grant management endpoints available

**Implementation Phases**:
- **Phase 1 (CURRENT)**: OAuth 2.1 baseline - COMPLETED
- **Phase 2 (OPTIONAL)**: FAPI-1 Advanced - 4-6 weeks implementation
- **Phase 3 (OPTIONAL)**: FAPI-2 Security - 6-8 weeks implementation

**Note**: FAPI compliance is OPTIONAL and should only be implemented if required for specific integrations (e.g., financial institutions, government agencies requiring FAPI certification).

---

### 2.10 Portal IAM Microfrontend Requirements

#### 2.10.1 Authentication Pages
**REQ-PORTAL-001**: The portal SHALL provide login page
- Username/password input
- Passkey authentication option (PRIMARY - default)
- Remember me option
- Forgot password link
- SSO provider buttons

**REQ-PORTAL-002**: The portal SHALL provide MFA verification page
- TOTP code input
- Backup code option
- Passkey authentication prompt (if supported)

**REQ-PORTAL-003**: The portal SHALL provide password reset page
- Email input
- Reset token validation
- New password input
- Password strength indicator

#### 2.10.2 User Dashboard
**REQ-PORTAL-004**: The portal SHALL provide user dashboard
- Profile overview
- Recent activity
- Active sessions
- Security alerts
- Quick actions (change password, setup MFA, manage passkeys)

#### 2.10.3 User Self-Service
**REQ-PORTAL-005**: The portal SHALL provide profile management
- View profile information
- Edit profile fields
- Upload profile picture (optional)
- Change email (with verification)

**REQ-PORTAL-006**: The portal SHALL provide password management
- Change password form
- Password strength indicator
- Password requirements display

**REQ-PORTAL-007**: The portal SHALL provide MFA management
- Setup TOTP
- View backup codes
- Regenerate backup codes
- Disable MFA (with confirmation)

**REQ-PORTAL-008**: The portal SHALL provide session management
- View active sessions
- Revoke individual sessions
- Revoke all sessions

**REQ-PORTAL-009**: The portal SHALL provide passkey management (MANDATORY)
- View registered passkeys
- Add new passkey
- Delete passkey (with confirmation)
- Update passkey nickname
- View passkey metadata (created date, last used date)
- Passkey registration flow UI
- Passkey authentication flow UI

#### 2.10.4 IAM Administration (Admin-only)
**REQ-PORTAL-010**: The portal SHALL provide user management interface
- User list with search and filters
- Create user form
- Edit user form
- Delete user (with confirmation)
- Reset user password
- Enable/disable user
- Assign roles to user

**REQ-PORTAL-011**: The portal SHALL provide realm management interface
- Realm list
- Create realm form
- Edit realm configuration
- Delete realm (with confirmation)
- Realm switching

**REQ-PORTAL-012**: The portal SHALL provide client management interface
- Client list
- Create client form
- Edit client configuration
- Delete client (with confirmation)
- Regenerate client secret
- View client credentials

**REQ-PORTAL-013**: The portal SHALL provide role management interface
- Role list
- Create role form
- Edit role
- Delete role (with confirmation)
- Assign permissions to role

**REQ-PORTAL-014**: The portal SHALL provide federation management interface
- Identity provider list
- Create identity provider form
- Edit identity provider configuration
- Delete identity provider (with confirmation)
- Test identity provider connection

**REQ-PORTAL-015**: The portal SHALL provide audit log viewer
- Audit log list with filters
- Event detail view
- Export audit logs (CSV/JSON)
- Real-time log streaming (optional)

**REQ-PORTAL-016**: The portal SHALL provide system configuration interface
- View system configuration
- Edit configuration parameters
- Restart services (with confirmation)
- View system health

---

## 3. Non-Functional Requirements

### 3.1 Performance Requirements

**REQ-PERF-001**: Authentication latency SHALL be < 100ms (p99)
- Measured from request receipt to response sent
- Excludes network latency
- Under normal load conditions

**REQ-PERF-002**: Token validation latency SHALL be < 50ms (p99)
- Cached validation results
- Measured from request receipt to response sent

**REQ-PERF-003**: The system SHALL support 1000 req/s authentication throughput
- Horizontal scaling capability
- Load balancing support
- Connection pooling

**REQ-PERF-004**: Database query latency SHALL be < 10ms (p95)
- Prepared statement caching
- Connection pooling (20 connections default)
- Index optimization

**REQ-PERF-005**: Portal page load time SHALL be < 2 seconds
- Initial page load
- Measured on 3G connection
- Includes WASM download and initialization

---

### 3.2 Scalability Requirements

**REQ-SCALE-001**: The system SHALL support horizontal scaling
- Stateless API servers
- Shared session storage (Redis)
- Database connection pooling

**REQ-SCALE-002**: The system SHALL support 10,000 concurrent users
- Per realm
- With acceptable performance degradation

**REQ-SCALE-003**: The system SHALL support 100,000 total users
- Across all realms
- With efficient database queries

---

### 3.3 Security Requirements

**REQ-SEC-001**: All passwords SHALL be hashed with Argon2id
- Memory cost: 64 MB
- Time cost: 3 iterations
- Parallelism: 4 threads
- Salt: 16 bytes random

**REQ-SEC-002**: All JWT tokens SHALL be signed with Ed25519
- Private key stored in Secreton
- Key rotation every 90 days
- Old keys retained for verification

**REQ-SEC-003**: All API endpoints SHALL use HTTPS
- TLS 1.3 minimum
- Strong cipher suites only
- HSTS header enabled

**REQ-SEC-004**: All gRPC communication SHALL use mTLS
- Client certificate verification
- Certificate rotation support
- Certificate revocation checking

**REQ-SEC-005**: All sensitive data SHALL be encrypted at rest
- Database encryption (PostgreSQL)
- Secrets stored in Secreton
- ChaCha20-Poly1305 encryption

**REQ-SEC-006**: The system SHALL implement rate limiting
- Per-IP: 100 requests/minute
- Per-user: 1000 requests/minute
- Adaptive rate limiting based on risk score

**REQ-SEC-007**: The system SHALL implement CORS properly
- Whitelist allowed origins
- Restrict allowed methods
- Restrict allowed headers
- Credentials support for authenticated requests

**REQ-SEC-008**: The system SHALL implement input validation
- All inputs validated against schema
- SQL injection prevention
- XSS prevention
- CSRF protection

**REQ-SEC-009**: The system SHALL implement security headers
- Content-Security-Policy
- X-Frame-Options: DENY
- X-Content-Type-Options: nosniff
- Referrer-Policy: strict-origin-when-cross-origin

**REQ-SEC-010**: The system SHALL implement phishing-resistant authentication (MANDATORY)
- WebAuthn/Passkeys as PRIMARY authentication method
- Origin-bound credentials
- Public key cryptography (no shared secrets)
- Credential counter validation (replay attack prevention)

**REQ-SEC-011**: The system SHALL enforce origin binding for passkeys
- Verify relying party ID matches expected value
- Verify origin matches expected value
- Prevent credential use on different origins
- Reject authentication attempts from unauthorized origins

**REQ-SEC-012**: The system SHALL prevent replay attacks
- Maintain credential counter per passkey
- Verify counter increments on each authentication
- Reject authentication if counter does not increment or decreases
- Log replay attack attempts

**REQ-SEC-013**: The system SHALL support FAPI security compliance (OPTIONAL)
- mTLS for token endpoint (FAPI-1)
- Signed Request Objects (FAPI-1)
- DPoP proof validation (FAPI-2)
- Sender-constrained tokens (FAPI-2)

---

### 3.4 Reliability Requirements

**REQ-REL-001**: The system SHALL have 99.9% uptime
- Measured monthly
- Excludes planned maintenance
- Includes all components

**REQ-REL-002**: The system SHALL support graceful degradation
- Continue operation with Redis unavailable (no caching)
- Continue operation with Secreton unavailable (use local keys)
- Fallback mechanisms for non-critical features

**REQ-REL-003**: The system SHALL implement health checks
- Liveness probe: /health/live
- Readiness probe: /health/ready
- Startup probe: /health/startup
- Database connectivity check
- Redis connectivity check (if enabled)

**REQ-REL-004**: The system SHALL implement circuit breakers
- For external service calls
- Failure threshold: 5 failures in 10 seconds
- Open duration: 30 seconds
- Half-open test requests: 3

---

### 3.5 Maintainability Requirements

**REQ-MAINT-001**: Code SHALL have >80% test coverage
- Unit tests for all business logic
- Integration tests for API endpoints
- Property-based tests for critical functions

**REQ-MAINT-002**: Code SHALL pass all Clippy lints
- No warnings in CI/CD
- Custom lint rules for project standards

**REQ-MAINT-003**: Code SHALL be formatted with rustfmt
- Consistent formatting across codebase
- Enforced in CI/CD

**REQ-MAINT-004**: Each crate SHALL have <500 lines per file
- Modular code organization
- Clear separation of concerns

**REQ-MAINT-005**: All public APIs SHALL have documentation
- OpenAPI/Swagger for REST APIs
- Proto documentation for gRPC
- Code examples for common use cases

---

### 3.6 Usability Requirements

**REQ-USE-001**: The portal SHALL be responsive
- Mobile-first design
- Support 320px to 4K displays
- Touch-friendly controls

**REQ-USE-002**: The portal SHALL be accessible
- WCAG 2.1 AA compliance
- Keyboard navigation support
- Screen reader support
- ARIA labels and roles

**REQ-USE-003**: The portal SHALL support internationalization
- Indonesian language (primary)
- English language (secondary)
- RTL support (optional)

**REQ-USE-004**: The portal SHALL provide user feedback
- Loading indicators
- Success/error messages
- Toast notifications
- Confirmation dialogs

---

### 3.7 Compatibility Requirements

**REQ-COMPAT-001**: The system SHALL support modern browsers
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

**REQ-COMPAT-002**: The system SHALL maintain API backward compatibility
- API versioning (/api/v1/, /api/v2/)
- Deprecation notices (6 months minimum)
- Migration guides for breaking changes

**REQ-COMPAT-003**: The system SHALL support database migrations
- Forward migrations (up)
- Rollback migrations (down)
- Idempotent migrations
- Zero-downtime migrations

---

### 3.8 WebAuthn Compatibility Requirements (MANDATORY)

**REQ-NFR-010**: The system SHALL support WebAuthn browser compatibility
- Chrome 67+ (WebAuthn Level 1)
- Firefox 60+ (WebAuthn Level 1)
- Safari 13+ (WebAuthn Level 1)
- Edge 18+ (WebAuthn Level 1)
- Chrome 93+ (WebAuthn Level 2 - conditional UI)
- Safari 16+ (WebAuthn Level 3 - passkey sync)

**REQ-NFR-011**: The system SHALL support passkey sync across devices
- iCloud Keychain (Apple ecosystem)
- Google Password Manager (Android/Chrome)
- Platform-specific passkey providers
- Cross-device authentication

**REQ-NFR-012**: The system SHALL support FAPI certification readiness (OPTIONAL)
- FAPI-1 Advanced Profile certification requirements
- FAPI-2 Security Profile certification requirements
- Conformance test suite compatibility
- Certification documentation

---

### 3.9 Deployment Requirements

**REQ-DEPLOY-001**: The system SHALL support containerized deployment
- Docker images for all services
- Multi-stage builds for optimization
- Health check endpoints

**REQ-DEPLOY-002**: The system SHALL support Kubernetes deployment
- Kustomize-based manifests
- Staging and production overlays
- HPA (Horizontal Pod Autoscaler) support
- PDB (Pod Disruption Budget) configuration

**REQ-DEPLOY-003**: The system SHALL support blue-green deployment
- Zero-downtime deployments
- Rollback capability
- Health check validation before traffic switch

---

### 3.10 Monitoring and Observability Requirements

**REQ-MON-001**: The system SHALL expose Prometheus metrics
- Authentication metrics (attempts, successes, failures)
- Token metrics (generated, validated, expired)
- Database metrics (connections, query duration)
- Cache metrics (hits, misses, evictions)
- gRPC metrics (requests, duration, status)

**REQ-MON-002**: The system SHALL implement structured logging
- JSON log format
- Log levels (ERROR, WARN, INFO, DEBUG, TRACE)
- Correlation IDs for request tracing
- No sensitive data in logs

**REQ-MON-003**: The system SHALL support distributed tracing
- OpenTelemetry integration
- Trace propagation via HTTP headers
- Trace propagation via gRPC metadata
- Jaeger/Zipkin export

**REQ-MON-004**: The system SHALL implement alerting
- High error rate alerts
- Database connection pool exhaustion
- High response time alerts
- Service down alerts

---

## 4. Architecture Requirements

### 4.1 Multi-Crate Structure Requirements

**REQ-ARCH-001**: Authenc SHALL be decomposed into separate crates
- authenc-types: Shared types and traits
- authenc-core: Business logic
- authenc-crypto: Cryptographic operations
- authenc-storage: Database layer
- authenc-api: Public REST API
- authenc-iam-api: Admin REST API
- authenc-grpc: gRPC service
- authenc-mfa: MFA logic
- authenc-federation: SSO/Federation
- authenc-cli: CLI tool

**REQ-ARCH-002**: Each crate SHALL have clear responsibilities
- No circular dependencies
- Dependency injection via traits
- Minimal public API surface

**REQ-ARCH-003**: authenc-types SHALL define all shared interfaces
- Domain types (User, Session, Realm, Client)
- Service traits (UserStore, SessionStore, etc.)
- Error types
- Configuration types

**REQ-ARCH-004**: authenc-core SHALL implement business logic
- Authentication service
- User management service
- Realm management service
- OAuth2/OIDC service
- Event publishing

**REQ-ARCH-005**: authenc-storage SHALL implement database layer
- PostgreSQL store implementations
- Connection pooling
- Prepared statement caching
- Transaction support
- Migration management

---

### 4.2 API Architecture Requirements

**REQ-API-001**: authenc-api SHALL provide public REST endpoints
- Authentication endpoints (/api/v1/auth/*)
- Token validation endpoint
- OAuth2/OIDC endpoints
- User profile endpoint
- CORS support for microfrontends

**REQ-API-002**: authenc-iam-api SHALL provide admin REST endpoints
- User management (/api/v1/iam/users/*)
- Realm management (/api/v1/iam/realms/*)
- Client management (/api/v1/iam/clients/*)
- Role management (/api/v1/iam/roles/*)
- Federation management (/api/v1/iam/identity-providers/*)
- Audit log access (/api/v1/iam/audit-logs/*)

**REQ-API-003**: authenc-grpc SHALL provide service-to-service API
- Authentication RPC
- Token validation RPC
- User management RPCs
- mTLS enforcement

---

### 4.3 Portal Architecture Requirements

**REQ-PORTAL-ARCH-001**: Portal SHALL be built with Leptos 0.8.x
- Client-side rendering (CSR) mode
- WASM compilation
- Trunk build tool

**REQ-PORTAL-ARCH-002**: Portal SHALL communicate directly with Authenc REST APIs
- No intermediate portal service layer
- Direct HTTP calls from WASM
- JWT token in Authorization header

**REQ-PORTAL-ARCH-003**: Portal SHALL use Leptos signals for state management
- RwSignal for mutable state
- Signal::derive for computed state
- Context providers for global state

**REQ-PORTAL-ARCH-004**: Portal SHALL implement resource-based API client
- Grouped methods by resource (users, realms, clients)
- Type-safe request/response types
- Error handling with Result types

**REQ-PORTAL-ARCH-005**: Portal SHALL use Tailwind CSS for styling
- Responsive design utilities
- Government branding colors
- Consistent component styling

---

### 4.4 Elimination of layanan-portal Requirements

**REQ-ELIM-001**: The layanan-portal service SHALL be eliminated
- No new features added to layanan-portal
- Existing functionality migrated to Authenc APIs
- Service decommissioned after migration

**REQ-ELIM-002**: Portal microfrontend SHALL call Authenc directly
- REST API calls from WASM
- No gRPC from frontend
- JWT token management in localStorage

---

## 5. Migration Requirements

### 5.1 Migration Strategy Requirements

**REQ-MIG-001**: Migration SHALL follow 6-phase approach
- Phase 1: Preparation (Week 1-2) - ✅ COMPLETED
- Phase 2: Core Migration (Week 3-6) - 🔄 IN PROGRESS
- Phase 3: API Migration (Week 7-8)
- Phase 4: Feature Migration (Week 9-10)
- Phase 5: Portal Refactoring (Week 11-12)
- Phase 6: Cleanup and Optimization (Week 13-14)

**REQ-MIG-002**: Migration SHALL maintain backward compatibility
- Old code maintained in parallel during migration
- API versioning for breaking changes
- Database schema backward compatible
- Dual-write pattern for critical data during transition

**REQ-MIG-003**: Migration SHALL have rollback capability
- Immediate rollback to old code if critical issues
- Gradual rollback by disabling new features via feature flags
- Data integrity maintained during rollback
- Rollback procedures documented and tested

**REQ-MIG-004**: Migration SHALL include comprehensive testing
- Unit tests for all new code (>80% coverage)
- Integration tests for API endpoints
- End-to-end tests for critical flows
- Performance testing before production
- Security testing after each phase

---

### 5.2 Crate Migration Requirements

**REQ-MIG-CRATE-001**: authenc-storage migration SHALL be completed first
- Database layer is foundation for all other crates
- All database operations migrated to `crates/storage/`
- Connection pooling, prepared statement caching, and transactions migrated
- Store implementations for all domain entities (users, sessions, realms, clients)
- Migration runner integrated
- Unit tests for all store implementations (>80% coverage)

**REQ-MIG-CRATE-002**: authenc-crypto migration SHALL be completed second
- Cryptographic operations are critical security components
- All crypto code migrated to `crates/crypto/`
- JWT generation and validation migrated
- Password hashing (Argon2id) migrated
- Key management (Ed25519, ECDSA) migrated
- Advanced crypto (PQC, mTLS, DPoP, SD-JWT) migrated
- Unit tests for all crypto operations (>90% coverage for security)

**REQ-MIG-CRATE-003**: authenc-core migration SHALL be completed third
- Business logic depends on storage and crypto
- All service implementations migrated to `crates/core/`
- Authentication service, user management, realm management migrated
- OAuth2/OIDC services migrated
- UMA 2.0, audit, event, and cache services migrated
- Configuration and SPI migrated
- Unit tests for all services (>80% coverage)

**REQ-MIG-CRATE-004**: authenc-webauthn migration SHALL be completed with core
- WebAuthn is MANDATORY PRIMARY authentication method
- WebAuthn service migrated to `crates/webauthn/`
- Credential store implementation verified
- Replay attack prevention tested
- Origin binding enforcement tested
- Unit tests and property-based tests (>90% coverage)

**REQ-MIG-CRATE-005**: authenc-api migration SHALL expose public REST endpoints
- Public authentication endpoints migrated to `crates/api/`
- OAuth2/OIDC public endpoints migrated
- WebAuthn endpoints migrated
- Federation/SSO endpoints migrated
- Middleware (auth, rate limiting, CORS) migrated
- Integration tests for all endpoints

**REQ-MIG-CRATE-006**: authenc-iam-api migration SHALL expose admin REST endpoints
- IAM administration endpoints migrated to `crates/iam-api/`
- User, realm, client, role management endpoints migrated
- Federation management endpoints migrated
- Audit log access endpoints migrated
- Admin authentication and authorization middleware migrated
- Integration tests for all admin endpoints

**REQ-MIG-CRATE-007**: authenc-grpc migration SHALL provide service-to-service API
- gRPC service implementation migrated to `crates/grpc/`
- Proto code generation verified
- mTLS configuration migrated
- gRPC interceptors migrated
- Integration tests with mTLS

**REQ-MIG-CRATE-008**: authenc-mfa migration SHALL provide MFA functionality
- MFA services migrated to `crates/mfa/`
- TOTP, backup codes, and MFA policies migrated
- MFA middleware migrated
- Unit tests for all MFA operations

**REQ-MIG-CRATE-009**: authenc-federation migration SHALL provide SSO/Federation
- Federation services migrated to `crates/federation/`
- SSO, identity brokering, SAML, and social login migrated
- User synchronization services migrated
- Integration tests with external IdPs

---

### 5.3 File Migration Requirements

**REQ-MIG-FILE-001**: File migration SHALL follow documented mapping
- All file migrations SHALL follow the mapping in design.md Section 2.2.3
- Source files SHALL be moved to target crate locations
- Imports SHALL be updated to use new crate paths
- No files SHALL be left in incorrect locations

**REQ-MIG-FILE-002**: Essential root files SHALL be preserved
- `src/main.rs` - Entry point (updated to use new crates)
- `src/lib.rs` - Re-exports (updated to re-export all crates)
- `src/app.rs` - Simplified AppState (updated to use new crates)
- `src/server.rs` - Server initialization (updated)
- `migrations/` - SQL migrations (stay in root)
- `proto/` - gRPC proto files (stay in root)

**REQ-MIG-FILE-003**: Optional features SHALL remain feature-gated
- `src/admin_console/` - Leptos admin UI (feature: admin_console)
- `src/bin/` - CLI tools (separate binaries)
- Feature flags SHALL be preserved during migration

**REQ-MIG-FILE-004**: Migrated files SHALL be removed from source
- After successful migration and testing, old files SHALL be removed
- Removal SHALL happen in Phase 6 (Cleanup)
- Removal SHALL be verified by CI/CD checks

---

### 5.4 Dependency Migration Requirements

**REQ-MIG-DEP-001**: Crate dependencies SHALL follow dependency order
- Migration order: types → storage → crypto → core → webauthn → api/iam-api/grpc → mfa/federation
- No circular dependencies SHALL be introduced
- Dependency graph SHALL be validated after each crate migration

**REQ-MIG-DEP-002**: Workspace dependencies SHALL be centralized
- All external dependencies SHALL be defined in root `Cargo.toml` `[workspace.dependencies]`
- Member crates SHALL use `dependency_name = { workspace = true }`
- No version specifications in member crate `Cargo.toml` files

**REQ-MIG-DEP-003**: Trait-based dependency injection SHALL be used
- Services SHALL depend on traits, not concrete implementations
- Traits SHALL be defined in `authenc-types`
- Implementations SHALL be in respective crates (storage, core, etc.)

---

### 5.5 Testing Migration Requirements

**REQ-MIG-TEST-001**: Tests SHALL be migrated with code
- Unit tests SHALL move with their corresponding modules
- Test utilities SHALL be shared via `authenc-types` or test-only modules
- Test coverage SHALL be maintained or improved (>80% target)

**REQ-MIG-TEST-002**: Integration tests SHALL be updated
- Integration tests SHALL use new crate APIs
- Test fixtures SHALL be updated to use new types
- End-to-end tests SHALL verify migration correctness

**REQ-MIG-TEST-003**: Property-based tests SHALL be added for critical functions
- Cryptographic operations SHALL have property-based tests
- WebAuthn operations SHALL have property-based tests
- Security invariants SHALL be tested with property-based tests

---

### 5.6 Documentation Migration Requirements

**REQ-MIG-DOC-001**: Documentation SHALL be updated during migration
- Each crate SHALL have comprehensive README.md
- Public APIs SHALL have rustdoc comments
- Architecture diagrams SHALL be updated to reflect new structure

**REQ-MIG-DOC-002**: Migration guide SHALL be maintained
- Step-by-step migration guide SHALL be kept up-to-date
- Rollback procedures SHALL be documented
- Troubleshooting guide SHALL be updated with common issues

**REQ-MIG-DOC-003**: AGENTS.md SHALL be updated
- Root AGENTS.md SHALL reflect new multi-crate architecture
- Authenc AGENTS.md SHALL document new crate structure
- Code patterns SHALL be updated for new architecture

---

### 5.7 Data Migration Requirements

**REQ-DATA-001**: Existing user data SHALL be preserved
- No data loss during migration
- User credentials remain valid
- Session continuity maintained
- Passkey credentials preserved and functional

**REQ-DATA-002**: Database schema changes SHALL be backward compatible
- Additive migrations only during transition
- Old columns retained until migration complete
- Database views for compatibility (if needed)
- Migration scripts tested on staging data

**REQ-DATA-003**: Secrets SHALL be migrated securely
- JWT signing keys SHALL remain in Secreton
- TOTP secrets SHALL remain in Secreton
- No secrets SHALL be exposed during migration
- Key rotation SHALL continue to work

---

### 5.8 Portal Migration Requirements

**REQ-MIG-PORTAL-001**: Portal SHALL be rebuilt with Leptos 0.8.x
- New portal SHALL use Leptos 0.8.x CSR mode
- Direct REST API integration with Authenc
- No intermediate portal service layer
- WebAuthn/Passkeys as PRIMARY authentication method

**REQ-MIG-PORTAL-002**: layanan-portal service SHALL be eliminated
- All portal functionality SHALL be migrated to direct Authenc API calls
- No new features SHALL be added to layanan-portal
- Service SHALL be decommissioned after migration complete
- Kubernetes manifests SHALL be updated to remove layanan-portal

**REQ-MIG-PORTAL-003**: Other microfrontends SHALL be updated
- Perlengkapan, Intel, and other microfrontends SHALL use Authenc API directly
- Token validation SHALL use Authenc REST API
- No gRPC calls from microfrontends

---

### 5.9 Deployment Migration Requirements

**REQ-MIG-DEPLOY-001**: Deployment SHALL support parallel operation
- Old and new code SHALL run in parallel during migration
- Traffic SHALL be gradually shifted to new code
- Rollback SHALL be possible at any time

**REQ-MIG-DEPLOY-002**: Kubernetes manifests SHALL be updated
- Authenc deployment SHALL use new multi-crate structure
- authenc-api and authenc-iam-api services SHALL be added
- authenc-grpc service SHALL be updated
- Portal deployment SHALL use new Leptos build
- layanan-portal deployment SHALL be removed

**REQ-MIG-DEPLOY-003**: CI/CD pipeline SHALL support multi-crate builds
- Workspace build SHALL compile all crates
- Crate-specific tests SHALL run in parallel
- Docker images SHALL be built for all services
- Deployment SHALL be automated with zero-downtime

---

### 5.10 Migration Validation Requirements

**REQ-MIG-VAL-001**: Each phase SHALL have validation checkpoint
- All tests SHALL pass before proceeding to next phase
- Performance SHALL be equal or better than baseline
- Security audit SHALL pass
- User acceptance testing SHALL be conducted

**REQ-MIG-VAL-002**: Migration SHALL be validated in staging
- Full migration SHALL be tested in staging environment
- Load testing SHALL be conducted in staging
- Security testing SHALL be conducted in staging
- Rollback procedures SHALL be tested in staging

**REQ-MIG-VAL-003**: Production migration SHALL be monitored
- Metrics SHALL be monitored during migration
- Error rates SHALL be tracked
- Performance SHALL be tracked
- Rollback SHALL be triggered if thresholds exceeded

---

## 6. Testing Requirements

### 6.1 Unit Testing Requirements

**REQ-TEST-001**: All business logic SHALL have unit tests
- Authentication service tests
- Password hasher tests
- JWT service tests
- User store tests
- Target: >80% line coverage, >90% branch coverage

### 6.2 Integration Testing Requirements

**REQ-TEST-002**: All API endpoints SHALL have integration tests
- End-to-end authentication flow
- OAuth2 authorization code flow
- MFA setup and verification
- Authenc-Portal integration

### 6.3 Property-Based Testing Requirements

**REQ-TEST-003**: Critical functions SHALL have property-based tests
- Password hash/verify roundtrip
- JWT encode/decode roundtrip
- Session expiry invariant
- TOTP verification properties

### 6.4 Performance Testing Requirements

**REQ-TEST-004**: System SHALL undergo load testing
- Authentication throughput: 1000 req/s
- Token validation throughput: 5000 req/s
- Database connection pool stress test
- Tools: wrk, k6, or custom benchmarks

### 6.5 Security Testing Requirements

**REQ-TEST-005**: System SHALL undergo security testing
- Penetration testing
- Vulnerability scanning (cargo audit)
- OWASP Top 10 validation
- Security code review

---

## 7. Documentation Requirements

**REQ-DOC-001**: All public APIs SHALL be documented
- OpenAPI/Swagger specification
- gRPC proto documentation
- Code examples for common use cases

**REQ-DOC-002**: Architecture SHALL be documented
- System architecture diagrams
- Component interaction diagrams
- Data flow diagrams
- Deployment architecture

**REQ-DOC-003**: Migration process SHALL be documented
- Step-by-step migration guide
- Rollback procedures
- Troubleshooting guide
- FAQ for common issues

**REQ-DOC-004**: User guides SHALL be provided
- End-user authentication guide
- Administrator IAM guide
- Developer integration guide
- API reference documentation

---

## 8. Compliance Requirements

**REQ-COMP-001**: System SHALL comply with Indonesian government standards
- Data residency requirements
- Security standards
- Audit requirements

**REQ-COMP-002**: System SHALL comply with GDPR (where applicable)
- Right to access personal data
- Right to be forgotten
- Data portability
- Consent management

**REQ-COMP-003**: System SHALL support OAuth 2.1 compliance
- PKCE enforcement for public clients
- Disable implicit flow
- Require HTTPS
- Token rotation

**REQ-COMP-004**: System SHALL support FAPI compliance (optional)
- FAPI-1: mTLS, PAR, JARM
- FAPI-2: DPoP, RAR, Grant Management

---

## 9. Success Criteria

### 9.1 Technical Success Criteria

**SUCCESS-TECH-001**: All unit tests pass with >80% coverage
**SUCCESS-TECH-002**: All integration tests pass
**SUCCESS-TECH-003**: Performance targets met (authentication <100ms p99)
**SUCCESS-TECH-004**: Security audit passes with no high/critical vulnerabilities
**SUCCESS-TECH-005**: Zero-downtime deployment achieved

### 9.2 Business Success Criteria

**SUCCESS-BIZ-001**: 50% reduction in time to add new features
**SUCCESS-BIZ-002**: 30% reduction in bug fix time
**SUCCESS-BIZ-003**: 99.9% uptime achieved
**SUCCESS-BIZ-004**: <0.1% authentication error rate
**SUCCESS-BIZ-005**: >90% user satisfaction (survey)

### 9.3 Migration Success Criteria

**SUCCESS-MIG-001**: All existing functionality preserved
**SUCCESS-MIG-002**: No data loss during migration
**SUCCESS-MIG-003**: Performance equal or better than old system
**SUCCESS-MIG-004**: All stakeholders trained on new system
**SUCCESS-MIG-005**: Old code successfully decommissioned

---

## 10. Acceptance Criteria

### 10.1 Authentication Acceptance Criteria

**AC-AUTH-001**: User can log in with valid credentials
- GIVEN a user with valid username and password
- WHEN the user submits login form
- THEN the user is authenticated and redirected to dashboard

**AC-AUTH-002**: User cannot log in with invalid credentials
- GIVEN a user with invalid password
- WHEN the user submits login form
- THEN an error message is displayed and user remains on login page

**AC-AUTH-003**: User is prompted for MFA when enabled
- GIVEN a user with MFA enabled
- WHEN the user submits valid credentials
- THEN the user is prompted for MFA code

**AC-AUTH-004**: Account is locked after 5 failed attempts
- GIVEN a user with 4 failed login attempts
- WHEN the user fails login for the 5th time
- THEN the account is locked for 15 minutes

**AC-AUTH-005**: User can reset password via email
- GIVEN a user who forgot password
- WHEN the user requests password reset
- THEN a reset email is sent with a valid reset link

---

### 10.2 User Management Acceptance Criteria

**AC-USER-001**: Admin can create new user
- GIVEN an admin user
- WHEN the admin submits create user form with valid data
- THEN a new user is created and appears in user list

**AC-USER-002**: Admin can edit user profile
- GIVEN an admin user viewing a user profile
- WHEN the admin updates user information
- THEN the user profile is updated and changes are saved

**AC-USER-003**: Admin can delete user
- GIVEN an admin user viewing a user profile
- WHEN the admin confirms user deletion
- THEN the user is soft-deleted and no longer appears in active user list

**AC-USER-004**: Admin can search users
- GIVEN an admin user on user management page
- WHEN the admin enters search criteria
- THEN matching users are displayed in the list

---

### 10.3 OAuth2 Acceptance Criteria

**AC-OAUTH-001**: Client can obtain authorization code
- GIVEN a registered OAuth2 client
- WHEN the client initiates authorization code flow
- THEN an authorization code is generated and returned

**AC-OAUTH-002**: Client can exchange code for tokens
- GIVEN a valid authorization code
- WHEN the client exchanges code for tokens
- THEN access token and refresh token are returned

**AC-OAUTH-003**: Client can refresh access token
- GIVEN a valid refresh token
- WHEN the client requests token refresh
- THEN a new access token is returned

---

### 10.4 MFA Acceptance Criteria

**AC-MFA-001**: User can setup TOTP
- GIVEN a user without MFA enabled
- WHEN the user initiates TOTP setup
- THEN a QR code is displayed and backup codes are generated

**AC-MFA-002**: User can verify TOTP code
- GIVEN a user with TOTP enabled
- WHEN the user enters valid TOTP code
- THEN authentication succeeds

**AC-MFA-003**: User can use backup code
- GIVEN a user with TOTP enabled but no access to authenticator
- WHEN the user enters valid backup code
- THEN authentication succeeds and backup code is consumed

---

### 10.5 WebAuthn/Passkeys Acceptance Criteria (MANDATORY)

**AC-PASSKEY-001**: User can register a passkey
- GIVEN an authenticated user without passkeys
- WHEN the user initiates passkey registration
- THEN a WebAuthn creation challenge is generated and browser prompts for biometric/PIN

**AC-PASSKEY-002**: User can authenticate with passkey
- GIVEN a user with registered passkey
- WHEN the user selects "Sign in with Passkey"
- THEN the browser prompts for biometric/PIN and authentication succeeds

**AC-PASSKEY-003**: User can authenticate without username (usernameless)
- GIVEN a user with discoverable credential
- WHEN the user clicks "Sign in with Passkey" without entering username
- THEN the browser shows passkey selection and authentication succeeds

**AC-PASSKEY-004**: User can manage multiple passkeys
- GIVEN a user with multiple registered passkeys
- WHEN the user views passkey management page
- THEN all passkeys are listed with metadata (nickname, created date, last used)

**AC-PASSKEY-005**: User can delete a passkey
- GIVEN a user viewing their passkeys
- WHEN the user confirms passkey deletion
- THEN the passkey is removed and can no longer be used for authentication

**AC-PASSKEY-006**: System prevents replay attacks
- GIVEN a user authenticating with passkey
- WHEN the same authentication response is submitted twice
- THEN the second attempt is rejected with replay attack error

**AC-PASSKEY-007**: Passkey is origin-bound
- GIVEN a passkey registered for domain A
- WHEN authentication is attempted from domain B
- THEN authentication fails with origin mismatch error

**AC-PASSKEY-008**: User can use platform authenticator
- GIVEN a device with Touch ID/Face ID/Windows Hello
- WHEN the user registers a passkey
- THEN the platform authenticator is used for biometric verification

**AC-PASSKEY-009**: User can use security key
- GIVEN a user with FIDO2 security key (YubiKey, etc.)
- WHEN the user registers a passkey
- THEN the security key is used for authentication

**AC-PASSKEY-010**: Passkey counter increments on each use
- GIVEN a user authenticating with passkey
- WHEN authentication succeeds
- THEN the credential counter is incremented and stored

---

### 10.6 FAPI Acceptance Criteria (OPTIONAL)

**AC-FAPI-001**: Token endpoint accepts mTLS client certificates (FAPI-1)
- GIVEN a confidential client with mTLS certificate
- WHEN the client requests tokens with certificate
- THEN tokens are issued after certificate validation

**AC-FAPI-002**: Authorization endpoint accepts signed request objects (FAPI-1)
- GIVEN a client with signed request object
- WHEN the authorization request includes request parameter
- THEN the request object is validated and authorization proceeds

**AC-FAPI-003**: ID tokens include hash claims (FAPI-1)
- GIVEN a successful authorization code exchange
- WHEN ID token is generated
- THEN ID token includes c_hash and s_hash claims

**AC-FAPI-004**: Authorization responses use JARM format (FAPI-1)
- GIVEN a client configured for JARM
- WHEN authorization completes
- THEN response is returned as signed JWT

**AC-FAPI-005**: PAR endpoint accepts authorization requests (FAPI-2)
- GIVEN a client using PAR
- WHEN authorization request is pushed to PAR endpoint
- THEN request_uri is returned for use in authorization endpoint

**AC-FAPI-006**: DPoP proof validation for token requests (FAPI-2)
- GIVEN a client using DPoP
- WHEN token request includes DPoP proof
- THEN proof is validated and token is bound to DPoP key

**AC-FAPI-007**: Access tokens bound to DPoP keys (FAPI-2)
- GIVEN a DPoP-bound access token
- WHEN token is used without valid DPoP proof
- THEN request is rejected with invalid_dpop_proof error

**AC-FAPI-008**: Grant management endpoints available (FAPI-2)
- GIVEN an issued grant
- WHEN client queries grant management endpoint
- THEN grant details are returned including scope and expiry

---

### 10.7 Portal Acceptance Criteria

**AC-PORTAL-001**: Portal loads within 2 seconds
- GIVEN a user accessing the portal
- WHEN the page loads
- THEN the portal is interactive within 2 seconds

**AC-PORTAL-002**: Portal is responsive on mobile
- GIVEN a user accessing portal on mobile device
- WHEN the user navigates the portal
- THEN all features are accessible and usable

**AC-PORTAL-003**: Portal supports keyboard navigation
- GIVEN a user using keyboard only
- WHEN the user navigates the portal
- THEN all interactive elements are accessible via keyboard

**AC-PORTAL-004**: Portal displays error messages clearly
- GIVEN a user encountering an error
- WHEN the error occurs
- THEN a clear error message is displayed with guidance

---

## 11. Constraints and Assumptions

### 11.1 Constraints

**CONSTRAINT-001**: Must use Rust Edition 2024, MSRV 1.90+
**CONSTRAINT-002**: Must use Leptos 0.8.x for portal microfrontend
**CONSTRAINT-003**: Must use PostgreSQL for database
**CONSTRAINT-004**: Must integrate with existing Secreton service
**CONSTRAINT-005**: Must maintain compatibility with existing microfrontends
**CONSTRAINT-006**: Must complete migration within 14 weeks

### 11.2 Assumptions

**ASSUMPTION-001**: PostgreSQL database is available and properly configured
**ASSUMPTION-002**: Secreton service is operational and accessible
**ASSUMPTION-003**: Kubernetes cluster is available for deployment
**ASSUMPTION-004**: Development team has Rust and Leptos expertise
**ASSUMPTION-005**: Existing users will be migrated without disruption
**ASSUMPTION-006**: Network infrastructure supports required throughput

---

## 12. Risks and Mitigation

### 12.1 Technical Risks

**RISK-TECH-001**: Performance degradation during migration
- **Mitigation**: Comprehensive performance testing, gradual rollout, rollback plan

**RISK-TECH-002**: Data loss during migration
- **Mitigation**: Comprehensive backup strategy, dual-write pattern, data validation

**RISK-TECH-003**: Security vulnerabilities in new code
- **Mitigation**: Security audit, penetration testing, code review

**RISK-TECH-004**: Integration issues with existing services
- **Mitigation**: Integration testing, staging environment testing, phased rollout

### 12.2 Project Risks

**RISK-PROJ-001**: Timeline overrun
- **Mitigation**: Agile methodology, regular progress reviews, scope management

**RISK-PROJ-002**: Resource constraints
- **Mitigation**: Clear prioritization, parallel work streams, external support if needed

**RISK-PROJ-003**: Stakeholder resistance
- **Mitigation**: Regular communication, training, demonstration of benefits

---

## 13. Dependencies

### 13.1 External Dependencies

**DEP-EXT-001**: PostgreSQL 16.x
**DEP-EXT-002**: Redis 7.x (optional, for caching)
**DEP-EXT-003**: Secreton service (for secret management)
**DEP-EXT-004**: Kubernetes cluster (for deployment)

### 13.2 Internal Dependencies

**DEP-INT-001**: lib-common (shared utilities)
**DEP-INT-002**: lib-ui (shared UI components)
**DEP-INT-003**: Existing database schema
**DEP-INT-004**: Existing user data

### 13.3 Crate Dependencies

**DEP-DEP-001**: webauthn-rs crate (MANDATORY)
- Version: 0.5.x or later
- Purpose: WebAuthn/FIDO2 protocol implementation
- Features: Passkey registration, authentication, credential management

**DEP-DEP-002**: FAPI-related crates (OPTIONAL)
- jsonwebtoken (JWT signing/verification)
- reqwest with mTLS support (FAPI-1)
- Additional FAPI compliance crates as needed

---

## 14. Glossary

**Authenc**: Authentication and authorization service for SIMPelv2
**Portal IAM**: Identity and Access Management portal microfrontend
**MFA**: Multi-Factor Authentication
**TOTP**: Time-based One-Time Password
**WebAuthn**: Web Authentication API - W3C standard for passwordless authentication
**Passkey**: FIDO2 credential that enables passwordless authentication
**FIDO2**: Fast Identity Online 2 - authentication standard
**Platform Authenticator**: Built-in biometric authenticator (Touch ID, Face ID, Windows Hello)
**Roaming Authenticator**: External security key (YubiKey, Titan Key)
**Discoverable Credential**: Passkey that can be used without username (usernameless authentication)
**OAuth2**: Open Authorization 2.0 protocol
**OIDC**: OpenID Connect protocol
**JWT**: JSON Web Token
**PKCE**: Proof Key for Code Exchange
**FAPI**: Financial-grade API - security profile for OAuth 2.0
**FAPI-1**: FAPI 1.0 Advanced Profile
**FAPI-2**: FAPI 2.0 Security Profile
**PAR**: Pushed Authorization Requests (RFC 9126)
**DPoP**: Demonstrating Proof-of-Possession (RFC 9449)
**JARM**: JWT-secured Authorization Response Mode
**JAR**: JWT-secured Authorization Requests
**RAR**: Rich Authorization Requests
**RBAC**: Role-Based Access Control
**ABAC**: Attribute-Based Access Control
**SSO**: Single Sign-On
**IdP**: Identity Provider
**SAML**: Security Assertion Markup Language
**LDAP**: Lightweight Directory Access Protocol
**mTLS**: Mutual TLS (Transport Layer Security)
**WASM**: WebAssembly
**CSR**: Client-Side Rendering
**HPA**: Horizontal Pod Autoscaler (Kubernetes)
**PDB**: Pod Disruption Budget (Kubernetes)
**WCAG**: Web Content Accessibility Guidelines
**GDPR**: General Data Protection Regulation

---

## 15. References

### 15.1 Design Documents
- [Design Document: Authenc & Portal IAM Comprehensive Refactoring](./design.md)

### 15.2 External Standards
- [OAuth 2.1 Authorization Framework](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-07)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [WebAuthn Level 3](https://www.w3.org/TR/webauthn-3/)
- [WebAuthn Level 2](https://www.w3.org/TR/webauthn-2/)
- [FIDO2 CTAP](https://fidoalliance.org/specs/fido-v2.0-ps-20190130/fido-client-to-authenticator-protocol-v2.0-ps-20190130.html)
- [FAPI 1.0 Advanced Profile](https://openid.net/specs/openid-financial-api-part-2-1_0.html)
- [FAPI 2.0 Security Profile](https://openid.net/specs/fapi-2_0-security-profile.html)
- [RFC 7636: PKCE](https://datatracker.ietf.org/doc/html/rfc7636)
- [RFC 8693: Token Exchange](https://datatracker.ietf.org/doc/html/rfc8693)
- [RFC 9126: PAR - Pushed Authorization Requests](https://datatracker.ietf.org/doc/html/rfc9126)
- [RFC 9449: DPoP - Demonstrating Proof-of-Possession](https://datatracker.ietf.org/doc/html/rfc9449)
- [RFC 9101: JARM - JWT-secured Authorization Response Mode](https://datatracker.ietf.org/doc/html/rfc9101)
- [WCAG 2.1](https://www.w3.org/TR/WCAG21/)

### 15.3 Internal Documentation
- [AGENTS.md - Root Project](../../AGENTS.md)
- [AGENTS.md - Authenc](../AGENTS.md)
- [AGENTS.md - Secreton](../../secreton/AGENTS.md)

---

**Document Version**: 2.0
**Last Updated**: 2026-02-19
**Authors**: SIMPelv2 Architecture Team
**Status**: Updated - Enhanced with WebAuthn/Passkeys (MANDATORY) and FAPI (OPTIONAL)
**Workflow**: Design-First
**Changes from v1.0**:
- Upgraded WebAuthn/Passkeys from optional to MANDATORY (PRIMARY authentication method)
- Added comprehensive WebAuthn requirements (REQ-WEBAUTHN-001 through REQ-WEBAUTHN-010)
- Added FAPI compliance requirements (REQ-FAPI-001 through REQ-FAPI-003) as OPTIONAL
- Added passkey management requirements to Portal (REQ-PORTAL-009)
- Added security requirements for phishing-resistant authentication (REQ-SEC-010 through REQ-SEC-013)
- Added WebAuthn compatibility requirements (REQ-NFR-010 through REQ-NFR-012)
- Added WebAuthn acceptance criteria (AC-PASSKEY-001 through AC-PASSKEY-010)
- Added FAPI acceptance criteria (AC-FAPI-001 through AC-FAPI-008)
- Added webauthn-rs crate dependency (DEP-DEP-001)
- Updated glossary with WebAuthn and FAPI terms
- Updated references with WebAuthn and FAPI standards
