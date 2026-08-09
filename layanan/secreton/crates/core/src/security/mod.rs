//! Security primitives: FIPS posture, seal wrapping, and a tamper-evident audit
//! chain.
//!
//! Read the module list below before reaching for anything here — two of the
//! larger pieces are **not compiled into the crate** and one is compiled but not
//! wired to any call path.
//!
//! | Module | State |
//! |---|---|
//! | [`fips_compliance`] | live — [`fips_compliance::FipsLevel`], the [`fips_compliance::FipsCompliant`] trait |
//! | [`seal_wrapping`] | live — [`seal_wrapping::SealWrappingEngine`] and the [`seal_wrapping::SealProvider`] trait |
//! | [`audit`] | compiles, but only its property tests call it; the running audit path is [`crate::audit`] |
//! | [`optimized_traits`] | live |
//! | `manager` (`SecurityManager`) | **commented out** in this file — not part of the crate |
//! | `concrete_implementations` | **commented out** in this file — not part of the crate |
//!
//! There is no `SecurityLevel` in this module. The classification enum lives in
//! `secreton_types::security` and, for audit records, in
//! [`crate::models::audit::SecurityLevel`].
//!
//! # Seal wrapping
//!
//! [`seal_wrapping::SealWrappingEngine`] wraps critical values under one or more
//! seal providers before they reach storage, so a storage compromise alone does
//! not yield plaintext. Providers are registered with a priority and the engine
//! fails over between them.
//!
//! ```rust
//! use secreton_core::security::seal_wrapping::SealWrappingEngine;
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let engine = SealWrappingEngine::new().await.unwrap();
//!
//! engine.add_provider("shamir".to_string(), 10).await.unwrap();
//! engine.add_provider("kms".to_string(), 20).await.unwrap();
//!
//! // Multi-seal is opt-in; a fresh engine wraps under a single provider.
//! let config = engine.get_multi_seal_config().await;
//! assert!(!config.enabled);
//! # });
//! ```
//!
//! [`seal_wrapping::DataType`] selects the per-kind [`seal_wrapping::WrapConfig`]
//! — `MasterKey` and `RootKey` are wrapped more strictly than `Configuration`.
//!
//! # HSM
//!
//! HSM support is a separate crate, `secreton_hsm` — `HsmBackend`, `HsmConfig`
//! and `Pkcs11Provider`. Nothing named `HsmProvider` exists.
//!
//! # FIPS
//!
//! [`fips_compliance::FipsLevel`] records the target level and
//! [`fips_compliance::FipsCompliant`] is the trait a component implements to
//! declare its posture. Declaring a level is not the same as being validated
//! against it: no module here has been through CMVP.

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
