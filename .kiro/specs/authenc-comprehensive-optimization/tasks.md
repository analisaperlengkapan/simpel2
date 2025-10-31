# Implementation Plan: Authenc Comprehensive Optimization

## Overview

Implementation plan untuk optimalisasi menyeluruh Authenc IAM system. Tasks diorganisir dalam epic yang fokus pada implementasi gRPC, optimasi performa, dan integrasi dengan ekosistem SIMPelv2.

**Current Status**: Authenc sudah memiliki foundation yang solid dengan Redis cache, Secreton integration, Kafka events, Ed25519 JWT, Argon2 password hashing, dan prepared statement cache. Tasks di bawah fokus pada gap yang tersisa untuk mencapai requirements.

## Task List

- [ ] 1. Setup gRPC Service Infrastructure
- [x] 1.1 Add tonic and prost dependencies to Cargo.toml
  - Add `tonic = "0.12"` with features ["transport", "tls"]
  - Add `prost = "0.13"` for protobuf support
  - Add `tonic-build = "0.12"` to build-dependencies
  - Add `prost-types = "0.13"` for well-known types
  - _Requirements: 16.1, 16.2_

- [x] 1.2 Create gRPC module structure
  - Create `src/grpc/mod.rs` with module exports
  - Create `src/grpc/authenc_service.rs` for service implementation
  - Create `src/grpc/interceptors.rs` for auth, logging, metrics middleware
  - Create `src/grpc/health.rs` for grpc.health.v1.Health service
  - _Requirements: 16.1, 16.5_

- [x] 1.3 Implement build.rs for proto compilation
  - Create `build.rs` in authenc root directory
  - Configure tonic_build to compile `../proto/authenc.proto` and `../proto/common.proto`
  - Set up proper include paths for proto imports
  - Generate Rust code from proto definitions into `src/grpc/proto.rs`
  - _Requirements: 16.1_

- [x] 1.4 Implement AuthencService gRPC methods
  - Implement `Authenticate` RPC (username/password + optional MFA)
  - Implement `ValidateToken` RPC (fast token validation < 50ms)
  - Implement `CheckPermission` RPC (authorization check < 20ms)
  - Implement `EnableMFA`, `VerifyMFA`, `DisableMFA` RPC methods
  - Implement `RefreshToken` and `RevokeToken` RPC methods
  - _Requirements: 16.1, 16.2, 20.1_

- [x] 1.5 Create gRPC server initialization and lifecycle
  - Add gRPC server to `src/main.rs` alongside Axum HTTP server
  - Configure server to listen on port 9088 with HTTP/2
  - Implement graceful shutdown coordination between HTTP and gRPC servers
  - Add gRPC server health checks
  - _Requirements: 16.3, 5.5_



- [x] 2. Implement Redis Cache Layer (COMPLETED)
- [x] 2.1 Enhance RedisCache implementation
  - ✓ Connection pooling with ConnectionManager implemented
  - ✓ Health check method exists
  - ✓ Basic cache operations (get, set, delete, exists, set_nx)
  - _Requirements: 15.1, 15.2_

- [x] 2.2 Add cache metrics collection
  - Implement hit/miss ratio tracking in RedisCache
  - Add cache operation latency metrics
  - Expose cache metrics via Prometheus
  - Add cache size and eviction metrics
  - _Requirements: 15.1, 6.1_

- [x] 2.3 Create MultiLayerCache wrapper
  - Implement L1 in-memory cache with DashMap (TTL: 60s, size: 10k entries)
  - Implement L2 Redis cache integration with fallback
  - Add cache-aside pattern with automatic L1 population from L2
  - Implement LRU eviction for L1 cache
  - _Requirements: 15.1, 15.2_

- [x] 2.4 Enhance cache invalidation mechanism
  - Implement Kafka-based cache invalidation subscriber
  - Add cache invalidation on permission/role changes
  - Add cache invalidation on user updates
  - Implement cache warming on startup for active users
  - _Requirements: 15.3, 5.3_

- [x] 3. Database Optimization (PARTIALLY COMPLETED)
- [x] 3.1 Implement prepared statement cache
  - ✓ PreparedStatementCache struct exists with DashMap
  - ✓ get_or_prepare method implemented
  - ✓ LRU eviction when cache is full
  - _Requirements: 4.2_

- [x] 3.2 Integrate prepared statement cache into operations
  - Add PreparedStatementCache to Database struct
  - Update user operations to use prepared statements
  - Update session operations to use prepared statements
  - Update audit log operations to use prepared statements
  - Update permission operations to use prepared statements
  - _Requirements: 4.2_

- [x] 3.3 Optimize database connection pool configuration
  - Review and tune deadpool-postgres settings (max: 50, min: 10)
  - Add connection pool metrics (active, idle, waiting)
  - Implement connection health checks with periodic validation
  - Add connection timeout and idle timeout configuration
  - _Requirements: 4.1_

- [x] 3.4 Add database indexes via migration
  - Create migration for user table indexes (email, username, satker_code)
  - Create migration for session table indexes (user_id, expires_at)
  - Create migration for audit_logs table indexes (user_id + timestamp composite)
  - Create migration for permissions table indexes (user_id + resource composite)
  - Use CREATE INDEX CONCURRENTLY to avoid locking
  - _Requirements: 4.3_

- [x] 3.5 Implement batch operations for performance
  - Add batch insert for audit logs (batch size: 1000)
  - Add batch query for user permissions
  - Add batch session validation
  - Implement batch user lookup by IDs
  - _Requirements: 4.4_

- [x] 4. Secreton Integration Enhancement (MOSTLY COMPLETED)
- [x] 4.1 Implement Secreton HTTP client
  - ✓ SecretonClient struct with HTTP client (reqwest)
  - ✓ get_secret method with satker isolation
  - ✓ get_signing_key and get_encryption_key methods
  - ✓ Circuit breaker implementation with retry logic
  - ✓ MFA-specific methods (setup, verify, status)
  - _Requirements: 17.1, 17.2_

- [x] 4.2 Integrate Secreton for MFA secrets
  - ✓ MfaService uses SecretonClient for secret storage
  - ✓ Encryption-as-a-service via Transit Engine methods
  - ✓ MFA secret encryption with Secreton
  - _Requirements: 17.2, 3.5_

- [x] 4.3 Add Secreton fallback mechanism
  - ✓ Circuit breaker for Secreton calls (failure threshold: 5, timeout: 60s)
  - ✓ Exponential backoff retry logic (max 3 retries)
  - ✓ Error handling with VaultError types
  - _Requirements: 17.4_

- [x] 4.4 Implement automatic key rotation
  - Create key rotation scheduler (runs every 30 days)
  - Implement RotateKey API integration with Secreton
  - Add rotation audit logging to database
  - Add notification on key rotation completion
  - _Requirements: 17.3_

- [x] 4.5 Add local encrypted storage fallback
  - Implement local AES-GCM encrypted storage for MFA secrets
  - Add degraded mode detection and warning logs
  - Implement automatic sync when Secreton recovers
  - Add metrics for fallback usage
  - _Requirements: 17.4_

- [x] 5. Event System and Kafka Integration (PARTIALLY COMPLETED)
- [x] 5.1 Basic Kafka event publishing
  - ✓ KafkaEventListener implemented with FutureProducer
  - ✓ Event publishing to user_events_topic and admin_events_topic
  - ✓ EventListenerProvider trait implementation
  - _Requirements: 5.4_

- [x] 5.2 Enhance EventPublisher with reliability
  - Add retry logic for failed publishes (max 3 retries with backoff)
  - Implement dead letter queue for permanently failed events
  - Add event batching for performance (batch size: 100, flush interval: 1s)
  - Add event publishing metrics (success/failure counters)
  - _Requirements: 5.4_

- [x] 5.3 Implement comprehensive audit events
  - Add UserLogin event with IP, user agent, timestamp
  - Add UserLogout event with session duration
  - Add MFAEnabled/Disabled events with admin actor
  - Add PermissionGranted/Revoked events with resource details
  - Add PasswordChanged event
  - _Requirements: 10.1, 10.2_

- [x] 5.4 Create event-driven cache invalidation
  - Subscribe to permission change events from Kafka
  - Subscribe to user update events from Kafka
  - Implement cache invalidation handler for events
  - Add cache invalidation for role changes
  - _Requirements: 15.3, 5.3_

- [x] 5.5 Add event retention management
  - Implement event archiving to cold storage (S3/MinIO)
  - Add configurable retention policies (default: 90 days)
  - Create cleanup scheduler (runs daily)
  - Add event retention metrics
  - _Requirements: 10.3_



- [ ] 6. Performance Optimization
- [x] 6.1 Implement async optimization
  - Refactor authentication flow to use tokio::join! for parallel DB + cache checks
  - Add batch processing for bulk permission checks
  - Optimize user lookup with parallel queries for profile + permissions
  - Use tokio::spawn for non-blocking audit log writes
  - _Requirements: 1.5_

- [x] 6.2 Optimize JWT validation with caching
  - Implement JWT validation result caching (TTL: 5 minutes)
  - Add fast-path for cached token validation (< 10ms)
  - Cache JWT signature verification results
  - Add token blacklist check in cache
  - _Requirements: 1.1, 15.1_

- [ ] 6.3 Add request-level caching middleware
  - Implement per-request cache using tower Layer
  - Cache user profile for request duration
  - Cache permissions for request duration
  - Add cache hit/miss metrics per request
  - _Requirements: 1.2, 15.1_

- [ ] 6.4 Optimize database connection pooling
  - Tune pool parameters (max: 50, min: 10, timeout: 30s)
  - Add connection reuse metrics
  - Implement connection health monitoring with periodic checks
  - Add connection wait time metrics
  - _Requirements: 1.3, 4.1_

- [ ] 6.5 Run performance benchmarks with Criterion
  - Benchmark authentication flow (target: < 100ms P95)
  - Benchmark token validation (target: < 50ms P95)
  - Benchmark permission checks (target: < 20ms P95)
  - Benchmark MFA verification (target: < 100ms P95)
  - Generate performance report with graphs
  - _Requirements: 8.4, 1.1, 1.2_

- [x] 7. Security Hardening (MOSTLY COMPLETED)
- [x] 7.1 Implement Ed25519 JWT signing
  - ✓ Ed25519 JWT signing implemented in handlers/oidc_ed25519.rs
  - ✓ Legacy RSA implementation deprecated and disabled
  - ✓ Ed25519 keys in crypto/ed25519_keys.rs
  - ✓ Token generation and validation using Ed25519
  - _Requirements: 3.1, 3.2_

- [x] 7.2 Password hashing with Argon2
  - ✓ Argon2 password hashing implemented
  - ✓ Password verification in utils/crypto
  - _Requirements: 3.3_

- [x] 7.3 Enhance password security
  - Verify Argon2 configuration (cost factor >= 10, memory >= 64MB)
  - Add password strength validation (min 8 chars, complexity rules)
  - Implement password history check (prevent reuse of last 5 passwords)
  - Add password expiration policy (configurable, default: 90 days)
  - _Requirements: 3.3_

- [x] 7.4 Implement adaptive rate limiting
  - Create AdaptiveRateLimiter struct with threat level tracking
  - Implement threat level detection based on failed attempts
  - Add dynamic rate limit adjustment (normal: 100/min, elevated: 50/min, high: 20/min, critical: 5/min)
  - Integrate with existing rate_limit_axum middleware
  - _Requirements: 3.4_

- [x] 7.5 Add comprehensive input validation
  - Implement garde validation for all API request structs
  - Add input sanitization for user-provided strings
  - Implement request size limits (max body: 1MB)
  - Add validation for email, username, password formats
  - _Requirements: 3.4_

- [x] 7.6 Run security testing suite
  - Test SQL injection prevention with malicious inputs
  - Test JWT manipulation attempts (signature tampering, expiry bypass)
  - Test rate limit bypass attempts
  - Test XSS prevention in admin console
  - Generate security test report
  - _Requirements: 8.1_

- [x] 8. Istio Service Mesh Integration (PARTIALLY COMPLETED)
- [x] 8.1 Basic Istio configuration
  - ✓ Istio Gateway configured (infra/k8s/00-istio-gateway.yaml)
  - ✓ VirtualService configured (infra/k8s/00-istio-virtualservice.yaml)
  - ✓ DestinationRule configured (infra/k8s/18-istio-destination-rules.yaml)
  - _Requirements: 18.1, 18.3, 18.4_

- [ ] 8.2 Configure Authenc-specific Istio resources
  - Create VirtualService for /auth/* prefix routing to authenc service
  - Add timeout policy (30s) and retry policy (3 attempts, 10s per try)
  - Configure traffic splitting for canary deployment (90/10 split)
  - Add request header manipulation for trace propagation
  - _Requirements: 18.3_

- [ ] 8.3 Create Authenc DestinationRule
  - Configure connection pooling (max connections: 100, http1: 50, http2: 100)
  - Add circuit breaker settings (consecutive errors: 5, interval: 30s, base ejection: 30s)
  - Implement outlier detection for unhealthy instances
  - Configure load balancing (LEAST_REQUEST)
  - _Requirements: 18.4_

- [ ] 8.4 Update Authenc deployment for Istio
  - Add Istio sidecar injection annotation (sidecar.istio.io/inject: "true")
  - Configure mTLS mode (ISTIO_MUTUAL)
  - Add Istio labels (app, version) for traffic management
  - Expose metrics port (15020) for Prometheus scraping
  - _Requirements: 18.1, 18.2_

- [ ] 8.5 Implement distributed tracing
  - Add OpenTelemetry instrumentation to Axum app
  - Configure trace header propagation (x-request-id, x-b3-traceid, x-b3-spanid)
  - Integrate with Jaeger collector
  - Add span creation for database queries, cache operations, external calls
  - Configure trace sampling (10% in production)
  - _Requirements: 18.5, 6.3_

- [ ] 9. Microservice Integration
- [ ] 9.1 Implement gRPC health check service
  - Add grpc.health.v1.Health service implementation
  - Implement health check logic (database, Redis, Secreton connectivity)
  - Return SERVING/NOT_SERVING status based on dependencies
  - Configure Kubernetes liveness/readiness probes to use gRPC health check
  - _Requirements: 5.5_

- [ ] 9.2 Implement CheckPermission gRPC endpoint
  - Implement fast permission check with cache-first strategy (< 20ms)
  - Add permission result caching (TTL: 5 minutes)
  - Implement batch permission check for multiple resources
  - Add context-aware permission evaluation (user, resource, action, satker)
  - Return detailed error with required vs current permissions
  - _Requirements: 20.1, 20.2, 20.5, 20.4_

- [ ] 9.3 Implement Satker-aware authorization
  - Add Satker hierarchy model (parent-child relationships)
  - Implement Satker hierarchy traversal for permission inheritance
  - Add cross-Satker operation validation
  - Implement Satker-scoped admin roles
  - Add API for querying Satker hierarchy
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 20.3_

- [ ] 9.4 Create Envoy ext_authz gRPC endpoint
  - Implement external authorization API (envoy.service.auth.v3.Authorization)
  - Add fast token validation from Authorization header (< 10ms)
  - Return CheckResponse with OK/DENIED status
  - Add rate limiting metadata in response headers
  - Implement token caching for Envoy layer
  - _Requirements: 21.1, 21.2, 21.4, 21.5_

- [ ] 9.5 Write microservice integration tests
  - Test gRPC Authenticate endpoint from mock layanan client
  - Test gRPC CheckPermission with various permission scenarios
  - Test Envoy ext_authz integration with mock requests
  - Test Satker hierarchy authorization
  - Test gRPC health check responses
  - _Requirements: 8.2_



- [x] 10. Portal SSO Integration (PARTIALLY COMPLETED)
- [x] 10.1 Basic OIDC endpoints
  - ✓ OIDC discovery endpoint exists (oidc_ed25519::oidc_discovery_ed25519)
  - ✓ Authorization endpoint exists (oidc_ed25519::oidc_authorize_ed25519)
  - ✓ Token endpoint exists (oidc_ed25519::oidc_token_ed25519)
  - _Requirements: 19.3_

- [x] 10.2 Enhance OIDC discovery endpoint
  - Add JWKS endpoint URL to discovery document
  - Add supported grant types (authorization_code, refresh_token)
  - Add supported response types (code, token, id_token)
  - Add supported scopes (openid, profile, email)
  - Add token endpoint auth methods
  - _Requirements: 19.3_

- [x] 10.3 Implement JWKS endpoint
  - Create /.well-known/jwks.json endpoint
  - Expose Ed25519 public keys in JWK format
  - Add key rotation support (multiple keys with kid)
  - Cache JWKS response (TTL: 1 hour)
  - _Requirements: 19.3_

- [x] 10.4 Enhance SSO cookie management
  - Implement secure AUTHENC_SSO cookie generation
  - Add cookie domain configuration (simpel.kejaksaan.go.id)
  - Set SameSite=Lax and HttpOnly=true flags
  - Add Secure flag for HTTPS
  - Implement cookie-based session tracking
  - _Requirements: 19.1_

- [x] 10.5 Implement authorization code flow
  - Create authorization request handler with PKCE support
  - Implement authorization code generation and storage (TTL: 10 minutes)
  - Create token exchange endpoint (code for access_token + refresh_token)
  - Add state parameter validation for CSRF protection
  - Implement redirect_uri validation
  - _Requirements: 19.2_

- [x] 10.6 Add silent token refresh
  - Implement refresh token endpoint
  - Add iframe-based silent refresh support
  - Implement token rotation (new refresh token on each use)
  - Add refresh token expiration (30 days)
  - Implement refresh token revocation
  - _Requirements: 19.4_

- [x] 10.7 Create logout endpoint
  - Implement SSO session termination
  - Add post_logout_redirect_uri parameter
  - Implement logout propagation to federated providers
  - Clear AUTHENC_SSO cookie
  - Add logout event publishing
  - _Requirements: 19.5_

- [x] 11. Observability Enhancement (PARTIALLY COMPLETED)
- [x] 11.1 Basic observability infrastructure
  - ✓ ObservabilityService implemented
  - ✓ PrometheusMetricsCollector exists
  - ✓ Structured logging with tracing
  - _Requirements: 6.1, 6.2_

- [ ] 11.2 Implement comprehensive Prometheus metrics
  - Add request latency histograms (P50, P95, P99) per endpoint
  - Add request rate counters per endpoint and status code
  - Add error rate counters by error type
  - Add cache hit/miss ratio metrics
  - Add database connection pool metrics (active, idle, waiting)
  - Add gRPC method call metrics
  - Add MFA verification metrics
  - _Requirements: 6.1, 6.4, 23.1_

- [ ] 11.3 Enhance structured logging
  - Add trace_id to all log entries using tower-http
  - Implement request/response logging middleware
  - Add error logging with full context (user_id, endpoint, params)
  - Configure log levels per module (ERROR for prod, DEBUG for dev)
  - Add correlation ID propagation
  - _Requirements: 6.2_

- [ ] 11.4 Create Grafana dashboards
  - Create authentication metrics dashboard (login rate, success/failure, MFA usage)
  - Create performance metrics dashboard (latency, throughput, cache hit ratio)
  - Create error tracking dashboard (error rate by type, failed requests)
  - Create business KPIs dashboard (DAU, MAU, MFA adoption rate)
  - Export dashboard JSON files to config/grafana/
  - _Requirements: 23.1, 23.3, 23.5_

- [ ] 11.5 Configure Prometheus alerts
  - Add high error rate alert (> 5% for 5 minutes)
  - Add high latency alert (P95 > 200ms for 5 minutes)
  - Add low cache hit ratio alert (< 70% for 10 minutes)
  - Add database connection pool exhaustion alert (> 90% utilization)
  - Add Secreton unavailability alert
  - Export alert rules to config/prometheus/authenc_alerts.yml
  - _Requirements: 23.4_

- [ ] 11.6 Test observability stack
  - Verify metrics are exposed on /metrics endpoint
  - Test alert triggering with simulated failures
  - Validate Grafana dashboard queries
  - Test log aggregation and search
  - _Requirements: 8.2_

- [ ] 12. Compliance and Audit
- [x] 12.1 Implement tamper-proof audit logging
  - Add HMAC-SHA256 signing to audit log entries
  - Store signature in audit_logs table (new column: signature)
  - Implement audit log verification function
  - Add periodic integrity check job (runs daily)
  - Add alert on integrity check failure
  - _Requirements: 24.1_

- [x] 12.2 Enhance audit log details
  - Add IP address to all audit events (from request headers)
  - Add user agent to all audit events
  - Add geolocation data (optional, from IP lookup)
  - Add request/response payloads (sanitized, PII removed)
  - Add session ID for correlation
  - _Requirements: 24.2_

- [ ] 12.3 Create audit log export API
  - Implement JSON export endpoint (/api/v1/audit/export?format=json)
  - Implement CSV export endpoint (/api/v1/audit/export?format=csv)
  - Implement SIEM format export (CEF format)
  - Add filtering by user_id, action, date range, resource
  - Add pagination (limit, offset)
  - Add authentication and authorization for export API
  - _Requirements: 24.3, 24.5_

- [ ] 12.4 Implement audit log archiving
  - Create archiving scheduler (runs monthly)
  - Implement cold storage integration (S3/MinIO)
  - Archive logs older than retention period (default: 90 days)
  - Add archive retrieval API
  - Implement archive search functionality
  - _Requirements: 24.4_

- [ ] 12.5 Write compliance tests
  - Test audit log completeness (all events logged)
  - Test audit log integrity (signature verification)
  - Validate GDPR compliance (data retention, right to erasure)
  - Validate ISO 27001 controls
  - Generate compliance report
  - _Requirements: 8.1, 24.1, 24.2_

- [ ] 13. Deployment and Operations
- [ ] 13.1 Create Authenc Kubernetes deployment manifest
  - Create infra/k8s/authenc-deployment.yaml
  - Add resource requests (cpu: 250m, memory: 512Mi)
  - Add resource limits (cpu: 1, memory: 2Gi)
  - Configure liveness probe (gRPC health check, initialDelay: 30s, period: 10s)
  - Configure readiness probe (gRPC health check, initialDelay: 10s, period: 5s)
  - Add pre-stop hook (sleep 15s for connection draining)
  - Add Istio sidecar injection annotation
  - _Requirements: 25.1, 25.3, 25.5_

- [ ] 13.2 Implement graceful shutdown
  - Add SIGTERM signal handling in main.rs
  - Implement connection draining (max 30s)
  - Wait for in-flight requests to complete
  - Close database connections gracefully
  - Close Redis connections gracefully
  - Stop Kafka producer gracefully
  - _Requirements: 25.1, 9.5_

- [ ] 13.3 Create database migration strategy
  - Document backward-compatible schema change patterns
  - Create migration rollback scripts for each migration
  - Implement migration dry-run mode (--dry-run flag)
  - Add migration validation before apply
  - Use advisory locks to prevent concurrent migrations
  - _Requirements: 22.1, 22.2, 22.3, 22.5_

- [ ] 13.4 Configure rolling update strategy
  - Set maxSurge: 1 and maxUnavailable: 0 in deployment
  - Add health check validation before marking pod ready
  - Configure deployment timeout (10 minutes)
  - Add deployment progress deadline
  - _Requirements: 25.2, 25.4_

- [ ] 13.5 Test zero-downtime deployment
  - Perform rolling update test with load
  - Verify no request failures during deployment (< 0.01% error rate)
  - Test rollback procedure
  - Measure deployment time
  - Validate session continuity during deployment
  - _Requirements: 8.2, 25.2_



- [ ] 14. Code Quality and Refactoring
- [ ] 14.1 Eliminate code duplication
  - Extract common authentication logic to services/auth_service.rs
  - Create reusable validation utilities in utils/validation.rs
  - Refactor error handling to use consistent AuthencError patterns
  - Extract common database query patterns
  - _Requirements: 2.1_

- [ ] 14.2 Improve module organization
  - Reorganize handlers by domain (handlers/auth/, handlers/authz/, handlers/mfa/, handlers/admin/)
  - Create clear separation between API handlers and business logic services
  - Add module-level documentation (//! comments) to all modules
  - Create services layer for business logic
  - _Requirements: 2.3_

- [ ] 14.3 Enhance error handling
  - Ensure all public functions return Result<T, AuthencError>
  - Add context to errors using anyhow::Context
  - Implement proper error propagation with ?
  - Add error codes to all error types
  - Improve error messages with actionable suggestions
  - _Requirements: 2.2_

- [ ] 14.4 Optimize dependency injection
  - Use Arc<T> for all shared state (already done for most)
  - Minimize cloning of large structures (use references where possible)
  - Implement lazy initialization for expensive resources
  - Review and optimize AppState structure
  - _Requirements: 2.4_

- [ ] 14.5 Run code quality checks
  - Run clippy with strict lints (--deny warnings)
  - Run rustfmt on all files
  - Check for unused dependencies with cargo-udeps
  - Run cargo audit for security vulnerabilities
  - Generate code quality report
  - _Requirements: 8.1_

- [ ] 15. Documentation and Developer Experience
- [ ] 15.1 Create OpenAPI/Swagger documentation
  - Add utoipa annotations to REST endpoint handlers
  - Generate OpenAPI spec file (openapi.json)
  - Setup Swagger UI endpoint at /api/docs
  - Add request/response examples to OpenAPI spec
  - Document authentication requirements
  - _Requirements: 12.1_

- [ ] 15.2 Enhance error responses
  - Add error codes to all AuthencError variants
  - Include suggested actions in error messages
  - Add trace_id to all error responses
  - Standardize error response format (JSON)
  - Document all error codes in API docs
  - _Requirements: 12.2_

- [ ] 15.3 Create integration guides
  - Write Portal SSO integration guide (docs/PORTAL_INTEGRATION.md)
  - Write Layanan gRPC integration guide (docs/LAYANAN_INTEGRATION.md)
  - Write Envoy ext_authz integration guide (docs/ENVOY_INTEGRATION.md)
  - Add code examples for each integration
  - _Requirements: 12.4_

- [ ] 15.4 Create example code
  - Add Rust HTTP client example (examples/http_client.rs)
  - Add Rust gRPC client example (examples/grpc_client.rs)
  - Add authentication flow example (examples/auth_flow.rs)
  - Add MFA setup example (examples/mfa_setup.rs)
  - _Requirements: 12.4_

- [ ] 15.5 Write comprehensive API documentation
  - Document all REST endpoints in docs/API.md
  - Document all gRPC methods in docs/GRPC_API.md
  - Document authentication flows (password, MFA, OAuth2, OIDC)
  - Document all error codes with descriptions
  - Add sequence diagrams for complex flows
  - _Requirements: 12.1_

- [x] 16. Configuration Management (MOSTLY COMPLETED)
- [x] 16.1 Basic configuration loading
  - ✓ Environment variable support
  - ✓ TOML config file support
  - ✓ Secreton integration for secrets
  - _Requirements: 7.1_

- [ ] 16.2 Enhance configuration validation
  - Validate all config on startup (database URL, Redis URL, ports)
  - Provide clear error messages for invalid config
  - Add config validation tests
  - Validate Secreton connectivity on startup
  - _Requirements: 7.3_

- [ ] 16.3 Create configuration templates
  - Create config/authenc.development.toml template
  - Create config/authenc.staging.toml template
  - Create config/authenc.production.toml template
  - Add comments explaining each configuration option
  - _Requirements: 7.4_

- [ ] 16.4 Implement hot-reload for non-critical settings
  - Add config reload endpoint (POST /api/v1/admin/config/reload)
  - Implement safe config updates (rate limits, log levels, feature flags)
  - Add config change audit logging
  - Add authentication and authorization for config reload
  - _Requirements: 7.2_

- [ ] 16.5 Test configuration management
  - Test config loading from environment variables
  - Test config loading from TOML files
  - Test config validation with invalid values
  - Test hot-reload functionality
  - Test config precedence (env > file > defaults)
  - _Requirements: 8.2_

- [ ] 17. Testing and Quality Assurance
- [ ] 17.1 Achieve 70% unit test coverage
  - Write unit tests for authentication service (login, logout, token validation)
  - Write unit tests for authorization service (permission checks, role assignment)
  - Write unit tests for MFA service (setup, verify, disable)
  - Write unit tests for cache layer (get, set, invalidate)
  - Write unit tests for JWT signing and validation
  - Run cargo tarpaulin for coverage report
  - _Requirements: 8.1_

- [ ] 17.2 Create integration test suite
  - Write database integration tests (user CRUD, session management)
  - Write gRPC integration tests (all RPC methods)
  - Write Kafka integration tests (event publishing, consumption)
  - Write Secreton integration tests (secret storage, retrieval)
  - Write Redis integration tests (cache operations)
  - Use testcontainers for isolated testing
  - _Requirements: 8.2_

- [ ] 17.3 Implement load testing with K6
  - Create K6 script for authentication flow (scripts/load-tests/auth.js)
  - Create K6 script for token validation (scripts/load-tests/validate.js)
  - Create K6 script for permission checks (scripts/load-tests/authz.js)
  - Run load tests with 100 VUs for 5 minutes
  - Generate load test report with P95, P99 latencies
  - Validate P95 < 100ms for authentication
  - _Requirements: 8.4, 1.1_

- [ ] 17.4 Run security testing
  - Perform penetration testing (SQL injection, XSS, CSRF)
  - Run OWASP Top 10 validation
  - Test for common vulnerabilities (weak passwords, session fixation)
  - Test JWT token manipulation attempts
  - Test rate limit bypass attempts
  - Generate security test report
  - _Requirements: 8.1_

- [ ] 17.5 Setup CI/CD pipeline enhancements
  - Configure automated testing on commit (GitLab CI)
  - Add code coverage reporting (codecov)
  - Add security scanning (cargo audit, trivy)
  - Configure automated deployment to staging
  - Add deployment approval for production
  - _Requirements: 8.5_

- [ ] 18. Final Integration and Validation
- [ ] 18.1 End-to-end testing
  - Test complete authentication flow (username/password + MFA)
  - Test Portal SSO integration (OIDC authorization code flow)
  - Test Layanan gRPC authorization (CheckPermission)
  - Test Envoy ext_authz integration (token validation)
  - Test session management and refresh
  - Test logout and session termination
  - _Requirements: 8.2_

- [ ] 18.2 Performance validation
  - Verify P95 latency < 100ms for authentication
  - Verify P95 latency < 50ms for token validation
  - Verify P95 latency < 20ms for permission checks
  - Verify cache hit ratio > 80%
  - Verify database connection pool utilization < 50%
  - Generate performance validation report
  - _Requirements: 1.1, 1.2, 1.3, 15.1, 15.2_

- [ ] 18.3 Security validation
  - Verify Ed25519 JWT signing is active
  - Verify Argon2 password hashing with cost >= 10
  - Verify rate limiting effectiveness (block after threshold)
  - Verify MFA secret encryption via Secreton
  - Verify no RSA usage (deprecated)
  - Verify HTTPS/TLS configuration
  - Generate security validation report
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [ ] 18.4 Compliance validation
  - Verify audit log completeness (all events logged)
  - Verify audit log integrity (signatures valid)
  - Verify GDPR compliance (data retention, right to erasure)
  - Verify ISO 27001 controls (access control, logging, encryption)
  - Generate compliance validation report
  - _Requirements: 10.1, 10.2, 10.3, 24.1, 24.2_

- [ ] 18.5 Production readiness review
  - Review all requirements fulfillment (checklist)
  - Review security hardening (penetration test results)
  - Review monitoring setup (metrics, alerts, dashboards)
  - Review documentation completeness (API docs, integration guides)
  - Review deployment strategy (zero-downtime, rollback)
  - Approve for production deployment
  - _Requirements: All_

## Task Execution Guidelines

### Current Implementation Status
**Completed Foundation**:
- ✅ Redis cache layer (RedisCache, MfaCache)
- ✅ Secreton integration (SecretonClient with circuit breaker)
- ✅ Kafka event publishing (KafkaEventListener)
- ✅ Ed25519 JWT signing (RSA deprecated)
- ✅ Argon2 password hashing
- ✅ Prepared statement cache
- ✅ Basic OIDC endpoints
- ✅ Istio gateway and virtual service

**Remaining Work**:
- 🔨 gRPC service implementation (Epic 1)
- 🔨 Multi-layer cache and metrics (Epic 2-3)
- 🔨 Performance optimization (Epic 6)
- 🔨 Microservice integration (Epic 9)
- 🔨 Observability enhancement (Epic 11)
- 🔨 Compliance and audit (Epic 12)

### Priority Levels
- **P0 (Critical)**: Epic 1 (gRPC), Epic 6 (Performance), Epic 7 (Security)
- **P1 (High)**: Epic 2-3 (Cache/DB), Epic 9 (Microservices), Epic 10 (Portal SSO)
- **P2 (Medium)**: Epic 11 (Observability), Epic 12 (Compliance), Epic 13 (Deployment)
- **P3 (Low)**: Epic 14 (Code Quality), Epic 15 (Documentation), Epic 16 (Config)
- **P4 (Validation)**: Epic 17 (Testing), Epic 18 (Final Validation)

### Execution Order
1. **Phase 1 (Weeks 1-2)**: Epic 1 (gRPC), Epic 2-3 (Cache/DB optimization)
2. **Phase 2 (Weeks 3-4)**: Epic 6 (Performance), Epic 9 (Microservices), Epic 10 (Portal SSO)
3. **Phase 3 (Weeks 5-6)**: Epic 11 (Observability), Epic 12 (Compliance), Epic 13 (Deployment)
4. **Phase 4 (Weeks 7-8)**: Epic 14-16 (Quality/Docs), Epic 17-18 (Testing/Validation)

### Testing Strategy
- Unit tests should be written alongside implementation
- Integration tests required for all external integrations (gRPC, Kafka, Secreton, Redis)
- Load tests must validate P95 latency requirements before production
- Security tests are mandatory for production readiness

### Rollback Plan
- Each epic should be implemented in a feature branch
- Merge to main only after code review and testing
- Tag each epic completion for easy rollback
- Maintain backward compatibility for all API changes
- Database migrations must be backward-compatible

