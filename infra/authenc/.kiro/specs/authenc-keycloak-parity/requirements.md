# Authenc Enterprise Features - Requirements Document

## 1. Problem Statement

### 1.1 Current Situation
Authenc is SIMPelv2's custom Identity and Access Management (IAM) service with 90%+ feature parity with industry-standard solutions like Keycloak. While core authentication protocols (OAuth2, OIDC, SAML, WebAuthn) are fully implemented and security features exceed industry standards (Ed25519, Post-Quantum Crypto), several enterprise features and UI components are incomplete or missing.

### 1.2 Business Impact
Without these enterprise features, Authenc cannot:
- Be effectively managed in production environments (no Admin Console UI)
- Support user self-service operations (no User Account Console)
- Meet government compliance requirements (incomplete audit logging)
- Integrate seamlessly with existing government systems (MySIMKARI, SIMAN)
- Provide flexible authentication flows for different security contexts
- Support complex organizational structures (Satker hierarchy)

### 1.3 Goals
1. Complete Admin Console UI for production management
2. Implement User Account Console for self-service
3. Enhance event system for compliance and audit requirements
4. Add flexible authentication flows and required actions
5. Implement government-specific features (NIP validation, Satker hierarchy)
6. Achieve full enterprise readiness for Indonesian government deployment

### 1.4 Success Criteria
- 100% of critical features implemented within 3 months
- Production deployment in 5+ government agencies (Satker) within 6 months
- Zero Keycloak dependencies by Q3 2026
- Audit compliance achieved
- Support 1000+ concurrent users with <100ms response time

## 1. Feature Comparison Matrix

## 3. Feature Comparison Matrix (Reference)

> This section provides a detailed comparison with Keycloak for reference. It informed the user stories above.

### 3.1 Core Authentication ✅ COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| OAuth 2.0 | ✅ Full RFC 6749 | ✅ | ✅ PARITY |
| OpenID Connect | ✅ Complete | ✅ | ✅ PARITY |
| SAML 2.0 | ✅ SP Implementation | ✅ SP + IdP | ⚠️ PARTIAL |
| WebAuthn/FIDO2 | ✅ | ✅ | ✅ PARITY |
| Kerberos | ❌ | ✅ | ❌ GAP |
| X.509 Client Certificates | ❌ | ✅ | ❌ GAP |

### 1.2 User Management ✅ MOSTLY COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| User CRUD | ✅ | ✅ | ✅ PARITY |
| User Federation (LDAP/AD) | ✅ | ✅ | ✅ PARITY |
| User Storage SPI | ⚠️ Partial | ✅ | ⚠️ NEEDS ENHANCEMENT |
| User Profile | ✅ | ✅ | ✅ PARITY |
| User Attributes | ✅ | ✅ | ✅ PARITY |
| User Groups | ⚠️ Basic | ✅ Hierarchical | ⚠️ NEEDS ENHANCEMENT |
| User Impersonation | ❌ | ✅ | ❌ GAP |
| User Self-Service | ❌ No UI | ✅ Account Console | ❌ CRITICAL GAP |

### 1.3 Multi-Factor Authentication ✅ COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| TOTP | ✅ | ✅ | ✅ PARITY |
| WebAuthn | ✅ | ✅ | ✅ PARITY |
| SMS OTP | ✅ | ✅ | ✅ PARITY |
| Email OTP | ✅ | ✅ | ✅ PARITY |
| Backup Codes | ✅ | ✅ | ✅ PARITY |
| Conditional MFA | ✅ Risk-based | ✅ | ✅ PARITY |

### 1.4 Authorization & Access Control ✅ MOSTLY COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| RBAC | ✅ | ✅ | ✅ PARITY |
| ABAC | ✅ | ✅ | ✅ PARITY |
| UMA 2.0 | ✅ | ✅ | ✅ PARITY |
| Fine-Grained Permissions | ⚠️ Basic | ✅ Advanced | ⚠️ NEEDS ENHANCEMENT |
| Policy Enforcement | ✅ | ✅ | ✅ PARITY |
| Resource-Based Permissions | ✅ | ✅ | ✅ PARITY |
| Scope-Based Permissions | ✅ | ✅ | ✅ PARITY |
| JavaScript Policies | ❌ | ✅ | ❌ GAP |
| Drools Policies | ❌ | ✅ | ❌ GAP |

### 1.5 Federation & Identity Brokering ✅ COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| SAML Identity Brokering | ✅ | ✅ | ✅ PARITY |
| OIDC Identity Brokering | ✅ | ✅ | ✅ PARITY |
| Social Login (Google, GitHub, etc.) | ✅ | ✅ | ✅ PARITY |
| LDAP/AD Federation | ✅ | ✅ | ✅ PARITY |
| JIT User Provisioning | ✅ | ✅ | ✅ PARITY |
| Identity Provider Mappers | ✅ | ✅ | ✅ PARITY |
| First Login Flow | ⚠️ Basic | ✅ Customizable | ⚠️ NEEDS ENHANCEMENT |

### 1.6 Session Management ✅ COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| Session Storage | ✅ Redis | ✅ Infinispan | ✅ PARITY |
| SSO Sessions | ✅ | ✅ | ✅ PARITY |
| Session Timeout | ✅ | ✅ | ✅ PARITY |
| Remember Me | ✅ | ✅ | ✅ PARITY |
| Offline Sessions | ⚠️ Basic | ✅ | ⚠️ NEEDS ENHANCEMENT |
| Session Revocation | ✅ | ✅ | ✅ PARITY |

### 1.7 Client Management ✅ MOSTLY COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| OAuth2/OIDC Clients | ✅ | ✅ | ✅ PARITY |
| SAML Clients | ✅ | ✅ | ✅ PARITY |
| Dynamic Client Registration | ✅ RFC 7591/7592 | ✅ | ✅ PARITY |
| Client Policies | ✅ | ✅ | ✅ PARITY |
| Client Scopes | ✅ | ✅ | ✅ PARITY |
| Protocol Mappers | ✅ | ✅ | ✅ PARITY |
| Service Accounts | ✅ | ✅ | ✅ PARITY |
| Client Adapters | ❌ | ✅ Java, JS, etc. | ❌ GAP |

### 1.8 Security Features ✅ SUPERIOR TO KEYCLOAK

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| Ed25519 Signatures | ✅ | ❌ RSA only | ✅ SUPERIOR |
| Post-Quantum Crypto | ✅ ML-KEM, ML-DSA | ❌ | ✅ SUPERIOR |
| Brute Force Protection | ✅ Adaptive | ✅ | ✅ PARITY |
| Anomaly Detection | ✅ AI-based | ⚠️ Basic | ✅ SUPERIOR |
| Risk-Based Auth | ✅ | ⚠️ Limited | ✅ SUPERIOR |
| CAPTCHA | ✅ AI-resistant | ⚠️ reCAPTCHA | ✅ SUPERIOR |
| Audit Logging | ✅ Tamper-proof | ✅ | ✅ PARITY |
| FIPS 140-3 Ready | ✅ | ✅ | ✅ PARITY |

### 1.9 Admin Console & UI ❌ CRITICAL GAP

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| Admin Console | ⚠️ Partial (Leptos) | ✅ Full React UI | ❌ CRITICAL GAP |
| User Account Console | ❌ | ✅ | ❌ CRITICAL GAP |
| Realm Management UI | ⚠️ Partial | ✅ | ❌ CRITICAL GAP |
| Client Management UI | ⚠️ Partial | ✅ | ❌ CRITICAL GAP |
| User Management UI | ⚠️ Partial | ✅ | ❌ CRITICAL GAP |
| Role Management UI | ⚠️ Partial | ✅ | ❌ CRITICAL GAP |
| Theme Customization | ❌ | ✅ | ❌ GAP |
| Localization (i18n) | ⚠️ Basic | ✅ 40+ languages | ⚠️ NEEDS ENHANCEMENT |

### 1.10 Advanced Features ✅ MOSTLY COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| Token Exchange (RFC 8693) | ✅ | ✅ | ✅ PARITY |
| PAR (RFC 9126) | ✅ | ✅ | ✅ PARITY |
| OID4VC | ✅ | ⚠️ Experimental | ✅ SUPERIOR |
| Device Flow | ❌ | ✅ | ❌ GAP |
| CIBA (Client Initiated Backchannel Auth) | ❌ | ✅ | ❌ GAP |
| Step-Up Authentication | ⚠️ Basic | ✅ | ⚠️ NEEDS ENHANCEMENT |
| Consent Management | ✅ GDPR-compliant | ✅ | ✅ PARITY |

### 1.11 Deployment & Operations ✅ MOSTLY COMPLETE

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| Docker Support | ✅ | ✅ | ✅ PARITY |
| Kubernetes Operator | ⚠️ Partial | ✅ | ⚠️ NEEDS ENHANCEMENT |
| High Availability | ✅ Raft | ✅ Infinispan | ✅ PARITY |
| Clustering | ✅ | ✅ | ✅ PARITY |
| Health Checks | ✅ | ✅ | ✅ PARITY |
| Metrics (Prometheus) | ✅ | ✅ | ✅ PARITY |
| Distributed Tracing | ✅ OpenTelemetry | ✅ | ✅ PARITY |
| Backup & Restore | ⚠️ Manual | ✅ | ⚠️ NEEDS ENHANCEMENT |

### 1.12 Integration & Extensibility ⚠️ NEEDS WORK

| Feature | Authenc | Keycloak | Status |
|---------|---------|----------|--------|
| SPI (Service Provider Interface) | ✅ | ✅ | ✅ PARITY |
| Custom Authenticators | ✅ | ✅ | ✅ PARITY |
| Custom Protocol Mappers | ✅ | ✅ | ✅ PARITY |
| Event Listeners | ✅ Kafka | ✅ | ✅ PARITY |
| REST API | ✅ | ✅ | ✅ PARITY |
| Admin CLI | ❌ | ✅ | ❌ GAP |
| Client Adapters | ❌ | ✅ Multiple languages | ❌ GAP |
| Import/Export | ⚠️ Basic | ✅ | ⚠️ NEEDS ENHANCEMENT |

## 4. Functional Requirements

### 4.1 Admin Console UI (CRITICAL)

**FR-1: Realm Management**
- System shall provide web interface for realm CRUD operations
- System shall support realm enable/disable toggle
- System shall validate realm names (alphanumeric, no spaces)
- System shall prevent deletion of realms with active users

**FR-2: User Management**
- System shall provide user CRUD operations via web interface
- System shall support user search by username, email, NIP
- System shall support bulk user operations (import CSV, bulk enable/disable)
- System shall display user sessions and allow session termination
- System shall support password reset by admin

**FR-3: Client Management**
- System shall provide client CRUD operations
- System shall generate client secrets securely
- System shall validate redirect URIs
- System shall support multiple redirect URIs per client
- System shall support client secret rotation

**FR-4: Role Management**
- System shall support realm roles and client roles
- System shall support composite roles (roles containing other roles)
- System shall detect circular role dependencies
- System shall display role hierarchy visualization
- System shall support role assignment to users and groups

**FR-5: Audit Log Viewer**
- System shall display admin events and user events separately
- System shall support event filtering by type, user, date, IP
- System shall support event export to CSV/JSON
- System shall paginate event lists (50 per page)
- System shall highlight security-relevant events

**FR-6: System Monitoring**
- System shall display real-time metrics (active users, sessions, requests/sec)
- System shall display database connection pool status
- System shall display Redis cache statistics
- System shall display recent errors and warnings
- System shall auto-refresh dashboard every 30 seconds

### 4.2 User Account Console (CRITICAL)

**FR-7: Profile Management**
- System shall allow users to view their profile
- System shall allow users to update editable fields (name, phone)
- System shall prevent users from modifying system fields (username, NIP)
- System shall validate profile updates
- System shall send confirmation email after profile changes

**FR-8: Password Management**
- System shall require current password for password change
- System shall enforce password complexity requirements
- System shall terminate other sessions after password change
- System shall send notification email after password change
- System shall prevent password reuse (last 5 passwords)

**FR-9: MFA Device Management**
- System shall display list of registered MFA devices
- System shall support TOTP device registration via QR code
- System shall support WebAuthn device registration
- System shall allow device removal with password confirmation
- System shall prevent removal of all devices if MFA is required
- System shall generate and display backup codes

**FR-10: Session Management**
- System shall display active sessions with device and location
- System shall mark current session clearly
- System shall allow termination of individual sessions
- System shall allow termination of all other sessions
- System shall send notification when session is terminated remotely

**FR-11: Activity Log**
- System shall display user's login history
- System shall display password changes
- System shall display MFA device changes
- System shall display profile updates
- System shall include timestamp, IP address, device for each event
- System shall paginate activity log (50 events per page)

**FR-12: Data Export & Deletion**
- System shall allow users to export personal data in JSON format
- System shall send download link via email
- System shall expire export links after 7 days
- System shall allow users to request account deletion
- System shall implement 30-day grace period for deletion
- System shall allow cancellation of deletion request

### 4.3 Event System (CRITICAL)

**FR-13: Event Separation**
- System shall store admin events separately from user events
- System shall store system events separately
- Admin events shall include: operation type, resource type, resource path, representation, admin user ID
- User events shall include: event type, user ID, client ID, IP address, error details
- System events shall include: startup, shutdown, configuration changes

**FR-14: Event Retention**
- System shall support configurable retention policies per realm
- System shall support different retention for admin vs user events
- System shall automatically delete expired events
- System shall allow marking critical events for permanent retention
- System shall notify admin before bulk event deletion

**FR-15: Event Export**
- System shall export events to Kafka topics
- System shall export events to Elasticsearch
- System shall support event filtering in export configuration
- System shall retry failed exports with exponential backoff
- System shall display export status in admin console

### 4.4 Authentication Flows (HIGH)

**FR-16: Flow Configuration**
- System shall support custom authentication flow creation
- System shall support authentication execution types: username-password, OTP, WebAuthn, CAPTCHA
- System shall support execution requirements: REQUIRED, ALTERNATIVE, CONDITIONAL, DISABLED
- System shall allow execution reordering by priority
- System shall allow copying and customizing built-in flows
- System shall prevent deletion of built-in flows

**FR-17: Required Actions**
- System shall support required action types: verify email, update password, configure MFA, accept terms
- System shall allow enabling/disabling required actions
- System shall support setting default actions for new users
- System shall allow manual assignment of required actions to users
- System shall block system access until required actions are completed
- System shall support custom required actions via SPI

### 4.5 Role Management Enhancement (HIGH)

**FR-18: Composite Roles**
- System shall support composite role creation
- System shall allow adding/removing child roles
- System shall automatically grant child roles when composite role is assigned
- System shall detect and prevent circular role dependencies
- System shall visualize role hierarchy in admin console

**FR-19: Role Scoping**
- System shall distinguish between realm roles and client roles
- Realm roles shall be available to all clients
- Client roles shall be scoped to specific clients
- System shall support mapping realm roles to client roles
- JWT tokens shall include only roles relevant to the requesting client

### 4.6 User Storage Federation (HIGH)

**FR-20: Storage Provider Interface**
- System shall support custom user storage providers via SPI
- System shall support read-only and read-write providers
- System shall support user import vs federation modes
- System shall delegate credential validation to providers
- System shall support attribute mapping from external sources
- System shall cache federated user data

**FR-21: MySIMKARI Integration**
- System shall integrate with MySIMKARI as user storage provider
- System shall validate NIP against MySIMKARI
- System shall auto-populate user attributes from MySIMKARI
- System shall cache MySIMKARI responses for 5 minutes
- System shall operate in read-only mode for MySIMKARI users

### 4.7 Session Management Enhancement (MEDIUM)

**FR-22: Session Policies**
- System shall support maximum sessions per user limit
- System shall revoke oldest session when limit is reached
- System shall support configurable session idle timeout
- System shall support configurable session maximum lifespan
- System shall support "Remember Me" functionality
- System shall support offline sessions for mobile apps

**FR-23: Session Cleanup**
- System shall automatically delete expired sessions hourly
- System shall delete idle sessions beyond timeout
- System shall delete sessions beyond maximum lifespan
- System shall log cleanup statistics
- System shall allow manual cleanup trigger from admin console

### 4.8 User Registration (HIGH)

**FR-24: Self-Registration**
- System shall provide public registration page
- System shall validate NIP against MySIMKARI during registration
- System shall require email verification before login
- System shall implement CAPTCHA protection
- System shall require terms and conditions acceptance
- System shall auto-assign default roles after registration
- System shall notify admin of new registrations

**FR-25: Registration Customization**
- System shall support custom registration fields
- System shall support field types: text, email, select, checkbox
- System shall support required/optional field configuration
- System shall support validation rules (regex patterns)
- System shall support select field options configuration
- System shall store custom fields as user attributes

### 4.9 Email Templates (HIGH)

**FR-26: Template Management**
- System shall provide email templates for: verification, password reset, password changed, account updated, MFA enabled, login alert
- System shall support template variables: user name, NIP, Satker, links, IP address, timestamp
- System shall support multiple languages (Indonesian, English)
- System shall provide HTML and plain text versions
- System shall allow template preview in admin console
- System shall support test email sending
- System shall include government branding in templates

### 4.10 Import/Export (MEDIUM)

**FR-27: Realm Export**
- System shall export realm configuration to JSON
- System shall support including/excluding users in export
- System shall support including/excluding credentials in export
- System shall split large exports into multiple files
- System shall export clients, roles, groups, identity providers, flows, scopes

**FR-28: Realm Import**
- System shall import realm configuration from JSON
- System shall support overwrite or create new realm
- System shall validate import before applying changes
- System shall report import errors clearly
- System shall support import rollback on failure
- System shall display import progress

### 4.11 Government-Specific Features (HIGH)

**FR-29: NIP Validation**
- System shall validate NIP format (18 digits)
- System shall verify NIP against MySIMKARI via gRPC
- System shall auto-populate employee data from MySIMKARI
- System shall cache NIP validation results for 5 minutes
- System shall reject invalid NIPs with clear error messages

**FR-30: Satker Hierarchy**
- System shall store user's Satker assignment
- System shall load Satker hierarchy from database
- System shall enforce hierarchical access control
- System shall allow users to access resources in their Satker and child Satkers
- System shall support special permissions to override Satker restrictions
- System shall include Satker path in JWT tokens

**FR-31: Audit Reports**
- System shall generate audit reports for specified date ranges
- Reports shall include: total users, logins, failed logins, MFA adoption rate, admin actions, security incidents, compliance status
- System shall export reports to PDF format
- System shall include government branding in reports

## 5. Non-Functional Requirements

## 5. Non-Functional Requirements

### 5.1 Performance
- **NFR-1**: Admin Console shall load in <2 seconds on standard broadband connection
- **NFR-2**: API response time shall be <100ms at 95th percentile
- **NFR-3**: Database queries shall complete in <10ms at 95th percentile
- **NFR-4**: System shall support 1000+ concurrent users
- **NFR-5**: Memory usage shall be <500MB per instance
- **NFR-6**: JWT token generation shall complete in <5ms
- **NFR-7**: Session lookup shall complete in <1ms (Redis cache)

### 5.2 Security
- **NFR-8**: All UI components shall follow OWASP Top 10 guidelines
- **NFR-9**: XSS protection shall be enabled on all pages
- **NFR-10**: CSRF protection shall be enabled on all state-changing operations
- **NFR-11**: Content Security Policy (CSP) shall be configured
- **NFR-12**: Secure headers shall be set (HSTS, X-Frame-Options, X-Content-Type-Options)
- **NFR-13**: Input validation shall be performed on all user inputs
- **NFR-14**: Rate limiting shall be applied to all authentication endpoints
- **NFR-15**: Passwords shall be hashed with Argon2id
- **NFR-16**: JWT tokens shall be signed with Ed25519
- **NFR-17**: All communication shall use TLS 1.3
- **NFR-18**: Audit logs shall be tamper-proof (cryptographic signatures)

### 5.3 Accessibility
- **NFR-19**: UI shall comply with WCAG 2.1 Level AA
- **NFR-20**: All interactive elements shall be keyboard accessible
- **NFR-21**: UI shall be compatible with screen readers
- **NFR-22**: UI shall support high contrast mode
- **NFR-23**: All images shall have alt text
- **NFR-24**: Form fields shall have proper ARIA labels

### 5.4 Internationalization
- **NFR-25**: UI shall support Indonesian and English languages
- **NFR-26**: UI shall support RTL (Right-to-Left) languages for future expansion
- **NFR-27**: Date/time shall be localized based on user preference
- **NFR-28**: Number formatting shall follow locale conventions
- **NFR-29**: All user-facing text shall be externalized for translation

### 5.5 Browser Compatibility
- **NFR-30**: UI shall work on Chrome/Edge (latest 2 versions)
- **NFR-31**: UI shall work on Firefox (latest 2 versions)
- **NFR-32**: UI shall work on Safari (latest 2 versions)
- **NFR-33**: UI shall work on mobile browsers (iOS Safari, Chrome Mobile)
- **NFR-34**: UI shall be responsive (desktop, tablet, mobile)

### 5.6 Maintainability
- **NFR-35**: Code shall follow Rust best practices and idioms
- **NFR-36**: All public APIs shall be documented
- **NFR-37**: Unit test coverage shall be >80%
- **NFR-38**: Integration test coverage shall be >70%
- **NFR-39**: E2E test coverage shall exist for critical user flows
- **NFR-40**: Code shall pass `cargo clippy` without warnings
- **NFR-41**: Code shall be formatted with `cargo fmt`

### 5.7 Scalability
- **NFR-42**: System shall support horizontal scaling
- **NFR-43**: System shall be stateless (sessions in Redis)
- **NFR-44**: Database connection pooling shall be used
- **NFR-45**: Caching strategy shall be implemented for frequently accessed data
- **NFR-46**: System shall be load balancer ready

### 5.8 Reliability
- **NFR-47**: System shall target 99.9% uptime
- **NFR-48**: System shall gracefully degrade when dependencies are unavailable
- **NFR-49**: Circuit breaker pattern shall be implemented for external calls
- **NFR-50**: Retry logic with exponential backoff shall be used for transient failures
- **NFR-51**: Health checks and readiness probes shall be provided
- **NFR-52**: System shall recover automatically from crashes

### 5.9 Observability
- **NFR-53**: Prometheus metrics shall be exposed for all operations
- **NFR-54**: Structured logging (JSON) shall be used
- **NFR-55**: Distributed tracing (OpenTelemetry) shall be implemented
- **NFR-56**: Audit trail shall be maintained for all admin operations
- **NFR-57**: Performance monitoring shall be enabled
- **NFR-58**: Error tracking and alerting shall be configured

### 5.10 Deployment
- **NFR-59**: Docker images shall be <300MB
- **NFR-60**: Kubernetes manifests shall be provided
- **NFR-61**: Helm charts shall be available
- **NFR-62**: CI/CD pipeline shall be configured
- **NFR-63**: Blue-green deployment shall be supported
- **NFR-64**: Canary deployment shall be supported
- **NFR-65**: Database migrations shall be automated
- **NFR-66**: Startup time shall be <5 seconds

## 6. Constraints and Assumptions

### 6.1 Technical Constraints
- **C-1**: Must use Rust 1.90+ (Edition 2024)
- **C-2**: Must use PostgreSQL 13+ as database
- **C-3**: Must use Redis 6+ for caching and sessions
- **C-4**: Must use Leptos 0.8.x for UI components
- **C-5**: Must use Axum 0.8.x for REST API
- **C-6**: Must use Tonic 0.14.x for gRPC
- **C-7**: Must maintain backward compatibility with existing APIs
- **C-8**: Must not break existing integrations
- **C-9**: Must follow SIMPelv2 coding standards
- **C-10**: Must use existing infrastructure (PostgreSQL, Redis, Kubernetes)

### 6.2 Integration Constraints
- **C-11**: Must integrate with Secreton for secret management
- **C-12**: Must integrate with MySIMKARI for user provisioning
- **C-13**: Must integrate with Kafka for event streaming
- **C-14**: Must integrate with Elasticsearch for audit logs
- **C-15**: Must integrate with Prometheus for metrics
- **C-16**: Must integrate with Grafana for dashboards

### 6.3 Operational Constraints
- **C-17**: Must support deployment on MicroK8s cluster
- **C-18**: Must support deployment with Istio service mesh
- **C-19**: Must support MetalLB for load balancing
- **C-20**: Must support Longhorn for persistent storage
- **C-21**: Must operate within government network restrictions
- **C-22**: Must comply with data residency requirements (data in Indonesia)

### 6.4 Assumptions
- **A-1**: PostgreSQL database is available and properly configured
- **A-2**: Redis cache is available and properly configured
- **A-3**: MySIMKARI gRPC service is available for NIP validation
- **A-4**: Secreton gRPC service is available for secret management
- **A-5**: SMTP server is available for sending emails
- **A-6**: Kafka cluster is available for event streaming (optional)
- **A-7**: Elasticsearch cluster is available for audit logs (optional)
- **A-8**: Users have modern web browsers (Chrome, Firefox, Safari)
- **A-9**: Network latency between services is <10ms
- **A-10**: Database backup and recovery procedures are in place

## 7. Dependencies

### 7.1 Internal Dependencies
- **Secreton**: For storing JWT signing keys, client secrets, and sensitive configuration
- **MySIMKARI Integration Service**: For NIP validation and employee data
- **PostgreSQL Database**: For persistent data storage
- **Redis Cache**: For session storage and caching
- **Kafka**: For event streaming (optional)
- **Elasticsearch**: For audit log storage and search (optional)

### 7.2 External Dependencies
- **Rust Toolchain**: 1.90+ with Edition 2024 support
- **Leptos**: 0.8.x for WASM-based UI
- **Axum**: 0.8.x for HTTP server
- **Tonic**: 0.14.x for gRPC
- **tokio-postgres**: For database access
- **deadpool-postgres**: For connection pooling
- **redis**: For cache access
- **jsonwebtoken**: For JWT operations
- **argon2**: For password hashing
- **totp-rs**: For TOTP MFA
- **webauthn-rs**: For WebAuthn/FIDO2

### 7.3 Infrastructure Dependencies
- **Kubernetes**: For container orchestration
- **Istio**: For service mesh and mTLS
- **MetalLB**: For load balancing
- **Longhorn**: For persistent storage
- **Prometheus**: For metrics collection
- **Grafana**: For metrics visualization

## 8. Success Metrics

### 8.1 Feature Completeness
- **M-1**: 100% of critical features (Admin Console, User Console, Event System) implemented by Q1 2026
- **M-2**: 90%+ of high-priority features implemented by Q2 2026
- **M-3**: 70%+ of medium-priority features implemented by Q3 2026

### 8.2 Quality Metrics
- **M-4**: Test coverage >80%
- **M-5**: Zero critical security vulnerabilities
- **M-6**: <5 high-priority bugs in production
- **M-7**: Performance targets met (p95 <100ms)
- **M-8**: All accessibility requirements met (WCAG 2.1 AA)

### 8.3 User Satisfaction
- **M-9**: Admin console usability score >4/5
- **M-10**: User account console usability score >4/5
- **M-11**: Documentation completeness score >4/5
- **M-12**: Support ticket reduction by 30%

### 8.4 Adoption Metrics
- **M-13**: 100% of SIMPelv2 services using Authenc by Q2 2026
- **M-14**: Zero Keycloak dependencies by Q3 2026
- **M-15**: Admin CLI usage >50% of admin operations by Q3 2026
- **M-16**: Production deployment in 5+ Satker by Q2 2026
- **M-17**: 1000+ active users by Q4 2026

### 8.5 Operational Metrics
- **M-18**: 99.9% uptime achieved
- **M-19**: Mean time to recovery (MTTR) <15 minutes
- **M-20**: Zero data loss incidents
- **M-21**: Audit compliance achieved by Q3 2026

## 9. Risk Assessment

### 9.1 Technical Risks

| Risk ID | Risk | Probability | Impact | Mitigation |
|---------|------|-------------|--------|------------|
| R-1 | Leptos UI complexity delays development | Medium | High | Prototype early, use proven patterns, allocate experienced developers |
| R-2 | Performance degradation with scale | Low | High | Load testing, profiling, optimization, horizontal scaling |
| R-3 | Security vulnerabilities discovered | Low | Critical | Security audits, penetration testing, bug bounty program |
| R-4 | Integration issues with MySIMKARI | Medium | Medium | Comprehensive integration tests, fallback mechanisms |
| R-5 | Database migration issues | Low | High | Backup strategy, rollback plan, test migrations in staging |
| R-6 | Redis cache failures | Low | Medium | Circuit breaker, graceful degradation, cache warming |

### 9.2 Project Risks

| Risk ID | Risk | Probability | Impact | Mitigation |
|---------|------|-------------|--------|------------|
| R-7 | Timeline delays | Medium | Medium | Phased approach, MVP first, parallel development |
| R-8 | Resource constraints | Medium | High | Prioritize critical features, hire contractors if needed |
| R-9 | Scope creep | High | Medium | Strict change control process, prioritization framework |
| R-10 | Dependency updates breaking changes | Low | Low | Version pinning, comprehensive testing, gradual updates |
| R-11 | Key personnel leaving | Low | High | Knowledge sharing, documentation, pair programming |

### 9.3 Operational Risks

| Risk ID | Risk | Probability | Impact | Mitigation |
|---------|------|-------------|--------|------------|
| R-12 | Production downtime | Low | Critical | Blue-green deployment, rollback procedures, monitoring |
| R-13 | Data loss | Very Low | Critical | Automated backups, replication, disaster recovery plan |
| R-14 | Performance issues in production | Medium | High | Monitoring, auto-scaling, performance testing |
| R-15 | Security breach | Low | Critical | Security hardening, monitoring, incident response plan |
| R-16 | Compliance violations | Low | High | Regular audits, compliance checklists, training |

## 10. Implementation Roadmap

### Phase 1: Critical Features (Q1 2026 - 3 months)
**Goal**: Make Authenc production-ready with full management capabilities

**Priority 1: Admin Console UI** (8 weeks)
- Week 1-2: Architecture, design, and Leptos setup
- Week 3-4: Realm and user management
- Week 5-6: Client and role management
- Week 7-8: Security settings, audit logs, and monitoring dashboard

**Priority 2: Event System** (3 weeks)
- Week 1: Database schema for separate admin/user events
- Week 2: Event storage and retrieval implementation
- Week 3: Event export to Kafka/Elasticsearch

**Priority 3: Required Actions** (2 weeks)
- Week 1: Required actions framework and database schema
- Week 2: UI integration and testing

**Priority 4: User Account Console** (4 weeks)
- Week 1-2: Profile and password management
- Week 3-4: MFA device management and session management

### Phase 2: High-Priority Features (Q2 2026 - 3 months)
**Goal**: Improve usability and integration

**Priority 5: Authentication Flows** (4 weeks)
- Week 1-2: Flow configuration framework
- Week 3-4: UI for flow management and testing

**Priority 6: Composite Roles** (2 weeks)
- Week 1: Database schema and core logic
- Week 2: UI integration and hierarchy visualization

**Priority 7: User Storage Federation** (3 weeks)
- Week 1: Storage provider SPI
- Week 2: MySIMKARI integration
- Week 3: Testing and caching

**Priority 8: User Self-Registration** (3 weeks)
- Week 1: Registration page and NIP validation
- Week 2: Email verification and CAPTCHA
- Week 3: Custom fields and testing

**Priority 9: Email Templates** (2 weeks)
- Week 1: Template engine and default templates
- Week 2: Admin UI for template customization

### Phase 3: Medium-Priority Features (Q3 2026 - 3 months)
**Goal**: Operational excellence and flexibility

**Priority 10: Client Scopes & Protocol Mappers** (3 weeks)
**Priority 11: Session Management Enhancement** (2 weeks)
**Priority 12: Import/Export System** (3 weeks)
**Priority 13: Theme System** (2 weeks)
**Priority 14: Admin CLI** (3 weeks)

### Phase 4: Low-Priority Features (Q4 2026 - 3 months)
**Goal**: Complete feature parity and polish

**Priority 15: Client Adapters** (4 weeks)
**Priority 16: Device Flow** (2 weeks)
**Priority 17: User Impersonation** (1 week)
**Priority 18: Additional government-specific features** (5 weeks)

## 11. Appendix

### 11.1 Glossary
- **Authenc**: SIMPelv2's custom Identity and Access Management service
- **Audit Report**: Comprehensive system audit and compliance reporting
- **MySIMKARI**: Government employee management system
- **NIP**: Nomor Induk Pegawai (Employee Identification Number)
- **Satker**: Satuan Kerja (Organizational Unit)
- **SIMAN**: Sistem Informasi Manajemen (Management Information System)
- **MonSAKTI**: Monitoring SAKTI (Government financial system monitoring)
- **SPBE**: Sistem Pemerintahan Berbasis Elektronik (Electronic-Based Government System)

### 11.2 References
- [Keycloak Documentation](https://www.keycloak.org/documentation)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [SAML 2.0](http://docs.oasis-open.org/security/saml/Post2.0/sstc-saml-tech-overview-2.0.html)
- [WebAuthn](https://www.w3.org/TR/webauthn/)
- [UMA 2.0](https://docs.kantarainitiative.org/uma/wg/rec-oauth-uma-grant-2.0.html)
- [WCAG 2.1](https://www.w3.org/TR/WCAG21/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

### 11.3 Document History
| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-12 | AI Analysis (Kiro) | Initial feature parity analysis |
| 2.0 | 2026-02-12 | AI Analysis (Kiro) | Restructured to standard requirements format with user stories |

---

**Document Version**: 2.0
**Last Updated**: 2026-02-12
**Status**: READY FOR DESIGN PHASE
**Next Step**: Create design.md based on these requirements

#### 2.1.1 Admin Console UI
**Status**: ⚠️ Partially implemented (feature-gated)
**Impact**: CRITICAL - Without a full admin console, Authenc cannot be managed effectively in production
**Location**: `infra/authenc/src/ui/admin/`

**Requirements**:
- Complete Leptos-based admin console implementation
- Realm management interface
- User management interface (CRUD, search, bulk operations)
- Client management interface
- Role and permission management
- Identity provider configuration
- Authentication flow configuration
- Security settings and policies
- Audit log viewer
- System monitoring dashboard

#### 2.1.2 User Account Management Console
**Status**: ❌ Not implemented
**Impact**: CRITICAL - Users cannot self-manage their accounts
**Location**: `infra/authenc/src/ui/account/`

**Requirements**:
- User profile management
- Password change interface
- MFA device management
- Session management (view active sessions, logout)
- Consent management
- Linked accounts (social login)
- Account activity log
- Personal data export (GDPR)
- Account deletion request

#### 2.1.3 Login & Registration Pages
**Status**: ⚠️ Basic implementation exists
**Impact**: HIGH - Cannot customize login pages for branding
**Location**: `infra/authenc/src/ui/auth/`

**Requirements**:
- Login page with customizable branding
- Registration page with custom fields
- Forgot password flow
- MFA challenge pages (TOTP, WebAuthn, SMS, Email)
- Email verification page
- Terms and conditions acceptance
- Theme engine for customization
- CSS/JavaScript customization
- Logo and branding support
- Multi-language support
- Theme inheritance
- Preview mode

### 2.2 MEDIUM PRIORITY (Should Implement)

#### 2.2.1 Admin CLI Tool
**Status**: ❌ Not implemented
**Impact**: MEDIUM - Automation and scripting difficult

**Requirements**:
- Command-line interface for all admin operations
- Batch operations support
- Configuration import/export
- Scripting support
- CI/CD integration

#### 2.2.2 Client Adapters
**Status**: ❌ Not implemented
**Impact**: MEDIUM - Integration requires manual implementation

**Requirements**:
- Rust client adapter (for Rust applications)
- JavaScript/TypeScript adapter (for frontend apps)
- Python adapter (for Python services)
- Documentation and examples

#### 2.2.3 Enhanced User Groups
**Status**: ⚠️ Basic implementation
**Impact**: MEDIUM - Complex organizational structures difficult to model

**Requirements**:
- Hierarchical group structure
- Group inheritance
- Group attributes
- Group-based role mapping
- Subgroup management

#### 2.2.4 User Impersonation
**Status**: ❌ Not implemented
**Impact**: MEDIUM - Support and debugging difficult

**Requirements**:
- Admin can impersonate users
- Audit trail for impersonation
- Permission checks
- Session isolation
- Impersonation banner

#### 2.2.5 Device Flow (RFC 8628)
**Status**: ❌ Not implemented
**Impact**: MEDIUM - IoT and device authentication limited

**Requirements**:
- Device authorization endpoint
- User code verification
- Device polling
- QR code support

### 2.3 LOW PRIORITY (Nice to Have)

#### 2.3.1 Kerberos Support
**Status**: ❌ Not implemented
**Impact**: LOW - Only needed for legacy Windows environments

**Requirements**:
- SPNEGO/Kerberos authentication
- Active Directory integration
- Ticket validation

#### 2.3.2 X.509 Client Certificates
**Status**: ❌ Not implemented
**Impact**: LOW - Alternative authentication methods available

**Requirements**:
- Client certificate authentication
- Certificate validation
- Certificate-to-user mapping

#### 2.3.3 JavaScript/Drools Policy Engine
**Status**: ❌ Not implemented
**Impact**: LOW - Current policy engine sufficient for most use cases

**Requirements**:
- JavaScript policy evaluation
- Drools rule engine integration
- Policy testing framework

#### 2.3.4 CIBA (Client Initiated Backchannel Authentication)
**Status**: ❌ Not implemented
**Impact**: LOW - Specialized use case

**Requirements**:
- Backchannel authentication endpoint
- Push notification support
- Polling support

## 3. Authenc Advantages Over Keycloak

### 3.1 Modern Cryptography ✅
- **Ed25519 Signatures**: Timing-attack resistant, faster than RSA
- **Post-Quantum Cryptography**: ML-KEM, ML-DSA, Falcon support
- **ChaCha20-Poly1305**: Modern AEAD cipher
- **Blake3 Hashing**: Faster than SHA-256

### 3.2 Advanced Security ✅
- **AI-Based Anomaly Detection**: Machine learning for threat detection
- **Risk-Based Authentication**: Dynamic risk scoring
- **AI-Resistant CAPTCHA**: Behavioral analysis
- **Tamper-Proof Audit Logs**: Cryptographic signatures

### 3.3 Performance ✅
- **Rust Implementation**: Memory-safe, zero-cost abstractions
- **Async-First Architecture**: Non-blocking I/O
- **Efficient Connection Pooling**: Better resource utilization
- **Redis Caching**: High-performance caching

### 3.4 Cloud-Native ✅
- **Lightweight**: Smaller Docker images (~200MB vs ~500MB)
- **Fast Startup**: Seconds vs minutes
- **Low Memory Footprint**: ~100MB vs ~500MB
- **Kubernetes-Ready**: Native Kubernetes integration

## 4. Recommended Implementation Roadmap

### Phase 1: Critical UI Components (3-4 months)
**Priority**: CRITICAL
**Goal**: Make Authenc production-ready with full management capabilities

#### 4.1 Admin Console (8 weeks)
- Week 1-2: Architecture and design
- Week 3-4: Realm and user management
- Week 5-6: Client and role management
- Week 7-8: Security settings and monitoring

#### 4.2 User Account Console (4 weeks)
- Week 1-2: Profile and password management
- Week 3-4: MFA and session management

#### 4.3 Theme System (2 weeks)
- Week 1: Theme engine implementation
- Week 2: Default themes and documentation

### Phase 2: Enhanced Features (2-3 months)
**Priority**: HIGH
**Goal**: Improve usability and integration

#### 4.4 Admin CLI (3 weeks)
- Week 1: Core CLI framework
- Week 2: Admin operations
- Week 3: Import/export and scripting

#### 4.5 Client Adapters (4 weeks)
- Week 1: Rust adapter
- Week 2: JavaScript/TypeScript adapter
- Week 3: Python adapter
- Week 4: Documentation and examples

#### 4.6 Enhanced User Groups (2 weeks)
- Week 1: Hierarchical structure
- Week 2: Group inheritance and attributes

#### 4.7 User Impersonation (1 week)
- Implementation and audit trail

### Phase 3: Advanced Features (2-3 months)
**Priority**: MEDIUM
**Goal**: Feature completeness

#### 4.8 Device Flow (2 weeks)
- OAuth2 Device Authorization Grant

#### 4.9 Enhanced Kubernetes Operator (3 weeks)
- Complete CRD implementation
- Automated deployment and scaling

#### 4.10 Backup & Restore (2 weeks)
- Automated backup system
- Point-in-time recovery

### Phase 4: Optional Features (1-2 months)
**Priority**: LOW
**Goal**: Complete feature parity

#### 4.11 Kerberos Support (2 weeks)
- SPNEGO/Kerberos authentication

#### 4.12 X.509 Client Certificates (1 week)
- Client certificate authentication

#### 4.13 JavaScript/Drools Policies (2 weeks)
- Advanced policy engine

#### 4.14 CIBA (2 weeks)
- Backchannel authentication

## 5. Acceptance Criteria

### 5.1 Admin Console
- [ ] AC-1.1: Admin can manage realms (create, update, delete, configure)
- [ ] AC-1.2: Admin can manage users (CRUD, search, bulk operations)
- [ ] AC-1.3: Admin can manage clients (CRUD, configure protocols)
- [ ] AC-1.4: Admin can manage roles and permissions
- [ ] AC-1.5: Admin can configure identity providers
- [ ] AC-1.6: Admin can view audit logs with filtering
- [ ] AC-1.7: Admin can monitor system health and metrics
- [ ] AC-1.8: UI is responsive and accessible (WCAG 2.1 AA)
- [ ] AC-1.9: UI supports multiple languages (i18n)
- [ ] AC-1.10: All operations have proper error handling

### 5.2 User Account Console
- [ ] AC-2.1: User can view and edit profile
- [ ] AC-2.2: User can change password
- [ ] AC-2.3: User can manage MFA devices (add, remove, verify)
- [ ] AC-2.4: User can view active sessions and logout
- [ ] AC-2.5: User can manage consents
- [ ] AC-2.6: User can link/unlink social accounts
- [ ] AC-2.7: User can view account activity log
- [ ] AC-2.8: User can export personal data (GDPR)
- [ ] AC-2.9: User can request account deletion
- [ ] AC-2.10: UI is responsive and accessible

### 5.3 Theme System
- [ ] AC-3.1: Themes can be created and customized
- [ ] AC-3.2: Themes support custom CSS and JavaScript
- [ ] AC-3.3: Themes support logo and branding
- [ ] AC-3.4: Themes support multiple languages
- [ ] AC-3.5: Theme inheritance works correctly
- [ ] AC-3.6: Theme preview mode available
- [ ] AC-3.7: Default themes provided (light, dark)

### 5.4 Admin CLI
- [ ] AC-4.1: CLI can perform all admin operations
- [ ] AC-4.2: CLI supports batch operations
- [ ] AC-4.3: CLI can import/export configurations
- [ ] AC-4.4: CLI supports scripting and automation
- [ ] AC-4.5: CLI has comprehensive help documentation
- [ ] AC-4.6: CLI integrates with CI/CD pipelines

### 5.5 Client Adapters
- [ ] AC-5.1: Rust adapter available and documented
- [ ] AC-5.2: JavaScript/TypeScript adapter available
- [ ] AC-5.3: Python adapter available
- [ ] AC-5.4: Adapters handle token refresh automatically
- [ ] AC-5.5: Adapters support all authentication flows
- [ ] AC-5.6: Examples and tutorials provided

### 5.6 Enhanced User Groups
- [ ] AC-6.1: Groups can be nested hierarchically
- [ ] AC-6.2: Group inheritance works correctly
- [ ] AC-6.3: Groups can have custom attributes
- [ ] AC-6.4: Group-based role mapping works
- [ ] AC-6.5: Subgroup management available

### 5.7 User Impersonation
- [ ] AC-7.1: Admin can impersonate users
- [ ] AC-7.2: Impersonation is audited
- [ ] AC-7.3: Permissions are checked before impersonation
- [ ] AC-7.4: Impersonation session is isolated
- [ ] AC-7.5: Impersonation banner is displayed

### 5.8 Device Flow
- [ ] AC-8.1: Device authorization endpoint works
- [ ] AC-8.2: User code verification works
- [ ] AC-8.3: Device polling works correctly
- [ ] AC-8.4: QR code support available
- [ ] AC-8.5: Flow complies with RFC 8628

## 6. Technical Requirements

### 6.1 Performance
- Admin Console loads in <2 seconds
- API response time <100ms (p95)
- Database queries <10ms (p95)
- Support 1000+ concurrent users
- Memory usage <500MB per instance

### 6.2 Security
- All UI components follow OWASP Top 10 guidelines
- XSS protection enabled
- CSRF protection enabled
- Content Security Policy (CSP) configured
- Secure headers (HSTS, X-Frame-Options, etc.)
- Input validation on all forms
- Rate limiting on all endpoints

### 6.3 Accessibility
- WCAG 2.1 Level AA compliance
- Keyboard navigation support
- Screen reader compatibility
- High contrast mode
- Proper ARIA labels

### 6.4 Internationalization
- Support for multiple languages
- RTL (Right-to-Left) language support
- Date/time localization
- Number formatting
- Currency formatting

### 6.5 Browser Compatibility
- Chrome/Edge (latest 2 versions)
- Firefox (latest 2 versions)
- Safari (latest 2 versions)
- Mobile browsers (iOS Safari, Chrome Mobile)

## 7. Non-Functional Requirements

### 7.1 Maintainability
- Code follows Rust best practices
- Comprehensive documentation
- Unit test coverage >80%
- Integration test coverage >70%
- E2E test coverage for critical flows

### 7.2 Scalability
- Horizontal scaling support
- Stateless design (session in Redis)
- Database connection pooling
- Caching strategy
- Load balancing ready

### 7.3 Reliability
- 99.9% uptime target
- Graceful degradation
- Circuit breaker pattern
- Retry logic with exponential backoff
- Health checks and readiness probes

### 7.4 Observability
- Prometheus metrics for all operations
- Structured logging (JSON)
- Distributed tracing (OpenTelemetry)
- Audit trail for all admin operations
- Performance monitoring
- Error tracking and alerting

### 7.5 Deployment
- Docker images <300MB
- Kubernetes manifests provided
- Helm charts available
- CI/CD pipeline configured
- Blue-green deployment support
- Canary deployment support

## 8. Dependencies and Constraints

### 8.1 Technical Dependencies
- Rust 1.90+ (Edition 2024)
- PostgreSQL 13+
- Redis 6+
- Leptos 0.8.x (for UI)
- Axum 0.8.x (for API)
- Tonic 0.14.x (for gRPC)

### 8.2 Integration Points
- Secreton (for secret management)
- MySIMKARI (for user provisioning)
- Kafka (for event streaming)
- Elasticsearch (for audit logs)
- Prometheus (for metrics)
- Grafana (for dashboards)

### 8.3 Constraints
- Must maintain backward compatibility with existing APIs
- Must not break existing integrations
- Must follow SIMPelv2 coding standards
- Must use existing infrastructure (PostgreSQL, Redis)
- Must integrate with existing monitoring

## 9. Success Metrics

### 9.1 Feature Completeness
- 100% of critical features implemented
- 90%+ of high-priority features implemented
- 70%+ of medium-priority features implemented

### 9.2 Quality Metrics
- Test coverage >80%
- Zero critical security vulnerabilities
- <5 high-priority bugs in production
- Performance targets met (p95 <100ms)

### 9.3 User Satisfaction
- Admin console usability score >4/5
- User account console usability score >4/5
- Documentation completeness score >4/5
- Support ticket reduction by 30%

### 9.4 Adoption Metrics
- 100% of SIMPelv2 services using Authenc
- Zero Keycloak dependencies
- Admin CLI usage >50% of admin operations
- Theme customization by >3 realms

## 10. Risk Assessment

### 10.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Leptos UI complexity | Medium | High | Prototype early, use proven patterns |
| Performance degradation | Low | High | Load testing, profiling, optimization |
| Security vulnerabilities | Low | Critical | Security audits, penetration testing |
| Integration issues | Medium | Medium | Comprehensive integration tests |
| Database migration issues | Low | High | Backup strategy, rollback plan |

### 10.2 Project Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Timeline delays | Medium | Medium | Phased approach, MVP first |
| Resource constraints | Medium | High | Prioritize critical features |
| Scope creep | High | Medium | Strict change control process |
| Dependency updates | Low | Low | Version pinning, testing |

### 10.3 Operational Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Production downtime | Low | Critical | Blue-green deployment, rollback |
| Data loss | Very Low | Critical | Automated backups, replication |
| Performance issues | Medium | High | Monitoring, auto-scaling |
| Security breach | Low | Critical | Security hardening, monitoring |

## 11. Conclusion

### 11.1 Summary
Authenc is a **production-ready, feature-rich IAM service** with 90%+ feature parity with Keycloak. The main gaps are in UI components (Admin Console, User Account Console) and some enterprise features (themes, client adapters).

### 11.2 Key Strengths
- ✅ Modern cryptography (Ed25519, Post-Quantum)
- ✅ Advanced security (AI-based anomaly detection, risk-based auth)
- ✅ High performance (Rust, async-first)
- ✅ Cloud-native (lightweight, fast startup)
- ✅ Complete protocol support (OAuth2, OIDC, SAML, WebAuthn)
- ✅ Comprehensive audit logging

### 11.3 Critical Gaps to Address
1. **Admin Console UI** - CRITICAL for production management
2. **User Account Console** - CRITICAL for user self-service
3. **Theme System** - HIGH for branding and customization
4. **Admin CLI** - HIGH for automation
5. **Client Adapters** - MEDIUM for easier integration

### 11.4 Recommendations

#### Immediate Actions (Next 3 months)
1. **Complete Admin Console** - Focus on core management features
2. **Implement User Account Console** - Enable user self-service
3. **Build Theme System** - Allow branding customization

#### Short-term Actions (3-6 months)
4. **Develop Admin CLI** - Enable automation and scripting
5. **Create Client Adapters** - Simplify integration
6. **Enhance User Groups** - Support complex hierarchies

#### Long-term Actions (6-12 months)
7. **Implement Device Flow** - Support IoT devices
8. **Complete Kubernetes Operator** - Improve deployment
9. **Add Optional Features** - Kerberos, X.509, CIBA

### 11.5 Strategic Decision
**Recommendation**: Continue with Authenc development rather than migrating to Keycloak

**Rationale**:
- Authenc already has 90%+ feature parity
- Superior security features (Ed25519, Post-Quantum)
- Better performance (Rust vs Java)
- Lower resource usage (200MB vs 500MB)
- Faster startup (seconds vs minutes)
- Custom features for Indonesian government (Satker hierarchy, MySIMKARI integration)
- Investment in Rust ecosystem aligns with SIMPelv2 architecture

**Estimated Effort**: 6-9 months to achieve full enterprise readiness
**Estimated Cost**: 3-4 full-time developers
**ROI**: High - custom features, better performance, lower operational costs

## 12. Appendix

### 12.1 Keycloak Features Reference
Based on Keycloak 26.0.0 (latest stable as of 2024):
- OAuth 2.0 / OpenID Connect
- SAML 2.0
- User Federation (LDAP, Active Directory, Custom)
- Identity Brokering (SAML, OIDC, Social)
- User Management
- Admin Console
- Account Console
- Fine-grained Authorization
- Standard Protocols
- Centralized Management
- Adapters (Java, JavaScript, Node.js, etc.)
- High Availability
- Clustering
- Themes
- Extensibility (SPI)

### 12.2 Authenc Current Implementation Status
**Version**: 0.1.0
**Status**: Production-ready (core features)
**Lines of Code**: ~50,000+ (Rust)
**Test Coverage**: >80%
**Database Migrations**: 44 migrations
**gRPC Services**: 20+ RPC methods
**REST Endpoints**: 40+ endpoints
**Services**: 80+ service modules

### 12.3 Technology Stack Comparison

| Component | Authenc | Keycloak |
|-----------|---------|----------|
| Language | Rust | Java |
| Framework | Axum | Quarkus |
| Database | PostgreSQL | PostgreSQL/MySQL/MariaDB |
| Cache | Redis | Infinispan |
| Clustering | Raft | Infinispan |
| Metrics | Prometheus | Prometheus |
| Tracing | OpenTelemetry | OpenTelemetry |
| Container Size | ~200MB | ~500MB |
| Memory Usage | ~100MB | ~500MB |
| Startup Time | <5s | ~30s |

### 12.4 References
- [Keycloak Documentation](https://www.keycloak.org/documentation)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [SAML 2.0](http://docs.oasis-open.org/security/saml/Post2.0/sstc-saml-tech-overview-2.0.html)
- [WebAuthn](https://www.w3.org/TR/webauthn/)
- [UMA 2.0](https://docs.kantarainitiative.org/uma/wg/rec-oauth-uma-grant-2.0.html)

---

**Document Version**: 1.0
**Last Updated**: 2026-02-12
**Author**: AI Analysis (Kiro)
**Status**: APPROVED FOR IMPLEMENTATION


---

## 13. Analisis Mendalam: Fitur Keycloak yang Belum Diimplementasikan di Authenc

### 13.1 Event System & Audit Logging (CRITICAL untuk Compliance)

#### 13.1.1 Admin Events vs User Events
**Status di Keycloak**: ✅ Terpisah dan lengkap
**Status di Authenc**: ⚠️ Digabung dalam satu audit log

**Keycloak Implementation**:
- **Admin Events**: Semua aksi administrator (create user, update role, change config)
- **User Events**: Semua aksi user (login, logout, register, update profile, MFA)
- Event filtering berdasarkan tipe, user, client, IP address
- Event expiration policy (auto-delete setelah X hari)
- Event export ke external systems (SIEM, Elasticsearch, Kafka)

**Gap di Authenc**:
```rust
// Authenc saat ini: Satu tabel audit_logs untuk semua event
// Keycloak: Dua tabel terpisah - admin_events & user_events

// Yang perlu ditambahkan:
pub enum EventCategory {
    AdminEvent,    // Admin operations
    UserEvent,     // User operations
    SystemEvent,   // System operations
}

pub struct AdminEvent {
    pub operation_type: AdminOperationType, // CREATE, UPDATE, DELETE, ACTION
    pub resource_type: String,              // USER, ROLE, CLIENT, REALM
    pub resource_path: String,              // /realms/master/users/123
    pub representation: Option<String>,     // JSON of changed object
    pub admin_user_id: Uuid,
    pub realm_id: Uuid,
}

pub struct UserEvent {
    pub event_type: UserEventType,  // LOGIN, LOGOUT, REGISTER, UPDATE_PROFILE
    pub user_id: Option<Uuid>,
    pub client_id: Option<String>,
    pub ip_address: String,
    pub error: Option<String>,
    pub details: HashMap<String, String>,
}
```

**Acceptance Criteria**:
- [ ] AC-13.1.1: Admin events dan user events terpisah dalam database
- [ ] AC-13.1.2: Admin dapat filter events berdasarkan kategori
- [ ] AC-13.1.3: Event expiration policy dapat dikonfigurasi per realm
- [ ] AC-13.1.4: Event dapat di-export ke Kafka/Elasticsearch
- [ ] AC-13.1.5: Event viewer di Admin Console dengan pagination
- [ ] AC-13.1.6: Event search dengan multiple filters (user, date range, type)

### 13.2 Authentication Flows & Required Actions (CRITICAL untuk Flexibility)

#### 13.2.1 Customizable Authentication Flows
**Status di Keycloak**: ✅ Fully customizable via UI
**Status di Authenc**: ⚠️ Hardcoded flows

**Keycloak Implementation**:
- **Browser Flow**: Login dengan username/password, MFA, remember me
- **Direct Grant Flow**: Direct access grant (username/password API)
- **Registration Flow**: User self-registration dengan custom fields
- **Reset Credentials Flow**: Forgot password flow
- **Client Authentication Flow**: Client credentials flow
- **First Broker Login Flow**: Flow saat pertama kali login via external IdP

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct AuthenticationFlow {
    pub id: Uuid,
    pub alias: String,              // "browser", "registration", "reset-credentials"
    pub description: String,
    pub provider_id: String,        // "basic-flow", "client-flow"
    pub top_level: bool,
    pub built_in: bool,
    pub executions: Vec<AuthenticationExecution>,
}

pub struct AuthenticationExecution {
    pub id: Uuid,
    pub authenticator: String,      // "username-password-form", "otp-form"
    pub requirement: ExecutionRequirement, // REQUIRED, ALTERNATIVE, DISABLED, CONDITIONAL
    pub priority: i32,
    pub authenticator_flow: bool,
    pub flow_alias: Option<String>,
}

pub enum ExecutionRequirement {
    Required,      // Must execute successfully
    Alternative,   // One of alternatives must succeed
    Disabled,      // Will not execute
    Conditional,   // Execute based on condition
}
```

**Acceptance Criteria**:
- [ ] AC-13.2.1: Admin dapat create custom authentication flows
- [ ] AC-13.2.2: Admin dapat add/remove/reorder executions dalam flow
- [ ] AC-13.2.3: Admin dapat set requirement (REQUIRED, ALTERNATIVE, etc.)
- [ ] AC-13.2.4: Flow dapat di-copy dan di-customize
- [ ] AC-13.2.5: Built-in flows tidak dapat dihapus
- [ ] AC-13.2.6: Client dapat override default flow

#### 13.2.2 Required Actions
**Status di Keycloak**: ✅ Extensible system
**Status di Authenc**: ⚠️ Limited implementation

**Keycloak Required Actions**:
- **Verify Email**: User must verify email before access
- **Update Password**: Force password change
- **Update Profile**: Force profile update
- **Configure TOTP**: Force MFA setup
- **Terms and Conditions**: User must accept T&C
- **Update User Locale**: Select language preference
- **Delete Account**: User can request account deletion
- **WebAuthn Register**: Register security key

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct RequiredAction {
    pub id: Uuid,
    pub alias: String,              // "VERIFY_EMAIL", "UPDATE_PASSWORD"
    pub name: String,
    pub provider_id: String,
    pub enabled: bool,
    pub default_action: bool,       // Auto-add to new users
    pub priority: i32,
    pub config: HashMap<String, String>,
}

pub struct UserRequiredAction {
    pub user_id: Uuid,
    pub required_action: String,
    pub created_at: DateTime<Utc>,
}

// Implementasi untuk proyek pemerintahan:
pub enum GovRequiredAction {
    VerifyEmail,
    UpdatePassword,
    ConfigureMFA,           // Wajib untuk pegawai tertentu
    AcceptTerms,            // Persetujuan penggunaan sistem
    UpdateNIP,              // Update NIP untuk integrasi MySIMKARI
    VerifyIdentity,         // Verifikasi identitas dengan KTP
    AcceptPrivacyPolicy,    // GDPR compliance
    UpdateSatker,           // Update satuan kerja
}
```

**Acceptance Criteria**:
- [ ] AC-13.2.7: Admin dapat enable/disable required actions
- [ ] AC-13.2.8: Admin dapat set default actions untuk new users
- [ ] AC-13.2.9: Admin dapat set priority/order required actions
- [ ] AC-13.2.10: User melihat required actions setelah login
- [ ] AC-13.2.11: User tidak dapat akses sistem sampai complete required actions
- [ ] AC-13.2.12: Required actions dapat di-trigger programmatically

### 13.3 Role Management Enhancement (HIGH untuk Organizational Structure)

#### 13.3.1 Composite Roles
**Status di Keycloak**: ✅ Full support
**Status di Authenc**: ❌ Not implemented

**Keycloak Implementation**:
- Role dapat contain other roles (composite)
- Composite role inheritance
- Realm roles vs Client roles
- Role hierarchy

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct CompositeRole {
    pub parent_role_id: Uuid,
    pub child_role_id: Uuid,
    pub realm_id: Uuid,
}

// Contoh untuk struktur pemerintahan:
// Role "Kepala Kejaksaan" composite dari:
//   - "Pegawai" (base role)
//   - "Pimpinan" (management role)
//   - "Approval_Level_1" (approval authority)
//   - "View_All_Reports" (reporting access)

impl RoleStore {
    pub async fn add_composite_role(&self, parent_id: Uuid, child_id: Uuid) -> Result<()>;
    pub async fn remove_composite_role(&self, parent_id: Uuid, child_id: Uuid) -> Result<()>;
    pub async fn get_composite_roles(&self, role_id: Uuid) -> Result<Vec<Role>>;
    pub async fn get_effective_roles(&self, user_id: Uuid) -> Result<Vec<Role>>;
}
```

**Acceptance Criteria**:
- [ ] AC-13.3.1: Admin dapat create composite roles
- [ ] AC-13.3.2: Admin dapat add/remove child roles dari composite
- [ ] AC-13.3.3: User dengan composite role mendapat semua child roles
- [ ] AC-13.3.4: Circular dependency detection (role A → role B → role A)
- [ ] AC-13.3.5: Role hierarchy visualization di Admin Console

#### 13.3.2 Client Roles vs Realm Roles
**Status di Keycloak**: ✅ Clear separation
**Status di Authenc**: ⚠️ Mixed implementation

**Keycloak Implementation**:
- **Realm Roles**: Global roles untuk seluruh realm
- **Client Roles**: Specific untuk satu client/application
- Role scope mapping per client

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub enum RoleType {
    RealmRole,      // Global role
    ClientRole,     // Client-specific role
}

pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub role_type: RoleType,
    pub client_id: Option<Uuid>,  // NULL untuk realm role
    pub realm_id: Uuid,
    pub composite: bool,
    pub description: Option<String>,
}

// Contoh untuk SIMPelv2:
// Realm Roles: "pegawai", "pimpinan", "admin"
// Client Roles (Portal): "portal_user", "portal_admin"
// Client Roles (Perlengkapan): "perlengkapan_user", "perlengkapan_approver"
```

**Acceptance Criteria**:
- [ ] AC-13.3.6: Realm roles dapat digunakan di semua clients
- [ ] AC-13.3.7: Client roles hanya berlaku untuk client tertentu
- [ ] AC-13.3.8: Admin dapat map realm roles ke client roles
- [ ] AC-13.3.9: Token hanya include roles yang relevant untuk client

### 13.4 User Storage Federation Enhancement (HIGH untuk Integration)

#### 13.4.1 Custom User Storage Provider
**Status di Keycloak**: ✅ Full SPI support
**Status di Authenc**: ⚠️ Basic LDAP only

**Keycloak Implementation**:
- Custom User Storage SPI
- Read-only vs Read-Write providers
- Import users vs Federated users
- Credential validation delegation
- User attribute mapping

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
#[async_trait]
pub trait UserStorageProvider: Send + Sync {
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn search_users(&self, query: &str, max_results: usize) -> Result<Vec<User>>;
    async fn validate_credentials(&self, username: &str, password: &str) -> Result<bool>;
    async fn update_user(&self, user: &User) -> Result<()>;
    async fn is_read_only(&self) -> bool;
}

// Implementasi untuk MySIMKARI:
pub struct MysimkariUserStorageProvider {
    grpc_client: IntegrasiClient,
    cache: Arc<RedisCache>,
}

impl UserStorageProvider for MysimkariUserStorageProvider {
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        // 1. Check cache
        if let Some(user) = self.cache.get_user(username).await? {
            return Ok(Some(user));
        }

        // 2. Fetch from MySIMKARI via gRPC
        let mysimkari_user = self.grpc_client
            .get_pegawai_by_nip(username)
            .await?;

        // 3. Map to Authenc User
        let user = User {
            username: mysimkari_user.nip,
            email: mysimkari_user.email,
            full_name: mysimkari_user.nama,
            attributes: hashmap! {
                "nip" => mysimkari_user.nip,
                "satker" => mysimkari_user.satker_id,
                "jabatan" => mysimkari_user.jabatan,
            },
            ..Default::default()
        };

        // 4. Cache for 5 minutes
        self.cache.set_user(&user, Duration::from_secs(300)).await?;

        Ok(Some(user))
    }

    async fn is_read_only(&self) -> bool {
        true  // MySIMKARI is read-only
    }
}
```

**Acceptance Criteria**:
- [ ] AC-13.4.1: Admin dapat configure multiple user storage providers
- [ ] AC-13.4.2: User storage providers dapat di-prioritize
- [ ] AC-13.4.3: Credential validation dapat di-delegate ke provider
- [ ] AC-13.4.4: User attributes dapat di-map dari external source
- [ ] AC-13.4.5: Read-only providers tidak allow user updates
- [ ] AC-13.4.6: MySIMKARI integration sebagai user storage provider

### 13.5 Client Scope & Protocol Mappers (MEDIUM untuk Token Customization)

#### 13.5.1 Client Scopes
**Status di Keycloak**: ✅ Full support
**Status di Authenc**: ⚠️ Basic implementation

**Keycloak Implementation**:
- Default scopes (auto-included in tokens)
- Optional scopes (included if requested)
- Scope consent (user must approve)
- Scope inheritance
- Full Scope Allowed toggle

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct ClientScope {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,           // "openid-connect", "saml"
    pub attributes: HashMap<String, String>,
    pub protocol_mappers: Vec<ProtocolMapper>,
}

pub struct ClientScopeMapping {
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub default_scope: bool,        // Auto-include vs optional
}

// Contoh untuk SIMPelv2:
// Scope "profile" → maps to: name, email, nip
// Scope "satker" → maps to: satker_id, satker_name, hierarchy
// Scope "roles" → maps to: realm_roles, client_roles
// Scope "perlengkapan" → maps to: perlengkapan_permissions
```

**Acceptance Criteria**:
- [ ] AC-13.5.1: Admin dapat create client scopes
- [ ] AC-13.5.2: Admin dapat assign scopes to clients (default/optional)
- [ ] AC-13.5.3: User dapat consent to optional scopes
- [ ] AC-13.5.4: Token hanya include requested scopes
- [ ] AC-13.5.5: Full Scope Allowed dapat di-toggle per client

#### 13.5.2 Protocol Mappers Enhancement
**Status di Keycloak**: ✅ Rich mapper types
**Status di Authenc**: ⚠️ Basic mappers

**Keycloak Protocol Mappers**:
- User Attribute Mapper
- User Property Mapper
- Role Name Mapper
- Group Membership Mapper
- Audience Mapper
- Hardcoded Claim Mapper
- Script Mapper (JavaScript)

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub enum ProtocolMapperType {
    UserAttribute,          // Map user attribute to claim
    UserProperty,           // Map user property (username, email)
    RoleName,              // Map roles to claim
    GroupMembership,       // Map groups to claim
    Audience,              // Add audience to token
    HardcodedClaim,        // Add static claim
    Script,                // JavaScript mapper
    SatkerHierarchy,       // Custom: Map satker hierarchy
}

pub struct ProtocolMapper {
    pub id: Uuid,
    pub name: String,
    pub protocol: String,
    pub mapper_type: ProtocolMapperType,
    pub config: HashMap<String, String>,
}

// Contoh untuk SIMPelv2:
// Mapper "NIP" → user.attributes.nip → token.nip
// Mapper "Satker" → user.satker → token.satker
// Mapper "Hierarchy" → satker.hierarchy → token.satker_path
// Mapper "Roles" → user.roles → token.realm_access.roles
```

**Acceptance Criteria**:
- [ ] AC-13.5.6: Admin dapat create protocol mappers
- [ ] AC-13.5.7: Mappers dapat map user attributes to claims
- [ ] AC-13.5.8: Mappers dapat transform values (uppercase, lowercase)
- [ ] AC-13.5.9: Mappers dapat add hardcoded claims
- [ ] AC-13.5.10: Custom mapper untuk satker hierarchy

### 13.6 User Registration & Self-Service (HIGH untuk User Experience)

#### 13.6.1 User Self-Registration
**Status di Keycloak**: ✅ Full support with customization
**Status di Authenc**: ❌ Not implemented

**Keycloak Implementation**:
- Registration form customization
- Custom registration fields
- Email verification
- reCAPTCHA integration
- Terms and conditions
- Registration flow per client

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct RegistrationConfig {
    pub enabled: bool,
    pub email_as_username: bool,
    pub verify_email: bool,
    pub recaptcha_enabled: bool,
    pub terms_and_conditions: bool,
    pub custom_fields: Vec<RegistrationField>,
}

pub struct RegistrationField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,      // TEXT, EMAIL, SELECT, CHECKBOX
    pub required: bool,
    pub validation: Option<String>,  // Regex pattern
    pub options: Option<Vec<String>>, // For SELECT type
}

// Contoh untuk pegawai pemerintah:
pub fn create_gov_registration_fields() -> Vec<RegistrationField> {
    vec![
        RegistrationField {
            name: "nip".to_string(),
            label: "NIP (Nomor Induk Pegawai)".to_string(),
            field_type: FieldType::Text,
            required: true,
            validation: Some(r"^\d{18}$".to_string()),  // 18 digit
            options: None,
        },
        RegistrationField {
            name: "satker".to_string(),
            label: "Satuan Kerja".to_string(),
            field_type: FieldType::Select,
            required: true,
            validation: None,
            options: Some(vec![
                "Kejaksaan Agung".to_string(),
                "Kejaksaan Tinggi DKI Jakarta".to_string(),
                // ... loaded from database
            ]),
        },
        RegistrationField {
            name: "phone".to_string(),
            label: "Nomor Telepon".to_string(),
            field_type: FieldType::Text,
            required: true,
            validation: Some(r"^(\+62|0)[0-9]{9,12}$".to_string()),
            options: None,
        },
    ]
}
```

**Acceptance Criteria**:
- [ ] AC-13.6.1: User dapat self-register via registration page
- [ ] AC-13.6.2: Admin dapat enable/disable registration per realm
- [ ] AC-13.6.3: Admin dapat customize registration fields
- [ ] AC-13.6.4: Email verification required sebelum login
- [ ] AC-13.6.5: CAPTCHA protection untuk registration
- [ ] AC-13.6.6: Terms and conditions acceptance
- [ ] AC-13.6.7: NIP validation dengan MySIMKARI
- [ ] AC-13.6.8: Auto-assign default roles setelah registration

### 13.7 Session Management Enhancement (MEDIUM untuk Security)

#### 13.7.1 Session Limits & Policies
**Status di Keycloak**: ✅ Comprehensive
**Status di Authenc**: ⚠️ Basic

**Keycloak Implementation**:
- Max sessions per user
- Session idle timeout
- Session max lifespan
- Remember me duration
- Offline session support
- Session revocation

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct SessionPolicy {
    pub max_sessions_per_user: Option<u32>,     // NULL = unlimited
    pub session_idle_timeout: Duration,          // 30 minutes default
    pub session_max_lifespan: Duration,          // 10 hours default
    pub remember_me_enabled: bool,
    pub remember_me_lifespan: Duration,          // 30 days default
    pub offline_session_enabled: bool,
    pub offline_session_idle_timeout: Duration,  // 30 days default
}

impl SessionStore {
    pub async fn enforce_session_limit(&self, user_id: Uuid) -> Result<()> {
        let policy = self.get_session_policy().await?;

        if let Some(max_sessions) = policy.max_sessions_per_user {
            let sessions = self.get_user_sessions(user_id).await?;

            if sessions.len() >= max_sessions as usize {
                // Revoke oldest session
                let oldest = sessions.iter()
                    .min_by_key(|s| s.last_activity)
                    .unwrap();
                self.revoke_session(oldest.id).await?;
            }
        }

        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let policy = self.get_session_policy().await?;
        let now = Utc::now();

        // Delete idle sessions
        let idle_cutoff = now - policy.session_idle_timeout;
        let idle_count = self.delete_sessions_before(idle_cutoff).await?;

        // Delete max lifespan sessions
        let max_cutoff = now - policy.session_max_lifespan;
        let max_count = self.delete_sessions_created_before(max_cutoff).await?;

        Ok(idle_count + max_count)
    }
}
```

**Acceptance Criteria**:
- [ ] AC-13.7.1: Admin dapat set max sessions per user
- [ ] AC-13.7.2: Oldest session auto-revoked saat limit reached
- [ ] AC-13.7.3: Session idle timeout configurable
- [ ] AC-13.7.4: Session max lifespan configurable
- [ ] AC-13.7.5: Remember me functionality
- [ ] AC-13.7.6: Offline session support untuk mobile apps
- [ ] AC-13.7.7: Automatic cleanup expired sessions

### 13.8 Import/Export & Backup (HIGH untuk Operations)

#### 13.8.1 Realm Import/Export
**Status di Keycloak**: ✅ Full JSON export/import
**Status di Authenc**: ⚠️ Database backup only

**Keycloak Implementation**:
- Export entire realm to JSON
- Import realm from JSON
- Partial export (users, clients, roles)
- Export with/without users
- Export with/without credentials

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct RealmExport {
    pub realm: String,
    pub enabled: bool,
    pub users: Option<Vec<UserExport>>,
    pub clients: Vec<ClientExport>,
    pub roles: RealmRolesExport,
    pub groups: Vec<GroupExport>,
    pub identity_providers: Vec<IdentityProviderExport>,
    pub authentication_flows: Vec<AuthenticationFlowExport>,
    pub client_scopes: Vec<ClientScopeExport>,
}

pub struct ExportOptions {
    pub include_users: bool,
    pub include_credentials: bool,
    pub include_service_accounts: bool,
    pub users_per_file: Option<usize>,  // Split large exports
}

impl RealmService {
    pub async fn export_realm(
        &self,
        realm_id: Uuid,
        options: ExportOptions,
    ) -> Result<RealmExport> {
        let realm = self.get_realm(realm_id).await?;

        let users = if options.include_users {
            Some(self.export_users(realm_id, options.include_credentials).await?)
        } else {
            None
        };

        let clients = self.export_clients(realm_id).await?;
        let roles = self.export_roles(realm_id).await?;
        // ... export other components

        Ok(RealmExport {
            realm: realm.name,
            enabled: realm.enabled,
            users,
            clients,
            roles,
            // ...
        })
    }

    pub async fn import_realm(
        &self,
        export: RealmExport,
        overwrite: bool,
    ) -> Result<Uuid> {
        // 1. Create or update realm
        let realm_id = if overwrite {
            self.update_realm_from_export(&export).await?
        } else {
            self.create_realm_from_export(&export).await?
        };

        // 2. Import users
        if let Some(users) = export.users {
            self.import_users(realm_id, users).await?;
        }

        // 3. Import clients, roles, etc.
        self.import_clients(realm_id, export.clients).await?;
        self.import_roles(realm_id, export.roles).await?;

        Ok(realm_id)
    }
}
```

**Acceptance Criteria**:
- [ ] AC-13.8.1: Admin dapat export realm ke JSON file
- [ ] AC-13.8.2: Admin dapat import realm dari JSON file
- [ ] AC-13.8.3: Export dapat include/exclude users
- [ ] AC-13.8.4: Export dapat include/exclude credentials
- [ ] AC-13.8.5: Large exports dapat di-split ke multiple files
- [ ] AC-13.8.6: Import dapat overwrite existing realm
- [ ] AC-13.8.7: Import validation sebelum apply changes
- [ ] AC-13.8.8: Backup schedule automation

### 13.9 Email Templates & Notifications (MEDIUM untuk Communication)

#### 13.9.1 Email Template System
**Status di Keycloak**: ✅ Full template system
**Status di Authenc**: ⚠️ Hardcoded emails

**Keycloak Implementation**:
- Email verification template
- Password reset template
- Account update notification
- Admin notification
- Custom email templates
- Multi-language support
- HTML + Plain text versions

**Gap di Authenc**:
```rust
// Yang perlu ditambahkan:
pub struct EmailTemplate {
    pub id: Uuid,
    pub name: String,               // "email-verification", "password-reset"
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
    pub locale: String,             // "id", "en"
    pub realm_id: Uuid,
}

pub enum EmailTemplateType {
    EmailVerification,
    PasswordReset,
    PasswordChanged,
    AccountUpdated,
    MfaEnabled,
    MfaDisabled,
    LoginAlert,                     // Suspicious login
    AdminNotification,
}

// Template variables untuk Indonesian government:
// ${user.name} - Nama pegawai
// ${user.nip} - NIP
// ${user.satker} - Satuan kerja
// ${link} - Verification/reset link
// ${ip_address} - IP address
// ${timestamp} - Waktu kejadian

pub const EMAIL_VERIFICATION_ID: &str = r#"
<html>
<body>
    <h2>Verifikasi Email - SIMPEL v2</h2>
    <p>Yth. ${user.name},</p>
    <p>Terima kasih telah mendaftar di SIMPEL v2.</p>
    <p>Silakan klik link berikut untuk verifikasi email Anda:</p>
    <p><a href="${link}">Verifikasi Email</a></p>
    <p>Link ini berlaku selama 24 jam.</p>
    <br>
    <p>Hormat kami,</p>
    <p>Tim SIMPEL v2<br>Kejaksaan Republik Indonesia</p>
</body>
</html>
"#;
```

**Acceptance Criteria**:
- [ ] AC-13.9.1: Admin dapat customize email templates
- [ ] AC-13.9.2: Email templates support variables
- [ ] AC-13.9.3: Multi-language email templates
- [ ] AC-13.9.4: HTML dan plain text versions
- [ ] AC-13.9.5: Email preview di Admin Console
- [ ] AC-13.9.6: Test email functionality
- [ ] AC-13.9.7: Email templates untuk Indonesian language
- [ ] AC-13.9.8: Government branding dalam emails

## 14. Prioritas Implementasi untuk Proyek SIMPelv2

### 14.1 CRITICAL (Harus Segera - 0-3 bulan)

#### Priority 1: Admin Console UI
**Alasan**: Tanpa Admin Console, management Authenc sangat sulit
**Estimasi**: 8 minggu
**Dependencies**: Leptos 0.8.x
**Impact**: Blocking untuk production deployment

#### Priority 2: Event System (Admin Events + User Events)
**Alasan**: Compliance requirement untuk audit pemerintah
**Estimasi**: 3 minggu
**Dependencies**: Database migration
**Impact**: Critical untuk compliance (ISO 27001, audit requirements)

#### Priority 3: Required Actions System
**Alasan**: Enforce security policies (MFA, password change, T&C)
**Estimasi**: 2 minggu
**Dependencies**: Authentication flow
**Impact**: Security compliance

#### Priority 4: User Account Console
**Alasan**: User self-service mengurangi beban admin
**Estimasi**: 4 minggu
**Dependencies**: Leptos UI
**Impact**: User experience improvement

### 14.2 HIGH (Penting - 3-6 bulan)

#### Priority 5: Authentication Flows Customization
**Alasan**: Flexibility untuk custom authentication requirements
**Estimasi**: 4 minggu
**Dependencies**: Database schema
**Impact**: Enables custom flows per client

#### Priority 6: Composite Roles
**Alasan**: Simplify role management untuk struktur organisasi kompleks
**Estimasi**: 2 minggu
**Dependencies**: Role store enhancement
**Impact**: Organizational structure support

#### Priority 7: User Storage Federation (MySIMKARI)
**Alasan**: Integration dengan MySIMKARI untuk auto-provisioning
**Estimasi**: 3 minggu
**Dependencies**: Integrasi gRPC client
**Impact**: Seamless integration dengan sistem pemerintah

#### Priority 8: User Self-Registration
**Alasan**: Reduce admin workload untuk user onboarding
**Estimasi**: 3 minggu
**Dependencies**: Email templates, CAPTCHA
**Impact**: Operational efficiency

#### Priority 9: Email Template System
**Alasan**: Professional communication dengan users
**Estimasi**: 2 minggu
**Dependencies**: Email service
**Impact**: User communication quality

### 14.3 MEDIUM (Berguna - 6-9 bulan)

#### Priority 10: Client Scopes & Protocol Mappers Enhancement
**Alasan**: Fine-grained token customization
**Estimasi**: 3 minggu
**Dependencies**: Token generation
**Impact**: Token optimization

#### Priority 11: Session Management Enhancement
**Alasan**: Better session control dan security
**Estimasi**: 2 minggu
**Dependencies**: Redis cache
**Impact**: Security improvement

#### Priority 12: Import/Export System
**Alasan**: Backup, disaster recovery, migration
**Estimasi**: 3 minggu
**Dependencies**: JSON serialization
**Impact**: Operational resilience

#### Priority 13: Theme System
**Alasan**: Branding customization per realm
**Estimasi**: 2 minggu
**Dependencies**: Template engine
**Impact**: Branding flexibility

#### Priority 14: Admin CLI
**Alasan**: Automation dan scripting
**Estimasi**: 3 minggu
**Dependencies**: REST API
**Impact**: DevOps automation

### 14.4 LOW (Optional - 9-12 bulan)

#### Priority 15: Client Adapters
**Alasan**: Easier integration untuk developers
**Estimasi**: 4 minggu
**Dependencies**: None
**Impact**: Developer experience

#### Priority 16: Device Flow
**Alasan**: IoT device authentication
**Estimasi**: 2 minggu
**Dependencies**: OAuth2 implementation
**Impact**: IoT support

#### Priority 17: User Impersonation
**Alasan**: Support dan debugging
**Estimasi**: 1 minggu
**Dependencies**: Session management
**Impact**: Support efficiency

## 15. Estimasi Total Effort

### Timeline Summary
- **Phase 1 (Critical)**: 3 bulan - 4 developers
- **Phase 2 (High)**: 3 bulan - 3 developers
- **Phase 3 (Medium)**: 3 bulan - 2 developers
- **Phase 4 (Low)**: 3 bulan - 2 developers

### Total Effort
- **Total Duration**: 12 bulan (dengan parallel development)
- **Total Person-Months**: ~36 person-months
- **Team Size**: 3-4 developers (full-time)

## 16. Rekomendasi Khusus untuk Proyek Pemerintahan Indonesia

### 16.1 Fitur Spesifik yang Harus Ditambahkan

#### 16.1.1 NIP (Nomor Induk Pegawai) Integration
```rust
pub struct NipValidator {
    mysimkari_client: Arc<IntegrasiClient>,
}

impl NipValidator {
    pub async fn validate_nip(&self, nip: &str) -> Result<NipValidationResult> {
        // 1. Format validation (18 digit)
        if !nip.chars().all(|c| c.is_numeric()) || nip.len() != 18 {
            return Ok(NipValidationResult::InvalidFormat);
        }

        // 2. Check with MySIMKARI
        match self.mysimkari_client.verify_nip(nip).await {
            Ok(pegawai) => Ok(NipValidationResult::Valid(pegawai)),
            Err(_) => Ok(NipValidationResult::NotFound),
        }
    }
}
```

#### 16.1.2 Satker Hierarchy Integration
```rust
pub struct SatkerHierarchyService {
    db: Arc<Database>,
}

impl SatkerHierarchyService {
    pub async fn get_satker_path(&self, satker_id: &str) -> Result<Vec<Satker>> {
        // Return: Kejaksaan Agung → Kejati DKI → Kejari Jakarta Pusat
        self.db.query_satker_hierarchy(satker_id).await
    }

    pub async fn check_satker_access(
        &self,
        user_satker: &str,
        resource_satker: &str,
    ) -> Result<bool> {
        // User dapat akses resource jika:
        // 1. Same satker
        // 2. Parent satker (hierarchical access)
        // 3. Has special permission

        let user_path = self.get_satker_path(user_satker).await?;
        let resource_path = self.get_satker_path(resource_satker).await?;

        // Check if user_satker is parent of resource_satker
        Ok(resource_path.iter().any(|s| s.id == user_satker))
    }
}
```

#### 16.1.3 Compliance Reporting untuk Audit
```rust
pub struct ComplianceReportService {
    audit_store: Arc<PgAuditLogStore>,
}

impl ComplianceReportService {
    pub async fn generate_bpk_audit_report(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<BpkAuditReport> {
        BpkAuditReport {
            period: format!("{} - {}", start_date, end_date),
            total_users: self.count_active_users().await?,
            total_logins: self.count_logins(start_date, end_date).await?,
            failed_logins: self.count_failed_logins(start_date, end_date).await?,
            mfa_adoption_rate: self.calculate_mfa_adoption().await?,
            admin_actions: self.get_admin_actions(start_date, end_date).await?,
            security_incidents: self.get_security_incidents(start_date, end_date).await?,
            compliance_status: self.check_compliance_status().await?,
        }
    }
}
```

### 16.2 Acceptance Criteria Tambahan untuk Pemerintahan

- [ ] AC-16.1: NIP validation dengan MySIMKARI
- [ ] AC-16.2: Satker hierarchy enforcement
- [ ] AC-16.3: Compliance report untuk audit
- [ ] AC-16.4: Indonesian language support (UI, emails, docs)
- [ ] AC-16.5: Government branding (logo, colors, themes)
- [ ] AC-16.6: Integration dengan SIMAN (Sistem Informasi Manajemen)
- [ ] AC-16.7: Integration dengan MonSAKTI (Monitoring SAKTI)
- [ ] AC-16.8: SPBE (Sistem Pemerintahan Berbasis Elektronik) compliance
- [ ] AC-16.9: Data residency (data harus di Indonesia)
- [ ] AC-16.10: Disaster recovery plan sesuai standar pemerintah

## 17. Kesimpulan dan Rekomendasi Final

### 17.1 Status Saat Ini
Authenc sudah memiliki **90%+ feature parity** dengan Keycloak untuk core authentication/authorization. Yang kurang adalah:
1. **UI Components** (Admin Console, User Account Console)
2. **Advanced Features** (Authentication Flows, Required Actions, Composite Roles)
3. **Operational Tools** (Import/Export, Admin CLI)
4. **Government-Specific Features** (NIP validation, Satker hierarchy, audit reporting)

### 17.2 Rekomendasi Strategis

**LANJUTKAN DENGAN AUTHENC** dengan alasan:
1. ✅ Sudah 90% complete
2. ✅ Superior security (Ed25519, Post-Quantum)
3. ✅ Better performance (Rust vs Java)
4. ✅ Custom features untuk pemerintah Indonesia
5. ✅ Integration dengan MySIMKARI, SIMAN, MonSAKTI
6. ✅ Lower operational cost
7. ✅ Alignment dengan SIMPelv2 tech stack (Rust)

**JANGAN MIGRATE KE KEYCLOAK** karena:
1. ❌ Effort migrasi lebih besar dari complete Authenc
2. ❌ Kehilangan custom features (Satker, MySIMKARI)
3. ❌ Higher resource usage (500MB vs 200MB)
4. ❌ Slower performance (Java vs Rust)
5. ❌ Tidak align dengan tech stack SIMPelv2

### 17.3 Action Plan

**Immediate (Q1 2026)**:
1. Complete Admin Console UI
2. Implement Event System (Admin + User Events)
3. Add Required Actions System
4. Build User Account Console

**Short-term (Q2 2026)**:
5. Implement Authentication Flows Customization
6. Add Composite Roles
7. Integrate MySIMKARI User Storage
8. Build User Self-Registration
9. Create Email Template System

**Medium-term (Q3 2026)**:
10. Enhance Client Scopes & Protocol Mappers
11. Improve Session Management
12. Build Import/Export System
13. Create Theme System
14. Develop Admin CLI

**Long-term (Q4 2026)**:
15. Build Client Adapters
16. Implement Device Flow
17. Add User Impersonation
18. Complete government-specific features

### 17.4 Success Metrics
- ✅ 100% critical features implemented (Q1 2026)
- ✅ 90% high-priority features implemented (Q2 2026)
- ✅ Production deployment di 5+ satker (Q2 2026)
- ✅ Zero Keycloak dependencies (Q3 2026)
- ✅ Audit compliance (Q3 2026)
- ✅ 1000+ active users (Q4 2026)

---

**Document Version**: 2.0
**Last Updated**: 2026-02-12
**Author**: AI Analysis (Kiro) - Deep Dive
**Status**: READY FOR IMPLEMENTATION
**Next Step**: Create design.md untuk Phase 1 features
