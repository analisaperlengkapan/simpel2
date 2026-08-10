//! Encryption at rest for any [`StorageBackend`].
//!
//! [`EncryptedStorage`] wraps another backend and seals
//! [`SecretEntry::encrypted_data`] under AES-256-GCM before it reaches the
//! inner backend, so a compromise of the storage medium alone — a stolen
//! `pg_dump`, a Velero restore, a detached PVC — yields ciphertext instead of
//! secrets.
//!
//! # This decorator used to be a lie
//!
//! Until it was fixed, every method here delegated straight to the inner
//! backend and encrypted nothing, while `get_stats` still reported
//! `encryption_enabled: true` and the gRPC write path stamped each row
//! `"algorithm": "aes-256-gcm"`. Secrets sat in `vault_entries.encrypted_data`
//! as plaintext JSON. The read paths proved it: both
//! `secreton_grpc::server::get_secret` and `SecretService::get_secret`
//! deserialized the stored bytes directly with `serde_json::from_slice`, which
//! no GCM ciphertext would ever survive.
//!
//! That is why [`encryption_at_rest_is_real`] exists below. It is not a
//! round-trip test — a pass-through passes those. It reaches past the wrapper
//! to the inner backend and asserts the plaintext is **not** in the stored
//! bytes.
//!
//! # Keys
//!
//! The key is fetched per operation from a [`MasterKeyProvider`] rather than
//! held in the struct, because the only key worth using is the engine's master
//! key and that key does not exist until the engine is unsealed — which happens
//! long after storage is constructed. A sealed engine therefore fails secret
//! reads and writes with [`StorageError::EncryptionFailed`], which is the
//! intended behaviour, not a regression.
//!
//! # Rows written before this existed
//!
//! Entries are tagged in [`SecretEntry::encryption_metadata`] with
//! `envelope: "aes256gcm-v1"`. An entry without that tag is a pre-encryption
//! row: it is returned as-is with a warning rather than failing the read, and
//! it becomes sealed the next time it is written. There is no key rotation yet
//! — `key_id` is recorded so a future re-wrap can tell generations apart.

use crate::{
    HealthStatus, QueryParams, SecretEntry, StorageBackend, StorageError, StorageResult,
    StorageStats, StorageTransaction,
};
use async_trait::async_trait;
use secreton_crypto::encryption::{Aes256GcmCipher, EncryptedData, SymmetricCipher};
use std::sync::Arc;
use uuid::Uuid;
use zeroize::Zeroizing;

/// Value written to `encryption_metadata.envelope` for entries sealed by this
/// decorator. Anything else is treated as a pre-encryption row.
const ENVELOPE_V1: &str = "aes256gcm-v1";

/// Supplies the 32-byte key used to seal entries.
///
/// Implementations resolve the key at call time. The production implementation
/// reads the engine master key, which is only available while unsealed.
#[async_trait]
pub trait MasterKeyProvider: Send + Sync {
    /// Return the current 32-byte data-encryption key.
    async fn master_key(&self) -> StorageResult<Zeroizing<Vec<u8>>>;
}

/// A [`StorageBackend`] that seals entry payloads before they reach the
/// backend it wraps.
pub struct EncryptedStorage {
    backend: Arc<dyn StorageBackend>,
    key_provider: Arc<dyn MasterKeyProvider>,
    key_id: String,
}

impl std::fmt::Debug for EncryptedStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedStorage")
            .field("key_id", &self.key_id)
            .finish_non_exhaustive()
    }
}

impl EncryptedStorage {
    /// Wrap `backend`, sealing payloads with keys from `key_provider`.
    ///
    /// `key_id` labels the key generation in each entry's metadata; it is
    /// recorded, not used to look the key up.
    pub fn new(
        backend: Arc<dyn StorageBackend>,
        key_provider: Arc<dyn MasterKeyProvider>,
        key_id: impl Into<String>,
    ) -> Self {
        Self {
            backend,
            key_provider,
            key_id: key_id.into(),
        }
    }

    /// The backend being wrapped.
    pub fn backend(&self) -> &Arc<dyn StorageBackend> {
        &self.backend
    }

    /// The key generation label recorded on entries this decorator writes.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    async fn key(&self) -> StorageResult<Zeroizing<Vec<u8>>> {
        let key = self.key_provider.master_key().await?;
        if key.len() != 32 {
            return Err(StorageError::EncryptionFailed {
                message: format!(
                    "master key must be 32 bytes for AES-256-GCM, provider returned {}",
                    key.len()
                ),
            });
        }
        Ok(key)
    }

    /// Encrypt `entry.encrypted_data` and stamp the envelope metadata.
    async fn seal(&self, entry: &SecretEntry) -> StorageResult<SecretEntry> {
        let key = self.key().await?;
        let sealed = Aes256GcmCipher
            .encrypt(&entry.encrypted_data, &key)
            .map_err(|e| StorageError::EncryptionFailed {
                message: format!("failed to seal entry at path '{}': {}", entry.path, e),
            })?;

        let blob = serde_json::to_vec(&sealed).map_err(|e| StorageError::SerializationError {
            message: format!("failed to serialize sealed entry: {}", e),
            source: Some(Box::new(e)),
        })?;

        let mut out = entry.clone();
        out.encrypted_data = blob;
        out.encryption_metadata = serde_json::json!({
            "envelope": ENVELOPE_V1,
            "algorithm": "aes-256-gcm",
            "key_id": self.key_id,
        });
        Ok(out)
    }

    /// Decrypt an entry this decorator sealed. Pre-encryption rows pass through.
    async fn open(&self, entry: SecretEntry) -> StorageResult<SecretEntry> {
        let envelope = entry
            .encryption_metadata
            .get("envelope")
            .and_then(|v| v.as_str());

        if envelope != Some(ENVELOPE_V1) {
            tracing::warn!(
                path = %entry.path,
                "entry is not sealed (no {ENVELOPE_V1} envelope) — written before \
                 encryption at rest existed; it will be sealed on next write"
            );
            return Ok(entry);
        }

        let key = self.key().await?;
        let sealed: EncryptedData = serde_json::from_slice(&entry.encrypted_data).map_err(|e| {
            StorageError::SerializationError {
                message: format!(
                    "entry at path '{}' is tagged {ENVELOPE_V1} but its payload is not an \
                     encryption envelope: {}",
                    entry.path, e
                ),
                source: Some(Box::new(e)),
            }
        })?;

        let plaintext =
            Aes256GcmCipher
                .decrypt(&sealed, &key)
                .map_err(|e| StorageError::EncryptionFailed {
                    message: format!("failed to open entry at path '{}': {}", entry.path, e),
                })?;

        let mut out = entry;
        out.encrypted_data = plaintext;
        Ok(out)
    }
}

#[async_trait]
impl StorageBackend for EncryptedStorage {
    async fn store(&self, entry: &SecretEntry) -> StorageResult<()> {
        let sealed = self.seal(entry).await?;
        self.backend.store(&sealed).await
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<SecretEntry>> {
        match self.backend.get_by_id(id).await? {
            Some(entry) => Ok(Some(self.open(entry).await?)),
            None => Ok(None),
        }
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<SecretEntry>> {
        match self.backend.get_by_path(path).await? {
            Some(entry) => Ok(Some(self.open(entry).await?)),
            None => Ok(None),
        }
    }

    async fn update(&self, entry: &SecretEntry) -> StorageResult<()> {
        let sealed = self.seal(entry).await?;
        self.backend.update(&sealed).await
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        self.backend.delete_by_id(id).await
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        self.backend.delete_by_path(path).await
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        self.backend.count(params).await
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        self.backend.exists(path).await
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        // Without this wrapper a caller could reach the inner backend through a
        // transaction and write plaintext past the decorator.
        Ok(Box::new(EncryptedTransaction {
            inner: self.backend.begin_transaction().await?,
            key_provider: self.key_provider.clone(),
            key_id: self.key_id.clone(),
        }))
    }

    async fn migrate(&self) -> StorageResult<()> {
        self.backend.migrate().await
    }

    async fn delete_expired(&self) -> StorageResult<u64> {
        self.backend.delete_expired().await
    }

    async fn compact(&self) -> StorageResult<()> {
        self.backend.compact().await
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<SecretEntry>> {
        let entries = self.backend.list(params).await?;
        let mut opened = Vec::with_capacity(entries.len());
        for entry in entries {
            opened.push(self.open(entry).await?);
        }
        Ok(opened)
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        self.backend.health_check().await
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let mut stats = self.backend.get_stats().await?;

        if let serde_json::Value::Object(ref mut map) = stats.metadata {
            map.insert("encryption_enabled".to_string(), serde_json::json!(true));
            map.insert(
                "encryption_key_id".to_string(),
                serde_json::json!(self.key_id),
            );
        }

        Ok(stats)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Transaction wrapper that seals writes on their way to the inner transaction.
struct EncryptedTransaction {
    inner: Box<dyn StorageTransaction>,
    key_provider: Arc<dyn MasterKeyProvider>,
    key_id: String,
}

impl EncryptedTransaction {
    async fn seal(&self, entry: &SecretEntry) -> StorageResult<SecretEntry> {
        let key = self.key_provider.master_key().await?;
        if key.len() != 32 {
            return Err(StorageError::EncryptionFailed {
                message: format!(
                    "master key must be 32 bytes for AES-256-GCM, provider returned {}",
                    key.len()
                ),
            });
        }

        let sealed = Aes256GcmCipher
            .encrypt(&entry.encrypted_data, &key)
            .map_err(|e| StorageError::EncryptionFailed {
                message: format!("failed to seal entry at path '{}': {}", entry.path, e),
            })?;

        let blob = serde_json::to_vec(&sealed).map_err(|e| StorageError::SerializationError {
            message: format!("failed to serialize sealed entry: {}", e),
            source: Some(Box::new(e)),
        })?;

        let mut out = entry.clone();
        out.encrypted_data = blob;
        out.encryption_metadata = serde_json::json!({
            "envelope": ENVELOPE_V1,
            "algorithm": "aes-256-gcm",
            "key_id": self.key_id,
        });
        Ok(out)
    }
}

#[async_trait]
impl StorageTransaction for EncryptedTransaction {
    async fn store(&mut self, entry: &SecretEntry) -> StorageResult<()> {
        let sealed = self.seal(entry).await?;
        self.inner.store(&sealed).await
    }

    async fn update(&mut self, entry: &SecretEntry) -> StorageResult<()> {
        let sealed = self.seal(entry).await?;
        self.inner.update(&sealed).await
    }

    async fn delete(&mut self, id: Uuid) -> StorageResult<bool> {
        self.inner.delete(id).await
    }

    async fn commit(self: Box<Self>) -> StorageResult<()> {
        self.inner.commit().await
    }

    async fn rollback(self: Box<Self>) -> StorageResult<()> {
        self.inner.rollback().await
    }
}

/// A fixed key, for tests and for callers that manage key material themselves.
pub struct StaticMasterKey(Zeroizing<Vec<u8>>);

impl StaticMasterKey {
    /// Build a provider over a 32-byte key.
    pub fn new(key: Vec<u8>) -> Self {
        Self(Zeroizing::new(key))
    }
}

#[async_trait]
impl MasterKeyProvider for StaticMasterKey {
    async fn master_key(&self) -> StorageResult<Zeroizing<Vec<u8>>> {
        Ok(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryBackend, SecurityLevel};
    use chrono::Utc;

    const PLAINTEXT: &[u8] = br#"{"password":"correct-horse-battery-staple"}"#;

    fn test_key() -> Arc<dyn MasterKeyProvider> {
        Arc::new(StaticMasterKey::new(vec![7u8; 32]))
    }

    fn create_test_entry(path: &str) -> SecretEntry {
        SecretEntry {
            id: Uuid::new_v4(),
            path: path.to_string(),
            encrypted_data: PLAINTEXT.to_vec(),
            encryption_metadata: serde_json::json!({}),
            security_level: SecurityLevel::Confidential,
            metadata: serde_json::json!({}),
            tags: vec![],
            version: 1,
            owner_id: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        }
    }

    /// The test the pass-through version could never have failed.
    ///
    /// A round-trip assertion passes whether or not anything is encrypted, which
    /// is exactly how the plaintext-at-rest bug survived. This one looks at what
    /// the INNER backend actually holds.
    #[tokio::test]
    async fn encryption_at_rest_is_real() {
        let inner = Arc::new(MemoryBackend::new());
        let storage = EncryptedStorage::new(inner.clone(), test_key(), "test-key");

        let entry = create_test_entry("kv/db/password");
        storage.store(&entry).await.unwrap();

        let stored = inner
            .get_by_path("kv/db/password")
            .await
            .unwrap()
            .expect("entry reached the inner backend");

        assert_ne!(
            stored.encrypted_data, PLAINTEXT,
            "payload reached storage unencrypted"
        );
        assert!(
            !stored
                .encrypted_data
                .windows(PLAINTEXT.len())
                .any(|w| w == PLAINTEXT),
            "plaintext is embedded in the stored bytes"
        );
        assert_eq!(
            stored.encryption_metadata.get("envelope").unwrap(),
            ENVELOPE_V1
        );
    }

    #[tokio::test]
    async fn round_trip_returns_the_original_payload() {
        let inner = Arc::new(MemoryBackend::new());
        let storage = EncryptedStorage::new(inner, test_key(), "test-key");

        let entry = create_test_entry("kv/db/password");
        storage.store(&entry).await.unwrap();

        let by_path = storage
            .get_by_path("kv/db/password")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(by_path.encrypted_data, PLAINTEXT);

        let by_id = storage.get_by_id(entry.id).await.unwrap().unwrap();
        assert_eq!(by_id.encrypted_data, PLAINTEXT);
    }

    #[tokio::test]
    async fn a_different_key_cannot_open_the_entry() {
        let inner = Arc::new(MemoryBackend::new());
        let writer = EncryptedStorage::new(inner.clone(), test_key(), "test-key");
        writer.store(&create_test_entry("kv/x")).await.unwrap();

        let attacker = EncryptedStorage::new(
            inner,
            Arc::new(StaticMasterKey::new(vec![9u8; 32])),
            "wrong-key",
        );

        let err = attacker.get_by_path("kv/x").await.unwrap_err();
        assert!(
            matches!(err, StorageError::EncryptionFailed { .. }),
            "expected EncryptionFailed, got {err:?}"
        );
    }

    /// Rows written before encryption existed must stay readable, or turning
    /// this on would strand every secret already in staging.
    #[tokio::test]
    async fn pre_encryption_rows_still_read() {
        let inner = Arc::new(MemoryBackend::new());
        let legacy = create_test_entry("kv/legacy");
        inner.store(&legacy).await.unwrap();

        let storage = EncryptedStorage::new(inner.clone(), test_key(), "test-key");
        let read = storage.get_by_path("kv/legacy").await.unwrap().unwrap();
        assert_eq!(read.encrypted_data, PLAINTEXT);

        // ...and writing it back through the decorator seals it.
        storage.store(&read).await.unwrap();
        let now_stored = inner.get_by_path("kv/legacy").await.unwrap().unwrap();
        assert_ne!(now_stored.encrypted_data, PLAINTEXT);
    }

    #[tokio::test]
    async fn list_decrypts_every_entry() {
        let inner = Arc::new(MemoryBackend::new());
        let storage = EncryptedStorage::new(inner, test_key(), "test-key");

        storage.store(&create_test_entry("kv/a")).await.unwrap();
        storage.store(&create_test_entry("kv/b")).await.unwrap();

        let listed = storage.list(&QueryParams::default()).await.unwrap();
        assert_eq!(listed.len(), 2);
        for entry in listed {
            assert_eq!(entry.encrypted_data, PLAINTEXT);
        }
    }

    #[tokio::test]
    async fn a_short_key_is_refused_rather_than_padded() {
        let inner = Arc::new(MemoryBackend::new());
        let storage = EncryptedStorage::new(
            inner,
            Arc::new(StaticMasterKey::new(vec![1u8; 16])),
            "short-key",
        );

        let err = storage.store(&create_test_entry("kv/x")).await.unwrap_err();
        assert!(matches!(err, StorageError::EncryptionFailed { .. }));
    }
}
