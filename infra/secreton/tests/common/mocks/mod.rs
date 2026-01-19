//! Test mocks for Secreton Enterprise Vault System
//! Provides mock implementations for testing various components

pub mod mfa_storage;
/// Mewakili pub `security_orchestrator`.
pub mod security_orchestrator;

// Re-export commonly used mocks
pub use mfa_storage::{MockMfaStorage, InMemoryMfaStorage};

// Additional mock utilities
/// Mewakili pub `mock_utils`.
pub mod mock_utils {
    use std::sync::Arc;

    /// Create a standard test MFA storage instance
    pub fn create_test_mfa_storage() -> Arc<super::InMemoryMfaStorage> {
        Arc::new(super::InMemoryMfaStorage::default())
    }
}
