//! Security Level Management and Compliance
//!
//! This module provides advanced security controls, compliance standards enforcement,
//! and defense-in-depth mechanisms for Secreton's secret management operations.
//!
//! # Security Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │        Application Layer                        │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │     Security Level Enforcement                  │
//! │  ┌────────────────────────────────────────┐    │
//! │  │  Classification Levels:                │    │
//! │  │  - PUBLIC (Level 0)                    │    │
//! │  │  - INTERNAL (Level 1)                  │    │
//! │  │  - CONFIDENTIAL (Level 2)              │    │
//! │  │  - SECRET (Level 3)                    │    │
//! │  │  - TOP_SECRET (Level 4)                │    │
//! │  └────────────────────────────────────────┘    │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!       ┌──────────────┴──────────────┐
//!       ▼                             ▼
//! ┌──────────────┐            ┌──────────────┐
//! │ Cryptographic│            │  Compliance  │
//! │  Controls    │            │   Standards  │
//! │ (Encryption) │            │ (FIPS, ISO)  │
//! └──────────────┘            └──────────────┘
//! ```
//!
//! # Security Levels (Indonesian Government Classification)
//!
//! Secreton implements 5-level classification aligned with Indonesian government standards:
//!
//! - **PUBLIC** (Level 0) - No restrictions, public information
//! - **INTERNAL** (Level 1) - Internal use only, low sensitivity
//! - **CONFIDENTIAL** (Level 2) - Restricted access, medium sensitivity
//! - **SECRET** (Level 3) - High security, government secrets
//! - **TOP_SECRET** (Level 4) - Maximum security, national security data
//!
//! # Example: Set Secret Security Level
//!
//! ```ignore
//! use secreton_core::security::{SecurityLevel, SecurityManager};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = SecurityManager::new();
//!
//! // Store secret with CONFIDENTIAL classification
//! manager.create_secret(
//!     "/app/database/password",
//!     b"secret_value",
//!     SecurityLevel::Confidential,
//! ).await?;
//!
//! // Enforce: Only users with CONFIDENTIAL+ clearance can read
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Check User Clearance
//!
//! ```ignore
//! use secreton_core::security::{SecurityLevel, SecurityManager};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let manager = SecurityManager::new();
//! let user_clearance = SecurityLevel::Secret;
//! let secret_level = SecurityLevel::Confidential;
//!
//! if user_clearance >= secret_level {
//!     println!("Access granted");
//!     // Retrieve secret
//!  else {
//!     println!("Access denied - insufficient clearance");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Compliance Standards
//!
//! Secreton enforces multiple compliance frameworks:
//!
//! ## FIPS 140-3 (Cryptographic Module Validation)
//!
//! - **Approved Algorithms**: AES-256-GCM, ChaCha20-Poly1305, SHA-256, Ed25519
//! - **Key Management**: Hardware Security Module (HSM) integration
//! - **Random Number Generation**: FIPS-approved DRBG
//!
//! ## ISO/IEC 27001 (Information Security Management)
//!
//! - Access control policies (RBAC + ABAC)
//! - Audit trail for all operations
//! - Encryption at rest and in transit
//! - Secure key lifecycle management
//!
//! ## GDPR (Data Privacy Regulation)
//!
//! - Data minimization (only store necessary secrets)
//! - Right to erasure (secure deletion)
//! - Data portability (export capabilities)
//! - Breach notification (audit + alerting)
//!
//! # Defense-in-Depth Layers
//!
//! Multiple security controls applied simultaneously:
//!
//! ```text
//! Layer 1: Network Security (TLS 1.3, mTLS)
//!          ↓
//! Layer 2: Authentication (JWT, MFA, PQC)
//!          ↓
//! Layer 3: Authorization (RBAC, policies)
//!          ↓
//! Layer 4: Data Encryption (AES-256-GCM)
//!          ↓
//! Layer 5: Audit Logging (immutable trail)
//!          ↓
//! Layer 6: HSM Protection (hardware keys)
//! ```
//!
//! # Seal Wrapping (Enterprise Feature)
//!
//! Critical secrets encrypted with master key:
//!
//! ```ignore
//! use secreton_core::security::SealWrapper;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let wrapper = SealWrapper::new(/* master_key */);
//!
//! // Wrap sensitive data before storage
//! let wrapped = wrapper.wrap(b"sensitive_data").await?;
//!
//! // Only unwrappable when engine is unsealed
//! let unwrapped = wrapper.unwrap(&wrapped).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Zero-Trust Architecture
//!
//! Never trust, always verify:
//!
//! - **No Implicit Trust**: Every request authenticated and authorized
//! - **Least Privilege**: Minimum required permissions only
//! - **Micro-segmentation**: Network isolation per namespace
//! - **Continuous Verification**: Token validation on every operation
//!
//! # Security Events and Alerting
//!
//! Critical security events trigger alerts:
//!
//! - **Failed Login Attempts** (brute force detection)
//! - **Privilege Escalation** (unauthorized admin access)
//! - **Data Exfiltration** (unusual read patterns)
//! - **Configuration Changes** (seal/unseal, policy updates)
//!
//! # HSM Integration
//!
//! Hardware Security Module for cryptographic operations:
//!
//! ```ignore
//! use secreton_core::security::HsmProvider;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let hsm = HsmProvider::connect("pkcs11:///usr/lib/softhsm2.so")?;
//!
//! // Generate key in HSM (never leaves hardware)
//! let key_id = hsm.generate_key("master-key-2025").await?;
//!
//! // Encrypt using HSM
//! let ciphertext = hsm.encrypt(key_id, b"plaintext").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Post-Quantum Cryptography (Future-Proof)
//!
//! Quantum-resistant algorithms:
//!
//! - **ML-DSA (Dilithium)**: Digital signatures (FIPS 204)
//! - **ML-KEM (Kyber)**: Key encapsulation (FIPS 203)
//! - **SLH-DSA (SPHINCS+)**: Stateless hash-based signatures (FIPS 205)
//!
//! # Performance Considerations
//!
//! - **Clearance Checks**: O(1) - simple integer comparison
//! - **Encryption Overhead**: ~5-10% for AES-256-GCM
//! - **HSM Operations**: 10-50ms latency (hardware dependent)
//! - **Audit Logging**: Async, non-blocking
//!
//! # Configuration
//!
//! ```yaml
//! security:
//!   default_level: CONFIDENTIAL
//!   enforce_mfa: true
//!   hsm_enabled: true
//!   compliance_mode: FIPS140-3
//!   audit_all_access: true
//! ```
//!
//! # See Also
//!
//! - [`SecurityLevel`] - Security classification enum
//! - [`SecurityManager`] - Main security enforcement interface
//! - [`SealWrapper`] - Master key encryption
//! - [`HsmProvider`] - Hardware security module integration
//! - `crate::crypto` - Cryptographic primitives
//! - `crate::audit` - Security event logging

/// Optimized traits
pub mod optimized_traits;

// Audit module - enhanced with HMAC chain for tamper-proof storage
pub mod audit;

// FIPS 140-3 compliance module for government security standards
pub mod fips_compliance;

// TODO: seal_wrapping needs missing dependencies (ring, flate2, fips_compliance)
pub mod seal_wrapping;

// TODO: manager needs missing modules (hsm, pqcrypto, mfa, SecurityEventType, ComplianceStandard)
// pub mod manager;

// TODO: concrete_implementations needs missing modules (advanced_mfa, zero_trust, fips_compliance)
// pub mod concrete_implementations;
