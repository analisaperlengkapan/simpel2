# Implementation Plan

- [x] 1. Create MFA Service Wrapper (Leveraging Existing Infrastructure)
  - Create wrapper service that uses existing `OtpCredentialProvider` and secreton `MfaManager`
  - Add MFA fields to user model and database schema
  - Integrate with existing authentication flow and audit logging
  - _Requirements: 3.1, 4.1, 6.1_

- [x] 1.1 Create MFA Service Wrapper
  - Create `infra/authenc/src/services/mfa_service.rs` that wraps existing `OtpCredentialProvider`
  - Implement integration with existing secreton client for encrypted secret storage
  - Add QR code generation using existing TOTP provisioning URI logic
  - _Requirements: 3.1, 4.1_

- [x] 1.2 Enhance User Model for MFA
  - Add `mfa_enabled` and `mfa_setup_at` fields to existing `User` struct in `infra/authenc/src/models/user.rs`
  - Create `MfaSetupData` and `MfaStatus` structs for API responses
  - Update database schema with new MFA fields and indexes
  - _Requirements: 3.1, 11.1_

- [x] 1.3 Enhance Error Handling for MFA
  - Add MFA-specific error variants to existing `AuthencError` enum in `infra/authenc/src/error.rs`
  - Implement proper error categorization and security event logging
  - Add error recovery mechanisms for MFA operations
  - _Requirements: 8.1, 11.1_

- [x] 2. Create Portal MFA Pages
  - Implement MFA setup and verification pages using Leptos 0.8.x
  - Integrate with existing shared component library for consistent UI
  - Add proper routing and navigation flow for MFA process
  - _Requirements: 1.1, 1.2, 5.1_

- [x] 2.1 Implement MFA Setup Page
  - Create `antarmuka/portal/src/pages/mfa_setup.rs` with QR code display and OTP verification
  - Use existing shared components (`Button`, `Input`) for consistent styling
  - Implement API calls using `gloo-net` for setup and verification
  - _Requirements: 1.1, 5.1_

- [x] 2.2 Implement MFA Verification Page
  - Create `antarmuka/portal/src/pages/mfa_verification.rs` for login-time OTP verification
  - Add rate limiting display and error handling for failed attempts
  - Implement proper navigation flow to dashboard after successful verification
  - _Requirements: 1.2, 5.1, 8.1_

- [x] 2.3 Create Reusable OTP Components
  - Add `OtpInput` component to `antarmuka/shared/src/components/forms.rs`
  - Add `QrCodeDisplay` component to `antarmuka/shared/src/components/display.rs`
  - Ensure components follow existing design system and accessibility standards
  - _Requirements: 5.1, 12.1_

- [x] 3. Enhance Authentication Flow Integration
  - Modify existing login handlers to check MFA status and redirect appropriately
  - Update session management to handle MFA verification states
  - Integrate with existing audit logging system for MFA events
  - _Requirements: 2.1, 6.1, 11.1_

- [x] 3.1 Update Login Handler
  - Modify existing login logic in `infra/authenc/src/handlers/` to check MFA status
  - Implement temporary session creation for users requiring MFA verification
  - Add proper redirect logic for first-time MFA setup vs. verification
  - _Requirements: 2.1, 6.1_

- [x] 3.2 Enhance Session Management
  - Update existing session management to handle MFA verification states
  - Implement session upgrade from temporary to full session after MFA verification
  - Add MFA status to session data for authorization checks
  - _Requirements: 6.1, 6.2_

- [x] 3.3 Integrate Audit Logging
  - Use existing audit logging infrastructure to log MFA setup and verification events
  - Add MFA-specific audit event types to existing audit system
  - Implement security event correlation fors MFA activity
  - _Requirements: 11.1, 11.2_

- [x] 4. Secreton Integration (Using Existing MfaManager)
  - Integrate with existing secreton `MfaManager` and `EnterpriseMfaManager`
  - Use existing secreton API endpoints for MFA secret storage
  - Leverage existing key rotation and audit capabilities
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 4.1 Integrate with Existing Secreton MfaManager
  - Use existing secreton MfaManager via API calls from authenc
  - Implement proper authentication for authenc-to-secreton MFA operations
  - Add error handling for secreton MFA API responses
  - _Requirements: 4.1, 4.2_

- [x] 4.2 Configure Secreton MFA Policies
  - Configure existing secreton MfaManager for government employee requirements
  - Set up proper access controls for MFA secret storage
  - Configure audit logging for MFA operations in secreton
  - _Requirements: 4.2, 4.3_

- [x] 5. API Endpoints Implementation
  - Create REST API endpoints for MFA setup and verification
  - Integrate with existing authentication middleware and rate limiting
  - Add proper request validation and response formatting
  - _Requirements: 1.1, 1.2, 10.1_

- [x] 5.1 Create MFA Setup Endpoints
  - Add `/api/auth/mfa/setup` endpoint for generating QR codes and secrets
  - Add `/api/auth/mfa/verify-setup` endpoint for initial OTP verification
  - Implement proper authentication and authorization checks
  - _Requirements: 1.1, 10.1_

- [x] 5.2 Create MFA Verification Endpoints
  - Add `/api/auth/mfa/verify` endpoint for login-time OTP verification
  - Add `/api/auth/mfa/status` endpoint for checking user MFA status
  - Implement rate limiting and brute force protection
  - _Requirements: 1.2, 6.3, 10.1_

- [x] 5.3 Add MFA Management Endpoints
  - Add `/api/auth/mfa/disable` endpoint for disabling MFA (admin only)
  - Add `/api/auth/mfa/reset` endpoint for resetting MFA configuration
  - Add `/api/auth/mfa/backup-codes` endpoint for managing backup codes
  - _Requirements: 9.1, 9.2_

- [x] 6. Rate Limiting and Security Enhancement
  - Implement MFA-specific rate limiting using existing infrastructure
  - Add brute force protection for OTP verification attempts
  - Implement account lockout policies for repeated MFA failures
  - _Requirements: 6.3, 6.4, 8.1_

- [x] 6.1 Implement MFA Rate Limiting
  - Add MFA-specific rate limiting rules to existing rate limiting system
  - Implement progressive delays for repeated failed OTP attempts
  - Add IP-based rate limiting for MFA endpoints
  - _Requirements: 6.3, 8.1_

- [x] 6.2 Add Account Lockout Protection
  - Implement account lockout after multiple failed MFA attempts
  - Add automatic unlock after specified time period
  - Implement admin override for locked accounts
  - _Requirements: 6.4, 8.1, 9.2_

- [x] 6.3 Enhance Security Monitoring
  - Add MFA-specific security event detection to existing monitoring system
  - Implement anomaly detection for unusual MFA patterns
  - Add alerting for suspicious MFA activity
  - _Requirements: 8.1, 11.3_

- [x] 7. Validate TOTP Standards Compliance (Using Existing Implementation)
  - Verify existing `OtpCredentialProvider` follows RFC 6238 standards
  - Ensure existing TOTP implementation has proper time synchronization
  - Validate compatibility with standard authenticator applications
  - _Requirements: 7.1, 7.2, 7.3_

- [x] 7.1 Validate Existing TOTP Implementation
  - Review existing `OtpCredentialProvider` for RFC 6238 compliance
  - Verify 30-second time steps, 6-digit codes, and HMAC-SHA1 algorithm
  - Test clock skew tolerance (±1 time window) implementation
  - _Requirements: 7.1, 7.2_

- [x] 7.2 Test Authenticator App Compatibility
  - Test existing QR code generation with Google Authenticator, Microsoft Authenticator, FreeOTP, and Authy
  - Verify existing TOTP URI format includes proper issuer and account information
  - Ensure manual secret key entry works with existing implementation
  - _Requirements: 7.3, 8.2_

- [x] 7.3 Enhance Backup Code System
  - Use existing backup code generation in secreton MfaManager
  - Integrate backup code verification with authenc authentication flow
  - Add proper UI for backup code display and usage
  - _Requirements: 7.4, 8.2_

- [x] 8. Performance Optimization and Caching
  - Implement caching for MFA status and recent verification results
  - Optimize database queries for MFA operations
  - Add performance monitoring for MFA endpoints
  - _Requirements: 10.1, 10.2, 10.3_

- [x] 8.1 Implement MFA Caching Strategy
  - Cache user MFA status in Redis to reduce database queries
  - Implement cache invalidation on MFA status changes
  - Add caching for recent OTP verification results to prevent replay attacks
  - _Requirements: 10.1, 10.4_

- [x] 8.2 Optimize Database Operations
  - Add proper indexes for MFA-related database queries
  - Optimize user lookup queries that include MFA status
  - Implement connection pooling optimization for MFA operations
  - _Requirements: 10.2, 10.4_

- [x] 8.3 Add Performance Monitoring
  - Add metrics collection for MFA setup and verification times
  - Implement performance alerts for slow MFA operations
  - Add dashboard monitoring for MFA success/failure rates
  - _Requirements: 10.3, 10.5_

- [x] 9. Administrative Management Features
  - Implement admin interfaces for managing user MFA settings
  - Add reporting capabilities for MFA adoption and usage
  - Create admin tools for troubleshooting MFA issues
  - _Requirements: 9.1, 9.2, 9.3_

- [x] 9.1 Create Admin MFA Management Interface
  - Add admin endpoints for viewing user MFA status across organization
  - Implement admin capability to reset user MFA settings with proper authorization
  - Add bulk MFA operations for organizational management
  - _Requirements: 9.1, 9.2_

- [x] 9.2 Implement MFA Reporting System
  - Create reports for MFA adoption rates by satker/wilayah/pusat
  - Add compliance reporting for security audits
  - Implement trend analysis for MFA usage patterns
  - _Requirements: 9.3, 11.4_

- [x] 9.3 Add MFA Troubleshooting Tools
  - Create admin tools for diagnosing MFA setup issues
  - Add user self-service options for common MFA problems
  - Implement guided troubleshooting workflows
  - _Requirements: 8.3, 9.4_

- [x] 10. Testing and Quality Assurance
  - Create comprehensive test suite for MFA functionality
  - Implement integration tests for authenc-secreton MFA communication
  - Add end-to-end tests for complete MFA user flows
  - _Requirements: 12.1, 12.2, 12.3_

- [x] 10.1 Create Unit Tests for MFA Components
  - Add unit tests for MFA service wrapper methods
  - Test integration with existing OtpCredentialProvider
  - Add tests for error handling and edge cases
  - _Requirements: 12.1, 12.5_

- [x] 10.2 Implement Integration Tests
  - Enhance existing MFA tests in secreton to include authenc integration
  - Test MFA setup and verification flows end-to-end using existing test infrastructure
  - Add tests for rate limiting and security features using existing test utilities
  - _Requirements: 12.2, 12.4_

- [x] 10.3 Add Frontend Component Tests
  - Create tests for MFA setup and verification page components
  - Test OTP input component functionality and accessibility
  - Add visual regression tests for QR code display
  - _Requirements: 12.3, 12.5_

- [x] 11. Documentation and User Guides
  - Create user documentation for MFA setup and usage
  - Add administrator guides for MFA management
  - Create troubleshooting documentation for common issues
  - _Requirements: 8.4, 9.5, 12.4_

- [x] 11.1 Create User Documentation
  - Write step-by-step guide for MFA setup process
  - Create documentation for using different authenticator apps
  - Add troubleshooting guide for common user issues
  - _Requirements: 8.4, 12.4_

- [x] 11.2 Create Administrator Documentation
  - Write admin guide for managing organizational MFA policies
  - Create documentation for MFA reporting and monitoring
  - Add security best practices guide for MFA deployment
  - _Requirements: 9.5, 11.5_

- [x] 11.3 Add Technical Documentation
  - Document MFA API endpoints and integration patterns
  - Create architecture documentation for MFA implementation
  - Add security considerations and compliance documentation
  - _Requirements: 12.4, 11.5_

- [-] 12. Security Validation and Compliance
  - Conduct security review of MFA implementation
  - Validate compliance with government security standards
  - Perform penetration testing of MFA functionality
  - _Requirements: 11.4, 11.5, 12.5_

- [x] 12.1 Conduct Security Code Review
  - Review MFA implementation for security vulnerabilities
  - Validate cryptographic implementations and key management
  - Ensure proper input validation and sanitization
  - _Requirements: 11.4, 12.5_

- [x] 12.2 Validate Government Compliance
  - Ensure MFA implementation meets Indonesian government security standards
  - Validate audit logging meets compliance requirements
  - Review data protection and privacy implementations
  - _Requirements: 11.5, 12.5_

- [ ] 12.3 Perform Security Testing
  - Conduct penetration testing of MFA endpoints and flows
  - Test for timing attacks and other cryptographic vulnerabilities
  - Validate rate limiting and brute force protection effectiveness
  - _Requirements: 12.5_
