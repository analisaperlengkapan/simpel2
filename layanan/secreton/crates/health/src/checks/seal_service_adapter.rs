//! Adapter to use SealService with SealStatusHealthCheck
//!
//! This module provides an implementation of SealStatusProvider for
//! secreton_core::services::seal::SealService, allowing the seal status
//! health check to work with the actual Secreton seal service.

use super::SealStatusProvider;
use async_trait::async_trait;

/// Adapter trait for SealService
///
/// This trait defines the minimal interface needed from SealService
/// to implement the health check. This allows the health check crate
/// to remain independent of the core crate.
#[async_trait]
pub trait SealServiceLike: Send + Sync {
    /// Returns true if the service is sealed
    async fn is_sealed(&self) -> bool;
}

/// Blanket implementation of SealStatusProvider for any SealServiceLike
#[async_trait]
impl<T: SealServiceLike> SealStatusProvider for T {
    async fn is_sealed(&self) -> bool {
        self.is_sealed().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockSealService {
        sealed: AtomicBool,
    }

    impl MockSealService {
        fn new(sealed: bool) -> Self {
            Self {
                sealed: AtomicBool::new(sealed),
            }
        }
    }

    #[async_trait]
    impl SealServiceLike for MockSealService {
        async fn is_sealed(&self) -> bool {
            self.sealed.load(Ordering::SeqCst)
        }
    }

    #[tokio::test]
    async fn test_seal_service_adapter() {
        let service = MockSealService::new(true);
        let provider: &dyn SealStatusProvider = &service;

        assert!(provider.is_sealed().await);
    }
}
