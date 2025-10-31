# Implementation Plan

- [x] 1. Code Analysis and Cleanup
  - Analyze existing cryptographic implementations in both authenc and secreton
  - Identify duplicate functions and redundant code
  - Create inventory of files to be enhanced vs. removed
  - _Requirements: 1.1, 1.2_

- [x] 1.1 Authenc Code Cleanup
  - Remove duplicate AES-GCM implementations, keep most performant version
  - Consolidate error types in `infra/authenc/src/error.rs`
  - Remove unused vault provider files (keep only secreton integration)
  - Clean up deprecated authentication handlers
  - _Requirements: 1.1, 6.1_

- [x] 1.2 Secreton Code Cleanup
  - Remove redundant storage backend implementations
  - Consolidate crypto modules in `infra/secreton/crates/crypto/src/`
  - Remove unused engine types and implementations
  - Clean up duplicate audit modules
  - _Requirements: 1.1, 6.2_

- [x] 2. Enhanced Cryptographic Engine Implementation
  - Enhance existing crypto modules with SIMKARI-specific operations
  - Implement hybrid post-quantum cryptography support
  - Add performance monitoring for cryptographic operations
  - _Requirements: 1.3, 5.1, 5.2_

- [x] 2.1 Authenc Crypto Enhancement
  - Enhance `infra/authenc/src/crypto/mod.rs` with JWT signing for pegawai
  - Add batch token validation capabilities
  - Implement session data encryption for SIMKARI users
  - Add audit signature generation for compliance
  - _Requirements: 1.3, 5.1_

- [x] 2.2 Secreton Hybrid Crypto Implementation
  - Create `infra/secreton/crates/crypto/src/hybrid.rs` module with actual implementation
  - Implement `HybridCrypto` struct with Classical, Hybrid, and PostQuantum modes
  - Add `SecurityRequirements` and `PerformancePriority` configuration
  - Implement hybrid encryption combining AES-256-GCM with ML-KEM
  - Implement hybrid signatures combining Ed25519 with ML-DSA
  - Add migration strategy support for gradual PQ transition
  - Export hybrid module from `infra/secreton/crates/crypto/src/lib.rs`
  - Replace mock implementations in tests with actual HybridCrypto
  - _Requirements: 1.3, 5.2, 5.3_

- [x] 3. Role-Based Access Control System
  - Implement hierarchical role management for Attorney General's Office
  - Create admin levels: AdminSatker, AdminWilayah, AdminEselonI, AdminPusat
  - Implement role scopes: Satker, Wilayah, Pusat
  - _Requirements: 3.1, 3.2, 7.1_

- [x] 3.1 User Model Enhancement
  - Enhance `infra/authenc/src/models/user.rs` with satker_code field
  - Add Role and AdminLevel enums with hierarchical structure
  - Implement SecretonAccessPolicy with role-based permissions
  - Remove BMN-specific fields, make system flexible for any resource type
  - _Requirements: 3.1, 7.1_

- [x] 3.2 Token Model Enhancement
  - Enhance `infra/authenc/src/models/token.rs` with satker_code
  - Update SecretonPermissions to be role-based rather than resource-specific
  - Add support for hierarchical admin operations
  - Implement flexible scope system
  - _Requirements: 3.1, 7.1_

- [x] 4. Secreton Integration Client
  - Rename `secreton_vault.rs` to `secreton_client.rs`
  - Enhance client with SIMKARI-specific operations
  - Add post-quantum key retrieval capabilities
  - Implement circuit breaker pattern for reliability
  - _Requirements: 3.3, 4.1, 6.3_

- [x] 4.1 Client Implementation
  - Enhance `infra/authenc/src/vault/secreton_client.rs` (renamed from secreton_vault.rs)
  - Add methods for application config retrieval
  - Implement user secret access validation
  - Add post-quantum key operations
  - _Requirements: 3.3, 4.1_

- [x] 4.2 Authentication Provider
  - Create `infra/secreton/crates/core/src/auth/authenc_provider.rs`
  - Implement user authentication with flexible credentials
  - Add token validation with post-quantum signature support
  - Implement resource permission checking based on roles
  - _Requirements: 3.4, 4.1_

- [x] 5. Enhanced Secret Management
  - Create flexible secret engine for SIMKARI operations
  - Implement satker-based secret organization
  - Add audit trail for all secret operations
  - _Requirements: 3.2, 4.2_

- [x] 5.1 Secret Engine Enhancement
  - Create `infra/secreton/crates/core/src/engines/enhanced.rs`
  - Implement application-specific secret retrieval
  - Add batch operations for performance
  - Integrate post-quantum encryption for sensitive secrets
  - _Requirements: 3.2, 5.1, 5.2_

- [x] 5.2 Secret Model Enhancement
  - Enhance `infra/secreton/crates/core/src/models/secret.rs`
  - Replace instansi_owner with satker_owner
  - Remove BMN-specific fields, make access control role-based
  - Add flexible metadata system
  - _Requirements: 3.2, 7.1_

- [x] 6. Error Handling Enhancement
  - Enhance error types in both projects for better integration
  - Add SIMKARI-specific error variants
  - Implement consistent error categorization
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 6.1 Authenc Error Enhancement
  - Enhance `infra/authenc/src/error.rs` with secreton integration errors
  - Add SecretonCommunicationError, SecretAccessDenied, SecretonAuthenticationFailed variants
  - Add satker-specific error variants for access control
  - Implement error retry logic helpers for secreton communication
  - Add compliance violation error types for audit requirements
  - _Requirements: 2.1, 2.3_

- [x] 6.2 Secreton Error Enhancement
  - Add authenc integration errors to `infra/secreton/crates/core/src/error.rs`
  - Add AuthencTokenValidationFailed error variant
  - Add IamPermissionDenied error variant for role-based access denial
  - Add AuthencAuthenticationFailed error variant
  - Add AuthencCommunicationTimeout error variant
  - Add post-quantum specific error types (PqKeyValidationFailed, PqSignatureVerificationFailed)
  - Add audit compliance error variants for kejaksaan operations
  - Implement helper methods: is_authenc_related(), requires_reauthentication()
  - _Requirements: 2.2, 2.3_

- [x] 7. Audit and Compliance System
  - Enhance audit models for Attorney General's Office compliance
  - Implement hierarchical audit trail
  - Add compliance flag system
  - _Requirements: 4.3, 7.2_

- [x] 7.1 Audit Model Enhancement
  - Enhance `infra/secreton/crates/core/src/models/audit.rs`
  - Add satker_code and admin_level fields
  - Implement compliance flags for kejaksaan operations
  - Add risk scoring for security operations
  - _Requirements: 4.3, 7.2_

- [x] 8. Post-Quantum Cryptography Integration
  - Implement hybrid cryptographic system with actual algorithms
  - Add ML-DSA and ML-KEM algorithm support
  - Create migration path from classical to post-quantum
  - _Requirements: 5.3, 5.4_

- [x] 8.1 Hybrid Crypto Module Implementation (Same as 2.2)
  - This task is covered by task 2.2 - Secreton Hybrid Crypto Implementation
  - _Requirements: 5.3, 5.4_

- [x] 8.2 Post-Quantum Key Management Integration
  - Create `infra/secreton/crates/crypto/src/pq_key_management.rs` with actual implementation
  - Implement PostQuantumKeyManager with ML-KEM and ML-DSA key generation
  - Integrate ML-KEM key encapsulation in EnhancedSecretEngine
  - Add ML-DSA signature support for authentication tokens in authenc
  - Implement hybrid key exchange (X25519 + ML-KEM) in SecretonClient
  - Add post-quantum encryption options for long-term secret storage
  - Update SecretonClient to request PQ keys from secreton
  - Replace mock key generation in tests with actual cryptographic implementations
  - _Requirements: 5.3, 5.4_

- [x] 9. Performance Optimization
  - Implement dynamic configuration system
  - Add intelligent caching with TTL
  - Optimize memory usage and CPU performance
  - _Requirements: 5.1, 5.5, 6.4_

- [x] 9.1 Dynamic Configuration
  - Create `DynamicConfig` struct with runtime adaptability
  - Implement load-based configuration adjustment
  - Add threat-level based security posture updates
  - Create performance profiling system
  - _Requirements: 5.5, 6.4_

- [x] 9.2 Caching and Memory Optimization
  - Implement LRU cache for frequently accessed secrets
  - Add connection pooling for HTTP communications
  - Implement proper zeroization of sensitive data
  - Add lazy loading for cryptographic contexts
  - _Requirements: 5.1, 5.5_

- [x] 10. Testing and Validation
  - Update test suite to use actual implementations instead of mocks
  - Add performance benchmarks for new features
  - Implement integration tests for authenc-secreton communication
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 10.1 Hybrid Crypto Unit Tests Update
  - Update `infra/secreton/tests/hybrid_crypto_tests.rs` to use actual HybridCrypto implementation
  - Replace all mock structures with real implementations from hybrid.rs
  - Test actual hybrid encryption and decryption with real algorithms
  - Test actual hybrid signature generation and verification with Ed25519 + ML-DSA
  - Test real migration strategy execution with actual key transitions
  - Verify proper algorithm selection based on security requirements
  - Remove mock helper functions and use actual crypto operations
  - _Requirements: 6.1, 6.2_

- [x] 10.2 Integration Testing Implementation
  - Create `infra/secreton/tests/integration/comprehensive_authenc_integration.rs`
  - Implement authenc-secreton integration tests with actual services
  - Test post-quantum key retrieval from secreton to authenc
  - Test hybrid encryption for cross-satker secret access
  - Validate PQ signature verification in token validation
  - Test error handling for authenc communication failures
  - _Requirements: 6.3_

- [x] 10.3 Performance Benchmarking
  - Update `infra/authenc/benches/performance.rs` with PQ operations
  - Update `infra/secreton/benches/performance.rs` with hybrid crypto benchmarks
  - Benchmark Classical vs Hybrid vs PostQuantum mode performance with real algorithms
  - Measure overhead of hybrid signatures vs classical
  - Create load testing for hierarchical operations with PQ
  - _Requirements: 6.4_

- [ ] 11. Documentation and Migration
  - Create comprehensive documentation for hybrid crypto functionality
  - Create migration guides for new features
  - Document role hierarchy and admin levels
  - _Requirements: 8.1, 8.2, 8.3_

- [ ] 11.1 API Documentation
  - Enhance module-level documentation in `infra/secreton/crates/crypto/src/hybrid.rs` with more examples
  - Add comprehensive doc comments to `infra/authenc/src/crypto/enhanced.rs` public methods
  - Document CryptoMode selection strategy and when to use each mode
  - Create migration guide document from classical to hybrid/PQ modes in `infra/secreton/docs/PQ_MIGRATION_GUIDE.md`
  - Document SecretonClient PQ key retrieval methods with usage examples
  - Add usage examples for ML-KEM and ML-DSA operations in doc comments
  - Document PostQuantumKeyManager API in `infra/secreton/crates/crypto/src/pq_key_management.rs`
  - _Requirements: 8.1, 8.2_

- [ ] 11.2 Architecture Documentation
  - Create `infra/secreton/docs/HYBRID_CRYPTO_ARCHITECTURE.md`
  - Document hybrid cryptography architecture with Mermaid diagrams
  - Create diagrams showing classical + PQ algorithm combinations
  - Document post-quantum migration strategy and phases
  - Add performance characteristics comparison (Classical vs Hybrid vs PQ)
  - Document security considerations for Attorney General's Office
  - Include integration patterns between authenc and secreton for PQ operations
  - Document EnhancedCryptoEngine architecture and usage patterns
  - _Requirements: 8.3, 8.4_

- [ ] 11.3 Integration Documentation
  - Create `infra/authenc/docs/SECRETON_INTEGRATION.md`
  - Document SecretonClient usage patterns and best practices
  - Add examples for signing key retrieval and encryption key operations
  - Document circuit breaker configuration and behavior
  - Add troubleshooting guide for common integration issues
  - Document hybrid key exchange flow between authenc and secreton
  - _Requirements: 8.1, 8.2_

- [ ] 12. Security Validation and Compliance
  - Validate zero-trust architecture maintenance with actual implementations
  - Ensure no shared dependencies between projects
  - Verify post-quantum readiness with real algorithms
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 12.1 Security Architecture Validation
  - Create security validation test suite in `infra/secreton/tests/security_validation.rs`
  - Verify HybridCrypto maintains zero-trust principles with actual crypto operations
  - Test that authenc and secreton remain independently deployable with PQ
  - Validate mTLS communication works with hybrid signatures
  - Ensure audit trail captures PQ algorithm usage (algorithm type, key IDs, timestamps)
  - Test secret isolation between satker with PQ encryption
  - Verify no shared cryptographic keys or dependencies between services
  - Test EnhancedCryptoEngine security properties (constant-time operations, key zeroization)
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 12.2 Compliance Verification
  - Create compliance test suite in `infra/authenc/tests/compliance_validation.rs`
  - Validate hybrid crypto meets Attorney General's Office requirements
  - Test hierarchical access control with PQ signatures
  - Verify audit logs properly record PQ operations (mode, algorithms, satker context)
  - Ensure role-based authorization works with hybrid tokens
  - Test migration path doesn't break existing functionality
  - Validate backward compatibility with classical-only deployments
  - Test SecretonClient circuit breaker behavior under failure conditions
  - Verify EnhancedCryptoEngine metrics collection and reporting
  - _Requirements: 4.4, 7.2_

- [ ] 13. Final Integration and Validation
  - Perform end-to-end integration testing of all components
  - Validate performance meets requirements
  - Ensure all documentation is complete and accurate
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 13.1 End-to-End Integration Tests
  - Create comprehensive integration test in `infra/authenc/tests/e2e_secreton_integration.rs`
  - Test complete flow: pegawai authentication → JWT signing → secreton secret retrieval
  - Test hybrid key exchange between authenc and secreton
  - Test post-quantum token signing and verification across services
  - Test circuit breaker behavior during secreton outages
  - Test session encryption/decryption with secreton-provided keys
  - Test audit signature generation and verification with PQ algorithms
  - _Requirements: 6.3_

- [ ] 13.2 Performance Validation
  - Run performance benchmarks for all new features
  - Compare Classical vs Hybrid vs PostQuantum mode performance
  - Measure overhead of circuit breaker and retry logic
  - Validate performance meets Attorney General's Office requirements
  - Document performance characteristics and recommendations
  - Create performance tuning guide for production deployment
  - _Requirements: 6.4_

- [ ] 13.3 Documentation Review and Completion
  - Review all API documentation for completeness and accuracy
  - Ensure all public methods have comprehensive doc comments
  - Verify all architecture diagrams are up-to-date
  - Complete migration guides with step-by-step instructions
  - Add troubleshooting sections to all documentation
  - Create quick-start guide for developers
  - _Requirements: 8.1, 8.2, 8.3_
