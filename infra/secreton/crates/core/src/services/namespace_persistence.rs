//! Namespace Persistence Service
//!
//! Provides functionality to persist and load namespace hierarchy from storage.
//! This ensures that namespace configuration survives application restarts.

use crate::namespace::NamespaceHierarchy;
use anyhow::{Context, Result};
use secreton_crypto::{AlgorithmId, CryptoEngine, EncryptedData, SecurityParams};
use secreton_storage::{SecretEntry, SecurityLevel, StorageBackend};
use std::sync::Arc;
use uuid::Uuid;

/// Path where the namespace hierarchy is stored in the backend
pub const NAMESPACE_STORAGE_PATH: &str = "sys/namespaces/hierarchy";

/// Hardcoded salt for namespace key derivation (fallback/legacy)
const NAMESPACE_KEY_SALT: &[u8] = b"secreton-namespace-persistence-salt";

/// Derive a consistent encryption key for namespace storage
fn derive_namespace_key(secret: &str, salt: &[u8]) -> Result<Vec<u8>> {
    // Use PBKDF2 to derive a 32-byte key from the secret
    // This ensures we have a valid key length for ChaCha20/AES
    let params = SecurityParams::new(AlgorithmId::Pbkdf2);

    let key = secreton_crypto::derive_key_pbkdf2(
        secret.as_bytes(),
        salt,
        params.iterations.unwrap_or(100_000),
        params.key_size,
    )
    .context("Failed to derive namespace key")?;

    Ok(key)
}

/// Save namespace hierarchy to storage
pub async fn save_hierarchy(
    hierarchy: &NamespaceHierarchy,
    storage: &Arc<dyn StorageBackend + Send + Sync>,
    crypto: &CryptoEngine,
    secret: &str,
) -> Result<()> {
    // Serialize hierarchy to JSON
    let json_bytes =
        serde_json::to_vec(hierarchy).context("Failed to serialize namespace hierarchy")?;

    // Generate random salt for KDF
    let salt = secreton_crypto::generate_random_bytes(16)?;

    // Derive encryption key with unique salt
    let key = derive_namespace_key(secret, &salt)?;

    // Encrypt data
    let encrypted = crypto
        .encrypt(AlgorithmId::ChaCha20Poly1305, &json_bytes, &key)
        .context("Failed to encrypt namespace hierarchy")?;

    // Serialize encrypted data structure
    let encrypted_bytes =
        serde_json::to_vec(&encrypted).context("Failed to serialize encrypted data")?;

    let existing = storage
        .get_by_path(NAMESPACE_STORAGE_PATH)
        .await
        .ok()
        .flatten();
    let entry_id = existing.map(|e| e.id).unwrap_or_else(Uuid::new_v4);

    let mut entry = SecretEntry::new(
        NAMESPACE_STORAGE_PATH.to_string(),
        encrypted_bytes,
        serde_json::json!({
            "algorithm": "chacha20-poly1305",
            "kdf": "pbkdf2",
            "type": "namespace_hierarchy",
            "salt": hex::encode(salt)
        }),
        SecurityLevel::Internal, // Namespaces are internal system data
        "system".to_string(),
    );

    // Set the ID to match existing if found
    entry.id = entry_id;

    // Persist to storage
    if storage
        .exists(NAMESPACE_STORAGE_PATH)
        .await
        .unwrap_or(false)
    {
        storage
            .update(&entry)
            .await
            .context("Failed to update namespace hierarchy in storage")?;
    } else {
        storage
            .store(&entry)
            .await
            .context("Failed to store namespace hierarchy")?;
    }

    Ok(())
}

/// Load namespace hierarchy from storage
pub async fn load_hierarchy(
    storage: &Arc<dyn StorageBackend + Send + Sync>,
    crypto: &CryptoEngine,
    secret: &str,
) -> Result<Option<NamespaceHierarchy>> {
    // Retrieve entry
    let entry = storage
        .get_by_path(NAMESPACE_STORAGE_PATH)
        .await
        .context("Failed to retrieve namespace hierarchy from storage")?;

    let entry = match entry {
        Some(e) => e,
        None => return Ok(None),
    };

    // Deserialize encrypted data structure
    let encrypted: EncryptedData = serde_json::from_slice(&entry.encrypted_data)
        .context("Failed to deserialize encrypted data structure")?;

    // Extract salt from metadata or use fallback
    let salt = if let Some(salt_hex) = entry
        .encryption_metadata
        .get("salt")
        .and_then(|v| v.as_str())
    {
        hex::decode(salt_hex).context("Failed to decode salt hex")?
    } else {
        NAMESPACE_KEY_SALT.to_vec()
    };

    // Derive encryption key
    let key = derive_namespace_key(secret, &salt)?;

    // Decrypt data
    let json_bytes = crypto
        .decrypt(&encrypted, &key)
        .context("Failed to decrypt namespace hierarchy")?;

    // Deserialize hierarchy
    let hierarchy: NamespaceHierarchy = serde_json::from_slice(&json_bytes)
        .context("Failed to deserialize namespace hierarchy JSON")?;

    Ok(Some(hierarchy))
}
