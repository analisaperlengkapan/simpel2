//! Hardware Security Module (HSM) Integration
//!
//! This module provides integration with Hardware Security Modules for hardware-backed
//! cryptographic operations, ensuring that sensitive keys never exist in software memory.
//!
//! # HSM Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │        Secreton Application                     │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │         HSM Backend (PKCS#11)                   │
//! │  ┌────────────────────────────────────────┐     │
//! │  │  Operations:                           │     │
//! │  │  - Key generation (never exported)     │     │
//! │  │  - Encryption/Decryption               │     │
//! │  │  - Digital signatures                  │     │
//! │  │  - Key derivation                      │     │
//! │  └────────────────────────────────────────┘     │
//! └────────────────────┬────────────────────────────┘
//!                      │ PKCS#11 API
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │     Hardware Security Module (HSM)              │
//! │  ┌────────────────────────────────────────┐     │
//! │  │  Tamper-Resistant Hardware             │     │
//! │  │  - Secure key storage                  │     │
//! │  │  - Hardware cryptographic accelerator  │     │
//! │  │  - Physical intrusion detection        │     │
//! │  │  - FIPS 140-3 Level 3+ certified       │     │
//! │  └────────────────────────────────────────┘     │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! # Supported HSM Types
//!
//! - **Thales Luna** - Enterprise-grade HSM (FIPS 140-2 Level 3)
//! - **Utimaco SecurityServer** - German HSM manufacturer
//! - **AWS CloudHSM** - Cloud-based HSM service
//! - **SoftHSM2** - Software emulation for development/testing
//! - **YubiHSM2** - USB-based HSM for smaller deployments
//!
//! # PKCS#11 Standard
//!
//! Secreton uses PKCS#11 (Cryptoki) standard for HSM communication:
//!
//! - **Industry standard** for cryptographic token interface
//! - **Vendor neutral** - works with any PKCS#11-compliant HSM
//! - **Well-documented** API with broad language support
//!
//! # Example: Initialize HSM Connection
//!
//! ```rust,no_run
//! use secreton_hsm::{HsmBackend, HsmConfig};
//! use secreton_hsm::config::HsmProvider;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = HsmConfig {
//!     enabled: true,
//!     provider: HsmProvider::Pkcs11,
//!     pkcs11_library_path: Some(PathBuf::from("/usr/lib/softhsm/libsofthsm2.so")),
//!     slot_id: Some(0),
//!     token_label: Some("secreton".to_string()),
//!     pin: Some("1234".to_string()), // From Secreton, never hard-coded
//!     ..Default::default()
//! };
//!
//! let hsm = HsmBackend::new(config)?;
//! hsm.initialize().await?;
//! println!("HSM connected successfully");
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Generate Master Key in HSM
//!
//! ```rust,no_run
//! use secreton_hsm::HsmBackend;
//!
//! # async fn example(hsm: HsmBackend) -> Result<(), Box<dyn std::error::Error>> {
//! // Generate an AES-256 master key. HSM keys are never exportable —
//! // `HsmKeyMetadata::exportable` is fixed at false by the backend.
//! let key = hsm
//!     .generate_hsm_key("master-key-2025", "AES", 256, vec!["encrypt".to_string()])
//!     .await?;
//!
//! println!("Master key generated in HSM: {}", key.key_id);
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Encrypt with HSM Key
//!
//! ```rust,no_run
//! use secreton_hsm::HsmBackend;
//!
//! # async fn example(hsm: HsmBackend) -> Result<(), Box<dyn std::error::Error>> {
//! let plaintext = b"Top secret data";
//!
//! // Both directions run inside the HSM; keys are addressed by id, not handle.
//! let ciphertext = hsm.hsm_encrypt("master-key-2025", plaintext).await?;
//! let decrypted = hsm.hsm_decrypt("master-key-2025", &ciphertext).await?;
//!
//! assert_eq!(plaintext, decrypted.as_slice());
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Digital Signature with HSM
//!
//! ```rust,no_run
//! use secreton_hsm::HsmBackend;
//!
//! # async fn example(hsm: HsmBackend) -> Result<(), Box<dyn std::error::Error>> {
//! // A signing key is an ordinary HSM key declared with a "sign" usage.
//! hsm.generate_hsm_key("api-signing-key", "Ed25519", 256, vec!["sign".to_string()])
//!     .await?;
//!
//! let message = b"Important document";
//!
//! // Signing happens inside the HSM; the private key never leaves it.
//! let signature = hsm.hsm_sign("api-signing-key", message, "Ed25519").await?;
//! assert!(!signature.is_empty());
//! # Ok(())
//! # }
//! ```
//!
//! # Key Lifecycle Management
//!
//! ```text
//! 1. Generation   → Key created inside HSM
//! 2. Activation   → Key ready for use
//! 3. Usage        → Encryption/signing operations
//! 4. Rotation     → New key generated, old key archived
//! 5. Deactivation → Key marked for deletion
//! 6. Destruction  → Secure erasure from HSM
//! ```
//!
//! # Security Properties
//!
//! ## FIPS 140-3 Compliance
//!
//! - **Level 3**: Physical tamper detection and response
//! - **Level 4**: Active intrusion detection with automatic zeroization
//!
//! ## Key Protection
//!
//! - Keys generated inside HSM **never** leave in plaintext
//! - Export only allowed if key marked "extractable" (rare)
//! - Backup/restore uses encrypted wrapping keys
//!
//! ## Audit Trail
//!
//! Every HSM operation is emitted through `tracing` (`info!`/`warn!`) from
//! [`backend`], so it lands in the service's normal log pipeline. There is no
//! `get_audit_logs()` accessor on the backend — the audit record lives in the
//! log sink, not in the HSM client.
//!
//! What the backend *can* tell you is which keys it is tracking:
//!
//! ```rust,no_run
//! use secreton_hsm::HsmBackend;
//!
//! # async fn example(hsm: HsmBackend) -> Result<(), Box<dyn std::error::Error>> {
//! for key in hsm.list_hsm_keys().await? {
//!     println!("{} ({} {} bits) created {}", key.key_id, key.algorithm, key.key_size, key.created_at);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # High Availability
//!
//! ## Clustered HSM
//!
//! Multiple HSMs for redundancy:
//!
//! ```yaml
//! hsm:
//!   primary: hsm1.kejaksaan.go.id:1792
//!   secondary: hsm2.kejaksaan.go.id:1792
//!   failover_timeout: 5s
//! ```
//!
//! ## Key Replication
//!
//! - Keys automatically replicated across cluster
//! - Synchronous replication for master keys
//! - Asynchronous for derived keys
//!
//! # Performance
//!
//! | Operation | Latency | Throughput |
//! |-----------|---------|------------|
//! | Key Generation | 50-200ms | 5-20/sec |
//! | Encryption | 1-5ms | 200-1000 ops/sec |
//! | Signing (Ed25519) | 2-10ms | 100-500 sigs/sec |
//! | Signing (RSA-4096) | 20-100ms | 10-50 sigs/sec |
//!
//! *Varies by HSM hardware*
//!
//! # Development Setup (SoftHSM2)
//!
//! For development/testing without physical HSM:
//!
//! ```bash
//! # Install SoftHSM2
//! apt-get install softhsm2
//!
//! # Initialize token
//! softhsm2-util --init-token --slot 0 --label "secreton-dev" --pin 1234 --so-pin 5678
//!
//! # Configure Secreton
//! export SECRETON_HSM_LIBRARY=/usr/lib/softhsm/libsofthsm2.so
//! export SECRETON_HSM_SLOT=0
//! export SECRETON_HSM_PIN=1234
//! ```
//!
//! # Production Deployment
//!
//! ## Network HSM Setup
//!
//! ```yaml
//! hsm:
//!   type: network  # vs. usb, pcie
//!   servers:
//!     - host: hsm1.internal.kejaksaan.go.id
//!       port: 1792
//!   ha_group: secreton-prod
//!   partition: kejaksaan-engine
//! ```
//!
//! ## Backup Strategy
//!
//! - **Daily**: Automated key backup to secondary HSM
//! - **Weekly**: Encrypted backup to offline storage
//! - **Disaster Recovery**: Key escrow with government authority
//!
//! # Troubleshooting
//!
//! ## HSM Not Responding
//!
//! ```rust,no_run
//! use secreton_hsm::HsmBackend;
//!
//! # async fn example(hsm: HsmBackend) -> Result<(), Box<dyn std::error::Error>> {
//! match hsm.health_check().await {
//!     Ok(_) => println!("HSM healthy"),
//!     Err(e) => {
//!         eprintln!("HSM error: {}", e);
//!         // Failover to secondary HSM
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Key Not Found
//!
//! - Check slot ID is correct
//! - Verify PIN/credentials
//! - Ensure key label matches
//! - Check key hasn't been deleted
//!
//! # See Also
//!
//! - [`HsmBackend`] - Main HSM interface
//! - [`HsmConfig`] - Configuration options
//! - [`Pkcs11Provider`] - PKCS#11 implementation
//! - `crate::crypto` - Software cryptography fallback
//! - `crate::security` - Security level enforcement

pub mod backend;
pub mod config;
pub mod error;
pub mod pkcs11;

pub use backend::HsmBackend;
pub use config::HsmConfig;
pub use error::{HsmError, HsmResult};
pub use pkcs11::Pkcs11Provider;
