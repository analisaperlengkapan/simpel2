//! Plain Shamir Secret Sharing (HashiCorp Vault Style)
//!
//! This module re-exports the shared implementation from lib_common.
//! See lib_common::crypto::shamir for the core implementation.

// Re-export everything from lib_common
pub use lib_common::crypto::shamir::*;
