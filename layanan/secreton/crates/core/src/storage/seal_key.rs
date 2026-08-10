//! Binds encryption at rest to the seal ceremony.
//!
//! [`SealMasterKeyProvider`] is the production [`MasterKeyProvider`]: it derives
//! the storage data-encryption key from the engine master key that
//! [`SealService`] reconstructs from unseal shares. That is the point of the
//! ceremony — while the engine is sealed the master key does not exist in
//! memory, so [`EncryptedStorage`](secreton_storage::EncryptedStorage) cannot
//! open or write secrets and secret I/O fails closed.
//!
//! The key is never cached here. Caching it would keep secrets readable across
//! a re-seal, which would make sealing cosmetic.

use async_trait::async_trait;
use secreton_storage::{MasterKeyProvider, StorageError, StorageResult};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use zeroize::Zeroizing;

use crate::services::seal::SealService;

/// Domain separation label. Changing it re-keys every entry, so it is versioned
/// alongside the envelope tag in `EncryptedStorage`.
const KDF_LABEL: &[u8] = b"secreton:storage:aes256gcm:v1";

/// Derives the storage key from the engine master key held by [`SealService`].
pub struct SealMasterKeyProvider {
    seal: Arc<SealService>,
}

impl SealMasterKeyProvider {
    /// Bind to a seal service.
    pub fn new(seal: Arc<SealService>) -> Self {
        Self { seal }
    }
}

#[async_trait]
impl MasterKeyProvider for SealMasterKeyProvider {
    async fn master_key(&self) -> StorageResult<Zeroizing<Vec<u8>>> {
        // Fails while sealed — that is the intended behaviour, not a fault.
        let master = Zeroizing::new(self.seal.get_master_key().await.map_err(|e| {
            StorageError::EncryptionFailed {
                message: format!("engine master key unavailable: {e}"),
            }
        })?);

        // SHA-256 to a fixed 32 bytes so the AES-256 key length never depends on
        // how the seal happens to size its master key, and to keep the storage
        // key distinct from the master key itself.
        let mut hasher = Sha256::new();
        hasher.update(KDF_LABEL);
        hasher.update(&*master);
        Ok(Zeroizing::new(hasher.finalize().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::seal::{SealConfig, SealService};

    #[tokio::test]
    async fn a_sealed_engine_yields_no_key() {
        let seal = Arc::new(SealService::new(SealConfig::default()));
        assert!(seal.is_sealed().await);

        let provider = SealMasterKeyProvider::new(seal);
        let err = provider.master_key().await.unwrap_err();

        assert!(
            matches!(err, StorageError::EncryptionFailed { .. }),
            "sealed engine must refuse to hand out a key, got {err:?}"
        );
    }

    #[test]
    fn the_derived_key_is_32_bytes_and_not_the_master_key() {
        let master = vec![0xABu8; 48];

        let mut hasher = Sha256::new();
        hasher.update(KDF_LABEL);
        hasher.update(&master);
        let derived = hasher.finalize().to_vec();

        assert_eq!(derived.len(), 32);
        assert_ne!(derived, master[..32].to_vec());
    }
}
